use htslib_rs::{
    hts_close, hts_open, hts_pos_t, hts_set_fai_filename, sam_hdr_destroy, sam_hdr_name2tid,
    sam_hdr_nref, sam_hdr_read, sam_hdr_tid2len, sam_hdr_tid2name,
};
use std::ffi::CStr;

fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

unsafe fn assert_alignment_header_targets(
    path: &str,
    reference: Option<&str>,
    expected: &[(&[u8], hts_pos_t)],
) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(!fp.is_null(), "failed to open {path}");

    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr().cast()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read header from {path}");
    assert_eq!(sam_hdr_nref(&*hdr), expected.len() as i32);
    assert_eq!((*hdr).n_targets, expected.len() as i32);
    assert!(!(*hdr).target_name.is_null());
    assert!(!(*hdr).target_len.is_null());

    for (tid, (name, len)) in expected.iter().enumerate() {
        let tid = tid as i32;
        assert_eq!(sam_hdr_name2tid(&mut *hdr, name), tid);
        assert_eq!(
            CStr::from_ptr(sam_hdr_tid2name(&*hdr, tid).cast()).to_bytes(),
            *name
        );
        assert_eq!(sam_hdr_tid2len(&*hdr, tid), *len);
        assert_eq!(
            CStr::from_ptr((*(*hdr).target_name.add(tid as usize)).cast()).to_bytes(),
            *name
        );
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
        assert_alignment_header_targets("htslib/test/realn01.sam", None, &[(b"000000F", 686)]);
        assert_alignment_header_targets("htslib/test/realn02.sam", None, &[(b"17", 4_200)]);
        assert_alignment_header_targets("htslib/test/realn03.sam", None, &[(b"MX", 11)]);
    }
}

#[test]
fn reads_auxiliary_and_padding_sam_headers_with_exact_references() {
    unsafe {
        assert_alignment_header_targets("htslib/test/auxf#values.sam", None, &[(b"Sheila", 20)]);
        assert_alignment_header_targets(
            "htslib/test/fieldarith.sam",
            None,
            &[(b"one", 1_000), (b"two", 500)],
        );
        assert_alignment_header_targets("htslib/test/c2#pad.sam", None, &[(b"c2", 9)]);
        assert_alignment_header_targets("htslib/test/tlen/d7.sam", None, &[(b"ref", 20)]);
    }
}

#[test]
fn reads_index_sam_header_with_exact_reference_order() {
    unsafe {
        assert_alignment_header_targets(
            "htslib/test/index_dos.sam",
            None,
            &[
                (b"CHROMOSOME_I", 1_009_800),
                (b"CHROMOSOME_II", 5_000),
                (b"CHROMOSOME_III", 5_000),
                (b"CHROMOSOME_IV", 5_000),
                (b"CHROMOSOME_V", 5_000),
                (b"CHROMOSOME_X", 5_000),
                (b"CHROMOSOME_MtDNA", 5_000),
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
                (b"CHROMOSOME_I", 1_009_800),
                (b"CHROMOSOME_II", 5_000),
                (b"CHROMOSOME_III", 5_000),
                (b"CHROMOSOME_IV", 5_000),
                (b"CHROMOSOME_V", 5_000),
                (b"CHROMOSOME_X", 5_000),
                (b"CHROMOSOME_MtDNA", 5_000),
            ],
        );
        assert_alignment_header_targets(
            "htslib/test/no_hdr_sq_1.bam",
            None,
            &[
                (b"CHROMOSOME_I", 1_009_800),
                (b"CHROMOSOME_II", 5_000),
                (b"CHROMOSOME_III", 5_000),
                (b"CHROMOSOME_IV", 5_000),
                (b"CHROMOSOME_V", 5_000),
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
                (b"chr1", 1_000),
                (b"chr1:100", 1_000),
                (b"chr1:100-200", 1_000),
                (b"chr2:100-200", 1_000),
                (b"chr3", 1_000),
                (b"chr1,chr3", 1_000),
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
                (b"CHROMOSOME_I", 1_009_800),
                (b"CHROMOSOME_II", 5_000),
                (b"CHROMOSOME_III", 5_000),
                (b"CHROMOSOME_IV", 5_000),
                (b"CHROMOSOME_V", 5_000),
                (b"CHROMOSOME_X", 5_000),
                (b"CHROMOSOME_MtDNA", 5_000),
            ],
        );
        assert_alignment_header_targets("htslib/test/tlen/d7.cram", None, &[(b"ref", 20)]);
    }
}
