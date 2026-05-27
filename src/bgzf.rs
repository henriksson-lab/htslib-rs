use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_uint, c_ulong, c_void, CStr},
    ptr, slice,
    sync::OnceLock,
};

#[cfg(test)]
use flate2::{read::GzDecoder, Decompress, FlushDecompress, Status};

use super::{
    c_compat,
    hts::{
        ed_is_big, ed_swap_8, hts_c_2538_hts_idx_check_range, hts_idx_push, hts_idx_t, hts_pos_t,
        ks_expand, kstring_t, BGZF,
    },
};

const BGZF_MAX_BLOCK_SIZE: usize = 0x10000;
const BGZF_BLOCK_SIZE: usize = 0xff00;
const BLOCK_HEADER_LENGTH: usize = 18;
const BLOCK_FOOTER_LENGTH: usize = 8;

const BGZF_ERR_ZLIB: u32 = 1;
const BGZF_ERR_HEADER: u32 = 2;
const BGZF_ERR_IO: u32 = 4;
const BGZF_ERR_MISUSE: u32 = 8;
const BGZF_ERR_CRC: u32 = 32;

const SEEK_SET: c_int = 0;
const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_NEED_DICT: c_int = 2;
const Z_ERRNO: c_int = -1;
const Z_STREAM_ERROR: c_int = -2;
const Z_DATA_ERROR: c_int = -3;
const Z_MEM_ERROR: c_int = -4;
const Z_BUF_ERROR: c_int = -5;
const Z_VERSION_ERROR: c_int = -6;
const Z_SYNC_FLUSH: c_int = 2;
const Z_PARTIAL_FLUSH: c_int = 1;
const Z_FINISH: c_int = 4;
const Z_DEFLATED: c_int = 8;
const Z_DEFAULT_COMPRESSION: c_int = -1;
const Z_DEFAULT_STRATEGY: c_int = 0;
const EOF_BLOCK: [u8; 28] = [
    31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 27, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static mut BGZF_ZERR_BUFFER: [c_char; 32] = [0; 32];

#[repr(C)]
struct z_stream {
    next_in: *mut u8,
    avail_in: c_uint,
    total_in: c_ulong,
    next_out: *mut u8,
    avail_out: c_uint,
    total_out: c_ulong,
    msg: *mut c_char,
    state: *mut c_void,
    zalloc: Option<unsafe extern "C" fn(*mut c_void, c_uint, c_uint) -> *mut c_void>,
    zfree: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    opaque: *mut c_void,
    data_type: c_int,
    adler: c_ulong,
    reserved: c_ulong,
}

type DeflateInit2Fn = unsafe extern "C" fn(
    strm: *mut z_stream,
    level: c_int,
    method: c_int,
    window_bits: c_int,
    mem_level: c_int,
    strategy: c_int,
    version: *const c_char,
    stream_size: c_int,
) -> c_int;
type DeflateFn = unsafe extern "C" fn(strm: *mut z_stream, flush: c_int) -> c_int;
type DeflateEndFn = unsafe extern "C" fn(strm: *mut z_stream) -> c_int;
type InflateInit2Fn = unsafe extern "C" fn(
    strm: *mut z_stream,
    window_bits: c_int,
    version: *const c_char,
    stream_size: c_int,
) -> c_int;
type InflateFn = unsafe extern "C" fn(strm: *mut z_stream, flush: c_int) -> c_int;
type InflateResetFn = unsafe extern "C" fn(strm: *mut z_stream) -> c_int;
type InflateEndFn = unsafe extern "C" fn(strm: *mut z_stream) -> c_int;
type Crc32Fn = unsafe extern "C" fn(crc: c_ulong, buf: *const u8, len: c_uint) -> c_ulong;
type ZlibVersionFn = unsafe extern "C" fn() -> *const c_char;

unsafe extern "C" {
    #[link_name = "deflateInit2_"]
    fn linked_deflate_init2(
        strm: *mut z_stream,
        level: c_int,
        method: c_int,
        window_bits: c_int,
        mem_level: c_int,
        strategy: c_int,
        version: *const c_char,
        stream_size: c_int,
    ) -> c_int;
    #[link_name = "deflate"]
    fn linked_deflate(strm: *mut z_stream, flush: c_int) -> c_int;
    #[link_name = "deflateEnd"]
    fn linked_deflate_end(strm: *mut z_stream) -> c_int;
    #[link_name = "inflateInit2_"]
    fn linked_inflate_init2(
        strm: *mut z_stream,
        window_bits: c_int,
        version: *const c_char,
        stream_size: c_int,
    ) -> c_int;
    #[link_name = "inflate"]
    fn linked_inflate(strm: *mut z_stream, flush: c_int) -> c_int;
    #[link_name = "inflateReset"]
    fn linked_inflate_reset(strm: *mut z_stream) -> c_int;
    #[link_name = "inflateEnd"]
    fn linked_inflate_end(strm: *mut z_stream) -> c_int;
    #[link_name = "crc32"]
    fn linked_crc32(crc: c_ulong, buf: *const u8, len: c_uint) -> c_ulong;
    #[link_name = "zlibVersion"]
    fn linked_zlib_version() -> *const c_char;
}

struct ZlibFns {
    deflate_init2: DeflateInit2Fn,
    deflate: DeflateFn,
    deflate_end: DeflateEndFn,
    inflate_init2: InflateInit2Fn,
    inflate: InflateFn,
    inflate_reset: InflateResetFn,
    inflate_end: InflateEndFn,
    crc32: Crc32Fn,
    zlib_version: ZlibVersionFn,
}

#[repr(C)]
struct GzipStream {
    zs: z_stream,
}

unsafe fn system_zlib() -> Option<&'static ZlibFns> {
    static ZLIB: OnceLock<Option<ZlibFns>> = OnceLock::new();
    ZLIB.get_or_init(|| unsafe {
        if !linked_zlib_version().is_null() {
            return Some(ZlibFns {
                deflate_init2: linked_deflate_init2,
                deflate: linked_deflate,
                deflate_end: linked_deflate_end,
                inflate_init2: linked_inflate_init2,
                inflate: linked_inflate,
                inflate_reset: linked_inflate_reset,
                inflate_end: linked_inflate_end,
                crc32: linked_crc32,
                zlib_version: linked_zlib_version,
            });
        }

        let mut should_close = false;
        let mut handle = libc::RTLD_DEFAULT;
        let mut deflate_init2 = libc::dlsym(handle, c"deflateInit2_".as_ptr());
        let mut deflate = libc::dlsym(handle, c"deflate".as_ptr());
        let mut deflate_end = libc::dlsym(handle, c"deflateEnd".as_ptr());
        let mut inflate_init2 = libc::dlsym(handle, c"inflateInit2_".as_ptr());
        let mut inflate = libc::dlsym(handle, c"inflate".as_ptr());
        let mut inflate_reset = libc::dlsym(handle, c"inflateReset".as_ptr());
        let mut inflate_end = libc::dlsym(handle, c"inflateEnd".as_ptr());
        let mut crc32 = libc::dlsym(handle, c"crc32".as_ptr());
        let mut zlib_version = libc::dlsym(handle, c"zlibVersion".as_ptr());

        if deflate_init2.is_null()
            || deflate.is_null()
            || deflate_end.is_null()
            || inflate_init2.is_null()
            || inflate.is_null()
            || inflate_reset.is_null()
            || inflate_end.is_null()
            || crc32.is_null()
            || zlib_version.is_null()
        {
            handle = libc::dlopen(c"libz.so.1".as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL);
            if handle.is_null() {
                handle = libc::dlopen(c"libz.so".as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL);
            }
            if handle.is_null() {
                return None;
            }
            should_close = true;

            deflate_init2 = libc::dlsym(handle, c"deflateInit2_".as_ptr());
            deflate = libc::dlsym(handle, c"deflate".as_ptr());
            deflate_end = libc::dlsym(handle, c"deflateEnd".as_ptr());
            inflate_init2 = libc::dlsym(handle, c"inflateInit2_".as_ptr());
            inflate = libc::dlsym(handle, c"inflate".as_ptr());
            inflate_reset = libc::dlsym(handle, c"inflateReset".as_ptr());
            inflate_end = libc::dlsym(handle, c"inflateEnd".as_ptr());
            crc32 = libc::dlsym(handle, c"crc32".as_ptr());
            zlib_version = libc::dlsym(handle, c"zlibVersion".as_ptr());
        }

        if deflate_init2.is_null()
            || deflate.is_null()
            || deflate_end.is_null()
            || inflate_init2.is_null()
            || inflate.is_null()
            || inflate_reset.is_null()
            || inflate_end.is_null()
            || crc32.is_null()
            || zlib_version.is_null()
        {
            if should_close {
                libc::dlclose(handle);
            }
            return None;
        }

        Some(ZlibFns {
            deflate_init2: std::mem::transmute::<*mut c_void, DeflateInit2Fn>(deflate_init2),
            deflate: std::mem::transmute::<*mut c_void, DeflateFn>(deflate),
            deflate_end: std::mem::transmute::<*mut c_void, DeflateEndFn>(deflate_end),
            inflate_init2: std::mem::transmute::<*mut c_void, InflateInit2Fn>(inflate_init2),
            inflate: std::mem::transmute::<*mut c_void, InflateFn>(inflate),
            inflate_reset: std::mem::transmute::<*mut c_void, InflateResetFn>(inflate_reset),
            inflate_end: std::mem::transmute::<*mut c_void, InflateEndFn>(inflate_end),
            crc32: std::mem::transmute::<*mut c_void, Crc32Fn>(crc32),
            zlib_version: std::mem::transmute::<*mut c_void, ZlibVersionFn>(zlib_version),
        })
    })
    .as_ref()
}

#[repr(C)]
pub struct z_stream_s {
    pub next_in: *const u8,
    pub avail_in: c_uint,
    pub total_in: c_ulong,
    pub next_out: *mut u8,
    pub avail_out: c_uint,
    pub total_out: c_ulong,
    pub msg: *mut c_char,
    pub state: *mut c_void,
    pub zalloc: *mut c_void,
    pub zfree: *mut c_void,
    pub opaque: *mut c_void,
    pub data_type: c_int,
    pub adler: c_ulong,
    pub reserved: c_ulong,
}

#[repr(C)]
pub struct bgzidx1_t {
    pub uaddr: u64,
    pub caddr: u64,
}

#[repr(C)]
pub struct bgzidx_t {
    pub noffs: c_int,
    pub moffs: c_int,
    pub offs: *mut bgzidx1_t,
    pub ublock_addr: u64,
}

// original: bgzf_job (htslib/bgzf.c:92)
#[repr(C)]
pub struct bgzf_job {
    pub fp: *mut BGZF,
    pub comp_data: [u8; BGZF_MAX_BLOCK_SIZE],
    pub comp_len: usize,
    pub uncomp_data: [u8; BGZF_MAX_BLOCK_SIZE],
    pub uncomp_len: usize,
    pub errcode: c_int,
    pub block_address: i64,
    pub hit_eof: c_int,
}

// original: mtaux_cmd (htslib/bgzf.c:104)
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum mtaux_cmd {
    NONE = 0,
    SEEK,
    SEEK_DONE,
    HAS_EOF,
    HAS_EOF_DONE,
    CLOSE,
}

// original: hts_idx_cache_entry (htslib/bgzf.c:115)
#[repr(C)]
pub struct hts_idx_cache_entry {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    pub tid: c_int,
    pub is_mapped: c_int,
    pub offset: u64,
    pub block_number: u64,
}

// original: hts_idx_cache_t (htslib/bgzf.c:121)
#[repr(C)]
pub struct hts_idx_cache_t {
    pub nentries: c_int,
    pub mentries: c_int,
    pub e: *mut hts_idx_cache_entry,
}

// original: bgzf_mtaux_t (htslib/bgzf.c:125)
#[repr(C)]
pub struct bgzf_mtaux_t {
    pub job_pool: *mut c_void,
    pub curr_job: *mut bgzf_job,
    pub n_threads: c_int,
    pub own_pool: c_int,
    pub pool: *mut super::thread_pool::hts_tpool,
    pub out_queue: *mut c_void,
    pub io_task: libc::pthread_t,
    pub job_pool_m: libc::pthread_mutex_t,
    pub jobs_pending: c_int,
    pub flush_pending: c_int,
    pub free_block: *mut c_void,
    pub hit_eof: c_int,
    pub errcode: c_int,
    pub block_address: u64,
    pub eof: c_int,
    pub command_m: libc::pthread_mutex_t,
    pub command_c: libc::pthread_cond_t,
    pub command: mtaux_cmd,
    pub idx_m: libc::pthread_mutex_t,
    pub hts_idx: *mut hts_idx_t,
    pub block_number: u64,
    pub block_written: u64,
    pub idx_cache: hts_idx_cache_t,
}

pub type BgzfPrivateDataCleanupFunc = unsafe extern "C" fn(*mut c_void);

// original: bgzf_cache_t (htslib/bgzf_internal.h:41)
#[repr(C)]
pub struct bgzf_cache_t {
    pub h: *mut c_void,
    pub last_pos: c_uint,
    pub private_data: *mut c_void,
    pub private_data_cleanup: Option<BgzfPrivateDataCleanupFunc>,
}

struct BgzfCachedBlock {
    size: c_int,
    end_offset: i64,
    block: Box<[u8; BGZF_MAX_BLOCK_SIZE]>,
}

struct BgzfBlockCache {
    blocks: HashMap<i64, BgzfCachedBlock>,
    order: Vec<i64>,
}

// original: bgzf_idx_push (htslib/bgzf.c:189)
pub unsafe fn bgzf_c_189_bgzf_idx_push(
    fp: *mut BGZF,
    hidx: *mut hts_idx_t,
    tid: c_int,
    beg: hts_pos_t,
    end: hts_pos_t,
    offset: u64,
    is_mapped: c_int,
) -> c_int {
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();

    if mt.is_null() {
        return hts_idx_push(hidx, tid, beg, end, offset, is_mapped);
    }

    if hts_c_2538_hts_idx_check_range(hidx, tid, beg, end) < 0 {
        return -1;
    }

    libc::pthread_mutex_lock(&mut (*mt).idx_m);

    (*mt).hts_idx = hidx;
    let ic = &mut (*mt).idx_cache as *mut hts_idx_cache_t;

    if (*ic).nentries >= (*ic).mentries {
        let new_sz = if (*ic).mentries != 0 {
            (*ic).mentries * 2
        } else {
            1024
        };
        let e = c_compat::realloc(
            (*ic).e.cast(),
            new_sz as u64 * std::mem::size_of::<hts_idx_cache_entry>() as u64,
        )
        .cast::<hts_idx_cache_entry>();
        if e.is_null() {
            libc::pthread_mutex_unlock(&mut (*mt).idx_m);
            return -1;
        }
        (*ic).e = e;
        (*ic).mentries = new_sz;
    }

    let e = (*ic).e.add((*ic).nentries as usize);
    (*ic).nentries += 1;
    (*e).tid = tid;
    (*e).beg = beg;
    (*e).end = end;
    (*e).is_mapped = is_mapped;
    (*e).offset = offset & 0xffff;
    (*e).block_number = (*mt).block_number;

    libc::pthread_mutex_unlock(&mut (*mt).idx_m);

    0
}

// original: bgzf_idx_flush (htslib/bgzf.c:228)
pub unsafe fn bgzf_c_228_bgzf_idx_flush(
    fp: *mut BGZF,
    block_uncomp_len: usize,
    block_comp_len: usize,
) -> c_int {
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();

    if (*mt).idx_cache.e.is_null() {
        (*mt).block_written += 1;
        return 0;
    }

    libc::pthread_mutex_lock(&mut (*mt).idx_m);

    let e = (*mt).idx_cache.e;
    let mut i = 0;
    while i < (*mt).idx_cache.nentries && (*e.add(i as usize)).block_number == (*mt).block_written {
        let entry = e.add(i as usize);
        if block_uncomp_len > 0 && (*entry).offset == block_uncomp_len as u64 {
            let next_block_addr = (*mt).block_address + block_comp_len as u64;
            if hts_idx_push(
                (*mt).hts_idx,
                (*entry).tid,
                (*entry).beg,
                (*entry).end,
                next_block_addr << 16,
                (*entry).is_mapped,
            ) < 0
            {
                libc::pthread_mutex_unlock(&mut (*mt).idx_m);
                return -1;
            }
            i += 1;
            break;
        }

        if hts_idx_push(
            (*mt).hts_idx,
            (*entry).tid,
            (*entry).beg,
            (*entry).end,
            ((*mt).block_address << 16) + (*entry).offset,
            (*entry).is_mapped,
        ) < 0
        {
            libc::pthread_mutex_unlock(&mut (*mt).idx_m);
            return -1;
        }
        i += 1;
    }

    ptr::copy(
        e.add(i as usize),
        e,
        ((*mt).idx_cache.nentries - i) as usize,
    );
    (*mt).idx_cache.nentries -= i;
    (*mt).block_written += 1;

    libc::pthread_mutex_unlock(&mut (*mt).idx_m);

    0
}

// original: bgzf_set_private_data (htslib/bgzf_internal.h:51)
pub unsafe fn bgzf_internal_h_51_bgzf_set_private_data(
    fp: *mut BGZF,
    private_data: *mut c_void,
    fn_: Option<BgzfPrivateDataCleanupFunc>,
) {
    assert!(!(*fp).cache.is_null());
    let cache = (*fp).cache.cast::<bgzf_cache_t>();
    (*cache).private_data = private_data;
    (*cache).private_data_cleanup = fn_;
}

// original: bgzf_clear_private_data (htslib/bgzf_internal.h:58)
pub unsafe fn bgzf_internal_h_58_bgzf_clear_private_data(fp: *mut BGZF) {
    assert!(!(*fp).cache.is_null());
    let cache = (*fp).cache.cast::<bgzf_cache_t>();
    if !(*cache).private_data.is_null() {
        if let Some(cleanup) = (*cache).private_data_cleanup {
            cleanup((*cache).private_data);
        }
        (*cache).private_data = ptr::null_mut();
    }
}

// original: bgzf_get_private_data (htslib/bgzf_internal.h:67)
pub unsafe fn bgzf_internal_h_67_bgzf_get_private_data(fp: *mut BGZF) -> *mut c_void {
    assert!(!(*fp).cache.is_null());
    (*((*fp).cache.cast::<bgzf_cache_t>())).private_data
}

fn flag(fp: *const BGZF, bit: u32) -> bool {
    unsafe { ((*fp).bitfields & (1 << bit)) != 0 }
}

unsafe fn set_flag(fp: *mut BGZF, bit: u32, value: bool) {
    if value {
        (*fp).bitfields |= 1 << bit;
    } else {
        (*fp).bitfields &= !(1 << bit);
    }
}

unsafe fn errcode(fp: *const BGZF) -> u32 {
    (*fp).bitfields & 0xffff
}

unsafe fn add_errcode(fp: *mut BGZF, err: u32) {
    (*fp).bitfields |= err & 0xffff;
}

unsafe fn set_compress_level(fp: *mut BGZF, level: c_int) {
    (*fp).bitfields &= !(0x1ff << 20);
    (*fp).bitfields |= ((level as u32) & 0x1ff) << 20;
}

fn unpack_u16(buf: &[u8]) -> u16 {
    u16::from_le_bytes([buf[0], buf[1]])
}

fn unpack_u32(buf: &[u8]) -> u32 {
    u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]])
}

pub unsafe fn packInt16(buffer: *mut u8, value: u16) {
    *buffer.add(0) = value as u8;
    *buffer.add(1) = (value >> 8) as u8;
}

pub unsafe fn unpackInt16(buffer: *const u8) -> c_int {
    *buffer.add(0) as c_int | ((*buffer.add(1) as c_int) << 8)
}

pub unsafe fn packInt32(buffer: *mut u8, value: u32) {
    *buffer.add(0) = value as u8;
    *buffer.add(1) = (value >> 8) as u8;
    *buffer.add(2) = (value >> 16) as u8;
    *buffer.add(3) = (value >> 24) as u8;
}

pub unsafe fn bgzf_zerr(errnum: c_int, zs: *mut z_stream_s) -> *const c_char {
    if !zs.is_null() && !(*zs).msg.is_null() {
        return (*zs).msg;
    }

    match errnum {
        Z_ERRNO => libc::strerror(*c_compat::__errno_location()),
        Z_STREAM_ERROR => c"invalid parameter/compression level, or inconsistent stream state"
            .as_ptr()
            .cast(),
        Z_DATA_ERROR => c"invalid or incomplete IO".as_ptr().cast(),
        Z_MEM_ERROR => c"out of memory".as_ptr().cast(),
        Z_BUF_ERROR => c"progress temporarily not possible, or in() / out() returned an error"
            .as_ptr()
            .cast(),
        Z_VERSION_ERROR => c"zlib version mismatch".as_ptr().cast(),
        Z_NEED_DICT => c"data was compressed using a dictionary".as_ptr().cast(),
        Z_OK | _ => {
            let buffer = std::ptr::addr_of_mut!(BGZF_ZERR_BUFFER).cast::<c_char>();
            libc::snprintf(buffer, 32, c"[%d] unknown".as_ptr(), errnum);
            buffer.cast()
        }
    }
}

pub unsafe fn check_header(header: *const u8) -> c_int {
    if *header.add(0) != 31 || *header.add(1) != 139 || *header.add(2) != 8 {
        return -2;
    }
    if (*header.add(3) & 4) != 0
        && unpackInt16(header.add(10)) == 6
        && *header.add(12) == b'B'
        && *header.add(13) == b'C'
        && unpackInt16(header.add(14)) == 2
    {
        0
    } else {
        -1
    }
}

