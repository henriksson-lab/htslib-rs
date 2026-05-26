use htslib_rs::{
    bcf_destroy, bcf_hdr_destroy, bcf_hdr_get_version, bcf_hdr_id2name, bcf_hdr_name2id,
    bcf_index_build2, bcf_index_load2, bcf_init, bcf_readrec, hts_close, hts_get_bgzfp,
    hts_idx_destroy, hts_idx_get_n_no_coor, hts_idx_nseq, hts_itr_destroy, hts_itr_next,
    hts_itr_query, hts_open, hts_pos_t, vcf_hdr_read, BGZF,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn fixture_path(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_path(path: &std::path::Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let dir = fixture_path(".tmp").join("real_data_variant_index_more");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{}-{}", std::process::id(), name))
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

unsafe fn count_indexed_records(
    path: &CStr,
    index_path: &CStr,
    chrom: &CStr,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> usize {
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null());

    let hdr = vcf_hdr_read(fp);
    assert!(!hdr.is_null());
    let tid = bcf_hdr_name2id(hdr, chrom.as_ptr());
    assert!(tid >= 0);

    let idx = bcf_index_load2(path.as_ptr(), index_path.as_ptr());
    assert!(!idx.is_null());
    let itr = hts_itr_query(idx, tid, beg, end, Some(bcf_readrec_adapter));
    assert!(!itr.is_null());

    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut count = 0;
    loop {
        let ret = hts_itr_next(hts_get_bgzfp(fp), itr, rec.cast(), std::ptr::null_mut());
        if ret < 0 {
            break;
        }
        count += 1;
    }

    bcf_destroy(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    count
}

#[test]
fn modhdr_csi_preserves_sparse_header_contig_id_and_translated_index_shape() {
    unsafe {
        let vcf = c_fixture("htslib/test/modhdr.vcf.gz");
        let csi = c_fixture("htslib/test/modhdr.vcf.gz.csi");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.3");
        assert!(bcf_hdr_id2name(hdr, 0).is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_id2name(hdr, 1)), c"chr22");
        let tid = bcf_hdr_name2id(hdr, c"chr22".as_ptr());
        assert_eq!(tid, 1);

        let idx = bcf_index_load2(vcf.as_ptr(), csi.as_ptr());
        assert!(!idx.is_null());
        assert_eq!(hts_idx_nseq(idx), 0);
        assert_eq!(hts_idx_get_n_no_coor(idx), 0);

        hts_idx_destroy(idx);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn generated_bcf_csi_queries_original_tabix_variant_fixture_intervals() {
    unsafe {
        let bcf = c_fixture("htslib/test/tabix/vcf_file.bcf");
        let csi_path = temp_path("vcf_file.bcf.csi");
        let _ = std::fs::remove_file(&csi_path);
        let csi = c_path(&csi_path);

        assert_eq!(bcf_index_build2(bcf.as_ptr(), csi.as_ptr(), 14), 0);

        assert_eq!(
            count_indexed_records(&bcf, &csi, c"1", 3_000_150, 3_000_151),
            1
        );
        assert_eq!(
            count_indexed_records(&bcf, &csi, c"1", 3_062_914, 3_062_915),
            2
        );
        assert_eq!(
            count_indexed_records(&bcf, &csi, c"4", 3_258_447, 3_258_456),
            1
        );

        let _ = std::fs::remove_file(csi_path);
    }
}

#[test]
fn modhdr_csi_missing_header_contig_does_not_create_query_iterator() {
    unsafe {
        let vcf = c_fixture("htslib/test/modhdr.vcf.gz");
        let csi = c_fixture("htslib/test/modhdr.vcf.gz.csi");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let missing_tid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
        assert_eq!(missing_tid, -1);

        let idx = bcf_index_load2(vcf.as_ptr(), csi.as_ptr());
        assert!(!idx.is_null());
        assert!(hts_itr_query(idx, missing_tid, 0, 1, Some(bcf_readrec_adapter),).is_null());

        hts_idx_destroy(idx);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}
