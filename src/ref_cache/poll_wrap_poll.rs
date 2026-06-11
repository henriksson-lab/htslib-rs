use std::ffi::{c_int, c_uint};

use super::poll_wrap::{Pw_events, Pw_fd_type, Pw_item};

const INIT_POLLED_SZ: usize = 16;
const INIT_IDX_SZ: usize = 16;

// original: Poll_wrap (htslib/ref_cache/poll_wrap_poll.c:47)
pub struct Poll_wrap {
    polled: Vec<libc::pollfd>,
    fd_index: Vec<c_uint>,
    // Owned arena of registered items, indexed by file descriptor.
    // `None` means the slot is unused (replaces the old NULL pointer check).
    item_index: Vec<Option<Box<Pw_item>>>,
    last_out: c_uint,
    need_compact: bool,
    debug: bool,
}

// original: pw_close (htslib/ref_cache/poll_wrap_poll.c:60)
pub fn ref_cache_poll_wrap_poll_c_60_pw_close(_pw: Option<Box<Poll_wrap>>) {
    // Dropping the Box frees the Poll_wrap and, recursively, every owned
    // item in item_index. Nothing else to do.
}

// original: pw_init (htslib/ref_cache/poll_wrap_poll.c:69)
pub fn ref_cache_poll_wrap_poll_c_69_pw_init(debug: bool) -> Option<Box<Poll_wrap>> {
    let mut polled = Vec::new();
    if polled.try_reserve_exact(INIT_POLLED_SZ).is_err() {
        return None;
    }

    let mut fd_index = Vec::new();
    if fd_index.try_reserve_exact(INIT_IDX_SZ).is_err() {
        return None;
    }

    let mut item_index = Vec::new();
    if item_index.try_reserve_exact(INIT_IDX_SZ).is_err() {
        return None;
    }

    fd_index.resize(INIT_IDX_SZ, 0);
    item_index.resize_with(INIT_IDX_SZ, || None);

    Some(Box::new(Poll_wrap {
        polled,
        fd_index,
        item_index,
        last_out: 0,
        need_compact: false,
        debug,
    }))
}

// original: pw_register (htslib/ref_cache/poll_wrap_poll.c:95)
pub fn ref_cache_poll_wrap_poll_c_95_pw_register(
    pw: &mut Poll_wrap,
    fd: c_int,
    fd_type: Pw_fd_type,
    init_events: u32,
    userp: usize,
) -> Option<&mut Pw_item> {
    if pw.debug {
        eprintln!(
            "pw_register({:p}, {}, {}, 0x{:04x}, {})",
            pw as *const Poll_wrap,
            fd,
            fd_type as c_int,
            init_events,
            userp,
        );
    }

    if fd < 0 {
        unsafe {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EBADF;
        }
        return None;
    }

    let fd_idx = fd as usize;

    if fd_idx < pw.item_index.len() && pw.item_index[fd_idx].is_some() {
        unsafe {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EEXIST;
        }
        return None;
    }

    if pw.item_index.len() <= fd_idx {
        let new_len = fd_idx + 1;
        let add = new_len - pw.item_index.len();
        if pw.fd_index.try_reserve_exact(add).is_err()
            || pw.item_index.try_reserve_exact(add).is_err()
        {
            return None;
        }
        pw.fd_index.resize(new_len, 0);
        pw.item_index.resize_with(new_len, || None);
    }
    if pw.polled.len() == pw.polled.capacity()
        && pw
            .polled
            .try_reserve_exact(pw.polled.capacity().max(1))
            .is_err()
    {
        return None;
    }

    pw.fd_index[fd_idx] = pw.polled.len() as c_uint;
    pw.item_index[fd_idx] = Some(Box::new(Pw_item {
        fd,
        fd_type,
        userp,
    }));

    pw.polled.push(libc::pollfd {
        fd,
        events: init_events as libc::c_short,
        revents: 0,
    });

    pw.item_index[fd_idx].as_deref_mut()
}