fn crc32_update(crc: u32, bytes: &[u8]) -> u32 {
    let mut crc = !crc;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn crc32(bytes: &[u8]) -> u32 {
    crc32_update(0, bytes)
}

pub unsafe fn hts_crc32(crc: u32, buf: *const c_void, len: usize) -> u32 {
    if len == 0 {
        return crc32_update(crc, &[]);
    }
    if let Some(zlib) = system_zlib() {
        let mut crc = crc as c_ulong;
        let mut ptr = buf.cast::<u8>();
        let mut remaining = len;
        while remaining > 0 {
            let chunk = remaining.min(c_uint::MAX as usize);
            crc = (zlib.crc32)(crc, ptr, chunk as c_uint);
            ptr = ptr.add(chunk);
            remaining -= chunk;
        }
        return crc as u32;
    }
    crc32_update(crc, slice::from_raw_parts(buf.cast::<u8>(), len))
}

pub unsafe fn bgzf_c_557_hts_crc32(crc: u32, buf: *const c_void, len: usize) -> u32 {
    hts_crc32(crc, buf, len)
}

pub unsafe fn bgzf_c_620_hts_crc32(crc: u32, buf: *const c_void, len: usize) -> u32 {
    hts_crc32(crc, buf, len)
}

pub unsafe fn bgzf_compress(
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    level: c_int,
) -> c_int {
    let src_bytes = slice::from_raw_parts(src.cast::<u8>(), slen);
    let dst_bytes = slice::from_raw_parts_mut(dst.cast::<u8>(), *dlen);

    let mut store_uncompressed = level == 0;
    let mut total_len = 0usize;
    if !store_uncompressed {
        let Some(zlib) = system_zlib() else {
            return -1;
        };
        if *dlen < BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH {
            return -1;
        }
        let out_len = *dlen - BLOCK_HEADER_LENGTH - BLOCK_FOOTER_LENGTH;
        let mut zs: z_stream = std::mem::zeroed();
        zs.next_in = src.cast::<u8>().cast_mut();
        zs.avail_in = slen as c_uint;
        zs.next_out = dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH);
        zs.avail_out = out_len as c_uint;

        let ret = (zlib.deflate_init2)(
            &mut zs,
            if level < 0 {
                Z_DEFAULT_COMPRESSION
            } else {
                level
            },
            Z_DEFLATED,
            -15,
            8,
            Z_DEFAULT_STRATEGY,
            (zlib.zlib_version)(),
            std::mem::size_of::<z_stream>() as c_int,
        );
        if ret != Z_OK {
            return -1;
        }

        let ret = (zlib.deflate)(&mut zs, Z_FINISH);
        if ret != Z_STREAM_END {
            if ret == Z_OK && zs.avail_out == 0 {
                (zlib.deflate_end)(&mut zs);
                store_uncompressed = true;
            } else {
                (zlib.deflate_end)(&mut zs);
                return -1;
            }
        } else if zs.avail_out == 0 {
            (zlib.deflate_end)(&mut zs);
            store_uncompressed = true;
        } else {
            let compressed_len = zs.total_out as usize;
            if (zlib.deflate_end)(&mut zs) != Z_OK {
                return -1;
            }
            total_len = compressed_len + BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH;
        }
    }

    if store_uncompressed {
        total_len = slen + 5 + BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH;
        if *dlen < total_len {
            return -1;
        }
        *dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH) = 1;
        packInt16(
            dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH + 1),
            slen as u16,
        );
        packInt16(
            dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH + 3),
            !(slen as u16),
        );
        ptr::copy_nonoverlapping(
            src_bytes.as_ptr(),
            dst_bytes.as_mut_ptr().add(BLOCK_HEADER_LENGTH + 5),
            slen,
        );
    }

    dst_bytes[..BLOCK_HEADER_LENGTH]
        .copy_from_slice(&[31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, 66, 67, 2, 0, 0, 0]);
    packInt16(dst_bytes.as_mut_ptr().add(16), (total_len - 1) as u16);
    packInt32(
        dst_bytes.as_mut_ptr().add(total_len - 8),
        hts_crc32(0, src, slen),
    );
    packInt32(dst_bytes.as_mut_ptr().add(total_len - 4), slen as u32);
    *dlen = total_len;
    0
}

pub unsafe fn bgzf_c_561_bgzf_compress(
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    level: c_int,
) -> c_int {
    bgzf_compress(dst, dlen, src, slen, level)
}

pub unsafe fn bgzf_c_624_bgzf_compress(
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    level: c_int,
) -> c_int {
    bgzf_compress(dst, dlen, src, slen, level)
}

pub unsafe fn bgzf_uncompress(
    dst: *mut u8,
    dlen: *mut usize,
    src: *const u8,
    slen: usize,
    expected_crc: u32,
) -> c_int {
    let Some(zlib) = system_zlib() else {
        return -1;
    };

    let mut zs = z_stream {
        next_in: src.cast_mut(),
        avail_in: slen as c_uint,
        total_in: 0,
        next_out: dst,
        avail_out: *dlen as c_uint,
        total_out: 0,
        msg: ptr::null_mut(),
        state: ptr::null_mut(),
        zalloc: None,
        zfree: None,
        opaque: ptr::null_mut(),
        data_type: 0,
        adler: 0,
        reserved: 0,
    };

    if (zlib.inflate_init2)(
        &mut zs,
        -15,
        (zlib.zlib_version)(),
        std::mem::size_of::<z_stream>() as c_int,
    ) != Z_OK
    {
        return -1;
    }

    let inflate_ret = (zlib.inflate)(&mut zs, Z_FINISH);
    let end_ret = (zlib.inflate_end)(&mut zs);
    if inflate_ret != Z_STREAM_END || end_ret != Z_OK {
        return -1;
    }

    *dlen = zs.total_out as usize;
    if hts_crc32(0, dst.cast(), *dlen) != expected_crc {
        return -2;
    }
    0
}

pub unsafe fn bgzf_c_730_bgzf_uncompress(
    dst: *mut u8,
    dlen: *mut usize,
    src: *const u8,
    slen: usize,
    expected_crc: u32,
) -> c_int {
    bgzf_uncompress(dst, dlen, src, slen, expected_crc)
}

pub unsafe fn bgzf_c_762_bgzf_uncompress(
    dst: *mut u8,
    dlen: *mut usize,
    src: *const u8,
    slen: usize,
    expected_crc: u32,
) -> c_int {
    bgzf_uncompress(dst, dlen, src, slen, expected_crc)
}

fn fd_to_ptr(fd: c_int) -> *mut super::hts::hFILE {
    (fd as isize + 1) as usize as *mut super::hts::hFILE
}

fn ptr_to_fd(ptr: *mut super::hts::hFILE) -> c_int {
    (ptr as usize as isize - 1) as c_int
}

fn mode_has(mode: &[u8], needle: u8) -> bool {
    mode.contains(&needle)
}

unsafe fn fd_read(fd: c_int, buffer: *mut c_void, nbytes: usize) -> isize {
    libc::read(fd, buffer, nbytes)
}

unsafe fn fd_write(fd: c_int, buffer: *const c_void, nbytes: usize) -> isize {
    libc::write(fd, buffer, nbytes)
}

unsafe fn fd_seek(fd: c_int, offset: i64, whence: c_int) -> i64 {
    libc::lseek(fd, offset as libc::off_t, whence) as i64
}

unsafe fn fd_tell(fd: c_int) -> i64 {
    fd_seek(fd, 0, libc::SEEK_CUR)
}

fn ptr_is_encoded_fd(ptr: *mut super::hts::hFILE) -> bool {
    (ptr as usize) < 4096
}

unsafe fn bgzf_hread_ptr(fp: *mut super::hts::hFILE, buffer: *mut c_void, nbytes: usize) -> isize {
    if ptr_is_encoded_fd(fp) {
        fd_read(ptr_to_fd(fp), buffer, nbytes)
    } else {
        super::hfile::htslib_hfile_h_247_hread(fp, buffer, nbytes)
    }
}

unsafe fn bgzf_hwrite_ptr(
    fp: *mut super::hts::hFILE,
    buffer: *const c_void,
    nbytes: usize,
) -> isize {
    if ptr_is_encoded_fd(fp) {
        fd_write(ptr_to_fd(fp), buffer, nbytes)
    } else {
        super::hfile::htslib_hfile_h_292_hwrite(fp, buffer, nbytes)
    }
}

unsafe fn bgzf_hseek_ptr(fp: *mut super::hts::hFILE, offset: i64, whence: c_int) -> i64 {
    if ptr_is_encoded_fd(fp) {
        fd_seek(ptr_to_fd(fp), offset, whence)
    } else {
        super::hfile::hseek(fp, offset, whence)
    }
}

unsafe fn bgzf_htell_ptr(fp: *mut super::hts::hFILE) -> i64 {
    bgzf_hseek_ptr(fp, 0, libc::SEEK_CUR)
}

unsafe fn bgzf_hclose_ptr(fp: *mut super::hts::hFILE) -> c_int {
    if ptr_is_encoded_fd(fp) {
        libc::close(ptr_to_fd(fp))
    } else {
        super::hfile::hclose(fp)
    }
}

unsafe fn bgzf_hflush_ptr(fp: *mut super::hts::hFILE) -> c_int {
    if ptr_is_encoded_fd(fp) {
        0
    } else {
        super::hfile::hflush(fp)
    }
}

unsafe fn bgzf_hclearerr_ptr(fp: *mut super::hts::hFILE) {
    if !ptr_is_encoded_fd(fp) {
        super::hfile::htslib_hfile_h_140_hclearerr(fp);
    }
}

unsafe fn bgzf_hpeek_ptr(fp: *mut super::hts::hFILE, buffer: *mut c_void, nbytes: usize) -> isize {
    if ptr_is_encoded_fd(fp) {
        0
    } else {
        super::hfile::hpeek(fp, buffer, nbytes)
    }
}

fn mode2level(mode: &[u8]) -> c_int {
    if mode_has(mode, b'u') {
        return -2;
    }
    mode.iter()
        .copied()
        .find(u8::is_ascii_digit)
        .map(|b| (b - b'0') as c_int)
        .unwrap_or(-1)
}

unsafe fn bgzf_alloc_cache() -> *mut bgzf_cache_t {
    let cache =
        c_compat::calloc(1, std::mem::size_of::<bgzf_cache_t>() as u64).cast::<bgzf_cache_t>();
    if !cache.is_null() {
        (*cache).h = ptr::null_mut();
        (*cache).last_pos = 0;
        (*cache).private_data = ptr::null_mut();
        (*cache).private_data_cleanup = None;
    }
    cache
}

unsafe fn bgzf_free_cache(fp: *mut BGZF) {
    if (*fp).cache.is_null() {
        return;
    }
    bgzf_internal_h_58_bgzf_clear_private_data(fp);
    let cache = (*fp).cache.cast::<bgzf_cache_t>();
    if !(*cache).h.is_null() {
        drop(Box::from_raw((*cache).h.cast::<BgzfBlockCache>()));
        (*cache).h = ptr::null_mut();
    }
    c_compat::free((*fp).cache);
    (*fp).cache = ptr::null_mut();
}

// original: free_cache (htslib/bgzf.c:905)
pub unsafe fn bgzf_c_905_free_cache(fp: *mut BGZF) {
    bgzf_free_cache(fp)
}

unsafe fn bgzf_block_cache(fp: *mut BGZF) -> Option<&'static mut BgzfBlockCache> {
    if fp.is_null() || (*fp).cache.is_null() {
        return None;
    }
    let cache = (*fp).cache.cast::<bgzf_cache_t>();
    if (*cache).h.is_null() {
        let h = Box::new(BgzfBlockCache {
            blocks: HashMap::new(),
            order: Vec::new(),
        });
        (*cache).h = Box::into_raw(h).cast();
    }
    Some(&mut *(*cache).h.cast::<BgzfBlockCache>())
}

