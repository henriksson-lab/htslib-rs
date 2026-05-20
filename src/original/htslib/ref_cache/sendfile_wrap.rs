use std::ffi::c_int;

// original: sendfile_wrap (htslib/ref_cache/sendfile_wrap.c:55)
#[cfg(target_os = "linux")]
pub unsafe fn ref_cache_sendfile_wrap_c_55_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    libc::sendfile(out_fd, in_fd, offset, count)
}

// original: sendfile_wrap (htslib/ref_cache/sendfile_wrap.c:61)
#[cfg(target_os = "freebsd")]
pub unsafe fn ref_cache_sendfile_wrap_c_61_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    let mut sbytes: libc::off_t = 0;
    let res: c_int;

    if count == 0 {
        return 0;
    }

    res = libc::sendfile(
        in_fd,
        out_fd,
        *offset,
        count,
        std::ptr::null_mut(),
        &mut sbytes,
        0,
    );
    *offset += sbytes;
    if res < 0 {
        res as libc::ssize_t
    } else {
        sbytes as libc::ssize_t
    }
}

// original: sendfile_wrap (htslib/ref_cache/sendfile_wrap.c:73)
#[cfg(target_os = "macos")]
pub unsafe fn ref_cache_sendfile_wrap_c_73_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    let mut len = count as libc::off_t;
    let res: c_int;

    if len == 0 {
        return 0;
    }

    res = libc::sendfile(in_fd, out_fd, *offset, &mut len, std::ptr::null_mut(), 0);
    if res == 0 || *libc::__error() == libc::EINTR || *libc::__error() == libc::EAGAIN {
        *offset += len;
    }
    if res < 0 {
        res as libc::ssize_t
    } else {
        len as libc::ssize_t
    }
}

// This should never be called
// original: sendfile_wrap (htslib/ref_cache/sendfile_wrap.c:87)
#[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos")))]
pub unsafe fn ref_cache_sendfile_wrap_c_87_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    if out_fd >= 0 || in_fd >= 0 || !offset.is_null() || count != 0 {
        return -2;
    }
    -1
}
