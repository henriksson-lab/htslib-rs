use htslib_rs::{
    bam1_t, bam_destroy1, bam_get_seq, bam_init1, bam_next_basemod, bam_parse_basemod,
    bam_parse_basemod2, bam_seqi, hts_base_mod, hts_base_mod_state_alloc, hts_base_mod_state_free,
    hts_close, hts_open, sam_hdr_destroy, sam_hdr_read, sam_read1, HTS_MOD_UNCHECKED,
    HTS_MOD_UNKNOWN,
};
use std::ffi::CString;

fn c_fixture(path: &str) -> CString {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    CString::new(path.to_string_lossy().as_bytes()).unwrap()
}

unsafe fn read_records(path: &str) -> Vec<*mut bam1_t> {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr(), c"r".as_ptr());
    assert!(!fp.is_null(), "failed to open {}", path.to_string_lossy());
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());

    let mut out = Vec::new();
    loop {
        let rec = bam_init1();
        assert!(!rec.is_null());
        let ret = sam_read1(fp, hdr, rec);
        if ret < 0 {
            bam_destroy1(rec);
            break;
        }
        out.push(rec);
    }

    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

unsafe fn base_at(rec: *const bam1_t, qpos: i32) -> char {
    match bam_seqi(bam_get_seq(rec), qpos as usize) {
        1 => 'A',
        2 => 'C',
        4 => 'G',
        8 => 'T',
        15 => 'N',
        other => panic!("unexpected encoded base {other}"),
    }
}

fn qual_label(qual: i32) -> String {
    match qual {
        HTS_MOD_UNCHECKED => "#".to_string(),
        HTS_MOD_UNKNOWN => ".".to_string(),
        _ => qual.to_string(),
    }
}

unsafe fn collect_base_mod_calls(rec: *const bam1_t) -> Vec<(i32, char, Vec<String>)> {
    collect_base_mod_calls_with_flags(rec, 0)
}

unsafe fn collect_base_mod_calls_with_flags(
    rec: *const bam1_t,
    flags: u32,
) -> Vec<(i32, char, Vec<String>)> {
    let state = hts_base_mod_state_alloc();
    assert!(!state.is_null());
    let parse_ret = if flags == 0 {
        bam_parse_basemod(rec, state)
    } else {
        bam_parse_basemod2(rec, state, flags)
    };
    assert_eq!(parse_ret, 0);

    let mut calls = Vec::new();
    loop {
        let mut mods = [hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 8];
        let mut pos = -1;
        let n = bam_next_basemod(rec, state, mods.as_mut_ptr(), mods.len() as i32, &mut pos);
        if n <= 0 {
            break;
        }

        let rendered = mods[..n as usize]
            .iter()
            .map(|m| {
                let sign = if m.strand == 0 { '+' } else { '-' };
                let modified = if (0..=255).contains(&m.modified_base) {
                    (m.modified_base as u8 as char).to_string()
                } else {
                    format!("({})", m.modified_base)
                };
                format!(
                    "{}{}{}{}",
                    m.canonical_base as u8 as char,
                    sign,
                    modified,
                    qual_label(m.qual)
                )
            })
            .collect();
        calls.push((pos, base_at(rec, pos), rendered));
    }

    hts_base_mod_state_free(state);
    calls
}

unsafe fn parse_base_mod_ret(rec: *const bam1_t) -> i32 {
    let state = hts_base_mod_state_alloc();
    assert!(!state.is_null());
    let ret = bam_parse_basemod(rec, state);
    hts_base_mod_state_free(state);
    ret
}

unsafe fn destroy_records(records: Vec<*mut bam1_t>) {
    for rec in records {
        bam_destroy1(rec);
    }
}

