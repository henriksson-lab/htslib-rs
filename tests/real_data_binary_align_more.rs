use htslib_mini_rs::{
    bam_destroy1, bam_get_qname, bam_init1, hclose, hopen, htsFormat, hts_check_EOF, hts_close,
    hts_detect_format2, hts_open, hts_set_fai_filename, sam_hdr_destroy, sam_hdr_read, sam_read1,
    HTS_FORMAT_BAM, HTS_FORMAT_CRAM,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct RecordCore {
    qname: String,
    flag: u16,
    tid: i32,
    pos: i64,
    qual: u8,
    mtid: i32,
    mpos: i64,
    isize: i64,
    n_cigar: u32,
    l_qseq: i32,
}

impl RecordCore {
    unsafe fn from_record(rec: *mut htslib_mini_rs::bam1_t) -> Self {
        Self {
            qname: CStr::from_ptr(bam_get_qname(rec))
                .to_string_lossy()
                .into_owned(),
            flag: (*rec).core.flag,
            tid: (*rec).core.tid,
            pos: (*rec).core.pos,
            qual: (*rec).core.qual,
            mtid: (*rec).core.mtid,
            mpos: (*rec).core.mpos,
            isize: (*rec).core.isize,
            n_cigar: (*rec).core.n_cigar,
            l_qseq: (*rec).core.l_qseq,
        }
    }
}

unsafe fn count_and_first_record(path: &str, reference: Option<&str>) -> (usize, RecordCore) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path_c.to_string_lossy());

    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read alignment header from {}",
        path_c.to_string_lossy()
    );

    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut count = 0;
    let mut first = None;
    loop {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        if first.is_none() {
            first = Some(RecordCore::from_record(rec));
        }
        count += 1;
    }

    let first = first.unwrap_or_else(|| panic!("{path} should contain records"));
    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    (count, first)
}

unsafe fn count_records(path: &str, reference: Option<&str>) -> usize {
    count_and_first_record(path, reference).0
}

