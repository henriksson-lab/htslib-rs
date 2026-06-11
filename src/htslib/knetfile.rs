pub const KNF_TYPE_LOCAL: i32 = 1;
pub const KNF_TYPE_FTP: i32 = 2;
pub const KNF_TYPE_HTTP: i32 = 3;

// original: knetFile_s (htslib/htslib/knetfile.h:61)
pub struct knetFile_s {
    pub type_: i32,
    pub fd: i32,
    pub offset: i64,
    pub host: Vec<u8>,
    pub port: Vec<u8>,
    pub ctrl_fd: i32,
    pub pasv_ip: [i32; 4],
    pub pasv_port: i32,
    pub max_response: i32,
    pub no_reconnect: i32,
    pub is_ready: i32,
    pub response: Vec<u8>,
    pub retr: Vec<u8>,
    pub size_cmd: Vec<u8>,
    pub seek_offset: i64,
    pub file_size: i64,
    pub path: Vec<u8>,
    pub http_host: Vec<u8>,
}

pub type knetFile = knetFile_s;
