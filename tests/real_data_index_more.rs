use htslib_rs::{
    bam_destroy1, bam_get_qname, bam_init1, hts_close, hts_idx_destroy, hts_itr_destroy, hts_open,
    hts_set_fai_filename, sam_c_4553_sam_write1, sam_hdr_destroy, sam_hdr_read, sam_hdr_write,
    sam_index_build, sam_index_load, sam_itr_next, sam_itr_querys, sam_read1,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn query_alignment_names(
    path: &str,
    reference: Option<&str>,
    region: &str,
    max_records: usize,
) -> Vec<String> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let idx = sam_index_load(fp, path.as_ptr());
    assert!(!idx.is_null());

    let region = CString::new(region).unwrap();
    let itr = sam_itr_querys(idx, hdr, region.as_ptr());
    assert!(!itr.is_null());

    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut names = Vec::new();
    while names.len() < max_records {
        let ret = sam_itr_next(fp, itr, rec);
        if ret < 0 {
            break;
        }
        names.push(
            CStr::from_ptr(bam_get_qname(rec))
                .to_string_lossy()
                .into_owned(),
        );
    }

    bam_destroy1(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    names
}

#[test]
fn indexed_range_bam_queries_return_expected_real_records() {
    unsafe {
        let names =
            query_alignment_names("htslib/test/range.bam", None, "CHROMOSOME_II:2976-2976", 3);
        assert_eq!(
            names.first().map(String::as_str),
            Some("HS18_09653:4:2112:13048:11874")
        );
        assert!(!names.is_empty());

        let names =
            query_alignment_names("htslib/test/range.bam", None, "CHROMOSOME_IV:1422-1483", 4);
        assert_eq!(
            names.first().map(String::as_str),
            Some("HS18_09653:4:1104:14796:5124")
        );
        assert!(names
            .iter()
            .any(|name| name == "HS18_09653:4:1303:14310:57578"));
    }
}

#[test]
fn indexed_colon_named_bam_queries_resolve_braced_real_regions() {
    unsafe {
        let names = query_alignment_names("htslib/test/colons.bam", None, "{chr1}:1-1000", 2);
        assert!(!names.is_empty());

        let names = query_alignment_names("htslib/test/colons.bam", None, "{chr1:100-200}", 2);
        assert!(!names.is_empty());
    }
}

#[test]
fn indexed_real_cram_query_uses_reference_and_returns_records() {
    unsafe {
        let names = query_alignment_names(
            "htslib/test/range.cram",
            Some("htslib/test/ce.fa"),
            "CHROMOSOME_II:2976-2976",
            3,
        );
        assert_eq!(
            names.first().map(String::as_str),
            Some("HS18_09653:4:2112:13048:11874")
        );
    }
}

fn temp_index2_bam(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-index2-{}-{}.bam",
        std::process::id(),
        label
    ))
}

unsafe fn build_index2_bam(label: &str) -> std::path::PathBuf {
    let sam_path = c_fixture("htslib/test/index2.sam");
    let bam_path = temp_index2_bam(label);
    let bam_path_c = CString::new(bam_path.to_string_lossy().as_bytes()).unwrap();

    let in_fp = hts_open(sam_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null());
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_fp = hts_open(bam_path_c.as_ptr(), c"wb".as_ptr());
    assert!(!out_fp.is_null());
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) > 0);
    }

    bam_destroy1(rec);
    assert_eq!(hts_close(out_fp), 0);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);

    assert_eq!(sam_index_build(bam_path_c.as_ptr(), 0), 0);
    bam_path
}

#[test]
fn indexed_index2_mapped_unmapped_mate_point_queries_match_htslib_counts() {
    unsafe {
        let bam_path = build_index2_bam("mate-point");
        let bam_path_s = bam_path.to_string_lossy().into_owned();

        for (region, expected_name) in [
            ("1:1000000-1000000", "um1"),
            ("1:2000000-2000000", "um2"),
            ("2:1000000-1000000", "mu1"),
            ("2:2000000-2000000", "mu2"),
        ] {
            let names = query_alignment_names(&bam_path_s, None, region, 3);
            assert_eq!(
                names,
                [expected_name.to_string(), expected_name.to_string()],
                "region {region} returned the wrong records"
            );
        }

        let _ = std::fs::remove_file(&bam_path);
        let _ = std::fs::remove_file(format!("{}.bai", bam_path.display()));
    }
}
