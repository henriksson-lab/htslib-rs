use crate::htslib_rs::hts::kstring_t;
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

unsafe extern "C" {
    static mut optarg: *mut c_char;
}

// original: clamp (htslib/test/test_kstring.c:40)
pub unsafe fn test_test_kstring_c_40_clamp(val: *mut i64, min: i64, max: i64) {
    if *val < min {
        *val = min;
    }
    if *val > max {
        *val = max;
    }
}

// original: test_kroundup_size_t (htslib/test/test_kstring.c:45)
pub unsafe fn test_test_kstring_c_45_test_kroundup_size_t(verbose: c_int) -> c_int {
    fn kroundup_size_t(mut x: usize) -> usize {
        if x == 0 {
            return 0;
        }
        x -= 1;
        x |= x >> (std::mem::size_of::<usize>() / 8);
        x |= x >> (std::mem::size_of::<usize>() / 4);
        x |= x >> (std::mem::size_of::<usize>() / 2);
        x |= x >> std::mem::size_of::<usize>();
        x |= x >> (std::mem::size_of::<usize>() * 2);
        x |= x >> (std::mem::size_of::<usize>() * 4);
        if (x & (1usize << (usize::BITS - 1))) == 0 {
            x += 1;
        }
        x
    }

    let mut ret = 0;
    let mut val = kroundup_size_t(0);
    if verbose != 0 {
        libc::printf(c"kroundup_size_t(0) = 0x%zx\n".as_ptr(), val);
    }
    if val != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"kroundup_size_t(0) produced 0x%zx, expected 0\n".as_ptr(),
            val,
        );
        ret = -1;
    }

    for exp in 0..usize::BITS {
        let mut expected = 1usize << exp;
        let first_delta: isize = if exp > 1 { -1 } else { 0 };
        let last_delta: isize = if exp < 2 { 0 } else { 1 };
        for delta in first_delta..=last_delta {
            let val_in = expected.wrapping_add(delta as usize);
            val = kroundup_size_t(val_in);
            if verbose != 0 {
                libc::printf(c"kroundup_size_t(0x%zx) = 0x%zx\n".as_ptr(), val_in, val);
            }
            if delta <= 0 {
                if val != expected {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"kroundup_size_t(0x%zx) produced 0x%zx, expected 0x%zx\n".as_ptr(),
                        val_in,
                        val,
                        expected,
                    );
                    ret = -1;
                }
            } else {
                expected = expected.wrapping_mul(2);
                if expected == 0 {
                    expected = expected.wrapping_sub(1);
                }
                if val != expected {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"kroundup_size_t(0x%zx) produced 0x%zx, expected 0x%zx\n".as_ptr(),
                        val_in,
                        val,
                        expected,
                    );
                    ret = -1;
                }
            }
        }
    }
    ret
}

// original: test_kroundup_signed (htslib/test/test_kstring.c:91)
pub unsafe fn test_test_kstring_c_91_test_kroundup_signed(verbose: c_int) -> c_int {
    fn kroundup32(mut x: u32) -> u32 {
        if x == 0 {
            return 0;
        }
        x -= 1;
        x |= x >> 1;
        x |= x >> 2;
        x |= x >> 4;
        x |= x >> 8;
        x |= x >> 16;
        if (x & 0x8000_0000) == 0 {
            x += 1;
        }
        x
    }

    let mut ret = 0;
    for exp in 0..31u32 {
        let mut expected = 1u32 << exp;
        let first_delta: i32 = if exp > 1 { -1 } else { 0 };
        let last_delta: i32 = if exp < 2 { 0 } else { 1 };
        for delta in first_delta..=last_delta {
            let val_in = expected.wrapping_add(delta as u32) as i32;
            let val = kroundup32(val_in as u32) as i32;
            if verbose != 0 {
                libc::printf(c"kroundup32(%d) = %d\n".as_ptr(), val_in, val);
            }
            if delta <= 0 {
                if val as u32 != expected {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"kroundup32(%d) produced %d, expected %u\n".as_ptr(),
                        val_in,
                        val,
                        expected,
                    );
                    ret = -1;
                }
            } else {
                if exp < 30 {
                    expected = expected.wrapping_mul(2);
                } else {
                    expected = ((expected - 1) << 1) | 1;
                }
                if val as u32 != expected {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"kroundup32(%d) produced %d, expected %u\n".as_ptr(),
                        val_in,
                        val,
                        expected,
                    );
                    ret = -1;
                }
            }
        }
    }
    ret
}

