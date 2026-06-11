use htslib_rs::{
    regidx_c_105_regidx_seq_names, regidx_c_246_regidx_init, regidx_c_311_regidx_destroy,
    regidx_c_401_regidx_overlap, regidx_c_498_regidx_parse_tab, regidx_c_584_regitr_init,
    regidx_c_606_regitr_destroy, regidx_c_612_regitr_overlap, regidx_c_91_regidx_seq_nregs,
    regidx_c_98_regidx_nregs,
};
use htslib_rs::regidx_t;

fn fixture_bytes(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    path.to_string_lossy().as_bytes().to_vec()
}

unsafe fn open_tab_index(path: &str) -> Box<regidx_t> {
    let fname = fixture_bytes(path);
    let idx = regidx_c_246_regidx_init(
        Some(&fname),
        Some(regidx_c_498_regidx_parse_tab),
        None,
        0,
        None,
    );
    assert!(
        idx.is_some(),
        "failed to load {}",
        String::from_utf8_lossy(&fname)
    );
    idx.unwrap()
}

fn collect_overlaps(idx: &mut regidx_t, seq: &[u8], beg: i64, end: i64) -> Vec<(i64, i64)> {
    let mut itr = regidx_c_584_regitr_init(idx);
    assert_eq!(
        regidx_c_401_regidx_overlap(idx, seq, beg, end, Some(&mut itr)),
        1
    );
    let mut out = Vec::new();
    while regidx_c_612_regitr_overlap(&mut itr) != 0 {
        out.push((itr.beg, itr.end));
    }
    regidx_c_606_regitr_destroy(itr);
    out
}

fn assert_no_overlap(idx: &mut regidx_t, seq: &[u8], beg: i64, end: i64) {
    let mut itr = regidx_c_584_regitr_init(idx);
    assert_eq!(
        regidx_c_401_regidx_overlap(idx, seq, beg, end, Some(&mut itr)),
        0
    );
    assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 0);
    regidx_c_606_regitr_destroy(itr);
}

#[test]
fn annot_tsv_source_index_skips_header_and_reports_exact_names() {
    unsafe {
        let idx = open_tab_index("htslib/test/annot-tsv/src.1.txt");
        assert_eq!(regidx_c_98_regidx_nregs(&idx), 4);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"1"), 4);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"2"), 0);

        let names = regidx_c_105_regidx_seq_names(&idx);
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].as_slice(), b"1");

        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn annot_tsv_source_overlaps_match_one_based_tab_coordinates() {
    unsafe {
        let mut idx = open_tab_index("htslib/test/annot-tsv/src.1.txt");

        assert_eq!(
            collect_overlaps(&mut idx, b"1", 13, 14),
            vec![(9, 19), (9, 19), (14, 14)]
        );
        assert_eq!(collect_overlaps(&mut idx, b"1", 34, 34), vec![(29, 39)]);
        assert_no_overlap(&mut idx, b"1", 20, 28);
        assert_no_overlap(&mut idx, b"2", 0, 100);

        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn annot_tsv_without_header_loads_same_intervals() {
    unsafe {
        let mut idx = open_tab_index("htslib/test/annot-tsv/src.2.txt");
        assert_eq!(regidx_c_98_regidx_nregs(&idx), 4);
        assert_eq!(
            collect_overlaps(&mut idx, b"1", 9, 19),
            vec![(9, 19), (9, 19), (14, 14)]
        );
        assert_eq!(collect_overlaps(&mut idx, b"1", 29, 39), vec![(29, 39)]);
        regidx_c_311_regidx_destroy(idx);
    }
}

#[test]
fn annot_tsv_destination_queries_select_expected_source_overlaps() {
    unsafe {
        let mut src = open_tab_index("htslib/test/annot-tsv/src.1.txt");
        let mut dst = open_tab_index("htslib/test/annot-tsv/dst.1.txt");

        let dst_rows = collect_overlaps(&mut dst, b"1", 0, 100);
        assert_eq!(dst_rows, vec![(13, 14), (34, 34)]);
        assert_eq!(
            collect_overlaps(&mut src, b"1", dst_rows[0].0, dst_rows[0].1),
            vec![(9, 19), (9, 19), (14, 14)]
        );
        assert_eq!(
            collect_overlaps(&mut src, b"1", dst_rows[1].0, dst_rows[1].1),
            vec![(29, 39)]
        );

        regidx_c_311_regidx_destroy(dst);
        regidx_c_311_regidx_destroy(src);
    }
}
