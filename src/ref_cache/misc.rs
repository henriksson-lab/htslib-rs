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
pub fn ref_cache_misc_h_38_hexval(c: u8) -> i32 {
    ref_cache_misc_h_36_hexvals[c as usize] as i32
}

// original: setnonblock (htslib/ref_cache/misc.h:40)
pub unsafe fn ref_cache_misc_h_40_setnonblock(fd: i32) -> i32 {
    let val = libc::fcntl(fd, libc::F_GETFL);
    if val == -1 {
        eprintln!("Couldn't get file descriptor flags");
        return -1;
    }

    if libc::fcntl(fd, libc::F_SETFL, val | libc::O_NONBLOCK) != 0 {
        eprintln!("Couldn't set socket to non-blocking mode");
        return -1;
    }
    0
}

// original: do_write_all (htslib/ref_cache/misc.h:55)
pub unsafe fn ref_cache_misc_h_55_do_write_all(fd: i32, buf: &[u8]) -> isize {
    let mut res: isize = 0;
    let mut offset = 0usize;
    while offset < buf.len() {
        let count = buf.len() - offset;
        loop {
            res = libc::write(fd, buf[offset..].as_ptr().cast(), count);
            if !(res < 0
                && (*libc::__errno_location() == libc::EINTR
                    || *libc::__errno_location() == libc::EAGAIN
                    || *libc::__errno_location() == libc::EWOULDBLOCK))
            {
                break;
            }
        }
        if res < 0 {
            break;
        }
        offset += res as usize;
    }
    if res >= 0 {
        0
    } else {
        -1
    }
}

// original: do_read_all (htslib/ref_cache/misc.h:72)
pub unsafe fn ref_cache_misc_h_72_do_read_all(fd: i32, buf: &mut [u8]) -> isize {
    let mut res: isize = 0;
    let mut bytes: isize = 0;
    let count = buf.len();

    while (bytes as usize) < count {
        loop {
            res = libc::read(fd, buf[bytes as usize..].as_mut_ptr().cast(), count);
            if !(res < 0
                && (*libc::__errno_location() == libc::EINTR
                    || *libc::__errno_location() == libc::EAGAIN
                    || *libc::__errno_location() == libc::EWOULDBLOCK))
            {
                break;
            }
        }
        if res <= 0 {
            break;
        }
        bytes += res;
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
pub fn ref_cache_misc_h_91_lim_strdup(str_: &[u8], max_len: usize) -> Option<Vec<u8>> {
    ref_cache_misc_h_91_lim_strdup_bytes(str_, max_len)
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
        let out = ref_cache_misc_h_91_lim_strdup(input, 5).unwrap();
        assert_eq!(out, b"ab...\0");
    }
}
