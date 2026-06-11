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
use std::ffi::CStr;
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
fn bgzip_c_59_ask_yn() -> i32 {
    use std::io::BufRead;
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).unwrap_or(0) == 0 {
        return 0;
    }
    let first = line.as_bytes().first().copied().unwrap_or(0);
    (first == b'Y' || first == b'y') as i32
}

// original: confirm_overwrite (htslib/bgzip.c:68)
pub unsafe fn bgzip_c_68_confirm_overwrite(fn_: &[u8]) -> i32 {
    bgzip_confirm_overwrite(fn_)
}

unsafe fn bgzip_confirm_overwrite(fn_: &[u8]) -> i32 {
    let save_errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    let mut ret = 0;

    if libc::isatty(libc::STDIN_FILENO) != 0 {
        eprint!(
            "[bgzip] {} already exists; do you wish to overwrite (y or n)? ",
            String::from_utf8_lossy(fn_)
        );
        if bgzip_c_59_ask_yn() != 0 {
            ret = 1;
        }
    }

    libc::__errno_location().write(save_errno);
    ret
}

// original: known_extension (htslib/bgzip.c:82)
pub fn bgzip_c_82_known_extension(ext: &[u8]) -> i32 {
    bgzip_known_extension(ext)
}

fn bgzip_known_extension(ext: &[u8]) -> i32 {
    (ext.eq_ignore_ascii_case(b"gz")
        || ext.eq_ignore_ascii_case(b"bgz")
        || ext.eq_ignore_ascii_case(b"bgzf")) as i32
}

// original: confirm_filename (htslib/bgzip.c:95)
pub unsafe fn bgzip_c_95_confirm_filename(is_forced: &mut i32, name: &[u8], ext: &[u8]) -> i32 {
    bgzip_confirm_filename(is_forced, name, ext)
}

unsafe fn bgzip_confirm_filename(is_forced: &mut i32, name: &[u8], ext: &[u8]) -> i32 {
    if *is_forced != 0 {
        *is_forced -= 1;
        return 1;
    }

    if libc::isatty(libc::STDIN_FILENO) == 0 {
        return 0;
    }

    eprint!(
        "[bgzip] .{} is not a known extension; do you wish to decompress to {} (y or n)? ",
        String::from_utf8_lossy(ext),
        String::from_utf8_lossy(name)
    );
    bgzip_c_59_ask_yn()
}

// original: getfilespec (htslib/bgzip.c:114)
pub unsafe fn bgzip_c_114_getfilespec(path: &[u8], status: &mut libc::stat) -> i32 {
    bgzip_getfilespec(path, status)
}

unsafe fn bgzip_getfilespec(path: &[u8], status: &mut libc::stat) -> i32 {
    if path == b"-" {
        return 0;
    }
    let mut path_z = path.to_vec();
    path_z.push(0);
    if libc::stat(path_z.as_ptr().cast(), status) < 0 {
        return -1;
    }
    0
}

// original: setfilespec (htslib/bgzip.c:134)
pub unsafe fn bgzip_c_134_setfilespec(path: &[u8], status: &libc::stat) -> i32 {
    bgzip_setfilespec(path, status)
}

unsafe fn bgzip_setfilespec(path: &[u8], status: &libc::stat) -> i32 {
    if path == b"-" {
        return 0;
    }

    let tval = [
        libc::timeval {
            tv_sec: status.st_atime as _,
            tv_usec: 0,
        },
        libc::timeval {
            tv_sec: status.st_mtime as _,
            tv_usec: 0,
        },
    ];
    let mut path_z = path.to_vec();
    path_z.push(0);
    if libc::utimes(path_z.as_ptr().cast(), tval.as_ptr()) < 0 {
        eprintln!("[bgzip] Failed to set file specifications.");
        return -1;
    }
    0
}

// original: check_name_and_extension (htslib/bgzip.c:168)
pub unsafe fn bgzip_c_168_check_name_and_extension(name: &mut [u8], forced: &mut i32) -> i32 {
    bgzip_check_name_and_extension(name, forced)
}

