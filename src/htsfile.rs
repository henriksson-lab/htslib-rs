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
use std::io::Write as _;
use std::ptr::NonNull;

const IDENTIFY: i32 = 0;
const VIEW_HEADERS: i32 = 1;
const VIEW_ALL: i32 = 2;
const COPY: i32 = 3;
const EFTYPE: i32 = libc::ENOEXEC;

static mut MODE: i32 = IDENTIFY;
static mut SHOW_HEADERS: i32 = 1;
static mut VERBOSE: i32 = 0;
static mut STATUS: i32 = 0;

// NUL-terminated mode/filename byte strings retained for the C FFI boundary.
const MODE_READ: &[u8] = b"r\0";
const MODE_WRITE: &[u8] = b"w\0";
const STDIO_FILENAME: &[u8] = b"-\0";

/// An owned filename. `bytes` keeps a trailing NUL so it can be handed to the
/// remaining C FFI entry points; `display()` yields the human-readable form
/// without the terminator.
pub struct CFilename {
    bytes: Vec<u8>,
}

impl CFilename {
    /// Build from an owned, NUL-free byte string.
    fn from_bytes(mut bytes: Vec<u8>) -> Self {
        bytes.push(0);
        Self { bytes }
    }

    /// NUL-terminated bytes for FFI calls.
    fn as_bytes_with_nul(&self) -> &[u8] {
        &self.bytes
    }

    /// Human-readable filename without the trailing NUL.
    fn display(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.bytes[..self.bytes.len() - 1])
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
//
// Emits "htsfile: <message>[: <strerror>]\n" to stderr (after flushing stdout)
// and records the failure in STATUS. `$err` is the saved errno (0 to suppress
// the strerror suffix). `$msg` is a pre-formatted message string.
macro_rules! error {
    ($err:expr, $msg:expr) => {{
        let err: i32 = $err;
        let msg: String = $msg;
        let _ = std::io::stdout().flush();
        if err != 0 {
            eprintln!(
                "htsfile: {}: {}",
                msg,
                std::io::Error::from_raw_os_error(err)
            );
        } else {
            eprintln!("htsfile: {}", msg);
        }
        let _ = std::io::stderr().flush();
        STATUS = 1;
    }};
}

// original: view_sam (htslib/htsfile.c:64)
#[allow(clippy::never_loop)]
pub unsafe fn htsfile_c_64_view_sam(input: &mut hts::htsFile, filename: &CFilename) {
    'clean: loop {
        let Some(hdr) = SamHeader::read(input) else {
            error!(0, format!("reading headers from \"{}\" failed", filename.display()));
            break 'clean;
        };

        let Some(out) = HtsFileHandle::open(STDIO_FILENAME, MODE_WRITE) else {
            error!(0, "reopening standard output failed".to_string());
            break 'clean;
        };

        let hdr_ref = hdr.as_ref();
        if SHOW_HEADERS != 0 && sam::sam_hdr_write(out.as_ptr(), hdr_ref) != 0 {
            error!(0, "writing headers to standard output failed".to_string());
            break 'clean;
        }

        if MODE == VIEW_ALL {
            let mut ret: i32;

            let Some(b) = BamRecord::new() else {
                error!(0, "can't create record".to_string());
                break 'clean;
            };

            loop {
                ret = sam::sam_read1(input, hdr.as_ptr(), b.as_ptr());
                if ret < 0 {
                    break;
                }
                let b_ref = b.as_ref();
                if sam::sam_c_4553_sam_write1(out.as_ptr(), hdr_ref, b_ref) < 0 {
                    error!(0, "writing to standard output failed".to_string());
                    break 'clean;
                }
            }

            if ret < -1 {
                error!(0, format!("reading \"{}\" failed", filename.display()));
                break 'clean;
            }
        }

        break 'clean;
    }
}

// original: view_vcf (htslib/htsfile.c:108)
#[allow(clippy::never_loop)]
pub unsafe fn htsfile_c_108_view_vcf(input: &mut hts::htsFile, filename: &CFilename) {
    'clean: loop {
        let Some(mut hdr) = VcfHeader::read(input) else {
            error!(0, format!("reading headers from \"{}\" failed", filename.display()));
            break 'clean;
        };

        let Some(out) = HtsFileHandle::open(STDIO_FILENAME, MODE_WRITE) else {
            error!(0, "reopening standard output failed".to_string());
            break 'clean;
        };

        if SHOW_HEADERS != 0 && vcf::bcf_hdr_write(out.as_ptr(), hdr.as_ptr()) != 0 {
            error!(0, "writing headers to standard output failed".to_string());
            break 'clean;
        }

        if MODE == VIEW_ALL {
            let mut ret: i32;

            let Some(mut rec) = VcfRecord::new() else {
                error!(0, "can't create record".to_string());
                break 'clean;
            };

            loop {
                ret = vcf::bcf_read(input, hdr.as_ref(), rec.as_ptr());
                if ret < 0 {
                    break;
                }
                if vcf::bcf_write(out.as_ptr(), hdr.as_mut(), rec.as_mut()) < 0 {
                    error!(0, "writing to standard output failed".to_string());
                    break 'clean;
                }
            }

            if ret < -1 {
                error!(0, format!("reading \"{}\" failed", filename.display()));
                break 'clean;
            }
        }

        break 'clean;
    }
}

