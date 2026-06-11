use crate::htslib_rs::{hts, sam};

// original: reg_expected (htslib/test/test-parse-reg.c:50)
pub unsafe fn test_test_parse_reg_c_50_reg_expected(
    hdr: *mut sam::sam_hdr_t,
    reg: *const u8,
    flags: i32,
    reg_exp: *const u8,
    tid_exp: i32,
    beg_exp: hts::hts_pos_t,
    end_exp: hts::hts_pos_t,
) {
    let mut tid_out = -1;
    let mut beg_out: hts::hts_pos_t = -1;
    let mut end_out: hts::hts_pos_t = -1;

    let reg_out = sam::sam_parse_region(hdr, reg, &mut tid_out, &mut beg_out, &mut end_out, flags);
    let has_reg_out = !reg_out.is_null();
    let has_reg_exp = !reg_exp.is_null();

    if (has_reg_out != has_reg_exp)
        || (has_reg_out && has_reg_exp && libc::strcmp(reg_out.cast(), reg_exp.cast()) != 0)
        || (has_reg_exp && tid_out != tid_exp)
        || (has_reg_exp && beg_out != beg_exp)
        || (has_reg_exp && end_out != end_exp)
    {
        let reg_str = String::from_utf8_lossy(std::ffi::CStr::from_ptr(reg.cast()).to_bytes());
        let exp_str = if has_reg_exp {
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(reg_exp.cast()).to_bytes()).into_owned()
        } else {
            "(null)".to_string()
        };
        let out_str = if has_reg_out {
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(reg_out.cast()).to_bytes()).into_owned()
        } else {
            "(null)".to_string()
        };
        eprintln!(
            "Parsing \"{}\" expected return \"{}\", {}:{}-{}, but got \"{}\", {}:{}-{}",
            reg_str, exp_str, tid_exp, beg_exp, end_exp, out_str, tid_out, beg_out, end_out,
        );
        libc::exit(1);
    }
}

// original: reg_test (htslib/test/test-parse-reg.c:72)
pub unsafe fn test_test_parse_reg_c_72_reg_test(fn_: *const u8) -> i32 {
    let fp = hts::hts_open(fn_.cast(), c"r".as_ptr().cast());
    if fp.is_null() {
        return 1;
    }

    let hdr = sam::sam_hdr_read(fp);
    if hdr.is_null() {
        return 1;
    }

    // 0 chr1
    // 1 chr1:100
    // 2 chr1:100-200
    // 3 chr2:100-200
    // 4 chr3
    // 5 chr1,chr3

    // Check range extensions.
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        49,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50".as_ptr().cast(),
        hts::HTS_PARSE_ONE_COORD,
        c"".as_ptr().cast(),
        0,
        49,
        50,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50-100".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        49,
        100,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50-".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        49,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:-50".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        0,
        50,
    );

    // Check quoting
    eprint!("Expected error: ");
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:100-200".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    ); // ambiguous
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1}:100-200".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1:100-200}".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        2,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1:100-200}:100-200".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        2,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr2:100-200}:100-200".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        3,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr2:100-200:100-200".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        3,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr2:100-200".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        3,
        0,
        hts::HTS_POS_MAX,
    );

    // Check numerics
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1000-1500".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1,000-1,500".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1k-1.5K".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1e3-1.5e3".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1e3-15e2".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        4,
        999,
        1500,
    );

    // Check list mode
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1,chr3".as_ptr().cast(),
        hts::HTS_PARSE_LIST,
        c"chr3".as_ptr().cast(),
        0,
        0,
        hts::HTS_POS_MAX,
    );
    eprint!("Expected error: ");
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:100-200,chr3".as_ptr().cast(),
        hts::HTS_PARSE_LIST,
        std::ptr::null(),
        0,
        0,
        0,
    ); // ambiguous
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1,chr3}".as_ptr().cast(),
        hts::HTS_PARSE_LIST,
        c"".as_ptr().cast(),
        5,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1,chr3},chr1".as_ptr().cast(),
        hts::HTS_PARSE_LIST,
        c"chr1".as_ptr().cast(),
        5,
        0,
        hts::HTS_POS_MAX,
    );
    // incorrect usage; first reg is valid (but not what user expects).
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1,000-1,500".as_ptr().cast(),
        hts::HTS_PARSE_LIST | hts::HTS_PARSE_ONE_COORD,
        c"000-1,500".as_ptr().cast(),
        4,
        0,
        1,
    );

    // More expected failures
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr2".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1,".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    );
    eprint!("Expected error: ");
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:10-10".as_ptr().cast(),
        0,
        c"".as_ptr().cast(),
        0,
        9,
        10,
    ); // OK
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:10-9".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    ); // Issue#353
    eprint!("Expected error: ");
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:x".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    );
    eprint!("Expected error: ");
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:1-y".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    );
    eprint!("Expected error: ");
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:1,chr3".as_ptr().cast(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    );

    sam::sam_hdr_destroy(hdr);
    hts::hts_close(fp);

    libc::exit(0);
}

