use htslib_rs::{
    bcf_destroy, bcf_get_format_string, bcf_get_format_values, bcf_get_info_values,
    bcf_hdr_add_sample, bcf_hdr_append, bcf_hdr_destroy, bcf_hdr_get_version, bcf_hdr_id2int,
    bcf_hdr_init, bcf_hdr_merge, bcf_hdr_name2id, bcf_hdr_read, bcf_hdr_sync, bcf_hdr_write,
    bcf_init, bcf_read, bcf_remove_filter, bcf_seqname, bcf_sweep_bwd, bcf_sweep_destroy,
    bcf_sweep_fwd, bcf_sweep_hdr, bcf_sweep_init, bcf_translate, bcf_unpack,
    bcf_update_alleles_str, bcf_update_filter, bcf_update_format, bcf_update_info, bcf_write,
    hts_close, hts_open,
};
fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn tmp_vcf_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "htslib_rs-vcf-{}-{}.vcf",
        std::process::id(),
        label
    ))
}

unsafe fn assert_alleles(rec: *mut htslib_rs::bcf1_t, expected: &[&[u8]]) {
    assert_eq!(bcf_unpack(rec, hts_sys::BCF_UN_STR as i32), 0);
    assert_eq!((*rec).n_allele() as usize, expected.len());
    let d = &(*rec).d;
    for (i, allele) in expected.iter().enumerate() {
        assert_eq!(d.allele[i].as_slice(), *allele);
    }
}

unsafe fn assert_sample_names(hdr: *const htslib_rs::bcf_hdr_t, expected: &[&[u8]]) {
    assert_eq!(
        (*hdr).n[hts_sys::BCF_DT_SAMPLE as usize] as usize,
        expected.len()
    );
    assert!(!(*hdr).samples.is_null());
    for (i, sample) in expected.iter().enumerate() {
        let p = *(*hdr).samples.add(i);
        assert_eq!(std::ffi::CStr::from_ptr(p.cast()).to_bytes(), *sample);
    }
}

unsafe fn get_info_i32(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &[u8],
) -> Vec<i32> {
    let mut values = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_info_values(
        hdr,
        rec,
        tag.as_ptr().cast(),
        &mut values,
        &mut nvalues,
        hts_sys::BCF_HT_INT as i32,
    );
    assert!(ret >= 0, "bcf_get_info_values({tag:?}) returned {ret}");
    assert!(nvalues >= ret);
    std::slice::from_raw_parts(values.cast::<i32>(), ret as usize).to_vec()
}

unsafe fn get_info_f32(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &[u8],
) -> Vec<f32> {
    let mut values = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_info_values(
        hdr,
        rec,
        tag.as_ptr().cast(),
        &mut values,
        &mut nvalues,
        hts_sys::BCF_HT_REAL as i32,
    );
    assert!(ret >= 0, "bcf_get_info_values({tag:?}) returned {ret}");
    assert!(nvalues >= ret);
    std::slice::from_raw_parts(values.cast::<f32>(), ret as usize).to_vec()
}

unsafe fn get_format_i32(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &[u8],
) -> Vec<i32> {
    let mut values: *mut () = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_format_values(
        hdr,
        rec,
        tag.as_ptr().cast(),
        &mut values,
        &mut nvalues,
        hts_sys::BCF_HT_INT as i32,
    );
    assert!(ret >= 0, "bcf_get_format_values({tag:?}) returned {ret}");
    assert!(nvalues >= ret);
    std::slice::from_raw_parts(values.cast::<i32>(), ret as usize).to_vec()
}

unsafe fn get_format_strings(
    hdr: *const htslib_rs::bcf_hdr_t,
    rec: *mut htslib_rs::bcf1_t,
    tag: &[u8],
) -> Vec<Vec<u8>> {
    let mut values: *mut *mut u8 = std::ptr::null_mut();
    let mut nvalues = 0;
    let ret = bcf_get_format_string(hdr, rec, tag.as_ptr().cast(), &mut values, &mut nvalues);
    assert!(ret >= 0, "bcf_get_format_string({tag:?}) returned {ret}");
    assert!(nvalues as u32 >= (*rec).n_sample());
    (0..(*rec).n_sample() as usize)
        .map(|i| std::ffi::CStr::from_ptr((*values.add(i)).cast()).to_bytes().to_vec())
        .collect()
}

