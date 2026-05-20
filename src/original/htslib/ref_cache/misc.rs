use std::ffi::{c_char, c_int, c_uchar, c_void};

// original: hexvals (htslib/ref_cache/misc.h:36)
pub const ref_cache_misc_h_36_hexvals: [i8; 256] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, 10, 11, 12, 13, 14, 15, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10,
    11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];

// original: hexval (htslib/ref_cache/misc.h:38)
pub fn ref_cache_misc_h_38_hexval(c: c_char) -> c_int {
    ref_cache_misc_h_36_hexvals[c as c_uchar as usize] as c_int
}

// original: setnonblock (htslib/ref_cache/misc.h:40)
pub unsafe fn ref_cache_misc_h_40_setnonblock(fd: c_int) -> c_int {
    let val = libc::fcntl(fd, libc::F_GETFL);
    if val == -1 {
        libc::perror(c"Couldn't get file descriptor flags".as_ptr());
        return -1;
    }

    if libc::fcntl(fd, libc::F_SETFL, val | libc::O_NONBLOCK) != 0 {
        libc::perror(c"Couldn't set socket to non-blocking mode".as_ptr());
        return -1;
    }
    0
}

// original: do_write_all (htslib/ref_cache/misc.h:55)
pub unsafe fn ref_cache_misc_h_55_do_write_all(
    fd: c_int,
    buf: *const c_void,
    mut count: usize,
) -> libc::ssize_t {
    let mut res: libc::ssize_t = 0;
    let mut ucbuf = buf.cast::<c_uchar>();
    while count > 0 {
        loop {
            res = libc::write(fd, ucbuf.cast(), count);
            if !(res < 0
                && (*crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR
                    || *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EAGAIN
                    || *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EWOULDBLOCK))
            {
                break;
            }
        }
        if res < 0 {
            break;
        }
        count -= res as usize;
        ucbuf = ucbuf.add(res as usize);
    }
    if res >= 0 {
        0
    } else {
        -1
    }
}

// original: do_read_all (htslib/ref_cache/misc.h:72)
pub unsafe fn ref_cache_misc_h_72_do_read_all(
    fd: c_int,
    buf: *mut c_void,
    count: usize,
) -> libc::ssize_t {
    let mut res: libc::ssize_t = 0;
    let mut bytes: libc::ssize_t = 0;
    let mut ucbuf = buf.cast::<c_uchar>();

    while (bytes as usize) < count {
        loop {
            res = libc::read(fd, ucbuf.cast(), count);
            if !(res < 0
                && (*crate::htslib_mini_rs::c_compat::__errno_location() == libc::EINTR
                    || *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EAGAIN
                    || *crate::htslib_mini_rs::c_compat::__errno_location() == libc::EWOULDBLOCK))
            {
                break;
            }
        }
        if res <= 0 {
            break;
        }
        bytes += res;
        ucbuf = ucbuf.add(res as usize);
    }
    if res < 0 {
        res
    } else {
        bytes
    }
}

// original: lim_strdup (htslib/ref_cache/misc.h:91)
pub unsafe fn ref_cache_misc_h_91_lim_strdup(
    str_: *const c_char,
    len: usize,
    max_len: usize,
) -> *mut c_char {
    let out: *mut c_char;

    if len == 0 {
        return std::ptr::null_mut();
    }
    if len < max_len {
        out = libc::malloc(len + 1).cast();
        if out.is_null() {
            return std::ptr::null_mut();
        }
        libc::memcpy(out.cast(), str_.cast(), len);
        *out.add(len) = 0;
        return out;
    }
    out = libc::malloc(max_len + 1).cast();
    if out.is_null() {
        return std::ptr::null_mut();
    }
    libc::memcpy(out.cast(), str_.cast(), max_len - 3);
    libc::memcpy(out.add(max_len - 3).cast(), c"...".as_ptr().cast(), 4);
    out
}
