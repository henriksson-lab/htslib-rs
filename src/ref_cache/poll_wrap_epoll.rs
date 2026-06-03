use crate::htslib_rs::cram;
use std::ffi::{c_int, c_void};

use super::poll_wrap::{Pw_fd_type, Pw_item};

const INIT_EPOLL_SIZE: c_int = 128;

// original: Pw_events (htslib/ref_cache/poll_wrap.h:53)
pub type Pw_events = libc::epoll_event;

// original: Poll_wrap (htslib/ref_cache/poll_wrap_epoll.c:43)
#[repr(C)]
pub struct Poll_wrap {
    pool: *mut cram::pool_alloc_t,
    epfd: c_int,
    debug: c_int,
}

// original: pw_init (htslib/ref_cache/poll_wrap_epoll.c:49)
pub unsafe fn ref_cache_poll_wrap_epoll_c_49_pw_init(debug: c_int) -> *mut Poll_wrap {
    let pw = libc::calloc(1, std::mem::size_of::<Poll_wrap>()).cast::<Poll_wrap>();
    if pw.is_null() {
        return std::ptr::null_mut();
    }

    (*pw).pool = cram::cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<Pw_item>());
    if (*pw).pool.is_null() {
        libc::free(pw.cast());
        return std::ptr::null_mut();
    }

    (*pw).epfd = libc::epoll_create(INIT_EPOLL_SIZE);
    if (*pw).epfd < 0 {
        libc::perror(c"epoll_create".as_ptr());
        cram::cram_pooled_alloc_c_84_pool_destroy((*pw).pool);
        libc::free(pw.cast());
        return std::ptr::null_mut();
    }

    (*pw).debug = debug;

    pw
}

// original: pw_close (htslib/ref_cache/poll_wrap_epoll.c:72)
pub unsafe fn ref_cache_poll_wrap_epoll_c_72_pw_close(pw: *mut Poll_wrap) {
    libc::close((*pw).epfd);
    cram::cram_pooled_alloc_c_84_pool_destroy((*pw).pool);
    libc::free(pw.cast());
}

// original: pw_register (htslib/ref_cache/poll_wrap_epoll.c:78)
pub unsafe fn ref_cache_poll_wrap_epoll_c_78_pw_register(
    pw: *mut Poll_wrap,
    fd: c_int,
    fd_type: Pw_fd_type,
    init_events: u32,
    userp: *mut c_void,
) -> *mut Pw_item {
    let mut event: libc::epoll_event = std::mem::zeroed();
    let item = cram::cram_pooled_alloc_c_115_pool_alloc((*pw).pool).cast::<Pw_item>();
    if item.is_null() {
        return std::ptr::null_mut();
    }

    if (*pw).debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"pw_register(%p, %d, %d, 0x%04x, %p)\n".as_ptr(),
            pw.cast::<c_void>(),
            fd,
            fd_type as c_int,
            init_events,
            userp,
        );
    }

    (*item).fd = fd;
    (*item).fd_type = fd_type;
    (*item).userp = userp;

    event.events = init_events;
    event.u64 = item as u64;

    if libc::epoll_ctl((*pw).epfd, libc::EPOLL_CTL_ADD, fd, &mut event) != 0 {
        libc::perror(c"epoll_ctl".as_ptr());
        cram::cram_pooled_alloc_c_144_pool_free((*pw).pool, item.cast());
        return std::ptr::null_mut();
    }

    item
}

// original: pw_mod (htslib/ref_cache/poll_wrap_epoll.c:106)
pub unsafe fn ref_cache_poll_wrap_epoll_c_106_pw_mod(
    pw: *mut Poll_wrap,
    item: *mut Pw_item,
    events: u32,
) -> c_int {
    let mut event: libc::epoll_event = std::mem::zeroed();

    if (*pw).debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"pw_mod(%p, %d, 0x%04x)\n".as_ptr(),
            pw.cast::<c_void>(),
            (*item).fd,
            events,
        );
    }

    event.events = events;
    event.u64 = item as u64;

    libc::epoll_ctl((*pw).epfd, libc::EPOLL_CTL_MOD, (*item).fd, &mut event)
}

// original: pw_wait (htslib/ref_cache/poll_wrap_epoll.c:120)
pub unsafe fn ref_cache_poll_wrap_epoll_c_120_pw_wait(
    pw: *mut Poll_wrap,
    events: *mut Pw_events,
    max_events: c_int,
    timeout: c_int,
) -> c_int {
    libc::epoll_wait((*pw).epfd, events, max_events, timeout)
}

// original: pw_remove (htslib/ref_cache/poll_wrap_epoll.c:126)
pub unsafe fn ref_cache_poll_wrap_epoll_c_126_pw_remove(
    pw: *mut Poll_wrap,
    item: *mut Pw_item,
    do_close: c_int,
) -> c_int {
    let mut dummy: libc::epoll_event = std::mem::zeroed();

    if (*pw).debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"pw_remove(%p, %d%s)\n".as_ptr(),
            pw.cast::<c_void>(),
            (*item).fd,
            if do_close != 0 {
                c", close".as_ptr()
            } else {
                c"".as_ptr()
            },
        );
    }

    let res = if do_close != 0 {
        libc::close((*item).fd)
    } else {
        libc::epoll_ctl((*pw).epfd, libc::EPOLL_CTL_DEL, (*item).fd, &mut dummy)
    };
    cram::cram_pooled_alloc_c_144_pool_free((*pw).pool, item.cast());
    res
}
