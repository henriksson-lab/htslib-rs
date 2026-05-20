use crate::original_stubs::{functions, structs};
use std::ffi::{c_char, c_int, c_void};

// original: Listeners (htslib/ref_cache/listener.c:46)
#[repr(C)]
pub struct Listeners {
    nsocks: usize,
    sockets: *mut c_int,
}

// original: open_socket (htslib/ref_cache/listener.c:51)
pub unsafe fn ref_cache_listener_c_51_open_socket(
    addr: *mut libc::addrinfo,
    cause: *mut *const c_char,
) -> c_int {
    let mut val: c_int = 1;

    let s = libc::socket((*addr).ai_family, (*addr).ai_socktype, (*addr).ai_protocol);
    if s == -1 {
        *cause = c"socket".as_ptr();
        return -1;
    }

    if libc::setsockopt(
        s,
        libc::SOL_SOCKET,
        libc::SO_REUSEADDR,
        (&mut val as *mut c_int).cast(),
        std::mem::size_of_val(&val) as libc::socklen_t,
    ) < 0
    {
        *cause = c"setsockopt".as_ptr();
        let serrno = *crate::htslib_mini_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_mini_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if (*addr).ai_family == libc::AF_INET6
        && libc::setsockopt(
            s,
            libc::IPPROTO_IPV6,
            libc::IPV6_V6ONLY,
            (&mut val as *mut c_int).cast(),
            std::mem::size_of_val(&val) as libc::socklen_t,
        ) < 0
    {
        *cause = c"setsockopt".as_ptr();
        let serrno = *crate::htslib_mini_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_mini_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if libc::bind(s, (*addr).ai_addr, (*addr).ai_addrlen) == -1 {
        *cause = c"bind".as_ptr();
        let serrno = *crate::htslib_mini_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_mini_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if functions::ref_cache_misc_h_40_setnonblock(s) != 0 {
        *cause = c"setnonblock".as_ptr();
        let serrno = *crate::htslib_mini_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_mini_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if libc::listen(s, libc::SOMAXCONN) != 0 {
        *cause = c"listen".as_ptr();
        let serrno = *crate::htslib_mini_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_mini_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    s
}

// original: get_listen_sockets (htslib/ref_cache/listener.c:95)
pub unsafe fn ref_cache_listener_c_95_get_listen_sockets(port: c_int) -> *mut Listeners {
    let lsocks = libc::calloc(1, std::mem::size_of::<Listeners>()).cast::<Listeners>();
    let mut hints: libc::addrinfo = std::mem::zeroed();
    let mut addr_list: *mut libc::addrinfo = std::ptr::null_mut();
    let mut pnum = [0 as c_char; 20];
    let mut cause: *const c_char = std::ptr::null();

    if lsocks.is_null() {
        return std::ptr::null_mut();
    }

    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE | libc::AI_NUMERICSERV;
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
    {
        hints.ai_flags |= libc::AI_ADDRCONFIG;
    }

    libc::snprintf(pnum.as_mut_ptr(), pnum.len(), c"%d".as_ptr(), port);
    let res = libc::getaddrinfo(std::ptr::null(), pnum.as_ptr(), &hints, &mut addr_list);
    if res != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"getaddrinfo failed: %s\n".as_ptr(),
            libc::gai_strerror(res),
        );
        libc::free(lsocks.cast());
        return std::ptr::null_mut();
    }

    let mut count = 0usize;
    let mut addr = addr_list;
    while !addr.is_null() {
        count += 1;
        addr = (*addr).ai_next;
    }

    if count == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"getaddrinfo returned nothing.\n".as_ptr(),
        );
        libc::free(lsocks.cast());
        return std::ptr::null_mut();
    }

    (*lsocks).sockets = libc::malloc(count * std::mem::size_of::<c_int>()).cast();
    if (*lsocks).sockets.is_null() {
        libc::perror(c"Allocating socket list".as_ptr());
        libc::freeaddrinfo(addr_list);
        libc::free(lsocks.cast());
        return std::ptr::null_mut();
    }

    addr = addr_list;
    while !addr.is_null() {
        assert!((*lsocks).nsocks < count);
        *(*lsocks).sockets.add((*lsocks).nsocks) =
            ref_cache_listener_c_51_open_socket(addr, &mut cause);
        if *(*lsocks).sockets.add((*lsocks).nsocks) != -1 {
            (*lsocks).nsocks += 1;
        }
        addr = (*addr).ai_next;
    }

    libc::freeaddrinfo(addr_list);

    if (*lsocks).nsocks == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Failure in %s while getting socket: %s\n".as_ptr(),
            cause,
            libc::strerror(*crate::htslib_mini_rs::c_compat::__errno_location()),
        );
        libc::free((*lsocks).sockets.cast());
        libc::free(lsocks.cast());
        return std::ptr::null_mut();
    }

    lsocks
}

