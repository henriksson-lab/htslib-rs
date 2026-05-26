use crate::htslib_rs::{
    bgzf::{
        bgzf_block_write, bgzf_close, bgzf_compression, bgzf_dopen, bgzf_flush_try,
        bgzf_index_build_init, bgzf_index_dump, bgzf_index_load, bgzf_mt, bgzf_open, bgzf_read,
        bgzf_read_block_data, bgzf_useek, bgzf_write, bgzf_write_direct_block,
    },
    hfile::{hclose, hclose_abruptly, hopen, htslib_hfile_h_247_hread},
    hts::{
        htsFormat, hts_detect_format, hts_version, HTS_COMPRESSION_NO_COMPRESSION, HTS_FORMAT_BED,
        HTS_FORMAT_FAI_FORMAT, HTS_FORMAT_FASTA_FORMAT, HTS_FORMAT_FASTQ_FORMAT,
        HTS_FORMAT_FQI_FORMAT, HTS_FORMAT_SAM, HTS_FORMAT_TEXT_FORMAT, HTS_FORMAT_VCF,
    },
};
use std::ffi::{c_char, c_int};
use std::ptr;

const WINDOW_SIZE: usize = 0xff00;
const NO_ARGUMENT: c_int = 0;
const REQUIRED_ARGUMENT: c_int = 1;

#[repr(C)]
struct GetoptLongOption {
    name: *const c_char,
    has_arg: c_int,
    flag: *mut c_int,
    val: c_int,
}

unsafe extern "C" {
    fn getopt_long(
        argc: c_int,
        argv: *mut *mut c_char,
        optstring: *const c_char,
        longopts: *const GetoptLongOption,
        longindex: *mut c_int,
    ) -> c_int;
    static mut optarg: *mut c_char;
    static mut optind: c_int;
}

// original: error (htslib/bgzip.c:51)
// Rust has no stable C-style variadic function definitions; call sites below
// translate the original fatal fprintf + exit behaviour directly.

// original: ask_yn (htslib/bgzip.c:59)
unsafe fn bgzip_c_59_ask_yn() -> c_int {
    let mut line = [0 as c_char; 1024];
    if libc::fgets(
        line.as_mut_ptr(),
        line.len() as c_int,
        hts_sys::stdin.cast(),
    )
    .is_null()
    {
        return 0;
    }
    (line[0] == b'Y' as c_char || line[0] == b'y' as c_char) as c_int
}

// original: confirm_overwrite (htslib/bgzip.c:68)
pub unsafe fn bgzip_c_68_confirm_overwrite(fn_: *const c_char) -> c_int {
    let save_errno = *libc::__errno_location();
    let mut ret = 0;

    if libc::isatty(libc::STDIN_FILENO) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] %s already exists; do you wish to overwrite (y or n)? ".as_ptr(),
            fn_,
        );
        if bgzip_c_59_ask_yn() != 0 {
            ret = 1;
        }
    }

    *libc::__errno_location() = save_errno;
    ret
}

// original: known_extension (htslib/bgzip.c:82)
pub unsafe fn bgzip_c_82_known_extension(ext: *const c_char) -> c_int {
    let known = [c"gz".as_ptr(), c"bgz".as_ptr(), c"bgzf".as_ptr()];
    for k in known {
        if libc::strcasecmp(ext, k) == 0 {
            return 1;
        }
    }
    0
}

// original: confirm_filename (htslib/bgzip.c:95)
pub unsafe fn bgzip_c_95_confirm_filename(
    is_forced: *mut c_int,
    name: *const c_char,
    ext: *const c_char,
) -> c_int {
    if *is_forced != 0 {
        *is_forced -= 1;
        return 1;
    }

    if libc::isatty(libc::STDIN_FILENO) == 0 {
        return 0;
    }

    libc::fprintf(
        hts_sys::stderr.cast(),
        c"[bgzip] .%s is not a known extension; do you wish to decompress to %s (y or n)? "
            .as_ptr(),
        ext,
        name,
    );
    bgzip_c_59_ask_yn()
}

// original: getfilespec (htslib/bgzip.c:114)
pub unsafe fn bgzip_c_114_getfilespec(path: *const c_char, status: *mut libc::stat) -> c_int {
    if path.is_null() || status.is_null() {
        return -1;
    }
    if libc::strcmp(path, c"-".as_ptr()) == 0 {
        return 0;
    }
    if libc::stat(path, status) < 0 {
        return -1;
    }
    0
}

