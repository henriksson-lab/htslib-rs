use htslib_rs::bgzf::{bgzf_close, bgzf_open, bgzf_write};
use htslib_rs::tabix::tabix_c_135_parse_regions;
use htslib_rs::{
    hts_close, hts_get_bgzfp, hts_itr_destroy, hts_itr_next, hts_open, ks_free, kstring_t,
    tbx_c_96_tbx_parse1, tbx_conf_bed, tbx_conf_gff, tbx_conf_vcf, tbx_destroy, tbx_index_build2,
    tbx_index_load2, tbx_intv_t, tbx_itr_querys1,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, PartialEq, Eq)]
struct ParsedInterval {
    seq: String,
    beg: i64,
    end: i64,
}

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_path(path: &std::path::Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

fn temp_path(name: &str) -> std::path::PathBuf {
    static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);
    let dir = fixture(".tmp").join("real_data_original_tabix_parse_more");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!(
        "{}-{}-{}",
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed),
        name
    ))
}

unsafe fn bgzip_copy(src: &std::path::Path, dst: &std::path::Path) {
    let bytes = std::fs::read(src).unwrap();
    let dst_c = c_path(dst);
    let fp = bgzf_open(dst_c.as_ptr(), c"w".as_ptr());
    assert!(!fp.is_null(), "failed to create {}", dst.display());
    assert_eq!(
        bgzf_write(fp, bytes.as_ptr().cast::<c_void>(), bytes.len()),
        bytes.len() as isize
    );
    assert_eq!(bgzf_close(fp), 0);
}

unsafe fn query_tabix_rows(
    src: &str,
    index_suffix: &str,
    min_shift: i32,
    conf: &htslib_rs::tbx_conf_t,
    region: &CStr,
) -> String {
    let bgz = temp_path(&format!("{src}.{index_suffix}.gz").replace('/', "_"));
    let idx = temp_path(&format!("{src}.{index_suffix}.gz.{index_suffix}").replace('/', "_"));
    let _ = std::fs::remove_file(&bgz);
    let _ = std::fs::remove_file(&idx);
    bgzip_copy(&fixture(src), &bgz);

    let bgz_c = c_path(&bgz);
    let idx_c = c_path(&idx);
    assert_eq!(
        tbx_index_build2(bgz_c.as_ptr(), idx_c.as_ptr(), min_shift, conf),
        0
    );

    let tbx = tbx_index_load2(bgz_c.as_ptr(), idx_c.as_ptr());
    assert!(!tbx.is_null());
    let fp = hts_open(bgz_c.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null());
    let itr = tbx_itr_querys1(tbx, region.as_ptr());
    assert!(!itr.is_null(), "failed to query region {region:?}");

    let mut line: kstring_t = std::mem::zeroed();
    let mut out = String::new();
    loop {
        let ret = hts_itr_next(
            hts_get_bgzfp(fp),
            itr,
            (&mut line as *mut kstring_t).cast::<c_void>(),
            tbx.cast::<c_void>(),
        );
        if ret < 0 {
            break;
        }
        let bytes = std::slice::from_raw_parts(line.s.cast::<u8>(), line.l);
        out.push_str(&String::from_utf8_lossy(bytes));
        out.push('\n');
    }

    ks_free(&mut line);
    hts_itr_destroy(itr);
    assert_eq!(hts_close(fp), 0);
    tbx_destroy(tbx);
    let _ = std::fs::remove_file(bgz);
    let _ = std::fs::remove_file(idx);
    out
}

unsafe fn parse_interval(conf: &htslib_rs::tbx_conf_t, line: &str) -> ParsedInterval {
    let mut bytes = CString::new(line).unwrap().into_bytes_with_nul();
    let mut intv: tbx_intv_t = std::mem::zeroed();
    assert_eq!(
        tbx_c_96_tbx_parse1(conf, bytes.len() - 1, bytes.as_mut_ptr().cast(), &mut intv),
        0,
        "failed to parse tabix data line: {line}"
    );

    let ss = intv.ss.expect("parsed interval sequence start").as_ptr();
    let seq = if let Some(se) = intv.se {
        let len = se.as_ptr().offset_from(ss) as usize;
        let bytes = std::slice::from_raw_parts(ss.cast::<u8>(), len);
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        CStr::from_ptr(ss).to_string_lossy().into_owned()
    };

    ParsedInterval {
        seq,
        beg: intv.beg,
        end: intv.end,
    }
}

