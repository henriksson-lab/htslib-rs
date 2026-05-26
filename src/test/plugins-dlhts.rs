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

use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::hts::hFILE;

unsafe extern "C" {
    static mut optind: c_int;
}

const EPROTONOSUPPORT: c_int = libc::EPROTONOSUPPORT;

type VoidFunc = unsafe extern "C" fn();
type HopenFunc = unsafe extern "C" fn(*const c_char, *const c_char, ...) -> *mut hFILE;
type HcloseAbruptlyFunc = unsafe extern "C" fn(*mut hFILE);
type CstrFunc = unsafe extern "C" fn() -> *const c_char;

static mut TEST_PLUGINS_DLHTS_ERRORS: c_int = 0;
static mut TEST_PLUGINS_DLHTS_VERBOSE: c_int = 0;

static mut TEST_PLUGINS_DLHTS_HOPEN_P: *mut c_void = std::ptr::null_mut();
static mut TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P: *mut c_void = std::ptr::null_mut();

// original: sym (htslib/test/plugins-dlhts.c:48)
pub unsafe fn test_plugins_dlhts_c_48_sym(htslib: *mut c_void, name: *const c_char) -> *mut c_void {
    let ptr = libc::dlsym(htslib, name);
    if ptr.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Can't find symbol \"%s\": %s\n".as_ptr(),
            name,
            libc::dlerror(),
        );
        libc::exit(libc::EXIT_FAILURE);
    }
    ptr
}

// original: func (htslib/test/plugins-dlhts.c:59)
pub unsafe fn test_plugins_dlhts_c_59_func(
    htslib: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    test_plugins_dlhts_c_48_sym(htslib, name)
}

// original: test_hopen (htslib/test/plugins-dlhts.c:75)
pub unsafe fn test_plugins_dlhts_c_75_test_hopen(fname: *const c_char, expected: c_int) {
    let fp = (std::mem::transmute::<*mut c_void, HopenFunc>(TEST_PLUGINS_DLHTS_HOPEN_P))(
        fname,
        c"r".as_ptr(),
    );
    if !fp.is_null() {
        (std::mem::transmute::<*mut c_void, HcloseAbruptlyFunc>(
            TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P,
        ))(fp);
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Opening \"%s\" actually succeeded\n".as_ptr(),
            fname,
        );
        TEST_PLUGINS_DLHTS_ERRORS += 1;
        return;
    }

    let errno = *crate::htslib_rs::c_compat::__errno_location();
    let supported = (errno != EPROTONOSUPPORT) as c_int;
    if supported != expected {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Opening \"%s\" failed badly: %s\n".as_ptr(),
            fname,
            libc::strerror(errno),
        );
        TEST_PLUGINS_DLHTS_ERRORS += 1;
    } else if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        libc::printf(
            c"Opening \"%s\" produces %s\n".as_ptr(),
            fname,
            libc::strerror(errno),
        );
    }
}

// original: verbose_log (htslib/test/plugins-dlhts.c:94)
pub unsafe fn test_plugins_dlhts_c_94_verbose_log(message: *const c_char) {
    libc::fflush(crate::htslib_rs::c_compat::stderr.cast());
    if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        libc::puts(message);
    }
    libc::fflush(crate::htslib_rs::c_compat::stdout.cast());
}

