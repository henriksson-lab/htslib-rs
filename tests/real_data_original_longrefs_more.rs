use htslib_rs::{
    bam_destroy1, bam_init1, bcf_destroy, bcf_hdr_destroy, bcf_hdr_get_version, bcf_hdr_id2name,
    bcf_hdr_name2id, bcf_init, bcf_read, hts_close, hts_idx_destroy, hts_itr_destroy, hts_open,
    ks_free, kstring_t, sam_c_1768_sam_itr_regarray, sam_c_4553_sam_write1, sam_format1,
    sam_hdr_destroy, sam_hdr_length, sam_hdr_read, sam_hdr_str, sam_hdr_write, sam_index_build3,
    sam_index_load2, sam_itr_next, sam_itr_querys, sam_read1, vcf_format, vcf_hdr_read,
};
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn kstring_text(ks: &kstring_t) -> String {
    String::from_utf8_lossy(std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l)).into_owned()
}

fn temp_longrefs_path(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "htslib_rs-longrefs-{name}-{}-{nanos}.sam.gz",
        std::process::id()
    ))
}

unsafe fn sam_header_text(hdr: *mut htslib_rs::sam_hdr_t) -> String {
    let len = sam_hdr_length(hdr);
    assert!(!sam_hdr_str(hdr).is_null());
    String::from_utf8(std::slice::from_raw_parts(sam_hdr_str(hdr).cast::<u8>(), len).to_vec())
        .unwrap()
}

unsafe fn append_formatted_record(
    hdr: *mut htslib_rs::sam_hdr_t,
    rec: *const htslib_rs::bam1_t,
    line: &mut kstring_t,
    out: &mut String,
) {
    line.l = 0;
    assert!(sam_format1(hdr, rec, line) >= 0);
    out.push_str(&kstring_text(line));
    out.push('\n');
}

