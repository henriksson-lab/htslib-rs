//! Real-data integration tests for the htslib/sam_mods.c API surface
//! (translated into src/sam_mods.rs). Exercises the base-modification
//! state lifecycle (hts_base_mod_state_alloc/free), parser
//! (bam_parse_basemod/2), per-record metadata (bam_mods_recorded,
//! bam_mods_query_type, bam_mods_queryi), and the position-driven
//! iterators (bam_next_basemod, bam_mods_at_next_pos, bam_mods_at_qpos)
//! against the base_mods/MM-*.sam fixtures shipped with htslib.
//!
//! Companion to tests/real_data_original_basemod_more.rs (which mostly
//! validates the `bam_next_basemod` output stream) and
//! tests/real_data_base_mod_output_more.rs (which validates the
//! formatted base_mod fixture output). This file targets the
//! state-querying helpers more directly.

use htslib_rs::{
    bam1_t, bam_destroy1, bam_get_seq, bam_init1, bam_mods_at_next_pos, bam_mods_at_qpos,
    bam_mods_query_type, bam_mods_queryi, bam_mods_recorded, bam_next_basemod, bam_parse_basemod,
    bam_parse_basemod2, bam_seqi, hts_base_mod, hts_base_mod_state, hts_base_mod_state_alloc,
    hts_base_mod_state_free, hts_close, hts_open, sam_hdr_destroy, sam_hdr_read, sam_read1,
};
use std::ffi::CString;
use std::os::raw::c_int;

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

unsafe fn destroy_records(records: Vec<*mut bam1_t>) {
    for rec in records {
        bam_destroy1(rec);
    }
}

fn base_label(code: u8) -> char {
    match code {
        1 => 'A',
        2 => 'C',
        4 => 'G',
        8 => 'T',
        15 => 'N',
        _ => '?',
    }
}

unsafe fn base_at(rec: *const bam1_t, qpos: i32) -> char {
    base_label(bam_seqi(bam_get_seq(rec), qpos as usize))
}

/// Collect the (sorted) set of recorded mod type codes via `bam_mods_recorded`.
unsafe fn recorded_types(state: &mut hts_base_mod_state) -> Vec<i32> {
    let mut ntype: c_int = 0;
    let types = bam_mods_recorded(state, &mut ntype);
    let mut out: Vec<i32> = (0..ntype as usize).map(|i| types[i]).collect();
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// State alloc / free lifecycle on real records
// ---------------------------------------------------------------------------

#[test]
fn state_alloc_and_free_round_trip_for_each_record_in_multi_fixture() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-multi.sam");
        assert_eq!(records.len(), 2);
        for &rec in &records {
            let mut state = hts_base_mod_state_alloc();
            assert_eq!(bam_parse_basemod(&*rec, &mut *state), 0);
            // Free without iterating must not leak or crash.
            hts_base_mod_state_free(Some(state));
        }
        destroy_records(records);
    }
}

// ---------------------------------------------------------------------------
// bam_mods_recorded reports the exact set of mod types present
// ---------------------------------------------------------------------------

#[test]
fn mods_recorded_reports_three_distinct_mod_types_for_mm_multi_r1() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-multi.sam");
        // r1 has separate C+m, C+h, N+n entries.
        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*records[0], &mut *state), 0);

        let codes = recorded_types(&mut *state);
        // 'h' = 104, 'm' = 109, 'n' = 110 (sorted ascending).
        assert_eq!(codes, vec![b'h' as i32, b'm' as i32, b'n' as i32]);

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

#[test]
fn mods_recorded_reports_chebi_numeric_code_as_negative_int() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-chebi.sam");
        assert_eq!(records.len(), 1);
        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*records[0], &mut *state), 0);

        let codes = recorded_types(&mut *state);
        // Three mod entries: C+m (109), C+76792 (CHEBI numeric, encoded as
        // -76792), N+n (110). After sort the negative number is first.
        assert_eq!(codes.len(), 3);
        assert_eq!(
            codes[0], -76792,
            "ChEBI numeric code should be -76792 (negative)"
        );
        assert!(codes.contains(&(b'm' as i32)));
        assert!(codes.contains(&(b'n' as i32)));

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