#[test]
fn test_vcf_api_out_preserves_header_samples_and_record_order() {
    unsafe {
        let vcf = c_fixture("htslib/test/test-vcf-api.out");
        let fp = hts_open(vcf.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        assert_eq!(
            std::ffi::CStr::from_ptr(bcf_hdr_get_version(hdr).cast()).to_bytes(),
            b"VCFv4.2"
        );
        assert_eq!(bcf_hdr_name2id(hdr, b"20\0".as_ptr().cast()), 0);
        assert_sample_names(hdr, &[b"NA00001", b"NA00002", b"NA00003"]);
        assert!(bcf_hdr_id2int(hdr, hts_sys::BCF_DT_ID as i32, b"NEG\0".as_ptr().cast()) >= 0);
        assert!(bcf_hdr_id2int(hdr, hts_sys::BCF_DT_ID as i32, b"TS\0".as_ptr().cast()) >= 0);

        let rec = bcf_init();
        assert!(!rec.is_null());
        let mut seen = Vec::new();
        while bcf_read(fp, hdr, rec) >= 0 {
            assert_eq!(std::ffi::CStr::from_ptr(bcf_seqname(hdr, rec).cast()).to_bytes(), b"20");
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
        let fp = hts_open(vcf.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        let mut checked = 0;
        while bcf_read(fp, hdr, rec) >= 0 {
            let af = get_info_f32(hdr, rec, b"AF\0");
            let neg = get_info_i32(hdr, rec, b"NEG\0");
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
        let fp = hts_open(vcf.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());
        assert_eq!(bcf_read(fp, hdr, rec), 0);

        assert_alleles(rec, &[b"G", b"A"]);
        assert_eq!(
            get_format_strings(hdr, rec, b"TS\0"),
            vec![
                b"String1".to_vec(),
                b"SomeOtherString2".to_vec(),
                b"YetAnotherString3".to_vec(),
            ]
        );
        assert_eq!(get_format_i32(hdr, rec, b"GQ\0"), vec![48, 48, 43]);
        assert_eq!(get_format_i32(hdr, rec, b"DP\0"), vec![1, 8, 5]);
        assert_eq!(
            get_format_i32(hdr, rec, b"HQ\0"),
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
        let fp = hts_open(vcf.as_ptr().cast(), b"r\0".as_ptr().cast());
        assert!(!fp.is_null());

        let hdr = bcf_hdr_read(fp);
        assert!(!hdr.is_null());
        let rec = bcf_init();
        assert!(!rec.is_null());

        for _ in 0..3 {
            assert_eq!(bcf_read(fp, hdr, rec), 0);
        }
        assert_eq!(bcf_read(fp, hdr, rec), 0);
        assert_eq!(std::ffi::CStr::from_ptr(bcf_seqname(hdr, rec).cast()).to_bytes(), b"20");
        assert_eq!((*rec).pos, 1_110_695);
        assert_alleles(rec, &[b"A", b"G", b"T"]);
        // NOTE (2026-05-29): htslib v1.23 made haploid genotypes implicitly
        // phased even for VCF < 4.4 (test fixture is VCFv4.2). v1.19.1 returned
        // [6, EOV, 4, EOV, 0, 0] (no phase bit); v1.23 sets the phase bit on
        // the haploid allele → [7, EOV, 5, EOV, 0, 0]. Our native parser
        // faithfully matches v1.23 — confirmed by `vcf_parse_native_matches_hts_sys`
        // (native and the linked C produce identical byte output).
        assert_eq!(
            get_format_i32(hdr, rec, b"GT\0"),
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
        let path_bytes = path.to_string_lossy().into_owned().into_bytes();
        let mut sw = bcf_sweep_init(&path_bytes).expect("bcf_sweep_init failed");
        let hdr = bcf_sweep_hdr(sw.as_mut()).expect("bcf_sweep_hdr failed") as *mut _;

        let mut out = String::new();
        let mut chksum = 0;
        loop {
            let Some(rec) = bcf_sweep_fwd(sw.as_mut()) else {
                break;
            };
            chksum += rec.pos + 1;
        }
        out.push_str(&format!("fwd position chksum: {chksum}\n"));

        chksum = 0;
        loop {
            let Some(rec) = bcf_sweep_bwd(sw.as_mut()) else {
                break;
            };
            chksum += rec.pos + 1;
        }
        out.push_str(&format!("bwd position chksum: {chksum}\n"));

        for (label, forward) in [("fwd PL chksum", true), ("bwd PL chksum", false)] {
            chksum = 0;
            loop {
                let rec = if forward {
                    bcf_sweep_fwd(sw.as_mut())
                } else {
                    bcf_sweep_bwd(sw.as_mut())
                };
                let Some(rec) = rec else {
                    break;
                };
                let mut pls: *mut () = std::ptr::null_mut();
                let mut m_pls = 0;
                let n_pls = bcf_get_format_values(
                    hdr,
                    rec as *mut _,
                    b"PL\0".as_ptr().cast(),
                    &mut pls,
                    &mut m_pls,
                    hts_sys::BCF_HT_INT as i32,
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
            }
            out.push_str(&format!("{label}: {chksum}\n"));
        }

        bcf_sweep_destroy(Some(sw));
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
        let mut c_path = path.to_string_lossy().into_owned().into_bytes();
        c_path.push(0);
        let fp = hts_open(c_path.as_ptr().cast(), b"w\0".as_ptr().cast());
        assert!(!fp.is_null());

        let hdr1 = bcf_hdr_init(b"w\0".as_ptr().cast());
        let mut hdr2 = bcf_hdr_init(b"w\0".as_ptr().cast());
        assert!(!hdr1.is_null());
        assert!(!hdr2.is_null());

        for line in [
            b"##contig=<ID=1>\0".as_slice(),
            b"##contig=<ID=2>\0",
            b"##FILTER=<ID=FLT1,Description=\"Filter 1\">\0",
            b"##FILTER=<ID=FLT2,Description=\"Filter 2\">\0",
            b"##FILTER=<ID=FLT3,Description=\"Filter 3\">\0",
            b"##INFO=<ID=INF1,Number=.,Type=Integer,Description=\"Info 1\">\0",
            b"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">\0",
            b"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">\0",
            b"##FORMAT=<ID=FMT1,Number=.,Type=Integer,Description=\"FMT 1\">\0",
            b"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">\0",
            b"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">\0",
        ] {
            assert_eq!(bcf_hdr_append(hdr1, line.as_ptr().cast()), 0);
        }
        for line in [
            b"##contig=<ID=2>\0".as_slice(),
            b"##contig=<ID=1>\0",
            b"##FILTER=<ID=FLT4,Description=\"Filter 4\">\0",
            b"##FILTER=<ID=FLT3,Description=\"Filter 3\">\0",
            b"##FILTER=<ID=FLT2,Description=\"Filter 2\">\0",
            b"##INFO=<ID=INF4,Number=.,Type=Integer,Description=\"Info 4\">\0",
            b"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">\0",
            b"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">\0",
            b"##FORMAT=<ID=FMT4,Number=.,Type=Integer,Description=\"FMT 4\">\0",
            b"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">\0",
            b"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">\0",
        ] {
            assert_eq!(bcf_hdr_append(hdr2, line.as_ptr().cast()), 0);
        }
        assert_eq!(bcf_hdr_add_sample(hdr1, b"SMPL1\0".as_ptr().cast()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr1, b"SMPL2\0".as_ptr().cast()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr2, b"SMPL1\0".as_ptr().cast()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr2, b"SMPL2\0".as_ptr().cast()), 0);
        assert_eq!(bcf_hdr_sync(hdr1), 0);
        assert_eq!(bcf_hdr_sync(hdr2), 0);

        hdr2 = bcf_hdr_merge(hdr2, hdr1);
        assert!(!hdr2.is_null());
        assert_eq!(bcf_hdr_sync(hdr2), 0);
        assert_eq!(bcf_hdr_write(fp, hdr2), 0);

        let rec = bcf_init();
        assert!(!rec.is_null());
        (*rec).rid = bcf_hdr_name2id(hdr1, b"1\0".as_ptr().cast());
        (*rec).pos = 0;
        assert_eq!(bcf_update_alleles_str(hdr1, rec, b"G,A\0".as_ptr().cast()), 0);

        let mut tmpi = [
            bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as i32, b"FLT1\0".as_ptr().cast()),
            bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as i32, b"FLT2\0".as_ptr().cast()),
            bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as i32, b"FLT3\0".as_ptr().cast()),
        ];
        assert_eq!(bcf_update_filter(hdr1, rec, tmpi.as_mut_ptr(), 3), 0);
        for (tag, value) in [(b"INF1\0".as_slice(), 1), (b"INF2\0", 2), (b"INF3\0", 3)] {
            tmpi[0] = value;
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    tag.as_ptr().cast(),
                    tmpi.as_ptr().cast(),
                    1,
                    hts_sys::BCF_HT_INT as i32
                ),
                0
            );
        }
        for (tag, value) in [(b"FMT1\0".as_slice(), 1), (b"FMT2\0", 2), (b"FMT3\0", 3)] {
            tmpi[0] = value;
            tmpi[1] = value;
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    tag.as_ptr().cast(),
                    tmpi.as_ptr().cast(),
                    2,
                    hts_sys::BCF_HT_INT as i32
                ),
                0
            );
        }

        assert_eq!(
            bcf_remove_filter(
                hdr1,
                rec,
                bcf_hdr_id2int(hdr1, hts_sys::BCF_DT_ID as i32, b"FLT2\0".as_ptr().cast()),
                0
            ),
            0
        );
        assert_eq!(
            bcf_update_info(
                hdr1,
                rec,
                b"INF2\0".as_ptr().cast(),
                std::ptr::null(),
                0,
                hts_sys::BCF_HT_INT as i32
            ),
            0
        );
        assert_eq!(
            bcf_update_format(
                hdr1,
                rec,
                b"FMT2\0".as_ptr().cast(),
                std::ptr::null(),
                0,
                hts_sys::BCF_HT_INT as i32
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
