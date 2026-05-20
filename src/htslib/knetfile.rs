use std::ffi::{c_char, c_int};

pub const KNF_TYPE_LOCAL: c_int = 1;
pub const KNF_TYPE_FTP: c_int = 2;
pub const KNF_TYPE_HTTP: c_int = 3;

// original: knetFile_s (htslib/htslib/knetfile.h:61)
#[repr(C)]
pub struct knetFile_s {
    pub type_: c_int,
    pub fd: c_int,
    pub offset: i64,
    pub host: *mut c_char,
    pub port: *mut c_char,
    pub ctrl_fd: c_int,
    pub pasv_ip: [c_int; 4],
    pub pasv_port: c_int,
    pub max_response: c_int,
    pub no_reconnect: c_int,
    pub is_ready: c_int,
    pub response: *mut c_char,
    pub retr: *mut c_char,
    pub size_cmd: *mut c_char,
    pub seek_offset: i64,
    pub file_size: i64,
    pub path: *mut c_char,
    pub http_host: *mut c_char,
}

pub type knetFile = knetFile_s;
