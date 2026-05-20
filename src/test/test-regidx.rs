use crate::htslib_rs::{
    hts::{hts_pos_t, isspace_c, kputsn, kstring_t},
    regidx::{
        regidx_c_198_regidx_insert, regidx_c_246_regidx_init, regidx_c_311_regidx_destroy,
        regidx_c_401_regidx_overlap, regidx_c_466_regidx_parse_bed, regidx_c_498_regidx_parse_tab,
        regidx_c_545_regidx_parse_reg, regidx_c_584_regitr_init, regidx_c_606_regitr_destroy,
        regidx_c_612_regitr_overlap, regidx_c_646_regitr_loop, regidx_parse_f,
    },
};
use std::{ffi::c_char, os::raw::c_int, ptr};

unsafe extern "C" {
    static mut optarg: *mut c_char;
}

static mut VERBOSE: c_int = 0;

// original: debug (htslib/test/test-regidx.c:45)
pub unsafe fn test_test_regidx_c_45_debug() {}

// original: info (htslib/test/test-regidx.c:55)
pub unsafe fn test_test_regidx_c_55_info() {}

// original: HTS_FORMAT (htslib/test/test-regidx.c:64)
pub unsafe fn test_test_regidx_c_64_HTS_FORMAT() {}

// original: custom_parse (htslib/test/test-regidx.c:75)
pub unsafe extern "C" fn test_test_regidx_c_75_custom_parse(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    payload: *mut std::ffi::c_void,
    _usr: *mut std::ffi::c_void,
) -> c_int {
    let ret = regidx_c_498_regidx_parse_tab(
        line,
        chr_beg,
        chr_end,
        beg,
        end,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    if ret != 0 {
        return ret;
    }

    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    for _i in 0..3 {
        while *ss != 0 && isspace_c(*ss) == 0 {
            ss = ss.add(1);
        }
        if *ss == 0 {
            return -2;
        }
        while *ss != 0 && isspace_c(*ss) != 0 {
            ss = ss.add(1);
        }
    }
    if *ss == 0 {
        return -2;
    }

    let mut se = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }
    let dat = payload.cast::<*mut c_char>();
    *dat = libc::malloc(se.offset_from(ss) as usize + 1).cast::<c_char>();
    libc::memcpy((*dat).cast(), ss.cast(), se.offset_from(ss) as usize + 1);
    *(*dat).add(se.offset_from(ss) as usize) = 0;
    0
}

// original: custom_free (htslib/test/test-regidx.c:100)
pub unsafe extern "C" fn test_test_regidx_c_100_custom_free(payload: *mut std::ffi::c_void) {
    let dat = payload.cast::<*mut c_char>();
    libc::free((*dat).cast());
}

// original: test_sequential_access (htslib/test/test-regidx.c:106)
pub unsafe fn test_test_regidx_c_106_test_sequential_access() {
    let idx = regidx_c_246_regidx_init(
        ptr::null(),
        Some(test_test_regidx_c_75_custom_parse),
        Some(test_test_regidx_c_100_custom_free),
        std::mem::size_of::<*mut c_char>(),
        ptr::null_mut(),
    );
    if idx.is_null() {
        libc::fprintf(hts_sys::stderr.cast(), c"init failed\n".as_ptr());
        libc::exit(-1);
    }

    let mut str_: kstring_t = std::mem::zeroed();
    let n = 10;
    for i in 0..n {
        let beg = 10 * (i + 1);
        str_.l = 0;
        let mut buf = [0 as c_char; 128];
        let len = libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"1\t%d\t%d\t%d".as_ptr(),
            beg,
            beg,
            beg,
        );
        kputsn(buf.as_ptr(), len as usize, &mut str_);
        if regidx_c_198_regidx_insert(idx, str_.s) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert failed: %s\n".as_ptr(),
                str_.s,
            );
            libc::exit(-1);
        }
    }

    let itr = regidx_c_584_regitr_init(idx);
    let mut i = 0;
    while regidx_c_646_regitr_loop(itr) != 0 {
        if (*itr).beg != (*itr).end || (*itr).beg + 1 != 10 * (i + 1) as hts_pos_t {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"listing failed, expected %d, found %ld\n".as_ptr(),
                10 * (i + 1),
                (*itr).beg + 1,
            );
            libc::exit(-1);
        }
        str_.l = 0;
        let mut buf = [0 as c_char; 128];
        let len = libc::snprintf(buf.as_mut_ptr(), buf.len(), c"%ld".as_ptr(), (*itr).beg + 1);
        kputsn(buf.as_ptr(), len as usize, &mut str_);
        let payload = *(*itr).payload.cast::<*mut c_char>();
        if libc::strcmp(payload, str_.s) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"listing failed, expected payload \"%s\", found \"%s\"\n".as_ptr(),
                str_.s,
                payload,
            );
            libc::exit(-1);
        }
        i += 1;
    }
    if i != n {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Expected %d regions, listed %d\n".as_ptr(),
            n,
            i,
        );
        libc::exit(-1);
    }
    if VERBOSE >= 2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"ok: listed %d regions\n".as_ptr(),
            n,
        );
    }

    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
    libc::free(str_.s.cast());
}

