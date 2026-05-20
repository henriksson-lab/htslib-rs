use crate::htslib_rs::{
    cram::{cram_block, cram_codec, cram_content_type, cram_string_alloc_t, mFILE},
    hts::hts_pos_t,
    sam::bam1_t,
};
use std::ffi::{c_char, c_int};

pub const MAX_STAT_VAL: usize = 1024;
pub const CRAM_MAX_METHOD: usize = 32;
pub const CRAM_MAP_HASH: usize = 32;
pub const DS_END: usize = 47;

pub type cram_encoding = c_int;
pub const E_NULL: cram_encoding = 0;
pub const E_EXTERNAL: cram_encoding = 1;
pub const E_GOLOMB: cram_encoding = 2;
pub const E_HUFFMAN: cram_encoding = 3;
pub const E_BYTE_ARRAY_LEN: cram_encoding = 4;
pub const E_BYTE_ARRAY_STOP: cram_encoding = 5;
pub const E_BETA: cram_encoding = 6;
pub const E_SUBEXP: cram_encoding = 7;
pub const E_GOLOMB_RICE: cram_encoding = 8;
pub const E_GAMMA: cram_encoding = 9;
pub const E_VARINT_UNSIGNED: cram_encoding = 41;
pub const E_VARINT_SIGNED: cram_encoding = 42;
pub const E_CONST_BYTE: cram_encoding = 43;
pub const E_CONST_INT: cram_encoding = 44;
pub const E_XHUFFMAN: cram_encoding = 50;
pub const E_XPACK: cram_encoding = 51;
pub const E_XRLE: cram_encoding = 52;
pub const E_XDELTA: cram_encoding = 53;
pub const E_NUM_CODECS: cram_encoding = 54;

pub type cram_block_method_int = c_int;

#[repr(C)]
pub struct kh_m_i2i_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct kh_m_s2i_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct kh_map_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct kh_m_tagmap_t {
    _private: [u8; 0],
}

// original: cram_stats (htslib/cram/cram_structs.h:95)
#[repr(C)]
pub struct cram_stats {
    pub freqs: [c_int; MAX_STAT_VAL],
    pub h: *mut kh_m_i2i_t,
    pub nsamp: c_int,
    pub nvals: c_int,
    pub min_val: i64,
    pub max_val: i64,
}

// original: cram_file_def (htslib/cram/cram_structs.h:200)
#[repr(C)]
pub struct cram_file_def {
    pub magic: [c_char; 4],
    pub major_version: u8,
    pub minor_version: u8,
    pub file_id: [c_char; 20],
}

// original: cram_metrics (htslib/cram/cram_structs.h:284)
#[repr(C)]
pub struct cram_metrics {
    pub trial: c_int,
    pub next_trial: c_int,
    pub consistency: c_int,
    pub sz: [c_int; CRAM_MAX_METHOD],
    pub input_avg_sz: c_int,
    pub input_avg_delta: c_int,
    pub method: c_int,
    pub revised_method: c_int,
    pub strat: c_int,
    pub cnt: [c_int; CRAM_MAX_METHOD],
    pub extra: [f64; CRAM_MAX_METHOD],
    pub unpackable: c_int,
}

// original: cram_block_compression_hdr (htslib/cram/cram_structs.h:341)
#[repr(C)]
pub struct cram_block_compression_hdr {
    pub ref_seq_id: i32,
    pub ref_seq_start: i64,
    pub ref_seq_span: i64,
    pub num_records: i32,
    pub num_landmarks: i32,
    pub landmark: *mut i32,
    pub read_names_included: c_int,
    pub AP_delta: c_int,
    pub substitution_matrix: [[c_char; 4]; 5],
    pub no_ref: c_int,
    pub qs_seq_orient: c_int,
    pub TD_blk: *mut cram_block,
    pub nTL: c_int,
    pub TL: *mut *mut u8,
    pub TD_hash: *mut kh_m_s2i_t,
    pub TD_keys: *mut cram_string_alloc_t,
    pub preservation_map: *mut kh_map_t,
    pub rec_encoding_map: [*mut cram_map; CRAM_MAP_HASH],
    pub tag_encoding_map: [*mut cram_map; CRAM_MAP_HASH],
    pub codecs: [*mut cram_codec; DS_END],
    pub uncomp: *mut c_char,
    pub uncomp_size: usize,
    pub uncomp_alloc: usize,
    pub ncodecs: c_int,
}

// original: cram_map (htslib/cram/cram_structs.h:377)
#[repr(C)]
pub struct cram_map {
    pub key: c_int,
    pub encoding: cram_encoding,
    pub offset: c_int,
    pub size: c_int,
    pub codec: *mut cram_codec,
    pub next: *mut cram_map,
}

// original: cram_tag_map (htslib/cram/cram_structs.h:386)
#[repr(C)]
pub struct cram_tag_map {
    pub codec: *mut cram_codec,
    pub blk: *mut cram_block,
    pub blk2: *mut cram_block,
    pub m: *mut cram_metrics,
}

// original: cram_block_slice_hdr (htslib/cram/cram_structs.h:397)
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

// original: cram_container (htslib/cram/cram_structs.h:422)
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
    pub ref_start: hts_pos_t,
    pub first_base: hts_pos_t,
    pub last_base: hts_pos_t,
    pub ref_end: hts_pos_t,
    pub ref_: *mut c_char,
    pub embed_ref: c_int,
    pub no_ref: c_int,
    pub bams: *mut *mut bam1_t,
    pub stats: [*mut cram_stats; DS_END],
    pub tags_used: *mut kh_m_tagmap_t,
    pub refs_used: *mut c_int,
    pub crc32: u32,
    pub s_num_bases: u64,
    pub s_aux_bytes: u64,
    pub n_mapped: u32,
    pub ref_free: c_int,
}

