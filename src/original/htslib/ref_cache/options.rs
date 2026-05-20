use std::ffi::{c_char, c_int};

pub const FIRST_SD_LISTEN_FD: c_int = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DaemonType {
    not_a_daemon = 0,
    sysv_daemon = 1,
    systemd_socket_service = 2,
}

// original: MatchAddr (htslib/ref_cache/options.h:45)
#[repr(C)]
pub struct MatchAddr {
    pub family: libc::sa_family_t,
    pub mask_bytes: u8,
    pub mask: u8,
    pub addr: [libc::c_uchar; 16],
}

// original: Options (htslib/ref_cache/options.h:52)
#[repr(C)]
pub struct Options {
    pub cache_dir: *const c_char,
    pub log_dir: *const c_char,
    pub error_log_file: *const c_char,
    pub log: *mut libc::FILE,
    pub upstream_url: *const c_char,
    pub upstream_url_len: usize,
    pub match_addrs: *mut MatchAddr,
    pub num_match_addrs: usize,
    pub match_addrs_size: usize,
    pub first_ip6: usize,
    pub max_log_sz: libc::off_t,
    pub cache_fd: c_int,
    pub listen_fds: c_int,
    pub daemon: DaemonType,
    pub port: u16,
    pub nlogs: u16,
    pub max_kids: u16,
    pub verbosity: u8,
    pub no_log: u8,
}
