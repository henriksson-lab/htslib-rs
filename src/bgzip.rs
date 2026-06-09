use crate::htslib_rs::{
    bgzf::{
        bgzf_block_write, bgzf_close, bgzf_compression, bgzf_dopen, bgzf_flush_try,
        bgzf_index_build_init, bgzf_index_dump, bgzf_index_load, bgzf_mt, bgzf_open, bgzf_read,
        bgzf_read_block_data, bgzf_useek, bgzf_write, bgzf_write_direct_block,
    },
    hfile::{hclose, hclose_abruptly, hopen, htslib_hfile_h_247_hread},
    hts::{
        htsFormat, hts_detect_format, hts_version, BGZF, HTS_COMPRESSION_NO_COMPRESSION,
        HTS_FORMAT_BED, HTS_FORMAT_FAI_FORMAT, HTS_FORMAT_FASTA_FORMAT, HTS_FORMAT_FASTQ_FORMAT,
        HTS_FORMAT_FQI_FORMAT, HTS_FORMAT_SAM, HTS_FORMAT_TEXT_FORMAT, HTS_FORMAT_VCF,
    },
};
use std::ffi::{CStr, CString};
use std::ptr;
use std::ptr::NonNull;

const WINDOW_SIZE: usize = 0xff00;
const NO_ARGUMENT: i32 = 0;
const REQUIRED_ARGUMENT: i32 = 1;

#[repr(C)]
struct GetoptLongOption {
    name: *const i8,
    has_arg: i32,
    flag: *mut i32,
    val: i32,
}

unsafe extern "C" {
    fn getopt_long(
        argc: i32,
        argv: *mut *mut i8,
        optstring: *const i8,
        longopts: *const GetoptLongOption,
        longindex: *mut i32,
    ) -> i32;
    static mut optarg: *mut i8;
    static mut optind: i32;
}

// original: error (htslib/bgzip.c:51)
// Rust has no stable C-style variadic function definitions; call sites below
// translate the original fatal fprintf + exit behaviour directly.

// original: ask_yn (htslib/bgzip.c:59)
unsafe fn bgzip_c_59_ask_yn() -> i32 {
    let mut line = [0u8; 1024];
    if libc::fgets(
        line.as_mut_ptr().cast(),
        line.len() as i32,
        crate::htslib_rs::c_compat::stdin.cast(),
    )
    .is_null()
    {
        return 0;
    }
    (line[0] == b'Y' || line[0] == b'y') as i32
}

// original: confirm_overwrite (htslib/bgzip.c:68)
pub unsafe fn bgzip_c_68_confirm_overwrite(fn_: *const i8) -> i32 {
    let Some(fn_) = fn_.as_ref().map(|_| CStr::from_ptr(fn_)) else {
        return 0;
    };
    bgzip_confirm_overwrite(fn_)
}

unsafe fn bgzip_confirm_overwrite(fn_: &CStr) -> i32 {
    let save_errno = *crate::htslib_rs::c_compat::__errno_location();
    let mut ret = 0;

    if libc::isatty(libc::STDIN_FILENO) != 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] %s already exists; do you wish to overwrite (y or n)? ".as_ptr(),
            fn_.as_ptr(),
        );
        if bgzip_c_59_ask_yn() != 0 {
            ret = 1;
        }
    }

    *crate::htslib_rs::c_compat::__errno_location() = save_errno;
    ret
}

// original: known_extension (htslib/bgzip.c:82)
pub unsafe fn bgzip_c_82_known_extension(ext: *const i8) -> i32 {
    let Some(ext) = ext.as_ref().map(|_| CStr::from_ptr(ext)) else {
        return 0;
    };
    bgzip_known_extension(ext)
}

fn bgzip_known_extension(ext: &CStr) -> i32 {
    let ext = ext.to_bytes();
    (ext.eq_ignore_ascii_case(b"gz")
        || ext.eq_ignore_ascii_case(b"bgz")
        || ext.eq_ignore_ascii_case(b"bgzf")) as i32
}

// original: confirm_filename (htslib/bgzip.c:95)
pub unsafe fn bgzip_c_95_confirm_filename(
    is_forced: *mut i32,
    name: *const i8,
    ext: *const i8,
) -> i32 {
    let Some(is_forced) = is_forced.as_mut() else {
        return 0;
    };
    let Some(name) = name.as_ref().map(|_| CStr::from_ptr(name)) else {
        return 0;
    };
    let Some(ext) = ext.as_ref().map(|_| CStr::from_ptr(ext)) else {
        return 0;
    };
    bgzip_confirm_filename(is_forced, name, ext)
}

