//! Real-data integration tests for the htslib/header.c API surface (translated
//! into src/header.rs). Exercises the public sam_hdr_* mutation/query
//! functions on real BAM/SAM fixtures from third_party/hts-sys/htslib/test.
//!
//! Companion to tests/real_data_header_refs_more.rs (which focuses on
//! tid<->name/length lookup) and tests/real_data_original_header_more.rs
//! (which focuses on existing find_line/find_tag against known fixtures).
//! This file adds coverage for the header-mutation entry points
//! (add_lines/add_line/update_line/remove_line_id/remove_line_pos/
//! remove_tag_id/add_pg/pg_id), and exercises count_lines/find_tag_id/
//! find_tag_pos/find_line_id/line_index/line_name/name2tid/tid2name on
//! real on-disk headers.

use htslib_rs::{
    htsFile, hts_close, hts_open, ks_free, kstring_t, sam_hdr_add_line, sam_hdr_add_lines,
    sam_hdr_add_pg, sam_hdr_count_lines, sam_hdr_destroy, sam_hdr_find_line_id,
    sam_hdr_find_line_pos, sam_hdr_find_tag_id, sam_hdr_find_tag_pos, sam_hdr_line_index,
    sam_hdr_line_name, sam_hdr_name2tid, sam_hdr_nref, sam_hdr_pg_id, sam_hdr_read,
    sam_hdr_remove_line_id, sam_hdr_remove_line_pos, sam_hdr_remove_tag_id, sam_hdr_str, sam_hdr_t,
    sam_hdr_tid2len, sam_hdr_tid2name, sam_hdr_update_line,
};

fn c_fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut bytes = path.to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

unsafe fn open_header(path: &str) -> (*mut htsFile, *mut sam_hdr_t) {
    let path_c = c_fixture(path);
    let fp = hts_open(path_c.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(!fp.is_null(), "failed to open {path}");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null(), "failed to read header from {path}");
    (fp, hdr)
}

unsafe fn close_header(fp: *mut htsFile, hdr: *mut sam_hdr_t) {
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
}

unsafe fn kstring_bytes(ks: &kstring_t) -> Vec<u8> {
    ks.data.clone()
}

unsafe fn header_text(hdr: *mut sam_hdr_t) -> Vec<u8> {
    let text = sam_hdr_str(&mut *hdr);
    assert!(!text.is_null());
    let mut len = 0usize;
    while *text.add(len) != 0 {
        len += 1;
    }
    std::slice::from_raw_parts(text, len).to_vec()
}

// ---------------------------------------------------------------------------
// sam_hdr_count_lines
// ---------------------------------------------------------------------------

#[test]
fn count_lines_returns_known_section_counts_for_xx_rg_sam_fixture() {
    unsafe {
        // xx#rg.sam has: 1 HD, 1 SQ, 2 RG, 1 PG, 2 CO lines.
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"HD"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"SQ"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 2);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"PG"), 1);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"CO"), 2);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"ZZ"), 0);
        close_header(fp, hdr);
    }
}

#[test]
fn count_lines_returns_sq_count_for_real_bam_header() {
    unsafe {
        // range.bam has 7 @SQ lines (CHROMOSOME_I..V, X, MtDNA).
        let (fp, hdr) = open_header("htslib/test/range.bam");
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"SQ"), 7);
        assert_eq!(sam_hdr_nref(&*hdr), 7);
        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_find_tag_id / sam_hdr_find_tag_pos
// ---------------------------------------------------------------------------

#[test]
fn find_tag_id_returns_specific_sq_and_rg_fields_for_xx_rg() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        let mut ks: kstring_t = kstring_t::default();
        // @SQ SN:xx LN:20 AS:? SP:? UR:? M5:bbf4...
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"SQ",
                Some((b"SN", b"xx")),
                b"LN",
                &mut ks,
            ),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"20");

        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"SQ",
                Some((b"SN", b"xx")),
                b"M5",
                &mut ks,
            ),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"bbf4de6d8497a119dda6e074521643dc");

        // @RG ID:x2 ... LB:x PI:1111
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"LB",
                &mut ks,
            ),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"x");
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"PI",
                &mut ks,
            ),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"1111");

        // Unknown tag => not found.
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"ZZ",
                &mut ks,
            ),
            -1
        );

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

