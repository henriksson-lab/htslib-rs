use std::ffi::{c_char, c_int};

// original: check_str2int (htslib/test/test_str2int.c:41)
pub unsafe fn test_test_str2int_c_41_check_str2int(verbose: c_int) -> c_int {
    let mut buffer = [0 as c_char; 64];
    let mut end: *mut c_char = std::ptr::null_mut();
    let mut failed = 0;
    let sentinel = b'#' as c_char;

    for i in 1..64 {
        let num = (1u64 << i) - 1;
        let start_offset: i64 = if i < 5 { -(1_i64 << (i - 1)) } else { -16 };
        for offset in start_offset..=30 {
            let efail = (offset > 0) as c_int;
            let input = num.wrapping_add(offset as u64);
            libc::snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                c"%llu%c".as_ptr(),
                input,
                sentinel as c_int,
            );

            let uval =
                crate::htslib_rs::hts::hts_str2uint(buffer.as_ptr(), &mut end, i, &mut failed);
            let expected = if efail == 0 { input } else { num };
            if failed != efail || uval != expected || *end != sentinel {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"hts_str2uint failed: %d bit %s %llu '%c' %d (%d)\n".as_ptr(),
                    i,
                    buffer.as_ptr(),
                    uval,
                    *end as c_int,
                    failed,
                    efail,
                );
                return -1;
            } else if verbose != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"hts_str2uint OK: %d bit %s %llu '%c' %d (%d)\n".as_ptr(),
                    i,
                    buffer.as_ptr(),
                    uval,
                    *end as c_int,
                    failed,
                    efail,
                );
            }
            failed = 0;
        }

        for offset in start_offset..=30 {
            let efail = (offset > 0) as c_int;
            let input = num.wrapping_add(offset as u64);
            libc::snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                c"%llu%c".as_ptr(),
                input,
                sentinel as c_int,
            );

            let val =
                crate::htslib_rs::hts::hts_str2int(buffer.as_ptr(), &mut end, i + 1, &mut failed);
            let expected = if efail == 0 { input as i64 } else { num as i64 };
            if failed != efail || val != expected || *end != sentinel {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"hts_str2int  failed: %d bit %s %lld '%c' %d (%d)\n".as_ptr(),
                    i + 1,
                    buffer.as_ptr(),
                    val,
                    *end as c_int,
                    failed,
                    efail,
                );
                return -1;
            } else if verbose != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"hts_str2int  OK: %d bit %s %lld '%c' %d (%d)\n".as_ptr(),
                    i + 1,
                    buffer.as_ptr(),
                    val,
                    *end as c_int,
                    failed,
                    efail,
                );
            }
            failed = 0;
        }

        for offset in start_offset..=30 {
            let efail = (offset > 0) as c_int;
            let input = num.wrapping_add(offset as u64).wrapping_add(1);
            libc::snprintf(
                buffer.as_mut_ptr(),
                buffer.len(),
                c"-%llu%c".as_ptr(),
                input,
                sentinel as c_int,
            );

            let val =
                crate::htslib_rs::hts::hts_str2int(buffer.as_ptr(), &mut end, i + 1, &mut failed);
            let expected = if efail == 0 {
                input
            } else {
                num.wrapping_add(1)
            };
            if failed != efail || (val as u64).wrapping_neg() != expected || *end != sentinel {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"hts_str2int  failed: %d bit %s %lld '%c' %d (%d)\n".as_ptr(),
                    i + 1,
                    buffer.as_ptr(),
                    val,
                    *end as c_int,
                    failed,
                    efail,
                );
                return -1;
            } else if verbose != 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"hts_str2int  OK: %d bit %s %lld '%c' %d (%d)\n".as_ptr(),
                    i + 1,
                    buffer.as_ptr(),
                    val,
                    *end as c_int,
                    failed,
                    efail,
                );
            }
            failed = 0;
        }
    }

    for offset in 0..=999 {
        let efail = (offset > 615) as c_int;
        libc::snprintf(
            buffer.as_mut_ptr(),
            buffer.len(),
            c"18446744073709551%03d%c".as_ptr(),
            offset,
            sentinel as c_int,
        );
        let uval = crate::htslib_rs::hts::hts_str2uint(buffer.as_ptr(), &mut end, 64, &mut failed);
        let expected = if efail != 0 {
            u64::MAX
        } else {
            18_446_744_073_709_551_000u64 + offset as u64
        };
        if failed != efail || uval != expected || *end != sentinel {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"hts_str2uint failed: 64 bit %s %llu '%c' %d (%d)\n".as_ptr(),
                buffer.as_ptr(),
                uval,
                *end as c_int,
                failed,
                efail,
            );
            return -1;
        } else if verbose != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"hts_str2uint OK: 64 bit %s %llu '%c' %d (%d)\n".as_ptr(),
                buffer.as_ptr(),
                uval,
                *end as c_int,
                failed,
                efail,
            );
        }
    }

    0
}

