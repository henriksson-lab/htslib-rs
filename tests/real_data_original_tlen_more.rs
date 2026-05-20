use htslib_mini_rs::{
    bam_destroy1, bam_init1, hts_close, hts_open, kstring_t, sam_format1, sam_hdr_destroy,
    sam_hdr_read, sam_read1,
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
