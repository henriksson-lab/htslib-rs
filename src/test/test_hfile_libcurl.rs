/*  test/test_hfile_libcurl.c -- Test cases for libcurl retry/resilience.

    Copyright (C) 2026 Broad Institute.

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

use std::ffi::{c_char, c_int, c_uchar};

const TEST_DATA_FILE: *const c_char = c"hfile_libcurl.tmp".as_ptr();
const TEST_DATA_SIZE: usize = 16384;

static mut FAILURES: c_int = 0;

#[cfg(target_os = "windows")]
// original: main (htslib/test/test_hfile_libcurl.c:30)
pub unsafe fn test_test_hfile_libcurl_c_30_main() -> c_int {
    libc::fprintf(
        hts_sys::stderr.cast(),
        c"libcurl retry tests not supported on Windows, skipping\n".as_ptr(),
    );
    0
}

#[cfg(all(
    not(target_os = "windows"),
    any(
        target_os = "android",
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "hurd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    )
))]
const EPROTONOSUPPORT: c_int = libc::EPROTONOSUPPORT;

#[cfg(all(
    not(target_os = "windows"),
    not(any(
        target_os = "android",
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "hurd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))
))]
const EPROTONOSUPPORT: c_int = libc::ENOSYS;

#[cfg(not(target_os = "windows"))]
macro_rules! pass {
    ($name:expr) => {{
        libc::fprintf(hts_sys::stderr.cast(), c"  PASS: %s\n".as_ptr(), $name);
    }};
}

#[cfg(not(target_os = "windows"))]
macro_rules! fail {
    ($name:expr, $format:expr $(, $arg:expr)* $(,)?) => {{
        libc::fprintf(hts_sys::stderr.cast(), c"  FAIL: %s: ".as_ptr(), $name);
        libc::fprintf(hts_sys::stderr.cast(), $format $(, $arg)*);
        libc::fprintf(hts_sys::stderr.cast(), c"\n".as_ptr());
        FAILURES += 1;
    }};
}

#[cfg(not(target_os = "windows"))]
// original: start_server (htslib/test/test_hfile_libcurl.c:65)
pub unsafe fn test_test_hfile_libcurl_c_65_start_server(
    mode: *const c_char,
    fail_count: c_int,
    port_out: *mut c_int,
) -> libc::pid_t {
    let mut pipefd = [0 as c_int; 2];
    let mut fail_count_str = [0 as c_char; 32];
    let mut port_buf = [0 as c_char; 32];
    let mut n: isize;
    let mut i: c_int;

    if libc::pipe(pipefd.as_mut_ptr()) < 0 {
        libc::perror(c"pipe".as_ptr());
        return -1;
    }

    libc::snprintf(
        fail_count_str.as_mut_ptr(),
        fail_count_str.len(),
        c"%d".as_ptr(),
        fail_count,
    );

    let pid = libc::fork();
    if pid < 0 {
        libc::perror(c"fork".as_ptr());
        libc::close(pipefd[0]);
        libc::close(pipefd[1]);
        return -1;
    }

    if pid == 0 {
        libc::close(pipefd[0]);
        libc::dup2(pipefd[1], libc::STDOUT_FILENO);
        libc::close(pipefd[1]);
        libc::execlp(
            c"python3".as_ptr(),
            c"python3".as_ptr(),
            c"test/mock_http_server.py".as_ptr(),
            c"--mode".as_ptr(),
            mode,
            c"--file".as_ptr(),
            c"test/hfile_libcurl.tmp".as_ptr(),
            c"--fail-count".as_ptr(),
            fail_count_str.as_ptr(),
            c"--port".as_ptr(),
            c"0".as_ptr(),
            std::ptr::null::<c_char>(),
        );
        libc::perror(c"execlp python3".as_ptr());
        libc::_exit(127);
    }

    libc::close(pipefd[1]);

    n = 0;
    i = 0;
    while i < 50 && n == 0 {
        n = libc::read(pipefd[0], port_buf.as_mut_ptr().cast(), port_buf.len() - 1);
        if n <= 0 {
            crate::htslib_rs::hts::hts_usleep(100000);
            n = 0;
        }
        i += 1;
    }
    libc::close(pipefd[0]);

    if n <= 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Failed to read port from mock server\n".as_ptr(),
        );
        libc::kill(pid, libc::SIGTERM);
        libc::waitpid(pid, std::ptr::null_mut(), 0);
        return -1;
    }

    port_buf[n as usize] = 0;
    *port_out = libc::atoi(port_buf.as_ptr());
    if *port_out <= 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Invalid port from mock server: %s\n".as_ptr(),
            port_buf.as_ptr(),
        );
        libc::kill(pid, libc::SIGTERM);
        libc::waitpid(pid, std::ptr::null_mut(), 0);
        return -1;
    }

    crate::htslib_rs::hts::hts_usleep(100000);
    pid
}

#[cfg(not(target_os = "windows"))]
// original: stop_server (htslib/test/test_hfile_libcurl.c:139)
pub unsafe fn test_test_hfile_libcurl_c_139_stop_server(pid: libc::pid_t) {
    let mut status = 0;
    let ret: libc::pid_t;

    if pid <= 0 {
        return;
    }

    libc::kill(pid, libc::SIGKILL);
    ret = libc::waitpid(pid, &mut status, 0);
    if ret < 0 {
        libc::perror(c"waitpid".as_ptr());
    }
}

#[cfg(not(target_os = "windows"))]
// original: generate_test_data (htslib/test/test_hfile_libcurl.c:154)
pub unsafe fn test_test_hfile_libcurl_c_154_generate_test_data() {
    let mut path = [0 as c_char; 256];

    libc::snprintf(
        path.as_mut_ptr(),
        path.len(),
        c"test/%s".as_ptr(),
        TEST_DATA_FILE,
    );
    let f = libc::fopen(path.as_ptr(), c"wb".as_ptr());
    if f.is_null() {
        libc::perror(c"fopen test data".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }
    for i in 0..TEST_DATA_SIZE {
        libc::fputc(((i * 7 + 13) & 0xff) as c_int, f);
    }
    libc::fclose(f);
}

#[cfg(not(target_os = "windows"))]
// original: read_expected_data (htslib/test/test_hfile_libcurl.c:173)
pub unsafe fn test_test_hfile_libcurl_c_173_read_expected_data(
    size_out: *mut usize,
) -> *mut c_uchar {
    let mut path = [0 as c_char; 256];

    libc::snprintf(
        path.as_mut_ptr(),
        path.len(),
        c"test/%s".as_ptr(),
        TEST_DATA_FILE,
    );
    let f = libc::fopen(path.as_ptr(), c"rb".as_ptr());
    if f.is_null() {
        libc::perror(c"fopen expected data".as_ptr());
        return std::ptr::null_mut();
    }
    let data = libc::malloc(TEST_DATA_SIZE).cast::<c_uchar>();
    if data.is_null() {
        libc::fclose(f);
        return std::ptr::null_mut();
    }
    *size_out = libc::fread(data.cast(), 1, TEST_DATA_SIZE, f);
    libc::fclose(f);
    data
}

#[cfg(not(target_os = "windows"))]
// original: hfile_read_all (htslib/test/test_hfile_libcurl.c:196)
pub unsafe fn test_test_hfile_libcurl_c_196_hfile_read_all(
    url: *const c_char,
    size_out: *mut usize,
) -> *mut c_uchar {
    let fp = crate::htslib_rs::hfile::hopen(url, c"r".as_ptr());
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    let buf = libc::malloc(TEST_DATA_SIZE + 1024).cast::<c_uchar>();
    if buf.is_null() {
        crate::htslib_rs::hfile::hclose_abruptly(fp);
        return std::ptr::null_mut();
    }

    let mut total = 0usize;
    let mut n: isize;
    loop {
        n = crate::htslib_rs::hfile::htslib_hfile_h_247_hread(fp, buf.add(total).cast(), 4096);
        if n <= 0 {
            break;
        }
        total += n as usize;
        if total > TEST_DATA_SIZE + 512 {
            break;
        }
    }

    if n < 0 {
        libc::free(buf.cast());
        crate::htslib_rs::hfile::hclose_abruptly(fp);
        *size_out = 0;
        return std::ptr::null_mut();
    }

    if crate::htslib_rs::hfile::hclose(fp) != 0 {
        libc::perror(c"hclose".as_ptr());
    }
    *size_out = total;
    buf
}

#[cfg(not(target_os = "windows"))]
// original: test_normal_transfer (htslib/test/test_hfile_libcurl.c:233)
pub unsafe fn test_test_hfile_libcurl_c_233_test_normal_transfer() {
    let name = c"Normal transfer".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(c"normal".as_ptr(), 0, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    let got = test_test_hfile_libcurl_c_196_hfile_read_all(url.as_ptr(), &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_null() {
        fail!(
            name,
            c"hfile_read_all returned NULL, errno=%d".as_ptr(),
            *crate::htslib_rs::c_compat::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(
            name,
            c"size mismatch: got %zu, expected %zu".as_ptr(),
            got_size,
            exp_size
        );
    } else if libc::memcmp(got.cast(), expected.cast(), exp_size) != 0 {
        fail!(name, c"data mismatch".as_ptr());
    } else {
        pass!(name);
    }

    libc::free(got.cast());
    libc::free(expected.cast());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_503_retry (htslib/test/test_hfile_libcurl.c:265)
pub unsafe fn test_test_hfile_libcurl_c_265_test_503_retry() {
    let name = c"503 retry succeeds".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(c"503_then_ok".as_ptr(), 2, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"3".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let got = test_test_hfile_libcurl_c_196_hfile_read_all(url.as_ptr(), &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_null() {
        fail!(
            name,
            c"hfile_read_all returned NULL, errno=%d".as_ptr(),
            *crate::htslib_rs::c_compat::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(
            name,
            c"size mismatch: got %zu, expected %zu".as_ptr(),
            got_size,
            exp_size
        );
    } else if libc::memcmp(got.cast(), expected.cast(), exp_size) != 0 {
        fail!(name, c"data mismatch".as_ptr());
    } else {
        pass!(name);
    }

    libc::free(got.cast());
    libc::free(expected.cast());
    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_429_retry (htslib/test/test_hfile_libcurl.c:302)
pub unsafe fn test_test_hfile_libcurl_c_302_test_429_retry() {
    let name = c"429 retry succeeds".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(c"429_then_ok".as_ptr(), 2, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"3".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let got = test_test_hfile_libcurl_c_196_hfile_read_all(url.as_ptr(), &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_null() {
        fail!(
            name,
            c"hfile_read_all returned NULL, errno=%d".as_ptr(),
            *crate::htslib_rs::c_compat::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(
            name,
            c"size mismatch: got %zu, expected %zu".as_ptr(),
            got_size,
            exp_size
        );
    } else if libc::memcmp(got.cast(), expected.cast(), exp_size) != 0 {
        fail!(name, c"data mismatch".as_ptr());
    } else {
        pass!(name);
    }

    libc::free(got.cast());
    libc::free(expected.cast());
    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_drop_mid_transfer (htslib/test/test_hfile_libcurl.c:339)
pub unsafe fn test_test_hfile_libcurl_c_339_test_drop_mid_transfer() {
    let name = c"Connection drop retry".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid =
        test_test_hfile_libcurl_c_65_start_server(c"drop_mid_transfer".as_ptr(), 2, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"5".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let got = test_test_hfile_libcurl_c_196_hfile_read_all(url.as_ptr(), &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_null() {
        fail!(
            name,
            c"hfile_read_all returned NULL, errno=%d".as_ptr(),
            *crate::htslib_rs::c_compat::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(
            name,
            c"size mismatch: got %zu, expected %zu".as_ptr(),
            got_size,
            exp_size
        );
    } else if libc::memcmp(got.cast(), expected.cast(), exp_size) != 0 {
        fail!(name, c"data mismatch".as_ptr());
    } else {
        pass!(name);
    }

    libc::free(got.cast());
    libc::free(expected.cast());
    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_404_no_retry (htslib/test/test_hfile_libcurl.c:376)
pub unsafe fn test_test_hfile_libcurl_c_376_test_404_no_retry() {
    let name = c"404 not retried".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];

    let pid = test_test_hfile_libcurl_c_65_start_server(c"404".as_ptr(), 0, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"3".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr(), c"r".as_ptr());
    let errno = *crate::htslib_rs::c_compat::__errno_location();
    if !fp.is_null() {
        fail!(name, c"hopen should have failed for 404".as_ptr());
        crate::htslib_rs::hfile::hclose_abruptly(fp);
    } else if errno != libc::ENOENT {
        fail!(
            name,
            c"expected ENOENT, got errno=%d (%s)".as_ptr(),
            errno,
            libc::strerror(errno)
        );
    } else {
        pass!(name);
    }

    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_retry_exhaustion (htslib/test/test_hfile_libcurl.c:407)
pub unsafe fn test_test_hfile_libcurl_c_407_test_retry_exhaustion() {
    let name = c"Retry exhaustion".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];

    let pid = test_test_hfile_libcurl_c_65_start_server(c"503_then_ok".as_ptr(), 999, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"2".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr(), c"r".as_ptr());
    if !fp.is_null() {
        fail!(
            name,
            c"hopen should have failed after retry exhaustion".as_ptr()
        );
        crate::htslib_rs::hfile::hclose_abruptly(fp);
    } else {
        pass!(name);
    }

    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_retry_disabled (htslib/test/test_hfile_libcurl.c:436)
pub unsafe fn test_test_hfile_libcurl_c_436_test_retry_disabled() {
    let name = c"Retry disabled".as_ptr();
    let mut port = 0;
    let mut url = [0 as c_char; 256];

    let pid = test_test_hfile_libcurl_c_65_start_server(c"503_then_ok".as_ptr(), 1, &mut port);
    if pid < 0 {
        fail!(name, c"could not start server".as_ptr());
        return;
    }

    libc::snprintf(
        url.as_mut_ptr(),
        url.len(),
        c"http://127.0.0.1:%d/data".as_ptr(),
        port,
    );
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"0".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr(), c"r".as_ptr());
    if !fp.is_null() {
        fail!(
            name,
            c"hopen should have failed with retries disabled".as_ptr()
        );
        crate::htslib_rs::hfile::hclose_abruptly(fp);
    } else {
        pass!(name);
    }

    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: main (htslib/test/test_hfile_libcurl.c:464)
pub unsafe fn test_test_hfile_libcurl_c_464_main() -> c_int {
    if libc::system(c"python3 --version >/dev/null 2>&1".as_ptr()) != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"python3 not found, skipping libcurl retry tests\n".as_ptr(),
        );
        return 0;
    }

    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"0".as_ptr(), 1);
    let probe = crate::htslib_rs::hfile::hopen(c"http://0.0.0.0:1/probe".as_ptr(), c"r".as_ptr());
    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    if !probe.is_null() {
        crate::htslib_rs::hfile::hclose_abruptly(probe);
    } else if *crate::htslib_rs::c_compat::__errno_location() == EPROTONOSUPPORT {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"HTTP not supported, skipping libcurl retry tests\n".as_ptr(),
        );
        return 0;
    }

    test_test_hfile_libcurl_c_154_generate_test_data();

    libc::fprintf(hts_sys::stderr.cast(), c"test_hfile_libcurl:\n".as_ptr());

    test_test_hfile_libcurl_c_233_test_normal_transfer();
    test_test_hfile_libcurl_c_265_test_503_retry();
    test_test_hfile_libcurl_c_302_test_429_retry();
    test_test_hfile_libcurl_c_339_test_drop_mid_transfer();
    test_test_hfile_libcurl_c_376_test_404_no_retry();
    test_test_hfile_libcurl_c_407_test_retry_exhaustion();
    test_test_hfile_libcurl_c_436_test_retry_disabled();

    libc::unlink(c"test/hfile_libcurl.tmp".as_ptr());

    if FAILURES > 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"%d test(s) FAILED\n".as_ptr(),
            FAILURES,
        );
        return libc::EXIT_FAILURE;
    }

    libc::fprintf(hts_sys::stderr.cast(), c"All tests passed.\n".as_ptr());
    libc::EXIT_SUCCESS
}
