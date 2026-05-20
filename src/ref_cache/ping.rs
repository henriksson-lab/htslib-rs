use std::ffi::c_int;

// original: check_running (htslib/ref_cache/ping.c:39)
pub unsafe fn ref_cache_ping_c_39_check_running(port_num: c_int) -> c_int {
    let mut port = [0 as libc::c_char; 16];
    let mut reply = [0 as libc::c_char; 256];
    let request = b"GET /hello HTTP/1.0\r\n\r\n";
    let expected = b"HTTP/1.0 200 OK\r\n";
    let fails = [
        c"No addresses returned by getaddrinfo.".as_ptr(),
        c"Error getting socket: ".as_ptr(),
        c"Couldn't connect socket: ".as_ptr(),
    ];
    let len = expected.len();
    let mut hints: libc::addrinfo = std::mem::zeroed();
    let mut addrs: *mut libc::addrinfo = std::ptr::null_mut();
    let mut res: c_int;
    let mut s: c_int = -1;
    let mut where_: c_int = 0;
    let mut got_connection: c_int = 0;

    libc::snprintf(port.as_mut_ptr(), port.len(), c"%d".as_ptr(), port_num);
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_NUMERICSERV;
    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        hints.ai_flags |= libc::AI_V4MAPPED;
        hints.ai_flags |= libc::AI_ADDRCONFIG;
    }

    res = libc::getaddrinfo(std::ptr::null(), port.as_ptr(), &hints, &mut addrs);
    if res != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"getaddrinfo: %s\n".as_ptr(),
            libc::gai_strerror(res),
        );
        return -1;
    }

    let mut addr = addrs;
    while !addr.is_null() {
        where_ = 0;
        loop {
            s = libc::socket((*addr).ai_family, (*addr).ai_socktype, (*addr).ai_protocol);
            if !(s == -1 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
                break;
            }
        }
        if s == -1 {
            where_ = 1;
            addr = (*addr).ai_next;
            continue;
        }

        loop {
            res = libc::connect(s, (*addr).ai_addr, (*addr).ai_addrlen);
            if !(res == -1 && *crate::htslib_rs::c_compat::__errno_location() == libc::EINTR) {
                break;
            }
        }
        if res == 0 {
            got_connection = 1;
            break;
        }

        where_ = 2;
        libc::close(s);
        addr = (*addr).ai_next;
    }

    libc::freeaddrinfo(addrs);

    if got_connection == 0 {
        if where_ == 2 && *crate::htslib_rs::c_compat::__errno_location() == libc::ECONNREFUSED {
            return 0;
        }

        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"%s%s\n".as_ptr(),
            fails[where_ as usize],
            if where_ > 0 {
                libc::strerror(*crate::htslib_rs::c_compat::__errno_location())
            } else {
                c"".as_ptr()
            },
        );
        return -1;
    }

    let mut count = request.len();
    let mut ucbuf = request.as_ptr();
    let mut write_res: libc::ssize_t = 0;
    while count > 0 {
        loop {
            write_res = libc::write(s, ucbuf.cast(), count);
            if !(write_res < 0
                && (*crate::htslib_rs::c_compat::__errno_location() == libc::EINTR
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EAGAIN
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EWOULDBLOCK))
            {
                break;
            }
        }
        if write_res < 0 {
            break;
        }
        count -= write_res as usize;
        ucbuf = ucbuf.add(write_res as usize);
    }

    if write_res < 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Writing to socket: %s\n".as_ptr(),
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        libc::close(s);
        return -1;
    }

    let mut read_res: libc::ssize_t = 0;
    let mut bytes: libc::ssize_t = 0;
    let mut ucbuf = reply.as_mut_ptr();
    while (bytes as usize) < reply.len() {
        loop {
            read_res = libc::read(s, ucbuf.cast(), reply.len());
            if !(read_res < 0
                && (*crate::htslib_rs::c_compat::__errno_location() == libc::EINTR
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EAGAIN
                    || *crate::htslib_rs::c_compat::__errno_location() == libc::EWOULDBLOCK))
            {
                break;
            }
        }
        if read_res <= 0 {
            break;
        }
        bytes += read_res;
        ucbuf = ucbuf.add(read_res as usize);
    }

    libc::close(s);
    if read_res < 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Reading from socket: %s\n".as_ptr(),
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        return -1;
    }

    if (bytes as usize) < len
        || libc::memcmp(reply.as_ptr().cast(), expected.as_ptr().cast(), len) != 0
    {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Unexpected reply from server:\n%.*s\n".as_ptr(),
            bytes as c_int,
            reply.as_ptr(),
        );
        return -1;
    }

    1
}
