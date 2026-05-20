/*
Copyright (c) 2012-2020, 2022-2026 Genome Research Ltd.
Author: James Bonfield <jkb@sanger.ac.uk>

Rust translation of htslib/cram/cram_decode.c.
*/

use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type BGZF;
    pub type hts_tpool_process;
    pub type hts_tpool;
    pub type hFILE_backend;
    pub type hts_idx_t;
    pub type hts_filter_t;
    pub type hts_md5_context;
    pub type hts_tpool_result;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn hts_log(
        severity: htsLogLevel,
        context: *const ::core::ffi::c_char,
        format: *const ::core::ffi::c_char,
        ...
    );
    fn ksprintf(s: *mut kstring_t, fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn kputd(d: ::core::ffi::c_double, s: *mut kstring_t) -> ::core::ffi::c_int;
    fn ksplit_core(
        s: *mut ::core::ffi::c_char,
        delimiter: ::core::ffi::c_int,
        _max: *mut ::core::ffi::c_int,
        _offsets: *mut *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn hts_itr_next(
        fp: *mut BGZF,
        iter: *mut hts_itr_t,
        r: *mut ::core::ffi::c_void,
        data: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn hts_itr_multi_next(
        fd: *mut htsFile,
        iter: *mut hts_itr_t,
        r: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn hts_md5_init() -> *mut hts_md5_context;
    fn hts_md5_update(
        ctx: *mut hts_md5_context,
        data: *const ::core::ffi::c_void,
        size: ::core::ffi::c_ulong,
    );
    fn hts_md5_final(digest: *mut ::core::ffi::c_uchar, ctx: *mut hts_md5_context);
    fn hts_md5_destroy(ctx: *mut hts_md5_context);
    fn hseek(fp: *mut hFILE, offset: off_t, whence: ::core::ffi::c_int) -> off_t;
    fn hgetdelim(
        buffer: *mut ::core::ffi::c_char,
        size: size_t,
        delim: ::core::ffi::c_int,
        fp: *mut hFILE,
    ) -> ssize_t;
    fn sam_hdr_init() -> *mut sam_hdr_t;
    fn sam_hdr_destroy(h: *mut sam_hdr_t);
    fn sam_hdr_dup(h0: *const sam_hdr_t) -> *mut sam_hdr_t;
    fn sam_hdr_parse(l_text: size_t, text: *const ::core::ffi::c_char) -> *mut sam_hdr_t;
    fn sam_hdr_line_name(
        bh: *mut sam_hdr_t,
        type_0: *const ::core::ffi::c_char,
        pos: ::core::ffi::c_int,
    ) -> *const ::core::ffi::c_char;
    fn sam_hdr_find_tag_id(
        h: *mut sam_hdr_t,
        type_0: *const ::core::ffi::c_char,
        ID_key: *const ::core::ffi::c_char,
        ID_value: *const ::core::ffi::c_char,
        key: *const ::core::ffi::c_char,
        ks: *mut kstring_t,
    ) -> ::core::ffi::c_int;
    fn sam_hdr_name2tid(h: *mut sam_hdr_t, ref_0: *const ::core::ffi::c_char)
        -> ::core::ffi::c_int;
    fn sam_hdr_tid2name(h: *const sam_hdr_t, tid: ::core::ffi::c_int)
        -> *const ::core::ffi::c_char;
    fn bam_set1(
        bam_0: *mut bam1_t,
        l_qname: size_t,
        qname: *const ::core::ffi::c_char,
        flag: uint16_t,
        tid: int32_t,
        pos: hts_pos_t,
        mapq: uint8_t,
        n_cigar: size_t,
        cigar: *const uint32_t,
        mtid: int32_t,
        mpos: hts_pos_t,
        isize: hts_pos_t,
        l_seq: size_t,
        seq: *const ::core::ffi::c_char,
        qual: *const ::core::ffi::c_char,
        l_aux: size_t,
    ) -> ::core::ffi::c_int;
    fn bam_aux_get(b: *const bam1_t, tag: *const ::core::ffi::c_char) -> *mut uint8_t;
    fn cram_free_compression_header(hdr: *mut cram_block_compression_hdr);
    fn cram_new_block(
        content_type: cram_content_type,
        content_id: ::core::ffi::c_int,
    ) -> *mut cram_block;
    fn cram_read_block(fd: *mut cram_fd) -> *mut cram_block;
    fn cram_free_block(b: *mut cram_block);
    fn cram_uncompress_block(b: *mut cram_block) -> ::core::ffi::c_int;
    fn cram_free_container(c: *mut cram_container);
    fn cram_read_container(fd: *mut cram_fd) -> *mut cram_container;
    fn cram_seek(fd: *mut cram_fd, offset: off_t, whence: ::core::ffi::c_int)
        -> ::core::ffi::c_int;
    fn sam_hrecs_sort_order(hrecs: *mut sam_hrecs_t) -> sam_sort_order;
    fn cram_decoder_init(
        hdr: *mut cram_block_compression_hdr,
        codec: cram_encoding,
        data: *mut ::core::ffi::c_char,
        size: ::core::ffi::c_int,
        option: cram_external_type,
        version: ::core::ffi::c_int,
        vv: *mut varint_vec,
    ) -> *mut cram_codec;
    fn cram_codec_to_id(c: *mut cram_codec, id2: *mut ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn hts_tpool_dispatch2(
        p: *mut hts_tpool,
        q: *mut hts_tpool_process,
        func: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void>,
        arg: *mut ::core::ffi::c_void,
        nonblock: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn hts_tpool_process_qsize(q: *mut hts_tpool_process) -> ::core::ffi::c_int;
    fn hts_tpool_next_result_wait(q: *mut hts_tpool_process) -> *mut hts_tpool_result;
    fn hts_tpool_delete_result(r: *mut hts_tpool_result, free_data: ::core::ffi::c_int);
    fn hts_tpool_result_data(r: *mut hts_tpool_result) -> *mut ::core::ffi::c_void;
    fn hts_tpool_process_empty(q: *mut hts_tpool_process) -> ::core::ffi::c_int;
    fn hts_tpool_process_len(q: *mut hts_tpool_process) -> ::core::ffi::c_int;
    fn hts_tpool_process_sz(q: *mut hts_tpool_process) -> ::core::ffi::c_int;
    fn pthread_mutex_lock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_mutex_unlock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn cram_get_ref(
        fd: *mut cram_fd,
        id: ::core::ffi::c_int,
        start: hts_pos_t,
        end: hts_pos_t,
    ) -> *mut ::core::ffi::c_char;
    fn cram_ref_decr(r: *mut refs_t, id: ::core::ffi::c_int);
    fn cram_free_slice(s: *mut cram_slice);
    fn cram_read_slice(fd: *mut cram_fd) -> *mut cram_slice;
}
pub type size_t = usize;
pub type __int8_t = i8;
pub type __uint8_t = u8;
pub type __int16_t = i16;
pub type __uint16_t = u16;
pub type __int32_t = i32;
pub type __uint32_t = u32;
pub type __int64_t = i64;
pub type __uint64_t = u64;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __ssize_t = ::core::ffi::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub __pad5: size_t,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type off_t = __off_t;
pub type ssize_t = __ssize_t;
pub type int8_t = __int8_t;
pub type int16_t = __int16_t;
pub type int32_t = __int32_t;
pub type int64_t = __int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
pub type ptrdiff_t = isize;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type htsLogLevel = ::core::ffi::c_uint;
pub const HTS_LOG_TRACE: htsLogLevel = 6;
pub const HTS_LOG_DEBUG: htsLogLevel = 5;
pub const HTS_LOG_INFO: htsLogLevel = 4;
pub const HTS_LOG_WARNING: htsLogLevel = 3;
pub const HTS_LOG_ERROR: htsLogLevel = 1;
pub const HTS_LOG_OFF: htsLogLevel = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kstring_t {
    pub l: size_t,
    pub m: size_t,
    pub s: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_fd {
    pub fp: *mut hFILE,
    pub mode: ::core::ffi::c_int,
    pub version: ::core::ffi::c_int,
    pub file_def: *mut cram_file_def,
    pub header: *mut sam_hdr_t,
    pub prefix: *mut ::core::ffi::c_char,
    pub record_counter: int64_t,
    pub err: ::core::ffi::c_int,
    pub ctr: *mut cram_container,
    pub ctr_mt: *mut cram_container,
    pub first_base: ::core::ffi::c_int,
    pub last_base: ::core::ffi::c_int,
    pub refs: *mut refs_t,
    pub ref_0: *mut ::core::ffi::c_char,
    pub ref_free: *mut ::core::ffi::c_char,
    pub ref_id: ::core::ffi::c_int,
    pub ref_start: hts_pos_t,
    pub ref_end: hts_pos_t,
    pub ref_fn: *mut ::core::ffi::c_char,
    pub level: ::core::ffi::c_int,
    pub m: [*mut cram_metrics; 47],
    pub tags_used: *mut kh_m_metrics_t,
    pub decode_md: ::core::ffi::c_int,
    pub seqs_per_slice: ::core::ffi::c_int,
    pub bases_per_slice: ::core::ffi::c_int,
    pub slices_per_container: ::core::ffi::c_int,
    pub embed_ref: ::core::ffi::c_int,
    pub no_ref: ::core::ffi::c_int,
    pub no_ref_counter: ::core::ffi::c_int,
    pub ignore_md5: ::core::ffi::c_int,
    pub use_bz2: ::core::ffi::c_int,
    pub use_rans: ::core::ffi::c_int,
    pub use_lzma: ::core::ffi::c_int,
    pub use_fqz: ::core::ffi::c_int,
    pub use_tok: ::core::ffi::c_int,
    pub use_arith: ::core::ffi::c_int,
    pub shared_ref: ::core::ffi::c_int,
    pub required_fields: ::core::ffi::c_uint,
    pub store_md: ::core::ffi::c_int,
    pub store_nm: ::core::ffi::c_int,
    pub range: cram_range,
    pub bam_flag_swap: [::core::ffi::c_uint; 4096],
    pub cram_flag_swap: [::core::ffi::c_uint; 4096],
    pub L1: [::core::ffi::c_uchar; 256],
    pub L2: [::core::ffi::c_uchar; 256],
    pub cram_sub_matrix: [[::core::ffi::c_char; 32]; 32],
    pub index_sz: ::core::ffi::c_int,
    pub index: *mut cram_index,
    pub first_container: off_t,
    pub curr_position: off_t,
    pub eof: ::core::ffi::c_int,
    pub last_slice: ::core::ffi::c_int,
    pub last_RI_count: ::core::ffi::c_int,
    pub multi_seq: ::core::ffi::c_int,
    pub multi_seq_user: ::core::ffi::c_int,
    pub unsorted: ::core::ffi::c_int,
    pub last_mapped: ::core::ffi::c_int,
    pub empty_container: ::core::ffi::c_int,
    pub own_pool: ::core::ffi::c_int,
    pub pool: *mut hts_tpool,
    pub rqueue: *mut hts_tpool_process,
    pub metrics_lock: pthread_mutex_t,
    pub ref_lock: pthread_mutex_t,
    pub range_lock: pthread_mutex_t,
    pub bl: *mut spare_bams,
    pub bam_list_lock: pthread_mutex_t,
    pub job_pending: *mut ::core::ffi::c_void,
    pub ooc: ::core::ffi::c_int,
    pub lossy_read_names: ::core::ffi::c_int,
    pub tlen_approx: ::core::ffi::c_int,
    pub tlen_zero: ::core::ffi::c_int,
    pub idxfp: *mut BGZF,
    pub vv: varint_vec,
    pub ap_delta: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct varint_vec {
    pub varint_decode32_crc: Option<
        unsafe extern "C" fn(*mut cram_fd, *mut int32_t, *mut uint32_t) -> ::core::ffi::c_int,
    >,
    pub varint_decode32s_crc: Option<
        unsafe extern "C" fn(*mut cram_fd, *mut int32_t, *mut uint32_t) -> ::core::ffi::c_int,
    >,
    pub varint_decode64_crc: Option<
        unsafe extern "C" fn(*mut cram_fd, *mut int64_t, *mut uint32_t) -> ::core::ffi::c_int,
    >,
    pub varint_get32: Option<
        unsafe extern "C" fn(
            *mut *mut ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_int,
        ) -> int64_t,
    >,
    pub varint_get32s: Option<
        unsafe extern "C" fn(
            *mut *mut ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_int,
        ) -> int64_t,
    >,
    pub varint_get64: Option<
        unsafe extern "C" fn(
            *mut *mut ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_int,
        ) -> int64_t,
    >,
    pub varint_get64s: Option<
        unsafe extern "C" fn(
            *mut *mut ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_int,
        ) -> int64_t,
    >,
    pub varint_put32: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_char,
            *mut ::core::ffi::c_char,
            int32_t,
        ) -> ::core::ffi::c_int,
    >,
    pub varint_put32s: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_char,
            *mut ::core::ffi::c_char,
            int32_t,
        ) -> ::core::ffi::c_int,
    >,
    pub varint_put64: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_char,
            *mut ::core::ffi::c_char,
            int64_t,
        ) -> ::core::ffi::c_int,
    >,
    pub varint_put64s: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_char,
            *mut ::core::ffi::c_char,
            int64_t,
        ) -> ::core::ffi::c_int,
    >,
    pub varint_put32_blk:
        Option<unsafe extern "C" fn(*mut cram_block, int32_t) -> ::core::ffi::c_int>,
    pub varint_put32s_blk:
        Option<unsafe extern "C" fn(*mut cram_block, int32_t) -> ::core::ffi::c_int>,
    pub varint_put64_blk:
        Option<unsafe extern "C" fn(*mut cram_block, int64_t) -> ::core::ffi::c_int>,
    pub varint_put64s_blk:
        Option<unsafe extern "C" fn(*mut cram_block, int64_t) -> ::core::ffi::c_int>,
    pub varint_size: Option<unsafe extern "C" fn(int64_t) -> ::core::ffi::c_int>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_block {
    pub method: cram_block_method_int,
    pub orig_method: cram_block_method_int,
    pub content_type: cram_content_type,
    pub content_id: int32_t,
    pub comp_size: int32_t,
    pub uncomp_size: int32_t,
    pub crc32: uint32_t,
    pub idx: int32_t,
    pub data: *mut ::core::ffi::c_uchar,
    pub alloc: size_t,
    pub byte: size_t,
    pub bit: ::core::ffi::c_int,
    pub m: *mut cram_metrics,
    pub crc32_checked: ::core::ffi::c_int,
    pub crc_part: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_metrics {
    pub trial: ::core::ffi::c_int,
    pub next_trial: ::core::ffi::c_int,
    pub consistency: ::core::ffi::c_int,
    pub sz: [::core::ffi::c_int; 32],
    pub input_avg_sz: ::core::ffi::c_int,
    pub input_avg_delta: ::core::ffi::c_int,
    pub method: ::core::ffi::c_int,
    pub revised_method: ::core::ffi::c_int,
    pub strat: ::core::ffi::c_int,
    pub cnt: [::core::ffi::c_int; 32],
    pub extra: [::core::ffi::c_double; 32],
    pub unpackable: ::core::ffi::c_int,
}
pub type cram_content_type = ::core::ffi::c_int;
pub const CORE: cram_content_type = 5;
pub const EXTERNAL: cram_content_type = 4;
pub const UNMAPPED_SLICE: cram_content_type = 3;
pub const MAPPED_SLICE: cram_content_type = 2;
pub const COMPRESSION_HEADER: cram_content_type = 1;
pub const FILE_HEADER: cram_content_type = 0;
pub const CT_ERROR: cram_content_type = -1;
pub type cram_block_method_int = ::core::ffi::c_int;
pub const ARITH_PR193: cram_block_method_int = 31;
pub const ARITH_PR192: cram_block_method_int = 30;
pub const ARITH_PR129: cram_block_method_int = 29;
pub const ARITH_PR128: cram_block_method_int = 28;
pub const ARITH_PR9: cram_block_method_int = 27;
pub const ARITH_PR64: cram_block_method_int = 26;
pub const ARITH_PR1: cram_block_method_int = 25;
pub const TOKA: cram_block_method_int = 24;
pub const RANS_PR193: cram_block_method_int = 23;
pub const RANS_PR192: cram_block_method_int = 22;
pub const RANS_PR129: cram_block_method_int = 21;
pub const RANS_PR128: cram_block_method_int = 20;
pub const RANS_PR9: cram_block_method_int = 19;
pub const RANS_PR64: cram_block_method_int = 18;
pub const RANS_PR1: cram_block_method_int = 17;
pub const RANS1: cram_block_method_int = 16;
pub const FQZ_d: cram_block_method_int = 15;
pub const FQZ_c: cram_block_method_int = 14;
pub const FQZ_b: cram_block_method_int = 13;
pub const GZIP_1: cram_block_method_int = 12;
pub const GZIP_RLE: cram_block_method_int = 11;
pub const TOK3: cram_block_method_int = 8;
pub const FQZ: cram_block_method_int = 7;
pub const ARITH_PR0: cram_block_method_int = 6;
pub const ARITH: cram_block_method_int = 6;
pub const RANS_PR0: cram_block_method_int = 5;
pub const RANSPR: cram_block_method_int = 5;
pub const RANS0: cram_block_method_int = 4;
pub const RANS: cram_block_method_int = 4;
pub const LZMA: cram_block_method_int = 3;
pub const BZIP2: cram_block_method_int = 2;
pub const GZIP: cram_block_method_int = 1;
pub const RAW: cram_block_method_int = 0;
pub const BM_ERROR: cram_block_method_int = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spare_bams {
    pub bams: *mut *mut bam_seq_t,
    pub next: *mut spare_bams,
}
pub type bam_seq_t = bam1_t;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct bam1_t {
    pub core: bam1_core_t,
    pub id: uint64_t,
    pub data: *mut uint8_t,
    pub l_data: ::core::ffi::c_int,
    pub m_data: uint32_t,
    #[bitfield(name = "mempolicy", ty = "uint32_t", bits = "0..=1")]
    #[bitfield(name = "c2rust_unnamed", ty = "uint32_t", bits = "2..=31")]
    pub mempolicy_c2rust_unnamed: [u8; 4],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bam1_core_t {
    pub pos: hts_pos_t,
    pub tid: int32_t,
    pub bin: uint16_t,
    pub qual: uint8_t,
    pub l_extranul: uint8_t,
    pub flag: uint16_t,
    pub l_qname: uint16_t,
    pub n_cigar: uint32_t,
    pub l_qseq: int32_t,
    pub mtid: int32_t,
    pub mpos: hts_pos_t,
    pub isize_0: hts_pos_t,
}
pub type hts_pos_t = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_index {
    pub nslice: ::core::ffi::c_int,
    pub nalloc: ::core::ffi::c_int,
    pub e: *mut cram_index,
    pub refid: ::core::ffi::c_int,
    pub start: ::core::ffi::c_int,
    pub end: ::core::ffi::c_int,
    pub nseq: ::core::ffi::c_int,
    pub slice: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
    pub offset: int64_t,
    pub e_next: *mut cram_index,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_range {
    pub refid: ::core::ffi::c_int,
    pub start: int64_t,
    pub end: int64_t,
}
pub type kh_m_metrics_t = kh_m_metrics_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_m_metrics_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut khint32_t,
    pub vals: *mut *mut cram_metrics,
}
pub type khint32_t = ::core::ffi::c_uint;
pub type khint_t = khint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct refs_t {
    pub pool: *mut string_alloc_t,
    pub h_meta: *mut kh_refs_t,
    pub ref_id: *mut *mut ref_entry,
    pub nref: ::core::ffi::c_int,
    pub fn_0: *mut ::core::ffi::c_char,
    pub fp: *mut BGZF,
    pub count: ::core::ffi::c_int,
    pub lock: pthread_mutex_t,
    pub last: *mut ref_entry,
    pub last_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ref_entry {
    pub name: *mut ::core::ffi::c_char,
    pub fn_0: *mut ::core::ffi::c_char,
    pub length: int64_t,
    pub LN_length: int64_t,
    pub offset: int64_t,
    pub bases_per_line: ::core::ffi::c_int,
    pub line_length: ::core::ffi::c_int,
    pub count: int64_t,
    pub seq: *mut ::core::ffi::c_char,
    pub mf: *mut mFILE,
    pub is_md5: ::core::ffi::c_int,
    pub validated_md5: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mFILE {
    pub fp: *mut FILE,
    pub data: *mut ::core::ffi::c_char,
    pub alloced: size_t,
    pub eof: ::core::ffi::c_int,
    pub mode: ::core::ffi::c_int,
    pub size: size_t,
    pub offset: size_t,
    pub flush_pos: size_t,
}
pub type kh_refs_t = kh_refs_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_refs_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut kh_cstr_t,
    pub vals: *mut *mut ref_entry,
}
pub type kh_cstr_t = *const ::core::ffi::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct string_alloc_t {
    pub max_length: size_t,
    pub nstrings: size_t,
    pub max_strings: size_t,
    pub strings: *mut string_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct string_t {
    pub str_0: *mut ::core::ffi::c_char,
    pub used: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_container {
    pub length: int32_t,
    pub ref_seq_id: int32_t,
    pub ref_seq_start: int64_t,
    pub ref_seq_span: int64_t,
    pub record_counter: int64_t,
    pub num_bases: int64_t,
    pub num_records: int32_t,
    pub num_blocks: int32_t,
    pub num_landmarks: int32_t,
    pub landmark: *mut int32_t,
    pub offset: size_t,
    pub comp_hdr: *mut cram_block_compression_hdr,
    pub comp_hdr_block: *mut cram_block,
    pub max_slice: ::core::ffi::c_int,
    pub curr_slice: ::core::ffi::c_int,
    pub curr_slice_mt: ::core::ffi::c_int,
    pub max_rec: ::core::ffi::c_int,
    pub curr_rec: ::core::ffi::c_int,
    pub max_c_rec: ::core::ffi::c_int,
    pub curr_c_rec: ::core::ffi::c_int,
    pub slice_rec: ::core::ffi::c_int,
    pub curr_ref: ::core::ffi::c_int,
    pub last_pos: int64_t,
    pub slices: *mut *mut cram_slice,
    pub slice: *mut cram_slice,
    pub pos_sorted: ::core::ffi::c_int,
    pub max_apos: int64_t,
    pub last_slice: ::core::ffi::c_int,
    pub multi_seq: ::core::ffi::c_int,
    pub unsorted: ::core::ffi::c_int,
    pub qs_seq_orient: ::core::ffi::c_int,
    pub ref_id: ::core::ffi::c_int,
    pub ref_start: hts_pos_t,
    pub first_base: hts_pos_t,
    pub last_base: hts_pos_t,
    pub ref_end: hts_pos_t,
    pub ref_0: *mut ::core::ffi::c_char,
    pub embed_ref: ::core::ffi::c_int,
    pub no_ref: ::core::ffi::c_int,
    pub bams: *mut *mut bam_seq_t,
    pub stats: [*mut cram_stats; 47],
    pub tags_used: *mut kh_m_tagmap_t,
    pub refs_used: *mut ::core::ffi::c_int,
    pub crc32: uint32_t,
    pub s_num_bases: uint64_t,
    pub s_aux_bytes: uint64_t,
    pub n_mapped: uint32_t,
    pub ref_free: ::core::ffi::c_int,
}
pub type kh_m_tagmap_t = kh_m_tagmap_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_m_tagmap_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut khint32_t,
    pub vals: *mut *mut cram_tag_map,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_tag_map {
    pub codec: *mut cram_codec,
    pub blk: *mut cram_block,
    pub blk2: *mut cram_block,
    pub m: *mut cram_metrics,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_codec {
    pub codec: cram_encoding,
    pub out: *mut cram_block,
    pub vv: *mut varint_vec,
    pub codec_id: ::core::ffi::c_int,
    pub free: Option<unsafe extern "C" fn(*mut cram_codec) -> ()>,
    pub decode: Option<
        unsafe extern "C" fn(
            *mut cram_slice,
            *mut cram_codec,
            *mut cram_block,
            *mut ::core::ffi::c_char,
            *mut ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
    pub encode: Option<
        unsafe extern "C" fn(
            *mut cram_slice,
            *mut cram_codec,
            *mut ::core::ffi::c_char,
            ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
    pub store: Option<
        unsafe extern "C" fn(
            *mut cram_codec,
            *mut cram_block,
            *mut ::core::ffi::c_char,
            ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
    pub size: Option<unsafe extern "C" fn(*mut cram_slice, *mut cram_codec) -> ::core::ffi::c_int>,
    pub flush: Option<unsafe extern "C" fn(*mut cram_codec) -> ::core::ffi::c_int>,
    pub get_block:
        Option<unsafe extern "C" fn(*mut cram_slice, *mut cram_codec) -> *mut cram_block>,
    pub describe:
        Option<unsafe extern "C" fn(*mut cram_codec, *mut kstring_t) -> ::core::ffi::c_int>,
    pub u: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub huffman: cram_huffman_decoder,
    pub external: cram_external_decoder,
    pub beta: cram_beta_decoder,
    pub gamma: cram_gamma_decoder,
    pub subexp: cram_subexp_decoder,
    pub byte_array_len: cram_byte_array_len_decoder,
    pub byte_array_stop: cram_byte_array_stop_decoder,
    pub xpack: cram_xpack_decoder,
    pub xrle: cram_xrle_decoder,
    pub xdelta: cram_xdelta_decoder,
    pub xconst: cram_const_codec,
    pub varint: cram_varint_decoder,
    pub e_huffman: cram_huffman_encoder,
    pub e_external: cram_external_decoder,
    pub e_byte_array_stop: cram_byte_array_stop_decoder,
    pub e_byte_array_len: cram_byte_array_len_encoder,
    pub e_beta: cram_beta_decoder,
    pub e_xpack: cram_xpack_decoder,
    pub e_xrle: cram_xrle_decoder,
    pub e_xdelta: cram_xdelta_decoder,
    pub e_xconst: cram_const_codec,
    pub e_varint: cram_varint_decoder,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_varint_decoder {
    pub content_id: int32_t,
    pub offset: int64_t,
    pub type_0: cram_external_type,
}
pub type cram_external_type = ::core::ffi::c_uint;
pub const E_SLONG: cram_external_type = 7;
pub const E_SINT: cram_external_type = 6;
pub const E_BYTE_ARRAY_BLOCK: cram_external_type = 5;
pub const E_BYTE_ARRAY: cram_external_type = 4;
pub const E_BYTE: cram_external_type = 3;
pub const E_LONG: cram_external_type = 2;
pub const E_INT: cram_external_type = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_const_codec {
    pub val: int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_xdelta_decoder {
    pub last: int64_t,
    pub word_size: uint8_t,
    pub sub_encoding: cram_encoding,
    pub sub_codec_dat: *mut ::core::ffi::c_void,
    pub sub_codec: *mut cram_codec,
}
pub type cram_encoding = ::core::ffi::c_uint;
pub const E_NUM_CODECS: cram_encoding = 54;
pub const E_XDELTA: cram_encoding = 53;
pub const E_XRLE: cram_encoding = 52;
pub const E_XPACK: cram_encoding = 51;
pub const E_XHUFFMAN: cram_encoding = 50;
pub const E_CONST_INT: cram_encoding = 44;
pub const E_CONST_BYTE: cram_encoding = 43;
pub const E_VARINT_SIGNED: cram_encoding = 42;
pub const E_VARINT_UNSIGNED: cram_encoding = 41;
pub const E_GAMMA: cram_encoding = 9;
pub const E_GOLOMB_RICE: cram_encoding = 8;
pub const E_SUBEXP: cram_encoding = 7;
pub const E_BETA: cram_encoding = 6;
pub const E_BYTE_ARRAY_STOP: cram_encoding = 5;
pub const E_BYTE_ARRAY_LEN: cram_encoding = 4;
pub const E_HUFFMAN: cram_encoding = 3;
pub const E_GOLOMB: cram_encoding = 2;
pub const E_EXTERNAL: cram_encoding = 1;
pub const E_NULL: cram_encoding = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_xrle_decoder {
    pub len_encoding: cram_encoding,
    pub lit_encoding: cram_encoding,
    pub len_dat: *mut ::core::ffi::c_void,
    pub lit_dat: *mut ::core::ffi::c_void,
    pub len_codec: *mut cram_codec,
    pub lit_codec: *mut cram_codec,
    pub cur_len: ::core::ffi::c_int,
    pub cur_lit: ::core::ffi::c_int,
    pub rep_score: [::core::ffi::c_int; 256],
    pub to_flush: *mut ::core::ffi::c_char,
    pub to_flush_size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_xpack_decoder {
    pub nbits: int32_t,
    pub sub_encoding: cram_encoding,
    pub sub_codec_dat: *mut ::core::ffi::c_void,
    pub sub_codec: *mut cram_codec,
    pub nval: ::core::ffi::c_int,
    pub rmap: [uint32_t; 256],
    pub map: [::core::ffi::c_int; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_beta_decoder {
    pub offset: int32_t,
    pub nbits: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_byte_array_len_encoder {
    pub len_encoding: cram_encoding,
    pub val_encoding: cram_encoding,
    pub len_dat: *mut ::core::ffi::c_void,
    pub val_dat: *mut ::core::ffi::c_void,
    pub len_codec: *mut cram_codec,
    pub val_codec: *mut cram_codec,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_byte_array_stop_decoder {
    pub stop: ::core::ffi::c_uchar,
    pub content_id: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_external_decoder {
    pub content_id: int32_t,
    pub type_0: cram_external_type,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_huffman_encoder {
    pub codes: *mut cram_huffman_code,
    pub nvals: ::core::ffi::c_int,
    pub val2code: [::core::ffi::c_int; 129],
    pub option: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_huffman_code {
    pub symbol: int64_t,
    pub p: int32_t,
    pub code: int32_t,
    pub len: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_byte_array_len_decoder {
    pub len_codec: *mut cram_codec,
    pub val_codec: *mut cram_codec,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_subexp_decoder {
    pub offset: int32_t,
    pub k: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_gamma_decoder {
    pub offset: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_huffman_decoder {
    pub ncodes: ::core::ffi::c_int,
    pub codes: *mut cram_huffman_code,
    pub option: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_slice {
    pub hdr: *mut cram_block_slice_hdr,
    pub hdr_block: *mut cram_block,
    pub block: *mut *mut cram_block,
    pub block_by_id: *mut *mut cram_block,
    pub last_apos: int64_t,
    pub max_apos: int64_t,
    pub crecs: *mut cram_record,
    pub cigar: *mut uint32_t,
    pub cigar_alloc: uint32_t,
    pub ncigar: uint32_t,
    pub features: *mut cram_feature,
    pub nfeatures: uint32_t,
    pub afeatures: uint32_t,
    pub TN: *mut uint32_t,
    pub nTN: ::core::ffi::c_int,
    pub aTN: ::core::ffi::c_int,
    pub name_blk: *mut cram_block,
    pub seqs_blk: *mut cram_block,
    pub qual_blk: *mut cram_block,
    pub base_blk: *mut cram_block,
    pub soft_blk: *mut cram_block,
    pub aux_blk: *mut cram_block,
    pub pair_keys: *mut string_alloc_t,
    pub pair: [*mut kh_m_s2i_t; 2],
    pub ref_0: *mut ::core::ffi::c_char,
    pub ref_start: hts_pos_t,
    pub ref_end: hts_pos_t,
    pub ref_id: ::core::ffi::c_int,
    pub naux_block: ::core::ffi::c_int,
    pub aux_block: *mut *mut cram_block,
    pub data_series: ::core::ffi::c_uint,
    pub decode_md: ::core::ffi::c_int,
    pub max_rec: ::core::ffi::c_int,
    pub curr_rec: ::core::ffi::c_int,
    pub slice_num: ::core::ffi::c_int,
}
pub type kh_m_s2i_t = kh_m_s2i_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_m_s2i_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut kh_cstr_t,
    pub vals: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union cram_feature {
    pub X: C2RustUnnamed_10,
    pub B: C2RustUnnamed_9,
    pub b: C2RustUnnamed_8,
    pub Q: C2RustUnnamed_7,
    pub S: C2RustUnnamed_6,
    pub I: C2RustUnnamed_5,
    pub i: C2RustUnnamed_4,
    pub D: C2RustUnnamed_3,
    pub N: C2RustUnnamed_2,
    pub P: C2RustUnnamed_1,
    pub H: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub base: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
    pub seq_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_6 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
    pub seq_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_7 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub qual: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_8 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub seq_idx: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_9 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub base: ::core::ffi::c_int,
    pub qual: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_10 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub base: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_record {
    pub s: *mut cram_slice,
    pub ref_id: int32_t,
    pub flags: int32_t,
    pub cram_flags: int32_t,
    pub len: int32_t,
    pub apos: int64_t,
    pub rg: int32_t,
    pub name: int32_t,
    pub name_len: int32_t,
    pub mate_line: int32_t,
    pub mate_ref_id: int32_t,
    pub mate_pos: int64_t,
    pub tlen: int64_t,
    pub explicit_tlen: int64_t,
    pub ntags: int32_t,
    pub aux: uint32_t,
    pub aux_size: uint32_t,
    pub TN_idx: int32_t,
    pub TL: ::core::ffi::c_int,
    pub seq: uint32_t,
    pub qual: uint32_t,
    pub cigar: uint32_t,
    pub ncigar: int32_t,
    pub aend: int64_t,
    pub mqual: int32_t,
    pub feature: uint32_t,
    pub nfeature: uint32_t,
    pub mate_flags: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_block_slice_hdr {
    pub content_type: cram_content_type,
    pub ref_seq_id: int32_t,
    pub ref_seq_start: int64_t,
    pub ref_seq_span: int64_t,
    pub num_records: int32_t,
    pub record_counter: int64_t,
    pub num_blocks: int32_t,
    pub num_content_ids: int32_t,
    pub block_content_ids: *mut int32_t,
    pub ref_base_id: int32_t,
    pub md5: [::core::ffi::c_uchar; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_stats {
    pub freqs: [::core::ffi::c_int; 1024],
    pub h: *mut kh_m_i2i_t,
    pub nsamp: ::core::ffi::c_int,
    pub nvals: ::core::ffi::c_int,
    pub min_val: int64_t,
    pub max_val: int64_t,
}
pub type kh_m_i2i_t = kh_m_i2i_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_m_i2i_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut khint64_t,
    pub vals: *mut ::core::ffi::c_int,
}
pub type khint64_t = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_block_compression_hdr {
    pub ref_seq_id: int32_t,
    pub ref_seq_start: int64_t,
    pub ref_seq_span: int64_t,
    pub num_records: int32_t,
    pub num_landmarks: int32_t,
    pub landmark: *mut int32_t,
    pub read_names_included: ::core::ffi::c_int,
    pub AP_delta: ::core::ffi::c_int,
    pub substitution_matrix: [[::core::ffi::c_char; 4]; 5],
    pub no_ref: ::core::ffi::c_int,
    pub qs_seq_orient: ::core::ffi::c_int,
    pub TD_blk: *mut cram_block,
    pub nTL: ::core::ffi::c_int,
    pub TL: *mut *mut ::core::ffi::c_uchar,
    pub TD_hash: *mut kh_m_s2i_t,
    pub TD_keys: *mut string_alloc_t,
    pub preservation_map: *mut kh_map_t,
    pub rec_encoding_map: [*mut cram_map; 32],
    pub tag_encoding_map: [*mut cram_map; 32],
    pub codecs: [*mut cram_codec; 47],
    pub uncomp: *mut ::core::ffi::c_char,
    pub uncomp_size: size_t,
    pub uncomp_alloc: size_t,
    pub ncodecs: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_map {
    pub key: ::core::ffi::c_int,
    pub encoding: cram_encoding,
    pub offset: ::core::ffi::c_int,
    pub size: ::core::ffi::c_int,
    pub codec: *mut cram_codec,
    pub next: *mut cram_map,
}
pub type kh_map_t = kh_map_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_map_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut kh_cstr_t,
    pub vals: *mut pmap_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pmap_t {
    pub i: ::core::ffi::c_int,
    pub p: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hdr_t {
    pub n_targets: int32_t,
    pub ignore_sam_err: int32_t,
    pub l_text: size_t,
    pub target_len: *mut uint32_t,
    pub cigar_tab: *const int8_t,
    pub target_name: *mut *mut ::core::ffi::c_char,
    pub text: *mut ::core::ffi::c_char,
    pub sdict: *mut ::core::ffi::c_void,
    pub hrecs: *mut sam_hrecs_t,
    pub ref_count: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hrecs_t {
    pub h: *mut kh_sam_hrecs_t_t,
    pub first_line: *mut sam_hrec_type_t,
    pub str_pool: *mut string_alloc_t,
    pub type_pool: *mut pool_alloc_t,
    pub tag_pool: *mut pool_alloc_t,
    pub nref: ::core::ffi::c_int,
    pub ref_sz: ::core::ffi::c_int,
    pub ref_0: *mut sam_hrec_sq_t,
    pub ref_hash: *mut kh_m_s2i_t,
    pub nrg: ::core::ffi::c_int,
    pub rg_sz: ::core::ffi::c_int,
    pub rg: *mut sam_hrec_rg_t,
    pub rg_hash: *mut kh_m_s2i_t,
    pub npg: ::core::ffi::c_int,
    pub pg_sz: ::core::ffi::c_int,
    pub npg_end: ::core::ffi::c_int,
    pub npg_end_alloc: ::core::ffi::c_int,
    pub pg: *mut sam_hrec_pg_t,
    pub pg_hash: *mut kh_m_s2i_t,
    pub pg_end: *mut ::core::ffi::c_int,
    pub ID_buf: *mut ::core::ffi::c_char,
    pub ID_buf_sz: uint32_t,
    pub ID_cnt: ::core::ffi::c_int,
    pub dirty: ::core::ffi::c_int,
    pub refs_changed: ::core::ffi::c_int,
    pub pgs_changed: ::core::ffi::c_int,
    pub type_count: ::core::ffi::c_int,
    pub type_order: *mut [::core::ffi::c_char; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hrec_pg_t {
    pub name: *const ::core::ffi::c_char,
    pub ty: *mut sam_hrec_type_t,
    pub name_len: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
    pub prev_id: ::core::ffi::c_int,
}
pub type sam_hrec_type_t = sam_hrec_type_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hrec_type_s {
    pub next: *mut sam_hrec_type_s,
    pub prev: *mut sam_hrec_type_s,
    pub global_next: *mut sam_hrec_type_s,
    pub global_prev: *mut sam_hrec_type_s,
    pub tag: *mut sam_hrec_tag_t,
    pub type_0: khint32_t,
}
pub type sam_hrec_tag_t = sam_hrec_tag_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hrec_tag_s {
    pub next: *mut sam_hrec_tag_s,
    pub str_0: *const ::core::ffi::c_char,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hrec_rg_t {
    pub name: *const ::core::ffi::c_char,
    pub ty: *mut sam_hrec_type_t,
    pub name_len: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sam_hrec_sq_t {
    pub name: *const ::core::ffi::c_char,
    pub len: hts_pos_t,
    pub ty: *mut sam_hrec_type_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pool_alloc_t {
    pub dsize: size_t,
    pub psize: size_t,
    pub npools: size_t,
    pub pools: *mut pool_t,
    pub free: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pool_t {
    pub pool: *mut ::core::ffi::c_void,
    pub used: size_t,
}
pub type kh_sam_hrecs_t_t = kh_sam_hrecs_t_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_sam_hrecs_t_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut khint32_t,
    pub vals: *mut *mut sam_hrec_type_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_file_def {
    pub magic: [::core::ffi::c_char; 4],
    pub major_version: uint8_t,
    pub minor_version: uint8_t,
    pub file_id: [::core::ffi::c_char; 20],
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct hFILE {
    pub buffer: *mut ::core::ffi::c_char,
    pub begin: *mut ::core::ffi::c_char,
    pub end: *mut ::core::ffi::c_char,
    pub limit: *mut ::core::ffi::c_char,
    pub backend: *const hFILE_backend,
    pub offset: off_t,
    #[bitfield(name = "at_eof", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "mobile", ty = "::core::ffi::c_uint", bits = "1..=1")]
    #[bitfield(name = "readonly", ty = "::core::ffi::c_uint", bits = "2..=2")]
    #[bitfield(name = "preserve", ty = "::core::ffi::c_uint", bits = "3..=3")]
    pub at_eof_mobile_readonly_preserve: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
    pub has_errno: ::core::ffi::c_int,
}
pub type htsFormatCategory = ::core::ffi::c_uint;
pub const category_maximum: htsFormatCategory = 32767;
pub const region_list: htsFormatCategory = 4;
pub const index_file: htsFormatCategory = 3;
pub const variant_data: htsFormatCategory = 2;
pub const sequence_data: htsFormatCategory = 1;
pub const unknown_category: htsFormatCategory = 0;
pub type htsExactFormat = ::core::ffi::c_uint;
pub const format_maximum: htsExactFormat = 32767;
pub const d4_format: htsExactFormat = 21;
pub const hts_crypt4gh_format: htsExactFormat = 20;
pub const fqi_format: htsExactFormat = 19;
pub const fai_format: htsExactFormat = 18;
pub const fastq_format: htsExactFormat = 17;
pub const fasta_format: htsExactFormat = 16;
pub const empty_format: htsExactFormat = 15;
pub const json: htsExactFormat = 14;
pub const htsget: htsExactFormat = 14;
pub const bed: htsExactFormat = 13;
pub const tbi: htsExactFormat = 12;
pub const gzi: htsExactFormat = 11;
pub const csi: htsExactFormat = 10;
pub const bcf: htsExactFormat = 9;
pub const vcf: htsExactFormat = 8;
pub const crai: htsExactFormat = 7;
pub const cram: htsExactFormat = 6;
pub const bai: htsExactFormat = 5;
pub const bam: htsExactFormat = 4;
pub const sam: htsExactFormat = 3;
pub const text_format: htsExactFormat = 2;
pub const binary_format: htsExactFormat = 1;
pub const unknown_format: htsExactFormat = 0;
pub type htsCompression = ::core::ffi::c_uint;
pub const compression_maximum: htsCompression = 32767;
pub const zstd_compression: htsCompression = 7;
pub const xz_compression: htsCompression = 6;
pub const razf_compression: htsCompression = 5;
pub const bzip2_compression: htsCompression = 4;
pub const custom: htsCompression = 3;
pub const bgzf: htsCompression = 2;
pub const gzip: htsCompression = 1;
pub const no_compression: htsCompression = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct htsFormat {
    pub category: htsFormatCategory,
    pub format: htsExactFormat,
    pub version: C2RustUnnamed_11,
    pub compression: htsCompression,
    pub compression_level: ::core::ffi::c_short,
    pub specific: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub major: ::core::ffi::c_short,
    pub minor: ::core::ffi::c_short,
}
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct htsFile {
    #[bitfield(name = "is_bin", ty = "uint32_t", bits = "0..=0")]
    #[bitfield(name = "is_write", ty = "uint32_t", bits = "1..=1")]
    #[bitfield(name = "is_be", ty = "uint32_t", bits = "2..=2")]
    #[bitfield(name = "is_cram", ty = "uint32_t", bits = "3..=3")]
    #[bitfield(name = "is_bgzf", ty = "uint32_t", bits = "4..=4")]
    #[bitfield(name = "dummy", ty = "uint32_t", bits = "5..=31")]
    pub is_bin_is_write_is_be_is_cram_is_bgzf_dummy: [u8; 4],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 4],
    pub lineno: int64_t,
    pub line: kstring_t,
    pub fn_0: *mut ::core::ffi::c_char,
    pub fn_aux: *mut ::core::ffi::c_char,
    pub fp: C2RustUnnamed_12,
    pub state: *mut ::core::ffi::c_void,
    pub format: htsFormat,
    pub idx: *mut hts_idx_t,
    pub fnidx: *const ::core::ffi::c_char,
    pub bam_header: *mut sam_hdr_t,
    pub filter: *mut hts_filter_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub bgzf: *mut BGZF,
    pub cram: *mut cram_fd,
    pub hfile: *mut hFILE,
}
pub type sam_fields = ::core::ffi::c_uint;
pub const SAM_RGAUX: sam_fields = 4096;
pub const SAM_AUX: sam_fields = 2048;
pub const SAM_QUAL: sam_fields = 1024;
pub const SAM_SEQ: sam_fields = 512;
pub const SAM_TLEN: sam_fields = 256;
pub const SAM_PNEXT: sam_fields = 128;
pub const SAM_RNEXT: sam_fields = 64;
pub const SAM_CIGAR: sam_fields = 32;
pub const SAM_MAPQ: sam_fields = 16;
pub const SAM_POS: sam_fields = 8;
pub const SAM_RNAME: sam_fields = 4;
pub const SAM_FLAG: sam_fields = 2;
pub const SAM_QNAME: sam_fields = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hts_pair_pos_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hts_pair64_max_t {
    pub u: uint64_t,
    pub v: uint64_t,
    pub max: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hts_reglist_t {
    pub reg: *const ::core::ffi::c_char,
    pub intervals: *mut hts_pair_pos_t,
    pub tid: ::core::ffi::c_int,
    pub count: uint32_t,
    pub min_beg: hts_pos_t,
    pub max_end: hts_pos_t,
}
pub type hts_readrec_func = unsafe extern "C" fn(
    *mut BGZF,
    *mut ::core::ffi::c_void,
    *mut ::core::ffi::c_void,
    *mut ::core::ffi::c_int,
    *mut hts_pos_t,
    *mut hts_pos_t,
) -> ::core::ffi::c_int;
pub type hts_seek_func = unsafe extern "C" fn(
    *mut ::core::ffi::c_void,
    int64_t,
    ::core::ffi::c_int,
) -> ::core::ffi::c_int;
pub type hts_tell_func = unsafe extern "C" fn(*mut ::core::ffi::c_void) -> int64_t;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct hts_itr_t {
    #[bitfield(name = "read_rest", ty = "uint32_t", bits = "0..=0")]
    #[bitfield(name = "finished", ty = "uint32_t", bits = "1..=1")]
    #[bitfield(name = "is_cram", ty = "uint32_t", bits = "2..=2")]
    #[bitfield(name = "nocoor", ty = "uint32_t", bits = "3..=3")]
    #[bitfield(name = "multi", ty = "uint32_t", bits = "4..=4")]
    #[bitfield(name = "dummy", ty = "uint32_t", bits = "5..=31")]
    pub read_rest_finished_is_cram_nocoor_multi_dummy: [u8; 4],
    pub tid: ::core::ffi::c_int,
    pub n_off: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
    pub n_reg: ::core::ffi::c_int,
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    pub reg_list: *mut hts_reglist_t,
    pub curr_tid: ::core::ffi::c_int,
    pub curr_reg: ::core::ffi::c_int,
    pub curr_intv: ::core::ffi::c_int,
    pub curr_beg: hts_pos_t,
    pub curr_end: hts_pos_t,
    pub curr_off: uint64_t,
    pub nocoor_off: uint64_t,
    pub off: *mut hts_pair64_max_t,
    pub readrec: Option<hts_readrec_func>,
    pub seek: Option<hts_seek_func>,
    pub tell: Option<hts_tell_func>,
    pub bins: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub n: ::core::ffi::c_int,
    pub m: ::core::ffi::c_int,
    pub a: *mut ::core::ffi::c_int,
}
pub type uint16_u = uint16_t;
pub type uint32_u = uint32_t;
pub type uint64_u = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_14 {
    pub u: uint32_t,
    pub f: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_15 {
    pub u: uint64_t,
    pub f: ::core::ffi::c_double,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_16 {
    pub u: uint32_t,
    pub f: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_17 {
    pub u: uint64_t,
    pub f: ::core::ffi::c_double,
}
pub type cigar_op = ::core::ffi::c_uint;
pub const BAM_CBASE_MISMATCH: cigar_op = 8;
pub const BAM_CBASE_MATCH: cigar_op = 7;
pub const BAM_CPAD_: cigar_op = 6;
pub const BAM_CHARD_CLIP_: cigar_op = 5;
pub const BAM_CSOFT_CLIP_: cigar_op = 4;
pub const BAM_CREF_SKIP_: cigar_op = 3;
pub const BAM_CDEL_: cigar_op = 2;
pub const BAM_CINS_: cigar_op = 1;
pub const BAM_CMATCH_: cigar_op = 0;
pub type sam_sort_order = ::core::ffi::c_int;
pub const ORDER_COORD: sam_sort_order = 2;
pub const ORDER_NAME: sam_sort_order = 1;
pub const ORDER_UNSORTED: sam_sort_order = 0;
pub const ORDER_UNKNOWN: sam_sort_order = -1;
pub type cram_DS_ID = ::core::ffi::c_uint;
pub const DS_END: cram_DS_ID = 47;
pub const DS_TV: cram_DS_ID = 46;
pub const DS_TM: cram_DS_ID = 45;
pub const DS_TC: cram_DS_ID = 44;
pub const DS_QQ_len: cram_DS_ID = 43;
pub const DS_BB_len: cram_DS_ID = 42;
pub const DS_SC_len: cram_DS_ID = 41;
pub const DS_RN_len: cram_DS_ID = 40;
pub const DS_TN: cram_DS_ID = 39;
pub const DS_QQ: cram_DS_ID = 38;
pub const DS_BB: cram_DS_ID = 37;
pub const DS_HC: cram_DS_ID = 36;
pub const DS_PD: cram_DS_ID = 35;
pub const DS_RS: cram_DS_ID = 34;
pub const DS_RI: cram_DS_ID = 33;
pub const DS_TL: cram_DS_ID = 32;
pub const DS_BS: cram_DS_ID = 31;
pub const DS_BA: cram_DS_ID = 30;
pub const DS_DL: cram_DS_ID = 29;
pub const DS_FP: cram_DS_ID = 28;
pub const DS_FC: cram_DS_ID = 27;
pub const DS_FN: cram_DS_ID = 26;
pub const DS_RL: cram_DS_ID = 25;
pub const DS_NF: cram_DS_ID = 24;
pub const DS_NP: cram_DS_ID = 23;
pub const DS_TS: cram_DS_ID = 22;
pub const DS_MF: cram_DS_ID = 21;
pub const DS_NS: cram_DS_ID = 20;
pub const DS_MQ: cram_DS_ID = 19;
pub const DS_RG: cram_DS_ID = 18;
pub const DS_AP: cram_DS_ID = 17;
pub const DS_CF: cram_DS_ID = 16;
pub const DS_BF: cram_DS_ID = 15;
pub const DS_SC: cram_DS_ID = 14;
pub const DS_IN: cram_DS_ID = 13;
pub const DS_QS: cram_DS_ID = 12;
pub const DS_RN: cram_DS_ID = 11;
pub const DS_ref: cram_DS_ID = 10;
pub const DS_aux_oz: cram_DS_ID = 9;
pub const DS_aux_os: cram_DS_ID = 8;
pub const DS_aux_oq: cram_DS_ID = 7;
pub const DS_aux_FZ: cram_DS_ID = 6;
pub const DS_aux_BI: cram_DS_ID = 5;
pub const DS_aux_BD: cram_DS_ID = 4;
pub const DS_aux_BQ: cram_DS_ID = 3;
pub const DS_aux_OQ: cram_DS_ID = 2;
pub const DS_aux: cram_DS_ID = 1;
pub const DS_CORE: cram_DS_ID = 0;
pub type SAM_hdr = sam_hdr_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_s_i2i_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut khint32_t,
    pub vals: *mut ::core::ffi::c_char,
}
pub type kh_s_i2i_t = kh_s_i2i_s;
pub type uc = ::core::ffi::c_uchar;
pub type cram_fields = ::core::ffi::c_uint;
pub const CRAM_ALL: cram_fields = 2147483647;
pub const CRAM_aux: cram_fields = 1073741824;
pub const CRAM_QQ_len: cram_fields = 536870912;
pub const CRAM_QQ: cram_fields = 268435456;
pub const CRAM_BB_len: cram_fields = 134217728;
pub const CRAM_BB: cram_fields = 67108864;
pub const CRAM_SC: cram_fields = 33554432;
pub const CRAM_HC: cram_fields = 16777216;
pub const CRAM_PD: cram_fields = 8388608;
pub const CRAM_RS: cram_fields = 4194304;
pub const CRAM_RI: cram_fields = 2097152;
pub const CRAM_CF: cram_fields = 1048576;
pub const CRAM_MF: cram_fields = 524288;
pub const CRAM_TS: cram_fields = 262144;
pub const CRAM_NP: cram_fields = 131072;
pub const CRAM_NS: cram_fields = 65536;
pub const CRAM_RN: cram_fields = 32768;
pub const CRAM_TL: cram_fields = 16384;
pub const CRAM_MQ: cram_fields = 8192;
pub const CRAM_RG: cram_fields = 4096;
pub const CRAM_IN: cram_fields = 2048;
pub const CRAM_BS: cram_fields = 1024;
pub const CRAM_FN: cram_fields = 512;
pub const CRAM_FC: cram_fields = 256;
pub const CRAM_QS: cram_fields = 128;
pub const CRAM_BA: cram_fields = 64;
pub const CRAM_NF: cram_fields = 32;
pub const CRAM_DL: cram_fields = 16;
pub const CRAM_RL: cram_fields = 8;
pub const CRAM_FP: cram_fields = 4;
pub const CRAM_AP: cram_fields = 2;
pub const CRAM_BF: cram_fields = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cram_decode_job {
    pub fd: *mut cram_fd,
    pub c: *mut cram_container,
    pub s: *mut cram_slice,
    pub h: *mut sam_hdr_t,
    pub exit_code: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const SEEK_CUR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn __bswap_16(mut __bsx: __uint16_t) -> __uint16_t {
    return (__bsx as ::core::ffi::c_int >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as __uint16_t;
}
#[inline]
unsafe extern "C" fn __bswap_32(mut __bsx: __uint32_t) -> __uint32_t {
    return (__bsx & 0xff000000 as __uint32_t) >> 24 as ::core::ffi::c_int
        | (__bsx & 0xff0000 as __uint32_t) >> 8 as ::core::ffi::c_int
        | (__bsx & 0xff00 as __uint32_t) << 8 as ::core::ffi::c_int
        | (__bsx & 0xff as __uint32_t) << 24 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn __bswap_64(mut __bsx: __uint64_t) -> __uint64_t {
    return ((__bsx as ::core::ffi::c_ulonglong & 0xff00000000000000 as ::core::ffi::c_ulonglong)
        >> 56 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000000000 as ::core::ffi::c_ulonglong)
            >> 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000000000 as ::core::ffi::c_ulonglong)
            >> 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00000000 as ::core::ffi::c_ulonglong)
            >> 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000 as ::core::ffi::c_ulonglong)
            << 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000 as ::core::ffi::c_ulonglong)
            << 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00 as ::core::ffi::c_ulonglong)
            << 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff as ::core::ffi::c_ulonglong)
            << 56 as ::core::ffi::c_int) as __uint64_t;
}
#[inline]
unsafe extern "C" fn __uint16_identity(mut __x: __uint16_t) -> __uint16_t {
    return __x;
}
#[inline]
unsafe extern "C" fn __uint32_identity(mut __x: __uint32_t) -> __uint32_t {
    return __x;
}
#[inline]
unsafe extern "C" fn __uint64_identity(mut __x: __uint64_t) -> __uint64_t {
    return __x;
}
pub const INT64_MIN: ::core::ffi::c_long =
    -(9223372036854775807 as ::core::ffi::c_long) - 1 as ::core::ffi::c_long;
pub const INT32_MAX: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const UINT8_MAX: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const UINT16_MAX: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
#[inline]
unsafe extern "C" fn hts_prefetch(mut p: *mut ::core::ffi::c_void) {
    *(p as *mut ::core::ffi::c_char);
}
#[inline]
unsafe extern "C" fn ks_initialize(mut s: *mut kstring_t) {
    (*s).m = 0 as size_t;
    (*s).l = (*s).m;
    (*s).s = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[inline]
unsafe extern "C" fn ks_resize(mut s: *mut kstring_t, mut size: size_t) -> ::core::ffi::c_int {
    if (*s).m < size {
        let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        size = if size > SIZE_MAX as size_t >> 2 as ::core::ffi::c_int {
            size
        } else {
            size.wrapping_add(size >> 1 as ::core::ffi::c_int)
        };
        tmp = realloc((*s).s as *mut ::core::ffi::c_void, size) as *mut ::core::ffi::c_char;
        if tmp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*s).s = tmp;
        (*s).m = size;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn ks_expand(mut s: *mut kstring_t, mut expansion: size_t) -> ::core::ffi::c_int {
    let mut new_size: size_t = (*s).l.wrapping_add(expansion);
    if new_size < (*s).l {
        *__errno_location() = EOVERFLOW;
        return -(1 as ::core::ffi::c_int);
    }
    return ks_resize(s, new_size);
}
#[inline]
unsafe extern "C" fn ks_str(mut s: *mut kstring_t) -> *mut ::core::ffi::c_char {
    return (*s).s;
}
#[inline]
unsafe extern "C" fn ks_c_str(mut s: *mut kstring_t) -> *const ::core::ffi::c_char {
    return if (*s).l != 0 && !(*s).s.is_null() {
        (*s).s as *const ::core::ffi::c_char
    } else {
        b"\0" as *const u8 as *const ::core::ffi::c_char
    };
}
#[inline]
unsafe extern "C" fn ks_len(mut s: *mut kstring_t) -> size_t {
    return (*s).l;
}
#[inline]
unsafe extern "C" fn ks_clear(mut s: *mut kstring_t) -> *mut kstring_t {
    (*s).l = 0 as size_t;
    return s;
}
#[inline]
unsafe extern "C" fn ks_release(mut s: *mut kstring_t) -> *mut ::core::ffi::c_char {
    let mut ss: *mut ::core::ffi::c_char = (*s).s;
    (*s).m = 0 as size_t;
    (*s).l = (*s).m;
    (*s).s = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return ss;
}
#[inline]
unsafe extern "C" fn ks_free(mut s: *mut kstring_t) {
    if !s.is_null() {
        free((*s).s as *mut ::core::ffi::c_void);
        ks_initialize(s);
    }
}
#[inline]
unsafe extern "C" fn kputsn(
    mut p: *const ::core::ffi::c_char,
    mut l: size_t,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    let mut new_sz: size_t = (*s).l.wrapping_add(l).wrapping_add(2 as size_t);
    if new_sz <= (*s).l {
        *__errno_location() = EOVERFLOW;
        return EOF;
    }
    if ks_resize(s, new_sz) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    memcpy(
        (*s).s.offset((*s).l as isize) as *mut ::core::ffi::c_void,
        p as *const ::core::ffi::c_void,
        l,
    );
    (*s).l = ((*s).l as ::core::ffi::c_ulong).wrapping_add(l as ::core::ffi::c_ulong) as size_t
        as size_t;
    *(*s).s.offset((*s).l as isize) = 0 as ::core::ffi::c_char;
    return l as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kputs(
    mut p: *const ::core::ffi::c_char,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    if p.is_null() {
        *__errno_location() = EFAULT;
        return -(1 as ::core::ffi::c_int);
    }
    return kputsn(p, strlen(p), s);
}
#[inline]
unsafe extern "C" fn kputc(mut c: ::core::ffi::c_int, mut s: *mut kstring_t) -> ::core::ffi::c_int {
    if ks_resize(s, (*s).l.wrapping_add(2 as size_t)) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    let fresh0 = (*s).l;
    (*s).l = (*s).l.wrapping_add(1);
    *(*s).s.offset(fresh0 as isize) = c as ::core::ffi::c_char;
    *(*s).s.offset((*s).l as isize) = 0 as ::core::ffi::c_char;
    return c as ::core::ffi::c_uchar as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kputc_(
    mut c: ::core::ffi::c_int,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    if ks_resize(s, (*s).l.wrapping_add(1 as size_t)) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    let fresh1 = (*s).l;
    (*s).l = (*s).l.wrapping_add(1);
    *(*s).s.offset(fresh1 as isize) = c as ::core::ffi::c_char;
    return 1 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kputsn_(
    mut p: *const ::core::ffi::c_void,
    mut l: size_t,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    let mut new_sz: size_t = (*s).l.wrapping_add(l);
    if new_sz < (*s).l {
        *__errno_location() = EOVERFLOW;
        return EOF;
    }
    if ks_resize(s, (if new_sz != 0 { new_sz } else { 1 as size_t })) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    memcpy(
        (*s).s.offset((*s).l as isize) as *mut ::core::ffi::c_void,
        p,
        l,
    );
    (*s).l = ((*s).l as ::core::ffi::c_ulong).wrapping_add(l as ::core::ffi::c_ulong) as size_t
        as size_t;
    return l as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kputuw(
    mut x: ::core::ffi::c_uint,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    static mut kputuw_num_digits: [::core::ffi::c_uint; 32] = [
        10 as ::core::ffi::c_int as ::core::ffi::c_uint,
        10 as ::core::ffi::c_int as ::core::ffi::c_uint,
        10 as ::core::ffi::c_int as ::core::ffi::c_uint,
        9 as ::core::ffi::c_int as ::core::ffi::c_uint,
        9 as ::core::ffi::c_int as ::core::ffi::c_uint,
        9 as ::core::ffi::c_int as ::core::ffi::c_uint,
        8 as ::core::ffi::c_int as ::core::ffi::c_uint,
        8 as ::core::ffi::c_int as ::core::ffi::c_uint,
        8 as ::core::ffi::c_int as ::core::ffi::c_uint,
        7 as ::core::ffi::c_int as ::core::ffi::c_uint,
        7 as ::core::ffi::c_int as ::core::ffi::c_uint,
        7 as ::core::ffi::c_int as ::core::ffi::c_uint,
        7 as ::core::ffi::c_int as ::core::ffi::c_uint,
        6 as ::core::ffi::c_int as ::core::ffi::c_uint,
        6 as ::core::ffi::c_int as ::core::ffi::c_uint,
        6 as ::core::ffi::c_int as ::core::ffi::c_uint,
        5 as ::core::ffi::c_int as ::core::ffi::c_uint,
        5 as ::core::ffi::c_int as ::core::ffi::c_uint,
        5 as ::core::ffi::c_int as ::core::ffi::c_uint,
        4 as ::core::ffi::c_int as ::core::ffi::c_uint,
        4 as ::core::ffi::c_int as ::core::ffi::c_uint,
        4 as ::core::ffi::c_int as ::core::ffi::c_uint,
        4 as ::core::ffi::c_int as ::core::ffi::c_uint,
        3 as ::core::ffi::c_int as ::core::ffi::c_uint,
        3 as ::core::ffi::c_int as ::core::ffi::c_uint,
        3 as ::core::ffi::c_int as ::core::ffi::c_uint,
        2 as ::core::ffi::c_int as ::core::ffi::c_uint,
        2 as ::core::ffi::c_int as ::core::ffi::c_uint,
        2 as ::core::ffi::c_int as ::core::ffi::c_uint,
        1 as ::core::ffi::c_int as ::core::ffi::c_uint,
        1 as ::core::ffi::c_int as ::core::ffi::c_uint,
        1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    ];
    static mut kputuw_thresholds: [::core::ffi::c_uint; 32] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        1000000000 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        100000000 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        10000000 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        1000000 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        100000 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        10000 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        1000 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        100 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        10 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
        0 as ::core::ffi::c_int as ::core::ffi::c_uint,
    ];
    static mut kputuw_dig2r: [::core::ffi::c_char; 201] = unsafe {
        ::core::mem::transmute::<
            [u8; 201],
            [::core::ffi::c_char; 201],
        >(
            *b"00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899\0",
        )
    };
    let mut l: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if x < 10 as ::core::ffi::c_uint {
        if ks_resize(s, (*s).l.wrapping_add(2 as size_t)) < 0 as ::core::ffi::c_int {
            return EOF;
        }
        let fresh2 = (*s).l;
        (*s).l = (*s).l.wrapping_add(1);
        *(*s).s.offset(fresh2 as isize) =
            ('0' as i32 as ::core::ffi::c_uint).wrapping_add(x) as ::core::ffi::c_char;
        *(*s).s.offset((*s).l as isize) = 0 as ::core::ffi::c_char;
        return 0 as ::core::ffi::c_int;
    }
    l = x.leading_zeros() as i32 as ::core::ffi::c_uint;
    l = kputuw_num_digits[l as usize].wrapping_sub(
        (x < kputuw_thresholds[l as usize]) as ::core::ffi::c_int as ::core::ffi::c_uint,
    );
    if ks_resize(
        s,
        (*s).l.wrapping_add(l as size_t).wrapping_add(2 as size_t),
    ) < 0 as ::core::ffi::c_int
    {
        return EOF;
    }
    j = l;
    cp = (*s).s.offset((*s).l as isize);
    while x >= 10 as ::core::ffi::c_uint {
        let mut d: *const ::core::ffi::c_char =
            (&raw const kputuw_dig2r as *const ::core::ffi::c_char).offset(
                (2 as ::core::ffi::c_uint).wrapping_mul(x.wrapping_rem(100 as ::core::ffi::c_uint))
                    as isize,
            ) as *const ::core::ffi::c_char;
        x = x.wrapping_div(100 as ::core::ffi::c_uint);
        j = j.wrapping_sub(2 as ::core::ffi::c_uint);
        memcpy(
            cp.offset(j as isize) as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            d as *const ::core::ffi::c_void,
            2 as size_t,
        );
    }
    if j == 1 as ::core::ffi::c_uint {
        *cp.offset(0 as ::core::ffi::c_int as isize) =
            x.wrapping_add('0' as i32 as ::core::ffi::c_uint) as ::core::ffi::c_char;
    }
    (*s).l = ((*s).l as ::core::ffi::c_ulong).wrapping_add(l as ::core::ffi::c_ulong) as size_t
        as size_t;
    *(*s).s.offset((*s).l as isize) = 0 as ::core::ffi::c_char;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kputw(mut c: ::core::ffi::c_int, mut s: *mut kstring_t) -> ::core::ffi::c_int {
    let mut x: ::core::ffi::c_uint = c as ::core::ffi::c_uint;
    if c < 0 as ::core::ffi::c_int {
        x = x.wrapping_neg();
        if ks_resize(s, (*s).l.wrapping_add(3 as size_t)) < 0 as ::core::ffi::c_int {
            return EOF;
        }
        let fresh3 = (*s).l;
        (*s).l = (*s).l.wrapping_add(1);
        *(*s).s.offset(fresh3 as isize) = '-' as i32 as ::core::ffi::c_char;
    }
    return kputuw(x, s);
}
#[inline]
unsafe extern "C" fn kputll(
    mut c: ::core::ffi::c_longlong,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    if ks_resize(s, (*s).l.wrapping_add(23 as size_t)) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    let mut x: ::core::ffi::c_ulonglong = c as ::core::ffi::c_ulonglong;
    if c < 0 as ::core::ffi::c_longlong {
        x = x.wrapping_neg();
        let fresh4 = (*s).l;
        (*s).l = (*s).l.wrapping_add(1);
        *(*s).s.offset(fresh4 as isize) = '-' as i32 as ::core::ffi::c_char;
    }
    if x <= UINT32_MAX as ::core::ffi::c_ulonglong {
        return kputuw(x as ::core::ffi::c_uint, s);
    }
    static mut kputull_dig2r: [::core::ffi::c_char; 201] = unsafe {
        ::core::mem::transmute::<
            [u8; 201],
            [::core::ffi::c_char; 201],
        >(
            *b"00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899\0",
        )
    };
    let mut l: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut m: uint64_t = 1 as uint64_t;
    l = 0 as ::core::ffi::c_uint;
    if ::core::mem::size_of::<::core::ffi::c_longlong>() as usize
        == ::core::mem::size_of::<uint64_t>() as usize
        && x >= 10000000000000000000 as ::core::ffi::c_ulonglong
    {
        l = 20 as ::core::ffi::c_uint;
    } else {
        loop {
            l = l.wrapping_add(1);
            m = (m as ::core::ffi::c_ulong).wrapping_mul(10 as ::core::ffi::c_ulong) as uint64_t
                as uint64_t;
            if !(x >= m as ::core::ffi::c_ulonglong) {
                break;
            }
        }
    }
    j = l;
    cp = (*s).s.offset((*s).l as isize);
    while x >= 10 as ::core::ffi::c_ulonglong {
        let mut d: *const ::core::ffi::c_char =
            (&raw const kputull_dig2r as *const ::core::ffi::c_char).offset(
                (2 as ::core::ffi::c_ulonglong)
                    .wrapping_mul(x.wrapping_rem(100 as ::core::ffi::c_ulonglong))
                    as isize,
            ) as *const ::core::ffi::c_char;
        x = x.wrapping_div(100 as ::core::ffi::c_ulonglong);
        j = j.wrapping_sub(2 as ::core::ffi::c_uint);
        memcpy(
            cp.offset(j as isize) as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            d as *const ::core::ffi::c_void,
            2 as size_t,
        );
    }
    if j == 1 as ::core::ffi::c_uint {
        *cp.offset(0 as ::core::ffi::c_int as isize) =
            x.wrapping_add('0' as i32 as ::core::ffi::c_ulonglong) as ::core::ffi::c_char;
    }
    (*s).l = ((*s).l as ::core::ffi::c_ulong).wrapping_add(l as ::core::ffi::c_ulong) as size_t
        as size_t;
    *(*s).s.offset((*s).l as isize) = 0 as ::core::ffi::c_char;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kputl(
    mut c: ::core::ffi::c_long,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    return kputll(c as ::core::ffi::c_longlong, s);
}
#[inline]
unsafe extern "C" fn ksplit(
    mut s: *mut kstring_t,
    mut delimiter: ::core::ffi::c_int,
    mut n: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_int {
    let mut max: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut offsets: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    *n = ksplit_core((*s).s, delimiter, &raw mut max, &raw mut offsets);
    return offsets;
}
#[inline]
unsafe extern "C" fn kinsert_char(
    mut c: ::core::ffi::c_char,
    mut pos: size_t,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    if s.is_null() || pos > (*s).l {
        return EOF;
    }
    if ks_resize(s, (*s).l.wrapping_add(2 as size_t)) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    memmove(
        (*s).s
            .offset(pos as isize)
            .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        (*s).s.offset(pos as isize) as *const ::core::ffi::c_void,
        (*s).l.wrapping_sub(pos),
    );
    *(*s).s.offset(pos as isize) = c;
    (*s).l = (*s).l.wrapping_add(1);
    *(*s).s.offset((*s).l as isize) = 0 as ::core::ffi::c_char;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kinsert_str(
    mut str: *const ::core::ffi::c_char,
    mut pos: size_t,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    let mut len: size_t = 0 as size_t;
    if s.is_null() || pos > (*s).l || str.is_null() {
        return EOF;
    }
    len = strlen(str);
    if len == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if ks_resize(s, (*s).l.wrapping_add(len).wrapping_add(1 as size_t)) < 0 as ::core::ffi::c_int {
        return EOF;
    }
    memmove(
        (*s).s.offset(pos as isize).offset(len as isize) as *mut ::core::ffi::c_void,
        (*s).s.offset(pos as isize) as *const ::core::ffi::c_void,
        (*s).l.wrapping_sub(pos),
    );
    memcpy(
        (*s).s.offset(pos as isize) as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    (*s).l = ((*s).l as ::core::ffi::c_ulong).wrapping_add(len as ::core::ffi::c_ulong) as size_t
        as size_t;
    *(*s).s.offset((*s).l as isize) = '\0' as i32 as ::core::ffi::c_char;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn hts_reg2bin(
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    mut min_shift: ::core::ffi::c_int,
    mut n_lvls: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut l: ::core::ffi::c_int = 0;
    let mut s: ::core::ffi::c_int = min_shift;
    let mut t: ::core::ffi::c_int = (((1 as ::core::ffi::c_int)
        << (n_lvls << 1 as ::core::ffi::c_int) + n_lvls)
        - 1 as ::core::ffi::c_int)
        / 7 as ::core::ffi::c_int;
    end -= 1;
    l = n_lvls;
    while l > 0 as ::core::ffi::c_int {
        if beg >> s == end >> s {
            return (t as hts_pos_t + (beg >> s)) as ::core::ffi::c_int;
        }
        l -= 1;
        s += 3 as ::core::ffi::c_int;
        t -= (1 as ::core::ffi::c_int) << (l << 1 as ::core::ffi::c_int) + l;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn hts_bin_level(mut bin: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut l: ::core::ffi::c_int = 0;
    let mut b: ::core::ffi::c_int = 0;
    l = 0 as ::core::ffi::c_int;
    b = bin;
    while b != 0 {
        l += 1;
        b = b - 1 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int;
    }
    return l;
}
#[inline]
unsafe extern "C" fn hts_bin_bot(
    mut bin: ::core::ffi::c_int,
    mut n_lvls: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut l: ::core::ffi::c_int = hts_bin_level(bin);
    return (bin
        - (((1 as ::core::ffi::c_int) << (l << 1 as ::core::ffi::c_int) + l)
            - 1 as ::core::ffi::c_int)
            / 7 as ::core::ffi::c_int)
        << (n_lvls - l) * 3 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn hts_bin_maxpos(
    mut min_shift: ::core::ffi::c_int,
    mut n_lvls: ::core::ffi::c_int,
) -> hts_pos_t {
    let mut one: hts_pos_t = 1 as hts_pos_t;
    return one << min_shift + n_lvls * 3 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn ed_is_big() -> ::core::ffi::c_int {
    let mut one: ::core::ffi::c_long = 1 as ::core::ffi::c_long;
    return (*(&raw mut one as *mut ::core::ffi::c_char) == 0) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn ed_swap_2(mut v: uint16_t) -> uint16_t {
    return ((v as ::core::ffi::c_uint & 0xff00ff as ::core::ffi::c_uint) << 8 as ::core::ffi::c_int
        | (v as ::core::ffi::c_uint & 0xff00ff00 as ::core::ffi::c_uint) >> 8 as ::core::ffi::c_int)
        as uint16_t;
}
#[inline]
unsafe extern "C" fn ed_swap_2p(mut x: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    *(x as *mut uint16_t) = ed_swap_2(*(x as *mut uint16_t));
    return x;
}
#[inline]
unsafe extern "C" fn ed_swap_4(mut v: uint32_t) -> uint32_t {
    v = (v & 0xffff as uint32_t) << 16 as ::core::ffi::c_int | v >> 16 as ::core::ffi::c_int;
    return (v & 0xff00ff as uint32_t) << 8 as ::core::ffi::c_int
        | (v & 0xff00ff00 as uint32_t) >> 8 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn ed_swap_4p(mut x: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    *(x as *mut uint32_t) = ed_swap_4(*(x as *mut uint32_t));
    return x;
}
#[inline]
unsafe extern "C" fn ed_swap_8(mut v: uint64_t) -> uint64_t {
    v = ((v as ::core::ffi::c_ulonglong & 0xffffffff as ::core::ffi::c_ulonglong)
        << 32 as ::core::ffi::c_int
        | (v >> 32 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong) as uint64_t;
    v = ((v as ::core::ffi::c_ulonglong & 0xffff0000ffff as ::core::ffi::c_ulonglong)
        << 16 as ::core::ffi::c_int
        | (v as ::core::ffi::c_ulonglong & 0xffff0000ffff0000 as ::core::ffi::c_ulonglong)
            >> 16 as ::core::ffi::c_int) as uint64_t;
    return ((v as ::core::ffi::c_ulonglong & 0xff00ff00ff00ff as ::core::ffi::c_ulonglong)
        << 8 as ::core::ffi::c_int
        | (v as ::core::ffi::c_ulonglong & 0xff00ff00ff00ff00 as ::core::ffi::c_ulonglong)
            >> 8 as ::core::ffi::c_int) as uint64_t;
}
#[inline]
unsafe extern "C" fn ed_swap_8p(mut x: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    *(x as *mut uint64_t) = ed_swap_8(*(x as *mut uint64_t));
    return x;
}
#[inline]
unsafe extern "C" fn kh_init_m_i2i() -> *mut kh_m_i2i_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_m_i2i_t>() as size_t) as *mut kh_m_i2i_t;
}
#[inline]
unsafe extern "C" fn kh_destroy_m_i2i(mut h: *mut kh_m_i2i_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_clear_m_i2i(mut h: *mut kh_m_i2i_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_m_i2i(mut h: *const kh_m_i2i_t, mut key: khint64_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = (key >> 33 as ::core::ffi::c_int ^ key ^ key << 11 as ::core::ffi::c_int) as khint32_t
            as khint_t;
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_m_i2i(
    mut h: *mut kh_m_i2i_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut khint64_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint64_t>() as size_t),
            ) as *mut khint64_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut ::core::ffi::c_int = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>() as size_t),
            ) as *mut ::core::ffi::c_int;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: khint64_t = *(*h).keys.offset(j as isize);
                let mut val: ::core::ffi::c_int = 0;
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh51 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh51 = (*fresh51 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = (key >> 33 as ::core::ffi::c_int ^ key ^ key << 11 as ::core::ffi::c_int)
                        as khint32_t as khint_t;
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh52 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh52 = (*fresh52 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: khint64_t = *(*h).keys.offset(i as isize);
                        *(*h).keys.offset(i as isize) = key;
                        key = tmp;
                        let mut tmp_0: ::core::ffi::c_int = *(*h).vals.offset(i as isize);
                        *(*h).vals.offset(i as isize) = val;
                        val = tmp_0;
                        let ref mut fresh53 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh53 = (*fresh53 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        *(*h).vals.offset(i as isize) = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint64_t>() as size_t),
            ) as *mut khint64_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>() as size_t),
            ) as *mut ::core::ffi::c_int;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_m_i2i(
    mut h: *mut kh_m_i2i_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_m_i2i(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_m_i2i(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_put_m_i2i(
    mut h: *mut kh_m_i2i_t,
    mut key: khint64_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_m_i2i(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_m_i2i(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = (key >> 33 as ::core::ffi::c_int ^ key ^ key << 11 as ::core::ffi::c_int) as khint32_t
        as khint_t;
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh54 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh54 = (*fresh54 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh55 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh55 = (*fresh55 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_del_m_i2i(mut h: *mut kh_m_i2i_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh56 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh56 = (*fresh56 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_stats_m_i2i(
    mut h: *mut kh_m_i2i_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = ((*(*h).keys.offset(i as isize) >> 33 as ::core::ffi::c_int
                ^ *(*h).keys.offset(i as isize)
                ^ *(*h).keys.offset(i as isize) << 11 as ::core::ffi::c_int)
                as ::core::ffi::c_uint
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh57 = *hist.offset(dist as isize);
            *fresh57 = (*fresh57).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_stats_s_i2i(
    mut h: *mut kh_s_i2i_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (*(*h).keys.offset(i as isize)
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh64 = *hist.offset(dist as isize);
            *fresh64 = (*fresh64).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_init_s_i2i() -> *mut kh_s_i2i_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_s_i2i_t>() as size_t) as *mut kh_s_i2i_t;
}
#[inline]
unsafe extern "C" fn kh_destroy_s_i2i(mut h: *mut kh_s_i2i_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_clear_s_i2i(mut h: *mut kh_s_i2i_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_s_i2i(mut h: *const kh_s_i2i_t, mut key: khint32_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = key as khint_t;
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_s_i2i(
    mut h: *mut kh_s_i2i_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut khint32_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: khint32_t = *(*h).keys.offset(j as isize);
                let mut val: ::core::ffi::c_char = 0;
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                let ref mut fresh58 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh58 = (*fresh58 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = key as khint_t;
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh59 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh59 = (*fresh59 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: khint32_t = *(*h).keys.offset(i as isize);
                        *(*h).keys.offset(i as isize) = key;
                        key = tmp;
                        let ref mut fresh60 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh60 = (*fresh60 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_s_i2i(
    mut h: *mut kh_s_i2i_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_s_i2i(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_s_i2i(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_put_s_i2i(
    mut h: *mut kh_s_i2i_t,
    mut key: khint32_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_s_i2i(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_s_i2i(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = key as khint_t;
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh61 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh61 = (*fresh61 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh62 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh62 = (*fresh62 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_del_s_i2i(mut h: *mut kh_s_i2i_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh63 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh63 = (*fresh63 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_resize_map(
    mut h: *mut kh_map_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut kh_cstr_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<kh_cstr_t>() as size_t),
            ) as *mut kh_cstr_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut pmap_t = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t).wrapping_mul(::core::mem::size_of::<pmap_t>() as size_t),
            ) as *mut pmap_t;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: kh_cstr_t = *(*h).keys.offset(j as isize);
                let mut val: pmap_t = pmap_t { i: 0 };
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh43 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh43 = (*fresh43 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh44 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh44 = (*fresh44 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: kh_cstr_t = *(*h).keys.offset(i as isize);
                        let ref mut fresh45 = *(*h).keys.offset(i as isize);
                        *fresh45 = key;
                        key = tmp;
                        let mut tmp_0: pmap_t = *(*h).vals.offset(i as isize);
                        *(*h).vals.offset(i as isize) = val;
                        val = tmp_0;
                        let ref mut fresh46 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh46 = (*fresh46 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        let ref mut fresh47 = *(*h).keys.offset(i as isize);
                        *fresh47 = key;
                        *(*h).vals.offset(i as isize) = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<kh_cstr_t>() as size_t),
            ) as *mut kh_cstr_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t).wrapping_mul(::core::mem::size_of::<pmap_t>() as size_t),
            ) as *mut pmap_t;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_put_map(
    mut h: *mut kh_map_t,
    mut key: kh_cstr_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_map(h, (*h).n_buckets.wrapping_sub(1 as khint_t)) < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_map(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(strcmp(
                    *(*h).keys.offset(i as isize) as *const ::core::ffi::c_char,
                    key as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        let ref mut fresh39 = *(*h).keys.offset(x as isize);
        *fresh39 = key;
        let ref mut fresh40 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh40 = (*fresh40 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        let ref mut fresh41 = *(*h).keys.offset(x as isize);
        *fresh41 = key;
        let ref mut fresh42 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh42 = (*fresh42 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_init_map() -> *mut kh_map_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_map_t>() as size_t) as *mut kh_map_t;
}
#[inline]
unsafe extern "C" fn kh_stats_map(
    mut h: *mut kh_map_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (__ac_FNV1a_hash_string(*(*h).keys.offset(i as isize) as *const ::core::ffi::c_char)
                as ::core::ffi::c_uint
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh66 = *hist.offset(dist as isize);
            *fresh66 = (*fresh66).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_destroy_map(mut h: *mut kh_map_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_clear_map(mut h: *mut kh_map_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_map(mut h: *const kh_map_t, mut key: kh_cstr_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(strcmp(
                    *(*h).keys.offset(i as isize) as *const ::core::ffi::c_char,
                    key as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_map(
    mut h: *mut kh_map_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_map(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_map(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_del_map(mut h: *mut kh_map_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh65 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh65 = (*fresh65 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_destroy_m_metrics(mut h: *mut kh_m_metrics_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_init_m_metrics() -> *mut kh_m_metrics_t {
    return calloc(
        1 as size_t,
        ::core::mem::size_of::<kh_m_metrics_t>() as size_t,
    ) as *mut kh_m_metrics_t;
}
#[inline]
unsafe extern "C" fn kh_put_m_metrics(
    mut h: *mut kh_m_metrics_t,
    mut key: khint32_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_m_metrics(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_m_metrics(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = key as khint_t;
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh72 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh72 = (*fresh72 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh73 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh73 = (*fresh73 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_clear_m_metrics(mut h: *mut kh_m_metrics_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_m_metrics(mut h: *const kh_m_metrics_t, mut key: khint32_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = key as khint_t;
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_m_metrics(
    mut h: *mut kh_m_metrics_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut khint32_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut *mut cram_metrics = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut cram_metrics>() as size_t),
            ) as *mut *mut cram_metrics;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: khint32_t = *(*h).keys.offset(j as isize);
                let mut val: *mut cram_metrics = ::core::ptr::null_mut::<cram_metrics>();
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh67 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh67 = (*fresh67 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = key as khint_t;
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh68 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh68 = (*fresh68 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: khint32_t = *(*h).keys.offset(i as isize);
                        *(*h).keys.offset(i as isize) = key;
                        key = tmp;
                        let mut tmp_0: *mut cram_metrics = *(*h).vals.offset(i as isize);
                        let ref mut fresh69 = *(*h).vals.offset(i as isize);
                        *fresh69 = val;
                        val = tmp_0;
                        let ref mut fresh70 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh70 = (*fresh70 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        let ref mut fresh71 = *(*h).vals.offset(i as isize);
                        *fresh71 = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut cram_metrics>() as size_t),
            ) as *mut *mut cram_metrics;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_m_metrics(
    mut h: *mut kh_m_metrics_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_m_metrics(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_m_metrics(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_del_m_metrics(mut h: *mut kh_m_metrics_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh74 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh74 = (*fresh74 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_stats_m_metrics(
    mut h: *mut kh_m_metrics_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (*(*h).keys.offset(i as isize)
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh75 = *hist.offset(dist as isize);
            *fresh75 = (*fresh75).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
pub const CRAM_MAP_HASH: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn kh_init_m_tagmap() -> *mut kh_m_tagmap_t {
    return calloc(
        1 as size_t,
        ::core::mem::size_of::<kh_m_tagmap_t>() as size_t,
    ) as *mut kh_m_tagmap_t;
}
#[inline]
unsafe extern "C" fn kh_destroy_m_tagmap(mut h: *mut kh_m_tagmap_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_clear_m_tagmap(mut h: *mut kh_m_tagmap_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_m_tagmap(mut h: *const kh_m_tagmap_t, mut key: khint32_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = key as khint_t;
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_m_tagmap(
    mut h: *mut kh_m_tagmap_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut khint32_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut *mut cram_tag_map = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut cram_tag_map>() as size_t),
            ) as *mut *mut cram_tag_map;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: khint32_t = *(*h).keys.offset(j as isize);
                let mut val: *mut cram_tag_map = ::core::ptr::null_mut::<cram_tag_map>();
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh76 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh76 = (*fresh76 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = key as khint_t;
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh77 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh77 = (*fresh77 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: khint32_t = *(*h).keys.offset(i as isize);
                        *(*h).keys.offset(i as isize) = key;
                        key = tmp;
                        let mut tmp_0: *mut cram_tag_map = *(*h).vals.offset(i as isize);
                        let ref mut fresh78 = *(*h).vals.offset(i as isize);
                        *fresh78 = val;
                        val = tmp_0;
                        let ref mut fresh79 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh79 = (*fresh79 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        let ref mut fresh80 = *(*h).vals.offset(i as isize);
                        *fresh80 = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut cram_tag_map>() as size_t),
            ) as *mut *mut cram_tag_map;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_m_tagmap(
    mut h: *mut kh_m_tagmap_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_m_tagmap(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_m_tagmap(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_put_m_tagmap(
    mut h: *mut kh_m_tagmap_t,
    mut key: khint32_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_m_tagmap(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_m_tagmap(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = key as khint_t;
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh81 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh81 = (*fresh81 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh82 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh82 = (*fresh82 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_del_m_tagmap(mut h: *mut kh_m_tagmap_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh83 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh83 = (*fresh83 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_stats_m_tagmap(
    mut h: *mut kh_m_tagmap_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (*(*h).keys.offset(i as isize)
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh84 = *hist.offset(dist as isize);
            *fresh84 = (*fresh84).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_init_refs() -> *mut kh_refs_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_refs_t>() as size_t) as *mut kh_refs_t;
}
#[inline]
unsafe extern "C" fn kh_destroy_refs(mut h: *mut kh_refs_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_clear_refs(mut h: *mut kh_refs_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_refs(mut h: *const kh_refs_t, mut key: kh_cstr_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(strcmp(
                    *(*h).keys.offset(i as isize) as *const ::core::ffi::c_char,
                    key as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_refs(
    mut h: *mut kh_refs_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut kh_cstr_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<kh_cstr_t>() as size_t),
            ) as *mut kh_cstr_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut *mut ref_entry = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut ref_entry>() as size_t),
            ) as *mut *mut ref_entry;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: kh_cstr_t = *(*h).keys.offset(j as isize);
                let mut val: *mut ref_entry = ::core::ptr::null_mut::<ref_entry>();
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh85 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh85 = (*fresh85 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh86 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh86 = (*fresh86 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: kh_cstr_t = *(*h).keys.offset(i as isize);
                        let ref mut fresh87 = *(*h).keys.offset(i as isize);
                        *fresh87 = key;
                        key = tmp;
                        let mut tmp_0: *mut ref_entry = *(*h).vals.offset(i as isize);
                        let ref mut fresh88 = *(*h).vals.offset(i as isize);
                        *fresh88 = val;
                        val = tmp_0;
                        let ref mut fresh89 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh89 = (*fresh89 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        let ref mut fresh90 = *(*h).keys.offset(i as isize);
                        *fresh90 = key;
                        let ref mut fresh91 = *(*h).vals.offset(i as isize);
                        *fresh91 = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<kh_cstr_t>() as size_t),
            ) as *mut kh_cstr_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut ref_entry>() as size_t),
            ) as *mut *mut ref_entry;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_refs(
    mut h: *mut kh_refs_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_refs(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_refs(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_put_refs(
    mut h: *mut kh_refs_t,
    mut key: kh_cstr_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_refs(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_refs(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(strcmp(
                    *(*h).keys.offset(i as isize) as *const ::core::ffi::c_char,
                    key as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        let ref mut fresh92 = *(*h).keys.offset(x as isize);
        *fresh92 = key;
        let ref mut fresh93 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh93 = (*fresh93 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        let ref mut fresh94 = *(*h).keys.offset(x as isize);
        *fresh94 = key;
        let ref mut fresh95 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh95 = (*fresh95 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_del_refs(mut h: *mut kh_refs_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh96 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh96 = (*fresh96 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_stats_refs(
    mut h: *mut kh_refs_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (__ac_FNV1a_hash_string(*(*h).keys.offset(i as isize) as *const ::core::ffi::c_char)
                as ::core::ffi::c_uint
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh97 = *hist.offset(dist as isize);
            *fresh97 = (*fresh97).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
pub const CRAM_M_REVERSE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CRAM_M_UNMAP: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CRAM_FLAG_PRESERVE_QUAL_SCORES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const CRAM_FLAG_DETACHED: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const CRAM_FLAG_MATE_DOWNSTREAM: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const CRAM_FLAG_NO_SEQ: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const CRAM_FLAG_EXPLICIT_TLEN: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn herrno(mut fp: *mut hFILE) -> ::core::ffi::c_int {
    return (*fp).has_errno;
}
#[inline]
unsafe extern "C" fn hclearerr(mut fp: *mut hFILE) {
    (*fp).has_errno = 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn htell(mut fp: *mut hFILE) -> off_t {
    return (*fp).offset + (*fp).begin.offset_from((*fp).buffer) as off_t;
}
#[inline]
unsafe extern "C" fn hgetc(mut fp: *mut hFILE) -> ::core::ffi::c_int {
    extern "C" {
        #[link_name = "hgetc2"]
        fn hgetc2_0(_: *mut hFILE) -> ::core::ffi::c_int;
    }
    return if (*fp).end > (*fp).begin {
        let fresh151 = (*fp).begin;
        (*fp).begin = (*fp).begin.offset(1);
        *fresh151 as ::core::ffi::c_uchar as ::core::ffi::c_int
    } else {
        hgetc2_0(fp)
    };
}
#[inline]
unsafe extern "C" fn hgetln(
    mut buffer: *mut ::core::ffi::c_char,
    mut size: size_t,
    mut fp: *mut hFILE,
) -> ssize_t {
    return hgetdelim(buffer, size, '\n' as i32, fp);
}
#[inline]
unsafe extern "C" fn hread(
    mut fp: *mut hFILE,
    mut buffer: *mut ::core::ffi::c_void,
    mut nbytes: size_t,
) -> ssize_t {
    extern "C" {
        #[link_name = "hread2"]
        fn hread2_0(_: *mut hFILE, _: *mut ::core::ffi::c_void, _: size_t, _: size_t) -> ssize_t;
    }
    let mut n: size_t = (*fp).end.offset_from((*fp).begin) as ::core::ffi::c_long as size_t;
    if n > nbytes {
        n = nbytes;
    }
    memcpy(buffer, (*fp).begin as *const ::core::ffi::c_void, n);
    (*fp).begin = (*fp).begin.offset(n as isize);
    return if n == nbytes || (*fp).mobile() == 0 {
        n as ssize_t
    } else {
        hread2_0(fp, buffer, nbytes, n)
    };
}
#[inline]
unsafe extern "C" fn hputc(mut c: ::core::ffi::c_int, mut fp: *mut hFILE) -> ::core::ffi::c_int {
    extern "C" {
        #[link_name = "hputc2"]
        fn hputc2_0(_: ::core::ffi::c_int, _: *mut hFILE) -> ::core::ffi::c_int;
    }
    if (*fp).begin < (*fp).limit {
        let fresh152 = (*fp).begin;
        (*fp).begin = (*fp).begin.offset(1);
        *fresh152 = c as ::core::ffi::c_char;
    } else {
        c = hputc2_0(c, fp);
    }
    return c;
}
#[inline]
unsafe extern "C" fn hputs(
    mut text: *const ::core::ffi::c_char,
    mut fp: *mut hFILE,
) -> ::core::ffi::c_int {
    extern "C" {
        #[link_name = "hputs2"]
        fn hputs2_0(
            _: *const ::core::ffi::c_char,
            _: size_t,
            _: size_t,
            _: *mut hFILE,
        ) -> ::core::ffi::c_int;
    }
    let mut nbytes: size_t = strlen(text);
    let mut n: size_t = (*fp).limit.offset_from((*fp).begin) as ::core::ffi::c_long as size_t;
    if n > nbytes {
        n = nbytes;
    }
    memcpy(
        (*fp).begin as *mut ::core::ffi::c_void,
        text as *const ::core::ffi::c_void,
        n,
    );
    (*fp).begin = (*fp).begin.offset(n as isize);
    return if n == nbytes {
        0 as ::core::ffi::c_int
    } else {
        hputs2_0(text, nbytes, n, fp)
    };
}
#[inline]
unsafe extern "C" fn hwrite(
    mut fp: *mut hFILE,
    mut buffer: *const ::core::ffi::c_void,
    mut nbytes: size_t,
) -> ssize_t {
    extern "C" {
        #[link_name = "hwrite2"]
        fn hwrite2_0(_: *mut hFILE, _: *const ::core::ffi::c_void, _: size_t, _: size_t)
            -> ssize_t;
    }
    extern "C" {
        #[link_name = "hfile_set_blksize"]
        fn hfile_set_blksize_0(fp_0: *mut hFILE, bufsiz: size_t) -> ::core::ffi::c_int;
    }
    if (*fp).mobile() == 0 {
        let mut n: size_t = (*fp).limit.offset_from((*fp).begin) as ::core::ffi::c_long as size_t;
        if n < nbytes {
            hfile_set_blksize_0(
                fp,
                ((*fp).limit.offset_from((*fp).buffer) as ::core::ffi::c_long as size_t)
                    .wrapping_add(nbytes),
            );
            (*fp).end = (*fp).limit;
        }
    }
    let mut n_0: size_t = (*fp).limit.offset_from((*fp).begin) as ::core::ffi::c_long as size_t;
    if nbytes >= n_0 && (*fp).begin == (*fp).buffer {
        return hwrite2_0(fp, buffer, nbytes, 0 as size_t);
    }
    if n_0 > nbytes {
        n_0 = nbytes;
    }
    memcpy((*fp).begin as *mut ::core::ffi::c_void, buffer, n_0);
    (*fp).begin = (*fp).begin.offset(n_0 as isize);
    return if n_0 == nbytes {
        n_0 as ssize_t
    } else {
        hwrite2_0(fp, buffer, nbytes, n_0)
    };
}
pub const BAM_CMATCH: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BAM_CINS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BAM_CDEL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BAM_CREF_SKIP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const BAM_CSOFT_CLIP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const BAM_CHARD_CLIP: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const BAM_CPAD: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const BAM_FPAIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BAM_FUNMAP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const BAM_FMUNMAP: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const BAM_FREVERSE: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const BAM_FMREVERSE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const BAM_FREAD1: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn bam_hdr_init() -> *mut sam_hdr_t {
    return sam_hdr_init();
}
#[inline]
unsafe extern "C" fn bam_hdr_destroy(mut h: *mut sam_hdr_t) {
    sam_hdr_destroy(h);
}
#[inline]
unsafe extern "C" fn bam_hdr_dup(mut h0: *const sam_hdr_t) -> *mut sam_hdr_t {
    return sam_hdr_dup(h0);
}
#[inline]
unsafe extern "C" fn bam_name2id(
    mut h: *mut sam_hdr_t,
    mut ref_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return sam_hdr_name2tid(h, ref_0);
}
#[inline]
unsafe extern "C" fn bam_set_mempolicy(mut b: *mut bam1_t, mut policy: uint32_t) {
    (*b).set_mempolicy(policy as uint32_t);
}
#[inline]
unsafe extern "C" fn bam_get_mempolicy(mut b: *mut bam1_t) -> uint32_t {
    return (*b).mempolicy();
}
#[inline]
unsafe extern "C" fn sam_itr_next(
    mut htsfp: *mut htsFile,
    mut itr: *mut hts_itr_t,
    mut r: *mut bam1_t,
) -> ::core::ffi::c_int {
    if (*htsfp).is_bgzf() == 0 && (*htsfp).is_cram() == 0 {
        hts_log(
            HTS_LOG_ERROR,
            b"sam_itr_next\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s not BGZF compressed\0" as *const u8 as *const ::core::ffi::c_char,
            if !(*htsfp).fn_0.is_null() {
                (*htsfp).fn_0 as *const ::core::ffi::c_char
            } else {
                b"File\0" as *const u8 as *const ::core::ffi::c_char
            },
        );
        return -(2 as ::core::ffi::c_int);
    }
    if itr.is_null() {
        hts_log(
            HTS_LOG_ERROR,
            b"sam_itr_next\0" as *const u8 as *const ::core::ffi::c_char,
            b"Null iterator\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return -(2 as ::core::ffi::c_int);
    }
    if (*itr).multi() != 0 {
        return hts_itr_multi_next(htsfp, itr, r as *mut ::core::ffi::c_void);
    } else {
        return hts_itr_next(
            if (*htsfp).is_bgzf() as ::core::ffi::c_int != 0 {
                (*htsfp).fp.bgzf
            } else {
                ::core::ptr::null_mut::<BGZF>()
            },
            itr,
            r as *mut ::core::ffi::c_void,
            htsfp as *mut ::core::ffi::c_void,
        );
    };
}
#[inline]
unsafe extern "C" fn sam_format_aux1(
    mut key: *const uint8_t,
    type_0: uint8_t,
    mut tag: *const uint8_t,
    mut end: *const uint8_t,
    mut ks: *mut kstring_t,
) -> *const uint8_t {
    let mut current_block: u64;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *const uint8_t = tag;
    r |= (kputsn_(
        key as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        2 as size_t,
        ks,
    ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    r |= (kputc_(':' as i32, ks) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    if type_0 as ::core::ffi::c_int == 'C' as i32 {
        r |= (kputsn_(
            b"i:\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
            ks,
        ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        r |= (kputw(*s as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        s = s.offset(1);
    } else if type_0 as ::core::ffi::c_int == 'c' as i32 {
        r |= (kputsn_(
            b"i:\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
            ks,
        ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        r |= (kputw(*(s as *mut int8_t) as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        s = s.offset(1);
    } else {
        if type_0 as ::core::ffi::c_int == 'S' as i32 {
            if end.offset_from(s) as ::core::ffi::c_long >= 2 as ::core::ffi::c_long {
                r |= (kputsn_(
                    b"i:\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    2 as size_t,
                    ks,
                ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
                s = s.offset(2 as ::core::ffi::c_int as isize);
                current_block = 16778110326724371720;
            } else {
                current_block = 17576619569270101465;
            }
        } else if type_0 as ::core::ffi::c_int == 's' as i32 {
            if end.offset_from(s) as ::core::ffi::c_long >= 2 as ::core::ffi::c_long {
                r |= (kputsn_(
                    b"i:\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    2 as size_t,
                    ks,
                ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
                s = s.offset(2 as ::core::ffi::c_int as isize);
                current_block = 16778110326724371720;
            } else {
                current_block = 17576619569270101465;
            }
        } else if type_0 as ::core::ffi::c_int == 'I' as i32 {
            if end.offset_from(s) as ::core::ffi::c_long >= 4 as ::core::ffi::c_long {
                r |= (kputsn_(
                    b"i:\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    2 as size_t,
                    ks,
                ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
                s = s.offset(4 as ::core::ffi::c_int as isize);
                current_block = 16778110326724371720;
            } else {
                current_block = 17576619569270101465;
            }
        } else if type_0 as ::core::ffi::c_int == 'i' as i32 {
            if end.offset_from(s) as ::core::ffi::c_long >= 4 as ::core::ffi::c_long {
                r |= (kputsn_(
                    b"i:\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    2 as size_t,
                    ks,
                ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
                s = s.offset(4 as ::core::ffi::c_int as isize);
                current_block = 16778110326724371720;
            } else {
                current_block = 17576619569270101465;
            }
        } else if type_0 as ::core::ffi::c_int == 'A' as i32 {
            r |= (kputsn_(
                b"A:\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                2 as size_t,
                ks,
            ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            r |= (kputc_(*s as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            s = s.offset(1);
            current_block = 16778110326724371720;
        } else if type_0 as ::core::ffi::c_int == 'f' as i32 {
            if end.offset_from(s) as ::core::ffi::c_long >= 4 as ::core::ffi::c_long {
                ksprintf(
                    ks,
                    b"f:%g\0" as *const u8 as *const ::core::ffi::c_char,
                    le_to_float(s) as ::core::ffi::c_double,
                );
                s = s.offset(4 as ::core::ffi::c_int as isize);
                current_block = 16778110326724371720;
            } else {
                current_block = 17576619569270101465;
            }
        } else if type_0 as ::core::ffi::c_int == 'd' as i32 {
            if end.offset_from(s) as ::core::ffi::c_long >= 8 as ::core::ffi::c_long {
                ksprintf(
                    ks,
                    b"d:%g\0" as *const u8 as *const ::core::ffi::c_char,
                    le_to_double(s),
                );
                s = s.offset(8 as ::core::ffi::c_int as isize);
                current_block = 16778110326724371720;
            } else {
                current_block = 17576619569270101465;
            }
        } else if type_0 as ::core::ffi::c_int == 'Z' as i32
            || type_0 as ::core::ffi::c_int == 'H' as i32
        {
            r |= (kputc_(type_0 as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            r |= (kputc_(':' as i32, ks) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            while s < end && *s as ::core::ffi::c_int != 0 {
                let fresh5 = s;
                s = s.offset(1);
                r |= (kputc_(*fresh5 as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
            }
            r |= (kputsn(
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                0 as size_t,
                ks,
            ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            if s >= end {
                current_block = 17576619569270101465;
            } else {
                s = s.offset(1);
                current_block = 16778110326724371720;
            }
        } else if type_0 as ::core::ffi::c_int == 'B' as i32 {
            let fresh6 = s;
            s = s.offset(1);
            let mut sub_type: uint8_t = *fresh6;
            let mut sub_type_size: ::core::ffi::c_uint = 0;
            match sub_type as ::core::ffi::c_int {
                65 | 99 | 67 => {
                    sub_type_size = 1 as ::core::ffi::c_uint;
                }
                115 | 83 => {
                    sub_type_size = 2 as ::core::ffi::c_uint;
                }
                105 | 73 | 102 => {
                    sub_type_size = 4 as ::core::ffi::c_uint;
                }
                _ => {
                    sub_type_size = 0 as ::core::ffi::c_uint;
                }
            }
            let mut i: uint32_t = 0;
            let mut n: uint32_t = 0;
            if sub_type_size == 0 as ::core::ffi::c_uint
                || (end.offset_from(s) as ::core::ffi::c_long) < 4 as ::core::ffi::c_long
            {
                current_block = 17576619569270101465;
            } else {
                n = le_to_u32(s);
                s = s.offset(4 as ::core::ffi::c_int as isize);
                if (end.offset_from(s) as ::core::ffi::c_long as size_t)
                    .wrapping_div(sub_type_size as size_t)
                    < n as size_t
                {
                    current_block = 17576619569270101465;
                } else {
                    r |= (kputsn_(
                        b"B:\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        2 as size_t,
                        ks,
                    ) < 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                    r |= (kputc(sub_type as ::core::ffi::c_int, ks) < 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int;
                    match sub_type as ::core::ffi::c_int {
                        99 => {
                            current_block = 1178848138987604321;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        67 => {
                            current_block = 383330402035013330;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        115 => {
                            current_block = 5527589250341021540;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        83 => {
                            current_block = 1063862699172499293;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        105 => {
                            current_block = 2210070406271078657;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        73 => {
                            current_block = 2376418918778612865;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        102 => {
                            current_block = 10186842371145338957;
                            match current_block {
                                10186842371145338957 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh13 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh13 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |=
                                                (kputd(le_to_float(s) as ::core::ffi::c_double, ks)
                                                    < 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                383330402035013330 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh8 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh8 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(
                                                *(s as *mut uint8_t) as ::core::ffi::c_uint,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                5527589250341021540 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh9 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh9 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i16(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                1063862699172499293 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh10 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh10 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u16(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(2 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2210070406271078657 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh11 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh11 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(le_to_i32(s) as ::core::ffi::c_int, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                2376418918778612865 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh12 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh12 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputuw(le_to_u32(s) as ::core::ffi::c_uint, ks)
                                                < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(4 as ::core::ffi::c_int as isize);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                                _ => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 15724872316646697593;
                                    } else {
                                        i = 0 as uint32_t;
                                        while i < n {
                                            let fresh7 = (*ks).l;
                                            (*ks).l = (*ks).l.wrapping_add(1);
                                            *(*ks).s.offset(fresh7 as isize) =
                                                ',' as i32 as ::core::ffi::c_char;
                                            r |= (kputw(
                                                *(s as *mut int8_t) as ::core::ffi::c_int,
                                                ks,
                                            ) < 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int;
                                            s = s.offset(1);
                                            i = i.wrapping_add(1);
                                        }
                                        current_block = 16778110326724371720;
                                    }
                                }
                            }
                            match current_block {
                                16778110326724371720 => {}
                                _ => {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"sam_format_aux1\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Out of memory\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    *__errno_location() = ENOMEM;
                                    return ::core::ptr::null::<uint8_t>();
                                }
                            }
                        }
                        _ => {
                            current_block = 17576619569270101465;
                        }
                    }
                }
            }
        } else {
            current_block = 17576619569270101465;
        }
        match current_block {
            16778110326724371720 => {}
            _ => {
                *__errno_location() = EINVAL;
                return ::core::ptr::null::<uint8_t>();
            }
        }
    }
    return if r != 0 {
        ::core::ptr::null::<uint8_t>()
    } else {
        s
    };
}
#[inline]
unsafe extern "C" fn bam_aux_tag(mut s: *const uint8_t) -> *const ::core::ffi::c_char {
    return s.offset(-(2 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn bam_aux_type(mut s: *const uint8_t) -> ::core::ffi::c_char {
    return *s as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn bam_aux_get_str(
    mut b: *const bam1_t,
    mut tag: *const ::core::ffi::c_char,
    mut s: *mut kstring_t,
) -> ::core::ffi::c_int {
    let mut t: *const uint8_t = bam_aux_get(b, tag);
    if t.is_null() {
        return if *__errno_location() == ENOENT {
            0 as ::core::ffi::c_int
        } else {
            -(1 as ::core::ffi::c_int)
        };
    }
    if sam_format_aux1(
        t.offset(-(2 as ::core::ffi::c_int as isize)),
        *t,
        t.offset(1 as ::core::ffi::c_int as isize),
        (*b).data.offset((*b).l_data as isize),
        s,
    )
    .is_null()
    {
        return -(1 as ::core::ffi::c_int);
    }
    return 1 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn sam_hdr_parse_(
    mut hdr: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut SAM_hdr {
    return sam_hdr_parse(len, hdr) as *mut SAM_hdr;
}
#[inline]
unsafe extern "C" fn sam_hdr_free(mut hdr: *mut SAM_hdr) {
    sam_hdr_destroy(hdr as *mut sam_hdr_t);
}
#[inline]
unsafe extern "C" fn TYPEKEY(mut type_0: *const ::core::ffi::c_char) -> khint32_t {
    let mut u0: ::core::ffi::c_uint = *type_0.offset(0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_uchar as ::core::ffi::c_uint;
    let mut u1: ::core::ffi::c_uint = *type_0.offset(1 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_uchar as ::core::ffi::c_uint;
    return (u0 as khint32_t) << 8 as ::core::ffi::c_int | u1 as khint32_t;
}
#[inline]
unsafe extern "C" fn kh_destroy_sam_hrecs_t(mut h: *mut kh_sam_hrecs_t_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_get_sam_hrecs_t(
    mut h: *const kh_sam_hrecs_t_t,
    mut key: khint32_t,
) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = key as khint_t;
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_sam_hrecs_t(
    mut h: *mut kh_sam_hrecs_t_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut khint32_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut *mut sam_hrec_type_t = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut sam_hrec_type_t>() as size_t),
            )
                as *mut *mut sam_hrec_type_t;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: khint32_t = *(*h).keys.offset(j as isize);
                let mut val: *mut sam_hrec_type_t = ::core::ptr::null_mut::<sam_hrec_type_t>();
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh14 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh14 = (*fresh14 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = key as khint_t;
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh15 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh15 = (*fresh15 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: khint32_t = *(*h).keys.offset(i as isize);
                        *(*h).keys.offset(i as isize) = key;
                        key = tmp;
                        let mut tmp_0: *mut sam_hrec_type_t = *(*h).vals.offset(i as isize);
                        let ref mut fresh16 = *(*h).vals.offset(i as isize);
                        *fresh16 = val;
                        val = tmp_0;
                        let ref mut fresh17 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh17 = (*fresh17 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        let ref mut fresh18 = *(*h).vals.offset(i as isize);
                        *fresh18 = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
            ) as *mut khint32_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<*mut sam_hrec_type_t>() as size_t),
            ) as *mut *mut sam_hrec_type_t;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_clear_sam_hrecs_t(mut h: *mut kh_sam_hrecs_t_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_sam_hrecs_t(
    mut h: *mut kh_sam_hrecs_t_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_sam_hrecs_t(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_sam_hrecs_t(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_put_sam_hrecs_t(
    mut h: *mut kh_sam_hrecs_t_t,
    mut key: khint32_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_sam_hrecs_t(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_sam_hrecs_t(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = key as khint_t;
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(*(*h).keys.offset(i as isize) == key))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh19 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh19 = (*fresh19 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        *(*h).keys.offset(x as isize) = key;
        let ref mut fresh20 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh20 = (*fresh20 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_del_sam_hrecs_t(mut h: *mut kh_sam_hrecs_t_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh21 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh21 = (*fresh21 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_stats_sam_hrecs_t(
    mut h: *mut kh_sam_hrecs_t_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (*(*h).keys.offset(i as isize)
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh22 = *hist.offset(dist as isize);
            *fresh22 = (*fresh22).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_init_sam_hrecs_t() -> *mut kh_sam_hrecs_t_t {
    return calloc(
        1 as size_t,
        ::core::mem::size_of::<kh_sam_hrecs_t_t>() as size_t,
    ) as *mut kh_sam_hrecs_t_t;
}
#[inline]
unsafe extern "C" fn kh_grow_to_fit_m_s2i(
    mut h: *mut kh_m_s2i_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_m_s2i(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
    }
    if UINT_MAX.wrapping_sub((*h).size) < n_items {
        return -(2 as ::core::ffi::c_int);
    }
    n_items = (n_items as ::core::ffi::c_uint).wrapping_add((*h).size as ::core::ffi::c_uint)
        as khint_t as khint_t;
    if n_items >= resize_limit {
        return -(2 as ::core::ffi::c_int);
    }
    n_buckets = (if n_items > 0 as ::core::ffi::c_uint {
        n_items as ::core::ffi::c_uint
    } else {
        1 as ::core::ffi::c_uint
    }) as khint_t;
    if n_buckets > 0 as ::core::ffi::c_uint {
        n_buckets = n_buckets.wrapping_sub(1);
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |=
            (n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        n_buckets |= (n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_add(
            (n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64
        < n_items as ::core::ffi::c_double
    {
        n_buckets = (n_buckets as ::core::ffi::c_uint).wrapping_mul(2 as ::core::ffi::c_uint)
            as khint_t as khint_t;
    }
    return kh_resize_m_s2i(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_destroy_m_s2i(mut h: *mut kh_m_s2i_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_init_m_s2i() -> *mut kh_m_s2i_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_m_s2i_t>() as size_t) as *mut kh_m_s2i_t;
}
#[inline]
unsafe extern "C" fn kh_clear_m_s2i(mut h: *mut kh_m_s2i_t) {
    if !h.is_null() && !(*h).flags.is_null() {
        memset(
            (*h).flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if (*h).n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                (*h).n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        (*h).n_occupied = 0 as khint_t;
        (*h).size = (*h).n_occupied;
    }
}
#[inline]
unsafe extern "C" fn kh_get_m_s2i(mut h: *const kh_m_s2i_t, mut key: kh_cstr_t) -> khint_t {
    if (*h).n_buckets != 0 {
        let mut k: khint_t = 0;
        let mut i: khint_t = 0;
        let mut last: khint_t = 0;
        let mut mask: khint_t = 0;
        let mut step: khint_t = 0 as khint_t;
        mask = ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint)
            as khint_t;
        k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
        i = k & mask;
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(strcmp(
                    *(*h).keys.offset(i as isize) as *const ::core::ffi::c_char,
                    key as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int))
        {
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if i == last {
                return (*h).n_buckets;
            }
        }
        return if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            != 0
        {
            (*h).n_buckets
        } else {
            i
        };
    } else {
        return 0 as khint_t;
    };
}
#[inline]
unsafe extern "C" fn kh_resize_m_s2i(
    mut h: *mut kh_m_s2i_t,
    mut new_n_buckets: khint_t,
) -> ::core::ffi::c_int {
    let mut new_flags: *mut khint32_t = ::core::ptr::null_mut::<khint32_t>();
    let mut j: khint_t = 1 as khint_t;
    if new_n_buckets > 0 as ::core::ffi::c_uint {
        new_n_buckets = new_n_buckets.wrapping_sub(1);
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(8 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_div(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |=
            (new_n_buckets >> ::core::mem::size_of::<khint_t>() as usize) as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(2 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets |= (new_n_buckets
            >> (::core::mem::size_of::<khint_t>() as usize).wrapping_mul(4 as usize))
            as ::core::ffi::c_uint;
        new_n_buckets = (new_n_buckets as ::core::ffi::c_uint).wrapping_add(
            (new_n_buckets as ::core::ffi::c_uint
                >> (::core::mem::size_of::<khint_t>() as usize)
                    .wrapping_mul(8 as usize)
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(
                        !((new_n_buckets as ::core::ffi::c_uint)
                            .wrapping_mul(0 as ::core::ffi::c_uint)
                            .wrapping_add(1 as ::core::ffi::c_uint)
                            .wrapping_neg()
                            > 0 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as usize,
                    )
                & 1 as ::core::ffi::c_uint
                == 0) as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) as khint_t as khint_t;
    } else {
    };
    if new_n_buckets < 4 as ::core::ffi::c_uint {
        new_n_buckets = 4 as khint_t;
    }
    if (*h).size >= (new_n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t {
        j = 0 as khint_t;
    } else {
        new_flags = malloc(
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        ) as *mut khint32_t;
        if new_flags.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        memset(
            new_flags as *mut ::core::ffi::c_void,
            0xaa as ::core::ffi::c_int,
            ((if new_n_buckets < 16 as ::core::ffi::c_uint {
                1 as ::core::ffi::c_uint
            } else {
                new_n_buckets as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int
            }) as size_t)
                .wrapping_mul(::core::mem::size_of::<khint32_t>() as size_t),
        );
        if (*h).n_buckets < new_n_buckets {
            let mut new_keys: *mut kh_cstr_t = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<kh_cstr_t>() as size_t),
            ) as *mut kh_cstr_t;
            if new_keys.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).keys = new_keys;
            let mut new_vals: *mut ::core::ffi::c_int = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>() as size_t),
            ) as *mut ::core::ffi::c_int;
            if new_vals.is_null() {
                free(new_flags as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            (*h).vals = new_vals;
        }
    }
    if j != 0 {
        j = 0 as khint_t;
        while j != (*h).n_buckets {
            if *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
            {
                let mut key: kh_cstr_t = *(*h).keys.offset(j as isize);
                let mut val: ::core::ffi::c_int = 0;
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh23 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh23 = (*fresh23 as ::core::ffi::c_ulong
                    | (1 as ::core::ffi::c_ulong)
                        << ((j as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)) as khint32_t;
                loop {
                    let mut k: khint_t = 0;
                    let mut i: khint_t = 0;
                    let mut step: khint_t = 0 as khint_t;
                    k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
                    i = k & new_mask;
                    while *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                            << 1 as ::core::ffi::c_int)
                        & 2 as ::core::ffi::c_uint
                        == 0
                    {
                        step = step.wrapping_add(1);
                        i = i.wrapping_add(step) & new_mask;
                    }
                    let ref mut fresh24 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh24 = (*fresh24 as ::core::ffi::c_ulong
                        & !((2 as ::core::ffi::c_ulong)
                            << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)))
                        as khint32_t;
                    if i < (*h).n_buckets
                        && *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                << 1 as ::core::ffi::c_int)
                            & 3 as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        let mut tmp: kh_cstr_t = *(*h).keys.offset(i as isize);
                        let ref mut fresh25 = *(*h).keys.offset(i as isize);
                        *fresh25 = key;
                        key = tmp;
                        let mut tmp_0: ::core::ffi::c_int = *(*h).vals.offset(i as isize);
                        *(*h).vals.offset(i as isize) = val;
                        val = tmp_0;
                        let ref mut fresh26 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh26 = (*fresh26 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        let ref mut fresh27 = *(*h).keys.offset(i as isize);
                        *fresh27 = key;
                        *(*h).vals.offset(i as isize) = val;
                        break;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if (*h).n_buckets > new_n_buckets {
            (*h).keys = realloc(
                (*h).keys as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<kh_cstr_t>() as size_t),
            ) as *mut kh_cstr_t;
            (*h).vals = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>() as size_t),
            ) as *mut ::core::ffi::c_int;
        }
        free((*h).flags as *mut ::core::ffi::c_void);
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound =
            ((*h).n_buckets as ::core::ffi::c_double * __ac_HASH_UPPER + 0.5f64) as khint_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn kh_del_m_s2i(mut h: *mut kh_m_s2i_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh32 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh32 = (*fresh32 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_put_m_s2i(
    mut h: *mut kh_m_s2i_t,
    mut key: kh_cstr_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_m_s2i(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_m_s2i(h, (*h).n_buckets.wrapping_add(1 as khint_t))
            < 0 as ::core::ffi::c_int
        {
            *ret = -(1 as ::core::ffi::c_int);
            return (*h).n_buckets;
        }
    }
    let mut k: khint_t = 0;
    let mut i: khint_t = 0;
    let mut site: khint_t = 0;
    let mut last: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    let mut step: khint_t = 0 as khint_t;
    site = (*h).n_buckets;
    x = site;
    k = __ac_FNV1a_hash_string(key as *const ::core::ffi::c_char);
    i = k & mask;
    if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        x = i;
    } else {
        last = i;
        while *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            == 0
            && (*(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
                || !(strcmp(
                    *(*h).keys.offset(i as isize) as *const ::core::ffi::c_char,
                    key as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int))
        {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 1 as ::core::ffi::c_uint
                != 0
            {
                site = i;
            }
            step = step.wrapping_add(1);
            i = i.wrapping_add(step) & mask;
            if !(i == last) {
                continue;
            }
            x = site;
            break;
        }
        if x == (*h).n_buckets {
            if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 2 as ::core::ffi::c_uint
                != 0
                && site != (*h).n_buckets
            {
                x = site;
            } else {
                x = i;
            }
        }
    }
    if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 2 as ::core::ffi::c_uint
        != 0
    {
        let ref mut fresh28 = *(*h).keys.offset(x as isize);
        *fresh28 = key;
        let ref mut fresh29 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh29 = (*fresh29 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        (*h).n_occupied = (*h).n_occupied.wrapping_add(1);
        *ret = 1 as ::core::ffi::c_int;
    } else if *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
        >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
        & 1 as ::core::ffi::c_uint
        != 0
    {
        let ref mut fresh30 = *(*h).keys.offset(x as isize);
        *fresh30 = key;
        let ref mut fresh31 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh31 = (*fresh31 as ::core::ffi::c_ulong
            & !((3 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int))) as khint32_t;
        (*h).size = (*h).size.wrapping_add(1);
        *ret = 2 as ::core::ffi::c_int;
    } else {
        *ret = 0 as ::core::ffi::c_int;
    }
    return x;
}
#[inline]
unsafe extern "C" fn kh_stats_m_s2i(
    mut h: *mut kh_m_s2i_t,
    mut empty: *mut khint_t,
    mut deleted: *mut khint_t,
    mut hist_size: *mut khint_t,
    mut hist_out: *mut *mut khint_t,
) -> ::core::ffi::c_int {
    let mut i: khint_t = 0;
    let mut hist: *mut khint_t = ::core::ptr::null_mut::<khint_t>();
    let mut dist_max: khint_t = 0 as khint_t;
    let mut k: khint_t = 0;
    let mut dist: khint_t = 0;
    let mut step: khint_t = 0;
    let mut mask: khint_t = (*h).n_buckets.wrapping_sub(1 as khint_t);
    *hist_size = 0 as khint_t;
    *deleted = *hist_size;
    *empty = *deleted;
    hist = calloc(1 as size_t, ::core::mem::size_of::<khint_t>() as size_t) as *mut khint_t;
    if hist.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as ::core::ffi::c_int as khint_t;
    while i < (*h).n_buckets {
        if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 2 as ::core::ffi::c_uint
            != 0
        {
            *empty = (*empty).wrapping_add(1);
        } else if *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 1 as ::core::ffi::c_uint
            != 0
        {
            *deleted = (*deleted).wrapping_add(1);
        } else {
            k = (__ac_FNV1a_hash_string(*(*h).keys.offset(i as isize) as *const ::core::ffi::c_char)
                as ::core::ffi::c_uint
                & ((*h).n_buckets as ::core::ffi::c_uint).wrapping_sub(1 as ::core::ffi::c_uint))
                as khint_t;
            dist = 0 as khint_t;
            step = 0 as khint_t;
            while k != i {
                dist = dist.wrapping_add(1);
                step = step.wrapping_add(1);
                k = k.wrapping_add(step) & mask;
            }
            if dist_max <= dist {
                let mut new_hist: *mut khint_t = realloc(
                    hist as *mut ::core::ffi::c_void,
                    (::core::mem::size_of::<khint_t>() as size_t).wrapping_mul(
                        (dist as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                            as size_t,
                    ),
                ) as *mut khint_t;
                if new_hist.is_null() {
                    free(hist as *mut ::core::ffi::c_void);
                    return -(1 as ::core::ffi::c_int);
                }
                k = (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint)
                    as khint_t;
                while k <= dist {
                    *new_hist.offset(k as isize) = 0 as khint_t;
                    k = k.wrapping_add(1);
                }
                hist = new_hist;
                dist_max = dist;
            }
            let ref mut fresh33 = *hist.offset(dist as isize);
            *fresh33 = (*fresh33).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
static mut __ac_HASH_UPPER: ::core::ffi::c_double = 0.77f64;
#[inline]
unsafe extern "C" fn __ac_X31_hash_string(mut s: *const ::core::ffi::c_char) -> khint_t {
    let mut h: khint_t = *s as khint_t;
    if h != 0 {
        s = s.offset(1);
        while *s != 0 {
            h = (h << 5 as ::core::ffi::c_int)
                .wrapping_sub(h)
                .wrapping_add(*s as khint_t);
            s = s.offset(1);
        }
    }
    return h;
}
#[inline]
unsafe extern "C" fn __ac_FNV1a_hash_string(mut s: *const ::core::ffi::c_char) -> khint_t {
    let offset_basis: khint_t = 2166136261 as ::core::ffi::c_long as khint_t;
    let FNV_prime: khint_t = 16777619 as ::core::ffi::c_int as khint_t;
    let mut h: khint_t = offset_basis;
    while *s != 0 {
        h = (h ^ *s as uint8_t as khint_t).wrapping_mul(FNV_prime);
        s = s.offset(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn __ac_X31_hash_kstring(ks: kstring_t) -> khint_t {
    let mut h: khint_t = 0 as khint_t;
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < ks.l {
        h = (h << 5 as ::core::ffi::c_int)
            .wrapping_sub(h)
            .wrapping_add(*ks.s.offset(i as isize) as khint_t);
        i = i.wrapping_add(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn __ac_FNV1a_hash_kstring(ks: kstring_t) -> khint_t {
    let offset_basis: khint_t = 2166136261 as ::core::ffi::c_long as khint_t;
    let FNV_prime: khint_t = 16777619 as ::core::ffi::c_int as khint_t;
    let mut h: khint_t = offset_basis;
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < ks.l {
        h = (h ^ *ks.s.offset(i as isize) as uint8_t as khint_t).wrapping_mul(FNV_prime);
        i = i.wrapping_add(1);
    }
    return h;
}
#[inline]
unsafe extern "C" fn __ac_Wang_hash(mut key: khint_t) -> khint_t {
    key = (key as ::core::ffi::c_uint)
        .wrapping_add(!(key << 15 as ::core::ffi::c_int) as ::core::ffi::c_uint)
        as khint_t as khint_t;
    key ^= (key >> 10 as ::core::ffi::c_int) as ::core::ffi::c_uint;
    key = (key as ::core::ffi::c_uint)
        .wrapping_add((key << 3 as ::core::ffi::c_int) as ::core::ffi::c_uint) as khint_t
        as khint_t;
    key ^= (key >> 6 as ::core::ffi::c_int) as ::core::ffi::c_uint;
    key = (key as ::core::ffi::c_uint)
        .wrapping_add(!(key << 11 as ::core::ffi::c_int) as ::core::ffi::c_uint)
        as khint_t as khint_t;
    key ^= (key >> 16 as ::core::ffi::c_int) as ::core::ffi::c_uint;
    return key;
}
#[inline]
unsafe extern "C" fn cram_not_enough_bits(
    mut blk: *mut cram_block,
    mut nbits: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if nbits < 0 as ::core::ffi::c_int
        || (*blk).byte >= (*blk).uncomp_size as size_t && nbits > 0 as ::core::ffi::c_int
        || ((*blk).uncomp_size as size_t).wrapping_sub((*blk).byte)
            <= (INT32_MAX / 8 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t
            && ((*blk).uncomp_size as size_t)
                .wrapping_sub((*blk).byte)
                .wrapping_mul(8 as size_t)
                .wrapping_add((*blk).bit as size_t)
                .wrapping_sub(7 as size_t)
                < nbits as size_t
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn le_to_u8(mut buf: *const uint8_t) -> uint8_t {
    return *buf;
}
#[inline]
unsafe extern "C" fn le_to_u16(mut buf: *const uint8_t) -> uint16_t {
    return *(buf as *mut uint16_u);
}
#[inline]
unsafe extern "C" fn le_to_u32(mut buf: *const uint8_t) -> uint32_t {
    return *(buf as *mut uint32_u);
}
#[inline]
unsafe extern "C" fn le_to_u64(mut buf: *const uint8_t) -> uint64_t {
    return *(buf as *mut uint64_u);
}
#[inline]
unsafe extern "C" fn u16_to_le(mut val: uint16_t, mut buf: *mut uint8_t) {
    *(buf as *mut uint16_u) = val as uint16_u;
}
#[inline]
unsafe extern "C" fn u32_to_le(mut val: uint32_t, mut buf: *mut uint8_t) {
    *(buf as *mut uint32_u) = val as uint32_u;
}
#[inline]
unsafe extern "C" fn u64_to_le(mut val: uint64_t, mut buf: *mut uint8_t) {
    *(buf as *mut uint64_u) = val as uint64_u;
}
#[inline]
unsafe extern "C" fn le_to_i8(mut buf: *const uint8_t) -> int8_t {
    return (if (*buf as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        *buf as int8_t as ::core::ffi::c_int
    } else {
        -((0xff as ::core::ffi::c_int - *buf as ::core::ffi::c_int) as int8_t as ::core::ffi::c_int)
            - 1 as ::core::ffi::c_int
    }) as int8_t;
}
#[inline]
unsafe extern "C" fn le_to_i16(mut buf: *const uint8_t) -> int16_t {
    let mut v: uint16_t = le_to_u16(buf);
    return (if (v as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int {
        v as int16_t as ::core::ffi::c_int
    } else {
        -((0xffff as ::core::ffi::c_int - v as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int)
            - 1 as ::core::ffi::c_int
    }) as int16_t;
}
#[inline]
unsafe extern "C" fn le_to_i32(mut buf: *const uint8_t) -> int32_t {
    let mut v: uint32_t = le_to_u32(buf);
    return if v < 0x80000000 as uint32_t {
        v as int32_t
    } else {
        -((0xffffffff as uint32_t).wrapping_sub(v) as int32_t) - 1 as int32_t
    };
}
#[inline]
unsafe extern "C" fn le_to_i64(mut buf: *const uint8_t) -> int64_t {
    let mut v: uint64_t = le_to_u64(buf);
    return if (v as ::core::ffi::c_ulonglong) < 0x8000000000000000 as ::core::ffi::c_ulonglong {
        v as int64_t
    } else {
        -((0xffffffffffffffff as ::core::ffi::c_ulonglong)
            .wrapping_sub(v as ::core::ffi::c_ulonglong) as int64_t)
            - 1 as int64_t
    };
}
#[inline]
unsafe extern "C" fn i16_to_le(mut val: int16_t, mut buf: *mut uint8_t) {
    u16_to_le(val as uint16_t, buf);
}
#[inline]
unsafe extern "C" fn i32_to_le(mut val: int32_t, mut buf: *mut uint8_t) {
    u32_to_le(val as uint32_t, buf);
}
#[inline]
unsafe extern "C" fn i64_to_le(mut val: int64_t, mut buf: *mut uint8_t) {
    u64_to_le(val as uint64_t, buf);
}
#[inline]
unsafe extern "C" fn le_to_float(mut buf: *const uint8_t) -> ::core::ffi::c_float {
    let mut convert: C2RustUnnamed_14 = C2RustUnnamed_14 { u: 0 };
    convert.u = le_to_u32(buf);
    return convert.f;
}
#[inline]
unsafe extern "C" fn le_to_double(mut buf: *const uint8_t) -> ::core::ffi::c_double {
    let mut convert: C2RustUnnamed_15 = C2RustUnnamed_15 { u: 0 };
    convert.u = le_to_u64(buf);
    return convert.f;
}
#[inline]
unsafe extern "C" fn float_to_le(mut val: ::core::ffi::c_float, mut buf: *mut uint8_t) {
    let mut convert: C2RustUnnamed_16 = C2RustUnnamed_16 { u: 0 };
    convert.f = val;
    u32_to_le(convert.u, buf);
}
#[inline]
unsafe extern "C" fn double_to_le(mut val: ::core::ffi::c_double, mut buf: *mut uint8_t) {
    let mut convert: C2RustUnnamed_17 = C2RustUnnamed_17 { u: 0 };
    convert.f = val;
    u64_to_le(convert.u, buf);
}
#[no_mangle]
// original: cram_decode_TD (htslib/cram/cram_decode.c:70)
pub unsafe extern "C" fn cram_decode_TD(
    mut fd: *mut cram_fd,
    mut cp: *mut ::core::ffi::c_char,
    mut endp: *const ::core::ffi::c_char,
    mut h: *mut cram_block_compression_hdr,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut op: *mut ::core::ffi::c_char = cp;
    let mut dat: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut b: *mut cram_block = ::core::ptr::null_mut::<cram_block>();
    let mut blk_size: int32_t = 0 as int32_t;
    let mut nTL: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut sz: ::core::ffi::c_int = 0;
    let mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    b = cram_new_block(FILE_HEADER, 0 as ::core::ffi::c_int);
    if b.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if !(*h).TD_blk.is_null() || !(*h).TL.is_null() {
        hts_log(
            HTS_LOG_WARNING,
            b"cram_decode_TD\0" as *const u8 as *const ::core::ffi::c_char,
            b"More than one TD block found in compression header\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        cram_free_block((*h).TD_blk);
        free((*h).TL as *mut ::core::ffi::c_void);
        (*h).TD_blk = ::core::ptr::null_mut::<cram_block>();
        (*h).TL = ::core::ptr::null_mut::<*mut ::core::ffi::c_uchar>();
    }
    blk_size =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    if blk_size == 0 {
        (*h).nTL = 0 as ::core::ffi::c_int;
        cram_free_block(b);
        return cp.offset_from(op) as ::core::ffi::c_long as ::core::ffi::c_int;
    }
    if err != 0
        || blk_size < 0 as int32_t
        || (endp.offset_from(cp) as ::core::ffi::c_long) < blk_size as ::core::ffi::c_long
    {
        cram_free_block(b);
        return -(1 as ::core::ffi::c_int);
    }
    if !(block_append(b, cp as *const ::core::ffi::c_void, blk_size as size_t)
        < 0 as ::core::ffi::c_int)
    {
        cp = cp.offset(blk_size as isize);
        sz = cp.offset_from(op) as ::core::ffi::c_long as ::core::ffi::c_int;
        if *(*b)
            .data
            .offset((*b).byte.wrapping_sub(1 as size_t) as isize)
            != 0
        {
            if block_append_char(b, '\0' as i32 as ::core::ffi::c_char) < 0 as ::core::ffi::c_int {
                current_block = 14261187824039300282;
            } else {
                current_block = 5948590327928692120;
            }
        } else {
            current_block = 5948590327928692120;
        }
        match current_block {
            14261187824039300282 => {}
            _ => {
                dat = (*b).data;
                i = 0 as ::core::ffi::c_int;
                nTL = i;
                while (i as size_t) < (*b).byte {
                    nTL += 1;
                    while *dat.offset(i as isize) != 0 {
                        i += 1;
                    }
                    i += 1;
                }
                (*h).TL = calloc(
                    nTL as size_t,
                    ::core::mem::size_of::<*mut ::core::ffi::c_uchar>() as size_t,
                ) as *mut *mut ::core::ffi::c_uchar;
                if (*h).TL.is_null() {
                    cram_free_block(b);
                    return -(1 as ::core::ffi::c_int);
                }
                i = 0 as ::core::ffi::c_int;
                nTL = i;
                while (i as size_t) < (*b).byte {
                    let fresh48 = nTL;
                    nTL = nTL + 1;
                    let ref mut fresh49 = *(*h).TL.offset(fresh48 as isize);
                    *fresh49 = dat.offset(i as isize) as *mut ::core::ffi::c_uchar;
                    while *dat.offset(i as isize) != 0 {
                        i += 1;
                    }
                    i += 1;
                }
                (*h).TD_blk = b;
                (*h).nTL = nTL;
                return sz;
            }
        }
    }
    cram_free_block(b);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
// original: cram_decode_compression_header (htslib/cram/cram_decode.c:144)
pub unsafe extern "C" fn cram_decode_compression_header(
    mut fd: *mut cram_fd,
    mut b: *mut cram_block,
) -> *mut cram_block_compression_hdr {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp_copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut hdr: *mut cram_block_compression_hdr = calloc(
        1 as size_t,
        ::core::mem::size_of::<cram_block_compression_hdr>() as size_t,
    ) as *mut cram_block_compression_hdr;
    let mut i: ::core::ffi::c_int = 0;
    let mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut map_size: int32_t = 0 as int32_t;
    let mut map_count: int32_t = 0 as int32_t;
    if hdr.is_null() {
        return ::core::ptr::null_mut::<cram_block_compression_hdr>();
    }
    if (*b).method as ::core::ffi::c_int != RAW as ::core::ffi::c_int {
        if cram_uncompress_block(b) != 0 {
            free(hdr as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
    }
    cp = (*b).data as *mut ::core::ffi::c_char;
    endp = cp.offset((*b).uncomp_size as isize);
    if (*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        (*hdr).ref_seq_id = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            (*hdr).ref_seq_start = (*fd).vv.varint_get64.expect("non-null function pointer")(
                &raw mut cp,
                endp,
                &raw mut err,
            );
            (*hdr).ref_seq_span = (*fd).vv.varint_get64.expect("non-null function pointer")(
                &raw mut cp,
                endp,
                &raw mut err,
            );
        } else {
            (*hdr).ref_seq_start = (*fd).vv.varint_get32.expect("non-null function pointer")(
                &raw mut cp,
                endp,
                &raw mut err,
            );
            (*hdr).ref_seq_span = (*fd).vv.varint_get32.expect("non-null function pointer")(
                &raw mut cp,
                endp,
                &raw mut err,
            );
        }
        (*hdr).num_records = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        (*hdr).num_landmarks = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        if (*hdr).num_landmarks < 0 as int32_t
            || (*hdr).num_landmarks as usize
                >= (SIZE_MAX as usize).wrapping_div(::core::mem::size_of::<int32_t>() as usize)
            || (endp.offset_from(cp) as ::core::ffi::c_long)
                < (*hdr).num_landmarks as ::core::ffi::c_long
        {
            free(hdr as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
        (*hdr).landmark = malloc(
            ((*hdr).num_landmarks as size_t)
                .wrapping_mul(::core::mem::size_of::<int32_t>() as size_t),
        ) as *mut int32_t;
        if (*hdr).landmark.is_null() {
            free(hdr as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
        i = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*hdr).num_landmarks {
            *(*hdr).landmark.offset(i as isize) = (*fd)
                .vv
                .varint_get32
                .expect("non-null function pointer")(
                &raw mut cp, endp, &raw mut err
            ) as int32_t;
            i += 1;
        }
    }
    (*hdr).preservation_map = kh_init_map();
    memset(
        &raw mut (*hdr).rec_encoding_map as *mut *mut cram_map as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (CRAM_MAP_HASH as size_t).wrapping_mul(::core::mem::size_of::<*mut cram_map>() as size_t),
    );
    memset(
        &raw mut (*hdr).tag_encoding_map as *mut *mut cram_map as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (CRAM_MAP_HASH as size_t).wrapping_mul(::core::mem::size_of::<*mut cram_map>() as size_t),
    );
    if (*hdr).preservation_map.is_null() {
        cram_free_compression_header(hdr);
        return ::core::ptr::null_mut::<cram_block_compression_hdr>();
    }
    (*hdr).read_names_included = 0 as ::core::ffi::c_int;
    (*hdr).AP_delta = 1 as ::core::ffi::c_int;
    (*hdr).qs_seq_orient = 1 as ::core::ffi::c_int;
    memcpy(
        &raw mut (*hdr).substitution_matrix as *mut [::core::ffi::c_char; 4]
            as *mut ::core::ffi::c_void,
        b"CGTNAGTNACTNACGNACGT\0" as *const u8 as *const ::core::ffi::c_char
            as *const ::core::ffi::c_void,
        20 as size_t,
    );
    map_size =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    cp_copy = cp;
    map_count =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    i = 0 as ::core::ffi::c_int;
    while (i as int32_t) < map_count {
        let mut hd: pmap_t = pmap_t { i: 0 };
        let mut k: khint_t = 0;
        let mut r: ::core::ffi::c_int = 0;
        if (endp.offset_from(cp) as ::core::ffi::c_long) < 3 as ::core::ffi::c_long {
            cram_free_compression_header(hdr);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
        cp = cp.offset(2 as ::core::ffi::c_int as isize);
        let mut current_block_132: u64;
        match (*cp.offset(-(2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uchar
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            | *cp.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uchar
                as ::core::ffi::c_int
        {
            21833 => {
                current_block_132 = 16395564228731822224;
            }
            19785 | 20553 => {
                current_block_132 = 16395564228731822224;
            }
            21070 => {
                let fresh35 = cp;
                cp = cp.offset(1);
                hd.i = *fresh35 as ::core::ffi::c_int;
                k = kh_put_map(
                    (*hdr).preservation_map,
                    b"RN\0" as *const u8 as kh_cstr_t,
                    &raw mut r,
                );
                if -(1 as ::core::ffi::c_int) == r {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                *(*(*hdr).preservation_map).vals.offset(k as isize) = hd;
                (*hdr).read_names_included = hd.i;
                current_block_132 = 14865402277128115059;
            }
            16720 => {
                let fresh36 = cp;
                cp = cp.offset(1);
                hd.i = *fresh36 as ::core::ffi::c_int;
                k = kh_put_map(
                    (*hdr).preservation_map,
                    b"AP\0" as *const u8 as kh_cstr_t,
                    &raw mut r,
                );
                if -(1 as ::core::ffi::c_int) == r {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                *(*(*hdr).preservation_map).vals.offset(k as isize) = hd;
                (*hdr).AP_delta = hd.i;
                current_block_132 = 14865402277128115059;
            }
            21074 => {
                let fresh37 = cp;
                cp = cp.offset(1);
                hd.i = *fresh37 as ::core::ffi::c_int;
                k = kh_put_map(
                    (*hdr).preservation_map,
                    b"RR\0" as *const u8 as kh_cstr_t,
                    &raw mut r,
                );
                if -(1 as ::core::ffi::c_int) == r {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                *(*(*hdr).preservation_map).vals.offset(k as isize) = hd;
                (*hdr).no_ref = (hd.i == 0) as ::core::ffi::c_int;
                current_block_132 = 14865402277128115059;
            }
            20815 => {
                let fresh38 = cp;
                cp = cp.offset(1);
                hd.i = *fresh38 as ::core::ffi::c_int;
                k = kh_put_map(
                    (*hdr).preservation_map,
                    b"QO\0" as *const u8 as kh_cstr_t,
                    &raw mut r,
                );
                if -(1 as ::core::ffi::c_int) == r {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                *(*(*hdr).preservation_map).vals.offset(k as isize) = hd;
                (*hdr).qs_seq_orient = hd.i;
                current_block_132 = 14865402277128115059;
            }
            21325 => {
                if (endp.offset_from(cp) as ::core::ffi::c_long) < 5 as ::core::ffi::c_long {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                (*hdr).substitution_matrix[0 as ::core::ffi::c_int as usize][(*cp
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'C' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[0 as ::core::ffi::c_int as usize][(*cp
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'G' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[0 as ::core::ffi::c_int as usize][(*cp
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'T' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[0 as ::core::ffi::c_int as usize][(*cp
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 0 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'N' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[1 as ::core::ffi::c_int as usize][(*cp
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'A' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[1 as ::core::ffi::c_int as usize][(*cp
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'G' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[1 as ::core::ffi::c_int as usize][(*cp
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'T' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[1 as ::core::ffi::c_int as usize][(*cp
                    .offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 0 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'N' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[2 as ::core::ffi::c_int as usize][(*cp
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'A' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[2 as ::core::ffi::c_int as usize][(*cp
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'C' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[2 as ::core::ffi::c_int as usize][(*cp
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'T' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[2 as ::core::ffi::c_int as usize][(*cp
                    .offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 0 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'N' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[3 as ::core::ffi::c_int as usize][(*cp
                    .offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'A' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[3 as ::core::ffi::c_int as usize][(*cp
                    .offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'C' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[3 as ::core::ffi::c_int as usize][(*cp
                    .offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'G' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[3 as ::core::ffi::c_int as usize][(*cp
                    .offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 0 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'N' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[4 as ::core::ffi::c_int as usize][(*cp
                    .offset(4 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'A' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[4 as ::core::ffi::c_int as usize][(*cp
                    .offset(4 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'C' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[4 as ::core::ffi::c_int as usize][(*cp
                    .offset(4 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'G' as i32 as ::core::ffi::c_char;
                (*hdr).substitution_matrix[4 as ::core::ffi::c_int as usize][(*cp
                    .offset(4 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    >> 0 as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int)
                    as usize] = 'T' as i32 as ::core::ffi::c_char;
                hd.p = cp;
                cp = cp.offset(5 as ::core::ffi::c_int as isize);
                k = kh_put_map(
                    (*hdr).preservation_map,
                    b"SM\0" as *const u8 as kh_cstr_t,
                    &raw mut r,
                );
                if -(1 as ::core::ffi::c_int) == r {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                *(*(*hdr).preservation_map).vals.offset(k as isize) = hd;
                current_block_132 = 14865402277128115059;
            }
            21572 => {
                let mut sz: ::core::ffi::c_int = cram_decode_TD(fd, cp, endp, hdr);
                if sz < 0 as ::core::ffi::c_int {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                hd.p = cp;
                cp = cp.offset(sz as isize);
                k = kh_put_map(
                    (*hdr).preservation_map,
                    b"TD\0" as *const u8 as kh_cstr_t,
                    &raw mut r,
                );
                if -(1 as ::core::ffi::c_int) == r {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
                *(*(*hdr).preservation_map).vals.offset(k as isize) = hd;
                current_block_132 = 14865402277128115059;
            }
            _ => {
                hts_log(
                    HTS_LOG_WARNING,
                    b"cram_decode_compression_header\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Unrecognised preservation map key %c%c\0" as *const u8
                        as *const ::core::ffi::c_char,
                    *cp.offset(-(2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int,
                    *cp.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int,
                );
                cp = cp.offset(1);
                current_block_132 = 14865402277128115059;
            }
        }
        match current_block_132 {
            16395564228731822224 => {
                let fresh34 = cp;
                cp = cp.offset(1);
                hd.i = *fresh34 as ::core::ffi::c_int;
            }
            _ => {}
        }
        i += 1;
    }
    if cp.offset_from(cp_copy) as ::core::ffi::c_long != map_size as ::core::ffi::c_long {
        cram_free_compression_header(hdr);
        return ::core::ptr::null_mut::<cram_block_compression_hdr>();
    }
    map_size =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    cp_copy = cp;
    map_count =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    let mut is_v4: ::core::ffi::c_int =
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    i = 0 as ::core::ffi::c_int;
    while (i as int32_t) < map_count {
        let mut key: *mut ::core::ffi::c_char = cp;
        let mut encoding: int32_t = E_NULL as ::core::ffi::c_int as int32_t;
        let mut size: int32_t = 0 as int32_t;
        let mut offset: ptrdiff_t = 0;
        let mut m: *mut cram_map = ::core::ptr::null_mut::<cram_map>();
        let mut ds_id: cram_DS_ID = DS_CORE;
        let mut type_0: cram_external_type = 0 as cram_external_type;
        if (endp.offset_from(cp) as ::core::ffi::c_long) < 4 as ::core::ffi::c_long {
            cram_free_compression_header(hdr);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
        cp = cp.offset(2 as ::core::ffi::c_int as isize);
        encoding = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        size = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        offset = cp.offset_from((*b).data as *mut ::core::ffi::c_char) as ::core::ffi::c_long
            as ptrdiff_t;
        if !(encoding == E_NULL as ::core::ffi::c_int as int32_t) {
            if size < 0 as int32_t
                || (endp.offset_from(cp) as ::core::ffi::c_long) < size as ::core::ffi::c_long
            {
                cram_free_compression_header(hdr);
                return ::core::ptr::null_mut::<cram_block_compression_hdr>();
            }
            ds_id = DS_CORE;
            if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'B' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'F' as i32
            {
                ds_id = DS_BF;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'C' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'F' as i32
            {
                ds_id = DS_CF;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'I' as i32
            {
                ds_id = DS_RI;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'L' as i32
            {
                ds_id = DS_RL;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'A' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'P' as i32
            {
                ds_id = DS_AP;
                type_0 = (if is_v4 != 0 {
                    E_SLONG as ::core::ffi::c_int
                } else {
                    E_INT as ::core::ffi::c_int
                }) as cram_external_type;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'G' as i32
            {
                ds_id = DS_RG;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'M' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'F' as i32
            {
                ds_id = DS_MF;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'N' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'S' as i32
            {
                ds_id = DS_NS;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'N' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'P' as i32
            {
                ds_id = DS_NP;
                type_0 = (if is_v4 != 0 {
                    E_LONG as ::core::ffi::c_int
                } else {
                    E_INT as ::core::ffi::c_int
                }) as cram_external_type;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'T' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'S' as i32
            {
                ds_id = DS_TS;
                type_0 = (if is_v4 != 0 {
                    E_SLONG as ::core::ffi::c_int
                } else {
                    E_INT as ::core::ffi::c_int
                }) as cram_external_type;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'N' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'F' as i32
            {
                ds_id = DS_NF;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'T' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'C' as i32
            {
                ds_id = DS_TC;
                type_0 = E_BYTE;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'T' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
            {
                ds_id = DS_TN;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'F' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
            {
                ds_id = DS_FN;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'F' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'C' as i32
            {
                ds_id = DS_FC;
                type_0 = E_BYTE;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'F' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'P' as i32
            {
                ds_id = DS_FP;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'B' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'S' as i32
            {
                ds_id = DS_BS;
                type_0 = E_BYTE;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'I' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
            {
                ds_id = DS_IN;
                type_0 = E_BYTE_ARRAY;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'S' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'C' as i32
            {
                ds_id = DS_SC;
                type_0 = E_BYTE_ARRAY;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'D' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'L' as i32
            {
                ds_id = DS_DL;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'B' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'A' as i32
            {
                ds_id = DS_BA;
                type_0 = E_BYTE;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'B' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'B' as i32
            {
                ds_id = DS_BB;
                type_0 = E_BYTE_ARRAY;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'S' as i32
            {
                ds_id = DS_RS;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'P' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'D' as i32
            {
                ds_id = DS_PD;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'H' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'C' as i32
            {
                ds_id = DS_HC;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'M' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'Q' as i32
            {
                ds_id = DS_MQ;
                type_0 = E_INT;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
            {
                ds_id = DS_RN;
                type_0 = E_BYTE_ARRAY_BLOCK;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'Q' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'S' as i32
            {
                ds_id = DS_QS;
                type_0 = E_BYTE;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'Q' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'Q' as i32
            {
                ds_id = DS_QQ;
                type_0 = E_BYTE_ARRAY;
            } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'T' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'L' as i32
            {
                ds_id = DS_TL;
                type_0 = E_INT;
            } else if !(*key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'T' as i32
                && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'M' as i32)
            {
                if !(*key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'T' as i32
                    && *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'V' as i32)
                {
                    hts_log(
                        HTS_LOG_WARNING,
                        b"cram_decode_compression_header\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"Unrecognised key: %.2s\0" as *const u8 as *const ::core::ffi::c_char,
                        key,
                    );
                }
            }
            if ds_id as ::core::ffi::c_uint != DS_CORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !(*hdr).codecs[ds_id as usize].is_null() {
                    hts_log(
                        HTS_LOG_WARNING,
                        b"cram_decode_compression_header\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"Codec for key %.2s defined more than once\0" as *const u8
                            as *const ::core::ffi::c_char,
                        key,
                    );
                    (*(*hdr).codecs[ds_id as usize])
                        .free
                        .expect("non-null function pointer")(
                        (*hdr).codecs[ds_id as usize]
                    );
                }
                (*hdr).codecs[ds_id as usize] = cram_decoder_init(
                    hdr,
                    encoding as cram_encoding,
                    cp,
                    size as ::core::ffi::c_int,
                    type_0,
                    (*fd).version,
                    &raw mut (*fd).vv,
                ) as *mut cram_codec;
                if (*hdr).codecs[ds_id as usize].is_null() {
                    cram_free_compression_header(hdr);
                    return ::core::ptr::null_mut::<cram_block_compression_hdr>();
                }
            }
            cp = cp.offset(size as isize);
            m = malloc(::core::mem::size_of::<cram_map>() as size_t) as *mut cram_map;
            if m.is_null() {
                cram_free_compression_header(hdr);
                return ::core::ptr::null_mut::<cram_block_compression_hdr>();
            }
            (*m).key = (*key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
                as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
                | *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
                    as ::core::ffi::c_int;
            (*m).encoding = encoding as cram_encoding;
            (*m).size = size as ::core::ffi::c_int;
            (*m).offset = offset as ::core::ffi::c_int;
            (*m).codec = ::core::ptr::null_mut::<cram_codec>();
            (*m).next = (*hdr).rec_encoding_map[(*key.offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                * 3 as ::core::ffi::c_int
                + *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & CRAM_MAP_HASH - 1 as ::core::ffi::c_int)
                as usize];
            (*hdr).rec_encoding_map[(*key.offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                * 3 as ::core::ffi::c_int
                + *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & CRAM_MAP_HASH - 1 as ::core::ffi::c_int)
                as usize] = m as *mut cram_map;
        }
        i += 1;
    }
    if cp.offset_from(cp_copy) as ::core::ffi::c_long != map_size as ::core::ffi::c_long {
        cram_free_compression_header(hdr);
        return ::core::ptr::null_mut::<cram_block_compression_hdr>();
    }
    map_size =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    cp_copy = cp;
    map_count =
        (*fd).vv.varint_get32.expect("non-null function pointer")(&raw mut cp, endp, &raw mut err)
            as int32_t;
    i = 0 as ::core::ffi::c_int;
    while (i as int32_t) < map_count {
        let mut encoding_0: int32_t = E_NULL as ::core::ffi::c_int as int32_t;
        let mut size_0: int32_t = 0 as int32_t;
        let mut m_0: *mut cram_map =
            malloc(::core::mem::size_of::<cram_map>() as size_t) as *mut cram_map;
        let mut key_0: [uint8_t; 3] = [0; 3];
        if m_0.is_null() || (endp.offset_from(cp) as ::core::ffi::c_long) < 6 as ::core::ffi::c_long
        {
            free(m_0 as *mut ::core::ffi::c_void);
            cram_free_compression_header(hdr);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
        (*m_0).key = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as ::core::ffi::c_int;
        key_0[0 as ::core::ffi::c_int as usize] =
            ((*m_0).key >> 16 as ::core::ffi::c_int) as uint8_t;
        key_0[1 as ::core::ffi::c_int as usize] =
            ((*m_0).key >> 8 as ::core::ffi::c_int) as uint8_t;
        key_0[2 as ::core::ffi::c_int as usize] = (*m_0).key as uint8_t;
        encoding_0 = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        size_0 = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp,
            endp,
            &raw mut err,
        ) as int32_t;
        (*m_0).encoding = encoding_0 as cram_encoding;
        (*m_0).size = size_0 as ::core::ffi::c_int;
        (*m_0).offset = cp.offset_from((*b).data as *mut ::core::ffi::c_char) as ::core::ffi::c_long
            as ::core::ffi::c_int;
        if size_0 < 0 as int32_t
            || (endp.offset_from(cp) as ::core::ffi::c_long) < size_0 as ::core::ffi::c_long
            || {
                (*m_0).codec = cram_decoder_init(
                    hdr,
                    encoding_0 as cram_encoding,
                    cp,
                    size_0 as ::core::ffi::c_int,
                    E_BYTE_ARRAY_BLOCK,
                    (*fd).version,
                    &raw mut (*fd).vv,
                ) as *mut cram_codec;
                (*m_0).codec.is_null()
            }
        {
            cram_free_compression_header(hdr);
            free(m_0 as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<cram_block_compression_hdr>();
        }
        cp = cp.offset(size_0 as isize);
        (*m_0).next = (*hdr).tag_encoding_map[(key_0[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            * 3 as ::core::ffi::c_int
            + key_0[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            & CRAM_MAP_HASH - 1 as ::core::ffi::c_int)
            as usize];
        (*hdr).tag_encoding_map[(key_0[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            * 3 as ::core::ffi::c_int
            + key_0[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            & CRAM_MAP_HASH - 1 as ::core::ffi::c_int) as usize] = m_0 as *mut cram_map;
        i += 1;
    }
    if err != 0 || cp.offset_from(cp_copy) as ::core::ffi::c_long != map_size as ::core::ffi::c_long
    {
        cram_free_compression_header(hdr);
        return ::core::ptr::null_mut::<cram_block_compression_hdr>();
    }
    return hdr;
}
#[no_mangle]
// original: cram_dependent_data_series (htslib/cram/cram_decode.c:553)
pub unsafe extern "C" fn cram_dependent_data_series(
    mut fd: *mut cram_fd,
    mut hdr: *mut cram_block_compression_hdr,
    mut s: *mut cram_slice,
) -> ::core::ffi::c_int {
    let mut block_used: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut core_used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    static mut i_to_id: [::core::ffi::c_int; 28] = [
        DS_BF as ::core::ffi::c_int,
        DS_AP as ::core::ffi::c_int,
        DS_FP as ::core::ffi::c_int,
        DS_RL as ::core::ffi::c_int,
        DS_DL as ::core::ffi::c_int,
        DS_NF as ::core::ffi::c_int,
        DS_BA as ::core::ffi::c_int,
        DS_QS as ::core::ffi::c_int,
        DS_FC as ::core::ffi::c_int,
        DS_FN as ::core::ffi::c_int,
        DS_BS as ::core::ffi::c_int,
        DS_IN as ::core::ffi::c_int,
        DS_RG as ::core::ffi::c_int,
        DS_MQ as ::core::ffi::c_int,
        DS_TL as ::core::ffi::c_int,
        DS_RN as ::core::ffi::c_int,
        DS_NS as ::core::ffi::c_int,
        DS_NP as ::core::ffi::c_int,
        DS_TS as ::core::ffi::c_int,
        DS_MF as ::core::ffi::c_int,
        DS_CF as ::core::ffi::c_int,
        DS_RI as ::core::ffi::c_int,
        DS_RS as ::core::ffi::c_int,
        DS_PD as ::core::ffi::c_int,
        DS_HC as ::core::ffi::c_int,
        DS_SC as ::core::ffi::c_int,
        DS_BB as ::core::ffi::c_int,
        DS_QQ as ::core::ffi::c_int,
    ];
    let mut orig_ds: uint32_t = 0;
    if (*fd).required_fields != 0 && (*fd).required_fields != INT_MAX as ::core::ffi::c_uint {
        (*s).data_series = 0 as ::core::ffi::c_uint;
        if (*fd).required_fields & SAM_QNAME as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_RN as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_BF as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_RNAME as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_RI as ::core::ffi::c_int | CRAM_BF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_POS as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_AP as ::core::ffi::c_int | CRAM_BF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_MAPQ as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_MQ as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_CIGAR as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FN as ::core::ffi::c_int
                | CRAM_FP as ::core::ffi::c_int
                | CRAM_FC as ::core::ffi::c_int
                | CRAM_DL as ::core::ffi::c_int
                | CRAM_IN as ::core::ffi::c_int
                | CRAM_SC as ::core::ffi::c_int
                | CRAM_HC as ::core::ffi::c_int
                | CRAM_PD as ::core::ffi::c_int
                | CRAM_RS as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_RNEXT as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_CF as ::core::ffi::c_int
                | CRAM_NF as ::core::ffi::c_int
                | CRAM_RI as ::core::ffi::c_int
                | CRAM_NS as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_PNEXT as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_CF as ::core::ffi::c_int
                | CRAM_NF as ::core::ffi::c_int
                | CRAM_AP as ::core::ffi::c_int
                | CRAM_NP as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_TLEN as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_CF as ::core::ffi::c_int
                | CRAM_NF as ::core::ffi::c_int
                | CRAM_AP as ::core::ffi::c_int
                | CRAM_TS as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int
                | CRAM_MF as ::core::ffi::c_int
                | CRAM_RI as ::core::ffi::c_int
                | (CRAM_FN as ::core::ffi::c_int
                    | CRAM_FP as ::core::ffi::c_int
                    | CRAM_FC as ::core::ffi::c_int
                    | CRAM_DL as ::core::ffi::c_int
                    | CRAM_IN as ::core::ffi::c_int
                    | CRAM_SC as ::core::ffi::c_int
                    | CRAM_HC as ::core::ffi::c_int
                    | CRAM_PD as ::core::ffi::c_int
                    | CRAM_RS as ::core::ffi::c_int
                    | CRAM_RL as ::core::ffi::c_int
                    | CRAM_BF as ::core::ffi::c_int))
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_SEQ as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FN as ::core::ffi::c_int
                | CRAM_FP as ::core::ffi::c_int
                | CRAM_FC as ::core::ffi::c_int
                | CRAM_DL as ::core::ffi::c_int
                | CRAM_IN as ::core::ffi::c_int
                | CRAM_SC as ::core::ffi::c_int
                | CRAM_HC as ::core::ffi::c_int
                | CRAM_PD as ::core::ffi::c_int
                | CRAM_RS as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int
                | CRAM_BA as ::core::ffi::c_int
                | CRAM_BS as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_AP as ::core::ffi::c_int
                | CRAM_BB as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_AUX as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
            (*s).decode_md = 0 as ::core::ffi::c_int;
        }
        if (*fd).required_fields & SAM_QUAL as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FN as ::core::ffi::c_int
                | CRAM_FP as ::core::ffi::c_int
                | CRAM_FC as ::core::ffi::c_int
                | CRAM_DL as ::core::ffi::c_int
                | CRAM_IN as ::core::ffi::c_int
                | CRAM_SC as ::core::ffi::c_int
                | CRAM_HC as ::core::ffi::c_int
                | CRAM_PD as ::core::ffi::c_int
                | CRAM_RS as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_AP as ::core::ffi::c_int
                | CRAM_QS as ::core::ffi::c_int
                | CRAM_QQ as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_AUX as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_RG as ::core::ffi::c_int
                | CRAM_TL as ::core::ffi::c_int
                | CRAM_aux as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*fd).required_fields & SAM_RGAUX as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_RG as ::core::ffi::c_int | CRAM_BF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if cram_uncompress_block(*(*s).block.offset(0 as ::core::ffi::c_int as isize)) != 0 {
            return -(1 as ::core::ffi::c_int);
        }
    } else {
        (*s).data_series = CRAM_ALL as ::core::ffi::c_int as ::core::ffi::c_uint;
        i = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*(*s).hdr).num_blocks {
            if cram_uncompress_block(*(*s).block.offset(i as isize)) != 0 {
                return -(1 as ::core::ffi::c_int);
            }
            i += 1;
        }
        return 0 as ::core::ffi::c_int;
    }
    block_used = calloc(
        ((*(*s).hdr).num_blocks + 1 as int32_t) as size_t,
        ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
    ) as *mut ::core::ffi::c_int;
    if block_used.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    loop {
        if (*s).data_series & CRAM_RS as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_PD as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_HC as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_QS as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_IN as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_SC as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_BS as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_DL as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_BA as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_BB as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_QQ as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series
            & (CRAM_FN as ::core::ffi::c_int
                | CRAM_FP as ::core::ffi::c_int
                | CRAM_FC as ::core::ffi::c_int
                | CRAM_DL as ::core::ffi::c_int
                | CRAM_IN as ::core::ffi::c_int
                | CRAM_SC as ::core::ffi::c_int
                | CRAM_HC as ::core::ffi::c_int
                | CRAM_PD as ::core::ffi::c_int
                | CRAM_RS as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_BF as ::core::ffi::c_int
                | CRAM_BA as ::core::ffi::c_int
                | CRAM_BS as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int
                | CRAM_AP as ::core::ffi::c_int
                | CRAM_BB as ::core::ffi::c_int
                | (CRAM_FN as ::core::ffi::c_int
                    | CRAM_FP as ::core::ffi::c_int
                    | CRAM_FC as ::core::ffi::c_int
                    | CRAM_DL as ::core::ffi::c_int
                    | CRAM_IN as ::core::ffi::c_int
                    | CRAM_SC as ::core::ffi::c_int
                    | CRAM_HC as ::core::ffi::c_int
                    | CRAM_PD as ::core::ffi::c_int
                    | CRAM_RS as ::core::ffi::c_int
                    | CRAM_RL as ::core::ffi::c_int
                    | CRAM_BF as ::core::ffi::c_int)) as ::core::ffi::c_uint
            != 0
        {
            (*s).data_series |= CRAM_RL as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_FP as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_FC as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_FC as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_FN as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_aux as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_TL as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_MF as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_CF as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_MQ as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_BF as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_BS as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= CRAM_RI as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*s).data_series
            & (CRAM_MF as ::core::ffi::c_int
                | CRAM_NS as ::core::ffi::c_int
                | CRAM_NP as ::core::ffi::c_int
                | CRAM_TS as ::core::ffi::c_int
                | CRAM_NF as ::core::ffi::c_int) as ::core::ffi::c_uint
            != 0
        {
            (*s).data_series |= CRAM_CF as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        if (*hdr).read_names_included == 0
            && (*s).data_series & CRAM_RN as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            (*s).data_series |= (CRAM_CF as ::core::ffi::c_int | CRAM_NF as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series
            & (CRAM_BA as ::core::ffi::c_int
                | CRAM_QS as ::core::ffi::c_int
                | CRAM_BB as ::core::ffi::c_int
                | CRAM_QQ as ::core::ffi::c_int) as ::core::ffi::c_uint
            != 0
        {
            (*s).data_series |= (CRAM_BF as ::core::ffi::c_int
                | CRAM_CF as ::core::ffi::c_int
                | CRAM_RL as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        if (*s).data_series & CRAM_FN as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            (*s).data_series |= (CRAM_SC as ::core::ffi::c_int
                | CRAM_IN as ::core::ffi::c_int
                | CRAM_BB as ::core::ffi::c_int)
                as ::core::ffi::c_uint;
        }
        orig_ds = (*s).data_series as uint32_t;
        i = 0 as ::core::ffi::c_int;
        while (i as usize)
            < (::core::mem::size_of::<[::core::ffi::c_int; 28]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
        {
            let mut bnum1: ::core::ffi::c_int = 0;
            let mut bnum2: ::core::ffi::c_int = 0;
            let mut j: ::core::ffi::c_int = 0;
            let mut c: *mut cram_codec = (*hdr).codecs[i_to_id[i as usize] as usize];
            if !((*s).data_series & ((1 as ::core::ffi::c_int) << i) as ::core::ffi::c_uint == 0) {
                if !c.is_null() {
                    bnum1 = cram_codec_to_id(c, &raw mut bnum2);
                    loop {
                        match bnum1 {
                            -2 => {}
                            -1 => {
                                core_used = 1 as ::core::ffi::c_int;
                            }
                            _ => {
                                j = 0 as ::core::ffi::c_int;
                                while (j as int32_t) < (*(*s).hdr).num_blocks {
                                    if (**(*s).block.offset(j as isize)).content_type
                                        as ::core::ffi::c_int
                                        == EXTERNAL as ::core::ffi::c_int
                                        && (**(*s).block.offset(j as isize)).content_id
                                            == bnum1 as int32_t
                                    {
                                        *block_used.offset(j as isize) = 1 as ::core::ffi::c_int;
                                        if cram_uncompress_block(*(*s).block.offset(j as isize))
                                            != 0
                                        {
                                            free(block_used as *mut ::core::ffi::c_void);
                                            return -(1 as ::core::ffi::c_int);
                                        }
                                    }
                                    j += 1;
                                }
                            }
                        }
                        if bnum2 == -(2 as ::core::ffi::c_int) || bnum1 == bnum2 {
                            break;
                        }
                        bnum1 = bnum2;
                    }
                }
            }
            i += 1;
        }
        if (*fd).required_fields & SAM_AUX as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            || (*s).data_series & CRAM_aux as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            i = 0 as ::core::ffi::c_int;
            while i < CRAM_MAP_HASH {
                let mut bnum1_0: ::core::ffi::c_int = 0;
                let mut bnum2_0: ::core::ffi::c_int = 0;
                let mut j_0: ::core::ffi::c_int = 0;
                let mut m: *mut cram_map = (*hdr).tag_encoding_map[i as usize];
                while !m.is_null() {
                    let mut c_0: *mut cram_codec = (*m).codec as *mut cram_codec;
                    if c_0.is_null() {
                        m = (*m).next as *mut cram_map;
                    } else {
                        bnum1_0 = cram_codec_to_id(c_0, &raw mut bnum2_0);
                        loop {
                            match bnum1_0 {
                                -2 => {}
                                -1 => {
                                    core_used = 1 as ::core::ffi::c_int;
                                }
                                _ => {
                                    j_0 = 0 as ::core::ffi::c_int;
                                    while (j_0 as int32_t) < (*(*s).hdr).num_blocks {
                                        if (**(*s).block.offset(j_0 as isize)).content_type
                                            as ::core::ffi::c_int
                                            == EXTERNAL as ::core::ffi::c_int
                                            && (**(*s).block.offset(j_0 as isize)).content_id
                                                == bnum1_0 as int32_t
                                        {
                                            *block_used.offset(j_0 as isize) =
                                                1 as ::core::ffi::c_int;
                                            if cram_uncompress_block(
                                                *(*s).block.offset(j_0 as isize),
                                            ) != 0
                                            {
                                                free(block_used as *mut ::core::ffi::c_void);
                                                return -(1 as ::core::ffi::c_int);
                                            }
                                        }
                                        j_0 += 1;
                                    }
                                }
                            }
                            if bnum2_0 == -(2 as ::core::ffi::c_int) || bnum1_0 == bnum2_0 {
                                break;
                            }
                            bnum1_0 = bnum2_0;
                        }
                        m = (*m).next as *mut cram_map;
                    }
                }
                i += 1;
            }
        }
        i = 0 as ::core::ffi::c_int;
        while (i as usize)
            < (::core::mem::size_of::<[::core::ffi::c_int; 28]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
        {
            let mut bnum1_1: ::core::ffi::c_int = 0;
            let mut bnum2_1: ::core::ffi::c_int = 0;
            let mut j_1: ::core::ffi::c_int = 0;
            let mut c_1: *mut cram_codec = (*hdr).codecs[i_to_id[i as usize] as usize];
            if !c_1.is_null() {
                bnum1_1 = cram_codec_to_id(c_1, &raw mut bnum2_1);
                loop {
                    match bnum1_1 {
                        -2 => {}
                        -1 => {
                            if core_used != 0 {
                                (*s).data_series |=
                                    ((1 as ::core::ffi::c_int) << i) as ::core::ffi::c_uint;
                            }
                        }
                        _ => {
                            j_1 = 0 as ::core::ffi::c_int;
                            while (j_1 as int32_t) < (*(*s).hdr).num_blocks {
                                if (**(*s).block.offset(j_1 as isize)).content_type
                                    as ::core::ffi::c_int
                                    == EXTERNAL as ::core::ffi::c_int
                                    && (**(*s).block.offset(j_1 as isize)).content_id
                                        == bnum1_1 as int32_t
                                {
                                    if *block_used.offset(j_1 as isize) != 0 {
                                        (*s).data_series |=
                                            ((1 as ::core::ffi::c_int) << i) as ::core::ffi::c_uint;
                                    }
                                }
                                j_1 += 1;
                            }
                        }
                    }
                    if bnum2_1 == -(2 as ::core::ffi::c_int) || bnum1_1 == bnum2_1 {
                        break;
                    }
                    bnum1_1 = bnum2_1;
                }
            }
            i += 1;
        }
        i = 0 as ::core::ffi::c_int;
        while i < CRAM_MAP_HASH {
            let mut bnum1_2: ::core::ffi::c_int = 0;
            let mut bnum2_2: ::core::ffi::c_int = 0;
            let mut j_2: ::core::ffi::c_int = 0;
            let mut m_0: *mut cram_map = (*hdr).tag_encoding_map[i as usize];
            while !m_0.is_null() {
                let mut c_2: *mut cram_codec = (*m_0).codec as *mut cram_codec;
                if c_2.is_null() {
                    m_0 = (*m_0).next as *mut cram_map;
                } else {
                    bnum1_2 = cram_codec_to_id(c_2, &raw mut bnum2_2);
                    loop {
                        match bnum1_2 {
                            -2 => {}
                            -1 => {
                                (*s).data_series |=
                                    CRAM_aux as ::core::ffi::c_int as ::core::ffi::c_uint;
                            }
                            _ => {
                                j_2 = 0 as ::core::ffi::c_int;
                                while (j_2 as int32_t) < (*(*s).hdr).num_blocks {
                                    if (**(*s).block.offset(j_2 as isize)).content_type
                                        as ::core::ffi::c_int
                                        == EXTERNAL as ::core::ffi::c_int
                                        && (**(*s).block.offset(j_2 as isize)).content_id
                                            == bnum1_2 as int32_t
                                    {
                                        if *block_used.offset(j_2 as isize) != 0 {
                                            (*s).data_series |= CRAM_aux as ::core::ffi::c_int
                                                as ::core::ffi::c_uint;
                                        }
                                    }
                                    j_2 += 1;
                                }
                            }
                        }
                        if bnum2_2 == -(2 as ::core::ffi::c_int) || bnum1_2 == bnum2_2 {
                            break;
                        }
                        bnum1_2 = bnum2_2;
                    }
                    m_0 = (*m_0).next as *mut cram_map;
                }
            }
            i += 1;
        }
        if !(orig_ds != (*s).data_series as uint32_t) {
            break;
        }
    }
    free(block_used as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
// original: cram_ds_unique (htslib/cram/cram_decode.c:876)
unsafe extern "C" fn cram_ds_unique(
    mut hdr: *mut cram_block_compression_hdr,
    mut c: *mut cram_codec,
    mut id: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut n_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut e_type: cram_encoding = E_NULL;
    i = 0 as ::core::ffi::c_int;
    while i < DS_END as ::core::ffi::c_int {
        let mut c_0: *mut cram_codec = ::core::ptr::null_mut::<cram_codec>();
        let mut bnum1: ::core::ffi::c_int = 0;
        let mut bnum2: ::core::ffi::c_int = 0;
        let mut old_n_id: ::core::ffi::c_int = 0;
        c_0 = (*hdr).codecs[i as usize] as *mut cram_codec;
        if !c_0.is_null() {
            bnum1 = cram_codec_to_id(c_0, &raw mut bnum2);
            old_n_id = n_id;
            if bnum1 == id {
                n_id += 1;
                e_type = (*c_0).codec;
            }
            if bnum2 == id {
                n_id += 1;
                e_type = (*c_0).codec;
            }
            if n_id == old_n_id + 2 as ::core::ffi::c_int {
                n_id -= 1;
            }
        }
        i += 1;
    }
    return (if n_id == 1 as ::core::ffi::c_int {
        e_type as ::core::ffi::c_uint
    } else {
        0 as ::core::ffi::c_uint
    }) as ::core::ffi::c_int;
}
#[no_mangle]
// original: cram_decode_estimate_sizes (htslib/cram/cram_decode.c:912)
pub unsafe extern "C" fn cram_decode_estimate_sizes(
    mut hdr: *mut cram_block_compression_hdr,
    mut s: *mut cram_slice,
    mut qual_size: *mut ::core::ffi::c_int,
    mut name_size: *mut ::core::ffi::c_int,
    mut q_id: *mut ::core::ffi::c_int,
) {
    let mut bnum1: ::core::ffi::c_int = 0;
    let mut bnum2: ::core::ffi::c_int = 0;
    let mut cd: *mut cram_codec = ::core::ptr::null_mut::<cram_codec>();
    *qual_size = 0 as ::core::ffi::c_int;
    *name_size = 0 as ::core::ffi::c_int;
    cd = (*hdr).codecs[DS_QS as ::core::ffi::c_int as usize] as *mut cram_codec;
    if cd.is_null() {
        return;
    }
    bnum1 = cram_codec_to_id(cd, &raw mut bnum2);
    if bnum1 < 0 as ::core::ffi::c_int && bnum2 >= 0 as ::core::ffi::c_int {
        bnum1 = bnum2;
    }
    if cram_ds_unique(hdr, cd, bnum1) != 0 {
        let mut b: *mut cram_block = cram_get_block_by_id(s, bnum1);
        if !b.is_null() {
            *qual_size = (*b).uncomp_size as ::core::ffi::c_int;
        }
        if !q_id.is_null()
            && (*cd).codec as ::core::ffi::c_uint
                == E_EXTERNAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            *q_id = bnum1;
        }
    }
    cd = (*hdr).codecs[DS_RN as ::core::ffi::c_int as usize] as *mut cram_codec;
    if cd.is_null() {
        return;
    }
    bnum1 = cram_codec_to_id(cd, &raw mut bnum2);
    if bnum1 < 0 as ::core::ffi::c_int && bnum2 >= 0 as ::core::ffi::c_int {
        bnum1 = bnum2;
    }
    if cram_ds_unique(hdr, cd, bnum1) != 0 {
        let mut b_0: *mut cram_block = cram_get_block_by_id(s, bnum1);
        if !b_0.is_null() {
            *name_size = (*b_0).uncomp_size as ::core::ffi::c_int;
        }
    }
}
#[no_mangle]
// original: cram_decode_slice_header (htslib/cram/cram_decode.c:954)
pub unsafe extern "C" fn cram_decode_slice_header(
    mut fd: *mut cram_fd,
    mut b: *mut cram_block,
) -> *mut cram_block_slice_hdr {
    let mut hdr: *mut cram_block_slice_hdr = ::core::ptr::null_mut::<cram_block_slice_hdr>();
    let mut cp: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut cp_end: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut i: ::core::ffi::c_int = 0;
    let mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*b).method as ::core::ffi::c_int != RAW as ::core::ffi::c_int {
        if cram_uncompress_block(b) < 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<cram_block_slice_hdr>();
        }
    }
    cp = (*b).data;
    cp_end = cp.offset((*b).uncomp_size as isize);
    if (*b).content_type as ::core::ffi::c_int != MAPPED_SLICE as ::core::ffi::c_int
        && (*b).content_type as ::core::ffi::c_int != UNMAPPED_SLICE as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<cram_block_slice_hdr>();
    }
    hdr = calloc(
        1 as size_t,
        ::core::mem::size_of::<cram_block_slice_hdr>() as size_t,
    ) as *mut cram_block_slice_hdr;
    if hdr.is_null() {
        return ::core::ptr::null_mut::<cram_block_slice_hdr>();
    }
    (*hdr).content_type = (*b).content_type;
    if (*b).content_type as ::core::ffi::c_int == MAPPED_SLICE as ::core::ffi::c_int {
        (*hdr).ref_seq_id = (*fd).vv.varint_get32s.expect("non-null function pointer")(
            &raw mut cp as *mut *mut ::core::ffi::c_char,
            cp_end as *mut ::core::ffi::c_char,
            &raw mut err,
        ) as int32_t;
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            (*hdr).ref_seq_start = (*fd).vv.varint_get64.expect("non-null function pointer")(
                &raw mut cp as *mut *mut ::core::ffi::c_char,
                cp_end as *mut ::core::ffi::c_char,
                &raw mut err,
            );
            (*hdr).ref_seq_span = (*fd).vv.varint_get64.expect("non-null function pointer")(
                &raw mut cp as *mut *mut ::core::ffi::c_char,
                cp_end as *mut ::core::ffi::c_char,
                &raw mut err,
            );
        } else {
            (*hdr).ref_seq_start = (*fd).vv.varint_get32.expect("non-null function pointer")(
                &raw mut cp as *mut *mut ::core::ffi::c_char,
                cp_end as *mut ::core::ffi::c_char,
                &raw mut err,
            );
            (*hdr).ref_seq_span = (*fd).vv.varint_get32.expect("non-null function pointer")(
                &raw mut cp as *mut *mut ::core::ffi::c_char,
                cp_end as *mut ::core::ffi::c_char,
                &raw mut err,
            );
        }
        if (*hdr).ref_seq_start < 0 as int64_t || (*hdr).ref_seq_span < 0 as int64_t {
            free(hdr as *mut ::core::ffi::c_void);
            hts_log(
                HTS_LOG_ERROR,
                b"cram_decode_slice_header\0" as *const u8 as *const ::core::ffi::c_char,
                b"Negative values not permitted for header sequence start or span fields\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<cram_block_slice_hdr>();
        }
    }
    (*hdr).num_records = (*fd).vv.varint_get32.expect("non-null function pointer")(
        &raw mut cp as *mut *mut ::core::ffi::c_char,
        cp_end as *mut ::core::ffi::c_char,
        &raw mut err,
    ) as int32_t;
    (*hdr).record_counter = 0 as int64_t;
    if (*fd).version >> 8 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        (*hdr).record_counter = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp as *mut *mut ::core::ffi::c_char,
            cp_end as *mut ::core::ffi::c_char,
            &raw mut err,
        );
    } else if (*fd).version >> 8 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int {
        (*hdr).record_counter = (*fd).vv.varint_get64.expect("non-null function pointer")(
            &raw mut cp as *mut *mut ::core::ffi::c_char,
            cp_end as *mut ::core::ffi::c_char,
            &raw mut err,
        );
    }
    (*hdr).num_blocks = (*fd).vv.varint_get32.expect("non-null function pointer")(
        &raw mut cp as *mut *mut ::core::ffi::c_char,
        cp_end as *mut ::core::ffi::c_char,
        &raw mut err,
    ) as int32_t;
    (*hdr).num_content_ids = (*fd).vv.varint_get32.expect("non-null function pointer")(
        &raw mut cp as *mut *mut ::core::ffi::c_char,
        cp_end as *mut ::core::ffi::c_char,
        &raw mut err,
    ) as int32_t;
    if (*hdr).num_content_ids < 1 as int32_t || (*hdr).num_content_ids >= 10000 as int32_t {
        free(hdr as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<cram_block_slice_hdr>();
    }
    (*hdr).block_content_ids = malloc(
        ((*hdr).num_content_ids as size_t)
            .wrapping_mul(::core::mem::size_of::<int32_t>() as size_t),
    ) as *mut int32_t;
    if (*hdr).block_content_ids.is_null() {
        free(hdr as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<cram_block_slice_hdr>();
    }
    i = 0 as ::core::ffi::c_int;
    while (i as int32_t) < (*hdr).num_content_ids {
        *(*hdr).block_content_ids.offset(i as isize) =
            (*fd).vv.varint_get32.expect("non-null function pointer")(
                &raw mut cp as *mut *mut ::core::ffi::c_char,
                cp_end as *mut ::core::ffi::c_char,
                &raw mut err,
            ) as int32_t;
        i += 1;
    }
    if err != 0 {
        free((*hdr).block_content_ids as *mut ::core::ffi::c_void);
        free(hdr as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<cram_block_slice_hdr>();
    }
    if (*b).content_type as ::core::ffi::c_int == MAPPED_SLICE as ::core::ffi::c_int {
        (*hdr).ref_base_id = (*fd).vv.varint_get32.expect("non-null function pointer")(
            &raw mut cp as *mut *mut ::core::ffi::c_char,
            cp_end as *mut ::core::ffi::c_char,
            &raw mut err,
        ) as int32_t;
    }
    if (*fd).version >> 8 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        if (cp_end.offset_from(cp) as ::core::ffi::c_long) < 16 as ::core::ffi::c_long {
            free((*hdr).block_content_ids as *mut ::core::ffi::c_void);
            free(hdr as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<cram_block_slice_hdr>();
        }
        memcpy(
            &raw mut (*hdr).md5 as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_void,
            cp as *const ::core::ffi::c_void,
            16 as size_t,
        );
    } else {
        memset(
            &raw mut (*hdr).md5 as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            16 as size_t,
        );
    }
    if err == 0 {
        return hdr;
    }
    free((*hdr).block_content_ids as *mut ::core::ffi::c_void);
    free(hdr as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<cram_block_slice_hdr>();
}
#[inline]
// original: add_md_char (htslib/cram/cram_decode.c:1080)
unsafe extern "C" fn add_md_char(
    mut s: *mut cram_slice,
    mut decode_md: ::core::ffi::c_int,
    mut c: ::core::ffi::c_char,
    mut md_dist: *mut int32_t,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    if decode_md != 0 {
        if block_append_uint((*s).aux_blk, *md_dist as ::core::ffi::c_uint)
            < 0 as ::core::ffi::c_int
        {
            current_block = 146926521951282760;
        } else if block_append_char((*s).aux_blk, c) < 0 as ::core::ffi::c_int {
            current_block = 146926521951282760;
        } else {
            *md_dist = 0 as ::core::ffi::c_int as int32_t;
            current_block = 4644295000439058019;
        }
        match current_block {
            4644295000439058019 => {}
            _ => return -(1 as ::core::ffi::c_int),
        }
    }
    return 0 as ::core::ffi::c_int;
}
// original: cram_decode_seq (htslib/cram/cram_decode.c:1096)
unsafe extern "C" fn cram_decode_seq(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut blk: *mut cram_block,
    mut cr: *mut cram_record,
    mut sh: *mut sam_hdr_t,
    mut cf: ::core::ffi::c_int,
    mut seq: *mut ::core::ffi::c_char,
    mut qual: *mut ::core::ffi::c_char,
    mut has_MD: ::core::ffi::c_int,
    mut has_NM: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut prev_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut f: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut out_sz: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut seq_pos: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut cig_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ref_pos: int64_t = (*cr).apos;
    let mut fn_0: int32_t = 0;
    let mut i32: int32_t = 0;
    let mut cig_op: cigar_op = BAM_CMATCH_;
    let mut cigar: *mut uint32_t = (*s).cigar;
    let mut ncigar: uint32_t = (*s).ncigar;
    let mut cigar_alloc: uint32_t = (*s).cigar_alloc;
    let mut nm: uint32_t = 0 as uint32_t;
    let mut md_dist: int32_t = 0 as int32_t;
    let mut orig_aux: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut do_md: ::core::ffi::c_int =
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            ((*s).decode_md > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            ((*s).decode_md != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        };
    let mut decode_md: ::core::ffi::c_int = (!(*s).ref_0.is_null()
        && (*cr).ref_id >= 0 as int32_t
        && (do_md != 0 && has_MD == 0 || has_MD < 0 as ::core::ffi::c_int))
        as ::core::ffi::c_int;
    let mut decode_nm: ::core::ffi::c_int = (!(*s).ref_0.is_null()
        && (*cr).ref_id >= 0 as int32_t
        && (do_md != 0 && has_NM == 0 || has_NM < 0 as ::core::ffi::c_int))
        as ::core::ffi::c_int;
    let mut ds: uint32_t = (*s).data_series as uint32_t;
    let mut bfd: *mut sam_hrecs_t = (*sh).hrecs;
    let mut codecs: *mut *mut cram_codec = &raw mut (*(*c).comp_hdr).codecs as *mut *mut cram_codec;
    if ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0
        && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
    {
        memset(
            qual as *mut ::core::ffi::c_void,
            255 as ::core::ffi::c_int,
            (*cr).len as size_t,
        );
    }
    if (*cr).cram_flags & CRAM_FLAG_NO_SEQ as int32_t != 0 {
        decode_nm = 0 as ::core::ffi::c_int;
        decode_md = decode_nm;
    }
    if decode_md != 0 {
        orig_aux = (*(*s).aux_blk).byte as ::core::ffi::c_int;
        if has_MD == 0 as ::core::ffi::c_int {
            if block_append(
                (*s).aux_blk,
                b"MDZ\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                3 as size_t,
            ) < 0 as ::core::ffi::c_int
            {
                current_block = 196986008739243050;
            } else {
                current_block = 8236137900636309791;
            }
        } else {
            current_block = 8236137900636309791;
        }
    } else {
        current_block = 8236137900636309791;
    }
    match current_block {
        8236137900636309791 => {
            if ds & CRAM_FN as ::core::ffi::c_int as uint32_t != 0 {
                if (*codecs.offset(DS_FN as ::core::ffi::c_int as isize)).is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                r |= (**codecs.offset(DS_FN as ::core::ffi::c_int as isize))
                    .decode
                    .expect("non-null function pointer")(
                    s,
                    *codecs.offset(DS_FN as ::core::ffi::c_int as isize) as *mut cram_codec,
                    blk,
                    &raw mut fn_0 as *mut ::core::ffi::c_char,
                    &raw mut out_sz,
                );
                if r != 0 {
                    return r;
                }
            } else {
                fn_0 = 0 as ::core::ffi::c_int as int32_t;
            }
            ref_pos -= 1;
            (*cr).cigar = ncigar;
            if ds & (CRAM_FC as ::core::ffi::c_int | CRAM_FP as ::core::ffi::c_int) as uint32_t == 0
            {
                current_block = 7933978793529434753;
            } else {
                if fn_0 != 0 {
                    if ds & CRAM_FC as ::core::ffi::c_int as uint32_t != 0
                        && (*codecs.offset(DS_FC as ::core::ffi::c_int as isize)).is_null()
                    {
                        return -(1 as ::core::ffi::c_int);
                    }
                    if ds & CRAM_FP as ::core::ffi::c_int as uint32_t != 0
                        && (*codecs.offset(DS_FP as ::core::ffi::c_int as isize)).is_null()
                    {
                        return -(1 as ::core::ffi::c_int);
                    }
                }
                f = 0 as ::core::ffi::c_int;
                's_145: loop {
                    if !((f as int32_t) < fn_0) {
                        current_block = 11957330206541185285;
                        break;
                    }
                    let mut pos: int32_t = 0 as int32_t;
                    let mut op: ::core::ffi::c_char = 0;
                    if ncigar.wrapping_add(2 as uint32_t) >= cigar_alloc {
                        cigar_alloc = if cigar_alloc != 0 {
                            cigar_alloc.wrapping_mul(2 as uint32_t)
                        } else {
                            1024 as uint32_t
                        };
                        cigar = realloc(
                            (*s).cigar as *mut ::core::ffi::c_void,
                            (cigar_alloc as size_t)
                                .wrapping_mul(::core::mem::size_of::<uint32_t>() as size_t),
                        ) as *mut uint32_t;
                        if cigar.is_null() {
                            return -(1 as ::core::ffi::c_int);
                        }
                        (*s).cigar = cigar;
                    }
                    if ds & CRAM_FC as ::core::ffi::c_int as uint32_t != 0 {
                        r |= (**codecs.offset(DS_FC as ::core::ffi::c_int as isize))
                            .decode
                            .expect("non-null function pointer")(
                            s,
                            *codecs.offset(DS_FC as ::core::ffi::c_int as isize) as *mut cram_codec,
                            blk,
                            &raw mut op,
                            &raw mut out_sz,
                        );
                        if r != 0 {
                            return r;
                        }
                    }
                    if !(ds & CRAM_FP as ::core::ffi::c_int as uint32_t == 0) {
                        r |= (**codecs.offset(DS_FP as ::core::ffi::c_int as isize))
                            .decode
                            .expect("non-null function pointer")(
                            s,
                            *codecs.offset(DS_FP as ::core::ffi::c_int as isize) as *mut cram_codec,
                            blk,
                            &raw mut pos as *mut ::core::ffi::c_char,
                            &raw mut out_sz,
                        );
                        if r != 0 {
                            return r;
                        }
                        pos = (pos as ::core::ffi::c_int + prev_pos) as int32_t;
                        if pos <= 0 as int32_t {
                            hts_log(
                                HTS_LOG_ERROR,
                                b"cram_decode_seq\0" as *const u8 as *const ::core::ffi::c_char,
                                b"Feature position %d before start of read\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                pos,
                            );
                            return -(1 as ::core::ffi::c_int);
                        }
                        if (*cr).len != 0 as int32_t && pos > (*cr).len {
                            let mut valid_end: int32_t = if op as ::core::ffi::c_int == 'N' as i32
                                || op as ::core::ffi::c_int == 'P' as i32
                                || op as ::core::ffi::c_int == 'H' as i32
                                || op as ::core::ffi::c_int == 'D' as i32
                            {
                                (*cr).len + 1 as int32_t
                            } else {
                                (*cr).len
                            };
                            if pos > valid_end {
                                hts_log(
                                    HTS_LOG_ERROR,
                                    b"cram_decode_seq\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"Feature position %d after end of read\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    pos,
                                );
                                return -(1 as ::core::ffi::c_int);
                            }
                        }
                        if pos > seq_pos as int32_t {
                            if !(*s).ref_0.is_null() && (*cr).ref_id >= 0 as int32_t {
                                if ref_pos + pos as int64_t - seq_pos as int64_t
                                    > (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                {
                                    static mut whinged: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    let mut rlen: ::core::ffi::c_int = 0;
                                    if whinged == 0 {
                                        hts_log(
                                            HTS_LOG_WARNING,
                                            b"cram_decode_seq\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"Ref pos outside of ref sequence boundary\0"
                                                as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                    }
                                    whinged = 1 as ::core::ffi::c_int;
                                    rlen = ((*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                        - ref_pos as hts_pos_t)
                                        as ::core::ffi::c_int;
                                    if rlen > 0 as ::core::ffi::c_int {
                                        if ref_pos + rlen as int64_t > (*s).ref_end {
                                            current_block = 15135571162060701019;
                                            break;
                                        }
                                        if (*cr).len != 0 {
                                            memcpy(
                                                seq.offset(
                                                    (seq_pos - 1 as ::core::ffi::c_int) as isize,
                                                )
                                                    as *mut ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_void,
                                                (*s).ref_0.offset(
                                                    (ref_pos - (*s).ref_start as int64_t
                                                        + 1 as int64_t)
                                                        as isize,
                                                )
                                                    as *mut ::core::ffi::c_char
                                                    as *const ::core::ffi::c_void,
                                                rlen as size_t,
                                            );
                                            if pos - seq_pos as int32_t - rlen as int32_t
                                                > 0 as int32_t
                                            {
                                                memset(
                                                    seq.offset(
                                                        (seq_pos - 1 as ::core::ffi::c_int + rlen)
                                                            as isize,
                                                    )
                                                        as *mut ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_void,
                                                    'N' as i32,
                                                    (pos - seq_pos as int32_t - rlen as int32_t)
                                                        as size_t,
                                                );
                                            }
                                        }
                                    } else if (*cr).len != 0 {
                                        memset(
                                            seq.offset((seq_pos - 1 as ::core::ffi::c_int) as isize)
                                                as *mut ::core::ffi::c_char
                                                as *mut ::core::ffi::c_void,
                                            'N' as i32,
                                            ((*cr).len - seq_pos as int32_t + 1 as int32_t)
                                                as size_t,
                                        );
                                    }
                                    if md_dist >= 0 as int32_t {
                                        md_dist = (md_dist as ::core::ffi::c_int
                                            + (pos - seq_pos as int32_t) as ::core::ffi::c_int)
                                            as int32_t;
                                    }
                                } else {
                                    if ref_pos + pos as int64_t - seq_pos as int64_t > (*s).ref_end
                                    {
                                        current_block = 15135571162060701019;
                                        break;
                                    }
                                    let mut refp: *const ::core::ffi::c_char = (*s)
                                        .ref_0
                                        .offset(ref_pos as isize)
                                        .offset(-((*s).ref_start as isize))
                                        .offset(1 as ::core::ffi::c_int as isize);
                                    let frag_len: ::core::ffi::c_int =
                                        pos as ::core::ffi::c_int - seq_pos;
                                    if decode_md != 0 || decode_nm != 0 {
                                        let mut N: *mut ::core::ffi::c_char = memchr(
                                            refp as *const ::core::ffi::c_void,
                                            'N' as i32,
                                            frag_len as size_t,
                                        )
                                            as *mut ::core::ffi::c_char;
                                        if !N.is_null() {
                                            let mut i: ::core::ffi::c_int = 0;
                                            i = 0 as ::core::ffi::c_int;
                                            while i < frag_len {
                                                let mut base: ::core::ffi::c_char =
                                                    *refp.offset(i as isize);
                                                if base as ::core::ffi::c_int == 'N' as i32 {
                                                    if add_md_char(
                                                        s,
                                                        decode_md,
                                                        'N' as i32 as ::core::ffi::c_char,
                                                        &raw mut md_dist,
                                                    ) < 0 as ::core::ffi::c_int
                                                    {
                                                        return -(1 as ::core::ffi::c_int);
                                                    }
                                                    nm = nm.wrapping_add(1);
                                                } else {
                                                    md_dist += 1;
                                                }
                                                i += 1;
                                            }
                                        } else {
                                            md_dist = (md_dist as ::core::ffi::c_int + frag_len)
                                                as int32_t;
                                        }
                                    }
                                    if (*cr).len != 0 {
                                        memcpy(
                                            seq.offset((seq_pos - 1 as ::core::ffi::c_int) as isize)
                                                as *mut ::core::ffi::c_char
                                                as *mut ::core::ffi::c_void,
                                            refp as *const ::core::ffi::c_void,
                                            frag_len as size_t,
                                        );
                                    }
                                }
                            }
                            if cig_len != 0
                                && cig_op as ::core::ffi::c_uint
                                    != BAM_CMATCH as ::core::ffi::c_uint
                            {
                                let fresh131 = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(fresh131 as isize) =
                                    ((cig_len << 4 as ::core::ffi::c_int) as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                cig_len = 0 as ::core::ffi::c_int;
                            }
                            cig_op = BAM_CMATCH_;
                            cig_len += (pos - seq_pos as int32_t) as ::core::ffi::c_int;
                            ref_pos = (ref_pos as ::core::ffi::c_long
                                + (pos - seq_pos as int32_t) as ::core::ffi::c_long)
                                as int64_t;
                            seq_pos = pos as ::core::ffi::c_int;
                        }
                        prev_pos = pos as ::core::ffi::c_int;
                        if ds & CRAM_FC as ::core::ffi::c_int as uint32_t == 0 {
                            current_block = 7933978793529434753;
                            break;
                        }
                        match op as ::core::ffi::c_int {
                            83 => {
                                let mut out_sz2: int32_t = if (*cr).len != 0 {
                                    (*cr).len - (pos - 1 as int32_t)
                                } else {
                                    1 as int32_t
                                };
                                let mut have_sc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                if cig_len != 0 {
                                    let fresh132 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh132 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                match (*fd).version >> 8 as ::core::ffi::c_int {
                                    1 => {
                                        if ds & CRAM_IN as ::core::ffi::c_int as uint32_t != 0 {
                                            if !(*codecs
                                                .offset(DS_IN as ::core::ffi::c_int as isize))
                                            .is_null()
                                            {
                                                r |= (**codecs
                                                    .offset(DS_IN as ::core::ffi::c_int as isize))
                                                .decode
                                                .expect("non-null function pointer")(
                                                    s,
                                                    *codecs.offset(
                                                        DS_IN as ::core::ffi::c_int as isize,
                                                    )
                                                        as *mut cram_codec,
                                                    blk,
                                                    if (*cr).len != 0 {
                                                        seq.offset((pos - 1 as int32_t) as isize)
                                                            as *mut ::core::ffi::c_char
                                                    } else {
                                                        ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                        )
                                                    },
                                                    &raw mut out_sz2,
                                                );
                                            } else {
                                                if (*cr).len != 0 {
                                                    *seq.offset((pos - 1 as int32_t) as isize) =
                                                        'N' as i32 as ::core::ffi::c_char;
                                                }
                                                out_sz2 = 1 as ::core::ffi::c_int as int32_t;
                                            }
                                            have_sc = 1 as ::core::ffi::c_int;
                                        }
                                    }
                                    2 | _ => {
                                        if ds & CRAM_SC as ::core::ffi::c_int as uint32_t != 0 {
                                            if !(*codecs
                                                .offset(DS_SC as ::core::ffi::c_int as isize))
                                            .is_null()
                                            {
                                                r |= (**codecs
                                                    .offset(DS_SC as ::core::ffi::c_int as isize))
                                                .decode
                                                .expect("non-null function pointer")(
                                                    s,
                                                    *codecs.offset(
                                                        DS_SC as ::core::ffi::c_int as isize,
                                                    )
                                                        as *mut cram_codec,
                                                    blk,
                                                    if (*cr).len != 0 {
                                                        seq.offset((pos - 1 as int32_t) as isize)
                                                            as *mut ::core::ffi::c_char
                                                    } else {
                                                        ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                        )
                                                    },
                                                    &raw mut out_sz2,
                                                );
                                            } else {
                                                if (*cr).len != 0 {
                                                    *seq.offset((pos - 1 as int32_t) as isize) =
                                                        'N' as i32 as ::core::ffi::c_char;
                                                }
                                                out_sz2 = 1 as ::core::ffi::c_int as int32_t;
                                            }
                                            have_sc = 1 as ::core::ffi::c_int;
                                        }
                                    }
                                }
                                if have_sc != 0 {
                                    if r != 0 {
                                        return r;
                                    }
                                    let fresh133 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh133 as isize) = ((out_sz2
                                        << 4 as ::core::ffi::c_int)
                                        + BAM_CSOFT_CLIP as int32_t)
                                        as uint32_t;
                                    cig_op = BAM_CSOFT_CLIP_;
                                    seq_pos += out_sz2 as ::core::ffi::c_int;
                                }
                            }
                            88 => {
                                let mut base_0: ::core::ffi::c_uchar = 0;
                                let mut ref_base: ::core::ffi::c_int = 0;
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CMATCH as ::core::ffi::c_uint
                                {
                                    let fresh134 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh134 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_BS as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_BS as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_BS as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_BS as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        &raw mut base_0 as *mut ::core::ffi::c_char,
                                        &raw mut out_sz,
                                    );
                                    if r != 0 {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    if (*cr).ref_id < 0 as int32_t
                                        || ref_pos
                                            >= (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                        || (*s).ref_0.is_null()
                                    {
                                        if (pos - 1 as int32_t) < (*cr).len {
                                            *seq.offset((pos - 1 as int32_t) as isize) =
                                                (*(*c).comp_hdr).substitution_matrix
                                                    [(*fd).L1['N' as i32 as usize] as usize]
                                                    [base_0 as usize];
                                        }
                                        if decode_md != 0 || decode_nm != 0 {
                                            if md_dist >= 0 as int32_t && decode_md != 0 {
                                                if block_append_uint(
                                                    (*s).aux_blk,
                                                    md_dist as ::core::ffi::c_uint,
                                                ) < 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 196986008739243050;
                                                    break;
                                                }
                                            }
                                            md_dist = -(1 as ::core::ffi::c_int) as int32_t;
                                            nm = nm.wrapping_sub(1);
                                        }
                                    } else {
                                        let mut ref_call: ::core::ffi::c_uchar = (if ref_pos
                                            < (*s).ref_end
                                        {
                                            *(*s).ref_0.offset(
                                                (ref_pos - (*s).ref_start as int64_t + 1 as int64_t)
                                                    as isize,
                                            ) as uc
                                                as ::core::ffi::c_int
                                        } else {
                                            'N' as i32
                                        })
                                            as ::core::ffi::c_uchar;
                                        ref_base =
                                            (*fd).L1[ref_call as usize] as ::core::ffi::c_int;
                                        if (pos - 1 as int32_t) < (*cr).len {
                                            *seq.offset((pos - 1 as int32_t) as isize) =
                                                (*(*c).comp_hdr).substitution_matrix
                                                    [ref_base as usize]
                                                    [base_0 as usize];
                                        }
                                        if add_md_char(
                                            s,
                                            decode_md,
                                            ref_call as ::core::ffi::c_char,
                                            &raw mut md_dist,
                                        ) < 0 as ::core::ffi::c_int
                                        {
                                            return -(1 as ::core::ffi::c_int);
                                        }
                                    }
                                }
                                cig_op = BAM_CMATCH_;
                                nm = nm.wrapping_add(1);
                                cig_len += 1;
                                seq_pos += 1;
                                ref_pos += 1;
                            }
                            68 => {
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CDEL as ::core::ffi::c_uint
                                {
                                    let fresh135 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh135 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_DL as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_DL as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_DL as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_DL as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        &raw mut i32 as *mut ::core::ffi::c_char,
                                        &raw mut out_sz,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                    if i32 < 0 as int32_t {
                                        current_block = 15135571162060701019;
                                        break;
                                    }
                                    if decode_md != 0 || decode_nm != 0 {
                                        if ref_pos + i32 as int64_t > (*s).ref_end {
                                            current_block = 15135571162060701019;
                                            break;
                                        }
                                        if md_dist >= 0 as int32_t && decode_md != 0 {
                                            if block_append_uint(
                                                (*s).aux_blk,
                                                md_dist as ::core::ffi::c_uint,
                                            ) < 0 as ::core::ffi::c_int
                                            {
                                                current_block = 196986008739243050;
                                                break;
                                            }
                                        }
                                        if ref_pos + i32 as int64_t
                                            <= (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                        {
                                            if decode_md != 0 {
                                                if block_append_char(
                                                    (*s).aux_blk,
                                                    '^' as i32 as ::core::ffi::c_char,
                                                ) < 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 196986008739243050;
                                                    break;
                                                }
                                                if block_append(
                                                    (*s).aux_blk,
                                                    (*s).ref_0.offset(
                                                        (ref_pos - (*s).ref_start as int64_t
                                                            + 1 as int64_t)
                                                            as isize,
                                                    )
                                                        as *mut ::core::ffi::c_char
                                                        as *const ::core::ffi::c_void,
                                                    i32 as size_t,
                                                ) < 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 196986008739243050;
                                                    break;
                                                }
                                                md_dist = 0 as ::core::ffi::c_int as int32_t;
                                            }
                                            nm = (nm as ::core::ffi::c_uint)
                                                .wrapping_add(i32 as ::core::ffi::c_uint)
                                                as uint32_t
                                                as uint32_t;
                                        } else {
                                            let mut dlen: uint32_t = 0;
                                            if (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                                >= ref_pos
                                            {
                                                if decode_md != 0 {
                                                    if block_append_char(
                                                        (*s).aux_blk,
                                                        '^' as i32 as ::core::ffi::c_char,
                                                    ) < 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 196986008739243050;
                                                        break;
                                                    }
                                                    if block_append(
                                                        (*s).aux_blk,
                                                        (*s).ref_0.offset(
                                                            (ref_pos - (*s).ref_start as int64_t
                                                                + 1 as int64_t)
                                                                as isize,
                                                        )
                                                            as *mut ::core::ffi::c_char
                                                            as *const ::core::ffi::c_void,
                                                        ((*(*bfd)
                                                            .ref_0
                                                            .offset((*cr).ref_id as isize))
                                                        .len - ref_pos as hts_pos_t)
                                                            as size_t,
                                                    ) < 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 196986008739243050;
                                                        break;
                                                    }
                                                    if block_append_uint(
                                                        (*s).aux_blk,
                                                        0 as ::core::ffi::c_uint,
                                                    ) < 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 196986008739243050;
                                                        break;
                                                    }
                                                }
                                                dlen = (i32 as hts_pos_t
                                                    - ((*(*bfd)
                                                        .ref_0
                                                        .offset((*cr).ref_id as isize))
                                                    .len - ref_pos as hts_pos_t))
                                                    as uint32_t;
                                                nm = (nm as ::core::ffi::c_uint).wrapping_add(
                                                    (i32 as uint32_t).wrapping_sub(dlen)
                                                        as ::core::ffi::c_uint,
                                                )
                                                    as uint32_t
                                                    as uint32_t;
                                            } else {
                                                dlen = i32 as uint32_t;
                                            }
                                            md_dist = -(1 as ::core::ffi::c_int) as int32_t;
                                        }
                                    }
                                    cig_op = BAM_CDEL_;
                                    cig_len += i32 as ::core::ffi::c_int;
                                    ref_pos = (ref_pos as ::core::ffi::c_long
                                        + i32 as ::core::ffi::c_long)
                                        as int64_t;
                                }
                            }
                            73 => {
                                let mut out_sz2_0: int32_t = if (*cr).len != 0 {
                                    (*cr).len - (pos - 1 as int32_t)
                                } else {
                                    1 as int32_t
                                };
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CINS as ::core::ffi::c_uint
                                {
                                    let fresh136 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh136 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_IN as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_IN as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_IN as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_IN as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            seq.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut out_sz2_0,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                    cig_op = BAM_CINS_;
                                    cig_len += out_sz2_0 as ::core::ffi::c_int;
                                    seq_pos += out_sz2_0 as ::core::ffi::c_int;
                                    nm = (nm as ::core::ffi::c_uint)
                                        .wrapping_add(out_sz2_0 as ::core::ffi::c_uint)
                                        as uint32_t
                                        as uint32_t;
                                }
                            }
                            105 => {
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CINS as ::core::ffi::c_uint
                                {
                                    let fresh137 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh137 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_BA as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_BA as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_BA as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_BA as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            seq.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut out_sz,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                }
                                cig_op = BAM_CINS_;
                                cig_len += 1;
                                seq_pos += 1;
                                nm = nm.wrapping_add(1);
                            }
                            98 => {
                                let mut len: int32_t = if (*cr).len != 0 {
                                    (*cr).len - (pos - 1 as int32_t)
                                } else {
                                    1 as int32_t
                                };
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CMATCH as ::core::ffi::c_uint
                                {
                                    let fresh138 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh138 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_BB as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_BB as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_BB as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_BB as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            seq.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut len,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                    if decode_md != 0 || decode_nm != 0 {
                                        let mut x: ::core::ffi::c_int = 0;
                                        if md_dist >= 0 as int32_t && decode_md != 0 {
                                            if block_append_uint(
                                                (*s).aux_blk,
                                                md_dist as ::core::ffi::c_uint,
                                            ) < 0 as ::core::ffi::c_int
                                            {
                                                current_block = 196986008739243050;
                                                break;
                                            }
                                        }
                                        x = 0 as ::core::ffi::c_int;
                                        while (x as int32_t) < len {
                                            if x != 0 && decode_md != 0 {
                                                if block_append_uint(
                                                    (*s).aux_blk,
                                                    0 as ::core::ffi::c_uint,
                                                ) < 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 196986008739243050;
                                                    break 's_145;
                                                }
                                            }
                                            if ref_pos + x as int64_t
                                                >= (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                                || (*s).ref_0.is_null()
                                            {
                                                md_dist = -(1 as ::core::ffi::c_int) as int32_t;
                                                break;
                                            } else {
                                                if decode_md != 0 {
                                                    if ref_pos + x as int64_t >= (*s).ref_end {
                                                        current_block = 15135571162060701019;
                                                        break 's_145;
                                                    }
                                                    let mut r_0: ::core::ffi::c_char =
                                                        *(*s).ref_0.offset(
                                                            (ref_pos + x as int64_t
                                                                - (*s).ref_start as int64_t
                                                                + 1 as int64_t)
                                                                as isize,
                                                        );
                                                    if block_append_char((*s).aux_blk, r_0)
                                                        < 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 196986008739243050;
                                                        break 's_145;
                                                    }
                                                }
                                                x += 1;
                                            }
                                        }
                                        nm = (nm as ::core::ffi::c_uint)
                                            .wrapping_add(x as ::core::ffi::c_uint)
                                            as uint32_t
                                            as uint32_t;
                                        md_dist = 0 as ::core::ffi::c_int as int32_t;
                                    }
                                }
                                cig_op = BAM_CMATCH_;
                                cig_len += len as ::core::ffi::c_int;
                                seq_pos += len as ::core::ffi::c_int;
                                ref_pos = (ref_pos as ::core::ffi::c_long
                                    + len as ::core::ffi::c_long)
                                    as int64_t;
                            }
                            113 => {
                                let mut len_0: int32_t = if (*cr).len != 0 {
                                    (*cr).len - (pos - 1 as int32_t)
                                } else {
                                    1 as int32_t
                                };
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CMATCH as ::core::ffi::c_uint
                                {
                                    let fresh139 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh139 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_QQ as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_QQ as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    if ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0
                                        && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                        && (*cr).len > 0 as int32_t
                                        && *qual as ::core::ffi::c_uchar as ::core::ffi::c_int
                                            == 255 as ::core::ffi::c_int
                                    {
                                        memset(
                                            qual as *mut ::core::ffi::c_void,
                                            30 as ::core::ffi::c_int,
                                            (*cr).len as size_t,
                                        );
                                    }
                                    r |= (**codecs.offset(DS_QQ as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_QQ as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            qual.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut len_0,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                }
                                cig_op = BAM_CMATCH_;
                            }
                            66 => {
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CMATCH as ::core::ffi::c_uint
                                {
                                    let fresh140 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh140 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_BA as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_BA as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_BA as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_BA as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            seq.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut out_sz,
                                    );
                                    if decode_md != 0 || decode_nm != 0 {
                                        if md_dist >= 0 as int32_t && decode_md != 0 {
                                            if block_append_uint(
                                                (*s).aux_blk,
                                                md_dist as ::core::ffi::c_uint,
                                            ) < 0 as ::core::ffi::c_int
                                            {
                                                current_block = 196986008739243050;
                                                break;
                                            }
                                        }
                                        if ref_pos
                                            >= (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                            || (*s).ref_0.is_null()
                                        {
                                            md_dist = -(1 as ::core::ffi::c_int) as int32_t;
                                        } else {
                                            if decode_md != 0 {
                                                if ref_pos >= (*s).ref_end {
                                                    current_block = 15135571162060701019;
                                                    break;
                                                }
                                                if block_append_char(
                                                    (*s).aux_blk,
                                                    *(*s).ref_0.offset(
                                                        (ref_pos - (*s).ref_start as int64_t
                                                            + 1 as int64_t)
                                                            as isize,
                                                    ),
                                                ) < 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 196986008739243050;
                                                    break;
                                                }
                                            }
                                            nm = nm.wrapping_add(1);
                                            md_dist = 0 as ::core::ffi::c_int as int32_t;
                                        }
                                    }
                                }
                                if ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_QS as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    if cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                        && (*cr).len > 0 as int32_t
                                        && *qual as ::core::ffi::c_uchar as ::core::ffi::c_int
                                            == 255 as ::core::ffi::c_int
                                    {
                                        memset(
                                            qual as *mut ::core::ffi::c_void,
                                            30 as ::core::ffi::c_int,
                                            (*cr).len as size_t,
                                        );
                                    }
                                    r |= (**codecs.offset(DS_QS as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_QS as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            qual.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut out_sz,
                                    );
                                }
                                cig_op = BAM_CMATCH_;
                                cig_len += 1;
                                seq_pos += 1;
                                ref_pos += 1;
                            }
                            81 => {
                                if ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_QS as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    if cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                        && (*cr).len > 0 as int32_t
                                        && *qual as ::core::ffi::c_uchar as ::core::ffi::c_int
                                            == 255 as ::core::ffi::c_int
                                    {
                                        memset(
                                            qual as *mut ::core::ffi::c_void,
                                            30 as ::core::ffi::c_int,
                                            (*cr).len as size_t,
                                        );
                                    }
                                    r |= (**codecs.offset(DS_QS as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_QS as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        if (*cr).len != 0 {
                                            qual.offset((pos - 1 as int32_t) as isize)
                                                as *mut ::core::ffi::c_char
                                        } else {
                                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                                        },
                                        &raw mut out_sz,
                                    );
                                }
                            }
                            72 => {
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CHARD_CLIP as ::core::ffi::c_uint
                                {
                                    let fresh141 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh141 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_HC as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_HC as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_HC as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_HC as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        &raw mut i32 as *mut ::core::ffi::c_char,
                                        &raw mut out_sz,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                    if i32 < 0 as int32_t {
                                        current_block = 15135571162060701019;
                                        break;
                                    }
                                    cig_op = BAM_CHARD_CLIP_;
                                    cig_len += i32 as ::core::ffi::c_int;
                                }
                            }
                            80 => {
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CPAD as ::core::ffi::c_uint
                                {
                                    let fresh142 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh142 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_PD as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_PD as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_PD as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_PD as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        &raw mut i32 as *mut ::core::ffi::c_char,
                                        &raw mut out_sz,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                    if i32 < 0 as int32_t {
                                        current_block = 15135571162060701019;
                                        break;
                                    }
                                    cig_op = BAM_CPAD_;
                                    cig_len += i32 as ::core::ffi::c_int;
                                }
                            }
                            78 => {
                                if cig_len != 0
                                    && cig_op as ::core::ffi::c_uint
                                        != BAM_CREF_SKIP as ::core::ffi::c_uint
                                {
                                    let fresh143 = ncigar;
                                    ncigar = ncigar.wrapping_add(1);
                                    *cigar.offset(fresh143 as isize) = ((cig_len
                                        << 4 as ::core::ffi::c_int)
                                        as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                                    cig_len = 0 as ::core::ffi::c_int;
                                }
                                if ds & CRAM_RS as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*codecs.offset(DS_RS as ::core::ffi::c_int as isize))
                                        .is_null()
                                    {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    r |= (**codecs.offset(DS_RS as ::core::ffi::c_int as isize))
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        *codecs.offset(DS_RS as ::core::ffi::c_int as isize)
                                            as *mut cram_codec,
                                        blk,
                                        &raw mut i32 as *mut ::core::ffi::c_char,
                                        &raw mut out_sz,
                                    );
                                    if r != 0 {
                                        return r;
                                    }
                                    if i32 < 0 as int32_t {
                                        current_block = 15135571162060701019;
                                        break;
                                    }
                                    cig_op = BAM_CREF_SKIP_;
                                    cig_len += i32 as ::core::ffi::c_int;
                                    ref_pos = (ref_pos as ::core::ffi::c_long
                                        + i32 as ::core::ffi::c_long)
                                        as int64_t;
                                }
                            }
                            _ => {
                                hts_log(
                                    HTS_LOG_ERROR,
                                    b"cram_decode_seq\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"Unknown feature code '%c'\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    op as ::core::ffi::c_int,
                                );
                                return -(1 as ::core::ffi::c_int);
                            }
                        }
                    }
                    f += 1;
                }
                match current_block {
                    7933978793529434753 => {}
                    196986008739243050 => {}
                    _ => {
                        match current_block {
                            11957330206541185285 => {
                                if ds & CRAM_FC as ::core::ffi::c_int as uint32_t == 0 {
                                    current_block = 7933978793529434753;
                                } else if ds & CRAM_FN as ::core::ffi::c_int as uint32_t != 0
                                    && (*cr).len >= seq_pos as int32_t
                                {
                                    if !(*s).ref_0.is_null() && (*cr).ref_id >= 0 as int32_t {
                                        if ref_pos + (*cr).len as int64_t - seq_pos as int64_t
                                            + 1 as int64_t
                                            > (*(*bfd).ref_0.offset((*cr).ref_id as isize)).len
                                        {
                                            static mut whinged_0: ::core::ffi::c_int =
                                                0 as ::core::ffi::c_int;
                                            let mut rlen_0: ::core::ffi::c_int = 0;
                                            if whinged_0 == 0 {
                                                hts_log(
                                                    HTS_LOG_WARNING,
                                                    b"cram_decode_seq\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"Ref pos outside of ref sequence boundary\0"
                                                        as *const u8
                                                        as *const ::core::ffi::c_char,
                                                );
                                            }
                                            whinged_0 = 1 as ::core::ffi::c_int;
                                            rlen_0 = ((*(*bfd).ref_0.offset((*cr).ref_id as isize))
                                                .len
                                                - ref_pos as hts_pos_t)
                                                as ::core::ffi::c_int;
                                            if rlen_0 > 0 as ::core::ffi::c_int {
                                                if ref_pos + rlen_0 as int64_t > (*s).ref_end {
                                                    current_block = 15135571162060701019;
                                                } else {
                                                    if (seq_pos as int32_t - 1 as int32_t
                                                        + rlen_0 as int32_t)
                                                        < (*cr).len
                                                    {
                                                        memcpy(
                                                            seq.offset(
                                                                (seq_pos - 1 as ::core::ffi::c_int)
                                                                    as isize,
                                                            )
                                                                as *mut ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_void,
                                                            (*s).ref_0.offset(
                                                                (ref_pos
                                                                    - (*s).ref_start as int64_t
                                                                    + 1 as int64_t)
                                                                    as isize,
                                                            )
                                                                as *mut ::core::ffi::c_char
                                                                as *const ::core::ffi::c_void,
                                                            rlen_0 as size_t,
                                                        );
                                                    }
                                                    if (*cr).len - seq_pos as int32_t + 1 as int32_t
                                                        - rlen_0 as int32_t
                                                        > 0 as int32_t
                                                    {
                                                        memset(
                                                            seq.offset(
                                                                (seq_pos - 1 as ::core::ffi::c_int
                                                                    + rlen_0)
                                                                    as isize,
                                                            )
                                                                as *mut ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_void,
                                                            'N' as i32,
                                                            ((*cr).len - seq_pos as int32_t
                                                                + 1 as int32_t
                                                                - rlen_0 as int32_t)
                                                                as size_t,
                                                        );
                                                    }
                                                    current_block = 13438929862036009938;
                                                }
                                            } else {
                                                if (*cr).len - seq_pos as int32_t + 1 as int32_t
                                                    > 0 as int32_t
                                                {
                                                    memset(
                                                        seq.offset(
                                                            (seq_pos - 1 as ::core::ffi::c_int)
                                                                as isize,
                                                        )
                                                            as *mut ::core::ffi::c_char
                                                            as *mut ::core::ffi::c_void,
                                                        'N' as i32,
                                                        ((*cr).len - seq_pos as int32_t
                                                            + 1 as int32_t)
                                                            as size_t,
                                                    );
                                                }
                                                current_block = 13438929862036009938;
                                            }
                                            match current_block {
                                                15135571162060701019 => {}
                                                _ => {
                                                    if md_dist >= 0 as int32_t {
                                                        md_dist = (md_dist as ::core::ffi::c_int
                                                            + ((*cr).len - seq_pos as int32_t
                                                                + 1 as int32_t)
                                                                as ::core::ffi::c_int)
                                                            as int32_t;
                                                    }
                                                    current_block = 12926621472508904600;
                                                }
                                            }
                                        } else {
                                            if (*cr).len - seq_pos as int32_t + 1 as int32_t
                                                > 0 as int32_t
                                            {
                                                if ref_pos + (*cr).len as int64_t
                                                    - seq_pos as int64_t
                                                    + 1 as int64_t
                                                    > (*s).ref_end
                                                {
                                                    current_block = 15135571162060701019;
                                                } else {
                                                    let mut remainder: ::core::ffi::c_int =
                                                        (*cr).len as ::core::ffi::c_int
                                                            - (seq_pos - 1 as ::core::ffi::c_int);
                                                    let mut j: ::core::ffi::c_int = (ref_pos
                                                        - (*s).ref_start as int64_t
                                                        + 1 as int64_t)
                                                        as ::core::ffi::c_int;
                                                    if decode_md != 0 || decode_nm != 0 {
                                                        let mut i_0: ::core::ffi::c_int = 0;
                                                        let mut N_0: *mut ::core::ffi::c_char =
                                                            memchr(
                                                                (*s).ref_0.offset(j as isize)
                                                                    as *mut ::core::ffi::c_char
                                                                    as *const ::core::ffi::c_void,
                                                                'N' as i32,
                                                                remainder as size_t,
                                                            )
                                                                as *mut ::core::ffi::c_char;
                                                        if N_0.is_null() {
                                                            md_dist = (md_dist
                                                                as ::core::ffi::c_int
                                                                + ((*cr).len
                                                                    - (seq_pos as int32_t
                                                                        - 1 as int32_t))
                                                                    as ::core::ffi::c_int)
                                                                as int32_t;
                                                        } else {
                                                            let mut refp_0: *mut ::core::ffi::c_char = (*s)
                                                                .ref_0
                                                                .offset((j - (seq_pos - 1 as ::core::ffi::c_int)) as isize)
                                                                as *mut ::core::ffi::c_char;
                                                            md_dist = (md_dist
                                                                as ::core::ffi::c_long
                                                                + N_0.offset_from(
                                                                    (*s).ref_0.offset(j as isize)
                                                                        as *mut ::core::ffi::c_char,
                                                                )
                                                                    as ::core::ffi::c_long)
                                                                as int32_t;
                                                            let mut i_start: ::core::ffi::c_int = ((seq_pos
                                                                - 1 as ::core::ffi::c_int) as ::core::ffi::c_long
                                                                + N_0
                                                                    .offset_from(
                                                                        (*s).ref_0.offset(j as isize) as *mut ::core::ffi::c_char,
                                                                    ) as ::core::ffi::c_long) as ::core::ffi::c_int;
                                                            i_0 = i_start;
                                                            while (i_0 as int32_t) < (*cr).len {
                                                                let mut base_1: ::core::ffi::c_char = *refp_0
                                                                    .offset(i_0 as isize);
                                                                if base_1 as ::core::ffi::c_int
                                                                    == 'N' as i32
                                                                {
                                                                    if add_md_char(
                                                                        s,
                                                                        decode_md,
                                                                        'N' as i32
                                                                            as ::core::ffi::c_char,
                                                                        &raw mut md_dist,
                                                                    ) < 0 as ::core::ffi::c_int
                                                                    {
                                                                        return -(1
                                                                            as ::core::ffi::c_int);
                                                                    }
                                                                    nm = nm.wrapping_add(1);
                                                                } else {
                                                                    md_dist += 1;
                                                                }
                                                                i_0 += 1;
                                                            }
                                                        }
                                                    }
                                                    memcpy(
                                                        seq.offset(
                                                            (seq_pos - 1 as ::core::ffi::c_int)
                                                                as isize,
                                                        )
                                                            as *mut ::core::ffi::c_char
                                                            as *mut ::core::ffi::c_void,
                                                        (*s).ref_0.offset(j as isize)
                                                            as *mut ::core::ffi::c_char
                                                            as *const ::core::ffi::c_void,
                                                        remainder as size_t,
                                                    );
                                                    current_block = 6848047883821882781;
                                                }
                                            } else {
                                                current_block = 6848047883821882781;
                                            }
                                            match current_block {
                                                15135571162060701019 => {}
                                                _ => {
                                                    ref_pos = (ref_pos as ::core::ffi::c_long
                                                        + ((*cr).len - seq_pos as int32_t
                                                            + 1 as int32_t)
                                                            as ::core::ffi::c_long)
                                                        as int64_t;
                                                    current_block = 12926621472508904600;
                                                }
                                            }
                                        }
                                    } else {
                                        if (*cr).ref_id >= 0 as int32_t {
                                            ref_pos = (ref_pos as ::core::ffi::c_long
                                                + ((*cr).len - seq_pos as int32_t + 1 as int32_t)
                                                    as ::core::ffi::c_long)
                                                as int64_t;
                                        }
                                        current_block = 12926621472508904600;
                                    }
                                    match current_block {
                                        15135571162060701019 => {}
                                        _ => {
                                            if ncigar.wrapping_add(1 as uint32_t) >= cigar_alloc {
                                                cigar_alloc = if cigar_alloc != 0 {
                                                    cigar_alloc.wrapping_mul(2 as uint32_t)
                                                } else {
                                                    1024 as uint32_t
                                                };
                                                cigar = realloc(
                                                    (*s).cigar as *mut ::core::ffi::c_void,
                                                    (cigar_alloc as size_t).wrapping_mul(
                                                        ::core::mem::size_of::<uint32_t>()
                                                            as size_t,
                                                    ),
                                                )
                                                    as *mut uint32_t;
                                                if cigar.is_null() {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                                (*s).cigar = cigar;
                                            }
                                            if cig_len != 0
                                                && cig_op as ::core::ffi::c_uint
                                                    != BAM_CMATCH as ::core::ffi::c_uint
                                            {
                                                let fresh144 = ncigar;
                                                ncigar = ncigar.wrapping_add(1);
                                                *cigar.offset(fresh144 as isize) = ((cig_len
                                                    << 4 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_uint)
                                                    .wrapping_add(cig_op as ::core::ffi::c_uint)
                                                    as uint32_t;
                                                cig_len = 0 as ::core::ffi::c_int;
                                            }
                                            cig_op = BAM_CMATCH_;
                                            cig_len += ((*cr).len - seq_pos as int32_t
                                                + 1 as int32_t)
                                                as ::core::ffi::c_int;
                                            current_block = 7933978793529434753;
                                        }
                                    }
                                } else {
                                    current_block = 7933978793529434753;
                                }
                            }
                            _ => {}
                        }
                        match current_block {
                            7933978793529434753 => {}
                            _ => {
                                hts_log(
                                    HTS_LOG_ERROR,
                                    b"cram_decode_seq\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"CRAM CIGAR extends beyond slice reference extents\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                return -(1 as ::core::ffi::c_int);
                            }
                        }
                    }
                }
            }
            match current_block {
                196986008739243050 => {}
                _ => {
                    if ds & CRAM_FN as ::core::ffi::c_int as uint32_t != 0 && decode_md != 0 {
                        if md_dist >= 0 as int32_t {
                            if block_append_uint((*s).aux_blk, md_dist as ::core::ffi::c_uint)
                                < 0 as ::core::ffi::c_int
                            {
                                current_block = 196986008739243050;
                            } else {
                                current_block = 15752106442776732052;
                            }
                        } else {
                            current_block = 15752106442776732052;
                        }
                    } else {
                        current_block = 15752106442776732052;
                    }
                    match current_block {
                        196986008739243050 => {}
                        _ => {
                            if cig_len != 0 {
                                if ncigar >= cigar_alloc {
                                    cigar_alloc = if cigar_alloc != 0 {
                                        cigar_alloc.wrapping_mul(2 as uint32_t)
                                    } else {
                                        1024 as uint32_t
                                    };
                                    cigar = realloc(
                                        (*s).cigar as *mut ::core::ffi::c_void,
                                        (cigar_alloc as size_t).wrapping_mul(
                                            ::core::mem::size_of::<uint32_t>() as size_t,
                                        ),
                                    ) as *mut uint32_t;
                                    if cigar.is_null() {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    (*s).cigar = cigar;
                                }
                                let fresh145 = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(fresh145 as isize) =
                                    ((cig_len << 4 as ::core::ffi::c_int) as ::core::ffi::c_uint)
                                        .wrapping_add(cig_op as ::core::ffi::c_uint)
                                        as uint32_t;
                            }
                            (*cr).ncigar = ncigar.wrapping_sub((*cr).cigar) as int32_t;
                            (*cr).aend = if ref_pos > (*cr).apos {
                                ref_pos
                            } else {
                                (*cr).apos
                            };
                            if ds & CRAM_MQ as ::core::ffi::c_int as uint32_t != 0 {
                                if (*codecs.offset(DS_MQ as ::core::ffi::c_int as isize)).is_null()
                                {
                                    return -(1 as ::core::ffi::c_int);
                                }
                                r |= (**codecs.offset(DS_MQ as ::core::ffi::c_int as isize))
                                    .decode
                                    .expect("non-null function pointer")(
                                    s,
                                    *codecs.offset(DS_MQ as ::core::ffi::c_int as isize)
                                        as *mut cram_codec,
                                    blk,
                                    &raw mut (*cr).mqual as *mut ::core::ffi::c_char,
                                    &raw mut out_sz,
                                );
                            } else {
                                (*cr).mqual = 40 as ::core::ffi::c_int as int32_t;
                            }
                            if ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0
                                && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES != 0
                            {
                                let mut out_sz2_1: int32_t = (*cr).len;
                                if (*codecs.offset(DS_QS as ::core::ffi::c_int as isize)).is_null()
                                {
                                    return -(1 as ::core::ffi::c_int);
                                }
                                r |= (**codecs.offset(DS_QS as ::core::ffi::c_int as isize))
                                    .decode
                                    .expect("non-null function pointer")(
                                    s,
                                    *codecs.offset(DS_QS as ::core::ffi::c_int as isize)
                                        as *mut cram_codec,
                                    blk,
                                    qual,
                                    &raw mut out_sz2_1,
                                );
                            }
                            (*s).cigar = cigar;
                            (*s).cigar_alloc = cigar_alloc;
                            (*s).ncigar = ncigar;
                            if (*cr).cram_flags & CRAM_FLAG_NO_SEQ as int32_t != 0 {
                                (*cr).len = 0 as ::core::ffi::c_int as int32_t;
                            }
                            if decode_md != 0 {
                                if block_append_char(
                                    (*s).aux_blk,
                                    '\0' as i32 as ::core::ffi::c_char,
                                ) < 0 as ::core::ffi::c_int
                                {
                                    current_block = 196986008739243050;
                                } else {
                                    let mut sz: size_t =
                                        (*(*s).aux_blk).byte.wrapping_sub(orig_aux as size_t);
                                    if has_MD < 0 as ::core::ffi::c_int {
                                        let mut tmp_MD_: [::core::ffi::c_char; 1024] = [0; 1024];
                                        let mut tmp_MD: *mut ::core::ffi::c_char =
                                            &raw mut tmp_MD_ as *mut ::core::ffi::c_char;
                                        let mut orig_aux_p: *mut ::core::ffi::c_uchar =
                                            (*(*s).aux_blk).data.offset(orig_aux as isize);
                                        if sz > 1024 as size_t {
                                            tmp_MD = malloc(sz) as *mut ::core::ffi::c_char;
                                            if tmp_MD.is_null() {
                                                return -(1 as ::core::ffi::c_int);
                                            }
                                        }
                                        memcpy(
                                            tmp_MD as *mut ::core::ffi::c_void,
                                            orig_aux_p as *const ::core::ffi::c_void,
                                            sz,
                                        );
                                        memmove(
                                            ((*(*s).aux_blk).data.offset(-has_MD as isize)
                                                as *mut ::core::ffi::c_uchar)
                                                .offset(sz as isize)
                                                as *mut ::core::ffi::c_void,
                                            (*(*s).aux_blk).data.offset(-has_MD as isize)
                                                as *mut ::core::ffi::c_uchar
                                                as *const ::core::ffi::c_void,
                                            orig_aux_p.offset_from(
                                                (*(*s).aux_blk).data.offset(-has_MD as isize)
                                                    as *mut ::core::ffi::c_uchar,
                                            )
                                                as ::core::ffi::c_long
                                                as size_t,
                                        );
                                        memcpy(
                                            (*(*s).aux_blk).data.offset(-has_MD as isize)
                                                as *mut ::core::ffi::c_uchar
                                                as *mut ::core::ffi::c_void,
                                            tmp_MD as *const ::core::ffi::c_void,
                                            sz,
                                        );
                                        if tmp_MD != &raw mut tmp_MD_ as *mut ::core::ffi::c_char {
                                            free(tmp_MD as *mut ::core::ffi::c_void);
                                        }
                                        if -has_NM > -has_MD {
                                            has_NM = (has_NM as ::core::ffi::c_ulong)
                                                .wrapping_sub(sz as ::core::ffi::c_ulong)
                                                as ::core::ffi::c_int
                                                as ::core::ffi::c_int;
                                        }
                                    }
                                    (*cr).aux_size = ((*cr).aux_size as ::core::ffi::c_ulong)
                                        .wrapping_add(sz as ::core::ffi::c_ulong)
                                        as uint32_t
                                        as uint32_t;
                                    current_block = 15667605415110443435;
                                }
                            } else {
                                current_block = 15667605415110443435;
                            }
                            match current_block {
                                196986008739243050 => {}
                                _ => {
                                    if decode_nm != 0 {
                                        if has_NM == 0 as ::core::ffi::c_int {
                                            let mut buf: [::core::ffi::c_char; 7] = [0; 7];
                                            let mut buf_size: size_t = 0;
                                            buf[0 as ::core::ffi::c_int as usize] =
                                                'N' as i32 as ::core::ffi::c_char;
                                            buf[1 as ::core::ffi::c_int as usize] =
                                                'M' as i32 as ::core::ffi::c_char;
                                            if nm <= UINT8_MAX as uint32_t {
                                                buf_size = 4 as size_t;
                                                buf[2 as ::core::ffi::c_int as usize] =
                                                    'C' as i32 as ::core::ffi::c_char;
                                                buf[3 as ::core::ffi::c_int as usize] = (nm
                                                    >> 0 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                            } else if nm <= UINT16_MAX as uint32_t {
                                                buf_size = 5 as size_t;
                                                buf[2 as ::core::ffi::c_int as usize] =
                                                    'S' as i32 as ::core::ffi::c_char;
                                                buf[3 as ::core::ffi::c_int as usize] = (nm
                                                    >> 0 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                                buf[4 as ::core::ffi::c_int as usize] = (nm
                                                    >> 8 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                            } else {
                                                buf_size = 7 as size_t;
                                                buf[2 as ::core::ffi::c_int as usize] =
                                                    'I' as i32 as ::core::ffi::c_char;
                                                buf[3 as ::core::ffi::c_int as usize] = (nm
                                                    >> 0 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                                buf[4 as ::core::ffi::c_int as usize] = (nm
                                                    >> 8 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                                buf[5 as ::core::ffi::c_int as usize] = (nm
                                                    >> 16 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                                buf[6 as ::core::ffi::c_int as usize] = (nm
                                                    >> 24 as ::core::ffi::c_int
                                                    & 0xff as uint32_t)
                                                    as ::core::ffi::c_char;
                                            }
                                            if block_append(
                                                (*s).aux_blk,
                                                &raw mut buf as *mut ::core::ffi::c_char
                                                    as *const ::core::ffi::c_void,
                                                buf_size,
                                            ) < 0 as ::core::ffi::c_int
                                            {
                                                current_block = 196986008739243050;
                                            } else {
                                                (*cr).aux_size = ((*cr).aux_size
                                                    as ::core::ffi::c_ulong)
                                                    .wrapping_add(buf_size as ::core::ffi::c_ulong)
                                                    as uint32_t
                                                    as uint32_t;
                                                current_block = 8155699268269024736;
                                            }
                                        } else {
                                            let mut buf_0: *mut ::core::ffi::c_uchar =
                                                (*(*s).aux_blk).data.offset(-has_NM as isize);
                                            *buf_0.offset(0 as ::core::ffi::c_int as isize) =
                                                (nm >> 0 as ::core::ffi::c_int & 0xff as uint32_t)
                                                    as ::core::ffi::c_uchar;
                                            *buf_0.offset(1 as ::core::ffi::c_int as isize) =
                                                (nm >> 8 as ::core::ffi::c_int & 0xff as uint32_t)
                                                    as ::core::ffi::c_uchar;
                                            *buf_0.offset(2 as ::core::ffi::c_int as isize) =
                                                (nm >> 16 as ::core::ffi::c_int & 0xff as uint32_t)
                                                    as ::core::ffi::c_uchar;
                                            *buf_0.offset(3 as ::core::ffi::c_int as isize) =
                                                (nm >> 24 as ::core::ffi::c_int & 0xff as uint32_t)
                                                    as ::core::ffi::c_uchar;
                                            current_block = 8155699268269024736;
                                        }
                                    } else {
                                        current_block = 8155699268269024736;
                                    }
                                    match current_block {
                                        196986008739243050 => {}
                                        _ => return r,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    return -(1 as ::core::ffi::c_int);
}
// original: map_find (htslib/cram/cram_decode.c:1926)
unsafe extern "C" fn map_find(
    mut map: *mut *mut cram_map,
    mut key: *mut ::core::ffi::c_uchar,
    mut id: ::core::ffi::c_int,
) -> *mut cram_map {
    let mut m: *mut cram_map = ::core::ptr::null_mut::<cram_map>();
    m = *map.offset(
        (*key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            * 3 as ::core::ffi::c_int
            + *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & CRAM_MAP_HASH - 1 as ::core::ffi::c_int) as isize,
    );
    while !m.is_null() && (*m).key != id {
        m = (*m).next as *mut cram_map;
    }
    return m;
}
// original: cram_decode_aux_1_0 (htslib/cram/cram_decode.c:1939)
unsafe extern "C" fn cram_decode_aux_1_0(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut blk: *mut cram_block,
    mut cr: *mut cram_record,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut i: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut out_sz: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut ntags: ::core::ffi::c_uchar = 0;
    if (*(*c).comp_hdr).codecs[DS_TC as ::core::ffi::c_int as usize].is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    r |= (*(*(*c).comp_hdr).codecs[DS_TC as ::core::ffi::c_int as usize])
        .decode
        .expect("non-null function pointer")(
        s,
        (*(*c).comp_hdr).codecs[DS_TC as ::core::ffi::c_int as usize],
        blk,
        &raw mut ntags as *mut ::core::ffi::c_char,
        &raw mut out_sz,
    );
    (*cr).ntags = ntags as int32_t;
    (*cr).aux_size = 0 as uint32_t;
    (*cr).aux = (*(*s).aux_blk).byte as uint32_t;
    i = 0 as ::core::ffi::c_int;
    loop {
        if !((i as int32_t) < (*cr).ntags) {
            current_block = 224731115979188411;
            break;
        }
        let mut id: int32_t = 0;
        let mut out_sz_0: int32_t = 1 as int32_t;
        let mut tag_data: [::core::ffi::c_uchar; 3] = [0; 3];
        let mut m: *mut cram_map = ::core::ptr::null_mut::<cram_map>();
        if (*(*c).comp_hdr).codecs[DS_TN as ::core::ffi::c_int as usize].is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        r |= (*(*(*c).comp_hdr).codecs[DS_TN as ::core::ffi::c_int as usize])
            .decode
            .expect("non-null function pointer")(
            s,
            (*(*c).comp_hdr).codecs[DS_TN as ::core::ffi::c_int as usize],
            blk,
            &raw mut id as *mut ::core::ffi::c_char,
            &raw mut out_sz_0,
        );
        if out_sz_0 == 3 as int32_t {
            memcpy(
                &raw mut tag_data as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_void,
                &raw mut id as *const ::core::ffi::c_void,
                3 as size_t,
            );
        } else {
            tag_data[0 as ::core::ffi::c_int as usize] =
                (id >> 16 as ::core::ffi::c_int & 0xff as int32_t) as ::core::ffi::c_uchar;
            tag_data[1 as ::core::ffi::c_int as usize] =
                (id >> 8 as ::core::ffi::c_int & 0xff as int32_t) as ::core::ffi::c_uchar;
            tag_data[2 as ::core::ffi::c_int as usize] =
                (id & 0xff as int32_t) as ::core::ffi::c_uchar;
        }
        m = map_find(
            &raw mut (*(*c).comp_hdr).tag_encoding_map as *mut *mut cram_map,
            &raw mut tag_data as *mut ::core::ffi::c_uchar,
            id as ::core::ffi::c_int,
        );
        if m.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if block_append(
            (*s).aux_blk,
            &raw mut tag_data as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_char
                as *const ::core::ffi::c_void,
            3 as size_t,
        ) < 0 as ::core::ffi::c_int
        {
            current_block = 10918223632682351237;
            break;
        }
        if (*m).codec.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        r |= (*(*m).codec).decode.expect("non-null function pointer")(
            s,
            (*m).codec,
            blk,
            (*s).aux_blk as *mut ::core::ffi::c_char,
            &raw mut out_sz_0,
        );
        (*cr).aux_size = ((*cr).aux_size as ::core::ffi::c_uint)
            .wrapping_add((out_sz_0 + 3 as int32_t) as ::core::ffi::c_uint)
            as uint32_t as uint32_t;
        i += 1;
    }
    match current_block {
        224731115979188411 => return r,
        _ => return -(1 as ::core::ffi::c_int),
    };
}
#[inline]
// original: aux_ele_size (htslib/cram/cram_decode.c:1989)
unsafe extern "C" fn aux_ele_size(mut type_0: uint8_t) -> ::core::ffi::c_int {
    match type_0 as ::core::ffi::c_int {
        65 | 99 | 67 => return 1 as ::core::ffi::c_int,
        115 | 83 => return 2 as ::core::ffi::c_int,
        105 | 73 | 102 => return 4 as ::core::ffi::c_int,
        100 => return 8 as ::core::ffi::c_int,
        _ => return 1 as ::core::ffi::c_int,
    };
}
// original: cram_decode_aux (htslib/cram/cram_decode.c:2008)
unsafe extern "C" fn cram_decode_aux(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut blk: *mut cram_block,
    mut cr: *mut cram_record,
    mut has_MD: *mut ::core::ffi::c_int,
    mut has_NM: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut i: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut out_sz: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut TL: int32_t = 0 as int32_t;
    let mut TN: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut ds: uint32_t = (*s).data_series as uint32_t;
    if ds & (CRAM_TL as ::core::ffi::c_int | CRAM_aux as ::core::ffi::c_int) as uint32_t == 0 {
        (*cr).aux = 0 as uint32_t;
        (*cr).aux_size = 0 as uint32_t;
        return 0 as ::core::ffi::c_int;
    }
    if (*(*c).comp_hdr).codecs[DS_TL as ::core::ffi::c_int as usize].is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    r |= (*(*(*c).comp_hdr).codecs[DS_TL as ::core::ffi::c_int as usize])
        .decode
        .expect("non-null function pointer")(
        s,
        (*(*c).comp_hdr).codecs[DS_TL as ::core::ffi::c_int as usize],
        blk,
        &raw mut TL as *mut ::core::ffi::c_char,
        &raw mut out_sz,
    );
    if r != 0 || TL < 0 as int32_t || TL >= (*(*c).comp_hdr).nTL as int32_t {
        return -(1 as ::core::ffi::c_int);
    }
    TN = *(*(*c).comp_hdr).TL.offset(TL as isize);
    (*cr).ntags = strlen(TN as *mut ::core::ffi::c_char).wrapping_div(3 as size_t) as int32_t;
    (*cr).aux_size = 0 as uint32_t;
    (*cr).aux = (*(*s).aux_blk).byte as uint32_t;
    if ds & CRAM_aux as ::core::ffi::c_int as uint32_t == 0 {
        return 0 as ::core::ffi::c_int;
    }
    i = 0 as ::core::ffi::c_int;
    loop {
        if !((i as int32_t) < (*cr).ntags) {
            current_block = 6528285054092551010;
            break;
        }
        let mut id: int32_t = 0;
        let mut out_sz_0: int32_t = 1 as int32_t;
        let mut tag_data: [::core::ffi::c_uchar; 7] = [0; 7];
        let mut m: *mut cram_map = ::core::ptr::null_mut::<cram_map>();
        if *TN.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32
            && *TN.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'D' as i32
            && !has_MD.is_null()
        {
            *has_MD = (*(*s).aux_blk).byte.wrapping_add(3 as size_t).wrapping_mul(
                (if *TN.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '*' as i32
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    1 as ::core::ffi::c_int
                }) as size_t,
            ) as ::core::ffi::c_int;
        }
        if *TN.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
            && *TN.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32
            && !has_NM.is_null()
        {
            *has_NM = (*(*s).aux_blk).byte.wrapping_add(3 as size_t).wrapping_mul(
                (if *TN.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '*' as i32
                {
                    -(1 as ::core::ffi::c_int)
                } else {
                    1 as ::core::ffi::c_int
                }) as size_t,
            ) as ::core::ffi::c_int;
        }
        tag_data[0 as ::core::ffi::c_int as usize] = *TN.offset(0 as ::core::ffi::c_int as isize);
        tag_data[1 as ::core::ffi::c_int as usize] = *TN.offset(1 as ::core::ffi::c_int as isize);
        tag_data[2 as ::core::ffi::c_int as usize] = *TN.offset(2 as ::core::ffi::c_int as isize);
        id = ((tag_data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | (tag_data[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
            | tag_data[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            as int32_t;
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int
            && *TN.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '*' as i32
        {
            let mut tag_data_size: ::core::ffi::c_int = 0;
            if *TN.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
                && *TN.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32
            {
                memcpy(
                    (&raw mut tag_data as *mut ::core::ffi::c_uchar)
                        .offset(2 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_uchar
                        as *mut ::core::ffi::c_void,
                    b"I\0\0\0\0\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    5 as size_t,
                );
                tag_data_size = 7 as ::core::ffi::c_int;
                current_block = 2604890879466389055;
            } else if *TN.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as i32
                && *TN.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'G' as i32
            {
                TN = TN.offset(3 as ::core::ffi::c_int as isize);
                let mut rg: *const ::core::ffi::c_char = sam_hdr_line_name(
                    (*fd).header,
                    b"RG\0" as *const u8 as *const ::core::ffi::c_char,
                    (*cr).rg as ::core::ffi::c_int,
                );
                if rg.is_null() {
                    current_block = 1856101646708284338;
                } else {
                    let mut rg_len: size_t = strlen(rg);
                    tag_data[2 as ::core::ffi::c_int as usize] = 'Z' as i32 as ::core::ffi::c_uchar;
                    if block_append(
                        (*s).aux_blk,
                        &raw mut tag_data as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        3 as size_t,
                    ) < 0 as ::core::ffi::c_int
                    {
                        current_block = 9499344950036252570;
                        break;
                    }
                    if block_append((*s).aux_blk, rg as *const ::core::ffi::c_void, rg_len)
                        < 0 as ::core::ffi::c_int
                    {
                        current_block = 9499344950036252570;
                        break;
                    }
                    if block_append_char((*s).aux_blk, '\0' as i32 as ::core::ffi::c_char)
                        < 0 as ::core::ffi::c_int
                    {
                        current_block = 9499344950036252570;
                        break;
                    }
                    (*cr).aux_size = ((*cr).aux_size as ::core::ffi::c_ulong)
                        .wrapping_add((3 as size_t).wrapping_add(rg_len).wrapping_add(1 as size_t)
                            as ::core::ffi::c_ulong)
                        as uint32_t as uint32_t;
                    (*cr).rg = -(1 as ::core::ffi::c_int) as int32_t;
                    current_block = 1856101646708284338;
                }
            } else {
                tag_data[2 as ::core::ffi::c_int as usize] = 'Z' as i32 as ::core::ffi::c_uchar;
                tag_data_size = 3 as ::core::ffi::c_int;
                current_block = 2604890879466389055;
            }
            match current_block {
                1856101646708284338 => {}
                _ => {
                    if block_append(
                        (*s).aux_blk,
                        &raw mut tag_data as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        tag_data_size as size_t,
                    ) < 0 as ::core::ffi::c_int
                    {
                        current_block = 9499344950036252570;
                        break;
                    }
                    (*cr).aux_size = ((*cr).aux_size as ::core::ffi::c_uint)
                        .wrapping_add(tag_data_size as ::core::ffi::c_uint)
                        as uint32_t as uint32_t;
                    TN = TN.offset(3 as ::core::ffi::c_int as isize);
                    current_block = 9353995356876505083;
                }
            }
        } else {
            TN = TN.offset(3 as ::core::ffi::c_int as isize);
            m = map_find(
                &raw mut (*(*c).comp_hdr).tag_encoding_map as *mut *mut cram_map,
                &raw mut tag_data as *mut ::core::ffi::c_uchar,
                id as ::core::ffi::c_int,
            );
            if m.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            if block_append(
                (*s).aux_blk,
                &raw mut tag_data as *mut ::core::ffi::c_uchar as *mut ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                3 as size_t,
            ) < 0 as ::core::ffi::c_int
            {
                current_block = 9499344950036252570;
                break;
            }
            if (*m).codec.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            if (*(*m).codec).codec as ::core::ffi::c_uint
                == E_BYTE_ARRAY_LEN as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*(*m).codec).codec as ::core::ffi::c_uint
                    == E_BYTE_ARRAY_STOP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                out_sz_0 = (out_sz_0 as ::core::ffi::c_int
                    * aux_ele_size(*TN.offset(-(1 as ::core::ffi::c_int) as isize) as uint8_t))
                    as int32_t;
            }
            r |= (*(*m).codec).decode.expect("non-null function pointer")(
                s,
                (*m).codec,
                blk,
                (*s).aux_blk as *mut ::core::ffi::c_char,
                &raw mut out_sz_0,
            );
            if r != 0 {
                current_block = 6528285054092551010;
                break;
            }
            (*cr).aux_size = ((*cr).aux_size as ::core::ffi::c_uint)
                .wrapping_add((out_sz_0 + 3 as int32_t) as ::core::ffi::c_uint)
                as uint32_t as uint32_t;
            if *TN.offset(-(3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == 'c' as i32
                && *TN.offset(-(2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    == 'F' as i32
                && *TN.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    == 'C' as i32
                && out_sz_0 == 1 as int32_t
            {
                let mut cF: uint8_t = *((*(*s).aux_blk).data.offset((*(*s).aux_blk).byte as isize)
                    as *mut ::core::ffi::c_uchar)
                    .offset(-(1 as ::core::ffi::c_int) as isize)
                    as uint8_t;
                (*(*s).aux_blk).byte = ((*(*s).aux_blk).byte as ::core::ffi::c_ulong)
                    .wrapping_sub((out_sz_0 + 3 as int32_t) as ::core::ffi::c_ulong)
                    as size_t as size_t;
                (*cr).aux_size = ((*cr).aux_size as ::core::ffi::c_uint)
                    .wrapping_sub((out_sz_0 + 3 as int32_t) as ::core::ffi::c_uint)
                    as uint32_t as uint32_t;
                if cF as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
                    && !has_MD.is_null()
                    && *has_MD == 0 as ::core::ffi::c_int
                {
                    *has_MD = 1 as ::core::ffi::c_int;
                }
                if cF as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0
                    && !has_NM.is_null()
                    && *has_NM == 0 as ::core::ffi::c_int
                {
                    *has_NM = 1 as ::core::ffi::c_int;
                }
            }
            current_block = 9353995356876505083;
        }
        match current_block {
            9353995356876505083 => {
                if (*(*s).aux_blk).byte
                    > ((1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int) as size_t
                {
                    hts_log(
                        HTS_LOG_ERROR,
                        b"cram_decode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                        b"CRAM->BAM aux block size overflow\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    current_block = 9499344950036252570;
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    match current_block {
        6528285054092551010 => return r,
        _ => return -(1 as ::core::ffi::c_int),
    };
}
// original: cram_decode_slice_xref (htslib/cram/cram_decode.c:2140)
unsafe extern "C" fn cram_decode_slice_xref(
    mut s: *mut cram_slice,
    mut required_fields: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rec: ::core::ffi::c_int = 0;
    if required_fields
        & (SAM_RNEXT as ::core::ffi::c_int
            | SAM_PNEXT as ::core::ffi::c_int
            | SAM_TLEN as ::core::ffi::c_int)
        == 0
    {
        rec = 0 as ::core::ffi::c_int;
        while (rec as int32_t) < (*(*s).hdr).num_records {
            let mut cr: *mut cram_record = (*s).crecs.offset(rec as isize) as *mut cram_record;
            (*cr).tlen = 0 as int64_t;
            (*cr).mate_pos = 0 as int64_t;
            (*cr).mate_ref_id = -(1 as ::core::ffi::c_int) as int32_t;
            rec += 1;
        }
        return 0 as ::core::ffi::c_int;
    }
    rec = 0 as ::core::ffi::c_int;
    while (rec as int32_t) < (*(*s).hdr).num_records {
        let mut cr_0: *mut cram_record = (*s).crecs.offset(rec as isize) as *mut cram_record;
        if (*cr_0).mate_line >= 0 as int32_t {
            if (*cr_0).mate_line < (*(*s).hdr).num_records {
                if (*cr_0).tlen == INT64_MIN as int64_t {
                    let mut id1: ::core::ffi::c_int = rec;
                    let mut id2: ::core::ffi::c_int = rec;
                    let mut aleft: int64_t = (*cr_0).apos;
                    let mut aright: int64_t = (*cr_0).aend;
                    let mut tlen: int64_t = 0;
                    let mut ref_0: ::core::ffi::c_int = (*cr_0).ref_id as ::core::ffi::c_int;
                    let mut left_cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut right_cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    loop {
                        if aleft > (*(*s).crecs.offset(id2 as isize)).apos {
                            aleft = (*(*s).crecs.offset(id2 as isize)).apos;
                            left_cnt = 1 as ::core::ffi::c_int;
                        } else if aleft == (*(*s).crecs.offset(id2 as isize)).apos {
                            left_cnt += 1;
                        }
                        if aright < (*(*s).crecs.offset(id2 as isize)).aend {
                            aright = (*(*s).crecs.offset(id2 as isize)).aend;
                            right_cnt = 1 as ::core::ffi::c_int;
                        } else if aright == (*(*s).crecs.offset(id2 as isize)).aend {
                            right_cnt += 1;
                        }
                        if (*(*s).crecs.offset(id2 as isize)).mate_line == -(1 as int32_t) {
                            (*(*s).crecs.offset(id2 as isize)).mate_line = rec as int32_t;
                            break;
                        } else {
                            if (*(*s).crecs.offset(id2 as isize)).mate_line <= id2 as int32_t
                                || (*(*s).crecs.offset(id2 as isize)).mate_line
                                    >= (*(*s).hdr).num_records
                            {
                                return -(1 as ::core::ffi::c_int);
                            }
                            id2 =
                                (*(*s).crecs.offset(id2 as isize)).mate_line as ::core::ffi::c_int;
                            if (*(*s).crecs.offset(id2 as isize)).ref_id != ref_0 as int32_t {
                                ref_0 = -(1 as ::core::ffi::c_int);
                            }
                            if !(id2 != id1) {
                                break;
                            }
                        }
                    }
                    if ref_0 != -(1 as ::core::ffi::c_int) {
                        tlen = aright - aleft + 1 as int64_t;
                        id2 = rec;
                        id1 = id2;
                        if (*(*s).crecs.offset(id2 as isize)).apos == aleft
                            && ((*(*s).crecs.offset(id2 as isize)).aend < aright
                                || left_cnt <= 1 as ::core::ffi::c_int)
                        {
                            (*(*s).crecs.offset(id2 as isize)).tlen = tlen;
                            tlen = -tlen;
                        } else if (*(*s).crecs.offset(id2 as isize)).apos == aleft
                            && (*(*s).crecs.offset(id2 as isize)).aend == aright
                            && left_cnt > 1 as ::core::ffi::c_int
                            && right_cnt > 1 as ::core::ffi::c_int
                        {
                            if (*(*s).crecs.offset(id2 as isize)).flags & BAM_FREAD1 as int32_t != 0
                            {
                                (*(*s).crecs.offset(id2 as isize)).tlen = tlen;
                                tlen = -tlen;
                            } else {
                                (*(*s).crecs.offset(id2 as isize)).tlen = -tlen;
                            }
                        } else {
                            (*(*s).crecs.offset(id2 as isize)).tlen = -tlen;
                        }
                        id2 = (*(*s).crecs.offset(id2 as isize)).mate_line as ::core::ffi::c_int;
                        while id2 != id1 {
                            (*(*s).crecs.offset(id2 as isize)).tlen = tlen;
                            id2 =
                                (*(*s).crecs.offset(id2 as isize)).mate_line as ::core::ffi::c_int;
                        }
                    } else {
                        id2 = rec;
                        id1 = id2;
                        (*(*s).crecs.offset(id2 as isize)).tlen = 0 as int64_t;
                        id2 = (*(*s).crecs.offset(id2 as isize)).mate_line as ::core::ffi::c_int;
                        while id2 != id1 {
                            (*(*s).crecs.offset(id2 as isize)).tlen = 0 as int64_t;
                            id2 =
                                (*(*s).crecs.offset(id2 as isize)).mate_line as ::core::ffi::c_int;
                        }
                    }
                }
                (*cr_0).mate_pos = (*(*s).crecs.offset((*cr_0).mate_line as isize)).apos;
                (*cr_0).mate_ref_id = (*(*s).crecs.offset((*cr_0).mate_line as isize)).ref_id;
                (*cr_0).flags = ((*cr_0).flags as ::core::ffi::c_int | BAM_FPAIRED) as int32_t;
                if (*(*s).crecs.offset((*cr_0).mate_line as isize)).flags & BAM_FUNMAP as int32_t
                    != 0
                {
                    (*cr_0).flags = ((*cr_0).flags as ::core::ffi::c_int | BAM_FMUNMAP) as int32_t;
                    (*cr_0).tlen = 0 as int64_t;
                }
                if (*cr_0).flags & BAM_FUNMAP as int32_t != 0 {
                    (*cr_0).tlen = 0 as int64_t;
                }
                if (*(*s).crecs.offset((*cr_0).mate_line as isize)).flags & BAM_FREVERSE as int32_t
                    != 0
                {
                    (*cr_0).flags =
                        ((*cr_0).flags as ::core::ffi::c_int | BAM_FMREVERSE) as int32_t;
                }
            } else {
                hts_log(
                    HTS_LOG_ERROR,
                    b"cram_decode_slice_xref\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Mate line out of bounds: %d vs [0, %d]\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*cr_0).mate_line,
                    (*(*s).hdr).num_records - 1 as int32_t,
                );
            }
        } else {
            if (*cr_0).mate_flags & CRAM_M_REVERSE as int32_t != 0 {
                (*cr_0).flags = ((*cr_0).flags as ::core::ffi::c_int
                    | (BAM_FPAIRED | BAM_FMREVERSE)) as int32_t;
            }
            if (*cr_0).mate_flags & CRAM_M_UNMAP as int32_t != 0 {
                (*cr_0).flags = ((*cr_0).flags as ::core::ffi::c_int | BAM_FMUNMAP) as int32_t;
            }
            if (*cr_0).flags & BAM_FPAIRED as int32_t == 0 {
                (*cr_0).mate_ref_id = -(1 as ::core::ffi::c_int) as int32_t;
            }
        }
        if (*cr_0).tlen == INT64_MIN as int64_t {
            (*cr_0).tlen = 0 as int64_t;
        }
        rec += 1;
    }
    rec = 0 as ::core::ffi::c_int;
    while (rec as int32_t) < (*(*s).hdr).num_records {
        let mut cr_1: *mut cram_record = (*s).crecs.offset(rec as isize) as *mut cram_record;
        if (*cr_1).explicit_tlen != INT64_MIN as int64_t {
            (*cr_1).tlen = (*cr_1).explicit_tlen;
        }
        rec += 1;
    }
    return 0 as ::core::ffi::c_int;
}
// original: md5_print (htslib/cram/cram_decode.c:2305)
unsafe extern "C" fn md5_print(
    mut md5: *mut ::core::ffi::c_uchar,
    mut out: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        *out.offset((i * 2 as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as isize) =
            ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"0123456789abcdef\0")
                [(*md5.offset(i as isize) as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)
                    as usize];
        *out.offset((i * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
            ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"0123456789abcdef\0")
                [(*md5.offset(i as isize) as ::core::ffi::c_int & 15 as ::core::ffi::c_int)
                    as usize];
        i += 1;
    }
    *out.offset(32 as ::core::ffi::c_int as isize) = 0 as ::core::ffi::c_char;
    return out;
}
// original: cram_decode_tlen (htslib/cram/cram_decode.c:2322)
unsafe extern "C" fn cram_decode_tlen(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut blk: *mut cram_block,
    mut tlen: *mut int64_t,
) -> ::core::ffi::c_int {
    let mut out_sz: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*(*c).comp_hdr).codecs[DS_TS as ::core::ffi::c_int as usize].is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if ((*fd).version >> 8 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
        let mut i32: int32_t = 0;
        r |= (*(*(*c).comp_hdr).codecs[DS_TS as ::core::ffi::c_int as usize])
            .decode
            .expect("non-null function pointer")(
            s,
            (*(*c).comp_hdr).codecs[DS_TS as ::core::ffi::c_int as usize],
            blk,
            &raw mut i32 as *mut ::core::ffi::c_char,
            &raw mut out_sz,
        );
        *tlen = i32 as int64_t;
    } else {
        r |= (*(*(*c).comp_hdr).codecs[DS_TS as ::core::ffi::c_int as usize])
            .decode
            .expect("non-null function pointer")(
            s,
            (*(*c).comp_hdr).codecs[DS_TS as ::core::ffi::c_int as usize],
            blk,
            tlen as *mut ::core::ffi::c_char,
            &raw mut out_sz,
        );
    }
    return r;
}
#[no_mangle]
// original: cram_decode_slice (htslib/cram/cram_decode.c:2346)
pub unsafe extern "C" fn cram_decode_slice(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut sh: *mut sam_hdr_t,
) -> ::core::ffi::c_int {
    let mut last_ref_id: ::core::ffi::c_int = 0;
    let mut current_block: u64;
    let mut blk: *mut cram_block = *(*s).block.offset(0 as ::core::ffi::c_int as isize);
    let mut bf: int32_t = 0;
    let mut ref_id: int32_t = 0;
    let mut cf: ::core::ffi::c_uchar = 0;
    let mut out_sz: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut rec: ::core::ffi::c_int = 0;
    let mut seq: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut qual: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut unknown_rg: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut embed_ref: ::core::ffi::c_int = 0;
    let mut refs: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut ds: uint32_t = 0;
    let mut bfd: *mut sam_hrecs_t = (*sh).hrecs;
    if cram_dependent_data_series(fd, (*c).comp_hdr, s) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    ds = (*s).data_series as uint32_t;
    (*blk).bit = 7 as ::core::ffi::c_int;
    let mut qsize: ::core::ffi::c_int = 0;
    let mut nsize: ::core::ffi::c_int = 0;
    let mut q_id: ::core::ffi::c_int = 0;
    cram_decode_estimate_sizes(
        (*c).comp_hdr,
        s,
        &raw mut qsize,
        &raw mut nsize,
        &raw mut q_id,
    );
    if qsize != 0 && ds & CRAM_RL as ::core::ffi::c_int as uint32_t != 0 {
        if block_resize_exact((*s).seqs_blk, (qsize + 1 as ::core::ffi::c_int) as size_t)
            < 0 as ::core::ffi::c_int
        {
            current_block = 8454407981203556117;
        } else {
            current_block = 3640593987805443782;
        }
    } else {
        current_block = 3640593987805443782;
    }
    match current_block {
        3640593987805443782 => {
            if qsize != 0 && ds & CRAM_RL as ::core::ffi::c_int as uint32_t != 0 {
                if block_resize_exact((*s).qual_blk, (qsize + 1 as ::core::ffi::c_int) as size_t)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 8454407981203556117;
                } else {
                    current_block = 2968425633554183086;
                }
            } else {
                current_block = 2968425633554183086;
            }
            match current_block {
                8454407981203556117 => {}
                _ => {
                    if nsize != 0 && ds & CRAM_NS as ::core::ffi::c_int as uint32_t != 0 {
                        if block_resize_exact(
                            (*s).name_blk,
                            (nsize + 1 as ::core::ffi::c_int) as size_t,
                        ) < 0 as ::core::ffi::c_int
                        {
                            current_block = 8454407981203556117;
                        } else {
                            current_block = 4166486009154926805;
                        }
                    } else {
                        current_block = 4166486009154926805;
                    }
                    match current_block {
                        8454407981203556117 => {}
                        _ => {
                            if (*bfd).nrg > 0 as ::core::ffi::c_int
                                && !(*(*bfd)
                                    .rg
                                    .offset(((*bfd).nrg - 1 as ::core::ffi::c_int) as isize))
                                .name
                                .is_null()
                                && strcmp(
                                    (*(*bfd)
                                        .rg
                                        .offset(((*bfd).nrg - 1 as ::core::ffi::c_int) as isize))
                                    .name,
                                    b"UNKNOWN\0" as *const u8 as *const ::core::ffi::c_char,
                                ) == 0
                            {
                                unknown_rg = (*bfd).nrg - 1 as ::core::ffi::c_int;
                            }
                            if (*blk).content_type as ::core::ffi::c_int
                                != CORE as ::core::ffi::c_int
                            {
                                return -(1 as ::core::ffi::c_int);
                            }
                            if !(*s).crecs.is_null() {
                                free((*s).crecs as *mut ::core::ffi::c_void);
                            }
                            (*s).crecs = malloc(
                                ((*(*s).hdr).num_records as size_t)
                                    .wrapping_mul(::core::mem::size_of::<cram_record>() as size_t),
                            ) as *mut cram_record;
                            if (*s).crecs.is_null() {
                                return -(1 as ::core::ffi::c_int);
                            }
                            ref_id = (*(*s).hdr).ref_seq_id;
                            if ((*fd).version >> 8 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int
                            {
                                embed_ref = if (*(*s).hdr).ref_base_id >= 0 as int32_t {
                                    1 as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                };
                            } else {
                                embed_ref = if (*(*s).hdr).ref_base_id > 0 as int32_t {
                                    1 as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                };
                            }
                            if ref_id >= 0 as int32_t {
                                if embed_ref != 0 {
                                    let mut b: *mut cram_block =
                                        ::core::ptr::null_mut::<cram_block>();
                                    if (*(*s).hdr).ref_base_id < 0 as int32_t {
                                        hts_log(
                                            HTS_LOG_ERROR,
                                            b"cram_decode_slice\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"No reference specified and no embedded reference is available at #%d:%ld-%ld\0"
                                                as *const u8 as *const ::core::ffi::c_char,
                                            ref_id,
                                            (*(*s).hdr).ref_seq_start,
                                            (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span
                                                - 1 as int64_t,
                                        );
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    b = cram_get_block_by_id(
                                        s,
                                        (*(*s).hdr).ref_base_id as ::core::ffi::c_int,
                                    );
                                    if b.is_null() {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    if cram_uncompress_block(b) != 0 as ::core::ffi::c_int {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    (*s).ref_0 = (*b).data as *mut ::core::ffi::c_char;
                                    (*s).ref_start = (*(*s).hdr).ref_seq_start as hts_pos_t;
                                    (*s).ref_end = ((*(*s).hdr).ref_seq_start
                                        + (*(*s).hdr).ref_seq_span
                                        - 1 as int64_t)
                                        as hts_pos_t;
                                    if (*(*s).hdr).ref_seq_span > (*b).uncomp_size as int64_t {
                                        hts_log(
                                            HTS_LOG_ERROR,
                                            b"cram_decode_slice\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"Embedded reference is too small at #%d:%ld-%ld\0"
                                                as *const u8
                                                as *const ::core::ffi::c_char,
                                            ref_id,
                                            (*s).ref_start,
                                            (*s).ref_end,
                                        );
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                } else if (*(*c).comp_hdr).no_ref == 0 {
                                    if (*fd).required_fields
                                        & SAM_SEQ as ::core::ffi::c_int as ::core::ffi::c_uint
                                        != 0
                                    {
                                        (*s).ref_0 = cram_get_ref(
                                            fd,
                                            (*(*s).hdr).ref_seq_id as ::core::ffi::c_int,
                                            (*(*s).hdr).ref_seq_start as hts_pos_t,
                                            (*(*s).hdr).ref_seq_start as hts_pos_t
                                                + (*(*s).hdr).ref_seq_span as hts_pos_t
                                                - 1 as hts_pos_t,
                                        );
                                    }
                                    (*s).ref_start = (*(*s).hdr).ref_seq_start as hts_pos_t;
                                    (*s).ref_end = ((*(*s).hdr).ref_seq_start
                                        + (*(*s).hdr).ref_seq_span
                                        - 1 as int64_t)
                                        as hts_pos_t;
                                    if (*s).ref_start < 0 as hts_pos_t {
                                        hts_log(
                                            HTS_LOG_WARNING,
                                            b"cram_decode_slice\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"Slice starts before base 1 at #%d:%ld-%ld\0"
                                                as *const u8
                                                as *const ::core::ffi::c_char,
                                            ref_id,
                                            (*(*s).hdr).ref_seq_start,
                                            (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span
                                                - 1 as int64_t,
                                        );
                                        (*s).ref_start = 0 as hts_pos_t;
                                    }
                                    pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                    pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
                                    if (*fd).required_fields
                                        & SAM_SEQ as ::core::ffi::c_int as ::core::ffi::c_uint
                                        != 0
                                        && ref_id < (*(*fd).refs).nref as int32_t
                                        && !(*(*fd).refs).ref_id.is_null()
                                        && (*s).ref_end
                                            > (**(*(*fd).refs).ref_id.offset(ref_id as isize))
                                                .length
                                    {
                                        (*s).ref_end =
                                            (**(*(*fd).refs).ref_id.offset(ref_id as isize)).length
                                                as hts_pos_t;
                                    }
                                    pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
                                    pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                }
                            }
                            if (*fd).required_fields
                                & SAM_SEQ as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0
                                && (*s).ref_0.is_null()
                                && (*(*s).hdr).ref_seq_id >= 0 as int32_t
                                && (*(*c).comp_hdr).no_ref == 0
                            {
                                hts_log(
                                    HTS_LOG_ERROR,
                                    b"cram_decode_slice\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Unable to fetch reference %s:%ld-%ld\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    if !(*(*fd).refs).ref_id.is_null()
                                        && ref_id >= 0 as int32_t
                                        && ref_id < (*(*fd).refs).nref as int32_t
                                    {
                                        (**(*(*fd).refs).ref_id.offset(ref_id as isize)).name
                                            as *const ::core::ffi::c_char
                                    } else {
                                        b"unknown\0" as *const u8 as *const ::core::ffi::c_char
                                    },
                                    (*(*s).hdr).ref_seq_start,
                                    (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span
                                        - 1 as int64_t,
                                );
                                return -(1 as ::core::ffi::c_int);
                            }
                            if (*fd).version >> 8 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                                && (*fd).required_fields
                                    & SAM_SEQ as ::core::ffi::c_int as ::core::ffi::c_uint
                                    != 0
                                && (*(*s).hdr).ref_seq_id >= 0 as int32_t
                                && (*fd).ignore_md5 == 0
                                && memcmp(
                                    &raw mut (*(*s).hdr).md5 as *mut ::core::ffi::c_uchar
                                        as *const ::core::ffi::c_void,
                                    b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0" as *const u8
                                        as *const ::core::ffi::c_char
                                        as *const ::core::ffi::c_void,
                                    16 as size_t,
                                ) != 0
                            {
                                let mut md5: *mut hts_md5_context =
                                    ::core::ptr::null_mut::<hts_md5_context>();
                                let mut digest: [::core::ffi::c_uchar; 16] = [0; 16];
                                if !(*s).ref_0.is_null() && (*(*s).hdr).ref_seq_id >= 0 as int32_t {
                                    let mut start: ::core::ffi::c_int = 0;
                                    let mut len: ::core::ffi::c_int = 0;
                                    if (*(*s).hdr).ref_seq_start >= (*s).ref_start {
                                        start = ((*(*s).hdr).ref_seq_start
                                            - (*s).ref_start as int64_t)
                                            as ::core::ffi::c_int;
                                    } else {
                                        hts_log(
                                            HTS_LOG_WARNING,
                                            b"cram_decode_slice\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"Slice starts before base 1 at #%d:%ld-%ld\0"
                                                as *const u8
                                                as *const ::core::ffi::c_char,
                                            ref_id,
                                            (*s).ref_start,
                                            (*s).ref_end,
                                        );
                                        start = 0 as ::core::ffi::c_int;
                                    }
                                    if (*(*s).hdr).ref_seq_span
                                        <= (*s).ref_end - (*s).ref_start + 1 as hts_pos_t
                                    {
                                        len = (*(*s).hdr).ref_seq_span as ::core::ffi::c_int;
                                    } else {
                                        hts_log(
                                            HTS_LOG_WARNING,
                                            b"cram_decode_slice\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"Slice ends beyond reference end at #%d:%ld-%ld\0"
                                                as *const u8
                                                as *const ::core::ffi::c_char,
                                            ref_id,
                                            (*s).ref_start,
                                            (*s).ref_end,
                                        );
                                        len = ((*s).ref_end - (*s).ref_start + 1 as hts_pos_t)
                                            as ::core::ffi::c_int;
                                    }
                                    md5 = hts_md5_init();
                                    if md5.is_null() {
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    if (start + len) as hts_pos_t
                                        > (*s).ref_end - (*s).ref_start + 1 as hts_pos_t
                                    {
                                        len = ((*s).ref_end - (*s).ref_start + 1 as hts_pos_t
                                            - start as hts_pos_t)
                                            as ::core::ffi::c_int;
                                    }
                                    if len >= 0 as ::core::ffi::c_int {
                                        hts_md5_update(
                                            md5,
                                            (*s).ref_0.offset(start as isize)
                                                as *const ::core::ffi::c_void,
                                            len as ::core::ffi::c_ulong,
                                        );
                                    }
                                    hts_md5_final(
                                        &raw mut digest as *mut ::core::ffi::c_uchar,
                                        md5,
                                    );
                                    hts_md5_destroy(md5);
                                } else if (*s).ref_0.is_null()
                                    && (*(*s).hdr).ref_base_id >= 0 as int32_t
                                {
                                    let mut b_0: *mut cram_block = cram_get_block_by_id(
                                        s,
                                        (*(*s).hdr).ref_base_id as ::core::ffi::c_int,
                                    );
                                    if !b_0.is_null() {
                                        md5 = hts_md5_init();
                                        if md5.is_null() {
                                            return -(1 as ::core::ffi::c_int);
                                        }
                                        hts_md5_update(
                                            md5,
                                            (*b_0).data as *const ::core::ffi::c_void,
                                            (*b_0).uncomp_size as ::core::ffi::c_ulong,
                                        );
                                        hts_md5_final(
                                            &raw mut digest as *mut ::core::ffi::c_uchar,
                                            md5,
                                        );
                                        hts_md5_destroy(md5);
                                    }
                                }
                                if (*(*c).comp_hdr).no_ref == 0
                                    && ((*s).ref_0.is_null()
                                        && (*(*s).hdr).ref_base_id < 0 as int32_t
                                        || memcmp(
                                            &raw mut digest as *mut ::core::ffi::c_uchar
                                                as *const ::core::ffi::c_void,
                                            &raw mut (*(*s).hdr).md5 as *mut ::core::ffi::c_uchar
                                                as *const ::core::ffi::c_void,
                                            16 as size_t,
                                        ) != 0 as ::core::ffi::c_int)
                                {
                                    let mut M: [::core::ffi::c_char; 33] = [0; 33];
                                    let mut rname: *const ::core::ffi::c_char =
                                        sam_hdr_tid2name(sh, ref_id as ::core::ffi::c_int);
                                    if rname.is_null() {
                                        rname = b"?\0" as *const u8 as *const ::core::ffi::c_char;
                                    }
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_decode_slice\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"MD5 checksum reference mismatch at %s:%ld-%ld\0"
                                            as *const u8
                                            as *const ::core::ffi::c_char,
                                        rname,
                                        (*s).ref_start,
                                        (*s).ref_end,
                                    );
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_decode_slice\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"CRAM  : %s\0" as *const u8 as *const ::core::ffi::c_char,
                                        md5_print(
                                            &raw mut (*(*s).hdr).md5 as *mut ::core::ffi::c_uchar,
                                            &raw mut M as *mut ::core::ffi::c_char,
                                        ),
                                    );
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_decode_slice\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Ref   : %s\0" as *const u8 as *const ::core::ffi::c_char,
                                        md5_print(
                                            &raw mut digest as *mut ::core::ffi::c_uchar,
                                            &raw mut M as *mut ::core::ffi::c_char,
                                        ),
                                    );
                                    let mut ks: kstring_t = kstring_t {
                                        l: 0 as size_t,
                                        m: 0 as size_t,
                                        s: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    };
                                    if sam_hdr_find_tag_id(
                                        sh,
                                        b"SQ\0" as *const u8 as *const ::core::ffi::c_char,
                                        b"SN\0" as *const u8 as *const ::core::ffi::c_char,
                                        rname,
                                        b"M5\0" as *const u8 as *const ::core::ffi::c_char,
                                        &raw mut ks,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        hts_log(
                                            HTS_LOG_ERROR,
                                            b"cram_decode_slice\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"@SQ M5: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            ks.s,
                                        );
                                    }
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_decode_slice\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Please check the reference given is correct\0"
                                            as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                    ks_free(&raw mut ks);
                                    return -(1 as ::core::ffi::c_int);
                                }
                            }
                            if ref_id == -(2 as int32_t) {
                                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
                                refs = calloc(
                                    (*(*fd).refs).nref as size_t,
                                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                                )
                                    as *mut *mut ::core::ffi::c_char;
                                pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
                                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                if refs.is_null() {
                                    return -(1 as ::core::ffi::c_int);
                                }
                            }
                            last_ref_id = -(9 as ::core::ffi::c_int);
                            rec = 0 as ::core::ffi::c_int;
                            loop {
                                if !((rec as int32_t) < (*(*s).hdr).num_records) {
                                    current_block = 11231108819225936288;
                                    break;
                                }
                                let mut cr: *mut cram_record =
                                    (*s).crecs.offset(rec as isize) as *mut cram_record;
                                let mut has_MD: ::core::ffi::c_int = 0;
                                let mut has_NM: ::core::ffi::c_int = 0;
                                (*cr).s = s as *mut cram_slice;
                                out_sz = 1 as ::core::ffi::c_int;
                                if ds & CRAM_BF as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*(*c).comp_hdr).codecs[DS_BF as ::core::ffi::c_int as usize]
                                        .is_null()
                                    {
                                        current_block = 8454407981203556117;
                                        break;
                                    }
                                    r |= (*(*(*c).comp_hdr).codecs
                                        [DS_BF as ::core::ffi::c_int as usize])
                                        .decode
                                        .expect("non-null function pointer")(
                                        s,
                                        (*(*c).comp_hdr).codecs
                                            [DS_BF as ::core::ffi::c_int as usize],
                                        blk,
                                        &raw mut bf as *mut ::core::ffi::c_char,
                                        &raw mut out_sz,
                                    );
                                    if r != 0
                                        || bf < 0 as int32_t
                                        || bf as usize
                                            >= (::core::mem::size_of::<[::core::ffi::c_uint; 4096]>(
                                            )
                                                as usize)
                                                .wrapping_div(::core::mem::size_of::<
                                                    ::core::ffi::c_uint,
                                                >(
                                                )
                                                    as usize)
                                    {
                                        current_block = 8454407981203556117;
                                        break;
                                    }
                                    bf = (*fd).bam_flag_swap[bf as usize] as int32_t;
                                    (*cr).flags = bf;
                                } else {
                                    bf = 0x4 as ::core::ffi::c_int as int32_t;
                                    (*cr).flags = bf;
                                }
                                if ds & CRAM_CF as ::core::ffi::c_int as uint32_t != 0 {
                                    if (*fd).version >> 8 as ::core::ffi::c_int
                                        == 1 as ::core::ffi::c_int
                                    {
                                        if (*(*c).comp_hdr).codecs
                                            [DS_CF as ::core::ffi::c_int as usize]
                                            .is_null()
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        r |= (*(*(*c).comp_hdr).codecs
                                            [DS_CF as ::core::ffi::c_int as usize])
                                            .decode
                                            .expect("non-null function pointer")(
                                            s,
                                            (*(*c).comp_hdr).codecs
                                                [DS_CF as ::core::ffi::c_int as usize],
                                            blk,
                                            &raw mut cf as *mut ::core::ffi::c_char,
                                            &raw mut out_sz,
                                        );
                                        if r != 0 {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        (*cr).cram_flags = cf as int32_t;
                                    } else {
                                        if (*(*c).comp_hdr).codecs
                                            [DS_CF as ::core::ffi::c_int as usize]
                                            .is_null()
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        r |= (*(*(*c).comp_hdr).codecs
                                            [DS_CF as ::core::ffi::c_int as usize])
                                            .decode
                                            .expect("non-null function pointer")(
                                            s,
                                            (*(*c).comp_hdr).codecs
                                                [DS_CF as ::core::ffi::c_int as usize],
                                            blk,
                                            &raw mut (*cr).cram_flags as *mut ::core::ffi::c_char,
                                            &raw mut out_sz,
                                        );
                                        if r != 0 {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        cf = (*cr).cram_flags as ::core::ffi::c_uchar;
                                    }
                                } else {
                                    (*cr).cram_flags = 0 as ::core::ffi::c_int as int32_t;
                                    cf = (*cr).cram_flags as ::core::ffi::c_uchar;
                                }
                                if (*fd).version >> 8 as ::core::ffi::c_int
                                    != 1 as ::core::ffi::c_int
                                    && ref_id == -(2 as int32_t)
                                {
                                    if ds & CRAM_RI as ::core::ffi::c_int as uint32_t != 0 {
                                        if (*(*c).comp_hdr).codecs
                                            [DS_RI as ::core::ffi::c_int as usize]
                                            .is_null()
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        r |= (*(*(*c).comp_hdr).codecs
                                            [DS_RI as ::core::ffi::c_int as usize])
                                            .decode
                                            .expect("non-null function pointer")(
                                            s,
                                            (*(*c).comp_hdr).codecs
                                                [DS_RI as ::core::ffi::c_int as usize],
                                            blk,
                                            &raw mut (*cr).ref_id as *mut ::core::ffi::c_char,
                                            &raw mut out_sz,
                                        );
                                        if r != 0 {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        if (*cr).ref_id < -(1 as int32_t)
                                            || (*cr).ref_id >= (*bfd).nref as int32_t
                                        {
                                            hts_log(
                                                HTS_LOG_ERROR,
                                                b"cram_decode_slice\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                b"Requested unknown reference ID %d\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                (*cr).ref_id,
                                            );
                                            current_block = 8454407981203556117;
                                            break;
                                        } else if (*fd).required_fields
                                            & (SAM_SEQ as ::core::ffi::c_int
                                                | SAM_TLEN as ::core::ffi::c_int)
                                                as ::core::ffi::c_uint
                                            != 0
                                            && (*cr).ref_id >= 0 as int32_t
                                            && (*cr).ref_id != last_ref_id as int32_t
                                        {
                                            if (*(*c).comp_hdr).no_ref == 0 {
                                                pthread_mutex_lock(&raw mut (*fd).range_lock);
                                                let mut need_ref: ::core::ffi::c_int = ((*fd)
                                                    .range
                                                    .refid
                                                    == -(2 as ::core::ffi::c_int)
                                                    || (*cr).ref_id == (*fd).range.refid as int32_t)
                                                    as ::core::ffi::c_int;
                                                pthread_mutex_unlock(&raw mut (*fd).range_lock);
                                                if need_ref != 0 {
                                                    if (*refs.offset((*cr).ref_id as isize))
                                                        .is_null()
                                                    {
                                                        let ref mut fresh128 =
                                                            *refs.offset((*cr).ref_id as isize);
                                                        *fresh128 = cram_get_ref(
                                                            fd,
                                                            (*cr).ref_id as ::core::ffi::c_int,
                                                            1 as hts_pos_t,
                                                            0 as hts_pos_t,
                                                        );
                                                    }
                                                    (*s).ref_0 =
                                                        *refs.offset((*cr).ref_id as isize);
                                                    if (*s).ref_0.is_null() {
                                                        current_block = 8454407981203556117;
                                                        break;
                                                    }
                                                } else {
                                                    (*s).ref_0 = ::core::ptr::null_mut::<
                                                        ::core::ffi::c_char,
                                                    >(
                                                    );
                                                }
                                                pthread_mutex_lock(&raw mut (*fd).range_lock);
                                                let mut discard_last_ref: ::core::ffi::c_int =
                                                    (last_ref_id >= 0 as ::core::ffi::c_int
                                                        && !(*refs.offset(last_ref_id as isize))
                                                            .is_null()
                                                        && ((*fd).range.refid
                                                            == -(2 as ::core::ffi::c_int)
                                                            || last_ref_id == (*fd).range.refid))
                                                        as ::core::ffi::c_int;
                                                pthread_mutex_unlock(&raw mut (*fd).range_lock);
                                                if discard_last_ref != 0 {
                                                    pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                                    discard_last_ref =
                                                        ((*fd).unsorted == 0) as ::core::ffi::c_int;
                                                    pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                                }
                                                if discard_last_ref != 0 {
                                                    cram_ref_decr((*fd).refs, last_ref_id);
                                                    let ref mut fresh129 =
                                                        *refs.offset(last_ref_id as isize);
                                                    *fresh129 = ::core::ptr::null_mut::<
                                                        ::core::ffi::c_char,
                                                    >(
                                                    );
                                                }
                                            }
                                            (*s).ref_start = 1 as hts_pos_t;
                                            pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                            pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
                                            (*s).ref_end = (**(*(*fd).refs)
                                                .ref_id
                                                .offset((*cr).ref_id as isize))
                                            .length
                                                as hts_pos_t;
                                            pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
                                            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                            last_ref_id = (*cr).ref_id as ::core::ffi::c_int;
                                        }
                                    } else {
                                        (*cr).ref_id = -(1 as ::core::ffi::c_int) as int32_t;
                                    }
                                } else {
                                    (*cr).ref_id = ref_id;
                                }
                                if (*cr).ref_id < -(1 as int32_t)
                                    || (*cr).ref_id >= (*bfd).nref as int32_t
                                {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_decode_slice\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Requested unknown reference ID %d\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        (*cr).ref_id,
                                    );
                                    current_block = 8454407981203556117;
                                    break;
                                } else {
                                    if ds & CRAM_RL as ::core::ffi::c_int as uint32_t != 0 {
                                        if (*(*c).comp_hdr).codecs
                                            [DS_RL as ::core::ffi::c_int as usize]
                                            .is_null()
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        r |= (*(*(*c).comp_hdr).codecs
                                            [DS_RL as ::core::ffi::c_int as usize])
                                            .decode
                                            .expect("non-null function pointer")(
                                            s,
                                            (*(*c).comp_hdr).codecs
                                                [DS_RL as ::core::ffi::c_int as usize],
                                            blk,
                                            &raw mut (*cr).len as *mut ::core::ffi::c_char,
                                            &raw mut out_sz,
                                        );
                                        if r != 0 {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        if (*cr).len < 0 as int32_t {
                                            hts_log(
                                                HTS_LOG_ERROR,
                                                b"cram_decode_slice\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                b"Read has negative length\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                    }
                                    if ds & CRAM_AP as ::core::ffi::c_int as uint32_t != 0 {
                                        if (*(*c).comp_hdr).codecs
                                            [DS_AP as ::core::ffi::c_int as usize]
                                            .is_null()
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        if (*fd).version >> 8 as ::core::ffi::c_int
                                            >= 4 as ::core::ffi::c_int
                                        {
                                            r |= (*(*(*c).comp_hdr).codecs
                                                [DS_AP as ::core::ffi::c_int as usize])
                                                .decode
                                                .expect("non-null function pointer")(
                                                s,
                                                (*(*c).comp_hdr).codecs
                                                    [DS_AP as ::core::ffi::c_int as usize],
                                                blk,
                                                &raw mut (*cr).apos as *mut ::core::ffi::c_char,
                                                &raw mut out_sz,
                                            );
                                        } else {
                                            let mut i32: int32_t = 0;
                                            r |= (*(*(*c).comp_hdr).codecs
                                                [DS_AP as ::core::ffi::c_int as usize])
                                                .decode
                                                .expect("non-null function pointer")(
                                                s,
                                                (*(*c).comp_hdr).codecs
                                                    [DS_AP as ::core::ffi::c_int as usize],
                                                blk,
                                                &raw mut i32 as *mut ::core::ffi::c_char,
                                                &raw mut out_sz,
                                            );
                                            (*cr).apos = i32 as int64_t;
                                        }
                                        if r != 0 {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        if (*(*c).comp_hdr).AP_delta != 0 {
                                            if (*cr).apos < 0 as int64_t
                                                && (*c).unsorted == 0 as ::core::ffi::c_int
                                            {
                                                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                                (*fd).unsorted = 1 as ::core::ffi::c_int;
                                                (*c).unsorted = (*fd).unsorted;
                                                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                            }
                                            (*cr).apos = ((*cr).apos as ::core::ffi::c_long
                                                + (*s).last_apos as ::core::ffi::c_long)
                                                as int64_t;
                                        }
                                        (*s).last_apos = (*cr).apos;
                                        if (*(*s).hdr).ref_seq_id >= 0 as int32_t
                                            && (*cr).apos < (*(*s).hdr).ref_seq_start
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                    } else {
                                        (*cr).apos = (*c).ref_seq_start;
                                    }
                                    if ds & CRAM_RG as ::core::ffi::c_int as uint32_t != 0 {
                                        if (*(*c).comp_hdr).codecs
                                            [DS_RG as ::core::ffi::c_int as usize]
                                            .is_null()
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        r |= (*(*(*c).comp_hdr).codecs
                                            [DS_RG as ::core::ffi::c_int as usize])
                                            .decode
                                            .expect("non-null function pointer")(
                                            s,
                                            (*(*c).comp_hdr).codecs
                                                [DS_RG as ::core::ffi::c_int as usize],
                                            blk,
                                            &raw mut (*cr).rg as *mut ::core::ffi::c_char,
                                            &raw mut out_sz,
                                        );
                                        if r != 0 {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        if (*cr).rg == unknown_rg as int32_t {
                                            (*cr).rg = -(1 as ::core::ffi::c_int) as int32_t;
                                        }
                                    } else {
                                        (*cr).rg = -(1 as ::core::ffi::c_int) as int32_t;
                                    }
                                    (*cr).name_len = 0 as ::core::ffi::c_int as int32_t;
                                    if (*(*c).comp_hdr).read_names_included != 0 {
                                        let mut out_sz2: int32_t = 1 as int32_t;
                                        (*cr).name = (*(*s).name_blk).byte as int32_t;
                                        if ds & CRAM_RN as ::core::ffi::c_int as uint32_t != 0 {
                                            if (*(*c).comp_hdr).codecs
                                                [DS_RN as ::core::ffi::c_int as usize]
                                                .is_null()
                                            {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            r |= (*(*(*c).comp_hdr).codecs
                                                [DS_RN as ::core::ffi::c_int as usize])
                                                .decode
                                                .expect("non-null function pointer")(
                                                s,
                                                (*(*c).comp_hdr).codecs
                                                    [DS_RN as ::core::ffi::c_int as usize],
                                                blk,
                                                (*s).name_blk as *mut ::core::ffi::c_char,
                                                &raw mut out_sz2,
                                            );
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            (*cr).name_len = out_sz2;
                                        }
                                    }
                                    (*cr).mate_pos = 0 as int64_t;
                                    (*cr).mate_line = -(1 as ::core::ffi::c_int) as int32_t;
                                    (*cr).mate_ref_id = -(1 as ::core::ffi::c_int) as int32_t;
                                    (*cr).explicit_tlen = INT64_MIN as int64_t;
                                    if ds & CRAM_CF as ::core::ffi::c_int as uint32_t != 0
                                        && cf as ::core::ffi::c_int & CRAM_FLAG_DETACHED != 0
                                    {
                                        if ds & CRAM_MF as ::core::ffi::c_int as uint32_t != 0 {
                                            if (*fd).version >> 8 as ::core::ffi::c_int
                                                == 1 as ::core::ffi::c_int
                                            {
                                                let mut mf: ::core::ffi::c_uchar = 0;
                                                if (*(*c).comp_hdr).codecs
                                                    [DS_MF as ::core::ffi::c_int as usize]
                                                    .is_null()
                                                {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                                r |= (*(*(*c).comp_hdr).codecs
                                                    [DS_MF as ::core::ffi::c_int as usize])
                                                    .decode
                                                    .expect("non-null function pointer")(
                                                    s,
                                                    (*(*c).comp_hdr).codecs
                                                        [DS_MF as ::core::ffi::c_int as usize],
                                                    blk,
                                                    &raw mut mf as *mut ::core::ffi::c_char,
                                                    &raw mut out_sz,
                                                );
                                                if r != 0 {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                                (*cr).mate_flags = mf as int32_t;
                                            } else {
                                                if (*(*c).comp_hdr).codecs
                                                    [DS_MF as ::core::ffi::c_int as usize]
                                                    .is_null()
                                                {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                                r |= (*(*(*c).comp_hdr).codecs
                                                    [DS_MF as ::core::ffi::c_int as usize])
                                                    .decode
                                                    .expect("non-null function pointer")(
                                                    s,
                                                    (*(*c).comp_hdr).codecs
                                                        [DS_MF as ::core::ffi::c_int as usize],
                                                    blk,
                                                    &raw mut (*cr).mate_flags
                                                        as *mut ::core::ffi::c_char,
                                                    &raw mut out_sz,
                                                );
                                                if r != 0 {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                            }
                                        } else {
                                            (*cr).mate_flags = 0 as ::core::ffi::c_int as int32_t;
                                        }
                                        if (*(*c).comp_hdr).read_names_included == 0 {
                                            let mut out_sz2_0: int32_t = 1 as int32_t;
                                            (*cr).name = (*(*s).name_blk).byte as int32_t;
                                            if ds & CRAM_RN as ::core::ffi::c_int as uint32_t != 0 {
                                                if (*(*c).comp_hdr).codecs
                                                    [DS_RN as ::core::ffi::c_int as usize]
                                                    .is_null()
                                                {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                                r |= (*(*(*c).comp_hdr).codecs
                                                    [DS_RN as ::core::ffi::c_int as usize])
                                                    .decode
                                                    .expect("non-null function pointer")(
                                                    s,
                                                    (*(*c).comp_hdr).codecs
                                                        [DS_RN as ::core::ffi::c_int as usize],
                                                    blk,
                                                    (*s).name_blk as *mut ::core::ffi::c_char,
                                                    &raw mut out_sz2_0,
                                                );
                                                if r != 0 {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                                (*cr).name_len = out_sz2_0;
                                            }
                                        }
                                        if ds & CRAM_NS as ::core::ffi::c_int as uint32_t != 0 {
                                            if (*(*c).comp_hdr).codecs
                                                [DS_NS as ::core::ffi::c_int as usize]
                                                .is_null()
                                            {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            r |= (*(*(*c).comp_hdr).codecs
                                                [DS_NS as ::core::ffi::c_int as usize])
                                                .decode
                                                .expect("non-null function pointer")(
                                                s,
                                                (*(*c).comp_hdr).codecs
                                                    [DS_NS as ::core::ffi::c_int as usize],
                                                blk,
                                                &raw mut (*cr).mate_ref_id
                                                    as *mut ::core::ffi::c_char,
                                                &raw mut out_sz,
                                            );
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            if (*cr).mate_ref_id < -(1 as int32_t)
                                                || (*cr).mate_ref_id >= (*bfd).nref as int32_t
                                            {
                                                hts_log(
                                                    HTS_LOG_ERROR,
                                                    b"cram_decode_slice\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"Requested unknown mate reference ID %d\0"
                                                        as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    (*cr).mate_ref_id,
                                                );
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                        }
                                        if ds & CRAM_NP as ::core::ffi::c_int as uint32_t != 0 {
                                            if (*(*c).comp_hdr).codecs
                                                [DS_NP as ::core::ffi::c_int as usize]
                                                .is_null()
                                            {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            if ((*fd).version >> 8 as ::core::ffi::c_int)
                                                < 4 as ::core::ffi::c_int
                                            {
                                                let mut i32_0: int32_t = 0;
                                                r |= (*(*(*c).comp_hdr).codecs
                                                    [DS_NP as ::core::ffi::c_int as usize])
                                                    .decode
                                                    .expect("non-null function pointer")(
                                                    s,
                                                    (*(*c).comp_hdr).codecs
                                                        [DS_NP as ::core::ffi::c_int as usize],
                                                    blk,
                                                    &raw mut i32_0 as *mut ::core::ffi::c_char,
                                                    &raw mut out_sz,
                                                );
                                                (*cr).mate_pos = i32_0 as int64_t;
                                            } else {
                                                r |= (*(*(*c).comp_hdr).codecs
                                                    [DS_NP as ::core::ffi::c_int as usize])
                                                    .decode
                                                    .expect("non-null function pointer")(
                                                    s,
                                                    (*(*c).comp_hdr).codecs
                                                        [DS_NP as ::core::ffi::c_int as usize],
                                                    blk,
                                                    &raw mut (*cr).mate_pos
                                                        as *mut ::core::ffi::c_char,
                                                    &raw mut out_sz,
                                                );
                                            }
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                        }
                                        if ds & CRAM_TS as ::core::ffi::c_int as uint32_t != 0 {
                                            if (*(*c).comp_hdr).codecs
                                                [DS_TS as ::core::ffi::c_int as usize]
                                                .is_null()
                                            {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            r = cram_decode_tlen(
                                                fd,
                                                c,
                                                s,
                                                blk,
                                                &raw mut (*cr).tlen,
                                            );
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                        } else {
                                            (*cr).tlen = INT64_MIN as int64_t;
                                        }
                                    } else if ds & CRAM_CF as ::core::ffi::c_int as uint32_t != 0
                                        && cf as ::core::ffi::c_int & CRAM_FLAG_MATE_DOWNSTREAM != 0
                                    {
                                        if ds & CRAM_NF as ::core::ffi::c_int as uint32_t != 0 {
                                            if (*(*c).comp_hdr).codecs
                                                [DS_NF as ::core::ffi::c_int as usize]
                                                .is_null()
                                            {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            r |= (*(*(*c).comp_hdr).codecs
                                                [DS_NF as ::core::ffi::c_int as usize])
                                                .decode
                                                .expect("non-null function pointer")(
                                                s,
                                                (*(*c).comp_hdr).codecs
                                                    [DS_NF as ::core::ffi::c_int as usize],
                                                blk,
                                                &raw mut (*cr).mate_line
                                                    as *mut ::core::ffi::c_char,
                                                &raw mut out_sz,
                                            );
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            (*cr).mate_line = ((*cr).mate_line
                                                as ::core::ffi::c_int
                                                + (rec + 1 as ::core::ffi::c_int))
                                                as int32_t;
                                            (*cr).mate_ref_id =
                                                -(1 as ::core::ffi::c_int) as int32_t;
                                            (*cr).tlen = INT64_MIN as int64_t;
                                            (*cr).mate_pos = 0 as int64_t;
                                        } else {
                                            (*cr).mate_flags = 0 as ::core::ffi::c_int as int32_t;
                                            (*cr).tlen = INT64_MIN as int64_t;
                                        }
                                        if ds & CRAM_CF as ::core::ffi::c_int as uint32_t != 0
                                            && cf as ::core::ffi::c_int & CRAM_FLAG_EXPLICIT_TLEN
                                                != 0
                                        {
                                            if ds & CRAM_TS as ::core::ffi::c_int as uint32_t != 0 {
                                                r = cram_decode_tlen(
                                                    fd,
                                                    c,
                                                    s,
                                                    blk,
                                                    &raw mut (*cr).explicit_tlen,
                                                );
                                                if r != 0 {
                                                    return r;
                                                }
                                            } else {
                                                (*cr).mate_flags =
                                                    0 as ::core::ffi::c_int as int32_t;
                                                (*cr).tlen = INT64_MIN as int64_t;
                                            }
                                        }
                                    } else if ds & CRAM_CF as ::core::ffi::c_int as uint32_t != 0
                                        && cf as ::core::ffi::c_int & CRAM_FLAG_EXPLICIT_TLEN != 0
                                    {
                                        if ds & CRAM_TS as ::core::ffi::c_int as uint32_t != 0 {
                                            r = cram_decode_tlen(
                                                fd,
                                                c,
                                                s,
                                                blk,
                                                &raw mut (*cr).explicit_tlen,
                                            );
                                            if r != 0 {
                                                return r;
                                            }
                                        } else {
                                            (*cr).mate_flags = 0 as ::core::ffi::c_int as int32_t;
                                            (*cr).tlen = INT64_MIN as int64_t;
                                        }
                                    } else {
                                        (*cr).mate_flags = 0 as ::core::ffi::c_int as int32_t;
                                        (*cr).tlen = INT64_MIN as int64_t;
                                    }
                                    has_NM = 0 as ::core::ffi::c_int;
                                    has_MD = has_NM;
                                    if (*fd).version >> 8 as ::core::ffi::c_int
                                        == 1 as ::core::ffi::c_int
                                    {
                                        r |= cram_decode_aux_1_0(c, s, blk, cr);
                                    } else {
                                        r |= cram_decode_aux(
                                            fd,
                                            c,
                                            s,
                                            blk,
                                            cr,
                                            &raw mut has_MD,
                                            &raw mut has_NM,
                                        );
                                    }
                                    if r != 0 {
                                        current_block = 8454407981203556117;
                                        break;
                                    }
                                    if ds & CRAM_RL as ::core::ffi::c_int as uint32_t != 0 {
                                        (*cr).seq = (*(*s).seqs_blk).byte as uint32_t;
                                        if block_resize(
                                            (*s).seqs_blk,
                                            (*cr).seq.wrapping_add((*cr).len as uint32_t) as size_t,
                                        ) < 0 as ::core::ffi::c_int
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        seq = (*(*s).seqs_blk)
                                            .data
                                            .offset((*(*s).seqs_blk).byte as isize)
                                            as *mut ::core::ffi::c_uchar
                                            as *mut ::core::ffi::c_char;
                                        (*(*s).seqs_blk).byte = ((*(*s).seqs_blk).byte
                                            as ::core::ffi::c_ulong)
                                            .wrapping_add((*cr).len as ::core::ffi::c_ulong)
                                            as size_t
                                            as size_t;
                                        if seq.is_null() {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        (*cr).qual = (*(*s).qual_blk).byte as uint32_t;
                                        if block_resize(
                                            (*s).qual_blk,
                                            (*cr).qual.wrapping_add((*cr).len as uint32_t)
                                                as size_t,
                                        ) < 0 as ::core::ffi::c_int
                                        {
                                            current_block = 8454407981203556117;
                                            break;
                                        }
                                        qual = (*(*s).qual_blk)
                                            .data
                                            .offset((*(*s).qual_blk).byte as isize)
                                            as *mut ::core::ffi::c_uchar
                                            as *mut ::core::ffi::c_char;
                                        (*(*s).qual_blk).byte = ((*(*s).qual_blk).byte
                                            as ::core::ffi::c_ulong)
                                            .wrapping_add((*cr).len as ::core::ffi::c_ulong)
                                            as size_t
                                            as size_t;
                                        if (*s).ref_0.is_null() {
                                            memset(
                                                seq as *mut ::core::ffi::c_void,
                                                '=' as i32,
                                                (*cr).len as size_t,
                                            );
                                        }
                                    }
                                    if bf & BAM_FUNMAP as int32_t == 0 {
                                        if ds & CRAM_AP as ::core::ffi::c_int as uint32_t != 0
                                            && (*cr).apos <= 0 as int64_t
                                        {
                                            hts_log(
                                                HTS_LOG_ERROR,
                                                b"cram_decode_slice\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                b"Read has alignment position %ld but no unmapped flag\0"
                                                    as *const u8 as *const ::core::ffi::c_char,
                                                (*cr).apos,
                                            );
                                            current_block = 8454407981203556117;
                                            break;
                                        } else if ds
                                            & (CRAM_FN as ::core::ffi::c_int
                                                | CRAM_FP as ::core::ffi::c_int
                                                | CRAM_FC as ::core::ffi::c_int
                                                | CRAM_DL as ::core::ffi::c_int
                                                | CRAM_IN as ::core::ffi::c_int
                                                | CRAM_SC as ::core::ffi::c_int
                                                | CRAM_HC as ::core::ffi::c_int
                                                | CRAM_PD as ::core::ffi::c_int
                                                | CRAM_RS as ::core::ffi::c_int
                                                | CRAM_RL as ::core::ffi::c_int
                                                | CRAM_BF as ::core::ffi::c_int
                                                | CRAM_BA as ::core::ffi::c_int
                                                | CRAM_BS as ::core::ffi::c_int
                                                | CRAM_RL as ::core::ffi::c_int
                                                | CRAM_AP as ::core::ffi::c_int
                                                | CRAM_BB as ::core::ffi::c_int
                                                | CRAM_MQ as ::core::ffi::c_int)
                                                as uint32_t
                                            != 0
                                        {
                                            r |= cram_decode_seq(
                                                fd,
                                                c,
                                                s,
                                                blk,
                                                cr,
                                                sh,
                                                cf as ::core::ffi::c_int,
                                                seq,
                                                qual,
                                                has_MD,
                                                has_NM,
                                            );
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                        } else {
                                            (*cr).cigar = 0 as uint32_t;
                                            (*cr).ncigar = 0 as ::core::ffi::c_int as int32_t;
                                            (*cr).aend = (*cr).apos;
                                            (*cr).mqual = 0 as ::core::ffi::c_int as int32_t;
                                        }
                                    } else {
                                        let mut out_sz2_1: ::core::ffi::c_int =
                                            (*cr).len as ::core::ffi::c_int;
                                        (*cr).cigar = 0 as uint32_t;
                                        (*cr).ncigar = 0 as ::core::ffi::c_int as int32_t;
                                        (*cr).aend = (*cr).apos;
                                        (*cr).mqual = 0 as ::core::ffi::c_int as int32_t;
                                        if ds & CRAM_BA as ::core::ffi::c_int as uint32_t != 0
                                            && (*cr).len != 0
                                        {
                                            if (*(*c).comp_hdr).codecs
                                                [DS_BA as ::core::ffi::c_int as usize]
                                                .is_null()
                                            {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                            r |= (*(*(*c).comp_hdr).codecs
                                                [DS_BA as ::core::ffi::c_int as usize])
                                                .decode
                                                .expect("non-null function pointer")(
                                                s,
                                                (*(*c).comp_hdr).codecs
                                                    [DS_BA as ::core::ffi::c_int as usize],
                                                blk,
                                                seq,
                                                &raw mut out_sz2_1,
                                            );
                                            if r != 0 {
                                                current_block = 8454407981203556117;
                                                break;
                                            }
                                        }
                                        if ds & CRAM_CF as ::core::ffi::c_int as uint32_t != 0
                                            && cf as ::core::ffi::c_int
                                                & CRAM_FLAG_PRESERVE_QUAL_SCORES
                                                != 0
                                        {
                                            out_sz2_1 = (*cr).len as ::core::ffi::c_int;
                                            if ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0
                                                && (*cr).len >= 0 as int32_t
                                            {
                                                if (*(*c).comp_hdr).codecs
                                                    [DS_QS as ::core::ffi::c_int as usize]
                                                    .is_null()
                                                {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                                r |= (*(*(*c).comp_hdr).codecs
                                                    [DS_QS as ::core::ffi::c_int as usize])
                                                    .decode
                                                    .expect("non-null function pointer")(
                                                    s,
                                                    (*(*c).comp_hdr).codecs
                                                        [DS_QS as ::core::ffi::c_int as usize],
                                                    blk,
                                                    qual,
                                                    &raw mut out_sz2_1,
                                                );
                                                if r != 0 {
                                                    current_block = 8454407981203556117;
                                                    break;
                                                }
                                            }
                                        } else if ds & CRAM_RL as ::core::ffi::c_int as uint32_t
                                            != 0
                                        {
                                            memset(
                                                qual as *mut ::core::ffi::c_void,
                                                255 as ::core::ffi::c_int,
                                                (*cr).len as size_t,
                                            );
                                        }
                                    }
                                    if (*(*c).comp_hdr).qs_seq_orient == 0
                                        && ds & CRAM_QS as ::core::ffi::c_int as uint32_t != 0
                                        && (*cr).flags & BAM_FREVERSE as int32_t != 0
                                    {
                                        let mut i: ::core::ffi::c_int = 0;
                                        let mut j: ::core::ffi::c_int = 0;
                                        i = 0 as ::core::ffi::c_int;
                                        j = ((*cr).len - 1 as int32_t) as ::core::ffi::c_int;
                                        while i < j {
                                            let mut c_0: ::core::ffi::c_uchar = 0;
                                            c_0 = *qual.offset(i as isize) as ::core::ffi::c_uchar;
                                            *qual.offset(i as isize) = *qual.offset(j as isize);
                                            *qual.offset(j as isize) = c_0 as ::core::ffi::c_char;
                                            i += 1;
                                            j -= 1;
                                        }
                                    }
                                    rec += 1;
                                }
                            }
                            match current_block {
                                8454407981203556117 => {}
                                _ => {
                                    pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                    if !refs.is_null() {
                                        let mut i_0: ::core::ffi::c_int = 0;
                                        i_0 = 0 as ::core::ffi::c_int;
                                        while i_0 < (*(*fd).refs).nref {
                                            if !(*refs.offset(i_0 as isize)).is_null() {
                                                cram_ref_decr((*fd).refs, i_0);
                                            }
                                            i_0 += 1;
                                        }
                                        free(refs as *mut ::core::ffi::c_void);
                                        refs = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
                                    } else if ref_id >= 0 as int32_t
                                        && (*s).ref_0 != (*fd).ref_free
                                        && embed_ref == 0
                                    {
                                        cram_ref_decr((*fd).refs, ref_id as ::core::ffi::c_int);
                                    }
                                    pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                    r |= cram_decode_slice_xref(
                                        s,
                                        (*fd).required_fields as ::core::ffi::c_int,
                                    );
                                    let mut i_1: ::core::ffi::c_int = 0;
                                    i_1 = 0 as ::core::ffi::c_int;
                                    while (i_1 as int32_t) < (*(*s).hdr).num_blocks {
                                        let mut b_1: *mut cram_block =
                                            *(*s).block.offset(i_1 as isize);
                                        cram_free_block(b_1);
                                        let ref mut fresh130 = *(*s).block.offset(i_1 as isize);
                                        *fresh130 = ::core::ptr::null_mut::<cram_block>();
                                        i_1 += 1;
                                    }
                                    if !(block_resize_exact(
                                        (*s).seqs_blk,
                                        (*(*s).seqs_blk).byte.wrapping_add(1 as size_t),
                                    ) < 0 as ::core::ffi::c_int)
                                    {
                                        if !(block_resize_exact(
                                            (*s).qual_blk,
                                            (*(*s).qual_blk).byte.wrapping_add(1 as size_t),
                                        ) < 0 as ::core::ffi::c_int)
                                        {
                                            if !(block_resize_exact(
                                                (*s).name_blk,
                                                (*(*s).name_blk).byte.wrapping_add(1 as size_t),
                                            ) < 0 as ::core::ffi::c_int)
                                            {
                                                if !(block_resize_exact(
                                                    (*s).aux_blk,
                                                    (*(*s).aux_blk).byte.wrapping_add(1 as size_t),
                                                ) < 0 as ::core::ffi::c_int)
                                                {
                                                    return r;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if !refs.is_null() {
        let mut i_2: ::core::ffi::c_int = 0;
        pthread_mutex_lock(&raw mut (*fd).ref_lock);
        i_2 = 0 as ::core::ffi::c_int;
        while i_2 < (*(*fd).refs).nref {
            if !(*refs.offset(i_2 as isize)).is_null() {
                cram_ref_decr((*fd).refs, i_2);
            }
            i_2 += 1;
        }
        free(refs as *mut ::core::ffi::c_void);
        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
// original: cram_decode_slice_thread (htslib/cram/cram_decode.c:3036)
pub unsafe extern "C" fn cram_decode_slice_thread(
    mut arg: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    let mut j: *mut cram_decode_job = arg as *mut cram_decode_job;
    (*j).exit_code = cram_decode_slice((*j).fd, (*j).c, (*j).s, (*j).h);
    return j as *mut ::core::ffi::c_void;
}
#[no_mangle]
// original: cram_decode_slice_mt (htslib/cram/cram_decode.c:3047)
pub unsafe extern "C" fn cram_decode_slice_mt(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut bfd: *mut sam_hdr_t,
) -> ::core::ffi::c_int {
    let mut j: *mut cram_decode_job = ::core::ptr::null_mut::<cram_decode_job>();
    let mut nonblock: ::core::ffi::c_int = 0;
    if (*fd).pool.is_null() {
        return cram_decode_slice(fd, c, s, bfd);
    }
    j = malloc(::core::mem::size_of::<cram_decode_job>() as size_t) as *mut cram_decode_job;
    if j.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*j).fd = fd;
    (*j).c = c;
    (*j).s = s;
    (*j).h = bfd;
    nonblock = if hts_tpool_process_sz((*fd).rqueue) != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut saved_errno: ::core::ffi::c_int = *__errno_location();
    *__errno_location() = 0 as ::core::ffi::c_int;
    if -(1 as ::core::ffi::c_int)
        == hts_tpool_dispatch2(
            (*fd).pool,
            (*fd).rqueue,
            Some(
                cram_decode_slice_thread
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
            ),
            j as *mut ::core::ffi::c_void,
            nonblock,
        )
    {
        if *__errno_location() != EAGAIN {
            return -(1 as ::core::ffi::c_int);
        }
        (*fd).job_pending = j as *mut ::core::ffi::c_void;
    } else {
        (*fd).job_pending = NULL_0;
    }
    *__errno_location() = saved_errno;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
// original: cram_to_bam (htslib/cram/cram_decode.c:3100)
pub unsafe extern "C" fn cram_to_bam(
    mut sh: *mut sam_hdr_t,
    mut fd: *mut cram_fd,
    mut s: *mut cram_slice,
    mut cr: *mut cram_record,
    mut rec: ::core::ffi::c_int,
    mut bam_0: *mut *mut bam_seq_t,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    let mut rg_len: ::core::ffi::c_int = 0;
    let mut name_a: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name_len: ::core::ffi::c_int = 0;
    let mut aux: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut seq: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut qual: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bfd: *mut sam_hrecs_t = (*sh).hrecs;
    if (*fd).required_fields & SAM_QNAME as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if (*cr).name_len != 0 {
            name = ((*(*s).name_blk).data as *mut ::core::ffi::c_char).offset((*cr).name as isize);
            name_len = (*cr).name_len as ::core::ffi::c_int;
        } else {
            name = &raw mut name_a as *mut ::core::ffi::c_char;
            if (*cr).mate_line >= 0 as int32_t
                && (*cr).mate_line < (*s).max_rec as int32_t
                && (*(*s).crecs.offset((*cr).mate_line as isize)).name_len > 0 as int32_t
            {
                memcpy(
                    &raw mut name_a as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    (*(*s).name_blk)
                        .data
                        .offset((*(*s).crecs.offset((*cr).mate_line as isize)).name as isize)
                        as *const ::core::ffi::c_void,
                    (*(*s).crecs.offset((*cr).mate_line as isize)).name_len as size_t,
                );
                name = (&raw mut name_a as *mut ::core::ffi::c_char)
                    .offset((*(*s).crecs.offset((*cr).mate_line as isize)).name_len as isize);
            } else {
                name_len = strlen((*fd).prefix) as ::core::ffi::c_int;
                memcpy(
                    name as *mut ::core::ffi::c_void,
                    (*fd).prefix as *const ::core::ffi::c_void,
                    name_len as size_t,
                );
                name = name.offset(name_len as isize);
                let fresh146 = name;
                name = name.offset(1);
                *fresh146 = ':' as i32 as ::core::ffi::c_char;
                if (*cr).mate_line >= 0 as int32_t && (*cr).mate_line < rec as int32_t {
                    name = append_uint64(
                        name as *mut ::core::ffi::c_uchar,
                        ((*(*s).hdr).record_counter + (*cr).mate_line as int64_t + 1 as int64_t)
                            as uint64_t,
                    ) as *mut ::core::ffi::c_char;
                } else {
                    name = append_uint64(
                        name as *mut ::core::ffi::c_uchar,
                        ((*(*s).hdr).record_counter + rec as int64_t + 1 as int64_t) as uint64_t,
                    ) as *mut ::core::ffi::c_char;
                }
            }
            name_len = name.offset_from(&raw mut name_a as *mut ::core::ffi::c_char)
                as ::core::ffi::c_long as ::core::ffi::c_int;
            name = &raw mut name_a as *mut ::core::ffi::c_char;
        }
    } else {
        name = b"?\0" as *const u8 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        name_len = 1 as ::core::ffi::c_int;
    }
    if (*cr).rg < -(1 as int32_t) || (*cr).rg >= (*bfd).nrg as int32_t {
        return -(1 as ::core::ffi::c_int);
    }
    rg_len = if (*cr).rg != -(1 as int32_t) {
        (*(*bfd).rg.offset((*cr).rg as isize)).name_len + 4 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    if (*fd).required_fields
        & (SAM_SEQ as ::core::ffi::c_int | SAM_QUAL as ::core::ffi::c_int) as ::core::ffi::c_uint
        != 0
    {
        if (*(*s).seqs_blk).data.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        seq = ((*(*s).seqs_blk).data as *mut ::core::ffi::c_char).offset((*cr).seq as isize);
    } else {
        seq = b"*\0" as *const u8 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        (*cr).len = 0 as ::core::ffi::c_int as int32_t;
    }
    if (*fd).required_fields & SAM_QUAL as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if (*(*s).qual_blk).data.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        qual = ((*(*s).qual_blk).data as *mut ::core::ffi::c_char).offset((*cr).qual as isize);
    } else {
        qual = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    ret = bam_set1(
        *bam_0,
        name_len as size_t,
        name,
        (*cr).flags as uint16_t,
        (*cr).ref_id,
        (*cr).apos as hts_pos_t - 1 as hts_pos_t,
        (*cr).mqual as uint8_t,
        (*cr).ncigar as size_t,
        (*s).cigar.offset((*cr).cigar as isize) as *mut uint32_t,
        (*cr).mate_ref_id,
        (*cr).mate_pos as hts_pos_t - 1 as hts_pos_t,
        (*cr).tlen as hts_pos_t,
        (*cr).len as size_t,
        seq,
        qual,
        (*cr).aux_size.wrapping_add(rg_len as uint32_t) as size_t,
    );
    if ret < 0 as ::core::ffi::c_int {
        return ret;
    }
    aux = (**bam_0)
        .data
        .offset(((**bam_0).core.n_cigar << 2 as ::core::ffi::c_int) as isize)
        .offset((**bam_0).core.l_qname as ::core::ffi::c_int as isize)
        .offset(((**bam_0).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int) as isize)
        .offset((**bam_0).core.l_qseq as isize) as *mut ::core::ffi::c_char;
    if (*cr).aux_size != 0 as uint32_t {
        memcpy(
            aux as *mut ::core::ffi::c_void,
            (*(*s).aux_blk).data.offset((*cr).aux as isize) as *const ::core::ffi::c_void,
            (*cr).aux_size as size_t,
        );
        aux = aux.offset((*cr).aux_size as isize);
        (**bam_0).l_data = ((**bam_0).l_data as ::core::ffi::c_uint)
            .wrapping_add((*cr).aux_size as ::core::ffi::c_uint)
            as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    if rg_len > 0 as ::core::ffi::c_int {
        let fresh147 = aux;
        aux = aux.offset(1);
        *fresh147 = 'R' as i32 as ::core::ffi::c_char;
        let fresh148 = aux;
        aux = aux.offset(1);
        *fresh148 = 'G' as i32 as ::core::ffi::c_char;
        let fresh149 = aux;
        aux = aux.offset(1);
        *fresh149 = 'Z' as i32 as ::core::ffi::c_char;
        let mut len: ::core::ffi::c_int = (*(*bfd).rg.offset((*cr).rg as isize)).name_len;
        memcpy(
            aux as *mut ::core::ffi::c_void,
            (*(*bfd).rg.offset((*cr).rg as isize)).name as *const ::core::ffi::c_void,
            len as size_t,
        );
        aux = aux.offset(len as isize);
        let fresh150 = aux;
        aux = aux.offset(1);
        *fresh150 = 0 as ::core::ffi::c_char;
        (**bam_0).l_data += rg_len;
    }
    return (**bam_0).l_data;
}
// original: cram_first_slice (htslib/cram/cram_decode.c:3212)
unsafe extern "C" fn cram_first_slice(mut fd: *mut cram_fd) -> *mut cram_container {
    let mut c: *mut cram_container = ::core::ptr::null_mut::<cram_container>();
    loop {
        if !(*fd).ctr.is_null() {
            cram_free_container((*fd).ctr);
        }
        (*fd).ctr = cram_read_container(fd);
        c = (*fd).ctr;
        if c.is_null() {
            return ::core::ptr::null_mut::<cram_container>();
        }
        (*c).curr_slice_mt = (*c).curr_slice;
        if !((*c).length == 0 as int32_t) {
            break;
        }
    }
    if (*fd).range.refid != -(2 as ::core::ffi::c_int) {
        while (*c).ref_seq_id != -(2 as int32_t)
            && ((*c).ref_seq_id < (*fd).range.refid as int32_t
                || (*fd).range.refid >= 0 as ::core::ffi::c_int
                    && (*c).ref_seq_id == (*fd).range.refid as int32_t
                    && ((*c).ref_seq_start + (*c).ref_seq_span - 1 as int64_t) < (*fd).range.start)
        {
            if 0 as ::core::ffi::c_int != cram_seek(fd, (*c).length as off_t, SEEK_CUR) {
                return ::core::ptr::null_mut::<cram_container>();
            }
            cram_free_container((*fd).ctr);
            loop {
                (*fd).ctr = cram_read_container(fd);
                c = (*fd).ctr;
                if c.is_null() {
                    return ::core::ptr::null_mut::<cram_container>();
                }
                if !((*c).length == 0 as int32_t) {
                    break;
                }
            }
        }
        if (*c).ref_seq_id != -(2 as int32_t) && (*c).ref_seq_id != (*fd).range.refid as int32_t {
            (*fd).eof = 1 as ::core::ffi::c_int;
            return ::core::ptr::null_mut::<cram_container>();
        }
    }
    (*c).comp_hdr_block = cram_read_block(fd);
    if (*c).comp_hdr_block.is_null() {
        return ::core::ptr::null_mut::<cram_container>();
    }
    if (*(*c).comp_hdr_block).content_type as ::core::ffi::c_int
        != COMPRESSION_HEADER as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<cram_container>();
    }
    (*c).comp_hdr = cram_decode_compression_header(fd, (*c).comp_hdr_block);
    if (*c).comp_hdr.is_null() {
        return ::core::ptr::null_mut::<cram_container>();
    }
    if (*(*c).comp_hdr).AP_delta == 0
        && sam_hrecs_sort_order((*(*fd).header).hrecs) as ::core::ffi::c_int
            != ORDER_COORD as ::core::ffi::c_int
    {
        pthread_mutex_lock(&raw mut (*fd).ref_lock);
        (*fd).unsorted = 1 as ::core::ffi::c_int;
        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
    }
    return c;
}
#[no_mangle]
// original: cram_next_slice (htslib/cram/cram_decode.c:3268)
pub unsafe extern "C" fn cram_next_slice(
    mut fd: *mut cram_fd,
    mut cp: *mut *mut cram_container,
) -> *mut cram_slice {
    let mut c_curr: *mut cram_container = ::core::ptr::null_mut::<cram_container>();
    let mut s_curr: *mut cram_slice = ::core::ptr::null_mut::<cram_slice>();
    c_curr = (*fd).ctr;
    if c_curr.is_null() {
        c_curr = cram_first_slice(fd);
        if c_curr.is_null() {
            return ::core::ptr::null_mut::<cram_slice>();
        }
    }
    s_curr = (*c_curr).slice as *mut cram_slice;
    if !s_curr.is_null() {
        (*c_curr).slice = ::core::ptr::null_mut::<cram_slice>();
        cram_free_slice(s_curr);
        s_curr = ::core::ptr::null_mut::<cram_slice>();
    }
    if (*c_curr).curr_slice == (*c_curr).max_slice {
        if (*fd).ctr == c_curr {
            (*fd).ctr = ::core::ptr::null_mut::<cram_container>();
        }
        if (*fd).ctr_mt == c_curr {
            (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
        }
        cram_free_container(c_curr);
        c_curr = ::core::ptr::null_mut::<cram_container>();
    }
    if (*fd).ctr_mt.is_null() {
        (*fd).ctr_mt = c_curr;
    }
    let mut current_block_82: u64;
    's_83: loop {
        let mut c_next: *mut cram_container = (*fd).ctr_mt;
        let mut s_next: *mut cram_slice = ::core::ptr::null_mut::<cram_slice>();
        if !(*fd).job_pending.is_null() {
            let mut j: *mut cram_decode_job = (*fd).job_pending as *mut cram_decode_job;
            c_next = (*j).c;
            s_next = (*j).s;
            free((*fd).job_pending);
            (*fd).job_pending = NULL_0;
        } else if (*fd).ooc == 0 {
            loop {
                if c_next.is_null() || (*c_next).curr_slice_mt == (*c_next).max_slice {
                    loop {
                        c_next = cram_read_container(fd);
                        if c_next.is_null() {
                            if !(*fd).pool.is_null() {
                                (*fd).ooc = 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                return ::core::ptr::null_mut::<cram_slice>();
                            }
                        } else {
                            (*c_next).curr_slice_mt = (*c_next).curr_slice;
                            if (*c_next).length != 0 as int32_t {
                                break;
                            }
                            cram_free_container(c_next);
                        }
                    }
                    if (*fd).ooc != 0 {
                        break 's_83;
                    }
                    if (*fd).range.refid != -(2 as ::core::ffi::c_int)
                        && (*c_next).ref_seq_id != -(2 as int32_t)
                    {
                        if (*c_next).ref_seq_id != (*fd).range.refid as int32_t {
                            cram_free_container(c_next);
                            (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
                            (*fd).ooc = 1 as ::core::ffi::c_int;
                            break 's_83;
                        } else if (*fd).range.refid != -(1 as ::core::ffi::c_int)
                            && (*c_next).ref_seq_start > (*fd).range.end
                        {
                            cram_free_container(c_next);
                            (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
                            (*fd).ooc = 1 as ::core::ffi::c_int;
                            break 's_83;
                        } else if (*fd).range.refid != -(1 as ::core::ffi::c_int)
                            && ((*c_next).ref_seq_start + (*c_next).ref_seq_span - 1 as int64_t)
                                < (*fd).range.start
                        {
                            let mut skip_length: off_t = (*c_next).length as off_t;
                            cram_free_container(c_next);
                            c_next = ::core::ptr::null_mut::<cram_container>();
                            (*fd).ooc = 0 as ::core::ffi::c_int;
                            if hseek((*fd).fp as *mut hFILE, skip_length, SEEK_CUR)
                                < 0 as ::core::ffi::c_long
                            {
                                return ::core::ptr::null_mut::<cram_slice>();
                            }
                            continue 's_83;
                        }
                    }
                    (*fd).ctr_mt = c_next;
                    (*c_next).comp_hdr_block = cram_read_block(fd);
                    if (*c_next).comp_hdr_block.is_null() {
                        return ::core::ptr::null_mut::<cram_slice>();
                    }
                    if (*(*c_next).comp_hdr_block).content_type as ::core::ffi::c_int
                        != COMPRESSION_HEADER as ::core::ffi::c_int
                    {
                        return ::core::ptr::null_mut::<cram_slice>();
                    }
                    (*c_next).comp_hdr =
                        cram_decode_compression_header(fd, (*c_next).comp_hdr_block);
                    if (*c_next).comp_hdr.is_null() {
                        return ::core::ptr::null_mut::<cram_slice>();
                    }
                    if (*(*c_next).comp_hdr).AP_delta == 0
                        && sam_hrecs_sort_order((*(*fd).header).hrecs) as ::core::ffi::c_int
                            != ORDER_COORD as ::core::ffi::c_int
                    {
                        pthread_mutex_lock(&raw mut (*fd).ref_lock);
                        (*fd).unsorted = 1 as ::core::ffi::c_int;
                        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                    }
                }
                if (*c_next).num_records == 0 as int32_t {
                    if (*fd).ctr == c_next {
                        (*fd).ctr = ::core::ptr::null_mut::<cram_container>();
                    }
                    if c_curr == c_next {
                        c_curr = ::core::ptr::null_mut::<cram_container>();
                    }
                    if (*fd).ctr_mt == c_next {
                        (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
                    }
                    cram_free_container(c_next);
                    c_next = ::core::ptr::null_mut::<cram_container>();
                } else {
                    (*c_next).slice = cram_read_slice(fd) as *mut cram_slice;
                    s_next = (*c_next).slice as *mut cram_slice;
                    if s_next.is_null() {
                        return ::core::ptr::null_mut::<cram_slice>();
                    }
                    (*c_next).curr_slice_mt += 1;
                    (*s_next).slice_num = (*c_next).curr_slice_mt;
                    (*s_next).curr_rec = 0 as ::core::ffi::c_int;
                    (*s_next).max_rec = (*(*s_next).hdr).num_records as ::core::ffi::c_int;
                    (*s_next).last_apos = (*(*s_next).hdr).ref_seq_start;
                    if (*fd).range.refid != -(2 as ::core::ffi::c_int)
                        && (*(*s_next).hdr).ref_seq_id != -(2 as int32_t)
                    {
                        current_block_82 = 8869332144787829186;
                        break;
                    } else {
                        current_block_82 = 5854763015135596753;
                        break;
                    }
                }
            }
            match current_block_82 {
                5854763015135596753 => {}
                _ => {
                    if (*(*s_next).hdr).ref_seq_id != (*fd).range.refid as int32_t {
                        (*fd).ooc = 1 as ::core::ffi::c_int;
                        cram_free_slice(s_next);
                        s_next = ::core::ptr::null_mut::<cram_slice>();
                        (*c_next).slice = s_next as *mut cram_slice;
                        break;
                    } else if (*fd).range.refid != -(1 as ::core::ffi::c_int)
                        && (*(*s_next).hdr).ref_seq_start > (*fd).range.end
                    {
                        (*fd).ooc = 1 as ::core::ffi::c_int;
                        cram_free_slice(s_next);
                        s_next = ::core::ptr::null_mut::<cram_slice>();
                        (*c_next).slice = s_next as *mut cram_slice;
                        break;
                    } else if (*fd).range.refid != -(1 as ::core::ffi::c_int)
                        && ((*(*s_next).hdr).ref_seq_start + (*(*s_next).hdr).ref_seq_span
                            - 1 as int64_t)
                            < (*fd).range.start
                    {
                        cram_free_slice(s_next);
                        s_next = ::core::ptr::null_mut::<cram_slice>();
                        (*c_next).slice = s_next as *mut cram_slice;
                        continue;
                    }
                }
            }
        }
        if c_next.is_null() || s_next.is_null() {
            break;
        }
        if cram_decode_slice_mt(fd, c_next, s_next, (*fd).header) != 0 as ::core::ffi::c_int {
            hts_log(
                HTS_LOG_ERROR,
                b"cram_next_slice\0" as *const u8 as *const ::core::ffi::c_char,
                b"Failure to decode slice\0" as *const u8 as *const ::core::ffi::c_char,
            );
            cram_free_slice(s_next);
            (*c_next).slice = ::core::ptr::null_mut::<cram_slice>();
            return ::core::ptr::null_mut::<cram_slice>();
        }
        if (*fd).pool.is_null() {
            c_curr = c_next;
            s_curr = s_next;
            break;
        } else {
            if !(*fd).job_pending.is_null() {
                break;
            }
            if hts_tpool_process_len((*fd).rqueue) > hts_tpool_process_qsize((*fd).rqueue) {
                break;
            }
        }
    }
    if !(*fd).pool.is_null() {
        let mut res: *mut hts_tpool_result = ::core::ptr::null_mut::<hts_tpool_result>();
        let mut j_0: *mut cram_decode_job = ::core::ptr::null_mut::<cram_decode_job>();
        if (*fd).ooc != 0 && hts_tpool_process_empty((*fd).rqueue) != 0 {
            (*fd).eof = 1 as ::core::ffi::c_int;
            return ::core::ptr::null_mut::<cram_slice>();
        }
        res = hts_tpool_next_result_wait((*fd).rqueue);
        if res.is_null() || hts_tpool_result_data(res).is_null() {
            hts_log(
                HTS_LOG_ERROR,
                b"cram_next_slice\0" as *const u8 as *const ::core::ffi::c_char,
                b"Call to hts_tpool_next_result failed\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<cram_slice>();
        }
        j_0 = hts_tpool_result_data(res) as *mut cram_decode_job;
        c_curr = (*j_0).c;
        s_curr = (*j_0).s;
        if (*j_0).exit_code != 0 as ::core::ffi::c_int {
            hts_log(
                HTS_LOG_ERROR,
                b"cram_next_slice\0" as *const u8 as *const ::core::ffi::c_char,
                b"Slice decode failure\0" as *const u8 as *const ::core::ffi::c_char,
            );
            (*fd).eof = 0 as ::core::ffi::c_int;
            hts_tpool_delete_result(res, 1 as ::core::ffi::c_int);
            return ::core::ptr::null_mut::<cram_slice>();
        }
        hts_tpool_delete_result(res, 1 as ::core::ffi::c_int);
    }
    *cp = c_curr;
    (*fd).ctr = c_curr;
    if !c_curr.is_null() {
        (*c_curr).slice = s_curr as *mut cram_slice;
        if !s_curr.is_null() {
            (*c_curr).curr_slice = (*s_curr).slice_num;
        }
    }
    if !s_curr.is_null() {
        (*s_curr).curr_rec = 0 as ::core::ffi::c_int;
    } else {
        (*fd).eof = 1 as ::core::ffi::c_int;
    }
    return s_curr;
}
#[no_mangle]
// original: cram_get_seq (htslib/cram/cram_decode.c:3549)
pub unsafe extern "C" fn cram_get_seq(mut fd: *mut cram_fd) -> *mut cram_record {
    let mut c: *mut cram_container = ::core::ptr::null_mut::<cram_container>();
    let mut s: *mut cram_slice = ::core::ptr::null_mut::<cram_slice>();
    loop {
        c = (*fd).ctr;
        if !c.is_null() && !(*c).slice.is_null() && (*(*c).slice).curr_rec < (*(*c).slice).max_rec {
            s = (*c).slice as *mut cram_slice;
            if !((*fd).range.refid != -(2 as ::core::ffi::c_int)) {
                break;
            }
            if (*fd).range.refid == -(1 as ::core::ffi::c_int)
                && (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id != -(1 as int32_t)
            {
                (*s).curr_rec += 1;
            } else if (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id
                < (*fd).range.refid as int32_t
                && (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id != -(1 as int32_t)
            {
                (*s).curr_rec += 1;
            } else {
                if (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id
                    != (*fd).range.refid as int32_t
                {
                    (*fd).eof = 1 as ::core::ffi::c_int;
                    cram_free_slice(s);
                    (*c).slice = ::core::ptr::null_mut::<cram_slice>();
                    return ::core::ptr::null_mut::<cram_record>();
                }
                if (*fd).range.refid != -(1 as ::core::ffi::c_int)
                    && (*(*s).crecs.offset((*s).curr_rec as isize)).apos > (*fd).range.end
                {
                    (*fd).eof = 1 as ::core::ffi::c_int;
                    cram_free_slice(s);
                    (*c).slice = ::core::ptr::null_mut::<cram_slice>();
                    return ::core::ptr::null_mut::<cram_record>();
                }
                if !((*fd).range.refid != -(1 as ::core::ffi::c_int)
                    && (*(*s).crecs.offset((*s).curr_rec as isize)).aend < (*fd).range.start)
                {
                    break;
                }
                (*s).curr_rec += 1;
            }
        } else {
            s = cram_next_slice(fd, &raw mut c);
            if s.is_null() {
                return ::core::ptr::null_mut::<cram_record>();
            }
        }
    }
    (*fd).ctr = c;
    (*c).slice = s as *mut cram_slice;
    let fresh127 = (*s).curr_rec;
    (*s).curr_rec = (*s).curr_rec + 1;
    return (*s).crecs.offset(fresh127 as isize) as *mut cram_record;
}
#[no_mangle]
// original: cram_get_bam_seq (htslib/cram/cram_decode.c:3615)
pub unsafe extern "C" fn cram_get_bam_seq(
    mut fd: *mut cram_fd,
    mut bam_0: *mut *mut bam_seq_t,
) -> ::core::ffi::c_int {
    let mut cr: *mut cram_record = ::core::ptr::null_mut::<cram_record>();
    let mut c: *mut cram_container = ::core::ptr::null_mut::<cram_container>();
    let mut s: *mut cram_slice = ::core::ptr::null_mut::<cram_slice>();
    cr = cram_get_seq(fd);
    if cr.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    c = (*fd).ctr;
    s = (*c).slice as *mut cram_slice;
    return cram_to_bam(
        (*fd).header,
        fd,
        s,
        cr,
        (*s).curr_rec - 1 as ::core::ffi::c_int,
        bam_0,
    );
}
#[no_mangle]
// original: cram_drain_rqueue (htslib/cram/cram_decode.c:3632)
pub unsafe extern "C" fn cram_drain_rqueue(mut fd: *mut cram_fd) {
    let mut lc: *mut cram_container = ::core::ptr::null_mut::<cram_container>();
    if (*fd).pool.is_null() || (*fd).rqueue.is_null() {
        return;
    }
    while hts_tpool_process_empty((*fd).rqueue) == 0 {
        let mut r: *mut hts_tpool_result = hts_tpool_next_result_wait((*fd).rqueue);
        if r.is_null() {
            break;
        }
        let mut j: *mut cram_decode_job = hts_tpool_result_data(r) as *mut cram_decode_job;
        if (*(*j).c).slice == (*j).s {
            (*(*j).c).slice = ::core::ptr::null_mut::<cram_slice>();
        }
        if (*j).c != lc {
            if !lc.is_null() {
                if (*fd).ctr == lc {
                    (*fd).ctr = ::core::ptr::null_mut::<cram_container>();
                }
                if (*fd).ctr_mt == lc {
                    (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
                }
                cram_free_container(lc);
            }
            lc = (*j).c;
        }
        cram_free_slice((*j).s);
        hts_tpool_delete_result(r, 1 as ::core::ffi::c_int);
    }
    if !(*fd).job_pending.is_null() {
        let mut j_0: *mut cram_decode_job = (*fd).job_pending as *mut cram_decode_job;
        if (*(*j_0).c).slice == (*j_0).s {
            (*(*j_0).c).slice = ::core::ptr::null_mut::<cram_slice>();
        }
        if (*j_0).c != lc {
            if !lc.is_null() {
                if (*fd).ctr == lc {
                    (*fd).ctr = ::core::ptr::null_mut::<cram_container>();
                }
                if (*fd).ctr_mt == lc {
                    (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
                }
                cram_free_container(lc);
            }
            lc = (*j_0).c;
        }
        cram_free_slice((*j_0).s);
        free(j_0 as *mut ::core::ffi::c_void);
        (*fd).job_pending = NULL_0;
    }
    if !lc.is_null() {
        if (*fd).ctr == lc {
            (*fd).ctr = ::core::ptr::null_mut::<cram_container>();
        }
        if (*fd).ctr_mt == lc {
            (*fd).ctr_mt = ::core::ptr::null_mut::<cram_container>();
        }
        cram_free_container(lc);
    }
}
#[inline]
unsafe extern "C" fn cram_get_block_by_id(
    mut slice: *mut cram_slice,
    mut id: ::core::ffi::c_int,
) -> *mut cram_block {
    let mut v: uint32_t = id as uint32_t;
    if !(*slice).block_by_id.is_null() && v < 256 as uint32_t {
        return *(*slice).block_by_id.offset(v as isize);
    } else {
        v = (256 as uint32_t).wrapping_add(v.wrapping_rem(251 as uint32_t));
        if !(*slice).block_by_id.is_null()
            && !(*(*slice).block_by_id.offset(v as isize)).is_null()
            && (**(*slice).block_by_id.offset(v as isize)).content_id == id as int32_t
        {
            return *(*slice).block_by_id.offset(v as isize);
        }
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*(*slice).hdr).num_blocks {
            let mut b: *mut cram_block = *(*slice).block.offset(i as isize);
            if !b.is_null()
                && (*b).content_type as ::core::ffi::c_int == EXTERNAL as ::core::ffi::c_int
                && (*b).content_id == id as int32_t
            {
                return b;
            }
            i += 1;
        }
    }
    return ::core::ptr::null_mut::<cram_block>();
}
#[inline]
unsafe extern "C" fn block_resize_exact(
    mut b: *mut cram_block,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut tmp: *mut ::core::ffi::c_uchar =
        realloc((*b).data as *mut ::core::ffi::c_void, len) as *mut ::core::ffi::c_uchar;
    if tmp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*b).alloc = len;
    (*b).data = tmp;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn block_resize(mut b: *mut cram_block, mut len: size_t) -> ::core::ffi::c_int {
    if (*b).alloc > len {
        return 0 as ::core::ffi::c_int;
    }
    let mut alloc: size_t = (*b).alloc.wrapping_add(800 as size_t);
    alloc = if alloc.wrapping_add(alloc >> 2 as ::core::ffi::c_int) > len {
        alloc.wrapping_add(alloc >> 2 as ::core::ffi::c_int)
    } else {
        len
    };
    return block_resize_exact(b, alloc);
}
#[inline]
unsafe extern "C" fn block_grow(mut b: *mut cram_block, mut len: size_t) -> ::core::ffi::c_int {
    return block_resize(b, (*b).byte.wrapping_add(len));
}
#[inline]
unsafe extern "C" fn block_append(
    mut b: *mut cram_block,
    mut s: *const ::core::ffi::c_void,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if block_grow(b, len) < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    if len != 0 {
        memcpy(
            (*b).data.offset((*b).byte as isize) as *mut ::core::ffi::c_uchar
                as *mut ::core::ffi::c_void,
            s,
            len,
        );
        (*b).byte = ((*b).byte as ::core::ffi::c_ulong).wrapping_add(len as ::core::ffi::c_ulong)
            as size_t as size_t;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn block_append_char(
    mut b: *mut cram_block,
    mut c: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if block_grow(b, 1 as size_t) < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    let fresh50 = (*b).byte;
    (*b).byte = (*b).byte.wrapping_add(1);
    *(*b).data.offset(fresh50 as isize) = c as ::core::ffi::c_uchar;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn block_append_uint(
    mut b: *mut cram_block,
    mut i: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    if block_grow(b, 11 as size_t) < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    let mut cp: *mut ::core::ffi::c_uchar =
        (*b).data.offset((*b).byte as isize) as *mut ::core::ffi::c_uchar;
    (*b).byte = ((*b).byte as ::core::ffi::c_ulong).wrapping_add(
        append_uint32(cp, i as uint32_t).offset_from(cp) as ::core::ffi::c_long
            as ::core::ffi::c_ulong,
    ) as size_t as size_t;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn append_uint32(
    mut cp: *mut ::core::ffi::c_uchar,
    mut i: uint32_t,
) -> *mut ::core::ffi::c_uchar {
    let mut current_block: u64;
    let mut j: uint32_t = 0;
    if i == 0 as uint32_t {
        let fresh98 = cp;
        cp = cp.offset(1);
        *fresh98 = '0' as i32 as ::core::ffi::c_uchar;
        return cp;
    }
    if i < 100 as uint32_t {
        current_block = 18006962674404780693;
    } else {
        if i < 10000 as uint32_t {
            current_block = 6375830885315177385;
        } else {
            if i < 1000000 as uint32_t {
                current_block = 10608955196113400758;
            } else {
                if i < 100000000 as uint32_t {
                    current_block = 3608731184299887663;
                } else {
                    j = i.wrapping_div(1000000000 as uint32_t);
                    if j != 0 {
                        let fresh99 = cp;
                        cp = cp.offset(1);
                        *fresh99 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint).wrapping_sub(
                            j.wrapping_mul(1000000000 as uint32_t) as ::core::ffi::c_uint,
                        ) as uint32_t as uint32_t;
                        let fresh109 = cp;
                        cp = cp.offset(1);
                        *fresh109 = i
                            .wrapping_div(100000000 as uint32_t)
                            .wrapping_add('0' as i32 as uint32_t)
                            as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_rem(100000000 as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 11658067726481655461;
                    } else {
                        j = i.wrapping_div(100000000 as uint32_t);
                        if j != 0 {
                            let fresh100 = cp;
                            cp = cp.offset(1);
                            *fresh100 =
                                j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                            i =
                                (i as ::core::ffi::c_uint)
                                    .wrapping_sub(j.wrapping_mul(100000000 as uint32_t)
                                        as ::core::ffi::c_uint)
                                    as uint32_t as uint32_t;
                            current_block = 11658067726481655461;
                        } else {
                            current_block = 3608731184299887663;
                        }
                    }
                    match current_block {
                        3608731184299887663 => {}
                        _ => {
                            let fresh110 = cp;
                            cp = cp.offset(1);
                            *fresh110 = i
                                .wrapping_div(10000000 as uint32_t)
                                .wrapping_add('0' as i32 as uint32_t)
                                as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint)
                                .wrapping_rem(10000000 as ::core::ffi::c_int as ::core::ffi::c_uint)
                                as uint32_t as uint32_t;
                            current_block = 15952379835070595718;
                        }
                    }
                }
                match current_block {
                    3608731184299887663 => {
                        j = i.wrapping_div(10000000 as uint32_t);
                        if j != 0 {
                            let fresh101 = cp;
                            cp = cp.offset(1);
                            *fresh101 =
                                j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint).wrapping_sub(
                                j.wrapping_mul(10000000 as uint32_t) as ::core::ffi::c_uint,
                            ) as uint32_t as uint32_t;
                            current_block = 15952379835070595718;
                        } else {
                            j = i.wrapping_div(1000000 as uint32_t);
                            if j != 0 {
                                let fresh102 = cp;
                                cp = cp.offset(1);
                                *fresh102 =
                                    j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                                i =
                                    (i as ::core::ffi::c_uint)
                                        .wrapping_sub(j.wrapping_mul(1000000 as uint32_t)
                                            as ::core::ffi::c_uint)
                                        as uint32_t as uint32_t;
                                current_block = 5888676067671508684;
                            } else {
                                current_block = 10608955196113400758;
                            }
                        }
                    }
                    _ => {}
                }
                match current_block {
                    10608955196113400758 => {}
                    _ => {
                        match current_block {
                            15952379835070595718 => {
                                let fresh111 = cp;
                                cp = cp.offset(1);
                                *fresh111 = i
                                    .wrapping_div(1000000 as uint32_t)
                                    .wrapping_add('0' as i32 as uint32_t)
                                    as ::core::ffi::c_uchar;
                                i = (i as ::core::ffi::c_uint).wrapping_rem(
                                    1000000 as ::core::ffi::c_int as ::core::ffi::c_uint,
                                ) as uint32_t as uint32_t;
                            }
                            _ => {}
                        }
                        let fresh112 = cp;
                        cp = cp.offset(1);
                        *fresh112 = i
                            .wrapping_div(100000 as uint32_t)
                            .wrapping_add('0' as i32 as uint32_t)
                            as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_rem(100000 as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 13965095521321759454;
                    }
                }
            }
            match current_block {
                10608955196113400758 => {
                    j = i.wrapping_div(100000 as uint32_t);
                    if j != 0 {
                        let fresh103 = cp;
                        cp = cp.offset(1);
                        *fresh103 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_sub(j.wrapping_mul(100000 as uint32_t) as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 13965095521321759454;
                    } else {
                        j = i.wrapping_div(10000 as uint32_t);
                        if j != 0 {
                            let fresh104 = cp;
                            cp = cp.offset(1);
                            *fresh104 =
                                j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint).wrapping_sub(
                                j.wrapping_mul(10000 as uint32_t) as ::core::ffi::c_uint,
                            ) as uint32_t as uint32_t;
                            current_block = 17433961600056345253;
                        } else {
                            current_block = 6375830885315177385;
                        }
                    }
                }
                _ => {}
            }
            match current_block {
                6375830885315177385 => {}
                _ => {
                    match current_block {
                        13965095521321759454 => {
                            let fresh113 = cp;
                            cp = cp.offset(1);
                            *fresh113 = i
                                .wrapping_div(10000 as uint32_t)
                                .wrapping_add('0' as i32 as uint32_t)
                                as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint)
                                .wrapping_rem(10000 as ::core::ffi::c_uint)
                                as uint32_t as uint32_t;
                        }
                        _ => {}
                    }
                    let fresh114 = cp;
                    cp = cp.offset(1);
                    *fresh114 = i
                        .wrapping_div(1000 as uint32_t)
                        .wrapping_add('0' as i32 as uint32_t)
                        as ::core::ffi::c_uchar;
                    i = (i as ::core::ffi::c_uint).wrapping_rem(1000 as ::core::ffi::c_uint)
                        as uint32_t as uint32_t;
                    current_block = 11701269959192775854;
                }
            }
        }
        match current_block {
            6375830885315177385 => {
                j = i.wrapping_div(1000 as uint32_t);
                if j != 0 {
                    let fresh105 = cp;
                    cp = cp.offset(1);
                    *fresh105 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                    i = (i as ::core::ffi::c_uint)
                        .wrapping_sub(j.wrapping_mul(1000 as uint32_t) as ::core::ffi::c_uint)
                        as uint32_t as uint32_t;
                    current_block = 11701269959192775854;
                } else {
                    j = i.wrapping_div(100 as uint32_t);
                    if j != 0 {
                        let fresh106 = cp;
                        cp = cp.offset(1);
                        *fresh106 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_sub(j.wrapping_mul(100 as uint32_t) as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 3183587366372611312;
                    } else {
                        current_block = 18006962674404780693;
                    }
                }
            }
            _ => {}
        }
        match current_block {
            18006962674404780693 => {}
            _ => {
                match current_block {
                    11701269959192775854 => {
                        let fresh115 = cp;
                        cp = cp.offset(1);
                        *fresh115 = i
                            .wrapping_div(100 as uint32_t)
                            .wrapping_add('0' as i32 as uint32_t)
                            as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint).wrapping_rem(100 as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                    }
                    _ => {}
                }
                let fresh116 = cp;
                cp = cp.offset(1);
                *fresh116 = i
                    .wrapping_div(10 as uint32_t)
                    .wrapping_add('0' as i32 as uint32_t)
                    as ::core::ffi::c_uchar;
                i = (i as ::core::ffi::c_uint).wrapping_rem(10 as ::core::ffi::c_uint) as uint32_t
                    as uint32_t;
                current_block = 9979761792777686870;
            }
        }
    }
    match current_block {
        18006962674404780693 => {
            j = i.wrapping_div(10 as uint32_t);
            if j != 0 {
                let fresh107 = cp;
                cp = cp.offset(1);
                *fresh107 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                i = (i as ::core::ffi::c_uint)
                    .wrapping_sub(j.wrapping_mul(10 as uint32_t) as ::core::ffi::c_uint)
                    as uint32_t as uint32_t;
            } else {
                if i != 0 {
                    let fresh108 = cp;
                    cp = cp.offset(1);
                    *fresh108 = i.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                }
                return cp;
            }
        }
        _ => {}
    }
    let fresh117 = cp;
    cp = cp.offset(1);
    *fresh117 = i.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    return cp;
}
#[inline]
unsafe extern "C" fn append_sub32(
    mut cp: *mut ::core::ffi::c_uchar,
    mut i: uint32_t,
) -> *mut ::core::ffi::c_uchar {
    let fresh118 = cp;
    cp = cp.offset(1);
    *fresh118 = i
        .wrapping_div(100000000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint)
        .wrapping_rem(100000000 as ::core::ffi::c_int as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh119 = cp;
    cp = cp.offset(1);
    *fresh119 = i
        .wrapping_div(10000000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint)
        .wrapping_rem(10000000 as ::core::ffi::c_int as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh120 = cp;
    cp = cp.offset(1);
    *fresh120 = i
        .wrapping_div(1000000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint)
        .wrapping_rem(1000000 as ::core::ffi::c_int as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh121 = cp;
    cp = cp.offset(1);
    *fresh121 = i
        .wrapping_div(100000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(100000 as ::core::ffi::c_int as ::core::ffi::c_uint)
        as uint32_t as uint32_t;
    let fresh122 = cp;
    cp = cp.offset(1);
    *fresh122 = i
        .wrapping_div(10000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(10000 as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh123 = cp;
    cp = cp.offset(1);
    *fresh123 = i
        .wrapping_div(1000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(1000 as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh124 = cp;
    cp = cp.offset(1);
    *fresh124 = i
        .wrapping_div(100 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(100 as ::core::ffi::c_uint) as uint32_t as uint32_t;
    let fresh125 = cp;
    cp = cp.offset(1);
    *fresh125 = i
        .wrapping_div(10 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(10 as ::core::ffi::c_uint) as uint32_t as uint32_t;
    let fresh126 = cp;
    cp = cp.offset(1);
    *fresh126 = i.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    return cp;
}
#[inline]
unsafe extern "C" fn append_uint64(
    mut cp: *mut ::core::ffi::c_uchar,
    mut i: uint64_t,
) -> *mut ::core::ffi::c_uchar {
    let mut j: uint64_t = 0;
    if i <= 0xffffffff as uint64_t {
        return append_uint32(cp, i as uint32_t);
    }
    j = i.wrapping_div(1000000000 as uint64_t);
    if j > 1000000000 as uint64_t {
        cp = append_uint32(cp, j.wrapping_div(1000000000 as uint64_t) as uint32_t);
        j = (j as ::core::ffi::c_ulong).wrapping_rem(1000000000 as ::core::ffi::c_ulong) as uint64_t
            as uint64_t;
        cp = append_sub32(cp, j as uint32_t);
    } else {
        cp = append_uint32(cp, i.wrapping_div(1000000000 as uint64_t) as uint32_t);
    }
    cp = append_sub32(cp, i.wrapping_rem(1000000000 as uint64_t) as uint32_t);
    return cp;
}
#[inline]
unsafe extern "C" fn cram_hfile(mut fd: *mut cram_fd) -> *mut hFILE {
    return (*fd).fp;
}
pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EAGAIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EFAULT: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

// original: nbits (htslib/cram/cram_decode.c:1051)
pub unsafe fn cram_cram_decode_c_1051_nbits(mut v: ::core::ffi::c_int) -> ::core::ffi::c_int {
    const MULTIPLY_DEBRUIJN_BIT_POSITION: [::core::ffi::c_int; 32] = [
        1, 10, 2, 11, 14, 22, 3, 30, 12, 15, 17, 19, 23, 26, 4, 31, 9, 13, 21, 29, 16, 18, 25, 8,
        20, 28, 24, 7, 27, 6, 5, 32,
    ];

    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;

    MULTIPLY_DEBRUIJN_BIT_POSITION[((v as u32).wrapping_mul(0x07C4_ACDD) >> 27) as usize]
}

// original: sort_freqs (htslib/cram/cram_decode.c:1069)
pub unsafe fn cram_cram_decode_c_1069_sort_freqs(
    vp1: *const ::core::ffi::c_void,
    vp2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let i1 = *vp1.cast::<::core::ffi::c_int>();
    let i2 = *vp2.cast::<::core::ffi::c_int>();
    i1 - i2
}
