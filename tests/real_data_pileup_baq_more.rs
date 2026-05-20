use htslib_rs::{
    bam1_t, bam_aux2Z, bam_aux_get, bam_destroy1, bam_get_qname, bam_get_qual, bam_init1,
    bam_pileup1_is_del, bam_pileup1_is_head, bam_pileup1_is_refskip, bam_pileup1_is_tail,
    bam_pileup1_t, bam_plp_auto, bam_plp_destroy, bam_plp_init, bam_plp_init_overlaps, fai_destroy,
    fai_load3_format, faidx_fetch_seq, htsFile, hts_close, hts_open, sam_hdr_destroy, sam_hdr_read,
    sam_hdr_t, sam_hdr_tid2len, sam_hdr_tid2name, sam_prob_realn, sam_read1, FAI_FASTA,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

struct SamReader {
    fp: *mut htsFile,
    hdr: *mut sam_hdr_t,
}

impl SamReader {
    unsafe fn open(path: &str) -> Self {
        let path = c_fixture(path);
        let fp = hts_open(path.as_ptr(), c"r".as_ptr());
        assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

        let hdr = sam_hdr_read(fp);
        assert!(
            !hdr.is_null(),
            "failed to read header from {}",
            path.to_string_lossy()
        );

        Self { fp, hdr }
    }

    unsafe fn target_name(&self, tid: c_int) -> String {
        CStr::from_ptr(sam_hdr_tid2name(self.hdr, tid))
            .to_string_lossy()
            .into_owned()
    }
}

impl Drop for SamReader {
    fn drop(&mut self) {
        unsafe {
            sam_hdr_destroy(self.hdr);
            assert_eq!(hts_close(self.fp), 0);
        }
    }
}

unsafe extern "C" fn read_record(data: *mut c_void, rec: *mut bam1_t) -> c_int {
    let reader = &mut *(data.cast::<SamReader>());
    sam_read1(reader.fp, reader.hdr, rec)
}

#[derive(Debug, PartialEq, Eq)]
struct PileupColumn {
    target: String,
    pos1: i64,
    depth: i32,
    heads: i32,
    tails: i32,
    dels: i32,
    refskips: i32,
    indels: Vec<i32>,
    qpos: Vec<i32>,
    qnames: Vec<String>,
}

unsafe fn collect_pileup(path: &str, overlaps: bool) -> Vec<PileupColumn> {
    let mut reader = SamReader::open(path);
    let iter = bam_plp_init(Some(read_record), (&mut reader as *mut SamReader).cast());
    assert!(!iter.is_null());
    if overlaps {
        assert_eq!(bam_plp_init_overlaps(iter), 0);
    }

    let mut columns = Vec::new();
    loop {
        let mut tid = -1;
        let mut pos = -1;
        let mut depth = 0;
        let pileup = bam_plp_auto(iter, &mut tid, &mut pos, &mut depth);
        if pileup.is_null() {
            assert_eq!(depth, 0);
            break;
        }

        let entries = std::slice::from_raw_parts(pileup, depth as usize);
        let mut column = PileupColumn {
            target: reader.target_name(tid),
            pos1: pos as i64 + 1,
            depth,
            heads: 0,
            tails: 0,
            dels: 0,
            refskips: 0,
            indels: Vec::new(),
            qpos: Vec::new(),
            qnames: Vec::new(),
        };

        for p in entries {
            column.heads += bam_pileup1_is_head(p) as i32;
            column.tails += bam_pileup1_is_tail(p) as i32;
            column.dels += bam_pileup1_is_del(p) as i32;
            column.refskips += bam_pileup1_is_refskip(p) as i32;
            column.indels.push(p.indel);
            column.qpos.push(p.qpos);
            column.qnames.push(
                CStr::from_ptr(bam_get_qname(p.b))
                    .to_string_lossy()
                    .into_owned(),
            );
        }

        columns.push(column);
    }

    bam_plp_destroy(iter);
    columns
}

unsafe fn read_all_records(path: &str) -> (*mut sam_hdr_t, Vec<*mut bam1_t>) {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());

    let hdr = sam_hdr_read(fp);
    assert!(
        !hdr.is_null(),
        "failed to read header from {}",
        path.to_string_lossy()
    );

    let mut records = Vec::new();
    loop {
        let rec = bam_init1();
        assert!(!rec.is_null());
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            bam_destroy1(rec);
            break;
        }
        records.push(rec);
    }

    assert_eq!(hts_close(fp), 0);
    (hdr, records)
}

