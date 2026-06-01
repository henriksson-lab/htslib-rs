// Functions translated from htslib/cram/cram_decode.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong, c_void, CStr};

use super::*;

/// original: cram_decode_TD (htslib/cram/cram_decode.c:71)
/// Native port. Parses the tag-dictionary (TD) block stored in the preservation
/// map. Allocates a fresh `cram_block` (via the native cram_new_block) holding
/// the TD bytes, builds the `tl` lookup table (one entry per NUL-separated
/// tag-list) and stores both into the compression header. Ownership matches C:
/// `tl` is calloc'd and `td_blk` is the new block; both are freed by
/// `cram_free_compression_header`. Returns bytes consumed, or -1 on error.
pub unsafe fn cram_cram_decode_c_71_cram_decode_TD(
    fd: *mut cram_fd,
    mut cp: *mut c_char,
    endp: *const c_char,
    h: *mut cram_block_compression_hdr_layout,
) -> c_int {
    let op = cp;
    // C: cram_new_block(0, 0)  -> content_type FILE_HEADER (=0), content_id 0.
    let b = cram_cram_io_c_1388_cram_new_block(CRAM_CONTENT_TYPE_FILE_HEADER, 0);
    if b.is_null() {
        return -1;
    }
    let bl = b.cast::<cram_block_layout>();

    if !(*h).td_blk.is_null() || !(*h).tl.is_null() {
        hts_log_cstr(
            HTS_LOG_WARNING,
            c"cram_decode_TD".as_ptr(),
            c"More than one TD block found in compression header".as_ptr(),
        );
        cram_cram_io_c_1565_cram_free_block((*h).td_blk.cast());
        free((*h).tl.cast());
        (*h).td_blk = std::ptr::null_mut();
        (*h).tl = std::ptr::null_mut();
    }

    let fdl = fd.cast::<cram_fd_layout>();
    let vv = &(*fdl).vv;
    let mut err: c_int = 0;
    let blk_size = (vv.varint_get32.unwrap())(&mut cp, endp, &mut err) as i32;
    if blk_size == 0 {
        (*h).ntl = 0;
        cram_cram_io_c_1565_cram_free_block(b);
        return cp.offset_from(op) as c_int;
    }
    if err != 0 || blk_size < 0 || (endp.offset_from(cp) as i64) < blk_size as i64 {
        cram_cram_io_c_1565_cram_free_block(b);
        return -1;
    }

    if cram_cram_io_h_248_block_append(b, cp.cast(), blk_size as usize) < 0 {
        cram_cram_io_c_1565_cram_free_block(b);
        return -1;
    }
    cp = cp.add(blk_size as usize);
    let sz = cp.offset_from(op) as c_int;
    // Force NUL termination if missing.
    if *(*bl).data.add((*bl).byte - 1) != 0
        && cram_cram_io_h_261_block_append_char(b, b'\0' as c_char) < 0
    {
        cram_cram_io_c_1565_cram_free_block(b);
        return -1;
    }

    let dat = (*bl).data;
    // Count tag-lists (NUL-separated strings).
    let mut ntl: c_int = 0;
    let mut i: usize = 0;
    while i < (*bl).byte {
        ntl += 1;
        while *dat.add(i) != 0 {
            i += 1;
        }
        i += 1;
    }

    (*h).tl = calloc(ntl as u64, std::mem::size_of::<*mut u8>() as u64).cast::<*mut u8>();
    if (*h).tl.is_null() {
        cram_cram_io_c_1565_cram_free_block(b);
        return -1;
    }
    let mut nidx: c_int = 0;
    i = 0;
    while i < (*bl).byte {
        *(*h).tl.add(nidx as usize) = dat.add(i);
        nidx += 1;
        while *dat.add(i) != 0 {
            i += 1;
        }
        i += 1;
    }
    (*h).td_blk = bl;
    (*h).ntl = nidx;

    sz
}

pub(crate) mod decode_pipeline {
    use super::{
        cram_cram_codecs_c_3968_cram_codec_to_id, cram_cram_io_c_145_cram_decode_compression_header,
        cram_cram_io_c_1565_cram_free_block, cram_cram_io_c_3213_cram_ref_decr,
        cram_cram_io_c_3409_cram_get_ref, cram_cram_io_c_3705_cram_free_container,
        cram_cram_io_c_3788_cram_read_container, cram_cram_io_c_4421_cram_free_slice,
        cram_cram_io_c_4568_cram_read_slice, cram_cram_io_c_1576_cram_uncompress_block,
        cram_cram_io_h_183_cram_get_block_by_id, cram_cram_io_h_216_block_resize_exact,
        cram_cram_io_h_226_block_resize, cram_cram_io_h_248_block_append,
        cram_cram_io_h_261_block_append_char, cram_cram_io_h_271_block_append_uint,
        cram_cram_io_h_340_append_uint64,
    };
    use crate::htslib_rs::c_compat::{calloc, free, malloc, realloc};
    use crate::htslib_rs::hts::{hFILE, kstring_t, BGZF};
    use crate::htslib_rs::sam::{
        bam1_t, bam_set1, sam_hdr_find_tag_id, sam_hdr_line_name, sam_hdr_t, sam_hdr_tid2name,
        sam_hrec_rg_t, sam_hrecs_sort_order, sam_hrecs_t, BAM_FMREVERSE, BAM_FMUNMAP, BAM_FPAIRED,
        BAM_FREAD1, BAM_FREVERSE, BAM_FUNMAP, ORDER_COORD,
    };
    use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

