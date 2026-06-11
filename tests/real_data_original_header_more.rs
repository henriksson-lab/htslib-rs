use htslib_rs::{
    bcf_hdr_destroy, bcf_hdr_fmt_text, bcf_hdr_get_version, bcf_hdr_id2int, bcf_hdr_read,
    hts_close, hts_open, ks_free, kstring_t, sam_hdr_count_lines, sam_hdr_destroy,
    sam_hdr_find_line_pos, sam_hdr_find_tag_id, sam_hdr_find_tag_pos, sam_hdr_length,
    sam_hdr_line_name, sam_hdr_name2tid, sam_hdr_read, sam_hdr_str, sam_hdr_tid2len,
    sam_hdr_tid2name, vcf_hdr_read,
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
    ks.data.clone()
}

unsafe fn sam_header_text(hdr: *mut htslib_rs::sam_hdr_t) -> String {
    let hdr = &mut *hdr;
    let len = sam_hdr_length(hdr);
    assert!(!sam_hdr_str(hdr).is_null());
    String::from_utf8(std::slice::from_raw_parts(sam_hdr_str(hdr).cast::<u8>(), len).to_vec())
        .unwrap()
}

unsafe fn assert_sam_tag_by_pos(
    hdr: *mut htslib_rs::sam_hdr_t,
    type_: &CStr,
    pos: c_int,
    key: &CStr,
    expected: &[u8],
) {
    let mut ks: kstring_t = kstring_t::default();
    assert_eq!(
        sam_hdr_find_tag_pos(&mut *hdr, type_, pos, key, &mut ks),
        0,
        "missing {type_:?}[{pos}].{key:?}"
    );
    assert_eq!(kstring_bytes(&ks), expected);
    ks_free(&mut ks);
}

unsafe fn assert_sam_tag_by_id(
    hdr: *mut htslib_rs::sam_hdr_t,
    type_: &CStr,
    id_key: &CStr,
    id_value: &CStr,
    key: &CStr,
    expected: &[u8],
) {
    let mut ks: kstring_t = kstring_t::default();
    assert_eq!(
        sam_hdr_find_tag_id(
            &mut *hdr,
            type_,
            id_key.as_ptr(),
            id_value.as_ptr(),
            key,
            &mut ks,
        ),
        0,
        "missing {type_:?}.{id_key:?}={id_value:?}.{key:?}"
    );
    assert_eq!(kstring_bytes(&ks), expected);
    ks_free(&mut ks);
}

unsafe fn sam_header(path: &str) -> (*mut htslib_rs::htsFile, *mut htslib_rs::sam_hdr_t) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {path}");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read SAM header from {path}");
    (fp, hdr)
}

unsafe fn close_sam_header(fp: *mut htslib_rs::htsFile, hdr: *mut htslib_rs::sam_hdr_t) {
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

unsafe fn formatted_vcf_header(hdr: *const htslib_rs::bcf_hdr_t) -> String {
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

        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"HD"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"SQ"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"RG"), 2);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"PG"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"CO"), 2);

        assert_sam_tag_by_pos(hdr, c"HD", 0, c"VN", b"1.4");
        assert_sam_tag_by_pos(hdr, c"HD", 0, c"SO", b"coordinate");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"SN", b"xx");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"LN", b"20");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"AS", b"?");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"SP", b"?");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"UR", b"?");
        assert_sam_tag_by_pos(hdr, c"SQ", 0, c"M5", b"bbf4de6d8497a119dda6e074521643dc");

        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(&mut *hdr, c"RG", 0)),
            c"x1"
        );
        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(&mut *hdr, c"RG", 1)),
            c"x2"
        );
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x1", c"SM", b"x1");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"SM", b"x2");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"LB", b"x");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"PG", b"foo:bar");
        assert_sam_tag_by_id(hdr, c"RG", c"ID", c"x2", c"PI", b"1111");

        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(&mut *hdr, c"PG", 0)),
            c"emacs"
        );
        assert_sam_tag_by_id(hdr, c"PG", c"ID", c"emacs", c"PN", b"emacs");
        assert_sam_tag_by_id(hdr, c"PG", c"ID", c"emacs", c"VN", b"23.1.1");

        let mut co: kstring_t = kstring_t::default();
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, c"CO", 0, &mut co), 0);
        assert_eq!(kstring_bytes(&co), b"@CO\talso test");
        ks_free(&mut co);
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, c"CO", 1, &mut co), 0);
        assert_eq!(kstring_bytes(&co), b"@CO\tother\theaders");
        ks_free(&mut co);

        close_sam_header(fp, hdr);
    }
}

