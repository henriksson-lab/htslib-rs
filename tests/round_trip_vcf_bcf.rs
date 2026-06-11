// Round-trip integration tests for VCF/BCF.
//
// Each test reads an input variant file, writes it back in the other format,
// reopens the output, and verifies that the rendered records (header text
// plus per-record `vcf_format` text) match the original record-for-record.
//
// This mirrors the canonical comparison strategy used elsewhere in the test
// suite: BCF is the binary serialisation of the VCF text model, so equality
// is best asserted on the rendered text rather than on the bit-level binary
// blob (which is sensitive to header re-emission order).

use htslib_rs::{
    bcf_destroy, bcf_hdr_destroy, bcf_hdr_write, bcf_init, bcf_read, bcf_subset_format, bcf_write,
    hts_close, hts_open, ks_free, kstring_t, vcf_format, vcf_hdr_read, vcf_read, vcf_write,
};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_path(path: &Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn tmp_path(label: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-round-trip-vcf-{}-{}.{}",
        std::process::id(),
        label,
        ext
    ))
}

/// Render an open variant file as `<header text><record1>\n<record2>\n...`.
unsafe fn render_variant_file(path: &Path) -> Vec<String> {
    let path_c = c_path(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());

    let hdr = vcf_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        path.display()
    );

    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut line = kstring_t::default();

    let mut out = Vec::new();
    loop {
        let ret = vcf_read(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert_eq!(bcf_subset_format(hdr, rec), 0);
        line.data.clear();
        assert_eq!(vcf_format(hdr, rec, &mut line), 0);
        let mut text = String::from_utf8_lossy(&line.data).into_owned();
        // vcf_format always terminates with '\n'; trim so per-record comparison
        // is line-oriented.
        if text.ends_with('\n') {
            text.pop();
        }
        out.push(text);
    }

    ks_free(&mut line);
    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

/// Copy every record from `input` to `output`.  `out_mode` selects the wire
/// format ("wb" = BCF, "w" = plain VCF, "wz" = bgzipped VCF).  Uses
/// vcf_read/bcf_write (or vice versa) — the actual choice does not matter
/// because the bcf1_t representation is shared between the two readers.
unsafe fn copy_variants(input: &Path, output: &Path, out_mode: &CStr) {
    let in_c = c_path(input);
    let out_c = c_path(output);

    let in_fp = hts_open(in_c.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", input.display());
    let hdr = vcf_hdr_read(in_fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        input.display()
    );

    let out_fp = hts_open(out_c.as_ptr(), out_mode.as_ptr());
    assert!(!out_fp.is_null(), "failed to create {}", output.display());
    assert_eq!(bcf_hdr_write(out_fp, hdr), 0);

    let rec = bcf_init();
    assert!(!rec.is_null());
    let is_text_out = matches!(out_mode.to_bytes(), b"w" | b"wz");
    loop {
        let ret = bcf_read(in_fp, hdr, rec);
        if ret < 0 {
            break;
        }
        let wret = if is_text_out {
            vcf_write(out_fp, hdr, rec)
        } else {
            bcf_write(out_fp, hdr, rec)
        };
        assert_eq!(wret, 0, "write failed for {}", output.display());
    }

    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
}

// ----- 1. VCF -> BCF -> VCF -------------------------------------------------

#[test]
fn round_trip_vcf_to_bcf_preserves_records() {
    unsafe {
        let input = fixture("htslib/test/formatcols.vcf");
        let bcf = tmp_path("vcf-bcf-formatcols", "bcf");
        cleanup(&bcf);

        let original = render_variant_file(&input);
        assert!(!original.is_empty(), "input VCF had no records");

        copy_variants(&input, &bcf, c"wb");
        let round_tripped = render_variant_file(&bcf);

        assert_eq!(round_tripped, original, "VCF->BCF->VCF record parity");
        cleanup(&bcf);
    }
}

#[test]
fn round_trip_vcf_to_bcf_handles_realistic_multi_sample_file() {
    // index.vcf has 720+ records across 23 contigs with multiple samples and
    // a dense set of INFO/FORMAT tags — a more demanding round trip.
    unsafe {
        let input = fixture("htslib/test/index.vcf");
        let bcf = tmp_path("vcf-bcf-index", "bcf");
        cleanup(&bcf);

        let original = render_variant_file(&input);
        assert!(original.len() > 100, "index.vcf should hold many records");

        copy_variants(&input, &bcf, c"wb");
        let round_tripped = render_variant_file(&bcf);

        assert_eq!(round_tripped.len(), original.len());
        assert_eq!(round_tripped, original, "index.vcf VCF->BCF->VCF parity");

        // Spot-check: at least two distinct contigs survived.
        let contigs: std::collections::BTreeSet<&str> = round_tripped
            .iter()
            .filter_map(|line| line.split('\t').next())
            .collect();
        assert!(
            contigs.len() >= 2,
            "expected multiple contigs, got {contigs:?}",
        );

        cleanup(&bcf);
    }
}

// ----- 2. BCF -> VCF -> BCF -------------------------------------------------

#[test]
fn round_trip_bcf_to_vcf_preserves_records() {
    unsafe {
        let input = fixture("htslib/test/tabix/vcf_file.bcf");
        let vcf = tmp_path("bcf-vcf-tabix", "vcf");
        let bcf = tmp_path("bcf-vcf-tabix-back", "bcf");
        cleanup(&vcf);
        cleanup(&bcf);

        let original = render_variant_file(&input);
        assert!(!original.is_empty(), "input BCF had no records");

        copy_variants(&input, &vcf, c"w");
        copy_variants(&vcf, &bcf, c"wb");
        let round_tripped = render_variant_file(&bcf);

        assert_eq!(round_tripped, original, "BCF->VCF->BCF record parity");

        cleanup(&vcf);
        cleanup(&bcf);
    }
}
