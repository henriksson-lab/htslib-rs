use htslib_rs::{
    bam_destroy1, bam_get_qname, bam_init1, bcf_destroy, bcf_get_format_values, bcf_hdr_destroy,
    bcf_hdr_id2name, bcf_hdr_name2id, bcf_hdr_read, bcf_hdr_seqnames, bcf_hdr_set_samples,
    bcf_index_load2, bcf_init, bcf_read, bcf_readrec, bcf_seqname,
    bgzf::{bgzf_check_EOF, bgzf_close, bgzf_is_bgzf, bgzf_open, bgzf_read},
    fai_destroy, fai_fetch, fai_fetchqual, fai_load, fai_load3_format, faidx_has_seq, faidx_iseq,
    faidx_nseq, faidx_seq_len, hclose, hopen, htsFormat, hts_check_EOF, hts_close,
    hts_detect_format2, hts_get_bgzfp, hts_idx_destroy, hts_itr_destroy, hts_itr_next,
    hts_itr_query, hts_open, hts_pos_t, hts_set_fai_filename, sam_hdr_destroy, sam_hdr_name2tid,
    sam_hdr_nref, sam_hdr_read, sam_hdr_t, sam_hdr_tid2len, sam_hdr_tid2name, sam_index_load,
    sam_itr_next, sam_itr_queryi, sam_itr_querys, sam_parse_region, sam_read1, vcf_hdr_read, BGZF,
    FAI_FASTA, FAI_FASTQ, HTS_FORMAT_BAI, HTS_FORMAT_BAM, HTS_FORMAT_BCF, HTS_FORMAT_CRAI_EXACT,
    HTS_FORMAT_CRAM, HTS_FORMAT_CSI, HTS_FORMAT_EMPTY_FORMAT, HTS_FORMAT_GZI, HTS_FORMAT_SAM,
    HTS_FORMAT_TBI, HTS_FORMAT_VCF, HTS_PARSE_LIST, HTS_PARSE_ONE_COORD, HTS_POS_MAX,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn detect_fixture_format(path: &str) -> htsFormat {
    unsafe {
        let path = c_fixture(path);
        let fp = hopen(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        let mut fmt: htsFormat = std::mem::zeroed();
        assert_eq!(hts_detect_format2(fp, path.as_ptr(), &mut fmt), 0);
        assert_eq!(hclose(fp), 0);
        fmt
    }
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

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    count
}

unsafe fn count_variant_records(path: &str) -> usize {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

    let hdr = bcf_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bcf_init();
    assert!(!rec.is_null());

    let mut count = 0;
    loop {
        let ret = bcf_read(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        count += 1;
    }

    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    count
}

unsafe extern "C" fn bcf_readrec_adapter(
    fp: *mut BGZF,
    data: *mut c_void,
    rec: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    unsafe { bcf_readrec(fp, data, rec, tid, beg, end) }
}

unsafe fn expect_sam_region(
    hdr: *mut sam_hdr_t,
    region: &str,
    flags: c_int,
    expected_rest: &str,
    expected_tid: c_int,
    expected_beg: hts_pos_t,
    expected_end: hts_pos_t,
) {
    let region = CString::new(region).unwrap();
    let mut tid = -1;
    let mut beg = -1;
    let mut end = -1;
    let rest =
        unsafe { sam_parse_region(hdr, region.as_ptr(), &mut tid, &mut beg, &mut end, flags) };
    assert!(!rest.is_null());
    assert_eq!(
        unsafe { CStr::from_ptr(rest) }.to_bytes(),
        expected_rest.as_bytes()
    );
    assert_eq!((tid, beg, end), (expected_tid, expected_beg, expected_end));
}

#[test]
fn reads_real_bgzf_fixture() {
    unsafe {
        let path = c_fixture("htslib/test/bgziptest.txt.gz");
        let fp = bgzf_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let mut buf = [0u8; 32];
        let n = bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len());
        assert!(n > 0);
        assert!(buf[..n as usize].iter().any(|&b| b != 0));

        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn checks_real_bgzf_fixture_eof_marker() {
    unsafe {
        let path = c_fixture("htslib/test/bgziptest.txt.gz");
        assert_eq!(bgzf_is_bgzf(path.as_ptr()), 1);

        let fp = bgzf_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        assert_eq!(bgzf_check_EOF(fp), 1);

        assert_eq!(bgzf_close(fp), 0);
    }
}

#[test]
fn detects_real_compressed_and_index_fixtures() {
    let sam = detect_fixture_format("htslib/test/index.sam");
    assert_eq!(sam.format, HTS_FORMAT_SAM);

    let bam = detect_fixture_format("htslib/test/range.bam");
    assert_eq!(bam.format, HTS_FORMAT_BAM);

    let cram = detect_fixture_format("htslib/test/range.cram");
    assert_eq!(cram.format, HTS_FORMAT_CRAM);

    let bgzf = detect_fixture_format("htslib/test/bgziptest.txt.gz");
    assert_eq!(bgzf.compression, hts_sys::htsCompression_bgzf);

    let vcf_gz = detect_fixture_format("htslib/test/modhdr.vcf.gz");
    assert_eq!(vcf_gz.format, HTS_FORMAT_VCF);
    assert_eq!(vcf_gz.compression, hts_sys::htsCompression_bgzf);

    let bai = detect_fixture_format("htslib/test/index.bam.bai");
    assert_eq!(bai.format, HTS_FORMAT_BAI);

    let csi = detect_fixture_format("htslib/test/index.bam.csi");
    assert_eq!(csi.format, HTS_FORMAT_CSI);

    let crai = detect_fixture_format("htslib/test/range.cram.crai");
    assert_eq!(crai.format, HTS_FORMAT_CRAI_EXACT);

    let tbi = detect_fixture_format("htslib/test/index.vcf.gz.tbi");
    assert_eq!(tbi.format, HTS_FORMAT_TBI);

    let bcf = detect_fixture_format("htslib/test/tabix/vcf_file.bcf");
    assert_eq!(bcf.format, HTS_FORMAT_BCF);

    let gzi = detect_fixture_format("htslib/test/bgziptest.txt.gz.gzi");
    assert_eq!(gzi.format, HTS_FORMAT_GZI);

    let empty = detect_fixture_format("htslib/test/emptyfile");
    assert_eq!(empty.format, HTS_FORMAT_EMPTY_FORMAT);
}

#[test]
fn scans_multiple_real_alignment_fixtures() {
    unsafe {
        assert_eq!(count_alignment_records("htslib/test/ce#1.sam", None), 1);
        assert_eq!(count_alignment_records("htslib/test/xx#pair.sam", None), 6);
        assert_eq!(count_alignment_records("htslib/test/c1#pad1.sam", None), 9);
        assert_eq!(count_alignment_records("htslib/test/index.sam", None), 181);

        assert!(count_alignment_records("htslib/test/range.bam", None) > 0);
        assert!(
            count_alignment_records("htslib/test/bgzf_boundaries/bgzf_boundaries1.bam", None) > 0
        );
        assert!(
            count_alignment_records("htslib/test/bgzf_boundaries/bgzf_boundaries2.bam", None) > 0
        );
        assert!(
            count_alignment_records("htslib/test/bgzf_boundaries/bgzf_boundaries3.bam", None) > 0
        );

        assert!(count_alignment_records("htslib/test/range.cram", Some("htslib/test/ce.fa")) > 0);
    }
}

#[test]
fn scans_real_alignment_edge_case_fixtures() {
    unsafe {
        assert_eq!(count_alignment_records("htslib/test/ce#5.sam", None), 6);
        assert_eq!(
            count_alignment_records("htslib/test/ce#1000.sam", None),
            1000
        );
        assert_eq!(
            count_alignment_records("htslib/test/xx#minimal.sam", None),
            8
        );
        assert_eq!(
            count_alignment_records("htslib/test/xx#triplet.sam", None),
            5
        );
        assert_eq!(
            count_alignment_records("htslib/test/xx#large_aux.sam", None),
            3
        );
        assert_eq!(
            count_alignment_records("htslib/test/xx#large_aux2.sam", None),
            10
        );
        assert_eq!(
            count_alignment_records("htslib/test/auxf#values.sam", None),
            2
        );
        assert_eq!(
            count_alignment_records("htslib/test/mpileup/mp_D.sam", None),
            3
        );
        assert_eq!(
            count_alignment_records("htslib/test/mpileup/mp_DI.sam", None),
            5
        );
    }
}

#[test]
fn scans_real_base_modification_and_tlen_fixtures() {
    unsafe {
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-explicit.sam", None),
            3
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-multi.sam", None),
            2
        );
        assert_eq!(
            count_alignment_records("htslib/test/base_mods/MM-variants.sam", None),
            8
        );
        assert_eq!(count_alignment_records("htslib/test/tlen/a7.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/b7.sam", None), 4);
        assert_eq!(count_alignment_records("htslib/test/tlen/d5.sam", None), 3);

        assert!(count_alignment_records("htslib/test/tlen/a7.cram", None) > 0);
        assert!(count_alignment_records("htslib/test/tlen/b7.cram", None) > 0);
        assert!(count_alignment_records("htslib/test/tlen/d5.cram", None) > 0);
    }
}

#[test]
fn scans_multiple_real_variant_fixtures() {
    unsafe {
        assert_eq!(count_variant_records("htslib/test/index.vcf"), 621);
        assert_eq!(count_variant_records("htslib/test/formatcols.vcf"), 1);
        assert_eq!(count_variant_records("htslib/test/formatmissing.vcf"), 1);
        assert_eq!(count_variant_records("htslib/test/tabix/vcf_file.vcf"), 15);
        assert_eq!(
            count_variant_records("htslib/test/bcf-sr/weird-chr-names.vcf"),
            6
        );
        assert_eq!(count_variant_records("htslib/test/tabix/vcf_file.bcf"), 15);
    }
}

#[test]
fn scans_real_variant_edge_case_fixtures() {
    unsafe {
        assert_eq!(count_variant_records("htslib/test/longrefs/index.vcf"), 192);
        assert_eq!(
            count_variant_records("htslib/test/bcf-sr/merge.noidx.a.vcf"),
            4
        );
        assert_eq!(
            count_variant_records("htslib/test/bcf-sr/merge.noidx.b.vcf"),
            8
        );
        assert_eq!(
            count_variant_records("htslib/test/bcf-sr/merge.noidx.c.vcf"),
            8
        );
        assert_eq!(count_variant_records("htslib/test/noroundtrip.vcf"), 5);
        assert_eq!(count_variant_records("htslib/test/test-vcf-hdr-in.vcf"), 10);
        assert_eq!(count_variant_records("htslib/test/tabix/large_chr.vcf"), 12);
    }
}

#[test]
fn reads_multiple_real_fasta_indexes() {
    unsafe {
        let c1 = c_fixture("htslib/test/c1.fa");
        let c1_fai = c_fixture("htslib/test/c1.fa.fai");
        let fai = fai_load3_format(c1.as_ptr(), c1_fai.as_ptr(), std::ptr::null(), 0, FAI_FASTA);
        assert!(!fai.is_null());
        assert_eq!(faidx_nseq(fai), 1);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"c1");
        assert_eq!(faidx_seq_len(fai, c"c1".as_ptr()), 10);
        fai_destroy(fai);

        let c2 = c_fixture("htslib/test/c2.fa");
        let c2_fai = c_fixture("htslib/test/c2.fa.fai");
        let fai = fai_load3_format(c2.as_ptr(), c2_fai.as_ptr(), std::ptr::null(), 0, FAI_FASTA);
        assert!(!fai.is_null());
        assert_eq!(faidx_nseq(fai), 1);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"c2");
        assert_eq!(faidx_seq_len(fai, c"c2".as_ptr()), 9);
        fai_destroy(fai);

        let xx = c_fixture("htslib/test/xx.fa");
        let xx_fai = c_fixture("htslib/test/xx.fa.fai");
        let fai = fai_load3_format(xx.as_ptr(), xx_fai.as_ptr(), std::ptr::null(), 0, FAI_FASTA);
        assert!(!fai.is_null());
        assert_eq!(faidx_nseq(fai), 3);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"xx");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)), c"yy");
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 2)), c"zz");
        assert_eq!(faidx_seq_len(fai, c"xx".as_ptr()), 20);
        assert_eq!(faidx_seq_len(fai, c"yy".as_ptr()), 20);
        assert_eq!(faidx_seq_len(fai, c"zz".as_ptr()), 30);
        fai_destroy(fai);
    }
}