// original: main (htslib/test/test-parse-reg.c:145)
pub unsafe fn test_test_parse_reg_c_145_main(mut argc: i32, mut argv: *mut *mut u8) -> i32 {
    let mut flags = 0;

    while argc > 1 {
        if libc::strcmp((*argv.add(1)).cast(), c"-m".as_ptr()) == 0 {
            flags |= hts::HTS_PARSE_LIST;
            argc -= 1;
            argv = argv.add(1);
            continue;
        }

        if libc::strcmp((*argv.add(1)).cast(), c"-c".as_ptr()) == 0 {
            flags |= hts::HTS_PARSE_ONE_COORD;
            argc -= 1;
            argv = argv.add(1);
            continue;
        }

        // Automatic mode for test harness
        if libc::strcmp((*argv.add(1)).cast(), c"-t".as_ptr()) == 0 {
            test_test_parse_reg_c_72_reg_test(*argv.add(2));
        }

        break;
    }

    // Interactive mode for debugging
    if argc != 3 {
        eprintln!("Usage: test-parse-reg [-m] [-c] region[,region]...");
        libc::exit(1);
    }

    let fp = hts::hts_open((*argv.add(1)).cast(), c"r".as_ptr().cast());
    if fp.is_null() {
        libc::perror((*argv.add(1)).cast());
        libc::exit(1);
    }

    let hdr = sam::sam_hdr_read(fp);
    if hdr.is_null() {
        eprintln!("Couldn't read header");
        libc::exit(1);
    }

    let mut reg = (*argv.add(2)).cast_const();
    while *reg != 0 {
        let mut tid = 0;
        let mut beg = 0;
        let mut end = 0;
        reg = sam::sam_parse_region(hdr, reg, &mut tid, &mut beg, &mut end, flags);
        if reg.is_null() {
            eprintln!("Failed to parse region");
            libc::exit(1);
        }
        let name = if tid == -1 {
            "*".to_string()
        } else {
            String::from_utf8_lossy(
                std::ffi::CStr::from_ptr((*(*hdr).target_name.add(tid as usize)).cast())
                    .to_bytes(),
            )
            .into_owned()
        };
        println!("{:<20} {:>12} {:>12}", name, beg, end);
    }

    sam::sam_hdr_destroy(hdr);
    hts::hts_close(fp);

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn run_main(args: &[&str]) -> i32 {
        let mut c_args: Vec<Vec<u8>> = args
            .iter()
            .map(|arg| {
                let mut v = arg.as_bytes().to_vec();
                v.push(0);
                v
            })
            .collect();
        let mut argv: Vec<*mut u8> = c_args
            .iter_mut()
            .map(|arg| arg.as_mut_ptr())
            .chain(std::iter::once(std::ptr::null_mut()))
            .collect();

        test_test_parse_reg_c_145_main(args.len() as i32, argv.as_mut_ptr())
    }

    #[test]
    fn test_parse_reg_main_parses_fixture_regions_and_options() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            assert_eq!(
                run_main(&[
                    "test-parse-reg",
                    "htslib/test/colons.bam",
                    "{chr1:100-200}:100-200"
                ]),
                0
            );
            assert_eq!(
                run_main(&[
                    "test-parse-reg",
                    "-m",
                    "htslib/test/colons.bam",
                    "chr1,chr3"
                ]),
                0
            );
            assert_eq!(
                run_main(&["test-parse-reg", "-c", "htslib/test/colons.bam", "chr1:50"]),
                0
            );
        }
    }
}
