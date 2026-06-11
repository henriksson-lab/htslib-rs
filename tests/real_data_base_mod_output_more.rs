use htslib_rs::{
    bam1_t, bam_destroy1, bam_get_qual, bam_get_seq, bam_init1, bam_mods_at_next_pos,
    bam_mods_at_qpos, bam_mods_query_type, bam_mods_queryi, bam_mods_recorded, bam_next_basemod,
    bam_parse_basemod, bam_parse_basemod2, bam_pileup1_is_del, bam_pileup1_t, bam_plp_auto,
    bam_plp_constructor, bam_plp_destroy, bam_plp_destructor, bam_plp_init, bam_seqi, htsFile,
    hts_base_mod, hts_base_mod_state_alloc, hts_base_mod_state_free, hts_close, hts_open,
    sam_hdr_destroy, sam_hdr_read, sam_hdr_t, sam_hdr_tid2name, sam_read1, HTS_MOD_UNCHECKED,
    HTS_MOD_UNKNOWN,
};
fn fixture_path(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn c_fixture(path: &str) -> Vec<u8> {
    let mut bytes = fixture_path(path).to_string_lossy().into_owned().into_bytes();
    bytes.push(0);
    bytes
}

fn base_label(code: u8) -> char {
    match code {
        0 => '=',
        1 => 'A',
        2 => 'C',
        3 => 'M',
        4 => 'G',
        5 => 'R',
        6 => 'S',
        7 => 'V',
        8 => 'T',
        9 => 'W',
        10 => 'Y',
        11 => 'H',
        12 => 'K',
        13 => 'D',
        14 => 'B',
        15 => 'N',
        other => panic!("unexpected encoded base {other}"),
    }
}

unsafe fn record_base(rec: *const bam1_t, pos: i32) -> char {
    base_label(bam_seqi(bam_get_seq(rec), pos as usize))
}

fn mod_code(code: i32) -> String {
    if code > 0 {
        (code as u8 as char).to_string()
    } else {
        format!("({})", -code)
    }
}

fn qual_label(qual: i32) -> String {
    match qual {
        HTS_MOD_UNCHECKED => "#".to_string(),
        HTS_MOD_UNKNOWN => ".".to_string(),
        _ => qual.to_string(),
    }
}

fn rendered_mod(m: &hts_base_mod) -> String {
    let strand = if m.strand == 0 { '+' } else { '-' };
    format!(
        "{}{}{}{}",
        m.canonical_base as u8 as char,
        strand,
        mod_code(m.modified_base),
        qual_label(m.qual)
    )
}

fn rendered_extended_mod(state: &htslib_rs::hts_base_mod_state, m: &hts_base_mod) -> String {
    unsafe {
        let mut strand: i32 = 0;
        let mut implicit = 0;
        let mut canonical: u8 = 0;
        assert_eq!(
            bam_mods_query_type(
                state,
                m.modified_base,
                Some(&mut strand),
                Some(&mut implicit),
                Some(&mut canonical),
            ),
            0
        );
        assert_eq!(canonical, m.canonical_base as u8);
        assert_eq!(strand, m.strand as i32);
        let implicit_marker = if implicit == 0 { '?' } else { '.' };
        let strand = if m.strand == 0 { '+' } else { '-' };
        format!(
            "{}{}{}{}{}",
            m.canonical_base as u8 as char,
            strand,
            mod_code(m.modified_base),
            implicit_marker,
            qual_label(m.qual)
        )
    }
}

unsafe fn append_mod_line(
    out: &mut String,
    pos: i32,
    base: char,
    mods: &[hts_base_mod],
    state: &htslib_rs::hts_base_mod_state,
    extended: bool,
) {
    out.push_str(&format!("{pos}\t{base}"));
    let mut sep = '\t';
    for m in mods {
        let text = if extended {
            rendered_extended_mod(state, m)
        } else {
            rendered_mod(m)
        };
        out.push(sep);
        out.push_str(&text);
        sep = ' ';
    }
    out.push('\n');
}

unsafe fn append_present_line(out: &mut String, state: &mut htslib_rs::hts_base_mod_state) {
    let mut ntype = 0;
    let types: Vec<i32> = bam_mods_recorded(state, &mut ntype).to_vec();
    out.push_str("Present:");
    for i in 0..ntype {
        let mut strand: i32 = 0;
        let mut implicit = 0;
        let mut canonical: u8 = 0;
        assert_eq!(
            bam_mods_queryi(
                state,
                i,
                Some(&mut strand),
                Some(&mut implicit),
                Some(&mut canonical)
            ),
            0
        );
        let code = types[i as usize];
        if code > 0 {
            out.push_str(&format!(" {}", code as u8 as char));
        } else {
            out.push_str(&format!(" #{}", code));
        }
        out.push(if implicit == 0 { '?' } else { '.' });
    }
    out.push('\n');
}

unsafe fn render_test_mod_fixture(path: &str, extended: bool, flags: u32) -> String {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(!fp.is_null(), "failed to open fixture");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let rec = bam_init1();
    assert!(!rec.is_null());
    let mut state = hts_base_mod_state_alloc();

    let mut out = String::new();
    loop {
        let read_ret = sam_read1(fp, hdr, rec);
        if read_ret < 0 {
            break;
        }

        assert_eq!(bam_parse_basemod2(&*rec, &mut *state, flags), 0);
        for pos in 0..(*rec).core.l_qseq {
            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }; 5];
            let n = bam_mods_at_next_pos(&*rec, &mut *state, &mut mods);
            assert!(n >= 0);
            append_mod_line(
                &mut out,
                pos,
                record_base(rec, pos),
                &mods[..std::cmp::min(n as usize, mods.len())],
                &*state,
                extended,
            );
        }

        out.push_str("---\n");
        assert_eq!(bam_parse_basemod2(&*rec, &mut *state, flags), 0);
        append_present_line(&mut out, &mut *state);

        loop {
            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }; 5];
            let mut pos = -1;
            let n = bam_next_basemod(&*rec, &mut *state, &mut mods, &mut pos);
            assert!(n >= 0);
            if n == 0 {
                break;
            }
            append_mod_line(
                &mut out,
                pos,
                record_base(rec, pos),
                &mods[..std::cmp::min(n as usize, mods.len())],
                &*state,
                false,
            );
        }

        out.push_str("\n===\n\n");
    }

    hts_base_mod_state_free(Some(state));
    bam_destroy1(rec);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

