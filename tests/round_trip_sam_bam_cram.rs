// Round-trip integration tests for SAM/BAM/CRAM.
//
// Each test reads an input file, writes it out in a different format, then
// reads back from the round-tripped output and compares the alignments
// record-by-record against the originals.  CRAM intentionally re-derives MD/NM
// from the reference, so for round trips that touch CRAM we strip those tags
// before comparing the formatted record text (the same canonicalisation other
// tests in this crate apply to CRAM aux records).

use htslib_rs::{
    bam_destroy1, bam_init1, hts_close, hts_open, hts_opt_add, hts_opt_apply, hts_opt_free,
    hts_set_fai_filename, kstring_t, sam_c_4553_sam_write1, sam_format1, sam_hdr_destroy,
    sam_hdr_read, sam_hdr_write, sam_read1,
};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

fn c_fixture(path: &str) -> CString {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn c_path(path: &Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn tmp_path(label: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-round-trip-{}-{}.{}",
        std::process::id(),
        label,
        ext
    ))
}

/// Read every alignment record from `path` and return its `sam_format1`
/// rendering.  The reference (if any) is required for CRAM inputs.
unsafe fn collect_formatted_records(path: &Path, reference: Option<&str>) -> Vec<String> {
    let c_path = c_path(path);
    let fp = hts_open(c_path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read header from {}", path.display());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut line = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let mut out = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(sam_format1(hdr, rec, &mut line) >= 0);
        out.push(CStr::from_ptr(line.s).to_string_lossy().into_owned());
    }

    if !line.s.is_null() {
        libc::free(line.s.cast());
    }
    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

/// Copy every alignment record from `input` to `output`, opening `output`
/// with `out_mode` (e.g. `b"wb"`, `b"wc"`, `b"w"`).
///
/// `out_opts` are passed verbatim to `hts_opt_add` on the output handle.
/// For CRAM that means strings like `"embed_ref=2"`, which is required for
/// round trips where the input BAM may contain bases that diverge from the
/// supplied reference (CRAM otherwise stores a delta and faithfully restores
/// the reference base on read-back, which is "correct" for the format but
/// not what a round-trip parity check wants).
unsafe fn copy_alignments(
    input: &Path,
    output: &Path,
    out_mode: &CStr,
    reference: Option<&str>,
    out_opts: &[&str],
) {
    let in_path = c_path(input);
    let out_path = c_path(output);

    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", input.display());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(in_fp, reference.as_ptr()), 0);
    }
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null(), "failed to read header from {}", input.display());

    let out_fp = hts_open(out_path.as_ptr(), out_mode.as_ptr());
    assert!(!out_fp.is_null(), "failed to create {}", output.display());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(out_fp, reference.as_ptr()), 0);
    }
    if !out_opts.is_empty() {
        let mut opts: *mut htslib_rs::hts_opt = std::ptr::null_mut();
        for opt in out_opts {
            let opt_c = CString::new(*opt).unwrap();
            assert_eq!(hts_opt_add(&mut opts, opt_c.as_ptr()), 0, "bad option {opt}");
        }
        assert_eq!(hts_opt_apply(out_fp, opts), 0);
        hts_opt_free(opts);
    }
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
}

/// Canonicalise a SAM record for CRAM round-trip comparison.
///
/// CRAM re-derives MD/NM from the reference and re-emits the aux block in
/// its own internal order (notably hoisting RG ahead of user tags).  This
/// matches the canonicalisation strategy used by the existing
/// `real_data_record_more.rs` tests: drop MD/NM and sort the remaining aux
/// tags alphabetically.
fn canonical_cram_record(line: &str) -> String {
    let mut fields: Vec<String> = line.split('\t').map(ToOwned::to_owned).collect();
    if fields.len() <= 11 {
        return line.to_owned();
    }
    let mut aux: Vec<String> = fields
        .drain(11..)
        .filter(|f| !f.starts_with("MD:Z:") && !f.starts_with("NM:i:"))
        .collect();
    aux.sort();
    fields.extend(aux);
    fields.join("\t")
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
}

// ----- 1. SAM -> BAM -> SAM -------------------------------------------------

#[test]
fn round_trip_sam_to_bam_preserves_formatted_records() {
    unsafe {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/auxf#values.sam");
        let bam = tmp_path("sam-bam-auxf", "bam");
        cleanup(&bam);

        let original = collect_formatted_records(&input, None);
        assert!(!original.is_empty(), "input SAM was empty");

        copy_alignments(&input, &bam, c"wb", None, &[]);
        let round_tripped = collect_formatted_records(&bam, None);

        assert_eq!(round_tripped, original, "SAM->BAM->SAM record parity");
        cleanup(&bam);
    }
}

