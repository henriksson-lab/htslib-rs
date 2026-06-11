use std::ffi::c_int;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchAddr {
    pub family: libc::sa_family_t,
    pub mask_bytes: u8,
    pub mask: u8,
    pub addr: [u8; 16],
}

// original: Options (htslib/ref_cache/options.h:52)
pub struct Options {
    pub cache_dir: Option<Vec<u8>>,
    pub log_dir: Option<Vec<u8>>,
    pub error_log_file: Option<Vec<u8>>,
    // genuine OS stdio stream handle; used at libc fileno/fflush boundaries.
    pub log: *mut libc::FILE,
    // C string carried as owned bytes (no trailing NUL); length is `.len()`.
    pub upstream_url: Option<Vec<u8>>,
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
    pub match_addrs_storage: Vec<MatchAddr>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            cache_dir: None,
            log_dir: None,
            error_log_file: None,
            log: std::ptr::null_mut(),
            upstream_url: None,
            first_ip6: 0,
            max_log_sz: 0,
            cache_fd: -1,
            listen_fds: 0,
            daemon: DaemonType::not_a_daemon,
            port: 0,
            nlogs: 0,
            max_kids: 0,
            verbosity: 0,
            no_log: 0,
            match_addrs_storage: Vec::new(),
        }
    }
}
