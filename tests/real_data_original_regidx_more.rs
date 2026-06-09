use htslib_rs::{
    regidx_c_105_regidx_seq_names, regidx_c_246_regidx_init, regidx_c_311_regidx_destroy,
    regidx_c_401_regidx_overlap, regidx_c_466_regidx_parse_bed, regidx_c_498_regidx_parse_tab,
    regidx_c_584_regitr_init, regidx_c_606_regitr_destroy, regidx_c_612_regitr_overlap,
    regidx_c_91_regidx_seq_nregs, regidx_c_98_regidx_nregs, regidx_parse_f,
};
use libc::c_int;
use std::ffi::{CStr, CString};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn open_index(path: &str, parsef: regidx_parse_f) -> *mut htslib_rs::regidx_t {
    let path = c_fixture(path);
    let idx = regidx_c_246_regidx_init(path.as_ptr(), parsef, None, 0, std::ptr::null_mut());
    assert!(!idx.is_null(), "failed to load {}", path.to_string_lossy());
    idx
}

unsafe fn seq_names(idx: *mut htslib_rs::regidx_t) -> Vec<String> {
    let mut nseq: c_int = -1;
    let names = regidx_c_105_regidx_seq_names(idx, &mut nseq);
    assert!(!names.is_null());
    (0..nseq)
        .map(|i| {
            CStr::from_ptr(*names.add(i as usize))
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

unsafe fn overlaps(
    idx: *mut htslib_rs::regidx_t,
    seq: &CStr,
    beg: i64,
    end: i64,
) -> Vec<(i64, i64)> {
    let itr = regidx_c_584_regitr_init(idx);
    assert!(!itr.is_null());

    let ret = regidx_c_401_regidx_overlap(idx, seq.as_ptr(), beg, end, itr);
    let mut out = Vec::new();
    while regidx_c_612_regitr_overlap(itr) != 0 {
        assert_eq!(
            CStr::from_ptr((*itr).seq.expect("regidx overlap seq").as_ptr()),
            seq
        );
        out.push(((*itr).beg, (*itr).end));
    }
    assert_eq!(ret, if out.is_empty() { 0 } else { 1 });

    regidx_c_606_regitr_destroy(itr);
    out
}

fn clipped_covered_bases(target: (i64, i64), regs: &[(i64, i64)]) -> i64 {
    let mut clipped = regs
        .iter()
        .filter_map(|&(beg, end)| {
            let beg = beg.max(target.0);
            let end = end.min(target.1);
            (beg <= end).then_some((beg, end))
        })
        .collect::<Vec<_>>();
    clipped.sort_unstable();

    let mut covered = 0;
    let mut current: Option<(i64, i64)> = None;
    for (beg, end) in clipped {
        match current {
            None => current = Some((beg, end)),
            Some((cur_beg, cur_end)) if beg <= cur_end + 1 => {
                current = Some((cur_beg, cur_end.max(end)));
            }
            Some((cur_beg, cur_end)) => {
                covered += cur_end - cur_beg + 1;
                current = Some((beg, end));
            }
        }
    }
    if let Some((beg, end)) = current {
        covered += end - beg + 1;
    }
    covered
}

#[test]
fn tabix_bed_fixture_uses_original_zero_based_half_open_intervals() {
    unsafe {
        let idx = open_index("htslib/test/tabix/bed_file.bed", None);

        assert_eq!(regidx_c_98_regidx_nregs(idx), 15);
        assert_eq!(seq_names(idx), vec!["X", "Y", "Z"]);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"X".as_ptr()), 5);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"Y".as_ptr()), 5);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"Z".as_ptr()), 5);

        assert_eq!(overlaps(idx, c"X", 1000, 1099), vec![(1000, 1099)]);
        assert_eq!(overlaps(idx, c"X", 1100, 1199), vec![]);
        assert_eq!(
            overlaps(idx, c"X", 1499, 1600),
            vec![(1400, 1499), (1600, 1699)]
        );
        assert_eq!(
            overlaps(idx, c"Y", 100450, 100650),
            vec![
                (100000, 100899),
                (100200, 100699),
                (100400, 100499),
                (100600, 100699),
            ]
        );
        assert_eq!(overlaps(idx, c"Z", 100001, 100001), vec![]);
        assert_eq!(overlaps(idx, c"Z", 100008, 100008), vec![(100008, 100008)]);

        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn tabix_vcf_fixture_auto_parser_indexes_positions_as_points() {
    unsafe {
        let idx = open_index("htslib/test/tabix/vcf_file.vcf", None);

        assert_eq!(regidx_c_98_regidx_nregs(idx), 15);
        assert_eq!(seq_names(idx), vec!["1", "2", "3", "4"]);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"1".as_ptr()), 11);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"2".as_ptr()), 1);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"3".as_ptr()), 1);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"4".as_ptr()), 2);

        assert_eq!(
            overlaps(idx, c"1", 3_000_149, 3_000_149),
            vec![(3_000_149, 3_000_149)]
        );
        assert_eq!(
            overlaps(idx, c"1", 3_062_914, 3_062_914),
            vec![(3_062_914, 3_062_914), (3_062_914, 3_062_914)]
        );
        assert_eq!(
            overlaps(idx, c"1", 3_106_153, 3_106_153),
            vec![(3_106_153, 3_106_153), (3_106_153, 3_106_153)]
        );
        assert_eq!(
            overlaps(idx, c"1", 3_177_143, 3_177_143),
            vec![(3_177_143, 3_177_143), (3_177_143, 3_177_143)]
        );
        assert_eq!(overlaps(idx, c"1", 3_177_142, 3_177_142), vec![]);
        assert_eq!(
            overlaps(idx, c"2", 3_199_811, 3_199_811),
            vec![(3_199_811, 3_199_811)]
        );
        assert_eq!(
            overlaps(idx, c"4", 3_258_447, 3_258_500),
            vec![(3_258_447, 3_258_447), (3_258_500, 3_258_500)]
        );

        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn tabix_large_chr_vcf_fixture_keeps_high_coordinates_as_zero_based_points() {
    unsafe {
        let idx = open_index("htslib/test/tabix/large_chr.vcf", None);

        assert_eq!(regidx_c_98_regidx_nregs(idx), 12);
        assert_eq!(seq_names(idx), vec!["chr11", "chr20"]);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"chr11".as_ptr()), 3);
        assert_eq!(regidx_c_91_regidx_seq_nregs(idx, c"chr20".as_ptr()), 9);
        assert_eq!(overlaps(idx, c"chr20", 76_960, 76_960), vec![]);
        assert_eq!(
            overlaps(idx, c"chr20", 76_961, 76_961),
            vec![(76_961, 76_961)]
        );
        assert_eq!(
            overlaps(idx, c"chr20", 620_255_099, 630_255_199),
            vec![(620_255_099, 620_255_099), (630_255_199, 630_255_199)]
        );
        assert_eq!(
            overlaps(idx, c"chr20", 2_147_483_646, 2_147_483_646),
            vec![(2_147_483_646, 2_147_483_646)]
        );
        assert_eq!(
            overlaps(idx, c"chr20", 2_147_483_647, 2_147_483_647),
            vec![]
        );

        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn tabix_bed_fixture_multi_region_queries_match_expected_windows() {
    unsafe {
        let idx = open_index("htslib/test/tabix/bed_file.bed", None);

        let x = overlaps(idx, c"X", 1099, 1399);
        assert_eq!(x, vec![(1000, 1099), (1200, 1299)]);
        assert_eq!(clipped_covered_bases((1099, 1399), &x), 101);

        let y = overlaps(idx, c"Y", 99_999, 100_549);
        assert_eq!(
            y,
            vec![(100000, 100899), (100200, 100699), (100400, 100499)]
        );
        assert_eq!(clipped_covered_bases((99_999, 100_549), &y), 550);

        let z = overlaps(idx, c"Z", 99_999, 100_004);
        assert_eq!(
            z,
            vec![(100000, 100000), (100002, 100002), (100004, 100004)]
        );
        assert_eq!(clipped_covered_bases((99_999, 100_004), &z), 3);

        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn annot_tsv_nbp_fixture_matches_exact_regidx_overlaps() {
    unsafe {
        let src = open_index(
            "htslib/test/annot-tsv/src.9.txt",
            Some(regidx_c_498_regidx_parse_tab),
        );
        let dst = open_index(
            "htslib/test/annot-tsv/dst.9.txt",
            Some(regidx_c_498_regidx_parse_tab),
        );

        let dst_rows = overlaps(dst, c"1", 0, 10_000);
        assert_eq!(dst_rows, vec![(999, 3999), (4999, 5999), (6999, 7999)]);

        let src_for_first = overlaps(src, c"1", dst_rows[0].0, dst_rows[0].1);
        assert_eq!(src_for_first, vec![(999, 1999), (2999, 3999)]);
        assert_eq!(clipped_covered_bases(dst_rows[0], &src_for_first), 2002);
        assert_eq!(dst_rows[0].1 - dst_rows[0].0 + 1, 3001);

        let src_for_second = overlaps(src, c"1", dst_rows[1].0, dst_rows[1].1);
        assert_eq!(src_for_second, vec![(4999, 5999)]);
        assert_eq!(clipped_covered_bases(dst_rows[1], &src_for_second), 1001);
        assert_eq!(dst_rows[1].1 - dst_rows[1].0 + 1, 1001);

        let src_for_third = overlaps(src, c"1", dst_rows[2].0, dst_rows[2].1);
        assert_eq!(src_for_third, vec![]);
        assert_eq!(clipped_covered_bases(dst_rows[2], &src_for_third), 0);
        assert_eq!(dst_rows[2].1 - dst_rows[2].0 + 1, 1001);

        regidx_c_311_regidx_destroy(dst);
        regidx_c_311_regidx_destroy(src);
    }
}

#[test]
fn annot_tsv_coordinate_mode_fixture_distinguishes_tab_from_bed_edges() {
    unsafe {
        let tab = open_index(
            "htslib/test/annot-tsv/src.14.txt",
            Some(regidx_c_498_regidx_parse_tab),
        );
        let bed = open_index(
            "htslib/test/annot-tsv/src.14.txt",
            Some(regidx_c_466_regidx_parse_bed),
        );

        assert_eq!(overlaps(tab, c"1", 0, 1), vec![(0, 1)]);
        assert_eq!(overlaps(tab, c"1", 4, 5), vec![(4, 5)]);
        assert_eq!(overlaps(tab, c"1", 6, 6), vec![]);

        assert_eq!(overlaps(bed, c"1", 1, 1), vec![(1, 1)]);
        assert_eq!(overlaps(bed, c"1", 5, 6), vec![(5, 5)]);
        assert_eq!(overlaps(bed, c"1", 0, 0), vec![]);

        regidx_c_311_regidx_destroy(bed);
        regidx_c_311_regidx_destroy(tab);
    }
}