// original: test_kputuw_from_to (htslib/test/test_kstring.c:127)
pub unsafe fn test_test_kstring_c_127_test_kputuw_from_to(
    str_: *mut kstring_t,
    s: c_uint,
    e: c_uint,
) -> c_int {
    let mut i = s;
    loop {
        (*str_).l = 0;
        libc::memset((*str_).s.cast(), 0xff, (*str_).m);
        if crate::htslib_rs::hts::kputuw(i, str_) < 0 || (*str_).s.is_null() {
            libc::perror(c"kputuw".as_ptr());
            return -1;
        }
        if (*str_).l >= (*str_).m || *(*str_).s.add((*str_).l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kputuw\n".as_ptr(),
            );
            return -1;
        }
        if i != libc::strtoul((*str_).s, std::ptr::null_mut(), 10) as c_uint {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kputuw wrote the wrong value, expected %u, got %s\n".as_ptr(),
                i,
                (*str_).s,
            );
            return -1;
        }
        if i >= e {
            break;
        }
        i = i.wrapping_add(1);
    }
    0
}

// original: test_kputuw (htslib/test/test_kstring.c:153)
pub unsafe fn test_test_kstring_c_153_test_kputuw(mut start: i64, mut end: i64) -> c_int {
    let mut str_: kstring_t = std::mem::zeroed();
    str_.s = libc::malloc(2).cast();
    if str_.s.is_null() {
        libc::perror(c"malloc".as_ptr());
        return -1;
    }
    str_.m = 2;

    let mut val = 0i64;
    while val < c_uint::MAX as i64 {
        let s = if val == 0 { 0 } else { val - 5 } as c_uint;
        let e = (val + 5) as c_uint;
        if test_test_kstring_c_127_test_kputuw_from_to(&mut str_, s, e) < 0 {
            libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
            return -1;
        }
        val = if val == 0 { 1 } else { val * 10 };
    }

    if test_test_kstring_c_127_test_kputuw_from_to(&mut str_, c_uint::MAX - 5, c_uint::MAX) < 0 {
        libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
        return -1;
    }

    str_.m = 1;
    test_test_kstring_c_40_clamp(&mut start, 0, c_uint::MAX as i64);
    test_test_kstring_c_40_clamp(&mut end, 0, c_uint::MAX as i64);
    if test_test_kstring_c_127_test_kputuw_from_to(&mut str_, start as c_uint, end as c_uint) < 0 {
        libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
        return -1;
    }

    libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
    0
}

// original: test_kputw_from_to (htslib/test/test_kstring.c:193)
pub unsafe fn test_test_kstring_c_193_test_kputw_from_to(
    str_: *mut kstring_t,
    s: c_int,
    e: c_int,
) -> c_int {
    let mut i = s;
    loop {
        (*str_).l = 0;
        libc::memset((*str_).s.cast(), 0xff, (*str_).m);
        if crate::htslib_rs::hts::kputw(i, str_) < 0 || (*str_).s.is_null() {
            libc::perror(c"kputw".as_ptr());
            return -1;
        }
        if (*str_).l >= (*str_).m || *(*str_).s.add((*str_).l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kputw\n".as_ptr(),
            );
            return -1;
        }
        if i != libc::strtol((*str_).s, std::ptr::null_mut(), 10) as c_int {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kputw wrote the wrong value, expected %d, got %s\n".as_ptr(),
                i,
                (*str_).s,
            );
            return -1;
        }
        if i >= e {
            break;
        }
        i = i.wrapping_add(1);
    }
    0
}