// original: load_block_from_cache (htslib/bgzf.c:903)
unsafe fn load_block_from_cache(fp: *mut BGZF, block_address: i64) -> c_int {
    if (*fp).cache.is_null() {
        return 0;
    }
    let cache = (*fp).cache.cast::<bgzf_cache_t>();
    if (*cache).h.is_null() {
        return 0;
    }

    let h = &mut *(*cache).h.cast::<BgzfBlockCache>();
    let Some(entry) = h.blocks.get(&block_address) else {
        return 0;
    };

    if (*fp).block_length != 0 {
        (*fp).block_offset = 0;
    }
    (*fp).block_address = block_address;
    (*fp).block_length = entry.size;
    ptr::copy_nonoverlapping(
        entry.block.as_ptr(),
        (*fp).uncompressed_block.cast::<u8>(),
        entry.size as usize,
    );
    if bgzf_hseek_ptr((*fp).fp, entry.end_offset, SEEK_SET) < 0 {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    entry.size
}

// original: load_block_from_cache (htslib/bgzf.c:918)
pub unsafe fn bgzf_c_918_load_block_from_cache(fp: *mut BGZF, block_address: i64) -> c_int {
    load_block_from_cache(fp, block_address)
}

// original: cache_block (htslib/bgzf.c:925)
unsafe fn cache_block(fp: *mut BGZF, compressed_size: c_int) {
    if (*fp).cache_size <= BGZF_MAX_BLOCK_SIZE as c_int {
        return;
    }
    if (*fp).block_length < 0 || (*fp).block_length > BGZF_MAX_BLOCK_SIZE as c_int {
        return;
    }

    let Some(h) = bgzf_block_cache(fp) else {
        return;
    };

    let max_blocks = ((*fp).cache_size as usize / BGZF_MAX_BLOCK_SIZE).max(1);
    if h.blocks.len() >= max_blocks {
        let mut index = (*(*fp).cache.cast::<bgzf_cache_t>()).last_pos as usize;
        if !h.order.is_empty() {
            index %= h.order.len();
            let key = h.order.remove(index);
            h.blocks.remove(&key);
            (*(*fp).cache.cast::<bgzf_cache_t>()).last_pos = if h.order.is_empty() {
                0
            } else {
                (index % h.order.len()) as c_uint
            };
        }
    }

    let mut block = Box::new([0u8; BGZF_MAX_BLOCK_SIZE]);
    ptr::copy_nonoverlapping(
        (*fp).uncompressed_block.cast::<u8>(),
        block.as_mut_ptr(),
        (*fp).block_length as usize,
    );
    let old = h.blocks.insert(
        (*fp).block_address,
        BgzfCachedBlock {
            size: (*fp).block_length,
            end_offset: (*fp).block_address + compressed_size as i64,
            block,
        },
    );
    if old.is_none() {
        h.order.push((*fp).block_address);
    }
}

// original: cache_block (htslib/bgzf.c:940)
pub unsafe fn bgzf_c_940_cache_block(fp: *mut BGZF, compressed_size: c_int) {
    cache_block(fp, compressed_size)
}

// original: bgzf_gzip_compress (htslib/bgzf.c:686)
pub unsafe fn bgzf_c_686_bgzf_gzip_compress(
    fp: *mut BGZF,
    dst: *mut c_void,
    dlen: *mut usize,
    src: *const c_void,
    slen: usize,
    _level: c_int,
) -> c_int {
    let Some(zlib) = system_zlib() else {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    };
    if (*fp).gz_stream.is_null() {
        let mut stream = Box::new(GzipStream {
            zs: std::mem::zeroed(),
        });
        if gzip_deflate_stream_init(&mut stream.zs, compress_level(fp)) != Z_OK {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
        (*fp).gz_stream = Box::into_raw(stream).cast();
    }

    let stream = &mut *(*fp).gz_stream.cast::<GzipStream>();
    let original_len = *dlen;
    stream.zs.next_in = src.cast::<u8>().cast_mut();
    stream.zs.avail_in = slen as c_uint;
    stream.zs.next_out = dst.cast::<u8>();
    stream.zs.avail_out = original_len as c_uint;

    let ret = (zlib.deflate)(
        &mut stream.zs,
        if slen != 0 { Z_PARTIAL_FLUSH } else { Z_FINISH },
    );
    if ret == Z_STREAM_ERROR {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }
    if stream.zs.avail_in != 0 {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }
    *dlen = original_len - stream.zs.avail_out as usize;
    0
}

unsafe fn gzip_stream_init(zs: *mut z_stream) -> c_int {
    let Some(zlib) = system_zlib() else {
        return Z_STREAM_ERROR;
    };
    (zlib.inflate_init2)(
        zs,
        15 + 16,
        (zlib.zlib_version)(),
        std::mem::size_of::<z_stream>() as c_int,
    )
}

unsafe fn gzip_deflate_stream_init(zs: *mut z_stream, level: c_int) -> c_int {
    let Some(zlib) = system_zlib() else {
        return Z_STREAM_ERROR;
    };
    (zlib.deflate_init2)(
        zs,
        level,
        Z_DEFLATED,
        15 + 16,
        8,
        Z_DEFAULT_STRATEGY,
        (zlib.zlib_version)(),
        std::mem::size_of::<z_stream>() as c_int,
    )
}

unsafe fn gzip_stream_reset(stream: &mut GzipStream) -> c_int {
    let Some(zlib) = system_zlib() else {
        return Z_STREAM_ERROR;
    };
    let next_in = stream.zs.next_in;
    let avail_in = stream.zs.avail_in;
    let next_out = stream.zs.next_out;
    let avail_out = stream.zs.avail_out;
    let ret = (zlib.inflate_reset)(&mut stream.zs);
    if ret == Z_OK {
        stream.zs.next_in = next_in;
        stream.zs.avail_in = avail_in;
        stream.zs.next_out = next_out;
        stream.zs.avail_out = avail_out;
    }
    ret
}

unsafe fn raw_inflate_stream_init(zs: *mut z_stream) -> c_int {
    let Some(zlib) = system_zlib() else {
        return Z_STREAM_ERROR;
    };
    (zlib.inflate_init2)(
        zs,
        -15,
        (zlib.zlib_version)(),
        std::mem::size_of::<z_stream>() as c_int,
    )
}

unsafe fn gzip_stream_end(stream: *mut GzipStream) {
    if let Some(zlib) = system_zlib() {
        (zlib.inflate_end)(&mut (*stream).zs);
    }
}

unsafe fn gzip_deflate_stream_end(stream: *mut GzipStream) {
    if let Some(zlib) = system_zlib() {
        (zlib.deflate_end)(&mut (*stream).zs);
    }
}

unsafe fn bgzf_read_init_hfile(hfp: *mut super::hts::hFILE) -> *mut BGZF {
    let mut magic = [0u8; BLOCK_HEADER_LENGTH];
    let preloads_probe = ptr_is_encoded_fd(hfp);
    let n = if preloads_probe {
        bgzf_hread_ptr(hfp, magic.as_mut_ptr().cast(), magic.len())
    } else {
        super::hfile::hpeek(hfp, magic.as_mut_ptr().cast(), magic.len())
    };
    if n < 0 {
        return ptr::null_mut();
    }

    if n == BLOCK_HEADER_LENGTH as isize
        && magic[0] == 0x1f
        && magic[1] == 0x8b
        && (magic[3] & 4) != 0
        && &magic[12..16] == b"RAZF"
    {
        *c_compat::__errno_location() = c_compat::ENOEXEC;
        return ptr::null_mut();
    }

    let fp = c_compat::calloc(1, std::mem::size_of::<BGZF>() as u64).cast::<BGZF>();
    if fp.is_null() {
        return ptr::null_mut();
    }

    let blocks = c_compat::malloc((2 * BGZF_MAX_BLOCK_SIZE) as u64);
    if blocks.is_null() {
        bgzf_free_without_hclose(fp);
        return ptr::null_mut();
    }
    let cache = bgzf_alloc_cache();
    if cache.is_null() {
        c_compat::free(blocks);
        c_compat::free(fp.cast());
        return ptr::null_mut();
    }

    (*fp).uncompressed_block = blocks;
    (*fp).compressed_block = (blocks.cast::<u8>()).add(BGZF_MAX_BLOCK_SIZE).cast();
    (*fp).cache = cache.cast();
    if preloads_probe && n > 0 {
        let preloaded = n as usize;
        if preloaded == BLOCK_HEADER_LENGTH && magic[0] == 0x1f && magic[1] == 0x8b {
            ptr::copy_nonoverlapping(magic.as_ptr(), (*fp).compressed_block.cast(), preloaded);
        } else {
            ptr::copy_nonoverlapping(magic.as_ptr(), (*fp).uncompressed_block.cast(), preloaded);
        }
        (*fp).block_clength = -(n as c_int);
    }
    set_flag(fp, 17, false);
    set_flag(
        fp,
        30,
        n == BLOCK_HEADER_LENGTH as isize && magic[0] == 0x1f && magic[1] == 0x8b,
    );
    set_flag(
        fp,
        31,
        flag(fp, 30) && !((magic[3] & 4) != 0 && &magic[12..16] == b"BC\x02\x00"),
    );
    (*fp).fp = hfp;
    fp
}

unsafe fn bgzf_read_init(fd: c_int) -> *mut BGZF {
    bgzf_read_init_hfile(fd_to_ptr(fd))
}

unsafe fn bgzf_write_init(fd: c_int, mode: &[u8]) -> *mut BGZF {
    let fp = c_compat::calloc(1, std::mem::size_of::<BGZF>() as u64).cast::<BGZF>();
    if fp.is_null() {
        return ptr::null_mut();
    }
    set_flag(fp, 17, true);
    (*fp).fp = fd_to_ptr(fd);
    let cache = bgzf_alloc_cache();
    if cache.is_null() {
        c_compat::free(fp.cast());
        return ptr::null_mut();
    }
    (*fp).cache = cache.cast();

    let level = mode2level(mode);
    if level == -2 {
        set_flag(fp, 30, false);
        set_compress_level(fp, level);
        return fp;
    }

    let blocks = c_compat::malloc((2 * BGZF_MAX_BLOCK_SIZE) as u64);
    if blocks.is_null() {
        c_compat::free(fp.cast());
        return ptr::null_mut();
    }
    (*fp).uncompressed_block = blocks;
    (*fp).compressed_block = (blocks.cast::<u8>()).add(BGZF_MAX_BLOCK_SIZE).cast();
    set_flag(fp, 30, true);
    set_compress_level(
        fp,
        if level < 0 || level > 9 {
            Z_DEFAULT_COMPRESSION
        } else {
            level
        },
    );
    if mode_has(mode, b'g') {
        let mut stream = Box::new(GzipStream {
            zs: std::mem::zeroed(),
        });
        if gzip_deflate_stream_init(&mut stream.zs, compress_level(fp)) != Z_OK {
            drop(stream);
            c_compat::free((*fp).uncompressed_block);
            c_compat::free((*fp).cache);
            c_compat::free(fp.cast());
            return ptr::null_mut();
        }
        (*fp).gz_stream = Box::into_raw(stream).cast();
    }
    fp
}

unsafe fn bgzf_free_without_hclose(fp: *mut BGZF) {
    if fp.is_null() {
        return;
    }
    bgzf_index_destroy(fp);
    bgzf_free_cache(fp);
    if !(*fp).gz_stream.is_null() {
        let stream = (*fp).gz_stream.cast::<GzipStream>();
        if flag(fp, 17) && flag(fp, 31) {
            gzip_deflate_stream_end(stream);
        } else {
            gzip_stream_end(stream);
        }
        drop(Box::from_raw(stream));
        (*fp).gz_stream = ptr::null_mut();
    }
    c_compat::free((*fp).uncompressed_block);
    c_compat::free(fp.cast());
}

unsafe fn compress_level(fp: *const BGZF) -> c_int {
    let raw = (((*fp).bitfields >> 20) & 0x1ff) as i32;
    if raw & 0x100 != 0 {
        raw | !0x1ff
    } else {
        raw
    }
}

unsafe fn deflate_block(fp: *mut BGZF, block_length: usize) -> c_int {
    let mut comp_size = BGZF_MAX_BLOCK_SIZE;
    let ret = bgzf_compress(
        (*fp).compressed_block,
        &mut comp_size,
        (*fp).uncompressed_block,
        block_length,
        compress_level(fp),
    );
    if ret != 0 {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }
    (*fp).block_offset = 0;
    comp_size as c_int
}

pub unsafe fn bgzf_flush(fp: *mut BGZF) -> c_int {
    if !flag(fp, 17) {
        return 0;
    }
    if !flag(fp, 30) {
        return bgzf_hflush_ptr((*fp).fp);
    }
    if flag(fp, 31) {
        if (*fp).block_offset > 0 && bgzf_gzip_flush(fp, Z_SYNC_FLUSH) != 0 {
            return -1;
        }
        return bgzf_hflush_ptr((*fp).fp);
    }
    if !(*fp).mt.is_null() {
        if bgzf_c_1852_mt_queue(fp) != 0 || bgzf_c_1897_mt_flush_queue(fp) != 0 {
            return -1;
        }
        return bgzf_hflush_ptr((*fp).fp);
    }
    while (*fp).block_offset > 0 {
        if (*fp).idx_build_otf != 0 {
            bgzf_index_add_block(fp);
            (*(*fp).idx.cast::<bgzidx_t>()).ublock_addr += (*fp).block_offset as u64;
        }
        let block_length = deflate_block(fp, (*fp).block_offset as usize);
        if block_length < 0 {
            return -1;
        }
        let written = bgzf_hwrite_ptr((*fp).fp, (*fp).compressed_block, block_length as usize);
        if written != block_length as isize {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        (*fp).block_address += block_length as i64;
    }
    0
}

unsafe fn bgzf_gzip_flush(fp: *mut BGZF, flush: c_int) -> c_int {
    let Some(zlib) = system_zlib() else {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    };
    if (*fp).gz_stream.is_null() {
        let mut stream = Box::new(GzipStream {
            zs: std::mem::zeroed(),
        });
        if gzip_deflate_stream_init(&mut stream.zs, compress_level(fp)) != Z_OK {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
        (*fp).gz_stream = Box::into_raw(stream).cast();
    }

    let stream = &mut *(*fp).gz_stream.cast::<GzipStream>();
    stream.zs.next_in = (*fp).uncompressed_block.cast();
    stream.zs.avail_in = (*fp).block_offset as c_uint;

    loop {
        stream.zs.next_out = (*fp).compressed_block.cast();
        stream.zs.avail_out = BGZF_MAX_BLOCK_SIZE as c_uint;
        let ret = (zlib.deflate)(&mut stream.zs, flush);
        if ret != Z_OK && !(flush == Z_FINISH && ret == Z_STREAM_END) {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
        let have = BGZF_MAX_BLOCK_SIZE - stream.zs.avail_out as usize;
        if have != 0 && bgzf_hwrite_ptr((*fp).fp, (*fp).compressed_block, have) != have as isize {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        if stream.zs.avail_out != 0 || ret == Z_STREAM_END {
            break;
        }
    }

    (*fp).block_address += (*fp).block_offset as i64;
    (*fp).block_offset = 0;
    0
}

unsafe fn bgzf_htell(fp: *mut BGZF) -> i64 {
    if !flag(fp, 30) {
        (*fp).block_address + (*fp).block_offset as i64
    } else {
        bgzf_htell_ptr((*fp).fp)
    }
}

unsafe fn bgzf_block_end_address(fp: *mut BGZF) -> i64 {
    if flag(fp, 30) && !flag(fp, 31) && (*fp).block_clength > 0 {
        (*fp).block_address + (*fp).block_clength as i64
    } else {
        bgzf_htell(fp)
    }
}

pub unsafe fn bgzf_c_315_razf_info(hfp: *mut super::hts::hFILE, mut filename: *const c_char) {
    if filename.is_null() || libc::strcmp(filename, c"-".as_ptr()) == 0 {
        filename = c"FILE".as_ptr();
    }

    let sizes_pos = super::hfile::hseek(hfp, -16, libc::SEEK_END);
    if sizes_pos >= 0 {
        let mut usize = 0_u64;
        let mut csize = 0_u64;
        if super::hfile::htslib_hfile_h_247_hread(
            hfp,
            (&mut usize as *mut u64).cast(),
            std::mem::size_of::<u64>(),
        ) == std::mem::size_of::<u64>() as isize
            && super::hfile::htslib_hfile_h_247_hread(
                hfp,
                (&mut csize as *mut u64).cast(),
                std::mem::size_of::<u64>(),
            ) == std::mem::size_of::<u64>() as isize
        {
            if ed_is_big() == 0 {
                usize = ed_swap_8(usize);
                csize = ed_swap_8(csize);
            }
            if csize < sizes_pos as u64 {
                let fname = bgzf_filename_lossy(filename);
                let msg = std::ffi::CString::new(format!(
                    "To decompress this file, use the following commands:\n    truncate -s {csize} {fname}\n    gunzip {fname}\nThe resulting uncompressed file should be {usize} bytes in length.\nIf you do not have a truncate command, skip that step (though gunzip will\nlikely produce a \"trailing garbage ignored\" message, which can be ignored)."
                ))
                .unwrap_or_default();
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_ERROR,
                    c"bgzf".as_ptr(),
                    msg.as_ptr(),
                );
                return;
            }
        }
    }

    let fname = bgzf_filename_lossy(filename);
    let msg = std::ffi::CString::new(format!(
        "To decompress this file, use the following command:\n    gunzip {fname}\nThis will likely produce a \"trailing garbage ignored\" message, which can\nusually be safely ignored."
    ))
    .unwrap_or_default();
    crate::htslib_rs::hts::hts_log_cstr(
        crate::htslib_rs::hts::HTS_LOG_ERROR,
        c"bgzf".as_ptr(),
        msg.as_ptr(),
    );
}

// Renders a possibly-null C filename for log messages, matching printf's
// tolerance of NULL (`%s` of NULL is rendered as an empty string here).
unsafe fn bgzf_filename_lossy(filename: *const c_char) -> std::borrow::Cow<'static, str> {
    if filename.is_null() {
        std::borrow::Cow::Borrowed("")
    } else {
        std::borrow::Cow::Owned(
            std::ffi::CStr::from_ptr(filename)
                .to_string_lossy()
                .into_owned(),
        )
    }
}

unsafe fn inflate_block(fp: *mut BGZF, block_length: usize) -> c_int {
    if block_length < BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH {
        add_errcode(fp, BGZF_ERR_HEADER);
        return -1;
    }
    let compressed = slice::from_raw_parts((*fp).compressed_block.cast::<u8>(), block_length);
    let payload_end = block_length - BLOCK_FOOTER_LENGTH;
    let expected_crc = unpack_u32(&compressed[payload_end..payload_end + 4]);
    let Some(zlib) = system_zlib() else {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    };

    if (*fp).gz_stream.is_null() {
        let mut stream = Box::new(GzipStream {
            zs: std::mem::zeroed(),
        });
        if raw_inflate_stream_init(&mut stream.zs) != Z_OK {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
        (*fp).gz_stream = Box::into_raw(stream).cast();
    } else if (zlib.inflate_reset)(&mut (*(*fp).gz_stream.cast::<GzipStream>()).zs) != Z_OK {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }

    let stream = &mut *(*fp).gz_stream.cast::<GzipStream>();
    stream.zs.next_in = (*fp).compressed_block.cast::<u8>().add(BLOCK_HEADER_LENGTH);
    stream.zs.avail_in = (block_length - BLOCK_HEADER_LENGTH) as c_uint;
    stream.zs.next_out = (*fp).uncompressed_block.cast();
    stream.zs.avail_out = BGZF_MAX_BLOCK_SIZE as c_uint;

    if (zlib.inflate)(&mut stream.zs, Z_FINISH) != Z_STREAM_END {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }

    let written = stream.zs.total_out as usize;
    if hts_crc32(0, (*fp).uncompressed_block, written) != expected_crc {
        add_errcode(fp, BGZF_ERR_CRC);
        return -1;
    }
    written as c_int
}

unsafe fn inflate_gzip_block(fp: *mut BGZF) -> c_int {
    if (*fp).gz_stream.is_null() {
        let mut stream = Box::new(GzipStream {
            zs: std::mem::zeroed(),
        });
        if gzip_stream_init(&mut stream.zs) != Z_OK {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
        (*fp).gz_stream = Box::into_raw(stream).cast();
    }

    let Some(zlib) = system_zlib() else {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    };
    let stream = &mut *(*fp).gz_stream.cast::<GzipStream>();
    let mut input_eof = false;
    stream.zs.next_out = (*fp)
        .uncompressed_block
        .cast::<u8>()
        .add((*fp).block_offset as usize);
    stream.zs.avail_out = (BGZF_MAX_BLOCK_SIZE - (*fp).block_offset as usize) as c_uint;

    while stream.zs.avail_out != 0 {
        if !input_eof && stream.zs.avail_in == 0 {
            let (n, from_preload) = if (*fp).block_clength < 0 {
                let preloaded = (-(*fp).block_clength) as usize;
                (*fp).block_clength = 0;
                (preloaded as isize, true)
            } else {
                (
                    bgzf_hread_ptr((*fp).fp, (*fp).compressed_block, BGZF_BLOCK_SIZE),
                    false,
                )
            };
            if n < 0 {
                add_errcode(fp, BGZF_ERR_IO);
                return n as c_int;
            }
            stream.zs.next_in = (*fp).compressed_block.cast::<u8>();
            stream.zs.avail_in = n as c_uint;
            if !from_preload && stream.zs.avail_in < BGZF_BLOCK_SIZE as c_uint {
                input_eof = true;
            }
        }

        let ret = (zlib.inflate)(&mut stream.zs, Z_SYNC_FLUSH);

        match ret {
            Z_STREAM_END => {
                let mut peeked = 0u8;
                let has_more_input = stream.zs.avail_in > 0
                    || bgzf_hpeek_ptr((*fp).fp, (&mut peeked as *mut u8).cast(), 1) == 1;
                if has_more_input {
                    if gzip_stream_reset(stream) != Z_OK {
                        add_errcode(fp, BGZF_ERR_ZLIB);
                        return -1;
                    }
                    continue;
                }
                break;
            }
            Z_OK | Z_BUF_ERROR => {
                if ret == Z_BUF_ERROR && input_eof && stream.zs.avail_out > 0 {
                    add_errcode(fp, BGZF_ERR_IO);
                    return -1;
                }
            }
            _ => {
                add_errcode(fp, BGZF_ERR_ZLIB);
                return -1;
            }
        }
    }

    (BGZF_MAX_BLOCK_SIZE - stream.zs.avail_out as usize) as c_int
}

// original: inflate_gzip_block (htslib/bgzf.c:829)
pub unsafe fn bgzf_c_829_inflate_gzip_block(fp: *mut BGZF) -> c_int {
    inflate_gzip_block(fp)
}

unsafe fn bgzf_read_block(fp: *mut BGZF) -> c_int {
    if errcode(fp) != 0 {
        return -1;
    }
    let mut header = [0u8; BLOCK_HEADER_LENGTH];
    let mut block_address = if (*fp).block_clength < 0 {
        (*fp).block_address
    } else {
        bgzf_htell(fp)
    };

    if !flag(fp, 30) {
        let preloaded = if (*fp).block_clength < 0 {
            (-(*fp).block_clength) as usize
        } else {
            0
        };
        let read = bgzf_hread_ptr(
            (*fp).fp,
            (*fp).uncompressed_block.cast::<u8>().add(preloaded).cast(),
            BGZF_MAX_BLOCK_SIZE - preloaded,
        );
        (*fp).block_clength = 0;
        if read < 0 {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        let count = preloaded as isize + read;
        if count == 0 {
            (*fp).block_length = 0;
            return 0;
        }
        if (*fp).block_length != 0 {
            (*fp).block_offset = 0;
        }
        (*fp).block_address = block_address;
        (*fp).block_length = count as c_int;
        return 0;
    }

    if flag(fp, 31) {
        let count = inflate_gzip_block(fp);
        if count < 0 {
            return -1;
        }
        if (*fp).block_length != 0 {
            (*fp).block_offset = 0;
        }
        (*fp).block_length = count;
        (*fp).block_address = block_address;
        if (*fp).idx_build_otf != 0 {
            return -1;
        }
        return 0;
    }

    if (*fp).cache_size != 0 {
        let cached = load_block_from_cache(fp, block_address);
        if cached < 0 {
            return -1;
        }
        if cached != 0 {
            return 0;
        }
    }

    loop {
        let count = if (*fp).block_clength < 0 {
            let preloaded = (-(*fp).block_clength) as usize;
            (*fp).block_clength = 0;
            if preloaded == header.len() {
                ptr::copy_nonoverlapping(
                    (*fp).compressed_block.cast::<u8>(),
                    header.as_mut_ptr(),
                    header.len(),
                );
            }
            preloaded as isize
        } else {
            bgzf_hread_ptr((*fp).fp, header.as_mut_ptr().cast(), header.len())
        };
        if count == 0 {
            if !flag(fp, 29) && !flag(fp, 18) {
                set_flag(fp, 18, true);
            }
            (*fp).block_length = 0;
            return 0;
        }
        let ret = check_header(header.as_ptr());
        if count != header.len() as isize || ret != 0 {
            add_errcode(fp, BGZF_ERR_HEADER);
            return -1;
        }
        let block_length = unpack_u16(&header[16..18]) as usize + 1;
        if !(BLOCK_HEADER_LENGTH..=BGZF_MAX_BLOCK_SIZE).contains(&block_length) {
            add_errcode(fp, BGZF_ERR_HEADER);
            return -1;
        }
        ptr::copy_nonoverlapping(
            header.as_ptr(),
            (*fp).compressed_block.cast::<u8>(),
            BLOCK_HEADER_LENGTH,
        );
        let remaining = block_length - BLOCK_HEADER_LENGTH;
        let read = bgzf_hread_ptr(
            (*fp).fp,
            (*fp)
                .compressed_block
                .cast::<u8>()
                .add(BLOCK_HEADER_LENGTH)
                .cast(),
            remaining,
        );
        if read != remaining as isize {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        let count = inflate_block(fp, block_length);
        if count < 0 {
            return -1;
        }
        set_flag(fp, 29, count == 0);
        if count != 0 {
            if (*fp).block_length != 0 {
                (*fp).block_offset = 0;
            }
            (*fp).block_address = block_address;
            (*fp).block_clength = block_length as c_int;
            (*fp).block_length = count;
            if (*fp).idx_build_otf != 0 {
                bgzf_index_add_block(fp);
                (*(*fp).idx.cast::<bgzidx_t>()).ublock_addr += count as u64;
            }
            cache_block(fp, block_length as c_int);
            return 0;
        }
        block_address += block_length as i64;
    }
}

pub unsafe fn bgzf_c_1390_bgzf_nul_func(arg: *mut c_void) -> *mut c_void {
    arg
}

// original: job_cleanup (htslib/bgzf.c:1322)
pub unsafe extern "C" fn bgzf_c_1322_job_cleanup(arg: *mut c_void) {
    let job = arg.cast::<bgzf_job>();
    let mt = (*(*job).fp).mt.cast::<bgzf_mtaux_t>();

    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).job_pool_m));
    super::cram::cram_pooled_alloc_c_144_pool_free((*mt).job_pool.cast(), job.cast());
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).job_pool_m));
}

unsafe fn bgzf_encode_job(job: *mut bgzf_job, level: c_int) -> *mut c_void {
    if job.is_null() {
        return ptr::null_mut();
    }

    (*job).errcode = 0;
    (*job).comp_len = BGZF_MAX_BLOCK_SIZE;
    if bgzf_compress(
        (*job).comp_data.as_mut_ptr().cast(),
        ptr::addr_of_mut!((*job).comp_len),
        (*job).uncomp_data.as_ptr().cast(),
        (*job).uncomp_len,
        level,
    ) != 0
    {
        (*job).errcode = BGZF_ERR_ZLIB as c_int;
    }
    job.cast()
}

// original: bgzf_encode_func (htslib/bgzf.c:1330)
pub unsafe extern "C" fn bgzf_encode_func(arg: *mut c_void) -> *mut c_void {
    let job = arg.cast::<bgzf_job>();
    if job.is_null() {
        return ptr::null_mut();
    }
    let level = if (*job).fp.is_null() {
        Z_DEFAULT_COMPRESSION
    } else {
        compress_level((*job).fp)
    };
    bgzf_encode_job(job, level)
}

// original: bgzf_encode_level0_func (htslib/bgzf.c:1345)
pub unsafe extern "C" fn bgzf_encode_level0_func(arg: *mut c_void) -> *mut c_void {
    bgzf_encode_job(arg.cast::<bgzf_job>(), 0)
}

// original: bgzf_decode_func (htslib/bgzf.c:1373)
pub unsafe extern "C" fn bgzf_decode_func(arg: *mut c_void) -> *mut c_void {
    let job = arg.cast::<bgzf_job>();
    if job.is_null() {
        return ptr::null_mut();
    }

    (*job).errcode = 0;
    (*job).uncomp_len = 0;
    if (*job).comp_len < BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH
        || check_header((*job).comp_data.as_ptr()) != 0
    {
        (*job).errcode = BGZF_ERR_HEADER as c_int;
        return job.cast();
    }

    let comp_data = &(*job).comp_data;
    let block_len = unpack_u16(&comp_data[16..18]) as usize + 1;
    if block_len > (*job).comp_len
        || !(BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH..=BGZF_MAX_BLOCK_SIZE).contains(&block_len)
    {
        (*job).errcode = BGZF_ERR_HEADER as c_int;
        return job.cast();
    }

    let payload_end = block_len - BLOCK_FOOTER_LENGTH;
    let expected_crc = unpack_u32(&comp_data[payload_end..payload_end + 4]);
    let mut out_len = BGZF_MAX_BLOCK_SIZE;
    match bgzf_uncompress(
        (*job).uncomp_data.as_mut_ptr(),
        &mut out_len,
        (*job).comp_data.as_ptr().add(BLOCK_HEADER_LENGTH),
        block_len - BLOCK_HEADER_LENGTH,
        expected_crc,
    ) {
        0 => (*job).uncomp_len = out_len,
        -2 => (*job).errcode = BGZF_ERR_CRC as c_int,
        _ => (*job).errcode = BGZF_ERR_ZLIB as c_int,
    }
    job.cast()
}

unsafe fn bgzf_mt_alloc_job(fp: *mut BGZF) -> *mut bgzf_job {
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    if mt.is_null() || (*mt).job_pool.is_null() {
        return ptr::null_mut();
    }

    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).job_pool_m));
    let job =
        super::cram::cram_pooled_alloc_c_115_pool_alloc((*mt).job_pool.cast()).cast::<bgzf_job>();
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).job_pool_m));
    if !job.is_null() {
        ptr::write_bytes(job, 0, 1);
        (*job).fp = fp;
    }
    job
}

unsafe fn bgzf_mt_free_job(fp: *mut BGZF, job: *mut bgzf_job) {
    if job.is_null() {
        return;
    }
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    if mt.is_null() || (*mt).job_pool.is_null() {
        return;
    }
    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).job_pool_m));
    super::cram::cram_pooled_alloc_c_144_pool_free((*mt).job_pool.cast(), job.cast());
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).job_pool_m));
}

unsafe fn bgzf_mt_dispatch(
    fp: *mut BGZF,
    job: *mut bgzf_job,
    worker: super::thread_pool::hts_tpool_worker,
) -> c_int {
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    if mt.is_null() || (*mt).pool.is_null() || (*mt).out_queue.is_null() || job.is_null() {
        return -1;
    }
    if super::thread_pool::hts_tpool_dispatch(
        (*mt).pool.cast(),
        (*mt).out_queue.cast(),
        worker,
        job.cast(),
    ) != 0
    {
        return -1;
    }
    (*mt).jobs_pending += 1;
    0
}

// original: bgzf_mt_writer (htslib/bgzf.c:1398)
pub extern "C" fn bgzf_c_1398_bgzf_mt_writer(arg: *mut c_void) -> *mut c_void {
    unsafe { bgzf_mt_writer_impl(arg) }
}

unsafe fn bgzf_mt_writer_impl(arg: *mut c_void) -> *mut c_void {
    let fp = arg.cast::<BGZF>();
    if fp.is_null() || (*fp).mt.is_null() {
        return ptr::null_mut();
    }
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();

    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).command_m));
    while (*mt).command != mtaux_cmd::CLOSE {
        libc::pthread_cond_wait(
            ptr::addr_of_mut!((*mt).command_c),
            ptr::addr_of_mut!((*mt).command_m),
        );
    }
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).command_m));
    ptr::null_mut()
}

