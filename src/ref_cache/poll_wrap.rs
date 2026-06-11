#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pw_fd_type {
    SV_LISTENER = 1,
    SV_UPSTREAM = 2,
    SV_LOG = 3,
    SV_CLIENT = 4,
    SV_US_PIPE = 5,
    US_COMMAND = 0x10000,
    US_CURL = 0x10001,
    US_PIPE = 0x10002,
    US_LIVE = 0x10003,
    MAIN_LOG_RD = 0x20000,
    MAIN_SIG = 0x20001,
}

// Genuine OS poll() flag bits; kept as the kernel-level constants.
pub const PW_IN: i32 = libc::POLLIN as i32;
pub const PW_OUT: i32 = libc::POLLOUT as i32;
pub const PW_ERR: i32 = libc::POLLERR as i32;
pub const PW_HUP: i32 = libc::POLLHUP as i32;
pub const PW_EDGE: i32 = 0;

// original: Pw_events (htslib/ref_cache/poll_wrap.h:75)
pub struct Pw_events {
    pub events: u32,
    // Snapshot of the registered Pw_item (was *mut Pw_item). Pw_item is Copy,
    // so events carry it by value rather than borrowing the owning arena.
    pub item: Option<Pw_item>,
}

// original: Pw_item (htslib/ref_cache/poll_wrap.h:89)
#[derive(Clone, Copy)]
pub struct Pw_item {
    pub fd: i32,
    pub fd_type: Pw_fd_type,
    // Client arena index (was void *userp). Callbacks recover the concrete
    // entry via `userp as usize` against the owning arena.
    pub userp: usize,
}
