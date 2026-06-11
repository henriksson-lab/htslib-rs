use crate::htslib_rs::{faidx, hts::hts_pos_t};

unsafe extern "C" {
    static mut optarg: *mut u8;
    static mut optind: i32;
    // The C standard streams, linked directly from the C runtime.
    static mut stdout: *mut libc::FILE;
    static mut stderr: *mut libc::FILE;
}

// original: file_compare (htslib/test/test_faidx.c:33)
pub unsafe fn test_test_faidx_c_33_file_compare(
    file1: *const u8,
    file2: *const u8,
) -> i32 {
    let mut buf1 = [0u8; 1024];
    let mut buf2 = [0u8; 1024];
    let mut lno: u32 = 1;
    let mut ret = -1;

    let f1 = libc::fopen(file1.cast(), c"rb".as_ptr());
    if f1.is_null() {
        libc::perror(file1.cast());
        return -1;
    }
    let f2 = libc::fopen(file2.cast(), c"rb".as_ptr());
    if f2.is_null() {
        libc::perror(file2.cast());
        libc::fclose(f1);
        return -1;
    }

    loop {
        let got1 = libc::fread(buf1.as_mut_ptr().cast(), 1, buf1.len(), f1);
        let got2 = libc::fread(buf2.as_mut_ptr().cast(), 1, buf2.len(), f2);
        let mut i = 0usize;
        while i < got1 && i < got2 && buf1[i] == buf2[i] {
            if buf1[i] == b'\n' {
                lno += 1;
            }
            i += 1;
        }
        if i < got1 || i < got2 {
            eprintln!(
                "{} and {} differ at line {}",
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(file1.cast()).to_bytes()),
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(file2.cast()).to_bytes()),
                lno,
            );
            break;
        }
        if got1 == 0 || got2 == 0 {
            if libc::ferror(f1) != 0 {
                libc::perror(file1.cast());
            } else if libc::ferror(f2) != 0 {
                libc::perror(file2.cast());
            } else if got1 > 0 || got2 > 0 {
                eprintln!(
                    "EOF on {} at line {}",
                    String::from_utf8_lossy(
                        std::ffi::CStr::from_ptr(
                            if got1 > 0 { file2 } else { file1 }.cast()
                        )
                        .to_bytes()
                    ),
                    lno,
                );
            } else {
                ret = 0;
            }
            break;
        }
    }

    libc::fclose(f1);
    libc::fclose(f2);
    ret
}

// original: load_index (htslib/test/test_faidx.c:87)
pub unsafe fn test_test_faidx_c_87_load_index(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
    format: crate::htslib_rs::faidx::fai_format_options,
) -> *mut crate::htslib_rs::faidx::faidx_t {
    let fai = crate::htslib_rs::faidx::fai_load3_format(
        fn_.cast(),
        fnfai.cast(),
        fngzi.cast(),
        flags,
        format,
    );
    if fai.is_null() {
        eprintln!(
            "Failed: fai_load3({}, {}, {}, {}, {})",
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(fn_.cast()).to_bytes()),
            if fnfai.is_null() {
                "NULL".to_string()
            } else {
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(fnfai.cast()).to_bytes())
                    .into_owned()
            },
            if fngzi.is_null() {
                "NULL".to_string()
            } else {
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(fngzi.cast()).to_bytes())
                    .into_owned()
            },
            flags,
            format as i32,
        );
    }
    fai
}

