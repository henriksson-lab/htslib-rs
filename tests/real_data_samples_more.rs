use htslib_rs::{
    bam_cigar_op, bam_cigar_oplen, bam_destroy1, bam_endpos, bam_get_cigar, bam_get_qname,
    bam_init1, fai_build3, fai_destroy, fai_fetch, fai_fetchqual, fai_load3_format, faidx_has_seq,
    faidx_iseq, faidx_nseq, faidx_seq_len, hclose, hopen, htsFormat, hts_detect_format2,
    regidx_c_246_regidx_init, regidx_c_311_regidx_destroy, regidx_c_401_regidx_overlap,
    regidx_c_584_regitr_init, regidx_c_606_regitr_destroy, regidx_c_612_regitr_overlap,
    regidx_c_91_regidx_seq_nregs, regidx_c_98_regidx_nregs, sam_hdr_destroy, sam_hdr_nref,
    sam_hdr_read, sam_hdr_tid2len, sam_hdr_tid2name, sam_read1, BAM_CMATCH, FAI_FASTA, FAI_FASTQ,
    HTS_COMPRESSION_NO_COMPRESSION, HTS_FORMAT_BED, HTS_FORMAT_FASTA_FORMAT,
    HTS_FORMAT_FASTQ_FORMAT, HTS_FORMAT_REGION_LIST, HTS_FORMAT_SAM, HTS_FORMAT_SEQUENCE_DATA,
};
use std::ffi::{CStr, CString};
use std::path::PathBuf;

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn c_path(path: PathBuf) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn temp_index_path(label: &str, suffix: &str) -> CString {
    let path = std::env::temp_dir().join(format!(
        "htslib_rs-real-data-samples-more-{}-{}{}",
        std::process::id(),
        label,
        suffix
    ));
    c_path(path)
}

fn detect_fixture_format(path: &str) -> htsFormat {
    unsafe {
        let path = c_fixture(path);
        let fp = hopen(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let mut fmt: htsFormat = std::mem::zeroed();
        assert_eq!(hts_detect_format2(fp, path.as_ptr(), &mut fmt), 0);
        assert_eq!(hclose(fp), 0);
        fmt
    }
}

unsafe fn fetch_text(fai: *const htslib_rs::faidx_t, region: &CStr) -> String {
    let mut len = 0;
    let seq = fai_fetch(fai, region.as_ptr(), &mut len);
    assert!(!seq.is_null());
    let text = CStr::from_ptr(seq).to_string_lossy().into_owned();
    assert_eq!(text.len(), len as usize);
    libc::free(seq.cast());
    text
}

unsafe fn fetch_qual_text(fai: *const htslib_rs::faidx_t, region: &CStr) -> String {
    let mut len = 0;
    let qual = fai_fetchqual(fai, region.as_ptr(), &mut len);
    assert!(!qual.is_null());
    let text = CStr::from_ptr(qual).to_string_lossy().into_owned();
    assert_eq!(text.len(), len as usize);
    libc::free(qual.cast());
    text
}

#[test]
fn detects_demo_sample_fixture_formats() {
    let sam = detect_fixture_format("htslib/samples/sample.sam");
    assert_eq!(sam.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(sam.format, HTS_FORMAT_SAM);
    assert_eq!(sam.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let fasta = detect_fixture_format("htslib/samples/sample.ref.fa");
    assert_eq!(fasta.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(fasta.format, HTS_FORMAT_FASTA_FORMAT);
    assert_eq!(fasta.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let fastq = detect_fixture_format("htslib/samples/sample.ref.fq");
    assert_eq!(fastq.category, HTS_FORMAT_SEQUENCE_DATA);
    assert_eq!(fastq.format, HTS_FORMAT_FASTQ_FORMAT);
    assert_eq!(fastq.compression, HTS_COMPRESSION_NO_COMPRESSION);

    let bed = detect_fixture_format("htslib/samples/sample.bed");
    assert_eq!(bed.category, HTS_FORMAT_REGION_LIST);
    assert_eq!(bed.format, HTS_FORMAT_BED);
    assert_eq!(bed.compression, HTS_COMPRESSION_NO_COMPRESSION);
}

#[test]
fn reads_demo_sample_sam_header_and_exact_records() {
    unsafe {
        let path = c_fixture("htslib/samples/sample.sam");
        let fp = htslib_rs::hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(sam_hdr_nref(hdr), 2);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 0)), c"T1");
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 1)), c"T2");
        assert_eq!(sam_hdr_tid2len(hdr, 0), 40);
        assert_eq!(sam_hdr_tid2len(hdr, 1), 40);

        let rec = bam_init1();
        assert!(!rec.is_null());

        assert!(sam_read1(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bam_get_qname(rec)), c"ITR1");
        assert_eq!((*rec).core.flag, 99);
        assert_eq!((*rec).core.tid, 0);
        assert_eq!((*rec).core.pos, 4);
        assert_eq!((*rec).core.qual, 40);
        assert_eq!((*rec).core.l_qseq, 4);
        assert_eq!(bam_endpos(rec), 8);
        assert_eq!((*rec).core.n_cigar, 1);
        assert_eq!(bam_cigar_op(*bam_get_cigar(rec)), BAM_CMATCH);
        assert_eq!(bam_cigar_oplen(*bam_get_cigar(rec)), 4);

        let mut names = vec!["ITR1".to_string()];
        loop {
            let ret = sam_read1(fp, hdr, rec);
            if ret < 0 {
                break;
            }
            names.push(
                CStr::from_ptr(bam_get_qname(rec))
                    .to_string_lossy()
                    .into_owned(),
            );
        }

        assert_eq!(names.len(), 16);
        assert_eq!(names[1], "ITR2");
        assert_eq!(names[2], "ITR2M");
        assert_eq!(names[3], "ITR1M");
        assert_eq!(names[15], "B5");

        bam_destroy1(rec);
        sam_hdr_destroy(hdr);
        assert_eq!(htslib_rs::hts_close(fp), 0);
    }
}

