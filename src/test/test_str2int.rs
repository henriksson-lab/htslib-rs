// original: check_str2int (htslib/test/test_str2int.c:41)
pub unsafe fn test_test_str2int_c_41_check_str2int(verbose: i32) -> i32 {
    let mut buffer = [0u8; 64];
    let mut end: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut failed = 0;
    let sentinel = b'#';

    for i in 1..64 {
        let num = (1u64 << i) - 1;
        let start_offset: i64 = if i < 5 { -(1_i64 << (i - 1)) } else { -16 };
        for offset in start_offset..=30 {
            let efail = (offset > 0) as i32;
            let input = num.wrapping_add(offset as u64);
            libc::snprintf(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                c"%llu%c".as_ptr(),
                input,
                sentinel as i32,
            );

            let uval =
                crate::htslib_rs::hts::hts_str2uint(buffer.as_ptr().cast(), &mut end, i, &mut failed);
            let expected = if efail == 0 { input } else { num };
            if failed != efail || uval != expected || *end as u8 != sentinel {
                eprintln!(
                    "hts_str2uint failed: {} bit {} {} '{}' {} ({})",
                    i,
                    String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                    uval,
                    *end as u8 as char,
                    failed,
                    efail,
                );
                return -1;
            } else if verbose != 0 {
                eprintln!(
                    "hts_str2uint OK: {} bit {} {} '{}' {} ({})",
                    i,
                    String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                    uval,
                    *end as u8 as char,
                    failed,
                    efail,
                );
            }
            failed = 0;
        }

        for offset in start_offset..=30 {
            let efail = (offset > 0) as i32;
            let input = num.wrapping_add(offset as u64);
            libc::snprintf(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                c"%llu%c".as_ptr(),
                input,
                sentinel as i32,
            );

            let val =
                crate::htslib_rs::hts::hts_str2int(buffer.as_ptr().cast(), &mut end, i + 1, &mut failed);
            let expected = if efail == 0 { input as i64 } else { num as i64 };
            if failed != efail || val != expected || *end as u8 != sentinel {
                eprintln!(
                    "hts_str2int  failed: {} bit {} {} '{}' {} ({})",
                    i + 1,
                    String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                    val,
                    *end as u8 as char,
                    failed,
                    efail,
                );
                return -1;
            } else if verbose != 0 {
                eprintln!(
                    "hts_str2int  OK: {} bit {} {} '{}' {} ({})",
                    i + 1,
                    String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                    val,
                    *end as u8 as char,
                    failed,
                    efail,
                );
            }
            failed = 0;
        }

        for offset in start_offset..=30 {
            let efail = (offset > 0) as i32;
            let input = num.wrapping_add(offset as u64).wrapping_add(1);
            libc::snprintf(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                c"-%llu%c".as_ptr(),
                input,
                sentinel as i32,
            );

            let val =
                crate::htslib_rs::hts::hts_str2int(buffer.as_ptr().cast(), &mut end, i + 1, &mut failed);
            let expected = if efail == 0 {
                input
            } else {
                num.wrapping_add(1)
            };
            if failed != efail || (val as u64).wrapping_neg() != expected || *end as u8 != sentinel {
                eprintln!(
                    "hts_str2int  failed: {} bit {} {} '{}' {} ({})",
                    i + 1,
                    String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                    val,
                    *end as u8 as char,
                    failed,
                    efail,
                );
                return -1;
            } else if verbose != 0 {
                eprintln!(
                    "hts_str2int  OK: {} bit {} {} '{}' {} ({})",
                    i + 1,
                    String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                    val,
                    *end as u8 as char,
                    failed,
                    efail,
                );
            }
            failed = 0;
        }
    }

    for offset in 0..=999 {
        let efail = (offset > 615) as i32;
        libc::snprintf(
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            c"18446744073709551%03d%c".as_ptr(),
            offset,
            sentinel as i32,
        );
        let uval = crate::htslib_rs::hts::hts_str2uint(buffer.as_ptr().cast(), &mut end, 64, &mut failed);
        let expected = if efail != 0 {
            u64::MAX
        } else {
            18_446_744_073_709_551_000u64 + offset as u64
        };
        if failed != efail || uval != expected || *end as u8 != sentinel {
            eprintln!(
                "hts_str2uint failed: 64 bit {} {} '{}' {} ({})",
                String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                uval,
                *end as u8 as char,
                failed,
                efail,
            );
            return -1;
        } else if verbose != 0 {
            eprintln!(
                "hts_str2uint OK: 64 bit {} {} '{}' {} ({})",
                String::from_utf8_lossy(&buffer[..buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len())]),
                uval,
                *end as u8 as char,
                failed,
                efail,
            );
        }
    }

    0
}

