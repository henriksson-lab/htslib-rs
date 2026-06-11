use crate::htslib_rs::{
    hts::{htsFile, hts_close, hts_open, kputc, kstring_t},
    sam,
};

// original: plp_dat (htslib/test/pileup_mod.c:32)
#[repr(C)]
pub struct plp_dat {
    pub fp: *mut htsFile,
    pub h: *mut sam::sam_hdr_t,
}

// original: readaln (htslib/test/pileup_mod.c:37)
pub unsafe extern "C" fn test_pileup_mod_c_37_readaln(
    data: *mut (),
    b: *mut sam::bam1_t,
) -> i32 {
    let dat = data.cast::<plp_dat>();
    sam::sam_read1((*dat).fp, (*dat).h, b)
}

// No modification reporting.
// This is just a simple base-line for comparison against mod_pileup1 for
// performance testing.
// original: process_pileup (htslib/test/pileup_mod.c:49)
pub unsafe fn test_pileup_mod_c_49_process_pileup(
    h: *mut sam::sam_hdr_t,
    mut p: *const sam::bam_pileup1_t,
    tid: i32,
    pos: i32,
    n: i32,
) {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut s = kstring_t::default();

    let name = sam::sam_hdr_tid2name(&*h, tid);
    print!(
        "{}\t{}\t",
        String::from_utf8_lossy(std::ffi::CStr::from_ptr(name.cast::<i8>()).to_bytes()),
        pos
    );
    for _ in 0..n {
        if sam::bam_pileup1_is_del(p) != 0 {
            print!("*");
            p = p.add(1);
            continue;
        }

        let seq = sam::bam_get_seq((*p).b);
        let qual = sam::bam_get_qual((*p).b);
        let c = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize];
        print!("{}", c as char);
        kputc(
            std::cmp::min(
                b'~' as i32,
                b'!' as i32 + *qual.add((*p).qpos as usize) as i32,
            ),
            &mut s,
        );
        p = p.add(1);
    }
    print!("\t");
    println!("{}", String::from_utf8_lossy(&s.data));
}

// Initialise and destroy the base modifier state data. This is called
// as each new read is added or removed from the pileups.
// original: pileup_cd_create (htslib/test/pileup_mod.c:74)
pub unsafe extern "C" fn test_pileup_mod_c_74_pileup_cd_create(
    _data: *mut (),
    b: *const sam::bam1_t,
    cd: *mut sam::bam_pileup_cd,
) -> i32 {
    let mut m = sam::hts_base_mod_state_alloc();
    if sam::bam_parse_basemod(&*b, &mut m) < 0 {
        sam::hts_base_mod_state_free(Some(m));
        return -1;
    }
    (*cd).p = Box::into_raw(m).cast();
    0
}

// original: pileup_cd_destroy (htslib/test/pileup_mod.c:84)
pub unsafe extern "C" fn test_pileup_mod_c_84_pileup_cd_destroy(
    _data: *mut (),
    _b: *const sam::bam1_t,
    cd: *mut sam::bam_pileup_cd,
) -> i32 {
    sam::hts_base_mod_state_free(Some(Box::from_raw(
        (*cd).p.cast::<sam::hts_base_mod_state>(),
    )));
    0
}

// Report a line of pileup, including base modifications inline with
// the sequence (including qualities), as [<strand><dir><qual>...]
// original: process_mod_pileup1 (htslib/test/pileup_mod.c:91)
pub unsafe fn test_pileup_mod_c_91_process_mod_pileup1(
    h: *mut sam::sam_hdr_t,
    mut p: *const sam::bam_pileup1_t,
    tid: i32,
    pos: i32,
    n: i32,
) {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut s = kstring_t::default();

    let name = sam::sam_hdr_tid2name(&*h, tid);
    print!(
        "{}\t{}\t",
        String::from_utf8_lossy(std::ffi::CStr::from_ptr(name.cast::<i8>()).to_bytes()),
        pos
    );
    for _ in 0..n {
        if sam::bam_pileup1_is_del(p) != 0 {
            print!("*");
            p = p.add(1);
            continue;
        }

        let seq = sam::bam_get_seq((*p).b);
        let qual = sam::bam_get_qual((*p).b);
        let c = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize];
        print!("{}", c as char);
        kputc(
            std::cmp::min(
                b'~' as i32,
                b'!' as i32 + *qual.add((*p).qpos as usize) as i32,
            ),
            &mut s,
        );

        // Simple mod detection; assumes at most 5 mods
        let m = &mut *(*p).cd.p.cast::<sam::hts_base_mod_state>();
        let mut mods = [sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 5];
        let nm = sam::bam_mods_at_qpos(&*(*p).b, (*p).qpos, m, &mut mods);
        if nm > 0 {
            print!("[");
            for j in 0..nm {
                if j >= 5 {
                    break;
                }
                let mod_ = mods[j as usize];
                if mod_.modified_base < 0 {
                    // ChEBI
                    print!(
                        "{}({}){}",
                        b"+-"[mod_.strand as usize] as char,
                        -mod_.modified_base,
                        mod_.qual,
                    );
                } else {
                    print!(
                        "{}{}{}",
                        b"+-"[mod_.strand as usize] as char,
                        mod_.modified_base as u8 as char,
                        mod_.qual,
                    );
                }
            }
            print!("]");
        }
        p = p.add(1);
    }
    print!("\t");
    println!("{}", String::from_utf8_lossy(&s.data));
}

