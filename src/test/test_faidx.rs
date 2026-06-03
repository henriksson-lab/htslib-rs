use std::ffi::{c_char, c_int, c_uint};

use crate::htslib_rs::{faidx, hts::hts_pos_t};

unsafe extern "C" {
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

// original: file_compare (htslib/test/test_faidx.c:33)
pub unsafe fn test_test_faidx_c_33_file_compare(
    file1: *const c_char,
    file2: *const c_char,
) -> c_int {
    let mut buf1 = [0u8; 1024];
    let mut buf2 = [0u8; 1024];
    let mut lno: c_uint = 1;
    let mut ret = -1;

    let f1 = libc::fopen(file1, c"rb".as_ptr());
    if f1.is_null() {
        libc::perror(file1);
        return -1;
    }
    let f2 = libc::fopen(file2, c"rb".as_ptr());
    if f2.is_null() {
        libc::perror(file2);
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
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"%s and %s differ at line %u\n".as_ptr(),
                file1,
                file2,
                lno,
            );
            break;
        }
        if got1 == 0 || got2 == 0 {
            if libc::ferror(f1) != 0 {
                libc::perror(file1);
            } else if libc::ferror(f2) != 0 {
                libc::perror(file2);
            } else if got1 > 0 || got2 > 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"EOF on %s at line %u\n".as_ptr(),
                    if got1 > 0 { file2 } else { file1 },
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
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
    format: crate::htslib_rs::faidx::fai_format_options,
) -> *mut crate::htslib_rs::faidx::faidx_t {
    let fai = crate::htslib_rs::faidx::fai_load3_format(fn_, fnfai, fngzi, flags, format);
    if fai.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Failed: fai_load3(%s, %s, %s, %d, %d)\n".as_ptr(),
            fn_,
            if fnfai.is_null() {
                c"NULL".as_ptr()
            } else {
                fnfai
            },
            if fngzi.is_null() {
                c"NULL".as_ptr()
            } else {
                fngzi
            },
            flags,
            format as c_int,
        );
    }
    fai
}