// original: test_custom_payload (htslib/test/test-regidx.c:143)
pub unsafe fn test_test_regidx_c_143_test_custom_payload() {
    let idx = regidx_c_246_regidx_init(
        ptr::null(),
        Some(test_test_regidx_c_75_custom_parse),
        Some(test_test_regidx_c_100_custom_free),
        std::mem::size_of::<*mut c_char>(),
        ptr::null_mut(),
    );
    if idx.is_null() {
        libc::fprintf(hts_sys::stderr.cast(), c"init failed\n".as_ptr());
        libc::exit(-1);
    }

    let mut line = c"1 10000000 10000000 1:10000000-10000000".as_ptr() as *mut c_char;
    if regidx_c_198_regidx_insert(idx, line) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"insert failed: %s\n".as_ptr(),
            line,
        );
        libc::exit(-1);
    }
    line = c"1 20000000 20000001 1:20000000-20000001".as_ptr() as *mut c_char;
    if regidx_c_198_regidx_insert(idx, line) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"insert failed: %s\n".as_ptr(),
            line,
        );
        libc::exit(-1);
    }
    line = c"1 20000002 20000002 1:20000002-20000002".as_ptr() as *mut c_char;
    if regidx_c_198_regidx_insert(idx, line) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"insert failed: %s\n".as_ptr(),
            line,
        );
        libc::exit(-1);
    }
    line = c"1 30000000 30000000 1:30000000-30000000".as_ptr() as *mut c_char;
    if regidx_c_198_regidx_insert(idx, line) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"insert failed: %s\n".as_ptr(),
            line,
        );
        libc::exit(-1);
    }
    line = c"1 8000000000 8000000000 1:8000000000-8000000000".as_ptr() as *mut c_char;
    if regidx_c_198_regidx_insert(idx, line) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"insert failed: %s\n".as_ptr(),
            line,
        );
        libc::exit(-1);
    }

    let itr = regidx_c_584_regitr_init(idx);
    let mut from: hts_pos_t = 10000000;
    let mut to: hts_pos_t = 10000000;

    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 1, to - 1, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from,
            to,
        );
        libc::exit(-1);
    }
    if libc::strcmp(
        c"1:10000000-10000000".as_ptr(),
        *(*itr).payload.cast::<*mut c_char>(),
    ) != 0
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld vs %s\n".as_ptr(),
            from,
            to,
            *(*itr).payload.cast::<*mut c_char>(),
        );
        libc::exit(-1);
    }
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 2, to - 1, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from - 1,
            to,
        );
        libc::exit(-1);
    }
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 2, to + 3, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from - 1,
            to + 2,
        );
        libc::exit(-1);
    }
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 2, to - 2, itr) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from - 1,
            to - 1,
        );
        libc::exit(-1);
    }

    from = 20000000;
    to = 20000000;
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 1, to - 1, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from,
            to,
        );
        libc::exit(-1);
    }

    from = 20000002;
    to = 20000002;
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 1, to - 1, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from,
            to,
        );
        libc::exit(-1);
    }

    from = 30000000;
    to = 30000000;
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 1, to - 1, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from,
            to,
        );
        libc::exit(-1);
    }

    from = 8000000000;
    to = 8000000000;
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 1, to - 1, itr) == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed: 1:%ld-%ld\n".as_ptr(),
            from,
            to,
        );
        libc::exit(-1);
    }

    from &= 0xffffffffu32 as hts_pos_t;
    to &= 0xffffffffu32 as hts_pos_t;
    if regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), from - 1, to - 1, itr) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query should not succeed: 1:%ld-%ld\n".as_ptr(),
            from,
            to,
        );
        libc::exit(-1);
    }

    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
}