// original: test_kputw (htslib/test/test_kstring.c:219)
pub unsafe fn test_test_kstring_c_219_test_kputw(mut start: i64, mut end: i64) -> c_int {
    let mut str_: kstring_t = std::mem::zeroed();
    str_.s = libc::malloc(2).cast();
    if str_.s.is_null() {
        libc::perror(c"malloc".as_ptr());
        return -1;
    }
    str_.m = 2;

    let mut val = 1i64;
    while val < c_int::MAX as i64 {
        let s = if val > 5 { val - 5 } else { 0 } as c_int;
        let e = (val + 5) as c_int;
        if test_test_kstring_c_193_test_kputw_from_to(&mut str_, s, e) < 0 {
            libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
            return -1;
        }
        val *= 10;
    }

    val = -1;
    while val > c_int::MIN as i64 {
        let s = (val - 5) as c_int;
        let e = if val < -5 { val + 5 } else { 0 } as c_int;
        if test_test_kstring_c_193_test_kputw_from_to(&mut str_, s, e) < 0 {
            libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
            return -1;
        }
        val *= 10;
    }

    if test_test_kstring_c_193_test_kputw_from_to(&mut str_, c_int::MAX - 5, c_int::MAX) < 0
        || test_test_kstring_c_193_test_kputw_from_to(&mut str_, c_int::MIN, c_int::MIN + 5) < 0
    {
        libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
        return -1;
    }

    str_.m = 1;
    test_test_kstring_c_40_clamp(&mut start, c_int::MIN as i64, c_int::MAX as i64);
    test_test_kstring_c_40_clamp(&mut end, c_int::MIN as i64, c_int::MAX as i64);
    if test_test_kstring_c_193_test_kputw_from_to(&mut str_, start as c_int, end as c_int) < 0 {
        libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
        return -1;
    }

    libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
    0
}

// original: test_kputll_from_to (htslib/test/test_kstring.c:268)
pub unsafe fn test_test_kstring_c_268_test_kputll_from_to(
    str_: *mut kstring_t,
    s: i64,
    e: i64,
) -> c_int {
    let mut i = s;
    loop {
        (*str_).l = 0;
        libc::memset((*str_).s.cast(), 0xff, (*str_).m);
        if crate::htslib_rs::hts::kputll(i, str_) < 0 || (*str_).s.is_null() {
            libc::perror(c"kputll".as_ptr());
            return -1;
        }
        if (*str_).l >= (*str_).m || *(*str_).s.add((*str_).l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kputll\n".as_ptr(),
            );
            return -1;
        }
        if i != libc::strtoll((*str_).s, std::ptr::null_mut(), 10) as i64 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kputll wrote the wrong value, expected %lld, got %s\n".as_ptr(),
                i,
                (*str_).s,
            );
            return -1;
        }
        if i >= e {
            break;
        }
        i = i.wrapping_add(1);
    }
    0
}

// original: test_kputll (htslib/test/test_kstring.c:294)
pub unsafe fn test_test_kstring_c_294_test_kputll(mut start: i64, mut end: i64) -> c_int {
    let mut str_: kstring_t = std::mem::zeroed();
    str_.s = libc::malloc(2).cast();
    if str_.s.is_null() {
        libc::perror(c"malloc".as_ptr());
        return -1;
    }
    str_.m = 2;

    let mut val = 1u64;
    while val < i64::MAX as u64 - 5 {
        let s = if val >= 5 { val - 5 } else { val } as i64;
        if test_test_kstring_c_268_test_kputll_from_to(&mut str_, s, val as i64) < 0 {
            libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
            return -1;
        }
        val *= 10;
    }

    val = 1;
    while val < i64::MAX as u64 - 5 {
        let valm = -(val as i64);
        let s = if valm >= 5 { valm - 5 } else { valm };
        if test_test_kstring_c_268_test_kputll_from_to(&mut str_, s, valm) < 0 {
            libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
            return -1;
        }
        val *= 10;
    }

    if test_test_kstring_c_268_test_kputll_from_to(&mut str_, i64::MAX - 5, i64::MAX) < 0
        || test_test_kstring_c_268_test_kputll_from_to(&mut str_, i64::MIN, i64::MIN + 5) < 0
    {
        libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
        return -1;
    }

    str_.m = 1;
    test_test_kstring_c_40_clamp(&mut start, i64::MIN, i64::MAX);
    test_test_kstring_c_40_clamp(&mut end, i64::MIN, i64::MAX);
    if test_test_kstring_c_268_test_kputll_from_to(&mut str_, start, end) < 0 {
        libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
        return -1;
    }

    libc::free(crate::htslib_rs::hts::ks_release(&mut str_).cast());
    0
}

