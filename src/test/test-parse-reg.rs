use std::ffi::{c_char, c_int};

use crate::htslib_rs::{hts, sam};

// original: reg_expected (htslib/test/test-parse-reg.c:50)
pub unsafe fn test_test_parse_reg_c_50_reg_expected(
    hdr: *mut sam::sam_hdr_t,
    reg: *const c_char,
    flags: c_int,
    reg_exp: *const c_char,
    tid_exp: c_int,
    beg_exp: hts::hts_pos_t,
    end_exp: hts::hts_pos_t,
) {
    let mut tid_out = -1;
    let mut beg_out: hts::hts_pos_t = -1;
    let mut end_out: hts::hts_pos_t = -1;

    let reg_out = sam::sam_parse_region(hdr, reg, &mut tid_out, &mut beg_out, &mut end_out, flags);

    if (!reg_out.is_null() != !reg_exp.is_null())
        || (!reg_out.is_null() && !reg_exp.is_null() && libc::strcmp(reg_out, reg_exp) != 0)
        || (!reg_exp.is_null() && tid_out != tid_exp)
        || (!reg_exp.is_null() && beg_out != beg_exp)
        || (!reg_exp.is_null() && end_out != end_exp)
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Parsing \"%s\" expected return \"%s\", %d:%lld-%lld, but got \"%s\", %d:%lld-%lld\n"
                .as_ptr(),
            reg,
            if !reg_exp.is_null() {
                reg_exp
            } else {
                c"(null)".as_ptr()
            },
            tid_exp,
            beg_exp,
            end_exp,
            if !reg_out.is_null() {
                reg_out
            } else {
                c"(null)".as_ptr()
            },
            tid_out,
            beg_out,
            end_out,
        );
        libc::exit(1);
    }
}