// original: do_retrieval (htslib/test/test_faidx.c:99)
#[allow(clippy::too_many_arguments)]
pub unsafe fn test_test_faidx_c_99_do_retrieval(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
    format: crate::htslib_rs::faidx::fai_format_options,
    fnout: *const c_char,
    interface: *const c_char,
    nreg: c_int,
    regions: *mut *mut c_char,
) -> c_int {
    let mut use_64bit = 1;
    let mut use_parse_reg = 0;
    let mut use_adjust_reg = 0;
    let mut out = crate::htslib_rs::c_compat::stdout.cast::<libc::FILE>();

    if !interface.is_null() {
        if libc::strcmp(interface, c"fai_fetch".as_ptr()) == 0 {
            use_64bit = 0;
        } else if libc::strcmp(interface, c"faidx_fetch_seq".as_ptr()) == 0 {
            use_64bit = 0;
            use_parse_reg = 1;
        } else if libc::strcmp(interface, c"faidx_fetch_seq64".as_ptr()) == 0
            || libc::strcmp(interface, c"fai_parse_region".as_ptr()) == 0
        {
            use_parse_reg = 1;
        } else if libc::strcmp(interface, c"fai_adjust_region".as_ptr()) == 0 {
            use_parse_reg = 1;
            use_adjust_reg = 1;
        }
    }

    if !fnout.is_null() {
        out = libc::fopen(fnout, c"wb".as_ptr());
        if out.is_null() {
            libc::perror(fnout);
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
        let mut tid: c_int = 0;
        let seq: *mut c_char;

        if use_parse_reg != 0 {
            let e = crate::htslib_rs::faidx::fai_parse_region(
                fai, region, &mut tid, &mut beg, &mut end, 0,
            );
            if e.is_null() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Failed: fai_parse_region(fai, %s, &tid, &beg, &end, 0)\n".as_ptr(),
                    region,
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
                let r = crate::htslib_rs::faidx::fai_adjust_region(fai, tid, &mut beg, &mut end);
                if r < 0
                    || (((r & 1) != 0) ^ (beg != orig_beg))
                    || (((r & 2) != 0) ^ (end != orig_end))
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Failed: fai_adjust_region(fai, %d, %lld, %lld) returned %d\nAfter: beg = %lld end = %lld\n".as_ptr(),
                        tid,
                        orig_beg as libc::c_longlong,
                        orig_end as libc::c_longlong,
                        r,
                        beg as libc::c_longlong,
                        end as libc::c_longlong,
                        );
                    crate::htslib_rs::faidx::fai_destroy(fai);
                    if !fnout.is_null() {
                        libc::fclose(out);
                    }
                    return -1;
                }
            }
            let name = crate::htslib_rs::faidx::faidx_iseq(fai, tid);
            if use_64bit != 0 {
                seq = crate::htslib_rs::faidx::faidx_fetch_seq64(fai, name, beg, end - 1, &mut len);
            } else {
                let mut ilen: c_int = 0;
                seq = crate::htslib_rs::faidx::faidx_fetch_seq(
                    fai,
                    name,
                    beg as c_int,
                    (end - 1) as c_int,
                    &mut ilen,
                );
                len = ilen as hts_pos_t;
            }
            if seq.is_null() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Failed: faidx_fetch_seq%s(fai, %s, %lld, %lld, &len)\n".as_ptr(),
                    if use_64bit != 0 {
                        c"64".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    name,
                    beg as libc::c_longlong,
                    end as libc::c_longlong,
                );
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
        } else {
            if use_64bit != 0 {
                seq = crate::htslib_rs::faidx::fai_fetch64(fai, region, &mut len);
            } else {
                let mut ilen: c_int = 0;
                seq = crate::htslib_rs::faidx::fai_fetch(fai, region, &mut ilen);
                len = ilen as hts_pos_t;
            }
            if seq.is_null() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Failed: fai_fetch%s(fai, %s, &len)\n".as_ptr(),
                    if use_64bit != 0 {
                        c"64".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    region,
                );
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
        }

        let l = libc::strlen(seq);
        libc::fprintf(
            out,
            c"%c%s length: %lld\n".as_ptr(),
            if format == crate::htslib_rs::faidx::FAI_FASTQ {
                b'@' as c_int
            } else {
                b'>' as c_int
            },
            region,
            len as libc::c_longlong,
        );
        let mut pos = 0usize;
        while pos < l {
            libc::fprintf(out, c"%.*s\n".as_ptr(), 50, seq.add(pos));
            pos += 50;
        }
        libc::free(seq.cast());

        if format == crate::htslib_rs::faidx::FAI_FASTQ {
            let mut qual_len: hts_pos_t = 0;
            let qual: *mut c_char;
            if use_parse_reg != 0 {
                let name = crate::htslib_rs::faidx::faidx_iseq(fai, tid);
                if use_64bit != 0 {
                    qual = crate::htslib_rs::faidx::faidx_fetch_qual64(
                        fai,
                        name,
                        beg,
                        end - 1,
                        &mut qual_len,
                    );
                } else {
                    let mut ilen: c_int = 0;
                    qual = crate::htslib_rs::faidx::faidx_fetch_qual(
                        fai,
                        name,
                        beg as c_int,
                        (end - 1) as c_int,
                        &mut ilen,
                    );
                    qual_len = ilen as hts_pos_t;
                }
            } else if use_64bit != 0 {
                qual = crate::htslib_rs::faidx::fai_fetchqual64(fai, region, &mut qual_len);
                if qual.is_null() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Failed: fai_fetchqual64(fai, %s, &len)\n".as_ptr(),
                        region,
                    );
                    crate::htslib_rs::faidx::fai_destroy(fai);
                    if !fnout.is_null() {
                        libc::fclose(out);
                    }
                    return -1;
                }
            } else {
                let mut ilen: c_int = 0;
                qual = crate::htslib_rs::faidx::fai_fetchqual(fai, region, &mut ilen);
                qual_len = ilen as hts_pos_t;
                if qual.is_null() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Failed: fai_fetchqual64(fai, %s, &len)\n".as_ptr(),
                        region,
                    );
                    crate::htslib_rs::faidx::fai_destroy(fai);
                    if !fnout.is_null() {
                        libc::fclose(out);
                    }
                    return -1;
                }
            }
            if qual_len != len {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Sequence and quality lengths differ for %s %s\n".as_ptr(),
                    fn_,
                    region,
                );
                libc::free(qual.cast());
                crate::htslib_rs::faidx::fai_destroy(fai);
                if !fnout.is_null() {
                    libc::fclose(out);
                }
                return -1;
            }
            libc::fprintf(out, c"+\n".as_ptr());
            let l = libc::strlen(qual);
            let mut pos = 0usize;
            while pos < l {
                libc::fprintf(out, c"%.*s\n".as_ptr(), 50, qual.add(pos));
                pos += 50;
            }
            libc::free(qual.cast());
        }
    }

    crate::htslib_rs::faidx::fai_destroy(fai);
    if !fnout.is_null() && libc::fclose(out) != 0 {
        libc::perror(fnout);
        return -1;
    }
    0
}