// Report a line of pileup, including base modifications.
// This replaces the base with the mod call (NB this can be confusing
// as both C and G can map to m depending on orientation).
// It also reports qualities in the QUAl column, remapped to
// phred scale as only one single mod is supported and hence extreme
// unlikely probabilities shouldn't be reported (although we don't
// scan to pick the highest).
// original: process_mod_pileup2 (htslib/test/pileup_mod.c:140)
pub unsafe fn test_pileup_mod_c_140_process_mod_pileup2(
    h: *mut sam::sam_hdr_t,
    mut p: *const sam::bam_pileup1_t,
    tid: i32,
    pos: i32,
    n: i32,
) {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut s = kstring_t::default();

    let name = sam::sam_hdr_tid2name(&*h, tid);
    print!(
        "{}\t{}\t{}\t",
        String::from_utf8_lossy(std::ffi::CStr::from_ptr(name.cast::<i8>()).to_bytes()),
        pos,
        n,
    );
    for _ in 0..n {
        if sam::bam_pileup1_is_del(p) != 0 {
            print!("*");
            p = p.add(1);
            continue;
        }

        let seq = sam::bam_get_seq((*p).b);
        let qual = sam::bam_get_qual((*p).b);
        let c = SEQ_NT16_STR[sam::bam_seqi(seq, (*p).qpos as usize) as usize];

        // Simple mod detection; assumes at most 2 non-ChEBI mods
        let m = &mut *(*p).cd.p.cast::<sam::hts_base_mod_state>();
        let is_rev = sam::bam_is_rev((*p).b);
        let mut mod_ = sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        };
        let mut q = *qual.add((*p).qpos as usize);
        let base = if sam::bam_mods_at_qpos(&*(*p).b, (*p).qpos, m, std::slice::from_mut(&mut mod_))
            > 0
        {
            // base mod as phred scale
            q = (-10.0 * (1.0 - ((mod_.qual as f64 + 0.5) / 256.0)).log10() + 0.5) as u8;
            mod_.modified_base
        } else {
            c as i32
        };

        // Case is inappropriate here as some mods (eg "a") are lc.
        // So we dim/bold them instead using ANSI escape codes.
        // It's a test script, so I'm not going to care about curses.
        if is_rev {
            print!("\x1b[2m{}\x1b[0m", base as u8 as char);
        } else {
            print!("\x1b[1m{}\x1b[0m", base as u8 as char);
        }
        kputc(std::cmp::min(b'~' as i32, b'!' as i32 + q as i32), &mut s);
        p = p.add(1);
    }
    print!("\t");
    println!("{}", String::from_utf8_lossy(&s.data));
}

// original: main (htslib/test/pileup_mod.c:185)
pub unsafe fn test_pileup_mod_c_185_main(argv: &[&[u8]]) -> i32 {
    let mut compact = 0;
    let mut idx = 1;
    while idx < argv.len() && argv[idx] == b"-c" {
        compact += 1;
        idx += 1;
    }

    let mut in_path: Vec<u8> = if idx < argv.len() {
        argv[idx].to_vec()
    } else {
        b"-".to_vec()
    };
    in_path.push(0);
    let in_ = hts_open(in_path.as_ptr().cast(), c"r".as_ptr());
    let b = sam::bam_init1();
    let h = sam::sam_hdr_read(in_);

    // Pileup iterator with constructor/destructor to parse base mod tags
    let mut dat = plp_dat { fp: in_, h };
    let iter = sam::bam_plp_init(
        Some(test_pileup_mod_c_37_readaln),
        (&mut dat as *mut plp_dat).cast(),
    );
    sam::bam_plp_constructor(iter, Some(test_pileup_mod_c_74_pileup_cd_create));
    sam::bam_plp_destructor(iter, Some(test_pileup_mod_c_84_pileup_cd_destroy));

    let mut tid = 0;
    let mut pos = 0;
    let mut n = 0;
    loop {
        let p = sam::bam_plp_auto(iter, &mut tid, &mut pos, &mut n);
        if p.is_null() {
            break;
        }
        match compact {
            0 => test_pileup_mod_c_91_process_mod_pileup1(h, p, tid, pos, n),
            1 => test_pileup_mod_c_140_process_mod_pileup2(h, p, tid, pos, n),
            _ => test_pileup_mod_c_49_process_pileup(h, p, tid, pos, n),
        }
    }
    sam::bam_plp_destroy(iter);

    hts_close(in_);
    sam::bam_destroy1(b);
    sam::sam_hdr_destroy(h);

    (n != 0) as i32
}