// original: cram_record (htslib/cram/cram_structs.h:486)
#[repr(C)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_X {
    pub pos: c_int,
    pub code: c_int,
    pub base: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_B {
    pub pos: c_int,
    pub code: c_int,
    pub base: c_int,
    pub qual: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_b {
    pub pos: c_int,
    pub code: c_int,
    pub seq_idx: c_int,
    pub len: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_Q {
    pub pos: c_int,
    pub code: c_int,
    pub qual: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_S {
    pub pos: c_int,
    pub code: c_int,
    pub len: c_int,
    pub seq_idx: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_i {
    pub pos: c_int,
    pub code: c_int,
    pub base: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cram_feature_len {
    pub pos: c_int,
    pub code: c_int,
    pub len: c_int,
}

// original: cram_feature (htslib/cram/cram_structs.h:541)
#[repr(C)]
pub union cram_feature {
    pub X: cram_feature_X,
    pub B: cram_feature_B,
    pub b: cram_feature_b,
    pub Q: cram_feature_Q,
    pub S: cram_feature_S,
    pub I: cram_feature_S,
    pub i: cram_feature_i,
    pub D: cram_feature_len,
    pub N: cram_feature_len,
    pub P: cram_feature_len,
    pub H: cram_feature_len,
}

// original: cram_slice (htslib/cram/cram_structs.h:608)
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
    pub pair: [*mut kh_m_s2i_t; 2],
    pub ref_: *mut c_char,
    pub ref_start: hts_pos_t,
    pub ref_end: hts_pos_t,
    pub ref_id: c_int,
    pub naux_block: c_int,
    pub aux_block: *mut *mut cram_block,
    pub data_series: u32,
    pub decode_md: c_int,
    pub max_rec: c_int,
    pub curr_rec: c_int,
    pub slice_num: c_int,
}

// original: ref_entry (htslib/cram/cram_structs.h:671)
#[repr(C)]
pub struct ref_entry {
    pub name: *mut c_char,
    pub fn_: *mut c_char,
    pub length: i64,
    pub LN_length: i64,
    pub offset: i64,
    pub bases_per_line: c_int,
    pub line_length: c_int,
    pub count: i64,
    pub seq: *mut c_char,
    pub mf: *mut mFILE,
    pub is_md5: c_int,
    pub validated_md5: c_int,
}

// original: cram_index (htslib/cram/cram_structs.h:720)
#[repr(C)]
pub struct cram_index {
    pub nslice: c_int,
    pub nalloc: c_int,
    pub e: *mut cram_index,
    pub refid: c_int,
    pub start: c_int,
    pub end: c_int,
    pub nseq: c_int,
    pub slice: c_int,
    pub len: c_int,
    pub offset: i64,
    pub e_next: *mut cram_index,
}

// original: spare_bams (htslib/cram/cram_structs.h:747)
#[repr(C)]
pub struct spare_bams {
    pub bams: *mut *mut bam1_t,
    pub next: *mut spare_bams,
}

#[repr(C)]
pub struct cram_fd {
    _private: [u8; 0],
}

// original: varint_vec (htslib/cram/cram_structs.h:753)
#[repr(C)]
pub struct varint_vec {
    pub varint_decode32_crc:
        Option<unsafe extern "C" fn(fd: *mut cram_fd, val_p: *mut i32, crc: *mut u32) -> c_int>,
    pub varint_decode32s_crc:
        Option<unsafe extern "C" fn(fd: *mut cram_fd, val_p: *mut i32, crc: *mut u32) -> c_int>,
    pub varint_decode64_crc:
        Option<unsafe extern "C" fn(fd: *mut cram_fd, val_p: *mut i64, crc: *mut u32) -> c_int>,
    pub varint_get32: Option<
        unsafe extern "C" fn(cp: *mut *mut c_char, endp: *const c_char, err: *mut c_int) -> i64,
    >,
    pub varint_get32s: Option<
        unsafe extern "C" fn(cp: *mut *mut c_char, endp: *const c_char, err: *mut c_int) -> i64,
    >,
    pub varint_get64: Option<
        unsafe extern "C" fn(cp: *mut *mut c_char, endp: *const c_char, err: *mut c_int) -> i64,
    >,
    pub varint_get64s: Option<
        unsafe extern "C" fn(cp: *mut *mut c_char, endp: *const c_char, err: *mut c_int) -> i64,
    >,
    pub varint_put32:
        Option<unsafe extern "C" fn(cp: *mut c_char, endp: *mut c_char, val_p: i32) -> c_int>,
    pub varint_put32s:
        Option<unsafe extern "C" fn(cp: *mut c_char, endp: *mut c_char, val_p: i32) -> c_int>,
    pub varint_put64:
        Option<unsafe extern "C" fn(cp: *mut c_char, endp: *mut c_char, val_p: i64) -> c_int>,
    pub varint_put64s:
        Option<unsafe extern "C" fn(cp: *mut c_char, endp: *mut c_char, val_p: i64) -> c_int>,
    pub varint_put32_blk: Option<unsafe extern "C" fn(blk: *mut cram_block, val_p: i32) -> c_int>,
    pub varint_put32s_blk: Option<unsafe extern "C" fn(blk: *mut cram_block, val_p: i32) -> c_int>,
    pub varint_put64_blk: Option<unsafe extern "C" fn(blk: *mut cram_block, val_p: i64) -> c_int>,
    pub varint_put64s_blk: Option<unsafe extern "C" fn(blk: *mut cram_block, val_p: i64) -> c_int>,
    pub varint_size: Option<unsafe extern "C" fn(val: i64) -> c_int>,
}