#[test]
fn find_tag_pos_resolves_rg_by_zero_based_index() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        let mut ks: kstring_t = kstring_t::default();
        // RG[0] => ID:x1
        assert_eq!(
            sam_hdr_find_tag_pos(&mut *hdr, b"RG", 0, b"ID", &mut ks),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"x1");
        // RG[1] => ID:x2
        assert_eq!(
            sam_hdr_find_tag_pos(&mut *hdr, b"RG", 1, b"ID", &mut ks),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"x2");
        // PG[0] => PN:emacs
        assert_eq!(
            sam_hdr_find_tag_pos(&mut *hdr, b"PG", 0, b"PN", &mut ks),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"emacs");
        // Out-of-range index => -1.
        assert_eq!(
            sam_hdr_find_tag_pos(&mut *hdr, b"RG", 99, b"ID", &mut ks),
            -1
        );

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_find_line_id / sam_hdr_find_line_pos
// ---------------------------------------------------------------------------

#[test]
fn find_line_id_returns_full_rg_line_text() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        let mut ks: kstring_t = kstring_t::default();
        assert_eq!(
            sam_hdr_find_line_id(&mut *hdr, b"RG", b"ID", b"x2", &mut ks,),
            0
        );
        let line = kstring_bytes(&ks);
        // Must start with the section type, contain the ID, and the auxiliary tags.
        assert!(line.starts_with(b"@RG"), "got: {:?}", line);
        assert!(line.windows(5).any(|w| w == b"ID:x2"));
        assert!(line.windows(4).any(|w| w == b"LB:x"));
        assert!(line.windows(7).any(|w| w == b"PI:1111"));

        // Find by non-existent ID => -1.
        assert_eq!(
            sam_hdr_find_line_id(
                &mut *hdr,
                b"RG",
                b"ID",
                b"nope",
                &mut ks,
            ),
            -1
        );

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

#[test]
fn find_line_pos_resolves_hd_and_co_lines() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        let mut ks: kstring_t = kstring_t::default();
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"HD", 0, &mut ks), 0);
        let hd_line = kstring_bytes(&ks);
        assert!(hd_line.starts_with(b"@HD"));
        assert!(hd_line.windows(7).any(|w| w == b"VN:1.4\t") || hd_line.ends_with(b"VN:1.4"));

        // CO lines preserve raw text.
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"CO", 0, &mut ks), 0);
        assert_eq!(kstring_bytes(&ks), b"@CO\talso test");
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"CO", 1, &mut ks), 0);
        // The second CO contains an embedded tab; verify both halves are kept.
        let co1 = kstring_bytes(&ks);
        assert!(co1.starts_with(b"@CO\t"));
        assert!(co1.windows(5).any(|w| w == b"other"));
        assert!(co1.windows(7).any(|w| w == b"headers"));

        // Out-of-range => -1.
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"CO", 5, &mut ks), -1);

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_line_index / sam_hdr_line_name
// ---------------------------------------------------------------------------

#[test]
fn line_index_round_trips_with_line_name_for_rg() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        // Forward: name -> index.
        assert_eq!(sam_hdr_line_index(&mut *hdr, b"RG", b"x1"), 0);
        assert_eq!(sam_hdr_line_index(&mut *hdr, b"RG", b"x2"), 1);
        // Missing => -1.
        assert_eq!(
            sam_hdr_line_index(&mut *hdr, b"RG", b"missing"),
            -1
        );

        // Reverse: index -> name.
        let name0 = sam_hdr_line_name(&mut *hdr, b"RG", 0);
        assert!(!name0.is_null());
        assert_eq!(std::slice::from_raw_parts(name0, 2), b"x1");
        let name1 = sam_hdr_line_name(&mut *hdr, b"RG", 1);
        assert!(!name1.is_null());
        assert_eq!(std::slice::from_raw_parts(name1, 2), b"x2");

        close_header(fp, hdr);
    }
}

#[test]
fn line_name_for_sq_resolves_via_target_array_on_real_bam() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/range.bam");

        // SQ section uses the target_name array (no hrecs needed).
        let n0 = sam_hdr_line_name(&mut *hdr, b"SQ", 0);
        assert!(!n0.is_null());
        assert_eq!(std::slice::from_raw_parts(n0, b"CHROMOSOME_I".len()), b"CHROMOSOME_I");
        let n6 = sam_hdr_line_name(&mut *hdr, b"SQ", 6);
        assert!(!n6.is_null());
        assert_eq!(
            std::slice::from_raw_parts(n6, b"CHROMOSOME_MtDNA".len()),
            b"CHROMOSOME_MtDNA"
        );

        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_name2tid / sam_hdr_tid2name / sam_hdr_tid2len
// ---------------------------------------------------------------------------

