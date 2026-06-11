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

const TEST_DATA_FILE: &[u8] = b"hfile_libcurl.tmp";
const TEST_DATA_SIZE: usize = 16384;

static mut FAILURES: i32 = 0;

#[cfg(target_os = "windows")]
// original: main (htslib/test/test_hfile_libcurl.c:30)
pub unsafe fn test_test_hfile_libcurl_c_30_main() -> i32 {
    eprintln!("libcurl retry tests not supported on Windows, skipping");
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
const EPROTONOSUPPORT: i32 = libc::EPROTONOSUPPORT;

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
const EPROTONOSUPPORT: i32 = libc::ENOSYS;

#[cfg(not(target_os = "windows"))]
macro_rules! pass {
    ($name:expr) => {{
        eprintln!("  PASS: {}", String::from_utf8_lossy($name));
    }};
}

#[cfg(not(target_os = "windows"))]
macro_rules! fail {
    ($name:expr, $format:expr $(, $arg:expr)* $(,)?) => {{
        eprint!("  FAIL: {}: ", String::from_utf8_lossy($name));
        eprintln!($format $(, $arg)*);
        FAILURES += 1;
    }};
}

#[cfg(not(target_os = "windows"))]
// original: start_server (htslib/test/test_hfile_libcurl.c:65)
pub unsafe fn test_test_hfile_libcurl_c_65_start_server(
    mode: &[u8],
    fail_count: i32,
    port_out: *mut i32,
) -> libc::pid_t {
    let mut pipefd = [0i32; 2];
    let mut fail_count_str = [0u8; 32];
    let mut port_buf = [0u8; 32];
    let mut n: isize = 0;
    let mut i: i32 = 0;

    if libc::pipe(pipefd.as_mut_ptr()) < 0 {
        libc::perror(c"pipe".as_ptr());
        return -1;
    }

    let fc = fail_count.to_string();
    let fc_len = fc.len().min(fail_count_str.len() - 1);
    fail_count_str[..fc_len].copy_from_slice(&fc.as_bytes()[..fc_len]);
    fail_count_str[fc_len] = 0;

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
            mode.as_ptr().cast::<std::ffi::c_char>(),
            c"--file".as_ptr(),
            c"test/hfile_libcurl.tmp".as_ptr(),
            c"--fail-count".as_ptr(),
            fail_count_str.as_ptr().cast::<std::ffi::c_char>(),
            c"--port".as_ptr(),
            c"0".as_ptr(),
            std::ptr::null::<std::ffi::c_char>(),
        );
        libc::perror(c"execlp python3".as_ptr());
        libc::_exit(127);
    }

    libc::close(pipefd[1]);

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
        eprintln!("Failed to read port from mock server");
        libc::kill(pid, libc::SIGTERM);
        libc::waitpid(pid, std::ptr::null_mut(), 0);
        return -1;
    }

    port_buf[n as usize] = 0;
    *port_out = std::str::from_utf8(&port_buf[..n as usize])
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);
    if *port_out <= 0 {
        eprintln!(
            "Invalid port from mock server: {}",
            String::from_utf8_lossy(&port_buf[..n as usize])
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

    if pid <= 0 {
        return;
    }

    libc::kill(pid, libc::SIGKILL);
    let ret = libc::waitpid(pid, &mut status, 0);
    if ret < 0 {
        libc::perror(c"waitpid".as_ptr());
    }
}

#[cfg(not(target_os = "windows"))]
// original: generate_test_data (htslib/test/test_hfile_libcurl.c:154)
pub unsafe fn test_test_hfile_libcurl_c_154_generate_test_data() {
    let mut path = [0u8; 256];

    let prefix = b"test/";
    path[..prefix.len()].copy_from_slice(prefix);
    path[prefix.len()..prefix.len() + TEST_DATA_FILE.len()].copy_from_slice(TEST_DATA_FILE);
    path[prefix.len() + TEST_DATA_FILE.len()] = 0;
    let f = libc::fopen(path.as_ptr().cast(), c"wb".as_ptr());
    if f.is_null() {
        libc::perror(c"fopen test data".as_ptr());
        libc::exit(libc::EXIT_FAILURE);
    }
    for i in 0..TEST_DATA_SIZE {
        libc::fputc(((i * 7 + 13) & 0xff) as i32, f);
    }
    libc::fclose(f);
}

#[cfg(not(target_os = "windows"))]
// original: read_expected_data (htslib/test/test_hfile_libcurl.c:173)
pub unsafe fn test_test_hfile_libcurl_c_173_read_expected_data(
    size_out: *mut usize,
) -> Option<Vec<u8>> {
    let mut path = [0u8; 256];

    let prefix = b"test/";
    path[..prefix.len()].copy_from_slice(prefix);
    path[prefix.len()..prefix.len() + TEST_DATA_FILE.len()].copy_from_slice(TEST_DATA_FILE);
    path[prefix.len() + TEST_DATA_FILE.len()] = 0;
    let f = libc::fopen(path.as_ptr().cast(), c"rb".as_ptr());
    if f.is_null() {
        libc::perror(c"fopen expected data".as_ptr());
        return None;
    }
    let mut data = vec![0u8; TEST_DATA_SIZE];
    let got = libc::fread(data.as_mut_ptr().cast(), 1, TEST_DATA_SIZE, f);
    *size_out = got;
    data.truncate(got);
    libc::fclose(f);
    Some(data)
}

#[cfg(not(target_os = "windows"))]
// original: hfile_read_all (htslib/test/test_hfile_libcurl.c:196)
pub unsafe fn test_test_hfile_libcurl_c_196_hfile_read_all(
    url: &[u8],
    size_out: *mut usize,
) -> Option<Vec<u8>> {
    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr().cast(), c"r".as_ptr());
    if fp.is_null() {
        return None;
    }

    let mut buf = vec![0u8; TEST_DATA_SIZE + 1024];

    let mut total = 0usize;
    let mut n: isize;
    loop {
        n = crate::htslib_rs::hfile::htslib_hfile_h_247_hread(
            fp,
            buf.as_mut_ptr().add(total).cast(),
            4096,
        );
        if n <= 0 {
            break;
        }
        total += n as usize;
        if total > TEST_DATA_SIZE + 512 {
            break;
        }
    }

    if n < 0 {
        crate::htslib_rs::hfile::hclose_abruptly(fp);
        *size_out = 0;
        return None;
    }

    if crate::htslib_rs::hfile::hclose(fp) != 0 {
        libc::perror(c"hclose".as_ptr());
    }
    buf.truncate(total);
    *size_out = total;
    Some(buf)
}

