use htslib_rs::{
    bcf_destroy, bcf_get_format_string, bcf_get_format_values, bcf_get_info_values,
    bcf_hdr_add_sample, bcf_hdr_append, bcf_hdr_destroy, bcf_hdr_get_version, bcf_hdr_id2int,
    bcf_hdr_init, bcf_hdr_merge, bcf_hdr_name2id, bcf_hdr_read, bcf_hdr_sync, bcf_hdr_write,
    bcf_init, bcf_read, bcf_remove_filter, bcf_seqname, bcf_sweep_bwd, bcf_sweep_destroy,
    bcf_sweep_fwd, bcf_sweep_hdr, bcf_sweep_init, bcf_translate, bcf_unpack,
    bcf_update_alleles_str, bcf_update_filter, bcf_update_format, bcf_update_info, bcf_write,
    hts_close, hts_open,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn tmp_vcf_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-vcf-{}-{}.vcf",
        std::process::id(),
        label
    ))
}

unsafe fn assert_alleles(rec: *mut htslib_rs::bcf1_t, expected: &[&CStr]) {
    assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int), 0);
    assert_eq!((*rec).n_allele() as usize, expected.len());
    for (i, allele) in expected.iter().enumerate() {
        assert_eq!(CStr::from_ptr(*(*rec).d.allele.add(i)), *allele);
    }
}

unsafe fn assert_sample_names(hdr: *const htslib_rs::bcf_hdr_t, expected: &[&CStr]) {
    assert_eq!(
        (*hdr).n[hts_sys::BCF_DT_SAMPLE as usize] as usize,
        expected.len()
    );
    assert!(!(*hdr).samples.is_null());
    for (i, sample) in expected.iter().enumerate() {
        assert_eq!(CStr::from_ptr(*(*hdr).samples.add(i)), *sample);
    }
}

unsafe fn get_info_i32(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &CStr,
) -> Vec<i32> {
    let mut values = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_info_values(
        hdr,
        rec,
        tag.as_ptr(),
        &mut values,
        &mut nvalues,
        hts_sys::BCF_HT_INT as c_int,
    );
    assert!(ret >= 0, "bcf_get_info_values({tag:?}) returned {ret}");
    assert!(nvalues >= ret);
    let out = std::slice::from_raw_parts(values.cast::<i32>(), ret as usize).to_vec();
    libc::free(values);
    out
}

unsafe fn get_info_f32(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &CStr,
) -> Vec<f32> {
    let mut values = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_info_values(
        hdr,
        rec,
        tag.as_ptr(),
        &mut values,
        &mut nvalues,
        hts_sys::BCF_HT_REAL as c_int,
    );
    assert!(ret >= 0, "bcf_get_info_values({tag:?}) returned {ret}");
    assert!(nvalues >= ret);
    let out = std::slice::from_raw_parts(values.cast::<f32>(), ret as usize).to_vec();
    libc::free(values);
    out
}

unsafe fn get_format_i32(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &CStr,
) -> Vec<i32> {
    let mut values: *mut c_void = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_format_values(
        hdr,
        rec,
        tag.as_ptr(),
        &mut values,
        &mut nvalues,
        hts_sys::BCF_HT_INT as c_int,
    );
    assert!(ret >= 0, "bcf_get_format_values({tag:?}) returned {ret}");
    assert!(nvalues >= ret);
    let out = std::slice::from_raw_parts(values.cast::<i32>(), ret as usize).to_vec();
    libc::free(values);
    out
}