    // ---- libc shims (match the mirror's free-standing libc calls) ----
    unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
        libc::memcpy(dst, src, n)
    }
    unsafe fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
        libc::memmove(dst, src, n)
    }
    unsafe fn memset(dst: *mut c_void, v: c_int, n: usize) -> *mut c_void {
        libc::memset(dst, v, n)
    }
    unsafe fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int {
        libc::memcmp(a, b, n)
    }
    unsafe fn memchr(s: *const c_void, c: c_int, n: usize) -> *mut c_void {
        libc::memchr(s, c, n)
    }
    unsafe fn strlen(s: *const c_char) -> usize {
        libc::strlen(s)
    }
    unsafe fn strcmp(a: *const c_char, b: *const c_char) -> c_int {
        libc::strcmp(a, b)
    }
    unsafe fn pthread_mutex_lock(m: *mut libc::pthread_mutex_t) {
        libc::pthread_mutex_lock(m);
    }
    unsafe fn pthread_mutex_unlock(m: *mut libc::pthread_mutex_t) {
        libc::pthread_mutex_unlock(m);
    }
    unsafe fn hseek(fp: *mut hFILE, off: libc::off_t, whence: c_int) -> libc::off_t {
        crate::htslib_rs::hfile::hseek(fp, off, whence)
    }
    type off_t = libc::off_t;
    const SEEK_CUR: c_int = libc::SEEK_CUR;
    const INT_MAX: c_int = c_int::MAX;
    const INT64_MIN: i64 = i64::MIN;
    const UINT8_MAX: c_int = 255;
    const UINT16_MAX: c_int = 65535;

    // ---- hts_log: thin wrapper to the production formatted logger ----
    macro_rules! hts_log {
        ($sev:expr, $func:expr, $fmt:expr $(, $arg:expr)*) => {{
            // Format the message ourselves; production hts_log_cstr takes a
            // pre-formatted C string. Most CRAM hts_log() calls here are error
            // diagnostics on failure paths; we forward a static message.
            let _ = ($func, $fmt $(, $arg)*);
            super::hts_log_cstr($sev, $func, $fmt);
        }};
    }
    const HTS_LOG_ERROR: c_int = super::HTS_LOG_ERROR;
    const HTS_LOG_WARNING: c_int = super::HTS_LOG_WARNING;

    // ---- numeric type aliases used by the transpiled bodies ----
    type int32_t = i32;
    type int64_t = i64;
    type uint8_t = u8;
    type uint16_t = u16;
    type uint32_t = u32;
    type uint64_t = u64;
    type size_t = usize;
    type hts_pos_t = i64;
    type uc = c_uchar;
    type bam_seq_t = bam1_t;

    // ---- CRAM data-series IDs (cram_DS_ID, cram_structs.h:143) ----
    pub const DS_CORE: c_int = 0;
    pub const DS_aux: c_int = 1;
    pub const DS_RN: c_int = 11;
    pub const DS_QS: c_int = 12;
    pub const DS_IN: c_int = 13;
    pub const DS_SC: c_int = 14;
    pub const DS_BF: c_int = 15;
    pub const DS_CF: c_int = 16;
    pub const DS_AP: c_int = 17;
    pub const DS_RG: c_int = 18;
    pub const DS_MQ: c_int = 19;
    pub const DS_NS: c_int = 20;
    pub const DS_MF: c_int = 21;
    pub const DS_TS: c_int = 22;
    pub const DS_NP: c_int = 23;
    pub const DS_NF: c_int = 24;
    pub const DS_RL: c_int = 25;
    pub const DS_FN: c_int = 26;
    pub const DS_FC: c_int = 27;
    pub const DS_FP: c_int = 28;
    pub const DS_DL: c_int = 29;
    pub const DS_BA: c_int = 30;
    pub const DS_BS: c_int = 31;
    pub const DS_TL: c_int = 32;
    pub const DS_RI: c_int = 33;
    pub const DS_RS: c_int = 34;
    pub const DS_PD: c_int = 35;
    pub const DS_HC: c_int = 36;
    pub const DS_BB: c_int = 37;
    pub const DS_QQ: c_int = 38;
    pub const DS_TN: c_int = 39;
    pub const DS_TC: c_int = 44;
    pub const DS_END: c_int = 47;

    // ---- CRAM field bitflags (cram_fields, cram_structs.h) ----
    pub const CRAM_BF: c_int = 0x0000_0001;
    pub const CRAM_AP: c_int = 0x0000_0002;
    pub const CRAM_FP: c_int = 0x0000_0004;
    pub const CRAM_RL: c_int = 0x0000_0008;
    pub const CRAM_DL: c_int = 0x0000_0010;
    pub const CRAM_NF: c_int = 0x0000_0020;
    pub const CRAM_BA: c_int = 0x0000_0040;
    pub const CRAM_QS: c_int = 0x0000_0080;
    pub const CRAM_FC: c_int = 0x0000_0100;
    pub const CRAM_FN: c_int = 0x0000_0200;
    pub const CRAM_BS: c_int = 0x0000_0400;
    pub const CRAM_IN: c_int = 0x0000_0800;
    pub const CRAM_RG: c_int = 0x0000_1000;
    pub const CRAM_MQ: c_int = 0x0000_2000;
    pub const CRAM_TL: c_int = 0x0000_4000;
    pub const CRAM_RN: c_int = 0x0000_8000;
    pub const CRAM_NS: c_int = 0x0001_0000;
    pub const CRAM_NP: c_int = 0x0002_0000;
    pub const CRAM_TS: c_int = 0x0004_0000;
    pub const CRAM_MF: c_int = 0x0008_0000;
    pub const CRAM_CF: c_int = 0x0010_0000;
    pub const CRAM_RI: c_int = 0x0020_0000;
    pub const CRAM_RS: c_int = 0x0040_0000;
    pub const CRAM_PD: c_int = 0x0080_0000;
    pub const CRAM_HC: c_int = 0x0100_0000;
    pub const CRAM_SC: c_int = 0x0200_0000;
    pub const CRAM_BB: c_int = 0x0400_0000;
    pub const CRAM_BB_len: c_int = 0x0800_0000;
    pub const CRAM_QQ: c_int = 0x1000_0000;
    pub const CRAM_QQ_len: c_int = 0x2000_0000;
    pub const CRAM_aux: c_int = 0x4000_0000;
    pub const CRAM_ALL: c_int = 0x7fff_ffff;

    // ---- SAM fields (sam_fields, sam.h) ----
    pub const SAM_QNAME: c_int = 0x0001;
    pub const SAM_FLAG: c_int = 0x0002;
    pub const SAM_RNAME: c_int = 0x0004;
    pub const SAM_POS: c_int = 0x0008;
    pub const SAM_MAPQ: c_int = 0x0010;
    pub const SAM_CIGAR: c_int = 0x0020;
    pub const SAM_RNEXT: c_int = 0x0040;
    pub const SAM_PNEXT: c_int = 0x0080;
    pub const SAM_TLEN: c_int = 0x0100;
    pub const SAM_SEQ: c_int = 0x0200;
    pub const SAM_QUAL: c_int = 0x0400;
    pub const SAM_AUX: c_int = 0x0800;
    pub const SAM_RGAUX: c_int = 0x1000;

    // ---- CRAM cram_flags / mate flags ----
    pub const CRAM_FLAG_PRESERVE_QUAL_SCORES: c_int = 1 << 0;
    pub const CRAM_FLAG_DETACHED: c_int = 1 << 1;
    pub const CRAM_FLAG_MATE_DOWNSTREAM: c_int = 1 << 2;
    pub const CRAM_FLAG_NO_SEQ: c_int = 1 << 3;
    pub const CRAM_FLAG_EXPLICIT_TLEN: c_int = 1 << 4;
    pub const CRAM_M_REVERSE: c_int = 1;
    pub const CRAM_M_UNMAP: c_int = 2;

    // ---- cram_content_type / cram_block_method ----
    pub const FILE_HEADER: c_int = 0;
    pub const COMPRESSION_HEADER: c_int = 1;
    pub const MAPPED_SLICE: c_int = 2;
    pub const UNMAPPED_SLICE: c_int = 3;
    pub const EXTERNAL: c_int = 4;
    pub const CORE: c_int = 5;
    pub const RAW: c_int = 0;

    // ---- cram_encoding ----
    pub const E_NULL: c_uint = 0;
    pub const E_EXTERNAL: c_uint = 1;
    pub const E_BYTE_ARRAY_LEN: c_uint = 4;
    pub const E_BYTE_ARRAY_STOP: c_uint = 5;

    // ---- BAM cigar ops ----
    pub const BAM_CMATCH: c_uint = 0;
    pub const BAM_CINS: c_uint = 1;
    pub const BAM_CDEL: c_uint = 2;
    pub const BAM_CREF_SKIP: c_uint = 3;
    pub const BAM_CSOFT_CLIP: c_uint = 4;
    pub const BAM_CHARD_CLIP: c_uint = 5;
    pub const BAM_CPAD: c_uint = 6;
    type cigar_op = c_uint;
    pub const BAM_CMATCH_: cigar_op = 0;
    pub const BAM_CINS_: cigar_op = 1;
    pub const BAM_CDEL_: cigar_op = 2;
    pub const BAM_CREF_SKIP_: cigar_op = 3;
    pub const BAM_CSOFT_CLIP_: cigar_op = 4;
    pub const BAM_CHARD_CLIP_: cigar_op = 5;
    pub const BAM_CPAD_: cigar_op = 6;

    pub const CRAM_MAP_HASH: c_int = 32;

    // ---- MD5 (delegating to the native crate::htslib_rs::md5 module) ----
    pub use crate::htslib_rs::md5::hts_md5_context;
    unsafe fn hts_md5_init() -> *mut hts_md5_context {
        crate::htslib_rs::md5::hts_md5_init()
    }
    unsafe fn hts_md5_update(ctx: *mut hts_md5_context, data: *const c_void, size: u64) {
        crate::htslib_rs::md5::hts_md5_update(ctx, data, size as std::ffi::c_ulong);
    }
    unsafe fn hts_md5_final(digest: *mut c_uchar, ctx: *mut hts_md5_context) {
        crate::htslib_rs::md5::hts_md5_final(digest, ctx);
    }
    unsafe fn hts_md5_destroy(ctx: *mut hts_md5_context) {
        crate::htslib_rs::md5::hts_md5_destroy(ctx);
    }

    // ---- ks_free shim ----
    unsafe fn ks_free(s: *mut kstring_t) {
        if !(*s).s.is_null() {
            free((*s).s.cast());
            (*s).s = std::ptr::null_mut();
        }
    }

    // =======================================================================
    // Byte-identical struct re-declarations (mirror field names).
    // =======================================================================
    type cram_block_method_int = c_int;
    type cram_content_type = c_int;
    pub enum cram_metrics {}

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
        pub data: *mut c_uchar,
        pub alloc: size_t,
        pub byte: size_t,
        pub bit: c_int,
        pub m: *mut cram_metrics,
        pub crc32_checked: c_int,
        pub crc_part: uint32_t,
    }

    type cram_encoding = c_uint;
    #[repr(C)]
    pub struct cram_codec {
        pub codec: cram_encoding,
        pub out: *mut cram_block,
        pub vv: *mut c_void,
        pub codec_id: c_int,
        pub free: Option<unsafe extern "C" fn(*mut cram_codec)>,
        pub decode: Option<
            unsafe extern "C" fn(
                *mut cram_slice,
                *mut cram_codec,
                *mut cram_block,
                *mut c_char,
                *mut c_int,
            ) -> c_int,
        >,
    }

    #[repr(C)]
    pub struct cram_map {
        pub key: c_int,
        pub encoding: cram_encoding,
        pub offset: c_int,
        pub size: c_int,
        pub codec: *mut cram_codec,
        pub next: *mut cram_map,
    }

    #[repr(C)]
    pub struct cram_block_compression_hdr {
        pub ref_seq_id: i32,
        pub ref_seq_start: i64,
        pub ref_seq_span: i64,
        pub num_records: i32,
        pub num_landmarks: i32,
        pub landmark: *mut i32,
        pub read_names_included: i32,
        pub AP_delta: i32,
        pub substitution_matrix: [[c_char; 4]; 5],
        pub no_ref: i32,
        pub qs_seq_orient: i32,
        pub TD_blk: *mut cram_block,
        pub nTL: i32,
        pub TL: *mut *mut u8,
        pub TD_hash: *mut c_void,
        pub TD_keys: *mut c_void,
        pub preservation_map: *mut c_void,
        pub rec_encoding_map: [*mut cram_map; 32],
        pub tag_encoding_map: [*mut cram_map; 32],
        pub codecs: [*mut cram_codec; 47],
        pub uncomp: *mut c_char,
        pub uncomp_size: usize,
        pub uncomp_alloc: usize,
        pub ncodecs: i32,
    }

    #[repr(C)]
    pub struct cram_block_slice_hdr {
        pub content_type: cram_content_type,
        pub ref_seq_id: i32,
        pub ref_seq_start: i64,
        pub ref_seq_span: i64,
        pub num_records: i32,
        pub record_counter: i64,
        pub num_blocks: i32,
        pub num_content_ids: i32,
        pub block_content_ids: *mut i32,
        pub ref_base_id: i32,
        pub md5: [u8; 16],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub union cram_feature {
        fields: [c_int; 4],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct cram_record {
        pub s: *mut cram_slice,
        pub ref_id: i32,
        pub flags: i32,
        pub cram_flags: i32,
        pub len: i32,
        pub apos: i64,
        pub rg: i32,
        pub name: i32,
        pub name_len: i32,
        pub mate_line: i32,
        pub mate_ref_id: i32,
        pub mate_pos: i64,
        pub tlen: i64,
        pub explicit_tlen: i64,
        pub ntags: i32,
        pub aux: u32,
        pub aux_size: u32,
        pub TN_idx: i32,
        pub TL: c_int,
        pub seq: u32,
        pub qual: u32,
        pub cigar: u32,
        pub ncigar: i32,
        pub aend: i64,
        pub mqual: i32,
        pub feature: u32,
        pub nfeature: u32,
        pub mate_flags: i32,
    }

    pub enum cram_string_alloc_t {}
    #[repr(C)]
    pub struct cram_slice {
        pub hdr: *mut cram_block_slice_hdr,
        pub hdr_block: *mut cram_block,
        pub block: *mut *mut cram_block,
        pub block_by_id: *mut *mut cram_block,
        pub last_apos: i64,
        pub max_apos: i64,
        pub crecs: *mut cram_record,
        pub cigar: *mut u32,
        pub cigar_alloc: u32,
        pub ncigar: u32,
        pub features: *mut cram_feature,
        pub nfeatures: u32,
        pub afeatures: u32,
        pub TN: *mut u32,
        pub nTN: c_int,
        pub aTN: c_int,
        pub name_blk: *mut cram_block,
        pub seqs_blk: *mut cram_block,
        pub qual_blk: *mut cram_block,
        pub base_blk: *mut cram_block,
        pub soft_blk: *mut cram_block,
        pub aux_blk: *mut cram_block,
        pub pair_keys: *mut cram_string_alloc_t,
        pub pair: [*mut c_void; 2],
        pub ref_0: *mut c_char,
        pub ref_start: i64,
        pub ref_end: i64,
        pub ref_id: c_int,
        pub naux_block: c_int,
        pub aux_block: *mut *mut cram_block,
        pub data_series: c_uint,
        pub decode_md: c_int,
        pub max_rec: c_int,
        pub curr_rec: c_int,
        pub slice_num: c_int,
    }

    #[repr(C)]
    pub struct cram_container {
        pub length: i32,
        pub ref_seq_id: i32,
        pub ref_seq_start: i64,
        pub ref_seq_span: i64,
        pub record_counter: i64,
        pub num_bases: i64,
        pub num_records: i32,
        pub num_blocks: i32,
        pub num_landmarks: i32,
        pub landmark: *mut i32,
        pub offset: usize,
        pub comp_hdr: *mut cram_block_compression_hdr,
        pub comp_hdr_block: *mut cram_block,
        pub max_slice: c_int,
        pub curr_slice: c_int,
        pub curr_slice_mt: c_int,
        pub max_rec: c_int,
        pub curr_rec: c_int,
        pub max_c_rec: c_int,
        pub curr_c_rec: c_int,
        pub slice_rec: c_int,
        pub curr_ref: c_int,
        pub last_pos: i64,
        pub slices: *mut *mut cram_slice,
        pub slice: *mut cram_slice,
        pub pos_sorted: c_int,
        pub max_apos: i64,
        pub last_slice: c_int,
        pub multi_seq: c_int,
        pub unsorted: c_int,
        pub qs_seq_orient: c_int,
        pub ref_id: c_int,
        pub ref_start: i64,
        pub first_base: i64,
        pub last_base: i64,
        pub ref_end: i64,
        pub ref_0: *mut c_char,
        pub embed_ref: c_int,
        pub no_ref: c_int,
        pub bams: *mut *mut bam1_t,
        pub stats: [*mut c_void; 47],
        pub tags_used: *mut c_void,
        pub refs_used: *mut c_int,
        pub crc32: u32,
        pub s_num_bases: u64,
        pub s_aux_bytes: u64,
        pub n_mapped: u32,
        pub ref_free: c_int,
    }

    #[repr(C)]
    pub struct ref_entry {
        pub name: *mut c_char,
        pub fn_0: *mut c_char,
        pub length: i64,
        pub LN_length: i64,
        pub offset: i64,
        pub bases_per_line: c_int,
        pub line_length: c_int,
        pub count: i64,
        pub seq: *mut c_char,
        pub mf: *mut c_void,
        pub is_md5: c_int,
        pub validated_md5: c_int,
    }

    #[repr(C)]
    pub struct refs_t {
        pub pool: *mut cram_string_alloc_t,
        pub h_meta: *mut c_void,
        pub ref_id: *mut *mut ref_entry,
        pub nref: c_int,
        pub fn_0: *mut c_char,
        pub fp: *mut BGZF,
        pub count: c_int,
        pub lock: libc::pthread_mutex_t,
        pub last: *mut ref_entry,
        pub last_id: c_int,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct cram_range {
        pub refid: c_int,
        pub start: i64,
        pub end: i64,
    }

    #[repr(C)]
    pub struct cram_fd {
        pub fp: *mut hFILE,
        pub mode: c_int,
        pub version: c_int,
        pub file_def: *mut c_void,
        pub header: *mut sam_hdr_t,
        pub prefix: *mut c_char,
        pub record_counter: i64,
        pub err: c_int,
        pub ctr: *mut cram_container,
        pub ctr_mt: *mut cram_container,
        pub first_base: c_int,
        pub last_base: c_int,
        pub refs: *mut refs_t,
        pub ref_0: *mut c_char,
        pub ref_free: *mut c_char,
        pub ref_id: c_int,
        pub ref_start: i64,
        pub ref_end: i64,
        pub ref_fn: *mut c_char,
        pub level: c_int,
        pub m: [*mut c_void; 47],
        pub tags_used: *mut c_void,
        pub decode_md: c_int,
        pub seqs_per_slice: c_int,
        pub bases_per_slice: c_int,
        pub slices_per_container: c_int,
        pub embed_ref: c_int,
        pub no_ref: c_int,
        pub no_ref_counter: c_int,
        pub ignore_md5: c_int,
        pub use_bz2: c_int,
        pub use_rans: c_int,
        pub use_lzma: c_int,
        pub use_fqz: c_int,
        pub use_tok: c_int,
        pub use_arith: c_int,
        pub shared_ref: c_int,
        pub required_fields: c_uint,
        pub store_md: c_int,
        pub store_nm: c_int,
        pub range: cram_range,
        pub bam_flag_swap: [c_uint; 0x1000],
        pub cram_flag_swap: [c_uint; 0x1000],
        pub L1: [u8; 256],
        pub L2: [u8; 256],
        pub cram_sub_matrix: [[c_char; 32]; 32],
        pub index_sz: c_int,
        pub index: *mut c_void,
        pub first_container: off_t,
        pub curr_position: off_t,
        pub eof: c_int,
        pub last_slice: c_int,
        pub last_ri_count: c_int,
        pub multi_seq: c_int,
        pub multi_seq_user: c_int,
        pub unsorted: c_int,
        pub last_mapped: c_int,
        pub empty_container: c_int,
        pub own_pool: c_int,
        pub pool: *mut c_void,
        pub rqueue: *mut c_void,
        pub metrics_lock: libc::pthread_mutex_t,
        pub ref_lock: libc::pthread_mutex_t,
        pub range_lock: libc::pthread_mutex_t,
        pub bl: *mut c_void,
        pub bam_list_lock: libc::pthread_mutex_t,
        pub job_pending: *mut c_void,
        pub ooc: c_int,
        pub lossy_read_names: c_int,
        pub tlen_approx: c_int,
        pub tlen_zero: c_int,
        pub idxfp: *mut BGZF,
        pub vv: [u8; std::mem::size_of::<super::varint_vec_layout>()],
        pub ap_delta: c_int,
    }

    // Compile-time layout cross-checks: each mirror struct must be byte-for-byte
    // identical (size) to the production *_layout it shadows, since the shims
    // cast pointers between them.
    const _: () = {
        assert!(std::mem::size_of::<cram_fd>() == std::mem::size_of::<super::cram_fd_layout>());
        assert!(
            std::mem::size_of::<cram_record>() == std::mem::size_of::<super::cram_record_layout>()
        );
        assert!(
            std::mem::size_of::<cram_slice>() == std::mem::size_of::<super::cram_slice_layout>()
        );
        assert!(std::mem::size_of::<cram_block>() == std::mem::size_of::<super::cram_block_layout>());
        assert!(
            std::mem::size_of::<cram_container>()
                == std::mem::size_of::<super::cram_container_layout>()
        );
        assert!(
            std::mem::size_of::<cram_block_compression_hdr>()
                == std::mem::size_of::<super::cram_block_compression_hdr_layout>()
        );
        assert!(
            std::mem::size_of::<cram_block_slice_hdr>()
                == std::mem::size_of::<super::cram_block_slice_hdr_layout>()
        );
        assert!(
            std::mem::size_of::<ref_entry>() == std::mem::size_of::<super::ref_entry_layout>()
        );
        assert!(std::mem::size_of::<refs_t>() == std::mem::size_of::<super::refs_t_layout>());
        // Field-offset cross-checks for the fields the pipeline reads via casts.
        assert!(
            std::mem::offset_of!(cram_fd, refs) == std::mem::offset_of!(super::cram_fd_layout, refs)
        );
        assert!(
            std::mem::offset_of!(cram_fd, version)
                == std::mem::offset_of!(super::cram_fd_layout, version)
        );
        assert!(
            std::mem::offset_of!(cram_fd, required_fields)
                == std::mem::offset_of!(super::cram_fd_layout, required_fields)
        );
        assert!(std::mem::offset_of!(cram_fd, L1) == std::mem::offset_of!(super::cram_fd_layout, l1));
        assert!(
            std::mem::offset_of!(cram_fd, ref_lock)
                == std::mem::offset_of!(super::cram_fd_layout, ref_lock)
        );
        assert!(
            std::mem::offset_of!(ref_entry, offset)
                == std::mem::offset_of!(super::ref_entry_layout, offset)
        );
        assert!(
            std::mem::offset_of!(ref_entry, length)
                == std::mem::offset_of!(super::ref_entry_layout, length)
        );
        assert!(
            std::mem::offset_of!(refs_t, ref_id)
                == std::mem::offset_of!(super::refs_t_layout, ref_id)
        );
    };

    // =======================================================================
    // Dependency shims: cast our mirror pointer types to the production
    // hts_sys / layout types the native functions expect, and delegate.
    // =======================================================================
    unsafe fn cram_uncompress_block(b: *mut cram_block) -> c_int {
        cram_cram_io_c_1576_cram_uncompress_block(b.cast())
    }
    unsafe fn cram_get_block_by_id(s: *mut cram_slice, id: c_int) -> *mut cram_block {
        cram_cram_io_h_183_cram_get_block_by_id(s.cast(), id).cast()
    }
    unsafe fn block_resize_exact(b: *mut cram_block, len: size_t) -> c_int {
        cram_cram_io_h_216_block_resize_exact(b.cast(), len)
    }
    unsafe fn block_resize(b: *mut cram_block, len: size_t) -> c_int {
        cram_cram_io_h_226_block_resize(b.cast(), len)
    }
    unsafe fn block_append(b: *mut cram_block, s: *const c_void, len: size_t) -> c_int {
        cram_cram_io_h_248_block_append(b.cast(), s, len)
    }
    unsafe fn block_append_char(b: *mut cram_block, c: c_char) -> c_int {
        cram_cram_io_h_261_block_append_char(b.cast(), c)
    }
    unsafe fn block_append_uint(b: *mut cram_block, i: c_uint) -> c_int {
        cram_cram_io_h_271_block_append_uint(b.cast(), i)
    }
    unsafe fn append_uint64(cp: *mut c_uchar, i: uint64_t) -> *mut c_uchar {
        cram_cram_io_h_340_append_uint64(cp, i)
    }
    unsafe fn cram_codec_to_id(c: *mut cram_codec, id2: *mut c_int) -> c_int {
        cram_cram_codecs_c_3968_cram_codec_to_id(c.cast(), id2)
    }
    unsafe fn cram_get_ref(fd: *mut cram_fd, id: c_int, start: hts_pos_t, end: hts_pos_t) -> *mut c_char {
        cram_cram_io_c_3409_cram_get_ref(fd.cast(), id, start, end)
    }
    unsafe fn cram_ref_decr(r: *mut refs_t, id: c_int) {
        cram_cram_io_c_3213_cram_ref_decr(r.cast(), id);
    }
    unsafe fn cram_free_block(b: *mut cram_block) {
        cram_cram_io_c_1565_cram_free_block(b.cast());
    }
    unsafe fn cram_read_container(fd: *mut cram_fd) -> *mut cram_container {
        cram_cram_io_c_3788_cram_read_container(fd.cast()).cast()
    }
    unsafe fn cram_free_container(c: *mut cram_container) {
        cram_cram_io_c_3705_cram_free_container(c.cast());
    }
    unsafe fn cram_read_slice(fd: *mut cram_fd) -> *mut cram_slice {
        cram_cram_io_c_4568_cram_read_slice(fd.cast()).cast()
    }
    unsafe fn cram_free_slice(s: *mut cram_slice) {
        cram_cram_io_c_4421_cram_free_slice(s.cast());
    }
    unsafe fn cram_read_block(fd: *mut cram_fd) -> *mut cram_block {
        super::cram_read_block(fd.cast()).cast()
    }
    unsafe fn cram_seek(fd: *mut cram_fd, off: off_t, whence: c_int) -> c_int {
        super::cram_seek(fd.cast(), off, whence)
    }
    unsafe fn cram_decode_compression_header(
        fd: *mut cram_fd,
        b: *mut cram_block,
    ) -> *mut cram_block_compression_hdr {
        cram_cram_io_c_145_cram_decode_compression_header(fd.cast(), b.cast()).cast()
    }

    // ---- thread-pool shims (pool path; unexercised when fd.pool is null) ----
    use crate::htslib_rs::thread_pool::{
        hts_tpool_delete_result, hts_tpool_dispatch2, hts_tpool_next_result_wait,
        hts_tpool_process_empty, hts_tpool_process_len, hts_tpool_process_qsize,
        hts_tpool_process_sz, hts_tpool_result_data,
    };
    use crate::htslib_rs::c_compat::__errno_location;
    const EAGAIN: c_int = libc::EAGAIN;
    pub enum hts_tpool {}
    pub enum hts_tpool_process {}
    pub enum hts_tpool_result {}

    #[repr(C)]
    pub struct cram_decode_job {
        pub fd: *mut cram_fd,
        pub c: *mut cram_container,
        pub s: *mut cram_slice,
        pub h: *mut sam_hdr_t,
        pub exit_code: c_int,
    }

    // original: cram_ds_unique (htslib/cram/cram_decode.c:876)
    unsafe fn cram_ds_unique(hdr: *mut cram_block_compression_hdr, c: *mut cram_codec, id: c_int) -> c_int {
        let mut n_id: c_int = 0;
        let mut e_type: cram_encoding = E_NULL;
        let mut i = 0;
        while i < DS_END {
            let c_0: *mut cram_codec = (*hdr).codecs[i as usize];
            let mut bnum1: c_int = 0;
            let mut bnum2: c_int = 0;
            if !c_0.is_null() {
                bnum1 = cram_codec_to_id(c_0, &raw mut bnum2);
                let old_n_id = n_id;
                if bnum1 == id {
                    n_id += 1;
                    e_type = (*c_0).codec;
                }
                if bnum2 == id {
                    n_id += 1;
                    e_type = (*c_0).codec;
                }
                if n_id == old_n_id + 2 {
                    n_id -= 1;
                }
            }
            i += 1;
        }
        if n_id == 1 { e_type as c_int } else { 0 }
    }

    // original: cram_decode_estimate_sizes (htslib/cram/cram_decode.c:912)
    unsafe fn cram_decode_estimate_sizes(
        hdr: *mut cram_block_compression_hdr,
        s: *mut cram_slice,
        qual_size: *mut c_int,
        name_size: *mut c_int,
        q_id: *mut c_int,
    ) {
        *qual_size = 0;
        *name_size = 0;
        let mut cd: *mut cram_codec = (*hdr).codecs[DS_QS as usize];
        if cd.is_null() {
            return;
        }
        let mut bnum2: c_int = 0;
        let mut bnum1 = cram_codec_to_id(cd, &raw mut bnum2);
        if bnum1 < 0 && bnum2 >= 0 {
            bnum1 = bnum2;
        }
        if cram_ds_unique(hdr, cd, bnum1) != 0 {
            let b = cram_get_block_by_id(s, bnum1);
            if !b.is_null() {
                *qual_size = (*b).uncomp_size;
            }
            if !q_id.is_null() && (*cd).codec == E_EXTERNAL {
                *q_id = bnum1;
            }
        }
        cd = (*hdr).codecs[DS_RN as usize];
        if cd.is_null() {
            return;
        }
        bnum1 = cram_codec_to_id(cd, &raw mut bnum2);
        if bnum1 < 0 && bnum2 >= 0 {
            bnum1 = bnum2;
        }
        if cram_ds_unique(hdr, cd, bnum1) != 0 {
            let b = cram_get_block_by_id(s, bnum1);
            if !b.is_null() {
                *name_size = (*b).uncomp_size;
            }
        }
    }

    // original: cram_dependent_data_series (htslib/cram/cram_decode.c:773)
    unsafe fn cram_dependent_data_series(
        fd: *mut cram_fd,
        hdr: *mut cram_block_compression_hdr,
        s: *mut cram_slice,
    ) -> c_int {
        static i_to_id: [c_int; 28] = [
            DS_BF, DS_AP, DS_FP, DS_RL, DS_DL, DS_NF, DS_BA, DS_QS, DS_FC, DS_FN, DS_BS, DS_IN,
            DS_RG, DS_MQ, DS_TL, DS_RN, DS_NS, DS_NP, DS_TS, DS_MF, DS_CF, DS_RI, DS_RS, DS_PD,
            DS_HC, DS_SC, DS_BB, DS_QQ,
        ];
        let mut core_used: c_int = 0;
        if (*fd).required_fields != 0 && (*fd).required_fields != INT_MAX as c_uint {
            (*s).data_series = 0;
            if (*fd).required_fields & SAM_QNAME as c_uint != 0 {
                (*s).data_series |= CRAM_RN as c_uint;
            }
            if (*fd).required_fields & SAM_FLAG as c_uint != 0 {
                (*s).data_series |= CRAM_BF as c_uint;
            }
            if (*fd).required_fields & SAM_RNAME as c_uint != 0 {
                (*s).data_series |= (CRAM_RI | CRAM_BF) as c_uint;
            }
            if (*fd).required_fields & SAM_POS as c_uint != 0 {
                (*s).data_series |= (CRAM_AP | CRAM_BF) as c_uint;
            }
            if (*fd).required_fields & SAM_MAPQ as c_uint != 0 {
                (*s).data_series |= CRAM_MQ as c_uint;
            }
            if (*fd).required_fields & SAM_CIGAR as c_uint != 0 {
                (*s).data_series |= (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC
                    | CRAM_HC | CRAM_PD | CRAM_RS | CRAM_RL | CRAM_BF) as c_uint;
            }
            if (*fd).required_fields & SAM_RNEXT as c_uint != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF | CRAM_RI | CRAM_NS | CRAM_BF) as c_uint;
            }
            if (*fd).required_fields & SAM_PNEXT as c_uint != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF | CRAM_AP | CRAM_NP | CRAM_BF) as c_uint;
            }
            if (*fd).required_fields & SAM_TLEN as c_uint != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF | CRAM_AP | CRAM_TS | CRAM_BF | CRAM_MF
                    | CRAM_RI
                    | (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC | CRAM_HC
                        | CRAM_PD | CRAM_RS | CRAM_RL | CRAM_BF)) as c_uint;
            }
            if (*fd).required_fields & SAM_SEQ as c_uint != 0 {
                (*s).data_series |= (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC
                    | CRAM_HC | CRAM_PD | CRAM_RS | CRAM_RL | CRAM_BF | CRAM_BA | CRAM_BS | CRAM_RL
                    | CRAM_AP | CRAM_BB) as c_uint;
            }
            if (*fd).required_fields & SAM_AUX as c_uint == 0 {
                (*s).decode_md = 0;
            }
            if (*fd).required_fields & SAM_QUAL as c_uint != 0 {
                (*s).data_series |= (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC
                    | CRAM_HC | CRAM_PD | CRAM_RS | CRAM_RL | CRAM_BF | CRAM_RL | CRAM_AP | CRAM_QS
                    | CRAM_QQ) as c_uint;
            }
            if (*fd).required_fields & SAM_AUX as c_uint != 0 {
                (*s).data_series |= (CRAM_RG | CRAM_TL | CRAM_aux) as c_uint;
            }
            if (*fd).required_fields & SAM_RGAUX as c_uint != 0 {
                (*s).data_series |= (CRAM_RG | CRAM_BF) as c_uint;
            }
            if cram_uncompress_block(*(*s).block.offset(0)) != 0 {
                return -1;
            }
        } else {
            (*s).data_series = CRAM_ALL as c_uint;
            let mut i = 0;
            while i < (*(*s).hdr).num_blocks {
                if cram_uncompress_block(*(*s).block.offset(i as isize)) != 0 {
                    return -1;
                }
                i += 1;
            }
            return 0;
        }
        let block_used =
            calloc(((*(*s).hdr).num_blocks + 1) as u64, std::mem::size_of::<c_int>() as u64)
                .cast::<c_int>();
        if block_used.is_null() {
            return -1;
        }
        loop {
            if (*s).data_series & CRAM_RS as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_PD as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_HC as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_QS as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_IN as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_SC as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_BS as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_DL as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_BA as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_BB as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series & CRAM_QQ as c_uint != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as c_uint;
            }
            if (*s).data_series
                & (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC | CRAM_HC | CRAM_PD
                    | CRAM_RS | CRAM_RL | CRAM_BF | CRAM_BA | CRAM_BS | CRAM_RL | CRAM_AP | CRAM_BB
                    | (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC | CRAM_HC | CRAM_PD
                        | CRAM_RS | CRAM_RL | CRAM_BF)) as c_uint
                != 0
            {
                (*s).data_series |= CRAM_RL as c_uint;
            }
            if (*s).data_series & CRAM_FP as c_uint != 0 {
                (*s).data_series |= CRAM_FC as c_uint;
            }
            if (*s).data_series & CRAM_FC as c_uint != 0 {
                (*s).data_series |= CRAM_FN as c_uint;
            }
            if (*s).data_series & CRAM_aux as c_uint != 0 {
                (*s).data_series |= CRAM_TL as c_uint;
            }
            if (*s).data_series & CRAM_MF as c_uint != 0 {
                (*s).data_series |= CRAM_CF as c_uint;
            }
            if (*s).data_series & CRAM_MQ as c_uint != 0 {
                (*s).data_series |= CRAM_BF as c_uint;
            }
            if (*s).data_series & CRAM_BS as c_uint != 0 {
                (*s).data_series |= CRAM_RI as c_uint;
            }
            if (*s).data_series & (CRAM_MF | CRAM_NS | CRAM_NP | CRAM_TS | CRAM_NF) as c_uint != 0 {
                (*s).data_series |= CRAM_CF as c_uint;
            }
            if (*hdr).read_names_included == 0 && (*s).data_series & CRAM_RN as c_uint != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF) as c_uint;
            }
            if (*s).data_series & (CRAM_BA | CRAM_QS | CRAM_BB | CRAM_QQ) as c_uint != 0 {
                (*s).data_series |= (CRAM_BF | CRAM_CF | CRAM_RL) as c_uint;
            }
            if (*s).data_series & CRAM_FN as c_uint != 0 {
                (*s).data_series |= (CRAM_SC | CRAM_IN | CRAM_BB) as c_uint;
            }
            let orig_ds = (*s).data_series;
            let mut i = 0usize;
            while i < 28 {
                let mut bnum1: c_int;
                let mut bnum2: c_int = 0;
                let c: *mut cram_codec = (*hdr).codecs[i_to_id[i] as usize];
                if (*s).data_series & (1u32 << i) != 0 && !c.is_null() {
                    bnum1 = cram_codec_to_id(c, &raw mut bnum2);
                    loop {
                        match bnum1 {
                            -2 => {}
                            -1 => core_used = 1,
                            _ => {
                                let mut j = 0;
                                while j < (*(*s).hdr).num_blocks {
                                    if (**(*s).block.offset(j as isize)).content_type == EXTERNAL
                                        && (**(*s).block.offset(j as isize)).content_id == bnum1
                                    {
                                        *block_used.offset(j as isize) = 1;
                                        if cram_uncompress_block(*(*s).block.offset(j as isize)) != 0
                                        {
                                            free(block_used.cast());
                                            return -1;
                                        }
                                    }
                                    j += 1;
                                }
                            }
                        }
                        if bnum2 == -2 || bnum1 == bnum2 {
                            break;
                        }
                        bnum1 = bnum2;
                    }
                }
                i += 1;
            }
            if (*fd).required_fields & SAM_AUX as c_uint != 0
                || (*s).data_series & CRAM_aux as c_uint != 0
            {
                let mut i = 0;
                while i < CRAM_MAP_HASH {
                    let mut m: *mut cram_map = (*hdr).tag_encoding_map[i as usize];
                    while !m.is_null() {
                        let c_0: *mut cram_codec = (*m).codec;
                        if c_0.is_null() {
                            m = (*m).next;
                            continue;
                        }
                        let mut bnum2: c_int = 0;
                        let mut bnum1 = cram_codec_to_id(c_0, &raw mut bnum2);
                        loop {
                            match bnum1 {
                                -2 => {}
                                -1 => core_used = 1,
                                _ => {
                                    let mut j = 0;
                                    while j < (*(*s).hdr).num_blocks {
                                        if (**(*s).block.offset(j as isize)).content_type == EXTERNAL
                                            && (**(*s).block.offset(j as isize)).content_id == bnum1
                                        {
                                            *block_used.offset(j as isize) = 1;
                                            if cram_uncompress_block(*(*s).block.offset(j as isize))
                                                != 0
                                            {
                                                free(block_used.cast());
                                                return -1;
                                            }
                                        }
                                        j += 1;
                                    }
                                }
                            }
                            if bnum2 == -2 || bnum1 == bnum2 {
                                break;
                            }
                            bnum1 = bnum2;
                        }
                        m = (*m).next;
                    }
                    i += 1;
                }
            }
            let mut i = 0usize;
            while i < 28 {
                let mut bnum2: c_int = 0;
                let c_1: *mut cram_codec = (*hdr).codecs[i_to_id[i] as usize];
                if !c_1.is_null() {
                    let mut bnum1 = cram_codec_to_id(c_1, &raw mut bnum2);
                    loop {
                        match bnum1 {
                            -2 => {}
                            -1 => {
                                if core_used != 0 {
                                    (*s).data_series |= 1u32 << i;
                                }
                            }
                            _ => {
                                let mut j = 0;
                                while j < (*(*s).hdr).num_blocks {
                                    if (**(*s).block.offset(j as isize)).content_type == EXTERNAL
                                        && (**(*s).block.offset(j as isize)).content_id == bnum1
                                        && *block_used.offset(j as isize) != 0
                                    {
                                        (*s).data_series |= 1u32 << i;
                                    }
                                    j += 1;
                                }
                            }
                        }
                        if bnum2 == -2 || bnum1 == bnum2 {
                            break;
                        }
                        bnum1 = bnum2;
                    }
                }
                i += 1;
            }
            let mut i = 0;
            while i < CRAM_MAP_HASH {
                let mut m_0: *mut cram_map = (*hdr).tag_encoding_map[i as usize];
                while !m_0.is_null() {
                    let c_2: *mut cram_codec = (*m_0).codec;
                    if c_2.is_null() {
                        m_0 = (*m_0).next;
                        continue;
                    }
                    let mut bnum2: c_int = 0;
                    let mut bnum1 = cram_codec_to_id(c_2, &raw mut bnum2);
                    loop {
                        match bnum1 {
                            -2 => {}
                            -1 => (*s).data_series |= CRAM_aux as c_uint,
                            _ => {
                                let mut j = 0;
                                while j < (*(*s).hdr).num_blocks {
                                    if (**(*s).block.offset(j as isize)).content_type == EXTERNAL
                                        && (**(*s).block.offset(j as isize)).content_id == bnum1
                                        && *block_used.offset(j as isize) != 0
                                    {
                                        (*s).data_series |= CRAM_aux as c_uint;
                                    }
                                    j += 1;
                                }
                            }
                        }
                        if bnum2 == -2 || bnum1 == bnum2 {
                            break;
                        }
                        bnum1 = bnum2;
                    }
                    m_0 = (*m_0).next;
                }
                i += 1;
            }
            if orig_ds == (*s).data_series {
                break;
            }
        }
        free(block_used.cast());
        0
    }

    // original: add_md_char (htslib/cram/cram_decode.c:1080)
    #[inline]
    unsafe fn add_md_char(
        s: *mut cram_slice,
        decode_md: c_int,
        c: c_char,
        md_dist: *mut int32_t,
    ) -> c_int {
        if decode_md != 0 {
            if block_append_uint((*s).aux_blk, *md_dist as c_uint) < 0 {
                return -1;
            }
            if block_append_char((*s).aux_blk, c) < 0 {
                return -1;
            }
            *md_dist = 0;
        }
        0
    }

    // original: map_find (htslib/cram/cram_decode.c:1926)
    unsafe fn map_find(map: *mut *mut cram_map, key: *mut c_uchar, id: c_int) -> *mut cram_map {
        let mut m: *mut cram_map = *map.offset(
            ((*key.offset(0) as c_int * 3 + *key.offset(1) as c_int) & (CRAM_MAP_HASH - 1)) as isize,
        );
        while !m.is_null() && (*m).key != id {
            m = (*m).next;
        }
        m
    }

    // original: aux_ele_size (htslib/cram/cram_decode.c:1989)
    #[inline]
    unsafe fn aux_ele_size(type_0: uint8_t) -> c_int {
        match type_0 as c_int {
            65 | 99 | 67 => 1,
            115 | 83 => 2,
            105 | 73 | 102 => 4,
            100 => 8,
            _ => 1,
        }
    }

    // original: md5_print (htslib/cram/cram_decode.c:2305)
    unsafe fn md5_print(md5: *mut c_uchar, out: *mut c_char) -> *mut c_char {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut i = 0;
        while i < 16 {
            *out.offset((i * 2) as isize) =
                HEX[(*md5.offset(i as isize) as c_int >> 4) as usize] as c_char;
            *out.offset((i * 2 + 1) as isize) =
                HEX[(*md5.offset(i as isize) as c_int & 15) as usize] as c_char;
            i += 1;
        }
        *out.offset(32) = 0;
        out
    }

    // original: cram_decode_tlen (htslib/cram/cram_decode.c:2322)
    unsafe fn cram_decode_tlen(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        blk: *mut cram_block,
        tlen: *mut int64_t,
    ) -> c_int {
        let mut out_sz: c_int = 1;
        let mut r: c_int = 0;
        if (*(*c).comp_hdr).codecs[DS_TS as usize].is_null() {
            return -1;
        }
        if (*fd).version >> 8 < 4 {
            let mut i32: int32_t = 0;
            r |= (*(*(*c).comp_hdr).codecs[DS_TS as usize]).decode.expect("decode")(
                s,
                (*(*c).comp_hdr).codecs[DS_TS as usize],
                blk,
                &raw mut i32 as *mut c_char,
                &raw mut out_sz,
            );
            *tlen = i32 as int64_t;
        } else {
            r |= (*(*(*c).comp_hdr).codecs[DS_TS as usize]).decode.expect("decode")(
                s,
                (*(*c).comp_hdr).codecs[DS_TS as usize],
                blk,
                tlen as *mut c_char,
                &raw mut out_sz,
            );
        }
        r
    }

    // original: cram_decode_aux_1_0 (htslib/cram/cram_decode.c:1939)
    unsafe fn cram_decode_aux_1_0(
        c: *mut cram_container,
        s: *mut cram_slice,
        blk: *mut cram_block,
        cr: *mut cram_record,
    ) -> c_int {
        let mut r: c_int = 0;
        let mut out_sz: c_int = 1;
        let mut ntags: c_uchar = 0;
        if (*(*c).comp_hdr).codecs[DS_TC as usize].is_null() {
            return -1;
        }
        r |= (*(*(*c).comp_hdr).codecs[DS_TC as usize]).decode.expect("decode")(
            s,
            (*(*c).comp_hdr).codecs[DS_TC as usize],
            blk,
            &raw mut ntags as *mut c_char,
            &raw mut out_sz,
        );
        (*cr).ntags = ntags as int32_t;
        (*cr).aux_size = 0;
        (*cr).aux = (*(*s).aux_blk).byte as uint32_t;
        let mut i = 0;
        while i < (*cr).ntags {
            let mut id: int32_t = 0;
            let mut out_sz_0: int32_t = 1;
            let mut tag_data: [c_uchar; 3] = [0; 3];
            if (*(*c).comp_hdr).codecs[DS_TN as usize].is_null() {
                return -1;
            }
            r |= (*(*(*c).comp_hdr).codecs[DS_TN as usize]).decode.expect("decode")(
                s,
                (*(*c).comp_hdr).codecs[DS_TN as usize],
                blk,
                &raw mut id as *mut c_char,
                &raw mut out_sz_0,
            );
            if out_sz_0 == 3 {
                memcpy(
                    &raw mut tag_data as *mut c_uchar as *mut c_void,
                    &raw mut id as *const c_void,
                    3,
                );
            } else {
                tag_data[0] = (id >> 16 & 0xff) as c_uchar;
                tag_data[1] = (id >> 8 & 0xff) as c_uchar;
                tag_data[2] = (id & 0xff) as c_uchar;
            }
            let m = map_find(
                &raw mut (*(*c).comp_hdr).tag_encoding_map as *mut *mut cram_map,
                &raw mut tag_data as *mut c_uchar,
                id,
            );
            if m.is_null() {
                return -1;
            }
            if block_append((*s).aux_blk, &raw mut tag_data as *mut c_uchar as *const c_void, 3) < 0
            {
                return -1;
            }
            if (*m).codec.is_null() {
                return -1;
            }
            r |= (*(*m).codec).decode.expect("decode")(
                s,
                (*m).codec,
                blk,
                (*s).aux_blk as *mut c_char,
                &raw mut out_sz_0,
            );
            (*cr).aux_size = (*cr).aux_size.wrapping_add((out_sz_0 + 3) as c_uint);
            i += 1;
        }
        r
    }

    // original: cram_decode_aux (htslib/cram/cram_decode.c:2008)
    unsafe fn cram_decode_aux(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        blk: *mut cram_block,
        cr: *mut cram_record,
        has_MD: *mut c_int,
        has_NM: *mut c_int,
    ) -> c_int {
        let mut r: c_int = 0;
        let mut out_sz: c_int = 1;
        let mut TL: int32_t = 0;
        let ds: uint32_t = (*s).data_series;
        if ds & (CRAM_TL | CRAM_aux) as c_uint == 0 {
            (*cr).aux = 0;
            (*cr).aux_size = 0;
            return 0;
        }
        if (*(*c).comp_hdr).codecs[DS_TL as usize].is_null() {
            return -1;
        }
        r |= (*(*(*c).comp_hdr).codecs[DS_TL as usize]).decode.expect("decode")(
            s,
            (*(*c).comp_hdr).codecs[DS_TL as usize],
            blk,
            &raw mut TL as *mut c_char,
            &raw mut out_sz,
        );
        if r != 0 || TL < 0 || TL >= (*(*c).comp_hdr).nTL {
            return -1;
        }
        let mut TN: *mut c_uchar = *(*(*c).comp_hdr).TL.offset(TL as isize);
        (*cr).ntags = (strlen(TN as *mut c_char) / 3) as int32_t;
        (*cr).aux_size = 0;
        (*cr).aux = (*(*s).aux_blk).byte as uint32_t;
        if ds & CRAM_aux as c_uint == 0 {
            return 0;
        }
        let mut i = 0;
        while i < (*cr).ntags {
            let mut id: int32_t;
            let mut out_sz_0: int32_t = 1;
            let mut tag_data: [c_uchar; 7] = [0; 7];
            if *TN.offset(0) as c_int == 'M' as i32
                && *TN.offset(1) as c_int == 'D' as i32
                && !has_MD.is_null()
            {
                *has_MD = (*(*s).aux_blk).byte.wrapping_add(3).wrapping_mul(
                    (if *TN.offset(2) as c_int == '*' as i32 { -1i32 } else { 1 }) as size_t,
                ) as c_int;
            }
            if *TN.offset(0) as c_int == 'N' as i32
                && *TN.offset(1) as c_int == 'M' as i32
                && !has_NM.is_null()
            {
                *has_NM = (*(*s).aux_blk).byte.wrapping_add(3).wrapping_mul(
                    (if *TN.offset(2) as c_int == '*' as i32 { -1i32 } else { 1 }) as size_t,
                ) as c_int;
            }
            tag_data[0] = *TN.offset(0);
            tag_data[1] = *TN.offset(1);
            tag_data[2] = *TN.offset(2);
            id = ((tag_data[0] as c_int) << 16 | (tag_data[1] as c_int) << 8 | tag_data[2] as c_int)
                as int32_t;
            if (*fd).version >> 8 >= 4 && *TN.offset(2) as c_int == '*' as i32 {
                let mut tag_data_size: c_int = 0;
                let mut handled = false;
                if *TN.offset(0) as c_int == 'N' as i32 && *TN.offset(1) as c_int == 'M' as i32 {
                    memcpy(
                        (&raw mut tag_data as *mut c_uchar).offset(2) as *mut c_void,
                        b"I\0\0\0\0\0" as *const u8 as *const c_char as *const c_void,
                        5,
                    );
                    tag_data_size = 7;
                } else if *TN.offset(0) as c_int == 'R' as i32
                    && *TN.offset(1) as c_int == 'G' as i32
                {
                    TN = TN.offset(3);
                    let rg = sam_hdr_line_name(
                        (*fd).header,
                        b"RG\0" as *const u8 as *const c_char,
                        (*cr).rg,
                    );
                    if !rg.is_null() {
                        let rg_len = strlen(rg);
                        tag_data[2] = 'Z' as i32 as c_uchar;
                        if block_append(
                            (*s).aux_blk,
                            &raw mut tag_data as *mut c_uchar as *const c_void,
                            3,
                        ) < 0
                        {
                            return -1;
                        }
                        if block_append((*s).aux_blk, rg as *const c_void, rg_len) < 0 {
                            return -1;
                        }
                        if block_append_char((*s).aux_blk, 0) < 0 {
                            return -1;
                        }
                        (*cr).aux_size =
                            (*cr).aux_size.wrapping_add((3 + rg_len + 1) as c_uint);
                        (*cr).rg = -1;
                    }
                    handled = true;
                } else {
                    tag_data[2] = 'Z' as i32 as c_uchar;
                    tag_data_size = 3;
                }
                if !handled {
                    if block_append(
                        (*s).aux_blk,
                        &raw mut tag_data as *mut c_uchar as *const c_void,
                        tag_data_size as size_t,
                    ) < 0
                    {
                        return -1;
                    }
                    (*cr).aux_size = (*cr).aux_size.wrapping_add(tag_data_size as c_uint);
                    TN = TN.offset(3);
                }
            } else {
                TN = TN.offset(3);
                let m = map_find(
                    &raw mut (*(*c).comp_hdr).tag_encoding_map as *mut *mut cram_map,
                    &raw mut tag_data as *mut c_uchar,
                    id,
                );
                if m.is_null() {
                    return -1;
                }
                if block_append(
                    (*s).aux_blk,
                    &raw mut tag_data as *mut c_uchar as *const c_void,
                    3,
                ) < 0
                {
                    return -1;
                }
                if (*m).codec.is_null() {
                    return -1;
                }
                if (*(*m).codec).codec == E_BYTE_ARRAY_LEN || (*(*m).codec).codec == E_BYTE_ARRAY_STOP
                {
                    out_sz_0 = out_sz_0 * aux_ele_size(*TN.offset(-1) as uint8_t);
                }
                r |= (*(*m).codec).decode.expect("decode")(
                    s,
                    (*m).codec,
                    blk,
                    (*s).aux_blk as *mut c_char,
                    &raw mut out_sz_0,
                );
                if r != 0 {
                    return r;
                }
                (*cr).aux_size = (*cr).aux_size.wrapping_add((out_sz_0 + 3) as c_uint);
                if *TN.offset(-3) as c_int == 'c' as i32
                    && *TN.offset(-2) as c_int == 'F' as i32
                    && *TN.offset(-1) as c_int == 'C' as i32
                    && out_sz_0 == 1
                {
                    let cF: uint8_t =
                        *((*(*s).aux_blk).data.offset((*(*s).aux_blk).byte as isize)).offset(-1);
                    (*(*s).aux_blk).byte = (*(*s).aux_blk).byte.wrapping_sub((out_sz_0 + 3) as size_t);
                    (*cr).aux_size = (*cr).aux_size.wrapping_sub((out_sz_0 + 3) as c_uint);
                    if cF as c_int & 1 != 0 && !has_MD.is_null() && *has_MD == 0 {
                        *has_MD = 1;
                    }
                    if cF as c_int & 2 != 0 && !has_NM.is_null() && *has_NM == 0 {
                        *has_NM = 1;
                    }
                }
            }
            if (*(*s).aux_blk).byte > (1u32 << 31) as size_t {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_decode_aux\0" as *const u8 as *const c_char,
                    b"CRAM->BAM aux block size overflow\0" as *const u8 as *const c_char
                );
                return -1;
            }
            i += 1;
        }
        r
    }

    // original: cram_decode_slice_xref (htslib/cram/cram_decode.c:2140)
    unsafe fn cram_decode_slice_xref(s: *mut cram_slice, required_fields: c_int) -> c_int {
        if required_fields & (SAM_RNEXT | SAM_PNEXT | SAM_TLEN) == 0 {
            let mut rec = 0;
            while rec < (*(*s).hdr).num_records {
                let cr = (*s).crecs.offset(rec as isize);
                (*cr).tlen = 0;
                (*cr).mate_pos = 0;
                (*cr).mate_ref_id = -1;
                rec += 1;
            }
            return 0;
        }
        let mut rec = 0;
        while rec < (*(*s).hdr).num_records {
            let cr_0 = (*s).crecs.offset(rec as isize);
            if (*cr_0).mate_line >= 0 {
                if (*cr_0).mate_line < (*(*s).hdr).num_records {
                    if (*cr_0).tlen == INT64_MIN {
                        let id1 = rec;
                        let mut id2 = rec;
                        let mut aleft = (*cr_0).apos;
                        let mut aright = (*cr_0).aend;
                        let mut tlen: int64_t;
                        let mut ref_0 = (*cr_0).ref_id;
                        let mut left_cnt = 0;
                        let mut right_cnt = 0;
                        loop {
                            if aleft > (*(*s).crecs.offset(id2 as isize)).apos {
                                aleft = (*(*s).crecs.offset(id2 as isize)).apos;
                                left_cnt = 1;
                            } else if aleft == (*(*s).crecs.offset(id2 as isize)).apos {
                                left_cnt += 1;
                            }
                            if aright < (*(*s).crecs.offset(id2 as isize)).aend {
                                aright = (*(*s).crecs.offset(id2 as isize)).aend;
                                right_cnt = 1;
                            } else if aright == (*(*s).crecs.offset(id2 as isize)).aend {
                                right_cnt += 1;
                            }
                            if (*(*s).crecs.offset(id2 as isize)).mate_line == -1 {
                                (*(*s).crecs.offset(id2 as isize)).mate_line = rec;
                                break;
                            } else {
                                if (*(*s).crecs.offset(id2 as isize)).mate_line <= id2
                                    || (*(*s).crecs.offset(id2 as isize)).mate_line
                                        >= (*(*s).hdr).num_records
                                {
                                    return -1;
                                }
                                id2 = (*(*s).crecs.offset(id2 as isize)).mate_line;
                                if (*(*s).crecs.offset(id2 as isize)).ref_id != ref_0 {
                                    ref_0 = -1;
                                }
                                if id2 == id1 {
                                    break;
                                }
                            }
                        }
                        if ref_0 != -1 {
                            tlen = aright - aleft + 1;
                            id2 = rec;
                            if (*(*s).crecs.offset(id2 as isize)).apos == aleft
                                && ((*(*s).crecs.offset(id2 as isize)).aend < aright || left_cnt <= 1)
                            {
                                (*(*s).crecs.offset(id2 as isize)).tlen = tlen;
                                tlen = -tlen;
                            } else if (*(*s).crecs.offset(id2 as isize)).apos == aleft
                                && (*(*s).crecs.offset(id2 as isize)).aend == aright
                                && left_cnt > 1
                                && right_cnt > 1
                            {
                                if (*(*s).crecs.offset(id2 as isize)).flags & BAM_FREAD1 != 0 {
                                    (*(*s).crecs.offset(id2 as isize)).tlen = tlen;
                                    tlen = -tlen;
                                } else {
                                    (*(*s).crecs.offset(id2 as isize)).tlen = -tlen;
                                }
                            } else {
                                (*(*s).crecs.offset(id2 as isize)).tlen = -tlen;
                            }
                            id2 = (*(*s).crecs.offset(id2 as isize)).mate_line;
                            while id2 != id1 {
                                (*(*s).crecs.offset(id2 as isize)).tlen = tlen;
                                id2 = (*(*s).crecs.offset(id2 as isize)).mate_line;
                            }
                        } else {
                            id2 = rec;
                            (*(*s).crecs.offset(id2 as isize)).tlen = 0;
                            id2 = (*(*s).crecs.offset(id2 as isize)).mate_line;
                            while id2 != id1 {
                                (*(*s).crecs.offset(id2 as isize)).tlen = 0;
                                id2 = (*(*s).crecs.offset(id2 as isize)).mate_line;
                            }
                        }
                    }
                    (*cr_0).mate_pos = (*(*s).crecs.offset((*cr_0).mate_line as isize)).apos;
                    (*cr_0).mate_ref_id = (*(*s).crecs.offset((*cr_0).mate_line as isize)).ref_id;
                    (*cr_0).flags |= BAM_FPAIRED;
                    if (*(*s).crecs.offset((*cr_0).mate_line as isize)).flags & BAM_FUNMAP != 0 {
                        (*cr_0).flags |= BAM_FMUNMAP;
                        (*cr_0).tlen = 0;
                    }
                    if (*cr_0).flags & BAM_FUNMAP != 0 {
                        (*cr_0).tlen = 0;
                    }
                    if (*(*s).crecs.offset((*cr_0).mate_line as isize)).flags & BAM_FREVERSE != 0 {
                        (*cr_0).flags |= BAM_FMREVERSE;
                    }
                } else {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice_xref\0" as *const u8 as *const c_char,
                        b"Mate line out of bounds\0" as *const u8 as *const c_char
                    );
                }
            } else {
                if (*cr_0).mate_flags & CRAM_M_REVERSE != 0 {
                    (*cr_0).flags |= BAM_FPAIRED | BAM_FMREVERSE;
                }
                if (*cr_0).mate_flags & CRAM_M_UNMAP != 0 {
                    (*cr_0).flags |= BAM_FMUNMAP;
                }
                if (*cr_0).flags & BAM_FPAIRED == 0 {
                    (*cr_0).mate_ref_id = -1;
                }
            }
            if (*cr_0).tlen == INT64_MIN {
                (*cr_0).tlen = 0;
            }
            rec += 1;
        }
        let mut rec = 0;
        while rec < (*(*s).hdr).num_records {
            let cr_1 = (*s).crecs.offset(rec as isize);
            if (*cr_1).explicit_tlen != INT64_MIN {
                (*cr_1).tlen = (*cr_1).explicit_tlen;
            }
            rec += 1;
        }
        0
    }

    // original: cram_decode_seq (htslib/cram/cram_decode.c:1096)
    // Faithful translation of the C2Rust `current_block` goto state machine.
    unsafe fn cram_decode_seq(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        blk: *mut cram_block,
        cr: *mut cram_record,
        sh: *mut sam_hdr_t,
        cf: c_int,
        seq: *mut c_char,
        qual: *mut c_char,
        mut has_MD: c_int,
        mut has_NM: c_int,
    ) -> c_int {
        let mut current_block: u64;
        let mut prev_pos: c_int = 0;
        let mut f: c_int;
        let mut r: c_int = 0;
        let mut out_sz: c_int = 1;
        let mut seq_pos: c_int = 1;
        let mut cig_len: c_int = 0;
        let mut ref_pos: int64_t = (*cr).apos;
        let mut fn_0: int32_t = 0;
        let mut i32: int32_t = 0;
        let mut cig_op: cigar_op = BAM_CMATCH_;
        let mut cigar: *mut uint32_t = (*s).cigar;
        let mut ncigar: uint32_t = (*s).ncigar;
        let mut cigar_alloc: uint32_t = (*s).cigar_alloc;
        let mut nm: uint32_t = 0;
        let mut md_dist: int32_t = 0;
        let mut orig_aux: c_int = 0;
        let do_md: c_int = if (*fd).version >> 8 >= 4 {
            ((*s).decode_md > 0) as c_int
        } else {
            ((*s).decode_md != 0) as c_int
        };
        let mut decode_md: c_int = (!(*s).ref_0.is_null()
            && (*cr).ref_id >= 0
            && (do_md != 0 && has_MD == 0 || has_MD < 0)) as c_int;
        let mut decode_nm: c_int = (!(*s).ref_0.is_null()
            && (*cr).ref_id >= 0
            && (do_md != 0 && has_NM == 0 || has_NM < 0)) as c_int;
        let ds: uint32_t = (*s).data_series;
        let bfd: *mut sam_hrecs_t = (*sh).hrecs;
        let comp = (*c).comp_hdr;
        macro_rules! codec {
            ($id:expr) => {
                (*comp).codecs[$id as usize]
            };
        }
        macro_rules! decode {
            ($id:expr, $out:expr, $sz:expr) => {{
                (*codec!($id)).decode.expect("decode")(s, codec!($id), blk, $out, $sz)
            }};
        }
        if ds & CRAM_QS as c_uint != 0 && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0 {
            memset(qual as *mut c_void, 255, (*cr).len as size_t);
        }
        if (*cr).cram_flags & CRAM_FLAG_NO_SEQ != 0 {
            decode_nm = 0;
            decode_md = 0;
        }
        if decode_md != 0 {
            orig_aux = (*(*s).aux_blk).byte as c_int;
            if has_MD == 0
                && block_append(
                    (*s).aux_blk,
                    b"MDZ\0" as *const u8 as *const c_char as *const c_void,
                    3,
                ) < 0
            {
                return -1;
            }
        }
        // -- feature loop --
        if ds & CRAM_FN as c_uint != 0 {
            if codec!(DS_FN).is_null() {
                return -1;
            }
            r |= decode!(DS_FN, &raw mut fn_0 as *mut c_char, &raw mut out_sz);
            if r != 0 {
                return r;
            }
        } else {
            fn_0 = 0;
        }
        ref_pos -= 1;
        (*cr).cigar = ncigar;
        let mut cigar_extends_error = false;
        'feature_done: {
            if ds & (CRAM_FC | CRAM_FP) as c_uint == 0 {
                break 'feature_done;
            }
            if fn_0 != 0 {
                if ds & CRAM_FC as c_uint != 0 && codec!(DS_FC).is_null() {
                    return -1;
                }
                if ds & CRAM_FP as c_uint != 0 && codec!(DS_FP).is_null() {
                    return -1;
                }
            }
            f = 0;
            while f < fn_0 {
                let mut pos: int32_t = 0;
                let mut op: c_char = 0;
                if ncigar.wrapping_add(2) >= cigar_alloc {
                    cigar_alloc = if cigar_alloc != 0 { cigar_alloc.wrapping_mul(2) } else { 1024 };
                    cigar = realloc(
                        (*s).cigar as *mut c_void,
                        cigar_alloc as u64 * std::mem::size_of::<uint32_t>() as u64,
                    )
                    .cast::<uint32_t>();
                    if cigar.is_null() {
                        return -1;
                    }
                    (*s).cigar = cigar;
                }
                if ds & CRAM_FC as c_uint != 0 {
                    r |= decode!(DS_FC, &raw mut op, &raw mut out_sz);
                    if r != 0 {
                        return r;
                    }
                }
                if ds & CRAM_FP as c_uint != 0 {
                    r |= decode!(DS_FP, &raw mut pos as *mut c_char, &raw mut out_sz);
                    if r != 0 {
                        return r;
                    }
                    pos += prev_pos;
                    if pos <= 0 {
                        hts_log!(
                            HTS_LOG_ERROR,
                            b"cram_decode_seq\0" as *const u8 as *const c_char,
                            b"Feature position before start of read\0" as *const u8 as *const c_char
                        );
                        return -1;
                    }
                    if (*cr).len != 0 && pos > (*cr).len {
                        let valid_end = if op as c_int == 'N' as i32
                            || op as c_int == 'P' as i32
                            || op as c_int == 'H' as i32
                            || op as c_int == 'D' as i32
                        {
                            (*cr).len + 1
                        } else {
                            (*cr).len
                        };
                        if pos > valid_end {
                            hts_log!(
                                HTS_LOG_ERROR,
                                b"cram_decode_seq\0" as *const u8 as *const c_char,
                                b"Feature position after end of read\0" as *const u8
                                    as *const c_char
                            );
                            return -1;
                        }
                    }
                    if pos > seq_pos {
                        if !(*s).ref_0.is_null() && (*cr).ref_id >= 0 {
                            if ref_pos + pos as int64_t - seq_pos as int64_t
                                > (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                            {
                                let rlen = ((*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                    - ref_pos) as c_int;
                                if rlen > 0 {
                                    if ref_pos + rlen as int64_t > (*s).ref_end {
                                        cigar_extends_error = true;
                                        break;
                                    }
                                    if (*cr).len != 0 {
                                        memcpy(
                                            seq.offset((seq_pos - 1) as isize) as *mut c_void,
                                            (*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                                as *const c_void,
                                            rlen as size_t,
                                        );
                                        if pos - seq_pos - rlen > 0 {
                                            memset(
                                                seq.offset((seq_pos - 1 + rlen) as isize)
                                                    as *mut c_void,
                                                'N' as i32,
                                                (pos - seq_pos - rlen) as size_t,
                                            );
                                        }
                                    }
                                } else if (*cr).len != 0 {
                                    memset(
                                        seq.offset((seq_pos - 1) as isize) as *mut c_void,
                                        'N' as i32,
                                        ((*cr).len - seq_pos + 1) as size_t,
                                    );
                                }
                                if md_dist >= 0 {
                                    md_dist += pos - seq_pos;
                                }
                            } else {
                                if ref_pos + pos as int64_t - seq_pos as int64_t > (*s).ref_end {
                                    cigar_extends_error = true;
                                    break;
                                }
                                let refp: *const c_char =
                                    (*s).ref_0.offset(ref_pos as isize).offset(-((*s).ref_start as isize)).offset(1);
                                let frag_len = pos - seq_pos;
                                if decode_md != 0 || decode_nm != 0 {
                                    let n = memchr(refp as *const c_void, 'N' as i32, frag_len as size_t)
                                        as *mut c_char;
                                    if !n.is_null() {
                                        let mut i = 0;
                                        while i < frag_len {
                                            let base = *refp.offset(i as isize);
                                            if base as c_int == 'N' as i32 {
                                                if add_md_char(s, decode_md, 'N' as i32 as c_char, &raw mut md_dist) < 0 {
                                                    return -1;
                                                }
                                                nm = nm.wrapping_add(1);
                                            } else {
                                                md_dist += 1;
                                            }
                                            i += 1;
                                        }
                                    } else {
                                        md_dist += frag_len;
                                    }
                                }
                                if (*cr).len != 0 {
                                    memcpy(
                                        seq.offset((seq_pos - 1) as isize) as *mut c_void,
                                        refp as *const c_void,
                                        frag_len as size_t,
                                    );
                                }
                            }
                        }
                        if cig_len != 0 && cig_op != BAM_CMATCH {
                            let t = ncigar;
                            ncigar = ncigar.wrapping_add(1);
                            *cigar.offset(t as isize) =
                                ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                            cig_len = 0;
                        }
                        cig_op = BAM_CMATCH_;
                        cig_len += pos - seq_pos;
                        ref_pos += (pos - seq_pos) as int64_t;
                        seq_pos = pos;
                    }
                    prev_pos = pos;
                    if ds & CRAM_FC as c_uint == 0 {
                        break 'feature_done;
                    }
                    match op as c_int {
                        83 => {
                            // 'S'
                            let mut out_sz2: int32_t =
                                if (*cr).len != 0 { (*cr).len - (pos - 1) } else { 1 };
                            let mut have_sc = 0;
                            if cig_len != 0 {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            let seqp = || {
                                if (*cr).len != 0 {
                                    seq.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                }
                            };
                            if (*fd).version >> 8 == 1 {
                                if ds & CRAM_IN as c_uint != 0 {
                                    if !codec!(DS_IN).is_null() {
                                        r |= decode!(DS_IN, seqp(), &raw mut out_sz2);
                                    } else {
                                        if (*cr).len != 0 {
                                            *seq.offset((pos - 1) as isize) = 'N' as i32 as c_char;
                                        }
                                        out_sz2 = 1;
                                    }
                                    have_sc = 1;
                                }
                            } else if ds & CRAM_SC as c_uint != 0 {
                                if !codec!(DS_SC).is_null() {
                                    r |= decode!(DS_SC, seqp(), &raw mut out_sz2);
                                } else {
                                    if (*cr).len != 0 {
                                        *seq.offset((pos - 1) as isize) = 'N' as i32 as c_char;
                                    }
                                    out_sz2 = 1;
                                }
                                have_sc = 1;
                            }
                            if have_sc != 0 {
                                if r != 0 {
                                    return r;
                                }
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((out_sz2 << 4) + BAM_CSOFT_CLIP as int32_t) as uint32_t;
                                cig_op = BAM_CSOFT_CLIP_;
                                seq_pos += out_sz2;
                            }
                        }
                        88 => {
                            // 'X'
                            let mut base_0: c_uchar = 0;
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BS as c_uint != 0 {
                                if codec!(DS_BS).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_BS, &raw mut base_0 as *mut c_char, &raw mut out_sz);
                                if r != 0 {
                                    return -1;
                                }
                                if (*cr).ref_id < 0
                                    || ref_pos >= (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                    || (*s).ref_0.is_null()
                                {
                                    if pos - 1 < (*cr).len {
                                        *seq.offset((pos - 1) as isize) = (*comp).substitution_matrix
                                            [(*fd).L1['N' as i32 as usize] as usize]
                                            [base_0 as usize];
                                    }
                                    if decode_md != 0 || decode_nm != 0 {
                                        if md_dist >= 0 && decode_md != 0 && block_append_uint((*s).aux_blk, md_dist as c_uint) < 0 {
                                            return -1;
                                        }
                                        md_dist = -1;
                                        nm = nm.wrapping_sub(1);
                                    }
                                } else {
                                    let ref_call: c_uchar = (if ref_pos < (*s).ref_end {
                                        *(*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                            as uc as c_int
                                    } else {
                                        'N' as i32
                                    }) as c_uchar;
                                    let ref_base = (*fd).L1[ref_call as usize] as c_int;
                                    if pos - 1 < (*cr).len {
                                        *seq.offset((pos - 1) as isize) =
                                            (*comp).substitution_matrix[ref_base as usize][base_0 as usize];
                                    }
                                    if add_md_char(s, decode_md, ref_call as c_char, &raw mut md_dist) < 0 {
                                        return -1;
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
                            // 'D'
                            if cig_len != 0 && cig_op != BAM_CDEL {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_DL as c_uint != 0 {
                                if codec!(DS_DL).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_DL, &raw mut i32 as *mut c_char, &raw mut out_sz);
                                if r != 0 {
                                    return r;
                                }
                                if i32 < 0 {
                                    cigar_extends_error = true;
                                    break;
                                }
                                if decode_md != 0 || decode_nm != 0 {
                                    if ref_pos + i32 as int64_t > (*s).ref_end {
                                        cigar_extends_error = true;
                                        break;
                                    }
                                    if md_dist >= 0 && decode_md != 0 && block_append_uint((*s).aux_blk, md_dist as c_uint) < 0 {
                                        return -1;
                                    }
                                    if ref_pos + i32 as int64_t
                                        <= (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                    {
                                        if decode_md != 0 {
                                            if block_append_char((*s).aux_blk, '^' as i32 as c_char) < 0 {
                                                return -1;
                                            }
                                            if block_append(
                                                (*s).aux_blk,
                                                (*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                                    as *const c_void,
                                                i32 as size_t,
                                            ) < 0
                                            {
                                                return -1;
                                            }
                                            md_dist = 0;
                                        }
                                        nm = nm.wrapping_add(i32 as c_uint);
                                    } else {
                                        if (*(*bfd).ref_.offset((*cr).ref_id as isize)).len >= ref_pos {
                                            if decode_md != 0 {
                                                if block_append_char((*s).aux_blk, '^' as i32 as c_char) < 0 {
                                                    return -1;
                                                }
                                                if block_append(
                                                    (*s).aux_blk,
                                                    (*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                                        as *const c_void,
                                                    ((*(*bfd).ref_.offset((*cr).ref_id as isize)).len - ref_pos)
                                                        as size_t,
                                                ) < 0
                                                {
                                                    return -1;
                                                }
                                                if block_append_uint((*s).aux_blk, 0) < 0 {
                                                    return -1;
                                                }
                                            }
                                            let dlen = (i32 as int64_t
                                                - ((*(*bfd).ref_.offset((*cr).ref_id as isize)).len - ref_pos))
                                                as uint32_t;
                                            nm = nm.wrapping_add((i32 as uint32_t).wrapping_sub(dlen));
                                        }
                                        md_dist = -1;
                                    }
                                }
                                cig_op = BAM_CDEL_;
                                cig_len += i32;
                                ref_pos += i32 as int64_t;
                            }
                        }
                        73 => {
                            // 'I'
                            let mut out_sz2_0: int32_t =
                                if (*cr).len != 0 { (*cr).len - (pos - 1) } else { 1 };
                            if cig_len != 0 && cig_op != BAM_CINS {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_IN as c_uint != 0 {
                                if codec!(DS_IN).is_null() {
                                    return -1;
                                }
                                let outp = if (*cr).len != 0 {
                                    seq.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_IN, outp, &raw mut out_sz2_0);
                                if r != 0 {
                                    return r;
                                }
                                cig_op = BAM_CINS_;
                                cig_len += out_sz2_0;
                                seq_pos += out_sz2_0;
                                nm = nm.wrapping_add(out_sz2_0 as c_uint);
                            }
                        }
                        105 => {
                            // 'i'
                            if cig_len != 0 && cig_op != BAM_CINS {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BA as c_uint != 0 {
                                if codec!(DS_BA).is_null() {
                                    return -1;
                                }
                                let outp = if (*cr).len != 0 {
                                    seq.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_BA, outp, &raw mut out_sz);
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
                            // 'b'
                            let mut len: int32_t = if (*cr).len != 0 { (*cr).len - (pos - 1) } else { 1 };
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BB as c_uint != 0 {
                                if codec!(DS_BB).is_null() {
                                    return -1;
                                }
                                let outp = if (*cr).len != 0 {
                                    seq.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_BB, outp, &raw mut len);
                                if r != 0 {
                                    return r;
                                }
                                if decode_md != 0 || decode_nm != 0 {
                                    let mut x = 0;
                                    if md_dist >= 0 && decode_md != 0 && block_append_uint((*s).aux_blk, md_dist as c_uint) < 0 {
                                        return -1;
                                    }
                                    while x < len {
                                        if x != 0 && decode_md != 0 && block_append_uint((*s).aux_blk, 0) < 0 {
                                            return -1;
                                        }
                                        if ref_pos + x as int64_t
                                            >= (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                            || (*s).ref_0.is_null()
                                        {
                                            md_dist = -1;
                                            break;
                                        } else {
                                            if decode_md != 0 {
                                                if ref_pos + x as int64_t >= (*s).ref_end {
                                                    cigar_extends_error = true;
                                                    break;
                                                }
                                                let r_0 = *(*s).ref_0.offset(
                                                    (ref_pos + x as int64_t - (*s).ref_start + 1) as isize,
                                                );
                                                if block_append_char((*s).aux_blk, r_0) < 0 {
                                                    return -1;
                                                }
                                            }
                                            x += 1;
                                        }
                                    }
                                    if cigar_extends_error {
                                        break;
                                    }
                                    nm = nm.wrapping_add(x as c_uint);
                                    md_dist = 0;
                                }
                            }
                            cig_op = BAM_CMATCH_;
                            cig_len += len;
                            seq_pos += len;
                            ref_pos += len as int64_t;
                        }
                        113 => {
                            // 'q'
                            let mut len_0: int32_t = if (*cr).len != 0 { (*cr).len - (pos - 1) } else { 1 };
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_QQ as c_uint != 0 {
                                if codec!(DS_QQ).is_null() {
                                    return -1;
                                }
                                if ds & CRAM_QS as c_uint != 0
                                    && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                    && (*cr).len > 0
                                    && *qual as c_uchar as c_int == 255
                                {
                                    memset(qual as *mut c_void, 30, (*cr).len as size_t);
                                }
                                let outp = if (*cr).len != 0 {
                                    qual.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_QQ, outp, &raw mut len_0);
                                if r != 0 {
                                    return r;
                                }
                            }
                            cig_op = BAM_CMATCH_;
                        }
                        66 => {
                            // 'B'
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BA as c_uint != 0 {
                                if codec!(DS_BA).is_null() {
                                    return -1;
                                }
                                let outp = if (*cr).len != 0 {
                                    seq.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_BA, outp, &raw mut out_sz);
                                if decode_md != 0 || decode_nm != 0 {
                                    if md_dist >= 0 && decode_md != 0 && block_append_uint((*s).aux_blk, md_dist as c_uint) < 0 {
                                        return -1;
                                    }
                                    if ref_pos >= (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                        || (*s).ref_0.is_null()
                                    {
                                        md_dist = -1;
                                    } else {
                                        if decode_md != 0 {
                                            if ref_pos >= (*s).ref_end {
                                                cigar_extends_error = true;
                                                break;
                                            }
                                            if block_append_char(
                                                (*s).aux_blk,
                                                *(*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize),
                                            ) < 0
                                            {
                                                return -1;
                                            }
                                        }
                                        nm = nm.wrapping_add(1);
                                        md_dist = 0;
                                    }
                                }
                            }
                            if ds & CRAM_QS as c_uint != 0 {
                                if codec!(DS_QS).is_null() {
                                    return -1;
                                }
                                if cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                    && (*cr).len > 0
                                    && *qual as c_uchar as c_int == 255
                                {
                                    memset(qual as *mut c_void, 30, (*cr).len as size_t);
                                }
                                let outp = if (*cr).len != 0 {
                                    qual.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_QS, outp, &raw mut out_sz);
                            }
                            cig_op = BAM_CMATCH_;
                            cig_len += 1;
                            seq_pos += 1;
                            ref_pos += 1;
                        }
                        81 => {
                            // 'Q'
                            if ds & CRAM_QS as c_uint != 0 {
                                if codec!(DS_QS).is_null() {
                                    return -1;
                                }
                                if cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                    && (*cr).len > 0
                                    && *qual as c_uchar as c_int == 255
                                {
                                    memset(qual as *mut c_void, 30, (*cr).len as size_t);
                                }
                                let outp = if (*cr).len != 0 {
                                    qual.offset((pos - 1) as isize)
                                } else {
                                    std::ptr::null_mut()
                                };
                                r |= decode!(DS_QS, outp, &raw mut out_sz);
                            }
                        }
                        72 => {
                            // 'H'
                            if cig_len != 0 && cig_op != BAM_CHARD_CLIP {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_HC as c_uint != 0 {
                                if codec!(DS_HC).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_HC, &raw mut i32 as *mut c_char, &raw mut out_sz);
                                if r != 0 {
                                    return r;
                                }
                                if i32 < 0 {
                                    cigar_extends_error = true;
                                    break;
                                }
                                cig_op = BAM_CHARD_CLIP_;
                                cig_len += i32;
                            }
                        }
                        80 => {
                            // 'P'
                            if cig_len != 0 && cig_op != BAM_CPAD {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_PD as c_uint != 0 {
                                if codec!(DS_PD).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_PD, &raw mut i32 as *mut c_char, &raw mut out_sz);
                                if r != 0 {
                                    return r;
                                }
                                if i32 < 0 {
                                    cigar_extends_error = true;
                                    break;
                                }
                                cig_op = BAM_CPAD_;
                                cig_len += i32;
                            }
                        }
                        78 => {
                            // 'N'
                            if cig_len != 0 && cig_op != BAM_CREF_SKIP {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_RS as c_uint != 0 {
                                if codec!(DS_RS).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_RS, &raw mut i32 as *mut c_char, &raw mut out_sz);
                                if r != 0 {
                                    return r;
                                }
                                if i32 < 0 {
                                    cigar_extends_error = true;
                                    break;
                                }
                                cig_op = BAM_CREF_SKIP_;
                                cig_len += i32;
                                ref_pos += i32 as int64_t;
                            }
                        }
                        _ => {
                            hts_log!(
                                HTS_LOG_ERROR,
                                b"cram_decode_seq\0" as *const u8 as *const c_char,
                                b"Unknown feature code\0" as *const u8 as *const c_char
                            );
                            return -1;
                        }
                    }
                }
                f += 1;
            }
            if cigar_extends_error {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_decode_seq\0" as *const u8 as *const c_char,
                    b"CRAM CIGAR extends beyond slice reference extents\0" as *const u8
                        as *const c_char
                );
                return -1;
            }
            // f-loop finished: tail of feature processing (FN != 0 && len >= seq_pos)
            if ds & CRAM_FC as c_uint == 0 {
                break 'feature_done;
            }
            if ds & CRAM_FN as c_uint != 0 && (*cr).len >= seq_pos {
                if !(*s).ref_0.is_null() && (*cr).ref_id >= 0 {
                    if ref_pos + (*cr).len as int64_t - seq_pos as int64_t + 1
                        > (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                    {
                        let rlen_0 = ((*(*bfd).ref_.offset((*cr).ref_id as isize)).len - ref_pos) as c_int;
                        if rlen_0 > 0 {
                            if ref_pos + rlen_0 as int64_t > (*s).ref_end {
                                cigar_extends_error = true;
                            } else {
                                if seq_pos - 1 + rlen_0 < (*cr).len {
                                    memcpy(
                                        seq.offset((seq_pos - 1) as isize) as *mut c_void,
                                        (*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                            as *const c_void,
                                        rlen_0 as size_t,
                                    );
                                }
                                if (*cr).len - seq_pos + 1 - rlen_0 > 0 {
                                    memset(
                                        seq.offset((seq_pos - 1 + rlen_0) as isize) as *mut c_void,
                                        'N' as i32,
                                        ((*cr).len - seq_pos + 1 - rlen_0) as size_t,
                                    );
                                }
                            }
                        } else {
                            if (*cr).len - seq_pos + 1 > 0 {
                                memset(
                                    seq.offset((seq_pos - 1) as isize) as *mut c_void,
                                    'N' as i32,
                                    ((*cr).len - seq_pos + 1) as size_t,
                                );
                            }
                        }
                        if !cigar_extends_error {
                            if md_dist >= 0 {
                                md_dist += (*cr).len - seq_pos + 1;
                            }
                        }
                    } else {
                        if (*cr).len - seq_pos + 1 > 0 {
                            if ref_pos + (*cr).len as int64_t - seq_pos as int64_t + 1 > (*s).ref_end {
                                cigar_extends_error = true;
                            } else {
                                let remainder = (*cr).len - (seq_pos - 1);
                                let j = (ref_pos - (*s).ref_start + 1) as c_int;
                                if decode_md != 0 || decode_nm != 0 {
                                    let n_0 = memchr(
                                        (*s).ref_0.offset(j as isize) as *const c_void,
                                        'N' as i32,
                                        remainder as size_t,
                                    ) as *mut c_char;
                                    if n_0.is_null() {
                                        md_dist += (*cr).len - (seq_pos - 1);
                                    } else {
                                        let refp_0 = (*s).ref_0.offset((j - (seq_pos - 1)) as isize);
                                        md_dist = (md_dist as int64_t
                                            + n_0.offset_from((*s).ref_0.offset(j as isize)) as int64_t)
                                            as int32_t;
                                        let i_start = ((seq_pos - 1) as int64_t
                                            + n_0.offset_from((*s).ref_0.offset(j as isize)) as int64_t)
                                            as c_int;
                                        let mut i_0 = i_start;
                                        while i_0 < (*cr).len {
                                            let base_1 = *refp_0.offset(i_0 as isize);
                                            if base_1 as c_int == 'N' as i32 {
                                                if add_md_char(s, decode_md, 'N' as i32 as c_char, &raw mut md_dist) < 0 {
                                                    return -1;
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
                                    seq.offset((seq_pos - 1) as isize) as *mut c_void,
                                    (*s).ref_0.offset(j as isize) as *const c_void,
                                    remainder as size_t,
                                );
                            }
                        }
                        if !cigar_extends_error {
                            ref_pos += ((*cr).len - seq_pos + 1) as int64_t;
                        }
                    }
                } else {
                    if (*cr).ref_id >= 0 {
                        ref_pos += ((*cr).len - seq_pos + 1) as int64_t;
                    }
                }
                if cigar_extends_error {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_seq\0" as *const u8 as *const c_char,
                        b"CRAM CIGAR extends beyond slice reference extents\0" as *const u8
                            as *const c_char
                    );
                    return -1;
                }
                if ncigar.wrapping_add(1) >= cigar_alloc {
                    cigar_alloc = if cigar_alloc != 0 { cigar_alloc.wrapping_mul(2) } else { 1024 };
                    cigar = realloc(
                        (*s).cigar as *mut c_void,
                        cigar_alloc as u64 * std::mem::size_of::<uint32_t>() as u64,
                    )
                    .cast::<uint32_t>();
                    if cigar.is_null() {
                        return -1;
                    }
                    (*s).cigar = cigar;
                }
                if cig_len != 0 && cig_op != BAM_CMATCH {
                    let t = ncigar;
                    ncigar = ncigar.wrapping_add(1);
                    *cigar.offset(t as isize) = ((cig_len << 4) as c_uint).wrapping_add(cig_op);
                    cig_len = 0;
                }
                cig_op = BAM_CMATCH_;
                cig_len += (*cr).len - seq_pos + 1;
            }
        } // 'feature_done
        // -- after feature processing --
        if ds & CRAM_FN as c_uint != 0 && decode_md != 0 && md_dist >= 0 {
            if block_append_uint((*s).aux_blk, md_dist as c_uint) < 0 {
                return -1;
            }
        }
        if cig_len != 0 {
            if ncigar >= cigar_alloc {
                cigar_alloc = if cigar_alloc != 0 { cigar_alloc.wrapping_mul(2) } else { 1024 };
                cigar = realloc(
                    (*s).cigar as *mut c_void,
                    cigar_alloc as u64 * std::mem::size_of::<uint32_t>() as u64,
                )
                .cast::<uint32_t>();
                if cigar.is_null() {
                    return -1;
                }
                (*s).cigar = cigar;
            }
            let t = ncigar;
            ncigar = ncigar.wrapping_add(1);
            *cigar.offset(t as isize) = ((cig_len << 4) as c_uint).wrapping_add(cig_op);
        }
        (*cr).ncigar = ncigar.wrapping_sub((*cr).cigar) as int32_t;
        (*cr).aend = if ref_pos > (*cr).apos { ref_pos } else { (*cr).apos };
        if ds & CRAM_MQ as c_uint != 0 {
            if codec!(DS_MQ).is_null() {
                return -1;
            }
            r |= decode!(DS_MQ, &raw mut (*cr).mqual as *mut c_char, &raw mut out_sz);
        } else {
            (*cr).mqual = 40;
        }
        if ds & CRAM_QS as c_uint != 0 && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES != 0 {
            let mut out_sz2_1: int32_t = (*cr).len;
            if codec!(DS_QS).is_null() {
                return -1;
            }
            r |= decode!(DS_QS, qual, &raw mut out_sz2_1);
        }
        (*s).cigar = cigar;
        (*s).cigar_alloc = cigar_alloc;
        (*s).ncigar = ncigar;
        if (*cr).cram_flags & CRAM_FLAG_NO_SEQ != 0 {
            (*cr).len = 0;
        }
        if decode_md != 0 {
            if block_append_char((*s).aux_blk, 0) < 0 {
                return -1;
            }
            let sz: size_t = (*(*s).aux_blk).byte.wrapping_sub(orig_aux as size_t);
            if has_MD < 0 {
                let mut tmp_MD_: [c_char; 1024] = [0; 1024];
                let mut tmp_MD: *mut c_char = &raw mut tmp_MD_ as *mut c_char;
                let orig_aux_p: *mut c_uchar = (*(*s).aux_blk).data.offset(orig_aux as isize);
                if sz > 1024 {
                    tmp_MD = malloc(sz as u64).cast::<c_char>();
                    if tmp_MD.is_null() {
                        return -1;
                    }
                }
                memcpy(tmp_MD as *mut c_void, orig_aux_p as *const c_void, sz);
                memmove(
                    ((*(*s).aux_blk).data.offset(-has_MD as isize)).offset(sz as isize) as *mut c_void,
                    (*(*s).aux_blk).data.offset(-has_MD as isize) as *const c_void,
                    orig_aux_p.offset_from((*(*s).aux_blk).data.offset(-has_MD as isize)) as size_t,
                );
                memcpy(
                    (*(*s).aux_blk).data.offset(-has_MD as isize) as *mut c_void,
                    tmp_MD as *const c_void,
                    sz,
                );
                if tmp_MD != &raw mut tmp_MD_ as *mut c_char {
                    free(tmp_MD as *mut c_void);
                }
                if -has_NM > -has_MD {
                    has_NM = (has_NM as size_t).wrapping_sub(sz) as c_int;
                }
            }
            (*cr).aux_size = (*cr).aux_size.wrapping_add(sz as uint32_t);
        }
        if decode_nm != 0 {
            if has_NM == 0 {
                let mut buf: [c_char; 7] = [0; 7];
                let buf_size: size_t;
                buf[0] = 'N' as i32 as c_char;
                buf[1] = 'M' as i32 as c_char;
                if nm <= UINT8_MAX as uint32_t {
                    buf_size = 4;
                    buf[2] = 'C' as i32 as c_char;
                    buf[3] = (nm & 0xff) as c_char;
                } else if nm <= UINT16_MAX as uint32_t {
                    buf_size = 5;
                    buf[2] = 'S' as i32 as c_char;
                    buf[3] = (nm & 0xff) as c_char;
                    buf[4] = (nm >> 8 & 0xff) as c_char;
                } else {
                    buf_size = 7;
                    buf[2] = 'I' as i32 as c_char;
                    buf[3] = (nm & 0xff) as c_char;
                    buf[4] = (nm >> 8 & 0xff) as c_char;
                    buf[5] = (nm >> 16 & 0xff) as c_char;
                    buf[6] = (nm >> 24 & 0xff) as c_char;
                }
                if block_append((*s).aux_blk, &raw mut buf as *mut c_char as *const c_void, buf_size) < 0 {
                    return -1;
                }
                (*cr).aux_size = (*cr).aux_size.wrapping_add(buf_size as uint32_t);
            } else {
                let buf_0: *mut c_uchar = (*(*s).aux_blk).data.offset(-has_NM as isize);
                *buf_0.offset(0) = (nm & 0xff) as c_uchar;
                *buf_0.offset(1) = (nm >> 8 & 0xff) as c_uchar;
                *buf_0.offset(2) = (nm >> 16 & 0xff) as c_uchar;
                *buf_0.offset(3) = (nm >> 24 & 0xff) as c_uchar;
            }
        }
        r
    }

    // original: cram_decode_slice (htslib/cram/cram_decode.c:2346)
    pub unsafe fn cram_decode_slice(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        sh: *mut sam_hdr_t,
    ) -> c_int {
        let mut last_ref_id: c_int;
        let blk: *mut cram_block = *(*s).block.offset(0);
        let mut bf: int32_t = 0;
        let ref_id: int32_t;
        let mut cf: c_uchar = 0;
        let mut out_sz: c_int;
        let mut r: c_int = 0;
        let mut rec: c_int;
        let mut seq: *mut c_char = std::ptr::null_mut();
        let mut qual: *mut c_char = std::ptr::null_mut();
        let mut unknown_rg: c_int = -1;
        let mut embed_ref: c_int;
        let mut refs: *mut *mut c_char = std::ptr::null_mut();
        let mut ds: uint32_t;
        let bfd: *mut sam_hrecs_t = (*sh).hrecs;
        let comp = (*c).comp_hdr;
        macro_rules! codec {
            ($id:expr) => {
                (*comp).codecs[$id as usize]
            };
        }
        macro_rules! decode {
            ($id:expr, $out:expr, $sz:expr) => {{
                (*codec!($id)).decode.expect("decode")(s, codec!($id), blk, $out, $sz)
            }};
        }

        if cram_dependent_data_series(fd, comp, s) != 0 {
            return -1;
        }
        ds = (*s).data_series;
        (*blk).bit = 7;
        let mut qsize: c_int = 0;
        let mut nsize: c_int = 0;
        let mut q_id: c_int = 0;
        cram_decode_estimate_sizes(comp, s, &raw mut qsize, &raw mut nsize, &raw mut q_id);
        if qsize != 0 && ds & CRAM_RL as c_uint != 0 && block_resize_exact((*s).seqs_blk, (qsize + 1) as size_t) < 0 {
            return -1;
        }
        if qsize != 0 && ds & CRAM_RL as c_uint != 0 && block_resize_exact((*s).qual_blk, (qsize + 1) as size_t) < 0 {
            return -1;
        }
        if nsize != 0 && ds & CRAM_NS as c_uint != 0 && block_resize_exact((*s).name_blk, (nsize + 1) as size_t) < 0 {
            return -1;
        }
        if (*bfd).nrg > 0
            && !(*(*bfd).rg.cast::<sam_hrec_rg_t>().offset(((*bfd).nrg - 1) as isize)).name.is_null()
            && strcmp(
                (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset(((*bfd).nrg - 1) as isize)).name,
                b"UNKNOWN\0" as *const u8 as *const c_char,
            ) == 0
        {
            unknown_rg = (*bfd).nrg - 1;
        }
        if (*blk).content_type != CORE {
            return -1;
        }
        if !(*s).crecs.is_null() {
            free((*s).crecs.cast());
        }
        (*s).crecs =
            malloc((*(*s).hdr).num_records as u64 * std::mem::size_of::<cram_record>() as u64)
                .cast::<cram_record>();
        if (*s).crecs.is_null() {
            return -1;
        }
        ref_id = (*(*s).hdr).ref_seq_id;
        embed_ref = if (*fd).version >> 8 < 4 {
            ((*(*s).hdr).ref_base_id >= 0) as c_int
        } else {
            ((*(*s).hdr).ref_base_id > 0) as c_int
        };
        if ref_id >= 0 {
            if embed_ref != 0 {
                if (*(*s).hdr).ref_base_id < 0 {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"No reference specified and no embedded reference is available\0"
                            as *const u8 as *const c_char
                    );
                    return -1;
                }
                let b = cram_get_block_by_id(s, (*(*s).hdr).ref_base_id);
                if b.is_null() {
                    return -1;
                }
                if cram_uncompress_block(b) != 0 {
                    return -1;
                }
                (*s).ref_0 = (*b).data as *mut c_char;
                (*s).ref_start = (*(*s).hdr).ref_seq_start;
                (*s).ref_end = (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span - 1;
                if (*(*s).hdr).ref_seq_span > (*b).uncomp_size as int64_t {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"Embedded reference is too small\0" as *const u8 as *const c_char
                    );
                    return -1;
                }
            } else if (*comp).no_ref == 0 {
                if (*fd).required_fields & SAM_SEQ as c_uint != 0 {
                    (*s).ref_0 = cram_get_ref(
                        fd,
                        (*(*s).hdr).ref_seq_id,
                        (*(*s).hdr).ref_seq_start,
                        (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span - 1,
                    );
                }
                (*s).ref_start = (*(*s).hdr).ref_seq_start;
                (*s).ref_end = (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span - 1;
                if (*s).ref_start < 0 {
                    hts_log!(
                        HTS_LOG_WARNING,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"Slice starts before base 1\0" as *const u8 as *const c_char
                    );
                    (*s).ref_start = 0;
                }
                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
                if (*fd).required_fields & SAM_SEQ as c_uint != 0
                    && ref_id < (*(*fd).refs).nref
                    && !(*(*fd).refs).ref_id.is_null()
                    && (*s).ref_end > (**(*(*fd).refs).ref_id.offset(ref_id as isize)).length
                {
                    (*s).ref_end = (**(*(*fd).refs).ref_id.offset(ref_id as isize)).length;
                }
                pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            }
        }
        if (*fd).required_fields & SAM_SEQ as c_uint != 0
            && (*s).ref_0.is_null()
            && (*(*s).hdr).ref_seq_id >= 0
            && (*comp).no_ref == 0
        {
            hts_log!(
                HTS_LOG_ERROR,
                b"cram_decode_slice\0" as *const u8 as *const c_char,
                b"Unable to fetch reference\0" as *const u8 as *const c_char
            );
            return -1;
        }
        if (*fd).version >> 8 != 1
            && (*fd).required_fields & SAM_SEQ as c_uint != 0
            && (*(*s).hdr).ref_seq_id >= 0
            && (*fd).ignore_md5 == 0
            && memcmp(
                &raw mut (*(*s).hdr).md5 as *mut c_uchar as *const c_void,
                b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0" as *const u8 as *const c_char as *const c_void,
                16,
            ) != 0
        {
            let mut digest: [c_uchar; 16] = [0; 16];
            let mut md5: *mut hts_md5_context = std::ptr::null_mut();
            if !(*s).ref_0.is_null() && (*(*s).hdr).ref_seq_id >= 0 {
                let mut start: c_int;
                let mut len: c_int;
                if (*(*s).hdr).ref_seq_start >= (*s).ref_start {
                    start = ((*(*s).hdr).ref_seq_start - (*s).ref_start) as c_int;
                } else {
                    hts_log!(
                        HTS_LOG_WARNING,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"Slice starts before base 1\0" as *const u8 as *const c_char
                    );
                    start = 0;
                }
                if (*(*s).hdr).ref_seq_span <= (*s).ref_end - (*s).ref_start + 1 {
                    len = (*(*s).hdr).ref_seq_span as c_int;
                } else {
                    hts_log!(
                        HTS_LOG_WARNING,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"Slice ends beyond reference end\0" as *const u8 as *const c_char
                    );
                    len = ((*s).ref_end - (*s).ref_start + 1) as c_int;
                }
                md5 = hts_md5_init();
                if md5.is_null() {
                    return -1;
                }
                if (start + len) as hts_pos_t > (*s).ref_end - (*s).ref_start + 1 {
                    len = ((*s).ref_end - (*s).ref_start + 1 - start as hts_pos_t) as c_int;
                }
                if len >= 0 {
                    hts_md5_update(md5, (*s).ref_0.offset(start as isize) as *const c_void, len as u64);
                }
                hts_md5_final(&raw mut digest as *mut c_uchar, md5);
                hts_md5_destroy(md5);
            } else if (*s).ref_0.is_null() && (*(*s).hdr).ref_base_id >= 0 {
                let b_0 = cram_get_block_by_id(s, (*(*s).hdr).ref_base_id);
                if !b_0.is_null() {
                    md5 = hts_md5_init();
                    if md5.is_null() {
                        return -1;
                    }
                    hts_md5_update(md5, (*b_0).data as *const c_void, (*b_0).uncomp_size as u64);
                    hts_md5_final(&raw mut digest as *mut c_uchar, md5);
                    hts_md5_destroy(md5);
                }
            }
            if (*comp).no_ref == 0
                && ((*s).ref_0.is_null() && (*(*s).hdr).ref_base_id < 0
                    || memcmp(
                        &raw mut digest as *mut c_uchar as *const c_void,
                        &raw mut (*(*s).hdr).md5 as *mut c_uchar as *const c_void,
                        16,
                    ) != 0)
            {
                let mut m_arr: [c_char; 33] = [0; 33];
                let mut rname = sam_hdr_tid2name(sh, ref_id);
                if rname.is_null() {
                    rname = b"?\0" as *const u8 as *const c_char;
                }
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_decode_slice\0" as *const u8 as *const c_char,
                    b"MD5 checksum reference mismatch\0" as *const u8 as *const c_char
                );
                let _ = md5_print(&raw mut (*(*s).hdr).md5 as *mut c_uchar, &raw mut m_arr as *mut c_char);
                let _ = md5_print(&raw mut digest as *mut c_uchar, &raw mut m_arr as *mut c_char);
                let mut ks: kstring_t = kstring_t { l: 0, m: 0, s: std::ptr::null_mut() };
                let _ = sam_hdr_find_tag_id(
                    sh,
                    b"SQ\0" as *const u8 as *const c_char,
                    b"SN\0" as *const u8 as *const c_char,
                    rname,
                    b"M5\0" as *const u8 as *const c_char,
                    &raw mut ks,
                );
                ks_free(&raw mut ks);
                return -1;
            }
        }
        if ref_id == -2 {
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
            refs = calloc((*(*fd).refs).nref as u64, std::mem::size_of::<*mut c_char>() as u64)
                .cast::<*mut c_char>();
            pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            if refs.is_null() {
                return -1;
            }
        }
        last_ref_id = -9;
        rec = 0;
        let mut decode_error = false;
        'rec_loop: while rec < (*(*s).hdr).num_records {
            let cr: *mut cram_record = (*s).crecs.offset(rec as isize);
            let mut has_MD: c_int;
            let mut has_NM: c_int;
            (*cr).s = s;
            out_sz = 1;
            if ds & CRAM_BF as c_uint != 0 {
                if codec!(DS_BF).is_null() {
                    decode_error = true;
                    break;
                }
                r |= decode!(DS_BF, &raw mut bf as *mut c_char, &raw mut out_sz);
                if r != 0 || bf < 0 || bf as usize >= 0x1000 {
                    decode_error = true;
                    break;
                }
                bf = (*fd).bam_flag_swap[bf as usize] as int32_t;
                (*cr).flags = bf;
            } else {
                bf = 0x4;
                (*cr).flags = bf;
            }
            if ds & CRAM_CF as c_uint != 0 {
                if (*fd).version >> 8 == 1 {
                    if codec!(DS_CF).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_CF, &raw mut cf as *mut c_char, &raw mut out_sz);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    (*cr).cram_flags = cf as int32_t;
                } else {
                    if codec!(DS_CF).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_CF, &raw mut (*cr).cram_flags as *mut c_char, &raw mut out_sz);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    cf = (*cr).cram_flags as c_uchar;
                }
            } else {
                (*cr).cram_flags = 0;
                cf = 0;
            }
            if (*fd).version >> 8 != 1 && ref_id == -2 {
                if ds & CRAM_RI as c_uint != 0 {
                    if codec!(DS_RI).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_RI, &raw mut (*cr).ref_id as *mut c_char, &raw mut out_sz);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    if (*cr).ref_id < -1 || (*cr).ref_id >= (*bfd).nref {
                        hts_log!(
                            HTS_LOG_ERROR,
                            b"cram_decode_slice\0" as *const u8 as *const c_char,
                            b"Requested unknown reference ID\0" as *const u8 as *const c_char
                        );
                        decode_error = true;
                        break;
                    } else if (*fd).required_fields & (SAM_SEQ | SAM_TLEN) as c_uint != 0
                        && (*cr).ref_id >= 0
                        && (*cr).ref_id != last_ref_id
                    {
                        if (*comp).no_ref == 0 {
                            pthread_mutex_lock(&raw mut (*fd).range_lock);
                            let need_ref = ((*fd).range.refid == -2
                                || (*cr).ref_id == (*fd).range.refid) as c_int;
                            pthread_mutex_unlock(&raw mut (*fd).range_lock);
                            if need_ref != 0 {
                                if (*refs.offset((*cr).ref_id as isize)).is_null() {
                                    *refs.offset((*cr).ref_id as isize) =
                                        cram_get_ref(fd, (*cr).ref_id, 1, 0);
                                }
                                (*s).ref_0 = *refs.offset((*cr).ref_id as isize);
                                if (*s).ref_0.is_null() {
                                    decode_error = true;
                                    break;
                                }
                            } else {
                                (*s).ref_0 = std::ptr::null_mut();
                            }
                            pthread_mutex_lock(&raw mut (*fd).range_lock);
                            let mut discard_last_ref = (last_ref_id >= 0
                                && !(*refs.offset(last_ref_id as isize)).is_null()
                                && ((*fd).range.refid == -2 || last_ref_id == (*fd).range.refid))
                                as c_int;
                            pthread_mutex_unlock(&raw mut (*fd).range_lock);
                            if discard_last_ref != 0 {
                                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                discard_last_ref = ((*fd).unsorted == 0) as c_int;
                                pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                            }
                            if discard_last_ref != 0 {
                                cram_ref_decr((*fd).refs, last_ref_id);
                                *refs.offset(last_ref_id as isize) = std::ptr::null_mut();
                            }
                        }
                        (*s).ref_start = 1;
                        pthread_mutex_lock(&raw mut (*fd).ref_lock);
                        pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
                        (*s).ref_end =
                            (**(*(*fd).refs).ref_id.offset((*cr).ref_id as isize)).length;
                        pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
                        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                        last_ref_id = (*cr).ref_id;
                    }
                } else {
                    (*cr).ref_id = -1;
                }
            } else {
                (*cr).ref_id = ref_id;
            }
            if (*cr).ref_id < -1 || (*cr).ref_id >= (*bfd).nref {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_decode_slice\0" as *const u8 as *const c_char,
                    b"Requested unknown reference ID\0" as *const u8 as *const c_char
                );
                decode_error = true;
                break;
            }
            if ds & CRAM_RL as c_uint != 0 {
                if codec!(DS_RL).is_null() {
                    decode_error = true;
                    break;
                }
                r |= decode!(DS_RL, &raw mut (*cr).len as *mut c_char, &raw mut out_sz);
                if r != 0 {
                    decode_error = true;
                    break;
                }
                if (*cr).len < 0 {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"Read has negative length\0" as *const u8 as *const c_char
                    );
                    decode_error = true;
                    break;
                }
            }
            if ds & CRAM_AP as c_uint != 0 {
                if codec!(DS_AP).is_null() {
                    decode_error = true;
                    break;
                }
                if (*fd).version >> 8 >= 4 {
                    r |= decode!(DS_AP, &raw mut (*cr).apos as *mut c_char, &raw mut out_sz);
                } else {
                    let mut i32: int32_t = 0;
                    r |= decode!(DS_AP, &raw mut i32 as *mut c_char, &raw mut out_sz);
                    (*cr).apos = i32 as int64_t;
                }
                if r != 0 {
                    decode_error = true;
                    break;
                }
                if (*comp).AP_delta != 0 {
                    if (*cr).apos < 0 && (*c).unsorted == 0 {
                        pthread_mutex_lock(&raw mut (*fd).ref_lock);
                        (*fd).unsorted = 1;
                        (*c).unsorted = (*fd).unsorted;
                        pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                    }
                    (*cr).apos += (*s).last_apos;
                }
                (*s).last_apos = (*cr).apos;
                if (*(*s).hdr).ref_seq_id >= 0 && (*cr).apos < (*(*s).hdr).ref_seq_start {
                    decode_error = true;
                    break;
                }
            } else {
                (*cr).apos = (*c).ref_seq_start;
            }
            if ds & CRAM_RG as c_uint != 0 {
                if codec!(DS_RG).is_null() {
                    decode_error = true;
                    break;
                }
                r |= decode!(DS_RG, &raw mut (*cr).rg as *mut c_char, &raw mut out_sz);
                if r != 0 {
                    decode_error = true;
                    break;
                }
                if (*cr).rg == unknown_rg {
                    (*cr).rg = -1;
                }
            } else {
                (*cr).rg = -1;
            }
            (*cr).name_len = 0;
            if (*comp).read_names_included != 0 {
                let mut out_sz2: int32_t = 1;
                (*cr).name = (*(*s).name_blk).byte as int32_t;
                if ds & CRAM_RN as c_uint != 0 {
                    if codec!(DS_RN).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_RN, (*s).name_blk as *mut c_char, &raw mut out_sz2);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    (*cr).name_len = out_sz2;
                }
            }
            (*cr).mate_pos = 0;
            (*cr).mate_line = -1;
            (*cr).mate_ref_id = -1;
            (*cr).explicit_tlen = INT64_MIN;
            if ds & CRAM_CF as c_uint != 0 && cf as c_int & CRAM_FLAG_DETACHED != 0 {
                if ds & CRAM_MF as c_uint != 0 {
                    if (*fd).version >> 8 == 1 {
                        let mut mf: c_uchar = 0;
                        if codec!(DS_MF).is_null() {
                            decode_error = true;
                            break;
                        }
                        r |= decode!(DS_MF, &raw mut mf as *mut c_char, &raw mut out_sz);
                        if r != 0 {
                            decode_error = true;
                            break;
                        }
                        (*cr).mate_flags = mf as int32_t;
                    } else {
                        if codec!(DS_MF).is_null() {
                            decode_error = true;
                            break;
                        }
                        r |= decode!(DS_MF, &raw mut (*cr).mate_flags as *mut c_char, &raw mut out_sz);
                        if r != 0 {
                            decode_error = true;
                            break;
                        }
                    }
                } else {
                    (*cr).mate_flags = 0;
                }
                if (*comp).read_names_included == 0 {
                    let mut out_sz2_0: int32_t = 1;
                    (*cr).name = (*(*s).name_blk).byte as int32_t;
                    if ds & CRAM_RN as c_uint != 0 {
                        if codec!(DS_RN).is_null() {
                            decode_error = true;
                            break;
                        }
                        r |= decode!(DS_RN, (*s).name_blk as *mut c_char, &raw mut out_sz2_0);
                        if r != 0 {
                            decode_error = true;
                            break;
                        }
                        (*cr).name_len = out_sz2_0;
                    }
                }
                if ds & CRAM_NS as c_uint != 0 {
                    if codec!(DS_NS).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_NS, &raw mut (*cr).mate_ref_id as *mut c_char, &raw mut out_sz);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    if (*cr).mate_ref_id < -1 || (*cr).mate_ref_id >= (*bfd).nref {
                        hts_log!(
                            HTS_LOG_ERROR,
                            b"cram_decode_slice\0" as *const u8 as *const c_char,
                            b"Requested unknown mate reference ID\0" as *const u8 as *const c_char
                        );
                        decode_error = true;
                        break;
                    }
                }
                if ds & CRAM_NP as c_uint != 0 {
                    if codec!(DS_NP).is_null() {
                        decode_error = true;
                        break;
                    }
                    if (*fd).version >> 8 < 4 {
                        let mut i32_0: int32_t = 0;
                        r |= decode!(DS_NP, &raw mut i32_0 as *mut c_char, &raw mut out_sz);
                        (*cr).mate_pos = i32_0 as int64_t;
                    } else {
                        r |= decode!(DS_NP, &raw mut (*cr).mate_pos as *mut c_char, &raw mut out_sz);
                    }
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                }
                if ds & CRAM_TS as c_uint != 0 {
                    if codec!(DS_TS).is_null() {
                        decode_error = true;
                        break;
                    }
                    r = cram_decode_tlen(fd, c, s, blk, &raw mut (*cr).tlen);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                } else {
                    (*cr).tlen = INT64_MIN;
                }
            } else if ds & CRAM_CF as c_uint != 0 && cf as c_int & CRAM_FLAG_MATE_DOWNSTREAM != 0 {
                if ds & CRAM_NF as c_uint != 0 {
                    if codec!(DS_NF).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_NF, &raw mut (*cr).mate_line as *mut c_char, &raw mut out_sz);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    (*cr).mate_line += rec + 1;
                    (*cr).mate_ref_id = -1;
                    (*cr).tlen = INT64_MIN;
                    (*cr).mate_pos = 0;
                } else {
                    (*cr).mate_flags = 0;
                    (*cr).tlen = INT64_MIN;
                }
                if ds & CRAM_CF as c_uint != 0 && cf as c_int & CRAM_FLAG_EXPLICIT_TLEN != 0 {
                    if ds & CRAM_TS as c_uint != 0 {
                        r = cram_decode_tlen(fd, c, s, blk, &raw mut (*cr).explicit_tlen);
                        if r != 0 {
                            return r;
                        }
                    } else {
                        (*cr).mate_flags = 0;
                        (*cr).tlen = INT64_MIN;
                    }
                }
            } else if ds & CRAM_CF as c_uint != 0 && cf as c_int & CRAM_FLAG_EXPLICIT_TLEN != 0 {
                if ds & CRAM_TS as c_uint != 0 {
                    r = cram_decode_tlen(fd, c, s, blk, &raw mut (*cr).explicit_tlen);
                    if r != 0 {
                        return r;
                    }
                } else {
                    (*cr).mate_flags = 0;
                    (*cr).tlen = INT64_MIN;
                }
            } else {
                (*cr).mate_flags = 0;
                (*cr).tlen = INT64_MIN;
            }
            has_NM = 0;
            has_MD = 0;
            if (*fd).version >> 8 == 1 {
                r |= cram_decode_aux_1_0(c, s, blk, cr);
            } else {
                r |= cram_decode_aux(fd, c, s, blk, cr, &raw mut has_MD, &raw mut has_NM);
            }
            if r != 0 {
                decode_error = true;
                break;
            }
            if ds & CRAM_RL as c_uint != 0 {
                (*cr).seq = (*(*s).seqs_blk).byte as uint32_t;
                if block_resize((*s).seqs_blk, (*cr).seq.wrapping_add((*cr).len as uint32_t) as size_t) < 0 {
                    decode_error = true;
                    break;
                }
                seq = (*(*s).seqs_blk).data.offset((*(*s).seqs_blk).byte as isize) as *mut c_char;
                (*(*s).seqs_blk).byte = (*(*s).seqs_blk).byte.wrapping_add((*cr).len as size_t);
                if seq.is_null() {
                    decode_error = true;
                    break;
                }
                (*cr).qual = (*(*s).qual_blk).byte as uint32_t;
                if block_resize((*s).qual_blk, (*cr).qual.wrapping_add((*cr).len as uint32_t) as size_t) < 0 {
                    decode_error = true;
                    break;
                }
                qual = (*(*s).qual_blk).data.offset((*(*s).qual_blk).byte as isize) as *mut c_char;
                (*(*s).qual_blk).byte = (*(*s).qual_blk).byte.wrapping_add((*cr).len as size_t);
                if (*s).ref_0.is_null() {
                    memset(seq as *mut c_void, '=' as i32, (*cr).len as size_t);
                }
            }
            if bf & BAM_FUNMAP == 0 {
                if ds & CRAM_AP as c_uint != 0 && (*cr).apos <= 0 {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const c_char,
                        b"Read has alignment position but no unmapped flag\0" as *const u8
                            as *const c_char
                    );
                    decode_error = true;
                    break;
                } else if ds
                    & (CRAM_FN | CRAM_FP | CRAM_FC | CRAM_DL | CRAM_IN | CRAM_SC | CRAM_HC | CRAM_PD
                        | CRAM_RS | CRAM_RL | CRAM_BF | CRAM_BA | CRAM_BS | CRAM_RL | CRAM_AP
                        | CRAM_BB | CRAM_MQ) as c_uint
                    != 0
                {
                    r |= cram_decode_seq(fd, c, s, blk, cr, sh, cf as c_int, seq, qual, has_MD, has_NM);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                } else {
                    (*cr).cigar = 0;
                    (*cr).ncigar = 0;
                    (*cr).aend = (*cr).apos;
                    (*cr).mqual = 0;
                }
            } else {
                let mut out_sz2_1: c_int = (*cr).len;
                (*cr).cigar = 0;
                (*cr).ncigar = 0;
                (*cr).aend = (*cr).apos;
                (*cr).mqual = 0;
                if ds & CRAM_BA as c_uint != 0 && (*cr).len != 0 {
                    if codec!(DS_BA).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_BA, seq, &raw mut out_sz2_1);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                }
                if ds & CRAM_CF as c_uint != 0 && cf as c_int & CRAM_FLAG_PRESERVE_QUAL_SCORES != 0 {
                    out_sz2_1 = (*cr).len;
                    if ds & CRAM_QS as c_uint != 0 && (*cr).len >= 0 {
                        if codec!(DS_QS).is_null() {
                            decode_error = true;
                            break;
                        }
                        r |= decode!(DS_QS, qual, &raw mut out_sz2_1);
                        if r != 0 {
                            decode_error = true;
                            break;
                        }
                    }
                } else if ds & CRAM_RL as c_uint != 0 {
                    memset(qual as *mut c_void, 255, (*cr).len as size_t);
                }
            }
            if (*comp).qs_seq_orient == 0
                && ds & CRAM_QS as c_uint != 0
                && (*cr).flags & BAM_FREVERSE != 0
            {
                let mut i = 0;
                let mut j = (*cr).len - 1;
                while i < j {
                    let c_0 = *qual.offset(i as isize) as c_uchar;
                    *qual.offset(i as isize) = *qual.offset(j as isize);
                    *qual.offset(j as isize) = c_0 as c_char;
                    i += 1;
                    j -= 1;
                }
            }
            rec += 1;
            let _ = &mut decode_error; // suppress unused-mut on the path that never sets it
            continue 'rec_loop;
        }
        if !decode_error {
            // success cleanup path
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            if !refs.is_null() {
                let mut i_0 = 0;
                while i_0 < (*(*fd).refs).nref {
                    if !(*refs.offset(i_0 as isize)).is_null() {
                        cram_ref_decr((*fd).refs, i_0);
                    }
                    i_0 += 1;
                }
                free(refs.cast());
                refs = std::ptr::null_mut();
            } else if ref_id >= 0 && (*s).ref_0 != (*fd).ref_free && embed_ref == 0 {
                cram_ref_decr((*fd).refs, ref_id);
            }
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            r |= cram_decode_slice_xref(s, (*fd).required_fields as c_int);
            let mut i_1 = 0;
            while i_1 < (*(*s).hdr).num_blocks {
                let b_1 = *(*s).block.offset(i_1 as isize);
                cram_free_block(b_1);
                *(*s).block.offset(i_1 as isize) = std::ptr::null_mut();
                i_1 += 1;
            }
            if block_resize_exact((*s).seqs_blk, (*(*s).seqs_blk).byte.wrapping_add(1)) < 0
                || block_resize_exact((*s).qual_blk, (*(*s).qual_blk).byte.wrapping_add(1)) < 0
                || block_resize_exact((*s).name_blk, (*(*s).name_blk).byte.wrapping_add(1)) < 0
                || block_resize_exact((*s).aux_blk, (*(*s).aux_blk).byte.wrapping_add(1)) < 0
            {
                // fall through to error cleanup (returns -1 below)
            } else {
                return r;
            }
        }
        // error cleanup
        if !refs.is_null() {
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            let mut i_2 = 0;
            while i_2 < (*(*fd).refs).nref {
                if !(*refs.offset(i_2 as isize)).is_null() {
                    cram_ref_decr((*fd).refs, i_2);
                }
                i_2 += 1;
            }
            free(refs.cast());
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
        }
        -1
    }

    // original: cram_decode_slice_thread (htslib/cram/cram_decode.c:3036)
    unsafe extern "C" fn cram_decode_slice_thread(arg: *mut c_void) -> *mut c_void {
        let j = arg as *mut cram_decode_job;
        (*j).exit_code = cram_decode_slice((*j).fd, (*j).c, (*j).s, (*j).h);
        j as *mut c_void
    }

    // original: cram_decode_slice_mt (htslib/cram/cram_decode.c:3047)
    unsafe fn cram_decode_slice_mt(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        bfd: *mut sam_hdr_t,
    ) -> c_int {
        if (*fd).pool.is_null() {
            return cram_decode_slice(fd, c, s, bfd);
        }
        let j = malloc(std::mem::size_of::<cram_decode_job>() as u64).cast::<cram_decode_job>();
        if j.is_null() {
            return -1;
        }
        (*j).fd = fd;
        (*j).c = c;
        (*j).s = s;
        (*j).h = bfd;
        let nonblock = if hts_tpool_process_sz((*fd).rqueue.cast()) != 0 { 1 } else { 0 };
        let saved_errno = *__errno_location();
        *__errno_location() = 0;
        if -1
            == hts_tpool_dispatch2(
                (*fd).pool.cast(),
                (*fd).rqueue.cast(),
                Some(cram_decode_slice_thread),
                j as *mut c_void,
                nonblock,
            )
        {
            if *__errno_location() != EAGAIN {
                return -1;
            }
            (*fd).job_pending = j as *mut c_void;
        } else {
            (*fd).job_pending = std::ptr::null_mut();
        }
        *__errno_location() = saved_errno;
        0
    }

    // original: cram_to_bam (htslib/cram/cram_decode.c:3100)
    pub unsafe fn cram_to_bam(
        sh: *mut sam_hdr_t,
        fd: *mut cram_fd,
        s: *mut cram_slice,
        cr: *mut cram_record,
        rec: c_int,
        bam_0: *mut *mut bam_seq_t,
    ) -> c_int {
        let ret: c_int;
        let rg_len: c_int;
        let mut name_a: [c_char; 1024] = [0; 1024];
        let mut name: *mut c_char;
        let mut name_len: c_int = 0;
        let mut aux: *mut c_char;
        let mut seq: *mut c_char;
        let qual: *mut c_char;
        let bfd: *mut sam_hrecs_t = (*sh).hrecs;
        if (*fd).required_fields & SAM_QNAME as c_uint != 0 {
            if (*cr).name_len != 0 {
                name = ((*(*s).name_blk).data as *mut c_char).offset((*cr).name as isize);
                name_len = (*cr).name_len;
            } else {
                name = &raw mut name_a as *mut c_char;
                if (*cr).mate_line >= 0
                    && (*cr).mate_line < (*s).max_rec
                    && (*(*s).crecs.offset((*cr).mate_line as isize)).name_len > 0
                {
                    memcpy(
                        &raw mut name_a as *mut c_char as *mut c_void,
                        (*(*s).name_blk)
                            .data
                            .offset((*(*s).crecs.offset((*cr).mate_line as isize)).name as isize)
                            as *const c_void,
                        (*(*s).crecs.offset((*cr).mate_line as isize)).name_len as size_t,
                    );
                    name = (&raw mut name_a as *mut c_char)
                        .offset((*(*s).crecs.offset((*cr).mate_line as isize)).name_len as isize);
                } else {
                    name_len = strlen((*fd).prefix) as c_int;
                    memcpy(name as *mut c_void, (*fd).prefix as *const c_void, name_len as size_t);
                    name = name.offset(name_len as isize);
                    let t = name;
                    name = name.offset(1);
                    *t = ':' as i32 as c_char;
                    if (*cr).mate_line >= 0 && (*cr).mate_line < rec {
                        name = append_uint64(
                            name as *mut c_uchar,
                            ((*(*s).hdr).record_counter + (*cr).mate_line as int64_t + 1) as uint64_t,
                        ) as *mut c_char;
                    } else {
                        name = append_uint64(
                            name as *mut c_uchar,
                            ((*(*s).hdr).record_counter + rec as int64_t + 1) as uint64_t,
                        ) as *mut c_char;
                    }
                }
                name_len = name.offset_from(&raw mut name_a as *mut c_char) as c_int;
                name = &raw mut name_a as *mut c_char;
            }
        } else {
            name = b"?\0" as *const u8 as *const c_char as *mut c_char;
            name_len = 1;
        }
        if (*cr).rg < -1 || (*cr).rg >= (*bfd).nrg {
            return -1;
        }
        rg_len = if (*cr).rg != -1 {
            (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset((*cr).rg as isize)).name_len + 4
        } else {
            0
        };
        if (*fd).required_fields & (SAM_SEQ | SAM_QUAL) as c_uint != 0 {
            if (*(*s).seqs_blk).data.is_null() {
                return -1;
            }
            seq = ((*(*s).seqs_blk).data as *mut c_char).offset((*cr).seq as isize);
        } else {
            seq = b"*\0" as *const u8 as *const c_char as *mut c_char;
            (*cr).len = 0;
        }
        if (*fd).required_fields & SAM_QUAL as c_uint != 0 {
            if (*(*s).qual_blk).data.is_null() {
                return -1;
            }
            qual = ((*(*s).qual_blk).data as *mut c_char).offset((*cr).qual as isize);
        } else {
            qual = std::ptr::null_mut();
        }
        ret = bam_set1(
            (*bam_0).cast(),
            name_len as size_t,
            name,
            (*cr).flags as u16,
            (*cr).ref_id,
            (*cr).apos as hts_pos_t - 1,
            (*cr).mqual as u8,
            (*cr).ncigar as size_t,
            (*s).cigar.offset((*cr).cigar as isize),
            (*cr).mate_ref_id,
            (*cr).mate_pos as hts_pos_t - 1,
            (*cr).tlen as hts_pos_t,
            (*cr).len as size_t,
            seq,
            qual,
            (*cr).aux_size.wrapping_add(rg_len as uint32_t) as size_t,
        );
        if ret < 0 {
            return ret;
        }
        let b = (*bam_0).cast::<bam1_t>();
        aux = (*b)
            .data
            .offset(((*b).core.n_cigar << 2) as isize)
            .offset((*b).core.l_qname as c_int as isize)
            .offset(((*b).core.l_qseq + 1 >> 1) as isize)
            .offset((*b).core.l_qseq as isize) as *mut c_char;
        if (*cr).aux_size != 0 {
            memcpy(
                aux as *mut c_void,
                (*(*s).aux_blk).data.offset((*cr).aux as isize) as *const c_void,
                (*cr).aux_size as size_t,
            );
            aux = aux.offset((*cr).aux_size as isize);
            (*b).l_data = ((*b).l_data as c_uint).wrapping_add((*cr).aux_size) as c_int;
        }
        if rg_len > 0 {
            let t1 = aux;
            aux = aux.offset(1);
            *t1 = 'R' as i32 as c_char;
            let t2 = aux;
            aux = aux.offset(1);
            *t2 = 'G' as i32 as c_char;
            let t3 = aux;
            aux = aux.offset(1);
            *t3 = 'Z' as i32 as c_char;
            let len = (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset((*cr).rg as isize)).name_len;
            memcpy(
                aux as *mut c_void,
                (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset((*cr).rg as isize)).name as *const c_void,
                len as size_t,
            );
            aux = aux.offset(len as isize);
            let t4 = aux;
            *t4 = 0;
            (*b).l_data += rg_len;
        }
        (*b).l_data
    }

    // original: cram_first_slice (htslib/cram/cram_decode.c:3212)
    unsafe fn cram_first_slice(fd: *mut cram_fd) -> *mut cram_container {
        let mut c: *mut cram_container;
        loop {
            if !(*fd).ctr.is_null() {
                cram_free_container((*fd).ctr);
            }
            (*fd).ctr = cram_read_container(fd);
            c = (*fd).ctr;
            if c.is_null() {
                return std::ptr::null_mut();
            }
            (*c).curr_slice_mt = (*c).curr_slice;
            if (*c).length != 0 {
                break;
            }
        }
        if (*fd).range.refid != -2 {
            while (*c).ref_seq_id != -2
                && ((*c).ref_seq_id < (*fd).range.refid
                    || (*fd).range.refid >= 0
                        && (*c).ref_seq_id == (*fd).range.refid
                        && ((*c).ref_seq_start + (*c).ref_seq_span - 1) < (*fd).range.start)
            {
                if 0 != cram_seek(fd, (*c).length as off_t, SEEK_CUR) {
                    return std::ptr::null_mut();
                }
                cram_free_container((*fd).ctr);
                loop {
                    (*fd).ctr = cram_read_container(fd);
                    c = (*fd).ctr;
                    if c.is_null() {
                        return std::ptr::null_mut();
                    }
                    if (*c).length != 0 {
                        break;
                    }
                }
            }
            if (*c).ref_seq_id != -2 && (*c).ref_seq_id != (*fd).range.refid {
                (*fd).eof = 1;
                return std::ptr::null_mut();
            }
        }
        (*c).comp_hdr_block = cram_read_block(fd);
        if (*c).comp_hdr_block.is_null() {
            return std::ptr::null_mut();
        }
        if (*(*c).comp_hdr_block).content_type != COMPRESSION_HEADER {
            return std::ptr::null_mut();
        }
        (*c).comp_hdr = cram_decode_compression_header(fd, (*c).comp_hdr_block);
        if (*c).comp_hdr.is_null() {
            return std::ptr::null_mut();
        }
        if (*(*c).comp_hdr).AP_delta == 0
            && sam_hrecs_sort_order((*(*fd).header).hrecs) != ORDER_COORD
        {
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            (*fd).unsorted = 1;
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
        }
        c
    }

    // original: cram_next_slice (htslib/cram/cram_decode.c:3268)
    pub(crate) unsafe fn cram_next_slice(fd: *mut cram_fd, cp: *mut *mut cram_container) -> *mut cram_slice {
        let mut c_curr: *mut cram_container = (*fd).ctr;
        let mut s_curr: *mut cram_slice;
        if c_curr.is_null() {
            c_curr = cram_first_slice(fd);
            if c_curr.is_null() {
                return std::ptr::null_mut();
            }
        }
        s_curr = (*c_curr).slice;
        if !s_curr.is_null() {
            (*c_curr).slice = std::ptr::null_mut();
            cram_free_slice(s_curr);
            s_curr = std::ptr::null_mut();
        }
        if (*c_curr).curr_slice == (*c_curr).max_slice {
            if (*fd).ctr == c_curr {
                (*fd).ctr = std::ptr::null_mut();
            }
            if (*fd).ctr_mt == c_curr {
                (*fd).ctr_mt = std::ptr::null_mut();
            }
            cram_free_container(c_curr);
            c_curr = std::ptr::null_mut();
        }
        if (*fd).ctr_mt.is_null() {
            (*fd).ctr_mt = c_curr;
        }
        let mut found = false;
        'outer: loop {
            let mut c_next: *mut cram_container = (*fd).ctr_mt;
            let mut s_next: *mut cram_slice = std::ptr::null_mut();
            if !(*fd).job_pending.is_null() {
                let j = (*fd).job_pending as *mut cram_decode_job;
                c_next = (*j).c;
                s_next = (*j).s;
                free((*fd).job_pending);
                (*fd).job_pending = std::ptr::null_mut();
            } else if (*fd).ooc == 0 {
                let mut got_slice = false;
                loop {
                    if c_next.is_null() || (*c_next).curr_slice_mt == (*c_next).max_slice {
                        loop {
                            c_next = cram_read_container(fd);
                            if c_next.is_null() {
                                if !(*fd).pool.is_null() {
                                    (*fd).ooc = 1;
                                    break;
                                } else {
                                    return std::ptr::null_mut();
                                }
                            } else {
                                (*c_next).curr_slice_mt = (*c_next).curr_slice;
                                if (*c_next).length != 0 {
                                    break;
                                }
                                cram_free_container(c_next);
                            }
                        }
                        if (*fd).ooc != 0 {
                            break 'outer;
                        }
                        if (*fd).range.refid != -2 && (*c_next).ref_seq_id != -2 {
                            if (*c_next).ref_seq_id != (*fd).range.refid {
                                cram_free_container(c_next);
                                (*fd).ctr_mt = std::ptr::null_mut();
                                (*fd).ooc = 1;
                                break 'outer;
                            } else if (*fd).range.refid != -1 && (*c_next).ref_seq_start > (*fd).range.end {
                                cram_free_container(c_next);
                                (*fd).ctr_mt = std::ptr::null_mut();
                                (*fd).ooc = 1;
                                break 'outer;
                            } else if (*fd).range.refid != -1
                                && ((*c_next).ref_seq_start + (*c_next).ref_seq_span - 1) < (*fd).range.start
                            {
                                let skip_length = (*c_next).length as off_t;
                                cram_free_container(c_next);
                                c_next = std::ptr::null_mut();
                                (*fd).ooc = 0;
                                if hseek((*fd).fp, skip_length, SEEK_CUR) < 0 {
                                    return std::ptr::null_mut();
                                }
                                continue 'outer;
                            }
                        }
                        (*fd).ctr_mt = c_next;
                        (*c_next).comp_hdr_block = cram_read_block(fd);
                        if (*c_next).comp_hdr_block.is_null() {
                            return std::ptr::null_mut();
                        }
                        if (*(*c_next).comp_hdr_block).content_type != COMPRESSION_HEADER {
                            return std::ptr::null_mut();
                        }
                        (*c_next).comp_hdr = cram_decode_compression_header(fd, (*c_next).comp_hdr_block);
                        if (*c_next).comp_hdr.is_null() {
                            return std::ptr::null_mut();
                        }
                        if (*(*c_next).comp_hdr).AP_delta == 0
                            && sam_hrecs_sort_order((*(*fd).header).hrecs) != ORDER_COORD
                        {
                            pthread_mutex_lock(&raw mut (*fd).ref_lock);
                            (*fd).unsorted = 1;
                            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
                        }
                    }
                    if (*c_next).num_records == 0 {
                        if (*fd).ctr == c_next {
                            (*fd).ctr = std::ptr::null_mut();
                        }
                        if c_curr == c_next {
                            c_curr = std::ptr::null_mut();
                        }
                        if (*fd).ctr_mt == c_next {
                            (*fd).ctr_mt = std::ptr::null_mut();
                        }
                        cram_free_container(c_next);
                        c_next = std::ptr::null_mut();
                    } else {
                        (*c_next).slice = cram_read_slice(fd);
                        s_next = (*c_next).slice;
                        if s_next.is_null() {
                            return std::ptr::null_mut();
                        }
                        (*c_next).curr_slice_mt += 1;
                        (*s_next).slice_num = (*c_next).curr_slice_mt;
                        (*s_next).curr_rec = 0;
                        (*s_next).max_rec = (*(*s_next).hdr).num_records;
                        (*s_next).last_apos = (*(*s_next).hdr).ref_seq_start;
                        if (*fd).range.refid != -2 && (*(*s_next).hdr).ref_seq_id != -2 {
                            // range filter on the slice
                            if (*(*s_next).hdr).ref_seq_id != (*fd).range.refid {
                                (*fd).ooc = 1;
                                cram_free_slice(s_next);
                                s_next = std::ptr::null_mut();
                                (*c_next).slice = s_next;
                                break;
                            } else if (*fd).range.refid != -1
                                && (*(*s_next).hdr).ref_seq_start > (*fd).range.end
                            {
                                (*fd).ooc = 1;
                                cram_free_slice(s_next);
                                s_next = std::ptr::null_mut();
                                (*c_next).slice = s_next;
                                break;
                            } else if (*fd).range.refid != -1
                                && ((*(*s_next).hdr).ref_seq_start + (*(*s_next).hdr).ref_seq_span - 1)
                                    < (*fd).range.start
                            {
                                cram_free_slice(s_next);
                                s_next = std::ptr::null_mut();
                                (*c_next).slice = s_next;
                                continue;
                            }
                            got_slice = true;
                            break;
                        } else {
                            got_slice = true;
                            break;
                        }
                    }
                }
                let _ = got_slice;
            }
            if c_next.is_null() || s_next.is_null() {
                break;
            }
            if cram_decode_slice_mt(fd, c_next, s_next, (*fd).header) != 0 {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_next_slice\0" as *const u8 as *const c_char,
                    b"Failure to decode slice\0" as *const u8 as *const c_char
                );
                cram_free_slice(s_next);
                (*c_next).slice = std::ptr::null_mut();
                return std::ptr::null_mut();
            }
            if (*fd).pool.is_null() {
                c_curr = c_next;
                s_curr = s_next;
                found = true;
                break;
            } else {
                if !(*fd).job_pending.is_null() {
                    break;
                }
                if hts_tpool_process_len((*fd).rqueue.cast())
                    > hts_tpool_process_qsize((*fd).rqueue.cast())
                {
                    break;
                }
            }
        }
        if !found {
            s_curr = std::ptr::null_mut();
        }
        if !(*fd).pool.is_null() {
            if (*fd).ooc != 0 && hts_tpool_process_empty((*fd).rqueue.cast()) != 0 {
                (*fd).eof = 1;
                return std::ptr::null_mut();
            }
            let res = hts_tpool_next_result_wait((*fd).rqueue.cast());
            if res.is_null() || hts_tpool_result_data(res).is_null() {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_next_slice\0" as *const u8 as *const c_char,
                    b"Call to hts_tpool_next_result failed\0" as *const u8 as *const c_char
                );
                return std::ptr::null_mut();
            }
            let j_0 = hts_tpool_result_data(res) as *mut cram_decode_job;
            c_curr = (*j_0).c;
            s_curr = (*j_0).s;
            if (*j_0).exit_code != 0 {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_next_slice\0" as *const u8 as *const c_char,
                    b"Slice decode failure\0" as *const u8 as *const c_char
                );
                (*fd).eof = 0;
                hts_tpool_delete_result(res, 1);
                return std::ptr::null_mut();
            }
            hts_tpool_delete_result(res, 1);
        }
        *cp = c_curr;
        (*fd).ctr = c_curr;
        if !c_curr.is_null() {
            (*c_curr).slice = s_curr;
            if !s_curr.is_null() {
                (*c_curr).curr_slice = (*s_curr).slice_num;
            }
        }
        if !s_curr.is_null() {
            (*s_curr).curr_rec = 0;
        } else {
            (*fd).eof = 1;
        }
        s_curr
    }

    // original: cram_get_seq (htslib/cram/cram_decode.c:3549)
    pub unsafe fn cram_get_seq(fd: *mut cram_fd) -> *mut cram_record {
        let mut c: *mut cram_container;
        let mut s: *mut cram_slice;
        loop {
            c = (*fd).ctr;
            if !c.is_null()
                && !(*c).slice.is_null()
                && (*(*c).slice).curr_rec < (*(*c).slice).max_rec
            {
                s = (*c).slice;
                if (*fd).range.refid == -2 {
                    break;
                }
                if (*fd).range.refid == -1
                    && (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id != -1
                {
                    (*s).curr_rec += 1;
                } else if (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id < (*fd).range.refid
                    && (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id != -1
                {
                    (*s).curr_rec += 1;
                } else {
                    if (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id != (*fd).range.refid {
                        (*fd).eof = 1;
                        cram_free_slice(s);
                        (*c).slice = std::ptr::null_mut();
                        return std::ptr::null_mut();
                    }
                    if (*fd).range.refid != -1
                        && (*(*s).crecs.offset((*s).curr_rec as isize)).apos > (*fd).range.end
                    {
                        (*fd).eof = 1;
                        cram_free_slice(s);
                        (*c).slice = std::ptr::null_mut();
                        return std::ptr::null_mut();
                    }
                    if !((*fd).range.refid != -1
                        && (*(*s).crecs.offset((*s).curr_rec as isize)).aend < (*fd).range.start)
                    {
                        break;
                    }
                    (*s).curr_rec += 1;
                }
            } else {
                s = cram_next_slice(fd, &raw mut c);
                if s.is_null() {
                    return std::ptr::null_mut();
                }
            }
        }
        (*fd).ctr = c;
        (*c).slice = s;
        let t = (*s).curr_rec;
        (*s).curr_rec += 1;
        (*s).crecs.offset(t as isize)
    }

    // original: cram_get_bam_seq (htslib/cram/cram_decode.c:3615)
    pub unsafe fn cram_get_bam_seq(fd: *mut cram_fd, bam_0: *mut *mut bam_seq_t) -> c_int {
        let cr = cram_get_seq(fd);
        if cr.is_null() {
            return -1;
        }
        let c = (*fd).ctr;
        let s = (*c).slice;
        cram_to_bam((*fd).header, fd, s, cr, (*s).curr_rec - 1, bam_0)
    }
}
