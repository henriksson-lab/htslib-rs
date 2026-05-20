use htslib_mini_rs::{
    bam_destroy1, bam_init1, hts_check_EOF, hts_close, hts_open, hts_set_fai_filename,
    sam_hdr_destroy, sam_hdr_read, sam_read1,
};
use std::ffi::CString;

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn count_alignment_records(path: &str, reference: Option<&str>) -> usize {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read alignment header from {}",
        path.to_string_lossy()
    );
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut count = 0;
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        count += 1;
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    count
}

unsafe fn assert_readable_alignment_fixture(path: &str, reference: Option<&str>) {
    assert!(
        count_alignment_records(path, reference) > 0,
        "{path} should contain readable alignment records"
    );
}

unsafe fn assert_alignment_eof_is_checkable(path: &str, reference: Option<&str>) {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read alignment header from {}",
        path.to_string_lossy()
    );
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut nread = 0;
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        nread += 1;
    }

    assert!(nread > 0);
    assert!(hts_check_EOF(fp) >= 0);

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

#[test]
fn counts_realn_sam_alignment_fixtures() {
    unsafe {
        assert_eq!(count_alignment_records("htslib/test/realn01.sam", None), 4);
        assert_eq!(
            count_alignment_records("htslib/test/realn01_exp.sam", None),
            4
        );
        assert_eq!(count_alignment_records("htslib/test/realn02.sam", None), 9);
        assert_eq!(
            count_alignment_records("htslib/test/realn02_exp.sam", None),
            9
        );
        assert_eq!(count_alignment_records("htslib/test/realn03.sam", None), 2);
        assert_eq!(
            count_alignment_records("htslib/test/realn03_exp.sam", None),
            2
        );
    }
}

#[test]
fn counts_fastq_sam_alignment_fixtures() {
    unsafe {
        assert_eq!(
            count_alignment_records("htslib/test/fastq/minimal.sam", None),
            1
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/single_aux.sam", None),
            5
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/inter_aux.sam", None),
            10
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/multiline.sam", None),
            2
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/longline.sam", None),
            1
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/UMI.sam", None),
            7
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/name2.sam", None),
            4
        );
        assert_eq!(
            count_alignment_records("htslib/test/fastq/filter_casava.sam", None),
            4
        );
    }
}

#[test]
fn counts_index_and_long_reference_sam_alignment_fixtures() {
    unsafe {
        assert_eq!(count_alignment_records("htslib/test/index2.sam", None), 8);
        assert_eq!(count_alignment_records("htslib/test/index3.sam", None), 12);
        assert_eq!(
            count_alignment_records("htslib/test/index_dos.sam", None),
            181
        );
        assert_eq!(
            count_alignment_records("htslib/test/fieldarith.sam", None),
            8
        );
    }
}

#[test]
fn counts_more_base_modification_sam_alignment_fixtures() {
    unsafe {
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-chebi.sam", None),
            1
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-double.sam", None),
            1
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-orient.sam", None),
            4
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-pileup.sam", None),
            4
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-pileup2.sam", None),
            2
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-not-all-modded.sam", None),
            4
        );
    }
}

#[test]
fn counts_more_template_length_sam_alignment_fixtures() {
    unsafe {
        assert_eq!(count_alignment_records("htslib/test/tlen/a4.sam", None), 3);
        assert_eq!(count_alignment_records("htslib/test/tlen/a5.sam", None), 3);
        assert_eq!(count_alignment_records("htslib/test/tlen/a8.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/a9.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/b8.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/c7.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/c8.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/d4.sam", None), 3);
        assert_eq!(count_alignment_records("htslib/test/tlen/d7.sam", None), 4);
    }
}

#[test]
fn reads_more_bam_alignment_fixtures() {
    unsafe {
        assert_readable_alignment_fixture("htslib/test/no_hdr_sq_1.bam", None);
        assert_readable_alignment_fixture("htslib/test/mpileup/small.bam", None);

        assert_alignment_eof_is_checkable("htslib/test/no_hdr_sq_1.bam", None);
        assert_alignment_eof_is_checkable("htslib/test/mpileup/small.bam", None);
    }
}

#[test]
fn reads_more_cram_alignment_fixtures() {
    unsafe {
        assert_readable_alignment_fixture(
            "htslib/test/auxf#values_java.cram",
            Some("htslib/test/auxf.fa"),
        );
        assert_readable_alignment_fixture("htslib/test/ce#5b_java.cram", Some("htslib/test/ce.fa"));
        assert_readable_alignment_fixture(
            "htslib/test/xx#large_aux_java.cram",
            Some("htslib/test/xx.fa"),
        );
        assert_readable_alignment_fixture("htslib/test/tlen/a4.cram", None);
        assert_readable_alignment_fixture("htslib/test/tlen/d4.cram", None);

        assert_alignment_eof_is_checkable(
            "htslib/test/auxf#values_java.cram",
            Some("htslib/test/auxf.fa"),
        );
        assert_alignment_eof_is_checkable("htslib/test/ce#5b_java.cram", Some("htslib/test/ce.fa"));
    }
}
