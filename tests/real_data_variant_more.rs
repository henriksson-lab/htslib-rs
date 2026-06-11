use htslib_rs::{
    bcf_destroy, bcf_get_format_values, bcf_get_info_values, bcf_get_variant_type,
    bcf_get_variant_types, bcf_has_variant_type, bcf_has_variant_types, bcf_hdr_destroy,
    bcf_hdr_id2name, bcf_hdr_name2id, bcf_hdr_read, bcf_hdr_seqnames, bcf_hdr_write,
    bcf_index_build, bcf_index_load2, bcf_init, bcf_read, bcf_readrec, bcf_seqname,
    bcf_sr_add_reader, bcf_sr_destroy, bcf_sr_get_header, bcf_sr_get_line, bcf_sr_has_line,
    bcf_sr_init, bcf_sr_next_line, bcf_sr_set_opt_allow_no_idx, bcf_sr_set_opt_pair_logic,
    bcf_sr_set_regions, bcf_sr_set_targets, bcf_unpack, bcf_write, hts_close, hts_get_bgzfp,
    hts_idx_destroy, hts_itr_destroy, hts_itr_next, hts_itr_query, hts_open, hts_pos_t,
    vcf_hdr_read, vcf_read, BGZF, VCF_INS,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};
use std::process::Command;

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
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

fn bcf_gt_unphased(idx: c_int) -> c_int {
    (idx + 1) << 1
}

unsafe fn count_indexed_variant_records(
    path: &str,
    index_path: &str,
    chrom: &CStr,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> usize {
    let path = c_fixture(path);
    let index_path = c_fixture(index_path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

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

fn tmp_fixture_path(label: &str, ext: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-bcf-sr-{}-{}.{}",
        std::process::id(),
        label,
        ext
    ))
}

unsafe fn make_indexed_bcf_from_vcf(vcf_path: &str, label: &str) -> CString {
    let bcf_path = tmp_fixture_path(label, "bcf");
    let c_bcf_path = CString::new(bcf_path.to_string_lossy().as_bytes()).unwrap();
    let _ = std::fs::remove_file(&bcf_path);
    let _ = std::fs::remove_file(bcf_path.with_extension("bcf.csi"));

    let in_path = c_fixture(vcf_path);
    let in_fp = hts_open(in_path.as_ptr(), c"r".as_ptr());
    assert!(!in_fp.is_null(), "failed to open {}", vcf_path);
    let hdr = vcf_hdr_read(in_fp);
    assert!(!hdr.is_null());

    let out_fp = hts_open(c_bcf_path.as_ptr(), c"wb".as_ptr());
    assert!(!out_fp.is_null(), "failed to create {}", bcf_path.display());
    assert_eq!(bcf_hdr_write(out_fp, hdr), 0);

    let rec = bcf_init();
    assert!(!rec.is_null());
    while vcf_read(in_fp, hdr, rec) >= 0 {
        assert_eq!(bcf_write(out_fp, hdr, rec), 0);
    }

    bcf_destroy(rec);
    assert_eq!(hts_close(out_fp), 0);
    bcf_hdr_destroy(hdr);
    assert_eq!(hts_close(in_fp), 0);
    assert_eq!(bcf_index_build(c_bcf_path.as_ptr(), 14), 0);

    c_bcf_path
}

fn remove_indexed_bcf(path: &CStr) {
    let path = std::path::Path::new(path.to_str().unwrap());
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("bcf.csi"));
}