// original: get_random_region (htslib/test/test-regidx.c:190)
pub unsafe fn test_test_regidx_c_190_get_random_region(
    min: u32,
    max: u32,
    beg: *mut u32,
    end: *mut u32,
) {
    let b = libc::rand() as u64;
    let e = libc::rand() as u64;
    *beg = min + ((b * (max - min) as u64) / libc::RAND_MAX as u64) as u32;
    *end = *beg + ((e * (max - *beg) as u64) / libc::RAND_MAX as u64) as u32;
}

// original: test_random (htslib/test/test-regidx.c:197)
pub unsafe fn test_test_regidx_c_197_test_random(nregs: c_int, mut min: u32, mut max: u32) {
    min -= 1;
    max -= 1;

    let idx = regidx_c_246_regidx_init(
        ptr::null(),
        Some(test_test_regidx_c_75_custom_parse),
        Some(test_test_regidx_c_100_custom_free),
        std::mem::size_of::<*mut c_char>(),
        ptr::null_mut(),
    );
    if idx.is_null() {
        libc::fprintf(hts_sys::stderr.cast(), c"init failed\n".as_ptr());
        libc::exit(-1);
    }

    let mut beg: u32 = 0;
    let mut end: u32 = 0;
    test_test_regidx_c_190_get_random_region(min, max, &mut beg, &mut end);

    let mut nexp = 0;
    let mut str_: kstring_t = std::mem::zeroed();
    for _i in 0..nregs {
        let mut b: u32 = 0;
        let mut e: u32 = 0;
        test_test_regidx_c_190_get_random_region(min, max, &mut b, &mut e);
        str_.l = 0;
        let mut buf = [0 as c_char; 256];
        let len = libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"1\t%u\t%u\t1:%u-%u".as_ptr(),
            b + 1,
            e + 1,
            b + 1,
            e + 1,
        );
        kputsn(buf.as_ptr(), len as usize, &mut str_);
        if regidx_c_198_regidx_insert(idx, str_.s) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert failed: %s\n".as_ptr(),
                str_.s,
            );
            libc::exit(-1);
        }
        if e >= beg && b <= end {
            nexp += 1;
        }
    }

    let itr = regidx_c_584_regitr_init(idx);
    let mut nhit = 0;
    let ret =
        regidx_c_401_regidx_overlap(idx, c"1".as_ptr(), beg as hts_pos_t, end as hts_pos_t, itr);
    if nexp != 0 && ret == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed, expected %d overlap(s), found none: %d-%d\n".as_ptr(),
            nexp,
            beg + 1,
            end + 1,
        );
        libc::exit(-1);
    }
    if nexp == 0 && ret != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed, expected no overlaps, found some: %d-%d\n".as_ptr(),
            beg + 1,
            end + 1,
        );
        libc::exit(-1);
    }
    while ret != 0 && regidx_c_612_regitr_overlap(itr) != 0 {
        str_.l = 0;
        let mut buf = [0 as c_char; 256];
        let len = libc::snprintf(
            buf.as_mut_ptr(),
            buf.len(),
            c"1:%ld-%ld".as_ptr(),
            (*itr).beg + 1,
            (*itr).end + 1,
        );
        kputsn(buf.as_ptr(), len as usize, &mut str_);
        let payload = *(*itr).payload.cast::<*mut c_char>();
        if libc::strcmp(str_.s, payload) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, incorrect payload: %s vs %s (%d-%d)\n".as_ptr(),
                str_.s,
                payload,
                beg + 1,
                end + 1,
            );
            libc::exit(-1);
        }
        if (*itr).beg > end as hts_pos_t || (*itr).end < beg as hts_pos_t {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, incorrect hit: %d-%d vs %ld-%ld, payload %s\n".as_ptr(),
                beg + 1,
                end + 1,
                (*itr).beg + 1,
                (*itr).end + 1,
                payload,
            );
            libc::exit(-1);
        }
        nhit += 1;
    }
    if nexp != nhit {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"query failed, expected %d overlap(s), found %d: %d-%d\n".as_ptr(),
            nexp,
            nhit,
            beg + 1,
            end + 1,
        );
        libc::exit(-1);
    }
    if VERBOSE >= 2 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"ok: found %d overlaps\n".as_ptr(),
            nexp,
        );
    }

    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
    libc::free(str_.s.cast());
}

