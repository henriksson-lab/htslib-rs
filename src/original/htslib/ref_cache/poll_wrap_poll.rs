use crate::htslib_mini_rs::cram;
use std::ffi::{c_int, c_uint, c_void};

use super::poll_wrap::{Pw_events, Pw_fd_type, Pw_item};

const INIT_POLLED_SZ: c_uint = 16;
const INIT_IDX_SZ: c_uint = 16;

// original: Poll_wrap (htslib/ref_cache/poll_wrap_poll.c:47)
#[repr(C)]
pub struct Poll_wrap {
    pool: *mut cram::pool_alloc_t,
    polled: *mut libc::pollfd,
    npolled: c_uint,
    polled_sz: c_uint,
    fd_index: *mut c_uint,
    item_index: *mut *mut Pw_item,
    idx_sz: c_uint,
    last_out: c_uint,
    need_compact: c_int,
    debug: c_int,
}

// original: pw_close (htslib/ref_cache/poll_wrap_poll.c:60)
pub unsafe fn ref_cache_poll_wrap_poll_c_60_pw_close(pw: *mut Poll_wrap) {
    if pw.is_null() {
        return;
    }
    if !(*pw).pool.is_null() {
        cram::cram_pooled_alloc_c_84_pool_destroy((*pw).pool);
    }
    if !(*pw).polled.is_null() {
        libc::free((*pw).polled.cast());
    }
    if !(*pw).fd_index.is_null() {
        libc::free((*pw).fd_index.cast());
    }
    if !(*pw).item_index.is_null() {
        libc::free((*pw).item_index.cast());
    }
    libc::free(pw.cast());
}

// original: pw_init (htslib/ref_cache/poll_wrap_poll.c:69)
pub unsafe fn ref_cache_poll_wrap_poll_c_69_pw_init(debug: c_int) -> *mut Poll_wrap {
    let pw = libc::calloc(1, std::mem::size_of::<Poll_wrap>()).cast::<Poll_wrap>();
    if pw.is_null() {
        return std::ptr::null_mut();
    }

    (*pw).pool = cram::cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<Pw_item>());
    if (*pw).pool.is_null() {
        ref_cache_poll_wrap_poll_c_60_pw_close(pw);
        return std::ptr::null_mut();
    }

    (*pw).polled_sz = INIT_POLLED_SZ;
    (*pw).polled =
        libc::malloc((*pw).polled_sz as usize * std::mem::size_of::<libc::pollfd>()).cast();
    if (*pw).polled.is_null() {
        ref_cache_poll_wrap_poll_c_60_pw_close(pw);
        return std::ptr::null_mut();
    }

    (*pw).idx_sz = INIT_IDX_SZ;
    (*pw).fd_index = libc::malloc((*pw).idx_sz as usize * std::mem::size_of::<c_uint>()).cast();
    if (*pw).fd_index.is_null() {
        ref_cache_poll_wrap_poll_c_60_pw_close(pw);
        return std::ptr::null_mut();
    }

    (*pw).item_index =
        libc::calloc((*pw).idx_sz as usize, std::mem::size_of::<*mut Pw_item>()).cast();
    if (*pw).item_index.is_null() {
        ref_cache_poll_wrap_poll_c_60_pw_close(pw);
        return std::ptr::null_mut();
    }

    (*pw).debug = debug;

    pw
}