unsafe fn build_longref_bgzf_sam_with_csi(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let in_path = c_fixture("htslib/test/longrefs/longref.sam");
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open longref.sam");
    let hdr = sam_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_path = temp_longrefs_path(name);
    let out_index_path = std::path::PathBuf::from(format!("{}.csi", out_path.display()));
    let out_path_c = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let out_fp = hts_open(out_path_c.as_ptr(), c"wz".as_ptr());
    assert!(!out_fp.is_null(), "failed to open temp BGZF SAM output");
    assert_eq!(sam_hdr_write(out_fp, hdr), 0);

    let rec = bam_init1();
    assert!(!rec.is_null());
    loop {
        let ret = sam_read1(in_fp, hdr, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        assert!(sam_c_4553_sam_write1(out_fp, hdr, rec) >= 0);
    }

    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(hts_close(out_fp), 0);

    let out_index_path_c = CString::new(out_index_path.to_string_lossy().as_bytes()).unwrap();
    assert_eq!(
        sam_index_build3(out_path_c.as_ptr(), out_index_path_c.as_ptr(), 14, 0),
        0
    );
    (out_path, out_index_path)
}

unsafe fn render_longref_single_region(
    path: &std::path::Path,
    index_path: &std::path::Path,
) -> String {
    render_longref_region(path, index_path, c"CHROMOSOME_I:10000000000-10000000003")
}

unsafe fn render_longref_region(
    path: &std::path::Path,
    index_path: &std::path::Path,
    region: &CStr,
) -> String {
    let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
    let index_path_c = CString::new(index_path.to_string_lossy().as_bytes()).unwrap();
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open temp BGZF SAM input");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let idx = sam_index_load2(fp, path_c.as_ptr(), index_path_c.as_ptr());
    assert!(!idx.is_null());
    let iter = sam_itr_querys(idx, hdr, region.as_ptr());
    assert!(!iter.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut line: kstring_t = std::mem::zeroed();
    let mut out = sam_header_text(hdr);

    loop {
        let ret = sam_itr_next(fp, iter, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        append_formatted_record(hdr, rec, &mut line, &mut out);
    }

    ks_free(&mut line);
    bam_destroy1(rec);
    hts_itr_destroy(iter);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn render_longref_multi_region(
    path: &std::path::Path,
    index_path: &std::path::Path,
) -> String {
    let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
    let index_path_c = CString::new(index_path.to_string_lossy().as_bytes()).unwrap();
    let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open temp BGZF SAM input");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let idx = sam_index_load2(fp, path_c.as_ptr(), index_path_c.as_ptr());
    assert!(!idx.is_null());
    let mut regions = [
        CString::new("CHROMOSOME_I:10000000000-10000000003").unwrap(),
        CString::new("CHROMOSOME_I:10000000100-10000000110").unwrap(),
    ];
    let mut region_ptrs = regions
        .iter_mut()
        .map(|region| region.as_ptr().cast_mut())
        .collect::<Vec<_>>();
    let iter = sam_c_1768_sam_itr_regarray(idx, hdr, region_ptrs.as_mut_ptr(), 2);
    assert!(!iter.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut line: kstring_t = std::mem::zeroed();
    let mut out = sam_header_text(hdr);

    loop {
        let ret = sam_itr_next(fp, iter, rec);
        if ret < 0 {
            assert_eq!(ret, -1);
            break;
        }
        append_formatted_record(hdr, rec, &mut line, &mut out);
    }

    ks_free(&mut line);
    bam_destroy1(rec);
    hts_itr_destroy(iter);
    hts_idx_destroy(idx);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

fn remove_temp_longrefs(path: &std::path::Path, index_path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(index_path);
}

unsafe fn formatted_vcf_records(path: &str) -> Vec<String> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

    let hdr = vcf_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bcf_init();
    assert!(!rec.is_null());
    let mut line: kstring_t = std::mem::zeroed();

    let mut records = Vec::new();
    while bcf_read(fp, hdr, rec) >= 0 {
        line.l = 0;
        assert_eq!(vcf_format(hdr, rec, &mut line), 0);
        records.push(kstring_text(&line));
    }

    ks_free(&mut line);
    bcf_destroy(rec);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    records
}

#[test]
fn longrefs_vcf_header_keeps_original_large_contig_and_record_count() {
    unsafe {
        let path = c_fixture("htslib/test/longrefs/index.vcf");
        let fp = hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.2");
        assert_eq!(bcf_hdr_name2id(hdr, c"1".as_ptr()), 0);
        assert_eq!(CStr::from_ptr(bcf_hdr_id2name(hdr, 0)), c"1");

        let rec = bcf_init();
        assert!(!rec.is_null());
        let mut count = 0;
        let mut first = None;
        let mut last = None;
        while bcf_read(fp, hdr, rec) >= 0 {
            let pos = (*rec).pos + 1;
            first.get_or_insert(pos);
            last = Some(pos);
            assert_eq!((*rec).rid, 0);
            count += 1;
        }
        assert_eq!(count, 192);
        assert_eq!(first, Some(10_009_999_919));
        assert_eq!(last, Some(10_010_000_110));

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn longrefs_vcf_format_matches_original_expected_region_window_exactly() {
    unsafe {
        let records = formatted_vcf_records("htslib/test/longrefs/index.vcf");

        let expected1 = include_str!("../htslib/test/longrefs/index.expected1.vcf")
            .lines()
            .map(|line| format!("{line}\n"))
            .collect::<Vec<_>>();
        assert_eq!(&records[181..187], expected1.as_slice());

        let expected2 = include_str!("../htslib/test/longrefs/index.expected2.vcf")
            .lines()
            .map(|line| format!("{line}\n"))
            .collect::<Vec<_>>();
        assert_eq!(&records[191..192], expected2.as_slice());
    }
}

#[test]
fn longrefs_sam_single_region_iterator_matches_original_expected_output_exactly() {
    unsafe {
        let (path, index_path) = build_longref_bgzf_sam_with_csi("single");
        let actual = render_longref_single_region(&path, &index_path);
        remove_temp_longrefs(&path, &index_path);
        assert_eq!(
            actual,
            include_str!("../htslib/test/longrefs/longref_itr.expected.sam")
        );
    }
}

#[test]
fn longrefs_sam_empty_region_iterator_returns_header_only() {
    unsafe {
        let (path, index_path) = build_longref_bgzf_sam_with_csi("empty");
        let actual =
            render_longref_region(&path, &index_path, c"CHROMOSOME_I:10000900000-10000900100");
        remove_temp_longrefs(&path, &index_path);
        assert_eq!(actual, "@SQ\tSN:CHROMOSOME_I\tLN:10001009800\n");
    }
}

#[test]
fn longrefs_sam_multi_region_iterator_matches_original_expected_output_exactly() {
    unsafe {
        let (path, index_path) = build_longref_bgzf_sam_with_csi("multi");
        let actual = render_longref_multi_region(&path, &index_path);
        remove_temp_longrefs(&path, &index_path);
        assert_eq!(
            actual,
            include_str!("../htslib/test/longrefs/longref_multi.expected.sam")
        );
    }
}