#[test]
fn indexes_and_fetches_demo_sample_fasta_sequences() {
    unsafe {
        let fasta = c_fixture("htslib/samples/sample.ref.fa");
        let fai_path = temp_index_path("sample-ref-fa", ".fai");
        assert_eq!(
            fai_build3(fasta.as_ptr(), fai_path.as_ptr(), std::ptr::null()),
            0
        );

        let fai = fai_load3_format(
            fasta.as_ptr(),
            fai_path.as_ptr(),
            std::ptr::null(),
            0,
            FAI_FASTA,
        );
        assert!(!fai.is_null());

        assert_eq!(faidx_nseq(fai), 2);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"T1");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)), c"T2");
        assert_eq!(faidx_has_seq(fai, c"T1".as_ptr()), 1);
        assert_eq!(faidx_has_seq(fai, c"T3".as_ptr()), 0);
        assert_eq!(faidx_seq_len(fai, c"T1".as_ptr()), 40);
        assert_eq!(faidx_seq_len(fai, c"T2".as_ptr()), 40);

        assert_eq!(fetch_text(fai, c"T1:1-12"), "AAAAACTGAAAA");
        assert_eq!(fetch_text(fai, c"T1:33-40"), "CAGTTTTT");
        assert_eq!(fetch_text(fai, c"T2:1-12"), "TTTTCCCCACTG");
        assert_eq!(fetch_text(fai, c"T2:29-40"), "ACTGTTAACAGT");

        fai_destroy(fai);
    }
}

#[test]
fn indexes_and_fetches_demo_sample_fastq_sequences_and_qualities() {
    unsafe {
        let fastq = c_fixture("htslib/samples/sample.ref.fq");
        let fqi_path = temp_index_path("sample-ref-fq", ".fqi");
        assert_eq!(
            fai_build3(fastq.as_ptr(), fqi_path.as_ptr(), std::ptr::null()),
            0
        );

        let fai = fai_load3_format(
            fastq.as_ptr(),
            fqi_path.as_ptr(),
            std::ptr::null(),
            0,
            FAI_FASTQ,
        );
        assert!(!fai.is_null());

        assert_eq!(faidx_nseq(fai), 4);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"T1");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)), c"T2");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 2)), c"T3");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 3)), c"T4");
        assert_eq!(faidx_seq_len(fai, c"T1".as_ptr()), 40);
        assert_eq!(faidx_seq_len(fai, c"T3".as_ptr()), 20);
        assert_eq!(faidx_seq_len(fai, c"T4".as_ptr()), 100);

        assert_eq!(fetch_text(fai, c"T1:1-12"), "AAAAACTGAAAA");
        assert_eq!(fetch_qual_text(fai, c"T1:1-12"), "AAAAACTGAAAA");
        assert_eq!(fetch_text(fai, c"T3:1-20"), "TTTTGGGGACTGTTAACAGT");
        assert_eq!(fetch_qual_text(fai, c"T3:1-20"), "TTTTGGGGACTGTTAACAGT");
        assert_eq!(fetch_text(fai, c"T4:81-100"), "TTTTGGGGACTGTTAACAGT");

        fai_destroy(fai);
    }
}

#[test]
fn parses_demo_sample_bed_with_regidx_and_queries_overlaps() {
    unsafe {
        let path = c_fixture("htslib/samples/sample.bed");
        let idx = regidx_c_246_regidx_init(path.as_ptr(), None, None, 0, std::ptr::null_mut());
        assert!(!idx.is_null());

        assert_eq!(regidx_c_98_regidx_nregs(idx), 4);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"T1".as_ptr()), 2);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"T2".as_ptr()), 2);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"T3".as_ptr()), 0);

        let itr = regidx_c_584_regitr_init(idx);
        assert!(!itr.is_null());

        assert_eq!(
            regidx_c_401_regidx_overlap(idx, c"T1".as_ptr(), 1, 1, itr),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
        assert_eq!((*itr).beg, 1);
        assert_eq!((*itr).end, 1);
        assert_eq!(regidx_c_612_regitr_overlap(itr), 0);

        assert_eq!(
            regidx_c_401_regidx_overlap(idx, c"T1".as_ptr(), 30, 34, itr),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
        assert_eq!((*itr).beg, 30);
        assert_eq!((*itr).end, 34);

        assert_eq!(
            regidx_c_401_regidx_overlap(idx, c"T2".as_ptr(), 39, 39, itr),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
        assert_eq!((*itr).beg, 30);
        assert_eq!((*itr).end, 39);

        assert_eq!(
            regidx_c_401_regidx_overlap(idx, c"T2".as_ptr(), 41, 41, itr),
            0
        );
        assert_eq!(
            regidx_c_401_regidx_overlap(idx, c"T3".as_ptr(), 0, 10, itr),
            0
        );

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }
}