// original: main (htslib/test/plugins-dlhts.c:101)
pub unsafe fn test_plugins_dlhts_c_101_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut dlflags = libc::RTLD_NOW;
    #[cfg(target_os = "macos")]
    let skip = ((dlflags & libc::RTLD_LOCAL) != 0) as c_int;
    #[cfg(not(target_os = "macos"))]
    let skip = 0;

    loop {
        let c = libc::getopt(argc, argv, c"glv".as_ptr());
        if c < 0 {
            break;
        }
        match c {
            c if c == b'g' as c_int => dlflags |= libc::RTLD_GLOBAL,
            c if c == b'l' as c_int => dlflags |= libc::RTLD_LOCAL,
            c if c == b'v' as c_int => TEST_PLUGINS_DLHTS_VERBOSE += 1,
            _ => {}
        }
    }

    if optind >= argc {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Usage: plugins-dlhts [-glv] LIBHTSFILE\n".as_ptr(),
        );
        return libc::EXIT_FAILURE;
    }

    let htslib = libc::dlopen(*argv.add(optind as usize), dlflags);
    if htslib.is_null() {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Can't dlopen \"%s\": %s\n".as_ptr(),
            *argv.add(optind as usize),
            libc::dlerror(),
        );
        return libc::EXIT_FAILURE;
    }

    if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        let hts_verbosep =
            test_plugins_dlhts_c_48_sym(htslib, c"hts_verbose".as_ptr()).cast::<c_int>();
        *hts_verbosep += TEST_PLUGINS_DLHTS_VERBOSE;

        libc::printf(
            c"Loaded HTSlib %s\n".as_ptr(),
            (std::mem::transmute::<*mut c_void, CstrFunc>(test_plugins_dlhts_c_59_func(
                htslib,
                c"hts_version".as_ptr(),
            )))(),
        );
    }

    TEST_PLUGINS_DLHTS_HOPEN_P = test_plugins_dlhts_c_59_func(htslib, c"hopen".as_ptr());
    TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P =
        test_plugins_dlhts_c_59_func(htslib, c"hclose_abruptly".as_ptr());

    test_plugins_dlhts_c_75_test_hopen(c"bad-scheme:unsupported".as_ptr(), 0);

    if skip == 0 {
        #[cfg(feature = "libcurl")]
        test_plugins_dlhts_c_75_test_hopen(c"https://localhost:99999/invalid_port".as_ptr(), 1);
        #[cfg(feature = "gcs")]
        test_plugins_dlhts_c_75_test_hopen(c"gs:invalid".as_ptr(), 1);
        #[cfg(feature = "s3")]
        test_plugins_dlhts_c_75_test_hopen(c"s3:invalid".as_ptr(), 1);
    } else {
        test_plugins_dlhts_c_94_verbose_log(c"Skipping most tests".as_ptr());
    }

    test_plugins_dlhts_c_94_verbose_log(c"Calling hts_lib_shutdown()".as_ptr());
    (std::mem::transmute::<*mut c_void, VoidFunc>(test_plugins_dlhts_c_59_func(
        htslib,
        c"hts_lib_shutdown".as_ptr(),
    )))();

    test_plugins_dlhts_c_94_verbose_log(c"Calling dlclose(htslib)".as_ptr());
    if libc::dlclose(htslib) < 0 {
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"Can't dlclose \"%s\": %s\n".as_ptr(),
            *argv.add(optind as usize),
            libc::dlerror(),
        );
        TEST_PLUGINS_DLHTS_ERRORS += 1;
    }

    test_plugins_dlhts_c_94_verbose_log(c"Returning from main()".as_ptr());

    if TEST_PLUGINS_DLHTS_ERRORS > 0 {
        libc::printf(c"FAILED: %d errors\n".as_ptr(), TEST_PLUGINS_DLHTS_ERRORS);
        return libc::EXIT_FAILURE;
    }

    if TEST_PLUGINS_DLHTS_VERBOSE != 0 {
        libc::printf(c"All tests passed\n".as_ptr());
    }
    libc::EXIT_SUCCESS
}

// original: main (htslib/test/plugins-dlhts.c:180)
pub unsafe fn test_plugins_dlhts_c_180_main() -> c_int {
    libc::printf(c"Tests skipped due to plugins being disabled\n".as_ptr());
    libc::EXIT_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::sync::{Mutex, OnceLock};

    unsafe extern "C" {
        #[link_name = "hopen"]
        fn linked_hopen(fname: *const c_char, mode: *const c_char, ...) -> *mut hFILE;
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
        TEST_PLUGINS_DLHTS_HOPEN_P = linked_hopen as *mut c_void;
        TEST_PLUGINS_DLHTS_HCLOSE_ABRUPTLY_P = linked_hclose_abruptly as *mut c_void;
        optind = 1;
    }

    #[test]
    fn plugins_dlhts_test_hopen_distinguishes_scheme_support_without_network() {
        let _guard = plugins_dlhts_test_lock();
        unsafe {
            reset_state();

            test_plugins_dlhts_c_75_test_hopen(c"bad-scheme:unsupported".as_ptr(), 0);
            let errors = TEST_PLUGINS_DLHTS_ERRORS;
            assert_eq!(errors, 0);

            let missing =
                CString::new(format!("/tmp/plugins-dlhts-missing-{}", std::process::id())).unwrap();
            test_plugins_dlhts_c_75_test_hopen(missing.as_ptr(), 1);
            let errors = TEST_PLUGINS_DLHTS_ERRORS;
            assert_eq!(errors, 0);
        }
    }

    #[test]
    fn plugins_dlhts_main_reports_usage_without_library_argument() {
        let _guard = plugins_dlhts_test_lock();
        unsafe {
            reset_state();
            let mut argv = [c"plugins-dlhts".as_ptr().cast_mut()];

            let rc = test_plugins_dlhts_c_101_main(argv.len() as c_int, argv.as_mut_ptr());

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
