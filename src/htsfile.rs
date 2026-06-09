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
use std::ptr::{self, NonNull};

const IDENTIFY: i32 = 0;
const VIEW_HEADERS: i32 = 1;
const VIEW_ALL: i32 = 2;
const COPY: i32 = 3;
const EFTYPE: i32 = libc::ENOEXEC;
const NO_ARGUMENT: i32 = 0;

unsafe extern "C" {
    static mut optind: i32;
}

static mut MODE: i32 = IDENTIFY;
static mut SHOW_HEADERS: i32 = 1;
static mut VERBOSE: i32 = 0;
static mut STATUS: i32 = libc::EXIT_SUCCESS;

const MODE_READ: &[u8] = b"r\0";
const MODE_WRITE: &[u8] = b"w\0";
const STDIO_FILENAME: &[u8] = b"-\0";
const UNKNOWN_FILENAME: &[u8] = b"?\0";

struct CFilename {
    bytes: Vec<u8>,
}

impl CFilename {
    unsafe fn from_ptr(ptr: *const i8) -> Option<Self> {
        NonNull::new(ptr.cast_mut()).map(|ptr| {
            let cstr = std::ffi::CStr::from_ptr(ptr.as_ptr());
            Self {
                bytes: cstr.to_bytes_with_nul().to_vec(),
            }
        })
    }

    fn as_c_ptr(&self) -> *const i8 {
        self.bytes.as_ptr().cast()
    }

    fn as_bytes_with_nul(&self) -> &[u8] {
        &self.bytes
    }
}

fn display_c_ptr(ptr: *const i8) -> *const i8 {
    if ptr.is_null() {
        UNKNOWN_FILENAME.as_ptr().cast()
    } else {
        ptr
    }
}

struct HFileHandle {
    ptr: NonNull<hts::hFILE>,
}

impl HFileHandle {
    unsafe fn open(filename: &[u8], mode: &[u8]) -> Option<Self> {
        NonNull::new(hfile::hopen(filename.as_ptr().cast(), mode.as_ptr().cast()))
            .map(|ptr| Self { ptr })
    }

    fn as_mut(&mut self) -> &mut hts::hFILE {
        unsafe { self.ptr.as_mut() }
    }

    fn as_ptr(&self) -> *mut hts::hFILE {
        self.ptr.as_ptr()
    }

    unsafe fn close(self) -> i32 {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        hfile::hclose(ptr)
    }

    unsafe fn close_abruptly(self) {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        hfile::hclose_abruptly(ptr);
    }
}

impl Drop for HFileHandle {
    fn drop(&mut self) {
        unsafe {
            hfile::hclose_abruptly(self.ptr.as_ptr());
        }
    }
}

struct HtsFileHandle {
    ptr: NonNull<hts::htsFile>,
}

impl HtsFileHandle {
    unsafe fn open(filename: &[u8], mode: &[u8]) -> Option<Self> {
        NonNull::new(hts::hts_open(
            filename.as_ptr().cast(),
            mode.as_ptr().cast(),
        ))
        .map(|ptr| Self { ptr })
    }

    unsafe fn from_hfile(
        hfile: HFileHandle,
        filename: &[u8],
        mode: &[u8],
    ) -> Result<Self, HFileHandle> {
        let ptr = hfile.ptr.as_ptr();
        match NonNull::new(hts::hts_hopen(
            ptr,
            filename.as_ptr().cast(),
            mode.as_ptr().cast(),
        )) {
            Some(ptr) => {
                std::mem::forget(hfile);
                Ok(Self { ptr })
            }
            None => Err(hfile),
        }
    }

    fn as_mut(&mut self) -> &mut hts::htsFile {
        unsafe { self.ptr.as_mut() }
    }

    fn as_ref(&self) -> &hts::htsFile {
        unsafe { self.ptr.as_ref() }
    }

    fn as_ptr(&self) -> *mut hts::htsFile {
        self.ptr.as_ptr()
    }

    unsafe fn close(self) -> i32 {
        let ptr = self.ptr.as_ptr();
        std::mem::forget(self);
        hts::hts_close(ptr)
    }
}