unsafe fn get_format_strings(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &CStr,
) -> Vec<String> {
    let mut values: *mut *mut c_char = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_format_string(hdr, rec, tag.as_ptr(), &mut values, &mut nvalues);
    assert!(ret >= 0, "bcf_get_format_string({tag:?}) returned {ret}");
    assert!(nvalues as u32 >= (*rec).n_sample());
    let out = (0..(*rec).n_sample() as usize)
        .map(|i| {
            CStr::from_ptr(*values.add(i))
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    libc::free((*values).cast());
    libc::free(values.cast());
    out
}

#[test]
fn test_vcf_api_out_preserves_header_samples_and_record_order() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-api.out");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(CStr::from_ptr(bcf_hdr_get_version(hdr)), c"VCFv4.2");
        assert_eq!(bcf_hdr_name2id(hdr, c"20".as_ptr()), 0);
        assert_sample_names(hdr, &[c"NA00001", c"NA00002", c"NA00003"]);
        assert!(bcf_hdr_id2int(hdr, hts_sys::BCF_DT_ID as c_int, c"NEG".as_ptr()) >= 0);
        assert!(bcf_hdr_id2int(hdr, hts_sys::BCF_DT_ID as c_int, c"TS".as_ptr()) >= 0);

        let rec = bcf_init();
        assert!(!rec.is_null());
        let mut seen = Vec::new();
        while bcf_read(fp, hdr, rec) >= 0 {
            assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"20");
            seen.push(((*rec).pos, (*rec).rlen, (*rec).qual.round() as i32));
        }
        assert_eq!(
            seen,
            vec![
                (14_369, 1, 29),
                (14_369, 1, 29),
                (14_369, 1, 29),
                (1_110_695, 1, 67),
                (1_110_695, 1, 67),
                (1_110_695, 1, 67),
            ]
        );

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn test_vcf_api_out_matches_original_get_info_values_checks() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-api.out");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        let mut checked = 0;
        while bcf_read(fp, hdr, rec) >= 0 {
            let af = get_info_f32(hdr, rec, c"AF");
            let neg = get_info_i32(hdr, rec, c"NEG");
            assert_eq!(neg.len(), 1);

            if (*rec).pos == 14_369 {
                assert_eq!(af, vec![0.5]);
                assert_eq!(neg[0], -127);
            } else {
                assert_eq!((*rec).pos, 1_110_695);
                assert_eq!(af.len(), 2);
                assert_eq!(af[0], 0.333);
                let missing_float = hts_sys::bcf_float_missing;
                assert_eq!(af[1].to_bits(), missing_float);
                assert_eq!(neg[0], -128);
            }
            checked += 1;
        }
        assert_eq!(checked, 6);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn test_vcf_api_out_keeps_format_strings_and_missing_integer_vectors() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-api.out");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());
        assert_eq!(bcf_read(fp, hdr, rec), 0);

        assert_alleles(rec, &[c"G", c"A"]);
        assert_eq!(
            get_format_strings(hdr, rec, c"TS"),
            vec![
                "String1".to_string(),
                "SomeOtherString2".to_string(),
                "YetAnotherString3".to_string(),
            ]
        );
        assert_eq!(get_format_i32(hdr, rec, c"GQ"), vec![48, 48, 43]);
        assert_eq!(get_format_i32(hdr, rec, c"DP"), vec![1, 8, 5]);
        assert_eq!(
            get_format_i32(hdr, rec, c"HQ"),
            vec![
                51,
                51,
                51,
                51,
                hts_sys::bcf_int32_missing,
                hts_sys::bcf_int32_missing,
            ]
        );

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn test_vcf_api_out_keeps_haploid_and_missing_genotype_sentinel_layout() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-api.out");
        let fp = hts_open(vcf.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        for _ in 0..3 {
            assert_eq!(bcf_read(fp, hdr, rec), 0);
        }
        assert_eq!(bcf_read(fp, hdr, rec), 0);
        assert_eq!(CStr::from_ptr(bcf_seqname(hdr, rec)), c"20");
        assert_eq!((*rec).pos, 1_110_695);
        assert_alleles(rec, &[c"A", c"G", c"T"]);
        // NOTE (2026-05-29): htslib v1.23 made haploid genotypes implicitly
        // phased even for VCF < 4.4 (test fixture is VCFv4.2). v1.19.1 returned
        // [6, EOV, 4, EOV, 0, 0] (no phase bit); v1.23 sets the phase bit on
        // the haploid allele → [7, EOV, 5, EOV, 0, 0]. Our native parser
        // faithfully matches v1.23 — confirmed by `vcf_parse_native_matches_hts_sys`
        // (native and the linked C produce identical byte output).
        assert_eq!(
            get_format_i32(hdr, rec, c"GT"),
            vec![
                7,
                hts_sys::bcf_int32_vector_end,
                5,
                hts_sys::bcf_int32_vector_end,
                0,
                0,
            ]
        );

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(fp), 0);
    }
}