unsafe fn aux_z(rec: *const bam1_t, tag: *const i8) -> String {
    let aux = bam_aux_get(rec, tag);
    assert!(!aux.is_null());
    CStr::from_ptr(bam_aux2Z(aux))
        .to_string_lossy()
        .into_owned()
}

unsafe fn maybe_aux_z(rec: *const bam1_t, tag: *const i8) -> Option<String> {
    let aux = bam_aux_get(rec, tag);
    if aux.is_null() {
        None
    } else {
        Some(
            CStr::from_ptr(bam_aux2Z(aux))
                .to_string_lossy()
                .into_owned(),
        )
    }
}

unsafe fn qual_text(rec: *const bam1_t) -> String {
    let qual = bam_get_qual(rec);
    let len = (*rec).core.l_qseq as usize;
    (0..len)
        .map(|i| char::from(*qual.add(i) + 33))
        .collect::<String>()
}

unsafe fn assert_realigned_records_match_expected(
    fasta_path: &str,
    fai_path: &str,
    target_name: &CStr,
    expected_ref_len: c_int,
    input_path: &str,
    expected_path: &str,
    flags: c_int,
) {
    let fasta = c_fixture(fasta_path);
    let fai_path = c_fixture(fai_path);
    let fai = fai_load3_format(
        fasta.as_ptr(),
        fai_path.as_ptr(),
        std::ptr::null(),
        0,
        FAI_FASTA,
    );
    assert!(!fai.is_null());
    let mut ref_len = 0;
    let ref_seq = faidx_fetch_seq(fai, target_name.as_ptr(), 0, c_int::MAX, &mut ref_len);
    assert!(!ref_seq.is_null());
    assert_eq!(ref_len, expected_ref_len);

    let (input_hdr, input_records) = read_all_records(input_path);
    let (expected_hdr, expected_records) = read_all_records(expected_path);
    assert_eq!(input_records.len(), expected_records.len());
    assert_eq!(
        sam_hdr_tid2len(input_hdr, 0),
        sam_hdr_tid2len(expected_hdr, 0)
    );

    for (observed, expected) in input_records.iter().zip(expected_records.iter()) {
        if (**observed).core.tid >= 0 {
            let ret = sam_prob_realn(*observed, ref_seq, ref_len.into(), flags);
            assert!(ret > -4, "sam_prob_realn returned {ret}");
        }

        assert_eq!(
            CStr::from_ptr(bam_get_qname(*observed)).to_bytes(),
            CStr::from_ptr(bam_get_qname(*expected)).to_bytes()
        );
        assert_eq!((**observed).core.tid, (**expected).core.tid);
        assert_eq!((**observed).core.pos, (**expected).core.pos);
        assert_eq!((**observed).core.flag, (**expected).core.flag);
        assert_eq!((**observed).core.n_cigar, (**expected).core.n_cigar);
        assert_eq!((**observed).core.l_qseq, (**expected).core.l_qseq);
        assert_eq!(qual_text(*observed), qual_text(*expected));
        assert_eq!(
            maybe_aux_z(*observed, c"BQ".as_ptr()),
            maybe_aux_z(*expected, c"BQ".as_ptr())
        );
        assert_eq!(
            maybe_aux_z(*observed, c"ZQ".as_ptr()),
            maybe_aux_z(*expected, c"ZQ".as_ptr())
        );
    }

    for rec in input_records {
        bam_destroy1(rec);
    }
    for rec in expected_records {
        bam_destroy1(rec);
    }
    sam_hdr_destroy(input_hdr);
    sam_hdr_destroy(expected_hdr);
    libc::free(ref_seq.cast());
    fai_destroy(fai);
}

unsafe fn base_quals_at(column: &PileupColumn, raw: *const bam_pileup1_t) -> Vec<u8> {
    let entries = std::slice::from_raw_parts(raw, column.depth as usize);
    entries
        .iter()
        .map(|p| *bam_get_qual(p.b).add(p.qpos as usize))
        .collect()
}