impl Drop for HtsFileHandle {
    fn drop(&mut self) {
        unsafe {
            hts::hts_close(self.ptr.as_ptr());
        }
    }
}

struct SamHeader {
    ptr: NonNull<sam::sam_hdr_t>,
}

impl SamHeader {
    unsafe fn read(input: &mut hts::htsFile) -> Option<Self> {
        NonNull::new(sam::sam_hdr_read(input)).map(|ptr| Self { ptr })
    }

    fn as_ref(&self) -> &sam::sam_hdr_t {
        unsafe { self.ptr.as_ref() }
    }

    fn as_ptr(&self) -> *mut sam::sam_hdr_t {
        self.ptr.as_ptr()
    }
}

impl Drop for SamHeader {
    fn drop(&mut self) {
        unsafe {
            sam::sam_hdr_destroy(self.ptr.as_ptr());
        }
    }
}

struct BamRecord {
    ptr: NonNull<sam::bam1_t>,
}

impl BamRecord {
    unsafe fn new() -> Option<Self> {
        NonNull::new(sam::bam_init1()).map(|ptr| Self { ptr })
    }

    fn as_ref(&self) -> &sam::bam1_t {
        unsafe { self.ptr.as_ref() }
    }

    fn as_ptr(&self) -> *mut sam::bam1_t {
        self.ptr.as_ptr()
    }
}

impl Drop for BamRecord {
    fn drop(&mut self) {
        unsafe {
            sam::bam_destroy1(self.ptr.as_ptr());
        }
    }
}

struct VcfHeader {
    ptr: NonNull<vcf::bcf_hdr_t>,
}

impl VcfHeader {
    unsafe fn read(input: &mut hts::htsFile) -> Option<Self> {
        NonNull::new(vcf::bcf_hdr_read(input)).map(|ptr| Self { ptr })
    }

    fn as_ref(&self) -> &vcf::bcf_hdr_t {
        unsafe { self.ptr.as_ref() }
    }

    fn as_mut(&mut self) -> &mut vcf::bcf_hdr_t {
        unsafe { self.ptr.as_mut() }
    }

    fn as_ptr(&self) -> *mut vcf::bcf_hdr_t {
        self.ptr.as_ptr()
    }
}

impl Drop for VcfHeader {
    fn drop(&mut self) {
        unsafe {
            vcf::bcf_hdr_destroy(self.ptr.as_ptr());
        }
    }
}

struct VcfRecord {
    ptr: NonNull<vcf::bcf1_t>,
}

impl VcfRecord {
    unsafe fn new() -> Option<Self> {
        NonNull::new(vcf::bcf_init()).map(|ptr| Self { ptr })
    }

    fn as_mut(&mut self) -> &mut vcf::bcf1_t {
        unsafe { self.ptr.as_mut() }
    }

    fn as_ptr(&self) -> *mut vcf::bcf1_t {
        self.ptr.as_ptr()
    }
}

impl Drop for VcfRecord {
    fn drop(&mut self) {
        unsafe {
            vcf::bcf_destroy(self.ptr.as_ptr());
        }
    }
}

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
pub unsafe fn htsfile_c_64_view_sam(in_: *mut hts::htsFile, filename: *const i8) {
    let Some(input) = in_.as_mut() else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(
            c"reading headers from \"%s\" failed",
            display_c_ptr(filename)
        );
        return;
    };
    let Some(filename) = CFilename::from_ptr(filename) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(
            c"reading headers from \"%s\" failed",
            UNKNOWN_FILENAME.as_ptr().cast::<i8>()
        );
        return;
    };
    htsfile_c_64_view_sam_ref(input, &filename);
}

