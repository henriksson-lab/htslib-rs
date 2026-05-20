use htslib_mini_rs::{
    hts_close, hts_open, hts_pos_t, hts_set_fai_filename, sam_hdr_destroy, sam_hdr_name2tid,
    sam_hdr_nref, sam_hdr_read, sam_hdr_tid2len, sam_hdr_tid2name,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn assert_alignment_header_targets(
    path: &str,
    reference: Option<&str>,
    expected: &[(&CStr, hts_pos_t)],
) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {path}");

    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read header from {path}");
    assert_eq!(sam_hdr_nref(hdr), expected.len() as i32);
    assert_eq!((*hdr).n_targets, expected.len() as i32);
    assert!(!(*hdr).target_name.is_null());
    assert!(!(*hdr).target_len.is_null());

    for (tid, (name, len)) in expected.iter().enumerate() {
        let tid = tid as i32;
        assert_eq!(sam_hdr_name2tid(hdr, name.as_ptr()), tid);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, tid)), *name);
        assert_eq!(sam_hdr_tid2len(hdr, tid), *len);
        assert_eq!(CStr::from_ptr(*(*hdr).target_name.add(tid as usize)), *name);
        if *len <= u32::MAX as hts_pos_t {
            assert_eq!(*(*hdr).target_len.add(tid as usize), *len as u32);
        }
    }

    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

#[test]
fn reads_realn_sam_headers_with_exact_single_references() {
    unsafe {
        assert_alignment_header_targets("htslib/test/realn01.sam", None, &[(c"000000F", 686)]);
        assert_alignment_header_targets("htslib/test/realn02.sam", None, &[(c"17", 4_200)]);
        assert_alignment_header_targets("htslib/test/realn03.sam", None, &[(c"MX", 11)]);
    }
}

#[test]
fn reads_auxiliary_and_padding_sam_headers_with_exact_references() {
    unsafe {
        assert_alignment_header_targets("htslib/test/auxf#values.sam", None, &[(c"Sheila", 20)]);
        assert_alignment_header_targets(
            "htslib/test/fieldarith.sam",
            None,
            &[(c"one", 1_000), (c"two", 500)],
        );
        assert_alignment_header_targets("htslib/test/c2#pad.sam", None, &[(c"c2", 9)]);
        assert_alignment_header_targets("htslib/test/tlen/d7.sam", None, &[(c"ref", 20)]);
    }
}

#[test]
fn reads_index_sam_header_with_exact_reference_order() {
    unsafe {
        assert_alignment_header_targets(
            "htslib/test/index_dos.sam",
            None,
            &[
                (c"CHROMOSOME_I", 1_009_800),
                (c"CHROMOSOME_II", 5_000),
                (c"CHROMOSOME_III", 5_000),
                (c"CHROMOSOME_IV", 5_000),
                (c"CHROMOSOME_V", 5_000),
                (c"CHROMOSOME_X", 5_000),
                (c"CHROMOSOME_MtDNA", 5_000),
            ],
        );
    }
}

#[test]
fn reads_real_bam_headers_with_exact_reference_targets() {
    unsafe {
        assert_alignment_header_targets(
            "htslib/test/range.bam",
            None,
            &[
                (c"CHROMOSOME_I", 1_009_800),
                (c"CHROMOSOME_II", 5_000),
                (c"CHROMOSOME_III", 5_000),
                (c"CHROMOSOME_IV", 5_000),
                (c"CHROMOSOME_V", 5_000),
                (c"CHROMOSOME_X", 5_000),
                (c"CHROMOSOME_MtDNA", 5_000),
            ],
        );
        assert_alignment_header_targets(
            "htslib/test/no_hdr_sq_1.bam",
            None,
            &[
                (c"CHROMOSOME_I", 1_009_800),
                (c"CHROMOSOME_II", 5_000),
                (c"CHROMOSOME_III", 5_000),
                (c"CHROMOSOME_IV", 5_000),
                (c"CHROMOSOME_V", 5_000),
            ],
        );
    }
}

#[test]
fn reads_colon_named_bam_header_with_exact_target_names() {
    unsafe {
        assert_alignment_header_targets(
            "htslib/test/colons.bam",
            None,
            &[
                (c"chr1", 1_000),
                (c"chr1:100", 1_000),
                (c"chr1:100-200", 1_000),
                (c"chr2:100-200", 1_000),
                (c"chr3", 1_000),
                (c"chr1,chr3", 1_000),
            ],
        );
    }
}

#[test]
fn reads_real_cram_headers_with_exact_reference_targets() {
    unsafe {
        assert_alignment_header_targets(
            "htslib/test/range.cram",
            Some("htslib/test/ce.fa"),
            &[
                (c"CHROMOSOME_I", 1_009_800),
                (c"CHROMOSOME_II", 5_000),
                (c"CHROMOSOME_III", 5_000),
                (c"CHROMOSOME_IV", 5_000),
                (c"CHROMOSOME_V", 5_000),
                (c"CHROMOSOME_X", 5_000),
                (c"CHROMOSOME_MtDNA", 5_000),
            ],
        );
        assert_alignment_header_targets("htslib/test/tlen/d7.cram", None, &[(c"ref", 20)]);
    }
}
