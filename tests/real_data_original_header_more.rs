use htslib_mini_rs::{
    bcf_hdr_destroy, bcf_hdr_fmt_text, bcf_hdr_get_version, bcf_hdr_id2int, bcf_hdr_read,
    hts_close, hts_open, ks_free, kstring_t, sam_hdr_count_lines, sam_hdr_destroy,
    sam_hdr_find_line_pos, sam_hdr_find_tag_id, sam_hdr_find_tag_pos, sam_hdr_line_name,
    sam_hdr_name2tid, sam_hdr_read, sam_hdr_tid2len, sam_hdr_tid2name, vcf_hdr_read,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_int;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> CString {
    CString::new(fixture(path).to_string_lossy().as_bytes()).unwrap()
}

unsafe fn kstring_bytes(ks: &kstring_t) -> Vec<u8> {
    assert!(!ks.s.is_null());
    std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l).to_vec()
}

unsafe fn assert_sam_tag_by_pos(
    hdr: *mut htslib_mini_rs::sam_hdr_t,
    type_: &CStr,
    pos: c_int,
    key: &CStr,
    expected: &[u8],
) {
    let mut ks: kstring_t = std::mem::zeroed();
    assert_eq!(
        sam_hdr_find_tag_pos(hdr, type_.as_ptr(), pos, key.as_ptr(), &mut ks),
        0,
        "missing {type_:?}[{pos}].{key:?}"
    );
    assert_eq!(kstring_bytes(&ks), expected);
    ks_free(&mut ks);
}

unsafe fn assert_sam_tag_by_id(
    hdr: *mut htslib_mini_rs::sam_hdr_t,
    type_: &CStr,
    id_key: &CStr,
    id_value: &CStr,
    key: &CStr,
    expected: &[u8],
) {
    let mut ks: kstring_t = std::mem::zeroed();
    assert_eq!(
        sam_hdr_find_tag_id(
            hdr,
            type_.as_ptr(),
            id_key.as_ptr(),
            id_value.as_ptr(),
            key.as_ptr(),
            &mut ks,
        ),
        0,
        "missing {type_:?}.{id_key:?}={id_value:?}.{key:?}"
    );
    assert_eq!(kstring_bytes(&ks), expected);
    ks_free(&mut ks);
}

unsafe fn sam_header(path: &str) -> (*mut htslib_mini_rs::htsFile, *mut htslib_mini_rs::sam_hdr_t) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {path}");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read SAM header from {path}");
    (fp, hdr)
}

unsafe fn close_sam_header(fp: *mut htslib_mini_rs::htsFile, hdr: *mut htslib_mini_rs::sam_hdr_t) {
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

unsafe fn formatted_vcf_header(hdr: *const htslib_mini_rs::bcf_hdr_t) -> String {
    let mut len = 0;
    let text = bcf_hdr_fmt_text(hdr, 0, &mut len);
    assert!(!text.is_null());
    let out = CStr::from_ptr(text).to_string_lossy().into_owned();
    assert_eq!(out.len(), len as usize);
    libc::free(text.cast());
    out
}

#[test]
fn xx_rg_sam_header_preserves_exact_lookup_tags_and_comments() {
    unsafe {
        let (fp, hdr) = sam_header("htslib/test/xx#rg.sam");

        assert_eq!(sam_hdr_count_lines(hdr, c"HD".as_ptr()), 1);
        assert_eq!(sam_hdr_count_lines(hdr, c"SQ".as_ptr()), 1);
        assert_eq!(sam_hdr_count_lines(hdr, c"RG".as_ptr()), 2);
        assert_eq!(sam_hdr_count_lines(hdr, c"PG".as_ptr()), 1);
        assert_eq!(sam_hdr_count_lines(hdr, c"CO".as_ptr()), 2);

        assert_sam_tag_by_pos(hdr, c"HD", 0, c"VN", b"1.4");
        assert_sam_tag_by_pos(hdr, c"HD", 0, c"SO", b"coordinate");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"SN", b"xx");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"LN", b"20");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"AS", b"?");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"SP", b"?");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"UR", b"?");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"M5", b"bbf4de6d8497a119dda6e074521643dc");

        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(hdr, c"RG".as_ptr(), 0)),
            c"x1"
        );
        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(hdr, c"RG".as_ptr(), 1)),
            c"x2"
        );
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x1", c"SM", b"x1");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"SM", b"x2");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"LB", b"x");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"PG", b"foo:bar");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"PI", b"1111");

        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(hdr, c"PG".as_ptr(), 0)),
            c"emacs"
        );
        assert_sam_tag_by_id(hdr, c"PG", c"ID", c"emacs", c"PN", b"emacs");
        assert_sam_tag_by_id(hdr, c"PG", c"ID", c"emacs", c"VN", b"23.1.1");

        let mut co: kstring_t = std::mem::zeroed();
        assert_eq!(sam_hdr_find_line_pos(hdr, c"CO".as_ptr(), 0, &mut co), 0);
        assert_eq!(kstring_bytes(&co), b"@CO\talso test");
        ks_free(&mut co);
        assert_eq!(sam_hdr_find_line_pos(hdr, c"CO".as_ptr(), 1, &mut co), 0);
        assert_eq!(kstring_bytes(&co), b"@CO\tother\theaders");
        ks_free(&mut co);

        close_sam_header(fp, hdr);
    }
}