// original: bgzf_mt_read_block (htslib/bgzf.c:1485)
pub unsafe fn bgzf_c_1485_bgzf_mt_read_block(fp: *mut BGZF) -> c_int {
    if fp.is_null() || (*fp).mt.is_null() {
        return bgzf_read_block(fp);
    }
    if errcode(fp) != 0 {
        return -1;
    }

    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    let block_address = if (*fp).block_clength < 0 {
        (*fp).block_address
    } else {
        bgzf_htell(fp)
    };
    let mut header = [0u8; BLOCK_HEADER_LENGTH];
    let count = if (*fp).block_clength < 0 {
        let preloaded = (-(*fp).block_clength) as usize;
        (*fp).block_clength = 0;
        if preloaded == BLOCK_HEADER_LENGTH {
            ptr::copy_nonoverlapping(
                (*fp).compressed_block.cast::<u8>(),
                header.as_mut_ptr(),
                header.len(),
            );
        }
        preloaded as isize
    } else {
        bgzf_hread_ptr((*fp).fp, header.as_mut_ptr().cast(), header.len())
    };

    if count == 0 {
        if !flag(fp, 29) && !flag(fp, 18) {
            set_flag(fp, 18, true);
        }
        (*fp).block_length = 0;
        (*mt).hit_eof = 1;
        return 0;
    }
    if count != header.len() as isize || check_header(header.as_ptr()) != 0 {
        add_errcode(fp, BGZF_ERR_HEADER);
        (*mt).errcode = BGZF_ERR_HEADER as c_int;
        return -1;
    }

    let block_length = unpack_u16(&header[16..18]) as usize + 1;
    if !(BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH..=BGZF_MAX_BLOCK_SIZE).contains(&block_length) {
        add_errcode(fp, BGZF_ERR_HEADER);
        (*mt).errcode = BGZF_ERR_HEADER as c_int;
        return -1;
    }

    let job = bgzf_mt_alloc_job(fp);
    if job.is_null() {
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }
    ptr::copy_nonoverlapping(header.as_ptr(), (*job).comp_data.as_mut_ptr(), header.len());
    let remaining = block_length - BLOCK_HEADER_LENGTH;
    let read = bgzf_hread_ptr(
        (*fp).fp,
        (*job)
            .comp_data
            .as_mut_ptr()
            .add(BLOCK_HEADER_LENGTH)
            .cast(),
        remaining,
    );
    if read != remaining as isize {
        bgzf_mt_free_job(fp, job);
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }
    (*job).comp_len = block_length;
    (*job).block_address = block_address;

    if bgzf_mt_dispatch(fp, job, Some(bgzf_decode_func)) != 0 {
        bgzf_mt_free_job(fp, job);
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }

    let result = super::thread_pool::hts_tpool_next_result_wait((*mt).out_queue.cast());
    if result.is_null() {
        (*mt).jobs_pending -= 1;
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }
    let job = super::thread_pool::hts_tpool_result_data(result).cast::<bgzf_job>();
    super::thread_pool::hts_tpool_delete_result(result, 0);
    (*mt).jobs_pending -= 1;

    if (*job).errcode != 0 {
        add_errcode(fp, (*job).errcode as u32);
        (*mt).errcode = (*job).errcode;
        bgzf_mt_free_job(fp, job);
        return -1;
    }

    if (*fp).block_length != 0 {
        (*fp).block_offset = 0;
    }
    ptr::copy_nonoverlapping(
        (*job).uncomp_data.as_ptr(),
        (*fp).uncompressed_block.cast(),
        (*job).uncomp_len,
    );
    (*fp).block_address = (*job).block_address;
    (*fp).block_clength = (*job).comp_len as c_int;
    (*fp).block_length = (*job).uncomp_len as c_int;
    set_flag(fp, 29, (*job).uncomp_len == 0);

    if (*job).uncomp_len != 0 && (*fp).idx_build_otf != 0 {
        bgzf_index_add_block(fp);
        (*(*fp).idx.cast::<bgzidx_t>()).ublock_addr += (*job).uncomp_len as u64;
    }

    bgzf_mt_free_job(fp, job);
    0
}

// original: bgzf_mt_reader (htslib/bgzf.c:1598)
pub extern "C" fn bgzf_c_1598_bgzf_mt_reader(arg: *mut c_void) -> *mut c_void {
    unsafe { bgzf_mt_reader_impl(arg) }
}

unsafe fn bgzf_mt_reader_impl(arg: *mut c_void) -> *mut c_void {
    let fp = arg.cast::<BGZF>();
    if fp.is_null() || (*fp).mt.is_null() {
        return ptr::null_mut();
    }
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();

    loop {
        libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).command_m));
        while (*mt).command == mtaux_cmd::NONE {
            libc::pthread_cond_wait(
                ptr::addr_of_mut!((*mt).command_c),
                ptr::addr_of_mut!((*mt).command_m),
            );
        }
        let command = (*mt).command;
        libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).command_m));

        match command {
            mtaux_cmd::SEEK => bgzf_c_1583_bgzf_mt_seek(fp),
            mtaux_cmd::HAS_EOF => bgzf_c_1566_bgzf_mt_eof(fp),
            mtaux_cmd::CLOSE => break,
            mtaux_cmd::SEEK_DONE | mtaux_cmd::HAS_EOF_DONE | mtaux_cmd::NONE => {
                libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).command_m));
                if (*mt).command == command {
                    (*mt).command = mtaux_cmd::NONE;
                }
                libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).command_m));
            }
        }
    }

    ptr::null_mut()
}

// original: mt_destroy (htslib/bgzf.c:1805)
pub unsafe fn bgzf_c_1805_mt_destroy(mt: *mut bgzf_mtaux_t) -> c_int {
    let mut ret: c_int;

    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).command_m));
    (*mt).command = mtaux_cmd::CLOSE;
    libc::pthread_cond_signal(ptr::addr_of_mut!((*mt).command_c));
    super::thread_pool::hts_tpool_wake_dispatch((*mt).out_queue.cast());
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).command_m));

    ret =
        -((super::thread_pool::hts_tpool_process_is_shutdown((*mt).out_queue.cast()) > 1) as c_int);
    super::thread_pool::hts_tpool_process_destroy((*mt).out_queue.cast());

    let mut retval: *mut c_void = ptr::null_mut();
    libc::pthread_join((*mt).io_task, ptr::addr_of_mut!(retval));
    if !retval.is_null() {
        ret = -1;
    }

    libc::pthread_mutex_destroy(ptr::addr_of_mut!((*mt).job_pool_m));
    libc::pthread_mutex_destroy(ptr::addr_of_mut!((*mt).command_m));
    libc::pthread_mutex_destroy(ptr::addr_of_mut!((*mt).idx_m));
    libc::pthread_cond_destroy(ptr::addr_of_mut!((*mt).command_c));

    if !(*mt).curr_job.is_null() {
        super::cram::cram_pooled_alloc_c_144_pool_free(
            (*mt).job_pool.cast(),
            (*mt).curr_job.cast(),
        );
    }

    if (*mt).own_pool != 0 {
        super::thread_pool::hts_tpool_destroy((*mt).pool.cast());
    }

    super::cram::cram_pooled_alloc_c_84_pool_destroy((*mt).job_pool.cast());

    if !(*mt).idx_cache.e.is_null() {
        c_compat::free((*mt).idx_cache.e.cast());
    }

    c_compat::free(mt.cast());
    // fflush(NULL) flushes all open output streams (incl. stderr) without
    // referencing the C library's stderr global.
    libc::fflush(std::ptr::null_mut());

    ret
}

// original: mt_queue (htslib/bgzf.c:1852)
pub unsafe fn bgzf_c_1852_mt_queue(fp: *mut BGZF) -> c_int {
    if fp.is_null() || (*fp).mt.is_null() {
        return bgzf_flush(fp);
    }
    if !flag(fp, 17) || !flag(fp, 30) || flag(fp, 31) || (*fp).block_offset <= 0 {
        return 0;
    }

    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    let job = bgzf_mt_alloc_job(fp);
    if job.is_null() {
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }

    let block_len = (*fp).block_offset as usize;
    if (*fp).idx_build_otf != 0 {
        bgzf_index_add_block(fp);
        (*(*fp).idx.cast::<bgzidx_t>()).ublock_addr += block_len as u64;
    }
    (*job).uncomp_len = block_len;
    (*job).block_address = (*mt).block_number as i64;
    ptr::copy_nonoverlapping(
        (*fp).uncompressed_block.cast::<u8>(),
        (*job).uncomp_data.as_mut_ptr(),
        block_len,
    );

    let worker: super::thread_pool::hts_tpool_worker = if compress_level(fp) == 0 {
        Some(bgzf_encode_level0_func)
    } else {
        Some(bgzf_encode_func)
    };
    if bgzf_mt_dispatch(fp, job, worker) != 0 {
        bgzf_mt_free_job(fp, job);
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }
    (*mt).block_number += 1;
    (*fp).block_offset = 0;
    0
}

// original: mt_flush_queue (htslib/bgzf.c:1897)
pub unsafe fn bgzf_c_1897_mt_flush_queue(fp: *mut BGZF) -> c_int {
    if fp.is_null() || (*fp).mt.is_null() {
        return 0;
    }
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    if (*mt).out_queue.is_null() {
        return -1;
    }

    if super::thread_pool::hts_tpool_process_flush((*mt).out_queue.cast()) != 0 {
        add_errcode(fp, BGZF_ERR_IO);
        (*mt).errcode = BGZF_ERR_IO as c_int;
        return -1;
    }

    while (*mt).jobs_pending > 0 {
        let result = super::thread_pool::hts_tpool_next_result_wait((*mt).out_queue.cast());
        if result.is_null() {
            add_errcode(fp, BGZF_ERR_IO);
            (*mt).errcode = BGZF_ERR_IO as c_int;
            return -1;
        }
        let job = super::thread_pool::hts_tpool_result_data(result).cast::<bgzf_job>();
        super::thread_pool::hts_tpool_delete_result(result, 0);
        (*mt).jobs_pending -= 1;

        if job.is_null() || (*job).errcode != 0 {
            let err = if job.is_null() {
                BGZF_ERR_IO as c_int
            } else {
                (*job).errcode
            };
            if !job.is_null() {
                bgzf_mt_free_job(fp, job);
            }
            add_errcode(fp, err as u32);
            (*mt).errcode = err;
            return -1;
        }

        let written = bgzf_hwrite_ptr((*fp).fp, (*job).comp_data.as_ptr().cast(), (*job).comp_len);
        if written != (*job).comp_len as isize {
            bgzf_mt_free_job(fp, job);
            add_errcode(fp, BGZF_ERR_IO);
            (*mt).errcode = BGZF_ERR_IO as c_int;
            return -1;
        }
        if (*fp).idx_build_otf != 0
            && bgzf_c_228_bgzf_idx_flush(fp, (*job).uncomp_len, (*job).comp_len) < 0
        {
            bgzf_mt_free_job(fp, job);
            add_errcode(fp, BGZF_ERR_IO);
            (*mt).errcode = BGZF_ERR_IO as c_int;
            return -1;
        }
        (*mt).block_address += (*job).comp_len as u64;
        (*fp).block_address = (*mt).block_address as i64;
        bgzf_mt_free_job(fp, job);
    }
    0
}

// original: bgzf_close_mt (htslib/bgzf.c:2071)
pub unsafe fn bgzf_c_2071_bgzf_close_mt(fp: *mut BGZF) -> c_int {
    if fp.is_null() || (*fp).mt.is_null() {
        return 0;
    }
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();
    let mut ret = 0;

    if (*fp).block_offset > 0 && bgzf_c_1852_mt_queue(fp) != 0 {
        ret = -1;
    }
    if bgzf_c_1897_mt_flush_queue(fp) != 0 {
        ret = -1;
    }

    set_compress_level(fp, Z_DEFAULT_COMPRESSION);
    let block_length = deflate_block(fp, 0);
    if block_length < 0
        || bgzf_hwrite_ptr((*fp).fp, (*fp).compressed_block, block_length as usize)
            != block_length as isize
        || bgzf_hflush_ptr((*fp).fp) != 0
    {
        add_errcode(fp, BGZF_ERR_IO);
        ret = -1;
    } else {
        (*mt).block_address += block_length as u64;
        (*fp).block_address = (*mt).block_address as i64;
    }

    if bgzf_c_1805_mt_destroy(mt) != 0 {
        ret = -1;
    }
    (*fp).mt = ptr::null_mut();
    ret
}

