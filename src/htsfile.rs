/*  htsfile.c -- file identifier and minimal viewer.

    Copyright (C) 2014-2019 Genome Research Ltd.

    Author: John Marshall <jm18@sanger.ac.uk>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.  */

use crate::htslib_rs::{hfile, hts, sam, vcf};
use std::ffi::{c_char, c_int, c_void};
use std::ptr;

const IDENTIFY: c_int = 0;
const VIEW_HEADERS: c_int = 1;
const VIEW_ALL: c_int = 2;
const COPY: c_int = 3;
const EFTYPE: c_int = libc::ENOEXEC;
const NO_ARGUMENT: c_int = 0;

unsafe extern "C" {
    static mut optind: c_int;
}

static mut MODE: c_int = IDENTIFY;
static mut SHOW_HEADERS: c_int = 1;
static mut VERBOSE: c_int = 0;
static mut STATUS: c_int = libc::EXIT_SUCCESS;

// original: error (htslib/htsfile.c:47)
macro_rules! error {
    ($format:expr $(, $arg:expr)* $(,)?) => {{
        let err = *crate::htslib_rs::c_compat::__errno_location();
        libc::fflush(crate::htslib_rs::c_compat::stdout.cast());
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"htsfile: ".as_ptr());
        libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), $format.as_ptr() $(, $arg)*);
        if err != 0 {
            libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c": %s\n".as_ptr(), libc::strerror(err));
        } else {
            libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"\n".as_ptr());
        }
        libc::fflush(crate::htslib_rs::c_compat::stderr.cast());
        STATUS = libc::EXIT_FAILURE;
    }};
}

// original: view_sam (htslib/htsfile.c:64)
#[allow(clippy::never_loop)]
pub unsafe fn htsfile_c_64_view_sam(in_: *mut hts::htsFile, filename: *const c_char) {
    let mut b: *mut sam::bam1_t = ptr::null_mut();
    let hdr: *mut sam::sam_hdr_t;
    let mut out: *mut hts::htsFile = ptr::null_mut();

    'clean: loop {
        hdr = sam::sam_hdr_read(in_);
        if hdr.is_null() {
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            error!(c"reading headers from \"%s\" failed", filename);
            break 'clean;
        }

        out = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if out.is_null() {
            error!(c"reopening standard output failed");
            break 'clean;
        }

        if SHOW_HEADERS != 0 && sam::sam_hdr_write(out, hdr) != 0 {
            error!(c"writing headers to standard output failed");
            break 'clean;
        }

        if MODE == VIEW_ALL {
            let mut ret: c_int;

            b = sam::bam_init1();
            if b.is_null() {
                error!(c"can't create record");
                break 'clean;
            }

            loop {
                ret = sam::sam_read1(in_, hdr, b);
                if ret < 0 {
                    break;
                }
                if sam::sam_c_4553_sam_write1(out, hdr, b) < 0 {
                    error!(c"writing to standard output failed");
                    break 'clean;
                }
            }

            if ret < -1 {
                error!(c"reading \"%s\" failed", filename);
                break 'clean;
            }
        }

        break 'clean;
    }

    sam::sam_hdr_destroy(hdr);
    sam::bam_destroy1(b);
    if !out.is_null() {
        hts::hts_close(out);
    }
}

// original: view_vcf (htslib/htsfile.c:108)
#[allow(clippy::never_loop)]
pub unsafe fn htsfile_c_108_view_vcf(in_: *mut hts::htsFile, filename: *const c_char) {
    let mut rec: *mut vcf::bcf1_t = ptr::null_mut();
    let hdr: *mut vcf::bcf_hdr_t;
    let mut out: *mut hts::htsFile = ptr::null_mut();

    'clean: loop {
        hdr = vcf::bcf_hdr_read(in_);
        if hdr.is_null() {
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            error!(c"reading headers from \"%s\" failed", filename);
            break 'clean;
        }

        out = hts::hts_open(c"-".as_ptr(), c"w".as_ptr());
        if out.is_null() {
            error!(c"reopening standard output failed");
            break 'clean;
        }

        if SHOW_HEADERS != 0 && vcf::bcf_hdr_write(out, hdr) != 0 {
            error!(c"writing headers to standard output failed");
            break 'clean;
        }

        if MODE == VIEW_ALL {
            let mut ret: c_int;

            rec = vcf::bcf_init();
            if rec.is_null() {
                error!(c"can't create record");
                break 'clean;
            }

            loop {
                ret = vcf::bcf_read(in_, hdr, rec);
                if ret < 0 {
                    break;
                }
                if vcf::bcf_write(out, hdr, rec) < 0 {
                    error!(c"writing to standard output failed");
                    break 'clean;
                }
            }

            if ret < -1 {
                error!(c"reading \"%s\" failed", filename);
                break 'clean;
            }
        }

        break 'clean;
    }

    if !hdr.is_null() {
        vcf::bcf_hdr_destroy(hdr);
    }
    if !rec.is_null() {
        vcf::bcf_destroy(rec);
    }
    if !out.is_null() {
        hts::hts_close(out);
    }
}