// original: mock_fgets (htslib/test/test_kstring.c:347)
pub unsafe extern "C" fn test_test_kstring_c_347_mock_fgets(
    str_: *mut c_char,
    _num: c_int,
    p: *mut c_void,
) -> *mut c_char {
    let mock_state = p.cast::<c_int>();
    *mock_state += 1;
    match *mock_state {
        1 | 4 | 7 => libc::strcpy(str_, c"ABCD".as_ptr()),
        2 | 3 => libc::strcpy(str_, c"\n".as_ptr()),
        5 | 6 => libc::strcpy(str_, c"\r\n".as_ptr()),
        _ => return std::ptr::null_mut(),
    };
    str_
}

// original: test_kgetline (htslib/test/test_kstring.c:375)
pub unsafe fn test_test_kstring_c_375_test_kgetline() -> c_int {
    let mut s: kstring_t = std::mem::zeroed();
    let mut mock_state = 0;

    crate::htslib_rs::hts::kputs(c"_".as_ptr(), &mut s);
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"_ABCD".as_ptr(), s.s) != 0
        || s.l != 5
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"".as_ptr(), s.s) != 0
        || s.l != 0
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"ABCD".as_ptr(), s.s) != 0
        || s.l != 4
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"".as_ptr(), s.s) != 0
        || s.l != 0
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"ABCD".as_ptr(), s.s) != 0
        || s.l != 4
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut c_int).cast(),
    ) != libc::EOF
        || s.l != 0
    {
        return -1;
    }

    crate::htslib_rs::hts::ks_free(&mut s);
    libc::EXIT_SUCCESS
}

// original: mock_fgets2 (htslib/test/test_kstring.c:403)
pub unsafe extern "C" fn test_test_kstring_c_403_mock_fgets2(
    str_: *mut c_char,
    _num: usize,
    p: *mut c_void,
) -> isize {
    let mock_state = p.cast::<c_int>();
    *mock_state += 1;
    match *mock_state {
        1 | 4 | 7 => libc::strcpy(str_, c"ABCD".as_ptr()),
        2 | 3 => libc::strcpy(str_, c"\n".as_ptr()),
        5 | 6 => libc::strcpy(str_, c"\r\n".as_ptr()),
        _ => return 0,
    };
    libc::strlen(str_) as isize
}

// original: test_kgetline2 (htslib/test/test_kstring.c:431)
pub unsafe fn test_test_kstring_c_431_test_kgetline2() -> c_int {
    let mut s: kstring_t = std::mem::zeroed();
    let mut mock_state = 0;

    crate::htslib_rs::hts::kputs(c"_".as_ptr(), &mut s);
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"_ABCD".as_ptr(), s.s) != 0
        || s.l != 5
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"".as_ptr(), s.s) != 0
        || s.l != 0
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"ABCD".as_ptr(), s.s) != 0
        || s.l != 4
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"".as_ptr(), s.s) != 0
        || s.l != 0
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut c_int).cast(),
    ) != 0
        || libc::strcmp(c"ABCD".as_ptr(), s.s) != 0
        || s.l != 4
    {
        return -1;
    }
    s.l = 0;
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut c_int).cast(),
    ) != libc::EOF
        || s.l != 0
    {
        return -1;
    }

    crate::htslib_rs::hts::ks_free(&mut s);
    libc::EXIT_SUCCESS
}