// original: do_retrieval (htslib/test/test_faidx.c:99)
#[allow(clippy::too_many_arguments)]
pub unsafe fn test_test_faidx_c_99_do_retrieval(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
    format: crate::htslib_rs::faidx::fai_format_options,
    fnout: *const u8,
    interface: *const u8,
    nreg: i32,
    regions: *mut *mut u8,
) -> i32 {
    let mut use_64bit = 1;
    let mut use_parse_reg = 0;
    let mut use_adjust_reg = 0;
    let mut out = stdout.cast::<libc::FILE>();

    if !interface.is_null() {
        let interface_bytes = std::ffi::CStr::from_ptr(interface.cast()).to_bytes();
        if interface_bytes == b"fai_fetch" {
            use_64bit = 0;
        } else if interface_bytes == b"faidx_fetch_seq" {
            use_64bit = 0;
            use_parse_reg = 1;
        } else if interface_bytes == b"faidx_fetch_seq64" || interface_bytes == b"fai_parse_region"
        {
            use_parse_reg = 1;
        } else if interface_bytes == b"fai_adjust_region" {
            use_parse_reg = 1;
            use_adjust_reg = 1;
        }
    }

    if !fnout.is_null() {
        out = libc::fopen(fnout.cast(), c"wb".as_ptr());
        if out.is_null() {
            libc::perror(fnout.cast());
            return -1;
        }
    }

    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, flags, format);
    if fai.is_null() {
        if !fnout.is_null() {
            libc::fclose(out);
        }
        return -1;
    }

    for i in 0..nreg {
        let region = *regions.add(i as usize);
        let mut len: hts_pos_t = 0;
        let mut beg: hts_pos_t = 0;
        let mut end: hts_pos_t = 0;
        let mut tid: i32 = 0;
        let seq: *mut u8;

        if use_parse_reg != 0 {
            let e = crate::htslib_rs::faidx::fai_parse_region(
                fai, region.cast(), &mut tid, &mut beg, &mut end, 0,
            );
            if e.is_null() {
                eprintln!(
                    "Failed: fai_parse_region(fai, {}, &tid, &beg, &end, 0)",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(region.cast()).to_bytes()),
                );
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
            if use_adjust_reg != 0 {
                let orig_beg = beg;
                let orig_end = end;
                let r = crate::htslib_rs::faidx::fai_adjust_region(&*fai, tid, &mut beg, &mut end);
                if r < 0
                    || (((r & 1) != 0) ^ (beg != orig_beg))
                    || (((r & 2) != 0) ^ (end != orig_end))
                {
                    eprintln!(
                        "Failed: fai_adjust_region(fai, {}, {}, {}) returned {}\nAfter: beg = {} end = {}",
                        tid,
                        orig_beg as i64,
                        orig_end as i64,
                        r,
                        beg as i64,
                        end as i64,
                    );
                    crate::htslib_rs::faidx::fai_destroy(fai);
                    if !fnout.is_null() {
                        libc::fclose(out);
                    }
                    return -1;
                }
            }
            let name = crate::htslib_rs::faidx::faidx_iseq(&*fai, tid).map(|b| {
                let mut v = b.to_vec();
                v.push(0);
                v
            });
            let name = name
                .as_ref()
                .map_or(std::ptr::null(), |b| b.as_ptr().cast());
            if use_64bit != 0 {
                seq = crate::htslib_rs::faidx::faidx_fetch_seq64(fai, name, beg, end - 1, &mut len)
                    .cast();
            } else {
                let mut ilen: i32 = 0;
                seq = crate::htslib_rs::faidx::faidx_fetch_seq(
                    fai,
                    name,
                    beg as i32,
                    (end - 1) as i32,
                    &mut ilen,
                )
                .cast();
                len = ilen as hts_pos_t;
            }
            if seq.is_null() {
                eprintln!(
                    "Failed: faidx_fetch_seq{}(fai, {}, {}, {}, &len)",
                    if use_64bit != 0 { "64" } else { "" },
                    if name.is_null() {
                        String::new()
                    } else {
                        String::from_utf8_lossy(std::ffi::CStr::from_ptr(name.cast()).to_bytes())
                            .into_owned()
                    },
                    beg as i64,
                    end as i64,
                );
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
        } else {
            if use_64bit != 0 {
                seq = crate::htslib_rs::faidx::fai_fetch64(fai, region.cast(), &mut len).cast();
            } else {
                let mut ilen: i32 = 0;
                seq = crate::htslib_rs::faidx::fai_fetch(fai, region.cast(), &mut ilen).cast();
                len = ilen as hts_pos_t;
            }
            if seq.is_null() {
                eprintln!(
                    "Failed: fai_fetch{}(fai, {}, &len)",
                    if use_64bit != 0 { "64" } else { "" },
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(region.cast()).to_bytes()),
                );
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
        }

        let l = libc::strlen(seq.cast());
        {
            let line = format!(
                "{}{} length: {}\n",
                if format == crate::htslib_rs::faidx::FAI_FASTQ {
                    '@'
                } else {
                    '>'
                },
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(region.cast()).to_bytes()),
                len as i64,
            );
            libc::fwrite(line.as_ptr().cast(), 1, line.len(), out);
        }
        let mut pos = 0usize;
        while pos < l {
            let chunk = std::cmp::min(50, l - pos);
            libc::fwrite(seq.add(pos).cast(), 1, chunk, out);
            libc::fwrite(b"\n".as_ptr().cast(), 1, 1, out);
            pos += 50;
        }
        libc::free(seq.cast());

        if format == crate::htslib_rs::faidx::FAI_FASTQ {
            let mut qual_len: hts_pos_t = 0;
            let qual: *mut u8;
            if use_parse_reg != 0 {
                let name = crate::htslib_rs::faidx::faidx_iseq(&*fai, tid).map(|b| {
                    let mut v = b.to_vec();
                    v.push(0);
                    v
                });
                let name = name
                    .as_ref()
                    .map_or(std::ptr::null(), |b| b.as_ptr().cast());
                if use_64bit != 0 {
                    qual = crate::htslib_rs::faidx::faidx_fetch_qual64(
                        fai,
                        name,
                        beg,
                        end - 1,
                        &mut qual_len,
                    )
                    .cast();
                } else {
                    let mut ilen: i32 = 0;
                    qual = crate::htslib_rs::faidx::faidx_fetch_qual(
                        fai,
                        name,
                        beg as i32,
                        (end - 1) as i32,
                        &mut ilen,
                    )
                    .cast();
                    qual_len = ilen as hts_pos_t;
                }
            } else if use_64bit != 0 {
                qual =
                    crate::htslib_rs::faidx::fai_fetchqual64(fai, region.cast(), &mut qual_len)
                        .cast();
                if qual.is_null() {
                    eprintln!(
                        "Failed: fai_fetchqual64(fai, {}, &len)",
                        String::from_utf8_lossy(std::ffi::CStr::from_ptr(region.cast()).to_bytes()),
                    );
                    crate::htslib_rs::faidx::fai_destroy(fai);
                    if !fnout.is_null() {
                        libc::fclose(out);
                    }
                    return -1;
                }
            } else {
                let mut ilen: i32 = 0;
                qual = crate::htslib_rs::faidx::fai_fetchqual(fai, region.cast(), &mut ilen).cast();
                qual_len = ilen as hts_pos_t;
                if qual.is_null() {
                    eprintln!(
                        "Failed: fai_fetchqual64(fai, {}, &len)",
                        String::from_utf8_lossy(std::ffi::CStr::from_ptr(region.cast()).to_bytes()),
                    );
                    crate::htslib_rs::faidx::fai_destroy(fai);
                    if !fnout.is_null() {
                        libc::fclose(out);
                    }
                    return -1;
                }
            }
            if qual_len != len {
                eprintln!(
                    "Sequence and quality lengths differ for {} {}",
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(fn_.cast()).to_bytes()),
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(region.cast()).to_bytes()),
                );
                libc::free(qual.cast());
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
            libc::fwrite(b"+\n".as_ptr().cast(), 1, 2, out);
            let l = libc::strlen(qual.cast());
            let mut pos = 0usize;
            while pos < l {
                let chunk = std::cmp::min(50, l - pos);
                libc::fwrite(qual.add(pos).cast(), 1, chunk, out);
                libc::fwrite(b"\n".as_ptr().cast(), 1, 1, out);
                pos += 50;
            }
            libc::free(qual.cast());
        }
    }

    crate::htslib_rs::faidx::fai_destroy(fai);
    if !fnout.is_null() && libc::fclose(out) != 0 {
        libc::perror(fnout.cast());
        return -1;
    }
    0
}