fn assert_test_mod_output(sam: &str, expected: &str, extended: bool, flags: u32) {
    let actual = unsafe { render_test_mod_fixture(sam, extended, flags) };
    let expected = std::fs::read_to_string(fixture_path(expected)).unwrap();
    assert_eq!(actual, expected);
}

struct SamReader {
    fp: *mut htsFile,
    hdr: *mut sam_hdr_t,
}

unsafe extern "C" fn read_record(data: *mut (), rec: *mut bam1_t) -> i32 {
    let reader = &mut *(data as *mut SamReader);
    sam_read1(reader.fp, reader.hdr, rec)
}

unsafe extern "C" fn pileup_cd_create(
    _data: *mut (),
    rec: *const bam1_t,
    cd: *mut htslib_rs::bam_pileup_cd,
) -> i32 {
    let mut state = hts_base_mod_state_alloc();
    if bam_parse_basemod(&*rec, &mut *state) < 0 {
        hts_base_mod_state_free(Some(state));
        return -1;
    }
    (*cd).p = Box::into_raw(state) as *mut ();
    0
}

unsafe extern "C" fn pileup_cd_destroy(
    _data: *mut (),
    _rec: *const bam1_t,
    cd: *mut htslib_rs::bam_pileup_cd,
) -> i32 {
    hts_base_mod_state_free(Some(Box::from_raw(
        (*cd).p as *mut htslib_rs::hts_base_mod_state,
    )));
    0
}