// original: setfilespec (htslib/bgzip.c:134)
pub unsafe fn bgzip_c_134_setfilespec(path: *const c_char, status: *const libc::stat) -> c_int {
    if path.is_null() || status.is_null() {
        return -1;
    }
    if libc::strcmp(path, c"-".as_ptr()) == 0 {
        return 0;
    }

    let mut tval = [
        libc::timeval {
            tv_sec: (*status).st_atime,
            tv_usec: 0,
        },
        libc::timeval {
            tv_sec: (*status).st_mtime,
            tv_usec: 0,
        },
    ];
    if libc::utimes(path, tval.as_mut_ptr()) < 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] Failed to set file specifications.\n".as_ptr(),
        );
        return -1;
    }
    0
}

// original: check_name_and_extension (htslib/bgzip.c:168)
pub unsafe fn bgzip_c_168_check_name_and_extension(name: *mut c_char, forced: *mut c_int) -> c_int {
    let mut pos = libc::strlen(name);

    while pos > 0 {
        if *name.add(pos) == b'.' as c_char || *name.add(pos) == b'/' as c_char {
            break;
        }
        pos -= 1;
    }

    if pos == 0 || *name.add(pos) != b'.' as c_char {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] can't find an extension in %s -- please rename\n".as_ptr(),
            name,
        );
        return 1;
    }

    *name.add(pos) = 0;
    let ext = name.add(pos + 1);

    if !(bgzip_c_82_known_extension(ext) != 0
        || bgzip_c_95_confirm_filename(forced, name, ext) != 0)
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] unknown extension .%s -- declining to decompress to %s\n".as_ptr(),
            ext,
            name,
        );
        return 2;
    }

    0
}

// original: bgzip_main_usage (htslib/bgzip.c:192)
pub unsafe fn bgzip_c_192_bgzip_main_usage(fp: *mut libc::FILE, status: c_int) -> c_int {
    libc::fprintf(fp, c"\n".as_ptr());
    libc::fprintf(fp, c"Version: %s\n".as_ptr(), hts_version());
    libc::fprintf(fp, c"Usage:   bgzip [OPTIONS] [FILE] ...\n".as_ptr());
    libc::fprintf(fp, c"Options:\n".as_ptr());
    libc::fprintf(
        fp,
        c"   -b, --offset INT           decompress at virtual file pointer (0-based uncompressed offset)\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -c, --stdout               write on standard output, keep original files unchanged\n"
            .as_ptr(),
    );
    libc::fprintf(fp, c"   -d, --decompress           decompress\n".as_ptr());
    libc::fprintf(
        fp,
        c"   -f, --force                overwrite files without asking\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -g, --rebgzip              use an index file to bgzip a file\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -h, --help                 give this help\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -i, --index                compress and create BGZF index\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -I, --index-name FILE      name of BGZF index file [file.gz.gzi]\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -k, --keep                 don't delete input files during operation\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -l, --compress-level INT   Compression level to use when compressing; 0 to 9, or -1 for default [-1]\n"
            .as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -o, --output FILE          write to file, keep original files unchanged\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -r, --reindex              (re)index compressed file\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -s, --size INT             decompress INT bytes (uncompressed size)\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -t, --test                 test integrity of compressed file\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"       --binary               Don't align blocks with text lines\n".as_ptr(),
    );
    libc::fprintf(
        fp,
        c"   -@, --threads INT          number of compression threads to use [1]\n".as_ptr(),
    );
    status
}

