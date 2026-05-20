use htslib_mini_rs::{
    fai_adjust_region, fai_destroy, fai_fetch64, fai_line_length, fai_load3_format,
    fai_parse_region, faidx_fetch_qual64, faidx_fetch_seq64, faidx_has_seq, faidx_iseq,
    faidx_seq_len, faidx_seq_len64, hts_pos_t, FAI_FASTQ,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn load_fastq_fai() -> *mut htslib_mini_rs::faidx_t {
    let input = c_fixture("htslib/test/faidx/fastqs.fq");
    let index = c_fixture("htslib/test/faidx/fastqs.fq.expected.fai");
    let fai = fai_load3_format(
        input.as_ptr(),
        index.as_ptr(),
        std::ptr::null(),
        0,
        FAI_FASTQ,
    );
    assert!(!fai.is_null());
    fai
}

unsafe fn fetched_string(ptr: *mut libc::c_char, len: hts_pos_t) -> String {
    assert!(!ptr.is_null());
    assert!(len >= 0);
    let text = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    assert_eq!(text.len(), len as usize);
    libc::free(ptr.cast());
    text
}

fn append_wrapped(out: &mut String, text: &str) {
    for chunk in text.as_bytes().chunks(50) {
        out.push_str(std::str::from_utf8(chunk).unwrap());
        out.push('\n');
    }
}

unsafe fn format_adjusted_fastq_records(
    fai: *const htslib_mini_rs::faidx_t,
    regions: &[&'static CStr],
) -> String {
    let mut out = String::new();
    for &region in regions {
        let mut tid = 0;
        let mut beg = 0;
        let mut end = 0;
        assert!(!fai_parse_region(fai, region.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null());
        assert!(fai_adjust_region(fai, tid, &mut beg, &mut end) >= 0);

        let name = faidx_iseq(fai, tid);
        let mut seq_len = 0;
        let seq = fetched_string(
            faidx_fetch_seq64(fai, name, beg, end - 1, &mut seq_len),
            seq_len,
        );
        let mut qual_len = 0;
        let qual = fetched_string(
            faidx_fetch_qual64(fai, name, beg, end - 1, &mut qual_len),
            qual_len,
        );
        assert_eq!(seq_len, qual_len);

        out.push('@');
        out.push_str(region.to_str().unwrap());
        out.push_str(" length: ");
        out.push_str(&seq_len.to_string());
        out.push('\n');
        append_wrapped(&mut out, &seq);
        out.push_str("+\n");
        append_wrapped(&mut out, &qual);
    }
    out
}

unsafe fn fetch_region_len(
    fai: *const htslib_mini_rs::faidx_t,
    region: &'static CStr,
) -> hts_pos_t {
    let mut len = 0;
    let seq = fai_fetch64(fai, region.as_ptr(), &mut len);
    let _ = fetched_string(seq, len);
    len
}

#[test]
fn original_fastq_faidx_tst_named_helper_edges_match_expected_index() {
    unsafe {
        let fai = load_fastq_fai();

        assert_eq!(fai_line_length(fai, c"FAKE0005_3".as_ptr()), 63);
        assert_eq!(fai_line_length(fai, c"SRR014849.203935_3".as_ptr()), 144);
        assert_eq!(faidx_has_seq(fai, c"SRR014849.203935_3".as_ptr()), 1);
        assert_eq!(faidx_has_seq(fai, c"absent".as_ptr()), 0);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"FAKE0005_1");
        assert_eq!(faidx_seq_len(fai, c"FSRRS4401CM938_1".as_ptr()), 453);
        assert_eq!(faidx_seq_len64(fai, c"FSRRS4401AOV6A_4".as_ptr()), 309);

        fai_destroy(fai);
    }
}

#[test]
fn original_fastq_faidx_adjust_region_matches_expected_fastq_output() {
    const EXPECTED_FASTQ: &str = include_str!("../htslib/test/faidx/fastqs.1.expected.fq");
    const REGIONS: &[&CStr] = &[
        c"FAKE0006_1:4-12",
        c"FSRRS4401BE7HA_1:81-120",
        c"FAKE0010_2",
        c"SRR014849.50939_3:71-90",
    ];

    unsafe {
        let fai = load_fastq_fai();
        assert_eq!(format_adjusted_fastq_records(fai, REGIONS), EXPECTED_FASTQ);
        fai_destroy(fai);
    }
}

#[test]
fn original_fastq_faidx_whole_record_and_slice_lengths_match_expected_regions() {
    unsafe {
        let fai = load_fastq_fai();

        assert_eq!(fetch_region_len(fai, c"FAKE0006_1:4-12"), 9);
        assert_eq!(fetch_region_len(fai, c"FSRRS4401BE7HA_1:81-120"), 40);
        assert_eq!(fetch_region_len(fai, c"FAKE0010_2"), 30);
        assert_eq!(fetch_region_len(fai, c"SRR014849.50939_3:71-90"), 20);

        fai_destroy(fai);
    }
}
