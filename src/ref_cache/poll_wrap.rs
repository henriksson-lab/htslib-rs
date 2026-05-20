use std::ffi::{c_int, c_void};

#[repr(C)]
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

pub const PW_IN: c_int = libc::POLLIN as c_int;
pub const PW_OUT: c_int = libc::POLLOUT as c_int;
pub const PW_ERR: c_int = libc::POLLERR as c_int;
pub const PW_HUP: c_int = libc::POLLHUP as c_int;
pub const PW_EDGE: c_int = 0;

// original: Pw_events (htslib/ref_cache/poll_wrap.h:75)
#[repr(C)]
pub struct Pw_events {
    pub events: u32,
    pub item: *mut Pw_item,
}

// original: Pw_item (htslib/ref_cache/poll_wrap.h:89)
#[repr(C)]
pub struct Pw_item {
    pub fd: c_int,
    pub fd_type: Pw_fd_type,
    pub userp: *mut c_void,
}