// original: test_fai_line_length (htslib/test/test_faidx.c:260)
pub unsafe fn test_test_faidx_c_260_test_fai_line_length(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const u8,
    reg: *const u8,
) -> i32 {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_len = crate::htslib_rs::faidx::fai_line_length(fai, reg.cast());
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_len = libc::strtoll(expected.cast(), std::ptr::null_mut(), 10);
        if found_len != exp_len {
            eprintln!(
                "Unexpected result {} from fai_line_length, expected {}",
                found_len,
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(expected.cast()).to_bytes()),
            );
            return -1;
        }
    } else {
        println!("{}", found_len);
    }
    0
}

// original: test_faidx_has_seq (htslib/test/test_faidx.c:285)
pub unsafe fn test_test_faidx_c_285_test_faidx_has_seq(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const u8,
    seq: *const u8,
) -> i32 {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let res = crate::htslib_rs::faidx::faidx_has_seq(fai, seq.cast());
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_res = libc::strtol(expected.cast(), std::ptr::null_mut(), 10);
        if res as i64 != exp_res {
            eprintln!(
                "Unexpected result {} from faidx_has_seq({}) expected {}",
                res,
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(seq.cast()).to_bytes()),
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(expected.cast()).to_bytes()),
            );
            return -1;
        }
    } else {
        println!("{}", res);
    }
    0
}