#[test]
fn padded_c1_sam_header_keeps_original_read_group_fields() {
    unsafe {
        let (fp, hdr) = sam_header("htslib/test/c1#pad3.sam");

        assert_eq!(sam_hdr_count_lines(hdr, c"SQ".as_ptr()), 1);
        assert_eq!(sam_hdr_count_lines(hdr, c"RG".as_ptr()), 1);
        assert_eq!(sam_hdr_name2tid(hdr, c"c1".as_ptr()), 0);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 0)), c"c1");
        assert_eq!(sam_hdr_tid2len(hdr, 0), 10);
        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(hdr, c"RG".as_ptr(), 0)),
            c"p.sam"
        );
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"p.sam", c"SM", b"unknown");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"p.sam", c"LB", b"p.sam");

        close_sam_header(fp, hdr);
    }
}

#[test]
fn index3_expected_sam_header_keeps_original_comments_and_m5_tag() {
    unsafe {
        let (fp, hdr) = sam_header("htslib/test/index3_exp.sam");

        assert_eq!(sam_hdr_count_lines(hdr, c"HD".as_ptr()), 1);
        assert_eq!(sam_hdr_count_lines(hdr, c"SQ".as_ptr()), 1);
        assert_eq!(sam_hdr_count_lines(hdr, c"CO".as_ptr()), 3);
        assert_sam_tag_by_pos(hdr, c"HD", 0, c"VN", b"1.6");
        assert_sam_tag_by_pos(hdr, c"HD", 0, c"SO", b"coordinate");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"SN", b"CHROMOSOME_I");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"LN", b"1009800");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"M5", b"8ede36131e0dbf3417807e48f77f3ebd");

        let expected_comments = [
            b"@CO\tCRAM container skipping test - very long reads followed by short ones."
                .as_slice(),
            b"@CO\tUse option seqs_per_slice=2 when writing CRAM so that records are".as_slice(),
            b"@CO\tstored in multiple containers.".as_slice(),
        ];
        let mut co: kstring_t = std::mem::zeroed();
        for (i, expected) in expected_comments.iter().enumerate() {
            assert_eq!(
                sam_hdr_find_line_pos(hdr, c"CO".as_ptr(), i as c_int, &mut co),
                0
            );
            assert_eq!(kstring_bytes(&co), *expected);
            ks_free(&mut co);
        }

        close_sam_header(fp, hdr);
    }
}

#[test]
fn test_vcf_hdr_in_formats_exact_original_expected_header() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-hdr-in.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());

        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.1");
        assert_eq!(
            bcf_hdr_id2int(hdr, hts_sys::BCF_DT_SAMPLE as c_int, c"NA00001".as_ptr()),
            0
        );
        assert_eq!(
            formatted_vcf_header(hdr),
            std::fs::read_to_string(fixture("htslib/test/test-vcf-hdr.out")).unwrap()
        );

        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn modhdr_vcf_gz_formats_exact_original_expected_header() {
    unsafe {
        let vcf = c_fixture("htslib/test/modhdr.vcf.gz");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());

        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.3");
        assert_eq!(
            formatted_vcf_header(hdr),
            std::fs::read_to_string(fixture("htslib/test/modhdr.expected.vcf")).unwrap()
        );

        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}