// original: should_adopt_socket (htslib/ref_cache/listener.c:160)
pub unsafe fn ref_cache_listener_c_160_should_adopt_socket(fd: c_int) -> c_int {
    let mut statbuf: libc::stat = std::mem::zeroed();
    let mut sock_type: c_int = 0;
    let mut accepting: c_int = 0;
    let mut addr: libc::sockaddr = std::mem::zeroed();
    let mut len: libc::socklen_t;
    let mut addrlen: libc::socklen_t;

    if libc::fstat(fd, &mut statbuf) < 0 {
        return 0;
    }
    if !libc::S_ISSOCK(statbuf.st_mode) {
        return 0;
    }

    len = std::mem::size_of_val(&sock_type) as libc::socklen_t;
    if libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_TYPE,
        (&mut sock_type as *mut c_int).cast(),
        &mut len,
    ) < 0
        || len != std::mem::size_of_val(&sock_type) as libc::socklen_t
        || sock_type != libc::SOCK_STREAM
    {
        return 0;
    }

    libc::memset(
        (&mut addr as *mut libc::sockaddr).cast(),
        0,
        std::mem::size_of_val(&addr),
    );
    addrlen = std::mem::size_of_val(&addr) as libc::socklen_t;
    if libc::getsockname(fd, &mut addr, &mut addrlen) < 0
        || addrlen < std::mem::size_of::<libc::sa_family_t>() as libc::socklen_t
        || (addr.sa_family as c_int != libc::AF_INET && addr.sa_family as c_int != libc::AF_INET6)
    {
        return 0;
    }

    len = std::mem::size_of_val(&accepting) as libc::socklen_t;
    if libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_ACCEPTCONN,
        (&mut accepting as *mut c_int).cast(),
        &mut len,
    ) < 0
        || len != std::mem::size_of_val(&accepting) as libc::socklen_t
    {
        return 0;
    }
    if accepting == 0 && libc::listen(fd, libc::SOMAXCONN) != 0 {
        return 0;
    }

    if functions::ref_cache_misc_h_40_setnonblock(fd) != 0 {
        return 0;
    }

    1
}

// original: adopt_listen_sockets (htslib/ref_cache/listener.c:203)
pub unsafe fn ref_cache_listener_c_203_adopt_listen_sockets(
    min_sock_fd: c_int,
    num_fds: c_int,
) -> *mut Listeners {
    let lsocks = libc::calloc(1, std::mem::size_of::<Listeners>()).cast::<Listeners>();

    assert!(min_sock_fd > 0 && num_fds > 0 && num_fds < c_int::MAX - min_sock_fd);

    if lsocks.is_null() {
        return std::ptr::null_mut();
    }

    (*lsocks).sockets =
        libc::malloc(num_fds as usize * std::mem::size_of::<c_int>()).cast::<c_int>();
    if (*lsocks).sockets.is_null() {
        libc::perror(c"Allocating socket list".as_ptr());
        libc::free(lsocks.cast());
        return std::ptr::null_mut();
    }

    let mut fd = min_sock_fd;
    while fd < min_sock_fd + num_fds {
        if ref_cache_listener_c_160_should_adopt_socket(fd) != 0 {
            *(*lsocks).sockets.add((*lsocks).nsocks) = fd;
            (*lsocks).nsocks += 1;
        } else {
            libc::close(fd);
        }
        fd += 1;
    }

    if (*lsocks).nsocks == 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"No suitable sockets found\n".as_ptr(),
        );
        libc::free((*lsocks).sockets.cast());
        libc::free(lsocks.cast());
        return std::ptr::null_mut();
    }

    lsocks
}

// original: close_listen_sockets (htslib/ref_cache/listener.c:235)
pub unsafe fn ref_cache_listener_c_235_close_listen_sockets(lsocks: *mut Listeners) {
    assert!(!lsocks.is_null());

    let mut i = 0usize;
    while i < (*lsocks).nsocks {
        libc::close(*(*lsocks).sockets.add(i));
        i += 1;
    }
}

// original: register_listener_pollers (htslib/ref_cache/listener.c:242)
pub unsafe fn ref_cache_listener_c_242_register_listener_pollers(
    lsocks: *mut Listeners,
    pw: *mut structs::Poll_wrap,
) -> *mut *mut structs::Pw_item {
    let polled_listeners = libc::calloc(
        (*lsocks).nsocks,
        std::mem::size_of::<*mut structs::Pw_item>(),
    )
    .cast::<*mut structs::Pw_item>();

    if polled_listeners.is_null() {
        libc::perror(c"Allocating listener poll structs".as_ptr());
        return std::ptr::null_mut();
    }

    let mut i = 0usize;
    while i < (*lsocks).nsocks {
        *polled_listeners.add(i) = functions::ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            *(*lsocks).sockets.add(i),
            1,
            (libc::POLLIN | libc::POLLERR | libc::POLLHUP) as u32,
            std::ptr::null_mut::<c_void>(),
        );
        if (*polled_listeners.add(i)).is_null() {
            libc::perror(c"Adding listener socket to poller".as_ptr());
            assert!((*polled_listeners.add(i)).is_null());
            if i > 0 {
                loop {
                    i -= 1;
                    if !(*polled_listeners.add(i)).is_null() {
                        functions::ref_cache_poll_wrap_epoll_c_126_pw_remove(
                            pw,
                            *polled_listeners.add(i),
                            0,
                        );
                    }
                    if i == 0 {
                        break;
                    }
                }
            }
            libc::free(polled_listeners.cast());
            return std::ptr::null_mut();
        }
        i += 1;
    }

    polled_listeners
}
