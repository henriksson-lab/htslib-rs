use htslib_mini_rs::{tbx_c_96_tbx_parse1, tbx_conf_bed, tbx_conf_gff, tbx_conf_vcf};
use std::ffi::{CStr, CString};

#[derive(Debug, PartialEq, Eq)]
struct ParsedInterval {
    seq: String,
    beg: i64,
    end: i64,
}

unsafe fn parse_interval(conf: &htslib_mini_rs::tbx_conf_t, line: &str) -> ParsedInterval {
    let mut bytes = CString::new(line).unwrap().into_bytes_with_nul();
    let mut intv: htslib_mini_rs::tbx_intv_t = std::mem::zeroed();
    assert_eq!(
        tbx_c_96_tbx_parse1(conf, bytes.len() - 1, bytes.as_mut_ptr().cast(), &mut intv),
        0,
        "failed to parse tabix data line: {line}"
    );

    let seq = if intv.se.is_null() {
        CStr::from_ptr(intv.ss).to_string_lossy().into_owned()
    } else {
        let len = intv.se.offset_from(intv.ss) as usize;
        let bytes = std::slice::from_raw_parts(intv.ss.cast::<u8>(), len);
        String::from_utf8_lossy(bytes).into_owned()
    };

    ParsedInterval {
        seq,
        beg: intv.beg,
        end: intv.end,
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
        let mut intv: htslib_mini_rs::tbx_intv_t = std::mem::zeroed();

        assert_eq!(
            tbx_c_96_tbx_parse1(&conf, bytes.len() - 1, bytes.as_mut_ptr().cast(), &mut intv),
            0
        );
        assert_eq!(bytes, original);
        assert_eq!(intv.beg, 99);
        assert_eq!(intv.end, 100);
    }
}
