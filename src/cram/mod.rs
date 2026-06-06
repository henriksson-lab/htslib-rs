use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    io::Read,
    ptr::NonNull,
};

use crate::htslib_rs::bgzf::{bgzf_close, bgzf_index_load, bgzf_open, bgzf_read, bgzf_useek};
use crate::htslib_rs::c_compat::{
    __errno_location, calloc, free, malloc, memcpy, realloc, strdup, EINVAL, ENOMEM,
};
use crate::htslib_rs::faidx::fai_build;
use crate::htslib_rs::hfile::{hclose, hclose_abruptly, hgets, hisremote, hopen};
use crate::htslib_rs::hfile::{
    htslib_hfile_h_163_hgetc, htslib_hfile_h_247_hread, htslib_hfile_h_292_hwrite,
};
use crate::htslib_rs::hts::{
    cram_fd, hFILE, htsFile, hts_fmt_option, hts_log_cstr, isspace_c, kputc, kputll, kputsn, kputw,
    kstring_t, toupper_c, BGZF, CRAM_OPT_POS_DELTA, CRAM_OPT_RANGE_NOSEEK, CRAM_OPT_USE_ARITH,
    CRAM_OPT_USE_FQZ, CRAM_OPT_USE_TOK, HTS_FORMAT_CRAM, HTS_IDX_NOCOOR, HTS_IDX_REST,
    HTS_IDX_START, HTS_LOG_ERROR, HTS_LOG_INFO, HTS_LOG_WARNING, HTS_OPT_PROFILE,
    HTS_PROFILE_ARCHIVE, HTS_PROFILE_FAST, HTS_PROFILE_NORMAL, HTS_PROFILE_SMALL,
};
use crate::htslib_rs::sam::{
    bam1_t, bam_aux_get, bam_cigar_type, bam_destroy1, sam_hdr_destroy, sam_hdr_dup, sam_hdr_t,
    BAM_CIGAR_MASK, BAM_CIGAR_SHIFT, BAM_FDUP, BAM_FPAIRED, BAM_FPROPER_PAIR, BAM_FQCFAIL,
    BAM_FREAD1, BAM_FREAD2, BAM_FREVERSE, BAM_FSECONDARY, BAM_FUNMAP,
};
use crate::htslib_rs::thread_pool::{
    hts_tpool_init, hts_tpool_process, hts_tpool_process_flush, hts_tpool_process_init,
    hts_tpool_size,
};

// Submodule split (2026-06-01): functions per htslib C source file.
// File names mirror htslib's source layout: htslib/cram/cram_io.c ->
// src/cram/cram_io.rs, etc.
pub mod cram_codecs;
pub mod cram_decode;
pub mod cram_encode;
pub mod cram_external;
pub mod cram_index;
pub mod cram_io;
pub mod cram_stats;
#[path = "mFILE.rs"]
pub mod mfile;
pub mod open_trace_file;
pub mod pooled_alloc;
pub mod string_alloc;

pub use cram_codecs::*;
pub use cram_decode::*;
pub use cram_encode::*;
pub use cram_external::*;
pub use cram_index::*;
pub use cram_io::*;
pub use cram_stats::*;
pub use mfile::*;
pub use open_trace_file::*;
pub use pooled_alloc::*;
pub use string_alloc::*;

// Native opaque CRAM types. Byte layouts are unchanged from hts_sys's
// bindgen-emitted versions; we only use these as raw `*mut` pointer types
// in this file (every dereference goes through one of the layout-mirror
// structs further down — `cram_block_layout`, `cram_container_layout`,
// `cram_slice_layout`, `cram_fd_layout`, `cram_metrics_layout`). Defining
// them locally retires ~270 `hts_sys::cram_*` references throughout cram.rs.
#[repr(C)]
pub struct cram_block {
    _private: [u8; 0],
}
#[repr(C)]
pub struct cram_container {
    _private: [u8; 0],
}
#[repr(C)]
pub struct cram_block_compression_hdr {
    _private: [u8; 0],
}
#[repr(C)]
pub struct cram_block_slice_hdr {
    _private: [u8; 0],
}
#[repr(C)]
pub struct cram_metrics {
    _private: [u8; 0],
}
#[repr(C)]
pub struct cram_slice {
    _private: [u8; 0],
}
#[repr(C)]
pub struct cram_record {
    _private: [u8; 0],
}
#[repr(C)]
pub struct refs_t {
    _private: [u8; 0],
}

// Value-type aliases — `cram_content_type` and `cram_block_method` are
// `c_int` enums in hts-sys bindings.
pub type cram_content_type = c_int;
pub type cram_block_method = c_int;

// Native cram_block_method enum values (cram/cram_structs.h).
pub const CRAM_BLOCK_METHOD_RAW: cram_block_method = 0;
pub const CRAM_BLOCK_METHOD_GZIP: cram_block_method = 1;
pub const CRAM_BLOCK_METHOD_BZIP2: cram_block_method = 2;
pub const CRAM_BLOCK_METHOD_LZMA: cram_block_method = 3;
pub const CRAM_BLOCK_METHOD_RANS: cram_block_method = 4;

// Native cram_content_type enum values (cram/cram_structs.h).
pub const CRAM_CONTENT_TYPE_FILE_HEADER: cram_content_type = 0;
pub const CRAM_CONTENT_TYPE_COMPRESSION_HEADER: cram_content_type = 1;
pub const CRAM_CONTENT_TYPE_MAPPED_SLICE: cram_content_type = 2;
pub const CRAM_CONTENT_TYPE_UNMAPPED_SLICE: cram_content_type = 3;
pub const CRAM_CONTENT_TYPE_EXTERNAL: cram_content_type = 4;
pub const CRAM_CONTENT_TYPE_CORE: cram_content_type = 5;

// Native cram_DS_ID enum values used by the container/slice primitives
// (htslib/cram/cram_structs.h). Only the ones referenced here are declared.
const DS_AUX: c_int = 1;
const DS_RN: c_int = 11;
const DS_QS: c_int = 12;
const DS_IN: c_int = 13;
const DS_SC: c_int = 14;
const DS_TN: c_int = 39;

// Native hts_fmt_option enum values (htslib/hts.h) not already defined in hts.rs.
pub const CRAM_OPT_DECODE_MD: hts_fmt_option = 0;
pub const CRAM_OPT_PREFIX: hts_fmt_option = 1;
pub const CRAM_OPT_VERBOSITY: hts_fmt_option = 2;
pub const CRAM_OPT_SEQS_PER_SLICE: hts_fmt_option = 3;
pub const CRAM_OPT_SLICES_PER_CONTAINER: hts_fmt_option = 4;
pub const CRAM_OPT_RANGE: hts_fmt_option = 5;
pub const CRAM_OPT_VERSION: hts_fmt_option = 6;
pub const CRAM_OPT_EMBED_REF: hts_fmt_option = 7;
pub const CRAM_OPT_IGNORE_MD5: hts_fmt_option = 8;
pub const CRAM_OPT_REFERENCE: hts_fmt_option = 9;
pub const CRAM_OPT_MULTI_SEQ_PER_SLICE: hts_fmt_option = 10;
pub const CRAM_OPT_NO_REF: hts_fmt_option = 11;
pub const CRAM_OPT_USE_BZIP2: hts_fmt_option = 12;
pub const CRAM_OPT_SHARED_REF: hts_fmt_option = 13;
pub const CRAM_OPT_NTHREADS: hts_fmt_option = 14;
pub const CRAM_OPT_THREAD_POOL: hts_fmt_option = 15;
pub const CRAM_OPT_USE_LZMA: hts_fmt_option = 16;
pub const CRAM_OPT_USE_RANS: hts_fmt_option = 17;
pub const CRAM_OPT_REQUIRED_FIELDS: hts_fmt_option = 18;
pub const CRAM_OPT_LOSSY_NAMES: hts_fmt_option = 19;
pub const CRAM_OPT_BASES_PER_SLICE: hts_fmt_option = 20;
pub const CRAM_OPT_STORE_MD: hts_fmt_option = 21;
pub const CRAM_OPT_STORE_NM: hts_fmt_option = 22;
pub const HTS_OPT_COMPRESSION_LEVEL: hts_fmt_option = 100;

// Native sam_fields bitflags (htslib/sam.h).
pub const SAM_RNAME: c_uint = 4;
pub const SAM_POS: c_uint = 8;
pub const SAM_CIGAR: c_uint = 32;

// Native HTS_IDX_DELIM separator (htslib/hts.h).
pub(crate) const HTS_IDX_DELIM: &[u8; 8] = b"##idx##\0";

#[derive(Clone, Copy)]
struct cram_ds_list {
    data_series: c_int,
    next: c_int,
}

pub struct cram_cid2ds_t {
    ds: Vec<cram_ds_list>,
    hash: HashMap<c_int, c_int>,
    ds_a: Vec<c_int>,
}

#[repr(C)]
pub struct cram_method_details {
    pub method: cram_block_method,
    pub level: c_int,
    pub order: c_int,
    pub rle: c_int,
    pub pack: c_int,
    pub stripe: c_int,
    pub cat: c_int,
    pub nosz: c_int,
    pub nway: c_int,
    pub ext: c_int,
}

#[repr(C)]
pub struct cram_codec {
    _private: [u8; 0],
}

#[inline]
pub(crate) const fn cram_fn_ptr(addr: usize) -> *mut c_void {
    addr as *mut c_void
}

#[inline]
pub(crate) const fn cram_data_series_id_ptr(id: usize) -> *mut c_void {
    id as *mut c_void
}

#[inline]
pub(crate) unsafe fn cram_fn<T: Copy>(ptr: *mut c_void) -> T {
    debug_assert!(!ptr.is_null());
    std::mem::transmute_copy(&ptr)
}

unsafe extern "C" {
    // rans_uncompress / fqz_decompress / rans_uncompress_4x16 / arith_uncompress_to /
    // tok3_decode_names are now served by the native `htscodecs` modules (see
    // cram_uncompress_block below) — the libhts externs were removed.
    #[link_name = "cram_free_compression_header"]
    fn htslib_cram_free_compression_header(hdr: *mut cram_block_compression_hdr);
    #[link_name = "stdin"]
    static mut HTSLIB_STDIN: *mut libc::FILE;
    #[link_name = "stdout"]
    static mut HTSLIB_STDOUT: *mut libc::FILE;
    #[link_name = "stderr"]
    static mut HTSLIB_STDERR: *mut libc::FILE;
}

pub const CRAM_STRING_ALLOC_MIN_STR_SIZE: usize = 1024;
const TRIAL_SPAN: c_int = 70;
const NTRIALS: c_int = 3;
const CRAM_FPAIRED: c_int = 256;
const CRAM_FPROPER_PAIR: c_int = 128;
const CRAM_FUNMAP: c_int = 64;
const CRAM_FREVERSE: c_int = 32;
const CRAM_FREAD1: c_int = 16;
const CRAM_FREAD2: c_int = 8;
const CRAM_FSECONDARY: c_int = 4;
const CRAM_FQCFAIL: c_int = 2;
const CRAM_FDUP: c_int = 1;
const CRAM_SUBST_MATRIX: &[u8; 20] = b"CGTNGTANCATNGCANACGT";

pub unsafe fn cram_read_block(fd: *mut cram_fd) -> *mut cram_block {
    cram_cram_io_c_1414_cram_read_block(fd.cast())
}

pub unsafe fn int32_put_blk(b: *mut cram_block, val: i32) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_1045_int32_put_blk(b, val)
}

pub unsafe fn int32_get_blk(b: *mut cram_block, val: *mut i32) -> c_int {
    crate::htslib_rs::cram::cram_cram_io_c_1029_int32_get_blk(b, val)
}

pub unsafe fn cram_block_size(b: *mut cram_block) -> u32 {
    cram_cram_io_c_1490_cram_block_size(b)
}

pub unsafe fn cram_write_block(fd: *mut cram_fd, b: *mut cram_block) -> c_int {
    cram_cram_io_c_1511_cram_write_block(fd.cast(), b)
}

pub unsafe fn cram_uncompress_block(b: *mut cram_block) -> c_int {
    cram_cram_io_c_1576_cram_uncompress_block(b)
}

pub unsafe fn cram_compress_block(
    fd: *mut cram_fd,
    b: *mut cram_block,
    metrics: *mut cram_metrics,
    method: c_int,
    level: c_int,
) -> c_int {
    cram_cram_io_c_2323_cram_compress_block(fd, b, metrics, method, level)
}

pub unsafe fn cram_set_header(fd: *mut cram_fd, hdr: *mut sam_hdr_t) -> c_int {
    cram_cram_io_c_2866_cram_set_header(fd.cast(), hdr.cast())
}

pub unsafe fn cram_new_container(nrec: c_int, nslice: c_int) -> *mut cram_container {
    cram_cram_io_c_3639_cram_new_container(nrec, nslice)
}

pub unsafe fn cram_free_container(c: *mut cram_container) {
    cram_cram_io_c_3705_cram_free_container(c)
}

pub unsafe fn cram_free_compression_header(hdr: *mut cram_block_compression_hdr) {
    cram_cram_io_c_4356_cram_free_compression_header(hdr)
}

pub unsafe fn cram_free_slice_header(hdr: *mut cram_block_slice_hdr) {
    unsafe { cram_cram_io_c_4409_cram_free_slice_header(hdr) }
}

pub unsafe fn cram_decode_compression_header(
    fd: *mut cram_fd,
    b: *mut cram_block,
) -> *mut cram_block_compression_hdr {
    cram_cram_decode_c_145_cram_decode_compression_header(fd, b)
}

/// khash STR (`KHASH_MAP_INIT_STR(map, pmap_t)`) hash function:
/// `__ac_FNV1a_hash_string` over a NUL-terminated C string.
pub unsafe fn kh_str_fnv1a_hash(mut s: *const c_char) -> u32 {
    let offset_basis: u32 = 2_166_136_261;
    let fnv_prime: u32 = 16_777_619;
    let mut h: u32 = offset_basis;
    while *s != 0 {
        h = (h ^ *s as u8 as u32).wrapping_mul(fnv_prime);
        s = s.add(1);
    }
    h
}

/// Legacy X31 hash helper used by non-CRAM khash translations that still map
/// to htslib tables built with `__ac_X31_hash_string`.
pub unsafe fn kh_str_x31_hash(mut s: *const c_char) -> u32 {
    let mut h: u32 = *s as u8 as u32;
    if h != 0 {
        s = s.add(1);
        while *s != 0 {
            h = (h << 5).wrapping_sub(h).wrapping_add(*s as u8 as u32);
            s = s.add(1);
        }
    }
    h
}

/// khash STR resize, faithful translation of `kh_resize_map` from
/// htslib/htslib/khash.h for the preservation-map (`const char*` keys,
/// 8-byte `pmap_t` values). Returns 0 on success, -1 on alloc failure.
pub unsafe fn kh_resize_map(h: *mut kh_generic_layout, mut new_n_buckets: u32) -> c_int {
    const HASH_UPPER: f64 = 0.77;
    let key_sz = std::mem::size_of::<*const c_char>() as u64;
    let val_sz = std::mem::size_of::<pmap_val>() as u64;

    // kroundup32
    new_n_buckets = new_n_buckets.wrapping_sub(1);
    new_n_buckets |= new_n_buckets >> 1;
    new_n_buckets |= new_n_buckets >> 2;
    new_n_buckets |= new_n_buckets >> 4;
    new_n_buckets |= new_n_buckets >> 8;
    new_n_buckets |= new_n_buckets >> 16;
    new_n_buckets = new_n_buckets.wrapping_add(1);
    if new_n_buckets < 4 {
        new_n_buckets = 4;
    }

    let mut new_flags: *mut u32 = std::ptr::null_mut();
    let mut j: u32 = 1;
    let fsize = |m: u32| -> u32 {
        if m < 16 {
            1
        } else {
            m >> 4
        }
    };
    if ((*h).size as f64) >= (new_n_buckets as f64 * HASH_UPPER + 0.5) {
        j = 0; // requested size too small
    } else {
        new_flags = malloc(fsize(new_n_buckets) as u64 * 4).cast::<u32>();
        if new_flags.is_null() {
            return -1;
        }
        for i in 0..fsize(new_n_buckets) {
            *new_flags.add(i as usize) = 0xaaaa_aaaa;
        }
        if (*h).n_buckets < new_n_buckets {
            let new_keys =
                realloc((*h).keys.cast(), new_n_buckets as u64 * key_sz).cast::<*const c_char>();
            if new_keys.is_null() {
                free(new_flags.cast());
                return -1;
            }
            (*h).keys = new_keys.cast();
            let new_vals =
                realloc((*h).vals.cast(), new_n_buckets as u64 * val_sz).cast::<pmap_val>();
            if new_vals.is_null() {
                free(new_flags.cast());
                return -1;
            }
            (*h).vals = new_vals.cast();
        }
    }

    if j != 0 {
        let keys = (*h).keys.cast::<*const c_char>();
        let vals = (*h).vals.cast::<pmap_val>();
        let old_n = (*h).n_buckets;
        let mut jj: u32 = 0;
        while jj != old_n {
            let flag = *(*h).flags.add((jj >> 4) as usize);
            if ((flag >> ((jj & 0xf) << 1)) & 3) == 0 {
                let mut key = *keys.add(jj as usize);
                let mut val = *vals.add(jj as usize);
                let new_mask = new_n_buckets - 1;
                // __ac_set_isdel_true on old flags[jj]
                *(*h).flags.add((jj >> 4) as usize) |= 1 << ((jj & 0xf) << 1);
                loop {
                    let k = kh_str_fnv1a_hash(key);
                    let mut i = k & new_mask;
                    let mut step: u32 = 0;
                    while ((*new_flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) == 0 {
                        step += 1;
                        i = (i + step) & new_mask;
                    }
                    // __ac_set_isempty_false on new_flags[i]
                    *new_flags.add((i >> 4) as usize) &= !(2 << ((i & 0xf) << 1));
                    if i < old_n
                        && ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 3) == 0
                    {
                        // kick out existing element
                        std::ptr::swap(&mut key, keys.add(i as usize));
                        std::ptr::swap(&mut val, vals.add(i as usize));
                        *(*h).flags.add((i >> 4) as usize) |= 1 << ((i & 0xf) << 1);
                    } else {
                        *keys.add(i as usize) = key;
                        *vals.add(i as usize) = val;
                        break;
                    }
                }
            }
            jj += 1;
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc((*h).keys.cast(), new_n_buckets as u64 * key_sz).cast();
            (*h).vals = realloc((*h).vals.cast(), new_n_buckets as u64 * val_sz).cast();
        }
        free((*h).flags.cast());
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound = ((*h).n_buckets as f64 * HASH_UPPER + 0.5) as u32;
    }
    0
}

/// khash STR `kh_put_map`. Inserts the (static) C-string `key` into the
/// preservation-map khash, returning the bucket index `x` and writing the
/// insertion status into `ret` (1 = new, 0 = present, 2 = replaced-deleted,
/// -1 = alloc failure). The key pointer is stored verbatim (khash STR does not
/// copy strings); the preservation-map keys are always static literals, so no
/// ownership transfer occurs and `cram_free_compression_header` only frees the
/// three khash arrays + struct. Faithful translation of `kh_put_map`.
pub unsafe fn kh_put_map(h: *mut kh_generic_layout, key: *const c_char, ret: *mut c_int) -> u32 {
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > ((*h).size << 1) {
            if kh_resize_map(h, (*h).n_buckets - 1) < 0 {
                *ret = -1;
                return (*h).n_buckets;
            }
        } else if kh_resize_map(h, (*h).n_buckets + 1) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }

    let keys = (*h).keys.cast::<*const c_char>();
    let mask = (*h).n_buckets - 1;
    let mut step: u32 = 0;
    let mut x = (*h).n_buckets;
    let mut site = (*h).n_buckets;
    let k = kh_str_fnv1a_hash(key);
    let mut i = k & mask;
    if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) != 0 {
        x = i; // empty -> fast path
    } else {
        let last = i;
        loop {
            let flag = *(*h).flags.add((i >> 4) as usize);
            let is_empty = (flag >> ((i & 0xf) << 1)) & 2;
            let is_del = (flag >> ((i & 0xf) << 1)) & 1;
            if is_empty != 0 || !(is_del != 0 || libc::strcmp(*keys.add(i as usize), key) != 0) {
                break;
            }
            if is_del != 0 {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                x = site;
                break;
            }
        }
        if x == (*h).n_buckets {
            if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = *(*h).flags.add((x >> 4) as usize);
    if ((flag >> ((x & 0xf) << 1)) & 2) != 0 {
        *keys.add(x as usize) = key;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        (*h).size += 1;
        (*h).n_occupied += 1;
        *ret = 1;
    } else if ((flag >> ((x & 0xf) << 1)) & 1) != 0 {
        *keys.add(x as usize) = key;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        (*h).size += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

pub unsafe fn cram_decode_slice_header(
    fd: *mut cram_fd,
    b: *mut cram_block,
) -> *mut cram_block_slice_hdr {
    unsafe { cram_cram_decode_c_955_cram_decode_slice_header(fd, b) }
}

pub unsafe fn cram_container_get_num_records(c: *mut cram_container) -> i32 {
    cram_cram_external_c_92_cram_container_get_num_records(c)
}

pub unsafe fn cram_container_get_num_bases(c: *mut cram_container) -> i64 {
    cram_cram_external_c_96_cram_container_get_num_bases(c)
}

pub unsafe fn cram_container_get_coords(
    c: *mut cram_container,
    refid: *mut c_int,
    start: *mut i64,
    span: *mut i64,
) {
    cram_cram_external_c_124_cram_container_get_coords(c, refid, start, span)
}

pub unsafe fn cram_read_container(fd: *mut cram_fd) -> *mut cram_container {
    cram_cram_io_c_3788_cram_read_container(fd.cast())
}

pub unsafe fn cram_container_size(c: *mut cram_container) -> c_int {
    cram_cram_io_c_3947_cram_container_size(c)
}

pub unsafe fn cram_store_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
    dat: *mut c_char,
    size: *mut c_int,
) -> c_int {
    cram_cram_io_c_3960_cram_store_container(fd.cast(), c.cast(), dat, size)
}

pub unsafe fn cram_write_container(fd: *mut cram_fd, h: *mut cram_container) -> c_int {
    cram_cram_io_c_4023_cram_write_container(fd.cast(), h.cast())
}

pub unsafe fn cram_cid2ds_free(cid2ds: *mut cram_cid2ds_t) {
    cram_cram_external_c_320_cram_cid2ds_free(cid2ds);
}

pub unsafe fn cram_update_cid2ds_map(
    hdr: *mut cram_block_compression_hdr,
    cid2ds: *mut cram_cid2ds_t,
) -> *mut cram_cid2ds_t {
    cram_cram_external_c_342_cram_update_cid2ds_map(hdr, cid2ds)
}

pub unsafe fn cram_cid2ds_query(
    c2d: *mut cram_cid2ds_t,
    content_id: c_int,
    n: *mut c_int,
) -> *mut c_int {
    cram_cram_external_c_443_cram_cid2ds_query(c2d, content_id, n)
}

pub unsafe fn cram_codec_get_content_ids(c: *mut cram_codec, ids: *mut c_int) {
    unsafe {
        *ids = cram_cram_codecs_c_3968_cram_codec_to_id(c.cast(), ids.add(1));
    }
}

pub unsafe fn cram_codec_describe(c: *mut cram_codec, ks: *mut kstring_t) -> c_int {
    unsafe { cram_cram_codecs_c_4185_cram_codec_describe(c.cast(), ks) }
}

pub unsafe fn cram_filter_container(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    c: *mut cram_container,
    ref_id: *mut c_int,
) -> c_int {
    unsafe { cram_cram_external_c_776_cram_filter_container(in_, out, c, ref_id) }
}

pub unsafe fn cram_open(filename: *const c_char, mode: *const c_char) -> *mut cram_fd {
    cram_cram_io_c_5264_cram_open(filename, mode)
}

pub unsafe fn cram_dopen(
    fp: *mut hFILE,
    filename: *const c_char,
    mode: *const c_char,
) -> *mut cram_fd {
    cram_cram_io_c_5289_cram_dopen(fp, filename, mode)
}

pub unsafe fn cram_seek(fd: *mut cram_fd, offset: libc::off_t, whence: c_int) -> c_int {
    cram_cram_io_c_5431_cram_seek(fd, offset, whence)
}

pub unsafe fn cram_flush(fd: *mut cram_fd) -> c_int {
    cram_cram_io_c_5446_cram_flush(fd)
}

pub unsafe fn cram_close(fd: *mut cram_fd) -> c_int {
    cram_cram_io_c_5558_cram_close(fd)
}

// Public shims bridging production's hts_sys-aliased cram types to the
// native flush helpers in `cram_flush_bridge`. The pointer casts are pure
// type-system noise: production's `cram_fd` / `cram_container` are aliases
// onto `hts_sys::*`, which are #[repr(C)] and byte-identical to the
// concrete struct types the mirror exposes.
pub unsafe fn cram_flush_container(fd: *mut cram_fd, c: *mut cram_container) -> c_int {
    cram_cram_io_c_4143_cram_flush_container(fd, c)
}

pub unsafe fn cram_flush_container_mt(fd: *mut cram_fd, c: *mut cram_container) -> c_int {
    cram_cram_io_c_4275_cram_flush_container_mt(fd, c)
}

pub unsafe fn cram_write_eof_block(fd: *mut cram_fd) -> c_int {
    cram_cram_io_c_5474_cram_write_eof_block(fd)
}

pub unsafe fn cram_eof(fd: *mut cram_fd) -> c_int {
    cram_cram_io_c_5662_cram_eof(fd.cast())
}

// Mirrors the tail of C `cram_set_voption(CRAM_OPT_RANGE, ...)`: after
// cram_seek_to_refpos has updated fd->range, OR SAM_POS into required_fields
// unless the special "refid == -2" sentinel is set (set by HTS_IDX_START/REST).
// Used by sam_cram_itr_query to complete the native CRAM iterator setup.
pub unsafe fn cram_set_required_fields_pos(fd: *mut cram_fd) {
    if fd.is_null() {
        return;
    }
    let fdl = fd.cast::<cram_fd_layout>();
    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).range_lock);
    if (*fdl).range.refid != -2 {
        (*fdl).required_fields |= crate::htslib_rs::cram::SAM_POS;
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).range_lock);
}

pub unsafe fn cram_set_voption(
    fd: *mut cram_fd,
    opt: hts_fmt_option,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> c_int {
    unsafe { cram_cram_io_c_5692_cram_set_voption(fd, opt, args) }
}

unsafe fn cram_voption_va_arg_word(args: *mut crate::htslib_rs::c_compat::__va_list_tag) -> usize {
    unsafe {
        if args.is_null() {
            return 0;
        }
        if (*args).gp_offset <= 40 {
            let p = (*args)
                .reg_save_area
                .cast::<u8>()
                .add((*args).gp_offset as usize);
            (*args).gp_offset += 8;
            p.cast::<usize>().read_unaligned()
        } else {
            let p = (*args).overflow_arg_area.cast::<u8>();
            (*args).overflow_arg_area = p.add(8).cast();
            p.cast::<usize>().read_unaligned()
        }
    }
}

unsafe fn cram_voption_va_arg_int(args: *mut crate::htslib_rs::c_compat::__va_list_tag) -> c_int {
    unsafe { cram_voption_va_arg_word(args) as c_int }
}

unsafe fn cram_voption_va_arg_ptr<T>(
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut T {
    unsafe { cram_voption_va_arg_word(args) as *mut T }
}

unsafe fn cram_voption_set_version(fd: *mut cram_fd, s: *const c_char) -> c_int {
    unsafe {
        if s.is_null() {
            *__errno_location() = EINVAL;
            return -1;
        }

        let Ok(ver) = CStr::from_ptr(s).to_str() else {
            *__errno_location() = EINVAL;
            return -1;
        };
        let Some((major_s, minor_s)) = ver.split_once('.') else {
            *__errno_location() = EINVAL;
            return -1;
        };
        let Ok(major) = major_s.parse::<c_int>() else {
            *__errno_location() = EINVAL;
            return -1;
        };
        let Ok(minor) = minor_s.parse::<c_int>() else {
            *__errno_location() = EINVAL;
            return -1;
        };

        let valid = (major == 1 && minor == 0)
            || (major == 2 && (minor == 0 || minor == 1))
            || (major == 3 && (minor == 0 || minor == 1))
            || (major == 4 && minor == 0);
        if !valid {
            *__errno_location() = EINVAL;
            return -1;
        }

        let fd = fd.cast::<cram_fd_layout>();
        (*fd).version = major * 256 + minor;
        (*fd).use_rans = if major >= 3 { 1 } else { 0 };
        (*fd).use_tok = if (major == 3 && minor >= 1) || major >= 4 {
            1
        } else {
            0
        };
        cram_cram_io_c_5170_cram_init_tables(fd.cast());
        0
    }
}

unsafe fn cram_voption_set_range_noseek(fd: *mut cram_fd, r: *const cram_range_layout) -> c_int {
    unsafe {
        if r.is_null() {
            *__errno_location() = EINVAL;
            return -1;
        }

        let fd = fd.cast::<cram_fd_layout>();
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fd).range_lock);
        (*fd).range = *r;
        if (*r).refid == HTS_IDX_NOCOOR {
            (*fd).range.refid = -1;
            (*fd).range.start = 0;
        } else if (*r).refid == HTS_IDX_START || (*r).refid == HTS_IDX_REST {
            (*fd).range.refid = -2;
        }
        if (*fd).range.refid != -2 {
            (*fd).required_fields |= crate::htslib_rs::cram::SAM_POS;
        }
        (*fd).ooc = 0;
        (*fd).eof = 0;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fd).range_lock);
        0
    }
}

pub unsafe fn cram_check_EOF(fd: *mut cram_fd) -> c_int {
    cram_cram_io_c_5960_cram_check_eof(fd)
}

pub unsafe fn cram_copy_slice(in_: *mut cram_fd, out: *mut cram_fd, num_slice: i32) -> c_int {
    cram_cram_external_c_683_cram_copy_slice(in_, out, num_slice)
}

pub unsafe fn cram_transcode_rg(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    c: *mut cram_container,
    nrg: c_int,
    in_rg: *mut c_int,
    out_rg: *mut c_int,
) -> c_int {
    unsafe { cram_cram_external_c_934_cram_transcode_rg(in_, out, c, nrg, in_rg, out_rg) }
}

pub unsafe fn cram_get_refs(fd: *mut htsFile) -> *mut refs_t {
    cram_cram_external_c_1029_cram_get_refs(fd)
}

#[repr(C)]
pub struct cram_string_alloc_string_t {
    pub str_: *mut c_char,
    pub used: usize,
}

#[repr(C)]
pub struct cram_string_alloc_t {
    pub max_length: usize,
    pub nstrings: usize,
    pub max_strings: usize,
    pub strings: *mut cram_string_alloc_string_t,
}

#[repr(C)]
pub struct mFILE {
    pub fp: *mut libc::FILE,
    pub data: *mut c_char,
    pub alloced: usize,
    pub eof: c_int,
    pub mode: c_int,
    pub size: usize,
    pub offset: usize,
    pub flush_pos: usize,
}

pub const MF_READ: c_int = 1;
pub const MF_WRITE: c_int = 2;
pub const MF_APPEND: c_int = 4;
pub const MF_BINARY: c_int = 8;
pub const MF_TRUNC: c_int = 16;
pub const MF_MODEX: c_int = 32;
pub const MF_MMAP: c_int = 64;
pub const POOLED_ALLOC_PSIZE: usize = 1024 * 1024;

#[repr(C)]
pub struct pool_t {
    pub pool: *mut c_void,
    pub used: usize,
}

#[repr(C)]
pub struct pool_alloc_t {
    pub dsize: usize,
    pub psize: usize,
    pub npools: usize,
    pub pools: *mut pool_t,
    pub free: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_block_layout {
    method: c_int,
    orig_method: c_int,
    content_type: cram_content_type,
    content_id: i32,
    comp_size: i32,
    uncomp_size: i32,
    crc32: u32,
    idx: i32,
    data: *mut u8,
    alloc: usize,
    byte: usize,
    bit: c_int,
    m: *mut cram_metrics_layout,
    crc32_checked: c_int,
    crc_part: u32,
}

// original: cram_container (htslib/cram/cram_structs.h:422)
// Full field set, byte-identical to C `cram_container` (cross-checked against
// src/cram/cram_structs.rs:160). Previously only the on-disk header prefix was
// modelled; the complete layout is required so cram_new_container's
// `calloc(1, sizeof(*c))` allocates the right size and the write-path fields
// (stats, tags_used, slices, ...) live at the C-correct offsets.
#[repr(C)]
pub(crate) struct cram_container_layout {
    pub(crate) length: i32,
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    record_counter: i64,
    num_bases: i64,
    num_records: i32,
    num_blocks: i32,
    num_landmarks: i32,
    landmark: *mut i32,
    pub(crate) offset: usize,
    comp_hdr: *mut cram_block_compression_hdr_layout,
    comp_hdr_block: *mut cram_block_layout,
    pub(crate) max_slice: c_int,
    pub(crate) curr_slice: c_int,
    curr_slice_mt: c_int,
    max_rec: c_int,
    curr_rec: c_int,
    max_c_rec: c_int,
    curr_c_rec: c_int,
    slice_rec: c_int,
    curr_ref: c_int,
    last_pos: i64,
    slices: *mut *mut cram_slice_layout,
    pub(crate) slice: *mut cram_slice_layout,
    pos_sorted: c_int,
    max_apos: i64,
    last_slice: c_int,
    multi_seq: c_int,
    unsorted: c_int,
    qs_seq_orient: c_int,
    ref_id: c_int,
    ref_start: i64,
    first_base: i64,
    last_base: i64,
    ref_end: i64,
    ref_: *mut c_char,
    embed_ref: c_int,
    no_ref: c_int,
    bams: *mut *mut bam1_t,
    stats: [*mut cram_stats_layout; CRAM_DS_END],
    tags_used: *mut kh_generic_layout,
    refs_used: *mut c_int,
    crc32: u32,
    s_num_bases: u64,
    s_aux_bytes: u64,
    n_mapped: u32,
    ref_free: c_int,
}

#[repr(C)]
struct cram_block_slice_hdr_layout {
    content_type: cram_content_type,
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    num_records: i32,
    record_counter: i64,
    num_blocks: i32,
    num_content_ids: i32,
    block_content_ids: *mut i32,
    ref_base_id: i32,
    md5: [u8; 16],
}

// Native cram_feature subtype layouts (htslib/cram/cram_structs.h:541).
// The on-disk container/slice primitives do not interpret features; cram_slice
// only stores a `*mut cram_feature_layout`. The union is the widest member.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union cram_feature_layout {
    fields: [c_int; 4],
}

// original: cram_record (htslib/cram/cram_structs.h:486)
// Byte-identical to C `cram_record` (and to src/cram/cram_structs.rs:213).
// `s` is the back-pointer to the owning slice; kept as a raw cram_slice pointer
// (hts_sys type in signature space; layout-identical to C `struct cram_slice *`).
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct cram_record_layout {
    s: *mut cram_slice_layout,
    ref_id: i32,
    flags: i32,
    cram_flags: i32,
    len: i32,
    apos: i64,
    rg: i32,
    name: i32,
    name_len: i32,
    mate_line: i32,
    mate_ref_id: i32,
    mate_pos: i64,
    tlen: i64,
    explicit_tlen: i64,
    ntags: i32,
    aux: u32,
    aux_size: u32,
    // TN_external is NOT defined in the htslib build, so the TN_idx variant is used.
    tn_idx: i32,
    tl: c_int,
    seq: u32,
    qual: u32,
    cigar: u32,
    ncigar: i32,
    aend: i64,
    mqual: i32,
    feature: u32,
    nfeature: u32,
    mate_flags: i32,
}

// original: cram_slice (htslib/cram/cram_structs.h:608)
// Full field set, byte-identical to C `cram_slice` (cross-checked against
// src/cram/cram_structs.rs:321). Contained-struct pointers stay typed as the
// hts_sys / native layout types they reference, matching the existing
// layout-struct convention; the byte layout equals the C struct.
#[repr(C)]
pub(crate) struct cram_slice_layout {
    hdr: *mut cram_block_slice_hdr_layout,
    hdr_block: *mut cram_block_layout,
    block: *mut *mut cram_block_layout,
    block_by_id: *mut *mut cram_block_layout,
    last_apos: i64,
    max_apos: i64,
    crecs: *mut cram_record_layout,
    cigar: *mut u32,
    cigar_alloc: u32,
    ncigar: u32,
    features: *mut cram_feature_layout,
    nfeatures: u32,
    afeatures: u32,
    // TN_external is NOT defined: TN[]/nTN/aTN variant is in effect.
    tn: *mut u32,
    n_tn: c_int,
    a_tn: c_int,
    name_blk: *mut cram_block_layout,
    seqs_blk: *mut cram_block_layout,
    qual_blk: *mut cram_block_layout,
    base_blk: *mut cram_block_layout,
    soft_blk: *mut cram_block_layout,
    aux_blk: *mut cram_block_layout,
    pair_keys: *mut cram_string_alloc_t,
    pair: [*mut kh_generic_layout; 2],
    ref_: *mut c_char,
    ref_start: i64,
    ref_end: i64,
    ref_id: c_int,
    naux_block: c_int,
    aux_block: *mut *mut cram_block_layout,
    data_series: c_uint,
    decode_md: c_int,
    pub(crate) max_rec: c_int,
    pub(crate) curr_rec: c_int,
    slice_num: c_int,
}

const CRAM_DS_END: usize = 47;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_range_layout {
    pub refid: c_int,
    pub start: i64,
    pub end: i64,
}

#[repr(C)]
pub struct cram_file_def_layout {
    magic: [c_char; 4],
    major_version: u8,
    minor_version: u8,
    file_id: [c_char; 20],
}

#[repr(C)]
pub(crate) struct cram_fd_layout {
    fp: *mut hFILE,
    mode: c_int,
    version: c_int,
    file_def: *mut c_void,
    header: *mut crate::htslib_rs::sam::sam_hdr_t,
    prefix: *mut c_char,
    record_counter: i64,
    err: c_int,
    pub(crate) ctr: *mut cram_container_layout,
    pub(crate) ctr_mt: *mut cram_container_layout,
    first_base: c_int,
    last_base: c_int,
    refs: *mut refs_t_layout,
    ref_: *mut c_char,
    ref_free: *mut c_char,
    ref_id: c_int,
    ref_start: i64,
    ref_end: i64,
    ref_fn: *mut c_char,
    level: c_int,
    m: [*mut cram_metrics_layout; CRAM_DS_END],
    tags_used: *mut c_void,
    decode_md: c_int,
    seqs_per_slice: c_int,
    bases_per_slice: c_int,
    slices_per_container: c_int,
    embed_ref: c_int,
    no_ref: c_int,
    no_ref_counter: c_int,
    ignore_md5: c_int,
    use_bz2: c_int,
    use_rans: c_int,
    use_lzma: c_int,
    use_fqz: c_int,
    use_tok: c_int,
    use_arith: c_int,
    shared_ref: c_int,
    required_fields: c_uint,
    store_md: c_int,
    store_nm: c_int,
    range: cram_range_layout,
    bam_flag_swap: [c_uint; 0x1000],
    cram_flag_swap: [c_uint; 0x1000],
    l1: [u8; 256],
    l2: [u8; 256],
    cram_sub_matrix: [[c_char; 32]; 32],
    index_sz: c_int,
    index: *mut c_void,
    pub(crate) first_container: libc::off_t,
    pub(crate) curr_position: libc::off_t,
    eof: c_int,
    last_slice: c_int,
    last_ri_count: c_int,
    multi_seq: c_int,
    multi_seq_user: c_int,
    unsorted: c_int,
    last_mapped: c_int,
    empty_container: c_int,
    own_pool: c_int,
    pool: *mut c_void,
    rqueue: *mut c_void,
    metrics_lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    ref_lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    range_lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    bl: *mut c_void,
    bam_list_lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    job_pending: *mut c_void,
    pub(crate) ooc: c_int,
    lossy_read_names: c_int,
    tlen_approx: c_int,
    tlen_zero: c_int,
    idxfp: *mut BGZF,
    vv: varint_vec_layout,
    ap_delta: c_int,
}

#[repr(C)]
struct hfile_layout {
    buffer: *mut c_char,
    begin: *mut c_char,
    end: *mut c_char,
    limit: *mut c_char,
    backend: *const c_void,
    offset: libc::off_t,
    flags: c_uint,
    has_errno: c_int,
}

const HFILE_MOBILE: c_uint = 1 << 1;
const CRAM_DEFAULT_LEVEL: c_int = 5;
const CRAM_DEFAULT_SEQS_PER_SLICE: c_int = 10000;
const CRAM_DEFAULT_BASES_PER_SLICE: c_int = CRAM_DEFAULT_SEQS_PER_SLICE * 500;

#[repr(C)]
struct cram_stats_layout {
    freqs: [c_int; 1024],
    h: *mut c_void,
    nsamp: c_int,
    nvals: c_int,
    min_val: i64,
    max_val: i64,
}

#[repr(C)]
struct cram_metrics_layout {
    trial: c_int,
    next_trial: c_int,
    consistency: c_int,
    sz: [c_int; 32],
    input_avg_sz: c_int,
    input_avg_delta: c_int,
    method: c_int,
    revised_method: c_int,
    strat: c_int,
    cnt: [c_int; 32],
    extra: [f64; 32],
    unpackable: c_int,
}

#[repr(C)]
struct ref_entry_layout {
    name: *mut c_char,
    fn_: *mut c_char,
    length: i64,
    ln_length: i64,
    offset: i64,
    bases_per_line: c_int,
    line_length: c_int,
    count: i64,
    seq: *mut c_char,
    mf: *mut mFILE,
    is_md5: c_int,
    validated_md5: c_int,
}

#[repr(C)]
struct kh_refs_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut *const c_char,
    vals: *mut *mut ref_entry_layout,
}

#[repr(C)]
pub struct kh_generic_layout {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut c_void,
    pub vals: *mut c_void,
}

/// `pmap_t` (htslib/cram/cram_structs.h:75): the preservation-map khash value,
/// a union of an `int` flag and a `char *` pointer into the comp-hdr block.
#[repr(C)]
#[derive(Clone, Copy)]
union pmap_val {
    i: c_int,
    p: *mut c_char,
}

#[repr(C)]
struct refs_t_layout {
    pool: *mut cram_string_alloc_t,
    h_meta: *mut kh_refs_layout,
    ref_id: *mut *mut ref_entry_layout,
    nref: c_int,
    fn_: *mut c_char,
    fp: *mut BGZF,
    count: c_int,
    lock: crate::htslib_rs::c_compat::pthread_mutex_t,
    last: *mut ref_entry_layout,
    last_id: c_int,
}

#[repr(C)]
struct sam_hrec_tag_layout {
    next: *mut sam_hrec_tag_layout,
    str_: *const c_char,
    len: c_int,
}

type CramCodecFreeFn = unsafe extern "C" fn(*mut cram_codec_base_layout);

#[repr(C)]
struct cram_codec_base_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: Option<CramCodecFreeFn>,
}

#[repr(C)]
struct kh_m_i2i_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut i64,
    vals: *mut c_int,
}

#[repr(C)]
pub(crate) struct cram_block_compression_hdr_layout {
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    num_records: i32,
    num_landmarks: i32,
    landmark: *mut i32,
    read_names_included: i32,
    ap_delta: i32,
    substitution_matrix: [[c_char; 4]; 5],
    no_ref: i32,
    qs_seq_orient: i32,
    td_blk: *mut cram_block_layout,
    ntl: i32,
    tl: *mut *mut u8,
    td_hash: *mut c_void,
    td_keys: *mut c_void,
    preservation_map: *mut c_void,
    rec_encoding_map: [*mut c_void; 32],
    tag_encoding_map: [*mut c_void; 32],
    codecs: [*mut c_void; 47],
    uncomp: *mut c_char,
    uncomp_size: usize,
    uncomp_alloc: usize,
    ncodecs: i32,
}

#[repr(C)]
struct cram_map_layout {
    key: c_int,
    encoding: c_int,
    offset: c_int,
    size: c_int,
    codec: *mut c_void,
    next: *mut cram_map_layout,
}

#[repr(C)]
struct cram_codec_iter_layout {
    hdr: *mut cram_block_compression_hdr_layout,
    curr_map: *mut cram_map_layout,
    idx: c_int,
    is_tag: c_int,
}

type VarintGet32Fn = unsafe extern "C" fn(*mut *mut c_char, *const c_char, *mut c_int) -> i64;
type VarintGet64Fn = unsafe extern "C" fn(*mut *mut c_char, *const c_char, *mut c_int) -> i64;
type VarintPut32Fn = unsafe extern "C" fn(*mut c_char, *mut c_char, i32) -> c_int;
type VarintPut64Fn = unsafe extern "C" fn(*mut c_char, *mut c_char, i64) -> c_int;
type VarintPut32BlkFn = unsafe extern "C" fn(*mut cram_block, i32) -> c_int;
type VarintPut64BlkFn = unsafe extern "C" fn(*mut cram_block, i64) -> c_int;
type VarintSizeFn = unsafe extern "C" fn(i64) -> c_int;
type CramCodecDecodeFn = unsafe extern "C" fn(
    *mut cram_slice,
    *mut c_void,
    *mut cram_block,
    *mut c_char,
    *mut c_int,
) -> c_int;
type CramCodecEncodeFn =
    unsafe extern "C" fn(*mut cram_slice, *mut c_void, *mut c_char, c_int) -> c_int;
type CramCodecFlushFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type CramCodecStoreFn =
    unsafe extern "C" fn(*mut c_void, *mut cram_block, *mut c_char, c_int) -> c_int;
type CramCodecDescribeFn = unsafe extern "C" fn(*mut c_void, *mut kstring_t) -> c_int;
type CramCodecSizeFn = unsafe extern "C" fn(*mut cram_slice, *mut c_void) -> c_int;
type CramCodecGetBlockFn = unsafe extern "C" fn(*mut cram_slice, *mut c_void) -> *mut cram_block;
type CramCodecDecodeInitFn =
    unsafe fn(*mut c_void, *mut c_char, c_int, c_int, c_int, c_int, *mut c_void) -> *mut c_void;
type CramCodecEncodeInitFn =
    unsafe fn(*mut c_void, c_int, c_int, *mut c_void, c_int, *mut c_void) -> *mut c_void;

#[repr(C)]
struct varint_vec_layout {
    varint_decode32_crc: *mut c_void,
    varint_decode32s_crc: *mut c_void,
    varint_decode64_crc: *mut c_void,
    varint_get32: Option<VarintGet32Fn>,
    varint_get32s: Option<VarintGet32Fn>,
    varint_get64: Option<VarintGet64Fn>,
    varint_get64s: Option<VarintGet64Fn>,
    varint_put32: Option<VarintPut32Fn>,
    varint_put32s: Option<VarintPut32Fn>,
    varint_put64: Option<VarintPut64Fn>,
    varint_put64s: Option<VarintPut64Fn>,
    varint_put32_blk: Option<VarintPut32BlkFn>,
    varint_put32s_blk: Option<VarintPut32BlkFn>,
    varint_put64_blk: Option<VarintPut64BlkFn>,
    varint_put64s_blk: Option<VarintPut64BlkFn>,
    varint_size: Option<VarintSizeFn>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_external_decoder_layout {
    content_id: i32,
    type_: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_varint_decoder_layout {
    content_id: i32,
    offset: i64,
    type_: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_codec_external_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    external: cram_external_decoder_layout,
}

#[repr(C)]
struct cram_codec_varint_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    varint: cram_varint_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_const_codec_layout {
    val: i64,
}

#[repr(C)]
struct cram_codec_const_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    xconst: cram_const_codec_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_beta_decoder_layout {
    offset: i32,
    nbits: i32,
}

#[repr(C)]
struct cram_codec_beta_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    beta: cram_beta_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_xpack_decoder_layout {
    nbits: i32,
    sub_encoding: c_int,
    sub_codec_dat: *mut c_void,
    sub_codec: *mut c_void,
    nval: c_int,
    rmap: [u32; 256],
    map: [c_int; 256],
}

#[repr(C)]
struct cram_codec_xpack_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    xpack: cram_xpack_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_xdelta_decoder_layout {
    last: i64,
    word_size: u8,
    sub_encoding: c_int,
    sub_codec_dat: *mut c_void,
    sub_codec: *mut c_void,
}

#[repr(C)]
struct cram_codec_xdelta_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    xdelta: cram_xdelta_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_xrle_decoder_layout {
    len_encoding: c_int,
    lit_encoding: c_int,
    len_dat: *mut c_void,
    lit_dat: *mut c_void,
    len_codec: *mut c_void,
    lit_codec: *mut c_void,
    cur_len: c_int,
    cur_lit: c_int,
    rep_score: [c_int; 256],
    to_flush: *mut c_char,
    to_flush_size: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_codec_xrle_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    xrle: cram_xrle_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_subexp_decoder_layout {
    offset: i32,
    k: i32,
}

#[repr(C)]
struct cram_codec_subexp_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    subexp: cram_subexp_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_gamma_decoder_layout {
    offset: i32,
}

#[repr(C)]
struct cram_codec_gamma_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    gamma: cram_gamma_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_huffman_code_layout {
    symbol: i64,
    p: i32,
    code: i32,
    len: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_huffman_decoder_layout {
    ncodes: c_int,
    codes: *mut cram_huffman_code_layout,
    option: c_int,
}

#[repr(C)]
struct cram_codec_huffman_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    huffman: cram_huffman_decoder_layout,
}

#[repr(C)]
struct cram_huffman_encoder_layout {
    codes: *mut cram_huffman_code_layout,
    nvals: c_int,
    val2code: [c_int; 129],
    option: c_int,
}

#[repr(C)]
struct cram_codec_huffman_encoder_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    huffman: cram_huffman_encoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_byte_array_len_decoder_layout {
    len_codec: *mut c_void,
    val_codec: *mut c_void,
}

#[repr(C)]
struct cram_byte_array_len_encoder_dat_layout {
    len_encoding: c_int,
    val_encoding: c_int,
    len_dat: *mut c_void,
    val_dat: *mut c_void,
    len_codec: *mut c_void,
    val_codec: *mut c_void,
}

#[repr(C)]
struct cram_codec_byte_array_len_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    byte_array_len: cram_byte_array_len_decoder_layout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_byte_array_stop_decoder_layout {
    stop: u8,
    content_id: i32,
}

#[repr(C)]
struct cram_codec_byte_array_stop_layout {
    codec: c_int,
    out: *mut cram_block_layout,
    vv: *mut varint_vec_layout,
    codec_id: c_int,
    free: *mut c_void,
    decode: *mut c_void,
    encode: *mut c_void,
    store: *mut c_void,
    size: *mut c_void,
    flush: *mut c_void,
    get_block: *mut c_void,
    describe: *mut c_void,
    byte_array_stop: cram_byte_array_stop_decoder_layout,
}

// =======================================================================
// Helpers for the cram_encode_slice family (lifted from cram-mirror tree).
// =======================================================================

/// cram_encoding enum values (htslib/cram/cram_structs.h).
const E_EXTERNAL_ENC: c_uint = 1;
const E_GOLOMB_ENC: c_uint = 2;
const E_HUFFMAN_ENC: c_uint = 3;
const E_BYTE_ARRAY_LEN_ENC: c_uint = 4;
const E_BYTE_ARRAY_STOP_ENC: c_uint = 5;
const E_BETA_ENC: c_uint = 6;
const E_SUBEXP_ENC: c_uint = 7;
const E_GOLOMB_RICE_ENC: c_uint = 8;
const E_GAMMA_ENC: c_uint = 9;
const E_VARINT_UNSIGNED_ENC: c_uint = 41;
const E_VARINT_SIGNED_ENC: c_uint = 42;
const E_CONST_BYTE_ENC: c_uint = 43;
const E_CONST_INT_ENC: c_uint = 44;
const E_XPACK_ENC: c_uint = 51;
const E_XRLE_ENC: c_uint = 52;
const E_XDELTA_ENC: c_uint = 53;

// cram_encode-related DS constants not yet defined above. We reuse the
// production DS_* set declared near the top (DS_AUX, DS_RN, DS_QS, DS_IN,
// DS_SC, DS_TN) and add the rest needed by the slice-encode family.
const DS_ENC_END: c_int = 47;
const DS_ENC_CORE: c_int = 0;
const DS_ENC_aux: c_int = 1;
const DS_ENC_aux_oz: c_int = 9;
const DS_ENC_ref: c_int = 10;
const DS_ENC_BF: c_int = 15;
const DS_ENC_CF: c_int = 16;
const DS_ENC_AP: c_int = 17;
const DS_ENC_RG: c_int = 18;
const DS_ENC_MQ: c_int = 19;
const DS_ENC_NS: c_int = 20;
const DS_ENC_MF: c_int = 21;
const DS_ENC_TS: c_int = 22;
const DS_ENC_NP: c_int = 23;
const DS_ENC_NF: c_int = 24;
const DS_ENC_RL: c_int = 25;
const DS_ENC_FN: c_int = 26;
const DS_ENC_FC: c_int = 27;
const DS_ENC_FP: c_int = 28;
const DS_ENC_DL: c_int = 29;
const DS_ENC_BA: c_int = 30;
const DS_ENC_BS: c_int = 31;
const DS_ENC_TL: c_int = 32;
const DS_ENC_RI: c_int = 33;
const DS_ENC_RS: c_int = 34;
const DS_ENC_PD: c_int = 35;
const DS_ENC_HC: c_int = 36;
const DS_ENC_BB: c_int = 37;
const DS_ENC_TC: c_int = 44;

// CRAM record cram_flags bits (cram_structs.h:71).
const CRAM_FLAG_DETACHED_ENC: c_int = 1 << 1;
const CRAM_FLAG_MATE_DOWNSTREAM_ENC: c_int = 1 << 2;
const CRAM_FLAG_EXPLICIT_TLEN_ENC: c_int = 1 << 4;
const CRAM_FLAG_MASK_ENC: c_int = (1 << 5) - 1;
// `(1U << 31)` cast through the unsigned-then-int-bitcast (htslib defines it
// as `1U<<31`). Stored in the signed `cram_flags: i32`.
const CRAM_FLAG_DISCARD_NAME_ENC: i32 = i32::MIN;

// BAM_FUNMAP (htslib/sam.h:1184) — duplicated locally to avoid the sam.rs
// re-export drift; matches the import already used at the top of cram.rs.
const BAM_FUNMAP_ENC: c_int = 4;

/// Feature variant layouts used by `cram_encode_slice_read`. All variants
/// alias the same union of 4 ints with named fields; cast `*mut cram_feature_layout`
/// to the correct view.
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_X_layout {
    pos: c_int,
    code: c_int,
    base: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_i_layout {
    pos: c_int,
    code: c_int,
    base: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_B_layout {
    pos: c_int,
    code: c_int,
    base: c_int,
    qual: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_D_layout {
    pos: c_int,
    code: c_int,
    len: c_int,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_b_layout {
    pos: c_int,
    code: c_int,
    seq_idx: c_int,
    len: c_int,
}
// Used by S (soft-clip) and I (insertion) variants; shape (pos, code, len, seq_idx).
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_S_layout {
    pos: c_int,
    code: c_int,
    len: c_int,
    seq_idx: c_int,
}
// Used by Q (quality) variant; shape (pos, code, qual).
#[repr(C)]
#[derive(Clone, Copy)]
struct cram_feature_Q_layout {
    pos: c_int,
    code: c_int,
    qual: c_int,
}

/// Invoke a codec's `encode` function pointer. The codec base is opaque to
/// production; cast to `cram_codec_external_layout` (variants share the
/// 12-field prefix through `describe`) and transmute the void pointer.
#[inline]
unsafe fn cram_codec_encode(
    codec: *mut c_void,
    s: *mut cram_slice,
    inp: *mut c_char,
    in_size: c_int,
) -> c_int {
    let cv = codec.cast::<cram_codec_external_layout>();
    let encode: CramCodecEncodeFn = cram_fn((*cv).encode);
    encode(s, codec, inp, in_size)
}

/// Invoke a codec's `flush` function pointer.
#[inline]
unsafe fn cram_codec_flush(codec: *mut c_void) -> c_int {
    let cv = codec.cast::<cram_codec_external_layout>();
    let flush: CramCodecFlushFn = cram_fn((*cv).flush);
    flush(codec)
}

// ============================================================================
// Reference-building helpers (used by cram_encode_container).
// `extend_ref`/`cram_add_to_ref_MD`/`cram_add_to_ref`/`cram_generate_reference`
// together synthesise a CRAM-embeddable reference from BAM records when no
// external reference is supplied (`embed_ref=2`). Faithful translations of
// htslib/cram/cram_encode.c:1508/1557/1663/1737.
// ============================================================================

// DS_ index constants shared by the cram_add_* feature-builder helpers below
// (mirror of the local constants used inside cram_encode_compression_header).
// Values come from htslib/cram/cram_structs.h cram_DS_ID enum.
const DS_FC_ENC: c_int = 27;
const DS_FP_ENC: c_int = 28;
const DS_DL_ENC: c_int = 29;
const DS_BA_ENC: c_int = 30;
const DS_BS_ENC: c_int = 31;
const DS_RS_ENC: c_int = 34;
const DS_PD_ENC: c_int = 35;
const DS_HC_ENC: c_int = 36;

// cram_tag_map: cram/cram_structs.h:386. 4 ptrs: codec, blk, blk2, m.
#[repr(C)]
pub struct cram_tag_map_layout {
    codec: *mut cram_codec,
    blk: *mut cram_block,
    blk2: *mut cram_block,
    m: *mut cram_metrics_layout,
}

// kh_m_tagmap: KHASH_MAP_INIT_INT(m_tagmap, cram_tag_map*) — int->cram_tag_map*.
// Layout-identical to khash_m_tagmap_t from htslib/cram/cram_structs.h.
#[repr(C)]
struct kh_m_tagmap_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut u32,
    vals: *mut *mut cram_tag_map_layout,
}

// kh_m_metrics: KHASH_MAP_INIT_INT(m_metrics, cram_metrics*) — int->cram_metrics*.
#[repr(C)]
struct kh_m_metrics_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut u32,
    vals: *mut *mut cram_metrics_layout,
}

// kh_m_s2i: KHASH_MAP_INIT_STR(m_s2i, int) — char*->int (FNV1a-hashed).
// Layout matches sam.rs::khash_m_s2i_t.
#[repr(C)]
struct kh_m_s2i_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut *const c_char,
    vals: *mut c_int,
}

// __ac_X31_hash_string: htslib/khash.h. Used by the legacy KHASH_MAP_INIT_STR
// macro before FNV1a became the default — not actually used here (m_s2i uses
// FNV1a per khash.h:480), kept only as a comment reference for completeness.

// Integer-keyed khash hash function: identity (uint32_t)key, per
// KHASH_MAP_INIT_INT.

#[inline]
fn kh_int_hash(key: u32) -> u32 {
    key
}

// kh_resize for an INT-keyed khash with `*mut T` values. Identical structure
// to the kh_m_s2u64 resize but with u32 keys and pointer-sized vals.
#[allow(clippy::too_many_arguments)]
unsafe fn kh_resize_int_ptr<T>(
    n_buckets_field: *mut u32,
    size_field: *mut u32,
    n_occupied_field: *mut u32,
    upper_bound_field: *mut u32,
    flags_field: *mut *mut u32,
    keys_field: *mut *mut u32,
    vals_field: *mut *mut *mut T,
    mut new_n_buckets: u32,
) -> c_int {
    const HASH_UPPER: f64 = 0.77;
    let key_sz = std::mem::size_of::<u32>() as u64;
    let val_sz = std::mem::size_of::<*mut T>() as u64;

    new_n_buckets = new_n_buckets.wrapping_sub(1);
    new_n_buckets |= new_n_buckets >> 1;
    new_n_buckets |= new_n_buckets >> 2;
    new_n_buckets |= new_n_buckets >> 4;
    new_n_buckets |= new_n_buckets >> 8;
    new_n_buckets |= new_n_buckets >> 16;
    new_n_buckets = new_n_buckets.wrapping_add(1);
    if new_n_buckets < 4 {
        new_n_buckets = 4;
    }

    let mut new_flags: *mut u32 = std::ptr::null_mut();
    let mut j: u32 = 1;
    let fsize = |m: u32| -> u32 {
        if m < 16 {
            1
        } else {
            m >> 4
        }
    };
    if (*size_field as f64) >= (new_n_buckets as f64 * HASH_UPPER + 0.5) {
        j = 0;
    } else {
        new_flags = malloc(fsize(new_n_buckets) as u64 * 4).cast::<u32>();
        if new_flags.is_null() {
            return -1;
        }
        for i in 0..fsize(new_n_buckets) {
            *new_flags.add(i as usize) = 0xaaaa_aaaa;
        }
        if *n_buckets_field < new_n_buckets {
            let new_keys =
                realloc((*keys_field).cast(), new_n_buckets as u64 * key_sz).cast::<u32>();
            if new_keys.is_null() {
                free(new_flags.cast());
                return -1;
            }
            *keys_field = new_keys;
            let new_vals =
                realloc((*vals_field).cast(), new_n_buckets as u64 * val_sz).cast::<*mut T>();
            if new_vals.is_null() {
                free(new_flags.cast());
                return -1;
            }
            *vals_field = new_vals;
        }
    }

    if j != 0 {
        let old_n = *n_buckets_field;
        let mut jj: u32 = 0;
        while jj != old_n {
            let flag = *(*flags_field).add((jj >> 4) as usize);
            if ((flag >> ((jj & 0xf) << 1)) & 3) == 0 {
                let mut key = *(*keys_field).add(jj as usize);
                let mut val = *(*vals_field).add(jj as usize);
                let new_mask = new_n_buckets - 1;
                *(*flags_field).add((jj >> 4) as usize) |= 1 << ((jj & 0xf) << 1);
                loop {
                    let k = kh_int_hash(key);
                    let mut i = k & new_mask;
                    let mut step: u32 = 0;
                    while ((*new_flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) == 0 {
                        step += 1;
                        i = (i + step) & new_mask;
                    }
                    *new_flags.add((i >> 4) as usize) &= !(2 << ((i & 0xf) << 1));
                    if i < old_n
                        && ((*(*flags_field).add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 3) == 0
                    {
                        std::ptr::swap(&mut key, (*keys_field).add(i as usize));
                        std::ptr::swap(&mut val, (*vals_field).add(i as usize));
                        *(*flags_field).add((i >> 4) as usize) |= 1 << ((i & 0xf) << 1);
                    } else {
                        *(*keys_field).add(i as usize) = key;
                        *(*vals_field).add(i as usize) = val;
                        break;
                    }
                }
            }
            jj += 1;
        }
        if *n_buckets_field > new_n_buckets {
            *keys_field = realloc((*keys_field).cast(), new_n_buckets as u64 * key_sz).cast();
            *vals_field = realloc((*vals_field).cast(), new_n_buckets as u64 * val_sz).cast();
        }
        free((*flags_field).cast());
        *flags_field = new_flags;
        *n_buckets_field = new_n_buckets;
        *n_occupied_field = *size_field;
        *upper_bound_field = (*n_buckets_field as f64 * HASH_UPPER + 0.5) as u32;
    }
    0
}

#[allow(clippy::too_many_arguments)]
unsafe fn kh_put_int_ptr<T>(
    n_buckets_field: *mut u32,
    size_field: *mut u32,
    n_occupied_field: *mut u32,
    upper_bound_field: *mut u32,
    flags_field: *mut *mut u32,
    keys_field: *mut *mut u32,
    vals_field: *mut *mut *mut T,
    key: u32,
    ret: *mut c_int,
) -> u32 {
    if *n_occupied_field >= *upper_bound_field {
        if *n_buckets_field > (*size_field << 1) {
            if kh_resize_int_ptr::<T>(
                n_buckets_field,
                size_field,
                n_occupied_field,
                upper_bound_field,
                flags_field,
                keys_field,
                vals_field,
                *n_buckets_field - 1,
            ) < 0
            {
                *ret = -1;
                return *n_buckets_field;
            }
        } else if kh_resize_int_ptr::<T>(
            n_buckets_field,
            size_field,
            n_occupied_field,
            upper_bound_field,
            flags_field,
            keys_field,
            vals_field,
            *n_buckets_field + 1,
        ) < 0
        {
            *ret = -1;
            return *n_buckets_field;
        }
    }

    let mask = *n_buckets_field - 1;
    let mut step: u32 = 0;
    let mut x = *n_buckets_field;
    let mut site = *n_buckets_field;
    let k = kh_int_hash(key);
    let mut i = k & mask;
    if ((*(*flags_field).add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) != 0 {
        x = i;
    } else {
        let last = i;
        loop {
            let flag = *(*flags_field).add((i >> 4) as usize);
            let is_empty = (flag >> ((i & 0xf) << 1)) & 2;
            let is_del = (flag >> ((i & 0xf) << 1)) & 1;
            if is_empty != 0 || !(is_del != 0 || *(*keys_field).add(i as usize) != key) {
                break;
            }
            if is_del != 0 {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                x = site;
                break;
            }
        }
        if x == *n_buckets_field {
            let flag = *(*flags_field).add((i >> 4) as usize);
            if ((flag >> ((i & 0xf) << 1)) & 2) != 0 && site != *n_buckets_field {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = *(*flags_field).add((x >> 4) as usize);
    if ((flag >> ((x & 0xf) << 1)) & 2) != 0 {
        *(*keys_field).add(x as usize) = key;
        *(*flags_field).add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        *size_field += 1;
        *n_occupied_field += 1;
        *ret = 1;
    } else if ((flag >> ((x & 0xf) << 1)) & 1) != 0 {
        *(*keys_field).add(x as usize) = key;
        *(*flags_field).add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        *size_field += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

unsafe fn kh_del_int_ptr(
    n_buckets_field: *mut u32,
    size_field: *mut u32,
    flags_field: *mut *mut u32,
    x: u32,
) {
    if x != *n_buckets_field
        && ((*(*flags_field).add((x >> 4) as usize) >> ((x & 0xf) << 1)) & 3) == 0
    {
        *(*flags_field).add((x >> 4) as usize) |= 1 << ((x & 0xf) << 1);
        *size_field -= 1;
    }
}

#[inline]
unsafe fn kh_put_m_tagmap(h: *mut kh_m_tagmap_layout, key: u32, ret: *mut c_int) -> u32 {
    kh_put_int_ptr::<cram_tag_map_layout>(
        &raw mut (*h).n_buckets,
        &raw mut (*h).size,
        &raw mut (*h).n_occupied,
        &raw mut (*h).upper_bound,
        &raw mut (*h).flags,
        &raw mut (*h).keys,
        &raw mut (*h).vals,
        key,
        ret,
    )
}

#[inline]
unsafe fn kh_put_m_metrics(h: *mut kh_m_metrics_layout, key: u32, ret: *mut c_int) -> u32 {
    kh_put_int_ptr::<cram_metrics_layout>(
        &raw mut (*h).n_buckets,
        &raw mut (*h).size,
        &raw mut (*h).n_occupied,
        &raw mut (*h).upper_bound,
        &raw mut (*h).flags,
        &raw mut (*h).keys,
        &raw mut (*h).vals,
        key,
        ret,
    )
}

#[inline]
unsafe fn kh_del_m_metrics(h: *mut kh_m_metrics_layout, x: u32) {
    kh_del_int_ptr(
        &raw mut (*h).n_buckets,
        &raw mut (*h).size,
        &raw mut (*h).flags,
        x,
    );
}

// kh_put for STR->int (FNV1a, equality strcmp == 0). Matches header.h:169
// KHASH_MAP_INIT_STR(m_s2i, int). Used here to populate comp_hdr->TD_hash.
unsafe fn kh_resize_m_s2i(h: *mut kh_m_s2i_layout, mut new_n_buckets: u32) -> c_int {
    const HASH_UPPER: f64 = 0.77;
    let key_sz = std::mem::size_of::<*const c_char>() as u64;
    let val_sz = std::mem::size_of::<c_int>() as u64;

    new_n_buckets = new_n_buckets.wrapping_sub(1);
    new_n_buckets |= new_n_buckets >> 1;
    new_n_buckets |= new_n_buckets >> 2;
    new_n_buckets |= new_n_buckets >> 4;
    new_n_buckets |= new_n_buckets >> 8;
    new_n_buckets |= new_n_buckets >> 16;
    new_n_buckets = new_n_buckets.wrapping_add(1);
    if new_n_buckets < 4 {
        new_n_buckets = 4;
    }

    let mut new_flags: *mut u32 = std::ptr::null_mut();
    let mut j: u32 = 1;
    let fsize = |m: u32| -> u32 {
        if m < 16 {
            1
        } else {
            m >> 4
        }
    };
    if ((*h).size as f64) >= (new_n_buckets as f64 * HASH_UPPER + 0.5) {
        j = 0;
    } else {
        new_flags = malloc(fsize(new_n_buckets) as u64 * 4).cast::<u32>();
        if new_flags.is_null() {
            return -1;
        }
        for i in 0..fsize(new_n_buckets) {
            *new_flags.add(i as usize) = 0xaaaa_aaaa;
        }
        if (*h).n_buckets < new_n_buckets {
            let new_keys =
                realloc((*h).keys.cast(), new_n_buckets as u64 * key_sz).cast::<*const c_char>();
            if new_keys.is_null() {
                free(new_flags.cast());
                return -1;
            }
            (*h).keys = new_keys;
            let new_vals = realloc((*h).vals.cast(), new_n_buckets as u64 * val_sz).cast::<c_int>();
            if new_vals.is_null() {
                free(new_flags.cast());
                return -1;
            }
            (*h).vals = new_vals;
        }
    }

    if j != 0 {
        let old_n = (*h).n_buckets;
        let mut jj: u32 = 0;
        while jj != old_n {
            let flag = *(*h).flags.add((jj >> 4) as usize);
            if ((flag >> ((jj & 0xf) << 1)) & 3) == 0 {
                let mut key = *(*h).keys.add(jj as usize);
                let mut val = *(*h).vals.add(jj as usize);
                let new_mask = new_n_buckets - 1;
                *(*h).flags.add((jj >> 4) as usize) |= 1 << ((jj & 0xf) << 1);
                loop {
                    let k = kh_fnv1a_hash(key);
                    let mut i = k & new_mask;
                    let mut step: u32 = 0;
                    while ((*new_flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) == 0 {
                        step += 1;
                        i = (i + step) & new_mask;
                    }
                    *new_flags.add((i >> 4) as usize) &= !(2 << ((i & 0xf) << 1));
                    if i < old_n
                        && ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 3) == 0
                    {
                        std::ptr::swap(&mut key, (*h).keys.add(i as usize));
                        std::ptr::swap(&mut val, (*h).vals.add(i as usize));
                        *(*h).flags.add((i >> 4) as usize) |= 1 << ((i & 0xf) << 1);
                    } else {
                        *(*h).keys.add(i as usize) = key;
                        *(*h).vals.add(i as usize) = val;
                        break;
                    }
                }
            }
            jj += 1;
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc((*h).keys.cast(), new_n_buckets as u64 * key_sz).cast();
            (*h).vals = realloc((*h).vals.cast(), new_n_buckets as u64 * val_sz).cast();
        }
        free((*h).flags.cast());
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound = ((*h).n_buckets as f64 * HASH_UPPER + 0.5) as u32;
    }
    0
}

unsafe fn kh_put_m_s2i(h: *mut kh_m_s2i_layout, key: *const c_char, ret: *mut c_int) -> u32 {
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > ((*h).size << 1) {
            if kh_resize_m_s2i(h, (*h).n_buckets - 1) < 0 {
                *ret = -1;
                return (*h).n_buckets;
            }
        } else if kh_resize_m_s2i(h, (*h).n_buckets + 1) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }

    let mask = (*h).n_buckets - 1;
    let mut step: u32 = 0;
    let mut x = (*h).n_buckets;
    let mut site = (*h).n_buckets;
    let k = kh_fnv1a_hash(key);
    let mut i = k & mask;
    if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) != 0 {
        x = i;
    } else {
        let last = i;
        loop {
            let flag = *(*h).flags.add((i >> 4) as usize);
            let is_empty = (flag >> ((i & 0xf) << 1)) & 2;
            let is_del = (flag >> ((i & 0xf) << 1)) & 1;
            if is_empty != 0 || !(is_del != 0 || libc::strcmp(*(*h).keys.add(i as usize), key) != 0)
            {
                break;
            }
            if is_del != 0 {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                x = site;
                break;
            }
        }
        if x == (*h).n_buckets {
            let flag = *(*h).flags.add((i >> 4) as usize);
            if ((flag >> ((i & 0xf) << 1)) & 2) != 0 && site != (*h).n_buckets {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = *(*h).flags.add((x >> 4) as usize);
    if ((flag >> ((x & 0xf) << 1)) & 2) != 0 {
        *(*h).keys.add(x as usize) = key;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        (*h).size += 1;
        (*h).n_occupied += 1;
        *ret = 1;
    } else if ((flag >> ((x & 0xf) << 1)) & 1) != 0 {
        *(*h).keys.add(x as usize) = key;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        (*h).size += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

// kh_m_s2u64: khash STR -> uint64 (FNV1a-hashed). Used by lossy_read_names.
// Layout-identical to khash_m_s2u64_t from htslib/cram/khash.h.
#[repr(C)]
struct kh_m_s2u64_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut *const c_char,
    vals: *mut u64,
}

/// FNV-1a hash of a NUL-terminated C string (htslib/htslib/khash.h
/// `__ac_FNV1a_hash_string`).
unsafe fn kh_fnv1a_hash(mut s: *const c_char) -> u32 {
    let offset_basis: u32 = 2_166_136_261;
    let fnv_prime: u32 = 16_777_619;
    let mut h: u32 = offset_basis;
    while *s != 0 {
        h = (h ^ *s as u8 as u32).wrapping_mul(fnv_prime);
        s = s.add(1);
    }
    h
}

unsafe fn kh_resize_m_s2u64(h: *mut kh_m_s2u64_layout, mut new_n_buckets: u32) -> c_int {
    const HASH_UPPER: f64 = 0.77;
    let key_sz = std::mem::size_of::<*const c_char>() as u64;
    let val_sz = std::mem::size_of::<u64>() as u64;

    new_n_buckets = new_n_buckets.wrapping_sub(1);
    new_n_buckets |= new_n_buckets >> 1;
    new_n_buckets |= new_n_buckets >> 2;
    new_n_buckets |= new_n_buckets >> 4;
    new_n_buckets |= new_n_buckets >> 8;
    new_n_buckets |= new_n_buckets >> 16;
    new_n_buckets = new_n_buckets.wrapping_add(1);
    if new_n_buckets < 4 {
        new_n_buckets = 4;
    }

    let mut new_flags: *mut u32 = std::ptr::null_mut();
    let mut j: u32 = 1;
    let fsize = |m: u32| -> u32 {
        if m < 16 {
            1
        } else {
            m >> 4
        }
    };
    if ((*h).size as f64) >= (new_n_buckets as f64 * HASH_UPPER + 0.5) {
        j = 0;
    } else {
        new_flags = malloc(fsize(new_n_buckets) as u64 * 4).cast::<u32>();
        if new_flags.is_null() {
            return -1;
        }
        for i in 0..fsize(new_n_buckets) {
            *new_flags.add(i as usize) = 0xaaaa_aaaa;
        }
        if (*h).n_buckets < new_n_buckets {
            let new_keys =
                realloc((*h).keys.cast(), new_n_buckets as u64 * key_sz).cast::<*const c_char>();
            if new_keys.is_null() {
                free(new_flags.cast());
                return -1;
            }
            (*h).keys = new_keys;
            let new_vals = realloc((*h).vals.cast(), new_n_buckets as u64 * val_sz).cast::<u64>();
            if new_vals.is_null() {
                free(new_flags.cast());
                return -1;
            }
            (*h).vals = new_vals;
        }
    }

    if j != 0 {
        let old_n = (*h).n_buckets;
        let mut jj: u32 = 0;
        while jj != old_n {
            let flag = *(*h).flags.add((jj >> 4) as usize);
            if ((flag >> ((jj & 0xf) << 1)) & 3) == 0 {
                let mut key = *(*h).keys.add(jj as usize);
                let mut val = *(*h).vals.add(jj as usize);
                let new_mask = new_n_buckets - 1;
                *(*h).flags.add((jj >> 4) as usize) |= 1 << ((jj & 0xf) << 1);
                loop {
                    let k = kh_fnv1a_hash(key);
                    let mut i = k & new_mask;
                    let mut step: u32 = 0;
                    while ((*new_flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) == 0 {
                        step += 1;
                        i = (i + step) & new_mask;
                    }
                    *new_flags.add((i >> 4) as usize) &= !(2 << ((i & 0xf) << 1));
                    if i < old_n
                        && ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 3) == 0
                    {
                        std::ptr::swap(&mut key, (*h).keys.add(i as usize));
                        std::ptr::swap(&mut val, (*h).vals.add(i as usize));
                        *(*h).flags.add((i >> 4) as usize) |= 1 << ((i & 0xf) << 1);
                    } else {
                        *(*h).keys.add(i as usize) = key;
                        *(*h).vals.add(i as usize) = val;
                        break;
                    }
                }
            }
            jj += 1;
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc((*h).keys.cast(), new_n_buckets as u64 * key_sz).cast();
            (*h).vals = realloc((*h).vals.cast(), new_n_buckets as u64 * val_sz).cast();
        }
        free((*h).flags.cast());
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound = ((*h).n_buckets as f64 * HASH_UPPER + 0.5) as u32;
    }
    0
}

unsafe fn kh_put_m_s2u64(h: *mut kh_m_s2u64_layout, key: *const c_char, ret: *mut c_int) -> u32 {
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > ((*h).size << 1) {
            if kh_resize_m_s2u64(h, (*h).n_buckets - 1) < 0 {
                *ret = -1;
                return (*h).n_buckets;
            }
        } else if kh_resize_m_s2u64(h, (*h).n_buckets + 1) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }

    let mask = (*h).n_buckets - 1;
    let mut step: u32 = 0;
    let mut x = (*h).n_buckets;
    let mut site = (*h).n_buckets;
    let k = kh_fnv1a_hash(key);
    let mut i = k & mask;
    if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) != 0 {
        x = i;
    } else {
        let last = i;
        loop {
            let flag = *(*h).flags.add((i >> 4) as usize);
            let is_empty = (flag >> ((i & 0xf) << 1)) & 2;
            let is_del = (flag >> ((i & 0xf) << 1)) & 1;
            if is_empty != 0 || !(is_del != 0 || libc::strcmp(*(*h).keys.add(i as usize), key) != 0)
            {
                break;
            }
            if is_del != 0 {
                site = i;
            }
            step += 1;
            i = (i + step) & mask;
            if i == last {
                x = site;
                break;
            }
        }
        if x == (*h).n_buckets {
            let flag = *(*h).flags.add((i >> 4) as usize);
            if ((flag >> ((i & 0xf) << 1)) & 2) != 0 && site != (*h).n_buckets {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = *(*h).flags.add((x >> 4) as usize);
    if ((flag >> ((x & 0xf) << 1)) & 2) != 0 {
        *(*h).keys.add(x as usize) = key;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        (*h).size += 1;
        (*h).n_occupied += 1;
        *ret = 1;
    } else if ((flag >> ((x & 0xf) << 1)) & 1) != 0 {
        *(*h).keys.add(x as usize) = key;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0xf) << 1));
        (*h).size += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

unsafe fn kh_get_m_s2u64(h: *const kh_m_s2u64_layout, key: *const c_char) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let k = kh_fnv1a_hash(key);
    let mut i = k & mask;
    let last = i;
    let mut step: u32 = 0;
    while ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 2) == 0
        && (((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 1) != 0
            || libc::strcmp(*(*h).keys.add(i as usize), key) != 0)
    {
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return (*h).n_buckets;
        }
    }
    if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0xf) << 1)) & 3) != 0 {
        (*h).n_buckets
    } else {
        i
    }
}

unsafe fn kh_destroy_m_s2u64(h: *mut kh_m_s2u64_layout) {
    if h.is_null() {
        return;
    }
    free((*h).flags.cast());
    free((*h).keys.cast());
    free((*h).vals.cast());
    free(h.cast());
}

pub unsafe fn cram_cram_io_h_183_cram_get_block_by_id(
    slice: *mut cram_slice,
    id: c_int,
) -> *mut cram_block {
    let slice = slice.cast::<cram_slice_layout>();
    let mut v = id as u32;
    if !(*slice).block_by_id.is_null() && v < 256 {
        return (*(*slice).block_by_id.add(v as usize)).cast();
    }

    v = 256 + v % 251;
    if !(*slice).block_by_id.is_null() {
        let b = *(*slice).block_by_id.add(v as usize);
        if !b.is_null() && (*(b.cast::<cram_block_layout>())).content_id == id {
            return b.cast();
        }
    }

    let hdr = (*slice).hdr;
    for i in 0..(*hdr).num_blocks {
        let b = *(*slice).block.add(i as usize);
        if !b.is_null()
            && (*(b.cast::<cram_block_layout>())).content_type
                == crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL
            && (*(b.cast::<cram_block_layout>())).content_id == id
        {
            return b.cast();
        }
    }

    std::ptr::null_mut()
}

pub unsafe fn cram_cram_io_h_216_block_resize_exact(b: *mut cram_block, len: usize) -> c_int {
    let b = b.cast::<cram_block_layout>();
    let tmp = realloc((*b).data.cast(), len as u64).cast::<u8>();
    if tmp.is_null() {
        return -1;
    }
    (*b).alloc = len;
    (*b).data = tmp;
    0
}

pub unsafe fn cram_cram_io_h_226_block_resize(b: *mut cram_block, len: usize) -> c_int {
    let block = b.cast::<cram_block_layout>();
    if (*block).alloc > len {
        return 0;
    }

    let mut alloc = (*block).alloc + 800;
    alloc = std::cmp::max(alloc + (alloc >> 2), len);
    cram_cram_io_h_216_block_resize_exact(b, alloc)
}

pub unsafe fn cram_cram_io_h_243_block_grow(b: *mut cram_block, len: usize) -> c_int {
    let block = b.cast::<cram_block_layout>();
    cram_cram_io_h_226_block_resize(b, (*block).byte + len)
}

pub unsafe fn cram_cram_io_h_248_block_append(
    b: *mut cram_block,
    s: *const c_void,
    len: usize,
) -> c_int {
    if cram_cram_io_h_243_block_grow(b, len) < 0 {
        return -1;
    }

    if len != 0 {
        let block = b.cast::<cram_block_layout>();
        memcpy((*block).data.add((*block).byte).cast(), s, len as u64);
        (*block).byte += len;
    }

    0
}

pub unsafe fn cram_cram_io_h_261_block_append_char(b: *mut cram_block, c: c_char) -> c_int {
    if cram_cram_io_h_243_block_grow(b, 1) < 0 {
        return -1;
    }

    let block = b.cast::<cram_block_layout>();
    *(*block).data.add((*block).byte) = c as u8;
    (*block).byte += 1;
    0
}

pub unsafe fn cram_cram_io_h_271_block_append_uint(b: *mut cram_block, i: libc::c_uint) -> c_int {
    if cram_cram_io_h_243_block_grow(b, 11) < 0 {
        return -1;
    }

    let block = b.cast::<cram_block_layout>();
    let cp = (*block).data.add((*block).byte);
    let end = cram_cram_io_h_288_append_uint32(cp, i);
    (*block).byte += end.offset_from(cp) as usize;
    0
}

pub unsafe fn cram_cram_io_h_288_append_uint32(mut cp: *mut u8, mut i: u32) -> *mut u8 {
    if i == 0 {
        *cp = b'0';
        return cp.add(1);
    }

    let mut div = 1_000_000_000u32;
    while div > i {
        div /= 10;
    }
    while div != 0 {
        *cp = (i / div) as u8 + b'0';
        cp = cp.add(1);
        i %= div;
        div /= 10;
    }
    cp
}

pub unsafe fn cram_cram_io_h_326_append_sub32(mut cp: *mut u8, mut i: u32) -> *mut u8 {
    let mut div = 100_000_000u32;
    while div != 0 {
        *cp = (i / div) as u8 + b'0';
        cp = cp.add(1);
        i %= div;
        div /= 10;
    }
    cp
}

pub unsafe fn cram_cram_io_h_340_append_uint64(mut cp: *mut u8, i: u64) -> *mut u8 {
    if i <= 0xffff_ffff {
        return cram_cram_io_h_288_append_uint32(cp, i as u32);
    }

    let mut j = i / 1_000_000_000;
    if j > 1_000_000_000 {
        cp = cram_cram_io_h_288_append_uint32(cp, (j / 1_000_000_000) as u32);
        j %= 1_000_000_000;
        cp = cram_cram_io_h_326_append_sub32(cp, j as u32);
    } else {
        cp = cram_cram_io_h_288_append_uint32(cp, j as u32);
    }
    cram_cram_io_h_326_append_sub32(cp, (i % 1_000_000_000) as u32)
}

pub unsafe fn cram_cram_io_h_646_cram_hfile(
    fd: *mut crate::htslib_rs::hts::cram_fd,
) -> *mut crate::htslib_rs::hts::hFILE {
    (*(fd.cast::<cram_fd_layout>())).fp.cast()
}

// Accessor used by `cram_flush_bridge` to peek at the cram_fd's `idxfp`
// field without re-declaring the (large, private) `cram_fd_layout` struct
// outside this module.
pub unsafe fn cram_fd_idxfp_get(fd: *mut cram_fd) -> *mut BGZF {
    (*fd.cast::<cram_fd_layout>()).idxfp
}

// Setter for `cram_fd::idxfp`. Used by `sam_idx_init` when opening a CRAM
// index sidecar (so sam.rs doesn't need the private `cram_fd_layout` shape).
pub unsafe fn cram_fd_idxfp_set(fd: *mut cram_fd, idxfp: *mut BGZF) {
    (*fd.cast::<cram_fd_layout>()).idxfp = idxfp;
}

/// Accessor for `cram_fd::header`. Used by `sam_hdr_read`'s CRAM branch to
/// duplicate the already-parsed header (libhts' `cram_dopen` consumes the
/// header bytes during `hts_open`, so a re-read would read past the cursor).
/// Returns an opaque pointer so callers don't need the private cram_fd_layout.
pub unsafe fn cram_fd_header_ptr(fd: *mut cram_fd) -> *mut c_void {
    if fd.is_null() {
        return std::ptr::null_mut();
    }
    (*fd.cast::<cram_fd_layout>()).header.cast()
}

/// Accessor for `cram_fd::ref_fn`. Used by `sam_hdr_write_cram` to feed the
/// reference-FASTA path into `cram_load_reference` before writing the header.
pub unsafe fn cram_fd_ref_fn(fd: *mut cram_fd) -> *mut c_char {
    if fd.is_null() {
        return std::ptr::null_mut();
    }
    (*fd.cast::<cram_fd_layout>()).ref_fn
}

// Layout-mirror for the per-decode-job state held in the rqueue. Mirrors
// `cram_decode_job` (htslib/cram/cram_decode.c:3032): a tuple of nullable
// pointers plus an exit code. The native side models the nullable pointers as
// `Option<NonNull<_>>`; the representation is pointer-sized, while accesses
// are forced to acknowledge the null case.
#[repr(C)]
struct cram_decode_job_layout {
    _fd: Option<NonNull<cram_fd>>,
    c: Option<NonNull<cram_container_layout>>,
    s: Option<NonNull<cram_slice_layout>>,
    _h: Option<NonNull<sam_hdr_t>>,
    _exit_code: c_int,
}

// Layout-mirror for the per-encode-job state in the rqueue. Mirrors
// `cram_job` (htslib/cram/cram_io.c:4151): { fd, c }. Used by
// `cram_flush_result` (WRITE-mode result-queue drain).
#[repr(C)]
struct cram_job_layout {
    fd: Option<NonNull<cram_fd>>,
    c: Option<NonNull<cram_container_layout>>,
}

// Native equivalent of htslib/cram/cram_io.c:4168 `cram_flush_result`.
// Drains the WRITE-mode encoder result queue: for each completed job, run
// `cram_flush_container2` (write the encoded container to disk) and free the
// per-container slices + container. Returns 0 on success, -1 on error.
unsafe fn cram_flush_result_native(fd_in: *mut cram_fd) -> c_int {
    let ret: c_int = 0;
    let mut lc: *mut cram_container_layout = std::ptr::null_mut();
    let fdl_in = fd_in.cast::<cram_fd_layout>();
    let rqueue = (*fdl_in)
        .rqueue
        .cast::<crate::htslib_rs::thread_pool::hts_tpool_process>();

    let mut current_fd = fd_in;
    loop {
        let r = crate::htslib_rs::thread_pool::hts_tpool_next_result(rqueue);
        if r.is_null() {
            break;
        }
        let j = crate::htslib_rs::thread_pool::hts_tpool_result_data(r).cast::<cram_job_layout>();
        if j.is_null() {
            crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 0);
            return -1;
        }
        let Some(job_fd) = (*j).fd else {
            crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 0);
            return -1;
        };
        let Some(job_container) = (*j).c else {
            crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 0);
            return -1;
        };
        current_fd = job_fd.as_ptr();
        let c = job_container.as_ptr();
        let fdl = current_fd.cast::<cram_fd_layout>();

        if (*fdl).mode == b'w' as c_int
            && cram_cram_io_c_4089_cram_flush_container2(current_fd, c.cast()) != 0
        {
            return -1;
        }

        // Free per-container slices (filled by encoder).
        if !(*c).slices.is_null() {
            for i in 0..(*c).max_slice {
                let s = *(*c).slices.add(i as usize);
                if !s.is_null() {
                    cram_cram_io_c_4421_cram_free_slice(s.cast());
                }
                if s == (*c).slice {
                    (*c).slice = std::ptr::null_mut();
                }
                *(*c).slices.add(i as usize) = std::ptr::null_mut();
            }
        }

        // Free the current slice (set by encoder & decoder).
        if !(*c).slice.is_null() {
            cram_cram_io_c_4421_cram_free_slice((*c).slice.cast());
            (*c).slice = std::ptr::null_mut();
        }
        (*c).curr_slice = 0;

        // Free the previous container once we switch to a new one.
        if c != lc {
            if !lc.is_null() {
                if (*fdl).ctr == lc {
                    (*fdl).ctr = std::ptr::null_mut();
                }
                if (*fdl).ctr_mt == lc {
                    (*fdl).ctr_mt = std::ptr::null_mut();
                }
                cram_cram_io_c_3705_cram_free_container(lc.cast());
            }
            lc = c;
        }

        crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 1);
    }

    if !lc.is_null() {
        let fdl = current_fd.cast::<cram_fd_layout>();
        if (*fdl).ctr == lc {
            (*fdl).ctr = std::ptr::null_mut();
        }
        if (*fdl).ctr_mt == lc {
            (*fdl).ctr_mt = std::ptr::null_mut();
        }
        cram_cram_io_c_3705_cram_free_container(lc.cast());
    }

    ret
}

// Native equivalent of htslib/cram/cram_decode.c:3632 `cram_drain_rqueue`.
// Called by cram_seek/cram_close in MT mode to flush in-flight decode jobs
// and the optional pending-job slot. Uses native `hts_tpool_*` helpers and
// `cram_free_container`/`cram_free_slice`.
unsafe fn cram_drain_rqueue_native(fdl: *mut cram_fd_layout) {
    if (*fdl).pool.is_null() || (*fdl).rqueue.is_null() {
        return;
    }
    let mut lc: *mut cram_container_layout = std::ptr::null_mut();
    let rqueue = (*fdl)
        .rqueue
        .cast::<crate::htslib_rs::thread_pool::hts_tpool_process>();

    while crate::htslib_rs::thread_pool::hts_tpool_process_empty(rqueue) == 0 {
        let r = crate::htslib_rs::thread_pool::hts_tpool_next_result_wait(rqueue);
        if r.is_null() {
            break;
        }
        let j = crate::htslib_rs::thread_pool::hts_tpool_result_data(r)
            .cast::<cram_decode_job_layout>();
        let Some(job_container) = (*j).c else {
            crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 0);
            break;
        };
        let Some(job_slice) = (*j).s else {
            crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 0);
            break;
        };
        let c = job_container.as_ptr();
        let s = job_slice.as_ptr();
        if (*c).slice == s {
            (*c).slice = std::ptr::null_mut();
        }
        if c != lc {
            if !lc.is_null() {
                if (*fdl).ctr == lc {
                    (*fdl).ctr = std::ptr::null_mut();
                }
                if (*fdl).ctr_mt == lc {
                    (*fdl).ctr_mt = std::ptr::null_mut();
                }
                cram_cram_io_c_3705_cram_free_container(lc.cast());
            }
            lc = c;
        }
        cram_cram_io_c_4421_cram_free_slice(s.cast());
        crate::htslib_rs::thread_pool::hts_tpool_delete_result(r, 1);
    }

    if !(*fdl).job_pending.is_null() {
        let j = (*fdl).job_pending.cast::<cram_decode_job_layout>();
        if let (Some(job_container), Some(job_slice)) = ((*j).c, (*j).s) {
            let c = job_container.as_ptr();
            let s = job_slice.as_ptr();
            if (*c).slice == s {
                (*c).slice = std::ptr::null_mut();
            }
            if c != lc {
                if !lc.is_null() {
                    if (*fdl).ctr == lc {
                        (*fdl).ctr = std::ptr::null_mut();
                    }
                    if (*fdl).ctr_mt == lc {
                        (*fdl).ctr_mt = std::ptr::null_mut();
                    }
                    cram_cram_io_c_3705_cram_free_container(lc.cast());
                }
                lc = c;
            }
            cram_cram_io_c_4421_cram_free_slice(s.cast());
        }
        crate::htslib_rs::c_compat::free(j.cast());
        (*fdl).job_pending = std::ptr::null_mut();
    }

    if !lc.is_null() {
        if (*fdl).ctr == lc {
            (*fdl).ctr = std::ptr::null_mut();
        }
        if (*fdl).ctr_mt == lc {
            (*fdl).ctr_mt = std::ptr::null_mut();
        }
        cram_cram_io_c_3705_cram_free_container(lc.cast());
    }
}

/// Copy a Rust-owned buffer into a libc-`malloc`'d block so the surrounding
/// C-style cram code can `free()` it. Empty input is treated as decoder
/// failure (returns null) — the dispatch short-circuits genuine zero-length
/// blocks before reaching a decoder, so a real output is always non-empty.
unsafe fn cram_dup_to_malloc(src: &[u8]) -> *mut u8 {
    if src.is_empty() {
        return std::ptr::null_mut();
    }
    let p = malloc(src.len() as u64).cast::<u8>();
    if !p.is_null() {
        std::ptr::copy_nonoverlapping(src.as_ptr(), p, src.len());
    }
    p
}

// CRAM compression-method enum tags (cram_block_method_int, cram_structs.h:215).
// Externally-visible values share names with the public CRAM_BLOCK_METHOD_*
// constants but the encoder dispatch operates on the *internal* enum, which
// includes the parameterised forms (GZIP_RLE, RANS_PR*, ARITH_PR*, FQZ_*, TOKA).
const CBMI_RAW: c_int = 0;
const CBMI_GZIP: c_int = 1;
const CBMI_BZIP2: c_int = 2;
const CBMI_LZMA: c_int = 3;
const CBMI_RANS0: c_int = 4;
const CBMI_RANS_PR0: c_int = 5;
const CBMI_ARITH_PR0: c_int = 6;
const CBMI_FQZ: c_int = 7;
const CBMI_TOK3: c_int = 8;
const CBMI_GZIP_RLE: c_int = 11;
const CBMI_GZIP_1: c_int = 12;
const CBMI_FQZ_B: c_int = 13;
const CBMI_FQZ_C: c_int = 14;
const CBMI_FQZ_D: c_int = 15;
const CBMI_RANS1: c_int = 16;
const CBMI_RANS_PR1: c_int = 17;
const CBMI_RANS_PR64: c_int = 18;
const CBMI_RANS_PR9: c_int = 19;
const CBMI_RANS_PR128: c_int = 20;
const CBMI_RANS_PR129: c_int = 21;
const CBMI_RANS_PR192: c_int = 22;
const CBMI_RANS_PR193: c_int = 23;
const CBMI_TOKA: c_int = 24;
const CBMI_ARITH_PR1: c_int = 25;
const CBMI_ARITH_PR64: c_int = 26;
const CBMI_ARITH_PR9: c_int = 27;
const CBMI_ARITH_PR128: c_int = 28;
const CBMI_ARITH_PR129: c_int = 29;
const CBMI_ARITH_PR192: c_int = 30;
const CBMI_ARITH_PR193: c_int = 31;

const CRAM_MAX_METHOD: usize = 32;

// zlib strategy constants.
const Z_FILTERED: c_int = 1;
const Z_DEFAULT_STRATEGY: c_int = 0;
const Z_RLE: c_int = 3;

const DS_RN_LOCAL: c_int = 11;

/// `zlib_mem_deflate` (htslib/cram/cram_io.c:1222). GZIP wrapper deflate
/// for `[data, data+size)` at the given level/strategy. Returns a libc
/// malloc-owned buffer of size `*cdata_size`, or null on failure.
///
/// Calls system zlib directly with `deflateInit2(level, Z_DEFLATED, 15|16,
/// 9, strat)` — windowBits=15|16 selects gzip wrapping, memLevel=9 matches
/// htslib's `cram_io.c:1248`. Going through `flate2::write::GzEncoder`
/// (the previous implementation) hardcoded `Z_DEFAULT_STRATEGY` and OS=0xff
/// and produced bytes that differed from C's CRAM output even on identical
/// input. Matching C exactly here is what makes bam_to_cram byte-identical.
unsafe fn cram_cram_io_c_1222_zlib_mem_deflate(
    data: *mut c_char,
    size: usize,
    cdata_size: *mut usize,
    level: c_int,
    strat: c_int,
) -> *mut c_char {
    use crate::htslib_rs::bgzf;
    // zlib return codes / flush modes / methods.
    const Z_OK: c_int = 0;
    const Z_STREAM_END: c_int = 1;
    const Z_NO_FLUSH: c_int = 0;
    const Z_FINISH: c_int = 4;
    const Z_DEFLATED: c_int = 8;

    *cdata_size = 0;

    let Some(zlib) = bgzf::system_zlib() else {
        return std::ptr::null_mut();
    };

    // C uses size*1.05 + 100 as initial allocation (cram_io.c:1231).
    let cdata_alloc = ((size as f64) * 1.05 + 100.0) as usize;
    let cdata = malloc(cdata_alloc.max(1) as u64).cast::<c_char>();
    if cdata.is_null() {
        return std::ptr::null_mut();
    }

    let mut zs: bgzf::z_stream = std::mem::zeroed();
    zs.next_in = data.cast::<u8>();
    zs.avail_in = size as c_uint;
    zs.next_out = cdata.cast::<u8>();
    zs.avail_out = cdata_alloc as c_uint;
    // Z_BINARY = 0 (default after zeroed()).

    let ret = (zlib.deflate_init2)(
        &mut zs,
        level,
        Z_DEFLATED,
        15 | 16, // gzip wrapping
        9,       // memLevel
        strat,
        (zlib.zlib_version)(),
        std::mem::size_of::<bgzf::z_stream>() as c_int,
    );
    if ret != Z_OK {
        free(cdata.cast());
        return std::ptr::null_mut();
    }

    // Loop with Z_NO_FLUSH while we still have input; then Z_FINISH to flush.
    let mut cdata_pos: usize = 0;
    while zs.avail_in != 0 {
        zs.next_out = cdata.cast::<u8>().add(cdata_pos);
        zs.avail_out = (cdata_alloc - cdata_pos) as c_uint;
        if cdata_alloc <= cdata_pos {
            (zlib.deflate_end)(&mut zs);
            free(cdata.cast());
            return std::ptr::null_mut();
        }
        let r = (zlib.deflate)(&mut zs, Z_NO_FLUSH);
        cdata_pos = cdata_alloc - zs.avail_out as usize;
        if r != Z_OK {
            break;
        }
    }
    let fin = (zlib.deflate)(&mut zs, Z_FINISH);
    if fin != Z_STREAM_END {
        (zlib.deflate_end)(&mut zs);
        free(cdata.cast());
        return std::ptr::null_mut();
    }
    let total = zs.total_out as usize;
    if (zlib.deflate_end)(&mut zs) != Z_OK {
        free(cdata.cast());
        return std::ptr::null_mut();
    }
    *cdata_size = total;
    cdata
}

/// `cram_compress_by_method` (htslib/cram/cram_io.c:1757).
///
/// Dispatch a single encoder for the given internal-enum `method`. The
/// caller-owned output buffer is returned via libc malloc; its size is
/// written to `*out_size`. Returns null on failure.
///
/// `s` is the cram_slice (needed by FQZ to walk per-record lengths/flags);
/// may be null for non-FQZ methods.
#[allow(clippy::too_many_arguments)]
unsafe fn cram_cram_io_c_1757_cram_compress_by_method(
    s: *mut cram_slice_layout,
    in_: *mut c_char,
    in_size: usize,
    _content_id: c_int,
    out_size: *mut usize,
    method: c_int,
    level: c_int,
    strat: c_int,
) -> *mut c_char {
    match method {
        CBMI_GZIP | CBMI_GZIP_RLE | CBMI_GZIP_1 => {
            cram_cram_io_c_1222_zlib_mem_deflate(in_, in_size, out_size, level, strat)
        }

        CBMI_BZIP2 => {
            // libbz2 not linked in pure-Rust build; behave like C without
            // HAVE_LIBBZ2 (return null, caller treats as failure).
            std::ptr::null_mut()
        }

        CBMI_FQZ | CBMI_FQZ_B | CBMI_FQZ_C | CBMI_FQZ_D => {
            if s.is_null() {
                return std::ptr::null_mut();
            }
            // Build an fqz_slice from this cram_slice's records: per-record
            // qual offsets into the DS_QS block give per-record lengths.
            let num_records = (*(*s).hdr).num_records;
            if num_records < 0 {
                return std::ptr::null_mut();
            }
            // sizeof(fqz_slice) + 2*num_records*sizeof(uint32_t).
            let f_sz = std::mem::size_of::<crate::htslib_rs::htscodecs::fqzcomp_qual::fqz_slice>()
                + 2 * (num_records as usize) * std::mem::size_of::<u32>();
            let f =
                malloc(f_sz as u64).cast::<crate::htslib_rs::htscodecs::fqzcomp_qual::fqz_slice>();
            if f.is_null() {
                return std::ptr::null_mut();
            }
            (*f).num_records = num_records;
            let len_ptr = (f as *mut u8)
                .add(std::mem::size_of::<
                    crate::htslib_rs::htscodecs::fqzcomp_qual::fqz_slice,
                >())
                .cast::<u32>();
            let flags_ptr = len_ptr.add(num_records as usize);
            (*f).len = len_ptr;
            (*f).flags = flags_ptr;
            // DS_QS index (12) — see cram_structs.h.
            const DS_QS_IDX: usize = 12;
            let qs_block = *(*s).block.add(DS_QS_IDX);
            let qs_uncomp = (*qs_block).uncomp_size;
            for i in 0..num_records as isize {
                let rec = (*s).crecs.offset(i);
                *(*f).flags.offset(i) = (*rec).flags as u32;
                let this_qual = (*rec).qual as i32;
                let next_qual = if (i + 1) < num_records as isize {
                    (*(*s).crecs.offset(i + 1)).qual as i32
                } else {
                    qs_uncomp
                };
                *(*f).len.offset(i) = (next_qual - this_qual) as u32;
            }
            // Run fqz_compress (strat & 0xff is cram vers; strat >> 8 is fqz strat).
            let mut out_sz_local: usize = 0;
            let in_slice = std::slice::from_raw_parts_mut(in_.cast::<u8>(), in_size);
            let v = crate::htslib_rs::htscodecs::fqzcomp_qual::fqz_compress(
                strat & 0xff,
                &mut *f,
                in_slice,
                &mut out_sz_local,
                strat >> 8,
                None,
            );
            free(f.cast());
            if v.is_empty() {
                *out_size = 0;
                return std::ptr::null_mut();
            }
            *out_size = v.len();
            let alloc = v.len().max(1);
            let p = malloc(alloc as u64).cast::<c_char>();
            if p.is_null() {
                *out_size = 0;
                return std::ptr::null_mut();
            }
            memcpy(p.cast(), v.as_ptr().cast(), v.len() as u64);
            // Honour the C contract: write the codec-reported size.
            let _ = out_sz_local;
            p
        }

        CBMI_LZMA => {
            // xz2 is wired in the dormant mirror but not in this module's
            // dependency surface; treat as unavailable (the C build can be
            // configured the same way via HAVE_LIBLZMA=no).
            std::ptr::null_mut()
        }

        CBMI_RANS0 | CBMI_RANS1 => {
            let mut out_size_u: c_uint = 0;
            let order: c_int = if method == CBMI_RANS0 { 0 } else { 1 };
            let cp = crate::htslib_rs::htscodecs::rans_static::rans_compress(
                in_.cast::<u8>(),
                in_size as c_uint,
                &mut out_size_u,
                order,
            );
            *out_size = out_size_u as usize;
            cp.cast::<c_char>()
        }

        CBMI_RANS_PR0 | CBMI_RANS_PR1 | CBMI_RANS_PR64 | CBMI_RANS_PR9 | CBMI_RANS_PR128
        | CBMI_RANS_PR129 | CBMI_RANS_PR192 | CBMI_RANS_PR193 => {
            // methmap maps RANS_PR1..RANS_PR193 to order bit-fields.
            const METHMAP: [c_int; 7] = [1, 64, 9, 128, 129, 192, 193];
            let m_order = if method == CBMI_RANS_PR0 {
                0
            } else {
                METHMAP[(method - CBMI_RANS_PR1) as usize]
            };
            let mut out_size_u: u32 = 0;
            let input = std::slice::from_raw_parts(in_.cast::<u8>(), in_size);
            let v = crate::htslib_rs::htscodecs::rans_static4x16pr::rans_compress_4x16(
                input,
                &mut out_size_u,
                m_order | crate::htslib_rs::htscodecs::rans_static4x16pr::RANS_ORDER_SIMD_AUTO,
            );
            *out_size = v.len();
            if v.is_empty() {
                return std::ptr::null_mut();
            }
            let p = malloc(v.len().max(1) as u64).cast::<c_char>();
            if p.is_null() {
                *out_size = 0;
                return std::ptr::null_mut();
            }
            memcpy(p.cast(), v.as_ptr().cast(), v.len() as u64);
            p
        }

        CBMI_ARITH_PR0 | CBMI_ARITH_PR1 | CBMI_ARITH_PR64 | CBMI_ARITH_PR9 | CBMI_ARITH_PR128
        | CBMI_ARITH_PR129 | CBMI_ARITH_PR192 | CBMI_ARITH_PR193 => {
            const METHMAP: [c_int; 7] = [1, 64, 9, 128, 129, 192, 193];
            let order = if method == CBMI_ARITH_PR0 {
                0
            } else {
                METHMAP[(method - CBMI_ARITH_PR1) as usize]
            };
            let mut out_size_u: u32 = 0;
            let input = std::slice::from_raw_parts(in_.cast::<u8>(), in_size);
            let v = crate::htslib_rs::htscodecs::arith_dynamic::arith_compress_to(
                input,
                None,
                &mut out_size_u,
                order,
            );
            *out_size = v.len();
            if v.is_empty() {
                return std::ptr::null_mut();
            }
            let p = malloc(v.len().max(1) as u64).cast::<c_char>();
            if p.is_null() {
                *out_size = 0;
                return std::ptr::null_mut();
            }
            memcpy(p.cast(), v.as_ptr().cast(), v.len() as u64);
            p
        }

        CBMI_TOK3 | CBMI_TOKA => {
            let mut out_len: i32 = 0;
            let mut lev = level;
            if method == CBMI_TOK3 && lev > 3 {
                lev = 3;
            }
            let use_arith = if method == CBMI_TOK3 { 0 } else { 1 };
            let blk = std::slice::from_raw_parts_mut(in_, in_size);
            let v = crate::htslib_rs::htscodecs::tokenise_name3::tok3_encode_names(
                blk,
                in_size as i32,
                lev,
                use_arith,
                &mut out_len,
                None,
            );
            *out_size = out_len as usize;
            match v {
                Some(buf) => {
                    if buf.is_empty() {
                        return std::ptr::null_mut();
                    }
                    let p = malloc(buf.len().max(1) as u64).cast::<c_char>();
                    if p.is_null() {
                        *out_size = 0;
                        return std::ptr::null_mut();
                    }
                    memcpy(p.cast(), buf.as_ptr().cast(), buf.len() as u64);
                    p
                }
                None => std::ptr::null_mut(),
            }
        }

        CBMI_RAW => std::ptr::null_mut(),

        _ => std::ptr::null_mut(),
    }
}

/// `cram_compress_block3` (htslib/cram/cram_io.c:1913).
///
/// Core encoder dispatch: optionally selects between a curated set of codecs
/// using per-block metrics, otherwise compresses with the cached best method.
/// On success, replaces `b->data` with the compressed buffer and sets
/// `b->method`/`b->comp_size`. Returns 0 on success, -1 on failure.
unsafe fn cram_cram_io_c_1913_cram_compress_block3(
    fd: *mut cram_fd,
    s: *mut cram_slice_layout,
    b: *mut cram_block_layout,
    metrics: *mut cram_metrics_layout,
    mut method: c_int,
    mut level: c_int,
    recurse: c_int,
) -> c_int {
    if b.is_null() {
        return 0;
    }

    let orig_method = method;
    let comp: *mut c_char;
    let mut comp_size: usize = 0;
    let strat: c_int;

    // methmap (cram_io.c:1929): maps internal enum to the "external" enum
    // for the on-wire b->method field.
    const METHMAP: [c_int; 32] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, // RAW..TOK3
        0, 0, // reserved
        1, 1, // GZIP_RLE, GZIP_1 -> GZIP
        7, 7, 7, // FQZ_b, FQZ_c, FQZ_d -> FQZ
        4, // RANS1 -> RANS
        5, 5, 5, 5, 5, 5, 5, // RANS_PR1..RANS_PR193 -> RANSPR
        8, // TOKA -> TOK3
        6, 6, 6, 6, 6, 6, 6, // ARITH_PR1..ARITH_PR193 -> ARITH
    ];

    let fdl = fd.cast::<cram_fd_layout>();

    if (*b).method != CBMI_RAW {
        // Already compressed (eg shared block reused).
        return 0;
    }

    if method == -1 {
        method = 1 << CBMI_GZIP;
        if (*fdl).use_bz2 != 0 {
            method |= 1 << CBMI_BZIP2;
        }
        if (*fdl).use_lzma != 0 {
            method |= 1 << CBMI_LZMA;
        }
    }

    if level == -1 {
        level = (*fdl).level;
    }

    if method == CBMI_RAW || level == 0 || (*b).uncomp_size == 0 {
        (*b).method = CBMI_RAW;
        (*b).comp_size = (*b).uncomp_size;
        return 0;
    }

    fn abs_i32(a: i32) -> i32 {
        if a >= 0 {
            a
        } else {
            -a
        }
    }

    if !metrics.is_null() {
        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).metrics_lock);
        // Sudden changes in size trigger a retrial.
        if (*metrics).input_avg_sz != 0
            && ((*b).uncomp_size / 4 - 750 > (*metrics).input_avg_sz
                || (*b).uncomp_size < (*metrics).input_avg_sz / 4 - 750)
            && abs_i32((*b).uncomp_size - (*metrics).input_avg_sz) / 10 > (*metrics).input_avg_delta
        {
            (*metrics).next_trial = 0;
        }

        if (*metrics).trial > 0 || {
            (*metrics).next_trial -= 1;
            (*metrics).next_trial <= 0
        } {
            let unpackable = (*metrics).unpackable;
            let mut sz_best: usize = (*b).uncomp_size as usize;
            let mut sz: [usize; CRAM_MAX_METHOD] = [0; CRAM_MAX_METHOD];
            let mut method_best: c_int = 0; // RAW
            let mut c_best: *mut c_char = std::ptr::null_mut();

            (*metrics).input_avg_delta = (0.9
                * ((*metrics).input_avg_delta as f64
                    + abs_i32((*b).uncomp_size - (*metrics).input_avg_sz) as f64))
                as c_int;

            (*metrics).input_avg_sz += ((*b).uncomp_size as f64 * 0.2) as c_int;
            (*metrics).input_avg_sz = ((*metrics).input_avg_sz as f64 * 0.8) as c_int;

            if (*metrics).revised_method != 0 {
                method = (*metrics).revised_method;
            } else {
                (*metrics).revised_method = method;
            }

            if (*metrics).next_trial <= 0 {
                (*metrics).next_trial = TRIAL_SPAN;
                (*metrics).trial = NTRIALS;
                for m in 0..CRAM_MAX_METHOD {
                    (*metrics).sz[m] /= 2;
                }
                (*metrics).unpackable = 0;
            }

            // Unpackable: avoid bit-pack methods on data with too many symbols.
            if unpackable != 0 && ((*fdl).version >> 8) > 3 {
                if (method & (1 << CBMI_RANS_PR128)) != 0 {
                    method = (method | (1 << CBMI_RANS_PR0)) & !(1 << CBMI_RANS_PR128);
                }
                if (method & (1 << CBMI_RANS_PR129)) != 0 {
                    method = (method | (1 << CBMI_RANS_PR1)) & !(1 << CBMI_RANS_PR129);
                }
                if (method & (1 << CBMI_RANS_PR192)) != 0 {
                    method = (method | (1 << CBMI_RANS_PR64)) & !(1 << CBMI_RANS_PR192);
                }
                if (method & (1 << CBMI_RANS_PR193)) != 0 {
                    method = (method | (1 << CBMI_RANS_PR64) | (1 << CBMI_RANS_PR1))
                        & !(1 << CBMI_RANS_PR193);
                }

                if (method & (1 << CBMI_ARITH_PR128)) != 0 {
                    method = (method | (1 << CBMI_ARITH_PR0)) & !(1 << CBMI_ARITH_PR128);
                }
                if (method & (1 << CBMI_ARITH_PR129)) != 0 {
                    method = (method | (1 << CBMI_ARITH_PR1)) & !(1 << CBMI_ARITH_PR129);
                }
                if (method & (1 << CBMI_ARITH_PR192)) != 0 {
                    method = (method | (1 << CBMI_ARITH_PR64)) & !(1 << CBMI_ARITH_PR192);
                }
                if (method & (1u32 << CBMI_ARITH_PR193) as c_int) != 0 {
                    method = (method | (1 << CBMI_ARITH_PR64) | (1 << CBMI_ARITH_PR1))
                        & !((1u32 << CBMI_ARITH_PR193) as c_int);
                }
            }

            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).metrics_lock);

            for m in 0..CRAM_MAX_METHOD as c_int {
                if (method & (1u32 << m) as c_int) != 0 {
                    let mut lvl = level;
                    let mstrat: c_int = match m {
                        x if x == CBMI_GZIP => Z_FILTERED,
                        x if x == CBMI_GZIP_1 => {
                            lvl = 1;
                            Z_DEFAULT_STRATEGY
                        }
                        x if x == CBMI_GZIP_RLE => Z_RLE,
                        x if x == CBMI_FQZ => (*fdl).version >> 8,
                        x if x == CBMI_FQZ_B => ((*fdl).version >> 8) + 256,
                        x if x == CBMI_FQZ_C => ((*fdl).version >> 8) + 2 * 256,
                        x if x == CBMI_FQZ_D => ((*fdl).version >> 8) + 3 * 256,
                        x if x == CBMI_TOK3 => 0,
                        x if x == CBMI_TOKA => 1,
                        _ => 0,
                    };

                    let c = cram_cram_io_c_1757_cram_compress_by_method(
                        s,
                        (*b).data.cast::<c_char>(),
                        (*b).uncomp_size as usize,
                        (*b).content_id,
                        &mut sz[m as usize],
                        m,
                        lvl,
                        mstrat,
                    );

                    if !c.is_null() && sz_best > sz[m as usize] {
                        sz_best = sz[m as usize];
                        method_best = m;
                        if !c_best.is_null() {
                            free(c_best.cast());
                        }
                        c_best = c;
                    } else if !c.is_null() {
                        free(c.cast());
                    } else {
                        sz[m as usize] = c_uint::MAX as usize;
                    }
                } else {
                    sz[m as usize] = c_uint::MAX as usize;
                }
            }

            if !c_best.is_null() {
                free((*b).data.cast());
                (*b).data = c_best.cast::<u8>();
                (*b).method = method_best;
                (*b).comp_size = sz_best as i32;
            }

            // Accumulate stats for all methods tried.
            crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).metrics_lock);
            for (metric_sz, &method_sz) in (*metrics)
                .sz
                .iter_mut()
                .zip(sz.iter())
                .take(CRAM_MAX_METHOD)
            {
                *metric_sz = (*metric_sz as usize).saturating_add(method_sz + 2000) as c_int;
            }

            // When enough trials performed, find the best on average.
            (*metrics).trial -= 1;
            if (*metrics).trial == 0 {
                let mut best_method: c_int = CBMI_RAW;
                let mut best_sz: c_int = c_int::MAX;

                // Relative costs of methods. See cram_io.c:2117.
                let mut meth_cost: [f64; 32] = [
                    1.00, 1.04, 1.07, 1.08, 1.00, 1.00, 1.04, 1.05, 1.05, 1.00, 1.00, 1.01, 1.01,
                    1.05, 1.05, 1.05, 1.01, 1.01, 1.00, 1.03, 1.00, 1.01, 1.00, 1.01, 1.07, 1.04,
                    1.04, 1.04, 1.03, 1.04, 1.04, 1.04,
                ];
                let _ = &mut meth_cost; // suppress unused-mut

                let fd_level = (*fdl).level;
                if fd_level <= 1 {
                    for (metric_sz, &cost) in (*metrics)
                        .sz
                        .iter_mut()
                        .zip(meth_cost.iter())
                        .take(CRAM_MAX_METHOD)
                    {
                        *metric_sz = (*metric_sz as f64 * (1.0 + (cost - 1.0) * 4.0)) as c_int;
                    }
                } else if fd_level <= 3 {
                    for (metric_sz, &cost) in (*metrics)
                        .sz
                        .iter_mut()
                        .zip(meth_cost.iter())
                        .take(CRAM_MAX_METHOD)
                    {
                        *metric_sz = (*metric_sz as f64 * (1.0 + (cost - 1.0))) as c_int;
                    }
                } else if fd_level <= 6 {
                    for (metric_sz, &cost) in (*metrics)
                        .sz
                        .iter_mut()
                        .zip(meth_cost.iter())
                        .take(CRAM_MAX_METHOD)
                    {
                        *metric_sz = (*metric_sz as f64 * (1.0 + (cost - 1.0) / 2.0)) as c_int;
                    }
                } else if fd_level <= 7 {
                    for (metric_sz, &cost) in (*metrics)
                        .sz
                        .iter_mut()
                        .zip(meth_cost.iter())
                        .take(CRAM_MAX_METHOD)
                    {
                        *metric_sz = (*metric_sz as f64 * (1.0 + (cost - 1.0) / 3.0)) as c_int;
                    }
                }

                // Ensure these are never used; BSC and ZSTD slots.
                (*metrics).sz[9] = c_int::MAX;
                (*metrics).sz[10] = c_int::MAX;

                for m in 0..CRAM_MAX_METHOD as c_int {
                    if (*metrics).sz[m as usize] == 0 || (method & (1u32 << m) as c_int) == 0 {
                        continue;
                    }
                    if best_sz > (*metrics).sz[m as usize] {
                        best_sz = (*metrics).sz[m as usize];
                        best_method = m;
                    }
                }

                if best_method != (*metrics).method {
                    (*metrics).consistency = 0;
                } else {
                    let factor = 2.0f64.min(1.0 + (*metrics).consistency as f64 / 4.0);
                    (*metrics).next_trial = ((*metrics).next_trial as f64 * factor) as c_int;
                    (*metrics).consistency += 1;
                }

                (*metrics).method = best_method;
                strat = match best_method {
                    x if x == CBMI_GZIP => Z_FILTERED,
                    x if x == CBMI_GZIP_1 => Z_DEFAULT_STRATEGY,
                    x if x == CBMI_GZIP_RLE => Z_RLE,
                    x if x == CBMI_FQZ => (*fdl).version >> 8,
                    x if x == CBMI_FQZ_B => ((*fdl).version >> 8) + 256,
                    x if x == CBMI_FQZ_C => ((*fdl).version >> 8) + 2 * 256,
                    x if x == CBMI_FQZ_D => ((*fdl).version >> 8) + 3 * 256,
                    x if x == CBMI_TOK3 => 0,
                    x if x == CBMI_TOKA => 1,
                    _ => 0,
                };
                (*metrics).strat = strat;

                // MAXDELTA=0.20, MAXFAILS=4.
                const MAXDELTA: f64 = 0.20;
                const MAXFAILS: c_int = 4;
                for m in 0..CRAM_MAX_METHOD as c_int {
                    if best_method == m {
                        (*metrics).cnt[m as usize] = 0;
                        (*metrics).extra[m as usize] = 0.0;
                    } else if best_sz < (*metrics).sz[m as usize] {
                        let r = (*metrics).sz[m as usize] as f64 / best_sz as f64 - 1.0;
                        let mul = 1 + (if (*fdl).level >= 7 { 1 } else { 0 });
                        (*metrics).cnt[m as usize] += 1;
                        if (*metrics).cnt[m as usize] >= MAXFAILS * mul {
                            (*metrics).extra[m as usize] += r;
                            if (*metrics).extra[m as usize] >= MAXDELTA * mul as f64 {
                                method &= !(1u32 << m) as c_int;
                            }
                        }

                        // Special case for fqzcomp.
                        if (m == CBMI_FQZ || m == CBMI_FQZ_B || m == CBMI_FQZ_C || m == CBMI_FQZ_D)
                            && (*metrics).sz[m as usize] > best_sz
                        {
                            method &= !(1u32 << m) as c_int;
                        }
                    }
                }

                (*metrics).revised_method = method;
            }
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).metrics_lock);
        } else {
            (*metrics).input_avg_delta = (0.9
                * ((*metrics).input_avg_delta as f64
                    + abs_i32((*b).uncomp_size - (*metrics).input_avg_sz) as f64))
                as c_int;

            (*metrics).input_avg_sz += ((*b).uncomp_size as f64 * 0.2) as c_int;
            (*metrics).input_avg_sz = ((*metrics).input_avg_sz as f64 * 0.8) as c_int;

            strat = (*metrics).strat;
            let cached_method = (*metrics).method;

            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).metrics_lock);
            comp = cram_cram_io_c_1757_cram_compress_by_method(
                s,
                (*b).data.cast::<c_char>(),
                (*b).uncomp_size as usize,
                (*b).content_id,
                &mut comp_size,
                cached_method,
                if cached_method == CBMI_GZIP_1 {
                    1
                } else {
                    level
                },
                strat,
            );
            if comp.is_null() {
                if recurse == 0 {
                    hts_log_cstr(
                        HTS_LOG_WARNING,
                        c"cram_compress_block".as_ptr(),
                        c"Compressed block method failed, redoing trial".as_ptr(),
                    );
                    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).metrics_lock);
                    (*metrics).trial = NTRIALS;
                    (*metrics).next_trial = TRIAL_SPAN;
                    (*metrics).revised_method = orig_method;
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).metrics_lock);
                    return cram_cram_io_c_1913_cram_compress_block3(
                        fd, s, b, metrics, method, level, 1,
                    );
                }
                return -1;
            }

            if comp_size < (*b).uncomp_size as usize {
                free((*b).data.cast());
                (*b).data = comp.cast::<u8>();
                (*b).comp_size = comp_size as i32;
                (*b).method = cached_method;
            } else {
                free(comp.cast());
            }
        }
    } else {
        // No cached metrics — just do GZIP.
        comp = cram_cram_io_c_1757_cram_compress_by_method(
            s,
            (*b).data.cast::<c_char>(),
            (*b).uncomp_size as usize,
            (*b).content_id,
            &mut comp_size,
            CBMI_GZIP,
            level,
            Z_FILTERED,
        );
        if comp.is_null() {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"cram_compress_block".as_ptr(),
                c"Compression failed!".as_ptr(),
            );
            return -1;
        }
        if comp_size < (*b).uncomp_size as usize {
            free((*b).data.cast());
            (*b).data = comp.cast::<u8>();
            (*b).comp_size = comp_size as i32;
            (*b).method = CBMI_GZIP;
        } else {
            free(comp.cast());
        }
        strat = Z_FILTERED;
        let _ = strat;
    }

    (*b).method = METHMAP[(*b).method as usize];

    let _ = DS_RN_LOCAL; // referenced for compatibility with cram_io.c layout
    0
}

// Default CRAM version numbers (htslib/cram/cram_io.c:5254).
const CRAM_OPEN_DEFAULT_MAJOR: c_int = 3;
const CRAM_OPEN_DEFAULT_MINOR: c_int = 1;

// Internal helper: free up a partially-built cram_fd from the C `err:` label
// of cram_dopen (htslib/cram/cram_io.c:5418). Mirrors the cascading free path
// the C original would take from goto err, including the cram_close-style
// teardown of any allocations that completed before failure.
unsafe fn cram_cram_io_c_5560_cram_dopen_cleanup_err(fd: *mut cram_fd_layout) {
    // metrics
    for k in 0..CRAM_DS_END {
        if !(*fd).m[k].is_null() {
            free((*fd).m[k].cast());
        }
    }
    if !(*fd).tags_used.is_null() {
        let h = (*fd).tags_used.cast::<kh_generic_layout>();
        if !(*h).flags.is_null() {
            free((*h).flags.cast());
        }
        if !(*h).keys.is_null() {
            free((*h).keys.cast());
        }
        if !(*h).vals.is_null() {
            free((*h).vals.cast());
        }
        free(h.cast());
    }
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fd).metrics_lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fd).ref_lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fd).range_lock);
    crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut (*fd).bam_list_lock);
    if !(*fd).refs.is_null() {
        cram_cram_io_c_2427_refs_free((*fd).refs.cast());
    }
    if !(*fd).file_def.is_null() {
        cram_cram_io_c_4698_cram_free_file_def((*fd).file_def.cast());
    }
    if !(*fd).header.is_null() {
        crate::htslib_rs::sam::sam_hdr_destroy((*fd).header.cast());
    }
    if !(*fd).prefix.is_null() {
        free((*fd).prefix.cast());
    }
    free(fd.cast());
}

// Native equivalents of the CRAM record encoder entry points
// (htslib/cram/cram_encode.c). This is the last libhts-side delegation in
// production: sam_write1's CRAM branch (src/sam.rs) used to call
// `hts_sys::sam_write1` which forwarded to `cram_put_bam_seq`. The encoder
// itself remains in the cram-mirror tree (c2rust output), but the three
// entry-points below run natively against the production layout-mirror
// structs; when a new slice/container is needed they hand off to the
// existing `cram_flush_container_mt` (already native via
// `cram_flush_bridge`) which drives the dormant encoder pipeline.
//
// Layout invariants: production `cram_fd_layout` / `cram_container_layout`
// / `cram_slice_layout` are byte-identical to the htslib C structs (and to
// the cram-mirror versions). Field-name differences vs. the c2rust port:
// production uses snake_case `last_ri_count` where C/mirror use `last_RI_count`.

#[repr(C)]
struct spare_bams_layout {
    bams: *mut *mut bam1_t,
    next: *mut spare_bams_layout,
}

// Native cram_update_curr_slice (htslib/cram/cram_encode.c:3262).
unsafe fn cram_update_curr_slice_native(c: *mut cram_container_layout, version: c_int) {
    let s = (*c).slice;
    let hdr = (*s).hdr;
    if (*c).multi_seq != 0 {
        (*hdr).ref_seq_id = -2;
        (*hdr).ref_seq_start = 0;
        (*hdr).ref_seq_span = 0;
    } else if (*c).curr_ref == -1 && (version >> 8) >= 4 {
        // CRAM_ge31 = >=3.1, which is version>>8 >= 4 (3.1 is 0x0301... wait)
        // Actually CRAM_ge31 is true for v3.1+; encoded version is (major<<8)|minor.
        // v3.1 = 0x0301 = 769, v4.0 = 0x0400 = 1024. CRAM_ge31 checks
        // version >= (3 * 256 + 1). We mirror that here.
        (*hdr).ref_seq_id = -1;
        (*hdr).ref_seq_start = 0;
        (*hdr).ref_seq_span = 0;
    } else {
        (*hdr).ref_seq_id = (*c).curr_ref;
        (*hdr).ref_seq_start = (*c).first_base;
        let span = (*c).last_base - (*c).first_base + 1;
        (*hdr).ref_seq_span = if span > 0 { span } else { 0 };
    }
    (*hdr).num_records = (*c).curr_rec;

    if (*c).curr_slice == 0 {
        if (*c).ref_seq_id != (*hdr).ref_seq_id {
            (*c).ref_seq_id = (*hdr).ref_seq_id;
        }
        (*c).ref_seq_start = (*c).first_base;
    }
    (*c).curr_slice += 1;
}

// Native cram_next_container (htslib/cram/cram_encode.c:3299). Called by
// cram_put_bam_seq when the current slice/container is full or the
// reference id changes (single-seq mode). Flushes the current container,
// allocates a new one, and seeds a fresh slice.
unsafe fn cram_next_container_native(
    fd: *mut cram_fd,
    b: *mut bam1_t,
) -> *mut cram_container_layout {
    let fdl = fd.cast::<cram_fd_layout>();
    let mut c = (*fdl).ctr;
    let bref = (*b).core.tid;

    // First occurrence
    if (*c).curr_ref == -2 {
        (*c).curr_ref = bref;
    }

    if !(*c).slice.is_null() {
        cram_update_curr_slice_native(c, (*fdl).version);
    }

    // Flush container when full or ref changes (single-seq mode).
    if (*c).curr_slice == (*c).max_slice || (bref != (*c).curr_ref && (*c).multi_seq == 0) {
        (*c).ref_seq_span = (*fdl).last_base as i64 - (*c).ref_seq_start + 1;
        // (Optional info log omitted; native hts_log_cstr would need a CString
        // built here. Matches behavior; doesn't affect correctness.)

        if -1 == cram_cram_io_c_4275_cram_flush_container_mt(fd, c.cast()) {
            return std::ptr::null_mut();
        }
        if (*fdl).pool.is_null() {
            // Single-threaded: cram_flush_container has handled the encode, but
            // the encoder doesn't free per-slice memory in non-MT mode — do it
            // here ahead of allocating the new container.
            for i in 0..(*c).max_slice {
                let s = *(*c).slices.add(i as usize);
                if !s.is_null() {
                    cram_cram_io_c_4421_cram_free_slice(s.cast());
                }
                *(*c).slices.add(i as usize) = std::ptr::null_mut();
            }
            (*c).slice = std::ptr::null_mut();
            (*c).curr_slice = 0;
            cram_cram_io_c_3705_cram_free_container(c.cast());
        }

        let new_ctr = cram_new_container((*fdl).seqs_per_slice, (*fdl).slices_per_container);
        if new_ctr.is_null() {
            return std::ptr::null_mut();
        }
        (*fdl).ctr = new_ctr.cast();
        c = (*fdl).ctr;

        crate::htslib_rs::c_compat::pthread_mutex_lock(&mut (*fdl).ref_lock);
        (*c).no_ref = (*fdl).no_ref;
        (*c).embed_ref = (*fdl).embed_ref;
        (*c).record_counter = (*fdl).record_counter;
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        (*c).curr_ref = bref;
    }

    let bpos = (*b).core.pos + 1;
    (*c).last_pos = bpos;
    (*c).first_base = bpos;
    (*c).last_base = bpos;

    // New slice.
    let new_slice = cram_cram_io_c_4506_cram_new_slice(2 /* MAPPED_SLICE */, (*c).max_rec);
    if new_slice.is_null() {
        return std::ptr::null_mut();
    }
    let slice = new_slice.cast::<cram_slice_layout>();
    *(*c).slices.add((*c).curr_slice as usize) = slice;
    (*c).slice = slice;
    let hdr = (*slice).hdr;

    if (*c).multi_seq != 0 {
        (*hdr).ref_seq_id = -2;
        (*hdr).ref_seq_start = 0;
        (*slice).last_apos = 1;
    } else {
        (*hdr).ref_seq_id = bref;
        (*hdr).ref_seq_start = bpos;
        (*slice).last_apos = bpos;
    }
    (*c).curr_rec = 0;
    (*c).s_num_bases = 0;
    (*c).n_mapped = 0;

    // QO field: v4+ uses 0, earlier uses 1.
    (*c).qs_seq_orient = if ((*fdl).version >> 8) >= 4 { 0 } else { 1 };

    c
}

// Function-pointer types for the varint decoders stored in cram_fd::vv.
// 32-bit signed/unsigned share one shape; 64-bit another. They are kept in the
// `vv` table as *mut c_void and are transmuted back here for invocation.
type CramVarintDecode32 =
    unsafe fn(*mut crate::htslib_rs::hts::cram_fd, *mut i32, *mut u32) -> c_int;
type CramVarintDecode64 =
    unsafe fn(*mut crate::htslib_rs::hts::cram_fd, *mut i64, *mut u32) -> c_int;

#[inline]
unsafe fn vv_decode32(
    vv: &varint_vec_layout,
    fd: *mut crate::htslib_rs::hts::cram_fd,
    val: *mut i32,
    crc: *mut u32,
) -> c_int {
    let f: CramVarintDecode32 = cram_fn(vv.varint_decode32_crc);
    f(fd, val, crc)
}
#[inline]
unsafe fn vv_decode32s(
    vv: &varint_vec_layout,
    fd: *mut crate::htslib_rs::hts::cram_fd,
    val: *mut i32,
    crc: *mut u32,
) -> c_int {
    let f: CramVarintDecode32 = cram_fn(vv.varint_decode32s_crc);
    f(fd, val, crc)
}
#[inline]
unsafe fn vv_decode64(
    vv: &varint_vec_layout,
    fd: *mut crate::htslib_rs::hts::cram_fd,
    val: *mut i64,
    crc: *mut u32,
) -> c_int {
    let f: CramVarintDecode64 = cram_fn(vv.varint_decode64_crc);
    f(fd, val, crc)
}

/// original: cram_store_container (htslib/cram/cram_io.c:3960)
///
/// Serializes the container struct fields (length, ref_seq_id, start/span,
/// num_records, record_counter, num_bases, num_blocks, num_landmarks,
/// landmark[], CRC32) into the caller's `dat` buffer using the fd's varint
/// dispatch vector. Updates `*size` to the actual bytes written; returns 0
/// on success and -1 if the supplied buffer is too small. Byte-for-byte
/// equivalent to the C cram_store_container — uses the same varint
/// function pointers (varint_put32/32s/64) that the C code calls via
/// `fd->vv.varint_put32(...)`.
unsafe fn cram_cram_io_c_3960_cram_store_container(
    fd: *mut cram_fd_layout,
    c: *mut cram_container_layout,
    dat: *mut c_char,
    size: *mut c_int,
) -> c_int {
    let fdl = fd;
    let cl = c;

    if cram_cram_io_c_3947_cram_container_size(c.cast()) > *size {
        return -1;
    }

    let dat_start = dat;
    let mut cp = dat;
    let null_end: *mut c_char = std::ptr::null_mut();

    let major = (*fdl).version >> 8;

    // length: v1 → itf8; v2/3 → raw LE int32; v4 → varint32.
    // (The C code emits LE int32 for everything that isn't v1, since
    // cram_store_container is invoked only with v2/v3 in practice; we
    // mirror that exactly. v4 currently uses the same LE-int32 path via
    // the fall-through, matching the C source.)
    if major == 1 {
        cp = cp.add(cram_cram_io_c_277_itf8_put(cp, (*cl).length) as usize);
    } else {
        // *(int32_t *)cp = le_int4(c->length)
        std::ptr::write_unaligned(cp.cast::<i32>(), (*cl).length.to_le());
        cp = cp.add(4);
    }

    if (*cl).multi_seq != 0 {
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, -2) as usize);
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, 0) as usize);
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, 0) as usize);
    } else {
        cp = cp.add(((*fdl).vv.varint_put32s.unwrap())(cp, null_end, (*cl).ref_seq_id) as usize);
        if major >= 4 {
            cp = cp
                .add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).ref_seq_start) as usize);
            cp = cp
                .add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).ref_seq_span) as usize);
        } else {
            cp = cp.add(
                ((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).ref_seq_start as i32)
                    as usize,
            );
            cp = cp.add(
                ((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).ref_seq_span as i32) as usize,
            );
        }
    }
    cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).num_records) as usize);
    if major == 2 {
        cp = cp.add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).record_counter) as usize);
    } else if major >= 3 {
        cp = cp.add(
            ((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).record_counter as i32) as usize,
        );
    }
    cp = cp.add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).num_bases) as usize);
    cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).num_blocks) as usize);
    cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).num_landmarks) as usize);
    let mut i = 0;
    while i < (*cl).num_landmarks {
        let lm = *(*cl).landmark.add(i as usize);
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, lm) as usize);
        i += 1;
    }

    if major >= 3 {
        let consumed = cp.offset_from(dat_start) as usize;
        (*cl).crc32 = crate::htslib_rs::bgzf::hts_crc32(0, dat_start.cast(), consumed);
        *cp = ((*cl).crc32 & 0xff) as c_char;
        *cp.add(1) = (((*cl).crc32 >> 8) & 0xff) as c_char;
        *cp.add(2) = (((*cl).crc32 >> 16) & 0xff) as c_char;
        *cp.add(3) = (((*cl).crc32 >> 24) & 0xff) as c_char;
        cp = cp.add(4);
    }

    *size = cp.offset_from(dat_start) as c_int;
    0
}

/// original: cram_write_container (htslib/cram/cram_io.c:4023)
///
/// Writes the container struct fields directly to the underlying hFILE,
/// using a stack buffer for the typical short-landmark case and falling
/// back to a heap allocation for very large landmark arrays (matching C's
/// `61 + c->num_landmarks * 10 >= 1024` fallback). Returns 0 on success,
/// -1 on allocation or hwrite failure. Byte-for-byte equivalent to the
/// C cram_write_container — identical varint dispatch and CRC32 layout.
unsafe fn cram_cram_io_c_4023_cram_write_container(
    fd: *mut cram_fd_layout,
    c: *mut cram_container_layout,
) -> c_int {
    let fdl = fd;
    let cl = c;

    let mut buf_a = [0u8; 1024];
    let need = 61 + (*cl).num_landmarks * 10;
    let (buf, owned): (*mut c_char, bool) = if need >= 1024 {
        let p = malloc(need as u64).cast::<c_char>();
        if p.is_null() {
            return -1;
        }
        (p, true)
    } else {
        (buf_a.as_mut_ptr().cast::<c_char>(), false)
    };

    let mut cp = buf;
    let null_end: *mut c_char = std::ptr::null_mut();
    let major = (*fdl).version >> 8;

    // length: v1 → itf8; v2..=3 → raw LE int32; v4+ → varint32.
    if major == 1 {
        cp = cp.add(cram_cram_io_c_277_itf8_put(cp, (*cl).length) as usize);
    } else if major <= 3 {
        std::ptr::write_unaligned(cp.cast::<i32>(), (*cl).length.to_le());
        cp = cp.add(4);
    } else {
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).length) as usize);
    }

    if (*cl).multi_seq != 0 {
        // C uses (uint32_t)-2 here; varint_put32 takes int32_t, so -2 is identical.
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, -2) as usize);
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, 0) as usize);
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, 0) as usize);
    } else {
        cp = cp.add(((*fdl).vv.varint_put32s.unwrap())(cp, null_end, (*cl).ref_seq_id) as usize);
        if major >= 4 {
            cp = cp
                .add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).ref_seq_start) as usize);
            cp = cp
                .add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).ref_seq_span) as usize);
        } else {
            cp = cp.add(
                ((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).ref_seq_start as i32)
                    as usize,
            );
            cp = cp.add(
                ((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).ref_seq_span as i32) as usize,
            );
        }
    }
    cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).num_records) as usize);
    if major >= 3 {
        cp = cp.add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).record_counter) as usize);
    } else {
        cp = cp.add(
            ((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).record_counter as i32) as usize,
        );
    }
    cp = cp.add(((*fdl).vv.varint_put64.unwrap())(cp, null_end, (*cl).num_bases) as usize);
    cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).num_blocks) as usize);
    cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, (*cl).num_landmarks) as usize);
    let mut i = 0;
    while i < (*cl).num_landmarks {
        let lm = *(*cl).landmark.add(i as usize);
        cp = cp.add(((*fdl).vv.varint_put32.unwrap())(cp, null_end, lm) as usize);
        i += 1;
    }

    if major >= 3 {
        let consumed = cp.offset_from(buf) as usize;
        (*cl).crc32 = crate::htslib_rs::bgzf::hts_crc32(0, buf.cast(), consumed);
        *cp = ((*cl).crc32 & 0xff) as c_char;
        *cp.add(1) = (((*cl).crc32 >> 8) & 0xff) as c_char;
        *cp.add(2) = (((*cl).crc32 >> 16) & 0xff) as c_char;
        *cp.add(3) = (((*cl).crc32 >> 24) & 0xff) as c_char;
        cp = cp.add(4);
    }

    let nbytes = cp.offset_from(buf) as usize;
    let fp = (*fdl).fp;
    let wrote = htslib_hfile_h_292_hwrite(fp, buf.cast(), nbytes);
    if wrote != nbytes as libc::ssize_t {
        if owned {
            free(buf.cast());
        }
        return -1;
    }
    if owned {
        free(buf.cast());
    }
    0
}

// original: cram_index (htslib/cram/cram_structs.h:720)
//
// Byte-faithful Rust mirror of the libhts `cram_index` struct. Lives in
// production so the native cram_index_free / cram_index_query / etc. can
// walk the tree without going through the cram-mirror tree.
#[repr(C)]
pub struct cram_index_layout {
    pub nslice: c_int,
    pub nalloc: c_int,
    pub e: *mut cram_index_layout,
    pub refid: c_int,
    pub start: c_int,
    pub end: c_int,
    pub nseq: c_int,
    pub slice: c_int,
    pub len: c_int,
    pub offset: i64,
    pub e_next: *mut cram_index_layout,
}

pub unsafe fn cram_cram_codecs_h_230_cram_not_enough_bits(
    blk: *mut cram_block,
    nbits: c_int,
) -> c_int {
    let blk = blk.cast::<cram_block_layout>();
    if nbits < 0 {
        return 1;
    }

    let byte = (*blk).byte;
    let uncomp_size = (*blk).uncomp_size;
    if byte >= uncomp_size as usize && nbits > 0 {
        return 1;
    }

    if uncomp_size >= 0 && byte <= uncomp_size as usize {
        let remaining = uncomp_size - byte as i32;
        if remaining <= i32::MAX / 8 + 1 && remaining * 8 + (*blk).bit - 7 < nbits {
            return 1;
        }
    }

    0
}

#[repr(C)]
struct pooled_alloc_test_xyz {
    x: c_int,
    y: c_int,
    z: c_int,
}

pub fn cram_os_h_155_le_int4(x: u32) -> u32 {
    u32::from_le(x)
}

pub fn cram_os_h_158_le_int2(x: u16) -> u16 {
    u16::from_le(x)
}

// ===========================================================================
// Native CRAM decode pipeline (htslib/cram/cram_decode.c)
//
// Faithful port of cram_decode_slice / cram_to_bam / cram_get_seq /
// cram_get_bam_seq (and the helpers they call) adapted from the dormant mirror
// src/cram/cram_decode.rs. Struct types are byte-identical re-declarations of
// the production *_layout structs but use the mirror's field names so the
// transpiled bodies port verbatim; all on-disk / codec / reference / slice
// dependencies delegate to the existing native production functions
// (cram_cram_io_c_* / cram_cram_codecs_c_* / crate::htslib_rs::sam::*).
//
// These functions are additive: nothing in production calls them yet (sam.rs
// wiring is P3). They are exercised by the end-to-end differential test below.
// ===========================================================================
#[allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_assignments,
    unused_variables,
    clippy::all
)]

// Public entry point into the native CRAM decode pipeline.
//
// Mirrors C `cram_get_bam_seq(fp->fp.cram, &b)` — decodes one CRAM record into
// the caller-provided bam1_t (whose `data` buffer may be re-alloc'd by the
// callee via `bam_set1`). The bam1_t handle itself is stable: the `**bam` C
// signature is preserved only because `cram_to_bam` dereferences it; nothing
// writes back a new pointer.
//
// Returns:
//   >= 0 : success (the new bam1_t `l_data`, matching C semantics).
//    < 0 : `cram_get_seq` failed (true EOF, range stop, or a decode error).
//          Callers must consult `cram_eof(fd)` to distinguish EOF (-1 here) vs
//          error, exactly as C `sam_read1_cram` does.
//
// This is the named entry point called from `sam.rs::sam_read1_cram` (the live
// `sam_read1` CRAM path) — see step P3 of the cut-over plan.
pub unsafe fn cram_get_bam_seq_native(
    fd: *mut crate::htslib_rs::hts::cram_fd,
    bam: *mut crate::htslib_rs::sam::bam1_t,
) -> c_int {
    // Stash the caller's bam1_t* in a local so we can hand a `**bam1_t` to the
    // inner pipeline (which matches the C `cram_get_bam_seq(fd, bam1_t **b)`
    // signature). The pipeline reads `*bam_0` to find the target bam1_t and
    // only mutates `(*b).data` / `(*b).core` in place; it never replaces the
    // bam1_t pointer itself, so `bp` is unchanged on return.
    let mut bp: *mut crate::htslib_rs::sam::bam1_t = bam;
    decode_pipeline::cram_get_bam_seq(fd.cast(), (&raw mut bp).cast())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::htslib_rs::sam::{
        bam1_core_t, BAM_CDEL, BAM_CINS, BAM_CMATCH, BAM_CSOFT_CLIP, BAM_FPAIRED,
    };
    use std::ffi::{CStr, CString};

    fn test_refs_marker() -> *mut refs_t_layout {
        static REFS_MARKER: u8 = 0;
        std::ptr::from_ref(&REFS_MARKER)
            .cast::<refs_t_layout>()
            .cast_mut()
    }

    fn test_hfile_marker() -> *mut hFILE {
        static HFILE_MARKER: u8 = 0;
        std::ptr::from_ref(&HFILE_MARKER).cast::<hFILE>().cast_mut()
    }

    fn test_void_token(value: usize) -> *mut c_void {
        cram_data_series_id_ptr(value)
    }

    unsafe fn call_cram_voption_words(
        fd: *mut cram_fd,
        opt: hts_fmt_option,
        words: &mut [usize],
    ) -> c_int {
        let mut reg_save = [0usize; 6];
        let mut overflow = [0usize; 8];
        for (i, word) in words.iter().copied().enumerate() {
            if i < reg_save.len() {
                reg_save[i] = word;
            } else {
                overflow[i - reg_save.len()] = word;
            }
        }
        let mut args = crate::htslib_rs::c_compat::__va_list_tag {
            gp_offset: 0,
            fp_offset: 48,
            overflow_arg_area: overflow.as_mut_ptr().cast(),
            reg_save_area: reg_save.as_mut_ptr().cast(),
        };
        unsafe { cram_set_voption(fd, opt, &mut args) }
    }

    #[test]
    fn cram_set_voption_updates_direct_integer_string_and_profile_options() {
        unsafe {
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.bases_per_slice = CRAM_DEFAULT_BASES_PER_SLICE;
            fd.level = CRAM_DEFAULT_LEVEL;
            fd.range.refid = 0;
            let fd_ptr = (&mut fd as *mut cram_fd_layout).cast::<cram_fd>();

            let mut words = [7usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_DECODE_MD,
                    &mut words,
                ),
                0
            );
            assert_eq!(fd.decode_md, 7);

            let mut words = [11usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_SEQS_PER_SLICE,
                    &mut words,
                ),
                0
            );
            assert_eq!(fd.seqs_per_slice, 11);
            assert_eq!(fd.bases_per_slice, 5500);

            let mut words = [1usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_LOSSY_NAMES,
                    &mut words
                ),
                0
            );
            assert_eq!(fd.lossy_read_names, 1);
            assert_eq!(fd.tlen_approx, 1);
            assert_eq!(fd.tlen_zero, 1);

            let prefix = CString::new("translated-prefix").unwrap();
            let mut words = [prefix.as_ptr() as usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_PREFIX,
                    &mut words
                ),
                0
            );
            assert_eq!(CStr::from_ptr(fd.prefix).to_bytes(), b"translated-prefix");

            let mut words = [0usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_REQUIRED_FIELDS,
                    &mut words,
                ),
                0
            );
            assert_ne!(fd.required_fields & crate::htslib_rs::cram::SAM_POS, 0);

            let mut words = [HTS_PROFILE_SMALL as usize];
            assert_eq!(
                call_cram_voption_words(fd_ptr, HTS_OPT_PROFILE, &mut words),
                0
            );
            assert_eq!(fd.level, 6);
            assert_eq!(fd.use_bz2, 1);
            assert_eq!(fd.use_fqz, 1);
            assert_eq!(fd.seqs_per_slice, 25000);

            free(fd.prefix.cast());
        }
    }

    #[test]
    fn cram_set_voption_version_and_error_paths_match_public_contract() {
        unsafe {
            let mut fd: cram_fd_layout = std::mem::zeroed();
            let fd_ptr = (&mut fd as *mut cram_fd_layout).cast::<cram_fd>();

            let version = CString::new("3.1").unwrap();
            let mut words = [version.as_ptr() as usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_VERSION,
                    &mut words
                ),
                0
            );
            assert_eq!(fd.version, 3 * 256 + 1);
            assert_eq!(fd.use_rans, 1);
            assert_eq!(fd.use_tok, 1);

            let bad_version = CString::new("9.9").unwrap();
            let mut words = [bad_version.as_ptr() as usize];
            assert_eq!(
                call_cram_voption_words(
                    fd_ptr,
                    crate::htslib_rs::cram::CRAM_OPT_VERSION,
                    &mut words
                ),
                -1
            );
            assert_eq!(*__errno_location(), EINVAL);

            let mut words = [0usize];
            assert_eq!(
                call_cram_voption_words(
                    std::ptr::null_mut(),
                    crate::htslib_rs::cram::CRAM_OPT_DECODE_MD,
                    &mut words
                ),
                -1
            );
            assert_eq!(*__errno_location(), libc::EBADF);

            let mut words = [0usize];
            assert_eq!(call_cram_voption_words(fd_ptr, 9999, &mut words), -1);
            assert_eq!(*__errno_location(), EINVAL);
        }
    }

    #[test]
    fn cram_os_little_endian_helpers_match_host_macros() {
        assert_eq!(cram_os_h_155_le_int4(0x1234_5678), 0x1234_5678);
        assert_eq!(cram_os_h_158_le_int2(0x1234), 0x1234);
    }

    #[test]
    fn cram_string_pool_allocates_duplicates_and_grows_like_c_allocator() {
        unsafe {
            let pool = cram_string_alloc_c_55_string_pool_create(8);
            assert!(!pool.is_null());
            assert_eq!((*pool).max_length, CRAM_STRING_ALLOC_MIN_STR_SIZE);
            assert_eq!((*pool).nstrings, 0);

            let input = CString::new("alpha").unwrap();
            let dup = cram_string_alloc_c_149_string_dup(pool, input.as_ptr());
            assert!(!dup.is_null());
            assert_eq!(CStr::from_ptr(dup).to_bytes(), b"alpha");
            assert_eq!((*pool).nstrings, 1);
            assert_eq!((*(*pool).strings).used, 6);

            let beta = CString::new("betamax").unwrap();
            let nduped = cram_string_alloc_c_153_string_ndup(pool, beta.as_ptr(), 4);
            assert!(!nduped.is_null());
            assert_eq!(CStr::from_ptr(nduped).to_bytes(), b"beta");
            assert_eq!((*pool).nstrings, 1);

            let oversized = cram_string_alloc_c_117_string_alloc(pool, 2048);
            assert!(!oversized.is_null());
            assert_eq!((*pool).max_length, 2048);
            assert_eq!((*pool).nstrings, 2);

            assert!(cram_string_alloc_c_117_string_alloc(pool, 0).is_null());
            cram_string_alloc_c_103_string_pool_destroy(pool);
        }
    }

    #[test]
    fn cram_string_pool_exact_fit_starts_new_slab_like_c_allocator() {
        unsafe {
            let pool = cram_string_alloc_c_55_string_pool_create(CRAM_STRING_ALLOC_MIN_STR_SIZE);
            assert!(!pool.is_null());

            let first =
                cram_string_alloc_c_117_string_alloc(pool, CRAM_STRING_ALLOC_MIN_STR_SIZE - 1);
            assert!(!first.is_null());
            assert_eq!((*pool).nstrings, 1);
            assert_eq!((*(*pool).strings).used, CRAM_STRING_ALLOC_MIN_STR_SIZE - 1);

            let second = cram_string_alloc_c_117_string_alloc(pool, 1);
            assert!(!second.is_null());
            assert_eq!((*pool).nstrings, 2);
            assert_eq!((*(*pool).strings.add(1)).used, 1);
            assert_ne!(first, second);

            cram_string_alloc_c_103_string_pool_destroy(pool);
        }
    }

    #[test]
    fn cram_encode_sub_idx_matches_four_base_scan_rule() {
        unsafe {
            let mut key = *b"ACGT";
            assert_eq!(
                cram_cram_encode_c_70_sub_idx(key.as_mut_ptr().cast(), b'A' as c_char),
                0
            );
            assert_eq!(
                cram_cram_encode_c_70_sub_idx(key.as_mut_ptr().cast(), b'T' as c_char),
                3
            );
            assert_eq!(
                cram_cram_encode_c_70_sub_idx(key.as_mut_ptr().cast(), b'N' as c_char),
                4
            );
        }
    }

    #[test]
    fn cram_encode_bam_aux2i_end_decodes_little_endian_and_bounds_checks() {
        unsafe {
            let c = [b'c', 0xfe];
            assert_eq!(
                cram_cram_encode_c_1253_bam_aux2i_end(c.as_ptr(), c.as_ptr().add(c.len())),
                -2
            );
            let s = [b'S', 0x34, 0x12];
            assert_eq!(
                cram_cram_encode_c_1253_bam_aux2i_end(s.as_ptr(), s.as_ptr().add(s.len())),
                0x1234
            );
            let i = [b'i', 0xfc, 0xff, 0xff, 0xff];
            assert_eq!(
                cram_cram_encode_c_1253_bam_aux2i_end(i.as_ptr(), i.as_ptr().add(i.len())),
                -4
            );
            let short = [b'I', 1, 2, 3];
            assert_eq!(
                cram_cram_encode_c_1253_bam_aux2i_end(
                    short.as_ptr(),
                    short.as_ptr().add(short.len())
                ),
                0
            );
        }
    }

    #[test]
    fn cram_encode_expected_template_count_uses_flags_tc_and_sa_tags() {
        unsafe {
            let mut data = vec![0, b'T', b'C', b'C', 5];
            let mut b = bam1_t {
                core: bam1_core_t {
                    pos: 0,
                    tid: 0,
                    bin: 0,
                    qual: 0,
                    l_extranul: 0,
                    flag: BAM_FPAIRED as u16,
                    l_qname: 1,
                    n_cigar: 0,
                    l_qseq: 0,
                    mtid: 0,
                    mpos: 0,
                    isize: 0,
                },
                id: 0,
                data: data.as_mut_ptr(),
                l_data: data.len() as c_int,
                m_data: data.len() as u32,
                mempolicy_and_reserved: 0,
            };

            assert_eq!(
                cram_cram_encode_c_1246_bam_data_end(&mut b).cast::<u8>(),
                b.data.add(b.l_data as usize)
            );
            assert_eq!(cram_cram_encode_c_1301_expected_template_count(&mut b), 5);

            let mut sa_data = vec![0, b'S', b'A', b'Z', b'r', b';', 0];
            b.core.flag = 0;
            b.data = sa_data.as_mut_ptr();
            b.l_data = sa_data.len() as c_int;
            b.m_data = sa_data.len() as u32;
            assert_eq!(
                cram_cram_encode_c_1301_expected_template_count(&mut b),
                c_int::MAX
            );

            let mut no_aux = vec![0];
            b.data = no_aux.as_mut_ptr();
            b.l_data = no_aux.len() as c_int;
            b.m_data = no_aux.len() as u32;
            assert_eq!(cram_cram_encode_c_1301_expected_template_count(&mut b), 1);
        }
    }

    #[test]
    fn cram_encode_next_cigar_op_skips_selected_ops_and_updates_state() {
        unsafe {
            let mut cigar = [
                (2 << BAM_CIGAR_SHIFT) | BAM_CSOFT_CLIP as u32,
                (3 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (1 << BAM_CIGAR_SHIFT) | BAM_CINS as u32,
                (1 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32,
            ];
            let mut skip = [0; 16];
            skip[BAM_CSOFT_CLIP as usize] = 1;
            skip[BAM_CINS as usize] = 1;
            let mut spos = 0;
            let mut cig_ind = 0;
            let mut cig_op = 0;
            let mut cig_len = 0;

            assert_eq!(
                cram_cram_encode_c_1476_next_cigar_op(
                    cigar.as_mut_ptr(),
                    cigar.len() as u32,
                    skip.as_mut_ptr(),
                    &mut spos,
                    &mut cig_ind,
                    &mut cig_op,
                    &mut cig_len
                ),
                BAM_CMATCH
            );
            assert_eq!(spos, 2);
            assert_eq!(cig_len, 2);
            assert_eq!(
                cram_cram_encode_c_1476_next_cigar_op(
                    cigar.as_mut_ptr(),
                    cigar.len() as u32,
                    skip.as_mut_ptr(),
                    &mut spos,
                    &mut cig_ind,
                    &mut cig_op,
                    &mut cig_len
                ),
                BAM_CMATCH
            );
            assert_eq!(
                cram_cram_encode_c_1476_next_cigar_op(
                    cigar.as_mut_ptr(),
                    cigar.len() as u32,
                    skip.as_mut_ptr(),
                    &mut spos,
                    &mut cig_ind,
                    &mut cig_op,
                    &mut cig_len
                ),
                BAM_CMATCH
            );
            assert_eq!(
                cram_cram_encode_c_1476_next_cigar_op(
                    cigar.as_mut_ptr(),
                    cigar.len() as u32,
                    skip.as_mut_ptr(),
                    &mut spos,
                    &mut cig_ind,
                    &mut cig_op,
                    &mut cig_len
                ),
                BAM_CDEL
            );
            assert_eq!(spos, 3);
            assert_eq!(
                cram_cram_encode_c_1476_next_cigar_op(
                    cigar.as_mut_ptr(),
                    cigar.len() as u32,
                    skip.as_mut_ptr(),
                    &mut spos,
                    &mut cig_ind,
                    &mut cig_op,
                    &mut cig_len
                ),
                -1
            );
        }
    }

    #[test]
    fn cram_external_slice_header_accessors_read_nullable_outputs_like_c() {
        unsafe {
            let mut hdr = cram_block_slice_hdr_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_MAPPED_SLICE,
                ref_seq_id: 17,
                ref_seq_start: 101,
                ref_seq_span: 250,
                num_records: 9,
                record_counter: 0,
                num_blocks: 4,
                num_content_ids: 0,
                block_content_ids: std::ptr::null_mut(),
                ref_base_id: 33,
                md5: [0; 16],
            };
            let h = (&mut hdr as *mut cram_block_slice_hdr_layout).cast();
            assert_eq!(cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(h), 4);
            assert_eq!(
                cram_cram_external_c_504_cram_slice_hdr_get_embed_ref_id(h),
                33
            );

            let mut refid = 0;
            let mut start = 0;
            let mut span = 0;
            cram_cram_external_c_508_cram_slice_hdr_get_coords(
                h, &mut refid, &mut start, &mut span,
            );
            assert_eq!((refid, start, span), (17, 101, 250));

            refid = -1;
            cram_cram_external_c_508_cram_slice_hdr_get_coords(
                h,
                &mut refid,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            assert_eq!(refid, 17);
        }
    }

    #[test]
    fn cram_external_container_accessors_match_c_fields() {
        unsafe {
            let mut landmarks = [11, 22, 33];
            let mut container = cram_container_layout {
                length: 100,
                ref_seq_id: 4,
                ref_seq_start: 500,
                ref_seq_span: 75,
                record_counter: 9,
                num_bases: 1234,
                num_records: 56,
                num_blocks: 7,
                num_landmarks: landmarks.len() as i32,
                landmark: landmarks.as_mut_ptr(),
                offset: 0,
                comp_hdr: std::ptr::null_mut(),
                comp_hdr_block: std::ptr::null_mut(),
                ..std::mem::zeroed()
            };
            let c = (&mut container as *mut cram_container_layout).cast();

            assert_eq!(cram_cram_external_c_75_cram_container_get_length(c), 100);
            cram_cram_external_c_79_cram_container_set_length(c, 101);
            assert_eq!(container.length, 101);
            assert_eq!(cram_cram_external_c_84_cram_container_get_num_blocks(c), 7);
            cram_cram_external_c_88_cram_container_set_num_blocks(c, 8);
            assert_eq!(container.num_blocks, 8);
            assert_eq!(
                cram_cram_external_c_92_cram_container_get_num_records(c),
                56
            );
            assert_eq!(cram_container_get_num_records(c), 56);
            assert_eq!(
                cram_cram_external_c_96_cram_container_get_num_bases(c),
                1234
            );
            assert_eq!(cram_container_get_num_bases(c), 1234);

            let mut nlandmarks = 0;
            let got = cram_cram_external_c_104_cram_container_get_landmarks(c, &mut nlandmarks);
            assert_eq!(nlandmarks, 3);
            assert_eq!(*got.add(1), 22);

            let mut replacement = [44, 55];
            cram_cram_external_c_112_cram_container_set_landmarks(
                c,
                replacement.len() as i32,
                replacement.as_mut_ptr(),
            );
            assert_eq!(container.num_landmarks, 2);
            assert_eq!(*container.landmark.add(0), 44);

            let mut refid = 0;
            let mut start = 0;
            let mut span = 0;
            cram_cram_external_c_124_cram_container_get_coords(
                c, &mut refid, &mut start, &mut span,
            );
            assert_eq!((refid, start, span), (4, 500, 75));
            refid = 0;
            start = 0;
            span = 0;
            cram_container_get_coords(c, &mut refid, &mut start, &mut span);
            assert_eq!((refid, start, span), (4, 500, 75));

            refid = -1;
            cram_cram_external_c_124_cram_container_get_coords(
                c,
                &mut refid,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            assert_eq!(refid, 4);
        }
    }

    #[test]
    fn cram_external_block_size_and_method_accessors_match_fields() {
        unsafe {
            let mut block = cram_block_layout {
                method: crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP,
                orig_method: crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 7,
                comp_size: 12,
                uncomp_size: 18,
                crc32: 99,
                idx: 5,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 5,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();
            assert_eq!(cram_cram_external_c_522_cram_block_get_content_id(b), 7);
            assert_eq!(cram_cram_external_c_525_cram_block_get_comp_size(b), 12);
            assert_eq!(cram_cram_external_c_526_cram_block_get_uncomp_size(b), 18);
            assert_eq!(cram_cram_external_c_527_cram_block_get_crc32(b), 99);
            assert_eq!(
                cram_cram_external_c_533_cram_block_get_content_type(b),
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL
            );
            assert_eq!(cram_cram_external_c_529_cram_block_get_size(b), 5);
            assert_eq!(
                cram_cram_external_c_530_cram_block_get_method(b),
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            cram_cram_external_c_542_cram_block_set_size(b, 23);
            assert_eq!(block.byte, 23);
            assert_eq!(cram_cram_external_c_554_cram_block_get_offset(b), 23);
            cram_cram_external_c_555_cram_block_set_offset(b, 8);
            assert_eq!(block.byte, 8);

            cram_cram_external_c_537_cram_block_set_content_id(b, 42);
            cram_cram_external_c_538_cram_block_set_comp_size(b, 6);
            cram_cram_external_c_539_cram_block_set_uncomp_size(b, 7);
            cram_cram_external_c_540_cram_block_set_crc32(b, -1);
            assert_eq!(block.content_id, 42);
            assert_eq!((block.comp_size, block.uncomp_size), (6, 7));
            assert_eq!(block.crc32, u32::MAX);

            cram_cram_external_c_551_cram_block_update_size(b);
            assert_eq!((block.comp_size, block.uncomp_size), (8, 8));

            let mut payload = *b"abc";
            cram_cram_external_c_541_cram_block_set_data(b, payload.as_mut_ptr().cast());
            assert_eq!(
                cram_cram_external_c_528_cram_block_get_data(b),
                payload.as_mut_ptr().cast()
            );

            block.content_type = crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE;
            assert_eq!(
                block.content_type,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE
            );
            assert_eq!(cram_cram_external_c_522_cram_block_get_content_id(b), -1);
        }
    }

    #[test]
    fn cram_external_expand_method_detects_methods_and_flags_like_c() {
        unsafe {
            let mut gzip_fast = [0u8; 10];
            gzip_fast[0] = 0x1f;
            gzip_fast[1] = 0x8b;
            gzip_fast[8] = 4;
            let cm = cram_cram_external_c_568_cram_expand_method(gzip_fast.as_mut_ptr(), 10, -1);
            assert_eq!((*cm).method, 1);
            assert_eq!((*cm).level, 1);
            free(cm.cast());

            let mut gzip_best = [0u8; 10];
            gzip_best[8] = 2;
            let cm = cram_cram_external_c_568_cram_expand_method(gzip_best.as_mut_ptr(), 10, 1);
            assert_eq!((*cm).level, 9);
            free(cm.cast());

            let mut bzip = *b"0BZh7";
            let cm = cram_cram_external_c_568_cram_expand_method(bzip.as_mut_ptr(), 5, -1);
            assert_eq!((*cm).method, 2);
            assert_eq!((*cm).level, 0);
            free(cm.cast());

            let mut bzip_level = [0, 0, 0, b'7'];
            let cm = cram_cram_external_c_568_cram_expand_method(bzip_level.as_mut_ptr(), 4, 2);
            assert_eq!((*cm).level, 7);
            free(cm.cast());

            let mut xz = [0xfd, b'7', b'z', b'X', b'Z', 0, 1];
            let cm = cram_cram_external_c_568_cram_expand_method(xz.as_mut_ptr(), 7, -1);
            assert_eq!((*cm).method, 3);
            free(cm.cast());

            let mut rans4 = [1u8];
            let cm = cram_cram_external_c_568_cram_expand_method(rans4.as_mut_ptr(), 1, 4);
            assert_eq!(((*cm).nway, (*cm).order), (4, 1));
            free(cm.cast());

            let mut rans16 = [0x04 | 0x08 | 0x10 | 0x20 | 0x40 | 0x80 | 1];
            let cm = cram_cram_external_c_568_cram_expand_method(rans16.as_mut_ptr(), 1, 5);
            assert_eq!((*cm).method, 5);
            assert_eq!(
                (
                    (*cm).order,
                    (*cm).nway,
                    (*cm).rle,
                    (*cm).pack,
                    (*cm).cat,
                    (*cm).stripe,
                    (*cm).nosz
                ),
                (1, 32, 1, 1, 1, 1, 1)
            );
            free(cm.cast());

            let mut arith = [0x04 | 0x40 | 0x80 | 0x20 | 0x08 | 0x10 | 2];
            let cm = cram_cram_external_c_568_cram_expand_method(arith.as_mut_ptr(), 1, 6);
            assert_eq!(
                (
                    (*cm).order,
                    (*cm).rle,
                    (*cm).pack,
                    (*cm).cat,
                    (*cm).stripe,
                    (*cm).nosz,
                    (*cm).ext
                ),
                (2, 1, 1, 1, 1, 1, 1)
            );
            free(cm.cast());

            let mut tok3 = [0u8; 9];
            tok3[8] = 1;
            let cm = cram_cram_external_c_568_cram_expand_method(tok3.as_mut_ptr(), 9, 8);
            assert_eq!((*cm).level, 11);
            free(cm.cast());
        }
    }

    #[test]
    fn cram_external_get_refs_returns_only_for_cram_htsfile() {
        unsafe {
            let refs = test_refs_marker();
            let mut cram_fd = cram_fd_layout {
                refs,
                ..std::mem::zeroed()
            };
            let mut fp: htsFile = std::mem::zeroed();
            fp.fp = crate::htslib_rs::hts::htsFilePtr {
                cram: (&mut cram_fd as *mut cram_fd_layout).cast(),
            };
            fp.format.format = HTS_FORMAT_CRAM;
            assert_eq!(
                cram_cram_external_c_1029_cram_get_refs(&mut fp),
                refs.cast()
            );

            fp.format.format = crate::htslib_rs::hts::HTS_FORMAT_BAM;
            assert!(cram_cram_external_c_1029_cram_get_refs(&mut fp).is_null());
        }
    }

    #[test]
    fn cram_external_codec_header_setters_and_iterator_match_c_rules() {
        unsafe {
            assert_eq!(
                cram_cram_external_c_224_cram_ds_to_key(17),
                256 * b'R' as c_int + b'G' as c_int
            );
            assert_eq!(cram_cram_external_c_224_cram_ds_to_key(42), -1);

            let mut hdr: cram_block_compression_hdr_layout = std::mem::zeroed();
            let mut code = cram_huffman_code_layout {
                symbol: 3,
                p: 0,
                code: 0,
                len: 0,
            };
            let mut huff = cram_codec_huffman_layout {
                codec: 3,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                huffman: cram_huffman_decoder_layout {
                    ncodes: 1,
                    codes: &mut code,
                    option: 0,
                },
            };
            hdr.codecs[17] = (&mut huff as *mut cram_codec_huffman_layout).cast();
            assert_eq!(
                cram_cram_external_c_177_cram_block_compression_hdr_set_rg(
                    (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                    12
                ),
                0
            );
            assert_eq!(code.symbol, 12);

            let mut beta = cram_codec_beta_layout {
                codec: 6,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                beta: cram_beta_decoder_layout {
                    offset: 0,
                    nbits: 0,
                },
            };
            hdr.codecs[17] = (&mut beta as *mut cram_codec_beta_layout).cast();
            assert_eq!(
                cram_cram_external_c_152_cram_block_compression_hdr_set_DS(
                    (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                    17,
                    23
                ),
                0
            );
            assert_eq!(beta.beta.offset, -23);
            beta.beta.nbits = 2;
            assert_eq!(
                cram_cram_external_c_177_cram_block_compression_hdr_set_rg(
                    (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                    1
                ),
                -1
            );
            assert_eq!(beta.beta.nbits, 2);

            hdr.codecs = [std::ptr::null_mut(); 47];
            let mut ds_codec = cram_codec_external_layout {
                codec: 1,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                external: cram_external_decoder_layout {
                    content_id: 9,
                    type_: 0,
                },
            };
            let mut tag_codec = cram_codec_external_layout {
                codec: 1,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                external: cram_external_decoder_layout {
                    content_id: 11,
                    type_: 0,
                },
            };
            hdr.codecs[10] = (&mut ds_codec as *mut cram_codec_external_layout).cast();
            let mut map = cram_map_layout {
                key: 0x4142,
                encoding: 0,
                offset: 0,
                size: 0,
                codec: (&mut tag_codec as *mut cram_codec_external_layout).cast(),
                next: std::ptr::null_mut(),
            };
            hdr.tag_encoding_map[0] = (&mut map as *mut cram_map_layout).cast();

            let mut iter: cram_codec_iter_layout = std::mem::zeroed();
            cram_cram_external_c_215_cram_codec_iter_init(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                (&mut iter as *mut cram_codec_iter_layout).cast(),
            );
            let mut key = 0;
            assert_eq!(
                cram_cram_external_c_264_cram_codec_iter_next(
                    (&mut iter as *mut cram_codec_iter_layout).cast(),
                    &mut key,
                ),
                (&mut ds_codec as *mut cram_codec_external_layout).cast()
            );
            assert_eq!(key, 256 * b'R' as c_int + b'N' as c_int);
            assert_eq!(
                cram_cram_external_c_264_cram_codec_iter_next(
                    (&mut iter as *mut cram_codec_iter_layout).cast(),
                    &mut key,
                ),
                (&mut tag_codec as *mut cram_codec_external_layout).cast()
            );
            assert_eq!(key, 0x4142);
            assert!(cram_cram_external_c_264_cram_codec_iter_next(
                (&mut iter as *mut cram_codec_iter_layout).cast(),
                &mut key,
            )
            .is_null());
        }
    }

    #[test]
    fn cram_io_block_lookup_uses_cache_hash_and_linear_collision_fallback() {
        unsafe {
            let mut hdr = cram_block_slice_hdr_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_MAPPED_SLICE,
                ref_seq_id: 0,
                ref_seq_start: 0,
                ref_seq_span: 0,
                num_records: 0,
                record_counter: 0,
                num_blocks: 3,
                num_content_ids: 0,
                block_content_ids: std::ptr::null_mut(),
                ref_base_id: 0,
                md5: [0; 16],
            };
            let mut direct = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 42,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut collision = cram_block_layout {
                content_id: 777,
                ..direct
            };
            let mut ignored_core = cram_block_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE,
                content_id: 888,
                ..direct
            };
            let direct_ptr = (&mut direct as *mut cram_block_layout).cast::<cram_block>();
            let collision_ptr = (&mut collision as *mut cram_block_layout).cast::<cram_block>();
            let ignored_ptr = (&mut ignored_core as *mut cram_block_layout).cast::<cram_block>();

            let mut by_id = vec![std::ptr::null_mut(); 512];
            by_id[42] = direct_ptr;
            by_id[256 + 777 % 251] = std::ptr::null_mut();
            let mut blocks = [ignored_ptr, collision_ptr, std::ptr::null_mut()];
            let mut slice = cram_slice_layout {
                hdr: &mut hdr,
                hdr_block: std::ptr::null_mut(),
                block: blocks.as_mut_ptr().cast(),
                block_by_id: by_id.as_mut_ptr().cast(),
                ..std::mem::zeroed()
            };

            assert_eq!(
                cram_cram_io_h_183_cram_get_block_by_id(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    42
                ),
                direct_ptr
            );
            assert_eq!(
                cram_cram_io_h_183_cram_get_block_by_id(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    777
                ),
                collision_ptr
            );
            assert!(cram_cram_io_h_183_cram_get_block_by_id(
                (&mut slice as *mut cram_slice_layout).cast(),
                888
            )
            .is_null());
        }
    }

    #[test]
    fn cram_io_append_integer_helpers_cover_zero_and_wide_values() {
        unsafe {
            let block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                3,
            );
            assert!(!block.is_null());

            assert_eq!(cram_cram_io_h_271_block_append_uint(block, 0), 0);
            assert_eq!(cram_cram_io_h_271_block_append_uint(block, u32::MAX), 0);

            let b = block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*b).data, (*b).byte),
                b"04294967295"
            );

            let mut out = [0u8; 32];
            let end = cram_cram_io_h_340_append_uint64(out.as_mut_ptr(), u64::MAX);
            let len = end.offset_from(out.as_mut_ptr()) as usize;
            assert_eq!(&out[..len], b"18446744073709551615");

            cram_cram_io_c_1565_cram_free_block(block);
        }
    }

    #[test]
    fn cram_external_cid2ds_map_tracks_shared_content_ids_like_c() {
        unsafe {
            let mut hdr: cram_block_compression_hdr_layout = std::mem::zeroed();
            let mut rn_codec = cram_codec_external_layout {
                codec: 1,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                external: cram_external_decoder_layout {
                    content_id: 7,
                    type_: 0,
                },
            };
            let mut qs_codec = rn_codec;
            qs_codec.external.content_id = 7;
            let mut mq_codec = rn_codec;
            mq_codec.external.content_id = 9;

            hdr.codecs[10] = (&mut rn_codec as *mut cram_codec_external_layout).cast();
            hdr.codecs[11] = (&mut qs_codec as *mut cram_codec_external_layout).cast();
            hdr.codecs[18] = (&mut mq_codec as *mut cram_codec_external_layout).cast();

            let c2d = cram_cram_external_c_342_cram_update_cid2ds_map(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                std::ptr::null_mut(),
            );
            assert!(!c2d.is_null());

            let mut n = -1;
            let ds = cram_cram_external_c_443_cram_cid2ds_query(c2d, 7, &mut n);
            assert_eq!(n, 2);
            assert_eq!(
                std::slice::from_raw_parts(ds, n as usize),
                &[
                    cram_cram_external_c_224_cram_ds_to_key(11),
                    cram_cram_external_c_224_cram_ds_to_key(10)
                ]
            );

            let ds = cram_cram_external_c_443_cram_cid2ds_query(c2d, 9, &mut n);
            assert_eq!(n, 1);
            assert_eq!(*ds, cram_cram_external_c_224_cram_ds_to_key(18));

            let missing = cram_cram_external_c_443_cram_cid2ds_query(c2d, 99, &mut n);
            assert!(missing.is_null());
            assert_eq!(n, 0);

            let c2d2 = cram_cram_external_c_342_cram_update_cid2ds_map(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                c2d,
            );
            assert_eq!(c2d2, c2d);
            let ds = cram_cram_external_c_443_cram_cid2ds_query(c2d, 7, &mut n);
            assert_eq!(n, 2);
            assert_eq!(
                std::slice::from_raw_parts(ds, n as usize),
                &[
                    cram_cram_external_c_224_cram_ds_to_key(11),
                    cram_cram_external_c_224_cram_ds_to_key(10)
                ]
            );

            cram_cram_external_c_320_cram_cid2ds_free(c2d);
            cram_cram_external_c_320_cram_cid2ds_free(std::ptr::null_mut());
        }
    }

    #[test]
    fn cram_io_block_append_and_decimal_helpers_match_c_layout() {
        unsafe {
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();

            assert_eq!(
                cram_cram_io_h_248_block_append(b, b"abc".as_ptr().cast(), 3),
                0
            );
            assert_eq!(cram_cram_io_h_261_block_append_char(b, b'/' as c_char), 0);
            assert_eq!(cram_cram_io_h_271_block_append_uint(b, 12345), 0);
            assert_eq!(
                std::slice::from_raw_parts(block.data, block.byte),
                b"abc/12345"
            );
            assert!(block.alloc >= block.byte);

            let mut buf = [0u8; 64];
            let end = cram_cram_io_h_288_append_uint32(buf.as_mut_ptr(), 0);
            assert_eq!(end.offset_from(buf.as_mut_ptr()), 1);
            assert_eq!(&buf[..1], b"0");

            let end = cram_cram_io_h_326_append_sub32(buf.as_mut_ptr(), 12);
            assert_eq!(end.offset_from(buf.as_mut_ptr()), 9);
            assert_eq!(&buf[..9], b"000000012");

            let end =
                cram_cram_io_h_340_append_uint64(buf.as_mut_ptr(), 18_446_744_073_709_551_615);
            let len = end.offset_from(buf.as_mut_ptr()) as usize;
            assert_eq!(&buf[..len], b"18446744073709551615");

            free(block.data.cast());
        }
    }

    #[test]
    fn cram_io_itf8_ltf8_helpers_match_c_encodings() {
        unsafe {
            let mut buf = [0u8; 16];
            assert_eq!(
                cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), 0x7f),
                1
            );
            assert_eq!(&buf[..1], &[0x7f]);
            assert_eq!(
                cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), 0x80),
                2
            );
            assert_eq!(&buf[..2], &[0x80, 0x80]);
            assert_eq!(
                cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), 0x4000),
                3
            );
            assert_eq!(&buf[..3], &[0xc0, 0x40, 0x00]);
            assert_eq!(
                cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), 0x20_0000),
                4
            );
            assert_eq!(&buf[..4], &[0xe0, 0x20, 0x00, 0x00]);
            assert_eq!(cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), -1), 5);
            assert_eq!(&buf[..5], &[0xff, 0xff, 0xff, 0xff, 0x0f]);

            assert_eq!(
                cram_cram_io_c_309_ltf8_put(buf.as_mut_ptr().cast(), 0x7f),
                1
            );
            assert_eq!(&buf[..1], &[0x7f]);
            assert_eq!(
                cram_cram_io_c_309_ltf8_put(buf.as_mut_ptr().cast(), 0x80),
                2
            );
            assert_eq!(&buf[..2], &[0x80, 0x80]);
            assert_eq!(
                cram_cram_io_c_309_ltf8_put(buf.as_mut_ptr().cast(), 1i64 << 32),
                5
            );
            assert_eq!(&buf[..5], &[0xf1, 0x00, 0x00, 0x00, 0x00]);
            assert_eq!(cram_cram_io_c_309_ltf8_put(buf.as_mut_ptr().cast(), -1), 9);
            assert_eq!(
                &buf[..9],
                &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
            );

            let mut safe = [0u8; 9];
            let len = cram_cram_io_c_747_safe_itf8_put(
                safe.as_mut_ptr().cast(),
                safe.as_mut_ptr().add(safe.len()).cast(),
                0x1f_ffff,
            );
            assert_eq!(len, 3);
            let mut cp = safe.as_mut_ptr().cast::<c_char>();
            let endp = safe.as_ptr().add(len as usize).cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_644_safe_itf8_get(&mut cp, endp, &mut err),
                0x1f_ffff
            );
            assert_eq!(err, 0);
            assert_eq!(cp, endp as *mut c_char);

            let mut truncated = [0x80u8];
            let mut cp = truncated.as_mut_ptr().cast::<c_char>();
            let endp = truncated.as_ptr().add(truncated.len()).cast::<c_char>();
            let mut err = 0;
            assert_eq!(cram_cram_io_c_644_safe_itf8_get(&mut cp, endp, &mut err), 0);
            assert_eq!(err, 1);
            assert_eq!(cp, truncated.as_mut_ptr().cast::<c_char>());

            let len = cram_cram_io_c_751_safe_ltf8_put(
                safe.as_mut_ptr().cast(),
                safe.as_mut_ptr().add(safe.len()).cast(),
                0x1_0000_0000,
            );
            assert_eq!(len, 5);
            let mut cp = safe.as_mut_ptr().cast::<c_char>();
            let endp = safe.as_ptr().add(len as usize).cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_673_safe_ltf8_get(&mut cp, endp, &mut err),
                0x1_0000_0000
            );
            assert_eq!(err, 0);
            assert_eq!(cp, endp as *mut c_char);

            let mut truncated = [0xf8u8, 0x01, 0x02];
            let mut cp = truncated.as_mut_ptr().cast::<c_char>();
            let endp = truncated.as_ptr().add(truncated.len()).cast::<c_char>();
            let mut err = 0;
            assert_eq!(cram_cram_io_c_673_safe_ltf8_get(&mut cp, endp, &mut err), 0);
            assert_eq!(err, 1);
            assert_eq!(cp, truncated.as_mut_ptr().cast::<c_char>());

            assert_eq!(cram_cram_io_c_755_itf8_size(0x7f), 1);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x80), 2);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x3fff), 2);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x4000), 3);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x1f_ffff), 3);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x20_0000), 4);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x0fff_ffff), 4);
            assert_eq!(cram_cram_io_c_755_itf8_size(0x1000_0000), 5);

            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!b.is_null());
            assert_eq!(cram_cram_io_c_620_itf8_put_blk(b, 0x4000), 3);
            assert_eq!(cram_cram_io_c_632_ltf8_put_blk(b, 0x80), 2);
            let block = b.cast::<cram_block_layout>();
            assert_eq!((*block).byte, 5);
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).byte),
                &[0xc0, 0x40, 0x00, 0x80, 0x80]
            );
            cram_cram_io_c_1565_cram_free_block(b);

            let path = std::env::temp_dir().join(format!(
                "htslib_rs-cram-varint-stream-{}",
                std::process::id()
            ));

            let mut bytes = Vec::new();
            for val in [0x7f, 0x80, 0x4000, 0x20_0000, -1] {
                let len = cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), val);
                bytes.extend_from_slice(&buf[..len as usize]);
            }
            for val in [0x7fi64, 0x80, 1 << 20, 1 << 32, 1 << 48, -1] {
                let len = cram_cram_io_c_309_ltf8_put(buf.as_mut_ptr().cast(), val);
                bytes.extend_from_slice(&buf[..len as usize]);
            }
            std::fs::write(&path, &bytes).unwrap();

            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = hopen(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = fp;

            for (expected_len, expected_val) in
                [(1, 0x7f), (2, 0x80), (3, 0x4000), (4, 0x20_0000), (5, -1)]
            {
                let mut val = 0;
                assert_eq!(
                    cram_cram_io_c_138_itf8_decode(
                        (&mut fd as *mut cram_fd_layout).cast(),
                        &mut val
                    ),
                    expected_len
                );
                assert_eq!(val, expected_val);
            }
            for (expected_len, expected_val) in [
                (1, 0x7fi64),
                (2, 0x80),
                (3, 1 << 20),
                (5, 1 << 32),
                (7, 1 << 48),
                (9, -1),
            ] {
                let mut val = 0;
                assert_eq!(
                    cram_cram_io_c_420_ltf8_decode(
                        (&mut fd as *mut cram_fd_layout).cast(),
                        &mut val
                    ),
                    expected_len
                );
                assert_eq!(val, expected_val);
            }
            assert_eq!(hclose(fp), 0);

            let fp = hopen(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            fd.fp = fp;
            let mut crc = 0u32;
            let mut offset = 0usize;
            for (expected_len, expected_val) in
                [(1, 0x7f), (2, 0x80), (3, 0x4000), (4, 0x20_0000), (5, -1)]
            {
                let mut val = 0;
                let before = crc;
                assert_eq!(
                    cram_cram_io_c_196_itf8_decode_crc(
                        (&mut fd as *mut cram_fd_layout).cast(),
                        &mut val,
                        &mut crc
                    ),
                    expected_len
                );
                assert_eq!(val, expected_val);
                assert_eq!(
                    crc,
                    crate::htslib_rs::bgzf::hts_crc32(
                        before,
                        bytes.as_ptr().add(offset).cast(),
                        expected_len as usize,
                    )
                );
                offset += expected_len as usize;
            }
            for (expected_len, expected_val) in [
                (1, 0x7fi64),
                (2, 0x80),
                (3, 1 << 20),
                (5, 1 << 32),
                (7, 1 << 48),
                (9, -1),
            ] {
                let mut val = 0;
                let before = crc;
                assert_eq!(
                    cram_cram_io_c_501_ltf8_decode_crc(
                        (&mut fd as *mut cram_fd_layout).cast(),
                        &mut val,
                        &mut crc
                    ),
                    expected_len
                );
                assert_eq!(val, expected_val);
                assert_eq!(
                    crc,
                    crate::htslib_rs::bgzf::hts_crc32(
                        before,
                        bytes.as_ptr().add(offset).cast(),
                        expected_len as usize,
                    )
                );
                offset += expected_len as usize;
            }
            assert_eq!(offset, bytes.len());
            assert_eq!(hclose(fp), 0);

            let fp = hopen(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fp.is_null());
            fd.fp = fp;
            let mut expected = Vec::new();
            for val in [0x7f, 0x80, 0x4000, 0x20_0000, -1] {
                let len = cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), val);
                expected.extend_from_slice(&buf[..len as usize]);
                assert_eq!(
                    cram_cram_io_c_382_itf8_encode((&mut fd as *mut cram_fd_layout).cast(), val),
                    0
                );
            }
            assert_eq!(hclose(fp), 0);
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn cram_io_uint7_sint7_helpers_match_htscodecs_varint_layout() {
        unsafe {
            let mut buf = [0u8; 16];
            assert_eq!(cram_cram_io_c_768_uint7_size(0x7f), 1);
            assert_eq!(cram_cram_io_c_768_uint7_size(0x80), 2);
            assert_eq!(cram_cram_io_c_768_uint7_size(0x3fff), 2);
            assert_eq!(cram_cram_io_c_768_uint7_size(0x4000), 3);
            assert_eq!(cram_cram_io_c_768_uint7_size(i64::MAX), 9);
            assert_eq!(cram_cram_io_c_768_uint7_size(-1), 10);

            assert_eq!(
                cram_cram_io_c_804_uint7_put_32(
                    buf.as_mut_ptr().cast(),
                    std::ptr::null_mut(),
                    0x7f
                ),
                1
            );
            assert_eq!(&buf[..1], &[0x7f]);
            assert_eq!(
                cram_cram_io_c_804_uint7_put_32(
                    buf.as_mut_ptr().cast(),
                    std::ptr::null_mut(),
                    0x80
                ),
                2
            );
            assert_eq!(&buf[..2], &[0x81, 0x00]);
            assert_eq!(
                cram_cram_io_c_804_uint7_put_32(
                    buf.as_mut_ptr().cast(),
                    std::ptr::null_mut(),
                    0x4000,
                ),
                3
            );
            assert_eq!(&buf[..3], &[0x81, 0x80, 0x00]);

            let mut cp = buf.as_mut_ptr().cast::<c_char>();
            let endp = buf.as_ptr().add(3).cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_772_uint7_get_32(&mut cp, endp, &mut err),
                0x4000
            );
            assert_eq!(err, 0);
            assert_eq!(cp, endp as *mut c_char);

            assert_eq!(
                cram_cram_io_c_808_sint7_put_32(buf.as_mut_ptr().cast(), std::ptr::null_mut(), -1),
                1
            );
            assert_eq!(&buf[..1], &[0x01]);
            let mut cp = buf.as_mut_ptr().cast::<c_char>();
            let endp = buf.as_ptr().add(1).cast::<c_char>();
            let mut err = 0;
            assert_eq!(cram_cram_io_c_780_sint7_get_32(&mut cp, endp, &mut err), -1);
            assert_eq!(err, 0);

            assert_eq!(
                cram_cram_io_c_812_uint7_put_64(
                    buf.as_mut_ptr().cast(),
                    std::ptr::null_mut(),
                    1i64 << 35,
                ),
                6
            );
            assert_eq!(&buf[..6], &[0x81, 0x80, 0x80, 0x80, 0x80, 0x00]);
            let mut cp = buf.as_mut_ptr().cast::<c_char>();
            let endp = buf.as_ptr().add(6).cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_788_uint7_get_64(&mut cp, endp, &mut err),
                1i64 << 35
            );
            assert_eq!(err, 0);

            assert_eq!(
                cram_cram_io_c_816_sint7_put_64(buf.as_mut_ptr().cast(), std::ptr::null_mut(), -65),
                2
            );
            assert_eq!(&buf[..2], &[0x81, 0x01]);
            let mut cp = buf.as_mut_ptr().cast::<c_char>();
            let endp = buf.as_ptr().add(2).cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_796_sint7_get_64(&mut cp, endp, &mut err),
                -65
            );
            assert_eq!(err, 0);

            let mut short = [0u8; 1];
            assert_eq!(
                cram_cram_io_c_804_uint7_put_32(
                    short.as_mut_ptr().cast(),
                    short.as_mut_ptr().add(short.len()).cast(),
                    0x80,
                ),
                0
            );
            let mut truncated = [0x81u8];
            let mut cp = truncated.as_mut_ptr().cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_772_uint7_get_32(
                    &mut cp,
                    truncated.as_ptr().add(truncated.len()).cast(),
                    &mut err,
                ),
                1
            );
            assert_eq!(err, 0);
            assert_eq!(cp, truncated.as_mut_ptr().add(1).cast::<c_char>());

            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!b.is_null());
            assert_eq!(cram_cram_io_c_821_uint7_put_blk_32(b, 0x80), 2);
            assert_eq!(cram_cram_io_c_831_sint7_put_blk_32(b, -1), 1);
            assert_eq!(cram_cram_io_c_841_uint7_put_blk_64(b, 0x4000), 3);
            assert_eq!(cram_cram_io_c_851_sint7_put_blk_64(b, -65), 2);
            let block = b.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).byte),
                &[0x81, 0x00, 0x01, 0x81, 0x80, 0x00, 0x81, 0x01]
            );
            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_crc_varint_and_int32_helpers_match_c_layout() {
        unsafe {
            let mut input = [0x81u8, 0x80, 0x00, 0x01, 0x81, 0x80, 0x80, 0x80, 0x00];
            let mut hfile = hfile_layout {
                buffer: input.as_mut_ptr().cast(),
                begin: input.as_mut_ptr().cast(),
                end: input.as_mut_ptr().add(input.len()).cast(),
                limit: input.as_mut_ptr().add(input.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();

            let mut crc = 0u32;
            let mut val32 = 0i32;
            assert_eq!(
                cram_cram_io_c_862_uint7_decode_crc32(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    &mut val32,
                    &mut crc,
                ),
                3
            );
            assert_eq!(val32, 0x4000);
            assert_eq!(
                crc,
                crate::htslib_rs::bgzf::hts_crc32(0, input.as_ptr().cast(), 3)
            );

            let mut signed = 0i32;
            let before = crc;
            assert_eq!(
                cram_cram_io_c_907_sint7_decode_crc32(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    &mut signed,
                    &mut crc,
                ),
                1
            );
            assert_eq!(signed, -1);
            assert_eq!(
                crc,
                crate::htslib_rs::bgzf::hts_crc32(before, input.as_ptr().add(3).cast(), 1)
            );

            let mut val64 = 0i64;
            let before = crc;
            assert_eq!(
                cram_cram_io_c_953_uint7_decode_crc64(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    &mut val64,
                    &mut crc,
                ),
                5
            );
            assert_eq!(val64, 1i64 << 28);
            assert_eq!(
                crc,
                crate::htslib_rs::bgzf::hts_crc32(before, input.as_ptr().add(4).cast(), 5)
            );
            assert_eq!(hfile.begin, hfile.end);

            let mut int_bytes = [0x78u8, 0x56, 0x34, 0x12];
            hfile.buffer = int_bytes.as_mut_ptr().cast();
            hfile.begin = int_bytes.as_mut_ptr().cast();
            hfile.end = int_bytes.as_mut_ptr().add(int_bytes.len()).cast();
            hfile.limit = hfile.end;
            let mut int_val = 0i32;
            assert_eq!(
                cram_cram_io_c_1005_int32_decode(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    &mut int_val
                ),
                4
            );
            assert_eq!(int_val, 0x1234_5678);
            assert_eq!(hfile.begin, hfile.end);

            let mut out = [0u8; 8];
            hfile.buffer = out.as_mut_ptr().cast();
            hfile.begin = out.as_mut_ptr().cast();
            hfile.end = out.as_mut_ptr().cast();
            hfile.limit = out.as_mut_ptr().add(out.len()).cast();
            assert_eq!(
                cram_cram_io_c_1020_int32_encode((&mut fd as *mut cram_fd_layout).cast(), -2i32,),
                4
            );
            assert_eq!(&out[..4], &[0xfe, 0xff, 0xff, 0xff]);
            assert_eq!(hfile.begin, out.as_mut_ptr().add(4).cast());

            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!b.is_null());
            assert_eq!(cram_cram_io_c_1045_int32_put_blk(b, -2), 0);
            let block = b.cast::<cram_block_layout>();
            assert_eq!((*block).byte, 4);
            assert_eq!(
                std::slice::from_raw_parts((*block).data, 4),
                &[0xfe, 0xff, 0xff, 0xff]
            );
            (*block).uncomp_size = 4;
            (*block).byte = 0;
            let mut block_val = 0i32;
            assert_eq!(cram_cram_io_c_1029_int32_get_blk(b, &mut block_val), 4);
            assert_eq!(block_val, -2);
            assert_eq!((*block).byte, 4);
            assert_eq!(cram_cram_io_c_1029_int32_get_blk(b, &mut block_val), -1);
            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn public_int32_get_blk_uses_block_size_cursor() {
        unsafe {
            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            let mut bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
            cram_cram_io_h_248_block_append(b, bytes.as_mut_ptr().cast(), bytes.len());
            (*block).uncomp_size = 8;
            (*block).byte = 4;
            (*block).idx = 0;

            let mut val = 0;
            assert_eq!(int32_get_blk(b.cast(), &mut val), 4);
            assert_eq!(val, 0x0807_0605);
            assert_eq!((*block).byte, 8);
            assert_eq!((*block).idx, 0);
            assert_eq!(int32_get_blk(b.cast(), &mut val), -1);
            assert_eq!((*block).byte, 8);

            (*block).uncomp_size = -1;
            (*block).byte = 0;
            val = 0x1122_3344;
            assert_eq!(int32_get_blk(b.cast(), &mut val), -1);
            assert_eq!(val, 0x1122_3344);
            assert_eq!((*block).byte, 0);

            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_init_varint_selects_versioned_function_tables() {
        unsafe {
            let mut vv4: varint_vec_layout = std::mem::zeroed();
            cram_cram_io_c_5127_cram_init_varint((&mut vv4 as *mut varint_vec_layout).cast(), 4);
            assert!(vv4.varint_get32.is_some());
            assert!(vv4.varint_get32s.is_some());
            assert!(vv4.varint_get64.is_some());
            assert!(vv4.varint_get64s.is_some());
            assert!(vv4.varint_put32.is_some());
            assert!(vv4.varint_put32s.is_some());
            assert!(vv4.varint_put64.is_some());
            assert!(vv4.varint_put64s.is_some());
            assert!(vv4.varint_put32_blk.is_some());
            assert!(vv4.varint_put32s_blk.is_some());
            assert!(vv4.varint_put64_blk.is_some());
            assert!(vv4.varint_put64s_blk.is_some());
            assert!(vv4.varint_size.is_some());
            assert!(!vv4.varint_decode32_crc.is_null());
            assert!(!vv4.varint_decode32s_crc.is_null());
            assert!(!vv4.varint_decode64_crc.is_null());

            let mut vv3: varint_vec_layout = std::mem::zeroed();
            cram_cram_io_c_5127_cram_init_varint((&mut vv3 as *mut varint_vec_layout).cast(), 3);
            assert!(vv3.varint_get32.is_some());
            assert!(vv3.varint_get32s.is_some());
            assert!(vv3.varint_get64.is_some());
            assert!(vv3.varint_get64s.is_some());
            assert!(vv3.varint_put32.is_some());
            assert!(vv3.varint_put32s.is_some());
            assert!(vv3.varint_put64.is_some());
            assert!(vv3.varint_put64s.is_some());
            assert!(vv3.varint_put32_blk.is_some());
            assert!(vv3.varint_put32s_blk.is_some());
            assert!(vv3.varint_put64_blk.is_some());
            assert!(vv3.varint_put64s_blk.is_some());
            assert!(vv3.varint_size.is_some());
            assert!(!vv3.varint_decode32_crc.is_null());
            assert!(!vv3.varint_decode32s_crc.is_null());
            assert!(!vv3.varint_decode64_crc.is_null());

            let mut buf = [0 as c_char; 8];
            assert_eq!(
                (vv4.varint_put32.unwrap())(buf.as_mut_ptr(), buf.as_mut_ptr().add(8), 128),
                2
            );
            let mut cp = buf.as_mut_ptr();
            let mut err = 0;
            assert_eq!(
                (vv4.varint_get32.unwrap())(&mut cp, buf.as_ptr().add(2), &mut err),
                128
            );
            assert_eq!(err, 0);

            let mut buf = [0 as c_char; 8];
            assert_eq!(
                (vv3.varint_put32.unwrap())(buf.as_mut_ptr(), buf.as_mut_ptr().add(8), 128),
                2
            );
            let mut cp = buf.as_mut_ptr();
            let mut err = 0;
            assert_eq!(
                (vv3.varint_get32.unwrap())(&mut cp, buf.as_ptr().add(2), &mut err),
                128
            );
            assert_eq!(err, 0);
        }
    }

    #[test]
    fn cram_io_uint7_getters_consume_overlong_and_short_buffers_like_htscodecs() {
        unsafe {
            let mut over32 = [0xffu8; 6];
            let mut cp = over32.as_mut_ptr().cast::<c_char>();
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_772_uint7_get_32(&mut cp, std::ptr::null(), &mut err),
                0xffff_ffff
            );
            assert_eq!(cp, over32.as_mut_ptr().add(6).cast::<c_char>());
            assert_eq!(err, 0);

            cp = over32.as_mut_ptr().cast::<c_char>();
            assert_eq!(
                cram_cram_io_c_772_uint7_get_32(&mut cp, over32.as_ptr().add(5).cast(), &mut err,),
                0xffff_ffff
            );
            assert_eq!(cp, over32.as_mut_ptr().add(5).cast::<c_char>());
            assert_eq!(err, 0);

            cp = over32.as_mut_ptr().cast::<c_char>();
            assert_eq!(
                cram_cram_io_c_772_uint7_get_32(&mut cp, over32.as_ptr().cast(), &mut err),
                0
            );
            assert_eq!(cp, over32.as_mut_ptr().cast::<c_char>());
            assert_eq!(err, 1);

            let mut over64 = [0xffu8; 11];
            let mut cp64 = over64.as_mut_ptr().cast::<c_char>();
            err = 0;
            assert_eq!(
                cram_cram_io_c_788_uint7_get_64(&mut cp64, std::ptr::null(), &mut err),
                -1
            );
            assert_eq!(cp64, over64.as_mut_ptr().add(11).cast::<c_char>());
            assert_eq!(err, 0);

            cp64 = over64.as_mut_ptr().cast::<c_char>();
            assert_eq!(
                cram_cram_io_c_788_uint7_get_64(
                    &mut cp64,
                    over64.as_ptr().add(10).cast(),
                    &mut err,
                ),
                -1
            );
            assert_eq!(cp64, over64.as_mut_ptr().add(10).cast::<c_char>());
            assert_eq!(err, 0);
        }
    }

    #[test]
    fn cram_io_init_tables_fills_lookup_flags_substitution_and_varints() {
        unsafe {
            let mut fd4: cram_fd_layout = std::mem::zeroed();
            fd4.version = 4 << 8;
            cram_cram_io_c_5170_cram_init_tables((&mut fd4 as *mut cram_fd_layout).cast());

            assert_eq!(fd4.l1[b'A' as usize], 0);
            assert_eq!(fd4.l1[b'c' as usize], 1);
            assert_eq!(fd4.l1[b'G' as usize], 2);
            assert_eq!(fd4.l1[b't' as usize], 3);
            assert_eq!(fd4.l1[b'N' as usize], 4);
            assert_eq!(fd4.l2[b'N' as usize], 4);
            assert_eq!(fd4.l2[b'x' as usize], 5);
            assert_eq!(fd4.bam_flag_swap[0x7ab], 0x7ab);
            assert_eq!(fd4.cram_flag_swap[0xabc], 0xabc);
            assert!(fd4.vv.varint_get32.is_some());

            let a_row = (b'A' & 0x1f) as usize;
            assert_eq!(fd4.cram_sub_matrix[a_row][(b'C' & 0x1f) as usize], 0);
            assert_eq!(fd4.cram_sub_matrix[a_row][(b'G' & 0x1f) as usize], 1);
            assert_eq!(fd4.cram_sub_matrix[a_row][(b'T' & 0x1f) as usize], 2);
            assert_eq!(fd4.cram_sub_matrix[a_row][(b'N' & 0x1f) as usize], 3);
            assert_eq!(fd4.cram_sub_matrix[a_row][31], 4);

            let mut fd1: cram_fd_layout = std::mem::zeroed();
            fd1.version = 1 << 8;
            cram_cram_io_c_5170_cram_init_tables((&mut fd1 as *mut cram_fd_layout).cast());
            assert_eq!(
                fd1.bam_flag_swap[(CRAM_FPAIRED | CRAM_FREVERSE | CRAM_FDUP) as usize] as c_int,
                BAM_FPAIRED | BAM_FREVERSE | BAM_FDUP
            );
            assert_eq!(
                fd1.cram_flag_swap[(BAM_FPAIRED | BAM_FREAD1 | BAM_FQCFAIL) as usize] as c_int,
                CRAM_FPAIRED | CRAM_FREAD1 | CRAM_FQCFAIL
            );
            assert_eq!(
                fd1.vv.varint_get32.unwrap() as usize,
                cram_cram_io_c_644_safe_itf8_get as usize
            );
        }
    }

    #[test]
    fn cram_io_zlib_mem_inflate_returns_malloc_owned_uncompressed_bytes() {
        unsafe {
            let payload = b"cram gzip memory inflate payload ".repeat(12);
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
            std::io::Write::write_all(&mut encoder, &payload).unwrap();
            let mut compressed = encoder.finish().unwrap();

            let mut size = 0usize;
            let out = cram_cram_io_c_1157_zlib_mem_inflate(
                compressed.as_mut_ptr().cast(),
                compressed.len(),
                &mut size,
            );
            assert!(!out.is_null());
            assert_eq!(size, payload.len());
            assert_eq!(std::slice::from_raw_parts(out.cast::<u8>(), size), payload);
            free(out.cast());

            let mut hinted_size = 1usize;
            let out = cram_cram_io_c_1068_zlib_mem_inflate(
                compressed.as_mut_ptr().cast(),
                compressed.len(),
                &mut hinted_size,
            );
            assert!(!out.is_null());
            assert_eq!(hinted_size, payload.len());
            assert_eq!(
                std::slice::from_raw_parts(out.cast::<u8>(), hinted_size),
                payload
            );
            free(out.cast());

            let mut bad = [0u8, 1, 2, 3];
            let out =
                cram_cram_io_c_1157_zlib_mem_inflate(bad.as_mut_ptr().cast(), bad.len(), &mut size);
            assert!(out.is_null());
        }
    }

    #[test]
    fn cram_io_file_def_reads_writes_and_rejects_invalid_headers() {
        unsafe {
            let mut header = [0u8; 26];
            header[..4].copy_from_slice(b"CRAM");
            header[4] = 4;
            header[5] = 1;
            header[6..].copy_from_slice(b"abcdefghijklmnopqrst");

            let mut hfile = hfile_layout {
                buffer: header.as_mut_ptr().cast(),
                begin: header.as_mut_ptr().cast(),
                end: header.as_mut_ptr().add(header.len()).cast(),
                limit: header.as_mut_ptr().add(header.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.first_container = 11;
            fd.curr_position = 5;
            fd.last_slice = 99;

            let def =
                cram_cram_io_c_4660_cram_read_file_def((&mut fd as *mut cram_fd_layout).cast());
            assert!(!def.is_null());
            assert_eq!(
                std::slice::from_raw_parts((*def).magic.as_ptr().cast::<u8>(), 4),
                b"CRAM"
            );
            assert_eq!((*def).major_version, 4);
            assert_eq!((*def).minor_version, 1);
            assert_eq!(
                std::slice::from_raw_parts((*def).file_id.as_ptr().cast::<u8>(), 20),
                b"abcdefghijklmnopqrst"
            );
            assert_eq!(fd.first_container, 37);
            assert_eq!(fd.curr_position, 37);
            assert_eq!(fd.last_slice, 0);
            cram_cram_io_c_4698_cram_free_file_def(def);

            let mut out = [0u8; 32];
            let mut out_hfile = hfile_layout {
                buffer: out.as_mut_ptr().cast(),
                begin: out.as_mut_ptr().cast(),
                end: out.as_mut_ptr().cast(),
                limit: out.as_mut_ptr().add(out.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            fd.fp = (&mut out_hfile as *mut hfile_layout).cast();
            let mut write_def = cram_file_def_layout {
                magic: [
                    b'C' as c_char,
                    b'R' as c_char,
                    b'A' as c_char,
                    b'M' as c_char,
                ],
                major_version: 3,
                minor_version: 0,
                file_id: [0; 20],
            };
            write_def.file_id[..4].copy_from_slice(&[
                b't' as c_char,
                b'e' as c_char,
                b's' as c_char,
                b't' as c_char,
            ]);
            assert_eq!(
                cram_cram_io_c_4694_cram_write_file_def(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    &mut write_def,
                ),
                0
            );
            assert_eq!(&out[..6], b"CRAM\x03\0");
            assert_eq!(&out[6..10], b"test");
            assert_eq!(out_hfile.begin, out.as_mut_ptr().add(26).cast());

            let mut bad_magic = header;
            bad_magic[0] = b'X';
            hfile.buffer = bad_magic.as_mut_ptr().cast();
            hfile.begin = bad_magic.as_mut_ptr().cast();
            hfile.end = bad_magic.as_mut_ptr().add(bad_magic.len()).cast();
            hfile.limit = hfile.end;
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            assert!(cram_cram_io_c_4660_cram_read_file_def(
                (&mut fd as *mut cram_fd_layout).cast()
            )
            .is_null());

            let mut bad_version = header;
            bad_version[4] = 5;
            hfile.buffer = bad_version.as_mut_ptr().cast();
            hfile.begin = bad_version.as_mut_ptr().cast();
            hfile.end = bad_version.as_mut_ptr().add(bad_version.len()).cast();
            hfile.limit = hfile.end;
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            assert!(cram_cram_io_c_4660_cram_read_file_def(
                (&mut fd as *mut cram_fd_layout).cast()
            )
            .is_null());

            hfile.begin = header.as_mut_ptr().cast();
            hfile.end = header.as_mut_ptr().add(25).cast();
            hfile.limit = header.as_mut_ptr().add(header.len()).cast();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            assert!(cram_cram_io_c_4660_cram_read_file_def(
                (&mut fd as *mut cram_fd_layout).cast()
            )
            .is_null());

            cram_cram_io_c_4698_cram_free_file_def(std::ptr::null_mut());
        }
    }

    #[test]
    fn cram_io_read_write_raw_block_round_trips_v4_layout() {
        unsafe {
            let mut hfile_buf = [0u8; 64];
            let mut hfile = hfile_layout {
                buffer: hfile_buf.as_mut_ptr().cast(),
                begin: hfile_buf.as_mut_ptr().cast(),
                end: hfile_buf.as_mut_ptr().cast(),
                limit: hfile_buf.as_mut_ptr().add(hfile_buf.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.version = 4 << 8;

            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                7,
            );
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            (*block).orig_method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            assert_eq!(
                cram_cram_io_h_248_block_append(b, b"abc".as_ptr().cast(), 3),
                0
            );
            (*block).comp_size = 3;
            (*block).uncomp_size = 3;

            assert_eq!(
                cram_cram_io_c_1511_cram_write_block((&mut fd as *mut cram_fd_layout).cast(), b),
                0
            );
            let written = hfile.begin.offset_from(hfile_buf.as_mut_ptr().cast()) as usize;
            assert_eq!(&hfile_buf[..8], &[0, 4, 7, 3, 3, b'a', b'b', b'c']);
            assert_eq!(written, 12);
            let expected_crc = crate::htslib_rs::bgzf::hts_crc32(0, hfile_buf.as_ptr().cast(), 8);
            assert_eq!(
                u32::from_le_bytes(hfile_buf[8..12].try_into().unwrap()),
                expected_crc
            );

            hfile.begin = hfile_buf.as_mut_ptr().cast();
            hfile.end = hfile_buf.as_mut_ptr().add(written).cast();
            let rb = cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast());
            assert!(!rb.is_null());
            let read_block = rb.cast::<cram_block_layout>();
            assert_eq!(
                (*read_block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!(
                (*read_block).orig_method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!(
                (*read_block).content_type,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL
            );
            assert_eq!((*read_block).content_id, 7);
            assert_eq!((*read_block).comp_size, 3);
            assert_eq!((*read_block).uncomp_size, 3);
            assert_eq!((*read_block).byte, 0);
            assert_eq!((*read_block).bit, 7);
            assert_eq!((*read_block).crc32, expected_crc);
            assert_eq!(
                std::slice::from_raw_parts((*read_block).data, (*read_block).alloc),
                b"abc"
            );
            assert_eq!(hfile.begin, hfile.end);

            cram_cram_io_c_1565_cram_free_block(rb);
            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_read_block_defers_bad_crc_until_uncompress_check() {
        unsafe {
            let mut bytes = [
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW as u8,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL as u8,
                4,
                1,
                1,
                b'Z',
                0,
                0,
                0,
                0,
            ];
            let mut hfile = hfile_layout {
                buffer: bytes.as_mut_ptr().cast(),
                begin: bytes.as_mut_ptr().cast(),
                end: bytes.as_mut_ptr().add(bytes.len()).cast(),
                limit: bytes.as_mut_ptr().add(bytes.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.version = 4 << 8;

            let b = cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast());
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!((*block).crc32, 0);
            assert_eq!((*block).crc32_checked, 0);
            assert_ne!((*block).crc_part, 0);
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).alloc),
                b"Z"
            );

            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), -1);
            assert_eq!((*block).crc32_checked, 1);

            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_read_block_honours_ignore_md5_for_crc_checked_state() {
        unsafe {
            let mut bytes = [
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW as u8,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL as u8,
                4,
                1,
                1,
                b'Z',
                0,
                0,
                0,
                0,
            ];
            let mut hfile = hfile_layout {
                buffer: bytes.as_mut_ptr().cast(),
                begin: bytes.as_mut_ptr().cast(),
                end: bytes.as_mut_ptr().add(bytes.len()).cast(),
                limit: bytes.as_mut_ptr().add(bytes.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.version = 4 << 8;
            fd.ignore_md5 = 1;

            let b = cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast());
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!((*block).crc32, 0);
            assert_eq!((*block).crc32_checked, 1);
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), 0);
            assert_eq!(
                (*block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );

            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_read_block_rejects_invalid_method_and_raw_size_mismatch() {
        unsafe {
            let mut invalid_method = [9u8];
            let mut hfile = hfile_layout {
                buffer: invalid_method.as_mut_ptr().cast(),
                begin: invalid_method.as_mut_ptr().cast(),
                end: invalid_method.as_mut_ptr().add(invalid_method.len()).cast(),
                limit: invalid_method.as_mut_ptr().add(invalid_method.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.version = 4 << 8;

            assert!(
                cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast())
                    .is_null()
            );
            assert_eq!(
                hfile.begin,
                invalid_method.as_mut_ptr().add(1).cast::<c_char>()
            );

            let mut raw_mismatch = [
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW as u8,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL as u8,
                0,
                1,
                2,
            ];
            hfile.buffer = raw_mismatch.as_mut_ptr().cast();
            hfile.begin = raw_mismatch.as_mut_ptr().cast();
            hfile.end = raw_mismatch.as_mut_ptr().add(raw_mismatch.len()).cast();
            hfile.limit = hfile.end;

            assert!(
                cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast())
                    .is_null()
            );
            assert_eq!(
                hfile.begin,
                raw_mismatch
                    .as_mut_ptr()
                    .add(raw_mismatch.len())
                    .cast::<c_char>()
            );
        }
    }

    #[test]
    fn cram_io_write_absent_raw_zero_block_round_trips_v3_and_v4() {
        unsafe {
            for version in [3, 4] {
                let mut hfile_buf = [0u8; 32];
                let mut hfile = hfile_layout {
                    buffer: hfile_buf.as_mut_ptr().cast(),
                    begin: hfile_buf.as_mut_ptr().cast(),
                    end: hfile_buf.as_mut_ptr().cast(),
                    limit: hfile_buf.as_mut_ptr().add(hfile_buf.len()).cast(),
                    backend: std::ptr::null(),
                    offset: 0,
                    flags: 0,
                    has_errno: 0,
                };
                let mut fd: cram_fd_layout = std::mem::zeroed();
                fd.fp = (&mut hfile as *mut hfile_layout).cast();
                fd.version = version << 8;

                let b = cram_cram_io_c_1388_cram_new_block(
                    crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                    0,
                );
                assert!(!b.is_null());
                let block = b.cast::<cram_block_layout>();
                (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
                (*block).orig_method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
                (*block).comp_size = 0;
                (*block).uncomp_size = 0;
                free((*block).data.cast());
                (*block).data = std::ptr::null_mut();
                (*block).alloc = 0;
                (*block).byte = 0;

                assert_eq!(cram_cram_io_c_1490_cram_block_size(b), 9);
                assert_eq!(
                    cram_cram_io_c_1511_cram_write_block(
                        (&mut fd as *mut cram_fd_layout).cast(),
                        b
                    ),
                    0
                );
                let written = hfile.begin.offset_from(hfile_buf.as_mut_ptr().cast()) as usize;
                assert_eq!(written, 9);
                assert_eq!(&hfile_buf[..5], &[0, 4, 0, 0, 0]);
                let expected_crc =
                    crate::htslib_rs::bgzf::hts_crc32(0, hfile_buf.as_ptr().cast(), 5);
                assert_eq!(
                    u32::from_le_bytes(hfile_buf[5..9].try_into().unwrap()),
                    expected_crc
                );

                hfile.begin = hfile_buf.as_mut_ptr().cast();
                hfile.end = hfile_buf.as_mut_ptr().add(written).cast();
                assert_eq!(hfile.begin, hfile_buf.as_mut_ptr().cast());
                assert_eq!(hfile.end, hfile_buf.as_mut_ptr().add(written).cast());
                let rb =
                    cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast());
                assert!(!rb.is_null());
                let read_block = rb.cast::<cram_block_layout>();
                assert_eq!(
                    (*read_block).method,
                    crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
                );
                assert_eq!(
                    (*read_block).content_type,
                    crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL
                );
                assert_eq!((*read_block).content_id, 0);
                assert_eq!((*read_block).comp_size, 0);
                assert_eq!((*read_block).uncomp_size, 0);
                assert_eq!((*read_block).alloc, 0);
                assert!((*read_block).data.is_null());
                assert_eq!((*read_block).crc32, expected_crc);
                assert_eq!((*read_block).crc_part, expected_crc);

                cram_cram_io_c_1565_cram_free_block(rb);
                cram_cram_io_c_1565_cram_free_block(b);
            }
        }
    }

    #[test]
    fn cram_io_read_raw_block_v2_has_no_crc_trailer() {
        unsafe {
            let mut bytes = [
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW as u8,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL as u8,
                7,
                3,
                3,
                b'x',
                b'y',
                b'z',
                0xde,
                0xad,
            ];
            let mut hfile = hfile_layout {
                buffer: bytes.as_mut_ptr().cast(),
                begin: bytes.as_mut_ptr().cast(),
                end: bytes.as_mut_ptr().add(bytes.len()).cast(),
                limit: bytes.as_mut_ptr().add(bytes.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.version = 2 << 8;

            let b = cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast());
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!(
                (*block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!((*block).content_id, 7);
            assert_eq!((*block).comp_size, 3);
            assert_eq!((*block).uncomp_size, 3);
            assert_eq!((*block).crc32_checked, 1);
            assert_eq!((*block).crc32, 0);
            assert_eq!((*block).crc_part, 0);
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).alloc),
                b"xyz"
            );
            assert_eq!(hfile.begin, bytes.as_mut_ptr().add(8).cast::<c_char>());

            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_read_block_v2_itf8_width_comes_from_first_byte() {
        unsafe {
            let mut bytes = [
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW as u8,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL as u8,
                0x80,
                0xff,
                0,
                0,
                0xde,
                0xad,
            ];
            let mut hfile = hfile_layout {
                buffer: bytes.as_mut_ptr().cast(),
                begin: bytes.as_mut_ptr().cast(),
                end: bytes.as_mut_ptr().add(bytes.len()).cast(),
                limit: bytes.as_mut_ptr().add(bytes.len()).cast(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            };
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.fp = (&mut hfile as *mut hfile_layout).cast();
            fd.version = 2 << 8;

            let b = cram_cram_io_c_1414_cram_read_block((&mut fd as *mut cram_fd_layout).cast());
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!((*block).content_id, 255);
            assert_eq!((*block).comp_size, 0);
            assert_eq!((*block).uncomp_size, 0);
            assert_eq!((*block).alloc, 0);
            assert!((*block).data.is_null());
            assert_eq!(hfile.begin, bytes.as_mut_ptr().add(6).cast::<c_char>());

            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_block_size_uses_compressed_payload_and_itf8_header_widths() {
        unsafe {
            let mut block = cram_block_layout {
                method: crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP,
                orig_method: crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0x4000,
                comp_size: 0x80,
                uncomp_size: 0x20_0000,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();
            assert_eq!(
                cram_cram_io_c_1490_cram_block_size(b),
                2 + 3 + 2 + 4 + 4 + 0x80
            );

            block.method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            assert_eq!(block.method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
            assert_eq!(
                cram_cram_io_c_1490_cram_block_size(b),
                2 + 3 + 2 + 4 + 4 + 0x20_0000
            );
        }
    }

    #[test]
    fn cram_io_uncompress_block_matches_crc_and_method_rules() {
        unsafe {
            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!(
                cram_cram_io_h_248_block_append(b, b"abc".as_ptr().cast(), 3),
                0
            );
            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            (*block).uncomp_size = 3;
            (*block).alloc = 3;
            (*block).crc32_checked = 0;
            (*block).crc_part = 0;
            (*block).crc32 = 0;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), -1);
            assert_eq!((*block).crc32_checked, 1);

            (*block).crc32 = crate::htslib_rs::bgzf::hts_crc32(0, (*block).data.cast(), 3);
            (*block).crc32_checked = 0;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), 0);
            assert_eq!(
                (*block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).alloc),
                b"abc"
            );

            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP;
            (*block).uncomp_size = 0;
            (*block).crc32_checked = 1;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), 0);
            assert_eq!(
                (*block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );

            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_BZIP2;
            (*block).uncomp_size = 3;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), -1);
            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_LZMA;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), -1);
            cram_cram_io_c_1565_cram_free_block(b);

            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, b"translated gzip payload").unwrap();
            let compressed = encoder.finish().unwrap();

            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!(
                cram_cram_io_h_248_block_append(b, compressed.as_ptr().cast(), compressed.len()),
                0
            );
            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP;
            (*block).comp_size = compressed.len() as i32;
            (*block).uncomp_size = b"translated gzip payload".len() as i32;
            (*block).crc32_checked = 1;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), 0);
            assert_eq!(
                (*block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!((*block).alloc, b"translated gzip payload".len());
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).alloc),
                b"translated gzip payload"
            );
            cram_cram_io_c_1565_cram_free_block(b);
        }
    }

    #[test]
    fn cram_io_new_metrics_initialises_trial_state_like_c() {
        unsafe {
            let m = cram_cram_io_c_2327_cram_new_metrics();
            assert!(!m.is_null());
            let metrics = m.cast::<cram_metrics_layout>();
            assert_eq!((*metrics).trial, 2);
            assert_eq!((*metrics).next_trial, 35);
            assert_eq!((*metrics).consistency, 0);
            assert_eq!(
                (*metrics).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!((*metrics).strat, 0);
            assert_eq!((*metrics).revised_method, 0);
            assert_eq!((*metrics).unpackable, 0);
            assert!((*metrics).sz.iter().all(|&v| v == 0));
            assert!((*metrics).cnt.iter().all(|&v| v == 0));
            assert!((*metrics).extra.iter().all(|&v| v == 0.0));
            free(m.cast());
        }
    }

    #[test]
    fn cram_io_new_compression_header_allocates_td_state_like_c() {
        unsafe {
            let hdr = cram_cram_io_c_4330_cram_new_compression_header();
            assert!(!hdr.is_null());
            let layout = hdr.cast::<cram_block_compression_hdr_layout>();
            assert!(!(*layout).td_blk.is_null());
            assert!(!(*layout).td_hash.is_null());
            assert!(!(*layout).td_keys.is_null());
            assert_eq!((*layout).ntl, 0);
            assert!((*layout).tl.is_null());
            assert!((*layout).preservation_map.is_null());
            assert!((*layout).rec_encoding_map.iter().all(|p| p.is_null()));
            assert!((*layout).tag_encoding_map.iter().all(|p| p.is_null()));
            assert!((*layout).codecs.iter().all(|p| p.is_null()));

            let block = (*layout).td_blk.cast::<cram_block_layout>();
            assert_eq!(
                (*block).content_type,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE
            );
            assert_eq!((*block).content_id, 0);
            let pool = (*layout).td_keys.cast::<cram_string_alloc_t>();
            assert_eq!((*pool).max_length, 8192);
            assert_eq!((*pool).nstrings, 0);

            cram_cram_io_c_4356_cram_free_compression_header(hdr);
            cram_cram_io_c_4356_cram_free_compression_header(std::ptr::null_mut());
        }
    }

    #[test]
    fn cram_io_reset_metrics_restores_trial_state_and_clears_sizes() {
        unsafe {
            let m0 = cram_cram_io_c_2327_cram_new_metrics().cast::<cram_metrics_layout>();
            let m3 = cram_cram_io_c_2327_cram_new_metrics().cast::<cram_metrics_layout>();
            assert!(!m0.is_null());
            assert!(!m3.is_null());

            (*m0).trial = 99;
            (*m0).next_trial = 12;
            (*m0).revised_method = 7;
            (*m0).unpackable = 1;
            (*m0).sz[0] = 100;
            (*m0).sz[31] = 200;
            (*m0).cnt[1] = 3;

            (*m3).trial = 55;
            (*m3).next_trial = 44;
            (*m3).revised_method = 9;
            (*m3).unpackable = 1;
            (*m3).sz[4] = 333;

            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.m[0] = m0.cast();
            fd.m[3] = m3.cast();

            cram_cram_io_c_4236_reset_metrics((&mut fd as *mut cram_fd_layout).cast());

            for m in [m0, m3] {
                assert_eq!((*m).trial, NTRIALS);
                assert_eq!((*m).next_trial, TRIAL_SPAN);
                assert_eq!((*m).revised_method, 0);
                assert_eq!((*m).unpackable, 0);
                assert!((*m).sz.iter().all(|&v| v == 0));
            }
            assert_eq!((*m0).cnt[1], 3);

            free(m0.cast());
            free(m3.cast());
        }
    }

    #[test]
    fn cram_io_ref_entry_free_seq_closes_or_frees_sequence_like_c() {
        unsafe {
            let seq = malloc(4).cast::<c_char>();
            assert!(!seq.is_null());
            std::ptr::copy_nonoverlapping(c"ACG".as_ptr(), seq, 4);
            let mut heap_entry = ref_entry_layout {
                name: std::ptr::null_mut(),
                fn_: std::ptr::null_mut(),
                length: 0,
                ln_length: 0,
                offset: 0,
                bases_per_line: 0,
                line_length: 0,
                count: 0,
                seq,
                mf: std::ptr::null_mut(),
                is_md5: 0,
                validated_md5: 0,
            };
            cram_cram_io_c_2417_ref_entry_free_seq(
                (&mut heap_entry as *mut ref_entry_layout).cast(),
            );
            assert!(heap_entry.seq.is_null());
            assert!(heap_entry.mf.is_null());

            let mf_data = malloc(4).cast::<c_char>();
            assert!(!mf_data.is_null());
            std::ptr::copy_nonoverlapping(c"TGA".as_ptr(), mf_data, 4);
            let mf = cram_mFILE_c_207_mfcreate(mf_data, 4);
            assert!(!mf.is_null());
            let mut mfile_entry = ref_entry_layout {
                name: std::ptr::null_mut(),
                fn_: std::ptr::null_mut(),
                length: 0,
                ln_length: 0,
                offset: 0,
                bases_per_line: 0,
                line_length: 0,
                count: 0,
                seq: mf_data,
                mf,
                is_md5: 0,
                validated_md5: 0,
            };
            cram_cram_io_c_2417_ref_entry_free_seq(
                (&mut mfile_entry as *mut ref_entry_layout).cast(),
            );
            assert!(mfile_entry.seq.is_null());
            assert!(mfile_entry.mf.is_null());
        }
    }

    #[test]
    fn cram_io_refs_create_and_free_manage_refcount_and_tables_like_c() {
        unsafe {
            let r = cram_cram_io_c_2467_refs_create();
            assert!(!r.is_null());
            let refs = r.cast::<refs_t_layout>();
            assert!(!(*refs).pool.is_null());
            assert!(!(*refs).h_meta.is_null());
            assert!((*refs).ref_id.is_null());
            assert_eq!((*refs).nref, 0);
            assert!((*refs).fn_.is_null());
            assert!((*refs).fp.is_null());
            assert_eq!((*refs).count, 1);
            assert!((*refs).last.is_null());
            assert_eq!((*refs).last_id, -1);
            assert_eq!((*(*refs).pool).max_length, 8192);
            assert_eq!((*(*refs).h_meta).n_buckets, 0);

            (*refs).count = 2;
            cram_cram_io_c_2427_refs_free(r);
            assert_eq!((*refs).count, 1);

            cram_cram_io_c_2427_refs_free(r);

            let r = cram_cram_io_c_2467_refs_create();
            assert!(!r.is_null());
            let refs = r.cast::<refs_t_layout>();
            let h = (*refs).h_meta;
            (*h).n_buckets = 1;
            (*h).size = 1;
            (*h).n_occupied = 1;
            (*h).upper_bound = 1;
            (*h).flags = malloc(std::mem::size_of::<u32>() as u64).cast::<u32>();
            (*h).keys = malloc(std::mem::size_of::<*const c_char>() as u64).cast::<*const c_char>();
            (*h).vals = malloc(std::mem::size_of::<*mut ref_entry_layout>() as u64)
                .cast::<*mut ref_entry_layout>();
            assert!(!(*h).flags.is_null());
            assert!(!(*h).keys.is_null());
            assert!(!(*h).vals.is_null());
            *(*h).flags = 0;
            *(*h).keys = c"chr1".as_ptr();

            let entry = calloc(1, std::mem::size_of::<ref_entry_layout>() as u64)
                .cast::<ref_entry_layout>();
            assert!(!entry.is_null());
            (*entry).seq = malloc(4).cast::<c_char>();
            assert!(!(*entry).seq.is_null());
            std::ptr::copy_nonoverlapping(c"NNN".as_ptr(), (*entry).seq, 4);
            *(*h).vals = entry;

            (*refs).ref_id = malloc(std::mem::size_of::<*mut ref_entry_layout>() as u64)
                .cast::<*mut ref_entry_layout>();
            assert!(!(*refs).ref_id.is_null());
            *(*refs).ref_id = entry;

            cram_cram_io_c_2427_refs_free(r);
        }
    }

    #[test]
    fn cram_io_bgzf_open_ref_strips_file_scheme_and_builds_fai_like_c() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-bgzf-open-ref-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let fai = path.with_extension("fa.fai");
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&fai);
            std::fs::write(&path, b">chr1\nACGTACGT\n").unwrap();

            let uri = CString::new(format!("file://{}", path.to_string_lossy())).unwrap();
            let mut uri_bytes = uri.into_bytes_with_nul();
            let mode = CString::new("r").unwrap();
            let fp = cram_cram_io_c_2503_bgzf_open_ref(
                uri_bytes.as_mut_ptr().cast::<c_char>(),
                mode.as_ptr().cast_mut(),
                0,
            );
            assert!(!fp.is_null());
            assert!(fai.exists());
            assert_eq!((*fp).bitfields & (1 << 30), 0);
            assert_eq!(bgzf_close(fp), 0);

            std::fs::remove_file(&path).unwrap();
            std::fs::remove_file(&fai).unwrap();
        }
    }

    #[test]
    fn cram_io_ref_count_transitions_match_c_delayed_free_rules() {
        unsafe {
            let r = cram_cram_io_c_2467_refs_create();
            assert!(!r.is_null());
            let refs = r.cast::<refs_t_layout>();
            (*refs).nref = 2;
            (*refs).ref_id = malloc(2 * std::mem::size_of::<*mut ref_entry_layout>() as u64)
                .cast::<*mut ref_entry_layout>();
            assert!(!(*refs).ref_id.is_null());

            let entry0 = calloc(1, std::mem::size_of::<ref_entry_layout>() as u64)
                .cast::<ref_entry_layout>();
            let entry1 = calloc(1, std::mem::size_of::<ref_entry_layout>() as u64)
                .cast::<ref_entry_layout>();
            assert!(!entry0.is_null());
            assert!(!entry1.is_null());
            (*entry0).seq = malloc(4).cast::<c_char>();
            (*entry1).seq = malloc(4).cast::<c_char>();
            assert!(!(*entry0).seq.is_null());
            assert!(!(*entry1).seq.is_null());
            (*entry0).length = 8;
            (*entry0).is_md5 = 1;
            (*entry0).count = 0;
            (*entry1).count = 1;
            *(*refs).ref_id = entry0;
            *(*refs).ref_id.add(1) = entry1;

            (*refs).last_id = 0;
            cram_cram_io_c_3183_cram_ref_incr(r, -1);
            assert_eq!((*entry0).count, 0);
            cram_cram_io_c_3183_cram_ref_incr(r, 0);
            assert_eq!((*entry0).count, 1);
            assert_eq!((*refs).last_id, -1);

            cram_cram_io_c_3213_cram_ref_decr(r, 0);
            assert_eq!((*entry0).count, 0);
            assert_eq!((*refs).last_id, 0);
            assert!(!(*entry0).seq.is_null());

            cram_cram_io_c_3213_cram_ref_decr(r, 1);
            assert_eq!((*entry1).count, 0);
            assert_eq!((*refs).last_id, 1);
            assert!((*entry0).seq.is_null());
            assert_eq!((*entry0).length, 0);

            cram_cram_io_c_2417_ref_entry_free_seq(entry1.cast());
            free(entry0.cast());
            free(entry1.cast());
            cram_cram_io_c_2427_refs_free(r);
        }
    }

    #[test]
    fn cram_reference_incr_decr_ignore_null_entries_and_unloaded_sequences() {
        unsafe {
            let mut unloaded = ref_entry_layout {
                name: std::ptr::null_mut(),
                fn_: std::ptr::null_mut(),
                length: 0,
                ln_length: 0,
                offset: 0,
                bases_per_line: 0,
                line_length: 0,
                count: 7,
                seq: std::ptr::null_mut(),
                mf: std::ptr::null_mut(),
                is_md5: 0,
                validated_md5: 0,
            };
            let mut ref_id = [std::ptr::null_mut(), &mut unloaded as *mut ref_entry_layout];
            let mut refs: refs_t_layout = std::mem::zeroed();
            refs.ref_id = ref_id.as_mut_ptr();
            refs.last_id = 1;

            cram_cram_io_c_3169_cram_ref_incr_locked((&mut refs as *mut refs_t_layout).cast(), 0);
            cram_cram_io_c_3189_cram_ref_decr_locked((&mut refs as *mut refs_t_layout).cast(), 0);
            assert_eq!(unloaded.count, 7);
            assert_eq!(refs.last_id, 1);

            cram_cram_io_c_3169_cram_ref_incr_locked((&mut refs as *mut refs_t_layout).cast(), 1);
            cram_cram_io_c_3189_cram_ref_decr_locked((&mut refs as *mut refs_t_layout).cast(), 1);
            assert_eq!(unloaded.count, 7);
            assert_eq!(refs.last_id, 1);
        }
    }

    #[test]
    fn cram_io_load_ref_portion_reads_raw_and_wrapped_fasta_like_c() {
        unsafe {
            let raw_path = std::env::temp_dir().join(format!(
                "htslib_rs-load-ref-raw-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let wrapped_path = std::env::temp_dir().join(format!(
                "htslib_rs-load-ref-wrapped-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let _ = std::fs::remove_file(&raw_path);
            let _ = std::fs::remove_file(&wrapped_path);
            std::fs::write(&raw_path, b"acgtnn").unwrap();
            std::fs::write(&wrapped_path, b">chr1\nacg\nTn\n").unwrap();

            let raw_c = CString::new(raw_path.to_string_lossy().as_bytes()).unwrap();
            let raw_fp = bgzf_open(raw_c.as_ptr(), c"r".as_ptr());
            assert!(!raw_fp.is_null());
            let mut raw_entry = ref_entry_layout {
                name: std::ptr::null_mut(),
                fn_: std::ptr::null_mut(),
                length: 0,
                ln_length: 0,
                offset: 0,
                bases_per_line: 0,
                line_length: 0,
                count: 0,
                seq: std::ptr::null_mut(),
                mf: std::ptr::null_mut(),
                is_md5: 0,
                validated_md5: 0,
            };
            let raw_seq = cram_cram_io_c_3228_load_ref_portion(
                raw_fp,
                (&mut raw_entry as *mut ref_entry_layout).cast(),
                2,
                5,
            );
            assert!(!raw_seq.is_null());
            assert_eq!(std::slice::from_raw_parts(raw_seq.cast::<u8>(), 4), b"CGTN");
            free(raw_seq.cast());
            assert_eq!(bgzf_close(raw_fp), 0);

            let wrapped_c = CString::new(wrapped_path.to_string_lossy().as_bytes()).unwrap();
            let wrapped_fp = bgzf_open(wrapped_c.as_ptr(), c"r".as_ptr());
            assert!(!wrapped_fp.is_null());
            let mut wrapped_entry = ref_entry_layout {
                name: std::ptr::null_mut(),
                fn_: std::ptr::null_mut(),
                length: 0,
                ln_length: 0,
                offset: 6,
                bases_per_line: 3,
                line_length: 4,
                count: 0,
                seq: std::ptr::null_mut(),
                mf: std::ptr::null_mut(),
                is_md5: 0,
                validated_md5: 0,
            };
            let wrapped_seq = cram_cram_io_c_3228_load_ref_portion(
                wrapped_fp,
                (&mut wrapped_entry as *mut ref_entry_layout).cast(),
                2,
                5,
            );
            assert!(!wrapped_seq.is_null());
            assert_eq!(
                std::slice::from_raw_parts(wrapped_seq.cast::<u8>(), 4),
                b"CGTN"
            );
            free(wrapped_seq.cast());
            assert_eq!(bgzf_close(wrapped_fp), 0);

            std::fs::remove_file(raw_path).unwrap();
            std::fs::remove_file(wrapped_path).unwrap();
        }
    }

    #[test]
    fn cram_io_cram_ref_load_opens_reference_and_tracks_last_like_c() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-cram-ref-load-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let fai = path.with_extension("fa.fai");
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&fai);
            std::fs::write(&path, b">chr1\nacgt\n").unwrap();
            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();

            let r = cram_cram_io_c_2467_refs_create();
            assert!(!r.is_null());
            let refs = r.cast::<refs_t_layout>();
            (*refs).fn_ = path_c.as_ptr().cast_mut();
            (*refs).nref = 1;
            (*refs).ref_id = malloc(std::mem::size_of::<*mut ref_entry_layout>() as u64)
                .cast::<*mut ref_entry_layout>();
            assert!(!(*refs).ref_id.is_null());
            let entry = calloc(1, std::mem::size_of::<ref_entry_layout>() as u64)
                .cast::<ref_entry_layout>();
            assert!(!entry.is_null());
            (*entry).fn_ = path_c.as_ptr().cast_mut();
            (*entry).length = 4;
            (*entry).offset = 6;
            (*entry).bases_per_line = 4;
            (*entry).line_length = 5;
            *(*refs).ref_id = entry;

            let loaded = cram_cram_io_c_3323_cram_ref_load(r, 0, 0);
            assert_eq!(loaded, entry.cast());
            assert!(!(*refs).fp.is_null());
            assert_eq!((*refs).last, entry);
            assert_eq!((*entry).count, 2);
            assert_eq!(
                std::slice::from_raw_parts((*entry).seq.cast::<u8>(), 4),
                b"ACGT"
            );
            assert!(fai.exists());

            let loaded_again = cram_cram_io_c_3323_cram_ref_load(r, 0, 0);
            assert_eq!(loaded_again, entry.cast());
            assert_eq!((*entry).count, 2);

            cram_cram_io_c_2417_ref_entry_free_seq(entry.cast());
            free(entry.cast());
            cram_cram_io_c_2427_refs_free(r);
            std::fs::remove_file(path).unwrap();
            std::fs::remove_file(fai).unwrap();
        }
    }

    #[test]
    fn cram_io_refs_load_fai_parses_real_index_and_feeds_ref_loader() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-refs-load-fai-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let fai = path.with_extension("fa.fai");
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&fai);
            std::fs::write(&path, b">chr1\nACGT\n>chr2\nnnac\n").unwrap();

            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let refs = cram_cram_io_c_2541_refs_load_fai(std::ptr::null_mut(), path_c.as_ptr(), 1);
            assert!(!refs.is_null());
            assert!(fai.exists());
            let refs_l = refs.cast::<refs_t_layout>();
            assert_eq!((*refs_l).nref, 2);
            assert!(!(*refs_l).fp.is_null());
            assert_eq!(CStr::from_ptr((*refs_l).fn_).to_bytes(), path_c.as_bytes());

            let chr1 = *(*refs_l).ref_id;
            let chr2 = *(*refs_l).ref_id.add(1);
            assert_eq!(CStr::from_ptr((*chr1).name).to_bytes(), b"chr1");
            assert_eq!((*chr1).length, 4);
            assert_eq!((*chr1).offset, 6);
            assert_eq!((*chr1).bases_per_line, 4);
            assert_eq!((*chr1).line_length, 5);
            assert_eq!(CStr::from_ptr((*chr2).name).to_bytes(), b"chr2");
            assert_eq!((*chr2).length, 4);
            assert_eq!((*chr2).offset, 17);

            let loaded = cram_cram_io_c_3323_cram_ref_load(refs, 1, 0);
            assert_eq!(loaded, chr2.cast());
            assert_eq!(
                std::slice::from_raw_parts((*chr2).seq.cast::<u8>(), 4),
                b"NNAC"
            );

            cram_cram_io_c_2427_refs_free(refs);
            std::fs::remove_file(path).unwrap();
            std::fs::remove_file(fai).unwrap();
        }
    }

    #[test]
    fn cram_io_refs_load_fai_uses_explicit_idx_delimiter_without_changing_ref_name() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-refs-load-explicit-idx-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let default_fai = path.with_extension("fa.fai");
            let idx_path = std::env::temp_dir().join(format!(
                "htslib_rs-refs-load-explicit-idx-{}-{}.fai",
                std::process::id(),
                line!()
            ));
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&default_fai);
            let _ = std::fs::remove_file(&idx_path);
            std::fs::write(&path, b">chrX\nACGT\n").unwrap();
            std::fs::write(&idx_path, b"chrX\t4\t6\t4\t5\n").unwrap();

            let spec = CString::new(format!(
                "{}##idx##{}",
                path.to_string_lossy(),
                idx_path.to_string_lossy()
            ))
            .unwrap();
            let refs = cram_cram_io_c_2541_refs_load_fai(std::ptr::null_mut(), spec.as_ptr(), 1);
            assert!(!refs.is_null());
            let refs_l = refs.cast::<refs_t_layout>();
            assert_eq!((*refs_l).nref, 1);
            assert_eq!(
                CStr::from_ptr((*refs_l).fn_).to_bytes(),
                path.to_string_lossy().as_bytes()
            );

            let chr = *(*refs_l).ref_id;
            assert_eq!(CStr::from_ptr((*chr).name).to_bytes(), b"chrX");
            assert_eq!((*chr).fn_, (*refs_l).fn_);
            assert_eq!((*chr).length, 4);
            assert_eq!((*chr).offset, 6);
            assert_eq!((*chr).bases_per_line, 4);
            assert_eq!((*chr).line_length, 5);

            cram_cram_io_c_2427_refs_free(refs);
            std::fs::remove_file(path).unwrap();
            let _ = std::fs::remove_file(default_fai);
            std::fs::remove_file(idx_path).unwrap();
        }
    }

    #[test]
    fn cram_io_refs2id_and_sanitise_sq_lines_match_header_names() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-refs2id-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let fai = path.with_extension("fa.fai");
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&fai);
            std::fs::write(&path, b">chr1\nACGT\n>chr2\nAACCGG\n").unwrap();

            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let refs = cram_cram_io_c_2541_refs_load_fai(std::ptr::null_mut(), path_c.as_ptr(), 1);
            assert!(!refs.is_null());
            let refs_l = refs.cast::<refs_t_layout>();
            let chr1 = *(*refs_l).ref_id;
            let chr2 = *(*refs_l).ref_id.add(1);
            assert_eq!(CStr::from_ptr((*chr1).name).to_bytes(), b"chr1");
            assert_eq!(CStr::from_ptr((*chr2).name).to_bytes(), b"chr2");

            let mut header_refs = [
                crate::htslib_rs::sam::sam_hrec_sq_t {
                    name: (*chr2).name,
                    len: 1,
                    ty: std::ptr::null_mut(),
                },
                crate::htslib_rs::sam::sam_hrec_sq_t {
                    name: (*chr1).name,
                    len: 2,
                    ty: std::ptr::null_mut(),
                },
                crate::htslib_rs::sam::sam_hrec_sq_t {
                    name: c"missing".as_ptr(),
                    len: 9,
                    ty: std::ptr::null_mut(),
                },
            ];
            let mut hrecs: crate::htslib_rs::sam::sam_hrecs_t = std::mem::zeroed();
            hrecs.nref = header_refs.len() as c_int;
            hrecs.ref_ = header_refs.as_mut_ptr();
            let mut hdr: sam_hdr_t = std::mem::zeroed();
            hdr.hrecs = &mut hrecs;

            (*refs_l).last = chr1;
            assert_eq!(cram_cram_io_c_2737_refs2id(refs, &mut hdr), 0);
            assert!((*refs_l).last.is_null());
            assert_eq!((*refs_l).nref, 3);
            assert_eq!(*(*refs_l).ref_id, chr2);
            assert_eq!(*(*refs_l).ref_id.add(1), chr1);
            assert!((*(*refs_l).ref_id.add(2)).is_null());

            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.header = (&mut hdr as *mut sam_hdr_t).cast();
            fd.refs = refs.cast();
            cram_cram_io_c_2693_sanitise_SQ_lines((&mut fd as *mut cram_fd_layout).cast());
            assert_eq!(header_refs[0].len, (*chr2).length);
            assert_eq!(header_refs[1].len, (*chr1).length);
            assert_eq!(header_refs[2].len, 9);

            cram_cram_io_c_2427_refs_free(refs);
            std::fs::remove_file(path).unwrap();
            std::fs::remove_file(fai).unwrap();
        }
    }

    #[test]
    fn cram_io_refs_from_header_adds_missing_sq_refs_to_metadata_hash() {
        unsafe {
            let refs = cram_cram_io_c_2467_refs_create();
            assert!(!refs.is_null());
            let refs_l = refs.cast::<refs_t_layout>();

            let chr1 = c"chr1";
            let chr2 = c"chr2";
            let mut header_refs = [
                crate::htslib_rs::sam::sam_hrec_sq_t {
                    name: chr1.as_ptr(),
                    len: 123,
                    ty: std::ptr::null_mut(),
                },
                crate::htslib_rs::sam::sam_hrec_sq_t {
                    name: chr2.as_ptr(),
                    len: 456,
                    ty: std::ptr::null_mut(),
                },
            ];
            let mut hrecs: crate::htslib_rs::sam::sam_hrecs_t = std::mem::zeroed();
            hrecs.nref = header_refs.len() as c_int;
            hrecs.ref_ = header_refs.as_mut_ptr();
            let mut hdr: sam_hdr_t = std::mem::zeroed();
            hdr.hrecs = &mut hrecs;

            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.refs = refs.cast();
            fd.header = (&mut hdr as *mut sam_hdr_t).cast();

            assert_eq!(
                cram_cram_io_c_2768_refs_from_header((&mut fd as *mut cram_fd_layout).cast()),
                0
            );
            assert_eq!((*refs_l).nref, 2);
            assert!(!(*refs_l).ref_id.is_null());
            assert_eq!(
                CStr::from_ptr((*(*(*refs_l).ref_id.add(0))).name),
                CStr::from_ptr(chr1.as_ptr())
            );
            assert_eq!(
                CStr::from_ptr((*(*(*refs_l).ref_id.add(1))).name),
                CStr::from_ptr(chr2.as_ptr())
            );
            assert_eq!((*(*(*refs_l).ref_id.add(0))).length, 0);

            assert_eq!(
                cram_cram_io_c_2768_refs_from_header((&mut fd as *mut cram_fd_layout).cast()),
                0
            );
            assert_eq!((*refs_l).nref, 2);

            cram_cram_io_c_2427_refs_free(refs);
        }
    }

    #[test]
    fn cram_io_load_reference_uses_fai_or_header_refs_like_c() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-load-ref-{}-{}.fa",
                std::process::id(),
                line!()
            ));
            let fai = path.with_extension("fa.fai");
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(&fai);
            std::fs::write(&path, b">chr1\nACGT\n>chr2\nAACCGG\n").unwrap();
            assert_eq!(
                fai_build(
                    CString::new(path.to_string_lossy().as_bytes())
                        .unwrap()
                        .as_ptr()
                ),
                0
            );

            let chr2 = c"chr2";
            let mut header_refs = [crate::htslib_rs::sam::sam_hrec_sq_t {
                name: chr2.as_ptr(),
                len: 1,
                ty: std::ptr::null_mut(),
            }];
            let mut hrecs: crate::htslib_rs::sam::sam_hrecs_t = std::mem::zeroed();
            hrecs.nref = 1;
            hrecs.ref_ = header_refs.as_mut_ptr();
            let mut hdr: sam_hdr_t = std::mem::zeroed();
            hdr.hrecs = &mut hrecs;
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.header = (&mut hdr as *mut sam_hdr_t).cast();
            fd.mode = b'r' as c_int;

            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            assert_eq!(
                cram_cram_io_c_3597_cram_load_reference(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    path_c.as_ptr().cast_mut(),
                ),
                0
            );
            assert!(!fd.refs.is_null());
            assert_eq!(fd.ref_fn, (*fd.refs.cast::<refs_t_layout>()).fn_);
            assert_eq!((*fd.refs.cast::<refs_t_layout>()).nref, 1);
            assert_eq!(header_refs[0].len, 6);
            assert_eq!(
                CStr::from_ptr((*(*(*fd.refs.cast::<refs_t_layout>()).ref_id)).name).to_bytes(),
                b"chr2"
            );
            cram_cram_io_c_2427_refs_free(fd.refs.cast());

            let refs = cram_cram_io_c_2467_refs_create();
            let mut header_refs = [crate::htslib_rs::sam::sam_hrec_sq_t {
                name: c"header_only".as_ptr(),
                len: 77,
                ty: std::ptr::null_mut(),
            }];
            let mut hrecs2: crate::htslib_rs::sam::sam_hrecs_t = std::mem::zeroed();
            hrecs2.nref = 1;
            hrecs2.ref_ = header_refs.as_mut_ptr();
            let mut hdr2: sam_hdr_t = std::mem::zeroed();
            hdr2.hrecs = &mut hrecs2;
            fd.header = (&mut hdr2 as *mut sam_hdr_t).cast();
            fd.refs = refs.cast();
            fd.ref_fn = std::ptr::null_mut();
            assert_eq!(
                cram_cram_io_c_3597_cram_load_reference(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    std::ptr::null_mut(),
                ),
                0
            );
            assert!(!fd.refs.is_null());
            assert_eq!((*fd.refs.cast::<refs_t_layout>()).nref, 1);
            assert!(
                CStr::from_ptr((*(*(*fd.refs.cast::<refs_t_layout>()).ref_id)).name).to_bytes()
                    == b"header_only"
            );
            cram_cram_io_c_2427_refs_free(fd.refs.cast());

            std::fs::remove_file(path).unwrap();
            std::fs::remove_file(fai).unwrap();
        }
    }

    #[test]
    fn cram_io_set_header2_attaches_header_and_populates_refs() {
        unsafe {
            assert_eq!(
                cram_cram_io_c_2852_cram_set_header2(std::ptr::null_mut(), std::ptr::null()),
                -1
            );

            let refs = cram_cram_io_c_2467_refs_create();
            assert!(!refs.is_null());
            let refs_l = refs.cast::<refs_t_layout>();

            let mut header_refs = [crate::htslib_rs::sam::sam_hrec_sq_t {
                name: c"chr_set".as_ptr(),
                len: 30,
                ty: std::ptr::null_mut(),
            }];
            let mut hrecs: crate::htslib_rs::sam::sam_hrecs_t = std::mem::zeroed();
            hrecs.nref = 1;
            hrecs.ref_ = header_refs.as_mut_ptr();
            let mut hdr: sam_hdr_t = std::mem::zeroed();
            hdr.hrecs = &mut hrecs;

            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.refs = refs.cast();
            fd.header = (&mut hdr as *mut sam_hdr_t).cast();

            assert_eq!(
                cram_cram_io_c_2852_cram_set_header2((&mut fd as *mut cram_fd_layout).cast(), &hdr,),
                0
            );
            assert_eq!(fd.header, (&mut hdr as *mut sam_hdr_t).cast());
            assert_eq!((*refs_l).nref, 1);
            assert_eq!(
                CStr::from_ptr((*(*(*refs_l).ref_id)).name).to_bytes(),
                b"chr_set"
            );

            assert_eq!(
                cram_cram_io_c_2866_cram_set_header(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    &mut hdr,
                ),
                0
            );
            assert_eq!((*refs_l).nref, 1);

            cram_cram_io_c_2427_refs_free(refs);
        }
    }

    #[test]
    fn cram_io_cram_hfile_returns_first_fd_field() {
        unsafe {
            let mut fd = cram_fd_layout {
                fp: test_hfile_marker(),
                mode: 0,
                version: 0,
                ..std::mem::zeroed()
            };
            assert_eq!(
                cram_cram_io_h_646_cram_hfile((&mut fd as *mut cram_fd_layout).cast()),
                test_hfile_marker()
            );
        }
    }

    #[test]
    fn cram_io_cram_eof_returns_fd_eof_field() {
        unsafe {
            let mut fd: cram_fd_layout = std::mem::zeroed();
            fd.eof = 0;
            assert_eq!(
                cram_cram_io_c_5662_cram_eof((&mut fd as *mut cram_fd_layout).cast()),
                0
            );
            fd.eof = 1;
            assert_eq!(
                cram_cram_io_c_5662_cram_eof((&mut fd as *mut cram_fd_layout).cast()),
                1
            );
        }
    }

    #[test]
    fn cram_stats_track_small_and_hash_values() {
        unsafe {
            let st = cram_cram_stats_c_48_cram_stats_create();
            assert!(!st.is_null());

            cram_cram_stats_c_52_cram_stats_add(st, 5);
            cram_cram_stats_c_52_cram_stats_add(st, 5);
            cram_cram_stats_c_52_cram_stats_add(st, 1024);
            cram_cram_stats_c_52_cram_stats_add(st, -3);

            let layout = st.cast::<cram_stats_layout>();
            assert_eq!((*layout).nsamp, 4);
            assert_eq!((*layout).freqs[5], 2);
            assert!(!(*layout).h.is_null());

            let h = (*layout).h.cast::<kh_m_i2i_layout>();
            let mut saw_1024 = 0;
            let mut saw_minus_3 = 0;
            for k in 0..(*h).n_buckets {
                let flag = *(*h).flags.add((k >> 4) as usize);
                if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                    continue;
                }
                match *(*h).keys.add(k as usize) {
                    1024 => saw_1024 = *(*h).vals.add(k as usize),
                    -3 => saw_minus_3 = *(*h).vals.add(k as usize),
                    _ => {}
                }
            }
            assert_eq!(saw_1024, 1);
            assert_eq!(saw_minus_3, 1);

            cram_cram_stats_c_80_cram_stats_del(st, 5);
            assert_eq!((*layout).freqs[5], 1);
            assert_eq!((*layout).nsamp, 3);

            cram_cram_stats_c_80_cram_stats_del(st, 99999);
            assert_eq!((*layout).nsamp, 3);

            let mut fd = cram_fd_layout {
                fp: std::ptr::null_mut(),
                mode: 0,
                version: 4 << 8,
                ..std::mem::zeroed()
            };
            assert_eq!(
                cram_cram_stats_c_134_cram_stats_encoding(
                    (&mut fd as *mut cram_fd_layout).cast(),
                    st
                ),
                42
            );
            assert_eq!((*layout).nvals, 3);
            assert_eq!((*layout).min_val, -3);
            assert_eq!((*layout).max_val, 1024);

            cram_cram_stats_c_223_cram_stats_free(st);
        }
    }

    #[test]
    fn cram_stats_encoding_matches_version_policy() {
        unsafe {
            let mut fd4 = cram_fd_layout {
                fp: std::ptr::null_mut(),
                mode: 0,
                version: 4 << 8,
                ..std::mem::zeroed()
            };
            let mut fd3 = cram_fd_layout {
                fp: std::ptr::null_mut(),
                mode: 0,
                version: 3 << 8,
                ..std::mem::zeroed()
            };

            let empty = cram_cram_stats_c_48_cram_stats_create();
            assert_eq!(
                cram_cram_stats_c_134_cram_stats_encoding(
                    (&mut fd4 as *mut cram_fd_layout).cast(),
                    empty
                ),
                42
            );
            cram_cram_stats_c_223_cram_stats_free(empty);

            let single = cram_cram_stats_c_48_cram_stats_create();
            cram_cram_stats_c_52_cram_stats_add(single, 7);
            assert_eq!(
                cram_cram_stats_c_134_cram_stats_encoding(
                    (&mut fd4 as *mut cram_fd_layout).cast(),
                    single
                ),
                44
            );
            assert_eq!(
                cram_cram_stats_c_134_cram_stats_encoding(
                    (&mut fd3 as *mut cram_fd_layout).cast(),
                    single
                ),
                3
            );
            cram_cram_stats_c_223_cram_stats_free(single);

            let multi = cram_cram_stats_c_48_cram_stats_create();
            cram_cram_stats_c_52_cram_stats_add(multi, 7);
            cram_cram_stats_c_52_cram_stats_add(multi, 8);
            assert_eq!(
                cram_cram_stats_c_134_cram_stats_encoding(
                    (&mut fd4 as *mut cram_fd_layout).cast(),
                    multi
                ),
                41
            );
            assert_eq!(
                cram_cram_stats_c_134_cram_stats_encoding(
                    (&mut fd3 as *mut cram_fd_layout).cast(),
                    multi
                ),
                1
            );
            cram_cram_stats_c_223_cram_stats_free(multi);
        }
    }

    #[test]
    fn cram_codecs_bit_readers_count_runs_and_extract_msb_values() {
        unsafe {
            let mut data = [0b1110_0101u8, 0b0000_0000];
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: data.len() as i32,
                crc32: 0,
                idx: 0,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();

            assert_eq!(cram_cram_codecs_c_73_get_bit_MSB(b), 1);
            assert_eq!((block.byte, block.bit), (0, 6));
            assert_eq!(cram_cram_codecs_c_95_get_one_bits_MSB(b), 2);
            assert_eq!((block.byte, block.bit), (0, 3));
            assert_eq!(cram_cram_codecs_c_169_get_bits_MSB(b, 4), 0b0101);
            assert_eq!((block.byte, block.bit), (1, 7));

            block.byte = 0;
            block.bit = 3;
            assert_eq!(cram_cram_codecs_c_113_get_zero_bits_MSB(b), 1);
            assert_eq!((block.byte, block.bit), (0, 1));

            let mut eof_block = cram_block_layout {
                byte: data.len(),
                ..block
            };
            let eof_b = (&mut eof_block as *mut cram_block_layout).cast();
            assert_eq!(cram_cram_codecs_c_95_get_one_bits_MSB(eof_b), -1);
            assert_eq!(cram_cram_codecs_c_113_get_zero_bits_MSB(eof_b), -1);
        }
    }

    #[test]
    fn cram_codecs_bit_writers_and_byte_store_match_msb_layout() {
        unsafe {
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();

            assert_eq!(cram_cram_codecs_c_259_store_bits_MSB(b, 0b101, 3), 0);
            cram_cram_codecs_c_133_store_bit_MSB(b, 1);
            assert_eq!(block.byte, 0);
            assert_eq!(block.bit, 3);
            assert_eq!(*block.data, 0b1011_0000);

            let mut bytes = *b"xy";
            cram_cram_codecs_c_152_store_bytes_MSB(b, bytes.as_mut_ptr().cast(), 2);
            assert_eq!(block.byte, 3);
            assert_eq!(std::slice::from_raw_parts(block.data, 3), b"\xb0xy");

            block.uncomp_size = 3;
            block.byte = 0;
            block.bit = 7;
            assert_eq!(cram_cram_codecs_c_169_get_bits_MSB(b, 4), 0b1011);
            assert_eq!(cram_cram_codecs_c_169_get_bits_MSB(b, 4), 0);
            assert_eq!(cram_cram_codecs_c_169_get_bits_MSB(b, 8), b'x' as i64);
            assert_eq!(cram_cram_codecs_c_169_get_bits_MSB(b, 8), b'y' as i64);

            free(block.data.cast());
        }
    }

    #[test]
    fn cram_codecs_not_enough_bits_and_beta_decoders_match_bitstream() {
        unsafe {
            let mut data = [0b1010_1100u8];
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: data.len() as i32,
                crc32: 0,
                idx: 0,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();
            assert_eq!(cram_cram_codecs_h_230_cram_not_enough_bits(b, 8), 0);
            assert_eq!(cram_cram_codecs_h_230_cram_not_enough_bits(b, 9), 1);
            assert_eq!(cram_cram_codecs_h_230_cram_not_enough_bits(b, -1), 1);

            let mut codec = cram_codec_beta_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                beta: cram_beta_decoder_layout {
                    offset: 1,
                    nbits: 4,
                },
            };
            let c = (&mut codec as *mut cram_codec_beta_layout).cast();
            let mut out_size = 2;
            let mut ints = [0i32; 2];
            assert_eq!(
                cram_cram_codecs_c_1090_cram_beta_decode_int(
                    std::ptr::null_mut(),
                    c,
                    b,
                    ints.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(ints, [9, 11]);

            block.byte = 0;
            block.bit = 7;
            let mut longs = [0i64; 2];
            assert_eq!(
                cram_cram_codecs_c_1072_cram_beta_decode_long(
                    std::ptr::null_mut(),
                    c,
                    b,
                    longs.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(longs, [9, 11]);

            block.byte = 0;
            block.bit = 7;
            assert_eq!(
                cram_cram_codecs_c_1108_cram_beta_decode_char(
                    std::ptr::null_mut(),
                    c,
                    b,
                    std::ptr::null_mut(),
                    &mut out_size
                ),
                0
            );
            assert_eq!((block.byte, block.bit), (1, 7));

            codec.beta.nbits = 0;
            codec.beta.offset = -5;
            let mut bytes = [0 as c_char; 3];
            out_size = 3;
            assert_eq!(
                cram_cram_codecs_c_1108_cram_beta_decode_char(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_beta_layout).cast(),
                    b,
                    bytes.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(bytes.as_ptr().cast::<u8>(), 3),
                &[5, 5, 5]
            );
        }
    }

    #[test]
    fn cram_codecs_beta_encoders_store_offset_values_msb_first() {
        unsafe {
            let mut out_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut codec = cram_codec_beta_layout {
                codec: 0,
                out: (&mut out_block as *mut cram_block_layout).cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                beta: cram_beta_decoder_layout {
                    offset: 1,
                    nbits: 4,
                },
            };
            let c = (&mut codec as *mut cram_codec_beta_layout).cast();

            let mut ints = [2i32, 5i32];
            assert_eq!(
                cram_cram_codecs_c_1219_cram_beta_encode_int(
                    std::ptr::null_mut(),
                    c,
                    ints.as_mut_ptr().cast(),
                    ints.len() as c_int
                ),
                0
            );
            assert_eq!(
                (out_block.byte, out_block.bit, *out_block.data),
                (1, 7, 0x36)
            );

            out_block.byte = 0;
            out_block.bit = 7;
            *out_block.data = 0;
            let mut longs = [6i64, 8i64];
            assert_eq!(
                cram_cram_codecs_c_1207_cram_beta_encode_long(
                    std::ptr::null_mut(),
                    c,
                    longs.as_mut_ptr().cast(),
                    longs.len() as c_int
                ),
                0
            );
            assert_eq!(
                (out_block.byte, out_block.bit, *out_block.data),
                (1, 7, 0x79)
            );

            out_block.byte = 0;
            out_block.bit = 7;
            *out_block.data = 0;
            let mut bytes = [1u8, 14u8];
            assert_eq!(
                cram_cram_codecs_c_1231_cram_beta_encode_char(
                    std::ptr::null_mut(),
                    c,
                    bytes.as_mut_ptr().cast(),
                    bytes.len() as c_int
                ),
                0
            );
            assert_eq!(
                (out_block.byte, out_block.bit, *out_block.data),
                (1, 7, 0x2f)
            );

            let mut dat = [-2i64, 5i64];
            let enc = cram_cram_codecs_c_1247_cram_beta_encode_init(
                std::ptr::null_mut(),
                6,
                1,
                dat.as_mut_ptr().cast(),
                3 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_beta_layout>();
            assert!(!enc.is_null());
            assert_eq!((*enc).codec, 6);
            assert_eq!((*enc).beta.offset, 2);
            assert_eq!((*enc).beta.nbits, 3);
            assert_eq!(
                (*enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_1219_cram_beta_encode_int as usize)
            );
            assert_eq!(
                (*enc).store,
                cram_fn_ptr(cram_cram_codecs_c_1183_cram_beta_encode_store as usize)
            );
            cram_cram_codecs_c_1243_cram_beta_encode_free(enc.cast());

            let mut st = cram_stats_layout {
                freqs: [0; 1024],
                h: std::ptr::null_mut(),
                nsamp: 0,
                nvals: 2,
                min_val: 0,
                max_val: 0,
            };
            st.freqs[3] = 10;
            st.freqs[9] = 1;
            let enc = cram_cram_codecs_c_1247_cram_beta_encode_init(
                (&mut st as *mut cram_stats_layout).cast(),
                6,
                1,
                std::ptr::null_mut(),
                3 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_beta_layout>();
            assert!(!enc.is_null());
            assert_eq!((*enc).beta.offset, -3);
            assert_eq!((*enc).beta.nbits, 3);
            cram_cram_codecs_c_1243_cram_beta_encode_free(enc.cast());

            free(out_block.data.cast());
        }
    }

    #[test]
    fn cram_codecs_xpack_decode_and_encode_use_maps() {
        unsafe {
            let mut input_data = [0b01_10_11_00u8];
            let mut input_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: input_data.len() as i32,
                crc32: 0,
                idx: 0,
                data: input_data.as_mut_ptr(),
                alloc: input_data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut out_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut rmap = [0u32; 256];
            rmap[0] = 10;
            rmap[1] = 20;
            rmap[2] = 30;
            rmap[3] = 40;
            let mut map = [0i32; 256];
            map[10] = 0;
            map[20] = 1;
            map[30] = 2;
            map[40] = 3;
            let mut codec = cram_codec_xpack_layout {
                codec: 0,
                out: (&mut out_block as *mut cram_block_layout).cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xpack: cram_xpack_decoder_layout {
                    nbits: 2,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: std::ptr::null_mut(),
                    nval: 4,
                    rmap,
                    map,
                },
            };
            let c = (&mut codec as *mut cram_codec_xpack_layout).cast();
            let b = (&mut input_block as *mut cram_block_layout).cast();

            let mut out_size = 4;
            let mut ints = [0i32; 4];
            assert_eq!(
                cram_cram_codecs_c_1359_cram_xpack_decode_int(
                    std::ptr::null_mut(),
                    c,
                    b,
                    ints.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(ints, [20, 30, 40, 10]);

            input_block.byte = 0;
            input_block.bit = 7;
            let b = (&mut input_block as *mut cram_block_layout).cast();
            let mut longs = [0i64; 4];
            assert_eq!(
                cram_cram_codecs_c_1344_cram_xpack_decode_long(
                    std::ptr::null_mut(),
                    c,
                    b,
                    longs.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(longs, [20, 30, 40, 10]);

            let mut syms = [20i32, 30, 40, 10];
            assert_eq!(
                cram_cram_codecs_c_1592_cram_xpack_encode_int(
                    std::ptr::null_mut(),
                    c,
                    syms.as_mut_ptr().cast(),
                    syms.len() as c_int
                ),
                0
            );
            assert_eq!(
                (out_block.byte, out_block.bit, *out_block.data),
                (1, 7, 0b01_10_11_00)
            );

            out_block.byte = 0;
            out_block.bit = 7;
            *out_block.data = 0;
            let mut long_syms = [40i64, 10, 20, 30];
            assert_eq!(
                cram_cram_codecs_c_1581_cram_xpack_encode_long(
                    std::ptr::null_mut(),
                    c,
                    long_syms.as_mut_ptr().cast(),
                    long_syms.len() as c_int
                ),
                0
            );
            assert_eq!(
                (out_block.byte, out_block.bit, *out_block.data),
                (1, 7, 0b11_00_01_10)
            );

            let mut raw = *b"xy";
            assert_eq!(
                cram_cram_codecs_c_1603_cram_xpack_encode_char(
                    std::ptr::null_mut(),
                    c,
                    raw.as_mut_ptr().cast(),
                    raw.len() as c_int
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(out_block.data, out_block.byte),
                b"\xc6xy"
            );

            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut sub_codec = cram_codec_xpack_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: cram_fn_ptr(test_byte_array_len_store_val as usize),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xpack: cram_xpack_decoder_layout {
                    nbits: 0,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: std::ptr::null_mut(),
                    nval: 0,
                    rmap: [0; 256],
                    map: [0; 256],
                },
            };
            codec.codec = 51;
            codec.vv = &mut vv as *mut varint_vec_layout;
            codec.xpack.sub_codec = (&mut sub_codec as *mut cram_codec_xpack_layout).cast();
            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_1537_cram_xpack_encode_store(
                    (&mut codec as *mut cram_codec_xpack_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                11
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 51, 8, 2, 4, 10, 20, 30, 40, b'V', b'A']
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            let sub_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                9,
            );
            let mut packed = [0b01_10_11_00u8];
            assert_eq!(
                cram_cram_io_h_248_block_append(
                    sub_block,
                    packed.as_mut_ptr().cast(),
                    packed.len()
                ),
                0
            );
            let sub_layout = sub_block.cast::<cram_block_layout>();
            (*sub_layout).uncomp_size = 1;
            (*sub_layout).byte = 0;
            let mut sub_get_codec = cram_codec_xpack_layout {
                codec: 0,
                out: sub_block.cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: cram_fn_ptr(test_xdelta_get_block as usize),
                describe: std::ptr::null_mut(),
                xpack: cram_xpack_decoder_layout {
                    nbits: 0,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: std::ptr::null_mut(),
                    nval: 0,
                    rmap: [0; 256],
                    map: [0; 256],
                },
            };
            codec.codec_id = 2;
            codec.xpack.sub_codec = (&mut sub_get_codec as *mut cram_codec_xpack_layout).cast();
            let mut by_id = vec![std::ptr::null_mut(); 520];
            let mut slice = cram_slice_layout {
                hdr: std::ptr::null_mut(),
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: by_id.as_mut_ptr(),
                ..std::mem::zeroed()
            };
            assert_eq!(
                cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    (&mut codec as *mut cram_codec_xpack_layout).cast(),
                ),
                0
            );
            let expanded = by_id[514].cast::<cram_block_layout>();
            assert_eq!((*expanded).uncomp_size, 4);
            assert_eq!(
                std::slice::from_raw_parts((*expanded).data, 4),
                &[10, 40, 30, 20]
            );
            assert_eq!(
                cram_cram_codecs_c_1443_cram_xpack_decode_size(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    (&mut codec as *mut cram_codec_xpack_layout).cast(),
                ),
                4
            );
            let mut unpacked = [0 as c_char; 4];
            let mut unpacked_size = 4;
            assert_eq!(
                cram_cram_codecs_c_1408_cram_xpack_decode_char(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    (&mut codec as *mut cram_codec_xpack_layout).cast(),
                    std::ptr::null_mut(),
                    unpacked.as_mut_ptr(),
                    &mut unpacked_size,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(unpacked.as_ptr().cast::<u8>(), 4),
                &[10, 40, 30, 20]
            );
            assert_eq!((*expanded).byte, 4);
            assert_eq!(
                cram_cram_codecs_c_1448_cram_xpack_get_block(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    (&mut codec as *mut cram_codec_xpack_layout).cast(),
                ),
                by_id[514].cast()
            );
            cram_cram_io_c_1565_cram_free_block(by_id[514].cast());
            cram_cram_io_c_1565_cram_free_block(sub_block);

            let mut hdr: cram_block_compression_hdr_layout = std::mem::zeroed();
            let mut xpack_header = [2u8, 4, 10, 20, 30, 40, 1, 1, 9];
            let xpack_dec = cram_cram_codecs_c_3872_cram_decoder_init(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                51,
                xpack_header.as_mut_ptr().cast(),
                xpack_header.len() as c_int,
                3,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_xpack_layout>();
            assert!(!xpack_dec.is_null());
            assert_eq!(hdr.ncodecs, 2);
            assert_eq!((*xpack_dec).codec_id, 1);
            assert_eq!(
                (*xpack_dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_1408_cram_xpack_decode_char as usize)
            );
            assert_eq!((*xpack_dec).xpack.rmap[3], 40);
            assert_eq!(
                (*(*xpack_dec)
                    .xpack
                    .sub_codec
                    .cast::<cram_codec_external_layout>())
                .external
                .content_id,
                9
            );
            cram_cram_codecs_c_1431_cram_xpack_decode_free(xpack_dec.cast());

            let mut xpack_dat = cram_xpack_decoder_layout {
                nbits: 2,
                sub_encoding: 1,
                sub_codec_dat: test_void_token(9),
                sub_codec: std::ptr::null_mut(),
                nval: 4,
                rmap: [0; 256],
                map: [-1; 256],
            };
            xpack_dat.map[10] = 0;
            xpack_dat.map[20] = 1;
            xpack_dat.map[30] = 2;
            xpack_dat.map[40] = 3;
            let xpack_enc = cram_cram_codecs_c_3928_cram_encoder_init(
                51,
                std::ptr::null_mut(),
                1,
                (&mut xpack_dat as *mut cram_xpack_decoder_layout).cast(),
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_xpack_layout>();
            assert!(!xpack_enc.is_null());
            assert_eq!((*xpack_enc).codec, 51);
            assert_eq!(
                (*xpack_enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_1592_cram_xpack_encode_int as usize)
            );
            assert_eq!(
                (*xpack_enc).flush,
                cram_fn_ptr(cram_cram_codecs_c_1515_cram_xpack_encode_flush as usize)
            );
            assert_eq!((*xpack_enc).xpack.rmap[0], 10);
            assert_eq!((*xpack_enc).xpack.rmap[3], 40);
            assert_eq!(
                (*(*xpack_enc)
                    .xpack
                    .sub_codec
                    .cast::<cram_codec_external_layout>())
                .external
                .content_id,
                9
            );
            cram_cram_codecs_c_1612_cram_xpack_encode_free(xpack_enc.cast());
            free(out_block.data.cast());
        }
    }

    #[test]
    fn cram_codecs_extract_block_advances_idx_and_bounds_checks_after_advance() {
        unsafe {
            let mut data = *b"abcdef";
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 6,
                crc32: 0,
                idx: 2,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let b = (&mut block as *mut cram_block_layout).cast();

            let cp = cram_cram_codecs_c_319_cram_extract_block(b, 3);
            assert_eq!(std::slice::from_raw_parts(cp.cast::<u8>(), 3), b"cde");
            assert_eq!(block.idx, 5);

            assert!(cram_cram_codecs_c_319_cram_extract_block(b, 2).is_null());
            assert_eq!(block.idx, 7);
        }
    }

    #[test]
    fn cram_io_new_and_free_block_initialise_c_fields() {
        unsafe {
            let b = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                99,
            );
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!(
                (*block).method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!(
                (*block).orig_method,
                crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW
            );
            assert_eq!(
                (*block).content_type,
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL
            );
            assert_eq!((*block).content_id, 99);
            assert_eq!((*block).byte, 0);
            assert_eq!((*block).bit, 7);
            assert!((*block).data.is_null());

            let mut bytes = *b"owned";
            assert_eq!(
                cram_cram_io_h_248_block_append(b, bytes.as_mut_ptr().cast(), bytes.len()),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).byte),
                b"owned"
            );
            (*block).uncomp_size = 5;
            (*block).comp_size = 3;
            assert_eq!(cram_cram_io_c_1490_cram_block_size(b), 14);
            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP;
            assert_eq!(cram_cram_io_c_1490_cram_block_size(b), 12);
            (*block).content_id = 16_384;
            assert_eq!(cram_cram_io_c_1490_cram_block_size(b), 14);
            cram_cram_io_c_1565_cram_free_block(b);
            cram_cram_io_c_1565_cram_free_block(std::ptr::null_mut());
        }
    }

    #[test]
    fn cram_codecs_xdelta_zigzag_and_placeholders_match_c() {
        assert_eq!(cram_cram_codecs_c_1676_zigzag8(0), 0);
        assert_eq!(cram_cram_codecs_c_1676_zigzag8(-1), 1);
        assert_eq!(cram_cram_codecs_c_1676_zigzag8(1), 2);
        assert_eq!(cram_cram_codecs_c_1677_zigzag16(-2), 3);
        assert_eq!(cram_cram_codecs_c_1678_zigzag32(-3), 5);
        assert_eq!(cram_cram_codecs_c_1681_unzigzag16(3), -2);
        assert_eq!(cram_cram_codecs_c_1682_unzigzag32(5), -3);
        assert_eq!(
            cram_cram_codecs_c_1713_le_int2(0x1234),
            i16::from_le(0x1234)
        );
        assert_eq!(
            cram_cram_codecs_c_1684_cram_xdelta_decode_long(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ),
            -1
        );
        assert_eq!(
            cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ),
            -1
        );
        assert_eq!(
            cram_cram_codecs_c_1709_cram_xdelta_decode_char(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ),
            -1
        );
        assert_eq!(
            cram_cram_codecs_c_1966_cram_xdelta_encode_long(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0
            ),
            -1
        );
        assert_eq!(
            cram_cram_codecs_c_1971_cram_xdelta_encode_int(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0
            ),
            -1
        );
    }

    #[test]
    fn cram_codecs_xdelta_size_and_get_block_use_slice_cache() {
        unsafe {
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 17,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let block_ptr = (&mut block as *mut cram_block_layout).cast::<cram_block>();
            let mut by_id = [std::ptr::null_mut(); 513];
            by_id[512] = block_ptr;
            let mut slice = cram_slice_layout {
                hdr: std::ptr::null_mut(),
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: by_id.as_mut_ptr().cast(),
                ..std::mem::zeroed()
            };
            let mut codec = cram_codec_xdelta_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xdelta: cram_xdelta_decoder_layout {
                    last: 0,
                    word_size: 2,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: std::ptr::null_mut(),
                },
            };
            let s = (&mut slice as *mut cram_slice_layout).cast();
            let c = (&mut codec as *mut cram_codec_xdelta_layout).cast();
            assert_eq!(cram_cram_codecs_c_1771_cram_xdelta_decode_size(s, c), 17);
            assert_eq!(
                cram_cram_codecs_c_1776_cram_xdelta_get_block(s, c),
                block_ptr
            );
        }
    }

    #[test]
    fn cram_codecs_xdelta_decode_int_and_block_apply_zigzag_deltas() {
        unsafe {
            let mut sub_values = [2u32, 1, 4];
            let mut sub_value_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: sub_values.as_mut_ptr().cast(),
                alloc: std::mem::size_of_val(&sub_values),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut sub_codec = cram_codec_xdelta_layout {
                codec: 0,
                out: (&mut sub_value_block as *mut cram_block_layout).cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: cram_fn_ptr(test_xdelta_decode_u32 as usize),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xdelta: cram_xdelta_decoder_layout {
                    last: 0,
                    word_size: 0,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: std::ptr::null_mut(),
                },
            };
            let mut codec = cram_codec_xdelta_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xdelta: cram_xdelta_decoder_layout {
                    last: 0,
                    word_size: 2,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: (&mut sub_codec as *mut cram_codec_xdelta_layout).cast(),
                },
            };
            let mut out_size = 3;
            let mut out = [0u32; 3];
            assert_eq!(
                cram_cram_codecs_c_1688_cram_xdelta_decode_int(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_xdelta_layout).cast(),
                    (&mut sub_value_block as *mut cram_block_layout).cast(),
                    out.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(out, [1, 0, 2]);

            let mut varint_data = [2u8, 4u8];
            let mut varint_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: varint_data.len() as i32,
                crc32: 0,
                idx: 0,
                data: varint_data.as_mut_ptr(),
                alloc: varint_data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            sub_codec.out = (&mut varint_block as *mut cram_block_layout).cast();
            sub_codec.get_block = cram_fn_ptr(test_xdelta_get_block as usize);
            codec.xdelta.sub_codec = (&mut sub_codec as *mut cram_codec_xdelta_layout).cast();
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: None,
                varint_get64: None,
                varint_get64s: None,
                varint_put32: None,
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: None,
                varint_put32s_blk: None,
                varint_put64_blk: None,
                varint_put64s_blk: None,
                varint_size: None,
            };
            codec.vv = &mut vv;
            codec.xdelta.last = 99;
            let out_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            out_size = 4;
            assert_eq!(
                cram_cram_codecs_c_1719_cram_xdelta_decode_block(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_xdelta_layout).cast(),
                    std::ptr::null_mut(),
                    out_block.cast(),
                    &mut out_size
                ),
                0
            );
            let out_block_layout = out_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*out_block_layout).data, (*out_block_layout).byte),
                &[1, 0, 3, 0]
            );
            cram_cram_io_c_1565_cram_free_block(out_block);

            let mut full_vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut hdr: cram_block_compression_hdr_layout = std::mem::zeroed();
            let mut xdelta_header = [2u8, 1, 1, 9];
            let dec = cram_cram_codecs_c_3872_cram_decoder_init(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                53,
                xdelta_header.as_mut_ptr().cast(),
                xdelta_header.len() as c_int,
                5,
                3 << 8,
                (&mut full_vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_xdelta_layout>();
            assert!(!dec.is_null());
            assert_eq!(hdr.ncodecs, 2);
            assert_eq!((*dec).codec, 53);
            assert_eq!((*dec).codec_id, 1);
            assert_eq!((*dec).xdelta.word_size, 2);
            assert_eq!(
                (*dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_1719_cram_xdelta_decode_block as usize)
            );
            assert_eq!(
                (*(*dec).xdelta.sub_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                9
            );
            cram_cram_codecs_c_1762_cram_xdelta_decode_free(dec.cast());

            let sub_out = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut sub_enc = cram_codec_xdelta_layout {
                codec: 0,
                out: sub_out.cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: cram_fn_ptr(test_byte_array_len_encode_val as usize),
                store: cram_fn_ptr(test_byte_array_len_store_val as usize),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xdelta: cram_xdelta_decoder_layout {
                    last: 0,
                    word_size: 0,
                    sub_encoding: 0,
                    sub_codec_dat: std::ptr::null_mut(),
                    sub_codec: std::ptr::null_mut(),
                },
            };
            codec.codec = 53;
            codec.vv = &mut full_vv;
            codec.xdelta.word_size = 2;
            codec.xdelta.sub_codec = (&mut sub_enc as *mut cram_codec_xdelta_layout).cast();
            let mut delta_payload = [1u8, 0, 3, 0];
            assert_eq!(
                cram_cram_codecs_c_1976_cram_xdelta_encode_char(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_xdelta_layout).cast(),
                    delta_payload.as_mut_ptr().cast(),
                    delta_payload.len() as c_int,
                ),
                0
            );
            let sub_out_layout = sub_out.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*sub_out_layout).data, (*sub_out_layout).byte),
                &[2, 4]
            );

            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_1930_cram_xdelta_encode_store(
                    (&mut codec as *mut cram_codec_xdelta_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                6
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 53, 3, 2, b'V', b'A']
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            let mut xdelta_dat = cram_xdelta_decoder_layout {
                last: 0,
                word_size: 2,
                sub_encoding: 1,
                sub_codec_dat: test_void_token(9),
                sub_codec: std::ptr::null_mut(),
            };
            let enc = cram_cram_codecs_c_3928_cram_encoder_init(
                53,
                std::ptr::null_mut(),
                4,
                (&mut xdelta_dat as *mut cram_xdelta_decoder_layout).cast(),
                3 << 8,
                (&mut full_vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_xdelta_layout>();
            assert!(!enc.is_null());
            assert_eq!((*enc).codec, 53);
            assert_eq!(
                (*enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_1976_cram_xdelta_encode_char as usize)
            );
            assert_eq!(
                (*enc).store,
                cram_fn_ptr(cram_cram_codecs_c_1930_cram_xdelta_encode_store as usize)
            );
            assert_eq!(
                (*enc).flush,
                cram_fn_ptr(cram_cram_codecs_c_1835_cram_xdelta_encode_flush as usize)
            );
            assert_eq!(
                (*(*enc).xdelta.sub_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                9
            );
            cram_cram_codecs_c_2011_cram_xdelta_encode_free(enc.cast());
            cram_cram_io_c_1565_cram_free_block(sub_out);
        }
    }

    #[test]
    fn cram_codecs_xrle_placeholders_and_encode_char_buffer_like_c() {
        unsafe {
            assert_eq!(
                cram_cram_codecs_c_2063_cram_xrle_decode_long(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -1
            );
            assert_eq!(
                cram_cram_codecs_c_2068_cram_xrle_decode_int(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -1
            );
            assert_eq!(
                cram_cram_codecs_c_2359_cram_xrle_encode_long(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0
                ),
                -1
            );
            assert_eq!(
                cram_cram_codecs_c_2365_cram_xrle_encode_int(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0
                ),
                -1
            );

            let mut codec = cram_codec_xrle_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xrle: cram_xrle_decoder_layout {
                    len_encoding: 0,
                    lit_encoding: 0,
                    len_dat: std::ptr::null_mut(),
                    lit_dat: std::ptr::null_mut(),
                    len_codec: std::ptr::null_mut(),
                    lit_codec: std::ptr::null_mut(),
                    cur_len: 0,
                    cur_lit: 0,
                    rep_score: [0; 256],
                    to_flush: std::ptr::null_mut(),
                    to_flush_size: 0,
                },
            };
            let c = (&mut codec as *mut cram_codec_xrle_layout).cast();
            let mut first = *b"abc";
            let mut second = *b"def";
            let mut third = *b"g";

            assert_eq!(
                cram_cram_codecs_c_2371_cram_xrle_encode_char(
                    std::ptr::null_mut(),
                    c,
                    first.as_mut_ptr().cast(),
                    first.len() as c_int
                ),
                0
            );
            assert!(codec.out.is_null());
            assert_eq!(codec.xrle.to_flush, first.as_mut_ptr().cast());
            assert_eq!(codec.xrle.to_flush_size, first.len());

            assert_eq!(
                cram_cram_codecs_c_2371_cram_xrle_encode_char(
                    std::ptr::null_mut(),
                    c,
                    second.as_mut_ptr().cast(),
                    second.len() as c_int
                ),
                0
            );
            assert!(!codec.out.is_null());
            let out = codec.out.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*out).data, (*out).byte),
                b"abcdef"
            );

            assert_eq!(
                cram_cram_codecs_c_2371_cram_xrle_encode_char(
                    std::ptr::null_mut(),
                    c,
                    third.as_mut_ptr().cast(),
                    third.len() as c_int
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts((*out).data, (*out).byte),
                b"abcdefg"
            );
            cram_cram_io_c_1565_cram_free_block(codec.out.cast());

            let mut len_payload = [6u8, 2, 1];
            let mut lit_payload = *b"abc";
            let mut len_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: len_payload.len() as i32,
                crc32: 0,
                idx: 0,
                data: len_payload.as_mut_ptr(),
                alloc: len_payload.len(),
                byte: len_payload.len(),
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut lit_block = cram_block_layout {
                data: lit_payload.as_mut_ptr(),
                uncomp_size: lit_payload.len() as i32,
                byte: lit_payload.len(),
                alloc: lit_payload.len(),
                ..len_block
            };
            let mut len_codec = cram_codec_xrle_layout {
                get_block: cram_fn_ptr(test_xrle_get_block as usize),
                size: cram_fn_ptr(test_xrle_size as usize),
                out: (&mut len_block as *mut cram_block_layout).cast(),
                ..codec
            };
            let mut lit_codec = cram_codec_xrle_layout {
                get_block: cram_fn_ptr(test_xrle_get_block as usize),
                out: (&mut lit_block as *mut cram_block_layout).cast(),
                ..codec
            };
            let mut xrle = cram_codec_xrle_layout {
                codec_id: 3,
                xrle: cram_xrle_decoder_layout {
                    len_codec: (&mut len_codec as *mut cram_codec_xrle_layout).cast(),
                    lit_codec: (&mut lit_codec as *mut cram_codec_xrle_layout).cast(),
                    rep_score: {
                        let mut s = [0; 256];
                        s[b'a' as usize] = 1;
                        s[b'b' as usize] = 1;
                        s
                    },
                    ..codec.xrle
                },
                ..codec
            };
            let mut by_id = [std::ptr::null_mut(); 520];
            let mut slice = cram_slice_layout {
                hdr: std::ptr::null_mut(),
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: by_id.as_mut_ptr(),
                ..std::mem::zeroed()
            };
            let s = (&mut slice as *mut cram_slice_layout).cast();
            let x = (&mut xrle as *mut cram_codec_xrle_layout).cast();
            assert_eq!(cram_cram_codecs_c_2115_cram_xrle_decode_size(s, x), 6);
            let cached = by_id[515].cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*cached).data, (*cached).uncomp_size as usize),
                b"aaabbc"
            );
            let mut decoded = [0u8; 4];
            let mut out_size = decoded.len() as c_int;
            assert_eq!(
                cram_cram_codecs_c_2125_cram_xrle_decode_char(
                    s,
                    x,
                    std::ptr::null_mut(),
                    decoded.as_mut_ptr().cast(),
                    &mut out_size,
                ),
                0
            );
            assert_eq!(&decoded, b"aaab");
            assert_eq!((*cached).idx, 4);
            assert_eq!(
                cram_cram_codecs_c_2120_cram_xrle_get_block(s, x),
                by_id[515].cast()
            );
            cram_cram_io_c_1565_cram_free_block(by_id[515].cast());

            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let len_out = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let lit_out = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut len_enc = cram_codec_byte_array_len_layout {
                out: len_out.cast(),
                encode: cram_fn_ptr(test_byte_array_len_encode_val as usize),
                store: cram_fn_ptr(test_byte_array_len_store_len as usize),
                ..std::mem::zeroed()
            };
            let mut lit_enc = cram_codec_byte_array_len_layout {
                out: lit_out.cast(),
                encode: cram_fn_ptr(test_byte_array_len_encode_val as usize),
                store: cram_fn_ptr(test_byte_array_len_store_val as usize),
                ..std::mem::zeroed()
            };
            let mut flush_input = *b"aaabbc";
            let mut enc_codec = cram_codec_xrle_layout {
                codec: 52,
                vv: &mut vv as *mut varint_vec_layout,
                xrle: cram_xrle_decoder_layout {
                    len_codec: (&mut len_enc as *mut cram_codec_byte_array_len_layout).cast(),
                    lit_codec: (&mut lit_enc as *mut cram_codec_byte_array_len_layout).cast(),
                    to_flush: flush_input.as_mut_ptr().cast(),
                    to_flush_size: flush_input.len(),
                    rep_score: {
                        let mut s = [0; 256];
                        s[b'a' as usize] = 1;
                        s[b'b' as usize] = 1;
                        s
                    },
                    ..codec.xrle
                },
                ..codec
            };
            assert_eq!(
                cram_cram_codecs_c_2257_cram_xrle_encode_flush(
                    (&mut enc_codec as *mut cram_codec_xrle_layout).cast()
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(
                    (*len_out.cast::<cram_block_layout>()).data,
                    (*len_out.cast::<cram_block_layout>()).byte,
                ),
                &[6, 2, 1]
            );
            assert_eq!(
                std::slice::from_raw_parts(
                    (*lit_out.cast::<cram_block_layout>()).data,
                    (*lit_out.cast::<cram_block_layout>()).byte,
                ),
                b"abc"
            );

            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_2303_cram_xrle_encode_store(
                    (&mut enc_codec as *mut cram_codec_xrle_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                9
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 52, 6, 2, b'a', b'b', b'L', b'V', b'A']
            );
            cram_cram_io_c_1565_cram_free_block(store_block);
            cram_cram_io_c_1565_cram_free_block(len_out);
            cram_cram_io_c_1565_cram_free_block(lit_out);

            let mut hdr = cram_block_compression_hdr_layout {
                ref_seq_id: 0,
                ref_seq_start: 0,
                ref_seq_span: 0,
                num_records: 0,
                num_landmarks: 0,
                landmark: std::ptr::null_mut(),
                read_names_included: 0,
                ap_delta: 0,
                substitution_matrix: [[0; 4]; 5],
                no_ref: 0,
                qs_seq_orient: 0,
                td_blk: std::ptr::null_mut(),
                ntl: 0,
                tl: std::ptr::null_mut(),
                td_hash: std::ptr::null_mut(),
                td_keys: std::ptr::null_mut(),
                preservation_map: std::ptr::null_mut(),
                rec_encoding_map: [std::ptr::null_mut(); 32],
                tag_encoding_map: [std::ptr::null_mut(); 32],
                codecs: [std::ptr::null_mut(); 47],
                uncomp: std::ptr::null_mut(),
                uncomp_size: 0,
                uncomp_alloc: 0,
                ncodecs: 0,
            };
            let mut header = [2u8, b'a', b'b', 1, 1, 7, 1, 1, 9];
            let dec = cram_cram_codecs_c_3872_cram_decoder_init(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                52,
                header.as_mut_ptr().cast(),
                header.len() as c_int,
                4,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_xrle_layout>();
            assert!(!dec.is_null());
            assert_eq!((*dec).codec, 52);
            assert_eq!((*dec).xrle.rep_score[b'a' as usize], 1);
            assert_eq!((*dec).xrle.rep_score[b'b' as usize], 1);
            assert_eq!(
                (*dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_2125_cram_xrle_decode_char as usize)
            );
            assert_eq!(
                (*(*dec).xrle.len_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                7
            );
            assert_eq!(
                (*(*dec).xrle.lit_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                9
            );
            cram_cram_codecs_c_2172_cram_xrle_decode_free(dec.cast());

            let mut enc_dat = cram_xrle_decoder_layout {
                len_encoding: 1,
                lit_encoding: 1,
                len_dat: test_void_token(7),
                lit_dat: test_void_token(9),
                rep_score: {
                    let mut s = [0; 256];
                    s[b'a' as usize] = 1;
                    s
                },
                ..codec.xrle
            };
            let enc = cram_cram_codecs_c_3928_cram_encoder_init(
                52,
                std::ptr::null_mut(),
                4,
                (&mut enc_dat as *mut cram_xrle_decoder_layout).cast(),
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_xrle_layout>();
            assert!(!enc.is_null());
            assert_eq!((*enc).codec, 52);
            assert_eq!(
                (*enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_2371_cram_xrle_encode_char as usize)
            );
            assert_eq!(
                (*enc).flush,
                cram_fn_ptr(cram_cram_codecs_c_2257_cram_xrle_encode_flush as usize)
            );
            assert_eq!((*enc).xrle.rep_score[b'a' as usize], 1);
            assert_eq!(
                (*(*enc).xrle.len_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                7
            );
            assert_eq!(
                (*(*enc).xrle.lit_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                9
            );
            cram_cram_codecs_c_2396_cram_xrle_encode_free(enc.cast());
        }
    }

    #[test]
    fn cram_codecs_subexp_and_gamma_decode_bitstreams() {
        unsafe {
            let mut subexp_data = [0b0101_0010u8];
            let mut subexp_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: subexp_data.len() as i32,
                crc32: 0,
                idx: 0,
                data: subexp_data.as_mut_ptr(),
                alloc: subexp_data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut subexp = cram_codec_subexp_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                subexp: cram_subexp_decoder_layout { offset: 1, k: 2 },
            };
            let mut out_size = 2;
            let mut subexp_out = [0i32; 2];
            assert_eq!(
                cram_cram_codecs_c_2452_cram_subexp_decode(
                    std::ptr::null_mut(),
                    (&mut subexp as *mut cram_codec_subexp_layout).cast(),
                    (&mut subexp_block as *mut cram_block_layout).cast(),
                    subexp_out.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(subexp_out, [1, 4]);

            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_2501_cram_subexp_describe(
                    (&mut subexp as *mut cram_codec_subexp_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"SUBEXP(offset=1,k=2)"
            );
            free(ks.s.cast());

            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut subexp_header = [1u8, 2u8];
            let subexp_init = cram_cram_codecs_c_2508_cram_subexp_decode_init(
                std::ptr::null_mut(),
                subexp_header.as_mut_ptr().cast(),
                subexp_header.len() as c_int,
                7,
                1,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_subexp_layout>();
            assert!(!subexp_init.is_null());
            assert_eq!((*subexp_init).subexp.offset, 1);
            assert_eq!((*subexp_init).subexp.k, 2);
            assert_eq!(
                (*subexp_init).decode,
                cram_fn_ptr(cram_cram_codecs_c_2452_cram_subexp_decode as usize)
            );
            cram_cram_codecs_c_2496_cram_subexp_decode_free(subexp_init.cast());

            let mut gamma_data = [0b0010_1000u8];
            let mut gamma_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: gamma_data.len() as i32,
                crc32: 0,
                idx: 0,
                data: gamma_data.as_mut_ptr(),
                alloc: gamma_data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut gamma = cram_codec_gamma_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                gamma: cram_gamma_decoder_layout { offset: 1 },
            };
            out_size = 1;
            let mut gamma_out = [0i32; 1];
            assert_eq!(
                cram_cram_codecs_c_2546_cram_gamma_decode(
                    std::ptr::null_mut(),
                    (&mut gamma as *mut cram_codec_gamma_layout).cast(),
                    (&mut gamma_block as *mut cram_block_layout).cast(),
                    gamma_out.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(gamma_out, [4]);

            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_2575_cram_gamma_describe(
                    (&mut gamma as *mut cram_codec_gamma_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"GAMMA(offset=1)"
            );
            free(ks.s.cast());

            let mut gamma_header = [3u8];
            let gamma_init = cram_cram_codecs_c_2580_cram_gamma_decode_init(
                std::ptr::null_mut(),
                gamma_header.as_mut_ptr().cast(),
                gamma_header.len() as c_int,
                9,
                1,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_gamma_layout>();
            assert!(!gamma_init.is_null());
            assert_eq!((*gamma_init).gamma.offset, 3);
            assert_eq!(
                (*gamma_init).decode,
                cram_fn_ptr(cram_cram_codecs_c_2546_cram_gamma_decode as usize)
            );
            cram_cram_codecs_c_2570_cram_gamma_decode_free(gamma_init.cast());
        }
    }

    #[test]
    fn cram_codecs_huffman_decode_simple_codes_and_zero_len_cases() {
        unsafe {
            let a = cram_huffman_code_layout {
                symbol: b'A' as i64,
                p: 0,
                code: 0,
                len: 1,
            };
            let b_code = cram_huffman_code_layout {
                symbol: b'B' as i64,
                p: 0,
                code: 1,
                len: 1,
            };
            assert!(
                cram_cram_codecs_c_2622_code_sort(
                    (&a as *const cram_huffman_code_layout).cast(),
                    (&b_code as *const cram_huffman_code_layout).cast()
                ) < 0
            );

            let mut codes = [a, b_code];
            let mut codec = cram_codec_huffman_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                huffman: cram_huffman_decoder_layout {
                    ncodes: 2,
                    codes: codes.as_mut_ptr(),
                    option: 0,
                },
            };
            let c = (&mut codec as *mut cram_codec_huffman_layout).cast();
            let mut data = [0b0100_0000u8];
            let mut block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: data.len() as i32,
                crc32: 0,
                idx: 0,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let block_ptr = (&mut block as *mut cram_block_layout).cast();
            let mut out_size = 3;
            let mut chars = [0 as c_char; 3];
            assert_eq!(
                cram_cram_codecs_c_2660_cram_huffman_decode_char(
                    std::ptr::null_mut(),
                    c,
                    block_ptr,
                    chars.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(chars.as_ptr().cast::<u8>(), 3),
                b"ABA"
            );

            block.byte = 0;
            block.bit = 7;
            let block_ptr = (&mut block as *mut cram_block_layout).cast();
            let mut ints = [0i32; 3];
            assert_eq!(
                cram_cram_codecs_c_2708_cram_huffman_decode_int(
                    std::ptr::null_mut(),
                    c,
                    block_ptr,
                    ints.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(ints, [b'A' as i32, b'B' as i32, b'A' as i32]);

            block.byte = 0;
            block.bit = 7;
            let block_ptr = (&mut block as *mut cram_block_layout).cast();
            let mut longs = [0i64; 3];
            assert_eq!(
                cram_cram_codecs_c_2758_cram_huffman_decode_long(
                    std::ptr::null_mut(),
                    c,
                    block_ptr,
                    longs.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(longs, [b'A' as i64, b'B' as i64, b'A' as i64]);

            let mut zero_code = [cram_huffman_code_layout {
                symbol: b'Z' as i64,
                p: 0,
                code: 0,
                len: 0,
            }];
            codec.huffman.ncodes = 1;
            codec.huffman.codes = zero_code.as_mut_ptr();
            let mut zero_chars = [0 as c_char; 2];
            out_size = 2;
            assert_eq!(
                cram_cram_codecs_c_2646_cram_huffman_decode_char0(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_huffman_layout).cast(),
                    std::ptr::null_mut(),
                    zero_chars.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(zero_chars.as_ptr().cast::<u8>(), 2),
                b"ZZ"
            );
            let mut zero_ints = [0i32; 2];
            assert_eq!(
                cram_cram_codecs_c_2695_cram_huffman_decode_int0(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_huffman_layout).cast(),
                    std::ptr::null_mut(),
                    zero_ints.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(zero_ints, [b'Z' as i32; 2]);
            let mut zero_longs = [0i64; 2];
            assert_eq!(
                cram_cram_codecs_c_2745_cram_huffman_decode_long0(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_huffman_layout).cast(),
                    std::ptr::null_mut(),
                    zero_longs.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(zero_longs, [b'Z' as i64; 2]);
            assert_eq!(
                cram_cram_codecs_c_2641_cram_huffman_decode_null(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut out_size
                ),
                -1
            );

            codec.huffman.ncodes = 2;
            codec.huffman.codes = codes.as_mut_ptr();
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_2795_cram_huffman_describe(
                    (&mut codec as *mut cram_codec_huffman_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"HUFFMAN(codes={65,66},lengths={1,1})"
            );
            free(ks.s.cast());

            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut huff_header = [2u8, b'A', b'B', 2, 1, 1];
            let huff_init = cram_cram_codecs_c_2814_cram_huffman_decode_init(
                std::ptr::null_mut(),
                huff_header.as_mut_ptr().cast(),
                huff_header.len() as c_int,
                3,
                3,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_huffman_layout>();
            assert!(!huff_init.is_null());
            assert_eq!((*huff_init).huffman.ncodes, 2);
            assert_eq!(
                (*huff_init).decode,
                cram_fn_ptr(cram_cram_codecs_c_2660_cram_huffman_decode_char as usize)
            );
            assert_eq!((*(*huff_init).huffman.codes.add(0)).symbol, b'A' as i64);
            assert_eq!((*(*huff_init).huffman.codes.add(0)).code, 0);
            assert_eq!((*(*huff_init).huffman.codes.add(1)).symbol, b'B' as i64);
            assert_eq!((*(*huff_init).huffman.codes.add(1)).code, 1);
            cram_cram_codecs_c_2632_cram_huffman_decode_free(huff_init.cast());
        }
    }

    #[test]
    fn cram_codecs_huffman_encode_simple_codes_and_zero_len_noops() {
        unsafe {
            let mut codes = [
                cram_huffman_code_layout {
                    symbol: b'A' as i64,
                    p: 0,
                    code: 0,
                    len: 1,
                },
                cram_huffman_code_layout {
                    symbol: b'B' as i64,
                    p: 0,
                    code: 1,
                    len: 1,
                },
                cram_huffman_code_layout {
                    symbol: 300,
                    p: 0,
                    code: 0b10,
                    len: 2,
                },
            ];
            let mut val2code = [0i32; 129];
            val2code[(b'A' + 1) as usize] = 0;
            val2code[(b'B' + 1) as usize] = 1;
            let out = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut codec = cram_codec_huffman_encoder_layout {
                codec: 0,
                out: out.cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                huffman: cram_huffman_encoder_layout {
                    codes: codes.as_mut_ptr(),
                    nvals: codes.len() as c_int,
                    val2code,
                    option: 0,
                },
            };
            let c = (&mut codec as *mut cram_codec_huffman_encoder_layout).cast();

            assert_eq!(
                cram_cram_codecs_c_2989_cram_huffman_encode_char0(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    0
                ),
                0
            );
            let mut chars = *b"ABA";
            assert_eq!(
                cram_cram_codecs_c_2994_cram_huffman_encode_char(
                    std::ptr::null_mut(),
                    c,
                    chars.as_mut_ptr().cast(),
                    chars.len() as c_int
                ),
                0
            );
            let out_block = out.cast::<cram_block_layout>();
            assert_eq!(
                ((*out_block).byte, (*out_block).bit, *(*out_block).data),
                (0, 4, 0b0100_0000)
            );

            (*out_block).byte = 0;
            (*out_block).bit = 7;
            *(*out_block).data = 0;
            let mut ints = [b'B' as i32, b'A' as i32, 300i32];
            assert_eq!(
                cram_cram_codecs_c_3025_cram_huffman_encode_int0(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    0
                ),
                0
            );
            assert_eq!(
                cram_cram_codecs_c_3030_cram_huffman_encode_int(
                    std::ptr::null_mut(),
                    c,
                    ints.as_mut_ptr().cast(),
                    ints.len() as c_int
                ),
                0
            );
            assert_eq!(
                ((*out_block).byte, (*out_block).bit, *(*out_block).data),
                (0, 3, 0b1010_0000)
            );

            (*out_block).byte = 0;
            (*out_block).bit = 7;
            *(*out_block).data = 0;
            let mut longs = [300i64, b'A' as i64];
            assert_eq!(
                cram_cram_codecs_c_3062_cram_huffman_encode_long0(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    0
                ),
                0
            );
            assert_eq!(
                cram_cram_codecs_c_3067_cram_huffman_encode_long(
                    std::ptr::null_mut(),
                    c,
                    longs.as_mut_ptr().cast(),
                    longs.len() as c_int
                ),
                0
            );
            assert_eq!(
                ((*out_block).byte, (*out_block).bit, *(*out_block).data),
                (0, 4, 0b1000_0000)
            );

            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            codec.vv = &mut vv as *mut varint_vec_layout;
            codec.codec = 3;
            codec.huffman.option = 3;
            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_3112_cram_huffman_encode_store(
                    (&mut codec as *mut cram_codec_huffman_encoder_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                11
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 3, 8, 3, 65, 66, 44, 3, 1, 1, 2]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            let mut st = cram_stats_layout {
                freqs: [0; 1024],
                h: std::ptr::null_mut(),
                nsamp: 0,
                nvals: 3,
                min_val: 0,
                max_val: 0,
            };
            st.freqs[b'A' as usize] = 5;
            st.freqs[b'B' as usize] = 2;
            st.freqs[300] = 1;
            let init = cram_cram_codecs_c_3176_cram_huffman_encode_init(
                (&mut st as *mut cram_stats_layout).cast(),
                3,
                1,
                std::ptr::null_mut(),
                3 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_huffman_encoder_layout>();
            assert!(!init.is_null());
            assert_eq!((*init).codec, 3);
            assert_eq!((*init).huffman.nvals, 3);
            assert_eq!(
                (*init).encode,
                cram_fn_ptr(cram_cram_codecs_c_3030_cram_huffman_encode_int as usize)
            );
            assert_eq!(
                (*init).store,
                cram_fn_ptr(cram_cram_codecs_c_3112_cram_huffman_encode_store as usize)
            );
            assert_eq!((*(*init).huffman.codes.add(0)).symbol, b'A' as i64);
            assert_eq!((*(*init).huffman.codes.add(0)).len, 1);
            assert_eq!((*(*init).huffman.codes.add(0)).code, 0);
            assert_eq!((*(*init).huffman.codes.add(1)).symbol, b'B' as i64);
            assert_eq!((*(*init).huffman.codes.add(1)).len, 2);
            assert_eq!((*(*init).huffman.codes.add(1)).code, 2);
            assert_eq!((*(*init).huffman.codes.add(2)).symbol, 300);
            assert_eq!((*(*init).huffman.codes.add(2)).len, 2);
            assert_eq!((*(*init).huffman.codes.add(2)).code, 3);
            assert_eq!((*init).huffman.val2code[(b'A' + 1) as usize], 0);
            assert_eq!((*init).huffman.val2code[(b'B' + 1) as usize], 1);
            cram_cram_codecs_c_3099_cram_huffman_encode_free(init.cast());
            cram_cram_io_c_1565_cram_free_block(out);
        }
    }

    #[test]
    fn cram_codecs_byte_array_len_decode_and_encode_delegate_to_children() {
        unsafe {
            let mut len_codec = cram_codec_byte_array_len_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 3,
                free: std::ptr::null_mut(),
                decode: cram_fn_ptr(test_byte_array_len_decode_len as usize),
                encode: cram_fn_ptr(test_byte_array_len_encode_len as usize),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: std::ptr::null_mut(),
                    val_codec: std::ptr::null_mut(),
                },
            };
            let out_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            len_codec.out = out_block.cast();
            let mut val_codec = cram_codec_byte_array_len_layout {
                codec: 0,
                out: out_block.cast(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: cram_fn_ptr(test_byte_array_len_decode_val as usize),
                encode: cram_fn_ptr(test_byte_array_len_encode_val as usize),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: std::ptr::null_mut(),
                    val_codec: std::ptr::null_mut(),
                },
            };
            let mut codec = cram_codec_byte_array_len_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: (&mut len_codec as *mut cram_codec_byte_array_len_layout).cast(),
                    val_codec: (&mut val_codec as *mut cram_codec_byte_array_len_layout).cast(),
                },
            };
            let c = (&mut codec as *mut cram_codec_byte_array_len_layout).cast();
            let mut out = [0 as c_char; 8];
            let mut out_size = out.len() as c_int;
            assert_eq!(
                cram_cram_codecs_c_3371_cram_byte_array_len_decode(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    out.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(out_size, 3);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 3),
                b"XYZ"
            );

            (*(codec
                .byte_array_len
                .len_codec
                .cast::<cram_codec_byte_array_len_layout>()))
            .codec_id = 9;
            out_size = 3;
            assert_eq!(
                cram_cram_codecs_c_3371_cram_byte_array_len_decode(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    out.as_mut_ptr(),
                    &mut out_size
                ),
                -1
            );

            let mut payload = *b"rust";
            assert_eq!(
                cram_cram_codecs_c_3479_cram_byte_array_len_encode(
                    std::ptr::null_mut(),
                    c,
                    payload.as_mut_ptr().cast(),
                    payload.len() as c_int
                ),
                0
            );
            let out_layout = out_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*out_layout).data, (*out_layout).byte),
                b"\x04rust"
            );

            len_codec.store = cram_fn_ptr(test_byte_array_len_store_len as usize);
            val_codec.store = cram_fn_ptr(test_byte_array_len_store_val as usize);
            codec.codec = 4;
            assert!(!len_codec.store.is_null());
            assert!(!val_codec.store.is_null());
            assert_eq!(codec.codec, 4);
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_xdelta_varint_get32),
                varint_get32s: Some(test_xdelta_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            codec.vv = &mut vv as *mut varint_vec_layout;
            assert!(!codec.vv.is_null());
            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_3506_cram_byte_array_len_encode_store(
                    c,
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                6
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 4, 3, b'L', b'V', b'A']
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            let mut hdr: cram_block_compression_hdr_layout = std::mem::zeroed();
            let mut init_header = [1u8, 1, 7, 1, 1, 8];
            let dec = cram_cram_codecs_c_3872_cram_decoder_init(
                (&mut hdr as *mut cram_block_compression_hdr_layout).cast(),
                4,
                init_header.as_mut_ptr().cast(),
                init_header.len() as c_int,
                3,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_byte_array_len_layout>();
            assert!(!dec.is_null());
            assert_eq!(hdr.ncodecs, 3);
            assert_eq!((*dec).codec_id, 2);
            assert_eq!((*dec).codec, 4);
            assert_eq!(
                (*dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_3371_cram_byte_array_len_decode as usize)
            );
            let dec_len = (*dec)
                .byte_array_len
                .len_codec
                .cast::<cram_codec_external_layout>();
            let dec_val = (*dec)
                .byte_array_len
                .val_codec
                .cast::<cram_codec_external_layout>();
            assert_eq!((*dec_len).codec_id, 0);
            assert_eq!((*dec_len).external.content_id, 7);
            assert_eq!((*dec_val).codec_id, 1);
            assert_eq!((*dec_val).external.content_id, 8);
            cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(dec.cast());

            let mut enc_st = cram_stats_layout {
                freqs: [0; 1024],
                h: std::ptr::null_mut(),
                nsamp: 0,
                nvals: 1,
                min_val: 0,
                max_val: 0,
            };
            let mut enc_dat = cram_byte_array_len_encoder_dat_layout {
                len_encoding: 1,
                val_encoding: 1,
                len_dat: test_void_token(7),
                val_dat: test_void_token(8),
                len_codec: std::ptr::null_mut(),
                val_codec: std::ptr::null_mut(),
            };
            let enc = cram_cram_codecs_c_3928_cram_encoder_init(
                4,
                (&mut enc_st as *mut cram_stats_layout).cast(),
                4,
                (&mut enc_dat as *mut cram_byte_array_len_encoder_dat_layout).cast(),
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_byte_array_len_layout>();
            assert!(!enc.is_null());
            assert_eq!((*enc).codec, 4);
            assert_eq!(
                (*enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_3479_cram_byte_array_len_encode as usize)
            );
            assert_eq!(
                (*enc).store,
                cram_fn_ptr(cram_cram_codecs_c_3506_cram_byte_array_len_encode_store as usize)
            );
            let enc_len = (*enc)
                .byte_array_len
                .len_codec
                .cast::<cram_codec_external_layout>();
            let enc_val = (*enc)
                .byte_array_len
                .val_codec
                .cast::<cram_codec_external_layout>();
            assert_eq!((*enc_len).external.content_id, 7);
            assert_eq!((*enc_len).external.type_, 1);
            assert_eq!((*enc_val).external.content_id, 8);
            assert_eq!((*enc_val).external.type_, 4);
            cram_cram_codecs_c_3493_cram_byte_array_len_encode_free(enc.cast());
            cram_cram_io_c_1565_cram_free_block(out_block);
        }
    }

    #[test]
    fn cram_codecs_byte_array_stop_decode_and_encode_stop_delimited_values() {
        unsafe {
            let mut data = *b"abc,def,";
            let mut input_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: data.len() as i32,
                crc32: 0,
                idx: 0,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 7,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let input_ptr = (&mut input_block as *mut cram_block_layout).cast::<cram_block>();
            let mut by_id = [input_ptr];
            let mut slice = cram_slice_layout {
                hdr: std::ptr::null_mut(),
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: by_id.as_mut_ptr().cast(),
                ..std::mem::zeroed()
            };
            let mut codec = cram_codec_byte_array_stop_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                byte_array_stop: cram_byte_array_stop_decoder_layout {
                    stop: b',',
                    content_id: 0,
                },
            };
            let s = (&mut slice as *mut cram_slice_layout).cast();
            let c = (&mut codec as *mut cram_codec_byte_array_stop_layout).cast();
            let mut out = [0 as c_char; 8];
            let mut out_size = 8;
            assert_eq!(
                cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char(
                    s,
                    c,
                    std::ptr::null_mut(),
                    out.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(out_size, 3);
            assert_eq!(input_block.idx, 4);
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 3),
                b"abc"
            );

            out_size = 8;
            assert_eq!(
                cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char(
                    s,
                    c,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(out_size, 3);
            assert_eq!(input_block.idx, 8);

            input_block.idx = 0;
            assert_eq!(input_block.idx, 0);
            let out_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            out_size = 8;
            assert_eq!(
                cram_cram_codecs_c_3626_cram_byte_array_stop_decode_block(
                    s,
                    c,
                    std::ptr::null_mut(),
                    out_block.cast(),
                    &mut out_size
                ),
                0
            );
            let out_layout = out_block.cast::<cram_block_layout>();
            assert_eq!(out_size, 3);
            assert_eq!(
                std::slice::from_raw_parts((*out_layout).data, (*out_layout).byte),
                b"abc"
            );
            cram_cram_io_c_1565_cram_free_block(out_block);

            let enc_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            codec.out = enc_block.cast();
            let mut payload = *b"xy";
            assert_eq!(
                cram_cram_codecs_c_3733_cram_byte_array_stop_encode(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_byte_array_stop_layout).cast(),
                    payload.as_mut_ptr().cast(),
                    payload.len() as c_int
                ),
                0
            );
            let enc_layout = enc_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*enc_layout).data, (*enc_layout).byte),
                b"xy,"
            );
            cram_cram_io_c_1565_cram_free_block(enc_block);
        }
    }

    #[test]
    fn cram_codecs_byte_array_describe_init_and_encoding_names_match_c() {
        unsafe {
            assert_eq!(
                std::ffi::CStr::from_ptr(cram_cram_codecs_c_3811_cram_encoding2str(4)).to_bytes(),
                b"BYTE_ARRAY_LEN"
            );
            assert_eq!(
                std::ffi::CStr::from_ptr(cram_cram_codecs_c_3811_cram_encoding2str(53)).to_bytes(),
                b"?"
            );

            let mut len_codec = cram_codec_byte_array_len_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: cram_fn_ptr(test_codec_describe_len as usize),
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: std::ptr::null_mut(),
                    val_codec: std::ptr::null_mut(),
                },
            };
            let mut val_codec = cram_codec_byte_array_len_layout {
                describe: cram_fn_ptr(test_codec_describe_val as usize),
                ..len_codec
            };
            let mut bal_codec = cram_codec_byte_array_len_layout {
                describe: std::ptr::null_mut(),
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: (&mut len_codec as *mut cram_codec_byte_array_len_layout).cast(),
                    val_codec: (&mut val_codec as *mut cram_codec_byte_array_len_layout).cast(),
                },
                ..len_codec
            };
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_3412_cram_byte_array_len_describe(
                    (&mut bal_codec as *mut cram_codec_byte_array_len_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"BYTE_ARRAY_LEN(len_codec={LEN},val_codec={VAL}"
            );
            free(ks.s.cast());

            let mut stop_codec = cram_codec_byte_array_stop_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                byte_array_stop: cram_byte_array_stop_decoder_layout {
                    stop: b',',
                    content_id: 123,
                },
            };
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_3675_cram_byte_array_stop_describe(
                    (&mut stop_codec as *mut cram_codec_byte_array_stop_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"BYTE_ARRAY_STOP(stop=44,id=123)"
            );
            free(ks.s.cast());

            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_varint_get32),
                varint_get32s: Some(test_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut header = [b'|', 0xaa, 0xbb];
            let dec = cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init(
                std::ptr::null_mut(),
                header.as_mut_ptr().cast(),
                header.len() as c_int,
                5,
                4,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_byte_array_stop_layout>();
            assert!(!dec.is_null());
            assert_eq!((*dec).codec, 5);
            assert_eq!((*dec).byte_array_stop.stop, b'|');
            assert_eq!((*dec).byte_array_stop.content_id, 0x1234);
            assert_eq!(
                (*dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char as usize)
            );
            cram_cram_codecs_c_3669_cram_byte_array_stop_decode_free(dec.cast());

            let mut dat = [b';' as c_int, 77];
            let enc = cram_cram_codecs_c_3785_cram_byte_array_stop_encode_init(
                std::ptr::null_mut(),
                5,
                4,
                dat.as_mut_ptr().cast(),
                3 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_byte_array_stop_layout>();
            assert!(!enc.is_null());
            assert_eq!((*enc).codec, 5);
            assert_eq!((*enc).byte_array_stop.stop, b';');
            assert_eq!((*enc).byte_array_stop.content_id, 77);
            assert_eq!(
                (*enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize)
            );
            assert_eq!(
                (*enc).store,
                cram_fn_ptr(cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store as usize)
            );
            (*enc).vv = &mut vv as *mut varint_vec_layout;
            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store(
                    enc.cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                5
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 5, 2, b';', 77]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);
            cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free(enc.cast());
        }
    }

    #[test]
    fn cram_codecs_core_codec_describe_init_and_store_paths_match_c() {
        unsafe {
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_varint_get32),
                varint_get32s: Some(test_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk_append),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };

            let mut external = cram_codec_external_layout {
                codec: 1,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                external: cram_external_decoder_layout {
                    content_id: 12,
                    type_: 3,
                },
            };
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_454_cram_external_describe(
                    (&mut external as *mut cram_codec_external_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"EXTERNAL(id=12)"
            );
            free(ks.s.cast());

            let mut external_header = [0u8; 2];
            let ext_dec = cram_cram_codecs_c_459_cram_external_decode_init(
                std::ptr::null_mut(),
                external_header.as_mut_ptr().cast(),
                external_header.len() as c_int,
                1,
                3,
                3 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_external_layout>();
            assert!(!ext_dec.is_null());
            assert_eq!((*ext_dec).external.content_id, 0x1234);
            assert_eq!(
                (*ext_dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_390_cram_external_decode_char as usize)
            );
            cram_cram_codecs_c_433_cram_external_decode_free(ext_dec.cast());

            let ext_enc = cram_cram_codecs_c_586_cram_external_encode_init(
                std::ptr::null_mut(),
                1,
                1,
                test_void_token(7),
                3 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_external_layout>();
            assert!(!ext_enc.is_null());
            (*ext_enc).vv = &mut vv;
            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_562_cram_external_encode_store(
                    ext_enc.cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                4
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 1, 1, 7]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);
            cram_cram_codecs_c_556_cram_external_encode_free(ext_enc.cast());

            let mut varint = cram_codec_varint_layout {
                codec: 42,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                varint: cram_varint_decoder_layout {
                    content_id: 9,
                    offset: -7,
                    type_: 6,
                },
            };
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_752_cram_varint_describe(
                    (&mut varint as *mut cram_codec_varint_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"VARINT(id=9,offset=-7,type=6)"
            );
            free(ks.s.cast());

            let mut varint_header = [0u8; 5];
            let var_dec = cram_cram_codecs_c_760_cram_varint_decode_init(
                std::ptr::null_mut(),
                varint_header.as_mut_ptr().cast(),
                varint_header.len() as c_int,
                42,
                7,
                4 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_varint_layout>();
            assert!(!var_dec.is_null());
            assert_eq!((*var_dec).varint.content_id, 0x1234);
            assert_eq!(
                (*var_dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_710_cram_varint_decode_slong as usize)
            );
            cram_cram_codecs_c_732_cram_varint_decode_free(var_dec.cast());

            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_854_cram_varint_encode_store(
                    (&mut varint as *mut cram_codec_varint_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    4 << 8,
                ),
                5
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 42, 2, 9, 249]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            let mut st = cram_stats_layout {
                freqs: [0; 1024],
                h: std::ptr::null_mut(),
                nsamp: 0,
                nvals: 1,
                min_val: 5,
                max_val: 100,
            };
            let var_enc = cram_cram_codecs_c_878_cram_varint_encode_init(
                (&mut st as *mut cram_stats_layout).cast(),
                42,
                1,
                test_void_token(11),
                4 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_varint_layout>();
            assert!(!var_enc.is_null());
            assert_eq!((*var_enc).codec, 42);
            assert_eq!((*var_enc).varint.content_id, 11);
            assert_eq!((*var_enc).varint.offset, -5);
            cram_cram_codecs_c_848_cram_varint_encode_free(var_enc.cast());

            let mut constant = cram_codec_const_layout {
                codec: 44,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xconst: cram_const_codec_layout { val: -5 },
            };
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_976_cram_const_describe(
                    (&mut constant as *mut cram_codec_const_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"CONST(val=-5)"
            );
            free(ks.s.cast());

            let mut const_header = [0u8; 3];
            let const_dec = cram_cram_codecs_c_981_cram_const_decode_init(
                std::ptr::null_mut(),
                const_header.as_mut_ptr().cast(),
                const_header.len() as c_int,
                44,
                1,
                4 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_const_layout>();
            assert!(!const_dec.is_null());
            assert!(!(*const_dec).decode.is_null());
            let decode: unsafe fn(
                *mut cram_slice,
                *mut c_void,
                *mut cram_block,
                *mut c_char,
                *mut c_int,
            ) -> c_int = cram_fn((*const_dec).decode);
            let mut decoded = [0i32; 2];
            let mut decoded_len = decoded.len() as c_int;
            assert_eq!(
                decode(
                    std::ptr::null_mut(),
                    const_dec.cast(),
                    std::ptr::null_mut(),
                    decoded.as_mut_ptr().cast(),
                    &mut decoded_len,
                ),
                0
            );
            assert_eq!(decoded, [(*const_dec).xconst.val as i32; 2]);
            cram_cram_codecs_c_967_cram_const_decode_free(const_dec.cast());

            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_1025_cram_const_encode_store(
                    (&mut constant as *mut cram_codec_const_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    4 << 8,
                ),
                4
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 44, 1, 251]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            st.min_val = -5;
            let const_enc = cram_cram_codecs_c_1048_cram_const_encode_init(
                (&mut st as *mut cram_stats_layout).cast(),
                44,
                1,
                std::ptr::null_mut(),
                4 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_const_layout>();
            assert!(!const_enc.is_null());
            assert_eq!((*const_enc).xconst.val, -5);
            cram_cram_codecs_c_967_cram_const_decode_free(const_enc.cast());

            let mut beta = cram_codec_beta_layout {
                codec: 6,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                beta: cram_beta_decoder_layout {
                    offset: 2,
                    nbits: 5,
                },
            };
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_1136_cram_beta_describe(
                    (&mut beta as *mut cram_codec_beta_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"BETA(offset=2, nbits=5)"
            );
            free(ks.s.cast());

            let mut beta_vv = varint_vec_layout {
                varint_get32: Some(test_xdelta_varint_get32),
                ..vv
            };
            let mut beta_header = [2u8, 5u8];
            let beta_dec = cram_cram_codecs_c_1142_cram_beta_decode_init(
                std::ptr::null_mut(),
                beta_header.as_mut_ptr().cast(),
                beta_header.len() as c_int,
                6,
                1,
                3 << 8,
                (&mut beta_vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_beta_layout>();
            assert!(!beta_dec.is_null());
            assert_eq!((*beta_dec).beta.offset, 2);
            assert_eq!((*beta_dec).beta.nbits, 5);
            assert_eq!(
                (*beta_dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_1090_cram_beta_decode_int as usize)
            );
            cram_cram_codecs_c_1131_cram_beta_decode_free(beta_dec.cast());

            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_1183_cram_beta_encode_store(
                    (&mut beta as *mut cram_codec_beta_layout).cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    3 << 8,
                ),
                5
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 6, 2, 2, 5]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);

            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut constant as *mut cram_codec_const_layout).cast(),
                    std::ptr::null_mut(),
                ),
                -2
            );
            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut beta as *mut cram_codec_beta_layout).cast(),
                    std::ptr::null_mut(),
                ),
                -1
            );
            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut external as *mut cram_codec_external_layout).cast(),
                    std::ptr::null_mut(),
                ),
                12
            );
            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut varint as *mut cram_codec_varint_layout).cast(),
                    std::ptr::null_mut(),
                ),
                9
            );
            let mut huff_one = cram_codec_huffman_layout {
                codec: 3,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                huffman: cram_huffman_decoder_layout {
                    ncodes: 1,
                    codes: std::ptr::null_mut(),
                    option: 1,
                },
            };
            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut huff_one as *mut cram_codec_huffman_layout).cast(),
                    std::ptr::null_mut(),
                ),
                -2
            );
            huff_one.huffman.ncodes = 2;
            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut huff_one as *mut cram_codec_huffman_layout).cast(),
                    std::ptr::null_mut(),
                ),
                -1
            );

            let mut len_child = cram_codec_external_layout {
                external: cram_external_decoder_layout {
                    content_id: 17,
                    type_: 1,
                },
                ..external
            };
            let mut stop_codec = cram_codec_byte_array_stop_layout {
                codec: 5,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                byte_array_stop: cram_byte_array_stop_decoder_layout {
                    stop: b'|',
                    content_id: 23,
                },
            };
            let mut val_child = cram_codec_byte_array_stop_layout {
                codec: 5,
                byte_array_stop: cram_byte_array_stop_decoder_layout {
                    stop: b'|',
                    content_id: 23,
                },
                ..stop_codec
            };
            let mut bal = cram_codec_byte_array_len_layout {
                codec: 4,
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: (&mut len_child as *mut cram_codec_external_layout).cast(),
                    val_codec: (&mut val_child as *mut cram_codec_byte_array_stop_layout).cast(),
                },
                ..std::mem::zeroed()
            };
            let mut id2 = 0;
            assert_eq!(
                cram_cram_codecs_c_3968_cram_codec_to_id(
                    (&mut bal as *mut cram_codec_byte_array_len_layout).cast(),
                    &mut id2,
                ),
                17
            );
            assert_eq!(id2, 23);
            let mut ids = [0; 2];
            cram_codec_get_content_ids(
                (&mut bal as *mut cram_codec_byte_array_len_layout).cast(),
                ids.as_mut_ptr(),
            );
            assert_eq!(ids, [17, 23]);

            external.describe = cram_fn_ptr(cram_cram_codecs_c_454_cram_external_describe as usize);
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                cram_cram_codecs_c_4185_cram_codec_describe(
                    (&mut external as *mut cram_codec_external_layout).cast(),
                    &mut ks,
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l),
                b"EXTERNAL(id=12)"
            );
            free(ks.s.cast());
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(cram_codec_describe(std::ptr::null_mut(), &mut ks), 0);
            assert_eq!(std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l), b"?");
            free(ks.s.cast());

            external.decode =
                cram_fn_ptr(cram_cram_codecs_c_390_cram_external_decode_char as usize);
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut external as *mut cram_codec_external_layout).cast(),
                ),
                0
            );
            assert_eq!(
                external.encode,
                cram_fn_ptr(cram_cram_codecs_c_547_cram_external_encode_char as usize)
            );
            assert_eq!(
                external.store,
                cram_fn_ptr(cram_cram_codecs_c_562_cram_external_encode_store as usize)
            );

            varint.decode = cram_fn_ptr(cram_cram_codecs_c_710_cram_varint_decode_slong as usize);
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut varint as *mut cram_codec_varint_layout).cast(),
                ),
                0
            );
            assert_eq!(
                varint.encode,
                cram_fn_ptr(cram_cram_codecs_c_841_cram_varint_encode_slong as usize)
            );
            beta.decode = cram_fn_ptr(cram_cram_codecs_c_1108_cram_beta_decode_char as usize);
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut beta as *mut cram_codec_beta_layout).cast(),
                ),
                0
            );
            assert_eq!(
                beta.encode,
                cram_fn_ptr(cram_cram_codecs_c_1231_cram_beta_encode_char as usize)
            );
            stop_codec.codec = 5;
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut stop_codec as *mut cram_codec_byte_array_stop_layout).cast(),
                ),
                0
            );
            assert_eq!(
                stop_codec.encode,
                cram_fn_ptr(cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize)
            );
            let huff_conv = calloc(
                1,
                std::mem::size_of::<cram_codec_huffman_encoder_layout>() as u64,
            )
            .cast::<cram_codec_huffman_layout>();
            let mut huff_codes = [
                cram_huffman_code_layout {
                    symbol: 5,
                    p: 0,
                    code: 0,
                    len: 1,
                },
                cram_huffman_code_layout {
                    symbol: 6,
                    p: 0,
                    code: 1,
                    len: 1,
                },
            ];
            (*huff_conv).codec = 3;
            (*huff_conv).vv = &mut vv;
            (*huff_conv).decode =
                cram_fn_ptr(cram_cram_codecs_c_2708_cram_huffman_decode_int as usize);
            (*huff_conv).huffman.ncodes = huff_codes.len() as c_int;
            (*huff_conv).huffman.codes = huff_codes.as_mut_ptr();
            (*huff_conv).huffman.option = 1;
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    huff_conv.cast(),
                ),
                0
            );
            let huff_enc = huff_conv.cast::<cram_codec_huffman_encoder_layout>();
            assert_eq!(
                (*huff_enc).encode,
                cram_fn_ptr(cram_cram_codecs_c_3030_cram_huffman_encode_int as usize)
            );
            assert_eq!((*huff_enc).huffman.nvals, 2);
            assert_eq!((*huff_enc).huffman.val2code[6], 0);
            assert_eq!((*huff_enc).huffman.val2code[7], 1);
            free(huff_conv.cast());
        }
    }

    unsafe extern "C" fn test_codec_describe_len(_c: *mut c_void, ks: *mut kstring_t) -> c_int {
        (kputsn(c"LEN".as_ptr(), 3, ks) < 0) as c_int
    }

    unsafe extern "C" fn test_codec_describe_val(_c: *mut c_void, ks: *mut kstring_t) -> c_int {
        (kputsn(c"VAL".as_ptr(), 3, ks) < 0) as c_int
    }

    unsafe extern "C" fn test_byte_array_len_decode_len(
        _slice: *mut cram_slice,
        c: *mut c_void,
        _in: *mut cram_block,
        out: *mut c_char,
        _out_size: *mut c_int,
    ) -> c_int {
        *(out.cast::<i32>()) = (*(c.cast::<cram_codec_byte_array_len_layout>())).codec_id;
        0
    }

    unsafe extern "C" fn test_byte_array_len_decode_val(
        _slice: *mut cram_slice,
        _c: *mut c_void,
        _in: *mut cram_block,
        out: *mut c_char,
        out_size: *mut c_int,
    ) -> c_int {
        memcpy(out.cast(), c"XYZ".as_ptr().cast(), *out_size as u64);
        0
    }

    unsafe extern "C" fn test_byte_array_len_encode_len(
        _slice: *mut cram_slice,
        c: *mut c_void,
        in_: *mut c_char,
        _in_size: c_int,
    ) -> c_int {
        let val = *(in_.cast::<i32>()) as u8;
        let c = c.cast::<cram_codec_byte_array_len_layout>();
        cram_cram_io_h_261_block_append_char((*c).out.cast(), val as c_char)
    }

    unsafe extern "C" fn test_byte_array_len_encode_val(
        _slice: *mut cram_slice,
        c: *mut c_void,
        in_: *mut c_char,
        in_size: c_int,
    ) -> c_int {
        let c = c.cast::<cram_codec_byte_array_len_layout>();
        cram_cram_io_h_248_block_append((*c).out.cast(), in_.cast(), in_size as usize)
    }

    unsafe extern "C" fn test_xdelta_decode_u32(
        _slice: *mut cram_slice,
        _c: *mut c_void,
        in_: *mut cram_block,
        out: *mut c_char,
        _out_size: *mut c_int,
    ) -> c_int {
        let block = in_.cast::<cram_block_layout>();
        let idx = (*block).idx as usize;
        *(out.cast::<u32>()) = *((*block).data.cast::<u32>()).add(idx);
        (*block).idx += 1;
        0
    }

    unsafe extern "C" fn test_byte_array_len_store_len(
        _c: *mut c_void,
        b: *mut cram_block,
        _prefix: *mut c_char,
        _version: c_int,
    ) -> c_int {
        let mut bytes = *b"L";
        cram_cram_io_h_248_block_append(b, bytes.as_mut_ptr().cast(), bytes.len());
        bytes.len() as c_int
    }

    unsafe extern "C" fn test_byte_array_len_store_val(
        _c: *mut c_void,
        b: *mut cram_block,
        _prefix: *mut c_char,
        _version: c_int,
    ) -> c_int {
        let mut bytes = *b"VA";
        cram_cram_io_h_248_block_append(b, bytes.as_mut_ptr().cast(), bytes.len());
        bytes.len() as c_int
    }

    unsafe extern "C" fn test_xdelta_get_block(
        _slice: *mut cram_slice,
        c: *mut c_void,
    ) -> *mut cram_block {
        (*(c.cast::<cram_codec_xdelta_layout>())).out.cast()
    }

    unsafe extern "C" fn test_xrle_get_block(
        _slice: *mut cram_slice,
        c: *mut c_void,
    ) -> *mut cram_block {
        (*(c.cast::<cram_codec_xrle_layout>())).out.cast()
    }

    unsafe extern "C" fn test_xrle_size(_slice: *mut cram_slice, c: *mut c_void) -> c_int {
        let b = (*(c.cast::<cram_codec_xrle_layout>()))
            .out
            .cast::<cram_block_layout>();
        (*b).uncomp_size
    }

    unsafe extern "C" fn test_xdelta_varint_get32(
        cp: *mut *mut c_char,
        _endp: *const c_char,
        _err: *mut c_int,
    ) -> i64 {
        let val = **cp as u8;
        *cp = (*cp).add(1);
        val as i64
    }

    unsafe extern "C" fn test_varint_get32(
        cp: *mut *mut c_char,
        _endp: *const c_char,
        _err: *mut c_int,
    ) -> i64 {
        unsafe {
            *cp = (*cp).add(2);
        }
        0x1234
    }

    unsafe extern "C" fn test_varint_put32(cp: *mut c_char, _endp: *mut c_char, val: i32) -> c_int {
        *cp = val as c_char;
        1
    }

    unsafe extern "C" fn test_varint_put64(cp: *mut c_char, _endp: *mut c_char, val: i64) -> c_int {
        *cp = val as c_char;
        1
    }

    unsafe extern "C" fn test_varint_get64(
        cp: *mut *mut c_char,
        _endp: *const c_char,
        _err: *mut c_int,
    ) -> i64 {
        unsafe {
            *cp = (*cp).add(3);
        }
        0x0102_0304_0506_0708
    }

    unsafe extern "C" fn test_varint_put64s(
        cp: *mut c_char,
        _endp: *mut c_char,
        val: i64,
    ) -> c_int {
        *cp = val as c_char;
        1
    }

    unsafe extern "C" fn test_varint_put32_blk(blk: *mut cram_block, val: i32) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = val;
        }
        1
    }

    unsafe extern "C" fn test_varint_put32_blk_append(blk: *mut cram_block, val: i32) -> c_int {
        let mut byte = val as u8;
        cram_cram_io_h_248_block_append(blk, (&mut byte as *mut u8).cast(), 1);
        1
    }

    unsafe extern "C" fn test_varint_size(_val: i64) -> c_int {
        1
    }

    unsafe extern "C" fn test_varint_put32s_blk(blk: *mut cram_block, val: i32) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = -val;
        }
        1
    }

    unsafe extern "C" fn test_varint_put64_blk(blk: *mut cram_block, val: i64) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = val as i32;
        }
        1
    }

    unsafe extern "C" fn test_varint_put64s_blk(blk: *mut cram_block, val: i64) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = (-val) as i32;
        }
        1
    }

    #[test]
    fn cram_codecs_external_v4_rejects_invalid_options_and_allows_empty_missing_block() {
        unsafe {
            let mut vv: varint_vec_layout = std::mem::zeroed();
            cram_cram_io_c_5127_cram_init_varint((&mut vv as *mut varint_vec_layout).cast(), 4);
            let mut header = [9u8];

            assert!(cram_cram_codecs_c_459_cram_external_decode_init(
                std::ptr::null_mut(),
                header.as_mut_ptr().cast(),
                header.len() as c_int,
                1,
                1,
                4 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .is_null());
            assert!(cram_cram_codecs_c_459_cram_external_decode_init(
                std::ptr::null_mut(),
                header.as_mut_ptr().cast(),
                header.len() as c_int,
                2,
                3,
                4 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .is_null());

            let dec = cram_cram_codecs_c_459_cram_external_decode_init(
                std::ptr::null_mut(),
                header.as_mut_ptr().cast(),
                header.len() as c_int,
                1,
                5,
                4 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_external_layout>();
            assert!(!dec.is_null());
            assert_eq!((*dec).external.content_id, 9);
            assert_eq!(
                (*dec).decode,
                cram_fn_ptr(cram_cram_codecs_c_410_cram_external_decode_block as usize)
            );

            let mut hdr = cram_block_slice_hdr_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_MAPPED_SLICE,
                ref_seq_id: 0,
                ref_seq_start: 0,
                ref_seq_span: 0,
                num_records: 0,
                record_counter: 0,
                num_blocks: 0,
                num_content_ids: 0,
                block_content_ids: std::ptr::null_mut(),
                ref_base_id: 0,
                md5: [0; 16],
            };
            let mut slice = cram_slice_layout {
                hdr: &mut hdr,
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: std::ptr::null_mut(),
                ..std::mem::zeroed()
            };
            let out_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!out_block.is_null());
            let mut out_size = 0;
            assert_eq!(
                cram_cram_codecs_c_410_cram_external_decode_block(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    dec.cast(),
                    std::ptr::null_mut(),
                    out_block.cast(),
                    &mut out_size,
                ),
                0
            );
            assert_eq!((*(out_block.cast::<cram_block_layout>())).byte, 0);

            out_size = 1;
            assert_eq!(
                cram_cram_codecs_c_410_cram_external_decode_block(
                    (&mut slice as *mut cram_slice_layout).cast(),
                    dec.cast(),
                    std::ptr::null_mut(),
                    out_block.cast(),
                    &mut out_size,
                ),
                -1
            );

            cram_cram_io_c_1565_cram_free_block(out_block);
            cram_cram_codecs_c_433_cram_external_decode_free(dec.cast());
        }
    }

    #[test]
    fn cram_codecs_external_decode_init_rejects_trailing_header_bytes() {
        unsafe {
            for version in [3, 4] {
                let mut vv: varint_vec_layout = std::mem::zeroed();
                cram_cram_io_c_5127_cram_init_varint(
                    (&mut vv as *mut varint_vec_layout).cast(),
                    version,
                );
                let mut header = [9u8, 0];

                assert!(cram_cram_codecs_c_459_cram_external_decode_init(
                    std::ptr::null_mut(),
                    header.as_mut_ptr().cast(),
                    header.len() as c_int,
                    1,
                    if version >= 4 { 5 } else { 1 },
                    version << 8,
                    (&mut vv as *mut varint_vec_layout).cast(),
                )
                .is_null());
            }
        }
    }

    #[test]
    fn cram_codecs_byte_array_stop_v1_uses_fixed_little_endian_content_id() {
        unsafe {
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_varint_get32),
                varint_get32s: Some(test_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut header = [b'|', 0x78, 0x56, 0x34, 0x12];
            let dec = cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init(
                std::ptr::null_mut(),
                header.as_mut_ptr().cast(),
                header.len() as c_int,
                5,
                4,
                1 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .cast::<cram_codec_byte_array_stop_layout>();
            assert!(!dec.is_null());
            assert_eq!((*dec).byte_array_stop.stop, b'|');
            assert_eq!((*dec).byte_array_stop.content_id, 0x1234_5678);
            cram_cram_codecs_c_3669_cram_byte_array_stop_decode_free(dec.cast());

            let mut short = [b'|', 0x78, 0x56, 0x34];
            assert!(cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init(
                std::ptr::null_mut(),
                short.as_mut_ptr().cast(),
                short.len() as c_int,
                5,
                4,
                1 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .is_null());

            let mut extra = [b'|', 0x78, 0x56, 0x34, 0x12, 0];
            assert!(cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init(
                std::ptr::null_mut(),
                extra.as_mut_ptr().cast(),
                extra.len() as c_int,
                5,
                4,
                1 << 8,
                (&mut vv as *mut varint_vec_layout).cast(),
            )
            .is_null());

            let mut dat = [b';' as c_int, 0x1234_5678];
            let enc = cram_cram_codecs_c_3785_cram_byte_array_stop_encode_init(
                std::ptr::null_mut(),
                5,
                4,
                dat.as_mut_ptr().cast(),
                1 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_byte_array_stop_layout>();
            assert!(!enc.is_null());
            (*enc).vv = &mut vv as *mut varint_vec_layout;
            let store_block = cram_cram_io_c_1388_cram_new_block(
                crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                0,
            );
            assert!(!store_block.is_null());
            let mut prefix = *b"P\0";
            assert_eq!(
                cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store(
                    enc.cast(),
                    store_block,
                    prefix.as_mut_ptr().cast(),
                    1 << 8,
                ),
                8
            );
            let store_layout = store_block.cast::<cram_block_layout>();
            assert_eq!(
                std::slice::from_raw_parts((*store_layout).data, (*store_layout).byte),
                &[b'P', 5, 5, b';', 0x78, 0x56, 0x34, 0x12]
            );
            cram_cram_io_c_1565_cram_free_block(store_block);
            cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free(enc.cast());
        }
    }

    #[test]
    fn cram_codecs_external_decoders_find_blocks_copy_and_advance() {
        unsafe {
            let mut data = *b"abcdefgh";
            let mut ext_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 17,
                comp_size: 0,
                uncomp_size: data.len() as i32,
                crc32: 0,
                idx: 1,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let ext_ptr = (&mut ext_block as *mut cram_block_layout).cast::<cram_block>();
            let mut blocks = [ext_ptr];
            let mut hdr = cram_block_slice_hdr_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_MAPPED_SLICE,
                ref_seq_id: 0,
                ref_seq_start: 0,
                ref_seq_span: 0,
                num_records: 0,
                record_counter: 0,
                num_blocks: 1,
                num_content_ids: 0,
                block_content_ids: std::ptr::null_mut(),
                ref_base_id: 0,
                md5: [0; 16],
            };
            let mut slice = cram_slice_layout {
                hdr: &mut hdr,
                hdr_block: std::ptr::null_mut(),
                block: blocks.as_mut_ptr().cast(),
                block_by_id: std::ptr::null_mut(),
                ..std::mem::zeroed()
            };
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_varint_get32),
                varint_get32s: Some(test_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut codec = cram_codec_external_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                external: cram_external_decoder_layout {
                    content_id: 17,
                    type_: 0,
                },
            };
            let s = (&mut slice as *mut cram_slice_layout).cast();
            let c = (&mut codec as *mut cram_codec_external_layout).cast();

            assert_eq!(cram_cram_codecs_c_439_cram_external_decode_size(s, c), 8);
            assert_eq!(
                cram_cram_codecs_c_450_cram_external_get_block(s, c),
                ext_ptr
            );

            let mut out_size = 3;
            let mut out = [0 as c_char; 4];
            assert_eq!(
                cram_cram_codecs_c_390_cram_external_decode_char(
                    s,
                    c,
                    std::ptr::null_mut(),
                    out.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(out.as_ptr().cast::<u8>(), 3),
                b"bcd"
            );
            assert_eq!(ext_block.idx, 4);

            let mut out_block = cram_block_layout {
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                idx: 0,
                uncomp_size: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                method: 0,
                orig_method: 0,
                comp_size: 0,
                crc32: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            out_size = 2;
            assert_eq!(
                cram_cram_codecs_c_410_cram_external_decode_block(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut out_block as *mut cram_block_layout).cast::<c_char>(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(out_block.data, out_block.byte),
                b"ef"
            );
            free(out_block.data.cast());

            let mut iout = 0i32;
            out_size = 99;
            assert_eq!(
                cram_cram_codecs_c_350_cram_external_decode_int(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut iout as *mut i32).cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!((iout, out_size, ext_block.idx), (0x1234, 1, 8));

            ext_block.idx = 0;
            let mut lout = 0i64;
            out_size = 99;
            assert_eq!(
                cram_cram_codecs_c_370_cram_external_decode_long(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut lout as *mut i64).cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                (lout, out_size, ext_block.idx),
                (0x0102_0304_0506_0708, 1, 3)
            );

            let mut missing_codec = cram_codec_external_layout {
                external: cram_external_decoder_layout {
                    content_id: 999,
                    type_: 0,
                },
                ..codec
            };
            let c_missing = (&mut missing_codec as *mut cram_codec_external_layout).cast();
            out_size = 0;
            assert_eq!(
                cram_cram_codecs_c_390_cram_external_decode_char(
                    s,
                    c_missing,
                    std::ptr::null_mut(),
                    out.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            out_size = 1;
            assert_eq!(
                cram_cram_codecs_c_390_cram_external_decode_char(
                    s,
                    c_missing,
                    std::ptr::null_mut(),
                    out.as_mut_ptr(),
                    &mut out_size
                ),
                -1
            );
        }
    }

    #[test]
    fn cram_codec_block_lookup_uses_cache_only_when_ids_match_then_scans_external_blocks() {
        unsafe {
            let mut direct = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 17,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut wrong_hash = cram_block_layout {
                content_id: 251,
                ..direct
            };
            let mut colliding = cram_block_layout {
                content_id: 502,
                ..direct
            };
            let mut non_external = cram_block_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE,
                content_id: 753,
                ..direct
            };

            let direct_ptr = (&mut direct as *mut cram_block_layout).cast::<cram_block>();
            let wrong_hash_ptr = (&mut wrong_hash as *mut cram_block_layout).cast::<cram_block>();
            let colliding_ptr = (&mut colliding as *mut cram_block_layout).cast::<cram_block>();
            let non_external_ptr =
                (&mut non_external as *mut cram_block_layout).cast::<cram_block>();
            let mut block_by_id = [std::ptr::null_mut(); 768];
            block_by_id[17] = direct_ptr;
            block_by_id[256] = wrong_hash_ptr;
            let mut blocks = [non_external_ptr, colliding_ptr];
            let mut hdr = cram_block_slice_hdr_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_MAPPED_SLICE,
                ref_seq_id: 0,
                ref_seq_start: 0,
                ref_seq_span: 0,
                num_records: 0,
                record_counter: 0,
                num_blocks: blocks.len() as c_int,
                num_content_ids: 0,
                block_content_ids: std::ptr::null_mut(),
                ref_base_id: 0,
                md5: [0; 16],
            };
            let mut slice = cram_slice_layout {
                hdr: &mut hdr,
                hdr_block: std::ptr::null_mut(),
                block: blocks.as_mut_ptr().cast(),
                block_by_id: block_by_id.as_mut_ptr().cast(),
                ..std::mem::zeroed()
            };
            let s = (&mut slice as *mut cram_slice_layout).cast();

            assert_eq!(cram_cram_io_h_183_cram_get_block_by_id(s, 17), direct_ptr);
            assert_eq!(
                cram_cram_io_h_183_cram_get_block_by_id(s, 502),
                colliding_ptr
            );
            assert!(cram_cram_io_h_183_cram_get_block_by_id(s, 753).is_null());
        }
    }

    #[test]
    fn cram_codecs_external_encoders_forward_to_varint_and_append_bytes() {
        unsafe {
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: None,
                varint_get32s: None,
                varint_get64: None,
                varint_get64s: None,
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut out_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut codec = cram_codec_external_layout {
                codec: 0,
                out: (&mut out_block as *mut cram_block_layout).cast(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                external: cram_external_decoder_layout {
                    content_id: 0,
                    type_: 0,
                },
            };
            let c = (&mut codec as *mut cram_codec_external_layout).cast();

            let mut u32v = 77u32;
            assert_eq!(
                cram_cram_codecs_c_523_cram_external_encode_int(
                    std::ptr::null_mut(),
                    c,
                    (&mut u32v as *mut u32).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 77);

            let mut i32v = -9i32;
            assert_eq!(
                cram_cram_codecs_c_529_cram_external_encode_sint(
                    std::ptr::null_mut(),
                    c,
                    (&mut i32v as *mut i32).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 9);

            let mut u64v = 1234u64;
            assert_eq!(
                cram_cram_codecs_c_535_cram_external_encode_long(
                    std::ptr::null_mut(),
                    c,
                    (&mut u64v as *mut u64).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 1234);

            let mut i64v = -55i64;
            assert_eq!(
                cram_cram_codecs_c_541_cram_external_encode_slong(
                    std::ptr::null_mut(),
                    c,
                    (&mut i64v as *mut i64).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 55);

            let mut bytes = *b"raw";
            assert_eq!(
                cram_cram_codecs_c_547_cram_external_encode_char(
                    std::ptr::null_mut(),
                    c,
                    bytes.as_mut_ptr().cast(),
                    bytes.len() as c_int
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(out_block.data, out_block.byte),
                b"raw"
            );
            free(out_block.data.cast());
        }
    }

    #[test]
    fn cram_codecs_varint_decoders_apply_offsets_and_find_blocks() {
        unsafe {
            let mut data = *b"abcdefgh";
            let mut var_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 23,
                comp_size: 0,
                uncomp_size: data.len() as i32,
                crc32: 0,
                idx: 0,
                data: data.as_mut_ptr(),
                alloc: data.len(),
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let var_ptr = (&mut var_block as *mut cram_block_layout).cast::<cram_block>();
            let mut blocks = [var_ptr];
            let mut hdr = cram_block_slice_hdr_layout {
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_MAPPED_SLICE,
                ref_seq_id: 0,
                ref_seq_start: 0,
                ref_seq_span: 0,
                num_records: 0,
                record_counter: 0,
                num_blocks: 1,
                num_content_ids: 0,
                block_content_ids: std::ptr::null_mut(),
                ref_base_id: 0,
                md5: [0; 16],
            };
            let mut slice = cram_slice_layout {
                hdr: &mut hdr,
                hdr_block: std::ptr::null_mut(),
                block: blocks.as_mut_ptr().cast(),
                block_by_id: std::ptr::null_mut(),
                ..std::mem::zeroed()
            };
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: Some(test_varint_get32),
                varint_get32s: Some(test_varint_get32),
                varint_get64: Some(test_varint_get64),
                varint_get64s: Some(test_varint_get64),
                varint_put32: None,
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: None,
                varint_put32s_blk: None,
                varint_put64_blk: None,
                varint_put64s_blk: None,
                varint_size: None,
            };
            let mut codec = cram_codec_varint_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                varint: cram_varint_decoder_layout {
                    content_id: 23,
                    offset: 5,
                    type_: 0,
                },
            };
            let s = (&mut slice as *mut cram_slice_layout).cast();
            let c = (&mut codec as *mut cram_codec_varint_layout).cast();

            assert_eq!(cram_cram_codecs_c_737_cram_varint_decode_size(s, c), 8);
            assert_eq!(cram_cram_codecs_c_748_cram_varint_get_block(s, c), var_ptr);

            let mut out_size = 99;
            let mut iout = 0i32;
            assert_eq!(
                cram_cram_codecs_c_644_cram_varint_decode_int(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut iout as *mut i32).cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!((iout, out_size, var_block.idx), (0x1234 + 5, 1, 2));

            var_block.idx = 0;
            assert_eq!(
                cram_cram_codecs_c_666_cram_varint_decode_sint(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut iout as *mut i32).cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!((iout, out_size, var_block.idx), (0x1234 + 5, 1, 2));

            var_block.idx = 0;
            let mut lout = 0i64;
            assert_eq!(
                cram_cram_codecs_c_688_cram_varint_decode_long(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut lout as *mut i64).cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                (lout, out_size, var_block.idx),
                (0x0102_0304_0506_0708 + 5, 1, 3)
            );

            var_block.idx = 0;
            assert_eq!(
                cram_cram_codecs_c_710_cram_varint_decode_slong(
                    s,
                    c,
                    std::ptr::null_mut(),
                    (&mut lout as *mut i64).cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                (lout, out_size, var_block.idx),
                (0x0102_0304_0506_0708 + 5, 1, 3)
            );

            let mut missing = cram_codec_varint_layout {
                varint: cram_varint_decoder_layout {
                    content_id: 999,
                    offset: 0,
                    type_: 0,
                },
                ..codec
            };
            let missing_c = (&mut missing as *mut cram_codec_varint_layout).cast();
            out_size = 0;
            assert_eq!(
                cram_cram_codecs_c_644_cram_varint_decode_int(
                    s,
                    missing_c,
                    std::ptr::null_mut(),
                    (&mut iout as *mut i32).cast(),
                    &mut out_size
                ),
                0
            );
            out_size = 1;
            assert_eq!(
                cram_cram_codecs_c_644_cram_varint_decode_int(
                    s,
                    missing_c,
                    std::ptr::null_mut(),
                    (&mut iout as *mut i32).cast(),
                    &mut out_size
                ),
                -1
            );
        }
    }

    #[test]
    fn cram_codecs_varint_encoders_subtract_offsets_before_callbacks() {
        unsafe {
            let mut vv = varint_vec_layout {
                varint_decode32_crc: std::ptr::null_mut(),
                varint_decode32s_crc: std::ptr::null_mut(),
                varint_decode64_crc: std::ptr::null_mut(),
                varint_get32: None,
                varint_get32s: None,
                varint_get64: None,
                varint_get64s: None,
                varint_put32: Some(test_varint_put32),
                varint_put32s: Some(test_varint_put32),
                varint_put64: Some(test_varint_put64),
                varint_put64s: Some(test_varint_put64s),
                varint_put32_blk: Some(test_varint_put32_blk),
                varint_put32s_blk: Some(test_varint_put32s_blk),
                varint_put64_blk: Some(test_varint_put64_blk),
                varint_put64s_blk: Some(test_varint_put64s_blk),
                varint_size: Some(test_varint_size),
            };
            let mut out_block = cram_block_layout {
                method: 0,
                orig_method: 0,
                content_type: crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL,
                content_id: 0,
                comp_size: 0,
                uncomp_size: 0,
                crc32: 0,
                idx: 0,
                data: std::ptr::null_mut(),
                alloc: 0,
                byte: 0,
                bit: 0,
                m: std::ptr::null_mut(),
                crc32_checked: 0,
                crc_part: 0,
            };
            let mut codec = cram_codec_varint_layout {
                codec: 0,
                out: (&mut out_block as *mut cram_block_layout).cast(),
                vv: &mut vv,
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                varint: cram_varint_decoder_layout {
                    content_id: 0,
                    offset: 10,
                    type_: 0,
                },
            };
            let c = (&mut codec as *mut cram_codec_varint_layout).cast();

            let mut u32v = 77u32;
            assert_eq!(
                cram_cram_codecs_c_820_cram_varint_encode_int(
                    std::ptr::null_mut(),
                    c,
                    (&mut u32v as *mut u32).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 67);

            let mut i32v = -9i32;
            assert_eq!(
                cram_cram_codecs_c_827_cram_varint_encode_sint(
                    std::ptr::null_mut(),
                    c,
                    (&mut i32v as *mut i32).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 19);

            let mut u64v = 1234u64;
            assert_eq!(
                cram_cram_codecs_c_834_cram_varint_encode_long(
                    std::ptr::null_mut(),
                    c,
                    (&mut u64v as *mut u64).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 1224);

            let mut i64v = -55i64;
            assert_eq!(
                cram_cram_codecs_c_841_cram_varint_encode_slong(
                    std::ptr::null_mut(),
                    c,
                    (&mut i64v as *mut i64).cast(),
                    1
                ),
                0
            );
            assert_eq!(out_block.idx, 65);
        }
    }

    #[test]
    fn cram_codecs_const_decode_repeats_value_and_encode_is_noop() {
        unsafe {
            let mut codec = cram_codec_const_layout {
                codec: 0,
                out: std::ptr::null_mut(),
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: std::ptr::null_mut(),
                describe: std::ptr::null_mut(),
                xconst: cram_const_codec_layout { val: 0x41 },
            };
            let c = (&mut codec as *mut cram_codec_const_layout).cast();

            let mut out_size = 4;
            let mut bytes = [0 as c_char; 4];
            assert_eq!(
                cram_cram_codecs_c_932_cram_const_decode_byte(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    bytes.as_mut_ptr(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(
                std::slice::from_raw_parts(bytes.as_ptr().cast::<u8>(), 4),
                b"AAAA"
            );
            assert_eq!(
                cram_cram_codecs_c_932_cram_const_decode_byte(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut out_size
                ),
                0
            );

            codec.xconst.val = -7;
            let mut ints = [0i32; 3];
            out_size = 3;
            assert_eq!(
                cram_cram_codecs_c_945_cram_const_decode_int(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_const_layout).cast(),
                    std::ptr::null_mut(),
                    ints.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(ints, [-7, -7, -7]);

            codec.xconst.val = 0x0102_0304_0506_0708;
            let mut longs = [0i64; 2];
            out_size = 2;
            assert_eq!(
                cram_cram_codecs_c_956_cram_const_decode_long(
                    std::ptr::null_mut(),
                    (&mut codec as *mut cram_codec_const_layout).cast(),
                    std::ptr::null_mut(),
                    longs.as_mut_ptr().cast(),
                    &mut out_size
                ),
                0
            );
            assert_eq!(longs, [0x0102_0304_0506_0708; 2]);
            assert_eq!(
                cram_cram_codecs_c_972_cram_const_decode_size(std::ptr::null_mut(), c),
                0
            );
            assert_eq!(
                cram_cram_codecs_c_1020_cram_const_encode(
                    std::ptr::null_mut(),
                    c,
                    std::ptr::null_mut(),
                    0
                ),
                0
            );
        }
    }

    #[test]
    fn cram_io_method_and_content_type_strings_match_c_switches() {
        unsafe {
            assert_eq!(
                CStr::from_ptr(cram_cram_io_c_2341_cram_block_method2str(0)).to_bytes(),
                b"RAW"
            );
            assert_eq!(
                CStr::from_ptr(cram_cram_io_c_2341_cram_block_method2str(16)).to_bytes(),
                b"RANS1"
            );
            assert_eq!(
                CStr::from_ptr(cram_cram_io_c_2341_cram_block_method2str(31)).to_bytes(),
                b"ARITH_PR193"
            );
            assert_eq!(
                CStr::from_ptr(cram_cram_io_c_2341_cram_block_method2str(10)).to_bytes(),
                b"?"
            );
            assert_eq!(
                CStr::from_ptr(cram_cram_io_c_2378_cram_content_type2str(
                    crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE
                ))
                .to_bytes(),
                b"CORE"
            );
            assert_eq!(
                CStr::from_ptr(cram_cram_io_c_2378_cram_content_type2str(-1)).to_bytes(),
                b"?"
            );
        }
    }

    #[test]
    fn cram_io_safe_varint_getters_report_truncation_without_advancing() {
        unsafe {
            let mut itf8 = [0xf0u8, 0x12, 0x34, 0x56];
            let mut cp = itf8.as_mut_ptr().cast::<c_char>();
            let start = cp;
            let mut err = 0;
            assert_eq!(
                cram_cram_io_c_644_safe_itf8_get(
                    &mut cp,
                    itf8.as_ptr().add(itf8.len()).cast(),
                    &mut err,
                ),
                0
            );
            assert_eq!(cp, start);
            assert_eq!(err, 1);

            let mut ltf8 = [0xffu8, 1, 2, 3, 4, 5, 6, 7];
            let mut cp = ltf8.as_mut_ptr().cast::<c_char>();
            let start = cp;
            err = 0;
            assert_eq!(
                cram_cram_io_c_673_safe_ltf8_get(
                    &mut cp,
                    ltf8.as_ptr().add(ltf8.len()).cast(),
                    &mut err,
                ),
                0
            );
            assert_eq!(cp, start);
            assert_eq!(err, 1);
        }
    }

    #[test]
    fn cram_io_is_directory_matches_stat_directory_bit() {
        unsafe {
            let dir = CString::new("/tmp").unwrap();
            assert_eq!(cram_cram_io_c_2873_is_directory(dir.as_ptr().cast_mut()), 1);

            let missing = CString::new("/tmp/htslib_rs-missing-directory-probe").unwrap();
            assert_eq!(
                cram_cram_io_c_2873_is_directory(missing.as_ptr().cast_mut()),
                0
            );
        }
    }

    #[test]
    fn cram_open_trace_file_is_file_matches_regular_file_stat_bit() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-open-trace-is-file-{}",
                std::process::id()
            ));
            std::fs::write(&path, b"trace").unwrap();
            let file = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            assert_eq!(
                cram_open_trace_file_c_90_is_file(file.as_ptr().cast_mut()),
                1
            );

            let dir = CString::new("/tmp").unwrap();
            assert_eq!(
                cram_open_trace_file_c_90_is_file(dir.as_ptr().cast_mut()),
                0
            );

            let missing = CString::new("/tmp/htslib_rs-missing-file-probe").unwrap();
            assert_eq!(
                cram_open_trace_file_c_90_is_file(missing.as_ptr().cast_mut()),
                0
            );

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn cram_open_trace_file_tokenise_search_path_matches_colon_rules() {
        unsafe {
            let input = CString::new("alpha::beta:gamma::delta::").unwrap();
            let out = cram_open_trace_file_c_108_tokenise_search_path(input.as_ptr());
            assert!(!out.is_null());

            let expected = b"alpha:beta\0gamma:delta:\0./\0\0";
            assert_eq!(
                std::slice::from_raw_parts(out.cast::<u8>(), expected.len()),
                expected
            );
            free(out.cast());

            let out = cram_open_trace_file_c_108_tokenise_search_path(std::ptr::null());
            assert!(!out.is_null());
            assert_eq!(std::slice::from_raw_parts(out.cast::<u8>(), 4), b"./\0\0");
            free(out.cast());
        }
    }

    #[test]
    fn cram_open_trace_file_tokenise_search_path_preserves_remote_url_prefixes() {
        unsafe {
            let input = CString::new("URL=http://example.invalid/data:local").unwrap();
            let out = cram_open_trace_file_c_108_tokenise_search_path(input.as_ptr());
            assert!(!out.is_null());

            let expected = b"URL=http://example.invalid/data\0local\0./\0\0";
            assert_eq!(
                std::slice::from_raw_parts(out.cast::<u8>(), expected.len()),
                expected
            );
            free(out.cast());
        }
    }

    #[test]
    fn cram_open_trace_file_expand_path_matches_percent_rules() {
        unsafe {
            let file = CString::new("abcdef").unwrap();
            let dirname = CString::new("root/%2s/%s").unwrap();
            let out =
                cram_open_trace_file_c_230_expand_path(file.as_ptr(), dirname.as_ptr(), c_int::MAX);
            assert!(!out.is_null());
            assert_eq!(CStr::from_ptr(out).to_bytes(), b"root/ab/cdef");
            free(out.cast());

            let absolute = CString::new("/tmp/ref.fa").unwrap();
            let dot = CString::new(".").unwrap();
            let out = cram_open_trace_file_c_230_expand_path(absolute.as_ptr(), dot.as_ptr(), 1);
            assert!(!out.is_null());
            assert_eq!(CStr::from_ptr(out).to_bytes(), b"/tmp/ref.fa");
            free(out.cast());

            let bad = CString::new("root/%22s").unwrap();
            let out = cram_open_trace_file_c_230_expand_path(file.as_ptr(), bad.as_ptr(), 1);
            assert!(!out.is_null());
            assert_eq!(CStr::from_ptr(out).to_bytes(), b"root/%22s/abcdef");
            free(out.cast());

            let zero = CString::new("root/%0s//").unwrap();
            let out = cram_open_trace_file_c_230_expand_path(file.as_ptr(), zero.as_ptr(), 1);
            assert!(!out.is_null());
            assert_eq!(CStr::from_ptr(out).to_bytes(), b"root/abcdef");
            free(out.cast());
        }
    }

    #[test]
    fn cram_open_trace_file_expand_path_consumes_file_across_multiple_percent_tokens() {
        unsafe {
            let file = CString::new("abcdef").unwrap();
            let dirname = CString::new("root/%1s/%2s/%s").unwrap();
            let out =
                cram_open_trace_file_c_230_expand_path(file.as_ptr(), dirname.as_ptr(), c_int::MAX);
            assert!(!out.is_null());
            assert_eq!(CStr::from_ptr(out).to_bytes(), b"root/a/bc/def");
            free(out.cast());
        }
    }

    #[test]
    fn cram_open_trace_file_find_path_skips_remote_and_returns_existing_local_path() {
        unsafe {
            let dir =
                std::env::temp_dir().join(format!("htslib_rs-find-path-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let file_path = dir.join("trace.ab1");
            std::fs::write(&file_path, b"trace").unwrap();

            let file = CString::new("trace.ab1").unwrap();
            let search = CString::new(format!(
                "URL=http://example.invalid/ref:{}",
                dir.to_string_lossy()
            ))
            .unwrap();
            let out = cram_open_trace_file_c_433_find_path(file.as_ptr(), search.as_ptr());
            assert!(!out.is_null());
            assert_eq!(
                CStr::from_ptr(out).to_bytes(),
                file_path.to_string_lossy().as_bytes()
            );
            free(out.cast());

            std::fs::remove_file(file_path).unwrap();
            std::fs::remove_dir(dir).unwrap();
        }
    }

    #[test]
    fn cram_open_trace_file_find_file_dir_opens_existing_file_as_mfile() {
        unsafe {
            let dir = std::env::temp_dir()
                .join(format!("htslib_rs-find-file-dir-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let file_path = dir.join("trace.dat");
            std::fs::write(&file_path, b"dir-data").unwrap();

            let file = CString::new("trace.dat").unwrap();
            let dirname = CString::new(dir.to_string_lossy().as_bytes()).unwrap();
            let mf = cram_open_trace_file_c_314_find_file_dir(
                file.as_ptr(),
                dirname.as_ptr().cast_mut(),
            );
            assert!(!mf.is_null());
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"dir-data"
            );
            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);

            std::fs::remove_file(file_path).unwrap();
            std::fs::remove_dir(dir).unwrap();
        }
    }

    #[test]
    fn cram_open_trace_file_url_and_open_path_mfile_load_contents() {
        unsafe {
            let dir = std::env::temp_dir()
                .join(format!("htslib_rs-open-trace-path-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let file_path = dir.join("trace.dat");
            std::fs::write(&file_path, b"trace payload").unwrap();

            let file = CString::new("trace.dat").unwrap();
            let dirname = CString::new(dir.to_string_lossy().as_bytes()).unwrap();
            let mf = cram_open_trace_file_c_182_find_file_url(
                file.as_ptr(),
                dirname.as_ptr().cast_mut(),
            );
            assert!(!mf.is_null());
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"trace payload"
            );
            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);

            let mut local = -1;
            let mf = cram_open_trace_file_c_352_open_path_mfile(
                file.as_ptr(),
                dirname.as_ptr().cast_mut(),
                std::ptr::null_mut(),
                &mut local,
            );
            assert!(!mf.is_null());
            assert_eq!(local, 1);
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"trace payload"
            );
            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);

            let relative = CString::new(file_path.to_string_lossy().as_bytes()).unwrap();
            let missing_path = CString::new("/definitely/not/here").unwrap();
            let mf = cram_open_trace_file_c_352_open_path_mfile(
                file.as_ptr(),
                missing_path.as_ptr().cast_mut(),
                relative.as_ptr().cast_mut(),
                &mut local,
            );
            assert!(!mf.is_null());
            assert_eq!(local, 1);
            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);

            std::fs::remove_file(file_path).unwrap();
            std::fs::remove_dir(dir).unwrap();
        }
    }

    #[test]
    fn cram_open_trace_file_open_path_mfile_reports_local_for_relative_fallback_and_miss() {
        unsafe {
            let dir = std::env::temp_dir().join(format!(
                "htslib_rs-open-path-fallback-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let file_path = dir.join("trace.dat");
            std::fs::write(&file_path, b"fallback-data").unwrap();

            let file = CString::new("trace.dat").unwrap();
            let missing_path = CString::new("/definitely/not/here").unwrap();
            let relative =
                CString::new(dir.join("anchor.cram").to_string_lossy().as_bytes()).unwrap();
            let mut local = -1;
            let mf = cram_open_trace_file_c_352_open_path_mfile(
                file.as_ptr(),
                missing_path.as_ptr().cast_mut(),
                relative.as_ptr().cast_mut(),
                &mut local,
            );
            assert!(!mf.is_null());
            assert_eq!(local, 1);
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"fallback-data"
            );
            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);

            let missing_file = CString::new("missing.dat").unwrap();
            local = -7;
            let mf = cram_open_trace_file_c_352_open_path_mfile(
                missing_file.as_ptr(),
                missing_path.as_ptr().cast_mut(),
                relative.as_ptr().cast_mut(),
                &mut local,
            );
            assert!(mf.is_null());
            assert_eq!(local, 1);

            std::fs::remove_file(file_path).unwrap();
            std::fs::remove_dir(dir).unwrap();
        }
    }

    #[test]
    fn cram_io_expand_cache_path_replaces_percent_patterns_like_c() {
        unsafe {
            let mut path = vec![0 as c_char; crate::htslib_rs::c_compat::PATH_MAX as usize];
            let mut dir = CString::new("/cache/%2s/%s").unwrap().into_bytes_with_nul();
            let file = CString::new("abcdef").unwrap();
            assert_eq!(
                cram_cram_io_c_2884_expand_cache_path(
                    path.as_mut_ptr(),
                    dir.as_mut_ptr().cast(),
                    file.as_ptr()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(path.as_ptr()).to_bytes(), b"/cache/ab/cdef");

            path.fill(0);
            let mut dir = CString::new("/cache/%x").unwrap().into_bytes_with_nul();
            let file = CString::new("ref.fa").unwrap();
            assert_eq!(
                cram_cram_io_c_2884_expand_cache_path(
                    path.as_mut_ptr(),
                    dir.as_mut_ptr().cast(),
                    file.as_ptr()
                ),
                0
            );
            assert_eq!(
                CStr::from_ptr(path.as_ptr()).to_bytes(),
                b"/cache/%x/ref.fa"
            );

            path.fill(0);
            let mut dir = CString::new("/cache/").unwrap().into_bytes_with_nul();
            let file = CString::new("ref.fa").unwrap();
            assert_eq!(
                cram_cram_io_c_2884_expand_cache_path(
                    path.as_mut_ptr(),
                    dir.as_mut_ptr().cast(),
                    file.as_ptr()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(path.as_ptr()).to_bytes(), b"/cache/ref.fa");
        }
    }

    #[test]
    fn cram_io_mkdir_prefix_creates_parent_directories_and_restores_path() {
        unsafe {
            let base =
                std::env::temp_dir().join(format!("htslib_rs-mkdir-prefix-{}", std::process::id()));
            let full = base.join("aa").join("bb").join("ref.fa");
            let mut path = CString::new(full.to_string_lossy().as_bytes())
                .unwrap()
                .into_bytes_with_nul();

            cram_cram_io_c_2947_mkdir_prefix(path.as_mut_ptr().cast(), 0o755);

            assert_eq!(
                CStr::from_ptr(path.as_ptr().cast()).to_bytes(),
                full.to_string_lossy().as_bytes()
            );
            assert!(base.join("aa").is_dir());
            assert!(base.join("aa").join("bb").is_dir());
            assert!(!full.exists());

            let mut no_slash = CString::new("ref.fa").unwrap().into_bytes_with_nul();
            cram_cram_io_c_2947_mkdir_prefix(no_slash.as_mut_ptr().cast(), 0o755);
            assert_eq!(
                CStr::from_ptr(no_slash.as_ptr().cast()).to_bytes(),
                b"ref.fa"
            );

            std::fs::remove_dir(base.join("aa").join("bb")).unwrap();
            std::fs::remove_dir(base.join("aa")).unwrap();
            std::fs::remove_dir(base).unwrap();
        }
    }

    #[test]
    fn cram_io_full_path_preserves_absolute_and_expands_local_paths() {
        unsafe {
            let mut out = vec![0 as c_char; crate::htslib_rs::c_compat::PATH_MAX as usize];

            let abs = CString::new("/tmp/reference.fa").unwrap();
            cram_cram_io_c_4850_full_path(out.as_mut_ptr(), abs.as_ptr().cast_mut());
            assert_eq!(
                CStr::from_ptr(out.as_ptr()).to_bytes(),
                b"/tmp/reference.fa"
            );

            out.fill(0);
            let unknown_scheme = CString::new("http://example.invalid/ref.fa").unwrap();
            cram_cram_io_c_4850_full_path(out.as_mut_ptr(), unknown_scheme.as_ptr().cast_mut());
            let expected_unknown = std::env::current_dir()
                .unwrap()
                .join("http://example.invalid/ref.fa");
            assert_eq!(
                CStr::from_ptr(out.as_ptr()).to_bytes(),
                expected_unknown.to_string_lossy().as_bytes()
            );

            out.fill(0);
            let rel = CString::new("ref.fa").unwrap();
            cram_cram_io_c_4850_full_path(out.as_mut_ptr(), rel.as_ptr().cast_mut());
            let expected = std::env::current_dir().unwrap().join("ref.fa");
            assert_eq!(
                CStr::from_ptr(out.as_ptr()).to_bytes(),
                expected.to_string_lossy().as_bytes()
            );
        }
    }

    #[test]
    fn cram_io_free_bam_list_destroys_each_record_and_array() {
        unsafe {
            let bams = calloc(3, std::mem::size_of::<*mut bam1_t>() as u64).cast::<*mut bam1_t>();
            assert!(!bams.is_null());
            *bams.add(0) = crate::htslib_rs::sam::bam_init1();
            *bams.add(1) = crate::htslib_rs::sam::bam_init1();
            *bams.add(2) = std::ptr::null_mut();
            assert!(!(*bams.add(0)).is_null());
            assert!(!(*bams.add(1)).is_null());

            cram_cram_io_c_3695_free_bam_list(bams, 3);
        }
    }

    #[test]
    fn cram_reference_decrement_eviction_frees_previous_unreferenced_cached_seq() {
        unsafe {
            let seq0 = malloc(4).cast::<c_char>();
            let seq1 = malloc(4).cast::<c_char>();
            assert!(!seq0.is_null());
            assert!(!seq1.is_null());

            let mut e0: ref_entry_layout = std::mem::zeroed();
            e0.seq = seq0;
            let mut e1: ref_entry_layout = std::mem::zeroed();
            e1.seq = seq1;
            let mut ref_id = [
                &mut e0 as *mut ref_entry_layout,
                &mut e1 as *mut ref_entry_layout,
            ];
            let mut refs: refs_t_layout = std::mem::zeroed();
            refs.ref_id = ref_id.as_mut_ptr();
            refs.nref = ref_id.len() as c_int;
            refs.last = &mut e1;
            refs.last_id = 1;

            cram_cram_io_c_3169_cram_ref_incr_locked((&mut refs as *mut refs_t_layout).cast(), 0);
            assert_eq!(e0.count, 1);
            assert_eq!(refs.last_id, 1);

            cram_cram_io_c_3189_cram_ref_decr_locked((&mut refs as *mut refs_t_layout).cast(), 0);
            assert_eq!(e0.count, 0);
            assert_eq!(refs.last_id, 0);
            assert!(e1.seq.is_null());

            cram_cram_io_c_2417_ref_entry_free_seq((&mut e0 as *mut ref_entry_layout).cast());
            assert!(e0.seq.is_null());
        }
    }

    #[test]
    fn cram_mfile_seek_tell_rewind_truncate_and_eof_match_field_rules() {
        unsafe {
            let mut mf = mFILE {
                fp: std::ptr::null_mut(),
                data: std::ptr::null_mut(),
                alloced: 0,
                eof: 1,
                mode: 0,
                size: 100,
                offset: 10,
                flush_pos: 0,
            };

            assert_eq!(cram_mFILE_c_451_mfseek(&mut mf, 20, libc::SEEK_SET), 0);
            assert_eq!(cram_mFILE_c_471_mftell(&mut mf), 20);
            assert_eq!(mf.eof, 0);

            assert_eq!(cram_mFILE_c_451_mfseek(&mut mf, 5, libc::SEEK_CUR), 0);
            assert_eq!(mf.offset, 25);
            assert_eq!(cram_mFILE_c_451_mfseek(&mut mf, -10, libc::SEEK_END), 0);
            assert_eq!(mf.offset, 90);

            mf.eof = 1;
            cram_mFILE_c_475_mrewind(&mut mf);
            assert_eq!((mf.offset, mf.eof), (0, 0));

            mf.offset = 80;
            cram_mFILE_c_488_mftruncate(&mut mf, 50);
            assert_eq!((mf.size, mf.offset), (50, 50));
            mf.eof = 7;
            assert_eq!(cram_mFILE_c_494_mfeof(&mut mf), 7);

            assert_eq!(cram_mFILE_c_451_mfseek(&mut mf, 0, -999), -1);
        }
    }

    #[test]
    fn cram_mfile_append_write_and_truncate_minus_one_match_c_edges() {
        unsafe {
            let data = malloc(16).cast::<c_char>();
            std::ptr::copy_nonoverlapping(b"abcdef".as_ptr().cast::<c_char>(), data, 6);
            let mf = cram_mFILE_c_207_mfcreate(data, 6);
            assert!(!mf.is_null());
            (*mf).alloced = 16;
            (*mf).mode = MF_READ | MF_WRITE | MF_APPEND;
            (*mf).offset = 2;
            (*mf).flush_pos = 6;

            let mut suffix = *b"XY";
            assert_eq!(
                cram_mFILE_c_527_mfwrite(suffix.as_mut_ptr().cast(), 1, suffix.len(), mf),
                suffix.len()
            );
            assert_eq!((*mf).offset, 8);
            assert_eq!((*mf).size, 8);
            assert_eq!((*mf).flush_pos, 6);
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"abcdefXY"
            );

            (*mf).offset = 3;
            cram_mFILE_c_488_mftruncate(mf, -1);
            assert_eq!((*mf).size, 3);

            (*mf).mode = MF_READ;
            let mut ignored = *b"zz";
            assert_eq!(
                cram_mFILE_c_527_mfwrite(ignored.as_mut_ptr().cast(), 1, ignored.len(), mf),
                0
            );
            assert_eq!((*mf).size, 3);

            assert_eq!(cram_mFILE_c_408_mfdestroy(mf), 0);
        }
    }

    #[test]
    fn cram_mfile_create_recreate_read_write_and_destroy_follow_memory_ownership() {
        unsafe {
            let mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
            assert!(!mf.is_null());
            assert_eq!((*mf).mode, MF_READ | MF_WRITE);

            let mut input = *b"hello";
            assert_eq!(
                cram_mFILE_c_527_mfwrite(input.as_mut_ptr().cast(), 1, input.len(), mf),
                input.len()
            );
            assert_eq!((*mf).size, 5);
            assert!((*mf).alloced >= 1024);

            cram_mFILE_c_475_mrewind(mf);
            let mut out = [0u8; 8];
            assert_eq!(
                cram_mFILE_c_502_mfread(out.as_mut_ptr().cast(), 2, 4, mf),
                2
            );
            assert_eq!(&out[..5], b"hello");
            assert_eq!((*mf).eof, 1);

            let repl = malloc(4).cast::<c_char>();
            std::ptr::copy_nonoverlapping(c"abc".as_ptr(), repl, 4);
            cram_mFILE_c_225_mfrecreate(mf, repl, 4);
            assert_eq!((*mf).size, 4);
            assert_eq!((*mf).offset, 0);
            assert_eq!(cram_mFILE_c_408_mfdestroy(mf), 0);
            assert_eq!(cram_mFILE_c_408_mfdestroy(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn cram_mfile_flush_and_detach_write_dirty_tail_and_preserve_buffer() {
        unsafe {
            let path =
                std::env::temp_dir().join(format!("htslib_rs-mfile-detach-{}", std::process::id()));
            let _ = std::fs::remove_file(&path);
            let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = libc::fopen(c_path.as_ptr(), c"w+b".as_ptr());
            assert!(!fp.is_null());

            let mf = cram_mFILE_c_264_mfreopen(c_path.as_ptr(), c"w+b".as_ptr(), fp);
            assert!(!mf.is_null());
            let mut bytes = *b"abcdef";
            assert_eq!(
                cram_mFILE_c_527_mfwrite(bytes.as_mut_ptr().cast(), 1, bytes.len(), mf),
                bytes.len()
            );
            (*mf).flush_pos = 3;
            assert_eq!(cram_mFILE_c_389_mfdetach(mf), 0);
            assert!((*mf).fp.is_null());
            assert_eq!((*mf).flush_pos, (*mf).size);
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"abcdef"
            );

            assert_eq!(cram_mFILE_c_408_mfdestroy(mf), 0);
            assert_eq!(std::fs::read(&path).unwrap(), b"\0\0\0def");
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn cram_mfile_load_open_reopen_and_create_from_read_file_contents() {
        unsafe {
            let path =
                std::env::temp_dir().join(format!("htslib_rs-mfile-open-{}", std::process::id()));
            std::fs::write(&path, b"abcdef").unwrap();
            let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let mode = CString::new("rb").unwrap();

            let fp = libc::fopen(c_path.as_ptr(), mode.as_ptr());
            assert!(!fp.is_null());
            let mut size = 0usize;
            let data = cram_mFILE_c_75_mfload(fp, c_path.as_ptr(), &mut size, 1);
            assert!(!data.is_null());
            assert_eq!(size, 6);
            assert_eq!(
                std::slice::from_raw_parts(data.cast::<u8>(), size),
                b"abcdef"
            );
            free(data.cast());
            libc::fclose(fp);

            let mf = cram_mFILE_c_347_mfopen(c_path.as_ptr(), c"rb".as_ptr());
            assert!(!mf.is_null());
            assert_eq!((*mf).size, 6);
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"abcdef"
            );
            assert!(!(*mf).fp.is_null());
            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);

            let fp = libc::fopen(c_path.as_ptr(), mode.as_ptr());
            assert!(!fp.is_null());
            let mf = cram_mFILE_c_246_mfcreate_from(c_path.as_ptr(), c"rb".as_ptr(), fp);
            assert!(!mf.is_null());
            assert!((*mf).fp.is_null());
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), (*mf).size),
                b"abcdef"
            );
            assert_eq!(cram_mFILE_c_408_mfdestroy(mf), 0);
            libc::fclose(fp);

            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn cram_mfile_reopen_write_only_modes_do_not_load_existing_file() {
        unsafe {
            let path = std::env::temp_dir()
                .join(format!("htslib_rs-mfile-write-mode-{}", std::process::id()));
            std::fs::write(&path, b"preexisting").unwrap();
            let c_path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = libc::fopen(c_path.as_ptr(), c"wb".as_ptr());
            assert!(!fp.is_null());

            let mf = cram_mFILE_c_264_mfreopen(c_path.as_ptr(), c"wb".as_ptr(), fp);
            assert!(!mf.is_null());
            assert_eq!((*mf).size, 0);
            assert_eq!((*mf).offset, 0);
            assert_eq!((*mf).flush_pos, 0);
            assert_eq!((*mf).mode & MF_READ, 0);
            assert_ne!((*mf).mode & MF_WRITE, 0);
            assert_ne!((*mf).mode & MF_TRUNC, 0);
            assert_ne!((*mf).mode & MF_BINARY, 0);
            assert!((*mf).data.is_null());

            assert_eq!(cram_mFILE_c_361_mfclose(mf), 0);
            assert_eq!(std::fs::read(&path).unwrap(), b"");
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn cram_mfile_character_line_ascii_and_steal_behaviour_matches_c() {
        unsafe {
            let line_bytes = *b"a\r\nbc\nz\0\0\0\0";
            let data = malloc(12).cast::<c_char>();
            std::ptr::copy_nonoverlapping(line_bytes.as_ptr().cast::<c_char>(), data, 11);
            let mf = cram_mFILE_c_207_mfcreate(data, 7);
            assert!(!mf.is_null());

            assert_eq!(cram_mFILE_c_557_mfgetc(mf), b'a' as c_int);
            assert_eq!(cram_mFILE_c_567_mungetc(b'X' as c_int, mf), b'X' as c_int);
            assert_eq!(cram_mFILE_c_557_mfgetc(mf), b'X' as c_int);

            let mut line = [0 as c_char; 8];
            assert_eq!(
                cram_mFILE_c_577_mfgets(line.as_mut_ptr(), 8, mf),
                line.as_mut_ptr()
            );
            assert_eq!(CStr::from_ptr(line.as_ptr()).to_bytes(), b"\r\n");

            cram_mFILE_c_475_mrewind(mf);
            cram_mFILE_c_656_mfascii(mf);
            assert_eq!((*mf).size, 6);
            assert_eq!(
                std::slice::from_raw_parts((*mf).data.cast::<u8>(), 6),
                b"X\nbc\nz"
            );

            let mut stolen_size = 0usize;
            let stolen = cram_mFILE_c_428_mfsteal(mf, &mut stolen_size);
            assert!(!stolen.is_null());
            assert_eq!(stolen_size, 6);
            free(stolen);
            assert!(cram_mFILE_c_428_mfsteal(std::ptr::null_mut(), std::ptr::null_mut()).is_null());
        }
    }

    #[test]
    fn cram_mfile_empty_read_ungetc_and_gets_set_eof_like_c() {
        unsafe {
            let mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
            assert!(!mf.is_null());

            assert_eq!(cram_mFILE_c_557_mfgetc(mf), -1);
            assert_eq!((*mf).eof, 1);

            (*mf).eof = 0;
            assert_eq!(cram_mFILE_c_567_mungetc(b'Z' as c_int, mf), -1);
            assert_eq!((*mf).eof, 1);

            (*mf).eof = 0;
            let mut line = [b'x' as c_char; 4];
            assert!(cram_mFILE_c_577_mfgets(line.as_mut_ptr(), line.len() as c_int, mf).is_null());
            assert_eq!(line[0], 0);
            assert_eq!((*mf).eof, 1);
            assert_eq!((*mf).offset, 0);

            assert_eq!(cram_mFILE_c_408_mfdestroy(mf), 0);
        }
    }

    #[test]
    fn cram_mfile_stdout_and_stderr_are_singleton_write_channels() {
        unsafe {
            let in1 = cram_mFILE_c_151_mstdin();
            let in2 = cram_mFILE_c_151_mstdin();
            assert!(!in1.is_null());
            assert_eq!(in1, in2);
            assert_eq!((*in1).mode, MF_READ | MF_WRITE);
            assert!(!(*in1).fp.is_null());

            let out1 = cram_mFILE_c_176_mstdout();
            let out2 = cram_mFILE_c_176_mstdout();
            assert!(!out1.is_null());
            assert_eq!(out1, out2);
            assert_eq!((*out1).mode, MF_WRITE);
            assert!(!(*out1).fp.is_null());

            let saved_fp = (*out1).fp;
            let tmp_fp = libc::tmpfile();
            assert!(!tmp_fp.is_null());
            (*out1).fp = tmp_fp;

            let mut msg = *b"x";
            assert_eq!(
                cram_mFILE_c_527_mfwrite((&mut msg as *mut u8).cast(), 1, 1, out1,),
                1
            );
            assert_eq!(cram_mFILE_c_607_mfflush(out1), 0);
            assert_eq!((*out1).offset, 0);
            assert_eq!((*out1).size, 0);
            assert_eq!((*out1).flush_pos, 0);
            (*out1).fp = saved_fp;
            libc::fclose(tmp_fp);

            let err1 = cram_mFILE_c_192_mstderr();
            let err2 = cram_mFILE_c_192_mstderr();
            assert!(!err1.is_null());
            assert_eq!(err1, err2);
            assert_ne!(err1, out1);
            assert_eq!((*err1).mode, MF_WRITE);
            assert!(!(*err1).fp.is_null());
        }
    }

    #[test]
    fn cram_pooled_allocator_rounds_sizes_allocates_and_reuses_free_list() {
        unsafe {
            assert_eq!(cram_pooled_alloc_c_47_next_power_2(0), 0);
            assert_eq!(cram_pooled_alloc_c_47_next_power_2(1), 1);
            assert_eq!(cram_pooled_alloc_c_47_next_power_2(17), 32);

            let p = cram_pooled_alloc_c_64_pool_create(3);
            assert!(!p.is_null());
            assert_eq!((*p).dsize, std::mem::size_of::<*mut c_void>());
            assert_eq!((*p).psize, 8192);
            assert_eq!((*p).npools, 0);

            let a = cram_pooled_alloc_c_115_pool_alloc(p);
            let b = cram_pooled_alloc_c_115_pool_alloc(p);
            assert!(!a.is_null());
            assert!(!b.is_null());
            assert_eq!((*p).npools, 1);
            assert_eq!((*(*p).pools).used, (*p).dsize * 2);
            assert_eq!(
                b.cast::<u8>().offset_from(a.cast::<u8>()) as usize,
                (*p).dsize
            );

            cram_pooled_alloc_c_144_pool_free(p, a);
            let reused = cram_pooled_alloc_c_115_pool_alloc(p);
            assert_eq!(reused, a);

            cram_pooled_alloc_c_84_pool_destroy(p);
        }
    }

    #[test]
    fn cram_pooled_allocator_starts_new_pool_at_exact_capacity_edge() {
        unsafe {
            let p = cram_pooled_alloc_c_64_pool_create(8192);
            assert!(!p.is_null());
            assert_eq!((*p).dsize, 8192);
            assert_eq!((*p).psize, POOLED_ALLOC_PSIZE);

            let slots_before_edge = (*p).psize / (*p).dsize - 1;
            let first = cram_pooled_alloc_c_115_pool_alloc(p);
            assert!(!first.is_null());
            for _ in 1..slots_before_edge {
                assert!(!cram_pooled_alloc_c_115_pool_alloc(p).is_null());
            }
            assert_eq!((*p).npools, 1);
            assert_eq!((*(*p).pools).used, (*p).psize - (*p).dsize);

            let second_pool_first = cram_pooled_alloc_c_115_pool_alloc(p);
            assert!(!second_pool_first.is_null());
            assert_eq!((*p).npools, 2);
            assert_eq!((*(*p).pools.add(1)).used, (*p).dsize);

            cram_pooled_alloc_c_84_pool_destroy(p);
        }
    }

    #[test]
    fn cram_pooled_allocator_disabled_branch_allocates_directly_and_test_main_runs() {
        unsafe {
            let p = cram_pooled_alloc_c_64_pool_create(12);
            assert!(!p.is_null());
            let ptr = cram_pooled_alloc_c_151_pool_alloc(p);
            assert!(!ptr.is_null());
            assert_eq!((*p).npools, 0);
            cram_pooled_alloc_c_155_pool_free(p, ptr);
            cram_pooled_alloc_c_84_pool_destroy(p);

            assert_eq!(cram_pooled_alloc_c_167_main(), 0);
        }
    }

    // ----------------------------------------------------------------------
    // PHASE-0 differential parity oracle.
    //
    // Proves the freshly-ported native container/slice on-disk primitives
    // (`cram_read_container`, `cram_read_block`, `cram_container_size`) parse
    // byte-identically to the reference C implementation in hts-sys.
    //
    // For each real CRAM fixture we open the file twice (two independent fds,
    // both set up via the C `cram_open` which consumes the file definition).
    // On fd_c we read the first container via C `cram_read_container`; on fd_n
    // we read it via the native port. We then assert every on-disk-derived
    // container field is identical. Next we read the compression-header block
    // (advancing past it) and the first slice-header block on BOTH fds — read
    // natively on fd_n, via C on fd_c — decode each slice header (shared C
    // `cram_decode_slice_header`) and assert the parsed slice-header fields
    // match. Internal/alloc-only fields (landmark pointer, offset bookkeeping,
    // comp_hdr, slices array) are deliberately NOT compared as they are not
    // on-disk-derived; `offset` (#header bytes consumed) IS compared because it
    // is fully determined by the parse.
    // ----------------------------------------------------------------------
    #[cfg(feature = "parity")]
    #[test]
    fn cram_read_container_and_slice_header_byte_parity_with_c() {
        // Candidate fixtures, all CRAM v3.0; first existing one is used.
        // Anchor on CARGO_MANIFEST_DIR so the lookup is cwd-independent: other
        // tests (e.g. original_sam_c_mempolicy_runs_sam_to_bam_to_cram_block_io)
        // chdir into a tmpdir and would race a relative-path fixture lookup.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let candidates: [String; 4] = [
            format!("{manifest_dir}/htslib/test/ce#5b_java.cram"),
            format!("{manifest_dir}/htslib/test/auxf#values_java.cram"),
            format!("{manifest_dir}/htslib/test/range.cram"),
            format!("{manifest_dir}/htslib/test/xx#large_aux_java.cram"),
        ];
        let path = candidates
            .iter()
            .find(|p| std::path::Path::new(p).exists())
            .map(String::as_str)
            .expect("no CRAM fixture found under htslib/test/");
        eprintln!("[parity] using fixture: {path}");

        unsafe {
            let c_path = CString::new(path).unwrap();
            let mode = CString::new("rb").unwrap();

            let fd_c = cram_open(c_path.as_ptr(), mode.as_ptr());
            let fd_n = cram_open(c_path.as_ptr(), mode.as_ptr());
            assert!(!fd_c.is_null(), "C cram_open failed");
            assert!(!fd_n.is_null(), "native-host cram_open failed");

            // --- container header ---
            let cc = hts_sys::cram_read_container(fd_c.cast());
            let cn = cram_cram_io_c_3788_cram_read_container(fd_n.cast());
            assert!(!cc.is_null(), "C cram_read_container returned NULL");
            assert!(!cn.is_null(), "native cram_read_container returned NULL");

            let cc = cc.cast::<cram_container_layout>();
            let cn = cn.cast::<cram_container_layout>();

            macro_rules! ceq {
                ($field:ident) => {
                    assert_eq!(
                        (*cc).$field,
                        (*cn).$field,
                        concat!(
                            "container field `",
                            stringify!($field),
                            "` differs (C vs native)"
                        )
                    );
                };
            }
            ceq!(length);
            ceq!(ref_seq_id);
            ceq!(ref_seq_start);
            ceq!(ref_seq_span);
            ceq!(num_records);
            ceq!(record_counter);
            ceq!(num_bases);
            ceq!(num_blocks);
            ceq!(num_landmarks);
            ceq!(offset);
            ceq!(crc32);
            // landmark arrays
            assert_eq!((*cc).num_landmarks, (*cn).num_landmarks);
            for i in 0..(*cc).num_landmarks {
                assert_eq!(
                    *(*cc).landmark.add(i as usize),
                    *(*cn).landmark.add(i as usize),
                    "landmark[{i}] differs (C vs native)"
                );
            }
            // cram_container_size is a pure function of num_landmarks.
            assert_eq!(
                hts_sys::cram_container_size(cc.cast()),
                cram_cram_io_c_3947_cram_container_size(cn.cast()),
            );

            eprintln!(
                "[parity] container: length={} ref_seq_id={} ref_seq_start={} ref_seq_span={} \
                 num_records={} record_counter={} num_bases={} num_blocks={} num_landmarks={} \
                 offset={} crc32={:#x}",
                (*cn).length,
                (*cn).ref_seq_id,
                (*cn).ref_seq_start,
                (*cn).ref_seq_span,
                (*cn).num_records,
                (*cn).record_counter,
                (*cn).num_bases,
                (*cn).num_blocks,
                (*cn).num_landmarks,
                (*cn).offset,
                (*cn).crc32,
            );

            // --- compression header block (read + skip on both fds) ---
            let comp_c = hts_sys::cram_read_block(fd_c.cast());
            let comp_n = cram_cram_io_c_1414_cram_read_block(fd_n.cast());
            assert!(
                !comp_c.is_null() && !comp_n.is_null(),
                "comp-hdr block read failed"
            );
            {
                let bc = comp_c.cast::<cram_block_layout>();
                let bn = comp_n.cast::<cram_block_layout>();
                assert_eq!(
                    (*bc).content_type,
                    (*bn).content_type,
                    "comp-hdr content_type"
                );
                assert_eq!((*bc).content_id, (*bn).content_id, "comp-hdr content_id");
                assert_eq!((*bc).comp_size, (*bn).comp_size, "comp-hdr comp_size");
                assert_eq!((*bc).uncomp_size, (*bn).uncomp_size, "comp-hdr uncomp_size");
                assert_eq!((*bc).crc32, (*bn).crc32, "comp-hdr crc32");
            }

            // --- first slice header block + decoded slice header ---
            let sb_c = hts_sys::cram_read_block(fd_c.cast());
            let sb_n = cram_cram_io_c_1414_cram_read_block(fd_n.cast());
            assert!(
                !sb_c.is_null() && !sb_n.is_null(),
                "slice-hdr block read failed"
            );
            {
                let bc = sb_c.cast::<cram_block_layout>();
                let bn = sb_n.cast::<cram_block_layout>();
                assert_eq!(
                    (*bc).content_type,
                    (*bn).content_type,
                    "slice-hdr content_type"
                );
                assert_eq!((*bc).comp_size, (*bn).comp_size, "slice-hdr comp_size");
                assert_eq!(
                    (*bc).uncomp_size,
                    (*bn).uncomp_size,
                    "slice-hdr uncomp_size"
                );
                assert_eq!((*bc).crc32, (*bn).crc32, "slice-hdr crc32");
            }

            // Differential decode: the C reference (oracle) decodes the fd_c
            // block; the NATIVE port decodes the fd_n block. All on-disk-derived
            // fields must match byte-for-byte.
            //
            // Test-only extern for the C oracle. Production no longer references
            // this symbol (the wrapper + internal call site are native), so the
            // extern lives here, gated to the parity feature.
            unsafe extern "C" {
                #[link_name = "cram_decode_slice_header"]
                fn htslib_cram_decode_slice_header(
                    fd: *mut cram_fd,
                    b: *mut cram_block,
                ) -> *mut cram_block_slice_hdr;
            }
            let hc = htslib_cram_decode_slice_header(fd_c, sb_c.cast());
            let hn = cram_cram_decode_c_955_cram_decode_slice_header(fd_n, sb_n.cast());
            assert!(
                !hc.is_null() && !hn.is_null(),
                "cram_decode_slice_header failed"
            );
            let hc = hc.cast::<cram_block_slice_hdr_layout>();
            let hn = hn.cast::<cram_block_slice_hdr_layout>();

            macro_rules! heq {
                ($field:ident) => {
                    assert_eq!(
                        (*hc).$field,
                        (*hn).$field,
                        concat!(
                            "slice-hdr field `",
                            stringify!($field),
                            "` differs (C vs native)"
                        )
                    );
                };
            }
            heq!(content_type);
            heq!(ref_seq_id);
            heq!(ref_seq_start);
            heq!(ref_seq_span);
            heq!(num_records);
            heq!(record_counter);
            heq!(num_blocks);
            heq!(num_content_ids);
            heq!(ref_base_id);
            for i in 0..(*hc).num_content_ids {
                assert_eq!(
                    *(*hc).block_content_ids.add(i as usize),
                    *(*hn).block_content_ids.add(i as usize),
                    "slice-hdr block_content_ids[{i}] differs (C vs native)"
                );
            }
            assert_eq!((*hc).md5, (*hn).md5, "slice-hdr md5 differs (C vs native)");

            eprintln!(
                "[parity] slice-hdr: content_type={} ref_seq_id={} ref_seq_start={} \
                 ref_seq_span={} num_records={} record_counter={} num_blocks={} \
                 num_content_ids={} ref_base_id={}",
                (*hn).content_type,
                (*hn).ref_seq_id,
                (*hn).ref_seq_start,
                (*hn).ref_seq_span,
                (*hn).num_records,
                (*hn).record_counter,
                (*hn).num_blocks,
                (*hn).num_content_ids,
                (*hn).ref_base_id,
            );

            // Cleanup. Slice-header blocks were consumed by decode; free headers
            // and remaining blocks via the matching allocator side.
            hts_sys::cram_free_slice_header(hc.cast());
            cram_cram_io_c_4409_cram_free_slice_header(hn.cast());
            cram_cram_io_c_1565_cram_free_block(comp_n);
            cram_cram_io_c_1565_cram_free_block(sb_n);
            hts_sys::cram_free_block(comp_c);
            hts_sys::cram_free_block(sb_c);
            cram_cram_io_c_3705_cram_free_container(cn.cast());
            hts_sys::cram_free_container(cc.cast());
            cram_close(fd_c);
            cram_close(fd_n);
        }
    }

    // ----------------------------------------------------------------------
    // DIFFERENTIAL parity test for the NATIVE `cram_decode_compression_header`.
    //
    // For each real CRAM fixture we open the file twice (two independent C
    // `cram_open` fds), read the first container header, then read the
    // compression-header block. On fd_c we decode the comp-hdr via the C oracle
    // `cram_decode_compression_header`; on fd_n we decode it via the NATIVE
    // port. We assert the on-disk-derived comp-hdr fields are equivalent.
    //
    // Fields compared (all fully determined by the byte stream):
    //   * preservation flags: read_names_included, ap_delta, no_ref,
    //     qs_seq_orient, and the 5x4 substitution_matrix.
    //   * TD block: ntl (#tag-lists), td_blk->byte (#bytes) and the raw bytes.
    //   * data-series codecs: for each of the DS_END entries, whether the codec
    //     is present (NULL vs non-NULL) and, when present, the codec enum
    //     (`codec`) and codec_id stored in the cram_codec base struct. The total
    //     count of built codecs is reported.
    //   * tag encoding map: for each of the 32 hash slots, the chain of
    //     (key, encoding, size) tuples plus per-entry codec enum/codec_id.
    //   * rec encoding map: for each of the 32 hash slots, the chain of
    //     (key, encoding, size, offset) tuples (rec-map codecs are always NULL
    //     in both, by construction).
    //
    // Fields deliberately EXCLUDED, with reason:
    //   * The `preservation_map` khash internal layout (n_buckets, bucket
    //     positions, flag words): not on-disk-derived; depends only on insertion
    //     order which is identical, but bucket indices are an implementation
    //     detail. The semantically meaningful results are mirrored into the
    //     public preservation flags, which ARE compared.
    //   * All pointer-identity fields (codec object pointers, td_blk pointer,
    //     tl[] pointers, preservation_map SM/TD `hd.p` pointers, cram_map next
    //     pointers, `data`): these are freshly allocated and necessarily differ.
    //     We compare the codec *descriptions* (codec enum + codec_id) instead.
    //   * `landmark`/`num_landmarks`/`ref_seq_*`/`num_records`: only populated
    //     for CRAM v1.0 comp-hdrs; the available fixtures are v2/v3 so these are
    //     all zero in both and not interesting to compare individually (still
    //     covered implicitly by the codec/map parse succeeding).
    // ----------------------------------------------------------------------
    #[cfg(feature = "parity")]
    #[test]
    fn cram_decode_compression_header_byte_parity_with_c() {
        // Anchor on CARGO_MANIFEST_DIR so the lookup is cwd-independent: other
        // tests (e.g. original_sam_c_mempolicy_runs_sam_to_bam_to_cram_block_io)
        // chdir into a tmpdir and would race a relative-path fixture lookup.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let candidates: [String; 4] = [
            format!("{manifest_dir}/htslib/test/ce#5b_java.cram"),
            format!("{manifest_dir}/htslib/test/auxf#values_java.cram"),
            format!("{manifest_dir}/htslib/test/range.cram"),
            format!("{manifest_dir}/htslib/test/xx#large_aux_java.cram"),
        ];

        unsafe extern "C" {
            #[link_name = "cram_decode_compression_header"]
            fn htslib_cram_decode_compression_header(
                fd: *mut cram_fd,
                b: *mut cram_block,
            ) -> *mut cram_block_compression_hdr;
        }

        let mut tested = 0usize;
        for path in candidates.iter().map(String::as_str) {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            eprintln!("[parity] comp-hdr fixture: {path}");
            unsafe {
                let c_path = CString::new(path).unwrap();
                let mode = CString::new("rb").unwrap();
                let fd_c = cram_open(c_path.as_ptr(), mode.as_ptr());
                let fd_n = cram_open(c_path.as_ptr(), mode.as_ptr());
                assert!(!fd_c.is_null() && !fd_n.is_null(), "cram_open failed");

                let cc = hts_sys::cram_read_container(fd_c.cast());
                let cn = cram_cram_io_c_3788_cram_read_container(fd_n.cast());
                assert!(!cc.is_null() && !cn.is_null(), "cram_read_container failed");

                let comp_c = hts_sys::cram_read_block(fd_c.cast());
                let comp_n = cram_cram_io_c_1414_cram_read_block(fd_n.cast());
                assert!(
                    !comp_c.is_null() && !comp_n.is_null(),
                    "comp-hdr read failed"
                );

                let hc = htslib_cram_decode_compression_header(fd_c, comp_c.cast());
                let hn = cram_cram_decode_c_145_cram_decode_compression_header(fd_n, comp_n.cast());
                assert!(
                    !hc.is_null(),
                    "C cram_decode_compression_header returned NULL"
                );
                assert!(
                    !hn.is_null(),
                    "native cram_decode_compression_header returned NULL"
                );
                let hc = hc.cast::<cram_block_compression_hdr_layout>();
                let hn = hn.cast::<cram_block_compression_hdr_layout>();

                // --- preservation flags ---
                assert_eq!(
                    (*hc).read_names_included,
                    (*hn).read_names_included,
                    "read_names_included"
                );
                assert_eq!((*hc).ap_delta, (*hn).ap_delta, "ap_delta");
                assert_eq!((*hc).no_ref, (*hn).no_ref, "no_ref");
                assert_eq!((*hc).qs_seq_orient, (*hn).qs_seq_orient, "qs_seq_orient");
                assert_eq!(
                    (*hc).substitution_matrix,
                    (*hn).substitution_matrix,
                    "substitution_matrix"
                );

                // --- TD block ---
                assert_eq!((*hc).ntl, (*hn).ntl, "ntl");
                let tdc = (*hc).td_blk;
                let tdn = (*hn).td_blk;
                assert_eq!(tdc.is_null(), tdn.is_null(), "td_blk presence");
                if !tdc.is_null() {
                    assert_eq!((*tdc).byte, (*tdn).byte, "td_blk byte count");
                    let a = std::slice::from_raw_parts((*tdc).data, (*tdc).byte);
                    let b = std::slice::from_raw_parts((*tdn).data, (*tdn).byte);
                    assert_eq!(a, b, "td_blk bytes");
                }

                // --- data-series codecs ---
                let mut ncodec_c = 0;
                let mut ncodec_n = 0;
                for ds in 0..CRAM_DS_END {
                    let cc_ = (*hc).codecs[ds].cast::<cram_codec_base_layout>();
                    let cn_ = (*hn).codecs[ds].cast::<cram_codec_base_layout>();
                    assert_eq!(
                        cc_.is_null(),
                        cn_.is_null(),
                        "codecs[{ds}] presence differs"
                    );
                    if !cc_.is_null() {
                        ncodec_c += 1;
                        ncodec_n += 1;
                        assert_eq!((*cc_).codec, (*cn_).codec, "codecs[{ds}] codec enum");
                        assert_eq!((*cc_).codec_id, (*cn_).codec_id, "codecs[{ds}] codec_id");
                    }
                }
                assert_eq!(ncodec_c, ncodec_n);

                // --- rec encoding map (key/encoding/size/offset chains) ---
                for slot in 0..32usize {
                    let mut mc = (*hc).rec_encoding_map[slot].cast::<cram_map_layout>();
                    let mut mn = (*hn).rec_encoding_map[slot].cast::<cram_map_layout>();
                    while !mc.is_null() || !mn.is_null() {
                        assert_eq!(mc.is_null(), mn.is_null(), "rec_map[{slot}] chain length");
                        assert_eq!((*mc).key, (*mn).key, "rec_map[{slot}] key");
                        assert_eq!((*mc).encoding, (*mn).encoding, "rec_map[{slot}] encoding");
                        assert_eq!((*mc).size, (*mn).size, "rec_map[{slot}] size");
                        assert_eq!((*mc).offset, (*mn).offset, "rec_map[{slot}] offset");
                        mc = (*mc).next;
                        mn = (*mn).next;
                    }
                }

                // --- tag encoding map (key/encoding/size + codec descriptions) ---
                let mut ntag_c = 0;
                let mut ntag_n = 0;
                for slot in 0..32usize {
                    let mut mc = (*hc).tag_encoding_map[slot].cast::<cram_map_layout>();
                    let mut mn = (*hn).tag_encoding_map[slot].cast::<cram_map_layout>();
                    while !mc.is_null() || !mn.is_null() {
                        assert_eq!(mc.is_null(), mn.is_null(), "tag_map[{slot}] chain length");
                        ntag_c += 1;
                        ntag_n += 1;
                        assert_eq!((*mc).key, (*mn).key, "tag_map[{slot}] key");
                        assert_eq!((*mc).encoding, (*mn).encoding, "tag_map[{slot}] encoding");
                        assert_eq!((*mc).size, (*mn).size, "tag_map[{slot}] size");
                        let cdc = (*mc).codec.cast::<cram_codec_base_layout>();
                        let cdn = (*mn).codec.cast::<cram_codec_base_layout>();
                        assert_eq!(
                            cdc.is_null(),
                            cdn.is_null(),
                            "tag_map[{slot}] codec presence"
                        );
                        if !cdc.is_null() {
                            assert_eq!((*cdc).codec, (*cdn).codec, "tag_map[{slot}] codec enum");
                            assert_eq!(
                                (*cdc).codec_id,
                                (*cdn).codec_id,
                                "tag_map[{slot}] codec_id"
                            );
                        }
                        mc = (*mc).next;
                        mn = (*mn).next;
                    }
                }
                assert_eq!(ntag_c, ntag_n);

                eprintln!(
                    "[parity] comp-hdr: read_names_included={} ap_delta={} no_ref={} \
                     qs_seq_orient={} ntl={} ds_codecs={} tag_codecs={}",
                    (*hn).read_names_included,
                    (*hn).ap_delta,
                    (*hn).no_ref,
                    (*hn).qs_seq_orient,
                    (*hn).ntl,
                    ncodec_n,
                    ntag_n,
                );

                // Cleanup: free each header via the allocator that built it.
                htslib_cram_free_compression_header(hc.cast());
                cram_cram_io_c_4356_cram_free_compression_header(hn.cast());
                cram_cram_io_c_1565_cram_free_block(comp_n);
                hts_sys::cram_free_block(comp_c);
                cram_cram_io_c_3705_cram_free_container(cn.cast());
                hts_sys::cram_free_container(cc.cast());
                cram_close(fd_c);
                cram_close(fd_n);
            }
            tested += 1;
        }
        assert!(tested > 0, "no CRAM fixture found under htslib/test/");
        eprintln!("[parity] comp-hdr fixtures tested: {tested}");
    }

    // End-to-end differential READ parity: decode every record of several CRAM
    // fixtures two ways and assert byte-for-byte equality of the resulting
    // bam1_t (core fields + the full .data block: qname, cigar, seq, qual, aux).
    //   (A) C oracle: hts_sys::cram_get_bam_seq pumps the C decode pipeline.
    //   (B) Native:   decode_pipeline::cram_get_bam_seq drives the ported
    //                 cram_get_seq + cram_to_bam, reusing the same fd layout.
    // cram_open stays the C/delegating open (builds the fd + refs); the
    // reference is set via cram_set_option(CRAM_OPT_REFERENCE) on both fds.
    #[cfg(feature = "parity")]
    #[test]
    fn cram_native_decode_pipeline_record_parity_with_c() {
        use crate::htslib_rs::cram::decode_pipeline;
        use crate::htslib_rs::sam::{bam1_t, bam_destroy1, bam_init1};

        // (cram fixture, reference fasta or None for no-ref/unmapped).
        // Anchor on CARGO_MANIFEST_DIR so the lookup is cwd-independent: other
        // tests (e.g. original_sam_c_mempolicy_runs_sam_to_bam_to_cram_block_io)
        // chdir into a tmpdir and would race a relative-path fixture lookup.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fixtures: Vec<(String, Option<String>)> = vec![
            (
                format!("{manifest_dir}/htslib/test/ce#5b_java.cram"),
                Some(format!("{manifest_dir}/htslib/test/ce.fa")),
            ),
            (
                format!("{manifest_dir}/htslib/test/auxf#values_java.cram"),
                Some(format!("{manifest_dir}/htslib/test/auxf.fa")),
            ),
            (
                format!("{manifest_dir}/htslib/test/xx#large_aux_java.cram"),
                Some(format!("{manifest_dir}/htslib/test/xx.fa")),
            ),
            // range.cram exercises multi-container pumping; its @SQ refs are
            // the CHROMOSOME_* sequences from ce.fa.
            (
                format!("{manifest_dir}/htslib/test/range.cram"),
                Some(format!("{manifest_dir}/htslib/test/ce.fa")),
            ),
        ];

        // Test-only externs for the C decode-pipeline oracle. Production no
        // longer references these symbols (the native ports are additive), so
        // the externs live here, gated to the parity feature.
        unsafe extern "C" {
            #[link_name = "cram_get_bam_seq"]
            fn htslib_cram_get_bam_seq(fd: *mut cram_fd, bam: *mut *mut c_void) -> c_int;
        }

        let mut tested = 0usize;
        for (cram, reff) in fixtures.iter() {
            let cram: &str = cram.as_str();
            let reff: Option<&str> = reff.as_deref();
            if !std::path::Path::new(cram).exists() {
                continue;
            }
            unsafe {
                let c_path = CString::new(cram).unwrap();
                let mode = CString::new("rb").unwrap();
                let fd_c = cram_open(c_path.as_ptr(), mode.as_ptr());
                let fd_n = cram_open(c_path.as_ptr(), mode.as_ptr());
                assert!(
                    !fd_c.is_null() && !fd_n.is_null(),
                    "cram_open failed for {cram}"
                );

                // Saved so we can restore + free the C-owned refs before
                // cram_close (the native refs_load_fai result uses the native
                // ref_entry layout and must be freed natively, not by libhts).
                let mut orig_refs_n: *mut refs_t_layout = std::ptr::null_mut();
                let mut native_refs_n: *mut refs_t_layout = std::ptr::null_mut();
                if let Some(rf) = reff {
                    if std::path::Path::new(rf).exists() {
                        let rf_c = CString::new(rf).unwrap();
                        // C oracle: let libhts load the reference (CRAM_OPT_REFERENCE=9).
                        hts_sys::cram_set_option(fd_c.cast(), 9, rf_c.as_ptr());
                        // Native fd: build refs with the NATIVE loader so the whole
                        // native decode pipeline (cram_get_ref/load_ref_portion) reads
                        // refs in the native ref_entry layout. This mirrors what
                        // cram_load_reference does (refs_load_fai + sanitise + refs2id).
                        let fnl = fd_n.cast::<cram_fd_layout>();
                        orig_refs_n = (*fnl).refs;
                        let new_refs = cram_cram_io_c_2541_refs_load_fai(
                            std::ptr::null_mut(),
                            rf_c.as_ptr(),
                            0,
                        );
                        native_refs_n = new_refs.cast();
                        assert!(!new_refs.is_null(), "{cram}: native refs_load_fai failed");
                        cram_cram_io_c_2693_sanitise_SQ_lines(fd_n);
                        (*fnl).refs = new_refs.cast();
                        let rl = new_refs.cast::<refs_t_layout>();
                        if !(*rl).fp.is_null() {
                            crate::htslib_rs::bgzf::bgzf_close((*rl).fp);
                            (*rl).fp = std::ptr::null_mut();
                        }
                        assert_eq!(
                            cram_cram_io_c_2737_refs2id(new_refs.cast(), (*fnl).header.cast()),
                            0,
                            "{cram}: native refs2id failed"
                        );
                    }
                }

                // Each fd needs the SAM header loaded for decode. cram_open did
                // that; grab it for both pipelines (they use their own fd's hdr).
                let mut bam_c: *mut bam1_t = bam_init1();
                let mut bam_n: *mut bam1_t = bam_init1();
                let mut nrec = 0usize;
                loop {
                    let rc = htslib_cram_get_bam_seq(fd_c, (&mut bam_c as *mut *mut bam1_t).cast());
                    let rn = decode_pipeline::cram_get_bam_seq(
                        fd_n.cast(),
                        (&mut bam_n as *mut *mut bam1_t).cast(),
                    );
                    if rc < 0 || rn < 0 {
                        assert_eq!(
                            rc < 0,
                            rn < 0,
                            "{cram}: EOF/error mismatch at record {nrec} (C rc={rc}, native rn={rn})"
                        );
                        break;
                    }
                    assert!(
                        !bam_c.is_null() && !bam_n.is_null(),
                        "{cram}: null bam at {nrec}"
                    );

                    let cc = &(*bam_c).core;
                    let cn = &(*bam_n).core;
                    macro_rules! feq {
                        ($f:ident) => {
                            assert_eq!(
                                cc.$f,
                                cn.$f,
                                "{}: record {} core.{} differs (C vs native)",
                                cram,
                                nrec,
                                stringify!($f)
                            );
                        };
                    }
                    feq!(tid);
                    feq!(pos);
                    feq!(flag);
                    feq!(qual);
                    feq!(l_qname);
                    feq!(n_cigar);
                    feq!(l_qseq);
                    feq!(mtid);
                    feq!(mpos);
                    feq!(isize);

                    let lc = (*bam_c).l_data;
                    let ln = (*bam_n).l_data;
                    assert_eq!(
                        lc, ln,
                        "{cram}: record {nrec} l_data differs (C={lc} native={ln})"
                    );
                    let dc = std::slice::from_raw_parts((*bam_c).data, lc as usize);
                    let dn = std::slice::from_raw_parts((*bam_n).data, ln as usize);
                    assert_eq!(
                        dc, dn,
                        "{cram}: record {nrec} bam1_t data block differs (qname/cigar/seq/qual/aux)"
                    );
                    nrec += 1;
                }

                eprintln!("[parity] {cram}: {nrec} records matched byte-for-byte");
                assert!(nrec > 0, "{cram}: decoded zero records");

                bam_destroy1(bam_c);
                bam_destroy1(bam_n);
                // Restore the libhts-owned refs on fd_n and free the
                // native-built refs ourselves (incompatible ABI with libhts'
                // refs_free), then close both fds with libhts cram_close.
                if !native_refs_n.is_null() {
                    let fnl = fd_n.cast::<cram_fd_layout>();
                    (*fnl).refs = orig_refs_n;
                    cram_cram_io_c_2427_refs_free(native_refs_n.cast());
                }
                cram_close(fd_c);
                cram_close(fd_n);
                tested += 1;
            }
        }
        assert!(tested > 0, "no CRAM read fixture found under htslib/test/");
        eprintln!("[parity] native decode-pipeline fixtures tested: {tested}");
    }
}