unsafe fn htsfile_c_64_view_sam_ref(input: &mut hts::htsFile, filename: &CFilename) {
    'clean: loop {
        let Some(hdr) = SamHeader::read(input) else {
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            error!(c"reading headers from \"%s\" failed", filename.as_c_ptr());
            break 'clean;
        };

        let Some(out) = HtsFileHandle::open(STDIO_FILENAME, MODE_WRITE) else {
            error!(c"reopening standard output failed");
            break 'clean;
        };

        let hdr_ref = hdr.as_ref();
        if SHOW_HEADERS != 0 && sam::sam_hdr_write(out.as_ptr(), hdr_ref) != 0 {
            error!(c"writing headers to standard output failed");
            break 'clean;
        }

        if MODE == VIEW_ALL {
            let mut ret: i32;

            let Some(b) = BamRecord::new() else {
                error!(c"can't create record");
                break 'clean;
            };

            loop {
                ret = sam::sam_read1(input, hdr.as_ptr(), b.as_ptr());
                if ret < 0 {
                    break;
                }
                let b_ref = b.as_ref();
                if sam::sam_c_4553_sam_write1(out.as_ptr(), hdr_ref, b_ref) < 0 {
                    error!(c"writing to standard output failed");
                    break 'clean;
                }
            }

            if ret < -1 {
                error!(c"reading \"%s\" failed", filename.as_c_ptr());
                break 'clean;
            }
        }

        break 'clean;
    }
}

// original: view_vcf (htslib/htsfile.c:108)
#[allow(clippy::never_loop)]
pub unsafe fn htsfile_c_108_view_vcf(in_: *mut hts::htsFile, filename: *const i8) {
    let Some(input) = in_.as_mut() else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(
            c"reading headers from \"%s\" failed",
            display_c_ptr(filename)
        );
        return;
    };
    let Some(filename) = CFilename::from_ptr(filename) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(
            c"reading headers from \"%s\" failed",
            UNKNOWN_FILENAME.as_ptr().cast::<i8>()
        );
        return;
    };
    htsfile_c_108_view_vcf_ref(input, &filename);
}

unsafe fn htsfile_c_108_view_vcf_ref(input: &mut hts::htsFile, filename: &CFilename) {
    'clean: loop {
        let Some(mut hdr) = VcfHeader::read(input) else {
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            error!(c"reading headers from \"%s\" failed", filename.as_c_ptr());
            break 'clean;
        };

        let Some(out) = HtsFileHandle::open(STDIO_FILENAME, MODE_WRITE) else {
            error!(c"reopening standard output failed");
            break 'clean;
        };

        if SHOW_HEADERS != 0 && vcf::bcf_hdr_write(out.as_ptr(), hdr.as_ptr()) != 0 {
            error!(c"writing headers to standard output failed");
            break 'clean;
        }

        if MODE == VIEW_ALL {
            let mut ret: i32;

            let Some(mut rec) = VcfRecord::new() else {
                error!(c"can't create record");
                break 'clean;
            };

            loop {
                ret = vcf::bcf_read(input, hdr.as_ref(), rec.as_ptr());
                if ret < 0 {
                    break;
                }
                if vcf::bcf_write(out.as_ptr(), hdr.as_mut(), rec.as_mut()) < 0 {
                    error!(c"writing to standard output failed");
                    break 'clean;
                }
            }

            if ret < -1 {
                error!(c"reading \"%s\" failed", filename.as_c_ptr());
                break 'clean;
            }
        }

        break 'clean;
    }
}

// original: view_raw (htslib/htsfile.c:152)
pub unsafe fn htsfile_c_152_view_raw(fp: *mut hts::hFILE, filename: *const i8) {
    let Some(fp) = fp.as_mut() else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(c"reading \"%s\" failed", display_c_ptr(filename));
        return;
    };
    let Some(filename) = CFilename::from_ptr(filename) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(
            c"reading \"%s\" failed",
            UNKNOWN_FILENAME.as_ptr().cast::<i8>()
        );
        return;
    };
    htsfile_c_152_view_raw_ref(fp, &filename);
}