// original: view_raw (htslib/htsfile.c:152)
pub unsafe fn htsfile_c_152_view_raw(fp: &mut hts::hFILE, filename: &CFilename) {
    let mut prev = b'\n' as i32;
    let mut out = std::io::stdout().lock();
    loop {
        let c = hfile::htslib_hfile_h_163_hgetc(fp);
        if c == libc::EOF {
            break;
        }
        if (0x20..=0x7e).contains(&c) || c == b'\n' as i32 || c == b'\t' as i32 {
            let _ = out.write_all(&[c as u8]);
        } else if c == b'\r' as i32 {
            let _ = out.write_all(b"\\r");
        } else if c == 0 {
            let _ = out.write_all(b"\\0");
        } else {
            let _ = write!(out, "\\x{:02x}", c);
        }
        prev = c;
    }

    if prev != b'\n' as i32 {
        let _ = out.write_all(b"\n");
    }
    drop(out);

    let herrno = hfile::htslib_hfile_h_134_herrno(fp);
    if herrno != 0 {
        error!(herrno, format!("reading \"{}\" failed", filename.display()));
    }
}

// original: copy_raw (htslib/htsfile.c:169)
pub unsafe fn htsfile_c_169_copy_raw(srcfilename: &CFilename, destfilename: &CFilename) {
    let Some(mut src) = HFileHandle::open(srcfilename.as_bytes_with_nul(), MODE_READ) else {
        error!(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            format!("can't open \"{}\"", srcfilename.display())
        );
        return;
    };

    let mut buffer = Vec::<u8>::new();
    if buffer.try_reserve_exact(1_048_576).is_err() {
        error!(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            "can't allocate copy buffer".to_string()
        );
        return;
    }
    buffer.resize(1_048_576, 0);

    let Some(mut dest) = HFileHandle::open(destfilename.as_bytes_with_nul(), MODE_WRITE) else {
        error!(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            format!("can't create \"{}\"", destfilename.display())
        );
        return;
    };

    let mut n: isize;
    loop {
        n = hfile::hread(src.as_mut(), &mut buffer);
        if n <= 0 {
            break;
        }
        if hfile::hwrite(dest.as_mut(), &buffer[..n as usize]) != n {
            error!(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                format!("writing to \"{}\" failed", destfilename.display())
            );
            return;
        }
    }

    if n < 0 {
        error!(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            format!("reading from \"{}\" failed", srcfilename.display())
        );
        return;
    }

    if dest.close() < 0 {
        error!(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            format!("closing \"{}\" failed", destfilename.display())
        );
    }
    if src.close() < 0 {
        error!(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            format!("closing \"{}\" failed", srcfilename.display())
        );
    }
}

// original: usage (htslib/htsfile.c:213)
//
// `to_stdout` selects the destination stream; `status` is the process exit code.
pub fn htsfile_c_213_usage(to_stdout: bool, status: i32) -> ! {
    const TEXT: &str = "Usage: htsfile [-chHv] FILE...\n       htsfile --copy [-v] FILE DESTFILE\nOptions:\n  -c, --view         Write textual form of FILEs to standard output\n  -C, --copy         Copy the exact contents of FILE to DESTFILE\n  -h, --header-only  Display only headers in view mode, not records\n  -H, --no-header    Suppress header display in view mode\n  -v, --verbose      Increase verbosity of warnings and diagnostics\n";
    if to_stdout {
        print!("{}", TEXT);
    } else {
        eprint!("{}", TEXT);
    }
    std::process::exit(status);
}

