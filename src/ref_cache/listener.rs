use super::misc::ref_cache_misc_h_40_setnonblock;
use super::poll_wrap::Pw_fd_type;
use super::poll_wrap_epoll::{
    ref_cache_poll_wrap_epoll_c_126_pw_remove, ref_cache_poll_wrap_epoll_c_78_pw_register,
    Poll_wrap,
};
// original: Listeners (htslib/ref_cache/listener.c:46)
pub struct Listeners {
    sockets: Vec<i32>,
}

// original: open_socket (htslib/ref_cache/listener.c:51)
// Returns the new socket fd on success, or Err(cause) naming the failed step.
pub unsafe fn ref_cache_listener_c_51_open_socket(
    addr: &mut libc::addrinfo,
) -> Result<i32, &'static [u8]> {
    let mut val: i32 = 1;

    let s = libc::socket(addr.ai_family, addr.ai_socktype, addr.ai_protocol);
    if s == -1 {
        return Err(b"socket");
    }

    if libc::setsockopt(
        s,
        libc::SOL_SOCKET,
        libc::SO_REUSEADDR,
        (&mut val as *mut i32).cast(),
        std::mem::size_of_val(&val) as libc::socklen_t,
    ) < 0
    {
        let serrno = *libc::__errno_location();
        libc::close(s);
        *libc::__errno_location() = serrno;
        return Err(b"setsockopt");
    }

    if addr.ai_family == libc::AF_INET6
        && libc::setsockopt(
            s,
            libc::IPPROTO_IPV6,
            libc::IPV6_V6ONLY,
            (&mut val as *mut i32).cast(),
            std::mem::size_of_val(&val) as libc::socklen_t,
        ) < 0
    {
        let serrno = *libc::__errno_location();
        libc::close(s);
        *libc::__errno_location() = serrno;
        return Err(b"setsockopt");
    }

    if libc::bind(s, addr.ai_addr, addr.ai_addrlen) == -1 {
        let serrno = *libc::__errno_location();
        libc::close(s);
        *libc::__errno_location() = serrno;
        return Err(b"bind");
    }

    if ref_cache_misc_h_40_setnonblock(s) != 0 {
        let serrno = *libc::__errno_location();
        libc::close(s);
        *libc::__errno_location() = serrno;
        return Err(b"setnonblock");
    }

    if libc::listen(s, libc::SOMAXCONN) != 0 {
        let serrno = *libc::__errno_location();
        libc::close(s);
        *libc::__errno_location() = serrno;
        return Err(b"listen");
    }

    Ok(s)
}

// original: get_listen_sockets (htslib/ref_cache/listener.c:95)
pub unsafe fn ref_cache_listener_c_95_get_listen_sockets(port: i32) -> Option<Box<Listeners>> {
    let mut hints: libc::addrinfo = std::mem::zeroed();
    let mut addr_list: *mut libc::addrinfo = std::ptr::null_mut();
    let pnum = format!("{port}\0").into_bytes();
    let mut cause: &[u8] = b"";

    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE | libc::AI_NUMERICSERV;
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
    {
        hints.ai_flags |= libc::AI_ADDRCONFIG;
    }

    let res = libc::getaddrinfo(
        std::ptr::null(),
        pnum.as_ptr().cast::<libc::c_char>(),
        &hints,
        &mut addr_list,
    );
    if res != 0 {
        let msg = std::ffi::CStr::from_ptr(libc::gai_strerror(res))
            .to_string_lossy()
            .into_owned();
        eprintln!("getaddrinfo failed: {msg}");
        return None;
    }

    let mut count = 0usize;
    let mut addr = addr_list;
    while !addr.is_null() {
        count += 1;
        addr = (*addr).ai_next;
    }

    if count == 0 {
        eprintln!("getaddrinfo returned nothing.");
        libc::freeaddrinfo(addr_list);
        return None;
    }

    let mut lsocks = Box::new(Listeners {
        sockets: Vec::new(),
    });
    if lsocks.sockets.try_reserve_exact(count).is_err() {
        eprintln!("Allocating listener socket list: cannot allocate memory");
        libc::freeaddrinfo(addr_list);
        return None;
    }

    addr = addr_list;
    while !addr.is_null() {
        assert!(lsocks.sockets.len() < count);
        match ref_cache_listener_c_51_open_socket(&mut *addr) {
            Ok(fd) => lsocks.sockets.push(fd),
            Err(step) => cause = step,
        }
        addr = (*addr).ai_next;
    }

    libc::freeaddrinfo(addr_list);

    if lsocks.sockets.is_empty() {
        let err = std::io::Error::last_os_error();
        eprintln!(
            "Failure in {} while getting socket: {}",
            String::from_utf8_lossy(cause),
            err
        );
        return None;
    }

    Some(lsocks)
}

