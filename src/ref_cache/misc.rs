use std::ffi::{c_char, c_int, c_uchar, c_void};

const fn build_hexvals() -> [i8; 256] {
    let mut vals = [-1; 256];
    let mut i = 0;
    while i < 10 {
        vals[b'0' as usize + i] = i as i8;
        i += 1;
    }
    i = 0;
    while i < 6 {
        vals[b'A' as usize + i] = (10 + i) as i8;
        vals[b'a' as usize + i] = (10 + i) as i8;
        i += 1;
    }
    vals
}

// original: hexvals (htslib/ref_cache/misc.c:29)
pub const ref_cache_misc_h_36_hexvals: [i8; 256] = build_hexvals();

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
                && (*crate::htslib_rs::c_compat::__errno_location() == libc::EINTR
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EAGAIN
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EWOULDBLOCK))
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
                && (*crate::htslib_rs::c_compat::__errno_location() == libc::EINTR
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EAGAIN
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EWOULDBLOCK))
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

pub fn ref_cache_misc_h_91_lim_strdup_bytes(input: &[u8], max_len: usize) -> Option<Vec<u8>> {
    if input.is_empty() {
        return None;
    }

    let mut out = Vec::with_capacity(input.len().min(max_len).saturating_add(1));
    if input.len() < max_len {
        out.extend_from_slice(input);
    } else {
        let prefix_len = max_len.saturating_sub(3);
        out.extend_from_slice(&input[..prefix_len.min(input.len())]);
        out.extend(std::iter::repeat_n(b'.', 3.min(max_len)));
    }
    out.push(0);
    Some(out)
}

// original: lim_strdup (htslib/ref_cache/misc.h:91)
pub unsafe fn ref_cache_misc_h_91_lim_strdup(
    str_: *const c_char,
    len: usize,
    max_len: usize,
) -> Option<Vec<u8>> {
    if len != 0 && str_.is_null() {
        return None;
    }
    ref_cache_misc_h_91_lim_strdup_bytes(
        std::slice::from_raw_parts(str_.cast::<u8>(), len),
        max_len,
    )
}

#[cfg(test)]
mod tests {
    use super::{ref_cache_misc_h_91_lim_strdup, ref_cache_misc_h_91_lim_strdup_bytes};

    #[test]
    fn lim_strdup_bytes_returns_none_for_empty_input() {
        assert_eq!(ref_cache_misc_h_91_lim_strdup_bytes(b"", 10), None);
    }

    #[test]
    fn lim_strdup_bytes_copies_short_input_with_nul_terminator() {
        assert_eq!(
            ref_cache_misc_h_91_lim_strdup_bytes(b"abcdef", 10).unwrap(),
            b"abcdef\0"
        );
    }

    #[test]
    fn lim_strdup_bytes_truncates_long_input_with_ellipsis_and_nul() {
        assert_eq!(
            ref_cache_misc_h_91_lim_strdup_bytes(b"abcdefgh", 6).unwrap(),
            b"abc...\0"
        );
    }

    #[test]
    fn lim_strdup_bytes_handles_tiny_max_len_without_underflow() {
        assert_eq!(
            ref_cache_misc_h_91_lim_strdup_bytes(b"abcdefgh", 2).unwrap(),
            b"..\0"
        );
    }

    #[test]
    fn lim_strdup_raw_input_adapter_borrows_into_owned_vec() {
        let input = b"abcdef";
        let out = unsafe { ref_cache_misc_h_91_lim_strdup(input.as_ptr().cast(), input.len(), 5) }
            .unwrap();
        assert_eq!(out, b"ab...\0");
    }
}
