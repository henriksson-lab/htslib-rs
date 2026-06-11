use htslib_rs::{
    bam_destroy1, bam_endpos, bam_get_l_aux, bam_get_qname, bam_init1,
    hts_c_4756_hts_idx_check_local, hts_close, hts_idx_destroy, hts_itr_destroy, hts_open,
    hts_set_fai_filename, sam_c_4553_sam_write1, sam_hdr_destroy, sam_hdr_read, sam_hdr_write,
    sam_index_build, sam_index_load, sam_index_load2, sam_itr_next, sam_itr_querys, sam_read1,
    HTS_FMT_BAI,
};
use std::path::{Path, PathBuf};

fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn c_path(path: &Path) -> Vec<u8> {
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn optional_external_cellsnp_bam() -> Option<PathBuf> {
    let path = PathBuf::from("/husky/henriksson/for_claude/cellsnp/gex_chr22_5m.bam");
    if path.is_file() && path.with_extension("bam.bai").is_file() {
        Some(path)
    } else {
        None
    }
}

unsafe fn query_alignment_names(
    path: &str,
    reference: Option<&str>,
    region: &str,
    max_records: usize,
) -> Vec<String> {
    query_alignment_names_with_index(path, None, reference, region, max_records)
}

unsafe fn query_alignment_names_with_index(
    path: &str,
    index: Option<&str>,
    reference: Option<&str>,
    region: &str,
    max_records: usize,
) -> Vec<String> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr().cast(), c"r".as_ptr());
    assert!(
        !fp.is_null(),
        "failed to open {}",
        String::from_utf8_lossy(&path[..path.len() - 1])
    );
    if let Some(reference) = reference {
        let reference = c_fixture(reference);
        assert_eq!(hts_set_fai_filename(fp, reference.as_ptr().cast()), 0);
    }

    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let index_c;
    let idx = if let Some(index) = index {
        index_c = c_fixture(index);
        sam_index_load2(fp, path.as_ptr().cast(), index_c.as_ptr().cast())
    } else {
        sam_index_load(fp, path.as_ptr().cast())
    };
    assert!(!idx.is_null());

    let mut region = region.as_bytes().to_vec();
    region.push(0);
    let itr = sam_itr_querys(idx, hdr, region.as_ptr().cast());
    assert!(!itr.is_null());

    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut names = Vec::new();
    while names.len() < max_records {
        let ret = sam_itr_next(fp, itr, rec);
        if ret < 0 {
            break;
        }
        let qname = bam_get_qname(rec);
        let bytes = std::ffi::CStr::from_ptr(qname.cast()).to_bytes();
        names.push(String::from_utf8_lossy(bytes).into_owned());
    }

    bam_destroy1(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    names
}