unsafe fn synced_reader_vcf_output(
    path: &CStr,
    region_or_target: &CStr,
    use_targets: bool,
    label: &str,
) -> String {
    let out_path = tmp_fixture_path(label, "vcf");
    let c_out_path = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
    let _ = std::fs::remove_file(&out_path);

    let sr = bcf_sr_init();
    assert!(!sr.is_null());
    let set_ret = if use_targets {
        bcf_sr_set_targets(sr, region_or_target.as_ptr(), 0, 0)
    } else {
        bcf_sr_set_regions(sr, region_or_target.as_ptr(), 0)
    };
    assert_eq!(set_ret, 0, "failed to set {:?}", region_or_target);
    assert_eq!(bcf_sr_add_reader(sr, path.as_ptr()), 1);

    let out_fp = hts_open(c_out_path.as_ptr(), c"w".as_ptr());
    assert!(!out_fp.is_null());
    let hdr = bcf_sr_get_header(sr, 0);
    assert!(!hdr.is_null());
    assert_eq!(bcf_hdr_write(out_fp, hdr), 0);
    while bcf_sr_next_line(sr) > 0 {
        let rec = bcf_sr_get_line(sr, 0);
        assert!(!rec.is_null());
        assert_eq!(bcf_write(out_fp, hdr, rec), 0);
    }
    assert_eq!((*sr).errnum, 0);
    assert_eq!(hts_close(out_fp), 0);
    bcf_sr_destroy(sr);

    let out = std::fs::read_to_string(&out_path).unwrap();
    let _ = std::fs::remove_file(&out_path);
    out
}

unsafe fn synced_reader_summary_no_index(paths: &[&str]) -> String {
    let sr = bcf_sr_init();
    assert!(!sr.is_null());
    assert_eq!(bcf_sr_set_opt_allow_no_idx(sr), 0);
    assert_eq!(
        bcf_sr_set_opt_pair_logic(sr, hts_sys::BCF_SR_PAIR_ANY as c_int),
        0
    );
    for path in paths {
        let c_path = c_fixture(path);
        assert_eq!(
            bcf_sr_add_reader(sr, c_path.as_ptr()),
            1,
            "failed on {path}"
        );
    }

    let mut out = String::new();
    while bcf_sr_next_line(sr) > 0 {
        for i in 0..(*sr).nreaders {
            if bcf_sr_has_line(sr, i) != 0 {
                let hdr = bcf_sr_get_header(sr, i);
                let rec = bcf_sr_get_line(sr, i);
                assert!(!hdr.is_null());
                assert!(!rec.is_null());
                out.push_str(CStr::from_ptr(bcf_seqname(hdr, rec)).to_str().unwrap());
                out.push(':');
                out.push_str(&((*rec).pos + 1).to_string());
                break;
            }
        }
        for i in 0..(*sr).nreaders {
            out.push('\t');
            if bcf_sr_has_line(sr, i) == 0 {
                out.push('-');
                continue;
            }
            let rec = bcf_sr_get_line(sr, i);
            assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int), 0);
            let n_allele = (*rec).n_allele();
            if n_allele > 1 {
                let d = &(*rec).d;
                out.push_str(std::str::from_utf8(&d.allele[1]).unwrap());
                for j in 2..n_allele {
                    out.push(',');
                    out.push_str(std::str::from_utf8(&d.allele[j as usize]).unwrap());
                }
            } else {
                out.push('.');
            }
        }
        out.push('\n');
    }
    assert_eq!((*sr).errnum, 0);
    bcf_sr_destroy(sr);
    out
}

unsafe fn synced_reader_no_index_status(paths: &[&str]) -> (c_int, c_int, u32) {
    let sr = bcf_sr_init();
    assert!(!sr.is_null());
    assert_eq!(bcf_sr_set_opt_allow_no_idx(sr), 0);
    assert_eq!(
        bcf_sr_set_opt_pair_logic(sr, hts_sys::BCF_SR_PAIR_ANY as c_int),
        0
    );
    for path in paths {
        let c_path = c_fixture(path);
        let add_ret = bcf_sr_add_reader(sr, c_path.as_ptr());
        if add_ret != 1 {
            let errnum = (*sr).errnum;
            bcf_sr_destroy(sr);
            return (add_ret, 0, errnum);
        }
    }

    let mut ret;
    loop {
        ret = bcf_sr_next_line(sr);
        if ret <= 0 {
            break;
        }
    }
    let errnum = (*sr).errnum;
    bcf_sr_destroy(sr);
    (1, ret, errnum)
}