#[test]
fn name2tid_and_tid2name_round_trip_on_real_bam_header() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/range.bam");

        let expected: &[(&[u8], i32, i64)] = &[
            (b"CHROMOSOME_I", 0, 1_009_800),
            (b"CHROMOSOME_II", 1, 5_000),
            (b"CHROMOSOME_III", 2, 5_000),
            (b"CHROMOSOME_IV", 3, 5_000),
            (b"CHROMOSOME_V", 4, 5_000),
            (b"CHROMOSOME_X", 5, 5_000),
            (b"CHROMOSOME_MtDNA", 6, 5_000),
        ];
        for &(name, tid, len) in expected {
            assert_eq!(sam_hdr_name2tid(&mut *hdr, name), tid);
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_tid2name(&*hdr, tid), name.len()),
                name
            );
            assert_eq!(sam_hdr_tid2len(&*hdr, tid), len);
        }
        // Missing name => -1; out-of-range tid => null/0.
        assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chrZ"), -1);
        assert!(sam_hdr_tid2name(&*hdr, 99).is_null());
        assert_eq!(sam_hdr_tid2len(&*hdr, 99), 0);

        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_add_line / sam_hdr_add_lines (mutating @CO comment)
// ---------------------------------------------------------------------------

#[test]
fn add_line_appends_co_comment_and_count_lines_reflects_it() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"CO"), 2);

        // CO lines take a single tag whose key is the comment text and whose
        // value pointer must be null.
        assert_eq!(
            sam_hdr_add_line(&mut *hdr, b"CO", &[(Some(b"Added by integration test"), None)],),
            0
        );
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"CO"), 3);

        // The new CO is the third one; verify its text.
        let mut ks: kstring_t = kstring_t::default();
        assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"CO", 2, &mut ks), 0);
        assert_eq!(kstring_bytes(&ks), b"@CO\tAdded by integration test");

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

#[test]
fn add_lines_parses_multiple_co_records_and_count_lines_increments() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");
        let before = sam_hdr_count_lines(&mut *hdr, b"CO");

        let bytes = b"@CO\textra one\n@CO\textra two\n";
        assert_eq!(sam_hdr_add_lines(&mut *hdr, bytes), 0);
        let after = sam_hdr_count_lines(&mut *hdr, b"CO");
        assert_eq!(after, before + 2);

        close_header(fp, hdr);
    }
}

#[test]
fn add_line_rejects_invalid_inputs() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        // CO with extra tags => -1.
        assert_eq!(
            sam_hdr_add_line(&mut *hdr, b"CO", &[(Some(b"text"), Some(b"v"))],),
            -1
        );

        // Bad type (XX) => -1.
        assert_eq!(
            sam_hdr_add_line(&mut *hdr, b"XX", &[(Some(b"KK"), Some(b"v"))],),
            -1
        );

        // SQ with no tags => -1.
        assert_eq!(sam_hdr_add_line(&mut *hdr, b"SQ", &[]), -1);

        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_update_line (modify an existing @PG line)
// ---------------------------------------------------------------------------

#[test]
fn update_line_modifies_pg_tag_value_on_real_header() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        // @PG ID:emacs PN:emacs VN:23.1.1 -> change VN to 99.0.
        assert_eq!(
            sam_hdr_update_line(
                &mut *hdr,
                b"PG",
                Some((b"ID", b"emacs")),
                &[(Some(b"VN"), Some(b"99.0"))],
            ),
            0
        );

        let mut ks: kstring_t = kstring_t::default();
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"PG",
                Some((b"ID", b"emacs")),
                b"VN",
                &mut ks,
            ),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"99.0");

        // PG ID changes are rejected (matches htslib).
        assert_eq!(
            sam_hdr_update_line(
                &mut *hdr,
                b"PG",
                Some((b"ID", b"emacs")),
                &[(Some(b"ID"), Some(b"changed"))],
            ),
            -1
        );

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_remove_line_id / sam_hdr_remove_line_pos / sam_hdr_remove_tag_id
// ---------------------------------------------------------------------------

#[test]
fn remove_line_id_drops_rg_and_count_decreases() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 2);

        assert_eq!(
            sam_hdr_remove_line_id(&mut *hdr, b"RG", Some((b"ID", b"x2"))),
            0
        );
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 1);

        // The remaining RG should be x1.
        let name = sam_hdr_line_name(&mut *hdr, b"RG", 0);
        assert!(!name.is_null());
        assert_eq!(std::slice::from_raw_parts(name, 2), b"x1");
        assert_eq!(sam_hdr_line_index(&mut *hdr, b"RG", b"x2"), -1);

        close_header(fp, hdr);
    }
}