#[test]
fn mods_recorded_is_zero_when_record_has_no_mm_tag() {
    unsafe {
        // The "not-all-modded" fixture has records 1 and 3 without MM tags.
        let records = read_records("htslib/test/base_mods/MM-not-all-modded.sam");
        assert_eq!(records.len(), 4);

        let mut state = hts_base_mod_state_alloc();
        // Records 1 and 3 (0-indexed) have no MM/ML tag => parse returns 0
        // and recorded mods are empty.
        for idx in [1usize, 3usize] {
            assert_eq!(bam_parse_basemod(&*records[idx], &mut *state), 0);
            let mut ntype: c_int = 0;
            let types = bam_mods_recorded(&mut *state, &mut ntype);
            assert_eq!(ntype, 0, "record {idx} should have no recorded mods");
            // `types` is the inline state buffer slice;
            // we just don't read past `ntype` elements.
            let _ = types;
        }

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

// ---------------------------------------------------------------------------
// bam_mods_query_type and bam_mods_queryi return the same (strand, implicit,
// canonical) metadata for each recorded mod
// ---------------------------------------------------------------------------

#[test]
fn mods_query_type_and_queryi_agree_on_orient_top_and_bottom_strands() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-orient.sam");
        assert_eq!(records.len(), 4);

        // top-fwd: C+m
        {
            let mut state = hts_base_mod_state_alloc();
            assert_eq!(bam_parse_basemod(&*records[0], &mut *state), 0);
            let mut strand_a = 0;
            let mut implicit_a = 0;
            let mut canonical_a: i8 = 0;
            assert_eq!(
                bam_mods_query_type(
                    &*state,
                    b'm' as c_int,
                    Some(&mut strand_a),
                    Some(&mut implicit_a),
                    Some(&mut canonical_a),
                ),
                0
            );
            let mut strand_b = 0;
            let mut implicit_b = 0;
            let mut canonical_b: i8 = 0;
            assert_eq!(
                bam_mods_queryi(
                    &*state,
                    0,
                    Some(&mut strand_b),
                    Some(&mut implicit_b),
                    Some(&mut canonical_b),
                ),
                0
            );
            assert_eq!(strand_a, strand_b);
            assert_eq!(implicit_a, implicit_b);
            assert_eq!(canonical_a, canonical_b);
            // top strand: strand == 0 (positive), canonical == 'C'.
            assert_eq!(strand_a, 0);
            assert_eq!(canonical_a as u8 as char, 'C');
            hts_base_mod_state_free(Some(state));
        }

        // bot-fwd: G-m  -> strand should be 1 (negative), canonical 'G'.
        {
            let mut state = hts_base_mod_state_alloc();
            assert_eq!(bam_parse_basemod(&*records[2], &mut *state), 0);
            let mut strand = 0;
            let mut implicit = 0;
            let mut canonical: i8 = 0;
            assert_eq!(
                bam_mods_query_type(
                    &*state,
                    b'm' as c_int,
                    Some(&mut strand),
                    Some(&mut implicit),
                    Some(&mut canonical),
                ),
                0
            );
            assert_eq!(strand, 1, "G-m must be reported on the negative strand");
            assert_eq!(canonical as u8 as char, 'G');
            hts_base_mod_state_free(Some(state));
        }
    }
}

#[test]
fn mods_query_type_returns_minus_one_for_unrecorded_code() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-double.sam");
        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*records[0], &mut *state), 0);

        // 'x' is never recorded in MM-double; expect -1.
        let mut strand = 0;
        let mut implicit = 0;
        let mut canonical: i8 = 0;
        assert_eq!(
            bam_mods_query_type(
                &*state,
                b'x' as c_int,
                Some(&mut strand),
                Some(&mut implicit),
                Some(&mut canonical),
            ),
            -1
        );

        // queryi out-of-range => -1 too.
        assert_eq!(
            bam_mods_queryi(
                &*state,
                99,
                Some(&mut strand),
                Some(&mut implicit),
                Some(&mut canonical),
            ),
            -1
        );

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

#[test]
fn mods_query_type_reports_implicit_default_for_chebi_fixture() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-chebi.sam");
        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*records[0], &mut *state), 0);

        // 'm' has no explicit '?' marker => implicit == 1 (default-zero).
        // ChEBI numeric code -76792 same.
        for &code in &[b'm' as c_int, b'n' as c_int, -76792i32] {
            let mut strand = 0;
            let mut implicit: c_int = -1;
            let mut canonical: i8 = 0;
            assert_eq!(
                bam_mods_query_type(
                    &*state,
                    code,
                    Some(&mut strand),
                    Some(&mut implicit),
                    Some(&mut canonical),
                ),
                0,
                "mod code {code} should be present"
            );
            assert_eq!(
                implicit, 1,
                "MM-chebi.sam uses implicit (default-zero) mode for code {code}"
            );
        }

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

// ---------------------------------------------------------------------------
// bam_mods_at_next_pos walks every qpos and reports the mod set for that base
// ---------------------------------------------------------------------------