unsafe fn assert_readable_and_eof_checkable(path: &str, reference: Option<&str>) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path_c.to_string_lossy());

    if let Some(reference) = reference {
        let reference_c = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference_c.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
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

    assert!(count > 0, "{path} should contain readable records");
    assert!(hts_check_EOF(fp) >= 0);

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

fn detect_fixture_format(path: &str) -> htsFormat {
    unsafe {
        let path_c = c_fixture(path);
        let fp = hopen(path_c.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path_c.to_string_lossy());

        let mut fmt: htsFormat = std::mem::zeroed();
        assert_eq!(hts_detect_format2(fp, path_c.as_ptr(), &mut fmt), 0);
        assert_eq!(hclose(fp), 0);
        fmt
    }
}

#[test]
fn detects_binary_alignment_fixture_formats() {
    assert_eq!(
        detect_fixture_format("htslib/test/no_hdr_sq_1.bam").format,
        HTS_FORMAT_BAM
    );
    assert_eq!(
        detect_fixture_format("htslib/test/mpileup/small.bam").format,
        HTS_FORMAT_BAM
    );
    assert_eq!(
        detect_fixture_format("htslib/test/bgzf_boundaries/bgzf_boundaries1.bam").format,
        HTS_FORMAT_BAM
    );
    assert_eq!(
        detect_fixture_format("htslib/test/tlen/d7.cram").format,
        HTS_FORMAT_CRAM
    );
}

#[test]
fn reads_bam_boundary_and_small_fixtures() {
    unsafe {
        assert_readable_and_eof_checkable("htslib/test/bgzf_boundaries/bgzf_boundaries1.bam", None);
        assert_readable_and_eof_checkable("htslib/test/bgzf_boundaries/bgzf_boundaries2.bam", None);
        assert_readable_and_eof_checkable("htslib/test/bgzf_boundaries/bgzf_boundaries3.bam", None);
        assert_readable_and_eof_checkable("htslib/test/mpileup/small.bam", None);
    }
}

#[test]
fn reads_no_hdr_sq_bam_first_record_metadata() {
    unsafe {
        let (count, first) = count_and_first_record("htslib/test/no_hdr_sq_1.bam", None);
        assert_eq!(count, 6);
        assert_eq!(
            first,
            RecordCore {
                qname: "I".to_string(),
                flag: 16,
                tid: 0,
                pos: 1,
                qual: 1,
                mtid: -1,
                mpos: -1,
                isize: 0,
                n_cigar: 3,
                l_qseq: 100,
            }
        );
    }
}

#[test]
fn counts_reference_linked_cram_fixtures() {
    unsafe {
        assert_eq!(
            count_records(
                "htslib/test/auxf#values_java.cram",
                Some("htslib/test/auxf.fa")
            ),
            2
        );
        assert_eq!(
            count_records("htslib/test/ce#5b_java.cram", Some("htslib/test/ce.fa")),
            7
        );
        assert_eq!(
            count_records(
                "htslib/test/xx#large_aux_java.cram",
                Some("htslib/test/xx.fa")
            ),
            3
        );
    }
}

#[test]
fn reads_reference_linked_cram_first_record_metadata() {
    unsafe {
        let (_, auxf) = count_and_first_record(
            "htslib/test/auxf#values_java.cram",
            Some("htslib/test/auxf.fa"),
        );
        assert_eq!(
            auxf,
            RecordCore {
                qname: "Fred".to_string(),
                flag: 16,
                tid: 0,
                pos: 0,
                qual: 86,
                mtid: -1,
                mpos: -1,
                isize: 0,
                n_cigar: 1,
                l_qseq: 10,
            }
        );

        let (_, ce) =
            count_and_first_record("htslib/test/ce#5b_java.cram", Some("htslib/test/ce.fa"));
        assert_eq!(
            ce,
            RecordCore {
                qname: "I".to_string(),
                flag: 16,
                tid: 0,
                pos: 1,
                qual: 1,
                mtid: -1,
                mpos: -1,
                isize: 0,
                n_cigar: 3,
                l_qseq: 100,
            }
        );

        let (_, xx) = count_and_first_record(
            "htslib/test/xx#large_aux_java.cram",
            Some("htslib/test/xx.fa"),
        );
        assert_eq!(
            xx,
            RecordCore {
                qname: "a1".to_string(),
                flag: 16,
                tid: 0,
                pos: 0,
                qual: 1,
                mtid: -1,
                mpos: -1,
                isize: 0,
                n_cigar: 1,
                l_qseq: 10,
            }
        );
    }
}

#[test]
fn counts_more_tlen_cram_fixtures() {
    unsafe {
        assert_eq!(count_records("htslib/test/tlen/a8.cram", None), 4);
        assert_eq!(count_records("htslib/test/tlen/a9b.cram", None), 4);
        assert_eq!(count_records("htslib/test/tlen/c8.cram", None), 4);
        assert_eq!(count_records("htslib/test/tlen/d5b.cram", None), 3);
        assert_eq!(count_records("htslib/test/tlen/d5e.cram", None), 3);
        assert_eq!(count_records("htslib/test/tlen/d7.cram", None), 4);
    }
}

#[test]
fn reads_tlen_cram_first_record_metadata() {
    unsafe {
        let (_, d7) = count_and_first_record("htslib/test/tlen/d7.cram", None);
        assert_eq!(
            d7,
            RecordCore {
                qname: "seq".to_string(),
                flag: 97,
                tid: 0,
                pos: 2,
                qual: 0,
                mtid: 0,
                mpos: 2,
                isize: 14,
                n_cigar: 1,
                l_qseq: 14,
            }
        );

        let (_, d5b) = count_and_first_record("htslib/test/tlen/d5b.cram", None);
        assert_eq!(
            d5b,
            RecordCore {
                qname: "seq".to_string(),
                flag: 81,
                tid: 0,
                pos: 3,
                qual: 0,
                mtid: 0,
                mpos: 3,
                isize: 14,
                n_cigar: 1,
                l_qseq: 14,
            }
        );

        let (_, a9b) = count_and_first_record("htslib/test/tlen/a9b.cram", None);
        assert_eq!(
            a9b,
            RecordCore {
                qname: "seq".to_string(),
                flag: 145,
                tid: 0,
                pos: 5,
                qual: 0,
                mtid: 0,
                mpos: 2,
                isize: -14,
                n_cigar: 1,
                l_qseq: 8,
            }
        );
    }
}