// original: test_fai_line_length (htslib/test/test_faidx.c:260)
pub unsafe fn test_test_faidx_c_260_test_fai_line_length(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const c_char,
    reg: *const c_char,
) -> c_int {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_len = crate::htslib_rs::faidx::fai_line_length(fai, reg);
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_len = libc::strtoll(expected, std::ptr::null_mut(), 10);
        if found_len != exp_len {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Unexpected result %lld from fai_line_length, expected %s\n".as_ptr(),
                found_len,
                expected,
            );
            return -1;
        }
    } else {
        libc::printf(c"%lld\n".as_ptr(), found_len);
    }
    0
}

// original: test_faidx_has_seq (htslib/test/test_faidx.c:285)
pub unsafe fn test_test_faidx_c_285_test_faidx_has_seq(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const c_char,
    seq: *const c_char,
) -> c_int {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let res = crate::htslib_rs::faidx::faidx_has_seq(fai, seq);
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_res = libc::strtol(expected, std::ptr::null_mut(), 10);
        if res as libc::c_long != exp_res {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Unexpected result %d from faidx_has_seq(%s) expected %s\n".as_ptr(),
                res,
                seq,
                expected,
            );
            return -1;
        }
    } else {
        libc::printf(c"%d\n".as_ptr(), res);
    }
    0
}

// original: test_faidx_iseq (htslib/test/test_faidx.c:310)
pub unsafe fn test_test_faidx_c_310_test_faidx_iseq(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const c_char,
    index: *const c_char,
) -> c_int {
    let idx = libc::atoi(index);
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_name = crate::htslib_rs::faidx::faidx_iseq(fai, idx);
    let mut ret = 0;
    if !expected.is_null() {
        if found_name.is_null() || libc::strcmp(found_name, expected) != 0 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Unexpected result %s from faidx_iseq(fai, %d), expected %s\n".as_ptr(),
                if found_name.is_null() {
                    c"(null)".as_ptr()
                } else {
                    found_name
                },
                idx,
                expected,
            );
            ret = -1;
        }
    } else {
        libc::printf(
            c"%s\n".as_ptr(),
            if found_name.is_null() {
                c"(null)".as_ptr()
            } else {
                found_name
            },
        );
    }
    crate::htslib_rs::faidx::fai_destroy(fai);
    ret
}

// original: test_faidx_seq_len (htslib/test/test_faidx.c:339)
pub unsafe fn test_test_faidx_c_339_test_faidx_seq_len(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const c_char,
    seq: *const c_char,
) -> c_int {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_len = crate::htslib_rs::faidx::faidx_seq_len(fai, seq);
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_len = libc::atoi(expected);
        if found_len != exp_len {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Unexpected result %d from faidx_seq_len(fai, %s) expected %s\n".as_ptr(),
                found_len,
                seq,
                expected,
            );
            return -1;
        }
    } else {
        libc::printf(c"%d\n".as_ptr(), found_len);
    }
    0
}