#[test]
fn tabix_parse_regions_expands_region_file_and_appends_argv_regions() {
    unsafe {
        let regions = fixture("htslib/test/tabix/bed_file.bed");
        let regions_c = c_path(&regions);
        let argv = [CString::new("chr7:10-20").unwrap()];
        let mut argv_ptrs = argv
            .iter()
            .map(|arg| arg.as_ptr().cast_mut())
            .collect::<Vec<_>>();
        let mut nregs = 0;

        let regs = tabix_c_135_parse_regions(
            regions_c.as_ptr().cast_mut(),
            argv_ptrs.as_mut_ptr(),
            argv_ptrs.len() as libc::c_int,
            &mut nregs,
        );
        assert!(!regs.is_null());
        assert_eq!(nregs, 16);

        let expanded = (0..nregs as usize)
            .map(|i| CStr::from_ptr(*regs.add(i)).to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(expanded.first().unwrap(), "X:1001-1100");
        assert_eq!(expanded[5], "Y:100001-100900");
        assert_eq!(expanded[14], "Z:100009-100009");
        assert_eq!(expanded.last().unwrap(), "chr7:10-20");

        for i in 0..nregs as usize {
            libc::free((*regs.add(i)).cast());
        }
        libc::free(regs.cast());
    }
}

#[test]
fn indexes_original_tabix_vcf_tbi_and_csi_queries_exact_expected_output() {
    unsafe {
        let conf = tbx_conf_vcf();
        for (suffix, min_shift) in [("tbi", 0), ("csi", 14)] {
            assert_eq!(
                query_tabix_rows(
                    "htslib/test/tabix/vcf_file.vcf",
                    suffix,
                    min_shift,
                    &conf,
                    c"1:3000151-3000151",
                ),
                include_str!("../htslib/test/tabix/vcf_file.1.3000151.out")
            );
            assert_eq!(
                query_tabix_rows(
                    "htslib/test/tabix/vcf_file.vcf",
                    suffix,
                    min_shift,
                    &conf,
                    c"2:3199812-3199812",
                ),
                include_str!("../htslib/test/tabix/vcf_file.2.3199812.out")
            );
        }
    }
}

#[test]
fn indexes_original_tabix_bed_and_gff_queries_exact_expected_output() {
    unsafe {
        let bed_conf = tbx_conf_bed();
        assert_eq!(
            query_tabix_rows(
                "htslib/test/tabix/bed_file.bed",
                "tbi",
                0,
                &bed_conf,
                c"Y:100200-100200",
            ),
            include_str!("../htslib/test/tabix/bed_file.Y.100200.out")
        );

        let gff_conf = tbx_conf_gff();
        assert_eq!(
            query_tabix_rows(
                "htslib/test/tabix/gff_file.gff",
                "tbi",
                0,
                &gff_conf,
                c"X:2934832-2935190",
            ),
            include_str!("../htslib/test/tabix/gff_file.X.2934832.2935190.out")
        );
    }
}

#[test]
fn original_large_chr_tbi_fails_but_csi_query_matches_expected_output() {
    unsafe {
        let bgz = temp_path("large_chr.tmp.vcf.gz");
        let tbi = temp_path("large_chr.tmp.vcf.gz.tbi");
        let _ = std::fs::remove_file(&bgz);
        let _ = std::fs::remove_file(&tbi);
        bgzip_copy(&fixture("htslib/test/tabix/large_chr.vcf"), &bgz);

        let conf = tbx_conf_vcf();
        let bgz_c = c_path(&bgz);
        let tbi_c = c_path(&tbi);
        assert_ne!(
            tbx_index_build2(bgz_c.as_ptr(), tbi_c.as_ptr(), 0, &conf),
            0
        );
        let _ = std::fs::remove_file(&bgz);
        let _ = std::fs::remove_file(&tbi);

        assert_eq!(
            query_tabix_rows(
                "htslib/test/tabix/large_chr.vcf",
                "csi",
                14,
                &conf,
                c"chr20:1-2147483647",
            ),
            include_str!("../htslib/test/tabix/large_chr.20.1.2147483647.out")
        );
    }
}

#[test]
fn original_large_chr_csi_query_includes_terminal_i32_boundary_record() {
    unsafe {
        let conf = tbx_conf_vcf();
        assert_eq!(
            query_tabix_rows(
                "htslib/test/tabix/large_chr.vcf",
                "csi",
                14,
                &conf,
                c"chr20:2147483647-2147483647",
            ),
            "chr20\t2147483647\t.\tA\tT\t999\tPASS\t.\n"
        );
    }
}

fn non_comment_lines(text: &'static str) -> impl Iterator<Item = &'static str> {
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn overlaps_half_open(interval: &ParsedInterval, seq: &str, beg: i64, end: i64) -> bool {
    interval.seq == seq && interval.beg < end && beg < interval.end
}

#[test]
fn parses_original_tabix_vcf_query_outputs_without_index_iteration() {
    unsafe {
        let conf = tbx_conf_vcf();
        let cases = [
            (
                include_str!("../htslib/test/tabix/vcf_file.1.3000151.out"),
                ParsedInterval {
                    seq: "1".to_string(),
                    beg: 3_000_150,
                    end: 3_000_151,
                },
            ),
            (
                include_str!("../htslib/test/tabix/vcf_file.2.3199812.out"),
                ParsedInterval {
                    seq: "2".to_string(),
                    beg: 3_199_811,
                    end: 3_199_812,
                },
            ),
        ];

        for (text, expected) in cases {
            let line = non_comment_lines(text).next().unwrap();
            assert_eq!(parse_interval(&conf, line), expected);
        }
    }
}

#[test]
fn parses_original_tabix_bed_expected_outputs_and_region_boundaries() {
    unsafe {
        let conf = tbx_conf_bed();

        let direct = non_comment_lines(include_str!("../htslib/test/tabix/bed_file.Y.100200.out"))
            .next()
            .unwrap();
        assert_eq!(
            parse_interval(&conf, direct),
            ParsedInterval {
                seq: "Y".to_string(),
                beg: 100_000,
                end: 100_900,
            }
        );

        let separate_regions = [
            ("X", 1_100, 1_400),
            ("Y", 100_000, 100_550),
            ("Z", 100_000, 100_005),
        ];
        let expected = [
            ParsedInterval {
                seq: "X".to_string(),
                beg: 1_200,
                end: 1_300,
            },
            ParsedInterval {
                seq: "Y".to_string(),
                beg: 100_000,
                end: 100_900,
            },
            ParsedInterval {
                seq: "Y".to_string(),
                beg: 100_200,
                end: 100_700,
            },
            ParsedInterval {
                seq: "Z".to_string(),
                beg: 100_000,
                end: 100_001,
            },
            ParsedInterval {
                seq: "Z".to_string(),
                beg: 100_002,
                end: 100_003,
            },
            ParsedInterval {
                seq: "Z".to_string(),
                beg: 100_004,
                end: 100_005,
            },
        ];

        let parsed: Vec<_> =
            non_comment_lines(include_str!("../htslib/test/tabix/bed_file.separate.out"))
                .filter(|line| {
                    !line.starts_with("X\t1000\t1100") && !line.starts_with("Y\t100400\t100500")
                })
                .map(|line| parse_interval(&conf, line))
                .collect();

        assert_eq!(parsed, expected);
        for interval in &parsed {
            assert!(
                separate_regions
                    .iter()
                    .any(|(seq, beg, end)| overlaps_half_open(interval, seq, *beg, *end)),
                "parsed interval did not overlap any original --separate-regions query: {interval:?}"
            );
        }
    }
}

#[test]
fn parses_original_tabix_gff_expected_output_rows_without_iterator() {
    unsafe {
        let conf = tbx_conf_gff();
        let expected = [
            ParsedInterval {
                seq: "X".to_string(),
                beg: 2_934_815,
                end: 2_964_270,
            },
            ParsedInterval {
                seq: "X".to_string(),
                beg: 2_934_815,
                end: 2_964_270,
            },
            ParsedInterval {
                seq: "X".to_string(),
                beg: 2_934_831,
                end: 2_935_190,
            },
        ];

        let parsed: Vec<_> = non_comment_lines(include_str!(
            "../htslib/test/tabix/gff_file.X.2934832.2935190.out"
        ))
        .skip(1)
        .map(|line| parse_interval(&conf, line))
        .collect();

        assert_eq!(parsed, expected);
        assert!(parsed
            .iter()
            .all(|interval| overlaps_half_open(interval, "X", 2_934_831, 2_935_190)));
    }
}

#[test]
fn parses_original_tabix_large_chr_csi_output_rows_below_i32_boundary() {
    unsafe {
        let conf = tbx_conf_vcf();
        let expected = [
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 76_961,
                end: 76_962,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 126_309,
                end: 126_312,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 138_124,
                end: 138_125,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 138_147,
                end: 138_148,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 271_224,
                end: 271_225,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 304_567,
                end: 304_568,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 620_255_099,
                end: 620_255_101,
            },
            ParsedInterval {
                seq: "chr20".to_string(),
                beg: 630_255_199,
                end: 630_255_200,
            },
        ];

        let parsed: Vec<_> = non_comment_lines(include_str!(
            "../htslib/test/tabix/large_chr.20.1.2147483647.out"
        ))
        .take(8)
        .map(|line| parse_interval(&conf, line))
        .collect();

        assert_eq!(parsed, expected);
        assert!(parsed.iter().all(|interval| overlaps_half_open(
            interval,
            "chr20",
            0,
            2_147_483_647
        )));
    }
}

#[test]
fn vcf_alt_allele_limit_restores_temporary_field_split_like_c() {
    unsafe {
        let conf = tbx_conf_vcf();
        let alt = vec!["A"; 65_536].join(",");
        let line = format!("1\t100\t.\tG\t{alt}\t.\tPASS\t.");
        let mut bytes = CString::new(line.as_str()).unwrap().into_bytes_with_nul();
        let original = bytes.clone();
        let mut intv: htslib_rs::tbx_intv_t = std::mem::zeroed();

        assert_eq!(
            tbx_c_96_tbx_parse1(&conf, bytes.len() - 1, bytes.as_mut_ptr().cast(), &mut intv),
            0
        );
        assert_eq!(bytes, original);
        assert_eq!(intv.beg, 99);
        assert_eq!(intv.end, 100);
    }
}