unsafe fn bgzip_check_name_and_extension(name: &mut [u8], forced: &mut i32) -> i32 {
    // `name` is the byte buffer with a trailing NUL terminator.
    let nul = name.len().saturating_sub(1);
    let stem = &name[..nul];
    let pos = stem
        .iter()
        .rposition(|&b| b == b'.' || b == b'/')
        .unwrap_or(0);

    if pos == 0 || name[pos] != b'.' {
        eprintln!(
            "[bgzip] can't find an extension in {} -- please rename",
            String::from_utf8_lossy(stem)
        );
        return 1;
    }

    // Split into the decompressed-target name (before the dot) and extension.
    let name_str = name[..pos].to_vec();
    let ext_str = name[pos + 1..nul].to_vec();

    if !(bgzip_known_extension(&ext_str) != 0
        || bgzip_confirm_filename(forced, &name_str, &ext_str) != 0)
    {
        eprintln!(
            "[bgzip] unknown extension .{} -- declining to decompress to {}",
            String::from_utf8_lossy(&ext_str),
            String::from_utf8_lossy(&name_str)
        );
        return 2;
    }

    // Truncate the target name in-place at the dot, mirroring the original
    // C in-buffer split that writes a NUL where the extension dot was.
    name[pos] = 0;

    0
}

unsafe fn bgzip_open_bgzf(path: &CStr, mode: &CStr) -> Option<NonNull<BGZF>> {
    NonNull::new(bgzf_open(path.as_ptr().cast(), mode.as_ptr().cast()))
}

unsafe fn bgzip_dopen_bgzf(fd: i32, mode: &CStr) -> Option<NonNull<BGZF>> {
    NonNull::new(bgzf_dopen(fd, mode.as_ptr().cast()))
}

unsafe fn bgzip_close_bgzf(fp: &mut Option<NonNull<BGZF>>) -> (i32, u32) {
    let Some(fp) = fp.take() else {
        return (-1, 0);
    };
    let err = fp.as_ref().bitfields >> 16;
    (bgzf_close(fp.as_ptr()), err)
}