// original: test_kinsertchar (htslib/test/test_kstring.c:458)
pub unsafe fn test_test_kstring_c_458_test_kinsertchar() -> c_int {
    let expected = [
        c"".as_ptr(),
        c"X0123".as_ptr(),
        c"0X123".as_ptr(),
        c"01X23".as_ptr(),
        c"012X3".as_ptr(),
        c"0123X".as_ptr(),
        c"".as_ptr(),
    ];

    for i in -1..6 {
        let mut s: kstring_t = std::mem::zeroed();
        crate::htslib_rs::hts::kputs(c"0123".as_ptr(), &mut s);
        if crate::htslib_rs::hts::kinsert_char(b'X' as c_char, i as usize, &mut s) < 0 {
            if !(0..=4).contains(&i) {
                crate::htslib_rs::hts::ks_free(&mut s);
                continue;
            }
            libc::fprintf(hts_sys::stderr.cast(), c"kinsert_char failed\n".as_ptr());
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        if *s.s.add(s.l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kinsert_char\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        if libc::memcmp(s.s.cast(), expected[(i + 1) as usize].cast(), s.l + 1) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_char comparison failed\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        crate::htslib_rs::hts::ks_free(&mut s);
    }

    let mut t: kstring_t = std::mem::zeroed();
    let mut res: kstring_t = std::mem::zeroed();
    for i in 0..7 {
        crate::htslib_rs::hts::kputc(b'A' as c_int + i, &mut res);
        if crate::htslib_rs::hts::kinsert_char((b'A' as c_int + i) as c_char, t.l, &mut t) < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_char failed in realloc\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
        if *t.s.add(t.l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kinsert_char in realloc\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
        if libc::memcmp(t.s.cast(), res.s.cast(), res.l + 1) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_char realloc comparison failed in realloc\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
    }
    crate::htslib_rs::hts::ks_free(&mut t);
    crate::htslib_rs::hts::ks_free(&mut res);
    0
}

// original: data (htslib/test/test_kstring.c:461)
#[repr(C)]
pub struct data {
    _private: [u8; 0],
}

// original: test_kinsertstr (htslib/test/test_kstring.c:514)
pub unsafe fn test_test_kstring_c_514_test_kinsertstr() -> c_int {
    let expected = [
        c"".as_ptr(),
        c"XYZ0123".as_ptr(),
        c"0XYZ123".as_ptr(),
        c"01XYZ23".as_ptr(),
        c"012XYZ3".as_ptr(),
        c"0123XYZ".as_ptr(),
        c"".as_ptr(),
    ];

    for i in -1..6 {
        let mut s: kstring_t = std::mem::zeroed();
        crate::htslib_rs::hts::kputs(c"0123".as_ptr(), &mut s);
        if crate::htslib_rs::hts::kinsert_str(c"XYZ".as_ptr(), i as usize, &mut s) < 0 {
            if !(0..=4).contains(&i) {
                crate::htslib_rs::hts::ks_free(&mut s);
                continue;
            }
            libc::fprintf(hts_sys::stderr.cast(), c"kinsert_str failed\n".as_ptr());
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        if *s.s.add(s.l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kinsert_str\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        if libc::memcmp(s.s.cast(), expected[(i + 1) as usize].cast(), s.l + 1) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_str comparison failed\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        crate::htslib_rs::hts::ks_free(&mut s);
    }

    let mut t: kstring_t = std::mem::zeroed();
    let mut res: kstring_t = std::mem::zeroed();
    for i in 0..15 {
        let ch = (b'A' + i as u8) as c_int;
        let mut one = [ch as c_char, 0];
        crate::htslib_rs::hts::kputs(one.as_mut_ptr(), &mut res);
        if crate::htslib_rs::hts::kinsert_str(one.as_mut_ptr(), t.l, &mut t) < 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_str failed in realloc\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
        if *t.s.add(t.l) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"No NUL termination on string from kinsert_str in realloc\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
        if libc::memcmp(t.s.cast(), res.s.cast(), res.l + 1) != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_str realloc comparison failed in realloc\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
    }

    crate::htslib_rs::hts::ks_free(&mut t);
    if crate::htslib_rs::hts::kinsert_str(c"".as_ptr(), 1, &mut t) != 0 {
        if crate::htslib_rs::hts::kinsert_str(c"".as_ptr(), 0, &mut t) != 0 || t.l != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kinsert_str empty insertion failed\n".as_ptr(),
            );
            crate::htslib_rs::hts::ks_free(&mut res);
            return -1;
        }
    } else {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"kinsert_str empty ins to invalid pos succeeded\n".as_ptr(),
        );
        crate::htslib_rs::hts::ks_free(&mut res);
        return -1;
    }

    let old_len = res.l;
    if crate::htslib_rs::hts::kinsert_str(c"".as_ptr(), 1, &mut res) != 0 || old_len != res.l {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"kinsert_str empty ins to valid pos failed\n".as_ptr(),
        );
        crate::htslib_rs::hts::ks_free(&mut res);
        return -1;
    }
    crate::htslib_rs::hts::ks_free(&mut res);
    0
}

// original: data (htslib/test/test_kstring.c:517)
#[repr(C)]
pub struct test_test_kstring_c_517_data {
    _private: [u8; 0],
}

// original: test_kmemmem (htslib/test/test_kstring.c:586)
pub unsafe fn test_test_kstring_c_586_test_kmemmem() -> c_int {
    let tests: &[(&[u8], c_int, &[u8], c_int, isize)] = &[
        (b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, b"f\0\0", 3, 0),
        (b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, b"\0\0f", 3, 1),
        (b"\0\0f\0\0f\0\0fbar\0\0f\0\0f", 18, b"\0\0f", 3, 0),
        (b"\0\0f\0\0f\0\0fbar\0\0f\0\0f", 18, b"f\0\0", 3, 2),
        (b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, b"bar", 3, 9),
        (b"f\0\0f\0\0f\0\0baz\0\0f\0\0f", 18, b"bar", 3, -1),
        (b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, b"", 0, 0),
        (b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, b"\0\0b", 3, 7),
        (b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, b"r\0\0", 3, 11),
        (b"bar", 3, b"f\0\0f\0\0f\0\0bar\0\0f\0\0f", 18, -1),
        (b"", 0, b"bar", 3, -1),
        (b"", 0, b"", 0, 0),
    ];
    let mut pass = 1;
    for (i, (str_, slen, pat, plen, expected)) in tests.iter().enumerate() {
        let found = crate::htslib_rs::hts::kmemmem(
            str_.as_ptr().cast(),
            *slen,
            pat.as_ptr().cast(),
            *plen,
            std::ptr::null_mut(),
        );
        let loc = if found.is_null() {
            -1
        } else {
            found.cast::<u8>().offset_from(str_.as_ptr())
        };
        if loc != *expected {
            pass = 0;
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kmemmem() test %zd failed - got %lld expected %lld\n".as_ptr(),
                i,
                loc as i64,
                *expected as i64,
            );
        }

        let found = crate::htslib_rs::hts::karp_rabin(
            str_.as_ptr().cast(),
            *slen as usize,
            pat.as_ptr().cast(),
            *plen as usize,
        );
        let loc = if found.is_null() {
            -1
        } else {
            found.cast::<u8>().offset_from(str_.as_ptr())
        };
        if loc != *expected {
            pass = 0;
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"karp_rabin() test %zd failed - got %lld expected %lld\n".as_ptr(),
                i,
                loc as i64,
                *expected as i64,
            );
        }
    }
    if pass != 0 {
        0
    } else {
        -1
    }
}