#[cfg(not(target_os = "windows"))]
// original: test_normal_transfer (htslib/test/test_hfile_libcurl.c:233)
pub unsafe fn test_test_hfile_libcurl_c_233_test_normal_transfer() {
    let name: &[u8] = b"Normal transfer";
    let mut port = 0;
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"normal\0", 0, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    let got = test_test_hfile_libcurl_c_196_hfile_read_all(&url, &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_none() {
        fail!(
            name,
            "hfile_read_all returned NULL, errno={}",
            *libc::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(name, "size mismatch: got {}, expected {}", got_size, exp_size);
    } else if got.as_deref() != expected.as_deref() {
        fail!(name, "data mismatch");
    } else {
        pass!(name);
    }

    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_503_retry (htslib/test/test_hfile_libcurl.c:265)
pub unsafe fn test_test_hfile_libcurl_c_265_test_503_retry() {
    let name: &[u8] = b"503 retry succeeds";
    let mut port = 0;
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"503_then_ok\0", 2, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"3".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let got = test_test_hfile_libcurl_c_196_hfile_read_all(&url, &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_none() {
        fail!(
            name,
            "hfile_read_all returned NULL, errno={}",
            *libc::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(name, "size mismatch: got {}, expected {}", got_size, exp_size);
    } else if got.as_deref() != expected.as_deref() {
        fail!(name, "data mismatch");
    } else {
        pass!(name);
    }

    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_429_retry (htslib/test/test_hfile_libcurl.c:302)
pub unsafe fn test_test_hfile_libcurl_c_302_test_429_retry() {
    let name: &[u8] = b"429 retry succeeds";
    let mut port = 0;
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"429_then_ok\0", 2, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"3".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let got = test_test_hfile_libcurl_c_196_hfile_read_all(&url, &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_none() {
        fail!(
            name,
            "hfile_read_all returned NULL, errno={}",
            *libc::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(name, "size mismatch: got {}, expected {}", got_size, exp_size);
    } else if got.as_deref() != expected.as_deref() {
        fail!(name, "data mismatch");
    } else {
        pass!(name);
    }

    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_drop_mid_transfer (htslib/test/test_hfile_libcurl.c:339)
pub unsafe fn test_test_hfile_libcurl_c_339_test_drop_mid_transfer() {
    let name: &[u8] = b"Connection drop retry";
    let mut port = 0;
    let mut got_size = 0usize;
    let mut exp_size = 0usize;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"drop_mid_transfer\0", 2, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"5".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let got = test_test_hfile_libcurl_c_196_hfile_read_all(&url, &mut got_size);
    let expected = test_test_hfile_libcurl_c_173_read_expected_data(&mut exp_size);

    if got.is_none() {
        fail!(
            name,
            "hfile_read_all returned NULL, errno={}",
            *libc::__errno_location()
        );
    } else if got_size != exp_size {
        fail!(name, "size mismatch: got {}, expected {}", got_size, exp_size);
    } else if got.as_deref() != expected.as_deref() {
        fail!(name, "data mismatch");
    } else {
        pass!(name);
    }

    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    libc::unsetenv(c"HTS_RETRY_DELAY".as_ptr());
    test_test_hfile_libcurl_c_139_stop_server(pid);
}

#[cfg(not(target_os = "windows"))]
// original: test_404_no_retry (htslib/test/test_hfile_libcurl.c:376)
pub unsafe fn test_test_hfile_libcurl_c_376_test_404_no_retry() {
    let name: &[u8] = b"404 not retried";
    let mut port = 0;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"404\0", 0, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"3".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr().cast(), c"r".as_ptr());
    let errno = *libc::__errno_location();
    if !fp.is_null() {
        fail!(name, "hopen should have failed for 404");
        crate::htslib_rs::hfile::hclose_abruptly(fp);
    } else if errno != libc::ENOENT {
        fail!(
            name,
            "expected ENOENT, got errno={} ({})",
            errno,
            std::ffi::CStr::from_ptr(libc::strerror(errno))
                .to_string_lossy()
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
    let name: &[u8] = b"Retry exhaustion";
    let mut port = 0;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"503_then_ok\0", 999, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"2".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr().cast(), c"r".as_ptr());
    if !fp.is_null() {
        fail!(name, "hopen should have failed after retry exhaustion");
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
    let name: &[u8] = b"Retry disabled";
    let mut port = 0;

    let pid = test_test_hfile_libcurl_c_65_start_server(b"503_then_ok\0", 1, &mut port);
    if pid < 0 {
        fail!(name, "could not start server");
        return;
    }

    let url = format!("http://127.0.0.1:{}/data\0", port).into_bytes();
    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"0".as_ptr(), 1);
    libc::setenv(c"HTS_RETRY_DELAY".as_ptr(), c"50".as_ptr(), 1);

    let fp = crate::htslib_rs::hfile::hopen(url.as_ptr().cast(), c"r".as_ptr());
    if !fp.is_null() {
        fail!(name, "hopen should have failed with retries disabled");
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
pub unsafe fn test_test_hfile_libcurl_c_464_main() -> i32 {
    if libc::system(c"python3 --version >/dev/null 2>&1".as_ptr()) != 0 {
        eprintln!("python3 not found, skipping libcurl retry tests");
        return 0;
    }

    libc::setenv(c"HTS_RETRY_MAX".as_ptr(), c"0".as_ptr(), 1);
    let probe = crate::htslib_rs::hfile::hopen(c"http://0.0.0.0:1/probe".as_ptr(), c"r".as_ptr());
    libc::unsetenv(c"HTS_RETRY_MAX".as_ptr());
    if !probe.is_null() {
        crate::htslib_rs::hfile::hclose_abruptly(probe);
    } else if *libc::__errno_location() == EPROTONOSUPPORT {
        eprintln!("HTTP not supported, skipping libcurl retry tests");
        return 0;
    }

    test_test_hfile_libcurl_c_154_generate_test_data();

    eprintln!("test_hfile_libcurl:");

    test_test_hfile_libcurl_c_233_test_normal_transfer();
    test_test_hfile_libcurl_c_265_test_503_retry();
    test_test_hfile_libcurl_c_302_test_429_retry();
    test_test_hfile_libcurl_c_339_test_drop_mid_transfer();
    test_test_hfile_libcurl_c_376_test_404_no_retry();
    test_test_hfile_libcurl_c_407_test_retry_exhaustion();
    test_test_hfile_libcurl_c_436_test_retry_disabled();

    libc::unlink(c"test/hfile_libcurl.tmp".as_ptr());

    if FAILURES > 0 {
        eprintln!("{} test(s) FAILED", FAILURES);
        return libc::EXIT_FAILURE;
    }

    eprintln!("All tests passed.");
    libc::EXIT_SUCCESS
}

#[cfg(not(target_os = "windows"))]
#[test]
fn translated_hfile_libcurl_retry_suite() {
    unsafe {
        if libc::access(c"test/mock_http_server.py".as_ptr(), libc::R_OK) != 0 {
            eprintln!("test/mock_http_server.py not found, skipping libcurl retry tests");
            return;
        }
        assert_eq!(test_test_hfile_libcurl_c_464_main(), libc::EXIT_SUCCESS);
    }
}
