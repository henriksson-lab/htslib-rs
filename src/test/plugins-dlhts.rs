/*  test/plugins-dlhts.c -- Test plugins with dynamically loaded libhts.

    Copyright (C) 2020 University of Glasgow.

    Author: John Marshall <John.W.Marshall@glasgow.ac.uk>

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

use crate::htslib_rs::hts::hFILE;

unsafe extern "C" {
    static mut optind: i32;
}

const EPROTONOSUPPORT: i32 = libc::EPROTONOSUPPORT;

type VoidFunc = unsafe extern "C" fn();
// __va_list_tag boundary: variadic hopen signature kept for transmute compatibility.
type HopenFunc = unsafe extern "C" fn(*const u8, *const u8, ...) -> *mut hFILE;
type HcloseAbruptlyFunc = unsafe extern "C" fn(*mut hFILE);
type CstrFunc = unsafe extern "C" fn() -> *const u8;

static mut TEST_PLUGINS_DLHTS_ERRORS: i32 = 0;
static mut TEST_PLUGINS_DLHTS_VERBOSE: i32 = 0;

static mut TEST_PLUGINS_DLHTS_HOPEN_P: *mut () = std::ptr::null_mut();
static mut TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P: *mut () = std::ptr::null_mut();

// original: sym (htslib/test/plugins-dlhts.c:48)
pub unsafe fn test_plugins_dlhts_c_48_sym(htslib: *mut (), name: &[u8]) -> *mut () {
    let ptr = libc::dlsym(htslib.cast(), name.as_ptr().cast());
    if ptr.is_null() {
        let err = std::ffi::CStr::from_ptr(libc::dlerror()).to_bytes();
        eprintln!(
            "Can't find symbol \"{}\": {}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(err)
        );
        std::process::exit(1);
    }
    ptr.cast()
}

// original: func (htslib/test/plugins-dlhts.c:59)
pub unsafe fn test_plugins_dlhts_c_59_func(htslib: *mut (), name: &[u8]) -> *mut () {
    test_plugins_dlhts_c_48_sym(htslib, name)
}

// original: test_hopen (htslib/test/plugins-dlhts.c:75)
pub unsafe fn test_plugins_dlhts_c_75_test_hopen(fname: &[u8], expected: i32) {
    let fp = (std::mem::transmute::<*mut (), HopenFunc>(TEST_PLUGINS_DLHTS_HOPEN_P))(
        fname.as_ptr(),
        b"r\0".as_ptr(),
    );
    if !fp.is_null() {
        (std::mem::transmute::<*mut (), HcloseAbruptlyFunc>(
            TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P,
        ))(fp);
        eprintln!(
            "Opening \"{}\" actually succeeded",
            String::from_utf8_lossy(fname)
        );
        TEST_PLUGINS_DLHTS_ERRORS += 1;
        return;
    }

    let errno = *libc::__errno_location();
    let supported = (errno != EPROTONOSUPPORT) as i32;
    if supported != expected {
        let strerr = std::ffi::CStr::from_ptr(libc::strerror(errno)).to_bytes();
        eprintln!(
            "Opening \"{}\" failed badly: {}",
            String::from_utf8_lossy(fname),
            String::from_utf8_lossy(strerr)
        );
        TEST_PLUGINS_DLHTS_ERRORS += 1;
    } else if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        let strerr = std::ffi::CStr::from_ptr(libc::strerror(errno)).to_bytes();
        println!(
            "Opening \"{}\" produces {}",
            String::from_utf8_lossy(fname),
            String::from_utf8_lossy(strerr)
        );
    }
}

// original: verbose_log (htslib/test/plugins-dlhts.c:94)
pub unsafe fn test_plugins_dlhts_c_94_verbose_log(message: &[u8]) {
    use std::io::Write;
    let _ = std::io::stderr().flush();
    if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        println!("{}", String::from_utf8_lossy(message));
    }
    let _ = std::io::stdout().flush();
}

// original: main (htslib/test/plugins-dlhts.c:101)
pub unsafe fn test_plugins_dlhts_c_101_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let mut dlflags = libc::RTLD_NOW;
    #[cfg(target_os = "macos")]
    let skip = ((dlflags & libc::RTLD_LOCAL) != 0) as i32;
    #[cfg(not(target_os = "macos"))]
    let skip = 0;

    loop {
        let c = libc::getopt(argc, argv.cast(), b"glv\0".as_ptr().cast());
        if c < 0 {
            break;
        }
        match c {
            c if c == b'g' as i32 => dlflags |= libc::RTLD_GLOBAL,
            c if c == b'l' as i32 => dlflags |= libc::RTLD_LOCAL,
            c if c == b'v' as i32 => TEST_PLUGINS_DLHTS_VERBOSE += 1,
            _ => {}
        }
    }

    if optind >= argc {
        eprintln!("Usage: plugins-dlhts [-glv] LIBHTSFILE");
        return libc::EXIT_FAILURE;
    }

    let htslib = libc::dlopen((*argv.add(optind as usize)).cast(), dlflags);
    if htslib.is_null() {
        let argname = std::ffi::CStr::from_ptr((*argv.add(optind as usize)).cast()).to_bytes();
        let err = std::ffi::CStr::from_ptr(libc::dlerror()).to_bytes();
        eprintln!(
            "Can't dlopen \"{}\": {}",
            String::from_utf8_lossy(argname),
            String::from_utf8_lossy(err)
        );
        return libc::EXIT_FAILURE;
    }

    if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        let hts_verbosep =
            test_plugins_dlhts_c_48_sym(htslib.cast(), b"hts_verbose\0").cast::<i32>();
        *hts_verbosep += TEST_PLUGINS_DLHTS_VERBOSE;

        let version = (std::mem::transmute::<*mut (), CstrFunc>(test_plugins_dlhts_c_59_func(
            htslib.cast(),
            b"hts_version\0",
        )))();
        let version = std::ffi::CStr::from_ptr(version.cast()).to_bytes();
        println!("Loaded HTSlib {}", String::from_utf8_lossy(version));
    }

    TEST_PLUGINS_DLHTS_HOPEN_P = test_plugins_dlhts_c_59_func(htslib.cast(), b"hopen\0");
    TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P =
        test_plugins_dlhts_c_59_func(htslib.cast(), b"hclose_abruptly\0");

    test_plugins_dlhts_c_75_test_hopen(b"bad-scheme:unsupported\0", 0);

    if skip == 0 {
        #[cfg(feature = "libcurl")]
        test_plugins_dlhts_c_75_test_hopen(b"https://localhost:99999/invalid_port\0", 1);
        #[cfg(feature = "gcs")]
        test_plugins_dlhts_c_75_test_hopen(b"gs:invalid\0", 1);
        #[cfg(feature = "s3")]
        test_plugins_dlhts_c_75_test_hopen(b"s3:invalid\0", 1);
    } else {
        test_plugins_dlhts_c_94_verbose_log(b"Skipping most tests");
    }

    test_plugins_dlhts_c_94_verbose_log(b"Calling hts_lib_shutdown()");
    (std::mem::transmute::<*mut (), VoidFunc>(test_plugins_dlhts_c_59_func(
        htslib.cast(),
        b"hts_lib_shutdown\0",
    )))();

    test_plugins_dlhts_c_94_verbose_log(b"Calling dlclose(htslib)");
    if libc::dlclose(htslib) < 0 {
        let argname = std::ffi::CStr::from_ptr((*argv.add(optind as usize)).cast()).to_bytes();
        let err = std::ffi::CStr::from_ptr(libc::dlerror()).to_bytes();
        eprintln!(
            "Can't dlclose \"{}\": {}",
            String::from_utf8_lossy(argname),
            String::from_utf8_lossy(err)
        );
        TEST_PLUGINS_DLHTS_ERRORS += 1;
    }

    test_plugins_dlhts_c_94_verbose_log(b"Returning from main()");

    if TEST_PLUGINS_DLHTS_ERRORS > 0 {
        println!("FAILED: {} errors", TEST_PLUGINS_DLHTS_ERRORS);
        return libc::EXIT_FAILURE;
    }

    if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        println!("All tests passed");
    }
    libc::EXIT_SUCCESS
}

// original: main (htslib/test/plugins-dlhts.c:180)
pub unsafe fn test_plugins_dlhts_c_180_main() -> i32 {
    println!("Tests skipped due to plugins being disabled");
    libc::EXIT_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    unsafe extern "C" {
        #[link_name = "hopen"]
        fn linked_hopen(fname: *const u8, mode: *const u8, ...) -> *mut hFILE;
        #[link_name = "hclose_abruptly"]
        fn linked_hclose_abruptly(fp: *mut hFILE);
    }

    fn plugins_dlhts_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    unsafe fn reset_state() {
        TEST_PLUGINS_DLHTS_ERRORS = 0;
        TEST_PLUGINS_DLHTS_VERBOSE = 0;
        TEST_PLUGINS_DLHTS_HOPEN_P = linked_hopen as *mut ();
        TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P = linked_hclose_abruptly as *mut ();
        // optind = 0 forces glibc getopt full reinit (shared-process tests).
        optind = 0;
    }

    #[test]
    fn plugins_dlhts_test_hopen_distinguishes_scheme_support_without_network() {
        let _guard = plugins_dlhts_test_lock();
        unsafe {
            reset_state();

            test_plugins_dlhts_c_75_test_hopen(b"bad-scheme:unsupported\0", 0);
            let errors = TEST_PLUGINS_DLHTS_ERRORS;
            assert_eq!(errors, 0);

            let mut missing =
                format!("/tmp/plugins-dlhts-missing-{}", std::process::id()).into_bytes();
            missing.push(0);
            test_plugins_dlhts_c_75_test_hopen(&missing, 1);
            let errors = TEST_PLUGINS_DLHTS_ERRORS;
            assert_eq!(errors, 0);
        }
    }

    #[test]
    fn plugins_dlhts_main_reports_usage_without_library_argument() {
        let _guard = plugins_dlhts_test_lock();
        unsafe {
            reset_state();
            let mut argv = [b"plugins-dlhts\0".as_ptr().cast_mut()];

            let rc = test_plugins_dlhts_c_101_main(argv.len() as i32, argv.as_mut_ptr());

            assert_eq!(rc, libc::EXIT_FAILURE);
            let errors = TEST_PLUGINS_DLHTS_ERRORS;
            let next_arg = optind;
            assert_eq!(errors, 0);
            assert_eq!(next_arg, 1);
        }
    }

    #[test]
    fn plugins_dlhts_disabled_main_matches_original_success_skip() {
        let _guard = plugins_dlhts_test_lock();
        unsafe {
            reset_state();

            let rc = test_plugins_dlhts_c_180_main();

            assert_eq!(rc, libc::EXIT_SUCCESS);
            let errors = TEST_PLUGINS_DLHTS_ERRORS;
            assert_eq!(errors, 0);
        }
    }
}