// original: main (htslib/htsfile.c:227)
//
// Argument vector is taken as owned, NUL-free byte strings (argv[0] included).
pub unsafe fn htsfile_c_227_main(argv: Vec<Vec<u8>>) -> i32 {
    STATUS = 0;

    // Inline long-option-aware argument parsing (replacing getopt_long). Short
    // options accepted: c C h H v; long options as listed below. Option parsing
    // stops at the first non-option argument or "--".
    let mut operands: Vec<CFilename> = Vec::new();
    let mut idx = 1; // skip program name
    let mut no_more_options = false;
    while idx < argv.len() {
        let arg = &argv[idx];
        if no_more_options || arg.is_empty() || arg[0] != b'-' || arg.as_slice() == b"-" {
            // Non-option operand.
            operands.push(CFilename::from_bytes(arg.clone()));
            idx += 1;
            continue;
        }
        if arg.as_slice() == b"--" {
            no_more_options = true;
            idx += 1;
            continue;
        }

        // Collect the option letters this argument contributes. A long option
        // ("--name") maps to its equivalent short letter or pseudo-code.
        let mut letters: Vec<i32> = Vec::new();
        if arg.len() >= 2 && arg[1] == b'-' {
            let name = &arg[2..];
            match name {
                b"copy" => letters.push(b'C' as i32),
                b"header-only" => letters.push(b'h' as i32),
                b"no-header" => letters.push(b'H' as i32),
                b"view" => letters.push(b'c' as i32),
                b"verbose" => letters.push(b'v' as i32),
                b"help" => letters.push(2),
                b"version" => letters.push(1),
                _ => letters.push(-1),
            }
        } else {
            for &byte in &arg[1..] {
                letters.push(byte as i32);
            }
        }

        for c in letters {
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
                    let version = std::ffi::CStr::from_ptr(hts::hts_version()).to_string_lossy();
                    println!(
                        "htsfile (htslib) {}\nCopyright (C) 2025 Genome Research Ltd.",
                        version
                    );
                    std::process::exit(0);
                }
                2 => htsfile_c_213_usage(true, 0),
                _ => htsfile_c_213_usage(false, 1),
            }
        }
        idx += 1;
    }

    if operands.is_empty() {
        htsfile_c_213_usage(false, 1);
    }

    if MODE == COPY {
        if operands.len() != 2 {
            htsfile_c_213_usage(false, 1);
        }
        htsfile_c_169_copy_raw(&operands[0], &operands[1]);
        return STATUS;
    }

    for filename in &operands {
        let Some(fp) = HFileHandle::open(filename.as_bytes_with_nul(), MODE_READ) else {
            error!(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                format!("can't open \"{}\"", filename.display())
            );
            continue;
        };

        if MODE == IDENTIFY {
            let mut fmt: hts::htsFormat = std::mem::zeroed();
            if hts::hts_detect_format2(
                fp.as_ptr(),
                filename.as_bytes_with_nul().as_ptr().cast(),
                &mut fmt,
            ) < 0
            {
                error!(
                    std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                    format!("detecting \"{}\" format failed", filename.display())
                );
                continue;
            }

            let description = hts::hts_format_description(&fmt);
            let desc = std::ffi::CStr::from_ptr(description).to_string_lossy();
            println!("{}:\t{}", filename.display(), desc);
            libc::free(description.cast());
        } else {
            match HtsFileHandle::from_hfile(fp, filename.as_bytes_with_nul(), MODE_READ) {
                Ok(mut hts_file) => {
                    match hts_file.as_ref().format.category {
                        hts::HTS_FORMAT_SEQUENCE_DATA => {
                            htsfile_c_64_view_sam(hts_file.as_mut(), filename)
                        }
                        hts::HTS_FORMAT_VARIANT_DATA => {
                            htsfile_c_108_view_vcf(hts_file.as_mut(), filename)
                        }
                        _ => {
                            if VERBOSE != 0 {
                                let fp = hts::hts_hfile(hts_file.as_ptr());
                                let Some(fp) = fp.as_mut() else {
                                    error!(
                                        libc::EINVAL,
                                        format!("reading \"{}\" failed", filename.display())
                                    );
                                    continue;
                                };
                                htsfile_c_152_view_raw(fp, filename);
                            } else {
                                error!(
                                    0,
                                    format!("can't view \"{}\": unknown format", filename.display())
                                );
                            }
                        }
                    }

                    if hts_file.close() < 0 {
                        error!(
                            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                            format!("closing \"{}\" failed", filename.display())
                        );
                    }
                    continue;
                }
                Err(mut fp) => {
                    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    if (errno == EFTYPE || errno == libc::ENOEXEC) && VERBOSE != 0 {
                        htsfile_c_152_view_raw(fp.as_mut(), filename);
                    } else {
                        error!(errno, format!("can't view \"{}\"", filename.display()));
                    }

                    if fp.close() < 0 {
                        error!(
                            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                            format!("closing \"{}\" failed", filename.display())
                        );
                    }
                    continue;
                }
            }
        }

        if fp.close() < 0 {
            error!(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
                format!("closing \"{}\" failed", filename.display())
            );
        }
    }

    // Flush standard output; report a genuine write failure (EBADF is benign).
    if std::io::stdout().flush().is_err() {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        if errno != libc::EBADF {
            error!(errno, "closing standard output failed".to_string());
        }
    }

    STATUS
}