#[test]
fn test_vcf_sweep_matches_original_fixture_output() {
    let path = tmp_vcf_path("sweep");
    let input = concat!(
        "##fileformat=VCFv4.2\n",
        "##contig=<ID=20>\n",
        "##INFO=<ID=NS,Number=1,Type=Integer,Description=\"Number of Samples With Data\">\n",
        "##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Total Depth\">\n",
        "##INFO=<ID=AF,Number=A,Type=Float,Description=\"Allele Frequency\">\n",
        "##INFO=<ID=DB,Number=0,Type=Flag,Description=\"dbSNP membership\">\n",
        "##INFO=<ID=H2,Number=0,Type=Flag,Description=\"HapMap2 membership\">\n",
        "##INFO=<ID=AA,Number=1,Type=String,Description=\"Ancestral Allele\">\n",
        "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n",
        "##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"Genotype Quality\">\n",
        "##FORMAT=<ID=DP,Number=1,Type=Integer,Description=\"Read Depth\">\n",
        "##FORMAT=<ID=HQ,Number=2,Type=Integer,Description=\"Haplotype Quality\">\n",
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tNA00001\tNA00002\tNA00003\n",
        "20\t14370\trs6054257\tG\tA\t29\tPASS\tNS=3;DP=14;AF=0.5;DB;H2\tGT:GQ:DP:HQ\t0|0:48:1:51,51\t1|0:48:8:51,51\t1/1:43:5:.,.\n",
        "20\t1110696\t.\tA\tG,T\t67\t.\tNS=2;DP=10;AF=0.333,.;AA=T;DB\tGT\t2\t1\t./.\n",
    );
    std::fs::write(&path, input).unwrap();

    unsafe {
        let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let sw = bcf_sweep_init(c_path.as_ptr());
        assert!(!sw.is_null());
        let hdr = bcf_sweep_hdr(sw);
        assert!(!hdr.is_null());

        let mut out = String::new();
        let mut chksum = 0;
        loop {
            let rec = bcf_sweep_fwd(sw);
            if rec.is_null() {
                break;
            }
            chksum += (*rec).pos + 1;
        }
        out.push_str(&format!("fwd position chksum: {chksum}\n"));

        chksum = 0;
        loop {
            let rec = bcf_sweep_bwd(sw);
            if rec.is_null() {
                break;
            }
            chksum += (*rec).pos + 1;
        }
        out.push_str(&format!("bwd position chksum: {chksum}\n"));

        for (label, forward) in [("fwd PL chksum", true), ("bwd PL chksum", false)] {
            chksum = 0;
            loop {
                let rec = if forward {
                    bcf_sweep_fwd(sw)
                } else {
                    bcf_sweep_bwd(sw)
                };
                if rec.is_null() {
                    break;
                }
                let mut pls: *mut c_void = std::ptr::null_mut();
                let mut m_pls = 0;
                let n_pls = bcf_get_format_values(
                    hdr,
                    rec,
                    c"PL".as_ptr(),
                    &mut pls,
                    &mut m_pls,
                    hts_sys::BCF_HT_INT as c_int,
                );
                if n_pls <= 0 {
                    continue;
                }
                let nvals = n_pls / (*hdr).n[hts_sys::BCF_DT_SAMPLE as usize];
                let values = std::slice::from_raw_parts(pls.cast::<i32>(), n_pls as usize);
                for sample in 0..(*hdr).n[hts_sys::BCF_DT_SAMPLE as usize] as usize {
                    for val in &values[sample * nvals as usize..][..nvals as usize] {
                        if *val == hts_sys::bcf_int32_vector_end {
                            break;
                        }
                        if *val != hts_sys::bcf_int32_missing {
                            chksum += *val as i64;
                        }
                    }
                }
                libc::free(pls);
            }
            out.push_str(&format!("{label}: {chksum}\n"));
        }

        bcf_sweep_destroy(sw);
        assert_eq!(
            out,
            std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("htslib/test/test-vcf-sweep.out")
            )
            .unwrap()
        );
    }

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_bcf_translate_matches_original_fixture_output() {
    let path = tmp_vcf_path("translate");
    let _ = std::fs::remove_file(&path);

    unsafe {
        let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let fp = hts_open(c_path.as_ptr(), c"w".as_ptr());
        assert!(!fp.is_null());

        let hdr1 = bcf_hdr_init(c"w".as_ptr());
        let mut hdr2 = bcf_hdr_init(c"w".as_ptr());
        assert!(!hdr1.is_null());
        assert!(!hdr2.is_null());

        for line in [
            c"##contig=<ID=1>",
            c"##contig=<ID=2>",
            c"##FILTER=<ID=FLT1,Description=\"Filter 1\">",
            c"##FILTER=<ID=FLT2,Description=\"Filter 2\">",
            c"##FILTER=<ID=FLT3,Description=\"Filter 3\">",
            c"##INFO=<ID=INF1,Number=.,Type=Integer,Description=\"Info 1\">",
            c"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">",
            c"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">",
            c"##FORMAT=<ID=FMT1,Number=.,Type=Integer,Description=\"FMT 1\">",
            c"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">",
            c"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">",
        ] {
            assert_eq!(bcf_hdr_append(hdr1, line.as_ptr()), 0);
        }
        for line in [
            c"##contig=<ID=2>",
            c"##contig=<ID=1>",
            c"##FILTER=<ID=FLT4,Description=\"Filter 4\">",
            c"##FILTER=<ID=FLT3,Description=\"Filter 3\">",
            c"##FILTER=<ID=FLT2,Description=\"Filter 2\">",
            c"##INFO=<ID=INF4,Number=.,Type=Integer,Description=\"Info 4\">",
            c"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">",
            c"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">",
            c"##FORMAT=<ID=FMT4,Number=.,Type=Integer,Description=\"FMT 4\">",
            c"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">",
            c"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">",
        ] {
            assert_eq!(bcf_hdr_append(hdr2, line.as_ptr()), 0);
        }
        assert_eq!(bcf_hdr_add_sample(hdr1, c"SMPL1".as_ptr()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr1, c"SMPL2".as_ptr()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr2, c"SMPL1".as_ptr()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr2, c"SMPL2".as_ptr()), 0);
        assert_eq!(bcf_hdr_sync(hdr1), 0);
        assert_eq!(bcf_hdr_sync(hdr2), 0);

        hdr2 = bcf_hdr_merge(hdr2, hdr1);
        assert!(!hdr2.is_null());
        assert_eq!(bcf_hdr_sync(hdr2), 0);
        assert_eq!(bcf_hdr_write(fp, hdr2), 0);

        let rec = bcf_init();
        assert!(!rec.is_null());
        (*rec).rid = bcf_hdr_name2id(hdr1, c"1".as_ptr());
        (*rec).pos = 0;
        assert_eq!(bcf_update_alleles_str(hdr1, rec, c"G,A".as_ptr()), 0);

        let mut tmpi = [
            bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT1".as_ptr()),
            bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT2".as_ptr()),
            bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT3".as_ptr()),
        ];
        assert_eq!(bcf_update_filter(hdr1, rec, tmpi.as_mut_ptr(), 3), 0);
        for (tag, value) in [(c"INF1", 1), (c"INF2", 2), (c"INF3", 3)] {
            tmpi[0] = value;
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    tag.as_ptr(),
                    tmpi.as_ptr().cast(),
                    1,
                    hts_sys::BCF_HT_INT as c_int
                ),
                0
            );
        }
        for (tag, value) in [(c"FMT1", 1), (c"FMT2", 2), (c"FMT3", 3)] {
            tmpi[0] = value;
            tmpi[1] = value;
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    tag.as_ptr(),
                    tmpi.as_ptr().cast(),
                    2,
                    hts_sys::BCF_HT_INT as c_int
                ),
                0
            );
        }

        assert_eq!(
            bcf_remove_filter(
                hdr1,
                rec,
                bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as c_int, c"FLT2".as_ptr()),
                0
            ),
            0
        );
        assert_eq!(
            bcf_update_info(
                hdr1,
                rec,
                c"INF2".as_ptr(),
                std::ptr::null(),
                0,
                hts_sys::BCF_HT_INT as c_int
            ),
            0
        );
        assert_eq!(
            bcf_update_format(
                hdr1,
                rec,
                c"FMT2".as_ptr(),
                std::ptr::null(),
                0,
                hts_sys::BCF_HT_INT as c_int
            ),
            0
        );

        assert_eq!(bcf_translate(hdr2, hdr1, rec), 0);
        assert_eq!(bcf_write(fp, hdr2, rec), 0);

        bcf_destroy(rec);
        bcf_hdr_destroy(hdr1);
        bcf_hdr_destroy(hdr2);
        assert_eq!(hts_close(fp), 0);
    }

    let actual = std::fs::read_to_string(&path).unwrap();
    let expected = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/test-bcf-translate.out"),
    )
    .unwrap();
    assert_eq!(actual, expected);
    let _ = std::fs::remove_file(path);
}