// original: test_explicit (htslib/test/test-regidx.c:246)
pub unsafe fn test_test_regidx_c_246_test_explicit(
    tgt: *mut c_char,
    qry: *mut c_char,
    mut exp: *mut c_char,
) {
    let idx = regidx_c_246_regidx_init(
        ptr::null(),
        Some(regidx_c_545_regidx_parse_reg),
        None,
        0,
        ptr::null_mut(),
    );

    let mut beg_p = tgt;
    let mut end_p: *mut c_char;
    let exp_ori = exp;
    let mut str_: kstring_t = std::mem::zeroed();
    while *beg_p != 0 {
        end_p = tgt;
        while *end_p != 0 && *end_p != b';' as c_char {
            end_p = end_p.add(1);
        }
        str_.l = 0;
        kputsn(beg_p, end_p.offset_from(beg_p) as usize, &mut str_);
        if VERBOSE >= 2 {
            libc::fprintf(hts_sys::stderr.cast(), c"insert: %s\n".as_ptr(), str_.s);
        }
        if regidx_c_198_regidx_insert(idx, str_.s) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert failed: %s\n".as_ptr(),
                str_.s,
            );
            libc::exit(-1);
        }
        beg_p = if *end_p != 0 { end_p.add(1) } else { end_p };
    }

    beg_p = qry;
    while *beg_p != 0 {
        end_p = qry;
        while *end_p != 0 && *end_p != b';' as c_char {
            end_p = end_p.add(1);
        }
        str_.l = 0;
        kputsn(beg_p, end_p.offset_from(beg_p) as usize, &mut str_);
        beg_p = if *end_p != 0 { end_p.add(1) } else { end_p };

        let mut chr_beg: *mut c_char = ptr::null_mut();
        let mut chr_end: *mut c_char = ptr::null_mut();
        let mut reg_beg: hts_pos_t = 0;
        let mut reg_end: hts_pos_t = 0;
        if regidx_c_545_regidx_parse_reg(
            str_.s,
            &mut chr_beg,
            &mut chr_end,
            &mut reg_beg,
            &mut reg_end,
            ptr::null_mut(),
            ptr::null_mut(),
        ) != 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"could not parse: %s in %s\n".as_ptr(),
                str_.s,
                qry,
            );
            libc::exit(-1);
        }
        *chr_end.add(1) = 0;
        let hit = regidx_c_401_regidx_overlap(idx, chr_beg, reg_beg, reg_end, ptr::null_mut());
        if *exp == b'1' as c_char {
            if hit == 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"query failed, there should be a hit .. %s:%ld-%ld\n".as_ptr(),
                    chr_beg,
                    reg_beg + 1,
                    reg_end + 1,
                );
                libc::exit(-1);
            } else if VERBOSE >= 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"ok: overlap found for %s:%ld-%ld\n".as_ptr(),
                    chr_beg,
                    reg_beg + 1,
                    reg_end + 1,
                );
            }
        } else if *exp == b'0' as c_char {
            if hit != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"query failed, there should be no hit .. %s:%ld-%ld\n".as_ptr(),
                    chr_beg,
                    reg_beg + 1,
                    reg_end + 1,
                );
                libc::exit(-1);
            } else if VERBOSE >= 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"ok: no overlap found for %s:%ld-%ld\n".as_ptr(),
                    chr_beg,
                    reg_beg + 1,
                    reg_end + 1,
                );
            }
        } else {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"could not parse: %s\n".as_ptr(),
                exp_ori,
            );
            libc::exit(-1);
        }
        exp = exp.add(1);
    }

    libc::free(str_.s.cast());
    regidx_c_311_regidx_destroy(idx);
}

// original: create_line_bed (htslib/test/test-regidx.c:307)
pub unsafe fn test_test_regidx_c_307_create_line_bed(
    line: *mut c_char,
    size: usize,
    chr: *mut c_char,
    start: c_int,
    end: c_int,
) {
    libc::snprintf(line, size, c"%s\t%d\t%d\n".as_ptr(), chr, start - 1, end);
}

// original: create_line_tab (htslib/test/test-regidx.c:311)
pub unsafe fn test_test_regidx_c_311_create_line_tab(
    line: *mut c_char,
    size: usize,
    chr: *mut c_char,
    start: c_int,
    end: c_int,
) {
    libc::snprintf(line, size, c"%s\t%d\t%d\n".as_ptr(), chr, start, end);
}

// original: create_line_reg (htslib/test/test-regidx.c:315)
pub unsafe fn test_test_regidx_c_315_create_line_reg(
    line: *mut c_char,
    size: usize,
    chr: *mut c_char,
    start: c_int,
    end: c_int,
) {
    libc::snprintf(line, size, c"%s:%d-%d\n".as_ptr(), chr, start, end);
}