pub unsafe fn bgzf_open(path: *const c_char, mode: *const c_char) -> *mut BGZF {
    if path.is_null() || mode.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return ptr::null_mut();
    }
    let mode_bytes = CStr::from_ptr(mode).to_bytes();
    if mode_has(mode_bytes, b'r') {
        let hfp = super::hfile::hopen(path, mode);
        if hfp.is_null() {
            return ptr::null_mut();
        }
        let fp = bgzf_read_init_hfile(hfp);
        if fp.is_null() {
            super::hfile::hclose_abruptly(hfp);
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else if mode_has(mode_bytes, b'w') || mode_has(mode_bytes, b'a') {
        let hfp = super::hfile::hopen(path, mode);
        if hfp.is_null() {
            return ptr::null_mut();
        }
        let fp = bgzf_write_init(ptr_to_fd(hfp), mode_bytes);
        if fp.is_null() {
            super::hfile::hclose(hfp);
            return ptr::null_mut();
        }
        (*fp).fp = hfp;
        set_flag(fp, 31, mode_has(mode_bytes, b'g'));
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else {
        *c_compat::__errno_location() = c_compat::EINVAL;
        ptr::null_mut()
    }
}

pub unsafe fn bgzf_dopen(fd: c_int, mode: *const c_char) -> *mut BGZF {
    if mode.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return ptr::null_mut();
    }
    let mode_bytes = CStr::from_ptr(mode).to_bytes();
    if mode_has(mode_bytes, b'r') {
        let hfp = super::hfile::hdopen(fd, mode);
        if hfp.is_null() {
            return ptr::null_mut();
        }
        let fp = bgzf_read_init_hfile(hfp);
        if fp.is_null() {
            super::hfile::hclose_abruptly(hfp);
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else if mode_has(mode_bytes, b'w') || mode_has(mode_bytes, b'a') {
        let hfp = super::hfile::hdopen(fd, mode);
        if hfp.is_null() {
            return ptr::null_mut();
        }
        let fp = bgzf_write_init(ptr_to_fd(hfp), mode_bytes);
        if fp.is_null() {
            super::hfile::hclose(hfp);
            return ptr::null_mut();
        }
        (*fp).fp = hfp;
        set_flag(fp, 31, mode_has(mode_bytes, b'g'));
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else {
        *c_compat::__errno_location() = c_compat::EINVAL;
        ptr::null_mut()
    }
}

pub unsafe fn bgzf_hopen(hfp: *mut super::hts::hFILE, mode: *const c_char) -> *mut BGZF {
    if hfp.is_null() || mode.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return ptr::null_mut();
    }
    let mode_bytes = CStr::from_ptr(mode).to_bytes();
    if mode_has(mode_bytes, b'r') {
        let fp = bgzf_read_init_hfile(hfp);
        if fp.is_null() {
            return ptr::null_mut();
        }
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else if mode_has(mode_bytes, b'w') || mode_has(mode_bytes, b'a') {
        let fp = bgzf_write_init(ptr_to_fd(hfp), mode_bytes);
        if fp.is_null() {
            return ptr::null_mut();
        }
        (*fp).fp = hfp;
        set_flag(fp, 31, mode_has(mode_bytes, b'g'));
        set_flag(fp, 19, ed_is_big() != 0);
        fp
    } else {
        *c_compat::__errno_location() = c_compat::EINVAL;
        ptr::null_mut()
    }
}

pub unsafe fn bgzf_close(fp: *mut BGZF) -> c_int {
    if fp.is_null() {
        return -1;
    }
    if flag(fp, 17) && flag(fp, 30) && flag(fp, 31) {
        if bgzf_gzip_flush(fp, Z_FINISH) != 0 || bgzf_hflush_ptr((*fp).fp) != 0 {
            add_errcode(fp, BGZF_ERR_IO);
            let _ = bgzf_hclose_ptr((*fp).fp);
            bgzf_free_without_hclose(fp);
            return -1;
        }
    } else if flag(fp, 17) && flag(fp, 30) {
        if !(*fp).mt.is_null() {
            if bgzf_c_2071_bgzf_close_mt(fp) != 0 {
                let _ = bgzf_hclose_ptr((*fp).fp);
                bgzf_free_without_hclose(fp);
                return -1;
            }
        } else if bgzf_flush(fp) != 0 {
            let _ = bgzf_hclose_ptr((*fp).fp);
            bgzf_free_without_hclose(fp);
            return -1;
        } else {
            set_compress_level(fp, Z_DEFAULT_COMPRESSION);
            let block_length = deflate_block(fp, 0);
            if block_length < 0
                || bgzf_hwrite_ptr((*fp).fp, (*fp).compressed_block, block_length as usize)
                    != block_length as isize
                || bgzf_hflush_ptr((*fp).fp) != 0
            {
                add_errcode(fp, BGZF_ERR_IO);
                let _ = bgzf_hclose_ptr((*fp).fp);
                bgzf_free_without_hclose(fp);
                return -1;
            }
        }
    }
    let had_error = errcode(fp) != 0;
    let ret = bgzf_hclose_ptr((*fp).fp);
    bgzf_index_destroy(fp);
    bgzf_free_cache(fp);
    if !(*fp).gz_stream.is_null() {
        let stream = (*fp).gz_stream.cast::<GzipStream>();
        if flag(fp, 17) && flag(fp, 31) {
            gzip_deflate_stream_end(stream);
        } else {
            gzip_stream_end(stream);
        }
        drop(Box::from_raw(stream));
        (*fp).gz_stream = ptr::null_mut();
    }
    c_compat::free((*fp).uncompressed_block);
    c_compat::free(fp.cast());
    if ret != 0 || had_error {
        -1
    } else {
        0
    }
}

pub unsafe fn bgzf_read(fp: *mut BGZF, data: *mut c_void, length: usize) -> isize {
    if length == 0 {
        return 0;
    }
    if fp.is_null() || flag(fp, 17) {
        return -1;
    }
    let mut bytes_read = 0usize;
    let mut output = data.cast::<u8>();
    while bytes_read < length {
        let mut available = (*fp).block_length - (*fp).block_offset;
        if available <= 0 {
            let read_block_ret = if !(*fp).mt.is_null() && flag(fp, 30) && !flag(fp, 31) {
                bgzf_c_1485_bgzf_mt_read_block(fp)
            } else {
                bgzf_read_block(fp)
            };
            if read_block_ret != 0 {
                add_errcode(fp, BGZF_ERR_ZLIB);
                return -1;
            }
            available = (*fp).block_length - (*fp).block_offset;
            if available == 0 {
                if (*fp).block_length == 0 {
                    break;
                }
                (*fp).block_address = bgzf_block_end_address(fp);
                (*fp).block_offset = 0;
                (*fp).block_length = 0;
                continue;
            }
            if available < 0 {
                add_errcode(fp, BGZF_ERR_MISUSE);
                return -1;
            }
        }
        let copy_length = (length - bytes_read).min(available as usize);
        ptr::copy_nonoverlapping(
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            output,
            copy_length,
        );
        (*fp).block_offset += copy_length as c_int;
        output = output.add(copy_length);
        bytes_read += copy_length;
        if (*fp).block_offset == (*fp).block_length {
            (*fp).block_address = bgzf_block_end_address(fp);
            (*fp).block_offset = 0;
            (*fp).block_length = 0;
        }
    }
    (*fp).uncompressed_address += bytes_read as i64;
    bytes_read as isize
}

pub unsafe fn bgzf_read_small(fp: *mut BGZF, data: *mut c_void, length: usize) -> isize {
    if (length as isize) < ((*fp).block_length - (*fp).block_offset) as isize {
        ptr::copy_nonoverlapping(
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            data.cast::<u8>(),
            length,
        );
        (*fp).block_offset += length as c_int;
        (*fp).uncompressed_address += length as i64;
        length as isize
    } else {
        bgzf_read(fp, data, length)
    }
}

pub unsafe fn bgzf_read_block_data(fp: *mut BGZF, data: *mut *const c_void) -> isize {
    if data.is_null() {
        return -1;
    }
    *data = ptr::null();
    if fp.is_null() || flag(fp, 17) {
        return -1;
    }
    if (*fp).block_offset >= (*fp).block_length {
        if bgzf_read_block(fp) != 0 {
            add_errcode(fp, BGZF_ERR_ZLIB);
            return -1;
        }
        if (*fp).block_length == 0 {
            return 0;
        }
    }
    let available = (*fp).block_length - (*fp).block_offset;
    if available < 0 {
        add_errcode(fp, BGZF_ERR_MISUSE);
        return -1;
    }
    *data = (*fp)
        .uncompressed_block
        .cast::<u8>()
        .add((*fp).block_offset as usize)
        .cast();
    (*fp).block_offset += available;
    (*fp).uncompressed_address += available as i64;
    if (*fp).block_offset == (*fp).block_length {
        (*fp).block_address = bgzf_block_end_address(fp);
        (*fp).block_offset = 0;
        (*fp).block_length = 0;
    }
    available as isize
}

pub unsafe fn bgzf_raw_read(fp: *mut BGZF, data: *mut c_void, length: usize) -> isize {
    let mut copied = 0usize;
    if (*fp).block_clength < 0 && length > 0 {
        let preloaded = (-(*fp).block_clength) as usize;
        let copy_len = preloaded.min(length);
        let src = if flag(fp, 30) {
            (*fp).compressed_block
        } else {
            (*fp).uncompressed_block
        }
        .cast::<u8>();
        ptr::copy_nonoverlapping(src, data.cast::<u8>(), copy_len);
        if copy_len < preloaded {
            ptr::copy(src.add(copy_len), src, preloaded - copy_len);
            (*fp).block_clength = -((preloaded - copy_len) as c_int);
            return copy_len as isize;
        }
        (*fp).block_clength = 0;
        copied = copy_len;
    }

    let ret = bgzf_hread_ptr(
        (*fp).fp,
        data.cast::<u8>().add(copied).cast(),
        length - copied,
    );
    if ret < 0 {
        add_errcode(fp, BGZF_ERR_IO);
    }
    if ret < 0 && copied == 0 {
        ret
    } else {
        copied as isize + ret.max(0)
    }
}

pub unsafe fn bgzf_write(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if fp.is_null() || !flag(fp, 17) {
        return -1;
    }
    if length == 0 {
        return 0;
    }
    if !flag(fp, 30) {
        let push = length + (*fp).block_offset as usize;
        (*fp).block_offset = (push % BGZF_MAX_BLOCK_SIZE) as c_int;
        (*fp).block_address += (push - (*fp).block_offset as usize) as i64;
        let written = bgzf_hwrite_ptr((*fp).fp, data, length);
        if written < 0 {
            add_errcode(fp, BGZF_ERR_IO);
        }
        return written;
    }
    let mut written = 0usize;
    while written < length {
        let available = BGZF_BLOCK_SIZE - (*fp).block_offset as usize;
        let copy_length = (length - written).min(available);
        ptr::copy_nonoverlapping(
            data.cast::<u8>().add(written),
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            copy_length,
        );
        (*fp).block_offset += copy_length as c_int;
        written += copy_length;
        if (*fp).block_offset as usize == BGZF_BLOCK_SIZE && bgzf_flush(fp) != 0 {
            return -1;
        }
    }
    written as isize
}

pub unsafe fn bgzf_raw_write(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    let ret = bgzf_hwrite_ptr((*fp).fp, data, length);
    if ret < 0 {
        add_errcode(fp, BGZF_ERR_IO);
    }
    ret
}

pub unsafe fn bgzf_flush_try(fp: *mut BGZF, size: isize) -> c_int {
    if (*fp).block_offset as isize + size > BGZF_BLOCK_SIZE as isize {
        return bgzf_flush(fp);
    }
    0
}

pub unsafe fn bgzf_c_1942_lazy_flush(fp: *mut BGZF) -> c_int {
    bgzf_flush(fp)
}

// original: lazy_flush (htslib/bgzf.c:1927)
pub unsafe fn lazy_flush(fp: *mut BGZF) -> c_int {
    bgzf_c_1942_lazy_flush(fp)
}

pub unsafe fn bgzf_block_write(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if fp.is_null() || !flag(fp, 17) {
        return -1;
    }
    if length == 0 {
        return 0;
    }
    if !flag(fp, 30) {
        let push = length + (*fp).block_offset as usize;
        (*fp).block_offset = (push % BGZF_MAX_BLOCK_SIZE) as c_int;
        (*fp).block_address += (push - (*fp).block_offset as usize) as i64;
        let written = bgzf_hwrite_ptr((*fp).fp, data, length);
        if written < 0 {
            add_errcode(fp, BGZF_ERR_IO);
        }
        return written;
    }

    let input = data.cast::<u8>();
    let mut remaining = length;
    let mut consumed = 0usize;
    while remaining > 0 {
        let idx = (*fp).idx.cast::<bgzidx_t>();
        let current_block = (*idx).moffs - (*idx).noffs;
        let ublock_size = if current_block + 1 < (*idx).moffs {
            let cur = (*idx).offs.add(current_block as usize);
            let next = (*idx).offs.add(current_block as usize + 1);
            (*next).uaddr - (*cur).uaddr
        } else {
            BGZF_MAX_BLOCK_SIZE as u64
        };
        let mut copy_length = (ublock_size as c_int - (*fp).block_offset) as usize;
        if copy_length > remaining {
            copy_length = remaining;
        }
        ptr::copy_nonoverlapping(
            input.add(consumed),
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            copy_length,
        );
        (*fp).block_offset += copy_length as c_int;
        consumed += copy_length;
        remaining -= copy_length;
        if (*fp).block_offset as u64 == ublock_size {
            if bgzf_flush(fp) != 0 {
                return -1;
            }
            if (*idx).noffs > 0 {
                (*idx).noffs -= 1;
            }
        }
    }
    consumed as isize
}

pub unsafe fn bgzf_write_direct_block(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if fp.is_null() || !flag(fp, 17) || length > BGZF_BLOCK_SIZE || (*fp).block_offset != 0 {
        return -1;
    }
    if length == 0 {
        return 0;
    }
    if !flag(fp, 30) {
        return bgzf_write(fp, data, length);
    }
    if (*fp).idx_build_otf != 0 {
        bgzf_index_add_block(fp);
        (*(*fp).idx.cast::<bgzidx_t>()).ublock_addr += length as u64;
    }

    let mut comp_size = BGZF_MAX_BLOCK_SIZE;
    if bgzf_compress(
        (*fp).compressed_block,
        &mut comp_size,
        data,
        length,
        compress_level(fp),
    ) != 0
    {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -1;
    }
    let written = bgzf_hwrite_ptr((*fp).fp, (*fp).compressed_block, comp_size);
    if written != comp_size as isize {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    (*fp).block_address += comp_size as i64;
    length as isize
}

pub unsafe fn bgzf_set_cache_size(fp: *mut BGZF, cache_size: c_int) {
    if !fp.is_null() && !(*fp).mt.is_null() {
        return;
    }
    if !fp.is_null() && !(*fp).cache.is_null() {
        (*fp).cache_size = cache_size;
    }
}

pub unsafe fn bgzf_write_small(fp: *mut BGZF, data: *const c_void, length: usize) -> isize {
    if flag(fp, 30) && BGZF_BLOCK_SIZE - (*fp).block_offset as usize > length {
        ptr::copy_nonoverlapping(
            data.cast::<u8>(),
            (*fp)
                .uncompressed_block
                .cast::<u8>()
                .add((*fp).block_offset as usize),
            length,
        );
        (*fp).block_offset += length as c_int;
        length as isize
    } else {
        bgzf_write(fp, data, length)
    }
}

pub unsafe fn bgzf_seek(fp: *mut BGZF, pos: i64, whence: c_int) -> i64 {
    if fp.is_null() || flag(fp, 17) || whence != SEEK_SET || flag(fp, 31) {
        if !fp.is_null() {
            add_errcode(fp, BGZF_ERR_MISUSE);
        }
        return -1;
    }
    (*fp).seeked = pos;
    bgzf_c_2175_bgzf_seek_common(fp, pos >> 16, (pos & 0xffff) as c_int)
}

pub unsafe fn bgzf_c_2175_bgzf_seek_common(
    fp: *mut BGZF,
    block_address: i64,
    block_offset: c_int,
) -> i64 {
    if bgzf_hseek_ptr((*fp).fp, block_address, SEEK_SET) < 0 {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    (*fp).block_length = 0;
    (*fp).block_address = block_address;
    (*fp).block_offset = block_offset;
    (*fp).block_clength = 0;
    0
}

pub unsafe fn bgzf_useek(fp: *mut BGZF, uoffset: i64, where_: c_int) -> c_int {
    if fp.is_null() || flag(fp, 17) || where_ != SEEK_SET || flag(fp, 31) {
        if !fp.is_null() {
            add_errcode(fp, BGZF_ERR_MISUSE);
        }
        return -1;
    }
    if uoffset >= (*fp).uncompressed_address - (*fp).block_offset as i64
        && uoffset
            < (*fp).uncompressed_address + (*fp).block_length as i64 - (*fp).block_offset as i64
    {
        (*fp).block_offset += (uoffset - (*fp).uncompressed_address) as c_int;
        (*fp).uncompressed_address = uoffset;
        return 0;
    }
    if !flag(fp, 30) {
        if bgzf_hseek_ptr((*fp).fp, uoffset, SEEK_SET) < 0 {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        (*fp).block_length = 0;
        (*fp).block_address = uoffset;
        (*fp).block_offset = 0;
        (*fp).block_clength = 0;
        if bgzf_read_block(fp) < 0 {
            add_errcode(fp, BGZF_ERR_IO);
            return -1;
        }
        (*fp).uncompressed_address = uoffset;
        return 0;
    }
    if (*fp).idx.is_null() {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }

    let idx = (*fp).idx.cast::<bgzidx_t>();
    if (*idx).noffs <= 0 || (*idx).offs.is_null() {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }

    let target = uoffset as u64;
    let mut lo = 0usize;
    let mut hi = (*idx).noffs as usize;
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        if (*(*idx).offs.add(mid)).uaddr <= target {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let entry = (*idx).offs.add(lo);
    if target < (*entry).uaddr {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    if bgzf_c_2175_bgzf_seek_common(fp, (*entry).caddr as i64, 0) < 0 {
        return -1;
    }
    if bgzf_read_block(fp) < 0 {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    let block_offset = target - (*entry).uaddr;
    if block_offset > (*fp).block_length as u64 {
        add_errcode(fp, BGZF_ERR_IO);
        return -1;
    }
    if block_offset > 0 {
        (*fp).block_offset = block_offset as c_int;
    }
    (*fp).uncompressed_address = uoffset;
    0
}

pub unsafe fn bgzf_check_EOF(fp: *mut BGZF) -> c_int {
    if fp.is_null() || !flag(fp, 30) || flag(fp, 31) {
        return 0;
    }
    let has_eof = bgzf_c_1542_bgzf_check_EOF_common(fp);
    set_flag(fp, 18, has_eof == 0);
    has_eof
}

pub unsafe fn bgzf_c_1542_bgzf_check_EOF_common(fp: *mut BGZF) -> c_int {
    let offset = bgzf_htell_ptr((*fp).fp);
    let mut buf = [0u8; 28];
    if bgzf_hseek_ptr((*fp).fp, -(EOF_BLOCK.len() as i64), libc::SEEK_END) < 0 {
        let errno = *c_compat::__errno_location();
        if errno == libc::ESPIPE {
            bgzf_hclearerr_ptr((*fp).fp);
            return 2;
        }
        if errno == libc::EINVAL {
            bgzf_hclearerr_ptr((*fp).fp);
            return 0;
        }
        return -1;
    }
    let n = bgzf_hread_ptr((*fp).fp, buf.as_mut_ptr().cast(), buf.len());
    if bgzf_hseek_ptr((*fp).fp, offset, SEEK_SET) < 0 {
        return -1;
    }
    if n < 0 {
        return -1;
    }
    if n != buf.len() as isize {
        return 0;
    }
    if buf == EOF_BLOCK {
        1
    } else {
        0
    }
}

// original: bgzf_mt_eof (htslib/bgzf.c:1566)
pub unsafe fn bgzf_c_1566_bgzf_mt_eof(fp: *mut BGZF) {
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();

    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).job_pool_m));
    (*mt).eof = bgzf_c_1542_bgzf_check_EOF_common(fp);
    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).job_pool_m));
    (*mt).command = mtaux_cmd::HAS_EOF_DONE;
    libc::pthread_cond_signal(ptr::addr_of_mut!((*mt).command_c));
}

// original: bgzf_mt_seek (htslib/bgzf.c:1583)
pub unsafe fn bgzf_c_1583_bgzf_mt_seek(fp: *mut BGZF) {
    let mt = (*fp).mt.cast::<bgzf_mtaux_t>();

    super::thread_pool::hts_tpool_process_reset((*mt).out_queue.cast(), 0);
    libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).job_pool_m));
    (*mt).errcode = 0;

    if bgzf_hseek_ptr((*fp).fp, (*mt).block_address as i64, SEEK_SET) < 0 {
        (*mt).errcode = *c_compat::__errno_location();
    }

    libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).job_pool_m));
    (*mt).command = mtaux_cmd::SEEK_DONE;
    libc::pthread_cond_signal(ptr::addr_of_mut!((*mt).command_c));
}

pub unsafe fn bgzf_is_bgzf(fn_: *const c_char) -> c_int {
    let fd = libc::open(fn_, libc::O_RDONLY);
    if fd < 0 {
        return 0;
    }
    let mut buf = [0u8; 16];
    let n = fd_read(fd, buf.as_mut_ptr().cast(), buf.len());
    if libc::close(fd) < 0 {
        return 0;
    }
    if n != 16 {
        return 0;
    }
    if check_header(buf.as_ptr()) == 0 {
        1
    } else {
        0
    }
}

pub unsafe fn bgzf_compression(fp: *mut BGZF) -> c_int {
    if !flag(fp, 30) {
        0
    } else if flag(fp, 31) {
        1
    } else {
        2
    }
}

pub unsafe fn bgzf_hfile(fp: *mut BGZF) -> *mut super::hts::hFILE {
    (*fp).fp
}

pub unsafe fn bgzf_utell(fp: *mut BGZF) -> i64 {
    (*fp).uncompressed_address
}

pub unsafe fn bgzf_getc(fp: *mut BGZF) -> c_int {
    if (*fp).block_offset + 1 < (*fp).block_length {
        (*fp).uncompressed_address += 1;
        let c = *(*fp)
            .uncompressed_block
            .cast::<u8>()
            .add((*fp).block_offset as usize);
        (*fp).block_offset += 1;
        return c as c_int;
    }

    if (*fp).block_offset >= (*fp).block_length {
        if bgzf_read_block(fp) != 0 {
            return -2;
        }
        if (*fp).block_length == 0 {
            return -1;
        }
    }
    let c = *(*fp)
        .uncompressed_block
        .cast::<u8>()
        .add((*fp).block_offset as usize);
    (*fp).block_offset += 1;
    if (*fp).block_offset == (*fp).block_length {
        (*fp).block_address = bgzf_block_end_address(fp);
        (*fp).block_offset = 0;
        (*fp).block_length = 0;
    }
    (*fp).uncompressed_address += 1;
    c as c_int
}

pub unsafe fn bgzf_getline(fp: *mut BGZF, delim: c_int, str_: *mut kstring_t) -> c_int {
    let mut state = 0;
    (*str_).l = 0;

    loop {
        if (*fp).block_offset >= (*fp).block_length {
            if bgzf_read_block(fp) != 0 {
                state = -2;
                break;
            }
            if (*fp).block_length == 0 {
                state = -1;
                break;
            }
        }

        let buf = (*fp).uncompressed_block.cast::<u8>();
        let found = libc::memchr(
            buf.add((*fp).block_offset as usize).cast(),
            delim,
            ((*fp).block_length - (*fp).block_offset) as usize,
        );
        let mut l = if found.is_null() {
            (*fp).block_length
        } else {
            found.cast::<u8>().offset_from(buf) as c_int
        };

        if l < (*fp).block_length {
            state = 1;
        }
        l -= (*fp).block_offset;
        if ks_expand(str_, l as usize + 2) < 0 {
            state = -3;
            break;
        }
        ptr::copy_nonoverlapping(
            buf.add((*fp).block_offset as usize),
            (*str_).s.add((*str_).l).cast::<u8>(),
            l as usize,
        );
        (*str_).l += l as usize;
        (*fp).block_offset += l + 1;
        if (*fp).block_offset >= (*fp).block_length {
            (*fp).block_address = bgzf_block_end_address(fp);
            (*fp).block_offset = 0;
            (*fp).block_length = 0;
        }
        if state != 0 {
            break;
        }
    }

    if state < -1 {
        return state;
    }
    if (*str_).l == 0 && state < 0 {
        return state;
    }
    (*fp).uncompressed_address += (*str_).l as i64 + 1;
    if delim == b'\n' as c_int && (*str_).l > 0 && *(*str_).s.add((*str_).l - 1) == b'\r' as c_char
    {
        (*str_).l -= 1;
    }
    *(*str_).s.add((*str_).l) = 0;
    if (*str_).l <= c_int::MAX as usize {
        (*str_).l as c_int
    } else {
        c_int::MAX
    }
}

unsafe fn bgzf_mt_init(
    fp: *mut BGZF,
    pool: *mut super::thread_pool::hts_tpool,
    qsize: c_int,
    own_pool: c_int,
) -> c_int {
    if fp.is_null() || pool.is_null() {
        *c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    if !flag(fp, 30) || flag(fp, 31) {
        return 0;
    }
    if !(*fp).mt.is_null() {
        return -2;
    }

    let mt = c_compat::calloc(1, std::mem::size_of::<bgzf_mtaux_t>() as u64).cast::<bgzf_mtaux_t>();
    if mt.is_null() {
        return -1;
    }
    (*mt).pool = pool;
    (*mt).own_pool = own_pool;
    (*mt).n_threads = super::thread_pool::hts_tpool_size(pool.cast());
    (*mt).job_pool =
        super::cram::cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<bgzf_job>()).cast();
    if (*mt).job_pool.is_null() {
        c_compat::free(mt.cast());
        return -1;
    }
    let queue_size = if qsize > 0 {
        qsize
    } else {
        ((*mt).n_threads * 2).max(2)
    };
    (*mt).out_queue = super::thread_pool::hts_tpool_process_init(pool.cast(), queue_size, 0).cast();
    if (*mt).out_queue.is_null() {
        super::cram::cram_pooled_alloc_c_84_pool_destroy((*mt).job_pool.cast());
        c_compat::free(mt.cast());
        return -1;
    }

    if libc::pthread_mutex_init(ptr::addr_of_mut!((*mt).job_pool_m), ptr::null()) != 0
        || libc::pthread_mutex_init(ptr::addr_of_mut!((*mt).command_m), ptr::null()) != 0
        || libc::pthread_mutex_init(ptr::addr_of_mut!((*mt).idx_m), ptr::null()) != 0
        || libc::pthread_cond_init(ptr::addr_of_mut!((*mt).command_c), ptr::null()) != 0
    {
        super::thread_pool::hts_tpool_process_destroy((*mt).out_queue.cast());
        super::cram::cram_pooled_alloc_c_84_pool_destroy((*mt).job_pool.cast());
        if own_pool != 0 {
            super::thread_pool::hts_tpool_destroy(pool.cast());
        }
        c_compat::free(mt.cast());
        return -1;
    }

    (*mt).block_address = (*fp).block_address as u64;
    (*mt).command = mtaux_cmd::NONE;
    (*fp).mt = mt.cast();
    let start = if flag(fp, 17) {
        bgzf_c_1398_bgzf_mt_writer
    } else {
        bgzf_c_1598_bgzf_mt_reader
    };
    if libc::pthread_create(
        ptr::addr_of_mut!((*mt).io_task),
        ptr::null(),
        start,
        fp.cast(),
    ) != 0
    {
        (*fp).mt = ptr::null_mut();
        libc::pthread_cond_destroy(ptr::addr_of_mut!((*mt).command_c));
        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*mt).idx_m));
        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*mt).command_m));
        libc::pthread_mutex_destroy(ptr::addr_of_mut!((*mt).job_pool_m));
        super::thread_pool::hts_tpool_process_destroy((*mt).out_queue.cast());
        super::cram::cram_pooled_alloc_c_84_pool_destroy((*mt).job_pool.cast());
        if own_pool != 0 {
            super::thread_pool::hts_tpool_destroy(pool.cast());
        }
        c_compat::free(mt.cast());
        return -1;
    }

    0
}

pub unsafe fn bgzf_thread_pool(
    fp: *mut BGZF,
    pool: *mut super::thread_pool::hts_tpool,
    qsize: c_int,
) -> c_int {
    if fp.is_null() {
        *c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    if !flag(fp, 30) {
        return 0;
    }
    if !(*fp).mt.is_null() {
        return -2;
    }
    if pool.is_null() {
        *c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    bgzf_mt_init(fp, pool, qsize, 0)
}

pub unsafe fn bgzf_mt(fp: *mut BGZF, n_threads: c_int, n_sub_blks: c_int) -> c_int {
    if n_threads <= 1 {
        return 0;
    }
    if fp.is_null() {
        *c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    if !flag(fp, 30) || flag(fp, 31) {
        return 0;
    }
    if !(*fp).mt.is_null() {
        return -2;
    }
    let pool = super::thread_pool::hts_tpool_init(n_threads);
    if pool.is_null() {
        return -1;
    }
    bgzf_mt_init(fp, pool.cast(), n_threads * n_sub_blks.max(1), 1)
}

pub unsafe fn bgzf_index_destroy(fp: *mut BGZF) {
    if (*fp).idx.is_null() {
        return;
    }
    let idx = (*fp).idx.cast::<bgzidx_t>();
    c_compat::free((*idx).offs.cast());
    c_compat::free(idx.cast());
    (*fp).idx = ptr::null_mut();
    (*fp).idx_build_otf = 0;
}

pub unsafe fn bgzf_index_build_init(fp: *mut BGZF) -> c_int {
    bgzf_index_destroy(fp);
    (*fp).idx = c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64);
    if (*fp).idx.is_null() {
        return -1;
    }
    (*fp).idx_build_otf = 1;
    0
}

pub unsafe fn bgzf_index_add_block(fp: *mut BGZF) -> c_int {
    let idx = (*fp).idx.cast::<bgzidx_t>();
    (*idx).noffs += 1;
    if (*idx).noffs > (*idx).moffs {
        (*idx).moffs = (*idx).noffs;
        let mut rounded = (*idx).moffs as u32;
        rounded = rounded.wrapping_sub(1);
        rounded |= rounded >> 1;
        rounded |= rounded >> 2;
        rounded |= rounded >> 4;
        rounded |= rounded >> 8;
        rounded |= rounded >> 16;
        rounded = rounded.wrapping_add(1);
        (*idx).moffs = rounded as c_int;
        let tmp = c_compat::realloc(
            (*idx).offs.cast(),
            ((*idx).moffs as usize * std::mem::size_of::<bgzidx1_t>()) as u64,
        )
        .cast::<bgzidx1_t>();
        if tmp.is_null() {
            return -1;
        }
        (*idx).offs = tmp;
    }
    let slot = (*idx).offs.add((*idx).noffs as usize - 1);
    (*slot).uaddr = (*idx).ublock_addr;
    (*slot).caddr = (*fp).block_address as u64;
    0
}

pub unsafe fn bgzf_index_dump_hfile(
    fp: *mut BGZF,
    idx_file: *mut super::hts::hFILE,
    _name: *const c_char,
) -> c_int {
    if (*fp).idx.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return -1;
    }
    if bgzf_flush(fp) != 0 {
        return -1;
    }

    let idx = (*fp).idx.cast::<bgzidx_t>();
    if !(*fp).mt.is_null() && (*idx).noffs > 0 {
        (*idx).noffs -= 1;
    }
    if hwrite_uint64(
        if (*idx).noffs > 0 {
            ((*idx).noffs - 1) as u64
        } else {
            0
        },
        idx_file,
    ) < 0
    {
        return -1;
    }
    let mut i = 1;
    while i < (*idx).noffs {
        let off = (*idx).offs.add(i as usize);
        if hwrite_uint64((*off).caddr, idx_file) < 0 {
            return -1;
        }
        if hwrite_uint64((*off).uaddr, idx_file) < 0 {
            return -1;
        }
        i += 1;
    }
    0
}

pub unsafe fn bgzf_index_dump(fp: *mut BGZF, bname: *const c_char, suffix: *const c_char) -> c_int {
    if (*fp).idx.is_null() {
        *c_compat::__errno_location() = c_compat::EINVAL;
        return -1;
    }

    let mut tmp = ptr::null_mut();
    let name = if !suffix.is_null() {
        tmp = get_name_suffix(bname, suffix);
        if tmp.is_null() {
            return -1;
        }
        tmp
    } else {
        bname.cast_mut()
    };

    let fd = libc::open(name, libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC, 0o666);
    if fd < 0 {
        c_compat::free(tmp.cast());
        return -1;
    }

    let ret = bgzf_index_dump_hfile(fp, fd_to_ptr(fd), name);
    if libc::close(fd) < 0 || ret != 0 {
        c_compat::free(tmp.cast());
        return -1;
    }
    c_compat::free(tmp.cast());
    0
}

pub unsafe fn bgzf_index_load_hfile(
    fp: *mut BGZF,
    idx_file: *mut super::hts::hFILE,
    _name: *const c_char,
) -> c_int {
    (*fp).idx = c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64);
    if (*fp).idx.is_null() {
        return -1;
    }

    let idx = (*fp).idx.cast::<bgzidx_t>();
    let mut x = 0u64;
    if hread_uint64(&mut x, idx_file) < 0 {
        bgzf_index_destroy(fp);
        return -1;
    }
    if x >= (usize::MAX as u64 / std::mem::size_of::<bgzidx1_t>() as u64 / 2) {
        bgzf_index_destroy(fp);
        return -1;
    }

    (*idx).noffs = x.wrapping_add(1) as c_int;
    (*idx).moffs = (*idx).noffs;
    (*idx).offs =
        c_compat::malloc(((*idx).moffs as usize * std::mem::size_of::<bgzidx1_t>()) as u64)
            .cast::<bgzidx1_t>();
    if (*idx).offs.is_null() {
        bgzf_index_destroy(fp);
        return -1;
    }
    (*(*idx).offs.add(0)).caddr = 0;
    (*(*idx).offs.add(0)).uaddr = 0;

    let mut i = 1;
    while i < (*idx).noffs {
        let off = (*idx).offs.add(i as usize);
        if hread_uint64(&mut (*off).caddr, idx_file) < 0 {
            bgzf_index_destroy(fp);
            return -1;
        }
        if hread_uint64(&mut (*off).uaddr, idx_file) < 0 {
            bgzf_index_destroy(fp);
            return -1;
        }
        i += 1;
    }

    0
}

