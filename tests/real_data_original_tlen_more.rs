use htslib_rs::{
    bam_destroy1, bam_get_qname, bam_init1, hts_close, hts_open, kstring_t, sam_c_4553_sam_write1,
    sam_format1, sam_hdr_destroy, sam_hdr_read, sam_hdr_write, sam_read1,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn formatted_alignment_records(path: &str) -> Vec<String> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        path.to_string_lossy()
    );

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

    libc::free(line.s.cast());
    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn assert_cram_matches_original_tlen_sam(stem: &str) {
    let expected = formatted_alignment_records(&format!("htslib/test/tlen/{stem}.sam"));
    let actual = formatted_alignment_records(&format!("htslib/test/tlen/{stem}.cram"));
    assert_eq!(actual, expected, "tlen fixture {stem}");
}

unsafe fn qname_isize_pairs(path: &str) -> Vec<(String, i64)> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut pairs = Vec::new();
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        pairs.push((
            CStr::from_ptr(bam_get_qname(rec))
                .to_string_lossy()
                .into_owned(),
            (*rec).core.isize,
        ));
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    pairs
}

unsafe fn copy_alignment_to_cram(input_path: &str, output_name: &str) -> std::path::PathBuf {
    let in_path = c_fixture(input_path);
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {input_path}");
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null(), "failed to read header from {input_path}");

    let out_path = std::env::temp_dir().join(format!(
        "htslib-rs-{output_name}-{}-{}.cram",
        std::process::id(),
        line!()
    ));
    let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(out_path_c.as_ptr(), c"wc".as_ptr());
    assert!(!out_fp.is_null(), "failed to open temp CRAM output");
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);
    out_path
}

#[test]
fn original_tlen_pairs_with_different_starts_and_ends_match_expected_sam() {
    unsafe {
        for stem in ["a7", "a7b", "a8", "a8b", "a9", "a9b"] {
            assert_cram_matches_original_tlen_sam(stem);
        }
    }
}

#[test]
fn original_tlen_pairs_with_matching_starts_and_ends_match_expected_sam() {
    unsafe {
        for stem in ["d7", "d7b"] {
            assert_cram_matches_original_tlen_sam(stem);
        }
    }
}

#[test]
fn original_tlen_pairs_with_matching_starts_and_different_ends_match_expected_sam() {
    unsafe {
        for stem in ["b7", "b7b", "b8", "b8b"] {
            assert_cram_matches_original_tlen_sam(stem);
        }
    }
}

#[test]
fn original_tlen_pairs_with_different_starts_and_matching_ends_match_expected_sam() {
    unsafe {
        for stem in ["c7", "c7b", "c8", "c8b"] {
            assert_cram_matches_original_tlen_sam(stem);
        }
    }
}

#[test]
fn original_tlen_triplets_with_matching_starts_and_ends_match_expected_sam() {
    unsafe {
        for stem in [
            "d4", "d4b", "d4c", "d4d", "d4e", "d4f", "d5", "d5b", "d5c", "d5d", "d5e", "d5f",
        ] {
            assert_cram_matches_original_tlen_sam(stem);
        }
    }
}

#[test]
fn original_tlen_triplets_with_different_starts_and_ends_match_expected_sam() {
    unsafe {
        for stem in ["a4", "a5"] {
            assert_cram_matches_original_tlen_sam(stem);
        }
    }
}

#[test]
fn original_tlen_explicit_nonstandard_values_survive_cram_roundtrip() {
    unsafe {
        let cram = copy_alignment_to_cram("htslib/test/xx#tlen.sam", "explicit-tlen");
        let expected = formatted_alignment_records("htslib/test/xx#tlen.sam");
        let actual = formatted_alignment_records(cram.to_str().unwrap());
        assert_eq!(actual, expected);
        let _ = std::fs::remove_file(cram);
    }
}

#[test]
fn original_tlen_cram_decodes_representative_signed_template_lengths() {
    unsafe {
        assert_eq!(
            qname_isize_pairs("htslib/test/tlen/a9b.cram"),
            vec![
                ("seq".to_string(), -14),
                ("seq".to_string(), 14),
                ("seqr".to_string(), -14),
                ("seqr".to_string(), 14),
            ]
        );
        assert_eq!(
            qname_isize_pairs("htslib/test/tlen/d5e.cram"),
            vec![
                ("seq".to_string(), 14),
                ("seq".to_string(), -14),
                ("seq".to_string(), -14),
            ]
        );
        assert_eq!(
            qname_isize_pairs("htslib/test/xx#tlen.sam"),
            vec![
                ("x1".to_string(), 20),
                ("x1".to_string(), -20),
                ("x2".to_string(), 8),
                ("x2".to_string(), -8),
                ("x3".to_string(), 8),
                ("x3".to_string(), -8),
                ("x4".to_string(), 20),
                ("x4".to_string(), -20),
                ("y1".to_string(), 20),
                ("y1".to_string(), -20),
                ("y2".to_string(), 8),
                ("y2".to_string(), -8),
                ("y3".to_string(), -2),
                ("y3".to_string(), 2),
                ("y4".to_string(), 10),
                ("y4".to_string(), -10),
            ]
        );
    }
}