#[test]
fn original_base_mod_explicit_fixture_matches_expected_mod_sites() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-explicit.sam");
        assert_eq!(records.len(), 3);

        assert_eq!(
            collect_base_mod_calls(records[0]),
            vec![
                (9, 'C', vec!["C+m200".to_string(), "C+h10".to_string()]),
                (10, 'C', vec!["C+m50".to_string(), "C+h170".to_string()]),
                (14, 'C', vec!["C+m160".to_string(), "C+h20".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls(records[1]),
            vec![
                (9, 'C', vec!["C+m200".to_string(), "C+h10".to_string()]),
                (10, 'C', vec!["C+m50".to_string(), "C+h170".to_string()]),
                (13, 'C', vec!["C+m10".to_string(), "C+h5".to_string()]),
                (14, 'C', vec!["C+m160".to_string(), "C+h20".to_string()]),
                (16, 'C', vec!["C+m10".to_string(), "C+h5".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls(records[2]),
            vec![
                (9, 'C', vec!["C+m200".to_string(), "C+h10".to_string()]),
                (10, 'C', vec!["C+h170".to_string()]),
                (13, 'C', vec!["C+h5".to_string()]),
                (14, 'C', vec!["C+m160".to_string(), "C+h20".to_string()]),
                (16, 'C', vec!["C+h5".to_string()]),
            ]
        );

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_multi_fixture_matches_expected_mixed_mod_sites() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-multi.sam");
        assert_eq!(records.len(), 2);

        assert_eq!(
            collect_base_mod_calls(records[0]),
            vec![
                (6, 'C', vec!["C+m128".to_string()]),
                (15, 'N', vec!["N+n215".to_string()]),
                (17, 'C', vec!["C+m153".to_string()]),
                (18, 'G', vec!["N+n240".to_string()]),
                (19, 'C', vec!["C+h159".to_string()]),
                (20, 'C', vec!["C+m179".to_string()]),
                (31, 'C', vec!["C+m204".to_string()]),
                (34, 'C', vec!["C+m230".to_string(), "C+h6".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls(records[1]),
            vec![
                (6, 'C', vec!["C+m77".to_string(), "C+h159".to_string()]),
                (15, 'N', vec!["N+n240".to_string()]),
                (17, 'C', vec!["C+m103".to_string(), "C+h133".to_string()]),
                (19, 'C', vec!["C+m128".to_string(), "C+h108".to_string()]),
                (20, 'C', vec!["C+m154".to_string(), "C+h82".to_string()]),
                (31, 'C', vec!["C+m179".to_string(), "C+h57".to_string()]),
                (34, 'C', vec!["C+m204".to_string(), "C+h31".to_string()]),
            ]
        );

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_double_fixture_matches_expected_mod_sites() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-double.sam");
        assert_eq!(records.len(), 1);

        assert_eq!(
            collect_base_mod_calls(records[0]),
            vec![
                (1, 'G', vec!["G-m115".to_string()]),
                (7, 'C', vec!["C+m128".to_string()]),
                (12, 'G', vec!["G-m141".to_string()]),
                (13, 'G', vec!["G-m166".to_string(), "G+o102".to_string()]),
                (22, 'G', vec!["G-m192".to_string()]),
                (30, 'C', vec!["C+m153".to_string()]),
                (31, 'C', vec!["C+m179".to_string()]),
            ]
        );

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_not_all_modded_fixture_matches_expected_present_and_absent_records() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-not-all-modded.sam");
        assert_eq!(records.len(), 4);

        let expected_modded = vec![
            (6, 'C', vec!["C+m128".to_string()]),
            (15, 'N', vec!["N+n215".to_string()]),
            (17, 'C', vec!["C+m153".to_string()]),
            (18, 'G', vec!["N+n240".to_string()]),
            (19, 'C', vec!["C+h159".to_string()]),
            (20, 'C', vec!["C+m179".to_string()]),
            (31, 'C', vec!["C+m204".to_string()]),
            (34, 'C', vec!["C+m230".to_string(), "C+h6".to_string()]),
        ];

        assert_eq!(collect_base_mod_calls(records[0]), expected_modded);
        assert_eq!(collect_base_mod_calls(records[1]), Vec::new());
        assert_eq!(collect_base_mod_calls(records[2]), expected_modded);
        assert_eq!(collect_base_mod_calls(records[3]), Vec::new());

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_variants_fixture_matches_unknown_and_unchecked_outputs() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-variants.sam");
        assert_eq!(records.len(), 8);

        assert_eq!(
            collect_base_mod_calls_with_flags(records[0], 1),
            vec![
                (2, 'G', vec!["G-a.".to_string()]),
                (3, 'C', vec!["C+m.".to_string()]),
                (6, 'C', vec!["C+m.".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls_with_flags(records[1], 1),
            vec![
                (2, 'G', vec!["C+m.".to_string()]),
                (3, 'C', vec!["G-a.".to_string()]),
                (6, 'G', vec!["C+m.".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls_with_flags(records[2], 1),
            vec![
                (2, 'G', vec!["G-a.".to_string()]),
                (3, 'C', vec!["C+m.".to_string()]),
                (6, 'C', vec!["C+m.".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls_with_flags(records[3], 1),
            vec![
                (2, 'G', vec!["C+m.".to_string()]),
                (3, 'C', vec!["G-a.".to_string()]),
                (6, 'G', vec!["C+m.".to_string()]),
            ]
        );
        assert_eq!(collect_base_mod_calls_with_flags(records[4], 1), Vec::new());
        assert_eq!(collect_base_mod_calls_with_flags(records[5], 1), Vec::new());
        assert_eq!(
            collect_base_mod_calls_with_flags(records[6], 1),
            vec![
                (2, 'G', vec!["G-a#".to_string()]),
                (3, 'C', vec!["C+m#".to_string()]),
                (6, 'C', vec!["C+m#".to_string()]),
            ]
        );
        assert_eq!(
            collect_base_mod_calls_with_flags(records[7], 1),
            vec![
                (2, 'G', vec!["C+m#".to_string()]),
                (3, 'C', vec!["G-a#".to_string()]),
                (6, 'G', vec!["C+m#".to_string()]),
            ]
        );

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_invalid_mn_length_fixture_is_rejected() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-MNf1.sam");
        assert_eq!(records.len(), 4);
        assert_ne!(parse_base_mod_ret(records[0]), 0);

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_invalid_mn_type_fixture_is_rejected() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-MNf2.sam");
        assert_eq!(records.len(), 4);
        assert_ne!(parse_base_mod_ret(records[3]), 0);

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_forward_bounds_fixture_is_rejected() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-bounds+.sam");
        assert_eq!(records.len(), 1);

        let state = hts_base_mod_state_alloc();
        assert!(!state.is_null());
        assert_eq!(bam_parse_basemod(records[0], state), 0);

        let mut mods = [hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 8];
        let mut pos = -1;
        let mut ret;
        loop {
            ret = bam_next_basemod(
                records[0],
                state,
                mods.as_mut_ptr(),
                mods.len() as i32,
                &mut pos,
            );
            if ret <= 0 {
                break;
            }
        }
        assert_eq!(ret, -1);
        hts_base_mod_state_free(state);

        destroy_records(records);
    }
}

#[test]
fn original_base_mod_reverse_bounds_fixture_is_rejected() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-bounds-.sam");
        assert_eq!(records.len(), 1);
        assert_ne!(parse_base_mod_ret(records[0]), 0);

        destroy_records(records);
    }
}