// original: view_raw (htslib/htsfile.c:152)
pub unsafe fn htsfile_c_152_view_raw(fp: *mut hts::hFILE, filename: *const c_char) {
    let mut prev = b'\n' as c_int;
    loop {
        let c = hfile::htslib_hfile_h_163_hgetc(fp);
        if c == libc::EOF {
            break;
        }
        if libc::isprint(c) != 0 || c == b'\n' as c_int || c == b'\t' as c_int {
            libc::putchar(c);
        } else if c == b'\r' as c_int {
            libc::fputs(c"\\r".as_ptr(), crate::htslib_rs::c_compat::stdout.cast());
        } else if c == 0 {
            libc::fputs(c"\\0".as_ptr(), crate::htslib_rs::c_compat::stdout.cast());
        } else {
            libc::printf(c"\\x%02x".as_ptr(), c);
        }
        prev = c;
    }

    if prev != b'\n' as c_int {
        libc::putchar(b'\n' as c_int);
    }

    if hfile::htslib_hfile_h_134_herrno(fp) != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = hfile::htslib_hfile_h_134_herrno(fp);
        error!(c"reading \"%s\" failed", filename);
    }
}

// original: copy_raw (htslib/htsfile.c:169)
pub unsafe fn htsfile_c_169_copy_raw(srcfilename: *const c_char, destfilename: *const c_char) {
    let mut src = hfile::hopen(srcfilename, c"r".as_ptr());
    if src.is_null() {
        error!(c"can't open \"%s\"", srcfilename);
        return;
    }

    let bufsize = 1_048_576usize;
    let buffer = libc::malloc(bufsize).cast::<c_char>();
    if buffer.is_null() {
        error!(c"can't allocate copy buffer");
        hfile::hclose_abruptly(src);
        return;
    }

    let mut dest = hfile::hopen(destfilename, c"w".as_ptr());
    if dest.is_null() {
        error!(c"can't create \"%s\"", destfilename);
        hfile::hclose_abruptly(src);
        libc::free(buffer.cast());
        return;
    }

    let mut n: isize;
    loop {
        n = hfile::htslib_hfile_h_247_hread(src, buffer.cast(), bufsize);
        if n <= 0 {
            break;
        }
        if hfile::htslib_hfile_h_292_hwrite(dest, buffer.cast_const().cast::<c_void>(), n as usize)
            != n
        {
            error!(c"writing to \"%s\" failed", destfilename);
            hfile::hclose_abruptly(dest);
            dest = ptr::null_mut();
            break;
        }
    }

    if n < 0 {
        error!(c"reading from \"%s\" failed", srcfilename);
        hfile::hclose_abruptly(src);
        src = ptr::null_mut();
    }

    if !dest.is_null() && hfile::hclose(dest) < 0 {
        error!(c"closing \"%s\" failed", destfilename);
    }
    if !src.is_null() && hfile::hclose(src) < 0 {
        error!(c"closing \"%s\" failed", srcfilename);
    }
    libc::free(buffer.cast());
}

// original: usage (htslib/htsfile.c:213)
pub unsafe fn htsfile_c_213_usage(fp: *mut libc::FILE, status: c_int) -> ! {
    libc::fprintf(
        fp,
        c"Usage: htsfile [-chHv] FILE...\n       htsfile --copy [-v] FILE DESTFILE\nOptions:\n  -c, --view         Write textual form of FILEs to standard output\n  -C, --copy         Copy the exact contents of FILE to DESTFILE\n  -h, --header-only  Display only headers in view mode, not records\n  -H, --no-header    Suppress header display in view mode\n  -v, --verbose      Increase verbosity of warnings and diagnostics\n"
            .as_ptr(),
    );
    libc::exit(status);
}

