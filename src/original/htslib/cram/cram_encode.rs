// Faithful Rust translation of htslib/cram/cram_encode.c generated from the vendored HTSlib source.
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type BGZF;
    pub type hts_tpool_process;
    pub type hts_tpool;
    pub type hFILE;
    pub type hts_idx_t;
    pub type hts_filter_t;
    pub type hts_md5_context;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn llabs(__x: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn tolower(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn toupper(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
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
    static seq_nt16_str: [::core::ffi::c_char; 0];
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
    fn hts_md5_hex(hex: *mut ::core::ffi::c_char, digest: *const ::core::ffi::c_uchar);
    fn hts_md5_destroy(ctx: *mut hts_md5_context);
    fn sam_hdr_init() -> *mut sam_hdr_t;
    fn sam_hdr_destroy(h: *mut sam_hdr_t);
    fn sam_hdr_dup(h0: *const sam_hdr_t) -> *mut sam_hdr_t;
    fn sam_hdr_parse(l_text: size_t, text: *const ::core::ffi::c_char) -> *mut sam_hdr_t;
    fn sam_hdr_name2tid(h: *mut sam_hdr_t, ref_0: *const ::core::ffi::c_char)
        -> ::core::ffi::c_int;
    fn bam_copy1(bdst: *mut bam1_t, bsrc: *const bam1_t) -> *mut bam1_t;
    fn bam_dup1(bsrc: *const bam1_t) -> *mut bam1_t;
    fn bam_cigar2qlen(n_cigar: ::core::ffi::c_int, cigar: *const uint32_t) -> hts_pos_t;
    fn bam_cigar2rlen(n_cigar: ::core::ffi::c_int, cigar: *const uint32_t) -> hts_pos_t;
    fn bam_aux_get(b: *const bam1_t, tag: *const ::core::ffi::c_char) -> *mut uint8_t;
    fn cram_new_block(
        content_type: cram_content_type,
        content_id: ::core::ffi::c_int,
    ) -> *mut cram_block;
    fn cram_free_block(b: *mut cram_block);
    fn cram_compress_block2(
        fd: *mut cram_fd,
        s: *mut cram_slice,
        b: *mut cram_block,
        metrics: *mut cram_metrics,
        method: ::core::ffi::c_int,
        level: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cram_new_container(
        nrec: ::core::ffi::c_int,
        nslice: ::core::ffi::c_int,
    ) -> *mut cram_container;
    fn cram_free_container(c: *mut cram_container);
    fn sam_hrecs_find_type_id(
        hrecs: *mut sam_hrecs_t,
        type_0: *const ::core::ffi::c_char,
        ID_key: *const ::core::ffi::c_char,
        ID_value: *const ::core::ffi::c_char,
    ) -> *mut sam_hrec_type_t;
    fn sam_hrecs_find_key(
        type_0: *mut sam_hrec_type_t,
        key: *const ::core::ffi::c_char,
        prev: *mut *mut sam_hrec_tag_t,
    ) -> *mut sam_hrec_tag_t;
    fn sam_hrecs_find_rg(
        hrecs: *mut sam_hrecs_t,
        rg: *const ::core::ffi::c_char,
    ) -> *mut sam_hrec_rg_t;
    fn string_ndup(
        a_str: *mut string_alloc_t,
        instr: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn cram_encoder_init(
        codec: cram_encoding,
        st: *mut cram_stats,
        option: cram_external_type,
        dat: *mut ::core::ffi::c_void,
        version: ::core::ffi::c_int,
        vv: *mut varint_vec,
    ) -> *mut cram_codec;
    fn pthread_mutex_lock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_mutex_unlock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn itf8_put_blk(blk: *mut cram_block, val: int32_t) -> ::core::ffi::c_int;
    fn cram_new_metrics() -> *mut cram_metrics;
    fn cram_get_ref(
        fd: *mut cram_fd,
        id: ::core::ffi::c_int,
        start: hts_pos_t,
        end: hts_pos_t,
    ) -> *mut ::core::ffi::c_char;
    fn cram_ref_incr(r: *mut refs_t, id: ::core::ffi::c_int);
    fn cram_ref_decr(r: *mut refs_t, id: ::core::ffi::c_int);
    fn cram_flush_container_mt(fd: *mut cram_fd, c: *mut cram_container) -> ::core::ffi::c_int;
    fn cram_free_slice(s: *mut cram_slice);
    fn cram_new_slice(type_0: cram_content_type, nrecs: ::core::ffi::c_int) -> *mut cram_slice;
    fn cram_stats_add(st: *mut cram_stats, val: int64_t) -> ::core::ffi::c_int;
    fn cram_stats_del(st: *mut cram_stats, val: int64_t);
    fn cram_stats_encoding(fd: *mut cram_fd, st: *mut cram_stats) -> cram_encoding;
    fn sam_realloc_bam_data(b: *mut bam1_t, desired: size_t) -> ::core::ffi::c_int;
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
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bam1_t {
    pub core: bam1_core_t,
    pub id: uint64_t,
    pub data: *mut uint8_t,
    pub l_data: ::core::ffi::c_int,
    pub m_data: uint32_t,
    pub mempolicy_and_reserved: uint32_t,
}
impl bam1_t {
    unsafe fn set_mempolicy(&mut self, val: uint32_t) {
        self.mempolicy_and_reserved = (self.mempolicy_and_reserved & !3) | (val & 3);
    }

    unsafe fn mempolicy(&self) -> uint32_t {
        self.mempolicy_and_reserved & 3
    }
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
    pub u: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
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
    pub X: C2RustUnnamed_11,
    pub B: C2RustUnnamed_10,
    pub b: C2RustUnnamed_9,
    pub Q: C2RustUnnamed_8,
    pub S: C2RustUnnamed_7,
    pub I: C2RustUnnamed_6,
    pub i: C2RustUnnamed_5,
    pub D: C2RustUnnamed_4,
    pub N: C2RustUnnamed_3,
    pub P: C2RustUnnamed_2,
    pub H: C2RustUnnamed_1,
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
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub base: ::core::ffi::c_int,
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
    pub len: ::core::ffi::c_int,
    pub seq_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_8 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub qual: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_9 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub seq_idx: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_10 {
    pub pos: ::core::ffi::c_int,
    pub code: ::core::ffi::c_int,
    pub base: ::core::ffi::c_int,
    pub qual: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
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
    pub version: C2RustUnnamed_12,
    pub compression: htsCompression,
    pub compression_level: ::core::ffi::c_short,
    pub specific: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
    pub major: ::core::ffi::c_short,
    pub minor: ::core::ffi::c_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct htsFile {
    pub bitfields: uint32_t,
    pub padding_0: uint32_t,
    pub lineno: int64_t,
    pub line: kstring_t,
    pub fn_0: *mut ::core::ffi::c_char,
    pub fn_aux: *mut ::core::ffi::c_char,
    pub fp: C2RustUnnamed_13,
    pub state: *mut ::core::ffi::c_void,
    pub format: htsFormat,
    pub idx: *mut hts_idx_t,
    pub fnidx: *const ::core::ffi::c_char,
    pub bam_header: *mut sam_hdr_t,
    pub filter: *mut hts_filter_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_13 {
    pub bgzf: *mut BGZF,
    pub cram: *mut cram_fd,
    pub hfile: *mut hFILE,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hts_itr_t {
    pub bitfields: uint32_t,
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
    pub bins: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub n: ::core::ffi::c_int,
    pub m: ::core::ffi::c_int,
    pub a: *mut ::core::ffi::c_int,
}
pub type uint16_u = uint16_t;
pub type uint32_u = uint32_t;
pub type uint64_u = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_15 {
    pub u: uint32_t,
    pub f: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_16 {
    pub u: uint64_t,
    pub f: ::core::ffi::c_double,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_17 {
    pub u: uint32_t,
    pub f: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_18 {
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
pub type kh_m_s2u64_t = kh_m_s2u64_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kh_m_s2u64_s {
    pub n_buckets: khint_t,
    pub size: khint_t,
    pub n_occupied: khint_t,
    pub upper_bound: khint_t,
    pub flags: *mut khint32_t,
    pub keys: *mut kh_cstr_t,
    pub vals: *mut uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_19 {
    pub i64_0: uint64_t,
    pub counts: C2RustUnnamed_20,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub e: int32_t,
    pub c: int32_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
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
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
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
                let ref mut fresh34 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh34 = (*fresh34 as ::core::ffi::c_ulong
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
                    let ref mut fresh35 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh35 = (*fresh35 as ::core::ffi::c_ulong
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
                        let ref mut fresh36 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh36 = (*fresh36 as ::core::ffi::c_ulong
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
        let ref mut fresh37 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh37 = (*fresh37 as ::core::ffi::c_ulong
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
        let ref mut fresh38 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh38 = (*fresh38 as ::core::ffi::c_ulong
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
        let ref mut fresh39 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh39 = (*fresh39 as ::core::ffi::c_ulong
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
            let ref mut fresh40 = *hist.offset(dist as isize);
            *fresh40 = (*fresh40).wrapping_add(1);
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
                let ref mut fresh41 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh41 = (*fresh41 as ::core::ffi::c_ulong
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
                    let ref mut fresh42 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh42 = (*fresh42 as ::core::ffi::c_ulong
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
                        let ref mut fresh43 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh43 = (*fresh43 as ::core::ffi::c_ulong
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
        let ref mut fresh44 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh44 = (*fresh44 as ::core::ffi::c_ulong
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
        let ref mut fresh45 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh45 = (*fresh45 as ::core::ffi::c_ulong
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
        let ref mut fresh46 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh46 = (*fresh46 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
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
            let ref mut fresh47 = *hist.offset(dist as isize);
            *fresh47 = (*fresh47).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
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
            let ref mut fresh58 = *hist.offset(dist as isize);
            *fresh58 = (*fresh58).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
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
                let ref mut fresh48 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh48 = (*fresh48 as ::core::ffi::c_ulong
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
                    let ref mut fresh49 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh49 = (*fresh49 as ::core::ffi::c_ulong
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
                        let ref mut fresh50 = *(*h).keys.offset(i as isize);
                        *fresh50 = key;
                        key = tmp;
                        let mut tmp_0: pmap_t = *(*h).vals.offset(i as isize);
                        *(*h).vals.offset(i as isize) = val;
                        val = tmp_0;
                        let ref mut fresh51 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh51 = (*fresh51 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        let ref mut fresh52 = *(*h).keys.offset(i as isize);
                        *fresh52 = key;
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
        let ref mut fresh53 = *(*h).keys.offset(x as isize);
        *fresh53 = key;
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
        let ref mut fresh55 = *(*h).keys.offset(x as isize);
        *fresh55 = key;
        let ref mut fresh56 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh56 = (*fresh56 as ::core::ffi::c_ulong
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
unsafe extern "C" fn kh_del_map(mut h: *mut kh_map_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh57 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh57 = (*fresh57 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_init_map() -> *mut kh_map_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_map_t>() as size_t) as *mut kh_map_t;
}
pub const CRAM_SUBST_MATRIX: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"CGTNGTANCATNGCANACGT\0")
};
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
            let ref mut fresh67 = *hist.offset(dist as isize);
            *fresh67 = (*fresh67).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
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
                let ref mut fresh59 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh59 = (*fresh59 as ::core::ffi::c_ulong
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
                    let ref mut fresh60 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh60 = (*fresh60 as ::core::ffi::c_ulong
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
                        let ref mut fresh61 = *(*h).vals.offset(i as isize);
                        *fresh61 = val;
                        val = tmp_0;
                        let ref mut fresh62 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh62 = (*fresh62 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        let ref mut fresh63 = *(*h).vals.offset(i as isize);
                        *fresh63 = val;
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
        let ref mut fresh64 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh64 = (*fresh64 as ::core::ffi::c_ulong
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
        let ref mut fresh65 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh65 = (*fresh65 as ::core::ffi::c_ulong
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
unsafe extern "C" fn kh_del_m_metrics(mut h: *mut kh_m_metrics_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh66 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh66 = (*fresh66 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
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
        let ref mut fresh73 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh73 = (*fresh73 as ::core::ffi::c_ulong
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
        let ref mut fresh74 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh74 = (*fresh74 as ::core::ffi::c_ulong
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
        let ref mut fresh75 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh75 = (*fresh75 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
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
                let ref mut fresh68 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh68 = (*fresh68 as ::core::ffi::c_ulong
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
                    let ref mut fresh69 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh69 = (*fresh69 as ::core::ffi::c_ulong
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
                        let ref mut fresh70 = *(*h).vals.offset(i as isize);
                        *fresh70 = val;
                        val = tmp_0;
                        let ref mut fresh71 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh71 = (*fresh71 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        *(*h).keys.offset(i as isize) = key;
                        let ref mut fresh72 = *(*h).vals.offset(i as isize);
                        *fresh72 = val;
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
            let ref mut fresh76 = *hist.offset(dist as isize);
            *fresh76 = (*fresh76).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
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
                let ref mut fresh77 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh77 = (*fresh77 as ::core::ffi::c_ulong
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
                    let ref mut fresh78 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh78 = (*fresh78 as ::core::ffi::c_ulong
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
                        let ref mut fresh79 = *(*h).keys.offset(i as isize);
                        *fresh79 = key;
                        key = tmp;
                        let mut tmp_0: *mut ref_entry = *(*h).vals.offset(i as isize);
                        let ref mut fresh80 = *(*h).vals.offset(i as isize);
                        *fresh80 = val;
                        val = tmp_0;
                        let ref mut fresh81 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh81 = (*fresh81 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        let ref mut fresh82 = *(*h).keys.offset(i as isize);
                        *fresh82 = key;
                        let ref mut fresh83 = *(*h).vals.offset(i as isize);
                        *fresh83 = val;
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
        let ref mut fresh84 = *(*h).keys.offset(x as isize);
        *fresh84 = key;
        let ref mut fresh85 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh85 = (*fresh85 as ::core::ffi::c_ulong
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
        let ref mut fresh86 = *(*h).keys.offset(x as isize);
        *fresh86 = key;
        let ref mut fresh87 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh87 = (*fresh87 as ::core::ffi::c_ulong
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
        let ref mut fresh88 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh88 = (*fresh88 as ::core::ffi::c_ulong
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
            let ref mut fresh89 = *hist.offset(dist as isize);
            *fresh89 = (*fresh89).wrapping_add(1);
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
pub const CRAM_FLAG_MASK: ::core::ffi::c_int =
    ((1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int) - 1 as ::core::ffi::c_int;
pub const CRAM_FLAG_STATS_ADDED: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int;
pub const CRAM_FLAG_DISCARD_NAME: ::core::ffi::c_uint =
    (1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int;
pub const BAM_CMATCH: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BAM_CINS: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
pub const BAM_CDEL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BAM_CREF_SKIP: ::core::ffi::c_uint = 3 as ::core::ffi::c_uint;
pub const BAM_CSOFT_CLIP: ::core::ffi::c_uint = 4 as ::core::ffi::c_uint;
pub const BAM_CHARD_CLIP: ::core::ffi::c_uint = 5 as ::core::ffi::c_uint;
pub const BAM_CPAD: ::core::ffi::c_uint = 6 as ::core::ffi::c_uint;
pub const BAM_CEQUAL: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const BAM_CDIFF: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const BAM_CIGAR_SHIFT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const BAM_CIGAR_MASK: ::core::ffi::c_int = 0xf as ::core::ffi::c_int;
pub const BAM_CIGAR_TYPE: ::core::ffi::c_int = 0x3c1a7 as ::core::ffi::c_int;
pub const BAM_FPAIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BAM_FUNMAP: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const BAM_FMUNMAP: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const BAM_FREVERSE: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const BAM_FMREVERSE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const BAM_FREAD1: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const BAM_FREAD2: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
pub const BAM_FSECONDARY: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const BAM_FSUPPLEMENTARY: ::core::ffi::c_int = 2048 as ::core::ffi::c_int;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
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
                current_block = 17451889097450565402;
            } else {
                n = le_to_u32(s);
                s = s.offset(4 as ::core::ffi::c_int as isize);
                if (end.offset_from(s) as ::core::ffi::c_long as size_t)
                    .wrapping_div(sub_type_size as size_t)
                    < n as size_t
                {
                    current_block = 17451889097450565402;
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
                            current_block = 9187841854201370858;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 17182852842979701788;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 10958298602014984662;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 11729212326994078217;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 13858073706069245343;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 15249656798102287732;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 2908188968556281218;
                            match current_block {
                                2908188968556281218 => {
                                    if ks_expand(ks, n.wrapping_mul(8 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                17182852842979701788 => {
                                    if ks_expand(ks, n.wrapping_mul(2 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                10958298602014984662 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                11729212326994078217 => {
                                    if ks_expand(ks, n.wrapping_mul(4 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                13858073706069245343 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                15249656798102287732 => {
                                    if ks_expand(ks, n.wrapping_mul(6 as uint32_t) as size_t)
                                        < 0 as ::core::ffi::c_int
                                    {
                                        current_block = 6092762445245489390;
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
                                        current_block = 6092762445245489390;
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
                            current_block = 17451889097450565402;
                        }
                    }
                }
            }
        } else {
            current_block = 17451889097450565402;
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
unsafe extern "C" fn kh_destroy_sam_hrecs_t(mut h: *mut kh_sam_hrecs_t_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
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
unsafe extern "C" fn kh_destroy_m_s2i(mut h: *mut kh_m_s2i_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
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
unsafe extern "C" fn kh_init_m_s2i() -> *mut kh_m_s2i_t {
    return calloc(1 as size_t, ::core::mem::size_of::<kh_m_s2i_t>() as size_t) as *mut kh_m_s2i_t;
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
    let mut convert: C2RustUnnamed_15 = C2RustUnnamed_15 { u: 0 };
    convert.u = le_to_u32(buf);
    return convert.f;
}
#[inline]
unsafe extern "C" fn le_to_double(mut buf: *const uint8_t) -> ::core::ffi::c_double {
    let mut convert: C2RustUnnamed_16 = C2RustUnnamed_16 { u: 0 };
    convert.u = le_to_u64(buf);
    return convert.f;
}
#[inline]
unsafe extern "C" fn float_to_le(mut val: ::core::ffi::c_float, mut buf: *mut uint8_t) {
    let mut convert: C2RustUnnamed_17 = C2RustUnnamed_17 { u: 0 };
    convert.f = val;
    u32_to_le(convert.u, buf);
}
#[inline]
unsafe extern "C" fn double_to_le(mut val: ::core::ffi::c_double, mut buf: *mut uint8_t) {
    let mut convert: C2RustUnnamed_18 = C2RustUnnamed_18 { u: 0 };
    convert.f = val;
    u64_to_le(convert.u, buf);
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
    let fresh90 = (*b).byte;
    (*b).byte = (*b).byte.wrapping_add(1);
    *(*b).data.offset(fresh90 as isize) = c as ::core::ffi::c_uchar;
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
        let fresh91 = cp;
        cp = cp.offset(1);
        *fresh91 = '0' as i32 as ::core::ffi::c_uchar;
        return cp;
    }
    if i < 100 as uint32_t {
        current_block = 14405742329680205474;
    } else {
        if i < 10000 as uint32_t {
            current_block = 2797425235348044640;
        } else {
            if i < 1000000 as uint32_t {
                current_block = 13806706438560306819;
            } else {
                if i < 100000000 as uint32_t {
                    current_block = 15200016511995544180;
                } else {
                    j = i.wrapping_div(1000000000 as uint32_t);
                    if j != 0 {
                        let fresh92 = cp;
                        cp = cp.offset(1);
                        *fresh92 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint).wrapping_sub(
                            j.wrapping_mul(1000000000 as uint32_t) as ::core::ffi::c_uint,
                        ) as uint32_t as uint32_t;
                        let fresh102 = cp;
                        cp = cp.offset(1);
                        *fresh102 = i
                            .wrapping_div(100000000 as uint32_t)
                            .wrapping_add('0' as i32 as uint32_t)
                            as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_rem(100000000 as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 2605302652125400332;
                    } else {
                        j = i.wrapping_div(100000000 as uint32_t);
                        if j != 0 {
                            let fresh93 = cp;
                            cp = cp.offset(1);
                            *fresh93 =
                                j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                            i =
                                (i as ::core::ffi::c_uint)
                                    .wrapping_sub(j.wrapping_mul(100000000 as uint32_t)
                                        as ::core::ffi::c_uint)
                                    as uint32_t as uint32_t;
                            current_block = 2605302652125400332;
                        } else {
                            current_block = 15200016511995544180;
                        }
                    }
                    match current_block {
                        15200016511995544180 => {}
                        _ => {
                            let fresh103 = cp;
                            cp = cp.offset(1);
                            *fresh103 = i
                                .wrapping_div(10000000 as uint32_t)
                                .wrapping_add('0' as i32 as uint32_t)
                                as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint)
                                .wrapping_rem(10000000 as ::core::ffi::c_int as ::core::ffi::c_uint)
                                as uint32_t as uint32_t;
                            current_block = 636314655508768091;
                        }
                    }
                }
                match current_block {
                    15200016511995544180 => {
                        j = i.wrapping_div(10000000 as uint32_t);
                        if j != 0 {
                            let fresh94 = cp;
                            cp = cp.offset(1);
                            *fresh94 =
                                j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint).wrapping_sub(
                                j.wrapping_mul(10000000 as uint32_t) as ::core::ffi::c_uint,
                            ) as uint32_t as uint32_t;
                            current_block = 636314655508768091;
                        } else {
                            j = i.wrapping_div(1000000 as uint32_t);
                            if j != 0 {
                                let fresh95 = cp;
                                cp = cp.offset(1);
                                *fresh95 =
                                    j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                                i =
                                    (i as ::core::ffi::c_uint)
                                        .wrapping_sub(j.wrapping_mul(1000000 as uint32_t)
                                            as ::core::ffi::c_uint)
                                        as uint32_t as uint32_t;
                                current_block = 10646083454859170841;
                            } else {
                                current_block = 13806706438560306819;
                            }
                        }
                    }
                    _ => {}
                }
                match current_block {
                    13806706438560306819 => {}
                    _ => {
                        match current_block {
                            636314655508768091 => {
                                let fresh104 = cp;
                                cp = cp.offset(1);
                                *fresh104 = i
                                    .wrapping_div(1000000 as uint32_t)
                                    .wrapping_add('0' as i32 as uint32_t)
                                    as ::core::ffi::c_uchar;
                                i = (i as ::core::ffi::c_uint).wrapping_rem(
                                    1000000 as ::core::ffi::c_int as ::core::ffi::c_uint,
                                ) as uint32_t as uint32_t;
                            }
                            _ => {}
                        }
                        let fresh105 = cp;
                        cp = cp.offset(1);
                        *fresh105 = i
                            .wrapping_div(100000 as uint32_t)
                            .wrapping_add('0' as i32 as uint32_t)
                            as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_rem(100000 as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 11215215966388952959;
                    }
                }
            }
            match current_block {
                13806706438560306819 => {
                    j = i.wrapping_div(100000 as uint32_t);
                    if j != 0 {
                        let fresh96 = cp;
                        cp = cp.offset(1);
                        *fresh96 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_sub(j.wrapping_mul(100000 as uint32_t) as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 11215215966388952959;
                    } else {
                        j = i.wrapping_div(10000 as uint32_t);
                        if j != 0 {
                            let fresh97 = cp;
                            cp = cp.offset(1);
                            *fresh97 =
                                j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint).wrapping_sub(
                                j.wrapping_mul(10000 as uint32_t) as ::core::ffi::c_uint,
                            ) as uint32_t as uint32_t;
                            current_block = 11844950396038381015;
                        } else {
                            current_block = 2797425235348044640;
                        }
                    }
                }
                _ => {}
            }
            match current_block {
                2797425235348044640 => {}
                _ => {
                    match current_block {
                        11215215966388952959 => {
                            let fresh106 = cp;
                            cp = cp.offset(1);
                            *fresh106 = i
                                .wrapping_div(10000 as uint32_t)
                                .wrapping_add('0' as i32 as uint32_t)
                                as ::core::ffi::c_uchar;
                            i = (i as ::core::ffi::c_uint)
                                .wrapping_rem(10000 as ::core::ffi::c_uint)
                                as uint32_t as uint32_t;
                        }
                        _ => {}
                    }
                    let fresh107 = cp;
                    cp = cp.offset(1);
                    *fresh107 = i
                        .wrapping_div(1000 as uint32_t)
                        .wrapping_add('0' as i32 as uint32_t)
                        as ::core::ffi::c_uchar;
                    i = (i as ::core::ffi::c_uint).wrapping_rem(1000 as ::core::ffi::c_uint)
                        as uint32_t as uint32_t;
                    current_block = 3359508080067919775;
                }
            }
        }
        match current_block {
            2797425235348044640 => {
                j = i.wrapping_div(1000 as uint32_t);
                if j != 0 {
                    let fresh98 = cp;
                    cp = cp.offset(1);
                    *fresh98 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                    i = (i as ::core::ffi::c_uint)
                        .wrapping_sub(j.wrapping_mul(1000 as uint32_t) as ::core::ffi::c_uint)
                        as uint32_t as uint32_t;
                    current_block = 3359508080067919775;
                } else {
                    j = i.wrapping_div(100 as uint32_t);
                    if j != 0 {
                        let fresh99 = cp;
                        cp = cp.offset(1);
                        *fresh99 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint)
                            .wrapping_sub(j.wrapping_mul(100 as uint32_t) as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                        current_block = 16512515393634988648;
                    } else {
                        current_block = 14405742329680205474;
                    }
                }
            }
            _ => {}
        }
        match current_block {
            14405742329680205474 => {}
            _ => {
                match current_block {
                    3359508080067919775 => {
                        let fresh108 = cp;
                        cp = cp.offset(1);
                        *fresh108 = i
                            .wrapping_div(100 as uint32_t)
                            .wrapping_add('0' as i32 as uint32_t)
                            as ::core::ffi::c_uchar;
                        i = (i as ::core::ffi::c_uint).wrapping_rem(100 as ::core::ffi::c_uint)
                            as uint32_t as uint32_t;
                    }
                    _ => {}
                }
                let fresh109 = cp;
                cp = cp.offset(1);
                *fresh109 = i
                    .wrapping_div(10 as uint32_t)
                    .wrapping_add('0' as i32 as uint32_t)
                    as ::core::ffi::c_uchar;
                i = (i as ::core::ffi::c_uint).wrapping_rem(10 as ::core::ffi::c_uint) as uint32_t
                    as uint32_t;
                current_block = 10029297795141560187;
            }
        }
    }
    match current_block {
        14405742329680205474 => {
            j = i.wrapping_div(10 as uint32_t);
            if j != 0 {
                let fresh100 = cp;
                cp = cp.offset(1);
                *fresh100 = j.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                i = (i as ::core::ffi::c_uint)
                    .wrapping_sub(j.wrapping_mul(10 as uint32_t) as ::core::ffi::c_uint)
                    as uint32_t as uint32_t;
            } else {
                if i != 0 {
                    let fresh101 = cp;
                    cp = cp.offset(1);
                    *fresh101 = i.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
                }
                return cp;
            }
        }
        _ => {}
    }
    let fresh110 = cp;
    cp = cp.offset(1);
    *fresh110 = i.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    return cp;
}
#[inline]
unsafe extern "C" fn append_sub32(
    mut cp: *mut ::core::ffi::c_uchar,
    mut i: uint32_t,
) -> *mut ::core::ffi::c_uchar {
    let fresh111 = cp;
    cp = cp.offset(1);
    *fresh111 = i
        .wrapping_div(100000000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint)
        .wrapping_rem(100000000 as ::core::ffi::c_int as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh112 = cp;
    cp = cp.offset(1);
    *fresh112 = i
        .wrapping_div(10000000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint)
        .wrapping_rem(10000000 as ::core::ffi::c_int as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh113 = cp;
    cp = cp.offset(1);
    *fresh113 = i
        .wrapping_div(1000000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint)
        .wrapping_rem(1000000 as ::core::ffi::c_int as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh114 = cp;
    cp = cp.offset(1);
    *fresh114 = i
        .wrapping_div(100000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(100000 as ::core::ffi::c_int as ::core::ffi::c_uint)
        as uint32_t as uint32_t;
    let fresh115 = cp;
    cp = cp.offset(1);
    *fresh115 = i
        .wrapping_div(10000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(10000 as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh116 = cp;
    cp = cp.offset(1);
    *fresh116 = i
        .wrapping_div(1000 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(1000 as ::core::ffi::c_uint) as uint32_t
        as uint32_t;
    let fresh117 = cp;
    cp = cp.offset(1);
    *fresh117 = i
        .wrapping_div(100 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(100 as ::core::ffi::c_uint) as uint32_t as uint32_t;
    let fresh118 = cp;
    cp = cp.offset(1);
    *fresh118 = i
        .wrapping_div(10 as uint32_t)
        .wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
    i = (i as ::core::ffi::c_uint).wrapping_rem(10 as ::core::ffi::c_uint) as uint32_t as uint32_t;
    let fresh119 = cp;
    cp = cp.offset(1);
    *fresh119 = i.wrapping_add('0' as i32 as uint32_t) as ::core::ffi::c_uchar;
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
#[inline]
unsafe extern "C" fn kh_get_m_s2u64(mut h: *const kh_m_s2u64_t, mut key: kh_cstr_t) -> khint_t {
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
unsafe extern "C" fn kh_grow_to_fit_m_s2u64(
    mut h: *mut kh_m_s2u64_t,
    mut n_items: khint_t,
) -> ::core::ffi::c_int {
    let mut n_buckets: khint_t = 0;
    let mut resize_limit: khint_t = (INT_MAX as ::core::ffi::c_double * __ac_HASH_UPPER) as khint_t;
    if n_items < (*h).upper_bound.wrapping_sub((*h).size) {
        if n_items < (*h).upper_bound.wrapping_sub((*h).n_occupied) {
            return 0 as ::core::ffi::c_int;
        }
        return kh_resize_m_s2u64(h, (*h).n_buckets.wrapping_sub(1 as khint_t));
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
    return kh_resize_m_s2u64(h, n_buckets);
}
#[inline]
unsafe extern "C" fn kh_init_m_s2u64() -> *mut kh_m_s2u64_t {
    return calloc(
        1 as size_t,
        ::core::mem::size_of::<kh_m_s2u64_t>() as size_t,
    ) as *mut kh_m_s2u64_t;
}
#[inline]
unsafe extern "C" fn kh_resize_m_s2u64(
    mut h: *mut kh_m_s2u64_t,
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
            let mut new_vals: *mut uint64_t = realloc(
                (*h).vals as *mut ::core::ffi::c_void,
                (new_n_buckets as size_t)
                    .wrapping_mul(::core::mem::size_of::<uint64_t>() as size_t),
            ) as *mut uint64_t;
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
                let mut val: uint64_t = 0;
                let mut new_mask: khint_t = 0;
                new_mask = (new_n_buckets as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint) as khint_t;
                val = *(*h).vals.offset(j as isize);
                let ref mut fresh176 = *(*h).flags.offset((j >> 4 as ::core::ffi::c_int) as isize);
                *fresh176 = (*fresh176 as ::core::ffi::c_ulong
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
                    let ref mut fresh177 =
                        *new_flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                    *fresh177 = (*fresh177 as ::core::ffi::c_ulong
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
                        let ref mut fresh178 = *(*h).keys.offset(i as isize);
                        *fresh178 = key;
                        key = tmp;
                        let mut tmp_0: uint64_t = *(*h).vals.offset(i as isize);
                        *(*h).vals.offset(i as isize) = val;
                        val = tmp_0;
                        let ref mut fresh179 =
                            *(*h).flags.offset((i >> 4 as ::core::ffi::c_int) as isize);
                        *fresh179 = (*fresh179 as ::core::ffi::c_ulong
                            | (1 as ::core::ffi::c_ulong)
                                << ((i as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int))
                            as khint32_t;
                    } else {
                        let ref mut fresh180 = *(*h).keys.offset(i as isize);
                        *fresh180 = key;
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
                    .wrapping_mul(::core::mem::size_of::<uint64_t>() as size_t),
            ) as *mut uint64_t;
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
unsafe extern "C" fn kh_destroy_m_s2u64(mut h: *mut kh_m_s2u64_t) {
    if !h.is_null() {
        free((*h).keys as *mut ::core::ffi::c_void);
        free((*h).flags as *mut ::core::ffi::c_void);
        free((*h).vals as *mut ::core::ffi::c_void);
        free(h as *mut ::core::ffi::c_void);
    }
}
#[inline]
unsafe extern "C" fn kh_clear_m_s2u64(mut h: *mut kh_m_s2u64_t) {
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
unsafe extern "C" fn kh_put_m_s2u64(
    mut h: *mut kh_m_s2u64_t,
    mut key: kh_cstr_t,
    mut ret: *mut ::core::ffi::c_int,
) -> khint_t {
    let mut x: khint_t = 0;
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > (*h).size << 1 as ::core::ffi::c_int {
            if kh_resize_m_s2u64(h, (*h).n_buckets.wrapping_sub(1 as khint_t))
                < 0 as ::core::ffi::c_int
            {
                *ret = -(1 as ::core::ffi::c_int);
                return (*h).n_buckets;
            }
        } else if kh_resize_m_s2u64(h, (*h).n_buckets.wrapping_add(1 as khint_t))
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
        let ref mut fresh172 = *(*h).keys.offset(x as isize);
        *fresh172 = key;
        let ref mut fresh173 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh173 = (*fresh173 as ::core::ffi::c_ulong
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
        let ref mut fresh174 = *(*h).keys.offset(x as isize);
        *fresh174 = key;
        let ref mut fresh175 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh175 = (*fresh175 as ::core::ffi::c_ulong
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
unsafe extern "C" fn kh_del_m_s2u64(mut h: *mut kh_m_s2u64_t, mut x: khint_t) {
    if x != (*h).n_buckets
        && *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >> ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint) << 1 as ::core::ffi::c_int)
            & 3 as ::core::ffi::c_uint
            == 0
    {
        let ref mut fresh185 = *(*h).flags.offset((x >> 4 as ::core::ffi::c_int) as isize);
        *fresh185 = (*fresh185 as ::core::ffi::c_ulong
            | (1 as ::core::ffi::c_ulong)
                << ((x as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)) as khint32_t;
        (*h).size = (*h).size.wrapping_sub(1);
    }
}
#[inline]
unsafe extern "C" fn kh_stats_m_s2u64(
    mut h: *mut kh_m_s2u64_t,
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
            let ref mut fresh186 = *hist.offset(dist as isize);
            *fresh186 = (*fresh186).wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *hist_out = hist;
    *hist_size =
        (dist_max as ::core::ffi::c_uint).wrapping_add(1 as ::core::ffi::c_uint) as khint_t;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn sub_idx(
    mut key: *mut ::core::ffi::c_char,
    mut val: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int && {
        let fresh129 = key;
        key = key.offset(1);
        *fresh129 as ::core::ffi::c_int != val as ::core::ffi::c_int
    } {
        i += 1;
    }
    return i;
}
#[no_mangle]
// original: cram_encode_compression_header (htslib/cram/cram_encode.c:83)
pub unsafe extern "C" fn cram_encode_compression_header(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut h: *mut cram_block_compression_hdr,
    mut embed_ref: ::core::ffi::c_int,
) -> *mut cram_block {
    let mut current_block: u64;
    let mut cb: *mut cram_block = cram_new_block(COMPRESSION_HEADER, 0 as ::core::ffi::c_int);
    let mut map: *mut cram_block = cram_new_block(COMPRESSION_HEADER, 0 as ::core::ffi::c_int);
    let mut i: ::core::ffi::c_int = 0;
    let mut mc: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut no_ref: ::core::ffi::c_int = (*c).no_ref;
    if cb.is_null() || map.is_null() {
        return ::core::ptr::null_mut::<cram_block>();
    }
    if (*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        r |= itf8_put_blk(cb, (*h).ref_seq_id);
        r |= itf8_put_blk(cb, (*h).ref_seq_start as int32_t);
        r |= itf8_put_blk(cb, (*h).ref_seq_span as int32_t);
        r |= itf8_put_blk(cb, (*h).num_records);
        r |= itf8_put_blk(cb, (*h).num_landmarks);
        i = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*h).num_landmarks {
            r |= itf8_put_blk(cb, *(*h).landmark.offset(i as isize));
            i += 1;
        }
    }
    if !(*h).preservation_map.is_null() {
        kh_destroy_map((*h).preservation_map);
        (*h).preservation_map = ::core::ptr::null_mut::<kh_map_t>();
    }
    if (*c).num_records > 0 as int32_t {
        let mut k: khint_t = 0;
        let mut r_0: ::core::ffi::c_int = 0;
        (*h).preservation_map = kh_init_map();
        if (*h).preservation_map.is_null() {
            return ::core::ptr::null_mut::<cram_block>();
        }
        k = kh_put_map(
            (*h).preservation_map,
            b"RN\0" as *const u8 as kh_cstr_t,
            &raw mut r_0,
        );
        if -(1 as ::core::ffi::c_int) == r_0 {
            return ::core::ptr::null_mut::<cram_block>();
        }
        (*(*(*h).preservation_map).vals.offset(k as isize)).i =
            ((*fd).lossy_read_names == 0) as ::core::ffi::c_int;
        if (*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            k = kh_put_map(
                (*h).preservation_map,
                b"PI\0" as *const u8 as kh_cstr_t,
                &raw mut r_0,
            );
            if -(1 as ::core::ffi::c_int) == r_0 {
                return ::core::ptr::null_mut::<cram_block>();
            }
            (*(*(*h).preservation_map).vals.offset(k as isize)).i = 0 as ::core::ffi::c_int;
            k = kh_put_map(
                (*h).preservation_map,
                b"UI\0" as *const u8 as kh_cstr_t,
                &raw mut r_0,
            );
            if -(1 as ::core::ffi::c_int) == r_0 {
                return ::core::ptr::null_mut::<cram_block>();
            }
            (*(*(*h).preservation_map).vals.offset(k as isize)).i = 1 as ::core::ffi::c_int;
            k = kh_put_map(
                (*h).preservation_map,
                b"MI\0" as *const u8 as kh_cstr_t,
                &raw mut r_0,
            );
            if -(1 as ::core::ffi::c_int) == r_0 {
                return ::core::ptr::null_mut::<cram_block>();
            }
            (*(*(*h).preservation_map).vals.offset(k as isize)).i = 1 as ::core::ffi::c_int;
        } else {
            k = kh_put_map(
                (*h).preservation_map,
                b"SM\0" as *const u8 as kh_cstr_t,
                &raw mut r_0,
            );
            if -(1 as ::core::ffi::c_int) == r_0 {
                return ::core::ptr::null_mut::<cram_block>();
            }
            (*(*(*h).preservation_map).vals.offset(k as isize)).i = 0 as ::core::ffi::c_int;
            k = kh_put_map(
                (*h).preservation_map,
                b"TD\0" as *const u8 as kh_cstr_t,
                &raw mut r_0,
            );
            if -(1 as ::core::ffi::c_int) == r_0 {
                return ::core::ptr::null_mut::<cram_block>();
            }
            (*(*(*h).preservation_map).vals.offset(k as isize)).i = 0 as ::core::ffi::c_int;
            k = kh_put_map(
                (*h).preservation_map,
                b"AP\0" as *const u8 as kh_cstr_t,
                &raw mut r_0,
            );
            if -(1 as ::core::ffi::c_int) == r_0 {
                return ::core::ptr::null_mut::<cram_block>();
            }
            (*(*(*h).preservation_map).vals.offset(k as isize)).i = (*h).AP_delta;
            if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
                k = kh_put_map(
                    (*h).preservation_map,
                    b"QO\0" as *const u8 as kh_cstr_t,
                    &raw mut r_0,
                );
                if -(1 as ::core::ffi::c_int) == r_0 {
                    return ::core::ptr::null_mut::<cram_block>();
                }
                (*(*(*h).preservation_map).vals.offset(k as isize)).i = (*h).qs_seq_orient;
            }
            if no_ref != 0 || embed_ref > 0 as ::core::ffi::c_int {
                k = kh_put_map(
                    (*h).preservation_map,
                    b"RR\0" as *const u8 as kh_cstr_t,
                    &raw mut r_0,
                );
                if -(1 as ::core::ffi::c_int) == r_0 {
                    return ::core::ptr::null_mut::<cram_block>();
                }
                (*(*(*h).preservation_map).vals.offset(k as isize)).i = 0 as ::core::ffi::c_int;
            }
        }
    }
    mc = 0 as ::core::ffi::c_int;
    (*map).byte = 0 as size_t;
    if !(*h).preservation_map.is_null() {
        let mut k_0: khint_t = 0;
        k_0 = 0 as ::core::ffi::c_int as khint_t;
        loop {
            if !(k_0 != (*(*h).preservation_map).n_buckets) {
                current_block = 777662472977924419;
                break;
            }
            let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut pmap: *mut kh_map_t = (*h).preservation_map;
            if !(*(*pmap)
                .flags
                .offset((k_0 >> 4 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >> ((k_0 as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                    << 1 as ::core::ffi::c_int)
                & 3 as ::core::ffi::c_uint
                != 0)
            {
                key = *(*pmap).keys.offset(k_0 as isize) as *const ::core::ffi::c_char;
                if block_append(map, key as *const ::core::ffi::c_void, 2 as size_t)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 2463460412742902288;
                    break;
                }
                match (*key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
                    as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int
                    | *key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
                        as ::core::ffi::c_int
                {
                    19785 | 21833 | 20553 | 16720 | 21070 | 21074 | 20815 => {
                        if block_append_char(
                            map,
                            (*(*pmap).vals.offset(k_0 as isize)).i as ::core::ffi::c_char,
                        ) < 0 as ::core::ffi::c_int
                        {
                            current_block = 2463460412742902288;
                            break;
                        }
                    }
                    21325 => {
                        let mut smat: [::core::ffi::c_char; 5] = [0; 5];
                        let mut mp: *mut ::core::ffi::c_char =
                            &raw mut smat as *mut ::core::ffi::c_char;
                        let fresh124 = mp;
                        mp = mp.offset(1);
                        *fresh124 = (sub_idx(
                            &raw mut *(&raw mut (*h).substitution_matrix
                                as *mut [::core::ffi::c_char; 4])
                                .offset(0 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_char,
                            'C' as i32 as ::core::ffi::c_char,
                        ) << 6 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'G' as i32 as ::core::ffi::c_char,
                            ) << 4 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'T' as i32 as ::core::ffi::c_char,
                            ) << 2 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'N' as i32 as ::core::ffi::c_char,
                            ) << 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let fresh125 = mp;
                        mp = mp.offset(1);
                        *fresh125 = (sub_idx(
                            &raw mut *(&raw mut (*h).substitution_matrix
                                as *mut [::core::ffi::c_char; 4])
                                .offset(1 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_char,
                            'A' as i32 as ::core::ffi::c_char,
                        ) << 6 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'G' as i32 as ::core::ffi::c_char,
                            ) << 4 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'T' as i32 as ::core::ffi::c_char,
                            ) << 2 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'N' as i32 as ::core::ffi::c_char,
                            ) << 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let fresh126 = mp;
                        mp = mp.offset(1);
                        *fresh126 = (sub_idx(
                            &raw mut *(&raw mut (*h).substitution_matrix
                                as *mut [::core::ffi::c_char; 4])
                                .offset(2 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_char,
                            'A' as i32 as ::core::ffi::c_char,
                        ) << 6 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(2 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'C' as i32 as ::core::ffi::c_char,
                            ) << 4 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(2 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'T' as i32 as ::core::ffi::c_char,
                            ) << 2 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(2 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'N' as i32 as ::core::ffi::c_char,
                            ) << 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let fresh127 = mp;
                        mp = mp.offset(1);
                        *fresh127 = (sub_idx(
                            &raw mut *(&raw mut (*h).substitution_matrix
                                as *mut [::core::ffi::c_char; 4])
                                .offset(3 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_char,
                            'A' as i32 as ::core::ffi::c_char,
                        ) << 6 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(3 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'C' as i32 as ::core::ffi::c_char,
                            ) << 4 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(3 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'G' as i32 as ::core::ffi::c_char,
                            ) << 2 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(3 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'N' as i32 as ::core::ffi::c_char,
                            ) << 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let fresh128 = mp;
                        mp = mp.offset(1);
                        *fresh128 = (sub_idx(
                            &raw mut *(&raw mut (*h).substitution_matrix
                                as *mut [::core::ffi::c_char; 4])
                                .offset(4 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_char,
                            'A' as i32 as ::core::ffi::c_char,
                        ) << 6 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(4 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'C' as i32 as ::core::ffi::c_char,
                            ) << 4 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(4 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'G' as i32 as ::core::ffi::c_char,
                            ) << 2 as ::core::ffi::c_int
                            | sub_idx(
                                &raw mut *(&raw mut (*h).substitution_matrix
                                    as *mut [::core::ffi::c_char; 4])
                                    .offset(4 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_char,
                                'T' as i32 as ::core::ffi::c_char,
                            ) << 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        if block_append(
                            map,
                            &raw mut smat as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                            5 as size_t,
                        ) < 0 as ::core::ffi::c_int
                        {
                            current_block = 2463460412742902288;
                            break;
                        }
                    }
                    21572 => {
                        r |= ((*fd)
                            .vv
                            .varint_put32_blk
                            .expect("non-null function pointer")(
                            map,
                            (*(*h).TD_blk).byte as int32_t,
                        ) <= 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int;
                        if block_append(
                            map,
                            (*(*h).TD_blk).data as *const ::core::ffi::c_void,
                            (*(*h).TD_blk).byte,
                        ) < 0 as ::core::ffi::c_int
                        {
                            current_block = 2463460412742902288;
                            break;
                        }
                    }
                    _ => {
                        hts_log(
                            HTS_LOG_WARNING,
                            b"cram_encode_compression_header\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"Unknown preservation key '%.2s'\0" as *const u8
                                as *const ::core::ffi::c_char,
                            key,
                        );
                    }
                }
                mc += 1;
            }
            k_0 = k_0.wrapping_add(1);
        }
    } else {
        current_block = 777662472977924419;
    }
    match current_block {
        777662472977924419 => {
            r |= ((*fd)
                .vv
                .varint_put32_blk
                .expect("non-null function pointer")(
                cb,
                (*map)
                    .byte
                    .wrapping_add((*fd).vv.varint_size.expect("non-null function pointer")(
                        mc as int64_t,
                    ) as size_t) as int32_t,
            ) <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            r |= ((*fd)
                .vv
                .varint_put32_blk
                .expect("non-null function pointer")(cb, mc as int32_t)
                <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            if !(block_append(cb, (*map).data as *const ::core::ffi::c_void, (*map).byte)
                < 0 as ::core::ffi::c_int)
            {
                mc = 0 as ::core::ffi::c_int;
                (*map).byte = 0 as size_t;
                if !(*h).codecs[DS_BF as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_BF as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_BF as ::core::ffi::c_int as usize],
                            map,
                            b"BF\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_CF as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_CF as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_CF as ::core::ffi::c_int as usize],
                            map,
                            b"CF\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_RL as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_RL as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_RL as ::core::ffi::c_int as usize],
                            map,
                            b"RL\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_AP as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_AP as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_AP as ::core::ffi::c_int as usize],
                            map,
                            b"AP\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_RG as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_RG as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_RG as ::core::ffi::c_int as usize],
                            map,
                            b"RG\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_MF as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_MF as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_MF as ::core::ffi::c_int as usize],
                            map,
                            b"MF\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_NS as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_NS as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_NS as ::core::ffi::c_int as usize],
                            map,
                            b"NS\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_NP as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_NP as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_NP as ::core::ffi::c_int as usize],
                            map,
                            b"NP\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_TS as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_TS as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_TS as ::core::ffi::c_int as usize],
                            map,
                            b"TS\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_NF as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_NF as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_NF as ::core::ffi::c_int as usize],
                            map,
                            b"NF\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_TC as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_TC as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_TC as ::core::ffi::c_int as usize],
                            map,
                            b"TC\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_TN as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_TN as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_TN as ::core::ffi::c_int as usize],
                            map,
                            b"TN\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_TL as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_TL as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_TL as ::core::ffi::c_int as usize],
                            map,
                            b"TL\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_FN as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_FN as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_FN as ::core::ffi::c_int as usize],
                            map,
                            b"FN\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_FC as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_FC as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_FC as ::core::ffi::c_int as usize],
                            map,
                            b"FC\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_FP as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_FP as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_FP as ::core::ffi::c_int as usize],
                            map,
                            b"FP\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_BS as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_BS as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_BS as ::core::ffi::c_int as usize],
                            map,
                            b"BS\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_IN as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_IN as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_IN as ::core::ffi::c_int as usize],
                            map,
                            b"IN\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_DL as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_DL as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_DL as ::core::ffi::c_int as usize],
                            map,
                            b"DL\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_BA as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_BA as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_BA as ::core::ffi::c_int as usize],
                            map,
                            b"BA\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_BB as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_BB as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_BB as ::core::ffi::c_int as usize],
                            map,
                            b"BB\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_MQ as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_MQ as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_MQ as ::core::ffi::c_int as usize],
                            map,
                            b"MQ\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_RN as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_RN as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_RN as ::core::ffi::c_int as usize],
                            map,
                            b"RN\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_QS as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_QS as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_QS as ::core::ffi::c_int as usize],
                            map,
                            b"QS\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_QQ as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_QQ as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_QQ as ::core::ffi::c_int as usize],
                            map,
                            b"QQ\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_RI as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_RI as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_RI as ::core::ffi::c_int as usize],
                            map,
                            b"RI\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if (*fd).version >> 8 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                    if !(*h).codecs[DS_SC as ::core::ffi::c_int as usize].is_null() {
                        if -(1 as ::core::ffi::c_int)
                            == (*(*h).codecs[DS_SC as ::core::ffi::c_int as usize])
                                .store
                                .expect("non-null function pointer")(
                                (*h).codecs[DS_SC as ::core::ffi::c_int as usize],
                                map,
                                b"SC\0" as *const u8 as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                (*fd).version,
                            )
                        {
                            return ::core::ptr::null_mut::<cram_block>();
                        }
                        mc += 1;
                    }
                    if !(*h).codecs[DS_RS as ::core::ffi::c_int as usize].is_null() {
                        if -(1 as ::core::ffi::c_int)
                            == (*(*h).codecs[DS_RS as ::core::ffi::c_int as usize])
                                .store
                                .expect("non-null function pointer")(
                                (*h).codecs[DS_RS as ::core::ffi::c_int as usize],
                                map,
                                b"RS\0" as *const u8 as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                (*fd).version,
                            )
                        {
                            return ::core::ptr::null_mut::<cram_block>();
                        }
                        mc += 1;
                    }
                    if !(*h).codecs[DS_PD as ::core::ffi::c_int as usize].is_null() {
                        if -(1 as ::core::ffi::c_int)
                            == (*(*h).codecs[DS_PD as ::core::ffi::c_int as usize])
                                .store
                                .expect("non-null function pointer")(
                                (*h).codecs[DS_PD as ::core::ffi::c_int as usize],
                                map,
                                b"PD\0" as *const u8 as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                (*fd).version,
                            )
                        {
                            return ::core::ptr::null_mut::<cram_block>();
                        }
                        mc += 1;
                    }
                    if !(*h).codecs[DS_HC as ::core::ffi::c_int as usize].is_null() {
                        if -(1 as ::core::ffi::c_int)
                            == (*(*h).codecs[DS_HC as ::core::ffi::c_int as usize])
                                .store
                                .expect("non-null function pointer")(
                                (*h).codecs[DS_HC as ::core::ffi::c_int as usize],
                                map,
                                b"HC\0" as *const u8 as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                (*fd).version,
                            )
                        {
                            return ::core::ptr::null_mut::<cram_block>();
                        }
                        mc += 1;
                    }
                }
                if !(*h).codecs[DS_TM as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_TM as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_TM as ::core::ffi::c_int as usize],
                            map,
                            b"TM\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                if !(*h).codecs[DS_TV as ::core::ffi::c_int as usize].is_null() {
                    if -(1 as ::core::ffi::c_int)
                        == (*(*h).codecs[DS_TV as ::core::ffi::c_int as usize])
                            .store
                            .expect("non-null function pointer")(
                            (*h).codecs[DS_TV as ::core::ffi::c_int as usize],
                            map,
                            b"TV\0" as *const u8 as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            (*fd).version,
                        )
                    {
                        return ::core::ptr::null_mut::<cram_block>();
                    }
                    mc += 1;
                }
                r |= ((*fd)
                    .vv
                    .varint_put32_blk
                    .expect("non-null function pointer")(
                    cb,
                    (*map).byte.wrapping_add((*fd)
                        .vv
                        .varint_size
                        .expect("non-null function pointer")(
                        mc as int64_t
                    ) as size_t) as int32_t,
                ) <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                r |= ((*fd)
                    .vv
                    .varint_put32_blk
                    .expect("non-null function pointer")(cb, mc as int32_t)
                    <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                if !(block_append(cb, (*map).data as *const ::core::ffi::c_void, (*map).byte)
                    < 0 as ::core::ffi::c_int)
                {
                    mc = 0 as ::core::ffi::c_int;
                    (*map).byte = 0 as size_t;
                    if !(*c).tags_used.is_null() {
                        let mut k_1: khint_t = 0;
                        k_1 = 0 as ::core::ffi::c_int as khint_t;
                        while k_1 != (*(*c).tags_used).n_buckets {
                            let mut key_0: ::core::ffi::c_int = 0;
                            if !(*(*(*c).tags_used)
                                .flags
                                .offset((k_1 >> 4 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_uint
                                >> ((k_1 as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int)
                                & 3 as ::core::ffi::c_uint
                                != 0)
                            {
                                key_0 = *(*(*c).tags_used).keys.offset(k_1 as isize)
                                    as ::core::ffi::c_int;
                                let mut cd: *mut cram_codec =
                                    (**(*(*c).tags_used).vals.offset(k_1 as isize)).codec
                                        as *mut cram_codec;
                                r |= ((*fd)
                                    .vv
                                    .varint_put32_blk
                                    .expect("non-null function pointer")(
                                    map, key_0 as int32_t
                                ) <= 0 as ::core::ffi::c_int)
                                    as ::core::ffi::c_int;
                                if -(1 as ::core::ffi::c_int)
                                    == (*cd).store.expect("non-null function pointer")(
                                        cd as *mut cram_codec,
                                        map,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        (*fd).version,
                                    )
                                {
                                    return ::core::ptr::null_mut::<cram_block>();
                                }
                                mc += 1;
                            }
                            k_1 = k_1.wrapping_add(1);
                        }
                    }
                    r |= ((*fd)
                        .vv
                        .varint_put32_blk
                        .expect("non-null function pointer")(
                        cb,
                        (*map).byte.wrapping_add((*fd)
                            .vv
                            .varint_size
                            .expect("non-null function pointer")(
                            mc as int64_t
                        ) as size_t) as int32_t,
                    ) <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                    r |= ((*fd)
                        .vv
                        .varint_put32_blk
                        .expect("non-null function pointer")(
                        cb, mc as int32_t
                    ) <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
                    if !(block_append(cb, (*map).data as *const ::core::ffi::c_void, (*map).byte)
                        < 0 as ::core::ffi::c_int)
                    {
                        hts_log(
                            HTS_LOG_INFO,
                            b"cram_encode_compression_header\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"Wrote compression block header in %d bytes\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*cb).byte as ::core::ffi::c_int,
                        );
                        (*cb).uncomp_size = (*cb).byte as int32_t;
                        (*cb).comp_size = (*cb).uncomp_size;
                        cram_free_block(map);
                        if r >= 0 as ::core::ffi::c_int {
                            return cb;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<cram_block>();
}
#[no_mangle]
// original: cram_encode_slice_header (htslib/cram/cram_encode.c:511)
pub unsafe extern "C" fn cram_encode_slice_header(
    mut fd: *mut cram_fd,
    mut s: *mut cram_slice,
) -> *mut cram_block {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut b: *mut cram_block = cram_new_block(MAPPED_SLICE, 0 as ::core::ffi::c_int);
    let mut j: ::core::ffi::c_int = 0;
    if b.is_null() {
        return ::core::ptr::null_mut::<cram_block>();
    }
    buf = malloc(
        (22 as int32_t + 16 as int32_t + 5 as int32_t * (8 as int32_t + (*(*s).hdr).num_blocks))
            as size_t,
    ) as *mut ::core::ffi::c_char;
    cp = buf;
    if buf.is_null() {
        cram_free_block(b);
        return ::core::ptr::null_mut::<cram_block>();
    }
    cp = cp.offset((*fd).vv.varint_put32s.expect("non-null function pointer")(
        cp,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        (*(*s).hdr).ref_seq_id,
    ) as isize);
    if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
        cp = cp.offset((*fd).vv.varint_put64.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).ref_seq_start,
        ) as isize);
        cp = cp.offset((*fd).vv.varint_put64.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).ref_seq_span,
        ) as isize);
    } else {
        if (*(*s).hdr).ref_seq_start < 0 as int64_t
            || (*(*s).hdr).ref_seq_start > INT_MAX as int64_t
        {
            hts_log(
                HTS_LOG_ERROR,
                b"cram_encode_slice_header\0" as *const u8 as *const ::core::ffi::c_char,
                b"Reference position too large for CRAM 3\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            cram_free_block(b);
            free(buf as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<cram_block>();
        }
        cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).ref_seq_start as int32_t,
        ) as isize);
        cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).ref_seq_span as int32_t,
        ) as isize);
    }
    cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
        cp,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        (*(*s).hdr).num_records,
    ) as isize);
    if (*fd).version >> 8 as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
        cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).record_counter as int32_t,
        ) as isize);
    } else if (*fd).version >> 8 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int {
        cp = cp.offset((*fd).vv.varint_put64.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).record_counter,
        ) as isize);
    }
    cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
        cp,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        (*(*s).hdr).num_blocks,
    ) as isize);
    cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
        cp,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        (*(*s).hdr).num_content_ids,
    ) as isize);
    j = 0 as ::core::ffi::c_int;
    while (j as int32_t) < (*(*s).hdr).num_content_ids {
        cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            *(*(*s).hdr).block_content_ids.offset(j as isize),
        ) as isize);
        j += 1;
    }
    if (*(*s).hdr).content_type as ::core::ffi::c_int == MAPPED_SLICE as ::core::ffi::c_int {
        cp = cp.offset((*fd).vv.varint_put32.expect("non-null function pointer")(
            cp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*(*s).hdr).ref_base_id,
        ) as isize);
    }
    if (*fd).version >> 8 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        memcpy(
            cp as *mut ::core::ffi::c_void,
            &raw mut (*(*s).hdr).md5 as *mut ::core::ffi::c_uchar as *const ::core::ffi::c_void,
            16 as size_t,
        );
        cp = cp.offset(16 as ::core::ffi::c_int as isize);
    }
    '_c2rust_label: {
        if cp.offset_from(buf) as ::core::ffi::c_long
            <= (22 as int32_t
                + 16 as int32_t
                + 5 as int32_t * (8 as int32_t + (*(*s).hdr).num_blocks))
                as ::core::ffi::c_long
        {
        } else {
            __assert_fail(
                b"cp-buf <= 22+16+5*(8+s->hdr->num_blocks)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"/data/henriksson/github/claude/cellsnp-lite/htslib-rs/htslib/cram/cram_encode.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                557 as ::core::ffi::c_uint,
                b"cram_block *cram_encode_slice_header(cram_fd *, cram_slice *)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*b).data = buf as *mut ::core::ffi::c_uchar;
    (*b).uncomp_size = cp.offset_from(buf) as ::core::ffi::c_long as int32_t;
    (*b).comp_size = (*b).uncomp_size;
    return b;
}
// original: cram_encode_slice_read (htslib/cram/cram_encode.c:572)
unsafe extern "C" fn cram_encode_slice_read(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut h: *mut cram_block_compression_hdr,
    mut s: *mut cram_slice,
    mut cr: *mut cram_record,
    mut last_pos: *mut int64_t,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i32: int32_t = 0;
    let mut i64: int64_t = 0;
    let mut uc: ::core::ffi::c_uchar = 0;
    i32 = (*fd).cram_flag_swap[((*cr).flags & 0xfff as int32_t) as usize] as int32_t;
    r |= (*(*h).codecs[DS_BF as ::core::ffi::c_int as usize])
        .encode
        .expect("non-null function pointer")(
        s,
        (*h).codecs[DS_BF as ::core::ffi::c_int as usize],
        &raw mut i32 as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    i32 = (*cr).cram_flags & CRAM_FLAG_MASK as int32_t;
    r |= (*(*h).codecs[DS_CF as ::core::ffi::c_int as usize])
        .encode
        .expect("non-null function pointer")(
        s,
        (*h).codecs[DS_CF as ::core::ffi::c_int as usize],
        &raw mut i32 as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    if (*fd).version >> 8 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        && (*(*s).hdr).ref_seq_id == -(2 as int32_t)
    {
        r |= (*(*h).codecs[DS_RI as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_RI as ::core::ffi::c_int as usize],
            &raw mut (*cr).ref_id as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    }
    r |= (*(*h).codecs[DS_RL as ::core::ffi::c_int as usize])
        .encode
        .expect("non-null function pointer")(
        s,
        (*h).codecs[DS_RL as ::core::ffi::c_int as usize],
        &raw mut (*cr).len as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    if (*c).pos_sorted != 0 {
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            i64 = (*cr).apos - *last_pos;
            r |= (*(*h).codecs[DS_AP as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_AP as ::core::ffi::c_int as usize],
                &raw mut i64 as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        } else {
            i32 = ((*cr).apos - *last_pos) as int32_t;
            r |= (*(*h).codecs[DS_AP as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_AP as ::core::ffi::c_int as usize],
                &raw mut i32 as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        }
        *last_pos = (*cr).apos;
    } else if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
        i64 = (*cr).apos;
        r |= (*(*h).codecs[DS_AP as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_AP as ::core::ffi::c_int as usize],
            &raw mut i64 as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        i32 = (*cr).apos as int32_t;
        r |= (*(*h).codecs[DS_AP as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_AP as ::core::ffi::c_int as usize],
            &raw mut i32 as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    }
    r |= (*(*h).codecs[DS_RG as ::core::ffi::c_int as usize])
        .encode
        .expect("non-null function pointer")(
        s,
        (*h).codecs[DS_RG as ::core::ffi::c_int as usize],
        &raw mut (*cr).rg as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    if (*cr).cram_flags & CRAM_FLAG_DETACHED as int32_t != 0 {
        i32 = (*cr).mate_flags;
        r |= (*(*h).codecs[DS_MF as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_MF as ::core::ffi::c_int as usize],
            &raw mut i32 as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        r |= (*(*h).codecs[DS_NS as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_NS as ::core::ffi::c_int as usize],
            &raw mut (*cr).mate_ref_id as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            r |= (*(*h).codecs[DS_NP as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_NP as ::core::ffi::c_int as usize],
                &raw mut (*cr).mate_pos as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
            r |= (*(*h).codecs[DS_TS as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_TS as ::core::ffi::c_int as usize],
                &raw mut (*cr).tlen as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        } else {
            i32 = (*cr).mate_pos as int32_t;
            r |= (*(*h).codecs[DS_NP as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_NP as ::core::ffi::c_int as usize],
                &raw mut i32 as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
            i32 = (*cr).tlen as int32_t;
            r |= (*(*h).codecs[DS_TS as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_TS as ::core::ffi::c_int as usize],
                &raw mut i32 as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        }
    } else {
        if (*cr).cram_flags & CRAM_FLAG_MATE_DOWNSTREAM as int32_t != 0 {
            r |= (*(*h).codecs[DS_NF as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_NF as ::core::ffi::c_int as usize],
                &raw mut (*cr).mate_line as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        }
        if (*cr).cram_flags & CRAM_FLAG_EXPLICIT_TLEN as int32_t != 0 {
            if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
                r |= (*(*h).codecs[DS_TS as ::core::ffi::c_int as usize])
                    .encode
                    .expect("non-null function pointer")(
                    s,
                    (*h).codecs[DS_TS as ::core::ffi::c_int as usize],
                    &raw mut (*cr).tlen as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                );
            }
        }
    }
    if (*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut j: ::core::ffi::c_int = 0;
        uc = (*cr).ntags as ::core::ffi::c_uchar;
        r |= (*(*h).codecs[DS_TC as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_TC as ::core::ffi::c_int as usize],
            &raw mut uc as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        j = 0 as ::core::ffi::c_int;
        while (j as int32_t) < (*cr).ntags {
            let mut i32_0: uint32_t = *(*s).TN.offset(((*cr).TN_idx + j as int32_t) as isize);
            r |= (*(*h).codecs[DS_TN as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_TN as ::core::ffi::c_int as usize],
                &raw mut i32_0 as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
            j += 1;
        }
    } else {
        r |= (*(*h).codecs[DS_TL as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_TL as ::core::ffi::c_int as usize],
            &raw mut (*cr).TL as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    }
    if (*cr).flags & BAM_FUNMAP as int32_t == 0 {
        let mut prev_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut j_0: ::core::ffi::c_int = 0;
        r |= (*(*h).codecs[DS_FN as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_FN as ::core::ffi::c_int as usize],
            &raw mut (*cr).nfeature as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        j_0 = 0 as ::core::ffi::c_int;
        while (j_0 as uint32_t) < (*cr).nfeature {
            let mut f: *mut cram_feature = (*s)
                .features
                .offset((*cr).feature.wrapping_add(j_0 as uint32_t) as isize)
                as *mut cram_feature;
            uc = (*f).X.code as ::core::ffi::c_uchar;
            r |= (*(*h).codecs[DS_FC as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_FC as ::core::ffi::c_int as usize],
                &raw mut uc as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
            i32 = ((*f).X.pos - prev_pos) as int32_t;
            r |= (*(*h).codecs[DS_FP as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_FP as ::core::ffi::c_int as usize],
                &raw mut i32 as *mut ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
            prev_pos = (*f).X.pos;
            match (*f).X.code {
                88 => {
                    uc = (*f).X.base as ::core::ffi::c_uchar;
                    r |= (*(*h).codecs[DS_BS as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_BS as ::core::ffi::c_int as usize],
                        &raw mut uc as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                105 => {
                    uc = (*f).i.base as ::core::ffi::c_uchar;
                    r |= (*(*h).codecs[DS_BA as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_BA as ::core::ffi::c_int as usize],
                        &raw mut uc as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                68 => {
                    i32 = (*f).D.len as int32_t;
                    r |= (*(*h).codecs[DS_DL as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_DL as ::core::ffi::c_int as usize],
                        &raw mut i32 as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                66 => {
                    uc = (*f).B.base as ::core::ffi::c_uchar;
                    r |= (*(*h).codecs[DS_BA as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_BA as ::core::ffi::c_int as usize],
                        &raw mut uc as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                98 => {
                    r |= (*(*h).codecs[DS_BB as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_BB as ::core::ffi::c_int as usize],
                        ((*(*s).seqs_blk).data as *mut ::core::ffi::c_char)
                            .offset((*f).b.seq_idx as isize),
                        (*f).b.len,
                    );
                }
                83 | 73 | 81 => {}
                78 => {
                    i32 = (*f).N.len as int32_t;
                    r |= (*(*h).codecs[DS_RS as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_RS as ::core::ffi::c_int as usize],
                        &raw mut i32 as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                80 => {
                    i32 = (*f).P.len as int32_t;
                    r |= (*(*h).codecs[DS_PD as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_PD as ::core::ffi::c_int as usize],
                        &raw mut i32 as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                72 => {
                    i32 = (*f).H.len as int32_t;
                    r |= (*(*h).codecs[DS_HC as ::core::ffi::c_int as usize])
                        .encode
                        .expect("non-null function pointer")(
                        s,
                        (*h).codecs[DS_HC as ::core::ffi::c_int as usize],
                        &raw mut i32 as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                }
                _ => {
                    hts_log(
                        HTS_LOG_ERROR,
                        b"cram_encode_slice_read\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Unhandled feature code %c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*f).X.code,
                    );
                    return -(1 as ::core::ffi::c_int);
                }
            }
            j_0 += 1;
        }
        r |= (*(*h).codecs[DS_MQ as ::core::ffi::c_int as usize])
            .encode
            .expect("non-null function pointer")(
            s,
            (*h).codecs[DS_MQ as ::core::ffi::c_int as usize],
            &raw mut (*cr).mqual as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        let mut seq: *mut ::core::ffi::c_char =
            ((*(*s).seqs_blk).data as *mut ::core::ffi::c_char).offset((*cr).seq as isize);
        if (*cr).len != 0 {
            r |= (*(*h).codecs[DS_BA as ::core::ffi::c_int as usize])
                .encode
                .expect("non-null function pointer")(
                s,
                (*h).codecs[DS_BA as ::core::ffi::c_int as usize],
                seq,
                (*cr).len as ::core::ffi::c_int,
            );
        }
    }
    return if r != 0 {
        -(1 as ::core::ffi::c_int)
    } else {
        0 as ::core::ffi::c_int
    };
}
// original: cram_compress_slice (htslib/cram/cram_encode.c:803)
unsafe extern "C" fn cram_compress_slice(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
) -> ::core::ffi::c_int {
    let mut level: ::core::ffi::c_int = (*fd).level;
    let mut i: ::core::ffi::c_int = 0;
    let mut method: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << GZIP as ::core::ffi::c_int
        | (1 as ::core::ffi::c_int) << GZIP_RLE as ::core::ffi::c_int;
    let mut methodF: ::core::ffi::c_int = method;
    let mut v31_or_above: ::core::ffi::c_int = ((*fd).version
        >= ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if level > 5 as ::core::ffi::c_int
        && (**(*s).block.offset(0 as ::core::ffi::c_int as isize)).uncomp_size > 500 as int32_t
    {
        cram_compress_block2(
            fd,
            s,
            *(*s).block.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<cram_metrics>(),
            (1 as ::core::ffi::c_int) << GZIP as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    }
    if (*fd).use_bz2 != 0 {
        method |= (1 as ::core::ffi::c_int) << BZIP2 as ::core::ffi::c_int;
    }
    let mut method_rans: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
        << RANS0 as ::core::ffi::c_int
        | (1 as ::core::ffi::c_int) << RANS1 as ::core::ffi::c_int;
    let mut method_ranspr: ::core::ffi::c_int = method_rans;
    if (*fd).use_rans != 0 {
        method_ranspr = (1 as ::core::ffi::c_int) << RANS_PR0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << RANS_PR1 as ::core::ffi::c_int;
        if level > 1 as ::core::ffi::c_int {
            method_ranspr |= (1 as ::core::ffi::c_int) << RANS_PR64 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << RANS_PR9 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << RANS_PR128 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << RANS_PR193 as ::core::ffi::c_int;
        }
        if level > 5 as ::core::ffi::c_int {
            method_ranspr |= (1 as ::core::ffi::c_int) << RANS_PR129 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << RANS_PR192 as ::core::ffi::c_int;
        }
    }
    if (*fd).use_rans != 0 {
        methodF |= if v31_or_above != 0 {
            method_ranspr
        } else {
            method_rans
        };
        method |= if v31_or_above != 0 {
            method_ranspr
        } else {
            method_rans
        };
    }
    let mut method_arith: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*fd).use_arith != 0 {
        method_arith = (1 as ::core::ffi::c_int) << ARITH_PR0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << ARITH_PR1 as ::core::ffi::c_int;
        if level > 1 as ::core::ffi::c_int {
            method_arith = (method_arith as ::core::ffi::c_uint
                | (((1 as ::core::ffi::c_int) << ARITH_PR64 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << ARITH_PR9 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << ARITH_PR128 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << ARITH_PR129 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << ARITH_PR192 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                    | (1 as ::core::ffi::c_uint) << ARITH_PR193 as ::core::ffi::c_int))
                as ::core::ffi::c_int;
        }
    }
    if (*fd).use_arith != 0 && v31_or_above != 0 {
        methodF |= method_arith;
        method |= method_arith;
    }
    if (*fd).use_lzma != 0 {
        method |= (1 as ::core::ffi::c_int) << LZMA as ::core::ffi::c_int;
    }
    methodF = method
        & !((1 as ::core::ffi::c_int) << GZIP as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << BZIP2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << LZMA as ::core::ffi::c_int);
    if level >= 5 as ::core::ffi::c_int {
        method |= (1 as ::core::ffi::c_int) << GZIP_1 as ::core::ffi::c_int;
        methodF = method;
    }
    if level == 1 as ::core::ffi::c_int {
        method &= !((1 as ::core::ffi::c_int) << GZIP as ::core::ffi::c_int);
        method |= (1 as ::core::ffi::c_int) << GZIP_1 as ::core::ffi::c_int;
        methodF = method;
    }
    let mut qmethod: ::core::ffi::c_int = method;
    let mut qmethodF: ::core::ffi::c_int = method;
    if v31_or_above != 0 && (*fd).use_fqz != 0 {
        qmethod |= (1 as ::core::ffi::c_int) << FQZ as ::core::ffi::c_int;
        qmethodF |= (1 as ::core::ffi::c_int) << FQZ as ::core::ffi::c_int;
        if (*fd).level > 4 as ::core::ffi::c_int {
            qmethod |= (1 as ::core::ffi::c_int) << FQZ_b as ::core::ffi::c_int;
            qmethodF |= (1 as ::core::ffi::c_int) << FQZ_b as ::core::ffi::c_int;
        }
        if (*fd).level > 6 as ::core::ffi::c_int {
            qmethod |= (1 as ::core::ffi::c_int) << FQZ_c as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << FQZ_d as ::core::ffi::c_int;
            qmethodF |= (1 as ::core::ffi::c_int) << FQZ_c as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << FQZ_d as ::core::ffi::c_int;
        }
    }
    pthread_mutex_lock(&raw mut (*fd).metrics_lock);
    i = 0 as ::core::ffi::c_int;
    while i < DS_END as ::core::ffi::c_int {
        if !(*c).stats[i as usize].is_null()
            && (*(*c).stats[i as usize]).nvals > 16 as ::core::ffi::c_int
        {
            (*(*fd).m[i as usize]).unpackable = 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
    if cram_compress_block2(
        fd,
        s,
        *(*s).block.offset(DS_IN as ::core::ffi::c_int as isize),
        (*fd).m[DS_IN as ::core::ffi::c_int as usize],
        method,
        level,
    ) != 0
    {
        return -(1 as ::core::ffi::c_int);
    }
    if !((*fd).level == 0 as ::core::ffi::c_int) {
        if (*fd).level == 1 as ::core::ffi::c_int {
            if cram_compress_block2(
                fd,
                s,
                *(*s).block.offset(DS_QS as ::core::ffi::c_int as isize),
                (*fd).m[DS_QS as ::core::ffi::c_int as usize],
                qmethodF,
                1 as ::core::ffi::c_int,
            ) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            i = DS_aux as ::core::ffi::c_int;
            while i <= DS_aux_oz as ::core::ffi::c_int {
                if !(*(*s).block.offset(i as isize)).is_null() {
                    if cram_compress_block2(
                        fd,
                        s,
                        *(*s).block.offset(i as isize),
                        (*fd).m[i as usize],
                        method,
                        1 as ::core::ffi::c_int,
                    ) != 0
                    {
                        return -(1 as ::core::ffi::c_int);
                    }
                }
                i += 1;
            }
        } else if (*fd).level < 3 as ::core::ffi::c_int {
            if cram_compress_block2(
                fd,
                s,
                *(*s).block.offset(DS_QS as ::core::ffi::c_int as isize),
                (*fd).m[DS_QS as ::core::ffi::c_int as usize],
                qmethod,
                1 as ::core::ffi::c_int,
            ) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            if cram_compress_block2(
                fd,
                s,
                *(*s).block.offset(DS_BA as ::core::ffi::c_int as isize),
                (*fd).m[DS_BA as ::core::ffi::c_int as usize],
                method,
                1 as ::core::ffi::c_int,
            ) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            if !(*(*s).block.offset(DS_BB as ::core::ffi::c_int as isize)).is_null() {
                if cram_compress_block2(
                    fd,
                    s,
                    *(*s).block.offset(DS_BB as ::core::ffi::c_int as isize),
                    (*fd).m[DS_BB as ::core::ffi::c_int as usize],
                    method,
                    1 as ::core::ffi::c_int,
                ) != 0
                {
                    return -(1 as ::core::ffi::c_int);
                }
            }
            i = DS_aux as ::core::ffi::c_int;
            while i <= DS_aux_oz as ::core::ffi::c_int {
                if !(*(*s).block.offset(i as isize)).is_null() {
                    if cram_compress_block2(
                        fd,
                        s,
                        *(*s).block.offset(i as isize),
                        (*fd).m[i as usize],
                        method,
                        level,
                    ) != 0
                    {
                        return -(1 as ::core::ffi::c_int);
                    }
                }
                i += 1;
            }
        } else {
            if cram_compress_block2(
                fd,
                s,
                *(*s).block.offset(DS_QS as ::core::ffi::c_int as isize),
                (*fd).m[DS_QS as ::core::ffi::c_int as usize],
                qmethod,
                level,
            ) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            if cram_compress_block2(
                fd,
                s,
                *(*s).block.offset(DS_BA as ::core::ffi::c_int as isize),
                (*fd).m[DS_BA as ::core::ffi::c_int as usize],
                method,
                level,
            ) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            if !(*(*s).block.offset(DS_BB as ::core::ffi::c_int as isize)).is_null() {
                if cram_compress_block2(
                    fd,
                    s,
                    *(*s).block.offset(DS_BB as ::core::ffi::c_int as isize),
                    (*fd).m[DS_BB as ::core::ffi::c_int as usize],
                    method,
                    level,
                ) != 0
                {
                    return -(1 as ::core::ffi::c_int);
                }
            }
            i = DS_aux as ::core::ffi::c_int;
            while i <= DS_aux_oz as ::core::ffi::c_int {
                if !(*(*s).block.offset(i as isize)).is_null() {
                    if cram_compress_block2(
                        fd,
                        s,
                        *(*s).block.offset(i as isize),
                        (*fd).m[i as usize],
                        method,
                        level,
                    ) != 0
                    {
                        return -(1 as ::core::ffi::c_int);
                    }
                }
                i += 1;
            }
        }
    }
    let mut method_rn: ::core::ffi::c_int = method
        & !(method_rans
            | method_ranspr
            | (1 as ::core::ffi::c_int) << GZIP_RLE as ::core::ffi::c_int);
    if (*fd).version
        >= ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int
        && (*fd).use_tok != 0
    {
        method_rn |= if (*fd).use_arith != 0 {
            (1 as ::core::ffi::c_int) << TOKA as ::core::ffi::c_int
        } else {
            (1 as ::core::ffi::c_int) << TOK3 as ::core::ffi::c_int
        };
    }
    if cram_compress_block2(
        fd,
        s,
        *(*s).block.offset(DS_RN as ::core::ffi::c_int as isize),
        (*fd).m[DS_RN as ::core::ffi::c_int as usize],
        method_rn,
        level,
    ) != 0
    {
        return -(1 as ::core::ffi::c_int);
    }
    if !(*(*s).block.offset(DS_NS as ::core::ffi::c_int as isize)).is_null()
        && *(*s).block.offset(DS_NS as ::core::ffi::c_int as isize)
            != *(*s).block.offset(0 as ::core::ffi::c_int as isize)
    {
        if cram_compress_block2(
            fd,
            s,
            *(*s).block.offset(DS_NS as ::core::ffi::c_int as isize),
            (*fd).m[DS_NS as ::core::ffi::c_int as usize],
            method,
            level,
        ) != 0
        {
            return -(1 as ::core::ffi::c_int);
        }
    }
    let mut i_0: ::core::ffi::c_int = 0;
    i_0 = DS_END as ::core::ffi::c_int;
    while (i_0 as int32_t) < (*(*s).hdr).num_blocks {
        if !((*(*s).block.offset(i_0 as isize)).is_null()
            || *(*s).block.offset(i_0 as isize)
                == *(*s).block.offset(0 as ::core::ffi::c_int as isize))
        {
            if !((**(*s).block.offset(i_0 as isize)).method as ::core::ffi::c_int
                != RAW as ::core::ffi::c_int)
            {
                if cram_compress_block2(
                    fd,
                    s,
                    *(*s).block.offset(i_0 as isize),
                    (**(*s).block.offset(i_0 as isize)).m,
                    method,
                    level,
                ) != 0
                {
                    return -(1 as ::core::ffi::c_int);
                }
            }
        }
        i_0 += 1;
    }
    let mut i_1: ::core::ffi::c_int = 0;
    i_1 = 1 as ::core::ffi::c_int;
    while (i_1 as int32_t) < (*(*s).hdr).num_blocks && i_1 < DS_END as ::core::ffi::c_int {
        if !((*(*s).block.offset(i_1 as isize)).is_null()
            || *(*s).block.offset(i_1 as isize)
                == *(*s).block.offset(0 as ::core::ffi::c_int as isize))
        {
            if !((**(*s).block.offset(i_1 as isize)).method as ::core::ffi::c_int
                != RAW as ::core::ffi::c_int)
            {
                if cram_compress_block2(
                    fd,
                    s,
                    *(*s).block.offset(i_1 as isize),
                    (*fd).m[i_1 as usize],
                    methodF,
                    level,
                ) != 0
                {
                    return -(1 as ::core::ffi::c_int);
                }
            }
        }
        i_1 += 1;
    }
    return 0 as ::core::ffi::c_int;
}
// original: cram_allocate_block (htslib/cram/cram_encode.c:1005)
unsafe extern "C" fn cram_allocate_block(
    mut codec: *mut cram_codec,
    mut s: *mut cram_slice,
    mut ds_id: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if codec.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    match (*codec).codec as ::core::ffi::c_uint {
        2 | 3 | 6 | 7 | 8 | 9 => {
            (*codec).out = *(*s).block.offset(0 as ::core::ffi::c_int as isize);
        }
        43 | 44 => {
            (*codec).out = ::core::ptr::null_mut::<cram_block>();
        }
        1 | 41 | 42 => {
            let ref mut fresh148 = *(*s).block.offset(ds_id as isize);
            *fresh148 = cram_new_block(EXTERNAL, ds_id);
            if (*fresh148).is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*codec).u.external.content_id = ds_id as int32_t;
            (*codec).out = *(*s).block.offset(ds_id as isize);
        }
        5 => {
            let ref mut fresh149 = *(*s).block.offset(ds_id as isize);
            *fresh149 = cram_new_block(EXTERNAL, ds_id);
            if (*fresh149).is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*codec).u.byte_array_stop.content_id = ds_id as int32_t;
            (*codec).out = *(*s).block.offset(ds_id as isize);
        }
        4 => {
            let mut bal: *mut cram_codec = (*codec).u.e_byte_array_len.len_codec as *mut cram_codec;
            if cram_allocate_block(bal, s, (*bal).u.external.content_id as ::core::ffi::c_int) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            bal = (*codec).u.e_byte_array_len.val_codec as *mut cram_codec;
            if cram_allocate_block(bal, s, (*bal).u.external.content_id as ::core::ffi::c_int) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
        }
        52 => {
            if cram_allocate_block((*codec).u.e_xrle.len_codec as *mut cram_codec, s, ds_id) != 0 {
                return -(1 as ::core::ffi::c_int);
            }
            if cram_allocate_block((*codec).u.e_xrle.lit_codec as *mut cram_codec, s, ds_id) != 0 {
                return -(1 as ::core::ffi::c_int);
            }
        }
        51 => {
            if cram_allocate_block((*codec).u.e_xpack.sub_codec as *mut cram_codec, s, ds_id) != 0 {
                return -(1 as ::core::ffi::c_int);
            }
            (*codec).out = cram_new_block(FILE_HEADER, 0 as ::core::ffi::c_int);
            if (*codec).out.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        }
        53 => {
            if cram_allocate_block((*codec).u.e_xdelta.sub_codec as *mut cram_codec, s, ds_id) != 0
            {
                return -(1 as ::core::ffi::c_int);
            }
            (*codec).out = cram_new_block(FILE_HEADER, 0 as ::core::ffi::c_int);
            if (*codec).out.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
// original: cram_encode_slice (htslib/cram/cram_encode.c:1096)
unsafe extern "C" fn cram_encode_slice(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut h: *mut cram_block_compression_hdr,
    mut s: *mut cram_slice,
    mut embed_ref: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rec: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut last_pos: int64_t = 0;
    let mut id: cram_DS_ID = DS_CORE;
    (*(*s).hdr).ref_base_id =
        (if embed_ref > 0 as ::core::ffi::c_int && (*(*s).hdr).ref_seq_span > 0 as int64_t {
            DS_ref as ::core::ffi::c_int
        } else if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            -(1 as ::core::ffi::c_int)
        }) as int32_t;
    (*(*s).hdr).record_counter = (*c).num_records as int64_t + (*c).record_counter;
    (*c).num_records = ((*c).num_records as ::core::ffi::c_int
        + (*(*s).hdr).num_records as ::core::ffi::c_int) as int32_t;
    let mut ntags: ::core::ffi::c_int = (if !(*c).tags_used.is_null() {
        (*(*c).tags_used).n_occupied as ::core::ffi::c_uint
    } else {
        0 as ::core::ffi::c_uint
    }) as ::core::ffi::c_int;
    (*s).block = calloc(
        (DS_END as ::core::ffi::c_int + ntags * 2 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut cram_block>() as size_t,
    ) as *mut *mut cram_block;
    (*(*s).hdr).block_content_ids = malloc(
        (DS_END as ::core::ffi::c_int as size_t)
            .wrapping_mul(::core::mem::size_of::<int32_t>() as size_t),
    ) as *mut int32_t;
    if (*s).block.is_null() || (*(*s).hdr).block_content_ids.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    let ref mut fresh134 = *(*s).block.offset(0 as ::core::ffi::c_int as isize);
    *fresh134 = cram_new_block(CORE, 0 as ::core::ffi::c_int);
    if (*fresh134).is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if (*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        if (*(*h).codecs[DS_TN as ::core::ffi::c_int as usize]).codec as ::core::ffi::c_uint
            == E_EXTERNAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let ref mut fresh135 = *(*s).block.offset(DS_TN as ::core::ffi::c_int as isize);
            *fresh135 = cram_new_block(EXTERNAL, DS_TN as ::core::ffi::c_int);
            if (*fresh135).is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*(*h).codecs[DS_TN as ::core::ffi::c_int as usize])
                .u
                .external
                .content_id = DS_TN as ::core::ffi::c_int as int32_t;
        } else {
            let ref mut fresh136 = *(*s).block.offset(DS_TN as ::core::ffi::c_int as isize);
            *fresh136 = *(*s).block.offset(0 as ::core::ffi::c_int as isize);
        }
    }
    if embed_ref > 0 as ::core::ffi::c_int {
        let ref mut fresh137 = *(*s).block.offset(DS_ref as ::core::ffi::c_int as isize);
        *fresh137 = cram_new_block(EXTERNAL, DS_ref as ::core::ffi::c_int);
        if (*fresh137).is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*s).ref_id = DS_ref as ::core::ffi::c_int;
        if block_append(
            *(*s).block.offset(DS_ref as ::core::ffi::c_int as isize),
            (*c).ref_0
                .offset((*(*s).hdr).ref_seq_start as isize)
                .offset(-((*c).ref_start as isize)) as *const ::core::ffi::c_void,
            (*(*s).hdr).ref_seq_span as size_t,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    }
    id = DS_QS;
    while (id as ::core::ffi::c_uint) < DS_TN as ::core::ffi::c_int as ::core::ffi::c_uint {
        if cram_allocate_block((*h).codecs[id as usize], s, id as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
        id += 1;
    }
    if !(*c).tags_used.is_null() {
        let mut n: ::core::ffi::c_int = 0;
        (*(*s).hdr).num_blocks = DS_END as ::core::ffi::c_int as int32_t;
        n = 0 as ::core::ffi::c_int;
        while n < (*s).naux_block {
            let fresh138 = (*(*s).hdr).num_blocks;
            (*(*s).hdr).num_blocks = (*(*s).hdr).num_blocks + 1;
            let ref mut fresh139 = *(*s).block.offset(fresh138 as isize);
            *fresh139 = *(*s).aux_block.offset(n as isize);
            let ref mut fresh140 = *(*s).aux_block.offset(n as isize);
            *fresh140 = ::core::ptr::null_mut::<cram_block>();
            n += 1;
        }
    }
    last_pos = (*(*s).hdr).ref_seq_start;
    rec = 0 as ::core::ffi::c_int;
    while (rec as int32_t) < (*(*s).hdr).num_records {
        let mut cr: *mut cram_record = (*s).crecs.offset(rec as isize) as *mut cram_record;
        if cram_encode_slice_read(fd, c, h, s, cr, &raw mut last_pos) == -(1 as ::core::ffi::c_int)
        {
            return -(1 as ::core::ffi::c_int);
        }
        rec += 1;
    }
    (**(*s).block.offset(0 as ::core::ffi::c_int as isize)).uncomp_size =
        (**(*s).block.offset(0 as ::core::ffi::c_int as isize))
            .byte
            .wrapping_add(
                ((**(*s).block.offset(0 as ::core::ffi::c_int as isize)).bit
                    < 7 as ::core::ffi::c_int) as ::core::ffi::c_int as size_t,
            ) as int32_t;
    (**(*s).block.offset(0 as ::core::ffi::c_int as isize)).comp_size =
        (**(*s).block.offset(0 as ::core::ffi::c_int as isize)).uncomp_size;
    if !(*(*s).block.offset(DS_IN as ::core::ffi::c_int as isize)).is_null() {
        cram_free_block(*(*s).block.offset(DS_IN as ::core::ffi::c_int as isize));
    }
    let ref mut fresh141 = *(*s).block.offset(DS_IN as ::core::ffi::c_int as isize);
    *fresh141 = (*s).base_blk;
    (*s).base_blk = ::core::ptr::null_mut::<cram_block>();
    if !(*(*s).block.offset(DS_QS as ::core::ffi::c_int as isize)).is_null() {
        cram_free_block(*(*s).block.offset(DS_QS as ::core::ffi::c_int as isize));
    }
    let ref mut fresh142 = *(*s).block.offset(DS_QS as ::core::ffi::c_int as isize);
    *fresh142 = (*s).qual_blk;
    (*s).qual_blk = ::core::ptr::null_mut::<cram_block>();
    if !(*(*s).block.offset(DS_RN as ::core::ffi::c_int as isize)).is_null() {
        cram_free_block(*(*s).block.offset(DS_RN as ::core::ffi::c_int as isize));
    }
    let ref mut fresh143 = *(*s).block.offset(DS_RN as ::core::ffi::c_int as isize);
    *fresh143 = (*s).name_blk;
    (*s).name_blk = ::core::ptr::null_mut::<cram_block>();
    if !(*(*s).block.offset(DS_SC as ::core::ffi::c_int as isize)).is_null() {
        cram_free_block(*(*s).block.offset(DS_SC as ::core::ffi::c_int as isize));
    }
    let ref mut fresh144 = *(*s).block.offset(DS_SC as ::core::ffi::c_int as isize);
    *fresh144 = (*s).soft_blk;
    (*s).soft_blk = ::core::ptr::null_mut::<cram_block>();
    id = DS_QS;
    while (id as ::core::ffi::c_uint) < DS_TN as ::core::ffi::c_int as ::core::ffi::c_uint {
        if !(*h).codecs[id as usize].is_null() && (*(*h).codecs[id as usize]).flush.is_some() {
            (*(*h).codecs[id as usize])
                .flush
                .expect("non-null function pointer")((*h).codecs[id as usize]);
        }
        id += 1;
    }
    id = DS_aux;
    while (id as ::core::ffi::c_uint) < (*(*s).hdr).num_blocks as ::core::ffi::c_uint {
        if !((*(*s).block.offset(id as isize)).is_null()
            || *(*s).block.offset(id as isize)
                == *(*s).block.offset(0 as ::core::ffi::c_int as isize))
        {
            if (**(*s).block.offset(id as isize)).uncomp_size == 0 as int32_t {
                let ref mut fresh145 = (**(*s).block.offset(id as isize)).uncomp_size;
                *fresh145 = (**(*s).block.offset(id as isize)).byte as int32_t;
                (**(*s).block.offset(id as isize)).comp_size = *fresh145;
            }
        }
        id += 1;
    }
    if cram_compress_slice(fd, c, s) == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    (*(*s).hdr).block_content_ids = realloc(
        (*(*s).hdr).block_content_ids as *mut ::core::ffi::c_void,
        ((*(*s).hdr).num_blocks as size_t)
            .wrapping_mul(::core::mem::size_of::<int32_t>() as size_t),
    ) as *mut int32_t;
    if (*(*s).hdr).block_content_ids.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    j = 1 as ::core::ffi::c_int;
    i = j;
    while (i as int32_t) < (*(*s).hdr).num_blocks {
        if !((*(*s).block.offset(i as isize)).is_null()
            || *(*s).block.offset(i as isize)
                == *(*s).block.offset(0 as ::core::ffi::c_int as isize))
        {
            if (**(*s).block.offset(i as isize)).uncomp_size == 0 as int32_t {
                cram_free_block(*(*s).block.offset(i as isize));
                let ref mut fresh146 = *(*s).block.offset(i as isize);
                *fresh146 = ::core::ptr::null_mut::<cram_block>();
            } else {
                let ref mut fresh147 = *(*s).block.offset(j as isize);
                *fresh147 = *(*s).block.offset(i as isize);
                *(*(*s).hdr)
                    .block_content_ids
                    .offset((j - 1 as ::core::ffi::c_int) as isize) =
                    (**(*s).block.offset(i as isize)).content_id;
                j += 1;
            }
        }
        i += 1;
    }
    (*(*s).hdr).num_content_ids = (j - 1 as ::core::ffi::c_int) as int32_t;
    (*(*s).hdr).num_blocks = j as int32_t;
    (*s).hdr_block = cram_encode_slice_header(fd, s);
    if (*s).hdr_block.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    return if r != 0 {
        -(1 as ::core::ffi::c_int)
    } else {
        0 as ::core::ffi::c_int
    };
}
#[inline]
unsafe extern "C" fn bam_data_end(mut b: *mut bam1_t) -> *const ::core::ffi::c_char {
    return ((*b).data as *const ::core::ffi::c_char).offset((*b).l_data as isize);
}
#[inline]
unsafe extern "C" fn bam_aux2i_end(
    mut aux: *const uint8_t,
    mut aux_end: *const uint8_t,
) -> ::core::ffi::c_int {
    let fresh162 = aux;
    aux = aux.offset(1);
    let mut type_0: ::core::ffi::c_int = *fresh162 as ::core::ffi::c_int;
    match type_0 {
        99 => {
            if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 1 as ::core::ffi::c_long {
                *__errno_location() = EINVAL;
                return 0 as ::core::ffi::c_int;
            }
            return *(aux as *mut int8_t) as ::core::ffi::c_int;
        }
        67 => {
            if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 1 as ::core::ffi::c_long {
                *__errno_location() = EINVAL;
                return 0 as ::core::ffi::c_int;
            }
            return *aux as ::core::ffi::c_int;
        }
        115 => {
            if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 2 as ::core::ffi::c_long {
                *__errno_location() = EINVAL;
                return 0 as ::core::ffi::c_int;
            }
            return le_to_i16(aux) as ::core::ffi::c_int;
        }
        83 => {
            if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 2 as ::core::ffi::c_long {
                *__errno_location() = EINVAL;
                return 0 as ::core::ffi::c_int;
            }
            return le_to_u16(aux) as ::core::ffi::c_int;
        }
        105 => {
            if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 4 as ::core::ffi::c_long {
                *__errno_location() = EINVAL;
                return 0 as ::core::ffi::c_int;
            }
            return le_to_i32(aux) as ::core::ffi::c_int;
        }
        73 => {
            if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 4 as ::core::ffi::c_long {
                *__errno_location() = EINVAL;
                return 0 as ::core::ffi::c_int;
            }
            return le_to_u32(aux) as ::core::ffi::c_int;
        }
        _ => {
            *__errno_location() = EINVAL;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn expected_template_count(mut b: *mut bam_seq_t) -> ::core::ffi::c_int {
    let mut expected: ::core::ffi::c_int =
        if (*b).core.flag as ::core::ffi::c_int & BAM_FPAIRED != 0 {
            2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    let mut TC: *mut uint8_t = bam_aux_get(b, b"TC\0" as *const u8 as *const ::core::ffi::c_char);
    if !TC.is_null() {
        let mut n: ::core::ffi::c_int =
            bam_aux2i_end(TC, bam_data_end(b as *mut bam1_t) as *mut uint8_t);
        if expected < n {
            expected = n;
        }
    }
    if TC.is_null() && !bam_aux_get(b, b"SA\0" as *const u8 as *const ::core::ffi::c_char).is_null()
    {
        expected = INT_MAX;
    }
    return expected;
}
// original: lossy_read_names (htslib/cram/cram_encode.c:1340)
unsafe extern "C" fn lossy_read_names(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut bam_start: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut r1: ::core::ffi::c_int = 0;
    let mut r2: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    r2 = 0 as ::core::ffi::c_int;
    while (r2 as int32_t) < (*(*s).hdr).num_records {
        (*(*s).crecs.offset(r2 as isize)).cram_flags = 0 as ::core::ffi::c_int as int32_t;
        r2 += 1;
    }
    if (*fd).lossy_read_names == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut names: *mut kh_m_s2u64_t = kh_init_m_s2u64();
    if !names.is_null() {
        r1 = bam_start;
        r2 = 0 as ::core::ffi::c_int;
        loop {
            if !((r2 as int32_t) < (*(*s).hdr).num_records) {
                current_block = 6669252993407410313;
                break;
            }
            let mut b: *mut bam_seq_t = *(*c).bams.offset(r1 as isize);
            let mut k: khint_t = 0;
            let mut n: ::core::ffi::c_int = 0;
            let mut e: uint64_t = 0;
            let mut u: C2RustUnnamed_19 = C2RustUnnamed_19 { i64_0: 0 };
            e = expected_template_count(b) as uint64_t;
            u.counts.e = e as int32_t;
            u.counts.c = 1 as ::core::ffi::c_int as int32_t;
            k = kh_put_m_s2u64(
                names,
                (*b).data as *mut ::core::ffi::c_char as kh_cstr_t,
                &raw mut n,
            );
            if n == -(1 as ::core::ffi::c_int) {
                current_block = 13998712643811550316;
                break;
            }
            if n == 0 as ::core::ffi::c_int {
                u.i64_0 = *(*names).vals.offset(k as isize);
                if u.counts.e as uint64_t != e {
                    *(*names).vals.offset(k as isize) = 0 as uint64_t;
                } else {
                    u.counts.c += 1;
                    if u.counts.e == u.counts.c {
                        *(*names).vals.offset(k as isize) = -(1 as ::core::ffi::c_int) as uint64_t;
                    } else {
                        *(*names).vals.offset(k as isize) = u.i64_0;
                    }
                }
            } else {
                *(*names).vals.offset(k as isize) = u.i64_0;
            }
            r1 += 1;
            r2 += 1;
        }
        match current_block {
            13998712643811550316 => {}
            _ => {
                r1 = bam_start;
                r2 = 0 as ::core::ffi::c_int;
                loop {
                    if !((r2 as int32_t) < (*(*s).hdr).num_records) {
                        current_block = 1538046216550696469;
                        break;
                    }
                    let mut cr: *mut cram_record =
                        (*s).crecs.offset(r2 as isize) as *mut cram_record;
                    let mut b_0: *mut bam_seq_t = *(*c).bams.offset(r1 as isize);
                    let mut k_0: khint_t = 0;
                    k_0 =
                        kh_get_m_s2u64(names, (*b_0).data as *mut ::core::ffi::c_char as kh_cstr_t);
                    if k_0 == (*names).n_buckets {
                        current_block = 13998712643811550316;
                        break;
                    }
                    if *(*names).vals.offset(k_0 as isize) == -(1 as ::core::ffi::c_int) as uint64_t
                    {
                        (*cr).cram_flags = CRAM_FLAG_DISCARD_NAME as int32_t;
                    }
                    r1 += 1;
                    r2 += 1;
                }
                match current_block {
                    13998712643811550316 => {}
                    _ => {
                        ret = 0 as ::core::ffi::c_int;
                    }
                }
            }
        }
    }
    if !names.is_null() {
        kh_destroy_m_s2u64(names);
    }
    return ret;
}
// original: add_read_names (htslib/cram/cram_encode.c:1433)
unsafe extern "C" fn add_read_names(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut bam_start: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut r1: ::core::ffi::c_int = 0;
    let mut r2: ::core::ffi::c_int = 0;
    let mut keep_names: ::core::ffi::c_int = ((*fd).lossy_read_names == 0) as ::core::ffi::c_int;
    r1 = bam_start;
    r2 = 0 as ::core::ffi::c_int;
    loop {
        if !(r1 < (*c).curr_c_rec && (r2 as int32_t) < (*(*s).hdr).num_records) {
            current_block = 3512920355445576850;
            break;
        }
        let mut cr: *mut cram_record = (*s).crecs.offset(r2 as isize) as *mut cram_record;
        let mut b: *mut bam_seq_t = *(*c).bams.offset(r1 as isize);
        (*cr).name = (*(*s).name_blk).byte as int32_t;
        if (*cr).cram_flags & CRAM_FLAG_DETACHED as int32_t != 0 || keep_names != 0 {
            if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int
                && (*cr).cram_flags & CRAM_FLAG_MATE_DOWNSTREAM as int32_t != 0
                && (*cr).mate_line != 0
            {
                if block_append(
                    (*s).name_blk,
                    b"\0\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    1 as size_t,
                ) < 0 as ::core::ffi::c_int
                {
                    current_block = 120651212025574253;
                    break;
                }
                (*cr).name_len = 1 as ::core::ffi::c_int as int32_t;
            } else {
                if block_append(
                    (*s).name_blk,
                    (*b).data as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    ((*b).core.l_qname as ::core::ffi::c_int
                        - (*b).core.l_extranul as ::core::ffi::c_int) as size_t,
                ) < 0 as ::core::ffi::c_int
                {
                    current_block = 120651212025574253;
                    break;
                }
                (*cr).name_len = ((*b).core.l_qname as ::core::ffi::c_int
                    - (*b).core.l_extranul as ::core::ffi::c_int)
                    as int32_t;
            }
        } else {
            (*cr).name_len = 0 as ::core::ffi::c_int as int32_t;
        }
        if cram_stats_add(
            (*c).stats[DS_RN as ::core::ffi::c_int as usize],
            (*cr).name_len as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            current_block = 120651212025574253;
            break;
        }
        r1 += 1;
        r2 += 1;
    }
    match current_block {
        120651212025574253 => return -(1 as ::core::ffi::c_int),
        _ => return 0 as ::core::ffi::c_int,
    };
}
#[inline]
unsafe extern "C" fn next_cigar_op(
    mut cigar: *mut uint32_t,
    mut ncigar: uint32_t,
    mut skip: *mut ::core::ffi::c_int,
    mut spos: *mut ::core::ffi::c_int,
    mut cig_ind: *mut uint32_t,
    mut cig_op: *mut uint32_t,
    mut cig_len: *mut uint32_t,
) -> ::core::ffi::c_int {
    loop {
        while *cig_len == 0 as uint32_t {
            if *cig_ind < ncigar {
                *cig_op = *cigar.offset(*cig_ind as isize) & BAM_CIGAR_MASK as uint32_t;
                *cig_len = *cigar.offset(*cig_ind as isize) >> BAM_CIGAR_SHIFT;
                *cig_ind = (*cig_ind).wrapping_add(1);
            } else {
                return -(1 as ::core::ffi::c_int);
            }
        }
        if *skip.offset(*cig_op as isize) != 0 {
            *spos = (*spos as ::core::ffi::c_uint).wrapping_add(
                ((BAM_CIGAR_TYPE >> (*cig_op << 1 as ::core::ffi::c_int)
                    & 3 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int) as uint32_t)
                    .wrapping_mul(*cig_len) as ::core::ffi::c_uint,
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
            *cig_len = 0 as uint32_t;
        } else {
            *cig_len = (*cig_len).wrapping_sub(1);
            break;
        }
    }
    return *cig_op as ::core::ffi::c_int;
}
#[inline]
// original: extend_ref (htslib/cram/cram_encode.c:1504)
unsafe extern "C" fn extend_ref(
    mut ref_0: *mut *mut ::core::ffi::c_char,
    mut hist: *mut *mut [uint32_t; 5],
    mut pos: hts_pos_t,
    mut ref_start: hts_pos_t,
    mut ref_end: *mut hts_pos_t,
    mut ref_end_alloc: *mut hts_pos_t,
) -> ::core::ffi::c_int {
    if *ref_end < pos && pos < *ref_end_alloc {
        *ref_end = pos;
    }
    if pos < ref_start {
        return -(1 as ::core::ffi::c_int);
    }
    if pos < *ref_end_alloc {
        return 0 as ::core::ffi::c_int;
    }
    if pos - ref_start > UINT_MAX as hts_pos_t {
        return -(2 as ::core::ffi::c_int);
    }
    let mut old_end: hts_pos_t = if *ref_end_alloc != 0 {
        *ref_end_alloc
    } else {
        ref_start
    };
    let mut new_end: hts_pos_t = ((ref_start + 1000 as hts_pos_t) as ::core::ffi::c_double
        + (pos - ref_start) as ::core::ffi::c_double * 1.5f64)
        as hts_pos_t;
    if (new_end - ref_start) as usize
        > (UINT_MAX as usize)
            .wrapping_div(::core::mem::size_of::<[uint32_t; 5]>() as usize)
            .wrapping_div(2 as usize)
    {
        return -(2 as ::core::ffi::c_int);
    }
    let mut tmp: *mut ::core::ffi::c_char = realloc(
        *ref_0 as *mut ::core::ffi::c_void,
        (new_end - ref_start + 1 as hts_pos_t) as size_t,
    ) as *mut ::core::ffi::c_char;
    if tmp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    *ref_0 = tmp;
    let mut tmp5: *mut [uint32_t; 5] = realloc(
        &raw mut **hist as *mut uint32_t as *mut ::core::ffi::c_void,
        ((new_end - ref_start) as size_t)
            .wrapping_mul(::core::mem::size_of::<[uint32_t; 5]>() as size_t),
    ) as *mut [uint32_t; 5];
    if tmp5.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    *hist = tmp5;
    *ref_end_alloc = new_end;
    old_end = (old_end as ::core::ffi::c_long - ref_start as ::core::ffi::c_long) as hts_pos_t;
    new_end = (new_end as ::core::ffi::c_long - ref_start as ::core::ffi::c_long) as hts_pos_t;
    memset(
        (*ref_0).offset(old_end as isize) as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (new_end - old_end) as size_t,
    );
    memset(
        (*hist).offset(old_end as isize) as *mut [uint32_t; 5] as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ((new_end - old_end) as size_t)
            .wrapping_mul(::core::mem::size_of::<[uint32_t; 5]>() as size_t),
    );
    if *ref_end < pos {
        *ref_end = pos;
    }
    return 0 as ::core::ffi::c_int;
}
// original: cram_add_to_ref_MD (htslib/cram/cram_encode.c:1553)
unsafe extern "C" fn cram_add_to_ref_MD(
    mut b: *mut bam1_t,
    mut ref_0: *mut *mut ::core::ffi::c_char,
    mut hist: *mut *mut [uint32_t; 5],
    mut ref_start: hts_pos_t,
    mut ref_end: *mut hts_pos_t,
    mut ref_end_alloc: *mut hts_pos_t,
    mut MD: *const uint8_t,
) -> ::core::ffi::c_int {
    let mut seq: *mut uint8_t = (*b)
        .data
        .offset(((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize)
        .offset((*b).core.l_qname as ::core::ffi::c_int as isize);
    let mut cigar: *mut uint32_t =
        (*b).data
            .offset((*b).core.l_qname as ::core::ffi::c_int as isize) as *mut uint32_t;
    let mut ncigar: uint32_t = (*b).core.n_cigar;
    let mut cig_op: uint32_t = 0 as uint32_t;
    let mut cig_len: uint32_t = 0 as uint32_t;
    let mut cig_ind: uint32_t = 0 as uint32_t;
    let mut rlen: hts_pos_t = bam_cigar2rlen(
        (*b).core.n_cigar as ::core::ffi::c_int,
        (*b).data
            .offset((*b).core.l_qname as ::core::ffi::c_int as isize) as *mut uint32_t,
    );
    let mut rseq_end: hts_pos_t = (*b).core.pos
        + (if rlen != 0 {
            rlen
        } else {
            (*b).core.l_qseq as hts_pos_t
        });
    if (*b).core.l_qseq == 0
        && extend_ref(ref_0, hist, rseq_end, ref_start, ref_end, ref_end_alloc)
            < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    let mut iseq: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_op: ::core::ffi::c_int = 0;
    let mut iref: hts_pos_t = (*b).core.pos - ref_start;
    static mut cig_skip: [::core::ffi::c_int; 16] = [
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ];
    while (iseq as int32_t) < (*b).core.l_qseq && *MD as ::core::ffi::c_int != 0 {
        if *(*__ctype_b_loc()).offset(*MD as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        {
            let mut overflow: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut len: ::core::ffi::c_int = hts_str2uint(
                MD as *mut ::core::ffi::c_char,
                &raw mut MD as *mut *mut ::core::ffi::c_char,
                31 as ::core::ffi::c_int,
                &raw mut overflow,
            ) as ::core::ffi::c_int;
            if overflow != 0
                || extend_ref(
                    ref_0,
                    hist,
                    iref + ref_start + len as hts_pos_t,
                    ref_start,
                    ref_end,
                    ref_end_alloc,
                ) < 0 as ::core::ffi::c_int
            {
                return -(1 as ::core::ffi::c_int);
            }
            while (iseq as int32_t) < (*b).core.l_qseq && len != 0 {
                next_op = next_cigar_op(
                    cigar,
                    ncigar,
                    &raw mut cig_skip as *mut ::core::ffi::c_int,
                    &raw mut iseq,
                    &raw mut cig_ind,
                    &raw mut cig_op,
                    &raw mut cig_len,
                );
                if next_op < 0 as ::core::ffi::c_int {
                    return -(1 as ::core::ffi::c_int);
                }
                if next_op != BAM_CMATCH && next_op != BAM_CEQUAL {
                    hts_log(
                        HTS_LOG_INFO,
                        b"cram_add_to_ref_MD\0" as *const u8 as *const ::core::ffi::c_char,
                        b"MD:Z and CIGAR are incompatible for record %s\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*b).data as *mut ::core::ffi::c_char,
                    );
                    return -(1 as ::core::ffi::c_int);
                }
                cig_len = cig_len.wrapping_add(1);
                loop {
                    cig_len = cig_len.wrapping_sub(1);
                    let fresh166 = iref;
                    iref = iref + 1;
                    *(*ref_0).offset(fresh166 as isize) =
                        *(&raw const seq_nt16_str as *const ::core::ffi::c_char).offset(
                            (*seq.offset((iseq >> 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                >> ((!iseq & 1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int)
                                & 0xf as ::core::ffi::c_int) as isize,
                        );
                    iseq += 1;
                    len -= 1;
                    if !(cig_len != 0 && (iseq as int32_t) < (*b).core.l_qseq && len != 0) {
                        break;
                    }
                }
            }
            if len > 0 as ::core::ffi::c_int {
                return -(1 as ::core::ffi::c_int);
            }
        } else if *MD as ::core::ffi::c_int == '^' as i32 {
            MD = MD.offset(1);
            while *(*__ctype_b_loc()).offset(*MD as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                if extend_ref(
                    ref_0,
                    hist,
                    iref + ref_start,
                    ref_start,
                    ref_end,
                    ref_end_alloc,
                ) < 0 as ::core::ffi::c_int
                {
                    return -(1 as ::core::ffi::c_int);
                }
                next_op = next_cigar_op(
                    cigar,
                    ncigar,
                    &raw mut cig_skip as *mut ::core::ffi::c_int,
                    &raw mut iseq,
                    &raw mut cig_ind,
                    &raw mut cig_op,
                    &raw mut cig_len,
                );
                if next_op < 0 as ::core::ffi::c_int {
                    return -(1 as ::core::ffi::c_int);
                }
                if next_op != BAM_CDEL {
                    hts_log(
                        HTS_LOG_INFO,
                        b"cram_add_to_ref_MD\0" as *const u8 as *const ::core::ffi::c_char,
                        b"MD:Z and CIGAR are incompatible\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    return -(1 as ::core::ffi::c_int);
                }
                let fresh167 = MD;
                MD = MD.offset(1);
                let fresh168 = iref;
                iref = iref + 1;
                *(*ref_0).offset(fresh168 as isize) = (*fresh167 as ::core::ffi::c_int
                    & !(0x20 as ::core::ffi::c_int))
                    as ::core::ffi::c_char;
            }
        } else {
            if extend_ref(
                ref_0,
                hist,
                iref + ref_start,
                ref_start,
                ref_end,
                ref_end_alloc,
            ) < 0 as ::core::ffi::c_int
            {
                return -(1 as ::core::ffi::c_int);
            }
            next_op = next_cigar_op(
                cigar,
                ncigar,
                &raw mut cig_skip as *mut ::core::ffi::c_int,
                &raw mut iseq,
                &raw mut cig_ind,
                &raw mut cig_op,
                &raw mut cig_len,
            );
            if next_op < 0 as ::core::ffi::c_int {
                return -(1 as ::core::ffi::c_int);
            }
            if next_op != BAM_CMATCH && next_op != BAM_CDIFF {
                hts_log(
                    HTS_LOG_INFO,
                    b"cram_add_to_ref_MD\0" as *const u8 as *const ::core::ffi::c_char,
                    b"MD:Z and CIGAR are incompatible\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return -(1 as ::core::ffi::c_int);
            }
            let fresh169 = MD;
            MD = MD.offset(1);
            let fresh170 = iref;
            iref = iref + 1;
            *(*ref_0).offset(fresh170 as isize) = (*fresh169 as ::core::ffi::c_int
                & !(0x20 as ::core::ffi::c_int))
                as ::core::ffi::c_char;
            iseq += 1;
        }
    }
    return 1 as ::core::ffi::c_int;
}
// original: cram_add_to_ref (htslib/cram/cram_encode.c:1659)
unsafe extern "C" fn cram_add_to_ref(
    mut b: *mut bam1_t,
    mut ref_0: *mut *mut ::core::ffi::c_char,
    mut hist: *mut *mut [uint32_t; 5],
    mut ref_start: hts_pos_t,
    mut ref_end: *mut hts_pos_t,
    mut ref_end_alloc: *mut hts_pos_t,
) -> ::core::ffi::c_int {
    let mut MD: *const uint8_t = bam_aux_get(b, b"MD\0" as *const u8 as *const ::core::ffi::c_char);
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !MD.is_null() && *MD as ::core::ffi::c_int == 'Z' as i32 {
        let mut ret_0: ::core::ffi::c_int = cram_add_to_ref_MD(
            b,
            ref_0,
            hist,
            ref_start,
            ref_end,
            ref_end_alloc,
            MD.offset(1 as ::core::ffi::c_int as isize),
        );
        if ret_0 > 0 as ::core::ffi::c_int {
            return ret_0;
        }
    }
    let mut cigar: *mut uint32_t =
        (*b).data
            .offset((*b).core.l_qname as ::core::ffi::c_int as isize) as *mut uint32_t;
    let mut ncigar: uint32_t = (*b).core.n_cigar;
    let mut i: uint32_t = 0;
    let mut j: uint32_t = 0;
    let mut iseq: hts_pos_t = 0 as hts_pos_t;
    let mut iref: hts_pos_t = (*b).core.pos - ref_start;
    let mut seq: *mut uint8_t = (*b)
        .data
        .offset(((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize)
        .offset((*b).core.l_qname as ::core::ffi::c_int as isize);
    i = 0 as uint32_t;
    while i < ncigar {
        match *cigar.offset(i as isize) & BAM_CIGAR_MASK as uint32_t {
            4 | 1 => {
                iseq = (iseq as ::core::ffi::c_long
                    + (*cigar.offset(i as isize) >> BAM_CIGAR_SHIFT) as ::core::ffi::c_long)
                    as hts_pos_t;
            }
            0 | 7 | 8 => {
                let mut len: ::core::ffi::c_int =
                    (*cigar.offset(i as isize) >> BAM_CIGAR_SHIFT) as ::core::ffi::c_int;
                static mut L16: [uint8_t; 16] = [
                    4 as ::core::ffi::c_int as uint8_t,
                    0 as ::core::ffi::c_int as uint8_t,
                    1 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    2 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    3 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                    4 as ::core::ffi::c_int as uint8_t,
                ];
                if extend_ref(
                    ref_0,
                    hist,
                    iref + ref_start + len as hts_pos_t,
                    ref_start,
                    ref_end,
                    ref_end_alloc,
                ) < 0 as ::core::ffi::c_int
                {
                    return -(1 as ::core::ffi::c_int);
                }
                if iseq + len as hts_pos_t <= (*b).core.l_qseq as hts_pos_t {
                    if ret < 0 as ::core::ffi::c_int {
                        memset(
                            (*ref_0).offset(iref as isize) as *mut ::core::ffi::c_char
                                as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            len as size_t,
                        );
                    }
                    j = 0 as uint32_t;
                    while j < len as uint32_t {
                        let ref mut fresh165 = (*(*hist).offset(iref as isize))[L16[(*seq
                            .offset((iseq >> 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            >> ((!iseq & 1 as hts_pos_t) << 2 as ::core::ffi::c_int)
                            & 0xf as ::core::ffi::c_int)
                            as usize]
                            as usize];
                        *fresh165 = (*fresh165).wrapping_add(1);
                        j = j.wrapping_add(1);
                        iref += 1;
                        iseq += 1;
                    }
                } else {
                    iseq = (iseq as ::core::ffi::c_long + len as ::core::ffi::c_long) as hts_pos_t;
                    iref = (iref as ::core::ffi::c_long + len as ::core::ffi::c_long) as hts_pos_t;
                }
            }
            2 | 3 => {
                iref = (iref as ::core::ffi::c_long
                    + (*cigar.offset(i as isize) >> BAM_CIGAR_SHIFT) as ::core::ffi::c_long)
                    as hts_pos_t;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
// original: cram_generate_reference (htslib/cram/cram_encode.c:1733)
unsafe extern "C" fn cram_generate_reference(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r1: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: hts_pos_t = 0;
    let mut current_block: u64;
    let mut ref_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut hist: *mut [uint32_t; 5] = ::core::ptr::null_mut::<[uint32_t; 5]>();
    let mut ref_start: hts_pos_t = (**(*c).bams.offset(r1 as isize)).core.pos;
    let mut ref_end: hts_pos_t = 0 as hts_pos_t;
    let mut ref_end_alloc: hts_pos_t = 0 as hts_pos_t;
    if ref_start < 0 as hts_pos_t {
        return -(1 as ::core::ffi::c_int);
    }
    if extend_ref(
        &raw mut ref_0,
        &raw mut hist,
        (**(*c)
            .bams
            .offset((r1 as int32_t + (*(*s).hdr).num_records - 1 as int32_t) as isize))
        .core
        .pos + (**(*c)
            .bams
            .offset((r1 as int32_t + (*(*s).hdr).num_records - 1 as int32_t) as isize))
        .core
        .l_qseq as hts_pos_t,
        ref_start,
        &raw mut ref_end,
        &raw mut ref_end_alloc,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    let mut r2: ::core::ffi::c_int = 0;
    let mut last_pos: hts_pos_t = -(1 as ::core::ffi::c_int) as hts_pos_t;
    r2 = 0 as ::core::ffi::c_int;
    loop {
        if !(r1 < (*c).curr_c_rec && (r2 as int32_t) < (*(*s).hdr).num_records) {
            current_block = 11650488183268122163;
            break;
        }
        if (**(*c).bams.offset(r1 as isize)).core.pos < last_pos {
            hts_log(
                HTS_LOG_ERROR,
                b"cram_generate_reference\0" as *const u8 as *const ::core::ffi::c_char,
                b"Cannot build reference with unsorted data\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            current_block = 5658119700820736675;
            break;
        } else {
            last_pos = (**(*c).bams.offset(r1 as isize)).core.pos;
            if cram_add_to_ref(
                *(*c).bams.offset(r1 as isize) as *mut bam1_t,
                &raw mut ref_0,
                &raw mut hist,
                ref_start,
                &raw mut ref_end,
                &raw mut ref_end_alloc,
            ) < 0 as ::core::ffi::c_int
            {
                current_block = 5658119700820736675;
                break;
            }
            r1 += 1;
            r2 += 1;
        }
    }
    match current_block {
        5658119700820736675 => {
            free(ref_0 as *mut ::core::ffi::c_void);
            free(hist as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
        _ => {
            i = 0;
            i = 0 as hts_pos_t;
            while i < ref_end - ref_start {
                if *ref_0.offset(i as isize) == 0 {
                    let mut max_v: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut max_j: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
                    let mut j: ::core::ffi::c_int = 0;
                    j = 0 as ::core::ffi::c_int;
                    while j < 4 as ::core::ffi::c_int {
                        if (max_v as uint32_t) < (*hist.offset(i as isize))[j as usize] {
                            max_v = (*hist.offset(i as isize))[j as usize] as ::core::ffi::c_int;
                            max_j = j;
                        }
                        j += 1;
                    }
                    *ref_0.offset(i as isize) =
                        ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"ACGTN\0")
                            [max_j as usize];
                }
                i += 1;
            }
            free(hist as *mut ::core::ffi::c_void);
            (*c).ref_0 = ref_0;
            (*c).ref_start = ref_start + 1 as hts_pos_t;
            (*c).ref_end = ref_end + 1 as hts_pos_t;
            (*c).ref_free = 1 as ::core::ffi::c_int;
            return 0 as ::core::ffi::c_int;
        }
    };
}
// original: validate_md5 (htslib/cram/cram_encode.c:1794)
unsafe extern "C" fn validate_md5(
    mut fd: *mut cram_fd,
    mut ref_id: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*fd).ignore_md5 != 0 || ref_id < 0 as ::core::ffi::c_int || ref_id >= (*(*fd).refs).nref {
        return 0 as ::core::ffi::c_int;
    }
    if (**(*(*fd).refs).ref_id.offset(ref_id as isize)).validated_md5 != 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut hrecs: *mut sam_hrecs_t = (*(*fd).header).hrecs;
    let mut ty: *mut sam_hrec_type_t = sam_hrecs_find_type_id(
        hrecs,
        b"SQ\0" as *const u8 as *const ::core::ffi::c_char,
        b"SN\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*hrecs).ref_0.offset(ref_id as isize)).name,
    );
    if ty.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut m5tag: *mut sam_hrec_tag_t = sam_hrecs_find_key(
        ty,
        b"M5\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<*mut sam_hrec_tag_t>(),
    );
    if m5tag.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut ref_0: *mut ::core::ffi::c_char = (**(*(*fd).refs).ref_id.offset(ref_id as isize)).seq;
    let mut len: int64_t = (**(*(*fd).refs).ref_id.offset(ref_id as isize)).length;
    let mut md5: *mut hts_md5_context = ::core::ptr::null_mut::<hts_md5_context>();
    let mut buf: [::core::ffi::c_uchar; 16] = [0; 16];
    let mut buf2: [::core::ffi::c_char; 33] = [0; 33];
    md5 = hts_md5_init();
    if md5.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    hts_md5_update(
        md5,
        ref_0 as *const ::core::ffi::c_void,
        len as ::core::ffi::c_ulong,
    );
    hts_md5_final(&raw mut buf as *mut ::core::ffi::c_uchar, md5);
    hts_md5_destroy(md5);
    hts_md5_hex(
        &raw mut buf2 as *mut ::core::ffi::c_char,
        &raw mut buf as *mut ::core::ffi::c_uchar,
    );
    if strcmp(
        (*m5tag).str_0.offset(3 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    ) != 0
    {
        hts_log(
            HTS_LOG_ERROR,
            b"validate_md5\0" as *const u8 as *const ::core::ffi::c_char,
            b"SQ header M5 tag discrepancy for reference '%s'\0" as *const u8
                as *const ::core::ffi::c_char,
            (*(*hrecs).ref_0.offset(ref_id as isize)).name,
        );
        hts_log(
            HTS_LOG_ERROR,
            b"validate_md5\0" as *const u8 as *const ::core::ffi::c_char,
            b"Please use the correct reference, or consider using embed_ref=2\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return -(1 as ::core::ffi::c_int);
    }
    (**(*(*fd).refs).ref_id.offset(ref_id as isize)).validated_md5 = 1 as ::core::ffi::c_int;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
// original: cram_encode_container (htslib/cram/cram_encode.c:1846)
pub unsafe extern "C" fn cram_encode_container(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
) -> ::core::ffi::c_int {
    let mut failed_embed: ::core::ffi::c_int = 0;
    let mut is_v4: ::core::ffi::c_int = 0;
    let mut current_block: u64;
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut slice_offset: ::core::ffi::c_int = 0;
    let mut h: *mut cram_block_compression_hdr = (*c).comp_hdr;
    let mut c_hdr: *mut cram_block = ::core::ptr::null_mut::<cram_block>();
    let mut multi_ref: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r1: ::core::ffi::c_int = 0;
    let mut r2: ::core::ffi::c_int = 0;
    let mut sn: ::core::ffi::c_int = 0;
    let mut nref: ::core::ffi::c_int = 0;
    let mut embed_ref: ::core::ffi::c_int = 0;
    let mut no_ref: ::core::ffi::c_int = 0;
    let mut spares: *mut spare_bams = ::core::ptr::null_mut::<spare_bams>();
    if !(*c).bams.is_null() {
        if !((*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int) {
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            failed_embed = ((*fd).no_ref_counter >= 5 as ::core::ffi::c_int) as ::core::ffi::c_int;
            if failed_embed == 0
                && (*c).embed_ref == -(2 as ::core::ffi::c_int)
                && (*c).ref_id >= 0 as ::core::ffi::c_int
            {
                hts_log(
                    HTS_LOG_WARNING,
                    b"cram_encode_container\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Retrying embed_ref=2 mode for #%d/5\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*fd).no_ref_counter,
                );
                (*c).no_ref = 0 as ::core::ffi::c_int;
                (*fd).no_ref = (*c).no_ref;
                (*c).embed_ref = 2 as ::core::ffi::c_int;
                (*fd).embed_ref = (*c).embed_ref;
            } else if failed_embed != 0 && (*c).embed_ref == -(2 as ::core::ffi::c_int) {
                hts_log(
                    HTS_LOG_WARNING,
                    b"cram_encode_container\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Keeping non-ref mode from now on\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                (*c).embed_ref = 0 as ::core::ffi::c_int;
                (*fd).embed_ref = (*c).embed_ref;
            }
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            '_restart: loop {
                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                nref = (*(*fd).refs).nref;
                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                embed_ref = (*c).embed_ref;
                no_ref = (*c).no_ref;
                if no_ref == 0 {
                    if (*c).bams.is_null()
                        || (*c).curr_c_rec == 0
                        || (*(*c).bams.offset(0 as ::core::ffi::c_int as isize)).is_null()
                    {
                        current_block = 17869886767212223845;
                        break;
                    }
                    let mut b: *mut bam_seq_t = *(*c).bams.offset(0 as ::core::ffi::c_int as isize);
                    if embed_ref <= 1 as ::core::ffi::c_int {
                        let mut ref_0: *mut ::core::ffi::c_char = cram_get_ref(
                            fd,
                            (*b).core.tid as ::core::ffi::c_int,
                            1 as hts_pos_t,
                            0 as hts_pos_t,
                        );
                        if ref_0.is_null() && (*b).core.tid >= 0 as int32_t {
                            if (*c).pos_sorted == 0 {
                                hts_log(
                                    HTS_LOG_WARNING,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Failed to load reference #%d\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    (*b).core.tid,
                                );
                                hts_log(
                                    HTS_LOG_WARNING,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Switching to non-ref mode\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                (*fd).embed_ref = 0 as ::core::ffi::c_int;
                                (*c).embed_ref = (*fd).embed_ref;
                                (*fd).no_ref = 1 as ::core::ffi::c_int;
                                (*c).no_ref = (*fd).no_ref;
                                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                                continue;
                            } else {
                                if (*c).multi_seq != 0 || embed_ref == 0 as ::core::ffi::c_int {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_encode_container\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Failed to load reference #%d\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        (*b).core.tid,
                                    );
                                    return -(1 as ::core::ffi::c_int);
                                }
                                hts_log(
                                    HTS_LOG_WARNING,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Failed to load reference #%d\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    (*b).core.tid,
                                );
                                hts_log(
                                    HTS_LOG_WARNING,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Enabling embed_ref=2 mode to auto-generate reference\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                if embed_ref <= 0 as ::core::ffi::c_int {
                                    hts_log(
                                        HTS_LOG_WARNING,
                                        b"cram_encode_container\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"NOTE: the CRAM file will be bigger than using an external reference\0"
                                            as *const u8 as *const ::core::ffi::c_char,
                                    );
                                }
                                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                (*fd).embed_ref = 2 as ::core::ffi::c_int;
                                (*c).embed_ref = (*fd).embed_ref;
                                embed_ref = (*c).embed_ref;
                                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                            }
                            current_block = 10053640148532328547;
                        } else {
                            if !ref_0.is_null() {
                                if validate_md5(fd, (*c).ref_seq_id as ::core::ffi::c_int)
                                    < 0 as ::core::ffi::c_int
                                {
                                    current_block = 17869886767212223845;
                                    break;
                                }
                            }
                            (*c).ref_id = (*b).core.tid as ::core::ffi::c_int;
                            if (*c).ref_id >= 0 as ::core::ffi::c_int {
                                (*c).ref_seq_id = (*c).ref_id as int32_t;
                                (*c).ref_0 =
                                    (**(*(*fd).refs).ref_id.offset((*c).ref_seq_id as isize)).seq;
                                (*c).ref_start = 1 as hts_pos_t;
                                (*c).ref_end = (**(*(*fd).refs)
                                    .ref_id
                                    .offset((*c).ref_seq_id as isize))
                                .length as hts_pos_t;
                            }
                            current_block = 3160140712158701372;
                        }
                    } else {
                        current_block = 10053640148532328547;
                    }
                    match current_block {
                        10053640148532328547 => {
                            (*c).ref_id = (*b).core.tid as ::core::ffi::c_int;
                            if (*c).ref_id >= 0 as ::core::ffi::c_int {
                                (*c).ref_0 = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                (*c).ref_free = 1 as ::core::ffi::c_int;
                            } else {
                                embed_ref = 0 as ::core::ffi::c_int;
                                (*c).no_ref = 1 as ::core::ffi::c_int;
                                no_ref = (*c).no_ref;
                            }
                        }
                        _ => {}
                    }
                    (*c).ref_seq_id = (*c).ref_id as int32_t;
                } else {
                    (*c).ref_id = (**(*c).bams.offset(0 as ::core::ffi::c_int as isize))
                        .core
                        .tid as ::core::ffi::c_int;
                    cram_ref_incr((*fd).refs, (*c).ref_id);
                    (*c).ref_seq_id = (*c).ref_id as int32_t;
                }
                if no_ref == 0 && !(*c).refs_used.is_null() {
                    i = 0 as ::core::ffi::c_int;
                    while i < nref {
                        if *(*c).refs_used.offset(i as isize) != 0 {
                            if !cram_get_ref(fd, i, 1 as hts_pos_t, 0 as hts_pos_t).is_null() {
                                if validate_md5(fd, i) < 0 as ::core::ffi::c_int {
                                    current_block = 17869886767212223845;
                                    break '_restart;
                                }
                            } else {
                                hts_log(
                                    HTS_LOG_WARNING,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Failed to find reference, switching to non-ref mode\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                (*c).no_ref = 1 as ::core::ffi::c_int;
                                no_ref = (*c).no_ref;
                            }
                        }
                        i += 1;
                    }
                }
                sn = 0 as ::core::ffi::c_int;
                r1 = sn;
                while r1 < (*c).curr_c_rec {
                    let mut s: *mut cram_slice =
                        *(*c).slices.offset(sn as isize) as *mut cram_slice;
                    let mut first_base: int64_t = INT64_MAX as int64_t;
                    let mut last_base: int64_t = INT64_MIN as int64_t;
                    let mut r1_start: ::core::ffi::c_int = r1;
                    '_c2rust_label: {
                        if sn < (*c).curr_slice {
                        } else {
                            __assert_fail(
                                b"sn < c->curr_slice\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"/data/henriksson/github/claude/cellsnp-lite/htslib-rs/htslib/cram/cram_encode.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                1982 as ::core::ffi::c_uint,
                                b"int cram_encode_container(cram_fd *, cram_container *)\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if lossy_read_names(fd, c, s, r1_start) != 0 as ::core::ffi::c_int {
                        return -(1 as ::core::ffi::c_int);
                    }
                    let mut MD: kstring_t = kstring_t {
                        l: 0 as size_t,
                        m: 0,
                        s: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    };
                    if embed_ref == 2 as ::core::ffi::c_int {
                        if (*c).ref_id < 0 as ::core::ffi::c_int
                            || cram_generate_reference(c, s, r1) < 0 as ::core::ffi::c_int
                        {
                            if sn > 0 as ::core::ffi::c_int {
                                hts_log(
                                    HTS_LOG_ERROR,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Failed to build reference, switching to non-ref mode\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                return -(1 as ::core::ffi::c_int);
                            } else {
                                hts_log(
                                    HTS_LOG_WARNING,
                                    b"cram_encode_container\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"Failed to build reference, switching to non-ref mode\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                            }
                            pthread_mutex_lock(&raw mut (*fd).ref_lock);
                            (*fd).embed_ref = -(2 as ::core::ffi::c_int);
                            (*c).embed_ref = (*fd).embed_ref;
                            (*fd).no_ref = 1 as ::core::ffi::c_int;
                            (*c).no_ref = (*fd).no_ref;
                            (*fd).no_ref_counter += 1;
                            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                            failed_embed = 1 as ::core::ffi::c_int;
                            continue '_restart;
                        } else {
                            pthread_mutex_lock(&raw mut (*fd).ref_lock);
                            (*fd).no_ref_counter -= ((*fd).no_ref_counter > 0 as ::core::ffi::c_int)
                                as ::core::ffi::c_int;
                            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                            let mut rlen: hts_pos_t =
                                if (**(*(*fd).refs).ref_id.offset((*c).ref_id as isize)).LN_length
                                    > (**(*(*fd).refs).ref_id.offset((*c).ref_id as isize)).length
                                {
                                    (**(*(*fd).refs).ref_id.offset((*c).ref_id as isize)).LN_length
                                        as hts_pos_t
                                } else {
                                    (**(*(*fd).refs).ref_id.offset((*c).ref_id as isize)).length
                                        as hts_pos_t
                                };
                            if (*c).ref_end > rlen && rlen != 0 {
                                (*c).ref_end = rlen;
                            }
                        }
                    }
                    r2 = 0 as ::core::ffi::c_int;
                    while r1 < (*c).curr_c_rec && (r2 as int32_t) < (*(*s).hdr).num_records {
                        let mut cr: *mut cram_record =
                            (*s).crecs.offset(r2 as isize) as *mut cram_record;
                        let mut b_0: *mut bam_seq_t = *(*c).bams.offset(r1 as isize);
                        if (*c).multi_seq != 0 && no_ref == 0 {
                            if (*b_0).core.tid != (*c).ref_seq_id && (*b_0).core.tid >= 0 as int32_t
                            {
                                if (*c).ref_seq_id >= 0 as int32_t {
                                    cram_ref_decr(
                                        (*fd).refs,
                                        (*c).ref_seq_id as ::core::ffi::c_int,
                                    );
                                }
                                if cram_get_ref(
                                    fd,
                                    (*b_0).core.tid as ::core::ffi::c_int,
                                    1 as hts_pos_t,
                                    0 as hts_pos_t,
                                )
                                .is_null()
                                {
                                    hts_log(
                                        HTS_LOG_ERROR,
                                        b"cram_encode_container\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        b"Failed to load reference #%d\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        (*b_0).core.tid,
                                    );
                                    free(MD.s as *mut ::core::ffi::c_void);
                                    return -(1 as ::core::ffi::c_int);
                                }
                                if validate_md5(fd, (*b_0).core.tid as ::core::ffi::c_int)
                                    < 0 as ::core::ffi::c_int
                                {
                                    return -(1 as ::core::ffi::c_int);
                                }
                                (*c).ref_seq_id = (*b_0).core.tid;
                                if (**(*(*fd).refs).ref_id.offset((*c).ref_seq_id as isize))
                                    .seq
                                    .is_null()
                                {
                                    return -(1 as ::core::ffi::c_int);
                                }
                                (*c).ref_0 =
                                    (**(*(*fd).refs).ref_id.offset((*c).ref_seq_id as isize)).seq;
                                (*c).ref_start = 1 as hts_pos_t;
                                (*c).ref_end = (**(*(*fd).refs)
                                    .ref_id
                                    .offset((*c).ref_seq_id as isize))
                                .length as hts_pos_t;
                            }
                        }
                        if process_one_read(fd, c, s, cr, b_0, r2, &raw mut MD, embed_ref, no_ref)
                            != 0 as ::core::ffi::c_int
                        {
                            free(MD.s as *mut ::core::ffi::c_void);
                            return -(1 as ::core::ffi::c_int);
                        }
                        if first_base > (*cr).apos {
                            first_base = (*cr).apos;
                        }
                        if last_base < (*cr).aend {
                            last_base = (*cr).aend;
                        }
                        r1 += 1;
                        r2 += 1;
                    }
                    free(MD.s as *mut ::core::ffi::c_void);
                    if add_read_names(fd, c, s, r1_start) < 0 as ::core::ffi::c_int {
                        return -(1 as ::core::ffi::c_int);
                    }
                    if (*c).multi_seq != 0 {
                        (*(*s).hdr).ref_seq_id = -(2 as ::core::ffi::c_int) as int32_t;
                        (*(*s).hdr).ref_seq_start = 0 as int64_t;
                        (*(*s).hdr).ref_seq_span = 0 as int64_t;
                    } else if (*c).ref_id == -(1 as ::core::ffi::c_int)
                        && (*fd).version >= 0x301 as ::core::ffi::c_int
                    {
                        (*(*s).hdr).ref_seq_id = -(1 as ::core::ffi::c_int) as int32_t;
                        (*(*s).hdr).ref_seq_start = 0 as int64_t;
                        (*(*s).hdr).ref_seq_span = 0 as int64_t;
                    } else {
                        (*(*s).hdr).ref_seq_id = (*c).ref_id as int32_t;
                        (*(*s).hdr).ref_seq_start = first_base;
                        (*(*s).hdr).ref_seq_span =
                            if 0 as int64_t > last_base - first_base + 1 as int64_t {
                                0 as int64_t
                            } else {
                                last_base - first_base + 1 as int64_t
                            };
                    }
                    (*(*s).hdr).num_records = r2 as int32_t;
                    if (*(*c).tags_used).n_occupied != 0 {
                        let mut ntags: ::core::ffi::c_int =
                            (*(*c).tags_used).n_occupied as ::core::ffi::c_int;
                        (*s).aux_block = calloc(
                            (ntags * 2 as ::core::ffi::c_int) as size_t,
                            ::core::mem::size_of::<*mut cram_block>() as size_t,
                        ) as *mut *mut cram_block;
                        if (*s).aux_block.is_null() {
                            return -(1 as ::core::ffi::c_int);
                        }
                        let mut k: khint_t = 0;
                        (*s).naux_block = 0 as ::core::ffi::c_int;
                        k = 0 as ::core::ffi::c_int as khint_t;
                        while k != (*(*c).tags_used).n_buckets {
                            if !(*(*(*c).tags_used)
                                .flags
                                .offset((k >> 4 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_uint
                                >> ((k as ::core::ffi::c_uint & 0xf as ::core::ffi::c_uint)
                                    << 1 as ::core::ffi::c_int)
                                & 3 as ::core::ffi::c_uint
                                != 0)
                            {
                                let mut tm: *mut cram_tag_map =
                                    *(*(*c).tags_used).vals.offset(k as isize);
                                if tm.is_null() {
                                    current_block = 17869886767212223845;
                                    break '_restart;
                                }
                                if !(*tm).blk.is_null() {
                                    let fresh130 = (*s).naux_block;
                                    (*s).naux_block = (*s).naux_block + 1;
                                    let ref mut fresh131 =
                                        *(*s).aux_block.offset(fresh130 as isize);
                                    *fresh131 = (*tm).blk;
                                    (*tm).blk = ::core::ptr::null_mut::<cram_block>();
                                    if !(*tm).blk2.is_null() {
                                        let fresh132 = (*s).naux_block;
                                        (*s).naux_block = (*s).naux_block + 1;
                                        let ref mut fresh133 =
                                            *(*s).aux_block.offset(fresh132 as isize);
                                        *fresh133 = (*tm).blk2;
                                        (*tm).blk2 = ::core::ptr::null_mut::<cram_block>();
                                    }
                                }
                            }
                            k = k.wrapping_add(1);
                        }
                        '_c2rust_label_0: {
                            if (*s).naux_block as ::core::ffi::c_uint
                                <= (2 as khint_t).wrapping_mul((*(*c).tags_used).n_occupied)
                            {
                            } else {
                                __assert_fail(
                                    b"s->naux_block <= 2*c->tags_used->n_occupied\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"/data/henriksson/github/claude/cellsnp-lite/htslib-rs/htslib/cram/cram_encode.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    2126 as ::core::ffi::c_uint,
                                    b"int cram_encode_container(cram_fd *, cram_container *)\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                );
                            }
                        };
                    }
                    sn += 1;
                }
                if (*c).multi_seq != 0 && no_ref == 0 {
                    if (*c).ref_seq_id >= 0 as int32_t {
                        cram_ref_decr((*fd).refs, (*c).ref_seq_id as ::core::ffi::c_int);
                    }
                }
                spares = malloc(::core::mem::size_of::<spare_bams>() as size_t) as *mut spare_bams;
                if spares.is_null() {
                    current_block = 17869886767212223845;
                    break;
                } else {
                    current_block = 9587810615301548814;
                    break;
                }
            }
            match current_block {
                17869886767212223845 => {}
                _ => {
                    pthread_mutex_lock(&raw mut (*fd).bam_list_lock);
                    (*spares).bams = (*c).bams;
                    (*spares).next = (*fd).bl as *mut spare_bams;
                    (*fd).bl = spares;
                    pthread_mutex_unlock(&raw mut (*fd).bam_list_lock);
                    (*c).bams = ::core::ptr::null_mut::<*mut bam_seq_t>();
                    cram_stats_encoding(fd, (*c).stats[DS_RI as ::core::ffi::c_int as usize]);
                    multi_ref = ((*(*c).stats[DS_RI as ::core::ffi::c_int as usize]).nvals
                        > 1 as ::core::ffi::c_int)
                        as ::core::ffi::c_int;
                    pthread_mutex_lock(&raw mut (*fd).metrics_lock);
                    (*fd).last_RI_count = (*(*c).stats[DS_RI as ::core::ffi::c_int as usize]).nvals;
                    pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
                    if multi_ref != 0 {
                        hts_log(
                            HTS_LOG_INFO,
                            b"cram_encode_container\0" as *const u8 as *const ::core::ffi::c_char,
                            b"Multi-ref container\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        (*c).ref_seq_id = -(2 as ::core::ffi::c_int) as int32_t;
                        (*c).ref_seq_start = 0 as int64_t;
                        (*c).ref_seq_span = 0 as int64_t;
                    }
                    no_ref = (*c).no_ref;
                    is_v4 = if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    i = 0 as ::core::ffi::c_int;
                    while i < (*c).curr_slice {
                        let mut s_0: *mut cram_slice =
                            *(*c).slices.offset(i as isize) as *mut cram_slice;
                        if (*fd).version >> 8 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                            if (*(*s_0).hdr).ref_seq_id >= 0 as int32_t
                                && (*c).multi_seq == 0 as ::core::ffi::c_int
                                && no_ref == 0
                            {
                                let mut md5: *mut hts_md5_context = hts_md5_init();
                                if md5.is_null() {
                                    return -(1 as ::core::ffi::c_int);
                                }
                                hts_md5_update(
                                    md5,
                                    (*c).ref_0
                                        .offset((*(*s_0).hdr).ref_seq_start as isize)
                                        .offset(-((*c).ref_start as isize))
                                        as *const ::core::ffi::c_void,
                                    (*(*s_0).hdr).ref_seq_span as ::core::ffi::c_ulong,
                                );
                                hts_md5_final(
                                    &raw mut (*(*s_0).hdr).md5 as *mut ::core::ffi::c_uchar,
                                    md5,
                                );
                                hts_md5_destroy(md5);
                            } else {
                                memset(
                                    &raw mut (*(*s_0).hdr).md5 as *mut ::core::ffi::c_uchar
                                        as *mut ::core::ffi::c_void,
                                    0 as ::core::ffi::c_int,
                                    16 as size_t,
                                );
                            }
                        }
                        i += 1;
                    }
                    (*c).num_records = 0 as ::core::ffi::c_int as int32_t;
                    (*c).num_blocks = 1 as ::core::ffi::c_int as int32_t;
                    (*c).length = 0 as ::core::ffi::c_int as int32_t;
                    (*h).codecs[DS_BF as ::core::ffi::c_int as usize] = cram_encoder_init(
                        cram_stats_encoding(fd, (*c).stats[DS_BF as ::core::ffi::c_int as usize]),
                        (*c).stats[DS_BF as ::core::ffi::c_int as usize],
                        E_INT,
                        NULL_0,
                        (*fd).version,
                        &raw mut (*fd).vv,
                    )
                        as *mut cram_codec;
                    if !((*(*c).stats[DS_BF as ::core::ffi::c_int as usize]).nvals != 0
                        && (*h).codecs[DS_BF as ::core::ffi::c_int as usize].is_null())
                    {
                        (*h).codecs[DS_CF as ::core::ffi::c_int as usize] = cram_encoder_init(
                            cram_stats_encoding(
                                fd,
                                (*c).stats[DS_CF as ::core::ffi::c_int as usize],
                            ),
                            (*c).stats[DS_CF as ::core::ffi::c_int as usize],
                            E_INT,
                            NULL_0,
                            (*fd).version,
                            &raw mut (*fd).vv,
                        )
                            as *mut cram_codec;
                        if !((*(*c).stats[DS_CF as ::core::ffi::c_int as usize]).nvals != 0
                            && (*h).codecs[DS_CF as ::core::ffi::c_int as usize].is_null())
                        {
                            if (*c).pos_sorted != 0
                                || (*fd).version >> 8 as ::core::ffi::c_int
                                    >= 4 as ::core::ffi::c_int
                            {
                                if (*c).pos_sorted != 0 {
                                    (*h).codecs[DS_AP as ::core::ffi::c_int as usize] =
                                        cram_encoder_init(
                                            cram_stats_encoding(
                                                fd,
                                                (*c).stats[DS_AP as ::core::ffi::c_int as usize],
                                            ),
                                            (*c).stats[DS_AP as ::core::ffi::c_int as usize],
                                            (if is_v4 != 0 {
                                                E_LONG as ::core::ffi::c_int
                                            } else {
                                                E_INT as ::core::ffi::c_int
                                            })
                                                as cram_external_type,
                                            NULL_0,
                                            (*fd).version,
                                            &raw mut (*fd).vv,
                                        )
                                            as *mut cram_codec;
                                } else {
                                    (*h).codecs[DS_AP as ::core::ffi::c_int as usize] =
                                        cram_encoder_init(
                                            (if is_v4 != 0 {
                                                E_VARINT_SIGNED as ::core::ffi::c_int
                                            } else {
                                                E_EXTERNAL as ::core::ffi::c_int
                                            })
                                                as cram_encoding,
                                            ::core::ptr::null_mut::<cram_stats>(),
                                            (if is_v4 != 0 {
                                                E_LONG as ::core::ffi::c_int
                                            } else {
                                                E_INT as ::core::ffi::c_int
                                            })
                                                as cram_external_type,
                                            NULL_0,
                                            (*fd).version,
                                            &raw mut (*fd).vv,
                                        )
                                            as *mut cram_codec;
                                }
                            } else {
                                let mut p: [hts_pos_t; 2] =
                                    [0 as ::core::ffi::c_int as hts_pos_t, (*c).max_apos];
                                (*h).codecs[DS_AP as ::core::ffi::c_int as usize] =
                                    cram_encoder_init(
                                        E_BETA,
                                        ::core::ptr::null_mut::<cram_stats>(),
                                        (if is_v4 != 0 {
                                            E_LONG as ::core::ffi::c_int
                                        } else {
                                            E_INT as ::core::ffi::c_int
                                        })
                                            as cram_external_type,
                                        &raw mut p as *mut hts_pos_t as *mut ::core::ffi::c_void,
                                        (*fd).version,
                                        &raw mut (*fd).vv,
                                    ) as *mut cram_codec;
                            }
                            if !(*h).codecs[DS_AP as ::core::ffi::c_int as usize].is_null() {
                                (*h).codecs[DS_RG as ::core::ffi::c_int as usize] =
                                    cram_encoder_init(
                                        cram_stats_encoding(
                                            fd,
                                            (*c).stats[DS_RG as ::core::ffi::c_int as usize],
                                        ),
                                        (*c).stats[DS_RG as ::core::ffi::c_int as usize],
                                        E_INT,
                                        NULL_0,
                                        (*fd).version,
                                        &raw mut (*fd).vv,
                                    ) as *mut cram_codec;
                                if !((*(*c).stats[DS_RG as ::core::ffi::c_int as usize]).nvals != 0
                                    && (*h).codecs[DS_RG as ::core::ffi::c_int as usize].is_null())
                                {
                                    (*h).codecs[DS_MQ as ::core::ffi::c_int as usize] =
                                        cram_encoder_init(
                                            cram_stats_encoding(
                                                fd,
                                                (*c).stats[DS_MQ as ::core::ffi::c_int as usize],
                                            ),
                                            (*c).stats[DS_MQ as ::core::ffi::c_int as usize],
                                            E_INT,
                                            NULL_0,
                                            (*fd).version,
                                            &raw mut (*fd).vv,
                                        )
                                            as *mut cram_codec;
                                    if !((*(*c).stats[DS_MQ as ::core::ffi::c_int as usize]).nvals
                                        != 0
                                        && (*h).codecs[DS_MQ as ::core::ffi::c_int as usize]
                                            .is_null())
                                    {
                                        (*h).codecs[DS_NS as ::core::ffi::c_int as usize] =
                                            cram_encoder_init(
                                                cram_stats_encoding(
                                                    fd,
                                                    (*c).stats
                                                        [DS_NS as ::core::ffi::c_int as usize],
                                                ),
                                                (*c).stats[DS_NS as ::core::ffi::c_int as usize],
                                                E_INT,
                                                NULL_0,
                                                (*fd).version,
                                                &raw mut (*fd).vv,
                                            )
                                                as *mut cram_codec;
                                        if !((*(*c).stats[DS_NS as ::core::ffi::c_int as usize])
                                            .nvals
                                            != 0
                                            && (*h).codecs[DS_NS as ::core::ffi::c_int as usize]
                                                .is_null())
                                        {
                                            (*h).codecs[DS_MF as ::core::ffi::c_int as usize] =
                                                cram_encoder_init(
                                                    cram_stats_encoding(
                                                        fd,
                                                        (*c).stats
                                                            [DS_MF as ::core::ffi::c_int as usize],
                                                    ),
                                                    (*c).stats
                                                        [DS_MF as ::core::ffi::c_int as usize],
                                                    E_INT,
                                                    NULL_0,
                                                    (*fd).version,
                                                    &raw mut (*fd).vv,
                                                )
                                                    as *mut cram_codec;
                                            if !((*(*c).stats
                                                [DS_MF as ::core::ffi::c_int as usize])
                                                .nvals
                                                != 0
                                                && (*h).codecs
                                                    [DS_MF as ::core::ffi::c_int as usize]
                                                    .is_null())
                                            {
                                                (*h).codecs[DS_TS as ::core::ffi::c_int as usize] =
                                                    cram_encoder_init(
                                                        cram_stats_encoding(
                                                            fd,
                                                            (*c).stats[DS_TS as ::core::ffi::c_int
                                                                as usize],
                                                        ),
                                                        (*c).stats
                                                            [DS_TS as ::core::ffi::c_int as usize],
                                                        (if is_v4 != 0 {
                                                            E_LONG as ::core::ffi::c_int
                                                        } else {
                                                            E_INT as ::core::ffi::c_int
                                                        })
                                                            as cram_external_type,
                                                        NULL_0,
                                                        (*fd).version,
                                                        &raw mut (*fd).vv,
                                                    )
                                                        as *mut cram_codec;
                                                if !((*(*c).stats
                                                    [DS_TS as ::core::ffi::c_int as usize])
                                                    .nvals
                                                    != 0
                                                    && (*h).codecs
                                                        [DS_TS as ::core::ffi::c_int as usize]
                                                        .is_null())
                                                {
                                                    (*h).codecs
                                                        [DS_NP as ::core::ffi::c_int as usize] =
                                                        cram_encoder_init(
                                                            cram_stats_encoding(
                                                                fd,
                                                                (*c).stats[DS_NP
                                                                    as ::core::ffi::c_int
                                                                    as usize],
                                                            ),
                                                            (*c).stats[DS_NP as ::core::ffi::c_int
                                                                as usize],
                                                            (if is_v4 != 0 {
                                                                E_LONG as ::core::ffi::c_int
                                                            } else {
                                                                E_INT as ::core::ffi::c_int
                                                            })
                                                                as cram_external_type,
                                                            NULL_0,
                                                            (*fd).version,
                                                            &raw mut (*fd).vv,
                                                        )
                                                            as *mut cram_codec;
                                                    if !((*(*c).stats
                                                        [DS_NP as ::core::ffi::c_int as usize])
                                                        .nvals
                                                        != 0
                                                        && (*h).codecs
                                                            [DS_NP as ::core::ffi::c_int as usize]
                                                            .is_null())
                                                    {
                                                        (*h).codecs[DS_NF as ::core::ffi::c_int
                                                            as usize] = cram_encoder_init(
                                                            cram_stats_encoding(
                                                                fd,
                                                                (*c).stats[DS_NF
                                                                    as ::core::ffi::c_int
                                                                    as usize],
                                                            ),
                                                            (*c).stats[DS_NF as ::core::ffi::c_int
                                                                as usize],
                                                            E_INT,
                                                            NULL_0,
                                                            (*fd).version,
                                                            &raw mut (*fd).vv,
                                                        )
                                                            as *mut cram_codec;
                                                        if !((*(*c).stats
                                                            [DS_NF as ::core::ffi::c_int as usize])
                                                            .nvals
                                                            != 0
                                                            && (*h).codecs[DS_NF
                                                                as ::core::ffi::c_int
                                                                as usize]
                                                                .is_null())
                                                        {
                                                            (*h).codecs[DS_RL as ::core::ffi::c_int
                                                                as usize] = cram_encoder_init(
                                                                cram_stats_encoding(
                                                                    fd,
                                                                    (*c).stats[DS_RL
                                                                        as ::core::ffi::c_int
                                                                        as usize],
                                                                ),
                                                                (*c).stats[DS_RL
                                                                    as ::core::ffi::c_int
                                                                    as usize],
                                                                E_INT,
                                                                NULL_0,
                                                                (*fd).version,
                                                                &raw mut (*fd).vv,
                                                            )
                                                                as *mut cram_codec;
                                                            if !((*(*c).stats[DS_RL
                                                                as ::core::ffi::c_int
                                                                as usize])
                                                                .nvals
                                                                != 0
                                                                && (*h).codecs[DS_RL
                                                                    as ::core::ffi::c_int
                                                                    as usize]
                                                                    .is_null())
                                                            {
                                                                (*h).codecs[DS_FN
                                                                    as ::core::ffi::c_int
                                                                    as usize] = cram_encoder_init(
                                                                    cram_stats_encoding(
                                                                        fd,
                                                                        (*c).stats[DS_FN
                                                                            as ::core::ffi::c_int
                                                                            as usize],
                                                                    ),
                                                                    (*c).stats[DS_FN
                                                                        as ::core::ffi::c_int
                                                                        as usize],
                                                                    E_INT,
                                                                    NULL_0,
                                                                    (*fd).version,
                                                                    &raw mut (*fd).vv,
                                                                )
                                                                    as *mut cram_codec;
                                                                if !((*(*c).stats[DS_FN
                                                                    as ::core::ffi::c_int
                                                                    as usize])
                                                                    .nvals
                                                                    != 0
                                                                    && (*h).codecs[DS_FN
                                                                        as ::core::ffi::c_int
                                                                        as usize]
                                                                        .is_null())
                                                                {
                                                                    (*h).codecs[DS_FC as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                        cram_stats_encoding(
                                                                            fd,
                                                                            (*c).stats[DS_FC as ::core::ffi::c_int as usize],
                                                                        ),
                                                                        (*c).stats[DS_FC as ::core::ffi::c_int as usize],
                                                                        E_BYTE,
                                                                        NULL_0,
                                                                        (*fd).version,
                                                                        &raw mut (*fd).vv,
                                                                    ) as *mut cram_codec;
                                                                    if !((*(*c).stats[DS_FC
                                                                        as ::core::ffi::c_int
                                                                        as usize])
                                                                        .nvals
                                                                        != 0
                                                                        && (*h).codecs[DS_FC
                                                                            as ::core::ffi::c_int
                                                                            as usize]
                                                                            .is_null())
                                                                    {
                                                                        (*h).codecs[DS_FP as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                            cram_stats_encoding(
                                                                                fd,
                                                                                (*c).stats[DS_FP as ::core::ffi::c_int as usize],
                                                                            ),
                                                                            (*c).stats[DS_FP as ::core::ffi::c_int as usize],
                                                                            E_INT,
                                                                            NULL_0,
                                                                            (*fd).version,
                                                                            &raw mut (*fd).vv,
                                                                        ) as *mut cram_codec;
                                                                        if !((*(*c).stats[DS_FP as ::core::ffi::c_int as usize])
                                                                            .nvals != 0
                                                                            && (*h)
                                                                                .codecs[DS_FP as ::core::ffi::c_int as usize]
                                                                                .is_null())
                                                                        {
                                                                            (*h).codecs[DS_DL as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                cram_stats_encoding(
                                                                                    fd,
                                                                                    (*c).stats[DS_DL as ::core::ffi::c_int as usize],
                                                                                ),
                                                                                (*c).stats[DS_DL as ::core::ffi::c_int as usize],
                                                                                E_INT,
                                                                                NULL_0,
                                                                                (*fd).version,
                                                                                &raw mut (*fd).vv,
                                                                            ) as *mut cram_codec;
                                                                            if !((*(*c).stats[DS_DL as ::core::ffi::c_int as usize])
                                                                                .nvals != 0
                                                                                && (*h)
                                                                                    .codecs[DS_DL as ::core::ffi::c_int as usize]
                                                                                    .is_null())
                                                                            {
                                                                                (*h).codecs[DS_BA as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                    cram_stats_encoding(
                                                                                        fd,
                                                                                        (*c).stats[DS_BA as ::core::ffi::c_int as usize],
                                                                                    ),
                                                                                    (*c).stats[DS_BA as ::core::ffi::c_int as usize],
                                                                                    E_BYTE,
                                                                                    NULL_0,
                                                                                    (*fd).version,
                                                                                    &raw mut (*fd).vv,
                                                                                ) as *mut cram_codec;
                                                                                if !((*(*c).stats[DS_BA as ::core::ffi::c_int as usize])
                                                                                    .nvals != 0
                                                                                    && (*h)
                                                                                        .codecs[DS_BA as ::core::ffi::c_int as usize]
                                                                                        .is_null())
                                                                                {
                                                                                    if (*fd).version >> 8 as ::core::ffi::c_int
                                                                                        >= 3 as ::core::ffi::c_int
                                                                                    {
                                                                                        let mut e: cram_byte_array_len_encoder = cram_byte_array_len_encoder {
                                                                                            len_encoding: E_NULL,
                                                                                            val_encoding: E_NULL,
                                                                                            len_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                                                                                            val_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                                                                                            len_codec: ::core::ptr::null_mut::<cram_codec>(),
                                                                                            val_codec: ::core::ptr::null_mut::<cram_codec>(),
                                                                                        };
                                                                                        e.len_encoding = (if (*fd).version
                                                                                            >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int
                                                                                        {
                                                                                            E_VARINT_UNSIGNED as ::core::ffi::c_int
                                                                                        } else {
                                                                                            E_EXTERNAL as ::core::ffi::c_int
                                                                                        }) as cram_encoding;
                                                                                        e.len_dat = DS_BB_len as ::core::ffi::c_int
                                                                                            as *mut ::core::ffi::c_void;
                                                                                        e.val_encoding = E_EXTERNAL;
                                                                                        e.val_dat = DS_BB as ::core::ffi::c_int
                                                                                            as *mut ::core::ffi::c_void;
                                                                                        (*h).codecs[DS_BB as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                            E_BYTE_ARRAY_LEN,
                                                                                            ::core::ptr::null_mut::<cram_stats>(),
                                                                                            E_BYTE_ARRAY,
                                                                                            &raw mut e as *mut ::core::ffi::c_void,
                                                                                            (*fd).version,
                                                                                            &raw mut (*fd).vv,
                                                                                        ) as *mut cram_codec;
                                                                                        if (*h)
                                                                                            .codecs[DS_BB as ::core::ffi::c_int as usize]
                                                                                            .is_null()
                                                                                        {
                                                                                            current_block = 17869886767212223845;
                                                                                        } else {
                                                                                            current_block = 11099343707781121639;
                                                                                        }
                                                                                    } else {
                                                                                        (*h).codecs[DS_BB as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                            cram_codec,
                                                                                        >();
                                                                                        current_block = 11099343707781121639;
                                                                                    }
                                                                                    match current_block {
                                                                                        17869886767212223845 => {}
                                                                                        _ => {
                                                                                            (*h).codecs[DS_BS as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                cram_stats_encoding(
                                                                                                    fd,
                                                                                                    (*c).stats[DS_BS as ::core::ffi::c_int as usize],
                                                                                                ),
                                                                                                (*c).stats[DS_BS as ::core::ffi::c_int as usize],
                                                                                                E_BYTE,
                                                                                                NULL_0,
                                                                                                (*fd).version,
                                                                                                &raw mut (*fd).vv,
                                                                                            ) as *mut cram_codec;
                                                                                            if !((*(*c).stats[DS_BS as ::core::ffi::c_int as usize])
                                                                                                .nvals != 0
                                                                                                && (*h)
                                                                                                    .codecs[DS_BS as ::core::ffi::c_int as usize]
                                                                                                    .is_null())
                                                                                            {
                                                                                                if (*fd).version >> 8 as ::core::ffi::c_int
                                                                                                    == 1 as ::core::ffi::c_int
                                                                                                {
                                                                                                    (*h).codecs[DS_TL as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_RI as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_RS as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_PD as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_HC as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_SC as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_TC as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                        cram_stats_encoding(
                                                                                                            fd,
                                                                                                            (*c).stats[DS_TC as ::core::ffi::c_int as usize],
                                                                                                        ),
                                                                                                        (*c).stats[DS_TC as ::core::ffi::c_int as usize],
                                                                                                        E_BYTE,
                                                                                                        NULL_0,
                                                                                                        (*fd).version,
                                                                                                        &raw mut (*fd).vv,
                                                                                                    ) as *mut cram_codec;
                                                                                                    if (*(*c).stats[DS_TC as ::core::ffi::c_int as usize]).nvals
                                                                                                        != 0
                                                                                                        && (*h)
                                                                                                            .codecs[DS_TC as ::core::ffi::c_int as usize]
                                                                                                            .is_null()
                                                                                                    {
                                                                                                        current_block = 17869886767212223845;
                                                                                                    } else {
                                                                                                        (*h).codecs[DS_TN as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                            cram_stats_encoding(
                                                                                                                fd,
                                                                                                                (*c).stats[DS_TN as ::core::ffi::c_int as usize],
                                                                                                            ),
                                                                                                            (*c).stats[DS_TN as ::core::ffi::c_int as usize],
                                                                                                            E_INT,
                                                                                                            NULL_0,
                                                                                                            (*fd).version,
                                                                                                            &raw mut (*fd).vv,
                                                                                                        ) as *mut cram_codec;
                                                                                                        if (*(*c).stats[DS_TN as ::core::ffi::c_int as usize]).nvals
                                                                                                            != 0
                                                                                                            && (*h)
                                                                                                                .codecs[DS_TN as ::core::ffi::c_int as usize]
                                                                                                                .is_null()
                                                                                                        {
                                                                                                            current_block = 17869886767212223845;
                                                                                                        } else {
                                                                                                            current_block = 16286683003977321678;
                                                                                                        }
                                                                                                    }
                                                                                                } else {
                                                                                                    (*h).codecs[DS_TC as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_TN as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                                                                                                        cram_codec,
                                                                                                    >();
                                                                                                    (*h).codecs[DS_TL as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                        cram_stats_encoding(
                                                                                                            fd,
                                                                                                            (*c).stats[DS_TL as ::core::ffi::c_int as usize],
                                                                                                        ),
                                                                                                        (*c).stats[DS_TL as ::core::ffi::c_int as usize],
                                                                                                        E_INT,
                                                                                                        NULL_0,
                                                                                                        (*fd).version,
                                                                                                        &raw mut (*fd).vv,
                                                                                                    ) as *mut cram_codec;
                                                                                                    if (*(*c).stats[DS_TL as ::core::ffi::c_int as usize]).nvals
                                                                                                        != 0
                                                                                                        && (*h)
                                                                                                            .codecs[DS_TL as ::core::ffi::c_int as usize]
                                                                                                            .is_null()
                                                                                                    {
                                                                                                        current_block = 17869886767212223845;
                                                                                                    } else {
                                                                                                        (*h).codecs[DS_RI as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                            cram_stats_encoding(
                                                                                                                fd,
                                                                                                                (*c).stats[DS_RI as ::core::ffi::c_int as usize],
                                                                                                            ),
                                                                                                            (*c).stats[DS_RI as ::core::ffi::c_int as usize],
                                                                                                            E_INT,
                                                                                                            NULL_0,
                                                                                                            (*fd).version,
                                                                                                            &raw mut (*fd).vv,
                                                                                                        ) as *mut cram_codec;
                                                                                                        if (*(*c).stats[DS_RI as ::core::ffi::c_int as usize]).nvals
                                                                                                            != 0
                                                                                                            && (*h)
                                                                                                                .codecs[DS_RI as ::core::ffi::c_int as usize]
                                                                                                                .is_null()
                                                                                                        {
                                                                                                            current_block = 17869886767212223845;
                                                                                                        } else {
                                                                                                            (*h).codecs[DS_RS as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                                cram_stats_encoding(
                                                                                                                    fd,
                                                                                                                    (*c).stats[DS_RS as ::core::ffi::c_int as usize],
                                                                                                                ),
                                                                                                                (*c).stats[DS_RS as ::core::ffi::c_int as usize],
                                                                                                                E_INT,
                                                                                                                NULL_0,
                                                                                                                (*fd).version,
                                                                                                                &raw mut (*fd).vv,
                                                                                                            ) as *mut cram_codec;
                                                                                                            if (*(*c).stats[DS_RS as ::core::ffi::c_int as usize]).nvals
                                                                                                                != 0
                                                                                                                && (*h)
                                                                                                                    .codecs[DS_RS as ::core::ffi::c_int as usize]
                                                                                                                    .is_null()
                                                                                                            {
                                                                                                                current_block = 17869886767212223845;
                                                                                                            } else {
                                                                                                                (*h).codecs[DS_PD as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                                    cram_stats_encoding(
                                                                                                                        fd,
                                                                                                                        (*c).stats[DS_PD as ::core::ffi::c_int as usize],
                                                                                                                    ),
                                                                                                                    (*c).stats[DS_PD as ::core::ffi::c_int as usize],
                                                                                                                    E_INT,
                                                                                                                    NULL_0,
                                                                                                                    (*fd).version,
                                                                                                                    &raw mut (*fd).vv,
                                                                                                                ) as *mut cram_codec;
                                                                                                                if (*(*c).stats[DS_PD as ::core::ffi::c_int as usize]).nvals
                                                                                                                    != 0
                                                                                                                    && (*h)
                                                                                                                        .codecs[DS_PD as ::core::ffi::c_int as usize]
                                                                                                                        .is_null()
                                                                                                                {
                                                                                                                    current_block = 17869886767212223845;
                                                                                                                } else {
                                                                                                                    (*h).codecs[DS_HC as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                                        cram_stats_encoding(
                                                                                                                            fd,
                                                                                                                            (*c).stats[DS_HC as ::core::ffi::c_int as usize],
                                                                                                                        ),
                                                                                                                        (*c).stats[DS_HC as ::core::ffi::c_int as usize],
                                                                                                                        E_INT,
                                                                                                                        NULL_0,
                                                                                                                        (*fd).version,
                                                                                                                        &raw mut (*fd).vv,
                                                                                                                    ) as *mut cram_codec;
                                                                                                                    if (*(*c).stats[DS_HC as ::core::ffi::c_int as usize]).nvals
                                                                                                                        != 0
                                                                                                                        && (*h)
                                                                                                                            .codecs[DS_HC as ::core::ffi::c_int as usize]
                                                                                                                            .is_null()
                                                                                                                    {
                                                                                                                        current_block = 17869886767212223845;
                                                                                                                    } else {
                                                                                                                        let mut i2: [::core::ffi::c_int; 2] = [
                                                                                                                            0 as ::core::ffi::c_int,
                                                                                                                            DS_SC as ::core::ffi::c_int,
                                                                                                                        ];
                                                                                                                        (*h).codecs[DS_SC as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                                            E_BYTE_ARRAY_STOP,
                                                                                                                            ::core::ptr::null_mut::<cram_stats>(),
                                                                                                                            E_BYTE_ARRAY,
                                                                                                                            &raw mut i2 as *mut ::core::ffi::c_int
                                                                                                                                as *mut ::core::ffi::c_void,
                                                                                                                            (*fd).version,
                                                                                                                            &raw mut (*fd).vv,
                                                                                                                        ) as *mut cram_codec;
                                                                                                                        if (*h)
                                                                                                                            .codecs[DS_SC as ::core::ffi::c_int as usize]
                                                                                                                            .is_null()
                                                                                                                        {
                                                                                                                            current_block = 17869886767212223845;
                                                                                                                        } else {
                                                                                                                            current_block = 16286683003977321678;
                                                                                                                        }
                                                                                                                    }
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                                match current_block {
                                                                                                    17869886767212223845 => {}
                                                                                                    _ => {
                                                                                                        let mut i2_0: [::core::ffi::c_int; 2] = [
                                                                                                            0 as ::core::ffi::c_int,
                                                                                                            DS_IN as ::core::ffi::c_int,
                                                                                                        ];
                                                                                                        (*h).codecs[DS_IN as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                            E_BYTE_ARRAY_STOP,
                                                                                                            ::core::ptr::null_mut::<cram_stats>(),
                                                                                                            E_BYTE_ARRAY,
                                                                                                            &raw mut i2_0 as *mut ::core::ffi::c_int
                                                                                                                as *mut ::core::ffi::c_void,
                                                                                                            (*fd).version,
                                                                                                            &raw mut (*fd).vv,
                                                                                                        ) as *mut cram_codec;
                                                                                                        if !(*h)
                                                                                                            .codecs[DS_IN as ::core::ffi::c_int as usize]
                                                                                                            .is_null()
                                                                                                        {
                                                                                                            (*h).codecs[DS_QS as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                                E_EXTERNAL,
                                                                                                                ::core::ptr::null_mut::<cram_stats>(),
                                                                                                                E_BYTE,
                                                                                                                DS_QS as ::core::ffi::c_int as *mut ::core::ffi::c_void,
                                                                                                                (*fd).version,
                                                                                                                &raw mut (*fd).vv,
                                                                                                            ) as *mut cram_codec;
                                                                                                            if !(*h)
                                                                                                                .codecs[DS_QS as ::core::ffi::c_int as usize]
                                                                                                                .is_null()
                                                                                                            {
                                                                                                                let mut i2_1: [::core::ffi::c_int; 2] = [
                                                                                                                    0 as ::core::ffi::c_int,
                                                                                                                    DS_RN as ::core::ffi::c_int,
                                                                                                                ];
                                                                                                                (*h).codecs[DS_RN as ::core::ffi::c_int as usize] = cram_encoder_init(
                                                                                                                    E_BYTE_ARRAY_STOP,
                                                                                                                    ::core::ptr::null_mut::<cram_stats>(),
                                                                                                                    E_BYTE_ARRAY,
                                                                                                                    &raw mut i2_1 as *mut ::core::ffi::c_int
                                                                                                                        as *mut ::core::ffi::c_void,
                                                                                                                    (*fd).version,
                                                                                                                    &raw mut (*fd).vv,
                                                                                                                ) as *mut cram_codec;
                                                                                                                if !(*h)
                                                                                                                    .codecs[DS_RN as ::core::ffi::c_int as usize]
                                                                                                                    .is_null()
                                                                                                                {
                                                                                                                    i = 0 as ::core::ffi::c_int;
                                                                                                                    while i < (*c).curr_slice {
                                                                                                                        hts_log(
                                                                                                                            HTS_LOG_INFO,
                                                                                                                            b"cram_encode_container\0" as *const u8
                                                                                                                                as *const ::core::ffi::c_char,
                                                                                                                            b"Encode slice %d\0" as *const u8
                                                                                                                                as *const ::core::ffi::c_char,
                                                                                                                            i,
                                                                                                                        );
                                                                                                                        let mut local_embed_ref: ::core::ffi::c_int = if embed_ref
                                                                                                                            > 0 as ::core::ffi::c_int
                                                                                                                            && (*(**(*c).slices.offset(i as isize)).hdr).ref_seq_id
                                                                                                                                != -(1 as int32_t)
                                                                                                                        {
                                                                                                                            1 as ::core::ffi::c_int
                                                                                                                        } else {
                                                                                                                            0 as ::core::ffi::c_int
                                                                                                                        };
                                                                                                                        if cram_encode_slice(
                                                                                                                            fd,
                                                                                                                            c,
                                                                                                                            h,
                                                                                                                            *(*c).slices.offset(i as isize) as *mut cram_slice,
                                                                                                                            local_embed_ref,
                                                                                                                        ) != 0 as ::core::ffi::c_int
                                                                                                                        {
                                                                                                                            return -(1 as ::core::ffi::c_int);
                                                                                                                        }
                                                                                                                        i += 1;
                                                                                                                    }
                                                                                                                    (*h).ref_seq_id = (*c).ref_seq_id;
                                                                                                                    (*h).ref_seq_start = (*c).ref_seq_start;
                                                                                                                    (*h).ref_seq_span = (*c).ref_seq_span;
                                                                                                                    (*h).num_records = (*c).num_records;
                                                                                                                    (*h).qs_seq_orient = (*c).qs_seq_orient;
                                                                                                                    (*h).AP_delta = (*c).pos_sorted;
                                                                                                                    memcpy(
                                                                                                                        &raw mut (*h).substitution_matrix
                                                                                                                            as *mut [::core::ffi::c_char; 4]
                                                                                                                            as *mut ::core::ffi::c_void,
                                                                                                                        CRAM_SUBST_MATRIX.as_ptr() as *const ::core::ffi::c_void,
                                                                                                                        20 as size_t,
                                                                                                                    );
                                                                                                                    c_hdr = cram_encode_compression_header(fd, c, h, embed_ref);
                                                                                                                    if c_hdr.is_null() {
                                                                                                                        return -(1 as ::core::ffi::c_int);
                                                                                                                    }
                                                                                                                    (*c).num_landmarks = (*c).curr_slice as int32_t;
                                                                                                                    (*c).landmark = malloc(
                                                                                                                        ((*c).num_landmarks as size_t)
                                                                                                                            .wrapping_mul(::core::mem::size_of::<int32_t>() as size_t),
                                                                                                                    ) as *mut int32_t;
                                                                                                                    if (*c).landmark.is_null() {
                                                                                                                        return -(1 as ::core::ffi::c_int);
                                                                                                                    }
                                                                                                                    slice_offset = (if (*c_hdr).method as ::core::ffi::c_int
                                                                                                                        == RAW as ::core::ffi::c_int
                                                                                                                    {
                                                                                                                        (*c_hdr).uncomp_size
                                                                                                                    } else {
                                                                                                                        (*c_hdr).comp_size
                                                                                                                    }) as ::core::ffi::c_int;
                                                                                                                    slice_offset
                                                                                                                        += 2 as ::core::ffi::c_int
                                                                                                                            + 4 as ::core::ffi::c_int
                                                                                                                                * ((*fd).version >> 8 as ::core::ffi::c_int
                                                                                                                                    >= 3 as ::core::ffi::c_int) as ::core::ffi::c_int
                                                                                                                            + (*fd)
                                                                                                                                .vv
                                                                                                                                .varint_size
                                                                                                                                .expect(
                                                                                                                                    "non-null function pointer",
                                                                                                                                )((*c_hdr).content_id as int64_t)
                                                                                                                            + (*fd)
                                                                                                                                .vv
                                                                                                                                .varint_size
                                                                                                                                .expect(
                                                                                                                                    "non-null function pointer",
                                                                                                                                )((*c_hdr).comp_size as int64_t)
                                                                                                                            + (*fd)
                                                                                                                                .vv
                                                                                                                                .varint_size
                                                                                                                                .expect(
                                                                                                                                    "non-null function pointer",
                                                                                                                                )((*c_hdr).uncomp_size as int64_t);
                                                                                                                    (*c).ref_seq_id = (*(**(*c)
                                                                                                                        .slices
                                                                                                                        .offset(0 as ::core::ffi::c_int as isize))
                                                                                                                        .hdr)
                                                                                                                        .ref_seq_id;
                                                                                                                    if (*c).ref_seq_id == -(1 as int32_t)
                                                                                                                        && (*fd).version >= 0x301 as ::core::ffi::c_int
                                                                                                                    {
                                                                                                                        (*c).ref_seq_start = 0 as int64_t;
                                                                                                                        (*c).ref_seq_span = 0 as int64_t;
                                                                                                                    } else {
                                                                                                                        (*c).ref_seq_start = (*(**(*c)
                                                                                                                            .slices
                                                                                                                            .offset(0 as ::core::ffi::c_int as isize))
                                                                                                                            .hdr)
                                                                                                                            .ref_seq_start;
                                                                                                                        (*c).ref_seq_span = (*(**(*c)
                                                                                                                            .slices
                                                                                                                            .offset(0 as ::core::ffi::c_int as isize))
                                                                                                                            .hdr)
                                                                                                                            .ref_seq_span;
                                                                                                                    }
                                                                                                                    i = 0 as ::core::ffi::c_int;
                                                                                                                    while i < (*c).curr_slice {
                                                                                                                        let mut s_1: *mut cram_slice = *(*c)
                                                                                                                            .slices
                                                                                                                            .offset(i as isize) as *mut cram_slice;
                                                                                                                        (*c).num_blocks = ((*c).num_blocks as ::core::ffi::c_int
                                                                                                                            + ((*(*s_1).hdr).num_blocks + 1 as int32_t)
                                                                                                                                as ::core::ffi::c_int) as int32_t;
                                                                                                                        *(*c).landmark.offset(i as isize) = slice_offset as int32_t;
                                                                                                                        if (*(*s_1).hdr).ref_seq_start + (*(*s_1).hdr).ref_seq_span
                                                                                                                            > (*c).ref_seq_start + (*c).ref_seq_span
                                                                                                                        {
                                                                                                                            (*c).ref_seq_span = (*(*s_1).hdr).ref_seq_start
                                                                                                                                + (*(*s_1).hdr).ref_seq_span - (*c).ref_seq_start;
                                                                                                                        }
                                                                                                                        slice_offset
                                                                                                                            += (if (*(*s_1).hdr_block).method as ::core::ffi::c_int
                                                                                                                                == RAW as ::core::ffi::c_int
                                                                                                                            {
                                                                                                                                (*(*s_1).hdr_block).uncomp_size
                                                                                                                            } else {
                                                                                                                                (*(*s_1).hdr_block).comp_size
                                                                                                                            }) as ::core::ffi::c_int;
                                                                                                                        slice_offset
                                                                                                                            += 2 as ::core::ffi::c_int
                                                                                                                                + 4 as ::core::ffi::c_int
                                                                                                                                    * ((*fd).version >> 8 as ::core::ffi::c_int
                                                                                                                                        >= 3 as ::core::ffi::c_int) as ::core::ffi::c_int
                                                                                                                                + (*fd)
                                                                                                                                    .vv
                                                                                                                                    .varint_size
                                                                                                                                    .expect(
                                                                                                                                        "non-null function pointer",
                                                                                                                                    )((*(*s_1).hdr_block).content_id as int64_t)
                                                                                                                                + (*fd)
                                                                                                                                    .vv
                                                                                                                                    .varint_size
                                                                                                                                    .expect(
                                                                                                                                        "non-null function pointer",
                                                                                                                                    )((*(*s_1).hdr_block).comp_size as int64_t)
                                                                                                                                + (*fd)
                                                                                                                                    .vv
                                                                                                                                    .varint_size
                                                                                                                                    .expect(
                                                                                                                                        "non-null function pointer",
                                                                                                                                    )((*(*s_1).hdr_block).uncomp_size as int64_t);
                                                                                                                        j = 0 as ::core::ffi::c_int;
                                                                                                                        while (j as int32_t) < (*(*s_1).hdr).num_blocks {
                                                                                                                            slice_offset
                                                                                                                                += 2 as ::core::ffi::c_int
                                                                                                                                    + 4 as ::core::ffi::c_int
                                                                                                                                        * ((*fd).version >> 8 as ::core::ffi::c_int
                                                                                                                                            >= 3 as ::core::ffi::c_int) as ::core::ffi::c_int
                                                                                                                                    + (*fd)
                                                                                                                                        .vv
                                                                                                                                        .varint_size
                                                                                                                                        .expect(
                                                                                                                                            "non-null function pointer",
                                                                                                                                        )((**(*s_1).block.offset(j as isize)).content_id as int64_t)
                                                                                                                                    + (*fd)
                                                                                                                                        .vv
                                                                                                                                        .varint_size
                                                                                                                                        .expect(
                                                                                                                                            "non-null function pointer",
                                                                                                                                        )((**(*s_1).block.offset(j as isize)).comp_size as int64_t)
                                                                                                                                    + (*fd)
                                                                                                                                        .vv
                                                                                                                                        .varint_size
                                                                                                                                        .expect(
                                                                                                                                            "non-null function pointer",
                                                                                                                                        )(
                                                                                                                                        (**(*s_1).block.offset(j as isize)).uncomp_size as int64_t,
                                                                                                                                    );
                                                                                                                            slice_offset
                                                                                                                                += (if (**(*s_1).block.offset(j as isize)).method
                                                                                                                                    as ::core::ffi::c_int == RAW as ::core::ffi::c_int
                                                                                                                                {
                                                                                                                                    (**(*s_1).block.offset(j as isize)).uncomp_size
                                                                                                                                } else {
                                                                                                                                    (**(*s_1).block.offset(j as isize)).comp_size
                                                                                                                                }) as ::core::ffi::c_int;
                                                                                                                            j += 1;
                                                                                                                        }
                                                                                                                        i += 1;
                                                                                                                    }
                                                                                                                    (*c).length = ((*c).length as ::core::ffi::c_int
                                                                                                                        + slice_offset) as int32_t;
                                                                                                                    (*c).comp_hdr_block = c_hdr;
                                                                                                                    if (*c).ref_seq_id >= 0 as int32_t {
                                                                                                                        if (*c).ref_free != 0 {
                                                                                                                            free((*c).ref_0 as *mut ::core::ffi::c_void);
                                                                                                                            (*c).ref_0 = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                                                                                                        } else {
                                                                                                                            cram_ref_decr(
                                                                                                                                (*fd).refs,
                                                                                                                                (*c).ref_seq_id as ::core::ffi::c_int,
                                                                                                                            );
                                                                                                                        }
                                                                                                                    }
                                                                                                                    if no_ref == 0 && !(*c).refs_used.is_null() {
                                                                                                                        i = 0 as ::core::ffi::c_int;
                                                                                                                        while i < (*(*fd).refs).nref {
                                                                                                                            if *(*c).refs_used.offset(i as isize) != 0 {
                                                                                                                                cram_ref_decr((*fd).refs, i);
                                                                                                                            }
                                                                                                                            i += 1;
                                                                                                                        }
                                                                                                                    }
                                                                                                                    return 0 as ::core::ffi::c_int;
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
                        }
                    }
                }
            }
        }
    }
    return -(1 as ::core::ffi::c_int);
}
// original: cram_add_feature (htslib/cram/cram_encode.c:2574)
unsafe extern "C" fn cram_add_feature(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut f: *mut cram_feature,
) -> ::core::ffi::c_int {
    if (*s).nfeatures >= (*s).afeatures {
        (*s).afeatures = if (*s).afeatures != 0 {
            (*s).afeatures.wrapping_mul(2 as uint32_t)
        } else {
            1024 as uint32_t
        };
        (*s).features = realloc(
            (*s).features as *mut ::core::ffi::c_void,
            ((*s).afeatures as size_t)
                .wrapping_mul(::core::mem::size_of::<cram_feature>() as size_t),
        ) as *mut cram_feature;
        if (*s).features.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
    }
    let fresh163 = (*r).nfeature;
    (*r).nfeature = (*r).nfeature.wrapping_add(1);
    if fresh163 == 0 {
        (*r).feature = (*s).nfeatures;
        if cram_stats_add(
            (*c).stats[DS_FP as ::core::ffi::c_int as usize],
            (*f).X.pos as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else if cram_stats_add(
        (*c).stats[DS_FP as ::core::ffi::c_int as usize],
        ((*f).X.pos
            - (*(*s).features.offset(
                (*r).feature
                    .wrapping_add((*r).nfeature)
                    .wrapping_sub(2 as uint32_t) as isize,
            ))
            .X
            .pos) as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if cram_stats_add(
        (*c).stats[DS_FC as ::core::ffi::c_int as usize],
        (*f).X.code as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    let fresh164 = (*s).nfeatures;
    (*s).nfeatures = (*s).nfeatures.wrapping_add(1);
    *(*s).features.offset(fresh164 as isize) = *f;
    return 0 as ::core::ffi::c_int;
}
// original: cram_add_substitution (htslib/cram/cram_encode.c:2601)
unsafe extern "C" fn cram_add_substitution(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut base: ::core::ffi::c_char,
    mut qual: ::core::ffi::c_char,
    mut ref_0: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    if ((*fd).L2[base as uc as usize] as ::core::ffi::c_int) < 4 as ::core::ffi::c_int
        || ((*fd).L2[base as uc as usize] as ::core::ffi::c_int) < 5 as ::core::ffi::c_int
            && ((*fd).L2[ref_0 as uc as usize] as ::core::ffi::c_int) < 4 as ::core::ffi::c_int
    {
        f.X.pos = pos + 1 as ::core::ffi::c_int;
        f.X.code = 'X' as i32;
        f.X.base = (*fd).cram_sub_matrix
            [(ref_0 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as usize]
            [(base as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int;
        if cram_stats_add(
            (*c).stats[DS_BS as ::core::ffi::c_int as usize],
            f.X.base as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else {
        f.B.pos = pos + 1 as ::core::ffi::c_int;
        f.B.code = 'B' as i32;
        f.B.base = base as ::core::ffi::c_int;
        f.B.qual = qual as ::core::ffi::c_int;
        if cram_stats_add(
            (*c).stats[DS_BA as ::core::ffi::c_int as usize],
            f.B.base as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
        if cram_stats_add(
            (*c).stats[DS_QS as ::core::ffi::c_int as usize],
            f.B.qual as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
        if block_append_char((*s).qual_blk, qual) < 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_add_bases (htslib/cram/cram_encode.c:2628)
unsafe extern "C" fn cram_add_bases(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.b.pos = pos + 1 as ::core::ffi::c_int;
    f.b.code = 'b' as i32;
    f.b.seq_idx = base.offset_from((*(*s).seqs_blk).data as *mut ::core::ffi::c_char)
        as ::core::ffi::c_long as ::core::ffi::c_int;
    f.b.len = len;
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_add_base (htslib/cram/cram_encode.c:2641)
unsafe extern "C" fn cram_add_base(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut base: ::core::ffi::c_char,
    mut qual: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.B.pos = pos + 1 as ::core::ffi::c_int;
    f.B.code = 'B' as i32;
    f.B.base = base as ::core::ffi::c_int;
    f.B.qual = qual as ::core::ffi::c_int;
    if cram_stats_add(
        (*c).stats[DS_BA as ::core::ffi::c_int as usize],
        base as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if cram_stats_add(
        (*c).stats[DS_QS as ::core::ffi::c_int as usize],
        qual as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if block_append_char((*s).qual_blk, qual) < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    } else {
        return cram_add_feature(c, s, r, &raw mut f);
    };
}
// original: cram_add_quality (htslib/cram/cram_encode.c:2658)
unsafe extern "C" fn cram_add_quality(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut qual: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.Q.pos = pos + 1 as ::core::ffi::c_int;
    f.Q.code = 'Q' as i32;
    f.Q.qual = qual as ::core::ffi::c_int;
    if cram_stats_add(
        (*c).stats[DS_QS as ::core::ffi::c_int as usize],
        qual as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if block_append_char((*s).qual_blk, qual) < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    } else {
        return cram_add_feature(c, s, r, &raw mut f);
    };
}
// original: cram_add_deletion (htslib/cram/cram_encode.c:2673)
unsafe extern "C" fn cram_add_deletion(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.D.pos = pos + 1 as ::core::ffi::c_int;
    f.D.code = 'D' as i32;
    f.D.len = len;
    if cram_stats_add(
        (*c).stats[DS_DL as ::core::ffi::c_int as usize],
        len as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_add_softclip (htslib/cram/cram_encode.c:2683)
unsafe extern "C" fn cram_add_softclip(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
    mut version: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.S.pos = pos + 1 as ::core::ffi::c_int;
    f.S.code = 'S' as i32;
    f.S.len = len;
    match version >> 8 as ::core::ffi::c_int {
        1 => {
            f.S.seq_idx = (*(*s).base_blk).byte as ::core::ffi::c_int;
            if block_append(
                (*s).base_blk,
                base as *const ::core::ffi::c_void,
                len as size_t,
            ) < 0 as ::core::ffi::c_int
            {
                current_block = 872639294066737149;
            } else if block_append_char((*s).base_blk, '\0' as i32 as ::core::ffi::c_char)
                < 0 as ::core::ffi::c_int
            {
                current_block = 872639294066737149;
            } else {
                current_block = 13056961889198038528;
            }
        }
        2 | _ => {
            f.S.seq_idx = (*(*s).soft_blk).byte as ::core::ffi::c_int;
            if !base.is_null() {
                if block_append(
                    (*s).soft_blk,
                    base as *const ::core::ffi::c_void,
                    len as size_t,
                ) < 0 as ::core::ffi::c_int
                {
                    current_block = 872639294066737149;
                } else {
                    current_block = 3512920355445576850;
                }
            } else {
                let mut i: ::core::ffi::c_int = 0;
                i = 0 as ::core::ffi::c_int;
                loop {
                    if !(i < len) {
                        current_block = 3512920355445576850;
                        break;
                    }
                    if block_append_char((*s).soft_blk, 'N' as i32 as ::core::ffi::c_char)
                        < 0 as ::core::ffi::c_int
                    {
                        current_block = 872639294066737149;
                        break;
                    }
                    i += 1;
                }
            }
            match current_block {
                872639294066737149 => {}
                _ => {
                    if block_append_char((*s).soft_blk, '\0' as i32 as ::core::ffi::c_char)
                        < 0 as ::core::ffi::c_int
                    {
                        current_block = 872639294066737149;
                    } else {
                        current_block = 13056961889198038528;
                    }
                }
            }
        }
    }
    match current_block {
        13056961889198038528 => return cram_add_feature(c, s, r, &raw mut f),
        _ => return -(1 as ::core::ffi::c_int),
    };
}
// original: cram_add_hardclip (htslib/cram/cram_encode.c:2719)
unsafe extern "C" fn cram_add_hardclip(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.S.pos = pos + 1 as ::core::ffi::c_int;
    f.S.code = 'H' as i32;
    f.S.len = len;
    if cram_stats_add(
        (*c).stats[DS_HC as ::core::ffi::c_int as usize],
        len as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_add_skip (htslib/cram/cram_encode.c:2729)
unsafe extern "C" fn cram_add_skip(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.S.pos = pos + 1 as ::core::ffi::c_int;
    f.S.code = 'N' as i32;
    f.S.len = len;
    if cram_stats_add(
        (*c).stats[DS_RS as ::core::ffi::c_int as usize],
        len as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_add_pad (htslib/cram/cram_encode.c:2739)
unsafe extern "C" fn cram_add_pad(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.S.pos = pos + 1 as ::core::ffi::c_int;
    f.S.code = 'P' as i32;
    f.S.len = len;
    if cram_stats_add(
        (*c).stats[DS_PD as ::core::ffi::c_int as usize],
        len as int64_t,
    ) < 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_add_insertion (htslib/cram/cram_encode.c:2749)
unsafe extern "C" fn cram_add_insertion(
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut r: *mut cram_record,
    mut pos: ::core::ffi::c_int,
    mut len: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut f: cram_feature = cram_feature {
        X: C2RustUnnamed_11 {
            pos: 0,
            code: 0,
            base: 0,
        },
    };
    f.I.pos = pos + 1 as ::core::ffi::c_int;
    if len == 1 as ::core::ffi::c_int {
        let mut b: ::core::ffi::c_char = (if !base.is_null() {
            *base as ::core::ffi::c_int
        } else {
            'N' as i32
        }) as ::core::ffi::c_char;
        f.i.code = 'i' as i32;
        f.i.base = b as ::core::ffi::c_int;
        if cram_stats_add(
            (*c).stats[DS_BA as ::core::ffi::c_int as usize],
            b as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else {
        f.I.code = 'I' as i32;
        f.I.len = len;
        f.S.seq_idx = (*(*s).base_blk).byte as ::core::ffi::c_int;
        if !base.is_null() {
            if block_append(
                (*s).base_blk,
                base as *const ::core::ffi::c_void,
                len as size_t,
            ) < 0 as ::core::ffi::c_int
            {
                current_block = 12098463999765396342;
            } else {
                current_block = 4166486009154926805;
            }
        } else {
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            loop {
                if !(i < len) {
                    current_block = 4166486009154926805;
                    break;
                }
                if block_append_char((*s).base_blk, 'N' as i32 as ::core::ffi::c_char)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 12098463999765396342;
                    break;
                }
                i += 1;
            }
        }
        match current_block {
            4166486009154926805 => {
                if block_append_char((*s).base_blk, '\0' as i32 as ::core::ffi::c_char)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 12098463999765396342;
                } else {
                    current_block = 17407779659766490442;
                }
            }
            _ => {}
        }
        match current_block {
            17407779659766490442 => {}
            _ => return -(1 as ::core::ffi::c_int),
        }
    }
    return cram_add_feature(c, s, r, &raw mut f);
}
// original: cram_encode_aux (htslib/cram/cram_encode.c:2784)
unsafe extern "C" fn cram_encode_aux(
    mut fd: *mut cram_fd,
    mut b: *mut bam_seq_t,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut cr: *mut cram_record,
    mut verbatim_NM: ::core::ffi::c_int,
    mut verbatim_MD: ::core::ffi::c_int,
    mut NM: ::core::ffi::c_int,
    mut MD: *mut kstring_t,
    mut cf_tag: ::core::ffi::c_int,
    mut no_ref: ::core::ffi::c_int,
    mut err: *mut ::core::ffi::c_int,
) -> *mut sam_hrec_rg_t {
    let mut current_block: u64;
    let mut aux: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut orig: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut brg: *mut sam_hrec_rg_t = ::core::ptr::null_mut::<sam_hrec_rg_t>();
    let mut aux_size: ::core::ffi::c_int = ((*b).l_data as uint32_t)
        .wrapping_sub((*b).core.n_cigar << 2 as ::core::ffi::c_int)
        .wrapping_sub((*b).core.l_qname as uint32_t)
        .wrapping_sub((*b).core.l_qseq as uint32_t)
        .wrapping_sub(((*b).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int) as uint32_t)
        as ::core::ffi::c_int;
    let mut aux_end: *const ::core::ffi::c_char = bam_data_end(b as *mut bam1_t);
    let mut td_b: *mut cram_block = (*(*c).comp_hdr).TD_blk;
    let mut TD_blk_size: ::core::ffi::c_int = (*td_b).byte as ::core::ffi::c_int;
    let mut new: ::core::ffi::c_int = 0;
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut k: khint_t = 0;
    if !err.is_null() {
        *err = 1 as ::core::ffi::c_int;
    }
    aux = (*b)
        .data
        .offset(((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize)
        .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
        .offset(((*b).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int) as isize)
        .offset((*b).core.l_qseq as isize) as *mut ::core::ffi::c_char;
    orig = aux;
    if cf_tag != 0 && ((*fd).version >> 8 as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
        aux = malloc((aux_size + 4 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        if aux.is_null() {
            return ::core::ptr::null_mut::<sam_hrec_rg_t>();
        }
        memcpy(
            aux as *mut ::core::ffi::c_void,
            orig as *const ::core::ffi::c_void,
            aux_size as size_t,
        );
        let fresh151 = aux_size;
        aux_size = aux_size + 1;
        *aux.offset(fresh151 as isize) = 'c' as i32 as ::core::ffi::c_char;
        let fresh152 = aux_size;
        aux_size = aux_size + 1;
        *aux.offset(fresh152 as isize) = 'F' as i32 as ::core::ffi::c_char;
        let fresh153 = aux_size;
        aux_size = aux_size + 1;
        *aux.offset(fresh153 as isize) = 'C' as i32 as ::core::ffi::c_char;
        let fresh154 = aux_size;
        aux_size = aux_size + 1;
        *aux.offset(fresh154 as isize) = cf_tag as ::core::ffi::c_char;
        orig = aux;
        aux_end = aux.offset(aux_size as isize);
    }
    loop {
        if !(aux_end.offset_from(aux) as ::core::ffi::c_long >= 1 as ::core::ffi::c_long
            && *aux.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
        {
            current_block = 13391418783698890455;
            break;
        }
        let mut r: ::core::ffi::c_int = 0;
        if aux.offset_from(orig) as ::core::ffi::c_long
            >= (aux_size - 3 as ::core::ffi::c_int) as ::core::ffi::c_long
        {
            current_block = 9865445363914956224;
            break;
        }
        if *aux.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'R' as i32
            && *aux.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'G' as i32
            && *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'Z' as i32
        {
            let mut rg: *mut ::core::ffi::c_char =
                aux.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
            aux = rg;
            while aux < aux_end as *mut ::core::ffi::c_char && {
                let fresh155 = aux;
                aux = aux.offset(1);
                *fresh155 as ::core::ffi::c_int != 0
            } {}
            if aux == aux_end as *mut ::core::ffi::c_char
                && *aux.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                hts_log(
                    HTS_LOG_ERROR,
                    b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Unterminated RG:Z tag for read \"%s\"\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*b).data as *mut ::core::ffi::c_char,
                );
                current_block = 9865445363914956224;
                break;
            } else {
                brg = sam_hrecs_find_rg((*(*fd).header).hrecs, rg);
                if !brg.is_null() {
                    if !((*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int) {
                        continue;
                    }
                    if block_append(
                        td_b,
                        b"RG*\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        3 as size_t,
                    ) < 0 as ::core::ffi::c_int
                    {
                        current_block = 9865445363914956224;
                        break;
                    } else {
                        continue;
                    }
                } else {
                    hts_log(
                        HTS_LOG_WARNING,
                        b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Missing @RG header for RG \"%s\"\0" as *const u8
                            as *const ::core::ffi::c_char,
                        rg,
                    );
                    aux = rg.offset(-(3 as ::core::ffi::c_int as isize));
                }
            }
        }
        if *aux.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32
            && *aux.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'D' as i32
            && *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'Z' as i32
        {
            if (*cr).len != 0
                && no_ref == 0
                && (*cr).flags & BAM_FUNMAP as int32_t == 0
                && verbatim_MD == 0
            {
                if !MD.is_null()
                    && !(*MD).s.is_null()
                    && strncasecmp(
                        (*MD).s,
                        aux.offset(3 as ::core::ffi::c_int as isize),
                        orig.offset(aux_size as isize)
                            .offset_from(aux.offset(3 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_long as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    while aux < aux_end as *mut ::core::ffi::c_char && {
                        let fresh156 = aux;
                        aux = aux.offset(1);
                        *fresh156 as ::core::ffi::c_int != 0
                    } {}
                    if aux == aux_end as *mut ::core::ffi::c_char
                        && *aux.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                            != '\0' as i32
                    {
                        hts_log(
                            HTS_LOG_ERROR,
                            b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                            b"Unterminated MD:Z tag for read \"%s\"\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*b).data as *mut ::core::ffi::c_char,
                        );
                        current_block = 9865445363914956224;
                        break;
                    } else {
                        if !((*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int) {
                            continue;
                        }
                        if block_append(
                            td_b,
                            b"MD*\0" as *const u8 as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            3 as size_t,
                        ) < 0 as ::core::ffi::c_int
                        {
                            current_block = 9865445363914956224;
                            break;
                        } else {
                            continue;
                        }
                    }
                }
            }
        }
        if *aux.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'N' as i32
            && *aux.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32
        {
            if (*cr).len != 0
                && no_ref == 0
                && (*cr).flags & BAM_FUNMAP as int32_t == 0
                && verbatim_NM == 0
            {
                let mut NM_: ::core::ffi::c_int = bam_aux2i_end(
                    (aux as *mut uint8_t).offset(2 as ::core::ffi::c_int as isize),
                    aux_end as *mut uint8_t,
                );
                if NM_ == NM {
                    match *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                        65 | 67 | 99 => {
                            aux = aux.offset(4 as ::core::ffi::c_int as isize);
                        }
                        83 | 115 => {
                            aux = aux.offset(5 as ::core::ffi::c_int as isize);
                        }
                        73 | 105 | 102 => {
                            aux = aux.offset(7 as ::core::ffi::c_int as isize);
                        }
                        _ => {
                            hts_log(
                                HTS_LOG_ERROR,
                                b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                                b"Unhandled type code for NM tag\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                            current_block = 9865445363914956224;
                            break;
                        }
                    }
                    if !((*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int) {
                        continue;
                    }
                    if block_append(
                        td_b,
                        b"NM*\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        3 as size_t,
                    ) < 0 as ::core::ffi::c_int
                    {
                        current_block = 9865445363914956224;
                        break;
                    } else {
                        continue;
                    }
                }
            }
        }
        if block_append(td_b, aux as *const ::core::ffi::c_void, 3 as size_t)
            < 0 as ::core::ffi::c_int
        {
            current_block = 9865445363914956224;
            break;
        }
        let mut key_0: ::core::ffi::c_int = (*(aux as *mut ::core::ffi::c_uchar)
            .offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int
            | (*(aux as *mut ::core::ffi::c_uchar).offset(1 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
            | *(aux as *mut ::core::ffi::c_uchar).offset(2 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int;
        k = kh_put_m_tagmap((*c).tags_used, key_0 as khint32_t, &raw mut r);
        if -(1 as ::core::ffi::c_int) == r {
            current_block = 9865445363914956224;
            break;
        }
        if r != 0 as ::core::ffi::c_int {
            let ref mut fresh157 = *(*(*c).tags_used).vals.offset(k as isize);
            *fresh157 = ::core::ptr::null_mut::<cram_tag_map>();
        }
        if r == 1 as ::core::ffi::c_int {
            let mut k_global: khint_t = 0;
            pthread_mutex_lock(&raw mut (*fd).metrics_lock);
            k_global = kh_put_m_metrics((*fd).tags_used, key_0 as khint32_t, &raw mut r);
            if -(1 as ::core::ffi::c_int) == r {
                pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
                current_block = 9865445363914956224;
                break;
            } else {
                if r >= 1 as ::core::ffi::c_int {
                    let ref mut fresh158 = *(*(*fd).tags_used).vals.offset(k_global as isize);
                    *fresh158 = cram_new_metrics();
                    if (*(*(*fd).tags_used).vals.offset(k_global as isize)).is_null() {
                        kh_del_m_metrics((*fd).tags_used, k_global);
                        pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
                        current_block = 9865445363914956224;
                        break;
                    }
                }
                pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
                let mut i2: [::core::ffi::c_int; 2] = ['\t' as i32, key_0];
                let mut sk: size_t = key_0 as size_t;
                let mut m: *mut cram_tag_map = calloc(
                    1 as size_t,
                    ::core::mem::size_of::<cram_tag_map>() as size_t,
                ) as *mut cram_tag_map;
                if m.is_null() {
                    current_block = 9865445363914956224;
                    break;
                }
                let ref mut fresh159 = *(*(*c).tags_used).vals.offset(k as isize);
                *fresh159 = m;
                let mut c_0: *mut cram_codec = ::core::ptr::null_mut::<cram_codec>();
                match *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                    90 | 72 => {
                        c_0 = cram_encoder_init(
                            E_BYTE_ARRAY_STOP,
                            ::core::ptr::null_mut::<cram_stats>(),
                            E_BYTE_ARRAY,
                            &raw mut i2 as *mut ::core::ffi::c_int as *mut ::core::ffi::c_void,
                            (*fd).version,
                            &raw mut (*fd).vv,
                        );
                    }
                    65 | 99 | 67 => {
                        let mut e: cram_byte_array_len_encoder = cram_byte_array_len_encoder {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            val_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            len_codec: ::core::ptr::null_mut::<cram_codec>(),
                            val_codec: ::core::ptr::null_mut::<cram_codec>(),
                        };
                        let mut st: cram_stats = cram_stats {
                            freqs: [0; 1024],
                            h: ::core::ptr::null_mut::<kh_m_i2i_t>(),
                            nsamp: 0,
                            nvals: 0,
                            min_val: 0,
                            max_val: 0,
                        };
                        if (*fd).version >> 8 as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
                            e.len_encoding = E_HUFFMAN;
                            e.len_dat = NULL_0;
                        } else {
                            e.len_encoding = E_CONST_INT;
                            e.len_dat = NULL_0;
                        }
                        memset(
                            &raw mut st as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            ::core::mem::size_of::<cram_stats>() as size_t,
                        );
                        if cram_stats_add(&raw mut st, 1 as int64_t) < 0 as ::core::ffi::c_int {
                            current_block = 9865445363914956224;
                            break;
                        }
                        cram_stats_encoding(fd, &raw mut st);
                        e.val_encoding = E_EXTERNAL;
                        e.val_dat = sk as *mut ::core::ffi::c_void;
                        c_0 = cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            &raw mut st,
                            E_BYTE_ARRAY,
                            &raw mut e as *mut ::core::ffi::c_void,
                            (*fd).version,
                            &raw mut (*fd).vv,
                        );
                    }
                    115 | 83 => {
                        let mut e_0: cram_byte_array_len_encoder = cram_byte_array_len_encoder {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            val_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            len_codec: ::core::ptr::null_mut::<cram_codec>(),
                            val_codec: ::core::ptr::null_mut::<cram_codec>(),
                        };
                        let mut st_0: cram_stats = cram_stats {
                            freqs: [0; 1024],
                            h: ::core::ptr::null_mut::<kh_m_i2i_t>(),
                            nsamp: 0,
                            nvals: 0,
                            min_val: 0,
                            max_val: 0,
                        };
                        if (*fd).version >> 8 as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
                            e_0.len_encoding = E_HUFFMAN;
                            e_0.len_dat = NULL_0;
                        } else {
                            e_0.len_encoding = E_CONST_INT;
                            e_0.len_dat = NULL_0;
                        }
                        memset(
                            &raw mut st_0 as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            ::core::mem::size_of::<cram_stats>() as size_t,
                        );
                        if cram_stats_add(&raw mut st_0, 2 as int64_t) < 0 as ::core::ffi::c_int {
                            current_block = 9865445363914956224;
                            break;
                        }
                        cram_stats_encoding(fd, &raw mut st_0);
                        e_0.val_encoding = E_EXTERNAL;
                        e_0.val_dat = sk as *mut ::core::ffi::c_void;
                        c_0 = cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            &raw mut st_0,
                            E_BYTE_ARRAY,
                            &raw mut e_0 as *mut ::core::ffi::c_void,
                            (*fd).version,
                            &raw mut (*fd).vv,
                        );
                    }
                    105 | 73 | 102 => {
                        let mut e_1: cram_byte_array_len_encoder = cram_byte_array_len_encoder {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            val_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            len_codec: ::core::ptr::null_mut::<cram_codec>(),
                            val_codec: ::core::ptr::null_mut::<cram_codec>(),
                        };
                        let mut st_1: cram_stats = cram_stats {
                            freqs: [0; 1024],
                            h: ::core::ptr::null_mut::<kh_m_i2i_t>(),
                            nsamp: 0,
                            nvals: 0,
                            min_val: 0,
                            max_val: 0,
                        };
                        if (*fd).version >> 8 as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
                            e_1.len_encoding = E_HUFFMAN;
                            e_1.len_dat = NULL_0;
                        } else {
                            e_1.len_encoding = E_CONST_INT;
                            e_1.len_dat = NULL_0;
                        }
                        memset(
                            &raw mut st_1 as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            ::core::mem::size_of::<cram_stats>() as size_t,
                        );
                        if cram_stats_add(&raw mut st_1, 4 as int64_t) < 0 as ::core::ffi::c_int {
                            current_block = 9865445363914956224;
                            break;
                        }
                        cram_stats_encoding(fd, &raw mut st_1);
                        e_1.val_encoding = E_EXTERNAL;
                        e_1.val_dat = sk as *mut ::core::ffi::c_void;
                        c_0 = cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            &raw mut st_1,
                            E_BYTE_ARRAY,
                            &raw mut e_1 as *mut ::core::ffi::c_void,
                            (*fd).version,
                            &raw mut (*fd).vv,
                        );
                    }
                    66 => {
                        let mut e_2: cram_byte_array_len_encoder = cram_byte_array_len_encoder {
                            len_encoding: E_NULL,
                            val_encoding: E_NULL,
                            len_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            val_dat: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            len_codec: ::core::ptr::null_mut::<cram_codec>(),
                            val_codec: ::core::ptr::null_mut::<cram_codec>(),
                        };
                        e_2.len_encoding = (if (*fd).version >> 8 as ::core::ffi::c_int
                            >= 4 as ::core::ffi::c_int
                        {
                            E_VARINT_UNSIGNED as ::core::ffi::c_int
                        } else {
                            E_EXTERNAL as ::core::ffi::c_int
                        }) as cram_encoding;
                        e_2.len_dat = sk as *mut ::core::ffi::c_void;
                        e_2.val_encoding = E_EXTERNAL;
                        e_2.val_dat = sk as *mut ::core::ffi::c_void;
                        c_0 = cram_encoder_init(
                            E_BYTE_ARRAY_LEN,
                            ::core::ptr::null_mut::<cram_stats>(),
                            E_BYTE_ARRAY,
                            &raw mut e_2 as *mut ::core::ffi::c_void,
                            (*fd).version,
                            &raw mut (*fd).vv,
                        );
                    }
                    _ => {
                        hts_log(
                            HTS_LOG_ERROR,
                            b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                            b"Unsupported SAM aux type '%c'\0" as *const u8
                                as *const ::core::ffi::c_char,
                            *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                        );
                        c_0 = ::core::ptr::null_mut::<cram_codec>();
                    }
                }
                if c_0.is_null() {
                    current_block = 9865445363914956224;
                    break;
                }
                (*m).codec = c_0 as *mut cram_codec;
                pthread_mutex_lock(&raw mut (*fd).metrics_lock);
                (*m).m = if k_global != 0 {
                    *(*(*fd).tags_used).vals.offset(k_global as isize)
                } else {
                    ::core::ptr::null_mut::<cram_metrics>()
                };
                pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
            }
        }
        let mut tm: *mut cram_tag_map = *(*(*c).tags_used).vals.offset(k as isize);
        if tm.is_null() {
            current_block = 9865445363914956224;
            break;
        }
        let mut codec: *mut cram_codec = (*tm).codec as *mut cram_codec;
        if (*tm).codec.is_null() {
            current_block = 9865445363914956224;
            break;
        }
        match *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            65 | 67 | 99 => {
                if (aux_end.offset_from(aux) as ::core::ffi::c_long)
                    < (3 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as ::core::ffi::c_long
                {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk = cram_new_block(EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    (*(*codec).u.e_byte_array_len.val_codec).out = (*tm).blk;
                }
                aux = aux.offset(3 as ::core::ffi::c_int as isize);
                if block_append_char((*tm).blk, *aux) < 0 as ::core::ffi::c_int {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(1);
            }
            83 | 115 => {
                if (aux_end.offset_from(aux) as ::core::ffi::c_long)
                    < (3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as ::core::ffi::c_long
                {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk = cram_new_block(EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    (*(*codec).u.e_byte_array_len.val_codec).out = (*tm).blk;
                }
                aux = aux.offset(3 as ::core::ffi::c_int as isize);
                if block_append((*tm).blk, aux as *const ::core::ffi::c_void, 2 as size_t)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(2 as ::core::ffi::c_int as isize);
            }
            73 | 105 | 102 => {
                if (aux_end.offset_from(aux) as ::core::ffi::c_long)
                    < (3 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as ::core::ffi::c_long
                {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk = cram_new_block(EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    (*(*codec).u.e_byte_array_len.val_codec).out = (*tm).blk;
                }
                aux = aux.offset(3 as ::core::ffi::c_int as isize);
                if block_append((*tm).blk, aux as *const ::core::ffi::c_void, 4 as size_t)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(4 as ::core::ffi::c_int as isize);
            }
            100 => {
                if (aux_end.offset_from(aux) as ::core::ffi::c_long)
                    < (3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as ::core::ffi::c_long
                {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk = cram_new_block(EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    (*(*codec).u.e_byte_array_len.val_codec).out = (*tm).blk;
                }
                aux = aux.offset(3 as ::core::ffi::c_int as isize);
                if block_append((*tm).blk, aux as *const ::core::ffi::c_void, 8 as size_t)
                    < 0 as ::core::ffi::c_int
                {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(8 as ::core::ffi::c_int as isize);
            }
            90 | 72 => {
                if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 3 as ::core::ffi::c_long {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*tm).blk.is_null() {
                    (*tm).blk = cram_new_block(EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    (*codec).out = (*tm).blk;
                }
                let mut aux_s: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                aux = aux.offset(3 as ::core::ffi::c_int as isize);
                aux_s = aux;
                while aux < aux_end as *mut ::core::ffi::c_char && {
                    let fresh160 = aux;
                    aux = aux.offset(1);
                    *fresh160 as ::core::ffi::c_int != 0
                } {}
                if aux == aux_end as *mut ::core::ffi::c_char
                    && *aux.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    hts_log(
                        HTS_LOG_ERROR,
                        b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Unterminated %c%c:%c tag for read \"%s\"\0" as *const u8
                            as *const ::core::ffi::c_char,
                        *aux_s.offset(-(3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int,
                        *aux_s.offset(-(2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int,
                        *aux_s.offset(-(1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int,
                        (*b).data as *mut ::core::ffi::c_char,
                    );
                    current_block = 9865445363914956224;
                    break;
                } else if (*codec).encode.expect("non-null function pointer")(
                    s,
                    codec as *mut cram_codec,
                    aux_s,
                    aux.offset_from(aux_s) as ::core::ffi::c_long as ::core::ffi::c_int,
                ) < 0 as ::core::ffi::c_int
                {
                    current_block = 9865445363914956224;
                    break;
                }
            }
            66 => {
                if (aux_end.offset_from(aux) as ::core::ffi::c_long)
                    < (4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as ::core::ffi::c_long
                {
                    current_block = 9865445363914956224;
                    break;
                }
                let mut type_0: ::core::ffi::c_int =
                    *aux.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                let mut count: uint64_t = (*(aux as *mut ::core::ffi::c_uchar)
                    .offset(4 as ::core::ffi::c_int as isize)
                    as uint64_t)
                    << 0 as ::core::ffi::c_int
                    | (*(aux as *mut ::core::ffi::c_uchar).offset(5 as ::core::ffi::c_int as isize)
                        as uint64_t)
                        << 8 as ::core::ffi::c_int
                    | (*(aux as *mut ::core::ffi::c_uchar).offset(6 as ::core::ffi::c_int as isize)
                        as uint64_t)
                        << 16 as ::core::ffi::c_int
                    | (*(aux as *mut ::core::ffi::c_uchar).offset(7 as ::core::ffi::c_int as isize)
                        as uint64_t)
                        << 24 as ::core::ffi::c_int;
                let mut blen: uint64_t = 0;
                if (*tm).blk.is_null() {
                    (*tm).blk = cram_new_block(EXTERNAL, key_0);
                    if (*tm).blk.is_null() {
                        current_block = 9865445363914956224;
                        break;
                    }
                    if (*(*codec).u.e_byte_array_len.val_codec).codec as ::core::ffi::c_uint
                        == E_XDELTA as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        (*tm).blk2 = cram_new_block(EXTERNAL, key_0 + 128 as ::core::ffi::c_int);
                        if (*tm).blk2.is_null() {
                            current_block = 9865445363914956224;
                            break;
                        }
                        (*(*codec).u.e_byte_array_len.len_codec).out = (*tm).blk2;
                        (*(*(*codec).u.e_byte_array_len.val_codec)
                            .u
                            .e_xdelta
                            .sub_codec)
                            .out = (*tm).blk;
                    } else {
                        (*(*codec).u.e_byte_array_len.len_codec).out = (*tm).blk;
                        (*(*codec).u.e_byte_array_len.val_codec).out = (*tm).blk;
                    }
                }
                aux = aux.offset(3 as ::core::ffi::c_int as isize);
                match type_0 {
                    99 | 67 => {
                        blen = count;
                    }
                    115 | 83 => {
                        blen = (2 as uint64_t).wrapping_mul(count);
                    }
                    105 | 73 | 102 => {
                        blen = (4 as uint64_t).wrapping_mul(count);
                    }
                    _ => {
                        hts_log(
                            HTS_LOG_ERROR,
                            b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                            b"Unknown sub-type '%c' for aux type 'B'\0" as *const u8
                                as *const ::core::ffi::c_char,
                            type_0,
                        );
                        current_block = 9865445363914956224;
                        break;
                    }
                }
                blen = (blen as ::core::ffi::c_ulong).wrapping_add(5 as ::core::ffi::c_ulong)
                    as uint64_t as uint64_t;
                if (aux_end.offset_from(aux) as ::core::ffi::c_long as uint64_t) < blen
                    || blen > INT_MAX as uint64_t
                {
                    current_block = 9865445363914956224;
                    break;
                }
                if (*codec).encode.expect("non-null function pointer")(
                    s,
                    codec as *mut cram_codec,
                    aux,
                    blen as ::core::ffi::c_int,
                ) < 0 as ::core::ffi::c_int
                {
                    current_block = 9865445363914956224;
                    break;
                }
                aux = aux.offset(blen as isize);
            }
            _ => {
                hts_log(
                    HTS_LOG_ERROR,
                    b"cram_encode_aux\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Unknown aux type '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                    if (aux_end.offset_from(aux) as ::core::ffi::c_long) < 2 as ::core::ffi::c_long
                    {
                        '?' as i32
                    } else {
                        *aux.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    },
                );
                current_block = 9865445363914956224;
                break;
            }
        }
        (*(*tm).blk).m = (*tm).m;
    }
    match current_block {
        13391418783698890455 => {
            if !(block_append_char(td_b, 0 as ::core::ffi::c_char) < 0 as ::core::ffi::c_int) {
                key = ((*td_b).data as *mut ::core::ffi::c_char).offset(TD_blk_size as isize);
                k = kh_put_m_s2i((*(*c).comp_hdr).TD_hash, key as kh_cstr_t, &raw mut new);
                if !(new < 0 as ::core::ffi::c_int) {
                    if new == 0 as ::core::ffi::c_int {
                        (*td_b).byte = TD_blk_size as size_t;
                        current_block = 18340277188286182087;
                    } else {
                        let mut pooled_key: *mut ::core::ffi::c_char = string_ndup(
                            (*(*c).comp_hdr).TD_keys,
                            ((*td_b).data as *mut ::core::ffi::c_char).offset(TD_blk_size as isize),
                            (*td_b).byte.wrapping_sub(TD_blk_size as size_t),
                        );
                        if pooled_key.is_null() {
                            current_block = 9865445363914956224;
                        } else {
                            let ref mut fresh161 =
                                *(*(*(*c).comp_hdr).TD_hash).keys.offset(k as isize);
                            *fresh161 = pooled_key as kh_cstr_t;
                            *(*(*(*c).comp_hdr).TD_hash).vals.offset(k as isize) =
                                (*(*c).comp_hdr).nTL;
                            (*(*c).comp_hdr).nTL += 1;
                            current_block = 18340277188286182087;
                        }
                    }
                    match current_block {
                        9865445363914956224 => {}
                        _ => {
                            (*cr).TL = *(*(*(*c).comp_hdr).TD_hash).vals.offset(k as isize);
                            if !(cram_stats_add(
                                (*c).stats[DS_TL as ::core::ffi::c_int as usize],
                                (*cr).TL as int64_t,
                            ) < 0 as ::core::ffi::c_int)
                            {
                                if orig
                                    != (*b)
                                        .data
                                        .offset(
                                            ((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize,
                                        )
                                        .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
                                        .offset(
                                            ((*b).core.l_qseq + 1 as int32_t
                                                >> 1 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                        .offset((*b).core.l_qseq as isize)
                                        as *mut ::core::ffi::c_char
                                {
                                    free(orig as *mut ::core::ffi::c_void);
                                }
                                if !err.is_null() {
                                    *err = 0 as ::core::ffi::c_int;
                                }
                                return brg;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if orig
        != (*b)
            .data
            .offset(((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize)
            .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
            .offset(((*b).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int) as isize)
            .offset((*b).core.l_qseq as isize) as *mut ::core::ffi::c_char
    {
        free(orig as *mut ::core::ffi::c_void);
    }
    return ::core::ptr::null_mut::<sam_hrec_rg_t>();
}
#[no_mangle]
// original: cram_update_curr_slice (htslib/cram/cram_encode.c:3258)
pub unsafe extern "C" fn cram_update_curr_slice(
    mut c: *mut cram_container,
    mut version: ::core::ffi::c_int,
) {
    let mut s: *mut cram_slice = (*c).slice as *mut cram_slice;
    if (*c).multi_seq != 0 {
        (*(*s).hdr).ref_seq_id = -(2 as ::core::ffi::c_int) as int32_t;
        (*(*s).hdr).ref_seq_start = 0 as int64_t;
        (*(*s).hdr).ref_seq_span = 0 as int64_t;
    } else if (*c).curr_ref == -(1 as ::core::ffi::c_int) && version >= 0x301 as ::core::ffi::c_int
    {
        (*(*s).hdr).ref_seq_id = -(1 as ::core::ffi::c_int) as int32_t;
        (*(*s).hdr).ref_seq_start = 0 as int64_t;
        (*(*s).hdr).ref_seq_span = 0 as int64_t;
    } else {
        (*(*s).hdr).ref_seq_id = (*c).curr_ref as int32_t;
        (*(*s).hdr).ref_seq_start = (*c).first_base as int64_t;
        (*(*s).hdr).ref_seq_span =
            (if 0 as hts_pos_t > (*c).last_base - (*c).first_base + 1 as hts_pos_t {
                0 as hts_pos_t
            } else {
                (*c).last_base - (*c).first_base + 1 as hts_pos_t
            }) as int64_t;
    }
    (*(*s).hdr).num_records = (*c).curr_rec as int32_t;
    if (*c).curr_slice == 0 as ::core::ffi::c_int {
        if (*c).ref_seq_id != (*(*s).hdr).ref_seq_id {
            (*c).ref_seq_id = (*(*s).hdr).ref_seq_id;
        }
        (*c).ref_seq_start = (*c).first_base as int64_t;
    }
    (*c).curr_slice += 1;
}
// original: cram_next_container (htslib/cram/cram_encode.c:3295)
unsafe extern "C" fn cram_next_container(
    mut fd: *mut cram_fd,
    mut b: *mut bam_seq_t,
) -> *mut cram_container {
    let mut c: *mut cram_container = (*fd).ctr;
    let mut i: ::core::ffi::c_int = 0;
    if (*c).curr_ref == -(2 as ::core::ffi::c_int) {
        (*c).curr_ref = (*b).core.tid as ::core::ffi::c_int;
    }
    if !(*c).slice.is_null() {
        cram_update_curr_slice(c, (*fd).version);
    }
    if (*c).curr_slice == (*c).max_slice
        || (*b).core.tid != (*c).curr_ref as int32_t && (*c).multi_seq == 0
    {
        (*c).ref_seq_span = (*fd).last_base as int64_t - (*c).ref_seq_start + 1 as int64_t;
        hts_log(
            HTS_LOG_INFO,
            b"cram_next_container\0" as *const u8 as *const ::core::ffi::c_char,
            b"Flush container %d/%ld..%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).ref_seq_id,
            (*c).ref_seq_start,
            (*c).ref_seq_start + (*c).ref_seq_span - 1 as int64_t,
        );
        if -(1 as ::core::ffi::c_int) == cram_flush_container_mt(fd, c) {
            return ::core::ptr::null_mut::<cram_container>();
        }
        if (*fd).pool.is_null() {
            i = 0 as ::core::ffi::c_int;
            while i < (*c).max_slice {
                cram_free_slice(*(*c).slices.offset(i as isize) as *mut cram_slice);
                let ref mut fresh122 = *(*c).slices.offset(i as isize);
                *fresh122 = ::core::ptr::null_mut::<cram_slice>();
                i += 1;
            }
            (*c).slice = ::core::ptr::null_mut::<cram_slice>();
            (*c).curr_slice = 0 as ::core::ffi::c_int;
            cram_free_container(c);
        }
        (*fd).ctr = cram_new_container((*fd).seqs_per_slice, (*fd).slices_per_container);
        c = (*fd).ctr;
        if c.is_null() {
            return ::core::ptr::null_mut::<cram_container>();
        }
        pthread_mutex_lock(&raw mut (*fd).ref_lock);
        (*c).no_ref = (*fd).no_ref;
        (*c).embed_ref = (*fd).embed_ref;
        (*c).record_counter = (*fd).record_counter;
        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
        (*c).curr_ref = (*b).core.tid as ::core::ffi::c_int;
    }
    (*c).last_base = (*b).core.pos + 1 as hts_pos_t;
    (*c).first_base = (*c).last_base;
    (*c).last_pos = (*c).first_base as int64_t;
    let ref mut fresh123 = *(*c).slices.offset((*c).curr_slice as isize);
    *fresh123 = cram_new_slice(MAPPED_SLICE, (*c).max_rec) as *mut cram_slice;
    (*c).slice = *fresh123;
    if (*c).slice.is_null() {
        return ::core::ptr::null_mut::<cram_container>();
    }
    if (*c).multi_seq != 0 {
        (*(*(*c).slice).hdr).ref_seq_id = -(2 as ::core::ffi::c_int) as int32_t;
        (*(*(*c).slice).hdr).ref_seq_start = 0 as int64_t;
        (*(*c).slice).last_apos = 1 as int64_t;
    } else {
        (*(*(*c).slice).hdr).ref_seq_id = (*b).core.tid;
        (*(*(*c).slice).hdr).ref_seq_start = ((*b).core.pos + 1 as hts_pos_t) as int64_t;
        (*(*c).slice).last_apos = ((*b).core.pos + 1 as hts_pos_t) as int64_t;
    }
    (*c).curr_rec = 0 as ::core::ffi::c_int;
    (*c).s_num_bases = 0 as uint64_t;
    (*c).n_mapped = 0 as uint32_t;
    (*c).qs_seq_orient = if (*fd).version >> 8 as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    return c;
}
// original: process_one_read (htslib/cram/cram_encode.c:3385)
unsafe extern "C" fn process_one_read(
    mut fd: *mut cram_fd,
    mut c: *mut cram_container,
    mut s: *mut cram_slice,
    mut cr: *mut cram_record,
    mut b: *mut bam_seq_t,
    mut rnum: ::core::ffi::c_int,
    mut MD: *mut kstring_t,
    mut embed_ref: ::core::ffi::c_int,
    mut no_ref: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut l: ::core::ffi::c_int = 0;
    let mut err: ::core::ffi::c_int = 0;
    let mut brg: *mut sam_hrec_rg_t = ::core::ptr::null_mut::<sam_hrec_rg_t>();
    let mut current_block: u64;
    let mut i: ::core::ffi::c_int = 0;
    let mut fake_qual: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut NM: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ref_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut seq: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut qual: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut verbatim_NM: ::core::ffi::c_int = (*fd).store_nm;
    let mut verbatim_MD: ::core::ffi::c_int = (*fd).store_md;
    (*cr).flags = (*b).core.flag as int32_t;
    (*cr).len = (*b).core.l_qseq;
    let mut md: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    md = bam_aux_get(b, b"MD\0" as *const u8 as *const ::core::ffi::c_char);
    if md.is_null() {
        MD = ::core::ptr::null_mut::<kstring_t>();
    } else {
        (*MD).l = 0 as size_t;
    }
    let mut cf_tag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if embed_ref == 2 as ::core::ffi::c_int {
        cf_tag = if !MD.is_null() {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
        cf_tag |= if !bam_aux_get(b, b"NM\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
            0 as ::core::ffi::c_int
        } else {
            2 as ::core::ffi::c_int
        };
    }
    ref_0 = if !(*c).ref_0.is_null() {
        (*c).ref_0
            .offset(-(((*c).ref_start - 1 as hts_pos_t) as isize))
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*cr).ref_id = (*b).core.tid;
    if !(cram_stats_add(
        (*c).stats[DS_RI as ::core::ffi::c_int as usize],
        (*cr).ref_id as int64_t,
    ) < 0 as ::core::ffi::c_int)
    {
        if !(cram_stats_add(
            (*c).stats[DS_BF as ::core::ffi::c_int as usize],
            (*fd).cram_flag_swap[((*cr).flags & 0xfff as int32_t) as usize] as int64_t,
        ) < 0 as ::core::ffi::c_int)
        {
            if no_ref == 0 || (*fd).version >> 8 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int {
                (*cr).cram_flags = ((*cr).cram_flags as ::core::ffi::c_int
                    | CRAM_FLAG_PRESERVE_QUAL_SCORES) as int32_t;
            }
            if (*cr).len <= 0 as int32_t
                && (*fd).version >> 8 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int
            {
                (*cr).cram_flags =
                    ((*cr).cram_flags as ::core::ffi::c_int | CRAM_FLAG_NO_SEQ) as int32_t;
            }
            (*c).num_bases = ((*c).num_bases as ::core::ffi::c_long
                + (*cr).len as ::core::ffi::c_long) as int64_t;
            (*cr).apos = ((*b).core.pos + 1 as hts_pos_t) as int64_t;
            if !((*cr).apos < 0 as int64_t || (*cr).apos > INT64_MAX as int64_t / 2 as int64_t) {
                if (*c).pos_sorted != 0 {
                    if (*cr).apos < (*s).last_apos && (*fd).ap_delta == 0 {
                        (*c).pos_sorted = 0 as ::core::ffi::c_int;
                        current_block = 2719512138335094285;
                    } else if cram_stats_add(
                        (*c).stats[DS_AP as ::core::ffi::c_int as usize],
                        (*cr).apos - (*s).last_apos,
                    ) < 0 as ::core::ffi::c_int
                    {
                        current_block = 4645371139350450943;
                    } else {
                        (*s).last_apos = (*cr).apos;
                        current_block = 2719512138335094285;
                    }
                } else {
                    current_block = 2719512138335094285;
                }
                match current_block {
                    4645371139350450943 => {}
                    _ => {
                        (*c).max_apos = ((*c).max_apos as ::core::ffi::c_long
                            + (((*cr).apos > (*c).max_apos) as ::core::ffi::c_int as int64_t
                                * ((*cr).apos - (*c).max_apos))
                                as ::core::ffi::c_long)
                            as int64_t;
                        (*cr).seq = (*(*s).seqs_blk).byte as uint32_t;
                        (*cr).qual = (*(*s).qual_blk).byte as uint32_t;
                        if !(block_grow((*s).seqs_blk, ((*cr).len + 1 as int32_t) as size_t)
                            < 0 as ::core::ffi::c_int)
                        {
                            if !(block_grow((*s).qual_blk, (*cr).len as size_t)
                                < 0 as ::core::ffi::c_int)
                            {
                                cp = (*(*s).seqs_blk).data.offset((*(*s).seqs_blk).byte as isize)
                                    as *mut ::core::ffi::c_uchar
                                    as *mut ::core::ffi::c_char;
                                seq = cp;
                                *seq = 0 as ::core::ffi::c_char;
                                nibble2base(
                                    (*b).data
                                        .offset(
                                            ((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize,
                                        )
                                        .offset((*b).core.l_qname as ::core::ffi::c_int as isize),
                                    cp,
                                    (*cr).len as ::core::ffi::c_int,
                                );
                                (*(*s).seqs_blk).byte =
                                    ((*(*s).seqs_blk).byte as ::core::ffi::c_ulong)
                                        .wrapping_add((*cr).len as ::core::ffi::c_ulong)
                                        as size_t as size_t;
                                cp = (*b)
                                    .data
                                    .offset(((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize)
                                    .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
                                    .offset(
                                        ((*b).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int)
                                            as isize,
                                    )
                                    as *mut ::core::ffi::c_char;
                                qual = cp;
                                if (*cr).flags & BAM_FUNMAP as int32_t == 0 {
                                    let mut cig_to: *mut uint32_t =
                                        ::core::ptr::null_mut::<uint32_t>();
                                    let mut cig_from: *mut uint32_t =
                                        ::core::ptr::null_mut::<uint32_t>();
                                    let mut apos: int64_t = (*cr).apos - 1 as int64_t;
                                    let mut spos: int64_t = 0 as int64_t;
                                    let mut MD_last: int64_t = apos;
                                    if apos < 0 as int64_t {
                                        hts_log(
                                            HTS_LOG_ERROR,
                                            b"process_one_read\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"Mapped read with position <= 0 is disallowed\0"
                                                as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        return -(1 as ::core::ffi::c_int);
                                    }
                                    (*cr).cigar = (*s).ncigar;
                                    (*cr).ncigar = (*b).core.n_cigar as int32_t;
                                    while (*cr).cigar.wrapping_add((*cr).ncigar as uint32_t)
                                        >= (*s).cigar_alloc
                                    {
                                        (*s).cigar_alloc = if (*s).cigar_alloc != 0 {
                                            (*s).cigar_alloc.wrapping_mul(2 as uint32_t)
                                        } else {
                                            1024 as uint32_t
                                        };
                                        (*s).cigar = realloc(
                                            (*s).cigar as *mut ::core::ffi::c_void,
                                            ((*s).cigar_alloc as size_t).wrapping_mul(
                                                ::core::mem::size_of::<uint32_t>() as size_t,
                                            ),
                                        )
                                            as *mut uint32_t;
                                        if (*s).cigar.is_null() {
                                            return -(1 as ::core::ffi::c_int);
                                        }
                                    }
                                    cig_to = (*s).cigar;
                                    cig_from = (*b)
                                        .data
                                        .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
                                        as *mut uint32_t;
                                    (*cr).feature = 0 as uint32_t;
                                    (*cr).nfeature = 0 as uint32_t;
                                    i = 0 as ::core::ffi::c_int;
                                    's_225: loop {
                                        if !((i as int32_t) < (*cr).ncigar) {
                                            current_block = 7848525887314104415;
                                            break;
                                        }
                                        let mut cig_op: cigar_op = (*cig_from.offset(i as isize)
                                            & BAM_CIGAR_MASK as uint32_t)
                                            as cigar_op;
                                        let mut cig_len: uint32_t =
                                            *cig_from.offset(i as isize) >> BAM_CIGAR_SHIFT;
                                        *cig_to.offset(i as isize) = *cig_from.offset(i as isize);
                                        match cig_op as ::core::ffi::c_uint {
                                            0 | 7 | 8 => {
                                                l = 0 as ::core::ffi::c_int;
                                                if no_ref == 0 && (*cr).len != 0 {
                                                    let mut end: ::core::ffi::c_int = (if cig_len
                                                        as int64_t
                                                        + apos
                                                        < (*c).ref_end
                                                    {
                                                        cig_len as hts_pos_t
                                                    } else {
                                                        (*c).ref_end - apos as hts_pos_t
                                                    })
                                                        as ::core::ffi::c_int;
                                                    let mut sp: *mut ::core::ffi::c_char = seq
                                                        .offset(spos as isize)
                                                        as *mut ::core::ffi::c_char;
                                                    let mut rp: *mut ::core::ffi::c_char = ref_0
                                                        .offset(apos as isize)
                                                        as *mut ::core::ffi::c_char;
                                                    let mut qp: *mut ::core::ffi::c_char = qual
                                                        .offset(spos as isize)
                                                        as *mut ::core::ffi::c_char;
                                                    if end as int32_t > (*cr).len {
                                                        hts_log(
                                                            HTS_LOG_ERROR,
                                                            b"process_one_read\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            b"CIGAR and query sequence are of different length\0"
                                                                as *const u8 as *const ::core::ffi::c_char,
                                                        );
                                                        return -(1 as ::core::ffi::c_int);
                                                    }
                                                    l = 0 as ::core::ffi::c_int;
                                                    while l < end {
                                                        if *rp.offset(l as isize)
                                                            as ::core::ffi::c_int
                                                            == 'N' as i32
                                                            && *sp.offset(l as isize)
                                                                as ::core::ffi::c_int
                                                                == 'N' as i32
                                                        {
                                                            verbatim_MD = 1 as ::core::ffi::c_int;
                                                            verbatim_NM = verbatim_MD;
                                                        }
                                                        if *rp.offset(l as isize)
                                                            as ::core::ffi::c_int
                                                            != *sp.offset(l as isize)
                                                                as ::core::ffi::c_int
                                                        {
                                                            if !MD.is_null() && !ref_0.is_null() {
                                                                if kputuw(
                                                                    (apos + l as int64_t - MD_last)
                                                                        as ::core::ffi::c_uint,
                                                                    MD,
                                                                ) < 0 as ::core::ffi::c_int
                                                                {
                                                                    current_block =
                                                                        4645371139350450943;
                                                                    break 's_225;
                                                                }
                                                                if kputc(
                                                                    *rp.offset(l as isize)
                                                                        as ::core::ffi::c_int,
                                                                    MD,
                                                                ) < 0 as ::core::ffi::c_int
                                                                {
                                                                    current_block =
                                                                        4645371139350450943;
                                                                    break 's_225;
                                                                }
                                                                MD_last = apos
                                                                    + l as int64_t
                                                                    + 1 as int64_t;
                                                            }
                                                            NM += 1;
                                                            if *sp.offset(l as isize) == 0 {
                                                                break;
                                                            }
                                                            if 0 as ::core::ffi::c_int != 0
                                                                && (*fd).version
                                                                    >> 8 as ::core::ffi::c_int
                                                                    >= 3 as ::core::ffi::c_int
                                                            {
                                                                let mut nl: ::core::ffi::c_int = l;
                                                                let mut max_end: ::core::ffi::c_int = nl;
                                                                let mut max_score: ::core::ffi::c_int = 0
                                                                    as ::core::ffi::c_int;
                                                                let mut score: ::core::ffi::c_int =
                                                                    0 as ::core::ffi::c_int;
                                                                while nl < end {
                                                                    if *rp.offset(nl as isize)
                                                                        as ::core::ffi::c_int
                                                                        != *sp.offset(nl as isize)
                                                                            as ::core::ffi::c_int
                                                                    {
                                                                        score +=
                                                                            3 as ::core::ffi::c_int;
                                                                        if max_score < score {
                                                                            max_score = score;
                                                                            max_end = nl;
                                                                        }
                                                                    } else {
                                                                        score -= 1;
                                                                        if score < -(2 as ::core::ffi::c_int)
                                                                            || max_score - score > 7 as ::core::ffi::c_int
                                                                        {
                                                                            break;
                                                                        }
                                                                    }
                                                                    nl += 1;
                                                                }
                                                                if max_score
                                                                    > 20 as ::core::ffi::c_int
                                                                {
                                                                    cram_add_bases(
                                                                        fd,
                                                                        c,
                                                                        s,
                                                                        cr,
                                                                        (spos + l as int64_t) as ::core::ffi::c_int,
                                                                        max_end - l,
                                                                        seq.offset((spos + l as int64_t) as isize)
                                                                            as *mut ::core::ffi::c_char,
                                                                    );
                                                                    l = max_end
                                                                        - 1 as ::core::ffi::c_int;
                                                                } else {
                                                                    while l < nl {
                                                                        if *rp.offset(l as isize) as ::core::ffi::c_int
                                                                            != *sp.offset(l as isize) as ::core::ffi::c_int
                                                                        {
                                                                            cram_add_substitution(
                                                                                fd,
                                                                                c,
                                                                                s,
                                                                                cr,
                                                                                (spos + l as int64_t) as ::core::ffi::c_int,
                                                                                *sp.offset(l as isize),
                                                                                *qp.offset(l as isize),
                                                                                *rp.offset(l as isize),
                                                                            );
                                                                        }
                                                                        l += 1;
                                                                    }
                                                                    l -= 1;
                                                                }
                                                            } else if cram_add_substitution(
                                                                fd,
                                                                c,
                                                                s,
                                                                cr,
                                                                (spos + l as int64_t)
                                                                    as ::core::ffi::c_int,
                                                                *sp.offset(l as isize),
                                                                *qp.offset(l as isize),
                                                                *rp.offset(l as isize),
                                                            ) != 0
                                                            {
                                                                return -(1 as ::core::ffi::c_int);
                                                            }
                                                        }
                                                        l += 1;
                                                    }
                                                    spos = (spos as ::core::ffi::c_long
                                                        + l as ::core::ffi::c_long)
                                                        as int64_t;
                                                    apos = (apos as ::core::ffi::c_long
                                                        + l as ::core::ffi::c_long)
                                                        as int64_t;
                                                }
                                                if (l as uint32_t) < cig_len && (*cr).len != 0 {
                                                    if no_ref != 0 {
                                                        if (*fd).version >> 8 as ::core::ffi::c_int
                                                            == 3 as ::core::ffi::c_int
                                                        {
                                                            if cram_add_bases(
                                                                fd,
                                                                c,
                                                                s,
                                                                cr,
                                                                spos as ::core::ffi::c_int,
                                                                cig_len.wrapping_sub(l as uint32_t)
                                                                    as ::core::ffi::c_int,
                                                                seq.offset(spos as isize)
                                                                    as *mut ::core::ffi::c_char,
                                                            ) != 0
                                                            {
                                                                return -(1 as ::core::ffi::c_int);
                                                            }
                                                            spos = (spos as ::core::ffi::c_long
                                                                + cig_len
                                                                    .wrapping_sub(l as uint32_t)
                                                                    as ::core::ffi::c_long)
                                                                as int64_t;
                                                        } else {
                                                            while (l as uint32_t) < cig_len
                                                                && *seq.offset(spos as isize)
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                            {
                                                                if cram_add_base(
                                                                    fd,
                                                                    c,
                                                                    s,
                                                                    cr,
                                                                    spos as ::core::ffi::c_int,
                                                                    *seq.offset(spos as isize),
                                                                    *qual.offset(spos as isize),
                                                                ) != 0
                                                                {
                                                                    return -(1
                                                                        as ::core::ffi::c_int);
                                                                }
                                                                l += 1;
                                                                spos += 1;
                                                            }
                                                        }
                                                    } else {
                                                        verbatim_MD = 1 as ::core::ffi::c_int;
                                                        verbatim_NM = verbatim_MD;
                                                        while (l as uint32_t) < cig_len
                                                            && *seq.offset(spos as isize)
                                                                as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            if cram_add_base(
                                                                fd,
                                                                c,
                                                                s,
                                                                cr,
                                                                spos as ::core::ffi::c_int,
                                                                *seq.offset(spos as isize),
                                                                *qual.offset(spos as isize),
                                                            ) != 0
                                                            {
                                                                return -(1 as ::core::ffi::c_int);
                                                            }
                                                            l += 1;
                                                            spos += 1;
                                                        }
                                                    }
                                                    apos = (apos as ::core::ffi::c_long
                                                        + cig_len as ::core::ffi::c_long)
                                                        as int64_t;
                                                } else if (*cr).len == 0 {
                                                    verbatim_MD = 1 as ::core::ffi::c_int;
                                                    verbatim_NM = verbatim_MD;
                                                    apos = (apos as ::core::ffi::c_long
                                                        + cig_len as ::core::ffi::c_long)
                                                        as int64_t;
                                                    spos = (spos as ::core::ffi::c_long
                                                        + cig_len as ::core::ffi::c_long)
                                                        as int64_t;
                                                }
                                            }
                                            2 => {
                                                if !MD.is_null() && !ref_0.is_null() {
                                                    if kputuw(
                                                        (apos - MD_last) as ::core::ffi::c_uint,
                                                        MD,
                                                    ) < 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 4645371139350450943;
                                                        break;
                                                    }
                                                    if apos < (*c).ref_end {
                                                        if kputc_('^' as i32, MD)
                                                            < 0 as ::core::ffi::c_int
                                                        {
                                                            current_block = 4645371139350450943;
                                                            break;
                                                        }
                                                        if kputsn(
                                                            ref_0.offset(apos as isize)
                                                                as *mut ::core::ffi::c_char,
                                                            (if ((*c).ref_end - apos as hts_pos_t)
                                                                < cig_len as hts_pos_t
                                                            {
                                                                (*c).ref_end - apos as hts_pos_t
                                                            } else {
                                                                cig_len as hts_pos_t
                                                            })
                                                                as size_t,
                                                            MD,
                                                        ) < 0 as ::core::ffi::c_int
                                                        {
                                                            current_block = 4645371139350450943;
                                                            break;
                                                        }
                                                    }
                                                }
                                                NM = (NM as ::core::ffi::c_uint)
                                                    .wrapping_add(cig_len as ::core::ffi::c_uint)
                                                    as ::core::ffi::c_int
                                                    as ::core::ffi::c_int;
                                                if cram_add_deletion(
                                                    c,
                                                    s,
                                                    cr,
                                                    spos as ::core::ffi::c_int,
                                                    cig_len as ::core::ffi::c_int,
                                                    seq.offset(spos as isize)
                                                        as *mut ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                                apos = (apos as ::core::ffi::c_long
                                                    + cig_len as ::core::ffi::c_long)
                                                    as int64_t;
                                                MD_last = apos;
                                            }
                                            3 => {
                                                if cram_add_skip(
                                                    c,
                                                    s,
                                                    cr,
                                                    spos as ::core::ffi::c_int,
                                                    cig_len as ::core::ffi::c_int,
                                                    seq.offset(spos as isize)
                                                        as *mut ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                                apos = (apos as ::core::ffi::c_long
                                                    + cig_len as ::core::ffi::c_long)
                                                    as int64_t;
                                                MD_last = (MD_last as ::core::ffi::c_long
                                                    + cig_len as ::core::ffi::c_long)
                                                    as int64_t;
                                            }
                                            1 => {
                                                if cram_add_insertion(
                                                    c,
                                                    s,
                                                    cr,
                                                    spos as ::core::ffi::c_int,
                                                    cig_len as ::core::ffi::c_int,
                                                    if (*cr).len != 0 {
                                                        seq.offset(spos as isize)
                                                            as *mut ::core::ffi::c_char
                                                    } else {
                                                        ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                        )
                                                    },
                                                ) != 0
                                                {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                                if no_ref != 0 && (*cr).len != 0 {
                                                    l = 0 as ::core::ffi::c_int;
                                                    while (l as uint32_t) < cig_len {
                                                        cram_add_quality(
                                                            fd,
                                                            c,
                                                            s,
                                                            cr,
                                                            spos as ::core::ffi::c_int,
                                                            *qual.offset(spos as isize),
                                                        );
                                                        l += 1;
                                                        spos += 1;
                                                    }
                                                } else {
                                                    spos = (spos as ::core::ffi::c_long
                                                        + cig_len as ::core::ffi::c_long)
                                                        as int64_t;
                                                }
                                                NM = (NM as ::core::ffi::c_uint)
                                                    .wrapping_add(cig_len as ::core::ffi::c_uint)
                                                    as ::core::ffi::c_int
                                                    as ::core::ffi::c_int;
                                            }
                                            4 => {
                                                if cram_add_softclip(
                                                    c,
                                                    s,
                                                    cr,
                                                    spos as ::core::ffi::c_int,
                                                    cig_len as ::core::ffi::c_int,
                                                    if (*cr).len != 0 {
                                                        seq.offset(spos as isize)
                                                            as *mut ::core::ffi::c_char
                                                    } else {
                                                        ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                        )
                                                    },
                                                    (*fd).version,
                                                ) != 0
                                                {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                                if no_ref != 0
                                                    && (*cr).cram_flags
                                                        & CRAM_FLAG_PRESERVE_QUAL_SCORES as int32_t
                                                        == 0
                                                {
                                                    if (*cr).len != 0 {
                                                        l = 0 as ::core::ffi::c_int;
                                                        while (l as uint32_t) < cig_len {
                                                            cram_add_quality(
                                                                fd,
                                                                c,
                                                                s,
                                                                cr,
                                                                spos as ::core::ffi::c_int,
                                                                *qual.offset(spos as isize),
                                                            );
                                                            l += 1;
                                                            spos += 1;
                                                        }
                                                    } else {
                                                        l = 0 as ::core::ffi::c_int;
                                                        while (l as uint32_t) < cig_len {
                                                            cram_add_quality(
                                                                fd,
                                                                c,
                                                                s,
                                                                cr,
                                                                spos as ::core::ffi::c_int,
                                                                -(1 as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_char,
                                                            );
                                                            l += 1;
                                                            spos += 1;
                                                        }
                                                    }
                                                } else {
                                                    spos = (spos as ::core::ffi::c_long
                                                        + cig_len as ::core::ffi::c_long)
                                                        as int64_t;
                                                }
                                            }
                                            5 => {
                                                if cram_add_hardclip(
                                                    c,
                                                    s,
                                                    cr,
                                                    spos as ::core::ffi::c_int,
                                                    cig_len as ::core::ffi::c_int,
                                                    seq.offset(spos as isize)
                                                        as *mut ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                            }
                                            6 => {
                                                if cram_add_pad(
                                                    c,
                                                    s,
                                                    cr,
                                                    spos as ::core::ffi::c_int,
                                                    cig_len as ::core::ffi::c_int,
                                                    seq.offset(spos as isize)
                                                        as *mut ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    return -(1 as ::core::ffi::c_int);
                                                }
                                            }
                                            _ => {
                                                hts_log(
                                                    HTS_LOG_ERROR,
                                                    b"process_one_read\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"Unknown CIGAR op code %d\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    cig_op as ::core::ffi::c_uint,
                                                );
                                                return -(1 as ::core::ffi::c_int);
                                            }
                                        }
                                        i += 1;
                                    }
                                    match current_block {
                                        4645371139350450943 => {}
                                        _ => {
                                            if (*cr).len != 0 && spos != (*cr).len as int64_t {
                                                hts_log(
                                                    HTS_LOG_ERROR,
                                                    b"process_one_read\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"CIGAR and query sequence are of different length\0"
                                                        as *const u8 as *const ::core::ffi::c_char,
                                                );
                                                return -(1 as ::core::ffi::c_int);
                                            }
                                            fake_qual = spos as ::core::ffi::c_int;
                                            (*cr).aend = if no_ref != 0 {
                                                apos
                                            } else if apos
                                                < (if 0 as hts_pos_t > (*c).ref_end {
                                                    0 as hts_pos_t
                                                } else {
                                                    (*c).ref_end
                                                })
                                            {
                                                apos
                                            } else if 0 as hts_pos_t > (*c).ref_end {
                                                0 as int64_t
                                            } else {
                                                (*c).ref_end as int64_t
                                            };
                                            if cram_stats_add(
                                                (*c).stats[DS_FN as ::core::ffi::c_int as usize],
                                                (*cr).nfeature as int64_t,
                                            ) < 0 as ::core::ffi::c_int
                                            {
                                                current_block = 4645371139350450943;
                                            } else if !MD.is_null() && !ref_0.is_null() {
                                                if kputuw(
                                                    (apos - MD_last) as ::core::ffi::c_uint,
                                                    MD,
                                                ) < 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 4645371139350450943;
                                                } else {
                                                    current_block = 9235179519944561532;
                                                }
                                            } else {
                                                current_block = 9235179519944561532;
                                            }
                                        }
                                    }
                                } else {
                                    (*cr).cram_flags = ((*cr).cram_flags as ::core::ffi::c_int
                                        | CRAM_FLAG_PRESERVE_QUAL_SCORES)
                                        as int32_t;
                                    (*cr).cigar = 0 as uint32_t;
                                    (*cr).ncigar = 0 as ::core::ffi::c_int as int32_t;
                                    (*cr).nfeature = 0 as uint32_t;
                                    (*cr).aend = if (*cr).apos < (*c).ref_end {
                                        (*cr).apos
                                    } else {
                                        (*c).ref_end as int64_t
                                    };
                                    i = 0 as ::core::ffi::c_int;
                                    loop {
                                        if !((i as int32_t) < (*cr).len) {
                                            current_block = 2408932541243239002;
                                            break;
                                        }
                                        if cram_stats_add(
                                            (*c).stats[DS_BA as ::core::ffi::c_int as usize],
                                            *seq.offset(i as isize) as int64_t,
                                        ) < 0 as ::core::ffi::c_int
                                        {
                                            current_block = 4645371139350450943;
                                            break;
                                        }
                                        i += 1;
                                    }
                                    match current_block {
                                        4645371139350450943 => {}
                                        _ => {
                                            fake_qual = 0 as ::core::ffi::c_int;
                                            current_block = 9235179519944561532;
                                        }
                                    }
                                }
                                match current_block {
                                    4645371139350450943 => {}
                                    _ => {
                                        (*cr).ntags = 0 as ::core::ffi::c_int as int32_t;
                                        err = 0 as ::core::ffi::c_int;
                                        brg = cram_encode_aux(
                                            fd,
                                            b,
                                            c,
                                            s,
                                            cr,
                                            verbatim_NM,
                                            verbatim_MD,
                                            NM,
                                            MD,
                                            cf_tag,
                                            no_ref,
                                            &raw mut err,
                                        );
                                        if !(err != 0) {
                                            if !brg.is_null() {
                                                (*cr).rg = (*brg).id as int32_t;
                                                current_block = 11322929247169729670;
                                            } else if (*fd).version >> 8 as ::core::ffi::c_int
                                                == 1 as ::core::ffi::c_int
                                            {
                                                let mut brg_0: *mut sam_hrec_rg_t =
                                                    sam_hrecs_find_rg(
                                                        (*(*fd).header).hrecs,
                                                        b"UNKNOWN\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                if brg_0.is_null() {
                                                    current_block = 4645371139350450943;
                                                } else {
                                                    (*cr).rg = (*brg_0).id as int32_t;
                                                    current_block = 11322929247169729670;
                                                }
                                            } else {
                                                (*cr).rg = -(1 as ::core::ffi::c_int) as int32_t;
                                                current_block = 11322929247169729670;
                                            }
                                            match current_block {
                                                4645371139350450943 => {}
                                                _ => {
                                                    if !(cram_stats_add(
                                                        (*c).stats
                                                            [DS_RG as ::core::ffi::c_int as usize],
                                                        (*cr).rg as int64_t,
                                                    ) < 0 as ::core::ffi::c_int)
                                                    {
                                                        if (*cr).cram_flags
                                                            & CRAM_FLAG_PRESERVE_QUAL_SCORES
                                                                as int32_t
                                                            != 0
                                                        {
                                                            if (*cr).len == 0 as int32_t {
                                                                (*cr).len = fake_qual as int32_t;
                                                                if block_grow(
                                                                    (*s).qual_blk,
                                                                    (*cr).len as size_t,
                                                                ) < 0 as ::core::ffi::c_int
                                                                {
                                                                    current_block =
                                                                        4645371139350450943;
                                                                } else {
                                                                    cp = (*(*s).qual_blk)
                                                                        .data
                                                                        .offset(
                                                                            (*(*s).qual_blk).byte
                                                                                as isize,
                                                                        )
                                                                        as *mut ::core::ffi::c_uchar
                                                                        as *mut ::core::ffi::c_char;
                                                                    memset(
                                                                        cp as *mut ::core::ffi::c_void,
                                                                        255 as ::core::ffi::c_int,
                                                                        (*cr).len as size_t,
                                                                    );
                                                                    current_block =
                                                                        16979802930995685524;
                                                                }
                                                            } else if block_grow(
                                                                (*s).qual_blk,
                                                                (*cr).len as size_t,
                                                            ) < 0 as ::core::ffi::c_int
                                                            {
                                                                current_block = 4645371139350450943;
                                                            } else {
                                                                cp = (*(*s).qual_blk).data.offset(
                                                                    (*(*s).qual_blk).byte as isize,
                                                                )
                                                                    as *mut ::core::ffi::c_uchar
                                                                    as *mut ::core::ffi::c_char;
                                                                let mut from: *mut ::core::ffi::c_char = (*b)
                                                                    .data
                                                                    .offset(
                                                                        ((*b).core.n_cigar << 2 as ::core::ffi::c_int) as isize,
                                                                    )
                                                                    .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
                                                                    .offset(
                                                                        ((*b).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int)
                                                                            as isize,
                                                                    )
                                                                    .offset(0 as ::core::ffi::c_int as isize) as *mut uint8_t
                                                                    as *mut ::core::ffi::c_char;
                                                                let mut to: *mut ::core::ffi::c_char = cp
                                                                    .offset(0 as ::core::ffi::c_int as isize)
                                                                    as *mut ::core::ffi::c_char;
                                                                memcpy(
                                                                    to as *mut ::core::ffi::c_void,
                                                                    from as *const ::core::ffi::c_void,
                                                                    (*cr).len as size_t,
                                                                );
                                                                if (*c).qs_seq_orient == 0 {
                                                                    if (*cr).flags
                                                                        & BAM_FREVERSE as int32_t
                                                                        != 0
                                                                    {
                                                                        let mut i_0: ::core::ffi::c_int = 0;
                                                                        let mut j: ::core::ffi::c_int = 0;
                                                                        i_0 =
                                                                            0 as ::core::ffi::c_int;
                                                                        j = ((*cr).len
                                                                            - 1 as int32_t)
                                                                            as ::core::ffi::c_int;
                                                                        while i_0 < j {
                                                                            let mut c_0: ::core::ffi::c_uchar = 0;
                                                                            c_0 = *to.offset(i_0 as isize) as ::core::ffi::c_uchar;
                                                                            *to.offset(
                                                                                i_0 as isize,
                                                                            ) = *to
                                                                                .offset(j as isize);
                                                                            *to.offset(j as isize) = c_0 as ::core::ffi::c_char;
                                                                            i_0 += 1;
                                                                            j -= 1;
                                                                        }
                                                                    }
                                                                }
                                                                current_block =
                                                                    16979802930995685524;
                                                            }
                                                            match current_block {
                                                                4645371139350450943 => {}
                                                                _ => {
                                                                    (*(*s).qual_blk).byte = ((*(*s).qual_blk).byte
                                                                        as ::core::ffi::c_ulong)
                                                                        .wrapping_add((*cr).len as ::core::ffi::c_ulong) as size_t
                                                                        as size_t;
                                                                    current_block =
                                                                        16718638665978159145;
                                                                }
                                                            }
                                                        } else {
                                                            if (*cr).len == 0 as int32_t {
                                                                (*cr).len = (if fake_qual
                                                                    >= 0 as ::core::ffi::c_int
                                                                {
                                                                    fake_qual as int64_t
                                                                } else {
                                                                    (*cr).aend - (*cr).apos
                                                                        + 1 as int64_t
                                                                })
                                                                    as int32_t;
                                                            }
                                                            current_block = 16718638665978159145;
                                                        }
                                                        match current_block {
                                                            4645371139350450943 => {}
                                                            _ => {
                                                                if !(cram_stats_add(
                                                                    (*c).stats[DS_RL
                                                                        as ::core::ffi::c_int
                                                                        as usize],
                                                                    (*cr).len as int64_t,
                                                                ) < 0 as ::core::ffi::c_int)
                                                                {
                                                                    let mut new: ::core::ffi::c_int = 0;
                                                                    let mut k: khint_t = 0;
                                                                    let mut sec: ::core::ffi::c_int = if (*cr).flags
                                                                        & BAM_FSECONDARY as int32_t != 0
                                                                    {
                                                                        1 as ::core::ffi::c_int
                                                                    } else {
                                                                        0 as ::core::ffi::c_int
                                                                    };
                                                                    if (*cr).flags
                                                                        & BAM_FPAIRED as int32_t
                                                                        != 0
                                                                    {
                                                                        k = kh_put_m_s2i(
                                                                            (*s).pair[sec as usize],
                                                                            (*b).data as *mut ::core::ffi::c_char as kh_cstr_t,
                                                                            &raw mut new,
                                                                        );
                                                                        if -(1
                                                                            as ::core::ffi::c_int)
                                                                            == new
                                                                        {
                                                                            return -(1 as ::core::ffi::c_int);
                                                                        } else if new > 0
                                                                            as ::core::ffi::c_int
                                                                        {
                                                                            let mut key: *mut ::core::ffi::c_char = string_ndup(
                                                                                (*s).pair_keys,
                                                                                (*b).data as *mut ::core::ffi::c_char,
                                                                                ((*b).core.l_qname as ::core::ffi::c_int
                                                                                    - (*b).core.l_extranul as ::core::ffi::c_int) as size_t,
                                                                            );
                                                                            if key.is_null() {
                                                                                return -(1 as ::core::ffi::c_int);
                                                                            }
                                                                            let ref mut fresh150 =
                                                                                *(*(*s).pair
                                                                                    [sec as usize])
                                                                                    .keys
                                                                                    .offset(
                                                                                        k as isize,
                                                                                    );
                                                                            *fresh150 =
                                                                                key as kh_cstr_t;
                                                                            *(*(*s).pair[sec as usize]).vals.offset(k as isize) = (rnum
                                                                                as ::core::ffi::c_uint
                                                                                | (((*cr).flags & BAM_FREAD1 as int32_t != 0 as int32_t)
                                                                                    as ::core::ffi::c_int as ::core::ffi::c_uint)
                                                                                    << 30 as ::core::ffi::c_int
                                                                                | (((*cr).flags & BAM_FREAD2 as int32_t != 0 as int32_t)
                                                                                    as ::core::ffi::c_int as ::core::ffi::c_uint)
                                                                                    << 31 as ::core::ffi::c_int) as ::core::ffi::c_int;
                                                                        }
                                                                    } else {
                                                                        new =
                                                                            1 as ::core::ffi::c_int;
                                                                        k = 0 as khint_t;
                                                                    }
                                                                    if new
                                                                        == 0 as ::core::ffi::c_int
                                                                    {
                                                                        let mut p: *mut cram_record = (*s)
                                                                            .crecs
                                                                            .offset(
                                                                                (*(**(&raw mut (*s).pair as *mut *mut kh_m_s2i_t)
                                                                                    .offset(sec as isize))
                                                                                    .vals
                                                                                    .offset(k as isize)
                                                                                    & ((1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int)
                                                                                        - 1 as ::core::ffi::c_int) as isize,
                                                                            ) as *mut cram_record;
                                                                        let mut aleft: int64_t = 0;
                                                                        let mut aright: int64_t = 0;
                                                                        let mut sign: ::core::ffi::c_int = 0;
                                                                        aleft = if (*cr).apos
                                                                            < (*p).apos
                                                                        {
                                                                            (*cr).apos
                                                                        } else {
                                                                            (*p).apos
                                                                        };
                                                                        aright = if (*cr).aend
                                                                            > (*p).aend
                                                                        {
                                                                            (*cr).aend
                                                                        } else {
                                                                            (*p).aend
                                                                        };
                                                                        if (*cr).apos < (*p).apos {
                                                                            sign = 1 as ::core::ffi::c_int;
                                                                        } else if (*cr).apos
                                                                            > (*p).apos
                                                                        {
                                                                            sign = -(1 as ::core::ffi::c_int);
                                                                        } else if (*cr).flags
                                                                            & BAM_FREAD1 as int32_t
                                                                            != 0
                                                                        {
                                                                            sign = 1 as ::core::ffi::c_int;
                                                                        } else {
                                                                            sign = -(1 as ::core::ffi::c_int);
                                                                        }
                                                                        let mut has_r1: ::core::ffi::c_int = *(*(*s)
                                                                            .pair[sec as usize])
                                                                            .vals
                                                                            .offset(k as isize)
                                                                            & (1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int;
                                                                        let mut has_r2: ::core::ffi::c_uint = *(*(*s)
                                                                            .pair[sec as usize])
                                                                            .vals
                                                                            .offset(k as isize) as ::core::ffi::c_uint
                                                                            & (1 as ::core::ffi::c_uint) << 31 as ::core::ffi::c_int;
                                                                        if has_r1 != 0 && (*cr).flags & BAM_FREAD1 as int32_t != 0
                                                                            || has_r2 != 0 && (*cr).flags & BAM_FREAD2 as int32_t != 0
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if (*fd).tlen_zero == 0
                                                                            && (if (*b).core.mpos + 1 as hts_pos_t > 0 as hts_pos_t {
                                                                                (*b).core.mpos + 1 as hts_pos_t
                                                                            } else {
                                                                                0 as hts_pos_t
                                                                            }) != (*p).apos
                                                                            && !((*fd).tlen_zero != 0
                                                                                && (*b).core.mpos == 0 as hts_pos_t)
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if ((*b).core.flag as ::core::ffi::c_int
                                                                            & BAM_FMUNMAP != 0 as ::core::ffi::c_int)
                                                                            as ::core::ffi::c_int
                                                                            != ((*p).flags & BAM_FUNMAP as int32_t != 0 as int32_t)
                                                                                as ::core::ffi::c_int
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if ((*b).core.flag as ::core::ffi::c_int
                                                                            & BAM_FMREVERSE != 0 as ::core::ffi::c_int)
                                                                            as ::core::ffi::c_int
                                                                            != ((*p).flags & BAM_FREVERSE as int32_t != 0 as int32_t)
                                                                                as ::core::ffi::c_int
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if (*p).ref_id != (*cr).ref_id
                                                                            && !((*fd).tlen_zero != 0 && (*p).ref_id == -(1 as int32_t))
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if (*p).mate_pos != (*cr).apos
                                                                            && !((*fd).tlen_zero != 0 && (*p).mate_pos == 0 as int64_t)
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if ((*p).flags & BAM_FMUNMAP as int32_t
                                                                            != 0 as int32_t) as ::core::ffi::c_int
                                                                            != ((*p).mate_flags & CRAM_M_UNMAP as int32_t
                                                                                != 0 as int32_t) as ::core::ffi::c_int
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if ((*p).flags & BAM_FMREVERSE as int32_t
                                                                            != 0 as int32_t) as ::core::ffi::c_int
                                                                            != ((*p).mate_flags & CRAM_M_REVERSE as int32_t
                                                                                != 0 as int32_t) as ::core::ffi::c_int
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if (*cr).flags & BAM_FSUPPLEMENTARY as int32_t != 0
                                                                            || (*p).flags & BAM_FSUPPLEMENTARY as int32_t != 0
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else if (*fd).lossy_read_names != 0
                                                                            && ((*cr).cram_flags as ::core::ffi::c_uint
                                                                                & CRAM_FLAG_DISCARD_NAME == 0
                                                                                || (*p).cram_flags as ::core::ffi::c_uint
                                                                                    & CRAM_FLAG_DISCARD_NAME == 0)
                                                                        {
                                                                            current_block = 9299819377351801223;
                                                                        } else {
                                                                            let mut explicit_tlen: ::core::ffi::c_int = 0
                                                                                as ::core::ffi::c_int;
                                                                            let mut tflag1: ::core::ffi::c_int = ((*b).core.isize_0 != 0
                                                                                && llabs(
                                                                                    ((*b).core.isize_0
                                                                                        - sign as hts_pos_t
                                                                                            * (aright as hts_pos_t - aleft as hts_pos_t
                                                                                                + 1 as hts_pos_t)) as ::core::ffi::c_longlong,
                                                                                ) > (*fd).tlen_approx as ::core::ffi::c_longlong
                                                                                || (*b).core.isize_0 == 0 && (*fd).tlen_zero == 0)
                                                                                as ::core::ffi::c_int;
                                                                            let mut tflag2: ::core::ffi::c_int = ((*p).tlen != 0
                                                                                && llabs(
                                                                                    ((*p).tlen
                                                                                        - -sign as int64_t * (aright - aleft + 1 as int64_t))
                                                                                        as ::core::ffi::c_longlong,
                                                                                ) > (*fd).tlen_approx as ::core::ffi::c_longlong
                                                                                || (*p).tlen == 0 && (*fd).tlen_zero == 0)
                                                                                as ::core::ffi::c_int;
                                                                            if tflag1 != 0 || tflag2 != 0 {
                                                                                if (*fd).version >> 8 as ::core::ffi::c_int
                                                                                    >= 4 as ::core::ffi::c_int
                                                                                {
                                                                                    explicit_tlen = CRAM_FLAG_EXPLICIT_TLEN;
                                                                                    current_block = 4931126274483841711;
                                                                                } else {
                                                                                    current_block = 9299819377351801223;
                                                                                }
                                                                            } else {
                                                                                current_block = 4931126274483841711;
                                                                            }
                                                                            match current_block {
                                                                                9299819377351801223 => {}
                                                                                _ => {
                                                                                    (*cr).mate_pos = (*p).apos;
                                                                                    cram_stats_add(
                                                                                        (*c).stats[DS_NP as ::core::ffi::c_int as usize],
                                                                                        (*cr).mate_pos,
                                                                                    );
                                                                                    (*cr).tlen = (if explicit_tlen != 0 {
                                                                                        (*b).core.isize_0
                                                                                    } else {
                                                                                        sign as hts_pos_t
                                                                                            * (aright as hts_pos_t - aleft as hts_pos_t
                                                                                                + 1 as hts_pos_t)
                                                                                    }) as int64_t;
                                                                                    cram_stats_add(
                                                                                        (*c).stats[DS_TS as ::core::ffi::c_int as usize],
                                                                                        (*cr).tlen,
                                                                                    );
                                                                                    (*cr).mate_flags = (((*p).flags & BAM_FMUNMAP as int32_t
                                                                                        == BAM_FMUNMAP as int32_t) as ::core::ffi::c_int
                                                                                        * CRAM_M_UNMAP
                                                                                        + ((*p).flags & BAM_FMREVERSE as int32_t
                                                                                            == BAM_FMREVERSE as int32_t) as ::core::ffi::c_int
                                                                                            * CRAM_M_REVERSE) as int32_t;
                                                                                    if (*p).cram_flags & CRAM_FLAG_STATS_ADDED as int32_t != 0 {
                                                                                        cram_stats_del(
                                                                                            (*c).stats[DS_NP as ::core::ffi::c_int as usize],
                                                                                            (*p).mate_pos,
                                                                                        );
                                                                                        cram_stats_del(
                                                                                            (*c).stats[DS_MF as ::core::ffi::c_int as usize],
                                                                                            (*p).mate_flags as int64_t,
                                                                                        );
                                                                                        if (*p).cram_flags & CRAM_FLAG_EXPLICIT_TLEN as int32_t == 0
                                                                                            && explicit_tlen == 0
                                                                                        {
                                                                                            cram_stats_del(
                                                                                                (*c).stats[DS_TS as ::core::ffi::c_int as usize],
                                                                                                (*p).tlen,
                                                                                            );
                                                                                        }
                                                                                        cram_stats_del(
                                                                                            (*c).stats[DS_NS as ::core::ffi::c_int as usize],
                                                                                            (*p).mate_ref_id as int64_t,
                                                                                        );
                                                                                    }
                                                                                    (*cr).cram_flags = ((*cr).cram_flags as ::core::ffi::c_int
                                                                                        & !CRAM_FLAG_DETACHED) as int32_t;
                                                                                    (*cr).cram_flags = ((*cr).cram_flags as ::core::ffi::c_int
                                                                                        | explicit_tlen) as int32_t;
                                                                                    if cram_stats_add(
                                                                                        (*c).stats[DS_CF as ::core::ffi::c_int as usize],
                                                                                        ((*cr).cram_flags & CRAM_FLAG_MASK as int32_t) as int64_t,
                                                                                    ) < 0 as ::core::ffi::c_int
                                                                                    {
                                                                                        current_block = 4645371139350450943;
                                                                                    } else {
                                                                                        if (*p).cram_flags & CRAM_FLAG_STATS_ADDED as int32_t != 0 {
                                                                                            cram_stats_del(
                                                                                                (*c).stats[DS_CF as ::core::ffi::c_int as usize],
                                                                                                ((*p).cram_flags & CRAM_FLAG_MASK as int32_t) as int64_t,
                                                                                            );
                                                                                            (*p).cram_flags = ((*p).cram_flags as ::core::ffi::c_int
                                                                                                & !CRAM_FLAG_STATS_ADDED) as int32_t;
                                                                                        }
                                                                                        (*p).cram_flags = ((*p).cram_flags as ::core::ffi::c_int
                                                                                            & !CRAM_FLAG_DETACHED) as int32_t;
                                                                                        (*p).cram_flags = ((*p).cram_flags as ::core::ffi::c_int
                                                                                            | (CRAM_FLAG_MATE_DOWNSTREAM | explicit_tlen)) as int32_t;
                                                                                        if cram_stats_add(
                                                                                            (*c).stats[DS_CF as ::core::ffi::c_int as usize],
                                                                                            ((*p).cram_flags & CRAM_FLAG_MASK as int32_t) as int64_t,
                                                                                        ) < 0 as ::core::ffi::c_int
                                                                                        {
                                                                                            current_block = 4645371139350450943;
                                                                                        } else {
                                                                                            (*p).mate_line = (rnum
                                                                                                - ((*(*(*s).pair[sec as usize]).vals.offset(k as isize)
                                                                                                    & ((1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int)
                                                                                                        - 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int))
                                                                                                as int32_t;
                                                                                            if cram_stats_add(
                                                                                                (*c).stats[DS_NF as ::core::ffi::c_int as usize],
                                                                                                (*p).mate_line as int64_t,
                                                                                            ) < 0 as ::core::ffi::c_int
                                                                                            {
                                                                                                current_block = 4645371139350450943;
                                                                                            } else {
                                                                                                let mut r12_flags: ::core::ffi::c_int = (*(*(*s)
                                                                                                    .pair[sec as usize])
                                                                                                    .vals
                                                                                                    .offset(k as isize) as ::core::ffi::c_uint
                                                                                                    & (3 as ::core::ffi::c_uint) << 30 as ::core::ffi::c_int)
                                                                                                    as ::core::ffi::c_int;
                                                                                                *(*(*s).pair[sec as usize]).vals.offset(k as isize) = (rnum
                                                                                                    as ::core::ffi::c_uint | r12_flags as ::core::ffi::c_uint
                                                                                                    | ((((*cr).flags & BAM_FREAD1 as int32_t != 0 as int32_t)
                                                                                                        as ::core::ffi::c_int) << 30 as ::core::ffi::c_int)
                                                                                                        as ::core::ffi::c_uint
                                                                                                    | (((*cr).flags & BAM_FREAD2 as int32_t != 0 as int32_t)
                                                                                                        as ::core::ffi::c_int as ::core::ffi::c_uint)
                                                                                                        << 31 as ::core::ffi::c_int) as ::core::ffi::c_int;
                                                                                                current_block = 8422527538794739384;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        current_block =
                                                                            9299819377351801223;
                                                                    }
                                                                    match current_block {
                                                                        4645371139350450943 => {}
                                                                        _ => {
                                                                            match current_block {
                                                                                9299819377351801223 => {
                                                                                    (*cr).mate_flags = 0 as ::core::ffi::c_int as int32_t;
                                                                                    if (*b).core.flag as ::core::ffi::c_int & BAM_FMUNMAP != 0 {
                                                                                        (*cr).mate_flags = ((*cr).mate_flags as ::core::ffi::c_int
                                                                                            | CRAM_M_UNMAP) as int32_t;
                                                                                    }
                                                                                    if (*b).core.flag as ::core::ffi::c_int & BAM_FMREVERSE != 0
                                                                                    {
                                                                                        (*cr).mate_flags = ((*cr).mate_flags as ::core::ffi::c_int
                                                                                            | CRAM_M_REVERSE) as int32_t;
                                                                                    }
                                                                                    if cram_stats_add(
                                                                                        (*c).stats[DS_MF as ::core::ffi::c_int as usize],
                                                                                        (*cr).mate_flags as int64_t,
                                                                                    ) < 0 as ::core::ffi::c_int
                                                                                    {
                                                                                        current_block = 4645371139350450943;
                                                                                    } else {
                                                                                        (*cr).mate_pos = (if (*b).core.mpos + 1 as hts_pos_t
                                                                                            > 0 as hts_pos_t
                                                                                        {
                                                                                            (*b).core.mpos + 1 as hts_pos_t
                                                                                        } else {
                                                                                            0 as hts_pos_t
                                                                                        }) as int64_t;
                                                                                        if cram_stats_add(
                                                                                            (*c).stats[DS_NP as ::core::ffi::c_int as usize],
                                                                                            (*cr).mate_pos,
                                                                                        ) < 0 as ::core::ffi::c_int
                                                                                        {
                                                                                            current_block = 4645371139350450943;
                                                                                        } else {
                                                                                            (*cr).tlen = (*b).core.isize_0 as int64_t;
                                                                                            if cram_stats_add(
                                                                                                (*c).stats[DS_TS as ::core::ffi::c_int as usize],
                                                                                                (*cr).tlen,
                                                                                            ) < 0 as ::core::ffi::c_int
                                                                                            {
                                                                                                current_block = 4645371139350450943;
                                                                                            } else {
                                                                                                (*cr).cram_flags = ((*cr).cram_flags as ::core::ffi::c_int
                                                                                                    | CRAM_FLAG_DETACHED) as int32_t;
                                                                                                if cram_stats_add(
                                                                                                    (*c).stats[DS_CF as ::core::ffi::c_int as usize],
                                                                                                    ((*cr).cram_flags & CRAM_FLAG_MASK as int32_t) as int64_t,
                                                                                                ) < 0 as ::core::ffi::c_int
                                                                                                {
                                                                                                    current_block = 4645371139350450943;
                                                                                                } else if cram_stats_add(
                                                                                                    (*c).stats[DS_NS as ::core::ffi::c_int as usize],
                                                                                                    (*b).core.mtid as int64_t,
                                                                                                ) < 0 as ::core::ffi::c_int
                                                                                                {
                                                                                                    current_block = 4645371139350450943;
                                                                                                } else {
                                                                                                    (*cr).cram_flags = ((*cr).cram_flags as ::core::ffi::c_int
                                                                                                        | CRAM_FLAG_STATS_ADDED) as int32_t;
                                                                                                    current_block = 8422527538794739384;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                _ => {}
                                                                            }
                                                                            match current_block {
                                                                                4645371139350450943 => {}
                                                                                _ => {
                                                                                    (*cr).mqual = (*b).core.qual as int32_t;
                                                                                    if !(cram_stats_add(
                                                                                        (*c).stats[DS_MQ as ::core::ffi::c_int as usize],
                                                                                        (*cr).mqual as int64_t,
                                                                                    ) < 0 as ::core::ffi::c_int)
                                                                                    {
                                                                                        (*cr).mate_ref_id = (*b).core.mtid;
                                                                                        if (*b).core.flag as ::core::ffi::c_int & BAM_FUNMAP == 0 {
                                                                                            if (*c).first_base > (*cr).apos {
                                                                                                (*c).first_base = (*cr).apos as hts_pos_t;
                                                                                            }
                                                                                            if (*c).last_base < (*cr).aend {
                                                                                                (*c).last_base = (*cr).aend as hts_pos_t;
                                                                                            }
                                                                                        }
                                                                                        return 0 as ::core::ffi::c_int;
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
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
// original: cram_put_bam_seq (htslib/cram/cram_encode.c:4045)
pub unsafe extern "C" fn cram_put_bam_seq(
    mut fd: *mut cram_fd,
    mut b: *mut bam_seq_t,
) -> ::core::ffi::c_int {
    let mut c: *mut cram_container = ::core::ptr::null_mut::<cram_container>();
    if (*fd).ctr.is_null() {
        (*fd).ctr = cram_new_container((*fd).seqs_per_slice, (*fd).slices_per_container);
        if (*fd).ctr.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*(*fd).ctr).record_counter = (*fd).record_counter;
        pthread_mutex_lock(&raw mut (*fd).ref_lock);
        (*(*fd).ctr).no_ref = (*fd).no_ref;
        (*(*fd).ctr).embed_ref = (*fd).embed_ref;
        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
    }
    c = (*fd).ctr;
    let mut embed_ref: ::core::ffi::c_int = (*c).embed_ref;
    if (*c).slice.is_null()
        || (*c).curr_rec == (*c).max_rec
        || (*b).core.tid != (*c).curr_ref as int32_t && (*c).curr_ref >= -(1 as ::core::ffi::c_int)
        || (*c).s_num_bases.wrapping_add((*c).s_aux_bytes) >= (*fd).bases_per_slice as uint64_t
    {
        let mut slice_rec: ::core::ffi::c_int = 0;
        let mut curr_rec: ::core::ffi::c_int = 0;
        let mut multi_seq: ::core::ffi::c_int =
            ((*fd).multi_seq == 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
        let mut curr_ref: ::core::ffi::c_int = if !(*c).slice.is_null() {
            (*c).curr_ref
        } else {
            (*b).core.tid as ::core::ffi::c_int
        };
        if (*fd).multi_seq == -(1 as ::core::ffi::c_int)
            && (*c).curr_rec < (*c).max_rec / 4 as ::core::ffi::c_int + 10 as ::core::ffi::c_int
            && (*fd).last_slice != 0
            && (*fd).last_slice < (*c).max_rec / 4 as ::core::ffi::c_int + 10 as ::core::ffi::c_int
            && embed_ref <= 0 as ::core::ffi::c_int
        {
            if (*c).multi_seq == 0 {
                hts_log(
                    HTS_LOG_INFO,
                    b"cram_put_bam_seq\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Multi-ref enabled for next container\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            multi_seq = 1 as ::core::ffi::c_int;
        } else if (*fd).multi_seq == 1 as ::core::ffi::c_int {
            pthread_mutex_lock(&raw mut (*fd).metrics_lock);
            if (*fd).last_RI_count <= (*c).max_slice
                && (*fd).multi_seq_user != 1 as ::core::ffi::c_int
            {
                multi_seq = 0 as ::core::ffi::c_int;
                hts_log(
                    HTS_LOG_INFO,
                    b"cram_put_bam_seq\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Multi-ref disabled for next container\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            pthread_mutex_unlock(&raw mut (*fd).metrics_lock);
        }
        slice_rec = (*c).slice_rec;
        curr_rec = (*c).curr_rec;
        if (*fd).version >> 8 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || (*c).curr_rec == (*c).max_rec
            || (*fd).multi_seq != 1 as ::core::ffi::c_int
            || (*c).slice.is_null()
            || (*c).s_num_bases.wrapping_add((*c).s_aux_bytes) >= (*fd).bases_per_slice as uint64_t
        {
            c = cram_next_container(fd, b);
            if c.is_null() {
                if !(*fd).ctr.is_null() {
                    (*fd).ctr_mt = (*fd).ctr;
                    (*fd).ctr = ::core::ptr::null_mut::<cram_container>();
                }
                return -(1 as ::core::ffi::c_int);
            }
        }
        if multi_seq == 0 as ::core::ffi::c_int
            && (*fd).multi_seq == 1 as ::core::ffi::c_int
            && (*fd).multi_seq_user == -(1 as ::core::ffi::c_int)
        {
            (*fd).multi_seq = -(1 as ::core::ffi::c_int);
        } else if multi_seq != 0 {
            (*fd).multi_seq = 1 as ::core::ffi::c_int;
            (*c).multi_seq = 1 as ::core::ffi::c_int;
            (*c).pos_sorted = 0 as ::core::ffi::c_int;
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            if (*fd).embed_ref > 0 as ::core::ffi::c_int
                && (*c).curr_rec == 0 as ::core::ffi::c_int
                && (*c).curr_slice == 0 as ::core::ffi::c_int
            {
                hts_log(
                    HTS_LOG_WARNING,
                    b"cram_put_bam_seq\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Changing from embed_ref to no_ref mode\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                (*fd).embed_ref = 0 as ::core::ffi::c_int;
                (*c).embed_ref = (*fd).embed_ref;
                (*fd).no_ref = 1 as ::core::ffi::c_int;
                (*c).no_ref = (*fd).no_ref;
            }
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            if (*c).refs_used.is_null() {
                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                (*c).refs_used = calloc(
                    (*(*fd).refs).nref as size_t,
                    ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
                ) as *mut ::core::ffi::c_int;
                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                if (*c).refs_used.is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
            }
        }
        (*fd).last_slice = curr_rec - slice_rec;
        (*c).slice_rec = (*c).curr_rec;
        if (*b).core.tid >= 0 as int32_t
            && curr_ref >= 0 as ::core::ffi::c_int
            && (*b).core.tid != curr_ref as int32_t
            && embed_ref <= 0 as ::core::ffi::c_int
            && (*fd).unsorted == 0
            && multi_seq != 0
        {
            if (*c).refs_used.is_null() {
                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                (*c).refs_used = calloc(
                    (*(*fd).refs).nref as size_t,
                    ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
                ) as *mut ::core::ffi::c_int;
                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                if (*c).refs_used.is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
            } else if !(*c).refs_used.is_null()
                && *(*c).refs_used.offset((*b).core.tid as isize) != 0
            {
                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                (*fd).unsorted = 1 as ::core::ffi::c_int;
                (*fd).multi_seq = 1 as ::core::ffi::c_int;
                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            }
        }
        (*c).curr_ref = (*b).core.tid as ::core::ffi::c_int;
        if !(*c).refs_used.is_null() && (*c).curr_ref >= 0 as ::core::ffi::c_int {
            let ref mut fresh120 = *(*c).refs_used.offset((*c).curr_ref as isize);
            *fresh120 += 1;
        }
    }
    if (*c).bams.is_null() {
        pthread_mutex_lock(&raw mut (*fd).bam_list_lock);
        if !(*fd).bl.is_null() {
            let mut spare: *mut spare_bams = (*fd).bl;
            (*c).bams = (*spare).bams;
            (*fd).bl = (*spare).next as *mut spare_bams;
            free(spare as *mut ::core::ffi::c_void);
        } else {
            (*c).bams = calloc(
                (*c).max_c_rec as size_t,
                ::core::mem::size_of::<*mut bam_seq_t>() as size_t,
            ) as *mut *mut bam_seq_t;
            if (*c).bams.is_null() {
                pthread_mutex_unlock(&raw mut (*fd).bam_list_lock);
                return -(1 as ::core::ffi::c_int);
            }
        }
        pthread_mutex_unlock(&raw mut (*fd).bam_list_lock);
    }
    if !(*(*c).bams.offset((*c).curr_c_rec as isize)).is_null() {
        if bam_copy1(
            *(*c).bams.offset((*c).curr_c_rec as isize) as *mut bam1_t,
            b,
        )
        .is_null()
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else {
        let ref mut fresh121 = *(*c).bams.offset((*c).curr_c_rec as isize);
        *fresh121 = bam_dup1(b) as *mut bam_seq_t;
        if (*(*c).bams.offset((*c).curr_c_rec as isize)).is_null() {
            return -(1 as ::core::ffi::c_int);
        }
    }
    if (*b).core.l_qseq != 0 {
        (*c).s_num_bases = ((*c).s_num_bases as ::core::ffi::c_ulong)
            .wrapping_add((*b).core.l_qseq as ::core::ffi::c_ulong)
            as uint64_t as uint64_t;
    } else {
        let mut qlen: hts_pos_t = bam_cigar2qlen(
            (*b).core.n_cigar as ::core::ffi::c_int,
            (*b).data
                .offset((*b).core.l_qname as ::core::ffi::c_int as isize)
                as *mut uint32_t,
        );
        if qlen > 100000000 as hts_pos_t {
            hts_log(
                HTS_LOG_ERROR,
                b"cram_put_bam_seq\0" as *const u8 as *const ::core::ffi::c_char,
                b"CIGAR query length %ld for read \"%s\" is too long\0" as *const u8
                    as *const ::core::ffi::c_char,
                qlen,
                (*b).data as *mut ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*c).s_num_bases = ((*c).s_num_bases as ::core::ffi::c_ulong)
            .wrapping_add(qlen as ::core::ffi::c_ulong) as uint64_t
            as uint64_t;
    }
    (*c).curr_rec += 1;
    (*c).curr_c_rec += 1;
    (*c).s_aux_bytes = ((*c).s_aux_bytes as ::core::ffi::c_ulong).wrapping_add(
        ((*b).l_data as uint32_t)
            .wrapping_sub((*b).core.n_cigar << 2 as ::core::ffi::c_int)
            .wrapping_sub((*b).core.l_qname as uint32_t)
            .wrapping_sub((*b).core.l_qseq as uint32_t)
            .wrapping_sub(((*b).core.l_qseq + 1 as int32_t >> 1 as ::core::ffi::c_int) as uint32_t)
            as ::core::ffi::c_ulong,
    ) as uint64_t as uint64_t;
    (*c).n_mapped = ((*c).n_mapped as ::core::ffi::c_uint).wrapping_add(
        (if (*b).core.flag as ::core::ffi::c_int & BAM_FUNMAP != 0 {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
    ) as uint32_t as uint32_t;
    (*fd).record_counter += 1;
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn realloc_bam_data(
    mut b: *mut bam1_t,
    mut desired: size_t,
) -> ::core::ffi::c_int {
    if desired <= (*b).m_data as size_t {
        return 0 as ::core::ffi::c_int;
    }
    return sam_realloc_bam_data(b, desired);
}
#[inline]
unsafe extern "C" fn possibly_expand_bam_data(
    mut b: *mut bam1_t,
    mut bytes: size_t,
) -> ::core::ffi::c_int {
    let mut new_len: size_t = ((*b).l_data as size_t).wrapping_add(bytes);
    if new_len > INT32_MAX as size_t || new_len < bytes {
        *__errno_location() = ENOMEM;
        return -(1 as ::core::ffi::c_int);
    }
    if new_len <= (*b).m_data as size_t {
        return 0 as ::core::ffi::c_int;
    }
    return sam_realloc_bam_data(b, new_len);
}
#[inline]
unsafe extern "C" fn nibble2base_default(
    mut nib: *mut uint8_t,
    mut seq: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    static mut code2base: [::core::ffi::c_char; 512] = unsafe {
        ::core::mem::transmute::<
            [u8; 512],
            [::core::ffi::c_char; 512],
        >(
            *b"===A=C=M=G=R=S=V=T=W=Y=H=K=D=B=NA=AAACAMAGARASAVATAWAYAHAKADABANC=CACCCMCGCRCSCVCTCWCYCHCKCDCBCNM=MAMCMMMGMRMSMVMTMWMYMHMKMDMBMNG=GAGCGMGGGRGSGVGTGWGYGHGKGDGBGNR=RARCRMRGRRRSRVRTRWRYRHRKRDRBRNS=SASCSMSGSRSSSVSTSWSYSHSKSDSBSNV=VAVCVMVGVRVSVVVTVWVYVHVKVDVBVNT=TATCTMTGTRTSTVTTTWTYTHTKTDTBTNW=WAWCWMWGWRWSWVWTWWWYWHWKWDWBWNY=YAYCYMYGYRYSYVYTYWYYYHYKYDYBYNH=HAHCHMHGHRHSHVHTHWHYHHHKHDHBHNK=KAKCKMKGKRKSKVKTKWKYKHKKKDKBKND=DADCDMDGDRDSDVDTDWDYDHDKDDDBDNB=BABCBMBGBRBSBVBTBWBYBHBKBDBBBNN=NANCNMNGNRNSNVNTNWNYNHNKNDNBNN",
        )
    };
    let mut i: ::core::ffi::c_int = 0;
    let mut len2: ::core::ffi::c_int = len / 2 as ::core::ffi::c_int;
    *seq.offset(0 as ::core::ffi::c_int as isize) = 0 as ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < len2 {
        memcpy(
            seq.offset((i * 2 as ::core::ffi::c_int) as isize) as *mut ::core::ffi::c_char
                as *mut ::core::ffi::c_void,
            (&raw const code2base as *const ::core::ffi::c_char)
                .offset((*nib.offset(i as isize) as size_t).wrapping_mul(2 as size_t) as isize)
                as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
        );
        i += 1;
    }
    i *= 2 as ::core::ffi::c_int;
    if i < len {
        *seq.offset(i as isize) = *(&raw const seq_nt16_str as *const ::core::ffi::c_char).offset(
            (*nib.offset((i >> 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                >> ((!i & 1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int)
                & 0xf as ::core::ffi::c_int) as isize,
        );
    }
}
#[inline]
unsafe extern "C" fn nibble2base(
    mut nib: *mut uint8_t,
    mut seq: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    nibble2base_default(nib, seq, len);
}
pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EFAULT: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
#[inline]
unsafe extern "C" fn isalnum_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isalpha_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isdigit_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isgraph_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISgraph as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn islower_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isprint_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISprint as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn ispunct_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISpunct as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isspace_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isupper_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn isxdigit_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return *(*__ctype_b_loc()).offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn tolower_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_char {
    return tolower(c as ::core::ffi::c_uchar as ::core::ffi::c_int) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn toupper_c(mut c: ::core::ffi::c_char) -> ::core::ffi::c_char {
    return toupper(c as ::core::ffi::c_uchar as ::core::ffi::c_int) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn hts_str2int(
    mut in_0: *const ::core::ffi::c_char,
    mut end: *mut *mut ::core::ffi::c_char,
    mut bits: ::core::ffi::c_int,
    mut failed: *mut ::core::ffi::c_int,
) -> int64_t {
    let mut n: uint64_t = 0 as uint64_t;
    let mut limit: uint64_t = ((1 as ::core::ffi::c_ulonglong) << bits - 1 as ::core::ffi::c_int)
        .wrapping_sub(1 as ::core::ffi::c_ulonglong) as uint64_t;
    let mut fast: uint32_t = ((bits - 1 as ::core::ffi::c_int) * 1000 as ::core::ffi::c_int
        / 3322 as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint32_t;
    let mut v: *const ::core::ffi::c_uchar = in_0 as *const ::core::ffi::c_uchar;
    let ascii_zero: ::core::ffi::c_uint = '0' as i32 as ::core::ffi::c_uint;
    let mut d: ::core::ffi::c_uint = 0;
    let mut neg: ::core::ffi::c_int = 0;
    let mut current_block_8: u64;
    match *v as ::core::ffi::c_int {
        45 => {
            limit = limit.wrapping_add(1);
            neg = 1 as ::core::ffi::c_int;
            v = v.offset(1);
            loop {
                fast = fast.wrapping_sub(1);
                if !(fast != 0
                    && *v as ::core::ffi::c_int >= '0' as i32
                    && *v as ::core::ffi::c_int <= '9' as i32)
                {
                    break;
                }
                let fresh181 = v;
                v = v.offset(1);
                n = n
                    .wrapping_mul(10 as uint64_t)
                    .wrapping_add(*fresh181 as uint64_t)
                    .wrapping_sub(ascii_zero as uint64_t);
            }
            current_block_8 = 2968425633554183086;
        }
        43 => {
            v = v.offset(1);
            current_block_8 = 12458458191105553904;
        }
        _ => {
            current_block_8 = 12458458191105553904;
        }
    }
    match current_block_8 {
        12458458191105553904 => {
            neg = 0 as ::core::ffi::c_int;
            loop {
                fast = fast.wrapping_sub(1);
                if !(fast != 0
                    && *v as ::core::ffi::c_int >= '0' as i32
                    && *v as ::core::ffi::c_int <= '9' as i32)
                {
                    break;
                }
                let fresh182 = v;
                v = v.offset(1);
                n = n
                    .wrapping_mul(10 as uint64_t)
                    .wrapping_add(*fresh182 as uint64_t)
                    .wrapping_sub(ascii_zero as uint64_t);
            }
        }
        _ => {}
    }
    if *v as ::core::ffi::c_int >= '0' as i32 && fast == 0 {
        let mut limit_d_10: uint64_t = limit.wrapping_div(10 as uint64_t);
        let mut limit_m_10: uint64_t =
            limit.wrapping_sub((10 as uint64_t).wrapping_mul(limit_d_10));
        loop {
            d = (*v as ::core::ffi::c_uint).wrapping_sub(ascii_zero);
            if !(d < 10 as ::core::ffi::c_uint) {
                break;
            }
            if n < limit_d_10 || n == limit_d_10 && d as uint64_t <= limit_m_10 {
                n = n.wrapping_mul(10 as uint64_t).wrapping_add(d as uint64_t);
                v = v.offset(1);
            } else {
                loop {
                    v = v.offset(1);
                    if !((*v as ::core::ffi::c_uint).wrapping_sub(ascii_zero)
                        < 10 as ::core::ffi::c_uint)
                    {
                        break;
                    }
                }
                n = limit;
                *failed = 1 as ::core::ffi::c_int;
                break;
            }
        }
    }
    *end = v as *mut ::core::ffi::c_char;
    return if neg != 0 {
        n.wrapping_neg() as int64_t
    } else {
        n as int64_t
    };
}
#[inline]
unsafe extern "C" fn hts_str2uint(
    mut in_0: *const ::core::ffi::c_char,
    mut end: *mut *mut ::core::ffi::c_char,
    mut bits: ::core::ffi::c_int,
    mut failed: *mut ::core::ffi::c_int,
) -> uint64_t {
    let mut n: uint64_t = 0 as uint64_t;
    let mut limit: uint64_t = (if bits < 64 as ::core::ffi::c_int {
        (1 as ::core::ffi::c_ulonglong) << bits
    } else {
        0 as ::core::ffi::c_ulonglong
    })
    .wrapping_sub(1 as ::core::ffi::c_ulonglong) as uint64_t;
    let mut v: *const ::core::ffi::c_uchar = in_0 as *const ::core::ffi::c_uchar;
    let ascii_zero: ::core::ffi::c_uint = '0' as i32 as ::core::ffi::c_uint;
    let mut fast: uint32_t = (bits * 1000 as ::core::ffi::c_int / 3322 as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint32_t;
    let mut d: ::core::ffi::c_uint = 0;
    if *v as ::core::ffi::c_int == '+' as i32 {
        v = v.offset(1);
    }
    loop {
        fast = fast.wrapping_sub(1);
        if !(fast != 0
            && *v as ::core::ffi::c_int >= '0' as i32
            && *v as ::core::ffi::c_int <= '9' as i32)
        {
            break;
        }
        let fresh171 = v;
        v = v.offset(1);
        n = n
            .wrapping_mul(10 as uint64_t)
            .wrapping_add(*fresh171 as uint64_t)
            .wrapping_sub(ascii_zero as uint64_t);
    }
    if (*v as ::core::ffi::c_uint).wrapping_sub(ascii_zero) < 10 as ::core::ffi::c_uint && fast == 0
    {
        let mut limit_d_10: uint64_t = limit.wrapping_div(10 as uint64_t);
        let mut limit_m_10: uint64_t =
            limit.wrapping_sub((10 as uint64_t).wrapping_mul(limit_d_10));
        loop {
            d = (*v as ::core::ffi::c_uint).wrapping_sub(ascii_zero);
            if !(d < 10 as ::core::ffi::c_uint) {
                break;
            }
            if n < limit_d_10 || n == limit_d_10 && d as uint64_t <= limit_m_10 {
                n = n.wrapping_mul(10 as uint64_t).wrapping_add(d as uint64_t);
                v = v.offset(1);
            } else {
                loop {
                    v = v.offset(1);
                    if !((*v as ::core::ffi::c_uint).wrapping_sub(ascii_zero)
                        < 10 as ::core::ffi::c_uint)
                    {
                        break;
                    }
                }
                n = limit;
                *failed = 1 as ::core::ffi::c_int;
                break;
            }
        }
    }
    *end = v as *mut ::core::ffi::c_char;
    return n;
}
#[inline]
unsafe extern "C" fn hts_str2dbl(
    mut in_0: *const ::core::ffi::c_char,
    mut end: *mut *mut ::core::ffi::c_char,
    mut failed: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_double {
    let mut n: uint64_t = 0 as uint64_t;
    let mut max_len: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    let mut v: *const ::core::ffi::c_uchar = in_0 as *const ::core::ffi::c_uchar;
    let ascii_zero: ::core::ffi::c_uint = '0' as i32 as ::core::ffi::c_uint;
    let mut neg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut point: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut d: ::core::ffi::c_double = 0.;
    static mut D: [::core::ffi::c_double; 22] = [
        1 as ::core::ffi::c_int as ::core::ffi::c_double,
        1 as ::core::ffi::c_int as ::core::ffi::c_double,
        1e1f64,
        1e2f64,
        1e3f64,
        1e4f64,
        1e5f64,
        1e6f64,
        1e7f64,
        1e8f64,
        1e9f64,
        1e10f64,
        1e11f64,
        1e12f64,
        1e13f64,
        1e14f64,
        1e15f64,
        1e16f64,
        1e17f64,
        1e18f64,
        1e19f64,
        1e20f64,
    ];
    while *(*__ctype_b_loc()).offset(*v as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        v = v.offset(1);
    }
    if *v as ::core::ffi::c_int == '-' as i32 {
        neg = 1 as ::core::ffi::c_int;
        v = v.offset(1);
    } else if *v as ::core::ffi::c_int == '+' as i32 {
        v = v.offset(1);
    }
    let mut current_block_12: u64;
    match *v as ::core::ffi::c_int {
        49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
            current_block_12 = 17833034027772472439;
        }
        48 => {
            if *v.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'x' as i32
                && *v.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'X' as i32
            {
                current_block_12 = 17833034027772472439;
            } else {
                current_block_12 = 8144921725673815254;
            }
        }
        _ => {
            current_block_12 = 8144921725673815254;
        }
    }
    match current_block_12 {
        17833034027772472439 => {}
        _ => {
            d = strtod(in_0, end);
            if *end == in_0 as *mut ::core::ffi::c_char {
                *failed = 1 as ::core::ffi::c_int;
            }
            return d;
        }
    }
    while *v as ::core::ffi::c_int == '0' as i32 {
        v = v.offset(1);
    }
    let mut start: *const ::core::ffi::c_uchar = v;
    loop {
        max_len -= 1;
        if !(max_len != 0
            && *v as ::core::ffi::c_int >= '0' as i32
            && *v as ::core::ffi::c_int <= '9' as i32)
        {
            break;
        }
        let fresh183 = v;
        v = v.offset(1);
        n = n
            .wrapping_mul(10 as uint64_t)
            .wrapping_add(*fresh183 as uint64_t)
            .wrapping_sub(ascii_zero as uint64_t);
    }
    if max_len != 0 && *v as ::core::ffi::c_int == '.' as i32 {
        point = v.offset_from(start) as ::core::ffi::c_long as ::core::ffi::c_int;
        v = v.offset(1);
        loop {
            max_len -= 1;
            if !(max_len != 0
                && *v as ::core::ffi::c_int >= '0' as i32
                && *v as ::core::ffi::c_int <= '9' as i32)
            {
                break;
            }
            let fresh184 = v;
            v = v.offset(1);
            n = n
                .wrapping_mul(10 as uint64_t)
                .wrapping_add(*fresh184 as uint64_t)
                .wrapping_sub(ascii_zero as uint64_t);
        }
    }
    if point < 0 as ::core::ffi::c_int {
        point = v.offset_from(start) as ::core::ffi::c_long as ::core::ffi::c_int;
    }
    if max_len == 0
        || *v as ::core::ffi::c_int == 'e' as i32
        || *v as ::core::ffi::c_int == 'E' as i32
    {
        d = strtod(in_0, end);
        if *end == in_0 as *mut ::core::ffi::c_char {
            *failed = 1 as ::core::ffi::c_int;
        }
        return d;
    }
    *end = v as *mut ::core::ffi::c_char;
    d = n as ::core::ffi::c_double
        / D[(v.offset_from(start) as ::core::ffi::c_long - point as ::core::ffi::c_long) as usize];
    return if neg != 0 { -d } else { d };
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