// original: pw_register (htslib/ref_cache/poll_wrap_poll.c:95)
pub unsafe fn ref_cache_poll_wrap_poll_c_95_pw_register(
    pw: *mut Poll_wrap,
    fd: c_int,
    fd_type: Pw_fd_type,
    init_events: u32,
    userp: *mut c_void,
) -> *mut Pw_item {
    let item;

    if (*pw).debug != 0 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"pw_register(%p, %d, %d, 0x%04x, %p)\n".as_ptr(),
            pw.cast::<c_void>(),
            fd,
            fd_type as c_int,
            init_events,
            userp,
        );
    }

    if fd < 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EBADF;
        return std::ptr::null_mut();
    }

    if (fd as c_uint) < (*pw).idx_sz && !(*(*pw).item_index.add(fd as usize)).is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EEXIST;
        return std::ptr::null_mut();
    }

    if (*pw).idx_sz <= fd as c_uint {
        let new_sz = fd as c_uint + 1;
        let new_index = libc::realloc(
            (*pw).fd_index.cast(),
            new_sz as usize * std::mem::size_of::<c_uint>(),
        )
        .cast::<c_uint>();
        let new_items;
        if new_index.is_null() {
            return std::ptr::null_mut();
        }
        (*pw).fd_index = new_index;

        new_items = libc::realloc(
            (*pw).item_index.cast(),
            new_sz as usize * std::mem::size_of::<*mut Pw_item>(),
        )
        .cast::<*mut Pw_item>();
        if new_items.is_null() {
            return std::ptr::null_mut();
        }
        libc::memset(
            new_items.add((*pw).idx_sz as usize).cast(),
            0,
            (new_sz - (*pw).idx_sz) as usize * std::mem::size_of::<*mut Pw_item>(),
        );
        (*pw).item_index = new_items;
        (*pw).idx_sz = new_sz;
    }
    if (*pw).npolled == (*pw).polled_sz {
        let new_sz = (*pw).polled_sz * 2;
        let new_polled = libc::realloc(
            (*pw).polled.cast(),
            new_sz as usize * std::mem::size_of::<libc::pollfd>(),
        )
        .cast::<libc::pollfd>();
        if new_polled.is_null() {
            return std::ptr::null_mut();
        }
        (*pw).polled = new_polled;
        (*pw).polled_sz = new_sz;
    }

    item = cram::cram_pooled_alloc_c_115_pool_alloc((*pw).pool).cast::<Pw_item>();
    if item.is_null() {
        return std::ptr::null_mut();
    }

    (*item).fd = fd;
    (*item).fd_type = fd_type;
    (*item).userp = userp;

    *(*pw).fd_index.add(fd as usize) = (*pw).npolled;
    *(*pw).item_index.add(fd as usize) = item;

    (*(*pw).polled.add((*pw).npolled as usize)).fd = fd;
    (*(*pw).polled.add((*pw).npolled as usize)).events = init_events as libc::c_short;
    (*pw).npolled += 1;

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
            hts_sys::stderr.cast(),
            c"pw_mod(%p, %d, 0x%04x)\n".as_ptr(),
            pw.cast::<c_void>(),
            (*item).fd,
            events,
        );
    }

    if (*item).fd < 0
        || (*item).fd as c_uint >= (*pw).idx_sz
        || (*(*pw).item_index.add((*item).fd as usize)).is_null()
    {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::ENOENT;
        return -1;
    }

    (*(*pw)
        .polled
        .add(*(*pw).fd_index.add((*item).fd as usize) as usize))
    .events = events as libc::c_short;
    0
}

// original: pw_wait (htslib/ref_cache/poll_wrap_poll.c:173)
pub unsafe fn ref_cache_poll_wrap_poll_c_173_pw_wait(
    pw: *mut Poll_wrap,
    events: *mut Pw_events,
    max_events: c_int,
    timeout: c_int,
) -> c_int {
    let mut j;
    let end;
    let mut out = 0;
    let res;

    if (*pw).need_compact != 0 {
        j = 0;
        for i in 0..(*pw).npolled {
            if i == (*pw).last_out {
                (*pw).last_out = j;
            }
            if (*(*pw)
                .item_index
                .add((*(*pw).polled.add(i as usize)).fd as usize))
            .is_null()
            {
                continue;
            }
            if i != j {
                *(*pw).polled.add(j as usize) = *(*pw).polled.add(i as usize);
                *(*pw)
                    .fd_index
                    .add((*(*pw).polled.add(j as usize)).fd as usize) = j;
            }
            j += 1;
        }
        (*pw).need_compact = 0;
        (*pw).npolled = j;
    }

    res = libc::poll(
        (*pw).polled,
        (*pw).npolled as libc::nfds_t,
        if out == 0 { timeout } else { 0 },
    );
    if res < 0 {
        return res;
    }

    end = (*pw).last_out;
    while (*pw).last_out < (*pw).npolled && out < max_events {
        if (*(*pw).polled.add((*pw).last_out as usize)).revents == 0 {
            (*pw).last_out += 1;
            continue;
        }
        (*events.add(out as usize)).events =
            (*(*pw).polled.add((*pw).last_out as usize)).revents as u32;
        (*events.add(out as usize)).item = *(*pw)
            .item_index
            .add((*(*pw).polled.add((*pw).last_out as usize)).fd as usize);
        out += 1;
        (*pw).last_out += 1;
    }
    (*pw).last_out = 0;
    while (*pw).last_out < end && out < max_events {
        if (*(*pw).polled.add((*pw).last_out as usize)).revents != 0 {
            (*events.add(out as usize)).events =
                (*(*pw).polled.add((*pw).last_out as usize)).revents as u32;
            (*events.add(out as usize)).item = *(*pw)
                .item_index
                .add((*(*pw).polled.add((*pw).last_out as usize)).fd as usize);
            out += 1;
        }
        (*pw).last_out += 1;
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
            hts_sys::stderr.cast(),
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

    if (*item).fd < 0
        || (*item).fd as c_uint >= (*pw).idx_sz
        || (*(*pw).item_index.add((*item).fd as usize)).is_null()
    {
        *crate::htslib_mini_rs::c_compat::__errno_location() = libc::ENOENT;
        return -1;
    }
    *(*pw).item_index.add((*item).fd as usize) = std::ptr::null_mut();
    (*(*pw)
        .polled
        .add(*(*pw).fd_index.add((*item).fd as usize) as usize))
    .events = 0;
    (*pw).need_compact = 1;
    cram::cram_pooled_alloc_c_144_pool_free((*pw).pool, item.cast());
    if do_close == 0 {
        return 0;
    }
    libc::close(fd)
}
