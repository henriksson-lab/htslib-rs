use crate::htslib_rs::cram;
use std::ffi::{c_char, c_int, c_void};
use std::ptr::NonNull;

use super::poll_wrap::{Pw_fd_type, Pw_item};

const INIT_EPOLL_SIZE: c_int = 128;

// original: Pw_events (htslib/ref_cache/poll_wrap.h:53)
pub type Pw_events = libc::epoll_event;

// original: Poll_wrap (htslib/ref_cache/poll_wrap_epoll.c:43)
pub struct Poll_wrap {
    pool: Box<cram::pool_alloc_t>,
    epfd: c_int,
    debug: c_int,
}

// original: pw_init (htslib/ref_cache/poll_wrap_epoll.c:49)
pub unsafe fn ref_cache_poll_wrap_epoll_c_49_pw_init(debug: c_int) -> *mut Poll_wrap {
    let pool = cram::pool_create(std::mem::size_of::<Pw_item>());
    let epfd = libc::epoll_create(INIT_EPOLL_SIZE);
    if epfd < 0 {
        libc::perror(b"epoll_create\0".as_ptr().cast::<c_char>());
        cram::pool_destroy_box(pool);
        return std::ptr::null_mut();
    }

    Box::into_raw(Box::new(Poll_wrap { pool, epfd, debug }))
}

// original: pw_close (htslib/ref_cache/poll_wrap_epoll.c:72)
pub unsafe fn ref_cache_poll_wrap_epoll_c_72_pw_close(pw: *mut Poll_wrap) {
    let Some(pw) = NonNull::new(pw) else {
        return;
    };
    let Poll_wrap { mut pool, epfd, .. } = *Box::from_raw(pw.as_ptr());
    libc::close(epfd);
    cram::pool_destroy(&mut pool);
}

// original: pw_register (htslib/ref_cache/poll_wrap_epoll.c:78)
pub unsafe fn ref_cache_poll_wrap_epoll_c_78_pw_register(
    pw: *mut Poll_wrap,
    fd: c_int,
    fd_type: Pw_fd_type,
    init_events: u32,
    userp: *mut c_void,
) -> *mut Pw_item {
    let Some(pw) = pw.as_mut() else {
        return std::ptr::null_mut();
    };
    let mut event: libc::epoll_event = std::mem::zeroed();
    let Some(item) = cram::pool_alloc(&mut pw.pool).map(NonNull::cast::<Pw_item>) else {
        return std::ptr::null_mut();
    };

    if pw.debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            b"pw_register(%p, %d, %d, 0x%04x, %p)\n"
                .as_ptr()
                .cast::<c_char>(),
            (pw as *mut Poll_wrap).cast::<c_void>(),
            fd,
            fd_type as c_int,
            init_events,
            userp,
        );
    }

    item.as_ptr().write(Pw_item { fd, fd_type, userp });

    event.events = init_events;
    event.u64 = item.as_ptr() as u64;

    if libc::epoll_ctl(pw.epfd, libc::EPOLL_CTL_ADD, fd, &mut event) != 0 {
        libc::perror(b"epoll_ctl\0".as_ptr().cast::<c_char>());
        cram::pool_free(&mut pw.pool, item.cast());
        return std::ptr::null_mut();
    }

    item.as_ptr()
}

// original: pw_mod (htslib/ref_cache/poll_wrap_epoll.c:106)
pub unsafe fn ref_cache_poll_wrap_epoll_c_106_pw_mod(
    pw: *mut Poll_wrap,
    item: *mut Pw_item,
    events: u32,
) -> c_int {
    let Some(pw) = pw.as_mut() else {
        return -1;
    };
    let Some(item) = item.as_mut() else {
        return -1;
    };
    let mut event: libc::epoll_event = std::mem::zeroed();

    if pw.debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            b"pw_mod(%p, %d, 0x%04x)\n".as_ptr().cast::<c_char>(),
            (pw as *mut Poll_wrap).cast::<c_void>(),
            item.fd,
            events,
        );
    }

    event.events = events;
    event.u64 = item as *mut Pw_item as u64;

    libc::epoll_ctl(pw.epfd, libc::EPOLL_CTL_MOD, item.fd, &mut event)
}

// original: pw_wait (htslib/ref_cache/poll_wrap_epoll.c:120)
pub unsafe fn ref_cache_poll_wrap_epoll_c_120_pw_wait(
    pw: *mut Poll_wrap,
    events: *mut Pw_events,
    max_events: c_int,
    timeout: c_int,
) -> c_int {
    let Some(pw) = pw.as_mut() else {
        return -1;
    };
    libc::epoll_wait(pw.epfd, events, max_events, timeout)
}

// original: pw_remove (htslib/ref_cache/poll_wrap_epoll.c:126)
pub unsafe fn ref_cache_poll_wrap_epoll_c_126_pw_remove(
    pw: *mut Poll_wrap,
    item: *mut Pw_item,
    do_close: c_int,
) -> c_int {
    let Some(pw) = pw.as_mut() else {
        return -1;
    };
    let Some(item) = item.as_mut() else {
        return -1;
    };
    let mut dummy: libc::epoll_event = std::mem::zeroed();

    if pw.debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            b"pw_remove(%p, %d%s)\n".as_ptr().cast::<c_char>(),
            (pw as *mut Poll_wrap).cast::<c_void>(),
            item.fd,
            if do_close != 0 {
                b", close\0".as_ptr().cast::<c_char>()
            } else {
                b"\0".as_ptr().cast::<c_char>()
            },
        );
    }

    let res = if do_close != 0 {
        libc::close(item.fd)
    } else {
        libc::epoll_ctl(pw.epfd, libc::EPOLL_CTL_DEL, item.fd, &mut dummy)
    };
    cram::pool_free(
        &mut pw.pool,
        NonNull::new(item as *mut Pw_item)
            .expect("checked non-null pooled epoll item")
            .cast(),
    );
    res
}