#[test]
fn mods_at_next_pos_reports_mods_at_exact_positions_in_mm_double() {
    unsafe {
        // MM-double r1: Mm:Z:C+m,1,3,0;G-m,0,2,0,4;G+o,4;
        // Per the existing real_data_original_basemod_more.rs validation,
        // the position-driven base-mod sites are at qpos 1, 7, 12, 13, 22,
        // 30, 31. Verify by stepping through every qpos with
        // bam_mods_at_next_pos.
        let records = read_records("htslib/test/base_mods/MM-double.sam");
        assert_eq!(records.len(), 1);
        let rec = records[0];

        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*rec, &mut *state), 0);

        let mut mods_per_pos = Vec::new();
        for pos in 0..(*rec).core.l_qseq {
            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }; 8];
            let n = bam_mods_at_next_pos(&*rec, &mut *state, &mut mods);
            assert!(n >= 0, "bam_mods_at_next_pos failed at qpos {pos}: {n}");
            if n > 0 {
                let mut codes: Vec<i32> = (0..n as usize).map(|i| mods[i].modified_base).collect();
                codes.sort();
                mods_per_pos.push((pos, base_at(rec, pos), codes));
            }
        }
        // Modded positions (matches what the bam_next_basemod-based test
        // observes).
        let observed_positions: Vec<i32> = mods_per_pos.iter().map(|(p, _, _)| *p).collect();
        assert_eq!(observed_positions, vec![1, 7, 12, 13, 22, 30, 31]);

        // qpos 13 sees two modifications (G-m and G+o).
        let (_p, _b, ref codes_at_13) = mods_per_pos.iter().find(|(p, _, _)| *p == 13).unwrap();
        assert_eq!(codes_at_13.len(), 2);
        assert!(codes_at_13.contains(&(b'm' as i32)));
        assert!(codes_at_13.contains(&(b'o' as i32)));

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

// ---------------------------------------------------------------------------
// bam_mods_at_qpos jumps directly to a given query position
// ---------------------------------------------------------------------------

#[test]
fn mods_at_qpos_seeks_directly_to_modded_position_in_mm_double() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-double.sam");
        let rec = records[0];

        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*rec, &mut *state), 0);

        // Jump to qpos 30 (a known modded C+m site) without iterating from 0.
        let mut mods = [hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 4];
        let n = bam_mods_at_qpos(&*rec, 30, &mut *state, &mut mods);
        assert_eq!(n, 1, "qpos 30 should have one mod");
        assert_eq!(mods[0].modified_base, b'm' as i32);
        assert_eq!(mods[0].canonical_base as u8 as char, 'C');
        assert_eq!(mods[0].strand, 0);
        assert_eq!(mods[0].qual, 153);

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

#[test]
fn mods_at_qpos_reports_no_mod_for_intermediate_unmodded_position() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-double.sam");
        let rec = records[0];

        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*rec, &mut *state), 0);

        // qpos 0 (a G, but the MM tag's first G-m site is qpos 1, not 0).
        let mut mods = [hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 4];
        let n = bam_mods_at_qpos(&*rec, 0, &mut *state, &mut mods);
        assert_eq!(n, 0, "qpos 0 should have no mod");

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

// ---------------------------------------------------------------------------
// bam_next_basemod returns positions ascending; verify on multi-mod fixture
// that the iterator agrees with the at_next_pos walk above
// ---------------------------------------------------------------------------

#[test]
fn next_basemod_returns_ascending_positions_for_mm_multi_r1() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-multi.sam");
        let rec = records[0]; // r1

        let mut state = hts_base_mod_state_alloc();
        assert_eq!(bam_parse_basemod(&*rec, &mut *state), 0);

        let mut seen = Vec::new();
        loop {
            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }; 8];
            let mut pos = -1;
            let n = bam_next_basemod(&*rec, &mut *state, &mut mods, &mut pos);
            if n <= 0 {
                break;
            }
            seen.push(pos);
        }

        // Per real_data_original_basemod_more.rs: positions are 6, 15, 17,
        // 18, 19, 20, 31, 34 for r1.
        assert_eq!(seen, vec![6, 15, 17, 18, 19, 20, 31, 34]);
        // Strictly ascending.
        for w in seen.windows(2) {
            assert!(w[0] < w[1]);
        }

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}

// ---------------------------------------------------------------------------
// bam_parse_basemod2 with HTS_MOD_REPORT_UNCHECKED flag emits unchecked
// markers; verify the marker shows up as HTS_MOD_UNCHECKED qual
// ---------------------------------------------------------------------------

#[test]
fn parse_basemod2_with_report_unchecked_flag_emits_hash_marker_for_variants_fixture() {
    unsafe {
        let records = read_records("htslib/test/base_mods/MM-variants.sam");
        // record[6] is the unchecked variant (per existing test).
        let rec = records[6];

        let mut state = hts_base_mod_state_alloc();
        // flag=1 == HTS_MOD_REPORT_UNCHECKED
        assert_eq!(bam_parse_basemod2(&*rec, &mut *state, 1), 0);

        // Walk and verify at least one mod has qual == HTS_MOD_UNCHECKED.
        let mut saw_unchecked = false;
        loop {
            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }; 8];
            let mut pos = -1;
            let n = bam_next_basemod(&*rec, &mut *state, &mut mods, &mut pos);
            if n <= 0 {
                break;
            }
            for m in &mods[..n as usize] {
                if m.qual == htslib_rs::HTS_MOD_UNCHECKED {
                    saw_unchecked = true;
                }
            }
        }
        assert!(
            saw_unchecked,
            "HTS_MOD_REPORT_UNCHECKED should surface at least one '#' qual"
        );

        hts_base_mod_state_free(Some(state));
        destroy_records(records);
    }
}