unsafe fn bgzip_confirm_filename(is_forced: &mut i32, name: &CStr, ext: &CStr) -> i32 {
    if *is_forced != 0 {
        *is_forced -= 1;
        return 1;
    }

    if libc::isatty(libc::STDIN_FILENO) == 0 {
        return 0;
    }

    libc::fprintf(
        crate::htslib_rs::c_compat::stderr.cast(),
        c"[bgzip] .%s is not a known extension; do you wish to decompress to %s (y or n)? "
            .as_ptr(),
        ext.as_ptr(),
        name.as_ptr(),
    );
    bgzip_c_59_ask_yn()
}

// original: getfilespec (htslib/bgzip.c:114)
pub unsafe fn bgzip_c_114_getfilespec(path: *const i8, status: *mut libc::stat) -> i32 {
    let Some(status) = status.as_mut() else {
        return -1;
    };
    let Some(path) = path.as_ref().map(|_| CStr::from_ptr(path)) else {
        return -1;
    };
    bgzip_getfilespec(path, status)
}

unsafe fn bgzip_getfilespec(path: &CStr, status: &mut libc::stat) -> i32 {
    if path.to_bytes() == b"-" {
        return 0;
    }
    if libc::stat(path.as_ptr(), status) < 0 {
        return -1;
    }
    0
}

// original: setfilespec (htslib/bgzip.c:134)
pub unsafe fn bgzip_c_134_setfilespec(path: *const i8, status: *const libc::stat) -> i32 {
    let Some(status) = status.as_ref() else {
        return -1;
    };
    let Some(path) = path.as_ref().map(|_| CStr::from_ptr(path)) else {
        return -1;
    };
    bgzip_setfilespec(path, status)
}

unsafe fn bgzip_setfilespec(path: &CStr, status: &libc::stat) -> i32 {
    if path.to_bytes() == b"-" {
        return 0;
    }

    let mut tval = [
        libc::timeval {
            tv_sec: status.st_atime as _,
            tv_usec: 0,
        },
        libc::timeval {
            tv_sec: status.st_mtime as _,
            tv_usec: 0,
        },
    ];
    if crate::htslib_rs::c_compat::utimes(path.as_ptr(), tval.as_mut_ptr()) < 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] Failed to set file specifications.\n".as_ptr(),
        );
        return -1;
    }
    0
}

// original: check_name_and_extension (htslib/bgzip.c:168)
pub unsafe fn bgzip_c_168_check_name_and_extension(name: *mut i8, forced: *mut i32) -> i32 {
    let Some(forced) = forced.as_mut() else {
        return 1;
    };
    if name.is_null() {
        return 1;
    }
    let len = CStr::from_ptr(name).to_bytes().len();
    let name = std::slice::from_raw_parts_mut(name.cast::<u8>(), len + 1);
    bgzip_check_name_and_extension(name, forced)
}

unsafe fn bgzip_check_name_and_extension(name: &mut [u8], forced: &mut i32) -> i32 {
    let nul = name.len().saturating_sub(1);
    let stem = &name[..nul];
    let pos = stem
        .iter()
        .rposition(|&b| b == b'.' || b == b'/')
        .unwrap_or(0);

    if pos == 0 || name[pos] != b'.' {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] can't find an extension in %s -- please rename\n".as_ptr(),
            name.as_ptr(),
        );
        return 1;
    }

    name[pos] = 0;
    let name_cstr = CStr::from_ptr(name.as_ptr().cast());
    let ext_cstr = CStr::from_ptr(name.as_ptr().add(pos + 1).cast());

    if !(bgzip_known_extension(ext_cstr) != 0
        || bgzip_confirm_filename(forced, name_cstr, ext_cstr) != 0)
    {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] unknown extension .%s -- declining to decompress to %s\n".as_ptr(),
            ext_cstr.as_ptr(),
            name_cstr.as_ptr(),
        );
        return 2;
    }

    0
}

unsafe fn bgzip_open_bgzf(path: &CStr, mode: &CStr) -> Option<NonNull<BGZF>> {
    NonNull::new(bgzf_open(path.as_ptr(), mode.as_ptr()))
}

unsafe fn bgzip_dopen_bgzf(fd: i32, mode: &CStr) -> Option<NonNull<BGZF>> {
    NonNull::new(bgzf_dopen(fd, mode.as_ptr()))
}

unsafe fn bgzip_close_bgzf(fp: &mut Option<NonNull<BGZF>>) -> (i32, u32) {
    let Some(fp) = fp.take() else {
        return (-1, 0);
    };
    let err = fp.as_ref().bitfields >> 16;
    (bgzf_close(fp.as_ptr()), err)
}