// original: test_faidx_iseq (htslib/test/test_faidx.c:310)
pub unsafe fn test_test_faidx_c_310_test_faidx_iseq(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const u8,
    index: *const u8,
) -> i32 {
    let idx = libc::atoi(index.cast());
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_name = crate::htslib_rs::faidx::faidx_iseq(&*fai, idx);
    let expected_bytes = if expected.is_null() {
        None
    } else {
        Some(std::ffi::CStr::from_ptr(expected.cast()).to_bytes())
    };
    let mut ret = 0;
    if let Some(expected_bytes) = expected_bytes {
        if found_name.as_deref() != Some(expected_bytes) {
            eprintln!(
                "Unexpected result {} from faidx_iseq(fai, {}), expected {}",
                found_name
                    .as_ref()
                    .map_or_else(|| "(null)".to_string(), |b| String::from_utf8_lossy(b)
                        .into_owned()),
                idx,
                String::from_utf8_lossy(expected_bytes),
            );
            ret = -1;
        }
    } else {
        println!(
            "{}",
            found_name
                .as_ref()
                .map_or_else(|| "(null)".to_string(), |b| String::from_utf8_lossy(b)
                    .into_owned()),
        );
    }
    crate::htslib_rs::faidx::fai_destroy(fai);
    ret
}

// original: test_faidx_seq_len (htslib/test/test_faidx.c:339)
pub unsafe fn test_test_faidx_c_339_test_faidx_seq_len(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const u8,
    seq: *const u8,
) -> i32 {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_len = crate::htslib_rs::faidx::faidx_seq_len(fai, seq.cast());
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_len = libc::atoi(expected.cast());
        if found_len != exp_len {
            eprintln!(
                "Unexpected result {} from faidx_seq_len(fai, {}) expected {}",
                found_len,
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(seq.cast()).to_bytes()),
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(expected.cast()).to_bytes()),
            );
            return -1;
        }
    } else {
        println!("{}", found_len);
    }
    0
}

// original: test_faidx_seq_len64 (htslib/test/test_faidx.c:366)
pub unsafe fn test_test_faidx_c_366_test_faidx_seq_len64(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const u8,
    seq: *const u8,
) -> i32 {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_len = crate::htslib_rs::faidx::faidx_seq_len(fai, seq.cast()) as hts_pos_t;
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_len = libc::strtoll(expected.cast(), std::ptr::null_mut(), 10);
        if found_len != exp_len {
            eprintln!(
                "Unexpected result {} from fai_seq_len64(fai, {}) expected {}",
                found_len,
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(seq.cast()).to_bytes()),
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(expected.cast()).to_bytes()),
            );
            return -1;
        }
    } else {
        println!("{}", found_len);
    }
    0
}