#[test]
fn padded_c1_sam_header_keeps_original_read_group_fields() {
    unsafe {
        let (fp, hdr) = sam_header("htslib/test/c1#pad3.sam");

        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"SQ"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"RG"), 1);
        assert_eq!(sam_hdr_name2tid(&mut *hdr, c"c1"), 0);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 0)), c"c1");
        assert_eq!(sam_hdr_tid2len(&*hdr, 0), 10);
        assert_eq!(
            CStr::from_ptr(sam_hdr_line_name(&mut *hdr, c"RG", 0)),
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

        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"HD"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"SQ"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"CO"), 3);
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
        let mut co: kstring_t = kstring_t::default();
        for (i, expected) in expected_comments.iter().enumerate() {
            assert_eq!(
                sam_hdr_find_line_pos(&mut *hdr, c"CO", i as c_int, &mut co),
                0
            );
            assert_eq!(kstring_bytes(&co), *expected);
            ks_free(&mut co);
        }

        close_sam_header(fp, hdr);
    }
}

#[test]
fn index_dos_sam_header_normalizes_crlf_and_keeps_all_sq_targets() {
    unsafe {
        let (fp, hdr) = sam_header("htslib/test/index_dos.sam");

        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"SQ"), 7);
        for (tid, (name, len, md5)) in [
            (
                c"CHROMOSOME_I",
                1_009_800,
                b"8ede36131e0dbf3417807e48f77f3ebd".as_slice(),
            ),
            (
                c"CHROMOSOME_II",
                5_000,
                b"8e7993f7a93158587ee897d7287948ec".as_slice(),
            ),
            (
                c"CHROMOSOME_III",
                5_000,
                b"3adcb065e1cf74fafdbba1e8c352b323".as_slice(),
            ),
            (
                c"CHROMOSOME_IV",
                5_000,
                b"251af66a69ee589c9f3757340ec2de6f".as_slice(),
            ),
            (
                c"CHROMOSOME_V",
                5_000,
                b"cf200a65fb754836dcc56b24b3170ee8".as_slice(),
            ),
            (
                c"CHROMOSOME_X",
                5_000,
                b"6f9368fd2192c89c613718399d2d31fc".as_slice(),
            ),
            (
                c"CHROMOSOME_MtDNA",
                5_000,
                b"cd05857ece6411f40257a565ccfe15bb".as_slice(),
            ),
        ]
        .iter()
        .enumerate()
        {
            let tid = tid as c_int;
            assert_eq!(sam_hdr_name2tid(&mut *hdr, name), tid);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, tid)), *name);
            assert_eq!(sam_hdr_tid2len(&*hdr, tid), *len);
            assert_sam_tag_by_pos(hdr, c"SQ", tid, c"M5", md5);
        }

        assert_sam_tag_by_id(hdr, c"PG", c"ID", c"bowtie2", c"PN", b"bowtie2");
        assert_sam_tag_by_id(hdr, c"PG", c"ID", c"bowtie2", c"VN", b"2.0.0-beta5");
        let text = sam_header_text(hdr);
        assert!(!text.contains('\r'));
        assert!(text.ends_with("@PG\tID:bowtie2\tPN:bowtie2\tVN:2.0.0-beta5\n"));

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

#[test]
fn minimal_sam_header_without_hd_keeps_two_original_targets_only() {
    unsafe {
        let (fp, hdr) = sam_header("htslib/test/xx#minimal.sam");

        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"HD"), 0);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"SQ"), 2);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"RG"), 0);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, c"PG"), 0);
        assert_eq!(sam_hdr_name2tid(&mut *hdr, c"xx"), 0);
        assert_eq!(sam_hdr_name2tid(&mut *hdr, c"yy"), 1);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 0)), c"xx");
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 1)), c"yy");
        assert_eq!(sam_hdr_tid2len(&*hdr, 0), 20);
        assert_eq!(sam_hdr_tid2len(&*hdr, 1), 20);
        assert_eq!(
            sam_header_text(hdr),
            "@SQ\tSN:xx\tLN:20\n@SQ\tSN:yy\tLN:20\n"
        );

        close_sam_header(fp, hdr);
    }
}