// original: check_strprint2 (htslib/test/test_str2int.c:141)
pub unsafe fn test_test_str2int_c_141_check_strprint2(
    verbose: c_int,
    str_: *const c_char,
    len: usize,
    destlen: usize,
    quote: c_char,
    expect: *const c_char,
) -> c_int {
    let mut buf = [0 as c_char; 100];
    crate::htslib_rs::hts::hts_strprint(buf.as_mut_ptr(), destlen, quote, str_, len);
    if libc::strcmp(buf.as_ptr(), expect) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"hts_strprint failed: length %zu: got \"%.*s\", expected \"%s\"\n".as_ptr(),
            destlen,
            destlen as c_int,
            buf.as_ptr(),
            expect,
        );
        -1
    } else {
        if verbose != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"hts_strprint OK: length %zu: got \"%s\"\n".as_ptr(),
                destlen,
                expect,
            );
        }
        0
    }
}

// original: check_strprint1 (htslib/test/test_str2int.c:158)
pub unsafe fn test_test_str2int_c_158_check_strprint1(
    v: c_int,
    str_: *const c_char,
    destlen: usize,
    expect: *const c_char,
) -> c_int {
    test_test_str2int_c_141_check_strprint2(v, str_, usize::MAX, destlen, 0, expect)
}

// original: check_strprintq (htslib/test/test_str2int.c:163)
pub unsafe fn test_test_str2int_c_163_check_strprintq(
    v: c_int,
    str_: *const c_char,
    destlen: usize,
    quote: c_char,
    expect: *const c_char,
) -> c_int {
    test_test_str2int_c_141_check_strprint2(v, str_, usize::MAX, destlen, quote, expect)
}

// original: check_strprint (htslib/test/test_str2int.c:169)
pub unsafe fn test_test_str2int_c_169_check_strprint(v: c_int) -> c_int {
    let mut res = 0;

    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr(), 9, c"chr10".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr(), 6, c"chr10".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr(), 5, c"c...".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr(), 4, c"...".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        c"tab\twxyz".as_ptr(),
        10,
        c"tab\\twxyz".as_ptr(),
    );
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr(), 9, c"tab\\t...".as_ptr());
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr(), 8, c"tab\\...".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr(), 7, c"tab...".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr(), 6, c"ta...".as_ptr());
    res |=
        test_test_str2int_c_158_check_strprint1(v, b"\xab\0".as_ptr().cast(), 5, c"\\xAB".as_ptr());
    res |=
        test_test_str2int_c_158_check_strprint1(v, b"\xab\0".as_ptr().cast(), 4, c"...".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        b"hello\xff\0".as_ptr().cast(),
        40,
        c"hello\\xFF".as_ptr(),
    );
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        b"hello\xff\0".as_ptr().cast(),
        10,
        c"hello\\xFF".as_ptr(),
    );
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        b"hello\xff\0".as_ptr().cast(),
        9,
        c"hello...".as_ptr(),
    );
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"hello\t".as_ptr(), 40, c"hello\\t".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"hello\t".as_ptr(), 8, c"hello\\t".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"hello\t".as_ptr(), 7, c"hel...".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"\t".as_ptr(), 40, c"\\t".as_ptr());
    res |= test_test_str2int_c_158_check_strprint1(v, c"".as_ptr(), 40, c"".as_ptr());

    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr(),
        9,
        b'\'' as c_char,
        c"'chr10'".as_ptr(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr(),
        8,
        b'\'' as c_char,
        c"'chr10'".as_ptr(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr(),
        7,
        b'\'' as c_char,
        c"'c'...".as_ptr(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr(),
        6,
        b'\'' as c_char,
        c"''...".as_ptr(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"quo'wxyz".as_ptr(),
        12,
        b'\'' as c_char,
        c"'quo\\'wxyz'".as_ptr(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"quo'wxyz".as_ptr(),
        11,
        b'\'' as c_char,
        c"'quo\\''...".as_ptr(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"quo'wxyz".as_ptr(),
        10,
        b'\'' as c_char,
        c"'quo\\'...".as_ptr(),
    );

    let nul = b"foo\0bar\0";
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr().cast(),
        usize::MAX,
        10,
        0,
        c"foo".as_ptr(),
    );
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr().cast(),
        7,
        10,
        0,
        c"foo\\0bar".as_ptr(),
    );
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr().cast(),
        7,
        9,
        0,
        c"foo\\0bar".as_ptr(),
    );
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr().cast(),
        7,
        8,
        0,
        c"foo\\...".as_ptr(),
    );

    res
}

// original: main (htslib/test/test_str2int.c:208)
pub unsafe fn test_test_str2int_c_208_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut verbose = 0;
    loop {
        let opt = libc::getopt(argc, argv, c"v".as_ptr());
        if opt == -1 {
            break;
        }
        match opt as u8 {
            b'v' => verbose = 1,
            _ => {
                libc::fprintf(hts_sys::stderr.cast(), c"Usage: %s [-v]\n".as_ptr(), *argv);
                return libc::EXIT_FAILURE;
            }
        }
    }

    let mut res = test_test_str2int_c_41_check_str2int(verbose);
    res |= test_test_str2int_c_169_check_strprint(verbose);
    if res != 0 {
        libc::EXIT_FAILURE
    } else {
        libc::EXIT_SUCCESS
    }
}