unsafe fn append_pileup_mod_line(
    out: &mut String,
    hdr: *mut sam_hdr_t,
    pileup: *const bam_pileup1_t,
    tid: i32,
    pos: i32,
    depth: i32,
) {
    let name_ptr = sam_hdr_tid2name(&*hdr, tid);
    let name_len = (0..).take_while(|&i| *name_ptr.add(i) != 0).count();
    let rname = std::str::from_utf8(std::slice::from_raw_parts(name_ptr.cast(), name_len)).unwrap();
    out.push_str(&format!("{rname}\t{pos}\t"));
    let mut quals = String::new();
    for i in 0..depth {
        let p = pileup.add(i as usize);
        if bam_pileup1_is_del(p) != 0 {
            out.push('*');
            continue;
        }

        let rec = (*p).b;
        let qpos = (*p).qpos;
        out.push(record_base(rec, qpos));
        let qual = *bam_get_qual(rec).add(qpos as usize);
        quals.push(std::cmp::min(b'~', b'!'.saturating_add(qual)) as char);

        let state = &mut *((*p).cd.p as *mut htslib_rs::hts_base_mod_state);
        let mut mods = [hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 5];
        let n = bam_mods_at_qpos(&*rec, qpos, state, &mut mods);
        assert!(n >= 0);
        if n > 0 {
            out.push('[');
            for m in &mods[..std::cmp::min(n as usize, mods.len())] {
                let strand = if m.strand == 0 { '+' } else { '-' };
                out.push_str(&format!(
                    "{}{}{}",
                    strand,
                    mod_code(m.modified_base),
                    m.qual
                ));
            }
            out.push(']');
        }
    }
    out.push('\t');
    out.push_str(&quals);
    out.push('\n');
}

unsafe fn render_pileup_mod_fixture(path: &str) -> String {
    let path = c_fixture(path);
    let fp = hts_open(path.as_ptr().cast(), b"r\0".as_ptr().cast());
    assert!(!fp.is_null(), "failed to open fixture");
    let hdr = sam_hdr_read(fp);
    assert!(!hdr.is_null());
    let mut reader = SamReader { fp, hdr };
    let iter = bam_plp_init(Some(read_record), (&mut reader as *mut SamReader).cast());
    assert!(!iter.is_null());
    bam_plp_constructor(iter, Some(pileup_cd_create));
    bam_plp_destructor(iter, Some(pileup_cd_destroy));

    let mut out = String::new();
    loop {
        let mut tid = 0;
        let mut pos = 0;
        let mut depth = 0;
        let pileup = bam_plp_auto(iter, &mut tid, &mut pos, &mut depth);
        if pileup.is_null() {
            break;
        }
        append_pileup_mod_line(&mut out, hdr, pileup, tid, pos, depth);
    }

    bam_plp_destroy(iter);
    sam_hdr_destroy(hdr);
    assert_eq!(hts_close(fp), 0);
    out
}

fn assert_pileup_mod_output(sam: &str, expected: &str) {
    let actual = unsafe { render_pileup_mod_fixture(sam) };
    let expected = std::fs::read_to_string(fixture_path(expected)).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn real_base_mod_test_mod_outputs_match_htslib_fixtures_exactly() {
    assert_test_mod_output(
        "htslib/test/base_mods/MM-chebi.sam",
        "htslib/test/base_mods/MM-chebi.out",
        false,
        0,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-double.sam",
        "htslib/test/base_mods/MM-double.out",
        false,
        0,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-multi.sam",
        "htslib/test/base_mods/MM-multi.out",
        false,
        0,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-explicit.sam",
        "htslib/test/base_mods/MM-explicit.out",
        false,
        0,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-explicit.sam",
        "htslib/test/base_mods/MM-explicit-x.out",
        true,
        0,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-explicit.sam",
        "htslib/test/base_mods/MM-explicit-f.out",
        false,
        1,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-not-all-modded.sam",
        "htslib/test/base_mods/MM-not-all-modded.out",
        false,
        0,
    );
    assert_test_mod_output(
        "htslib/test/base_mods/MM-variants.sam",
        "htslib/test/base_mods/MM-variants.out",
        false,
        1,
    );
}

#[test]
fn real_base_mod_pileup_mod_outputs_match_htslib_fixtures_exactly() {
    assert_pileup_mod_output(
        "htslib/test/base_mods/MM-pileup.sam",
        "htslib/test/base_mods/MM-pileup.out",
    );
    assert_pileup_mod_output(
        "htslib/test/base_mods/MM-pileup2.sam",
        "htslib/test/base_mods/MM-pileup2.out",
    );
    assert_pileup_mod_output(
        "htslib/test/base_mods/MM-MNp.sam",
        "htslib/test/base_mods/MM-pileup.out",
    );
}
