use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    io::Read,
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
    HTS_PROFILE_ARCHIVE,
    HTS_PROFILE_FAST, HTS_PROFILE_NORMAL, HTS_PROFILE_SMALL,
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

#[allow(unused_assignments)]
#[path = "cram/cram_external.rs"]
mod cram_external;

pub type cram_block = hts_sys::cram_block;
pub type cram_container = hts_sys::cram_container;
pub type cram_block_compression_hdr = hts_sys::cram_block_compression_hdr;
pub type cram_block_slice_hdr = hts_sys::cram_block_slice_hdr;
pub type cram_metrics = hts_sys::cram_metrics;
pub type refs_t = hts_sys::refs_t;
pub type cram_content_type = hts_sys::cram_content_type;
pub type cram_block_method = hts_sys::cram_block_method;

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
pub const CRAM_CONTENT_TYPE_EXTERNAL: cram_content_type = 4;
pub const CRAM_CONTENT_TYPE_CORE: cram_content_type = 5;

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
pub const HTS_IDX_DELIM: &[u8; 8] = b"##idx##\0";

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

unsafe extern "C" {
    #[link_name = "zlib_mem_inflate"]
    fn htslib_zlib_mem_inflate(cdata: *mut c_char, csize: usize, size: *mut usize) -> *mut c_char;
    // rans_uncompress / fqz_decompress / rans_uncompress_4x16 / arith_uncompress_to /
    // tok3_decode_names are now served by the native `htscodecs` modules (see
    // cram_uncompress_block below) — the libhts externs were removed.
    #[link_name = "cram_free_compression_header"]
    fn htslib_cram_free_compression_header(hdr: *mut cram_block_compression_hdr);
    #[link_name = "cram_free_slice_header"]
    fn htslib_cram_free_slice_header(hdr: *mut cram_block_slice_hdr);
    #[link_name = "cram_decode_compression_header"]
    fn htslib_cram_decode_compression_header(
        fd: *mut cram_fd,
        b: *mut cram_block,
    ) -> *mut cram_block_compression_hdr;
    #[link_name = "cram_decode_slice_header"]
    fn htslib_cram_decode_slice_header(
        fd: *mut cram_fd,
        b: *mut cram_block,
    ) -> *mut cram_block_slice_hdr;
    #[link_name = "cram_num_containers_between"]
    fn htslib_cram_num_containers_between(
        fd: *mut cram_fd,
        cstart: libc::off_t,
        cend: libc::off_t,
        first: *mut i64,
        last: *mut i64,
    ) -> i64;
    #[link_name = "cram_num_containers"]
    fn htslib_cram_num_containers(fd: *mut cram_fd) -> i64;
    #[link_name = "cram_container_num2offset"]
    fn htslib_cram_container_num2offset(fd: *mut cram_fd, num: i64) -> libc::off_t;
    #[link_name = "cram_container_offset2num"]
    fn htslib_cram_container_offset2num(fd: *mut cram_fd, pos: libc::off_t) -> i64;
    #[link_name = "cram_index_extents"]
    fn htslib_cram_index_extents(
        fd: *mut cram_fd,
        refid: c_int,
        start: i64,
        end: i64,
        first: *mut libc::off_t,
        last: *mut libc::off_t,
    ) -> c_int;
    #[link_name = "cram_describe_encodings"]
    fn htslib_cram_describe_encodings(
        hdr: *mut cram_block_compression_hdr,
        ks: *mut kstring_t,
    ) -> c_int;
    #[link_name = "cram_expand_method"]
    fn htslib_cram_expand_method(
        data: *mut u8,
        size: i32,
        comp: cram_block_method,
    ) -> *mut cram_method_details;
    #[link_name = "cram_codec_get_content_ids"]
    fn htslib_cram_codec_get_content_ids(c: *mut cram_codec, ids: *mut c_int);
    #[link_name = "cram_codec_describe"]
    fn htslib_cram_codec_describe(c: *mut cram_codec, ks: *mut kstring_t) -> c_int;
    #[link_name = "hts_pack"]
    fn htscodecs_hts_pack(
        data: *mut u8,
        len: i64,
        out_meta: *mut u8,
        out_meta_len: *mut c_int,
        out_len: *mut u64,
    ) -> *mut u8;
    #[link_name = "hts_rle_encode"]
    fn htscodecs_hts_rle_encode(
        data: *mut u8,
        data_len: u64,
        run: *mut u8,
        run_len: *mut u64,
        rle_syms: *mut u8,
        rle_nsyms: *mut c_int,
        out: *mut u8,
        out_len: *mut u64,
    ) -> *mut u8;
    #[link_name = "hts_rle_decode"]
    fn htscodecs_hts_rle_decode(
        lit: *mut u8,
        lit_len: u64,
        run: *mut u8,
        run_len: u64,
        rle_syms: *mut u8,
        rle_nsyms: c_int,
        out: *mut u8,
        out_len: *mut u64,
    ) -> *mut u8;
    #[link_name = "sam_hdr_fill_hrecs"]
    fn htslib_sam_hdr_fill_hrecs(h: *mut sam_hdr_t) -> c_int;
    #[link_name = "sam_hrecs_find_type_id"]
    fn htslib_sam_hrecs_find_type_id(
        hrecs: *mut c_void,
        type_: *const c_char,
        id_key: *const c_char,
        id_value: *const c_char,
    ) -> *mut c_void;
    // MD5 helpers (htslib/hts.c) — used by the reference-cache populate path.
    #[link_name = "hts_md5_init"]
    fn htslib_hts_md5_init() -> *mut c_void;
    #[link_name = "hts_md5_update"]
    fn htslib_hts_md5_update(ctx: *mut c_void, data: *const c_void, size: std::ffi::c_ulong);
    #[link_name = "hts_md5_final"]
    fn htslib_hts_md5_final(digest: *mut u8, ctx: *mut c_void);
    #[link_name = "hts_md5_destroy"]
    fn htslib_hts_md5_destroy(ctx: *mut c_void);
    #[link_name = "hts_md5_hex"]
    fn htslib_hts_md5_hex(hex: *mut c_char, digest: *const u8);
    // Temporary-file helper (htslib/hts.c) — used when writing into REF_CACHE.
    #[link_name = "hts_open_tmpfile"]
    fn htslib_hts_open_tmpfile(
        fname: *const c_char,
        mode: *const c_char,
        tmpname: *mut kstring_t,
    ) -> *mut hFILE;
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
    hts_sys::cram_read_block(fd.cast())
}

pub unsafe fn int32_put_blk(b: *mut cram_block, val: i32) -> c_int {
    hts_sys::int32_put_blk(b, val)
}

pub unsafe fn int32_get_blk(b: *mut cram_block, val: *mut i32) -> c_int {
    let block = b.cast::<cram_block_layout>();
    if (*block).uncomp_size < 0 || ((*block).uncomp_size as usize).saturating_sub((*block).byte) < 4
    {
        return -1;
    }

    let data = (*block).data.add((*block).byte);
    let v = (*data as u32)
        | ((*data.add(1) as u32) << 8)
        | ((*data.add(2) as u32) << 16)
        | ((*data.add(3) as u32) << 24);
    *val = v as i32;
    (*block).byte += 4;
    4
}

pub unsafe fn cram_block_size(b: *mut cram_block) -> u32 {
    cram_cram_io_c_1490_cram_block_size(b)
}

pub unsafe fn cram_write_block(fd: *mut cram_fd, b: *mut cram_block) -> c_int {
    hts_sys::cram_write_block(fd.cast(), b)
}

pub unsafe fn cram_uncompress_block(b: *mut cram_block) -> c_int {
    hts_sys::cram_uncompress_block(b)
}

pub unsafe fn cram_compress_block(
    fd: *mut cram_fd,
    b: *mut cram_block,
    metrics: *mut cram_metrics,
    method: c_int,
    level: c_int,
) -> c_int {
    hts_sys::cram_compress_block(fd.cast(), b, metrics, method, level)
}

pub unsafe fn cram_set_header(fd: *mut cram_fd, hdr: *mut sam_hdr_t) -> c_int {
    hts_sys::cram_set_header(fd.cast(), hdr.cast())
}

pub unsafe fn cram_new_container(nrec: c_int, nslice: c_int) -> *mut cram_container {
    hts_sys::cram_new_container(nrec, nslice)
}

pub unsafe fn cram_free_container(c: *mut cram_container) {
    hts_sys::cram_free_container(c)
}

pub unsafe fn cram_free_compression_header(hdr: *mut cram_block_compression_hdr) {
    cram_cram_io_c_4356_cram_free_compression_header(hdr)
}

pub unsafe fn cram_free_slice_header(hdr: *mut cram_block_slice_hdr) {
    unsafe { htslib_cram_free_slice_header(hdr) }
}

pub unsafe fn cram_decode_compression_header(
    fd: *mut cram_fd,
    b: *mut cram_block,
) -> *mut cram_block_compression_hdr {
    unsafe { htslib_cram_decode_compression_header(fd, b) }
}

pub unsafe fn cram_decode_slice_header(
    fd: *mut cram_fd,
    b: *mut cram_block,
) -> *mut cram_block_slice_hdr {
    unsafe { htslib_cram_decode_slice_header(fd, b) }
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
    hts_sys::cram_read_container(fd.cast())
}

pub unsafe fn cram_container_size(c: *mut cram_container) -> c_int {
    hts_sys::cram_container_size(c)
}

pub unsafe fn cram_store_container(
    fd: *mut cram_fd,
    c: *mut cram_container,
    dat: *mut c_char,
    size: *mut c_int,
) -> c_int {
    hts_sys::cram_store_container(fd.cast(), c, dat, size)
}

pub unsafe fn cram_write_container(fd: *mut cram_fd, h: *mut cram_container) -> c_int {
    hts_sys::cram_write_container(fd.cast(), h)
}

pub unsafe fn cram_num_containers_between(
    fd: *mut cram_fd,
    cstart: libc::off_t,
    cend: libc::off_t,
    first: *mut i64,
    last: *mut i64,
) -> i64 {
    unsafe { htslib_cram_num_containers_between(fd, cstart, cend, first, last) }
}

pub unsafe fn cram_num_containers(fd: *mut cram_fd) -> i64 {
    unsafe { htslib_cram_num_containers(fd) }
}

pub unsafe fn cram_container_num2offset(fd: *mut cram_fd, num: i64) -> libc::off_t {
    unsafe { htslib_cram_container_num2offset(fd, num) }
}

pub unsafe fn cram_container_offset2num(fd: *mut cram_fd, pos: libc::off_t) -> i64 {
    unsafe { htslib_cram_container_offset2num(fd, pos) }
}

pub unsafe fn cram_index_extents(
    fd: *mut cram_fd,
    refid: c_int,
    start: i64,
    end: i64,
    first: *mut libc::off_t,
    last: *mut libc::off_t,
) -> c_int {
    unsafe { htslib_cram_index_extents(fd, refid, start, end, first, last) }
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

pub unsafe fn cram_describe_encodings(
    hdr: *mut cram_block_compression_hdr,
    ks: *mut kstring_t,
) -> c_int {
    unsafe { htslib_cram_describe_encodings(hdr, ks) }
}

pub unsafe fn cram_expand_method(
    data: *mut u8,
    size: i32,
    comp: cram_block_method,
) -> *mut cram_method_details {
    unsafe { htslib_cram_expand_method(data, size, comp) }
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
    unsafe {
        cram_external::cram_cram_external_c_776_cram_filter_container(
            in_.cast(),
            out.cast(),
            c,
            ref_id,
        )
    }
}

pub unsafe fn cram_open(filename: *const c_char, mode: *const c_char) -> *mut cram_fd {
    hts_sys::cram_open(filename, mode).cast()
}

pub unsafe fn cram_dopen(
    fp: *mut hFILE,
    filename: *const c_char,
    mode: *const c_char,
) -> *mut cram_fd {
    hts_sys::cram_dopen(fp.cast(), filename, mode).cast()
}

pub unsafe fn cram_seek(fd: *mut cram_fd, offset: libc::off_t, whence: c_int) -> c_int {
    hts_sys::cram_seek(fd.cast(), offset, whence)
}

pub unsafe fn cram_flush(fd: *mut cram_fd) -> c_int {
    hts_sys::cram_flush(fd.cast())
}

pub unsafe fn cram_close(fd: *mut cram_fd) -> c_int {
    hts_sys::cram_close(fd.cast())
}

pub unsafe fn cram_eof(fd: *mut cram_fd) -> c_int {
    hts_sys::cram_eof(fd.cast())
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

unsafe fn cram_voption_va_arg_ptr<T>(args: *mut crate::htslib_rs::c_compat::__va_list_tag) -> *mut T {
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
        libc::pthread_mutex_lock(&mut (*fd).range_lock);
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
        libc::pthread_mutex_unlock(&mut (*fd).range_lock);
        0
    }
}

// original: cram_set_voption (htslib/cram/cram_io.c:5692)
pub unsafe fn cram_cram_io_c_5692_cram_set_voption(
    fd: *mut cram_fd,
    opt: hts_fmt_option,
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> c_int {
    unsafe {
        if fd.is_null() {
            *__errno_location() = libc::EBADF;
            return -1;
        }

        let fdl = fd.cast::<cram_fd_layout>();
        match opt {
            x if x == crate::htslib_rs::cram::CRAM_OPT_DECODE_MD => {
                (*fdl).decode_md = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_PREFIX => {
                let prefix = cram_voption_va_arg_ptr::<c_char>(args);
                free((*fdl).prefix.cast());
                (*fdl).prefix = if prefix.is_null() {
                    std::ptr::null_mut()
                } else {
                    strdup(prefix)
                };
                if !prefix.is_null() && (*fdl).prefix.is_null() {
                    return -1;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_VERBOSITY => {}
            x if x == crate::htslib_rs::cram::CRAM_OPT_SEQS_PER_SLICE => {
                (*fdl).seqs_per_slice = cram_voption_va_arg_int(args);
                if (*fdl).bases_per_slice == CRAM_DEFAULT_BASES_PER_SLICE {
                    (*fdl).bases_per_slice = (*fdl).seqs_per_slice * 500;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_BASES_PER_SLICE => {
                (*fdl).bases_per_slice = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_SLICES_PER_CONTAINER => {
                (*fdl).slices_per_container = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_EMBED_REF => {
                (*fdl).embed_ref = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_NO_REF => {
                (*fdl).no_ref = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_POS_DELTA => {
                (*fdl).ap_delta = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_IGNORE_MD5 => {
                (*fdl).ignore_md5 = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_LOSSY_NAMES => {
                (*fdl).lossy_read_names = cram_voption_va_arg_int(args);
                (*fdl).tlen_approx = (*fdl).lossy_read_names;
                (*fdl).tlen_zero = (*fdl).lossy_read_names;
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_USE_BZIP2 => {
                (*fdl).use_bz2 = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_USE_RANS => {
                (*fdl).use_rans = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_USE_TOK => {
                (*fdl).use_tok = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_USE_FQZ => {
                (*fdl).use_fqz = cram_voption_va_arg_int(args);
            }
            x if x == CRAM_OPT_USE_ARITH => {
                (*fdl).use_arith = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_USE_LZMA => {
                (*fdl).use_lzma = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_SHARED_REF => {
                (*fdl).shared_ref = 1;
                let refs = cram_voption_va_arg_ptr::<hts_sys::refs_t>(args);
                if refs != (*fdl).refs {
                    if !(*fdl).refs.is_null() {
                        cram_cram_io_c_2427_refs_free((*fdl).refs.cast());
                    }
                    (*fdl).refs = refs;
                    if !(*fdl).refs.is_null() {
                        (*(*fdl).refs.cast::<refs_t_layout>()).count += 1;
                    }
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_RANGE => {
                // Native va_list element is layout-identical to the C one; cast
                // the pointer when forwarding to the C library.
                return hts_sys::cram_set_voption(fd.cast(), opt, args.cast());
            }
            x if x == CRAM_OPT_RANGE_NOSEEK => {
                return cram_voption_set_range_noseek(
                    fd,
                    cram_voption_va_arg_ptr::<cram_range_layout>(args),
                );
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_REFERENCE => {
                return cram_cram_io_c_3597_cram_load_reference(
                    fd,
                    cram_voption_va_arg_ptr::<c_char>(args),
                );
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_VERSION => {
                return cram_voption_set_version(fd, cram_voption_va_arg_ptr::<c_char>(args));
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_MULTI_SEQ_PER_SLICE => {
                let multi_seq = cram_voption_va_arg_int(args);
                (*fdl).multi_seq = multi_seq;
                (*fdl).multi_seq_user = multi_seq;
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_NTHREADS => {
                let nthreads = cram_voption_va_arg_int(args);
                if !(*fdl).pool.is_null() {
                    return -2;
                }
                if nthreads >= 1 {
                    (*fdl).pool = hts_tpool_init(nthreads).cast();
                    if (*fdl).pool.is_null() {
                        return -1;
                    }
                    (*fdl).rqueue =
                        hts_tpool_process_init((*fdl).pool.cast(), nthreads * 2, 0).cast();
                    (*fdl).shared_ref = 1;
                    (*fdl).own_pool = 1;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_THREAD_POOL => {
                let p = cram_voption_va_arg_ptr::<hts_sys::htsThreadPool>(args);
                if !(*fdl).pool.is_null() {
                    return -2;
                }
                (*fdl).pool = if p.is_null() {
                    std::ptr::null_mut()
                } else {
                    (*p).pool.cast()
                };
                if !(*fdl).pool.is_null() {
                    let qsize = if (*p).qsize != 0 {
                        (*p).qsize
                    } else {
                        hts_tpool_size((*fdl).pool.cast()) * 2
                    };
                    (*fdl).rqueue = hts_tpool_process_init((*fdl).pool.cast(), qsize, 0).cast();
                }
                (*fdl).shared_ref = 1;
                (*fdl).own_pool = 0;
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_REQUIRED_FIELDS => {
                (*fdl).required_fields = cram_voption_va_arg_int(args) as c_uint;
                if (*fdl).range.refid != -2 {
                    (*fdl).required_fields |= crate::htslib_rs::cram::SAM_POS;
                }
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_STORE_MD => {
                (*fdl).store_md = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::CRAM_OPT_STORE_NM => {
                (*fdl).store_nm = cram_voption_va_arg_int(args);
            }
            x if x == crate::htslib_rs::cram::HTS_OPT_COMPRESSION_LEVEL => {
                (*fdl).level = cram_voption_va_arg_int(args);
            }
            x if x == HTS_OPT_PROFILE => {
                match cram_voption_va_arg_int(args) {
                    HTS_PROFILE_FAST => {
                        if (*fdl).level == CRAM_DEFAULT_LEVEL {
                            (*fdl).level = 1;
                        }
                        (*fdl).use_tok = 0;
                        (*fdl).seqs_per_slice = 10000;
                    }
                    HTS_PROFILE_NORMAL => {}
                    HTS_PROFILE_SMALL => {
                        if (*fdl).level == CRAM_DEFAULT_LEVEL {
                            (*fdl).level = 6;
                        }
                        (*fdl).use_bz2 = 1;
                        (*fdl).use_fqz = 1;
                        (*fdl).seqs_per_slice = 25000;
                    }
                    HTS_PROFILE_ARCHIVE => {
                        if (*fdl).level == CRAM_DEFAULT_LEVEL {
                            (*fdl).level = 7;
                        }
                        (*fdl).use_bz2 = 1;
                        (*fdl).use_fqz = 1;
                        (*fdl).use_arith = 1;
                        if (*fdl).level > 7 {
                            (*fdl).use_lzma = 1;
                        }
                        (*fdl).seqs_per_slice = 100000;
                    }
                    _ => {}
                }
                if (*fdl).bases_per_slice == CRAM_DEFAULT_BASES_PER_SLICE {
                    (*fdl).bases_per_slice = (*fdl).seqs_per_slice * 500;
                }
            }
            _ => {
                *__errno_location() = EINVAL;
                return -1;
            }
        }

        0
    }
}

pub unsafe fn cram_check_EOF(fd: *mut cram_fd) -> c_int {
    hts_sys::cram_check_EOF(fd.cast())
}

pub unsafe fn cram_copy_slice(in_: *mut cram_fd, out: *mut cram_fd, num_slice: i32) -> c_int {
    hts_sys::cram_copy_slice(in_.cast(), out.cast(), num_slice)
}

pub unsafe fn cram_transcode_rg(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    c: *mut cram_container,
    nrg: c_int,
    in_rg: *mut c_int,
    out_rg: *mut c_int,
) -> c_int {
    unsafe {
        cram_external::cram_cram_external_c_934_cram_transcode_rg(
            in_.cast(),
            out.cast(),
            c,
            nrg,
            in_rg,
            out_rg,
        )
    }
}

pub unsafe fn cram_get_refs(fd: *mut htsFile) -> *mut refs_t {
    hts_sys::cram_get_refs(fd.cast())
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

static mut M_CHANNEL: [*mut mFILE; 3] = [std::ptr::null_mut(); 3];
static mut DONE_STDIN: c_int = 0;

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
    content_type: hts_sys::cram_content_type,
    content_id: i32,
    comp_size: i32,
    uncomp_size: i32,
    crc32: u32,
    idx: i32,
    data: *mut u8,
    alloc: usize,
    byte: usize,
    bit: c_int,
    m: *mut hts_sys::cram_metrics,
    crc32_checked: c_int,
    crc_part: u32,
}

#[repr(C)]
struct cram_container_layout {
    length: i32,
    ref_seq_id: i32,
    ref_seq_start: i64,
    ref_seq_span: i64,
    record_counter: i64,
    num_bases: i64,
    num_records: i32,
    num_blocks: i32,
    num_landmarks: i32,
    landmark: *mut i32,
    offset: usize,
    comp_hdr: *mut cram_block_compression_hdr_layout,
    comp_hdr_block: *mut hts_sys::cram_block,
}

#[repr(C)]
struct cram_block_slice_hdr_layout {
    content_type: hts_sys::cram_content_type,
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

#[repr(C)]
struct cram_slice_layout {
    hdr: *mut cram_block_slice_hdr_layout,
    hdr_block: *mut hts_sys::cram_block,
    block: *mut *mut hts_sys::cram_block,
    block_by_id: *mut *mut hts_sys::cram_block,
}

const CRAM_DS_END: usize = 47;

#[repr(C)]
#[derive(Clone, Copy)]
struct cram_range_layout {
    refid: c_int,
    start: i64,
    end: i64,
}

#[repr(C)]
pub struct cram_file_def_layout {
    magic: [c_char; 4],
    major_version: u8,
    minor_version: u8,
    file_id: [c_char; 20],
}

#[repr(C)]
struct cram_fd_layout {
    fp: *mut hFILE,
    mode: c_int,
    version: c_int,
    file_def: *mut c_void,
    header: *mut hts_sys::sam_hdr_t,
    prefix: *mut c_char,
    record_counter: i64,
    err: c_int,
    ctr: *mut cram_container_layout,
    ctr_mt: *mut cram_container_layout,
    first_base: c_int,
    last_base: c_int,
    refs: *mut hts_sys::refs_t,
    ref_: *mut c_char,
    ref_free: *mut c_char,
    ref_id: c_int,
    ref_start: i64,
    ref_end: i64,
    ref_fn: *mut c_char,
    level: c_int,
    m: [*mut hts_sys::cram_metrics; CRAM_DS_END],
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
    first_container: libc::off_t,
    curr_position: libc::off_t,
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
    metrics_lock: libc::pthread_mutex_t,
    ref_lock: libc::pthread_mutex_t,
    range_lock: libc::pthread_mutex_t,
    bl: *mut c_void,
    bam_list_lock: libc::pthread_mutex_t,
    job_pending: *mut c_void,
    ooc: c_int,
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
struct kh_generic_layout {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut c_void,
    vals: *mut c_void,
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
    lock: libc::pthread_mutex_t,
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
    out: *mut hts_sys::cram_block,
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

pub unsafe fn cram_cram_stats_c_48_cram_stats_create() -> *mut c_void {
    calloc(1, std::mem::size_of::<cram_stats_layout>() as u64)
}

pub unsafe fn cram_cram_stats_c_52_cram_stats_add(st: *mut c_void, val: c_int) {
    let st = st.cast::<cram_stats_layout>();
    (*st).nsamp += 1;

    if val >= 0 && val < 1024 {
        (*st).freqs[val as usize] += 1;
        return;
    }

    if (*st).h.is_null() {
        let h = calloc(1, std::mem::size_of::<kh_m_i2i_layout>() as u64).cast::<kh_m_i2i_layout>();
        if h.is_null() {
            return;
        }
        let n_buckets = 4u32;
        let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
        (*h).flags = malloc(n_flags as u64 * std::mem::size_of::<u32>() as u64).cast::<u32>();
        (*h).keys = malloc(n_buckets as u64 * std::mem::size_of::<i64>() as u64).cast::<i64>();
        (*h).vals = malloc(n_buckets as u64 * std::mem::size_of::<c_int>() as u64).cast::<c_int>();
        if (*h).flags.is_null() || (*h).keys.is_null() || (*h).vals.is_null() {
            free((*h).flags.cast());
            free((*h).keys.cast());
            free((*h).vals.cast());
            free(h.cast());
            return;
        }
        for i in 0..n_flags {
            *(*h).flags.add(i as usize) = 0xaaaa_aaaa;
        }
        (*h).n_buckets = n_buckets;
        (*h).upper_bound = (n_buckets as f64 * 0.77) as u32;
        (*st).h = h.cast();
    }

    let mut h = (*st).h.cast::<kh_m_i2i_layout>();
    if (*h).n_buckets != 0 {
        let key = val as u64;
        let hash = ((key >> 33) ^ key ^ (key << 11)) as u32;
        let mask = (*h).n_buckets - 1;
        let mut k = hash & mask;
        let last = k;
        let mut step = 0u32;
        loop {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 2) != 0 {
                break;
            }
            if ((flag >> ((k & 0x0f) << 1)) & 1) == 0 && *(*h).keys.add(k as usize) == val as i64 {
                *(*h).vals.add(k as usize) += 1;
                return;
            }
            step += 1;
            k = (k + step) & mask;
            if k == last {
                break;
            }
        }
    }

    if (*h).n_occupied >= (*h).upper_bound {
        let old_h = h;
        let old_n = (*old_h).n_buckets;
        let new_n = if old_n == 0 { 4 } else { old_n << 1 };
        let new_flags_n = if new_n < 16 { 1 } else { new_n >> 4 };
        let new_h =
            calloc(1, std::mem::size_of::<kh_m_i2i_layout>() as u64).cast::<kh_m_i2i_layout>();
        if new_h.is_null() {
            return;
        }
        (*new_h).flags =
            malloc(new_flags_n as u64 * std::mem::size_of::<u32>() as u64).cast::<u32>();
        (*new_h).keys = malloc(new_n as u64 * std::mem::size_of::<i64>() as u64).cast::<i64>();
        (*new_h).vals = malloc(new_n as u64 * std::mem::size_of::<c_int>() as u64).cast::<c_int>();
        if (*new_h).flags.is_null() || (*new_h).keys.is_null() || (*new_h).vals.is_null() {
            free((*new_h).flags.cast());
            free((*new_h).keys.cast());
            free((*new_h).vals.cast());
            free(new_h.cast());
            return;
        }
        for i in 0..new_flags_n {
            *(*new_h).flags.add(i as usize) = 0xaaaa_aaaa;
        }
        (*new_h).n_buckets = new_n;
        (*new_h).upper_bound = (new_n as f64 * 0.77 + 0.5) as u32;

        for k in 0..old_n {
            let flag = *(*old_h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            let key = *(*old_h).keys.add(k as usize);
            let hash_key = key as u64;
            let hash = ((hash_key >> 33) ^ hash_key ^ (hash_key << 11)) as u32;
            let mask = new_n - 1;
            let mut i = hash & mask;
            let mut step = 0u32;
            loop {
                let new_flag = *(*new_h).flags.add((i >> 4) as usize);
                if ((new_flag >> ((i & 0x0f) << 1)) & 2) != 0 {
                    break;
                }
                step += 1;
                i = (i + step) & mask;
            }
            *(*new_h).keys.add(i as usize) = key;
            *(*new_h).vals.add(i as usize) = *(*old_h).vals.add(k as usize);
            *(*new_h).flags.add((i >> 4) as usize) &= !(3 << ((i & 0x0f) << 1));
            (*new_h).size += 1;
            (*new_h).n_occupied += 1;
        }
        free((*old_h).flags.cast());
        free((*old_h).keys.cast());
        free((*old_h).vals.cast());
        free(old_h.cast());
        (*st).h = new_h.cast();
        h = new_h;
    }

    let key = val as u64;
    let hash = ((key >> 33) ^ key ^ (key << 11)) as u32;
    let mask = (*h).n_buckets - 1;
    let mut x = (*h).n_buckets;
    let mut site = (*h).n_buckets;
    let mut i = hash & mask;
    let flag = *(*h).flags.add((i >> 4) as usize);
    if ((flag >> ((i & 0x0f) << 1)) & 2) != 0 {
        x = i;
    } else {
        let last = i;
        let mut step = 0u32;
        while {
            let flag = *(*h).flags.add((i >> 4) as usize);
            ((flag >> ((i & 0x0f) << 1)) & 2) == 0
                && (((flag >> ((i & 0x0f) << 1)) & 1) != 0
                    || *(*h).keys.add(i as usize) != val as i64)
        } {
            let flag = *(*h).flags.add((i >> 4) as usize);
            if ((flag >> ((i & 0x0f) << 1)) & 1) != 0 {
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
            if ((flag >> ((i & 0x0f) << 1)) & 2) != 0 && site != (*h).n_buckets {
                x = site;
            } else {
                x = i;
            }
        }
    }

    let flag = *(*h).flags.add((x >> 4) as usize);
    if ((flag >> ((x & 0x0f) << 1)) & 2) != 0 {
        *(*h).keys.add(x as usize) = val as i64;
        *(*h).vals.add(x as usize) = 1;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0x0f) << 1));
        (*h).size += 1;
        (*h).n_occupied += 1;
    } else if ((flag >> ((x & 0x0f) << 1)) & 1) != 0 {
        *(*h).keys.add(x as usize) = val as i64;
        *(*h).vals.add(x as usize) = 1;
        *(*h).flags.add((x >> 4) as usize) &= !(3 << ((x & 0x0f) << 1));
        (*h).size += 1;
    }
}

pub unsafe fn cram_cram_stats_c_80_cram_stats_del(st: *mut c_void, val: c_int) {
    let st = st.cast::<cram_stats_layout>();
    (*st).nsamp -= 1;

    if val >= 0 && val < 1024 {
        (*st).freqs[val as usize] -= 1;
        debug_assert!((*st).freqs[val as usize] >= 0);
        return;
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        if (*h).n_buckets != 0 {
            let key = val as u64;
            let hash = ((key >> 33) ^ key ^ (key << 11)) as u32;
            let mask = (*h).n_buckets - 1;
            let mut k = hash & mask;
            let last = k;
            let mut step = 0u32;
            loop {
                let flag = *(*h).flags.add((k >> 4) as usize);
                if ((flag >> ((k & 0x0f) << 1)) & 2) != 0 {
                    break;
                }
                if ((flag >> ((k & 0x0f) << 1)) & 1) == 0
                    && *(*h).keys.add(k as usize) == val as i64
                {
                    *(*h).vals.add(k as usize) -= 1;
                    if *(*h).vals.add(k as usize) == 0 {
                        *(*h).flags.add((k >> 4) as usize) |= 1 << ((k & 0x0f) << 1);
                        (*h).size -= 1;
                    }
                    return;
                }
                step += 1;
                k = (k + step) & mask;
                if k == last {
                    break;
                }
            }
        }
    }

    (*st).nsamp += 1;
}

pub unsafe fn cram_cram_stats_c_105_cram_stats_dump(st: *mut c_void) {
    let st = st.cast::<cram_stats_layout>();
    libc::fprintf(crate::htslib_rs::c_compat::stderr.cast(), c"cram_stats:\n".as_ptr());

    for i in 0..1024usize {
        let freq = (*st).freqs[i];
        if freq == 0 {
            continue;
        }
        libc::fprintf(
            crate::htslib_rs::c_compat::stderr.cast(),
            c"\t%d\t%d\n".as_ptr(),
            i as c_int,
            freq,
        );
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        for k in 0..(*h).n_buckets {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            libc::fprintf(
                crate::htslib_rs::c_compat::stderr.cast(),
                c"\t%lld\t%d\n".as_ptr(),
                *(*h).keys.add(k as usize) as libc::c_longlong,
                *(*h).vals.add(k as usize),
            );
        }
    }
}

pub unsafe fn cram_cram_stats_c_134_cram_stats_encoding(fd: *mut c_void, st: *mut c_void) -> c_int {
    let fd = fd.cast::<cram_fd_layout>();
    let st = st.cast::<cram_stats_layout>();
    let mut nvals = 0i32;
    let mut max_val = 0i32;
    let mut min_val = i32::MAX;
    let mut ntot = 0i32;

    for i in 0..1024usize {
        if (*st).freqs[i] == 0 {
            continue;
        }
        ntot += (*st).freqs[i];
        if max_val < i as i32 {
            max_val = i as i32;
        }
        if min_val > i as i32 {
            min_val = i as i32;
        }
        nvals += 1;
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        for k in 0..(*h).n_buckets {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            let i = *(*h).keys.add(k as usize) as i32;
            ntot += *(*h).vals.add(k as usize);
            if max_val < i {
                max_val = i;
            }
            if min_val > i {
                min_val = i;
            }
            nvals += 1;
        }
    }

    (*st).nvals = nvals;
    (*st).min_val = min_val as i64;
    (*st).max_val = max_val as i64;
    debug_assert_eq!(ntot, (*st).nsamp);

    if (*fd).version >> 8 >= 4 {
        if nvals == 1 {
            44
        } else if nvals == 0 || min_val < 0 {
            42
        } else {
            41
        }
    } else if nvals <= 1 {
        3
    } else {
        1
    }
}

pub unsafe fn cram_cram_stats_c_223_cram_stats_free(st: *mut c_void) {
    if st.is_null() {
        return;
    }
    let st_layout = st.cast::<cram_stats_layout>();
    if !(*st_layout).h.is_null() {
        let h = (*st_layout).h.cast::<kh_m_i2i_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }
    free(st);
}

#[repr(C)]
struct cram_block_compression_hdr_layout {
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
    td_blk: *mut hts_sys::cram_block,
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
type VarintPut32BlkFn = unsafe extern "C" fn(*mut hts_sys::cram_block, i32) -> c_int;
type VarintPut64BlkFn = unsafe extern "C" fn(*mut hts_sys::cram_block, i64) -> c_int;
type VarintSizeFn = unsafe extern "C" fn(i64) -> c_int;
type CramCodecDecodeFn = unsafe extern "C" fn(
    *mut hts_sys::cram_slice,
    *mut c_void,
    *mut hts_sys::cram_block,
    *mut c_char,
    *mut c_int,
) -> c_int;
type CramCodecEncodeFn =
    unsafe extern "C" fn(*mut hts_sys::cram_slice, *mut c_void, *mut c_char, c_int) -> c_int;
type CramCodecFlushFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type CramCodecStoreFn =
    unsafe extern "C" fn(*mut c_void, *mut hts_sys::cram_block, *mut c_char, c_int) -> c_int;
type CramCodecDescribeFn = unsafe extern "C" fn(*mut c_void, *mut kstring_t) -> c_int;
type CramCodecSizeFn = unsafe extern "C" fn(*mut hts_sys::cram_slice, *mut c_void) -> c_int;
type CramCodecGetBlockFn =
    unsafe extern "C" fn(*mut hts_sys::cram_slice, *mut c_void) -> *mut hts_sys::cram_block;
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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
    out: *mut hts_sys::cram_block,
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

pub unsafe fn cram_string_alloc_c_55_string_pool_create(
    mut max_length: usize,
) -> *mut cram_string_alloc_t {
    let a_str =
        malloc(std::mem::size_of::<cram_string_alloc_t>() as u64) as *mut cram_string_alloc_t;
    if a_str.is_null() {
        return std::ptr::null_mut();
    }

    if max_length < CRAM_STRING_ALLOC_MIN_STR_SIZE {
        max_length = CRAM_STRING_ALLOC_MIN_STR_SIZE;
    }

    (*a_str).nstrings = 0;
    (*a_str).max_strings = 0;
    (*a_str).max_length = max_length;
    (*a_str).strings = std::ptr::null_mut();

    a_str
}

pub unsafe fn cram_string_alloc_c_75_new_string_pool(
    a_str: *mut cram_string_alloc_t,
) -> *mut cram_string_alloc_string_t {
    if (*a_str).nstrings == (*a_str).max_strings {
        let new_max = ((*a_str).max_strings | ((*a_str).max_strings >> 2)) + 1;
        let str_ = realloc(
            (*a_str).strings.cast(),
            (new_max * std::mem::size_of::<cram_string_alloc_string_t>()) as u64,
        ) as *mut cram_string_alloc_string_t;

        if str_.is_null() {
            return std::ptr::null_mut();
        }

        (*a_str).strings = str_;
        (*a_str).max_strings = new_max;
    }

    let str_ = (*a_str).strings.add((*a_str).nstrings);
    (*str_).str_ = malloc((*a_str).max_length as u64).cast();

    if (*str_).str_.is_null() {
        return std::ptr::null_mut();
    }

    (*str_).used = 0;
    (*a_str).nstrings += 1;

    str_
}

pub unsafe fn cram_string_alloc_c_103_string_pool_destroy(a_str: *mut cram_string_alloc_t) {
    for i in 0..(*a_str).nstrings {
        free((*(*a_str).strings.add(i)).str_.cast());
    }

    free((*a_str).strings.cast());
    free(a_str.cast());
}

pub unsafe fn cram_string_alloc_c_117_string_alloc(
    a_str: *mut cram_string_alloc_t,
    length: usize,
) -> *mut c_char {
    if length == 0 {
        return std::ptr::null_mut();
    }

    if (*a_str).nstrings != 0 {
        let str_ = (*a_str).strings.add((*a_str).nstrings - 1);

        if (*str_).used + length < (*a_str).max_length {
            let ret = (*str_).str_.add((*str_).used);
            (*str_).used += length;
            return ret;
        }
    }

    if length > (*a_str).max_length {
        (*a_str).max_length = length;
    }

    let str_ = cram_string_alloc_c_75_new_string_pool(a_str);
    if str_.is_null() {
        return std::ptr::null_mut();
    }

    (*str_).used = length;
    (*str_).str_
}

pub unsafe fn cram_string_alloc_c_149_string_dup(
    a_str: *mut cram_string_alloc_t,
    instr: *const c_char,
) -> *mut c_char {
    cram_string_alloc_c_153_string_ndup(a_str, instr, libc::strlen(instr))
}

pub unsafe fn cram_string_alloc_c_153_string_ndup(
    a_str: *mut cram_string_alloc_t,
    instr: *const c_char,
    len: usize,
) -> *mut c_char {
    let str_ = cram_string_alloc_c_117_string_alloc(a_str, len + 1);
    if str_.is_null() {
        return std::ptr::null_mut();
    }

    memcpy(str_.cast(), instr.cast(), len as u64);
    *str_.add(len) = 0;

    str_
}

pub unsafe fn cram_cram_encode_c_70_sub_idx(key: *mut c_char, val: c_char) -> c_int {
    let mut i = 0;
    let mut keyp = key;
    while i < 4 {
        let c = *keyp;
        keyp = keyp.add(1);
        if c == val {
            break;
        }
        i += 1;
    }
    i
}

pub unsafe fn cram_cram_encode_c_1246_bam_data_end(b: *mut bam1_t) -> *const c_char {
    (*b).data.add((*b).l_data as usize).cast()
}

pub unsafe fn cram_cram_encode_c_1253_bam_aux2i_end(
    mut aux: *const u8,
    aux_end: *const u8,
) -> c_int {
    let type_ = *aux;
    aux = aux.add(1);
    match type_ {
        b'c' => {
            if aux_end.offset_from(aux) < 1 {
                *__errno_location() = EINVAL;
                return 0;
            }
            *(aux.cast::<i8>()) as c_int
        }
        b'C' => {
            if aux_end.offset_from(aux) < 1 {
                *__errno_location() = EINVAL;
                return 0;
            }
            *aux as c_int
        }
        b's' => {
            if aux_end.offset_from(aux) < 2 {
                *__errno_location() = EINVAL;
                return 0;
            }
            i16::from_le_bytes([*aux, *aux.add(1)]) as c_int
        }
        b'S' => {
            if aux_end.offset_from(aux) < 2 {
                *__errno_location() = EINVAL;
                return 0;
            }
            u16::from_le_bytes([*aux, *aux.add(1)]) as c_int
        }
        b'i' => {
            if aux_end.offset_from(aux) < 4 {
                *__errno_location() = EINVAL;
                return 0;
            }
            i32::from_le_bytes([*aux, *aux.add(1), *aux.add(2), *aux.add(3)]) as c_int
        }
        b'I' => {
            if aux_end.offset_from(aux) < 4 {
                *__errno_location() = EINVAL;
                return 0;
            }
            u32::from_le_bytes([*aux, *aux.add(1), *aux.add(2), *aux.add(3)]) as c_int
        }
        _ => {
            *__errno_location() = EINVAL;
            0
        }
    }
}

pub unsafe fn cram_cram_encode_c_1301_expected_template_count(b: *mut bam1_t) -> c_int {
    let mut expected = if ((*b).core.flag as c_int & BAM_FPAIRED) != 0 {
        2
    } else {
        1
    };

    let tc_tag = [b'T' as c_char, b'C' as c_char, 0];
    let tc = bam_aux_get(b, tc_tag.as_ptr());
    if !tc.is_null() {
        let n = cram_cram_encode_c_1253_bam_aux2i_end(
            tc,
            cram_cram_encode_c_1246_bam_data_end(b).cast(),
        );
        if expected < n {
            expected = n;
        }
    }

    let sa_tag = [b'S' as c_char, b'A' as c_char, 0];
    if tc.is_null() && !bam_aux_get(b, sa_tag.as_ptr()).is_null() {
        expected = c_int::MAX;
    }

    expected
}

pub unsafe fn cram_cram_encode_c_1476_next_cigar_op(
    cigar: *mut u32,
    ncigar: u32,
    skip: *mut c_int,
    spos: *mut c_int,
    cig_ind: *mut u32,
    cig_op: *mut u32,
    cig_len: *mut u32,
) -> c_int {
    loop {
        while *cig_len == 0 {
            if *cig_ind < ncigar {
                *cig_op = *cigar.add(*cig_ind as usize) & BAM_CIGAR_MASK;
                *cig_len = *cigar.add(*cig_ind as usize) >> BAM_CIGAR_SHIFT;
                *cig_ind += 1;
            } else {
                return -1;
            }
        }

        if *skip.add(*cig_op as usize) != 0 {
            *spos += (bam_cigar_type(*cig_op as c_int) & 1) * *cig_len as c_int;
            *cig_len = 0;
            continue;
        }

        *cig_len -= 1;
        break;
    }

    *cig_op as c_int
}

pub unsafe fn cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(
    hdr: *mut hts_sys::cram_block_slice_hdr,
) -> i32 {
    (*(hdr.cast::<cram_block_slice_hdr_layout>())).num_blocks
}

pub unsafe fn cram_cram_external_c_504_cram_slice_hdr_get_embed_ref_id(
    h: *mut hts_sys::cram_block_slice_hdr,
) -> c_int {
    (*(h.cast::<cram_block_slice_hdr_layout>())).ref_base_id
}

pub unsafe fn cram_cram_external_c_508_cram_slice_hdr_get_coords(
    h: *mut hts_sys::cram_block_slice_hdr,
    refid: *mut c_int,
    start: *mut crate::htslib_rs::hts::hts_pos_t,
    span: *mut crate::htslib_rs::hts::hts_pos_t,
) {
    let h = h.cast::<cram_block_slice_hdr_layout>();
    if !refid.is_null() {
        *refid = (*h).ref_seq_id;
    }
    if !start.is_null() {
        *start = (*h).ref_seq_start;
    }
    if !span.is_null() {
        *span = (*h).ref_seq_span;
    }
}

pub unsafe fn cram_cram_external_c_529_cram_block_get_size(b: *mut hts_sys::cram_block) -> i32 {
    (*(b.cast::<cram_block_layout>())).byte as i32
}

pub unsafe fn cram_cram_external_c_530_cram_block_get_method(
    b: *mut hts_sys::cram_block,
) -> hts_sys::cram_block_method {
    (*(b.cast::<cram_block_layout>())).orig_method
}

pub unsafe fn cram_cram_external_c_542_cram_block_set_size(b: *mut hts_sys::cram_block, size: i32) {
    (*(b.cast::<cram_block_layout>())).byte = size as usize;
}

pub unsafe fn cram_cram_io_h_183_cram_get_block_by_id(
    slice: *mut hts_sys::cram_slice,
    id: c_int,
) -> *mut hts_sys::cram_block {
    let slice = slice.cast::<cram_slice_layout>();
    let mut v = id as u32;
    if !(*slice).block_by_id.is_null() && v < 256 {
        return *(*slice).block_by_id.add(v as usize);
    }

    v = 256 + v % 251;
    if !(*slice).block_by_id.is_null() {
        let b = *(*slice).block_by_id.add(v as usize);
        if !b.is_null() && (*(b.cast::<cram_block_layout>())).content_id == id {
            return b;
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
            return b;
        }
    }

    std::ptr::null_mut()
}

pub unsafe fn cram_cram_io_h_216_block_resize_exact(
    b: *mut hts_sys::cram_block,
    len: usize,
) -> c_int {
    let b = b.cast::<cram_block_layout>();
    let tmp = realloc((*b).data.cast(), len as u64).cast::<u8>();
    if tmp.is_null() {
        return -1;
    }
    (*b).alloc = len;
    (*b).data = tmp;
    0
}

pub unsafe fn cram_cram_io_h_226_block_resize(b: *mut hts_sys::cram_block, len: usize) -> c_int {
    let block = b.cast::<cram_block_layout>();
    if (*block).alloc > len {
        return 0;
    }

    let mut alloc = (*block).alloc + 800;
    alloc = std::cmp::max(alloc + (alloc >> 2), len);
    cram_cram_io_h_216_block_resize_exact(b, alloc)
}

pub unsafe fn cram_cram_io_h_243_block_grow(b: *mut hts_sys::cram_block, len: usize) -> c_int {
    let block = b.cast::<cram_block_layout>();
    cram_cram_io_h_226_block_resize(b, (*block).byte + len)
}

pub unsafe fn cram_cram_io_h_248_block_append(
    b: *mut hts_sys::cram_block,
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

pub unsafe fn cram_cram_io_h_261_block_append_char(
    b: *mut hts_sys::cram_block,
    c: c_char,
) -> c_int {
    if cram_cram_io_h_243_block_grow(b, 1) < 0 {
        return -1;
    }

    let block = b.cast::<cram_block_layout>();
    *(*block).data.add((*block).byte) = c as u8;
    (*block).byte += 1;
    0
}

pub unsafe fn cram_cram_io_h_271_block_append_uint(
    b: *mut hts_sys::cram_block,
    i: libc::c_uint,
) -> c_int {
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

pub unsafe fn cram_cram_io_h_646_cram_hfile(fd: *mut hts_sys::cram_fd) -> *mut hts_sys::hFILE {
    (*(fd.cast::<cram_fd_layout>())).fp.cast()
}

pub unsafe fn cram_cram_io_c_5662_cram_eof(fd: *mut cram_fd) -> c_int {
    (*fd.cast::<cram_fd_layout>()).eof
}

pub unsafe fn cram_cram_codecs_c_73_get_bit_MSB(block: *mut hts_sys::cram_block) -> c_int {
    let block = block.cast::<cram_block_layout>();
    if (*block).byte > (*block).alloc {
        return -1;
    }

    let val = *(*block).data.add((*block).byte) >> (*block).bit;
    (*block).bit -= 1;
    if (*block).bit == -1 {
        (*block).bit = 7;
        (*block).byte += 1;
    }

    (val & 1) as c_int
}

pub unsafe fn cram_cram_codecs_c_95_get_one_bits_MSB(block: *mut hts_sys::cram_block) -> c_int {
    let block = block.cast::<cram_block_layout>();
    let mut n = 0;
    if (*block).byte >= (*block).uncomp_size as usize {
        return -1;
    }

    loop {
        let b = *(*block).data.add((*block).byte) >> (*block).bit;
        (*block).bit -= 1;
        if (*block).bit == -1 {
            (*block).bit = 7;
            (*block).byte += 1;
            if (*block).byte == (*block).uncomp_size as usize && (b & 1) != 0 {
                return -1;
            }
        }
        n += 1;
        if (b & 1) == 0 {
            break;
        }
    }

    n - 1
}

pub unsafe fn cram_cram_codecs_c_113_get_zero_bits_MSB(block: *mut hts_sys::cram_block) -> c_int {
    let block = block.cast::<cram_block_layout>();
    let mut n = 0;
    if (*block).byte >= (*block).uncomp_size as usize {
        return -1;
    }

    loop {
        let b = *(*block).data.add((*block).byte) >> (*block).bit;
        (*block).bit -= 1;
        if (*block).bit == -1 {
            (*block).bit = 7;
            (*block).byte += 1;
            if (*block).byte == (*block).uncomp_size as usize && (b & 1) == 0 {
                return -1;
            }
        }
        n += 1;
        if (b & 1) != 0 {
            break;
        }
    }

    n - 1
}

pub unsafe fn cram_cram_codecs_c_133_store_bit_MSB(
    block: *mut hts_sys::cram_block,
    bit: libc::c_uint,
) {
    let block = block.cast::<cram_block_layout>();
    if (*block).byte >= (*block).alloc {
        (*block).alloc = if (*block).alloc != 0 {
            (*block).alloc * 2
        } else {
            1024
        };
        (*block).data = realloc((*block).data.cast(), (*block).alloc as u64).cast::<u8>();
    }

    if bit != 0 {
        *(*block).data.add((*block).byte) |= 1 << (*block).bit;
    }

    (*block).bit -= 1;
    if (*block).bit == -1 {
        (*block).bit = 7;
        (*block).byte += 1;
        *(*block).data.add((*block).byte) = 0;
    }
}

pub unsafe fn cram_cram_codecs_c_152_store_bytes_MSB(
    block: *mut hts_sys::cram_block,
    bytes: *mut c_char,
    len: c_int,
) {
    let block = block.cast::<cram_block_layout>();
    if (*block).bit != 7 {
        (*block).bit = 7;
        (*block).byte += 1;
    }

    while (*block).byte + len as usize >= (*block).alloc {
        (*block).alloc = if (*block).alloc != 0 {
            (*block).alloc * 2
        } else {
            1024
        };
        (*block).data = realloc((*block).data.cast(), (*block).alloc as u64).cast::<u8>();
    }

    memcpy(
        (*block).data.add((*block).byte).cast(),
        bytes.cast(),
        len as u64,
    );
    (*block).byte += len as usize;
}

pub unsafe fn cram_cram_codecs_c_169_get_bits_MSB(
    block: *mut hts_sys::cram_block,
    mut nbits: c_int,
) -> i64 {
    let block = block.cast::<cram_block_layout>();
    let mut val = 0u64;

    if nbits <= (*block).bit + 1 {
        val = ((*(*block).data.add((*block).byte) >> ((*block).bit - (nbits - 1))) as u16
            & ((1u16 << nbits) - 1)) as u64;
        (*block).bit -= nbits;
        if (*block).bit == -1 {
            (*block).bit = 7;
            (*block).byte += 1;
        }
        return val as i64;
    }

    while nbits > 0 {
        val <<= 1;
        val |= ((*(*block).data.add((*block).byte) >> (*block).bit) & 1) as u64;
        (*block).bit -= 1;
        if (*block).bit < 0 {
            (*block).byte += 1;
            (*block).bit &= 7;
        }
        nbits -= 1;
    }

    val as i64
}

pub unsafe fn cram_cram_codecs_c_259_store_bits_MSB(
    block: *mut hts_sys::cram_block,
    val: u64,
    mut nbits: c_int,
) -> c_int {
    let block = block.cast::<cram_block_layout>();
    if (*block).byte + 8 >= (*block).alloc {
        if (*block).byte != 0 {
            (*block).alloc *= 2;
            (*block).data = realloc((*block).data.cast(), ((*block).alloc + 8) as u64).cast::<u8>();
            if (*block).data.is_null() {
                return -1;
            }
        } else {
            (*block).alloc = 1024;
            (*block).data = realloc((*block).data.cast(), ((*block).alloc + 8) as u64).cast::<u8>();
            if (*block).data.is_null() {
                return -1;
            }
            *(*block).data = 0;
        }
    }

    if nbits <= (*block).bit + 1 {
        *(*block).data.add((*block).byte) |= (val << ((*block).bit + 1 - nbits)) as u8;
        (*block).bit -= nbits;
        if (*block).bit == -1 {
            (*block).bit = 7;
            (*block).byte += 1;
            *(*block).data.add((*block).byte) = 0;
        }
        return 0;
    }

    nbits -= (*block).bit + 1;
    *(*block).data.add((*block).byte) |= (val >> nbits) as u8;
    (*block).bit = 7;
    (*block).byte += 1;
    *(*block).data.add((*block).byte) = 0;

    let mut mask = 1u32 << (nbits - 1);
    loop {
        if (val & mask as u64) != 0 {
            *(*block).data.add((*block).byte) |= 1 << (*block).bit;
        }
        (*block).bit -= 1;
        if (*block).bit == -1 {
            (*block).bit = 7;
            (*block).byte += 1;
            *(*block).data.add((*block).byte) = 0;
        }
        mask >>= 1;
        nbits -= 1;
        if nbits == 0 {
            break;
        }
    }

    0
}

pub unsafe fn cram_cram_codecs_c_319_cram_extract_block(
    b: *mut hts_sys::cram_block,
    size: c_int,
) -> *mut c_char {
    let b = b.cast::<cram_block_layout>();
    let cp = (*b).data.add((*b).idx as usize).cast::<c_char>();
    (*b).idx += size;
    if (*b).idx > (*b).uncomp_size {
        return std::ptr::null_mut();
    }

    cp
}

pub unsafe fn cram_cram_codecs_c_350_cram_external_decode_int(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get32.unwrap())(&mut cp, endp, &mut err);
    *(out.cast::<i32>()) = val as i32;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;

    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_370_cram_external_decode_long(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get64.unwrap())(&mut cp, endp, &mut err);
    *(out.cast::<i64>()) = val;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;

    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_390_cram_external_decode_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let cp = cram_cram_codecs_c_319_cram_extract_block(b, *out_size);
    if cp.is_null() {
        return -1;
    }

    if !out.is_null() {
        memcpy(out.cast(), cp.cast(), *out_size as u64);
    }
    0
}

pub unsafe fn cram_cram_codecs_c_410_cram_external_decode_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out_: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }

    let cp = cram_cram_codecs_c_319_cram_extract_block(b, *out_size);
    if cp.is_null() {
        return -1;
    }

    cram_cram_io_h_248_block_append(out_.cast(), cp.cast(), *out_size as usize)
}

pub unsafe fn cram_cram_codecs_c_433_cram_external_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_439_cram_external_decode_size(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id);
    if b.is_null() {
        return -1;
    }

    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_450_cram_external_get_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> *mut hts_sys::cram_block {
    let c = c.cast::<cram_codec_external_layout>();
    cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).external.content_id)
}

pub unsafe fn cram_cram_codecs_c_454_cram_external_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    if kputsn(c"EXTERNAL(id=".as_ptr(), 12, ks) < 0
        || kputw((*c).external.content_id, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_459_cram_external_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if size < 1 {
        return std::ptr::null_mut();
    }

    let c = malloc(std::mem::size_of::<cram_codec_external_layout>() as u64)
        .cast::<cram_codec_external_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }

    (*c).codec = 1;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    if (version >> 8) >= 4 {
        if codec != 1 {
            free(c.cast());
            return std::ptr::null_mut();
        }
        (*c).decode = if option == 5 {
            cram_cram_codecs_c_410_cram_external_decode_block as usize as *mut c_void
        } else if option == 3 || option == 4 {
            cram_cram_codecs_c_390_cram_external_decode_char as usize as *mut c_void
        } else {
            free(c.cast());
            return std::ptr::null_mut();
        };
    } else if option == 1 {
        (*c).decode = cram_cram_codecs_c_350_cram_external_decode_int as usize as *mut c_void;
    } else if option == 2 {
        (*c).decode = cram_cram_codecs_c_370_cram_external_decode_long as usize as *mut c_void;
    } else if option == 4 || option == 3 {
        (*c).decode = cram_cram_codecs_c_390_cram_external_decode_char as usize as *mut c_void;
    } else {
        (*c).decode = cram_cram_codecs_c_410_cram_external_decode_block as usize as *mut c_void;
    }
    (*c).free = cram_cram_codecs_c_433_cram_external_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = cram_cram_codecs_c_439_cram_external_decode_size as usize as *mut c_void;
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = cram_cram_codecs_c_450_cram_external_get_block as usize as *mut c_void;
    (*c).describe = cram_cram_codecs_c_454_cram_external_describe as usize as *mut c_void;

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).external.content_id =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp.offset_from(data) != size as isize {
        free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).external.type_ = option;
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_523_cram_external_encode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i32_ = *(in_.cast::<u32>()) as i32;
    if ((*(*c).vv).varint_put32_blk.unwrap())((*c).out, i32_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_529_cram_external_encode_sint(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i32_ = *(in_.cast::<i32>());
    if ((*(*c).vv).varint_put32s_blk.unwrap())((*c).out, i32_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_535_cram_external_encode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i64_ = *(in_.cast::<u64>()) as i64;
    if ((*(*c).vv).varint_put64_blk.unwrap())((*c).out, i64_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_541_cram_external_encode_slong(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let i64_ = *(in_.cast::<i64>());
    if ((*(*c).vv).varint_put64s_blk.unwrap())((*c).out, i64_) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_547_cram_external_encode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    cram_cram_io_h_248_block_append((*c).out, in_.cast(), in_size as usize)
}

pub unsafe fn cram_cram_codecs_c_556_cram_external_encode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_562_cram_external_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_external_layout>();
    let mut tmp = [0 as c_char; 99];
    let mut tp = tmp.as_mut_ptr();
    let tpend = tmp.as_mut_ptr().add(99);
    let mut len = 0;
    let mut r = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    tp = tp.add(((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*c).external.content_id) as usize);
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, tp.offset_from(tmp.as_ptr()) as i32);
    len += n;
    r |= n;
    let nbytes = tp.offset_from(tmp.as_ptr()) as usize;
    if cram_cram_io_h_248_block_append(b, tmp.as_ptr().cast(), nbytes) != 0 {
        return -1;
    }
    len += nbytes as c_int;

    if r > 0 {
        len
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_586_cram_external_encode_init(
    _st: *mut c_void,
    codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_external_layout>() as u64)
        .cast::<cram_codec_external_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 1;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_556_cram_external_encode_free as usize as *mut c_void;
    if (version >> 8) >= 4 {
        if codec != 1 || (option != 3 && option != 4) {
            free(c.cast());
            return std::ptr::null_mut();
        }
        (*c).encode = cram_cram_codecs_c_547_cram_external_encode_char as usize as *mut c_void;
    } else if option == 1 {
        (*c).encode = cram_cram_codecs_c_523_cram_external_encode_int as usize as *mut c_void;
    } else if option == 2 {
        (*c).encode = cram_cram_codecs_c_535_cram_external_encode_long as usize as *mut c_void;
    } else if option == 4 || option == 3 {
        (*c).encode = cram_cram_codecs_c_547_cram_external_encode_char as usize as *mut c_void;
    } else {
        libc::abort();
    }
    (*c).decode = std::ptr::null_mut();
    (*c).store = cram_cram_codecs_c_562_cram_external_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();
    (*c).external.content_id = dat as usize as i32;
    (*c).external.type_ = option;
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_644_cram_varint_decode_int(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get32.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i32>()) = val as i32;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_666_cram_varint_decode_sint(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get32s.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i32>()) = val as i32;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_688_cram_varint_decode_long(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get64.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i64>()) = val;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_710_cram_varint_decode_slong(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let block = b.cast::<cram_block_layout>();
    let mut cp = (*block).data.add((*block).idx as usize).cast::<c_char>();
    let endp = (*block)
        .data
        .add((*block).uncomp_size as usize)
        .cast::<c_char>();
    let mut err = 0;
    let val = ((*(*c).vv).varint_get64s.unwrap())(&mut cp, endp, &mut err) + (*c).varint.offset;
    *(out.cast::<i64>()) = val;
    (*block).idx = cp.offset_from((*block).data.cast::<c_char>()) as i32;
    *out_size = 1;
    if err != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_732_cram_varint_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_737_cram_varint_decode_size(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id);
    if b.is_null() {
        return -1;
    }
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_748_cram_varint_get_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> *mut hts_sys::cram_block {
    let c = c.cast::<cram_codec_varint_layout>();
    cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).varint.content_id)
}

pub unsafe fn cram_cram_codecs_c_752_cram_varint_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    if kputsn(c"VARINT(id=".as_ptr(), 10, ks) < 0
        || kputw((*c).varint.content_id, ks) < 0
        || kputsn(c",offset=".as_ptr(), 8, ks) < 0
        || kputll((*c).varint.offset, ks) < 0
        || kputsn(c",type=".as_ptr(), 6, ks) < 0
        || kputw((*c).varint.type_, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_760_cram_varint_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_varint_layout>() as u64)
        .cast::<cram_codec_varint_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = codec;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).decode = match codec {
        41 => {
            if option == 1 || option == 6 {
                cram_cram_codecs_c_644_cram_varint_decode_int as usize as *mut c_void
            } else if option == 2 || option == 7 {
                cram_cram_codecs_c_688_cram_varint_decode_long as usize as *mut c_void
            } else {
                free(c.cast());
                return std::ptr::null_mut();
            }
        }
        42 => {
            if option == 1 || option == 6 {
                cram_cram_codecs_c_666_cram_varint_decode_sint as usize as *mut c_void
            } else if option == 2 || option == 7 {
                cram_cram_codecs_c_710_cram_varint_decode_slong as usize as *mut c_void
            } else {
                free(c.cast());
                return std::ptr::null_mut();
            }
        }
        _ => {
            free(c.cast());
            return std::ptr::null_mut();
        }
    };
    (*c).free = cram_cram_codecs_c_732_cram_varint_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = cram_cram_codecs_c_737_cram_varint_decode_size as usize as *mut c_void;
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = cram_cram_codecs_c_748_cram_varint_get_block as usize as *mut c_void;
    (*c).describe = cram_cram_codecs_c_752_cram_varint_describe as usize as *mut c_void;

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).varint.content_id =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    (*c).varint.offset =
        ((*vv).varint_get64s.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut());
    if cp.offset_from(data) != size as isize {
        free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).varint.type_ = option;
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_820_cram_varint_encode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<u32>()) as i64 - (*c).varint.offset;
    if ((*(*c).vv).varint_put32_blk.unwrap())((*c).out, val as i32) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_827_cram_varint_encode_sint(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<i32>()) as i64 - (*c).varint.offset;
    if ((*(*c).vv).varint_put32s_blk.unwrap())((*c).out, val as i32) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_834_cram_varint_encode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<u64>()) as i64 - (*c).varint.offset;
    if ((*(*c).vv).varint_put64_blk.unwrap())((*c).out, val) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_841_cram_varint_encode_slong(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    _in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let val = *(in_.cast::<i64>()) - (*c).varint.offset;
    if ((*(*c).vv).varint_put64s_blk.unwrap())((*c).out, val) >= 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_848_cram_varint_encode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_854_cram_varint_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_varint_layout>();
    let mut tmp = [0 as c_char; 99];
    let mut tp = tmp.as_mut_ptr();
    let mut len = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    tp = tp.add(
        ((*(*c).vv).varint_put32.unwrap())(tp, std::ptr::null_mut(), (*c).varint.content_id)
            as usize,
    );
    tp = tp.add(
        ((*(*c).vv).varint_put64s.unwrap())(tp, std::ptr::null_mut(), (*c).varint.offset) as usize,
    );
    len += ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += ((*(*c).vv).varint_put32_blk.unwrap())(b, tp.offset_from(tmp.as_ptr()) as i32);
    let nbytes = tp.offset_from(tmp.as_ptr()) as usize;
    if cram_cram_io_h_248_block_append(b, tmp.as_ptr().cast(), nbytes) != 0 {
        return -1;
    }
    len + nbytes as c_int
}

pub unsafe fn cram_cram_codecs_c_878_cram_varint_encode_init(
    st: *mut c_void,
    mut codec: c_int,
    option: c_int,
    dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_varint_layout>() as u64)
        .cast::<cram_codec_varint_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }

    (*c).varint.offset = 0;
    if !st.is_null() {
        let st = st.cast::<cram_stats_layout>();
        if (*st).min_val < 0 && (*st).min_val >= -127 && (*st).max_val / -(*st).min_val > 100 {
            (*c).varint.offset = -(*st).min_val;
            codec = 41;
        } else if (*st).min_val > 0 {
            (*c).varint.offset = -(*st).min_val;
        }
    }

    (*c).codec = codec;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_848_cram_varint_encode_free as usize as *mut c_void;
    (*c).encode = match codec {
        41 => {
            if option == 1 {
                cram_cram_codecs_c_820_cram_varint_encode_int as usize as *mut c_void
            } else {
                cram_cram_codecs_c_834_cram_varint_encode_long as usize as *mut c_void
            }
        }
        42 => {
            if option == 1 {
                cram_cram_codecs_c_827_cram_varint_encode_sint as usize as *mut c_void
            } else {
                cram_cram_codecs_c_841_cram_varint_encode_slong as usize as *mut c_void
            }
        }
        _ => {
            free(c.cast());
            return std::ptr::null_mut();
        }
    };
    (*c).decode = std::ptr::null_mut();
    (*c).store = cram_cram_codecs_c_854_cram_varint_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();
    (*c).varint.content_id = dat as usize as i32;
    (*c).varint.type_ = option;
    c.cast()
}

pub unsafe fn cram_cram_io_c_1388_cram_new_block(
    content_type: hts_sys::cram_content_type,
    content_id: c_int,
) -> *mut hts_sys::cram_block {
    let b = malloc(std::mem::size_of::<cram_block_layout>() as u64).cast::<cram_block_layout>();
    if b.is_null() {
        return std::ptr::null_mut();
    }
    (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
    (*b).orig_method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
    (*b).content_type = content_type;
    (*b).content_id = content_id;
    (*b).comp_size = 0;
    (*b).uncomp_size = 0;
    (*b).data = std::ptr::null_mut();
    (*b).alloc = 0;
    (*b).byte = 0;
    (*b).bit = 7;
    (*b).crc32 = 0;
    (*b).idx = 0;
    (*b).m = std::ptr::null_mut();
    (*b).crc32_checked = 0;
    (*b).crc_part = 0;
    b.cast()
}

pub unsafe fn cram_cram_io_c_277_itf8_put(cp: *mut c_char, val: i32) -> c_int {
    let up = cp.cast::<u8>();
    let v = val as u32;
    if (v & !0x0000_007f) == 0 {
        *up = v as u8;
        1
    } else if (v & !0x0000_3fff) == 0 {
        *up = ((v >> 8) | 0x80) as u8;
        *up.add(1) = (v & 0xff) as u8;
        2
    } else if (v & !0x001f_ffff) == 0 {
        *up = ((v >> 16) | 0xc0) as u8;
        *up.add(1) = ((v >> 8) & 0xff) as u8;
        *up.add(2) = (v & 0xff) as u8;
        3
    } else if (v & !0x0fff_ffff) == 0 {
        *up = ((v >> 24) | 0xe0) as u8;
        *up.add(1) = ((v >> 16) & 0xff) as u8;
        *up.add(2) = ((v >> 8) & 0xff) as u8;
        *up.add(3) = (v & 0xff) as u8;
        4
    } else {
        *up = (0xf0 | ((v >> 28) & 0xff)) as u8;
        *up.add(1) = ((v >> 20) & 0xff) as u8;
        *up.add(2) = ((v >> 12) & 0xff) as u8;
        *up.add(3) = ((v >> 4) & 0xff) as u8;
        *up.add(4) = (v & 0x0f) as u8;
        5
    }
}

pub unsafe fn cram_cram_io_c_309_ltf8_put(cp: *mut c_char, val: i64) -> c_int {
    let up = cp.cast::<u8>();
    let v = val as u64;
    if (v & !((1u64 << 7) - 1)) == 0 {
        *up = v as u8;
        1
    } else if (v & !((1u64 << (6 + 8)) - 1)) == 0 {
        *up = ((v >> 8) | 0x80) as u8;
        *up.add(1) = (v & 0xff) as u8;
        2
    } else if (v & !((1u64 << (5 + 2 * 8)) - 1)) == 0 {
        *up = ((v >> 16) | 0xc0) as u8;
        *up.add(1) = ((v >> 8) & 0xff) as u8;
        *up.add(2) = (v & 0xff) as u8;
        3
    } else if (v & !((1u64 << (4 + 3 * 8)) - 1)) == 0 {
        *up = ((v >> 24) | 0xe0) as u8;
        *up.add(1) = ((v >> 16) & 0xff) as u8;
        *up.add(2) = ((v >> 8) & 0xff) as u8;
        *up.add(3) = (v & 0xff) as u8;
        4
    } else if (v & !((1u64 << (3 + 4 * 8)) - 1)) == 0 {
        *up = ((v >> 32) | 0xf0) as u8;
        *up.add(1) = ((v >> 24) & 0xff) as u8;
        *up.add(2) = ((v >> 16) & 0xff) as u8;
        *up.add(3) = ((v >> 8) & 0xff) as u8;
        *up.add(4) = (v & 0xff) as u8;
        5
    } else if (v & !((1u64 << (2 + 5 * 8)) - 1)) == 0 {
        *up = ((v >> 40) | 0xf8) as u8;
        *up.add(1) = ((v >> 32) & 0xff) as u8;
        *up.add(2) = ((v >> 24) & 0xff) as u8;
        *up.add(3) = ((v >> 16) & 0xff) as u8;
        *up.add(4) = ((v >> 8) & 0xff) as u8;
        *up.add(5) = (v & 0xff) as u8;
        6
    } else if (v & !((1u64 << (1 + 6 * 8)) - 1)) == 0 {
        *up = ((v >> 48) | 0xfc) as u8;
        *up.add(1) = ((v >> 40) & 0xff) as u8;
        *up.add(2) = ((v >> 32) & 0xff) as u8;
        *up.add(3) = ((v >> 24) & 0xff) as u8;
        *up.add(4) = ((v >> 16) & 0xff) as u8;
        *up.add(5) = ((v >> 8) & 0xff) as u8;
        *up.add(6) = (v & 0xff) as u8;
        7
    } else if (v & !((1u64 << (7 * 8)) - 1)) == 0 {
        *up = ((v >> 56) | 0xfe) as u8;
        *up.add(1) = ((v >> 48) & 0xff) as u8;
        *up.add(2) = ((v >> 40) & 0xff) as u8;
        *up.add(3) = ((v >> 32) & 0xff) as u8;
        *up.add(4) = ((v >> 24) & 0xff) as u8;
        *up.add(5) = ((v >> 16) & 0xff) as u8;
        *up.add(6) = ((v >> 8) & 0xff) as u8;
        *up.add(7) = (v & 0xff) as u8;
        8
    } else {
        *up = 0xff;
        *up.add(1) = ((v >> 56) & 0xff) as u8;
        *up.add(2) = ((v >> 48) & 0xff) as u8;
        *up.add(3) = ((v >> 40) & 0xff) as u8;
        *up.add(4) = ((v >> 32) & 0xff) as u8;
        *up.add(5) = ((v >> 24) & 0xff) as u8;
        *up.add(6) = ((v >> 16) & 0xff) as u8;
        *up.add(7) = ((v >> 8) & 0xff) as u8;
        *up.add(8) = (v & 0xff) as u8;
        9
    }
}

pub unsafe fn cram_cram_io_c_138_itf8_decode(fd: *mut cram_fd, val_p: *mut i32) -> c_int {
    let nbytes = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 4];
    let nbits = [
        0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x3f, 0x3f, 0x3f, 0x3f, 0x1f, 0x1f, 0x0f,
        0x0f,
    ];
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut val = htslib_hfile_h_163_hgetc(fp);
    if val == -1 {
        return -1;
    }

    let i = nbytes[(val >> 4) as usize];
    val &= nbits[(val >> 4) as usize];

    match i {
        0 => {
            *val_p = val;
            1
        }
        1 => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            *val_p = val;
            2
        }
        2 => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            *val_p = val;
            3
        }
        3 => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            *val_p = val;
            4
        }
        _ => {
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 8) | (htslib_hfile_h_163_hgetc(fp) as u8 as c_int);
            val = (val << 4) | ((htslib_hfile_h_163_hgetc(fp) as u8 as c_int) & 0x0f);
            *val_p = val;
            5
        }
    }
}

pub unsafe fn cram_cram_io_c_196_itf8_decode_crc(
    fd: *mut cram_fd,
    val_p: *mut i32,
    crc: *mut u32,
) -> c_int {
    let nbytes = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 3, 4];
    let nbits = [
        0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x3f, 0x3f, 0x3f, 0x3f, 0x1f, 0x1f, 0x0f,
        0x0f,
    ];
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut c = [0u8; 5];

    let mut val = htslib_hfile_h_163_hgetc(fp);
    if val == -1 {
        return -1;
    }
    c[0] = val as u8;

    let i = nbytes[(val >> 4) as usize];
    val &= nbits[(val >> 4) as usize];

    if i > 0 && htslib_hfile_h_247_hread(fp, c.as_mut_ptr().add(1).cast(), i as usize) < i as isize
    {
        return -1;
    }

    match i {
        0 => {
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 1);
            1
        }
        1 => {
            val = (val << 8) | c[1] as c_int;
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 2);
            2
        }
        2 => {
            val = (val << 8) | c[1] as c_int;
            val = (val << 8) | c[2] as c_int;
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 3);
            3
        }
        3 => {
            val = (val << 8) | c[1] as c_int;
            val = (val << 8) | c[2] as c_int;
            val = (val << 8) | c[3] as c_int;
            *val_p = val;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 4);
            4
        }
        _ => {
            let mut uv = val as u32;
            uv = (uv << 8) | c[1] as u32;
            uv = (uv << 8) | c[2] as u32;
            uv = (uv << 8) | c[3] as u32;
            uv = (uv << 4) | (c[4] as u32 & 0x0f);
            *val_p = uv as i32;
            *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 5);
            5
        }
    }
}

pub unsafe fn cram_cram_io_c_382_itf8_encode(fd: *mut cram_fd, val: i32) -> c_int {
    let mut buf = [0 as c_char; 5];
    let len = cram_cram_io_c_277_itf8_put(buf.as_mut_ptr(), val);
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    if htslib_hfile_h_292_hwrite(fp, buf.as_ptr().cast(), len as usize) == len as libc::ssize_t {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_io_c_420_ltf8_decode(fd: *mut cram_fd, val_p: *mut i64) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let c = htslib_hfile_h_163_hgetc(fp);
    if c == -1 {
        return -1;
    }

    let mut val = c as u8 as u64;
    if val < 0x80 {
        *val_p = val as i64;
        1
    } else if val < 0xc0 {
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        *val_p = (val & ((1u64 << (6 + 8)) - 1)) as i64;
        2
    } else if val < 0xe0 {
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        *val_p = (val & ((1u64 << (5 + 2 * 8)) - 1)) as i64;
        3
    } else if val < 0xf0 {
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        *val_p = (val & ((1u64 << (4 + 3 * 8)) - 1)) as i64;
        4
    } else if val < 0xf8 {
        for _ in 0..4 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (3 + 4 * 8)) - 1)) as i64;
        5
    } else if val < 0xfc {
        for _ in 0..5 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (2 + 5 * 8)) - 1)) as i64;
        6
    } else if val < 0xfe {
        for _ in 0..6 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (1 + 6 * 8)) - 1)) as i64;
        7
    } else if val < 0xff {
        for _ in 0..7 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = (val & ((1u64 << (7 * 8)) - 1)) as i64;
        8
    } else {
        for _ in 0..8 {
            val = (val << 8) | htslib_hfile_h_163_hgetc(fp) as u8 as u64;
        }
        *val_p = val as i64;
        9
    }
}

pub unsafe fn cram_cram_io_c_1068_zlib_mem_inflate(
    cdata: *mut c_char,
    csize: usize,
    size: *mut usize,
) -> *mut c_char {
    cram_cram_io_c_1157_zlib_mem_inflate(cdata, csize, size)
}

pub unsafe fn cram_cram_io_c_1157_zlib_mem_inflate(
    cdata: *mut c_char,
    csize: usize,
    size: *mut usize,
) -> *mut c_char {
    let input = std::slice::from_raw_parts(cdata.cast::<u8>(), csize);
    let mut decoder = flate2::read::GzDecoder::new(input);
    let mut out = Vec::with_capacity((csize as f64 * 1.2) as usize + 100);
    if decoder.read_to_end(&mut out).is_err() {
        return std::ptr::null_mut();
    }

    let alloc_len = out.len().max(1);
    let data = malloc(alloc_len as u64).cast::<c_char>();
    if data.is_null() {
        return std::ptr::null_mut();
    }
    if !out.is_empty() {
        memcpy(data.cast(), out.as_ptr().cast(), out.len() as u64);
    }
    *size = out.len();
    data
}

pub unsafe fn cram_cram_io_c_5127_cram_init_varint(vv: *mut c_void, version: c_int) {
    let vv = vv.cast::<varint_vec_layout>();
    if version >= 4 {
        (*vv).varint_get32 = Some(cram_cram_io_c_772_uint7_get_32);
        (*vv).varint_get32s = Some(cram_cram_io_c_780_sint7_get_32);
        (*vv).varint_get64 = Some(cram_cram_io_c_788_uint7_get_64);
        (*vv).varint_get64s = Some(cram_cram_io_c_796_sint7_get_64);
        (*vv).varint_put32 = Some(cram_cram_io_c_804_uint7_put_32);
        (*vv).varint_put32s = Some(cram_cram_io_c_808_sint7_put_32);
        (*vv).varint_put64 = Some(cram_cram_io_c_812_uint7_put_64);
        (*vv).varint_put64s = Some(cram_cram_io_c_816_sint7_put_64);
        (*vv).varint_put32_blk = Some(cram_cram_io_c_821_uint7_put_blk_32);
        (*vv).varint_put32s_blk = Some(cram_cram_io_c_831_sint7_put_blk_32);
        (*vv).varint_put64_blk = Some(cram_cram_io_c_841_uint7_put_blk_64);
        (*vv).varint_put64s_blk = Some(cram_cram_io_c_851_sint7_put_blk_64);
        (*vv).varint_size = Some(cram_cram_io_c_768_uint7_size);
        (*vv).varint_decode32_crc = cram_cram_io_c_862_uint7_decode_crc32 as usize as *mut c_void;
        (*vv).varint_decode32s_crc = cram_cram_io_c_907_sint7_decode_crc32 as usize as *mut c_void;
        (*vv).varint_decode64_crc = cram_cram_io_c_953_uint7_decode_crc64 as usize as *mut c_void;
    } else {
        (*vv).varint_get32 = Some(cram_cram_io_c_644_safe_itf8_get);
        (*vv).varint_get32s = Some(cram_cram_io_c_644_safe_itf8_get);
        (*vv).varint_get64 = Some(cram_cram_io_c_673_safe_ltf8_get);
        (*vv).varint_get64s = Some(cram_cram_io_c_673_safe_ltf8_get);
        (*vv).varint_put32 = Some(cram_cram_io_c_747_safe_itf8_put);
        (*vv).varint_put32s = Some(cram_cram_io_c_747_safe_itf8_put);
        (*vv).varint_put64 = Some(cram_cram_io_c_751_safe_ltf8_put);
        (*vv).varint_put64s = Some(cram_cram_io_c_751_safe_ltf8_put);
        (*vv).varint_put32_blk = Some(cram_cram_io_c_620_itf8_put_blk);
        (*vv).varint_put32s_blk = Some(cram_cram_io_c_620_itf8_put_blk);
        (*vv).varint_put64_blk = Some(cram_cram_io_c_632_ltf8_put_blk);
        (*vv).varint_put64s_blk = Some(cram_cram_io_c_632_ltf8_put_blk);
        (*vv).varint_size = Some(cram_cram_io_c_755_itf8_size);
        (*vv).varint_decode32_crc = cram_cram_io_c_196_itf8_decode_crc as usize as *mut c_void;
        (*vv).varint_decode32s_crc = cram_cram_io_c_196_itf8_decode_crc as usize as *mut c_void;
        (*vv).varint_decode64_crc = cram_cram_io_c_501_ltf8_decode_crc as usize as *mut c_void;
    }
}

pub unsafe fn cram_cram_io_c_5170_cram_init_tables(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();

    (*fd).l1 = [4; 256];
    (*fd).l1[b'A' as usize] = 0;
    (*fd).l1[b'a' as usize] = 0;
    (*fd).l1[b'C' as usize] = 1;
    (*fd).l1[b'c' as usize] = 1;
    (*fd).l1[b'G' as usize] = 2;
    (*fd).l1[b'g' as usize] = 2;
    (*fd).l1[b'T' as usize] = 3;
    (*fd).l1[b't' as usize] = 3;

    (*fd).l2 = [5; 256];
    (*fd).l2[b'A' as usize] = 0;
    (*fd).l2[b'a' as usize] = 0;
    (*fd).l2[b'C' as usize] = 1;
    (*fd).l2[b'c' as usize] = 1;
    (*fd).l2[b'G' as usize] = 2;
    (*fd).l2[b'g' as usize] = 2;
    (*fd).l2[b'T' as usize] = 3;
    (*fd).l2[b't' as usize] = 3;
    (*fd).l2[b'N' as usize] = 4;
    (*fd).l2[b'n' as usize] = 4;

    if ((*fd).version >> 8) == 1 {
        for i in 0..0x200usize {
            let mut f = 0;
            let i_c = i as c_int;
            if (i_c & CRAM_FPAIRED) != 0 {
                f |= BAM_FPAIRED;
            }
            if (i_c & CRAM_FPROPER_PAIR) != 0 {
                f |= BAM_FPROPER_PAIR;
            }
            if (i_c & CRAM_FUNMAP) != 0 {
                f |= BAM_FUNMAP;
            }
            if (i_c & CRAM_FREVERSE) != 0 {
                f |= BAM_FREVERSE;
            }
            if (i_c & CRAM_FREAD1) != 0 {
                f |= BAM_FREAD1;
            }
            if (i_c & CRAM_FREAD2) != 0 {
                f |= BAM_FREAD2;
            }
            if (i_c & CRAM_FSECONDARY) != 0 {
                f |= BAM_FSECONDARY;
            }
            if (i_c & CRAM_FQCFAIL) != 0 {
                f |= BAM_FQCFAIL;
            }
            if (i_c & CRAM_FDUP) != 0 {
                f |= BAM_FDUP;
            }
            (*fd).bam_flag_swap[i] = f as c_uint;
        }

        for i in 0..0x1000usize {
            let mut g = 0;
            let i_c = i as c_int;
            if (i_c & BAM_FPAIRED) != 0 {
                g |= CRAM_FPAIRED;
            }
            if (i_c & BAM_FPROPER_PAIR) != 0 {
                g |= CRAM_FPROPER_PAIR;
            }
            if (i_c & BAM_FUNMAP) != 0 {
                g |= CRAM_FUNMAP;
            }
            if (i_c & BAM_FREVERSE) != 0 {
                g |= CRAM_FREVERSE;
            }
            if (i_c & BAM_FREAD1) != 0 {
                g |= CRAM_FREAD1;
            }
            if (i_c & BAM_FREAD2) != 0 {
                g |= CRAM_FREAD2;
            }
            if (i_c & BAM_FSECONDARY) != 0 {
                g |= CRAM_FSECONDARY;
            }
            if (i_c & BAM_FQCFAIL) != 0 {
                g |= CRAM_FQCFAIL;
            }
            if (i_c & BAM_FDUP) != 0 {
                g |= CRAM_FDUP;
            }
            (*fd).cram_flag_swap[i] = g as c_uint;
        }
    } else {
        for i in 0..0x1000usize {
            (*fd).bam_flag_swap[i] = i as c_uint;
        }
        for i in 0..0x1000usize {
            (*fd).cram_flag_swap[i] = i as c_uint;
        }
    }

    (*fd).cram_sub_matrix = [[4; 32]; 32];
    for i in 0..32usize {
        (*fd).cram_sub_matrix[i][(b'A' & 0x1f) as usize] = 0;
        (*fd).cram_sub_matrix[i][(b'C' & 0x1f) as usize] = 1;
        (*fd).cram_sub_matrix[i][(b'G' & 0x1f) as usize] = 2;
        (*fd).cram_sub_matrix[i][(b'T' & 0x1f) as usize] = 3;
        (*fd).cram_sub_matrix[i][(b'N' & 0x1f) as usize] = 4;
    }
    for i in (0..20usize).step_by(4) {
        for j in 0..20usize {
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
            (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize][j] = 3;
        }
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i] & 0x1f) as usize] = 0;
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i + 1] & 0x1f) as usize] = 1;
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i + 2] & 0x1f) as usize] = 2;
        (*fd).cram_sub_matrix[(b"ACGTN"[i >> 2] & 0x1f) as usize]
            [(CRAM_SUBST_MATRIX[i + 3] & 0x1f) as usize] = 3;
    }

    cram_cram_io_c_5127_cram_init_varint(
        (&mut (*fd).vv as *mut varint_vec_layout).cast(),
        (*fd).version >> 8,
    );
}

pub unsafe fn cram_cram_io_c_4236_reset_metrics(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();

    if !(*fd).pool.is_null() {
        for i in 0..CRAM_DS_END {
            let m = (*fd).m[i].cast::<cram_metrics_layout>();
            if m.is_null() {
                continue;
            }
            (*m).next_trial = 999;
        }

        libc::pthread_mutex_unlock(&mut (*fd).metrics_lock);
        hts_tpool_process_flush((*fd).rqueue.cast::<hts_tpool_process>());
        libc::pthread_mutex_lock(&mut (*fd).metrics_lock);
    }

    for i in 0..CRAM_DS_END {
        let m = (*fd).m[i].cast::<cram_metrics_layout>();
        if m.is_null() {
            continue;
        }

        (*m).trial = NTRIALS;
        (*m).next_trial = TRIAL_SPAN;
        (*m).revised_method = 0;
        (*m).unpackable = 0;
        (*m).sz = [0; 32];
    }
}

pub unsafe fn cram_cram_io_c_501_ltf8_decode_crc(
    fd: *mut cram_fd,
    val_p: *mut i64,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let mut c = [0u8; 9];

    let mut val = htslib_hfile_h_163_hgetc(fp);
    if val < 0 {
        return -1;
    }
    c[0] = val as u8;

    if val < 0x80 {
        *val_p = val as i64;
        *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 1);
        return 1;
    } else if val < 0xc0 {
        let v = htslib_hfile_h_163_hgetc(fp);
        if v < 0 {
            return -1;
        }
        c[1] = v as u8;
        val = (val << 8) | c[1] as c_int;
        *val_p = (val & ((1 << (6 + 8)) - 1)) as i64;
        *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), 2);
        return 2;
    }

    let nread = if val < 0xe0 {
        2
    } else if val < 0xf0 {
        3
    } else if val < 0xf8 {
        4
    } else if val < 0xfc {
        5
    } else if val < 0xfe {
        6
    } else if val < 0xff {
        7
    } else {
        8
    };
    if htslib_hfile_h_247_hread(fp, c.as_mut_ptr().add(1).cast(), nread) < nread as isize {
        return -1;
    }

    let len = nread + 1;
    if c[0] < 0xff {
        let mut uval = c[0] as u64;
        for j in 1..len {
            uval = (uval << 8) | c[j] as u64;
        }
        let bits = match len {
            3 => 5 + 2 * 8,
            4 => 4 + 3 * 8,
            5 => 3 + 4 * 8,
            6 => 2 + 5 * 8,
            7 => 1 + 6 * 8,
            8 => 7 * 8,
            _ => unreachable!(),
        };
        *val_p = (uval & ((1u64 << bits) - 1)) as i64;
    } else {
        let mut uval = c[1] as u64;
        for j in 2..9 {
            uval = (uval << 8) | c[j] as u64;
        }
        *val_p = if c[1] < 0x80 {
            uval as i64
        } else {
            -((0xffff_ffff_ffff_ffffu64 - uval) as i64) - 1
        };
    }
    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, c.as_ptr().cast(), len);
    len as c_int
}

pub unsafe extern "C" fn cram_cram_io_c_620_itf8_put_blk(
    blk: *mut hts_sys::cram_block,
    val: i32,
) -> c_int {
    let mut buf = [0u8; 5];
    let sz = cram_cram_io_c_277_itf8_put(buf.as_mut_ptr().cast(), val);
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}

pub unsafe extern "C" fn cram_cram_io_c_632_ltf8_put_blk(
    blk: *mut hts_sys::cram_block,
    val: i64,
) -> c_int {
    let mut buf = [0u8; 9];
    let sz = cram_cram_io_c_309_ltf8_put(buf.as_mut_ptr().cast(), val);
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}

pub unsafe extern "C" fn cram_cram_io_c_644_safe_itf8_get(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let up = (*cp).cast::<u8>();
    if !endp.is_null() {
        let remaining = endp.offset_from(*cp);
        let needed = if remaining <= 0 {
            1
        } else {
            match *up >> 4 {
                0..=7 => 1,
                8..=11 => 2,
                12..=13 => 3,
                14 => 4,
                _ => 5,
            }
        };
        if remaining < 5 && (remaining <= 0 || remaining < needed) {
            if !err.is_null() {
                *err = 1;
            }
            return 0;
        }
    }

    if *up < 0x80 {
        *cp = (*cp).add(1);
        *up as i64
    } else if *up < 0xc0 {
        *cp = (*cp).add(2);
        ((((*up as u32) << 8) | *up.add(1) as u32) & 0x3fff) as i32 as i64
    } else if *up < 0xe0 {
        *cp = (*cp).add(3);
        ((((*up as u32) << 16) | ((*up.add(1) as u32) << 8) | *up.add(2) as u32) & 0x1f_ffff) as i32
            as i64
    } else if *up < 0xf0 {
        *cp = (*cp).add(4);
        ((((*up as u32) << 24)
            | ((*up.add(1) as u32) << 16)
            | ((*up.add(2) as u32) << 8)
            | *up.add(3) as u32)
            & 0x0fff_ffff) as i32 as i64
    } else {
        *cp = (*cp).add(5);
        (((((*up as u32) & 0x0f) << 28)
            | ((*up.add(1) as u32) << 20)
            | ((*up.add(2) as u32) << 12)
            | ((*up.add(3) as u32) << 4)
            | ((*up.add(4) as u32) & 0x0f)) as i32) as i64
    }
}

pub unsafe extern "C" fn cram_cram_io_c_673_safe_ltf8_get(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let up = (*cp).cast::<u8>();
    if !endp.is_null() {
        let remaining = endp.offset_from(*cp);
        let needed = if remaining <= 0 {
            1
        } else if *up < 0x80 {
            1
        } else if *up < 0xc0 {
            2
        } else if *up < 0xe0 {
            3
        } else if *up < 0xf0 {
            4
        } else if *up < 0xf8 {
            5
        } else if *up < 0xfc {
            6
        } else if *up < 0xfe {
            7
        } else if *up < 0xff {
            8
        } else {
            9
        };
        if remaining < 9 && (remaining <= 0 || remaining < needed) {
            if !err.is_null() {
                *err = 1;
            }
            return 0;
        }
    }

    if *up < 0x80 {
        *cp = (*cp).add(1);
        *up as i64
    } else if *up < 0xc0 {
        *cp = (*cp).add(2);
        ((((*up as u64) << 8) | *up.add(1) as u64) & ((1u64 << (6 + 8)) - 1)) as i64
    } else if *up < 0xe0 {
        *cp = (*cp).add(3);
        ((((*up as u64) << 16) | ((*up.add(1) as u64) << 8) | *up.add(2) as u64)
            & ((1u64 << (5 + 2 * 8)) - 1)) as i64
    } else if *up < 0xf0 {
        *cp = (*cp).add(4);
        ((((*up as u64) << 24)
            | ((*up.add(1) as u64) << 16)
            | ((*up.add(2) as u64) << 8)
            | *up.add(3) as u64)
            & ((1u64 << (4 + 3 * 8)) - 1)) as i64
    } else if *up < 0xf8 {
        *cp = (*cp).add(5);
        ((((*up as u64) << 32)
            | ((*up.add(1) as u64) << 24)
            | ((*up.add(2) as u64) << 16)
            | ((*up.add(3) as u64) << 8)
            | *up.add(4) as u64)
            & ((1u64 << (3 + 4 * 8)) - 1)) as i64
    } else if *up < 0xfc {
        *cp = (*cp).add(6);
        ((((*up as u64) << 40)
            | ((*up.add(1) as u64) << 32)
            | ((*up.add(2) as u64) << 24)
            | ((*up.add(3) as u64) << 16)
            | ((*up.add(4) as u64) << 8)
            | *up.add(5) as u64)
            & ((1u64 << (2 + 5 * 8)) - 1)) as i64
    } else if *up < 0xfe {
        *cp = (*cp).add(7);
        ((((*up as u64) << 48)
            | ((*up.add(1) as u64) << 40)
            | ((*up.add(2) as u64) << 32)
            | ((*up.add(3) as u64) << 24)
            | ((*up.add(4) as u64) << 16)
            | ((*up.add(5) as u64) << 8)
            | *up.add(6) as u64)
            & ((1u64 << (1 + 6 * 8)) - 1)) as i64
    } else if *up < 0xff {
        *cp = (*cp).add(8);
        ((((*up.add(1) as u64) << 48)
            | ((*up.add(2) as u64) << 40)
            | ((*up.add(3) as u64) << 32)
            | ((*up.add(4) as u64) << 24)
            | ((*up.add(5) as u64) << 16)
            | ((*up.add(6) as u64) << 8)
            | *up.add(7) as u64)
            & ((1u64 << (7 * 8)) - 1)) as i64
    } else {
        *cp = (*cp).add(9);
        (((*up.add(1) as u64) << 56)
            | ((*up.add(2) as u64) << 48)
            | ((*up.add(3) as u64) << 40)
            | ((*up.add(4) as u64) << 32)
            | ((*up.add(5) as u64) << 24)
            | ((*up.add(6) as u64) << 16)
            | ((*up.add(7) as u64) << 8)
            | *up.add(8) as u64) as i64
    }
}

pub unsafe extern "C" fn cram_cram_io_c_747_safe_itf8_put(
    cp: *mut c_char,
    _cp_end: *mut c_char,
    val: i32,
) -> c_int {
    cram_cram_io_c_277_itf8_put(cp, val)
}

pub unsafe extern "C" fn cram_cram_io_c_751_safe_ltf8_put(
    cp: *mut c_char,
    _cp_end: *mut c_char,
    val: i64,
) -> c_int {
    cram_cram_io_c_309_ltf8_put(cp, val)
}

pub extern "C" fn cram_cram_io_c_755_itf8_size(v: i64) -> c_int {
    if (v & !0x7f) == 0 {
        1
    } else if (v & !0x3fff) == 0 {
        2
    } else if (v & !0x1f_ffff) == 0 {
        3
    } else if (v & !0x0fff_ffff) == 0 {
        4
    } else {
        5
    }
}

pub extern "C" fn cram_cram_io_c_768_uint7_size(v: i64) -> c_int {
    let mut v = v as u64;
    let mut n = 1;
    while v >= 0x80 {
        n += 1;
        v >>= 7;
    }
    n
}

pub unsafe extern "C" fn cram_cram_io_c_772_uint7_get_32(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let mut p = (*cp).cast::<u8>();
    let end = endp.cast::<u8>();
    let mut val = 0u32;
    let mut nb = 0usize;
    let limit = if end.is_null() || end.offset_from(p) >= 6 {
        6
    } else if p >= end as *mut u8 {
        if !err.is_null() {
            *err = 1;
        }
        return 0;
    } else {
        end.offset_from(p) as usize
    };

    while nb < limit {
        let c = *p;
        p = p.add(1);
        nb += 1;
        val = (val << 7) | (c & 0x7f) as u32;
        if (c & 0x80) == 0 {
            break;
        }
    }

    *cp = p.cast();
    val as i64
}

pub unsafe extern "C" fn cram_cram_io_c_780_sint7_get_32(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let u = cram_cram_io_c_772_uint7_get_32(cp, endp, err) as u32;
    ((u >> 1) as i32 ^ -((u & 1) as i32)) as i64
}

pub unsafe extern "C" fn cram_cram_io_c_788_uint7_get_64(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let mut p = (*cp).cast::<u8>();
    let end = endp.cast::<u8>();
    let mut val = 0u64;
    let mut nb = 0usize;
    let limit = if end.is_null() || end.offset_from(p) >= 11 {
        11
    } else if p >= end as *mut u8 {
        if !err.is_null() {
            *err = 1;
        }
        return 0;
    } else {
        end.offset_from(p) as usize
    };

    while nb < limit {
        let c = *p;
        p = p.add(1);
        nb += 1;
        val = (val << 7) | (c & 0x7f) as u64;
        if (c & 0x80) == 0 {
            break;
        }
    }

    *cp = p.cast();
    val as i64
}

pub unsafe extern "C" fn cram_cram_io_c_796_sint7_get_64(
    cp: *mut *mut c_char,
    endp: *const c_char,
    err: *mut c_int,
) -> i64 {
    let u = cram_cram_io_c_788_uint7_get_64(cp, endp, err) as u64;
    ((u >> 1) as i64) ^ -((u & 1) as i64)
}

pub unsafe extern "C" fn cram_cram_io_c_804_uint7_put_32(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i32,
) -> c_int {
    cram_cram_io_c_812_uint7_put_64(cp, endp, val as u32 as i64)
}

pub unsafe extern "C" fn cram_cram_io_c_808_sint7_put_32(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i32,
) -> c_int {
    cram_cram_io_c_804_uint7_put_32(cp, endp, ((val as u32) << 1 ^ (val >> 31) as u32) as i32)
}

pub unsafe extern "C" fn cram_cram_io_c_812_uint7_put_64(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i64,
) -> c_int {
    let mut p = cp.cast::<u8>();
    let end = endp.cast::<u8>();
    let v = val as u64;
    let n = cram_cram_io_c_768_uint7_size(val);

    if !end.is_null() && end.offset_from(p) < n as isize {
        return 0;
    }

    for i in (0..n).rev() {
        let mut c = ((v >> (i * 7)) & 0x7f) as u8;
        if i != 0 {
            c |= 0x80;
        }
        *p = c;
        p = p.add(1);
    }

    n
}

pub unsafe extern "C" fn cram_cram_io_c_816_sint7_put_64(
    cp: *mut c_char,
    endp: *mut c_char,
    val: i64,
) -> c_int {
    cram_cram_io_c_812_uint7_put_64(cp, endp, ((val as u64) << 1 ^ (val >> 63) as u64) as i64)
}

pub unsafe extern "C" fn cram_cram_io_c_821_uint7_put_blk_32(
    blk: *mut hts_sys::cram_block,
    v: i32,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_804_uint7_put_32(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}

pub unsafe extern "C" fn cram_cram_io_c_831_sint7_put_blk_32(
    blk: *mut hts_sys::cram_block,
    v: i32,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_808_sint7_put_32(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}

pub unsafe extern "C" fn cram_cram_io_c_841_uint7_put_blk_64(
    blk: *mut hts_sys::cram_block,
    v: i64,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_812_uint7_put_64(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}

pub unsafe extern "C" fn cram_cram_io_c_851_sint7_put_blk_64(
    blk: *mut hts_sys::cram_block,
    v: i64,
) -> c_int {
    let mut buf = [0u8; 10];
    let sz = cram_cram_io_c_816_sint7_put_64(
        buf.as_mut_ptr().cast(),
        buf.as_mut_ptr().add(10).cast(),
        v,
    );
    if cram_cram_io_h_248_block_append(blk, buf.as_ptr().cast(), sz as usize) != 0 {
        return -1;
    }
    sz
}

pub unsafe fn cram_cram_io_c_862_uint7_decode_crc32(
    fd: *mut hts_sys::cram_fd,
    val_p: *mut i32,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let layout = fp.cast::<hfile_layout>();
    let mut b = [0u8; 5];
    let mut i = 0usize;
    let mut v = 0u32;

    loop {
        let c = if (*layout).end > (*layout).begin {
            let c = *(*layout).begin as u8;
            (*layout).begin = (*layout).begin.add(1);
            c as c_int
        } else {
            crate::htslib_rs::hfile::hgetc2(fp.cast())
        };
        if c < 0 {
            return -1;
        }
        b[i] = c as u8;
        i += 1;
        v = (v << 7) | (c as u32 & 0x7f);
        if i >= 5 || (c & 0x80) == 0 {
            break;
        }
    }

    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, b.as_ptr().cast(), i);
    *val_p = v as i32;
    i as c_int
}

pub unsafe fn cram_cram_io_c_907_sint7_decode_crc32(
    fd: *mut hts_sys::cram_fd,
    val_p: *mut i32,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let layout = fp.cast::<hfile_layout>();
    let mut b = [0u8; 5];
    let mut i = 0usize;
    let mut v = 0u32;

    loop {
        let c = if (*layout).end > (*layout).begin {
            let c = *(*layout).begin as u8;
            (*layout).begin = (*layout).begin.add(1);
            c as c_int
        } else {
            crate::htslib_rs::hfile::hgetc2(fp.cast())
        };
        if c < 0 {
            return -1;
        }
        b[i] = c as u8;
        i += 1;
        v = (v << 7) | (c as u32 & 0x7f);
        if i >= 5 || (c & 0x80) == 0 {
            break;
        }
    }

    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, b.as_ptr().cast(), i);
    *val_p = ((v >> 1) as i32) ^ -((v & 1) as i32);
    i as c_int
}

pub unsafe fn cram_cram_io_c_953_uint7_decode_crc64(
    fd: *mut hts_sys::cram_fd,
    val_p: *mut i64,
    crc: *mut u32,
) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let layout = fp.cast::<hfile_layout>();
    let mut b = [0u8; 10];
    let mut i = 0usize;
    let mut v = 0u64;

    loop {
        let c = if (*layout).end > (*layout).begin {
            let c = *(*layout).begin as u8;
            (*layout).begin = (*layout).begin.add(1);
            c as c_int
        } else {
            crate::htslib_rs::hfile::hgetc2(fp.cast())
        };
        if c < 0 {
            return -1;
        }
        b[i] = c as u8;
        i += 1;
        v = (v << 7) | (c as u64 & 0x7f);
        if i >= 5 || (c & 0x80) == 0 {
            break;
        }
    }

    *crc = crate::htslib_rs::bgzf::hts_crc32(*crc, b.as_ptr().cast(), i);
    *val_p = v as i64;
    i as c_int
}

pub unsafe fn cram_cram_io_c_1005_int32_decode(fd: *mut hts_sys::cram_fd, val: *mut i32) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let layout = fp.cast::<hfile_layout>();
    let mut i = 0i32;
    let buffer = (&mut i as *mut i32).cast::<c_void>();
    let nbytes = std::mem::size_of::<i32>();
    let mut n = (*layout).end.offset_from((*layout).begin) as usize;
    if n > nbytes {
        n = nbytes;
    }
    memcpy(buffer, (*layout).begin.cast(), n as u64);
    (*layout).begin = (*layout).begin.add(n);
    let got = if n == nbytes || ((*layout).flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        crate::htslib_rs::hfile::hread2(fp.cast(), buffer, nbytes, n)
    };
    if got != nbytes as libc::ssize_t {
        return -1;
    }

    *val = i32::from_le(i);
    4
}

pub unsafe fn cram_cram_io_c_1020_int32_encode(fd: *mut hts_sys::cram_fd, val: i32) -> c_int {
    let fp = (*fd.cast::<cram_fd_layout>()).fp;
    let layout = fp.cast::<hfile_layout>();
    let v = val.to_le();
    let buffer = (&v as *const i32).cast::<c_void>();
    let nbytes = std::mem::size_of::<i32>();

    if ((*layout).flags & HFILE_MOBILE) == 0 {
        let n = (*layout).limit.offset_from((*layout).begin) as usize;
        if n < nbytes {
            crate::htslib_rs::hfile::hfile_set_blksize(
                fp.cast(),
                (*layout).limit.offset_from((*layout).buffer) as usize + nbytes,
            );
            (*layout).end = (*layout).limit;
        }
    }

    let mut n = (*layout).limit.offset_from((*layout).begin) as usize;
    let wrote = if nbytes >= n && (*layout).begin == (*layout).buffer {
        crate::htslib_rs::hfile::hwrite2(fp.cast(), buffer, nbytes, 0)
    } else {
        if n > nbytes {
            n = nbytes;
        }
        memcpy((*layout).begin.cast(), buffer, n as u64);
        (*layout).begin = (*layout).begin.add(n);
        if n == nbytes {
            n as libc::ssize_t
        } else {
            crate::htslib_rs::hfile::hwrite2(fp.cast(), buffer, nbytes, n)
        }
    };

    if wrote != nbytes as libc::ssize_t {
        return -1;
    }
    4
}

pub unsafe fn cram_cram_io_c_1029_int32_get_blk(
    b: *mut hts_sys::cram_block,
    val: *mut i32,
) -> c_int {
    let block = b.cast::<cram_block_layout>();
    if (*block).uncomp_size < 0 || ((*block).uncomp_size as usize).saturating_sub((*block).byte) < 4
    {
        return -1;
    }

    let data = (*block).data.add((*block).byte);
    let v = (*data as u32)
        | ((*data.add(1) as u32) << 8)
        | ((*data.add(2) as u32) << 16)
        | ((*data.add(3) as u32) << 24);
    *val = v as i32;
    (*block).byte += 4;
    4
}

pub unsafe fn cram_cram_io_c_1045_int32_put_blk(b: *mut hts_sys::cram_block, val: i32) -> c_int {
    let v = val as u32;
    let cp = [
        (v & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        ((v >> 16) & 0xff) as u8,
        ((v >> 24) & 0xff) as u8,
    ];
    if cram_cram_io_h_248_block_append(b, cp.as_ptr().cast(), 4) != 0 {
        return -1;
    }
    0
}

pub unsafe fn cram_cram_io_c_1414_cram_read_block(
    fd: *mut hts_sys::cram_fd,
) -> *mut hts_sys::cram_block {
    let fd_layout = fd.cast::<cram_fd_layout>();
    let fp = (*fd_layout).fp;
    let hfile = fp.cast::<hfile_layout>();
    let b = malloc(std::mem::size_of::<cram_block_layout>() as u64).cast::<cram_block_layout>();
    if b.is_null() {
        return std::ptr::null_mut();
    }

    let c = if (*hfile).end > (*hfile).begin {
        let c = *(*hfile).begin as u8;
        (*hfile).begin = (*hfile).begin.add(1);
        c as c_int
    } else {
        crate::htslib_rs::hfile::hgetc2(fp.cast())
    };
    if c == -1 {
        free(b.cast());
        return std::ptr::null_mut();
    }
    (*b).method = c;
    if (*b).method > 8 {
        free(b.cast());
        return std::ptr::null_mut();
    }
    let c_byte = c as u8;
    let mut crc = crate::htslib_rs::bgzf::hts_crc32(0, (&c_byte as *const u8).cast(), 1);

    let c = if (*hfile).end > (*hfile).begin {
        let c = *(*hfile).begin as u8;
        (*hfile).begin = (*hfile).begin.add(1);
        c as c_int
    } else {
        crate::htslib_rs::hfile::hgetc2(fp.cast())
    };
    if c == -1 {
        free(b.cast());
        return std::ptr::null_mut();
    }
    (*b).content_type = c;
    let c_byte = c as u8;
    crc = crate::htslib_rs::bgzf::hts_crc32(crc, (&c_byte as *const u8).cast(), 1);

    if ((*fd_layout).version >> 8) >= 4 {
        if cram_cram_io_c_862_uint7_decode_crc32(fd, &mut (*b).content_id, &mut crc) == -1
            || cram_cram_io_c_862_uint7_decode_crc32(fd, &mut (*b).comp_size, &mut crc) == -1
            || cram_cram_io_c_862_uint7_decode_crc32(fd, &mut (*b).uncomp_size, &mut crc) == -1
        {
            free(b.cast());
            return std::ptr::null_mut();
        }
    } else {
        for out in [
            &mut (*b).content_id,
            &mut (*b).comp_size,
            &mut (*b).uncomp_size,
        ] {
            let mut buf = [0u8; 5];
            let mut n = 0usize;
            let mut want = 0usize;
            loop {
                let c = if (*hfile).end > (*hfile).begin {
                    let c = *(*hfile).begin as u8;
                    (*hfile).begin = (*hfile).begin.add(1);
                    c as c_int
                } else {
                    crate::htslib_rs::hfile::hgetc2(fp.cast())
                };
                if c < 0 {
                    free(b.cast());
                    return std::ptr::null_mut();
                }
                buf[n] = c as u8;
                n += 1;
                if n == 1 {
                    want = match buf[0] {
                        0x00..=0x7f => 1,
                        0x80..=0xbf => 2,
                        0xc0..=0xdf => 3,
                        0xe0..=0xef => 4,
                        _ => 5,
                    };
                }
                if n == want {
                    break;
                }
            }
            let mut cp = buf.as_mut_ptr().cast::<c_char>();
            *out = cram_cram_io_c_644_safe_itf8_get(
                &mut cp,
                buf.as_ptr().add(n).cast::<c_char>(),
                std::ptr::null_mut(),
            ) as i32;
            crc = crate::htslib_rs::bgzf::hts_crc32(crc, buf.as_ptr().cast(), n);
        }
    }

    let data_len = if (*b).method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
        if (*b).uncomp_size < 0 || (*b).comp_size != (*b).uncomp_size {
            free(b.cast());
            return std::ptr::null_mut();
        }
        (*b).uncomp_size as usize
    } else {
        if (*b).comp_size < 0 || (*b).uncomp_size < 0 {
            free(b.cast());
            return std::ptr::null_mut();
        }
        (*b).comp_size as usize
    };

    (*b).alloc = data_len;
    (*b).data = if data_len == 0 {
        std::ptr::null_mut()
    } else {
        malloc(data_len as u64).cast::<u8>()
    };
    if data_len != 0 && (*b).data.is_null() {
        free(b.cast());
        return std::ptr::null_mut();
    }
    let mut n = (*hfile).end.offset_from((*hfile).begin) as usize;
    if n > data_len {
        n = data_len;
    }
    if data_len != 0 {
        memcpy((*b).data.cast(), (*hfile).begin.cast(), n as u64);
    }
    (*hfile).begin = (*hfile).begin.add(n);
    let got = if n == data_len || ((*hfile).flags & HFILE_MOBILE) == 0 {
        n as libc::ssize_t
    } else {
        crate::htslib_rs::hfile::hread2(fp.cast(), (*b).data.cast(), data_len, n)
    };
    if got != data_len as libc::ssize_t {
        free((*b).data.cast());
        free(b.cast());
        return std::ptr::null_mut();
    }

    if ((*fd_layout).version >> 8) >= 3 {
        if cram_cram_io_c_1005_int32_decode(fd, (&mut (*b).crc32 as *mut u32).cast()) == -1 {
            free((*b).data.cast());
            free(b.cast());
            return std::ptr::null_mut();
        }
        (*b).crc32_checked = (*fd_layout).ignore_md5;
        (*b).crc_part = crc;
    } else {
        (*b).crc32_checked = 1;
        (*b).crc_part = 0;
        (*b).crc32 = 0;
    }
    (*b).orig_method = (*b).method;
    (*b).idx = 0;
    (*b).byte = 0;
    (*b).bit = 7;
    (*b).m = std::ptr::null_mut();
    b.cast()
}

pub unsafe fn cram_cram_io_c_1511_cram_write_block(
    fd: *mut hts_sys::cram_fd,
    b: *mut hts_sys::cram_block,
) -> c_int {
    let fd_layout = fd.cast::<cram_fd_layout>();
    let fp = (*fd_layout).fp;
    let hfile = fp.cast::<hfile_layout>();
    let b = b.cast::<cram_block_layout>();

    for c in [(*b).method, (*b).content_type] {
        let r = if (*hfile).begin < (*hfile).limit {
            *(*hfile).begin = c as c_char;
            (*hfile).begin = (*hfile).begin.add(1);
            c
        } else {
            crate::htslib_rs::hfile::hputc2(c, fp.cast())
        };
        if r == libc::EOF {
            return -1;
        }
    }

    let mut vardata = [0u8; 100];
    let mut vardata_o = 0usize;
    if ((*fd_layout).version >> 8) >= 4 {
        for val in [(*b).content_id, (*b).comp_size, (*b).uncomp_size] {
            let n = cram_cram_io_c_804_uint7_put_32(
                vardata.as_mut_ptr().add(vardata_o).cast(),
                vardata.as_mut_ptr().add(vardata.len()).cast(),
                val,
            );
            if n <= 0 {
                return -1;
            }
            vardata_o += n as usize;
        }
    } else {
        for val in [(*b).content_id, (*b).comp_size, (*b).uncomp_size] {
            let n = cram_cram_io_c_277_itf8_put(vardata.as_mut_ptr().add(vardata_o).cast(), val);
            vardata_o += n as usize;
        }
    }

    let mut n = (*hfile).limit.offset_from((*hfile).begin) as usize;
    let wrote = if vardata_o >= n && (*hfile).begin == (*hfile).buffer {
        crate::htslib_rs::hfile::hwrite2(fp.cast(), vardata.as_ptr().cast(), vardata_o, 0)
    } else {
        if n > vardata_o {
            n = vardata_o;
        }
        memcpy((*hfile).begin.cast(), vardata.as_ptr().cast(), n as u64);
        (*hfile).begin = (*hfile).begin.add(n);
        if n == vardata_o {
            n as libc::ssize_t
        } else {
            crate::htslib_rs::hfile::hwrite2(fp.cast(), vardata.as_ptr().cast(), vardata_o, n)
        }
    };
    if wrote != vardata_o as libc::ssize_t {
        return -1;
    }

    let data_len = if (*b).method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
        (*b).uncomp_size
    } else {
        (*b).comp_size
    };
    if !(*b).data.is_null() {
        let data_len = data_len as usize;
        let mut n = (*hfile).limit.offset_from((*hfile).begin) as usize;
        let wrote = if data_len >= n && (*hfile).begin == (*hfile).buffer {
            crate::htslib_rs::hfile::hwrite2(fp.cast(), (*b).data.cast(), data_len, 0)
        } else {
            if n > data_len {
                n = data_len;
            }
            memcpy((*hfile).begin.cast(), (*b).data.cast(), n as u64);
            (*hfile).begin = (*hfile).begin.add(n);
            if n == data_len {
                n as libc::ssize_t
            } else {
                crate::htslib_rs::hfile::hwrite2(fp.cast(), (*b).data.cast(), data_len, n)
            }
        };
        if wrote != data_len as libc::ssize_t {
            return -1;
        }
    }

    if ((*fd_layout).version >> 8) >= 3 {
        let mut dat = [0u8; 100];
        let mut cp = 0usize;
        dat[cp] = (*b).method as u8;
        cp += 1;
        dat[cp] = (*b).content_type as u8;
        cp += 1;
        if ((*fd_layout).version >> 8) >= 4 {
            for val in [(*b).content_id, (*b).comp_size, (*b).uncomp_size] {
                cp += cram_cram_io_c_804_uint7_put_32(
                    dat.as_mut_ptr().add(cp).cast(),
                    dat.as_mut_ptr().add(dat.len()).cast(),
                    val,
                ) as usize;
            }
        } else {
            for val in [(*b).content_id, (*b).comp_size, (*b).uncomp_size] {
                cp += cram_cram_io_c_277_itf8_put(dat.as_mut_ptr().add(cp).cast(), val) as usize;
            }
        }
        let mut crc = crate::htslib_rs::bgzf::hts_crc32(0, dat.as_ptr().cast(), cp);
        let data_len = if (*b).method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
            (*b).uncomp_size
        } else {
            (*b).comp_size
        } as usize;
        crc = crate::htslib_rs::bgzf::hts_crc32(
            crc,
            if (*b).data.is_null() {
                c"".as_ptr().cast()
            } else {
                (*b).data.cast()
            },
            data_len,
        );
        (*b).crc32 = crc;
        if cram_cram_io_c_1020_int32_encode(fd, crc as i32) == -1 {
            return -1;
        }
    }

    0
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

pub unsafe fn cram_cram_io_c_1576_cram_uncompress_block(b: *mut hts_sys::cram_block) -> c_int {
    let b = b.cast::<cram_block_layout>();

    if (*b).crc32_checked == 0 {
        let crc = crate::htslib_rs::bgzf::hts_crc32(
            (*b).crc_part,
            if (*b).data.is_null() {
                c"".as_ptr().cast()
            } else {
                (*b).data.cast()
            },
            (*b).alloc,
        );
        (*b).crc32_checked = 1;
        if crc != (*b).crc32 {
            return -1;
        }
    }

    if (*b).uncomp_size == 0 {
        (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
        return 0;
    }

    match (*b).method {
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW => 0,
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP => {
            let mut uncomp_size = (*b).uncomp_size as usize;
            let uncomp = htslib_zlib_mem_inflate(
                (*b).data.cast::<c_char>(),
                (*b).comp_size as usize,
                &mut uncomp_size,
            );
            if uncomp.is_null() {
                return -1;
            }
            if uncomp_size != (*b).uncomp_size as usize {
                free(uncomp.cast());
                return -1;
            }
            free((*b).data.cast());
            (*b).data = uncomp.cast();
            (*b).alloc = uncomp_size;
            (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            0
        }
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_BZIP2 => -1,
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_LZMA => -1,
        crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RANS => {
            let usize = (*b).uncomp_size as c_uint;
            let mut usize2 = 0 as c_uint;
            let uncomp = crate::htslib_rs::htscodecs::rans_4x8::rans_uncompress(
                (*b).data,
                (*b).comp_size as c_uint,
                &mut usize2,
            );
            if uncomp.is_null() {
                return -1;
            }
            if usize != usize2 {
                free(uncomp.cast());
                return -1;
            }
            free((*b).data.cast());
            (*b).data = uncomp;
            (*b).alloc = usize2 as usize;
            (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            (*b).uncomp_size = usize2 as i32;
            0
        }
        7 => {
            let mut uncomp_size = (*b).uncomp_size as usize;
            let input = std::slice::from_raw_parts((*b).data, (*b).comp_size as usize);
            let v = crate::htslib_rs::htscodecs::fqzcomp_qual::fqz_decompress(
                input,
                &mut uncomp_size,
                &mut [],
                0,
            );
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            free((*b).data.cast());
            (*b).data = uncomp;
            (*b).alloc = uncomp_size;
            (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            (*b).uncomp_size = uncomp_size as i32;
            0
        }
        5 => {
            let usize = (*b).uncomp_size as c_uint;
            let mut usize2 = 0 as c_uint;
            let input = std::slice::from_raw_parts((*b).data, (*b).comp_size as usize);
            let v = crate::htslib_rs::htscodecs::rans_static_4x16pr::rans_uncompress_4x16(
                input,
                &mut usize2,
            );
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            if usize != usize2 {
                free(uncomp.cast());
                return -1;
            }
            (*b).orig_method = 5;
            free((*b).data.cast());
            (*b).data = uncomp;
            (*b).alloc = usize2 as usize;
            (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            (*b).uncomp_size = usize2 as i32;
            0
        }
        6 => {
            let usize = (*b).uncomp_size as c_uint;
            let mut usize2 = 0 as c_uint;
            let input = std::slice::from_raw_parts((*b).data, (*b).comp_size as usize);
            let v = crate::htslib_rs::htscodecs::arith_dynamic::arith_uncompress_to(
                input,
                None,
                &mut usize2,
            );
            let uncomp = cram_dup_to_malloc(&v);
            if uncomp.is_null() {
                return -1;
            }
            if usize != usize2 {
                free(uncomp.cast());
                return -1;
            }
            (*b).orig_method = 6;
            free((*b).data.cast());
            (*b).data = uncomp;
            (*b).alloc = usize2 as usize;
            (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            (*b).uncomp_size = usize2 as i32;
            0
        }
        8 => {
            let mut out_len = 0u32;
            let input = std::slice::from_raw_parts((*b).data, (*b).comp_size as usize);
            let cp = match crate::htslib_rs::htscodecs::tokenise_name3::tok3_decode_names(
                input,
                (*b).comp_size as u32,
                &mut out_len,
            ) {
                Some(v) => cram_dup_to_malloc(&v),
                None => std::ptr::null_mut(),
            };
            if cp.is_null() {
                return -1;
            }
            (*b).orig_method = 8;
            (*b).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
            free((*b).data.cast());
            (*b).data = cp;
            (*b).alloc = out_len as usize;
            (*b).uncomp_size = out_len as i32;
            0
        }
        _ => -1,
    }
}

pub unsafe fn cram_cram_io_c_2327_cram_new_metrics() -> *mut hts_sys::cram_metrics {
    let m =
        calloc(1, std::mem::size_of::<cram_metrics_layout>() as u64).cast::<cram_metrics_layout>();
    if m.is_null() {
        return std::ptr::null_mut();
    }

    (*m).trial = 2;
    (*m).next_trial = 35;
    (*m).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW;
    (*m).strat = 0;
    (*m).revised_method = 0;
    (*m).unpackable = 0;

    m.cast()
}

pub unsafe fn cram_cram_io_c_2417_ref_entry_free_seq(e: *mut c_void) {
    let e = e.cast::<ref_entry_layout>();
    if !(*e).mf.is_null() {
        cram_mFILE_c_361_mfclose((*e).mf);
    }
    if !(*e).seq.is_null() && (*e).mf.is_null() {
        free((*e).seq.cast());
    }

    (*e).seq = std::ptr::null_mut();
    (*e).mf = std::ptr::null_mut();
}

pub unsafe fn cram_cram_io_c_2427_refs_free(r: *mut refs_t) {
    let r = r.cast::<refs_t_layout>();
    if r.is_null() {
        return;
    }

    (*r).count -= 1;
    if (*r).count > 0 {
        return;
    }

    if !(*r).pool.is_null() {
        cram_string_alloc_c_103_string_pool_destroy((*r).pool);
    }

    if !(*r).h_meta.is_null() {
        let h = (*r).h_meta;
        for k in 0..(*h).n_buckets {
            if ((*(*h).flags.add((k >> 4) as usize) >> ((k & 0x0f) << 1)) & 3) != 0 {
                continue;
            }
            let e = *(*h).vals.add(k as usize);
            if e.is_null() {
                continue;
            }
            cram_cram_io_c_2417_ref_entry_free_seq(e.cast());
            free(e.cast());
        }
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }

    if !(*r).ref_id.is_null() {
        free((*r).ref_id.cast());
    }

    if !(*r).fp.is_null() {
        bgzf_close((*r).fp);
    }

    libc::pthread_mutex_destroy(&mut (*r).lock);

    free(r.cast());
}

pub unsafe fn cram_cram_io_c_2467_refs_create() -> *mut refs_t {
    let r = calloc(1, std::mem::size_of::<refs_t_layout>() as u64).cast::<refs_t_layout>();
    if r.is_null() {
        return std::ptr::null_mut();
    }

    (*r).pool = cram_string_alloc_c_55_string_pool_create(8192);
    if (*r).pool.is_null() {
        cram_cram_io_c_2427_refs_free(r.cast());
        return std::ptr::null_mut();
    }

    (*r).ref_id = std::ptr::null_mut();
    (*r).count = 1;
    (*r).last = std::ptr::null_mut();
    (*r).last_id = -1;

    (*r).h_meta = calloc(1, std::mem::size_of::<kh_refs_layout>() as u64).cast::<kh_refs_layout>();
    if (*r).h_meta.is_null() {
        cram_cram_io_c_2427_refs_free(r.cast());
        return std::ptr::null_mut();
    }

    libc::pthread_mutex_init(&mut (*r).lock, std::ptr::null());

    r.cast()
}

pub unsafe fn cram_cram_io_c_2503_bgzf_open_ref(
    mut fn_: *mut c_char,
    mode: *mut c_char,
    is_md5: c_int,
) -> *mut BGZF {
    if libc::strncmp(fn_, c"file://".as_ptr(), 7) == 0 {
        fn_ = fn_.add(7);
    }

    if is_md5 == 0 && hisremote(fn_) == 0 {
        let mut fai_file = [0 as c_char; libc::PATH_MAX as usize];
        libc::snprintf(
            fai_file.as_mut_ptr(),
            libc::PATH_MAX as usize,
            c"%s.fai".as_ptr(),
            fn_,
        );
        if libc::access(fai_file.as_ptr(), libc::R_OK) != 0 && fai_build(fn_) != 0 {
            return std::ptr::null_mut();
        }
    }

    let fp = bgzf_open(fn_, mode);
    if fp.is_null() {
        libc::perror(fn_);
        return std::ptr::null_mut();
    }

    if ((*fp).bitfields & (1 << 30)) != 0 && bgzf_index_load(fp, fn_, c".gzi".as_ptr()) < 0 {
        let msg = std::ffi::CString::new(format!(
            "Unable to load .gzi index '{}.gzi'",
            CStr::from_ptr(fn_).to_string_lossy()
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"bgzf_open_ref".as_ptr(), msg.as_ptr());
        bgzf_close(fp);
        return std::ptr::null_mut();
    }

    fp
}

pub unsafe fn cram_cram_io_c_2541_refs_load_fai(
    r_orig: *mut refs_t,
    fn_: *const c_char,
    is_err: c_int,
) -> *mut refs_t {
    let mut fai_fn = [0 as c_char; libc::PATH_MAX as usize];
    let mut line = [0 as c_char; 8192];
    let mut r = r_orig.cast::<refs_t_layout>();
    let fn_l = libc::strlen(fn_);
    let mut id = 0i32;
    let mut id_alloc = 0i32;

    if r.is_null() {
        r = cram_cram_io_c_2467_refs_create().cast::<refs_t_layout>();
        if r.is_null() {
            return std::ptr::null_mut();
        }
    }

    if !(*r).fp.is_null() && bgzf_close((*r).fp) != 0 {
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }
    (*r).fp = std::ptr::null_mut();

    let fn_delim = libc::strstr(fn_, c"##idx##".as_ptr());
    if !fn_delim.is_null() {
        (*r).fn_ =
            cram_string_alloc_c_153_string_ndup((*r).pool, fn_, fn_delim.offset_from(fn_) as usize);
        if (*r).fn_.is_null() {
            if r_orig.is_null() {
                cram_cram_io_c_2427_refs_free(r.cast());
            }
            return std::ptr::null_mut();
        }
        let idx = fn_delim.add(7);
        libc::snprintf(
            fai_fn.as_mut_ptr(),
            libc::PATH_MAX as usize,
            c"%s".as_ptr(),
            idx,
        );
    } else if fn_l > 4 && libc::strcmp(fn_.add(fn_l - 4), c".fai".as_ptr()) == 0 {
        if (*r).fn_.is_null() {
            (*r).fn_ = cram_string_alloc_c_153_string_ndup((*r).pool, fn_, fn_l - 4);
            if (*r).fn_.is_null() {
                if r_orig.is_null() {
                    cram_cram_io_c_2427_refs_free(r.cast());
                }
                return std::ptr::null_mut();
            }
        }
        libc::snprintf(
            fai_fn.as_mut_ptr(),
            libc::PATH_MAX as usize,
            c"%s".as_ptr(),
            fn_,
        );
    } else {
        (*r).fn_ = cram_string_alloc_c_149_string_dup((*r).pool, fn_);
        if (*r).fn_.is_null() {
            if r_orig.is_null() {
                cram_cram_io_c_2427_refs_free(r.cast());
            }
            return std::ptr::null_mut();
        }
        libc::snprintf(
            fai_fn.as_mut_ptr(),
            libc::PATH_MAX as usize,
            c"%.*s.fai".as_ptr(),
            libc::PATH_MAX - 5,
            fn_,
        );
    }

    (*r).fp = cram_cram_io_c_2503_bgzf_open_ref((*r).fn_, c"r".as_ptr().cast_mut(), 0);
    if (*r).fp.is_null() {
        let msg = std::ffi::CString::new(format!(
            "Failed to open reference file '{}'",
            CStr::from_ptr((*r).fn_).to_string_lossy()
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"refs_load_fai".as_ptr(), msg.as_ptr());
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }

    let fp = hopen(fai_fn.as_ptr(), c"r".as_ptr());
    if fp.is_null() {
        let msg = std::ffi::CString::new(format!(
            "Failed to open index file '{}'",
            CStr::from_ptr(fai_fn.as_ptr()).to_string_lossy()
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"refs_load_fai".as_ptr(), msg.as_ptr());
        if is_err != 0 {
            libc::perror(fai_fn.as_ptr());
        }
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }

    while !hgets(line.as_mut_ptr(), 8192, fp).is_null() {
        let e = malloc(std::mem::size_of::<ref_entry_layout>() as u64).cast::<ref_entry_layout>();
        if e.is_null() {
            hclose_abruptly(fp);
            if r_orig.is_null() {
                cram_cram_io_c_2427_refs_free(r.cast());
            }
            return std::ptr::null_mut();
        }
        std::ptr::write_bytes(e, 0, 1);

        let mut cp = line.as_mut_ptr();
        while *cp != 0 && isspace_c(*cp) == 0 {
            cp = cp.add(1);
        }
        *cp = 0;
        cp = cp.add(1);
        (*e).name = cram_string_alloc_c_149_string_dup((*r).pool, line.as_ptr());

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).length = libc::strtoll(cp, &mut cp, 10);

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).offset = libc::strtoll(cp, &mut cp, 10);

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).bases_per_line = libc::strtol(cp, &mut cp, 10) as c_int;

        while *cp != 0 && isspace_c(*cp) != 0 {
            cp = cp.add(1);
        }
        (*e).line_length = libc::strtol(cp, &mut cp, 10) as c_int;
        (*e).fn_ = (*r).fn_;
        (*e).count = 0;
        (*e).seq = std::ptr::null_mut();
        (*e).mf = std::ptr::null_mut();
        (*e).is_md5 = 0;
        (*e).validated_md5 = 0;

        if (*e).name.is_null() {
            free(e.cast());
            hclose_abruptly(fp);
            if r_orig.is_null() {
                cram_cram_io_c_2427_refs_free(r.cast());
            }
            return std::ptr::null_mut();
        }

        let h = (*r).h_meta;
        if (*h).n_occupied >= (*h).upper_bound {
            let mut new_n_buckets = (*h).n_buckets + 1;
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

            let flags_words = if new_n_buckets < 16 {
                1
            } else {
                new_n_buckets >> 4
            };
            let new_flags =
                malloc((flags_words as usize * std::mem::size_of::<u32>()) as u64).cast::<u32>();
            let new_keys = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*const c_char>() as u64,
            )
            .cast::<*const c_char>();
            let new_vals = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*mut ref_entry_layout>() as u64,
            )
            .cast::<*mut ref_entry_layout>();
            if new_flags.is_null() || new_keys.is_null() || new_vals.is_null() {
                free(new_flags.cast());
                free(new_keys.cast());
                free(new_vals.cast());
                free(e.cast());
                hclose_abruptly(fp);
                if r_orig.is_null() {
                    cram_cram_io_c_2427_refs_free(r.cast());
                }
                return std::ptr::null_mut();
            }
            for x in 0..flags_words {
                *new_flags.add(x as usize) = 0xaaaa_aaaa;
            }

            for old in 0..(*h).n_buckets {
                if ((*(*h).flags.add((old >> 4) as usize) >> ((old & 0x0f) << 1)) & 3) != 0 {
                    continue;
                }
                let key = *(*h).keys.add(old as usize);
                let val = *(*h).vals.add(old as usize);
                let mut hash = 2166136261u32;
                let mut p = key;
                while *p != 0 {
                    hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                    p = p.add(1);
                }
                let mut i = hash & (new_n_buckets - 1);
                let mut step = 0u32;
                while ((*new_flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) == 0 {
                    step += 1;
                    i = (i + step) & (new_n_buckets - 1);
                }
                *new_keys.add(i as usize) = key;
                *new_vals.add(i as usize) = val;
                *new_flags.add((i >> 4) as usize) &= !(3u32 << ((i & 0x0f) << 1));
            }

            free((*h).flags.cast());
            free((*h).keys.cast());
            free((*h).vals.cast());
            (*h).flags = new_flags;
            (*h).keys = new_keys;
            (*h).vals = new_vals;
            (*h).n_buckets = new_n_buckets;
            (*h).n_occupied = (*h).size;
            (*h).upper_bound = ((*h).n_buckets as f64 * 0.77 + 0.5) as u32;
        }

        let mut ret = 0;
        let mut hash = 2166136261u32;
        let mut p = (*e).name;
        while *p != 0 {
            hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
            p = p.add(1);
        }
        let mask = (*h).n_buckets - 1;
        let mut x = (*h).n_buckets;
        let mut site = (*h).n_buckets;
        let mut i = hash & mask;
        if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0 {
            x = i;
        } else {
            let last = i;
            let mut step = 0u32;
            while ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) == 0
                && (((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
                    || libc::strcmp(*(*h).keys.add(i as usize), (*e).name) != 0)
            {
                if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0 {
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
                if ((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
                    && site != (*h).n_buckets
                {
                    x = site;
                } else {
                    x = i;
                }
            }
        }

        if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) != 0 {
            *(*h).keys.add(x as usize) = (*e).name;
            *(*h).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*h).size += 1;
            (*h).n_occupied += 1;
            ret = 1;
        } else if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0 {
            *(*h).keys.add(x as usize) = (*e).name;
            *(*h).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*h).size += 1;
            ret = 2;
        }

        if ret != 0 {
            *(*h).vals.add(x as usize) = e;
        } else {
            let re = *(*h).vals.add(x as usize);
            if !re.is_null() && ((*re).count != 0 || (*re).length != 0) {
                free(e.cast());
            } else {
                if !re.is_null() {
                    free(re.cast());
                }
                *(*h).vals.add(x as usize) = e;
            }
        }

        if id >= id_alloc {
            id_alloc = if id_alloc != 0 { id_alloc * 2 } else { 16 };
            let new_refs = realloc(
                (*r).ref_id.cast(),
                (id_alloc as usize * std::mem::size_of::<*mut ref_entry_layout>()) as u64,
            )
            .cast::<*mut ref_entry_layout>();
            if new_refs.is_null() {
                hclose_abruptly(fp);
                if r_orig.is_null() {
                    cram_cram_io_c_2427_refs_free(r.cast());
                }
                return std::ptr::null_mut();
            }
            (*r).ref_id = new_refs;
            for x in id..id_alloc {
                *(*r).ref_id.add(x as usize) = std::ptr::null_mut();
            }
        }
        *(*r).ref_id.add(id as usize) = e;
        id += 1;
        (*r).nref = id;
    }

    if hclose(fp) < 0 {
        if r_orig.is_null() {
            cram_cram_io_c_2427_refs_free(r.cast());
        }
        return std::ptr::null_mut();
    }

    r.cast()
}

pub unsafe fn cram_cram_io_c_2693_sanitise_SQ_lines(fd: *mut cram_fd) {
    let fd = fd.cast::<cram_fd_layout>();
    if (*fd).header.is_null() || (*(*fd).header.cast::<sam_hdr_t>()).hrecs.is_null() {
        return;
    }

    if (*fd).refs.is_null() || (*(*fd).refs.cast::<refs_t_layout>()).h_meta.is_null() {
        return;
    }

    let hdr = (*fd).header.cast::<sam_hdr_t>();
    let hrecs = (*hdr).hrecs;
    let refs = (*fd).refs.cast::<refs_t_layout>();
    let h = (*refs).h_meta;
    for iref in 0..(*hrecs).nref {
        let name = (*(*hrecs).ref_.add(iref as usize)).name;
        let mut k = (*h).n_buckets;
        if (*h).n_buckets != 0 {
            let mut hash = 2166136261u32;
            let mut p = name;
            while *p != 0 {
                hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                p = p.add(1);
            }
            let mask = (*h).n_buckets - 1;
            let mut x = hash & mask;
            let last = x;
            let mut step = 0u32;
            while ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0
                && (((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0
                    || libc::strcmp(*(*h).keys.add(x as usize), name) != 0)
            {
                step += 1;
                x = (x + step) & mask;
                if x == last {
                    break;
                }
            }
            if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 3) == 0 {
                k = x;
            }
        }
        if k == (*h).n_buckets {
            continue;
        }

        let r = *(*h).vals.add(k as usize);
        if r.is_null() {
            continue;
        }

        if (*r).length != 0 && (*r).length != (*(*hrecs).ref_.add(iref as usize)).len {
            assert_eq!(
                libc::strcmp((*r).name, (*(*hrecs).ref_.add(iref as usize)).name),
                0
            );
            let msg = std::ffi::CString::new(format!(
                "Header @SQ length mismatch for ref {}, {} vs {}",
                CStr::from_ptr((*r).name).to_string_lossy(),
                (*(*hrecs).ref_.add(iref as usize)).len,
                (*r).length as c_int,
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"sanitise_SQ_lines".as_ptr(), msg.as_ptr());
            (*(*hrecs).ref_.add(iref as usize)).len = (*r).length;
        }
    }
}

pub unsafe fn cram_cram_io_c_2737_refs2id(r: *mut refs_t, hdr: *mut sam_hdr_t) -> c_int {
    let r = r.cast::<refs_t_layout>();
    let hrec = (*hdr).hrecs;

    if !(*r).ref_id.is_null() {
        free((*r).ref_id.cast());
    }
    if !(*r).last.is_null() {
        (*r).last = std::ptr::null_mut();
    }

    (*r).ref_id = calloc(
        (*hrec).nref as u64,
        std::mem::size_of::<*mut ref_entry_layout>() as u64,
    )
    .cast::<*mut ref_entry_layout>();
    if (*r).ref_id.is_null() {
        return -1;
    }

    (*r).nref = (*hrec).nref;
    let h = (*r).h_meta;
    for iref in 0..(*hrec).nref {
        let name = (*(*hrec).ref_.add(iref as usize)).name;
        let mut k = (*h).n_buckets;
        if (*h).n_buckets != 0 {
            let mut hash = 2166136261u32;
            let mut p = name;
            while *p != 0 {
                hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                p = p.add(1);
            }
            let mask = (*h).n_buckets - 1;
            let mut x = hash & mask;
            let last = x;
            let mut step = 0u32;
            while ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0
                && (((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0
                    || libc::strcmp(*(*h).keys.add(x as usize), name) != 0)
            {
                step += 1;
                x = (x + step) & mask;
                if x == last {
                    break;
                }
            }
            if ((*(*h).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 3) == 0 {
                k = x;
            }
        }

        if k != (*h).n_buckets {
            *(*r).ref_id.add(iref as usize) = *(*h).vals.add(k as usize);
        } else {
            let msg = std::ffi::CString::new(format!(
                "Unable to find ref name '{}'",
                CStr::from_ptr(name).to_string_lossy()
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"refs2id".as_ptr(), msg.as_ptr());
        }
    }

    0
}

pub unsafe fn cram_cram_io_c_2852_cram_set_header2(
    fd: *mut cram_fd,
    hdr: *const sam_hdr_t,
) -> c_int {
    if fd.is_null() || hdr.is_null() {
        return -1;
    }

    let fd = fd.cast::<cram_fd_layout>();
    if (*fd).header != hdr.cast_mut().cast() {
        if !(*fd).header.is_null() {
            sam_hdr_destroy((*fd).header.cast());
        }
        (*fd).header = sam_hdr_dup(hdr).cast();
        if (*fd).header.is_null() {
            return -1;
        }
    }

    cram_cram_io_c_2768_refs_from_header(fd.cast())
}

pub unsafe fn cram_cram_io_c_2866_cram_set_header(fd: *mut cram_fd, hdr: *mut sam_hdr_t) -> c_int {
    cram_cram_io_c_2852_cram_set_header2(fd, hdr)
}

pub unsafe fn cram_cram_io_c_2768_refs_from_header(fd: *mut cram_fd) -> c_int {
    if fd.is_null() {
        return -1;
    }

    let fd = fd.cast::<cram_fd_layout>();
    let r = (*fd).refs.cast::<refs_t_layout>();
    if r.is_null() {
        return -1;
    }

    let h = (*fd).header.cast::<sam_hdr_t>();
    if h.is_null() {
        return 0;
    }

    if (*h).hrecs.is_null() && htslib_sam_hdr_fill_hrecs(h) == -1 {
        return -1;
    }

    let hrecs = (*h).hrecs;
    if (*hrecs).nref == 0 {
        return 0;
    }

    let new_ref_id = realloc(
        (*r).ref_id.cast(),
        (((*r).nref + (*hrecs).nref) as usize * std::mem::size_of::<*mut ref_entry_layout>())
            as u64,
    )
    .cast::<*mut ref_entry_layout>();
    if new_ref_id.is_null() {
        return -1;
    }
    (*r).ref_id = new_ref_id;

    let mut j = (*r).nref;
    for i in 0..(*hrecs).nref {
        let h_ref = (*hrecs).ref_.add(i as usize);
        let name = (*h_ref).name;

        let kh = (*r).h_meta;
        let mut k = (*kh).n_buckets;
        if (*kh).n_buckets != 0 {
            let mut hash = 2166136261u32;
            let mut p = name;
            while *p != 0 {
                hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                p = p.add(1);
            }
            let mask = (*kh).n_buckets - 1;
            let mut x = hash & mask;
            let last = x;
            let mut step = 0u32;
            while ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0
                && (((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0
                    || libc::strcmp(*(*kh).keys.add(x as usize), name) != 0)
            {
                step += 1;
                x = (x + step) & mask;
                if x == last {
                    break;
                }
            }
            if ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 3) == 0 {
                k = x;
            }
        }
        if k != (*kh).n_buckets {
            continue;
        }

        let e =
            calloc(1, std::mem::size_of::<ref_entry_layout>() as u64).cast::<ref_entry_layout>();
        if e.is_null() {
            return -1;
        }
        *(*r).ref_id.add(j as usize) = e;

        if name.is_null() {
            return -1;
        }

        (*e).name = cram_string_alloc_c_149_string_dup((*r).pool, name);
        if (*e).name.is_null() {
            return -1;
        }
        (*e).length = 0;

        if !(*h_ref).ty.is_null() {
            let tag = crate::htslib_rs::sam::sam_hrecs_find_key(
                (*h_ref).ty.cast(),
                c"M5".as_ptr(),
                std::ptr::null_mut(),
            )
            .cast::<sam_hrec_tag_layout>();
            if !tag.is_null() {
                (*e).fn_ = cram_string_alloc_c_149_string_dup((*r).pool, (*tag).str_.add(3));
            }

            let tag = crate::htslib_rs::sam::sam_hrecs_find_key(
                (*h_ref).ty.cast(),
                c"LN".as_ptr(),
                std::ptr::null_mut(),
            )
            .cast::<sam_hrec_tag_layout>();
            if !tag.is_null() {
                (*e).ln_length = libc::strtoll((*tag).str_.add(3), std::ptr::null_mut(), 0);
                if (*e).ln_length < 0 {
                    (*e).ln_length = 0;
                }
            }
        }

        if (*kh).n_occupied >= (*kh).upper_bound {
            let mut new_n_buckets = (*kh).n_buckets + 1;
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

            let flags_words = if new_n_buckets < 16 {
                1
            } else {
                new_n_buckets >> 4
            };
            let new_flags =
                malloc((flags_words as usize * std::mem::size_of::<u32>()) as u64).cast::<u32>();
            let new_keys = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*const c_char>() as u64,
            )
            .cast::<*const c_char>();
            let new_vals = calloc(
                new_n_buckets as u64,
                std::mem::size_of::<*mut ref_entry_layout>() as u64,
            )
            .cast::<*mut ref_entry_layout>();
            if new_flags.is_null() || new_keys.is_null() || new_vals.is_null() {
                free(new_flags.cast());
                free(new_keys.cast());
                free(new_vals.cast());
                return -1;
            }
            for x in 0..flags_words {
                *new_flags.add(x as usize) = 0xaaaa_aaaa;
            }

            for old in 0..(*kh).n_buckets {
                if ((*(*kh).flags.add((old >> 4) as usize) >> ((old & 0x0f) << 1)) & 3) != 0 {
                    continue;
                }
                let key = *(*kh).keys.add(old as usize);
                let val = *(*kh).vals.add(old as usize);
                let mut hash = 2166136261u32;
                let mut p = key;
                while *p != 0 {
                    hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
                    p = p.add(1);
                }
                let mut x = hash & (new_n_buckets - 1);
                let mut step = 0u32;
                while ((*new_flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) == 0 {
                    step += 1;
                    x = (x + step) & (new_n_buckets - 1);
                }
                *new_keys.add(x as usize) = key;
                *new_vals.add(x as usize) = val;
                *new_flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            }

            free((*kh).flags.cast());
            free((*kh).keys.cast());
            free((*kh).vals.cast());
            (*kh).flags = new_flags;
            (*kh).keys = new_keys;
            (*kh).vals = new_vals;
            (*kh).n_buckets = new_n_buckets;
            (*kh).n_occupied = (*kh).size;
            (*kh).upper_bound = ((*kh).n_buckets as f64 * 0.77 + 0.5) as u32;
        }

        let mut ret = 0;
        let mut hash = 2166136261u32;
        let mut p = (*e).name;
        while *p != 0 {
            hash = (hash ^ (*p as u8 as u32)).wrapping_mul(16777619);
            p = p.add(1);
        }
        let mask = (*kh).n_buckets - 1;
        let mut x = (*kh).n_buckets;
        let mut site = (*kh).n_buckets;
        let mut pos = hash & mask;
        if ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 2) != 0 {
            x = pos;
        } else {
            let last = pos;
            let mut step = 0u32;
            while ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 2) == 0
                && (((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 1) != 0
                    || libc::strcmp(*(*kh).keys.add(pos as usize), (*e).name) != 0)
            {
                if ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 1) != 0 {
                    site = pos;
                }
                step += 1;
                pos = (pos + step) & mask;
                if pos == last {
                    x = site;
                    break;
                }
            }
            if x == (*kh).n_buckets {
                if ((*(*kh).flags.add((pos >> 4) as usize) >> ((pos & 0x0f) << 1)) & 2) != 0
                    && site != (*kh).n_buckets
                {
                    x = site;
                } else {
                    x = pos;
                }
            }
        }

        if ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 2) != 0 {
            *(*kh).keys.add(x as usize) = (*e).name;
            *(*kh).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*kh).size += 1;
            (*kh).n_occupied += 1;
            ret = 1;
        } else if ((*(*kh).flags.add((x >> 4) as usize) >> ((x & 0x0f) << 1)) & 1) != 0 {
            *(*kh).keys.add(x as usize) = (*e).name;
            *(*kh).flags.add((x >> 4) as usize) &= !(3u32 << ((x & 0x0f) << 1));
            (*kh).size += 1;
            ret = 2;
        }
        if ret <= 0 {
            return -1;
        }
        *(*kh).vals.add(x as usize) = e;
        j += 1;
    }
    (*r).nref = j;

    0
}

pub unsafe fn cram_cram_io_c_3169_cram_ref_incr_locked(r: *mut refs_t, id: c_int) {
    let r = r.cast::<refs_t_layout>();
    if id < 0
        || (*(*r).ref_id.add(id as usize)).is_null()
        || (*(*(*r).ref_id.add(id as usize))).seq.is_null()
    {
        return;
    }

    if (*r).last_id == id {
        (*r).last_id = -1;
    }

    (*(*(*r).ref_id.add(id as usize))).count += 1;
}

pub unsafe fn cram_cram_io_c_3183_cram_ref_incr(r: *mut refs_t, id: c_int) {
    let rl = r.cast::<refs_t_layout>();
    libc::pthread_mutex_lock(&mut (*rl).lock);
    cram_cram_io_c_3169_cram_ref_incr_locked(r, id);
    libc::pthread_mutex_unlock(&mut (*rl).lock);
}

pub unsafe fn cram_cram_io_c_3189_cram_ref_decr_locked(r: *mut refs_t, id: c_int) {
    let r = r.cast::<refs_t_layout>();
    if id < 0
        || (*(*r).ref_id.add(id as usize)).is_null()
        || (*(*(*r).ref_id.add(id as usize))).seq.is_null()
    {
        return;
    }

    let e = *(*r).ref_id.add(id as usize);
    (*e).count -= 1;
    if (*e).count <= 0 {
        assert_eq!((*e).count, 0);
        if (*r).last_id >= 0 {
            let last = *(*r).ref_id.add((*r).last_id as usize);
            if (*last).count <= 0 && !(*last).seq.is_null() {
                cram_cram_io_c_2417_ref_entry_free_seq(last.cast());
                if (*last).is_md5 != 0 {
                    (*last).length = 0;
                }
            }
        }
        (*r).last_id = id;
    }
}

pub unsafe fn cram_cram_io_c_3213_cram_ref_decr(r: *mut refs_t, id: c_int) {
    let rl = r.cast::<refs_t_layout>();
    libc::pthread_mutex_lock(&mut (*rl).lock);
    cram_cram_io_c_3189_cram_ref_decr_locked(r, id);
    libc::pthread_mutex_unlock(&mut (*rl).lock);
}

pub unsafe fn cram_cram_io_c_3228_load_ref_portion(
    fp: *mut BGZF,
    e: *mut c_void,
    start: i64,
    mut end: i64,
) -> *mut c_char {
    let e = e.cast::<ref_entry_layout>();

    if end < start {
        end = start;
    }

    let offset = if (*e).line_length != 0 {
        (*e).offset
            + (start - 1) / (*e).bases_per_line as i64 * (*e).line_length as i64
            + (start - 1) % (*e).bases_per_line as i64
    } else {
        start - 1
    };

    let len = (if (*e).line_length != 0 {
        (*e).offset
            + (end - 1) / (*e).bases_per_line as i64 * (*e).line_length as i64
            + (end - 1) % (*e).bases_per_line as i64
    } else {
        end - 1
    }) - offset
        + 1;

    if bgzf_useek(fp, offset, libc::SEEK_SET) < 0 {
        libc::perror(c"bgzf_useek() on reference file".as_ptr());
        return std::ptr::null_mut();
    }

    if len == 0 {
        return std::ptr::null_mut();
    }
    let seq = malloc(len as u64).cast::<c_char>();
    if seq.is_null() {
        return std::ptr::null_mut();
    }

    if bgzf_read(fp, seq.cast(), len as usize) != len as isize {
        libc::perror(c"bgzf_read() on reference file".as_ptr());
        free(seq.cast());
        return std::ptr::null_mut();
    }

    if len != end - start + 1 {
        let mut i = 0i64;
        let mut j = 0i64;
        while i < len {
            let ch = *seq.add(i as usize);
            if isspace_c(ch) == 0 {
                *seq.add(j as usize) = ((ch as u8) & !0x20) as c_char;
                j += 1;
            } else {
                break;
            }
            i += 1;
        }
        while i < len && isspace_c(*seq.add(i as usize)) != 0 {
            i += 1;
        }
        while i < len - (*e).line_length as i64 {
            let j_end = j + (*e).bases_per_line as i64;
            while j < j_end {
                *seq.add(j as usize) = ((*seq.add(i as usize) as u8) & !0x20) as c_char;
                j += 1;
                i += 1;
            }
            i += ((*e).line_length - (*e).bases_per_line) as i64;
        }
        while i < len {
            let ch = *seq.add(i as usize);
            if isspace_c(ch) == 0 {
                *seq.add(j as usize) = ((ch as u8) & !0x20) as c_char;
                j += 1;
            }
            i += 1;
        }

        if j != end - start + 1 {
            hts_log_cstr(
                HTS_LOG_ERROR,
                c"load_ref_portion".as_ptr(),
                c"Malformed reference file".as_ptr(),
            );
            free(seq.cast());
            return std::ptr::null_mut();
        }
    } else {
        for i in 0..len {
            *seq.add(i as usize) = toupper_c(*seq.add(i as usize));
        }
    }

    seq
}

pub unsafe fn cram_cram_io_c_3323_cram_ref_load(
    r: *mut refs_t,
    id: c_int,
    is_md5: c_int,
) -> *mut c_void {
    let r = r.cast::<refs_t_layout>();
    let e = *(*r).ref_id.add(id as usize);
    let start = 1i64;
    let end = (*e).length;

    if !(*e).seq.is_null() {
        return e.cast();
    }

    assert_eq!((*e).count, 0);

    if !(*r).last.is_null() {
        assert!((*(*r).last).count > 0);
        (*(*r).last).count -= 1;
        if (*(*r).last).count <= 0 && !(*(*r).last).seq.is_null() {
            cram_cram_io_c_2417_ref_entry_free_seq((*r).last.cast());
        }
    }

    if (*r).fn_.is_null() {
        return std::ptr::null_mut();
    }

    if libc::strcmp((*r).fn_, (*e).fn_) != 0 || (*r).fp.is_null() {
        if !(*r).fp.is_null() && bgzf_close((*r).fp) != 0 {
            return std::ptr::null_mut();
        }
        (*r).fn_ = (*e).fn_;
        (*r).fp = cram_cram_io_c_2503_bgzf_open_ref((*r).fn_, c"r".as_ptr().cast_mut(), is_md5);
        if (*r).fp.is_null() {
            return std::ptr::null_mut();
        }
    }

    let seq = cram_cram_io_c_3228_load_ref_portion((*r).fp, e.cast(), start, end);
    if seq.is_null() {
        return std::ptr::null_mut();
    }

    (*e).seq = seq;
    (*e).mf = std::ptr::null_mut();
    (*e).count += 1;
    (*r).last = e;
    (*e).count += 1;

    e.cast()
}

// original: cram_populate_ref (htslib/cram/cram_io.c:2979)
//
// Locates the on-disk reference for ref id `id` and, where possible, fills in
// the ref_entry so that cram_get_ref can read the bases via load_ref_portion.
//
// Mirrors the HAVE_MMAP-defined build of htslib (the default): the
// `#ifndef HAVE_MMAP` REF_PATH `find_path` shortcut is omitted, and we rely on
// the native `open_path_mfile` to load full sequences from REF_PATH/REF_CACHE.
pub unsafe fn cram_cram_io_c_2977_cram_populate_ref(
    fd: *mut cram_fd,
    id: c_int,
    r: *mut c_void,
) -> c_int {
    let fdl = fd.cast::<cram_fd_layout>();
    let r = r.cast::<ref_entry_layout>();
    let ref_path = libc::getenv(c"REF_PATH".as_ptr());
    let local_cache = libc::getenv(c"REF_CACHE".as_ptr());
    let mut path = [0i8; libc::PATH_MAX as usize];
    let mut path_tmp: kstring_t = std::mem::zeroed();
    let mut local_path = 0i32;

    {
        let msg = std::ffi::CString::new(format!(
            "Running cram_populate_ref on fd {:p}, id {}",
            fd, id
        ))
        .unwrap();
        hts_log_cstr(HTS_LOG_INFO, c"cram_populate_ref".as_ptr(), msg.as_ptr());
    }

    if (*r).name.is_null() {
        return -1;
    }

    let hrecs = (*(*fdl).header.cast::<sam_hdr_t>()).hrecs;
    let ty = htslib_sam_hrecs_find_type_id(
        hrecs.cast(),
        c"SQ".as_ptr(),
        c"SN".as_ptr(),
        (*r).name,
    );
    if ty.is_null() {
        return -1;
    }

    let m5tag = crate::htslib_rs::sam::sam_hrecs_find_key(
        ty.cast(),
        c"M5".as_ptr(),
        std::ptr::null_mut(),
    )
    .cast::<sam_hrec_tag_layout>();

    // `'no_M5` block models C's `goto no_M5;` target.
    let from_m5: bool = !m5tag.is_null();
    if from_m5 {
        let m5 = (*m5tag).str_.add(3);
        {
            let msg = std::ffi::CString::new(format!(
                "Querying ref {}",
                CStr::from_ptr(m5).to_string_lossy()
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_INFO, c"cram_populate_ref".as_ptr(), msg.as_ptr());
        }

        // Use cache if available.
        if !local_cache.is_null() && *local_cache != 0 {
            let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
            if cram_cram_io_c_2884_expand_cache_path(path.as_mut_ptr(), local_cache, m5) == 0
                && libc::stat(path.as_ptr(), sb.as_mut_ptr()) == 0
            {
                local_path = 1;
            }
        }

        // Found via REF_CACHE: open it and fall back to cram_get_ref().
        if local_path != 0 {
            let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
            if libc::stat(path.as_ptr(), sb.as_mut_ptr()) == 0 {
                let sb = sb.assume_init();
                if (sb.st_mode & libc::S_IFMT) == libc::S_IFREG {
                    let fp = bgzf_open(path.as_ptr(), c"r".as_ptr());
                    if !fp.is_null() {
                        (*r).length = sb.st_size as i64;
                        (*r).offset = 0;
                        (*r).line_length = 0;
                        (*r).bases_per_line = 0;
                        (*r).fn_ = cram_string_alloc_c_149_string_dup(
                            (*(*fdl).refs.cast::<refs_t_layout>()).pool,
                            path.as_ptr(),
                        );
                        let refs = (*fdl).refs.cast::<refs_t_layout>();
                        if !(*refs).fp.is_null() && bgzf_close((*refs).fp) != 0 {
                            return -1;
                        }
                        (*refs).fp = fp;
                        (*refs).fn_ = (*r).fn_;
                        (*r).is_md5 = 1;
                        (*r).validated_md5 = 1;
                        return 0;
                    }
                }
            }
        }

        // Otherwise search full REF_PATH; slower as it loads the entire file.
        let mut is_local = 0i32;
        let mf = cram_open_trace_file_c_352_open_path_mfile(
            m5,
            ref_path,
            std::ptr::null_mut(),
            &mut is_local,
        );
        if !mf.is_null() {
            let mut sz: usize = 0;
            let stolen = cram_mFILE_c_428_mfsteal(mf, &mut sz).cast::<c_char>();
            if !stolen.is_null() {
                (*r).seq = stolen;
                (*r).mf = std::ptr::null_mut();
            } else {
                // Couldn't detach; keep mf around.
                (*r).seq = (*mf).data;
                (*r).mf = mf;
            }
            (*r).length = sz as i64;
            (*r).is_md5 = 1;
            (*r).validated_md5 = 1;

            // Populate the local disk cache if required.
            if is_local == 0 && !local_cache.is_null() && *local_cache != 0 {
                if cram_cram_io_c_2884_expand_cache_path(path.as_mut_ptr(), local_cache, m5) < 0 {
                    return 0; // Not fatal - we have the data already.
                }
                {
                    let msg = std::ffi::CString::new(format!(
                        "Writing cache file '{}'",
                        CStr::from_ptr(path.as_ptr()).to_string_lossy()
                    ))
                    .unwrap();
                    hts_log_cstr(HTS_LOG_INFO, c"cram_populate_ref".as_ptr(), msg.as_ptr());
                }
                cram_cram_io_c_2947_mkdir_prefix(path.as_mut_ptr(), 0o1777);

                let fp = htslib_hts_open_tmpfile(path.as_ptr(), c"wx".as_ptr(), &mut path_tmp);
                if fp.is_null() {
                    libc::perror(path_tmp.s);
                    free(path_tmp.s.cast());
                    return 0; // Not fatal.
                }

                // Verify md5sum.
                let md5 = htslib_hts_md5_init();
                if md5.is_null() {
                    hclose_abruptly(fp);
                    libc::unlink(path_tmp.s);
                    free(path_tmp.s.cast());
                    return -1;
                }
                let mut md5_buf1 = [0u8; 16];
                let mut md5_buf2 = [0i8; 33];
                htslib_hts_md5_update(md5, (*r).seq.cast(), (*r).length as std::ffi::c_ulong);
                htslib_hts_md5_final(md5_buf1.as_mut_ptr(), md5);
                htslib_hts_md5_destroy(md5);
                htslib_hts_md5_hex(md5_buf2.as_mut_ptr(), md5_buf1.as_ptr());

                if libc::strncmp(m5, md5_buf2.as_ptr(), 32) != 0 {
                    hts_log_cstr(
                        HTS_LOG_ERROR,
                        c"cram_populate_ref".as_ptr(),
                        c"Mismatching md5sum for downloaded reference".as_ptr(),
                    );
                    hclose_abruptly(fp);
                    libc::unlink(path_tmp.s);
                    free(path_tmp.s.cast());
                    return -1;
                }

                let length_written =
                    htslib_hfile_h_292_hwrite(fp, (*r).seq.cast(), (*r).length as usize);
                if hclose(fp) < 0
                    || length_written != (*r).length as isize
                    || libc::chmod(path_tmp.s, 0o444) < 0
                    || libc::rename(path_tmp.s, path.as_ptr()) < 0
                {
                    let msg = std::ffi::CString::new(format!(
                        "Creating reference at {} failed: {}",
                        CStr::from_ptr(path.as_ptr()).to_string_lossy(),
                        CStr::from_ptr(libc::strerror(*__errno_location())).to_string_lossy()
                    ))
                    .unwrap();
                    hts_log_cstr(HTS_LOG_ERROR, c"cram_populate_ref".as_ptr(), msg.as_ptr());
                    libc::unlink(path_tmp.s);
                }
            }

            free(path_tmp.s.cast());
            return 0;
        }
    }

    // no_M5: failed to find in search path or M5 cache; try @SQ UR: tag.
    let ur_tag = crate::htslib_rs::sam::sam_hrecs_find_key(
        ty.cast(),
        c"UR".as_ptr(),
        std::ptr::null_mut(),
    )
    .cast::<sam_hrec_tag_layout>();
    if ur_tag.is_null() {
        return -1;
    }

    let ur = (*ur_tag).str_.add(3);
    if !libc::strstr(ur, c"://".as_ptr()).is_null()
        && libc::strncmp(ur, c"file:".as_ptr(), 5) != 0
    {
        hts_log_cstr(
            HTS_LOG_ERROR,
            c"cram_populate_ref".as_ptr(),
            c"UR tags pointing to remote files are not supported".as_ptr(),
        );
        return -1;
    }

    let fn_ = if libc::strncmp(ur, c"file:".as_ptr(), 5) == 0 {
        (*ur_tag).str_.add(8)
    } else {
        ur
    };

    let refs0 = (*fdl).refs.cast::<refs_t_layout>();
    if !(*refs0).fp.is_null() {
        if bgzf_close((*refs0).fp) != 0 {
            return -1;
        }
        (*refs0).fp = std::ptr::null_mut();
    }

    let refs = cram_cram_io_c_2541_refs_load_fai((*fdl).refs, fn_, 0);
    if refs.is_null() {
        return -1;
    }
    cram_cram_io_c_2693_sanitise_SQ_lines(fd);

    (*fdl).refs = refs.cast();
    let refsl = (*fdl).refs.cast::<refs_t_layout>();
    if !(*refsl).fp.is_null() {
        if bgzf_close((*refsl).fp) != 0 {
            return -1;
        }
        (*refsl).fp = std::ptr::null_mut();
    }

    if (*refsl).fn_.is_null() {
        return -1;
    }

    if cram_cram_io_c_2737_refs2id((*fdl).refs, (*fdl).header.cast()) == -1 {
        return -1;
    }
    if (*refsl).ref_id.is_null() || (*(*refsl).ref_id.add(id as usize)).is_null() {
        return -1;
    }

    // Local copy already, so fall back to cram_get_ref().
    0
}

// original: cram_get_ref (htslib/cram/cram_io.c:3411)
pub unsafe fn cram_cram_io_c_3409_cram_get_ref(
    fd: *mut cram_fd,
    id: c_int,
    mut start: i64,
    mut end: i64,
) -> *mut c_char {
    let fdl = fd.cast::<cram_fd_layout>();
    let ostart = start;

    if id == -1 || start < 1 {
        return std::ptr::null_mut();
    }

    libc::pthread_mutex_lock(&mut (*fdl).ref_lock);

    // Unsorted data implies we want to fetch an entire reference at a time.
    if (*fdl).unsorted != 0 {
        (*fdl).shared_ref = 1;
    }

    // Sanity checking: does this ID exist?
    let refs = (*fdl).refs.cast::<refs_t_layout>();
    if (*fdl).refs.is_null()
        || id < 0
        || id >= (*refs).nref
        || (*(*refs).ref_id.add(id as usize)).is_null()
    {
        let msg =
            std::ffi::CString::new(format!("No reference found for id {}", id)).unwrap();
        hts_log_cstr(HTS_LOG_ERROR, c"cram_get_ref".as_ptr(), msg.as_ptr());
        libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return std::ptr::null_mut();
    }

    let mut r = *(*refs).ref_id.add(id as usize);

    libc::pthread_mutex_lock(&mut (*refs).lock);
    if (*r).length == 0 {
        if !(*fdl).ref_fn.is_null() {
            let msg = std::ffi::CString::new(format!(
                "Reference file given, but ref '{}' not present",
                CStr::from_ptr((*r).name).to_string_lossy()
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"cram_get_ref".as_ptr(), msg.as_ptr());
        }
        if cram_cram_io_c_2977_cram_populate_ref(fd, id, r.cast()) == -1 {
            let msg = std::ffi::CString::new(format!(
                "Failed to populate reference \"{}\"",
                CStr::from_ptr((*r).name).to_string_lossy()
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_WARNING, c"cram_get_ref".as_ptr(), msg.as_ptr());
            hts_log_cstr(
                HTS_LOG_WARNING,
                c"cram_get_ref".as_ptr(),
                c"See https://www.htslib.org/doc/reference_seqs.html for further suggestions"
                    .as_ptr(),
            );
            libc::pthread_mutex_unlock(&mut (*refs).lock);
            libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
            return std::ptr::null_mut();
        }
        // cram_populate_ref may have replaced fd->refs.
        let refs = (*fdl).refs.cast::<refs_t_layout>();
        r = *(*refs).ref_id.add(id as usize);
        if (*fdl).unsorted != 0 {
            cram_cram_io_c_3169_cram_ref_incr_locked((*fdl).refs, id);
        }
    }

    // Re-read refs in case cram_populate_ref reassigned fd->refs.
    let refs = (*fdl).refs.cast::<refs_t_layout>();

    if end < 1 {
        end = (*r).length;
    }
    if end >= (*r).length {
        end = (*r).length;
    }

    if (end - start) as f64 >= 0.5 * (*r).length as f64 || (*fdl).shared_ref != 0 {
        start = 1;
        end = (*r).length;
    }

    if (*fdl).shared_ref != 0 || !(*r).seq.is_null() || (start == 1 && end == (*r).length) {
        let cp: *mut c_char;
        if id >= 0 {
            if !(*r).seq.is_null() {
                cram_cram_io_c_3169_cram_ref_incr_locked((*fdl).refs, id);
            } else {
                let e = cram_cram_io_c_3323_cram_ref_load((*fdl).refs, id, (*r).is_md5);
                if e.is_null() {
                    libc::pthread_mutex_unlock(&mut (*refs).lock);
                    libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
                    return std::ptr::null_mut();
                }
                if (*fdl).unsorted != 0 {
                    cram_cram_io_c_3169_cram_ref_incr_locked((*fdl).refs, id);
                }
            }

            (*fdl).ref_ = std::ptr::null_mut();
            (*fdl).ref_start = 1;
            (*fdl).ref_end = (*r).length;
            (*fdl).ref_id = id;

            cp = (*(*(*refs).ref_id.add(id as usize))).seq.add((ostart - 1) as usize);
        } else {
            (*fdl).ref_ = std::ptr::null_mut();
            cp = std::ptr::null_mut();
        }

        libc::pthread_mutex_unlock(&mut (*refs).lock);
        libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return cp;
    }

    // Not sharing, no cached copy, only a small portion requested.

    // Unmapped ref ID.
    if id < 0 || (*refs).fn_.is_null() {
        if !(*fdl).ref_free.is_null() {
            free((*fdl).ref_free.cast());
            (*fdl).ref_free = std::ptr::null_mut();
        }
        (*fdl).ref_ = std::ptr::null_mut();
        (*fdl).ref_id = id;
        libc::pthread_mutex_unlock(&mut (*refs).lock);
        libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return std::ptr::null_mut();
    }

    // Open file if it's not already the current open reference.
    if libc::strcmp((*refs).fn_, (*r).fn_) != 0 || (*refs).fp.is_null() {
        if !(*refs).fp.is_null() && bgzf_close((*refs).fp) != 0 {
            return std::ptr::null_mut();
        }
        (*refs).fn_ = (*r).fn_;
        (*refs).fp = cram_cram_io_c_2503_bgzf_open_ref((*refs).fn_, c"r".as_ptr().cast_mut(), (*r).is_md5);
        if (*refs).fp.is_null() {
            libc::pthread_mutex_unlock(&mut (*refs).lock);
            libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
            return std::ptr::null_mut();
        }
    }

    let loaded = cram_cram_io_c_3228_load_ref_portion((*refs).fp, r.cast(), start, end);
    if loaded.is_null() {
        libc::pthread_mutex_unlock(&mut (*refs).lock);
        libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);
        return std::ptr::null_mut();
    }
    (*fdl).ref_ = loaded;

    if !(*fdl).ref_free.is_null() {
        free((*fdl).ref_free.cast());
    }

    (*fdl).ref_id = id;
    (*fdl).ref_start = start;
    (*fdl).ref_end = end;
    (*fdl).ref_free = (*fdl).ref_;
    let seq = (*fdl).ref_;

    libc::pthread_mutex_unlock(&mut (*refs).lock);
    libc::pthread_mutex_unlock(&mut (*fdl).ref_lock);

    if seq.is_null() {
        std::ptr::null_mut()
    } else {
        seq.add((ostart - start) as usize)
    }
}

pub unsafe fn cram_cram_io_c_3597_cram_load_reference(
    fd: *mut cram_fd,
    mut fn_: *mut c_char,
) -> c_int {
    let fd = fd.cast::<cram_fd_layout>();
    let mut ret = 0;

    if !fn_.is_null() {
        (*fd).refs = cram_cram_io_c_2541_refs_load_fai(
            (*fd).refs.cast(),
            fn_,
            !((*fd).embed_ref > 0 && (*fd).mode == b'r' as c_int) as c_int,
        )
        .cast();
        fn_ = if !(*fd).refs.is_null() {
            (*(*fd).refs.cast::<refs_t_layout>()).fn_
        } else {
            std::ptr::null_mut()
        };
        if fn_.is_null() {
            ret = -1;
        }
        cram_cram_io_c_2693_sanitise_SQ_lines(fd.cast());
    }
    (*fd).ref_fn = fn_;

    if ((*fd).refs.is_null() || ((*(*fd).refs.cast::<refs_t_layout>()).nref == 0 && fn_.is_null()))
        && !(*fd).header.is_null()
    {
        if !(*fd).refs.is_null() {
            cram_cram_io_c_2427_refs_free((*fd).refs.cast());
        }
        (*fd).refs = cram_cram_io_c_2467_refs_create().cast();
        if (*fd).refs.is_null() {
            return -1;
        }
        if cram_cram_io_c_2768_refs_from_header(fd.cast()) == -1 {
            return -1;
        }
    }

    if !(*fd).header.is_null()
        && cram_cram_io_c_2737_refs2id((*fd).refs.cast(), (*fd).header.cast()) == -1
    {
        return -1;
    }

    ret
}

pub unsafe fn cram_cram_io_c_1490_cram_block_size(b: *mut hts_sys::cram_block) -> u32 {
    let b = b.cast::<cram_block_layout>();
    let itf8_len = |v: i64| -> u32 {
        if (v & !0x7f) == 0 {
            1
        } else if (v & !0x3fff) == 0 {
            2
        } else if (v & !0x1f_ffff) == 0 {
            3
        } else if (v & !0xfff_ffff) == 0 {
            4
        } else {
            5
        }
    };

    let header = 2
        + itf8_len((*b).content_id as i64)
        + itf8_len((*b).comp_size as i64)
        + itf8_len((*b).uncomp_size as i64)
        + 4;
    let payload = if (*b).method == crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW {
        (*b).uncomp_size
    } else {
        (*b).comp_size
    };
    header + payload as u32
}

pub unsafe fn cram_cram_io_c_4330_cram_new_compression_header(
) -> *mut hts_sys::cram_block_compression_hdr {
    let hdr = calloc(
        1,
        std::mem::size_of::<cram_block_compression_hdr_layout>() as u64,
    )
    .cast::<cram_block_compression_hdr_layout>();
    if hdr.is_null() {
        return std::ptr::null_mut();
    }

    (*hdr).td_blk = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE, 0);
    if (*hdr).td_blk.is_null() {
        free(hdr.cast());
        return std::ptr::null_mut();
    }

    (*hdr).td_hash = calloc(1, std::mem::size_of::<kh_generic_layout>() as u64).cast::<c_void>();
    if (*hdr).td_hash.is_null() {
        cram_cram_io_c_1565_cram_free_block((*hdr).td_blk);
        free(hdr.cast());
        return std::ptr::null_mut();
    }

    (*hdr).td_keys = cram_string_alloc_c_55_string_pool_create(8192).cast();
    if (*hdr).td_keys.is_null() {
        let h = (*hdr).td_hash.cast::<kh_generic_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
        cram_cram_io_c_1565_cram_free_block((*hdr).td_blk);
        free(hdr.cast());
        return std::ptr::null_mut();
    }

    hdr.cast()
}

pub unsafe fn cram_cram_io_c_4356_cram_free_compression_header(
    hdr: *mut hts_sys::cram_block_compression_hdr,
) {
    let hdr = hdr.cast::<cram_block_compression_hdr_layout>();
    if hdr.is_null() {
        return;
    }

    if !(*hdr).landmark.is_null() {
        free((*hdr).landmark.cast());
    }

    if !(*hdr).preservation_map.is_null() {
        let h = (*hdr).preservation_map.cast::<kh_generic_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }

    for i in 0..32usize {
        let mut m = (*hdr).rec_encoding_map[i].cast::<cram_map_layout>();
        while !m.is_null() {
            let m2 = (*m).next;
            if !(*m).codec.is_null() {
                let c = (*m).codec.cast::<cram_codec_base_layout>();
                if let Some(free_fn) = (*c).free {
                    free_fn(c);
                }
            }
            free(m.cast());
            m = m2;
        }
    }

    for i in 0..32usize {
        let mut m = (*hdr).tag_encoding_map[i].cast::<cram_map_layout>();
        while !m.is_null() {
            let m2 = (*m).next;
            if !(*m).codec.is_null() {
                let c = (*m).codec.cast::<cram_codec_base_layout>();
                if let Some(free_fn) = (*c).free {
                    free_fn(c);
                }
            }
            free(m.cast());
            m = m2;
        }
    }

    for i in 0..CRAM_DS_END {
        let c = (*hdr).codecs[i].cast::<cram_codec_base_layout>();
        if !c.is_null() {
            if let Some(free_fn) = (*c).free {
                free_fn(c);
            }
        }
    }

    if !(*hdr).tl.is_null() {
        free((*hdr).tl.cast());
    }
    if !(*hdr).td_blk.is_null() {
        cram_cram_io_c_1565_cram_free_block((*hdr).td_blk);
    }
    if !(*hdr).td_hash.is_null() {
        let h = (*hdr).td_hash.cast::<kh_generic_layout>();
        free((*h).flags.cast());
        free((*h).keys.cast());
        free((*h).vals.cast());
        free(h.cast());
    }
    if !(*hdr).td_keys.is_null() {
        cram_string_alloc_c_103_string_pool_destroy((*hdr).td_keys.cast());
    }

    free(hdr.cast());
}

pub unsafe fn cram_cram_io_c_4660_cram_read_file_def(
    fd: *mut cram_fd,
) -> *mut cram_file_def_layout {
    let def =
        malloc(std::mem::size_of::<cram_file_def_layout>() as u64).cast::<cram_file_def_layout>();
    if def.is_null() {
        return std::ptr::null_mut();
    }

    let fd_layout = fd.cast::<cram_fd_layout>();
    if htslib_hfile_h_247_hread(
        (*fd_layout).fp,
        &mut (*def).magic[0] as *mut c_char as *mut c_void,
        26,
    ) != 26
    {
        free(def.cast());
        return std::ptr::null_mut();
    }

    if libc::memcmp((*def).magic.as_ptr().cast(), c"CRAM".as_ptr().cast(), 4) != 0 {
        free(def.cast());
        return std::ptr::null_mut();
    }

    if (*def).major_version > 4 {
        free(def.cast());
        return std::ptr::null_mut();
    }

    (*fd_layout).first_container += 26;
    (*fd_layout).curr_position = (*fd_layout).first_container;
    (*fd_layout).last_slice = 0;

    def
}

pub unsafe fn cram_cram_io_c_4694_cram_write_file_def(
    fd: *mut cram_fd,
    def: *mut cram_file_def_layout,
) -> c_int {
    let fd_layout = fd.cast::<cram_fd_layout>();
    if htslib_hfile_h_292_hwrite(
        (*fd_layout).fp,
        &(*def).magic[0] as *const c_char as *const c_void,
        26,
    ) == 26
    {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_io_c_4698_cram_free_file_def(def: *mut cram_file_def_layout) {
    if !def.is_null() {
        free(def.cast());
    }
}

pub unsafe fn cram_cram_io_c_1565_cram_free_block(b: *mut hts_sys::cram_block) {
    if b.is_null() {
        return;
    }
    let b = b.cast::<cram_block_layout>();
    if !(*b).data.is_null() {
        free((*b).data.cast());
    }
    free(b.cast());
}

pub unsafe fn cram_cram_codecs_c_932_cram_const_decode_byte(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    if out.is_null() {
        return 0;
    }
    let c = c.cast::<cram_codec_const_layout>();
    for i in 0..*out_size {
        *out.add(i as usize) = (*c).xconst.val as c_char;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_945_cram_const_decode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    let out_i = out.cast::<i32>();
    for i in 0..*out_size {
        *out_i.add(i as usize) = (*c).xconst.val as i32;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_956_cram_const_decode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    let out_i = out.cast::<i64>();
    for i in 0..*out_size {
        *out_i.add(i as usize) = (*c).xconst.val;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_967_cram_const_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub fn cram_cram_codecs_c_972_cram_const_decode_size(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
) -> c_int {
    0
}

pub unsafe fn cram_cram_codecs_c_976_cram_const_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    if kputsn(c"CONST(val=".as_ptr(), 10, ks) < 0
        || kputll((*c).xconst.val, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_981_cram_const_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_const_layout>() as u64)
        .cast::<cram_codec_const_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = codec;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).decode = if codec == 43 && option == 3 {
        cram_cram_codecs_c_932_cram_const_decode_byte as usize as *mut c_void
    } else if codec == 44 && (option == 1 || option == 6) {
        cram_cram_codecs_c_945_cram_const_decode_int as usize as *mut c_void
    } else if codec == 44 && (option == 2 || option == 7) {
        cram_cram_codecs_c_956_cram_const_decode_long as usize as *mut c_void
    } else {
        free(c.cast());
        return std::ptr::null_mut();
    };
    (*c).free = cram_cram_codecs_c_967_cram_const_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = cram_cram_codecs_c_972_cram_const_decode_size as usize as *mut c_void;
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = cram_cram_codecs_c_976_cram_const_describe as usize as *mut c_void;

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).xconst.val =
        ((*vv).varint_get64s.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut());
    if cp.offset_from(data) != size as isize {
        free(c.cast());
        return std::ptr::null_mut();
    }
    c.cast()
}

pub fn cram_cram_codecs_c_1020_cram_const_encode(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub unsafe fn cram_cram_codecs_c_1025_cram_const_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_const_layout>();
    let mut tmp = [0 as c_char; 99];
    let mut tp = tmp.as_mut_ptr();
    let mut len = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    tp = tp.add(
        ((*(*c).vv).varint_put64s.unwrap())(tp, std::ptr::null_mut(), (*c).xconst.val) as usize,
    );
    len += ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += ((*(*c).vv).varint_put32_blk.unwrap())(b, tp.offset_from(tmp.as_ptr()) as i32);
    let nbytes = tp.offset_from(tmp.as_ptr()) as usize;
    if cram_cram_io_h_248_block_append(b, tmp.as_ptr().cast(), nbytes) != 0 {
        return -1;
    }
    len + nbytes as c_int
}

pub unsafe fn cram_cram_codecs_c_1048_cram_const_encode_init(
    st: *mut c_void,
    codec: c_int,
    _option: c_int,
    _dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_const_layout>() as u64)
        .cast::<cram_codec_const_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = codec;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_967_cram_const_decode_free as usize as *mut c_void;
    (*c).decode = std::ptr::null_mut();
    (*c).encode = cram_cram_codecs_c_1020_cram_const_encode as usize as *mut c_void;
    (*c).store = cram_cram_codecs_c_1025_cram_const_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();
    (*c).xconst.val = (*(st.cast::<cram_stats_layout>())).min_val;
    c.cast()
}

pub unsafe fn cram_cram_codecs_h_230_cram_not_enough_bits(
    blk: *mut hts_sys::cram_block,
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

pub unsafe fn cram_cram_codecs_c_1072_cram_beta_decode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let out_i = out.cast::<i64>();
    let n = *out_size;
    if (*c).beta.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).beta.nbits * n) != 0 {
            return -1;
        }
        for i in 0..n {
            *out_i.add(i as usize) =
                cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits) - (*c).beta.offset as i64;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = -((*c).beta.offset as i64);
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1090_cram_beta_decode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let out_i = out.cast::<i32>();
    let n = *out_size;
    if (*c).beta.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).beta.nbits * n) != 0 {
            return -1;
        }
        for i in 0..n {
            *out_i.add(i as usize) =
                cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits) as i32 - (*c).beta.offset;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = -(*c).beta.offset;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1108_cram_beta_decode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let n = *out_size;
    if (*c).beta.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).beta.nbits * n) != 0 {
            return -1;
        }
        if !out.is_null() {
            for i in 0..n {
                *out.add(i as usize) = (cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits)
                    as i32
                    - (*c).beta.offset) as c_char;
            }
        } else {
            for _ in 0..n {
                cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).beta.nbits);
            }
        }
    } else if !out.is_null() {
        for i in 0..n {
            *out.add(i as usize) = (-(*c).beta.offset) as c_char;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1131_cram_beta_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_1136_cram_beta_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    if kputsn(c"BETA(offset=".as_ptr(), 12, ks) < 0
        || kputw((*c).beta.offset, ks) < 0
        || kputsn(c", nbits=".as_ptr(), 8, ks) < 0
        || kputw((*c).beta.nbits, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_1142_cram_beta_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_beta_layout>() as u64)
        .cast::<cram_codec_beta_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 6;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).decode = if option == 1 || option == 6 {
        cram_cram_codecs_c_1090_cram_beta_decode_int as usize as *mut c_void
    } else if option == 2 || option == 7 {
        cram_cram_codecs_c_1072_cram_beta_decode_long as usize as *mut c_void
    } else if option == 4 || option == 3 {
        cram_cram_codecs_c_1108_cram_beta_decode_char as usize as *mut c_void
    } else {
        free(c.cast());
        return std::ptr::null_mut();
    };
    (*c).free = cram_cram_codecs_c_1131_cram_beta_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = cram_cram_codecs_c_1136_cram_beta_describe as usize as *mut c_void;

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).beta.nbits = -1;
    (*c).beta.offset =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp < endp {
        (*c).beta.nbits =
            ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    }
    if cp.offset_from(data) != size as isize || (*c).beta.nbits < 0 || (*c).beta.nbits > 32 {
        free(c.cast());
        return std::ptr::null_mut();
    }
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_1183_cram_beta_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let mut len = 0;
    let mut r = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let body_len = ((*(*c).vv).varint_size.unwrap())((*c).beta.offset as i64)
        + ((*(*c).vv).varint_size.unwrap())((*c).beta.nbits as i64);
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, body_len);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).beta.offset);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).beta.nbits);
    len += n;
    r |= n;

    if r > 0 {
        len
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1207_cram_beta_encode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let syms = in_.cast::<i64>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out,
            (*syms.add(i as usize) + (*c).beta.offset as i64) as u64,
            (*c).beta.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1219_cram_beta_encode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let syms = in_.cast::<c_int>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out,
            (*syms.add(i as usize) + (*c).beta.offset) as u64,
            (*c).beta.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1231_cram_beta_encode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_beta_layout>();
    let syms = in_.cast::<u8>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out,
            (*syms.add(i as usize) as i32 + (*c).beta.offset) as u64,
            (*c).beta.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1243_cram_beta_encode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_1247_cram_beta_encode_init(
    st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_beta_layout>() as u64)
        .cast::<cram_codec_beta_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 6;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_1243_cram_beta_encode_free as usize as *mut c_void;
    (*c).encode = if option == 1 || option == 6 {
        cram_cram_codecs_c_1219_cram_beta_encode_int as usize as *mut c_void
    } else if option == 2 || option == 7 {
        cram_cram_codecs_c_1207_cram_beta_encode_long as usize as *mut c_void
    } else {
        cram_cram_codecs_c_1231_cram_beta_encode_char as usize as *mut c_void
    };
    (*c).store = cram_cram_codecs_c_1183_cram_beta_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();

    let (min_val, max_val) = if !dat.is_null() {
        let dat = dat.cast::<i64>();
        (*dat, *dat.add(1))
    } else {
        let st = st.cast::<cram_stats_layout>();
        let mut min_val = i32::MAX as i64;
        let mut max_val = i32::MIN as i64;
        for i in 0..1024usize {
            if (*st).freqs[i] == 0 {
                continue;
            }
            if min_val > i as i64 {
                min_val = i as i64;
            }
            max_val = i as i64;
        }
        if !(*st).h.is_null() {
            let h = (*st).h.cast::<kh_m_i2i_layout>();
            for k in 0..(*h).n_buckets {
                let flag = *(*h).flags.add((k >> 4) as usize);
                if ((flag >> ((k & 0xf) << 1)) & 3) != 0 {
                    continue;
                }
                let i = *(*h).keys.add(k as usize);
                if min_val > i {
                    min_val = i;
                }
                if max_val < i {
                    max_val = i;
                }
            }
        }
        (min_val, max_val)
    };

    if max_val < min_val {
        free(c.cast());
        return std::ptr::null_mut();
    }

    let mut range = max_val - min_val;
    match option {
        6 => {
            if min_val < i32::MIN as i64 || range > i32::MAX as i64 {
                free(c.cast());
                return std::ptr::null_mut();
            }
        }
        1 => {
            if max_val > u32::MAX as i64 || range > u32::MAX as i64 {
                free(c.cast());
                return std::ptr::null_mut();
            }
        }
        _ => {}
    }

    (*c).beta.offset = (-min_val) as i32;
    let mut len = 0;
    while range != 0 {
        len += 1;
        range >>= 1;
    }
    (*c).beta.nbits = len;

    c.cast()
}

pub unsafe fn cram_cram_codecs_c_1344_cram_xpack_decode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let out_i = out.cast::<i64>();
    let n = *out_size;
    if (*c).xpack.nbits != 0 {
        for i in 0..n {
            let idx = cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).xpack.nbits) as usize;
            *out_i.add(i as usize) = (*c).xpack.rmap[idx] as i64;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = (*c).xpack.rmap[0] as i64;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1359_cram_xpack_decode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let out_i = out.cast::<i32>();
    let n = *out_size;
    if (*c).xpack.nbits != 0 {
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, (*c).xpack.nbits * n) != 0 {
            return -1;
        }
        for i in 0..n {
            let idx = cram_cram_codecs_c_169_get_bits_MSB(in_, (*c).xpack.nbits) as usize;
            *out_i.add(i as usize) = (*c).xpack.rmap[idx] as i32;
        }
    } else {
        for i in 0..n {
            *out_i.add(i as usize) = (*c).xpack.rmap[0] as i32;
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let slot = (512 + (*c_xpack).codec_id) as usize;
    let cached = *(*slice_layout).block_by_id.add(slot);
    if !cached.is_null() {
        return 0;
    }

    let sub_codec = (*c_xpack).xpack.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xpack_layout>();
    let get_block: CramCodecGetBlockFn = std::mem::transmute((*sub_layout).get_block);
    let sub_b = get_block(slice, sub_codec);
    if sub_b.is_null() || (*c_xpack).xpack.nbits == 0 {
        return -1;
    }

    let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if b.is_null() {
        return -1;
    }
    *(*slice_layout).block_by_id.add(slot) = b;
    let sub = sub_b.cast::<cram_block_layout>();
    let out_n = (*sub).uncomp_size * 8 / (*c_xpack).xpack.nbits;
    if cram_cram_io_h_243_block_grow(b, out_n as usize) < 0 {
        return -1;
    }
    let out = b.cast::<cram_block_layout>();
    (*out).uncomp_size = out_n;

    let nsym = 8 / (*c_xpack).xpack.nbits;
    let mut i = 0usize;
    let mut j = 0usize;
    while i < out_n as usize {
        let mut byte = *(*sub).data.add(j);
        j += 1;
        match nsym {
            8 => {
                for _ in 0..8 {
                    if i >= out_n as usize {
                        break;
                    }
                    *(*out).data.add(i) = (*c_xpack).xpack.rmap[(byte & 1) as usize] as u8;
                    byte >>= 1;
                    i += 1;
                }
            }
            4 => {
                for _ in 0..4 {
                    if i >= out_n as usize {
                        break;
                    }
                    *(*out).data.add(i) = (*c_xpack).xpack.rmap[(byte & 3) as usize] as u8;
                    byte >>= 2;
                    i += 1;
                }
            }
            2 => {
                for _ in 0..2 {
                    if i >= out_n as usize {
                        break;
                    }
                    *(*out).data.add(i) = (*c_xpack).xpack.rmap[(byte & 15) as usize] as u8;
                    byte >>= 4;
                    i += 1;
                }
            }
            1 => {
                *(*out).data.add(i) = byte;
                i += 1;
            }
            _ => return -1,
        }
    }

    0
}

pub unsafe fn cram_cram_codecs_c_1408_cram_xpack_decode_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    if (*c_xpack).xpack.nval > 1 {
        cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(slice, c);
        let slice_layout = slice.cast::<cram_slice_layout>();
        let b = *(*slice_layout)
            .block_by_id
            .add((512 + (*c_xpack).codec_id) as usize);
        if b.is_null() {
            return -1;
        }
        let block = b.cast::<cram_block_layout>();
        if !out.is_null() {
            memcpy(
                out.cast(),
                (*block).data.add((*block).byte).cast(),
                *out_size as u64,
            );
        }
        (*block).byte += *out_size as usize;
    } else if !out.is_null() {
        std::ptr::write_bytes(out, (*c_xpack).xpack.rmap[0] as u8, *out_size as usize);
    }

    0
}

pub unsafe fn cram_cram_codecs_c_1431_cram_xpack_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    if !(*c_xpack).xpack.sub_codec.is_null() {
        let sub = (*c_xpack).xpack.sub_codec.cast::<cram_codec_xpack_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*sub).free);
            free_fn((*c_xpack).xpack.sub_codec);
        }
    }
    free(c);
}

pub unsafe fn cram_cram_codecs_c_1443_cram_xpack_decode_size(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(slice, c);
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let slice_layout = slice.cast::<cram_slice_layout>();
    let b = *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xpack).codec_id) as usize);
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_1448_cram_xpack_get_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> *mut hts_sys::cram_block {
    cram_cram_codecs_c_1377_cram_xpack_decode_expand_char(slice, c);
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let slice_layout = slice.cast::<cram_slice_layout>();
    *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xpack).codec_id) as usize)
}

pub unsafe fn cram_cram_codecs_c_1453_cram_xpack_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = calloc(1, std::mem::size_of::<cram_codec_xpack_layout>() as u64)
        .cast::<cram_codec_xpack_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }

    (*c).codec = 51;
    (*c).decode = if option == 2 {
        cram_cram_codecs_c_1344_cram_xpack_decode_long as usize as *mut c_void
    } else if option == 1 {
        cram_cram_codecs_c_1359_cram_xpack_decode_int as usize as *mut c_void
    } else if option == 4 || option == 3 {
        cram_cram_codecs_c_1408_cram_xpack_decode_char as usize as *mut c_void
    } else {
        cram_cram_codecs_c_1431_cram_xpack_decode_free(c.cast());
        return std::ptr::null_mut();
    };
    (*c).free = cram_cram_codecs_c_1431_cram_xpack_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = cram_cram_codecs_c_1443_cram_xpack_decode_size as usize as *mut c_void;
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = cram_cram_codecs_c_1448_cram_xpack_get_block as usize as *mut c_void;
    (*c).describe = std::ptr::null_mut();

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).xpack.nbits =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as i32;
    (*c).xpack.nval =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as i32;
    if (*c).xpack.nbits >= 8 || (*c).xpack.nbits < 0 || (*c).xpack.nval > 256 || (*c).xpack.nval < 0
    {
        cram_cram_codecs_c_1431_cram_xpack_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    for i in 0..(*c).xpack.nval {
        let v =
            ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
                as u32;
        if v >= 256 {
            cram_cram_codecs_c_1431_cram_xpack_decode_free(c.cast());
            return std::ptr::null_mut();
        }
        (*c).xpack.rmap[i as usize] = v;
    }

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        cram_cram_codecs_c_1431_cram_xpack_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).xpack.sub_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, option, version, vv);
    if (*c).xpack.sub_codec.is_null() {
        cram_cram_codecs_c_1431_cram_xpack_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    if cp.offset_from(data) != size as isize
        || (*c).xpack.nbits < 0
        || (*c).xpack.nbits > (8 * std::mem::size_of::<i64>()) as i32
    {
        cram_cram_codecs_c_1431_cram_xpack_decode_free(c.cast());
        return std::ptr::null_mut();
    }

    c.cast()
}

pub unsafe fn cram_cram_codecs_c_1581_cram_xpack_encode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let syms = in_.cast::<i64>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out,
            (*c).xpack.map[*syms.add(i as usize) as usize] as u64,
            (*c).xpack.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1592_cram_xpack_encode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let syms = in_.cast::<c_int>();
    let mut r = 0;
    for i in 0..in_size {
        r |= cram_cram_codecs_c_259_store_bits_MSB(
            (*c).out,
            (*c).xpack.map[*syms.add(i as usize) as usize] as u64,
            (*c).xpack.nbits,
        );
    }
    r
}

pub unsafe fn cram_cram_codecs_c_1603_cram_xpack_encode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    if cram_cram_io_h_248_block_append((*c).out, in_.cast(), in_size as usize) == 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1612_cram_xpack_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    if !(*c_xpack).xpack.sub_codec.is_null() {
        let sub = (*c_xpack).xpack.sub_codec.cast::<cram_codec_xpack_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*sub).free);
            free_fn((*c_xpack).xpack.sub_codec);
        }
    }
    cram_cram_io_c_1565_cram_free_block((*c_xpack).out);
    free(c);
}

pub unsafe extern "C" fn cram_cram_codecs_c_1515_cram_xpack_encode_flush(c: *mut c_void) -> c_int {
    let c_xpack = c.cast::<cram_codec_xpack_layout>();
    let out_block = (*c_xpack).out.cast::<cram_block_layout>();
    let mut meta_len = 0;
    let mut out_len = 0u64;
    let mut out_meta = [0u8; 1024];
    let out = htscodecs_hts_pack(
        (*out_block).data,
        (*out_block).byte as i64,
        out_meta.as_mut_ptr(),
        &mut meta_len,
        &mut out_len,
    );
    let sub_codec = (*c_xpack).xpack.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xpack_layout>();
    let encode: CramCodecEncodeFn = std::mem::transmute((*sub_layout).encode);
    if encode(
        std::ptr::null_mut(),
        sub_codec,
        out.cast(),
        out_len as c_int,
    ) != 0
    {
        return -1;
    }

    let mut r = 0;
    if !(*sub_layout).flush.is_null() {
        let flush: CramCodecFlushFn = std::mem::transmute((*sub_layout).flush);
        r = flush(sub_codec);
    }

    free(out.cast());
    r
}

pub unsafe fn cram_cram_codecs_c_1537_cram_xpack_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xpack_layout>();
    let mut len = 0;
    let mut r = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    let tc = (*c).xpack.sub_codec;
    let tb = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if tb.is_null() {
        return -1;
    }
    let store: CramCodecStoreFn =
        std::mem::transmute((*(tc.cast::<cram_codec_xpack_layout>())).store);
    let len2 = store(tc, tb, std::ptr::null_mut(), version);

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;

    let mut len1 = 0;
    for i in 0..(*c).xpack.nval {
        let n = ((*(*c).vv).varint_size.unwrap())((*c).xpack.rmap[i as usize] as i64);
        len1 += n;
        r |= n;
    }
    let body_len = ((*(*c).vv).varint_size.unwrap())((*c).xpack.nbits as i64)
        + ((*(*c).vv).varint_size.unwrap())((*c).xpack.nval as i64)
        + len1
        + len2;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, body_len);
    len += n;
    r |= n;

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xpack.nbits);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xpack.nval);
    len += n;
    r |= n;
    for i in 0..(*c).xpack.nval {
        let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xpack.rmap[i as usize] as i32);
        len += n;
        r |= n;
    }

    if cram_cram_io_h_248_block_append(
        b,
        (*(tb.cast::<cram_block_layout>())).data.cast(),
        (*(tb.cast::<cram_block_layout>())).byte,
    ) != 0
    {
        cram_cram_io_c_1565_cram_free_block(tb);
        return -1;
    }
    cram_cram_io_c_1565_cram_free_block(tb);

    if r > 0 {
        len + len2
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_1623_cram_xpack_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_xpack_layout>() as u64)
        .cast::<cram_codec_xpack_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 51;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_1612_cram_xpack_encode_free as usize as *mut c_void;
    (*c).decode = std::ptr::null_mut();
    (*c).encode = if option == 2 {
        cram_cram_codecs_c_1581_cram_xpack_encode_long as usize as *mut c_void
    } else if option == 1 {
        cram_cram_codecs_c_1592_cram_xpack_encode_int as usize as *mut c_void
    } else {
        cram_cram_codecs_c_1603_cram_xpack_encode_char as usize as *mut c_void
    };
    (*c).store = cram_cram_codecs_c_1537_cram_xpack_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = cram_cram_codecs_c_1515_cram_xpack_encode_flush as usize as *mut c_void;
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();

    let e = dat.cast::<cram_xpack_decoder_layout>();
    (*c).xpack.nbits = (*e).nbits;
    (*c).xpack.nval = (*e).nval;
    (*c).xpack.sub_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).sub_encoding,
        std::ptr::null_mut(),
        4,
        (*e).sub_codec_dat,
        version,
        vv,
    );
    memcpy(
        (*c).xpack.map.as_mut_ptr().cast(),
        (*e).map.as_ptr().cast(),
        std::mem::size_of_val(&(*e).map) as u64,
    );
    let mut n = 0;
    for i in 0..256usize {
        if (*e).map[i] != -1 {
            (*c).xpack.rmap[n as usize] = i as u32;
            n += 1;
        }
    }
    if n != (*e).nval {
        return std::ptr::null_mut();
    }

    c.cast()
}

pub fn cram_cram_codecs_c_1676_zigzag8(x: i8) -> u8 {
    ((x.wrapping_shl(1)) ^ (x >> 7)) as u8
}

pub fn cram_cram_codecs_c_1677_zigzag16(x: i16) -> u16 {
    ((x.wrapping_shl(1)) ^ (x >> 15)) as u16
}

pub fn cram_cram_codecs_c_1678_zigzag32(x: i32) -> u32 {
    ((x.wrapping_shl(1)) ^ (x >> 31)) as u32
}

pub fn cram_cram_codecs_c_1681_unzigzag16(x: u16) -> i16 {
    (((x >> 1) as i32) ^ -((x & 1) as i32)) as i16
}

pub fn cram_cram_codecs_c_1682_unzigzag32(x: u32) -> i32 {
    ((x >> 1) ^ 0u32.wrapping_sub(x & 1)) as i32
}

pub fn cram_cram_codecs_c_1684_cram_xdelta_decode_long(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub unsafe fn cram_cram_codecs_c_1688_cram_xdelta_decode_int(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let out32 = out.cast::<u32>();
    for i in 0..*out_size {
        let mut v = 0u32;
        let mut one = 1;
        let sub = (*c).xdelta.sub_codec;
        let sub_codec = sub.cast::<cram_codec_xdelta_layout>();
        let decode_fn: CramCodecDecodeFn = std::mem::transmute((*sub_codec).decode);
        if decode_fn(slice, sub, in_, (&mut v as *mut u32).cast(), &mut one) < 0 {
            return -1;
        }
        let d = cram_cram_codecs_c_1682_unzigzag32(v) as u32;
        (*c).xdelta.last = d.wrapping_add((*c).xdelta.last as u32) as i64;
        *out32.add(i as usize) = (*c).xdelta.last as u32;
    }
    0
}

pub fn cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1709_cram_xdelta_decode_char(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1713_le_int2(i: i16) -> i16 {
    i16::from_ne_bytes(i.to_le_bytes())
}

pub unsafe fn cram_cram_codecs_c_1719_cram_xdelta_decode_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out_: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let out = out_.cast::<hts_sys::cram_block>();
    let sub = (*c).xdelta.sub_codec;
    let sub_codec = sub.cast::<cram_codec_xdelta_layout>();
    let get_block_fn: CramCodecGetBlockFn = std::mem::transmute((*sub_codec).get_block);
    let b = get_block_fn(slice, sub);
    let w = (*c).xdelta.word_size as c_int;
    let mut npad = (w - *out_size % w) % w;
    let out_sz = *out_size + npad;
    (*c).xdelta.last = 0;

    let mut i = 0;
    while i < out_sz {
        let block = b.cast::<cram_block_layout>();
        let mut cp = (*block).data.add((*block).byte).cast::<c_char>();
        let cp_end = (*block)
            .data
            .add((*block).uncomp_size as usize)
            .cast::<c_char>();
        let mut err = 0;
        let v = ((*(*c).vv).varint_get32.unwrap())(&mut cp, cp_end, &mut err) as u16;
        if err != 0 {
            return -1;
        }
        (*block).byte = cp.offset_from((*block).data.cast::<c_char>()) as usize;

        match w {
            2 => {
                let d = cram_cram_codecs_c_1681_unzigzag16(v) as i64;
                (*c).xdelta.last = d + (*c).xdelta.last;
                let z = cram_cram_codecs_c_1713_le_int2((*c).xdelta.last as i16);
                if cram_cram_io_h_248_block_append(
                    out,
                    (&z as *const i16).cast(),
                    (2 - npad) as usize,
                ) != 0
                {
                    return -1;
                }
                npad = 0;
            }
            _ => return -1,
        }
        i += w;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_1762_cram_xdelta_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xdelta = c.cast::<cram_codec_xdelta_layout>();
    if !(*c_xdelta).xdelta.sub_codec.is_null() {
        let sub = (*c_xdelta)
            .xdelta
            .sub_codec
            .cast::<cram_codec_xdelta_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*sub).free);
            free_fn((*c_xdelta).xdelta.sub_codec);
        }
    }
    free(c);
}

pub unsafe fn cram_cram_codecs_c_1771_cram_xdelta_decode_size(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(slice, c);
    let slice = slice.cast::<cram_slice_layout>();
    let c = c.cast::<cram_codec_xdelta_layout>();
    let b = *(*slice).block_by_id.add((512 + (*c).codec_id) as usize);
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe fn cram_cram_codecs_c_1776_cram_xdelta_get_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> *mut hts_sys::cram_block {
    cram_cram_codecs_c_1705_cram_xdelta_decode_expand_char(slice, c);
    let slice = slice.cast::<cram_slice_layout>();
    let c = c.cast::<cram_codec_xdelta_layout>();
    *(*slice).block_by_id.add((512 + (*c).codec_id) as usize)
}

pub unsafe fn cram_cram_codecs_c_1781_cram_xdelta_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    mut option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = calloc(1, std::mem::size_of::<cram_codec_xdelta_layout>() as u64)
        .cast::<cram_codec_xdelta_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 53;
    (*c).decode = if option == 2 {
        cram_cram_codecs_c_1684_cram_xdelta_decode_long as usize as *mut c_void
    } else if option == 1 {
        cram_cram_codecs_c_1688_cram_xdelta_decode_int as usize as *mut c_void
    } else if option == 4 || option == 3 {
        cram_cram_codecs_c_1709_cram_xdelta_decode_char as usize as *mut c_void
    } else if option == 5 {
        option = 4;
        cram_cram_codecs_c_1719_cram_xdelta_decode_block as usize as *mut c_void
    } else {
        free(c.cast());
        return std::ptr::null_mut();
    };
    (*c).free = cram_cram_codecs_c_1762_cram_xdelta_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = cram_cram_codecs_c_1771_cram_xdelta_decode_size as usize as *mut c_void;
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = cram_cram_codecs_c_1776_cram_xdelta_get_block as usize as *mut c_void;
    (*c).describe = std::ptr::null_mut();

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).xdelta.word_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as u8;
    (*c).xdelta.last = 0;

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        cram_cram_codecs_c_1762_cram_xdelta_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).xdelta.sub_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, option, version, vv);
    if (*c).xdelta.sub_codec.is_null() {
        cram_cram_codecs_c_1762_cram_xdelta_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    if cp.offset_from(data) != size as isize {
        cram_cram_codecs_c_1762_cram_xdelta_decode_free(c.cast());
        return std::ptr::null_mut();
    }

    c.cast()
}

pub unsafe extern "C" fn cram_cram_codecs_c_1835_cram_xdelta_encode_flush(c: *mut c_void) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if b.is_null() {
        return -1;
    }
    let out = (*c).out.cast::<cram_block_layout>();
    let mut r = -1;

    match (*c).xdelta.word_size {
        2 => {
            let n = (*out).byte / 2;
            let mut dat = (*out).data.cast::<u8>();
            let mut last = 0u16;
            if n * 2 < (*out).byte {
                last = *dat as u16;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1677_zigzag16(last as i16) as i32,
                );
                dat = dat.add(1);
            }
            let dat16 = dat.cast::<u16>();
            for i in 0..n {
                let v = std::ptr::read_unaligned(dat16.add(i));
                let d = v.wrapping_sub(last);
                last = v;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1677_zigzag16(d as i16) as i32,
                );
            }
        }
        4 => {
            let n = (*out).byte / 4;
            let dat = (*out).data.cast::<u32>();
            let mut last = 0u32;
            for i in 0..n {
                let v = std::ptr::read_unaligned(dat.add(i));
                let d = v.wrapping_sub(last);
                last = v;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1678_zigzag32(d as i32) as i32,
                );
            }
        }
        1 => {
            let n = (*out).byte;
            let dat = (*out).data;
            let mut last = 0u8;
            for i in 0..n {
                let v = *dat.add(i);
                let d = v.wrapping_sub(last);
                last = v;
                ((*(*c).vv).varint_put32_blk.unwrap())(
                    b,
                    cram_cram_codecs_c_1676_zigzag8(d as i8) as i32,
                );
            }
        }
        _ => {
            cram_cram_io_c_1565_cram_free_block(b);
            return -1;
        }
    }

    let sub_codec = (*c).xdelta.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xdelta_layout>();
    let encode: CramCodecEncodeFn = std::mem::transmute((*sub_layout).encode);
    let b_layout = b.cast::<cram_block_layout>();
    if encode(
        std::ptr::null_mut(),
        sub_codec,
        (*b_layout).data.cast(),
        (*b_layout).byte as c_int,
    ) == 0
    {
        r = 0;
    }

    cram_cram_io_c_1565_cram_free_block(b);
    r
}

pub unsafe fn cram_cram_codecs_c_1930_cram_xdelta_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let mut len = 0;
    let mut r = 0;
    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    let tc = (*c).xdelta.sub_codec;
    let tb = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if tb.is_null() {
        return -1;
    }
    let store: CramCodecStoreFn =
        std::mem::transmute((*(tc.cast::<cram_codec_xdelta_layout>())).store);
    let len2 = store(tc, tb, std::ptr::null_mut(), version);

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(
        b,
        ((*(*c).vv).varint_size.unwrap())((*c).xdelta.word_size as i64) + len2,
    );
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).xdelta.word_size as i32);
    len += n;
    r |= n;

    let tb_layout = tb.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*tb_layout).data.cast(), (*tb_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(tb);
        return -1;
    }
    cram_cram_io_c_1565_cram_free_block(tb);

    if r > 0 {
        len + len2
    } else {
        -1
    }
}

pub fn cram_cram_codecs_c_1966_cram_xdelta_encode_long(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_1971_cram_xdelta_encode_int(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub unsafe fn cram_cram_codecs_c_1976_cram_xdelta_encode_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xdelta_layout>();
    let dat = malloc((in_size * 5) as u64).cast::<c_char>();
    if dat.is_null() {
        return -1;
    }
    let mut cp = dat;
    let cp_end = dat.add((in_size * 5) as usize);
    (*c).xdelta.last = 0;

    if (*c).xdelta.word_size == 2 {
        let part = in_size % 2;
        if part != 0 {
            let z = *in_ as i16;
            (*c).xdelta.last = cram_cram_codecs_c_1713_le_int2(z) as i64;
            cp = cp.add(((*(*c).vv).varint_put32.unwrap())(
                cp,
                cp_end,
                cram_cram_codecs_c_1677_zigzag16((*c).xdelta.last as i16) as i32,
            ) as usize);
        }
        let in16 = in_.add(part as usize).cast::<i16>();
        for i in 0..(in_size / 2) {
            let v = cram_cram_codecs_c_1713_le_int2(std::ptr::read_unaligned(in16.add(i as usize)));
            let d = (v as i64 - (*c).xdelta.last) as i16;
            (*c).xdelta.last = v as i64;
            cp = cp.add(((*(*c).vv).varint_put32.unwrap())(
                cp,
                cp_end,
                cram_cram_codecs_c_1677_zigzag16(d) as i32,
            ) as usize);
        }
    }

    let sub_codec = (*c).xdelta.sub_codec;
    let sub_layout = sub_codec.cast::<cram_codec_xdelta_layout>();
    let encode: CramCodecEncodeFn = std::mem::transmute((*sub_layout).encode);
    if encode(slice, sub_codec, dat, cp.offset_from(dat) as c_int) != 0 {
        free(dat.cast());
        return -1;
    }

    free(dat.cast());
    0
}

pub unsafe fn cram_cram_codecs_c_2011_cram_xdelta_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xdelta = c.cast::<cram_codec_xdelta_layout>();
    if !(*c_xdelta).xdelta.sub_codec.is_null() {
        let sub = (*c_xdelta)
            .xdelta
            .sub_codec
            .cast::<cram_codec_xdelta_layout>();
        if !(*sub).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*sub).free);
            free_fn((*c_xdelta).xdelta.sub_codec);
        }
    }
    cram_cram_io_c_1565_cram_free_block((*c_xdelta).out);
    free(c);
}

pub unsafe fn cram_cram_codecs_c_2022_cram_xdelta_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_xdelta_layout>() as u64)
        .cast::<cram_codec_xdelta_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 53;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_2011_cram_xdelta_encode_free as usize as *mut c_void;
    (*c).decode = std::ptr::null_mut();
    (*c).encode = if option == 2 {
        cram_cram_codecs_c_1966_cram_xdelta_encode_long as usize as *mut c_void
    } else if option == 1 {
        cram_cram_codecs_c_1971_cram_xdelta_encode_int as usize as *mut c_void
    } else {
        cram_cram_codecs_c_1976_cram_xdelta_encode_char as usize as *mut c_void
    };
    (*c).store = cram_cram_codecs_c_1930_cram_xdelta_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = cram_cram_codecs_c_1835_cram_xdelta_encode_flush as usize as *mut c_void;
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();

    let e = dat.cast::<cram_xdelta_decoder_layout>();
    (*c).xdelta.word_size = (*e).word_size;
    (*c).xdelta.last = 0;
    (*c).xdelta.sub_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).sub_encoding,
        std::ptr::null_mut(),
        4,
        (*e).sub_codec_dat,
        version,
        vv,
    );

    c.cast()
}

pub fn cram_cram_codecs_c_2063_cram_xrle_decode_long(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2068_cram_xrle_decode_int(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub unsafe fn cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let cache_index = (512 + (*c_xrle).codec_id) as usize;
    let slot = (*slice_layout).block_by_id.add(cache_index);
    if !(*slot).is_null() {
        return 0;
    }

    let b = cram_cram_io_c_1388_cram_new_block(0, 0);
    *slot = b;
    if b.is_null() {
        return -1;
    }

    let lit_codec = (*c_xrle).xrle.lit_codec;
    let lit_get_block: CramCodecGetBlockFn =
        std::mem::transmute((*(lit_codec.cast::<cram_codec_xrle_layout>())).get_block);
    let lit_b = lit_get_block(slice, lit_codec);
    if lit_b.is_null() {
        return -1;
    }
    let lit_layout = lit_b.cast::<cram_block_layout>();
    let lit_dat = (*lit_layout).data;
    let lit_sz = (*lit_layout).uncomp_size as u64;

    let len_codec = (*c_xrle).xrle.len_codec;
    let len_size_fn: CramCodecSizeFn =
        std::mem::transmute((*(len_codec.cast::<cram_codec_xrle_layout>())).size);
    let len_sz = len_size_fn(slice, len_codec) as usize;
    let len_get_block: CramCodecGetBlockFn =
        std::mem::transmute((*(len_codec.cast::<cram_codec_xrle_layout>())).get_block);
    let len_b = len_get_block(slice, len_codec);
    if len_b.is_null() {
        return -1;
    }
    let len_layout = len_b.cast::<cram_block_layout>();
    let len_dat = (*len_layout).data;

    let mut rle_syms = [0u8; 256];
    let mut rle_nsyms = 0;
    for i in 0..256usize {
        if (*c_xrle).xrle.rep_score[i] > 0 {
            rle_syms[rle_nsyms] = i as u8;
            rle_nsyms += 1;
        }
    }

    let mut cp = len_dat;
    let endp = len_dat.add(len_sz);
    let mut out_sz = 0u64;
    let mut shift = 0u32;
    if cp >= endp {
        out_sz = 0;
    } else {
        loop {
            let ch = *cp;
            cp = cp.add(1);
            out_sz |= ((ch & 0x7f) as u64) << shift;
            shift += 7;
            if (ch & 0x80) == 0 || cp >= endp {
                break;
            }
        }
    }
    let nb = cp.offset_from(len_dat) as usize;

    let b_layout = b.cast::<cram_block_layout>();
    (*b_layout).data = malloc(out_sz).cast();
    if (*b_layout).data.is_null() {
        return -1;
    }
    htscodecs_hts_rle_decode(
        lit_dat,
        lit_sz,
        len_dat.add(nb),
        (len_sz - nb) as u64,
        rle_syms.as_mut_ptr(),
        rle_nsyms as c_int,
        (*b_layout).data,
        &mut out_sz,
    );
    (*b_layout).uncomp_size = out_sz as i32;
    0
}

pub unsafe extern "C" fn cram_cram_codecs_c_2115_cram_xrle_decode_size(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> c_int {
    cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(slice, c);
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let b = *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xrle).codec_id) as usize);
    (*(b.cast::<cram_block_layout>())).uncomp_size
}

pub unsafe extern "C" fn cram_cram_codecs_c_2120_cram_xrle_get_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
) -> *mut hts_sys::cram_block {
    cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(slice, c);
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xrle).codec_id) as usize)
}

pub unsafe extern "C" fn cram_cram_codecs_c_2125_cram_xrle_decode_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let n = *out_size;
    cram_cram_codecs_c_2074_cram_xrle_decode_expand_char(slice, c);
    let slice_layout = slice.cast::<cram_slice_layout>();
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let b = *(*slice_layout)
        .block_by_id
        .add((512 + (*c_xrle).codec_id) as usize);
    let b_layout = b.cast::<cram_block_layout>();
    if !out.is_null() {
        memcpy(
            out.cast(),
            (*b_layout).data.add((*b_layout).idx as usize).cast(),
            n as u64,
        );
    }
    (*b_layout).idx += n;
    0
}

pub unsafe fn cram_cram_codecs_c_2172_cram_xrle_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    if !(*c_xrle).xrle.len_codec.is_null() {
        let len = (*c_xrle).xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*len).free);
            free_fn((*c_xrle).xrle.len_codec);
        }
    }
    if !(*c_xrle).xrle.lit_codec.is_null() {
        let lit = (*c_xrle).xrle.lit_codec.cast::<cram_codec_xrle_layout>();
        if !(*lit).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*lit).free);
            free_fn((*c_xrle).xrle.lit_codec);
        }
    }
    free(c);
}

pub unsafe fn cram_cram_codecs_c_2184_cram_xrle_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = calloc(1, std::mem::size_of::<cram_codec_xrle_layout>() as u64)
        .cast::<cram_codec_xrle_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }

    (*c).codec = 52;
    (*c).decode = if option == 2 {
        cram_cram_codecs_c_2063_cram_xrle_decode_long as usize as *mut c_void
    } else if option == 1 {
        cram_cram_codecs_c_2068_cram_xrle_decode_int as usize as *mut c_void
    } else if option == 4 || option == 3 {
        cram_cram_codecs_c_2125_cram_xrle_decode_char as usize as *mut c_void
    } else {
        free(c.cast());
        return std::ptr::null_mut();
    };
    (*c).free = cram_cram_codecs_c_2172_cram_xrle_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = cram_cram_codecs_c_2115_cram_xrle_decode_size as usize as *mut c_void;
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = cram_cram_codecs_c_2120_cram_xrle_get_block as usize as *mut c_void;
    (*c).describe = std::ptr::null_mut();
    (*c).xrle.cur_len = 0;
    (*c).xrle.cur_lit = -1;

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    let mut err = 0;

    let nrle = ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    (*c).xrle.rep_score = [0; 256];
    for _ in 0..nrle.min(256) {
        let j = ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
        if (0..256).contains(&j) {
            (*c).xrle.rep_score[j as usize] = 1;
        }
    }

    (*c).xrle.len_encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        cram_cram_codecs_c_2172_cram_xrle_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).xrle.len_codec = cram_cram_codecs_c_3872_cram_decoder_init(
        hdr,
        (*c).xrle.len_encoding,
        cp,
        sub_size,
        1,
        version,
        vv,
    );
    if (*c).xrle.len_codec.is_null() {
        cram_cram_codecs_c_2172_cram_xrle_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    (*c).xrle.lit_encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), &mut err) as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        cram_cram_codecs_c_2172_cram_xrle_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).xrle.lit_codec = cram_cram_codecs_c_3872_cram_decoder_init(
        hdr,
        (*c).xrle.lit_encoding,
        cp,
        sub_size,
        option,
        version,
        vv,
    );
    if (*c).xrle.lit_codec.is_null() {
        cram_cram_codecs_c_2172_cram_xrle_decode_free(c.cast());
        return std::ptr::null_mut();
    }

    if err != 0 {
        cram_cram_codecs_c_2172_cram_xrle_decode_free(c.cast());
        return std::ptr::null_mut();
    }

    c.cast()
}

pub unsafe extern "C" fn cram_cram_codecs_c_2257_cram_xrle_encode_flush(c: *mut c_void) -> c_int {
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let mut rle_syms = [0u8; 256];
    let mut rle_nsyms = 0;
    for i in 0..256usize {
        if (*c_xrle).xrle.rep_score[i] > 0 {
            rle_syms[rle_nsyms] = i as u8;
            rle_nsyms += 1;
        }
    }

    if (*c_xrle).xrle.to_flush.is_null() {
        let out = (*c_xrle).out.cast::<cram_block_layout>();
        (*c_xrle).xrle.to_flush = (*out).data.cast();
        (*c_xrle).xrle.to_flush_size = (*out).byte;
    }

    let out_len = malloc(((*c_xrle).xrle.to_flush_size + 8) as u64).cast::<u8>();
    if out_len.is_null() {
        return -1;
    }

    let mut v = (*c_xrle).xrle.to_flush_size as u64;
    let mut nb = 0usize;
    loop {
        *out_len.add(nb) = ((v & 0x7f) as u8) + if v >= 0x80 { 0x80 } else { 0 };
        nb += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }

    let mut out_len_size = 0u64;
    let mut out_lit_size = 0u64;
    let mut rle_nsyms_i = rle_nsyms as c_int;
    let out_lit = htscodecs_hts_rle_encode(
        (*c_xrle).xrle.to_flush.cast(),
        (*c_xrle).xrle.to_flush_size as u64,
        out_len.add(nb),
        &mut out_len_size,
        rle_syms.as_mut_ptr(),
        &mut rle_nsyms_i,
        std::ptr::null_mut(),
        &mut out_lit_size,
    );
    out_len_size += nb as u64;

    let len_codec = (*c_xrle).xrle.len_codec;
    let len_encode: CramCodecEncodeFn =
        std::mem::transmute((*(len_codec.cast::<cram_codec_xrle_layout>())).encode);
    if len_encode(
        std::ptr::null_mut(),
        len_codec,
        out_len.cast(),
        out_len_size as c_int,
    ) != 0
    {
        free(out_len.cast());
        free(out_lit.cast());
        return -1;
    }

    let lit_codec = (*c_xrle).xrle.lit_codec;
    let lit_encode: CramCodecEncodeFn =
        std::mem::transmute((*(lit_codec.cast::<cram_codec_xrle_layout>())).encode);
    if lit_encode(
        std::ptr::null_mut(),
        lit_codec,
        out_lit.cast(),
        out_lit_size as c_int,
    ) != 0
    {
        free(out_len.cast());
        free(out_lit.cast());
        return -1;
    }

    free(out_len.cast());
    free(out_lit.cast());
    0
}

pub unsafe extern "C" fn cram_cram_codecs_c_2303_cram_xrle_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    let mut len = 0;
    let mut r = 0;
    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    let b_rle = cram_cram_io_c_1388_cram_new_block(0, 0);
    if b_rle.is_null() {
        return -1;
    }
    let mut nrle = 0;
    let mut len1 = 0;
    for i in 0..256i32 {
        if (*c_xrle).xrle.rep_score[i as usize] > 0 {
            nrle += 1;
            let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(b_rle, i);
            len1 += n;
            r |= n;
        }
    }

    let tc = (*c_xrle).xrle.len_codec;
    let b_len = cram_cram_io_c_1388_cram_new_block(0, 0);
    if b_len.is_null() {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        return -1;
    }
    let store: CramCodecStoreFn =
        std::mem::transmute((*(tc.cast::<cram_codec_xrle_layout>())).store);
    let len2 = store(tc, b_len, std::ptr::null_mut(), version);

    let tc = (*c_xrle).xrle.lit_codec;
    let b_lit = cram_cram_io_c_1388_cram_new_block(0, 0);
    if b_lit.is_null() {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        return -1;
    }
    let store: CramCodecStoreFn =
        std::mem::transmute((*(tc.cast::<cram_codec_xrle_layout>())).store);
    let len3 = store(tc, b_lit, std::ptr::null_mut(), version);

    let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(b, (*c_xrle).codec);
    len += n;
    r |= n;
    let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(
        b,
        len1 + len2 + len3 + ((*(*c_xrle).vv).varint_size.unwrap())(nrle as i64),
    );
    len += n;
    r |= n;
    let n = ((*(*c_xrle).vv).varint_put32_blk.unwrap())(b, nrle);
    len += n;
    r |= n;

    let b_rle_layout = b_rle.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*b_rle_layout).data.cast(), (*b_rle_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_lit);
        return -1;
    }
    let b_len_layout = b_len.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*b_len_layout).data.cast(), (*b_len_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_lit);
        return -1;
    }
    let b_lit_layout = b_lit.cast::<cram_block_layout>();
    if cram_cram_io_h_248_block_append(b, (*b_lit_layout).data.cast(), (*b_lit_layout).byte) != 0 {
        cram_cram_io_c_1565_cram_free_block(b_rle);
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_lit);
        return -1;
    }

    cram_cram_io_c_1565_cram_free_block(b_rle);
    cram_cram_io_c_1565_cram_free_block(b_len);
    cram_cram_io_c_1565_cram_free_block(b_lit);

    if r > 0 {
        len + len1 + len2 + len3
    } else {
        -1
    }
}

pub fn cram_cram_codecs_c_2359_cram_xrle_encode_long(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub fn cram_cram_codecs_c_2365_cram_xrle_encode_int(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    -1
}

pub unsafe fn cram_cram_codecs_c_2371_cram_xrle_encode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_xrle_layout>();
    if !(*c).xrle.to_flush.is_null() {
        if (*c).out.is_null() {
            (*c).out = cram_cram_io_c_1388_cram_new_block(0, 0);
            if (*c).out.is_null() {
                return -1;
            }
        }
        if cram_cram_io_h_248_block_append(
            (*c).out,
            (*c).xrle.to_flush.cast(),
            (*c).xrle.to_flush_size,
        ) != 0
        {
            return -1;
        }
        (*c).xrle.to_flush = std::ptr::null_mut();
        (*c).xrle.to_flush_size = 0;
    }

    if !(*c).out.is_null() && (*((*c).out.cast::<cram_block_layout>())).byte > 0 {
        if cram_cram_io_h_248_block_append((*c).out, in_.cast(), in_size as usize) != 0 {
            return -1;
        }
        return 0;
    }

    (*c).xrle.to_flush = in_;
    (*c).xrle.to_flush_size = in_size as usize;
    0
}

pub unsafe fn cram_cram_codecs_c_2396_cram_xrle_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_xrle = c.cast::<cram_codec_xrle_layout>();
    if !(*c_xrle).xrle.len_codec.is_null() {
        let len = (*c_xrle).xrle.len_codec.cast::<cram_codec_xrle_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*len).free);
            free_fn((*c_xrle).xrle.len_codec);
        }
    }
    if !(*c_xrle).xrle.lit_codec.is_null() {
        let lit = (*c_xrle).xrle.lit_codec.cast::<cram_codec_xrle_layout>();
        if !(*lit).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*lit).free);
            free_fn((*c_xrle).xrle.lit_codec);
        }
    }
    cram_cram_io_c_1565_cram_free_block((*c_xrle).out);
    free(c);
}

pub unsafe fn cram_cram_codecs_c_2409_cram_xrle_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_xrle_layout>() as u64)
        .cast::<cram_codec_xrle_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 52;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_2396_cram_xrle_encode_free as usize as *mut c_void;
    (*c).decode = std::ptr::null_mut();
    (*c).encode = if option == 2 {
        cram_cram_codecs_c_2359_cram_xrle_encode_long as usize as *mut c_void
    } else if option == 1 {
        cram_cram_codecs_c_2365_cram_xrle_encode_int as usize as *mut c_void
    } else {
        cram_cram_codecs_c_2371_cram_xrle_encode_char as usize as *mut c_void
    };
    (*c).store = cram_cram_codecs_c_2303_cram_xrle_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = cram_cram_codecs_c_2257_cram_xrle_encode_flush as usize as *mut c_void;
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();

    let e = dat.cast::<cram_xrle_decoder_layout>();
    (*c).xrle.len_encoding = (*e).len_encoding;
    (*c).xrle.lit_encoding = (*e).lit_encoding;
    (*c).xrle.len_dat = (*e).len_dat;
    (*c).xrle.lit_dat = (*e).lit_dat;
    (*c).xrle.len_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).len_encoding,
        std::ptr::null_mut(),
        3,
        (*e).len_dat,
        version,
        vv,
    );
    (*c).xrle.lit_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).lit_encoding,
        std::ptr::null_mut(),
        3,
        (*e).lit_dat,
        version,
        vv,
    );
    (*c).xrle.cur_lit = -1;
    (*c).xrle.cur_len = -1;
    (*c).xrle.to_flush = std::ptr::null_mut();
    (*c).xrle.to_flush_size = 0;
    (*c).xrle.rep_score = (*e).rep_score;

    c.cast()
}

pub unsafe fn cram_cram_codecs_c_2452_cram_subexp_decode(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_subexp_layout>();
    let out_i = out.cast::<i32>();
    let k = (*c).subexp.k;
    for count in 0..*out_size {
        let i = cram_cram_codecs_c_95_get_one_bits_MSB(in_);
        if i < 0
            || cram_cram_codecs_h_230_cram_not_enough_bits(in_, if i > 0 { i + k - 1 } else { k })
                != 0
        {
            return -1;
        }
        let val = if i != 0 {
            let tail = i + k - 1;
            let bits = if tail != 0 {
                cram_cram_codecs_c_169_get_bits_MSB(in_, tail) as i32
            } else {
                0
            };
            bits + (1 << tail)
        } else if k != 0 {
            cram_cram_codecs_c_169_get_bits_MSB(in_, k) as i32
        } else {
            0
        };
        *out_i.add(count as usize) = val - (*c).subexp.offset;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2496_cram_subexp_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_2501_cram_subexp_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_subexp_layout>();
    if kputsn(c"SUBEXP(offset=".as_ptr(), 14, ks) < 0
        || kputw((*c).subexp.offset, ks) < 0
        || kputsn(c",k=".as_ptr(), 3, ks) < 0
        || kputw((*c).subexp.k, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_2508_cram_subexp_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if option != 1 {
        return std::ptr::null_mut();
    }
    let c = malloc(std::mem::size_of::<cram_codec_subexp_layout>() as u64)
        .cast::<cram_codec_subexp_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 7;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_2496_cram_subexp_decode_free as usize as *mut c_void;
    (*c).decode = cram_cram_codecs_c_2452_cram_subexp_decode as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = cram_cram_codecs_c_2501_cram_subexp_describe as usize as *mut c_void;
    (*c).subexp.k = -1;

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).subexp.offset =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    (*c).subexp.k =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp.offset_from(data) != size as isize || (*c).subexp.k < 0 {
        free(c.cast());
        return std::ptr::null_mut();
    }
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_2546_cram_gamma_decode(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_gamma_layout>();
    let out_i = out.cast::<i32>();
    for i in 0..*out_size {
        let mut nz = cram_cram_codecs_c_113_get_zero_bits_MSB(in_);
        if cram_cram_codecs_h_230_cram_not_enough_bits(in_, nz) != 0 {
            return -1;
        }
        let mut val = 1;
        while nz > 0 {
            val <<= 1;
            val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
            nz -= 1;
        }
        *out_i.add(i as usize) = val - (*c).gamma.offset;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2570_cram_gamma_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_2575_cram_gamma_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_gamma_layout>();
    if kputsn(c"GAMMA(offset=".as_ptr(), 13, ks) < 0
        || kputw((*c).gamma.offset, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_2580_cram_gamma_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if option != 1 || size < 1 {
        return std::ptr::null_mut();
    }
    let c = malloc(std::mem::size_of::<cram_codec_gamma_layout>() as u64)
        .cast::<cram_codec_gamma_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 9;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_2570_cram_gamma_decode_free as usize as *mut c_void;
    (*c).decode = cram_cram_codecs_c_2546_cram_gamma_decode as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = cram_cram_codecs_c_2575_cram_gamma_describe as usize as *mut c_void;

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);
    (*c).gamma.offset =
        ((*vv).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut()) as i32;
    if cp.offset_from(data) != size as isize {
        free(c.cast());
        return std::ptr::null_mut();
    }
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_2622_code_sort(vp1: *const c_void, vp2: *const c_void) -> c_int {
    let c1 = vp1.cast::<cram_huffman_code_layout>();
    let c2 = vp2.cast::<cram_huffman_code_layout>();
    if (*c1).len != (*c2).len {
        (*c1).len - (*c2).len
    } else if (*c1).symbol < (*c2).symbol {
        -1
    } else if (*c1).symbol > (*c2).symbol {
        1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_2632_cram_huffman_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c = c.cast::<cram_codec_huffman_layout>();
    if !(*c).huffman.codes.is_null() {
        free((*c).huffman.codes.cast());
    }
    free(c.cast());
}

pub unsafe fn cram_cram_codecs_c_2795_cram_huffman_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let mut r = 0;
    r |= (kputsn(c"HUFFMAN(codes={".as_ptr(), 15, ks) < 0) as c_int;
    for n in 0..(*c).huffman.ncodes {
        if n != 0 {
            r |= (kputsn(c",".as_ptr(), 1, ks) < 0) as c_int;
        }
        r |= (kputll((*(*c).huffman.codes.add(n as usize)).symbol, ks) < 0) as c_int;
    }
    r |= (kputsn(c"},lengths={".as_ptr(), 11, ks) < 0) as c_int;
    for n in 0..(*c).huffman.ncodes {
        if n != 0 {
            r |= (kputsn(c",".as_ptr(), 1, ks) < 0) as c_int;
        }
        r |= (kputw((*(*c).huffman.codes.add(n as usize)).len, ks) < 0) as c_int;
    }
    r |= (kputsn(c"})".as_ptr(), 2, ks) < 0) as c_int;
    r
}

pub unsafe fn cram_cram_codecs_c_2814_cram_huffman_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    _version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if option == 5 {
        return std::ptr::null_mut();
    }

    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let data_end = data.add(size as usize);
    let mut err = 0;
    let ncodes64 = ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err);
    if ncodes64 < 0 {
        return std::ptr::null_mut();
    }
    let ncodes = ncodes64 as c_int;
    if ncodes as usize >= usize::MAX / std::mem::size_of::<cram_huffman_code_layout>() {
        *__errno_location() = ENOMEM;
        return std::ptr::null_mut();
    }

    let h = calloc(
        1,
        std::mem::size_of::<cram_codec_huffman_encoder_layout>() as u64,
    )
    .cast::<cram_codec_huffman_layout>();
    if h.is_null() {
        return std::ptr::null_mut();
    }
    (*h).codec = 3;
    (*h).free = cram_cram_codecs_c_2632_cram_huffman_decode_free as usize as *mut c_void;
    (*h).huffman.ncodes = ncodes;
    (*h).huffman.option = option;

    let codes = if ncodes != 0 {
        let p = malloc((ncodes as usize * std::mem::size_of::<cram_huffman_code_layout>()) as u64)
            .cast::<cram_huffman_code_layout>();
        if p.is_null() {
            free(h.cast());
            return std::ptr::null_mut();
        }
        p
    } else {
        std::ptr::null_mut()
    };
    (*h).huffman.codes = codes;

    if option == 2 {
        for i in 0..ncodes {
            (*codes.add(i as usize)).symbol =
                ((*vv).varint_get64.unwrap())(&mut cp, data_end.cast_const(), &mut err);
        }
    } else if option == 1 || option == 3 {
        for i in 0..ncodes {
            (*codes.add(i as usize)).symbol =
                ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err);
        }
    } else {
        free(codes.cast());
        free(h.cast());
        return std::ptr::null_mut();
    }
    if err != 0 {
        free(codes.cast());
        free(h.cast());
        return std::ptr::null_mut();
    }

    let n_lens = ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err) as c_int;
    if n_lens != ncodes {
        free(codes.cast());
        free(h.cast());
        return std::ptr::null_mut();
    }

    if ncodes == 0 {
        (*h).decode = cram_cram_codecs_c_2641_cram_huffman_decode_null as usize as *mut c_void;
        return h.cast();
    }

    let mut max_len = 0;
    for i in 0..ncodes {
        let len = ((*vv).varint_get32.unwrap())(&mut cp, data_end.cast_const(), &mut err) as i32;
        (*codes.add(i as usize)).len = len;
        if err != 0 || len < 0 {
            free(codes.cast());
            free(h.cast());
            return std::ptr::null_mut();
        }
        if max_len < len {
            max_len = len;
        }
    }
    if err != 0 || cp.offset_from(data) != size as isize || max_len >= ncodes {
        free(codes.cast());
        free(h.cast());
        return std::ptr::null_mut();
    }
    if max_len > 31 {
        free(codes.cast());
        free(h.cast());
        return std::ptr::null_mut();
    }

    let slice = std::slice::from_raw_parts_mut(codes, ncodes as usize);
    slice.sort_by(|a, b| {
        if a.len != b.len {
            a.len.cmp(&b.len)
        } else {
            a.symbol.cmp(&b.symbol)
        }
    });

    let mut val = -1;
    let mut last_len = 0;
    let mut max_val = 0u32;
    for i in 0..ncodes {
        val += 1;
        if val as u32 > max_val {
            free(codes.cast());
            free(h.cast());
            return std::ptr::null_mut();
        }
        if (*codes.add(i as usize)).len > last_len {
            val <<= (*codes.add(i as usize)).len - last_len;
            last_len = (*codes.add(i as usize)).len;
            max_val = (1u32 << last_len) - 1;
        }
        (*codes.add(i as usize)).code = val;
    }

    last_len = 0;
    let mut j = 0;
    for i in 0..ncodes {
        if (*codes.add(i as usize)).len > last_len {
            j = (*codes.add(i as usize)).code - i;
            last_len = (*codes.add(i as usize)).len;
        }
        (*codes.add(i as usize)).p = j;
    }

    if option == 3 || option == 4 {
        (*h).decode = if (*(*h).huffman.codes).len == 0 {
            cram_cram_codecs_c_2646_cram_huffman_decode_char0 as usize as *mut c_void
        } else {
            cram_cram_codecs_c_2660_cram_huffman_decode_char as usize as *mut c_void
        };
    } else if option == 2 || option == 7 {
        (*h).decode = if (*(*h).huffman.codes).len == 0 {
            cram_cram_codecs_c_2745_cram_huffman_decode_long0 as usize as *mut c_void
        } else {
            cram_cram_codecs_c_2758_cram_huffman_decode_long as usize as *mut c_void
        };
    } else if option == 1 || option == 6 || option == 3 {
        (*h).decode = if (*(*h).huffman.codes).len == 0 {
            cram_cram_codecs_c_2695_cram_huffman_decode_int0 as usize as *mut c_void
        } else {
            cram_cram_codecs_c_2708_cram_huffman_decode_int as usize as *mut c_void
        };
    } else {
        return std::ptr::null_mut();
    }
    (*h).describe = cram_cram_codecs_c_2795_cram_huffman_describe as usize as *mut c_void;

    h.cast()
}

pub fn cram_cram_codecs_c_2641_cram_huffman_decode_null(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    _out: *mut c_char,
    _out_size: *mut c_int,
) -> c_int {
    -1
}

pub unsafe fn cram_cram_codecs_c_2646_cram_huffman_decode_char0(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    if out.is_null() {
        return 0;
    }
    let c = c.cast::<cram_codec_huffman_layout>();
    let symbol = (*(*c).huffman.codes).symbol as c_char;
    for i in 0..*out_size {
        *out.add(i as usize) = symbol;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2660_cram_huffman_decode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ncodes = (*c).huffman.ncodes;
    let codes = (*c).huffman.codes;
    for i in 0..*out_size {
        let mut idx = 0;
        let mut val = 0;
        let mut len = 0;
        let mut last_len = 0;
        loop {
            let mut dlen = (*codes.add(idx as usize)).len - last_len;
            if cram_cram_codecs_h_230_cram_not_enough_bits(in_, dlen) != 0 {
                return -1;
            }
            last_len = {
                len += dlen;
                len
            };
            while dlen != 0 {
                val <<= 1;
                val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
                dlen -= 1;
            }
            idx = val - (*codes.add(idx as usize)).p;
            if idx >= ncodes || idx < 0 {
                return -1;
            }
            if (*codes.add(idx as usize)).code == val && (*codes.add(idx as usize)).len == len {
                if !out.is_null() {
                    *out.add(i as usize) = (*codes.add(idx as usize)).symbol as c_char;
                }
                break;
            }
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2695_cram_huffman_decode_int0(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let out_i = out.cast::<i32>();
    let symbol = (*(*c).huffman.codes).symbol as i32;
    for i in 0..*out_size {
        *out_i.add(i as usize) = symbol;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2708_cram_huffman_decode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ncodes = (*c).huffman.ncodes;
    let codes = (*c).huffman.codes;
    let out_i = out.cast::<i32>();
    for i in 0..*out_size {
        let mut idx = 0;
        let mut val = 0;
        let mut len = 0;
        let mut last_len = 0;
        loop {
            let mut dlen = (*codes.add(idx as usize)).len - last_len;
            if cram_cram_codecs_h_230_cram_not_enough_bits(in_, dlen) != 0 {
                return -1;
            }
            last_len = {
                len += dlen;
                len
            };
            while dlen != 0 {
                val <<= 1;
                val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
                dlen -= 1;
            }
            idx = val - (*codes.add(idx as usize)).p;
            if idx >= ncodes || idx < 0 {
                return -1;
            }
            if (*codes.add(idx as usize)).code == val && (*codes.add(idx as usize)).len == len {
                *out_i.add(i as usize) = (*codes.add(idx as usize)).symbol as i32;
                break;
            }
        }
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2745_cram_huffman_decode_long0(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let out_i = out.cast::<i64>();
    let symbol = (*(*c).huffman.codes).symbol;
    for i in 0..*out_size {
        *out_i.add(i as usize) = symbol;
    }
    0
}

pub unsafe fn cram_cram_codecs_c_2758_cram_huffman_decode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_layout>();
    let ncodes = (*c).huffman.ncodes;
    let codes = (*c).huffman.codes;
    let out_i = out.cast::<i64>();
    for i in 0..*out_size {
        let mut idx = 0;
        let mut val = 0;
        let mut len = 0;
        let mut last_len = 0;
        loop {
            let mut dlen = (*codes.add(idx as usize)).len - last_len;
            if cram_cram_codecs_h_230_cram_not_enough_bits(in_, dlen) != 0 {
                return -1;
            }
            last_len = {
                len += dlen;
                len
            };
            while dlen != 0 {
                val <<= 1;
                val |= cram_cram_codecs_c_73_get_bit_MSB(in_);
                dlen -= 1;
            }
            idx = val - (*codes.add(idx as usize)).p;
            if idx >= ncodes || idx < 0 {
                return -1;
            }
            if (*codes.add(idx as usize)).code == val && (*codes.add(idx as usize)).len == len {
                *out_i.add(i as usize) = (*codes.add(idx as usize)).symbol;
                break;
            }
        }
    }
    0
}

pub fn cram_cram_codecs_c_2989_cram_huffman_encode_char0(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub unsafe fn cram_cram_codecs_c_2994_cram_huffman_encode_char(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    mut in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let mut syms = in_.cast::<u8>();
    let mut r = 0;
    while in_size != 0 {
        let sym = *syms as c_int;
        syms = syms.add(1);
        let i = if sym >= -1 && sym < 128 {
            (*c).huffman.val2code[(sym + 1) as usize]
        } else {
            let mut i = 0;
            while i < (*c).huffman.nvals {
                if (*(*c).huffman.codes.add(i as usize)).symbol == sym as i64 {
                    break;
                }
                i += 1;
            }
            if i == (*c).huffman.nvals {
                return -1;
            }
            i
        };
        let code = (*(*c).huffman.codes.add(i as usize)).code;
        let len = (*(*c).huffman.codes.add(i as usize)).len;
        r |= cram_cram_codecs_c_259_store_bits_MSB((*c).out, code as u64, len);
        in_size -= 1;
    }
    r
}

pub fn cram_cram_codecs_c_3025_cram_huffman_encode_int0(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub unsafe fn cram_cram_codecs_c_3030_cram_huffman_encode_int(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    mut in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let mut syms = in_.cast::<c_int>();
    let mut r = 0;
    while in_size != 0 {
        let sym = *syms;
        syms = syms.add(1);
        let i = if sym >= -1 && sym < 128 {
            (*c).huffman.val2code[(sym + 1) as usize]
        } else {
            let mut i = 0;
            while i < (*c).huffman.nvals {
                if (*(*c).huffman.codes.add(i as usize)).symbol == sym as i64 {
                    break;
                }
                i += 1;
            }
            if i == (*c).huffman.nvals {
                return -1;
            }
            i
        };
        let code = (*(*c).huffman.codes.add(i as usize)).code;
        let len = (*(*c).huffman.codes.add(i as usize)).len;
        r |= cram_cram_codecs_c_259_store_bits_MSB((*c).out, code as u64, len);
        in_size -= 1;
    }
    r
}

pub fn cram_cram_codecs_c_3062_cram_huffman_encode_long0(
    _slice: *mut hts_sys::cram_slice,
    _c: *mut c_void,
    _in: *mut c_char,
    _in_size: c_int,
) -> c_int {
    0
}

pub unsafe fn cram_cram_codecs_c_3067_cram_huffman_encode_long(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    mut in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let mut syms = in_.cast::<i64>();
    let mut r = 0;
    while in_size != 0 {
        let sym64 = *syms;
        syms = syms.add(1);
        let i = if sym64 >= -1 && sym64 < 128 {
            (*c).huffman.val2code[(sym64 + 1) as usize]
        } else {
            let mut i = 0;
            while i < (*c).huffman.nvals {
                if (*(*c).huffman.codes.add(i as usize)).symbol == sym64 {
                    break;
                }
                i += 1;
            }
            if i == (*c).huffman.nvals {
                return -1;
            }
            i
        };
        let code = (*(*c).huffman.codes.add(i as usize)).code;
        let len = (*(*c).huffman.codes.add(i as usize)).len;
        r |= cram_cram_codecs_c_259_store_bits_MSB((*c).out, code as u64, len);
        in_size -= 1;
    }
    r
}

pub unsafe fn cram_cram_codecs_c_3099_cram_huffman_encode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    if !(*c).huffman.codes.is_null() {
        free((*c).huffman.codes.cast());
    }
    free(c.cast());
}

pub unsafe fn cram_cram_codecs_c_3112_cram_huffman_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    _version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_huffman_encoder_layout>();
    let codes = (*c).huffman.codes;
    let tmp_len = 6usize
        .saturating_mul((*c).huffman.nvals as usize)
        .saturating_add(16);
    let tmp = malloc(tmp_len as u64).cast::<c_char>();
    if tmp.is_null() {
        return -1;
    }
    let mut tp = tmp;
    let tpend = tmp.add(tmp_len);
    let mut len = 0;
    let mut r = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            free(tmp.cast());
            return -1;
        }
        len += l as c_int;
    }

    tp = tp.add(((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*c).huffman.nvals) as usize);
    if (*c).huffman.option == 2 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put64.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol,
            ) as usize);
        }
    } else if (*c).huffman.option == 7 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put64s.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol,
            ) as usize);
        }
    } else if (*c).huffman.option == 1 || (*c).huffman.option == 3 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put32.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol as i32,
            ) as usize);
        }
    } else if (*c).huffman.option == 6 {
        for i in 0..(*c).huffman.nvals {
            tp = tp.add(((*(*c).vv).varint_put32s.unwrap())(
                tp,
                tpend,
                (*codes.add(i as usize)).symbol as i32,
            ) as usize);
        }
    } else {
        free(tmp.cast());
        return -1;
    }

    tp = tp.add(((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*c).huffman.nvals) as usize);
    for i in 0..(*c).huffman.nvals {
        tp = tp.add(
            ((*(*c).vv).varint_put32.unwrap())(tp, tpend, (*codes.add(i as usize)).len) as usize,
        );
    }

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let payload_len = tp.offset_from(tmp) as c_int;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, payload_len);
    len += n;
    r |= n;
    if cram_cram_io_h_248_block_append(b, tmp.cast(), payload_len as usize) != 0 {
        free(tmp.cast());
        return -1;
    }
    len += payload_len;
    free(tmp.cast());

    if r > 0 {
        len
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_3176_cram_huffman_encode_init(
    st: *mut c_void,
    _codec: c_int,
    option: c_int,
    _dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let mut vals: *mut c_int = std::ptr::null_mut();
    let mut freqs: *mut c_int = std::ptr::null_mut();
    let mut lens: *mut c_int = std::ptr::null_mut();
    let mut vals_alloc = 0usize;
    let mut nvals = 0usize;
    let mut max_val = 0i32;
    let mut min_val = i32::MAX;

    let c = malloc(std::mem::size_of::<cram_codec_huffman_encoder_layout>() as u64)
        .cast::<cram_codec_huffman_encoder_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 3;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).decode = std::ptr::null_mut();
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();

    let st = st.cast::<cram_stats_layout>();
    for i in 0..1024usize {
        if (*st).freqs[i] == 0 {
            continue;
        }
        if nvals >= vals_alloc {
            vals_alloc = if vals_alloc != 0 {
                vals_alloc * 2
            } else {
                1024
            };
            let new_vals = realloc(
                vals.cast(),
                (vals_alloc * std::mem::size_of::<c_int>()) as u64,
            )
            .cast::<c_int>();
            if new_vals.is_null() {
                free(vals.cast());
                free(freqs.cast());
                free(lens.cast());
                free(c.cast());
                return std::ptr::null_mut();
            }
            vals = new_vals;
            let new_freqs = realloc(
                freqs.cast(),
                (vals_alloc * std::mem::size_of::<c_int>()) as u64,
            )
            .cast::<c_int>();
            if new_freqs.is_null() {
                free(vals.cast());
                free(freqs.cast());
                free(lens.cast());
                free(c.cast());
                return std::ptr::null_mut();
            }
            freqs = new_freqs;
        }
        *vals.add(nvals) = i as c_int;
        *freqs.add(nvals) = (*st).freqs[i];
        if max_val < i as i32 {
            max_val = i as i32;
        }
        if min_val > i as i32 {
            min_val = i as i32;
        }
        nvals += 1;
    }

    if !(*st).h.is_null() {
        let h = (*st).h.cast::<kh_m_i2i_layout>();
        let i_after_stat_loop = 1024i32;
        for k in 0..(*h).n_buckets {
            let flag = *(*h).flags.add((k >> 4) as usize);
            if ((flag >> ((k & 0xf) << 1)) & 3) != 0 {
                continue;
            }
            if nvals >= vals_alloc {
                vals_alloc = if vals_alloc != 0 {
                    vals_alloc * 2
                } else {
                    1024
                };
                let new_vals = realloc(
                    vals.cast(),
                    (vals_alloc * std::mem::size_of::<c_int>()) as u64,
                )
                .cast::<c_int>();
                if new_vals.is_null() {
                    free(vals.cast());
                    free(freqs.cast());
                    free(lens.cast());
                    free(c.cast());
                    return std::ptr::null_mut();
                }
                vals = new_vals;
                let new_freqs = realloc(
                    freqs.cast(),
                    (vals_alloc * std::mem::size_of::<c_int>()) as u64,
                )
                .cast::<c_int>();
                if new_freqs.is_null() {
                    free(vals.cast());
                    free(freqs.cast());
                    free(lens.cast());
                    free(c.cast());
                    return std::ptr::null_mut();
                }
                freqs = new_freqs;
            }
            *vals.add(nvals) = *(*h).keys.add(k as usize) as c_int;
            *freqs.add(nvals) = *(*h).vals.add(k as usize);
            if max_val < i_after_stat_loop {
                max_val = i_after_stat_loop;
            }
            if min_val > i_after_stat_loop {
                min_val = i_after_stat_loop;
            }
            nvals += 1;
        }
    }

    if nvals == 0 {
        free(vals.cast());
        free(freqs.cast());
        free(lens.cast());
        free(c.cast());
        return std::ptr::null_mut();
    }

    let new_freqs = realloc(
        freqs.cast(),
        (2 * nvals * std::mem::size_of::<c_int>()) as u64,
    )
    .cast::<c_int>();
    if new_freqs.is_null() {
        free(vals.cast());
        free(freqs.cast());
        free(lens.cast());
        free(c.cast());
        return std::ptr::null_mut();
    }
    freqs = new_freqs;
    lens = calloc((2 * nvals) as u64, std::mem::size_of::<c_int>() as u64).cast::<c_int>();
    if lens.is_null() {
        free(vals.cast());
        free(freqs.cast());
        free(c.cast());
        return std::ptr::null_mut();
    }

    let mut heap_nvals = nvals;
    loop {
        let mut low1 = i32::MAX;
        let mut low2 = i32::MAX;
        let mut ind1 = 0usize;
        let mut ind2 = 0usize;
        for i in 0..heap_nvals {
            let f = *freqs.add(i);
            if f < 0 {
                continue;
            }
            if low1 > f {
                low2 = low1;
                ind2 = ind1;
                low1 = f;
                ind1 = i;
            } else if low2 > f {
                low2 = f;
                ind2 = i;
            }
        }
        if low2 == i32::MAX {
            break;
        }
        *freqs.add(heap_nvals) = low1 + low2;
        *lens.add(ind1) = heap_nvals as c_int;
        *lens.add(ind2) = heap_nvals as c_int;
        *freqs.add(ind1) *= -1;
        *freqs.add(ind2) *= -1;
        heap_nvals += 1;
    }
    nvals = heap_nvals / 2 + 1;

    for i in 0..nvals {
        let mut code_len = 0;
        let mut k = *lens.add(i);
        while k != 0 {
            code_len += 1;
            k = *lens.add(k as usize);
        }
        *lens.add(i) = code_len;
        *freqs.add(i) *= -1;
    }

    let codes = malloc(nvals as u64 * std::mem::size_of::<cram_huffman_code_layout>() as u64)
        .cast::<cram_huffman_code_layout>();
    if codes.is_null() {
        free(vals.cast());
        free(freqs.cast());
        free(lens.cast());
        free(c.cast());
        return std::ptr::null_mut();
    }
    for i in 0..nvals {
        (*codes.add(i)).symbol = *vals.add(i) as i64;
        (*codes.add(i)).p = 0;
        (*codes.add(i)).code = 0;
        (*codes.add(i)).len = *lens.add(i);
    }

    std::slice::from_raw_parts_mut(codes, nvals).sort_by(|a, b| {
        if a.len != b.len {
            a.len.cmp(&b.len)
        } else {
            a.symbol.cmp(&b.symbol)
        }
    });

    let mut code = 0;
    let mut len = (*codes).len;
    for i in 0..nvals {
        while len != (*codes.add(i)).len {
            code <<= 1;
            len += 1;
        }
        (*codes.add(i)).code = code;
        code += 1;

        let symbol = (*codes.add(i)).symbol;
        if symbol >= -1 && symbol < 128 {
            (*c).huffman.val2code[(symbol + 1) as usize] = i as c_int;
        }
    }

    free(lens.cast());
    free(vals.cast());
    free(freqs.cast());

    (*c).huffman.codes = codes;
    (*c).huffman.nvals = nvals as c_int;
    (*c).huffman.option = option;
    (*c).free = cram_cram_codecs_c_3099_cram_huffman_encode_free as usize as *mut c_void;
    (*c).encode = if option == 3 || option == 4 {
        if (*(*c).huffman.codes).len == 0 {
            cram_cram_codecs_c_2989_cram_huffman_encode_char0 as usize as *mut c_void
        } else {
            cram_cram_codecs_c_2994_cram_huffman_encode_char as usize as *mut c_void
        }
    } else if option == 1 || option == 6 {
        if (*(*c).huffman.codes).len == 0 {
            cram_cram_codecs_c_3025_cram_huffman_encode_int0 as usize as *mut c_void
        } else {
            cram_cram_codecs_c_3030_cram_huffman_encode_int as usize as *mut c_void
        }
    } else if option == 2 || option == 7 {
        if (*(*c).huffman.codes).len == 0 {
            cram_cram_codecs_c_3062_cram_huffman_encode_long0 as usize as *mut c_void
        } else {
            cram_cram_codecs_c_3067_cram_huffman_encode_long as usize as *mut c_void
        }
    } else {
        return std::ptr::null_mut();
    };
    (*c).store = cram_cram_codecs_c_3112_cram_huffman_encode_store as usize as *mut c_void;

    let _ = (max_val, min_val);
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_3371_cram_byte_array_len_decode(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut hts_sys::cram_block,
    out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut len = 0i32;
    let mut one = 1;
    let len_codec = (*c).byte_array_len.len_codec;
    let val_codec = (*c).byte_array_len.val_codec;
    let len_decode: CramCodecDecodeFn =
        std::mem::transmute((*(len_codec.cast::<cram_codec_byte_array_len_layout>())).decode);
    let mut r = len_decode(
        slice,
        len_codec,
        in_,
        (&mut len as *mut i32).cast(),
        &mut one,
    );

    let val_layout = val_codec.cast::<cram_codec_external_layout>();
    let val_is_external_block =
        !val_codec.is_null() && (*val_layout).codec == 1 && (*val_layout).external.type_ == 5;
    if len < 0 || (len > *out_size && !val_is_external_block) {
        return -1;
    }

    if r == 0 && !val_codec.is_null() {
        let val_decode: CramCodecDecodeFn =
            std::mem::transmute((*(val_codec.cast::<cram_codec_byte_array_len_layout>())).decode);
        r = val_decode(slice, val_codec, in_, out, &mut len);
    } else {
        return -1;
    }
    *out_size = len;
    r
}

pub unsafe fn cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c: *mut c_void) {
    if c.is_null() {
        return;
    }
    let c_ba = c.cast::<cram_codec_byte_array_len_layout>();
    if !(*c_ba).byte_array_len.len_codec.is_null() {
        let len = (*c_ba)
            .byte_array_len
            .len_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*len).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*len).free);
            free_fn((*c_ba).byte_array_len.len_codec);
        }
    }
    if !(*c_ba).byte_array_len.val_codec.is_null() {
        let val = (*c_ba)
            .byte_array_len
            .val_codec
            .cast::<cram_codec_byte_array_len_layout>();
        if !(*val).free.is_null() {
            let free_fn: unsafe fn(*mut c_void) = std::mem::transmute((*val).free);
            free_fn((*c_ba).byte_array_len.val_codec);
        }
    }
    free(c);
}

pub unsafe fn cram_cram_codecs_c_3412_cram_byte_array_len_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut r = 0;
    r |= (kputsn(c"BYTE_ARRAY_LEN(len_codec={".as_ptr(), 26, ks) < 0) as c_int;
    let len_codec = (*c).byte_array_len.len_codec;
    if !(*(len_codec.cast::<cram_codec_byte_array_len_layout>()))
        .describe
        .is_null()
    {
        let describe: CramCodecDescribeFn =
            std::mem::transmute((*(len_codec.cast::<cram_codec_byte_array_len_layout>())).describe);
        r |= describe(len_codec, ks);
    } else {
        r |= (kputsn(c"?".as_ptr(), 1, ks) < 0) as c_int;
    }
    r |= (kputsn(c"},val_codec={".as_ptr(), 13, ks) < 0) as c_int;
    let val_codec = (*c).byte_array_len.val_codec;
    if !(*(val_codec.cast::<cram_codec_byte_array_len_layout>()))
        .describe
        .is_null()
    {
        let describe: CramCodecDescribeFn =
            std::mem::transmute((*(val_codec.cast::<cram_codec_byte_array_len_layout>())).describe);
        r |= describe(val_codec, ks);
    } else {
        r |= (kputsn(c"?".as_ptr(), 1, ks) < 0) as c_int;
    }
    r |= (kputsn(c"}".as_ptr(), 1, ks) < 0) as c_int;
    r
}

pub unsafe fn cram_cram_codecs_c_3428_cram_byte_array_len_decode_init(
    hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_byte_array_len_layout>() as u64)
        .cast::<cram_codec_byte_array_len_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 4;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).decode = cram_cram_codecs_c_3371_cram_byte_array_len_decode as usize as *mut c_void;
    (*c).free = cram_cram_codecs_c_3400_cram_byte_array_len_decode_free as usize as *mut c_void;
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = cram_cram_codecs_c_3412_cram_byte_array_len_describe as usize as *mut c_void;
    (*c).byte_array_len.len_codec = std::ptr::null_mut();
    (*c).byte_array_len.val_codec = std::ptr::null_mut();

    let vv_layout = vv.cast::<varint_vec_layout>();
    let mut cp = data;
    let endp = data.add(size as usize);

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).byte_array_len.len_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, 1, version, vv);
    if (*c).byte_array_len.len_codec.is_null() {
        cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    let encoding =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    let sub_size =
        ((*vv_layout).varint_get32.unwrap())(&mut cp, endp.cast_const(), std::ptr::null_mut())
            as c_int;
    if sub_size < 0 || endp.offset_from(cp) < sub_size as isize {
        cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    (*c).byte_array_len.val_codec =
        cram_cram_codecs_c_3872_cram_decoder_init(hdr, encoding, cp, sub_size, option, version, vv);
    if (*c).byte_array_len.val_codec.is_null() {
        cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c.cast());
        return std::ptr::null_mut();
    }
    cp = cp.add(sub_size as usize);

    if cp.offset_from(data) != size as isize {
        cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c.cast());
        return std::ptr::null_mut();
    }

    c.cast()
}

pub unsafe fn cram_cram_codecs_c_3479_cram_byte_array_len_encode(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut i32_ = in_size;
    let len_codec = (*c).byte_array_len.len_codec;
    let val_codec = (*c).byte_array_len.val_codec;
    let len_encode: CramCodecEncodeFn =
        std::mem::transmute((*(len_codec.cast::<cram_codec_byte_array_len_layout>())).encode);
    let val_encode: CramCodecEncodeFn =
        std::mem::transmute((*(val_codec.cast::<cram_codec_byte_array_len_layout>())).encode);
    let mut r = 0;
    r |= len_encode(slice, len_codec, (&mut i32_ as *mut i32).cast(), 1);
    r |= val_encode(slice, val_codec, in_, in_size);
    r
}

pub unsafe fn cram_cram_codecs_c_3493_cram_byte_array_len_encode_free(c: *mut c_void) {
    cram_cram_codecs_c_3400_cram_byte_array_len_decode_free(c);
}

pub unsafe fn cram_cram_codecs_c_3506_cram_byte_array_len_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_len_layout>();
    let mut len = 0;
    let mut r = 0;
    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    let tc = (*c).byte_array_len.len_codec;
    let b_len = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if b_len.is_null() {
        return -1;
    }
    let store: CramCodecStoreFn =
        std::mem::transmute((*(tc.cast::<cram_codec_byte_array_len_layout>())).store);
    let len2 = store(tc, b_len, std::ptr::null_mut(), version);
    if len2 < 0 {
        cram_cram_io_c_1565_cram_free_block(b_len);
        return -1;
    }

    let tc = (*c).byte_array_len.val_codec;
    let b_val = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if b_val.is_null() {
        cram_cram_io_c_1565_cram_free_block(b_len);
        return -1;
    }
    let store: CramCodecStoreFn =
        std::mem::transmute((*(tc.cast::<cram_codec_byte_array_len_layout>())).store);
    let len3 = store(tc, b_val, std::ptr::null_mut(), version);
    if len3 < 0 {
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_val);
        return -1;
    }

    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, (*c).codec);
    len += n;
    r |= n;
    let n = ((*(*c).vv).varint_put32_blk.unwrap())(b, len2 + len3);
    len += n;
    r |= n;
    if cram_cram_io_h_248_block_append(
        b,
        (*(b_len.cast::<cram_block_layout>())).data.cast(),
        (*(b_len.cast::<cram_block_layout>())).byte,
    ) != 0
        || cram_cram_io_h_248_block_append(
            b,
            (*(b_val.cast::<cram_block_layout>())).data.cast(),
            (*(b_val.cast::<cram_block_layout>())).byte,
        ) != 0
    {
        cram_cram_io_c_1565_cram_free_block(b_len);
        cram_cram_io_c_1565_cram_free_block(b_val);
        return -1;
    }

    cram_cram_io_c_1565_cram_free_block(b_len);
    cram_cram_io_c_1565_cram_free_block(b_val);

    if r > 0 {
        len + len2 + len3
    } else {
        -1
    }
}

pub unsafe fn cram_cram_codecs_c_3547_cram_byte_array_len_encode_init(
    st: *mut c_void,
    _codec: c_int,
    _option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let e = dat.cast::<cram_byte_array_len_encoder_dat_layout>();
    let c = malloc(std::mem::size_of::<cram_codec_byte_array_len_layout>() as u64)
        .cast::<cram_codec_byte_array_len_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 4;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_3493_cram_byte_array_len_encode_free as usize as *mut c_void;
    (*c).decode = std::ptr::null_mut();
    (*c).encode = cram_cram_codecs_c_3479_cram_byte_array_len_encode as usize as *mut c_void;
    (*c).store = cram_cram_codecs_c_3506_cram_byte_array_len_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();

    (*c).byte_array_len.len_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).len_encoding,
        st,
        1,
        (*e).len_dat,
        version,
        vv,
    );
    (*c).byte_array_len.val_codec = cram_cram_codecs_c_3928_cram_encoder_init(
        (*e).val_encoding,
        std::ptr::null_mut(),
        4,
        (*e).val_dat,
        version,
        vv,
    );
    if (*c).byte_array_len.len_codec.is_null() || (*c).byte_array_len.val_codec.is_null() {
        cram_cram_codecs_c_3493_cram_byte_array_len_encode_free(c.cast());
        return std::ptr::null_mut();
    }

    c.cast()
}

pub unsafe fn cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    mut out: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).byte_array_stop.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let b = b.cast::<cram_block_layout>();
    if (*b).idx >= (*b).uncomp_size {
        return -1;
    }
    let mut term = (*b).uncomp_size - (*b).idx;
    let mut cp = (*b).data.add((*b).idx as usize);
    let start_idx = (*b).idx;
    if !out.is_null() {
        if term > *out_size {
            term = *out_size;
        }
        loop {
            term -= 1;
            if term < 0 || *cp == (*c).byte_array_stop.stop {
                break;
            }
            *out = *cp as c_char;
            out = out.add(1);
            cp = cp.add(1);
        }
    } else {
        loop {
            term -= 1;
            if term < 0 || *cp == (*c).byte_array_stop.stop {
                break;
            }
            cp = cp.add(1);
        }
    }
    if cp >= (*b).data.add((*b).uncomp_size as usize) || *cp != (*c).byte_array_stop.stop {
        return -1;
    }
    *out_size = cp.offset_from((*b).data.add(start_idx as usize)) as c_int;
    (*b).idx = cp.offset_from((*b).data) as i32 + 1;
    0
}

pub unsafe fn cram_cram_codecs_c_3626_cram_byte_array_stop_decode_block(
    slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    _in: *mut hts_sys::cram_block,
    out_: *mut c_char,
    out_size: *mut c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let b = cram_cram_io_h_183_cram_get_block_by_id(slice, (*c).byte_array_stop.content_id);
    if b.is_null() {
        return if *out_size != 0 { -1 } else { 0 };
    }
    let b = b.cast::<cram_block_layout>();
    if (*b).idx >= (*b).uncomp_size {
        return -1;
    }
    let mut cp = (*b).data.add((*b).idx as usize);
    let cp_end = (*b).data.add((*b).uncomp_size as usize);
    let stop = if (*b).orig_method == 8 {
        0
    } else {
        (*c).byte_array_stop.stop
    };
    let cp_start = cp;
    while cp != cp_end && *cp != stop {
        cp = cp.add(1);
    }
    if cram_cram_io_h_248_block_append(
        out_.cast(),
        cp_start.cast(),
        cp.offset_from(cp_start) as usize,
    ) != 0
    {
        return -1;
    }
    *out_size = cp.offset_from((*b).data.add((*b).idx as usize)) as c_int;
    (*b).idx = cp.offset_from((*b).data) as i32 + 1;
    0
}

pub unsafe fn cram_cram_codecs_c_3669_cram_byte_array_stop_decode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_3675_cram_byte_array_stop_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    if kputsn(c"BYTE_ARRAY_STOP(stop=".as_ptr(), 21, ks) < 0
        || kputw((*c).byte_array_stop.stop as c_int, ks) < 0
        || kputsn(c",id=".as_ptr(), 4, ks) < 0
        || kputw((*c).byte_array_stop.content_id, ks) < 0
        || kputsn(c")".as_ptr(), 1, ks) < 0
    {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init(
    _hdr: *mut c_void,
    data: *mut c_char,
    size: c_int,
    _codec: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let vv = vv.cast::<varint_vec_layout>();
    let mut cp = data.cast::<u8>();
    let min_size = if (version >> 8) == 1 { 5 } else { 2 };
    if size < min_size {
        return std::ptr::null_mut();
    }

    let c = malloc(std::mem::size_of::<cram_codec_byte_array_stop_layout>() as u64)
        .cast::<cram_codec_byte_array_stop_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }

    (*c).codec = 5;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_3669_cram_byte_array_stop_decode_free as usize as *mut c_void;
    (*c).decode = match option {
        5 => cram_cram_codecs_c_3626_cram_byte_array_stop_decode_block as usize as *mut c_void,
        4 => cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char as usize as *mut c_void,
        _ => {
            free(c.cast());
            return std::ptr::null_mut();
        }
    };
    (*c).encode = std::ptr::null_mut();
    (*c).store = std::ptr::null_mut();
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = cram_cram_codecs_c_3675_cram_byte_array_stop_describe as usize as *mut c_void;

    (*c).byte_array_stop.stop = *cp;
    cp = cp.add(1);
    if (version >> 8) == 1 {
        (*c).byte_array_stop.content_id = *cp.add(0) as i32
            + ((*cp.add(1) as i32) << 8)
            + ((*cp.add(2) as i32) << 16)
            + ((*cp.add(3) as u32) << 24) as i32;
        cp = cp.add(4);
    } else {
        let mut err = 0;
        let mut c_cp = cp.cast::<c_char>();
        let endp = data.add(size as usize);
        (*c).byte_array_stop.content_id =
            ((*vv).varint_get32.unwrap())(&mut c_cp, endp.cast_const(), &mut err) as i32;
        cp = c_cp.cast::<u8>();
        if err != 0 {
            free(c.cast());
            return std::ptr::null_mut();
        }
    }

    if cp.cast::<c_char>().offset_from(data) != size as isize {
        free(c.cast());
        return std::ptr::null_mut();
    }

    c.cast()
}

pub unsafe fn cram_cram_codecs_c_3733_cram_byte_array_stop_encode(
    _slice: *mut hts_sys::cram_slice,
    c: *mut c_void,
    in_: *mut c_char,
    in_size: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    if cram_cram_io_h_248_block_append((*c).out, in_.cast(), in_size as usize) != 0 {
        return -1;
    }
    cram_cram_io_h_261_block_append_char((*c).out, (*c).byte_array_stop.stop as c_char)
}

pub unsafe fn cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free(c: *mut c_void) {
    if !c.is_null() {
        free(c);
    }
}

pub unsafe fn cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store(
    c: *mut c_void,
    b: *mut hts_sys::cram_block,
    prefix: *mut c_char,
    version: c_int,
) -> c_int {
    let c = c.cast::<cram_codec_byte_array_stop_layout>();
    let mut len = 0;

    if !prefix.is_null() {
        let l = libc::strlen(prefix);
        if cram_cram_io_h_248_block_append(b, prefix.cast(), l) != 0 {
            return -1;
        }
        len += l as c_int;
    }

    let mut buf = [0 as c_char; 20];
    let mut cp = buf.as_mut_ptr();
    let endp = buf.as_mut_ptr().add(20);
    let vv = (*c).vv;
    cp = cp.add(((*vv).varint_put32.unwrap())(cp, endp, (*c).codec) as usize);
    if (version >> 8) == 1 {
        cp = cp.add(((*vv).varint_put32.unwrap())(cp, endp, 5) as usize);
        *cp = (*c).byte_array_stop.stop as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 0) as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 8) as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 16) as c_char;
        cp = cp.add(1);
        *cp = ((*c).byte_array_stop.content_id >> 24) as c_char;
        cp = cp.add(1);
    } else {
        cp = cp.add(((*vv).varint_put32.unwrap())(
            cp,
            endp,
            1 + ((*vv).varint_size.unwrap())((*c).byte_array_stop.content_id as i64),
        ) as usize);
        *cp = (*c).byte_array_stop.stop as c_char;
        cp = cp.add(1);
        cp = cp
            .add(((*vv).varint_put32.unwrap())(cp, endp, (*c).byte_array_stop.content_id) as usize);
    }

    let n = cp.offset_from(buf.as_ptr()) as usize;
    if cram_cram_io_h_248_block_append(b, buf.as_ptr().cast(), n) != 0 {
        return -1;
    }
    len + n as c_int
}

pub unsafe fn cram_cram_codecs_c_3785_cram_byte_array_stop_encode_init(
    _st: *mut c_void,
    _codec: c_int,
    _option: c_int,
    dat: *mut c_void,
    _version: c_int,
    _vv: *mut c_void,
) -> *mut c_void {
    let c = malloc(std::mem::size_of::<cram_codec_byte_array_stop_layout>() as u64)
        .cast::<cram_codec_byte_array_stop_layout>();
    if c.is_null() {
        return std::ptr::null_mut();
    }
    (*c).codec = 5;
    (*c).out = std::ptr::null_mut();
    (*c).vv = std::ptr::null_mut();
    (*c).codec_id = 0;
    (*c).free = cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free as usize as *mut c_void;
    (*c).decode = std::ptr::null_mut();
    (*c).encode = cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize as *mut c_void;
    (*c).store = cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store as usize as *mut c_void;
    (*c).size = std::ptr::null_mut();
    (*c).flush = std::ptr::null_mut();
    (*c).get_block = std::ptr::null_mut();
    (*c).describe = std::ptr::null_mut();
    let dat = dat.cast::<c_int>();
    (*c).byte_array_stop.stop = *dat as u8;
    (*c).byte_array_stop.content_id = *dat.add(1);
    c.cast()
}

pub unsafe fn cram_cram_codecs_c_3872_cram_decoder_init(
    hdr: *mut c_void,
    codec: c_int,
    data: *mut c_char,
    size: c_int,
    option: c_int,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    let init: Option<CramCodecDecodeInitFn> = match codec {
        1 => Some(cram_cram_codecs_c_459_cram_external_decode_init),
        3 => Some(cram_cram_codecs_c_2814_cram_huffman_decode_init),
        4 => Some(cram_cram_codecs_c_3428_cram_byte_array_len_decode_init),
        5 => Some(cram_cram_codecs_c_3682_cram_byte_array_stop_decode_init),
        6 => Some(cram_cram_codecs_c_1142_cram_beta_decode_init),
        7 => Some(cram_cram_codecs_c_2508_cram_subexp_decode_init),
        9 => Some(cram_cram_codecs_c_2580_cram_gamma_decode_init),
        41 | 42 => Some(cram_cram_codecs_c_760_cram_varint_decode_init),
        43 | 44 => Some(cram_cram_codecs_c_981_cram_const_decode_init),
        51 => Some(cram_cram_codecs_c_1453_cram_xpack_decode_init),
        52 => Some(cram_cram_codecs_c_2184_cram_xrle_decode_init),
        53 => Some(cram_cram_codecs_c_1781_cram_xdelta_decode_init),
        _ => None,
    };

    if let Some(init) = init {
        let r = init(hdr, data, size, codec, option, version, vv);
        if !r.is_null() {
            let hdr_layout = hdr.cast::<cram_block_compression_hdr_layout>();
            (*(r.cast::<cram_codec_external_layout>())).vv = vv.cast::<varint_vec_layout>();
            (*(r.cast::<cram_codec_external_layout>())).codec_id = (*hdr_layout).ncodecs;
            (*hdr_layout).ncodecs += 1;
        }
        r
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn cram_cram_codecs_c_3928_cram_encoder_init(
    mut codec: c_int,
    st: *mut c_void,
    option: c_int,
    dat: *mut c_void,
    version: c_int,
    vv: *mut c_void,
) -> *mut c_void {
    if !st.is_null() && (*(st.cast::<cram_stats_layout>())).nvals == 0 {
        return std::ptr::null_mut();
    }

    if option == 3 || option == 4 || option == 5 {
        if codec == 41 || codec == 42 {
            codec = 1;
        } else if codec == 44 {
            codec = 43;
        }
    }

    let init: Option<CramCodecEncodeInitFn> = match codec {
        1 => Some(cram_cram_codecs_c_586_cram_external_encode_init),
        3 => Some(cram_cram_codecs_c_3176_cram_huffman_encode_init),
        4 => Some(cram_cram_codecs_c_3547_cram_byte_array_len_encode_init),
        5 => Some(cram_cram_codecs_c_3785_cram_byte_array_stop_encode_init),
        6 => Some(cram_cram_codecs_c_1247_cram_beta_encode_init),
        41 | 42 => Some(cram_cram_codecs_c_878_cram_varint_encode_init),
        43 | 44 => Some(cram_cram_codecs_c_1048_cram_const_encode_init),
        51 => Some(cram_cram_codecs_c_1623_cram_xpack_encode_init),
        52 => Some(cram_cram_codecs_c_2409_cram_xrle_encode_init),
        53 => Some(cram_cram_codecs_c_2022_cram_xdelta_encode_init),
        _ => None,
    };

    if let Some(init) = init {
        let r = init(st, codec, option, dat, version, vv);
        if r.is_null() {
            return std::ptr::null_mut();
        }
        (*(r.cast::<cram_codec_external_layout>())).out = std::ptr::null_mut();
        (*(r.cast::<cram_codec_external_layout>())).vv = vv.cast::<varint_vec_layout>();
        r
    } else {
        libc::abort();
    }
}

pub unsafe fn cram_cram_codecs_c_3968_cram_codec_to_id(c: *mut c_void, id2: *mut c_int) -> c_int {
    let codec = (*(c.cast::<cram_codec_external_layout>())).codec;
    let mut bnum2 = -2;
    let bnum1 = match codec {
        43 | 44 => -2,
        3 => {
            let c = c.cast::<cram_codec_huffman_layout>();
            if (*c).huffman.ncodes == 1 {
                -2
            } else {
                -1
            }
        }
        2 | 6 | 7 | 8 | 9 => -1,
        1 | 41 | 42 => {
            (*(c.cast::<cram_codec_external_layout>()))
                .external
                .content_id
        }
        4 => {
            let c = c.cast::<cram_codec_byte_array_len_layout>();
            let len_codec = (*c).byte_array_len.len_codec;
            let val_codec = (*c).byte_array_len.val_codec;
            bnum2 = cram_cram_codecs_c_3968_cram_codec_to_id(val_codec, std::ptr::null_mut());
            cram_cram_codecs_c_3968_cram_codec_to_id(len_codec, std::ptr::null_mut())
        }
        5 => {
            (*(c.cast::<cram_codec_byte_array_stop_layout>()))
                .byte_array_stop
                .content_id
        }
        0 => -2,
        _ => -1,
    };
    if !id2.is_null() {
        *id2 = bnum2;
    }
    bnum1
}

pub unsafe fn cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
    _fd: *mut c_void,
    c: *mut c_void,
) -> c_int {
    let base = c.cast::<cram_codec_external_layout>();
    match (*base).codec {
        43 | 44 => {
            (*base).store = cram_cram_codecs_c_1025_cram_const_encode_store as usize as *mut c_void;
            0
        }
        1 => {
            (*base).free = cram_cram_codecs_c_556_cram_external_encode_free as usize as *mut c_void;
            (*base).store =
                cram_cram_codecs_c_562_cram_external_encode_store as usize as *mut c_void;
            (*base).encode = if (*base).decode
                == cram_cram_codecs_c_350_cram_external_decode_int as usize as *mut c_void
            {
                cram_cram_codecs_c_523_cram_external_encode_int as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_370_cram_external_decode_long as usize as *mut c_void
            {
                cram_cram_codecs_c_535_cram_external_encode_long as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_390_cram_external_decode_char as usize as *mut c_void
                || (*base).decode
                    == cram_cram_codecs_c_410_cram_external_decode_block as usize as *mut c_void
            {
                cram_cram_codecs_c_547_cram_external_encode_char as usize as *mut c_void
            } else {
                return -1;
            };
            0
        }
        41 | 42 => {
            (*base).free = cram_cram_codecs_c_848_cram_varint_encode_free as usize as *mut c_void;
            (*base).store = cram_cram_codecs_c_854_cram_varint_encode_store as usize as *mut c_void;
            (*base).encode = if (*base).decode
                == cram_cram_codecs_c_644_cram_varint_decode_int as usize as *mut c_void
            {
                cram_cram_codecs_c_820_cram_varint_encode_int as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_666_cram_varint_decode_sint as usize as *mut c_void
            {
                cram_cram_codecs_c_827_cram_varint_encode_sint as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_688_cram_varint_decode_long as usize as *mut c_void
            {
                cram_cram_codecs_c_834_cram_varint_encode_long as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_710_cram_varint_decode_slong as usize as *mut c_void
            {
                cram_cram_codecs_c_841_cram_varint_encode_slong as usize as *mut c_void
            } else {
                return -1;
            };
            0
        }
        6 => {
            (*base).free = cram_cram_codecs_c_1243_cram_beta_encode_free as usize as *mut c_void;
            (*base).store = cram_cram_codecs_c_1183_cram_beta_encode_store as usize as *mut c_void;
            (*base).encode = if (*base).decode
                == cram_cram_codecs_c_1090_cram_beta_decode_int as usize as *mut c_void
            {
                cram_cram_codecs_c_1219_cram_beta_encode_int as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_1072_cram_beta_decode_long as usize as *mut c_void
            {
                cram_cram_codecs_c_1207_cram_beta_encode_long as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_1108_cram_beta_decode_char as usize as *mut c_void
            {
                cram_cram_codecs_c_1231_cram_beta_encode_char as usize as *mut c_void
            } else {
                return -1;
            };
            0
        }
        3 => {
            let dec = c.cast::<cram_codec_huffman_layout>();
            let enc = c.cast::<cram_codec_huffman_encoder_layout>();
            (*enc).codec = 3;
            (*enc).vv = (*dec).vv;
            (*enc).out = (*dec).out;
            (*enc).codec_id = (*dec).codec_id;
            (*enc).free = cram_cram_codecs_c_3099_cram_huffman_encode_free as usize as *mut c_void;
            (*enc).store =
                cram_cram_codecs_c_3112_cram_huffman_encode_store as usize as *mut c_void;
            let codes = (*dec).huffman.codes;
            let nvals = (*dec).huffman.ncodes;
            let option = (*dec).huffman.option;
            (*enc).huffman.codes = codes;
            (*enc).huffman.nvals = nvals;
            (*enc).huffman.val2code = [0; 129];
            (*enc).huffman.option = option;
            for j in 0..nvals {
                let sym = (*codes.add(j as usize)).symbol as i32;
                if (-1..128).contains(&sym) {
                    (*enc).huffman.val2code[(sym + 1) as usize] = j;
                }
            }
            (*enc).encode = if (*base).decode
                == cram_cram_codecs_c_2646_cram_huffman_decode_char0 as usize as *mut c_void
            {
                cram_cram_codecs_c_2989_cram_huffman_encode_char0 as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_2660_cram_huffman_decode_char as usize as *mut c_void
            {
                cram_cram_codecs_c_2994_cram_huffman_encode_char as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_2695_cram_huffman_decode_int0 as usize as *mut c_void
            {
                cram_cram_codecs_c_3025_cram_huffman_encode_int0 as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_2708_cram_huffman_decode_int as usize as *mut c_void
            {
                cram_cram_codecs_c_3030_cram_huffman_encode_int as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_2745_cram_huffman_decode_long0 as usize as *mut c_void
            {
                cram_cram_codecs_c_3062_cram_huffman_encode_long0 as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_2758_cram_huffman_decode_long as usize as *mut c_void
            {
                cram_cram_codecs_c_3067_cram_huffman_encode_long as usize as *mut c_void
            } else {
                return -1;
            };
            0
        }
        51 => {
            (*base).free = cram_cram_codecs_c_1612_cram_xpack_encode_free as usize as *mut c_void;
            (*base).store = cram_cram_codecs_c_1537_cram_xpack_encode_store as usize as *mut c_void;
            (*base).encode = if (*base).decode
                == cram_cram_codecs_c_1344_cram_xpack_decode_long as usize as *mut c_void
            {
                cram_cram_codecs_c_1581_cram_xpack_encode_long as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_1359_cram_xpack_decode_int as usize as *mut c_void
            {
                cram_cram_codecs_c_1592_cram_xpack_encode_int as usize as *mut c_void
            } else if (*base).decode
                == cram_cram_codecs_c_1408_cram_xpack_decode_char as usize as *mut c_void
            {
                cram_cram_codecs_c_1603_cram_xpack_encode_char as usize as *mut c_void
            } else {
                return -1;
            };
            let xpack = c.cast::<cram_codec_xpack_layout>();
            if cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                std::ptr::null_mut(),
                (*xpack).xpack.sub_codec,
            ) == -1
            {
                return -1;
            }
            0
        }
        4 => {
            (*base).free =
                cram_cram_codecs_c_3493_cram_byte_array_len_encode_free as usize as *mut c_void;
            (*base).store =
                cram_cram_codecs_c_3506_cram_byte_array_len_encode_store as usize as *mut c_void;
            (*base).encode =
                cram_cram_codecs_c_3479_cram_byte_array_len_encode as usize as *mut c_void;
            let bal = c.cast::<cram_codec_byte_array_len_layout>();
            if cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                std::ptr::null_mut(),
                (*bal).byte_array_len.len_codec,
            ) == -1
                || cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (*bal).byte_array_len.val_codec,
                ) == -1
            {
                return -1;
            }
            0
        }
        5 => {
            (*base).free =
                cram_cram_codecs_c_3743_cram_byte_array_stop_encode_free as usize as *mut c_void;
            (*base).store =
                cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store as usize as *mut c_void;
            (*base).encode =
                cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize as *mut c_void;
            0
        }
        _ => -1,
    }
}

pub unsafe fn cram_cram_codecs_c_4185_cram_codec_describe(
    c: *mut c_void,
    ks: *mut kstring_t,
) -> c_int {
    if !c.is_null()
        && !(*(c.cast::<cram_codec_external_layout>()))
            .describe
            .is_null()
    {
        let describe: CramCodecDescribeFn =
            std::mem::transmute((*(c.cast::<cram_codec_external_layout>())).describe);
        describe(c, ks)
    } else if kputsn(c"?".as_ptr(), 1, ks) < 0 {
        -1
    } else {
        0
    }
}

pub fn cram_cram_codecs_c_3811_cram_encoding2str(t: c_int) -> *mut c_char {
    let s: &'static [u8] = match t {
        0 => b"NULL\0",
        1 => b"EXTERNAL\0",
        2 => b"GOLOMB\0",
        3 => b"HUFFMAN\0",
        4 => b"BYTE_ARRAY_LEN\0",
        5 => b"BYTE_ARRAY_STOP\0",
        6 => b"BETA\0",
        7 => b"SUBEXP\0",
        8 => b"GOLOMB_RICE\0",
        9 => b"GAMMA\0",
        41 => b"VARINT_UNSIGNED\0",
        42 => b"VARINT_SIGNED\0",
        43 => b"CONST_BYTE\0",
        44 => b"CONST_INT\0",
        _ => b"?\0",
    };
    s.as_ptr().cast::<c_char>().cast_mut()
}

pub fn cram_cram_io_c_2341_cram_block_method2str(m: c_int) -> *mut c_char {
    let s: &'static [u8] = match m {
        -1 => b"?\0",
        0 => b"RAW\0",
        1 => b"GZIP\0",
        2 => b"BZIP2\0",
        3 => b"LZMA\0",
        4 => b"RANS0\0",
        5 => b"RANS_PR0\0",
        6 => b"ARITH_PR0\0",
        7 => b"FQZ\0",
        8 => b"TOK3_R\0",
        11 => b"GZIP_RLE\0",
        12 => b"GZIP_1\0",
        13 => b"FQZ_b\0",
        14 => b"FQZ_c\0",
        15 => b"FQZ_d\0",
        16 => b"RANS1\0",
        17 => b"RANS_PR1\0",
        18 => b"RANS_PR64\0",
        19 => b"RANS_PR9\0",
        20 => b"RANS_PR128\0",
        21 => b"RANS_PR129\0",
        22 => b"RANS_PR192\0",
        23 => b"RANS_PR193\0",
        24 => b"TOK3_A\0",
        25 => b"ARITH_PR1\0",
        26 => b"ARITH_PR64\0",
        27 => b"ARITH_PR9\0",
        28 => b"ARITH_PR128\0",
        29 => b"ARITH_PR129\0",
        30 => b"ARITH_PR192\0",
        31 => b"ARITH_PR193\0",
        _ => b"?\0",
    };
    s.as_ptr().cast::<c_char>().cast_mut()
}

pub fn cram_cram_io_c_2378_cram_content_type2str(t: hts_sys::cram_content_type) -> *mut c_char {
    let s: &'static [u8] = match t {
        0 => b"FILE_HEADER\0",
        1 => b"COMPRESSION_HEADER\0",
        2 => b"MAPPED_SLICE\0",
        3 => b"UNMAPPED_SLICE\0",
        4 => b"EXTERNAL\0",
        5 => b"CORE\0",
        _ => b"?\0",
    };
    s.as_ptr().cast::<c_char>().cast_mut()
}

pub unsafe fn cram_cram_io_c_2873_is_directory(fn_: *mut c_char) -> c_int {
    let mut buf = std::mem::MaybeUninit::<libc::stat>::uninit();
    if libc::stat(fn_, buf.as_mut_ptr()) != 0 {
        return 0;
    }
    let buf = buf.assume_init();
    ((buf.st_mode & libc::S_IFMT) == libc::S_IFDIR) as c_int
}

pub unsafe fn cram_open_trace_file_c_90_is_file(fn_: *mut c_char) -> c_int {
    let mut buf = std::mem::MaybeUninit::<libc::stat>::uninit();
    if libc::stat(fn_, buf.as_mut_ptr()) != 0 {
        return 0;
    }
    let buf = buf.assume_init();
    ((buf.st_mode & libc::S_IFMT) == libc::S_IFREG) as c_int
}

pub unsafe fn cram_open_trace_file_c_108_tokenise_search_path(
    searchpath: *const c_char,
) -> *mut c_char {
    let path_sep = if cfg!(windows) { b';' } else { b':' };
    let searchpath = if searchpath.is_null() {
        c"".as_ptr()
    } else {
        searchpath
    };
    let len = libc::strlen(searchpath);
    let newsearch = malloc((len + 5) as u64).cast::<c_char>();
    if newsearch.is_null() {
        return std::ptr::null_mut();
    }

    let mut i = 0usize;
    let mut j = 0usize;
    while i < len {
        let cur = *searchpath.add(i) as u8;
        if i < len - 1 && cur == b':' && *searchpath.add(i + 1) as u8 == b':' {
            *newsearch.add(j) = b':' as c_char;
            j += 1;
            i += 2;
            continue;
        }

        if path_sep == b':'
            && (i == 0 || *searchpath.add(i - 1) as u8 == b':')
            && (libc::strncmp(searchpath.add(i), c"http:".as_ptr(), 5) == 0
                || libc::strncmp(searchpath.add(i), c"https:".as_ptr(), 6) == 0
                || libc::strncmp(searchpath.add(i), c"ftp:".as_ptr(), 4) == 0
                || libc::strncmp(searchpath.add(i), c"|http:".as_ptr(), 6) == 0
                || libc::strncmp(searchpath.add(i), c"|https:".as_ptr(), 7) == 0
                || libc::strncmp(searchpath.add(i), c"|ftp:".as_ptr(), 5) == 0
                || libc::strncmp(searchpath.add(i), c"URL=http:".as_ptr(), 9) == 0
                || libc::strncmp(searchpath.add(i), c"URL=https:".as_ptr(), 10) == 0
                || libc::strncmp(searchpath.add(i), c"URL=ftp:".as_ptr(), 8) == 0)
        {
            loop {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                let was_colon = *searchpath.add(i) as u8 == b':';
                i += 1;
                if i >= len || was_colon {
                    break;
                }
            }
            if *searchpath.add(i) as u8 == b':' {
                i += 1;
            }
            if *searchpath.add(i) as u8 == b'/' {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                i += 1;
            }
            if *searchpath.add(i) as u8 == b'/' {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                i += 1;
            }
            loop {
                *newsearch.add(j) = *searchpath.add(i);
                j += 1;
                i += 1;
                if i >= len || *searchpath.add(i) as u8 == b':' || *searchpath.add(i) as u8 == b'/'
                {
                    break;
                }
            }
            *newsearch.add(j) = *searchpath.add(i);
            j += 1;
            i += 1;
            if *searchpath.add(i) as u8 == b':' {
                i += 1;
            }
        }

        if *searchpath.add(i) as u8 == path_sep {
            if j != 0 && *newsearch.add(j - 1) != 0 {
                *newsearch.add(j) = 0;
                j += 1;
            }
        } else {
            *newsearch.add(j) = *searchpath.add(i);
            j += 1;
        }
        i += 1;
    }

    if j != 0 {
        *newsearch.add(j) = 0;
        j += 1;
    }
    *newsearch.add(j) = b'.' as c_char;
    j += 1;
    *newsearch.add(j) = b'/' as c_char;
    j += 1;
    *newsearch.add(j) = 0;
    j += 1;
    *newsearch.add(j) = 0;

    newsearch
}

pub unsafe fn cram_open_trace_file_c_182_find_file_url(
    file: *const c_char,
    url: *mut c_char,
) -> *mut mFILE {
    let path = cram_open_trace_file_c_230_expand_path(file, url.cast_const(), 1);
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let hf = crate::htslib_rs::hfile::hopen(path, c"r".as_ptr());
    if hf.is_null() {
        free(path.cast());
        return std::ptr::null_mut();
    }

    let mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if mf.is_null() {
        crate::htslib_rs::hfile::hclose_abruptly(hf);
        free(path.cast());
        return std::ptr::null_mut();
    }

    let mut buf = [0u8; 8192];
    loop {
        let len = crate::htslib_rs::hfile::hread2(hf, buf.as_mut_ptr().cast(), buf.len(), 0);
        if len <= 0 {
            if crate::htslib_rs::hfile::hclose(hf) < 0 || len < 0 {
                cram_mFILE_c_408_mfdestroy(mf);
                free(path.cast());
                return std::ptr::null_mut();
            }
            break;
        }
        if cram_mFILE_c_527_mfwrite(buf.as_mut_ptr().cast(), len as usize, 1, mf) == 0 {
            crate::htslib_rs::hfile::hclose_abruptly(hf);
            cram_mFILE_c_408_mfdestroy(mf);
            free(path.cast());
            return std::ptr::null_mut();
        }
    }

    free(path.cast());
    cram_mFILE_c_475_mrewind(mf);
    mf
}

pub unsafe fn cram_open_trace_file_c_230_expand_path(
    mut file: *const c_char,
    mut dirname: *const c_char,
    max_s_digits: c_int,
) -> *mut c_char {
    let mut len = libc::strlen(dirname);
    let mut lenf = libc::strlen(file);
    let mut end_dirname = dirname.add(len);
    let path = malloc((len + lenf + 2) as u64).cast::<c_char>();
    if path.is_null() {
        return std::ptr::null_mut();
    }

    while len > 1 && *dirname.add(len - 1) as u8 == b'/' {
        len -= 1;
        end_dirname = end_dirname.sub(1);
    }

    if *file as u8 == b'/' || (len == 1 && *dirname as u8 == b'.') {
        memcpy(path.cast(), file.cast(), (lenf + 1) as u64);
    } else {
        let mut path_end = path;
        loop {
            let cp = libc::strchr(dirname, b'%' as c_int);
            if cp.is_null() {
                break;
            }

            let mut endp: *mut c_char = std::ptr::null_mut();
            let l = libc::strtol(cp.add(1), &mut endp, 10);
            if *endp as u8 != b's' || l < 0 || endp.offset_from(cp) - 1 > max_s_digits as isize {
                let mut e = endp.add(1).cast_const();
                if e > end_dirname {
                    e = end_dirname;
                }
                let n = e.offset_from(dirname) as usize;
                memcpy(path_end.cast(), dirname.cast(), n as u64);
                path_end = path_end.add(n);
                dirname = e;
                continue;
            }

            let n = cp.cast_const().offset_from(dirname) as usize;
            memcpy(path_end.cast(), dirname.cast(), n as u64);
            path_end = path_end.add(n);

            let to_copy = if l > 0 {
                std::cmp::min(lenf, l as usize)
            } else {
                lenf
            };
            memcpy(path_end.cast(), file.cast(), to_copy as u64);
            path_end = path_end.add(to_copy);
            file = file.add(to_copy);
            lenf -= to_copy;

            dirname = endp.add(1);
        }

        if dirname < end_dirname {
            let n = end_dirname.offset_from(dirname) as usize;
            memcpy(path_end.cast(), dirname.cast(), n as u64);
            path_end = path_end.add(n);
        }

        if *file != 0 {
            if path_end > path && *path_end.sub(1) as u8 != b'/' {
                *path_end = b'/' as c_char;
                path_end = path_end.add(1);
            }
            memcpy(path_end.cast(), file.cast(), lenf as u64);
            path_end = path_end.add(lenf);
        }
        *path_end = 0;
    }

    path
}

pub unsafe fn cram_open_trace_file_c_433_find_path(
    file: *const c_char,
    mut path: *const c_char,
) -> *mut c_char {
    if path.is_null() {
        path = libc::getenv(c"RAWDATA".as_ptr());
    }
    let newsearch = cram_open_trace_file_c_108_tokenise_search_path(path);
    if newsearch.is_null() {
        return std::ptr::null_mut();
    }

    let mut ele = newsearch;
    while *ele != 0 {
        let ele2 = if *ele as u8 == b'|' { ele.add(1) } else { ele };

        if libc::strncmp(ele2, c"URL=".as_ptr(), 4) != 0
            && libc::strncmp(ele2, c"http:".as_ptr(), 5) != 0
            && libc::strncmp(ele2, c"https:".as_ptr(), 6) != 0
            && libc::strncmp(ele2, c"ftp:".as_ptr(), 4) != 0
        {
            let outpath = cram_open_trace_file_c_230_expand_path(file, ele2, c_int::MAX);
            if cram_open_trace_file_c_90_is_file(outpath) != 0 {
                free(newsearch.cast());
                return outpath;
            }
            free(outpath.cast());
        }

        ele = ele.add(libc::strlen(ele) + 1);
    }

    free(newsearch.cast());
    std::ptr::null_mut()
}

pub unsafe fn cram_open_trace_file_c_314_find_file_dir(
    file: *const c_char,
    dirname: *mut c_char,
) -> *mut mFILE {
    let path = cram_open_trace_file_c_230_expand_path(file, dirname.cast_const(), c_int::MAX);
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let mf = if cram_open_trace_file_c_90_is_file(path) != 0 {
        cram_mFILE_c_347_mfopen(path, c"rbm".as_ptr())
    } else {
        std::ptr::null_mut()
    };
    free(path.cast());
    mf
}

pub unsafe fn cram_open_trace_file_c_352_open_path_mfile(
    file: *const c_char,
    mut path: *mut c_char,
    relative_to: *mut c_char,
    local: *mut c_int,
) -> *mut mFILE {
    if !local.is_null() {
        *local = 1;
    }

    if path.is_null() {
        path = libc::getenv(c"RAWDATA".as_ptr());
    }
    let newsearch = cram_open_trace_file_c_108_tokenise_search_path(path);
    if newsearch.is_null() {
        return std::ptr::null_mut();
    }

    let mut ele = newsearch;
    while *ele != 0 {
        let ele2 = if *ele as u8 == b'|' { ele.add(1) } else { ele };

        if libc::strncmp(ele2, c"URL=".as_ptr(), 4) == 0 {
            let fp = cram_open_trace_file_c_182_find_file_url(file, ele2.add(4));
            if !fp.is_null() {
                if !local.is_null() {
                    *local = if libc::strncmp(ele2.add(4), c"file:".as_ptr(), 5) == 0 {
                        1
                    } else {
                        0
                    };
                }
                free(newsearch.cast());
                return fp;
            }
        } else if crate::htslib_rs::hfile::hisremote(ele2) != 0 {
            let fp = cram_open_trace_file_c_182_find_file_url(file, ele2);
            if !fp.is_null() {
                if !local.is_null() {
                    *local = 0;
                }
                free(newsearch.cast());
                return fp;
            }
        } else {
            let fp = cram_open_trace_file_c_314_find_file_dir(file, ele2);
            if !fp.is_null() {
                free(newsearch.cast());
                return fp;
            }
        }

        ele = ele.add(libc::strlen(ele) + 1);
    }

    free(newsearch.cast());

    if !relative_to.is_null() {
        let mut relative_path = [0 as c_char; libc::PATH_MAX as usize + 1];
        libc::strcpy(relative_path.as_mut_ptr(), relative_to);
        let cp = libc::strrchr(relative_path.as_mut_ptr(), b'/' as c_int);
        if !cp.is_null() {
            *cp = 0;
        }
        let fp = cram_open_trace_file_c_314_find_file_dir(file, relative_path.as_mut_ptr());
        if !fp.is_null() {
            return fp;
        }
    }

    std::ptr::null_mut()
}

pub unsafe fn cram_cram_io_c_2884_expand_cache_path(
    mut path: *mut c_char,
    mut dir: *mut c_char,
    mut fn_: *const c_char,
) -> c_int {
    let start = path;
    let mut sz = libc::PATH_MAX as usize;

    loop {
        let cp0 = libc::strchr(dir, b'%' as c_int);
        if cp0.is_null() {
            break;
        }
        let mut cp = cp0;
        let dir_len = cp.offset_from(dir) as usize;
        if dir_len >= sz {
            return -1;
        }
        libc::strncpy(path, dir, dir_len);
        path = path.add(dir_len);
        sz -= dir_len;

        cp = cp.add(1);
        if *cp == b's' as c_char {
            let len = libc::strlen(fn_);
            if len >= sz {
                return -1;
            }
            libc::strcpy(path, fn_);
            path = path.add(len);
            sz -= len;
            fn_ = fn_.add(len);
            cp = cp.add(1);
        } else if *cp >= b'0' as c_char && *cp <= b'9' as c_char {
            let mut endp: *mut c_char = std::ptr::null_mut();
            let mut l = libc::strtol(cp, &mut endp, 10);
            let fn_len = libc::strlen(fn_) as libc::c_long;
            if l > fn_len {
                l = fn_len;
            }
            if *endp == b's' as c_char {
                if l as usize >= sz {
                    return -1;
                }
                libc::strncpy(path, fn_, l as usize);
                path = path.add(l as usize);
                fn_ = fn_.add(l as usize);
                sz -= l as usize;
                *path = 0;
                cp = endp.add(1);
            } else {
                if sz < 3 {
                    return -1;
                }
                *path = b'%' as c_char;
                path = path.add(1);
                *path = *cp;
                path = path.add(1);
                cp = cp.add(1);
            }
        } else {
            if sz < 3 {
                return -1;
            }
            *path = b'%' as c_char;
            path = path.add(1);
            *path = *cp;
            path = path.add(1);
            cp = cp.add(1);
        }
        dir = cp;
    }

    let mut len = libc::strlen(dir);
    if len >= sz {
        return -1;
    }
    libc::strcpy(path, dir);
    path = path.add(len);
    sz -= len;

    len = libc::strlen(fn_)
        + if *fn_ != 0 && path > start && *path.sub(1) != b'/' as c_char {
            1
        } else {
            0
        };
    if len >= sz {
        return -1;
    }
    if *fn_ != 0 && path > start && *path.sub(1) != b'/' as c_char {
        *path = b'/' as c_char;
        path = path.add(1);
    }
    libc::strcpy(path, fn_);
    0
}

pub unsafe fn cram_cram_io_c_2947_mkdir_prefix(path: *mut c_char, mode: c_int) {
    let cp = libc::strrchr(path, b'/' as c_int);
    if cp.is_null() {
        return;
    }

    *cp = 0;
    if cram_cram_io_c_2873_is_directory(path) != 0 {
        *cp = b'/' as c_char;
        return;
    }

    if libc::mkdir(path, mode as libc::mode_t) == 0 {
        libc::chmod(path, mode as libc::mode_t);
        *cp = b'/' as c_char;
        return;
    }

    cram_cram_io_c_2947_mkdir_prefix(path, mode);
    libc::mkdir(path, mode as libc::mode_t);
    libc::chmod(path, mode as libc::mode_t);
    *cp = b'/' as c_char;
}

pub unsafe fn cram_cram_io_c_3695_free_bam_list(bams: *mut *mut bam1_t, max_rec: c_int) {
    for i in 0..max_rec {
        bam_destroy1(*bams.add(i as usize));
    }
    free(bams.cast());
}

pub unsafe fn cram_cram_io_c_4850_full_path(out: *mut c_char, in_: *mut c_char) {
    let in_l = libc::strlen(in_);
    if hisremote(in_) != 0 {
        if in_l > libc::PATH_MAX as usize {
            let msg = std::ffi::CString::new(format!(
                "Reference path is longer than {}",
                libc::PATH_MAX
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_ERROR, c"full_path".as_ptr(), msg.as_ptr());
            return;
        }
        libc::strncpy(out, in_, libc::PATH_MAX as usize - 1);
        *out.add(libc::PATH_MAX as usize - 1) = 0;
        return;
    }

    let is_windows_abs = in_l > 3
        && toupper_c(*in_) >= b'A' as c_char
        && toupper_c(*in_) <= b'Z' as c_char
        && *in_.add(1) == b':' as c_char
        && (*in_.add(2) == b'/' as c_char || *in_.add(2) == b'\\' as c_char);

    if *in_ == b'/' as c_char || is_windows_abs {
        libc::strncpy(out, in_, libc::PATH_MAX as usize - 1);
        *out.add(libc::PATH_MAX as usize - 1) = 0;
    } else {
        if libc::getcwd(out, libc::PATH_MAX as usize).is_null() {
            libc::strncpy(out, in_, libc::PATH_MAX as usize - 1);
            *out.add(libc::PATH_MAX as usize - 1) = 0;
            return;
        }

        let len = libc::strlen(out);
        if len + 1 + in_l >= libc::PATH_MAX as usize {
            libc::strncpy(out, in_, libc::PATH_MAX as usize - 1);
            *out.add(libc::PATH_MAX as usize - 1) = 0;
            return;
        }

        libc::snprintf(
            out.add(len),
            libc::PATH_MAX as usize - len,
            c"/%s".as_ptr(),
            in_,
        );
    }
}

pub unsafe fn cram_mFILE_c_75_mfload(
    fp: *mut libc::FILE,
    mut fn_: *const c_char,
    size: *mut usize,
    _binary: c_int,
) -> *mut c_char {
    let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
    let mut data = std::ptr::null_mut::<c_char>();
    let mut allocated = 0usize;
    let mut used = 0usize;
    let mut bufsize = 8192usize;

    if !fn_.is_null() && libc::stat(fn_, sb.as_mut_ptr()) != -1 {
        let sb = sb.assume_init();
        allocated = sb.st_size as usize;
        data = malloc(allocated as u64).cast::<c_char>();
        if data.is_null() {
            return std::ptr::null_mut();
        }
        bufsize = sb.st_size as usize;

        loop {
            if used + bufsize > allocated {
                allocated += bufsize;
                let datan = realloc(data.cast(), allocated as u64).cast::<c_char>();
                if datan.is_null() {
                    free(data.cast());
                    return std::ptr::null_mut();
                }
                data = datan;
            }
            let len = libc::fread(data.add(used).cast(), 1, allocated - used, fp);
            if len > 0 {
                used += len;
            }
            if libc::feof(fp) != 0 || used >= sb.st_size as usize {
                break;
            }
        }
    } else {
        fn_ = std::ptr::null();
        loop {
            if used + bufsize > allocated {
                allocated += bufsize;
                let datan = realloc(data.cast(), allocated as u64).cast::<c_char>();
                if datan.is_null() {
                    free(data.cast());
                    return std::ptr::null_mut();
                }
                data = datan;
            }
            let len = libc::fread(data.add(used).cast(), 1, allocated - used, fp);
            if len > 0 {
                used += len;
            }
            if libc::feof(fp) != 0 || !fn_.is_null() {
                break;
            }
        }
    }

    *size = used;
    data
}

pub unsafe fn cram_mFILE_c_127_mfmmap(
    mf: *mut mFILE,
    fp: *mut libc::FILE,
    fn_: *const c_char,
) -> c_int {
    let mut sb = std::mem::MaybeUninit::<libc::stat>::uninit();
    if libc::stat(fn_, sb.as_mut_ptr()) != 0 {
        return -1;
    }
    let sb = sb.assume_init();
    (*mf).size = sb.st_size as usize;
    let data = libc::mmap(
        std::ptr::null_mut(),
        (*mf).size,
        libc::PROT_READ,
        libc::MAP_SHARED,
        libc::fileno(fp),
        0,
    );
    if data.is_null() || data == libc::MAP_FAILED {
        return -1;
    }

    (*mf).data = data.cast::<c_char>();
    (*mf).alloced = 0;
    0
}

pub unsafe fn cram_mFILE_c_151_mstdin() -> *mut mFILE {
    if !M_CHANNEL[0].is_null() {
        return M_CHANNEL[0];
    }

    M_CHANNEL[0] = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if M_CHANNEL[0].is_null() {
        return std::ptr::null_mut();
    }
    (*M_CHANNEL[0]).fp = HTSLIB_STDIN;
    M_CHANNEL[0]
}

pub unsafe fn cram_mFILE_c_161_init_mstdin() {
    if DONE_STDIN != 0 {
        return;
    }

    (*M_CHANNEL[0]).data =
        cram_mFILE_c_75_mfload(HTSLIB_STDIN, std::ptr::null(), &mut (*M_CHANNEL[0]).size, 1);
    (*M_CHANNEL[0]).mode = MF_READ;
    DONE_STDIN = 1;
}

pub unsafe fn cram_mFILE_c_176_mstdout() -> *mut mFILE {
    if !M_CHANNEL[1].is_null() {
        return M_CHANNEL[1];
    }

    M_CHANNEL[1] = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if M_CHANNEL[1].is_null() {
        return std::ptr::null_mut();
    }
    (*M_CHANNEL[1]).fp = HTSLIB_STDOUT;
    (*M_CHANNEL[1]).mode = MF_WRITE;
    M_CHANNEL[1]
}

pub unsafe fn cram_mFILE_c_192_mstderr() -> *mut mFILE {
    if !M_CHANNEL[2].is_null() {
        return M_CHANNEL[2];
    }

    M_CHANNEL[2] = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
    if M_CHANNEL[2].is_null() {
        return std::ptr::null_mut();
    }
    (*M_CHANNEL[2]).fp = HTSLIB_STDERR;
    (*M_CHANNEL[2]).mode = MF_WRITE;
    M_CHANNEL[2]
}

pub unsafe fn cram_mFILE_c_207_mfcreate(data: *mut c_char, size: c_int) -> *mut mFILE {
    let mf = malloc(std::mem::size_of::<mFILE>() as u64).cast::<mFILE>();
    if mf.is_null() {
        return std::ptr::null_mut();
    }
    (*mf).fp = std::ptr::null_mut();
    (*mf).data = data;
    (*mf).alloced = size as usize;
    (*mf).size = size as usize;
    (*mf).eof = 0;
    (*mf).offset = 0;
    (*mf).flush_pos = 0;
    (*mf).mode = MF_READ | MF_WRITE;
    mf
}

pub unsafe fn cram_mFILE_c_225_mfrecreate(mf: *mut mFILE, data: *mut c_char, size: c_int) {
    if !(*mf).data.is_null() {
        free((*mf).data.cast());
    }
    (*mf).data = data;
    (*mf).size = size as usize;
    (*mf).alloced = size as usize;
    (*mf).eof = 0;
    (*mf).offset = 0;
    (*mf).flush_pos = 0;
}

pub unsafe fn cram_mFILE_c_246_mfcreate_from(
    path: *const c_char,
    mode_str: *const c_char,
    fp: *mut libc::FILE,
) -> *mut mFILE {
    let mf = cram_mFILE_c_264_mfreopen(path, mode_str, fp);
    if mf.is_null() {
        return std::ptr::null_mut();
    }
    (*mf).fp = std::ptr::null_mut();
    mf
}

pub unsafe fn cram_mFILE_c_264_mfreopen(
    path: *const c_char,
    mode_str: *const c_char,
    fp: *mut libc::FILE,
) -> *mut mFILE {
    let mut r = 0;
    let mut w = 0;
    let mut a = 0;
    let mut b = 0;
    let mut x = 0;
    let mut mode = 0;

    if !libc::strchr(mode_str, b'r' as c_int).is_null() {
        r = 1;
        mode |= MF_READ;
    }
    if !libc::strchr(mode_str, b'w' as c_int).is_null() {
        w = 1;
        mode |= MF_WRITE | MF_TRUNC;
    }
    if !libc::strchr(mode_str, b'a' as c_int).is_null() {
        w = 1;
        a = 1;
        mode |= MF_WRITE | MF_APPEND;
    }
    if !libc::strchr(mode_str, b'b' as c_int).is_null() {
        b = 1;
        mode |= MF_BINARY;
    }
    if !libc::strchr(mode_str, b'x' as c_int).is_null() {
        x = 1;
    }
    if !libc::strchr(mode_str, b'+' as c_int).is_null() {
        w = 1;
        mode |= MF_READ | MF_WRITE;
        if a != 0 {
            r = 1;
        }
    }
    if !libc::strchr(mode_str, b'm' as c_int).is_null() && w == 0 {
        mode |= MF_MMAP;
    }

    let mf;
    if r != 0 {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
        if (mode & MF_TRUNC) == 0 {
            if (mode & MF_MMAP) != 0 && cram_mFILE_c_127_mfmmap(mf, fp, path) == -1 {
                (*mf).data = std::ptr::null_mut();
                mode &= !MF_MMAP;
            }
            if (*mf).data.is_null() {
                (*mf).data = cram_mFILE_c_75_mfload(fp, path, &mut (*mf).size, b);
                if (*mf).data.is_null() {
                    free(mf.cast());
                    return std::ptr::null_mut();
                }
                (*mf).alloced = (*mf).size;
                if a == 0 {
                    libc::fseek(fp, 0, libc::SEEK_SET);
                }
            }
        }
    } else if w != 0 {
        mf = cram_mFILE_c_207_mfcreate(std::ptr::null_mut(), 0);
        if mf.is_null() {
            return std::ptr::null_mut();
        }
    } else {
        return std::ptr::null_mut();
    }

    (*mf).fp = fp;
    (*mf).mode = mode;
    if x != 0 {
        (*mf).mode |= MF_MODEX;
    }
    if a != 0 {
        (*mf).flush_pos = (*mf).size;
        libc::fseek(fp, 0, libc::SEEK_END);
    }

    mf
}

pub unsafe fn cram_mFILE_c_347_mfopen(path: *const c_char, mode: *const c_char) -> *mut mFILE {
    let fp = libc::fopen(path, mode);
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    cram_mFILE_c_264_mfreopen(path, mode, fp)
}

pub unsafe fn cram_mFILE_c_361_mfclose(mf: *mut mFILE) -> c_int {
    if mf.is_null() {
        return -1;
    }
    cram_mFILE_c_607_mfflush(mf);
    if ((*mf).mode & MF_MMAP) != 0 && !(*mf).data.is_null() {
        libc::munmap((*mf).data.cast(), (*mf).size);
        (*mf).data = std::ptr::null_mut();
    }
    if !(*mf).fp.is_null() {
        libc::fclose((*mf).fp);
    }
    cram_mFILE_c_408_mfdestroy(mf);
    0
}

pub unsafe fn cram_mFILE_c_389_mfdetach(mf: *mut mFILE) -> c_int {
    if mf.is_null() {
        return -1;
    }
    if cram_mFILE_c_607_mfflush(mf) != 0 {
        return -1;
    }
    if ((*mf).mode & MF_MMAP) != 0 {
        return -1;
    }
    if !(*mf).fp.is_null() {
        libc::fclose((*mf).fp);
        (*mf).fp = std::ptr::null_mut();
    }
    0
}

pub unsafe fn cram_mFILE_c_408_mfdestroy(mf: *mut mFILE) -> c_int {
    if mf.is_null() {
        return -1;
    }
    if !(*mf).data.is_null() {
        free((*mf).data.cast());
    }
    free(mf.cast());
    0
}

pub unsafe fn cram_mFILE_c_428_mfsteal(mf: *mut mFILE, size_out: *mut usize) -> *mut c_void {
    if mf.is_null() {
        return std::ptr::null_mut();
    }
    let data = (*mf).data;
    if !size_out.is_null() {
        *size_out = (*mf).size;
    }
    if cram_mFILE_c_389_mfdetach(mf) != 0 {
        return std::ptr::null_mut();
    }
    (*mf).data = std::ptr::null_mut();
    cram_mFILE_c_408_mfdestroy(mf);
    data.cast()
}

pub unsafe fn cram_mFILE_c_451_mfseek(
    mf: *mut mFILE,
    offset: libc::c_long,
    whence: c_int,
) -> c_int {
    match whence {
        libc::SEEK_SET => {
            (*mf).offset = offset as usize;
        }
        libc::SEEK_CUR => {
            (*mf).offset = (*mf).offset.wrapping_add(offset as usize);
        }
        libc::SEEK_END => {
            (*mf).offset = (*mf).size.wrapping_add(offset as usize);
        }
        _ => {
            *__errno_location() = EINVAL;
            return -1;
        }
    }

    (*mf).eof = 0;
    0
}

pub unsafe fn cram_mFILE_c_471_mftell(mf: *mut mFILE) -> libc::c_long {
    (*mf).offset as libc::c_long
}

pub unsafe fn cram_mFILE_c_475_mrewind(mf: *mut mFILE) {
    (*mf).offset = 0;
    (*mf).eof = 0;
}

pub unsafe fn cram_mFILE_c_488_mftruncate(mf: *mut mFILE, offset: libc::c_long) {
    (*mf).size = if offset != -1 {
        offset as usize
    } else {
        (*mf).offset
    };
    if (*mf).offset > (*mf).size {
        (*mf).offset = (*mf).size;
    }
}

pub unsafe fn cram_mFILE_c_494_mfeof(mf: *mut mFILE) -> c_int {
    (*mf).eof
}

pub unsafe fn cram_mFILE_c_502_mfread(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    mf: *mut mFILE,
) -> usize {
    if (*mf).size <= (*mf).offset {
        return 0;
    }

    let wanted = size.wrapping_mul(nmemb);
    let available = (*mf).size - (*mf).offset;
    let len = if wanted <= available {
        wanted
    } else {
        available
    };
    if size == 0 {
        return 0;
    }

    memcpy(
        ptr,
        (*mf).data.add((*mf).offset).cast::<c_void>(),
        len as u64,
    );
    (*mf).offset += len;

    if len != wanted {
        (*mf).eof = 1;
    }

    len / size
}

pub unsafe fn cram_mFILE_c_527_mfwrite(
    ptr: *mut c_void,
    size: usize,
    nmemb: usize,
    mf: *mut mFILE,
) -> usize {
    if ((*mf).mode & MF_WRITE) == 0 {
        return 0;
    }

    if ((*mf).mode & MF_APPEND) != 0 {
        (*mf).offset = (*mf).size;
    }

    let wanted = size.wrapping_mul(nmemb);
    while wanted + (*mf).offset > (*mf).alloced {
        let new_alloced = if (*mf).alloced != 0 {
            (*mf).alloced * 2
        } else {
            1024
        };
        let new_data = realloc((*mf).data.cast(), new_alloced as u64).cast::<c_char>();
        if new_data.is_null() {
            return 0;
        }
        (*mf).alloced = new_alloced;
        (*mf).data = new_data;
    }

    if (*mf).offset < (*mf).flush_pos {
        (*mf).flush_pos = (*mf).offset;
    }

    memcpy(
        (*mf).data.add((*mf).offset).cast::<c_void>(),
        ptr,
        wanted as u64,
    );
    (*mf).offset += wanted;
    if (*mf).size < (*mf).offset {
        (*mf).size = (*mf).offset;
    }

    nmemb
}

pub unsafe fn cram_mFILE_c_557_mfgetc(mf: *mut mFILE) -> c_int {
    if (*mf).offset < (*mf).size {
        let c = *(*mf).data.add((*mf).offset) as u8;
        (*mf).offset += 1;
        return c as c_int;
    }

    (*mf).eof = 1;
    -1
}

pub unsafe fn cram_mFILE_c_567_mungetc(c: c_int, mf: *mut mFILE) -> c_int {
    if (*mf).offset > 0 {
        (*mf).offset -= 1;
        *(*mf).data.add((*mf).offset) = c as c_char;
        return c;
    }

    (*mf).eof = 1;
    -1
}

pub unsafe fn cram_mFILE_c_577_mfgets(s: *mut c_char, size: c_int, mf: *mut mFILE) -> *mut c_char {
    let mut i = 0;

    *s = 0;
    while i < size - 1 {
        if (*mf).offset < (*mf).size {
            *s.add(i as usize) = *(*mf).data.add((*mf).offset);
            (*mf).offset += 1;
            i += 1;
            if *s.add((i - 1) as usize) == b'\n' as c_char {
                break;
            }
        } else {
            (*mf).eof = 1;
            break;
        }
    }

    *s.add(i as usize) = 0;
    if i != 0 {
        s
    } else {
        std::ptr::null_mut()
    }
}

pub unsafe fn cram_mFILE_c_607_mfflush(mf: *mut mFILE) -> c_int {
    if (*mf).fp.is_null() {
        return 0;
    }

    if mf == M_CHANNEL[1] || mf == M_CHANNEL[2] {
        if (*mf).flush_pos < (*mf).size {
            let bytes = (*mf).size - (*mf).flush_pos;
            if libc::fwrite((*mf).data.add((*mf).flush_pos).cast(), 1, bytes, (*mf).fp) < bytes {
                return -1;
            }
            if libc::fflush((*mf).fp) != 0 {
                return -1;
            }
        }
        (*mf).offset = 0;
        (*mf).size = 0;
        (*mf).flush_pos = 0;
    }

    if ((*mf).mode & MF_WRITE) != 0 {
        if (*mf).flush_pos < (*mf).size {
            let bytes = (*mf).size - (*mf).flush_pos;
            if ((*mf).mode & MF_MODEX) == 0 {
                libc::fseek((*mf).fp, (*mf).flush_pos as libc::c_long, libc::SEEK_SET);
            }
            if libc::fwrite((*mf).data.add((*mf).flush_pos).cast(), 1, bytes, (*mf).fp) < bytes {
                return -1;
            }
            if libc::fflush((*mf).fp) != 0 {
                return -1;
            }
        }
        let pos = libc::ftell((*mf).fp);
        if pos != -1 && libc::ftruncate(libc::fileno((*mf).fp), pos) == -1 {
            return -1;
        }
        (*mf).flush_pos = (*mf).size;
    }

    0
}

pub unsafe fn cram_mFILE_c_656_mfascii(mf: *mut mFILE) {
    let mut p1 = 1usize;
    let mut p2 = 1usize;

    while p1 < (*mf).size {
        if *(*mf).data.add(p1) == b'\n' as c_char && *(*mf).data.add(p1 - 1) == b'\r' as c_char {
            p2 -= 1;
        }
        *(*mf).data.add(p2) = *(*mf).data.add(p1);
        p1 += 1;
        p2 += 1;
    }
    (*mf).size = p2;

    (*mf).offset = 0;
    (*mf).flush_pos = 0;
}

pub fn cram_pooled_alloc_c_47_next_power_2(mut v: u32) -> c_int {
    v = v.wrapping_sub(1);
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v = v.wrapping_add(1);
    v as c_int
}

pub unsafe fn cram_pooled_alloc_c_64_pool_create(dsize: usize) -> *mut pool_alloc_t {
    let p = malloc(std::mem::size_of::<pool_alloc_t>() as u64).cast::<pool_alloc_t>();
    if p.is_null() {
        return std::ptr::null_mut();
    }

    let mut rounded = (dsize + std::mem::size_of::<*mut c_void>() - 1)
        & !(std::mem::size_of::<*mut c_void>() - 1);
    if rounded < std::mem::size_of::<*mut c_void>() {
        rounded = std::mem::size_of::<*mut c_void>();
    }
    (*p).dsize = rounded;
    (*p).psize = std::cmp::min(
        POOLED_ALLOC_PSIZE,
        cram_pooled_alloc_c_47_next_power_2(((*p).dsize * 1024) as u32) as usize,
    );
    (*p).npools = 0;
    (*p).pools = std::ptr::null_mut();
    (*p).free = std::ptr::null_mut();

    p
}

pub unsafe fn cram_pooled_alloc_c_84_pool_destroy(p: *mut pool_alloc_t) {
    for i in 0..(*p).npools {
        free((*(*p).pools.add(i)).pool);
    }
    free((*p).pools.cast());
    free(p.cast());
}

pub unsafe fn cram_pooled_alloc_c_96_new_pool(p: *mut pool_alloc_t) -> *mut pool_t {
    let n = (*p).psize / (*p).dsize;
    let pools = realloc(
        (*p).pools.cast(),
        ((*p).npools + 1) as u64 * std::mem::size_of::<pool_t>() as u64,
    )
    .cast::<pool_t>();
    if pools.is_null() {
        return std::ptr::null_mut();
    }
    (*p).pools = pools;
    let pool = (*p).pools.add((*p).npools);

    (*pool).pool = malloc((n * (*p).dsize) as u64);
    if (*pool).pool.is_null() {
        return std::ptr::null_mut();
    }
    (*pool).used = 0;
    (*p).npools += 1;

    pool
}

pub unsafe fn cram_pooled_alloc_c_115_pool_alloc(p: *mut pool_alloc_t) -> *mut c_void {
    if !(*p).free.is_null() {
        let ret = (*p).free;
        (*p).free = *(ret.cast::<*mut c_void>());
        return ret;
    }

    if (*p).npools != 0 {
        let pool = (*p).pools.add((*p).npools - 1);
        if (*pool).used + (*p).dsize < (*p).psize {
            let ret = (*pool).pool.cast::<u8>().add((*pool).used).cast::<c_void>();
            (*pool).used += (*p).dsize;
            return ret;
        }
    }

    let pool = cram_pooled_alloc_c_96_new_pool(p);
    if pool.is_null() {
        return std::ptr::null_mut();
    }
    (*pool).used = (*p).dsize;
    (*pool).pool
}

pub unsafe fn cram_pooled_alloc_c_144_pool_free(p: *mut pool_alloc_t, ptr: *mut c_void) {
    *(ptr.cast::<*mut c_void>()) = (*p).free;
    (*p).free = ptr;
}

pub unsafe fn cram_pooled_alloc_c_151_pool_alloc(p: *mut pool_alloc_t) -> *mut c_void {
    malloc((*p).dsize as u64)
}

pub unsafe fn cram_pooled_alloc_c_155_pool_free(_p: *mut pool_alloc_t, ptr: *mut c_void) {
    free(ptr);
}

#[repr(C)]
struct pooled_alloc_test_xyz {
    x: c_int,
    y: c_int,
    z: c_int,
}

pub unsafe fn cram_pooled_alloc_c_167_main() -> c_int {
    let p = cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<pooled_alloc_test_xyz>());
    if p.is_null() {
        return 1;
    }

    let np = 10000usize;
    let items = malloc((np * std::mem::size_of::<*mut pooled_alloc_test_xyz>()) as u64)
        .cast::<*mut pooled_alloc_test_xyz>();
    if items.is_null() {
        cram_pooled_alloc_c_84_pool_destroy(p);
        return 1;
    }

    for i in 0..np {
        let item = cram_pooled_alloc_c_115_pool_alloc(p).cast::<pooled_alloc_test_xyz>();
        if item.is_null() {
            free(items.cast());
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        }
        (*item).x = i as c_int;
        (*item).y = i as c_int + 1;
        (*item).z = i as c_int + 2;
        *items.add(i) = item;
    }

    for i in 0..np {
        let item = *items.add(i);
        if i % 3 != 0 {
            cram_pooled_alloc_c_144_pool_free(p, item.cast());
        }
    }

    for i in 0..np {
        let item = cram_pooled_alloc_c_115_pool_alloc(p).cast::<pooled_alloc_test_xyz>();
        if item.is_null() {
            free(items.cast());
            cram_pooled_alloc_c_84_pool_destroy(p);
            return 1;
        }
        (*item).x = 1_000_000 + i as c_int;
        (*item).y = 1_000_000 + i as c_int + 1;
        (*item).z = 1_000_000 + i as c_int + 2;
    }

    for i in 0..np {
        cram_pooled_alloc_c_144_pool_free(p, (*items.add(i)).cast());
    }

    free(items.cast());
    cram_pooled_alloc_c_84_pool_destroy(p);
    0
}

pub unsafe fn cram_cram_external_c_58_cram_fd_get_header(
    fd: *mut hts_sys::cram_fd,
) -> *mut hts_sys::sam_hdr_t {
    hts_sys::cram_fd_get_header(fd)
}

pub unsafe fn cram_cram_external_c_59_cram_fd_set_header(
    fd: *mut hts_sys::cram_fd,
    hdr: *mut hts_sys::sam_hdr_t,
) {
    hts_sys::cram_fd_set_header(fd, hdr);
}

pub unsafe fn cram_cram_external_c_61_cram_fd_get_version(fd: *mut hts_sys::cram_fd) -> c_int {
    hts_sys::cram_fd_get_version(fd)
}

pub unsafe fn cram_cram_external_c_62_cram_fd_set_version(fd: *mut hts_sys::cram_fd, vers: c_int) {
    hts_sys::cram_fd_set_version(fd, vers);
}

pub unsafe fn cram_cram_external_c_64_cram_major_vers(fd: *mut hts_sys::cram_fd) -> c_int {
    hts_sys::cram_major_vers(fd)
}

pub unsafe fn cram_cram_external_c_65_cram_minor_vers(fd: *mut hts_sys::cram_fd) -> c_int {
    hts_sys::cram_minor_vers(fd)
}

pub unsafe fn cram_cram_external_c_67_cram_fd_get_fp(
    fd: *mut hts_sys::cram_fd,
) -> *mut hts_sys::hFILE {
    hts_sys::cram_fd_get_fp(fd)
}

pub unsafe fn cram_cram_external_c_68_cram_fd_set_fp(
    fd: *mut hts_sys::cram_fd,
    fp: *mut hts_sys::hFILE,
) {
    hts_sys::cram_fd_set_fp(fd, fp);
}

pub unsafe fn cram_cram_external_c_75_cram_container_get_length(
    c: *mut hts_sys::cram_container,
) -> i32 {
    (*c.cast::<cram_container_layout>()).length
}

pub unsafe fn cram_cram_external_c_79_cram_container_set_length(
    c: *mut hts_sys::cram_container,
    length: i32,
) {
    (*c.cast::<cram_container_layout>()).length = length;
}

pub unsafe fn cram_cram_external_c_84_cram_container_get_num_blocks(
    c: *mut hts_sys::cram_container,
) -> i32 {
    (*c.cast::<cram_container_layout>()).num_blocks
}

pub unsafe fn cram_cram_external_c_88_cram_container_set_num_blocks(
    c: *mut hts_sys::cram_container,
    num_blocks: i32,
) {
    (*c.cast::<cram_container_layout>()).num_blocks = num_blocks;
}

pub unsafe fn cram_cram_external_c_92_cram_container_get_num_records(
    c: *mut hts_sys::cram_container,
) -> i32 {
    (*c.cast::<cram_container_layout>()).num_records
}

pub unsafe fn cram_cram_external_c_96_cram_container_get_num_bases(
    c: *mut hts_sys::cram_container,
) -> i64 {
    (*c.cast::<cram_container_layout>()).num_bases
}

pub unsafe fn cram_cram_external_c_104_cram_container_get_landmarks(
    c: *mut hts_sys::cram_container,
    num_landmarks: *mut i32,
) -> *mut i32 {
    let c = c.cast::<cram_container_layout>();
    *num_landmarks = (*c).num_landmarks;
    (*c).landmark
}

pub unsafe fn cram_cram_external_c_112_cram_container_set_landmarks(
    c: *mut hts_sys::cram_container,
    num_landmarks: i32,
    landmarks: *mut i32,
) {
    let c = c.cast::<cram_container_layout>();
    (*c).num_landmarks = num_landmarks;
    (*c).landmark = landmarks;
}

pub unsafe fn cram_cram_external_c_120_cram_container_is_empty(fd: *mut hts_sys::cram_fd) -> c_int {
    hts_sys::cram_container_is_empty(fd)
}

pub unsafe fn cram_cram_external_c_124_cram_container_get_coords(
    c: *mut hts_sys::cram_container,
    refid: *mut c_int,
    start: *mut i64,
    span: *mut i64,
) {
    let c = c.cast::<cram_container_layout>();
    if !refid.is_null() {
        *refid = (*c).ref_seq_id;
    }
    if !start.is_null() {
        *start = (*c).ref_seq_start;
    }
    if !span.is_null() {
        *span = (*c).ref_seq_span;
    }
}

pub unsafe fn cram_cram_external_c_152_cram_block_compression_hdr_set_DS(
    ch: *mut c_void,
    ds: c_int,
    new_rg: c_int,
) -> c_int {
    if ch.is_null() {
        return -1;
    }
    let ch = ch.cast::<cram_block_compression_hdr_layout>();
    if (*ch).codecs[ds as usize].is_null() {
        return -1;
    }

    let co = (*ch).codecs[ds as usize];
    match *(co.cast::<c_int>()) {
        3 => {
            let co = co.cast::<cram_codec_huffman_layout>();
            if (*co).huffman.ncodes != 1 {
                return -1;
            }
            (*(*co).huffman.codes).symbol = new_rg as i64;
            0
        }
        6 => {
            let co = co.cast::<cram_codec_beta_layout>();
            if (*co).beta.nbits != 0 {
                return -1;
            }
            (*co).beta.offset = -new_rg;
            0
        }
        _ => -1,
    }
}

pub unsafe fn cram_cram_external_c_177_cram_block_compression_hdr_set_rg(
    ch: *mut c_void,
    new_rg: c_int,
) -> c_int {
    cram_cram_external_c_152_cram_block_compression_hdr_set_DS(ch, 17, new_rg)
}

pub unsafe fn cram_cram_external_c_189_cram_block_compression_hdr_decoder2encoder(
    fd: *mut c_void,
    ch: *mut c_void,
) -> c_int {
    if ch.is_null() {
        return -1;
    }
    let ch = ch.cast::<cram_block_compression_hdr_layout>();
    for i in 0..46usize {
        let co = (*ch).codecs[i];
        if co.is_null() {
            continue;
        }
        if cram_cram_codecs_c_4031_cram_codec_decoder2encoder(fd, co) == -1 {
            return -1;
        }
    }
    0
}

pub unsafe fn cram_cram_external_c_215_cram_codec_iter_init(hdr: *mut c_void, iter: *mut c_void) {
    let iter = iter.cast::<cram_codec_iter_layout>();
    (*iter).hdr = hdr.cast::<cram_block_compression_hdr_layout>();
    (*iter).curr_map = std::ptr::null_mut();
    (*iter).idx = 0;
    (*iter).is_tag = 0;
}

pub fn cram_cram_external_c_224_cram_ds_to_key(ds: c_int) -> c_int {
    match ds {
        10 => 256 * b'R' as c_int + b'N' as c_int,
        11 => 256 * b'Q' as c_int + b'S' as c_int,
        12 => 256 * b'I' as c_int + b'N' as c_int,
        13 => 256 * b'S' as c_int + b'C' as c_int,
        14 => 256 * b'B' as c_int + b'F' as c_int,
        15 => 256 * b'C' as c_int + b'F' as c_int,
        16 => 256 * b'A' as c_int + b'P' as c_int,
        17 => 256 * b'R' as c_int + b'G' as c_int,
        18 => 256 * b'M' as c_int + b'Q' as c_int,
        19 => 256 * b'N' as c_int + b'S' as c_int,
        20 => 256 * b'M' as c_int + b'F' as c_int,
        21 => 256 * b'T' as c_int + b'S' as c_int,
        22 => 256 * b'N' as c_int + b'P' as c_int,
        23 => 256 * b'N' as c_int + b'F' as c_int,
        24 => 256 * b'R' as c_int + b'L' as c_int,
        25 => 256 * b'F' as c_int + b'N' as c_int,
        26 => 256 * b'F' as c_int + b'C' as c_int,
        27 => 256 * b'F' as c_int + b'P' as c_int,
        28 => 256 * b'D' as c_int + b'L' as c_int,
        29 => 256 * b'B' as c_int + b'A' as c_int,
        30 => 256 * b'B' as c_int + b'S' as c_int,
        31 => 256 * b'T' as c_int + b'L' as c_int,
        32 => 256 * b'R' as c_int + b'I' as c_int,
        33 => 256 * b'R' as c_int + b'S' as c_int,
        34 => 256 * b'P' as c_int + b'D' as c_int,
        35 => 256 * b'H' as c_int + b'C' as c_int,
        36 => 256 * b'B' as c_int + b'B' as c_int,
        37 => 256 * b'Q' as c_int + b'Q' as c_int,
        38 => 256 * b'T' as c_int + b'N' as c_int,
        43 => 256 * b'T' as c_int + b'C' as c_int,
        44 => 256 * b'T' as c_int + b'M' as c_int,
        45 => 256 * b'T' as c_int + b'V' as c_int,
        _ => -1,
    }
}

pub unsafe fn cram_cram_external_c_264_cram_codec_iter_next(
    iter: *mut c_void,
    key: *mut c_int,
) -> *mut c_void {
    let iter = iter.cast::<cram_codec_iter_layout>();
    let hdr = (*iter).hdr;

    if (*iter).is_tag == 0 {
        let mut cc;
        loop {
            cc = (*hdr).codecs[(*iter).idx as usize];
            (*iter).idx += 1;
            if !cc.is_null() || (*iter).idx >= 46 {
                break;
            }
        }
        if !cc.is_null() {
            *key = cram_cram_external_c_224_cram_ds_to_key((*iter).idx - 1);
            return cc;
        }

        (*iter).idx = 0;
        (*iter).is_tag = 1;
    }

    loop {
        if (*iter).curr_map.is_null() {
            (*iter).curr_map =
                (*hdr).tag_encoding_map[(*iter).idx as usize].cast::<cram_map_layout>();
            (*iter).idx += 1;
        }

        let cc = if !(*iter).curr_map.is_null() {
            (*(*iter).curr_map).codec
        } else {
            std::ptr::null_mut()
        };
        if !cc.is_null() {
            *key = (*(*iter).curr_map).key;
            (*iter).curr_map = (*(*iter).curr_map).next;
            return cc;
        }
        if (*iter).idx >= 32 {
            break;
        }
    }

    std::ptr::null_mut()
}

pub unsafe fn cram_cram_external_c_320_cram_cid2ds_free(cid2ds: *mut cram_cid2ds_t) {
    if !cid2ds.is_null() {
        drop(Box::from_raw(cid2ds));
    }
}

pub unsafe fn cram_cram_external_c_342_cram_update_cid2ds_map(
    hdr: *mut cram_block_compression_hdr,
    cid2ds: *mut cram_cid2ds_t,
) -> *mut cram_cid2ds_t {
    let c2d = if cid2ds.is_null() {
        Box::into_raw(Box::new(cram_cid2ds_t {
            ds: Vec::new(),
            hash: HashMap::new(),
            ds_a: Vec::new(),
        }))
    } else {
        cid2ds
    };

    let mut citer = cram_codec_iter_layout {
        hdr: std::ptr::null_mut(),
        curr_map: std::ptr::null_mut(),
        idx: 0,
        is_tag: 0,
    };
    cram_cram_external_c_215_cram_codec_iter_init(
        hdr.cast(),
        (&mut citer as *mut cram_codec_iter_layout).cast(),
    );

    let mut key = 0;
    loop {
        let codec = cram_cram_external_c_264_cram_codec_iter_next(
            (&mut citer as *mut cram_codec_iter_layout).cast(),
            &mut key,
        );
        if codec.is_null() {
            break;
        }

        let mut bnum = [-2; 2];
        cram_cram_external_c_665_cram_codec_get_content_ids(codec, bnum.as_mut_ptr());
        for block_id in bnum {
            if block_id <= -2 {
                continue;
            }

            let c2d_ref = &mut *c2d;
            if let Some(head_ref) = c2d_ref.hash.get_mut(&block_id) {
                let mut dsi = *head_ref;
                while dsi >= 0 {
                    let ds = c2d_ref.ds[dsi as usize];
                    if ds.data_series == key {
                        break;
                    }
                    dsi = ds.next;
                }

                if dsi == -1 {
                    let new_idx = c2d_ref.ds.len() as c_int;
                    c2d_ref.ds.push(cram_ds_list {
                        data_series: key,
                        next: *head_ref,
                    });
                    *head_ref = new_idx;
                }
            } else {
                let new_idx = c2d_ref.ds.len() as c_int;
                c2d_ref.ds.push(cram_ds_list {
                    data_series: key,
                    next: -1,
                });
                c2d_ref.hash.insert(block_id, new_idx);
            }
        }
    }

    c2d
}

pub unsafe fn cram_cram_external_c_443_cram_cid2ds_query(
    c2d: *mut cram_cid2ds_t,
    content_id: c_int,
    n: *mut c_int,
) -> *mut c_int {
    *n = 0;
    if c2d.is_null() {
        return std::ptr::null_mut();
    }

    let c2d = &mut *c2d;
    let Some(mut dsi) = c2d.hash.get(&content_id).copied() else {
        return std::ptr::null_mut();
    };

    c2d.ds_a.clear();
    while dsi >= 0 {
        let ds = c2d.ds[dsi as usize];
        c2d.ds_a.push(ds.data_series);
        dsi = ds.next;
    }

    *n = c2d.ds_a.len() as c_int;
    c2d.ds_a.as_mut_ptr()
}

pub unsafe fn cram_cram_external_c_476_cram_describe_encodings(
    hdr: *mut hts_sys::cram_block_compression_hdr,
    ks: *mut kstring_t,
) -> c_int {
    let mut citer = cram_codec_iter_layout {
        hdr: std::ptr::null_mut(),
        curr_map: std::ptr::null_mut(),
        idx: 0,
        is_tag: 0,
    };
    cram_cram_external_c_215_cram_codec_iter_init(
        hdr.cast(),
        (&mut citer as *mut cram_codec_iter_layout).cast(),
    );

    let mut r = 0;
    let mut key = 0;
    loop {
        let codec = cram_cram_external_c_264_cram_codec_iter_next(
            (&mut citer as *mut cram_codec_iter_layout).cast(),
            &mut key,
        );
        if codec.is_null() {
            break;
        }

        let mut key_s = [0 as c_char; 4];
        let mut key_i = 0usize;
        if (key >> 16) != 0 {
            key_s[key_i] = (key >> 16) as c_char;
            key_i += 1;
        }
        key_s[key_i] = ((key >> 8) & 0xff) as c_char;
        key_i += 1;
        key_s[key_i] = (key & 0xff) as c_char;
        key_i += 1;

        r |= (kputc(b'\t' as c_int, ks) < 0) as c_int;
        r |= (kputsn(key_s.as_ptr(), key_i, ks) < 0) as c_int;
        r |= (kputc(b'\t' as c_int, ks) < 0) as c_int;
        r |= (cram_cram_codecs_c_4185_cram_codec_describe(codec, ks) < 0) as c_int;
        r |= (kputc(b'\n' as c_int, ks) < 0) as c_int;
    }

    if r != 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn cram_cram_external_c_522_cram_block_get_content_id(
    b: *mut hts_sys::cram_block,
) -> i32 {
    let b = b.cast::<cram_block_layout>();
    if (*b).content_type == crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE {
        -1
    } else {
        (*b).content_id
    }
}

pub unsafe fn cram_cram_external_c_525_cram_block_get_comp_size(
    b: *mut hts_sys::cram_block,
) -> i32 {
    (*b.cast::<cram_block_layout>()).comp_size
}

pub unsafe fn cram_cram_external_c_526_cram_block_get_uncomp_size(
    b: *mut hts_sys::cram_block,
) -> i32 {
    (*b.cast::<cram_block_layout>()).uncomp_size
}

pub unsafe fn cram_cram_external_c_527_cram_block_get_crc32(b: *mut hts_sys::cram_block) -> i32 {
    (*b.cast::<cram_block_layout>()).crc32 as i32
}

pub unsafe fn cram_cram_external_c_528_cram_block_get_data(
    b: *mut hts_sys::cram_block,
) -> *mut c_void {
    (*b.cast::<cram_block_layout>()).data.cast()
}

pub unsafe fn cram_cram_external_c_533_cram_block_get_content_type(
    b: *mut hts_sys::cram_block,
) -> hts_sys::cram_content_type {
    (*b.cast::<cram_block_layout>()).content_type
}

pub unsafe fn cram_cram_external_c_537_cram_block_set_content_id(
    b: *mut hts_sys::cram_block,
    id: i32,
) {
    (*b.cast::<cram_block_layout>()).content_id = id;
}

pub unsafe fn cram_cram_external_c_538_cram_block_set_comp_size(
    b: *mut hts_sys::cram_block,
    size: i32,
) {
    (*b.cast::<cram_block_layout>()).comp_size = size;
}

pub unsafe fn cram_cram_external_c_539_cram_block_set_uncomp_size(
    b: *mut hts_sys::cram_block,
    size: i32,
) {
    (*b.cast::<cram_block_layout>()).uncomp_size = size;
}

pub unsafe fn cram_cram_external_c_540_cram_block_set_crc32(b: *mut hts_sys::cram_block, crc: i32) {
    (*b.cast::<cram_block_layout>()).crc32 = crc as u32;
}

pub unsafe fn cram_cram_external_c_541_cram_block_set_data(
    b: *mut hts_sys::cram_block,
    data: *mut c_void,
) {
    (*b.cast::<cram_block_layout>()).data = data.cast();
}

pub unsafe fn cram_cram_external_c_544_cram_block_append(
    b: *mut hts_sys::cram_block,
    data: *const c_void,
    size: c_int,
) -> c_int {
    cram_cram_io_h_248_block_append(b, data, size as usize)
}

pub unsafe fn cram_cram_external_c_551_cram_block_update_size(b: *mut hts_sys::cram_block) {
    let b = b.cast::<cram_block_layout>();
    (*b).comp_size = (*b).byte as i32;
    (*b).uncomp_size = (*b).byte as i32;
}

pub unsafe fn cram_cram_external_c_554_cram_block_get_offset(b: *mut hts_sys::cram_block) -> u64 {
    (*b.cast::<cram_block_layout>()).byte as u64
}

pub unsafe fn cram_cram_external_c_555_cram_block_set_offset(
    b: *mut hts_sys::cram_block,
    offset: u64,
) {
    (*b.cast::<cram_block_layout>()).byte = offset as usize;
}

pub unsafe fn cram_cram_external_c_568_cram_expand_method(
    data: *mut u8,
    size: i32,
    mut comp: hts_sys::cram_block_method,
) -> *mut cram_method_details {
    const CRAM_COMP_UNKNOWN: hts_sys::cram_block_method = -1;
    const CRAM_COMP_GZIP: hts_sys::cram_block_method = 1;
    const CRAM_COMP_BZIP2: hts_sys::cram_block_method = 2;
    const CRAM_COMP_LZMA: hts_sys::cram_block_method = 3;
    const CRAM_COMP_RANS4X8: hts_sys::cram_block_method = 4;
    const CRAM_COMP_RANSNX16: hts_sys::cram_block_method = 5;
    const CRAM_COMP_ARITH: hts_sys::cram_block_method = 6;
    const CRAM_COMP_TOK3: hts_sys::cram_block_method = 8;
    const RANS_ORDER_X32: u8 = 0x04;
    const RANS_ORDER_STRIPE: u8 = 0x08;
    const RANS_ORDER_NOSZ: u8 = 0x10;
    const RANS_ORDER_CAT: u8 = 0x20;
    const RANS_ORDER_RLE: u8 = 0x40;
    const RANS_ORDER_PACK: u8 = 0x80;

    let cm =
        calloc(1, std::mem::size_of::<cram_method_details>() as u64).cast::<cram_method_details>();
    if cm.is_null() {
        return std::ptr::null_mut();
    }

    if comp == CRAM_COMP_UNKNOWN {
        if size > 1 && *data == 0x1f && *data.add(1) == 0x8b {
            comp = CRAM_COMP_GZIP;
        } else if size > 3 && *data.add(1) == b'B' && *data.add(2) == b'Z' && *data.add(3) == b'h' {
            comp = CRAM_COMP_BZIP2;
        } else if size > 6
            && *data == 0xfd
            && *data.add(1) == b'7'
            && *data.add(2) == b'z'
            && *data.add(3) == b'X'
            && *data.add(4) == b'Z'
            && *data.add(5) == 0
        {
            comp = CRAM_COMP_LZMA;
        } else {
            comp = CRAM_COMP_UNKNOWN;
        }
    }
    (*cm).method = comp;

    match comp {
        CRAM_COMP_GZIP => {
            if size > 8 {
                (*cm).level = match *data.add(8) {
                    4 => 1,
                    2 => 9,
                    _ => 5,
                };
            }
        }
        CRAM_COMP_BZIP2 => {
            if size > 3 && *data.add(3) >= b'1' && *data.add(3) <= b'9' {
                (*cm).level = (*data.add(3) - b'0') as c_int;
            }
        }
        CRAM_COMP_RANS4X8 => {
            (*cm).nway = 4;
            (*cm).order = if size > 0 && *data == 1 { 1 } else { 0 };
        }
        CRAM_COMP_RANSNX16 => {
            if size > 0 {
                let flags = *data;
                (*cm).order = (flags & 1) as c_int;
                (*cm).nway = if flags & RANS_ORDER_X32 != 0 { 32 } else { 4 };
                (*cm).rle = (flags & RANS_ORDER_RLE != 0) as c_int;
                (*cm).pack = (flags & RANS_ORDER_PACK != 0) as c_int;
                (*cm).cat = (flags & RANS_ORDER_CAT != 0) as c_int;
                (*cm).stripe = (flags & RANS_ORDER_STRIPE != 0) as c_int;
                (*cm).nosz = (flags & RANS_ORDER_NOSZ != 0) as c_int;
            }
        }
        CRAM_COMP_ARITH => {
            if size > 0 {
                let flags = *data;
                (*cm).order = (flags & 3) as c_int;
                (*cm).rle = (flags & RANS_ORDER_RLE != 0) as c_int;
                (*cm).pack = (flags & RANS_ORDER_PACK != 0) as c_int;
                (*cm).cat = (flags & RANS_ORDER_CAT != 0) as c_int;
                (*cm).stripe = (flags & RANS_ORDER_STRIPE != 0) as c_int;
                (*cm).nosz = (flags & RANS_ORDER_NOSZ != 0) as c_int;
                (*cm).ext = (flags & 4 != 0) as c_int;
            }
        }
        CRAM_COMP_TOK3 => {
            if size > 8 {
                (*cm).level = match *data.add(8) {
                    1 => 11,
                    0 => 1,
                    _ => (*cm).level,
                };
            }
        }
        _ => {}
    }

    cm
}

pub unsafe fn cram_cram_external_c_665_cram_codec_get_content_ids(c: *mut c_void, ids: *mut c_int) {
    *ids = cram_cram_codecs_c_3968_cram_codec_to_id(c, ids.add(1));
}

pub unsafe fn cram_cram_external_c_683_cram_copy_slice(
    in_: *mut cram_fd,
    out: *mut cram_fd,
    num_slice: i32,
) -> c_int {
    for _ in 0..num_slice {
        let mut blk = cram_read_block(in_);
        if blk.is_null() {
            return -1;
        }

        let hdr = cram_decode_slice_header(in_, blk);
        if hdr.is_null() {
            cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }

        if cram_write_block(out, blk) != 0 {
            cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }
        cram_cram_io_c_1565_cram_free_block(blk);

        let num_blocks = cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(hdr);
        for _ in 0..num_blocks {
            blk = cram_read_block(in_);
            if blk.is_null() || cram_write_block(out, blk) != 0 {
                if !blk.is_null() {
                    cram_cram_io_c_1565_cram_free_block(blk);
                }
                return -1;
            }
            cram_cram_io_c_1565_cram_free_block(blk);
        }
        cram_free_slice_header(hdr);
    }

    0
}

pub unsafe fn cram_cram_external_c_725_cram_skip_container(
    in_: *mut cram_fd,
    c: *mut cram_container,
) -> c_int {
    let mut blk = cram_read_block(in_);
    if blk.is_null() {
        return -1;
    }
    cram_cram_io_c_1565_cram_free_block(blk);

    let c = c.cast::<cram_container_layout>();
    for _ in 0..(*c).num_landmarks {
        blk = cram_read_block(in_);
        if blk.is_null() {
            return -1;
        }
        let hdr = cram_decode_slice_header(in_, blk);
        if hdr.is_null() {
            cram_cram_io_c_1565_cram_free_block(blk);
            return -1;
        }
        cram_cram_io_c_1565_cram_free_block(blk);

        let num_blocks = cram_cram_external_c_500_cram_slice_hdr_get_num_blocks(hdr);
        for _ in 0..num_blocks {
            blk = cram_read_block(in_);
            if blk.is_null() {
                cram_free_slice_header(hdr);
                return -1;
            }
            cram_cram_io_c_1565_cram_free_block(blk);
        }
        cram_free_slice_header(hdr);
    }

    0
}

pub unsafe fn cram_cram_external_c_1029_cram_get_refs(fd: *mut htsFile) -> *mut refs_t {
    if (*fd).format.format == HTS_FORMAT_CRAM {
        (*(*fd).fp.cram.cast::<cram_fd_layout>()).refs
    } else {
        std::ptr::null_mut()
    }
}

pub fn cram_os_h_155_le_int4(x: u32) -> u32 {
    u32::from_le(x)
}

pub fn cram_os_h_158_le_int2(x: u16) -> u16 {
    u16::from_le(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::htslib_rs::sam::{
        bam1_core_t, BAM_CDEL, BAM_CINS, BAM_CMATCH, BAM_CSOFT_CLIP, BAM_FPAIRED,
    };
    use std::ffi::{CStr, CString};

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
            assert_eq!(block.content_type, crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE);
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
            let refs = 0x1234usize as *mut hts_sys::refs_t;
            let mut cram_fd = cram_fd_layout {
                refs,
                ..std::mem::zeroed()
            };
            let mut fp: htsFile = std::mem::zeroed();
            fp.fp = crate::htslib_rs::hts::htsFilePtr {
                cram: (&mut cram_fd as *mut cram_fd_layout).cast(),
            };
            fp.format.format = HTS_FORMAT_CRAM;
            assert_eq!(cram_cram_external_c_1029_cram_get_refs(&mut fp), refs);

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
            let direct_ptr = (&mut direct as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let collision_ptr =
                (&mut collision as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let ignored_ptr =
                (&mut ignored_core as *mut cram_block_layout).cast::<hts_sys::cram_block>();

            let mut by_id = vec![std::ptr::null_mut(); 512];
            by_id[42] = direct_ptr;
            by_id[256 + 777 % 251] = std::ptr::null_mut();
            let mut blocks = [ignored_ptr, collision_ptr, std::ptr::null_mut()];
            let mut slice = cram_slice_layout {
                hdr: &mut hdr,
                hdr_block: std::ptr::null_mut(),
                block: blocks.as_mut_ptr(),
                block_by_id: by_id.as_mut_ptr(),
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
            let block = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 3);
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

            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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

            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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

            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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

            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 7);
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
            assert_eq!((*read_block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
            assert_eq!((*read_block).orig_method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
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
            assert_eq!((*block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);

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

                let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                assert_eq!((*read_block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
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
            assert_eq!((*block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
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
            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
            assert_eq!((*block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
            assert_eq!(
                std::slice::from_raw_parts((*block).data, (*block).alloc),
                b"abc"
            );

            (*block).method = crate::htslib_rs::cram::CRAM_BLOCK_METHOD_GZIP;
            (*block).uncomp_size = 0;
            (*block).crc32_checked = 1;
            assert_eq!(cram_cram_io_c_1576_cram_uncompress_block(b), 0);
            assert_eq!((*block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);

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

            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
            assert_eq!((*block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
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
            assert_eq!((*metrics).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
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
            assert_eq!((*block).content_type, crate::htslib_rs::cram::CRAM_CONTENT_TYPE_CORE);
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
            std::ptr::copy_nonoverlapping(b"ACG\0".as_ptr().cast::<c_char>(), seq, 4);
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
            std::ptr::copy_nonoverlapping(b"TGA\0".as_ptr().cast::<c_char>(), mf_data, 4);
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
            std::ptr::copy_nonoverlapping(b"NNN\0".as_ptr().cast::<c_char>(), (*entry).seq, 4);
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
            fd.refs = refs;
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
            fd.refs = refs;
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
            fd.refs = refs;
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
            fd.refs = refs;
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
                fp: 0x1234usize as *mut hFILE,
                mode: 0,
                version: 0,
                ..std::mem::zeroed()
            };
            assert_eq!(
                cram_cram_io_h_646_cram_hfile((&mut fd as *mut cram_fd_layout).cast()),
                0x1234usize as *mut hts_sys::hFILE
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
                cram_cram_codecs_c_1219_cram_beta_encode_int as usize as *mut c_void
            );
            assert_eq!(
                (*enc).store,
                cram_cram_codecs_c_1183_cram_beta_encode_store as usize as *mut c_void
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
                store: test_byte_array_len_store_val as usize as *mut c_void,
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
            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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

            let sub_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 9);
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
                out: sub_block,
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: std::ptr::null_mut(),
                store: std::ptr::null_mut(),
                size: std::ptr::null_mut(),
                flush: std::ptr::null_mut(),
                get_block: test_xdelta_get_block as usize as *mut c_void,
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
                by_id[514]
            );
            cram_cram_io_c_1565_cram_free_block(by_id[514]);
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
                cram_cram_codecs_c_1408_cram_xpack_decode_char as usize as *mut c_void
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
                sub_codec_dat: 9usize as *mut c_void,
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
                cram_cram_codecs_c_1592_cram_xpack_encode_int as usize as *mut c_void
            );
            assert_eq!(
                (*xpack_enc).flush,
                cram_cram_codecs_c_1515_cram_xpack_encode_flush as usize as *mut c_void
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
            let b = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 99);
            assert!(!b.is_null());
            let block = b.cast::<cram_block_layout>();
            assert_eq!((*block).method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
            assert_eq!((*block).orig_method, crate::htslib_rs::cram::CRAM_BLOCK_METHOD_RAW);
            assert_eq!((*block).content_type, crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL);
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
            let block_ptr = (&mut block as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let mut by_id = [std::ptr::null_mut(); 513];
            by_id[512] = block_ptr;
            let mut slice = cram_slice_layout {
                hdr: std::ptr::null_mut(),
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: by_id.as_mut_ptr(),
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
                decode: test_xdelta_decode_u32 as usize as *mut c_void,
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
            sub_codec.get_block = test_xdelta_get_block as usize as *mut c_void;
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
            let out_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_1719_cram_xdelta_decode_block as usize as *mut c_void
            );
            assert_eq!(
                (*(*dec).xdelta.sub_codec.cast::<cram_codec_external_layout>())
                    .external
                    .content_id,
                9
            );
            cram_cram_codecs_c_1762_cram_xdelta_decode_free(dec.cast());

            let sub_out =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
            let mut sub_enc = cram_codec_xdelta_layout {
                codec: 0,
                out: sub_out,
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: std::ptr::null_mut(),
                encode: test_byte_array_len_encode_val as usize as *mut c_void,
                store: test_byte_array_len_store_val as usize as *mut c_void,
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

            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                sub_codec_dat: 9usize as *mut c_void,
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
                cram_cram_codecs_c_1976_cram_xdelta_encode_char as usize as *mut c_void
            );
            assert_eq!(
                (*enc).store,
                cram_cram_codecs_c_1930_cram_xdelta_encode_store as usize as *mut c_void
            );
            assert_eq!(
                (*enc).flush,
                cram_cram_codecs_c_1835_cram_xdelta_encode_flush as usize as *mut c_void
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
            cram_cram_io_c_1565_cram_free_block(codec.out);

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
                get_block: test_xrle_get_block as usize as *mut c_void,
                size: test_xrle_size as usize as *mut c_void,
                out: (&mut len_block as *mut cram_block_layout).cast(),
                ..codec
            };
            let mut lit_codec = cram_codec_xrle_layout {
                get_block: test_xrle_get_block as usize as *mut c_void,
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
                by_id[515]
            );
            cram_cram_io_c_1565_cram_free_block(by_id[515]);

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
            let len_out =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
            let lit_out =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
            let mut len_enc = cram_codec_byte_array_len_layout {
                out: len_out,
                encode: test_byte_array_len_encode_val as usize as *mut c_void,
                store: test_byte_array_len_store_len as usize as *mut c_void,
                ..std::mem::zeroed()
            };
            let mut lit_enc = cram_codec_byte_array_len_layout {
                out: lit_out,
                encode: test_byte_array_len_encode_val as usize as *mut c_void,
                store: test_byte_array_len_store_val as usize as *mut c_void,
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

            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_2125_cram_xrle_decode_char as usize as *mut c_void
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
                len_dat: 7usize as *mut c_void,
                lit_dat: 9usize as *mut c_void,
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
                cram_cram_codecs_c_2371_cram_xrle_encode_char as usize as *mut c_void
            );
            assert_eq!(
                (*enc).flush,
                cram_cram_codecs_c_2257_cram_xrle_encode_flush as usize as *mut c_void
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
                cram_cram_codecs_c_2452_cram_subexp_decode as usize as *mut c_void
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
                cram_cram_codecs_c_2546_cram_gamma_decode as usize as *mut c_void
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
                cram_cram_codecs_c_2660_cram_huffman_decode_char as usize as *mut c_void
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
            let out = cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
            let mut codec = cram_codec_huffman_encoder_layout {
                codec: 0,
                out,
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
            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_3030_cram_huffman_encode_int as usize as *mut c_void
            );
            assert_eq!(
                (*init).store,
                cram_cram_codecs_c_3112_cram_huffman_encode_store as usize as *mut c_void
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
                decode: test_byte_array_len_decode_len as usize as *mut c_void,
                encode: test_byte_array_len_encode_len as usize as *mut c_void,
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
            let out_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
            len_codec.out = out_block;
            let mut val_codec = cram_codec_byte_array_len_layout {
                codec: 0,
                out: out_block,
                vv: std::ptr::null_mut(),
                codec_id: 0,
                free: std::ptr::null_mut(),
                decode: test_byte_array_len_decode_val as usize as *mut c_void,
                encode: test_byte_array_len_encode_val as usize as *mut c_void,
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

            len_codec.store = test_byte_array_len_store_len as usize as *mut c_void;
            val_codec.store = test_byte_array_len_store_val as usize as *mut c_void;
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
            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_3371_cram_byte_array_len_decode as usize as *mut c_void
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
                len_dat: 7usize as *mut c_void,
                val_dat: 8usize as *mut c_void,
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
                cram_cram_codecs_c_3479_cram_byte_array_len_encode as usize as *mut c_void
            );
            assert_eq!(
                (*enc).store,
                cram_cram_codecs_c_3506_cram_byte_array_len_encode_store as usize as *mut c_void
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
            let input_ptr =
                (&mut input_block as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let mut by_id = [input_ptr];
            let mut slice = cram_slice_layout {
                hdr: std::ptr::null_mut(),
                hdr_block: std::ptr::null_mut(),
                block: std::ptr::null_mut(),
                block_by_id: by_id.as_mut_ptr(),
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
            let out_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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

            let enc_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
            codec.out = enc_block;
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
                describe: test_codec_describe_len as usize as *mut c_void,
                byte_array_len: cram_byte_array_len_decoder_layout {
                    len_codec: std::ptr::null_mut(),
                    val_codec: std::ptr::null_mut(),
                },
            };
            let mut val_codec = cram_codec_byte_array_len_layout {
                describe: test_codec_describe_val as usize as *mut c_void,
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
                cram_cram_codecs_c_3586_cram_byte_array_stop_decode_char as usize as *mut c_void
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
                cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize as *mut c_void
            );
            assert_eq!(
                (*enc).store,
                cram_cram_codecs_c_3749_cram_byte_array_stop_encode_store as usize as *mut c_void
            );
            (*enc).vv = &mut vv as *mut varint_vec_layout;
            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_390_cram_external_decode_char as usize as *mut c_void
            );
            cram_cram_codecs_c_433_cram_external_decode_free(ext_dec.cast());

            let ext_enc = cram_cram_codecs_c_586_cram_external_encode_init(
                std::ptr::null_mut(),
                1,
                1,
                7usize as *mut c_void,
                3 << 8,
                std::ptr::null_mut(),
            )
            .cast::<cram_codec_external_layout>();
            assert!(!ext_enc.is_null());
            (*ext_enc).vv = &mut vv;
            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_710_cram_varint_decode_slong as usize as *mut c_void
            );
            cram_cram_codecs_c_732_cram_varint_decode_free(var_dec.cast());

            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                11usize as *mut c_void,
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
                *mut hts_sys::cram_slice,
                *mut c_void,
                *mut hts_sys::cram_block,
                *mut c_char,
                *mut c_int,
            ) -> c_int = std::mem::transmute((*const_dec).decode);
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

            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
                cram_cram_codecs_c_1090_cram_beta_decode_int as usize as *mut c_void
            );
            cram_cram_codecs_c_1131_cram_beta_decode_free(beta_dec.cast());

            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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

            external.describe =
                cram_cram_codecs_c_454_cram_external_describe as usize as *mut c_void;
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
                cram_cram_codecs_c_390_cram_external_decode_char as usize as *mut c_void;
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut external as *mut cram_codec_external_layout).cast(),
                ),
                0
            );
            assert_eq!(
                external.encode,
                cram_cram_codecs_c_547_cram_external_encode_char as usize as *mut c_void
            );
            assert_eq!(
                external.store,
                cram_cram_codecs_c_562_cram_external_encode_store as usize as *mut c_void
            );

            varint.decode = cram_cram_codecs_c_710_cram_varint_decode_slong as usize as *mut c_void;
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut varint as *mut cram_codec_varint_layout).cast(),
                ),
                0
            );
            assert_eq!(
                varint.encode,
                cram_cram_codecs_c_841_cram_varint_encode_slong as usize as *mut c_void
            );
            beta.decode = cram_cram_codecs_c_1108_cram_beta_decode_char as usize as *mut c_void;
            assert_eq!(
                cram_cram_codecs_c_4031_cram_codec_decoder2encoder(
                    std::ptr::null_mut(),
                    (&mut beta as *mut cram_codec_beta_layout).cast(),
                ),
                0
            );
            assert_eq!(
                beta.encode,
                cram_cram_codecs_c_1231_cram_beta_encode_char as usize as *mut c_void
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
                cram_cram_codecs_c_3733_cram_byte_array_stop_encode as usize as *mut c_void
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
                cram_cram_codecs_c_2708_cram_huffman_decode_int as usize as *mut c_void;
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
                cram_cram_codecs_c_3030_cram_huffman_encode_int as usize as *mut c_void
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
        _slice: *mut hts_sys::cram_slice,
        c: *mut c_void,
        _in: *mut hts_sys::cram_block,
        out: *mut c_char,
        _out_size: *mut c_int,
    ) -> c_int {
        *(out.cast::<i32>()) = (*(c.cast::<cram_codec_byte_array_len_layout>())).codec_id;
        0
    }

    unsafe extern "C" fn test_byte_array_len_decode_val(
        _slice: *mut hts_sys::cram_slice,
        _c: *mut c_void,
        _in: *mut hts_sys::cram_block,
        out: *mut c_char,
        out_size: *mut c_int,
    ) -> c_int {
        memcpy(out.cast(), c"XYZ".as_ptr().cast(), *out_size as u64);
        0
    }

    unsafe extern "C" fn test_byte_array_len_encode_len(
        _slice: *mut hts_sys::cram_slice,
        c: *mut c_void,
        in_: *mut c_char,
        _in_size: c_int,
    ) -> c_int {
        let val = *(in_.cast::<i32>()) as u8;
        let c = c.cast::<cram_codec_byte_array_len_layout>();
        cram_cram_io_h_261_block_append_char((*c).out, val as c_char)
    }

    unsafe extern "C" fn test_byte_array_len_encode_val(
        _slice: *mut hts_sys::cram_slice,
        c: *mut c_void,
        in_: *mut c_char,
        in_size: c_int,
    ) -> c_int {
        let c = c.cast::<cram_codec_byte_array_len_layout>();
        cram_cram_io_h_248_block_append((*c).out, in_.cast(), in_size as usize)
    }

    unsafe extern "C" fn test_xdelta_decode_u32(
        _slice: *mut hts_sys::cram_slice,
        _c: *mut c_void,
        in_: *mut hts_sys::cram_block,
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
        b: *mut hts_sys::cram_block,
        _prefix: *mut c_char,
        _version: c_int,
    ) -> c_int {
        let mut bytes = *b"L";
        cram_cram_io_h_248_block_append(b, bytes.as_mut_ptr().cast(), bytes.len());
        bytes.len() as c_int
    }

    unsafe extern "C" fn test_byte_array_len_store_val(
        _c: *mut c_void,
        b: *mut hts_sys::cram_block,
        _prefix: *mut c_char,
        _version: c_int,
    ) -> c_int {
        let mut bytes = *b"VA";
        cram_cram_io_h_248_block_append(b, bytes.as_mut_ptr().cast(), bytes.len());
        bytes.len() as c_int
    }

    unsafe extern "C" fn test_xdelta_get_block(
        _slice: *mut hts_sys::cram_slice,
        c: *mut c_void,
    ) -> *mut hts_sys::cram_block {
        (*(c.cast::<cram_codec_xdelta_layout>())).out
    }

    unsafe extern "C" fn test_xrle_get_block(
        _slice: *mut hts_sys::cram_slice,
        c: *mut c_void,
    ) -> *mut hts_sys::cram_block {
        (*(c.cast::<cram_codec_xrle_layout>())).out
    }

    unsafe extern "C" fn test_xrle_size(_slice: *mut hts_sys::cram_slice, c: *mut c_void) -> c_int {
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

    unsafe extern "C" fn test_varint_put32_blk(blk: *mut hts_sys::cram_block, val: i32) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = val;
        }
        1
    }

    unsafe extern "C" fn test_varint_put32_blk_append(
        blk: *mut hts_sys::cram_block,
        val: i32,
    ) -> c_int {
        let mut byte = val as u8;
        cram_cram_io_h_248_block_append(blk, (&mut byte as *mut u8).cast(), 1);
        1
    }

    unsafe extern "C" fn test_varint_size(_val: i64) -> c_int {
        1
    }

    unsafe extern "C" fn test_varint_put32s_blk(blk: *mut hts_sys::cram_block, val: i32) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = -val;
        }
        1
    }

    unsafe extern "C" fn test_varint_put64_blk(blk: *mut hts_sys::cram_block, val: i64) -> c_int {
        unsafe {
            let b = blk.cast::<cram_block_layout>();
            (*b).idx = val as i32;
        }
        1
    }

    unsafe extern "C" fn test_varint_put64s_blk(blk: *mut hts_sys::cram_block, val: i64) -> c_int {
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
                cram_cram_codecs_c_410_cram_external_decode_block as usize as *mut c_void
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
            };
            let out_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
            let store_block =
                cram_cram_io_c_1388_cram_new_block(crate::htslib_rs::cram::CRAM_CONTENT_TYPE_EXTERNAL, 0);
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
            let ext_ptr = (&mut ext_block as *mut cram_block_layout).cast::<hts_sys::cram_block>();
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
                block: blocks.as_mut_ptr(),
                block_by_id: std::ptr::null_mut(),
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

            let direct_ptr = (&mut direct as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let wrong_hash_ptr =
                (&mut wrong_hash as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let colliding_ptr =
                (&mut colliding as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let non_external_ptr =
                (&mut non_external as *mut cram_block_layout).cast::<hts_sys::cram_block>();
            let mut block_by_id = [std::ptr::null_mut(); 768];
            block_by_id[17] = direct_ptr;
            block_by_id[256 + 502 % 251] = wrong_hash_ptr;
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
                block: blocks.as_mut_ptr(),
                block_by_id: block_by_id.as_mut_ptr(),
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
            let var_ptr = (&mut var_block as *mut cram_block_layout).cast::<hts_sys::cram_block>();
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
                block: blocks.as_mut_ptr(),
                block_by_id: std::ptr::null_mut(),
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
            let mut path = vec![0 as c_char; libc::PATH_MAX as usize];
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
            let mut out = vec![0 as c_char; libc::PATH_MAX as usize];

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
            std::ptr::copy_nonoverlapping(b"abc\0".as_ptr().cast::<c_char>(), repl, 4);
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
            let data = malloc(12).cast::<c_char>();
            std::ptr::copy_nonoverlapping(
                b"a\r\nbc\nz\0\0\0\0".as_ptr().cast::<c_char>(),
                data,
                11,
            );
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
}