// original: reg_test (htslib/test/test-parse-reg.c:72)
pub unsafe fn test_test_parse_reg_c_72_reg_test(fn_: *const c_char) -> c_int {
    let fp = hts::hts_open(fn_, c"r".as_ptr());
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
        c"chr1".as_ptr(),
        0,
        c"".as_ptr(),
        0,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50".as_ptr(),
        0,
        c"".as_ptr(),
        0,
        49,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50".as_ptr(),
        hts::HTS_PARSE_ONE_COORD,
        c"".as_ptr(),
        0,
        49,
        50,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50-100".as_ptr(),
        0,
        c"".as_ptr(),
        0,
        49,
        100,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:50-".as_ptr(),
        0,
        c"".as_ptr(),
        0,
        49,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr1:-50".as_ptr(), 0, c"".as_ptr(), 0, 0, 50);

    // Check quoting
    libc::fprintf(hts_sys::stderr.cast(), c"Expected error: ".as_ptr());
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:100-200".as_ptr(),
        0,
        std::ptr::null(),
        0,
        0,
        0,
    ); // ambiguous
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1}:100-200".as_ptr(),
        0,
        c"".as_ptr(),
        0,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1:100-200}".as_ptr(),
        0,
        c"".as_ptr(),
        2,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1:100-200}:100-200".as_ptr(),
        0,
        c"".as_ptr(),
        2,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr2:100-200}:100-200".as_ptr(),
        0,
        c"".as_ptr(),
        3,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr2:100-200:100-200".as_ptr(),
        0,
        c"".as_ptr(),
        3,
        99,
        200,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr2:100-200".as_ptr(),
        0,
        c"".as_ptr(),
        3,
        0,
        hts::HTS_POS_MAX,
    );

    // Check numerics
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1000-1500".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1,000-1,500".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1k-1.5K".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1e3-1.5e3".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        999,
        1500,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1e3-15e2".as_ptr(),
        0,
        c"".as_ptr(),
        4,
        999,
        1500,
    );

    // Check list mode
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1,chr3".as_ptr(),
        hts::HTS_PARSE_LIST,
        c"chr3".as_ptr(),
        0,
        0,
        hts::HTS_POS_MAX,
    );
    libc::fprintf(hts_sys::stderr.cast(), c"Expected error: ".as_ptr());
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:100-200,chr3".as_ptr(),
        hts::HTS_PARSE_LIST,
        std::ptr::null(),
        0,
        0,
        0,
    ); // ambiguous
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1,chr3}".as_ptr(),
        hts::HTS_PARSE_LIST,
        c"".as_ptr(),
        5,
        0,
        hts::HTS_POS_MAX,
    );
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"{chr1,chr3},chr1".as_ptr(),
        hts::HTS_PARSE_LIST,
        c"chr1".as_ptr(),
        5,
        0,
        hts::HTS_POS_MAX,
    );
    // incorrect usage; first reg is valid (but not what user expects).
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr3:1,000-1,500".as_ptr(),
        hts::HTS_PARSE_LIST | hts::HTS_PARSE_ONE_COORD,
        c"000-1,500".as_ptr(),
        4,
        0,
        1,
    );

    // More expected failures
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr2".as_ptr(), 0, std::ptr::null(), 0, 0, 0);
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr1,".as_ptr(), 0, std::ptr::null(), 0, 0, 0);
    libc::fprintf(hts_sys::stderr.cast(), c"Expected error: ".as_ptr());
    test_test_parse_reg_c_50_reg_expected(hdr, c"{chr1".as_ptr(), 0, std::ptr::null(), 0, 0, 0);
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr1:10-10".as_ptr(), 0, c"".as_ptr(), 0, 9, 10); // OK
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr1:10-9".as_ptr(), 0, std::ptr::null(), 0, 0, 0); // Issue#353
    libc::fprintf(hts_sys::stderr.cast(), c"Expected error: ".as_ptr());
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr1:x".as_ptr(), 0, std::ptr::null(), 0, 0, 0);
    libc::fprintf(hts_sys::stderr.cast(), c"Expected error: ".as_ptr());
    test_test_parse_reg_c_50_reg_expected(hdr, c"chr1:1-y".as_ptr(), 0, std::ptr::null(), 0, 0, 0);
    libc::fprintf(hts_sys::stderr.cast(), c"Expected error: ".as_ptr());
    test_test_parse_reg_c_50_reg_expected(
        hdr,
        c"chr1:1,chr3".as_ptr(),
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
pub unsafe fn test_test_parse_reg_c_145_main(mut argc: c_int, mut argv: *mut *mut c_char) -> c_int {
    let mut flags = 0;

    while argc > 1 {
        if libc::strcmp(*argv.add(1), c"-m".as_ptr()) == 0 {
            flags |= hts::HTS_PARSE_LIST;
            argc -= 1;
            argv = argv.add(1);
            continue;
        }

        if libc::strcmp(*argv.add(1), c"-c".as_ptr()) == 0 {
            flags |= hts::HTS_PARSE_ONE_COORD;
            argc -= 1;
            argv = argv.add(1);
            continue;
        }

        // Automatic mode for test harness
        if libc::strcmp(*argv.add(1), c"-t".as_ptr()) == 0 {
            test_test_parse_reg_c_72_reg_test(*argv.add(2));
        }

        break;
    }

    // Interactive mode for debugging
    if argc != 3 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Usage: test-parse-reg [-m] [-c] region[,region]...\n".as_ptr(),
        );
        libc::exit(1);
    }

    let fp = hts::hts_open(*argv.add(1), c"r".as_ptr());
    if fp.is_null() {
        libc::perror(*argv.add(1));
        libc::exit(1);
    }

    let hdr = sam::sam_hdr_read(fp);
    if hdr.is_null() {
        libc::fprintf(hts_sys::stderr.cast(), c"Couldn't read header\n".as_ptr());
        libc::exit(1);
    }

    let mut reg = (*argv.add(2)).cast_const();
    while *reg != 0 {
        let mut tid = 0;
        let mut beg = 0;
        let mut end = 0;
        reg = sam::sam_parse_region(hdr, reg, &mut tid, &mut beg, &mut end, flags);
        if reg.is_null() {
            libc::fprintf(hts_sys::stderr.cast(), c"Failed to parse region\n".as_ptr());
            libc::exit(1);
        }
        libc::printf(
            c"%-20s %12lld %12lld\n".as_ptr(),
            if tid == -1 {
                c"*".as_ptr()
            } else {
                *(*hdr).target_name.add(tid as usize)
            },
            beg,
            end,
        );
    }

    sam::sam_hdr_destroy(hdr);
    hts::hts_close(fp);

    0
}