#[test]
fn mpileup_deletion_fixture_reports_exact_pileup_columns() {
    unsafe {
        let columns = collect_pileup("htslib/test/mpileup/mp_D.sam", false);

        let observed: Vec<_> = columns
            .iter()
            .map(|c| {
                (
                    c.target.as_str(),
                    c.pos1,
                    c.depth,
                    c.heads,
                    c.tails,
                    c.dels,
                    c.refskips,
                    c.indels.clone(),
                    c.qpos.clone(),
                    c.qnames.iter().map(String::as_str).collect::<Vec<&str>>(),
                )
            })
            .collect();

        assert_eq!(
            observed,
            vec![
                (
                    "z",
                    2,
                    3,
                    3,
                    0,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![0, 0, 0],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    3,
                    3,
                    0,
                    0,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![1, 1, 0],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    4,
                    3,
                    0,
                    0,
                    0,
                    0,
                    vec![0, 0, 0],
                    vec![2, 2, 0],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    5,
                    3,
                    0,
                    0,
                    0,
                    0,
                    vec![0, -3, 0],
                    vec![3, 3, 1],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    6,
                    3,
                    0,
                    0,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![4, 4, 2],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    7,
                    3,
                    0,
                    0,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![5, 4, 3],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    8,
                    3,
                    0,
                    0,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![6, 4, 4],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    9,
                    3,
                    0,
                    0,
                    0,
                    0,
                    vec![0, 0, 0],
                    vec![7, 4, 5],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    10,
                    3,
                    0,
                    0,
                    0,
                    0,
                    vec![0, 0, -2],
                    vec![8, 5, 6],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    11,
                    3,
                    0,
                    0,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![9, 6, 7],
                    vec!["s1", "s2", "s3"]
                ),
                (
                    "z",
                    12,
                    3,
                    0,
                    3,
                    1,
                    0,
                    vec![0, 0, 0],
                    vec![10, 7, 7],
                    vec!["s1", "s2", "s3"]
                ),
            ]
        );
    }
}

#[test]
fn mpileup_insertion_deletion_fixture_preserves_indel_lengths_and_positions() {
    unsafe {
        let columns = collect_pileup("htslib/test/mpileup/mp_ID.sam", false);
        assert_eq!(columns.len(), 12);

        assert_eq!(
            (
                columns[0].target.as_str(),
                columns[0].pos1,
                columns[0].depth
            ),
            ("z", 2, 3)
        );
        assert_eq!(columns[0].heads, 3);
        assert_eq!(columns[0].indels, vec![0, 0, 0]);
        assert_eq!(columns[0].qpos, vec![0, 0, 0]);

        assert_eq!(
            (
                columns[2].target.as_str(),
                columns[2].pos1,
                columns[2].depth
            ),
            ("z", 4, 5)
        );
        assert_eq!(columns[2].heads, 2);
        assert_eq!(columns[2].indels, vec![0, 0, 0, 0, 0]);
        assert_eq!(columns[2].qnames, vec!["s1", "s2", "s3", "s4", "s5"]);
        assert_eq!(columns[2].qpos, vec![2, 2, 2, 2, 1]);

        assert_eq!(
            (
                columns[6].target.as_str(),
                columns[6].pos1,
                columns[6].depth
            ),
            ("z", 8, 5)
        );
        assert_eq!(columns[6].indels, vec![2, 2, 1, 0, 0]);
        assert_eq!(columns[6].qpos, vec![6, 6, 6, 4, 3]);

        assert_eq!(
            (
                columns[8].target.as_str(),
                columns[8].pos1,
                columns[8].depth
            ),
            ("z", 10, 5)
        );
        assert_eq!(columns[8].tails, 2);
        assert_eq!(columns[8].dels, 3);
        assert_eq!(columns[8].qpos, vec![9, 9, 8, 6, 5]);

        assert_eq!(
            (
                columns[11].target.as_str(),
                columns[11].pos1,
                columns[11].depth
            ),
            ("z", 13, 1)
        );
        assert_eq!(columns[11].tails, 1);
        assert_eq!(columns[11].qnames, vec!["s1"]);
    }
}