#[test]
fn remove_line_pos_drops_sq_and_target_array_updates() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/range.bam");
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"SQ"), 7);
        assert_eq!(sam_hdr_nref(&*hdr), 7);

        // Remove the last @SQ (CHROMOSOME_MtDNA, position 6).
        assert_eq!(sam_hdr_remove_line_pos(&mut *hdr, b"SQ", 6), 0);
        assert_eq!(sam_hdr_count_lines(&mut *hdr, b"SQ"), 6);
        assert_eq!(sam_hdr_nref(&*hdr), 6);
        assert_eq!(sam_hdr_name2tid(&mut *hdr, b"CHROMOSOME_MtDNA"), -1);

        // Names that survive should still resolve.
        assert_eq!(sam_hdr_name2tid(&mut *hdr, b"CHROMOSOME_I"), 0);

        // Removing @PG by position is disallowed.
        assert_eq!(sam_hdr_remove_line_pos(&mut *hdr, b"PG", 0), -1);

        close_header(fp, hdr);
    }
}

#[test]
fn remove_tag_id_drops_specific_tag_from_rg_line() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        // Confirm the PI tag exists on RG ID:x2.
        let mut ks: kstring_t = kstring_t::default();
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"PI",
                &mut ks,
            ),
            0
        );
        assert_eq!(kstring_bytes(&ks), b"1111");

        // Remove it.
        assert_eq!(
            sam_hdr_remove_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"PI",
            ),
            0
        );
        assert_eq!(
            sam_hdr_find_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"PI",
                &mut ks,
            ),
            -1
        );
        // Removing a non-existent tag => -1 (per the htslib normalisation).
        assert_eq!(
            sam_hdr_remove_tag_id(
                &mut *hdr,
                b"RG",
                Some((b"ID", b"x2")),
                b"ZZ",
            ),
            -1
        );

        ks_free(&mut ks);
        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// sam_hdr_add_pg / sam_hdr_pg_id
// ---------------------------------------------------------------------------

#[test]
fn add_pg_appends_new_pg_and_pg_id_uniquifies_clashing_name() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");
        let before = sam_hdr_count_lines(&mut *hdr, b"PG");
        assert_eq!(before, 1);

        // sam_hdr_pg_id with a fresh name returns the same name pointer (no clash).
        let fresh = sam_hdr_pg_id(&mut *hdr, b"fresh");
        assert!(!fresh.is_null());
        assert_eq!(std::slice::from_raw_parts(fresh, b"fresh".len()), b"fresh");

        // Append a new @PG.
        assert_eq!(
            sam_hdr_add_pg(
                &mut *hdr,
                b"myprog",
                &[(Some(b"VN"), Some(b"1.0"))],
            ),
            0
        );
        let after = sam_hdr_count_lines(&mut *hdr, b"PG");
        assert_eq!(after, before + 1);

        // Now `emacs` is already taken; pg_id should return a uniquified copy
        // (e.g. "emacs.1") rather than the literal input.
        let unique = sam_hdr_pg_id(&mut *hdr, b"emacs");
        assert!(!unique.is_null());
        let mut unique_len = 0usize;
        while *unique.add(unique_len) != 0 {
            unique_len += 1;
        }
        let unique_bytes = std::slice::from_raw_parts(unique, unique_len);
        assert_ne!(unique_bytes, b"emacs");
        assert!(
            unique_bytes.starts_with(b"emacs."),
            "got: {:?}",
            unique_bytes
        );

        close_header(fp, hdr);
    }
}

// ---------------------------------------------------------------------------
// Combined: header_text reflects mutations
// ---------------------------------------------------------------------------

#[test]
fn header_text_reflects_added_co_and_removed_rg() {
    unsafe {
        let (fp, hdr) = open_header("htslib/test/xx#rg.sam");

        // Mutate: add a CO and remove an RG.
        assert_eq!(
            sam_hdr_add_line(&mut *hdr, b"CO", &[(Some(b"tag-from-test"), None)],),
            0
        );
        assert_eq!(
            sam_hdr_remove_line_id(&mut *hdr, b"RG", Some((b"ID", b"x1"))),
            0
        );

        let text = header_text(hdr);
        let text_str = String::from_utf8_lossy(&text);
        // The added comment must show up...
        assert!(
            text_str.contains("tag-from-test"),
            "missing added CO in:\n{text_str}"
        );
        // ...and the removed RG must be gone, while x2 stays.
        assert!(!text_str.contains("ID:x1\n") && !text_str.contains("ID:x1\t"));
        assert!(text_str.contains("ID:x2"));

        close_header(fp, hdr);
    }
}