// original: main (htslib/bgzip.c:217)
pub unsafe fn bgzip_c_217_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut c: c_int;
    let mut compress_level: c_int = -1;
    let mut index: c_int = 0;
    let mut rebgzip: c_int = 0;
    let mut reindex: c_int = 0;
    let mut fp: *mut crate::htslib_rs::hts::BGZF = ptr::null_mut();
    let mut buffer: *mut c_char;
    let mut start: libc::c_long = 0;
    let mut end: libc::c_long = -1;
    let mut size: libc::c_long = -1;
    let mut filestat: libc::stat = std::mem::zeroed();
    let mut statfilename: *mut c_char;
    let mut index_fname: *mut c_char = ptr::null_mut();
    let mut write_fname: *mut c_char = ptr::null_mut();
    let mut threads: c_int = 1;
    let mut isstdin: c_int;
    let mut usedstdout: c_int = 0;
    let mut ret: c_int = 0;
    let mut exp_out_open: c_int = 0;
    let mut f_dst: c_int = -1;

    let mut compress: c_int = 1;
    let mut pstdout: c_int = 0;
    let mut is_forced: c_int = 0;
    let mut test: c_int = 0;
    let mut keep: c_int = 0;
    let mut binary: c_int = 0;

    let mut loptions = [
        GetoptLongOption {
            name: c"help".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'h' as c_int,
        },
        GetoptLongOption {
            name: c"offset".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'b' as c_int,
        },
        GetoptLongOption {
            name: c"stdout".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'c' as c_int,
        },
        GetoptLongOption {
            name: c"decompress".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'd' as c_int,
        },
        GetoptLongOption {
            name: c"force".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'f' as c_int,
        },
        GetoptLongOption {
            name: c"index".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'i' as c_int,
        },
        GetoptLongOption {
            name: c"index-name".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'I' as c_int,
        },
        GetoptLongOption {
            name: c"compress-level".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'l' as c_int,
        },
        GetoptLongOption {
            name: c"reindex".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'r' as c_int,
        },
        GetoptLongOption {
            name: c"rebgzip".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'g' as c_int,
        },
        GetoptLongOption {
            name: c"size".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b's' as c_int,
        },
        GetoptLongOption {
            name: c"threads".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'@' as c_int,
        },
        GetoptLongOption {
            name: c"test".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b't' as c_int,
        },
        GetoptLongOption {
            name: c"version".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 1,
        },
        GetoptLongOption {
            name: c"keep".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'k' as c_int,
        },
        GetoptLongOption {
            name: c"binary".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 2,
        },
        GetoptLongOption {
            name: c"output".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'o' as c_int,
        },
        GetoptLongOption {
            name: ptr::null(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: 0,
        },
    ];

    loop {
        c = getopt_long(
            argc,
            argv,
            c"cdh?fb:@:s:iI:l:grtko:".as_ptr(),
            loptions.as_mut_ptr(),
            ptr::null_mut(),
        );
        if c < 0 {
            break;
        }
        match c {
            x if x == b'd' as c_int => compress = 0,
            x if x == b'c' as c_int => pstdout = 1,
            x if x == b'b' as c_int => {
                start = libc::atol(optarg);
                compress = 0;
                pstdout = 1;
            }
            x if x == b's' as c_int => {
                size = libc::atol(optarg);
                pstdout = 1;
            }
            x if x == b'f' as c_int => is_forced += 1,
            x if x == b'i' as c_int => index = 1,
            x if x == b'I' as c_int => index_fname = optarg,
            x if x == b'l' as c_int => compress_level = libc::atol(optarg) as c_int,
            x if x == b'g' as c_int => rebgzip = 1,
            x if x == b'r' as c_int => {
                reindex = 1;
                compress = 0;
            }
            x if x == b'@' as c_int => threads = libc::atoi(optarg),
            x if x == b't' as c_int => {
                test = 1;
                compress = 0;
                reindex = 0;
            }
            x if x == b'k' as c_int => keep = 1,
            x if x == b'o' as c_int => write_fname = optarg,
            1 => {
                libc::printf(
                    c"bgzip (htslib) %s\nCopyright (C) 2025 Genome Research Ltd.\n".as_ptr(),
                    hts_version(),
                );
                return libc::EXIT_SUCCESS;
            }
            2 => binary = 1,
            x if x == b'h' as c_int => {
                return bgzip_c_192_bgzip_main_usage(hts_sys::stdout.cast(), libc::EXIT_SUCCESS)
            }
            x if x == b'?' as c_int => {
                return bgzip_c_192_bgzip_main_usage(hts_sys::stderr.cast(), libc::EXIT_FAILURE)
            }
            _ => {}
        }
    }

    if size >= 0 {
        end = start + size;
    }
    if end >= 0 && end < start {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] Illegal region: [%ld, %ld]\n".as_ptr(),
            start,
            end,
        );
        return 1;
    }

    if (index != 0 || reindex != 0) && rebgzip != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] Can't produce a index and rebgzip simultaneously\n".as_ptr(),
        );
        return 1;
    }
    if rebgzip != 0 && index_fname.is_null() {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] Index file name expected with rebgzip.  See -I option.\n".as_ptr(),
        );
        return 1;
    }
    if (index != 0 || reindex != 0)
        && write_fname.is_null()
        && !index_fname.is_null()
        && argc - optind > 1
    {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"[bgzip] Cannot specify index filename with multiple data file on index, reindex.\n"
                .as_ptr(),
        );
        return 1;
    }

    if !write_fname.is_null() {
        if pstdout != 0 {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"[bgzip] Cannot write to %s and stdout at the same time.\n".as_ptr(),
                write_fname,
            );
            return 1;
        } else if libc::strncmp(write_fname, c"-".as_ptr(), libc::strlen(write_fname)) == 0 {
            pstdout = 1;
            write_fname = ptr::null_mut();
        }
    }

    loop {
        isstdin = if optind >= argc {
            1
        } else {
            (libc::strcmp(c"-".as_ptr(), *argv.add(optind as usize)) == 0) as c_int
        };

        if write_fname.is_null() {
            usedstdout |= isstdin | pstdout | test;
        }

        statfilename = ptr::null_mut();

        if compress == 1 {
            let mut out_mode = [b'w' as c_char, 0, 0];
            let mut out_mode_exclusive = [b'w' as c_char, b'x' as c_char, 0, 0];

            if compress_level < -1 || compress_level > 9 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"[bgzip] Invalid compress-level: %d\n".as_ptr(),
                    compress_level,
                );
                return 1;
            }
            if compress_level >= 0 {
                out_mode[1] = (compress_level + b'0' as c_int) as c_char;
                out_mode_exclusive[2] = (compress_level + b'0' as c_int) as c_char;
            }
            let f_src = hopen(
                if isstdin == 0 {
                    *argv.add(optind as usize)
                } else {
                    c"-".as_ptr().cast_mut()
                },
                c"r".as_ptr(),
            );
            if f_src.is_null() {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"[bgzip] %s: %s\n".as_ptr(),
                    libc::strerror(*libc::__errno_location()),
                    if isstdin != 0 {
                        c"stdin".as_ptr()
                    } else {
                        *argv.add(optind as usize)
                    },
                );
                return 1;
            }

            if !write_fname.is_null() {
                if exp_out_open == 0 {
                    fp = bgzf_open(write_fname, out_mode.as_ptr());
                    if fp.is_null() {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"[bgzip] can't create %s: %s\n".as_ptr(),
                            write_fname,
                            libc::strerror(*libc::__errno_location()),
                        );
                        return 1;
                    } else {
                        exp_out_open = 1;
                    }
                }
            } else if argc > optind && isstdin == 0 {
                if pstdout != 0 {
                    fp = bgzf_dopen(libc::STDOUT_FILENO, out_mode.as_ptr());
                } else {
                    let name =
                        libc::malloc(libc::strlen(*argv.add(optind as usize)) + 5).cast::<c_char>();
                    libc::strcpy(name, *argv.add(optind as usize));
                    libc::strcat(name, c".gz".as_ptr());
                    fp = bgzf_open(
                        name,
                        if is_forced != 0 {
                            out_mode.as_ptr()
                        } else {
                            out_mode_exclusive.as_ptr()
                        },
                    );
                    if fp.is_null() && *libc::__errno_location() == libc::EEXIST {
                        if bgzip_c_68_confirm_overwrite(name) != 0 {
                            fp = bgzf_open(name, out_mode.as_ptr());
                        } else {
                            ret = 2;
                            hclose_abruptly(f_src);
                            libc::free(name.cast());
                            optind += 1;
                            if optind >= argc {
                                break;
                            }
                            continue;
                        }
                    }
                    if fp.is_null() {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"[bgzip] can't create %s: %s\n".as_ptr(),
                            name,
                            libc::strerror(*libc::__errno_location()),
                        );
                        libc::free(name.cast());
                        return 1;
                    }
                    statfilename = name;
                }
            } else if pstdout == 0 && libc::isatty(libc::fileno(hts_sys::stdout.cast())) != 0 {
                return bgzip_c_192_bgzip_main_usage(hts_sys::stderr.cast(), libc::EXIT_FAILURE);
            } else if index != 0 && index_fname.is_null() {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"[bgzip] Index file name expected when writing to stdout\n".as_ptr(),
                );
                return 1;
            } else {
                fp = bgzf_dopen(libc::STDOUT_FILENO, out_mode.as_ptr());
            }

            if index != 0 {
                bgzf_index_build_init(fp);
            }
            if threads > 1 {
                if bgzf_mt(fp, threads, 256) != 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] threaded BGZF is not yet supported in this translation\n"
                            .as_ptr(),
                    );
                    bgzf_close(fp);
                    if !statfilename.is_null() {
                        libc::free(statfilename.cast());
                    }
                    hclose_abruptly(f_src);
                    return 1;
                }
            }

            buffer = libc::malloc(WINDOW_SIZE).cast::<c_char>();
            if buffer.is_null() {
                if !statfilename.is_null() {
                    libc::free(statfilename.cast());
                }
                return 1;
            }
            if rebgzip != 0 {
                if bgzf_index_load(fp, index_fname, ptr::null()) < 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Could not load index: %s.%s\n".as_ptr(),
                        if isstdin == 0 {
                            *argv.add(optind as usize)
                        } else {
                            index_fname
                        },
                        if isstdin == 0 {
                            c"gzi".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }

                loop {
                    c = htslib_hfile_h_247_hread(f_src, buffer.cast(), WINDOW_SIZE) as c_int;
                    if c <= 0 {
                        break;
                    }
                    if bgzf_block_write(fp, buffer.cast(), c as usize) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write %d bytes: Error %d\n".as_ptr(),
                            c,
                            (*fp).bitfields >> 16,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
            } else {
                let mut fmt: htsFormat = std::mem::zeroed();
                let mut textual = 0;
                if binary == 0
                    && hts_detect_format(f_src, &mut fmt) == 0
                    && fmt.compression == HTS_COMPRESSION_NO_COMPRESSION
                {
                    match fmt.format {
                        HTS_FORMAT_TEXT_FORMAT
                        | HTS_FORMAT_SAM
                        | HTS_FORMAT_VCF
                        | HTS_FORMAT_BED
                        | HTS_FORMAT_FASTA_FORMAT
                        | HTS_FORMAT_FASTQ_FORMAT
                        | HTS_FORMAT_FAI_FORMAT
                        | HTS_FORMAT_FQI_FORMAT => textual = 1,
                        _ => {}
                    }
                }

                if binary != 0 || textual == 0 {
                    loop {
                        c = htslib_hfile_h_247_hread(f_src, buffer.cast(), WINDOW_SIZE) as c_int;
                        if c <= 0 {
                            break;
                        }
                        if bgzf_write(fp, buffer.cast(), c as usize) < 0 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"Could not write %d bytes: Error %d\n".as_ptr(),
                                c,
                                (*fp).bitfields >> 16,
                            );
                            libc::exit(libc::EXIT_FAILURE);
                        }
                    }
                } else {
                    let mut in_header = 1;
                    let mut n: c_int = 0;
                    let mut long_line = 0;
                    loop {
                        c = htslib_hfile_h_247_hread(
                            f_src,
                            buffer.add(n as usize).cast(),
                            WINDOW_SIZE - n as usize,
                        ) as c_int;
                        if c <= 0 {
                            break;
                        }
                        let c2 = c + n;
                        let mut flush = 0;
                        if in_header != 0
                            && (long_line != 0
                                || *buffer == b'@' as c_char
                                || *buffer == b'#' as c_char)
                        {
                            let mut last_start = 0;
                            n = 0;
                            while n < c2 {
                                if *buffer.add(n as usize) != b'\n' as c_char {
                                    n += 1;
                                    continue;
                                }
                                n += 1;

                                last_start = n;
                                if n < c2
                                    && !(*buffer.add(n as usize) == b'@' as c_char
                                        || *buffer.add(n as usize) == b'#' as c_char)
                                {
                                    in_header = 0;
                                    break;
                                }
                            }
                            if last_start == 0 {
                                n = c2;
                                long_line = 1;
                            } else {
                                n = last_start;
                                flush = 1;
                                long_line = 0;
                            }
                        } else {
                            n += c;
                            loop {
                                n -= 1;
                                if n < 0 || *buffer.add(n as usize) == b'\n' as c_char {
                                    break;
                                }
                            }

                            if n >= 0 {
                                flush = 1;
                                n += 1;
                            } else {
                                n = c2;
                            }
                        }

                        let wrote = if flush != 0 && (*fp).block_offset == 0 {
                            bgzf_write_direct_block(fp, buffer.cast(), n as usize)
                        } else {
                            bgzf_write(fp, buffer.cast(), n as usize)
                        };
                        if wrote < 0 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"Could not write %d bytes: Error %d\n".as_ptr(),
                                n,
                                (*fp).bitfields >> 16,
                            );
                            libc::exit(libc::EXIT_FAILURE);
                        }
                        if flush != 0 && (*fp).block_offset != 0 && bgzf_flush_try(fp, 65536) < 0 {
                            if !statfilename.is_null() {
                                libc::free(statfilename.cast());
                            }
                            return -1;
                        }

                        libc::memmove(
                            buffer.cast(),
                            buffer.add(n as usize).cast(),
                            (c2 - n) as usize,
                        );
                        n = c2 - n;
                    }

                    if bgzf_write(fp, buffer.cast(), n as usize) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write %d bytes: Error %d\n".as_ptr(),
                            n,
                            (*fp).bitfields >> 16,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
            }
            if index != 0 && write_fname.is_null() {
                if !index_fname.is_null() {
                    if bgzf_index_dump(fp, index_fname, ptr::null()) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write index to '%s'\n".as_ptr(),
                            index_fname,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else if isstdin == 0 {
                    if bgzf_index_dump(fp, *argv.add(optind as usize), c".gz.gzi".as_ptr()) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write index to '%s.gz.gzi'\n".as_ptr(),
                            *argv.add(optind as usize),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Can not write index for stdin data without index filename, use -I option to set index file.\n".as_ptr(),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            if write_fname.is_null() && bgzf_close(fp) < 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Output close failed: Error %d\n".as_ptr(),
                    (*fp).bitfields >> 16,
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if hclose(f_src) < 0 {
                libc::fprintf(hts_sys::stderr.cast(), c"Input close failed\n".as_ptr());
                libc::exit(libc::EXIT_FAILURE);
            }

            if !statfilename.is_null() {
                if bgzip_c_114_getfilespec(*argv.add(optind as usize), &mut filestat) == 0 {
                    if bgzip_c_134_setfilespec(statfilename, &filestat) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"[bgzip] Failed to set file specification.\n".as_ptr(),
                        );
                    }
                } else {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] Failed to get file specification.\n".as_ptr(),
                    );
                }
                libc::free(statfilename.cast());
            }

            if argc > optind && pstdout == 0 && keep == 0 && isstdin == 0 && write_fname.is_null() {
                libc::unlink(*argv.add(optind as usize));
            }

            libc::free(buffer.cast());
        } else if reindex != 0 {
            if argc > optind && isstdin == 0 {
                fp = bgzf_open(*argv.add(optind as usize), c"r".as_ptr());
                if fp.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] Could not open file: %s\n".as_ptr(),
                        *argv.add(optind as usize),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            } else {
                if index_fname.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] Index file name expected when reading from stdin\n".as_ptr(),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
                fp = bgzf_open(c"-".as_ptr(), c"r".as_ptr());
                if fp.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] Could not read from stdin: %s\n".as_ptr(),
                        libc::strerror(*libc::__errno_location()),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            buffer = libc::malloc(WINDOW_SIZE).cast::<c_char>();
            bgzf_index_build_init(fp);
            let mut read_ret: isize;
            loop {
                read_ret = bgzf_read(fp, buffer.cast(), WINDOW_SIZE);
                if read_ret <= 0 {
                    break;
                }
            }
            libc::free(buffer.cast());
            if read_ret < 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Is the file gzipped or bgzipped? The latter is required for indexing.\n"
                        .as_ptr(),
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if !index_fname.is_null() {
                if bgzf_index_dump(fp, index_fname, ptr::null()) < 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Could not write index to '%s'\n".as_ptr(),
                        index_fname,
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            } else if isstdin == 0 {
                if bgzf_index_dump(fp, *argv.add(optind as usize), c".gzi".as_ptr()) < 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Could not write index to '%s.gzi'\n".as_ptr(),
                        *argv.add(optind as usize),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            } else {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Can not write index for stdin data without index filename, use -I option to set index file.\n".as_ptr(),
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if bgzf_close(fp) < 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Close failed: Error %d\n".as_ptr(),
                    (*fp).bitfields >> 16,
                );
                libc::exit(libc::EXIT_FAILURE);
            }
        } else {
            let mut is_forced_tmp = is_forced;

            if argc > optind && isstdin == 0 {
                fp = bgzf_open(*argv.add(optind as usize), c"r".as_ptr());
                if fp.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] Could not open %s: %s\n".as_ptr(),
                        *argv.add(optind as usize),
                        libc::strerror(*libc::__errno_location()),
                    );
                    return 1;
                }
                if bgzf_compression(fp) == HTS_COMPRESSION_NO_COMPRESSION as c_int {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] %s: not a compressed file -- ignored\n".as_ptr(),
                        *argv.add(optind as usize),
                    );
                    bgzf_close(fp);
                    return 1;
                }

                if pstdout != 0 || test != 0 {
                    f_dst = libc::fileno(hts_sys::stdout.cast());
                } else {
                    let wrflags = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
                    let name = libc::strdup(*argv.add(optind as usize));
                    if name.is_null() {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"[bgzip] unable to allocate memory for output file name.\n".as_ptr(),
                        );
                        bgzf_close(fp);
                        return 1;
                    }

                    let check = bgzip_c_168_check_name_and_extension(name, &mut is_forced_tmp);
                    if check != 0 {
                        bgzf_close(fp);

                        if check == 1 {
                            return 1;
                        } else {
                            ret = 2;
                            optind += 1;
                            if optind >= argc {
                                break;
                            }
                            continue;
                        }
                    }

                    if exp_out_open == 0 {
                        if !write_fname.is_null() {
                            is_forced_tmp = 1;
                            exp_out_open = 1;
                        }

                        f_dst = libc::open(
                            if !write_fname.is_null() {
                                write_fname
                            } else {
                                name
                            },
                            if is_forced_tmp != 0 {
                                wrflags
                            } else {
                                wrflags | libc::O_EXCL
                            },
                            0o666,
                        );

                        if f_dst < 0 && *libc::__errno_location() == libc::EEXIST {
                            if bgzip_c_68_confirm_overwrite(name) != 0 {
                                f_dst = libc::open(name, wrflags, 0o666);
                            } else {
                                ret = 2;
                                bgzf_close(fp);
                                libc::free(name.cast());
                                optind += 1;
                                if optind >= argc {
                                    break;
                                }
                                continue;
                            }
                        }
                        if f_dst < 0 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"[bgzip] can't create %s: %s\n".as_ptr(),
                                name,
                                libc::strerror(*libc::__errno_location()),
                            );
                            libc::free(name.cast());
                            return 1;
                        }
                    }

                    statfilename = name;
                }
            } else if pstdout == 0 && libc::isatty(libc::fileno(hts_sys::stdin.cast())) != 0 {
                return bgzip_c_192_bgzip_main_usage(hts_sys::stderr.cast(), libc::EXIT_FAILURE);
            } else {
                f_dst = libc::fileno(hts_sys::stdout.cast());
                fp = bgzf_open(c"-".as_ptr(), c"r".as_ptr());
                if fp.is_null() {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] Could not read from stdin: %s\n".as_ptr(),
                        libc::strerror(*libc::__errno_location()),
                    );
                    return 1;
                }
                if bgzf_compression(fp) == HTS_COMPRESSION_NO_COMPRESSION as c_int {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] stdin is not compressed -- ignored\n".as_ptr(),
                    );
                    bgzf_close(fp);
                    return 1;
                }

                if write_fname.is_null() {
                    f_dst = libc::fileno(hts_sys::stdout.cast());
                } else if exp_out_open == 0 {
                    exp_out_open = 1;

                    f_dst = libc::open(
                        write_fname,
                        libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                        0o666,
                    );

                    if f_dst < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"[bgzip] can't create %s: %s\n".as_ptr(),
                            write_fname,
                            libc::strerror(*libc::__errno_location()),
                        );
                        return 1;
                    }
                }
            }

            buffer = libc::malloc(WINDOW_SIZE).cast::<c_char>();
            if start > 0 {
                if !index_fname.is_null() {
                    if bgzf_index_load(fp, index_fname, ptr::null()) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not load index: %s\n".as_ptr(),
                            index_fname,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else {
                    if optind >= argc || isstdin != 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"The -b option requires -I when reading from stdin (and stdin must be seekable)\n".as_ptr(),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    if bgzf_index_load(fp, *argv.add(optind as usize), c".gzi".as_ptr()) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not load index: %s.gzi\n".as_ptr(),
                            *argv.add(optind as usize),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
                if bgzf_useek(fp, start, libc::SEEK_SET) < 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Could not seek to %ld-th (uncompressd) byte\n".as_ptr(),
                        start,
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            if threads > 1 {
                if bgzf_mt(fp, threads, 256) != 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"[bgzip] threaded BGZF is not yet supported in this translation\n"
                            .as_ptr(),
                    );
                    bgzf_close(fp);
                    return 1;
                }
            }

            let start_reg = start;
            let end_reg = end;
            if end < 0 && start == 0 {
                loop {
                    let mut block_data: *const libc::c_void = ptr::null();
                    c = bgzf_read_block_data(fp, &mut block_data) as c_int;
                    if c == 0 {
                        break;
                    }
                    if c < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Error %d in block starting at offset %ld(%lX)\n".as_ptr(),
                            (*fp).bitfields >> 16,
                            (*fp).block_address,
                            (*fp).block_address,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    if test == 0 && libc::write(f_dst, block_data, c as usize) != c as isize {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write %d bytes\n".as_ptr(),
                            c,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
            } else {
                loop {
                    c = bgzf_read(
                        fp,
                        buffer.cast(),
                        if end < 0 || end - start > WINDOW_SIZE as libc::c_long {
                            WINDOW_SIZE
                        } else {
                            (end - start) as usize
                        },
                    ) as c_int;
                    if c == 0 {
                        break;
                    }
                    if c < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Error %d in block starting at offset %ld(%lX)\n".as_ptr(),
                            (*fp).bitfields >> 16,
                            (*fp).block_address,
                            (*fp).block_address,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    start += c as libc::c_long;
                    if test == 0 && libc::write(f_dst, buffer.cast(), c as usize) != c as isize {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write %d bytes\n".as_ptr(),
                            c,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    if end >= 0 && start >= end {
                        break;
                    }
                }
            }
            start = start_reg;
            end = end_reg;
            libc::free(buffer.cast());
            if bgzf_close(fp) < 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Close failed: Error %d\n".as_ptr(),
                    (*fp).bitfields >> 16,
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if !statfilename.is_null() {
                if write_fname.is_null() {
                    if bgzip_c_114_getfilespec(*argv.add(optind as usize), &mut filestat) == 0 {
                        if bgzip_c_134_setfilespec(statfilename, &filestat) < 0 {
                            libc::fprintf(
                                hts_sys::stderr.cast(),
                                c"[bgzip] Failed to set file specification.\n".as_ptr(),
                            );
                        }
                    } else {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"[bgzip] Failed to get file specification.\n".as_ptr(),
                        );
                    }
                }

                libc::free(statfilename.cast());
            }

            if argc > optind
                && pstdout == 0
                && test == 0
                && keep == 0
                && isstdin == 0
                && write_fname.is_null()
            {
                libc::unlink(*argv.add(optind as usize));
            }
            if isstdin == 0 && pstdout == 0 && test == 0 && write_fname.is_null() {
                libc::close(f_dst);
            }
        }

        optind += 1;
        if optind >= argc {
            break;
        }
    }

    if usedstdout != 0 && reindex == 0 {
        if libc::fclose(hts_sys::stdout.cast()) != 0 && *libc::__errno_location() != libc::EBADF {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"[bgzip] Failed to close stdout, errno %d".as_ptr(),
                *libc::__errno_location(),
            );
            ret = 1;
        }
    } else if !write_fname.is_null() {
        if compress == 1 {
            if index != 0 {
                if !index_fname.is_null() {
                    if bgzf_index_dump(fp, index_fname, ptr::null()) < 0 {
                        libc::fprintf(
                            hts_sys::stderr.cast(),
                            c"Could not write index to '%s'\n".as_ptr(),
                            index_fname,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else if bgzf_index_dump(fp, write_fname, c".gzi".as_ptr()) < 0 {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Could not write index to '%s.gzi'\n".as_ptr(),
                        write_fname,
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            if bgzf_close(fp) < 0 {
                libc::fprintf(
                    hts_sys::stderr.cast(),
                    c"Output close failed: Error %d\n".as_ptr(),
                    (*fp).bitfields >> 16,
                );
                libc::exit(libc::EXIT_FAILURE);
            }
        } else {
            libc::close(f_dst);
        }
    }

    ret
}
