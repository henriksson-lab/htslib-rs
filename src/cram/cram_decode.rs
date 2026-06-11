// Functions translated from htslib/cram/cram_decode.c.
// Extracted from src/cram.rs (cut-over completed 2026-06-01).

use super::*;

/// original: cram_decode_TD (htslib/cram/cram_decode.c:71)
/// Native port. Parses the tag-dictionary (TD) block stored in the preservation
/// map. Allocates a fresh `cram_block` (via the native cram_new_block) holding
/// the TD bytes, builds the `tl` lookup table (one entry per NUL-separated
/// tag-list) and stores both into the compression header. Ownership matches C:
/// `tl` is calloc'd and `td_blk` is the new block; both are freed by
/// `cram_free_compression_header`. Returns bytes consumed, or -1 on error.
pub(crate) unsafe fn cram_cram_decode_c_71_cram_decode_TD(
    fd: *mut cram_fd,
    mut cp: *mut u8,
    endp: *const u8,
    h: *mut cram_block_compression_hdr_layout,
) -> i32 {
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
            b"cram_decode_TD",
            b"More than one TD block found in compression header",
        );
        cram_free_block((*h).td_blk.cast());
        drop(Vec::from_raw_parts((*h).tl, 0, 0));
        (*h).td_blk = std::ptr::null_mut();
        (*h).tl = std::ptr::null_mut();
    }

    let fdl = fd.cast::<cram_fd_layout>();
    let vv = &(*fdl).vv;
    let mut err: i32 = 0;
    let blk_size = (vv.varint_get32.unwrap())(&mut cp, endp, &mut err) as i32;
    if blk_size == 0 {
        (*h).ntl = 0;
        cram_free_block(b);
        return cp.offset_from(op) as i32;
    }
    if err != 0 || blk_size < 0 || (endp.offset_from(cp) as i64) < blk_size as i64 {
        cram_free_block(b);
        return -1;
    }

    if cram_cram_io_h_248_block_append(b, cp.cast(), blk_size as usize) < 0 {
        cram_free_block(b);
        return -1;
    }
    cp = cp.add(blk_size as usize);
    let sz = cp.offset_from(op) as i32;
    // Force NUL termination if missing.
    if *(*bl).data.add((*bl).byte - 1) != 0
        && cram_cram_io_h_261_block_append_char(b, 0) < 0
    {
        cram_free_block(b);
        return -1;
    }

    let dat = (*bl).data;
    // Count tag-lists (NUL-separated strings).
    let mut ntl: i32 = 0;
    let mut i: usize = 0;
    while i < (*bl).byte {
        ntl += 1;
        while *dat.add(i) != 0 {
            i += 1;
        }
        i += 1;
    }

    // tl is an array of `ntl` pointers into the TD block data, owned by the
    // compression header; allocate it as a Vec and hand the raw pointer over.
    let mut tl_vec: Vec<*mut u8> = vec![std::ptr::null_mut(); ntl as usize];
    let tl_ptr = tl_vec.as_mut_ptr();
    std::mem::forget(tl_vec);
    (*h).tl = tl_ptr;
    let mut nidx: i32 = 0;
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
        cram_cram_codecs_c_3968_cram_codec_to_id,
        cram_cram_decode_c_145_cram_decode_compression_header,
        cram_cram_io_c_3213_cram_ref_decr,
        cram_cram_io_c_3409_cram_get_ref, cram_cram_io_c_3705_cram_free_container,
        cram_cram_io_c_3788_cram_read_container, cram_cram_io_c_4421_cram_free_slice,
        cram_cram_io_c_4568_cram_read_slice, cram_cram_io_h_183_cram_get_block_by_id,
        cram_cram_io_h_216_block_resize_exact, cram_cram_io_h_226_block_resize,
        cram_cram_io_h_248_block_append, cram_cram_io_h_261_block_append_char,
        cram_cram_io_h_271_block_append_uint, cram_cram_io_h_340_append_uint64,
    };
    use crate::htslib_rs::hts::{hFILE, kstring_t, BGZF};
    use crate::htslib_rs::sam::{
        bam1_t, bam_set1, sam_hdr_find_tag_id, sam_hdr_line_name, sam_hdr_t, sam_hdr_tid2name,
        sam_hrec_rg_t, sam_hrecs_sort_order, sam_hrecs_t, BAM_FMREVERSE, BAM_FMUNMAP, BAM_FPAIRED,
        BAM_FREAD1, BAM_FREVERSE, BAM_FUNMAP, ORDER_COORD,
    };
    use std::ptr::NonNull;

    // ---- mem/str shims (match the mirror's free-standing libc calls) ----
    unsafe fn memcpy(dst: *mut (), src: *const (), n: usize) -> *mut () {
        std::ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), n);
        dst
    }
    unsafe fn memmove(dst: *mut (), src: *const (), n: usize) -> *mut () {
        std::ptr::copy(src.cast::<u8>(), dst.cast::<u8>(), n);
        dst
    }
    unsafe fn memset(dst: *mut (), v: i32, n: usize) -> *mut () {
        std::ptr::write_bytes(dst.cast::<u8>(), v as u8, n);
        dst
    }
    unsafe fn memcmp(a: *const (), b: *const (), n: usize) -> i32 {
        let sa = std::slice::from_raw_parts(a.cast::<u8>(), n);
        let sb = std::slice::from_raw_parts(b.cast::<u8>(), n);
        match sa.cmp(sb) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
    unsafe fn memchr(s: *const (), c: i32, n: usize) -> *mut () {
        let sl = std::slice::from_raw_parts(s.cast::<u8>(), n);
        match sl.iter().position(|&b| b == c as u8) {
            Some(i) => s.cast::<u8>().add(i) as *mut (),
            None => std::ptr::null_mut(),
        }
    }
    unsafe fn strlen(s: *const u8) -> usize {
        let mut n = 0usize;
        while *s.add(n) != 0 {
            n += 1;
        }
        n
    }
    unsafe fn strcmp(a: *const u8, b: *const u8) -> i32 {
        let mut i = 0usize;
        loop {
            let ca = *a.add(i);
            let cb = *b.add(i);
            if ca != cb {
                return ca as i32 - cb as i32;
            }
            if ca == 0 {
                return 0;
            }
            i += 1;
        }
    }
    unsafe fn pthread_mutex_lock(m: *mut libc::pthread_mutex_t) {
        libc::pthread_mutex_lock(m);
    }
    unsafe fn pthread_mutex_unlock(m: *mut libc::pthread_mutex_t) {
        libc::pthread_mutex_unlock(m);
    }
    unsafe fn hseek(fp: *mut hFILE, off: i64, whence: i32) -> i64 {
        crate::htslib_rs::hfile::hseek(fp, off, whence)
    }
    type off_t = i64;
    const SEEK_CUR: i32 = libc::SEEK_CUR;
    const INT_MAX: i32 = i32::MAX;
    const INT64_MIN: i64 = i64::MIN;
    const UINT8_MAX: i32 = 255;
    const UINT16_MAX: i32 = 65535;

    // ---- hts_log: thin wrapper to the production formatted logger ----
    macro_rules! hts_log {
        ($sev:expr, $func:expr, $fmt:expr $(, $arg:expr)*) => {{
            // Format the message ourselves; production hts_log_cstr takes a
            // pre-formatted C string. Most CRAM hts_log() calls here are error
            // diagnostics on failure paths; we forward a static message.
            let _ = ($func, $fmt $(, $arg)*);
            super::hts_log_cstr(
                $sev,
                std::ffi::CStr::from_ptr(($func) as *const i8).to_bytes(),
                std::ffi::CStr::from_ptr(($fmt) as *const i8).to_bytes(),
            );
        }};
    }
    const HTS_LOG_ERROR: i32 = super::HTS_LOG_ERROR;
    const HTS_LOG_WARNING: i32 = super::HTS_LOG_WARNING;

    // ---- numeric type aliases used by the transpiled bodies ----
    type int32_t = i32;
    type int64_t = i64;
    type uint8_t = u8;
    type uint16_t = u16;
    type uint32_t = u32;
    type uint64_t = u64;
    type size_t = usize;
    type hts_pos_t = i64;
    type uc = u8;
    type bam_seq_t = bam1_t;

    // ---- CRAM data-series IDs (cram_DS_ID, cram_structs.h:143) ----
    pub const DS_CORE: i32 = 0;
    pub const DS_aux: i32 = 1;
    pub const DS_RN: i32 = 11;
    pub const DS_QS: i32 = 12;
    pub const DS_IN: i32 = 13;
    pub const DS_SC: i32 = 14;
    pub const DS_BF: i32 = 15;
    pub const DS_CF: i32 = 16;
    pub const DS_AP: i32 = 17;
    pub const DS_RG: i32 = 18;
    pub const DS_MQ: i32 = 19;
    pub const DS_NS: i32 = 20;
    pub const DS_MF: i32 = 21;
    pub const DS_TS: i32 = 22;
    pub const DS_NP: i32 = 23;
    pub const DS_NF: i32 = 24;
    pub const DS_RL: i32 = 25;
    pub const DS_FN: i32 = 26;
    pub const DS_FC: i32 = 27;
    pub const DS_FP: i32 = 28;
    pub const DS_DL: i32 = 29;
    pub const DS_BA: i32 = 30;
    pub const DS_BS: i32 = 31;
    pub const DS_TL: i32 = 32;
    pub const DS_RI: i32 = 33;
    pub const DS_RS: i32 = 34;
    pub const DS_PD: i32 = 35;
    pub const DS_HC: i32 = 36;
    pub const DS_BB: i32 = 37;
    pub const DS_QQ: i32 = 38;
    pub const DS_TN: i32 = 39;
    pub const DS_TC: i32 = 44;
    pub const DS_END: i32 = 47;

    // ---- CRAM field bitflags (cram_fields, cram_structs.h) ----
    pub const CRAM_BF: i32 = 0x0000_0001;
    pub const CRAM_AP: i32 = 0x0000_0002;
    pub const CRAM_FP: i32 = 0x0000_0004;
    pub const CRAM_RL: i32 = 0x0000_0008;
    pub const CRAM_DL: i32 = 0x0000_0010;
    pub const CRAM_NF: i32 = 0x0000_0020;
    pub const CRAM_BA: i32 = 0x0000_0040;
    pub const CRAM_QS: i32 = 0x0000_0080;
    pub const CRAM_FC: i32 = 0x0000_0100;
    pub const CRAM_FN: i32 = 0x0000_0200;
    pub const CRAM_BS: i32 = 0x0000_0400;
    pub const CRAM_IN: i32 = 0x0000_0800;
    pub const CRAM_RG: i32 = 0x0000_1000;
    pub const CRAM_MQ: i32 = 0x0000_2000;
    pub const CRAM_TL: i32 = 0x0000_4000;
    pub const CRAM_RN: i32 = 0x0000_8000;
    pub const CRAM_NS: i32 = 0x0001_0000;
    pub const CRAM_NP: i32 = 0x0002_0000;
    pub const CRAM_TS: i32 = 0x0004_0000;
    pub const CRAM_MF: i32 = 0x0008_0000;
    pub const CRAM_CF: i32 = 0x0010_0000;
    pub const CRAM_RI: i32 = 0x0020_0000;
    pub const CRAM_RS: i32 = 0x0040_0000;
    pub const CRAM_PD: i32 = 0x0080_0000;
    pub const CRAM_HC: i32 = 0x0100_0000;
    pub const CRAM_SC: i32 = 0x0200_0000;
    pub const CRAM_BB: i32 = 0x0400_0000;
    pub const CRAM_BB_len: i32 = 0x0800_0000;
    pub const CRAM_QQ: i32 = 0x1000_0000;
    pub const CRAM_QQ_len: i32 = 0x2000_0000;
    pub const CRAM_aux: i32 = 0x4000_0000;
    pub const CRAM_ALL: i32 = 0x7fff_ffff;

    // ---- SAM fields (sam_fields, sam.h) ----
    pub const SAM_QNAME: i32 = 0x0001;
    pub const SAM_FLAG: i32 = 0x0002;
    pub const SAM_RNAME: i32 = 0x0004;
    pub const SAM_POS: i32 = 0x0008;
    pub const SAM_MAPQ: i32 = 0x0010;
    pub const SAM_CIGAR: i32 = 0x0020;
    pub const SAM_RNEXT: i32 = 0x0040;
    pub const SAM_PNEXT: i32 = 0x0080;
    pub const SAM_TLEN: i32 = 0x0100;
    pub const SAM_SEQ: i32 = 0x0200;
    pub const SAM_QUAL: i32 = 0x0400;
    pub const SAM_AUX: i32 = 0x0800;
    pub const SAM_RGAUX: i32 = 0x1000;

    // ---- CRAM cram_flags / mate flags ----
    pub const CRAM_FLAG_PRESERVE_QUAL_SCORES: i32 = 1 << 0;
    pub const CRAM_FLAG_DETACHED: i32 = 1 << 1;
    pub const CRAM_FLAG_MATE_DOWNSTREAM: i32 = 1 << 2;
    pub const CRAM_FLAG_NO_SEQ: i32 = 1 << 3;
    pub const CRAM_FLAG_EXPLICIT_TLEN: i32 = 1 << 4;
    pub const CRAM_M_REVERSE: i32 = 1;
    pub const CRAM_M_UNMAP: i32 = 2;

    // ---- cram_content_type / cram_block_method ----
    pub const FILE_HEADER: i32 = 0;
    pub const COMPRESSION_HEADER: i32 = 1;
    pub const MAPPED_SLICE: i32 = 2;
    pub const UNMAPPED_SLICE: i32 = 3;
    pub const EXTERNAL: i32 = 4;
    pub const CORE: i32 = 5;
    pub const RAW: i32 = 0;

    // ---- cram_encoding ----
    pub const E_NULL: u32 = 0;
    pub const E_EXTERNAL: u32 = 1;
    pub const E_BYTE_ARRAY_LEN: u32 = 4;
    pub const E_BYTE_ARRAY_STOP: u32 = 5;

    // ---- BAM cigar ops ----
    pub const BAM_CMATCH: u32 = 0;
    pub const BAM_CINS: u32 = 1;
    pub const BAM_CDEL: u32 = 2;
    pub const BAM_CREF_SKIP: u32 = 3;
    pub const BAM_CSOFT_CLIP: u32 = 4;
    pub const BAM_CHARD_CLIP: u32 = 5;
    pub const BAM_CPAD: u32 = 6;
    type cigar_op = u32;
    pub const BAM_CMATCH_: cigar_op = 0;
    pub const BAM_CINS_: cigar_op = 1;
    pub const BAM_CDEL_: cigar_op = 2;
    pub const BAM_CREF_SKIP_: cigar_op = 3;
    pub const BAM_CSOFT_CLIP_: cigar_op = 4;
    pub const BAM_CHARD_CLIP_: cigar_op = 5;
    pub const BAM_CPAD_: cigar_op = 6;

    pub const CRAM_MAP_HASH: i32 = 32;

    // ---- MD5 (delegating to the native crate::htslib_rs::md5 module) ----
    pub use crate::htslib_rs::md5::hts_md5_context;
    unsafe fn hts_md5_init() -> Box<hts_md5_context> {
        crate::htslib_rs::md5::hts_md5_init()
    }
    unsafe fn hts_md5_update(ctx: &mut hts_md5_context, data: &[u8]) {
        crate::htslib_rs::md5::hts_md5_update(ctx, data, data.len());
    }
    unsafe fn hts_md5_final(digest: &mut [u8], ctx: &mut hts_md5_context) {
        crate::htslib_rs::md5::hts_md5_final(digest, ctx);
    }
    unsafe fn hts_md5_destroy(ctx: Option<Box<hts_md5_context>>) {
        crate::htslib_rs::md5::hts_md5_destroy(ctx);
    }

    // ---- ks_free shim ----
    unsafe fn ks_free(s: &mut kstring_t) {
        s.data = Vec::new();
    }

    // =======================================================================
    // Byte-identical struct re-declarations (mirror field names).
    // =======================================================================
    type cram_block_method_int = i32;
    type cram_content_type = i32;
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
        pub data: *mut u8,
        pub alloc: size_t,
        pub byte: size_t,
        pub bit: i32,
        pub m: *mut cram_metrics,
        pub crc32_checked: i32,
        pub crc_part: uint32_t,
    }

    type cram_encoding = u32;
    #[repr(C)]
    pub struct cram_codec {
        pub codec: cram_encoding,
        pub out: *mut cram_block,
        pub vv: *mut (),
        pub codec_id: i32,
        pub free: Option<unsafe extern "C" fn(*mut cram_codec)>,
        pub decode: Option<
            unsafe extern "C" fn(
                *mut cram_slice,
                *mut cram_codec,
                *mut cram_block,
                *mut u8,
                *mut i32,
            ) -> i32,
        >,
    }

    #[repr(C)]
    pub struct cram_map {
        pub key: i32,
        pub encoding: cram_encoding,
        pub offset: i32,
        pub size: i32,
        pub codec: *mut cram_codec,
        pub next: *mut cram_map,
    }

    // The production `cram_block_compression_hdr_layout` is an OWNED, non-repr(C)
    // struct (its `td_keys` field is `Option<Box<StringPool>>`). It is therefore
    // NOT byte-identical to a C-ABI mirror, so the pipeline uses the owned type
    // directly rather than casting a repr(C) mirror pointer to/from it. Field
    // access below goes to the real owned fields (note the renames: AP_delta ->
    // ap_delta, nTL -> ntl, TL -> tl); the `codecs`/`rec_encoding_map`/
    // `tag_encoding_map` arrays are `*mut ()` in the owned layout and are
    // `.cast()` to the local `cram_codec`/`cram_map` mirrors at use sites (those
    // mirrors remain byte-identical to the pointed-to structs).
    use super::cram_block_compression_hdr_layout as cram_block_compression_hdr;

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
        fields: [i32; 4],
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
        pub TL: i32,
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
        pub nTN: i32,
        pub aTN: i32,
        pub name_blk: *mut cram_block,
        pub seqs_blk: *mut cram_block,
        pub qual_blk: *mut cram_block,
        pub base_blk: *mut cram_block,
        pub soft_blk: *mut cram_block,
        pub aux_blk: *mut cram_block,
        pub pair_keys: *mut cram_string_alloc_t,
        pub pair: [*mut (); 2],
        pub ref_0: *mut u8,
        pub ref_start: i64,
        pub ref_end: i64,
        pub ref_id: i32,
        pub naux_block: i32,
        pub aux_block: *mut *mut cram_block,
        pub data_series: u32,
        pub decode_md: i32,
        pub max_rec: i32,
        pub curr_rec: i32,
        pub slice_num: i32,
    }

    // The production `cram_container_layout` is OWNED/non-repr(C): its `stats`
    // field is `[Option<Box<cram_stats_layout>>; CRAM_DS_END]`, so it is not
    // byte-identical to a C-ABI mirror. Use the owned type directly. All fields
    // the pipeline reads (length, ref_seq_*, comp_hdr, comp_hdr_block, slice,
    // curr_slice, max_slice, num_records, unsorted, ...) keep identical names;
    // `comp_hdr`/`comp_hdr_block`/`slice` are `*mut *_layout`, `.cast()` to the
    // local mirrors at use sites where needed.
    use super::cram_container_layout as cram_container;

    // The production `refs_t_layout` is OWNED/non-repr(C): its pool is
    // `Option<Box<StringPool>>` and fn_ is `Vec<u8>`. Use the owned type
    // directly. The pipeline only reads ABI-trivial fields here (refs_t:
    // lock/nref/ref_id), so no owned field is reinterpreted through a mirror.
    // refs_t.ref_id is `*mut *mut ref_entry_layout`, so the ref_entry reads
    // (only `.length`) go straight to the owned `super::ref_entry_layout` and
    // need no local alias.
    use super::refs_t_layout as refs_t;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct cram_range {
        pub refid: i32,
        pub start: i64,
        pub end: i64,
    }

    #[repr(C)]
    pub struct cram_fd {
        pub fp: *mut hFILE,
        pub mode: i32,
        pub version: i32,
        pub file_def: *mut (),
        pub header: *mut sam_hdr_t,
        pub prefix: *mut u8,
        pub record_counter: i64,
        pub err: i32,
        pub ctr: *mut cram_container,
        pub ctr_mt: *mut cram_container,
        pub first_base: i32,
        pub last_base: i32,
        pub refs: *mut refs_t,
        pub ref_0: *mut u8,
        pub ref_free: *mut u8,
        pub ref_id: i32,
        pub ref_start: i64,
        pub ref_end: i64,
        pub ref_fn: *mut u8,
        pub level: i32,
        pub m: [*mut (); 47],
        pub tags_used: *mut (),
        pub decode_md: i32,
        pub seqs_per_slice: i32,
        pub bases_per_slice: i32,
        pub slices_per_container: i32,
        pub embed_ref: i32,
        pub no_ref: i32,
        pub no_ref_counter: i32,
        pub ignore_md5: i32,
        pub use_bz2: i32,
        pub use_rans: i32,
        pub use_lzma: i32,
        pub use_fqz: i32,
        pub use_tok: i32,
        pub use_arith: i32,
        pub shared_ref: i32,
        pub required_fields: u32,
        pub store_md: i32,
        pub store_nm: i32,
        pub range: cram_range,
        pub bam_flag_swap: [u32; 0x1000],
        pub cram_flag_swap: [u32; 0x1000],
        pub L1: [u8; 256],
        pub L2: [u8; 256],
        pub cram_sub_matrix: [[u8; 32]; 32],
        pub index_sz: i32,
        pub index: *mut (),
        pub first_container: off_t,
        pub curr_position: off_t,
        pub eof: i32,
        pub last_slice: i32,
        pub last_ri_count: i32,
        pub multi_seq: i32,
        pub multi_seq_user: i32,
        pub unsorted: i32,
        pub last_mapped: i32,
        pub empty_container: i32,
        pub own_pool: i32,
        pub pool: *mut (),
        pub rqueue: *mut (),
        pub metrics_lock: libc::pthread_mutex_t,
        pub ref_lock: libc::pthread_mutex_t,
        pub range_lock: libc::pthread_mutex_t,
        pub bl: *mut (),
        pub bam_list_lock: libc::pthread_mutex_t,
        pub job_pending: *mut (),
        pub ooc: i32,
        pub lossy_read_names: i32,
        pub tlen_approx: i32,
        pub tlen_zero: i32,
        pub idxfp: *mut BGZF,
        pub vv: [u8; std::mem::size_of::<super::varint_vec_layout>()],
        pub ap_delta: i32,
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
        assert!(
            std::mem::size_of::<cram_block>() == std::mem::size_of::<super::cram_block_layout>()
        );
        // cram_container, cram_block_compression_hdr, ref_entry and refs_t are
        // now type aliases of the OWNED, non-repr(C) `super::*_layout` structs
        // (those diverged from the C ABI: cram_container.stats is an array of
        // Option<Box<..>>, cram_block_compression_hdr.td_keys / refs_t.pool are
        // Option<Box<..>>, and ref_entry/refs_t name/fn_/seq are Vec<u8>). The
        // pipeline accesses them as the owned types directly, so no mirror/layout
        // cast is performed for them and no size/offset cross-check is needed.
        assert!(
            std::mem::size_of::<cram_block_slice_hdr>()
                == std::mem::size_of::<super::cram_block_slice_hdr_layout>()
        );
        // Field-offset cross-checks for the cram_fd fields the pipeline reads via casts.
        assert!(
            std::mem::offset_of!(cram_fd, refs)
                == std::mem::offset_of!(super::cram_fd_layout, refs)
        );
        assert!(
            std::mem::offset_of!(cram_fd, version)
                == std::mem::offset_of!(super::cram_fd_layout, version)
        );
        assert!(
            std::mem::offset_of!(cram_fd, required_fields)
                == std::mem::offset_of!(super::cram_fd_layout, required_fields)
        );
        assert!(
            std::mem::offset_of!(cram_fd, L1) == std::mem::offset_of!(super::cram_fd_layout, l1)
        );
        assert!(
            std::mem::offset_of!(cram_fd, ref_lock)
                == std::mem::offset_of!(super::cram_fd_layout, ref_lock)
        );
    };

    // =======================================================================
    // Dependency shims: cast our mirror pointer types to the production
    // hts_sys / layout types the native functions expect, and delegate.
    // =======================================================================
    unsafe fn cram_uncompress_block(b: *mut cram_block) -> i32 {
        super::cram_uncompress_block(&mut *b.cast())
    }
    unsafe fn cram_get_block_by_id(s: *mut cram_slice, id: i32) -> *mut cram_block {
        cram_cram_io_h_183_cram_get_block_by_id(s.cast(), id).cast()
    }
    unsafe fn block_resize_exact(b: *mut cram_block, len: size_t) -> i32 {
        cram_cram_io_h_216_block_resize_exact(b.cast(), len)
    }
    unsafe fn block_resize(b: *mut cram_block, len: size_t) -> i32 {
        cram_cram_io_h_226_block_resize(b.cast(), len)
    }
    unsafe fn block_append(b: *mut cram_block, s: *const (), len: size_t) -> i32 {
        cram_cram_io_h_248_block_append(b.cast(), s, len)
    }
    unsafe fn block_append_char(b: *mut cram_block, c: u8) -> i32 {
        cram_cram_io_h_261_block_append_char(b.cast(), c)
    }
    unsafe fn block_append_uint(b: *mut cram_block, i: u32) -> i32 {
        cram_cram_io_h_271_block_append_uint(b.cast(), i)
    }
    unsafe fn append_uint64(cp: *mut u8, i: uint64_t) -> *mut u8 {
        cram_cram_io_h_340_append_uint64(cp, i)
    }
    unsafe fn cram_codec_to_id(c: *mut cram_codec, id2: *mut i32) -> i32 {
        cram_cram_codecs_c_3968_cram_codec_to_id(c.cast(), id2)
    }
    unsafe fn cram_get_ref(
        fd: *mut cram_fd,
        id: i32,
        start: hts_pos_t,
        end: hts_pos_t,
    ) -> *mut u8 {
        cram_cram_io_c_3409_cram_get_ref(fd.cast(), id, start, end)
    }
    unsafe fn cram_ref_decr(r: *mut refs_t, id: i32) {
        cram_cram_io_c_3213_cram_ref_decr(r.cast(), id);
    }
    unsafe fn cram_free_block(b: *mut cram_block) {
        super::cram_free_block(b.cast());
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
    unsafe fn cram_seek(fd: *mut cram_fd, off: off_t, whence: i32) -> i32 {
        super::cram_seek(fd.cast(), off, whence)
    }
    unsafe fn cram_decode_compression_header(
        fd: *mut cram_fd,
        b: *mut cram_block,
    ) -> *mut cram_block_compression_hdr {
        cram_cram_decode_c_145_cram_decode_compression_header(fd.cast(), b.cast()).cast()
    }

    // ---- thread-pool shims (pool path; unexercised when fd.pool is null) ----
    // errno is a genuine OS thread-local; read it via libc at the syscall-like
    // dispatch boundary.
    use libc::__errno_location;
    use crate::htslib_rs::thread_pool::{
        hts_tpool_delete_result, hts_tpool_dispatch2, hts_tpool_next_result_wait,
        hts_tpool_process_empty, hts_tpool_process_len, hts_tpool_process_qsize,
        hts_tpool_process_sz, hts_tpool_result_data,
    };
    const EAGAIN: i32 = libc::EAGAIN;
    pub enum hts_tpool {}
    pub enum hts_tpool_process {}
    pub enum hts_tpool_result {}

    #[repr(C)]
    pub struct cram_decode_job {
        pub fd: Option<NonNull<cram_fd>>,
        pub c: Option<NonNull<cram_container>>,
        pub s: Option<NonNull<cram_slice>>,
        pub h: Option<NonNull<sam_hdr_t>>,
        pub exit_code: i32,
    }

    // original: cram_ds_unique (htslib/cram/cram_decode.c:876)
    unsafe fn cram_ds_unique(
        hdr: *mut cram_block_compression_hdr,
        c: *mut cram_codec,
        id: i32,
    ) -> i32 {
        let mut n_id: i32 = 0;
        let mut e_type: cram_encoding = E_NULL;
        let mut i = 0;
        while i < DS_END {
            let c_0: *mut cram_codec = (*hdr).codecs[i as usize].cast();
            let bnum1: i32;
            let mut bnum2: i32 = 0;
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
        if n_id == 1 {
            e_type as i32
        } else {
            0
        }
    }

    // original: cram_decode_estimate_sizes (htslib/cram/cram_decode.c:912)
    unsafe fn cram_decode_estimate_sizes(
        hdr: *mut cram_block_compression_hdr,
        s: *mut cram_slice,
        qual_size: *mut i32,
        name_size: *mut i32,
        q_id: *mut i32,
    ) {
        *qual_size = 0;
        *name_size = 0;
        let mut cd: *mut cram_codec = (*hdr).codecs[DS_QS as usize].cast();
        if cd.is_null() {
            return;
        }
        let mut bnum2: i32 = 0;
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
        cd = (*hdr).codecs[DS_RN as usize].cast();
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
    ) -> i32 {
        static i_to_id: [i32; 28] = [
            DS_BF, DS_AP, DS_FP, DS_RL, DS_DL, DS_NF, DS_BA, DS_QS, DS_FC, DS_FN, DS_BS, DS_IN,
            DS_RG, DS_MQ, DS_TL, DS_RN, DS_NS, DS_NP, DS_TS, DS_MF, DS_CF, DS_RI, DS_RS, DS_PD,
            DS_HC, DS_SC, DS_BB, DS_QQ,
        ];
        let mut core_used: i32 = 0;
        if (*fd).required_fields != 0 && (*fd).required_fields != INT_MAX as u32 {
            (*s).data_series = 0;
            if (*fd).required_fields & SAM_QNAME as u32 != 0 {
                (*s).data_series |= CRAM_RN as u32;
            }
            if (*fd).required_fields & SAM_FLAG as u32 != 0 {
                (*s).data_series |= CRAM_BF as u32;
            }
            if (*fd).required_fields & SAM_RNAME as u32 != 0 {
                (*s).data_series |= (CRAM_RI | CRAM_BF) as u32;
            }
            if (*fd).required_fields & SAM_POS as u32 != 0 {
                (*s).data_series |= (CRAM_AP | CRAM_BF) as u32;
            }
            if (*fd).required_fields & SAM_MAPQ as u32 != 0 {
                (*s).data_series |= CRAM_MQ as u32;
            }
            if (*fd).required_fields & SAM_CIGAR as u32 != 0 {
                (*s).data_series |= (CRAM_FN
                    | CRAM_FP
                    | CRAM_FC
                    | CRAM_DL
                    | CRAM_IN
                    | CRAM_SC
                    | CRAM_HC
                    | CRAM_PD
                    | CRAM_RS
                    | CRAM_RL
                    | CRAM_BF) as u32;
            }
            if (*fd).required_fields & SAM_RNEXT as u32 != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF | CRAM_RI | CRAM_NS | CRAM_BF) as u32;
            }
            if (*fd).required_fields & SAM_PNEXT as u32 != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF | CRAM_AP | CRAM_NP | CRAM_BF) as u32;
            }
            if (*fd).required_fields & SAM_TLEN as u32 != 0 {
                (*s).data_series |= (CRAM_CF
                    | CRAM_NF
                    | CRAM_AP
                    | CRAM_TS
                    | CRAM_BF
                    | CRAM_MF
                    | CRAM_RI
                    | (CRAM_FN
                        | CRAM_FP
                        | CRAM_FC
                        | CRAM_DL
                        | CRAM_IN
                        | CRAM_SC
                        | CRAM_HC
                        | CRAM_PD
                        | CRAM_RS
                        | CRAM_RL
                        | CRAM_BF)) as u32;
            }
            if (*fd).required_fields & SAM_SEQ as u32 != 0 {
                (*s).data_series |= (CRAM_FN
                    | CRAM_FP
                    | CRAM_FC
                    | CRAM_DL
                    | CRAM_IN
                    | CRAM_SC
                    | CRAM_HC
                    | CRAM_PD
                    | CRAM_RS
                    | CRAM_RL
                    | CRAM_BF
                    | CRAM_BA
                    | CRAM_BS
                    | CRAM_RL
                    | CRAM_AP
                    | CRAM_BB) as u32;
            }
            if (*fd).required_fields & SAM_AUX as u32 == 0 {
                (*s).decode_md = 0;
            }
            if (*fd).required_fields & SAM_QUAL as u32 != 0 {
                (*s).data_series |= (CRAM_FN
                    | CRAM_FP
                    | CRAM_FC
                    | CRAM_DL
                    | CRAM_IN
                    | CRAM_SC
                    | CRAM_HC
                    | CRAM_PD
                    | CRAM_RS
                    | CRAM_RL
                    | CRAM_BF
                    | CRAM_RL
                    | CRAM_AP
                    | CRAM_QS
                    | CRAM_QQ) as u32;
            }
            if (*fd).required_fields & SAM_AUX as u32 != 0 {
                (*s).data_series |= (CRAM_RG | CRAM_TL | CRAM_aux) as u32;
            }
            if (*fd).required_fields & SAM_RGAUX as u32 != 0 {
                (*s).data_series |= (CRAM_RG | CRAM_BF) as u32;
            }
            if cram_uncompress_block(*(*s).block.offset(0)) != 0 {
                return -1;
            }
        } else {
            (*s).data_series = CRAM_ALL as u32;
            let mut i = 0;
            while i < (*(*s).hdr).num_blocks {
                if cram_uncompress_block(*(*s).block.offset(i as isize)) != 0 {
                    return -1;
                }
                i += 1;
            }
            return 0;
        }
        let mut block_used_vec: Vec<i32> = vec![0; ((*(*s).hdr).num_blocks + 1) as usize];
        let block_used = block_used_vec.as_mut_ptr();
        loop {
            if (*s).data_series & CRAM_RS as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_PD as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_HC as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_QS as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_IN as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_SC as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_BS as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_DL as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_BA as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_BB as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series & CRAM_QQ as u32 != 0 {
                (*s).data_series |= (CRAM_FC | CRAM_FP) as u32;
            }
            if (*s).data_series
                & (CRAM_FN
                    | CRAM_FP
                    | CRAM_FC
                    | CRAM_DL
                    | CRAM_IN
                    | CRAM_SC
                    | CRAM_HC
                    | CRAM_PD
                    | CRAM_RS
                    | CRAM_RL
                    | CRAM_BF
                    | CRAM_BA
                    | CRAM_BS
                    | CRAM_RL
                    | CRAM_AP
                    | CRAM_BB
                    | (CRAM_FN
                        | CRAM_FP
                        | CRAM_FC
                        | CRAM_DL
                        | CRAM_IN
                        | CRAM_SC
                        | CRAM_HC
                        | CRAM_PD
                        | CRAM_RS
                        | CRAM_RL
                        | CRAM_BF)) as u32
                != 0
            {
                (*s).data_series |= CRAM_RL as u32;
            }
            if (*s).data_series & CRAM_FP as u32 != 0 {
                (*s).data_series |= CRAM_FC as u32;
            }
            if (*s).data_series & CRAM_FC as u32 != 0 {
                (*s).data_series |= CRAM_FN as u32;
            }
            if (*s).data_series & CRAM_aux as u32 != 0 {
                (*s).data_series |= CRAM_TL as u32;
            }
            if (*s).data_series & CRAM_MF as u32 != 0 {
                (*s).data_series |= CRAM_CF as u32;
            }
            if (*s).data_series & CRAM_MQ as u32 != 0 {
                (*s).data_series |= CRAM_BF as u32;
            }
            if (*s).data_series & CRAM_BS as u32 != 0 {
                (*s).data_series |= CRAM_RI as u32;
            }
            if (*s).data_series & (CRAM_MF | CRAM_NS | CRAM_NP | CRAM_TS | CRAM_NF) as u32 != 0 {
                (*s).data_series |= CRAM_CF as u32;
            }
            if (*hdr).read_names_included == 0 && (*s).data_series & CRAM_RN as u32 != 0 {
                (*s).data_series |= (CRAM_CF | CRAM_NF) as u32;
            }
            if (*s).data_series & (CRAM_BA | CRAM_QS | CRAM_BB | CRAM_QQ) as u32 != 0 {
                (*s).data_series |= (CRAM_BF | CRAM_CF | CRAM_RL) as u32;
            }
            if (*s).data_series & CRAM_FN as u32 != 0 {
                (*s).data_series |= (CRAM_SC | CRAM_IN | CRAM_BB) as u32;
            }
            let orig_ds = (*s).data_series;
            let mut i = 0usize;
            while i < 28 {
                let mut bnum1: i32;
                let mut bnum2: i32 = 0;
                let c: *mut cram_codec = (*hdr).codecs[i_to_id[i] as usize].cast();
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
                                        if cram_uncompress_block(*(*s).block.offset(j as isize))
                                            != 0
                                        {
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
            if (*fd).required_fields & SAM_AUX as u32 != 0
                || (*s).data_series & CRAM_aux as u32 != 0
            {
                let mut i = 0;
                while i < CRAM_MAP_HASH {
                    let mut m: *mut cram_map = (*hdr).tag_encoding_map[i as usize].cast();
                    while !m.is_null() {
                        let c_0: *mut cram_codec = (*m).codec;
                        if c_0.is_null() {
                            m = (*m).next;
                            continue;
                        }
                        let mut bnum2: i32 = 0;
                        let mut bnum1 = cram_codec_to_id(c_0, &raw mut bnum2);
                        loop {
                            match bnum1 {
                                -2 => {}
                                -1 => core_used = 1,
                                _ => {
                                    let mut j = 0;
                                    while j < (*(*s).hdr).num_blocks {
                                        if (**(*s).block.offset(j as isize)).content_type
                                            == EXTERNAL
                                            && (**(*s).block.offset(j as isize)).content_id == bnum1
                                        {
                                            *block_used.offset(j as isize) = 1;
                                            if cram_uncompress_block(*(*s).block.offset(j as isize))
                                                != 0
                                            {
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
                let mut bnum2: i32 = 0;
                let c_1: *mut cram_codec = (*hdr).codecs[i_to_id[i] as usize].cast();
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
                let mut m_0: *mut cram_map = (*hdr).tag_encoding_map[i as usize].cast();
                while !m_0.is_null() {
                    let c_2: *mut cram_codec = (*m_0).codec;
                    if c_2.is_null() {
                        m_0 = (*m_0).next;
                        continue;
                    }
                    let mut bnum2: i32 = 0;
                    let mut bnum1 = cram_codec_to_id(c_2, &raw mut bnum2);
                    loop {
                        match bnum1 {
                            -2 => {}
                            -1 => (*s).data_series |= CRAM_aux as u32,
                            _ => {
                                let mut j = 0;
                                while j < (*(*s).hdr).num_blocks {
                                    if (**(*s).block.offset(j as isize)).content_type == EXTERNAL
                                        && (**(*s).block.offset(j as isize)).content_id == bnum1
                                        && *block_used.offset(j as isize) != 0
                                    {
                                        (*s).data_series |= CRAM_aux as u32;
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
        drop(block_used_vec);
        0
    }

    // original: add_md_char (htslib/cram/cram_decode.c:1080)
    #[inline]
    unsafe fn add_md_char(
        s: *mut cram_slice,
        decode_md: i32,
        c: u8,
        md_dist: *mut int32_t,
    ) -> i32 {
        if decode_md != 0 {
            if block_append_uint((*s).aux_blk, *md_dist as u32) < 0 {
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
    unsafe fn map_find(map: *mut *mut cram_map, key: *mut u8, id: i32) -> *mut cram_map {
        let mut m: *mut cram_map = *map.offset(
            ((*key.offset(0) as i32 * 3 + *key.offset(1) as i32) & (CRAM_MAP_HASH - 1))
                as isize,
        );
        while !m.is_null() && (*m).key != id {
            m = (*m).next;
        }
        m
    }

    // original: aux_ele_size (htslib/cram/cram_decode.c:1989)
    #[inline]
    unsafe fn aux_ele_size(type_0: uint8_t) -> i32 {
        match type_0 as i32 {
            65 | 99 | 67 => 1,
            115 | 83 => 2,
            105 | 73 | 102 => 4,
            100 => 8,
            _ => 1,
        }
    }

    // original: md5_print (htslib/cram/cram_decode.c:2305)
    unsafe fn md5_print(md5: *mut u8, out: *mut u8) -> *mut u8 {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut i = 0;
        while i < 16 {
            *out.offset((i * 2) as isize) =
                HEX[(*md5.offset(i as isize) as i32 >> 4) as usize] as u8;
            *out.offset((i * 2 + 1) as isize) =
                HEX[(*md5.offset(i as isize) as i32 & 15) as usize] as u8;
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
    ) -> i32 {
        let mut out_sz: i32 = 1;
        let mut r: i32 = 0;
        if (*(*c).comp_hdr).codecs[DS_TS as usize].cast::<cram_codec>().is_null() {
            return -1;
        }
        if (*fd).version >> 8 < 4 {
            let mut i32: int32_t = 0;
            r |= (*(*(*c).comp_hdr).codecs[DS_TS as usize].cast::<cram_codec>())
                .decode
                .expect("decode")(
                s,
                (*(*c).comp_hdr).codecs[DS_TS as usize].cast::<cram_codec>(),
                blk,
                &raw mut i32 as *mut u8,
                &raw mut out_sz,
            );
            *tlen = i32 as int64_t;
        } else {
            r |= (*(*(*c).comp_hdr).codecs[DS_TS as usize].cast::<cram_codec>())
                .decode
                .expect("decode")(
                s,
                (*(*c).comp_hdr).codecs[DS_TS as usize].cast::<cram_codec>(),
                blk,
                tlen as *mut u8,
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
    ) -> i32 {
        let mut r: i32 = 0;
        let mut out_sz: i32 = 1;
        let mut ntags: u8 = 0;
        if (*(*c).comp_hdr).codecs[DS_TC as usize].cast::<cram_codec>().is_null() {
            return -1;
        }
        r |= (*(*(*c).comp_hdr).codecs[DS_TC as usize].cast::<cram_codec>())
            .decode
            .expect("decode")(
            s,
            (*(*c).comp_hdr).codecs[DS_TC as usize].cast::<cram_codec>(),
            blk,
            &raw mut ntags as *mut u8,
            &raw mut out_sz,
        );
        (*cr).ntags = ntags as int32_t;
        (*cr).aux_size = 0;
        (*cr).aux = (*(*s).aux_blk).byte as uint32_t;
        let mut i = 0;
        while i < (*cr).ntags {
            let mut id: int32_t = 0;
            let mut out_sz_0: int32_t = 1;
            let mut tag_data: [u8; 3] = [0; 3];
            if (*(*c).comp_hdr).codecs[DS_TN as usize].cast::<cram_codec>().is_null() {
                return -1;
            }
            r |= (*(*(*c).comp_hdr).codecs[DS_TN as usize].cast::<cram_codec>())
                .decode
                .expect("decode")(
                s,
                (*(*c).comp_hdr).codecs[DS_TN as usize].cast::<cram_codec>(),
                blk,
                &raw mut id as *mut u8,
                &raw mut out_sz_0,
            );
            if out_sz_0 == 3 {
                memcpy(
                    &raw mut tag_data as *mut u8 as *mut (),
                    &raw mut id as *const (),
                    3,
                );
            } else {
                tag_data[0] = (id >> 16 & 0xff) as u8;
                tag_data[1] = (id >> 8 & 0xff) as u8;
                tag_data[2] = (id & 0xff) as u8;
            }
            let m = map_find(
                &raw mut (*(*c).comp_hdr).tag_encoding_map as *mut *mut cram_map,
                &raw mut tag_data as *mut u8,
                id,
            );
            if m.is_null() {
                return -1;
            }
            if block_append(
                (*s).aux_blk,
                &raw mut tag_data as *mut u8 as *const (),
                3,
            ) < 0
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
                (*s).aux_blk as *mut u8,
                &raw mut out_sz_0,
            );
            (*cr).aux_size = (*cr).aux_size.wrapping_add((out_sz_0 + 3) as u32);
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
        has_MD: *mut i32,
        has_NM: *mut i32,
    ) -> i32 {
        let mut r: i32 = 0;
        let mut out_sz: i32 = 1;
        let mut TL: int32_t = 0;
        let ds: uint32_t = (*s).data_series;
        if ds & (CRAM_TL | CRAM_aux) as u32 == 0 {
            (*cr).aux = 0;
            (*cr).aux_size = 0;
            return 0;
        }
        if (*(*c).comp_hdr).codecs[DS_TL as usize].cast::<cram_codec>().is_null() {
            return -1;
        }
        r |= (*(*(*c).comp_hdr).codecs[DS_TL as usize].cast::<cram_codec>())
            .decode
            .expect("decode")(
            s,
            (*(*c).comp_hdr).codecs[DS_TL as usize].cast::<cram_codec>(),
            blk,
            &raw mut TL as *mut u8,
            &raw mut out_sz,
        );
        if r != 0 || TL < 0 || TL >= (*(*c).comp_hdr).ntl {
            return -1;
        }
        let mut TN: *mut u8 = *(*(*c).comp_hdr).tl.offset(TL as isize);
        (*cr).ntags = (strlen(TN as *mut u8) / 3) as int32_t;
        (*cr).aux_size = 0;
        (*cr).aux = (*(*s).aux_blk).byte as uint32_t;
        if ds & CRAM_aux as u32 == 0 {
            return 0;
        }
        let mut i = 0;
        while i < (*cr).ntags {
            let mut out_sz_0: int32_t = 1;
            let mut tag_data: [u8; 7] = [0; 7];
            if *TN.offset(0) as i32 == 'M' as i32
                && *TN.offset(1) as i32 == 'D' as i32
                && !has_MD.is_null()
            {
                *has_MD = (*(*s).aux_blk).byte.wrapping_add(3).wrapping_mul(
                    (if *TN.offset(2) as i32 == '*' as i32 {
                        -1i32
                    } else {
                        1
                    }) as size_t,
                ) as i32;
            }
            if *TN.offset(0) as i32 == 'N' as i32
                && *TN.offset(1) as i32 == 'M' as i32
                && !has_NM.is_null()
            {
                *has_NM = (*(*s).aux_blk).byte.wrapping_add(3).wrapping_mul(
                    (if *TN.offset(2) as i32 == '*' as i32 {
                        -1i32
                    } else {
                        1
                    }) as size_t,
                ) as i32;
            }
            tag_data[0] = *TN.offset(0);
            tag_data[1] = *TN.offset(1);
            tag_data[2] = *TN.offset(2);
            let id = ((tag_data[0] as i32) << 16
                | (tag_data[1] as i32) << 8
                | tag_data[2] as i32) as int32_t;
            if (*fd).version >> 8 >= 4 && *TN.offset(2) as i32 == '*' as i32 {
                let mut tag_data_size: i32 = 0;
                let mut handled = false;
                if *TN.offset(0) as i32 == 'N' as i32 && *TN.offset(1) as i32 == 'M' as i32 {
                    memcpy(
                        (&raw mut tag_data as *mut u8).offset(2) as *mut (),
                        b"I\0\0\0\0\0" as *const u8 as *const u8 as *const (),
                        5,
                    );
                    tag_data_size = 7;
                } else if *TN.offset(0) as i32 == 'R' as i32
                    && *TN.offset(1) as i32 == 'G' as i32
                {
                    TN = TN.offset(3);
                    let rg = sam_hdr_line_name(
                        &mut *(*fd).header,
                        b"RG",
                        (*cr).rg,
                    );
                    if !rg.is_null() {
                        let rg_len = strlen(rg);
                        tag_data[2] = 'Z' as i32 as u8;
                        if block_append(
                            (*s).aux_blk,
                            &raw mut tag_data as *mut u8 as *const (),
                            3,
                        ) < 0
                        {
                            return -1;
                        }
                        if block_append((*s).aux_blk, rg as *const (), rg_len) < 0 {
                            return -1;
                        }
                        if block_append_char((*s).aux_blk, 0) < 0 {
                            return -1;
                        }
                        (*cr).aux_size = (*cr).aux_size.wrapping_add((3 + rg_len + 1) as u32);
                        (*cr).rg = -1;
                    }
                    handled = true;
                } else {
                    tag_data[2] = 'Z' as i32 as u8;
                    tag_data_size = 3;
                }
                if !handled {
                    if block_append(
                        (*s).aux_blk,
                        &raw mut tag_data as *mut u8 as *const (),
                        tag_data_size as size_t,
                    ) < 0
                    {
                        return -1;
                    }
                    (*cr).aux_size = (*cr).aux_size.wrapping_add(tag_data_size as u32);
                    TN = TN.offset(3);
                }
            } else {
                TN = TN.offset(3);
                let m = map_find(
                    &raw mut (*(*c).comp_hdr).tag_encoding_map as *mut *mut cram_map,
                    &raw mut tag_data as *mut u8,
                    id,
                );
                if m.is_null() {
                    return -1;
                }
                if block_append(
                    (*s).aux_blk,
                    &raw mut tag_data as *mut u8 as *const (),
                    3,
                ) < 0
                {
                    return -1;
                }
                if (*m).codec.is_null() {
                    return -1;
                }
                if (*(*m).codec).codec == E_BYTE_ARRAY_LEN
                    || (*(*m).codec).codec == E_BYTE_ARRAY_STOP
                {
                    out_sz_0 *= aux_ele_size(*TN.offset(-1) as uint8_t);
                }
                r |= (*(*m).codec).decode.expect("decode")(
                    s,
                    (*m).codec,
                    blk,
                    (*s).aux_blk as *mut u8,
                    &raw mut out_sz_0,
                );
                if r != 0 {
                    return r;
                }
                (*cr).aux_size = (*cr).aux_size.wrapping_add((out_sz_0 + 3) as u32);
                if *TN.offset(-3) as i32 == 'c' as i32
                    && *TN.offset(-2) as i32 == 'F' as i32
                    && *TN.offset(-1) as i32 == 'C' as i32
                    && out_sz_0 == 1
                {
                    let cF: uint8_t = *((*(*s).aux_blk).data.add((*(*s).aux_blk).byte)).offset(-1);
                    (*(*s).aux_blk).byte =
                        (*(*s).aux_blk).byte.wrapping_sub((out_sz_0 + 3) as size_t);
                    (*cr).aux_size = (*cr).aux_size.wrapping_sub((out_sz_0 + 3) as u32);
                    if cF as i32 & 1 != 0 && !has_MD.is_null() && *has_MD == 0 {
                        *has_MD = 1;
                    }
                    if cF as i32 & 2 != 0 && !has_NM.is_null() && *has_NM == 0 {
                        *has_NM = 1;
                    }
                }
            }
            if (*(*s).aux_blk).byte > (1u32 << 31) as size_t {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_decode_aux\0" as *const u8 as *const u8,
                    b"CRAM->BAM aux block size overflow\0" as *const u8 as *const u8
                );
                return -1;
            }
            i += 1;
        }
        r
    }

    // original: cram_decode_slice_xref (htslib/cram/cram_decode.c:2140)
    unsafe fn cram_decode_slice_xref(s: *mut cram_slice, required_fields: i32) -> i32 {
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
                                && ((*(*s).crecs.offset(id2 as isize)).aend < aright
                                    || left_cnt <= 1)
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
                        b"cram_decode_slice_xref\0" as *const u8 as *const u8,
                        b"Mate line out of bounds\0" as *const u8 as *const u8
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
    #[allow(clippy::too_many_arguments)]
    unsafe fn cram_decode_seq(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        blk: *mut cram_block,
        cr: *mut cram_record,
        sh: *mut sam_hdr_t,
        cf: i32,
        seq: *mut u8,
        qual: *mut u8,
        has_MD: i32,
        mut has_NM: i32,
    ) -> i32 {
        let mut current_block: u64;
        let mut prev_pos: i32 = 0;
        let mut f: i32;
        let mut r: i32 = 0;
        let mut out_sz: i32 = 1;
        let mut seq_pos: i32 = 1;
        let mut cig_len: i32 = 0;
        let mut ref_pos: int64_t = (*cr).apos;
        let mut fn_0: int32_t = 0;
        let mut i32: int32_t = 0;
        let mut cig_op: cigar_op = BAM_CMATCH_;
        let mut cigar: *mut uint32_t = (*s).cigar;
        let mut ncigar: uint32_t = (*s).ncigar;
        let mut cigar_alloc: uint32_t = (*s).cigar_alloc;
        let mut nm: uint32_t = 0;
        let mut md_dist: int32_t = 0;
        let mut orig_aux: i32 = 0;
        let do_md: i32 = if (*fd).version >> 8 >= 4 {
            ((*s).decode_md > 0) as i32
        } else {
            ((*s).decode_md != 0) as i32
        };
        let mut decode_md: i32 = (!(*s).ref_0.is_null()
            && (*cr).ref_id >= 0
            && (do_md != 0 && has_MD == 0 || has_MD < 0))
            as i32;
        let mut decode_nm: i32 = (!(*s).ref_0.is_null()
            && (*cr).ref_id >= 0
            && (do_md != 0 && has_NM == 0 || has_NM < 0))
            as i32;
        let ds: uint32_t = (*s).data_series;
        let bfd: *mut sam_hrecs_t = (*sh).hrecs;
        let comp = (*c).comp_hdr;
        macro_rules! codec {
            ($id:expr) => {
                (*comp).codecs[$id as usize].cast::<cram_codec>()
            };
        }
        macro_rules! decode {
            ($id:expr, $out:expr, $sz:expr) => {{
                (*codec!($id)).decode.expect("decode")(s, codec!($id), blk, $out, $sz)
            }};
        }
        if ds & CRAM_QS as u32 != 0 && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0 {
            memset(qual as *mut (), 255, (*cr).len as size_t);
        }
        if (*cr).cram_flags & CRAM_FLAG_NO_SEQ != 0 {
            decode_nm = 0;
            decode_md = 0;
        }
        if decode_md != 0 {
            orig_aux = (*(*s).aux_blk).byte as i32;
            if has_MD == 0
                && block_append(
                    (*s).aux_blk,
                    b"MDZ\0" as *const u8 as *const u8 as *const (),
                    3,
                ) < 0
            {
                return -1;
            }
        }
        // -- feature loop --
        if ds & CRAM_FN as u32 != 0 {
            if codec!(DS_FN).is_null() {
                return -1;
            }
            r |= decode!(DS_FN, &raw mut fn_0 as *mut u8, &raw mut out_sz);
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
            if ds & (CRAM_FC | CRAM_FP) as u32 == 0 {
                break 'feature_done;
            }
            if fn_0 != 0 {
                if ds & CRAM_FC as u32 != 0 && codec!(DS_FC).is_null() {
                    return -1;
                }
                if ds & CRAM_FP as u32 != 0 && codec!(DS_FP).is_null() {
                    return -1;
                }
            }
            f = 0;
            while f < fn_0 {
                let mut pos: int32_t = 0;
                let mut op: u8 = 0;
                if ncigar.wrapping_add(2) >= cigar_alloc {
                    cigar_alloc = if cigar_alloc != 0 {
                        cigar_alloc.wrapping_mul(2)
                    } else {
                        1024
                    };
                    cigar = libc::realloc((*s).cigar as *mut libc::c_void, cigar_alloc as usize * std::mem::size_of::<uint32_t>()).cast::<uint32_t>();
                    if cigar.is_null() {
                        return -1;
                    }
                    (*s).cigar = cigar;
                }
                if ds & CRAM_FC as u32 != 0 {
                    r |= decode!(DS_FC, &raw mut op, &raw mut out_sz);
                    if r != 0 {
                        return r;
                    }
                }
                if ds & CRAM_FP as u32 != 0 {
                    r |= decode!(DS_FP, &raw mut pos as *mut u8, &raw mut out_sz);
                    if r != 0 {
                        return r;
                    }
                    pos += prev_pos;
                    if pos <= 0 {
                        hts_log!(
                            HTS_LOG_ERROR,
                            b"cram_decode_seq\0" as *const u8 as *const u8,
                            b"Feature position before start of read\0" as *const u8
                                as *const u8
                        );
                        return -1;
                    }
                    if (*cr).len != 0 && pos > (*cr).len {
                        let valid_end = if op as i32 == 'N' as i32
                            || op as i32 == 'P' as i32
                            || op as i32 == 'H' as i32
                            || op as i32 == 'D' as i32
                        {
                            (*cr).len + 1
                        } else {
                            (*cr).len
                        };
                        if pos > valid_end {
                            hts_log!(
                                HTS_LOG_ERROR,
                                b"cram_decode_seq\0" as *const u8 as *const u8,
                                b"Feature position after end of read\0" as *const u8
                                    as *const u8
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
                                    - ref_pos) as i32;
                                if rlen > 0 {
                                    if ref_pos + rlen as int64_t > (*s).ref_end {
                                        cigar_extends_error = true;
                                        break;
                                    }
                                    if (*cr).len != 0 {
                                        memcpy(
                                            seq.offset((seq_pos - 1) as isize) as *mut (),
                                            (*s).ref_0
                                                .offset((ref_pos - (*s).ref_start + 1) as isize)
                                                as *const (),
                                            rlen as size_t,
                                        );
                                        if pos - seq_pos - rlen > 0 {
                                            memset(
                                                seq.offset((seq_pos - 1 + rlen) as isize)
                                                    as *mut (),
                                                'N' as i32,
                                                (pos - seq_pos - rlen) as size_t,
                                            );
                                        }
                                    }
                                } else if (*cr).len != 0 {
                                    memset(
                                        seq.offset((seq_pos - 1) as isize) as *mut (),
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
                                let refp: *const u8 = (*s)
                                    .ref_0
                                    .offset(ref_pos as isize)
                                    .offset(-((*s).ref_start as isize))
                                    .offset(1);
                                let frag_len = pos - seq_pos;
                                if decode_md != 0 || decode_nm != 0 {
                                    let n = memchr(
                                        refp as *const (),
                                        'N' as i32,
                                        frag_len as size_t,
                                    ) as *mut u8;
                                    if !n.is_null() {
                                        let mut i = 0;
                                        while i < frag_len {
                                            let base = *refp.offset(i as isize);
                                            if base as i32 == 'N' as i32 {
                                                if add_md_char(
                                                    s,
                                                    decode_md,
                                                    'N' as i32 as u8,
                                                    &raw mut md_dist,
                                                ) < 0
                                                {
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
                                        seq.offset((seq_pos - 1) as isize) as *mut (),
                                        refp as *const (),
                                        frag_len as size_t,
                                    );
                                }
                            }
                        }
                        if cig_len != 0 && cig_op != BAM_CMATCH {
                            let t = ncigar;
                            ncigar = ncigar.wrapping_add(1);
                            *cigar.offset(t as isize) =
                                ((cig_len << 4) as u32).wrapping_add(cig_op);
                            cig_len = 0;
                        }
                        cig_op = BAM_CMATCH_;
                        cig_len += pos - seq_pos;
                        ref_pos += (pos - seq_pos) as int64_t;
                        seq_pos = pos;
                    }
                    prev_pos = pos;
                    if ds & CRAM_FC as u32 == 0 {
                        break 'feature_done;
                    }
                    match op as i32 {
                        83 => {
                            // 'S'
                            let mut out_sz2: int32_t = if (*cr).len != 0 {
                                (*cr).len - (pos - 1)
                            } else {
                                1
                            };
                            let mut have_sc = 0;
                            if cig_len != 0 {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
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
                                if ds & CRAM_IN as u32 != 0 {
                                    if !codec!(DS_IN).is_null() {
                                        r |= decode!(DS_IN, seqp(), &raw mut out_sz2);
                                    } else {
                                        if (*cr).len != 0 {
                                            *seq.offset((pos - 1) as isize) = 'N' as i32 as u8;
                                        }
                                        out_sz2 = 1;
                                    }
                                    have_sc = 1;
                                }
                            } else if ds & CRAM_SC as u32 != 0 {
                                if !codec!(DS_SC).is_null() {
                                    r |= decode!(DS_SC, seqp(), &raw mut out_sz2);
                                } else {
                                    if (*cr).len != 0 {
                                        *seq.offset((pos - 1) as isize) = 'N' as i32 as u8;
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
                            let mut base_0: u8 = 0;
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BS as u32 != 0 {
                                if codec!(DS_BS).is_null() {
                                    return -1;
                                }
                                r |=
                                    decode!(DS_BS, &raw mut base_0 as *mut u8, &raw mut out_sz);
                                if r != 0 {
                                    return -1;
                                }
                                if (*cr).ref_id < 0
                                    || ref_pos >= (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                    || (*s).ref_0.is_null()
                                {
                                    if pos - 1 < (*cr).len {
                                        *seq.offset((pos - 1) as isize) = (*comp)
                                            .substitution_matrix
                                            [(*fd).L1['N' as i32 as usize] as usize]
                                            [base_0 as usize] as u8;
                                    }
                                    if decode_md != 0 || decode_nm != 0 {
                                        if md_dist >= 0
                                            && decode_md != 0
                                            && block_append_uint((*s).aux_blk, md_dist as u32)
                                                < 0
                                        {
                                            return -1;
                                        }
                                        md_dist = -1;
                                        nm = nm.wrapping_sub(1);
                                    }
                                } else {
                                    let ref_call: u8 = (if ref_pos < (*s).ref_end {
                                        *(*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                            as uc as i32
                                    } else {
                                        'N' as i32
                                    })
                                        as u8;
                                    let ref_base = (*fd).L1[ref_call as usize] as i32;
                                    if pos - 1 < (*cr).len {
                                        *seq.offset((pos - 1) as isize) =
                                            (*comp).substitution_matrix[ref_base as usize]
                                                [base_0 as usize] as u8;
                                    }
                                    if add_md_char(
                                        s,
                                        decode_md,
                                        ref_call as u8,
                                        &raw mut md_dist,
                                    ) < 0
                                    {
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
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_DL as u32 != 0 {
                                if codec!(DS_DL).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_DL, &raw mut i32 as *mut u8, &raw mut out_sz);
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
                                    if md_dist >= 0
                                        && decode_md != 0
                                        && block_append_uint((*s).aux_blk, md_dist as u32) < 0
                                    {
                                        return -1;
                                    }
                                    if ref_pos + i32 as int64_t
                                        <= (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                    {
                                        if decode_md != 0 {
                                            if block_append_char((*s).aux_blk, '^' as i32 as u8)
                                                < 0
                                            {
                                                return -1;
                                            }
                                            if block_append(
                                                (*s).aux_blk,
                                                (*s).ref_0
                                                    .offset((ref_pos - (*s).ref_start + 1) as isize)
                                                    as *const (),
                                                i32 as size_t,
                                            ) < 0
                                            {
                                                return -1;
                                            }
                                            md_dist = 0;
                                        }
                                        nm = nm.wrapping_add(i32 as u32);
                                    } else {
                                        if (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                                            >= ref_pos
                                        {
                                            if decode_md != 0 {
                                                if block_append_char(
                                                    (*s).aux_blk,
                                                    '^' as i32 as u8,
                                                ) < 0
                                                {
                                                    return -1;
                                                }
                                                if block_append(
                                                    (*s).aux_blk,
                                                    (*s).ref_0.offset(
                                                        (ref_pos - (*s).ref_start + 1) as isize,
                                                    )
                                                        as *const (),
                                                    ((*(*bfd).ref_.offset((*cr).ref_id as isize))
                                                        .len
                                                        - ref_pos)
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
                                                - ((*(*bfd).ref_.offset((*cr).ref_id as isize))
                                                    .len
                                                    - ref_pos))
                                                as uint32_t;
                                            nm = nm
                                                .wrapping_add((i32 as uint32_t).wrapping_sub(dlen));
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
                            let mut out_sz2_0: int32_t = if (*cr).len != 0 {
                                (*cr).len - (pos - 1)
                            } else {
                                1
                            };
                            if cig_len != 0 && cig_op != BAM_CINS {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_IN as u32 != 0 {
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
                                nm = nm.wrapping_add(out_sz2_0 as u32);
                            }
                        }
                        105 => {
                            // 'i'
                            if cig_len != 0 && cig_op != BAM_CINS {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BA as u32 != 0 {
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
                            let mut len: int32_t = if (*cr).len != 0 {
                                (*cr).len - (pos - 1)
                            } else {
                                1
                            };
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BB as u32 != 0 {
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
                                    if md_dist >= 0
                                        && decode_md != 0
                                        && block_append_uint((*s).aux_blk, md_dist as u32) < 0
                                    {
                                        return -1;
                                    }
                                    while x < len {
                                        if x != 0
                                            && decode_md != 0
                                            && block_append_uint((*s).aux_blk, 0) < 0
                                        {
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
                                                    (ref_pos + x as int64_t - (*s).ref_start + 1)
                                                        as isize,
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
                                    nm = nm.wrapping_add(x as u32);
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
                            let mut len_0: int32_t = if (*cr).len != 0 {
                                (*cr).len - (pos - 1)
                            } else {
                                1
                            };
                            if cig_len != 0 && cig_op != BAM_CMATCH {
                                let t = ncigar;
                                ncigar = ncigar.wrapping_add(1);
                                *cigar.offset(t as isize) =
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_QQ as u32 != 0 {
                                if codec!(DS_QQ).is_null() {
                                    return -1;
                                }
                                if ds & CRAM_QS as u32 != 0
                                    && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                    && (*cr).len > 0
                                    && *qual as u8 as i32 == 255
                                {
                                    memset(qual as *mut (), 30, (*cr).len as size_t);
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
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_BA as u32 != 0 {
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
                                    if md_dist >= 0
                                        && decode_md != 0
                                        && block_append_uint((*s).aux_blk, md_dist as u32) < 0
                                    {
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
                                                *(*s).ref_0.offset(
                                                    (ref_pos - (*s).ref_start + 1) as isize,
                                                ),
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
                            if ds & CRAM_QS as u32 != 0 {
                                if codec!(DS_QS).is_null() {
                                    return -1;
                                }
                                if cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                    && (*cr).len > 0
                                    && *qual as u8 as i32 == 255
                                {
                                    memset(qual as *mut (), 30, (*cr).len as size_t);
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
                            if ds & CRAM_QS as u32 != 0 {
                                if codec!(DS_QS).is_null() {
                                    return -1;
                                }
                                if cf & CRAM_FLAG_PRESERVE_QUAL_SCORES == 0
                                    && (*cr).len > 0
                                    && *qual as u8 as i32 == 255
                                {
                                    memset(qual as *mut (), 30, (*cr).len as size_t);
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
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_HC as u32 != 0 {
                                if codec!(DS_HC).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_HC, &raw mut i32 as *mut u8, &raw mut out_sz);
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
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_PD as u32 != 0 {
                                if codec!(DS_PD).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_PD, &raw mut i32 as *mut u8, &raw mut out_sz);
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
                                    ((cig_len << 4) as u32).wrapping_add(cig_op);
                                cig_len = 0;
                            }
                            if ds & CRAM_RS as u32 != 0 {
                                if codec!(DS_RS).is_null() {
                                    return -1;
                                }
                                r |= decode!(DS_RS, &raw mut i32 as *mut u8, &raw mut out_sz);
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
                                b"cram_decode_seq\0" as *const u8 as *const u8,
                                b"Unknown feature code\0" as *const u8 as *const u8
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
                    b"cram_decode_seq\0" as *const u8 as *const u8,
                    b"CRAM CIGAR extends beyond slice reference extents\0" as *const u8
                        as *const u8
                );
                return -1;
            }
            // f-loop finished: tail of feature processing (FN != 0 && len >= seq_pos)
            if ds & CRAM_FC as u32 == 0 {
                break 'feature_done;
            }
            if ds & CRAM_FN as u32 != 0 && (*cr).len >= seq_pos {
                if !(*s).ref_0.is_null() && (*cr).ref_id >= 0 {
                    if ref_pos + (*cr).len as int64_t - seq_pos as int64_t + 1
                        > (*(*bfd).ref_.offset((*cr).ref_id as isize)).len
                    {
                        let rlen_0 =
                            ((*(*bfd).ref_.offset((*cr).ref_id as isize)).len - ref_pos) as i32;
                        if rlen_0 > 0 {
                            if ref_pos + rlen_0 as int64_t > (*s).ref_end {
                                cigar_extends_error = true;
                            } else {
                                if seq_pos - 1 + rlen_0 < (*cr).len {
                                    memcpy(
                                        seq.offset((seq_pos - 1) as isize) as *mut (),
                                        (*s).ref_0.offset((ref_pos - (*s).ref_start + 1) as isize)
                                            as *const (),
                                        rlen_0 as size_t,
                                    );
                                }
                                if (*cr).len - seq_pos + 1 - rlen_0 > 0 {
                                    memset(
                                        seq.offset((seq_pos - 1 + rlen_0) as isize) as *mut (),
                                        'N' as i32,
                                        ((*cr).len - seq_pos + 1 - rlen_0) as size_t,
                                    );
                                }
                            }
                        } else if (*cr).len - seq_pos + 1 > 0 {
                            memset(
                                seq.offset((seq_pos - 1) as isize) as *mut (),
                                'N' as i32,
                                ((*cr).len - seq_pos + 1) as size_t,
                            );
                        }
                        if !cigar_extends_error && md_dist >= 0 {
                            md_dist += (*cr).len - seq_pos + 1;
                        }
                    } else {
                        if (*cr).len - seq_pos + 1 > 0 {
                            if ref_pos + (*cr).len as int64_t - seq_pos as int64_t + 1
                                > (*s).ref_end
                            {
                                cigar_extends_error = true;
                            } else {
                                let remainder = (*cr).len - (seq_pos - 1);
                                let j = (ref_pos - (*s).ref_start + 1) as i32;
                                if decode_md != 0 || decode_nm != 0 {
                                    let n_0 = memchr(
                                        (*s).ref_0.offset(j as isize) as *const (),
                                        'N' as i32,
                                        remainder as size_t,
                                    ) as *mut u8;
                                    if n_0.is_null() {
                                        md_dist += (*cr).len - (seq_pos - 1);
                                    } else {
                                        let refp_0 =
                                            (*s).ref_0.offset((j - (seq_pos - 1)) as isize);
                                        md_dist = (md_dist as int64_t
                                            + n_0.offset_from((*s).ref_0.offset(j as isize))
                                                as int64_t)
                                            as int32_t;
                                        let i_start = ((seq_pos - 1) as int64_t
                                            + n_0.offset_from((*s).ref_0.offset(j as isize))
                                                as int64_t)
                                            as i32;
                                        let mut i_0 = i_start;
                                        while i_0 < (*cr).len {
                                            let base_1 = *refp_0.offset(i_0 as isize);
                                            if base_1 as i32 == 'N' as i32 {
                                                if add_md_char(
                                                    s,
                                                    decode_md,
                                                    'N' as i32 as u8,
                                                    &raw mut md_dist,
                                                ) < 0
                                                {
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
                                    seq.offset((seq_pos - 1) as isize) as *mut (),
                                    (*s).ref_0.offset(j as isize) as *const (),
                                    remainder as size_t,
                                );
                            }
                        }
                        if !cigar_extends_error {
                            ref_pos += ((*cr).len - seq_pos + 1) as int64_t;
                        }
                    }
                } else if (*cr).ref_id >= 0 {
                    ref_pos += ((*cr).len - seq_pos + 1) as int64_t;
                }
                if cigar_extends_error {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_seq\0" as *const u8 as *const u8,
                        b"CRAM CIGAR extends beyond slice reference extents\0" as *const u8
                            as *const u8
                    );
                    return -1;
                }
                if ncigar.wrapping_add(1) >= cigar_alloc {
                    cigar_alloc = if cigar_alloc != 0 {
                        cigar_alloc.wrapping_mul(2)
                    } else {
                        1024
                    };
                    cigar = libc::realloc((*s).cigar as *mut libc::c_void, cigar_alloc as usize * std::mem::size_of::<uint32_t>()).cast::<uint32_t>();
                    if cigar.is_null() {
                        return -1;
                    }
                    (*s).cigar = cigar;
                }
                if cig_len != 0 && cig_op != BAM_CMATCH {
                    let t = ncigar;
                    ncigar = ncigar.wrapping_add(1);
                    *cigar.offset(t as isize) = ((cig_len << 4) as u32).wrapping_add(cig_op);
                    cig_len = 0;
                }
                cig_op = BAM_CMATCH_;
                cig_len += (*cr).len - seq_pos + 1;
            }
        } // 'feature_done
          // -- after feature processing --
        if ds & CRAM_FN as u32 != 0
            && decode_md != 0
            && md_dist >= 0
            && block_append_uint((*s).aux_blk, md_dist as u32) < 0
        {
            return -1;
        }
        if cig_len != 0 {
            if ncigar >= cigar_alloc {
                cigar_alloc = if cigar_alloc != 0 {
                    cigar_alloc.wrapping_mul(2)
                } else {
                    1024
                };
                cigar = libc::realloc((*s).cigar as *mut libc::c_void, cigar_alloc as usize * std::mem::size_of::<uint32_t>()).cast::<uint32_t>();
                if cigar.is_null() {
                    return -1;
                }
                (*s).cigar = cigar;
            }
            let t = ncigar;
            ncigar = ncigar.wrapping_add(1);
            *cigar.offset(t as isize) = ((cig_len << 4) as u32).wrapping_add(cig_op);
        }
        (*cr).ncigar = ncigar.wrapping_sub((*cr).cigar) as int32_t;
        (*cr).aend = if ref_pos > (*cr).apos {
            ref_pos
        } else {
            (*cr).apos
        };
        if ds & CRAM_MQ as u32 != 0 {
            if codec!(DS_MQ).is_null() {
                return -1;
            }
            r |= decode!(DS_MQ, &raw mut (*cr).mqual as *mut u8, &raw mut out_sz);
        } else {
            (*cr).mqual = 40;
        }
        if ds & CRAM_QS as u32 != 0 && cf & CRAM_FLAG_PRESERVE_QUAL_SCORES != 0 {
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
                let mut tmp_MD_: [u8; 1024] = [0; 1024];
                // Heap-backed scratch for the >1024 case; the stack array is used
                // otherwise. `tmp_MD` points at whichever backing store is live.
                let mut tmp_MD_heap: Vec<u8> = Vec::new();
                let mut tmp_MD: *mut u8 = &raw mut tmp_MD_ as *mut u8;
                let orig_aux_p: *mut u8 = (*(*s).aux_blk).data.offset(orig_aux as isize);
                if sz > 1024 {
                    tmp_MD_heap = vec![0u8; sz];
                    tmp_MD = tmp_MD_heap.as_mut_ptr();
                }
                memcpy(tmp_MD as *mut (), orig_aux_p as *const (), sz);
                memmove(
                    ((*(*s).aux_blk).data.offset(-has_MD as isize)).add(sz) as *mut (),
                    (*(*s).aux_blk).data.offset(-has_MD as isize) as *const (),
                    orig_aux_p.offset_from((*(*s).aux_blk).data.offset(-has_MD as isize)) as size_t,
                );
                memcpy(
                    (*(*s).aux_blk).data.offset(-has_MD as isize) as *mut (),
                    tmp_MD as *const (),
                    sz,
                );
                drop(tmp_MD_heap);
                if -has_NM > -has_MD {
                    has_NM = (has_NM as size_t).wrapping_sub(sz) as i32;
                }
            }
            (*cr).aux_size = (*cr).aux_size.wrapping_add(sz as uint32_t);
        }
        if decode_nm != 0 {
            if has_NM == 0 {
                let mut buf: [u8; 7] = [0; 7];
                let buf_size: size_t;
                buf[0] = 'N' as i32 as u8;
                buf[1] = 'M' as i32 as u8;
                if nm <= UINT8_MAX as uint32_t {
                    buf_size = 4;
                    buf[2] = 'C' as i32 as u8;
                    buf[3] = (nm & 0xff) as u8;
                } else if nm <= UINT16_MAX as uint32_t {
                    buf_size = 5;
                    buf[2] = 'S' as i32 as u8;
                    buf[3] = (nm & 0xff) as u8;
                    buf[4] = (nm >> 8 & 0xff) as u8;
                } else {
                    buf_size = 7;
                    buf[2] = 'I' as i32 as u8;
                    buf[3] = (nm & 0xff) as u8;
                    buf[4] = (nm >> 8 & 0xff) as u8;
                    buf[5] = (nm >> 16 & 0xff) as u8;
                    buf[6] = (nm >> 24 & 0xff) as u8;
                }
                if block_append(
                    (*s).aux_blk,
                    &raw mut buf as *mut u8 as *const (),
                    buf_size,
                ) < 0
                {
                    return -1;
                }
                (*cr).aux_size = (*cr).aux_size.wrapping_add(buf_size as uint32_t);
            } else {
                let buf_0: *mut u8 = (*(*s).aux_blk).data.offset(-has_NM as isize);
                *buf_0.offset(0) = (nm & 0xff) as u8;
                *buf_0.offset(1) = (nm >> 8 & 0xff) as u8;
                *buf_0.offset(2) = (nm >> 16 & 0xff) as u8;
                *buf_0.offset(3) = (nm >> 24 & 0xff) as u8;
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
    ) -> i32 {
        let mut last_ref_id: i32;
        let blk: *mut cram_block = *(*s).block.offset(0);
        let mut bf: int32_t = 0;
        let ref_id: int32_t = (*(*s).hdr).ref_seq_id;
        let mut cf: u8 = 0;
        let mut out_sz: i32;
        let mut r: i32 = 0;
        let mut rec: i32;
        let mut seq: *mut u8 = std::ptr::null_mut();
        let mut qual: *mut u8 = std::ptr::null_mut();
        let mut unknown_rg: i32 = -1;
        let embed_ref: i32 = if (*fd).version >> 8 < 4 {
            ((*(*s).hdr).ref_base_id >= 0) as i32
        } else {
            ((*(*s).hdr).ref_base_id > 0) as i32
        };
        // Per-reference scratch table, owned here for the whole function.
        let mut refs_vec: Vec<*mut u8> = Vec::new();
        let mut refs: *mut *mut u8 = std::ptr::null_mut();
        let bfd: *mut sam_hrecs_t = (*sh).hrecs;
        let comp = (*c).comp_hdr;
        macro_rules! codec {
            ($id:expr) => {
                (*comp).codecs[$id as usize].cast::<cram_codec>()
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
        let ds: uint32_t = (*s).data_series;
        (*blk).bit = 7;
        let mut qsize: i32 = 0;
        let mut nsize: i32 = 0;
        let mut q_id: i32 = 0;
        cram_decode_estimate_sizes(comp, s, &raw mut qsize, &raw mut nsize, &raw mut q_id);
        if qsize != 0
            && ds & CRAM_RL as u32 != 0
            && block_resize_exact((*s).seqs_blk, (qsize + 1) as size_t) < 0
        {
            return -1;
        }
        if qsize != 0
            && ds & CRAM_RL as u32 != 0
            && block_resize_exact((*s).qual_blk, (qsize + 1) as size_t) < 0
        {
            return -1;
        }
        if nsize != 0
            && ds & CRAM_NS as u32 != 0
            && block_resize_exact((*s).name_blk, (nsize + 1) as size_t) < 0
        {
            return -1;
        }
        if (*bfd).nrg > 0
            && !(*(*bfd)
                .rg
                .cast::<sam_hrec_rg_t>()
                .offset(((*bfd).nrg - 1) as isize))
            .name
            .is_null()
            && strcmp(
                (*(*bfd)
                    .rg
                    .cast::<sam_hrec_rg_t>()
                    .offset(((*bfd).nrg - 1) as isize))
                .name,
                b"UNKNOWN\0" as *const u8 as *const u8,
            ) == 0
        {
            unknown_rg = (*bfd).nrg - 1;
        }
        if (*blk).content_type != CORE {
            return -1;
        }
        if !(*s).crecs.is_null() {
            libc::free((*s).crecs.cast());
        }
        (*s).crecs =
            libc::malloc((*(*s).hdr).num_records as usize * std::mem::size_of::<cram_record>())
                .cast::<cram_record>();
        if (*s).crecs.is_null() {
            return -1;
        }
        if ref_id >= 0 {
            if embed_ref != 0 {
                if (*(*s).hdr).ref_base_id < 0 {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"No reference specified and no embedded reference is available\0"
                            as *const u8 as *const u8
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
                (*s).ref_0 = (*b).data as *mut u8;
                (*s).ref_start = (*(*s).hdr).ref_seq_start;
                (*s).ref_end = (*(*s).hdr).ref_seq_start + (*(*s).hdr).ref_seq_span - 1;
                if (*(*s).hdr).ref_seq_span > (*b).uncomp_size as int64_t {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"Embedded reference is too small\0" as *const u8 as *const u8
                    );
                    return -1;
                }
            } else if (*comp).no_ref == 0 {
                if (*fd).required_fields & SAM_SEQ as u32 != 0 {
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
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"Slice starts before base 1\0" as *const u8 as *const u8
                    );
                    (*s).ref_start = 0;
                }
                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
                if (*fd).required_fields & SAM_SEQ as u32 != 0
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
        if (*fd).required_fields & SAM_SEQ as u32 != 0
            && (*s).ref_0.is_null()
            && (*(*s).hdr).ref_seq_id >= 0
            && (*comp).no_ref == 0
        {
            hts_log!(
                HTS_LOG_ERROR,
                b"cram_decode_slice\0" as *const u8 as *const u8,
                b"Unable to fetch reference\0" as *const u8 as *const u8
            );
            return -1;
        }
        if (*fd).version >> 8 != 1
            && (*fd).required_fields & SAM_SEQ as u32 != 0
            && (*(*s).hdr).ref_seq_id >= 0
            && (*fd).ignore_md5 == 0
            && memcmp(
                &raw mut (*(*s).hdr).md5 as *mut u8 as *const (),
                b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0" as *const u8 as *const u8
                    as *const (),
                16,
            ) != 0
        {
            let mut digest: [u8; 16] = [0; 16];
            let mut md5: Box<hts_md5_context>;
            if !(*s).ref_0.is_null() && (*(*s).hdr).ref_seq_id >= 0 {
                let mut len: i32;
                let start: i32 = if (*(*s).hdr).ref_seq_start >= (*s).ref_start {
                    ((*(*s).hdr).ref_seq_start - (*s).ref_start) as i32
                } else {
                    hts_log!(
                        HTS_LOG_WARNING,
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"Slice starts before base 1\0" as *const u8 as *const u8
                    );
                    0
                };
                if (*(*s).hdr).ref_seq_span <= (*s).ref_end - (*s).ref_start + 1 {
                    len = (*(*s).hdr).ref_seq_span as i32;
                } else {
                    hts_log!(
                        HTS_LOG_WARNING,
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"Slice ends beyond reference end\0" as *const u8 as *const u8
                    );
                    len = ((*s).ref_end - (*s).ref_start + 1) as i32;
                }
                md5 = hts_md5_init();
                if (start + len) as hts_pos_t > (*s).ref_end - (*s).ref_start + 1 {
                    len = ((*s).ref_end - (*s).ref_start + 1 - start as hts_pos_t) as i32;
                }
                if len >= 0 {
                    hts_md5_update(
                        &mut *md5,
                        std::slice::from_raw_parts(
                            (*s).ref_0.offset(start as isize) as *const u8,
                            len as usize,
                        ),
                    );
                }
                hts_md5_final(&mut digest, &mut *md5);
                hts_md5_destroy(Some(md5));
            } else if (*s).ref_0.is_null() && (*(*s).hdr).ref_base_id >= 0 {
                let b_0 = cram_get_block_by_id(s, (*(*s).hdr).ref_base_id);
                if !b_0.is_null() {
                    md5 = hts_md5_init();
                    hts_md5_update(
                        &mut *md5,
                        std::slice::from_raw_parts(
                            (*b_0).data as *const u8,
                            (*b_0).uncomp_size as usize,
                        ),
                    );
                    hts_md5_final(&mut digest, &mut *md5);
                    hts_md5_destroy(Some(md5));
                }
            }
            if (*comp).no_ref == 0
                && ((*s).ref_0.is_null() && (*(*s).hdr).ref_base_id < 0
                    || memcmp(
                        &raw mut digest as *mut u8 as *const (),
                        &raw mut (*(*s).hdr).md5 as *mut u8 as *const (),
                        16,
                    ) != 0)
            {
                let mut m_arr: [u8; 33] = [0; 33];
                let mut rname = sam_hdr_tid2name(&*sh, ref_id);
                if rname.is_null() {
                    rname = b"?\0" as *const u8 as *const u8;
                }
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_decode_slice\0" as *const u8 as *const u8,
                    b"MD5 checksum reference mismatch\0" as *const u8 as *const u8
                );
                let _ = md5_print(
                    &raw mut (*(*s).hdr).md5 as *mut u8,
                    &raw mut m_arr as *mut u8,
                );
                let _ = md5_print(
                    &raw mut digest as *mut u8,
                    &raw mut m_arr as *mut u8,
                );
                let mut ks: kstring_t = kstring_t::default();
                let _ = sam_hdr_find_tag_id(
                    &mut *sh,
                    b"SQ",
                    Some((
                        b"SN",
                        std::ffi::CStr::from_ptr(rname.cast::<i8>()).to_bytes(),
                    )),
                    b"M5",
                    &mut ks,
                );
                ks_free(&mut ks);
                return -1;
            }
        }
        if ref_id == -2 {
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            pthread_mutex_lock(&raw mut (*(*fd).refs).lock);
            refs_vec = vec![std::ptr::null_mut(); (*(*fd).refs).nref as usize];
            refs = refs_vec.as_mut_ptr();
            pthread_mutex_unlock(&raw mut (*(*fd).refs).lock);
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
        }
        last_ref_id = -9;
        rec = 0;
        let mut decode_error = false;
        'rec_loop: while rec < (*(*s).hdr).num_records {
            let cr: *mut cram_record = (*s).crecs.offset(rec as isize);
            let mut has_MD: i32;
            let mut has_NM: i32;
            (*cr).s = s;
            out_sz = 1;
            if ds & CRAM_BF as u32 != 0 {
                if codec!(DS_BF).is_null() {
                    decode_error = true;
                    break;
                }
                r |= decode!(DS_BF, &raw mut bf as *mut u8, &raw mut out_sz);
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
            if ds & CRAM_CF as u32 != 0 {
                if (*fd).version >> 8 == 1 {
                    if codec!(DS_CF).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_CF, &raw mut cf as *mut u8, &raw mut out_sz);
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
                    r |= decode!(
                        DS_CF,
                        &raw mut (*cr).cram_flags as *mut u8,
                        &raw mut out_sz
                    );
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    cf = (*cr).cram_flags as u8;
                }
            } else {
                (*cr).cram_flags = 0;
                cf = 0;
            }
            if (*fd).version >> 8 != 1 && ref_id == -2 {
                if ds & CRAM_RI as u32 != 0 {
                    if codec!(DS_RI).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_RI, &raw mut (*cr).ref_id as *mut u8, &raw mut out_sz);
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    if (*cr).ref_id < -1 || (*cr).ref_id >= (*bfd).nref {
                        hts_log!(
                            HTS_LOG_ERROR,
                            b"cram_decode_slice\0" as *const u8 as *const u8,
                            b"Requested unknown reference ID\0" as *const u8 as *const u8
                        );
                        decode_error = true;
                        break;
                    } else if (*fd).required_fields & (SAM_SEQ | SAM_TLEN) as u32 != 0
                        && (*cr).ref_id >= 0
                        && (*cr).ref_id != last_ref_id
                    {
                        if (*comp).no_ref == 0 {
                            pthread_mutex_lock(&raw mut (*fd).range_lock);
                            let need_ref = ((*fd).range.refid == -2
                                || (*cr).ref_id == (*fd).range.refid)
                                as i32;
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
                                as i32;
                            pthread_mutex_unlock(&raw mut (*fd).range_lock);
                            if discard_last_ref != 0 {
                                pthread_mutex_lock(&raw mut (*fd).ref_lock);
                                discard_last_ref = ((*fd).unsorted == 0) as i32;
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
                    b"cram_decode_slice\0" as *const u8 as *const u8,
                    b"Requested unknown reference ID\0" as *const u8 as *const u8
                );
                decode_error = true;
                break;
            }
            if ds & CRAM_RL as u32 != 0 {
                if codec!(DS_RL).is_null() {
                    decode_error = true;
                    break;
                }
                r |= decode!(DS_RL, &raw mut (*cr).len as *mut u8, &raw mut out_sz);
                if r != 0 {
                    decode_error = true;
                    break;
                }
                if (*cr).len < 0 {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"Read has negative length\0" as *const u8 as *const u8
                    );
                    decode_error = true;
                    break;
                }
            }
            if ds & CRAM_AP as u32 != 0 {
                if codec!(DS_AP).is_null() {
                    decode_error = true;
                    break;
                }
                if (*fd).version >> 8 >= 4 {
                    r |= decode!(DS_AP, &raw mut (*cr).apos as *mut u8, &raw mut out_sz);
                } else {
                    let mut i32: int32_t = 0;
                    r |= decode!(DS_AP, &raw mut i32 as *mut u8, &raw mut out_sz);
                    (*cr).apos = i32 as int64_t;
                }
                if r != 0 {
                    decode_error = true;
                    break;
                }
                if (*comp).ap_delta != 0 {
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
            if ds & CRAM_RG as u32 != 0 {
                if codec!(DS_RG).is_null() {
                    decode_error = true;
                    break;
                }
                r |= decode!(DS_RG, &raw mut (*cr).rg as *mut u8, &raw mut out_sz);
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
                if ds & CRAM_RN as u32 != 0 {
                    if codec!(DS_RN).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(DS_RN, (*s).name_blk as *mut u8, &raw mut out_sz2);
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
            if ds & CRAM_CF as u32 != 0 && cf as i32 & CRAM_FLAG_DETACHED != 0 {
                if ds & CRAM_MF as u32 != 0 {
                    if (*fd).version >> 8 == 1 {
                        let mut mf: u8 = 0;
                        if codec!(DS_MF).is_null() {
                            decode_error = true;
                            break;
                        }
                        r |= decode!(DS_MF, &raw mut mf as *mut u8, &raw mut out_sz);
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
                        r |= decode!(
                            DS_MF,
                            &raw mut (*cr).mate_flags as *mut u8,
                            &raw mut out_sz
                        );
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
                    if ds & CRAM_RN as u32 != 0 {
                        if codec!(DS_RN).is_null() {
                            decode_error = true;
                            break;
                        }
                        r |= decode!(DS_RN, (*s).name_blk as *mut u8, &raw mut out_sz2_0);
                        if r != 0 {
                            decode_error = true;
                            break;
                        }
                        (*cr).name_len = out_sz2_0;
                    }
                }
                if ds & CRAM_NS as u32 != 0 {
                    if codec!(DS_NS).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(
                        DS_NS,
                        &raw mut (*cr).mate_ref_id as *mut u8,
                        &raw mut out_sz
                    );
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                    if (*cr).mate_ref_id < -1 || (*cr).mate_ref_id >= (*bfd).nref {
                        hts_log!(
                            HTS_LOG_ERROR,
                            b"cram_decode_slice\0" as *const u8 as *const u8,
                            b"Requested unknown mate reference ID\0" as *const u8 as *const u8
                        );
                        decode_error = true;
                        break;
                    }
                }
                if ds & CRAM_NP as u32 != 0 {
                    if codec!(DS_NP).is_null() {
                        decode_error = true;
                        break;
                    }
                    if (*fd).version >> 8 < 4 {
                        let mut i32_0: int32_t = 0;
                        r |= decode!(DS_NP, &raw mut i32_0 as *mut u8, &raw mut out_sz);
                        (*cr).mate_pos = i32_0 as int64_t;
                    } else {
                        r |= decode!(
                            DS_NP,
                            &raw mut (*cr).mate_pos as *mut u8,
                            &raw mut out_sz
                        );
                    }
                    if r != 0 {
                        decode_error = true;
                        break;
                    }
                }
                if ds & CRAM_TS as u32 != 0 {
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
            } else if ds & CRAM_CF as u32 != 0 && cf as i32 & CRAM_FLAG_MATE_DOWNSTREAM != 0 {
                if ds & CRAM_NF as u32 != 0 {
                    if codec!(DS_NF).is_null() {
                        decode_error = true;
                        break;
                    }
                    r |= decode!(
                        DS_NF,
                        &raw mut (*cr).mate_line as *mut u8,
                        &raw mut out_sz
                    );
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
                if ds & CRAM_CF as u32 != 0 && cf as i32 & CRAM_FLAG_EXPLICIT_TLEN != 0 {
                    if ds & CRAM_TS as u32 != 0 {
                        r = cram_decode_tlen(fd, c, s, blk, &raw mut (*cr).explicit_tlen);
                        if r != 0 {
                            return r;
                        }
                    } else {
                        (*cr).mate_flags = 0;
                        (*cr).tlen = INT64_MIN;
                    }
                }
            } else if ds & CRAM_CF as u32 != 0 && cf as i32 & CRAM_FLAG_EXPLICIT_TLEN != 0 {
                if ds & CRAM_TS as u32 != 0 {
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
            if ds & CRAM_RL as u32 != 0 {
                (*cr).seq = (*(*s).seqs_blk).byte as uint32_t;
                if block_resize(
                    (*s).seqs_blk,
                    (*cr).seq.wrapping_add((*cr).len as uint32_t) as size_t,
                ) < 0
                {
                    decode_error = true;
                    break;
                }
                seq = (*(*s).seqs_blk).data.add((*(*s).seqs_blk).byte) as *mut u8;
                (*(*s).seqs_blk).byte = (*(*s).seqs_blk).byte.wrapping_add((*cr).len as size_t);
                if seq.is_null() {
                    decode_error = true;
                    break;
                }
                (*cr).qual = (*(*s).qual_blk).byte as uint32_t;
                if block_resize(
                    (*s).qual_blk,
                    (*cr).qual.wrapping_add((*cr).len as uint32_t) as size_t,
                ) < 0
                {
                    decode_error = true;
                    break;
                }
                qual = (*(*s).qual_blk).data.add((*(*s).qual_blk).byte) as *mut u8;
                (*(*s).qual_blk).byte = (*(*s).qual_blk).byte.wrapping_add((*cr).len as size_t);
                if (*s).ref_0.is_null() {
                    memset(seq as *mut (), '=' as i32, (*cr).len as size_t);
                }
            }
            if bf & BAM_FUNMAP == 0 {
                if ds & CRAM_AP as u32 != 0 && (*cr).apos <= 0 {
                    hts_log!(
                        HTS_LOG_ERROR,
                        b"cram_decode_slice\0" as *const u8 as *const u8,
                        b"Read has alignment position but no unmapped flag\0" as *const u8
                            as *const u8
                    );
                    decode_error = true;
                    break;
                } else if ds
                    & (CRAM_FN
                        | CRAM_FP
                        | CRAM_FC
                        | CRAM_DL
                        | CRAM_IN
                        | CRAM_SC
                        | CRAM_HC
                        | CRAM_PD
                        | CRAM_RS
                        | CRAM_RL
                        | CRAM_BF
                        | CRAM_BA
                        | CRAM_BS
                        | CRAM_RL
                        | CRAM_AP
                        | CRAM_BB
                        | CRAM_MQ) as u32
                    != 0
                {
                    r |= cram_decode_seq(
                        fd,
                        c,
                        s,
                        blk,
                        cr,
                        sh,
                        cf as i32,
                        seq,
                        qual,
                        has_MD,
                        has_NM,
                    );
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
                let mut out_sz2_1: i32 = (*cr).len;
                (*cr).cigar = 0;
                (*cr).ncigar = 0;
                (*cr).aend = (*cr).apos;
                (*cr).mqual = 0;
                if ds & CRAM_BA as u32 != 0 && (*cr).len != 0 {
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
                if ds & CRAM_CF as u32 != 0 && cf as i32 & CRAM_FLAG_PRESERVE_QUAL_SCORES != 0
                {
                    out_sz2_1 = (*cr).len;
                    if ds & CRAM_QS as u32 != 0 && (*cr).len >= 0 {
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
                } else if ds & CRAM_RL as u32 != 0 {
                    memset(qual as *mut (), 255, (*cr).len as size_t);
                }
            }
            if (*comp).qs_seq_orient == 0
                && ds & CRAM_QS as u32 != 0
                && (*cr).flags & BAM_FREVERSE != 0
            {
                let mut i = 0;
                let mut j = (*cr).len - 1;
                while i < j {
                    let c_0 = *qual.offset(i as isize) as u8;
                    *qual.offset(i as isize) = *qual.offset(j as isize);
                    *qual.offset(j as isize) = c_0 as u8;
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
                refs_vec = Vec::new();
                refs = std::ptr::null_mut();
            } else if ref_id >= 0 && (*s).ref_0 != (*fd).ref_free && embed_ref == 0 {
                cram_ref_decr((*fd).refs, ref_id);
            }
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
            r |= cram_decode_slice_xref(s, (*fd).required_fields as i32);
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
            refs_vec = Vec::new();
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
        }
        let _ = refs_vec;
        -1
    }

    // original: cram_decode_slice_thread (htslib/cram/cram_decode.c:3036)
    unsafe extern "C" fn cram_decode_slice_thread(
        arg: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void {
        let j = arg as *mut cram_decode_job;
        let (Some(fd), Some(c), Some(s), Some(h)) = ((*j).fd, (*j).c, (*j).s, (*j).h) else {
            (*j).exit_code = -1;
            return j.cast::<std::ffi::c_void>();
        };
        (*j).exit_code = cram_decode_slice(fd.as_ptr(), c.as_ptr(), s.as_ptr(), h.as_ptr());
        j.cast::<std::ffi::c_void>()
    }

    // original: cram_decode_slice_mt (htslib/cram/cram_decode.c:3047)
    unsafe fn cram_decode_slice_mt(
        fd: *mut cram_fd,
        c: *mut cram_container,
        s: *mut cram_slice,
        bfd: *mut sam_hdr_t,
    ) -> i32 {
        if (*fd).pool.is_null() {
            return cram_decode_slice(fd, c, s, bfd);
        }
        let j = libc::malloc(std::mem::size_of::<cram_decode_job>()).cast::<cram_decode_job>();
        if j.is_null() {
            return -1;
        }
        std::ptr::write(
            j,
            cram_decode_job {
                fd: NonNull::new(fd),
                c: NonNull::new(c),
                s: NonNull::new(s),
                h: NonNull::new(bfd),
                exit_code: 0,
            },
        );
        let nonblock = if hts_tpool_process_sz(&mut *(*fd).rqueue.cast()) != 0 {
            1
        } else {
            0
        };
        let saved_errno = *__errno_location();
        *__errno_location() = 0;
        if -1
            == hts_tpool_dispatch2(
                (*fd).pool.cast(),
                (*fd).rqueue.cast(),
                Some(cram_decode_slice_thread),
                j.cast::<std::ffi::c_void>(),
                nonblock,
            )
        {
            if *__errno_location() != EAGAIN {
                return -1;
            }
            (*fd).job_pending = j as *mut ();
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
        rec: i32,
        bam_0: *mut *mut bam_seq_t,
    ) -> i32 {
        let mut name_a: [u8; 1024] = [0; 1024];
        let mut name: *mut u8;
        let mut name_len: i32;
        let mut aux: *mut u8;
        let bfd: *mut sam_hrecs_t = (*sh).hrecs;
        if (*fd).required_fields & SAM_QNAME as u32 != 0 {
            if (*cr).name_len != 0 {
                name = ((*(*s).name_blk).data as *mut u8).offset((*cr).name as isize);
                name_len = (*cr).name_len;
            } else {
                name = &raw mut name_a as *mut u8;
                if (*cr).mate_line >= 0
                    && (*cr).mate_line < (*s).max_rec
                    && (*(*s).crecs.offset((*cr).mate_line as isize)).name_len > 0
                {
                    memcpy(
                        &raw mut name_a as *mut u8 as *mut (),
                        (*(*s).name_blk)
                            .data
                            .offset((*(*s).crecs.offset((*cr).mate_line as isize)).name as isize)
                            as *const (),
                        (*(*s).crecs.offset((*cr).mate_line as isize)).name_len as size_t,
                    );
                    name = (&raw mut name_a as *mut u8)
                        .offset((*(*s).crecs.offset((*cr).mate_line as isize)).name_len as isize);
                } else {
                    name_len = strlen((*fd).prefix) as i32;
                    memcpy(
                        name as *mut (),
                        (*fd).prefix as *const (),
                        name_len as size_t,
                    );
                    name = name.offset(name_len as isize);
                    let t = name;
                    name = name.offset(1);
                    *t = ':' as i32 as u8;
                    if (*cr).mate_line >= 0 && (*cr).mate_line < rec {
                        name = append_uint64(
                            name as *mut u8,
                            ((*(*s).hdr).record_counter + (*cr).mate_line as int64_t + 1)
                                as uint64_t,
                        ) as *mut u8;
                    } else {
                        name = append_uint64(
                            name as *mut u8,
                            ((*(*s).hdr).record_counter + rec as int64_t + 1) as uint64_t,
                        ) as *mut u8;
                    }
                }
                name_len = name.offset_from(&raw mut name_a as *mut u8) as i32;
                name = &raw mut name_a as *mut u8;
            }
        } else {
            name = b"?\0" as *const u8 as *const u8 as *mut u8;
            name_len = 1;
        }
        if (*cr).rg < -1 || (*cr).rg >= (*bfd).nrg {
            return -1;
        }
        let rg_len: i32 = if (*cr).rg != -1 {
            (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset((*cr).rg as isize)).name_len + 4
        } else {
            0
        };
        let seq: *mut u8 = if (*fd).required_fields & (SAM_SEQ | SAM_QUAL) as u32 != 0 {
            if (*(*s).seqs_blk).data.is_null() {
                return -1;
            }
            ((*(*s).seqs_blk).data as *mut u8).offset((*cr).seq as isize)
        } else {
            (*cr).len = 0;
            b"*\0" as *const u8 as *const u8 as *mut u8
        };
        let qual: *mut u8 = if (*fd).required_fields & SAM_QUAL as u32 != 0 {
            if (*(*s).qual_blk).data.is_null() {
                return -1;
            }
            ((*(*s).qual_blk).data as *mut u8).offset((*cr).qual as isize)
        } else {
            std::ptr::null_mut()
        };
        let ret: i32 = bam_set1(
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
        // SEAM: bam1_t.data is now an owned Vec<u8>. C builds the record core via
        // bam_set1 (which left l_data == data_len, the qname/cigar/seq portion)
        // then appends the aux region in-place with `b->l_data += aux_size`/
        // `+= rg_len`. The Rust bam_set1 truncates data.len() to data_len, so the
        // aux bytes must be written WITHIN the logical Vec, not into reserved
        // capacity past len() (which sam_format1 would never see). Grow the Vec to
        // the final length up-front (capacity is already reserved by bam_set1's
        // realloc_bam_data(data_len + l_aux)); the writes below fill that region.
        let data_len = (*b).data.len();
        let final_len = data_len + (*cr).aux_size as usize + rg_len as usize;
        (*b).data.resize(final_len, 0);
        aux = (*b)
            .data
            .as_mut_ptr()
            .offset(((*b).core.n_cigar << 2) as isize)
            .offset((*b).core.l_qname as i32 as isize)
            .offset((((*b).core.l_qseq + 1) >> 1) as isize)
            .offset((*b).core.l_qseq as isize) as *mut u8;
        if (*cr).aux_size != 0 {
            memcpy(
                aux as *mut (),
                (*(*s).aux_blk).data.offset((*cr).aux as isize) as *const (),
                (*cr).aux_size as size_t,
            );
            aux = aux.offset((*cr).aux_size as isize);
            // C: l_data += aux_size. data.len() already equals the final
            // data_len + aux_size + rg_len (bam_set1 sized it), so this is a no-op.
        }
        if rg_len > 0 {
            let t1 = aux;
            aux = aux.offset(1);
            *t1 = 'R' as i32 as u8;
            let t2 = aux;
            aux = aux.offset(1);
            *t2 = 'G' as i32 as u8;
            let t3 = aux;
            aux = aux.offset(1);
            *t3 = 'Z' as i32 as u8;
            let len = (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset((*cr).rg as isize)).name_len;
            memcpy(
                aux as *mut (),
                (*(*bfd).rg.cast::<sam_hrec_rg_t>().offset((*cr).rg as isize)).name
                    as *const (),
                len as size_t,
            );
            aux = aux.offset(len as isize);
            let t4 = aux;
            *t4 = 0;
            // C: l_data += rg_len. data.len() already includes rg_len; no-op.
        }
        (*b).data.len() as i32
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
        (*c).comp_hdr_block = cram_read_block(fd).cast();
        if (*c).comp_hdr_block.is_null() {
            return std::ptr::null_mut();
        }
        if (*(*c).comp_hdr_block.cast::<cram_block>()).content_type != COMPRESSION_HEADER {
            return std::ptr::null_mut();
        }
        (*c).comp_hdr = cram_decode_compression_header(fd, (*c).comp_hdr_block.cast());
        if (*c).comp_hdr.is_null() {
            return std::ptr::null_mut();
        }
        if (*(*c).comp_hdr).ap_delta == 0
            && sam_hrecs_sort_order(&mut *(*(*fd).header).hrecs) != ORDER_COORD
        {
            pthread_mutex_lock(&raw mut (*fd).ref_lock);
            (*fd).unsorted = 1;
            pthread_mutex_unlock(&raw mut (*fd).ref_lock);
        }
        c
    }

    // original: cram_next_slice (htslib/cram/cram_decode.c:3268)
    pub(crate) unsafe fn cram_next_slice(
        fd: *mut cram_fd,
        cp: *mut *mut cram_container,
    ) -> *mut cram_slice {
        let mut c_curr: *mut cram_container = (*fd).ctr;
        let mut s_curr: *mut cram_slice;
        if c_curr.is_null() {
            c_curr = cram_first_slice(fd);
            if c_curr.is_null() {
                return std::ptr::null_mut();
            }
        }
        s_curr = (*c_curr).slice.cast();
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
                if let (Some(c), Some(s)) = ((*j).c, (*j).s) {
                    c_next = c.as_ptr();
                    s_next = s.as_ptr();
                }
                libc::free((*fd).job_pending.cast());
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
                            if (*c_next).ref_seq_id != (*fd).range.refid
                                || ((*fd).range.refid != -1
                                    && (*c_next).ref_seq_start > (*fd).range.end)
                            {
                                cram_free_container(c_next);
                                (*fd).ctr_mt = std::ptr::null_mut();
                                (*fd).ooc = 1;
                                break 'outer;
                            } else if (*fd).range.refid != -1
                                && ((*c_next).ref_seq_start + (*c_next).ref_seq_span - 1)
                                    < (*fd).range.start
                            {
                                let skip_length = (*c_next).length as off_t;
                                cram_free_container(c_next);
                                (*fd).ooc = 0;
                                if hseek((*fd).fp, skip_length, SEEK_CUR) < 0 {
                                    return std::ptr::null_mut();
                                }
                                continue 'outer;
                            }
                        }
                        (*fd).ctr_mt = c_next;
                        (*c_next).comp_hdr_block = cram_read_block(fd).cast();
                        if (*c_next).comp_hdr_block.is_null() {
                            return std::ptr::null_mut();
                        }
                        if (*(*c_next).comp_hdr_block.cast::<cram_block>()).content_type
                            != COMPRESSION_HEADER
                        {
                            return std::ptr::null_mut();
                        }
                        (*c_next).comp_hdr =
                            cram_decode_compression_header(fd, (*c_next).comp_hdr_block.cast());
                        if (*c_next).comp_hdr.is_null() {
                            return std::ptr::null_mut();
                        }
                        if (*(*c_next).comp_hdr).ap_delta == 0
                            && sam_hrecs_sort_order(&mut *(*(*fd).header).hrecs) != ORDER_COORD
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
                        (*c_next).slice = cram_read_slice(fd).cast();
                        s_next = (*c_next).slice.cast();
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
                            if (*(*s_next).hdr).ref_seq_id != (*fd).range.refid
                                || ((*fd).range.refid != -1
                                    && (*(*s_next).hdr).ref_seq_start > (*fd).range.end)
                            {
                                (*fd).ooc = 1;
                                cram_free_slice(s_next);
                                s_next = std::ptr::null_mut();
                                (*c_next).slice = s_next.cast();
                                break;
                            } else if (*fd).range.refid != -1
                                && ((*(*s_next).hdr).ref_seq_start + (*(*s_next).hdr).ref_seq_span
                                    - 1)
                                    < (*fd).range.start
                            {
                                cram_free_slice(s_next);
                                s_next = std::ptr::null_mut();
                                (*c_next).slice = s_next.cast();
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
                    b"cram_next_slice\0" as *const u8 as *const u8,
                    b"Failure to decode slice\0" as *const u8 as *const u8
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
                if hts_tpool_process_len(&mut *(*fd).rqueue.cast())
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
            if (*fd).ooc != 0 && hts_tpool_process_empty(&mut *(*fd).rqueue.cast()) != 0 {
                (*fd).eof = 1;
                return std::ptr::null_mut();
            }
            let res = hts_tpool_next_result_wait((*fd).rqueue.cast());
            if res.is_null() || hts_tpool_result_data(&mut *res).is_null() {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_next_slice\0" as *const u8 as *const u8,
                    b"Call to hts_tpool_next_result failed\0" as *const u8 as *const u8
                );
                return std::ptr::null_mut();
            }
            let j_0 = hts_tpool_result_data(&mut *res) as *mut cram_decode_job;
            let (Some(c), Some(s)) = ((*j_0).c, (*j_0).s) else {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_next_slice\0" as *const u8 as *const u8,
                    b"Slice decode failure\0" as *const u8 as *const u8
                );
                (*fd).eof = 0;
                hts_tpool_delete_result(res, 1);
                return std::ptr::null_mut();
            };
            c_curr = c.as_ptr();
            s_curr = s.as_ptr();
            if (*j_0).exit_code != 0 {
                hts_log!(
                    HTS_LOG_ERROR,
                    b"cram_next_slice\0" as *const u8 as *const u8,
                    b"Slice decode failure\0" as *const u8 as *const u8
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
            (*c_curr).slice = s_curr.cast();
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
                && (*(*c).slice.cast::<cram_slice>()).curr_rec
                    < (*(*c).slice.cast::<cram_slice>()).max_rec
            {
                s = (*c).slice.cast();
                if (*fd).range.refid == -2 {
                    break;
                }
                let curr_ref_id = (*(*s).crecs.offset((*s).curr_rec as isize)).ref_id;
                if ((*fd).range.refid == -1 || curr_ref_id < (*fd).range.refid) && curr_ref_id != -1
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
        (*c).slice = s.cast();
        let t = (*s).curr_rec;
        (*s).curr_rec += 1;
        (*s).crecs.offset(t as isize)
    }

    // original: cram_get_bam_seq (htslib/cram/cram_decode.c:3615)
    pub unsafe fn cram_get_bam_seq(fd: *mut cram_fd, bam_0: *mut *mut bam_seq_t) -> i32 {
        let cr = cram_get_seq(fd);
        if cr.is_null() {
            return -1;
        }
        let c = (*fd).ctr;
        let s = (*c).slice.cast();
        cram_to_bam((*fd).header, fd, s, cr, (*s).curr_rec - 1, bam_0)
    }
}

/// original: cram_decode_compression_header (htslib/cram/cram_decode.c:145)
/// Native port. Decodes a CRAM container's compression-header block into a
/// freshly calloc'd `cram_block_compression_hdr`. Faithfully reproduces the C
/// control flow: optional in-place block decompression; the CRAM v1-only
/// landmark/ref-seq prelude; the preservation map (RN/AP/RR/QO/SM/TD plus the
/// V1.0 MI/UI/PI single-byte keys) into the string-keyed `preservation_map`
/// khash with the public preservation flags mirrored; the data-series encoding
/// map (per 2-char key -> DS_ID + external type, building each codec via the
/// native `cram_decoder_init`, with the CRAM-v4 SLONG/LONG type selection for
/// AP/NP/TS); and the tag encoding map (3-char keys -> BYTE_ARRAY_BLOCK codecs).
/// Map-size mismatches and decoder-init failures free everything and return
/// NULL. Ownership matches C exactly (see `cram_free_compression_header`).
pub unsafe fn cram_cram_decode_c_145_cram_decode_compression_header(
    fd: *mut cram_fd,
    b: *mut cram_block,
) -> *mut cram_block_compression_hdr {
    // cram_DS_ID values (htslib/cram/cram_structs.h:143). Local copy because the
    // production module only exports a subset as module consts.
    const DS_CORE: i32 = 0;
    const DS_BF: i32 = 15;
    const DS_CF: i32 = 16;
    const DS_AP: i32 = 17;
    const DS_RG: i32 = 18;
    const DS_MQ: i32 = 19;
    const DS_NS: i32 = 20;
    const DS_MF: i32 = 21;
    const DS_TS: i32 = 22;
    const DS_NP: i32 = 23;
    const DS_NF: i32 = 24;
    const DS_RL: i32 = 25;
    const DS_FN: i32 = 26;
    const DS_FC: i32 = 27;
    const DS_FP: i32 = 28;
    const DS_DL: i32 = 29;
    const DS_BA: i32 = 30;
    const DS_BS: i32 = 31;
    const DS_TL: i32 = 32;
    const DS_RI: i32 = 33;
    const DS_RS: i32 = 34;
    const DS_PD: i32 = 35;
    const DS_HC: i32 = 36;
    const DS_BB: i32 = 37;
    const DS_QQ: i32 = 38;
    const DS_TC: i32 = 44;
    // cram_DS_ID values shared with the module-level consts: DS_RN=11, DS_QS=12,
    // DS_IN=13, DS_SC=14, DS_TN=39 (already defined).

    // cram_encoding / cram_external_type values.
    const E_NULL: i32 = 0;
    const E_INT: i32 = 1;
    const E_LONG: i32 = 2;
    const E_BYTE: i32 = 3;
    const E_BYTE_ARRAY: i32 = 4;
    const E_BYTE_ARRAY_BLOCK: i32 = 5;
    const E_SLONG: i32 = 7;
    const CRAM_MAP_HASH: i32 = 32;

    let bl = b.cast::<cram_block_layout>();
    let hdr = libc::calloc(
        1,
        std::mem::size_of::<cram_block_compression_hdr_layout>(),
    )
    .cast::<cram_block_compression_hdr_layout>();
    if hdr.is_null() {
        return std::ptr::null_mut();
    }

    if (*bl).method != CRAM_BLOCK_METHOD_RAW
        && cram_uncompress_block(&mut *b.cast()) != 0
    {
        libc::free(hdr.cast());
        return std::ptr::null_mut();
    }

    let mut cp: *mut u8 = (*bl).data.cast::<u8>();
    let endp: *const u8 = cp.add((*bl).uncomp_size as usize).cast_const();

    let fdl = fd.cast::<cram_fd_layout>();
    let major = (*fdl).version >> 8;
    let vv = &(*fdl).vv;
    let mut err: i32 = 0;

    macro_rules! get32 {
        () => {
            (vv.varint_get32.unwrap())(&mut cp, endp, &mut err)
        };
    }
    macro_rules! get64 {
        () => {
            (vv.varint_get64.unwrap())(&mut cp, endp, &mut err)
        };
    }

    if major == 1 {
        (*hdr).ref_seq_id = get32!() as i32;
        if major >= 4 {
            (*hdr).ref_seq_start = get64!();
            (*hdr).ref_seq_span = get64!();
        } else {
            (*hdr).ref_seq_start = get32!();
            (*hdr).ref_seq_span = get32!();
        }
        (*hdr).num_records = get32!() as i32;
        (*hdr).num_landmarks = get32!() as i32;
        if (*hdr).num_landmarks < 0
            || (*hdr).num_landmarks as usize >= usize::MAX / std::mem::size_of::<i32>()
            || (endp.offset_from(cp) as i64) < (*hdr).num_landmarks as i64
        {
            libc::free(hdr.cast());
            return std::ptr::null_mut();
        }
        (*hdr).landmark =
            libc::malloc((*hdr).num_landmarks as usize * std::mem::size_of::<i32>()).cast::<i32>();
        if (*hdr).landmark.is_null() {
            libc::free(hdr.cast());
            return std::ptr::null_mut();
        }
        let mut i = 0;
        while i < (*hdr).num_landmarks {
            *(*hdr).landmark.add(i as usize) = get32!() as i32;
            i += 1;
        }
    }

    // hdr->preservation_map = kh_init(map): calloc(1, sizeof(kh_map_t)).
    (*hdr).preservation_map =
        libc::calloc(1, std::mem::size_of::<kh_generic_layout>()).cast::<()>();
    // rec/tag_encoding_map were zeroed by calloc already (memset to 0 in C).
    if (*hdr).preservation_map.is_null() {
        cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
        return std::ptr::null_mut();
    }
    let pmap = (*hdr).preservation_map.cast::<kh_generic_layout>();

    // Preservation-map defaults.
    (*hdr).read_names_included = 0;
    (*hdr).ap_delta = 1;
    (*hdr).qs_seq_orient = 1;
    std::ptr::copy_nonoverlapping(
        c"CGTNAGTNACTNACGNACGT".as_ptr().cast::<u8>(),
        (&raw mut (*hdr).substitution_matrix).cast::<u8>(),
        20,
    );

    // --- Preservation map ---
    let mut map_size = get32!() as i32;
    let mut cp_copy = cp;
    let mut map_count = get32!() as i32;
    let mut i = 0;
    while i < map_count {
        if (endp.offset_from(cp) as i64) < 3 {
            cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
            return std::ptr::null_mut();
        }
        cp = cp.add(2);
        // CRAM_KEY(a,b) = ((uc)a << 8) | (uc)b
        let key2 = ((*cp.offset(-2) as u8 as i32) << 8) | (*cp.offset(-1) as u8 as i32);
        let mut hd = pmap_val { i: 0 };
        let mut r: i32 = 0;
        // CRAM_KEY constants
        const K_MI: i32 = (b'M' as i32) << 8 | b'I' as i32;
        const K_UI: i32 = (b'U' as i32) << 8 | b'I' as i32;
        const K_PI: i32 = (b'P' as i32) << 8 | b'I' as i32;
        const K_RN: i32 = (b'R' as i32) << 8 | b'N' as i32;
        const K_AP: i32 = (b'A' as i32) << 8 | b'P' as i32;
        const K_RR: i32 = (b'R' as i32) << 8 | b'R' as i32;
        const K_QO: i32 = (b'Q' as i32) << 8 | b'O' as i32;
        const K_SM: i32 = (b'S' as i32) << 8 | b'M' as i32;
        const K_TD: i32 = (b'T' as i32) << 8 | b'D' as i32;
        match key2 {
            K_MI | K_UI | K_PI => {
                // C writes hd.i but never reads it back for these keys.
                let _ = *cp as i32;
                cp = cp.add(1);
            }
            K_RN => {
                hd.i = *cp as i32;
                cp = cp.add(1);
                let k = kh_put_map(pmap, b"RN\0".as_ptr(), &mut r);
                if r == -1 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                *(*pmap).vals.cast::<pmap_val>().add(k as usize) = hd;
                (*hdr).read_names_included = hd.i;
            }
            K_AP => {
                hd.i = *cp as i32;
                cp = cp.add(1);
                let k = kh_put_map(pmap, b"AP\0".as_ptr(), &mut r);
                if r == -1 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                *(*pmap).vals.cast::<pmap_val>().add(k as usize) = hd;
                (*hdr).ap_delta = hd.i;
            }
            K_RR => {
                hd.i = *cp as i32;
                cp = cp.add(1);
                let k = kh_put_map(pmap, b"RR\0".as_ptr(), &mut r);
                if r == -1 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                *(*pmap).vals.cast::<pmap_val>().add(k as usize) = hd;
                (*hdr).no_ref = (hd.i == 0) as i32;
            }
            K_QO => {
                hd.i = *cp as i32;
                cp = cp.add(1);
                let k = kh_put_map(pmap, b"QO\0".as_ptr(), &mut r);
                if r == -1 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                *(*pmap).vals.cast::<pmap_val>().add(k as usize) = hd;
                (*hdr).qs_seq_orient = hd.i;
            }
            K_SM => {
                if (endp.offset_from(cp) as i64) < 5 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                let sm = &mut (*hdr).substitution_matrix;
                let cb = |o: isize| -> i32 { *cp.offset(o) as i32 };
                sm[0][((cb(0) >> 6) & 3) as usize] = b'C' as u8;
                sm[0][((cb(0) >> 4) & 3) as usize] = b'G' as u8;
                sm[0][((cb(0) >> 2) & 3) as usize] = b'T' as u8;
                sm[0][(cb(0) & 3) as usize] = b'N' as u8;
                sm[1][((cb(1) >> 6) & 3) as usize] = b'A' as u8;
                sm[1][((cb(1) >> 4) & 3) as usize] = b'G' as u8;
                sm[1][((cb(1) >> 2) & 3) as usize] = b'T' as u8;
                sm[1][(cb(1) & 3) as usize] = b'N' as u8;
                sm[2][((cb(2) >> 6) & 3) as usize] = b'A' as u8;
                sm[2][((cb(2) >> 4) & 3) as usize] = b'C' as u8;
                sm[2][((cb(2) >> 2) & 3) as usize] = b'T' as u8;
                sm[2][(cb(2) & 3) as usize] = b'N' as u8;
                sm[3][((cb(3) >> 6) & 3) as usize] = b'A' as u8;
                sm[3][((cb(3) >> 4) & 3) as usize] = b'C' as u8;
                sm[3][((cb(3) >> 2) & 3) as usize] = b'G' as u8;
                sm[3][(cb(3) & 3) as usize] = b'N' as u8;
                sm[4][((cb(4) >> 6) & 3) as usize] = b'A' as u8;
                sm[4][((cb(4) >> 4) & 3) as usize] = b'C' as u8;
                sm[4][((cb(4) >> 2) & 3) as usize] = b'G' as u8;
                sm[4][(cb(4) & 3) as usize] = b'T' as u8;
                hd.p = cp.cast::<u8>();
                cp = cp.add(5);
                let k = kh_put_map(pmap, b"SM\0".as_ptr(), &mut r);
                if r == -1 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                *(*pmap).vals.cast::<pmap_val>().add(k as usize) = hd;
            }
            K_TD => {
                let sz = cram_cram_decode_c_71_cram_decode_TD(fd, cp, endp, hdr);
                if sz < 0 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                hd.p = cp.cast::<u8>();
                cp = cp.add(sz as usize);
                let k = kh_put_map(pmap, b"TD\0".as_ptr(), &mut r);
                if r == -1 {
                    cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                    return std::ptr::null_mut();
                }
                *(*pmap).vals.cast::<pmap_val>().add(k as usize) = hd;
            }
            _ => {
                hts_log_cstr(
                    HTS_LOG_WARNING,
                    b"cram_decode_compression_header",
                    b"Unrecognised preservation map key",
                );
                cp = cp.add(1);
            }
        }
        i += 1;
    }
    if cp.offset_from(cp_copy) as i64 != map_size as i64 {
        cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
        return std::ptr::null_mut();
    }

    // --- Record (data-series) encoding map ---
    map_size = get32!() as i32;
    cp_copy = cp;
    map_count = get32!() as i32;
    let is_v4 = if major >= 4 { 1 } else { 0 };
    i = 0;
    while i < map_count {
        let key = cp;
        if (endp.offset_from(cp) as i64) < 4 {
            cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
            return std::ptr::null_mut();
        }
        cp = cp.add(2);
        let encoding = get32!() as i32;
        let size = get32!() as i32;
        let offset = cp.offset_from((*bl).data.cast::<u8>()) as i32;

        if encoding == E_NULL {
            i += 1;
            continue;
        }
        if size < 0 || (endp.offset_from(cp) as i64) < size as i64 {
            cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
            return std::ptr::null_mut();
        }

        let k0 = *key as u8;
        let k1 = *key.add(1) as u8;
        let mut ds_id = DS_CORE;
        let mut type_: i32 = 0;
        match (k0, k1) {
            (b'B', b'F') => {
                ds_id = DS_BF;
                type_ = E_INT;
            }
            (b'C', b'F') => {
                ds_id = DS_CF;
                type_ = E_INT;
            }
            (b'R', b'I') => {
                ds_id = DS_RI;
                type_ = E_INT;
            }
            (b'R', b'L') => {
                ds_id = DS_RL;
                type_ = E_INT;
            }
            (b'A', b'P') => {
                ds_id = DS_AP;
                type_ = if is_v4 != 0 { E_SLONG } else { E_INT };
            }
            (b'R', b'G') => {
                ds_id = DS_RG;
                type_ = E_INT;
            }
            (b'M', b'F') => {
                ds_id = DS_MF;
                type_ = E_INT;
            }
            (b'N', b'S') => {
                ds_id = DS_NS;
                type_ = E_INT;
            }
            (b'N', b'P') => {
                ds_id = DS_NP;
                type_ = if is_v4 != 0 { E_LONG } else { E_INT };
            }
            (b'T', b'S') => {
                ds_id = DS_TS;
                type_ = if is_v4 != 0 { E_SLONG } else { E_INT };
            }
            (b'N', b'F') => {
                ds_id = DS_NF;
                type_ = E_INT;
            }
            (b'T', b'C') => {
                ds_id = DS_TC;
                type_ = E_BYTE;
            }
            (b'T', b'N') => {
                ds_id = DS_TN;
                type_ = E_INT;
            }
            (b'F', b'N') => {
                ds_id = DS_FN;
                type_ = E_INT;
            }
            (b'F', b'C') => {
                ds_id = DS_FC;
                type_ = E_BYTE;
            }
            (b'F', b'P') => {
                ds_id = DS_FP;
                type_ = E_INT;
            }
            (b'B', b'S') => {
                ds_id = DS_BS;
                type_ = E_BYTE;
            }
            (b'I', b'N') => {
                ds_id = DS_IN;
                type_ = E_BYTE_ARRAY;
            }
            (b'S', b'C') => {
                ds_id = DS_SC;
                type_ = E_BYTE_ARRAY;
            }
            (b'D', b'L') => {
                ds_id = DS_DL;
                type_ = E_INT;
            }
            (b'B', b'A') => {
                ds_id = DS_BA;
                type_ = E_BYTE;
            }
            (b'B', b'B') => {
                ds_id = DS_BB;
                type_ = E_BYTE_ARRAY;
            }
            (b'R', b'S') => {
                ds_id = DS_RS;
                type_ = E_INT;
            }
            (b'P', b'D') => {
                ds_id = DS_PD;
                type_ = E_INT;
            }
            (b'H', b'C') => {
                ds_id = DS_HC;
                type_ = E_INT;
            }
            (b'M', b'Q') => {
                ds_id = DS_MQ;
                type_ = E_INT;
            }
            (b'R', b'N') => {
                ds_id = DS_RN;
                type_ = E_BYTE_ARRAY_BLOCK;
            }
            (b'Q', b'S') => {
                ds_id = DS_QS;
                type_ = E_BYTE;
            }
            (b'Q', b'Q') => {
                ds_id = DS_QQ;
                type_ = E_BYTE_ARRAY;
            }
            (b'T', b'L') => {
                ds_id = DS_TL;
                type_ = E_INT;
            }
            (b'T', b'M') | (b'T', b'V') => {}
            _ => {
                hts_log_cstr(
                    HTS_LOG_WARNING,
                    b"cram_decode_compression_header",
                    b"Unrecognised key",
                );
            }
        }

        if ds_id != DS_CORE {
            if !(*hdr).codecs[ds_id as usize].is_null() {
                hts_log_cstr(
                    HTS_LOG_WARNING,
                    b"cram_decode_compression_header",
                    b"Codec for key defined more than once",
                );
                let c = (*hdr).codecs[ds_id as usize].cast::<cram_codec_base_layout>();
                if let Some(free_fn) = (*c).free {
                    free_fn(c);
                }
            }
            (*hdr).codecs[ds_id as usize] = cram_cram_codecs_c_3872_cram_decoder_init(
                hdr.cast(),
                encoding,
                cp,
                size,
                type_,
                (*fdl).version,
                &raw const (*fdl).vv as *mut (),
            )
            .cast();
            if (*hdr).codecs[ds_id as usize].is_null() {
                cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
                return std::ptr::null_mut();
            }
        }

        cp = cp.add(size as usize);

        // Fill out cram_map purely for cram_dump.
        let m = libc::malloc(std::mem::size_of::<cram_map_layout>()).cast::<cram_map_layout>();
        if m.is_null() {
            cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
            return std::ptr::null_mut();
        }
        (*m).key = ((k0 as i32) << 8) | k1 as i32;
        (*m).encoding = encoding;
        (*m).size = size;
        (*m).offset = offset;
        (*m).codec = std::ptr::null_mut();
        let slot = ((k0 as i32 * 3 + k1 as i32) & (CRAM_MAP_HASH - 1)) as usize;
        (*m).next = (*hdr).rec_encoding_map[slot].cast();
        (*hdr).rec_encoding_map[slot] = m.cast();

        i += 1;
    }
    if cp.offset_from(cp_copy) as i64 != map_size as i64 {
        cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
        return std::ptr::null_mut();
    }

    // --- Tag encoding map ---
    map_size = get32!() as i32;
    cp_copy = cp;
    map_count = get32!() as i32;
    i = 0;
    while i < map_count {
        let m = libc::malloc(std::mem::size_of::<cram_map_layout>()).cast::<cram_map_layout>();
        if m.is_null() || (endp.offset_from(cp) as i64) < 6 {
            libc::free(m.cast());
            cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
            return std::ptr::null_mut();
        }
        (*m).key = get32!() as i32;
        let k0 = ((*m).key >> 16) as u8;
        let k1 = ((*m).key >> 8) as u8;
        let encoding = get32!() as i32;
        let size = get32!() as i32;
        (*m).encoding = encoding;
        (*m).size = size;
        (*m).offset = cp.offset_from((*bl).data.cast::<u8>()) as i32;

        let codec = if size < 0 || (endp.offset_from(cp) as i64) < size as i64 {
            std::ptr::null_mut()
        } else {
            cram_cram_codecs_c_3872_cram_decoder_init(
                hdr.cast(),
                encoding,
                cp,
                size,
                E_BYTE_ARRAY_BLOCK,
                (*fdl).version,
                &raw const (*fdl).vv as *mut (),
            )
        };
        (*m).codec = codec.cast();
        if size < 0 || (endp.offset_from(cp) as i64) < size as i64 || codec.is_null() {
            cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
            libc::free(m.cast());
            return std::ptr::null_mut();
        }

        cp = cp.add(size as usize);
        let slot = ((k0 as i32 * 3 + k1 as i32) & (CRAM_MAP_HASH - 1)) as usize;
        (*m).next = (*hdr).tag_encoding_map[slot].cast();
        (*hdr).tag_encoding_map[slot] = m.cast();

        i += 1;
    }
    if err != 0 || cp.offset_from(cp_copy) as i64 != map_size as i64 {
        cram_cram_io_c_4356_cram_free_compression_header(hdr.cast());
        return std::ptr::null_mut();
    }

    hdr.cast()
}
/// original: cram_decode_slice_header (htslib/cram/cram_decode.c:955)
/// Native port. Decodes a slice-header block into a freshly calloc'd
/// `cram_block_slice_hdr`. Mirrors the C control flow exactly, including the
/// version-dependent itf8 vs ltf8 selection (via the fd's `vv` varint vector)
/// and the CRAM-version gating of the `record_counter` and `md5` fields.
/// Ownership matches C: block_content_ids is malloc'd; on error everything is
/// freed and NULL returned.
pub unsafe fn cram_cram_decode_c_955_cram_decode_slice_header(
    fd: *mut cram_fd,
    b: *mut cram_block,
) -> *mut cram_block_slice_hdr {
    let bl = b.cast::<cram_block_layout>();

    // Uncompress in place if the block is not stored RAW.
    if (*bl).method != CRAM_BLOCK_METHOD_RAW
        && cram_uncompress_block(&mut *b.cast()) < 0
    {
        return std::ptr::null_mut();
    }

    let mut cp: *mut u8 = (*bl).data.cast::<u8>();
    let cp_end: *const u8 = cp.add((*bl).uncomp_size as usize).cast_const();

    if (*bl).content_type != CRAM_CONTENT_TYPE_MAPPED_SLICE
        && (*bl).content_type != CRAM_CONTENT_TYPE_UNMAPPED_SLICE
    {
        return std::ptr::null_mut();
    }

    let hdr = libc::calloc(1, std::mem::size_of::<cram_block_slice_hdr_layout>())
        .cast::<cram_block_slice_hdr_layout>();
    if hdr.is_null() {
        return std::ptr::null_mut();
    }

    let fdl = fd.cast::<cram_fd_layout>();
    let major = (*fdl).version >> 8;
    let vv = &(*fdl).vv;
    let mut err: i32 = 0;

    macro_rules! get32 {
        () => {
            (vv.varint_get32.unwrap())(&mut cp, cp_end, &mut err)
        };
    }
    macro_rules! get32s {
        () => {
            (vv.varint_get32s.unwrap())(&mut cp, cp_end, &mut err)
        };
    }
    macro_rules! get64 {
        () => {
            (vv.varint_get64.unwrap())(&mut cp, cp_end, &mut err)
        };
    }

    (*hdr).content_type = (*bl).content_type;

    if (*bl).content_type == CRAM_CONTENT_TYPE_MAPPED_SLICE {
        (*hdr).ref_seq_id = get32s!() as i32;
        if major >= 4 {
            (*hdr).ref_seq_start = get64!();
            (*hdr).ref_seq_span = get64!();
        } else {
            (*hdr).ref_seq_start = get32!();
            (*hdr).ref_seq_span = get32!();
        }
        if (*hdr).ref_seq_start < 0 || (*hdr).ref_seq_span < 0 {
            libc::free(hdr.cast());
            hts_log_cstr(
                HTS_LOG_ERROR,
                b"cram_decode_slice_header",
                b"Negative values not permitted for header sequence start or span fields",
            );
            return std::ptr::null_mut();
        }
    }

    (*hdr).num_records = get32!() as i32;
    (*hdr).record_counter = 0;
    if major == 2 {
        (*hdr).record_counter = get32!();
    } else if major >= 3 {
        (*hdr).record_counter = get64!();
    }

    (*hdr).num_blocks = get32!() as i32;
    (*hdr).num_content_ids = get32!() as i32;
    if (*hdr).num_content_ids < 1 || (*hdr).num_content_ids >= 10000 {
        libc::free(hdr.cast());
        return std::ptr::null_mut();
    }

    (*hdr).block_content_ids =
        libc::malloc((*hdr).num_content_ids as usize * std::mem::size_of::<i32>()).cast::<i32>();
    if (*hdr).block_content_ids.is_null() {
        libc::free(hdr.cast());
        return std::ptr::null_mut();
    }

    let mut i = 0;
    while i < (*hdr).num_content_ids {
        *(*hdr).block_content_ids.add(i as usize) = get32!() as i32;
        i += 1;
    }
    if err != 0 {
        libc::free((*hdr).block_content_ids.cast());
        libc::free(hdr.cast());
        return std::ptr::null_mut();
    }

    if (*bl).content_type == CRAM_CONTENT_TYPE_MAPPED_SLICE {
        (*hdr).ref_base_id = get32!() as i32;
    }

    if major != 1 {
        if (cp_end as isize - cp as isize) < 16 {
            libc::free((*hdr).block_content_ids.cast());
            libc::free(hdr.cast());
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(cp.cast_const().cast::<u8>(), (&raw mut (*hdr).md5).cast::<u8>(), 16);
    } else {
        (*hdr).md5 = [0u8; 16];
    }

    if err == 0 {
        return hdr.cast();
    }
    libc::free((*hdr).block_content_ids.cast());
    libc::free(hdr.cast());
    std::ptr::null_mut()
}
