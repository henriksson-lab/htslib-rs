use std::ffi::c_int;

pub unsafe fn ref_cache_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    platform_sendfile_wrap(out_fd, in_fd, offset, count)
}

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

#[cfg(target_os = "linux")]
unsafe fn platform_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    ref_cache_sendfile_wrap_c_55_sendfile_wrap(out_fd, in_fd, offset, count)
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

#[cfg(target_os = "freebsd")]
unsafe fn platform_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    ref_cache_sendfile_wrap_c_61_sendfile_wrap(out_fd, in_fd, offset, count)
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
    if res == 0
        || *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR
        || *crate::htslib_rs::c_compat::__errno_location() == libc::EAGAIN
    {
        *offset += len;
    }
    if res < 0 {
        res as libc::ssize_t
    } else {
        len as libc::ssize_t
    }
}

#[cfg(target_os = "macos")]
unsafe fn platform_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    ref_cache_sendfile_wrap_c_73_sendfile_wrap(out_fd, in_fd, offset, count)
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
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -2;
    }
    *crate::htslib_rs::c_compat::__errno_location() = crate::htslib_rs::c_compat::ENOSYS;
    -1
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos")))]
unsafe fn platform_sendfile_wrap(
    out_fd: c_int,
    in_fd: c_int,
    offset: *mut libc::off_t,
    count: usize,
) -> libc::ssize_t {
    ref_cache_sendfile_wrap_c_87_sendfile_wrap(out_fd, in_fd, offset, count)
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    mod linux {
        use super::super::ref_cache_sendfile_wrap;
        use std::fs::{self, File, OpenOptions};
        use std::io::{Read, Write};
        use std::os::fd::AsRawFd;

        #[test]
        fn common_wrapper_uses_linux_sendfile_and_updates_explicit_offset() {
            let dir = std::env::temp_dir();
            let pid = std::process::id();
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let input_path = dir.join(format!("htslib-rs-ref-cache-sendfile-in-{pid}-{nanos}"));
            let output_path = dir.join(format!("htslib-rs-ref-cache-sendfile-out-{pid}-{nanos}"));

            let mut input = File::create(&input_path).expect("create input");
            input.write_all(b"abcdef").expect("write input");
            input.sync_all().expect("sync input");
            drop(input);

            let input = File::open(&input_path).expect("open input");
            let output = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&output_path)
                .expect("open output");
            let mut offset: libc::off_t = 2;

            let copied = unsafe {
                ref_cache_sendfile_wrap(output.as_raw_fd(), input.as_raw_fd(), &mut offset, 0)
            };
            assert_eq!(copied, 0);
            assert_eq!(offset, 2);

            let copied = unsafe {
                ref_cache_sendfile_wrap(output.as_raw_fd(), input.as_raw_fd(), &mut offset, 4)
            };
            assert_eq!(copied, 4);
            assert_eq!(offset, 6);
            drop(output);

            let mut actual = Vec::new();
            File::open(&output_path)
                .expect("reopen output")
                .read_to_end(&mut actual)
                .expect("read output");
            assert_eq!(actual, b"cdef");

            fs::remove_file(input_path).expect("remove input");
            fs::remove_file(output_path).expect("remove output");
        }
    }
}
