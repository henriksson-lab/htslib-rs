use super::misc::ref_cache_misc_h_40_setnonblock;
use super::poll_wrap::{Pw_fd_type, Pw_item};
use super::poll_wrap_epoll::{
    ref_cache_poll_wrap_epoll_c_126_pw_remove, ref_cache_poll_wrap_epoll_c_78_pw_register,
    Poll_wrap,
};
use std::ffi::{c_char, c_int, c_void};

// original: Listeners (htslib/ref_cache/listener.c:46)
pub struct Listeners {
    sockets: Vec<c_int>,
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
        let serrno = *crate::htslib_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_rs::c_compat::__errno_location() = serrno;
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
        let serrno = *crate::htslib_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if libc::bind(s, (*addr).ai_addr, (*addr).ai_addrlen) == -1 {
        *cause = c"bind".as_ptr();
        let serrno = *crate::htslib_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if ref_cache_misc_h_40_setnonblock(s) != 0 {
        *cause = c"setnonblock".as_ptr();
        let serrno = *crate::htslib_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    if libc::listen(s, libc::SOMAXCONN) != 0 {
        *cause = c"listen".as_ptr();
        let serrno = *crate::htslib_rs::c_compat::__errno_location();
        libc::close(s);
        *crate::htslib_rs::c_compat::__errno_location() = serrno;
        return -1;
    }

    s
}

// original: get_listen_sockets (htslib/ref_cache/listener.c:95)
pub unsafe fn ref_cache_listener_c_95_get_listen_sockets(port: c_int) -> *mut Listeners {
    let mut hints: libc::addrinfo = std::mem::zeroed();
    let mut addr_list: *mut libc::addrinfo = std::ptr::null_mut();
    let pnum = format!("{port}\0").into_bytes();
    let mut cause: *const c_char = std::ptr::null();

    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE | libc::AI_NUMERICSERV;
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
    {
        hints.ai_flags |= libc::AI_ADDRCONFIG;
    }

    let res = libc::getaddrinfo(
        std::ptr::null(),
        pnum.as_ptr().cast::<c_char>(),
        &hints,
        &mut addr_list,
    );
    if res != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"getaddrinfo failed: %s\n".as_ptr(),
            libc::gai_strerror(res),
        );
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
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"getaddrinfo returned nothing.\n".as_ptr(),
        );
        libc::freeaddrinfo(addr_list);
        return std::ptr::null_mut();
    }

    let mut lsocks = Box::new(Listeners {
        sockets: Vec::new(),
    });
    if lsocks.sockets.try_reserve_exact(count).is_err() {
        libc::perror(c"Allocating listener socket list".as_ptr());
        libc::freeaddrinfo(addr_list);
        return std::ptr::null_mut();
    }

    addr = addr_list;
    while !addr.is_null() {
        assert!(lsocks.sockets.len() < count);
        let fd = ref_cache_listener_c_51_open_socket(addr, &mut cause);
        if fd != -1 {
            lsocks.sockets.push(fd);
        }
        addr = (*addr).ai_next;
    }

    libc::freeaddrinfo(addr_list);

    if lsocks.sockets.is_empty() {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"Failure in %s while getting socket: %s\n".as_ptr(),
            cause,
            libc::strerror(*crate::htslib_rs::c_compat::__errno_location()),
        );
        return std::ptr::null_mut();
    }

    Box::into_raw(lsocks)
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
    if !crate::htslib_rs::ref_cache::compat::s_issock(statbuf.st_mode) {
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

    if ref_cache_misc_h_40_setnonblock(fd) != 0 {
        return 0;
    }

    1
}

// original: adopt_listen_sockets (htslib/ref_cache/listener.c:203)
pub unsafe fn ref_cache_listener_c_203_adopt_listen_sockets(
    min_sock_fd: c_int,
    num_fds: c_int,
) -> *mut Listeners {
    assert!(min_sock_fd > 0 && num_fds > 0 && num_fds < c_int::MAX - min_sock_fd);

    let mut lsocks = Box::new(Listeners {
        sockets: Vec::new(),
    });
    if lsocks.sockets.try_reserve_exact(num_fds as usize).is_err() {
        libc::perror(c"Allocating listener socket list".as_ptr());
        let mut fd = min_sock_fd;
        while fd < min_sock_fd + num_fds {
            libc::close(fd);
            fd += 1;
        }
        return std::ptr::null_mut();
    }

    let mut fd = min_sock_fd;
    while fd < min_sock_fd + num_fds {
        if ref_cache_listener_c_160_should_adopt_socket(fd) != 0 {
            lsocks.sockets.push(fd);
        } else {
            libc::close(fd);
        }
        fd += 1;
    }

    if lsocks.sockets.is_empty() {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"No suitable sockets found\n".as_ptr(),
        );
        return std::ptr::null_mut();
    }

    Box::into_raw(lsocks)
}

// original: close_listen_sockets (htslib/ref_cache/listener.c:235)
pub unsafe fn ref_cache_listener_c_235_close_listen_sockets(lsocks: *mut Listeners) {
    assert!(!lsocks.is_null());

    let lsocks = Box::from_raw(lsocks);
    for &socket in &lsocks.sockets {
        libc::close(socket);
    }
}

// original: register_listener_pollers (htslib/ref_cache/listener.c:242)
pub unsafe fn ref_cache_listener_c_242_register_listener_pollers(
    lsocks: *mut Listeners,
    pw: *mut Poll_wrap,
) -> c_int {
    let mut polled_items: Vec<*mut Pw_item> = Vec::new();
    if polled_items
        .try_reserve_exact((&(*lsocks).sockets).len())
        .is_err()
    {
        libc::perror(c"Allocating listener poll structs".as_ptr());
        return -1;
    }

    let mut i = 0usize;
    while i < (*lsocks).sockets.len() {
        let item = ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            (&(*lsocks).sockets)[i],
            Pw_fd_type::SV_LISTENER,
            (libc::POLLIN | libc::POLLERR | libc::POLLHUP) as u32,
            std::ptr::null_mut::<c_void>(),
        );
        if item.is_null() {
            libc::perror(c"Adding listener socket to poller".as_ptr());
            while let Some(item) = polled_items.pop() {
                if !item.is_null() {
                    ref_cache_poll_wrap_epoll_c_126_pw_remove(pw, item, 0);
                }
            }
            return -1;
        }
        polled_items.push(item);
        i += 1;
    }

    0
}