// original: bgzip_main_usage (htslib/bgzip.c:192)
pub unsafe fn bgzip_c_192_bgzip_main_usage(fp: *mut libc::FILE, status: i32) -> i32 {
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
pub unsafe fn bgzip_c_217_main(argc: i32, argv: *mut *mut i8) -> i32 {
    let mut c: i32;
    let mut compress_level: i32 = -1;
    let mut index = false;
    let mut rebgzip = false;
    let mut reindex = false;
    let mut fp: Option<NonNull<BGZF>> = None;
    let mut start: libc::c_long = 0;
    let mut end: libc::c_long = -1;
    let mut size: libc::c_long = -1;
    let mut filestat: libc::stat = std::mem::zeroed();
    let mut statfilename: Option<NonNull<i8>>;
    let mut index_fname: Option<&CStr> = None;
    let mut write_fname: Option<&CStr> = None;
    let mut threads: i32 = 1;
    let mut isstdin: bool;
    let mut usedstdout = false;
    let mut ret: i32 = 0;
    let mut exp_out_open = false;
    let mut f_dst: i32 = -1;

    let mut compress = true;
    let mut pstdout = false;
    let mut is_forced: i32 = 0;
    let mut test = false;
    let mut keep = false;
    let mut binary = false;

    let mut loptions = [
        GetoptLongOption {
            name: c"help".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'h' as i32,
        },
        GetoptLongOption {
            name: c"offset".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'b' as i32,
        },
        GetoptLongOption {
            name: c"stdout".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'c' as i32,
        },
        GetoptLongOption {
            name: c"decompress".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'd' as i32,
        },
        GetoptLongOption {
            name: c"force".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'f' as i32,
        },
        GetoptLongOption {
            name: c"index".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'i' as i32,
        },
        GetoptLongOption {
            name: c"index-name".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'I' as i32,
        },
        GetoptLongOption {
            name: c"compress-level".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'l' as i32,
        },
        GetoptLongOption {
            name: c"reindex".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'r' as i32,
        },
        GetoptLongOption {
            name: c"rebgzip".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'g' as i32,
        },
        GetoptLongOption {
            name: c"size".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b's' as i32,
        },
        GetoptLongOption {
            name: c"threads".as_ptr(),
            has_arg: REQUIRED_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'@' as i32,
        },
        GetoptLongOption {
            name: c"test".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b't' as i32,
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
            val: b'k' as i32,
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
            val: b'o' as i32,
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
            x if x == b'd' as i32 => compress = false,
            x if x == b'c' as i32 => pstdout = true,
            x if x == b'b' as i32 => {
                start = libc::atol(optarg);
                compress = false;
                pstdout = true;
            }
            x if x == b's' as i32 => {
                size = libc::atol(optarg);
                pstdout = true;
            }
            x if x == b'f' as i32 => is_forced += 1,
            x if x == b'i' as i32 => index = true,
            x if x == b'I' as i32 => index_fname = Some(CStr::from_ptr(optarg)),
            x if x == b'l' as i32 => compress_level = libc::atol(optarg) as i32,
            x if x == b'g' as i32 => rebgzip = true,
            x if x == b'r' as i32 => {
                reindex = true;
                compress = false;
            }
            x if x == b'@' as i32 => threads = libc::atoi(optarg),
            x if x == b't' as i32 => {
                test = true;
                compress = false;
                reindex = false;
            }
            x if x == b'k' as i32 => keep = true,
            x if x == b'o' as i32 => write_fname = Some(CStr::from_ptr(optarg)),
            1 => {
                libc::printf(
                    c"bgzip (htslib) %s\nCopyright (C) 2025 Genome Research Ltd.\n".as_ptr(),
                    hts_version(),
                );
                return libc::EXIT_SUCCESS;
            }
            2 => binary = true,
            x if x == b'h' as i32 => {
                return bgzip_c_192_bgzip_main_usage(
                    crate::htslib_rs::c_compat::stdout.cast(),
                    libc::EXIT_SUCCESS,
                );
            }
            x if x == b'?' as i32 => {
                return bgzip_c_192_bgzip_main_usage(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    libc::EXIT_FAILURE,
                );
            }
            _ => {}
        }
    }

    if size >= 0 {
        end = start + size;
    }
    if end >= 0 && end < start {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] Illegal region: [%ld, %ld]\n".as_ptr(),
            start,
            end,
        );
        return 1;
    }

    if (index || reindex) && rebgzip {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] Can't produce a index and rebgzip simultaneously\n".as_ptr(),
        );
        return 1;
    }
    if rebgzip && index_fname.is_none() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] Index file name expected with rebgzip.  See -I option.\n".as_ptr(),
        );
        return 1;
    }
    if (index || reindex) && write_fname.is_none() && index_fname.is_some() && argc - optind > 1 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"[bgzip] Cannot specify index filename with multiple data file on index, reindex.\n"
                .as_ptr(),
        );
        return 1;
    }

    if let Some(output_name) = write_fname {
        if pstdout {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"[bgzip] Cannot write to %s and stdout at the same time.\n".as_ptr(),
                output_name.as_ptr(),
            );
            return 1;
        } else if output_name.to_bytes() == b"-" {
            pstdout = true;
            write_fname = None;
        }
    }

    loop {
        isstdin = if optind >= argc {
            true
        } else {
            CStr::from_ptr(*argv.add(optind as usize)).to_bytes() == b"-"
        };

        if write_fname.is_none() {
            usedstdout |= isstdin || pstdout || test;
        }

        statfilename = None;
        let mut statfilename_owner: Option<CString> = None;
        let mut statfilename_buf_owner: Option<Vec<u8>> = None;

        if compress {
            let mut out_mode = [b'w', 0, 0];
            let mut out_mode_exclusive = [b'w', b'x', 0, 0];

            if !(-1..=9).contains(&compress_level) {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"[bgzip] Invalid compress-level: %d\n".as_ptr(),
                    compress_level,
                );
                return 1;
            }
            if compress_level >= 0 {
                out_mode[1] = (compress_level + b'0' as i32) as u8;
                out_mode_exclusive[2] = (compress_level + b'0' as i32) as u8;
            }
            let f_src = hopen(
                if !isstdin {
                    *argv.add(optind as usize)
                } else {
                    c"-".as_ptr().cast_mut()
                },
                c"r".as_ptr(),
            );
            if f_src.is_null() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"[bgzip] %s: %s\n".as_ptr(),
                    libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                    if isstdin {
                        c"stdin".as_ptr()
                    } else {
                        *argv.add(optind as usize)
                    },
                );
                return 1;
            }

            if let Some(output_name) = write_fname {
                if !exp_out_open {
                    fp = bgzip_open_bgzf(output_name, CStr::from_ptr(out_mode.as_ptr().cast()));
                    if fp.is_none() {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"[bgzip] can't create %s: %s\n".as_ptr(),
                            output_name.as_ptr(),
                            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                        );
                        return 1;
                    } else {
                        exp_out_open = true;
                    }
                }
            } else if argc > optind && !isstdin {
                if pstdout {
                    fp = bgzip_dopen_bgzf(
                        libc::STDOUT_FILENO,
                        CStr::from_ptr(out_mode.as_ptr().cast()),
                    );
                } else {
                    let mut name_bytes = CStr::from_ptr(*argv.add(optind as usize))
                        .to_bytes()
                        .to_vec();
                    name_bytes.extend_from_slice(b".gz");
                    let name_owner = CString::new(name_bytes).unwrap();
                    let name = name_owner.as_ptr().cast_mut();
                    fp = bgzip_open_bgzf(
                        name_owner.as_c_str(),
                        if is_forced != 0 {
                            CStr::from_ptr(out_mode.as_ptr().cast())
                        } else {
                            CStr::from_ptr(out_mode_exclusive.as_ptr().cast())
                        },
                    );
                    if fp.is_none()
                        && *crate::htslib_rs::c_compat::__errno_location() == libc::EEXIST
                    {
                        if bgzip_confirm_overwrite(name_owner.as_c_str()) != 0 {
                            fp = bgzip_open_bgzf(
                                name_owner.as_c_str(),
                                CStr::from_ptr(out_mode.as_ptr().cast()),
                            );
                        } else {
                            ret = 2;
                            hclose_abruptly(f_src);
                            optind += 1;
                            if optind >= argc {
                                break;
                            }
                            continue;
                        }
                    }
                    if fp.is_none() {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"[bgzip] can't create %s: %s\n".as_ptr(),
                            name,
                            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                        );
                        return 1;
                    }
                    statfilename_owner = Some(name_owner);
                    statfilename = NonNull::new(name);
                }
            } else if !pstdout
                && libc::isatty(libc::fileno(crate::htslib_rs::c_compat::stdout.cast())) != 0
            {
                return bgzip_c_192_bgzip_main_usage(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    libc::EXIT_FAILURE,
                );
            } else if index && index_fname.is_none() {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"[bgzip] Index file name expected when writing to stdout\n".as_ptr(),
                );
                return 1;
            } else {
                fp = bgzip_dopen_bgzf(
                    libc::STDOUT_FILENO,
                    CStr::from_ptr(out_mode.as_ptr().cast()),
                );
            }

            if index {
                let fp_ref = fp.as_mut().unwrap().as_mut();
                bgzf_index_build_init(fp_ref);
            }
            if threads > 1 && bgzf_mt(fp.as_mut().unwrap().as_mut(), threads, 256) != 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"[bgzip] threaded BGZF is not yet supported in this translation\n".as_ptr(),
                );
                let _ = bgzip_close_bgzf(&mut fp);
                hclose_abruptly(f_src);
                return 1;
            }

            let mut buffer_storage = vec![0u8; WINDOW_SIZE];
            let buffer = buffer_storage.as_mut_ptr();
            if rebgzip {
                if bgzf_index_load(
                    fp.as_mut().unwrap().as_mut(),
                    index_fname.map_or(ptr::null(), CStr::as_ptr),
                    ptr::null(),
                ) < 0
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Could not load index: %s.%s\n".as_ptr(),
                        if !isstdin {
                            *argv.add(optind as usize)
                        } else {
                            index_fname.map_or(ptr::null(), CStr::as_ptr)
                        },
                        if !isstdin {
                            c"gzi".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }

                loop {
                    c = htslib_hfile_h_247_hread(f_src, buffer.cast(), WINDOW_SIZE) as i32;
                    if c <= 0 {
                        break;
                    }
                    let fp_ref = fp.as_mut().unwrap().as_mut();
                    if bgzf_block_write(fp_ref, buffer.cast(), c as usize) < 0 {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not write %d bytes: Error %d\n".as_ptr(),
                            c,
                            fp_ref.bitfields >> 16,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
            } else {
                let mut fmt: htsFormat = std::mem::zeroed();
                let mut textual = false;
                if !binary
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
                        | HTS_FORMAT_FQI_FORMAT => textual = true,
                        _ => {}
                    }
                }

                if binary || !textual {
                    loop {
                        c = htslib_hfile_h_247_hread(f_src, buffer.cast(), WINDOW_SIZE) as i32;
                        if c <= 0 {
                            break;
                        }
                        let fp_ref = fp.as_mut().unwrap().as_mut();
                        if bgzf_write(fp_ref, buffer.cast(), c as usize) < 0 {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"Could not write %d bytes: Error %d\n".as_ptr(),
                                c,
                                fp_ref.bitfields >> 16,
                            );
                            libc::exit(libc::EXIT_FAILURE);
                        }
                    }
                } else {
                    let mut in_header = true;
                    let mut n: i32 = 0;
                    let mut long_line = false;
                    loop {
                        c = htslib_hfile_h_247_hread(
                            f_src,
                            buffer.add(n as usize).cast(),
                            WINDOW_SIZE - n as usize,
                        ) as i32;
                        if c <= 0 {
                            break;
                        }
                        let c2 = c + n;
                        let mut flush = false;
                        if in_header && (long_line || *buffer == b'@' || *buffer == b'#') {
                            let mut last_start = 0;
                            n = 0;
                            while n < c2 {
                                if *buffer.add(n as usize) != b'\n' {
                                    n += 1;
                                    continue;
                                }
                                n += 1;

                                last_start = n;
                                if n < c2
                                    && !(*buffer.add(n as usize) == b'@'
                                        || *buffer.add(n as usize) == b'#')
                                {
                                    in_header = false;
                                    break;
                                }
                            }
                            if last_start == 0 {
                                n = c2;
                                long_line = true;
                            } else {
                                n = last_start;
                                flush = true;
                                long_line = false;
                            }
                        } else {
                            n += c;
                            loop {
                                n -= 1;
                                if n < 0 || *buffer.add(n as usize) == b'\n' {
                                    break;
                                }
                            }

                            if n >= 0 {
                                flush = true;
                                n += 1;
                            } else {
                                n = c2;
                            }
                        }

                        let fp_ref = fp.as_mut().unwrap().as_mut();
                        let wrote = if flush && fp_ref.block_offset == 0 {
                            bgzf_write_direct_block(fp_ref, buffer.cast(), n as usize)
                        } else {
                            bgzf_write(fp_ref, buffer.cast(), n as usize)
                        };
                        if wrote < 0 {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"Could not write %d bytes: Error %d\n".as_ptr(),
                                n,
                                fp_ref.bitfields >> 16,
                            );
                            libc::exit(libc::EXIT_FAILURE);
                        }
                        if flush && fp_ref.block_offset != 0 && bgzf_flush_try(fp_ref, 65536) < 0 {
                            return -1;
                        }

                        libc::memmove(
                            buffer.cast(),
                            buffer.add(n as usize).cast(),
                            (c2 - n) as usize,
                        );
                        n = c2 - n;
                    }

                    let fp_ref = fp.as_mut().unwrap().as_mut();
                    if bgzf_write(fp_ref, buffer.cast(), n as usize) < 0 {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not write %d bytes: Error %d\n".as_ptr(),
                            n,
                            fp_ref.bitfields >> 16,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
            }
            if index && write_fname.is_none() {
                if let Some(index_fname) = index_fname {
                    if bgzf_index_dump(
                        fp.as_mut().unwrap().as_mut(),
                        index_fname.as_ptr(),
                        ptr::null(),
                    ) < 0
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not write index to '%s'\n".as_ptr(),
                            index_fname.as_ptr(),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else if !isstdin {
                    if bgzf_index_dump(
                        fp.as_mut().unwrap().as_mut(),
                        *argv.add(optind as usize),
                        c".gz.gzi".as_ptr(),
                    ) < 0
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not write index to '%s.gz.gzi'\n".as_ptr(),
                            *argv.add(optind as usize),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Can not write index for stdin data without index filename, use -I option to set index file.\n".as_ptr(),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            let close_result = if write_fname.is_none() {
                Some(bgzip_close_bgzf(&mut fp))
            } else {
                None
            };
            if let Some((close_ret, close_err)) = close_result {
                if close_ret < 0 {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Output close failed: Error %d\n".as_ptr(),
                        close_err,
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            if hclose(f_src) < 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Input close failed\n".as_ptr(),
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if let Some(statfilename) = statfilename {
                let input_name = CStr::from_ptr(*argv.add(optind as usize));
                let output_name = CStr::from_ptr(statfilename.as_ptr());
                if bgzip_getfilespec(input_name, &mut filestat) == 0 {
                    if bgzip_setfilespec(output_name, &filestat) < 0 {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"[bgzip] Failed to set file specification.\n".as_ptr(),
                        );
                    }
                } else {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] Failed to get file specification.\n".as_ptr(),
                    );
                }
                drop(statfilename_owner.take());
                drop(statfilename_buf_owner.take());
            }

            if argc > optind && !pstdout && !keep && !isstdin && write_fname.is_none() {
                libc::unlink(*argv.add(optind as usize));
            }
        } else if reindex {
            if argc > optind && !isstdin {
                fp = bgzip_open_bgzf(CStr::from_ptr(*argv.add(optind as usize)), c"r");
                if fp.is_none() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] Could not open file: %s\n".as_ptr(),
                        *argv.add(optind as usize),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            } else {
                if index_fname.is_none() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] Index file name expected when reading from stdin\n".as_ptr(),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
                fp = bgzip_open_bgzf(c"-", c"r");
                if fp.is_none() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] Could not read from stdin: %s\n".as_ptr(),
                        libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            let mut buffer_storage = vec![0u8; WINDOW_SIZE];
            let buffer = buffer_storage.as_mut_ptr();
            bgzf_index_build_init(fp.as_mut().unwrap().as_mut());
            let mut read_ret: isize;
            loop {
                read_ret = bgzf_read(fp.as_mut().unwrap().as_mut(), buffer.cast(), WINDOW_SIZE);
                if read_ret <= 0 {
                    break;
                }
            }
            if read_ret < 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Is the file gzipped or bgzipped? The latter is required for indexing.\n"
                        .as_ptr(),
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if let Some(index_fname) = index_fname {
                if bgzf_index_dump(
                    fp.as_mut().unwrap().as_mut(),
                    index_fname.as_ptr(),
                    ptr::null(),
                ) < 0
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Could not write index to '%s'\n".as_ptr(),
                        index_fname.as_ptr(),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            } else if !isstdin {
                if bgzf_index_dump(
                    fp.as_mut().unwrap().as_mut(),
                    *argv.add(optind as usize),
                    c".gzi".as_ptr(),
                ) < 0
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Could not write index to '%s.gzi'\n".as_ptr(),
                        *argv.add(optind as usize),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            } else {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Can not write index for stdin data without index filename, use -I option to set index file.\n".as_ptr(),
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            let (close_ret, close_err) = bgzip_close_bgzf(&mut fp);
            if close_ret < 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Close failed: Error %d\n".as_ptr(),
                    close_err,
                );
                libc::exit(libc::EXIT_FAILURE);
            }
        } else {
            let mut is_forced_tmp = is_forced;

            if argc > optind && !isstdin {
                fp = bgzip_open_bgzf(CStr::from_ptr(*argv.add(optind as usize)), c"r");
                if fp.is_none() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] Could not open %s: %s\n".as_ptr(),
                        *argv.add(optind as usize),
                        libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                    );
                    return 1;
                }
                if bgzf_compression(fp.as_mut().unwrap().as_mut())
                    == HTS_COMPRESSION_NO_COMPRESSION as i32
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] %s: not a compressed file -- ignored\n".as_ptr(),
                        *argv.add(optind as usize),
                    );
                    let _ = bgzip_close_bgzf(&mut fp);
                    return 1;
                }

                if pstdout || test {
                    f_dst = libc::fileno(crate::htslib_rs::c_compat::stdout.cast());
                } else {
                    let wrflags = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
                    let mut name_owner: Vec<u8> = CStr::from_ptr(*argv.add(optind as usize))
                        .to_bytes_with_nul()
                        .to_vec();
                    let name: *mut i8 = name_owner.as_mut_ptr().cast();

                    let check = bgzip_check_name_and_extension(
                        name_owner.as_mut_slice(),
                        &mut is_forced_tmp,
                    );
                    if check != 0 {
                        let _ = bgzip_close_bgzf(&mut fp);

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

                    if !exp_out_open {
                        if write_fname.is_some() {
                            is_forced_tmp = 1;
                            exp_out_open = true;
                        }

                        let output_name = write_fname.map_or(name.cast_const(), CStr::as_ptr);
                        f_dst = libc::open(
                            output_name,
                            if is_forced_tmp != 0 {
                                wrflags
                            } else {
                                wrflags | libc::O_EXCL
                            },
                            0o666,
                        );

                        if f_dst < 0
                            && *crate::htslib_rs::c_compat::__errno_location() == libc::EEXIST
                        {
                            if bgzip_confirm_overwrite(CStr::from_ptr(output_name)) != 0 {
                                f_dst = libc::open(output_name, wrflags, 0o666);
                            } else {
                                ret = 2;
                                let _ = bgzip_close_bgzf(&mut fp);
                                optind += 1;
                                if optind >= argc {
                                    break;
                                }
                                continue;
                            }
                        }
                        if f_dst < 0 {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"[bgzip] can't create %s: %s\n".as_ptr(),
                                output_name,
                                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                            );
                            return 1;
                        }
                    }

                    statfilename_buf_owner = Some(name_owner);
                    statfilename = NonNull::new(name);
                }
            } else if !pstdout
                && libc::isatty(libc::fileno(crate::htslib_rs::c_compat::stdin.cast())) != 0
            {
                return bgzip_c_192_bgzip_main_usage(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    libc::EXIT_FAILURE,
                );
            } else {
                f_dst = libc::fileno(crate::htslib_rs::c_compat::stdout.cast());
                fp = bgzip_open_bgzf(c"-", c"r");
                if fp.is_none() {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] Could not read from stdin: %s\n".as_ptr(),
                        libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                    );
                    return 1;
                }
                if bgzf_compression(fp.as_mut().unwrap().as_mut())
                    == HTS_COMPRESSION_NO_COMPRESSION as i32
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"[bgzip] stdin is not compressed -- ignored\n".as_ptr(),
                    );
                    let _ = bgzip_close_bgzf(&mut fp);
                    return 1;
                }

                if let Some(output_name) = write_fname {
                    if !exp_out_open {
                        exp_out_open = true;

                        f_dst = libc::open(
                            output_name.as_ptr(),
                            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                            0o666,
                        );

                        if f_dst < 0 {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"[bgzip] can't create %s: %s\n".as_ptr(),
                                output_name.as_ptr(),
                                libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
                            );
                            return 1;
                        }
                    }
                } else {
                    f_dst = libc::fileno(crate::htslib_rs::c_compat::stdout.cast());
                }
            }

            let mut buffer_storage = vec![0u8; WINDOW_SIZE];
            let buffer = buffer_storage.as_mut_ptr();
            if start > 0 {
                if let Some(index_fname) = index_fname {
                    if bgzf_index_load(
                        fp.as_mut().unwrap().as_mut(),
                        index_fname.as_ptr(),
                        ptr::null(),
                    ) < 0
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not load index: %s\n".as_ptr(),
                            index_fname.as_ptr(),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else {
                    if optind >= argc || isstdin {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"The -b option requires -I when reading from stdin (and stdin must be seekable)\n".as_ptr(),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    if bgzf_index_load(
                        fp.as_mut().unwrap().as_mut(),
                        *argv.add(optind as usize),
                        c".gzi".as_ptr(),
                    ) < 0
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not load index: %s.gzi\n".as_ptr(),
                            *argv.add(optind as usize),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
                if bgzf_useek(fp.as_mut().unwrap().as_mut(), start as i64, libc::SEEK_SET) < 0 {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Could not seek to %ld-th (uncompressd) byte\n".as_ptr(),
                        start,
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            if threads > 1 && bgzf_mt(fp.as_mut().unwrap().as_mut(), threads, 256) != 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"[bgzip] threaded BGZF is not yet supported in this translation\n".as_ptr(),
                );
                let _ = bgzip_close_bgzf(&mut fp);
                return 1;
            }

            let start_reg = start;
            let end_reg = end;
            if end < 0 && start == 0 {
                loop {
                    let mut block_data: *const libc::c_void = ptr::null();
                    let fp_ref = fp.as_mut().unwrap().as_mut();
                    c = bgzf_read_block_data(fp_ref, &mut block_data) as i32;
                    if c == 0 {
                        break;
                    }
                    if c < 0 {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Error %d in block starting at offset %ld(%lX)\n".as_ptr(),
                            fp_ref.bitfields >> 16,
                            fp_ref.block_address,
                            fp_ref.block_address,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    if !test
                        && crate::htslib_rs::c_compat::fd_write(f_dst, block_data, c as usize)
                            != c as libc::ssize_t
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not write %d bytes\n".as_ptr(),
                            c,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                }
            } else {
                loop {
                    c = bgzf_read(
                        fp.as_mut().unwrap().as_mut(),
                        buffer.cast(),
                        if end < 0 || end - start > WINDOW_SIZE as libc::c_long {
                            WINDOW_SIZE
                        } else {
                            (end - start) as usize
                        },
                    ) as i32;
                    if c == 0 {
                        break;
                    }
                    if c < 0 {
                        let fp_ref = fp.as_mut().unwrap().as_mut();
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Error %d in block starting at offset %ld(%lX)\n".as_ptr(),
                            fp_ref.bitfields >> 16,
                            fp_ref.block_address,
                            fp_ref.block_address,
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                    start += c as libc::c_long;
                    if !test
                        && crate::htslib_rs::c_compat::fd_write(f_dst, buffer.cast(), c as usize)
                            != c as libc::ssize_t
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
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
            let (close_ret, close_err) = bgzip_close_bgzf(&mut fp);
            if close_ret < 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Close failed: Error %d\n".as_ptr(),
                    close_err,
                );
                libc::exit(libc::EXIT_FAILURE);
            }

            if let Some(statfilename) = statfilename {
                if write_fname.is_none() {
                    let input_name = CStr::from_ptr(*argv.add(optind as usize));
                    let output_name = CStr::from_ptr(statfilename.as_ptr());
                    if bgzip_getfilespec(input_name, &mut filestat) == 0 {
                        if bgzip_setfilespec(output_name, &filestat) < 0 {
                            libc::fprintf(
                                crate::htslib_rs::c_compat::stderr.cast(),
                                c"[bgzip] Failed to set file specification.\n".as_ptr(),
                            );
                        }
                    } else {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"[bgzip] Failed to get file specification.\n".as_ptr(),
                        );
                    }
                }

                drop(statfilename_owner.take());
                drop(statfilename_buf_owner.take());
            }

            if argc > optind && !pstdout && !test && !keep && !isstdin && write_fname.is_none() {
                libc::unlink(*argv.add(optind as usize));
            }
            if !isstdin && !pstdout && !test && write_fname.is_none() {
                libc::close(f_dst);
            }
        }

        optind += 1;
        if optind >= argc {
            break;
        }
    }

    if usedstdout && !reindex {
        if libc::fclose(crate::htslib_rs::c_compat::stdout.cast()) != 0
            && *crate::htslib_rs::c_compat::__errno_location() != libc::EBADF
        {
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"[bgzip] Failed to close stdout, errno %d".as_ptr(),
                *crate::htslib_rs::c_compat::__errno_location(),
            );
            ret = 1;
        }
    } else if let Some(write_fname) = write_fname {
        if compress {
            if index {
                if let Some(index_fname) = index_fname {
                    if bgzf_index_dump(
                        fp.as_mut().unwrap().as_mut(),
                        index_fname.as_ptr(),
                        ptr::null(),
                    ) < 0
                    {
                        libc::fprintf(
                            crate::htslib_rs::c_compat::stderr.cast(),
                            c"Could not write index to '%s'\n".as_ptr(),
                            index_fname.as_ptr(),
                        );
                        libc::exit(libc::EXIT_FAILURE);
                    }
                } else if bgzf_index_dump(
                    fp.as_mut().unwrap().as_mut(),
                    write_fname.as_ptr(),
                    c".gzi".as_ptr(),
                ) < 0
                {
                    libc::fprintf(
                        crate::htslib_rs::c_compat::stderr.cast(),
                        c"Could not write index to '%s.gzi'\n".as_ptr(),
                        write_fname.as_ptr(),
                    );
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            let (close_ret, close_err) = bgzip_close_bgzf(&mut fp);
            if close_ret < 0 {
                libc::fprintf(
                    crate::htslib_rs::c_compat::stderr.cast(),
                    c"Output close failed: Error %d\n".as_ptr(),
                    close_err,
                );
                libc::exit(libc::EXIT_FAILURE);
            }
        } else {
            libc::close(f_dst);
        }
    }

    ret
}
