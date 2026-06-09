use crate::htslib_rs::cram;
use std::ffi::{c_int, c_uint, c_void};
use std::ptr::NonNull;

use super::poll_wrap::{Pw_events, Pw_fd_type, Pw_item};

const INIT_POLLED_SZ: c_uint = 16;
const INIT_IDX_SZ: c_uint = 16;

// original: Poll_wrap (htslib/ref_cache/poll_wrap_poll.c:47)
pub struct Poll_wrap {
    pool: Option<Box<cram::pool_alloc_t>>,
    polled: Vec<libc::pollfd>,
    fd_index: Vec<c_uint>,
    item_index: Vec<*mut Pw_item>,
    last_out: c_uint,
    need_compact: c_int,
    debug: c_int,
}

// original: pw_close (htslib/ref_cache/poll_wrap_poll.c:60)
pub unsafe fn ref_cache_poll_wrap_poll_c_60_pw_close(pw: *mut Poll_wrap) {
    if pw.is_null() {
        return;
    }
    let mut pw = Box::from_raw(pw);
    if let Some(pool) = pw.pool.take() {
        cram::pool_destroy_box(pool);
    }
}

// original: pw_init (htslib/ref_cache/poll_wrap_poll.c:69)
pub unsafe fn ref_cache_poll_wrap_poll_c_69_pw_init(debug: c_int) -> *mut Poll_wrap {
    let mut pw = Box::new(Poll_wrap {
        pool: Some(cram::pool_create(std::mem::size_of::<Pw_item>())),
        polled: Vec::new(),
        fd_index: Vec::new(),
        item_index: Vec::new(),
        last_out: 0,
        need_compact: 0,
        debug,
    });
    if pw
        .polled
        .try_reserve_exact(INIT_POLLED_SZ as usize)
        .is_err()
    {
        ref_cache_poll_wrap_poll_c_60_pw_close(Box::into_raw(pw));
        return std::ptr::null_mut();
    }
    if pw.fd_index.try_reserve_exact(INIT_IDX_SZ as usize).is_err() {
        ref_cache_poll_wrap_poll_c_60_pw_close(Box::into_raw(pw));
        return std::ptr::null_mut();
    }
    if pw
        .item_index
        .try_reserve_exact(INIT_IDX_SZ as usize)
        .is_err()
    {
        ref_cache_poll_wrap_poll_c_60_pw_close(Box::into_raw(pw));
        return std::ptr::null_mut();
    }
    pw.fd_index.resize(INIT_IDX_SZ as usize, 0);
    pw.item_index
        .resize(INIT_IDX_SZ as usize, std::ptr::null_mut());

    Box::into_raw(pw)
}

// original: pw_register (htslib/ref_cache/poll_wrap_poll.c:95)
pub unsafe fn ref_cache_poll_wrap_poll_c_95_pw_register(
    pw: *mut Poll_wrap,
    fd: c_int,
    fd_type: Pw_fd_type,
    init_events: u32,
    userp: *mut c_void,
) -> *mut Pw_item {
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

    if fd < 0 {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EBADF;
        return std::ptr::null_mut();
    }

    let pw_ref = &mut *pw;
    let fd_idx = fd as usize;

    if fd_idx < pw_ref.item_index.len() && !pw_ref.item_index[fd_idx].is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EEXIST;
        return std::ptr::null_mut();
    }

    if pw_ref.item_index.len() <= fd_idx {
        let new_len = fd_idx + 1;
        let add = new_len - pw_ref.item_index.len();
        if pw_ref.fd_index.try_reserve_exact(add).is_err()
            || pw_ref.item_index.try_reserve_exact(add).is_err()
        {
            return std::ptr::null_mut();
        }
        pw_ref.fd_index.resize(new_len, 0);
        pw_ref.item_index.resize(new_len, std::ptr::null_mut());
    }
    if pw_ref.polled.len() == pw_ref.polled.capacity()
        && pw_ref
            .polled
            .try_reserve_exact(pw_ref.polled.capacity().max(1))
            .is_err()
    {
        return std::ptr::null_mut();
    }

    let Some(pool) = pw_ref.pool.as_mut() else {
        return std::ptr::null_mut();
    };
    let Some(mut item) = cram::pool_alloc(pool).map(NonNull::cast::<Pw_item>) else {
        return std::ptr::null_mut();
    };

    item.as_mut().fd = fd;
    item.as_mut().fd_type = fd_type;
    item.as_mut().userp = userp;
    let item = item.as_ptr();

    pw_ref.fd_index[fd_idx] = pw_ref.polled.len() as c_uint;
    pw_ref.item_index[fd_idx] = item;

    pw_ref.polled.push(libc::pollfd {
        fd,
        events: init_events as libc::c_short,
        revents: 0,
    });

    item
}

