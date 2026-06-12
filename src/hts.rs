use std::{
    ffi::{c_char, c_void, CStr},
    path::{Path, PathBuf},
    ptr::NonNull,
};

use super::bgzf::{
    bgzf_check_EOF, bgzf_close, bgzf_flush, bgzf_getline, bgzf_hfile, bgzf_hopen, bgzf_mt,
    bgzf_open, bgzf_read, bgzf_seek, bgzf_set_cache_size, bgzf_thread_pool, bgzf_useek, bgzf_utell,
    bgzf_write,
};
use super::cram::{cram_check_EOF, cram_dopen};
use super::hfile::{
    hclose_abruptly, hfile_set_blksize, hisremote, hopen, hpeek, htslib_hfile_h_134_herrno,
    htslib_hfile_h_195_hgetln, htslib_hfile_h_247_hread, htslib_hfile_h_292_hwrite,
};
use super::{path_bytes, path_from_bytes};

use crate::htslib_rs::cram::{
    cram_fd_layout, cram_index_last, cram_index_query, cram_index_query_last,
};

const BGZF_HTS_OPEN_FAST_BAM_READ: u32 = 1 << 16;

pub type hts_pos_t = i64;
pub const HTS_POS_MAX: hts_pos_t = ((i32::MAX as hts_pos_t) << 32) | i32::MAX as hts_pos_t;
pub type htsFormatCategory = u32;
pub const HTS_FORMAT_UNKNOWN_CATEGORY: htsFormatCategory = 0;
pub const HTS_FORMAT_SEQUENCE_DATA: htsFormatCategory = 1;
pub const HTS_FORMAT_VARIANT_DATA: htsFormatCategory = 2;
pub const HTS_FORMAT_INDEX_FILE: htsFormatCategory = 3;
pub const HTS_FORMAT_REGION_LIST: htsFormatCategory = 4;
pub type htsExactFormat = u32;
pub type htsCompression = u32;
pub type hts_fmt_option = u32;
pub struct htsThreadPool {
    pub pool: *mut crate::htslib_rs::thread_pool::hts_tpool,
    pub qsize: i32,
}
pub union hts_opt_val {
    pub i: i32,
    pub s: *mut c_char,
}

pub struct hts_opt {
    pub arg: NonNull<c_char>,
    pub opt: u32,
    pub val: hts_opt_val,
    pub next: Option<NonNull<hts_opt>>,
    _arg_buf: Box<[c_char]>,
}
pub const HTS_FORMAT_UNKNOWN_FORMAT: htsExactFormat = 0;
pub const HTS_FORMAT_BINARY_FORMAT: htsExactFormat = 1;
pub const HTS_FORMAT_TEXT_FORMAT: htsExactFormat = 2;
pub const HTS_FORMAT_SAM: htsExactFormat = 3;
pub const HTS_FORMAT_BAM: htsExactFormat = 4;
pub const HTS_FORMAT_BAI: htsExactFormat = 5;
pub const HTS_FORMAT_CRAM: htsExactFormat = 6;
pub const HTS_FORMAT_CRAI_EXACT: htsExactFormat = 7;
pub const HTS_FORMAT_VCF: htsExactFormat = 8;
pub const HTS_FORMAT_BCF: htsExactFormat = 9;
pub const HTS_FORMAT_CSI: htsExactFormat = 10;
pub const HTS_FORMAT_GZI: htsExactFormat = 11;
pub const HTS_FORMAT_TBI: htsExactFormat = 12;
pub const HTS_FORMAT_BED: htsExactFormat = 13;
pub const HTS_FORMAT_HTSGET: htsExactFormat = 14;
pub const HTS_FORMAT_EMPTY_FORMAT: htsExactFormat = 15;
pub const HTS_FORMAT_FASTA_FORMAT: htsExactFormat = 16;
pub const HTS_FORMAT_FASTQ_FORMAT: htsExactFormat = 17;
pub const HTS_FORMAT_FAI_FORMAT: htsExactFormat = 18;
pub const HTS_FORMAT_FQI_FORMAT: htsExactFormat = 19;
pub const HTS_FORMAT_CRYPT4GH_FORMAT: htsExactFormat = 20;
pub const HTS_FORMAT_D4_FORMAT: htsExactFormat = 21;
pub const HTS_COMPRESSION_NO_COMPRESSION: htsCompression = 0;
pub type htsLogLevel = i32;
pub const HTS_LOG_OFF: htsLogLevel = 0;
pub const HTS_LOG_ERROR: htsLogLevel = 1;
pub const HTS_LOG_WARNING: htsLogLevel = 3;
pub const HTS_LOG_INFO: htsLogLevel = 4;
pub const HTS_LOG_DEBUG: htsLogLevel = 5;
pub const HTS_LOG_TRACE: htsLogLevel = 6;
pub const HTS_FMT_CSI: i32 = 0;
pub const HTS_FMT_BAI: i32 = 1;
pub const HTS_FMT_CRAI: i32 = 3;
pub const HTS_FMT_FAI: i32 = 4;
pub const HTS_IDX_SAVE_REMOTE: i32 = 1;
pub const HTS_IDX_SILENT_FAIL: i32 = 2;
// Path delimiter inserted between a file URL and an inline index path
// (see htslib/hts.h: "##idx##").
pub const HTS_IDX_DELIM: &[u8; 8] = b"##idx##\0";
pub const HTS_COMPRESSION_RAZF: htsCompression = 5;
pub const HTS_COMPRESSION_XZ: htsCompression = 6;
pub const HTS_COMPRESSION_ZSTD: htsCompression = 7;
pub const HTS_IDX_NOCOOR: i32 = -2;
pub const HTS_IDX_START: i32 = -3;
pub const HTS_IDX_REST: i32 = -4;
pub const HTS_IDX_NONE: i32 = -5;
pub const HTS_RESIZE_CLEAR: i32 = 1;
pub const KS_SEP_LINE: i32 = 2;
pub const CRAM_OPT_RANGE_NOSEEK: hts_fmt_option = 23;
pub const CRAM_OPT_USE_TOK: hts_fmt_option = 24;
pub const CRAM_OPT_USE_FQZ: hts_fmt_option = 25;
pub const CRAM_OPT_USE_ARITH: hts_fmt_option = 26;
pub const CRAM_OPT_POS_DELTA: hts_fmt_option = 27;
pub const HTS_OPT_FILTER: hts_fmt_option = 105;
pub const HTS_OPT_PROFILE: hts_fmt_option = 106;
// Remaining htsCompression values (values from the C enum).
pub const HTS_COMPRESSION_GZIP: htsCompression = 1;
pub const HTS_COMPRESSION_BGZF: htsCompression = 2;
pub const HTS_COMPRESSION_CUSTOM: htsCompression = 3;
pub const HTS_COMPRESSION_BZIP2: htsCompression = 4;
// Tabix index format tag.
pub const HTS_FMT_TBI: i32 = 2;
// hts_fmt_option values used by the core open/option paths.
pub const CRAM_OPT_DECODE_MD: hts_fmt_option = 0;
pub const CRAM_OPT_PREFIX: hts_fmt_option = 1;
pub const CRAM_OPT_VERSION: hts_fmt_option = 6;
pub const CRAM_OPT_REFERENCE: hts_fmt_option = 9;
pub const HTS_OPT_COMPRESSION_LEVEL: hts_fmt_option = 100;
pub const HTS_OPT_NTHREADS: hts_fmt_option = 101;
pub const HTS_OPT_CACHE_SIZE: hts_fmt_option = 103;
pub const HTS_OPT_BLOCK_SIZE: hts_fmt_option = 104;
pub const HTS_OPT_THREAD_POOL: hts_fmt_option = 102;
pub const CRAM_OPT_VERBOSITY: hts_fmt_option = 2;
pub const CRAM_OPT_SEQS_PER_SLICE: hts_fmt_option = 3;
pub const CRAM_OPT_SLICES_PER_CONTAINER: hts_fmt_option = 4;
pub const CRAM_OPT_EMBED_REF: hts_fmt_option = 7;
pub const CRAM_OPT_IGNORE_MD5: hts_fmt_option = 8;
pub const CRAM_OPT_MULTI_SEQ_PER_SLICE: hts_fmt_option = 10;
pub const CRAM_OPT_NO_REF: hts_fmt_option = 11;
pub const CRAM_OPT_USE_BZIP2: hts_fmt_option = 12;
pub const CRAM_OPT_NTHREADS: hts_fmt_option = 14;
pub const CRAM_OPT_THREAD_POOL: hts_fmt_option = 15;
pub const CRAM_OPT_USE_LZMA: hts_fmt_option = 16;
pub const CRAM_OPT_USE_RANS: hts_fmt_option = 17;
pub const CRAM_OPT_REQUIRED_FIELDS: hts_fmt_option = 18;
pub const CRAM_OPT_LOSSY_NAMES: hts_fmt_option = 19;
pub const CRAM_OPT_BASES_PER_SLICE: hts_fmt_option = 20;
pub const CRAM_OPT_STORE_MD: hts_fmt_option = 21;
pub const CRAM_OPT_STORE_NM: hts_fmt_option = 22;
pub const HTS_PROFILE_FAST: i32 = 0;
pub const HTS_PROFILE_NORMAL: i32 = 1;
pub const HTS_PROFILE_SMALL: i32 = 2;
pub const HTS_PROFILE_ARCHIVE: i32 = 3;
pub const HTS_PARSE_THOUSANDS_SEP: i32 = 1;
pub const HTS_PARSE_ONE_COORD: i32 = 2;
pub const HTS_PARSE_LIST: i32 = 4;
pub const HTS_FEATURE_CONFIGURE: u32 = 1;
pub const HTS_FEATURE_PLUGINS: u32 = 2;
pub const HTS_FEATURE_LIBCURL: u32 = 1 << 10;
pub const HTS_FEATURE_S3: u32 = 1 << 11;
pub const HTS_FEATURE_GCS: u32 = 1 << 12;
pub const HTS_FEATURE_LIBDEFLATE: u32 = 1 << 20;
pub const HTS_FEATURE_LZMA: u32 = 1 << 21;
pub const HTS_FEATURE_BZIP2: u32 = 1 << 22;
pub const HTS_FEATURE_HTSCODECS: u32 = 1 << 23;
pub const HTS_FEATURE_CC: u32 = 1 << 27;
pub const HTS_FEATURE_CFLAGS: u32 = 1 << 28;
pub const HTS_FEATURE_CPPFLAGS: u32 = 1 << 29;
pub const HTS_FEATURE_LDFLAGS: u32 = 1 << 30;
pub const HTS_MAX_EXT_LEN: usize = 9;
pub static mut hts_verbose: i32 = HTS_LOG_WARNING;
pub type hts_name2id_f =
    Option<unsafe extern "C" fn(data: *mut c_void, name: *const c_char) -> i32>;
pub type hts_id2name_f =
    Option<unsafe extern "C" fn(data: *mut c_void, id: i32) -> *const c_char>;
pub type hts_readrec_func = Option<
    unsafe extern "C" fn(
        fp: *mut BGZF,
        data: *mut c_void,
        r: *mut c_void,
        tid: *mut i32,
        beg: *mut hts_pos_t,
        end: *mut hts_pos_t,
    ) -> i32,
>;
pub type hts_seek_func =
    Option<unsafe extern "C" fn(fp: *mut c_void, offset: i64, where_: i32) -> i32>;
pub type hts_tell_func = Option<unsafe extern "C" fn(fp: *mut c_void) -> i64>;
pub type hts_itr_query_func = Option<
    unsafe extern "C" fn(
        idx: *const hts_idx_t,
        tid: i32,
        beg: hts_pos_t,
        end: hts_pos_t,
        readrec: hts_readrec_func,
    ) -> *mut hts_itr_t,
>;
pub type hts_itr_multi_query_func =
    Option<unsafe extern "C" fn(idx: *const hts_idx_t, iter: *mut hts_itr_t) -> i32>;
pub type kgets_func = Option<
    unsafe extern "C" fn(buffer: *mut u8, size: i32, stream: *mut ()) -> *mut u8,
>;
pub type kgets_func2 =
    Option<unsafe extern "C" fn(buffer: *mut u8, size: usize, stream: *mut ()) -> isize>;
pub type hts_expr_sym_func = Option<
    unsafe extern "C" fn(
        data: *mut c_void,
        str_: *mut c_char,
        end: *mut *mut c_char,
        res: *mut crate::htslib_rs::hts_expr::hts_expr_val_t,
    ) -> i32,
>;

#[repr(C)]
pub struct hts_json_token {
    pub type_: c_char,
    pub str_: *mut c_char,
}

pub unsafe fn hts_version() -> *const c_char {
    c"1.23.1-24-g7c895563".as_ptr()
}

pub unsafe fn hts_features() -> u32 {
    HTS_FEATURE_HTSCODECS
}

pub unsafe fn hts_test_feature(id: u32) -> *const c_char {
    let feat = hts_features();
    match id {
        HTS_FEATURE_CONFIGURE => {
            if feat & HTS_FEATURE_CONFIGURE != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_PLUGINS => {
            if feat & HTS_FEATURE_PLUGINS != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_LIBCURL => {
            if feat & HTS_FEATURE_LIBCURL != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_S3 => {
            if feat & HTS_FEATURE_S3 != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_GCS => {
            if feat & HTS_FEATURE_GCS != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_LIBDEFLATE => {
            if feat & HTS_FEATURE_LIBDEFLATE != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_BZIP2 => {
            if feat & HTS_FEATURE_BZIP2 != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_LZMA => {
            if feat & HTS_FEATURE_LZMA != 0 {
                c"yes".as_ptr()
            } else {
                std::ptr::null()
            }
        }
        HTS_FEATURE_HTSCODECS => c"builtin".as_ptr(),
        HTS_FEATURE_CC | HTS_FEATURE_CFLAGS | HTS_FEATURE_LDFLAGS | HTS_FEATURE_CPPFLAGS => {
            c"".as_ptr()
        }
        _ => std::ptr::null(),
    }
}

pub unsafe fn hts_feature_string() -> *const c_char {
    c"build=Makefile libcurl=no S3=no GCS=no libdeflate=no lzma=no bzip2=no plugins=no htscodecs=builtin".as_ptr()
}

pub unsafe fn hts_str2int(
    in_: *const u8,
    end: *mut *mut u8,
    bits: i32,
    failed: *mut i32,
) -> i64 {
    let mut n = 0u64;
    let mut limit = (1u64 << (bits - 1)) - 1;
    let mut fast = ((bits - 1) as u32 * 1000 / 3322) + 1;
    let mut v = in_;
    let neg;

    match *v {
        b'-' => {
            limit += 1;
            neg = true;
            v = v.add(1);
            while {
                fast = fast.wrapping_sub(1);
                fast != 0 && *v >= b'0' && *v <= b'9'
            } {
                n = n * 10 + (*v - b'0') as u64;
                v = v.add(1);
            }
        }
        b'+' => {
            v = v.add(1);
            neg = false;
            while {
                fast = fast.wrapping_sub(1);
                fast != 0 && *v >= b'0' && *v <= b'9'
            } {
                n = n * 10 + (*v - b'0') as u64;
                v = v.add(1);
            }
        }
        _ => {
            neg = false;
            while {
                fast = fast.wrapping_sub(1);
                fast != 0 && *v >= b'0' && *v <= b'9'
            } {
                n = n * 10 + (*v - b'0') as u64;
                v = v.add(1);
            }
        }
    }

    if *v >= b'0' && fast == 0 {
        let limit_d_10 = limit / 10;
        let limit_m_10 = limit - 10 * limit_d_10;
        while *v >= b'0' && *v <= b'9' {
            let d = (*v - b'0') as u64;
            if n < limit_d_10 || (n == limit_d_10 && d <= limit_m_10) {
                n = n * 10 + d;
                v = v.add(1);
            } else {
                while *v >= b'0' && *v <= b'9' {
                    v = v.add(1);
                }
                n = limit;
                *failed = 1;
                break;
            }
        }
    }

    *end = v.cast_mut();
    if neg {
        (n as i64).wrapping_neg()
    } else {
        n as i64
    }
}

pub unsafe fn hts_str2uint(
    in_: *const u8,
    end: *mut *mut u8,
    bits: i32,
    failed: *mut i32,
) -> u64 {
    let mut n = 0u64;
    let limit = if bits < 64 {
        (1u64 << bits) - 1
    } else {
        u64::MAX
    };
    let mut v = in_;
    let mut fast = (bits as u32 * 1000 / 3322) + 1;

    if *v == b'+' {
        v = v.add(1);
    }

    while {
        fast = fast.wrapping_sub(1);
        fast != 0 && *v >= b'0' && *v <= b'9'
    } {
        n = n * 10 + (*v - b'0') as u64;
        v = v.add(1);
    }

    if *v >= b'0' && *v <= b'9' && fast == 0 {
        let limit_d_10 = limit / 10;
        let limit_m_10 = limit - 10 * limit_d_10;
        while *v >= b'0' && *v <= b'9' {
            let d = (*v - b'0') as u64;
            if n < limit_d_10 || (n == limit_d_10 && d <= limit_m_10) {
                n = n * 10 + d;
                v = v.add(1);
            } else {
                while *v >= b'0' && *v <= b'9' {
                    v = v.add(1);
                }
                n = limit;
                *failed = 1;
                break;
            }
        }
    }

    *end = v.cast_mut();
    n
}

pub unsafe fn hts_str2dbl(in_: *const u8, end: *mut *mut u8, failed: *mut i32) -> f64 {
    let mut n = 0u64;
    let mut max_len = 15;
    let mut v = in_;
    let mut neg = false;
    let mut point = -1isize;
    const D: [f64; 22] = [
        1.0, 1.0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15,
        1e16, 1e17, 1e18, 1e19, 1e20,
    ];

    while libc::isspace(*v as i32) != 0 {
        v = v.add(1);
    }
    if *v == b'-' {
        neg = true;
        v = v.add(1);
    } else if *v == b'+' {
        v = v.add(1);
    }

    match *v {
        b'1'..=b'9' => {}
        b'0' if *v.add(1) != b'x' && *v.add(1) != b'X' => {}
        _ => {
            let d = libc::strtod(in_.cast(), end.cast());
            if *end == in_.cast_mut() {
                *failed = 1;
            }
            return d;
        }
    }

    while *v == b'0' {
        v = v.add(1);
    }
    let start = v;

    while {
        max_len -= 1;
        max_len != 0 && *v >= b'0' && *v <= b'9'
    } {
        n = n * 10 + (*v - b'0') as u64;
        v = v.add(1);
    }
    if max_len != 0 && *v == b'.' {
        point = v.offset_from(start);
        v = v.add(1);
        while {
            max_len -= 1;
            max_len != 0 && *v >= b'0' && *v <= b'9'
        } {
            n = n * 10 + (*v - b'0') as u64;
            v = v.add(1);
        }
    }
    if point < 0 {
        point = v.offset_from(start);
    }

    if max_len == 0 || *v == b'e' || *v == b'E' {
        let d = libc::strtod(in_.cast(), end.cast());
        if *end == in_.cast_mut() {
            *failed = 1;
        }
        return d;
    }

    *end = v.cast_mut();
    let d = n as f64 / D[(v.offset_from(start) - point) as usize];
    if neg {
        -d
    } else {
        d
    }
}

pub unsafe fn dehex(c: u8) -> i32 {
    if c.is_ascii_lowercase() && c <= b'f' {
        (c - b'a' + 10) as i32
    } else if c.is_ascii_uppercase() && c <= b'F' {
        (c - b'A' + 10) as i32
    } else if c.is_ascii_digit() {
        (c - b'0') as i32
    } else {
        -1
    }
}

pub use crate::htslib_rs::textutils::{
    debase64, hts_base64_decoded_length, hts_decode_base64, hts_decode_percent,
};

pub unsafe fn encode_utf8(mut s: *mut u8, x: u32) -> *mut u8 {
    if x >= 0x10000 {
        *s = (0xf0 | (x >> 18)) as u8;
        s = s.add(1);
        *s = (0x80 | ((x >> 12) & 0x3f)) as u8;
        s = s.add(1);
        *s = (0x80 | ((x >> 6) & 0x3f)) as u8;
        s = s.add(1);
        *s = (0x80 | (x & 0x3f)) as u8;
        s = s.add(1);
    } else if x >= 0x800 {
        *s = (0xe0 | (x >> 12)) as u8;
        s = s.add(1);
        *s = (0x80 | ((x >> 6) & 0x3f)) as u8;
        s = s.add(1);
        *s = (0x80 | (x & 0x3f)) as u8;
        s = s.add(1);
    } else if x >= 0x80 {
        *s = (0xc0 | (x >> 6)) as u8;
        s = s.add(1);
        *s = (0x80 | (x & 0x3f)) as u8;
        s = s.add(1);
    } else {
        *s = x as u8;
        s = s.add(1);
    }
    s
}

pub use crate::htslib_rs::textutils::sscan_string;

pub unsafe fn token_type(token: *mut hts_json_token) -> c_char {
    let s = (*token).str_;
    match *s as u8 {
        b'f' => {
            if libc::strcmp(s, c"false".as_ptr()) == 0 {
                b'b' as c_char
            } else {
                b'?' as c_char
            }
        }
        b'n' => {
            if libc::strcmp(s, c"null".as_ptr()) == 0 {
                b'.' as c_char
            } else {
                b'?' as c_char
            }
        }
        b't' => {
            if libc::strcmp(s, c"true".as_ptr()) == 0 {
                b'b' as c_char
            } else {
                b'?' as c_char
            }
        }
        b'-' | b'0'..=b'9' => b'n' as c_char,
        _ => b'?' as c_char,
    }
}

pub use crate::htslib_rs::textutils::{
    hts_json_alloc_token, hts_json_free_token, hts_json_snext, hts_json_token_str,
    hts_json_token_type,
};

unsafe fn fscan_string(fp: *mut hFILE, d: *mut kstring_t) -> i32 {
    let mut e: u32 = 0;
    loop {
        let mut c = super::hfile::htslib_hfile_h_163_hgetc(fp);
        if c == libc::EOF {
            break;
        }
        match c as u8 {
            b'\\' => {
                c = super::hfile::htslib_hfile_h_163_hgetc(fp);
                if c == libc::EOF {
                    return if e == 0 { 0 } else { -1 };
                }
                match c as u8 {
                    b'b' => e |= (kputc(b'\x08' as i32, &mut *d) < 0) as u32,
                    b'f' => e |= (kputc(b'\x0c' as i32, &mut *d) < 0) as u32,
                    b'n' => e |= (kputc(b'\n' as i32, &mut *d) < 0) as u32,
                    b'r' => e |= (kputc(b'\r' as i32, &mut *d) < 0) as u32,
                    b't' => e |= (kputc(b'\t' as i32, &mut *d) < 0) as u32,
                    b'u' => {
                        let c1 = super::hfile::htslib_hfile_h_163_hgetc(fp);
                        let d1 = if c1 != libc::EOF {
                            dehex(c1 as u8)
                        } else {
                            -1
                        };
                        let c2 = if c1 != libc::EOF && d1 >= 0 {
                            super::hfile::htslib_hfile_h_163_hgetc(fp)
                        } else {
                            libc::EOF
                        };
                        let d2 = if c2 != libc::EOF {
                            dehex(c2 as u8)
                        } else {
                            -1
                        };
                        let c3 = if c2 != libc::EOF && d2 >= 0 {
                            super::hfile::htslib_hfile_h_163_hgetc(fp)
                        } else {
                            libc::EOF
                        };
                        let d3 = if c3 != libc::EOF {
                            dehex(c3 as u8)
                        } else {
                            -1
                        };
                        let c4 = if c3 != libc::EOF && d3 >= 0 {
                            super::hfile::htslib_hfile_h_163_hgetc(fp)
                        } else {
                            libc::EOF
                        };
                        let d4 = if c4 != libc::EOF {
                            dehex(c4 as u8)
                        } else {
                            -1
                        };
                        if d1 >= 0 && d2 >= 0 && d3 >= 0 && d4 >= 0 {
                            let mut buf = [0u8; 8];
                            let lim = encode_utf8(
                                buf.as_mut_ptr(),
                                ((d1 << 12) | (d2 << 8) | (d3 << 4) | d4) as u32,
                            );
                            let len = lim.offset_from(buf.as_ptr()) as usize;
                            let buf_bytes = std::slice::from_raw_parts(buf.as_ptr(), len);
                            e |= (kputsn(buf_bytes, len, &mut *d) < 0) as u32;
                        }
                    }
                    _ => e |= (kputc(c, &mut *d) < 0) as u32,
                }
            }
            b'"' => return if e == 0 { 0 } else { -1 },
            _ => e |= (kputc(c, &mut *d) < 0) as u32,
        }
    }
    if e == 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn hts_json_fnext(
    fp: *mut hFILE,
    token: *mut hts_json_token,
    kstr: *mut kstring_t,
) -> c_char {
    loop {
        let c = super::hfile::htslib_hfile_h_163_hgetc(fp);
        match c {
            x if x == b' ' as i32
                || x == b'\t' as i32
                || x == b'\r' as i32
                || x == b'\n' as i32
                || x == b',' as i32
                || x == b':' as i32 =>
            {
                continue;
            }
            x if x == libc::EOF => {
                (*token).type_ = 0;
                return (*token).type_;
            }
            x if x == b'{' as i32
                || x == b'[' as i32
                || x == b'}' as i32
                || x == b']' as i32 =>
            {
                (*token).type_ = c as c_char;
                return (*token).type_;
            }
            x if x == b'"' as i32 => {
                // SEAM: kstr may be an unallocated (NULL-pointer) calloc'd kstring;
                // truncate(0) empties it without the NULL-pointer abort that
                // Vec::clear() would hit (see hts_getline).
                (*kstr).data.truncate(0);
                fscan_string(fp, kstr);
                if (*kstr).data.is_empty() {
                    kputsn(b"", 0, &mut *kstr);
                }
                (*token).str_ = (*kstr).data.as_mut_ptr().cast();
                (*token).type_ = b's' as c_char;
                return (*token).type_;
            }
            _ => {
                (*kstr).data.truncate(0);
                kputc(c, &mut *kstr);
                let mut peek: c_char = 0;
                while hpeek(fp, (&mut peek as *mut c_char).cast(), 1) == 1
                    && libc::strchr(c" \t\r\n,]}".as_ptr(), peek as i32).is_null()
                {
                    let nc = super::hfile::htslib_hfile_h_163_hgetc(fp);
                    if nc == libc::EOF {
                        break;
                    }
                    kputc(nc, &mut *kstr);
                }
                (*token).str_ = (*kstr).data.as_mut_ptr().cast();
                (*token).type_ = token_type(token);
                return (*token).type_;
            }
        }
    }
}

unsafe fn fnext(arg1: *mut (), arg2: *mut (), token: *mut hts_json_token) -> c_char {
    hts_json_fnext(arg1.cast(), token, arg2.cast())
}

pub unsafe fn hts_json_fskip_value(fp: *mut hFILE, type_: c_char) -> c_char {
    let mut str_: kstring_t = kstring_t::default();
    let ret = skip_value(
        type_,
        Some(fnext),
        fp.cast(),
        (&mut str_ as *mut kstring_t).cast(),
    );
    ret
}

pub type hts_json_nextfn =
    Option<unsafe fn(arg1: *mut (), arg2: *mut (), token: *mut hts_json_token) -> c_char>;

pub unsafe fn skip_value(
    type_: c_char,
    next: hts_json_nextfn,
    arg1: *mut (),
    arg2: *mut (),
) -> c_char {
    let mut token = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };
    let first = if type_ != 0 {
        type_
    } else if let Some(next) = next {
        next(arg1, arg2, &mut token)
    } else {
        0
    };

    let mut level;
    match first as u8 {
        0 => return 0,
        b'?' | b'}' | b']' => return b'?' as c_char,
        b'{' | b'[' => level = 1,
        _ => return b'v' as c_char,
    }

    while level > 0 {
        let t = if let Some(next) = next {
            next(arg1, arg2, &mut token)
        } else {
            0
        };
        match t as u8 {
            0 => return 0,
            b'?' => return b'?' as c_char,
            b'{' | b'[' => level += 1,
            b'}' | b']' => level -= 1,
            _ => {}
        }
    }

    b'v' as c_char
}

pub use crate::htslib_rs::textutils::{hts_json_sskip_value, hts_strprint, snext, stringify_argv};

pub unsafe fn find_file_extension(fn_: *const c_char, ext_out: *mut c_char) -> i32 {
    if fn_.is_null() {
        return -1;
    }
    let idx_delim = c"##idx##";
    let mut delim = libc::strstr(fn_, idx_delim.as_ptr()).cast_const();
    if delim.is_null() {
        delim = fn_.add(libc::strlen(fn_));
    }

    let mut ext = delim;
    while ext > fn_ && *ext != b'.' as c_char && *ext != b'/' as c_char {
        ext = ext.sub(1);
    }
    if *ext == b'.' as c_char
        && ext > fn_
        && ((delim.offset_from(ext) == 3
            && *ext.add(1) == b'g' as c_char
            && *ext.add(2) == b'z' as c_char)
            || (delim.offset_from(ext) == 4
                && *ext.add(1) == b'b' as c_char
                && *ext.add(2) == b'g' as c_char
                && *ext.add(3) == b'z' as c_char))
    {
        ext = ext.sub(1);
        while ext > fn_ && *ext != b'.' as c_char && *ext != b'/' as c_char {
            ext = ext.sub(1);
        }
    }

    let ext_len = delim.offset_from(ext);
    if *ext != b'.' as c_char || ext_len > HTS_MAX_EXT_LEN as isize || ext_len < 3 {
        return -1;
    }
    std::ptr::copy_nonoverlapping(ext.add(1), ext_out, (ext_len - 1) as usize);
    *ext_out.add((ext_len - 1) as usize) = 0;
    0
}

pub unsafe fn hts_usleep(usec: i64) -> i32 {
    let req = libc::timespec {
        tv_sec: (usec / 1_000_000) as libc::time_t,
        tv_nsec: ((usec % 1_000_000) * 1000) as i64,
    };
    libc::nanosleep(&req, std::ptr::null_mut())
}

pub unsafe fn svlen_on_ref_for_vcf_alt(alt: *const c_char, size: i32) -> i32 {
    if *alt != b'<' as c_char {
        return 0;
    }
    let sz = if size >= 0 {
        size as usize
    } else {
        libc::strlen(alt)
    };
    if sz < 5 {
        return 0;
    }
    if *alt.add(4) != b'>' as c_char && *alt.add(4) != b':' as c_char {
        return 0;
    }
    if libc::memcmp(alt.cast(), c"<CNV".as_ptr().cast(), 4) != 0
        && libc::memcmp(alt.cast(), c"<DEL".as_ptr().cast(), 4) != 0
        && libc::memcmp(alt.cast(), c"<DUP".as_ptr().cast(), 4) != 0
        && libc::memcmp(alt.cast(), c"<INV".as_ptr().cast(), 4) != 0
    {
        return 0;
    }
    (*alt.add(sz - 1) == b'>' as c_char) as i32
}

pub fn isalnum_c(c: u8) -> i32 {
    unsafe { libc::isalnum(c as i32) }
}

pub fn isalpha_c(c: u8) -> i32 {
    unsafe { libc::isalpha(c as i32) }
}

pub fn isdigit_c(c: u8) -> i32 {
    unsafe { libc::isdigit(c as i32) }
}

pub fn isgraph_c(c: u8) -> i32 {
    unsafe { libc::isgraph(c as i32) }
}

pub fn islower_c(c: u8) -> i32 {
    unsafe { libc::islower(c as i32) }
}

pub fn isprint_c(c: u8) -> i32 {
    unsafe { libc::isprint(c as i32) }
}

pub fn ispunct_c(c: u8) -> i32 {
    unsafe { libc::ispunct(c as i32) }
}

pub fn isspace_c(c: u8) -> i32 {
    unsafe { libc::isspace(c as i32) }
}

pub fn isupper_c(c: u8) -> i32 {
    unsafe { libc::isupper(c as i32) }
}

pub fn isxdigit_c(c: u8) -> i32 {
    unsafe { libc::isxdigit(c as i32) }
}

pub fn tolower_c(c: u8) -> u8 {
    unsafe { libc::tolower(c as i32) as u8 }
}

pub fn toupper_c(c: u8) -> u8 {
    unsafe { libc::toupper(c as i32) as u8 }
}

pub fn hts_bin_first(l: i32) -> i32 {
    ((1 << ((l << 1) + l)) - 1) / 7
}

pub fn hts_bin_parent(b: i32) -> i32 {
    (b - 1) >> 3
}

pub fn hts_reg2bin(beg: hts_pos_t, mut end: hts_pos_t, min_shift: i32, n_lvls: i32) -> i32 {
    let mut s = min_shift;
    let mut t = ((1 << ((n_lvls << 1) + n_lvls)) - 1) / 7;
    end -= 1;
    let mut l = n_lvls;
    while l > 0 {
        if beg >> s == end >> s {
            return t + (beg >> s) as i32;
        }
        l -= 1;
        s += 3;
        t -= 1 << ((l << 1) + l);
    }
    0
}

pub fn hts_bin_level(bin: i32) -> i32 {
    let mut l = 0;
    let mut b = bin;
    while b != 0 {
        l += 1;
        b = hts_bin_parent(b);
    }
    l
}

pub fn format_category(fmt: htsExactFormat) -> htsFormatCategory {
    match fmt {
        HTS_FORMAT_BAM
        | HTS_FORMAT_SAM
        | HTS_FORMAT_CRAM
        | HTS_FORMAT_FASTQ_FORMAT
        | HTS_FORMAT_FASTA_FORMAT => HTS_FORMAT_SEQUENCE_DATA,
        HTS_FORMAT_VCF | HTS_FORMAT_BCF => HTS_FORMAT_VARIANT_DATA,
        HTS_FORMAT_BAI
        | HTS_FORMAT_CRAI_EXACT
        | HTS_FORMAT_CSI
        | HTS_FORMAT_FAI_FORMAT
        | HTS_FORMAT_FQI_FORMAT
        | HTS_FORMAT_GZI
        | HTS_FORMAT_TBI => HTS_FORMAT_INDEX_FILE,
        HTS_FORMAT_BED | HTS_FORMAT_D4_FORMAT => HTS_FORMAT_REGION_LIST,
        HTS_FORMAT_HTSGET | HTS_FORMAT_CRYPT4GH_FORMAT => HTS_FORMAT_UNKNOWN_CATEGORY,
        _ => HTS_FORMAT_UNKNOWN_CATEGORY,
    }
}

pub unsafe fn parse_version(fmt: *mut htsFormat, u: *const u8, ulim: *const u8) {
    let mut s = u;
    let slim = ulim;
    (*fmt).version.major = -1;
    (*fmt).version.minor = -1;

    let mut v: i16 = 0;
    while s < slim && (*s).is_ascii_digit() {
        v = 10 * v + (*s - b'0') as i16;
        s = s.add(1);
    }

    if s < slim {
        (*fmt).version.major = v;
        if *s == b'.' {
            s = s.add(1);
            v = 0;
            while s < slim && (*s).is_ascii_digit() {
                v = 10 * v + (*s - b'0') as i16;
                s = s.add(1);
            }
            if s < slim {
                (*fmt).version.minor = v;
            }
        } else {
            (*fmt).version.minor = 0;
        }
    }
}

pub unsafe fn cmp_nonblank(key: *const c_char, mut u: *const u8, ulim: *const u8) -> i32 {
    let mut ukey = key.cast::<u8>();
    while *ukey != 0 {
        if u >= ulim {
            return 1;
        } else if (*u).is_ascii_whitespace() {
            u = u.add(1);
        } else if *u != *ukey {
            return if *ukey < *u { -1 } else { 1 };
        } else {
            u = u.add(1);
            ukey = ukey.add(1);
        }
    }
    0
}

pub unsafe fn is_text_only(mut u: *const u8, ulim: *const u8) -> i32 {
    while u < ulim {
        if !(*u >= b' ' || *u == b'\t' || *u == b'\r' || *u == b'\n') {
            return 0;
        }
        u = u.add(1);
    }
    1
}

pub unsafe fn alternate_zeros(mut u: *const u8, ulim: *const u8) -> i32 {
    while u < ulim {
        if *u != 0 {
            return 0;
        }
        u = u.add(2);
    }
    1
}

pub unsafe fn is_utf16_text(u: *const u8, ulim: *const u8) -> i32 {
    if ulim.offset_from(u) >= 6
        && ((*u.add(0) == 0xfe && *u.add(1) == 0xff && alternate_zeros(u.add(2), ulim) != 0)
            || (*u.add(0) == 0xff && *u.add(1) == 0xfe && alternate_zeros(u.add(3), ulim) != 0))
    {
        2
    } else if ulim.offset_from(u) >= 8
        && (alternate_zeros(u, ulim) != 0 || alternate_zeros(u.add(1), ulim) != 0)
    {
        1
    } else {
        0
    }
}

pub unsafe fn hts_c_313_decompress_peek_gz(
    fp: *mut hFILE,
    dest: *mut u8,
    destsize: usize,
) -> isize {
    let mut buffer = [0u8; 2048];
    let npeek = hpeek(fp, buffer.as_mut_ptr().cast(), buffer.len());
    if npeek < 0 {
        return -1;
    }

    let mut output_pos = 0usize;
    let input_len = npeek as usize;
    let output = std::slice::from_raw_parts_mut(dest, destsize);
    let mut decoder = flate2::read::MultiGzDecoder::new(std::io::Cursor::new(&buffer[..input_len]));

    while output_pos < destsize {
        match std::io::Read::read(&mut decoder, &mut output[output_pos..]) {
            Ok(0) | Err(_) => break,
            Ok(n) => output_pos += n,
        }
    }

    output_pos as isize
}

pub unsafe fn hts_c_356_decompress_peek_xz(
    fp: *mut hFILE,
    dest: *mut u8,
    destsize: usize,
) -> isize {
    let mut buffer = [0u8; 2048];
    let npeek = hpeek(fp, buffer.as_mut_ptr().cast(), buffer.len());
    if npeek < 0 {
        return -1;
    }

    let input_len = npeek as usize;
    let output = std::slice::from_raw_parts_mut(dest, destsize);
    let mut output_pos = 0usize;
    let mut decoder = xz2::read::XzDecoder::new(std::io::Cursor::new(&buffer[..input_len]));

    while output_pos < destsize {
        match std::io::Read::read(&mut decoder, &mut output[output_pos..]) {
            Ok(0) => break,
            Ok(n) => output_pos += n,
            Err(_) => return -1,
        }
    }

    output_pos as isize
}

pub unsafe fn hts_c_458_is_fastaq(u: *const u8, ulim: *const u8) -> i32 {
    let len = ulim.offset_from(u) as usize;
    let mut eol = std::ptr::null();
    for i in 0..len {
        if *u.add(i) == b'\n' {
            eol = u.add(i);
            break;
        }
    }

    if is_text_only(u, if eol.is_null() { ulim } else { eol }) == 0 {
        return 0;
    }
    if eol.is_null() {
        return 1;
    }

    let mut p = eol.add(1);
    while p < ulim
        && (crate::htslib_rs::sam::SEQ_NT16_TABLE[*p as usize] != 15
            || (*p as u8).eq_ignore_ascii_case(&b'N'))
    {
        if *p == b'=' {
            return 0;
        }
        p = p.add(1);
    }

    (p == ulim || *p == b'\r' || *p == b'\n') as i32
}

pub unsafe fn hts_c_483_parse_tabbed_text(
    columns: *mut c_char,
    column_len: i32,
    u: *const u8,
    ulim: *const u8,
    complete: *mut i32,
) -> i32 {
    const DIGIT: u32 = 1;
    const LEADING_SIGN: u32 = 2;
    const CIGAR_OPERATOR: u32 = 4;
    const OTHER: u32 = 8;
    const BAM_CIGAR_STR: &[u8] = b"MIDNSHP=XB\0";

    let mut str_ = u.cast::<c_char>();
    let slim = ulim.cast::<c_char>();
    let mut s = str_;
    let mut ncolumns = 0;
    let mut seen = 0u32;
    *complete = 0;

    while s < slim {
        if *s >= b' ' as c_char {
            if (*s as u8).is_ascii_digit() {
                seen |= DIGIT;
            } else if (*s == b'+' as c_char || *s == b'-' as c_char) && s == str_ {
                seen |= LEADING_SIGN;
            } else if !libc::strchr(BAM_CIGAR_STR.as_ptr().cast(), *s as i32).is_null()
                && s > str_
                && (*s.offset(-1) as u8).is_ascii_digit()
            {
                seen |= CIGAR_OPERATOR;
            } else {
                seen |= OTHER;
            }
        } else if *s == b'\t' as c_char || *s == b'\r' as c_char || *s == b'\n' as c_char {
            let len = s.offset_from(str_) as usize;
            let type_ = if seen == DIGIT || seen == (LEADING_SIGN | DIGIT) {
                b'i'
            } else if seen == (DIGIT | CIGAR_OPERATOR) {
                b'C'
            } else if len == 1 {
                match *str_ as u8 {
                    b'*' => b'C',
                    b'+' | b'-' | b'.' => b's',
                    _ => b'Z',
                }
            } else if len >= 5 && *str_.add(2) == b':' as c_char && *str_.add(4) == b':' as c_char {
                b'O'
            } else {
                b'Z'
            };

            *columns.add(ncolumns as usize) = type_ as c_char;
            ncolumns += 1;
            if *s != b'\t' as c_char || ncolumns >= column_len - 1 {
                *complete = 1;
                break;
            }
            str_ = s.add(1);
            seen = 0;
        } else {
            return -1;
        }
        s = s.add(1);
    }

    *columns.add(ncolumns as usize) = 0;
    ncolumns
}

pub unsafe fn hts_c_540_colmatch(columns: *const c_char, pattern: *const c_char) -> i32 {
    let mut i = 0usize;
    while *columns.add(i) != 0 {
        if *pattern.add(i) == b'+' as c_char {
            return i as i32;
        }
        if !(*columns.add(i) == *pattern.add(i) || *pattern.add(i) == b'Z' as c_char) {
            return 0;
        }
        i += 1;
    }
    i as i32
}

pub unsafe fn hts_is_utf16_text(str_: *const kstring_t) -> i32 {
    let u = (*str_).data.as_ptr();
    if !(*str_).data.is_empty() {
        is_utf16_text(u, u.add((*str_).data.len()))
    } else {
        0
    }
}

pub fn push_digit(i: u64, c: u8) -> u64 {
    let digit = c - b'0';
    10 * i + digit as u64
}

pub unsafe fn hts_parse_decimal(
    str_: *const u8,
    strend: *mut *mut u8,
    flags: i32,
) -> i64 {
    let mut n = 0u64;
    let mut digits = 0;
    let mut decimals = 0;
    let mut e = 0i32;
    let mut lost = 0u64;
    let mut sign = b'+';
    let mut esign = b'+';
    let str_orig = str_;
    let mut strp = str_;

    while (*strp).is_ascii_whitespace() {
        strp = strp.add(1);
    }
    let mut s = strp;

    if *s == b'+' || *s == b'-' {
        sign = *s;
        s = s.add(1);
    }
    while *s != 0 {
        if (*s).is_ascii_digit() {
            digits += 1;
            n = push_digit(n, *s);
            s = s.add(1);
        } else if *s == b',' && (flags & HTS_PARSE_THOUSANDS_SEP) != 0 {
            s = s.add(1);
        } else {
            break;
        }
    }

    if *s == b'.' {
        s = s.add(1);
        while (*s).is_ascii_digit() {
            decimals += 1;
            digits += 1;
            n = push_digit(n, *s);
            s = s.add(1);
        }
    }

    match *s {
        b'e' | b'E' => {
            s = s.add(1);
            if *s == b'+' || *s == b'-' {
                esign = *s;
                s = s.add(1);
            }
            while (*s).is_ascii_digit() {
                e = push_digit(e as u64, *s) as i32;
                s = s.add(1);
            }
            if esign == b'-' {
                e = -e;
            }
        }
        b'k' | b'K' => {
            e += 3;
            s = s.add(1);
        }
        b'm' | b'M' => {
            e += 6;
            s = s.add(1);
        }
        b'g' | b'G' => {
            e += 9;
            s = s.add(1);
        }
        _ => {}
    }

    e -= decimals;
    while e > 0 {
        n = n.wrapping_mul(10);
        e -= 1;
    }
    while e < 0 {
        lost += n % 10;
        n /= 10;
        e += 1;
    }
    let _ = lost;

    if !strend.is_null() {
        *strend = if digits > 0 {
            s.cast_mut()
        } else {
            str_orig.cast_mut()
        };
    }

    if sign == b'+' {
        n as i64
    } else if n == (i64::MAX as u64) + 1 {
        i64::MIN
    } else {
        -(n as i64)
    }
}

pub unsafe fn hts_memrchr(s: *const (), c: i32, n: usize) -> *mut () {
    let u = s.cast::<u8>();
    let mut i = n;
    while i > 0 {
        if *u.add(i - 1) == c as u8 {
            return u.add(i - 1).cast::<()>().cast_mut();
        }
        i -= 1;
    }
    std::ptr::null_mut()
}

pub unsafe fn hts_parse_region(
    s: &CStr,
    tid: &mut i32,
    beg: &mut hts_pos_t,
    end: &mut hts_pos_t,
    getid: hts_name2id_f,
    hdr: *mut (),
    mut flags: i32,
) -> *const c_char {
    if getid.is_none() {
        return std::ptr::null();
    }

    let mut s = s.as_ptr();
    let mut s_len = CStr::from_ptr(s).to_bytes().len();
    let mut ks = kstring_t::default();
    let mut colon: *const c_char = std::ptr::null();
    let mut quoted = 0usize;

    if (flags & HTS_PARSE_LIST) != 0 {
        flags &= !HTS_PARSE_THOUSANDS_SEP;
    } else {
        flags |= HTS_PARSE_THOUSANDS_SEP;
    }

    let mut s_end = s.add(s_len);
    if *s == b'{' as c_char {
        let close = libc::memchr(s.cast(), b'}' as i32, s_len).cast::<c_char>();
        if close.is_null() {
            *tid = -1;
            return std::ptr::null();
        }
        s = s.add(1);
        s_len -= 1;
        if *close.add(1) == b':' as c_char {
            colon = close.add(1);
        }
        quoted = 1;
        if (flags & HTS_PARSE_LIST) != 0 {
            let comma = libc::strchr(close, b',' as i32);
            if !comma.is_null() {
                s_len = comma.offset_from(s) as usize;
                s_end = comma.add(1);
            }
        }
    } else {
        if (flags & HTS_PARSE_LIST) != 0 {
            let comma = libc::strchr(s, b',' as i32);
            if !comma.is_null() {
                s_len = comma.offset_from(s) as usize;
                s_end = comma.add(1);
            }
        }
        colon = hts_memrchr(s.cast(), b':' as i32, s_len).cast::<c_char>();
    }

    let getid = getid.unwrap_unchecked();

    if colon.is_null() {
        *beg = 0;
        *end = HTS_POS_MAX;
        kputsn(
            std::slice::from_raw_parts(s.cast::<u8>(), s_len - quoted),
            s_len - quoted,
            &mut ks,
        );
        ks.data.push(0);
        *tid = getid(hdr.cast(), ks.data.as_ptr().cast());
        ks.data.pop();
        return if *tid >= 0 { s_end } else { std::ptr::null() };
    }

    if quoted == 0 {
        *beg = 0;
        *end = HTS_POS_MAX;
        kputsn(
            std::slice::from_raw_parts(s.cast::<u8>(), s_len),
            s_len,
            &mut ks,
        );
        ks.data.push(0);
        *tid = getid(hdr.cast(), ks.data.as_ptr().cast());
        ks.data.pop();
        if *tid >= 0 {
            ks.data.clear();
            let len = colon.offset_from(s) as usize;
            kputsn(std::slice::from_raw_parts(s.cast::<u8>(), len), len, &mut ks);
            ks.data.push(0);
            let found = getid(hdr.cast(), ks.data.as_ptr().cast());
            ks.data.pop();
            if found >= 0 {
                *tid = -1;
                return std::ptr::null();
            }
            return s_end;
        }
        if *tid < -1 {
            return std::ptr::null();
        }
    }

    ks.data.clear();
    let len = colon.offset_from(s) as usize - quoted;
    kputsn(std::slice::from_raw_parts(s.cast::<u8>(), len), len, &mut ks);
    ks.data.push(0);
    *tid = getid(hdr.cast(), ks.data.as_ptr().cast());
    ks.data.pop();
    if *tid < 0 {
        return std::ptr::null();
    }

    let mut hyphen: *mut u8 = std::ptr::null_mut();
    *beg = hts_parse_decimal(colon.add(1).cast(), &mut hyphen, flags) - 1;
    if *beg < 0 {
        if *beg != -1 && *hyphen == b'-' && *colon.add(1) != 0 {
            return std::ptr::null();
        }
        if (*hyphen).is_ascii_digit() || *hyphen == 0 || *hyphen == b',' {
            *end = if *beg == -1 { HTS_POS_MAX } else { -(*beg + 1) };
            *beg = 0;
            return s_end;
        } else if *beg < -1 {
            return std::ptr::null();
        }
    }

    if *hyphen == 0 || ((flags & HTS_PARSE_LIST) != 0 && *hyphen == b',') {
        *end = if (flags & HTS_PARSE_ONE_COORD) != 0 {
            *beg + 1
        } else {
            HTS_POS_MAX
        };
    } else if *hyphen == b'-' {
        *end = hts_parse_decimal(hyphen.add(1), &mut hyphen, flags);
        if *hyphen != 0 && *hyphen != b',' {
            return std::ptr::null();
        }
    } else {
        return std::ptr::null();
    }

    if *end == 0 {
        *end = HTS_POS_MAX;
    }
    if *beg >= *end {
        return std::ptr::null();
    }
    s_end
}

pub unsafe fn hts_parse_reg64(
    s: *const c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> *const c_char {
    let colon = libc::strrchr(s, b':' as i32);
    if colon.is_null() {
        *beg = 0;
        *end = HTS_POS_MAX;
        return s.add(CStr::from_ptr(s).to_bytes().len());
    }

    let mut hyphen: *mut u8 = std::ptr::null_mut();
    *beg = hts_parse_decimal(colon.add(1).cast(), &mut hyphen, HTS_PARSE_THOUSANDS_SEP) - 1;
    if *beg < 0 {
        *beg = 0;
    }

    if *hyphen == 0 {
        *end = HTS_POS_MAX;
    } else if *hyphen == b'-' {
        *end = hts_parse_decimal(hyphen.add(1), std::ptr::null_mut(), HTS_PARSE_THOUSANDS_SEP);
    } else {
        return std::ptr::null();
    }

    if *beg >= *end {
        return std::ptr::null();
    }
    colon
}

pub unsafe fn hts_parse_reg(s: *const c_char, beg: *mut i32, end: *mut i32) -> *const c_char {
    let mut beg64 = 0;
    let mut end64 = 0;
    let colon = hts_parse_reg64(s, &mut beg64, &mut end64);
    if beg64 > i32::MAX as hts_pos_t {
        return std::ptr::null();
    }
    if end64 > i32::MAX as hts_pos_t {
        if end64 == HTS_POS_MAX {
            end64 = i32::MAX as hts_pos_t;
        } else {
            return std::ptr::null();
        }
    }
    *beg = beg64 as i32;
    *end = end64 as i32;
    colon
}

pub unsafe fn hts_time_normalise(tens: *mut i32, units: *mut i32, base: i32) -> i32 {
    if *units < 0 || *units >= base {
        let delta = if *units >= 0 {
            *units / base
        } else {
            -1 - (-1 - *units) / base
        };
        let tmp = *tens as i64 + delta as i64;
        if tmp < i32::MIN as i64 || tmp > i32::MAX as i64 {
            return 1;
        }
        *tens = tmp as i32;
        *units -= delta * base;
    }
    0
}

pub fn hts_year_is_leap(year: i64) -> i32 {
    ((year % 4 == 0) && (year % 100 != 0) || (year % 400 == 0)) as i32
}

pub fn hts_leaps_to_year_start(mut year: i64) -> i64 {
    year -= 1;
    year / 4 - year / 100 + year / 400
}

pub unsafe fn hts_time_normalise_tm(t: *mut libc::tm) -> i32 {
    let days_per_mon = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];
    let year_days = [365, 366];
    let mut overflow = 0;

    if (*t).tm_sec > 62 {
        overflow |= hts_time_normalise(&mut (*t).tm_min, &mut (*t).tm_sec, 60);
    }
    overflow |= hts_time_normalise(&mut (*t).tm_hour, &mut (*t).tm_min, 60);
    overflow |= hts_time_normalise(&mut (*t).tm_mday, &mut (*t).tm_hour, 24);
    overflow |= hts_time_normalise(&mut (*t).tm_year, &mut (*t).tm_mon, 12);
    if overflow != 0 {
        return 1;
    }

    let mut year = (*t).tm_year as i64 + 1900;
    while (*t).tm_mday <= 0 {
        year -= 1;
        (*t).tm_mday += year_days[hts_year_is_leap(year + (1 < (*t).tm_mon) as i64) as usize];
    }
    while (*t).tm_mday > 366 {
        (*t).tm_mday -= year_days[hts_year_is_leap(year + (1 < (*t).tm_mon) as i64) as usize];
        year += 1;
    }
    loop {
        let mdays = days_per_mon[hts_year_is_leap(year) as usize][(*t).tm_mon as usize];
        if (*t).tm_mday <= mdays {
            break;
        }
        (*t).tm_mday -= mdays;
        (*t).tm_mon += 1;
        if (*t).tm_mon >= 12 {
            year += 1;
            (*t).tm_mon = 0;
        }
    }
    year -= 1900;
    if year != (*t).tm_year as i64 {
        if year < i32::MIN as i64 || year > i32::MAX as i64 {
            return 1;
        }
        (*t).tm_year = year as i32;
    }
    0
}

pub unsafe fn hts_time_gm(target: *mut libc::tm) -> libc::time_t {
    let month_start = [
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
    ];

    if hts_time_normalise_tm(target) != 0 || (*target).tm_year < 70 {
        *libc::__errno_location() = libc::EOVERFLOW;
        return -1 as libc::time_t;
    }

    let years_from_epoch = (*target).tm_year - 70;
    let leaps =
        hts_leaps_to_year_start((*target).tm_year as i64 + 1900) - hts_leaps_to_year_start(1970);
    let days = 365 * (years_from_epoch as i64 - leaps)
        + 366 * leaps
        + month_start[hts_year_is_leap((*target).tm_year as i64 + 1900) as usize]
            [(*target).tm_mon as usize] as i64
        + (*target).tm_mday as i64
        - 1;
    let secs = days * 86400
        + (*target).tm_hour as i64 * 3600
        + (*target).tm_min as i64 * 60
        + (*target).tm_sec as i64;

    if std::mem::size_of::<libc::time_t>() < 8 && secs > i32::MAX as i64 {
        *libc::__errno_location() = libc::EOVERFLOW;
        return -1 as libc::time_t;
    }
    secs as libc::time_t
}

pub unsafe fn hts_resize_array_(
    item_size: usize,
    num: usize,
    size_sz: usize,
    size_in_out: *mut (),
    ptr_in_out: *mut *mut (),
    flags: i32,
    _func: *const u8,
) -> i32 {
    let safe = 1usize << (std::mem::size_of::<usize>() * 4);
    let mut new_size = num;
    if new_size > 0 {
        new_size -= 1;
        new_size |= new_size >> (std::mem::size_of::<usize>() / 8);
        new_size |= new_size >> (std::mem::size_of::<usize>() / 4);
        new_size |= new_size >> (std::mem::size_of::<usize>() / 2);
        new_size |= new_size >> std::mem::size_of::<usize>();
        new_size |= new_size >> (std::mem::size_of::<usize>() * 2);
        new_size |= new_size >> (std::mem::size_of::<usize>() * 4);
        if ((new_size >> (std::mem::size_of::<usize>() * 8 - 1)) & 1) == 0 {
            new_size += 1;
        }
    }
    let bytes = item_size.wrapping_mul(new_size);

    if new_size > ((1usize << (size_sz * 8 - 1)) - 1)
        || (((item_size > safe) || (new_size > safe))
            && (new_size == 0 || bytes / new_size != item_size))
    {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }

    // Type-erased reallocation. The previous allocation size is not tracked by
    // this generic primitive, so we re-grow via a fresh allocation + copy. The
    // caller's old size is recovered from `size_in_out` only in the CLEAR path.
    let old_count = match size_sz {
        4 => *(size_in_out.cast::<u32>()) as usize,
        8 => *(size_in_out.cast::<u64>()) as usize,
        _ => std::process::abort(),
    };
    let old_bytes = old_count.wrapping_mul(item_size);
    let mut buf: Vec<u8> = if (*ptr_in_out).is_null() {
        Vec::new()
    } else {
        // Reconstitute the previously handed-out Vec allocation.
        Vec::from_raw_parts((*ptr_in_out).cast::<u8>(), old_bytes, old_bytes)
    };
    if bytes > buf.capacity() {
        buf.reserve(bytes - buf.len());
    }
    if (flags & HTS_RESIZE_CLEAR) != 0 {
        // Zero from the old element count onward, matching the C CLEAR path.
        buf.resize(old_count * item_size, 0);
        buf.resize(bytes, 0);
    } else {
        buf.set_len(bytes);
    }
    let new_ptr = buf.as_mut_ptr().cast::<()>();
    std::mem::forget(buf);

    // SAFETY: see comment above — `size_in_out` is always a naturally aligned
    // pointer to a `u32`/`u64` (or sign-equivalent) sized field.
    match size_sz {
        4 => *(size_in_out.cast::<u32>()) = new_size as u32,
        8 => *(size_in_out.cast::<u64>()) = new_size as u64,
        _ => std::process::abort(),
    }
    *ptr_in_out = new_ptr;
    0
}

fn kroundup_size_t(x: &mut usize) {
    if *x == 0 {
        return;
    }
    *x -= 1;
    *x |= *x >> 1;
    *x |= *x >> 2;
    *x |= *x >> 4;
    *x |= *x >> 8;
    *x |= *x >> 16;
    if std::mem::size_of::<usize>() == 8 {
        *x |= *x >> 32;
    }
    *x += 1;
}

pub unsafe fn hts_free(ptr: *mut ()) {
    // Type-erased free for buffers handed out by `hts_resize_array_` /
    // `hts_realloc_or_die`, which are Vec<u8>-backed but carry no recorded
    // length here. With no caller in-tree and no size available this cannot
    // safely reconstruct the owning Vec; left as a no-op pending a typed API.
    let _ = ptr;
}

pub unsafe fn hts_realloc_or_die(
    n: usize,
    m: usize,
    m_sz: usize,
    size: usize,
    clear: i32,
    ptr: *mut *mut (),
    func: &[u8],
) -> usize {
    let safe = 1usize << (std::mem::size_of::<usize>() * 4);
    let mut new_m = n;
    kroundup_size_t(&mut new_m);
    let bytes = size.wrapping_mul(new_m);

    if new_m > ((1usize << (m_sz * 8 - 1)) - 1)
        || ((size > safe || new_m > safe) && new_m != 0 && bytes / new_m != size)
    {
        *libc::__errno_location() = libc::ENOMEM;
        let msg = CStr::from_ptr(libc::strerror(*libc::__errno_location()));
        hts_log_cstr(HTS_LOG_ERROR, func, msg.to_bytes());
        std::process::exit(1);
    }

    // Grow the Vec<u8>-backed allocation, preserving existing contents.
    let old_bytes = m.wrapping_mul(size);
    let mut buf: Vec<u8> = if (*ptr).is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*ptr).cast::<u8>(), old_bytes, old_bytes)
    };
    if clear != 0 && new_m > m {
        buf.resize(bytes, 0);
    } else {
        if bytes > buf.capacity() {
            buf.reserve(bytes - buf.len());
        }
        buf.set_len(bytes);
    }
    let new_ptr = buf.as_mut_ptr().cast::<()>();
    std::mem::forget(buf);

    *ptr = new_ptr;
    new_m
}

pub unsafe fn hts_lib_shutdown() {
    crate::htslib_rs::hfile::hfile_c_983_hfile_shutdown(1);
}

pub use crate::htslib_rs::hts_expr::{
    hts_filter_eval, hts_filter_eval2, hts_filter_free, hts_filter_init,
};

pub unsafe fn hts_set_log_level(level: htsLogLevel) {
    hts_verbose = level;
}

pub unsafe fn hts_get_log_level() -> htsLogLevel {
    hts_verbose
}

pub fn get_severity_tag(severity: htsLogLevel) -> u8 {
    match severity {
        HTS_LOG_ERROR => b'E',
        HTS_LOG_WARNING => b'W',
        HTS_LOG_INFO => b'I',
        HTS_LOG_DEBUG => b'D',
        HTS_LOG_TRACE => b'T',
        _ => b'*',
    }
}

pub unsafe fn hts_log_cstr(severity: htsLogLevel, context: &[u8], message: &[u8]) {
    let save_errno = *libc::__errno_location();
    if severity <= hts_verbose {
        eprintln!(
            "[{}::{}] {}",
            get_severity_tag(severity) as char,
            String::from_utf8_lossy(context),
            String::from_utf8_lossy(message),
        );
    }
    *libc::__errno_location() = save_errno;
}

pub fn hts_bin_bot(bin: i32, n_lvls: i32) -> i32 {
    let l = hts_bin_level(bin);
    (bin - hts_bin_first(l)) << ((n_lvls - l) * 3)
}

pub fn hts_bin_maxpos(min_shift: i32, n_lvls: i32) -> hts_pos_t {
    1_i64 << (min_shift + n_lvls * 3)
}

pub fn ed_is_big() -> i32 {
    if cfg!(target_endian = "big") {
        1
    } else {
        0
    }
}

pub fn ed_swap_2(v: u16) -> u16 {
    v.swap_bytes()
}

pub unsafe fn ed_swap_2p(x: *mut ()) -> *mut () {
    // SAFETY: in-tree callers pass naturally aligned `*mut u16` targets
    // (see `hts_c_*_idx_dump_meta` in this file). The BAM/SAM byte-swap
    // helpers operate on htslib-laid-out records whose multi-byte fields
    // sit on aligned offsets within the malloc'd `bam1_t::data` buffer.
    *x.cast::<u16>() = ed_swap_2(*x.cast::<u16>());
    x
}

pub fn ed_swap_4(v: u32) -> u32 {
    v.swap_bytes()
}

pub unsafe fn ed_swap_4p(x: *mut ()) -> *mut () {
    // SAFETY: in-tree callers pass either naturally aligned `*mut u32`/
    // `*mut i32` locals or pointers into BAM record data at aligned
    // offsets (cigar starts at `data + l_qname` where `l_qname` is padded
    // to a multiple of 4 via `l_extranul` per the htslib BAM layout).
    *x.cast::<u32>() = ed_swap_4(*x.cast::<u32>());
    x
}

pub fn ed_swap_8(v: u64) -> u64 {
    v.swap_bytes()
}

pub unsafe fn ed_swap_8p(x: *mut ()) -> *mut () {
    // SAFETY: in-tree callers pass naturally aligned `*mut u64` targets
    // (`hts_pair64_*_t` fields and stack `u64` locals).
    *x.cast::<u64>() = ed_swap_8(*x.cast::<u64>());
    x
}

pub unsafe fn le_to_u8(buf: *const u8) -> u8 {
    *buf
}

pub unsafe fn le_to_u16(buf: *const u8) -> u16 {
    u16::from_le(std::ptr::read_unaligned(buf.cast::<u16>()))
}

pub unsafe fn le_to_u32(buf: *const u8) -> u32 {
    u32::from_le(std::ptr::read_unaligned(buf.cast::<u32>()))
}

pub unsafe fn le_to_u64(buf: *const u8) -> u64 {
    u64::from_le(std::ptr::read_unaligned(buf.cast::<u64>()))
}

pub unsafe fn u16_to_le(val: u16, buf: *mut u8) {
    // C `memcpy(buf, &val_le, sizeof val_le)` — a single unaligned store.
    // The byte-by-byte form below is what we used to write, but LLVM didn't
    // always fold those 2/4/8 stores into one when the function was crossed
    // from another translation unit. bcf_enc_vint emits these per integer
    // value (millions of calls per multi-sample VCF), so the store-coalesce
    // matters.
    std::ptr::write_unaligned(buf.cast::<u16>(), val.to_le());
}

pub unsafe fn u32_to_le(val: u32, buf: *mut u8) {
    std::ptr::write_unaligned(buf.cast::<u32>(), val.to_le());
}

pub unsafe fn u64_to_le(val: u64, buf: *mut u8) {
    std::ptr::write_unaligned(buf.cast::<u64>(), val.to_le());
}

pub unsafe fn le_to_i8(buf: *const u8) -> i8 {
    let v = le_to_u8(buf);
    if v < 0x80 {
        v as i8
    } else {
        -((0xff - v) as i8) - 1
    }
}

pub unsafe fn le_to_i16(buf: *const u8) -> i16 {
    let v = le_to_u16(buf);
    if v < 0x8000 {
        v as i16
    } else {
        -((0xffff - v) as i16) - 1
    }
}

pub unsafe fn le_to_i32(buf: *const u8) -> i32 {
    let v = le_to_u32(buf);
    if v < 0x8000_0000 {
        v as i32
    } else {
        -((0xffff_ffff - v) as i32) - 1
    }
}

pub unsafe fn le_to_i64(buf: *const u8) -> i64 {
    let v = le_to_u64(buf);
    if v < 0x8000_0000_0000_0000 {
        v as i64
    } else {
        -((0xffff_ffff_ffff_ffff - v) as i64) - 1
    }
}

pub unsafe fn i16_to_le(val: i16, buf: *mut u8) {
    u16_to_le(val as u16, buf);
}

pub unsafe fn i32_to_le(val: i32, buf: *mut u8) {
    u32_to_le(val as u32, buf);
}

pub unsafe fn i64_to_le(val: i64, buf: *mut u8) {
    u64_to_le(val as u64, buf);
}

pub unsafe fn le_to_float(buf: *const u8) -> f32 {
    f32::from_bits(le_to_u32(buf))
}

pub unsafe fn le_to_double(buf: *const u8) -> f64 {
    f64::from_bits(le_to_u64(buf))
}

pub unsafe fn float_to_le(val: f32, buf: *mut u8) {
    u32_to_le(val.to_bits(), buf);
}

pub unsafe fn double_to_le(val: f64, buf: *mut u8) {
    u64_to_le(val.to_bits(), buf);
}

// === BGZF: owned root struct (SEAM phase 1) ===
//
// Previously a `#[repr(C)]` mirror of the C struct where
// uncompressed_block/compressed_block/cache/mt/idx were `*mut c_void` and
// `fp` was `*mut hFILE`. Now an owned Rust struct: the BGZF owns its buffers
// (Vec<u8>), its hFILE, its cache, its multi-threading aux, and its index.
//
// `bitfields` keeps the original packed bit-twiddling (open/compress/is_write/
// is_be/compress_level/is_compressed/is_gzip/cache_size bits) — accessor
// methods in bgzf.rs continue to mask/shift it.
pub struct BGZF {
    pub bitfields: u32,
    pub cache_size: i32,
    pub block_length: i32,
    pub block_clength: i32,
    pub block_offset: i32,
    pub block_address: i64,
    pub uncompressed_address: i64,
    /// owned scratch buffer for the decompressed BGZF block
    pub uncompressed_block: Vec<u8>,
    /// owned scratch buffer for the compressed BGZF block
    pub compressed_block: Vec<u8>,
    /// owned LRU block cache (None when caching disabled)
    pub cache: Option<Box<crate::htslib_rs::bgzf::bgzf_cache_t>>,
    /// the BGZF's underlying file handle: None / a bare fd / an owned hFILE
    pub fp: crate::htslib_rs::bgzf::BgzfFp,
    /// owned multi-threaded writer/reader aux state (None for single-threaded)
    pub mt: Option<Box<crate::htslib_rs::bgzf::bgzf_mtaux_t>>,
    /// owned on-the-fly GZI index (None when not indexing)
    pub idx: Option<Box<crate::htslib_rs::bgzf::bgzidx_t>>,
    pub idx_build_otf: i32,
    pub gz_stream: Option<Box<crate::htslib_rs::bgzf::GzipState>>,
    pub seeked: i64,
}

// === Parity mirror for BGZF ===
// Only used under `parity` for differential tests against the C hts_sys lib.
// Keeps the original C ABI layout (raw void*/hFILE*) so a *mut BGZF_c handed
// to/from the C library can be reinterpreted. Conversion happens at the FFI
// boundary in bgzf.rs (see bgzf_to_c / bgzf_from_c there); production paths
// never touch this type.
#[cfg(feature = "parity")]
#[repr(C)]
pub struct BGZF_c {
    pub bitfields: u32,
    pub cache_size: i32,
    pub block_length: i32,
    pub block_clength: i32,
    pub block_offset: i32,
    pub block_address: i64,
    pub uncompressed_address: i64,
    pub uncompressed_block: *mut c_void,
    pub compressed_block: *mut c_void,
    pub cache: *mut c_void,
    pub fp: *mut hFILE_c,
    pub mt: *mut c_void,
    pub idx: *mut c_void,
    pub idx_build_otf: i32,
    pub gz_stream: *mut c_void,
    pub seeked: i64,
}

#[repr(C)]
pub struct cram_fd {
    _private: [u8; 0],
}

// === hFILE: owned root struct (SEAM phase 1) ===
//
// Previously an opaque forward-decl `struct hFILE { _private: [u8;0] }` whose
// real layout lived in hfile.rs as `hfile_layout` with raw `*mut u8`
// buffer/begin/end/limit pointers and a `*const hfile_backend_layout` vtable.
//
// Now an owned struct: the buffer is a `Vec<u8>` and begin/end/limit are
// `usize` indices into it. The backend vtable-of-fn-pointers is replaced by
// the `HFileBackend` enum (see hfile.rs) which carries each finite backend's
// own state inline and dispatches via `match`.
pub struct hFILE {
    /// owned I/O buffer; `begin`/`end`/`limit` are byte indices into it
    pub buffer: Vec<u8>,
    /// index of first buffered-but-unconsumed byte
    pub begin: usize,
    /// index one past the last valid buffered byte
    pub end: usize,
    /// usable capacity (buffer is sized to this)
    pub limit: usize,
    /// enum dispatch replacing the old `*const hfile_backend_layout`
    pub backend: crate::htslib_rs::hfile::HFileBackend,
    pub offset: libc::off_t,
    pub flags: u32,
    pub has_errno: i32,
}

// === Parity mirror for hFILE ===
// C ABI layout of the buffered hFILE, used only under `parity`.
#[cfg(feature = "parity")]
#[repr(C)]
pub struct hFILE_c {
    pub buffer: *mut u8,
    pub begin: *mut u8,
    pub end: *mut u8,
    pub limit: *mut u8,
    pub backend: *const c_void,
    pub offset: libc::off_t,
    pub flags: u32,
    pub has_errno: i32,
}

#[derive(Clone, Default, Debug)]
pub struct kstring_t {
    pub data: Vec<u8>,
}

#[repr(C)]
pub struct kstring_c_t {
    pub l: usize,
    pub m: usize,
    pub s: *mut c_char,
}

#[repr(C)]
pub struct ks_tokaux_t {
    pub tab: [u64; 4],
    pub sep: i32,
    pub finished: i32,
    pub p: *const c_char,
}

#[repr(C)]
pub struct kbitset_t {
    pub n: usize,
    pub n_max: usize,
    pub b: [u64; 1],
}

#[repr(C)]
pub struct kbitset_iter_t {
    pub mask: u64,
    pub elt: usize,
    pub i: i32,
}

const KBS_ELTBITS: usize = std::mem::size_of::<u64>() * 8;

fn kbs_elt(i: i32) -> usize {
    i as usize / KBS_ELTBITS
}

fn kbs_mask(i: i32) -> u64 {
    (1 as u64) << (i as usize % KBS_ELTBITS)
}

fn kbs_words(bs: &kbitset_t) -> *mut u64 {
    bs.b.as_ptr().cast_mut()
}

pub fn kbs_last_mask(ni: usize) -> u64 {
    let mask = ((1 as u64) << (ni % KBS_ELTBITS)) - 1;
    if mask != 0 {
        mask
    } else {
        !0 as u64
    }
}

pub unsafe fn kbs_init2(ni: usize, fill: i32) -> *mut kbitset_t {
    let n = ni.div_ceil(KBS_ELTBITS);
    let size = std::mem::size_of::<kbitset_t>() + n * std::mem::size_of::<u64>();
    // Single flexible-array-member allocation backed by an owned Vec<u8>.
    let mut backing: Vec<u8> = Vec::with_capacity(size);
    backing.set_len(size);
    let bs = backing.as_mut_ptr().cast::<kbitset_t>();
    std::mem::forget(backing);
    (*bs).n = n;
    (*bs).n_max = n;
    let words = kbs_words(&*bs);
    let fill_byte = if fill != 0 { 0xff } else { 0 };
    std::ptr::write_bytes(
        words.cast::<u8>(),
        fill_byte,
        n * std::mem::size_of::<u64>(),
    );
    *words.add(n) = kbs_last_mask(ni);
    if fill != 0 && n > 0 {
        *words.add(n - 1) &= *words.add(n);
    }
    bs
}

pub unsafe fn kbs_init(ni: usize) -> *mut kbitset_t {
    kbs_init2(ni, 0)
}

pub unsafe fn kbs_resize2(bsp: *mut *mut kbitset_t, ni_new: usize, fill: i32) -> i32 {
    let mut bs = *bsp;
    let n = if bs.is_null() { 0 } else { (*bs).n };
    let n_new = ni_new.div_ceil(KBS_ELTBITS);
    if bs.is_null() || n_new > (*bs).n_max {
        let size = std::mem::size_of::<kbitset_t>() + n_new * std::mem::size_of::<u64>();
        let mut backing: Vec<u8> = if bs.is_null() {
            Vec::new()
        } else {
            let old_size =
                std::mem::size_of::<kbitset_t>() + (*bs).n_max * std::mem::size_of::<u64>();
            Vec::from_raw_parts(bs.cast::<u8>(), old_size, old_size)
        };
        backing.resize(size, 0);
        bs = backing.as_mut_ptr().cast::<kbitset_t>();
        std::mem::forget(backing);
        (*bs).n_max = n_new;
        *bsp = bs;
    }

    (*bs).n = n_new;
    let words = kbs_words(&*bs);
    if n_new >= n {
        let fill_byte = if fill != 0 { 0xff } else { 0 };
        std::ptr::write_bytes(
            words.add(n).cast::<u8>(),
            fill_byte,
            (n_new - n) * std::mem::size_of::<u64>(),
        );
    }
    *words.add(n_new) = kbs_last_mask(ni_new);
    if n_new > 0 {
        *words.add(n_new - 1) &= *words.add(n_new);
    }
    0
}

pub unsafe fn kbs_resize(bsp: *mut *mut kbitset_t, ni_new: usize) -> i32 {
    kbs_resize2(bsp, ni_new, 0)
}

pub unsafe fn kbs_destroy(bs: *mut kbitset_t) {
    if !bs.is_null() {
        let size =
            std::mem::size_of::<kbitset_t>() + (*bs).n_max * std::mem::size_of::<u64>();
        drop(Vec::from_raw_parts(bs.cast::<u8>(), size, size));
    }
}

pub unsafe fn kbs_clear(bs: &mut kbitset_t) {
    std::ptr::write_bytes(
        kbs_words(bs).cast::<u8>(),
        0,
        bs.n * std::mem::size_of::<u64>(),
    );
}

pub unsafe fn kbs_insert_all(bs: &mut kbitset_t) {
    std::ptr::write_bytes(
        kbs_words(bs).cast::<u8>(),
        0xff,
        bs.n * std::mem::size_of::<u64>(),
    );
    if bs.n > 0 {
        let words = kbs_words(bs);
        *words.add(bs.n - 1) &= *words.add(bs.n);
    }
}

pub unsafe fn kbs_insert(bs: &mut kbitset_t, i: i32) {
    *kbs_words(bs).add(kbs_elt(i)) |= kbs_mask(i);
}

pub unsafe fn kbs_delete(bs: &mut kbitset_t, i: i32) {
    *kbs_words(bs).add(kbs_elt(i)) &= !kbs_mask(i);
}

pub unsafe fn kbs_exists(bs: &kbitset_t, i: i32) -> i32 {
    ((*kbs_words(bs).add(kbs_elt(i)) & kbs_mask(i)) != 0) as i32
}

pub unsafe fn kbs_start(itr: &mut kbitset_iter_t) {
    itr.mask = 1;
    itr.elt = 0;
    itr.i = 0;
}

pub unsafe fn kbs_next(bs: &kbitset_t, itr: &mut kbitset_iter_t) -> i32 {
    let words = kbs_words(bs);
    let mut b = *words.add(itr.elt);
    loop {
        if itr.mask == 0 {
            loop {
                itr.elt += 1;
                b = *words.add(itr.elt);
                if b != 0 {
                    break;
                }
                itr.i += KBS_ELTBITS as i32;
            }
            if itr.elt == bs.n {
                return -1;
            }
            itr.mask = 1;
        }

        if (b & itr.mask) != 0 {
            break;
        }
        itr.i += 1;
        itr.mask <<= 1;
    }

    itr.mask <<= 1;
    let ret = itr.i;
    itr.i += 1;
    ret
}

pub fn ks_initialize(s: &mut kstring_t) {
    s.data.clear();
}

pub fn ks_resize(s: &mut kstring_t, size: usize) -> i32 {
    s.data.reserve(size.saturating_sub(s.data.len()));
    0
}

pub fn ks_expand(s: &mut kstring_t, expansion: usize) -> i32 {
    s.data.reserve(expansion);
    0
}

pub fn ks_str(s: &mut kstring_t) -> &mut Vec<u8> {
    &mut s.data
}

pub fn ks_c_str(s: &kstring_t) -> &[u8] {
    &s.data
}

pub fn ks_len(s: &kstring_t) -> usize {
    s.data.len()
}

pub fn ks_clear(s: &mut kstring_t) {
    s.data.clear();
}

pub fn ks_release(s: &mut kstring_t) -> Vec<u8> {
    std::mem::take(&mut s.data)
}

pub fn ks_free(s: &mut kstring_t) {
    s.data = Vec::new();
}

pub unsafe fn hts_prefetch(p: *mut ()) {
    let _ = std::ptr::read_volatile(p.cast::<u8>());
}

pub unsafe fn hts_prefetch_builtin(p: *mut ()) {
    hts_prefetch(p);
}

pub fn kputsn(p: &[u8], l: usize, s: &mut kstring_t) -> i32 {
    // C kputsn always ensures a backing buffer (for the trailing NUL), so even a
    // zero-length write leaves `s->s != NULL`. Preserve that invariant here by
    // forcing a non-zero capacity; downstream code (e.g. hts_expr's
    // `s.s == NULL` exists-test) relies on capacity to distinguish an
    // explicitly-set empty string from a never-allocated one.
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(&p[..l]);
    l as i32
}

pub fn kputs(p: &[u8], s: &mut kstring_t) -> i32 {
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(p);
    p.len() as i32
}

pub fn kputc(c: i32, s: &mut kstring_t) -> i32 {
    s.data.push(c as u8);
    (c as u8) as i32
}

pub fn kputc_(c: i32, s: &mut kstring_t) -> i32 {
    s.data.push(c as u8);
    1
}

pub fn kputsn_(p: &[u8], l: usize, s: &mut kstring_t) -> i32 {
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(&p[..l]);
    l as i32
}

pub fn kputuw(x: u32, s: &mut kstring_t) -> i32 {
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(x.to_string().as_bytes());
    0
}

pub fn kputw(c: i32, s: &mut kstring_t) -> i32 {
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(c.to_string().as_bytes());
    0
}

pub fn kputll(c: i64, s: &mut kstring_t) -> i32 {
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(c.to_string().as_bytes());
    0
}

pub fn kputl(c: isize, s: &mut kstring_t) -> i32 {
    if s.data.capacity() == 0 {
        s.data.reserve(1);
    }
    s.data.extend_from_slice(c.to_string().as_bytes());
    0
}

pub use crate::htslib_rs::kstring::{ksplit, ksplit_core};

pub unsafe fn __ac_X31_hash_string(mut s: *const c_char) -> u32 {
    let mut h = *s as u32;
    if h != 0 {
        s = s.add(1);
        while *s != 0 {
            h = (h << 5).wrapping_sub(h).wrapping_add(*s as u32);
            s = s.add(1);
        }
    }
    h
}

pub unsafe fn __ac_FNV1a_hash_string(mut s: *const c_char) -> u32 {
    let offset_basis: u32 = 2166136261;
    let fnv_prime: u32 = 16777619;
    let mut h = offset_basis;
    while *s != 0 {
        h = (h ^ (*s as u8 as u32)).wrapping_mul(fnv_prime);
        s = s.add(1);
    }
    h
}

pub fn __ac_X31_hash_kstring(ks: &kstring_t) -> u32 {
    let mut h = 0u32;
    let mut i = 0usize;
    while i < ks.data.len() {
        h = (h << 5).wrapping_sub(h).wrapping_add(ks.data[i] as u32);
        i += 1;
    }
    h
}

pub fn __ac_FNV1a_hash_kstring(ks: &kstring_t) -> u32 {
    let offset_basis: u32 = 2166136261;
    let fnv_prime: u32 = 16777619;
    let mut h = offset_basis;
    let mut i = 0usize;
    while i < ks.data.len() {
        h = (h ^ (ks.data[i] as u32)).wrapping_mul(fnv_prime);
        i += 1;
    }
    h
}

pub fn __ac_Wang_hash(mut key: u32) -> u32 {
    key = key.wrapping_add(!(key << 15));
    key ^= key >> 10;
    key = key.wrapping_add(key << 3);
    key ^= key >> 6;
    key = key.wrapping_add(!(key << 11));
    key ^= key >> 16;
    key
}

pub use crate::htslib_rs::kstring::{fgets_wrapper, kfgetline, kgetline, kgetline2, kstrtok};

pub use crate::htslib_rs::kstring::{
    boyer_moore, fast_exp, karp_rabin, kmemmem, ksBM_prep, kstrnstr, kstrstr,
};

pub fn kinsert_char(c: c_char, pos: usize, s: &mut kstring_t) -> i32 {
    if pos > s.data.len() {
        return -1;
    }
    s.data.insert(pos, c as u8);
    0
}

pub fn kinsert_str(str_: &[u8], pos: usize, s: &mut kstring_t) -> i32 {
    if pos > s.data.len() {
        return -1;
    }
    if str_.is_empty() {
        return 0;
    }
    s.data.splice(pos..pos, str_.iter().copied());
    0
}

pub use crate::htslib_rs::hts_expr::{
    expr_func_avg, expr_func_length, expr_func_max, expr_func_min, expr_val_init,
    hts_expr_val_exists, hts_expr_val_existsT, hts_expr_val_free, hts_expr_val_t,
    hts_expr_val_undef, hts_filter_t,
};

pub unsafe fn ws(mut str_: *mut u8) -> *mut u8 {
    while *str_ != 0 && (*str_ == b' ' || *str_ == b'\t') {
        str_ = str_.add(1);
    }
    str_
}

pub use crate::htslib_rs::hts_expr::{
    and_expr, bitand_expr, bitor_expr, bitxor_expr, cmp_expr, eq_expr, expression, func_expr,
    hts_expr_c_849_hts_filter_init, hts_expr_c_863_hts_filter_free, hts_expr_c_903_hts_filter_eval,
    hts_expr_c_920_hts_filter_eval2, hts_filter_eval_, mul_expr, simple_expr, unary_expr,
};

pub use crate::htslib_rs::kstring::kputd;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct htsFormatVersion {
    pub major: i16,
    pub minor: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct htsFormat {
    pub category: htsFormatCategory,
    pub format: htsExactFormat,
    pub version: htsFormatVersion,
    pub compression: htsCompression,
    pub compression_level: i16,
    pub specific: *mut c_void,
}

impl Default for htsFormat {
    fn default() -> Self {
        htsFormat {
            category: 0,
            format: 0,
            version: htsFormatVersion { major: 0, minor: 0 },
            compression: 0,
            compression_level: 0,
            specific: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union htsFilePtr {
    pub bgzf: *mut BGZF,
    pub cram: *mut cram_fd,
    pub hfile: *mut hFILE,
}

impl Default for htsFilePtr {
    fn default() -> Self {
        htsFilePtr {
            bgzf: std::ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_pair64_t {
    pub u: u64,
    pub v: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_pair64_max_t {
    pub u: u64,
    pub v: u64,
    pub max: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_pair_pos_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
}

#[repr(C)]
pub struct hts_idx_bins_t {
    pub m: i32,
    pub n: i32,
    pub loff: u64,
    pub list: *mut hts_pair64_t,
}

#[repr(C)]
pub struct hts_idx_bidx_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut u32,
    pub vals: *mut hts_idx_bins_t,
}

#[repr(C)]
pub struct hts_idx_lidx_t {
    pub n: hts_pos_t,
    pub m: hts_pos_t,
    pub offset: *mut u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_idx_z_t {
    pub last_bin: u32,
    pub save_bin: u32,
    pub last_coor: hts_pos_t,
    pub last_tid: i32,
    pub save_tid: i32,
    pub finished: i32,
    pub padding_0: i32,
    pub last_off: u64,
    pub save_off: u64,
    pub off_beg: u64,
    pub off_end: u64,
    pub n_mapped: u64,
    pub n_unmapped: u64,
}

#[repr(C)]
pub struct hts_idx_t {
    pub fmt: i32,
    pub min_shift: i32,
    pub n_lvls: i32,
    pub n_bins: i32,
    pub l_meta: u32,
    pub n: i32,
    pub m: i32,
    pub padding_0: i32,
    pub n_no_coor: u64,
    pub bidx: *mut *mut hts_idx_bidx_t,
    pub lidx: *mut hts_idx_lidx_t,
    pub meta: *mut u8,
    pub tbi_n: i32,
    pub last_tbi_tid: i32,
    pub z: hts_idx_z_t,
    pub otf_fp: *mut BGZF,
}

#[repr(C)]
pub struct hts_cram_idx_t {
    pub fmt: i32,
    pub cram: *mut cram_fd,
}

#[repr(C)]
pub struct hts_itr_t {
    pub bitfields: u32,
    pub tid: i32,
    pub n_off: i32,
    pub i: i32,
    pub n_reg: i32,
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    pub reg_list: *mut hts_reglist_t,
    pub curr_tid: i32,
    pub curr_reg: i32,
    pub curr_intv: i32,
    pub curr_beg: hts_pos_t,
    pub curr_end: hts_pos_t,
    pub curr_off: u64,
    pub nocoor_off: u64,
    pub off: *mut hts_pair64_max_t,
    pub readrec: hts_readrec_func,
    pub seek: hts_seek_func,
    pub tell: hts_tell_func,
    pub bins: hts_itr_bins_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_itr_bins_t {
    pub n: i32,
    pub m: i32,
    pub a: *mut i32,
}

#[repr(C)]
pub struct hts_reglist_t {
    pub reg: *const c_char,
    pub intervals: *mut hts_pair_pos_t,
    pub tid: i32,
    pub count: u32,
    pub min_beg: hts_pos_t,
    pub max_end: hts_pos_t,
}

#[repr(C)]
pub struct htsFile {
    pub bitfields: u32,
    pub padding_0: u32,
    pub lineno: i64,
    pub line: kstring_t,
    pub fn_: *mut c_char,
    pub fn_aux: *mut c_char,
    pub fp: htsFilePtr,
    pub state: *mut c_void,
    pub format: htsFormat,
    pub idx: *mut hts_idx_t,
    pub fnidx: *const c_char,
    pub bam_header: *mut c_void,
    pub filter: *mut c_void,
}

impl Default for htsFile {
    fn default() -> Self {
        htsFile {
            bitfields: 0,
            padding_0: 0,
            lineno: 0,
            line: kstring_t::default(),
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: htsFilePtr::default(),
            state: std::ptr::null_mut(),
            format: htsFormat::default(),
            idx: std::ptr::null_mut(),
            fnidx: std::ptr::null(),
            bam_header: std::ptr::null_mut(),
            filter: std::ptr::null_mut(),
        }
    }
}

use crate::htslib_rs::hfile::hclose;

extern "C" {
    fn clock() -> libc::clock_t;
}

// (the libhts `kvsprintf` extern was removed 2026-05-29 — the public
// `kvsprintf` wrapper above now routes to the native `kstring_c_142_kvsprintf`.)

unsafe fn bgzf_is_compressed(fp: *const BGZF) -> bool {
    ((*fp).bitfields & (1 << 30)) != 0
}

unsafe fn hts_idx_close_otf_fp(idx: *mut hts_idx_t) -> i32 {
    if idx.is_null() || (*idx).otf_fp.is_null() {
        return 0;
    }

    let mut ret = 0;
    if !bgzf_is_compressed((*idx).otf_fp) {
        let n_no_coor = (*idx).n_no_coor.to_le_bytes();
        if bgzf_write((*idx).otf_fp.cast(), n_no_coor.as_ptr().cast(), 8) < 0 {
            ret = 1;
        }
    }
    if bgzf_close((*idx).otf_fp.cast()) < 0 {
        ret = 1;
    }
    (*idx).otf_fp = std::ptr::null_mut();

    if ret == 0 {
        0
    } else {
        -1
    }
}

// original: hts_open_tmpfile (htslib/hts.c:1980)
//
// Creates a temp file alongside `fname` (suffix `.tmp_<pid>_<n>_<rnd>`) using
// `hopen(.., mode)` and writes the chosen name into `tmpname`. Tries up to
// 100 times with O_EXCL semantics (mode "wx") to avoid clobbering an
// existing file. Returns NULL if all attempts fail.
pub unsafe fn hts_open_tmpfile(
    fname: *const c_char,
    mode: *const c_char,
    tmpname: *mut kstring_t,
) -> *mut hFILE {
    let pid = libc::getpid() as i32;
    let ptr_seed = tmpname as usize as u32;
    let mut n: i32 = 0;
    let mut fp: Option<NonNull<hFILE>> = None;

    loop {
        let now = libc::time(std::ptr::null_mut()) as u32;
        let mut ts: libc::timespec = std::mem::zeroed();
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        let nanos = ts.tv_nsec as u32;
        let t: u32 = now ^ nanos ^ ptr_seed;
        n += 1;

        ks_clear(&mut *tmpname);
        // ksprintf(tmpname, "%s.tmp_%d_%d_%u", fname, pid, n, t)
        // Our portable ksprintf supports %s/%d; %u is the same shape, format
        // as int via the same Int variant since values fit.
        let r = crate::htslib_rs::kstring::ksprintf(
            &mut *tmpname,
            b"%s.tmp_%d_%d_%d",
            &[
                crate::htslib_rs::kstring::KsPrintfArg::Str(CStr::from_ptr(fname).to_bytes()),
                crate::htslib_rs::kstring::KsPrintfArg::Int(pid),
                crate::htslib_rs::kstring::KsPrintfArg::Int(n),
                crate::htslib_rs::kstring::KsPrintfArg::Int(t as i32),
            ],
        );
        if r < 0 {
            break;
        }

        (*tmpname).data.push(0);
        fp = NonNull::new(crate::htslib_rs::hfile::hopen(
            (*tmpname).data.as_ptr().cast(),
            mode.cast(),
        ));
        (*tmpname).data.pop();
        if fp.is_some() {
            break;
        }
        let errno = *libc::__errno_location();
        if errno != libc::EEXIST || n >= 100 {
            break;
        }
    }

    fp.map_or(std::ptr::null_mut(), NonNull::as_ptr)
}

pub unsafe fn hts_open_format(
    fn_: *const c_char,
    mode: *const c_char,
    fmt: *const htsFormat,
) -> *mut htsFile {
    let mut smode = [0 as c_char; 101];
    let mut fmt_code = 0 as c_char;
    let mut uncomp: *mut c_char = std::ptr::null_mut();
    let hfile: Option<NonNull<hFILE>>;
    let format_to_mode = b"\0g\0\0b\0c\0\0b\0g\0\0\0\0\0Ff\0\0";

    libc::strncpy(smode.as_mut_ptr(), mode, 99);
    smode[99] = 0;
    let comma = libc::strchr(smode.as_mut_ptr(), b',' as i32);
    if !comma.is_null() {
        *comma = 0;
    }

    let mut cp = smode.as_mut_ptr();
    let mut cp2 = smode.as_mut_ptr();
    while *cp != 0 {
        if *cp == b'b' as c_char {
            fmt_code = b'b' as c_char;
        } else if *cp == b'c' as c_char {
            fmt_code = b'c' as c_char;
        } else {
            *cp2 = *cp;
            cp2 = cp2.add(1);
            if uncomp.is_null() && *cp == b'u' as c_char {
                uncomp = cp2.sub(1);
            }
        }
        cp = cp.add(1);
    }
    let mode_c = cp2;
    *cp2 = fmt_code;
    cp2 = cp2.add(1);
    *cp2 = 0;

    if !fmt.is_null()
        && (*fmt).format > HTS_FORMAT_UNKNOWN_FORMAT
        && ((*fmt).format as usize) < format_to_mode.len()
    {
        *mode_c = format_to_mode[(*fmt).format as usize] as c_char;
    }

    if !uncomp.is_null()
        && *mode_c == b'b' as c_char
        && (!libc::strchr(smode.as_ptr(), b'w' as i32).is_null()
            || !libc::strchr(smode.as_ptr(), b'a' as i32).is_null())
    {
        *uncomp = b'0' as c_char;
    }

    if !libc::strchr(mode, b'w' as i32).is_null()
        && !fmt.is_null()
        && (*fmt).compression == HTS_COMPRESSION_BGZF
        && ((*fmt).format == HTS_FORMAT_SAM
            || (*fmt).format == HTS_FORMAT_VCF
            || (*fmt).format == HTS_FORMAT_TEXT_FORMAT)
    {
        *mode_c = b'z' as c_char;
    }

    let idx_delim = c"##idx##";
    let fnidx = libc::strstr(fn_, idx_delim.as_ptr());
    let mut stripped_name: Vec<c_char>;
    let fn_open = if !fnidx.is_null() {
        stripped_name = CStr::from_ptr(fn_)
            .to_bytes_with_nul()
            .iter()
            .map(|&b| b as c_char)
            .collect::<Vec<_>>();
        stripped_name[fnidx.offset_from(fn_) as usize] = 0;
        stripped_name.as_mut_ptr()
    } else {
        fn_
    };

    hfile = NonNull::new(hopen(fn_open.cast(), smode.as_ptr().cast()));
    let Some(hfile_ptr) = hfile else {
        goto_hts_open_format_error(fn_open, hfile);
        return std::ptr::null_mut();
    };

    let fp = hts_hopen(hfile_ptr.as_ptr(), fn_open, smode.as_ptr());
    if fp.is_null() {
        goto_hts_open_format_error(fn_open, hfile);
        return std::ptr::null_mut();
    }

    if ((*fp).bitfields & (1 << 1)) != 0
        && !fmt.is_null()
        && ((*fmt).format == HTS_FORMAT_BAM
            || (*fmt).format == HTS_FORMAT_SAM
            || (*fmt).format == HTS_FORMAT_VCF
            || (*fmt).format == HTS_FORMAT_BCF
            || (*fmt).format == HTS_FORMAT_BED
            || (*fmt).format == HTS_FORMAT_FASTA_FORMAT
            || (*fmt).format == HTS_FORMAT_FASTQ_FORMAT)
    {
        (*fp).format.format = (*fmt).format;
    }

    if !fmt.is_null()
        && !(*fmt).specific.is_null()
        && hts_opt_apply(fp, (*fmt).specific.cast()) != 0
    {
        let opt = (*fmt).specific.cast::<hts_opt>();
        let errno = *libc::__errno_location();
        if !opt.is_null()
            && (*opt).opt == CRAM_OPT_REFERENCE
            && (errno == libc::ENOENT
                || errno == libc::EIO
                || errno == libc::EBADF
                || errno == libc::EACCES
                || errno == libc::EISDIR)
        {
            *libc::__errno_location() = libc::EINVAL;
        }
        goto_hts_open_format_error(fn_open, hfile);
        return std::ptr::null_mut();
    }

    fp
}

unsafe fn goto_hts_open_format_error(fn_: *const c_char, hfile: Option<NonNull<hFILE>>) {
    eprintln!(
        "[E::hts_open_format] Failed to open file \"{}\"",
        String::from_utf8_lossy(CStr::from_ptr(fn_).to_bytes()),
    );
    if let Some(hfile) = hfile {
        hclose_abruptly(hfile.as_ptr());
    }
}

pub unsafe fn hts_open(fn_: *const c_char, mode: *const c_char) -> *mut htsFile {
    hts_open_format(fn_, mode, std::ptr::null())
}

pub unsafe fn hts_hopen(fp: *mut hFILE, fn_: *const c_char, mode: *const c_char) -> *mut htsFile {
    if fp.is_null() || fn_.is_null() || mode.is_null() {
        *libc::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }

    // `htsFile` contains a `kstring_t` (owning a `Vec<u8>`), so `mem::zeroed`
    // is an invalid zero-init that aborts at runtime; `Default` builds the same
    // all-null/empty state soundly (empty kstring, null pointers, zero ints).
    let hts_fp = Box::into_raw(Box::new(htsFile::default()));
    // strdup(fn_): copy the nul-terminated name into an owned byte buffer and
    // hand its raw pointer to the still-raw `fn_` field.
    let fn_owned = CStr::from_ptr(fn_).to_bytes_with_nul().to_vec().into_boxed_slice();
    (*hts_fp).fn_ = Box::into_raw(fn_owned).cast::<c_char>();
    if ed_is_big() != 0 {
        (*hts_fp).bitfields |= 1 << 2;
    }

    let mode_len = libc::strlen(mode);
    let comma = libc::strchr(mode, b',' as i32);
    let simple_len = if comma.is_null() {
        mode_len.min(100)
    } else {
        (comma.offset_from(mode) as usize).min(100)
    };
    let mut simple_mode = [0 as c_char; 101];
    if simple_len > 0 {
        std::ptr::copy_nonoverlapping(mode, simple_mode.as_mut_ptr(), simple_len);
    }
    simple_mode[simple_len] = 0;
    let opts = if comma.is_null() {
        std::ptr::null()
    } else {
        comma.add(1)
    };

    if !libc::strchr(simple_mode.as_ptr(), b'r' as i32).is_null() {
        if hts_detect_format2(fp, fn_, &mut (*hts_fp).format) < 0 {
            goto_hts_hopen_error(hts_fp);
            return std::ptr::null_mut();
        }
    } else if !libc::strchr(simple_mode.as_ptr(), b'w' as i32).is_null()
        || !libc::strchr(simple_mode.as_ptr(), b'a' as i32).is_null()
    {
        (*hts_fp).bitfields |= 1 << 1;
        let fmt = &mut (*hts_fp).format;
        if !libc::strchr(simple_mode.as_ptr(), b'b' as i32).is_null() {
            fmt.format = HTS_FORMAT_BINARY_FORMAT;
        } else if !libc::strchr(simple_mode.as_ptr(), b'c' as i32).is_null() {
            fmt.format = HTS_FORMAT_CRAM;
        } else if !libc::strchr(simple_mode.as_ptr(), b'f' as i32).is_null() {
            fmt.format = HTS_FORMAT_FASTQ_FORMAT;
        } else if !libc::strchr(simple_mode.as_ptr(), b'F' as i32).is_null() {
            fmt.format = HTS_FORMAT_FASTA_FORMAT;
        } else {
            fmt.format = HTS_FORMAT_TEXT_FORMAT;
        }

        if !libc::strchr(simple_mode.as_ptr(), b'z' as i32).is_null() {
            fmt.compression = HTS_COMPRESSION_BGZF;
        } else if !libc::strchr(simple_mode.as_ptr(), b'g' as i32).is_null() {
            fmt.compression = HTS_COMPRESSION_GZIP;
        } else if !libc::strchr(simple_mode.as_ptr(), b'u' as i32).is_null() {
            fmt.compression = HTS_COMPRESSION_NO_COMPRESSION;
        } else {
            fmt.compression = match fmt.format {
                HTS_FORMAT_BINARY_FORMAT => HTS_COMPRESSION_BGZF,
                HTS_FORMAT_CRAM => HTS_COMPRESSION_CUSTOM,
                HTS_FORMAT_FASTQ_FORMAT | HTS_FORMAT_FASTA_FORMAT | HTS_FORMAT_TEXT_FORMAT => {
                    HTS_COMPRESSION_NO_COMPRESSION
                }
                _ => {
                    goto_hts_hopen_error(hts_fp);
                    return std::ptr::null_mut();
                }
            };
        }
        fmt.category = format_category(fmt.format);
        fmt.version.major = -1;
        fmt.version.minor = -1;
        fmt.compression_level = -1;
        fmt.specific = std::ptr::null_mut();
    } else {
        *libc::__errno_location() = libc::EINVAL;
        goto_hts_hopen_error(hts_fp);
        return std::ptr::null_mut();
    }

    match (*hts_fp).format.format {
        HTS_FORMAT_BINARY_FORMAT | HTS_FORMAT_BAM | HTS_FORMAT_BCF => {
            (*hts_fp).fp.bgzf = bgzf_hopen(fp, simple_mode.as_ptr().cast());
            if (*hts_fp).fp.bgzf.is_null() {
                goto_hts_hopen_error(hts_fp);
                return std::ptr::null_mut();
            }
            if ((*hts_fp).bitfields & (1 << 1)) == 0 && (*hts_fp).format.format == HTS_FORMAT_BAM {
                (*(*hts_fp).fp.bgzf).bitfields |= BGZF_HTS_OPEN_FAST_BAM_READ;
            }
            (*hts_fp).bitfields |= 1 | (1 << 4);
        }
        HTS_FORMAT_CRAM => {
            (*hts_fp).fp.cram = cram_dopen(fp, fn_.cast(), simple_mode.as_ptr().cast());
            if (*hts_fp).fp.cram.is_null() {
                goto_hts_hopen_error(hts_fp);
                return std::ptr::null_mut();
            }
            if ((*hts_fp).bitfields & (1 << 1)) == 0 {
                if let Some(cram) = (*hts_fp).fp.cram.as_mut() {
                    crate::cram_options_bridge::cram_set_option_int(cram, CRAM_OPT_DECODE_MD, -1);
                }
            }
            (*hts_fp).bitfields |= 1 << 3;
        }
        HTS_FORMAT_EMPTY_FORMAT
        | HTS_FORMAT_TEXT_FORMAT
        | HTS_FORMAT_BED
        | HTS_FORMAT_FASTA_FORMAT
        | HTS_FORMAT_FASTQ_FORMAT
        | HTS_FORMAT_SAM
        | HTS_FORMAT_VCF => {
            if (*hts_fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
                (*hts_fp).fp.bgzf = bgzf_hopen(fp, simple_mode.as_ptr().cast());
                if (*hts_fp).fp.bgzf.is_null() {
                    goto_hts_hopen_error(hts_fp);
                    return std::ptr::null_mut();
                }
                (*hts_fp).bitfields |= 1 << 4;
            } else {
                (*hts_fp).fp.hfile = fp;
            }
        }
        _ => {
            *libc::__errno_location() = libc::ENOEXEC;
            goto_hts_hopen_error(hts_fp);
            return std::ptr::null_mut();
        }
    }

    if !opts.is_null() {
        let _ = hts_c_1413_hts_process_opts(hts_fp, opts);
    }

    hts_fp
}

unsafe fn goto_hts_hopen_error(fp: *mut htsFile) {
    if !fp.is_null() {
        if !(*fp).fn_.is_null() {
            let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*fp).fn_.cast::<u8>(),
                len,
            )));
        }
        if !(*fp).fn_aux.is_null() {
            let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*fp).fn_aux.cast::<u8>(),
                len,
            )));
        }
        drop(Box::from_raw(fp));
    }
}

pub unsafe fn hts_detect_format(fp: *mut hFILE, fmt: *mut htsFormat) -> i32 {
    hts_detect_format2(fp, std::ptr::null(), fmt)
}

pub unsafe fn hts_detect_format2(
    fp: *mut hFILE,
    fname: *const c_char,
    fmt: *mut htsFormat,
) -> i32 {
    hts_c_556_hts_detect_format2(fp, fname, fmt)
}

pub unsafe fn hts_c_556_hts_detect_format2(
    hfile: *mut hFILE,
    fname: *const c_char,
    fmt: *mut htsFormat,
) -> i32 {
    let mut extension = [0 as c_char; HTS_MAX_EXT_LEN];
    let mut columns = [0 as c_char; 24];
    let mut s = [0u8; 1024];
    let mut complete = 0;
    let mut len = hpeek(hfile, s.as_mut_ptr().cast(), 18);
    if len < 0 {
        return -1;
    }

    (*fmt).category = HTS_FORMAT_UNKNOWN_CATEGORY;
    (*fmt).format = HTS_FORMAT_UNKNOWN_FORMAT;
    (*fmt).version.major = -1;
    (*fmt).version.minor = -1;
    (*fmt).compression = HTS_COMPRESSION_NO_COMPRESSION;
    (*fmt).compression_level = -1;
    (*fmt).specific = std::ptr::null_mut();

    if len >= 2 && s[0] == 0x1f && s[1] == 0x8b {
        (*fmt).compression = HTS_COMPRESSION_GZIP;
        if len >= 18 && (s[3] & 4) != 0 {
            if &s[12..16] == b"BC\x02\x00" {
                (*fmt).compression = HTS_COMPRESSION_BGZF;
            } else if &s[12..16] == b"RAZF" {
                (*fmt).compression = HTS_COMPRESSION_RAZF;
            }
        }
        if len >= 9 && s[2] == 8 {
            (*fmt).compression_level = if s[8] == 2 {
                9
            } else if s[8] == 4 {
                1
            } else {
                -1
            };
        }
        len = hts_c_313_decompress_peek_gz(hfile, s.as_mut_ptr(), s.len());
    } else if len >= 10
        && &s[..3] == b"BZh"
        && (&s[4..10] == b"\x31\x41\x59\x26\x53\x59" || &s[4..10] == b"\x17\x72\x45\x38\x50\x90")
    {
        (*fmt).compression = HTS_COMPRESSION_BZIP2;
        (*fmt).compression_level = (s[3] - b'0') as i16;
        if s[4] == b'\x31' {
            return 0;
        } else {
            len = 0;
        }
    } else if len >= 6 && &s[..6] == b"\xfd\x37\x7a\x58\x5a\x00" {
        (*fmt).compression = HTS_COMPRESSION_XZ;
        len = hts_c_356_decompress_peek_xz(hfile, s.as_mut_ptr(), s.len());
    } else if len >= 4 && &s[..4] == b"\x28\xb5\x2f\xfd" {
        (*fmt).compression = HTS_COMPRESSION_ZSTD;
        return 0;
    } else {
        len = hpeek(hfile, s.as_mut_ptr().cast(), s.len());
    }
    if len < 0 {
        return -1;
    }
    let len_usize = len as usize;

    if len == 0 {
        (*fmt).format = HTS_FORMAT_EMPTY_FORMAT;
        return 0;
    }

    if !fname.is_null() && libc::strcmp(fname, c"-".as_ptr()) != 0 {
        if find_file_extension(fname, extension.as_mut_ptr()) < 0 {
            extension[0] = 0;
        }
        let mut p = extension.as_mut_ptr();
        while *p != 0 {
            *p = tolower_c(*p as u8) as c_char;
            p = p.add(1);
        }
    } else {
        extension[0] = 0;
    }

    if len >= 6 && &s[..4] == b"CRAM" && s[4] >= 1 && s[4] <= 7 && s[5] <= 7 {
        (*fmt).category = HTS_FORMAT_SEQUENCE_DATA;
        (*fmt).format = HTS_FORMAT_CRAM;
        (*fmt).version.major = s[4] as i16;
        (*fmt).version.minor = s[5] as i16;
        (*fmt).compression = HTS_COMPRESSION_CUSTOM;
        return 0;
    } else if len >= 4 && s[3] <= 4 {
        if &s[..4] == b"BAM\x01" {
            (*fmt).category = HTS_FORMAT_SEQUENCE_DATA;
            (*fmt).format = HTS_FORMAT_BAM;
            (*fmt).version.major = 1;
            (*fmt).version.minor = -1;
            return 0;
        } else if &s[..4] == b"BAI\x01" {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_BAI;
            (*fmt).version.major = -1;
            (*fmt).version.minor = -1;
            return 0;
        } else if &s[..4] == b"BCF\x04" {
            (*fmt).category = HTS_FORMAT_VARIANT_DATA;
            (*fmt).format = HTS_FORMAT_BCF;
            (*fmt).version.major = 1;
            (*fmt).version.minor = -1;
            return 0;
        } else if &s[..4] == b"BCF\x02" {
            (*fmt).category = HTS_FORMAT_VARIANT_DATA;
            (*fmt).format = HTS_FORMAT_BCF;
            (*fmt).version.major = s[3] as i16;
            (*fmt).version.minor = if len >= 5 && s[4] <= 2 {
                s[4] as i16
            } else {
                0
            };
            return 0;
        } else if &s[..4] == b"CSI\x01" {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_CSI;
            (*fmt).version.major = 1;
            (*fmt).version.minor = -1;
            return 0;
        } else if &s[..4] == b"TBI\x01" {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_TBI;
            return 0;
        } else if libc::strcmp(extension.as_ptr(), c"gzi".as_ptr()) == 0 {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_GZI;
            return 0;
        }
    } else if len >= 16 && &s[..16] == b"##fileformat=VCF" {
        (*fmt).category = HTS_FORMAT_VARIANT_DATA;
        (*fmt).format = HTS_FORMAT_VCF;
        if len >= 21 && s[16] == b'v' {
            parse_version(fmt, s.as_ptr().add(17), s.as_ptr().add(len_usize));
        }
        return 0;
    } else if len >= 4
        && s[0] == b'@'
        && (&s[..4] == b"@HD\t"
            || &s[..4] == b"@SQ\t"
            || &s[..4] == b"@RG\t"
            || &s[..4] == b"@PG\t"
            || &s[..4] == b"@CO\t")
    {
        (*fmt).category = HTS_FORMAT_SEQUENCE_DATA;
        (*fmt).format = HTS_FORMAT_SAM;
        if len >= 9 && &s[..7] == b"@HD\tVN:" {
            parse_version(fmt, s.as_ptr().add(7), s.as_ptr().add(len_usize));
        } else {
            (*fmt).version.major = 1;
            (*fmt).version.minor = -1;
        }
        return 0;
    } else if len >= 8 && &s[..4] == b"d4\xdd\xdd" {
        (*fmt).category = HTS_FORMAT_REGION_LIST;
        (*fmt).format = HTS_FORMAT_D4_FORMAT;
        return 0;
    } else if cmp_nonblank(
        c"{\"htsget\":".as_ptr(),
        s.as_ptr(),
        s.as_ptr().add(len_usize),
    ) == 0
    {
        (*fmt).category = HTS_FORMAT_UNKNOWN_CATEGORY;
        (*fmt).format = HTS_FORMAT_HTSGET;
        return 0;
    } else if len > 8 && &s[..8] == b"crypt4gh" {
        (*fmt).category = HTS_FORMAT_UNKNOWN_CATEGORY;
        (*fmt).format = HTS_FORMAT_CRYPT4GH_FORMAT;
        return 0;
    } else if len >= 1
        && s[0] == b'>'
        && hts_c_458_is_fastaq(s.as_ptr(), s.as_ptr().add(len_usize)) != 0
    {
        (*fmt).category = HTS_FORMAT_SEQUENCE_DATA;
        (*fmt).format = HTS_FORMAT_FASTA_FORMAT;
        return 0;
    } else if len >= 1
        && s[0] == b'@'
        && hts_c_458_is_fastaq(s.as_ptr(), s.as_ptr().add(len_usize)) != 0
    {
        (*fmt).category = HTS_FORMAT_SEQUENCE_DATA;
        (*fmt).format = HTS_FORMAT_FASTQ_FORMAT;
        return 0;
    } else if hts_c_483_parse_tabbed_text(
        columns.as_mut_ptr(),
        columns.len() as i32,
        s.as_ptr(),
        s.as_ptr().add(len_usize),
        &mut complete,
    ) > 0
    {
        if hts_c_540_colmatch(
            columns.as_ptr(),
            c"ZiZiiCZiiZZOOOOOOOOOOOOOOOOOOOO+".as_ptr(),
        ) >= 9 + 2 * complete
        {
            (*fmt).category = HTS_FORMAT_SEQUENCE_DATA;
            (*fmt).format = HTS_FORMAT_SAM;
            (*fmt).version.major = 1;
            (*fmt).version.minor = -1;
            return 0;
        } else if (*fmt).compression == HTS_COMPRESSION_GZIP
            && hts_c_540_colmatch(columns.as_ptr(), c"iiiiii".as_ptr()) == 6
        {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_CRAI_EXACT;
            return 0;
        } else if !libc::strstr(extension.as_ptr(), c"fqi".as_ptr()).is_null()
            && hts_c_540_colmatch(columns.as_ptr(), c"Ziiiii".as_ptr()) == 6
        {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_FQI_FORMAT;
            return 0;
        } else if !libc::strstr(extension.as_ptr(), c"fai".as_ptr()).is_null()
            && hts_c_540_colmatch(columns.as_ptr(), c"Ziiii".as_ptr()) == 5
        {
            (*fmt).category = HTS_FORMAT_INDEX_FILE;
            (*fmt).format = HTS_FORMAT_FAI_FORMAT;
            return 0;
        } else if hts_c_540_colmatch(columns.as_ptr(), c"Zii+".as_ptr()) >= 3 {
            (*fmt).category = HTS_FORMAT_REGION_LIST;
            (*fmt).format = HTS_FORMAT_BED;
            return 0;
        }
    }

    if is_text_only(s.as_ptr(), s.as_ptr().add(len_usize)) != 0 {
        (*fmt).format = HTS_FORMAT_TEXT_FORMAT;
    }

    0
}

pub unsafe fn hts_format_description(format: *const htsFormat) -> *mut c_char {
    hts_c_775_hts_format_description(format)
}

pub unsafe fn hts_c_775_hts_format_description(format: *const htsFormat) -> *mut c_char {
    const RAZF_COMPRESSION: htsCompression = 5;
    const XZ_COMPRESSION: htsCompression = 6;
    const ZSTD_COMPRESSION: htsCompression = 7;

    let mut str_: kstring_t = kstring_t::default();

    match (*format).format {
        x if x == HTS_FORMAT_SAM => {
            kputs(b"SAM", &mut str_);
        }
        x if x == HTS_FORMAT_BAM => {
            kputs(b"BAM", &mut str_);
        }
        x if x == HTS_FORMAT_CRAM => {
            kputs(b"CRAM", &mut str_);
        }
        x if x == HTS_FORMAT_FASTA_FORMAT => {
            kputs(b"FASTA", &mut str_);
        }
        x if x == HTS_FORMAT_FASTQ_FORMAT => {
            kputs(b"FASTQ", &mut str_);
        }
        x if x == HTS_FORMAT_VCF => {
            kputs(b"VCF", &mut str_);
        }
        x if x == HTS_FORMAT_BCF => {
            if (*format).version.major == 1 {
                kputs(b"Legacy BCF", &mut str_);
            } else {
                kputs(b"BCF", &mut str_);
            }
        }
        x if x == HTS_FORMAT_BAI => {
            kputs(b"BAI", &mut str_);
        }
        x if x == HTS_FORMAT_CRAI_EXACT => {
            kputs(b"CRAI", &mut str_);
        }
        x if x == HTS_FORMAT_CSI => {
            kputs(b"CSI", &mut str_);
        }
        x if x == HTS_FORMAT_FAI_FORMAT => {
            kputs(b"FASTA-IDX", &mut str_);
        }
        x if x == HTS_FORMAT_FQI_FORMAT => {
            kputs(b"FASTQ-IDX", &mut str_);
        }
        x if x == HTS_FORMAT_GZI => {
            kputs(b"GZI", &mut str_);
        }
        x if x == HTS_FORMAT_TBI => {
            kputs(b"Tabix", &mut str_);
        }
        x if x == HTS_FORMAT_BED => {
            kputs(b"BED", &mut str_);
        }
        x if x == HTS_FORMAT_D4_FORMAT => {
            kputs(b"D4", &mut str_);
        }
        x if x == HTS_FORMAT_HTSGET => {
            kputs(b"htsget", &mut str_);
        }
        x if x == HTS_FORMAT_CRYPT4GH_FORMAT => {
            kputs(b"crypt4gh", &mut str_);
        }
        x if x == HTS_FORMAT_EMPTY_FORMAT => {
            kputs(b"empty", &mut str_);
        }
        _ => {
            kputs(b"unknown", &mut str_);
        }
    }

    if (*format).version.major >= 0 {
        kputs(b" version ", &mut str_);
        kputw((*format).version.major as i32, &mut str_);
        if (*format).version.minor >= 0 {
            kputc(b'.' as i32, &mut str_);
            kputw((*format).version.minor as i32, &mut str_);
        }
    }

    match (*format).compression {
        x if x == HTS_COMPRESSION_BZIP2 => {
            kputs(b" bzip2-compressed", &mut str_);
        }
        RAZF_COMPRESSION => {
            kputs(b" legacy-RAZF-compressed", &mut str_);
        }
        XZ_COMPRESSION => {
            kputs(b" XZ-compressed", &mut str_);
        }
        ZSTD_COMPRESSION => {
            kputs(b" Zstandard-compressed", &mut str_);
        }
        x if x == HTS_COMPRESSION_CUSTOM => {
            kputs(b" compressed", &mut str_);
        }
        x if x == HTS_COMPRESSION_GZIP => {
            kputs(b" gzip-compressed", &mut str_);
        }
        x if x == HTS_COMPRESSION_BGZF => match (*format).format {
            x if x == HTS_FORMAT_BAM
                || x == HTS_FORMAT_BCF
                || x == HTS_FORMAT_CSI
                || x == HTS_FORMAT_TBI =>
            {
                kputs(b" compressed", &mut str_);
            }
            _ => {
                kputs(b" BGZF-compressed", &mut str_);
            }
        },
        x if x == HTS_COMPRESSION_NO_COMPRESSION => match (*format).format {
            x if x == HTS_FORMAT_BAM
                || x == HTS_FORMAT_BCF
                || x == HTS_FORMAT_CRAM
                || x == HTS_FORMAT_CSI
                || x == HTS_FORMAT_TBI =>
            {
                kputs(b" uncompressed", &mut str_);
            }
            _ => {}
        },
        _ => {}
    }

    match (*format).category {
        x if x == HTS_FORMAT_SEQUENCE_DATA => {
            kputs(b" sequence", &mut str_);
        }
        x if x == HTS_FORMAT_VARIANT_DATA => {
            kputs(b" variant calling", &mut str_);
        }
        x if x == HTS_FORMAT_INDEX_FILE => {
            kputs(b" index", &mut str_);
        }
        x if x == HTS_FORMAT_REGION_LIST => {
            kputs(b" genomic region", &mut str_);
        }
        _ => {}
    }

    if (*format).compression == HTS_COMPRESSION_NO_COMPRESSION {
        match (*format).format {
            x if x == HTS_FORMAT_TEXT_FORMAT
                || x == HTS_FORMAT_SAM
                || x == HTS_FORMAT_CRAI_EXACT
                || x == HTS_FORMAT_VCF
                || x == HTS_FORMAT_BED
                || x == HTS_FORMAT_FAI_FORMAT
                || x == HTS_FORMAT_FQI_FORMAT
                || x == HTS_FORMAT_FASTA_FORMAT
                || x == HTS_FORMAT_FASTQ_FORMAT
                || x == HTS_FORMAT_HTSGET =>
            {
                kputs(b" text", &mut str_);
            }
            x if x == HTS_FORMAT_EMPTY_FORMAT => {}
            _ => {
                kputs(b" data", &mut str_);
            }
        }
    } else {
        kputs(b" data", &mut str_);
    }

    let released = ks_release(&mut str_);
    // Return an owned, nul-terminated C string; hand its raw pointer out.
    let mut owned = released;
    owned.push(0);
    Box::into_raw(owned.into_boxed_slice()).cast::<c_char>()
}

pub unsafe fn hts_opt_add(opts: *mut *mut hts_opt, c_arg: *const c_char) -> i32 {
    hts_c_1021_hts_opt_add(opts, c_arg)
}

pub unsafe fn hts_c_1002_scan_keyword(
    mut str_: *const c_char,
    delim: c_char,
    buf: *mut c_char,
    buflen: usize,
) -> *const c_char {
    let mut i = 0usize;
    while *str_ != 0 && *str_ != delim {
        if i < buflen - 1 {
            *buf.add(i) = tolower_c(*str_ as u8) as c_char;
            i += 1;
        }
        str_ = str_.add(1);
    }
    *buf.add(i) = 0;
    if *str_ != 0 {
        str_.add(1)
    } else {
        str_
    }
}

fn hts_c_1021_opt_key_matches(key: &[u8], lower: &[u8]) -> bool {
    key == lower
        || (key.len() == lower.len()
            && key
                .iter()
                .zip(lower.iter())
                .all(|(&k, &l)| k == l.to_ascii_uppercase()))
}

pub unsafe fn hts_c_1021_hts_opt_add(opts: *mut *mut hts_opt, c_arg: *const c_char) -> i32 {
    if c_arg.is_null() {
        return -1;
    }

    let mut arg_buf = CStr::from_ptr(c_arg)
        .to_bytes_with_nul()
        .iter()
        .map(|&b| b as c_char)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let arg = NonNull::new(arg_buf.as_mut_ptr()).expect("boxed option argument is non-empty");
    let mut o = Box::new(hts_opt {
        arg,
        opt: 0,
        val: hts_opt_val { i: 0 },
        next: None,
        _arg_buf: arg_buf,
    });

    let mut val = libc::strchr(o.arg.as_ptr(), b'=' as i32);
    if val.is_null() {
        val = c"1".as_ptr().cast_mut();
    } else {
        *val = 0;
        val = val.add(1);
    }

    let key = CStr::from_ptr(o.arg.as_ptr()).to_bytes();
    let mut endp: *mut c_char = std::ptr::null_mut();
    if hts_c_1021_opt_key_matches(key, b"decode_md") {
        o.opt = CRAM_OPT_DECODE_MD;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"verbosity") {
        o.opt = CRAM_OPT_VERBOSITY;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"seqs_per_slice") {
        o.opt = CRAM_OPT_SEQS_PER_SLICE;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"bases_per_slice") {
        o.opt = CRAM_OPT_BASES_PER_SLICE;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"slices_per_container") {
        o.opt = CRAM_OPT_SLICES_PER_CONTAINER;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"embed_ref") {
        o.opt = CRAM_OPT_EMBED_REF;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"no_ref") {
        o.opt = CRAM_OPT_NO_REF;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"pos_delta") {
        o.opt = CRAM_OPT_POS_DELTA;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"ignore_md5") {
        o.opt = CRAM_OPT_IGNORE_MD5;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"use_bzip2") {
        o.opt = CRAM_OPT_USE_BZIP2;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"use_rans") {
        o.opt = CRAM_OPT_USE_RANS;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"use_lzma") {
        o.opt = CRAM_OPT_USE_LZMA;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"use_tok") {
        o.opt = CRAM_OPT_USE_TOK;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"use_fqz") {
        o.opt = CRAM_OPT_USE_FQZ;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"use_arith") {
        o.opt = CRAM_OPT_USE_ARITH;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"fast") {
        o.opt = HTS_OPT_PROFILE;
        o.val.i = HTS_PROFILE_FAST;
    } else if hts_c_1021_opt_key_matches(key, b"normal") {
        o.opt = HTS_OPT_PROFILE;
        o.val.i = HTS_PROFILE_NORMAL;
    } else if hts_c_1021_opt_key_matches(key, b"small") {
        o.opt = HTS_OPT_PROFILE;
        o.val.i = HTS_PROFILE_SMALL;
    } else if hts_c_1021_opt_key_matches(key, b"archive") {
        o.opt = HTS_OPT_PROFILE;
        o.val.i = HTS_PROFILE_ARCHIVE;
    } else if hts_c_1021_opt_key_matches(key, b"reference") {
        o.opt = CRAM_OPT_REFERENCE;
        o.val.s = val;
    } else if hts_c_1021_opt_key_matches(key, b"version") {
        o.opt = CRAM_OPT_VERSION;
        o.val.s = val;
    } else if hts_c_1021_opt_key_matches(key, b"multi_seq_per_slice") {
        o.opt = CRAM_OPT_MULTI_SEQ_PER_SLICE;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"nthreads") {
        o.opt = HTS_OPT_NTHREADS;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"cache_size") {
        o.opt = HTS_OPT_CACHE_SIZE;
        o.val.i = libc::strtol(val, &mut endp, 0) as i32;
        match *endp {
            x if x == b'g' as c_char || x == b'G' as c_char => {
                o.val.i *= 1024;
                o.val.i *= 1024;
                o.val.i *= 1024;
            }
            x if x == b'm' as c_char || x == b'M' as c_char => {
                o.val.i *= 1024;
                o.val.i *= 1024;
            }
            x if x == b'k' as c_char || x == b'K' as c_char => {
                o.val.i *= 1024;
            }
            0 => {}
            _ => {
                return -1;
            }
        }
    } else if hts_c_1021_opt_key_matches(key, b"required_fields") {
        o.opt = CRAM_OPT_REQUIRED_FIELDS;
        o.val.i = libc::strtol(val, std::ptr::null_mut(), 0) as i32;
    } else if hts_c_1021_opt_key_matches(key, b"lossy_names") {
        o.opt = CRAM_OPT_LOSSY_NAMES;
        o.val.i = libc::strtol(val, std::ptr::null_mut(), 0) as i32;
    } else if hts_c_1021_opt_key_matches(key, b"name_prefix") {
        o.opt = CRAM_OPT_PREFIX;
        o.val.s = val;
    } else if key == b"store_md" {
        o.opt = CRAM_OPT_STORE_MD;
        o.val.i = libc::atoi(val);
    } else if key == b"store_nm" {
        o.opt = CRAM_OPT_STORE_NM;
        o.val.i = libc::atoi(val);
    } else if hts_c_1021_opt_key_matches(key, b"block_size") {
        o.opt = HTS_OPT_BLOCK_SIZE;
        o.val.i = libc::strtol(val, std::ptr::null_mut(), 0) as i32;
    } else if hts_c_1021_opt_key_matches(key, b"level") {
        o.opt = HTS_OPT_COMPRESSION_LEVEL;
        o.val.i = libc::strtol(val, std::ptr::null_mut(), 0) as i32;
    } else if hts_c_1021_opt_key_matches(key, b"filter") {
        o.opt = HTS_OPT_FILTER;
        o.val.s = val;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_aux") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_AUX as hts_fmt_option;
        o.val.s = val;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_barcode") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_BARCODE as hts_fmt_option;
        o.val.s = val;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_rnum") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_RNUM as hts_fmt_option;
        o.val.i = 1;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_casava") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_CASAVA as hts_fmt_option;
        o.val.i = 1;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_name2") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_NAME2 as hts_fmt_option;
        o.val.i = 1;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_umi") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_UMI as hts_fmt_option;
        o.val.s = val;
    } else if hts_c_1021_opt_key_matches(key, b"fastq_umi_regex") {
        o.opt = crate::htslib_rs::sam::FASTQ_OPT_UMI_REGEX as hts_fmt_option;
        o.val.s = val;
    } else {
        return -1;
    }

    let o = Box::into_raw(o);
    if !(*opts).is_null() {
        let mut t = *opts;
        while let Some(next) = (*t).next {
            t = next.as_ptr();
        }
        (*t).next = NonNull::new(o);
    } else {
        *opts = o;
    }
    0
}

pub unsafe fn hts_opt_apply(fp: *mut htsFile, opts: *mut hts_opt) -> i32 {
    hts_c_1247_hts_opt_apply(fp, opts)
}

pub unsafe fn hts_c_1247_hts_opt_apply(fp: *mut htsFile, mut opts: *mut hts_opt) -> i32 {
    while !opts.is_null() {
        match (*opts).opt {
            x if x == CRAM_OPT_REFERENCE => {
                let owned = CStr::from_ptr((*opts).val.s)
                    .to_bytes_with_nul()
                    .to_vec()
                    .into_boxed_slice();
                (*fp).fn_aux = Box::into_raw(owned).cast::<c_char>();
                if hts_set_opt_ptr(fp, (*opts).opt, (*opts).val.s.cast()) != 0 {
                    return -1;
                }
            }
            x if x == CRAM_OPT_VERSION
                || x == CRAM_OPT_PREFIX
                || x == HTS_OPT_FILTER
                || x == crate::htslib_rs::sam::FASTQ_OPT_AUX as hts_fmt_option
                || x == crate::htslib_rs::sam::FASTQ_OPT_BARCODE as hts_fmt_option
                || x == crate::htslib_rs::sam::FASTQ_OPT_UMI as hts_fmt_option
                || x == crate::htslib_rs::sam::FASTQ_OPT_UMI_REGEX as hts_fmt_option =>
            {
                if hts_set_opt_ptr(fp, (*opts).opt, (*opts).val.s.cast()) != 0 {
                    return -1;
                }
            }
            _ => {
                if hts_set_opt_int(fp, (*opts).opt, (*opts).val.i) != 0 {
                    return -1;
                }
            }
        }
        opts = (*opts).next.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    }
    0
}

pub unsafe fn hts_set_opt_int(fp: *mut htsFile, opt: hts_fmt_option, val: i32) -> i32 {
    match opt {
        x if x == HTS_OPT_NTHREADS => hts_set_threads(fp, val),
        x if x == HTS_OPT_CACHE_SIZE => {
            hts_set_cache_size(fp, val);
            0
        }
        x if x == HTS_OPT_BLOCK_SIZE => {
            let hf = hts_hfile(fp);
            if !hf.is_null() {
                if hfile_set_blksize(hf, val as usize) != 0 {
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        b"hts_set_opt",
                        b"Failed to change block size",
                    );
                }
            } else {
                hts_log_cstr(
                    HTS_LOG_WARNING,
                    b"hts_set_opt",
                    b"Cannot change block size for this format",
                );
            }
            0
        }
        x if x == crate::htslib_rs::sam::FASTQ_OPT_CASAVA as hts_fmt_option
            || x == crate::htslib_rs::sam::FASTQ_OPT_RNUM as hts_fmt_option
            || x == crate::htslib_rs::sam::FASTQ_OPT_NAME2 as hts_fmt_option =>
        {
            if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT
                || (*fp).format.format == HTS_FORMAT_FASTA_FORMAT
            {
                super::sam::sam_c_3815_fastq_state_set(fp, opt as i32, std::ptr::null())
            } else {
                0
            }
        }
        x if x == HTS_OPT_COMPRESSION_LEVEL => {
            if ((*fp).bitfields & (1 << 4)) != 0 {
                (*(*fp).fp.bgzf).bitfields &= !(0x1ff << 20);
                (*(*fp).fp.bgzf).bitfields |= ((val as u32) & 0x1ff) << 20;
                0
            } else if (*fp).format.format == HTS_FORMAT_CRAM {
                (*fp).fp.cram.as_mut().map_or(-1, |cram| {
                    crate::cram_options_bridge::cram_set_option_int(cram, opt, val)
                })
            } else {
                0
            }
        }
        x if x == HTS_OPT_PROFILE => {
            if ((*fp).bitfields & (1 << 4)) != 0 {
                let level = match val {
                    HTS_PROFILE_FAST => 1,
                    HTS_PROFILE_NORMAL => -1,
                    HTS_PROFILE_SMALL => 8,
                    HTS_PROFILE_ARCHIVE => 9,
                    _ => (*(*fp).fp.bgzf).bitfields.wrapping_shr(20) as i32 & 0x1ff,
                };
                (*(*fp).fp.bgzf).bitfields &= !(0x1ff << 20);
                (*(*fp).fp.bgzf).bitfields |= ((level as u32) & 0x1ff) << 20;
            }
            0
        }
        _ => {
            if (*fp).format.format == HTS_FORMAT_CRAM {
                (*fp).fp.cram.as_mut().map_or(-1, |cram| {
                    crate::cram_options_bridge::cram_set_option_int(cram, opt, val)
                })
            } else {
                0
            }
        }
    }
}

pub unsafe fn hts_set_opt_ptr(fp: *mut htsFile, opt: hts_fmt_option, val: *mut ()) -> i32 {
    match opt {
        x if x == HTS_OPT_THREAD_POOL => hts_set_thread_pool(fp, val.cast::<htsThreadPool>()),
        x if x == CRAM_OPT_REFERENCE => hts_set_fai_filename(fp, val.cast::<c_char>()),
        x if x == HTS_OPT_FILTER => hts_set_filter_expression(fp, val.cast::<c_char>()),
        x if x == crate::htslib_rs::sam::FASTQ_OPT_AUX as hts_fmt_option
            || x == crate::htslib_rs::sam::FASTQ_OPT_BARCODE as hts_fmt_option
            || x == crate::htslib_rs::sam::FASTQ_OPT_UMI as hts_fmt_option
            || x == crate::htslib_rs::sam::FASTQ_OPT_UMI_REGEX as hts_fmt_option =>
        {
            if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT
                || (*fp).format.format == HTS_FORMAT_FASTA_FORMAT
            {
                super::sam::sam_c_3815_fastq_state_set(fp, opt as i32, val.cast::<u8>())
            } else {
                0
            }
        }
        _ => {
            if (*fp).format.format == HTS_FORMAT_CRAM {
                (*fp).fp.cram.as_mut().map_or(-1, |cram| {
                    crate::cram_options_bridge::cram_set_option_ptr(cram, opt, val.cast::<()>().as_ref())
                })
            } else {
                0
            }
        }
    }
}

pub unsafe fn hts_opt_free(opts: *mut hts_opt) {
    hts_c_1279_hts_opt_free(opts)
}

pub unsafe fn hts_c_1279_hts_opt_free(mut opts: *mut hts_opt) {
    while !opts.is_null() {
        let last = opts;
        opts = (*opts).next.map_or(std::ptr::null_mut(), NonNull::as_ptr);
        drop(Box::from_raw(last));
    }
}

pub unsafe fn hts_parse_format(opt: *mut htsFormat, str_: *const c_char) -> i32 {
    hts_c_1337_hts_parse_format(opt, str_)
}

pub unsafe fn hts_c_1300_hts_parse_opt_list(fmt: *mut htsFormat, mut str_: *const c_char) -> i32 {
    while !str_.is_null() && *str_ != 0 {
        let mut arg = [0 as c_char; 8001];
        while *str_ != 0 && *str_ == b',' as c_char {
            str_ = str_.add(1);
        }
        let str_start = str_;
        while *str_ != 0 && *str_ != b',' as c_char {
            str_ = str_.add(1);
        }
        let len = str_.offset_from(str_start) as usize;
        let copy_len = if len < 8000 { len } else { 8000 };
        std::ptr::copy_nonoverlapping(str_start, arg.as_mut_ptr(), copy_len);
        arg[copy_len] = 0;
        if hts_c_1021_hts_opt_add(
            (&mut (*fmt).specific as *mut *mut c_void).cast::<*mut hts_opt>(),
            arg.as_ptr(),
        ) != 0
        {
            return -1;
        }
        if *str_ != 0 {
            str_ = str_.add(1);
        }
    }
    0
}

pub unsafe fn hts_c_1337_hts_parse_format(format: *mut htsFormat, str_: *const c_char) -> i32 {
    let mut fmt = [0 as c_char; 9];
    let cp = hts_c_1002_scan_keyword(str_, b',' as c_char, fmt.as_mut_ptr(), fmt.len());

    (*format).version.minor = 0;
    (*format).version.major = 0;

    let key = CStr::from_ptr(fmt.as_ptr()).to_bytes();
    if key == b"sam" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_SAM;
        (*format).compression = HTS_COMPRESSION_NO_COMPRESSION;
        (*format).compression_level = 0;
    } else if key == b"sam.gz" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_SAM;
        (*format).compression = HTS_COMPRESSION_BGZF;
        (*format).compression_level = -1;
    } else if key == b"bam" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_BAM;
        (*format).compression = HTS_COMPRESSION_BGZF;
        (*format).compression_level = -1;
    } else if key == b"cram" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_CRAM;
        (*format).compression = HTS_COMPRESSION_CUSTOM;
        (*format).compression_level = -1;
    } else if key == b"vcf" {
        (*format).category = HTS_FORMAT_VARIANT_DATA;
        (*format).format = HTS_FORMAT_VCF;
        (*format).compression = HTS_COMPRESSION_NO_COMPRESSION;
        (*format).compression_level = 0;
    } else if key == b"bcf" {
        (*format).category = HTS_FORMAT_VARIANT_DATA;
        (*format).format = HTS_FORMAT_BCF;
        (*format).compression = HTS_COMPRESSION_BGZF;
        (*format).compression_level = -1;
    } else if key == b"fastq" || key == b"fq" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_FASTQ_FORMAT;
        (*format).compression = HTS_COMPRESSION_NO_COMPRESSION;
        (*format).compression_level = 0;
    } else if key == b"fastq.gz" || key == b"fq.gz" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_FASTQ_FORMAT;
        (*format).compression = HTS_COMPRESSION_BGZF;
        (*format).compression_level = 0;
    } else if key == b"fasta" || key == b"fa" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_FASTA_FORMAT;
        (*format).compression = HTS_COMPRESSION_NO_COMPRESSION;
        (*format).compression_level = 0;
    } else if key == b"fasta.gz" || key == b"fa.gz" {
        (*format).category = HTS_FORMAT_SEQUENCE_DATA;
        (*format).format = HTS_FORMAT_FASTA_FORMAT;
        (*format).compression = HTS_COMPRESSION_BGZF;
        (*format).compression_level = 0;
    } else {
        return -1;
    }

    hts_c_1300_hts_parse_opt_list(format, cp)
}

pub unsafe fn hts_c_1413_hts_process_opts(fp: *mut htsFile, opts: *const c_char) -> i32 {
    let mut fmt: htsFormat = std::mem::zeroed();
    fmt.specific = std::ptr::null_mut();
    if hts_c_1300_hts_parse_opt_list(&mut fmt, opts) != 0 {
        return -1;
    }
    if hts_c_1247_hts_opt_apply(fp, fmt.specific.cast::<hts_opt>()) != 0 {
        hts_c_1279_hts_opt_free(fmt.specific.cast::<hts_opt>());
        return -1;
    }
    hts_c_1279_hts_opt_free(fmt.specific.cast::<hts_opt>());
    0
}

pub unsafe fn hts_c_1430_hts_crypt4gh_redirect(
    fn_: *const c_char,
    mode: *const c_char,
    hfile_ptr: *mut *mut hFILE,
    _fp: *mut htsFile,
) -> i32 {
    let hfile1 = *hfile_ptr;
    let mut mode2 = [0 as c_char; 102];
    let prefix = c"crypt4gh:";
    let fn2_len = prefix.to_bytes().len() + libc::strlen(fn_) + 1;
    let mut ret = -1;

    if fn2_len >= i32::MAX as usize {
        return -1;
    }

    let mut fn2 = vec![0 as c_char; fn2_len];
    libc::snprintf(
        fn2.as_mut_ptr(),
        fn2.len(),
        c"%s%s".as_ptr(),
        prefix.as_ptr(),
        fn_,
    );
    libc::snprintf(
        mode2.as_mut_ptr(),
        mode2.len(),
        c"%s%s".as_ptr(),
        mode,
        if libc::strchr(mode, b':' as i32).is_null() {
            c":".as_ptr()
        } else {
            c"".as_ptr()
        },
    );

    // Native equivalent of hopen(fn2, mode2, "parent", hfile1, NULL): pass the
    // parent hFILE as a typed option so the crypt4gh scheme handler's vopen
    // could read it. (No backend currently consumes "parent"; vopen handlers
    // ignore it.)
    let opts = [super::hfile::HFileOpt::Parent(hfile1)];
    let hfile2 = super::hfile::hfile_c_1317_hopen_vargs(fn2.as_mut_ptr().cast(), mode2.as_ptr().cast(), &opts);
    if !hfile2.is_null() {
        *hfile_ptr = hfile2;
        ret = 0;
    }

    ret
}

pub unsafe fn hts_parse_opt_list(opt: *mut htsFormat, str_: *const c_char) -> i32 {
    hts_c_1300_hts_parse_opt_list(opt, str_)
}

pub unsafe fn hts_set_threads(fp: *mut htsFile, n: i32) -> i32 {
    if (*fp).format.format == HTS_FORMAT_SAM {
        super::sam::sam_c_3746_sam_set_threads(fp, n)
    } else if (*fp).format.compression == HTS_COMPRESSION_BGZF {
        bgzf_mt(hts_get_bgzfp(fp), n, 256)
    } else if (*fp).format.format == HTS_FORMAT_CRAM {
        (*fp).fp.cram.as_mut().map_or(-1, |cram| {
            crate::cram_options_bridge::cram_set_option_int(cram, CRAM_OPT_NTHREADS, n)
        })
    } else {
        0
    }
}

pub unsafe fn hts_set_thread_pool(fp: *mut htsFile, p: *mut htsThreadPool) -> i32 {
    if (*fp).format.format == HTS_FORMAT_SAM || (*fp).format.format == HTS_FORMAT_TEXT_FORMAT {
        super::sam::sam_c_3719_sam_set_thread_pool(fp, p)
    } else if (*fp).format.compression == HTS_COMPRESSION_BGZF {
        bgzf_thread_pool(hts_get_bgzfp(fp), (*p).pool, (*p).qsize)
    } else if (*fp).format.format == HTS_FORMAT_CRAM {
        (*fp).fp.cram.as_mut().map_or(-1, |cram| {
            crate::cram_options_bridge::cram_set_option_ptr(
                cram,
                CRAM_OPT_THREAD_POOL,
                p.cast::<()>().as_ref(),
            )
        })
    } else {
        0
    }
}

pub unsafe fn hts_set_cache_size(fp: *mut htsFile, n: i32) {
    if (*fp).format.compression == HTS_COMPRESSION_BGZF {
        bgzf_set_cache_size(hts_get_bgzfp(fp), n);
    }
}

pub unsafe fn hts_set_fai_filename(fp: *mut htsFile, fn_aux: *const c_char) -> i32 {
    if !(*fp).fn_aux.is_null() {
        let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
        drop(Box::from_raw(std::slice::from_raw_parts_mut(
            (*fp).fn_aux.cast::<u8>(),
            len,
        )));
    }
    if !fn_aux.is_null() {
        let owned = CStr::from_ptr(fn_aux).to_bytes_with_nul().to_vec().into_boxed_slice();
        (*fp).fn_aux = Box::into_raw(owned).cast::<c_char>();
    } else {
        (*fp).fn_aux = std::ptr::null_mut();
    }

    if (*fp).format.format == HTS_FORMAT_CRAM
        && (*fp).fp.cram.as_mut().map_or(-1, |cram| {
            crate::cram_options_bridge::cram_set_option_ptr(
                cram,
                CRAM_OPT_REFERENCE,
                (*fp).fn_aux.cast::<()>().as_ref(),
            )
        }) != 0
    {
        return -1;
    }

    0
}

pub unsafe fn hts_set_filter_expression(fp: *mut htsFile, expr: *const c_char) -> i32 {
    if !(*fp).filter.is_null() {
        hts_filter_free((*fp).filter.cast());
    }

    if expr.is_null() {
        (*fp).filter = std::ptr::null_mut();
        return 0;
    }

    (*fp).filter = hts_filter_init(CStr::from_ptr(expr).to_bytes()).cast();
    if !(*fp).filter.is_null() {
        0
    } else {
        -1
    }
}

pub unsafe fn hts_c_1979_hts_open_tmpfile(
    fname: *const c_char,
    mode: *const c_char,
    tmpname: *mut kstring_t,
) -> *mut hFILE {
    let pid = libc::getpid() as i32;
    let ptr = tmpname as usize as u32;
    let mut n = 0;
    let mut fp: *mut hFILE;

    loop {
        let t = (libc::time(std::ptr::null_mut()) as u32) ^ (clock() as u32) ^ ptr;
        n += 1;

        ks_clear(&mut *tmpname);
        if kputs(CStr::from_ptr(fname).to_bytes(), &mut *tmpname) < 0 {
            break std::ptr::null_mut();
        }
        let suffix = format!(".tmp_{}_{}_{}", pid, n, t);
        if kputsn(suffix.as_bytes(), suffix.len(), &mut *tmpname) < 0 {
            break std::ptr::null_mut();
        }

        (*tmpname).data.push(0);
        fp = hopen((*tmpname).data.as_ptr().cast(), mode.cast());
        (*tmpname).data.pop();
        if !fp.is_null() || *libc::__errno_location() != libc::EEXIST as i32 || n >= 100 {
            break fp;
        }
    }
}

pub unsafe fn hts_check_EOF(fp: *mut htsFile) -> i32 {
    hts_c_2208_hts_check_EOF(fp)
}

pub unsafe fn hts_c_2208_hts_check_EOF(fp: *mut htsFile) -> i32 {
    if (*fp).format.compression == HTS_COMPRESSION_BGZF {
        bgzf_check_EOF(hts_get_bgzfp(fp))
    } else if (*fp).format.format == HTS_FORMAT_CRAM {
        cram_check_EOF((*fp).fp.cram)
    } else {
        3
    }
}

pub unsafe fn hts_c_2270_idx_format_name(fmt: i32) -> *mut c_char {
    match fmt {
        x if x == HTS_FMT_CSI => c"csi".as_ptr().cast_mut(),
        x if x == HTS_FMT_BAI => c"bai".as_ptr().cast_mut(),
        x if x == HTS_FMT_TBI as i32 => c"tbi".as_ptr().cast_mut(),
        x if x == HTS_FMT_CRAI => c"crai".as_ptr().cast_mut(),
        _ => c"unknown".as_ptr().cast_mut(),
    }
}

pub unsafe fn hts_c_2281_idx_dump(idx: *const hts_idx_t) {
    if idx.is_null() {
        eprintln!("Null index");
        return;
    }

    eprint!(
        "format='{}', min_shift={}, n_lvls={}, n_bins={}, l_meta={} ",
        String::from_utf8_lossy(CStr::from_ptr(hts_c_2270_idx_format_name((*idx).fmt)).to_bytes()),
        (*idx).min_shift,
        (*idx).n_lvls,
        (*idx).n_bins,
        (*idx).l_meta,
    );
    eprintln!(
        "n={}, m={}, n_no_coor={}",
        (*idx).n,
        (*idx).m,
        (*idx).n_no_coor,
    );

    for i in 0..(*idx).n {
        let bidx = if !(*idx).bidx.is_null() {
            *(*idx).bidx.add(i as usize)
        } else {
            std::ptr::null_mut()
        };
        let lidx = if !(*idx).lidx.is_null() {
            (*idx).lidx.add(i as usize)
        } else {
            std::ptr::null_mut()
        };
        if !bidx.is_null() {
            eprintln!(
                "======== BIN Index - tid={}, n_buckets={}, size={}",
                i,
                (*bidx).n_buckets,
                (*bidx).size,
            );
            for b in 0..meta_bin(idx) {
                let k = kh_get_bin(bidx, b);
                if k != (*bidx).n_buckets {
                    let entries = (*bidx).vals.add(k as usize);
                    let l = hts_bin_level(b as i32);
                    let bin_width = 1i64 << (((*idx).n_lvls - l) * 3 + (*idx).min_shift);
                    eprintln!(
                        "\tbin={}, level={}, parent={}, n_chunks={}, loff={}, interval=[{} - {}]",
                        b,
                        l,
                        hts_bin_parent(b as i32),
                        (*entries).n,
                        (*entries).loff,
                        (b as i32 - hts_bin_first(l)) as i64 * bin_width + 1,
                        (b as i32 + 1 - hts_bin_first(l)) as i64 * bin_width,
                    );
                    for j in 0..(*entries).n {
                        let chunk = (*entries).list.add(j as usize);
                        eprintln!(
                            "\t\tchunk={}, u={}, v={}",
                            j as i64,
                            (*chunk).u,
                            (*chunk).v,
                        );
                    }
                }
            }
        }
        if !lidx.is_null() {
            eprintln!(
                "======== LINEAR Index - tid={}, n_values={}",
                i,
                (*lidx).n as i64,
            );
            for j in 0..(*lidx).n {
                eprintln!(
                    "\t\tentry={}, offset={}, interval=[{} - {}]",
                    j as i64,
                    *(*lidx).offset.add(j as usize),
                    j * (1 << (*idx).min_shift) + 1,
                    (j + 1) * (1 << (*idx).min_shift),
                );
            }
        }
    }
}

pub unsafe fn hts_getline(fp: *mut htsFile, delimiter: i32, str: *mut kstring_t) -> i32 {
    if !(delimiter == KS_SEP_LINE || delimiter == b'\n' as i32) {
        eprintln!("[E::hts_getline] Unexpected delimiter {}", delimiter);
        std::process::abort();
    }

    let ret = match (*fp).format.compression {
        HTS_COMPRESSION_NO_COMPRESSION => {
            // SEAM: `str` may be a freshly calloc'd kstring (e.g. htsFile.line from
            // hts_hopen), whose Vec has a NULL data pointer. Vec::clear() routes
            // through slice::from_raw_parts_mut, which aborts on a NULL pointer even
            // at len 0. `truncate(0)` uses the raw (precondition-free) slice path and
            // is equivalent for our purposes (it empties the Vec without freeing),
            // so it is safe on a not-yet-allocated kstring and a no-op once empty.
            (*str).data.truncate(0);
            let mut ret = kgetline2(&mut *str, Some(hgetln_wrapper), (*fp).fp.hfile.cast::<()>());
            if ret >= 0 {
                ret = if (*str).data.len() <= i32::MAX as usize {
                    (*str).data.len() as i32
                } else {
                    i32::MAX
                };
            } else if htslib_hfile_h_134_herrno((*fp).fp.hfile) != 0 {
                ret = -2;
                *libc::__errno_location() = htslib_hfile_h_134_herrno((*fp).fp.hfile);
            } else {
                ret = -1;
            }
            ret
        }
        x if x == HTS_COMPRESSION_GZIP || x == HTS_COMPRESSION_BGZF => {
            bgzf_getline((*fp).fp.bgzf, b'\n' as i32, str)
        }
        _ => std::process::abort(),
    };
    (*fp).lineno += 1;
    ret
}

pub unsafe extern "C" fn hgetln_wrapper(
    buf: *mut u8,
    len: usize,
    vfp: *mut (),
) -> isize {
    htslib_hfile_h_195_hgetln(buf.cast(), len, vfp.cast())
}

pub unsafe fn hts_readlist(fn_: *const c_char, is_file: i32, n: *mut i32) -> *mut *mut c_char {
    hts_c_2065_hts_readlist(fn_, is_file, n)
}

pub unsafe fn hts_c_2065_hts_readlist(
    string: *const c_char,
    is_file: i32,
    n_out: *mut i32,
) -> *mut *mut c_char {
    // Owned array of owned, nul-terminated C strings.
    let mut s: Vec<*mut c_char> = Vec::new();

    if is_file != 0 {
        let fp = bgzf_open(string.cast(), c"r".as_ptr().cast());
        if fp.is_null() {
            return std::ptr::null_mut();
        }

        let mut str_: kstring_t = kstring_t::default();
        let mut ret;
        loop {
            ret = bgzf_getline(fp, b'\n' as i32, &mut str_);
            if ret < 0 {
                break;
            }
            if str_.data.is_empty() {
                continue;
            }
            if s.is_empty() && hts_is_utf16_text(&str_) != 0 {
                let name = if string.is_null() {
                    std::borrow::Cow::Borrowed("")
                } else {
                    std::ffi::CStr::from_ptr(string).to_string_lossy()
                };
                let msg = format!("'{name}' appears to be encoded as UTF-16");
                hts_log_cstr(HTS_LOG_WARNING, b"hts_readlist", msg.as_bytes());
            }
            let mut entry = str_.data.clone();
            entry.push(0);
            s.push(Box::into_raw(entry.into_boxed_slice()).cast::<c_char>());
        }
        if ret < -1 {
            for &p in &s {
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            bgzf_close(fp);
            return std::ptr::null_mut();
        }
        bgzf_close(fp);
    } else {
        let mut q = string;
        let mut p = string;
        loop {
            if *p == b',' as c_char || *p == 0 {
                let len = p.offset_from(q) as usize;
                let mut entry: Vec<u8> = vec![0u8; len + 1];
                std::ptr::copy_nonoverlapping(q.cast::<u8>(), entry.as_mut_ptr(), len);
                s.push(Box::into_raw(entry.into_boxed_slice()).cast::<c_char>());
                q = p.add(1);
            }
            if *p == 0 {
                break;
            }
            p = p.add(1);
        }
    }

    let n = s.len();
    *n_out = n as i32;
    // Shrink to exactly `n` and hand the raw array out.
    s.shrink_to_fit();
    let ptr = s.as_mut_ptr();
    std::mem::forget(s);
    ptr
}

pub unsafe fn hts_readlines(fn_: *const c_char, n: *mut i32) -> *mut *mut c_char {
    hts_c_2130_hts_readlines(fn_, n)
}

pub unsafe fn hts_c_2130_hts_readlines(fn_: *const c_char, n_out: *mut i32) -> *mut *mut c_char {
    // Owned array of owned, nul-terminated C strings.
    let mut s: Vec<*mut c_char> = Vec::new();
    let fp = bgzf_open(fn_.cast(), c"r".as_ptr().cast());

    if !fp.is_null() {
        let mut str_: kstring_t = kstring_t::default();
        let mut ret;
        loop {
            ret = bgzf_getline(fp, b'\n' as i32, &mut str_);
            if ret < 0 {
                break;
            }
            if str_.data.is_empty() {
                continue;
            }
            if s.is_empty() && hts_is_utf16_text(&str_) != 0 {
                let name = if fn_.is_null() {
                    std::borrow::Cow::Borrowed("")
                } else {
                    std::ffi::CStr::from_ptr(fn_).to_string_lossy()
                };
                let msg = format!("'{name}' appears to be encoded as UTF-16");
                hts_log_cstr(HTS_LOG_WARNING, b"hts_readlines", msg.as_bytes());
            }
            let mut entry = str_.data.clone();
            entry.push(0);
            s.push(Box::into_raw(entry.into_boxed_slice()).cast::<c_char>());
        }
        if ret < -1 {
            for &p in &s {
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            bgzf_close(fp);
            return std::ptr::null_mut();
        }
        bgzf_close(fp);
    } else if *fn_ == b':' as c_char {
        let mut q = fn_.add(1);
        let mut p = q;
        loop {
            if *p == b',' as c_char || *p == 0 {
                let len = p.offset_from(q) as usize;
                let mut entry: Vec<u8> = vec![0u8; len + 1];
                std::ptr::copy_nonoverlapping(q.cast::<u8>(), entry.as_mut_ptr(), len);
                s.push(Box::into_raw(entry.into_boxed_slice()).cast::<c_char>());
                q = p.add(1);
                if *p == 0 {
                    break;
                }
            }
            p = p.add(1);
        }
    } else {
        return std::ptr::null_mut();
    }

    let n = s.len();
    *n_out = n as i32;
    s.shrink_to_fit();
    let ptr = s.as_mut_ptr();
    std::mem::forget(s);
    ptr
}

pub unsafe fn hts_file_type(fname: *const c_char) -> i32 {
    hts_c_2186_hts_file_type(fname)
}

pub unsafe fn hts_c_2186_hts_file_type(fname: *const c_char) -> i32 {
    const FT_UNKN: i32 = 0;
    const FT_GZ: i32 = 1;
    const FT_VCF: i32 = 2;
    const FT_VCF_GZ: i32 = FT_GZ | FT_VCF;
    const FT_BCF: i32 = 1 << 2;
    const FT_BCF_GZ: i32 = FT_GZ | FT_BCF;
    const FT_STDIN: i32 = 1 << 3;

    let name = CStr::from_ptr(fname).to_bytes();
    if name.eq_ignore_ascii_case(b"-") {
        return FT_STDIN;
    }
    if name.len() >= 7 && name[name.len() - 7..].eq_ignore_ascii_case(b".vcf.gz") {
        return FT_VCF_GZ;
    }
    if name.len() >= 4 && name[name.len() - 4..].eq_ignore_ascii_case(b".vcf") {
        return FT_VCF;
    }
    if name.len() >= 4 && name[name.len() - 4..].eq_ignore_ascii_case(b".bcf") {
        return FT_BCF_GZ;
    }

    let f = hopen(fname.cast(), c"r".as_ptr().cast());
    if f.is_null() {
        return FT_UNKN;
    }

    let mut fmt: htsFormat = std::mem::zeroed();
    if hts_detect_format2(f, fname, &mut fmt) < 0 {
        hclose_abruptly(f);
        return FT_UNKN;
    }
    if hclose(f) < 0 {
        return FT_UNKN;
    }

    if fmt.format == HTS_FORMAT_VCF {
        if fmt.compression == HTS_COMPRESSION_NO_COMPRESSION {
            FT_VCF
        } else {
            FT_VCF_GZ
        }
    } else if fmt.format == HTS_FORMAT_BCF {
        if fmt.compression == HTS_COMPRESSION_NO_COMPRESSION {
            FT_BCF
        } else {
            FT_BCF_GZ
        }
    } else {
        FT_UNKN
    }
}

pub unsafe fn hts_reglist_create(
    argv: *mut *mut c_char,
    argc: i32,
    r_count: *mut i32,
    hdr: *mut (),
    getid: hts_name2id_f,
) -> *mut hts_reglist_t {
    if argv.is_null() || r_count.is_null() || argc <= 0 {
        return std::ptr::null_mut();
    }
    let argv_raw = std::slice::from_raw_parts(argv, argc as usize);
    if argv_raw.iter().any(|arg| arg.is_null()) {
        return std::ptr::null_mut();
    }
    let argv_refs: Vec<&[u8]> = argv_raw
        .iter()
        .copied()
        .map(|arg| CStr::from_ptr(arg).to_bytes())
        .collect();
    crate::htslib_rs::region::region_c_177_hts_reglist_create(
        Some(argv_refs.as_slice()),
        Some(&mut *r_count),
        hdr.as_mut(),
        getid,
    )
    .map_or(std::ptr::null_mut(), |reglist| {
        // Materialize the owned `region_reglist_entry` list into the `repr(C)`
        // `hts_reglist_t` array consumed by the iterator FFI: `reg` is strdup'd
        // and `intervals` is a malloc'd copy of the owned interval Vec.
        let raw: Vec<hts_reglist_t> = reglist
            .into_iter()
            .map(|entry| {
                let reg = entry.reg.map_or(std::ptr::null(), |name| {
                    let mut cstr: Vec<u8> = name;
                    cstr.push(0);
                    Box::into_raw(cstr.into_boxed_slice()).cast::<c_char>().cast_const()
                });
                let intervals = if entry.intervals.is_empty() {
                    std::ptr::null_mut()
                } else {
                    let copy: Vec<hts_pair_pos_t> = entry.intervals.clone();
                    Box::into_raw(copy.into_boxed_slice()).cast::<hts_pair_pos_t>()
                };
                hts_reglist_t {
                    reg,
                    intervals,
                    tid: entry.tid,
                    count: entry.intervals.len() as u32,
                    min_beg: entry.min_beg,
                    max_end: entry.max_end,
                }
            })
            .collect();
        *r_count = raw.len() as i32;
        Box::into_raw(raw.into_boxed_slice()).cast()
    })
}

pub unsafe fn hts_reglist_free(reglist: *mut hts_reglist_t, count: i32) {
    if reglist.is_null() || count <= 0 {
        return;
    }
    let reglist_slice = std::ptr::slice_from_raw_parts_mut(reglist, count as usize);
    let boxed = Box::from_raw(reglist_slice);
    for entry in boxed.iter() {
        if !entry.reg.is_null() {
            let len = CStr::from_ptr(entry.reg).to_bytes_with_nul().len();
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                entry.reg.cast_mut().cast::<u8>(),
                len,
            )));
        }
        if !entry.intervals.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                entry.intervals,
                entry.count as usize,
            )));
        }
    }
    drop(boxed);
}

pub unsafe fn hts_close(fp: *mut htsFile) -> i32 {
    if fp.is_null() {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }
    if !(*fp).filter.is_null() {
        hts_filter_free((*fp).filter.cast());
        (*fp).filter = std::ptr::null_mut();
    }
    match (*fp).format.format {
        HTS_FORMAT_BINARY_FORMAT | HTS_FORMAT_BAM | HTS_FORMAT_BCF => {
            let ret = bgzf_close((*fp).fp.bgzf.cast()) | hts_idx_close_otf_fp((*fp).idx);
            super::sam::sam_hdr_destroy((*fp).bam_header.cast());
            hts_idx_destroy((*fp).idx);
            if !(*fp).fn_.is_null() {
                let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_.cast::<u8>(),
                    len,
                )));
            }
            if !(*fp).fn_aux.is_null() {
                let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_aux.cast::<u8>(),
                    len,
                )));
            }
            ks_free(&mut (*fp).line);
            drop(Box::from_raw(fp));
            ret
        }
        HTS_FORMAT_EMPTY_FORMAT | HTS_FORMAT_TEXT_FORMAT | HTS_FORMAT_BED | HTS_FORMAT_VCF => {
            let ret = if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
                bgzf_close((*fp).fp.bgzf.cast())
            } else {
                hclose((*fp).fp.hfile)
            } | hts_idx_close_otf_fp((*fp).idx);
            super::sam::sam_hdr_destroy((*fp).bam_header.cast());
            hts_idx_destroy((*fp).idx);
            if !(*fp).fn_.is_null() {
                let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_.cast::<u8>(),
                    len,
                )));
            }
            if !(*fp).fn_aux.is_null() {
                let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_aux.cast::<u8>(),
                    len,
                )));
            }
            ks_free(&mut (*fp).line);
            drop(Box::from_raw(fp));
            ret
        }
        HTS_FORMAT_CRAM => {
            if ((*fp).bitfields & (1 << 1)) == 0 {
                let _ = crate::htslib_rs::cram::cram_eof((*fp).fp.cram);
            }
            let ret =
                crate::htslib_rs::cram::cram_close((*fp).fp.cram) | hts_idx_close_otf_fp((*fp).idx);
            super::sam::sam_hdr_destroy((*fp).bam_header.cast());
            hts_idx_destroy((*fp).idx);
            if !(*fp).fn_.is_null() {
                let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_.cast::<u8>(),
                    len,
                )));
            }
            if !(*fp).fn_aux.is_null() {
                let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_aux.cast::<u8>(),
                    len,
                )));
            }
            ks_free(&mut (*fp).line);
            drop(Box::from_raw(fp));
            ret
        }
        HTS_FORMAT_SAM | HTS_FORMAT_FASTA_FORMAT | HTS_FORMAT_FASTQ_FORMAT
            if (*fp).state.is_null() =>
        {
            let ret = if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
                bgzf_close((*fp).fp.bgzf.cast())
            } else {
                hclose((*fp).fp.hfile)
            } | hts_idx_close_otf_fp((*fp).idx);
            super::sam::sam_hdr_destroy((*fp).bam_header.cast());
            hts_idx_destroy((*fp).idx);
            if !(*fp).fn_.is_null() {
                let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_.cast::<u8>(),
                    len,
                )));
            }
            if !(*fp).fn_aux.is_null() {
                let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_aux.cast::<u8>(),
                    len,
                )));
            }
            ks_free(&mut (*fp).line);
            drop(Box::from_raw(fp));
            ret
        }
        HTS_FORMAT_SAM | HTS_FORMAT_FASTA_FORMAT | HTS_FORMAT_FASTQ_FORMAT => {
            if (*fp).format.format == HTS_FORMAT_FASTA_FORMAT
                || (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT
            {
                super::sam::sam_c_3802_fastq_state_destroy(fp);
            }
            let ret = if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
                bgzf_close((*fp).fp.bgzf.cast())
            } else {
                hclose((*fp).fp.hfile)
            } | hts_idx_close_otf_fp((*fp).idx);
            super::sam::sam_hdr_destroy((*fp).bam_header.cast());
            hts_idx_destroy((*fp).idx);
            if !(*fp).fn_.is_null() {
                let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_.cast::<u8>(),
                    len,
                )));
            }
            if !(*fp).fn_aux.is_null() {
                let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_aux.cast::<u8>(),
                    len,
                )));
            }
            ks_free(&mut (*fp).line);
            drop(Box::from_raw(fp));
            ret
        }
        _ => {
            let _ = hts_idx_close_otf_fp((*fp).idx);
            super::sam::sam_hdr_destroy((*fp).bam_header.cast());
            hts_idx_destroy((*fp).idx);
            if !(*fp).fn_.is_null() {
                let len = CStr::from_ptr((*fp).fn_).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_.cast::<u8>(),
                    len,
                )));
            }
            if !(*fp).fn_aux.is_null() {
                let len = CStr::from_ptr((*fp).fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*fp).fn_aux.cast::<u8>(),
                    len,
                )));
            }
            ks_free(&mut (*fp).line);
            drop(Box::from_raw(fp));
            -1
        }
    }
}

pub unsafe fn hts_flush(fp: *mut htsFile) -> i32 {
    if fp.is_null() {
        return 0;
    }

    match (*fp).format.format {
        HTS_FORMAT_BINARY_FORMAT | HTS_FORMAT_BAM | HTS_FORMAT_BCF => bgzf_flush((*fp).fp.bgzf),
        HTS_FORMAT_CRAM => 0,
        HTS_FORMAT_EMPTY_FORMAT
        | HTS_FORMAT_TEXT_FORMAT
        | HTS_FORMAT_BED
        | HTS_FORMAT_FASTA_FORMAT
        | HTS_FORMAT_FASTQ_FORMAT
        | HTS_FORMAT_SAM
        | HTS_FORMAT_VCF => {
            if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
                bgzf_flush((*fp).fp.bgzf)
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub unsafe fn hts_get_format(fp: *mut htsFile) -> *const htsFormat {
    if fp.is_null() {
        std::ptr::null()
    } else {
        std::ptr::addr_of!((*fp).format)
    }
}

pub unsafe fn hts_format_file_extension(format: *const htsFormat) -> *const c_char {
    if format.is_null() {
        return c"?".as_ptr();
    }

    match (*format).format {
        HTS_FORMAT_SAM => c"sam".as_ptr(),
        HTS_FORMAT_BAM => c"bam".as_ptr(),
        HTS_FORMAT_BAI => c"bai".as_ptr(),
        HTS_FORMAT_CRAM => c"cram".as_ptr(),
        HTS_FORMAT_CRAI_EXACT => c"crai".as_ptr(),
        HTS_FORMAT_VCF => c"vcf".as_ptr(),
        HTS_FORMAT_BCF => c"bcf".as_ptr(),
        HTS_FORMAT_CSI => c"csi".as_ptr(),
        HTS_FORMAT_FAI_FORMAT => c"fai".as_ptr(),
        HTS_FORMAT_FQI_FORMAT => c"fqi".as_ptr(),
        HTS_FORMAT_GZI => c"gzi".as_ptr(),
        HTS_FORMAT_TBI => c"tbi".as_ptr(),
        HTS_FORMAT_BED => c"bed".as_ptr(),
        HTS_FORMAT_D4_FORMAT => c"d4".as_ptr(),
        HTS_FORMAT_FASTA_FORMAT => c"fa".as_ptr(),
        HTS_FORMAT_FASTQ_FORMAT => c"fq".as_ptr(),
        _ => c"?".as_ptr(),
    }
}

pub unsafe fn hts_hfile(fp: *mut htsFile) -> *mut hFILE {
    match (*fp).format.format {
        HTS_FORMAT_BINARY_FORMAT | HTS_FORMAT_BCF | HTS_FORMAT_BAM => bgzf_hfile((*fp).fp.bgzf),
        HTS_FORMAT_CRAM => std::ptr::null_mut(),
        HTS_FORMAT_TEXT_FORMAT
        | HTS_FORMAT_VCF
        | HTS_FORMAT_FASTQ_FORMAT
        | HTS_FORMAT_FASTA_FORMAT
        | HTS_FORMAT_SAM => {
            if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
                bgzf_hfile((*fp).fp.bgzf)
            } else {
                (*fp).fp.hfile
            }
        }
        _ => std::ptr::null_mut(),
    }
}

pub unsafe fn hts_get_bgzfp(fp: *mut htsFile) -> *mut BGZF {
    if ((*fp).bitfields & (1 << 4)) != 0 {
        (*fp).fp.bgzf
    } else {
        std::ptr::null_mut()
    }
}

fn encoded_hfile_fd(fp: *mut hFILE) -> i32 {
    (fp as usize as isize - 1) as i32
}

pub unsafe fn hts_useek(fp: *mut htsFile, uoffset: i64, where_: i32) -> i32 {
    if ((*fp).bitfields & (1 << 4)) != 0 {
        bgzf_useek((*fp).fp.bgzf, uoffset, where_)
    } else if libc::lseek(
        encoded_hfile_fd((*fp).fp.hfile),
        uoffset as libc::off_t,
        libc::SEEK_SET,
    ) >= 0
    {
        0
    } else {
        -1
    }
}

pub unsafe fn hts_utell(fp: *mut htsFile) -> i64 {
    if ((*fp).bitfields & (1 << 4)) != 0 {
        bgzf_utell((*fp).fp.bgzf)
    } else {
        libc::lseek(encoded_hfile_fd((*fp).fp.hfile), 0, libc::SEEK_CUR) as i64
    }
}

pub unsafe fn hts_idx_destroy(idx: *mut hts_idx_t) {
    if idx.is_null() {
        return;
    }
    if (*idx).fmt == HTS_FMT_CRAI {
        // CRAI: idx is actually an `hts_cram_idx_t` (sam.c:1649). Free the
        // CRAI b-tree owned by the cram_fd, then the wrapper allocation
        // itself. Matches htslib/hts.c:2696.
        let cidx = idx.cast::<hts_cram_idx_t>();
        crate::htslib_rs::cram::cram_index_free(&mut *(*cidx).cram.cast::<cram_fd_layout>());
        drop(Box::from_raw(cidx));
        return;
    }
    let m = (*idx).m as usize;
    for i in 0..(*idx).m {
        let bidx = *(*idx).bidx.add(i as usize);
        let lidx = (*idx).lidx.add(i as usize);
        if !(*lidx).offset.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*lidx).offset,
                (*lidx).m as usize,
            )));
        }
        if bidx.is_null() {
            continue;
        }
        for k in 0..(*bidx).n_buckets {
            if kh_exist((*bidx).flags, k) {
                let bin = (*bidx).vals.add(k as usize);
                if !(*bin).list.is_null() {
                    drop(Box::from_raw(std::slice::from_raw_parts_mut(
                        (*bin).list,
                        (*bin).m as usize,
                    )));
                }
            }
        }
        if !(*bidx).flags.is_null() {
            let n_flags = if (*bidx).n_buckets < 16 { 1 } else { (*bidx).n_buckets >> 4 };
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*bidx).flags,
                n_flags as usize,
            )));
        }
        if !(*bidx).keys.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*bidx).keys,
                (*bidx).n_buckets as usize,
            )));
        }
        if !(*bidx).vals.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*bidx).vals,
                (*bidx).n_buckets as usize,
            )));
        }
        drop(Box::from_raw(bidx));
    }
    if !(*idx).bidx.is_null() {
        drop(Box::from_raw(std::slice::from_raw_parts_mut((*idx).bidx, m)));
    }
    if !(*idx).lidx.is_null() {
        drop(Box::from_raw(std::slice::from_raw_parts_mut((*idx).lidx, m)));
    }
    if !(*idx).meta.is_null() {
        drop(Box::from_raw(std::slice::from_raw_parts_mut(
            (*idx).meta,
            (*idx).l_meta as usize,
        )));
    }
    drop(Box::from_raw(idx));
}

pub unsafe fn hts_idx_init(
    n: i32,
    fmt: i32,
    offset0: u64,
    min_shift: i32,
    n_lvls: i32,
) -> *mut hts_idx_t {
    hts_c_2405_hts_idx_init(n, fmt, offset0, min_shift, n_lvls)
}

pub unsafe fn hts_idx_push(
    idx: *mut hts_idx_t,
    tid: i32,
    beg: hts_pos_t,
    end: hts_pos_t,
    offset: u64,
    is_mapped: i32,
) -> i32 {
    hts_c_2558_hts_idx_push(idx, tid, beg, end, offset, is_mapped)
}

pub unsafe fn hts_c_2558_hts_idx_push(
    idx: *mut hts_idx_t,
    tid: i32,
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    offset: u64,
    is_mapped: i32,
) -> i32 {
    if tid < 0 {
        beg = -1;
        end = 0;
    }
    if hts_c_2538_hts_idx_check_range(idx, tid, beg, end) < 0 {
        return -1;
    }
    if tid >= (*idx).m {
        let new_m = if (*idx).m * 2 > tid + 1 {
            (*idx).m * 2
        } else {
            tid + 1
        };
        let old_m = (*idx).m as usize;
        // Grow the bidx pointer array, zero-filling new slots.
        let mut bidx_vec: Vec<*mut hts_idx_bidx_t> = if (*idx).bidx.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*idx).bidx, old_m, old_m)
        };
        bidx_vec.resize(new_m as usize, std::ptr::null_mut());
        (*idx).bidx = Box::into_raw(bidx_vec.into_boxed_slice()).cast::<*mut hts_idx_bidx_t>();
        // Grow the lidx array, zero-filling new slots.
        let mut lidx_vec: Vec<hts_idx_lidx_t> = if (*idx).lidx.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*idx).lidx, old_m, old_m)
        };
        while lidx_vec.len() < new_m as usize {
            lidx_vec.push(std::mem::zeroed());
        }
        (*idx).lidx = Box::into_raw(lidx_vec.into_boxed_slice()).cast::<hts_idx_lidx_t>();
        (*idx).m = new_m;
    }
    if (*idx).n < tid + 1 {
        (*idx).n = tid + 1;
    }
    if (*idx).z.finished != 0 {
        return 0;
    }
    if (*idx).z.last_tid != tid || ((*idx).z.last_tid >= 0 && tid < 0) {
        if tid >= 0 && (*idx).n_no_coor != 0 {
            return -1;
        }
        if tid >= 0 && !(*(*idx).bidx.add(tid as usize)).is_null() {
            return -1;
        }
        (*idx).z.last_tid = tid;
        (*idx).z.last_bin = 0xffff_ffffu32;
    } else if tid >= 0 && (*idx).z.last_coor > beg {
        return -1;
    }
    if end < beg {
        return -1;
    }
    if tid >= 0 {
        if (*(*idx).bidx.add(tid as usize)).is_null() {
            let Some(bidx) = alloc_bidx(0) else {
                return -1;
            };
            *(*idx).bidx.add(tid as usize) = bidx;
        }
        if beg < 0 {
            beg = 0;
        }
        if end <= 0 {
            end = 1;
        }
        if hts_c_2347_insert_to_l(
            (*idx).lidx.add(tid as usize),
            beg,
            end,
            (*idx).z.last_off,
            (*idx).min_shift,
        ) < 0
        {
            return -1;
        }
    } else {
        (*idx).n_no_coor += 1;
    }
    let bin = hts_reg2bin(beg, end, (*idx).min_shift, (*idx).n_lvls);
    if (*idx).z.last_bin as i32 != bin {
        if (*idx).z.save_bin != 0xffff_ffff
            && hts_c_2320_insert_to_b(
                *(*idx).bidx.add((*idx).z.save_tid as usize),
                (*idx).z.save_bin as i32,
                (*idx).z.save_off,
                (*idx).z.last_off,
            ) < 0
        {
            return -1;
        }
        if (*idx).z.last_bin == 0xffff_ffffu32 && (*idx).z.save_bin != 0xffff_ffffu32 {
            (*idx).z.off_end = (*idx).z.last_off;
            if hts_c_2320_insert_to_b(
                *(*idx).bidx.add((*idx).z.save_tid as usize),
                meta_bin(idx) as i32,
                (*idx).z.off_beg,
                (*idx).z.off_end,
            ) < 0
            {
                return -1;
            }
            if hts_c_2320_insert_to_b(
                *(*idx).bidx.add((*idx).z.save_tid as usize),
                meta_bin(idx) as i32,
                (*idx).z.n_mapped,
                (*idx).z.n_unmapped,
            ) < 0
            {
                return -1;
            }
            (*idx).z.n_mapped = 0;
            (*idx).z.n_unmapped = 0;
            (*idx).z.off_beg = (*idx).z.off_end;
        }
        (*idx).z.save_off = (*idx).z.last_off;
        (*idx).z.save_bin = bin as u32;
        (*idx).z.last_bin = bin as u32;
        (*idx).z.save_tid = tid;
    }
    if is_mapped != 0 {
        (*idx).z.n_mapped += 1;
    } else {
        (*idx).z.n_unmapped += 1;
    }
    (*idx).z.last_off = offset;
    (*idx).z.last_coor = beg;
    0
}

pub unsafe fn hts_c_2372_hts_adjust_csi_settings(
    max_len_in: i64,
    min_shift_: *mut i32,
    n_lvls_: *mut i32,
) {
    const MAX_N_LVLS: i32 = 9;
    let mut min_shift = *min_shift_;
    let mut n_lvls = *n_lvls_;
    let max_len = max_len_in + 256;

    if max_len <= hts_bin_maxpos(min_shift, MAX_N_LVLS) {
        let mut maxpos = hts_bin_maxpos(min_shift, n_lvls);
        while max_len > maxpos {
            n_lvls += 1;
            maxpos *= 8;
        }
        *n_lvls_ = n_lvls;
    } else {
        n_lvls = MAX_N_LVLS;
        let mut maxpos = hts_bin_maxpos(min_shift, n_lvls);
        while max_len > maxpos {
            min_shift += 1;
            maxpos *= 2;
        }
        *n_lvls_ = n_lvls;
        *min_shift_ = min_shift;
    }
}

pub unsafe fn hts_c_2320_insert_to_b(
    b: *mut hts_idx_bidx_t,
    bin: i32,
    beg: u64,
    end: u64,
) -> i32 {
    let bin = bin as u32;
    let mut absent = 1;
    let mut k = kh_get_bin(b, bin);
    if k != (*b).n_buckets {
        absent = 0;
    } else {
        let Some(new_k) = insert_bidx_bin(b, bin) else {
            return -1;
        };
        k = new_k;
    }

    let l = (*b).vals.add(k as usize);
    if absent != 0 {
        (*l).m = 1;
        (*l).n = 0;
        let list: Vec<hts_pair64_t> = (0..(*l).m).map(|_| std::mem::zeroed()).collect();
        (*l).list = Box::into_raw(list.into_boxed_slice()).cast::<hts_pair64_t>();
    } else if (*l).n == (*l).m {
        let new_m = if (*l).m != 0 { (*l).m << 1 } else { 1 };
        let old_m = (*l).m as usize;
        let mut list: Vec<hts_pair64_t> = if (*l).list.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*l).list, old_m, old_m)
        };
        while list.len() < new_m as usize {
            list.push(std::mem::zeroed());
        }
        (*l).list = Box::into_raw(list.into_boxed_slice()).cast::<hts_pair64_t>();
        (*l).m = new_m;
    }
    (*(*l).list.add((*l).n as usize)).u = beg;
    (*(*l).list.add((*l).n as usize)).v = end;
    (*l).n += 1;
    0
}

pub unsafe fn hts_c_2347_insert_to_l(
    l: *mut hts_idx_lidx_t,
    _beg: i64,
    _end: i64,
    offset: u64,
    min_shift: i32,
) -> i32 {
    let beg = _beg >> min_shift;
    let end = (_end - 1) >> min_shift;
    if (*l).m < end + 1 {
        let new_m = if (*l).m * 2 > end + 1 {
            (*l).m * 2
        } else {
            end + 1
        };
        let old_m = (*l).m as usize;
        let mut offset: Vec<u64> = if (*l).offset.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*l).offset, old_m, old_m)
        };
        // New slots initialised to all-ones (u64::MAX), matching memset(0xff).
        offset.resize(new_m as usize, u64::MAX);
        (*l).m = new_m;
        (*l).offset = Box::into_raw(offset.into_boxed_slice()).cast::<u64>();
    }
    let mut i = beg;
    while i <= end {
        if *(*l).offset.add(i as usize) == u64::MAX {
            *(*l).offset.add(i as usize) = offset;
        }
        i += 1;
    }
    if (*l).n < end + 1 {
        (*l).n = end + 1;
    }
    0
}

pub unsafe fn hts_c_2405_hts_idx_init(
    n: i32,
    fmt: i32,
    offset0: u64,
    min_shift: i32,
    n_lvls: i32,
) -> *mut hts_idx_t {
    let idx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_t>()));
    (*idx).fmt = fmt;
    (*idx).min_shift = min_shift;
    (*idx).n_lvls = n_lvls;
    (*idx).n_bins = ((1 << (3 * n_lvls + 3)) - 1) / 7;
    (*idx).z.save_tid = -1;
    (*idx).z.last_tid = -1;
    (*idx).z.save_bin = 0xffff_ffffu32;
    (*idx).z.last_bin = 0xffff_ffffu32;
    (*idx).z.save_off = offset0;
    (*idx).z.last_off = offset0;
    (*idx).z.off_beg = offset0;
    (*idx).z.off_end = offset0;
    (*idx).z.last_coor = 0xffff_ffffu32 as hts_pos_t;
    if n != 0 {
        (*idx).n = n;
        (*idx).m = n;
        let bidx: Vec<*mut hts_idx_bidx_t> = vec![std::ptr::null_mut(); n as usize];
        (*idx).bidx = Box::into_raw(bidx.into_boxed_slice()).cast::<*mut hts_idx_bidx_t>();
        let lidx: Vec<hts_idx_lidx_t> = (0..n).map(|_| std::mem::zeroed()).collect();
        (*idx).lidx = Box::into_raw(lidx.into_boxed_slice()).cast::<hts_idx_lidx_t>();
    }
    (*idx).tbi_n = -1;
    (*idx).last_tbi_tid = -1;
    (*idx).otf_fp = std::ptr::null_mut();
    idx
}

pub unsafe fn hts_c_2431_update_loff(idx: *mut hts_idx_t, i: i32, free_lidx: i32) {
    let bidx = *(*idx).bidx.add(i as usize);
    let lidx = (*idx).lidx.add(i as usize);

    if (*lidx).n >= 2 {
        let mut l = (*lidx).n - 2;
        loop {
            if *(*lidx).offset.add(l as usize) == u64::MAX {
                *(*lidx).offset.add(l as usize) = *(*lidx).offset.add((l + 1) as usize);
            }
            if l == 0 {
                break;
            }
            l -= 1;
        }
    }
    if bidx.is_null() {
        return;
    }
    let mut k = 0;
    while k < (*bidx).n_buckets {
        if kh_exist((*bidx).flags, k) {
            let key = *(*bidx).keys.add(k as usize);
            let val = (*bidx).vals.add(k as usize);
            if key < (*idx).n_bins as u32 {
                let bot_bin = hts_bin_bot(key as i32, (*idx).n_lvls);
                (*val).loff = if (bot_bin as hts_pos_t) < (*lidx).n {
                    *(*lidx).offset.add(bot_bin as usize)
                } else {
                    0
                };
            } else {
                (*val).loff = 0;
            }
        }
        k += 1;
    }
    if free_lidx != 0 {
        if !(*lidx).offset.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*lidx).offset,
                (*lidx).m as usize,
            )));
        }
        (*lidx).m = 0;
        (*lidx).n = 0;
        (*lidx).offset = std::ptr::null_mut();
    }
}

pub unsafe fn hts_c_2462_compress_binning(idx: *mut hts_idx_t, i: i32) -> i32 {
    const HTS_MIN_MARKER_DIST: u64 = 0x10000;
    let bidx = *(*idx).bidx.add(i as usize);
    if bidx.is_null() {
        return 0;
    }

    let mut l = (*idx).n_lvls;
    while l > 0 {
        let start = hts_bin_first(l) as u32;
        let mut k = 0;
        while k < (*bidx).n_buckets {
            if kh_exist((*bidx).flags, k) {
                let key = *(*bidx).keys.add(k as usize);
                if key < (*idx).n_bins as u32 && key >= start {
                    let p = (*bidx).vals.add(k as usize);
                    if l < (*idx).n_lvls && (*p).n > 1 {
                        let list = std::slice::from_raw_parts_mut((*p).list, (*p).n as usize);
                        list.sort_by(|a, b| a.u.cmp(&b.u));
                    }
                    if (*p).n > 0
                        && ((*(*p).list.add(((*p).n - 1) as usize)).v >> 16)
                            .wrapping_sub((*(*p).list).u >> 16)
                            < HTS_MIN_MARKER_DIST
                    {
                        let kp = kh_get_bin(bidx, hts_bin_parent(key as i32) as u32);
                        if kp != (*bidx).n_buckets {
                            let q = (*bidx).vals.add(kp as usize);
                            if (*q).n + (*p).n > (*q).m {
                                let mut new_m = ((*q).n + (*p).n) as u32;
                                new_m = new_m.wrapping_sub(1);
                                new_m |= new_m >> 1;
                                new_m |= new_m >> 2;
                                new_m |= new_m >> 4;
                                new_m |= new_m >> 8;
                                new_m |= new_m >> 16;
                                new_m = new_m.wrapping_add(1);
                                if new_m > i32::MAX as u32 {
                                    return -1;
                                }
                                let old_m = (*q).m as usize;
                                let mut qlist: Vec<hts_pair64_t> = if (*q).list.is_null() {
                                    Vec::new()
                                } else {
                                    Vec::from_raw_parts((*q).list, old_m, old_m)
                                };
                                while qlist.len() < new_m as usize {
                                    qlist.push(std::mem::zeroed());
                                }
                                (*q).m = new_m as i32;
                                (*q).list = Box::into_raw(qlist.into_boxed_slice())
                                    .cast::<hts_pair64_t>();
                            }
                            std::ptr::copy_nonoverlapping(
                                (*p).list,
                                (*q).list.add((*q).n as usize),
                                (*p).n as usize,
                            );
                            (*q).n += (*p).n;
                            if !(*p).list.is_null() {
                                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                                    (*p).list,
                                    (*p).m as usize,
                                )));
                            }
                            (*p).list = std::ptr::null_mut();
                            (*p).n = 0;
                            (*p).m = 0;
                            kh_del_bin(bidx, k);
                        }
                    }
                }
            }
            k += 1;
        }
        l -= 1;
    }

    let k0 = kh_get_bin(bidx, 0);
    if k0 != (*bidx).n_buckets {
        let p = (*bidx).vals.add(k0 as usize);
        if (*p).n > 1 {
            let list = std::slice::from_raw_parts_mut((*p).list, (*p).n as usize);
            list.sort_by(|a, b| a.u.cmp(&b.u));
        }
    }

    let mut k = 0;
    while k < (*bidx).n_buckets {
        if kh_exist((*bidx).flags, k) {
            let key = *(*bidx).keys.add(k as usize);
            if key < (*idx).n_bins as u32 {
                let p = (*bidx).vals.add(k as usize);
                if (*p).n > 0 {
                    let mut l = 1;
                    let mut m = 0;
                    while l < (*p).n {
                        let pm = *(*p).list.add(m as usize);
                        let pl = *(*p).list.add(l as usize);
                        if pm.v >> 16 >= pl.u >> 16 {
                            if (*(*p).list.add(m as usize)).v < pl.v {
                                (*(*p).list.add(m as usize)).v = pl.v;
                            }
                        } else {
                            m += 1;
                            *(*p).list.add(m as usize) = pl;
                        }
                        l += 1;
                    }
                    (*p).n = m + 1;
                }
            }
        }
        k += 1;
    }
    0
}

pub unsafe fn hts_c_2515_hts_idx_finish(idx: *mut hts_idx_t, final_offset: u64) -> i32 {
    let mut ret = 0;
    if idx.is_null() || (*idx).z.finished != 0 {
        return 0;
    }
    if (*idx).z.save_tid >= 0 {
        ret |= hts_c_2320_insert_to_b(
            *(*idx).bidx.add((*idx).z.save_tid as usize),
            (*idx).z.save_bin as i32,
            (*idx).z.save_off,
            final_offset,
        );
        ret |= hts_c_2320_insert_to_b(
            *(*idx).bidx.add((*idx).z.save_tid as usize),
            meta_bin(idx) as i32,
            (*idx).z.off_beg,
            final_offset,
        );
        ret |= hts_c_2320_insert_to_b(
            *(*idx).bidx.add((*idx).z.save_tid as usize),
            meta_bin(idx) as i32,
            (*idx).z.n_mapped,
            (*idx).z.n_unmapped,
        );
    }
    for i in 0..(*idx).n {
        hts_c_2431_update_loff(idx, i, ((*idx).fmt == HTS_FMT_CSI) as i32);
        ret |= hts_c_2462_compress_binning(idx, i);
    }
    (*idx).z.finished = 1;
    ret
}

pub unsafe fn hts_idx_finish(idx: *mut hts_idx_t, final_offset: u64) -> i32 {
    hts_c_2515_hts_idx_finish(idx, final_offset)
}

pub unsafe fn hts_c_2533_hts_idx_maxpos(idx: *const hts_idx_t) -> hts_pos_t {
    hts_bin_maxpos((*idx).min_shift, (*idx).n_lvls)
}

pub unsafe fn hts_c_2538_hts_idx_check_range(
    idx: *mut hts_idx_t,
    tid: i32,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> i32 {
    let maxpos = hts_c_2533_hts_idx_maxpos(idx);
    if tid < 0 || (beg <= maxpos && end <= maxpos) {
        return 0;
    }
    *libc::__errno_location() = libc::ERANGE;
    -1
}

pub unsafe fn hts_c_2682_hts_idx_amend_last(idx: *mut hts_idx_t, offset: u64) {
    (*idx).z.last_off = offset;
}

pub unsafe fn hts_c_2648_hts_idx_tbi_name(
    idx: *mut hts_idx_t,
    tid: i32,
    name: *const c_char,
) -> i32 {
    if tid == (*idx).last_tbi_tid || tid < 0 || name.is_null() {
        return (*idx).tbi_n;
    }

    let len = CStr::from_ptr(name).to_bytes_with_nul().len() as u32;
    let old_l = (*idx).l_meta as usize;
    let mut meta_vec: Vec<u8> = if (*idx).meta.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*idx).meta, old_l, old_l)
    };
    meta_vec.resize(old_l + len as usize, 0);
    (*idx).meta = Box::into_raw(meta_vec.into_boxed_slice()).cast::<u8>();
    std::ptr::copy_nonoverlapping(name.cast::<u8>(), (*idx).meta.add(old_l), len as usize);
    (*idx).l_meta += len;
    u32_to_le(le_to_u32((*idx).meta.add(24)) + len, (*idx).meta.add(24));

    (*idx).last_tbi_tid = tid;
    (*idx).tbi_n += 1;
    (*idx).tbi_n
}

pub unsafe fn hts_c_2714_hts_idx_fmt(idx: *mut hts_idx_t) -> i32 {
    (*idx).fmt
}

pub unsafe fn hts_idx_fmt(idx: *mut hts_idx_t) -> i32 {
    hts_c_2714_hts_idx_fmt(idx)
}

pub unsafe fn hts_idx_nseq(idx: *const hts_idx_t) -> i32 {
    if idx.is_null() {
        -1
    } else {
        (*idx).n
    }
}

pub unsafe fn hts_c_3110_hts_idx_nseq(idx: *const hts_idx_t) -> i32 {
    if idx.is_null() {
        -1
    } else {
        (*idx).n
    }
}

pub unsafe fn hts_idx_tbi_name(idx: *mut hts_idx_t, tid: i32, name: *const c_char) -> i32 {
    hts_c_2648_hts_idx_tbi_name(idx, tid, name)
}

pub unsafe fn hts_c_2721_idx_write_int32(fp: *mut BGZF, mut x: i32) -> isize {
    if ed_is_big() != 0 {
        x = ed_swap_4(x as u32) as i32;
    }
    bgzf_write(fp, (&x as *const i32).cast(), std::mem::size_of_val(&x))
}

pub unsafe fn hts_c_2727_idx_write_uint32(fp: *mut BGZF, mut x: u32) -> isize {
    if ed_is_big() != 0 {
        x = ed_swap_4(x);
    }
    bgzf_write(fp, (&x as *const u32).cast(), std::mem::size_of_val(&x))
}

pub unsafe fn hts_c_2733_idx_write_uint64(fp: *mut BGZF, mut x: u64) -> isize {
    if ed_is_big() != 0 {
        x = ed_swap_8(x);
    }
    bgzf_write(fp, (&x as *const u64).cast(), std::mem::size_of_val(&x))
}

pub unsafe fn hts_c_2739_swap_bins(p: *mut hts_idx_bins_t) {
    let mut i = 0;
    while i < (*p).n {
        ed_swap_8p((&mut (*(*p).list.add(i as usize)).u as *mut u64).cast());
        ed_swap_8p((&mut (*(*p).list.add(i as usize)).v as *mut u64).cast());
        i += 1;
    }
}

pub unsafe fn hts_c_2748_need_idx_ugly_delay_hack(idx: *const hts_idx_t) -> i32 {
    (!(*idx).otf_fp.is_null() && !bgzf_is_compressed((*idx).otf_fp)) as i32
}

pub unsafe fn hts_c_2759_idx_save_core(idx: *const hts_idx_t, fp: *mut BGZF, fmt: i32) -> i32 {
    const TBX_VCF: u32 = 2;

    let mut nids = (*idx).n;
    if !(*idx).meta.is_null() && (*idx).l_meta >= 4 && le_to_u32((*idx).meta) == TBX_VCF {
        nids = 0;
        for i in 0..(*idx).n {
            if !(*(*idx).bidx.add(i as usize)).is_null() {
                nids += 1;
            }
        }
    }
    if hts_c_2721_idx_write_int32(fp, nids) < 0 {
        return -1;
    }
    if fmt == HTS_FMT_TBI as i32
        && (*idx).l_meta != 0
        && bgzf_write(fp, (*idx).meta.cast(), (*idx).l_meta as usize) < 0
    {
        return -1;
    }

    for i in 0..(*idx).n {
        let bidx = *(*idx).bidx.add(i as usize);
        let lidx = (*idx).lidx.add(i as usize);

        if (nids == (*idx).n || !bidx.is_null())
            && hts_c_2721_idx_write_int32(
                fp,
                if !bidx.is_null() {
                    (*bidx).size as i32
                } else {
                    0
                },
            ) < 0
        {
            return -1;
        }
        if !bidx.is_null() {
            for k in 0..(*bidx).n_buckets {
                if kh_exist((*bidx).flags, k) {
                    let p = (*bidx).vals.add(k as usize);
                    if hts_c_2727_idx_write_uint32(fp, *(*bidx).keys.add(k as usize)) < 0 {
                        return -1;
                    }
                    if fmt == HTS_FMT_CSI && hts_c_2733_idx_write_uint64(fp, (*p).loff) < 0 {
                        return -1;
                    }
                    if hts_c_2721_idx_write_int32(fp, (*p).n) < 0 {
                        return -1;
                    }
                    for j in 0..(*p).n {
                        if hts_c_2733_idx_write_uint64(fp, (*(*p).list.add(j as usize)).u) < 0 {
                            return -1;
                        }
                        if hts_c_2733_idx_write_uint64(fp, (*(*p).list.add(j as usize)).v) < 0 {
                            return -1;
                        }
                    }
                }
            }
        }

        if fmt != HTS_FMT_CSI {
            if hts_c_2721_idx_write_int32(fp, (*lidx).n as i32) < 0 {
                return -1;
            }
            for j in 0..(*lidx).n {
                if hts_c_2733_idx_write_uint64(fp, *(*lidx).offset.add(j as usize)) < 0 {
                    return -1;
                }
            }
        }
    }

    if hts_c_2748_need_idx_ugly_delay_hack(idx) == 0
        && hts_c_2733_idx_write_uint64(fp, (*idx).n_no_coor) < 0
    {
        return -1;
    }

    0
}

pub unsafe fn hts_c_2847_hts_idx_write_out(
    idx: *const hts_idx_t,
    fp: *mut BGZF,
    fmt: i32,
) -> i32 {
    if fmt == HTS_FMT_CSI {
        if bgzf_write(fp, b"CSI\x01".as_ptr().cast(), 4) < 0 {
            return -1;
        }
        if hts_c_2721_idx_write_int32(fp, (*idx).min_shift) < 0 {
            return -1;
        }
        if hts_c_2721_idx_write_int32(fp, (*idx).n_lvls) < 0 {
            return -1;
        }
        if hts_c_2727_idx_write_uint32(fp, (*idx).l_meta) < 0 {
            return -1;
        }
        if (*idx).l_meta != 0 && bgzf_write(fp, (*idx).meta.cast(), (*idx).l_meta as usize) < 0 {
            return -1;
        }
    } else if fmt == HTS_FMT_TBI as i32 {
        if bgzf_write(fp, b"TBI\x01".as_ptr().cast(), 4) < 0 {
            return -1;
        }
    } else if fmt == HTS_FMT_BAI {
        if bgzf_write(fp, b"BAI\x01".as_ptr().cast(), 4) < 0 {
            return -1;
        }
    } else {
        std::process::abort();
    }

    if hts_c_2759_idx_save_core(idx, fp, fmt) < 0 {
        return -1;
    }
    0
}

pub unsafe fn hts_idx_save(idx: *const hts_idx_t, fn_: *const c_char, fmt: i32) -> i32 {
    hts_c_2825_hts_idx_save(idx, fn_, fmt)
}

pub unsafe fn hts_c_2825_hts_idx_save(
    idx: *const hts_idx_t,
    fn_: *const c_char,
    fmt: i32,
) -> i32 {
    if idx.is_null() || fn_.is_null() {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    }
    let ext = if fmt == HTS_FMT_BAI {
        c".bai".as_ptr()
    } else if fmt == HTS_FMT_CSI {
        c".csi".as_ptr()
    } else if fmt == HTS_FMT_TBI as i32 {
        c".tbi".as_ptr()
    } else {
        std::process::abort();
    };
    // Build "<fn_><ext>" as an owned nul-terminated byte buffer.
    let mut fnidx_buf: Vec<u8> = CStr::from_ptr(fn_).to_bytes().to_vec();
    fnidx_buf.extend_from_slice(CStr::from_ptr(ext).to_bytes());
    fnidx_buf.push(0);
    let ret = hts_c_2869_hts_idx_save_as(idx, fn_, fnidx_buf.as_ptr().cast::<c_char>(), fmt);
    let save = *libc::__errno_location();
    drop(fnidx_buf);
    *libc::__errno_location() = save;
    ret
}

pub unsafe fn hts_idx_save_as(
    idx: *const hts_idx_t,
    fn_: *const c_char,
    fnidx: *const c_char,
    fmt: i32,
) -> i32 {
    hts_c_2869_hts_idx_save_as(idx, fn_, fnidx, fmt)
}

pub unsafe fn hts_c_2869_hts_idx_save_as(
    idx: *const hts_idx_t,
    fn_: *const c_char,
    fnidx: *const c_char,
    fmt: i32,
) -> i32 {
    if fnidx.is_null() {
        return hts_c_2825_hts_idx_save(idx, fn_, fmt);
    }
    let mode = if fmt == HTS_FMT_BAI {
        c"wu".as_ptr()
    } else {
        c"w".as_ptr()
    };
    let fp = bgzf_open(fnidx.cast(), mode.cast());
    if fp.is_null() {
        return -1;
    }
    if hts_c_2847_hts_idx_write_out(idx, fp, fmt) < 0 {
        let save_errno = *libc::__errno_location();
        bgzf_close(fp);
        *libc::__errno_location() = save_errno;
        return -1;
    }
    bgzf_close(fp)
}

pub unsafe fn hts_c_2894_hts_idx_save_but_not_close(
    idx: *mut hts_idx_t,
    fnidx: *const c_char,
    fmt: i32,
) -> i32 {
    let mode = if fmt == HTS_FMT_BAI {
        c"wu".as_ptr()
    } else {
        c"w".as_ptr()
    };
    (*idx).otf_fp = bgzf_open(fnidx.cast(), mode.cast());
    if (*idx).otf_fp.is_null() {
        return -1;
    }
    if hts_c_2847_hts_idx_write_out(idx, (*idx).otf_fp, fmt) < 0 {
        let save_errno = *libc::__errno_location();
        bgzf_close((*idx).otf_fp);
        (*idx).otf_fp = std::ptr::null_mut();
        *libc::__errno_location() = save_errno;
        return -1;
    }
    bgzf_flush((*idx).otf_fp)
}

pub unsafe fn hts_c_2925_idx_read_core(idx: *mut hts_idx_t, fp: *mut BGZF, fmt: i32) -> i32 {
    if idx.is_null() {
        return -4;
    }
    for i in 0..(*idx).n {
        let l = (*idx).lidx.add(i as usize);
        let Some(n) = bgzf_read_u32(fp) else {
            return -1;
        };
        if n > i32::MAX as u32 {
            return -3;
        }
        let Some(h) = alloc_bidx(n) else {
            return -2;
        };
        *(*idx).bidx.add(i as usize) = h;
        for _ in 0..n {
            let Some(key) = bgzf_read_u32(fp) else {
                return -1;
            };
            let Some(k) = insert_bidx_bin(h, key) else {
                return -3;
            };
            let p = (*h).vals.add(k as usize);
            if fmt == HTS_FMT_CSI {
                let Some(loff) = bgzf_read_u64(fp) else {
                    return -1;
                };
                (*p).loff = loff;
            } else {
                (*p).loff = 0;
            }
            let Some(n_chunk) = bgzf_read_u32(fp) else {
                return -1;
            };
            if n_chunk > i32::MAX as u32 {
                return -3;
            }
            (*p).n = n_chunk as i32;
            (*p).m = n_chunk as i32;
            let bytes = n_chunk as usize * std::mem::size_of::<hts_pair64_t>();
            let mut list: Vec<hts_pair64_t> =
                (0..n_chunk).map(|_| std::mem::zeroed()).collect();
            let list_ptr = list.as_mut_ptr();
            std::mem::forget(list);
            (*p).list = list_ptr;
            if !bgzf_read_exact(fp, (*p).list.cast(), bytes) {
                return -1;
            }
        }
        if fmt != HTS_FMT_CSI {
            let Some(x) = bgzf_read_u32(fp) else {
                return -1;
            };
            if x > i32::MAX as u32 {
                return -3;
            }
            (*l).n = x as hts_pos_t;
            (*l).m = x as hts_pos_t;
            let Some(bytes) = (x as usize).checked_mul(std::mem::size_of::<u64>()) else {
                return -2;
            };
            let mut offset: Vec<u64> = vec![0u64; x as usize];
            let offset_ptr = offset.as_mut_ptr();
            std::mem::forget(offset);
            (*l).offset = offset_ptr;
            if !bgzf_read_exact(fp, (*l).offset.cast(), bytes) {
                return -1;
            }
            let mut k = 0;
            let mut j = 0;
            while j < (*l).n && *(*l).offset.add(j as usize) == 0 {
                k = j + 1;
                j += 1;
            }
            j = (*l).n - 1;
            while j > k {
                if *(*l).offset.add((j - 1) as usize) == 0 {
                    *(*l).offset.add((j - 1) as usize) = *(*l).offset.add(j as usize);
                }
                j -= 1;
            }
            hts_c_2431_update_loff(idx, i, 0);
        }
    }
    (*idx).n_no_coor = bgzf_read_u64(fp).unwrap_or(0);
    0
}

pub unsafe fn hts_c_2990_idx_read(fn_: *const c_char) -> *mut hts_idx_t {
    let mut idx: Option<NonNull<hts_idx_t>> = None;
    let mut meta: Option<Vec<u8>> = None;
    let Some(fp) = NonNull::new(bgzf_open(fn_.cast(), c"r".as_ptr().cast())) else {
        return std::ptr::null_mut();
    };

    let mut magic = [0u8; 4];
    if !bgzf_read_exact(fp.as_ptr(), magic.as_mut_ptr().cast(), 4) {
        goto_idx_read_fail(fp, idx, meta);
        return std::ptr::null_mut();
    }

    if &magic == b"CSI\x01" {
        let Some(min_shift) = bgzf_read_u32(fp.as_ptr()) else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        let Some(n_lvls) = bgzf_read_u32(fp.as_ptr()) else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        let Some(l_meta) = bgzf_read_u32(fp.as_ptr()) else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        if l_meta != 0 {
            // Owned meta buffer of exactly `l_meta` bytes.
            let mut meta_vec: Vec<u8> = vec![0u8; l_meta as usize];
            if !bgzf_read_exact(fp.as_ptr(), meta_vec.as_mut_ptr().cast(), l_meta as usize) {
                goto_idx_read_fail(fp, idx, Some(meta_vec));
                return std::ptr::null_mut();
            }
            meta = Some(meta_vec);
        }
        let Some(n) = bgzf_read_u32(fp.as_ptr()) else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        if n > i32::MAX as u32 || min_shift > i32::MAX as u32 || n_lvls > i32::MAX as u32 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
        idx = NonNull::new(hts_c_2405_hts_idx_init(
            n as i32,
            HTS_FMT_CSI,
            0,
            min_shift as i32,
            n_lvls as i32,
        ));
        let Some(idx_ptr) = idx else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        (*idx_ptr.as_ptr()).l_meta = l_meta;
        (*idx_ptr.as_ptr()).meta = match meta.take() {
            Some(v) => Box::into_raw(v.into_boxed_slice()).cast::<u8>(),
            None => std::ptr::null_mut(),
        };
        if hts_c_2925_idx_read_core(idx_ptr.as_ptr(), fp.as_ptr(), HTS_FMT_CSI) < 0 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
    } else if &magic == b"TBI\x01" {
        let mut x = [0u8; 8 * 4];
        if !bgzf_read_exact(fp.as_ptr(), x.as_mut_ptr().cast(), x.len()) {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
        let n = u32::from_le_bytes([x[0], x[1], x[2], x[3]]);
        if n > i32::MAX as u32 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
        idx = NonNull::new(hts_c_2405_hts_idx_init(
            n as i32,
            HTS_FMT_TBI as i32,
            0,
            14,
            5,
        ));
        let Some(idx_ptr) = idx else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        let name_len = u32::from_le_bytes([x[28], x[29], x[30], x[31]]);
        if name_len > u32::MAX - 29 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
        (*idx_ptr.as_ptr()).l_meta = 28 + name_len;
        // Owned meta buffer of exactly `l_meta` bytes.
        let mut tbi_meta_vec: Vec<u8> = vec![0u8; (*idx_ptr.as_ptr()).l_meta as usize];
        let tbi_meta_ptr = tbi_meta_vec.as_mut_ptr();
        std::mem::forget(tbi_meta_vec);
        let tbi_meta = NonNull::new(tbi_meta_ptr).unwrap();
        (*idx_ptr.as_ptr()).meta = tbi_meta.as_ptr();
        std::ptr::copy_nonoverlapping(x.as_ptr().add(4), tbi_meta.as_ptr(), 28);
        if !bgzf_read_exact(
            fp.as_ptr(),
            tbi_meta.as_ptr().add(28).cast(),
            name_len as usize,
        ) {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
        if hts_c_2925_idx_read_core(idx_ptr.as_ptr(), fp.as_ptr(), HTS_FMT_TBI as i32) < 0 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
    } else if &magic == b"BAI\x01" {
        let Some(n) = bgzf_read_u32(fp.as_ptr()) else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        if n > i32::MAX as u32 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
        idx = NonNull::new(hts_c_2405_hts_idx_init(n as i32, HTS_FMT_BAI, 0, 14, 5));
        let Some(idx_ptr) = idx else {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        };
        if hts_c_2925_idx_read_core(idx_ptr.as_ptr(), fp.as_ptr(), HTS_FMT_BAI) < 0 {
            goto_idx_read_fail(fp, idx, meta);
            return std::ptr::null_mut();
        }
    } else {
        *libc::__errno_location() = libc::EINVAL;
        goto_idx_read_fail(fp, idx, meta);
        return std::ptr::null_mut();
    }

    bgzf_close(fp.as_ptr());
    idx.map_or(std::ptr::null_mut(), NonNull::as_ptr)
}

pub unsafe fn hts_c_4623_idx_test_and_fetch(
    fn_: *const c_char,
    local_fn: *mut *const c_char,
    local_len: *mut i32,
    download: i32,
) -> i32 {
    if fn_.is_null() {
        return -1;
    }

    if hisremote(fn_.cast()) != 0 {
        let buf_size = 1024 * 1024usize;
        let mut fmt: htsFormat = std::mem::zeroed();
        let mut s: kstring_t = kstring_t::default();
        let mut tmps: kstring_t = kstring_t::default();

        let non_s3 = libc::strncmp(fn_, c"s3://".as_ptr(), 5) != 0
            && libc::strncmp(fn_, c"s3+http://".as_ptr(), 10) != 0
            && libc::strncmp(fn_, c"s3+https://".as_ptr(), 11) != 0;
        let e = fn_.add(if non_s3 {
            libc::strcspn(fn_, c"?#".as_ptr())
        } else {
            libc::strcspn(fn_, c"?".as_ptr())
        });
        let mut p = e;
        while p > fn_ && *p != b'/' as c_char {
            p = p.sub(1);
        }
        if *p == b'/' as c_char {
            p = p.add(1);
        }

        if kputsn(
            std::slice::from_raw_parts(p.cast::<u8>(), e.offset_from(p) as usize),
            e.offset_from(p) as usize,
            &mut s,
        ) < 0
        {
            return -2;
        }
        s.data.push(0);
        if libc::access(s.data.as_ptr().cast(), libc::R_OK) == 0 {
            *local_fn = p;
            *local_len = e.offset_from(p) as i32;
            return 0;
        }

        let remote_hfp = hopen(fn_.cast(), c"r".as_ptr().cast());
        if remote_hfp.is_null() {
            return -1;
        }
        if hts_c_556_hts_detect_format2(remote_hfp, fn_, &mut fmt) != 0 {
            let save_errno = *libc::__errno_location();
            hclose_abruptly(remote_hfp);
            *libc::__errno_location() = save_errno;
            return -2;
        }
        if fmt.category != HTS_FORMAT_INDEX_FILE
            || !(fmt.format == HTS_FORMAT_BAI
                || fmt.format == HTS_FORMAT_CSI
                || fmt.format == HTS_FORMAT_TBI
                || fmt.format == HTS_FORMAT_CRAI_EXACT
                || fmt.format == HTS_FORMAT_FAI_FORMAT)
        {
            let save_errno = *libc::__errno_location();
            hclose_abruptly(remote_hfp);
            *libc::__errno_location() = save_errno;
            return -2;
        }

        if download != 0 {
            let local_fp = hts_c_1979_hts_open_tmpfile(s.data.as_ptr().cast(), c"wx".as_ptr(), &mut tmps);
            tmps.data.push(0);
            if local_fp.is_null() {
                let save_errno = *libc::__errno_location();
                hclose_abruptly(remote_hfp);
                *libc::__errno_location() = save_errno;
                return -2;
            }
            // Scratch I/O buffer owned for the duration of the copy loop.
            let mut buf: Vec<u8> = vec![0u8; buf_size];
            loop {
                let l = htslib_hfile_h_247_hread(remote_hfp, buf.as_mut_ptr().cast(), buf_size);
                if l <= 0 {
                    if l < 0 {
                        let save_errno = *libc::__errno_location();
                        hclose_abruptly(remote_hfp);
                        hclose_abruptly(local_fp);
                        if !tmps.data.is_empty() {
                            libc::unlink(tmps.data.as_ptr().cast());
                        }
                        *libc::__errno_location() = save_errno;
                        return -2;
                    }
                    break;
                }
                if htslib_hfile_h_292_hwrite(local_fp, buf.as_ptr().cast(), l as usize) != l {
                    let save_errno = *libc::__errno_location();
                    hclose_abruptly(remote_hfp);
                    hclose_abruptly(local_fp);
                    if !tmps.data.is_empty() {
                        libc::unlink(tmps.data.as_ptr().cast());
                    }
                    *libc::__errno_location() = save_errno;
                    return -2;
                }
            }
            if hclose(local_fp) < 0 {
                let save_errno = *libc::__errno_location();
                hclose_abruptly(remote_hfp);
                if !tmps.data.is_empty() {
                    libc::unlink(tmps.data.as_ptr().cast());
                }
                *libc::__errno_location() = save_errno;
                return -2;
            }
            if libc::rename(tmps.data.as_ptr().cast(), s.data.as_ptr().cast()) < 0 {
                let save_errno = *libc::__errno_location();
                hclose_abruptly(remote_hfp);
                if !tmps.data.is_empty() {
                    libc::unlink(tmps.data.as_ptr().cast());
                }
                *libc::__errno_location() = save_errno;
                return -2;
            }
            ks_clear(&mut tmps);
            *local_fn = p;
            *local_len = e.offset_from(p) as i32;
        } else {
            *local_fn = fn_;
            *local_len = e.offset_from(fn_) as i32;
        }

        hclose(remote_hfp);
        return 0;
    }

    let local_hfp = hopen(fn_.cast(), c"r".as_ptr().cast());
    if !local_hfp.is_null() {
        hclose_abruptly(local_hfp);
        *local_fn = fn_;
        *local_len = CStr::from_ptr(fn_).to_bytes().len() as i32;
        0
    } else {
        -1
    }
}

pub unsafe fn hts_c_4756_hts_idx_check_local(
    fn_: *const c_char,
    fmt: i32,
    fnidx: *mut *mut c_char,
) -> i32 {
    if fn_.is_null() {
        return 0;
    }
    let bytes = CStr::from_ptr(fn_).to_bytes();
    let fn_tmp = if hisremote(fn_.cast()) != 0 {
        match bytes.iter().rposition(|&b| b == b'/') {
            Some(pos) => &bytes[pos + 1..],
            None => return 0,
        }
    } else if bytes.starts_with(b"file://localhost/") {
        &bytes[16..]
    } else if bytes.starts_with(b"file:///") {
        &bytes[7..]
    } else {
        bytes
    };

    let mut candidates: Vec<Vec<u8>> = Vec::new();
    let push_pair = |candidates: &mut Vec<Vec<u8>>, ext: &[u8]| {
        let mut appended = fn_tmp.to_vec();
        appended.extend_from_slice(ext);
        candidates.push(appended);
        if let Some(pos) = fn_tmp.iter().rposition(|&b| b == b'.') {
            if pos > 0 {
                let mut replaced = fn_tmp[..pos].to_vec();
                replaced.extend_from_slice(ext);
                candidates.push(replaced);
            }
        }
    };

    push_pair(&mut candidates, b".csi");
    if fmt == HTS_FMT_BAI {
        push_pair(&mut candidates, b".bai");
    } else if fmt == HTS_FMT_TBI as i32 {
        push_pair(&mut candidates, b".tbi");
    } else if fmt == HTS_FMT_CRAI {
        push_pair(&mut candidates, b".crai");
    } else if fmt == HTS_FMT_FAI {
        let mut gzi_ok = true;
        if fn_tmp.ends_with(b".gz") || fn_tmp.ends_with(b".bgzf") {
            let mut gzi = fn_tmp.to_vec();
            gzi.extend_from_slice(b".gzi");
            gzi_ok = path_from_bytes(&gzi).exists();
        }

        let mut fai = fn_tmp.to_vec();
        fai.extend_from_slice(b".fai");
        let exists = gzi_ok && path_from_bytes(&fai).exists();
        let mut fai_c = fai;
        fai_c.push(0);
        *fnidx = Box::into_raw(fai_c.into_boxed_slice()).cast::<c_char>();
        return exists as i32;
    }

    for candidate in candidates {
        if path_from_bytes(&candidate).exists() {
            let mut out = candidate;
            out.push(0);
            *fnidx = Box::into_raw(out.into_boxed_slice()).cast::<c_char>();
            return 1;
        }
    }
    0
}

pub unsafe fn hts_c_4885_idx_filename(
    fn_: *const c_char,
    ext: *const c_char,
    download: i32,
) -> *mut c_char {
    if fn_.is_null() || ext.is_null() {
        return std::ptr::null_mut();
    }
    let fn_bytes = CStr::from_ptr(fn_).to_bytes();
    let ext_bytes = CStr::from_ptr(ext).to_bytes();
    let mut candidates = Vec::new();
    let mut appended = fn_bytes.to_vec();
    appended.extend_from_slice(ext_bytes);
    candidates.push(appended);
    if let Some(pos) = fn_bytes.iter().rposition(|&b| b == b'.') {
        let mut replaced = fn_bytes[..pos].to_vec();
        replaced.extend_from_slice(ext_bytes);
        candidates.push(replaced);
    }
    for candidate in candidates {
        // Owned nul-terminated copy of the candidate path.
        let mut c_buf = candidate;
        c_buf.push(0);
        let c_candidate = c_buf.as_mut_ptr().cast::<c_char>();
        let mut local_fn = std::ptr::null();
        let mut local_len = 0;
        let ret =
            hts_c_4623_idx_test_and_fetch(c_candidate, &mut local_fn, &mut local_len, download);
        if ret == 0 {
            // Replace contents with the fetched local filename and return it.
            let mut out: Vec<u8> = vec![0u8; local_len as usize + 1];
            std::ptr::copy_nonoverlapping(local_fn.cast::<u8>(), out.as_mut_ptr(), local_len as usize);
            drop(c_buf);
            return Box::into_raw(out.into_boxed_slice()).cast::<c_char>();
        }
        drop(c_buf);
        if ret < -1 {
            return std::ptr::null_mut();
        }
    }
    std::ptr::null_mut()
}

pub unsafe fn hts_c_4915_hts_idx_getfn(fn_: *const c_char, ext: *const c_char) -> *mut c_char {
    hts_c_4885_idx_filename(fn_, ext, HTS_IDX_SAVE_REMOTE)
}

pub unsafe fn hts_c_4920_hts_idx_locatefn(fn_: *const c_char, ext: *const c_char) -> *mut c_char {
    hts_c_4885_idx_filename(fn_, ext, 0)
}

pub unsafe fn hts_c_4925_idx_find_and_load(
    fn_: *const c_char,
    fmt: i32,
    flags: i32,
) -> *mut hts_idx_t {
    if fn_.is_null() {
        return std::ptr::null_mut();
    }
    let delim = b"##idx##";
    let fn_bytes = CStr::from_ptr(fn_).to_bytes();
    if let Some(pos) = fn_bytes.windows(delim.len()).position(|w| w == delim) {
        // Owned nul-terminated copies of the split filename / index paths.
        let mut fn2_buf: Vec<u8> = fn_bytes[..pos].to_vec();
        fn2_buf.push(0);
        let fnidx_bytes = &fn_bytes[pos + delim.len()..];
        let mut fnidx_buf: Vec<u8> = fnidx_bytes.to_vec();
        fnidx_buf.push(0);
        let idx = hts_idx_load3(
            fn2_buf.as_ptr().cast::<c_char>(),
            fnidx_buf.as_ptr().cast::<c_char>(),
            fmt,
            flags,
        );
        drop(fn2_buf);
        drop(fnidx_buf);
        return idx;
    }

    let mut fnidx = std::ptr::null_mut();
    if hts_c_4756_hts_idx_check_local(fn_, fmt, &mut fnidx) == 0 {
        if (flags & HTS_IDX_SILENT_FAIL) == 0 {
            *libc::__errno_location() = libc::ENOENT;
        }
        return std::ptr::null_mut();
    }
    let idx = hts_c_2990_idx_read(fnidx);
    // `fnidx` was allocated by hts_c_4756_hts_idx_check_local as an owned C string.
    if !fnidx.is_null() {
        let len = CStr::from_ptr(fnidx).to_bytes_with_nul().len();
        drop(Box::from_raw(std::slice::from_raw_parts_mut(
            fnidx.cast::<u8>(),
            len,
        )));
    }
    idx
}

unsafe fn goto_idx_read_fail(
    fp: NonNull<BGZF>,
    idx: Option<NonNull<hts_idx_t>>,
    meta: Option<Vec<u8>>,
) {
    bgzf_close(fp.as_ptr());
    if let Some(idx) = idx {
        hts_idx_destroy(idx.as_ptr());
    }
    // `meta`, if present, is dropped here, freeing the buffer.
    drop(meta);
}

pub unsafe fn hts_idx_load3(
    fn_: *const c_char,
    fnidx: *const c_char,
    fmt: i32,
    flags: i32,
) -> *mut hts_idx_t {
    // Mirrors htslib/hts.c:4989 sam_hdr_load3. When fnidx is null we have
    // to resolve a sidecar index path (or `fn##idx##path` inline form);
    // delegate to idx_find_and_load. With fnidx, the file is read directly
    // via idx_read after an optional remote-fetch (handled by
    // hts_idx_load3_local_index when the path is on local disk).
    if fnidx.is_null() {
        return hts_c_4925_idx_find_and_load(fn_, fmt, flags);
    }
    if let Some(idx) = hts_idx_load3_local_index(fn_, fnidx, fmt) {
        return idx;
    }
    // Path exists check failed (e.g. remote fnidx not yet cached); fall
    // through to the find_and_load resolver which handles remote-fetch.
    hts_c_4925_idx_find_and_load(fn_, fmt, flags)
}

pub unsafe fn hts_idx_load(fn_: *const c_char, fmt: i32) -> *mut hts_idx_t {
    hts_idx_load3(fn_, std::ptr::null(), fmt, HTS_IDX_SAVE_REMOTE)
}

pub unsafe fn hts_idx_load2(fn_: *const c_char, fnidx: *const c_char) -> *mut hts_idx_t {
    hts_idx_load3(fn_, fnidx, 0, 0)
}

pub unsafe fn hts_idx_get_meta(idx: *mut hts_idx_t, l_meta: *mut u32) -> *mut u8 {
    hts_c_3084_hts_idx_get_meta(idx, l_meta)
}

pub unsafe fn hts_c_3084_hts_idx_get_meta(idx: *mut hts_idx_t, l_meta: *mut u32) -> *mut u8 {
    *l_meta = (*idx).l_meta;
    (*idx).meta
}

pub unsafe fn hts_idx_set_meta(
    idx: *mut hts_idx_t,
    l_meta: u32,
    meta: *mut u8,
    is_copy: i32,
) -> i32 {
    hts_c_3062_hts_idx_set_meta(idx, l_meta, meta, is_copy)
}

pub unsafe fn hts_c_3062_hts_idx_set_meta(
    idx: *mut hts_idx_t,
    l_meta: u32,
    meta: *mut u8,
    is_copy: i32,
) -> i32 {
    let mut new_meta = meta;
    if is_copy != 0 {
        let l = l_meta as usize;
        if l > usize::MAX - 1 {
            *libc::__errno_location() = libc::ENOMEM;
            return -1;
        }
        // Owned copy of exactly `l` bytes.
        let mut copy: Vec<u8> = vec![0u8; l];
        std::ptr::copy_nonoverlapping(meta, copy.as_mut_ptr(), l);
        new_meta = Box::into_raw(copy.into_boxed_slice()).cast::<u8>();
    }
    if !(*idx).meta.is_null() {
        let old_l = (*idx).l_meta as usize;
        drop(Box::from_raw(std::slice::from_raw_parts_mut((*idx).meta, old_l)));
    }
    (*idx).l_meta = l_meta;
    (*idx).meta = new_meta;
    0
}

pub unsafe fn hts_c_3090_hts_idx_seqnames(
    idx: *const hts_idx_t,
    n: *mut i32,
    getid: hts_id2name_f,
    hdr: *mut (),
) -> *mut *const c_char {
    if idx.is_null() || (*idx).n == 0 {
        *n = 0;
        return std::ptr::null_mut();
    }

    let names_vec: Vec<*const c_char> = vec![std::ptr::null(); (*idx).n as usize];
    let names = Box::into_raw(names_vec.into_boxed_slice()).cast::<*const c_char>();
    let mut tid = 0;
    for i in 0..(*idx).n {
        let bidx = *(*idx).bidx.add(i as usize);
        if bidx.is_null() {
            continue;
        }
        *names.add(tid as usize) = getid.map_or(std::ptr::null(), |f| f(hdr.cast(), i));
        tid += 1;
    }
    *n = tid;
    names
}

pub unsafe fn hts_c_3115_hts_idx_get_stat(
    idx: *const hts_idx_t,
    tid: i32,
    mapped: *mut u64,
    unmapped: *mut u64,
) -> i32 {
    if idx.is_null() {
        return -1;
    }
    if (*idx).fmt == HTS_FMT_CRAI {
        *mapped = 0;
        *unmapped = 0;
        return -1;
    }

    let h = *(*idx).bidx.add(tid as usize);
    if h.is_null() {
        return -1;
    }
    let k = kh_get_bin(h, meta_bin(idx));
    if k != (*h).n_buckets {
        *mapped = (*(*(*h).vals.add(k as usize)).list.add(1)).u;
        *unmapped = (*(*(*h).vals.add(k as usize)).list.add(1)).v;
        0
    } else {
        *mapped = 0;
        *unmapped = 0;
        -1
    }
}

pub unsafe fn hts_c_3136_hts_idx_get_n_no_coor(idx: *const hts_idx_t) -> u64 {
    if (*idx).fmt == HTS_FMT_CRAI {
        0
    } else {
        (*idx).n_no_coor
    }
}

pub unsafe fn hts_idx_get_stat(
    idx: *const hts_idx_t,
    tid: i32,
    mapped: *mut u64,
    unmapped: *mut u64,
) -> i32 {
    hts_c_3115_hts_idx_get_stat(idx, tid, mapped, unmapped)
}

pub unsafe fn hts_idx_get_n_no_coor(idx: *const hts_idx_t) -> u64 {
    hts_c_3136_hts_idx_get_n_no_coor(idx)
}

pub unsafe fn hts_idx_seqnames(
    idx: *const hts_idx_t,
    n: *mut i32,
    getid: hts_id2name_f,
    hdr: *mut (),
) -> *mut *const c_char {
    hts_c_3090_hts_idx_seqnames(idx, n, getid, hdr)
}

pub unsafe extern "C" fn hts_itr_multi_bam(idx: *const hts_idx_t, iter: *mut hts_itr_t) -> i32 {
    hts_c_3602_hts_itr_multi_bam(idx, iter)
}

pub unsafe fn hts_c_3602_hts_itr_multi_bam(idx: *const hts_idx_t, iter: *mut hts_itr_t) -> i32 {
    if idx.is_null() || iter.is_null() || ((*iter).bitfields & (1 << 4)) == 0 {
        return -1;
    }

    (*iter).i = -1;
    for i in 0..(*iter).n_reg {
        let curr_reg = (*iter).reg_list.add(i as usize);
        let tid = (*curr_reg).tid;

        if tid < 0 {
            let t_off = hts_itr_off(idx, tid);
            if t_off != u64::MAX {
                match tid {
                    HTS_IDX_NONE => {
                        itr_set_finished(iter);
                        (*iter).curr_off = t_off;
                        (*iter).n_reg = 0;
                        (*iter).reg_list = std::ptr::null_mut();
                        (*iter).bitfields |= 1;
                        return 0;
                    }
                    HTS_IDX_START | HTS_IDX_REST => {
                        (*iter).curr_off = t_off;
                        (*iter).n_reg = 0;
                        (*iter).reg_list = std::ptr::null_mut();
                        (*iter).bitfields |= 1;
                        return 0;
                    }
                    HTS_IDX_NOCOOR => {
                        (*iter).bitfields |= 1 << 3;
                        (*iter).nocoor_off = t_off;
                    }
                    _ => {}
                }
            }
            continue;
        }

        if tid >= (*idx).n {
            continue;
        }
        let bidx = *(*idx).bidx.add(tid as usize);
        if bidx.is_null() || (*bidx).size == 0 {
            continue;
        }

        let k_meta = kh_get_bin(bidx, meta_bin(idx));
        let unmapped = if k_meta != (*bidx).n_buckets {
            (*(*(*bidx).vals.add(k_meta as usize)).list.add(1)).v as u32
        } else {
            1
        };
        let idx_maxpos = hts_c_2533_hts_idx_maxpos(idx);

        for j in 0..(*curr_reg).count {
            let curr_intv = (*curr_reg).intervals.add(j as usize);
            if (*curr_intv).end < (*curr_intv).beg {
                continue;
            }

            let beg = (*curr_intv).beg;
            let end = (*curr_intv).end;
            if beg >= idx_maxpos {
                continue;
            }
            let rel_off = (beg >> (*idx).min_shift) as u32;
            let mut bin = hts_bin_first((*idx).n_lvls) as u32 + rel_off;
            let mut k;
            loop {
                k = kh_get_bin(bidx, bin);
                if k != (*bidx).n_buckets {
                    break;
                }
                let first = ((hts_bin_parent(bin as i32) << 3) + 1) as u32;
                if bin > first {
                    bin -= 1;
                } else {
                    bin = hts_bin_parent(bin as i32) as u32;
                }
                if bin == 0 {
                    break;
                }
            }
            if bin == 0 {
                k = kh_get_bin(bidx, bin);
            }
            let mut min_off = if k != (*bidx).n_buckets {
                (*(*bidx).vals.add(k as usize)).loff
            } else {
                0
            };

            let lidx = (*idx).lidx.add(tid as usize);
            if !(*lidx).offset.is_null() && (rel_off as hts_pos_t) < (*lidx).n {
                let lin = *(*lidx).offset.add(rel_off as usize);
                if min_off < lin {
                    min_off = lin;
                }
                if unmapped != 0 {
                    let mut tmp_off = rel_off as i32 - 1;
                    while tmp_off >= 0 {
                        let off = *(*lidx).offset.add(tmp_off as usize);
                        if off < min_off {
                            min_off = off;
                            break;
                        }
                        tmp_off -= 1;
                    }
                    if k != (*bidx).n_buckets
                        && (min_off < (*(*bidx).vals.add(k as usize)).loff || tmp_off < 0)
                    {
                        min_off = (*(*bidx).vals.add(k as usize)).loff;
                    }
                }
            } else if unmapped != 0 && k != (*bidx).n_buckets {
                min_off = (*(*bidx).vals.add(k as usize)).loff;
            }

            let max_off = if end <= idx_maxpos {
                let mut bin = (hts_bin_first((*idx).n_lvls) as hts_pos_t
                    + ((end - 1) >> (*idx).min_shift)
                    + 1) as u32;
                if bin >= (*idx).n_bins as u32 {
                    bin = 0;
                }
                loop {
                    while bin % 8 == 1 {
                        bin = hts_bin_parent(bin as i32) as u32;
                    }
                    if bin == 0 {
                        break u64::MAX;
                    }
                    k = kh_get_bin(bidx, bin);
                    if k != (*bidx).n_buckets && (*(*bidx).vals.add(k as usize)).n > 0 {
                        break (*(*(*bidx).vals.add(k as usize)).list).u;
                    }
                    bin = bin.wrapping_add(1);
                }
            } else {
                u64::MAX
            };

            if hts_c_3304_reg2intervals(
                iter,
                idx,
                tid,
                beg,
                end,
                j,
                min_off,
                max_off,
                (*idx).min_shift,
                (*idx).n_lvls,
            ) < 0
            {
                return -1;
            }
        }
    }

    if (*iter).n_off > 1 {
        let off = std::slice::from_raw_parts_mut((*iter).off, (*iter).n_off as usize);
        off.sort_by(|a, b| a.u.cmp(&b.u).then_with(|| a.max.cmp(&b.max)));
    }
    if (*iter).n_off == 0 && !itr_nocoor(iter) {
        itr_set_finished(iter);
    }
    0
}

pub unsafe extern "C" fn hts_itr_multi_cram(idx: *const hts_idx_t, iter: *mut hts_itr_t) -> i32 {
    hts_c_3748_hts_itr_multi_cram(idx, iter)
}

pub unsafe fn hts_c_3748_hts_itr_multi_cram(idx: *const hts_idx_t, iter: *mut hts_itr_t) -> i32 {
    let cidx = idx.cast::<hts_cram_idx_t>();
    if cidx.is_null() || iter.is_null() || ((*iter).bitfields & (1 << 4)) == 0 {
        return -1;
    }

    (*iter).bitfields |= 1 << 2;
    (*iter).bitfields &= !1;
    (*iter).off = std::ptr::null_mut();
    (*iter).n_off = 0;
    (*iter).curr_off = 0;
    (*iter).i = -1;

    let mut off: Vec<hts_pair64_max_t> = Vec::new();
    let mut n_off = 0usize;

    for i in 0..(*iter).n_reg {
        let curr_reg = (*iter).reg_list.add(i as usize);
        let tid = (*curr_reg).tid;

        if tid >= 0 {
            let count = (*curr_reg).count as usize;
            if count == 0 {
                continue;
            }
            let new_len = match n_off.checked_add(count) {
                Some(len) => len,
                None => return hts_itr_multi_cram_err(off),
            };
            off.resize(new_len, std::mem::zeroed());
            let off_ptr = off.as_mut_ptr();

            for j in 0..(*curr_reg).count {
                let curr_intv = (*curr_reg).intervals.add(j as usize);
                if (*curr_intv).end < (*curr_intv).beg {
                    continue;
                }

                let beg = (*curr_intv).beg;
                let end = (*curr_intv).end;
                let Some(e) = cram_index_query(&*(*cidx).cram.cast::<cram_fd_layout>(), tid, beg + 1, None) else {
                    continue;
                };
                let e = e.as_ptr();

                (*off_ptr.add(n_off)).u = (*e).offset as u64;
                (*off_ptr.add(n_off)).max = ((tid as u64) << 32) | j as u64;

                let e = if end >= HTS_POS_MAX {
                    cram_index_last(&*(*cidx).cram.cast::<cram_fd_layout>(), tid, None)
                } else {
                    cram_index_query_last(&*(*cidx).cram.cast::<cram_fd_layout>(), tid, end + 1)
                };

                if let Some(e) = e {
                    let e = e.as_ptr();
                    let e_next = (*e).e_next;
                    (*off_ptr.add(n_off)).v = if !e_next.is_null() {
                        (*e_next).offset as u64
                    } else {
                        ((*e).offset + (*e).slice as i64 + (*e).len as i64) as u64
                    };
                    n_off += 1;
                } else {
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        b"hts_itr_multi_cram",
                        b"Could not set offset end for region; skipping",
                    );
                }
            }
        } else {
            match tid {
                HTS_IDX_NOCOOR => {
                    let e = cram_index_query(&*(*cidx).cram.cast::<cram_fd_layout>(), tid, 1, None);
                    if let Some(e) = e {
                        let e = e.as_ptr();
                        (*iter).bitfields |= 1 << 3;
                        (*iter).nocoor_off = (*e).offset as u64;
                    } else {
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            b"hts_itr_multi_cram",
                            b"No index entry for NOCOOR region",
                        );
                    }
                }
                HTS_IDX_START => {
                    let e = cram_index_query(&*(*cidx).cram.cast::<cram_fd_layout>(), tid, 1, None);
                    if let Some(e) = e {
                        let e = e.as_ptr();
                        (*iter).bitfields |= 1;
                        off.resize(1, std::mem::zeroed());
                        off[0].u = (*e).offset as u64;
                        off[0].v = 0;
                        off[0].max = 0;
                        n_off = 1;
                    } else {
                        hts_log_cstr(
                            HTS_LOG_WARNING,
                            b"hts_itr_multi_cram",
                            b"No index entries",
                        );
                    }
                }
                HTS_IDX_REST => {}
                HTS_IDX_NONE => {
                    itr_set_finished(iter);
                }
                _ => {
                    hts_log_cstr(
                        HTS_LOG_ERROR,
                        b"hts_itr_multi_cram",
                        b"Query with this tid is not implemented for CRAM files",
                    );
                }
            }
        }
    }

    if n_off != 0 {
        off[..n_off].sort_by(|a, b| a.u.cmp(&b.u).then_with(|| a.max.cmp(&b.max)));
        (*iter).n_off = n_off as i32;
        // Hand ownership of the buffer to the raw `(*iter).off` field.
        let off_ptr = off.as_mut_ptr();
        std::mem::forget(off);
        (*iter).off = off_ptr;
    }
    // Otherwise `off` is dropped here, freeing the buffer.

    if n_off == 0 && !itr_nocoor(iter) {
        itr_set_finished(iter);
    }
    0
}

unsafe fn hts_itr_multi_cram_err(_off: Vec<hts_pair64_max_t>) -> i32 {
    // `_off` is dropped here, freeing the buffer.
    -1
}

pub unsafe fn hts_itr_querys(
    idx: *const hts_idx_t,
    reg: *const c_char,
    getid: hts_name2id_f,
    hdr: *mut (),
    itr_query: hts_itr_query_func,
    readrec: hts_readrec_func,
) -> *mut hts_itr_t {
    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;

    if reg.is_null() {
        return std::ptr::null_mut();
    }

    if libc::strcmp(reg, c".".as_ptr()) == 0 {
        return itr_query.map_or(std::ptr::null_mut(), |f| {
            f(idx, HTS_IDX_START, 0, 0, readrec)
        });
    } else if libc::strcmp(reg, c"*".as_ptr()) == 0 {
        return itr_query.map_or(std::ptr::null_mut(), |f| {
            f(idx, HTS_IDX_NOCOOR, 0, 0, readrec)
        });
    }

    if hts_parse_region(
        CStr::from_ptr(reg),
        &mut tid,
        &mut beg,
        &mut end,
        getid,
        hdr,
        HTS_PARSE_THOUSANDS_SEP,
    )
    .is_null()
    {
        return std::ptr::null_mut();
    }

    itr_query.map_or(std::ptr::null_mut(), |f| f(idx, tid, beg, end, readrec))
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn hts_itr_regions(
    idx: *const hts_idx_t,
    regs: &mut [hts_reglist_t],
    getid: hts_name2id_f,
    hdr: *mut (),
    itr_specific: hts_itr_multi_query_func,
    readrec: hts_readrec_func,
    seek: hts_seek_func,
    tell: hts_tell_func,
) -> *mut hts_itr_t {
    let itr = Box::into_raw(Box::new(std::mem::zeroed::<hts_itr_t>()));

    (*itr).n_reg = regs.len() as i32;
    (*itr).readrec = readrec;
    (*itr).seek = seek;
    (*itr).tell = tell;
    (*itr).reg_list = regs.as_mut_ptr();
    (*itr).bitfields |= 1 << 4;

    for curr in regs.iter_mut() {
        if !curr.reg.is_null() {
            if libc::strcmp(curr.reg, c".".as_ptr()) == 0 {
                curr.tid = HTS_IDX_START;
                continue;
            }
            if libc::strcmp(curr.reg, c"*".as_ptr()) == 0 {
                curr.tid = HTS_IDX_NOCOOR;
                continue;
            }
            curr.tid = getid.map_or(-1, |f| f(hdr.cast(), curr.reg));
            if curr.tid < 0 {
                if curr.tid < -1 {
                    hts_log_cstr(
                        HTS_LOG_ERROR,
                        b"hts_itr_regions",
                        b"Failed to parse header",
                    );
                    hts_itr_destroy(itr);
                    return std::ptr::null_mut();
                } else {
                    eprintln!(
                        "[W::hts_itr_regions] Region '{}' specifies an unknown reference name. Continue anyway",
                        String::from_utf8_lossy(CStr::from_ptr(curr.reg).to_bytes()),
                    );
                }
            }
        }
    }

    if regs.len() > 1 {
        regs.sort_by(|a, b| compare_regions(a, b).cmp(&0));
    }

    if itr_specific.map_or(-1, |f| f(idx, itr)) != 0 {
        hts_log_cstr(
            HTS_LOG_ERROR,
            b"hts_itr_regions",
            b"Failed to create the multi-region iterator!",
        );
        hts_itr_destroy(itr);
        std::ptr::null_mut()
    } else {
        itr
    }
}

fn compare_regions(reg1: &hts_reglist_t, reg2: &hts_reglist_t) -> i32 {
    if reg1.tid < 0 && reg2.tid >= 0 {
        1
    } else if reg1.tid >= 0 && reg2.tid < 0 {
        -1
    } else {
        reg1.tid - reg2.tid
    }
}

unsafe fn hts_idx_load3_local_index(
    fn_: *const c_char,
    fnidx: *const c_char,
    fmt: i32,
) -> Option<*mut hts_idx_t> {
    let idx_path = local_index_path(fn_, fnidx, fmt)?;
    let idx_c = std::ffi::CString::new(path_bytes(&idx_path).as_ref()).ok()?;
    let fp = NonNull::new(bgzf_open(idx_c.as_ptr().cast(), c"r".as_ptr().cast()))?;
    let idx = read_local_index(fp.as_ptr());
    bgzf_close(fp.as_ptr());
    idx
}

unsafe fn local_index_path(
    fn_: *const c_char,
    fnidx: *const c_char,
    fmt: i32,
) -> Option<PathBuf> {
    if !fnidx.is_null() {
        let path = path_from_bytes(CStr::from_ptr(fnidx).to_bytes());
        return path.exists().then_some(path);
    }
    if fn_.is_null() {
        return None;
    }
    let fn_bytes = CStr::from_ptr(fn_).to_bytes();
    let idx_delim = b"##idx##";
    let data_bytes = if let Some(pos) = fn_bytes
        .windows(idx_delim.len())
        .position(|w| w == idx_delim)
    {
        let idx_path = path_from_bytes(&fn_bytes[pos + idx_delim.len()..]);
        return idx_path.exists().then_some(idx_path);
    } else {
        fn_bytes
    };
    let data_path = path_from_bytes(data_bytes);
    if let Some(path) = index_path_with_ext(&data_path, b".csi") {
        return Some(path);
    }
    if fmt == HTS_FMT_CSI {
        return None;
    }
    if fmt == HTS_FMT_TBI as i32 {
        return index_path_with_ext(&data_path, b".tbi");
    }
    index_path_with_ext(&data_path, b".bai")
}

fn index_path_with_ext(data_path: &Path, ext: &[u8]) -> Option<PathBuf> {
    let mut bytes = path_bytes(data_path).into_owned();
    bytes.extend_from_slice(ext);
    let path = path_from_bytes(&bytes);
    if path.exists() {
        return Some(path);
    }
    let mut bytes = path_bytes(data_path).into_owned();
    if let Some(pos) = bytes.iter().rposition(|&b| b == b'.') {
        bytes.truncate(pos);
        bytes.extend_from_slice(ext);
        let path = path_from_bytes(&bytes);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

unsafe fn bgzf_read_exact(fp: *mut BGZF, dst: *mut (), len: usize) -> bool {
    bgzf_read(fp, dst, len) == len as isize
}

unsafe fn bgzf_read_u32(fp: *mut BGZF) -> Option<u32> {
    let mut buf = [0u8; 4];
    bgzf_read_exact(fp, buf.as_mut_ptr().cast(), 4).then(|| u32::from_le_bytes(buf))
}

unsafe fn bgzf_read_u64(fp: *mut BGZF) -> Option<u64> {
    let mut buf = [0u8; 8];
    bgzf_read_exact(fp, buf.as_mut_ptr().cast(), 8).then(|| u64::from_le_bytes(buf))
}

unsafe fn read_local_index(fp: *mut BGZF) -> Option<*mut hts_idx_t> {
    let mut magic = [0u8; 4];
    if !bgzf_read_exact(fp, magic.as_mut_ptr().cast(), 4) {
        return None;
    }
    if &magic == b"CSI\x01" {
        return read_csi_index(fp);
    }
    if &magic == b"TBI\x01" {
        return read_tbi_index(fp);
    }
    if &magic != b"BAI\x01" {
        return None;
    }
    read_bai_index(fp)
}

unsafe fn read_bai_index(fp: *mut BGZF) -> Option<*mut hts_idx_t> {
    let n = bgzf_read_u32(fp)?;
    if n > i32::MAX as u32 {
        return None;
    }
    let idx = hts_idx_init_local(n as i32, HTS_FMT_BAI, 14, 5)?;
    for tid in 0..n as i32 {
        if read_index_reference(fp, idx, tid).is_none() {
            hts_idx_destroy(idx);
            return None;
        }
    }
    (*idx).n_no_coor = bgzf_read_u64(fp).unwrap_or(0);
    Some(idx)
}

unsafe fn read_tbi_index(fp: *mut BGZF) -> Option<*mut hts_idx_t> {
    let mut x = [0u8; 8 * 4];
    if !bgzf_read_exact(fp, x.as_mut_ptr().cast(), x.len()) {
        return None;
    }
    let n = u32::from_le_bytes([x[0], x[1], x[2], x[3]]);
    if n > i32::MAX as u32 {
        return None;
    }
    let idx = hts_idx_init_local(n as i32, HTS_FMT_TBI as i32, 14, 5)?;
    let name_len = u32::from_le_bytes([x[28], x[29], x[30], x[31]]);
    if name_len > u32::MAX - 29 {
        hts_idx_destroy(idx);
        return None;
    }
    (*idx).l_meta = 28 + name_len;
    // Owned meta buffer of exactly `l_meta` bytes.
    let mut meta_vec: Vec<u8> = vec![0u8; (*idx).l_meta as usize];
    (*idx).meta = meta_vec.as_mut_ptr();
    std::mem::forget(meta_vec);
    std::ptr::copy_nonoverlapping(x.as_ptr().add(4), (*idx).meta, 28);
    if !bgzf_read_exact(fp, (*idx).meta.add(28).cast(), name_len as usize) {
        hts_idx_destroy(idx);
        return None;
    }
    for tid in 0..n as i32 {
        if read_index_reference(fp, idx, tid).is_none() {
            hts_idx_destroy(idx);
            return None;
        }
    }
    (*idx).n_no_coor = bgzf_read_u64(fp).unwrap_or(0);
    Some(idx)
}

unsafe fn read_csi_index(fp: *mut BGZF) -> Option<*mut hts_idx_t> {
    let min_shift = bgzf_read_u32(fp)?;
    let n_lvls = bgzf_read_u32(fp)?;
    let l_meta = bgzf_read_u32(fp)?;
    let mut meta: Option<Vec<u8>> = None;
    if l_meta > 0 {
        let mut meta_vec: Vec<u8> = vec![0u8; l_meta as usize];
        if !bgzf_read_exact(fp, meta_vec.as_mut_ptr().cast(), l_meta as usize) {
            // `meta_vec` is dropped here, freeing the buffer.
            return None;
        }
        meta = Some(meta_vec);
    }
    let n = bgzf_read_u32(fp)?;
    if n > i32::MAX as u32 || min_shift > i32::MAX as u32 || n_lvls > i32::MAX as u32 {
        // `meta`, if present, is dropped here.
        return None;
    }
    let idx = hts_idx_init_local(n as i32, HTS_FMT_CSI, min_shift as i32, n_lvls as i32)?;
    (*idx).l_meta = l_meta;
    (*idx).meta = match meta {
        Some(v) => Box::into_raw(v.into_boxed_slice()).cast::<u8>(),
        None => std::ptr::null_mut(),
    };
    for tid in 0..n as i32 {
        if read_index_reference(fp, idx, tid).is_none() {
            hts_idx_destroy(idx);
            return None;
        }
    }
    (*idx).n_no_coor = bgzf_read_u64(fp).unwrap_or(0);
    Some(idx)
}

unsafe fn hts_idx_init_local(
    n: i32,
    fmt: i32,
    min_shift: i32,
    n_lvls: i32,
) -> Option<*mut hts_idx_t> {
    let idx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_t>()));
    (*idx).fmt = fmt;
    (*idx).min_shift = min_shift;
    (*idx).n_lvls = n_lvls;
    (*idx).n_bins = ((1 << (3 * (*idx).n_lvls + 3)) - 1) / 7;
    (*idx).n = n;
    (*idx).m = n;
    (*idx).tbi_n = -1;
    (*idx).last_tbi_tid = -1;
    (*idx).z.save_tid = -1;
    (*idx).z.last_tid = -1;
    (*idx).z.save_bin = u32::MAX;
    (*idx).z.last_bin = u32::MAX;
    (*idx).z.last_coor = u32::MAX as hts_pos_t;
    if n > 0 {
        let bidx: Vec<*mut hts_idx_bidx_t> = vec![std::ptr::null_mut(); n as usize];
        (*idx).bidx = Box::into_raw(bidx.into_boxed_slice()).cast::<*mut hts_idx_bidx_t>();
        let lidx: Vec<hts_idx_lidx_t> = (0..n).map(|_| std::mem::zeroed()).collect();
        (*idx).lidx = Box::into_raw(lidx.into_boxed_slice()).cast::<hts_idx_lidx_t>();
    }
    Some(idx)
}

unsafe fn read_index_reference(fp: *mut BGZF, idx: *mut hts_idx_t, tid: i32) -> Option<()> {
    let n_bin = bgzf_read_u32(fp)?;
    if n_bin > i32::MAX as u32 {
        return None;
    }
    let bidx = alloc_bidx(n_bin)?;
    *(*idx).bidx.add(tid as usize) = bidx;
    for _ in 0..n_bin {
        let bin = bgzf_read_u32(fp)?;
        let k = insert_bidx_bin(bidx, bin)?;
        let val = (*bidx).vals.add(k as usize);
        if (*idx).fmt == HTS_FMT_CSI {
            (*val).loff = bgzf_read_u64(fp)?;
        }
        let n_chunk = bgzf_read_u32(fp)?;
        if n_chunk > i32::MAX as u32 {
            return None;
        }
        (*val).n = n_chunk as i32;
        (*val).m = n_chunk as i32;
        if n_chunk > 0 {
            let bytes = (n_chunk as usize).checked_mul(std::mem::size_of::<hts_pair64_t>())?;
            let mut list: Vec<hts_pair64_t> =
                (0..n_chunk).map(|_| std::mem::zeroed()).collect();
            (*val).list = list.as_mut_ptr();
            std::mem::forget(list);
            if !bgzf_read_exact(fp, (*val).list.cast(), bytes) {
                return None;
            }
        }
    }
    if (*idx).fmt == HTS_FMT_CSI {
        return Some(());
    }
    let n_intv = bgzf_read_u32(fp)?;
    if n_intv > i32::MAX as u32 {
        return None;
    }
    let lidx = (*idx).lidx.add(tid as usize);
    (*lidx).n = n_intv as hts_pos_t;
    (*lidx).m = n_intv as hts_pos_t;
    if n_intv > 0 {
        let bytes = (n_intv as usize).checked_mul(std::mem::size_of::<u64>())?;
        let mut offset: Vec<u64> = vec![0u64; n_intv as usize];
        (*lidx).offset = offset.as_mut_ptr();
        std::mem::forget(offset);
        if !bgzf_read_exact(fp, (*lidx).offset.cast(), bytes) {
            return None;
        }
        let mut k = 0usize;
        while k < n_intv as usize && *(*lidx).offset.add(k) == 0 {
            k += 1;
        }
        let mut j = n_intv as isize - 1;
        while j > k as isize {
            if *(*lidx).offset.add((j - 1) as usize) == 0 {
                *(*lidx).offset.add((j - 1) as usize) = *(*lidx).offset.add(j as usize);
            }
            j -= 1;
        }
    }
    update_bai_loff(idx, tid);
    Some(())
}

unsafe fn alloc_bidx(n_bin: u32) -> Option<*mut hts_idx_bidx_t> {
    if n_bin > i32::MAX as u32 {
        return None;
    }
    let bidx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_bidx_t>()));
    if n_bin == 0 {
        return Some(bidx);
    }
    let mut n_buckets = 2u32;
    while n_buckets < n_bin.saturating_mul(2) {
        n_buckets <<= 1;
    }
    (*bidx).n_buckets = n_buckets;
    (*bidx).size = 0;
    (*bidx).n_occupied = 0;
    (*bidx).upper_bound = (n_buckets as f64 * 0.77) as u32;
    let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
    // khash flag/key/val arrays as owned Vecs handed to the still-raw fields.
    let flags: Vec<u32> = vec![0xaaaa_aaaau32; n_flags as usize];
    let keys: Vec<u32> = vec![0u32; n_buckets as usize];
    let vals: Vec<hts_idx_bins_t> = (0..n_buckets).map(|_| std::mem::zeroed()).collect();
    (*bidx).flags = Box::into_raw(flags.into_boxed_slice()).cast::<u32>();
    (*bidx).keys = Box::into_raw(keys.into_boxed_slice()).cast::<u32>();
    (*bidx).vals = Box::into_raw(vals.into_boxed_slice()).cast::<hts_idx_bins_t>();
    Some(bidx)
}

unsafe fn kh_resize_bin(bidx: *mut hts_idx_bidx_t, mut new_n_buckets: u32) -> Option<()> {
    if new_n_buckets == 0 {
        new_n_buckets = 1;
    }
    new_n_buckets = new_n_buckets.next_power_of_two().max(4);
    if (*bidx).size >= ((new_n_buckets as f64 * 0.77) + 0.5) as u32 {
        return None;
    }

    let n_flags = if new_n_buckets < 16 {
        1
    } else {
        new_n_buckets >> 4
    };
    // Fresh flags array (owned), filled with the khash "empty" pattern.
    let new_flags_vec: Vec<u32> = vec![0xaaaa_aaaau32; n_flags as usize];
    let new_flags = Box::into_raw(new_flags_vec.into_boxed_slice()).cast::<u32>();

    let old_n_buckets = (*bidx).n_buckets;
    if old_n_buckets < new_n_buckets {
        // Grow keys array, preserving existing entries.
        let mut keys: Vec<u32> = if (*bidx).keys.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*bidx).keys, old_n_buckets as usize, old_n_buckets as usize)
        };
        keys.resize(new_n_buckets as usize, 0);
        (*bidx).keys = Box::into_raw(keys.into_boxed_slice()).cast::<u32>();

        // Grow vals array, preserving existing entries.
        let mut vals: Vec<hts_idx_bins_t> = if (*bidx).vals.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*bidx).vals, old_n_buckets as usize, old_n_buckets as usize)
        };
        while vals.len() < new_n_buckets as usize {
            vals.push(std::mem::zeroed());
        }
        (*bidx).vals = Box::into_raw(vals.into_boxed_slice()).cast::<hts_idx_bins_t>();
    }

    for j in 0..(*bidx).n_buckets {
        if kh_exist((*bidx).flags, j) {
            let mut key = *(*bidx).keys.add(j as usize);
            let mut val = std::ptr::read((*bidx).vals.add(j as usize));
            let new_mask = new_n_buckets - 1;
            kh_set_isdel_true((*bidx).flags, j);
            loop {
                let mut i = key & new_mask;
                let mut step = 0;
                while !kh_isempty(new_flags, i) {
                    step += 1;
                    i = (i + step) & new_mask;
                }
                kh_set_isempty_false(new_flags, i);
                if i < (*bidx).n_buckets && kh_iseither((*bidx).flags, i) == 0 {
                    std::ptr::swap(&mut key, (*bidx).keys.add(i as usize));
                    std::ptr::swap(&mut val, (*bidx).vals.add(i as usize));
                    kh_set_isdel_true((*bidx).flags, i);
                } else {
                    *(*bidx).keys.add(i as usize) = key;
                    std::ptr::write((*bidx).vals.add(i as usize), val);
                    break;
                }
            }
        }
    }

    if !(*bidx).flags.is_null() {
        let old_n_flags = if old_n_buckets < 16 { 1 } else { old_n_buckets >> 4 };
        drop(Box::from_raw(std::slice::from_raw_parts_mut(
            (*bidx).flags,
            old_n_flags as usize,
        )));
    }
    (*bidx).flags = new_flags;
    (*bidx).n_buckets = new_n_buckets;
    (*bidx).n_occupied = (*bidx).size;
    (*bidx).upper_bound = ((*bidx).n_buckets as f64 * 0.77 + 0.5) as u32;
    Some(())
}

unsafe fn insert_bidx_bin(bidx: *mut hts_idx_bidx_t, bin: u32) -> Option<u32> {
    if (*bidx).n_occupied >= (*bidx).upper_bound {
        let new_n_buckets = if (*bidx).n_buckets > ((*bidx).size << 1) {
            (*bidx).n_buckets - 1
        } else {
            (*bidx).n_buckets + 1
        };
        kh_resize_bin(bidx, new_n_buckets)?;
    }
    if (*bidx).n_buckets == 0 {
        return None;
    }
    let mask = (*bidx).n_buckets - 1;
    let mut x = (*bidx).n_buckets;
    let mut site = (*bidx).n_buckets;
    let mut k = bin & mask;
    let mut step = 0;
    if kh_isempty((*bidx).flags, k) {
        x = k;
    } else {
        let last = k;
        while !kh_isempty((*bidx).flags, k)
            && (kh_isdel((*bidx).flags, k) || *(*bidx).keys.add(k as usize) != bin)
        {
            if kh_isdel((*bidx).flags, k) {
                site = k;
            }
            step += 1;
            k = (k + step) & mask;
            if k == last {
                x = site;
                break;
            }
        }
        if x == (*bidx).n_buckets {
            if kh_isempty((*bidx).flags, k) && site != (*bidx).n_buckets {
                x = site;
            } else {
                x = k;
            }
        }
    }

    if kh_isempty((*bidx).flags, x) {
        *(*bidx).keys.add(x as usize) = bin;
        kh_set_isboth_false((*bidx).flags, x);
        (*bidx).size += 1;
        (*bidx).n_occupied += 1;
        Some(x)
    } else if kh_isdel((*bidx).flags, x) {
        *(*bidx).keys.add(x as usize) = bin;
        kh_set_isboth_false((*bidx).flags, x);
        (*bidx).size += 1;
        Some(x)
    } else {
        None
    }
}

unsafe fn kh_del_bin(h: *mut hts_idx_bidx_t, k: u32) {
    if k == (*h).n_buckets || !kh_exist((*h).flags, k) {
        return;
    }
    let flag = (*h).flags.add((k >> 4) as usize);
    *flag |= 1 << ((k & 0x0f) << 1);
    (*h).size -= 1;
}

unsafe fn update_bai_loff(idx: *mut hts_idx_t, tid: i32) {
    let bidx = *(*idx).bidx.add(tid as usize);
    if bidx.is_null() {
        return;
    }
    let lidx = (*idx).lidx.add(tid as usize);
    for k in 0..(*bidx).n_buckets {
        if !kh_exist((*bidx).flags, k) {
            continue;
        }
        let key = *(*bidx).keys.add(k as usize);
        let val = (*bidx).vals.add(k as usize);
        if key < (*idx).n_bins as u32 {
            let bot_bin = hts_bin_bot(key as i32, (*idx).n_lvls);
            (*val).loff = if bot_bin >= 0 && (bot_bin as hts_pos_t) < (*lidx).n {
                *(*lidx).offset.add(bot_bin as usize)
            } else {
                0
            };
        } else {
            (*val).loff = 0;
        }
    }
}

unsafe fn kh_exist(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3) == 0
}

unsafe fn kh_set_isdel_true(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) |= 1u32 << ((i & 0x0f) << 1);
}

unsafe fn kh_set_isempty_false(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) &= !(2u32 << ((i & 0x0f) << 1));
}

unsafe fn kh_set_isboth_false(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) &= !(3u32 << ((i & 0x0f) << 1));
}

unsafe fn kh_isempty(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
}

unsafe fn kh_isdel(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
}

unsafe fn kh_iseither(flags: *const u32, i: u32) -> u32 {
    (*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3
}

unsafe fn kh_get_bin(h: *const hts_idx_bidx_t, key: u32) -> u32 {
    if h.is_null() {
        return 0;
    }
    if (*h).n_buckets != 0 {
        let mask = (*h).n_buckets - 1;
        let mut i = key & mask;
        let last = i;
        let mut step = 0;
        while !kh_isempty((*h).flags, i)
            && (kh_isdel((*h).flags, i) || *(*h).keys.add(i as usize) != key)
        {
            step += 1;
            i = (i + step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        if kh_iseither((*h).flags, i) != 0 {
            (*h).n_buckets
        } else {
            i
        }
    } else {
        0
    }
}

fn meta_bin(idx: *const hts_idx_t) -> u32 {
    unsafe { ((*idx).n_bins + 1) as u32 }
}

pub unsafe fn hts_itr_destroy(iter: *mut hts_itr_t) {
    if !iter.is_null() {
        if ((*iter).bitfields & (1 << 4)) != 0 {
            hts_reglist_free((*iter).reg_list, (*iter).n_reg);
        } else if !(*iter).bins.a.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*iter).bins.a,
                (*iter).bins.m as usize,
            )));
        }
        if !(*iter).off.is_null() {
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                (*iter).off,
                (*iter).n_off as usize,
            )));
        }
        drop(Box::from_raw(iter));
    }
}

pub unsafe fn hts_itr_next(
    fp: *mut BGZF,
    iter: *mut hts_itr_t,
    r: *mut (),
    data: *mut (),
) -> i32 {
    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;
    if iter.is_null() || itr_finished(iter) {
        return -1;
    }
    if itr_read_rest(iter) {
        if (*iter).curr_off != 0 {
            if bgzf_seek(fp, (*iter).curr_off as i64, 0) < 0 {
                return -2;
            }
            (*iter).curr_off = 0;
        }
        let ret = if let Some(readrec) = (*iter).readrec {
            readrec(fp, data.cast(), r.cast(), &mut tid, &mut beg, &mut end)
        } else {
            -1
        };
        if ret < 0 {
            itr_set_finished(iter);
        }
        (*iter).curr_tid = tid;
        (*iter).curr_beg = beg;
        (*iter).curr_end = end;
        return ret;
    }

    loop {
        let off = (*iter).off.cast::<hts_pair64_t>();
        if (*iter).curr_off == 0 || (*iter).curr_off >= (*off.add((*iter).i as usize)).v {
            if (*iter).i == (*iter).n_off - 1 {
                itr_set_finished(iter);
                return -1;
            }
            if (*iter).i < 0
                || (*off.add((*iter).i as usize)).v != (*off.add(((*iter).i + 1) as usize)).u
            {
                if bgzf_seek(fp, (*off.add(((*iter).i + 1) as usize)).u as i64, 0) < 0 {
                    return -2;
                }
                (*iter).curr_off = bgzf_tell(fp);
            }
            (*iter).i += 1;
        }
        let ret = if let Some(readrec) = (*iter).readrec {
            readrec(fp, data.cast(), r.cast(), &mut tid, &mut beg, &mut end)
        } else {
            -1
        };
        if ret >= 0 {
            (*iter).curr_off = bgzf_tell(fp);
            if tid != (*iter).tid || beg >= (*iter).end {
                itr_set_finished(iter);
                return -1;
            }
            if end > (*iter).beg && (*iter).end > beg {
                (*iter).curr_tid = tid;
                (*iter).curr_beg = beg;
                (*iter).curr_end = end;
                return ret;
            }
        } else {
            itr_set_finished(iter);
            return ret;
        }
    }
}

unsafe fn bgzf_tell(fp: *const BGZF) -> u64 {
    (((*fp).block_address as u64) << 16) | ((*fp).block_offset as u64 & 0xffff)
}

unsafe fn itr_read_rest(iter: *const hts_itr_t) -> bool {
    ((*iter).bitfields & 1) != 0
}

unsafe fn itr_finished(iter: *const hts_itr_t) -> bool {
    ((*iter).bitfields & (1 << 1)) != 0
}

unsafe fn itr_is_cram(iter: *const hts_itr_t) -> bool {
    ((*iter).bitfields & (1 << 2)) != 0
}

unsafe fn itr_nocoor(iter: *const hts_itr_t) -> bool {
    ((*iter).bitfields & (1 << 3)) != 0
}

unsafe fn itr_set_finished(iter: *mut hts_itr_t) {
    (*iter).bitfields |= 1 << 1;
}

pub unsafe fn hts_c_3221_add_to_interval(
    iter: *mut hts_itr_t,
    bin: *mut hts_idx_bins_t,
    tid: i32,
    interval: u32,
    min_off: u64,
    max_off: u64,
) -> i32 {
    if (*bin).n == 0 {
        return 0;
    }
    let old_n = (*iter).n_off as usize;
    let new_n = ((*iter).n_off + (*bin).n) as usize;
    let mut off_vec: Vec<hts_pair64_max_t> = if (*iter).off.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*iter).off, old_n, old_n)
    };
    while off_vec.len() < new_n {
        off_vec.push(std::mem::zeroed());
    }
    (*iter).off = Box::into_raw(off_vec.into_boxed_slice()).cast::<hts_pair64_max_t>();
    let mut j = 0;
    while j < (*bin).n {
        let chunk = *(*bin).list.add(j as usize);
        if chunk.v > min_off && chunk.u < max_off {
            (*(*iter).off.add((*iter).n_off as usize)).u =
                if min_off > chunk.u { min_off } else { chunk.u };
            (*(*iter).off.add((*iter).n_off as usize)).v =
                if max_off < chunk.v { max_off } else { chunk.v };
            (*(*iter).off.add((*iter).n_off as usize)).max = ((tid as u64) << 32) | interval as u64;
            (*iter).n_off += 1;
        }
        j += 1;
    }
    0
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn hts_c_3252_reg2intervals_narrow(
    iter: *mut hts_itr_t,
    bidx: *const hts_idx_bidx_t,
    tid: i32,
    beg: i64,
    mut end: i64,
    interval: u32,
    min_off: u64,
    max_off: u64,
    min_shift: i32,
    n_lvls: i32,
) -> i32 {
    let mut s = min_shift + n_lvls * 3;
    let mut t = 0i32;
    end -= 1;
    for l in 0..=n_lvls {
        let b = t as hts_pos_t + (beg >> s);
        let e = t as hts_pos_t + (end >> s);
        let mut i = b;
        while i <= e {
            let k = kh_get_bin(bidx, i as u32);
            if k != (*bidx).n_buckets {
                let bin = (*bidx).vals.add(k as usize);
                let res = hts_c_3221_add_to_interval(iter, bin, tid, interval, min_off, max_off);
                if res < 0 {
                    return res;
                }
            }
            i += 1;
        }
        s -= 3;
        t += 1 << ((l << 1) + l);
    }
    0
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn hts_c_3276_reg2intervals_wide(
    iter: *mut hts_itr_t,
    bidx: *const hts_idx_bidx_t,
    tid: i32,
    mut beg: i64,
    mut end: i64,
    interval: u32,
    min_off: u64,
    max_off: u64,
    min_shift: i32,
    n_lvls: i32,
) -> i32 {
    let max_shift = 3 * n_lvls + min_shift;
    end -= 1;
    if beg < 0 {
        beg = 0;
    }
    let mut i = 0;
    while i < (*bidx).n_buckets {
        if kh_exist((*bidx).flags, i) {
            let bin_key = *(*bidx).keys.add(i as usize) as hts_pos_t;
            let level = hts_bin_level(bin_key as i32);
            if level <= n_lvls {
                let first = hts_bin_first(level) as hts_pos_t;
                let beg_at_level = first + (beg >> (max_shift - 3 * level));
                let end_at_level = first + (end >> (max_shift - 3 * level));
                if beg_at_level <= bin_key && bin_key <= end_at_level {
                    let bin = (*bidx).vals.add(i as usize);
                    let res =
                        hts_c_3221_add_to_interval(iter, bin, tid, interval, min_off, max_off);
                    if res < 0 {
                        return res;
                    }
                }
            }
        }
        i += 1;
    }
    0
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn hts_c_3304_reg2intervals(
    iter: *mut hts_itr_t,
    idx: *const hts_idx_t,
    tid: i32,
    beg: i64,
    mut end: i64,
    interval: u32,
    min_off: u64,
    max_off: u64,
    min_shift: i32,
    n_lvls: i32,
) -> i32 {
    if iter.is_null() || idx.is_null() || beg > end {
        return -1;
    }
    let bidx = *(*idx).bidx.add(tid as usize);
    if bidx.is_null() {
        return -1;
    }
    let mut s = min_shift + (n_lvls << 1) + n_lvls;
    if end >= 1_i64 << s {
        end = 1_i64 << s;
    }
    let end1 = end - 1;
    let mut reg_bin_count = 0usize;
    for _l in 0..=n_lvls {
        reg_bin_count += ((end1 >> s) - (beg >> s) + 1) as usize;
        s -= 3;
    }
    let start_n_off = (*iter).n_off;
    let res = if reg_bin_count < (*bidx).n_buckets as usize {
        hts_c_3252_reg2intervals_narrow(
            iter, bidx, tid, beg, end, interval, min_off, max_off, min_shift, n_lvls,
        )
    } else {
        hts_c_3276_reg2intervals_wide(
            iter, bidx, tid, beg, end, interval, min_off, max_off, min_shift, n_lvls,
        )
    };
    if res < 0 {
        return res;
    }

    if (*iter).n_off - start_n_off > 1 {
        let count = ((*iter).n_off - start_n_off) as usize;
        let off = std::slice::from_raw_parts_mut((*iter).off.add(start_n_off as usize), count);
        off.sort_by(|a, b| a.u.cmp(&b.u).then_with(|| a.max.cmp(&b.max)));

        let mut i = start_n_off;
        let mut j = start_n_off + 1;
        while j < (*iter).n_off {
            if (*(*iter).off.add(i as usize)).v >= (*(*iter).off.add(j as usize)).u {
                if (*(*iter).off.add(i as usize)).v < (*(*iter).off.add(j as usize)).v {
                    (*(*iter).off.add(i as usize)).v = (*(*iter).off.add(j as usize)).v;
                }
            } else {
                i += 1;
                if i < j {
                    *(*iter).off.add(i as usize) = *(*iter).off.add(j as usize);
                }
            }
            j += 1;
        }
        (*iter).n_off = i + 1;
    }

    (*iter).n_off
}

unsafe fn reg2bins_narrow(
    beg: hts_pos_t,
    mut end: hts_pos_t,
    itr: *mut hts_itr_t,
    min_shift: i32,
    n_lvls: i32,
    bidx: *const hts_idx_bidx_t,
) -> i32 {
    end -= 1;
    let mut s = min_shift + n_lvls * 3;
    let mut t = 0i32;
    for l in 0..=n_lvls {
        let b = t as hts_pos_t + (beg >> s);
        let e = t as hts_pos_t + (end >> s);
        let mut i = b;
        while i <= e {
            if kh_get_bin(bidx, i as u32) != (*bidx).n_buckets {
                *(*itr).bins.a.add((*itr).bins.n as usize) = i as i32;
                (*itr).bins.n += 1;
            }
            i += 1;
        }
        s -= 3;
        t += 1 << ((l << 1) + l);
    }
    (*itr).bins.n
}

unsafe fn reg2bins_wide(
    beg: hts_pos_t,
    mut end: hts_pos_t,
    itr: *mut hts_itr_t,
    min_shift: i32,
    n_lvls: i32,
    bidx: *const hts_idx_bidx_t,
) -> i32 {
    end -= 1;
    let beg = beg.max(0);
    let max_shift = 3 * n_lvls + min_shift;
    for i in 0..(*bidx).n_buckets {
        if !kh_exist((*bidx).flags, i) {
            continue;
        }
        let bin = *(*bidx).keys.add(i as usize) as hts_pos_t;
        let level = hts_bin_level(bin as i32);
        if level > n_lvls {
            continue;
        }
        let first = hts_bin_first(level) as hts_pos_t;
        let beg_at_level = first + (beg >> (max_shift - 3 * level));
        let end_at_level = first + (end >> (max_shift - 3 * level));
        if beg_at_level <= bin && bin <= end_at_level {
            *(*itr).bins.a.add((*itr).bins.n as usize) = bin as i32;
            (*itr).bins.n += 1;
        }
    }
    (*itr).bins.n
}

unsafe fn reg2bins(
    beg: hts_pos_t,
    mut end: hts_pos_t,
    itr: *mut hts_itr_t,
    min_shift: i32,
    n_lvls: i32,
    bidx: *const hts_idx_bidx_t,
) -> i32 {
    let mut s = min_shift + n_lvls * 3;
    if end >= 1_i64 << s {
        end = 1_i64 << s;
    }
    if beg >= end {
        return 0;
    }
    let end1 = end - 1;
    let mut reg_bin_count = 0usize;
    for _ in 0..=n_lvls {
        reg_bin_count += ((end1 >> s) - (beg >> s) + 1) as usize;
        s -= 3;
    }
    let hash_bin_count = (*bidx).n_buckets as usize;
    let max_bins = reg_bin_count.min((*bidx).size as usize);
    if ((*itr).bins.m - (*itr).bins.n) < max_bins as i32 {
        if max_bins > i32::MAX as usize {
            *libc::__errno_location() =
                libc::ENOMEM as i32;
            return -1;
        }
        let new_m = max_bins + (*itr).bins.n as usize;
        let old_m = (*itr).bins.m as usize;
        let mut a_vec: Vec<i32> = if (*itr).bins.a.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*itr).bins.a, old_m, old_m)
        };
        a_vec.resize(new_m, 0);
        (*itr).bins.a = Box::into_raw(a_vec.into_boxed_slice()).cast::<i32>();
        (*itr).bins.m = new_m as i32;
    }
    if reg_bin_count < hash_bin_count {
        reg2bins_narrow(beg, end, itr, min_shift, n_lvls, bidx)
    } else {
        reg2bins_wide(beg, end, itr, min_shift, n_lvls, bidx)
    }
}

unsafe fn hts_itr_off(idx: *const hts_idx_t, tid: i32) -> u64 {
    match tid {
        HTS_IDX_START => {
            if idx.is_null() {
                return u64::MAX;
            }
            let mut off0 = u64::MAX;
            for i in 0..(*idx).n {
                let bidx = *(*idx).bidx.add(i as usize);
                if bidx.is_null() {
                    continue;
                }
                let k = kh_get_bin(bidx, meta_bin(idx));
                if k == (*bidx).n_buckets {
                    continue;
                }
                let off = (*(*(*bidx).vals.add(k as usize)).list).u;
                if off0 > off {
                    off0 = off;
                }
            }
            if off0 == u64::MAX && (*idx).n_no_coor != 0 {
                0
            } else {
                off0
            }
        }
        HTS_IDX_NOCOOR => {
            if idx.is_null() {
                return u64::MAX;
            }
            let mut off0 = u64::MAX;
            for i in 0..(*idx).n {
                let bidx = *(*idx).bidx.add(i as usize);
                if bidx.is_null() {
                    continue;
                }
                let k = kh_get_bin(bidx, meta_bin(idx));
                if k != (*bidx).n_buckets {
                    let off = (*(*(*bidx).vals.add(k as usize)).list).v;
                    if off0 == u64::MAX || off0 < off {
                        off0 = off;
                    }
                }
            }
            if off0 == u64::MAX && (*idx).n_no_coor != 0 {
                0
            } else {
                off0
            }
        }
        HTS_IDX_REST | HTS_IDX_NONE => 0,
        _ => u64::MAX,
    }
}

pub unsafe fn hts_itr_query(
    idx: *const hts_idx_t,
    tid: i32,
    beg: hts_pos_t,
    end: hts_pos_t,
    readrec: hts_readrec_func,
) -> *mut hts_itr_t {
    if idx.is_null() && !(tid == HTS_IDX_REST || tid == HTS_IDX_NONE) {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return std::ptr::null_mut();
    }

    let iter = Box::into_raw(Box::new(std::mem::zeroed::<hts_itr_t>()));

    if tid < 0 {
        let off = hts_itr_off(idx, tid);
        if off == u64::MAX {
            drop(Box::from_raw(iter));
            return std::ptr::null_mut();
        }
        (*iter).bitfields |= 1;
        (*iter).curr_off = off;
        (*iter).readrec = readrec;
        if tid == HTS_IDX_NONE {
            itr_set_finished(iter);
        }
        return iter;
    }

    if tid >= (*idx).n || (*(*idx).bidx.add(tid as usize)).is_null() {
        itr_set_finished(iter);
        return iter;
    }

    let bidx = *(*idx).bidx.add(tid as usize);
    let beg = beg.max(0);
    if end < beg {
        drop(Box::from_raw(iter));
        return std::ptr::null_mut();
    }

    let meta_k = kh_get_bin(bidx, meta_bin(idx));
    let unmapped = if meta_k != (*bidx).n_buckets {
        (*(*bidx).vals.add(meta_k as usize)).list.add(1).read().v as u32
    } else {
        1
    };

    (*iter).tid = tid;
    (*iter).beg = beg;
    (*iter).end = end;
    (*iter).i = -1;
    (*iter).readrec = readrec;

    if (*bidx).size == 0 {
        itr_set_finished(iter);
        return iter;
    }

    let idx_maxpos = hts_bin_maxpos((*idx).min_shift, (*idx).n_lvls);
    if beg >= idx_maxpos {
        itr_set_finished(iter);
        return iter;
    }

    let rel_off = (beg >> (*idx).min_shift) as u32;
    let mut bin = (hts_bin_first((*idx).n_lvls) as u32).wrapping_add(rel_off);
    let mut k;
    loop {
        k = kh_get_bin(bidx, bin);
        if k != (*bidx).n_buckets {
            break;
        }
        let first = ((hts_bin_parent(bin as i32) << 3) + 1) as u32;
        if bin > first {
            bin -= 1;
        } else {
            bin = hts_bin_parent(bin as i32) as u32;
        }
        if bin == 0 {
            break;
        }
    }
    if bin == 0 {
        k = kh_get_bin(bidx, 0);
    }
    let mut min_off = if k != (*bidx).n_buckets {
        (*(*bidx).vals.add(k as usize)).loff
    } else {
        0
    };

    let lidx = (*idx).lidx.add(tid as usize);
    if !(*lidx).offset.is_null() && (rel_off as hts_pos_t) < (*lidx).n {
        let lin = *(*lidx).offset.add(rel_off as usize);
        if min_off < lin {
            min_off = lin;
        }
        if unmapped != 0 {
            let mut tmp_off = rel_off as i32 - 1;
            while tmp_off >= 0 {
                let off = *(*lidx).offset.add(tmp_off as usize);
                if off < min_off {
                    min_off = off;
                    break;
                }
                tmp_off -= 1;
            }
            if k != (*bidx).n_buckets
                && (min_off < (*(*bidx).vals.add(k as usize)).loff || tmp_off < 0)
            {
                min_off = (*(*bidx).vals.add(k as usize)).loff;
            }
        }
    } else if unmapped != 0 && k != (*bidx).n_buckets {
        min_off = (*(*bidx).vals.add(k as usize)).loff;
    }

    let max_off = if end <= idx_maxpos {
        let mut bin = (hts_bin_first((*idx).n_lvls) as hts_pos_t
            + ((end - 1) >> (*idx).min_shift)
            + 1) as u32;
        if bin >= (*idx).n_bins as u32 {
            bin = 0;
        }
        loop {
            while bin % 8 == 1 {
                bin = hts_bin_parent(bin as i32) as u32;
            }
            if bin == 0 {
                break u64::MAX;
            }
            let k = kh_get_bin(bidx, bin);
            if k != (*bidx).n_buckets && (*(*bidx).vals.add(k as usize)).n > 0 {
                break (*(*(*bidx).vals.add(k as usize)).list).u;
            }
            bin = bin.wrapping_add(1);
        }
    } else {
        u64::MAX
    };

    if reg2bins(beg, end, iter, (*idx).min_shift, (*idx).n_lvls, bidx) < 0 {
        hts_itr_destroy(iter);
        return std::ptr::null_mut();
    }

    let mut n_off = 0usize;
    for i in 0..(*iter).bins.n {
        let k = kh_get_bin(bidx, *(*iter).bins.a.add(i as usize) as u32);
        if k != (*bidx).n_buckets {
            n_off += (*(*bidx).vals.add(k as usize)).n as usize;
        }
    }
    if n_off == 0 {
        itr_set_finished(iter);
        return iter;
    }

    let mut off_vec: Vec<hts_pair64_max_t> =
        (0..n_off).map(|_| std::mem::zeroed()).collect();
    let off = off_vec.as_mut_ptr();

    n_off = 0;
    for i in 0..(*iter).bins.n {
        let k = kh_get_bin(bidx, *(*iter).bins.a.add(i as usize) as u32);
        if k == (*bidx).n_buckets {
            continue;
        }
        let p = (*bidx).vals.add(k as usize);
        for j in 0..(*p).n {
            let chunk = *(*p).list.add(j as usize);
            if chunk.v > min_off && chunk.u < max_off {
                (*off.add(n_off)).u = min_off.max(chunk.u);
                (*off.add(n_off)).v = max_off.min(chunk.v);
                (*off.add(n_off)).max = ((tid as u64) << 32) | j as u64;
                n_off += 1;
            }
        }
    }

    if n_off == 0 {
        drop(off_vec);
        itr_set_finished(iter);
        return iter;
    }

    let off_slice = std::slice::from_raw_parts_mut(off, n_off);
    off_slice.sort_by(|a, b| a.u.cmp(&b.u).then_with(|| a.max.cmp(&b.max)));

    let mut l = 0usize;
    for i in 1..n_off {
        if off_slice[l].v < off_slice[i].v {
            l += 1;
            off_slice[l] = off_slice[i];
        }
    }
    n_off = l + 1;

    for i in 1..n_off {
        if off_slice[i - 1].v >= off_slice[i].u {
            off_slice[i - 1].v = off_slice[i].u;
        }
    }

    l = 0;
    for i in 1..n_off {
        if off_slice[l].v >> 16 == off_slice[i].u >> 16 {
            off_slice[l].v = off_slice[i].v;
        } else {
            l += 1;
            off_slice[l] = off_slice[i];
        }
    }
    n_off = l + 1;
    (*iter).n_off = n_off as i32;
    // Shrink to exactly `n_off` so the buffer's capacity matches what
    // `hts_itr_destroy` reconstructs, then hand it to the raw field.
    off_vec.truncate(n_off);
    (*iter).off = Box::into_raw(off_vec.into_boxed_slice()).cast::<hts_pair64_max_t>();
    iter
}

pub unsafe fn hts_itr_multi_next(fd: *mut htsFile, iter: *mut hts_itr_t, r: *mut ()) -> i32 {
    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;
    if iter.is_null() || itr_finished(iter) {
        return -1;
    }
    let fp = if itr_is_cram(iter) {
        (*fd).fp.cram.cast::<c_void>()
    } else {
        (*fd).fp.bgzf.cast::<c_void>()
    };
    if itr_read_rest(iter) {
        if (*iter).curr_off != 0 {
            if let Some(seek) = (*iter).seek {
                if seek(fp, (*iter).curr_off as i64, 0) < 0 {
                    return -2;
                }
            } else {
                return -2;
            }
            (*iter).curr_off = 0;
        }
        let ret = if let Some(readrec) = (*iter).readrec {
            readrec(
                fp.cast::<BGZF>(),
                fd.cast(),
                r.cast(),
                &mut tid,
                &mut beg,
                &mut end,
            )
        } else {
            -1
        };
        if ret < 0 {
            itr_set_finished(iter);
        }
        (*iter).curr_tid = tid;
        (*iter).curr_beg = beg;
        (*iter).curr_end = end;
        return ret;
    }
    let mut ret;
    let mut next_range = false;

    loop {
        if next_range
            || (*iter).curr_off == 0
            || (*iter).i >= (*iter).n_off
            || (*iter).curr_off >= (*(*iter).off.add((*iter).i as usize)).v
            || ((*(*iter).off.add((*iter).i as usize)).max >> 32 == (*iter).curr_tid as u64
                && ((*(*iter).off.add((*iter).i as usize)).max & 0xffff_ffff)
                    < (*iter).curr_intv as u64)
        {
            loop {
                (*iter).i += 1;
                if (*iter).i >= (*iter).n_off {
                    break;
                }
                let off = *(*iter).off.add((*iter).i as usize);
                if !((*iter).curr_off >= off.v
                    || (off.max >> 32 == (*iter).curr_tid as u64
                        && (off.max & 0xffff_ffff) < (*iter).curr_intv as u64))
                {
                    break;
                }
            }

            if (*iter).i >= (*iter).n_off {
                if itr_nocoor(iter) {
                    if let Some(seek) = (*iter).seek {
                        if seek(fp, (*iter).nocoor_off as i64, 0) < 0 {
                            return -2;
                        }
                    } else {
                        return -2;
                    }
                    loop {
                        ret = if let Some(readrec) = (*iter).readrec {
                            readrec(
                                fp.cast::<BGZF>(),
                                fd.cast(),
                                r.cast(),
                                &mut tid,
                                &mut beg,
                                &mut end,
                            )
                        } else {
                            -1
                        };
                        if !(tid >= 0 && ret >= 0) {
                            break;
                        }
                    }
                    if ret < 0 {
                        itr_set_finished(iter);
                    } else {
                        (*iter).bitfields |= 1;
                    }
                    (*iter).curr_off = 0;
                    (*iter).curr_tid = tid;
                    (*iter).curr_beg = beg;
                    (*iter).curr_end = end;
                    return ret;
                }
                ret = -1;
                break;
            } else if (*iter).i < (*iter).n_off {
                let off = *(*iter).off.add((*iter).i as usize);
                if (*iter).curr_off < off.u || next_range {
                    (*iter).curr_off = off.u;
                    if let Some(seek) = (*iter).seek {
                        if seek(fp, (*iter).curr_off as i64, 0) < 0 {
                            return -2;
                        }
                    } else {
                        return -2;
                    }
                }
            }
        }

        ret = if let Some(readrec) = (*iter).readrec {
            readrec(
                fp.cast::<BGZF>(),
                fd.cast(),
                r.cast(),
                &mut tid,
                &mut beg,
                &mut end,
            )
        } else {
            -1
        };
        if ret < 0 {
            break;
        }

        (*iter).curr_off = if let Some(tell) = (*iter).tell {
            tell(fp) as u64
        } else {
            0
        };

        if tid != (*iter).curr_tid {
            let mut found = -1;
            for j in 0..(*iter).n_reg {
                if (*(*iter).reg_list.add(j as usize)).tid == tid {
                    found = j;
                    break;
                }
            }
            if found < 0 {
                continue;
            }
            (*iter).curr_reg = found;
            (*iter).curr_tid = tid;
            (*iter).curr_intv = 0;
        }

        let cr = (*iter).curr_reg;
        let ci = (*iter).curr_intv;
        let reg = (*iter).reg_list.add(cr as usize);
        let mut i = ci;
        while i < (*reg).count as i32 {
            let interval = *(*reg).intervals.add(i as usize);
            if end > interval.beg && interval.end > beg {
                (*iter).curr_beg = beg;
                (*iter).curr_end = end;
                (*iter).curr_intv = i;
                return ret;
            }
            if beg > interval.end {
                (*iter).curr_intv = i + 1;
            }
            if end < interval.beg {
                break;
            }
            i += 1;
        }
        next_range = false;
    }

    itr_set_finished(iter);
    ret
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{c_void, CStr, CString},
        mem::{align_of, size_of},
    };

    use super::*;

    #[test]
    fn public_hts_struct_layout_matches_htslib_abi_shape() {
        assert_eq!(size_of::<kstring_t>(), 24);
        assert_eq!(align_of::<kstring_t>(), 8);
        assert_eq!(size_of::<htsFormatVersion>(), 4);
        assert_eq!(align_of::<htsFormatVersion>(), 2);
        assert_eq!(size_of::<htsFormat>(), 32);
        assert_eq!(align_of::<htsFormat>(), 8);
        assert_eq!(size_of::<htsFilePtr>(), 8);
        assert_eq!(align_of::<htsFilePtr>(), 8);
        // BGZF is intentionally no longer a C-ABI mirror: the owned refactor
        // replaced its raw void*/hFILE* fields with Vec/Box/Option/enum members
        // (uncompressed_block, compressed_block, cache, fp: BgzfFp, mt, idx,
        // gz_stream), so it is wider than the old 112-byte C struct. Assert the
        // owned size rather than the obsolete C-ABI size.
        assert_eq!(size_of::<BGZF>(), 144);
        assert_eq!(align_of::<BGZF>(), 8);
        assert_eq!(size_of::<htsFile>(), 136);
        assert_eq!(align_of::<htsFile>(), 8);
        assert_eq!(size_of::<hts_pair64_t>(), 16);
        assert_eq!(align_of::<hts_pair64_t>(), 8);
        assert_eq!(size_of::<hts_pair64_max_t>(), 24);
        assert_eq!(align_of::<hts_pair64_max_t>(), 8);
        assert_eq!(size_of::<hts_pair_pos_t>(), 16);
        assert_eq!(align_of::<hts_pair_pos_t>(), 8);
        assert_eq!(size_of::<hts_reglist_t>(), 40);
        assert_eq!(align_of::<hts_reglist_t>(), 8);
        assert_eq!(size_of::<hts_idx_bins_t>(), 24);
        assert_eq!(align_of::<hts_idx_bins_t>(), 8);
        assert_eq!(size_of::<hts_idx_bidx_t>(), 40);
        assert_eq!(align_of::<hts_idx_bidx_t>(), 8);
        assert_eq!(size_of::<hts_idx_lidx_t>(), 24);
        assert_eq!(align_of::<hts_idx_lidx_t>(), 8);
        assert_eq!(size_of::<hts_idx_z_t>(), 80);
        assert_eq!(align_of::<hts_idx_z_t>(), 8);
        assert_eq!(size_of::<hts_idx_t>(), 160);
        assert_eq!(align_of::<hts_idx_t>(), 8);
        assert_eq!(size_of::<hts_cram_idx_t>(), 16);
        assert_eq!(align_of::<hts_cram_idx_t>(), 8);
        assert_eq!(size_of::<hts_itr_bins_t>(), 16);
        assert_eq!(align_of::<hts_itr_bins_t>(), 8);
        assert_eq!(size_of::<hts_itr_t>(), 144);
        assert_eq!(align_of::<hts_itr_t>(), 8);

        assert_eq!(std::mem::offset_of!(htsFile, lineno), 8);
        assert_eq!(std::mem::offset_of!(htsFile, line), 16);
        assert_eq!(std::mem::offset_of!(htsFile, fn_), 40);
        assert_eq!(std::mem::offset_of!(htsFile, fn_aux), 48);
        assert_eq!(std::mem::offset_of!(htsFile, fp), 56);
        assert_eq!(std::mem::offset_of!(htsFile, state), 64);
        assert_eq!(std::mem::offset_of!(htsFile, format), 72);
        assert_eq!(std::mem::offset_of!(htsFile, idx), 104);
        assert_eq!(std::mem::offset_of!(htsFile, fnidx), 112);
        assert_eq!(std::mem::offset_of!(htsFile, bam_header), 120);
        assert_eq!(std::mem::offset_of!(htsFile, filter), 128);
        // BGZF field-offset asserts are obsolete: BGZF is no longer #[repr(C)]
        // (it is now an owned struct, see size assert above), so the compiler is
        // free to reorder its fields and these C-ABI offsets no longer hold.
        assert_eq!(std::mem::offset_of!(hts_idx_t, fmt), 0);
        assert_eq!(std::mem::offset_of!(hts_idx_t, l_meta), 16);
        assert_eq!(std::mem::offset_of!(hts_idx_t, n_no_coor), 32);
        assert_eq!(std::mem::offset_of!(hts_idx_t, bidx), 40);
        assert_eq!(std::mem::offset_of!(hts_idx_t, lidx), 48);
        assert_eq!(std::mem::offset_of!(hts_idx_t, meta), 56);
        assert_eq!(std::mem::offset_of!(hts_idx_t, tbi_n), 64);
        assert_eq!(std::mem::offset_of!(hts_idx_t, z), 72);
        assert_eq!(std::mem::offset_of!(hts_idx_t, otf_fp), 152);
        assert_eq!(std::mem::offset_of!(hts_cram_idx_t, fmt), 0);
        assert_eq!(std::mem::offset_of!(hts_cram_idx_t, cram), 8);

        assert_eq!(std::mem::offset_of!(hts_itr_t, tid), 4);
        assert_eq!(std::mem::offset_of!(hts_itr_t, n_off), 8);
        assert_eq!(std::mem::offset_of!(hts_itr_t, i), 12);
        assert_eq!(std::mem::offset_of!(hts_itr_t, n_reg), 16);
        assert_eq!(std::mem::offset_of!(hts_itr_t, beg), 24);
        assert_eq!(std::mem::offset_of!(hts_itr_t, end), 32);
        assert_eq!(std::mem::offset_of!(hts_itr_t, reg_list), 40);
        assert_eq!(std::mem::offset_of!(hts_itr_t, curr_tid), 48);
        assert_eq!(std::mem::offset_of!(hts_itr_t, curr_reg), 52);
        assert_eq!(std::mem::offset_of!(hts_itr_t, curr_intv), 56);
        assert_eq!(std::mem::offset_of!(hts_itr_t, curr_beg), 64);
        assert_eq!(std::mem::offset_of!(hts_itr_t, curr_end), 72);
        assert_eq!(std::mem::offset_of!(hts_itr_t, curr_off), 80);
        assert_eq!(std::mem::offset_of!(hts_itr_t, nocoor_off), 88);
        assert_eq!(std::mem::offset_of!(hts_itr_t, off), 96);
        assert_eq!(std::mem::offset_of!(hts_itr_t, readrec), 104);
        assert_eq!(std::mem::offset_of!(hts_itr_t, seek), 112);
        assert_eq!(std::mem::offset_of!(hts_itr_t, tell), 120);
        assert_eq!(std::mem::offset_of!(hts_itr_t, bins), 128);
        assert_eq!(std::mem::offset_of!(hts_itr_bins_t, n), 0);
        assert_eq!(std::mem::offset_of!(hts_itr_bins_t, m), 4);
        assert_eq!(std::mem::offset_of!(hts_itr_bins_t, a), 8);
        assert_eq!(std::mem::offset_of!(hts_pair64_max_t, max), 16);
        assert_eq!(std::mem::offset_of!(hts_pair_pos_t, end), 8);
        assert_eq!(std::mem::offset_of!(hts_reglist_t, reg), 0);
        assert_eq!(std::mem::offset_of!(hts_reglist_t, intervals), 8);
        assert_eq!(std::mem::offset_of!(hts_reglist_t, tid), 16);
        assert_eq!(std::mem::offset_of!(hts_reglist_t, count), 20);
        assert_eq!(std::mem::offset_of!(hts_reglist_t, min_beg), 24);
        assert_eq!(std::mem::offset_of!(hts_reglist_t, max_end), 32);
    }

    #[test]
    fn hts_itr_multi_cram_rejects_invalid_inputs() {
        unsafe {
            let mut iter: hts_itr_t = std::mem::zeroed();
            let mut cidx = hts_cram_idx_t {
                fmt: HTS_FMT_CRAI,
                cram: std::ptr::null_mut(),
            };
            let cidx_ptr = &mut cidx as *mut hts_cram_idx_t as *const hts_idx_t;

            assert_eq!(hts_itr_multi_cram(std::ptr::null(), &mut iter), -1);
            assert_eq!(hts_itr_multi_cram(cidx_ptr, std::ptr::null_mut()), -1);
            assert_eq!(hts_itr_multi_cram(cidx_ptr, &mut iter), -1);
        }
    }

    #[test]
    fn hts_itr_multi_cram_marks_none_region_finished_without_cram_index_lookup() {
        unsafe {
            let mut reg = hts_reglist_t {
                reg: std::ptr::null(),
                intervals: std::ptr::null_mut(),
                tid: HTS_IDX_NONE,
                count: 0,
                min_beg: 0,
                max_end: 0,
            };
            let mut iter: hts_itr_t = std::mem::zeroed();
            let mut offset_marker = std::mem::MaybeUninit::<hts_pair64_max_t>::uninit();
            iter.bitfields = (1 << 4) | 1;
            iter.n_reg = 1;
            iter.reg_list = &mut reg;
            iter.off = offset_marker.as_mut_ptr();
            iter.n_off = 7;
            iter.curr_off = 123;
            iter.i = 9;

            let mut cidx = hts_cram_idx_t {
                fmt: HTS_FMT_CRAI,
                cram: std::ptr::null_mut(),
            };
            let cidx_ptr = &mut cidx as *mut hts_cram_idx_t as *const hts_idx_t;

            assert_eq!(hts_itr_multi_cram(cidx_ptr, &mut iter), 0);
            assert_ne!(iter.bitfields & (1 << 2), 0);
            assert_eq!(iter.bitfields & 1, 0);
            assert_ne!(iter.bitfields & (1 << 1), 0);
            assert_eq!(iter.n_off, 0);
            assert_eq!(iter.off, std::ptr::null_mut());
            assert_eq!(iter.curr_off, 0);
            assert_eq!(iter.i, -1);
        }
    }

    #[test]
    fn hts_metadata_feature_and_format_accessors_match_c_rules() {
        unsafe {
            assert_eq!(
                CStr::from_ptr(hts_version()).to_bytes(),
                b"1.23.1-24-g7c895563"
            );
            assert_eq!(hts_features(), HTS_FEATURE_HTSCODECS);
            assert_eq!(
                CStr::from_ptr(hts_test_feature(HTS_FEATURE_HTSCODECS)).to_bytes(),
                b"builtin"
            );
            assert!(hts_test_feature(HTS_FEATURE_LIBCURL).is_null());
            assert_eq!(
                CStr::from_ptr(hts_test_feature(HTS_FEATURE_CC)).to_bytes(),
                b""
            );
            assert!(CStr::from_ptr(hts_feature_string())
                .to_bytes()
                .starts_with(b"build=Makefile "));

            let mut format = htsFormat {
                category: HTS_FORMAT_SEQUENCE_DATA,
                format: HTS_FORMAT_BAM,
                version: htsFormatVersion {
                    major: -1,
                    minor: -1,
                },
                compression: 2,
                compression_level: -1,
                specific: std::ptr::null_mut(),
            };
            let mut hfile_marker = std::mem::MaybeUninit::<hFILE>::uninit();
            assert_eq!(
                CStr::from_ptr(hts_format_file_extension(&format)).to_bytes(),
                b"bam"
            );
            format.format = HTS_FORMAT_FASTA_FORMAT;
            assert_eq!(
                CStr::from_ptr(hts_format_file_extension(&format)).to_bytes(),
                b"fa"
            );
            format.format = HTS_FORMAT_UNKNOWN_FORMAT;
            assert_eq!(
                CStr::from_ptr(hts_format_file_extension(&format)).to_bytes(),
                b"?"
            );
            assert_eq!(
                CStr::from_ptr(hts_format_file_extension(std::ptr::null())).to_bytes(),
                b"?"
            );

            let mut fp = htsFile {
                bitfields: 0,
                padding_0: 0,
                lineno: 0,
                line: kstring_t::default(),
                fn_: std::ptr::null_mut(),
                fn_aux: std::ptr::null_mut(),
                fp: htsFilePtr {
                    hfile: hfile_marker.as_mut_ptr(),
                },
                state: std::ptr::null_mut(),
                format,
                idx: std::ptr::null_mut(),
                fnidx: std::ptr::null(),
                bam_header: std::ptr::null_mut(),
                filter: std::ptr::null_mut(),
            };
            assert!(hts_get_format(std::ptr::null_mut()).is_null());
            assert_eq!(hts_get_format(&mut fp), std::ptr::addr_of!(fp.format));
            fp.format.format = HTS_FORMAT_SAM;
            fp.format.compression = HTS_COMPRESSION_NO_COMPRESSION;
            assert_eq!(hts_hfile(&mut fp), hfile_marker.as_mut_ptr());
            assert!(hts_get_bgzfp(&mut fp).is_null());
            assert_eq!(hts_flush(std::ptr::null_mut()), 0);
            assert_eq!(hts_flush(&mut fp), 0);

            let mut bgzf = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 5,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 10,
                uncompressed_block: Vec::new(),
                compressed_block: Vec::new(),
                cache: None,
                fp: crate::htslib_rs::bgzf::BgzfFp::None,
                mt: None,
                idx: None,
                idx_build_otf: 0,
                gz_stream: None,
                seeked: 0,
            };
            fp.bitfields = 1 << 4;
            fp.fp.bgzf = &mut bgzf;
            assert!(std::ptr::eq(hts_get_bgzfp(&mut fp), &bgzf));
            assert_eq!(hts_utell(&mut fp), 10);
            assert_eq!(hts_useek(&mut fp, 12, 0), 0);
            assert_eq!(bgzf.uncompressed_address, 12);
            assert_eq!(bgzf.block_offset, 2);
            assert_eq!(hts_utell(&mut fp), 12);
        }
    }

    #[test]
    fn textutils_numeric_conversion_helpers_match_c_rules() {
        unsafe {
            let mut end: *mut u8 = std::ptr::null_mut();
            let mut failed = 0;

            let input = CString::new("127x").unwrap();
            assert_eq!(hts_str2int(input.as_ptr().cast(), &mut end, 8, &mut failed), 127);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("128x").unwrap();
            failed = 0;
            assert_eq!(hts_str2int(input.as_ptr().cast(), &mut end, 8, &mut failed), 127);
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("-128x").unwrap();
            failed = 0;
            assert_eq!(hts_str2int(input.as_ptr().cast(), &mut end, 8, &mut failed), -128);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 4);

            let input = CString::new("-129x").unwrap();
            failed = 0;
            assert_eq!(hts_str2int(input.as_ptr().cast(), &mut end, 8, &mut failed), -128);
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 4);

            let input = CString::new("+12z").unwrap();
            failed = 0;
            assert_eq!(hts_str2int(input.as_ptr().cast(), &mut end, 16, &mut failed), 12);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("255x").unwrap();
            failed = 0;
            assert_eq!(hts_str2uint(input.as_ptr().cast(), &mut end, 8, &mut failed), 255);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("256x").unwrap();
            failed = 0;
            assert_eq!(hts_str2uint(input.as_ptr().cast(), &mut end, 8, &mut failed), 255);
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("+42!").unwrap();
            failed = 0;
            assert_eq!(hts_str2uint(input.as_ptr().cast(), &mut end, 16, &mut failed), 42);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("  -0012.34x").unwrap();
            failed = 0;
            assert_eq!(hts_str2dbl(input.as_ptr().cast(), &mut end, &mut failed), -12.34);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 10);

            let input = CString::new("nan!").unwrap();
            failed = 0;
            assert!(hts_str2dbl(input.as_ptr().cast(), &mut end, &mut failed).is_nan());
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);

            let input = CString::new("abc").unwrap();
            failed = 0;
            assert_eq!(hts_str2dbl(input.as_ptr().cast(), &mut end, &mut failed), 0.0);
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 0);

            let input = CString::new("1e2!").unwrap();
            failed = 0;
            assert_eq!(hts_str2dbl(input.as_ptr().cast(), &mut end, &mut failed), 100.0);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 3);
        }
    }

    #[test]
    fn textutils_numeric_conversion_boundary_edges_match_c_rules() {
        unsafe {
            let mut end: *mut u8 = std::ptr::null_mut();
            let mut failed = 0;

            let input = CString::new("9223372036854775807!").unwrap();
            assert_eq!(
                hts_str2int(input.as_ptr().cast(), &mut end, 64, &mut failed),
                i64::MAX
            );
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 19);

            let input = CString::new("9223372036854775808!").unwrap();
            failed = 0;
            assert_eq!(
                hts_str2int(input.as_ptr().cast(), &mut end, 64, &mut failed),
                i64::MAX
            );
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 19);

            let input = CString::new("-9223372036854775808!").unwrap();
            failed = 0;
            assert_eq!(
                hts_str2int(input.as_ptr().cast(), &mut end, 64, &mut failed),
                i64::MIN
            );
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 20);

            let input = CString::new("-9223372036854775809!").unwrap();
            failed = 0;
            assert_eq!(
                hts_str2int(input.as_ptr().cast(), &mut end, 64, &mut failed),
                i64::MIN
            );
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 20);

            let input = CString::new("18446744073709551615!").unwrap();
            failed = 0;
            assert_eq!(
                hts_str2uint(input.as_ptr().cast(), &mut end, 64, &mut failed),
                u64::MAX
            );
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 20);

            let input = CString::new("18446744073709551616!").unwrap();
            failed = 0;
            assert_eq!(
                hts_str2uint(input.as_ptr().cast(), &mut end, 64, &mut failed),
                u64::MAX
            );
            assert_eq!(failed, 1);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 20);

            let input = CString::new("+x").unwrap();
            failed = 0;
            assert_eq!(hts_str2int(input.as_ptr().cast(), &mut end, 8, &mut failed), 0);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 1);

            let input = CString::new("-x").unwrap();
            failed = 0;
            assert_eq!(hts_str2uint(input.as_ptr().cast(), &mut end, 8, &mut failed), 0);
            assert_eq!(failed, 0);
            assert_eq!(end.offset_from(input.as_ptr().cast()), 0);
        }
    }

    #[test]
    fn textutils_percent_and_base64_helpers_match_c_rules() {
        unsafe {
            assert_eq!(dehex(b'a'), 10);
            assert_eq!(dehex(b'F'), 15);
            assert_eq!(dehex(b'7'), 7);
            assert_eq!(dehex(0), -1);
            assert_eq!(debase64(b'A'), 0);
            assert_eq!(debase64(b'z'), 51);
            assert_eq!(debase64(b'9'), 61);
            assert_eq!(debase64(b'+'), 62);
            assert_eq!(debase64(b'/'), 63);
            assert_eq!(debase64(b'='), -1);

            let input = CString::new("a%20b%2fc%ZZ").unwrap();
            let mut out = [0u8; 32];
            let mut out_len = 0usize;
            assert_eq!(
                hts_decode_percent(&mut out, &mut out_len, input.as_bytes()),
                0
            );
            assert_eq!(out_len, 8);
            assert_eq!(CStr::from_ptr(out.as_ptr().cast()).to_bytes(), b"a b/c%ZZ");

            assert_eq!(hts_base64_decoded_length(0), 0);
            assert_eq!(hts_base64_decoded_length(1), 0);
            assert_eq!(hts_base64_decoded_length(2), 3);
            assert_eq!(hts_base64_decoded_length(4), 3);
            assert_eq!(hts_base64_decoded_length(5), 3);
            assert_eq!(hts_base64_decoded_length(6), 6);

            let input = CString::new("TWFu").unwrap();
            let mut decoded = [0u8; 16];
            out_len = 0;
            assert_eq!(
                hts_decode_base64(&mut decoded, &mut out_len, input.as_bytes()),
                0
            );
            assert_eq!(out_len, 3);
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), out_len),
                b"Man"
            );

            let input = CString::new("TWE=").unwrap();
            out_len = 0;
            assert_eq!(
                hts_decode_base64(&mut decoded, &mut out_len, input.as_bytes()),
                0
            );
            assert_eq!(out_len, 2);
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), out_len),
                b"Ma"
            );

            let input = CString::new("TQ==").unwrap();
            out_len = 0;
            assert_eq!(
                hts_decode_base64(&mut decoded, &mut out_len, input.as_bytes()),
                0
            );
            assert_eq!(out_len, 1);
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), out_len),
                b"M"
            );

            let input = CString::new("TWFu!ignored").unwrap();
            out_len = 0;
            assert_eq!(
                hts_decode_base64(&mut decoded, &mut out_len, input.as_bytes()),
                0
            );
            assert_eq!(out_len, 3);
            assert_eq!(
                std::slice::from_raw_parts(decoded.as_ptr().cast::<u8>(), out_len),
                b"Man"
            );
        }
    }

    #[test]
    fn textutils_json_string_and_token_helpers_match_c_rules() {
        unsafe {
            let mut buf = [0u8; 8];
            let end = encode_utf8(buf.as_mut_ptr(), 0x24);
            assert_eq!(end.offset_from(buf.as_ptr()), 1);
            assert_eq!(
                std::slice::from_raw_parts(buf.as_ptr(), 1),
                b"$"
            );
            let end = encode_utf8(buf.as_mut_ptr(), 0xa3);
            assert_eq!(end.offset_from(buf.as_ptr()), 2);
            assert_eq!(
                std::slice::from_raw_parts(buf.as_ptr(), 2),
                &[0xc2, 0xa3]
            );
            let end = encode_utf8(buf.as_mut_ptr(), 0x20ac);
            assert_eq!(end.offset_from(buf.as_ptr()), 3);
            assert_eq!(
                std::slice::from_raw_parts(buf.as_ptr(), 3),
                &[0xe2, 0x82, 0xac]
            );
            let end = encode_utf8(buf.as_mut_ptr(), 0x1f600);
            assert_eq!(end.offset_from(buf.as_ptr()), 4);
            assert_eq!(
                std::slice::from_raw_parts(buf.as_ptr(), 4),
                &[0xf0, 0x9f, 0x98, 0x80]
            );

            let mut json_string = b"a\\n\\t\\u00a3\\\"z\"tail\0".to_vec();
            let after = sscan_string(&mut json_string);
            assert_eq!(after.unwrap(), 15);
            assert_eq!(
                CStr::from_ptr(json_string.as_ptr().cast()).to_bytes(),
                b"a\n\t\xc2\xa3\"z"
            );

            let token = hts_json_alloc_token();
            assert!(!token.is_null());
            assert_eq!(hts_json_token_type(token), 0);
            assert!(hts_json_token_str(&*token).is_null());
            (*token).type_ = b'n' as c_char;
            (*token).str_ = c"123".as_ptr().cast_mut();
            assert_eq!(hts_json_token_type(token), b'n' as c_char);
            assert_eq!(hts_json_token_str(&*token), c"123".as_ptr().cast_mut());

            (*token).str_ = c"false".as_ptr().cast_mut();
            assert_eq!(token_type(token), b'b' as c_char);
            (*token).str_ = c"null".as_ptr().cast_mut();
            assert_eq!(token_type(token), b'.' as c_char);
            (*token).str_ = c"true".as_ptr().cast_mut();
            assert_eq!(token_type(token), b'b' as c_char);
            (*token).str_ = c"-12.5".as_ptr().cast_mut();
            assert_eq!(token_type(token), b'n' as c_char);
            (*token).str_ = c"maybe".as_ptr().cast_mut();
            assert_eq!(token_type(token), b'?' as c_char);

            hts_json_free_token(token);

            let mut json = b"{\"a\":\"b\\n\",\"n\":12,\"arr\":[true,null]}\0".to_vec();
            let mut state = 0usize;
            let mut token = hts_json_token {
                type_: 0,
                str_: std::ptr::null_mut(),
            };
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'{' as c_char
            );
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b's' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"a");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b's' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"b\n");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b's' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"n");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'n' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"12");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b's' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"arr");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'[' as c_char
            );
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'b' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"true");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'.' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"null");
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b']' as c_char
            );
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'}' as c_char
            );
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                0
            );
        }
    }

    #[test]
    fn textutils_json_skip_value_helpers_match_c_rules() {
        unsafe {
            let mut json = b"{\"outer\":[1,{\"x\":false}],\"tail\":7}\0".to_vec();
            let mut state = 0usize;
            assert_eq!(
                hts_json_sskip_value(json.as_mut_ptr().cast(), &mut state, 0),
                b'v' as c_char
            );
            assert_eq!(
                hts_json_snext(
                    json.as_mut_ptr().cast(),
                    &mut state,
                    &mut hts_json_token {
                        type_: 0,
                        str_: std::ptr::null_mut(),
                    },
                ),
                0
            );

            let mut json = b"[true,{\"a\":[null]},3]tail\0".to_vec();
            state = 0;
            let mut token = hts_json_token {
                type_: 0,
                str_: std::ptr::null_mut(),
            };
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'[' as c_char
            );
            assert_eq!(
                hts_json_sskip_value(json.as_mut_ptr().cast(), &mut state, b'[' as c_char),
                b'v' as c_char
            );
            assert_eq!(
                hts_json_snext(json.as_mut_ptr().cast(), &mut state, &mut token),
                b'?' as c_char
            );

            let mut scalar = b"123,456\0".to_vec();
            state = 0;
            assert_eq!(
                hts_json_sskip_value(scalar.as_mut_ptr().cast(), &mut state, 0),
                b'v' as c_char
            );
            assert_eq!(
                hts_json_snext(scalar.as_mut_ptr().cast(), &mut state, &mut token),
                b'n' as c_char
            );
            assert_eq!(CStr::from_ptr(token.str_).to_bytes(), b"456");

            let mut broken = b"]\0".to_vec();
            state = 0;
            assert_eq!(
                hts_json_sskip_value(broken.as_mut_ptr().cast(), &mut state, 0),
                b'?' as c_char
            );

            let mut unterminated = b"{\"a\": [1, 2\0".to_vec();
            state = 0;
            assert_eq!(
                hts_json_sskip_value(unterminated.as_mut_ptr().cast(), &mut state, 0),
                0
            );
        }
    }

    #[test]
    fn textutils_stringify_argv_matches_c_rules() {
        unsafe {
            let args: [&[u8]; 4] = [b"prog", b"one\ttwo", b"", b"last"];
            let s = stringify_argv(&args);
            assert!(s.is_some());
            assert_eq!(s.unwrap().as_slice(), b"prog one two  last");

            let s = stringify_argv(&[]);
            assert!(s.is_some());
            assert_eq!(s.unwrap().as_slice(), b"");
        }
    }

    #[test]
    fn textutils_strprint_matches_c_rules() {
        unsafe {
            let mut buf = [0u8; 64];
            let input = CString::new("a\nb\t\"\\").unwrap();
            assert!(hts_strprint(&mut buf, b'"', input.as_bytes()));
            assert_eq!(
                CStr::from_ptr(buf.as_ptr().cast()).to_bytes(),
                b"\"a\\nb\\t\\\"\\\\\""
            );

            let bytes = [b'a', 0, 1, b'z'];
            assert!(hts_strprint(&mut buf, 0, &bytes));
            assert_eq!(CStr::from_ptr(buf.as_ptr().cast()).to_bytes(), b"a\\0\\x01z");

            let input = CString::new("abcdef").unwrap();
            assert!(hts_strprint(&mut buf[..8], b'\'', input.as_bytes()));
            assert_eq!(CStr::from_ptr(buf.as_ptr().cast()).to_bytes(), b"'ab'...");
        }
    }

    #[test]
    fn hts_internal_file_extension_sleep_and_svlen_helpers_match_c_rules() {
        unsafe {
            let mut ext = [0 as c_char; HTS_MAX_EXT_LEN];
            assert_eq!(
                find_file_extension(c"sample.bam".as_ptr(), ext.as_mut_ptr()),
                0
            );
            assert_eq!(CStr::from_ptr(ext.as_ptr()).to_bytes(), b"bam");
            assert_eq!(
                find_file_extension(c"sample.sam.gz".as_ptr(), ext.as_mut_ptr()),
                0
            );
            assert_eq!(CStr::from_ptr(ext.as_ptr()).to_bytes(), b"sam.gz");
            assert_eq!(
                find_file_extension(
                    c"sample.vcf.bgz##idx##custom.csi".as_ptr(),
                    ext.as_mut_ptr()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(ext.as_ptr()).to_bytes(), b"vcf.bgz");
            assert_eq!(
                find_file_extension(c"/tmp/noext".as_ptr(), ext.as_mut_ptr()),
                -1
            );
            assert_eq!(
                find_file_extension(c"sample.too_long_ext".as_ptr(), ext.as_mut_ptr()),
                -1
            );
            assert_eq!(find_file_extension(std::ptr::null(), ext.as_mut_ptr()), -1);

            assert_eq!(hts_usleep(0), 0);

            assert_eq!(svlen_on_ref_for_vcf_alt(c"<DEL>".as_ptr(), -1), 1);
            assert_eq!(svlen_on_ref_for_vcf_alt(c"<CNV:TR>".as_ptr(), -1), 1);
            assert_eq!(svlen_on_ref_for_vcf_alt(c"<DUP:ME>tail".as_ptr(), 8), 1);
            assert_eq!(svlen_on_ref_for_vcf_alt(c"<INS>".as_ptr(), -1), 0);
            assert_eq!(svlen_on_ref_for_vcf_alt(c"ACGT".as_ptr(), -1), 0);
            assert_eq!(svlen_on_ref_for_vcf_alt(c"<DEL".as_ptr(), -1), 0);
        }
    }

    #[test]
    fn hts_decompress_peek_gz_reads_concatenated_gzip_streams() {
        unsafe {
            let mut compressed = Vec::new();
            {
                let mut enc =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                std::io::Write::write_all(&mut enc, b"abc").unwrap();
                compressed.extend(enc.finish().unwrap());
            }
            {
                let mut enc =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                std::io::Write::write_all(&mut enc, b"def").unwrap();
                compressed.extend(enc.finish().unwrap());
            }

            let buffer = Box::into_raw(compressed.clone().into_boxed_slice()).cast::<u8>();
            let fp = crate::htslib_rs::hfile::hfile_c_835_create_hfile_mem(
                buffer,
                c"r".as_ptr().cast(),
                compressed.len(),
                compressed.len(),
            );
            assert!(!fp.is_null());

            let mut out = [0u8; 8];
            assert_eq!(
                hts_c_313_decompress_peek_gz(fp, out.as_mut_ptr(), out.len()),
                6
            );
            assert_eq!(&out[..6], b"abcdef");
            assert_eq!(crate::htslib_rs::hfile::hclose(fp), 0);
        }
    }

    #[test]
    fn hts_decompress_peek_xz_reads_xz_stream() {
        unsafe {
            let mut enc = xz2::write::XzEncoder::new(Vec::new(), 6);
            std::io::Write::write_all(&mut enc, b"xz payload").unwrap();
            let compressed = enc.finish().unwrap();

            let buffer = Box::into_raw(compressed.clone().into_boxed_slice()).cast::<u8>();
            let fp = crate::htslib_rs::hfile::hfile_c_835_create_hfile_mem(
                buffer,
                c"r".as_ptr().cast(),
                compressed.len(),
                compressed.len(),
            );
            assert!(!fp.is_null());

            let mut out = [0u8; 16];
            assert_eq!(
                hts_c_356_decompress_peek_xz(fp, out.as_mut_ptr(), out.len()),
                10
            );
            assert_eq!(&out[..10], b"xz payload");
            assert_eq!(crate::htslib_rs::hfile::hclose(fp), 0);
        }
    }

    #[test]
    fn hts_expr_simple_string_functions_match_c_rules() {
        unsafe {
            let text = CString::new(" \t abc").unwrap();
            let p = ws(text.as_ptr().cast_mut().cast());
            assert_eq!(p.offset_from(text.as_ptr().cast::<u8>()), 3);

            let mut res = hts_expr_val_t {
                is_str: 0,
                is_true: 0,
                s: kstring_t::default(),
                d: 3.0,
            };
            assert_eq!(expr_func_length(&mut res), -1);

            let bytes = b"AZaz";
            res.is_str = 1;
            res.s.data = bytes.to_vec();
            assert_eq!(expr_func_length(&mut res), 0);
            assert_eq!(res.is_str, 0);
            assert_eq!(res.d, 4.0);

            res.is_str = 1;
            res.s.data = bytes.to_vec();
            assert_eq!(expr_func_min(&mut res), 0);
            assert_eq!(res.is_str, 0);
            assert_eq!(res.d, b'A' as f64);

            res.is_str = 1;
            res.s.data = bytes.to_vec();
            assert_eq!(expr_func_max(&mut res), 0);
            assert_eq!(res.d, b'z' as f64);

            res.is_str = 1;
            res.s.data = bytes.to_vec();
            assert_eq!(expr_func_avg(&mut res), 0);
            assert_eq!(
                res.d,
                (b'A' as f64 + b'Z' as f64 + b'a' as f64 + b'z' as f64) / 4.0
            );

            res.is_str = 1;
            res.s.data.clear();
            assert_eq!(expr_func_min(&mut res), 0);
            assert!(res.d.is_nan());
            res.is_str = 1;
            res.s.data.clear();
            assert_eq!(expr_func_max(&mut res), 0);
            assert!(res.d.is_nan());
            res.is_str = 1;
            res.s.data.clear();
            assert_eq!(expr_func_avg(&mut res), 0);
            assert_eq!(res.d, 0.0);
        }
    }

    unsafe extern "C" fn test_expr_sym(
        _data: *mut c_void,
        str_: *mut c_char,
        end: *mut *mut c_char,
        res: *mut hts_expr_val_t,
    ) -> i32 {
        let Some(res_ref) = res.as_mut() else {
            return -1;
        };
        if libc::strncmp(str_, c"NUM".as_ptr(), 3) == 0 {
            *end = str_.add(3);
            (*res).is_str = 0;
            (*res).is_true = 1;
            (*res).d = 5.0;
            0
        } else if libc::strncmp(str_, c"ZERO".as_ptr(), 4) == 0 {
            *end = str_.add(4);
            (*res).is_str = 0;
            (*res).is_true = 0;
            (*res).d = 0.0;
            0
        } else if libc::strncmp(str_, c"STR".as_ptr(), 3) == 0 {
            *end = str_.add(3);
            (*res).is_str = 1;
            (*res).is_true = 1;
            ks_clear(&mut (*res).s);
            kputsn(b"abc", 3, &mut (*res).s);
            0
        } else if libc::strncmp(str_, c"MISSING".as_ptr(), 7) == 0 {
            *end = str_.add(7);
            hts_expr_val_undef(res_ref);
            0
        } else {
            -1
        }
    }

    unsafe extern "C" fn upstream_expr_sym(
        _data: *mut c_void,
        str_: *mut c_char,
        end: *mut *mut c_char,
        res: *mut hts_expr_val_t,
    ) -> i32 {
        let Some(res_ref) = res.as_mut() else {
            return -1;
        };
        if libc::strncmp(str_, c"foo".as_ptr(), 3) == 0 {
            *end = str_.add(3);
            (*res).is_str = 0;
            (*res).d = 15551.0;
            0
        } else if *str_ == b'a' as c_char {
            *end = str_.add(1);
            (*res).is_str = 0;
            (*res).d = 1.0;
            0
        } else if *str_ == b'b' as c_char {
            *end = str_.add(1);
            (*res).is_str = 0;
            (*res).d = 2.0;
            0
        } else if *str_ == b'c' as c_char {
            *end = str_.add(1);
            (*res).is_str = 0;
            (*res).d = 3.0;
            0
        } else if libc::strncmp(str_, c"magic".as_ptr(), 5) == 0 {
            *end = str_.add(5);
            (*res).is_str = 1;
            ks_clear(&mut (*res).s);
            kputsn(b"plugh", 5, &mut (*res).s);
            0
        } else if libc::strncmp(str_, c"empty-but-true".as_ptr(), 14) == 0 {
            *end = str_.add(14);
            (*res).is_true = 1;
            (*res).is_str = 1;
            ks_clear(&mut (*res).s);
            kputsn(b"", 0, &mut (*res).s);
            0
        } else if libc::strncmp(str_, c"empty".as_ptr(), 5) == 0 {
            *end = str_.add(5);
            (*res).is_str = 1;
            ks_clear(&mut (*res).s);
            kputsn(b"", 0, &mut (*res).s);
            0
        } else if libc::strncmp(str_, c"zero-but-true".as_ptr(), 13) == 0 {
            *end = str_.add(13);
            (*res).is_str = 0;
            (*res).d = 0.0;
            (*res).is_true = 1;
            0
        } else if libc::strncmp(str_, c"null-but-true".as_ptr(), 13) == 0 {
            *end = str_.add(13);
            hts_expr_val_undef(res_ref);
            (*res).is_true = 1;
            0
        } else if libc::strncmp(str_, c"null".as_ptr(), 4) == 0 {
            *end = str_.add(4);
            hts_expr_val_undef(res_ref);
            0
        } else if libc::strncmp(str_, c"nan".as_ptr(), 3) == 0 {
            *end = str_.add(3);
            hts_expr_val_undef(res_ref);
            0
        } else {
            -1
        }
    }

    struct ExprCase {
        expr: &'static str,
        truth: u8,
        d: f64,
        s: Option<&'static str>,
    }

    fn same_float(actual: f64, expected: f64) -> bool {
        actual == expected || (actual.is_nan() && expected.is_nan())
    }

    unsafe fn assert_upstream_expr_cases(cases: &[ExprCase]) {
        for case in cases {
            let filt = hts_expr_c_849_hts_filter_init(case.expr.as_bytes());
            assert!(!filt.is_null(), "failed to init {}", case.expr);
            let mut res: hts_expr_val_t = hts_expr_val_t::default();
            assert_eq!(
                hts_expr_c_920_hts_filter_eval2(
                    filt,
                    std::ptr::null_mut(),
                    Some(upstream_expr_sym),
                    &mut res,
                ),
                0,
                "failed to eval {}",
                case.expr
            );
            assert_eq!(res.is_true, case.truth, "truth for {}", case.expr);
            assert!(
                same_float(res.d, case.d),
                "numeric value for {}: got {}, expected {}",
                case.expr,
                res.d,
                case.d
            );
            match case.s {
                Some(expected) => {
                    assert_ne!(res.is_str, 0, "string flag for {}", case.expr);
                    assert_eq!(
                        std::str::from_utf8(&res.s.data).unwrap(),
                        expected,
                        "string value for {}",
                        case.expr
                    );
                }
                None => assert_eq!(res.is_str, 0, "string flag for {}", case.expr),
            }
            hts_expr_val_free(&mut res);
            hts_expr_c_863_hts_filter_free(filt);
        }
    }

    #[test]
    fn hts_expr_parser_eval_edges_match_c_rules() {
        unsafe {
            let cases: [&[u8]; 6] = [
                b"1 + 2 * 3 == 7 && (5 & 3) == 1",
                b"length(\"A\\nB\") == 3",
                b"STR =~ \"^ab\" && STR !~ \"zz\"",
                b"default(MISSING, NUM) == 5",
                b"!exists(MISSING) && exists(ZERO)",
                b"MISSING || NUM",
            ];

            for expr in cases {
                let filt = hts_expr_c_849_hts_filter_init(expr);
                assert!(!filt.is_null());
                let mut res: hts_expr_val_t = hts_expr_val_t::default();
                assert_eq!(
                    hts_expr_c_920_hts_filter_eval2(
                        filt,
                        std::ptr::null_mut(),
                        Some(test_expr_sym),
                        &mut res,
                    ),
                    0
                );
                assert_eq!(res.is_str, 0);
                assert_eq!(res.is_true, 1);
                assert_eq!(res.d, 1.0);
                hts_expr_val_free(&mut res);
                hts_expr_c_863_hts_filter_free(filt);
            }

            let filt = hts_expr_c_849_hts_filter_init(b"MISSING && NUM");
            assert!(!filt.is_null());
            let mut res: hts_expr_val_t = hts_expr_val_t::default();
            assert_eq!(
                hts_expr_c_920_hts_filter_eval2(
                    filt,
                    std::ptr::null_mut(),
                    Some(test_expr_sym),
                    &mut res,
                ),
                0
            );
            assert_eq!(res.is_true, 0);
            assert_eq!(res.d, 0.0);
            hts_expr_val_free(&mut res);
            hts_expr_c_863_hts_filter_free(filt);
        }
    }

    #[test]
    fn hts_expr_upstream_precedence_and_bit_ops_match_test_expr_table() {
        unsafe {
            assert_upstream_expr_cases(&[
                ExprCase {
                    expr: "1<2 == 3>2",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "1 ^ 0&4 ^ 3",
                    truth: 1,
                    d: 2.0,
                    s: None,
                },
                ExprCase {
                    expr: "1 | 0^4 | 3",
                    truth: 1,
                    d: 7.0,
                    s: None,
                },
                ExprCase {
                    expr: "4 & 2 || 1",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "4 & (2 || 1)",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: " (2*3)&7  > 4",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: " (2*3)&(7 > 4)",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: "1 | null",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "null ^ 1",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
            ]);
        }
    }

    #[test]
    fn hts_expr_upstream_null_nan_truthiness_matches_test_expr_table() {
        unsafe {
            assert_upstream_expr_cases(&[
                ExprCase {
                    expr: "null",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "!null",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "!!null",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: "null && 1",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: "null || 1",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "null <= 0",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "nan",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "null-but-true",
                    truth: 1,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "zero-but-true",
                    truth: 1,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: "empty-but-true",
                    truth: 1,
                    d: 1.0,
                    s: Some(""),
                },
                ExprCase {
                    expr: "!empty-but-true",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
            ]);
        }
    }

    #[test]
    fn hts_expr_upstream_string_regex_and_functions_match_test_expr_table() {
        unsafe {
            assert_upstream_expr_cases(&[
                ExprCase {
                    expr: "magic",
                    truth: 1,
                    d: 1.0,
                    s: Some("plugh"),
                },
                ExprCase {
                    expr: "empty",
                    truth: 1,
                    d: 1.0,
                    s: Some(""),
                },
                ExprCase {
                    expr: "\"abc\" < \"def\"",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "\"abc\" > \"ab\"",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "null == \"x\"",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "\"abbc\" =~ \"^a+b+c+$\"",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "\"aBBc\" =~ \"^a+b+c+$\"",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: "\"aBBc\" !~ \"^a+b+c+$\"",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "\"xyzzy plugh abracadabra\" =~ magic",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
                ExprCase {
                    expr: "log(exp(9))",
                    truth: 1,
                    d: 9.0,
                    s: None,
                },
                ExprCase {
                    expr: "pow(2,3)",
                    truth: 1,
                    d: 8.0,
                    s: None,
                },
                ExprCase {
                    expr: "sqrt(-9)",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "default(null,3)",
                    truth: 1,
                    d: 3.0,
                    s: None,
                },
                ExprCase {
                    expr: "default(null-but-true,0)",
                    truth: 1,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "exists(null)",
                    truth: 0,
                    d: 0.0,
                    s: None,
                },
                ExprCase {
                    expr: "exists(null-but-true)",
                    truth: 1,
                    d: 1.0,
                    s: None,
                },
            ]);
        }
    }

    #[test]
    fn hts_expr_pow_with_null_operand_is_undefined_not_parse_error() {
        unsafe {
            assert_upstream_expr_cases(&[
                ExprCase {
                    expr: "pow(null,3)",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
                ExprCase {
                    expr: "pow(3,null)",
                    truth: 0,
                    d: f64::NAN,
                    s: None,
                },
            ]);
        }
    }

    #[test]
    fn hts_inline_binning_and_endian_helpers_match_c_rules() {
        assert_eq!(hts_bin_first(0), 0);
        assert_eq!(hts_bin_first(1), 1);
        assert_eq!(hts_bin_first(5), 4681);
        assert_eq!(hts_bin_parent(9), 1);
        assert_eq!(hts_bin_level(0), 0);
        assert_eq!(hts_bin_level(4681), 5);
        assert_eq!(hts_bin_bot(4681, 5), 0);
        assert_eq!(hts_bin_maxpos(14, 5), 1_i64 << 29);
        assert_eq!(hts_reg2bin(0, 1, 14, 5), 4681);
        assert_eq!(hts_reg2bin(0, 1_i64 << 29, 14, 5), 0);
        unsafe {
            assert_eq!(
                CStr::from_ptr(hts_c_2270_idx_format_name(HTS_FMT_CSI)),
                c"csi"
            );
            assert_eq!(
                CStr::from_ptr(hts_c_2270_idx_format_name(HTS_FMT_BAI)),
                c"bai"
            );
            assert_eq!(
                CStr::from_ptr(hts_c_2270_idx_format_name(HTS_FMT_TBI as i32)),
                c"tbi"
            );
            assert_eq!(
                CStr::from_ptr(hts_c_2270_idx_format_name(HTS_FMT_CRAI)),
                c"crai"
            );
            assert_eq!(CStr::from_ptr(hts_c_2270_idx_format_name(99)), c"unknown");

            let mut min_shift = 14;
            let mut n_lvls = 1;
            hts_c_2372_hts_adjust_csi_settings(1_000_000, &mut min_shift, &mut n_lvls);
            assert_eq!(min_shift, 14);
            assert!(n_lvls > 1);

            let mut idx: hts_idx_t = std::mem::zeroed();
            idx.min_shift = 14;
            idx.n_lvls = 5;
            idx.fmt = HTS_FMT_BAI;
            idx.z.last_off = 123;
            assert_eq!(hts_c_2533_hts_idx_maxpos(&idx), 1_i64 << 29);
            assert_eq!(
                hts_c_2538_hts_idx_check_range(&mut idx, -1, i64::MAX, i64::MAX),
                0
            );
            assert_eq!(
                hts_c_2538_hts_idx_check_range(&mut idx, 0, 0, 1_i64 << 29),
                0
            );
            *libc::__errno_location() = 0;
            assert_eq!(
                hts_c_2538_hts_idx_check_range(&mut idx, 0, 0, (1_i64 << 29) + 1),
                -1
            );
            assert_eq!(*libc::__errno_location(), libc::ERANGE);
            hts_c_2682_hts_idx_amend_last(&mut idx, 456);
            assert_eq!(idx.z.last_off, 456);

            let mut pairs = [
                hts_pair64_t {
                    u: 0x0123_4567_89ab_cdef,
                    v: 0xfedc_ba98_7654_3210,
                },
                hts_pair64_t {
                    u: 0x1111_2222_3333_4444,
                    v: 0xaaaa_bbbb_cccc_dddd,
                },
            ];
            let mut bins = hts_idx_bins_t {
                m: 2,
                n: 2,
                loff: 0,
                list: pairs.as_mut_ptr(),
            };
            hts_c_2739_swap_bins(&mut bins);
            assert_eq!(pairs[0].u, 0xefcd_ab89_6745_2301);
            assert_eq!(pairs[0].v, 0x1032_5476_98ba_dcfe);
            assert_eq!(pairs[1].u, 0x4444_3333_2222_1111);
            assert_eq!(pairs[1].v, 0xdddd_cccc_bbbb_aaaa);

            let mut fp: BGZF = BGZF::default();
            idx.otf_fp = std::ptr::null_mut();
            assert_eq!(hts_c_2748_need_idx_ugly_delay_hack(&idx), 0);
            idx.otf_fp = &mut fp;
            fp.bitfields = 0;
            assert!(!bgzf_is_compressed(&fp));
            assert_eq!(hts_c_2748_need_idx_ugly_delay_hack(&idx), 1);
            fp.bitfields = 1 << 30;
            assert!(bgzf_is_compressed(&fp));
            assert_eq!(hts_c_2748_need_idx_ugly_delay_hack(&idx), 0);

            idx.fmt = HTS_FMT_TBI as i32;
            assert_eq!(hts_c_2714_hts_idx_fmt(&mut idx), HTS_FMT_TBI as i32);

            let meta_len = 28u32;
            let mut meta_vec: Vec<u8> = vec![0u8; meta_len as usize];
            idx.meta = meta_vec.as_mut_ptr();
            std::mem::forget(meta_vec);
            idx.l_meta = meta_len;
            idx.tbi_n = 0;
            idx.last_tbi_tid = -1;
            u32_to_le(0, idx.meta.add(24));
            assert_eq!(
                hts_c_2648_hts_idx_tbi_name(&mut idx, 3, c"chr1".as_ptr()),
                1
            );
            assert_eq!(idx.tbi_n, 1);
            assert_eq!(idx.last_tbi_tid, 3);
            assert_eq!(idx.l_meta, meta_len + 5);
            assert_eq!(le_to_u32(idx.meta.add(24)), 5);
            assert_eq!(
                CStr::from_ptr(idx.meta.add(meta_len as usize).cast()),
                c"chr1"
            );
            assert_eq!(
                hts_c_2648_hts_idx_tbi_name(&mut idx, 3, c"ignored".as_ptr()),
                1
            );
            assert_eq!(
                hts_c_2648_hts_idx_tbi_name(&mut idx, -1, c"ignored".as_ptr()),
                1
            );
            drop(Vec::from_raw_parts(idx.meta, idx.l_meta as usize, idx.l_meta as usize));

            let mut lidx: hts_idx_lidx_t = std::mem::zeroed();
            assert_eq!(hts_c_2347_insert_to_l(&mut lidx, 0, 32_768, 77, 14), 0);
            assert_eq!(lidx.n, 2);
            assert_eq!(lidx.m, 2);
            assert_eq!(*lidx.offset.add(0), 77);
            assert_eq!(*lidx.offset.add(1), 77);
            assert_eq!(hts_c_2347_insert_to_l(&mut lidx, 16_384, 49_152, 88, 14), 0);
            assert_eq!(lidx.n, 3);
            assert!(lidx.m >= 3);
            assert_eq!(*lidx.offset.add(0), 77);
            assert_eq!(*lidx.offset.add(1), 77);
            assert_eq!(*lidx.offset.add(2), 88);
            drop(Box::from_raw(std::slice::from_raw_parts_mut(lidx.offset, lidx.m as usize)));

            let init = hts_c_2405_hts_idx_init(2, HTS_FMT_BAI, 1234, 14, 5);
            assert!(!init.is_null());
            assert_eq!((*init).fmt, HTS_FMT_BAI);
            assert_eq!((*init).min_shift, 14);
            assert_eq!((*init).n_lvls, 5);
            assert_eq!((*init).n_bins, 37449);
            assert_eq!((*init).n, 2);
            assert_eq!((*init).m, 2);
            assert_eq!((*init).z.last_tid, -1);
            assert_eq!((*init).z.save_tid, -1);
            assert_eq!((*init).z.last_bin, 0xffff_ffff);
            assert_eq!((*init).z.save_bin, 0xffff_ffff);
            assert_eq!((*init).z.last_off, 1234);
            assert_eq!((*init).z.save_off, 1234);
            assert_eq!((*init).z.off_beg, 1234);
            assert_eq!((*init).z.off_end, 1234);
            assert_eq!((*init).z.last_coor, 0xffff_ffff);
            assert_eq!((*init).tbi_n, -1);
            assert_eq!((*init).last_tbi_tid, -1);
            assert!(!(*init).bidx.is_null());
            assert!(!(*init).lidx.is_null());
            hts_idx_destroy(init);

            let insert_bidx = alloc_bidx(4).unwrap();
            assert_eq!(hts_c_2320_insert_to_b(insert_bidx, 3, 100, 200), 0);
            assert_eq!(hts_c_2320_insert_to_b(insert_bidx, 3, 300, 400), 0);
            let insert_k = kh_get_bin(insert_bidx, 3);
            assert_ne!(insert_k, (*insert_bidx).n_buckets);
            let insert_bin = (*insert_bidx).vals.add(insert_k as usize);
            assert_eq!((*insert_bin).n, 2);
            assert_eq!((*insert_bin).m, 2);
            assert_eq!((*(*insert_bin).list.add(0)).u, 100);
            assert_eq!((*(*insert_bin).list.add(0)).v, 200);
            assert_eq!((*(*insert_bin).list.add(1)).u, 300);
            assert_eq!((*(*insert_bin).list.add(1)).v, 400);
            {
                let nb = (*insert_bidx).n_buckets;
                let nf = if nb < 16 { 1 } else { nb >> 4 };
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    (*insert_bin).list,
                    (*insert_bin).m as usize,
                )));
                drop(Box::from_raw(std::slice::from_raw_parts_mut((*insert_bidx).flags, nf as usize)));
                drop(Box::from_raw(std::slice::from_raw_parts_mut((*insert_bidx).keys, nb as usize)));
                drop(Box::from_raw(std::slice::from_raw_parts_mut((*insert_bidx).vals, nb as usize)));
                drop(Box::from_raw(insert_bidx));
            }
            assert!(alloc_bidx(i32::MAX as u32 + 1).is_none());

            let update = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!update.is_null());
            let update_lidx = (*update).lidx;
            (*update_lidx).n = 4;
            (*update_lidx).m = 4;
            let update_offset: Vec<u64> = vec![0u64; 4];
            (*update_lidx).offset = Box::into_raw(update_offset.into_boxed_slice()).cast::<u64>();
            *(*update_lidx).offset.add(0) = u64::MAX;
            *(*update_lidx).offset.add(1) = 100;
            *(*update_lidx).offset.add(2) = u64::MAX;
            *(*update_lidx).offset.add(3) = 300;
            let bidx = alloc_bidx(4).unwrap();
            let k0 = insert_bidx_bin(bidx, hts_bin_first(5) as u32).unwrap();
            let km = insert_bidx_bin(bidx, meta_bin(update)).unwrap();
            *(*update).bidx = bidx;
            hts_c_2431_update_loff(update, 0, 0);
            assert_eq!(*(*update_lidx).offset.add(0), 100);
            assert_eq!(*(*update_lidx).offset.add(1), 100);
            assert_eq!(*(*update_lidx).offset.add(2), 300);
            assert_eq!(*(*update_lidx).offset.add(3), 300);
            assert_eq!((*(*bidx).vals.add(k0 as usize)).loff, 100);
            assert_eq!((*(*bidx).vals.add(km as usize)).loff, 0);
            hts_c_2431_update_loff(update, 0, 1);
            assert_eq!((*update_lidx).n, 0);
            assert_eq!((*update_lidx).m, 0);
            assert!((*update_lidx).offset.is_null());
            hts_idx_destroy(update);

            let mut interval_iter: hts_itr_t = std::mem::zeroed();
            let mut interval_chunks = [
                hts_pair64_t { u: 0, v: 10 },
                hts_pair64_t { u: 10, v: 30 },
                hts_pair64_t { u: 40, v: 60 },
            ];
            let mut interval_bin = hts_idx_bins_t {
                m: 3,
                n: 3,
                loff: 0,
                list: interval_chunks.as_mut_ptr(),
            };
            assert_eq!(
                hts_c_3221_add_to_interval(&mut interval_iter, &mut interval_bin, 7, 9, 15, 45),
                0
            );
            assert_eq!(interval_iter.n_off, 2);
            assert_eq!((*interval_iter.off.add(0)).u, 15);
            assert_eq!((*interval_iter.off.add(0)).v, 30);
            assert_eq!((*interval_iter.off.add(0)).max, (7u64 << 32) | 9);
            assert_eq!((*interval_iter.off.add(1)).u, 40);
            assert_eq!((*interval_iter.off.add(1)).v, 45);
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                interval_iter.off,
                interval_iter.n_off as usize,
            )));

            let interval_idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!interval_idx.is_null());
            let interval_bidx = alloc_bidx(8).unwrap();
            let interval_k =
                insert_bidx_bin(interval_bidx, hts_reg2bin(0, 10, 14, 5) as u32).unwrap();
            let stored = (*interval_bidx).vals.add(interval_k as usize);
            (*stored).m = 2;
            (*stored).n = 2;
            let stored_list: Vec<hts_pair64_t> = (0..2).map(|_| std::mem::zeroed()).collect();
            (*stored).list = Box::into_raw(stored_list.into_boxed_slice()).cast::<hts_pair64_t>();
            *(*stored).list.add(0) = hts_pair64_t { u: 100, v: 150 };
            *(*stored).list.add(1) = hts_pair64_t { u: 140, v: 180 };
            *(*interval_idx).bidx = interval_bidx;
            let mut reg_iter: hts_itr_t = std::mem::zeroed();
            assert_eq!(
                hts_c_3304_reg2intervals(
                    &mut reg_iter,
                    interval_idx,
                    0,
                    0,
                    10,
                    11,
                    0,
                    u64::MAX,
                    14,
                    5,
                ),
                1
            );
            assert_eq!(reg_iter.n_off, 1);
            assert_eq!((*reg_iter.off).u, 100);
            assert_eq!((*reg_iter.off).v, 180);
            assert_eq!((*reg_iter.off).max, 11);
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                reg_iter.off,
                reg_iter.n_off as usize,
            )));
            hts_idx_destroy(interval_idx);

            let compress_idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!compress_idx.is_null());
            let compress_bidx = alloc_bidx(8).unwrap();
            *(*compress_idx).bidx = compress_bidx;
            let child_bin = hts_bin_first(5);
            let parent_bin = hts_bin_parent(child_bin);
            assert_eq!(
                hts_c_2320_insert_to_b(compress_bidx, parent_bin, 100 << 16, (100 << 16) + 10),
                0
            );
            assert_eq!(
                hts_c_2320_insert_to_b(
                    compress_bidx,
                    child_bin,
                    (100 << 16) + 20,
                    (100 << 16) + 30
                ),
                0
            );
            assert_eq!(
                hts_c_2320_insert_to_b(
                    compress_bidx,
                    child_bin,
                    (100 << 16) + 40,
                    (100 << 16) + 50
                ),
                0
            );
            assert_ne!(
                kh_get_bin(compress_bidx, child_bin as u32),
                (*compress_bidx).n_buckets
            );
            assert_eq!(hts_c_2462_compress_binning(compress_idx, 0), 0);
            assert_eq!(
                kh_get_bin(compress_bidx, child_bin as u32),
                (*compress_bidx).n_buckets
            );
            let parent_k = kh_get_bin(compress_bidx, parent_bin as u32);
            assert_ne!(parent_k, (*compress_bidx).n_buckets);
            let parent = (*compress_bidx).vals.add(parent_k as usize);
            assert_eq!((*parent).n, 1);
            assert_eq!((*(*parent).list).u, 100 << 16);
            assert_eq!((*(*parent).list).v, (100 << 16) + 50);
            hts_idx_destroy(compress_idx);

            let finish_idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!finish_idx.is_null());
            *(*finish_idx).bidx = alloc_bidx(8).unwrap();
            (*finish_idx).z.save_tid = 0;
            (*finish_idx).z.save_bin = hts_reg2bin(0, 10, 14, 5) as u32;
            (*finish_idx).z.save_off = 11;
            (*finish_idx).z.off_beg = 7;
            (*finish_idx).z.n_mapped = 2;
            (*finish_idx).z.n_unmapped = 3;
            assert_eq!(hts_c_2515_hts_idx_finish(finish_idx, 99), 0);
            assert_eq!((*finish_idx).z.finished, 1);
            assert_eq!(hts_c_2515_hts_idx_finish(finish_idx, 100), 0);
            let finish_bidx = *(*finish_idx).bidx;
            let finish_bin_k = kh_get_bin(finish_bidx, hts_reg2bin(0, 10, 14, 5) as u32);
            assert_ne!(finish_bin_k, (*finish_bidx).n_buckets);
            let finish_bin = (*finish_bidx).vals.add(finish_bin_k as usize);
            assert_eq!((*finish_bin).n, 1);
            assert_eq!((*(*finish_bin).list).u, 11);
            assert_eq!((*(*finish_bin).list).v, 99);
            let finish_meta_k = kh_get_bin(finish_bidx, meta_bin(finish_idx));
            assert_ne!(finish_meta_k, (*finish_bidx).n_buckets);
            let finish_meta = (*finish_bidx).vals.add(finish_meta_k as usize);
            assert_eq!((*finish_meta).n, 2);
            assert_eq!((*(*finish_meta).list.add(0)).u, 7);
            assert_eq!((*(*finish_meta).list.add(0)).v, 99);
            assert_eq!((*(*finish_meta).list.add(1)).u, 2);
            assert_eq!((*(*finish_meta).list.add(1)).v, 3);
            hts_idx_destroy(finish_idx);

            let push_idx = hts_c_2405_hts_idx_init(0, HTS_FMT_BAI, 5, 14, 5);
            assert!(!push_idx.is_null());
            assert_eq!(hts_c_2558_hts_idx_push(push_idx, 0, 0, 10, 20, 1), 0);
            assert_eq!((*push_idx).n, 1);
            assert_eq!((*push_idx).m, 1);
            assert_eq!((*push_idx).z.last_tid, 0);
            assert_eq!((*push_idx).z.save_tid, 0);
            assert_eq!((*push_idx).z.save_off, 5);
            assert_eq!((*push_idx).z.last_off, 20);
            assert_eq!((*push_idx).z.n_mapped, 1);
            assert_eq!((*push_idx).z.n_unmapped, 0);
            assert_eq!(*(*(*push_idx).lidx).offset, 5);
            assert_eq!(hts_c_2558_hts_idx_push(push_idx, 0, 20, 30, 40, 0), 0);
            assert_eq!((*push_idx).z.n_mapped, 1);
            assert_eq!((*push_idx).z.n_unmapped, 1);
            assert_eq!(hts_c_2558_hts_idx_push(push_idx, 0, 15, 16, 50, 1), -1);
            assert_eq!(hts_c_2515_hts_idx_finish(push_idx, 60), 0);
            let push_bidx = *(*push_idx).bidx;
            let push_k = kh_get_bin(push_bidx, hts_reg2bin(0, 10, 14, 5) as u32);
            assert_ne!(push_k, (*push_bidx).n_buckets);
            let push_bin = (*push_bidx).vals.add(push_k as usize);
            assert_eq!((*push_bin).n, 1);
            assert_eq!((*(*push_bin).list).u, 5);
            assert_eq!((*(*push_bin).list).v, 60);
            let push_meta_k = kh_get_bin(push_bidx, meta_bin(push_idx));
            assert_ne!(push_meta_k, (*push_bidx).n_buckets);
            let push_meta = (*push_bidx).vals.add(push_meta_k as usize);
            assert_eq!((*push_meta).n, 2);
            assert_eq!((*(*push_meta).list.add(1)).u, 1);
            assert_eq!((*(*push_meta).list.add(1)).v, 1);
            hts_idx_destroy(push_idx);

            let stats_idx = hts_c_2405_hts_idx_init(3, HTS_FMT_BAI, 0, 14, 5);
            assert!(!stats_idx.is_null());
            assert_eq!(hts_c_3110_hts_idx_nseq(stats_idx), 3);
            assert_eq!(hts_c_3110_hts_idx_nseq(std::ptr::null()), -1);
            let meta_bytes = *b"abcd";
            assert_eq!(
                hts_c_3062_hts_idx_set_meta(
                    stats_idx,
                    meta_bytes.len() as u32,
                    meta_bytes.as_ptr().cast::<u8>().cast_mut(),
                    1,
                ),
                0
            );
            let mut got_meta_len = 0;
            let got_meta = hts_c_3084_hts_idx_get_meta(stats_idx, &mut got_meta_len);
            assert_eq!(got_meta_len, 4);
            assert_eq!(
                std::slice::from_raw_parts(got_meta, got_meta_len as usize),
                b"abcd"
            );
            let stats_bidx0 = alloc_bidx(2).unwrap();
            let stats_bidx2 = alloc_bidx(2).unwrap();
            let stats_k = insert_bidx_bin(stats_bidx0, meta_bin(stats_idx)).unwrap();
            let stats_val = (*stats_bidx0).vals.add(stats_k as usize);
            (*stats_val).m = 2;
            (*stats_val).n = 2;
            let stats_list: Vec<hts_pair64_t> = (0..2).map(|_| std::mem::zeroed()).collect();
            (*stats_val).list = Box::into_raw(stats_list.into_boxed_slice()).cast::<hts_pair64_t>();
            *(*stats_val).list.add(1) = hts_pair64_t { u: 12, v: 34 };
            *(*stats_idx).bidx.add(0) = stats_bidx0;
            *(*stats_idx).bidx.add(2) = stats_bidx2;
            (*stats_idx).n_no_coor = 56;
            let mut mapped = 0;
            let mut unmapped = 0;
            assert_eq!(
                hts_c_3115_hts_idx_get_stat(stats_idx, 0, &mut mapped, &mut unmapped),
                0
            );
            assert_eq!(mapped, 12);
            assert_eq!(unmapped, 34);
            assert_eq!(
                hts_c_3115_hts_idx_get_stat(stats_idx, 1, &mut mapped, &mut unmapped),
                -1
            );
            assert_eq!(hts_c_3136_hts_idx_get_n_no_coor(stats_idx), 56);
            let mut n_names = -1;
            let names = hts_c_3090_hts_idx_seqnames(
                stats_idx,
                &mut n_names,
                Some(test_id2name),
                stats_idx.cast(),
            );
            assert_eq!(n_names, 2);
            assert_eq!(CStr::from_ptr(*names.add(0)), c"seq0");
            assert_eq!(CStr::from_ptr(*names.add(1)), c"seq2");
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                names,
                (*stats_idx).n as usize,
            )));
            hts_idx_destroy(stats_idx);
        }
        assert_eq!(ed_is_big(), if cfg!(target_endian = "big") { 1 } else { 0 });
        assert_eq!(ed_swap_2(0x1234), 0x3412);
        assert_eq!(ed_swap_4(0x1234_5678), 0x7856_3412);
        assert_eq!(ed_swap_8(0x0123_4567_89ab_cdef), 0xefcd_ab89_6745_2301);
        unsafe {
            let mut v2 = 0x1234u16;
            let mut v4 = 0x1234_5678u32;
            let mut v8 = 0x0123_4567_89ab_cdefu64;
            assert_eq!(
                ed_swap_2p((&mut v2 as *mut u16).cast()),
                (&mut v2 as *mut u16).cast()
            );
            assert_eq!(v2, 0x3412);
            assert_eq!(
                ed_swap_4p((&mut v4 as *mut u32).cast()),
                (&mut v4 as *mut u32).cast()
            );
            assert_eq!(v4, 0x7856_3412);
            assert_eq!(
                ed_swap_8p((&mut v8 as *mut u64).cast()),
                (&mut v8 as *mut u64).cast()
            );
            assert_eq!(v8, 0xefcd_ab89_6745_2301);
        }
    }

    unsafe extern "C" fn test_id2name(_data: *mut c_void, id: i32) -> *const c_char {
        match id {
            0 => c"seq0".as_ptr(),
            2 => c"seq2".as_ptr(),
            _ => c"missing".as_ptr(),
        }
    }

    #[test]
    fn hts_idx_write_scalar_helpers_emit_little_endian_values() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs_idx_write_{}_{}.bin",
                std::process::id(),
                1
            ));
            let c_path = CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
            let fp = bgzf_open(c_path.as_ptr().cast(), c"wu".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(hts_c_2721_idx_write_int32(fp, -2), 4);
            assert_eq!(hts_c_2727_idx_write_uint32(fp, 0x1234_5678), 4);
            assert_eq!(hts_c_2733_idx_write_uint64(fp, 0x0123_4567_89ab_cdef), 8);
            assert_eq!(bgzf_close(fp), 0);

            let bytes = std::fs::read(&path).unwrap();
            assert_eq!(&bytes[..4], &(-2i32).to_le_bytes());
            assert_eq!(&bytes[4..8], &0x1234_5678u32.to_le_bytes());
            assert_eq!(&bytes[8..16], &0x0123_4567_89ab_cdefu64.to_le_bytes());
            std::fs::remove_file(path).unwrap();

            let idx_path = std::env::temp_dir().join(format!(
                "htslib_rs_idx_write_out_{}_{}.bai",
                std::process::id(),
                2
            ));
            let c_idx_path = CString::new(idx_path.as_os_str().as_encoded_bytes()).unwrap();
            let idx_fp = bgzf_open(c_idx_path.as_ptr().cast(), c"wu".as_ptr().cast());
            assert!(!idx_fp.is_null());
            let idx = hts_c_2405_hts_idx_init(0, HTS_FMT_BAI, 5, 14, 5);
            assert!(!idx.is_null());
            assert_eq!(hts_c_2558_hts_idx_push(idx, 0, 0, 10, 20, 1), 0);
            assert_eq!(hts_c_2515_hts_idx_finish(idx, 30), 0);
            assert_eq!(hts_c_2847_hts_idx_write_out(idx, idx_fp, HTS_FMT_BAI), 0);
            assert_eq!(bgzf_close(idx_fp), 0);
            hts_idx_destroy(idx);

            let idx_bytes = std::fs::read(&idx_path).unwrap();
            assert_eq!(&idx_bytes[..4], b"BAI\x01");
            assert_eq!(&idx_bytes[4..8], &1i32.to_le_bytes());
            assert_eq!(&idx_bytes[idx_bytes.len() - 8..], &0u64.to_le_bytes());
            std::fs::remove_file(idx_path).unwrap();

            let multi_idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!multi_idx.is_null());
            assert_eq!(
                hts_c_2558_hts_idx_push(multi_idx, 0, 0, 100, 100 << 16, 200 << 16),
                0
            );
            assert_eq!(hts_c_2515_hts_idx_finish(multi_idx, 100), 0);
            let mut interval = hts_pair_pos_t { beg: 0, end: 100 };
            let mut reg = hts_reglist_t {
                reg: c"chr1".as_ptr(),
                intervals: &mut interval,
                tid: 0,
                count: 1,
                min_beg: 0,
                max_end: 100,
            };
            let mut iter: hts_itr_t = std::mem::zeroed();
            iter.bitfields = 1 << 4;
            iter.n_reg = 1;
            iter.reg_list = &mut reg;
            assert_eq!(hts_c_3602_hts_itr_multi_bam(multi_idx, &mut iter), 0);
            assert_eq!(iter.n_off, 1);
            assert_eq!((*iter.off).u, 0);
            assert_eq!((*iter.off).v, 100);
            assert_eq!((*iter.off).max, 0);
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                iter.off,
                iter.n_off as usize,
            )));
            hts_idx_destroy(multi_idx);

            let save_base = std::env::temp_dir().join(format!(
                "htslib_rs_idx_save_{}_{}",
                std::process::id(),
                3
            ));
            let save_bai = save_base.with_extension("bai");
            let c_save_base = CString::new(save_base.as_os_str().as_encoded_bytes()).unwrap();
            let save_idx = hts_c_2405_hts_idx_init(0, HTS_FMT_BAI, 5, 14, 5);
            assert!(!save_idx.is_null());
            assert_eq!(hts_c_2558_hts_idx_push(save_idx, 0, 0, 10, 20, 1), 0);
            assert_eq!(hts_c_2515_hts_idx_finish(save_idx, 30), 0);
            assert_eq!(
                hts_c_2825_hts_idx_save(save_idx, c_save_base.as_ptr(), HTS_FMT_BAI),
                0
            );
            let saved = std::fs::read(&save_bai).unwrap();
            assert_eq!(&saved[..4], b"BAI\x01");
            let c_save_bai = CString::new(save_bai.as_os_str().as_encoded_bytes()).unwrap();
            let mut checked_fnidx: *mut c_char = std::ptr::null_mut();
            assert_eq!(
                hts_c_4756_hts_idx_check_local(
                    c_save_base.as_ptr(),
                    HTS_FMT_BAI,
                    &mut checked_fnidx
                ),
                1
            );
            assert_eq!(
                CStr::from_ptr(checked_fnidx).to_bytes(),
                path_bytes(&save_bai).as_ref()
            );
            {
                let len = CStr::from_ptr(checked_fnidx).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(checked_fnidx.cast::<u8>(), len)));
            }

            let fai_base = std::env::temp_dir().join(format!(
                "htslib_rs_idx_check_fai_{}_{}.fa.gz",
                std::process::id(),
                4
            ));
            let mut fai_bytes = path_bytes(&fai_base).into_owned();
            fai_bytes.extend_from_slice(b".fai");
            let fai_path = path_from_bytes(&fai_bytes);
            let mut gzi_bytes = path_bytes(&fai_base).into_owned();
            gzi_bytes.extend_from_slice(b".gzi");
            let gzi_path = path_from_bytes(&gzi_bytes);
            std::fs::write(&fai_base, b">ref\nA\n").unwrap();
            std::fs::write(&fai_path, b"ref\t1\t5\t1\t2\n").unwrap();
            let c_fai_base = CString::new(path_bytes(&fai_base).as_ref()).unwrap();

            let mut checked_fai: *mut c_char = std::ptr::null_mut();
            assert_eq!(
                hts_c_4756_hts_idx_check_local(c_fai_base.as_ptr(), HTS_FMT_FAI, &mut checked_fai),
                0
            );
            assert_eq!(
                CStr::from_ptr(checked_fai).to_bytes(),
                path_bytes(&fai_path).as_ref()
            );
            {
                let len = CStr::from_ptr(checked_fai).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(checked_fai.cast::<u8>(), len)));
            }

            std::fs::write(&gzi_path, b"\0\0\0\0\0\0\0\0").unwrap();
            let mut checked_fai: *mut c_char = std::ptr::null_mut();
            assert_eq!(
                hts_c_4756_hts_idx_check_local(c_fai_base.as_ptr(), HTS_FMT_FAI, &mut checked_fai),
                1
            );
            assert_eq!(
                CStr::from_ptr(checked_fai).to_bytes(),
                path_bytes(&fai_path).as_ref()
            );
            {
                let len = CStr::from_ptr(checked_fai).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(checked_fai.cast::<u8>(), len)));
            }
            std::fs::remove_file(&gzi_path).unwrap();
            std::fs::remove_file(&fai_path).unwrap();
            std::fs::remove_file(&fai_base).unwrap();

            let located = hts_c_4920_hts_idx_locatefn(c_save_base.as_ptr(), c".bai".as_ptr());
            assert!(!located.is_null());
            assert_eq!(
                CStr::from_ptr(located).to_bytes(),
                path_bytes(&save_bai).as_ref()
            );
            {
                let len = CStr::from_ptr(located).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(located.cast::<u8>(), len)));
            }
            let mut local_fn = std::ptr::null();
            let mut local_len = 0;
            assert_eq!(
                hts_c_4623_idx_test_and_fetch(
                    c_save_bai.as_ptr(),
                    &mut local_fn,
                    &mut local_len,
                    0
                ),
                0
            );
            assert_eq!(local_fn, c_save_bai.as_ptr());
            assert_eq!(local_len as usize, path_bytes(&save_bai).len());
            let read_idx = hts_c_2990_idx_read(c_save_bai.as_ptr());
            assert!(!read_idx.is_null());
            assert_eq!((*read_idx).fmt, HTS_FMT_BAI);
            assert_eq!((*read_idx).n, 1);
            assert_eq!((*read_idx).n_no_coor, 0);
            let read_bidx = *(*read_idx).bidx;
            assert!(!read_bidx.is_null());
            let read_k = kh_get_bin(read_bidx, hts_reg2bin(0, 10, 14, 5) as u32);
            assert_ne!(read_k, (*read_bidx).n_buckets);
            let read_bin = (*read_bidx).vals.add(read_k as usize);
            assert_eq!((*read_bin).n, 1);
            assert_eq!((*(*read_bin).list).u, 5);
            assert_eq!((*(*read_bin).list).v, 30);
            hts_idx_destroy(read_idx);
            let found_idx = hts_c_4925_idx_find_and_load(c_save_base.as_ptr(), HTS_FMT_BAI, 0);
            assert!(!found_idx.is_null());
            assert_eq!((*found_idx).fmt, HTS_FMT_BAI);
            hts_idx_destroy(found_idx);
            let mut decorated = path_bytes(&save_base).into_owned();
            decorated.extend_from_slice(b"##idx##");
            decorated.extend_from_slice(path_bytes(&save_bai).as_ref());
            let c_decorated = CString::new(decorated).unwrap();
            let decorated_idx = hts_c_4925_idx_find_and_load(c_decorated.as_ptr(), HTS_FMT_BAI, 0);
            assert!(!decorated_idx.is_null());
            assert_eq!((*decorated_idx).fmt, HTS_FMT_BAI);
            hts_idx_destroy(decorated_idx);
            std::fs::remove_file(save_bai).unwrap();
            hts_idx_destroy(save_idx);

            let otf_path = std::env::temp_dir().join(format!(
                "htslib_rs_idx_otf_{}_{}.bai",
                std::process::id(),
                4
            ));
            let c_otf_path = CString::new(otf_path.as_os_str().as_encoded_bytes()).unwrap();
            let otf_idx = hts_c_2405_hts_idx_init(0, HTS_FMT_BAI, 5, 14, 5);
            assert!(!otf_idx.is_null());
            assert_eq!(hts_c_2558_hts_idx_push(otf_idx, 0, 0, 10, 20, 1), 0);
            (*otf_idx).n_no_coor = 9;
            assert_eq!(hts_c_2515_hts_idx_finish(otf_idx, 30), 0);
            assert_eq!(
                hts_c_2894_hts_idx_save_but_not_close(otf_idx, c_otf_path.as_ptr(), HTS_FMT_BAI),
                0
            );
            assert!(!(*otf_idx).otf_fp.is_null());
            assert_eq!(hts_idx_close_otf_fp(otf_idx), 0);
            assert!((*otf_idx).otf_fp.is_null());
            let otf = std::fs::read(&otf_path).unwrap();
            assert_eq!(&otf[..4], b"BAI\x01");
            assert_eq!(&otf[otf.len() - 8..], &9u64.to_le_bytes());
            std::fs::remove_file(otf_path).unwrap();
            hts_idx_destroy(otf_idx);
        }
    }

    #[test]
    fn hts_idx_csi_save_load_preserves_meta_loff_and_nocoor() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs_idx_csi_{}_{}",
                std::process::id(),
                line!()
            ));
            let csi_path = path.with_extension("csi");
            let c_path = CString::new(path_bytes(&path).as_ref()).unwrap();
            let c_csi_path = CString::new(path_bytes(&csi_path).as_ref()).unwrap();
            let idx = hts_c_2405_hts_idx_init(1, HTS_FMT_CSI, 7, 5, 2);
            assert!(!idx.is_null());
            let meta = *b"csi-meta";
            assert_eq!(
                hts_c_3062_hts_idx_set_meta(
                    idx,
                    meta.len() as u32,
                    meta.as_ptr().cast::<u8>().cast_mut(),
                    1,
                ),
                0
            );
            assert_eq!(hts_c_2558_hts_idx_push(idx, 0, 0, 16, 100 << 16, 1), 0);
            assert_eq!(hts_c_2515_hts_idx_finish(idx, 200 << 16), 0);
            (*idx).n_no_coor = 1;
            assert_eq!(
                hts_c_2825_hts_idx_save(idx, c_path.as_ptr(), HTS_FMT_CSI),
                0
            );

            let read_idx = hts_c_2990_idx_read(c_csi_path.as_ptr());
            assert!(!read_idx.is_null());
            assert_eq!((*read_idx).fmt, HTS_FMT_CSI);
            assert_eq!((*read_idx).min_shift, 5);
            assert_eq!((*read_idx).n_lvls, 2);
            assert_eq!((*read_idx).n, 1);
            assert_eq!((*read_idx).n_no_coor, 1);
            assert_eq!((*read_idx).l_meta, meta.len() as u32);
            assert_eq!(
                std::slice::from_raw_parts((*read_idx).meta, (*read_idx).l_meta as usize),
                b"csi-meta"
            );

            let bidx = *(*read_idx).bidx;
            assert!(!bidx.is_null());
            let bin_k = kh_get_bin(bidx, hts_reg2bin(0, 16, 5, 2) as u32);
            assert_ne!(bin_k, (*bidx).n_buckets);
            let bin = (*bidx).vals.add(bin_k as usize);
            assert_eq!((*bin).loff, 7);
            assert_eq!((*bin).n, 1);
            assert_eq!((*(*bin).list).u, 7);
            assert_eq!((*(*bin).list).v, 200 << 16);
            let meta_k = kh_get_bin(bidx, meta_bin(read_idx));
            assert_ne!(meta_k, (*bidx).n_buckets);
            let meta_bin_val = (*bidx).vals.add(meta_k as usize);
            assert_eq!((*meta_bin_val).n, 2);
            assert_eq!((*(*meta_bin_val).list.add(1)).u, 1);
            assert_eq!((*(*meta_bin_val).list.add(1)).v, 0);

            hts_idx_destroy(read_idx);
            hts_idx_destroy(idx);
            std::fs::remove_file(csi_path).unwrap();
        }
    }

    #[test]
    fn hts_reg2bins_wide_clamps_negative_begin_and_skips_meta_bins() {
        unsafe {
            let idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!idx.is_null());
            let bidx = alloc_bidx(3).unwrap();
            *(*idx).bidx = bidx;

            assert!(insert_bidx_bin(bidx, 0).is_some());
            assert!(insert_bidx_bin(bidx, hts_bin_first(5) as u32).is_some());
            assert!(insert_bidx_bin(bidx, (hts_bin_first(5) + 1) as u32).is_some());
            assert!(insert_bidx_bin(bidx, meta_bin(idx)).is_some());

            let mut iter: hts_itr_t = std::mem::zeroed();
            assert_eq!(reg2bins(-100, 10, &mut iter, 14, 5, bidx), 2);
            assert_eq!(iter.bins.n, 2);
            let bins = std::slice::from_raw_parts(iter.bins.a, iter.bins.n as usize);
            assert!(bins.contains(&0));
            assert!(bins.contains(&hts_bin_first(5)));
            assert!(!bins.contains(&(hts_bin_first(5) + 1)));
            assert!(!bins.contains(&(meta_bin(idx) as i32)));
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                iter.bins.a,
                iter.bins.m as usize,
            )));

            hts_idx_destroy(idx);
        }
    }

    #[test]
    fn hts_reg2bins_clamps_end_at_index_max_position() {
        unsafe {
            let idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 0, 14, 5);
            assert!(!idx.is_null());
            let bidx = alloc_bidx(8).unwrap();
            *(*idx).bidx = bidx;

            let max_pos = hts_bin_maxpos(14, 5);
            assert!(insert_bidx_bin(bidx, 0).is_some());
            assert!(insert_bidx_bin(bidx, (hts_bin_first(5) - 1) as u32).is_some());
            assert!(insert_bidx_bin(bidx, hts_bin_first(5) as u32).is_some());

            let mut iter: hts_itr_t = std::mem::zeroed();
            assert_eq!(
                reg2bins(max_pos - 1, max_pos + 1000, &mut iter, 14, 5, bidx),
                2
            );
            assert_eq!(iter.bins.n, 2);
            let bins = std::slice::from_raw_parts(iter.bins.a, iter.bins.n as usize);
            assert!(bins.contains(&0));
            assert!(bins.contains(&(hts_bin_first(5) - 1)));
            assert!(!bins.contains(&hts_bin_first(5)));
            drop(Box::from_raw(std::slice::from_raw_parts_mut(
                iter.bins.a,
                iter.bins.m as usize,
            )));

            hts_idx_destroy(idx);
        }
    }

    #[test]
    fn hts_reg2bin_uses_end_exclusive_boundaries_and_no_coor_pushes_stay_unbinned() {
        assert_eq!(hts_bin_parent(hts_bin_first(5)), hts_bin_first(4));
        assert_eq!(hts_bin_level(hts_bin_first(5)), 5);
        assert_eq!(hts_reg2bin(0, 1, 14, 5), hts_bin_first(5));
        assert_eq!(hts_reg2bin(0, 1 << 14, 14, 5), hts_bin_first(5));
        assert_eq!(
            hts_reg2bin(0, (1 << 14) + 1, 14, 5),
            hts_bin_parent(hts_bin_first(5))
        );

        unsafe {
            let idx = hts_c_2405_hts_idx_init(1, HTS_FMT_BAI, 10, 14, 5);
            assert!(!idx.is_null());

            assert_eq!(hts_c_2558_hts_idx_push(idx, -1, 0, 0, 20, 0), 0);
            assert_eq!(hts_c_2558_hts_idx_push(idx, -1, 0, 0, 30, 0), 0);
            assert_eq!((*idx).n_no_coor, 2);
            assert_eq!((*idx).z.save_tid, -1);
            assert_eq!((*idx).z.last_tid, -1);
            assert_eq!((*idx).z.n_mapped, 0);
            assert_eq!((*idx).z.n_unmapped, 2);
            assert!((*(*idx).bidx).is_null());

            assert_eq!(hts_c_2558_hts_idx_push(idx, 0, 0, 1, 40, 1), -1);
            assert_eq!(hts_c_2515_hts_idx_finish(idx, 50), 0);
            assert!((*(*idx).bidx).is_null());
            assert_eq!((*idx).z.finished, 1);

            hts_idx_destroy(idx);
        }
    }

    #[test]
    fn hts_idx_destroy_frees_synthetic_bai_like_index() {
        unsafe {
            let idx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_t>()));
            (*idx).fmt = 1;
            (*idx).m = 1;
            (*idx).bidx = Box::into_raw(vec![std::ptr::null_mut::<hts_idx_bidx_t>(); 1].into_boxed_slice())
                .cast::<*mut hts_idx_bidx_t>();
            let lidx_vec: Vec<hts_idx_lidx_t> = (0..1).map(|_| std::mem::zeroed()).collect();
            (*idx).lidx = Box::into_raw(lidx_vec.into_boxed_slice()).cast::<hts_idx_lidx_t>();
            (*idx).l_meta = 2;
            (*idx).meta = Box::into_raw(vec![0u8; 2].into_boxed_slice()).cast::<u8>();

            let bidx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_bidx_t>()));
            (*bidx).n_buckets = 1;
            (*bidx).flags = Box::into_raw(vec![0u32; 1].into_boxed_slice()).cast::<u32>();
            (*bidx).keys = Box::into_raw(vec![0u32; 1].into_boxed_slice()).cast::<u32>();
            let vals_vec: Vec<hts_idx_bins_t> = (0..1).map(|_| std::mem::zeroed()).collect();
            (*bidx).vals = Box::into_raw(vals_vec.into_boxed_slice()).cast::<hts_idx_bins_t>();
            let list_vec: Vec<hts_pair64_t> = (0..1).map(|_| std::mem::zeroed()).collect();
            (*(*bidx).vals).list = Box::into_raw(list_vec.into_boxed_slice()).cast::<hts_pair64_t>();
            (*(*bidx).vals).m = 1;
            *(*idx).bidx = bidx;
            (*(*idx).lidx).offset = Box::into_raw(vec![0u64; 1].into_boxed_slice()).cast::<u64>();
            (*(*idx).lidx).m = 1;

            hts_idx_destroy(idx);
        }
    }

    #[test]
    fn hts_itr_query_builds_bai_chunk_list_without_htslib_delegate() {
        unsafe {
            let idx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_t>()));
            (*idx).fmt = HTS_FMT_BAI;
            (*idx).min_shift = 14;
            (*idx).n_lvls = 5;
            (*idx).n_bins = hts_bin_first(6);
            (*idx).n = 1;
            (*idx).m = 1;
            (*idx).bidx = Box::into_raw(vec![std::ptr::null_mut::<hts_idx_bidx_t>(); 1].into_boxed_slice())
                .cast::<*mut hts_idx_bidx_t>();
            let lidx_vec: Vec<hts_idx_lidx_t> = (0..1).map(|_| std::mem::zeroed()).collect();
            (*idx).lidx = Box::into_raw(lidx_vec.into_boxed_slice()).cast::<hts_idx_lidx_t>();

            let bidx = Box::into_raw(Box::new(std::mem::zeroed::<hts_idx_bidx_t>()));
            (*bidx).n_buckets = 8;
            (*bidx).size = 1;
            (*bidx).n_occupied = 1;
            let n_flags = if (*bidx).n_buckets < 16 { 1 } else { (*bidx).n_buckets >> 4 };
            (*bidx).flags = Box::into_raw(vec![0u32; n_flags as usize].into_boxed_slice()).cast::<u32>();
            let query_bin = hts_reg2bin(10, 20, (*idx).min_shift, (*idx).n_lvls) as u32;
            let bucket = query_bin & ((*bidx).n_buckets - 1);
            *(*bidx).flags = 0xaaaa;
            kh_set_isboth_false((*bidx).flags, bucket);
            (*bidx).keys = Box::into_raw(vec![0u32; (*bidx).n_buckets as usize].into_boxed_slice()).cast::<u32>();
            *(*bidx).keys.add(bucket as usize) = query_bin;
            let vals_vec: Vec<hts_idx_bins_t> =
                (0..(*bidx).n_buckets).map(|_| std::mem::zeroed()).collect();
            (*bidx).vals = Box::into_raw(vals_vec.into_boxed_slice()).cast::<hts_idx_bins_t>();
            let bin = (*bidx).vals.add(bucket as usize);
            (*bin).n = 1;
            (*bin).m = 1;
            (*bin).loff = 90;
            (*bin).m = 1;
            let bin_list: Vec<hts_pair64_t> = (0..1).map(|_| std::mem::zeroed()).collect();
            (*bin).list = Box::into_raw(bin_list.into_boxed_slice()).cast::<hts_pair64_t>();
            *(*bin).list = hts_pair64_t { u: 100, v: 200 };
            *(*idx).bidx = bidx;

            let iter = hts_itr_query(idx, 0, 10, 20, Some(synthetic_readrec));
            assert!(!iter.is_null());
            assert_eq!((*iter).tid, 0);
            assert_eq!((*iter).beg, 10);
            assert_eq!((*iter).end, 20);
            assert_eq!((*iter).i, -1);
            assert_eq!((*iter).n_off, 1);
            assert_eq!((*(*iter).off).u, 100);
            assert_eq!((*(*iter).off).v, 200);
            assert_eq!((*(*iter).off).max, 0);
            assert_ne!((*iter).bins.n, 0);
            hts_itr_destroy(iter);

            let iter = hts_itr_query(idx, 1, 10, 20, Some(synthetic_readrec));
            assert!(!iter.is_null());
            assert_ne!((*iter).bitfields & (1 << 1), 0);
            hts_itr_destroy(iter);

            let iter = hts_itr_query(
                std::ptr::null(),
                HTS_IDX_NONE,
                0,
                0,
                Some(synthetic_readrec),
            );
            assert!(!iter.is_null());
            assert_ne!((*iter).bitfields & 1, 0);
            assert_ne!((*iter).bitfields & (1 << 1), 0);
            assert_eq!((*iter).curr_off, 0);
            hts_itr_destroy(iter);

            let iter = hts_itr_query(
                std::ptr::null(),
                HTS_IDX_REST,
                0,
                0,
                Some(synthetic_readrec),
            );
            assert!(!iter.is_null());
            assert_ne!((*iter).bitfields & 1, 0);
            assert_eq!((*iter).bitfields & (1 << 1), 0);
            assert_eq!((*iter).curr_off, 0);
            hts_itr_destroy(iter);

            *libc::__errno_location() = 0;
            assert!(hts_itr_query(std::ptr::null(), 0, 0, 1, Some(synthetic_readrec)).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );
            assert!(hts_itr_query(std::ptr::null(), HTS_IDX_START, 0, 0, None).is_null());
            assert!(hts_itr_query(std::ptr::null(), HTS_IDX_NOCOOR, 0, 0, None).is_null());

            hts_idx_destroy(idx);
        }
    }

    #[test]
    fn hts_itr_special_offsets_follow_meta_bin_rules() {
        unsafe {
            assert_eq!(hts_itr_off(std::ptr::null(), HTS_IDX_REST), 0);
            assert_eq!(hts_itr_off(std::ptr::null(), HTS_IDX_NONE), 0);
            assert_eq!(hts_itr_off(std::ptr::null(), HTS_IDX_START), u64::MAX);

            let idx = hts_c_2405_hts_idx_init(2, HTS_FMT_BAI, 0, 14, 5);
            assert!(!idx.is_null());
            (*idx).n_no_coor = 3;
            assert_eq!(hts_itr_off(idx, HTS_IDX_START), 0);
            assert_eq!(hts_itr_off(idx, HTS_IDX_NOCOOR), 0);

            for &(tid, beg, end) in &[(0, 500u64, 800u64), (1, 100u64, 900u64)] {
                let bidx = alloc_bidx(2).unwrap();
                let k = insert_bidx_bin(bidx, meta_bin(idx)).unwrap();
                let p = (*bidx).vals.add(k as usize);
                (*p).m = 1;
                (*p).n = 1;
                let p_list: Vec<hts_pair64_t> = (0..1).map(|_| std::mem::zeroed()).collect();
                (*p).list = Box::into_raw(p_list.into_boxed_slice()).cast::<hts_pair64_t>();
                *(*p).list = hts_pair64_t { u: beg, v: end };
                *(*idx).bidx.add(tid as usize) = bidx;
            }

            assert_eq!(hts_itr_off(idx, HTS_IDX_START), 100);
            assert_eq!(hts_itr_off(idx, HTS_IDX_NOCOOR), 900);
            assert_eq!(hts_itr_off(idx, HTS_IDX_REST), 0);
            assert_eq!(hts_itr_off(idx, HTS_IDX_NONE), 0);
            assert_eq!(hts_itr_off(idx, -99), u64::MAX);
            hts_idx_destroy(idx);
        }
    }

    unsafe extern "C" fn synthetic_readrec(
        _fp: *mut BGZF,
        data: *mut c_void,
        _r: *mut c_void,
        tid: *mut i32,
        beg: *mut hts_pos_t,
        end: *mut hts_pos_t,
    ) -> i32 {
        let calls = data.cast::<i32>();
        if *calls == 0 {
            *calls += 1;
            *tid = 7;
            *beg = 11;
            *end = 13;
            42
        } else {
            -1
        }
    }

    unsafe extern "C" fn synthetic_multi_readrec(
        _fp: *mut BGZF,
        _data: *mut c_void,
        r: *mut c_void,
        tid: *mut i32,
        beg: *mut hts_pos_t,
        end: *mut hts_pos_t,
    ) -> i32 {
        let calls = r.cast::<i32>();
        if *calls == 0 {
            *calls += 1;
            *tid = 7;
            *beg = 11;
            *end = 13;
            42
        } else {
            -1
        }
    }

    unsafe extern "C" fn synthetic_seek(fp: *mut c_void, offset: i64, _where_: i32) -> i32 {
        *fp.cast::<u64>() = offset as u64;
        0
    }

    unsafe extern "C" fn synthetic_tell(fp: *mut c_void) -> i64 {
        *fp.cast::<u64>() as i64
    }

    unsafe extern "C" fn synthetic_chunked_multi_readrec(
        fp: *mut BGZF,
        _data: *mut c_void,
        r: *mut c_void,
        tid: *mut i32,
        beg: *mut hts_pos_t,
        end: *mut hts_pos_t,
    ) -> i32 {
        let calls = r.cast::<i32>();
        match *calls {
            0 => {
                *calls += 1;
                *fp.cast::<u64>() = 110;
                *tid = 7;
                *beg = 0;
                *end = 5;
                11
            }
            1 => {
                *calls += 1;
                *fp.cast::<u64>() = 150;
                *tid = 7;
                *beg = 12;
                *end = 13;
                42
            }
            _ => -1,
        }
    }

    #[test]
    fn hts_itr_next_read_rest_matches_htslib_state_updates() {
        let mut iter = hts_itr_t {
            bitfields: 1,
            tid: 0,
            n_off: 0,
            i: 0,
            n_reg: 0,
            beg: 0,
            end: 0,
            reg_list: std::ptr::null_mut(),
            curr_tid: 0,
            curr_reg: 0,
            curr_intv: 0,
            curr_beg: 0,
            curr_end: 0,
            curr_off: 0,
            nocoor_off: 0,
            off: std::ptr::null_mut(),
            readrec: Some(synthetic_readrec),
            seek: None,
            tell: None,
            bins: hts_itr_bins_t {
                n: 0,
                m: 0,
                a: std::ptr::null_mut(),
            },
        };
        let mut calls = 0;
        unsafe {
            assert_eq!(
                hts_itr_next(
                    std::ptr::null_mut(),
                    &mut iter,
                    std::ptr::null_mut(),
                    (&mut calls as *mut i32).cast()
                ),
                42
            );
            assert_eq!(iter.curr_tid, 7);
            assert_eq!(iter.curr_beg, 11);
            assert_eq!(iter.curr_end, 13);
            assert_eq!(iter.bitfields & (1 << 1), 0);

            assert_eq!(
                hts_itr_next(
                    std::ptr::null_mut(),
                    &mut iter,
                    std::ptr::null_mut(),
                    (&mut calls as *mut i32).cast()
                ),
                -1
            );
            assert_ne!(iter.bitfields & (1 << 1), 0);
        }
    }

    #[test]
    fn hts_itr_multi_next_read_rest_matches_htslib_state_updates() {
        let mut fp = htsFile {
            bitfields: 1 << 4,
            padding_0: 0,
            lineno: 0,
            line: kstring_t::default(),
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: htsFilePtr {
                bgzf: std::ptr::null_mut(),
            },
            state: std::ptr::null_mut(),
            format: htsFormat {
                category: 0,
                format: HTS_FORMAT_BAM,
                version: htsFormatVersion { major: 0, minor: 0 },
                compression: 0,
                compression_level: 0,
                specific: std::ptr::null_mut(),
            },
            idx: std::ptr::null_mut(),
            fnidx: std::ptr::null(),
            bam_header: std::ptr::null_mut(),
            filter: std::ptr::null_mut(),
        };
        let mut iter = hts_itr_t {
            bitfields: 1,
            tid: 0,
            n_off: 0,
            i: 0,
            n_reg: 0,
            beg: 0,
            end: 0,
            reg_list: std::ptr::null_mut(),
            curr_tid: 0,
            curr_reg: 0,
            curr_intv: 0,
            curr_beg: 0,
            curr_end: 0,
            curr_off: 0,
            nocoor_off: 0,
            off: std::ptr::null_mut(),
            readrec: Some(synthetic_multi_readrec),
            seek: None,
            tell: None,
            bins: hts_itr_bins_t {
                n: 0,
                m: 0,
                a: std::ptr::null_mut(),
            },
        };
        let mut calls = 0;
        unsafe {
            assert_eq!(
                hts_itr_multi_next(&mut fp, &mut iter, (&mut calls as *mut i32).cast()),
                42
            );
            assert_eq!(iter.curr_tid, 7);
            assert_eq!(iter.curr_beg, 11);
            assert_eq!(iter.curr_end, 13);
            assert_eq!(iter.bitfields & (1 << 1), 0);

            assert_eq!(
                hts_itr_multi_next(&mut fp, &mut iter, (&mut calls as *mut i32).cast()),
                -1
            );
            assert_ne!(iter.bitfields & (1 << 1), 0);
        }
    }

    #[test]
    fn hts_itr_multi_next_chunked_bam_filters_against_region_intervals() {
        let mut fake_offset = 0u64;
        let mut fp = htsFile {
            bitfields: 1 << 4,
            padding_0: 0,
            lineno: 0,
            line: kstring_t::default(),
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: htsFilePtr {
                bgzf: (&mut fake_offset as *mut u64).cast::<BGZF>(),
            },
            state: std::ptr::null_mut(),
            format: htsFormat {
                category: 0,
                format: HTS_FORMAT_BAM,
                version: htsFormatVersion { major: 0, minor: 0 },
                compression: 0,
                compression_level: 0,
                specific: std::ptr::null_mut(),
            },
            idx: std::ptr::null_mut(),
            fnidx: std::ptr::null(),
            bam_header: std::ptr::null_mut(),
            filter: std::ptr::null_mut(),
        };
        let mut intervals = [hts_pair_pos_t { beg: 10, end: 20 }];
        let mut reg_list = [hts_reglist_t {
            reg: std::ptr::null(),
            intervals: intervals.as_mut_ptr(),
            tid: 7,
            count: 1,
            min_beg: 10,
            max_end: 20,
        }];
        let mut off = [hts_pair64_max_t {
            u: 100,
            v: 200,
            max: 7u64 << 32,
        }];
        let mut iter = hts_itr_t {
            bitfields: 1 << 4,
            tid: 0,
            n_off: 1,
            i: -1,
            n_reg: 1,
            beg: 0,
            end: 0,
            reg_list: reg_list.as_mut_ptr(),
            curr_tid: 7,
            curr_reg: 0,
            curr_intv: 0,
            curr_beg: 0,
            curr_end: 0,
            curr_off: 0,
            nocoor_off: 0,
            off: off.as_mut_ptr(),
            readrec: Some(synthetic_chunked_multi_readrec),
            seek: Some(synthetic_seek),
            tell: Some(synthetic_tell),
            bins: hts_itr_bins_t {
                n: 0,
                m: 0,
                a: std::ptr::null_mut(),
            },
        };
        let mut calls = 0;
        unsafe {
            assert_eq!(
                hts_itr_multi_next(&mut fp, &mut iter, (&mut calls as *mut i32).cast()),
                42
            );
            assert_eq!(calls, 2);
            assert_eq!(iter.curr_off, 150);
            assert_eq!(iter.curr_tid, 7);
            assert_eq!(iter.curr_beg, 12);
            assert_eq!(iter.curr_end, 13);
            assert_eq!(iter.curr_intv, 0);

            assert_eq!(
                hts_itr_multi_next(&mut fp, &mut iter, (&mut calls as *mut i32).cast()),
                -1
            );
            assert_ne!(iter.bitfields & (1 << 1), 0);
        }
    }

    #[test]
    fn kstring_split_token_and_search_helpers_match_c_rules() {
        unsafe {
            let mut ks = kstring_t {
                data: b"  alpha\tbeta gamma  ".to_vec(),
            };
            let offsets = crate::htslib_rs::kstring::ksplit_vec(&mut ks, 0).unwrap();
            assert_eq!(offsets, [2, 8, 13]);
            assert_eq!(
                CStr::from_ptr(ks.data.as_ptr().add(2).cast()).to_bytes(),
                b"alpha"
            );
            assert_eq!(
                CStr::from_ptr(ks.data.as_ptr().add(8).cast()).to_bytes(),
                b"beta"
            );

            let input = b"ab:cde:fg/hij::k\0";
            let sep = b":/\0";
            let mut aux = ks_tokaux_t {
                tab: [0; 4],
                sep: 0,
                finished: 0,
                p: std::ptr::null(),
            };
            let mut starts = Vec::new();
            let mut tok = kstrtok(
                Some(&input[..input.len() - 1]),
                Some(&sep[..sep.len() - 1]),
                &mut aux,
            );
            while !tok.is_null() {
                starts.push(tok.offset_from(input.as_ptr()));
                tok = kstrtok(None, None, &mut aux);
            }
            assert_eq!(starts, [0, 3, 7, 10, 14, 15]);

            let hay = b"xxACGTACGTyy";
            let pat = b"ACGTyy";
            let mut prep: Option<Box<[i32]>> = None;
            let found = kstrstr(hay, pat, Some(&mut prep));
            assert_eq!(found.unwrap(), 6);
            assert!(prep.is_some());
            // `prep` is a Box<[i32]> of length `pat.len() + 256` (Boyer-Moore tables).
            assert_eq!(prep.as_ref().unwrap().len(), pat.len() + 256);

            let with_nul = b"abc\0needle\0";
            assert!(kstrnstr(with_nul, b"needle", with_nul.len(), None).is_none());

            let mem = b"0123456789";
            let mem_found = kmemmem(mem, b"456", None);
            assert_eq!(mem_found.unwrap(), 4);
        }
    }

    #[test]
    fn kfgetline_uses_stdio_fgets_wrapper() {
        unsafe {
            let fp = libc::tmpfile();
            assert!(!fp.is_null());
            assert!(libc::fputs(c"alpha\r\nbeta\n".as_ptr(), fp) >= 0);
            libc::rewind(fp);

            let mut ks: kstring_t = kstring_t::default();
            assert_eq!(kfgetline(&mut ks, fp), 0);
            assert_eq!(ks.data.as_slice(), b"alpha");
            assert_eq!(kfgetline(&mut ks, fp), 0);
            assert_eq!(ks.data.as_slice(), b"alphabeta");
            assert_eq!(kfgetline(&mut ks, fp), libc::EOF);

            assert_eq!(libc::fclose(fp), 0);
        }
    }

    #[test]
    fn kstring_integer_writers_match_decimal_c_rules() {
        unsafe {
            let mut ks = kstring_t::default();
            assert_eq!(kputuw(0, &mut ks), 0);
            assert_eq!(kputc(b',' as i32, &mut ks), b',' as i32);
            assert_eq!(kputuw(4_294_967_295, &mut ks), 0);
            assert_eq!(kputc(b',' as i32, &mut ks), b',' as i32);
            assert_eq!(kputw(i32::MIN, &mut ks), 0);
            assert_eq!(kputc(b',' as i32, &mut ks), b',' as i32);
            assert_eq!(kputll(i64::MIN, &mut ks), 0);
            assert_eq!(
                ks.data.as_slice(),
                b"0,4294967295,-2147483648,-9223372036854775808"
            );
        }
    }

    #[test]
    fn kstring_insert_and_c_string_accessors_match_c_rules() {
        unsafe {
            let mut ks: kstring_t = kstring_t::default();
            assert_eq!(ks_c_str(&ks), b"");
            assert_eq!(kputs(b"ace", &mut ks), 3);
            assert_eq!(kinsert_char(b'b' as c_char, 1, &mut ks), 0);
            assert_eq!(kinsert_str(b"de", 3, &mut ks), 0);
            assert_eq!(ks_len(&ks), 6);
            assert_eq!(ks_c_str(&ks), b"abcdee");
            assert_eq!(kinsert_str(b"", ks.data.len(), &mut ks), 0);
            assert_eq!(kinsert_char(b'!' as c_char, ks.data.len() + 1, &mut ks), -1);
            assert_eq!(kinsert_str(&[], 0, &mut ks), 0);
            let released = ks_release(&mut ks);
            assert_eq!(released.as_slice(), b"abcdee");
            assert_eq!(ks.data.len(), 0);
        }
    }

    #[test]
    fn kstring_raw_append_clear_and_release_match_c_rules() {
        unsafe {
            let mut ks: kstring_t = kstring_t::default();

            assert_eq!(ks_c_str(&ks), b"");
            assert_eq!(kputsn_(b"abc", 3, &mut ks), 3);
            assert_eq!(ks_len(&ks), 3);
            assert_eq!(ks_str(&mut ks).as_slice(), b"abc");

            assert_eq!(kputc_(b'X' as i32, &mut ks), 1);
            assert_eq!(ks_len(&ks), 4);
            assert_eq!(ks_str(&mut ks).as_slice(), b"abcX");

            assert_eq!(kputc(0xff, &mut ks), 0xff);
            assert_eq!(ks_len(&ks), 5);
            assert_eq!(ks_str(&mut ks).as_slice(), b"abcX\xff");

            ks_clear(&mut ks);
            assert_eq!(ks_len(&ks), 0);
            assert_eq!(ks_c_str(&ks), b"");

            let released = ks_release(&mut ks);
            assert_eq!(released.as_slice(), b"");
            assert_eq!(ks_len(&ks), 0);

            ks_initialize(&mut ks);
            assert_eq!(kputsn(b"", 0, &mut ks), 0);
            assert_eq!(ks_len(&ks), 0);
            assert_eq!(ks_str(&mut ks).as_slice(), b"");
            ks_free(&mut ks);
        }
    }

    #[test]
    fn hts_resize_array_rounds_updates_and_clears_like_htslib() {
        unsafe {
            let mut size: u64 = 2;
            // Initial allocation as a Vec<u8> of exactly `size * 4` bytes.
            let mut init: Vec<u8> = vec![0u8; size as usize * std::mem::size_of::<u32>()];
            let mut ptr = init.as_mut_ptr().cast::<()>();
            std::mem::forget(init);
            let words = ptr.cast::<u32>();
            *words.add(0) = 11;
            *words.add(1) = 22;

            assert_eq!(
                hts_resize_array_(
                    std::mem::size_of::<u32>(),
                    3,
                    std::mem::size_of::<u64>(),
                    (&mut size as *mut u64).cast(),
                    &mut ptr,
                    HTS_RESIZE_CLEAR,
                    c"test".as_ptr().cast(),
                ),
                0
            );
            assert_eq!(size, 4);
            let words = ptr.cast::<u32>();
            assert_eq!(*words.add(0), 11);
            assert_eq!(*words.add(1), 22);
            assert_eq!(*words.add(2), 0);
            assert_eq!(*words.add(3), 0);
            // Final buffer is `size * 4` = 16 bytes.
            drop(Vec::from_raw_parts(ptr.cast::<u8>(), size as usize * 4, size as usize * 4));

            let mut small_size: u32 = 0;
            let mut small_ptr: *mut () = std::ptr::null_mut();
            assert_eq!(
                hts_resize_array_(
                    1,
                    5,
                    std::mem::size_of::<u32>(),
                    (&mut small_size as *mut u32).cast(),
                    &mut small_ptr,
                    0,
                    c"test".as_ptr().cast(),
                ),
                0
            );
            assert_eq!(small_size, 8);
            drop(Vec::from_raw_parts(
                small_ptr.cast::<u8>(),
                small_size as usize,
                small_size as usize,
            ));
        }
    }

    #[test]
    fn hts_log_level_accessors_and_tags_match_htslib() {
        unsafe {
            let old = hts_get_log_level();
            hts_set_log_level(HTS_LOG_TRACE);
            assert_eq!(hts_get_log_level(), HTS_LOG_TRACE);
            hts_set_log_level(old);
        }
        assert_eq!(get_severity_tag(HTS_LOG_ERROR), b'E');
        assert_eq!(get_severity_tag(HTS_LOG_WARNING), b'W');
        assert_eq!(get_severity_tag(HTS_LOG_INFO), b'I');
        assert_eq!(get_severity_tag(HTS_LOG_DEBUG), b'D');
        assert_eq!(get_severity_tag(HTS_LOG_TRACE), b'T');
        assert_eq!(get_severity_tag(99), b'*');
    }

    #[test]
    fn hts_format_and_text_detection_helpers_match_c_rules() {
        assert_eq!(format_category(HTS_FORMAT_SAM), HTS_FORMAT_SEQUENCE_DATA);
        assert_eq!(format_category(HTS_FORMAT_BCF), HTS_FORMAT_VARIANT_DATA);
        assert_eq!(format_category(HTS_FORMAT_CSI), HTS_FORMAT_INDEX_FILE);
        assert_eq!(format_category(HTS_FORMAT_BED), HTS_FORMAT_REGION_LIST);
        assert_eq!(
            format_category(HTS_FORMAT_BINARY_FORMAT),
            HTS_FORMAT_UNKNOWN_CATEGORY
        );

        let mut fmt = htsFormat {
            category: 0,
            format: 0,
            version: htsFormatVersion { major: 0, minor: 0 },
            compression: 0,
            compression_level: 0,
            specific: std::ptr::null_mut(),
        };
        unsafe {
            let version = b"1.10;";
            parse_version(
                &mut fmt,
                version.as_ptr(),
                version.as_ptr().add(version.len()),
            );
            assert_eq!(fmt.version.major, 1);
            assert_eq!(fmt.version.minor, 10);

            let short = b"1.10";
            parse_version(&mut fmt, short.as_ptr(), short.as_ptr().add(short.len()));
            assert_eq!(fmt.version.major, 1);
            assert_eq!(fmt.version.minor, -1);

            assert_eq!(
                cmp_nonblank(
                    c"fileformat".as_ptr(),
                    b"file format".as_ptr(),
                    b"file format".as_ptr().add(11)
                ),
                0
            );
            assert_eq!(
                is_text_only(b"abc\t\r\n".as_ptr(), b"abc\t\r\n".as_ptr().add(6)),
                1
            );
            let not_text = [b'a', 0];
            assert_eq!(
                is_text_only(not_text.as_ptr(), not_text.as_ptr().add(not_text.len())),
                0
            );

            let utf16le = [0xff, 0xfe, b'a', 0, b'b', 0, b'\n', 0];
            assert_eq!(
                is_utf16_text(utf16le.as_ptr(), utf16le.as_ptr().add(utf16le.len())),
                2
            );
            let alternating = [b'a', 0, b'b', 0, b'c', 0, b'd', 0];
            assert_eq!(
                is_utf16_text(
                    alternating.as_ptr(),
                    alternating.as_ptr().add(alternating.len())
                ),
                1
            );

            let ks = kstring_t {
                data: utf16le.to_vec(),
            };
            assert_eq!(hts_is_utf16_text(&ks), 2);

            assert_eq!(hts_c_2186_hts_file_type(c"-".as_ptr()), 8);
            assert_eq!(hts_c_2186_hts_file_type(c"sample.vcf".as_ptr()), 2);
            assert_eq!(hts_c_2186_hts_file_type(c"sample.vcf.gz".as_ptr()), 3);
            assert_eq!(hts_c_2186_hts_file_type(c"sample.bcf".as_ptr()), 5);
            assert_eq!(hts_c_2186_hts_file_type(c"sample.unknown".as_ptr()), 0);

            let mut eof_fp: htsFile = htsFile::default();
            eof_fp.format.compression = HTS_COMPRESSION_NO_COMPRESSION;
            eof_fp.format.format = HTS_FORMAT_SAM;
            assert_eq!(hts_c_2208_hts_check_EOF(&mut eof_fp), 3);

            let base = std::ffi::CString::new(format!(
                "{}/hts_tmpfile_test_{}",
                std::env::temp_dir().display(),
                std::process::id()
            ))
            .unwrap();
            let mut tmpname: kstring_t = kstring_t::default();
            let fp = hts_c_1979_hts_open_tmpfile(base.as_ptr(), c"w".as_ptr(), &mut tmpname);
            assert!(!fp.is_null());
            assert_eq!(hclose(fp), 0);
            let created = String::from_utf8_lossy(&tmpname.data).into_owned();
            assert!(created.contains(".tmp_"));
            assert!(std::fs::metadata(&created).is_ok());
            std::fs::remove_file(&created).unwrap();
            ks_free(&mut tmpname);

            let mut n = 0;
            let list = hts_c_2065_hts_readlist(c"alpha,beta".as_ptr(), 0, &mut n);
            assert_eq!(n, 2);
            assert_eq!(CStr::from_ptr(*list.add(0)).to_bytes(), b"alpha");
            assert_eq!(CStr::from_ptr(*list.add(1)).to_bytes(), b"beta");
            for i in 0..n {
                let p = *list.add(i as usize);
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            drop(Box::from_raw(std::slice::from_raw_parts_mut(list, n as usize)));

            let list = hts_c_2065_hts_readlist(c",alpha,,beta,".as_ptr(), 0, &mut n);
            assert_eq!(n, 5);
            assert_eq!(CStr::from_ptr(*list.add(0)).to_bytes(), b"");
            assert_eq!(CStr::from_ptr(*list.add(1)).to_bytes(), b"alpha");
            assert_eq!(CStr::from_ptr(*list.add(2)).to_bytes(), b"");
            assert_eq!(CStr::from_ptr(*list.add(3)).to_bytes(), b"beta");
            assert_eq!(CStr::from_ptr(*list.add(4)).to_bytes(), b"");
            for i in 0..n {
                let p = *list.add(i as usize);
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            drop(Box::from_raw(std::slice::from_raw_parts_mut(list, n as usize)));

            let lines = hts_c_2130_hts_readlines(c":gamma,delta".as_ptr(), &mut n);
            assert_eq!(n, 2);
            assert_eq!(CStr::from_ptr(*lines.add(0)).to_bytes(), b"gamma");
            assert_eq!(CStr::from_ptr(*lines.add(1)).to_bytes(), b"delta");
            for i in 0..n {
                let p = *lines.add(i as usize);
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            drop(Box::from_raw(std::slice::from_raw_parts_mut(lines, n as usize)));

            let lines = hts_c_2130_hts_readlines(c":,gamma,,delta,".as_ptr(), &mut n);
            assert_eq!(n, 5);
            assert_eq!(CStr::from_ptr(*lines.add(0)).to_bytes(), b"");
            assert_eq!(CStr::from_ptr(*lines.add(1)).to_bytes(), b"gamma");
            assert_eq!(CStr::from_ptr(*lines.add(2)).to_bytes(), b"");
            assert_eq!(CStr::from_ptr(*lines.add(3)).to_bytes(), b"delta");
            assert_eq!(CStr::from_ptr(*lines.add(4)).to_bytes(), b"");
            for i in 0..n {
                let p = *lines.add(i as usize);
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            drop(Box::from_raw(std::slice::from_raw_parts_mut(lines, n as usize)));

            let list_file = format!(
                "{}/hts_readlist_test_{}.txt",
                std::env::temp_dir().display(),
                std::process::id()
            );
            std::fs::write(&list_file, b"one\n\ntwo\n").unwrap();
            let list_file_c = std::ffi::CString::new(list_file.clone()).unwrap();
            let file_list = hts_c_2065_hts_readlist(list_file_c.as_ptr(), 1, &mut n);
            assert_eq!(n, 2);
            assert_eq!(CStr::from_ptr(*file_list.add(0)).to_bytes(), b"one");
            assert_eq!(CStr::from_ptr(*file_list.add(1)).to_bytes(), b"two");
            for i in 0..n {
                let p = *file_list.add(i as usize);
                let len = CStr::from_ptr(p).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(p.cast::<u8>(), len)));
            }
            drop(Box::from_raw(std::slice::from_raw_parts_mut(file_list, n as usize)));
            std::fs::remove_file(&list_file).unwrap();

            let mut desc_fmt = htsFormat {
                category: HTS_FORMAT_VARIANT_DATA,
                format: HTS_FORMAT_VCF,
                version: htsFormatVersion {
                    major: -1,
                    minor: -1,
                },
                compression: HTS_COMPRESSION_NO_COMPRESSION,
                compression_level: -1,
                specific: std::ptr::null_mut(),
            };
            let desc = hts_c_775_hts_format_description(&desc_fmt);
            assert_eq!(CStr::from_ptr(desc).to_bytes(), b"VCF variant calling text");
            {
                let len = CStr::from_ptr(desc).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(desc.cast::<u8>(), len)));
            }

            desc_fmt.format = HTS_FORMAT_BAM;
            desc_fmt.category = HTS_FORMAT_SEQUENCE_DATA;
            desc_fmt.compression = HTS_COMPRESSION_BGZF;
            let desc = hts_c_775_hts_format_description(&desc_fmt);
            assert_eq!(
                CStr::from_ptr(desc).to_bytes(),
                b"BAM compressed sequence data"
            );
            {
                let len = CStr::from_ptr(desc).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(desc.cast::<u8>(), len)));
            }

            let mut keyword = [0 as c_char; 9];
            let rest = hts_c_1002_scan_keyword(
                c"Fq.Gz,level=5".as_ptr(),
                b',' as c_char,
                keyword.as_mut_ptr(),
                keyword.len(),
            );
            assert_eq!(CStr::from_ptr(keyword.as_ptr()).to_bytes(), b"fq.gz");
            assert_eq!(CStr::from_ptr(rest).to_bytes(), b"level=5");

            let mut opts: *mut hts_opt = std::ptr::null_mut();
            assert_eq!(
                hts_c_1021_hts_opt_add(&mut opts, c"cache_size=2K".as_ptr()),
                0
            );
            assert_eq!((*opts).opt, HTS_OPT_CACHE_SIZE);
            assert_eq!((*opts).val.i, 2048);
            assert_eq!(
                hts_c_1021_hts_opt_add(&mut opts, c"reference=ref.fa".as_ptr()),
                0
            );
            let next = (*opts).next.unwrap().as_ptr();
            assert_eq!((*next).opt, CRAM_OPT_REFERENCE);
            assert_eq!(CStr::from_ptr((*next).val.s).to_bytes(), b"ref.fa");

            let mut apply_fp: htsFile = htsFile::default();
            apply_fp.format.format = HTS_FORMAT_SAM;
            assert_eq!(hts_c_1247_hts_opt_apply(&mut apply_fp, next), 0);
            assert_eq!(CStr::from_ptr(apply_fp.fn_aux).to_bytes(), b"ref.fa");
            {
                let len = CStr::from_ptr(apply_fp.fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    apply_fp.fn_aux.cast::<u8>(),
                    len,
                )));
            }
            hts_c_1279_hts_opt_free(opts);

            let mut uppercase_opts: *mut hts_opt = std::ptr::null_mut();
            assert_eq!(
                hts_c_1021_hts_opt_add(&mut uppercase_opts, c"CACHE_SIZE=2K".as_ptr()),
                0
            );
            assert_eq!((*uppercase_opts).opt, HTS_OPT_CACHE_SIZE);
            assert_eq!((*uppercase_opts).val.i, 2048);
            hts_c_1279_hts_opt_free(uppercase_opts);

            let mut mixed_case_opts: *mut hts_opt = std::ptr::null_mut();
            assert_eq!(
                hts_c_1021_hts_opt_add(&mut mixed_case_opts, c"Cache_Size=2K".as_ptr()),
                -1
            );
            assert!(mixed_case_opts.is_null());

            let mut store_md_opts: *mut hts_opt = std::ptr::null_mut();
            assert_eq!(
                hts_c_1021_hts_opt_add(&mut store_md_opts, c"STORE_MD=1".as_ptr()),
                -1
            );
            assert!(store_md_opts.is_null());

            let mut parsed: htsFormat = std::mem::zeroed();
            assert_eq!(
                hts_c_1337_hts_parse_format(&mut parsed, c"fq.gz,level=7,fastq_rnum".as_ptr()),
                0
            );
            assert_eq!(parsed.category, HTS_FORMAT_SEQUENCE_DATA);
            assert_eq!(parsed.format, HTS_FORMAT_FASTQ_FORMAT);
            assert_eq!(parsed.compression, HTS_COMPRESSION_BGZF);
            assert!(!parsed.specific.is_null());
            let parsed_opt = parsed.specific.cast::<hts_opt>();
            assert_eq!((*parsed_opt).opt, HTS_OPT_COMPRESSION_LEVEL);
            assert_eq!((*parsed_opt).val.i, 7);
            let next = (*parsed_opt).next.unwrap().as_ptr();
            assert_eq!(
                (*next).opt,
                crate::htslib_rs::sam::FASTQ_OPT_RNUM as hts_fmt_option
            );
            hts_c_1279_hts_opt_free(parsed.specific.cast::<hts_opt>());

            let mut bad_fmt: htsFormat = std::mem::zeroed();
            assert_eq!(
                hts_c_1337_hts_parse_format(&mut bad_fmt, c"unknown".as_ptr()),
                -1
            );

            let mut process_fp: htsFile = htsFile::default();
            process_fp.format.format = HTS_FORMAT_SAM;
            assert_eq!(
                hts_c_1413_hts_process_opts(&mut process_fp, c"reference=ref2.fa".as_ptr()),
                0
            );
            assert_eq!(CStr::from_ptr(process_fp.fn_aux).to_bytes(), b"ref2.fa");
            {
                let len = CStr::from_ptr(process_fp.fn_aux).to_bytes_with_nul().len();
                drop(Box::from_raw(std::slice::from_raw_parts_mut(
                    process_fp.fn_aux.cast::<u8>(),
                    len,
                )));
            }

            let detect_path = format!(
                "{}/hts_detect_format_{}.vcf",
                std::env::temp_dir().display(),
                std::process::id()
            );
            std::fs::write(&detect_path, b"##fileformat=VCFv4.3\n#CHROM\tPOS\n").unwrap();
            let detect_c = std::ffi::CString::new(detect_path.clone()).unwrap();
            let hf = hopen(detect_c.as_ptr().cast(), c"r".as_ptr().cast());
            assert!(!hf.is_null());
            let mut detected: htsFormat = std::mem::zeroed();
            assert_eq!(
                hts_c_556_hts_detect_format2(hf, detect_c.as_ptr(), &mut detected),
                0
            );
            assert_eq!(detected.category, HTS_FORMAT_VARIANT_DATA);
            assert_eq!(detected.format, HTS_FORMAT_VCF);
            assert_eq!(detected.version.major, 4);
            assert_eq!(detected.version.minor, 3);
            assert_eq!(hclose(hf), 0);
            std::fs::remove_file(&detect_path).unwrap();

            let detect_path = format!(
                "{}/hts_detect_format_{}.fai",
                std::env::temp_dir().display(),
                std::process::id()
            );
            std::fs::write(&detect_path, b"chr1\t10\t6\t10\t11\n").unwrap();
            let detect_c = std::ffi::CString::new(detect_path.clone()).unwrap();
            let hf = hopen(detect_c.as_ptr().cast(), c"r".as_ptr().cast());
            assert!(!hf.is_null());
            assert_eq!(
                hts_c_556_hts_detect_format2(hf, detect_c.as_ptr(), &mut detected),
                0
            );
            assert_eq!(detected.category, HTS_FORMAT_INDEX_FILE);
            assert_eq!(detected.format, HTS_FORMAT_FAI_FORMAT);
            assert_eq!(hclose(hf), 0);
            std::fs::remove_file(&detect_path).unwrap();
        }
    }

    #[test]
    fn hts_detect_format_recognizes_index_magic_and_extension_fallbacks() {
        unsafe {
            let cases: &[(&[u8], *const c_char, htsFormatCategory, htsExactFormat)] = &[
                (
                    b"BAI\x01\x00\x00\x00\x00",
                    c"sample.bam.bai".as_ptr(),
                    HTS_FORMAT_INDEX_FILE,
                    HTS_FORMAT_BAI,
                ),
                (
                    b"CSI\x01\x00\x00\x00\x00",
                    c"sample.bam.csi".as_ptr(),
                    HTS_FORMAT_INDEX_FILE,
                    HTS_FORMAT_CSI,
                ),
                (
                    b"TBI\x01\x00\x00\x00\x00",
                    c"sample.vcf.gz.tbi".as_ptr(),
                    HTS_FORMAT_INDEX_FILE,
                    HTS_FORMAT_TBI,
                ),
                (
                    b"\x00\x00\x00\x00\x00\x00\x00\x00",
                    c"sample.bgz.gzi".as_ptr(),
                    HTS_FORMAT_INDEX_FILE,
                    HTS_FORMAT_GZI,
                ),
            ];

            for &(bytes, name, category, format) in cases {
                let buffer = Box::into_raw(bytes.to_vec().into_boxed_slice()).cast::<u8>();
                let fp = crate::htslib_rs::hfile::hfile_c_835_create_hfile_mem(
                    buffer,
                    c"r".as_ptr().cast(),
                    bytes.len(),
                    bytes.len(),
                );
                assert!(!fp.is_null());

                let mut detected: htsFormat = std::mem::zeroed();
                assert_eq!(hts_c_556_hts_detect_format2(fp, name, &mut detected), 0);
                assert_eq!(detected.category, category);
                assert_eq!(detected.format, format);
                assert_eq!(hclose(fp), 0);
            }
        }
    }

    #[test]
    fn hts_detect_format_distinguishes_plain_text_from_unknown_binary() {
        unsafe {
            let cases: &[(&[u8], htsFormatCategory, htsExactFormat)] = &[
                (
                    b"plain text without a known hts shape\n",
                    HTS_FORMAT_UNKNOWN_CATEGORY,
                    HTS_FORMAT_TEXT_FORMAT,
                ),
                (
                    b"\x00\xff\x10binary",
                    HTS_FORMAT_UNKNOWN_CATEGORY,
                    HTS_FORMAT_UNKNOWN_FORMAT,
                ),
            ];

            for &(bytes, category, format) in cases {
                let buffer = Box::into_raw(bytes.to_vec().into_boxed_slice()).cast::<u8>();
                let fp = crate::htslib_rs::hfile::hfile_c_835_create_hfile_mem(
                    buffer,
                    c"r".as_ptr().cast(),
                    bytes.len(),
                    bytes.len(),
                );
                assert!(!fp.is_null());

                let mut detected: htsFormat = std::mem::zeroed();
                assert_eq!(
                    hts_c_556_hts_detect_format2(fp, c"sample.dat".as_ptr(), &mut detected),
                    0
                );
                assert_eq!(detected.category, category);
                assert_eq!(detected.format, format);
                assert_eq!(detected.compression, HTS_COMPRESSION_NO_COMPRESSION);
                assert_eq!(hclose(fp), 0);
            }
        }
    }

    #[test]
    fn textutils_ctype_wrappers_match_unsigned_char_ctype_calls() {
        assert_ne!(isalnum_c(b'A' as u8), 0);
        assert_ne!(isalpha_c(b'z' as u8), 0);
        assert_ne!(isdigit_c(b'7' as u8), 0);
        assert_ne!(isgraph_c(b'!' as u8), 0);
        assert_ne!(islower_c(b'q' as u8), 0);
        assert_ne!(isprint_c(b' ' as u8), 0);
        assert_ne!(ispunct_c(b'.' as u8), 0);
        assert_ne!(isspace_c(b'\n' as u8), 0);
        assert_ne!(isupper_c(b'Q' as u8), 0);
        assert_ne!(isxdigit_c(b'f' as u8), 0);
        assert_eq!(tolower_c(b'Q' as u8), b'q' as u8);
        assert_eq!(toupper_c(b'q' as u8), b'Q' as u8);
        assert_eq!(isdigit_c((-1i8) as u8), unsafe { libc::isdigit(255) });
    }

    #[test]
    fn hts_hopen_translated_plain_read_and_write_paths_match_format_state() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-hts-hopen-{}-{}.sam",
                std::process::id(),
                line!()
            ));
            std::fs::write(&path, b"@HD\tVN:1.6\n").unwrap();
            let c_path =
                std::ffi::CString::new(crate::htslib_rs::path_bytes(&path).as_ref()).unwrap();

            let hfile = hopen(c_path.as_ptr().cast(), c"r".as_ptr().cast());
            assert!(!hfile.is_null());
            let fp = hts_hopen(hfile, c_path.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!((*fp).format.format, HTS_FORMAT_SAM);
            assert_eq!((*fp).format.compression, HTS_COMPRESSION_NO_COMPRESSION);
            assert_eq!((*fp).fp.hfile, hfile);
            assert_eq!(hts_close(fp), 0);

            let hfile_w = hopen(c_path.as_ptr().cast(), c"w".as_ptr().cast());
            assert!(!hfile_w.is_null());
            let fp_w = hts_hopen(hfile_w, c_path.as_ptr(), c"w".as_ptr());
            assert!(!fp_w.is_null());
            assert_ne!((*fp_w).bitfields & (1 << 1), 0);
            assert_eq!((*fp_w).format.format, HTS_FORMAT_TEXT_FORMAT);
            assert_eq!((*fp_w).format.compression, HTS_COMPRESSION_NO_COMPRESSION);
            assert_eq!((*fp_w).fp.hfile, hfile_w);
            assert_eq!(hts_close(fp_w), 0);

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn hts_hopen_unknown_format_failure_sets_eftype_and_leaves_hfile_to_caller() {
        unsafe {
            let bytes = b"\x00\xff\x10binary";
            let buffer = Box::into_raw(bytes.to_vec().into_boxed_slice()).cast::<u8>();
            let hfile = crate::htslib_rs::hfile::hfile_c_835_create_hfile_mem(
                buffer,
                c"r".as_ptr().cast(),
                bytes.len(),
                bytes.len(),
            );
            assert!(!hfile.is_null());

            *libc::__errno_location() = 0;
            let fp = hts_hopen(hfile, c"sample.bin".as_ptr(), c"r".as_ptr());
            assert!(fp.is_null());
            assert_eq!(*libc::__errno_location(), libc::ENOEXEC);
            assert_eq!(crate::htslib_rs::hfile::hclose(hfile), 0);
        }
    }

    #[test]
    fn hts_open_format_rewrites_modes_and_strips_idx_delimiter_like_htslib() {
        unsafe {
            let base = std::env::temp_dir().join(format!(
                "htslib_rs-open-format-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let mut decorated = path_bytes(&base).into_owned();
            decorated.extend_from_slice(b"##idx##ignored.fai");
            let c_decorated = CString::new(decorated).unwrap();
            let fmt = htsFormat {
                category: HTS_FORMAT_SEQUENCE_DATA,
                format: HTS_FORMAT_FASTA_FORMAT,
                version: htsFormatVersion {
                    major: -1,
                    minor: -1,
                },
                compression: HTS_COMPRESSION_NO_COMPRESSION,
                compression_level: -1,
                specific: std::ptr::null_mut(),
            };

            let fp = hts_open_format(c_decorated.as_ptr(), c"wbu,ignored=1".as_ptr(), &fmt);
            assert!(!fp.is_null());
            assert_ne!((*fp).bitfields & (1 << 1), 0);
            assert_eq!((*fp).format.format, HTS_FORMAT_FASTA_FORMAT);
            assert_eq!((*fp).format.compression, HTS_COMPRESSION_NO_COMPRESSION);
            assert_eq!(
                CStr::from_ptr((*fp).fn_).to_bytes(),
                path_bytes(&base).as_ref()
            );
            assert_eq!(hts_flush(fp), 0);
            assert_eq!(hts_close(fp), 0);
            assert!(base.exists());
            std::fs::remove_file(base).unwrap();

            let bgzf_path = std::env::temp_dir().join(format!(
                "htslib_rs-open-format-{}-{}.sam.gz",
                std::process::id(),
                line!()
            ));
            let c_bgzf_path = CString::new(path_bytes(&bgzf_path).as_ref()).unwrap();
            let bgzf_fmt = htsFormat {
                category: HTS_FORMAT_SEQUENCE_DATA,
                format: HTS_FORMAT_SAM,
                version: htsFormatVersion {
                    major: -1,
                    minor: -1,
                },
                compression: HTS_COMPRESSION_BGZF,
                compression_level: -1,
                specific: std::ptr::null_mut(),
            };
            let fp = hts_open_format(c_bgzf_path.as_ptr(), c"wu".as_ptr(), &bgzf_fmt);
            assert!(!fp.is_null());
            assert_eq!((*fp).format.format, HTS_FORMAT_SAM);
            assert_eq!((*fp).format.compression, HTS_COMPRESSION_BGZF);
            assert!(!hts_get_bgzfp(fp).is_null());
            assert_eq!(hts_close(fp), 0);
            std::fs::remove_file(bgzf_path).unwrap();
        }
    }

    #[test]
    fn hts_prefetch_wrappers_accept_valid_memory() {
        let mut byte = b'x';
        unsafe {
            hts_prefetch((&mut byte as *mut u8).cast());
            hts_prefetch_builtin((&mut byte as *mut u8).cast());
        }
    }

    #[test]
    fn hts_time_leaf_helpers_match_c_rules() {
        unsafe {
            let mut tens = 10;
            let mut units = 75;
            assert_eq!(hts_time_normalise(&mut tens, &mut units, 60), 0);
            assert_eq!((tens, units), (11, 15));

            tens = 10;
            units = -1;
            assert_eq!(hts_time_normalise(&mut tens, &mut units, 60), 0);
            assert_eq!((tens, units), (9, 59));

            tens = i32::MAX;
            units = 60;
            assert_eq!(hts_time_normalise(&mut tens, &mut units, 60), 1);
        }
        assert_eq!(hts_year_is_leap(2000), 1);
        assert_eq!(hts_year_is_leap(1900), 0);
        assert_eq!(hts_year_is_leap(2024), 1);
        assert_eq!(hts_year_is_leap(2023), 0);
        assert_eq!(hts_leaps_to_year_start(1), 0);
        assert_eq!(hts_leaps_to_year_start(1970), 477);
        assert_eq!(hts_leaps_to_year_start(2001), 485);
    }

    #[test]
    fn hts_time_tm_normalise_and_gm_match_calendar_rules() {
        unsafe {
            let mut t: libc::tm = std::mem::zeroed();
            t.tm_year = 124;
            t.tm_mon = 0;
            t.tm_mday = 32;
            assert_eq!(hts_time_normalise_tm(&mut t), 0);
            assert_eq!((t.tm_year, t.tm_mon, t.tm_mday), (124, 1, 1));

            let mut epoch: libc::tm = std::mem::zeroed();
            epoch.tm_year = 70;
            epoch.tm_mon = 0;
            epoch.tm_mday = 1;
            assert_eq!(hts_time_gm(&mut epoch), 0);

            let mut y2k: libc::tm = std::mem::zeroed();
            y2k.tm_year = 100;
            y2k.tm_mon = 0;
            y2k.tm_mday = 1;
            assert_eq!(hts_time_gm(&mut y2k), 946684800);

            let mut before_epoch: libc::tm = std::mem::zeroed();
            before_epoch.tm_year = 69;
            before_epoch.tm_mon = 11;
            before_epoch.tm_mday = 31;
            assert_eq!(hts_time_gm(&mut before_epoch), -1 as libc::time_t);
            assert_eq!(*libc::__errno_location(), libc::EOVERFLOW);
        }
    }

    unsafe extern "C" fn test_name2id(_data: *mut c_void, name: *const c_char) -> i32 {
        let name = CStr::from_ptr(name).to_bytes();
        match name {
            b"chr1" => 0,
            b"chr1:100-200" => 1,
            b"HLA-DRB1*12:17" => 2,
            b"chr3" => 3,
            b"chr1,chr3" => 4,
            _ => -1,
        }
    }

    #[test]
    fn hts_decimal_and_region_parsers_match_c_rules() {
        unsafe {
            let mut endp: *mut u8 = std::ptr::null_mut();
            let number = c"  -1,234.56k+tail";
            assert_eq!(
                hts_parse_decimal(number.as_ptr().cast(), &mut endp, HTS_PARSE_THOUSANDS_SEP),
                -1_234_560
            );
            assert_eq!(CStr::from_ptr(endp.cast()).to_bytes(), b"+tail");

            let exponent = c"2.5e3x";
            assert_eq!(hts_parse_decimal(exponent.as_ptr().cast(), &mut endp, 0), 2500);
            assert_eq!(*endp, b'x');

            let invalid = c"abc";
            assert_eq!(hts_parse_decimal(invalid.as_ptr().cast(), &mut endp, 0), 0);
            assert_eq!(endp, invalid.as_ptr().cast_mut().cast());

            let comma_stops_without_flag = c"1,234";
            assert_eq!(
                hts_parse_decimal(comma_stops_without_flag.as_ptr().cast(), &mut endp, 0),
                1
            );
            assert_eq!(CStr::from_ptr(endp.cast()).to_bytes(), b",234");

            let suffix_without_digits = c"k";
            assert_eq!(
                hts_parse_decimal(suffix_without_digits.as_ptr().cast(), &mut endp, 0),
                0
            );
            assert_eq!(endp, suffix_without_digits.as_ptr().cast_mut().cast());

            let hay = b"ab:cd:ef";
            let p = hts_memrchr(hay.as_ptr().cast(), b':' as i32, hay.len());
            assert_eq!(p.cast::<u8>().offset_from(hay.as_ptr()), 5);

            let mut beg64 = 0;
            let mut end64 = 0;
            let colon = hts_parse_reg64(c"chr1:1,001-2k".as_ptr(), &mut beg64, &mut end64);
            assert!(!colon.is_null());
            assert_eq!(CStr::from_ptr(colon).to_bytes(), b":1,001-2k");
            assert_eq!(beg64, 1000);
            assert_eq!(end64, 2000);

            let mut beg32 = 0;
            let mut end32 = 0;
            let colon32 = hts_parse_reg(c"chr1:10-12".as_ptr(), &mut beg32, &mut end32);
            assert!(!colon32.is_null());
            assert_eq!((beg32, end32), (9, 12));

            let mut tid = -99;
            let mut beg = -1;
            let mut end = -1;
            let ret = hts_parse_region(
                c"chr1:7-9",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(!ret.is_null());
            assert_eq!((tid, beg, end), (0, 6, 9));

            let ret = hts_parse_region(
                c"chr1:7",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(!ret.is_null());
            assert_eq!((tid, beg, end), (0, 6, HTS_POS_MAX));

            let ret = hts_parse_region(
                c"chr1:7",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                HTS_PARSE_ONE_COORD,
            );
            assert!(!ret.is_null());
            assert_eq!((tid, beg, end), (0, 6, 7));

            let ret = hts_parse_region(
                c"chr1:7-0",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(!ret.is_null());
            assert_eq!((tid, beg, end), (0, 6, HTS_POS_MAX));

            let ret = hts_parse_region(
                c"{HLA-DRB1*12:17}:3-3,chr1",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                HTS_PARSE_LIST | HTS_PARSE_ONE_COORD,
            );
            assert!(!ret.is_null());
            assert_eq!(CStr::from_ptr(ret).to_bytes(), b"chr1");
            assert_eq!((tid, beg, end), (2, 2, 3));

            let ret = hts_parse_region(
                c"{chr1,chr3},chr1",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                HTS_PARSE_LIST,
            );
            assert!(!ret.is_null());
            assert_eq!(CStr::from_ptr(ret).to_bytes(), b"chr1");
            assert_eq!((tid, beg, end), (4, 0, HTS_POS_MAX));

            let ret = hts_parse_region(
                c"chr3:1,000-1,500",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                HTS_PARSE_LIST | HTS_PARSE_ONE_COORD,
            );
            assert!(!ret.is_null());
            assert_eq!(CStr::from_ptr(ret).to_bytes(), b"000-1,500");
            assert_eq!((tid, beg, end), (3, 0, 1));

            let invalid_comma = hts_parse_region(
                c"chr1:1,chr3",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(invalid_comma.is_null());

            let invalid_negative_start = hts_parse_region(
                c"chr1:-1-10",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(invalid_negative_start.is_null());

            let mismatched_brace = hts_parse_region(
                c"{chr1",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(mismatched_brace.is_null());
            assert_eq!(tid, -1);

            let ambiguous = hts_parse_region(
                c"chr1:100-200",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(ambiguous.is_null());
            assert_eq!(tid, -1);
        }
    }

    #[test]
    fn hts_region_parsers_reject_nulls_and_32bit_overflow_like_c_rules() {
        unsafe {
            let mut tid = 0;
            let mut beg = 0;
            let mut end = 0;
            assert!(hts_parse_region(
                c"chr1",
                &mut tid,
                &mut beg,
                &mut end,
                None,
                std::ptr::null_mut(),
                0,
            )
            .is_null());

            let mut beg32 = 0;
            let mut end32 = 0;
            assert!(hts_parse_reg(
                c"chr1:2147483648-2147483649".as_ptr(),
                &mut beg32,
                &mut end32
            )
            .is_null());
            assert!(hts_parse_reg(c"chr1:1-2147483648".as_ptr(), &mut beg32, &mut end32).is_null());

            let mut beg64 = 0;
            let mut end64 = 0;
            assert!(hts_parse_reg64(c"chr1:20-10".as_ptr(), &mut beg64, &mut end64).is_null());
        }
    }

    #[test]
    fn hts_region_parser_list_and_open_ended_edges_match_c_rules() {
        unsafe {
            let mut tid = -99;
            let mut beg = -1;
            let mut end = -1;
            let next = hts_parse_region(
                c"chr1:5-,chr3",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                HTS_PARSE_LIST,
            );
            assert!(!next.is_null());
            assert_eq!(CStr::from_ptr(next).to_bytes(), b"chr3");
            assert_eq!((tid, beg, end), (0, 4, HTS_POS_MAX));

            let next = hts_parse_region(
                c"chr1:5-5,chr3",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                HTS_PARSE_LIST | HTS_PARSE_ONE_COORD,
            );
            assert!(!next.is_null());
            assert_eq!(CStr::from_ptr(next).to_bytes(), b"chr3");
            assert_eq!((tid, beg, end), (0, 4, 5));

            let empty_coord = hts_parse_region(
                c"chr1:",
                &mut tid,
                &mut beg,
                &mut end,
                Some(test_name2id),
                std::ptr::null_mut(),
                0,
            );
            assert!(!empty_coord.is_null());
            assert_eq!((tid, beg, end), (0, 0, HTS_POS_MAX));
        }
    }
}