unsafe fn htsfile_c_152_view_raw_ref(fp: &mut hts::hFILE, filename: &CFilename) {
    let mut prev = b'\n' as i32;
    loop {
        let c = hfile::htslib_hfile_h_163_hgetc_ref(fp);
        if c == libc::EOF {
            break;
        }
        if libc::isprint(c) != 0 || c == b'\n' as i32 || c == b'\t' as i32 {
            libc::putchar(c);
        } else if c == b'\r' as i32 {
            libc::fputs(c"\\r".as_ptr(), crate::htslib_rs::c_compat::stdout.cast());
        } else if c == 0 {
            libc::fputs(c"\\0".as_ptr(), crate::htslib_rs::c_compat::stdout.cast());
        } else {
            libc::printf(c"\\x%02x".as_ptr(), c);
        }
        prev = c;
    }

    if prev != b'\n' as i32 {
        libc::putchar(b'\n' as i32);
    }

    let herrno = hfile::htslib_hfile_h_134_herrno_ref(fp);
    if herrno != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = herrno;
        error!(c"reading \"%s\" failed", filename.as_c_ptr());
    }
}

// original: copy_raw (htslib/htsfile.c:169)
pub unsafe fn htsfile_c_169_copy_raw(srcfilename: *const i8, destfilename: *const i8) {
    let Some(srcfilename) = CFilename::from_ptr(srcfilename) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(c"can't open \"%s\"", UNKNOWN_FILENAME.as_ptr().cast::<i8>());
        return;
    };
    let Some(destfilename) = CFilename::from_ptr(destfilename) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        error!(
            c"can't create \"%s\"",
            UNKNOWN_FILENAME.as_ptr().cast::<i8>()
        );
        return;
    };
    htsfile_c_169_copy_raw_ref(&srcfilename, &destfilename);
}

unsafe fn htsfile_c_169_copy_raw_ref(srcfilename: &CFilename, destfilename: &CFilename) {
    let Some(mut src) = HFileHandle::open(srcfilename.as_bytes_with_nul(), MODE_READ) else {
        error!(c"can't open \"%s\"", srcfilename.as_c_ptr());
        return;
    };

    let mut buffer = Vec::<u8>::new();
    if buffer.try_reserve_exact(1_048_576).is_err() {
        error!(c"can't allocate copy buffer");
        return;
    }
    buffer.resize(1_048_576, 0);

    let Some(mut dest) = HFileHandle::open(destfilename.as_bytes_with_nul(), MODE_WRITE) else {
        error!(c"can't create \"%s\"", destfilename.as_c_ptr());
        return;
    };

    let mut n: isize;
    loop {
        n = hfile::htslib_hfile_h_247_hread_ref(src.as_mut(), &mut buffer);
        if n <= 0 {
            break;
        }
        if hfile::htslib_hfile_h_292_hwrite_ref(dest.as_mut(), &buffer[..n as usize]) != n {
            error!(c"writing to \"%s\" failed", destfilename.as_c_ptr());
            return;
        }
    }

    if n < 0 {
        error!(c"reading from \"%s\" failed", srcfilename.as_c_ptr());
        return;
    }

    if dest.close() < 0 {
        error!(c"closing \"%s\" failed", destfilename.as_c_ptr());
    }
    if src.close() < 0 {
        error!(c"closing \"%s\" failed", srcfilename.as_c_ptr());
    }
}

// original: usage (htslib/htsfile.c:213)
pub unsafe fn htsfile_c_213_usage(fp: *mut libc::FILE, status: i32) -> ! {
    libc::fprintf(
        fp,
        c"Usage: htsfile [-chHv] FILE...\n       htsfile --copy [-v] FILE DESTFILE\nOptions:\n  -c, --view         Write textual form of FILEs to standard output\n  -C, --copy         Copy the exact contents of FILE to DESTFILE\n  -h, --header-only  Display only headers in view mode, not records\n  -H, --no-header    Suppress header display in view mode\n  -v, --verbose      Increase verbosity of warnings and diagnostics\n"
            .as_ptr(),
    );
    libc::exit(status);
}

