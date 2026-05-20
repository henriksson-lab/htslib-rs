use htslib_mini_rs::{
    hts_close, hts_open, hts_pos_t, sam_hdr_destroy, sam_hdr_read, sam_hdr_t, sam_parse_region,
    HTS_PARSE_LIST, HTS_PARSE_ONE_COORD, HTS_POS_MAX,
};
use libc::c_int;
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn expect_region(
    hdr: *mut sam_hdr_t,
    region: &str,
    flags: c_int,
    expected_rest: Option<&str>,
    expected_tid: c_int,
    expected_beg: hts_pos_t,
    expected_end: hts_pos_t,
) {
    let region = CString::new(region).unwrap();
    let mut tid = -1;
    let mut beg = -1;
    let mut end = -1;
    let rest = sam_parse_region(hdr, region.as_ptr(), &mut tid, &mut beg, &mut end, flags);

    match expected_rest {
        Some(expected_rest) => {
            assert!(!rest.is_null(), "failed to parse {:?}", region);
            assert_eq!(CStr::from_ptr(rest).to_bytes(), expected_rest.as_bytes());
            assert_eq!((tid, beg, end), (expected_tid, expected_beg, expected_end));
        }
        None => assert!(rest.is_null(), "unexpectedly parsed {:?}", region),
    }
}

unsafe fn with_colons_header(test: impl FnOnce(*mut sam_hdr_t)) {
    let path = c_fixture("htslib/test/colons.bam");
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    test(hdr);

    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

#[test]
fn parses_real_colon_contig_header_region_ranges() {
    unsafe {
        with_colons_header(|hdr| {
            expect_region(hdr, "chr1", 0, Some(""), 0, 0, HTS_POS_MAX);
            expect_region(hdr, "chr1:50", 0, Some(""), 0, 49, HTS_POS_MAX);
            expect_region(hdr, "chr1:50", HTS_PARSE_ONE_COORD, Some(""), 0, 49, 50);
            expect_region(hdr, "chr1:50-100", 0, Some(""), 0, 49, 100);
            expect_region(hdr, "chr1:50-", 0, Some(""), 0, 49, HTS_POS_MAX);
            expect_region(hdr, "chr1:-50", 0, Some(""), 0, 0, 50);
        });
    }
}

#[test]
fn parses_real_colon_contig_header_quoted_names() {
    unsafe {
        with_colons_header(|hdr| {
            expect_region(hdr, "chr1:100-200", 0, None, 0, 0, 0);
            expect_region(hdr, "{chr1}:100-200", 0, Some(""), 0, 99, 200);
            expect_region(hdr, "{chr1:100-200}", 0, Some(""), 2, 0, HTS_POS_MAX);
            expect_region(hdr, "{chr1:100-200}:100-200", 0, Some(""), 2, 99, 200);
            expect_region(hdr, "{chr2:100-200}:100-200", 0, Some(""), 3, 99, 200);
            expect_region(hdr, "chr2:100-200:100-200", 0, Some(""), 3, 99, 200);
            expect_region(hdr, "chr2:100-200", 0, Some(""), 3, 0, HTS_POS_MAX);
        });
    }
}

#[test]
fn parses_real_colon_contig_header_numeric_forms() {
    unsafe {
        with_colons_header(|hdr| {
            expect_region(hdr, "chr3", 0, Some(""), 4, 0, HTS_POS_MAX);
            expect_region(hdr, "chr3:", 0, Some(""), 4, 0, HTS_POS_MAX);
            expect_region(hdr, "chr3:1000-1500", 0, Some(""), 4, 999, 1500);
            expect_region(hdr, "chr3:1,000-1,500", 0, Some(""), 4, 999, 1500);
            expect_region(hdr, "chr3:1k-1.5K", 0, Some(""), 4, 999, 1500);
            expect_region(hdr, "chr3:1e3-1.5e3", 0, Some(""), 4, 999, 1500);
            expect_region(hdr, "chr3:1e3-15e2", 0, Some(""), 4, 999, 1500);
        });
    }
}

#[test]
fn parses_real_colon_contig_header_list_mode_edges() {
    unsafe {
        with_colons_header(|hdr| {
            expect_region(
                hdr,
                "chr1,chr3",
                HTS_PARSE_LIST,
                Some("chr3"),
                0,
                0,
                HTS_POS_MAX,
            );
            expect_region(hdr, "chr1:100-200,chr3", HTS_PARSE_LIST, None, 0, 0, 0);
            expect_region(
                hdr,
                "{chr1,chr3}",
                HTS_PARSE_LIST,
                Some(""),
                5,
                0,
                HTS_POS_MAX,
            );
            expect_region(
                hdr,
                "{chr1,chr3},chr1",
                HTS_PARSE_LIST,
                Some("chr1"),
                5,
                0,
                HTS_POS_MAX,
            );
            expect_region(
                hdr,
                "chr3:1,000-1,500",
                HTS_PARSE_LIST | HTS_PARSE_ONE_COORD,
                Some("000-1,500"),
                4,
                0,
                1,
            );
        });
    }
}

#[test]
fn rejects_real_colon_contig_header_invalid_regions() {
    unsafe {
        with_colons_header(|hdr| {
            expect_region(hdr, "chr2", 0, None, 0, 0, 0);
            expect_region(hdr, "chr1,", 0, None, 0, 0, 0);
            expect_region(hdr, "{chr1", 0, None, 0, 0, 0);
            expect_region(hdr, "chr1:10-10", 0, Some(""), 0, 9, 10);
            expect_region(hdr, "chr1:10-9", 0, None, 0, 0, 0);
            expect_region(hdr, "chr1:x", 0, None, 0, 0, 0);
            expect_region(hdr, "chr1:1-y", 0, None, 0, 0, 0);
            expect_region(hdr, "chr1:1,chr3", 0, None, 0, 0, 0);
        });
    }
}