type set_line_f = unsafe fn(*mut c_char, usize, *mut c_char, c_int, c_int);

// original: test (htslib/test/test-regidx.c:322)
pub unsafe fn test_test_regidx_c_322_test(set_line: set_line_f, parse: regidx_parse_f) {
    let idx = regidx_c_246_regidx_init(ptr::null(), parse, None, 0, ptr::null_mut());
    if idx.is_null() {
        libc::fprintf(hts_sys::stderr.cast(), c"init failed\n".as_ptr());
        libc::exit(-1);
    }

    let mut line = [0 as c_char; 250];
    let chr = c"1".as_ptr() as *mut c_char;
    let n = 10;
    for i in 1..n {
        let mut start = 10 * i;
        let mut end = start;
        set_line(line.as_mut_ptr(), line.len(), chr, start, end);
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert: %s".as_ptr(),
                line.as_ptr(),
            );
        }
        if regidx_c_198_regidx_insert(idx, line.as_mut_ptr()) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert failed: %s\n".as_ptr(),
                line.as_ptr(),
            );
            libc::exit(-1);
        }

        start = 10 * i + 1;
        end = start;
        set_line(line.as_mut_ptr(), line.len(), chr, start, end);
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert: %s".as_ptr(),
                line.as_ptr(),
            );
        }
        if regidx_c_198_regidx_insert(idx, line.as_mut_ptr()) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert failed: %s\n".as_ptr(),
                line.as_ptr(),
            );
            libc::exit(-1);
        }

        start = 20000 * i;
        end = start + 2000;
        set_line(line.as_mut_ptr(), line.len(), chr, start, end);
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert: %s".as_ptr(),
                line.as_ptr(),
            );
        }
        if regidx_c_198_regidx_insert(idx, line.as_mut_ptr()) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"insert failed: %s\n".as_ptr(),
                line.as_ptr(),
            );
            libc::exit(-1);
        }
    }

    let itr = regidx_c_584_regitr_init(idx);
    for i in 1..n {
        let mut start = 10 * i - 1;
        let mut end = start;
        if regidx_c_401_regidx_overlap(
            idx,
            chr,
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            itr,
        ) != 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, there should be no hit: %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"ok: no overlap found for %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
        }

        start = 10 * i;
        end = start;
        if regidx_c_401_regidx_overlap(
            idx,
            chr,
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            itr,
        ) == 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, there should be a hit: %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"ok: overlap(s) found for %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
        }
        let mut nhit = 0;
        while regidx_c_612_regitr_overlap(itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"query failed, incorrect region: %ld-%ld for %d-%d\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end,
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"\t %ld-%ld\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                );
            }
            nhit += 1;
        }
        if nhit != 1 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, expected one hit, found %d: %s:%d-%d\n".as_ptr(),
                nhit,
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }

        start = 10 * i + 1;
        end = start;
        if regidx_c_401_regidx_overlap(
            idx,
            chr,
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            itr,
        ) == 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, there should be a hit: %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"ok: overlap(s) found for %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
        }
        nhit = 0;
        while regidx_c_612_regitr_overlap(itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"query failed, incorrect region: %ld-%ld for %d-%d\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end,
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"\t %ld-%ld\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                );
            }
            nhit += 1;
        }
        if nhit != 1 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, expected one hit, found %d: %s:%d-%d\n".as_ptr(),
                nhit,
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }

        start = 10 * i;
        end = start + 1;
        if regidx_c_401_regidx_overlap(
            idx,
            chr,
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            itr,
        ) == 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, there should be a hit: %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"ok: overlap(s) found for %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
        }
        nhit = 0;
        while regidx_c_612_regitr_overlap(itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"query failed, incorrect region: %ld-%ld for %d-%d\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end,
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"\t %ld-%ld\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                );
            }
            nhit += 1;
        }
        if nhit != 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, expected two hits, found %d: %s:%d-%d\n".as_ptr(),
                nhit,
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }

        start = 20000 * i - 5000;
        end = 20000 * i + 3000;
        set_line(line.as_mut_ptr(), line.len(), chr, start, end);
        if regidx_c_401_regidx_overlap(
            idx,
            chr,
            (start - 1) as hts_pos_t,
            (end - 1) as hts_pos_t,
            itr,
        ) == 0
        {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, there should be a hit: %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }
        if VERBOSE >= 2 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"ok: overlap(s) found for %s:%d-%d\n".as_ptr(),
                chr,
                start,
                end,
            );
        }
        nhit = 0;
        while regidx_c_612_regitr_overlap(itr) != 0 {
            if (*itr).beg > (end - 1) as hts_pos_t || (*itr).end < (start - 1) as hts_pos_t {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"query failed, incorrect region: %ld-%ld for %d-%d\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                    start,
                    end,
                );
                libc::exit(-1);
            }
            if VERBOSE >= 2 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"\t %ld-%ld\n".as_ptr(),
                    (*itr).beg + 1,
                    (*itr).end + 1,
                );
            }
            nhit += 1;
        }
        if nhit != 1 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"query failed, expected one hit, found %d: %s:%d-%d\n".as_ptr(),
                nhit,
                chr,
                start,
                end,
            );
            libc::exit(-1);
        }
    }
    regidx_c_606_regitr_destroy(itr);
    regidx_c_311_regidx_destroy(idx);
}