pub unsafe fn bgzf_index_load(fp: *mut BGZF, bname: *const c_char, suffix: *const c_char) -> c_int {
    let mut tmp = ptr::null_mut();
    let name = if !suffix.is_null() {
        tmp = get_name_suffix(bname, suffix);
        if tmp.is_null() {
            return -1;
        }
        tmp
    } else {
        bname.cast_mut()
    };

    let fd = libc::open(name, libc::O_RDONLY);
    if fd < 0 {
        c_compat::free(tmp.cast());
        return -1;
    }

    let ret = bgzf_index_load_hfile(fp, fd_to_ptr(fd), name);
    if libc::close(fd) != 0 || ret != 0 {
        c_compat::free(tmp.cast());
        return -1;
    }
    c_compat::free(tmp.cast());
    0
}

pub unsafe fn bgzf_peek(fp: *mut BGZF) -> c_int {
    if fp.is_null() {
        return -2;
    }
    let mut available = (*fp).block_length - (*fp).block_offset;
    if available <= 0 && bgzf_read_block(fp) < 0 {
        add_errcode(fp, BGZF_ERR_ZLIB);
        return -2;
    }
    available = (*fp).block_length - (*fp).block_offset;
    if available != 0 {
        *(*fp)
            .uncompressed_block
            .cast::<u8>()
            .add((*fp).block_offset as usize) as c_int
    } else {
        -1
    }
}

pub unsafe fn hwrite_uint64(mut x: u64, f: *mut super::hts::hFILE) -> c_int {
    if ed_is_big() != 0 {
        x = ed_swap_8(x);
    }
    if bgzf_hwrite_ptr(f, (&x as *const u64).cast(), std::mem::size_of::<u64>())
        != std::mem::size_of::<u64>() as isize
    {
        return -1;
    }
    0
}

pub unsafe fn hread_uint64(xptr: *mut u64, f: *mut super::hts::hFILE) -> c_int {
    if bgzf_hread_ptr(f, xptr.cast::<c_void>(), std::mem::size_of::<u64>())
        != std::mem::size_of::<u64>() as isize
    {
        return -1;
    }
    if ed_is_big() != 0 {
        *xptr = ed_swap_8(*xptr);
    }
    0
}