// original: test_kstrstr (htslib/test/test_kstring.c:638)
pub unsafe fn test_test_kstring_c_638_test_kstrstr() -> c_int {
    let tests = [
        (c"foofoofoobaroofoof".as_ptr(), c"bar".as_ptr(), 9isize),
        (c"foofoofoobazoofoof".as_ptr(), c"bar".as_ptr(), -1),
        (c"foofoofoobaroofoof".as_ptr(), c"".as_ptr(), 0),
        (c"foofoofoobaroofoof".as_ptr(), c"oob".as_ptr(), 7),
        (c"foofoofoobaroofoof".as_ptr(), c"roo".as_ptr(), 11),
        (c"bar".as_ptr(), c"foofoofoobaroofoof".as_ptr(), -1),
        (c"".as_ptr(), c"bar".as_ptr(), -1),
        (c"".as_ptr(), c"".as_ptr(), 0),
    ];
    let mut pass = 1;
    for (i, (str_, pat, expected)) in tests.iter().enumerate() {
        let found = crate::htslib_rs::hts::kstrstr(*str_, *pat, std::ptr::null_mut());
        let loc = if found.is_null() {
            -1
        } else {
            found.offset_from(*str_)
        };
        if loc != *expected {
            pass = 0;
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kstrstr() test %zd failed - got %lld expected %lld\n".as_ptr(),
                i,
                loc as i64,
                *expected as i64,
            );
        }
    }
    if pass != 0 {
        0
    } else {
        -1
    }
}