// original: should_adopt_socket (htslib/ref_cache/listener.c:160)
pub unsafe fn ref_cache_listener_c_160_should_adopt_socket(fd: i32) -> bool {
    let mut statbuf: libc::stat = std::mem::zeroed();
    let mut sock_type: i32 = 0;
    let mut accepting: i32 = 0;
    let mut addr: libc::sockaddr = std::mem::zeroed();
    let mut len: libc::socklen_t;
    let mut addrlen: libc::socklen_t;

    if libc::fstat(fd, &mut statbuf) < 0 {
        return false;
    }
    if !crate::htslib_rs::ref_cache::compat::s_issock(statbuf.st_mode) {
        return false;
    }

    len = std::mem::size_of_val(&sock_type) as libc::socklen_t;
    if libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_TYPE,
        (&mut sock_type as *mut i32).cast(),
        &mut len,
    ) < 0
        || len != std::mem::size_of_val(&sock_type) as libc::socklen_t
        || sock_type != libc::SOCK_STREAM
    {
        return false;
    }

    addrlen = std::mem::size_of_val(&addr) as libc::socklen_t;
    if libc::getsockname(fd, &mut addr, &mut addrlen) < 0
        || addrlen < std::mem::size_of::<libc::sa_family_t>() as libc::socklen_t
        || (addr.sa_family as i32 != libc::AF_INET && addr.sa_family as i32 != libc::AF_INET6)
    {
        return false;
    }

    len = std::mem::size_of_val(&accepting) as libc::socklen_t;
    if libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_ACCEPTCONN,
        (&mut accepting as *mut i32).cast(),
        &mut len,
    ) < 0
        || len != std::mem::size_of_val(&accepting) as libc::socklen_t
    {
        return false;
    }
    if accepting == 0 && libc::listen(fd, libc::SOMAXCONN) != 0 {
        return false;
    }

    if ref_cache_misc_h_40_setnonblock(fd) != 0 {
        return false;
    }

    true
}

// original: adopt_listen_sockets (htslib/ref_cache/listener.c:203)
pub unsafe fn ref_cache_listener_c_203_adopt_listen_sockets(
    min_sock_fd: i32,
    num_fds: i32,
) -> Option<Box<Listeners>> {
    assert!(min_sock_fd > 0 && num_fds > 0 && num_fds < i32::MAX - min_sock_fd);

    let mut lsocks = Box::new(Listeners {
        sockets: Vec::new(),
    });
    if lsocks.sockets.try_reserve_exact(num_fds as usize).is_err() {
        eprintln!("Allocating listener socket list: cannot allocate memory");
        let mut fd = min_sock_fd;
        while fd < min_sock_fd + num_fds {
            libc::close(fd);
            fd += 1;
        }
        return None;
    }

    let mut fd = min_sock_fd;
    while fd < min_sock_fd + num_fds {
        if ref_cache_listener_c_160_should_adopt_socket(fd) {
            lsocks.sockets.push(fd);
        } else {
            libc::close(fd);
        }
        fd += 1;
    }

    if lsocks.sockets.is_empty() {
        eprintln!("No suitable sockets found");
        return None;
    }

    Some(lsocks)
}

// original: close_listen_sockets (htslib/ref_cache/listener.c:235)
pub unsafe fn ref_cache_listener_c_235_close_listen_sockets(lsocks: Box<Listeners>) {
    for &socket in &lsocks.sockets {
        libc::close(socket);
    }
}

// original: register_listener_pollers (htslib/ref_cache/listener.c:242)
pub unsafe fn ref_cache_listener_c_242_register_listener_pollers(
    lsocks: &Listeners,
    pw: &mut Poll_wrap,
) -> i32 {
    // Poller arena indices of the registered listener sockets (was a
    // Vec<*mut Pw_item>); the new poll API returns an `Option<usize>` handle.
    let mut polled_items: Vec<usize> = Vec::new();
    if polled_items.try_reserve_exact(lsocks.sockets.len()).is_err() {
        eprintln!("Allocating listener poll structs: cannot allocate memory");
        return -1;
    }

    let mut i = 0usize;
    while i < lsocks.sockets.len() {
        let item = ref_cache_poll_wrap_epoll_c_78_pw_register(
            pw,
            lsocks.sockets[i],
            Pw_fd_type::SV_LISTENER,
            (libc::POLLIN | libc::POLLERR | libc::POLLHUP) as u32,
            0,
        );
        let Some(item) = item else {
            eprintln!("Adding listener socket to poller: error");
            while let Some(item) = polled_items.pop() {
                ref_cache_poll_wrap_epoll_c_126_pw_remove(pw, item, false);
            }
            return -1;
        };
        polled_items.push(item);
        i += 1;
    }

    0
}