#[test]
fn scans_additional_real_vcf_text_fixtures_with_exact_counts() {
    unsafe {
        assert_eq!(
            count_variant_records("htslib/test/formatmissing-out.vcf"),
            1
        );
        assert_eq!(count_variant_records("htslib/test/vcf_meta_meta.vcf"), 1);
        assert_eq!(count_variant_records("htslib/test/modhdr.expected.vcf"), 0);
        assert_eq!(
            count_variant_records("htslib/test/bcf-sr/merge.noidx.rec_order.vcf"),
            4
        );
        assert_eq!(
            count_variant_records("htslib/test/bcf-sr/merge.noidx.hdr_order.vcf"),
            4
        );
        assert_eq!(count_variant_records("htslib/test/noroundtrip-out.vcf"), 5);
    }
}

#[test]
fn reads_real_bcf_fixture_record_metadata() {
    unsafe {
        let bcf = c_fixture("htslib/test/tabix/vcf_file.bcf");
        let fp = hts_open(bcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(bcf_hdr_name2id(hdr, c"1".as_ptr()), 0);
        assert_eq!(bcf_hdr_name2id(hdr, c"4".as_ptr()), 3);

        let mut nseqs = 0;
        let seqs = bcf_hdr_seqnames(hdr, &mut nseqs);
        assert!(!seqs.is_null());
        assert_eq!(nseqs, 4);
        assert_eq!(CStr::from_ptr(*seqs), c"1");
        assert_eq!(CStr::from_ptr(*seqs.add(3)), c"4");
        libc::free(seqs.cast());

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert!(bcf_read(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 3_000_149);
        assert_eq!((*rec).rlen, 1);

        let mut ac = std::ptr::null_mut();
        let mut nac = 0;
        assert_eq!(
            bcf_get_info_values(
                hdr,
                rec,
                c"AC".as_ptr(),
                &mut ac,
                &mut nac,
                hts_sys::BCF_HT_INT as i32,
            ),
            1
        );
        assert!(nac >= 1);
        assert_eq!(*ac.cast::<i32>(), 2);
        libc::free(ac);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }

    unsafe {
        assert_eq!(count_variant_records("htslib/test/tabix/vcf_file.bcf"), 15);
    }
}

#[test]
fn index_vcf_first_record_decodes_real_info_and_format_arrays() {
    unsafe {
        let vcf = c_fixture("htslib/test/index.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());
        assert_eq!(bcf_read(fp, hdr, rec), 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 9_999_918);
        assert_eq!((*rec).rlen, 1);
        assert_eq!((*rec).n_sample(), 1);
        assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int), 0);
        let d = &(*rec).d;
        assert_eq!(d.allele[0].as_slice(), b"G");
        assert_eq!(d.allele[1].as_slice(), b"<*>");

        let mut dp = std::ptr::null_mut();
        let mut ndp = 0;
        assert_eq!(
            bcf_get_info_values(
                hdr,
                rec,
                c"DP".as_ptr(),
                &mut dp,
                &mut ndp,
                hts_sys::BCF_HT_INT as c_int,
            ),
            1
        );
        assert!(ndp >= 1);
        assert_eq!(*dp.cast::<i32>(), 1);
        libc::free(dp);

        let mut i16 = std::ptr::null_mut();
        let mut ni16 = 0;
        assert_eq!(
            bcf_get_info_values(
                hdr,
                rec,
                c"I16".as_ptr(),
                &mut i16,
                &mut ni16,
                hts_sys::BCF_HT_REAL as c_int,
            ),
            16
        );
        assert!(ni16 >= 16);
        let i16 = std::slice::from_raw_parts(i16.cast::<f32>(), 16);
        assert_eq!(
            i16.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            [
                1.0f32, 0.0, 0.0, 0.0, 26.0, 676.0, 0.0, 0.0, 60.0, 3600.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0,
            ]
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
        );
        libc::free(i16.as_ptr().cast_mut().cast());

        let mut qs = std::ptr::null_mut();
        let mut nqs = 0;
        assert_eq!(
            bcf_get_info_values(
                hdr,
                rec,
                c"QS".as_ptr(),
                &mut qs,
                &mut nqs,
                hts_sys::BCF_HT_REAL as c_int,
            ),
            2
        );
        assert!(nqs >= 2);
        assert_eq!(
            std::slice::from_raw_parts(qs.cast::<f32>(), 2)
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            [1.0f32, 0.0]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        libc::free(qs);

        let mut pl = std::ptr::null_mut();
        let mut npl = 0;
        assert_eq!(
            bcf_get_format_values(
                hdr,
                rec,
                c"PL".as_ptr(),
                &mut pl,
                &mut npl,
                hts_sys::BCF_HT_INT as c_int,
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
fn tabix_vcf_fixture_variant_type_matrix_matches_htslib_classification() {
    unsafe {
        let vcf = c_fixture("htslib/test/tabix/vcf_file.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        let mut seen = Vec::new();
        while bcf_read(fp, hdr, rec) >= 0 {
            let pos = (*rec).pos + 1;
            if matches!(
                pos,
                3_000_150 | 3_062_915 | 3_106_154 | 3_177_144 | 3_199_812
            ) {
                assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int), 0);
                let d = &(*rec).d;
                seen.push((
                    pos,
                    String::from_utf8_lossy(&d.allele[0]).into_owned(),
                    (1..(*rec).n_allele())
                        .map(|i| bcf_get_variant_type(rec, i as c_int))
                        .collect::<Vec<_>>(),
                    bcf_get_variant_types(rec),
                ));
            }
        }

        assert_eq!(
            seen,
            vec![
                (
                    3_000_150,
                    "C".to_string(),
                    vec![hts_sys::VCF_SNP as c_int],
                    hts_sys::VCF_SNP as c_int
                ),
                (
                    3_062_915,
                    "GTTT".to_string(),
                    vec![hts_sys::VCF_INDEL as c_int],
                    hts_sys::VCF_INDEL as c_int
                ),
                (
                    3_062_915,
                    "G".to_string(),
                    vec![hts_sys::VCF_SNP as c_int, hts_sys::VCF_SNP as c_int],
                    hts_sys::VCF_SNP as c_int
                ),
                (
                    3_106_154,
                    "CAAA".to_string(),
                    vec![hts_sys::VCF_INDEL as c_int],
                    hts_sys::VCF_INDEL as c_int
                ),
                (
                    3_106_154,
                    "C".to_string(),
                    vec![hts_sys::VCF_INDEL as c_int],
                    hts_sys::VCF_INDEL as c_int
                ),
                (
                    3_177_144,
                    "G".to_string(),
                    vec![hts_sys::VCF_SNP as c_int],
                    hts_sys::VCF_SNP as c_int
                ),
                (3_177_144, "G".to_string(), vec![], 0),
                (
                    3_199_812,
                    "G".to_string(),
                    vec![hts_sys::VCF_INDEL as c_int, hts_sys::VCF_INDEL as c_int],
                    hts_sys::VCF_INDEL as c_int
                ),
            ]
        );

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn tabix_vcf_fixture_decodes_high_numbered_gt_alleles() {
    unsafe {
        let vcf = c_fixture("htslib/test/tabix/vcf_file.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        while bcf_read(fp, hdr, rec) >= 0 && (*rec).pos + 1 != 3_258_501 {}
        assert_eq!((*rec).pos + 1, 3_258_501);
        assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int), 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"4");
        let d = &(*rec).d;
        assert_eq!(d.allele[0].as_slice(), b"C");
        assert_eq!((*rec).n_allele(), 306);
        assert_eq!(
            bcf_has_variant_types(rec, hts_sys::VCF_SNP, 1),
            hts_sys::VCF_SNP as c_int
        );
        assert_eq!(
            bcf_has_variant_types(rec, hts_sys::VCF_INDEL | VCF_INS, 1),
            (hts_sys::VCF_INDEL | VCF_INS) as c_int
        );
        assert_eq!(bcf_has_variant_type(rec, 300, VCF_INS), VCF_INS as c_int);

        let mut gt = std::ptr::null_mut();
        let mut ngt = 0;
        assert_eq!(
            bcf_get_format_values(
                hdr,
                rec,
                c"GT".as_ptr(),
                &mut gt,
                &mut ngt,
                hts_sys::BCF_HT_INT as c_int,
            ),
            4
        );
        assert!(ngt >= 4);
        assert_eq!(
            std::slice::from_raw_parts(gt.cast::<i32>(), 4),
            &[
                bcf_gt_unphased(0),
                bcf_gt_unphased(300),
                bcf_gt_unphased(240),
                bcf_gt_unphased(260),
            ]
        );
        libc::free(gt);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn reads_real_vcf_header_with_colon_and_dash_contig_names() {
    unsafe {
        let vcf = c_fixture("htslib/test/bcf-sr/weird-chr-names.vcf");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = vcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(bcf_hdr_name2id(hdr, c"1".as_ptr()), 0);
        assert_eq!(bcf_hdr_name2id(hdr, c"1:1".as_ptr()), 1);
        assert_eq!(bcf_hdr_name2id(hdr, c"1:1-1".as_ptr()), 2);
        assert_eq!(CStr::from_ptr(bcf_hdr_id2name(hdr, 2)), c"1:1-1");

        let rec = bcf_init();
        assert!(!rec.is_null());
        assert!(bcf_read(fp, hdr, rec) >= 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"1");
        assert_eq!((*rec).pos, 0);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn synced_bcf_reader_disambiguates_braced_weird_chromosome_regions() {
    unsafe {
        let bcf = make_indexed_bcf_from_vcf("htslib/test/bcf-sr/weird-chr-names.vcf", "weird");
        let cases = [
            (c"1", "htslib/test/bcf-sr/weird-chr-names.1.out"),
            (c"1:1-2", "htslib/test/bcf-sr/weird-chr-names.1.out"),
            (c"1:1,1:2", "htslib/test/bcf-sr/weird-chr-names.1.out"),
            (c"1:1-1", "htslib/test/bcf-sr/weird-chr-names.2.out"),
            (c"{1:1}", "htslib/test/bcf-sr/weird-chr-names.3.out"),
            (c"{1:1}:1-2", "htslib/test/bcf-sr/weird-chr-names.3.out"),
            (
                c"{1:1}:1,{1:1}:2",
                "htslib/test/bcf-sr/weird-chr-names.3.out",
            ),
            (c"{1:1}:1-1", "htslib/test/bcf-sr/weird-chr-names.4.out"),
            (c"{1:1-1}", "htslib/test/bcf-sr/weird-chr-names.5.out"),
            (c"{1:1-1}:1-2", "htslib/test/bcf-sr/weird-chr-names.5.out"),
            (
                c"{1:1-1}:1,{1:1-1}:2",
                "htslib/test/bcf-sr/weird-chr-names.5.out",
            ),
            (c"{1:1-1}:1-1", "htslib/test/bcf-sr/weird-chr-names.6.out"),
        ];

        for (idx, (region, expected_path)) in cases.iter().enumerate() {
            let actual = synced_reader_vcf_output(
                bcf.as_c_str(),
                region,
                false,
                &format!("weird-region-{idx}"),
            );
            let expected = std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(expected_path),
            )
            .unwrap();
            assert_eq!(actual, expected, "region {:?}", region);
        }

        let sr = bcf_sr_init();
        assert!(!sr.is_null());
        assert_ne!(bcf_sr_set_regions(sr, c"{1:1-1}-2".as_ptr(), 0), 0);
        bcf_sr_destroy(sr);
        remove_indexed_bcf(bcf.as_c_str());
    }
}

#[test]
fn synced_bcf_reader_disambiguates_braced_weird_chromosome_targets() {
    unsafe {
        let bcf =
            make_indexed_bcf_from_vcf("htslib/test/bcf-sr/weird-chr-names.vcf", "weird-target");
        let cases = [
            (c"1", "htslib/test/bcf-sr/weird-chr-names.1.out"),
            (c"1:1-2", "htslib/test/bcf-sr/weird-chr-names.1.out"),
            (c"1:1,1:2", "htslib/test/bcf-sr/weird-chr-names.1.out"),
            (c"1:1-1", "htslib/test/bcf-sr/weird-chr-names.2.out"),
            (c"{1:1}", "htslib/test/bcf-sr/weird-chr-names.3.out"),
            (c"{1:1}:1-2", "htslib/test/bcf-sr/weird-chr-names.3.out"),
            (
                c"{1:1}:1,{1:1}:2",
                "htslib/test/bcf-sr/weird-chr-names.3.out",
            ),
            (c"{1:1}:1-1", "htslib/test/bcf-sr/weird-chr-names.4.out"),
            (c"{1:1-1}", "htslib/test/bcf-sr/weird-chr-names.5.out"),
            (c"{1:1-1}:1-2", "htslib/test/bcf-sr/weird-chr-names.5.out"),
            (
                c"{1:1-1}:1,{1:1-1}:2",
                "htslib/test/bcf-sr/weird-chr-names.5.out",
            ),
            (c"{1:1-1}:1-1", "htslib/test/bcf-sr/weird-chr-names.6.out"),
        ];

        for (idx, (target, expected_path)) in cases.iter().enumerate() {
            let actual = synced_reader_vcf_output(
                bcf.as_c_str(),
                target,
                true,
                &format!("weird-target-{idx}"),
            );
            let expected = std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(expected_path),
            )
            .unwrap();
            assert_eq!(actual, expected, "target {:?}", target);
        }

        let sr = bcf_sr_init();
        assert!(!sr.is_null());
        assert_ne!(bcf_sr_set_targets(sr, c"{1:1-1}-2".as_ptr(), 0, 0), 0);
        bcf_sr_destroy(sr);
        remove_indexed_bcf(bcf.as_c_str());
    }
}

#[test]
fn synced_bcf_reader_no_index_merge_fixture_matches_expected_summary() {
    unsafe {
        let actual = synced_reader_summary_no_index(&[
            "htslib/test/bcf-sr/merge.noidx.a.vcf",
            "htslib/test/bcf-sr/merge.noidx.b.vcf",
            "htslib/test/bcf-sr/merge.noidx.c.vcf",
        ]);
        let expected = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("htslib/test/bcf-sr/merge.noidx.abc.expected.out"),
        )
        .unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn synced_bcf_reader_no_index_merge_rejects_bad_input_ordering() {
    const CHILD_ENV: &str = "HTSLIB_MINI_RS_NO_INDEX_BAD_MERGE_CASE";

    if let Ok(case) = std::env::var(CHILD_ENV) {
        let paths = match case.as_str() {
            "hdr_order" => [
                "htslib/test/bcf-sr/merge.noidx.a.vcf",
                "htslib/test/bcf-sr/merge.noidx.hdr_order.vcf",
            ],
            "rec_order_second" => [
                "htslib/test/bcf-sr/merge.noidx.a.vcf",
                "htslib/test/bcf-sr/merge.noidx.rec_order.vcf",
            ],
            "rec_order_first" => [
                "htslib/test/bcf-sr/merge.noidx.rec_order.vcf",
                "htslib/test/bcf-sr/merge.noidx.a.vcf",
            ],
            _ => panic!("unknown bad no-index merge case {case}"),
        };
        unsafe {
            let (add_ret, next_ret, errnum) = synced_reader_no_index_status(&paths);
            std::process::exit(if add_ret != 1 || next_ret != 0 || errnum != 0 {
                1
            } else {
                0
            });
        }
    }

    for case in ["hdr_order", "rec_order_second", "rec_order_first"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("synced_bcf_reader_no_index_merge_rejects_bad_input_ordering")
            .env(CHILD_ENV, case)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "no-index merge case {case} unexpectedly succeeded"
        );
    }
}

#[test]
fn queries_real_compressed_vcf_csi_fixture_with_exact_empty_count() {
    unsafe {
        assert_eq!(
            count_indexed_variant_records(
                "htslib/test/modhdr.vcf.gz",
                "htslib/test/modhdr.vcf.gz.csi",
                c"chr22",
                0,
                51_304_566,
            ),
            0
        );
    }
}