// original: test_kstrnstr (htslib/test/test_kstring.c:673)
pub unsafe fn test_test_kstring_c_673_test_kstrnstr() -> c_int {
    let tests = [
        (c"foofoofoobaroofoof".as_ptr(), c"bar".as_ptr(), 18, 9isize),
        (c"foofoofoobazoofoof".as_ptr(), c"bar".as_ptr(), 18, -1),
        (c"foofoofoobaroofoof".as_ptr(), c"bar".as_ptr(), 9, -1),
        (c"foofoofoobaroofoof".as_ptr(), c"".as_ptr(), 18, 0),
        (c"bar".as_ptr(), c"foofoofoobaroofoof".as_ptr(), 18, -1),
        (
            b"foofoof\0obaroofoof\0".as_ptr().cast(),
            c"bar".as_ptr(),
            18,
            -1,
        ),
        (c"".as_ptr(), c"bar".as_ptr(), 3, -1),
        (c"".as_ptr(), c"".as_ptr(), 0, 0),
    ];
    let mut pass = 1;
    for (i, (str_, pat, n, expected)) in tests.iter().enumerate() {
        let found = crate::htslib_rs::hts::kstrnstr(*str_, *pat, *n, std::ptr::null_mut());
        let loc = if found.is_null() {
            -1
        } else {
            found.offset_from(*str_)
        };
        if loc != *expected {
            pass = 0;
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"kstrnstr() test %zd failed - got %lld expected %lld\n".as_ptr(),
                i,
                loc as i64,
                *expected as i64,
            );
        }
    }
    if pass != 0 {
        0
    } else {
        -1
    }
}

// original: main (htslib/test/test_kstring.c:709)
pub unsafe fn test_test_kstring_c_709_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut res = libc::EXIT_SUCCESS;
    let mut start = 0i64;
    let mut end = 0i64;
    let mut test: *mut c_char = std::ptr::null_mut();
    let mut verbose = 0;

    loop {
        let opt = libc::getopt(argc, argv, c"e:s:t:v".as_ptr());
        if opt == -1 {
            break;
        }
        match opt as u8 {
            b's' => start = libc::strtoll(optarg, std::ptr::null_mut(), 0) as i64,
            b'e' => end = libc::strtoll(optarg, std::ptr::null_mut(), 0) as i64,
            b't' => test = optarg,
            b'v' => verbose += 1,
            _ => {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Usage : %s [-s <num>] [-e <num>] [-t <test>]\n".as_ptr(),
                    *argv,
                );
                return libc::EXIT_FAILURE;
            }
        }
    }

    if test.is_null() || libc::strcmp(test, c"kroundup_size_t".as_ptr()) == 0 {
        if test_test_kstring_c_45_test_kroundup_size_t(verbose) != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kroundup_signed".as_ptr()) == 0 {
        if test_test_kstring_c_91_test_kroundup_signed(verbose) != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kputuw".as_ptr()) == 0 {
        if test_test_kstring_c_153_test_kputuw(start, end) != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kputw".as_ptr()) == 0 {
        if test_test_kstring_c_219_test_kputw(start, end) != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kputll".as_ptr()) == 0 {
        if test_test_kstring_c_294_test_kputll(start, end) != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kgetline".as_ptr()) == 0 {
        if test_test_kstring_c_375_test_kgetline() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kgetline2".as_ptr()) == 0 {
        if test_test_kstring_c_431_test_kgetline2() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kinsertchar".as_ptr()) == 0 {
        if test_test_kstring_c_458_test_kinsertchar() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kinsertstr".as_ptr()) == 0 {
        if test_test_kstring_c_514_test_kinsertstr() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kmemmem".as_ptr()) == 0 {
        if test_test_kstring_c_586_test_kmemmem() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kstrstr".as_ptr()) == 0 {
        if test_test_kstring_c_638_test_kstrstr() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }
    if test.is_null() || libc::strcmp(test, c"kstrnstr".as_ptr()) == 0 {
        if test_test_kstring_c_673_test_kstrnstr() != 0 {
            res = libc::EXIT_FAILURE;
        }
    }

    res
}