// original: pw_mod (htslib/ref_cache/poll_wrap_poll.c:157)
pub unsafe fn ref_cache_poll_wrap_poll_c_157_pw_mod(
    pw: *mut Poll_wrap,
    item: *mut Pw_item,
    events: u32,
) -> c_int {
    if (*pw).debug != 0 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"pw_mod(%p, %d, 0x%04x)\n".as_ptr(),
            pw.cast::<c_void>(),
            (*item).fd,
            events,
        );
    }

    let pw_ref = &mut *pw;
    let fd_idx = (*item).fd as usize;
    if (*item).fd < 0 || fd_idx >= pw_ref.item_index.len() || pw_ref.item_index[fd_idx].is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOENT;
        return -1;
    }

    pw_ref.polled[pw_ref.fd_index[fd_idx] as usize].events = events as libc::c_short;
    0
}

// original: pw_wait (htslib/ref_cache/poll_wrap_poll.c:173)
pub unsafe fn ref_cache_poll_wrap_poll_c_173_pw_wait(
    pw: *mut Poll_wrap,
    events: *mut Pw_events,
    max_events: c_int,
    timeout: c_int,
) -> c_int {
    let pw_ref = &mut *pw;
    let mut out = 0;

    if pw_ref.need_compact != 0 {
        let mut j = 0usize;
        for i in 0..pw_ref.polled.len() {
            if i as c_uint == pw_ref.last_out {
                pw_ref.last_out = j as c_uint;
            }
            if pw_ref.item_index[pw_ref.polled[i].fd as usize].is_null() {
                continue;
            }
            if i != j {
                pw_ref.polled[j] = pw_ref.polled[i];
                pw_ref.fd_index[pw_ref.polled[j].fd as usize] = j as c_uint;
            }
            j += 1;
        }
        pw_ref.need_compact = 0;
        pw_ref.polled.truncate(j);
    }

    let res = libc::poll(
        pw_ref.polled.as_mut_ptr(),
        pw_ref.polled.len() as libc::nfds_t,
        if out == 0 { timeout } else { 0 },
    );
    if res < 0 {
        return res;
    }

    let end = pw_ref.last_out;
    while (pw_ref.last_out as usize) < pw_ref.polled.len() && out < max_events {
        if pw_ref.polled[pw_ref.last_out as usize].revents == 0 {
            pw_ref.last_out += 1;
            continue;
        }
        let polled = pw_ref.polled[pw_ref.last_out as usize];
        (*events.add(out as usize)).events = polled.revents as u32;
        (*events.add(out as usize)).item = pw_ref.item_index[polled.fd as usize];
        out += 1;
        pw_ref.last_out += 1;
    }
    pw_ref.last_out = 0;
    while pw_ref.last_out < end && out < max_events {
        let polled = pw_ref.polled[pw_ref.last_out as usize];
        if polled.revents != 0 {
            (*events.add(out as usize)).events = polled.revents as u32;
            (*events.add(out as usize)).item = pw_ref.item_index[polled.fd as usize];
            out += 1;
        }
        pw_ref.last_out += 1;
    }

    out
}

// original: pw_remove (htslib/ref_cache/poll_wrap_poll.c:220)
pub unsafe fn ref_cache_poll_wrap_poll_c_220_pw_remove(
    pw: *mut Poll_wrap,
    item: *mut Pw_item,
    do_close: c_int,
) -> c_int {
    let fd = (*item).fd;

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

    let pw_ref = &mut *pw;
    let fd_idx = (*item).fd as usize;
    if (*item).fd < 0 || fd_idx >= pw_ref.item_index.len() || pw_ref.item_index[fd_idx].is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOENT;
        return -1;
    }
    pw_ref.item_index[fd_idx] = std::ptr::null_mut();
    pw_ref.polled[pw_ref.fd_index[fd_idx] as usize].events = 0;
    pw_ref.need_compact = 1;
    if let (Some(pool), Some(item)) = (pw_ref.pool.as_mut(), NonNull::new(item.cast::<u8>())) {
        cram::pool_free(pool, item);
    }
    if do_close == 0 {
        return 0;
    }
    libc::close(fd)
}