// original: check_strprint2 (htslib/test/test_str2int.c:141)
pub unsafe fn test_test_str2int_c_141_check_strprint2(
    verbose: i32,
    str_: *const u8,
    len: usize,
    destlen: usize,
    quote: u8,
    expect: *const u8,
) -> i32 {
    let mut buf = [0u8; 100];
    let slen = if len == usize::MAX {
        libc::strlen(str_.cast())
    } else {
        len
    };
    let s = std::slice::from_raw_parts(str_, slen);
    crate::htslib_rs::hts::hts_strprint(&mut buf[..destlen], quote, s);
    if libc::strcmp(buf.as_ptr().cast(), expect.cast()) != 0 {
        let got_len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len()).min(destlen);
        let expect_slice =
            std::slice::from_raw_parts(expect, libc::strlen(expect.cast()));
        eprintln!(
            "hts_strprint failed: length {}: got \"{}\", expected \"{}\"",
            destlen,
            String::from_utf8_lossy(&buf[..got_len]),
            String::from_utf8_lossy(expect_slice),
        );
        -1
    } else {
        if verbose != 0 {
            let expect_slice =
                std::slice::from_raw_parts(expect, libc::strlen(expect.cast()));
            eprintln!(
                "hts_strprint OK: length {}: got \"{}\"",
                destlen,
                String::from_utf8_lossy(expect_slice),
            );
        }
        0
    }
}

// original: check_strprint1 (htslib/test/test_str2int.c:158)
pub unsafe fn test_test_str2int_c_158_check_strprint1(
    v: i32,
    str_: *const u8,
    destlen: usize,
    expect: *const u8,
) -> i32 {
    test_test_str2int_c_141_check_strprint2(v, str_, usize::MAX, destlen, 0, expect)
}

// original: check_strprintq (htslib/test/test_str2int.c:163)
pub unsafe fn test_test_str2int_c_163_check_strprintq(
    v: i32,
    str_: *const u8,
    destlen: usize,
    quote: u8,
    expect: *const u8,
) -> i32 {
    test_test_str2int_c_141_check_strprint2(v, str_, usize::MAX, destlen, quote, expect)
}

// original: check_strprint (htslib/test/test_str2int.c:169)
pub unsafe fn test_test_str2int_c_169_check_strprint(v: i32) -> i32 {
    let mut res = 0;

    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr().cast(), 9, c"chr10".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr().cast(), 6, c"chr10".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr().cast(), 5, c"c...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"chr10".as_ptr().cast(), 4, c"...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        c"tab\twxyz".as_ptr().cast(),
        10,
        c"tab\\twxyz".as_ptr().cast(),
    );
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr().cast(), 9, c"tab\\t...".as_ptr().cast());
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr().cast(), 8, c"tab\\...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr().cast(), 7, c"tab...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"tab\twxyz".as_ptr().cast(), 6, c"ta...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"\xAB".as_ptr().cast(), 5, c"\\xAB".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"\xAB".as_ptr().cast(), 4, c"...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        c"hello\xFF".as_ptr().cast(),
        40,
        c"hello\\xFF".as_ptr().cast(),
    );
    res |= test_test_str2int_c_158_check_strprint1(
        v,
        c"hello\xFF".as_ptr().cast(),
        10,
        c"hello\\xFF".as_ptr().cast(),
    );
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"hello\xFF".as_ptr().cast(), 9, c"hello...".as_ptr().cast());
    res |=
        test_test_str2int_c_158_check_strprint1(v, c"hello\t".as_ptr().cast(), 40, c"hello\\t".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"hello\t".as_ptr().cast(), 8, c"hello\\t".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"hello\t".as_ptr().cast(), 7, c"hel...".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"\t".as_ptr().cast(), 40, c"\\t".as_ptr().cast());
    res |= test_test_str2int_c_158_check_strprint1(v, c"".as_ptr().cast(), 40, c"".as_ptr().cast());

    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr().cast(),
        9,
        b'\'',
        c"'chr10'".as_ptr().cast(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr().cast(),
        8,
        b'\'',
        c"'chr10'".as_ptr().cast(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr().cast(),
        7,
        b'\'',
        c"'c'...".as_ptr().cast(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"chr10".as_ptr().cast(),
        6,
        b'\'',
        c"''...".as_ptr().cast(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"quo'wxyz".as_ptr().cast(),
        12,
        b'\'',
        c"'quo\\'wxyz'".as_ptr().cast(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"quo'wxyz".as_ptr().cast(),
        11,
        b'\'',
        c"'quo\\''...".as_ptr().cast(),
    );
    res |= test_test_str2int_c_163_check_strprintq(
        v,
        c"quo'wxyz".as_ptr().cast(),
        10,
        b'\'',
        c"'quo\\'...".as_ptr().cast(),
    );

    let nul = b"foo\0bar\0";
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr(),
        usize::MAX,
        10,
        0,
        c"foo".as_ptr().cast(),
    );
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr(),
        7,
        10,
        0,
        c"foo\\0bar".as_ptr().cast(),
    );
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr(),
        7,
        9,
        0,
        c"foo\\0bar".as_ptr().cast(),
    );
    res |= test_test_str2int_c_141_check_strprint2(
        v,
        nul.as_ptr(),
        7,
        8,
        0,
        c"foo\\...".as_ptr().cast(),
    );

    res
}

// original: main (htslib/test/test_str2int.c:208)
pub unsafe fn test_test_str2int_c_208_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut verbose = 0;
    loop {
        let opt = libc::getopt(argc, argv.cast(), c"v".as_ptr());
        if opt == -1 {
            break;
        }
        match opt as u8 {
            b'v' => verbose = 1,
            _ => {
                let prog = std::slice::from_raw_parts(*argv, libc::strlen((*argv).cast()));
                eprintln!("Usage: {} [-v]", String::from_utf8_lossy(prog));
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