// original: main (htslib/htsfile.c:227)
pub unsafe fn htsfile_c_227_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut options = [
        libc::option {
            name: c"copy".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'C' as c_int,
        },
        libc::option {
            name: c"header-only".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'h' as c_int,
        },
        libc::option {
            name: c"no-header".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'H' as c_int,
        },
        libc::option {
            name: c"view".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'c' as c_int,
        },
        libc::option {
            name: c"verbose".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'v' as c_int,
        },
        libc::option {
            name: c"help".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 2,
        },
        libc::option {
            name: c"version".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: 1,
        },
        libc::option {
            name: ptr::null(),
            has_arg: 0,
            flag: ptr::null_mut(),
            val: 0,
        },
    ];

    STATUS = libc::EXIT_SUCCESS;
    loop {
        let c = libc::getopt_long(
            argc,
            argv,
            c"cChHv".as_ptr(),
            options.as_mut_ptr(),
            ptr::null_mut(),
        );
        if c < 0 {
            break;
        }
        match c {
            x if x == b'c' as c_int => MODE = VIEW_ALL,
            x if x == b'C' as c_int => MODE = COPY,
            x if x == b'h' as c_int => {
                MODE = VIEW_HEADERS;
                SHOW_HEADERS = 1;
            }
            x if x == b'H' as c_int => SHOW_HEADERS = 0,
            x if x == b'v' as c_int => {
                hts::hts_verbose += 1;
                VERBOSE += 1;
            }
            1 => {
                libc::printf(
                    c"htsfile (htslib) %s\nCopyright (C) 2025 Genome Research Ltd.\n".as_ptr(),
                    hts::hts_version(),
                );
                libc::exit(libc::EXIT_SUCCESS);
            }
            2 => htsfile_c_213_usage(
                crate::htslib_rs::c_compat::stdout.cast(),
                libc::EXIT_SUCCESS,
            ),
            _ => htsfile_c_213_usage(
                crate::htslib_rs::c_compat::stderr.cast(),
                libc::EXIT_FAILURE,
            ),
        }
    }

    if optind == argc {
        htsfile_c_213_usage(
            crate::htslib_rs::c_compat::stderr.cast(),
            libc::EXIT_FAILURE,
        );
    }

    if MODE == COPY {
        if optind + 2 != argc {
            htsfile_c_213_usage(
                crate::htslib_rs::c_compat::stderr.cast(),
                libc::EXIT_FAILURE,
            );
        }
        htsfile_c_169_copy_raw(*argv.add(optind as usize), *argv.add(optind as usize + 1));
        return STATUS;
    }

    let mut i = optind;
    while i < argc {
        let mut fp = hfile::hopen(*argv.add(i as usize), c"r".as_ptr());
        if fp.is_null() {
            error!(c"can't open \"%s\"", *argv.add(i as usize));
            i += 1;
            continue;
        }

        if MODE == IDENTIFY {
            let mut fmt: hts::htsFormat = std::mem::zeroed();
            if hts::hts_detect_format2(fp, *argv.add(i as usize), &mut fmt) < 0 {
                error!(c"detecting \"%s\" format failed", *argv.add(i as usize));
                hfile::hclose_abruptly(fp);
                i += 1;
                continue;
            }

            let description = hts::hts_format_description(&fmt);
            libc::printf(c"%s:\t%s\n".as_ptr(), *argv.add(i as usize), description);
            libc::free(description.cast());
        } else {
            let hts = hts::hts_hopen(fp, *argv.add(i as usize), c"r".as_ptr());
            if !hts.is_null() {
                match (*hts::hts_get_format(hts)).category {
                    hts::HTS_FORMAT_SEQUENCE_DATA => {
                        htsfile_c_64_view_sam(hts, *argv.add(i as usize))
                    }
                    hts::HTS_FORMAT_VARIANT_DATA => {
                        htsfile_c_108_view_vcf(hts, *argv.add(i as usize))
                    }
                    _ => {
                        if VERBOSE != 0 {
                            htsfile_c_152_view_raw(fp, *argv.add(i as usize));
                        } else {
                            *crate::htslib_rs::c_compat::__errno_location() = 0;
                            error!(c"can't view \"%s\": unknown format", *argv.add(i as usize));
                        }
                    }
                }

                if hts::hts_close(hts) < 0 {
                    error!(c"closing \"%s\" failed", *argv.add(i as usize));
                }
                fp = ptr::null_mut();
            } else if (*crate::htslib_rs::c_compat::__errno_location() == EFTYPE
                || *crate::htslib_rs::c_compat::__errno_location() == libc::ENOEXEC)
                && VERBOSE != 0
            {
                htsfile_c_152_view_raw(fp, *argv.add(i as usize));
            } else {
                error!(c"can't view \"%s\"", *argv.add(i as usize));
            }
        }

        if !fp.is_null() && hfile::hclose(fp) < 0 {
            error!(c"closing \"%s\" failed", *argv.add(i as usize));
        }
        i += 1;
    }

    if libc::fclose(crate::htslib_rs::c_compat::stdout.cast()) != 0
        && *crate::htslib_rs::c_compat::__errno_location() != libc::EBADF
    {
        error!(c"closing standard output failed");
    }

    STATUS
}