#[test]
fn mpileup_overlap_fixture_reports_overlap_adjusted_qualities() {
    unsafe {
        let mut reader = SamReader::open("htslib/test/mpileup/mp_overlap1.sam");
        let iter = bam_plp_init(Some(read_record), (&mut reader as *mut SamReader).cast());
        assert!(!iter.is_null());
        assert_eq!(bam_plp_init_overlaps(iter), 0);

        let mut tid = -1;
        let mut pos = -1;
        let mut depth = 0;
        let pileup = bam_plp_auto(iter, &mut tid, &mut pos, &mut depth);
        assert!(!pileup.is_null());

        let column = PileupColumn {
            target: reader.target_name(tid),
            pos1: pos as i64 + 1,
            depth,
            heads: 0,
            tails: 0,
            dels: 0,
            refskips: 0,
            indels: Vec::new(),
            qpos: Vec::new(),
            qnames: Vec::new(),
        };
        let mut qualities = base_quals_at(&column, pileup);
        qualities.sort_unstable();

        let entries = std::slice::from_raw_parts(pileup, depth as usize);
        assert_eq!(
            (column.target.as_str(), column.pos1, column.depth),
            ("1", 100003, 2)
        );
        assert_eq!(sam_hdr_tid2len(reader.hdr, tid), 249_250_621);
        assert_eq!(
            entries
                .iter()
                .map(|p| CStr::from_ptr(bam_get_qname(p.b))
                    .to_string_lossy()
                    .into_owned())
                .collect::<Vec<_>>(),
            vec!["r1", "r1"]
        );
        assert_eq!(
            entries.iter().map(|p| p.qpos).collect::<Vec<_>>(),
            vec![0, 0]
        );
        assert_eq!(
            entries.iter().map(|p| p.indel).collect::<Vec<_>>(),
            vec![0, 0]
        );
        assert_eq!(qualities, vec![0, 90]);

        bam_plp_destroy(iter);
    }
}

#[test]
fn realn_baq_expected_fixture_exposes_exact_baq_tags_and_metadata() {
    unsafe {
        let (hdr, records) = read_all_records("htslib/test/realn03_exp.sam");
        assert_eq!(records.len(), 2);
        assert_eq!(sam_hdr_tid2len(hdr, 0), 11);
        assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 0)), c"MX");

        for (rec, qname) in records.iter().zip(["M", "X"]) {
            assert_eq!(
                CStr::from_ptr(bam_get_qname(*rec)).to_bytes(),
                qname.as_bytes()
            );
            assert_eq!((**rec).core.tid, 0);
            assert_eq!((**rec).core.pos, 0);
            assert_eq!((**rec).core.qual, 60);
            assert_eq!((**rec).core.flag, 64);
            assert_eq!((**rec).core.l_qseq, 11);
            assert_eq!(aux_z(*rec, c"BQ".as_ptr()), "D@@@@@@@@@D");
        }

        for rec in records {
            bam_destroy1(rec);
        }
        sam_hdr_destroy(hdr);
    }
}

#[test]
fn realn01_baq_default_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn01.fa",
            "htslib/test/realn01.fa.fai",
            c"000000F",
            686,
            "htslib/test/realn01.sam",
            "htslib/test/realn01_exp.sam",
            0,
        );
    }
}

#[test]
fn realn01_baq_apply_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn01.fa",
            "htslib/test/realn01.fa.fai",
            c"000000F",
            686,
            "htslib/test/realn01.sam",
            "htslib/test/realn01_exp-a.sam",
            1,
        );
    }
}

#[test]
fn realn01_baq_extend_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn01.fa",
            "htslib/test/realn01.fa.fai",
            c"000000F",
            686,
            "htslib/test/realn01.sam",
            "htslib/test/realn01_exp-e.sam",
            2,
        );
    }
}

#[test]
fn realn02_baq_apply_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            c"17",
            4200,
            "htslib/test/realn02.sam",
            "htslib/test/realn02_exp-a.sam",
            1,
        );
    }
}

#[test]
fn realn02_baq_revert_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            c"17",
            4200,
            "htslib/test/realn02_exp-a.sam",
            "htslib/test/realn02_exp.sam",
            0,
        );
    }
}

#[test]
fn realn02_baq_redo_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn02.fa",
            "htslib/test/realn02.fa.fai",
            c"17",
            4200,
            "htslib/test/realn02-r.sam",
            "htslib/test/realn02_exp.sam",
            4,
        );
    }
}

#[test]
fn realn03_baq_extend_matches_upstream_expected_fixture() {
    unsafe {
        assert_realigned_records_match_expected(
            "htslib/test/realn03.fa",
            "htslib/test/realn03.fa.fai",
            c"MX",
            11,
            "htslib/test/realn03.sam",
            "htslib/test/realn03_exp.sam",
            2,
        );
    }
}