unsafe fn query_alignment_count_for_path(path: &Path, region: &[u8]) -> usize {
    let path_c = c_path(path);
    let fp = hts_open(path_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let idx = sam_index_load(fp, path_c.as_ptr().cast());
    assert!(!idx.is_null());
    let itr = sam_itr_querys(idx, hdr, region.as_ptr().cast());
    assert!(!itr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut count = 0usize;
    loop {
        let ret = sam_itr_next(fp, itr, rec);
        if ret < 0 {
            break;
        }
        count += 1;
    }

    bam_destroy1(rec);
    hts_itr_destroy(itr);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    count
}

unsafe fn scan_alignment_prefix_for_path(path: &Path, limit: usize) -> (usize, usize, i64, i64) {
    let path_c = c_path(path);
    let fp = hts_open(path_c.as_ptr().cast(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.display());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());

    let mut count = 0usize;
    let mut with_aux = 0usize;
    let mut min_pos = i64::MAX;
    let mut max_end = i64::MIN;
    while count < limit {
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            break;
        }
        assert!(!bam_get_qname(rec).is_null());
        let tid = (*rec).core.tid;
        if tid >= 0 {
            assert!(tid < (*hdr).n_targets);
            assert!((*rec).core.pos >= 0);
            let end = bam_endpos(rec);
            assert!(end >= (*rec).core.pos);
            min_pos = min_pos.min((*rec).core.pos);
            max_end = max_end.max(end);
        }
        if bam_get_l_aux(rec) > 0 {
            with_aux += 1;
        }
        count += 1;
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    (count, with_aux, min_pos, max_end)
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
fn indexed_range_bam_explicit_sidecar_index_matches_default_query_results() {
    unsafe {
        let default_names =
            query_alignment_names("htslib/test/range.bam", None, "CHROMOSOME_II:2976-3070", 8);
        let explicit_names = query_alignment_names_with_index(
            "htslib/test/range.bam",
            Some("htslib/test/range.bam.bai"),
            None,
            "CHROMOSOME_II:2976-3070",
            8,
        );

        assert_eq!(explicit_names, default_names);
        assert_eq!(
            explicit_names.first().map(String::as_str),
            Some("HS18_09653:4:2112:13048:11874")
        );
        assert!(explicit_names.len() > 1);
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

#[test]
fn explicit_real_csi_index_returns_no_sq_bam_fixture_records() {
    unsafe {
        let csi_names = query_alignment_names_with_index(
            "htslib/test/no_hdr_sq_1.bam",
            Some("htslib/test/no_hdr_sq_1.bam.csi"),
            None,
            "CHROMOSOME_I:2-2",
            8,
        );

        assert_eq!(csi_names, ["I", "II.14978392", "III", "IV", "V", "VI"]);
    }
}

#[test]
fn external_cellsnp_chr22_bam_index_query_counts_all_real_records_when_available() {
    let Some(path) = optional_external_cellsnp_bam() else {
        return;
    };
    unsafe {
        assert_eq!(query_alignment_count_for_path(&path, b"chr22\0"), 5_000_000);
        assert_eq!(
            query_alignment_count_for_path(&path, b"chr22:1-1000000\0"),
            0
        );
    }
}

#[test]
fn external_cellsnp_chr22_bam_prefix_scan_has_realistic_mapped_records_when_available() {
    let Some(path) = optional_external_cellsnp_bam() else {
        return;
    };
    unsafe {
        let (count, with_aux, min_pos, max_end) = scan_alignment_prefix_for_path(&path, 100_000);
        assert_eq!(count, 100_000);
        assert!(with_aux > 90_000, "expected most records to carry aux tags");
        assert!(min_pos >= 0);
        assert!(max_end > min_pos);
    }
}

fn temp_index2_bam(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-index2-{}-{}.bam",
        std::process::id(),
        label
    ))
}

fn repo_temp_name(label: &str) -> String {
    format!("htslib_rs-index-{}-{}", std::process::id(), label)
}

unsafe fn build_index2_bam(label: &str) -> std::path::PathBuf {
    let sam_path = c_fixture("htslib/test/index2.sam");
    let bam_path = temp_index2_bam(label);
    let mut bam_path_c = bam_path.to_string_lossy().into_owned().into_bytes();
    bam_path_c.push(0);

    let in_fp = hts_open(sam_path.as_ptr().cast(), c"r".as_ptr());
    assert!(!in_fp.is_null());
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_fp = hts_open(bam_path_c.as_ptr().cast(), c"wb".as_ptr());
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

    assert_eq!(sam_index_build(bam_path_c.as_ptr().cast(), 0), 0);
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

#[test]
fn remote_index_check_probes_local_basename_like_htslib() {
    unsafe {
        let base = repo_temp_name("remote-basename.bam");
        let csi = format!("{base}.csi");
        std::fs::write(&csi, b"not read by hts_idx_check_local").unwrap();

        let mut remote = format!("mem:/path/{base}").into_bytes();
        remote.push(0);
        let mut fnidx = std::ptr::null_mut();
        assert_eq!(
            hts_c_4756_hts_idx_check_local(remote.as_ptr().cast(), HTS_FMT_BAI, &mut fnidx),
            1
        );
        assert!(!fnidx.is_null());
        assert_eq!(
            std::ffi::CStr::from_ptr(fnidx.cast()).to_bytes(),
            csi.as_bytes()
        );

        std::fs::remove_file(csi).unwrap();
    }
}