pub unsafe fn get_name_suffix(bname: *const c_char, suffix: *const c_char) -> *mut c_char {
    let len = libc::strlen(bname) + libc::strlen(suffix) + 1;
    let buff = c_compat::malloc(len as u64).cast::<c_char>();
    if buff.is_null() {
        return ptr::null_mut();
    }
    libc::snprintf(buff, len, c"%s%s".as_ptr(), bname, suffix);
    buff
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::CString,
        fs::File,
        io::{Read, Write},
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn translated_bgzf_reads_and_seeks_noodles_bgzf_file() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-{}-{}.gz",
            std::process::id(),
            "read_seek"
        ));
        let payload = b"abcdef0123456789".repeat(8192);
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(&payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());

            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                buf.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_check_EOF(fp), 1);

            assert_eq!(bgzf_seek(fp, 0, 0), 0);
            assert_eq!(bgzf_peek(fp), b'a' as c_int);

            let mut small = [0u8; 6];
            assert_eq!(bgzf_read(fp, small.as_mut_ptr().cast(), small.len()), 6);
            assert_eq!(&small, b"abcdef");
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_open_allocates_cache_and_close_runs_private_cleanup() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-cache-{}-{}.gz",
            std::process::id(),
            "cleanup"
        ));
        let payload = b"cache-backed bgzf state";
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            PRIVATE_DATA_CLEANUPS.store(0, Ordering::SeqCst);
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert!(!(*fp).cache.is_null());

            bgzf_set_cache_size(fp, 8192);
            assert_eq!((*fp).cache_size, 8192);

            let data = Box::into_raw(Box::new(23usize)).cast::<c_void>();
            bgzf_internal_h_51_bgzf_set_private_data(fp, data, Some(count_private_data_cleanup));
            assert_eq!(bgzf_internal_h_67_bgzf_get_private_data(fp), data);
            assert_eq!(bgzf_close(fp), 0);
            assert_eq!(PRIVATE_DATA_CLEANUPS.load(Ordering::SeqCst), 1);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_cache_reuses_decoded_block_after_virtual_seek() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-cache-{}-{}.gz",
            std::process::id(),
            "reuse"
        ));
        let payload: Vec<u8> = (0..BGZF_BLOCK_SIZE + 1000).map(|i| i as u8).collect();
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(&payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            bgzf_set_cache_size(fp, (BGZF_MAX_BLOCK_SIZE * 2) as c_int);

            let mut byte = [0u8; 1];
            assert_eq!(bgzf_read(fp, byte.as_mut_ptr().cast(), 1), 1);
            assert_eq!(byte[0], payload[0]);

            let cache = (*fp).cache.cast::<bgzf_cache_t>();
            assert!(!(*cache).h.is_null());
            let h = &*(*cache).h.cast::<BgzfBlockCache>();
            assert_eq!(h.blocks.len(), 1);
            let end_offset = h.blocks.get(&0).unwrap().end_offset;

            assert_eq!(bgzf_seek(fp, 0, SEEK_SET), 0);
            assert_eq!(bgzf_htell_ptr((*fp).fp), 0);
            assert_eq!(bgzf_peek(fp), payload[0] as c_int);
            assert_eq!(bgzf_htell_ptr((*fp).fp), end_offset);

            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_gzip_compress_helper_streams_and_finishes_valid_gzip_member() {
        unsafe {
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            set_flag(&mut fp, 17, true);
            set_flag(&mut fp, 30, true);
            set_flag(&mut fp, 31, true);
            set_compress_level(&mut fp, 6);

            let payload = b"gzip helper payload across partial and finish flushes".repeat(300);
            let mut compressed = Vec::new();
            for chunk in payload.chunks(777) {
                let mut out = vec![0u8; BGZF_MAX_BLOCK_SIZE];
                let mut out_len = out.len();
                assert_eq!(
                    bgzf_c_686_bgzf_gzip_compress(
                        &mut fp,
                        out.as_mut_ptr().cast(),
                        &mut out_len,
                        chunk.as_ptr().cast(),
                        chunk.len(),
                        6,
                    ),
                    0
                );
                compressed.extend_from_slice(&out[..out_len]);
            }
            let mut out = vec![0u8; BGZF_MAX_BLOCK_SIZE];
            let mut out_len = out.len();
            assert_eq!(
                bgzf_c_686_bgzf_gzip_compress(
                    &mut fp,
                    out.as_mut_ptr().cast(),
                    &mut out_len,
                    ptr::null(),
                    0,
                    6,
                ),
                0
            );
            compressed.extend_from_slice(&out[..out_len]);

            let mut decoded = Vec::new();
            GzDecoder::new(compressed.as_slice())
                .read_to_end(&mut decoded)
                .unwrap();
            assert_eq!(decoded, payload);

            let stream = fp.gz_stream.cast::<GzipStream>();
            gzip_deflate_stream_end(stream);
            drop(Box::from_raw(stream));
        }
    }

    #[test]
    fn bgzf_hopen_read_real_hfile_uses_rust_state_and_closes_cleanly() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-hopen-real-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let payload = b"hfile-backed bgzf read".repeat(400);
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(&payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let hfile = super::super::hfile::hopen(path_c.as_ptr(), c"r".as_ptr());
            assert!(!hfile.is_null());
            let fp = bgzf_hopen(hfile, c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hfile(fp), hfile);
            assert!(!(*fp).cache.is_null());

            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                payload.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn translated_bgzf_writes_readable_bgzf_file() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-{}-{}.gz",
            std::process::id(),
            "write"
        ));
        let payload = b"translated-bgzf-write\n".repeat(7000);
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);
        }

        let file = File::open(&path).unwrap();
        let mut reader = noodles::bgzf::io::Reader::new(file);
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, payload);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_seek_to_block_boundary_reads_next_block() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-boundary-{}-{}.gz",
            std::process::id(),
            "seek"
        ));
        let payload: Vec<u8> = (0..BGZF_BLOCK_SIZE + 17)
            .map(|i| b'a' + (i % 23) as u8)
            .collect();
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());

            let mut first_block = vec![0u8; BGZF_BLOCK_SIZE];
            assert_eq!(
                bgzf_read(fp, first_block.as_mut_ptr().cast(), first_block.len()),
                BGZF_BLOCK_SIZE as isize
            );
            assert_eq!(first_block, payload[..BGZF_BLOCK_SIZE]);
            assert_eq!((*fp).block_offset, 0);
            assert_eq!((*fp).block_length, 0);

            let boundary_vo = (*fp).block_address << 16;
            assert!(boundary_vo > 0);
            assert_eq!(bgzf_seek(fp, boundary_vo, libc::SEEK_SET), 0);
            assert_eq!((*fp).seeked, boundary_vo);

            let mut tail = vec![0u8; payload.len() - BGZF_BLOCK_SIZE];
            assert_eq!(
                bgzf_read(fp, tail.as_mut_ptr().cast(), tail.len()),
                tail.len() as isize
            );
            assert_eq!(tail, payload[BGZF_BLOCK_SIZE..]);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_seek_rejects_non_set_whence_without_mutating_position_state() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-seek-misuse-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        std::fs::write(&path, EOF_BLOCK).unwrap();
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hseek_ptr((*fp).fp, 3, libc::SEEK_SET), 3);
            (*fp).seeked = 123;
            (*fp).block_address = 7;
            (*fp).block_offset = 9;

            assert_eq!(bgzf_seek(fp, 0, libc::SEEK_CUR), -1);
            assert_eq!((*fp).seeked, 123);
            assert_eq!((*fp).block_address, 7);
            assert_eq!((*fp).block_offset, 9);
            assert_eq!(bgzf_htell_ptr((*fp).fp), 3);
            assert_ne!(errcode(fp) & BGZF_ERR_MISUSE, 0);
            assert_eq!(bgzf_close(fp), -1);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_dopen_and_hopen_read_fd_backed_bgzf_streams() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-open-wrappers-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let payload = b"fd-backed bgzf payload";
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fd = libc::open(path_c.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let fp = bgzf_dopen(fd, c"r".as_ptr());
            assert!(!fp.is_null());
            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                payload.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_close(fp), 0);

            let fd = libc::open(path_c.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let hfp = fd_to_ptr(fd);
            let fp = bgzf_hopen(hfp, c"r".as_ptr());
            assert!(!fp.is_null());
            let mut buf = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                payload.len() as isize
            );
            assert_eq!(buf, payload);
            assert_eq!(bgzf_hfile(fp), hfp);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_dopen_rejects_razf_during_read_init_like_c() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-razf-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let mut razf = [0u8; BLOCK_HEADER_LENGTH];
        razf[..12].copy_from_slice(&[31, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0]);
        razf[12..16].copy_from_slice(b"RAZF");
        std::fs::write(&path, razf).unwrap();
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            *c_compat::__errno_location() = 0;
            let fd = libc::open(path_c.as_ptr(), libc::O_RDONLY);
            assert!(fd >= 0);
            let fp = bgzf_dopen(fd, c"r".as_ptr());
            assert!(fp.is_null());
            assert_eq!(*c_compat::__errno_location(), c_compat::ENOEXEC);
            *c_compat::__errno_location() = 0;
            assert_eq!(libc::close(fd), -1);
            assert_eq!(*c_compat::__errno_location(), libc::EBADF);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_dopen_reads_bgzf_from_non_seekable_fd_like_hpeek() {
        unsafe {
            let payload = b"bgzf payload through a pipe";
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            let mut stream = Vec::from(&block[..block_len]);
            stream.extend_from_slice(&EOF_BLOCK);

            let mut fds = [0; 2];
            assert_eq!(libc::pipe(fds.as_mut_ptr()), 0);
            assert_eq!(
                libc::write(fds[1], stream.as_ptr().cast(), stream.len()),
                stream.len() as isize
            );
            assert_eq!(libc::close(fds[1]), 0);

            let fp = bgzf_dopen(fds[0], c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, out.as_mut_ptr().cast(), out.len()),
                payload.len() as isize
            );
            assert_eq!(out, payload);
            assert_eq!(bgzf_read(fp, out.as_mut_ptr().cast(), out.len()), 0);
            assert_eq!(bgzf_close(fp), 0);
        }
    }

    #[test]
    fn bgzf_check_eof_on_non_seekable_stream_returns_unknown_like_c() {
        unsafe {
            let payload = b"bgzf eof probe through a pipe";
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            let mut stream = Vec::from(&block[..block_len]);
            stream.extend_from_slice(&EOF_BLOCK);

            let mut fds = [0; 2];
            assert_eq!(libc::pipe(fds.as_mut_ptr()), 0);
            assert_eq!(
                libc::write(fds[1], stream.as_ptr().cast(), stream.len()),
                stream.len() as isize
            );
            assert_eq!(libc::close(fds[1]), 0);

            let fp = bgzf_dopen(fds[0], c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_check_EOF(fp), 2);
            assert!(!flag(fp, 18));
            assert_eq!(bgzf_close(fp), 0);
        }
    }

    #[test]
    fn bgzf_check_eof_common_restores_seekable_hfile_offset() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-eof-position-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let payload = b"seekable eof common probe";
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(payload).unwrap();
            writer.finish().unwrap();
        }
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hseek_ptr((*fp).fp, 7, libc::SEEK_SET), 7);

            assert_eq!(bgzf_c_1542_bgzf_check_EOF_common(fp), 1);
            assert_eq!(bgzf_htell_ptr((*fp).fp), 7);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_mt_eof_records_eof_and_completes_command() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-mt-eof-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(b"mt eof command payload").unwrap();
            writer.finish().unwrap();
        }
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hseek_ptr((*fp).fp, 3, SEEK_SET), 3);

            let mut mt: bgzf_mtaux_t = std::mem::zeroed();
            assert_eq!(
                libc::pthread_mutex_init(ptr::addr_of_mut!(mt.job_pool_m), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!(mt.command_c), ptr::null()),
                0
            );
            mt.command = mtaux_cmd::HAS_EOF;
            (*fp).mt = ptr::addr_of_mut!(mt).cast();

            bgzf_c_1566_bgzf_mt_eof(fp);

            assert_eq!(mt.eof, 1);
            assert_eq!(mt.command, mtaux_cmd::HAS_EOF_DONE);
            assert_eq!(bgzf_htell_ptr((*fp).fp), 3);

            (*fp).mt = ptr::null_mut();
            assert_eq!(
                libc::pthread_cond_destroy(ptr::addr_of_mut!(mt.command_c)),
                0
            );
            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(mt.job_pool_m)),
                0
            );
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_mt_seek_resets_queue_and_seeks_underlying_hfile() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-mt-seek-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(b"mt seek command payload").unwrap();
            writer.finish().unwrap();
        }
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let pool = super::super::thread_pool::hts_tpool_init(1);
            assert!(!pool.is_null());
            let queue = super::super::thread_pool::hts_tpool_process_init(pool, 2, 0);
            assert!(!queue.is_null());

            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hseek_ptr((*fp).fp, 11, SEEK_SET), 11);

            let mut mt: bgzf_mtaux_t = std::mem::zeroed();
            assert_eq!(
                libc::pthread_mutex_init(ptr::addr_of_mut!(mt.job_pool_m), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!(mt.command_c), ptr::null()),
                0
            );
            mt.out_queue = queue.cast();
            mt.block_address = 0;
            mt.errcode = 99;
            mt.command = mtaux_cmd::SEEK;
            (*fp).mt = ptr::addr_of_mut!(mt).cast();

            bgzf_c_1583_bgzf_mt_seek(fp);

            assert_eq!(mt.errcode, 0);
            assert_eq!(mt.command, mtaux_cmd::SEEK_DONE);
            assert_eq!(bgzf_htell_ptr((*fp).fp), 0);

            (*fp).mt = ptr::null_mut();
            assert_eq!(
                libc::pthread_cond_destroy(ptr::addr_of_mut!(mt.command_c)),
                0
            );
            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(mt.job_pool_m)),
                0
            );
            assert_eq!(bgzf_close(fp), 0);
            super::super::thread_pool::hts_tpool_process_destroy(queue);
            super::super::thread_pool::hts_tpool_destroy(pool);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_job_cleanup_returns_job_to_mt_pool_under_lock() {
        unsafe {
            let pool = super::super::cram::cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<
                bgzf_job,
            >());
            assert!(!pool.is_null());

            let mut fp: BGZF = std::mem::zeroed();
            let mut mt: bgzf_mtaux_t = std::mem::zeroed();
            assert_eq!(
                libc::pthread_mutex_init(ptr::addr_of_mut!(mt.job_pool_m), ptr::null()),
                0
            );
            mt.job_pool = pool.cast();
            fp.mt = ptr::addr_of_mut!(mt).cast();

            let job =
                super::super::cram::cram_pooled_alloc_c_115_pool_alloc(pool).cast::<bgzf_job>();
            assert!(!job.is_null());
            (*job).fp = ptr::addr_of_mut!(fp);

            bgzf_c_1322_job_cleanup(job.cast());

            let reused =
                super::super::cram::cram_pooled_alloc_c_115_pool_alloc(pool).cast::<bgzf_job>();
            assert_eq!(reused, job);

            assert_eq!(
                libc::pthread_mutex_destroy(ptr::addr_of_mut!(mt.job_pool_m)),
                0
            );
            super::super::cram::cram_pooled_alloc_c_84_pool_destroy(pool);
        }
    }

    extern "C" fn mt_destroy_test_io_thread(arg: *mut c_void) -> *mut c_void {
        unsafe {
            let mt = arg.cast::<bgzf_mtaux_t>();
            libc::pthread_mutex_lock(ptr::addr_of_mut!((*mt).command_m));
            while (*mt).command != mtaux_cmd::CLOSE {
                libc::pthread_cond_wait(
                    ptr::addr_of_mut!((*mt).command_c),
                    ptr::addr_of_mut!((*mt).command_m),
                );
            }
            libc::pthread_mutex_unlock(ptr::addr_of_mut!((*mt).command_m));
        }
        ptr::null_mut()
    }

    #[test]
    fn bgzf_mt_destroy_closes_io_thread_and_releases_owned_state() {
        unsafe {
            let mt = c_compat::calloc(1, std::mem::size_of::<bgzf_mtaux_t>() as u64)
                .cast::<bgzf_mtaux_t>();
            assert!(!mt.is_null());
            assert_eq!(
                libc::pthread_mutex_init(ptr::addr_of_mut!((*mt).job_pool_m), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_mutex_init(ptr::addr_of_mut!((*mt).command_m), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_mutex_init(ptr::addr_of_mut!((*mt).idx_m), ptr::null()),
                0
            );
            assert_eq!(
                libc::pthread_cond_init(ptr::addr_of_mut!((*mt).command_c), ptr::null()),
                0
            );

            (*mt).job_pool = super::super::cram::cram_pooled_alloc_c_64_pool_create(
                std::mem::size_of::<bgzf_job>(),
            )
            .cast();
            assert!(!(*mt).job_pool.is_null());
            (*mt).curr_job =
                super::super::cram::cram_pooled_alloc_c_115_pool_alloc((*mt).job_pool.cast())
                    .cast();
            assert!(!(*mt).curr_job.is_null());
            (*mt).idx_cache.e =
                c_compat::calloc(2, std::mem::size_of::<hts_idx_cache_entry>() as u64).cast();
            assert!(!(*mt).idx_cache.e.is_null());

            (*mt).pool = super::super::thread_pool::hts_tpool_init(1).cast();
            assert!(!(*mt).pool.is_null());
            (*mt).out_queue =
                super::super::thread_pool::hts_tpool_process_init((*mt).pool.cast(), 2, 0).cast();
            assert!(!(*mt).out_queue.is_null());
            (*mt).own_pool = 1;

            assert_eq!(
                libc::pthread_create(
                    ptr::addr_of_mut!((*mt).io_task),
                    ptr::null(),
                    mt_destroy_test_io_thread,
                    mt.cast(),
                ),
                0
            );

            assert_eq!(bgzf_c_1805_mt_destroy(mt), 0);
        }
    }

    #[test]
    fn bgzf_dopen_preserves_plain_pipe_probe_bytes() {
        unsafe {
            let payload = b"plain uncompressed payload through a non-seekable fd";
            let mut fds = [0; 2];
            assert_eq!(libc::pipe(fds.as_mut_ptr()), 0);
            assert_eq!(
                libc::write(fds[1], payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(libc::close(fds[1]), 0);

            let fp = bgzf_dopen(fds[0], c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_compression(fp), 0);
            let mut out = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, out.as_mut_ptr().cast(), out.len()),
                payload.len() as isize
            );
            assert_eq!(out, payload);
            assert_eq!(bgzf_close(fp), 0);
        }
    }

    #[test]
    fn bgzf_useek_on_plain_file_discards_probe_bytes() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-plain-useek-{}-{}",
            std::process::id(),
            "read"
        ));
        let payload = b"0123456789abcdefghijklmnopqrstuvwxyz";
        std::fs::write(&path, payload).unwrap();
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_compression(fp), 0);
            assert_eq!(bgzf_useek(fp, 7, libc::SEEK_SET), 0);
            let mut out = [0u8; 5];
            assert_eq!(bgzf_read(fp, out.as_mut_ptr().cast(), out.len()), 5);
            assert_eq!(&out, &payload[7..12]);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_hopen_write_real_hfile_uses_hfile_for_eof_block() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-hfile-write-{}-{}.gz",
            std::process::id(),
            "write"
        ));
        let payload = b"real hfile bgzf write payload\n".repeat(200);
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let hfp = super::super::hfile::hopen(path_c.as_ptr(), c"w".as_ptr());
            assert!(!hfp.is_null());
            let fp = bgzf_hopen(hfp, c"w".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hfile(fp), hfp);
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);
        }

        let file = File::open(&path).unwrap();
        let mut reader = noodles::bgzf::io::Reader::new(file);
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, payload);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn crc32_matches_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn hts_crc32_matches_zlib_incremental_crc_rules() {
        unsafe {
            assert_eq!(hts_crc32(0, b"123456789".as_ptr().cast(), 9), 0xcbf4_3926);
            let first = hts_crc32(0, b"1234".as_ptr().cast(), 4);
            assert_eq!(hts_crc32(first, b"56789".as_ptr().cast(), 5), 0xcbf4_3926);
        }
    }

    #[test]
    fn bgzf_compress_writes_bgzf_block_header_payload_and_footer() {
        unsafe {
            let payload = b"literal-bgzf-payload";
            let mut out = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut dlen = out.len();
            assert_eq!(
                bgzf_compress(
                    out.as_mut_ptr().cast(),
                    &mut dlen,
                    payload.as_ptr().cast(),
                    payload.len(),
                    0,
                ),
                0
            );
            assert_eq!(
                dlen,
                BLOCK_HEADER_LENGTH + 5 + payload.len() + BLOCK_FOOTER_LENGTH
            );
            assert_eq!(check_header(out.as_ptr()), 0);
            assert_eq!(unpackInt16(out.as_ptr().add(16)), dlen as c_int - 1);
            assert_eq!(out[BLOCK_HEADER_LENGTH], 1);
            assert_eq!(
                unpackInt16(out.as_ptr().add(BLOCK_HEADER_LENGTH + 1)),
                payload.len() as c_int
            );
            assert_eq!(
                unpackInt16(out.as_ptr().add(BLOCK_HEADER_LENGTH + 3)),
                (!(payload.len() as u16)) as c_int
            );
            assert_eq!(
                &out[BLOCK_HEADER_LENGTH + 5..BLOCK_HEADER_LENGTH + 5 + payload.len()],
                payload
            );
            assert_eq!(
                unpack_u32(&out[dlen - 8..dlen - 4]),
                hts_crc32(0, payload.as_ptr().cast(), payload.len())
            );
            assert_eq!(unpack_u32(&out[dlen - 4..dlen]), payload.len() as u32);

            let compressed_payload = b"compressible payload ".repeat(40);
            let mut compressed = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut compressed_len = compressed.len();
            assert_eq!(
                bgzf_compress(
                    compressed.as_mut_ptr().cast(),
                    &mut compressed_len,
                    compressed_payload.as_ptr().cast(),
                    compressed_payload.len(),
                    6,
                ),
                0
            );
            assert_eq!(check_header(compressed.as_ptr()), 0);
            let payload_end = compressed_len - BLOCK_FOOTER_LENGTH;
            let mut decoded = vec![0u8; compressed_payload.len()];
            let mut decoder = Decompress::new(false);
            assert!(matches!(
                decoder.decompress(
                    &compressed[BLOCK_HEADER_LENGTH..payload_end],
                    &mut decoded,
                    FlushDecompress::Finish,
                ),
                Ok(Status::StreamEnd)
            ));
            assert_eq!(decoded, compressed_payload);
        }
    }

    #[test]
    fn bgzf_mt_job_encode_and_decode_round_trip_payload() {
        unsafe {
            let payload = b"bgzf mt worker payload ".repeat(200);
            let mut job: bgzf_job = std::mem::zeroed();
            ptr::copy_nonoverlapping(
                payload.as_ptr(),
                job.uncomp_data.as_mut_ptr(),
                payload.len(),
            );
            job.uncomp_len = payload.len();

            assert_eq!(
                bgzf_encode_level0_func(ptr::addr_of_mut!(job).cast()),
                ptr::addr_of_mut!(job).cast()
            );
            assert_eq!(job.errcode, 0);
            assert!(job.comp_len > BLOCK_HEADER_LENGTH + BLOCK_FOOTER_LENGTH);
            assert_eq!(check_header(job.comp_data.as_ptr()), 0);

            job.uncomp_data.fill(0);
            job.uncomp_len = 0;
            assert_eq!(
                bgzf_decode_func(ptr::addr_of_mut!(job).cast()),
                ptr::addr_of_mut!(job).cast()
            );
            assert_eq!(job.errcode, 0);
            assert_eq!(job.uncomp_len, payload.len());
            assert_eq!(&job.uncomp_data[..payload.len()], payload.as_slice());
        }
    }

    #[test]
    fn bgzf_mt_job_decode_reports_crc_errors() {
        unsafe {
            let payload = b"crc-protected worker payload";
            let mut job: bgzf_job = std::mem::zeroed();
            ptr::copy_nonoverlapping(
                payload.as_ptr(),
                job.uncomp_data.as_mut_ptr(),
                payload.len(),
            );
            job.uncomp_len = payload.len();
            bgzf_encode_level0_func(ptr::addr_of_mut!(job).cast());
            assert_eq!(job.errcode, 0);

            let block_len = unpack_u16(&job.comp_data[16..18]) as usize + 1;
            job.comp_data[block_len - 8] ^= 1;
            job.uncomp_data.fill(0);
            job.uncomp_len = 0;

            bgzf_decode_func(ptr::addr_of_mut!(job).cast());
            assert_eq!(job.errcode, BGZF_ERR_CRC as c_int);
        }
    }

    #[test]
    fn bgzf_read_rejects_bsize_smaller_than_header() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-invalid-bsize-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let payload = b"invalid bsize payload";
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            packInt16(block.as_mut_ptr().add(16), (BLOCK_HEADER_LENGTH as u16) - 2);
            std::fs::write(&path, &block[..block_len]).unwrap();

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let mut buf = [0u8; 8];
            assert_eq!(bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()), -1);
            assert_ne!(errcode(fp) & BGZF_ERR_HEADER, 0);
            assert_eq!(bgzf_close(fp), -1);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_uncompress_inflates_raw_payload_and_checks_crc() {
        unsafe {
            let payload = b"translated bgzf uncompress payload ".repeat(20);
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            let expected_crc = unpack_u32(&block[block_len - 8..block_len - 4]);
            let mut decoded = vec![0u8; BGZF_MAX_BLOCK_SIZE];
            let mut decoded_len = decoded.len();
            assert_eq!(
                bgzf_uncompress(
                    decoded.as_mut_ptr(),
                    &mut decoded_len,
                    block.as_ptr().add(BLOCK_HEADER_LENGTH),
                    block_len - BLOCK_HEADER_LENGTH,
                    expected_crc,
                ),
                0
            );
            assert_eq!(decoded_len, payload.len());
            assert_eq!(&decoded[..decoded_len], payload);

            decoded_len = decoded.len();
            assert_eq!(
                bgzf_uncompress(
                    decoded.as_mut_ptr(),
                    &mut decoded_len,
                    block.as_ptr().add(BLOCK_HEADER_LENGTH),
                    block_len - BLOCK_HEADER_LENGTH,
                    expected_crc ^ 1,
                ),
                -2
            );
        }
    }

    #[test]
    fn bgzf_read_ignores_footer_isize_like_htslib_inflate_block() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-bad-isize-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let payload = b"isize is not checked by bgzf inflate_block";
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            packInt32(block.as_mut_ptr().add(block_len - 4), 0);
            std::fs::write(&path, &block[..block_len]).unwrap();

            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut out = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, out.as_mut_ptr().cast(), out.len()),
                payload.len() as isize
            );
            assert_eq!(out, payload);
            assert_eq!(errcode(fp), 0);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_pack_unpack_helpers_match_little_endian_layout() {
        unsafe {
            let mut buf = [0u8; 4];
            packInt16(buf.as_mut_ptr(), 0xabcd);
            assert_eq!(buf[..2], [0xcd, 0xab]);
            assert_eq!(unpackInt16(buf.as_ptr()), 0xabcd);
            packInt32(buf.as_mut_ptr(), 0x1234_abcd);
            assert_eq!(buf, [0xcd, 0xab, 0x34, 0x12]);
        }
    }

    #[test]
    fn bgzf_leaf_helpers_match_c_rules() {
        unsafe {
            let header = [
                31u8, 139, 8, 4, 0, 0, 0, 0, 0, 255, 6, 0, b'B', b'C', 2, 0, 27, 0,
            ];
            assert_eq!(check_header(header.as_ptr()), 0);
            let mut no_extra = header;
            no_extra[3] = 0;
            assert_eq!(check_header(no_extra.as_ptr()), -1);
            let mut not_gzip = header;
            not_gzip[0] = 0;
            assert_eq!(check_header(not_gzip.as_ptr()), -2);

            let msg = bgzf_zerr(Z_MEM_ERROR, ptr::null_mut());
            assert_eq!(CStr::from_ptr(msg).to_bytes(), b"out of memory");
            let mut zs = z_stream_s {
                next_in: ptr::null(),
                avail_in: 0,
                total_in: 0,
                next_out: ptr::null_mut(),
                avail_out: 0,
                total_out: 0,
                msg: c"z-stream-message".as_ptr().cast_mut().cast(),
                state: ptr::null_mut(),
                zalloc: ptr::null_mut(),
                zfree: ptr::null_mut(),
                opaque: ptr::null_mut(),
                data_type: 0,
                adler: 0,
                reserved: 0,
            };
            assert_eq!(bgzf_zerr(Z_DATA_ERROR, &mut zs), zs.msg.cast());

            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: 0x123usize as *mut _,
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            assert_eq!(bgzf_compression(&mut fp), 0);
            set_flag(&mut fp, 30, true);
            assert_eq!(bgzf_compression(&mut fp), 2);
            set_flag(&mut fp, 31, true);
            assert_eq!(bgzf_compression(&mut fp), 1);
            assert_eq!(bgzf_hfile(&mut fp), 0x123usize as *mut _);

            let mut cache_marker = 0u8;
            fp.mt = ptr::null_mut();
            fp.cache = (&mut cache_marker as *mut u8).cast();
            bgzf_set_cache_size(&mut fp, 8192);
            assert_eq!(fp.cache_size, 8192);
            fp.mt = 1usize as *mut c_void;
            bgzf_set_cache_size(&mut fp, 16384);
            assert_eq!(fp.cache_size, 8192);

            let suffix = get_name_suffix(c"base".as_ptr(), c".gzi".as_ptr());
            assert!(!suffix.is_null());
            assert_eq!(CStr::from_ptr(suffix).to_bytes(), b"base.gzi");
            c_compat::free(suffix.cast());
        }
    }

    #[test]
    fn bgzf_raw_byte_and_uint64_helpers_match_c_rules() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-raw-{}-{}",
            std::process::id(),
            "io"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"wu\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_raw_write(fp, b"raw".as_ptr().cast(), 3), 3);
            assert_eq!(bgzf_flush_try(fp, BGZF_BLOCK_SIZE as isize), 0);
            assert_eq!(bgzf_close(fp), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let mut buf = [0u8; 3];
            assert_eq!(bgzf_raw_read(fp, buf.as_mut_ptr().cast(), buf.len()), 3);
            assert_eq!(&buf, b"raw");
            assert_eq!(bgzf_close(fp), 0);

            let fd = libc::open(path_c.as_ptr(), libc::O_RDWR | libc::O_TRUNC);
            assert!(fd >= 0);
            let hfile = fd_to_ptr(fd);
            assert_eq!(hwrite_uint64(0x0123_4567_89ab_cdef, hfile), 0);
            assert_eq!(libc::lseek(fd, 0, libc::SEEK_SET), 0);
            let mut x = 0;
            assert_eq!(hread_uint64(&mut x, hfile), 0);
            assert_eq!(x, 0x0123_4567_89ab_cdef);
            assert_eq!(libc::close(fd), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"w\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_index_build_init(fp), 0);
            assert_eq!(bgzf_block_write(fp, b"block".as_ptr().cast(), 5), 5);
            assert_eq!(bgzf_close(fp), 0);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_getc_utell_and_probe_match_c_rules() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-getc-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let plain = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-getc-{}-{}.txt",
            std::process::id(),
            "plain"
        ));
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(b"abcdef").unwrap();
            writer.finish().unwrap();
        }
        std::fs::write(&plain, b"plain").unwrap();

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        let plain_c = CString::new(super::super::path_bytes(&plain).as_ref()).unwrap();
        unsafe {
            assert_eq!(bgzf_is_bgzf(path_c.as_ptr()), 1);
            assert_eq!(bgzf_is_bgzf(plain_c.as_ptr()), 0);

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_utell(fp), 0);
            assert_eq!(bgzf_getc(fp), b'a' as c_int);
            assert_eq!(bgzf_getc(fp), b'b' as c_int);
            assert_eq!(bgzf_utell(fp), 2);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(plain);
    }

    #[test]
    fn bgzf_raw_io_and_uint64_helpers_accept_real_hfile_pointer() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-hfile-raw-{}-{}.bin",
            std::process::id(),
            "io"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let hfile = super::super::hfile::hopen(path_c.as_ptr(), c"w+".as_ptr());
            assert!(!hfile.is_null());

            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: hfile,
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(bgzf_raw_write(&mut fp, b"raw-io".as_ptr().cast(), 6), 6);
            assert_eq!(super::super::hfile::hflush(hfile), 0);
            assert_eq!(super::super::hfile::hseek(hfile, 0, libc::SEEK_SET), 0);

            let mut raw = [0u8; 6];
            assert_eq!(
                bgzf_raw_read(&mut fp, raw.as_mut_ptr().cast(), raw.len()),
                6
            );
            assert_eq!(&raw, b"raw-io");
            assert_eq!(errcode(&fp), 0);

            assert_eq!(super::super::hfile::hseek(hfile, 0, libc::SEEK_SET), 0);
            assert_eq!(hwrite_uint64(0x8877_6655_4433_2211, hfile), 0);
            assert_eq!(super::super::hfile::hflush(hfile), 0);
            assert_eq!(super::super::hfile::hseek(hfile, 0, libc::SEEK_SET), 0);

            let mut x = 0;
            assert_eq!(hread_uint64(&mut x, hfile), 0);
            assert_eq!(x, 0x8877_6655_4433_2211);
            assert_eq!(super::super::hfile::hclose(hfile), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_check_eof_uses_real_hfile_pointer_and_restores_offset() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-hfile-eof-{}-{}.gz",
            std::process::id(),
            "check"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        let mut raw = b"prefix".to_vec();
        raw.extend_from_slice(&EOF_BLOCK);
        std::fs::write(&path, raw).unwrap();

        unsafe {
            let hfile = super::super::hfile::hopen(path_c.as_ptr(), c"r".as_ptr());
            assert!(!hfile.is_null());
            assert_eq!(super::super::hfile::hseek(hfile, 3, libc::SEEK_SET), 3);

            let mut fp = BGZF {
                bitfields: 1 << 30,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: hfile,
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(bgzf_check_EOF(&mut fp), 1);
            assert_eq!(super::super::hfile::hseek(hfile, 0, libc::SEEK_CUR), 3);
            assert!(!flag(&fp, 18));
            assert_eq!(errcode(&fp), 0);
            assert_eq!(super::super::hfile::hclose(hfile), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_check_eof_absent_marker_restores_offset() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-no-eof-{}-{}.gz",
            std::process::id(),
            "check"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let payload = b"bgzf data without terminal eof marker";
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            assert!(block_len > EOF_BLOCK.len());
            std::fs::write(&path, &block[..block_len]).unwrap();

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_hseek_ptr((*fp).fp, 7, libc::SEEK_SET), 7);
            assert_eq!(bgzf_check_EOF(fp), 0);
            assert_eq!(bgzf_htell_ptr((*fp).fp), 7);
            assert!(flag(fp, 18));
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_check_eof_short_file_returns_zero_and_preserves_offset() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-short-eof-{}-{}.gz",
            std::process::id(),
            "check"
        ));
        std::fs::write(&path, b"short").unwrap();
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let hfile = super::super::hfile::hopen(path_c.as_ptr(), c"r".as_ptr());
            assert!(!hfile.is_null());
            assert_eq!(super::super::hfile::hseek(hfile, 2, libc::SEEK_SET), 2);

            let mut fp = BGZF {
                bitfields: 1 << 30,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: hfile,
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(bgzf_check_EOF(&mut fp), 0);
            assert_eq!(super::super::hfile::hseek(hfile, 0, libc::SEEK_CUR), 2);
            assert!(flag(&fp, 18));
            assert_eq!(errcode(&fp), 0);
            assert_eq!(super::super::hfile::hclose(hfile), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_read_past_terminal_empty_block_returns_short_then_eof() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-terminal-eof-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let payload = b"terminal empty bgzf block";
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());

            let mut buf = [0u8; 64];
            assert_eq!(
                bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()),
                payload.len() as isize
            );
            assert_eq!(&buf[..payload.len()], payload);
            assert_eq!(errcode(fp), 0);

            assert_eq!(bgzf_read(fp, buf.as_mut_ptr().cast(), 1), 0);
            assert_eq!(errcode(fp), 0);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_getline_reads_lines_and_trims_cr_before_newline() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-getline-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(b"alpha\r\nbeta\nlast").unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: ptr::null_mut(),
            };

            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), 5);
            assert_eq!(CStr::from_ptr(line.s).to_bytes(), b"alpha");
            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), 4);
            assert_eq!(CStr::from_ptr(line.s).to_bytes(), b"beta");
            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), 4);
            assert_eq!(CStr::from_ptr(line.s).to_bytes(), b"last");
            assert_eq!(bgzf_getline(fp, b'\n' as c_int, &mut line), -1);

            c_compat::free(line.s.cast());
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_index_build_add_and_destroy_match_c_state_transitions() {
        unsafe {
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 123,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 99,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(bgzf_index_build_init(&mut fp), 0);
            assert!(!fp.idx.is_null());
            assert_eq!(fp.idx_build_otf, 1);

            let idx = fp.idx.cast::<bgzidx_t>();
            (*idx).ublock_addr = 42;
            assert_eq!(bgzf_index_add_block(&mut fp), 0);
            assert_eq!((*idx).noffs, 1);
            assert!((*idx).moffs >= 1);
            assert!(!(*idx).offs.is_null());
            assert_eq!((*(*idx).offs.add(0)).uaddr, 42);
            assert_eq!((*(*idx).offs.add(0)).caddr, 123);

            fp.block_address = 456;
            (*idx).ublock_addr = 1000;
            assert_eq!(bgzf_index_add_block(&mut fp), 0);
            assert_eq!((*idx).noffs, 2);
            assert!((*idx).moffs >= 2);
            assert_eq!((*(*idx).offs.add(1)).uaddr, 1000);
            assert_eq!((*(*idx).offs.add(1)).caddr, 456);

            bgzf_index_destroy(&mut fp);
            assert!(fp.idx.is_null());
            assert_eq!(fp.idx_build_otf, 0);
        }
    }

    #[test]
    fn bgzf_mt_index_flush_uses_virtual_offsets_and_keeps_remaining_cache() {
        unsafe {
            let hidx = super::super::hts::hts_idx_init(1, super::super::hts::HTS_FMT_BAI, 0, 14, 5);
            assert!(!hidx.is_null());

            let mut mt: bgzf_mtaux_t = std::mem::zeroed();
            assert_eq!(libc::pthread_mutex_init(&mut mt.idx_m, ptr::null()), 0);
            mt.hts_idx = hidx;
            mt.block_address = 7;
            mt.block_written = 3;
            mt.idx_cache.nentries = 3;
            mt.idx_cache.mentries = 3;
            mt.idx_cache.e = c_compat::calloc(3, std::mem::size_of::<hts_idx_cache_entry>() as u64)
                .cast::<hts_idx_cache_entry>();
            assert!(!mt.idx_cache.e.is_null());

            let entries = [
                hts_idx_cache_entry {
                    beg: 0,
                    end: 10,
                    tid: 0,
                    is_mapped: 1,
                    offset: 5,
                    block_number: 3,
                },
                hts_idx_cache_entry {
                    beg: 10,
                    end: 20,
                    tid: 0,
                    is_mapped: 1,
                    offset: 100,
                    block_number: 3,
                },
                hts_idx_cache_entry {
                    beg: 20,
                    end: 30,
                    tid: 0,
                    is_mapped: 1,
                    offset: 1,
                    block_number: 4,
                },
            ];
            ptr::copy_nonoverlapping(entries.as_ptr(), mt.idx_cache.e, entries.len());

            let fp_idx = c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64);
            assert!(!fp_idx.is_null());
            (*(fp_idx.cast::<bgzidx_t>())).ublock_addr = 55;
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 999,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: (&mut mt as *mut bgzf_mtaux_t).cast(),
                idx: fp_idx,
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(bgzf_c_228_bgzf_idx_flush(&mut fp, 100, 11), 0);
            assert_eq!(mt.block_written, 4);
            assert_eq!(mt.idx_cache.nentries, 1);
            assert_eq!((*mt.idx_cache.e).block_number, 4);
            assert_eq!((*hidx).z.last_off, 18_u64 << 16);
            assert_eq!(fp.block_address, 999);
            assert_eq!((*(fp_idx.cast::<bgzidx_t>())).ublock_addr, 55);

            c_compat::free(mt.idx_cache.e.cast());
            c_compat::free(fp_idx);
            libc::pthread_mutex_destroy(&mut mt.idx_m);
            super::super::hts::hts_idx_destroy(hidx);
        }
    }

    #[test]
    fn bgzf_mt_index_push_caches_offsets_until_matching_block_flush() {
        unsafe {
            let hidx = super::super::hts::hts_idx_init(1, super::super::hts::HTS_FMT_BAI, 0, 14, 5);
            assert!(!hidx.is_null());

            let mut mt: bgzf_mtaux_t = std::mem::zeroed();
            assert_eq!(libc::pthread_mutex_init(&mut mt.idx_m, ptr::null()), 0);
            mt.block_number = 12;
            mt.block_written = 12;
            mt.block_address = 40;
            mt.idx_cache.mentries = 1;
            mt.idx_cache.e = c_compat::calloc(1, std::mem::size_of::<hts_idx_cache_entry>() as u64)
                .cast::<hts_idx_cache_entry>();
            assert!(!mt.idx_cache.e.is_null());

            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 999,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: (&mut mt as *mut bgzf_mtaux_t).cast(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(
                bgzf_c_189_bgzf_idx_push(&mut fp, hidx, 0, 0, 10, (77 << 16) + 3, 1),
                0
            );
            assert_eq!(mt.idx_cache.nentries, 1);
            assert_eq!(mt.idx_cache.mentries, 1);
            assert_eq!(mt.hts_idx, hidx);
            assert_eq!((*mt.idx_cache.e).tid, 0);
            assert_eq!((*mt.idx_cache.e).beg, 0);
            assert_eq!((*mt.idx_cache.e).end, 10);
            assert_eq!((*mt.idx_cache.e).is_mapped, 1);
            assert_eq!((*mt.idx_cache.e).offset, 3);
            assert_eq!((*mt.idx_cache.e).block_number, 12);
            assert_eq!((*hidx).z.last_off, 0);

            mt.block_number = 13;
            assert_eq!(
                bgzf_c_189_bgzf_idx_push(&mut fp, hidx, 0, 20, 30, (88 << 16) + 0x2345, 0),
                0
            );
            assert_eq!(mt.idx_cache.nentries, 2);
            assert!(mt.idx_cache.mentries >= 2);
            assert_eq!((*mt.idx_cache.e.add(1)).offset, 0x2345);
            assert_eq!((*mt.idx_cache.e.add(1)).block_number, 13);
            assert_eq!((*hidx).z.last_off, 0);

            assert_eq!(bgzf_c_228_bgzf_idx_flush(&mut fp, 100, 5), 0);
            assert_eq!(mt.block_written, 13);
            assert_eq!(mt.idx_cache.nentries, 1);
            assert_eq!((*mt.idx_cache.e).beg, 20);
            assert_eq!((*mt.idx_cache.e).offset, 0x2345);
            assert_eq!((*mt.idx_cache.e).block_number, 13);
            assert_eq!((*hidx).z.last_off, (40 << 16) + 3);

            mt.block_address = 45;
            assert_eq!(bgzf_c_228_bgzf_idx_flush(&mut fp, 100, 5), 0);
            assert_eq!(mt.block_written, 14);
            assert_eq!(mt.idx_cache.nentries, 0);
            assert_eq!((*hidx).z.last_off, (45 << 16) + 0x2345);
            assert_eq!((*hidx).z.n_mapped, 1);
            assert_eq!((*hidx).z.n_unmapped, 1);

            c_compat::free(mt.idx_cache.e.cast());
            libc::pthread_mutex_destroy(&mut mt.idx_m);
            super::super::hts::hts_idx_destroy(hidx);
        }
    }

    #[test]
    fn bgzf_index_dump_and_load_round_trip_gzi_layout() {
        let base = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-index-{}-{}",
            std::process::id(),
            "roundtrip"
        ));
        let index_path = base.with_extension("gzi");
        let hfile_index_path = base.with_extension("hfile.gzi");
        let base_c = CString::new(super::super::path_bytes(&base).as_ref()).unwrap();
        let hfile_index_c =
            CString::new(super::super::path_bytes(&hfile_index_path).as_ref()).unwrap();

        unsafe {
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            assert!(!fp.idx.is_null());
            let idx = fp.idx.cast::<bgzidx_t>();
            (*idx).noffs = 3;
            (*idx).moffs = 3;
            (*idx).offs =
                c_compat::calloc(3, std::mem::size_of::<bgzidx1_t>() as u64).cast::<bgzidx1_t>();
            assert!(!(*idx).offs.is_null());
            (*(*idx).offs.add(1)).caddr = 11;
            (*(*idx).offs.add(1)).uaddr = 101;
            (*(*idx).offs.add(2)).caddr = 22;
            (*(*idx).offs.add(2)).uaddr = 202;

            assert_eq!(
                bgzf_index_dump(&mut fp, base_c.as_ptr(), c".gzi".as_ptr()),
                0
            );

            let mut loaded = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };

            assert_eq!(
                bgzf_index_load(&mut loaded, base_c.as_ptr(), c".gzi".as_ptr()),
                0
            );
            let loaded_idx = loaded.idx.cast::<bgzidx_t>();
            assert_eq!((*loaded_idx).noffs, 3);
            assert_eq!((*loaded_idx).moffs, 3);
            assert_eq!((*(*loaded_idx).offs.add(0)).caddr, 0);
            assert_eq!((*(*loaded_idx).offs.add(0)).uaddr, 0);
            assert_eq!((*(*loaded_idx).offs.add(1)).caddr, 11);
            assert_eq!((*(*loaded_idx).offs.add(1)).uaddr, 101);
            assert_eq!((*(*loaded_idx).offs.add(2)).caddr, 22);
            assert_eq!((*(*loaded_idx).offs.add(2)).uaddr, 202);

            bgzf_index_destroy(&mut fp);
            bgzf_index_destroy(&mut loaded);

            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: c_compat::calloc(1, std::mem::size_of::<bgzidx_t>() as u64),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            assert!(!fp.idx.is_null());
            let idx = fp.idx.cast::<bgzidx_t>();
            (*idx).noffs = 2;
            (*idx).moffs = 2;
            (*idx).offs =
                c_compat::calloc(2, std::mem::size_of::<bgzidx1_t>() as u64).cast::<bgzidx1_t>();
            assert!(!(*idx).offs.is_null());
            (*(*idx).offs.add(1)).caddr = 333;
            (*(*idx).offs.add(1)).uaddr = 444;

            let hfile = super::super::hfile::hopen(hfile_index_c.as_ptr(), c"w".as_ptr());
            assert!(!hfile.is_null());
            assert_eq!(
                bgzf_index_dump_hfile(&mut fp, hfile, hfile_index_c.as_ptr()),
                0
            );
            assert_eq!(super::super::hfile::hclose(hfile), 0);

            let mut loaded = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: ptr::null_mut(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            let hfile = super::super::hfile::hopen(hfile_index_c.as_ptr(), c"r".as_ptr());
            assert!(!hfile.is_null());
            assert_eq!(
                bgzf_index_load_hfile(&mut loaded, hfile, hfile_index_c.as_ptr()),
                0
            );
            assert_eq!(super::super::hfile::hclose(hfile), 0);

            let loaded_idx = loaded.idx.cast::<bgzidx_t>();
            assert_eq!((*loaded_idx).noffs, 2);
            assert_eq!((*(*loaded_idx).offs.add(0)).caddr, 0);
            assert_eq!((*(*loaded_idx).offs.add(0)).uaddr, 0);
            assert_eq!((*(*loaded_idx).offs.add(1)).caddr, 333);
            assert_eq!((*(*loaded_idx).offs.add(1)).uaddr, 444);

            bgzf_index_destroy(&mut fp);
            bgzf_index_destroy(&mut loaded);
        }

        let _ = std::fs::remove_file(index_path);
        let _ = std::fs::remove_file(hfile_index_path);
    }

    #[test]
    fn bgzf_read_error_preserves_specific_crc_bit_when_read_adds_zlib_bit() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-bad-crc-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let payload = b"crc-specific bgzf error";
            let mut block = [0u8; BGZF_MAX_BLOCK_SIZE];
            let mut block_len = block.len();
            assert_eq!(
                bgzf_compress(
                    block.as_mut_ptr().cast(),
                    &mut block_len,
                    payload.as_ptr().cast(),
                    payload.len(),
                    6,
                ),
                0
            );
            block[block_len - 8] ^= 1;
            std::fs::write(&path, &block[..block_len]).unwrap();

            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let mut buf = [0u8; 8];
            assert_eq!(bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()), -1);
            assert_ne!(errcode(fp) & BGZF_ERR_CRC, 0);
            assert_ne!(errcode(fp) & BGZF_ERR_ZLIB, 0);

            assert_eq!(bgzf_read(fp, buf.as_mut_ptr().cast(), buf.len()), -1);
            assert_ne!(errcode(fp) & BGZF_ERR_CRC, 0);
            assert_eq!(bgzf_close(fp), -1);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_useek_within_loaded_block_rewinds_buffer_without_io_or_index() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-useek-buffer-{}-{}.gz",
            std::process::id(),
            "read"
        ));
        let payload = b"0123456789abcdef";
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());

            let mut head = [0u8; 6];
            assert_eq!(bgzf_read(fp, head.as_mut_ptr().cast(), head.len()), 6);
            assert_eq!(&head, b"012345");
            assert_eq!(bgzf_utell(fp), 6);

            assert_eq!(bgzf_useek(fp, 2, libc::SEEK_SET), 0);
            assert_eq!(bgzf_utell(fp), 2);

            let mut reread = [0u8; 4];
            assert_eq!(bgzf_read(fp, reread.as_mut_ptr().cast(), reread.len()), 4);
            assert_eq!(&reread, b"2345");
            assert_eq!(bgzf_utell(fp), 6);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn uncompressed_write_mode_tracks_raw_block_boundary_accounting() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-raw-boundary-{}-{}",
            std::process::id(),
            "write"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        let payload: Vec<u8> = (0..BGZF_MAX_BLOCK_SIZE + 3)
            .map(|i| (i % 251) as u8)
            .collect();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), b"wu\0".as_ptr().cast());
            assert!(!fp.is_null());
            assert_eq!(bgzf_compression(fp), 0);
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!((*fp).block_address, BGZF_MAX_BLOCK_SIZE as i64);
            assert_eq!((*fp).block_offset, 3);
            assert_eq!(bgzf_close(fp), 0);
        }

        let mut written = Vec::new();
        File::open(&path)
            .unwrap()
            .read_to_end(&mut written)
            .unwrap();
        assert_eq!(written, payload);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_hopen_uncompressed_write_flushes_real_hfile_like_c() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-hfile-raw-flush-{}-{}",
            std::process::id(),
            "write"
        ));
        let payload = b"uncompressed hfile-backed bgzf payload";
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let hfile = super::super::hfile::hopen(path_c.as_ptr(), c"w".as_ptr());
            assert!(!hfile.is_null());
            let fp = bgzf_hopen(hfile, c"wu".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_compression(fp), 0);
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_flush(fp), 0);
            assert_eq!(bgzf_close(fp), 0);
        }

        let mut written = Vec::new();
        File::open(&path)
            .unwrap()
            .read_to_end(&mut written)
            .unwrap();
        assert_eq!(written, payload);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_lazy_flush_writes_partial_block_and_keeps_stream_writable() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-lazy-flush-{}-{}.gz",
            std::process::id(),
            "write"
        ));
        let first = b"first partial bgzf block\n";
        let second = b"second block after lazy flush\n";
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(
                bgzf_write(fp, first.as_ptr().cast(), first.len()),
                first.len() as isize
            );
            assert_eq!((*fp).block_offset, first.len() as c_int);
            assert_eq!(bgzf_c_1942_lazy_flush(fp), 0);
            assert_eq!((*fp).block_offset, 0);
            assert!((*fp).block_address > 0);

            assert_eq!(
                bgzf_write(fp, second.as_ptr().cast(), second.len()),
                second.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);
        }

        let mut observed = vec![0u8; first.len() + second.len()];
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(
                bgzf_read(fp, observed.as_mut_ptr().cast(), observed.len()),
                observed.len() as isize
            );
            assert_eq!(&observed[..first.len()], first);
            assert_eq!(&observed[first.len()..], second);
            assert_eq!(bgzf_check_EOF(fp), 1);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_mt_writes_and_reads_round_trip_through_thread_pool() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-mt-roundtrip-{}-{}.gz",
            std::process::id(),
            "write"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        let payload: Vec<u8> = (0..(BGZF_BLOCK_SIZE * 2 + 137))
            .map(|i| (i % 251) as u8)
            .collect();

        unsafe {
            assert_eq!(bgzf_mt(ptr::null_mut(), 2, 256), -1);
            assert_eq!(*c_compat::__errno_location(), libc::EINVAL);

            let fp = bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_mt(fp, 0, 256), 0);
            assert_eq!(bgzf_mt(fp, 1, 256), 0);
            assert!((*fp).mt.is_null());
            assert_eq!(errcode(fp), 0);
            assert_eq!(bgzf_mt(fp, 2, 2), 0);
            assert!(!(*fp).mt.is_null());
            assert_eq!(errcode(fp), 0);
            assert_eq!(bgzf_mt(fp, 2, 256), -2);
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);

            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_mt(fp, 2, 2), 0);
            assert!(!(*fp).mt.is_null());
            let mut observed = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, observed.as_mut_ptr().cast(), observed.len()),
                observed.len() as isize
            );
            assert_eq!(observed, payload);
            assert_eq!(
                bgzf_read(fp, observed.as_mut_ptr().cast(), observed.len()),
                0
            );
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_thread_pool_installs_external_pool_and_closes_without_owning_it() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-thread-pool-{}-{}.gz",
            std::process::id(),
            "write"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        let payload = b"external thread pool bgzf payload";

        unsafe {
            let pool = super::super::thread_pool::hts_tpool_init(2);
            assert!(!pool.is_null());

            assert_eq!(bgzf_thread_pool(ptr::null_mut(), pool, 0), -1);
            assert_eq!(*c_compat::__errno_location(), libc::EINVAL);

            let fp = bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_thread_pool(fp, pool, 4), 0);
            assert!(!(*fp).mt.is_null());
            assert_eq!(errcode(fp), 0);
            assert_eq!(bgzf_thread_pool(fp, pool, 0), -2);
            assert_eq!(
                bgzf_write(fp, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(bgzf_close(fp), 0);

            super::super::thread_pool::hts_tpool_destroy(pool);

            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut observed = vec![0u8; payload.len()];
            assert_eq!(
                bgzf_read(fp, observed.as_mut_ptr().cast(), observed.len()),
                observed.len() as isize
            );
            assert_eq!(observed, payload);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bgzf_thread_pool_noops_on_uncompressed_handle_without_pool() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-thread-pool-uncompressed-{}-{}",
            std::process::id(),
            "write"
        ));
        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();

        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"wu".as_ptr());
            assert!(!fp.is_null());
            assert_eq!(bgzf_compression(fp), 0);
            assert_eq!(bgzf_thread_pool(fp, ptr::null_mut(), 0), 0);
            assert!((*fp).mt.is_null());
            assert_eq!(errcode(fp), 0);
            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    static PRIVATE_DATA_CLEANUPS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn count_private_data_cleanup(data: *mut c_void) {
        PRIVATE_DATA_CLEANUPS.fetch_add(1, Ordering::SeqCst);
        drop(Box::from_raw(data.cast::<usize>()));
    }

    #[test]
    fn bgzf_private_data_cleanup_is_called_once_and_clears_pointer() {
        unsafe {
            PRIVATE_DATA_CLEANUPS.store(0, Ordering::SeqCst);
            let mut cache = bgzf_cache_t {
                h: ptr::null_mut(),
                last_pos: 0,
                private_data: ptr::null_mut(),
                private_data_cleanup: None,
            };
            let mut fp = BGZF {
                bitfields: 0,
                cache_size: 0,
                block_length: 0,
                block_clength: 0,
                block_offset: 0,
                block_address: 0,
                uncompressed_address: 0,
                uncompressed_block: ptr::null_mut(),
                compressed_block: ptr::null_mut(),
                cache: (&mut cache as *mut bgzf_cache_t).cast(),
                fp: ptr::null_mut(),
                mt: ptr::null_mut(),
                idx: ptr::null_mut(),
                idx_build_otf: 0,
                gz_stream: ptr::null_mut(),
                seeked: 0,
            };
            let data = Box::into_raw(Box::new(17usize)).cast::<c_void>();

            bgzf_internal_h_51_bgzf_set_private_data(
                &mut fp,
                data,
                Some(count_private_data_cleanup),
            );
            assert_eq!(bgzf_internal_h_67_bgzf_get_private_data(&mut fp), data);

            bgzf_internal_h_58_bgzf_clear_private_data(&mut fp);
            assert!(bgzf_internal_h_67_bgzf_get_private_data(&mut fp).is_null());
            assert_eq!(PRIVATE_DATA_CLEANUPS.load(Ordering::SeqCst), 1);

            bgzf_internal_h_58_bgzf_clear_private_data(&mut fp);
            assert_eq!(PRIVATE_DATA_CLEANUPS.load(Ordering::SeqCst), 1);
        }
    }

    // Regression test for read-side on-the-fly index population (htslib/bgzf.c:1233).
    // With idx_build_otf enabled, native reads must record each block boundary so
    // that a subsequent bgzf_useek back to an earlier virtual offset works.
    #[test]
    fn bgzf_useek_works_after_native_read_with_otf_index() {
        let path = std::env::temp_dir().join(format!(
            "cellsnp-lite-bgzf-otf-{}-{}.gz",
            std::process::id(),
            "useek"
        ));
        // Payload spanning several bgzf blocks so the OTF index has many entries.
        let payload: Vec<u8> = (0..(BGZF_BLOCK_SIZE * 5 + 12345))
            .map(|i| (i as u32).wrapping_mul(2654435761) as u8)
            .collect();
        {
            let file = File::create(&path).unwrap();
            let mut writer = noodles::bgzf::io::Writer::new(file);
            writer.write_all(&payload).unwrap();
            writer.finish().unwrap();
        }

        let path_c = CString::new(super::super::path_bytes(&path).as_ref()).unwrap();
        unsafe {
            let fp = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            // Enable on-the-fly index building during reading.
            assert_eq!(bgzf_index_build_init(fp), 0);
            assert_eq!((*fp).idx_build_otf, 1);

            // Read forward through several blocks natively, in chunks. This must
            // populate the OTF read index via bgzf_read_block.
            let chunk = BGZF_BLOCK_SIZE + 777;
            let mut total = 0usize;
            let mut buf = vec![0u8; chunk];
            while total < BGZF_BLOCK_SIZE * 3 + 500 {
                let got = bgzf_read(fp, buf.as_mut_ptr().cast(), chunk);
                assert!(got > 0, "native read returned {got}");
                assert_eq!(&buf[..got as usize], &payload[total..total + got as usize]);
                total += got as usize;
            }

            // Index must have recorded multiple block boundaries.
            let idx = (*fp).idx.cast::<bgzidx_t>();
            assert!(
                (*idx).noffs >= 3,
                "expected >=3 recorded blocks, got {}",
                (*idx).noffs
            );

            // Seek back to an earlier virtual offset that lives in a prior block
            // (well before the current position) and confirm bytes match.
            let target = (BGZF_BLOCK_SIZE / 2) as i64;
            assert_eq!(
                bgzf_useek(fp, target, SEEK_SET),
                0,
                "bgzf_useek back failed after native read"
            );
            let mut back = vec![0u8; 4096];
            assert_eq!(
                bgzf_read(fp, back.as_mut_ptr().cast(), back.len()),
                back.len() as isize
            );
            assert_eq!(
                &back[..],
                &payload[target as usize..target as usize + back.len()]
            );

            // Seek forward to a far offset (in a later block) and verify too.
            let target2 = (BGZF_BLOCK_SIZE * 4 + 100) as i64;
            assert_eq!(bgzf_useek(fp, target2, SEEK_SET), 0);
            let mut fwd = vec![0u8; 2048];
            assert_eq!(
                bgzf_read(fp, fwd.as_mut_ptr().cast(), fwd.len()),
                fwd.len() as isize
            );
            assert_eq!(
                &fwd[..],
                &payload[target2 as usize..target2 as usize + fwd.len()]
            );

            assert_eq!(bgzf_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }
}