// ----- 2. BAM -> CRAM -> BAM ------------------------------------------------

#[test]
fn round_trip_bam_to_cram_preserves_formatted_records() {
    unsafe {
        // range.bam is the only ce-referenced BAM in the htslib test corpus.
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/range.bam");
        let cram = tmp_path("bam-cram-range", "cram");
        let bam = tmp_path("bam-cram-range-back", "bam");
        cleanup(&cram);
        cleanup(&bam);

        let original = collect_formatted_records(&input, None);
        assert!(!original.is_empty(), "input BAM was empty");

        // BAM -> CRAM with no_ref=1: sequences are stored verbatim and CRAM
        // does not compute MD/NM from a reference, so the round trip
        // preserves the original BAM bases bit-for-bit (range.bam has reads
        // whose bases diverge from ce.fa).
        copy_alignments(
            &input,
            &cram,
            c"wc",
            Some("htslib/test/ce.fa"),
            &["no_ref=1", "store_md=0", "store_nm=0"],
        );
        // CRAM -> BAM (back to a lossless container)
        copy_alignments(&cram, &bam, c"wb", Some("htslib/test/ce.fa"), &[]);

        let round_tripped = collect_formatted_records(&bam, None);

        let original_canon: Vec<String> = original.iter().map(|l| canonical_cram_record(l)).collect();
        let round_canon: Vec<String> = round_tripped.iter().map(|l| canonical_cram_record(l)).collect();
        assert_eq!(
            round_canon.len(),
            original_canon.len(),
            "BAM->CRAM->BAM record count parity",
        );
        assert_eq!(round_canon, original_canon, "BAM->CRAM->BAM record parity (MD/NM stripped)");

        cleanup(&cram);
        cleanup(&bam);
    }
}

// ----- 3. SAM -> CRAM -> SAM ------------------------------------------------

#[test]
fn round_trip_sam_to_cram_preserves_formatted_records() {
    unsafe {
        // auxf#values.sam pairs with auxf.fa (single contig "Sheila").
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/auxf#values.sam");
        let cram = tmp_path("sam-cram-auxf", "cram");
        let sam = tmp_path("sam-cram-auxf-back", "sam");
        cleanup(&cram);
        cleanup(&sam);

        let original = collect_formatted_records(&input, None);
        assert!(!original.is_empty(), "input SAM was empty");

        copy_alignments(
            &input,
            &cram,
            c"wc",
            Some("htslib/test/auxf.fa"),
            &["embed_ref=2"],
        );
        copy_alignments(&cram, &sam, c"w", Some("htslib/test/auxf.fa"), &[]);

        let round_tripped = collect_formatted_records(&sam, None);

        let original_canon: Vec<String> = original.iter().map(|l| canonical_cram_record(l)).collect();
        let round_canon: Vec<String> = round_tripped.iter().map(|l| canonical_cram_record(l)).collect();
        assert_eq!(round_canon.len(), original_canon.len());
        assert_eq!(round_canon, original_canon, "SAM->CRAM->SAM record parity (MD/NM stripped)");

        cleanup(&cram);
        cleanup(&sam);
    }
}

// ----- 4. Multi-reference round trip ---------------------------------------

#[test]
fn round_trip_multi_reference_sam_to_bam_preserves_records() {
    // index.sam contains 7 contigs (CHROMOSOME_I..CHROMOSOME_MtDNA) and 180+
    // alignment records — a meaningful multi-reference exercise.
    unsafe {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/index.sam");
        let bam = tmp_path("multi-ref-bam", "bam");
        let sam = tmp_path("multi-ref-sam-back", "sam");
        cleanup(&bam);
        cleanup(&sam);

        let original = collect_formatted_records(&input, None);
        assert!(original.len() > 100, "index.sam should hold many records");

        copy_alignments(&input, &bam, c"wb", None, &[]);
        copy_alignments(&bam, &sam, c"w", None, &[]);

        let round_tripped = collect_formatted_records(&sam, None);
        assert_eq!(round_tripped, original, "multi-ref SAM->BAM->SAM parity");

        // Spot-check that records from multiple distinct contigs survived.
        let contigs: std::collections::BTreeSet<&str> = round_tripped
            .iter()
            .filter_map(|line| line.split('\t').nth(2))
            .collect();
        assert!(
            contigs.len() >= 2,
            "expected multiple contigs in round-tripped multi-ref data, got {contigs:?}",
        );

        cleanup(&bam);
        cleanup(&sam);
    }
}