// original: test_faidx_seq_len64 (htslib/test/test_faidx.c:366)
pub unsafe fn test_test_faidx_c_366_test_faidx_seq_len64(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    format: crate::htslib_rs::faidx::fai_format_options,
    expected: *const c_char,
    seq: *const c_char,
) -> c_int {
    let fai = test_test_faidx_c_87_load_index(fn_, fnfai, fngzi, 0, format);
    if fai.is_null() {
        return -1;
    }
    let found_len = crate::htslib_rs::faidx::faidx_seq_len(fai, seq) as hts_pos_t;
    crate::htslib_rs::faidx::fai_destroy(fai);
    if !expected.is_null() {
        let exp_len = libc::strtoll(expected, std::ptr::null_mut(), 10);
        if found_len != exp_len {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Unexpected result %lld from fai_seq_len64(fai, %s) expected %s\n".as_ptr(),
                found_len,
                seq,
                expected,
            );
            return -1;
        }
    } else {
        libc::printf(c"%lld\n".as_ptr(), found_len);
    }
    0
}

// original: usage (htslib/test/test_faidx.c:394)
pub unsafe fn test_test_faidx_c_394_usage(out: *mut libc::FILE, arg0: *const c_char) {
    libc::fprintf(
        out,
        c"Usage: %s [-c] -i fasta/q [-f fai_file] [-g gzi_file] [-e expected_fai]\n       %s [-cQ] -i fasta/q [-f fai_file] [-g gzi_file] [region]\n       %s -t FUNC -i fasta/q [-f fai_file] [-g gzi_file] [-e expected] <PARAM>\n       %s -h\n".as_ptr(),
        arg0,
        arg0,
        arg0,
        arg0,
    );
}

// original: help (htslib/test/test_faidx.c:403)
pub unsafe fn test_test_faidx_c_403_help(out: *mut libc::FILE, arg0: *const c_char) {
    test_test_faidx_c_394_usage(out, arg0);
    libc::fprintf(
        out,
        c"Options:\n  -i FILE      Input file\n  -f FILE      Fasta/q index file name\n  -g FILE      Bgzip index file name\n  -o FILE      Output file name\n  -e FILE|STR  Expected output\n  -c           Set FAI_CREATE flag\n  -Q           Output fastq format\n  -t FUNC      Test function\n  -h           Print this help\n\nExpected output is compared to the FAI file in indexing mode; the output file\nin retrieval mode; expected output for various -t function tests.\n\nUnit tests (-t option):\n   fai_line_length, faidx_has_seq, faidx_iseq, faidx_seq_len, faidx_seq_len64\nIn retrieval mode, -t can change the functions used to fetch data:\n   fai_fetch, fai_fetch64, faidx_fetch_seq, faidx_fetch_seq64,\n   fai_parse_region, fai_adjust_region\n\n".as_ptr(),
    );
}

