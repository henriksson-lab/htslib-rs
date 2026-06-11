use crate::htslib_rs::hts::kstring_t;

unsafe extern "C" {
    static mut optarg: *mut u8;
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
pub unsafe fn test_test_kstring_c_45_test_kroundup_size_t(verbose: i32) -> i32 {
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
        print!("kroundup_size_t(0) = 0x{:x}\n", val);
    }
    if val != 0 {
        eprint!("kroundup_size_t(0) produced 0x{:x}, expected 0\n", val);
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
                print!("kroundup_size_t(0x{:x}) = 0x{:x}\n", val_in, val);
            }
            if delta <= 0 {
                if val != expected {
                    eprint!(
                        "kroundup_size_t(0x{:x}) produced 0x{:x}, expected 0x{:x}\n",
                        val_in, val, expected,
                    );
                    ret = -1;
                }
            } else {
                expected = expected.wrapping_mul(2);
                if expected == 0 {
                    expected = expected.wrapping_sub(1);
                }
                if val != expected {
                    eprint!(
                        "kroundup_size_t(0x{:x}) produced 0x{:x}, expected 0x{:x}\n",
                        val_in, val, expected,
                    );
                    ret = -1;
                }
            }
        }
    }
    ret
}

// original: test_kroundup_signed (htslib/test/test_kstring.c:91)
pub unsafe fn test_test_kstring_c_91_test_kroundup_signed(verbose: i32) -> i32 {
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
                print!("kroundup32({}) = {}\n", val_in, val);
            }
            if delta <= 0 {
                if val as u32 != expected {
                    eprint!(
                        "kroundup32({}) produced {}, expected {}\n",
                        val_in, val, expected,
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
                    eprint!(
                        "kroundup32({}) produced {}, expected {}\n",
                        val_in, val, expected,
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
    str_: &mut kstring_t,
    s: u32,
    e: u32,
) -> i32 {
    let mut i = s;
    loop {
        str_.data.clear();
        if crate::htslib_rs::hts::kputuw(i, str_) < 0 {
            libc::perror(c"kputuw".as_ptr());
            return -1;
        }
        let mut c_str = str_.data.clone();
        c_str.push(0);
        if i != libc::strtoul(c_str.as_ptr().cast(), std::ptr::null_mut(), 10) as u32 {
            eprint!(
                "kputuw wrote the wrong value, expected {}, got {}\n",
                i,
                String::from_utf8_lossy(&str_.data),
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
pub unsafe fn test_test_kstring_c_153_test_kputuw(mut start: i64, mut end: i64) -> i32 {
    let mut str_: kstring_t = kstring_t::default();
    str_.data.reserve(2);

    let mut val = 0i64;
    while val < u32::MAX as i64 {
        let s = if val == 0 { 0 } else { val - 5 } as u32;
        let e = (val + 5) as u32;
        if test_test_kstring_c_127_test_kputuw_from_to(&mut str_, s, e) < 0 {
            crate::htslib_rs::hts::ks_release(&mut str_);
            return -1;
        }
        val = if val == 0 { 1 } else { val * 10 };
    }

    if test_test_kstring_c_127_test_kputuw_from_to(&mut str_, u32::MAX - 5, u32::MAX) < 0 {
        crate::htslib_rs::hts::ks_release(&mut str_);
        return -1;
    }

    test_test_kstring_c_40_clamp(&mut start, 0, u32::MAX as i64);
    test_test_kstring_c_40_clamp(&mut end, 0, u32::MAX as i64);
    if test_test_kstring_c_127_test_kputuw_from_to(&mut str_, start as u32, end as u32) < 0 {
        crate::htslib_rs::hts::ks_release(&mut str_);
        return -1;
    }

    crate::htslib_rs::hts::ks_release(&mut str_);
    0
}

// original: test_kputw_from_to (htslib/test/test_kstring.c:193)
pub unsafe fn test_test_kstring_c_193_test_kputw_from_to(
    str_: &mut kstring_t,
    s: i32,
    e: i32,
) -> i32 {
    let mut i = s;
    loop {
        str_.data.clear();
        if crate::htslib_rs::hts::kputw(i, str_) < 0 {
            libc::perror(c"kputw".as_ptr());
            return -1;
        }
        let mut c_str = str_.data.clone();
        c_str.push(0);
        if i != libc::strtol(c_str.as_ptr().cast(), std::ptr::null_mut(), 10) as i32 {
            eprint!(
                "kputw wrote the wrong value, expected {}, got {}\n",
                i,
                String::from_utf8_lossy(&str_.data),
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
pub unsafe fn test_test_kstring_c_219_test_kputw(mut start: i64, mut end: i64) -> i32 {
    let mut str_: kstring_t = kstring_t::default();
    str_.data.reserve(2);

    let mut val = 1i64;
    while val < i32::MAX as i64 {
        let s = if val > 5 { val - 5 } else { 0 } as i32;
        let e = (val + 5) as i32;
        if test_test_kstring_c_193_test_kputw_from_to(&mut str_, s, e) < 0 {
            crate::htslib_rs::hts::ks_release(&mut str_);
            return -1;
        }
        val *= 10;
    }

    val = -1;
    while val > i32::MIN as i64 {
        let s = (val - 5) as i32;
        let e = if val < -5 { val + 5 } else { 0 } as i32;
        if test_test_kstring_c_193_test_kputw_from_to(&mut str_, s, e) < 0 {
            crate::htslib_rs::hts::ks_release(&mut str_);
            return -1;
        }
        val *= 10;
    }

    if test_test_kstring_c_193_test_kputw_from_to(&mut str_, i32::MAX - 5, i32::MAX) < 0
        || test_test_kstring_c_193_test_kputw_from_to(&mut str_, i32::MIN, i32::MIN + 5) < 0
    {
        crate::htslib_rs::hts::ks_release(&mut str_);
        return -1;
    }

    test_test_kstring_c_40_clamp(&mut start, i32::MIN as i64, i32::MAX as i64);
    test_test_kstring_c_40_clamp(&mut end, i32::MIN as i64, i32::MAX as i64);
    if test_test_kstring_c_193_test_kputw_from_to(&mut str_, start as i32, end as i32) < 0 {
        crate::htslib_rs::hts::ks_release(&mut str_);
        return -1;
    }

    crate::htslib_rs::hts::ks_release(&mut str_);
    0
}

// original: test_kputll_from_to (htslib/test/test_kstring.c:268)
pub unsafe fn test_test_kstring_c_268_test_kputll_from_to(
    str_: &mut kstring_t,
    s: i64,
    e: i64,
) -> i32 {
    let mut i = s;
    loop {
        str_.data.clear();
        if crate::htslib_rs::hts::kputll(i, str_) < 0 {
            libc::perror(c"kputll".as_ptr());
            return -1;
        }
        let mut c_str = str_.data.clone();
        c_str.push(0);
        if i != libc::strtoll(c_str.as_ptr().cast(), std::ptr::null_mut(), 10) as i64 {
            eprint!(
                "kputll wrote the wrong value, expected {}, got {}\n",
                i,
                String::from_utf8_lossy(&str_.data),
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
pub unsafe fn test_test_kstring_c_294_test_kputll(mut start: i64, mut end: i64) -> i32 {
    let mut str_: kstring_t = kstring_t::default();
    str_.data.reserve(2);

    let mut val = 1u64;
    while val < i64::MAX as u64 - 5 {
        let s = if val >= 5 { val - 5 } else { val } as i64;
        if test_test_kstring_c_268_test_kputll_from_to(&mut str_, s, val as i64) < 0 {
            crate::htslib_rs::hts::ks_release(&mut str_);
            return -1;
        }
        val *= 10;
    }

    val = 1;
    while val < i64::MAX as u64 - 5 {
        let valm = -(val as i64);
        let s = if valm >= 5 { valm - 5 } else { valm };
        if test_test_kstring_c_268_test_kputll_from_to(&mut str_, s, valm) < 0 {
            crate::htslib_rs::hts::ks_release(&mut str_);
            return -1;
        }
        val *= 10;
    }

    if test_test_kstring_c_268_test_kputll_from_to(&mut str_, i64::MAX - 5, i64::MAX) < 0
        || test_test_kstring_c_268_test_kputll_from_to(&mut str_, i64::MIN, i64::MIN + 5) < 0
    {
        crate::htslib_rs::hts::ks_release(&mut str_);
        return -1;
    }

    test_test_kstring_c_40_clamp(&mut start, i64::MIN, i64::MAX);
    test_test_kstring_c_40_clamp(&mut end, i64::MIN, i64::MAX);
    if test_test_kstring_c_268_test_kputll_from_to(&mut str_, start, end) < 0 {
        crate::htslib_rs::hts::ks_release(&mut str_);
        return -1;
    }

    crate::htslib_rs::hts::ks_release(&mut str_);
    0
}

// original: mock_fgets (htslib/test/test_kstring.c:347)
pub unsafe extern "C" fn test_test_kstring_c_347_mock_fgets(
    str_: *mut u8,
    _num: i32,
    p: *mut (),
) -> *mut u8 {
    let mock_state = p.cast::<i32>();
    *mock_state += 1;
    match *mock_state {
        1 | 4 | 7 => libc::strcpy(str_.cast(), c"ABCD".as_ptr()),
        2 | 3 => libc::strcpy(str_.cast(), c"\n".as_ptr()),
        5 | 6 => libc::strcpy(str_.cast(), c"\r\n".as_ptr()),
        _ => return std::ptr::null_mut(),
    };
    str_
}

// original: test_kgetline (htslib/test/test_kstring.c:375)
pub unsafe fn test_test_kstring_c_375_test_kgetline() -> i32 {
    let mut s: kstring_t = kstring_t::default();
    let mut mock_state = 0;

    crate::htslib_rs::hts::kputs(b"_", &mut s);
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || s.data != b"_ABCD"
        || s.data.len() != 5
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || !s.data.is_empty()
        || s.data.len() != 0
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || s.data != b"ABCD"
        || s.data.len() != 4
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || !s.data.is_empty()
        || s.data.len() != 0
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || s.data != b"ABCD"
        || s.data.len() != 4
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline(
        &mut s,
        Some(test_test_kstring_c_347_mock_fgets),
        (&mut mock_state as *mut i32).cast(),
    ) != libc::EOF
        || s.data.len() != 0
    {
        return -1;
    }

    crate::htslib_rs::hts::ks_free(&mut s);
    libc::EXIT_SUCCESS
}

// original: mock_fgets2 (htslib/test/test_kstring.c:403)
pub unsafe extern "C" fn test_test_kstring_c_403_mock_fgets2(
    str_: *mut std::ffi::c_char,
    _num: usize,
    p: *mut std::ffi::c_void,
) -> isize {
    let mock_state = p.cast::<i32>();
    *mock_state += 1;
    match *mock_state {
        1 | 4 | 7 => libc::strcpy(str_.cast(), c"ABCD".as_ptr()),
        2 | 3 => libc::strcpy(str_.cast(), c"\n".as_ptr()),
        5 | 6 => libc::strcpy(str_.cast(), c"\r\n".as_ptr()),
        _ => return 0,
    };
    libc::strlen(str_.cast()) as isize
}

// original: test_kgetline2 (htslib/test/test_kstring.c:431)
pub unsafe fn test_test_kstring_c_431_test_kgetline2() -> i32 {
    let mut s: kstring_t = kstring_t::default();
    let mut mock_state = 0;

    crate::htslib_rs::hts::kputs(b"_", &mut s);
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || s.data != b"_ABCD"
        || s.data.len() != 5
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || !s.data.is_empty()
        || s.data.len() != 0
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || s.data != b"ABCD"
        || s.data.len() != 4
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || !s.data.is_empty()
        || s.data.len() != 0
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut i32).cast(),
    ) != 0
        || s.data != b"ABCD"
        || s.data.len() != 4
    {
        return -1;
    }
    s.data.clear();
    if crate::htslib_rs::hts::kgetline2(
        &mut s,
        Some(test_test_kstring_c_403_mock_fgets2),
        (&mut mock_state as *mut i32).cast(),
    ) != libc::EOF
        || s.data.len() != 0
    {
        return -1;
    }

    crate::htslib_rs::hts::ks_free(&mut s);
    libc::EXIT_SUCCESS
}

// original: test_kinsertchar (htslib/test/test_kstring.c:458)
pub unsafe fn test_test_kstring_c_458_test_kinsertchar() -> i32 {
    let expected: [&[u8]; 7] = [
        b"",
        b"X0123",
        b"0X123",
        b"01X23",
        b"012X3",
        b"0123X",
        b"",
    ];

    for i in -1i32..6 {
        let mut s: kstring_t = kstring_t::default();
        crate::htslib_rs::hts::kputs(b"0123", &mut s);
        if crate::htslib_rs::hts::kinsert_char(b'X' as std::ffi::c_char, i as usize, &mut s) < 0 {
            if !(0..=4).contains(&i) {
                crate::htslib_rs::hts::ks_free(&mut s);
                continue;
            }
            eprint!("kinsert_char failed\n");
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        let exp_bytes = expected[(i + 1) as usize];
        if s.data != exp_bytes {
            eprint!("kinsert_char comparison failed\n");
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        crate::htslib_rs::hts::ks_free(&mut s);
    }

    let mut t: kstring_t = kstring_t::default();
    let mut res: kstring_t = kstring_t::default();
    for i in 0..7 {
        crate::htslib_rs::hts::kputc(b'A' as i32 + i, &mut res);
        let t_len = t.data.len();
        if crate::htslib_rs::hts::kinsert_char((b'A' as i32 + i) as std::ffi::c_char, t_len, &mut t) < 0 {
            eprint!("kinsert_char failed in realloc\n");
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
        if t.data != res.data {
            eprint!("kinsert_char realloc comparison failed in realloc\n");
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
pub unsafe fn test_test_kstring_c_514_test_kinsertstr() -> i32 {
    let expected: [&[u8]; 7] = [
        b"",
        b"XYZ0123",
        b"0XYZ123",
        b"01XYZ23",
        b"012XYZ3",
        b"0123XYZ",
        b"",
    ];

    for i in -1i32..6 {
        let mut s: kstring_t = kstring_t::default();
        crate::htslib_rs::hts::kputs(b"0123", &mut s);
        if crate::htslib_rs::hts::kinsert_str(b"XYZ", i as usize, &mut s) < 0 {
            if !(0..=4).contains(&i) {
                crate::htslib_rs::hts::ks_free(&mut s);
                continue;
            }
            eprint!("kinsert_str failed\n");
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        let exp_bytes = expected[(i + 1) as usize];
        if s.data != exp_bytes {
            eprint!("kinsert_str comparison failed\n");
            crate::htslib_rs::hts::ks_free(&mut s);
            return -1;
        }
        crate::htslib_rs::hts::ks_free(&mut s);
    }

    let mut t: kstring_t = kstring_t::default();
    let mut res: kstring_t = kstring_t::default();
    for i in 0..15 {
        let ch = b'A' + i as u8;
        let one = [ch];
        crate::htslib_rs::hts::kputs(&one, &mut res);
        let t_len = t.data.len();
        if crate::htslib_rs::hts::kinsert_str(&one, t_len, &mut t) < 0 {
            eprint!("kinsert_str failed in realloc\n");
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
        if t.data != res.data {
            eprint!("kinsert_str realloc comparison failed in realloc\n");
            crate::htslib_rs::hts::ks_free(&mut res);
            crate::htslib_rs::hts::ks_free(&mut t);
            return -1;
        }
    }

    crate::htslib_rs::hts::ks_free(&mut t);
    if crate::htslib_rs::hts::kinsert_str(b"", 1, &mut t) == 0 {
        eprint!("kinsert_str empty ins to invalid pos succeeded\n");
        crate::htslib_rs::hts::ks_free(&mut res);
        return -1;
    } else if crate::htslib_rs::hts::kinsert_str(b"", 0, &mut t) != 0 || t.data.len() != 0 {
        eprint!("kinsert_str empty insertion failed\n");
        crate::htslib_rs::hts::ks_free(&mut res);
        return -1;
    }

    let old_len = res.data.len();
    if crate::htslib_rs::hts::kinsert_str(b"", 1, &mut res) != 0 || old_len != res.data.len() {
        eprint!("kinsert_str empty ins to valid pos failed\n");
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
pub unsafe fn test_test_kstring_c_586_test_kmemmem() -> i32 {
    type KmemmemTest<'a> = (&'a [u8], i32, &'a [u8], i32, isize);

    let tests: &[KmemmemTest<'_>] = &[
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
            &str_[..*slen as usize],
            &pat[..*plen as usize],
            None,
        );
        let loc = match found {
            None => -1,
            Some(off) => off as isize,
        };
        if loc != *expected {
            pass = 0;
            eprint!(
                "kmemmem() test {} failed - got {} expected {}\n",
                i, loc as i64, *expected as i64,
            );
        }

        let found = crate::htslib_rs::hts::karp_rabin(
            &str_[..*slen as usize],
            &pat[..*plen as usize],
        );
        let loc = match found {
            None => -1,
            Some(off) => off as isize,
        };
        if loc != *expected {
            pass = 0;
            eprint!(
                "karp_rabin() test {} failed - got {} expected {}\n",
                i, loc as i64, *expected as i64,
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
pub unsafe fn test_test_kstring_c_638_test_kstrstr() -> i32 {
    let tests: [(&[u8], &[u8], isize); 8] = [
        (b"foofoofoobaroofoof", b"bar", 9isize),
        (b"foofoofoobazoofoof", b"bar", -1),
        (b"foofoofoobaroofoof", b"", 0),
        (b"foofoofoobaroofoof", b"oob", 7),
        (b"foofoofoobaroofoof", b"roo", 11),
        (b"bar", b"foofoofoobaroofoof", -1),
        (b"", b"bar", -1),
        (b"", b"", 0),
    ];
    let mut pass = 1;
    for (i, (str_, pat, expected)) in tests.iter().enumerate() {
        let str_bytes = *str_;
        let pat_bytes = *pat;
        let found = crate::htslib_rs::hts::kstrstr(str_bytes, pat_bytes, None);
        let loc = match found {
            None => -1,
            Some(off) => off as isize,
        };
        if loc != *expected {
            pass = 0;
            eprint!(
                "kstrstr() test {} failed - got {} expected {}\n",
                i, loc as i64, *expected as i64,
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
pub unsafe fn test_test_kstring_c_673_test_kstrnstr() -> i32 {
    const STR_WITH_EMBEDDED_NUL: [u8; 19] = [
        b'f', b'o', b'o', b'f', b'o', b'o', b'f', 0, b'o', b'b', b'a', b'r', b'o', b'o', b'f',
        b'o', b'o', b'f', 0,
    ];
    let tests: [(&[u8], &[u8], usize, isize); 8] = [
        (b"foofoofoobaroofoof", b"bar", 18, 9isize),
        (b"foofoofoobazoofoof", b"bar", 18, -1),
        (b"foofoofoobaroofoof", b"bar", 9, -1),
        (b"foofoofoobaroofoof", b"", 18, 0),
        (b"bar", b"foofoofoobaroofoof", 18, -1),
        (
            &STR_WITH_EMBEDDED_NUL,
            b"bar",
            18,
            -1,
        ),
        (b"", b"bar", 3, -1),
        (b"", b"", 0, 0),
    ];
    let mut pass = 1;
    for (i, (str_, pat, n, expected)) in tests.iter().enumerate() {
        let nn = *n;
        let str_bytes = *str_;
        let pat_bytes = *pat;
        let found = crate::htslib_rs::hts::kstrnstr(str_bytes, pat_bytes, nn, None);
        let loc = match found {
            None => -1,
            Some(off) => off as isize,
        };
        if loc != *expected {
            pass = 0;
            eprint!(
                "kstrnstr() test {} failed - got {} expected {}\n",
                i, loc as i64, *expected as i64,
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
pub unsafe fn test_test_kstring_c_709_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut res = libc::EXIT_SUCCESS;
    let mut start = 0i64;
    let mut end = 0i64;
    let mut test: *mut u8 = std::ptr::null_mut();
    let mut verbose = 0;

    loop {
        let opt = libc::getopt(argc, argv.cast(), c"e:s:t:v".as_ptr());
        if opt == -1 {
            break;
        }
        match opt as u8 {
            b's' => start = libc::strtoll(optarg.cast(), std::ptr::null_mut(), 0) as i64,
            b'e' => end = libc::strtoll(optarg.cast(), std::ptr::null_mut(), 0) as i64,
            b't' => test = optarg,
            b'v' => verbose += 1,
            _ => {
                eprint!(
                    "Usage : {} [-s <num>] [-e <num>] [-t <test>]\n",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr((*argv).cast()).to_bytes()),
                );
                return libc::EXIT_FAILURE;
            }
        }
    }

    if (test.is_null() || libc::strcmp(test.cast(), c"kroundup_size_t".as_ptr()) == 0)
        && test_test_kstring_c_45_test_kroundup_size_t(verbose) != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kroundup_signed".as_ptr()) == 0)
        && test_test_kstring_c_91_test_kroundup_signed(verbose) != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kputuw".as_ptr()) == 0)
        && test_test_kstring_c_153_test_kputuw(start, end) != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kputw".as_ptr()) == 0)
        && test_test_kstring_c_219_test_kputw(start, end) != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kputll".as_ptr()) == 0)
        && test_test_kstring_c_294_test_kputll(start, end) != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kgetline".as_ptr()) == 0)
        && test_test_kstring_c_375_test_kgetline() != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kgetline2".as_ptr()) == 0)
        && test_test_kstring_c_431_test_kgetline2() != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kinsertchar".as_ptr()) == 0)
        && test_test_kstring_c_458_test_kinsertchar() != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kinsertstr".as_ptr()) == 0)
        && test_test_kstring_c_514_test_kinsertstr() != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kmemmem".as_ptr()) == 0)
        && test_test_kstring_c_586_test_kmemmem() != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kstrstr".as_ptr()) == 0)
        && test_test_kstring_c_638_test_kstrstr() != 0
    {
        res = libc::EXIT_FAILURE;
    }
    if (test.is_null() || libc::strcmp(test.cast(), c"kstrnstr".as_ptr()) == 0)
        && test_test_kstring_c_673_test_kstrnstr() != 0
    {
        res = libc::EXIT_FAILURE;
    }

    res
}
