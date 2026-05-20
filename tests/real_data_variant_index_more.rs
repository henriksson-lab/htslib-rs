use htslib_rs::{
    bcf_hdr_destroy, bcf_hdr_get_version, bcf_hdr_id2name, bcf_hdr_name2id, bcf_index_load2,
    bcf_readrec, hts_close, hts_idx_destroy, hts_idx_get_n_no_coor, hts_idx_nseq, hts_itr_query,
    hts_open, hts_pos_t, vcf_hdr_read, BGZF,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
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