// original: main (htslib/test/test_faidx.c:430)
pub unsafe fn test_test_faidx_c_430_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut fn_: *const c_char = std::ptr::null();
    let mut fnout: *const c_char = std::ptr::null();
    let mut fnfai: *const c_char = std::ptr::null();
    let mut fngzi: *const c_char = std::ptr::null();
    let mut expected: *const c_char = std::ptr::null();
    let mut func: *const c_char = c"".as_ptr();
    let mut flags: c_int = 0;
    let mut format = faidx::FAI_FASTA;

    loop {
        let opt = libc::getopt(argc, argv, c"i:f:g:o:e:t:cQh".as_ptr());
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
                test_test_faidx_c_403_help(crate::htslib_rs::c_compat::stdout.cast(), *argv);
                return libc::EXIT_SUCCESS;
            }
            _ => {
                test_test_faidx_c_394_usage(crate::htslib_rs::c_compat::stderr.cast(), *argv);
                return libc::EXIT_FAILURE;
            }
        }
    }

    if fn_.is_null() {
        test_test_faidx_c_394_usage(crate::htslib_rs::c_compat::stderr.cast(), *argv);
        return libc::EXIT_FAILURE;
    }

    let res = if optind == argc {
        let mut res = faidx::fai_build3(fn_, fnfai, fngzi);
        if res != 0 {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"Failed: fai_build3(%s, %s, %s)\n".as_ptr(),
                fn_,
                if fnfai.is_null() {
                    c"NULL".as_ptr()
                } else {
                    fnfai
                },
                if fngzi.is_null() {
                    c"NULL".as_ptr()
                } else {
                    fngzi
                },
            );
        } else if !expected.is_null() {
            res = test_test_faidx_c_33_file_compare(fnfai, expected);
        }
        res
    } else if libc::strcmp(func, c"fai_line_length".as_ptr()) == 0 {
        test_test_faidx_c_260_test_fai_line_length(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if libc::strcmp(func, c"faidx_has_seq".as_ptr()) == 0 {
        test_test_faidx_c_285_test_faidx_has_seq(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if libc::strcmp(func, c"faidx_iseq".as_ptr()) == 0 {
        test_test_faidx_c_310_test_faidx_iseq(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if libc::strcmp(func, c"faidx_seq_len".as_ptr()) == 0 {
        test_test_faidx_c_339_test_faidx_seq_len(
            fn_,
            fnfai,
            fngzi,
            format,
            expected,
            *argv.add(optind as usize),
        )
    } else if libc::strcmp(func, c"faidx_seq_len64".as_ptr()) == 0 {
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
    use std::ffi::CString;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static GETOPT_LOCK: Mutex<()> = Mutex::new(());

    fn fixture(path: &str) -> CString {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        CString::new(path.to_string_lossy().as_bytes()).unwrap()
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

    unsafe fn run_main(args: &[CString]) -> c_int {
        // NOTE: callers must already hold `ORIGINAL_MAIN_LOCK` (see
        // src/test/mod.rs). `GETOPT_LOCK` is retained for backward-compat
        // but is now effectively a no-op while the global lock is held.
        let _guard = GETOPT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        optarg = std::ptr::null_mut();
        // optind = 0 forces glibc getopt full reinit (shared-process tests).
        optind = 0;
        let mut argv = args
            .iter()
            .map(|arg| arg.as_ptr() as *mut c_char)
            .collect::<Vec<_>>();
        test_test_faidx_c_430_main(argv.len() as c_int, argv.as_mut_ptr())
    }

    #[test]
    fn original_test_faidx_main_fastq_retrieval_compares_expected_output() {
        // Process-wide lock for cross-file isolation (see src/test/mod.rs).
        let _cwd = crate::htslib_rs::test::CwdGuard::new();
        let _global = crate::htslib_rs::test::ORIGINAL_MAIN_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let out_path = temp_path("fastq-retrieval").with_extension("fq");
        let out = CString::new(out_path.to_string_lossy().as_bytes()).unwrap();
        let args = vec![
            c"test_faidx".into(),
            c"-i".into(),
            fixture("htslib/test/faidx/fastqs.fq"),
            c"-f".into(),
            fixture("htslib/test/faidx/fastqs.fq.expected.fai"),
            c"-o".into(),
            out.clone(),
            c"-e".into(),
            fixture("htslib/test/faidx/fastqs.1.expected.fq"),
            c"-Q".into(),
            c"FAKE0006_1:4-12".into(),
            c"FSRRS4401BE7HA_1:81-120".into(),
            c"FAKE0010_2".into(),
            c"SRR014849.50939_3:71-90".into(),
        ];

        unsafe {
            assert_eq!(run_main(&args), libc::EXIT_SUCCESS);
        }
        let _ = std::fs::remove_file(out_path);
    }
}