// original: bgzip_main_usage (htslib/bgzip.c:192)
pub unsafe fn bgzip_c_192_bgzip_main_usage(to_stderr: bool, status: i32) -> i32 {
    let version = CStr::from_ptr(hts_version()).to_string_lossy().into_owned();
    let text = format!(
        "\n\
         Version: {version}\n\
         Usage:   bgzip [OPTIONS] [FILE] ...\n\
         Options:\n\
            -b, --offset INT           decompress at virtual file pointer (0-based uncompressed offset)\n\
            -c, --stdout               write on standard output, keep original files unchanged\n\
            -d, --decompress           decompress\n\
            -f, --force                overwrite files without asking\n\
            -g, --rebgzip              use an index file to bgzip a file\n\
            -h, --help                 give this help\n\
            -i, --index                compress and create BGZF index\n\
            -I, --index-name FILE      name of BGZF index file [file.gz.gzi]\n\
            -k, --keep                 don't delete input files during operation\n\
            -l, --compress-level INT   Compression level to use when compressing; 0 to 9, or -1 for default [-1]\n\
            -o, --output FILE          write to file, keep original files unchanged\n\
            -r, --reindex              (re)index compressed file\n\
            -s, --size INT             decompress INT bytes (uncompressed size)\n\
            -t, --test                 test integrity of compressed file\n    \
                --binary               Don't align blocks with text lines\n\
            -@, --threads INT          number of compression threads to use [1]\n",
    );
    if to_stderr {
        eprint!("{text}");
    } else {
        print!("{text}");
    }
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
    let mut start: i64 = 0;
    let mut end: i64 = -1;
    let mut size: i64 = -1;
    let mut filestat: libc::stat = std::mem::zeroed();
    // Owns the output filename bytes (NUL-terminated) when one was generated.
    let mut statfilename: Option<Vec<u8>>;
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
                start = CStr::from_ptr(optarg)
                    .to_string_lossy()
                    .trim()
                    .parse()
                    .unwrap_or(0);
                compress = false;
                pstdout = true;
            }
            x if x == b's' as i32 => {
                size = CStr::from_ptr(optarg)
                    .to_string_lossy()
                    .trim()
                    .parse()
                    .unwrap_or(0);
                pstdout = true;
            }
            x if x == b'f' as i32 => is_forced += 1,
            x if x == b'i' as i32 => index = true,
            x if x == b'I' as i32 => index_fname = Some(CStr::from_ptr(optarg)),
            x if x == b'l' as i32 => {
                compress_level = CStr::from_ptr(optarg)
                    .to_string_lossy()
                    .trim()
                    .parse()
                    .unwrap_or(0);
            }
            x if x == b'g' as i32 => rebgzip = true,
            x if x == b'r' as i32 => {
                reindex = true;
                compress = false;
            }
            x if x == b'@' as i32 => {
                threads = CStr::from_ptr(optarg)
                    .to_string_lossy()
                    .trim()
                    .parse()
                    .unwrap_or(0);
            }
            x if x == b't' as i32 => {
                test = true;
                compress = false;
                reindex = false;
            }
            x if x == b'k' as i32 => keep = true,
            x if x == b'o' as i32 => write_fname = Some(CStr::from_ptr(optarg)),
            1 => {
                println!(
                    "bgzip (htslib) {}\nCopyright (C) 2025 Genome Research Ltd.",
                    CStr::from_ptr(hts_version()).to_string_lossy()
                );
                return 0;
            }
            2 => binary = true,
            x if x == b'h' as i32 => {
                return bgzip_c_192_bgzip_main_usage(false, 0);
            }
            x if x == b'?' as i32 => {
                return bgzip_c_192_bgzip_main_usage(true, 1);
            }
            _ => {}
        }
    }

    if size >= 0 {
        end = start + size;
    }
    if end >= 0 && end < start {
        eprintln!("[bgzip] Illegal region: [{start}, {end}]");
        return 1;
    }

    if (index || reindex) && rebgzip {
        eprintln!("[bgzip] Can't produce a index and rebgzip simultaneously");
        return 1;
    }
    if rebgzip && index_fname.is_none() {
        eprintln!("[bgzip] Index file name expected with rebgzip.  See -I option.");
        return 1;
    }
    if (index || reindex) && write_fname.is_none() && index_fname.is_some() && argc - optind > 1 {
        eprintln!(
            "[bgzip] Cannot specify index filename with multiple data file on index, reindex."
        );
        return 1;
    }

    if let Some(output_name) = write_fname {
        if pstdout {
            eprintln!(
                "[bgzip] Cannot write to {} and stdout at the same time.",
                output_name.to_string_lossy()
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

        if compress {
            let mut out_mode = [b'w', 0, 0];
            let mut out_mode_exclusive = [b'w', b'x', 0, 0];

            if !(-1..=9).contains(&compress_level) {
                eprintln!("[bgzip] Invalid compress-level: {compress_level}");
                return 1;
            }
            if compress_level >= 0 {
                out_mode[1] = (compress_level + b'0' as i32) as u8;
                out_mode_exclusive[2] = (compress_level + b'0' as i32) as u8;
            }
            let f_src = hopen(
                if !isstdin {
                    (*argv.add(optind as usize)).cast::<u8>().cast_const()
                } else {
                    c"-".as_ptr().cast::<u8>()
                },
                c"r".as_ptr().cast::<u8>(),
            );
            if f_src.is_null() {
                let src_name = if isstdin {
                    "stdin".to_string()
                } else {
                    CStr::from_ptr(*argv.add(optind as usize))
                        .to_string_lossy()
                        .into_owned()
                };
                eprintln!(
                    "[bgzip] {}: {}",
                    std::io::Error::last_os_error(),
                    src_name
                );
                return 1;
            }

            if let Some(output_name) = write_fname {
                if !exp_out_open {
                    fp = bgzip_open_bgzf(output_name, CStr::from_ptr(out_mode.as_ptr().cast()));
                    if fp.is_none() {
                        eprintln!(
                            "[bgzip] can't create {}: {}",
                            output_name.to_string_lossy(),
                            std::io::Error::last_os_error()
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
                    // Build "<input>.gz" as a NUL-terminated byte buffer.
                    let mut name_buf = CStr::from_ptr(*argv.add(optind as usize))
                        .to_bytes()
                        .to_vec();
                    name_buf.extend_from_slice(b".gz");
                    let name_display = String::from_utf8_lossy(&name_buf).into_owned();
                    name_buf.push(0);
                    let name_cstr = CStr::from_bytes_with_nul(&name_buf).unwrap();
                    fp = bgzip_open_bgzf(
                        name_cstr,
                        if is_forced != 0 {
                            CStr::from_ptr(out_mode.as_ptr().cast())
                        } else {
                            CStr::from_ptr(out_mode_exclusive.as_ptr().cast())
                        },
                    );
                    if fp.is_none()
                        && std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST)
                    {
                        if bgzip_confirm_overwrite(&name_buf[..name_buf.len() - 1]) != 0 {
                            fp = bgzip_open_bgzf(
                                name_cstr,
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
                        eprintln!(
                            "[bgzip] can't create {}: {}",
                            name_display,
                            std::io::Error::last_os_error()
                        );
                        return 1;
                    }
                    statfilename = Some(name_buf);
                }
            } else if !pstdout
                && libc::isatty(libc::STDOUT_FILENO) != 0
            {
                return bgzip_c_192_bgzip_main_usage(true, 1);
            } else if index && index_fname.is_none() {
                eprintln!("[bgzip] Index file name expected when writing to stdout");
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
                eprintln!("[bgzip] threaded BGZF is not yet supported in this translation");
                let _ = bgzip_close_bgzf(&mut fp);
                hclose_abruptly(f_src);
                return 1;
            }

            let mut buffer_storage = vec![0u8; WINDOW_SIZE];
            let buffer = buffer_storage.as_mut_ptr();
            if rebgzip {
                if bgzf_index_load(
                    fp.as_mut().unwrap().as_mut(),
                    index_fname.map_or(ptr::null(), |s| s.as_ptr().cast::<u8>()),
                    ptr::null(),
                ) < 0
                {
                    let (load_name, load_ext) = if !isstdin {
                        (
                            CStr::from_ptr(*argv.add(optind as usize))
                                .to_string_lossy()
                                .into_owned(),
                            "gzi",
                        )
                    } else {
                        (
                            index_fname.map_or(String::new(), |s| {
                                s.to_string_lossy().into_owned()
                            }),
                            "",
                        )
                    };
                    eprintln!("Could not load index: {load_name}.{load_ext}");
                    std::process::exit(1);
                }

                loop {
                    c = htslib_hfile_h_247_hread(f_src, buffer.cast(), WINDOW_SIZE) as i32;
                    if c <= 0 {
                        break;
                    }
                    let fp_ref = fp.as_mut().unwrap().as_mut();
                    if bgzf_block_write(fp_ref, buffer.cast(), c as usize) < 0 {
                        eprintln!(
                            "Could not write {} bytes: Error {}",
                            c,
                            fp_ref.bitfields >> 16
                        );
                        std::process::exit(1);
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
                            eprintln!(
                                "Could not write {} bytes: Error {}",
                                c,
                                fp_ref.bitfields >> 16
                            );
                            std::process::exit(1);
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
                            eprintln!(
                                "Could not write {} bytes: Error {}",
                                n,
                                fp_ref.bitfields >> 16
                            );
                            std::process::exit(1);
                        }
                        if flush && fp_ref.block_offset != 0 && bgzf_flush_try(fp_ref, 65536) < 0 {
                            return -1;
                        }

                        // Shift the unprocessed tail [n..c2) to the start of the buffer.
                        buffer_storage.copy_within(n as usize..c2 as usize, 0);
                        n = c2 - n;
                    }

                    let fp_ref = fp.as_mut().unwrap().as_mut();
                    if bgzf_write(fp_ref, buffer.cast(), n as usize) < 0 {
                        eprintln!(
                            "Could not write {} bytes: Error {}",
                            n,
                            fp_ref.bitfields >> 16
                        );
                        std::process::exit(1);
                    }
                }
            }
            if index && write_fname.is_none() {
                if let Some(index_fname) = index_fname {
                    if bgzf_index_dump(
                        fp.as_mut().unwrap().as_mut(),
                        index_fname.as_ptr().cast::<u8>(),
                        ptr::null(),
                    ) < 0
                    {
                        eprintln!(
                            "Could not write index to '{}'",
                            index_fname.to_string_lossy()
                        );
                        std::process::exit(1);
                    }
                } else if !isstdin {
                    if bgzf_index_dump(
                        fp.as_mut().unwrap().as_mut(),
                        (*argv.add(optind as usize)).cast::<u8>().cast_const(),
                        c".gz.gzi".as_ptr().cast::<u8>(),
                    ) < 0
                    {
                        eprintln!(
                            "Could not write index to '{}.gz.gzi'",
                            CStr::from_ptr(*argv.add(optind as usize)).to_string_lossy()
                        );
                        std::process::exit(1);
                    }
                } else {
                    eprintln!(
                        "Can not write index for stdin data without index filename, use -I option to set index file."
                    );
                    std::process::exit(1);
                }
            }

            let close_result = if write_fname.is_none() {
                Some(bgzip_close_bgzf(&mut fp))
            } else {
                None
            };
            if let Some((close_ret, close_err)) = close_result {
                if close_ret < 0 {
                    eprintln!("Output close failed: Error {close_err}");
                    std::process::exit(1);
                }
            }

            if hclose(f_src) < 0 {
                eprintln!("Input close failed");
                std::process::exit(1);
            }

            if let Some(statfilename) = &statfilename {
                let input_name = CStr::from_ptr(*argv.add(optind as usize))
                    .to_bytes()
                    .to_vec();
                // `statfilename` is NUL-terminated; strip the terminator.
                let output_name = &statfilename[..statfilename.len() - 1];
                if bgzip_getfilespec(&input_name, &mut filestat) == 0 {
                    if bgzip_setfilespec(output_name, &filestat) < 0 {
                        eprintln!("[bgzip] Failed to set file specification.");
                    }
                } else {
                    eprintln!("[bgzip] Failed to get file specification.");
                }
            }

            if argc > optind && !pstdout && !keep && !isstdin && write_fname.is_none() {
                libc::unlink(*argv.add(optind as usize));
            }
        } else if reindex {
            if argc > optind && !isstdin {
                fp = bgzip_open_bgzf(CStr::from_ptr(*argv.add(optind as usize)), c"r");
                if fp.is_none() {
                    eprintln!(
                        "[bgzip] Could not open file: {}",
                        CStr::from_ptr(*argv.add(optind as usize)).to_string_lossy()
                    );
                    std::process::exit(1);
                }
            } else {
                if index_fname.is_none() {
                    eprintln!("[bgzip] Index file name expected when reading from stdin");
                    std::process::exit(1);
                }
                fp = bgzip_open_bgzf(c"-", c"r");
                if fp.is_none() {
                    eprintln!(
                        "[bgzip] Could not read from stdin: {}",
                        std::io::Error::last_os_error()
                    );
                    std::process::exit(1);
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
                eprintln!("Is the file gzipped or bgzipped? The latter is required for indexing.");
                std::process::exit(1);
            }

            if let Some(index_fname) = index_fname {
                if bgzf_index_dump(
                    fp.as_mut().unwrap().as_mut(),
                    index_fname.as_ptr().cast::<u8>(),
                    ptr::null(),
                ) < 0
                {
                    eprintln!(
                        "Could not write index to '{}'",
                        index_fname.to_string_lossy()
                    );
                    std::process::exit(1);
                }
            } else if !isstdin {
                if bgzf_index_dump(
                    fp.as_mut().unwrap().as_mut(),
                    (*argv.add(optind as usize)).cast::<u8>().cast_const(),
                    c".gzi".as_ptr().cast::<u8>(),
                ) < 0
                {
                    eprintln!(
                        "Could not write index to '{}.gzi'",
                        CStr::from_ptr(*argv.add(optind as usize)).to_string_lossy()
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!(
                    "Can not write index for stdin data without index filename, use -I option to set index file."
                );
                std::process::exit(1);
            }

            let (close_ret, close_err) = bgzip_close_bgzf(&mut fp);
            if close_ret < 0 {
                eprintln!("Close failed: Error {close_err}");
                std::process::exit(1);
            }
        } else {
            let mut is_forced_tmp = is_forced;

            if argc > optind && !isstdin {
                fp = bgzip_open_bgzf(CStr::from_ptr(*argv.add(optind as usize)), c"r");
                if fp.is_none() {
                    eprintln!(
                        "[bgzip] Could not open {}: {}",
                        CStr::from_ptr(*argv.add(optind as usize)).to_string_lossy(),
                        std::io::Error::last_os_error()
                    );
                    return 1;
                }
                if bgzf_compression(fp.as_mut().unwrap().as_mut())
                    == HTS_COMPRESSION_NO_COMPRESSION as i32
                {
                    eprintln!(
                        "[bgzip] {}: not a compressed file -- ignored",
                        CStr::from_ptr(*argv.add(optind as usize)).to_string_lossy()
                    );
                    let _ = bgzip_close_bgzf(&mut fp);
                    return 1;
                }

                if pstdout || test {
                    f_dst = libc::STDOUT_FILENO;
                } else {
                    let wrflags = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
                    // NUL-terminated buffer holding the input filename; gets
                    // truncated in place at the extension dot below.
                    let mut name_buf: Vec<u8> = CStr::from_ptr(*argv.add(optind as usize))
                        .to_bytes_with_nul()
                        .to_vec();

                    let check =
                        bgzip_check_name_and_extension(name_buf.as_mut_slice(), &mut is_forced_tmp);
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

                    // Trim the buffer to the truncated name's NUL terminator.
                    let name_nul = name_buf.iter().position(|&b| b == 0).unwrap_or(name_buf.len());
                    name_buf.truncate(name_nul + 1);

                    if !exp_out_open {
                        if write_fname.is_some() {
                            is_forced_tmp = 1;
                            exp_out_open = true;
                        }

                        // NUL-terminated path for the open() syscall.
                        let output_name: Vec<u8> = match write_fname {
                            Some(w) => w.to_bytes_with_nul().to_vec(),
                            None => name_buf.clone(),
                        };
                        let output_display =
                            String::from_utf8_lossy(&output_name[..output_name.len() - 1])
                                .into_owned();
                        f_dst = libc::open(
                            output_name.as_ptr().cast(),
                            if is_forced_tmp != 0 {
                                wrflags
                            } else {
                                wrflags | libc::O_EXCL
                            },
                            0o666,
                        );

                        if f_dst < 0
                            && std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST)
                        {
                            if bgzip_confirm_overwrite(&output_name[..output_name.len() - 1]) != 0 {
                                f_dst = libc::open(output_name.as_ptr().cast(), wrflags, 0o666);
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
                            eprintln!(
                                "[bgzip] can't create {}: {}",
                                output_display,
                                std::io::Error::last_os_error()
                            );
                            return 1;
                        }
                    }

                    statfilename = Some(name_buf);
                }
            } else if !pstdout
                && libc::isatty(libc::STDIN_FILENO) != 0
            {
                return bgzip_c_192_bgzip_main_usage(true, 1);
            } else {
                f_dst = libc::STDOUT_FILENO;
                fp = bgzip_open_bgzf(c"-", c"r");
                if fp.is_none() {
                    eprintln!(
                        "[bgzip] Could not read from stdin: {}",
                        std::io::Error::last_os_error()
                    );
                    return 1;
                }
                if bgzf_compression(fp.as_mut().unwrap().as_mut())
                    == HTS_COMPRESSION_NO_COMPRESSION as i32
                {
                    eprintln!("[bgzip] stdin is not compressed -- ignored");
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
                            eprintln!(
                                "[bgzip] can't create {}: {}",
                                output_name.to_string_lossy(),
                                std::io::Error::last_os_error()
                            );
                            return 1;
                        }
                    }
                } else {
                    f_dst = libc::STDOUT_FILENO;
                }
            }

            let mut buffer_storage = vec![0u8; WINDOW_SIZE];
            let buffer = buffer_storage.as_mut_ptr();
            if start > 0 {
                if let Some(index_fname) = index_fname {
                    if bgzf_index_load(
                        fp.as_mut().unwrap().as_mut(),
                        index_fname.as_ptr().cast::<u8>(),
                        ptr::null(),
                    ) < 0
                    {
                        eprintln!("Could not load index: {}", index_fname.to_string_lossy());
                        std::process::exit(1);
                    }
                } else {
                    if optind >= argc || isstdin {
                        eprintln!(
                            "The -b option requires -I when reading from stdin (and stdin must be seekable)"
                        );
                        std::process::exit(1);
                    }
                    if bgzf_index_load(
                        fp.as_mut().unwrap().as_mut(),
                        (*argv.add(optind as usize)).cast::<u8>().cast_const(),
                        c".gzi".as_ptr().cast::<u8>(),
                    ) < 0
                    {
                        eprintln!(
                            "Could not load index: {}.gzi",
                            CStr::from_ptr(*argv.add(optind as usize)).to_string_lossy()
                        );
                        std::process::exit(1);
                    }
                }
                if bgzf_useek(fp.as_mut().unwrap().as_mut(), start as i64, libc::SEEK_SET) < 0 {
                    eprintln!("Could not seek to {start}-th (uncompressd) byte");
                    std::process::exit(1);
                }
            }

            if threads > 1 && bgzf_mt(fp.as_mut().unwrap().as_mut(), threads, 256) != 0 {
                eprintln!("[bgzip] threaded BGZF is not yet supported in this translation");
                let _ = bgzip_close_bgzf(&mut fp);
                return 1;
            }

            let start_reg = start;
            let end_reg = end;
            if end < 0 && start == 0 {
                loop {
                    let mut block_data: *const () = ptr::null();
                    let fp_ref = fp.as_mut().unwrap().as_mut();
                    c = bgzf_read_block_data(fp_ref, &mut block_data) as i32;
                    if c == 0 {
                        break;
                    }
                    if c < 0 {
                        eprintln!(
                            "Error {} in block starting at offset {}({:X})",
                            fp_ref.bitfields >> 16,
                            fp_ref.block_address,
                            fp_ref.block_address
                        );
                        std::process::exit(1);
                    }
                    if !test
                        && libc::write(f_dst, block_data.cast(), c as usize) != c as isize
                    {
                        eprintln!("Could not write {c} bytes");
                        std::process::exit(1);
                    }
                }
            } else {
                loop {
                    c = bgzf_read(
                        fp.as_mut().unwrap().as_mut(),
                        buffer.cast(),
                        if end < 0 || end - start > WINDOW_SIZE as i64 {
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
                        eprintln!(
                            "Error {} in block starting at offset {}({:X})",
                            fp_ref.bitfields >> 16,
                            fp_ref.block_address,
                            fp_ref.block_address
                        );
                        std::process::exit(1);
                    }
                    start += c as i64;
                    if !test
                        && libc::write(f_dst, buffer.cast(), c as usize) != c as isize
                    {
                        eprintln!("Could not write {c} bytes");
                        std::process::exit(1);
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
                eprintln!("Close failed: Error {close_err}");
                std::process::exit(1);
            }

            if let Some(statfilename) = &statfilename {
                if write_fname.is_none() {
                    let input_name = CStr::from_ptr(*argv.add(optind as usize))
                        .to_bytes()
                        .to_vec();
                    // `statfilename` is NUL-terminated; strip the terminator.
                    let output_name = &statfilename[..statfilename.len() - 1];
                    if bgzip_getfilespec(&input_name, &mut filestat) == 0 {
                        if bgzip_setfilespec(output_name, &filestat) < 0 {
                            eprintln!("[bgzip] Failed to set file specification.");
                        }
                    } else {
                        eprintln!("[bgzip] Failed to get file specification.");
                    }
                }
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
        if libc::close(libc::STDOUT_FILENO) != 0 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno != libc::EBADF {
                eprint!("[bgzip] Failed to close stdout, errno {errno}");
                ret = 1;
            }
        }
    } else if let Some(write_fname) = write_fname {
        if compress {
            if index {
                if let Some(index_fname) = index_fname {
                    if bgzf_index_dump(
                        fp.as_mut().unwrap().as_mut(),
                        index_fname.as_ptr().cast::<u8>(),
                        ptr::null(),
                    ) < 0
                    {
                        eprintln!(
                            "Could not write index to '{}'",
                            index_fname.to_string_lossy()
                        );
                        std::process::exit(1);
                    }
                } else if bgzf_index_dump(
                    fp.as_mut().unwrap().as_mut(),
                    write_fname.as_ptr().cast::<u8>(),
                    c".gzi".as_ptr().cast::<u8>(),
                ) < 0
                {
                    eprintln!(
                        "Could not write index to '{}.gzi'",
                        write_fname.to_string_lossy()
                    );
                    std::process::exit(1);
                }
            }

            let (close_ret, close_err) = bgzip_close_bgzf(&mut fp);
            if close_ret < 0 {
                eprintln!("Output close failed: Error {close_err}");
                std::process::exit(1);
            }
        } else {
            libc::close(f_dst);
        }
    }

    ret
}