// original: main (htslib/htsfile.c:227)
pub unsafe fn htsfile_c_227_main(argc: i32, argv: *mut *mut i8) -> i32 {
    let mut options = [
        libc::option {
            name: c"copy".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'C' as i32,
        },
        libc::option {
            name: c"header-only".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'h' as i32,
        },
        libc::option {
            name: c"no-header".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'H' as i32,
        },
        libc::option {
            name: c"view".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'c' as i32,
        },
        libc::option {
            name: c"verbose".as_ptr(),
            has_arg: NO_ARGUMENT,
            flag: ptr::null_mut(),
            val: b'v' as i32,
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
            x if x == b'c' as i32 => MODE = VIEW_ALL,
            x if x == b'C' as i32 => MODE = COPY,
            x if x == b'h' as i32 => {
                MODE = VIEW_HEADERS;
                SHOW_HEADERS = 1;
            }
            x if x == b'H' as i32 => SHOW_HEADERS = 0,
            x if x == b'v' as i32 => {
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
        let Some(srcfilename) = CFilename::from_ptr(*argv.add(optind as usize)) else {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            error!(c"can't open \"%s\"", UNKNOWN_FILENAME.as_ptr().cast::<i8>());
            return STATUS;
        };
        let Some(destfilename) = CFilename::from_ptr(*argv.add(optind as usize + 1)) else {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            error!(
                c"can't create \"%s\"",
                UNKNOWN_FILENAME.as_ptr().cast::<i8>()
            );
            return STATUS;
        };
        htsfile_c_169_copy_raw_ref(&srcfilename, &destfilename);
        return STATUS;
    }

    let mut i = optind;
    while i < argc {
        let Some(filename) = CFilename::from_ptr(*argv.add(i as usize)) else {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            error!(c"can't open \"%s\"", UNKNOWN_FILENAME.as_ptr().cast::<i8>());
            i += 1;
            continue;
        };
        let Some(fp) = HFileHandle::open(filename.as_bytes_with_nul(), MODE_READ) else {
            error!(c"can't open \"%s\"", filename.as_c_ptr());
            i += 1;
            continue;
        };

        if MODE == IDENTIFY {
            let mut fmt: hts::htsFormat = std::mem::zeroed();
            if hts::hts_detect_format2(fp.as_ptr(), filename.as_c_ptr(), &mut fmt) < 0 {
                error!(c"detecting \"%s\" format failed", filename.as_c_ptr());
                i += 1;
                continue;
            }

            let description = hts::hts_format_description(&fmt);
            libc::printf(c"%s:\t%s\n".as_ptr(), filename.as_c_ptr(), description);
            libc::free(description.cast());
        } else {
            match HtsFileHandle::from_hfile(fp, filename.as_bytes_with_nul(), MODE_READ) {
                Ok(mut hts_file) => {
                    match hts_file.as_ref().format.category {
                        hts::HTS_FORMAT_SEQUENCE_DATA => {
                            htsfile_c_64_view_sam_ref(hts_file.as_mut(), &filename)
                        }
                        hts::HTS_FORMAT_VARIANT_DATA => {
                            htsfile_c_108_view_vcf_ref(hts_file.as_mut(), &filename)
                        }
                        _ => {
                            if VERBOSE != 0 {
                                let fp = hts::hts_hfile(hts_file.as_ptr());
                                let Some(fp) = fp.as_mut() else {
                                    *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
                                    error!(c"reading \"%s\" failed", filename.as_c_ptr());
                                    i += 1;
                                    continue;
                                };
                                htsfile_c_152_view_raw_ref(fp, &filename);
                            } else {
                                *crate::htslib_rs::c_compat::__errno_location() = 0;
                                error!(c"can't view \"%s\": unknown format", filename.as_c_ptr());
                            }
                        }
                    }

                    if hts_file.close() < 0 {
                        error!(c"closing \"%s\" failed", filename.as_c_ptr());
                    }
                    i += 1;
                    continue;
                }
                Err(mut fp) => {
                    if (*crate::htslib_rs::c_compat::__errno_location() == EFTYPE
                        || *crate::htslib_rs::c_compat::__errno_location() == libc::ENOEXEC)
                        && VERBOSE != 0
                    {
                        htsfile_c_152_view_raw_ref(fp.as_mut(), &filename);
                    } else {
                        error!(c"can't view \"%s\"", filename.as_c_ptr());
                    }

                    if fp.close() < 0 {
                        error!(c"closing \"%s\" failed", filename.as_c_ptr());
                    }
                    i += 1;
                    continue;
                }
            }
        }

        if fp.close() < 0 {
            error!(c"closing \"%s\" failed", filename.as_c_ptr());
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