// original: usage (htslib/test/test-regidx.c:415)
pub unsafe fn test_test_regidx_c_415_usage() -> ! {
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"Usage: test-regidx [OPTIONS]\n".as_ptr(),
    );
    libc::fprintf(hts_sys::stderr.cast(), c"Options:\n".as_ptr());
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -h, --help          this help message\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -s, --seed <int>    random seed\n".as_ptr(),
    );
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"   -v, --verbose       increase verbosity by giving multiple times\n".as_ptr(),
    );
    libc::exit(1);
}

// original: main (htslib/test/test-regidx.c:426)
pub unsafe fn test_test_regidx_c_426_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut loptions = [
        libc::option {
            name: c"help".as_ptr(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: b'h' as c_int,
        },
        libc::option {
            name: c"verbose".as_ptr(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: b'v' as c_int,
        },
        libc::option {
            name: c"seed".as_ptr(),
            has_arg: 1,
            flag: ptr::null_mut(),
            val: b's' as c_int,
        },
        libc::option {
            name: ptr::null(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: 0,
        },
    ];
    let mut seed = libc::time(ptr::null_mut()) as c_int;
    loop {
        let c = libc::getopt_long(
            argc,
            argv,
            c"hvs:".as_ptr(),
            loptions.as_mut_ptr(),
            ptr::null_mut(),
        );
        if c < 0 {
            break;
        }
        match c {
            x if x == b's' as c_int => seed = libc::atoi(optarg),
            x if x == b'v' as c_int => VERBOSE += 1,
            _ => test_test_regidx_c_415_usage(),
        }
    }

    if VERBOSE >= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Testing sequential access\n".as_ptr(),
        );
    }
    test_test_regidx_c_106_test_sequential_access();

    if VERBOSE >= 1 {
        libc::fprintf(hts_sys::stderr.cast(), c"Testing TAB\n".as_ptr());
    }
    test_test_regidx_c_322_test(
        test_test_regidx_c_311_create_line_tab,
        Some(regidx_c_498_regidx_parse_tab),
    );

    if VERBOSE >= 1 {
        libc::fprintf(hts_sys::stderr.cast(), c"Testing REG\n".as_ptr());
    }
    test_test_regidx_c_322_test(
        test_test_regidx_c_315_create_line_reg,
        Some(regidx_c_545_regidx_parse_reg),
    );

    if VERBOSE >= 1 {
        libc::fprintf(hts_sys::stderr.cast(), c"Testing BED\n".as_ptr());
    }
    test_test_regidx_c_322_test(
        test_test_regidx_c_307_create_line_bed,
        Some(regidx_c_466_regidx_parse_bed),
    );

    if VERBOSE >= 1 {
        libc::fprintf(hts_sys::stderr.cast(), c"Testing custom payload\n".as_ptr());
    }
    test_test_regidx_c_143_test_custom_payload();

    if VERBOSE >= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Testing cases encountered in past\n".as_ptr(),
        );
    }
    test_test_regidx_c_246_test_explicit(
        c"12:2064519-2064763".as_ptr() as *mut c_char,
        c"12:2064488-2067434".as_ptr() as *mut c_char,
        c"1".as_ptr() as *mut c_char,
    );

    let ntest = 1000;
    let nreg = 50;
    libc::srand(seed as libc::c_uint);
    if VERBOSE >= 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%d randomized tests, %d regions per test. Random seed is %d\n".as_ptr(),
            ntest,
            nreg,
            seed,
        );
    }
    for _i in 0..ntest {
        test_test_regidx_c_197_test_random(nreg, 1, 1000);
    }

    0
}