// original: usage (htslib/test/test_faidx.c:394)
pub unsafe fn test_test_faidx_c_394_usage(out: *mut libc::FILE, arg0: *const u8) {
    let arg0 = String::from_utf8_lossy(std::ffi::CStr::from_ptr(arg0.cast()).to_bytes());
    let msg = format!(
        "Usage: {arg0} [-c] -i fasta/q [-f fai_file] [-g gzi_file] [-e expected_fai]\n       {arg0} [-cQ] -i fasta/q [-f fai_file] [-g gzi_file] [region]\n       {arg0} -t FUNC -i fasta/q [-f fai_file] [-g gzi_file] [-e expected] <PARAM>\n       {arg0} -h\n",
    );
    libc::fwrite(msg.as_ptr().cast(), 1, msg.len(), out);
}

// original: help (htslib/test/test_faidx.c:403)
pub unsafe fn test_test_faidx_c_403_help(out: *mut libc::FILE, arg0: *const u8) {
    test_test_faidx_c_394_usage(out, arg0);
    let msg = "Options:\n  -i FILE      Input file\n  -f FILE      Fasta/q index file name\n  -g FILE      Bgzip index file name\n  -o FILE      Output file name\n  -e FILE|STR  Expected output\n  -c           Set FAI_CREATE flag\n  -Q           Output fastq format\n  -t FUNC      Test function\n  -h           Print this help\n\nExpected output is compared to the FAI file in indexing mode; the output file\nin retrieval mode; expected output for various -t function tests.\n\nUnit tests (-t option):\n   fai_line_length, faidx_has_seq, faidx_iseq, faidx_seq_len, faidx_seq_len64\nIn retrieval mode, -t can change the functions used to fetch data:\n   fai_fetch, fai_fetch64, faidx_fetch_seq, faidx_fetch_seq64,\n   fai_parse_region, fai_adjust_region\n\n";
    libc::fwrite(msg.as_ptr().cast(), 1, msg.len(), out);
}

