use std::ffi::{c_char, c_int};

use crate::htslib_mini_rs::{hts, sam};

// original: code (htslib/test/test_mod.c:76)
pub unsafe fn test_test_mod_c_76_code(id: c_int) -> *mut c_char {
    static mut CODE: [c_char; 20] = [0; 20];
    let code = std::ptr::addr_of_mut!(CODE).cast::<c_char>();
    if id > 0 {
        *code = id as c_char;
        *code.add(1) = 0;
    } else {
        libc::snprintf(code, 20, c"(%d)".as_ptr(), -id);
    }
    code
}

// original: main (htslib/test/test_mod.c:88)
pub unsafe fn test_test_mod_c_88_main(mut argc: c_int, mut argv: *mut *mut c_char) -> c_int {
    const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let mut extended = 0;
    let mut flags = 0_u32;

    if argc > 1 && libc::strcmp(*argv.add(1), c"-x".as_ptr()) == 0 {
        extended = 1;
        argv = argv.add(1);
        argc -= 1;
    }

    if argc > 2 && libc::strcmp(*argv.add(1), c"-f".as_ptr()) == 0 {
        flags = libc::atoi(*argv.add(2)) as u32;
        argv = argv.add(2);
        argc -= 2;
    }

    if argc < 2 {
        return 1;
    }

    let in_ = hts::hts_open(*argv.add(1), c"r".as_ptr());
    if in_.is_null() {
        return 1;
    }

    let b = sam::bam_init1();
    let h = sam::sam_hdr_read(in_);
    let m = sam::hts_base_mod_state_alloc();
    if h.is_null() || b.is_null() || m.is_null() {
        sam::bam_destroy1(b);
        sam::sam_hdr_destroy(h);
        sam::hts_base_mod_state_free(m);
        return if hts::hts_close(in_) != 0 { 1 } else { 2 };
    }

    let mut r;
    loop {
        r = sam::sam_read1(in_, h, b);
        if r < 0 {
            break;
        }

        if sam::bam_parse_basemod2(b, m, flags) < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"Failed to parse MM/ML aux tags\n".as_ptr(),
            );
            sam::bam_destroy1(b);
            sam::sam_hdr_destroy(h);
            sam::hts_base_mod_state_free(m);
            return if hts::hts_close(in_) != 0 { 1 } else { 2 };
        }

        let mut mods = [sam::hts_base_mod {
            modified_base: 0,
            canonical_base: 0,
            strand: 0,
            qual: 0,
        }; 5];
        let mut i = 0;
        while i < (*b).core.l_qseq {
            let mut sp = b'\t' as c_char;
            let n = sam::bam_mods_at_next_pos(b, m, mods.as_mut_ptr(), 5);
            libc::printf(
                c"%d\t%c".as_ptr(),
                i,
                SEQ_NT16_STR[sam::bam_seqi(sam::bam_get_seq(b), i as usize) as usize] as c_int,
            );
            let mut j = 0;
            while j < n && j < 5 {
                let mut qstr = [0 as c_char; 10];
                if mods[j as usize].qual == sam::HTS_MOD_UNCHECKED {
                    qstr[0] = b'#' as c_char;
                    qstr[1] = 0;
                } else if mods[j as usize].qual == sam::HTS_MOD_UNKNOWN {
                    qstr[0] = b'.' as c_char;
                    qstr[1] = 0;
                } else {
                    libc::snprintf(qstr.as_mut_ptr(), 10, c"%d".as_ptr(), mods[j as usize].qual);
                }

                if extended != 0 {
                    let mut m_strand = 0;
                    let mut m_implicit = 0;
                    let mut m_canonical = 0;
                    let ret = sam::bam_mods_query_type(
                        m,
                        mods[j as usize].modified_base,
                        &mut m_strand,
                        &mut m_implicit,
                        &mut m_canonical,
                    );
                    if ret < 0
                        || m_canonical as c_int != mods[j as usize].canonical_base
                        || m_strand != mods[j as usize].strand
                    {
                        sam::bam_destroy1(b);
                        sam::sam_hdr_destroy(h);
                        sam::hts_base_mod_state_free(m);
                        return if hts::hts_close(in_) != 0 { 1 } else { 2 };
                    }
                    libc::printf(
                        c"%c%c%c%s%c%s".as_ptr(),
                        sp as c_int,
                        mods[j as usize].canonical_base,
                        *b"+-".as_ptr().add(mods[j as usize].strand as usize) as c_int,
                        test_test_mod_c_76_code(mods[j as usize].modified_base),
                        *b"?.".as_ptr().add(m_implicit as usize) as c_int,
                        qstr.as_ptr(),
                    );
                } else {
                    libc::printf(
                        c"%c%c%c%s%s".as_ptr(),
                        sp as c_int,
                        mods[j as usize].canonical_base,
                        *b"+-".as_ptr().add(mods[j as usize].strand as usize) as c_int,
                        test_test_mod_c_76_code(mods[j as usize].modified_base),
                        qstr.as_ptr(),
                    );
                }
                sp = b' ' as c_char;
                j += 1;
            }
            libc::putchar(b'\n' as c_int);
            i += 1;
        }

        libc::puts(c"---".as_ptr());

        sam::bam_parse_basemod2(b, m, flags);

        let mut all_mods_n = 0;
        let all_mods = sam::bam_mods_recorded(m, &mut all_mods_n);
        libc::printf(c"Present:".as_ptr());
        i = 0;
        while i < all_mods_n {
            let mut m_strand = 0;
            let mut m_implicit = 0;
            let mut m_canonical = 0;
            sam::bam_mods_queryi(m, i, &mut m_strand, &mut m_implicit, &mut m_canonical);
            libc::printf(
                if *all_mods.add(i as usize) > 0 {
                    c" %c".as_ptr()
                } else {
                    c" #%d".as_ptr()
                },
                *all_mods.add(i as usize),
            );
            libc::putchar(*b"?.".as_ptr().add(m_implicit as usize) as c_int);
            i += 1;
        }
        libc::putchar(b'\n' as c_int);

        let mut pos = 0;
        loop {
            let n = sam::bam_next_basemod(b, m, mods.as_mut_ptr(), 5, &mut pos);
            if n <= 0 {
                if n < 0 {
                    sam::bam_destroy1(b);
                    sam::sam_hdr_destroy(h);
                    sam::hts_base_mod_state_free(m);
                    return if hts::hts_close(in_) != 0 { 1 } else { 2 };
                }
                break;
            }
            let mut sp = b'\t' as c_char;
            libc::printf(
                c"%d\t%c".as_ptr(),
                pos,
                SEQ_NT16_STR[sam::bam_seqi(sam::bam_get_seq(b), pos as usize) as usize] as c_int,
            );
            let mut j = 0;
            while j < n && j < 5 {
                let mut qstr = [0 as c_char; 10];
                if mods[j as usize].qual == sam::HTS_MOD_UNCHECKED {
                    qstr[0] = b'#' as c_char;
                    qstr[1] = 0;
                } else if mods[j as usize].qual == sam::HTS_MOD_UNKNOWN {
                    qstr[0] = b'.' as c_char;
                    qstr[1] = 0;
                } else {
                    libc::snprintf(qstr.as_mut_ptr(), 10, c"%d".as_ptr(), mods[j as usize].qual);
                }

                libc::printf(
                    c"%c%c%c%s%s".as_ptr(),
                    sp as c_int,
                    mods[j as usize].canonical_base,
                    *b"+-".as_ptr().add(mods[j as usize].strand as usize) as c_int,
                    test_test_mod_c_76_code(mods[j as usize].modified_base),
                    qstr.as_ptr(),
                );
                sp = b' ' as c_char;
                j += 1;
            }
            libc::putchar(b'\n' as c_int);
        }

        libc::puts(c"\n===\n".as_ptr());
    }

    libc::fflush(hts_sys::stdout.cast());
    let mut ret = 0;
    if hts::hts_close(in_) != 0 || r < -1 {
        ret = 1;
    }

    sam::bam_destroy1(b);
    sam::sam_hdr_destroy(h);
    sam::hts_base_mod_state_free(m);
    ret
}