#[test]
fn reads_real_bam_fixture() {
    unsafe {
        let bam = c_fixture("htslib/test/range.bam");
        let fp = hts_open(bam.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());

        let rec = bam_init1();
        assert!(!rec.is_null());
        assert!(sam_read1(fp, hdr, rec) >= 0);

        bam_destroy1(rec);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn reads_real_sam_header_and_record_metadata() {
    unsafe {
        let sam = c_fixture("htslib/test/index.sam");
        let fp = hts_open(sam.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(sam_hdr_nref(hdr), 7);
        assert_eq!(sam_hdr_name2tid(hdr, c"CHROMOSOME_I".as_ptr()), 0);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 0)), c"CHROMOSOME_I");
        assert_eq!(sam_hdr_tid2len(hdr, 0), 1_009_800);

        let rec = bam_init1();
        assert!(!rec.is_null());
        assert!(sam_read1(fp, hdr, rec) >= 0);
        assert_eq!(
            CStr::from_ptr(bam_get_qname(rec)).to_bytes(),
            b"SRR065390.17240207"
        );
        assert_eq!((*rec).core.tid, 0);
        assert_eq!((*rec).core.pos, 999_900);
        assert_eq!((*rec).core.qual, 42);
        assert_eq!((*rec).core.l_qseq, 100);

        bam_destroy1(rec);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn queries_real_indexed_bam_fixture() {
    unsafe {
        let bam = c_fixture("htslib/test/range.bam");
        let fp = hts_open(bam.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        let idx = sam_index_load(fp, bam.as_ptr());
        assert!(!idx.is_null());
        let itr = sam_itr_queryi(idx, 0, 0, 1_000_000);
        assert!(!itr.is_null());

        let rec = bam_init1();
        assert!(!rec.is_null());
        assert!(sam_itr_next(fp, itr, rec) >= 0);

        bam_destroy1(rec);
        hts_itr_destroy(itr);
        hts_idx_destroy(idx);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn parses_real_bam_header_regions_with_colons_and_lists() {
    unsafe {
        let bam = c_fixture("htslib/test/colons.bam");
        let fp = hts_open(bam.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());

        expect_sam_region(hdr, "chr1", 0, "", 0, 0, HTS_POS_MAX);
        expect_sam_region(hdr, "{chr1}:100-200", 0, "", 0, 99, 200);
        expect_sam_region(hdr, "{chr1:100-200}", 0, "", 2, 0, HTS_POS_MAX);
        expect_sam_region(hdr, "chr3:1,000-1,500", 0, "", 4, 999, 1500);
        expect_sam_region(hdr, "chr1:50", HTS_PARSE_ONE_COORD, "", 0, 49, 50);
        expect_sam_region(hdr, "chr1,chr3", HTS_PARSE_LIST, "chr3", 0, 0, HTS_POS_MAX);
        expect_sam_region(hdr, "{chr1,chr3}", HTS_PARSE_LIST, "", 5, 0, HTS_POS_MAX);

        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn special_dot_region_queries_real_bam_from_start() {
    unsafe {
        let bam = c_fixture("htslib/test/range.bam");
        let fp = hts_open(bam.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        let idx = sam_index_load(fp, bam.as_ptr());
        assert!(!idx.is_null());
        let itr = sam_itr_querys(idx, hdr, c".".as_ptr());
        assert!(!itr.is_null());

        let rec = bam_init1();
        assert!(!rec.is_null());
        assert!(sam_itr_next(fp, itr, rec) >= 0);

        bam_destroy1(rec);
        hts_itr_destroy(itr);
        hts_idx_destroy(idx);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn reads_real_cram_fixture() {
    unsafe {
        let cram = c_fixture("htslib/test/range.cram");
        let reference = c_fixture("htslib/test/ce.fa");
        let fp = hts_open(cram.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());

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
}

#[test]
fn queries_real_indexed_cram_fixture() {
    unsafe {
        let cram = c_fixture("htslib/test/range.cram");
        let reference = c_fixture("htslib/test/ce.fa");
        let fp = hts_open(cram.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);

        let hdr = sam_hdr_read(fp);
        assert!(!hdr.is_null());
        let idx = sam_index_load(fp, cram.as_ptr());
        assert!(!idx.is_null());
        let itr = sam_itr_queryi(idx, 0, 0, 1_000_000);
        assert!(!itr.is_null());

        let rec = bam_init1();
        assert!(!rec.is_null());
        assert!(sam_itr_next(fp, itr, rec) >= 0);

        bam_destroy1(rec);
        hts_itr_destroy(itr);
        hts_idx_destroy(idx);
        sam_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn reads_real_vcf_fixture() {
    unsafe {
        let vcf = c_fixture("htslib/test/index.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(bcf_hdr_name2id(hdr, c"1".as_ptr()), 0);
        assert_eq!(CStr::from_ptr(bcf_hdr_id2name(hdr, 0)), c"1");

        let mut nseqs = 0;
        let seqs = bcf_hdr_seqnames(hdr, &mut nseqs);
        assert!(!seqs.is_null());
        assert!(nseqs > 0);
        assert_eq!(CStr::from_ptr(*seqs), c"1");
        libc::free(seqs.cast());

        assert_eq!(bcf_hdr_set_samples(hdr, c"ERS220911".as_ptr(), 0), 0);

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert!(bcf_read(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 9_999_918);

        let mut pl = std::ptr::null_mut();
        let mut npl = 0;
        assert_eq!(
            bcf_get_format_values(
                hdr,
                rec,
                c"PL".as_ptr(),
                &mut pl,
                &mut npl,
                hts_sys::BCF_HT_INT as i32,
            ),
            3
        );
        assert!(npl >= 3);
        assert_eq!(std::slice::from_raw_parts(pl.cast::<i32>(), 3), &[0, 3, 26]);
        libc::free(pl);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn reads_real_compressed_vcf_header_fixture() {
    unsafe {
        let vcf = c_fixture("htslib/test/modhdr.vcf.gz");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(bcf_hdr_name2id(hdr, c"chr22".as_ptr()), 1);

        let mut nseqs = 0;
        let seqs = bcf_hdr_seqnames(hdr, &mut nseqs);
        assert!(!seqs.is_null());
        assert_eq!(nseqs, 1);
        assert_eq!(CStr::from_ptr(*seqs), c"chr22");
        libc::free(seqs.cast());

        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn queries_real_indexed_compressed_vcf_fixture() {
    unsafe {
        let vcf = c_fixture("htslib/test/modhdr.vcf.gz");
        let csi = c_fixture("htslib/test/modhdr.vcf.gz.csi");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let tid = bcf_hdr_name2id(hdr, c"chr22".as_ptr());
        assert_eq!(tid, 1);

        let idx = bcf_index_load2(vcf.as_ptr(), csi.as_ptr());
        assert!(!idx.is_null());
        let itr = hts_itr_query(idx, tid, 0, 1_000_000_000, Some(bcf_readrec_adapter));
        assert!(!itr.is_null());

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert_eq!(
            hts_itr_next(hts_get_bgzfp(fp), itr, rec.cast(), std::ptr::null_mut()),
            -1
        );

        bcf_destroy(rec);
        hts_itr_destroy(itr);
        hts_idx_destroy(idx);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn reads_real_fasta_index_fixture() {
    unsafe {
        let fasta = c_fixture("htslib/test/auxf.fa");
        let fai = fai_load(fasta.as_ptr());
        assert!(!fai.is_null());

        let mut len = 0;
        assert_eq!(faidx_has_seq(fai, c"Sheila".as_ptr()), 1);
        assert_eq!(faidx_has_seq(fai, c"missing-contig".as_ptr()), 0);

        let seq = fai_fetch(fai, c"Sheila".as_ptr(), &mut len);
        assert!(!seq.is_null());
        assert!(len > 0);
        libc::free(seq.cast());

        fai_destroy(fai);
    }
}

#[test]
fn reads_real_fastq_index_quality_fixture() {
    unsafe {
        let fastq = c_fixture("htslib/test/faidx/fastqs.fq");
        let fastq_fai = c_fixture("htslib/test/faidx/fastqs.fq.expected.fai");
        let fai = fai_load3_format(
            fastq.as_ptr(),
            fastq_fai.as_ptr(),
            std::ptr::null(),
            0,
            FAI_FASTQ,
        );
        assert!(!fai.is_null());

        assert!(faidx_nseq(fai) > 0);
        assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)), c"FAKE0005_1");
        assert_eq!(faidx_seq_len(fai, c"FAKE0005_1".as_ptr()), 63);

        let mut len = 0;
        let seq = fai_fetch(fai, c"FAKE0005_1:1-4".as_ptr(), &mut len);
        assert!(!seq.is_null());
        assert_eq!(len, 4);
        assert_eq!(CStr::from_ptr(seq).to_bytes(), b"ACGT");
        libc::free(seq.cast());

        let qual = fai_fetchqual(fai, c"FAKE0005_1:1-4".as_ptr(), &mut len);
        assert!(!qual.is_null());
        assert_eq!(len, 4);
        assert_eq!(CStr::from_ptr(qual).to_bytes(), b"@ABC");
        libc::free(qual.cast());

        fai_destroy(fai);
    }
}