// original: main (htslib/test/test_faidx.c:430)
pub unsafe fn test_test_faidx_c_430_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut fn_: *const u8 = std::ptr::null();
    let mut fnout: *const u8 = std::ptr::null();
    let mut fnfai: *const u8 = std::ptr::null();
    let mut fngzi: *const u8 = std::ptr::null();
    let mut expected: *const u8 = std::ptr::null();
    let mut func: *const u8 = c"".as_ptr().cast();
    let mut flags: i32 = 0;
    let mut format = faidx::FAI_FASTA;

    loop {
        let opt = libc::getopt(argc, argv.cast(), c"i:f:g:o:e:t:cQh".as_ptr());
        if opt <= 0 {
            break;
        }
        match opt as u8 {
            b'i' => fn_ = optarg,
            b'f' => fnfai = optarg,
            b'g' => fngzi = optarg,
            b'o' => fnout = optarg,
            b'e' => expected = optarg,
            b'c' => flags |= faidx::FAI_CREATE,
            b'Q' => format = faidx::FAI_FASTQ,
            b't' => func = optarg,
            b'h' => {
                test_test_faidx_c_403_help(stdout.cast(), *argv);
                return libc::EXIT_SUCCESS;
            }
            _ => {
                test_test_faidx_c_394_usage(stderr.cast(), *argv);
                return libc::EXIT_FAILURE;
            }
        }
    }

    if fn_.is_null() {
        test_test_faidx_c_394_usage(stderr.cast(), *argv);
        return libc::EXIT_FAILURE;
    }

    let func_bytes = std::ffi::CStr::from_ptr(func.cast()).to_bytes();
    let res = if optind == argc {
        let mut res = faidx::fai_build3(fn_.cast(), fnfai.cast(), fngzi.cast());
        if res != 0 {
            eprintln!(
                "Failed: fai_build3({}, {}, {})",
                String::from_utf8_lossy(std::ffi::CStr::from_ptr(fn_.cast()).to_bytes()),
                if fnfai.is_null() {
                    "NULL".to_string()
                } else {
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(fnfai.cast()).to_bytes())
                        .into_owned()
                },
                if fngzi.is_null() {
                    "NULL".to_string()
                } else {
                    String::from_utf8_lossy(std::ffi::CStr::from_ptr(fngzi.cast()).to_bytes())
                        .into_owned()
                },
            );
        } else if !expected.is_null() {
            res = test_test_faidx_c_33_file_compare(fnfai, expected);
        }
        res
    } else if func_bytes == b"fai_line_length" {
        test_test_faidx_c_260_test_fai_line_length(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if func_bytes == b"faidx_has_seq" {
        test_test_faidx_c_285_test_faidx_has_seq(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if func_bytes == b"faidx_iseq" {
        test_test_faidx_c_310_test_faidx_iseq(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if func_bytes == b"faidx_seq_len" {
        test_test_faidx_c_339_test_faidx_seq_len(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if func_bytes == b"faidx_seq_len64" {
        test_test_faidx_c_366_test_faidx_seq_len64(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else {
        let mut res = test_test_faidx_c_99_do_retrieval(
            fn_,
            fnfai,
            fngzi,
            flags,
            format,
            fnout,
            func,
            argc - optind,
            argv.add(optind as usize),
        );
        if res == 0 && !fnout.is_null() && !expected.is_null() {
            res = test_test_faidx_c_33_file_compare(fnout, expected);
        }
        res
    };

    if res == 0 {
        libc::EXIT_SUCCESS
    } else {
        libc::EXIT_FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static GETOPT_LOCK: Mutex<()> = Mutex::new(());

    fn fixture(path: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let mut bytes = path.to_string_lossy().into_owned().into_bytes();
        bytes.push(0);
        bytes
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "htslib-rs-test-faidx-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    unsafe fn run_main(args: &[Vec<u8>]) -> i32 {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs). `GETOPT_LOCK` is retained for backward-compat
        // but is now effectively a no-op while the global lock is held.
        let _guard = GETOPT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        optarg = std::ptr::null_mut();
        // optind = 0 forces glibc getopt full reinit (shared-process tests).
        optind = 0;
        let mut argv = args
            .iter()
            .map(|arg| arg.as_ptr() as *mut u8)
            .collect::<Vec<_>>();
        test_test_faidx_c_430_main(argv.len() as i32, argv.as_mut_ptr())
    }

    #[test]
    fn original_test_faidx_main_fastq_retrieval_compares_expected_output() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let out_path = temp_path("fastq-retrieval").with_extension("fq");
        let mut out = out_path.to_string_lossy().into_owned().into_bytes();
        out.push(0);
        let args = vec![
            b"test_faidx\0".to_vec(),
            b"-i\0".to_vec(),
            fixture("htslib/test/faidx/fastqs.fq"),
            b"-f\0".to_vec(),
            fixture("htslib/test/faidx/fastqs.fq.expected.fai"),
            b"-o\0".to_vec(),
            out.clone(),
            b"-e\0".to_vec(),
            fixture("htslib/test/faidx/fastqs.1.expected.fq"),
            b"-Q\0".to_vec(),
            b"FAKE0006_1:4-12\0".to_vec(),
            b"FSRRS4401BE7HA_1:81-120\0".to_vec(),
            b"FAKE0010_2\0".to_vec(),
            b"SRR014849.50939_3:71-90\0".to_vec(),
        ];

        unsafe {
            assert_eq!(run_main(&args), libc::EXIT_SUCCESS);
        }
        let _ = std::fs::remove_file(out_path);
    }
}
