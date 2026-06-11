use super::poll_wrap::{Pw_fd_type, Pw_item};

const INIT_EPOLL_SIZE: i32 = 128;

// original: Pw_events (htslib/ref_cache/poll_wrap.h:53)
pub type Pw_events = libc::epoll_event;

// original: Poll_wrap (htslib/ref_cache/poll_wrap_epoll.c:43)
//
// The C version stored each registered `Pw_item` in an intrusive memory pool
// and stashed the raw item pointer in epoll's opaque `u64` field. Here the
// items live in an owned arena (`items`); the index into that arena is what we
// store in epoll's `u64` field and recover on wakeup. `free` holds the indices
// of slots that have been removed and may be reused.
pub struct Poll_wrap {
    items: Vec<Option<Pw_item>>,
    free: Vec<usize>,
    epfd: i32,
    debug: bool,
}

// original: pw_init (htslib/ref_cache/poll_wrap_epoll.c:49)
pub fn ref_cache_poll_wrap_epoll_c_49_pw_init(debug: bool) -> Option<Box<Poll_wrap>> {
    let epfd = unsafe { libc::epoll_create(INIT_EPOLL_SIZE) };
    if epfd < 0 {
        eprintln!("epoll_create: {}", std::io::Error::last_os_error());
        return None;
    }

    Some(Box::new(Poll_wrap {
        items: Vec::new(),
        free: Vec::new(),
        epfd,
        debug,
    }))
}

// original: pw_close (htslib/ref_cache/poll_wrap_epoll.c:72)
pub fn ref_cache_poll_wrap_epoll_c_72_pw_close(pw: Option<Box<Poll_wrap>>) {
    let Some(pw) = pw else {
        return;
    };
    unsafe { libc::close(pw.epfd) };
    // `pw` (and its owned `items` arena) drops here.
}

impl Poll_wrap {
    // Read-only access to a registered item by its arena index. The epoll `u64`
    // field carries this index, so an event source is mapped back to its
    // `(fd, fd_type, userp)` triple through here.
    pub fn item_at(&self, idx: usize) -> Option<&Pw_item> {
        self.items.get(idx).and_then(|i| i.as_ref())
    }
}

// original: pw_register (htslib/ref_cache/poll_wrap_epoll.c:78)
//
// Returns the arena index of the freshly registered item, which callers use as
// the handle to refer to it later (mod/remove/wait).
pub fn ref_cache_poll_wrap_epoll_c_78_pw_register(
    pw: &mut Poll_wrap,
    fd: i32,
    fd_type: Pw_fd_type,
    init_events: u32,
    userp: usize,
) -> Option<usize> {
    if pw.debug {
        eprintln!(
            "pw_register({:p}, {}, {}, 0x{:04x}, {})",
            pw as *const Poll_wrap,
            fd,
            fd_type as i32,
            init_events,
            userp,
        );
    }

    let item = Pw_item { fd, fd_type, userp };
    let idx = match pw.free.pop() {
        Some(idx) => {
            pw.items[idx] = Some(item);
            idx
        }
        None => {
            pw.items.push(Some(item));
            pw.items.len() - 1
        }
    };

    let mut event: libc::epoll_event = unsafe { std::mem::zeroed() };
    event.events = init_events;
    event.u64 = idx as u64;

    if unsafe { libc::epoll_ctl(pw.epfd, libc::EPOLL_CTL_ADD, fd, &mut event) } != 0 {
        eprintln!("epoll_ctl: {}", std::io::Error::last_os_error());
        pw.items[idx] = None;
        pw.free.push(idx);
        return None;
    }

    Some(idx)
}

// original: pw_mod (htslib/ref_cache/poll_wrap_epoll.c:106)
pub fn ref_cache_poll_wrap_epoll_c_106_pw_mod(
    pw: &mut Poll_wrap,
    item: usize,
    events: u32,
) -> i32 {
    let Some(fd) = pw.items.get(item).and_then(|i| i.as_ref()).map(|i| i.fd) else {
        return -1;
    };

    if pw.debug {
        eprintln!("pw_mod({:p}, {}, 0x{:04x})", pw as *const Poll_wrap, fd, events);
    }

    let mut event: libc::epoll_event = unsafe { std::mem::zeroed() };
    event.events = events;
    event.u64 = item as u64;

    unsafe { libc::epoll_ctl(pw.epfd, libc::EPOLL_CTL_MOD, fd, &mut event) }
}

// original: pw_wait (htslib/ref_cache/poll_wrap_epoll.c:120)
pub fn ref_cache_poll_wrap_epoll_c_120_pw_wait(
    pw: &mut Poll_wrap,
    events: &mut [Pw_events],
    timeout: i32,
) -> i32 {
    unsafe {
        libc::epoll_wait(
            pw.epfd,
            events.as_mut_ptr(),
            events.len() as i32,
            timeout,
        )
    }
}

// original: pw_remove (htslib/ref_cache/poll_wrap_epoll.c:126)
pub fn ref_cache_poll_wrap_epoll_c_126_pw_remove(
    pw: &mut Poll_wrap,
    item: usize,
    do_close: bool,
) -> i32 {
    let Some(fd) = pw.items.get(item).and_then(|i| i.as_ref()).map(|i| i.fd) else {
        return -1;
    };

    if pw.debug {
        eprintln!(
            "pw_remove({:p}, {}{})",
            pw as *const Poll_wrap,
            fd,
            if do_close { ", close" } else { "" },
        );
    }

    let res = if do_close {
        unsafe { libc::close(fd) }
    } else {
        let mut dummy: libc::epoll_event = unsafe { std::mem::zeroed() };
        unsafe { libc::epoll_ctl(pw.epfd, libc::EPOLL_CTL_DEL, fd, &mut dummy) }
    };

    pw.items[item] = None;
    pw.free.push(item);
    res
}