// original: pw_mod (htslib/ref_cache/poll_wrap_poll.c:157)
pub fn ref_cache_poll_wrap_poll_c_157_pw_mod(
    pw: &mut Poll_wrap,
    item: &Pw_item,
    events: u32,
) -> c_int {
    if pw.debug {
        eprintln!(
            "pw_mod({:p}, {}, 0x{:04x})",
            pw as *const Poll_wrap,
            item.fd,
            events,
        );
    }

    let fd_idx = item.fd as usize;
    if item.fd < 0 || fd_idx >= pw.item_index.len() || pw.item_index[fd_idx].is_none() {
        unsafe {
            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOENT;
        }
        return -1;
    }

    pw.polled[pw.fd_index[fd_idx] as usize].events = events as libc::c_short;
    0
}

// original: pw_wait (htslib/ref_cache/poll_wrap_poll.c:173)
pub fn ref_cache_poll_wrap_poll_c_173_pw_wait(
    pw: &mut Poll_wrap,
    events: &mut [Pw_events],
    max_events: c_int,
    timeout: c_int,
) -> c_int {
    let mut out = 0;

    if pw.need_compact {
        let mut j = 0usize;
        for i in 0..pw.polled.len() {
            if i as c_uint == pw.last_out {
                pw.last_out = j as c_uint;
            }
            if pw.item_index[pw.polled[i].fd as usize].is_none() {
                continue;
            }
            if i != j {
                pw.polled[j] = pw.polled[i];
                pw.fd_index[pw.polled[j].fd as usize] = j as c_uint;
            }
            j += 1;
        }
        pw.need_compact = false;
        pw.polled.truncate(j);
    }

    let res = unsafe {
        libc::poll(
            pw.polled.as_mut_ptr(),
            pw.polled.len() as libc::nfds_t,
            if out == 0 { timeout } else { 0 },
        )
    };
    if res < 0 {
        return res;
    }

    let end = pw.last_out;
    while (pw.last_out as usize) < pw.polled.len() && out < max_events {
        if pw.polled[pw.last_out as usize].revents == 0 {
            pw.last_out += 1;
            continue;
        }
        let polled = pw.polled[pw.last_out as usize];
        events[out as usize].events = polled.revents as u32;
        events[out as usize].item = pw.item_index[polled.fd as usize].as_deref().copied();
        out += 1;
        pw.last_out += 1;
    }
    pw.last_out = 0;
    while pw.last_out < end && out < max_events {
        let polled = pw.polled[pw.last_out as usize];
        if polled.revents != 0 {
            events[out as usize].events = polled.revents as u32;
            events[out as usize].item = pw.item_index[polled.fd as usize].as_deref().copied();
            out += 1;
        }
        pw.last_out += 1;
    }

    out
}

// original: pw_remove (htslib/ref_cache/poll_wrap_poll.c:220)
pub fn ref_cache_poll_wrap_poll_c_220_pw_remove(
    pw: &mut Poll_wrap,
    item: &Pw_item,
    do_close: bool,
) -> c_int {
    let fd = item.fd;

    if pw.debug {
        eprintln!(
            "pw_remove({:p}, {}{})",
            pw as *const Poll_wrap,
            item.fd,
            if do_close { ", close" } else { "" },
        );
    }

    let fd_idx = item.fd as usize;
    if item.fd < 0 || fd_idx >= pw.item_index.len() || pw.item_index[fd_idx].is_none() {
        unsafe {
            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOENT;
        }
        return -1;
    }
    pw.polled[pw.fd_index[fd_idx] as usize].events = 0;
    pw.need_compact = true;
    // Drop the owned item; this replaces both the NULL store and pool_free.
    pw.item_index[fd_idx] = None;
    if !do_close {
        return 0;
    }
    unsafe { libc::close(fd) }
}
