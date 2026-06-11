use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::CStr,
    mem,
    ptr::NonNull,
    sync::{Mutex, OnceLock},
};

use crate::htslib_rs::bgzf::{
    bgzf_check_EOF, bgzf_flush, bgzf_flush_try, bgzf_peek, bgzf_read, bgzf_read_small, bgzf_seek,
    bgzf_write, bgzf_write_small,
};
use crate::htslib_rs::hfile::hflush;
use crate::htslib_rs::hts::{
    __ac_FNV1a_hash_string, __ac_Wang_hash, __ac_X31_hash_string, double_to_le, ed_swap_4p,
    find_file_extension, float_to_le, htsFile, htsLogLevel, hts_bin_maxpos, hts_expr_val_t,
    hts_filter_eval2, hts_filter_t, hts_getline, hts_idx_destroy, hts_idx_finish, hts_idx_init,
    hts_idx_load3, hts_idx_push, hts_idx_save_as, hts_idx_t, hts_itr_multi_bam, hts_itr_multi_cram,
    hts_itr_multi_next, hts_itr_multi_query_func, hts_itr_next, hts_itr_query, hts_itr_regions,
    hts_itr_t, hts_name2id_f, hts_parse_region, hts_pos_t, hts_readrec_func, hts_reg2bin,
    hts_reglist_create, hts_reglist_free, hts_reglist_t, hts_seek_func, hts_str2int, hts_str2uint,
    hts_tell_func, i16_to_le, i32_to_le, isalnum_c, isalpha_c, isdigit_c, islower_c, isspace_c,
    isupper_c, kputc, kputc_, kputll, kputs, kputsn, kputsn_, kputuw, kputw, ks_clear, ks_expand,
    ks_free, ks_release, ks_resize, kstring_t, toupper_c, u16_to_le, u32_to_le, u64_to_le, BGZF,
    HTS_FMT_BAI, HTS_FMT_CRAI, HTS_FMT_CSI, HTS_FORMAT_BAM, HTS_FORMAT_BINARY_FORMAT,
    HTS_FORMAT_CRAM, HTS_FORMAT_EMPTY_FORMAT, HTS_FORMAT_FASTA_FORMAT, HTS_FORMAT_FASTQ_FORMAT,
    HTS_FORMAT_SAM, HTS_FORMAT_SEQUENCE_DATA, HTS_FORMAT_TEXT_FORMAT, HTS_IDX_NOCOOR,
    HTS_IDX_SAVE_REMOTE, HTS_IDX_START, HTS_MAX_EXT_LEN, HTS_PARSE_THOUSANDS_SEP, HTS_POS_MAX,
};

use crate::htslib_rs::hfile::hpeek;

// Re-exports of items extracted into sibling files at the crate root.
// The Rust file layout mirrors htslib: htslib/header.c -> src/header.rs,
// htslib/sam_mods.c -> src/sam_mods.rs. These re-exports preserve the
// public surface so `crate::htslib_rs::sam::sam_hdr_destroy` etc. still
// resolves the same way it did when these files lived in src/sam/.
pub use crate::htslib_rs::header::*;
pub use crate::htslib_rs::sam_mods::*;

pub const BAM_CMATCH: i32 = 0;
pub const BAM_CINS: i32 = 1;
pub const BAM_CDEL: i32 = 2;
pub const BAM_CREF_SKIP: i32 = 3;
pub const BAM_CSOFT_CLIP: i32 = 4;
pub const BAM_CHARD_CLIP: i32 = 5;
pub const BAM_CPAD: i32 = 6;
pub const BAM_CEQUAL: i32 = 7;
pub const BAM_CDIFF: i32 = 8;
pub const BAM_CBACK: i32 = 9;
pub const BAM_CIGAR_SHIFT: u32 = 4;
pub const BAM_CIGAR_MASK: u32 = 0x0f;

pub const BAM_CIGAR_TYPE: [i32; 16] = [3, 1, 2, 2, 1, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0];
pub static BAM_CIGAR_TABLE: [i8; 256] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 7, -1, -1, -1, -1, 9, -1, 2, -1, -1, -1, 5,
    1, -1, -1, -1, 0, 3, -1, 6, -1, -1, 4, -1, -1, -1, -1, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];
pub static SEQ_NT16_TABLE: [u8; 256] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    1, 2, 4, 8, 15, 15, 15, 15, 15, 15, 15, 15, 15, 0, 15, 15, 15, 1, 14, 2, 13, 15, 15, 4, 11, 15,
    15, 12, 15, 3, 15, 15, 15, 15, 5, 6, 8, 8, 7, 9, 15, 10, 15, 15, 15, 15, 15, 15, 15, 1, 14, 2,
    13, 15, 15, 4, 11, 15, 15, 12, 15, 3, 15, 15, 15, 15, 5, 6, 8, 8, 7, 9, 15, 10, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
];

const UMI_TAGS: usize = 5;
pub const FASTQ_OPT_CASAVA: i32 = 1000;
pub const FASTQ_OPT_AUX: i32 = 1001;
pub const FASTQ_OPT_RNUM: i32 = 1002;
pub const FASTQ_OPT_BARCODE: i32 = 1003;
pub const FASTQ_OPT_NAME2: i32 = 1004;
pub const FASTQ_OPT_UMI: i32 = 1005;
pub const FASTQ_OPT_UMI_REGEX: i32 = 1006;

pub const BAM_FPAIRED: i32 = 1;
pub const BAM_FPROPER_PAIR: i32 = 2;
pub const BAM_FUNMAP: i32 = 4;
pub const BAM_FMUNMAP: i32 = 8;
pub const BAM_FREVERSE: i32 = 16;
pub const BAM_FMREVERSE: i32 = 32;
pub const BAM_FREAD1: i32 = 64;
pub const BAM_FREAD2: i32 = 128;
pub const BAM_FSECONDARY: i32 = 256;
pub const BAM_FQCFAIL: i32 = 512;
pub const BAM_FDUP: i32 = 1024;
pub const BAM_FSUPPLEMENTARY: i32 = 2048;

pub const BAM_USER_OWNS_STRUCT: u32 = 1;
pub const BAM_USER_OWNS_DATA: u32 = 2;
pub const SAM_FORMAT_VERSION: &str = "1.6";
pub const HTS_MOD_UNKNOWN: i32 = -1;
pub const HTS_MOD_UNCHECKED: i32 = -2;
pub const HTS_MOD_REPORT_UNCHECKED: u32 = 1;
pub const ORDER_UNKNOWN: i32 = -1;
pub const ORDER_UNSORTED: i32 = 0;
pub const ORDER_NAME: i32 = 1;
pub const ORDER_COORD: i32 = 2;
pub const ORDER_GO_UNKNOWN: i32 = -1;
pub const ORDER_GO_NONE: i32 = 0;
pub const ORDER_GO_QUERY: i32 = 1;
pub const ORDER_GO_REFERENCE: i32 = 2;

pub(crate) const SEQI_RC: [i32; 16] = [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];
pub(crate) const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

fn set_pileup_is_del(p: &mut bam_pileup1_t, value: bool) {
    if value {
        p.bitfields |= 1;
    } else {
        p.bitfields &= !1;
    }
}

fn set_pileup_is_head(p: &mut bam_pileup1_t, value: bool) {
    if value {
        p.bitfields |= 1 << 1;
    } else {
        p.bitfields &= !(1 << 1);
    }
}

fn set_pileup_is_tail(p: &mut bam_pileup1_t, value: bool) {
    if value {
        p.bitfields |= 1 << 2;
    } else {
        p.bitfields &= !(1 << 2);
    }
}

fn set_pileup_is_refskip(p: &mut bam_pileup1_t, value: bool) {
    if value {
        p.bitfields |= 1 << 3;
    } else {
        p.bitfields &= !(1 << 3);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct bam1_core_t {
    pub pos: hts_pos_t,
    pub tid: i32,
    pub bin: u16,
    pub qual: u8,
    pub l_extranul: u8,
    pub flag: u16,
    pub l_qname: u16,
    pub n_cigar: u32,
    pub l_qseq: i32,
    pub mtid: i32,
    pub mpos: hts_pos_t,
    pub isize: hts_pos_t,
}

// SEAM (bam1_t keystone, phase 1): the packed BAM record blob is now an OWNED
// `Vec<u8>`. The former `l_data` (used bytes) and `m_data` (capacity) fields are
// COLLAPSED into the Vec: logical `l_data == data.len()`, `m_data == data.capacity()`.
//
// INVARIANT (resize-to-len): `data.len()` always equals the logical `l_data`.
// Any path that grows the record (sam_realloc_bam_data / realloc_bam_data) MUST
// `resize(new_len, 0)` so that the full used range is in-bounds for the raw
// `as_mut_ptr().add(off)` writes done by the bam_get_* accessors and their
// callers. The sweep maps `(*b).l_data` -> `(*b).data.len() as i32` and
// `(*b).m_data` -> `(*b).data.capacity() as u32`.
//
// Owned bam1_t is NO LONGER #[repr(C)]; the repr(C) FFI mirror is `bam1_c_t`
// below (parity feature only).
pub struct bam1_t {
    pub core: bam1_core_t,
    pub id: u64,
    pub data: Vec<u8>,
    pub mempolicy_and_reserved: u32,
}

impl Default for bam1_core_t {
    fn default() -> Self {
        Self {
            pos: 0,
            tid: 0,
            bin: 0,
            qual: 0,
            l_extranul: 0,
            flag: 0,
            l_qname: 0,
            n_cigar: 0,
            l_qseq: 0,
            mtid: 0,
            mpos: 0,
            isize: 0,
        }
    }
}

impl Default for bam1_t {
    fn default() -> Self {
        Self {
            core: bam1_core_t::default(),
            id: 0,
            data: Vec::new(),
            mempolicy_and_reserved: 0,
        }
    }
}

// PARITY BRIDGE: #[repr(C)] mirror of the original C-ABI bam1_t layout, shared
// with hts_sys for FFI. Conversion to/from owned `bam1_t` is implemented by the
// sweep; this only defines the layout.
#[cfg(feature = "parity")]
#[repr(C)]
pub struct bam1_c_t {
    pub core: bam1_core_t,
    pub id: u64,
    pub data: *mut u8,
    pub l_data: i32,
    pub m_data: u32,
    pub mempolicy_and_reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union bam_pileup_cd {
    pub p: *mut (),
    pub i: i64,
    pub f: f64,
}

#[repr(C)]
pub struct bam_pileup1_t {
    pub b: *mut bam1_t,
    pub qpos: i32,
    pub indel: i32,
    pub level: i32,
    pub bitfields: u32,
    pub cd: bam_pileup_cd,
    pub cigar_ind: i32,
}

impl Default for bam_pileup1_t {
    fn default() -> Self {
        Self {
            b: std::ptr::null_mut(),
            qpos: 0,
            indel: 0,
            level: 0,
            bitfields: 0,
            cd: bam_pileup_cd { i: 0 },
            cigar_ind: 0,
        }
    }
}

pub const MAX_BASE_MOD: usize = 256;

#[repr(C)]
pub struct hts_base_mod_state {
    pub type_: [i32; MAX_BASE_MOD],
    pub canonical: [i32; MAX_BASE_MOD],
    pub strand: [u8; MAX_BASE_MOD],
    pub mmcount: [i32; MAX_BASE_MOD],
    pub mm: [*mut u8; MAX_BASE_MOD],
    pub mmend: [*mut u8; MAX_BASE_MOD],
    pub ml: [*mut u8; MAX_BASE_MOD],
    pub mlstride: [i32; MAX_BASE_MOD],
    pub implicit: [i32; MAX_BASE_MOD],
    pub seq_pos: i32,
    pub nmods: i32,
    pub flags: u32,
}

#[repr(C)]
pub struct sp_bams {
    pub next: Option<NonNull<sp_bams>>,
    pub serial: i32,
    pub bams: *mut bam1_t,
    pub nbams: i32,
    pub abams: i32,
    pub bam_mem: usize,
    pub fd: Option<NonNull<SAM_state>>,
}

#[repr(C)]
pub struct sp_lines {
    pub next: Option<NonNull<sp_lines>>,
    pub serial: i32,
    pub data: *mut u8,
    pub data_size: i32,
    pub alloc: i32,
    pub fd: Option<NonNull<SAM_state>>,
    pub bams: Option<NonNull<sp_bams>>,
}

#[repr(C)]
pub struct SAM_state {
    pub h: *mut sam_hdr_t,
    pub p: *mut (),
    pub own_pool: i32,
    pub lines: Option<NonNull<sp_lines>>,
    pub bams: Option<NonNull<sp_bams>>,
    pub curr_bam: Option<NonNull<sp_bams>>,
    pub curr_idx: i32,
    pub serial: i32,
    pub command: i32,
    pub errcode: i32,
    pub fp: *mut htsFile,
}

#[repr(C)]
struct hb_pair {
    h: *const sam_hdr_t,
    b: *const bam1_t,
}

pub struct fastq_state {
    pub name: kstring_t,
    pub comment: kstring_t,
    pub seq: kstring_t,
    pub qual: kstring_t,
    pub casava: i32,
    pub aux: i32,
    pub rnum: i32,
    pub BC: [u8; 3],
    pub UMI: [[u8; 3]; UMI_TAGS],
    pub tags: *mut (),
    pub nprefix: u8,
    pub sra_names: i32,
    pub regex: libc::regex_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_base_mod {
    pub modified_base: i32,
    pub canonical_base: i32,
    pub strand: i32,
    pub qual: i32,
}

#[repr(C)]
pub struct sam_hrec_sq_t {
    pub name: *const u8,
    pub len: hts_pos_t,
    pub ty: *mut (),
}

#[repr(C)]
pub struct sam_hrec_rg_t {
    pub name: *const u8,
    pub ty: *mut (),
    pub name_len: i32,
    pub id: i32,
}

#[repr(C)]
pub struct sam_hrec_pg_t {
    pub name: *const u8,
    pub ty: *mut (),
    pub name_len: i32,
    pub id: i32,
    pub prev_id: i32,
}

// original: sam_hrec_tag_s (htslib/header.h:98)
#[repr(C)]
pub struct sam_hrec_tag_t {
    pub next: *mut sam_hrec_tag_t,
    pub str_: *const u8,
    pub len: i32,
}

// original: sam_hrec_type_s (htslib/header.h:119)
#[repr(C)]
pub struct sam_hrec_type_t {
    pub next: *mut sam_hrec_type_t,
    pub prev: *mut sam_hrec_type_t,
    pub global_next: *mut sam_hrec_type_t,
    pub global_prev: *mut sam_hrec_type_t,
    pub tag: *mut sam_hrec_tag_t,
    pub type_: u32,
}

#[repr(C)]
pub struct sam_hrecs_t {
    pub h: *mut (),
    pub first_line: *mut (),
    pub str_pool: *mut (),
    pub type_pool: *mut (),
    pub tag_pool: *mut (),
    pub nref: i32,
    pub ref_sz: i32,
    pub ref_: *mut sam_hrec_sq_t,
    pub ref_hash: *mut (),
    pub nrg: i32,
    pub rg_sz: i32,
    pub rg: *mut (),
    pub rg_hash: *mut (),
    pub npg: i32,
    pub pg_sz: i32,
    pub npg_end: i32,
    pub npg_end_alloc: i32,
    pub pg: *mut (),
    pub pg_hash: *mut (),
    pub pg_end: *mut i32,
    pub ID_buf: *mut u8,
    pub ID_buf_sz: u32,
    pub ID_cnt: i32,
    pub dirty: i32,
    pub refs_changed: i32,
    pub pgs_changed: i32,
    pub type_count: i32,
    pub type_order: *mut [u8; 3],
}

#[repr(C)]
pub struct khash_s2i_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut *mut u8,
    pub vals: *mut i64,
}

#[repr(C)]
pub struct khash_tag_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut i32,
}

#[repr(C)]
pub struct khash_m_s2i_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut *mut u8,
    pub vals: *mut i32,
}

#[repr(C)]
pub struct sam_hdr_t {
    pub n_targets: i32,
    pub ignore_sam_err: i32,
    pub l_text: usize,
    pub target_len: *mut u32,
    pub cigar_tab: *const i8,
    pub target_name: *mut *mut u8,
    pub text: *mut u8,
    pub sdict: *mut (),
    pub hrecs: *mut sam_hrecs_t,
    pub ref_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cstate_t {
    pub k: i32,
    pub y: i32,
    pub x: hts_pos_t,
    pub end: hts_pos_t,
}

pub const G_CSTATE_NULL: cstate_t = cstate_t {
    k: -1,
    y: 0,
    x: 0,
    end: 0,
};

#[repr(C)]
pub struct lbnode_t {
    pub b: bam1_t,
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    pub s: cstate_t,
    pub next: Option<NonNull<lbnode_t>>,
    pub cd: bam_pileup_cd,
}

pub type __linkbuf_t = lbnode_t;

#[repr(C)]
pub struct mempool_t {
    pub cnt: i32,
    pub n: i32,
    pub max: i32,
    pub padding_0: i32,
    pub buf: Vec<NonNull<lbnode_t>>,
}

#[repr(C)]
pub struct olap_hash_t {
    _private: [u8; 0],
}

type OlapHash = HashMap<Vec<u8>, NonNull<lbnode_t>>;

fn olap_hash_new() -> Box<OlapHash> {
    Box::new(OlapHash::new())
}

unsafe fn olap_hash_mut(ptr: NonNull<olap_hash_t>) -> &'static mut OlapHash {
    &mut *ptr.as_ptr().cast::<OlapHash>()
}

unsafe fn olap_hash_free(ptr: NonNull<olap_hash_t>) {
    drop(Box::from_raw(ptr.as_ptr().cast::<OlapHash>()));
}

pub type bam_plp_t = *mut bam_plp_s;
pub type bam_plp_auto_f = Option<unsafe extern "C" fn(*mut (), *mut bam1_t) -> i32>;
pub type bam_plp_constructor_f =
    Option<unsafe extern "C" fn(*mut (), *const bam1_t, *mut bam_pileup_cd) -> i32>;

#[repr(C)]
pub struct bam_plp_s {
    pub mp: Option<Box<mempool_t>>,
    pub head: Option<NonNull<lbnode_t>>,
    pub tail: Option<NonNull<lbnode_t>>,
    pub tid: i32,
    pub max_tid: i32,
    pub pos: hts_pos_t,
    pub max_pos: hts_pos_t,
    pub is_eof: i32,
    pub max_plp: i32,
    pub error: i32,
    pub maxcnt: i32,
    pub id: u64,
    pub plp: Vec<bam_pileup1_t>,
    pub b: Option<NonNull<bam1_t>>,
    pub func: bam_plp_auto_f,
    pub data: *mut (),
    pub overlaps: Option<NonNull<olap_hash_t>>,
    pub plp_construct: bam_plp_constructor_f,
    pub plp_destruct: bam_plp_constructor_f,
}

pub type bam_mplp_t = *mut bam_mplp_s;

#[repr(C)]
pub struct bam_mplp_s {
    pub n: i32,
    pub min_tid: i32,
    pub tid: Vec<i32>,
    pub min_pos: hts_pos_t,
    pub pos: Vec<hts_pos_t>,
    pub iter: Vec<NonNull<bam_plp_s>>,
    pub n_plp: Vec<i32>,
    pub plp: Vec<*const bam_pileup1_t>,
}

unsafe fn parse_sq_target(line: &[u8]) -> Option<(&[u8], hts_pos_t)> {
    if !line.starts_with(b"@SQ\t") {
        return None;
    }
    let mut name = None;
    let mut len = None;
    for field in line.split(|&b| b == b'\t').skip(1) {
        let field = field.strip_suffix(b"\r").unwrap_or(field);
        if let Some(value) = field.strip_prefix(b"SN:") {
            name = Some(value);
        } else if let Some(raw_len) = field.strip_prefix(b"LN:") {
            let mut value = 0u64;
            let mut digits = raw_len;
            if digits.first() == Some(&b'+') {
                digits = &digits[1..];
            }
            if digits.is_empty() || !digits[0].is_ascii_digit() {
                return None;
            }
            for &ch in digits {
                if !ch.is_ascii_digit() {
                    break;
                }
                value = value.checked_mul(10)?.checked_add((ch - b'0') as u64)?;
                if value > HTS_POS_MAX as u64 {
                    return None;
                }
            }
            if value == 0 {
                return None;
            }
            let value = value as hts_pos_t;
            if let Some(prev) = len {
                if prev != value {
                    return None;
                }
            } else {
                len = Some(value);
            }
        }
    }
    Some((name?, len?))
}

fn sam_hdr_text_contains_sq_line(text: &[u8]) -> bool {
    text.split(|&b| b == b'\n')
        .any(|raw| raw.strip_suffix(b"\r").unwrap_or(raw).starts_with(b"@SQ\t"))
}

unsafe fn sam_hdr_validate_ref_aliases_from_text(h: *const sam_hdr_t) -> i32 {
    if h.is_null() || (*h).text.is_null() || (*h).l_text == 0 {
        return 0;
    }

    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    for raw in text.split(|&b| b == b'\n') {
        let line = raw.strip_suffix(b"\r").unwrap_or(raw);
        if !line.starts_with(b"@SQ\t") {
            continue;
        }

        let Some((name, _)) = parse_sq_target(line) else {
            return -1;
        };
        if name.is_empty() || !seen.insert(name.to_vec()) {
            return -1;
        }

        if let Some(aliases) = sam_hdr_text_find_tag_value(line, b"AN") {
            for alias in aliases.split(|&b| b == b',') {
                if alias.is_empty() || !seen.insert(alias.to_vec()) {
                    return -1;
                }
            }
        }
    }

    0
}

unsafe fn kh_resize_s2i(h: *mut khash_s2i_t, new_n_buckets: u32) -> i32 {
    let n_flags = if new_n_buckets < 16 {
        1
    } else {
        new_n_buckets >> 4
    };
    let mut flags_vec: Vec<u32> = vec![0u32; n_flags as usize];
    let mut keys_vec: Vec<*mut u8> = vec![std::ptr::null_mut(); new_n_buckets as usize];
    let mut vals_vec: Vec<i64> = vec![0i64; new_n_buckets as usize];
    let flags = flags_vec.as_mut_ptr();
    let keys = keys_vec.as_mut_ptr();
    let vals = vals_vec.as_mut_ptr();
    std::mem::forget(flags_vec);
    std::mem::forget(keys_vec);
    std::mem::forget(vals_vec);
    for i in 0..n_flags {
        *flags.add(i as usize) = 0xaaaa_aaaa;
    }

    let old_flags = (*h).flags;
    let old_keys = (*h).keys;
    let old_vals = (*h).vals;
    let old_n = (*h).n_buckets;
    let old_n_flags = if old_n < 16 { 1 } else { old_n >> 4 };

    (*h).flags = flags;
    (*h).keys = keys;
    (*h).vals = vals;
    (*h).n_buckets = new_n_buckets;
    (*h).size = 0;
    (*h).n_occupied = 0;
    (*h).upper_bound = (new_n_buckets as f64 * 0.77) as u32;

    for i in 0..old_n {
        if !kh_iseither(old_flags, i) {
            let key = *old_keys.add(i as usize);
            let mask = (*h).n_buckets - 1;
            let mut site = __ac_FNV1a_hash_string(key.cast()) & mask;
            let mut step = 0;
            while !kh_isempty((*h).flags, site) {
                step += 1;
                site = (site + step) & mask;
            }
            *(*h).keys.add(site as usize) = key;
            *(*h).vals.add(site as usize) = *old_vals.add(i as usize);
            kh_set_occupied((*h).flags, site);
            (*h).size += 1;
            (*h).n_occupied += 1;
        }
    }

    if !old_flags.is_null() {
        drop(Vec::from_raw_parts(
            old_flags,
            old_n_flags as usize,
            old_n_flags as usize,
        ));
    }
    if !old_keys.is_null() {
        drop(Vec::from_raw_parts(
            old_keys,
            old_n as usize,
            old_n as usize,
        ));
    }
    if !old_vals.is_null() {
        drop(Vec::from_raw_parts(
            old_vals,
            old_n as usize,
            old_n as usize,
        ));
    }
    0
}

unsafe fn kh_put_s2i(h: *mut khash_s2i_t, key: *const u8, ret: *mut i32) -> u32 {
    if h.is_null() {
        *ret = -1;
        return 0;
    }
    if (*h).n_occupied >= (*h).upper_bound {
        let mut new_n = if (*h).n_buckets == 0 {
            4
        } else {
            (*h).n_buckets << 1
        };
        if new_n < 4 {
            new_n = 4;
        }
        if kh_resize_s2i(h, new_n) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }

    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key.cast()) & mask;
    let mut site = (*h).n_buckets;
    let last = i;
    let mut step = 0;
    while !kh_isempty((*h).flags, i) {
        if kh_isdel((*h).flags, i) {
            if site == (*h).n_buckets {
                site = i;
            }
        } else if cstr_eq(*(*h).keys.add(i as usize), key) {
            *ret = 0;
            return i;
        }
        step += 1;
        i = (i + step) & mask;
        if i == last {
            break;
        }
    }
    if site == (*h).n_buckets {
        site = i;
    }
    *(*h).keys.add(site as usize) = key.cast_mut();
    if kh_isempty((*h).flags, site) {
        (*h).n_occupied += 1;
        *ret = 1;
    } else {
        *ret = 2;
    }
    kh_set_occupied((*h).flags, site);
    (*h).size += 1;
    site
}

unsafe fn sam_hdr_set_long_target_len(
    h: *mut sam_hdr_t,
    name: *const u8,
    len: hts_pos_t,
) -> i32 {
    if (*h).sdict.is_null() {
        let sdict =
            Box::into_raw(Box::new(std::mem::zeroed::<khash_s2i_t>())).cast::<()>()
                .cast::<khash_s2i_t>();
        if sdict.is_null() {
            return -1;
        }
        if kh_resize_s2i(sdict, 4) < 0 {
            kh_destroy_s2i(sdict);
            return -1;
        }
        (*h).sdict = sdict.cast();
    }

    let long_refs = (*h).sdict.cast::<khash_s2i_t>();
    let mut ret = 0;
    let k = kh_put_s2i(long_refs, name, &mut ret);
    if ret < 0 {
        return -1;
    }
    *(*long_refs).vals.add(k as usize) = len;
    0
}

unsafe fn sam_hdr_append_target(h: *mut sam_hdr_t, name: &[u8], len: hts_pos_t) -> i32 {
    let new_n = (*h).n_targets + 1;
    if new_n <= 0 {
        *libc::__errno_location() =
            libc::EOVERFLOW as i32;
        return -1;
    }

    let old_n = (*h).n_targets as usize;
    let mut target_len_vec = if (*h).target_len.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*h).target_len, old_n, old_n)
    };
    target_len_vec.resize(new_n as usize, 0u32);
    (*h).target_len = target_len_vec.as_mut_ptr();
    std::mem::forget(target_len_vec);

    let mut target_name_vec = if (*h).target_name.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*h).target_name, old_n, old_n)
    };
    target_name_vec.resize(new_n as usize, std::ptr::null_mut::<u8>());
    (*h).target_name = target_name_vec.as_mut_ptr();
    std::mem::forget(target_name_vec);

    let dup = { let mut v: Vec<u8> = vec![0u8; name.len() + 1]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
    std::ptr::copy_nonoverlapping(name.as_ptr().cast::<u8>(), dup.cast(), (name.len()) as usize);
    *dup.add(name.len()) = 0;

    let idx = (*h).n_targets as usize;
    *(*h).target_name.add(idx) = dup;
    *(*h).target_len.add(idx) = if len >= u32::MAX as hts_pos_t {
        u32::MAX
    } else {
        len as u32
    };
    if len > u32::MAX as hts_pos_t && sam_hdr_set_long_target_len(h, dup, len) < 0 {
        {
            let mut n = 0usize;
            while *dup.add(n) != 0 {
                n += 1;
            }
            drop(Vec::from_raw_parts(dup, n + 1, n + 1));
        }
        *(*h).target_name.add(idx) = std::ptr::null_mut();
        return -1;
    }
    (*h).n_targets = new_n;
    0
}

unsafe fn sam_hdr_free_tmp_targets(tmp: *mut sam_hdr_t) {
    let n = (*tmp).n_targets as usize;
    for i in 0..n {
        let name = *(*tmp).target_name.add(i);
        if !name.is_null() {
            let mut l = 0usize;
            while *name.add(l) != 0 {
                l += 1;
            }
            drop(Vec::from_raw_parts(name, l + 1, l + 1));
        }
    }
    if !(*tmp).target_name.is_null() {
        drop(Vec::from_raw_parts((*tmp).target_name, n, n));
    }
    if !(*tmp).target_len.is_null() {
        drop(Vec::from_raw_parts((*tmp).target_len, n, n));
    }
    kh_destroy_s2i((*tmp).sdict.cast());
}

unsafe fn sam_hdr_clear_targets(h: *mut sam_hdr_t) {
    let n = (*h).n_targets as usize;
    for i in 0..n {
        let name = *(*h).target_name.add(i);
        if !name.is_null() {
            let mut l = 0usize;
            while *name.add(l) != 0 {
                l += 1;
            }
            drop(Vec::from_raw_parts(name, l + 1, l + 1));
        }
    }
    if !(*h).target_name.is_null() {
        drop(Vec::from_raw_parts((*h).target_name, n, n));
    }
    if !(*h).target_len.is_null() {
        drop(Vec::from_raw_parts((*h).target_len, n, n));
    }
    kh_destroy_s2i((*h).sdict.cast());
    (*h).target_name = std::ptr::null_mut();
    (*h).target_len = std::ptr::null_mut();
    (*h).sdict = std::ptr::null_mut();
    (*h).n_targets = 0;
}

pub(crate) unsafe fn sam_hdr_rebuild_targets_from_text(h: *mut sam_hdr_t) -> i32 {
    sam_hdr_clear_targets(h);
    sam_hdr_fill_targets_from_text(h)
}

pub(crate) fn sam_hdr_text_scratch() -> &'static Mutex<HashMap<usize, Vec<u8>>> {
    static SCRATCH: OnceLock<Mutex<HashMap<usize, Vec<u8>>>> = OnceLock::new();
    SCRATCH.get_or_init(|| Mutex::new(HashMap::new()))
}

// Registry of `sam_hdr_t` pointers that were allocated/owned by the C
// library (returned from `hts_sys::sam_hdr_read` for CRAM). Such headers
// must be released by `hts_sys::sam_hdr_destroy` so that C-pool memory is
// freed by the allocator that created it.
fn sam_hdr_c_owned_registry() -> &'static Mutex<HashSet<usize>> {
    static C_OWNED: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    C_OWNED.get_or_init(|| Mutex::new(HashSet::new()))
}

unsafe fn sam_hdr_mark_c_owned(h: *mut sam_hdr_t) {
    if h.is_null() {
        return;
    }
    if let Ok(mut set) = sam_hdr_c_owned_registry().lock() {
        set.insert(h as usize);
    }
}

unsafe fn sam_hdr_is_c_owned(h: *mut sam_hdr_t) -> bool {
    if h.is_null() {
        return false;
    }
    match sam_hdr_c_owned_registry().lock() {
        Ok(set) => set.contains(&(h as usize)),
        Err(_) => false,
    }
}

// Registry of `sam_hdr_t` pointers whose `hrecs` field points at a Rust-built
// sam_hrecs_t. This lets us distinguish from headers whose hrecs was filled
// by the C library (e.g. via a stray `hts_sys::sam_hdr_add_*` call in a test
// helper) — those still have a non-null hrecs, but the memory inside lives
// in C string/type pools and cannot be walked or freed from our side. Used
// by sam_hdr_write/sam_hdr_str to dispatch correctly.
fn sam_hdr_rust_hrecs_registry() -> &'static Mutex<HashSet<usize>> {
    static RUST_HRECS: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    RUST_HRECS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub(crate) unsafe fn sam_hdr_mark_rust_hrecs(h: *mut sam_hdr_t) {
    if h.is_null() {
        return;
    }
    if let Ok(mut set) = sam_hdr_rust_hrecs_registry().lock() {
        set.insert(h as usize);
    }
}

pub(crate) unsafe fn sam_hdr_has_rust_hrecs(h: *mut sam_hdr_t) -> bool {
    if h.is_null() {
        return false;
    }
    match sam_hdr_rust_hrecs_registry().lock() {
        Ok(set) => set.contains(&(h as usize)),
        Err(_) => false,
    }
}

pub(crate) unsafe fn sam_hdr_forget_rust_hrecs(h: *mut sam_hdr_t) {
    if let Ok(mut set) = sam_hdr_rust_hrecs_registry().lock() {
        set.remove(&(h as usize));
    }
}

unsafe fn sam_hdr_forget_c_owned(h: *mut sam_hdr_t) {
    if let Ok(mut set) = sam_hdr_c_owned_registry().lock() {
        set.remove(&(h as usize));
    }
}

unsafe fn sam_hdr_restore_text_len(h: *mut sam_hdr_t, old_len: usize) {
    let prev_len = (*h).l_text;
    (*h).l_text = old_len;
    if old_len == 0 {
        if !(*h).text.is_null() {
            drop(Vec::from_raw_parts((*h).text, prev_len + 1, prev_len + 1));
        }
        (*h).text = std::ptr::null_mut();
    } else if !(*h).text.is_null() {
        *(*h).text.add(old_len) = 0;
    }
}

pub(crate) unsafe fn sam_hdr_fill_targets_from_text(h: *mut sam_hdr_t) -> i32 {
    if (*h).text.is_null() || (*h).l_text == 0 {
        return 0;
    }
    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    for raw in text.split(|&b| b == b'\n') {
        let line = raw.strip_suffix(b"\r").unwrap_or(raw);
        if line.is_empty() {
            continue;
        }
        if line.starts_with(b"@SQ\t") {
            let Some((name, len)) = parse_sq_target(line) else {
                return -1;
            };
            if sam_hdr_append_target(h, name, len) < 0 {
                return -1;
            }
        }
    }
    0
}

// Append `len` bytes of header text to `(*h).text` without populating
// hrecs. Used only by the initial parse path (sam_c_1907_sam_hdr_create) so
// the header can stay text-only until a real mutation lands; that way the
// first mutator — Rust or hts_sys — gets to fill hrecs with records its
// own allocators can later read/free. Returns 0 on success, -1 on overflow
// or allocation failure.
unsafe fn sam_hdr_append_text_raw(h: *mut sam_hdr_t, src: *const u8, len: usize) -> i32 {
    if h.is_null() || src.is_null() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    let old_len = (*h).l_text;
    let new_len = match old_len.checked_add(len) {
        Some(v) => v,
        None => {
            *libc::__errno_location() =
                libc::EOVERFLOW as i32;
            return -1;
        }
    };
    let mut text_vec = if (*h).text.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*h).text, old_len + 1, old_len + 1)
    };
    text_vec.resize(new_len + 1, 0);
    let text = text_vec.as_mut_ptr();
    std::mem::forget(text_vec);
    (*h).text = text;
    std::ptr::copy_nonoverlapping(src.cast::<u8>(), (*h).text.add(old_len).cast(), (len) as usize);
    (*h).l_text = new_len;
    *(*h).text.add(new_len) = 0;
    0
}

// hrecs-backed path for sam_hdr_add_line (mirrors htslib/header.c:1693, which
// always fills and mutates hrecs). Inputs are pre-validated by the caller.
pub(crate) unsafe fn sam_hdr_add_line_hrecs(
    h: *mut sam_hdr_t,
    type_: *const u8,
    tags: &[(*const u8, *const u8)],
) -> i32 {
    let hrecs = (*h).hrecs;
    let ret = sam_hrecs_vadd(hrecs, type_, tags);
    if ret != 0 {
        return ret;
    }
    // An added @SQ line changes the reference set; rebuild the target arrays so
    // n_targets/target_name stay in sync (matches sam_hdr_update_line).
    if *type_ == b'S' as u8 && *type_.add(1) == b'Q' as u8 && rebuild_target_arrays(h) < 0 {
        return -1;
    }
    // sam_hrecs_vadd sets dirty; clear the cached text so the next read/write
    // rebuilds from hrecs (matches the redact_header_text call in htslib C's
    // sam_hdr_add_line).
    redact_header_text(&mut *h);
    0
}

pub(crate) unsafe fn sam_hdr_text_remove_line(
    h: *mut sam_hdr_t,
    type0: u8,
    type1: u8,
    unchanged_ret: i32,
    mut should_remove: impl FnMut(&[u8], i32) -> bool,
) -> i32 {
    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut out = Vec::with_capacity(text.len());
    let mut changed = false;
    let mut seen = 0;
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let end = start + rel_end;
        let line = &text[start..end];
        let line_no_cr = line.strip_suffix(b"\r").unwrap_or(line);
        let has_newline = end < text.len();
        let matching_type = sam_hdr_text_line_has_type(line_no_cr, type0, type1);
        let remove = matching_type && should_remove(line_no_cr, seen);
        if matching_type {
            seen += 1;
        }
        if remove {
            changed = true;
        } else {
            out.extend_from_slice(&text[start..end]);
            if has_newline {
                out.push(b'\n');
            }
        }
        start = end + usize::from(has_newline);
    }

    if !changed {
        return unchanged_ret;
    }
    let new_len = out.len();
    let mut new_vec = out;
    new_vec.push(0);
    let new_text = new_vec.as_mut_ptr();
    std::mem::forget(new_vec);
    if !(*h).text.is_null() {
        drop(Vec::from_raw_parts(
            (*h).text,
            (*h).l_text + 1,
            (*h).l_text + 1,
        ));
    }
    (*h).text = new_text;
    (*h).l_text = new_len;
    if type0 == b'S' && type1 == b'Q' && sam_hdr_rebuild_targets_from_text(h) < 0 {
        return -1;
    }
    0
}

pub(crate) unsafe fn sam_hdr_text_remove_tag_id(
    h: *mut sam_hdr_t,
    type0: u8,
    type1: u8,
    id: Option<(&[u8], &[u8])>,
    key: &[u8],
) -> i32 {
    if key.is_empty() {
        return -1;
    }

    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut out = Vec::with_capacity(text.len());
    let mut changed = false;
    let mut matched_line = false;
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let end = start + rel_end;
        let line = &text[start..end];
        let line_no_cr = line.strip_suffix(b"\r").unwrap_or(line);
        let has_newline = end < text.len();
        let mut wrote_replacement = false;

        if !matched_line && sam_hdr_text_line_has_type(line_no_cr, type0, type1) {
            let id_matches = id.is_none_or(|(id_key, id_value)| {
                sam_hdr_text_find_tag_value(line_no_cr, id_key) == Some(id_value)
            });
            if id_matches {
                matched_line = true;
                if let Some(replacement) = sam_hdr_text_line_without_tag(line_no_cr, key) {
                    out.extend_from_slice(&replacement);
                    if line.ends_with(b"\r") {
                        out.push(b'\r');
                    }
                    if has_newline {
                        out.push(b'\n');
                    }
                    changed = true;
                    wrote_replacement = true;
                }
            }
        }

        if !wrote_replacement {
            out.extend_from_slice(&text[start..end]);
            if has_newline {
                out.push(b'\n');
            }
        }
        start = end + usize::from(has_newline);
    }

    if !changed {
        return -1;
    }
    let new_len = out.len();
    let mut new_vec = out;
    new_vec.push(0);
    let new_text = new_vec.as_mut_ptr();
    std::mem::forget(new_vec);
    if !(*h).text.is_null() {
        drop(Vec::from_raw_parts(
            (*h).text,
            (*h).l_text + 1,
            (*h).l_text + 1,
        ));
    }
    (*h).text = new_text;
    (*h).l_text = new_len;
    if type0 == b'S'
        && type1 == b'Q'
        && (key == b"SN" || key == b"LN")
        && sam_hdr_rebuild_targets_from_text(h) < 0
    {
        return -1;
    }
    0
}

fn sam_hdr_text_line_without_tag(line: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    if line.len() <= 3 {
        return None;
    }

    let mut out = Vec::with_capacity(line.len());
    out.extend_from_slice(&line[..3]);
    let mut removed = false;
    for field in line[3..].split(|&b| b == b'\t').filter(|f| !f.is_empty()) {
        let remove = field.len() > key.len()
            && field.starts_with(key)
            && field.get(key.len()) == Some(&b':');
        if remove {
            removed = true;
        } else {
            out.push(b'\t');
            out.extend_from_slice(field);
        }
    }

    removed.then_some(out)
}

pub(crate) unsafe fn sam_hdr_text_find_line_pos<'a>(
    h: *const sam_hdr_t,
    type_: *const u8,
    pos: i32,
) -> Option<&'a [u8]> {
    if pos < 0 || h.is_null() || type_.is_null() || (*h).text.is_null() {
        return None;
    }

    let type0 = *type_ as u8;
    let type1 = *type_.add(1) as u8;
    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut seen = 0;
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let mut line = &text[start..start + rel_end];
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }
        if sam_hdr_text_line_has_type(line, type0, type1) {
            if seen == pos {
                return Some(line);
            }
            seen += 1;
        }
        start += rel_end + usize::from(start + rel_end < text.len());
    }
    None
}

pub(crate) unsafe fn sam_hdr_text_find_line_id<'a>(
    h: *const sam_hdr_t,
    type_: *const u8,
    id_key: &[u8],
    id_val: &[u8],
) -> Option<&'a [u8]> {
    if id_key.is_empty() || h.is_null() || type_.is_null() || (*h).text.is_null() {
        return None;
    }

    let type0 = *type_ as u8;
    let type1 = *type_.add(1) as u8;
    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let mut line = &text[start..start + rel_end];
        if line.last() == Some(&b'\r') {
            line = &line[..line.len() - 1];
        }
        if sam_hdr_text_line_has_type(line, type0, type1)
            && sam_hdr_text_find_tag_value(line, id_key) == Some(id_val)
        {
            return Some(line);
        }
        start += rel_end + usize::from(start + rel_end < text.len());
    }
    None
}

pub(crate) fn sam_hdr_text_line_has_type(line: &[u8], type0: u8, type1: u8) -> bool {
    line.len() >= 3
        && line[0] == b'@'
        && line[1] == type0
        && line[2] == type1
        && (line.len() == 3 || line[3] == b'\t')
}

pub(crate) fn sam_hdr_text_find_tag_value<'a>(line: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    if key.is_empty() || line.len() <= 3 {
        return None;
    }
    for field in line[3..].split(|&b| b == b'\t').filter(|f| !f.is_empty()) {
        if field.len() > key.len() && field.starts_with(key) && field.get(key.len()) == Some(&b':')
        {
            return Some(&field[key.len() + 1..]);
        }
    }
    None
}

pub(crate) unsafe fn sam_hdr_text_pg_id_exists(h: *const sam_hdr_t, id: &[u8]) -> bool {
    if h.is_null() || (*h).text.is_null() {
        return false;
    }

    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let end = start + rel_end;
        let line = text[start..end]
            .strip_suffix(b"\r")
            .unwrap_or(&text[start..end]);
        if sam_hdr_text_line_has_type(line, b'P', b'G')
            && sam_hdr_text_find_tag_value(line, b"ID") == Some(id)
        {
            return true;
        }
        start = end + usize::from(end < text.len());
    }
    false
}

unsafe fn sam_hdr_text_pg_rows(h: *const sam_hdr_t) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
    let mut rows = Vec::new();
    if h.is_null() || (*h).text.is_null() {
        return rows;
    }

    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let end = start + rel_end;
        let line = text[start..end]
            .strip_suffix(b"\r")
            .unwrap_or(&text[start..end]);
        if sam_hdr_text_line_has_type(line, b'P', b'G') {
            if let Some(id) = sam_hdr_text_find_tag_value(line, b"ID") {
                rows.push((
                    id.to_vec(),
                    sam_hdr_text_find_tag_value(line, b"PP").map(|pp| pp.to_vec()),
                ));
            }
        }
        start = end + usize::from(end < text.len());
    }
    rows
}

unsafe fn sam_hdr_text_pg_leaf_ids(h: *const sam_hdr_t) -> Vec<Vec<u8>> {
    let rows = sam_hdr_text_pg_rows(h);
    let mut referenced = Vec::<Vec<u8>>::new();
    for (id, pp) in &rows {
        if let Some(pp) = pp {
            if pp != id && !referenced.iter().any(|seen| seen == pp) {
                referenced.push(pp.clone());
            }
        }
    }

    let mut leaves = Vec::new();
    for (id, _) in rows {
        if !referenced.iter().any(|seen| seen == &id) {
            leaves.push(id);
        }
    }
    leaves
}

unsafe fn sam_hdr_text_append_bytes(h: *mut sam_hdr_t, bytes: &[u8]) -> i32 {
    let new_len = match (*h).l_text.checked_add(bytes.len()) {
        Some(v) => v,
        None => {
            *libc::__errno_location() =
                libc::EOVERFLOW as i32;
            return -1;
        }
    };
    let old_len = (*h).l_text;
    let mut text_vec = if (*h).text.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*h).text, old_len + 1, old_len + 1)
    };
    text_vec.resize(new_len + 1, 0);
    (*h).text = text_vec.as_mut_ptr();
    std::mem::forget(text_vec);
    if !bytes.is_empty() {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<u8>(), (*h).text.add((*h).l_text).cast(), (bytes.len()) as usize);
    }
    (*h).l_text = new_len;
    *(*h).text.add(new_len) = 0;
    0
}

unsafe fn sam_hdr_text_add_pg_line(
    h: *mut sam_hdr_t,
    id: &[u8],
    name: &[u8],
    tags: &[(&[u8], &[u8])],
    pp: Option<&[u8]>,
) -> i32 {
    let mut line = Vec::new();
    line.extend_from_slice(b"@PG");
    let has_id_tag = tags.iter().any(|(key, _)| *key == b"ID");
    if has_id_tag {
        if !tags.iter().any(|(key, _)| *key == b"PN") {
            line.extend_from_slice(b"\tPN:");
            line.extend_from_slice(name);
        }
    } else {
        line.extend_from_slice(b"\tID:");
        line.extend_from_slice(id);
        line.extend_from_slice(b"\tPN:");
        line.extend_from_slice(name);
    }
    for (key, value) in tags {
        line.push(b'\t');
        line.extend_from_slice(key);
        line.push(b':');
        line.extend_from_slice(value);
    }
    if let Some(pp) = pp {
        line.extend_from_slice(b"\tPP:");
        line.extend_from_slice(pp);
    }
    line.push(b'\n');
    sam_hdr_text_append_bytes(h, &line)
}

// Generates a unique @PG ID against the hrecs pg_hash, mirroring
// htslib/header.c:sam_hdr_pg_id. Returns a NUL-terminated byte vector. The
// suffix counter hrecs->ID_cnt persists across calls (so the first generated
// fallback is "name.0"). Reads our own pg_hash rather than delegating to the C
// library, whose kh_get cannot index a Rust-built hash table.
unsafe fn sam_hrecs_unique_pg_id(hrecs: *mut sam_hrecs_t, name: *const u8) -> Option<Vec<u8>> {
    let name_bytes = CStr::from_ptr(name.cast()).to_bytes();
    if (*hrecs).pg_hash.is_null() || sam_hrecs_hash_value((*hrecs).pg_hash, name).is_none() {
        let mut v = name_bytes.to_vec();
        v.push(0);
        return Some(v);
    }
    let truncated = if name_bytes.len() > 1000 {
        &name_bytes[..1000]
    } else {
        name_bytes
    };
    loop {
        let mut cand = truncated.to_vec();
        cand.push(b'.');
        cand.extend_from_slice((*hrecs).ID_cnt.to_string().as_bytes());
        (*hrecs).ID_cnt += 1;
        cand.push(0);
        if sam_hrecs_hash_value((*hrecs).pg_hash, cand.as_ptr().cast()).is_none() {
            return Some(cand);
        }
    }
}

// Returns the ID to use for the next @PG record: a fresh unique ID (as a
// NUL-terminated byte vector) when none was specified, otherwise a single NUL
// byte (empty) so the user's own ID tag is used instead.
unsafe fn sam_hdr_pg_gen_id(
    hrecs: *mut sam_hrecs_t,
    name: *const u8,
    specified_id: *const u8,
) -> Option<Vec<u8>> {
    if specified_id.is_null() {
        sam_hrecs_unique_pg_id(hrecs, name)
    } else {
        Some(vec![0])
    }
}

// Adds one @PG line to hrecs with the generated `id` (omitted when empty),
// `pn` (omitted when empty), and `pp` (omitted when null/empty), followed by
// the caller's `user_tags`. Mirrors a single sam_hrecs_vadd("PG", ...) call in
// htslib/header.c:sam_hdr_add_pg, where the computed ID/PN/PP prefix is added
// ahead of the user-supplied tags and empty-value prefix pairs are skipped.
unsafe fn sam_hrecs_pg_add_one(
    hrecs: *mut sam_hrecs_t,
    id: *const u8,
    pn: *const u8,
    pp: *const u8,
    user_tags: &[(*const u8, *const u8)],
) -> i32 {
    let mut combined: Vec<(*const u8, *const u8)> = Vec::new();
    if !id.is_null() && *id != 0 {
        combined.push((c"ID".as_ptr().cast(), id));
    }
    if !pn.is_null() && *pn != 0 {
        combined.push((c"PN".as_ptr().cast(), pn));
    }
    if !pp.is_null() && *pp != 0 {
        combined.push((c"PP".as_ptr().cast(), pp));
    }
    combined.extend_from_slice(user_tags);
    sam_hrecs_vadd(hrecs, c"PG".as_ptr().cast(), &combined)
}

// hrecs-backed path for sam_hdr_add_pg (mirrors htslib/header.c:2614, which
// always operates on hrecs). Generates a suitable ID when unspecified, chains
// new records onto each existing @PG leaf, and validates ID/PP references.
pub(crate) unsafe fn sam_hdr_add_pg_hrecs(
    h: *mut sam_hdr_t,
    name: *const u8,
    tags: &[(*const u8, *const u8)],
) -> i32 {
    let hrecs = (*h).hrecs;
    (*hrecs).pgs_changed = 1;
    if sam_hdr_link_pg(h) < 0 {
        return -1;
    }
    let hrecs = (*h).hrecs;

    // Find ID / PN / PP in the supplied tags (non-empty values; last wins).
    let mut specified_id: *const u8 = std::ptr::null();
    let mut specified_pn: *const u8 = std::ptr::null();
    let mut specified_pp: *const u8 = std::ptr::null();
    for &(key, value) in tags {
        if key.is_null() || value.is_null() {
            break;
        }
        let k = CStr::from_ptr(key.cast()).to_bytes();
        if CStr::from_ptr(value.cast()).to_bytes().is_empty() {
            continue;
        }
        if k == b"PN" {
            specified_pn = value;
        } else if k == b"PP" {
            specified_pp = value;
        } else if k == b"ID" {
            specified_id = value;
        }
    }

    if !specified_id.is_null()
        && !(*hrecs).pg_hash.is_null()
        && sam_hrecs_hash_value((*hrecs).pg_hash, specified_id).is_some()
    {
        return -1;
    }
    if !specified_pp.is_null()
        && ((*hrecs).pg_hash.is_null()
            || sam_hrecs_hash_value((*hrecs).pg_hash, specified_pp).is_none())
    {
        return -1;
    }

    let pn: *const u8 = if specified_pn.is_null() {
        name
    } else {
        c"".as_ptr().cast()
    };

    if specified_pp.is_null() && (*hrecs).npg_end > 0 {
        // Snapshot the leaf PP names up front: each sam_hrecs_vadd rebuilds the
        // pg array, so we must not hold raw pointers into it across iterations.
        let nends = (*hrecs).npg_end;
        let mut leaves: Vec<Vec<u8>> = Vec::with_capacity(nends as usize);
        for i in 0..nends {
            let end = *(*hrecs).pg_end.add(i as usize);
            if end < 0 || end >= (*hrecs).npg || (*hrecs).pg.is_null() {
                return -1;
            }
            let pg = (*hrecs).pg.cast::<sam_hrec_pg_t>().add(end as usize);
            if (*pg).name.is_null() {
                return -1;
            }
            let mut v = CStr::from_ptr((*pg).name.cast()).to_bytes().to_vec();
            v.push(0);
            leaves.push(v);
        }
        for leaf in &leaves {
            let Some(id) = sam_hdr_pg_gen_id(hrecs, name, specified_id) else {
                return -1;
            };
            if sam_hrecs_pg_add_one(hrecs, id.as_ptr().cast(), pn, leaf.as_ptr().cast(), tags) < 0 {
                return -1;
            }
        }
    } else {
        let Some(id) = sam_hdr_pg_gen_id(hrecs, name, specified_id) else {
            return -1;
        };
        if sam_hrecs_pg_add_one(
            hrecs,
            id.as_ptr().cast(),
            pn,
            std::ptr::null::<u8>(),
            tags,
        ) < 0
        {
            return -1;
        }
    }

    (*hrecs).dirty = 1;
    0
}

fn sam_hdr_text_an_contains(value: &[u8], needle: &[u8]) -> bool {
    value.split(|&b| b == b',').any(|part| part == needle)
}

pub(crate) unsafe fn sam_hdr_text_name_key_for_type(type_: *const u8) -> Option<&'static [u8]> {
    if type_.is_null() {
        return None;
    }
    match (*type_ as u8, *type_.add(1) as u8) {
        (b'S', b'Q') => Some(b"SN"),
        (b'R', b'G') | (b'P', b'G') => Some(b"ID"),
        _ => None,
    }
}

pub(crate) unsafe fn sam_hdr_text_name2tid(h: *const sam_hdr_t, ref_: *const u8) -> i32 {
    if h.is_null() || ref_.is_null() || (*h).text.is_null() {
        return -1;
    }
    let needle = CStr::from_ptr(ref_.cast()).to_bytes();
    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut tid = 0;
    let mut start = 0usize;
    while start < text.len() {
        let rel_end = text[start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(text.len() - start);
        let end = start + rel_end;
        let line = text[start..end]
            .strip_suffix(b"\r")
            .unwrap_or(&text[start..end]);
        if sam_hdr_text_line_has_type(line, b'S', b'Q') {
            if sam_hdr_text_find_tag_value(line, b"SN") == Some(needle)
                || sam_hdr_text_find_tag_value(line, b"AN")
                    .is_some_and(|value| sam_hdr_text_an_contains(value, needle))
            {
                return tid;
            }
            tid += 1;
        }
        start = end + usize::from(end < text.len());
    }
    -1
}

// original: sam_hdr_link_pg (htslib/header.c:2468)
pub(crate) unsafe fn sam_hdr_link_pg(h: *mut sam_hdr_t) -> i32 {
    if h.is_null() {
        return -1;
    }
    if (*h).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *h) < 0 {
        return -1;
    }
    let hrecs = (*h).hrecs;
    if (*hrecs).pgs_changed == 0 || (*hrecs).npg == 0 {
        return 0;
    }

    let old_pg_end_alloc = (*hrecs).npg_end_alloc as usize;
    let mut pg_end_vec = if (*hrecs).pg_end.is_null() {
        Vec::new()
    } else {
        Vec::from_raw_parts((*hrecs).pg_end, old_pg_end_alloc, old_pg_end_alloc)
    };
    pg_end_vec.resize((*hrecs).npg as usize, 0i32);
    (*hrecs).pg_end = pg_end_vec.as_mut_ptr();
    std::mem::forget(pg_end_vec);
    (*hrecs).npg_end_alloc = (*hrecs).npg;

    let mut chain_size = vec![0i32; (*hrecs).npg as usize];
    for i in 0..(*hrecs).npg {
        *(*hrecs).pg_end.add(i as usize) = i;
        let pg = (*hrecs).pg.cast::<sam_hrec_pg_t>().add(i as usize);
        (*pg).prev_id = -1;
    }

    for i in 0..(*hrecs).npg {
        let pg = (*hrecs).pg.cast::<sam_hrec_pg_t>().add(i as usize);
        let line = (*pg).ty.cast::<sam_hrec_type_t>();
        let pp = sam_hrec_tag_value_cstr(line, b"PP");
        if pp.is_null() {
            continue;
        }
        let Some(pp_idx) = sam_hrecs_hash_value((*hrecs).pg_hash, pp) else {
            continue;
        };
        if pp_idx == i || pp_idx < 0 || pp_idx >= (*hrecs).npg {
            continue;
        }
        let prev = (*hrecs).pg.cast::<sam_hrec_pg_t>().add(pp_idx as usize);
        (*pg).prev_id = (*prev).id;
        *(*hrecs).pg_end.add(pp_idx as usize) = -1;
        chain_size[i as usize] = chain_size[pp_idx as usize] + 1;
    }

    let mut last_end = -1;
    let mut n_end = 0;
    for i in 0..(*hrecs).npg {
        let end = *(*hrecs).pg_end.add(i as usize);
        if end != -1 {
            last_end = end;
            if chain_size[i as usize] > 0 {
                *(*hrecs).pg_end.add(n_end as usize) = end;
                n_end += 1;
            }
        }
    }
    if n_end == 0 && (*hrecs).npg_end > 0 && last_end >= 0 {
        *(*hrecs).pg_end = last_end;
        n_end = 1;
    }
    (*hrecs).npg_end = n_end;
    (*hrecs).pgs_changed = 0;
    (*hrecs).dirty = 1;
    0
}

// original: KHASH_DECLARE (htslib/header.c:44)
// Translated by the concrete khash_s2i_t declarations and helpers in this file.

// original: TYPEKEY (htslib/header.h:58)
pub unsafe fn header_h_58_TYPEKEY(type_: *const u8) -> u32 {
    let u0 = *type_ as u8 as u32;
    let u1 = *type_.add(1) as u8 as u32;
    (u0 << 8) | u1
}

pub(crate) unsafe fn sam_hrecs_walk_global(
    hrecs: *mut sam_hrecs_t,
    mut visit: impl FnMut(*mut sam_hrec_type_t) -> bool,
) {
    if hrecs.is_null() || (*hrecs).first_line.is_null() {
        return;
    }

    let first = (*hrecs).first_line.cast::<sam_hrec_type_t>();
    let mut line = first;
    loop {
        if !visit(line) {
            break;
        }
        let next = (*line).global_next;
        if next.is_null() || next == first {
            break;
        }
        line = next;
    }
}

pub(crate) unsafe fn sam_hrecs_find_first_type(
    hrecs: *mut sam_hrecs_t,
    type_: u32,
) -> *mut sam_hrec_type_t {
    if hrecs.is_null() {
        return std::ptr::null_mut();
    }

    let mut found = std::ptr::null_mut();
    sam_hrecs_walk_global(hrecs, |line| {
        if (*line).type_ == type_ {
            found = line;
            return false;
        }
        true
    });
    found
}

pub(crate) unsafe fn sam_hrec_find_tag_value<'a>(
    line: *mut sam_hrec_type_t,
    key0: u8,
    key1: u8,
) -> Option<&'a [u8]> {
    let mut tag = (*line).tag;
    while !tag.is_null() {
        if !(*tag).str_.is_null()
            && (*tag).len >= 3
            && *(*tag).str_.cast::<u8>() == key0
            && *(*tag).str_.cast::<u8>().add(1) == key1
        {
            let len = (*tag).len as usize;
            let bytes = std::slice::from_raw_parts((*tag).str_.cast::<u8>(), len);
            return Some(&bytes[3..]);
        }
        tag = (*tag).next;
    }
    None
}

pub(crate) unsafe fn sam_hrec_tag_matches_key(
    tag: *mut sam_hrec_tag_t,
    key: *const u8,
) -> bool {
    !tag.is_null()
        && !key.is_null()
        && !(*tag).str_.is_null()
        && (*tag).len >= 2
        && *(*tag).str_ == *key
        && *(*tag).str_.add(1) == *key.add(1)
}

unsafe fn sam_hrecs_strdup_bytes(bytes: &[u8]) -> *mut u8 {
    let s = { let mut v: Vec<u8> = vec![0u8; (bytes.len() as u64 + 1) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
    if s.is_null() {
        return std::ptr::null_mut();
    }
    if !bytes.is_empty() {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<u8>(), s.cast(), (bytes.len()) as usize);
    }
    *s.add(bytes.len()) = 0;
    s
}

unsafe fn sam_hrecs_free_tags(mut tag: *mut sam_hrec_tag_t) {
    while !tag.is_null() {
        let next = (*tag).next;
        if !(*tag).str_.is_null() {
            let cap = (*tag).len as usize + 1;
            drop(Vec::from_raw_parts((*tag).str_.cast_mut(), cap, cap));
        }
        drop(Box::from_raw(tag));
        tag = next;
    }
}

// original: sam_hrecs_global_list_add (htslib/header.c:216)
// Mirrors htslib/header.c:216's sam_hrecs_global_list_add. `after = NULL`
// means "append at the end" (with @HD special-cased to the top if no @HD
// already exists); a non-NULL `after` inserts the new line directly after it.
//
// Also splices `line` into the per-type ring (next/prev fields). The ring
// invariant — each line links to the next/prev line of the same type, with
// every type's ring closing on itself — is what
// header_c_704_sam_hrecs_remove_line walks via (*cur).next; uninitialized
// next/prev fields produce a SIGSEGV during type-wide removes.
unsafe fn sam_hrecs_global_list_add(
    hrecs: *mut sam_hrecs_t,
    line: *mut sam_hrec_type_t,
    after: *mut sam_hrec_type_t,
) -> i32 {
    if hrecs.is_null() || line.is_null() {
        return -1;
    }
    if (*hrecs).first_line.is_null() {
        (*line).global_next = line;
        (*line).global_prev = line;
        (*line).next = line;
        (*line).prev = line;
        (*hrecs).first_line = line.cast();
        (*hrecs).dirty = 1;
        return 0;
    }

    let first = (*hrecs).first_line.cast::<sam_hrec_type_t>();
    let hd_key = header_h_58_TYPEKEY(c"HD".as_ptr().cast());
    let mut anchor = after;
    let mut update_first_line = false;
    // @HD floats to the top (unless an @HD already exists there).
    if (*line).type_ == hd_key && (*first).type_ != hd_key {
        anchor = (*first).global_prev;
        update_first_line = true;
    }
    if anchor.is_null() {
        anchor = (*first).global_prev;
    }
    let next = (*anchor).global_next;
    (*line).global_prev = anchor;
    (*line).global_next = next;
    (*anchor).global_next = line;
    (*next).global_prev = line;
    if update_first_line {
        (*hrecs).first_line = line.cast();
    }

    // Per-type ring: find any existing line of the same type and splice in
    // at its tail. If no existing record of this type, the ring is self.
    let same_type = sam_hrecs_find_last_of_type_excluding(hrecs, (*line).type_, line);
    if same_type.is_null() {
        (*line).next = line;
        (*line).prev = line;
    } else {
        let ring_next = (*same_type).next;
        (*line).prev = same_type;
        (*line).next = ring_next;
        (*same_type).next = line;
        if !ring_next.is_null() {
            (*ring_next).prev = line;
        } else {
            // Shouldn't happen if invariants hold, but recover defensively.
            (*line).next = line;
            (*line).prev = same_type;
            (*same_type).next = line;
        }
    }

    (*hrecs).dirty = 1;
    0
}

// Walk the global list and return the last record of `type_`, skipping
// `exclude` (the newly-added line itself). Returns null if none exists.
unsafe fn sam_hrecs_find_last_of_type_excluding(
    hrecs: *mut sam_hrecs_t,
    type_: u32,
    exclude: *mut sam_hrec_type_t,
) -> *mut sam_hrec_type_t {
    let mut found: *mut sam_hrec_type_t = std::ptr::null_mut();
    sam_hrecs_walk_global(hrecs, |cur| {
        if cur != exclude && (*cur).type_ == type_ {
            found = cur;
        }
        true
    });
    found
}

unsafe fn sam_hrecs_type_rank(hrecs: *mut sam_hrecs_t, type_: u32) -> i32 {
    if hrecs.is_null() || (*hrecs).type_order.is_null() {
        return i32::MAX;
    }
    for i in 0..(*hrecs).type_count {
        let ty = (*hrecs).type_order.add(i as usize);
        if ((*ty)[0] as u32) << 8 | ((*ty)[1] as u8 as u32) == type_ {
            return i;
        }
    }
    i32::MAX
}

// Locate the last line of `type_` in the global list (or null if none). Used
// to mimic C's per-type-ring "h_type->prev" insertion anchor inside
// sam_hrecs_vadd.
unsafe fn sam_hrecs_find_last_of_type(hrecs: *mut sam_hrecs_t, type_: u32) -> *mut sam_hrec_type_t {
    let mut found: *mut sam_hrec_type_t = std::ptr::null_mut();
    sam_hrecs_walk_global(hrecs, |cur| {
        if (*cur).type_ == type_ {
            found = cur;
        }
        true
    });
    found
}

unsafe fn sam_hrecs_type_list_add(hrecs: *mut sam_hrecs_t, line: *mut sam_hrec_type_t) {
    // Match C: append immediately after the last existing line of the same
    // type, or at the global end if none exists yet (with the @HD-at-top
    // special case handled by sam_hrecs_global_list_add).
    let after = sam_hrecs_find_last_of_type(hrecs, (*line).type_);
    let _ = sam_hrecs_global_list_add(hrecs, line, after);
}

// original: sam_hrecs_remove_line (htslib/header.c:250)
pub(crate) unsafe fn sam_hrecs_remove_line(
    hrecs: *mut sam_hrecs_t,
    line: *mut sam_hrec_type_t,
) -> i32 {
    if hrecs.is_null() || line.is_null() || (*hrecs).first_line.is_null() {
        return -1;
    }
    if (*line).global_next == line || (*line).global_next.is_null() {
        (*hrecs).first_line = std::ptr::null_mut();
    } else {
        (*(*line).global_prev).global_next = (*line).global_next;
        (*(*line).global_next).global_prev = (*line).global_prev;
        if (*hrecs).first_line == line.cast() {
            (*hrecs).first_line = (*line).global_next.cast();
        }
    }
    sam_hrecs_free_tags((*line).tag);
    drop(Box::from_raw(line));
    (*hrecs).dirty = 1;
    0
}

// original: sam_hrecs_init_type_order (htslib/header.c:69)
pub(crate) unsafe fn sam_hrecs_init_type_order(
    hrecs: *mut sam_hrecs_t,
    type_list: *mut u8,
) -> i32 {
    if hrecs.is_null() {
        return -1;
    }
    if !type_list.is_null() {
        return 0;
    }

    (*hrecs).type_count = 5;
    let type_order = {
        let mut v: Vec<[u8; 3]> = vec![[0u8; 3]; 5];
        let p = v.as_mut_ptr();
        std::mem::forget(v);
        p
    };
    let defaults = [b"HD\0", b"SQ\0", b"RG\0", b"PG\0", b"CO\0"];
    for (i, value) in defaults.iter().enumerate() {
        (*type_order.add(i))[0] = value[0] as u8;
        (*type_order.add(i))[1] = value[1] as u8;
        (*type_order.add(i))[2] = 0;
    }
    (*hrecs).type_order = type_order;
    0
}

// original: sam_hrecs_find_type_pos (htslib/header.c:1546)
unsafe fn sam_hrecs_find_type_pos(
    hrecs: *mut sam_hrecs_t,
    type_: *const u8,
    idx: i32,
) -> *mut sam_hrec_type_t {
    if hrecs.is_null() || type_.is_null() || idx < 0 {
        return std::ptr::null_mut();
    }

    if *type_ == b'S' as u8 && *type_.add(1) == b'Q' as u8 {
        return if idx < (*hrecs).nref && !(*hrecs).ref_.is_null() {
            (*(*hrecs).ref_.add(idx as usize)).ty.cast()
        } else {
            std::ptr::null_mut()
        };
    }
    if *type_ == b'R' as u8 && *type_.add(1) == b'G' as u8 {
        return if idx < (*hrecs).nrg && !(*hrecs).rg.is_null() {
            (*(*hrecs).rg.cast::<sam_hrec_rg_t>().add(idx as usize))
                .ty
                .cast()
        } else {
            std::ptr::null_mut()
        };
    }
    if *type_ == b'P' as u8 && *type_.add(1) == b'G' as u8 {
        return if idx < (*hrecs).npg && !(*hrecs).pg.is_null() {
            (*(*hrecs).pg.cast::<sam_hrec_pg_t>().add(idx as usize))
                .ty
                .cast()
        } else {
            std::ptr::null_mut()
        };
    }

    let type_key = header_h_58_TYPEKEY(type_);
    let mut seen = 0;
    let mut found = std::ptr::null_mut();
    sam_hrecs_walk_global(hrecs, |line| {
        if (*line).type_ == type_key {
            if seen == idx {
                found = line;
                return false;
            }
            seen += 1;
        }
        true
    });
    found
}

unsafe fn sam_hrecs_hash_value(hash: *mut (), key: *const u8) -> Option<i32> {
    let hash = hash.cast::<khash_m_s2i_t>();
    if hash.is_null() || key.is_null() {
        return None;
    }
    let k = kh_get_m_s2i(hash, key);
    if k == (*hash).n_buckets {
        None
    } else {
        Some(*(*hash).vals.add(k as usize))
    }
}

unsafe fn sam_hrecs_ref_name_ptr(hrecs: *const sam_hrecs_t, key: *const u8) -> bool {
    if hrecs.is_null() || key.is_null() || (*hrecs).ref_.is_null() {
        return false;
    }
    for i in 0..(*hrecs).nref {
        if (*(*hrecs).ref_.add(i as usize)).name == key {
            return true;
        }
    }
    false
}

pub(crate) unsafe fn sam_hrecs_free_ref_altname_hash_keys(hrecs: *mut sam_hrecs_t) {
    let _ = hrecs;
    // ref_hash mixes borrowed SN tag pointers with allocated AN token keys.
    // Without an ownership bit per khash entry, freeing only aliases is not
    // ABI-safe after deletions/resizes have mutated the table.
}

// original: sam_hrecs_add_ref_altnames (htslib/header.c:88)
unsafe fn sam_hrecs_add_ref_altnames(
    hrecs: *mut sam_hrecs_t,
    nref: i32,
    list: *const u8,
) -> i32 {
    if hrecs.is_null() {
        return -1;
    }
    if list.is_null() {
        return 0;
    }

    for token in CStr::from_ptr(list.cast()).to_bytes().split(|&b| b == b',') {
        if token.is_empty() {
            continue;
        }
        let name = sam_hrecs_strdup_bytes(token);
        if name.is_null() {
            return -1;
        }
        let hash = (*hrecs).ref_hash.cast::<khash_m_s2i_t>();
        let mut ret = 0;
        let k = kh_put_str2int(hash, name, &mut ret);
        if ret < 0 {
            {
            let mut l = 0usize;
            while *name.add(l) != 0 {
                l += 1;
            }
            drop(Vec::from_raw_parts(name, l + 1, l + 1));
        }
            return -1;
        }
        if ret > 0 {
            *(*hash).vals.add(k as usize) = nref;
        } else {
            {
            let mut l = 0usize;
            while *name.add(l) != 0 {
                l += 1;
            }
            drop(Vec::from_raw_parts(name, l + 1, l + 1));
        }
        }
    }
    0
}

// original: sam_hrecs_remove_ref_altnames (htslib/header.c:115)
pub(crate) unsafe fn sam_hrecs_remove_ref_altnames(
    hrecs: *mut sam_hrecs_t,
    expected: i32,
    list: *const u8,
) {
    if hrecs.is_null() || list.is_null() || expected < 0 || expected >= (*hrecs).nref {
        return;
    }
    let sn = (*(*hrecs).ref_.add(expected as usize)).name;
    let hash = (*hrecs).ref_hash.cast::<khash_m_s2i_t>();
    for token in CStr::from_ptr(list.cast()).to_bytes().split(|&b| b == b',') {
        if token.is_empty() {
            continue;
        }
        let name = sam_hrecs_strdup_bytes(token);
        if name.is_null() {
            continue;
        }
        let k = kh_get_m_s2i(hash, name);
        if k != (*hash).n_buckets && *(*hash).vals.add(k as usize) == expected && !cstr_eq(sn, name)
        {
            *(*hash).flags.add((k >> 4) as usize) |= 1 << ((k & 0x0f) << 1);
            (*hash).size = (*hash).size.saturating_sub(1);
        }
        {
            let mut l = 0usize;
            while *name.add(l) != 0 {
                l += 1;
            }
            drop(Vec::from_raw_parts(name, l + 1, l + 1));
        }
    }
}

unsafe fn sam_hrec_set_tag_value(
    hrecs: *mut sam_hrecs_t,
    type_: *mut sam_hrec_type_t,
    key: &[u8],
    value: &[u8],
) -> i32 {
    if hrecs.is_null() || type_.is_null() || key.len() != 2 {
        return -1;
    }

    let key_cstr = [key[0], key[1], 0];
    let (found, prev) =
        sam_hrecs_find_key(&mut *type_, &key_cstr);
    let mut tag = found.map_or(std::ptr::null_mut(), |p| p.as_ptr());
    if tag.is_null() {
        tag = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_tag_t>())).cast::<()>()
            .cast::<sam_hrec_tag_t>();
        if tag.is_null() {
            return -1;
        }
        match prev {
            None => (*type_).tag = tag,
            Some(prev) => (*prev.as_ptr()).next = tag,
        }
    }

    let len = match 3usize.checked_add(value.len()) {
        Some(v) if v <= i32::MAX as usize => v,
        _ => return -1,
    };
    let str_ = { let mut v: Vec<u8> = vec![0u8; (len as u64 + 1) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
    if str_.is_null() {
        return -1;
    }
    *str_.add(0) = key[0] as u8;
    *str_.add(1) = key[1] as u8;
    *str_.add(2) = b':' as u8;
    if !value.is_empty() {
        std::ptr::copy_nonoverlapping(value.as_ptr().cast::<u8>(), str_.add(3).cast::<u8>(), (value.len()) as usize);
    }
    *str_.add(len) = 0;
    (*tag).str_ = str_;
    (*tag).len = len as i32;
    (*hrecs).dirty = 1;
    0
}

// Allocates a single header tag whose serialized form is `field` (e.g.
// b"SN:chr1" for a key/value pair, or a bare comment for @CO lines).
unsafe fn sam_hrecs_alloc_tag(field: &[u8]) -> *mut sam_hrec_tag_t {
    if field.len() > i32::MAX as usize {
        return std::ptr::null_mut();
    }
    let tag = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_tag_t>())).cast::<()>()
        .cast::<sam_hrec_tag_t>();
    if tag.is_null() {
        return std::ptr::null_mut();
    }
    let s = sam_hrecs_strdup_bytes(field);
    if s.is_null() {
        drop(Box::from_raw(tag));
        return std::ptr::null_mut();
    }
    (*tag).str_ = s;
    (*tag).len = field.len() as i32;
    (*tag).next = std::ptr::null_mut();
    tag
}

// original: sam_hrecs_vadd (htslib/header.c:553)
//
// Adds a new header line of the given `type_` to the hrecs structure with the
// supplied key/value `tags`. The C function takes a va_list plus trailing
// varargs so that variadic callers (sam_hdr_add_pg) can splice their own pairs
// ahead of the caller's; the Rust slice form collapses both lists into one, so
// callers pre-build the combined slice (and pre-drop empty-value prefix pairs,
// matching the `if (*val == '\0') continue;` behaviour of the C trailing-vararg
// loop). For @CO lines each entry's key holds the comment text and the value is
// ignored. If an @HD line already exists this updates it in place, mirroring C.
unsafe fn sam_hrecs_vadd(
    hrecs: *mut sam_hrecs_t,
    type_: *const u8,
    tags: &[(*const u8, *const u8)],
) -> i32 {
    if hrecs.is_null() || type_.is_null() {
        return -1;
    }
    let type_bytes = [*type_ as u8, *type_.add(1) as u8];
    let is_co = &type_bytes == b"CO";

    // @HD is a singleton: update the existing line rather than adding a second.
    if &type_bytes == b"HD" {
        let hd = sam_hrecs_find_type_id(&mut *hrecs, CStr::from_ptr(type_.cast()).to_bytes(), None);
        if let Some(hd) = hd {
            return sam_hrecs_update_pairs(hrecs, hd.as_ptr(), tags);
        }
    }

    let ty = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_type_t>())).cast::<()>()
        .cast::<sam_hrec_type_t>();
    if ty.is_null() {
        return -1;
    }
    (*ty).type_ = ((type_bytes[0] as u32) << 8) | type_bytes[1] as u32;
    sam_hrecs_type_list_add(hrecs, ty);

    let mut last: *mut sam_hrec_tag_t = std::ptr::null_mut();
    for &(key, value) in tags {
        if key.is_null() {
            break;
        }
        let field = if is_co {
            // @CO tags are stored with their leading tab (matching
            // parse_comment_line, which keeps line[3..]); the rebuild emits the
            // type followed by the tag string verbatim, with no extra separator.
            let comment = CStr::from_ptr(key.cast()).to_bytes();
            let mut field = Vec::with_capacity(1 + comment.len());
            field.push(b'\t');
            field.extend_from_slice(comment);
            field
        } else {
            if value.is_null() {
                break;
            }
            let key_b = CStr::from_ptr(key.cast()).to_bytes();
            let value_b = CStr::from_ptr(value.cast()).to_bytes();
            let mut field = Vec::with_capacity(key_b.len() + 1 + value_b.len());
            field.extend_from_slice(key_b);
            field.push(b':');
            field.extend_from_slice(value_b);
            field
        };
        let tag = sam_hrecs_alloc_tag(&field);
        if tag.is_null() {
            return -1;
        }
        if last.is_null() {
            (*ty).tag = tag;
        } else {
            (*last).next = tag;
        }
        last = tag;
    }

    if sam_hrecs_update_hashes(hrecs) < 0 {
        return -1;
    }
    if &type_bytes == b"PG" {
        (*hrecs).pgs_changed = 1;
    }
    (*hrecs).dirty = 1;
    0
}

pub(crate) unsafe fn sam_hrecs_update_pairs(
    hrecs: *mut sam_hrecs_t,
    type_: *mut sam_hrec_type_t,
    tags: &[(*const u8, *const u8)],
) -> i32 {
    if hrecs.is_null() || type_.is_null() {
        return -1;
    }
    for &(key, value) in tags {
        if key.is_null() {
            return -1;
        }
        let key = CStr::from_ptr(key.cast()).to_bytes();
        if key.len() != 2 {
            return -1;
        }
        let value = if value.is_null() {
            b"" as &[u8]
        } else {
            CStr::from_ptr(value.cast()).to_bytes()
        };
        if sam_hrec_set_tag_value(hrecs, type_, key, value) < 0 {
            return -1;
        }
    }
    0
}

pub(crate) enum SamHrecNameUpdate {
    Unchanged,
    Changed,
    Clash,
}

// original: check_for_name_update (htslib/header.c:1866)
pub(crate) unsafe fn check_for_name_update(
    hrecs: *mut sam_hrecs_t,
    rec: *mut sam_hrec_type_t,
    tags: &[(*const u8, *const u8)],
) -> SamHrecNameUpdate {
    if hrecs.is_null() || rec.is_null() {
        return SamHrecNameUpdate::Unchanged;
    }

    let (id_tag, hash) = if (*rec).type_ == header_h_58_TYPEKEY(c"SQ".as_ptr().cast()) {
        (b"SN" as &[u8], (*hrecs).ref_hash)
    } else if (*rec).type_ == header_h_58_TYPEKEY(c"RG".as_ptr().cast()) {
        (b"ID" as &[u8], (*hrecs).rg_hash)
    } else if (*rec).type_ == header_h_58_TYPEKEY(c"PG".as_ptr().cast()) {
        (b"ID" as &[u8], (*hrecs).pg_hash)
    } else {
        return SamHrecNameUpdate::Unchanged;
    };

    let old = sam_hrec_tag_value_cstr(rec, &[id_tag[0], id_tag[1]]);
    if old.is_null() {
        return SamHrecNameUpdate::Unchanged;
    }

    let mut ret = SamHrecNameUpdate::Unchanged;
    for &(key, value) in tags {
        if key.is_null() {
            continue;
        }
        let key = CStr::from_ptr(key.cast()).to_bytes();
        if key != id_tag {
            continue;
        }
        let value = if value.is_null() { c"".as_ptr().cast() } else { value };
        if cstr_eq(value, old) {
            ret = SamHrecNameUpdate::Unchanged;
            continue;
        }
        ret = if sam_hrecs_hash_value(hash, value).is_some() {
            SamHrecNameUpdate::Clash
        } else {
            SamHrecNameUpdate::Changed
        };
    }
    ret
}

unsafe fn sam_hrecs_parse_tag(field: &[u8]) -> *mut sam_hrec_tag_t {
    if field.len() < 3 || field[2] != b':' {
        return std::ptr::null_mut();
    }
    let tag = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_tag_t>())).cast::<()>()
        .cast::<sam_hrec_tag_t>();
    if tag.is_null() {
        return std::ptr::null_mut();
    }
    let s = sam_hrecs_strdup_bytes(field);
    if s.is_null() {
        drop(Box::from_raw(tag));
        return std::ptr::null_mut();
    }
    (*tag).str_ = s;
    (*tag).len = field.len() as i32;
    tag
}

// original: parse_comment_line (htslib/header.c:976)
unsafe fn parse_comment_line(line: &[u8]) -> *mut sam_hrec_type_t {
    if !line.starts_with(b"@CO") {
        return std::ptr::null_mut();
    }
    let ty = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_type_t>())).cast::<()>()
        .cast::<sam_hrec_type_t>();
    if ty.is_null() {
        return std::ptr::null_mut();
    }
    (*ty).type_ = header_h_58_TYPEKEY(c"CO".as_ptr().cast());
    let comment = &line[3..];
    let tag = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_tag_t>())).cast::<()>()
        .cast::<sam_hrec_tag_t>();
    if tag.is_null() {
        drop(Box::from_raw(ty));
        return std::ptr::null_mut();
    }
    let s = sam_hrecs_strdup_bytes(comment);
    if s.is_null() {
        drop(Box::from_raw(tag));
        drop(Box::from_raw(ty));
        return std::ptr::null_mut();
    }
    (*tag).str_ = s;
    (*tag).len = comment.len() as i32;
    (*ty).tag = tag;
    ty
}

// original: parse_noncomment_line (htslib/header.c:1011)
unsafe fn parse_noncomment_line(line: &[u8]) -> *mut sam_hrec_type_t {
    if line.len() < 4 || line[0] != b'@' || line[3] != b'\t' {
        return std::ptr::null_mut();
    }
    let ty = Box::into_raw(Box::new(std::mem::zeroed::<sam_hrec_type_t>())).cast::<()>()
        .cast::<sam_hrec_type_t>();
    if ty.is_null() {
        return std::ptr::null_mut();
    }
    (*ty).type_ = ((line[1] as u32) << 8) | line[2] as u32;

    let mut tail: *mut sam_hrec_tag_t = std::ptr::null_mut();
    for field in line[4..].split(|&b| b == b'\t') {
        if field.is_empty() {
            continue;
        }
        let tag = sam_hrecs_parse_tag(field);
        if tag.is_null() {
            sam_hrecs_free_tags((*ty).tag);
            drop(Box::from_raw(ty));
            return std::ptr::null_mut();
        }
        if tail.is_null() {
            (*ty).tag = tag;
        } else {
            (*tail).next = tag;
        }
        tail = tag;
    }
    ty
}

// original: sam_hrecs_parse_single_line (htslib/header.c:1118)
unsafe fn sam_hrecs_parse_single_line(hrecs: *mut sam_hrecs_t, line: &[u8]) -> i32 {
    if hrecs.is_null() || line.len() < 3 || line[0] != b'@' {
        return -1;
    }
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let ty = if line.starts_with(b"@CO") {
        parse_comment_line(line)
    } else {
        parse_noncomment_line(line)
    };
    if ty.is_null() {
        return -1;
    }
    // C's sam_hrecs_parse_single_line appends each parsed line at the global
    // end (with @HD floated to the top by sam_hrecs_global_list_add). It does
    // NOT type-group, so the canonical text order is preserved verbatim — for
    // both the initial header parse and incremental sam_hdr_add_lines calls.
    sam_hrecs_global_list_add(hrecs, ty, std::ptr::null_mut());
    0
}

// original: sam_hrecs_parse_lines (htslib/header.c:1188)
pub(crate) unsafe fn sam_hrecs_parse_lines(
    hrecs: *mut sam_hrecs_t,
    text: *const u8,
    len: usize,
) -> i32 {
    if hrecs.is_null() || text.is_null() {
        return -1;
    }
    let bytes = std::slice::from_raw_parts(text.cast::<u8>(), len);
    for raw in bytes.split(|&b| b == b'\n') {
        if raw.is_empty() {
            continue;
        }
        if sam_hrecs_parse_single_line(hrecs, raw) < 0 {
            return -1;
        }
    }
    0
}

unsafe fn sam_hrec_tag_value_cstr(line: *mut sam_hrec_type_t, key: &[u8; 2]) -> *const u8 {
    let mut tag = (*line).tag;
    while !tag.is_null() {
        if !(*tag).str_.is_null()
            && (*tag).len >= 3
            && *(*tag).str_.cast::<u8>() == key[0]
            && *(*tag).str_.cast::<u8>().add(1) == key[1]
            && *(*tag).str_.cast::<u8>().add(2) == b':'
        {
            return (*tag).str_.add(3);
        }
        tag = (*tag).next;
    }
    std::ptr::null()
}

unsafe fn sam_hrec_tag_value_len(line: *mut sam_hrec_type_t, key: &[u8; 2]) -> i32 {
    let mut tag = (*line).tag;
    while !tag.is_null() {
        if !(*tag).str_.is_null()
            && (*tag).len >= 3
            && *(*tag).str_.cast::<u8>() == key[0]
            && *(*tag).str_.cast::<u8>().add(1) == key[1]
        {
            return (*tag).len - 3;
        }
        tag = (*tag).next;
    }
    -1
}

unsafe fn sam_hrec_parse_len(value: *const u8) -> Option<hts_pos_t> {
    if value.is_null() {
        return None;
    }
    let bytes = CStr::from_ptr(value.cast()).to_bytes();
    let mut n = 0u64;
    if bytes.is_empty() {
        return None;
    }
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        n = n.checked_mul(10)?.checked_add((b - b'0') as u64)?;
        if n > HTS_POS_MAX as u64 {
            return None;
        }
    }
    if n == 0 {
        None
    } else {
        Some(n as hts_pos_t)
    }
}

unsafe fn sam_hrecs_reset_hash(hash: *mut *mut ()) -> i32 {
    khash_str2int_destroy(*hash);
    *hash = khash_str2int_init();
    if (*hash).is_null() {
        -1
    } else {
        0
    }
}

// original: sam_hrecs_remove_hash_entry (htslib/header.c:378)
unsafe fn sam_hrecs_remove_hash_entry(hash: *mut (), key: *const u8) -> i32 {
    let hash = hash.cast::<khash_m_s2i_t>();
    if hash.is_null() || key.is_null() {
        return -1;
    }
    let k = kh_get_m_s2i(hash, key);
    if k == (*hash).n_buckets {
        return 0;
    }
    *(*hash).flags.add((k >> 4) as usize) |= 1 << ((k & 0x0f) << 1);
    (*hash).size = (*hash).size.saturating_sub(1);
    1
}

// original: rebuild_hash (htslib/header.c:605)
unsafe fn rebuild_hash(hrecs: *mut sam_hrecs_t, type_: u32) -> i32 {
    if hrecs.is_null() {
        return -1;
    }
    match type_ {
        t if t == header_h_58_TYPEKEY(c"SQ".as_ptr().cast()) => {
            if sam_hrecs_reset_hash(&mut (*hrecs).ref_hash) < 0 {
                return -1;
            }
            for i in 0..(*hrecs).nref {
                let r = (*hrecs).ref_.add(i as usize);
                if !(*r).name.is_null() && khash_str2int_set((*hrecs).ref_hash, (*r).name, i) < 0 {
                    return -1;
                }
            }
        }
        t if t == header_h_58_TYPEKEY(c"RG".as_ptr().cast()) => {
            if sam_hrecs_reset_hash(&mut (*hrecs).rg_hash) < 0 {
                return -1;
            }
            for i in 0..(*hrecs).nrg {
                let rg = (*hrecs).rg.cast::<sam_hrec_rg_t>().add(i as usize);
                if !(*rg).name.is_null() && khash_str2int_set((*hrecs).rg_hash, (*rg).name, i) < 0 {
                    return -1;
                }
            }
        }
        t if t == header_h_58_TYPEKEY(c"PG".as_ptr().cast()) => {
            if sam_hrecs_reset_hash(&mut (*hrecs).pg_hash) < 0 {
                return -1;
            }
            for i in 0..(*hrecs).npg {
                let pg = (*hrecs).pg.cast::<sam_hrec_pg_t>().add(i as usize);
                if !(*pg).name.is_null() && khash_str2int_set((*hrecs).pg_hash, (*pg).name, i) < 0 {
                    return -1;
                }
            }
        }
        _ => {}
    }
    0
}

pub(crate) unsafe fn build_header_line(ty: *const sam_hrec_type_t, ks: *mut kstring_t) -> i32 {
    let c = [((*ty).type_ >> 8) as u8, ((*ty).type_ & 0xff) as u8];
    if kputc_(b'@' as i32, &mut *ks) < 0 || kputsn(&c, 2, &mut *ks) < 0 {
        return -1;
    }

    if (*ty).type_ == header_h_58_TYPEKEY(c"CO".as_ptr().cast()) {
        if !(*ty).tag.is_null() && !(*(*ty).tag).str_.is_null() {
            let len = (*(*ty).tag).len as usize;
            let slice = std::slice::from_raw_parts((*(*ty).tag).str_.cast::<u8>(), len);
            if kputsn(slice, len, &mut *ks) < 0 {
                return -1;
            }
        }
        return 0;
    }

    let mut tag = (*ty).tag;
    while !tag.is_null() {
        let len = (*tag).len as usize;
        let slice = std::slice::from_raw_parts((*tag).str_.cast::<u8>(), len);
        if kputc_(b'\t' as i32, &mut *ks) < 0 || kputsn(slice, len, &mut *ks) < 0 {
            return -1;
        }
        tag = (*tag).next;
    }
    0
}

// original: sam_hrecs_update_hashes (htslib/header.c:1285)
pub(crate) unsafe fn sam_hrecs_update_hashes(hrecs: *mut sam_hrecs_t) -> i32 {
    if hrecs.is_null() {
        return -1;
    }

    if !(*hrecs).ref_.is_null() {
        let cap = (*hrecs).ref_sz as usize;
        drop(Vec::from_raw_parts((*hrecs).ref_, cap, cap));
    }
    if !(*hrecs).rg.is_null() {
        let cap = (*hrecs).rg_sz as usize;
        drop(Vec::from_raw_parts(
            (*hrecs).rg.cast::<sam_hrec_rg_t>(),
            cap,
            cap,
        ));
    }
    if !(*hrecs).pg.is_null() {
        let cap = (*hrecs).pg_sz as usize;
        drop(Vec::from_raw_parts(
            (*hrecs).pg.cast::<sam_hrec_pg_t>(),
            cap,
            cap,
        ));
    }
    (*hrecs).ref_ = std::ptr::null_mut();
    (*hrecs).rg = std::ptr::null_mut();
    (*hrecs).pg = std::ptr::null_mut();
    (*hrecs).nref = 0;
    (*hrecs).nrg = 0;
    (*hrecs).npg = 0;
    (*hrecs).ref_sz = 0;
    (*hrecs).rg_sz = 0;
    (*hrecs).pg_sz = 0;

    let mut ref_rows: Vec<sam_hrec_sq_t> = Vec::new();
    let mut rg_rows: Vec<sam_hrec_rg_t> = Vec::new();
    let mut pg_rows: Vec<sam_hrec_pg_t> = Vec::new();

    let sq_key = header_h_58_TYPEKEY(c"SQ".as_ptr().cast());
    let rg_key = header_h_58_TYPEKEY(c"RG".as_ptr().cast());
    let pg_key = header_h_58_TYPEKEY(c"PG".as_ptr().cast());
    let mut ok = true;
    sam_hrecs_walk_global(hrecs, |line| {
        if (*line).type_ == sq_key {
            let name = sam_hrec_tag_value_cstr(line, b"SN");
            let len = sam_hrec_parse_len(sam_hrec_tag_value_cstr(line, b"LN"));
            if name.is_null() || len.is_none() {
                ok = false;
                return false;
            }
            ref_rows.push(sam_hrec_sq_t {
                name,
                len: len.unwrap(),
                ty: line.cast(),
            });
        } else if (*line).type_ == rg_key {
            let name = sam_hrec_tag_value_cstr(line, b"ID");
            if name.is_null() {
                ok = false;
                return false;
            }
            rg_rows.push(sam_hrec_rg_t {
                name,
                ty: line.cast(),
                name_len: sam_hrec_tag_value_len(line, b"ID"),
                id: rg_rows.len() as i32,
            });
        } else if (*line).type_ == pg_key {
            let name = sam_hrec_tag_value_cstr(line, b"ID");
            if name.is_null() {
                ok = false;
                return false;
            }
            pg_rows.push(sam_hrec_pg_t {
                name,
                ty: line.cast(),
                name_len: sam_hrec_tag_value_len(line, b"ID"),
                id: pg_rows.len() as i32,
                prev_id: -1,
            });
        }
        true
    });
    if !ok {
        return -1;
    }

    if !ref_rows.is_empty() {
        (*hrecs).nref = ref_rows.len() as i32;
        (*hrecs).ref_sz = (*hrecs).nref;
        let mut ref_rows = ref_rows;
        (*hrecs).ref_ = ref_rows.as_mut_ptr();
        std::mem::forget(ref_rows);
    }
    if !rg_rows.is_empty() {
        (*hrecs).nrg = rg_rows.len() as i32;
        (*hrecs).rg_sz = (*hrecs).nrg;
        let mut rg_rows = rg_rows;
        (*hrecs).rg = rg_rows.as_mut_ptr().cast();
        std::mem::forget(rg_rows);
    }
    if !pg_rows.is_empty() {
        (*hrecs).npg = pg_rows.len() as i32;
        (*hrecs).pg_sz = (*hrecs).npg;
        let mut pg_rows = pg_rows;
        (*hrecs).pg = pg_rows.as_mut_ptr().cast();
        std::mem::forget(pg_rows);
    }

    if rebuild_hash(hrecs, sq_key) < 0
        || rebuild_hash(hrecs, rg_key) < 0
        || rebuild_hash(hrecs, pg_key) < 0
    {
        return -1;
    }
    for i in 0..(*hrecs).nref {
        let line = (*(*hrecs).ref_.add(i as usize))
            .ty
            .cast::<sam_hrec_type_t>();
        let altnames = sam_hrec_tag_value_cstr(line, b"AN");
        if sam_hrecs_add_ref_altnames(hrecs, i, altnames) < 0 {
            return -1;
        }
    }
    (*hrecs).refs_changed = -1;
    (*hrecs).pgs_changed = 0;
    0
}

// original: rebuild_target_arrays (htslib/header.c:1398)
pub(crate) unsafe fn rebuild_target_arrays(h: *mut sam_hdr_t) -> i32 {
    if h.is_null() || (*h).hrecs.is_null() {
        return -1;
    }
    sam_hdr_clear_targets(h);
    let hrecs = (*h).hrecs;
    for i in 0..(*hrecs).nref {
        let r = (*hrecs).ref_.add(i as usize);
        if (*r).name.is_null()
            || sam_hdr_append_target(h, CStr::from_ptr((*r).name.cast()).to_bytes(), (*r).len) < 0
        {
            return -1;
        }
    }
    0
}

// original: sam_hdr_update_target_arrays (htslib/header.c:1468)
unsafe fn sam_hdr_update_target_arrays(h: *mut sam_hdr_t) -> i32 {
    if h.is_null() || (*h).hrecs.is_null() {
        return -1;
    }
    if (*(*h).hrecs).refs_changed < 0 {
        return 0;
    }
    rebuild_target_arrays(h)
}

// original: sam_hrecs_refs_from_targets_array (htslib/header.c:1501)
unsafe fn sam_hrecs_refs_from_targets_array(h: *mut sam_hdr_t) -> i32 {
    if h.is_null() || (*h).hrecs.is_null() {
        return -1;
    }
    let hrecs = (*h).hrecs;
    for i in 0..(*h).n_targets {
        let name = *(*h).target_name.add(i as usize);
        if name.is_null() {
            return -1;
        }
        let mut line = Vec::new();
        line.extend_from_slice(b"@SQ\tSN:");
        line.extend_from_slice(CStr::from_ptr(name.cast()).to_bytes());
        line.extend_from_slice(b"\tLN:");
        line.extend_from_slice(
            (*(*h).target_len.add(i as usize) as u64)
                .to_string()
                .as_bytes(),
        );
        if sam_hrecs_parse_single_line(hrecs, &line) < 0 {
            return -1;
        }
    }
    sam_hrecs_update_hashes(hrecs)
}

// original: add_stub_ref_sq_lines (htslib/header.c:1777)
pub(crate) unsafe fn add_stub_ref_sq_lines(h: *mut sam_hdr_t) -> i32 {
    sam_hrecs_refs_from_targets_array(h)
}

// original: sam_hrecs_rebuild_lines (htslib/header.c:758)
unsafe fn sam_hrecs_rebuild_lines(hrecs: *const sam_hrecs_t, ks: *mut kstring_t) -> i32 {
    sam_hrecs_rebuild_text(&*hrecs, &mut *ks)
}

// original: sam_hdr_build_from_sam_file (htslib/header.c:2711)
unsafe fn sam_hdr_build_from_sam_file(h: *mut sam_hdr_t) -> i32 {
    sam_hdr_fill_hrecs(&mut *h)
}

// original: sam_hrecs_dump (htslib/header.c:2353)
unsafe fn sam_hrecs_dump(hrecs: *const sam_hrecs_t) -> Option<Vec<u8>> {
    let mut ks = kstring_t::default();
    if sam_hrecs_rebuild_text(&*hrecs, &mut ks) < 0 {
        ks_free(&mut ks);
        return None;
    }
    Some(ks_release(&mut ks))
}

// original: sam_hrecs_dup (htslib/header.c:2919)
unsafe fn sam_hrecs_dup(src: *const sam_hrecs_t) -> *mut sam_hrecs_t {
    if src.is_null() {
        return std::ptr::null_mut();
    }
    let text = match sam_hrecs_dump(src) {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };
    let dst = sam_hrecs_new();
    if dst.is_null() {
        return std::ptr::null_mut();
    }
    let len = text.len();
    if sam_hrecs_parse_lines(dst, text.as_ptr().cast(), len) < 0
        || sam_hrecs_update_hashes(dst) < 0
    {
        sam_hrecs_free(dst);
        return std::ptr::null_mut();
    }
    (*dst).dirty = (*src).dirty;
    dst
}

// htslib/header.c:414
// Faithful 1:1 translation of static `sam_hrecs_remove_hash_entry(hrecs, type, h_type)`.
// Removes a single SQ or RG entry from the indexed ref/rg arrays and the
// associated `m_s2i` hash table.  The htslib v1.23 C source uses
// `kh_del(m_s2i, ...)` to drop the slot; we mirror that by flipping the
// `flags` `del` bit and decrementing `size`, matching how the native
// `sam_hrecs_remove_hash_entry` (the 2-arg key-deleter at sam.rs:3445) does
// it for the same kind of hash.
unsafe fn header_c_414_sam_hrecs_remove_hash_entry(
    hrecs: *mut sam_hrecs_t,
    type_: u32,
    h_type: *mut sam_hrec_type_t,
) -> i32 {
    if hrecs.is_null() || h_type.is_null() {
        return -1;
    }

    let sq_key = header_h_58_TYPEKEY(c"SQ".as_ptr().cast());
    let rg_key = header_h_58_TYPEKEY(c"RG".as_ptr().cast());

    // Remove name and any alternative names from reference hash.
    if type_ == sq_key {
        let mut key: *const u8 = std::ptr::null();
        let mut altnames: *const u8 = std::ptr::null();

        let mut tag = (*h_type).tag;
        while !tag.is_null() {
            let s = (*tag).str_;
            if !s.is_null() && (*tag).len >= 3 {
                let b0 = *s as u8;
                let b1 = *s.add(1) as u8;
                if b0 == b'S' && b1 == b'N' {
                    key = s.add(3);
                } else if b0 == b'A' && b1 == b'N' {
                    altnames = s.add(3);
                }
            }
            tag = (*tag).next;
        }

        if !key.is_null() {
            let hash = (*hrecs).ref_hash.cast::<khash_m_s2i_t>();
            let k = kh_get_m_s2i(hash, key);
            if k != (*hash).n_buckets {
                let idx = *(*hash).vals.add(k as usize);
                if idx + 1 < (*hrecs).nref {
                    let dst = (*hrecs).ref_.add(idx as usize);
                    let src = (*hrecs).ref_.add(idx as usize + 1);
                    std::ptr::copy(src.cast::<u8>(), dst.cast(), (std::mem::size_of::<sam_hrec_sq_t>() as u64
                            * ((*hrecs).nref - idx - 1) as u64) as usize);
                }
                if !altnames.is_null() {
                    sam_hrecs_remove_ref_altnames(hrecs, idx, altnames);
                }
                // kh_del: mark deleted, drop size by 1.
                *(*hash).flags.add((k >> 4) as usize) |= 1 << ((k & 0x0f) << 1);
                (*hash).size = (*hash).size.saturating_sub(1);
                (*hrecs).nref -= 1;
                if (*hrecs).refs_changed < 0 || (*hrecs).refs_changed > idx {
                    (*hrecs).refs_changed = idx;
                }
                for kk in 0..(*hash).n_buckets {
                    if !kh_iseither((*hash).flags, kk) && *(*hash).vals.add(kk as usize) > idx {
                        *(*hash).vals.add(kk as usize) -= 1;
                    }
                }
            }
        }
    }

    // Remove from read-group hash.
    if type_ == rg_key {
        let mut tag = (*h_type).tag;
        while !tag.is_null() {
            let s = (*tag).str_;
            if !s.is_null() && (*tag).len >= 3 && *s as u8 == b'I' && *s.add(1) as u8 == b'D' {
                let key = s.add(3);
                let hash = (*hrecs).rg_hash.cast::<khash_m_s2i_t>();
                let k = kh_get_m_s2i(hash, key);
                if k != (*hash).n_buckets {
                    let idx = *(*hash).vals.add(k as usize);
                    if idx + 1 < (*hrecs).nrg {
                        let dst = (*hrecs).rg.cast::<sam_hrec_rg_t>().add(idx as usize);
                        let src = (*hrecs).rg.cast::<sam_hrec_rg_t>().add(idx as usize + 1);
                        std::ptr::copy(src.cast::<u8>(), dst.cast(), (std::mem::size_of::<sam_hrec_rg_t>() as u64
                                * ((*hrecs).nrg - idx - 1) as u64) as usize);
                    }
                    *(*hash).flags.add((k >> 4) as usize) |= 1 << ((k & 0x0f) << 1);
                    (*hash).size = (*hash).size.saturating_sub(1);
                    (*hrecs).nrg -= 1;
                    for kk in 0..(*hash).n_buckets {
                        if !kh_iseither((*hash).flags, kk) && *(*hash).vals.add(kk as usize) > idx {
                            *(*hash).vals.add(kk as usize) -= 1;
                        }
                    }
                }
                break;
            }
            tag = (*tag).next;
        }
    }

    0
}

// htslib/header.c:704
// Faithful 1:1 translation of static `sam_hrecs_remove_line(hrecs, type_name,
// type_found, remove_hash)`.  Unlike the existing 2-arg native
// `sam_hrecs_remove_line` (used only by `sam_hrecs_free`, sam.rs:2633), this
// variant honours the `remove_hash` flag (for SQ/RG entries) and removes the
// record from its per-type linked list.  The C original also calls
// `kh_get(sam_hrecs_t, hrecs->h, itype)` / `kh_del(sam_hrecs_t, ...)` to
// maintain a per-type hash; this Rust port has no such hash table — the
// native lookups (`sam_hrecs_find_type_id`, etc.) use `ref_hash` / `rg_hash`
// / `pg_hash` plus a global-list walk, so the `hrecs->h` step is omitted (it
// has always been a redundant lookup in the native code path).
unsafe fn header_c_704_sam_hrecs_remove_line(
    hrecs: *mut sam_hrecs_t,
    type_name: *const u8,
    type_found: *mut sam_hrec_type_t,
    remove_hash: i32,
) -> i32 {
    if hrecs.is_null() || type_name.is_null() || type_found.is_null() {
        return -1;
    }

    let itype = header_h_58_TYPEKEY(type_name);

    // Remove from global doubly-linked list (remembering it may be the only
    // line).
    if (*hrecs).first_line == type_found.cast() {
        (*hrecs).first_line = if (*type_found).global_next != type_found {
            (*type_found).global_next.cast()
        } else {
            std::ptr::null_mut()
        };
    }
    (*(*type_found).global_next).global_prev = (*type_found).global_prev;
    (*(*type_found).global_prev).global_next = (*type_found).global_next;

    // Per-type circular list: if the record is the only one of its type, the
    // C original removes the hash entry too; in this Rust port the `hrecs->h`
    // table is unused (see fn-level comment), so we only need to update the
    // per-type circular `prev`/`next` pointers when more than one line of
    // this type remains.
    if (*type_found).prev != type_found && (*type_found).next != type_found {
        (*(*type_found).prev).next = (*type_found).next;
        (*(*type_found).next).prev = (*type_found).prev;
    }

    let sq_key = header_h_58_TYPEKEY(c"SQ".as_ptr().cast());
    let rg_key = header_h_58_TYPEKEY(c"RG".as_ptr().cast());
    if remove_hash != 0 && (itype == sq_key || itype == rg_key) {
        header_c_414_sam_hrecs_remove_hash_entry(hrecs, itype, type_found);
    }

    // Faithful note: the C original `pool_free(hrecs->type_pool, type_found)`
    // returns the record to a pool for later reuse — a no-op from the
    // allocator's perspective.  We can't unconditionally `c_compat::free` it:
    // when hrecs was built by hts_sys (e.g. via a C-side `sam_hdr_add_pg`
    // call on a native header), `type_found` is a slab-interior pointer
    // inside a libhts string pool and free()'ing it aborts the process
    // (`free(): invalid pointer`).  We therefore drop the record on the
    // floor — its memory leaks until the owning pool or arena is released,
    // which matches the existing native lifecycle for header records (the
    // native `sam_hdr_destroy`, sam.rs:5398, never frees the hrecs sub-tree
    // either).  We do NOT call `sam_hrecs_free_tags` for the same reason.
    let _ = type_found;

    (*hrecs).dirty = 1;

    0
}

// htslib/header.c:1784
// Faithful 1:1 translation of `sam_hdr_remove_line_id`, hrecs-mode body only.
// Looks up a header line by `(type, ID_key, ID_value)` and removes it,
// rebuilding the target arrays if a SQ removal changed `refs_changed` and
// invalidating the cached text if the header is now dirty.
pub(crate) unsafe fn header_c_1784_sam_hdr_remove_line_id_hrecs(
    bh: *mut sam_hdr_t,
    type_: *const u8,
    id_key: *const u8,
    id_value: *const u8,
) -> i32 {
    if bh.is_null() || type_.is_null() {
        return -1;
    }

    if (*bh).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *bh) != 0 {
        return -1;
    }
    let hrecs = (*bh).hrecs;

    if *type_ as u8 == b'P' && *type_.add(1) as u8 == b'G' {
        // hts_log_warning: Removing PG lines is not supported
        return -1;
    }

    let id = if id_key.is_null() {
        None
    } else {
        Some((CStr::from_ptr(id_key.cast()).to_bytes(), CStr::from_ptr(id_value.cast()).to_bytes()))
    };
    let type_found = sam_hrecs_find_type_id(&mut *hrecs, CStr::from_ptr(type_.cast()).to_bytes(), id);
    let type_found = match type_found {
        None => return 0,
        Some(t) => t.as_ptr(),
    };

    let ret = header_c_704_sam_hrecs_remove_line(hrecs, type_, type_found, 1);
    if ret == 0 {
        if (*hrecs).refs_changed >= 0 && rebuild_target_arrays(bh) != 0 {
            return -1;
        }
        if (*hrecs).dirty != 0 {
            redact_header_text(&mut *bh);
        }
    }

    ret
}

// htslib/header.c:1823
// Faithful 1:1 translation of `sam_hdr_remove_line_pos`, hrecs-mode body
// only.  Identical to `sam_hdr_remove_line_id` but selects the victim by
// position within the type group.
pub(crate) unsafe fn header_c_1823_sam_hdr_remove_line_pos_hrecs(
    bh: *mut sam_hdr_t,
    type_: *const u8,
    position: i32,
) -> i32 {
    if bh.is_null() || type_.is_null() || position < 0 {
        return -1;
    }

    if (*bh).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *bh) != 0 {
        return -1;
    }
    let hrecs = (*bh).hrecs;

    if *type_ as u8 == b'P' && *type_.add(1) as u8 == b'G' {
        // hts_log_warning: Removing PG lines is not supported
        return -1;
    }

    let type_found = sam_hrecs_find_type_pos(hrecs, type_, position);
    if type_found.is_null() {
        return -1;
    }

    let ret = header_c_704_sam_hrecs_remove_line(hrecs, type_, type_found, 1);
    if ret == 0 {
        if (*hrecs).refs_changed >= 0 && rebuild_target_arrays(bh) != 0 {
            return -1;
        }
        if (*hrecs).dirty != 0 {
            redact_header_text(&mut *bh);
        }
    }

    ret
}

// htslib/header.c:2015
// Faithful 1:1 translation of `sam_hdr_remove_except`, hrecs-mode body only.
// Removes every line of @p type except the one identified by
// `(ID_key, ID_value)`.  If `ID_key` is NULL all lines are removed.  PG and
// CO are rejected.
pub(crate) unsafe fn header_c_2015_sam_hdr_remove_except_hrecs(
    bh: *mut sam_hdr_t,
    type_: *const u8,
    id_key: *const u8,
    id_value: *const u8,
) -> i32 {
    if bh.is_null() || type_.is_null() {
        return -1;
    }

    if (*bh).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *bh) != 0 {
        return -1;
    }
    let hrecs = (*bh).hrecs;

    let t0 = *type_ as u8;
    let t1 = *type_.add(1) as u8;
    if (t0 == b'P' && t1 == b'G') || (t0 == b'C' && t1 == b'O') {
        // hts_log_warning: Removing PG or CO lines is not supported
        return -1;
    }

    let mut ret: i32 = 1;
    let mut remove_all: i32 = if id_key.is_null() { 1 } else { 0 };

    let id = if id_key.is_null() {
        None
    } else {
        Some((CStr::from_ptr(id_key.cast()).to_bytes(), CStr::from_ptr(id_value.cast()).to_bytes()))
    };
    let mut type_found = sam_hrecs_find_type_id(&mut *hrecs, CStr::from_ptr(type_.cast()).to_bytes(), id)
        .map_or(std::ptr::null_mut(), |t| t.as_ptr());
    if type_found.is_null() {
        // Could not match an exception — drop the whole type group, if any.
        // The C reaches the same point via kh_get(sam_hrecs_t, hrecs->h,
        // TYPEKEY(type)); in this Rust port we use sam_hrecs_find_type_pos
        // with index 0 to obtain the head of the type's circular list, which
        // is the same `kh_val(hrecs->h, k)` that the C reads.
        type_found = sam_hrecs_find_type_pos(hrecs, type_, 0);
        if type_found.is_null() {
            return 0;
        }
        remove_all = 1;
    }

    let mut step = (*type_found).next;
    while step != type_found {
        let to_remove = step;
        step = (*step).next;
        ret &= header_c_704_sam_hrecs_remove_line(hrecs, type_, to_remove, 0);
    }

    if remove_all != 0 {
        ret &= header_c_704_sam_hrecs_remove_line(hrecs, type_, type_found, 0);
    }

    // For SQ/RG, faster to drop & rebuild the secondary hashes than delete
    // each entry individually. We also refresh the dense ref_/rg_/pg_ arrays
    // by calling sam_hrecs_update_hashes — without it, a subsequent
    // sam_hrecs_find_type_pos would dereference a stale pointer into a
    // freed record (the rg[] array indexes into removed records).
    if ((t0 == b'S' && t1 == b'Q') || (t0 == b'R' && t1 == b'G'))
        && rebuild_hash(hrecs, header_h_58_TYPEKEY(type_)) != 0
    {
        return -1;
    }
    if sam_hrecs_update_hashes(hrecs) < 0 {
        return -1;
    }

    if ret == 0 && (*hrecs).dirty != 0 {
        redact_header_text(&mut *bh);
    }

    0
}

// htslib/header.c:2071
// Faithful 1:1 translation of `sam_hdr_remove_lines`, hrecs-mode body only.
// `vrh` points to a string-set (KHASH_SET_INIT_STR / `rmhash_t`) that lists
// the values of @p id to *keep*; every line whose `id` value is absent from
// the set is removed.  When `vrh` is NULL we drop the whole type group.
// The native call sites pass a hash created with `khash_str2int_init`, which
// is layout-compatible with `rmhash_t` for the purposes of a key lookup
// (both share the `khash_m_s2i_t` head: `n_buckets`, `flags`, `keys`, …).
pub(crate) unsafe fn header_c_2071_sam_hdr_remove_lines_hrecs(
    bh: *mut sam_hdr_t,
    type_: *const u8,
    id: *const u8,
    vrh: *mut (),
) -> i32 {
    if bh.is_null() || type_.is_null() {
        return -1;
    }
    if vrh.is_null() {
        return header_c_2015_sam_hdr_remove_except_hrecs(
            bh,
            type_,
            std::ptr::null(),
            std::ptr::null(),
        );
    }
    if id.is_null() {
        return -1;
    }

    if (*bh).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *bh) != 0 {
        return -1;
    }
    let hrecs = (*bh).hrecs;
    let rh = vrh.cast::<khash_m_s2i_t>();

    // sam_hrecs_find_type_pos(.., 0) is the native analogue of the C
    // kh_get(sam_hrecs_t, hrecs->h, TYPEKEY(type)) lookup: it returns the
    // head of the per-type circular list, or NULL if no line of that type
    // exists.
    let head = sam_hrecs_find_type_pos(hrecs, type_, 0);
    if head.is_null() {
        return 0;
    }

    let mut ret: i32 = 0;
    let mut step = (*head).next;
    while step != head {
        let tag = sam_hrecs_find_key(&mut *step, CStr::from_ptr(id.cast()).to_bytes())
            .0
            .map_or(std::ptr::null_mut(), |t| t.as_ptr());
        if !tag.is_null() && !(*tag).str_.is_null() && (*tag).len >= 3 {
            let value = (*tag).str_.add(3);
            let k = kh_get_m_s2i(rh, value);
            if k == (*rh).n_buckets {
                // Value is not in the keep-set → remove this line.
                let to_remove = step;
                step = (*step).next;
                ret |= header_c_704_sam_hrecs_remove_line(hrecs, type_, to_remove, 0);
            } else {
                step = (*step).next;
            }
        } else {
            step = (*step).next;
        }
    }

    // Process the head line.  Note: as in C, `head` may have been moved if
    // we removed it via the loop above (but that loop never inspects head
    // itself); we re-fetch via the same find_type_pos to be defensive.
    let mut head = head;
    let tag = sam_hrecs_find_key(&mut *head, CStr::from_ptr(id.cast()).to_bytes())
        .0
        .map_or(std::ptr::null_mut(), |t| t.as_ptr());
    if !tag.is_null() && !(*tag).str_.is_null() && (*tag).len >= 3 {
        let value = (*tag).str_.add(3);
        let k = kh_get_m_s2i(rh, value);
        if k == (*rh).n_buckets {
            let to_remove = head;
            head = (*head).next;
            ret |= header_c_704_sam_hrecs_remove_line(hrecs, type_, to_remove, 0);
        }
    }
    let _ = head; // suppress unused warning if not consulted again

    let t0 = *type_ as u8;
    let t1 = *type_.add(1) as u8;
    if ((t0 == b'S' && t1 == b'Q') || (t0 == b'R' && t1 == b'G'))
        && rebuild_hash(hrecs, header_h_58_TYPEKEY(type_)) != 0
    {
        return -1;
    }

    if ret == 0 && (*hrecs).dirty != 0 {
        redact_header_text(&mut *bh);
    }

    ret
}

// htslib/header.c:2346
// Faithful 1:1 translation of `sam_hdr_remove_tag_id`, hrecs-mode body only.
// Removes a single tag from the line identified by `(type, ID_key, ID_value)`.
pub(crate) unsafe fn header_c_2346_sam_hdr_remove_tag_id_hrecs(
    bh: *mut sam_hdr_t,
    type_: *const u8,
    id_key: *const u8,
    id_value: *const u8,
    key: *const u8,
) -> i32 {
    if bh.is_null() || type_.is_null() || key.is_null() {
        return -1;
    }

    if (*bh).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *bh) != 0 {
        return -1;
    }
    let hrecs = (*bh).hrecs;

    let id = if id_key.is_null() {
        None
    } else {
        Some((CStr::from_ptr(id_key.cast()).to_bytes(), CStr::from_ptr(id_value.cast()).to_bytes()))
    };
    let ty = sam_hrecs_find_type_id(&mut *hrecs, CStr::from_ptr(type_.cast()).to_bytes(), id);
    let ty = match ty {
        None => return -1,
        Some(t) => t.as_ptr(),
    };

    let ret = sam_hrecs_remove_key(&mut *hrecs, &mut *ty, CStr::from_ptr(key.cast()).to_bytes());
    if ret == 0 && (*hrecs).dirty != 0 {
        redact_header_text(&mut *bh);
    }

    ret
}

// htslib/header.c:2562
// Faithful 1:1 translation of `sam_hdr_pg_id`, hrecs-mode body only.
// Returns a pointer to a unique PG `ID` candidate.  If @p name is already
// free the caller's pointer is returned unchanged; otherwise a clashing
// suffix `name.N` is generated in `hrecs->ID_buf` (a `realloc`-backed
// scratch buffer owned by the hrecs).
pub(crate) unsafe fn header_c_2562_sam_hdr_pg_id_hrecs(
    bh: *mut sam_hdr_t,
    name: *const u8,
) -> *const u8 {
    let name_extra: usize = 17;

    if bh.is_null() || name.is_null() {
        return std::ptr::null();
    }

    if (*bh).hrecs.is_null() && sam_hdr_fill_hrecs(&mut *bh) != 0 {
        return std::ptr::null();
    }
    let hrecs = (*bh).hrecs;

    let pg_hash = (*hrecs).pg_hash.cast::<khash_m_s2i_t>();
    let k = kh_get_m_s2i(pg_hash, name);
    if k == (*pg_hash).n_buckets {
        return name;
    }

    let mut name_len = libc::strlen(name.cast());
    if name_len > 1000 {
        name_len = 1000;
    }

    // Saturating add (hts_add_sat2): on overflow we'd return SIZE_MAX which
    // realloc would refuse — short-circuit to NULL like the C does after the
    // realloc fails.
    let needed = match name_len.checked_add(name_extra) {
        Some(v) => v,
        None => return std::ptr::null(),
    };
    if ((*hrecs).ID_buf_sz as usize) < needed {
        let old_sz = (*hrecs).ID_buf_sz as usize;
        let mut id_buf_vec = if (*hrecs).ID_buf.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*hrecs).ID_buf, old_sz, old_sz)
        };
        id_buf_vec.resize(needed, 0u8);
        (*hrecs).ID_buf = id_buf_vec.as_mut_ptr();
        std::mem::forget(id_buf_vec);
        (*hrecs).ID_buf_sz = needed as u32;
    }

    // Take a bounded copy of name into a stack buffer so we can pass it to
    // libc::snprintf safely; the C uses "%.1000s.%d" which limits to 1000
    // chars from name; we replicate that bound here in Rust.
    loop {
        let written = libc::snprintf(
            (*hrecs).ID_buf.cast(),
            (*hrecs).ID_buf_sz as usize,
            c"%.1000s.%d".as_ptr(),
            name.cast::<i8>(),
            (*hrecs).ID_cnt,
        );
        (*hrecs).ID_cnt += 1;
        if written < 0 {
            return std::ptr::null();
        }
        let k = kh_get_m_s2i(pg_hash, (*hrecs).ID_buf);
        if k == (*pg_hash).n_buckets {
            break;
        }
    }

    (*hrecs).ID_buf
}

// htslib/sam.c:170
// Faithful 1:1 translation of `sam_hdr_dup`, hrecs-mode body only.  Rebuilds
// the serialized text from the source `hrecs` (via `sam_hrecs_rebuild_text`)
// and populates the new header's `target_name` / `target_len` / `sdict`
// arrays from the source's `hrecs->ref[]` table — matching the work that the
// C `sam_hdr_update_target_arrays(bh, h0->hrecs, 0)` call performs.  The
// new header's own `hrecs` field is left NULL, exactly as in the C.
pub(crate) unsafe fn sam_c_170_sam_hdr_dup_hrecs(h0: *const sam_hdr_t, h: *mut sam_hdr_t) -> i32 {
    let mut tmp = kstring_t::default();
    if sam_hrecs_rebuild_text(&*(*h0).hrecs, &mut tmp) != 0 {
        ks_free(&mut tmp);
        return -1;
    }

    let bytes = ks_release(&mut tmp);
    (*h).l_text = bytes.len();
    // (*h).text is a Vec-owned NUL-terminated buffer (reconstructed and dropped
    // on teardown); leak the owned bytes plus a trailing NUL at this boundary.
    let mut text_vec = bytes;
    text_vec.push(0);
    (*h).text = text_vec.as_mut_ptr();
    std::mem::forget(text_vec);

    // Replicate sam_hdr_update_target_arrays(h, h0->hrecs, 0).  We use the
    // existing native target-array primitives (`sam_hdr_clear_targets` +
    // `sam_hdr_append_target`) — the same pair `rebuild_target_arrays`
    // (sam.rs:3674) uses against `h->hrecs`.  That avoids translating the
    // full C function while preserving the post-condition: `h->n_targets`,
    // `h->target_name[]`, `h->target_len[]` and `h->sdict` are populated to
    // match `h0->hrecs->ref[]`.
    let src_hrecs = (*h0).hrecs;
    sam_hdr_clear_targets(h);
    for i in 0..(*src_hrecs).nref {
        let r = (*src_hrecs).ref_.add(i as usize);
        if (*r).name.is_null() {
            return -1;
        }
        if sam_hdr_append_target(h, CStr::from_ptr((*r).name.cast()).to_bytes(), (*r).len) < 0 {
            return -1;
        }
    }

    0
}

// htslib/sam.c:2157
// Faithful 1:1 translation of `sam_hdr_change_HD`, hrecs-mode body only.
// Routes to `sam_hdr_update_line("HD", NULL, NULL, key, val, NULL)` to set,
// or `sam_hdr_remove_tag_id("HD", NULL, NULL, key)` to clear, then forces a
// text rebuild via `sam_hdr_rebuild`.
pub(crate) unsafe fn sam_c_2157_sam_hdr_change_HD_hrecs(
    h: *mut sam_hdr_t,
    key: *const u8,
    val: *const u8,
) -> i32 {
    if !val.is_null() {
        // Add a fresh @HD line if none exists yet (matches the text-mode
        // behavior in htslib's old_sam_hdr_change_HD). update_line on a
        // missing HD record returns -1 in the hrecs path; without this branch
        // callers would see a regression vs the text-mode entry-point.
        let hrecs = (*h).hrecs;
        let hd = sam_hrecs_find_type_id(&mut *hrecs, b"HD", None);
        let key_b = CStr::from_ptr(key.cast()).to_bytes();
        let val_b = CStr::from_ptr(val.cast()).to_bytes();
        if hd.is_none() {
            if sam_hdr_add_line(
                &mut *h,
                b"HD",
                &[(Some(b"VN"), Some(b"1.6")), (Some(key_b), Some(val_b))],
            ) != 0
            {
                return -1;
            }
        } else if sam_hdr_update_line(&mut *h, b"HD", None, &[(Some(key_b), Some(val_b))]) != 0 {
            return -1;
        }
    } else if sam_hdr_remove_tag_id(&mut *h, b"HD", None, CStr::from_ptr(key.cast()).to_bytes()) < 0
    {
        // sam_hdr_remove_tag_id returns 1 on actual removal and 0 on no-op
        // (mirroring htslib); both are success. Only -1 (find_type_id failed
        // or null inputs) is an error.
        return -1;
    }
    sam_hdr_rebuild(&mut *h)
}

pub(crate) unsafe fn sam_c_144_sam_hdr_dup_sdict(h0: *const sam_hdr_t, h: *mut sam_hdr_t) -> i32 {
    let src_long_refs = (*h0).sdict.cast::<khash_s2i_t>();
    let dest_long_refs =
        Box::into_raw(Box::new(std::mem::zeroed::<khash_s2i_t>())).cast::<()>()
            .cast::<khash_s2i_t>();
    if dest_long_refs.is_null() {
        return -1;
    }

    let mut n_long = 0u32;
    for i in 0..(*h).n_targets {
        if *(*h).target_len.add(i as usize) == u32::MAX {
            n_long += 1;
        }
    }

    if n_long != 0 {
        let mut n_buckets = 4u32;
        while n_buckets < n_long.saturating_mul(2) {
            n_buckets <<= 1;
        }
        let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
        (*dest_long_refs).flags =
            { let mut v: Vec<u32> = vec![0; n_flags as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
        (*dest_long_refs).keys = {
            let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); n_buckets as usize];
            let p = v.as_mut_ptr();
            std::mem::forget(v);
            p
        };
        (*dest_long_refs).vals = {
            let mut v: Vec<i64> = vec![0i64; n_buckets as usize];
            let p = v.as_mut_ptr();
            std::mem::forget(v);
            p
        };
        if (*dest_long_refs).flags.is_null()
            || (*dest_long_refs).keys.is_null()
            || (*dest_long_refs).vals.is_null()
        {
            kh_destroy_s2i(dest_long_refs);
            return -1;
        }
        for i in 0..n_flags {
            *(*dest_long_refs).flags.add(i as usize) = 0xaaaa_aaaa;
        }
        (*dest_long_refs).n_buckets = n_buckets;
        (*dest_long_refs).upper_bound = (n_buckets as f64 * 0.77) as u32;
    }

    for i in 0..(*h).n_targets {
        if *(*h).target_len.add(i as usize) < u32::MAX {
            continue;
        }
        let key = *(*h).target_name.add(i as usize);
        let ksrc = kh_get_s2i(src_long_refs, key);
        if ksrc == (*src_long_refs).n_buckets {
            continue;
        }

        let mask = (*dest_long_refs).n_buckets - 1;
        let mut kdest = __ac_FNV1a_hash_string(key.cast()) & mask;
        let mut step = 0;
        while !kh_isempty((*dest_long_refs).flags, kdest) {
            if !kh_isdel((*dest_long_refs).flags, kdest)
                && cstr_eq(*(*dest_long_refs).keys.add(kdest as usize), key)
            {
                break;
            }
            step += 1;
            kdest = (kdest + step) & mask;
        }
        if kh_iseither((*dest_long_refs).flags, kdest) {
            if kh_isempty((*dest_long_refs).flags, kdest) {
                (*dest_long_refs).n_occupied += 1;
            }
            kh_set_occupied((*dest_long_refs).flags, kdest);
            (*dest_long_refs).size += 1;
            *(*dest_long_refs).keys.add(kdest as usize) = key;
        }
        *(*dest_long_refs).vals.add(kdest as usize) = *(*src_long_refs).vals.add(ksrc as usize);
    }

    (*h).sdict = dest_long_refs.cast();
    0
}

pub unsafe fn bam_hdr_init() -> *mut sam_hdr_t {
    sam_hdr_init()
}

pub unsafe fn bam_hdr_destroy(_h: *mut sam_hdr_t) {
    sam_hdr_destroy(_h);
}

pub unsafe fn bam_hdr_dup(_h0: *const sam_hdr_t) -> *mut sam_hdr_t {
    sam_hdr_dup(_h0)
}

pub unsafe fn bam_hdr_read(_fp: *mut BGZF) -> *mut sam_hdr_t {
    let _ = bgzf_check_EOF(_fp.cast());
    let mut buf = [0u8; 4];
    let mut bytes = bgzf_read(_fp.cast(), buf.as_mut_ptr().cast(), 4);
    if bytes != 4 || &buf != b"BAM\x01" {
        return std::ptr::null_mut();
    }

    let h = sam_hdr_init();
    if h.is_null() {
        return std::ptr::null_mut();
    }

    let mut num_names = 0;
    bytes = bgzf_read(_fp.cast(), buf.as_mut_ptr().cast(), 4);
    if bytes != 4 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    (*h).l_text = u32::from_le_bytes(buf) as usize;

    let bufsize = (*h).l_text.wrapping_add(1);
    if bufsize < (*h).l_text {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    (*h).text = ({ let mut v: Vec<u8> = vec![0u8; (bufsize) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
    if (*h).text.is_null() {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    *(*h).text.add((*h).l_text) = 0;
    bytes = bgzf_read(_fp.cast(), (*h).text.cast(), (*h).l_text as _);
    if bytes != (*h).l_text as _ {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }

    bytes = bgzf_read(_fp.cast(), buf.as_mut_ptr().cast(), 4);
    if bytes != 4 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    (*h).n_targets = i32::from_le_bytes(buf);
    if (*h).n_targets < 0 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }

    if (*h).n_targets > 0 {
        (*h).target_name = ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut::<u8>(); (*h).n_targets as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
        .cast();
        if (*h).target_name.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        (*h).target_len = ({ let mut v: Vec<u32> = vec![0u32; (*h).n_targets as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
        .cast();
        if (*h).target_len.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
    }

    let mut i = 0;
    while i != (*h).n_targets {
        bytes = bgzf_read(_fp.cast(), buf.as_mut_ptr().cast(), 4);
        if bytes != 4 {
            (*h).n_targets = num_names;
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        let name_len = i32::from_le_bytes(buf);
        if name_len <= 0 {
            (*h).n_targets = num_names;
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }

        *(*h).target_name.add(i as usize) =
            ({ let mut v: Vec<u8> = vec![0u8; (name_len) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
        if (*(*h).target_name.add(i as usize)).is_null() {
            (*h).n_targets = num_names;
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        num_names += 1;

        bytes = bgzf_read(
            _fp.cast(),
            (*(*h).target_name.add(i as usize)).cast(),
            name_len as _,
        );
        if bytes != name_len as _ {
            (*h).n_targets = num_names;
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }

        if *(*(*h).target_name.add(i as usize)).add(name_len as usize - 1) != 0 {
            if name_len == i32::MAX {
                (*h).n_targets = num_names;
                sam_hdr_destroy(h);
                return std::ptr::null_mut();
            }
            let mut name_vec = Vec::from_raw_parts(
                *(*h).target_name.add(i as usize),
                name_len as usize,
                name_len as usize,
            );
            name_vec.resize(name_len as usize + 1, 0u8);
            let new_name = name_vec.as_mut_ptr();
            std::mem::forget(name_vec);
            *(*h).target_name.add(i as usize) = new_name;
            *new_name.add(name_len as usize) = 0;
        }

        bytes = bgzf_read(_fp.cast(), buf.as_mut_ptr().cast(), 4);
        if bytes != 4 {
            (*h).n_targets = num_names;
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        *(*h).target_len.add(i as usize) = u32::from_le_bytes(buf);
        i += 1;
    }

    h
}

pub unsafe fn bam_hdr_write(fp: *mut BGZF, h: *const sam_hdr_t) -> i32 {
    if h.is_null() {
        return -1;
    }
    // Rust-built hrecs is the source of truth — sync cached text and fall
    // through to the native BAM writer. Production never produces unmarked
    // hrecs (cram_dopen's C-pool header is dup'd in sam_hdr_read into a
    // hrecs-null copy); a caller that somehow gets one falls through using
    // the existing (*h).text.
    if !(*h).hrecs.is_null()
        && sam_hdr_has_rust_hrecs(h.cast_mut())
        && sam_hdr_rebuild(&mut *h.cast_mut()) < 0
    {
        return -1;
    }
    if (*h).l_text > u32::MAX as usize {
        return -1;
    }

    if bgzf_write(fp, c"BAM\x01".as_ptr().cast(), 4) < 0 {
        return -1;
    }

    let is_be = ((*fp).bitfields & (1 << 19)) != 0;
    let mut l_text = (*h).l_text as u32;
    if is_be {
        ed_swap_4p((&mut l_text as *mut u32).cast());
    }
    if bgzf_write(fp, (&l_text as *const u32).cast(), 4) < 0 {
        return -1;
    }
    if (*h).l_text != 0 && bgzf_write(fp, (*h).text.cast(), (*h).l_text) < 0 {
        return -1;
    }
    let mut n_targets = (*h).n_targets;
    if is_be {
        ed_swap_4p((&mut n_targets as *mut i32).cast());
    }
    if bgzf_write(fp, (&n_targets as *const i32).cast(), 4) < 0 {
        return -1;
    }

    for i in 0..(*h).n_targets {
        let p = *(*h).target_name.add(i as usize);
        let mut name_len = libc::strlen(p.cast()) as i32 + 1;
        if is_be {
            ed_swap_4p((&mut name_len as *mut i32).cast());
        }
        if bgzf_write(fp, (&name_len as *const i32).cast(), 4) < 0 {
            return -1;
        }
        let write_name_len = libc::strlen(p.cast()) + 1;
        if bgzf_write(fp, p.cast(), write_name_len) < 0 {
            return -1;
        }
        let mut target_len = *(*h).target_len.add(i as usize);
        if is_be {
            ed_swap_4p((&mut target_len as *mut u32).cast());
        }
        if bgzf_write(fp, (&target_len as *const u32).cast(), 4) < 0 {
            return -1;
        }
    }
    if bgzf_flush(fp) < 0 {
        return -1;
    }
    0
}

pub(crate) unsafe fn sam_hdr_write_cram(fp: *mut htsFile, h: *const sam_hdr_t) -> i32 {
    // Rust-built hrecs is the source of truth — sync text from it so the
    // CRAM writer's text consumers see the up-to-date header.
    if !(*h).hrecs.is_null()
        && sam_hdr_has_rust_hrecs(h.cast_mut())
        && sam_hdr_rebuild(&mut *h.cast_mut()) < 0
    {
        return -1;
    }

    // Mirror libhts' sam_hdr_write CRAM wrapper sequence:
    //   cram_set_header2(fd, h) — dup the header and run refs_from_header
    //   cram_load_reference(fd, fd->ref_fn) — load any pending reference
    //   cram_write_SAM_hdr(fd, fd->header) — write into a CRAM container
    let cram_fd = (*fp).fp.cram;
    if crate::htslib_rs::cram::cram_cram_io_c_2866_cram_set_header(cram_fd, h.cast_mut()) != 0 {
        return -1;
    }
    let ref_fn = crate::htslib_rs::cram::cram_fd_ref_fn(cram_fd);
    if !ref_fn.is_null()
        && crate::htslib_rs::cram::cram_cram_io_c_3597_cram_load_reference(cram_fd, ref_fn.cast()) != 0
    {
        return -1;
    }
    let hdr_native = crate::htslib_rs::cram::cram_fd_header_ptr(cram_fd).cast::<sam_hdr_t>();
    crate::htslib_rs::cram::cram_cram_io_c_4889_cram_write_SAM_hdr(cram_fd, hdr_native)
}

/// Strip lines whose content is just `@CO` followed only by trailing
/// whitespace before the newline (e.g. `@CO\n`, `@CO\r\n`, `@CO \t\n`).
/// v1.23 `sam_hdr_parse` rejects such lines as malformed; they carry no
/// comment data and are safe to drop before re-parsing for the CRAM write.
fn strip_bare_comment_lines(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        // Find end of current line (inclusive of the newline if present).
        let line_end = text[i..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| i + p + 1)
            .unwrap_or(text.len());
        let line = &text[i..line_end];
        // A line is "bare comment" if it starts with `@CO`, then only
        // optional spaces/tabs/CR before \n (or end of text).
        let is_bare = line.starts_with(b"@CO")
            && line[3..]
                .iter()
                .all(|&b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'));
        if !is_bare {
            out.extend_from_slice(line);
        }
        i = line_end;
    }
    out
}

pub(crate) unsafe fn sam_hdr_write_bytes(
    fp: *mut htsFile,
    bytes: *const (),
    len: usize,
) -> i32 {
    if len == 0 {
        return 0;
    }
    if ((*fp).bitfields & (1 << 4)) != 0 {
        if bgzf_write((*fp).fp.bgzf, bytes.cast(), len) == len as isize {
            0
        } else {
            -1
        }
    } else if crate::htslib_rs::hfile::htslib_hfile_h_292_hwrite((*fp).fp.hfile, bytes.cast(), len)
        == len as isize
    {
        0
    } else {
        -1
    }
}

pub(crate) unsafe fn sam_hdr_write_store_copy(fp: *mut htsFile, h: *const sam_hdr_t) -> i32 {
    let tmp = (*fp).bam_header.cast::<sam_hdr_t>();
    (*fp).bam_header = sam_hdr_dup(h).cast();
    sam_hdr_destroy(tmp);
    if (*fp).bam_header.is_null() {
        -1
    } else {
        0
    }
}

pub(crate) unsafe fn sam_hdr_sanitise(_h: *mut sam_hdr_t) -> *mut sam_hdr_t {
    if _h.is_null() {
        return std::ptr::null_mut();
    }
    if (*_h).l_text == 0 {
        return _h;
    }

    let mut i = 0usize;
    let mut last = b'\n' as u8;
    while i < (*_h).l_text {
        let ch = *(*_h).text.add(i);
        if ch == 0 {
            break;
        }
        if last == b'\n' as u8 && ch != b'@' as u8 {
            sam_hdr_destroy(_h);
            return std::ptr::null_mut();
        }
        last = ch;
        i += 1;
    }

    if last != b'\n' as u8 {
        if (*_h).l_text < 2 || i >= (*_h).l_text - 2 {
            if (*_h).l_text >= usize::MAX - 2 {
                sam_hdr_destroy(_h);
                return std::ptr::null_mut();
            }
            let old_cap = (*_h).l_text + 1;
            let mut text_vec = if (*_h).text.is_null() {
                Vec::new()
            } else {
                Vec::from_raw_parts((*_h).text, old_cap, old_cap)
            };
            text_vec.resize((*_h).l_text + 2, 0u8);
            (*_h).text = text_vec.as_mut_ptr();
            std::mem::forget(text_vec);
        }
        *(*_h).text.add(i) = b'\n' as u8;
        i += 1;
        if (*_h).l_text < i {
            (*_h).l_text = i;
        }
        *(*_h).text.add((*_h).l_text) = 0;
    }

    _h
}

pub(crate) unsafe fn sam_c_1907_sam_hdr_create(fp: *mut htsFile) -> *mut sam_hdr_t {
    let h = sam_hdr_init();
    if h.is_null() {
        return std::ptr::null_mut();
    }

    loop {
        let next_c = if ((*fp).bitfields & (1 << 4)) != 0 {
            bgzf_peek((*fp).fp.bgzf.cast())
        } else {
            let mut nc = 0u8;
            let pret = hpeek((*fp).fp.hfile, (&mut nc as *mut u8).cast(), 1);
            if pret > 0 {
                nc as i32
            } else {
                pret as i32 - 1
            }
        };
        if next_c != b'@' as i32 {
            (*fp).line.data.truncate(0);
            break;
        }

        let ret = crate::htslib_rs::hts::hts_getline(fp, 2, &mut (*fp).line);
        if ret < 0 {
            if ret < -1 {
                sam_hdr_destroy(h);
                return std::ptr::null_mut();
            }
            break;
        }
        let line_data = &(*fp).line.data;
        if line_data.is_empty() || line_data[0] != b'@' {
            (*fp).line.data.truncate(0);
            break;
        }
        // Append raw line text to (*h).text without filling hrecs. Lazy
        // hrecs population — by the first Rust or C mutation — is the
        // mechanism the test/runtime relies on to route subsequent
        // reads/writes through the correct (matching-allocator) library.
        // Going through sam_hdr_add_lines here would eagerly fill a
        // Rust-built hrecs, which then breaks downstream hts_sys mutators
        // in test::sam helpers that operate on the same header.
        if sam_hdr_append_text_raw(h, (*fp).line.data.as_ptr().cast(), (*fp).line.data.len()) != 0
            || sam_hdr_append_text_raw(h, b"\n".as_ptr().cast(), 1) != 0
        {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
    }

    // Fill the target arrays from the accumulated text without populating
    // hrecs — keeps the header in text-only mode so the first mutator
    // (Rust or hts_sys) gets to decide who owns hrecs.
    if sam_hdr_fill_targets_from_text(h) < 0 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    if !(*fp).bam_header.is_null() {
        sam_hdr_destroy((*fp).bam_header.cast());
    }
    (*fp).bam_header = sam_hdr_sanitise(h).cast();
    if (*fp).bam_header.is_null() {
        return std::ptr::null_mut();
    }
    (*(*fp).bam_header.cast::<sam_hdr_t>()).ref_count = 1;
    (*fp).bam_header.cast()
}

pub(crate) unsafe fn sam_c_2080_old_sam_hdr_change_HD(
    h: *mut sam_hdr_t,
    key: *const u8,
    val: *const u8,
) -> i32 {
    if h.is_null() || key.is_null() {
        return -1;
    }

    let key_bytes = CStr::from_ptr(key.cast()).to_bytes();
    let val_bytes = if val.is_null() {
        None
    } else {
        Some(CStr::from_ptr(val.cast()).to_bytes())
    };
    let old_text = if !(*h).text.is_null() && (*h).l_text > 0 {
        std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text)
    } else {
        &[]
    };

    let mut beg: Option<usize> = None;
    let mut end: usize = 0;
    if (*h).l_text > 3 && old_text.len() >= 3 && &old_text[..3] == b"@HD" {
        let Some(newline) = old_text.iter().position(|&c| c == b'\n') else {
            return -1;
        };
        let hd_line = &old_text[..newline];
        let mut tmp = Vec::with_capacity(4);
        tmp.push(b'\t');
        if let Some(&c) = key_bytes.first() {
            tmp.push(c);
        }
        if key_bytes.len() > 1 {
            tmp.push(key_bytes[1]);
            tmp.push(b':');
        }

        if let Some(q) = hd_line.windows(tmp.len()).position(|w| w == tmp.as_slice()) {
            beg = Some(q);
            let mut e = q + 4;
            while e < old_text.len() && old_text[e] != b'\n' && old_text[e] != b'\t' {
                e += 1;
            }
            end = e;
            if let Some(v) = val_bytes {
                let old_val = if q + 4 <= end {
                    &old_text[q + 4..end]
                } else {
                    &[]
                };
                if old_val == v {
                    return 0;
                }
            }
        } else {
            beg = Some(newline);
            end = newline;
        }
    }

    let mut new_text = Vec::<u8>::new();
    if let Some(b) = beg {
        new_text.extend_from_slice(&old_text[..b]);
        if let Some(v) = val_bytes {
            new_text.push(b'\t');
            new_text.extend_from_slice(key_bytes);
            new_text.push(b':');
            new_text.extend_from_slice(v);
        }
        new_text.extend_from_slice(&old_text[end..]);
    } else {
        new_text.extend_from_slice(b"@HD\tVN:");
        new_text.extend_from_slice(SAM_FORMAT_VERSION.as_bytes());
        if let Some(v) = val_bytes {
            new_text.push(b'\t');
            new_text.extend_from_slice(key_bytes);
            new_text.push(b':');
            new_text.extend_from_slice(v);
        }
        new_text.push(b'\n');
        new_text.extend_from_slice(old_text);
    }

    let new_len = new_text.len();
    let mut new_vec = new_text;
    new_vec.push(0);
    let newtext = new_vec.as_mut_ptr();
    std::mem::forget(new_vec);
    if !(*h).text.is_null() {
        drop(Vec::from_raw_parts(
            (*h).text,
            (*h).l_text + 1,
            (*h).l_text + 1,
        ));
    }
    (*h).text = newtext;
    (*h).l_text = new_len;
    0
}

unsafe fn sam_c_1173_bam_get_library(h: *const sam_hdr_t, b: *const bam1_t) -> *const u8 {
    // Concurrency fix vs the htslib v1.23 C baseline (htslib/sam.c::bam_get_library):
    //   The C original uses a function-static `char lb_text[1024]` and returns
    //   a pointer into it.  htslib documents this routine as non-reentrant
    //   because two threads decoding different records would race on the
    //   shared buffer (one thread's library name can be observed mid-write or
    //   overwritten by another thread before the caller consumes it).  In
    //   Rust we can do better at zero cost: each thread gets its own 1024B
    //   buffer via a `thread_local!`, so concurrent callers on distinct
    //   threads never alias.  The returned `*const u8` aliases the
    //   per-thread buffer and is therefore only valid while the calling
    //   thread is alive AND only on the thread that produced it.  In practice
    //   the sole production call site (`sam_format_aux` library handler,
    //   sam.rs:6207) consumes the pointer with `kputs` on the same thread
    //   immediately, which matches this invariant.  The public function
    //   signature (`unsafe fn ... -> *const u8`) is unchanged.
    thread_local! {
        static LB_TEXT: std::cell::UnsafeCell<[u8; 1024]> = const {
            std::cell::UnsafeCell::new([0; 1024])
        };
    }

    if h.is_null() {
        return std::ptr::null();
    }
    let mut lib = kstring_t::default();
    let rg = bam_aux_get(b, c"RG".as_ptr().cast());
    if rg.is_null() {
        return std::ptr::null();
    }

    // Use the native wrapper rather than hts_sys: calling the C function on a
    // header with no hrecs makes the C library build a C-owned hrecs into our
    // struct, which our allocator then frees incorrectly on destroy.
    if sam_hdr_find_tag_id(
        &mut *h.cast_mut(),
        b"RG",
        Some((b"ID", CStr::from_ptr(rg.add(1).cast()).to_bytes())),
        b"LB",
        &mut lib,
    ) < 0
    {
        return std::ptr::null();
    }

    let len = if lib.data.len() < 1023 {
        lib.data.len()
    } else {
        1023
    };
    // Obtain a `*mut u8` to this thread's private buffer.  `UnsafeCell::get`
    // returns a raw pointer whose validity is tied to the thread-local's
    // storage, which lives until thread exit, so the pointer remains valid
    // beyond the `with` closure for any same-thread use that follows.
    let lb_text = LB_TEXT.with(|cell| cell.get().cast::<u8>());
    if len > 0 {
        std::ptr::copy_nonoverlapping(lib.data.as_ptr().cast::<u8>(), lb_text, len);
    }
    *lb_text.add(len) = 0;
    lb_text.cast_const()
}

unsafe fn sam_c_2221_grow_B_array(b: *mut bam1_t, n: *mut u32, size: usize) -> i64 {
    if *n > (i32::MAX as f64 * 0.666) as u32 {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }

    let bytes = size.wrapping_mul((*n >> 1) as usize);
    if possibly_expand_bam_data(b, bytes) < 0 {
        return -1;
    }
    *n += *n >> 1;
    0
}

unsafe fn sam_c_2244_sam_parse_Bc_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 1) < 0 {
            return std::ptr::null_mut();
        }
        let v = hts_str2int(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 8, overflow) as u8;
        let off = (*b).data.len();
        if realloc_bam_data(b, off + 1) < 0 {
            return std::ptr::null_mut();
        }
        *(*b).data.as_mut_ptr().add(off) = v;
    }
    q
}

unsafe fn sam_c_2258_sam_parse_BC_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 1) < 0 {
            return std::ptr::null_mut();
        }
        if *q.add(1) != b'-' as u8 {
            let v = hts_str2uint(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 8, overflow) as u8;
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 1) < 0 {
                return std::ptr::null_mut();
            }
            *(*b).data.as_mut_ptr().add(off) = v;
        } else {
            *overflow = 1;
            q = q.add(1);
            while *q > b'\t' as u8 && *q != b',' as u8 {
                q = q.add(1);
            }
        }
    }
    q
}

unsafe fn sam_c_2278_sam_parse_Bs_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 2) < 0 {
            return std::ptr::null_mut();
        }
        let v = hts_str2int(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 16, overflow) as i16;
        let off = (*b).data.len();
        if realloc_bam_data(b, off + 2) < 0 {
            return std::ptr::null_mut();
        }
        i16_to_le(v, (*b).data.as_mut_ptr().add(off));
    }
    q
}

unsafe fn sam_c_2293_sam_parse_BS_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 2) < 0 {
            return std::ptr::null_mut();
        }
        if *q.add(1) != b'-' as u8 {
            let v = hts_str2uint(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 16, overflow) as u16;
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 2) < 0 {
                return std::ptr::null_mut();
            }
            u16_to_le(v, (*b).data.as_mut_ptr().add(off));
        } else {
            *overflow = 1;
            q = q.add(1);
            while *q > b'\t' as u8 && *q != b',' as u8 {
                q = q.add(1);
            }
        }
    }
    q
}

unsafe fn sam_c_2314_sam_parse_Bi_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 4) < 0 {
            return std::ptr::null_mut();
        }
        let v = hts_str2int(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 32, overflow) as i32;
        let off = (*b).data.len();
        if realloc_bam_data(b, off + 4) < 0 {
            return std::ptr::null_mut();
        }
        i32_to_le(v, (*b).data.as_mut_ptr().add(off));
    }
    q
}

unsafe fn sam_c_2329_sam_parse_BI_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 4) < 0 {
            return std::ptr::null_mut();
        }
        if *q.add(1) != b'-' as u8 {
            let v = hts_str2uint(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 32, overflow) as u32;
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 4) < 0 {
                return std::ptr::null_mut();
            }
            u32_to_le(v, (*b).data.as_mut_ptr().add(off));
        } else {
            *overflow = 1;
            q = q.add(1);
            while *q > b'\t' as u8 && *q != b',' as u8 {
                q = q.add(1);
            }
        }
    }
    q
}

unsafe fn sam_c_2350_sam_parse_Bf_vals(
    b: *mut bam1_t,
    mut q: *mut u8,
    nused: *mut u32,
    nalloc: *mut u32,
    _overflow: *mut i32,
) -> *mut u8 {
    while *q == b',' as u8 {
        *nused += 1;
        if *nused > *nalloc && sam_c_2221_grow_B_array(b, nalloc, 4) < 0 {
            return std::ptr::null_mut();
        }
        let mut end: *mut u8 = std::ptr::null_mut();
        let val = libc::strtod(q.add(1).cast(), (&mut end as *mut *mut u8).cast());
        q = end;
        let off = (*b).data.len();
        if realloc_bam_data(b, off + 4) < 0 {
            return std::ptr::null_mut();
        }
        float_to_le(val as f32, (*b).data.as_mut_ptr().add(off));
    }
    q
}

unsafe fn sam_c_2364_sam_parse_B_vals_r(
    type_: u8,
    mut nalloc: u32,
    in_: *mut u8,
    end: *mut *mut u8,
    b: *mut bam1_t,
    ctr: *mut i32,
) -> i32 {
    *ctr += 1;
    if *ctr > 2 {
        return -1;
    }

    let orig_l = (*b).data.len() as i32;
    let mut q = in_;
    let size = aux_type2size(type_ as u8);
    if size <= 0 || size > 4 {
        return -1;
    }

    if nalloc == 0 {
        nalloc = 7;
    }
    let bytes = (nalloc as usize).wrapping_mul(size as usize);
    if bytes / size as usize != nalloc as usize
        || possibly_expand_bam_data(b, bytes + 2 + std::mem::size_of::<u32>()) != 0
    {
        return -1;
    }

    let mut nused = 0u32;
    let hdr_off = (*b).data.len();
    if realloc_bam_data(b, hdr_off + 2 + std::mem::size_of::<u32>()) < 0 {
        return -1;
    }
    *(*b).data.as_mut_ptr().add(hdr_off) = b'B';
    *(*b).data.as_mut_ptr().add(hdr_off + 1) = type_ as u8;
    let b_len_idx = (hdr_off + 2) as i32;

    let mut overflow = 0;
    q = match type_ as u8 {
        b'c' => sam_c_2244_sam_parse_Bc_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        b'C' => sam_c_2258_sam_parse_BC_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        b's' => sam_c_2278_sam_parse_Bs_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        b'S' => sam_c_2293_sam_parse_BS_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        b'i' => sam_c_2314_sam_parse_Bi_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        b'I' => sam_c_2329_sam_parse_BI_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        b'f' => sam_c_2350_sam_parse_Bf_vals(b, q, &mut nused, &mut nalloc, &mut overflow),
        _ => q,
    };
    if q.is_null() {
        return -1;
    }
    if *q != b'\t' as u8 && *q != 0 {
        return -1;
    }
    i32_to_le(nused as i32, (*b).data.as_mut_ptr().add(b_len_idx as usize));

    if overflow == 0 {
        *end = q;
        return 0;
    }

    let r = q;
    q = in_;
    overflow = 0;
    (*b).data.truncate(orig_l as usize);
    let mut max = 0i64;
    let mut min = 0i64;
    while q < r {
        let val = hts_str2int(q.add(1).cast(), (&mut q as *mut *mut u8).cast(), 64, &mut overflow);
        if max < val {
            max = val;
        }
        if min > val {
            min = val;
        }
        while *q > b'\t' as u8 && *q != b',' as u8 {
            q = q.add(1);
        }
    }

    if overflow == 0 {
        if min < 0 {
            if min >= i8::MIN as i64 && max <= i8::MAX as i64 {
                return sam_c_2364_sam_parse_B_vals_r(b'c' as u8, nalloc, in_, end, b, ctr);
            } else if min >= i16::MIN as i64 && max <= i16::MAX as i64 {
                return sam_c_2364_sam_parse_B_vals_r(b's' as u8, nalloc, in_, end, b, ctr);
            } else if min >= i32::MIN as i64 && max <= i32::MAX as i64 {
                return sam_c_2364_sam_parse_B_vals_r(b'i' as u8, nalloc, in_, end, b, ctr);
            }
        } else if max < u8::MAX as i64 {
            return sam_c_2364_sam_parse_B_vals_r(b'C' as u8, nalloc, in_, end, b, ctr);
        } else if max <= u16::MAX as i64 {
            return sam_c_2364_sam_parse_B_vals_r(b'S' as u8, nalloc, in_, end, b, ctr);
        } else if max <= u32::MAX as i64 {
            return sam_c_2364_sam_parse_B_vals_r(b'I' as u8, nalloc, in_, end, b, ctr);
        }
    }
    -1
}

unsafe fn sam_c_2490_sam_parse_B_vals(
    type_: u8,
    in_: *mut u8,
    end: *mut *mut u8,
    b: *mut bam1_t,
) -> i32 {
    let mut ctr = 0;
    sam_c_2364_sam_parse_B_vals_r(type_, 0, in_, end, b, &mut ctr)
}

unsafe fn sam_c_2498_parse_sam_flag(
    v: *mut u8,
    rv: *mut *mut u8,
    overflow: *mut i32,
) -> u32 {
    if *v >= b'1' as u8 && *v <= b'9' as u8 {
        hts_str2uint(v.cast(), rv.cast(), 16, overflow) as u32
    } else if *v == b'0' as u8 {
        if *v.add(1) == b'\t' as u8 {
            *rv = v.add(1);
            0
        } else {
            let val = libc::strtoul(v.cast(), rv.cast(), 0);
            if val > 65535 {
                *overflow = 1;
                65535
            } else {
                val as u32
            }
        }
    } else {
        *rv = v;
        0
    }
}

unsafe fn sam_c_2524_aux_parse(
    start: *mut u8,
    end: *mut u8,
    b: *mut bam1_t,
    lenient: i32,
    tag_whitelist: *mut (),
) -> i32 {
    let mut overflow = 0;
    let mut q = start;
    let p = end;

    'loop_: while q < p {
        let checkpoint = (*b).data.len() as i32;
        let parse_err = |cond: bool| cond;

        if p.offset_from(q) < 5 {
            if lenient != 0 {
                break;
            }
            return -2;
        }
        if parse_err(*q < b'!' as u8 || *q.add(1) < b'!' as u8) {
            if lenient != 0 {
                while q < p && isspace_c(*q as i8) == 0 {
                    q = q.add(1);
                }
                while q < p && isspace_c(*q as i8) != 0 {
                    q = q.add(1);
                }
                (*b).data.truncate(checkpoint as usize);
                continue 'loop_;
            }
            return -2;
        }

        if lenient != 0 && ((*q.add(2) as u8) | (*q.add(4) as u8)) != b':' {
            while q < p && isspace_c(*q as i8) == 0 {
                q = q.add(1);
            }
            while q < p && isspace_c(*q as i8) != 0 {
                q = q.add(1);
            }
            continue;
        }

        if !tag_whitelist.is_null() {
            let tt = (*q as i32) * 256 + *q.add(1) as i32;
            let tags = tag_whitelist.cast::<khash_tag_t>();
            let mut k = (*tags).n_buckets;
            if (*tags).n_buckets != 0 {
                let mask = (*tags).n_buckets - 1;
                let mut i = __ac_Wang_hash(tt as u32) & mask;
                let last = i;
                let mut step = 0;
                while !kh_isempty((*tags).flags, i)
                    && (kh_isdel((*tags).flags, i) || *(*tags).keys.add(i as usize) != tt)
                {
                    step += 1;
                    i = (i + step) & mask;
                    if i == last {
                        break;
                    }
                }
                if !kh_iseither((*tags).flags, i) {
                    k = i;
                }
            }
            if k == (*tags).n_buckets {
                while q < p && *q != b'\t' as u8 {
                    q = q.add(1);
                }
                continue;
            }
        }

        if possibly_expand_bam_data(b, 2) < 0 {
            return -2;
        }
        {
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 2) < 0 {
                return -2;
            }
            *(*b).data.as_mut_ptr().add(off) = *q as u8;
            *(*b).data.as_mut_ptr().add(off + 1) = *q.add(1) as u8;
        }

        q = q.add(3);
        let mut type_ = *q;
        q = q.add(2);
        if type_ != b'Z' as u8 && type_ != b'H' as u8 && *q <= b'\t' as u8 {
            if lenient != 0 {
                while q < p && isspace_c(*q as i8) == 0 {
                    q = q.add(1);
                }
                while q < p && isspace_c(*q as i8) != 0 {
                    q = q.add(1);
                }
                (*b).data.truncate(checkpoint as usize);
                continue 'loop_;
            }
            return -2;
        }

        if possibly_expand_bam_data(b, 16) < 0 {
            return -2;
        }

        if matches!(type_ as u8, b'A' | b'a' | b'c' | b'C') {
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 2) < 0 {
                return -2;
            }
            *(*b).data.as_mut_ptr().add(off) = b'A';
            *(*b).data.as_mut_ptr().add(off + 1) = *q as u8;
            q = q.add(1);
        } else if matches!(type_ as u8, b'i' | b'I') {
            if *q == b'-' as u8 {
                let x = hts_str2int(q.cast(), (&mut q as *mut *mut u8).cast(), 32, &mut overflow);
                if x >= i8::MIN as i64 {
                    let off = (*b).data.len();
                    if realloc_bam_data(b, off + 2) < 0 {
                        return -2;
                    }
                    *(*b).data.as_mut_ptr().add(off) = b'c';
                    *(*b).data.as_mut_ptr().add(off + 1) = x as u8;
                } else if x >= i16::MIN as i64 {
                    let off = (*b).data.len();
                    if realloc_bam_data(b, off + 3) < 0 {
                        return -2;
                    }
                    *(*b).data.as_mut_ptr().add(off) = b's';
                    i16_to_le(x as i16, (*b).data.as_mut_ptr().add(off + 1));
                } else {
                    let off = (*b).data.len();
                    if realloc_bam_data(b, off + 5) < 0 {
                        return -2;
                    }
                    *(*b).data.as_mut_ptr().add(off) = b'i';
                    i32_to_le(x as i32, (*b).data.as_mut_ptr().add(off + 1));
                }
            } else {
                let x = hts_str2uint(q.cast(), (&mut q as *mut *mut u8).cast(), 32, &mut overflow);
                if x <= u8::MAX as u64 {
                    let off = (*b).data.len();
                    if realloc_bam_data(b, off + 2) < 0 {
                        return -2;
                    }
                    *(*b).data.as_mut_ptr().add(off) = b'C';
                    *(*b).data.as_mut_ptr().add(off + 1) = x as u8;
                } else if x <= u16::MAX as u64 {
                    let off = (*b).data.len();
                    if realloc_bam_data(b, off + 3) < 0 {
                        return -2;
                    }
                    *(*b).data.as_mut_ptr().add(off) = b'S';
                    u16_to_le(x as u16, (*b).data.as_mut_ptr().add(off + 1));
                } else {
                    let off = (*b).data.len();
                    if realloc_bam_data(b, off + 5) < 0 {
                        return -2;
                    }
                    *(*b).data.as_mut_ptr().add(off) = b'I';
                    u32_to_le(x as u32, (*b).data.as_mut_ptr().add(off + 1));
                }
            }
        } else if type_ == b'f' as u8 {
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 1 + std::mem::size_of::<f32>()) < 0 {
                return -2;
            }
            *(*b).data.as_mut_ptr().add(off) = b'f';
            let value = libc::strtod(q.cast(), (&mut q as *mut *mut u8).cast());
            float_to_le(value as f32, (*b).data.as_mut_ptr().add(off + 1));
        } else if type_ == b'd' as u8 {
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 1 + std::mem::size_of::<f64>()) < 0 {
                return -2;
            }
            *(*b).data.as_mut_ptr().add(off) = b'd';
            let value = libc::strtod(q.cast(), (&mut q as *mut *mut u8).cast());
            double_to_le(value, (*b).data.as_mut_ptr().add(off + 1));
        } else if matches!(type_ as u8, b'Z' | b'H') {
            let mut zend = q;
            while zend < p && *zend != b'\t' as u8 && *zend != 0 {
                zend = zend.add(1);
            }
            if type_ == b'H' as u8 && (zend.offset_from(q) & 1) != 0 {
                if lenient != 0 {
                    while q < p && isspace_c(*q as i8) == 0 {
                        q = q.add(1);
                    }
                    while q < p && isspace_c(*q as i8) != 0 {
                        q = q.add(1);
                    }
                    (*b).data.truncate(checkpoint as usize);
                    continue 'loop_;
                }
                return -2;
            }
            let zlen = zend.offset_from(q) as usize;
            let off = (*b).data.len();
            if realloc_bam_data(b, off + 1 + zlen + 1) < 0 {
                return -2;
            }
            *(*b).data.as_mut_ptr().add(off) = type_ as u8;
            std::ptr::copy_nonoverlapping(q.cast::<u8>(), (*b).data.as_mut_ptr().add(off + 1).cast::<u8>(), (zlen) as usize);
            *(*b).data.as_mut_ptr().add(off + 1 + zlen) = 0;
            q = zend;
        } else if type_ == b'B' as u8 {
            type_ = *q;
            q = q.add(1);
            if *q != 0 && *q != b',' as u8 && *q != b'\t' as u8 {
                if lenient != 0 {
                    while q < p && isspace_c(*q as i8) == 0 {
                        q = q.add(1);
                    }
                    while q < p && isspace_c(*q as i8) != 0 {
                        q = q.add(1);
                    }
                    (*b).data.truncate(checkpoint as usize);
                    continue 'loop_;
                }
                return -2;
            }
            if sam_c_2490_sam_parse_B_vals(type_, q, &mut q, b) < 0 {
                return -2;
            }
        } else if lenient != 0 {
            while q < p && isspace_c(*q as i8) == 0 {
                q = q.add(1);
            }
            while q < p && isspace_c(*q as i8) != 0 {
                q = q.add(1);
            }
            (*b).data.truncate(checkpoint as usize);
            continue;
        } else {
            return -2;
        }

        while q < p && *q > b'\t' as u8 {
            q = q.add(1);
        }
        if q < p {
            q = q.add(1);
        }
    }

    if lenient == 0 && overflow != 0 {
        return -2;
    }
    0
}

pub unsafe fn sam_c_2662_sam_parse1(s: *mut kstring_t, h: *mut sam_hdr_t, b: *mut bam1_t) -> i32 {
    // The owned kstring is parsed in place: tokens are NUL-delimited by writing
    // into this Vec's backing buffer, so take a raw mutable pointer to it.
    //
    // C contract: a kstring_t passed here is always NUL-terminated (`s[l] == 0`,
    // with `l` excluding the NUL). The integer/float aux scanners (hts_str2int /
    // hts_str2uint / strtod) are *unbounded* — they stop at the first non-digit,
    // relying on that terminating NUL. An owned Vec built from exactly the line
    // bytes (e.g. `line.to_vec()` or a worker copy of `line_len` bytes) does NOT
    // carry the NUL, so a trailing integer aug field (…NM:i:0) would read past the
    // buffer into adjacent heap bytes (UB; wrong value). Restore the invariant:
    // ensure a NUL exists at index `s_len` without counting it in the parse length.
    let s_len = (*s).data.len();
    if (*s).data.last() != Some(&0) {
        (*s).data.push(0);
    }
    let s_start = (*s).data.as_mut_ptr().cast::<u8>();
    let mut p = s_start;
    let mut q: *mut u8;
    let mut overflow = 0;
    let c = &mut (*b).core;

    macro_rules! read_token {
        ($p:ident) => {{
            let token = $p;
            let tab = libc::strchr($p.cast(), b'\t' as i32);
            if tab.is_null() {
                return -2;
            }
            *tab = 0;
            $p = tab.add(1).cast();
            token
        }};
    }

    // SEAM: `b` may be a calloc'd bam1_t slot from the parse-worker pool, whose
    // `data` Vec has a NULL pointer. Vec::clear() aborts on a NULL pointer even at
    // len 0; truncate(0) empties it via the precondition-free raw-slice path and is
    // equivalent (no free; later realloc_bam_data/reserve allocates as needed).
    (*b).data.truncate(0);
    std::ptr::write_bytes((c as *mut bam1_core_t).cast::<u8>(), 0, 32);

    q = read_token!(p);
    if p.offset_from(q) > 255 {
        return -2;
    }
    if possibly_expand_bam_data(b, p.offset_from(q) as usize + 4) < 0 {
        return -2;
    }
    {
        let qlen = p.offset_from(q) as usize;
        let off = (*b).data.len();
        if realloc_bam_data(b, off + qlen) < 0 {
            return -2;
        }
        std::ptr::copy_nonoverlapping(q.cast::<u8>(), (*b).data.as_mut_ptr().add(off).cast::<u8>(), (qlen) as usize);
    }
    c.l_extranul = ((4 - ((*b).data.len() as i32 & 3)) & 3) as u8;
    {
        let off = (*b).data.len();
        if realloc_bam_data(b, off + c.l_extranul as usize) < 0 {
            return -2;
        }
        std::ptr::write_bytes((*b).data.as_mut_ptr().add(off), 0, c.l_extranul as usize);
    }
    c.l_qname = (p.offset_from(q) as i32 + c.l_extranul as i32) as u16;

    c.flag = sam_c_2498_parse_sam_flag(p, &mut p, &mut overflow) as u16;
    if *p != b'\t' as u8 {
        return -2;
    }
    p = p.add(1);

    q = read_token!(p);
    if libc::strcmp(q.cast(), c"*".as_ptr()) != 0 {
        if (*h).n_targets == 0 {
            return -2;
        }
        c.tid = bam_name2id(h, q);
        if c.tid < -1 {
            return -2;
        }
    } else {
        c.tid = -1;
    }

    c.pos = hts_str2uint(p.cast(), (&mut p as *mut *mut u8).cast(), 62, &mut overflow) as hts_pos_t - 1;
    if *p != b'\t' as u8 {
        return -2;
    }
    p = p.add(1);
    if c.pos < 0 && c.tid >= 0 {
        c.tid = -1;
    }
    if c.tid < 0 {
        c.flag |= BAM_FUNMAP as u16;
    }

    c.qual = hts_str2uint(p.cast(), (&mut p as *mut *mut u8).cast(), 8, &mut overflow) as u8;
    if *p != b'\t' as u8 {
        return -2;
    }
    p = p.add(1);

    let cigreflen;
    if *p != b'*' as u8 {
        let old_l_data = (*b).data.len() as i32;
        let n_cigar = bam_parse_cigar(p, &mut p, b);
        if n_cigar < 1 || *p != b'\t' as u8 {
            return -2;
        }
        p = p.add(1);
        let cigar = (*b).data.as_ptr().add(old_l_data as usize).cast::<u32>();
        cigreflen = if (c.flag as i32 & BAM_FUNMAP) == 0 {
            bam_cigar2rlen(c.n_cigar as i32, cigar)
        } else {
            1
        };
        let cigreflen = if cigreflen == 0 { 1 } else { cigreflen };
        if HTS_POS_MAX - cigreflen <= c.pos {
            return -2;
        }
        c.bin = hts_reg2bin(c.pos, c.pos + cigreflen, 14, 5) as u16;
    } else {
        c.flag |= BAM_FUNMAP as u16;
        q = read_token!(p);
        let _ = q;
        cigreflen = 1;
        if HTS_POS_MAX - cigreflen <= c.pos {
            return -2;
        }
        c.bin = hts_reg2bin(c.pos, c.pos + cigreflen, 14, 5) as u16;
    }

    q = read_token!(p);
    if libc::strcmp(q.cast(), c"=".as_ptr()) == 0 {
        c.mtid = c.tid;
    } else if libc::strcmp(q.cast(), c"*".as_ptr()) == 0 {
        c.mtid = -1;
    } else {
        c.mtid = bam_name2id(h, q);
        if c.mtid < -1 {
            return -2;
        }
    }

    c.mpos = hts_str2uint(p.cast(), (&mut p as *mut *mut u8).cast(), 62, &mut overflow) as hts_pos_t - 1;
    if *p != b'\t' as u8 {
        return -2;
    }
    p = p.add(1);
    if c.mpos < 0 && c.mtid >= 0 {
        c.mtid = -1;
    }

    c.isize = hts_str2int(p.cast(), (&mut p as *mut *mut u8).cast(), 63, &mut overflow);
    if *p != b'\t' as u8 {
        return -2;
    }
    p = p.add(1);
    if overflow != 0 {
        return -2;
    }

    q = read_token!(p);
    if libc::strcmp(q.cast(), c"*".as_ptr()) != 0 {
        let seq_len = p.offset_from(q) - 1;
        if seq_len > i32::MAX as isize {
            return -2;
        }
        c.l_qseq = seq_len as i32;
        let ql = bam_cigar2qlen(c.n_cigar as i32, (*b).data.as_ptr().add(c.l_qname as usize).cast());
        if c.n_cigar != 0 && ql != c.l_qseq as hts_pos_t {
            return -2;
        }
        let seq_bytes = ((c.l_qseq + 1) >> 1) as usize;
        if possibly_expand_bam_data(b, seq_bytes) < 0 {
            return -2;
        }
        let seq_off = (*b).data.len();
        if realloc_bam_data(b, seq_off + seq_bytes) < 0 {
            return -2;
        }
        let t = (*b).data.as_mut_ptr().add(seq_off);
        let lqs2 = (c.l_qseq & !1) as usize;
        let mut i = 0usize;
        while i < lqs2 {
            *t.add(i >> 1) = (SEQ_NT16_TABLE[*q.add(i) as u8 as usize] << 4)
                | SEQ_NT16_TABLE[*q.add(i + 1) as u8 as usize];
            i += 2;
        }
        while i < c.l_qseq as usize {
            *t.add(i >> 1) = SEQ_NT16_TABLE[*q.add(i) as u8 as usize] << ((i & 1 ^ 1) << 2);
            i += 1;
        }
    } else {
        c.l_qseq = 0;
    }

    if possibly_expand_bam_data(b, c.l_qseq as usize) < 0 {
        return -2;
    }
    let qual_off = (*b).data.len();
    if realloc_bam_data(b, qual_off + c.l_qseq as usize) < 0 {
        return -2;
    }
    let t = (*b).data.as_mut_ptr().add(qual_off);
    if *p == b'*' as u8 && (*p.add(1) == b'\t' as u8 || *p.add(1) == 0) {
        std::ptr::write_bytes(t, 0xff, c.l_qseq as usize);
        p = p.add(2);
    } else {
        if s_len < p.offset_from(s_start) as usize + c.l_qseq as usize
            || (*p.add(c.l_qseq as usize) != b'\t' as u8 && *p.add(c.l_qseq as usize) != 0)
        {
            return -2;
        }
        let mut failed = 0u8;
        for i in 0..c.l_qseq as usize {
            *t.add(i) = (*p.add(i) as u8).wrapping_sub(33);
            failed |= *t.add(i);
        }
        if (failed & 0x80) != 0 {
            return -2;
        }
        p = p.add(c.l_qseq as usize + 1);
    }

    if sam_c_2524_aux_parse(p, s_start.add(s_len), b, 0, std::ptr::null_mut()) < 0 {
        return -2;
    }

    if bam_tag2cigar(b, 1, 1) < 0 {
        return -2;
    }
    0
}

unsafe fn sam_c_3048_sam_state_create(fp: *mut htsFile) -> *mut SAM_state {
    if (*fp).format.format != HTS_FORMAT_SAM && (*fp).format.format != HTS_FORMAT_TEXT_FORMAT {
        return std::ptr::null_mut();
    }
    let fd = Box::into_raw(Box::new(std::mem::zeroed::<SAM_state>())).cast::<()>()
        .cast::<SAM_state>();
    if fd.is_null() {
        return std::ptr::null_mut();
    }
    (*fp).state = fd.cast();
    (*fd).fp = fp;
    fd
}

unsafe fn sam_c_3069_sam_state_err(fd: *mut SAM_state, errcode: i32) {
    if !fd.is_null() && (*fd).errcode == 0 {
        (*fd).errcode = errcode;
    }
}

unsafe fn sam_c_3076_sam_free_sp_bams(b: *mut sp_bams) {
    if b.is_null() {
        return;
    }
    if !(*b).bams.is_null() {
        // `bams` is a `Vec<bam1_t>` of length == capacity == `abams`, every slot a
        // valid `bam1_t::default()`. Reconstituting and dropping that Vec drops
        // each record — and each record's owned `data: Vec<u8>` — exactly once,
        // then frees the array. Dropping `b.data` separately would double-free.
        let cap = (*b).abams as usize;
        drop(Vec::from_raw_parts((*b).bams, cap, cap));
    }
    drop(Box::from_raw(b));
}

unsafe extern "C" fn sam_c_3200_cleanup_sp_lines(arg: *mut ()) {
    let gl = arg.cast::<sp_lines>();
    if gl.is_null() {
        return;
    }

    assert!((*gl).next.is_none());

    if !(*gl).data.is_null() {
        let cap = (*gl).alloc as usize;
        drop(Vec::from_raw_parts((*gl).data, cap, cap));
    }
    if let Some(bams) = (*gl).bams {
        sam_c_3076_sam_free_sp_bams(bams.as_ptr());
    }
    drop(Box::from_raw(gl));
}

unsafe extern "C" fn sam_c_3313_sam_parse_eof(_arg: *mut ()) -> *mut () {
    std::ptr::null_mut()
}

unsafe extern "C" fn sam_c_3318_cleanup_sp_bams(arg: *mut ()) {
    sam_c_3076_sam_free_sp_bams(arg.cast::<sp_bams>());
}

unsafe extern "C" fn sam_c_3215_sam_parse_worker(arg: *mut ()) -> *mut () {
    let gl = arg.cast::<sp_lines>();
    let mut gb = std::ptr::null_mut::<sp_bams>();
    let lines = (*gl).data;
    let fd = (*gl)
        .fd
        .map(|fd| fd.as_ptr())
        .unwrap_or(std::ptr::null_mut());

    if !fd.is_null() {
        if let Some(cached) = (*fd).bams {
            gb = cached.as_ptr();
            (*fd).bams = (*gb).next;
        }
    }

    if gb.is_null() {
        gb = Box::into_raw(Box::new(std::mem::zeroed::<sp_bams>())).cast::<()>()
            .cast::<sp_bams>();
        if gb.is_null() {
            return std::ptr::null_mut();
        }
        (*gb).abams = 100;
        (*gb).bams = {
            let mut v: Vec<bam1_t> = (0..(*gb).abams as usize)
                .map(|_| bam1_t::default())
                .collect();
            let p = v.as_mut_ptr();
            std::mem::forget(v);
            p
        };
        (*gb).nbams = 0;
        (*gb).bam_mem = 0;
    }
    (*gb).serial = (*gl).serial;
    (*gb).next = None;

    let mut b = (*gb).bams;
    if b.is_null() {
        sam_c_3069_sam_state_err(fd, libc::ENOMEM as i32);
        sam_c_3076_sam_free_sp_bams(gb);
        return std::ptr::null_mut();
    }

    let mut i = 0;
    let mut cp = lines;
    let cp_end = lines.add((*gl).data_size as usize);
    while cp < cp_end {
        if i >= (*gb).abams {
            let old_abams = (*gb).abams;
            (*gb).abams *= 2;
            let mut bams_vec = Vec::from_raw_parts(
                (*gb).bams,
                old_abams as usize,
                old_abams as usize,
            );
            bams_vec.reserve((*gb).abams as usize - old_abams as usize);
            while bams_vec.len() < (*gb).abams as usize {
                bams_vec.push(bam1_t::default());
            }
            b = bams_vec.as_mut_ptr();
            std::mem::forget(bams_vec);
            (*gb).bams = b;
        }

        let mut nl = cp;
        while nl < cp_end && *nl != b'\n' as u8 {
            nl = nl.add(1);
        }
        let mut line_end = nl;
        let next = if nl < cp_end { nl.add(1) } else { cp_end };
        if line_end > cp && *line_end.sub(1) == b'\r' as u8 {
            line_end = line_end.sub(1);
        }
        *line_end = 0;
        let line_len = line_end.offset_from(cp) as usize;
        // sam_parse1 owns/mutates its kstring in place; copy this line's bytes
        // (which live in the shared `gl` buffer) into an owned kstring.
        let mut ks = kstring_t {
            data: std::slice::from_raw_parts(cp.cast::<u8>(), line_len).to_vec(),
        };
        if sam_c_2662_sam_parse1(&mut ks, (*fd).h, b.add(i as usize)) < 0 {
            let errno = *libc::__errno_location();
            sam_c_3069_sam_state_err(
                fd,
                if errno != 0 {
                    errno
                } else {
                    libc::EIO as i32
                },
            );
            sam_c_3200_cleanup_sp_lines(gl.cast());
            sam_c_3076_sam_free_sp_bams(gb);
            return std::ptr::null_mut();
        }

        cp = next;
        i += 1;
    }
    (*gb).nbams = i;

    if !fd.is_null() {
        (*gl).next = (*fd).lines;
        (*fd).lines = NonNull::new(gl);
    }
    gb.cast()
}

unsafe extern "C" fn sam_c_3652_sam_format_worker(arg: *mut ()) -> *mut () {
    let gb = arg.cast::<sp_bams>();
    let fd = (*gb)
        .fd
        .expect("sp_bams formatting worker requires SAM_state")
        .as_ptr();
    let fp = (*fd).fp;
    let mut gl = std::ptr::null_mut::<sp_lines>();

    if let Some(cached) = (*fd).lines {
        gl = cached.as_ptr();
        (*fd).lines = (*gl).next;
    }

    if gl.is_null() {
        gl = Box::into_raw(Box::new(std::mem::zeroed::<sp_lines>())).cast::<()>()
            .cast::<sp_lines>();
        if gl.is_null() {
            sam_c_3069_sam_state_err(fd, libc::ENOMEM as i32);
            return std::ptr::null_mut();
        }
        (*gl).alloc = 0;
        (*gl).data_size = 0;
        (*gl).data = std::ptr::null_mut();
    }
    (*gl).serial = (*gb).serial;
    (*gl).next = None;

    let mut ks = kstring_t::default();
    for i in 0..(*gb).nbams {
        if sam_c_4324_sam_format1_append((*fd).h, (*gb).bams.add(i as usize), &mut ks) < 0 {
            let errno = *libc::__errno_location();
            sam_c_3069_sam_state_err(
                fd,
                if errno != 0 {
                    errno
                } else {
                    libc::EIO as i32
                },
            );
            drop(Box::from_raw(gl));
            return std::ptr::null_mut();
        }
        kputc(b'\n' as i32, &mut ks);
    }

    // (*gl).data is a Vec-owned buffer reused across worker calls (reconstructed
    // and dropped in sam_state_destroy); leak the formatted owned bytes here.
    let formatted = ks_release(&mut ks);
    if !(*gl).data.is_null() {
        let cap = (*gl).alloc as usize;
        drop(Vec::from_raw_parts((*gl).data, cap, cap));
    }
    let formatted_len = formatted.len();
    let mut buf_vec = formatted;
    (*gl).data_size = formatted_len as i32;
    (*gl).alloc = buf_vec.capacity() as i32;
    (*gl).data = buf_vec.as_mut_ptr();
    std::mem::forget(buf_vec);

    if !fp.is_null() && !(*fp).idx.is_null() {
        (*gl).bams = NonNull::new(gb);
    } else {
        (*gb).next = (*fd).bams;
        (*fd).bams = NonNull::new(gb);
        (*gl).bams = None;
    }

    gl.cast()
}

pub unsafe fn sam_state_destroy(fp: *mut htsFile) -> i32 {
    if fp.is_null() || (*fp).state.is_null() {
        return 0;
    }
    let fd = (*fp).state.cast::<SAM_state>();
    let ret = -(*fd).errcode;

    let mut l = (*fd).lines;
    while let Some(line) = l {
        let line = line.as_ptr();
        let n = (*line).next;
        if !(*line).data.is_null() {
            let cap = (*line).alloc as usize;
            drop(Vec::from_raw_parts((*line).data, cap, cap));
        }
        drop(Box::from_raw(line));
        l = n;
    }

    let mut b = (*fd).bams;
    while let Some(bams) = b {
        let bams = bams.as_ptr();
        if (*fd).curr_bam == NonNull::new(bams) {
            (*fd).curr_bam = None;
        }
        let n = (*bams).next;
        sam_c_3076_sam_free_sp_bams(bams);
        b = n;
    }
    if let Some(curr_bam) = (*fd).curr_bam {
        sam_c_3076_sam_free_sp_bams(curr_bam.as_ptr());
    }
    sam_hdr_destroy((*fd).h);
    if !(*fp).state.is_null() {
        drop(Box::from_raw((*fp).state.cast::<SAM_state>()));
    }
    (*fp).state = std::ptr::null_mut();
    ret
}

pub unsafe fn bam_name2id(_h: *mut sam_hdr_t, _ref_: *const u8) -> i32 {
    if _h.is_null() || _ref_.is_null() {
        return -1;
    }
    sam_hdr_name2tid(&mut *_h, CStr::from_ptr(_ref_.cast()).to_bytes())
}

unsafe extern "C" fn sam_c_418_bam_name2id_wrapper(
    vhdr: *mut std::ffi::c_void,
    ref_: *const std::ffi::c_char,
) -> i32 {
    bam_name2id(vhdr.cast(), ref_.cast())
}

pub unsafe fn sam_parse_region(
    h: *mut sam_hdr_t,
    s: *const u8,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    flags: i32,
) -> *const u8 {
    hts_parse_region(
        CStr::from_ptr(s.cast()),
        &mut *tid,
        &mut *beg,
        &mut *end,
        Some(sam_c_418_bam_name2id_wrapper),
        h.cast(),
        flags,
    )
    .cast()
}

unsafe extern "C" fn sam_c_1210_bam_sym_lookup(
    data: *mut std::ffi::c_void,
    str_: *mut std::ffi::c_char,
    end: *mut *mut std::ffi::c_char,
    res: *mut hts_expr_val_t,
) -> i32 {
    let hb = data.cast::<hb_pair>();
    let b = (*hb).b;
    (*res).is_str = 0;

    match *str_ as u8 {
        b'c' if libc::memcmp(str_.cast(), c"cigar".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            ks_clear(&mut (*res).s);
            let s = &mut (*res).s;
            let cigar = bam_get_cigar(b);
            let n = (*b).core.n_cigar as i32;
            let mut r = 0;
            if n != 0 {
                for i in 0..n {
                    let c = *cigar.add(i as usize);
                    r |= (kputw(bam_cigar_oplen(c) as i32, s) < 0) as i32;
                    r |= (kputc_(b"MIDNSHP=XB??????"[bam_cigar_op(c) as usize] as i32, s) < 0)
                        as i32;
                }
                r |= (kputs(b"", s) < 0) as i32;
            } else {
                r |= (kputs(b"*", s) < 0) as i32;
            }
            if r != 0 {
                -1
            } else {
                0
            }
        }
        b'e' if libc::memcmp(str_.cast(), c"endpos".as_ptr().cast(), 6) == 0 => {
            *end = str_.add(6);
            (*res).d = bam_endpos(b) as f64;
            0
        }
        b'f' if libc::memcmp(str_.cast(), c"flag".as_ptr().cast(), 4) == 0 => {
            let mut s = str_.add(4);
            *end = s;
            if *s != b'.' as i8 {
                (*res).d = (*b).core.flag as f64;
                return 0;
            }
            s = s.add(1);
            let flags: &[(&[u8], i32)] = &[
                (b"paired", BAM_FPAIRED),
                (b"proper_pair", BAM_FPROPER_PAIR),
                (b"unmap", BAM_FUNMAP),
                (b"munmap", BAM_FMUNMAP),
                (b"reverse", BAM_FREVERSE),
                (b"mreverse", BAM_FMREVERSE),
                (b"read1", BAM_FREAD1),
                (b"read2", BAM_FREAD2),
                (b"secondary", BAM_FSECONDARY),
                (b"qcfail", BAM_FQCFAIL),
                (b"dup", BAM_FDUP),
                (b"supplementary", BAM_FSUPPLEMENTARY),
            ];
            for (name, flag) in flags {
                if libc::memcmp(s.cast(), name.as_ptr().cast(), name.len()) == 0 {
                    *end = s.add(name.len());
                    (*res).d = ((*b).core.flag as i32 & *flag) as f64;
                    return 0;
                }
            }
            -1
        }
        b'h' if libc::memcmp(str_.cast(), c"hclen".as_ptr().cast(), 5) == 0 => {
            let mut hclen = 0;
            let cigar = bam_get_cigar(b);
            let ncigar = (*b).core.n_cigar;
            if ncigar > 0 && bam_cigar_op(*cigar) == BAM_CHARD_CLIP {
                hclen = bam_cigar_oplen(*cigar) as i32;
            }
            if ncigar > 1 && bam_cigar_op(*cigar.add(ncigar as usize - 1)) == BAM_CHARD_CLIP {
                hclen += bam_cigar_oplen(*cigar.add(ncigar as usize - 1)) as i32;
            }
            *end = str_.add(5);
            (*res).d = hclen as f64;
            0
        }
        b'l' if libc::memcmp(str_.cast(), c"library".as_ptr().cast(), 7) == 0 => {
            *end = str_.add(7);
            (*res).is_str = 1;
            let lib = sam_c_1173_bam_get_library((*hb).h, b);
            ks_clear(&mut (*res).s);
            let lib_bytes: &[u8] = if lib.is_null() {
                b""
            } else {
                CStr::from_ptr(lib.cast()).to_bytes()
            };
            kputs(lib_bytes, &mut (*res).s);
            0
        }
        b'm' if libc::memcmp(str_.cast(), c"mapq".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            (*res).d = (*b).core.qual as f64;
            0
        }
        b'm' if libc::memcmp(str_.cast(), c"mpos".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            (*res).d = ((*b).core.mpos + 1) as f64;
            0
        }
        b'm' if libc::memcmp(str_.cast(), c"mrname".as_ptr().cast(), 6) == 0 => {
            *end = str_.add(6);
            (*res).is_str = 1;
            let rn = if (*hb).h.is_null() {
                std::ptr::null()
            } else {
                sam_hdr_tid2name(&*(*hb).h, (*b).core.mtid)
            };
            ks_clear(&mut (*res).s);
            let rn_bytes: &[u8] = if rn.is_null() {
                b"*"
            } else {
                CStr::from_ptr(rn.cast()).to_bytes()
            };
            kputs(rn_bytes, &mut (*res).s);
            0
        }
        b'm' if libc::memcmp(str_.cast(), c"mrefid".as_ptr().cast(), 6) == 0 => {
            *end = str_.add(6);
            (*res).d = (*b).core.mtid as f64;
            0
        }
        b'n' if libc::memcmp(str_.cast(), c"ncigar".as_ptr().cast(), 6) == 0 => {
            *end = str_.add(6);
            (*res).d = (*b).core.n_cigar as f64;
            0
        }
        b'p' if libc::memcmp(str_.cast(), c"pos".as_ptr().cast(), 3) == 0 => {
            *end = str_.add(3);
            (*res).d = ((*b).core.pos + 1) as f64;
            0
        }
        b'p' if libc::memcmp(str_.cast(), c"pnext".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).d = ((*b).core.mpos + 1) as f64;
            0
        }
        b'q' if libc::memcmp(str_.cast(), c"qlen".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            (*res).d = bam_cigar2qlen((*b).core.n_cigar as i32, bam_get_cigar(b)) as f64;
            0
        }
        b'q' if libc::memcmp(str_.cast(), c"qname".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            ks_clear(&mut (*res).s);
            kputs(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), &mut (*res).s);
            0
        }
        b'q' if libc::memcmp(str_.cast(), c"qual".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            ks_clear(&mut (*res).s);
            let qlen = (*b).core.l_qseq as usize;
            let qual = std::slice::from_raw_parts(bam_get_qual(b), qlen);
            (*res).s.data.extend_from_slice(qual);
            (*res).is_str = 1;
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"rlen".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            (*res).d = bam_cigar2rlen((*b).core.n_cigar as i32, bam_get_cigar(b)) as f64;
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"rname".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            let rn = if (*hb).h.is_null() {
                std::ptr::null()
            } else {
                sam_hdr_tid2name(&*(*hb).h, (*b).core.tid)
            };
            ks_clear(&mut (*res).s);
            let rn_bytes: &[u8] = if rn.is_null() {
                b"*"
            } else {
                CStr::from_ptr(rn.cast()).to_bytes()
            };
            kputs(rn_bytes, &mut (*res).s);
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"rnext".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            let rn = if (*hb).h.is_null() {
                std::ptr::null()
            } else {
                sam_hdr_tid2name(&*(*hb).h, (*b).core.mtid)
            };
            ks_clear(&mut (*res).s);
            let rn_bytes: &[u8] = if rn.is_null() {
                b"*"
            } else {
                CStr::from_ptr(rn.cast()).to_bytes()
            };
            kputs(rn_bytes, &mut (*res).s);
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"refid".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).d = (*b).core.tid as f64;
            0
        }
        b's' if libc::memcmp(str_.cast(), c"seq".as_ptr().cast(), 3) == 0 => {
            *end = str_.add(3);
            ks_clear(&mut (*res).s);
            let seq_len = (*b).core.l_qseq as usize;
            let packed = std::slice::from_raw_parts(bam_get_seq(b), seq_len.div_ceil(2));
            (*res).s.data.resize(seq_len, 0);
            let seq = std::slice::from_raw_parts_mut((*res).s.data.as_mut_ptr().cast::<u8>(), seq_len);
            nibble2base(packed, seq);
            (*res).is_str = 1;
            0
        }
        b's' if libc::memcmp(str_.cast(), c"sclen".as_ptr().cast(), 5) == 0 => {
            let mut sclen = 0;
            let cigar = bam_get_cigar(b);
            let ncigar = (*b).core.n_cigar as i32;
            let mut left = 0;
            if ncigar > 0 && bam_cigar_op(*cigar) == BAM_CSOFT_CLIP {
                sclen += bam_cigar_oplen(*cigar) as i32;
            } else if ncigar > 1
                && bam_cigar_op(*cigar) == BAM_CHARD_CLIP
                && bam_cigar_op(*cigar.add(1)) == BAM_CSOFT_CLIP
            {
                left = 1;
                sclen += bam_cigar_oplen(*cigar.add(1)) as i32;
            }
            if ncigar - 1 > left && bam_cigar_op(*cigar.add(ncigar as usize - 1)) == BAM_CSOFT_CLIP
            {
                sclen += bam_cigar_oplen(*cigar.add(ncigar as usize - 1)) as i32;
            } else if ncigar - 2 > left
                && bam_cigar_op(*cigar.add(ncigar as usize - 1)) == BAM_CHARD_CLIP
                && bam_cigar_op(*cigar.add(ncigar as usize - 2)) == BAM_CSOFT_CLIP
            {
                sclen += bam_cigar_oplen(*cigar.add(ncigar as usize - 2)) as i32;
            }
            *end = str_.add(5);
            (*res).d = sclen as f64;
            0
        }
        b't' if libc::memcmp(str_.cast(), c"tlen".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            (*res).d = (*b).core.isize as f64;
            0
        }
        b'[' if *str_.add(1) != 0 && *str_.add(2) != 0 && *str_.add(3) == b']' as i8 => {
            *end = str_.add(4);
            let aux = bam_aux_get(b, str_.add(1).cast());
            if aux.is_null() {
                (*res).is_str = 1;
                (*res).s.data.clear();
                (*res).d = 0.0;
                (*res).is_true = 0;
                return 0;
            }
            (*res).is_true = 1;
            match *aux as u8 {
                b'Z' | b'H' => {
                    (*res).is_str = 1;
                    ks_clear(&mut (*res).s);
                    kputs(CStr::from_ptr(aux.add(1).cast()).to_bytes(), &mut (*res).s);
                }
                b'A' => {
                    (*res).is_str = 1;
                    ks_clear(&mut (*res).s);
                    let a = std::slice::from_raw_parts(aux.add(1).cast::<u8>(), 1);
                    kputsn(a, 1, &mut (*res).s);
                }
                b'i' | b'I' | b's' | b'S' | b'c' | b'C' => {
                    (*res).is_str = 0;
                    (*res).d = bam_aux2i(aux) as f64;
                }
                b'f' | b'd' => {
                    (*res).is_str = 0;
                    (*res).d = bam_aux2f(aux);
                }
                _ => return -1,
            }
            0
        }
        _ => -1,
    }
}

pub unsafe fn sam_c_1535_sam_passes_filter(
    h: *const sam_hdr_t,
    b: *const bam1_t,
    filt: *mut std::ffi::c_void,
) -> i32 {
    let mut hb = hb_pair { h, b };
    let mut res = hts_expr_val_t {
        is_str: 0,
        is_true: 0,
        s: kstring_t::default(),
        d: 0.0,
    };
    if hts_filter_eval2(
        &mut *filt.cast::<hts_filter_t>(),
        (&mut hb as *mut hb_pair).cast(),
        Some(sam_c_1210_bam_sym_lookup),
        &mut res,
    ) != 0
    {
        crate::htslib_rs::hts::hts_expr_val_free(&mut res);
        return -1;
    }

    let t = res.is_true as i32;
    crate::htslib_rs::hts::hts_expr_val_free(&mut res);
    t
}

unsafe fn sam_c_3786_fastq_state_init(name_char: i32) -> *mut fastq_state {
    // fastq_state embeds four owned kstring_t (each a Vec), so it cannot be
    // mem::zeroed wholesale (a zeroed Vec is invalid).  Allocate uninitialized,
    // zero the raw bytes (sets the POD/regex fields), then write valid (empty)
    // Vecs into the kstring fields before the value is ever used as a Box value.
    let mut boxed = Box::<fastq_state>::new_uninit();
    let x = boxed.as_mut_ptr();
    std::ptr::write_bytes(x.cast::<u8>(), 0, mem::size_of::<fastq_state>());
    std::ptr::write(&mut (*x).name, kstring_t::default());
    std::ptr::write(&mut (*x).comment, kstring_t::default());
    std::ptr::write(&mut (*x).seq, kstring_t::default());
    std::ptr::write(&mut (*x).qual, kstring_t::default());
    let x = Box::into_raw(boxed.assume_init());
    (*x).BC[0] = b'B' as u8;
    (*x).BC[1] = b'C' as u8;
    (*x).BC[2] = 0;
    (*x).nprefix = name_char as u8;

    if libc::regcomp(
        &mut (*x).regex,
        c"^[^:]+:[^:]+:[^:]+:[^:]+:[^:]+:[^:]+:[^:]+:([^:#/]+)".as_ptr(),
        libc::REG_EXTENDED,
    ) != 0
    {
        drop(Box::from_raw(x));
        return std::ptr::null_mut();
    }

    x
}

pub unsafe fn sam_c_3802_fastq_state_destroy(fp: *mut htsFile) {
    if !fp.is_null() && !(*fp).state.is_null() {
        let x = (*fp).state.cast::<fastq_state>();
        ks_free(&mut (*x).name);
        ks_free(&mut (*x).seq);
        ks_free(&mut (*x).qual);
        if !(*x).tags.is_null() {
            let tags = (*x).tags.cast::<khash_tag_t>();
            let nb = (*tags).n_buckets as usize;
            let nf = if (*tags).n_buckets < 16 { 1 } else { ((*tags).n_buckets >> 4) as usize };
            if !(*tags).flags.is_null() {
                drop(Vec::from_raw_parts((*tags).flags, nf, nf));
            }
            if !(*tags).keys.is_null() {
                drop(Vec::from_raw_parts((*tags).keys, nb, nb));
            }
            drop(Box::from_raw(tags));
        }
        libc::regfree(&mut (*x).regex);
        drop(Box::from_raw(x));
        (*fp).state = std::ptr::null_mut();
    }
}

pub unsafe fn sam_c_3815_fastq_state_set(
    fp: *mut htsFile,
    opt: i32,
    arg: *const u8,
) -> i32 {
    if fp.is_null() {
        return -1;
    }
    if (*fp).state.is_null() {
        (*fp).state =
            sam_c_3786_fastq_state_init(if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
                b'@'
            } else {
                b'>'
            } as i32)
            .cast();
        if (*fp).state.is_null() {
            return -1;
        }
    }

    let x = (*fp).state.cast::<fastq_state>();
    match opt {
        FASTQ_OPT_CASAVA => (*x).casava = 1,
        FASTQ_OPT_NAME2 => (*x).sra_names = 1,
        FASTQ_OPT_RNUM => (*x).rnum = 1,
        FASTQ_OPT_AUX => {
            (*x).aux = 1;
            if !arg.is_null() && libc::strcmp(arg.cast(), c"1".as_ptr()) != 0 {
                if (*x).tags.is_null() {
                    let tags = Box::into_raw(Box::new(std::mem::zeroed::<khash_tag_t>()));
                    let tlen = {
                        let mut n = 0usize;
                        while *arg.add(n) != 0 {
                            n += 1;
                        }
                        n
                    };
                    let mut n_buckets = 4u32;
                    while (n_buckets as usize) < ((tlen / 3) + 1) * 2 {
                        n_buckets <<= 1;
                    }
                    let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
                    (*tags).flags = {
                        let mut v: Vec<u32> = vec![0u32; n_flags as usize];
                        let p = v.as_mut_ptr();
                        std::mem::forget(v);
                        p
                    };
                    (*tags).keys = {
                        let mut v: Vec<i32> = vec![0i32; n_buckets as usize];
                        let p = v.as_mut_ptr();
                        std::mem::forget(v);
                        p
                    };
                    for i in 0..n_flags {
                        *(*tags).flags.add(i as usize) = 0xaaaa_aaaa;
                    }
                    (*tags).n_buckets = n_buckets;
                    (*tags).upper_bound = (n_buckets as f64 * 0.77) as u32;
                    (*x).tags = tags.cast();
                }

                let tags = (*x).tags.cast::<khash_tag_t>();
                let tag = CStr::from_ptr(arg.cast()).to_bytes();
                let tlen = tag.len();
                let mut i = 0usize;
                while i + 3 <= tlen + 1 {
                    let c0 = *arg.add(i);
                    let c1 = *arg.add(i + 1);
                    let c2 = if i + 2 < tlen { *arg.add(i + 2) } else { 0 };
                    if c0 == b',' as u8
                        || c1 == b',' as u8
                        || !(c2 == b',' as u8 || c2 == 0)
                    {
                        break;
                    }
                    let tcode = c0 as i32 * 256 + c1 as i32;
                    let mask = (*tags).n_buckets - 1;
                    let mut k = __ac_Wang_hash(tcode as u32) & mask;
                    let mut step = 0;
                    while !kh_isempty((*tags).flags, k) {
                        if !kh_isdel((*tags).flags, k) && *(*tags).keys.add(k as usize) == tcode {
                            break;
                        }
                        step += 1;
                        k = (k + step) & mask;
                    }
                    if kh_iseither((*tags).flags, k) {
                        if kh_isempty((*tags).flags, k) {
                            (*tags).n_occupied += 1;
                        }
                        kh_set_occupied((*tags).flags, k);
                        *(*tags).keys.add(k as usize) = tcode;
                        (*tags).size += 1;
                    }
                    i += 3;
                }
            }
        }
        FASTQ_OPT_BARCODE => {
            if !arg.is_null() {
                libc::strncpy((*x).BC.as_mut_ptr().cast(), arg.cast(), 2);
                (*x).BC[2] = 0;
            }
        }
        FASTQ_OPT_UMI => {
            let bc = if arg.is_null() || libc::strcmp(arg.cast(), c"1".as_ptr()) == 0 {
                c"RX".as_ptr().cast()
            } else {
                arg
            };
            let mut p = bc;
            let mut ntags = 0usize;
            let mut err = 0;
            while *p != 0 && ntags < UMI_TAGS {
                if isalpha_c(*p as i8) == 0 || isalnum_c(*p.add(1) as i8) == 0 {
                    err = 1;
                    break;
                }
                (*x).UMI[ntags][0] = *p;
                (*x).UMI[ntags][1] = *p.add(1);
                p = p.add(2);
                if *p != 0 && *p != b',' as u8 {
                    err = 1;
                    break;
                }
                if *p == b',' as u8 {
                    p = p.add(1);
                }
                (*x).UMI[ntags][2] = 0;
                ntags += 1;
            }
            while ntags < UMI_TAGS {
                (*x).UMI[ntags] = [0; 3];
                ntags += 1;
            }
            let _ = err;
        }
        FASTQ_OPT_UMI_REGEX => {
            if !arg.is_null() {
                libc::regfree(&mut (*x).regex);
                if libc::regcomp(
                    &mut (*x).regex,
                    arg.cast(),
                    libc::REG_EXTENDED,
                ) != 0
                {
                    return -1;
                }
            }
        }
        _ => {}
    }
    0
}

unsafe fn sam_c_3927_fastq_parse1(fp: *mut htsFile, b: *mut bam1_t) -> i32 {
    let x = (*fp).state.cast::<fastq_state>();
    let mut ret;

    if (*fp).format.format == HTS_FORMAT_FASTA_FORMAT && !(*fp).line.data.is_empty() {
        // Move the already-read line into name (Vec move; the old name Vec is
        // dropped by the assignment).
        (*x).name = std::mem::take(&mut (*fp).line);
    } else {
        ret = hts_getline(
            fp,
            2,
            &mut (*x).name as *mut crate::htslib_rs::hts::kstring_t,
        );
        if ret == -1 {
            return -1;
        }
        if ret < -1 {
            return ret;
        }
    }

    let name_data = &(*x).name.data;
    if name_data.is_empty() || name_data[0] != (*x).nprefix as u8 {
        return -2;
    }

    // The fastq parser tokenizes name/comment in place and calls C string
    // routines (strpbrk/strtol/regexec) on the buffer, so ensure a working NUL
    // terminator is present (one past the logical length) without counting it
    // in name.data.len().
    (*x).name.data.reserve(1);
    let s = (*x).name.data.as_mut_ptr().cast::<u8>();
    *s.add((*x).name.data.len()) = 0;

    let mut i = 0usize;
    let mut name = s.add(1);
    if (*x).sra_names != 0 {
        let cp0: *mut u8 = libc::strpbrk(s.cast(), c" \t".as_ptr()).cast();
        if !cp0.is_null() {
            let mut cp = cp0;
            while *cp == b' ' as u8 || *cp == b'\t' as u8 {
                cp = cp.add(1);
            }
            cp = cp.sub(1);
            *cp = b'@' as u8;
            i = cp.offset_from(s) as usize;
            name = cp.add(1);
        }
    }

    let l = (*x).name.data.len();
    while i < l && isspace_c(*s.add(i) as i8) == 0 {
        i += 1;
    }
    if i < l {
        // NUL already written into the buffer at index i above; just shorten
        // the logical length (capacity still covers the terminator).
        *s.add(i) = 0;
        (*x).name.data.truncate(i);
        i += 1;
    }
    while i < l && isspace_c(*s.add(i) as i8) != 0 {
        i += 1;
    }
    // `comment` aliases into name's buffer (a raw ptr/len pair into name.data),
    // matching the C parser; it is consumed before name is mutated further.
    let comment_ptr: *mut u8 = s.add(i);
    let comment_len: usize = l - i;

    (*x).seq.data.clear();
    loop {
        ret = hts_getline(
            fp,
            2,
            &mut (*fp).line as *mut crate::htslib_rs::hts::kstring_t,
        );
        if ret < 0 && ((*fp).format.format == HTS_FORMAT_FASTQ_FORMAT || ret < -1) {
            return -2;
        }
        let line_data = &(*fp).line.data;
        if ret == -1
            || line_data[0]
                == if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
                    b'+'
                } else {
                    b'>'
                }
        {
            break;
        }
        if (*x).seq.data.is_empty() && (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
            mem::swap(&mut (*x).seq, &mut (*fp).line);
        } else {
            let line_data = (*fp).line.data.clone();
            if kputsn(&line_data, line_data.len(), &mut (*x).seq) < 0 {
                return -2;
            }
        }
    }

    if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
        let mut remainder = (*x).seq.data.len();
        (*x).qual.data.clear();
        while remainder > 0 {
            if hts_getline(
                fp,
                2,
                &mut (*fp).line as *mut crate::htslib_rs::hts::kstring_t,
            ) < 0
            {
                return -2;
            }
            if (*fp).line.data.len() > remainder {
                return -2;
            }
            let line_len = (*fp).line.data.len();
            if (*x).qual.data.is_empty() && (*fp).line.data.len() == remainder {
                mem::swap(&mut (*x).qual, &mut (*fp).line);
            } else {
                let line_data = (*fp).line.data.clone();
                if kputsn(&line_data, line_data.len(), &mut (*x).qual) < 0 {
                    return -2;
                }
            }
            remainder -= line_len;
        }

        let qual_data = &mut (*x).qual.data;
        for j in 0..qual_data.len() {
            qual_data[j] = qual_data[j].wrapping_sub(b'!');
        }
    }

    let mut flag = BAM_FUNMAP;
    let pflag = BAM_FMUNMAP | BAM_FPAIRED;
    let name_data = &(*x).name.data;
    if name_data.len() > 2
        && name_data[name_data.len() - 2] == b'/'
        && isdigit_c(name_data[name_data.len() - 1] as i8) != 0
    {
        match name_data[name_data.len() - 1] {
            b'1' => flag |= BAM_FREAD1 | pflag,
            b'2' => flag |= BAM_FREAD2 | pflag,
            _ => flag |= BAM_FREAD1 | BAM_FREAD2 | pflag,
        }
        let new_len = name_data.len() - 2;
        let name_data = &mut (*x).name.data;
        name_data.truncate(new_len);
        *s.add(new_len) = 0;
    }

    let mut umi_seq = [0 as u8; 256];
    let mut umi_len = 0usize;
    if (*x).UMI[0][0] != 0 {
        let mut mat: [libc::regmatch_t; 3] = std::mem::zeroed();
        if libc::regexec(&(*x).regex, s.cast(), 2, mat.as_mut_ptr(), 0) == 0
            && mat[0].rm_so >= 0
            && mat[1].rm_so >= 0
        {
            umi_len = (mat[1].rm_eo - mat[1].rm_so) as usize;
            if umi_len > 255 {
                return -2;
            }
            for (j, dst) in umi_seq.iter_mut().enumerate().take(umi_len) {
                let c = *s.add(j + mat[1].rm_so as usize);
                *dst = if isalpha_c(c as i8) != 0 { c } else { b'-' as u8 };
            }
            if umi_len != 0 {
                umi_seq[umi_len] = 0;
                umi_len += 1;

                // Rebuild name in place: keep the prefix up to the UMI match,
                // then append the suffix after the match.  All writes stay
                // within the existing (never-reallocated) name buffer.
                let mut nlen = mat[1].rm_so as usize;
                if nlen > 0 && *s.add(nlen - 1) == b':' as u8 {
                    nlen -= 1;
                }
                let mut cp = s.add(mat[1].rm_eo as usize);
                while *cp != 0 {
                    *s.add(nlen) = *cp;
                    nlen += 1;
                    cp = cp.add(1);
                }
                *s.add(nlen) = 0;
                (*x).name.data.truncate(nlen);
            }
        }
    }

    let l_qname = s.add((*x).name.data.len()).offset_from(name) as usize;
    // C passes `x->qual.s`, which is NULL for FASTA (never written). An empty
    // Rust Vec yields a non-null dangling pointer, so bam_set1's `qual != NULL`
    // test would wrongly try to memcpy from it. Use capacity (our `s != NULL`
    // proxy) to pass a genuine null pointer when qual was never allocated.
    let qual_ptr = if (*x).qual.data.capacity() == 0 {
        std::ptr::null()
    } else {
        (*x).qual.data.as_ptr()
    };
    ret = bam_set1_fastq_unmapped(
        b,
        l_qname,
        name,
        flag as u16,
        (*x).seq.data.len(),
        (*x).seq.data.as_ptr().cast(),
        qual_ptr.cast(),
    );
    if ret < 0 {
        return -2;
    }

    if umi_len != 0
        && bam_aux_append(
            b,
            (*x).UMI[0].as_ptr(),
            b'Z' as u8,
            umi_len as i32,
            umi_seq.as_ptr().cast(),
        ) < 0
    {
        ret = -2;
    }

    let mut barcode = std::ptr::null_mut::<u8>();
    let mut barcode_len = 0i32;
    // `comment` aliases into name's (never-reallocated) buffer; operate via the
    // raw ptr/len captured before name was rebuilt.
    let kc_s: *mut u8 = comment_ptr;
    let kc_l: usize = comment_len;
    if (*x).casava != 0 && kc_l > 6 {
        let mut endptr: *mut u8 = std::ptr::null_mut();
        if (*kc_s.add(1) as u8 | *kc_s.add(3) as u8) == b':'
            && isdigit_c(*kc_s as i8) != 0
            && libc::strtol(kc_s.add(4).cast(), (&mut endptr as *mut *mut u8).cast(), 10) >= 0
            && endptr != kc_s.add(4)
            && *endptr == b':' as u8
        {
            match *kc_s as u8 {
                b'1' => (*b).core.flag |= (BAM_FREAD1 | pflag) as u16,
                b'2' => (*b).core.flag |= (BAM_FREAD2 | pflag) as u16,
                _ => (*b).core.flag |= (BAM_FREAD1 | BAM_FREAD2 | pflag) as u16,
            }
            if *kc_s.add(2) == b'Y' as u8 {
                (*b).core.flag |= BAM_FQCFAIL as u16;
            }
            if isdigit_c(*endptr.add(1) as i8) == 0 {
                barcode = endptr.add(1);
                let mut j = barcode.offset_from(kc_s) as usize;
                while j < kc_l {
                    if isspace_c(*kc_s.add(j) as i8) != 0 {
                        break;
                    }
                    j += 1;
                }
                *kc_s.add(j) = 0;
                barcode_len = (j + 1 - barcode.offset_from(kc_s) as usize) as i32;
            }
        }
    }

    if ret >= 0
        && barcode_len != 0
        && bam_aux_append(
            b,
            (*x).BC.as_ptr(),
            b'Z' as u8,
            barcode_len,
            barcode.cast(),
        ) < 0
    {
        ret = -2;
    }

    if (*x).aux == 0 {
        return ret;
    }

    if sam_c_2524_aux_parse(kc_s.add(barcode_len as usize), kc_s.add(kc_l), b, 1, (*x).tags) < 0 {
        ret = -2;
    }

    ret
}

unsafe fn sam_c_4413_fastq_format1(
    x: *mut fastq_state,
    b: *const bam1_t,
    str_: *mut kstring_t,
) -> i32 {
    let flag = (*b).core.flag as i32;
    let len = (*b).core.l_qseq as usize;
    let mut e = 0;
    (*str_).data.clear();

    if kputc((*x).nprefix as i32, &mut *str_) < 0
        || kputs(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), &mut *str_) < 0
    {
        return -1;
    }

    if (*x).UMI[0][0] != 0 {
        let mut plex = [0 as u8; 256];
        // Byte at the logical end (index == len) plays the role of the old NUL
        // terminator: there is no terminator now, so treat it as 0.
        let byte_at = |i: usize| -> u8 {
            let str_data = &(*str_).data;
            if i < str_data.len() {
                str_data[i]
            } else {
                0
            }
        };
        let mut name_len = (*str_).data.len();
        while name_len != 0 && byte_at(name_len) != b':' && byte_at(name_len) != b'#' {
            name_len -= 1;
        }

        if byte_at(name_len) == b'#' && (*str_).data.len() - name_len < 255 {
            let copy_len = (*str_).data.len() - name_len;
            std::ptr::copy_nonoverlapping((*str_).data.as_ptr().add(name_len).cast::<u8>(), plex.as_mut_ptr().cast::<u8>(), (copy_len) as usize);
            plex[copy_len] = 0;
            (*str_).data.truncate(name_len);
        }

        let mut bc = std::ptr::null_mut::<u8>();
        let mut n = 0usize;
        while bc.is_null() && n < UMI_TAGS {
            if (*x).UMI[n][0] != 0 {
                bc = bam_aux_get(b, (*x).UMI[n].as_ptr());
            }
            n += 1;
        }
        if !bc.is_null() && *bc == b'Z' {
            if kputc(b':' as i32, &mut *str_) < 0 {
                return -1;
            }
            bc = bc.add(1);
            while *bc != 0 {
                let c = *bc as u8;
                if kputc(
                    if isalpha_c(c as i8) != 0 {
                        toupper_c(c as i8) as i32
                    } else {
                        b'+' as i32
                    },
                    &mut *str_,
                ) < 0
                {
                    return -1;
                }
                bc = bc.add(1);
            }
        }

        if plex[0] != 0 && kputs(CStr::from_ptr(plex.as_ptr().cast()).to_bytes(), &mut *str_) < 0 {
            return -1;
        }
    }

    if (*x).rnum != 0 && (flag & BAM_FPAIRED) != 0 {
        let r12 = flag & (BAM_FREAD1 | BAM_FREAD2);
        if r12 == BAM_FREAD1 {
            if kputs(b"/1", &mut *str_) < 0 {
                return -1;
            }
        } else if r12 == BAM_FREAD2 && kputs(b"/2", &mut *str_) < 0 {
            return -1;
        }
    }

    if (*x).casava != 0 {
        let rnum = if (flag & BAM_FREAD1) != 0 {
            1
        } else if (flag & BAM_FREAD2) != 0 {
            2
        } else {
            0
        };
        let filtered = if (flag & BAM_FQCFAIL) != 0 {
            b'Y' as i32
        } else {
            b'N' as i32
        };
        let bc = bam_aux_get(b, (*x).BC.as_ptr());
        e |= (kputc(b' ' as i32, &mut *str_) < 0) as i32;
        e |= (kputw(rnum, &mut *str_) < 0) as i32;
        e |= (kputc(b':' as i32, &mut *str_) < 0) as i32;
        e |= (kputc(filtered, &mut *str_) < 0) as i32;
        e |= (kputsn_(b":0:", 3, &mut *str_) < 0) as i32;
        if bc.is_null() {
            e |= (kputc(b'0' as i32, &mut *str_) < 0) as i32;
        } else {
            e |= (kputs(CStr::from_ptr(bc.add(1).cast()).to_bytes(), &mut *str_) < 0) as i32;
        }
        if e != 0 {
            return -1;
        }

        if !bc.is_null()
            && (*bc != b'Z'
                || (isupper_c(*bc.add(1) as i8) == 0 && islower_c(*bc.add(1) as i8) == 0))
        {
            let bc_len = CStr::from_ptr(bc.cast()).to_bytes().len();
            let str_data = &mut (*str_).data;
            if bc_len >= 2 && str_data.len() >= bc_len - 2 {
                let new_len = str_data.len() - (bc_len - 2);
                str_data.truncate(new_len);
                str_data[new_len - 1] = b'0';
            }
        } else if !bc.is_null() {
            let bc_len = CStr::from_ptr(bc.add(1).cast()).to_bytes().len();
            let str_data = &mut (*str_).data;
            let start = str_data.len() - bc_len;
            for i in 0..bc_len {
                let ch = str_data[start + i] as u8;
                if isalpha_c(ch as i8) == 0 {
                    str_data[start + i] = b'+';
                } else if islower_c(ch as i8) != 0 {
                    str_data[start + i] = toupper_c(ch as i8) as u8;
                }
            }
        }
    }

    if (*x).aux != 0 {
        let mut s = bam_get_aux(b).cast_mut();
        let end = (*b).data.as_ptr().add((*b).data.len()).cast_mut();
        while !s.is_null() && end.offset_from(s) >= 4 {
            let tt = *s as i32 * 256 + *s.add(1) as i32;
            let mut keep = (*x).tags.is_null();
            if !keep {
                let tags = (*x).tags.cast::<khash_tag_t>();
                if (*tags).n_buckets != 0 {
                    let mask = (*tags).n_buckets - 1;
                    let mut k = __ac_Wang_hash(tt as u32) & mask;
                    let last = k;
                    let mut step = 0;
                    while !kh_isempty((*tags).flags, k)
                        && (kh_isdel((*tags).flags, k) || *(*tags).keys.add(k as usize) != tt)
                    {
                        step += 1;
                        k = (k + step) & mask;
                        if k == last {
                            break;
                        }
                    }
                    keep = !kh_iseither((*tags).flags, k);
                }
            }
            if keep {
                e |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
                s = sam_format_aux1(s, *s.add(2), s.add(3), end, str_).cast_mut();
                if s.is_null() {
                    return -1;
                }
            } else {
                s = skip_aux(s.add(2), end);
            }
        }
        e |= (kputsn(b"", 0, &mut *str_) < 0) as i32;
    }

    if ks_resize(
        &mut *str_,
        (*str_).data.len() + 1 + len + 1 + 2 + len + 1 + 1,
    ) < 0
    {
        return -1;
    }
    e |= (kputc_(b'\n' as i32, &mut *str_) < 0) as i32;

    let seq = bam_get_seq(b);
    if (flag & BAM_FREVERSE) != 0 {
        for i in (0..len).rev() {
            e |= (kputc_(
                b"!TGKCYSBAWRDMHVN"[bam_seqi(seq, i) as usize] as i32,
                &mut *str_,
            ) < 0) as i32;
        }
    } else {
        for i in 0..len {
            e |= (kputc_(SEQ_NT16_STR[bam_seqi(seq, i) as usize] as i32, &mut *str_) < 0) as i32;
        }
    }

    if (*x).nprefix == b'@' as u8 {
        kputsn(b"\n+\n", 3, &mut *str_);
        let qual = bam_get_qual(b);
        if *qual == 0xff {
            for _ in 0..len {
                e |= (kputc_(b'B' as i32, &mut *str_) < 0) as i32;
            }
        } else if (flag & BAM_FREVERSE) != 0 {
            for i in (0..len).rev() {
                e |= (kputc_(33 + *qual.add(i) as i32, &mut *str_) < 0) as i32;
            }
        } else {
            for i in 0..len {
                e |= (kputc_(33 + *qual.add(i) as i32, &mut *str_) < 0) as i32;
            }
        }
    }
    e |= (kputc(b'\n' as i32, &mut *str_) < 0) as i32;

    if e != 0 {
        -1
    } else {
        (*str_).data.len() as i32
    }
}

pub unsafe fn bam_set_mempolicy(b: *mut bam1_t, policy: u32) {
    (*b).mempolicy_and_reserved = policy;
}

pub unsafe fn bam_get_mempolicy(b: *mut bam1_t) -> u32 {
    (*b).mempolicy_and_reserved
}

fn kroundup32(mut x: u32) -> u32 {
    if x == 0 {
        return 0;
    }
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x + 1
}

pub unsafe fn sam_realloc_bam_data(b: *mut bam1_t, desired: usize) -> i32 {
    if desired > (i32::MAX as f64 * 0.666) as usize {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }
    let mut new_m_data = kroundup32(desired as u32);
    new_m_data = new_m_data.wrapping_add(32);
    if (new_m_data as usize) < desired {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }

    // SEAM: owned Vec replaces malloc/realloc/free and the BAM_USER_OWNS_DATA
    // copy-on-grow dance (the Vec always owns its buffer; the user-owns-data
    // mempolicy bit is cleared on grow to preserve the original semantics).
    //
    // Reserve to the kroundup'd capacity (mirrors the old m_data), then enforce
    // the resize-to-len invariant: data.len() must cover `desired` so that the
    // accessor raw-pointer writes into [0..desired] are in-bounds. We grow len
    // to `desired` here; we never shrink it (callers may have set core fields
    // expecting the existing length to remain valid).
    let data = &mut (*b).data;
    let cur_cap = data.capacity();
    if (new_m_data as usize) > cur_cap {
        data.reserve_exact((new_m_data as usize) - data.len());
    }
    if desired > data.len() {
        data.resize(desired, 0);
    }
    if (bam_get_mempolicy(b) & BAM_USER_OWNS_DATA) != 0 {
        bam_set_mempolicy(b, bam_get_mempolicy(b) & !BAM_USER_OWNS_DATA);
    }
    0
}

pub unsafe fn realloc_bam_data(b: *mut bam1_t, desired: usize) -> i32 {
    // `m_data` == data.capacity(): only reallocate when the desired used length
    // exceeds the current capacity. When it fits, still enforce resize-to-len so
    // data.len() covers `desired` for the accessor writes.
    if desired <= (*b).data.capacity() {
        if desired > (*b).data.len() {
            (*b).data.resize(desired, 0);
        }
        return 0;
    }
    sam_realloc_bam_data(b, desired)
}

pub unsafe fn bam_init1() -> *mut bam1_t {
    // SEAM: owned construction via Default (data: Vec::new()); no mem::zeroed
    // (which would be UB now that bam1_t holds a Vec).
    Box::into_raw(Box::new(bam1_t::default()))
}

pub unsafe fn bam_destroy1(b: *mut bam1_t) {
    if b.is_null() {
        return;
    }
    // SEAM: the data Vec owns its buffer and is freed when the Box is dropped;
    // the explicit free()/null-out of data is gone. We still honour the
    // BAM_USER_OWNS_STRUCT bit: when the struct is user-owned we clear the data
    // (dropping its buffer) but must not drop the Box itself.
    if (bam_get_mempolicy(b) & BAM_USER_OWNS_STRUCT) != 0 {
        if (bam_get_mempolicy(b) & BAM_USER_OWNS_DATA) == 0 {
            (*b).data = Vec::new();
        }
        return;
    }
    drop(Box::from_raw(b));
}

pub unsafe fn bam_copy1(bdst: *mut bam1_t, bsrc: *const bam1_t) -> *mut bam1_t {
    // SEAM: clone the owned data Vec (cap to the src used length == src len) and
    // the core; no realloc/memcpy. Resize-to-len holds automatically since we
    // clone exactly the used bytes.
    //
    // `bdst` may be a calloc'd pileup mempool node whose `data` Vec has a NULL
    // pointer; Vec::clear() aborts on a NULL pointer even at len 0, so empty via the
    // precondition-free truncate(0) (equivalent: no free; extend_from_slice then
    // reserves as needed).
    (*bdst).data.truncate(0);
    (*bdst).data.extend_from_slice(&(*bsrc).data);
    (*bdst).core = (*bsrc).core;
    (*bdst).id = (*bsrc).id;
    bdst
}

pub unsafe fn bam_dup1(bsrc: *const bam1_t) -> *mut bam1_t {
    if bsrc.is_null() {
        return std::ptr::null_mut();
    }
    let bdst = bam_init1();
    if bdst.is_null() {
        return std::ptr::null_mut();
    }
    if bam_copy1(bdst, bsrc).is_null() {
        bam_destroy1(bdst);
        return std::ptr::null_mut();
    }
    bdst
}

pub fn mp_init() -> Box<mempool_t> {
    Box::new(mempool_t {
        cnt: 0,
        n: 0,
        max: 0,
        padding_0: 0,
        buf: Vec::new(),
    })
}

pub unsafe fn mp_destroy(mut mp: Box<mempool_t>) {
    for node in mp.buf.drain(..) {
        let node = node.as_ptr();
        // Reclaiming the node Box drops the whole `lbnode_t`, including its
        // embedded `bam1_t` and that record's owned `data: Vec<u8>`. The C
        // original free()'d `b.data` separately because it was a raw pointer;
        // doing that here would double-free the Vec buffer.
        drop(Box::from_raw(node));
    }
}

pub unsafe fn mp_alloc(mp: &mut mempool_t) -> Option<NonNull<lbnode_t>> {
    mp.cnt += 1;
    if mp.n == 0 {
        // The C original calloc's the node (all-zero bytes). That is invalid for
        // the embedded `bam1_t.data: Vec<u8>` (a zeroed Vec has a NULL data
        // pointer, which is UB to operate on). Build the node from owned defaults
        // instead: `bam1_t::default()` gives a sound empty record, and the
        // remaining POD/union fields keep their all-zero initial state.
        NonNull::new(Box::into_raw(Box::new(lbnode_t {
            b: bam1_t::default(),
            beg: 0,
            end: 0,
            s: std::mem::zeroed::<cstate_t>(),
            next: None,
            cd: std::mem::zeroed::<bam_pileup_cd>(),
        })))
    } else {
        let node = mp
            .buf
            .pop()
            .expect("mempool free-list length tracks buffered nodes");
        mp.n = mp.buf.len() as i32;
        Some(node)
    }
}

pub unsafe fn mp_free(mp: &mut mempool_t, mut p: NonNull<lbnode_t>) {
    mp.cnt -= 1;
    p.as_mut().next = None;
    if mp.n == mp.max {
        mp.max = if mp.max != 0 { mp.max << 1 } else { 256 };
        if mp.buf.capacity() < mp.max as usize {
            mp.buf.reserve(mp.max as usize - mp.buf.capacity());
        }
    }
    mp.buf.push(p);
    mp.n = mp.buf.len() as i32;
}

pub unsafe fn resolve_cigar2(p: &mut bam_pileup1_t, pos: hts_pos_t, s: &mut cstate_t) -> i32 {
    let b = p.b;
    let c = &(*b).core;
    let cigar = bam_get_cigar(b);

    if s.k == -1 {
        p.qpos = 0;
        if c.n_cigar == 1 {
            let op = bam_cigar_op(*cigar);
            if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                s.k = 0;
                s.x = c.pos;
                s.y = 0;
            }
        } else {
            s.x = c.pos;
            s.y = 0;
            let mut k = 0;
            while k < c.n_cigar {
                let cg = *cigar.add(k as usize);
                let op = bam_cigar_op(cg);
                let l = bam_cigar_oplen(cg) as i32;
                if op == BAM_CMATCH
                    || op == BAM_CDEL
                    || op == BAM_CREF_SKIP
                    || op == BAM_CEQUAL
                    || op == BAM_CDIFF
                {
                    break;
                } else if op == BAM_CINS || op == BAM_CSOFT_CLIP {
                    s.y += l;
                }
                k += 1;
            }
            assert!(k < c.n_cigar);
            s.k = k as i32;
        }
    } else {
        let mut l = bam_cigar_oplen(*cigar.add(s.k as usize)) as hts_pos_t;
        if pos - s.x >= l {
            assert!(s.k < c.n_cigar as i32);
            let op = bam_cigar_op(*cigar.add(s.k as usize + 1));
            if op == BAM_CMATCH
                || op == BAM_CDEL
                || op == BAM_CREF_SKIP
                || op == BAM_CEQUAL
                || op == BAM_CDIFF
            {
                let cur_op = bam_cigar_op(*cigar.add(s.k as usize));
                if cur_op == BAM_CMATCH || cur_op == BAM_CEQUAL || cur_op == BAM_CDIFF {
                    s.y += l as i32;
                }
                s.x += l;
                s.k += 1;
            } else {
                let cur_op = bam_cigar_op(*cigar.add(s.k as usize));
                if cur_op == BAM_CMATCH || cur_op == BAM_CEQUAL || cur_op == BAM_CDIFF {
                    s.y += l as i32;
                }
                s.x += l;
                let mut k = s.k + 1;
                while k < c.n_cigar as i32 {
                    let cg = *cigar.add(k as usize);
                    let op = bam_cigar_op(cg);
                    l = bam_cigar_oplen(cg) as hts_pos_t;
                    if op == BAM_CMATCH
                        || op == BAM_CDEL
                        || op == BAM_CREF_SKIP
                        || op == BAM_CEQUAL
                        || op == BAM_CDIFF
                    {
                        break;
                    } else if op == BAM_CINS || op == BAM_CSOFT_CLIP {
                        s.y += l as i32;
                    }
                    k += 1;
                }
                s.k = k;
            }
            assert!(s.k < c.n_cigar as i32);
        }
    }

    let op = bam_cigar_op(*cigar.add(s.k as usize));
    let l = bam_cigar_oplen(*cigar.add(s.k as usize)) as hts_pos_t;
    set_pileup_is_del(p, false);
    p.indel = 0;
    set_pileup_is_refskip(p, false);
    if s.x + l - 1 == pos && s.k + 1 < c.n_cigar as i32 {
        let mut op2 = bam_cigar_op(*cigar.add(s.k as usize + 1));
        let mut l2 = bam_cigar_oplen(*cigar.add(s.k as usize + 1)) as i32;
        if op2 == BAM_CDEL && op != BAM_CDEL {
            p.indel = -l2;
            let mut k = s.k + 2;
            while k < c.n_cigar as i32 {
                op2 = bam_cigar_op(*cigar.add(k as usize));
                l2 = bam_cigar_oplen(*cigar.add(k as usize)) as i32;
                if op2 == BAM_CDEL {
                    p.indel -= l2;
                } else {
                    break;
                }
                k += 1;
            }
        } else if op2 == BAM_CINS {
            p.indel = l2;
            let mut k = s.k + 2;
            while k < c.n_cigar as i32 {
                op2 = bam_cigar_op(*cigar.add(k as usize));
                l2 = bam_cigar_oplen(*cigar.add(k as usize)) as i32;
                if op2 == BAM_CINS {
                    p.indel += l2;
                } else if op2 != BAM_CPAD {
                    break;
                }
                k += 1;
            }
        } else if op2 == BAM_CPAD && s.k + 2 < c.n_cigar as i32 {
            let mut l3 = 0;
            let mut k = s.k + 2;
            while k < c.n_cigar as i32 {
                op2 = bam_cigar_op(*cigar.add(k as usize));
                l2 = bam_cigar_oplen(*cigar.add(k as usize)) as i32;
                if op2 == BAM_CINS {
                    l3 += l2;
                } else if op2 == BAM_CDEL
                    || op2 == BAM_CMATCH
                    || op2 == BAM_CREF_SKIP
                    || op2 == BAM_CEQUAL
                    || op2 == BAM_CDIFF
                {
                    break;
                }
                k += 1;
            }
            if l3 > 0 {
                p.indel = l3;
            }
        }
    }
    if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
        p.qpos = s.y + (pos - s.x) as i32;
    } else if op == BAM_CDEL || op == BAM_CREF_SKIP {
        set_pileup_is_del(p, true);
        p.qpos = s.y;
        set_pileup_is_refskip(p, op == BAM_CREF_SKIP);
    }
    set_pileup_is_head(p, pos == c.pos);
    set_pileup_is_tail(p, pos == s.end);
    p.cigar_ind = s.k;
    1
}

unsafe fn cigar_iref2iseq_set(
    cigar: *mut *const u32,
    cigar_max: *const u32,
    icig: *mut hts_pos_t,
    iseq: *mut hts_pos_t,
    iref: *mut hts_pos_t,
) -> i32 {
    let mut pos = *iref;
    if pos < 0 {
        return -1;
    }
    *icig = 0;
    *iseq = 0;
    *iref = 0;
    while *cigar < cigar_max {
        let cig = bam_cigar_op(**cigar);
        let ncig = bam_cigar_oplen(**cigar) as hts_pos_t;
        if cig == BAM_CSOFT_CLIP {
            *cigar = (*cigar).add(1);
            *iseq += ncig;
            *icig = 0;
            continue;
        }
        if cig == BAM_CHARD_CLIP || cig == BAM_CPAD {
            *cigar = (*cigar).add(1);
            *icig = 0;
            continue;
        }
        if cig == BAM_CMATCH || cig == BAM_CEQUAL || cig == BAM_CDIFF {
            pos -= ncig;
            if pos < 0 {
                *icig = ncig + pos;
                *iseq += *icig;
                *iref += *icig;
                return BAM_CMATCH;
            }
            *cigar = (*cigar).add(1);
            *iseq += ncig;
            *icig = 0;
            *iref += ncig;
            continue;
        }
        if cig == BAM_CINS {
            *cigar = (*cigar).add(1);
            *iseq += ncig;
            *icig = 0;
            continue;
        }
        if cig == BAM_CDEL || cig == BAM_CREF_SKIP {
            pos -= ncig;
            if pos < 0 {
                pos = 0;
            }
            *cigar = (*cigar).add(1);
            *icig = 0;
            *iref += ncig;
            continue;
        }
        return -2;
    }
    *iseq = -1;
    -1
}

unsafe fn cigar_iref2iseq_next(
    cigar: *mut *const u32,
    cigar_max: *const u32,
    icig: *mut hts_pos_t,
    iseq: *mut hts_pos_t,
    iref: *mut hts_pos_t,
) -> i32 {
    while *cigar < cigar_max {
        let cig = bam_cigar_op(**cigar);
        let ncig = bam_cigar_oplen(**cigar) as hts_pos_t;
        if cig == BAM_CMATCH || cig == BAM_CEQUAL || cig == BAM_CDIFF {
            if *icig >= ncig - 1 {
                *icig = -1;
                *cigar = (*cigar).add(1);
                continue;
            }
            *iseq += 1;
            *icig += 1;
            *iref += 1;
            return BAM_CMATCH;
        }
        if cig == BAM_CDEL || cig == BAM_CREF_SKIP {
            *cigar = (*cigar).add(1);
            *iref += ncig;
            *icig = -1;
            continue;
        }
        if cig == BAM_CINS {
            *cigar = (*cigar).add(1);
            *iseq += ncig;
            *icig = -1;
            continue;
        }
        if cig == BAM_CSOFT_CLIP {
            *cigar = (*cigar).add(1);
            *iseq += ncig;
            *icig = -1;
            continue;
        }
        if cig == BAM_CHARD_CLIP || cig == BAM_CPAD {
            *cigar = (*cigar).add(1);
            *icig = -1;
            continue;
        }
        return -2;
    }
    *iseq = -1;
    *iref = -1;
    -1
}

unsafe fn tweak_overlap_quality(a: &mut bam1_t, b: &mut bam1_t) -> i32 {
    let mut a_cigar = bam_get_cigar(a);
    let a_cigar_max = a_cigar.add(a.core.n_cigar as usize);
    let mut b_cigar = bam_get_cigar(b);
    let b_cigar_max = b_cigar.add(b.core.n_cigar as usize);
    let mut a_icig = 0;
    let mut a_iseq = 0;
    let mut b_icig = 0;
    let mut b_iseq = 0;
    let a_qual = bam_get_qual(a) as *mut u8;
    let b_qual = bam_get_qual(b) as *mut u8;
    let a_seq = bam_get_seq(a);
    let b_seq = bam_get_seq(b);
    let mut iref = b.core.pos;
    let mut a_iref = iref - a.core.pos;
    let mut b_iref = iref - b.core.pos;

    let mut a_ret = cigar_iref2iseq_set(
        &mut a_cigar,
        a_cigar_max,
        &mut a_icig,
        &mut a_iseq,
        &mut a_iref,
    );
    if a_ret < 0 {
        return if a_ret < -1 { -1 } else { 0 };
    }
    let mut b_ret = cigar_iref2iseq_set(
        &mut b_cigar,
        b_cigar_max,
        &mut b_icig,
        &mut b_iseq,
        &mut b_iref,
    );
    if b_ret < 0 {
        return if b_ret < -1 { -1 } else { 0 };
    }

    let (amul, bmul) = if (__ac_Wang_hash(__ac_X31_hash_string(bam_get_qname(a).cast())) & 1) != 0 {
        (1u8, 0u8)
    } else {
        (0u8, 1u8)
    };

    loop {
        while a_ret >= 0 && a_iref >= 0 && a_iref < iref - a.core.pos {
            a_ret = cigar_iref2iseq_next(
                &mut a_cigar,
                a_cigar_max,
                &mut a_icig,
                &mut a_iseq,
                &mut a_iref,
            );
        }
        if a_ret < 0 {
            return if a_ret < -1 { -1 } else { 0 };
        }
        while b_ret >= 0 && b_iref >= 0 && b_iref < iref - b.core.pos {
            b_ret = cigar_iref2iseq_next(
                &mut b_cigar,
                b_cigar_max,
                &mut b_icig,
                &mut b_iseq,
                &mut b_iref,
            );
        }
        if b_ret < 0 {
            return if b_ret < -1 { -1 } else { 0 };
        }
        if iref < a_iref + a.core.pos {
            iref = a_iref + a.core.pos;
        }
        if iref < b_iref + b.core.pos {
            iref = b_iref + b.core.pos;
        }
        iref += 1;

        if a_iref + a.core.pos != b_iref + b.core.pos {
            if a_iref + a.core.pos < b_iref + b.core.pos
                && b_cigar > bam_get_cigar(b)
                && bam_cigar_op(*b_cigar.sub(1)) == BAM_CDEL
            {
                loop {
                    *a_qual.add(a_iseq as usize) = if amul != 0 {
                        ((*a_qual.add(a_iseq as usize) as f64) * 0.8) as u8
                    } else {
                        0
                    };
                    a_ret = cigar_iref2iseq_next(
                        &mut a_cigar,
                        a_cigar_max,
                        &mut a_icig,
                        &mut a_iseq,
                        &mut a_iref,
                    );
                    if a_ret < 0 {
                        return if a_ret < -1 { -1 } else { 0 };
                    }
                    if a_iref + a.core.pos >= b_iref + b.core.pos {
                        break;
                    }
                }
            } else if a_cigar > bam_get_cigar(a) && bam_cigar_op(*a_cigar.sub(1)) == BAM_CDEL {
                loop {
                    *b_qual.add(b_iseq as usize) = if bmul != 0 {
                        ((*b_qual.add(b_iseq as usize) as f64) * 0.8) as u8
                    } else {
                        0
                    };
                    b_ret = cigar_iref2iseq_next(
                        &mut b_cigar,
                        b_cigar_max,
                        &mut b_icig,
                        &mut b_iseq,
                        &mut b_iref,
                    );
                    if b_ret < 0 {
                        return if b_ret < -1 { -1 } else { 0 };
                    }
                    if b_iref + b.core.pos >= a_iref + a.core.pos {
                        break;
                    }
                }
            } else {
                continue;
            }
        }

        if a_iseq > a.core.l_qseq as hts_pos_t || b_iseq > b.core.l_qseq as hts_pos_t {
            return -1;
        }
        let ai = a_iseq as usize;
        let bi = b_iseq as usize;
        if bam_seqi(a_seq, ai) == bam_seqi(b_seq, bi) {
            let qual = *a_qual.add(ai) as i32 + *b_qual.add(bi) as i32;
            let capped = if qual > 200 { 200 } else { qual } as u8;
            *a_qual.add(ai) = amul * capped;
            *b_qual.add(bi) = bmul * capped;
        } else if *a_qual.add(ai) > *b_qual.add(bi) {
            *a_qual.add(ai) = ((*a_qual.add(ai) as f64) * 0.8) as u8;
            *b_qual.add(bi) = 0;
        } else if *a_qual.add(ai) < *b_qual.add(bi) {
            *b_qual.add(bi) = ((*b_qual.add(bi) as f64) * 0.8) as u8;
            *a_qual.add(ai) = 0;
        } else {
            *a_qual.add(ai) = amul * (((*a_qual.add(ai) as f64) * 0.8) as u8);
            *b_qual.add(bi) = bmul * (((*b_qual.add(bi) as f64) * 0.8) as u8);
        }
    }
}

unsafe fn overlap_push(iter: &mut bam_plp_s, mut node: NonNull<lbnode_t>) -> i32 {
    if iter.overlaps.is_none() {
        return 0;
    }
    let node_ref = node.as_mut();
    if ((node_ref.b.core.flag as i32) & BAM_FMUNMAP) != 0
        || ((node_ref.b.core.flag as i32) & BAM_FPROPER_PAIR) == 0
    {
        return 0;
    }
    if (node_ref.b.core.mtid >= 0 && node_ref.b.core.tid != node_ref.b.core.mtid)
        || (node_ref.b.core.isize.abs() >= 2 * node_ref.b.core.l_qseq as hts_pos_t
            && node_ref.b.core.mpos >= node_ref.end)
    {
        return 0;
    }

    let overlaps = olap_hash_mut(iter.overlaps.unwrap());
    let key = CStr::from_ptr(bam_get_qname(&node_ref.b).cast())
        .to_bytes()
        .to_vec();
    if let Some(a) = overlaps.remove(&key) {
        let a = a.as_ptr();
        let err = tweak_overlap_quality(&mut (*a).b, &mut node_ref.b);
        debug_assert_eq!((*a).end - 1, (*a).s.end);
        err
    } else {
        if node_ref.b.core.mpos >= node_ref.b.core.pos
            || ((node_ref.b.core.flag as i32) & BAM_FPAIRED) != 0 && node_ref.b.core.mpos == -1
        {
            overlaps.insert(key, node);
        }
        0
    }
}

unsafe fn overlap_remove(iter: &mut bam_plp_s, b: Option<&bam1_t>) {
    if iter.overlaps.is_none() {
        return;
    }
    let overlaps = olap_hash_mut(iter.overlaps.unwrap());
    let Some(b) = b else {
        overlaps.clear();
        return;
    };
    if (b.core.flag as i32 & BAM_FUNMAP) != 0 || (b.core.flag as i32 & BAM_FPROPER_PAIR) == 0 {
        return;
    }
    let key = CStr::from_ptr(bam_get_qname(b).cast()).to_bytes();
    overlaps.remove(key);
}

pub(crate) unsafe fn kh_get_m_s2i(h: *const khash_m_s2i_t, key: *const u8) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key.cast()) & mask;
    let last = i;
    let mut step = 0;
    while !kh_isempty((*h).flags, i)
        && (kh_isdel((*h).flags, i) || !cstr_eq(*(*h).keys.add(i as usize), key))
    {
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return (*h).n_buckets;
        }
    }
    if kh_iseither((*h).flags, i) {
        (*h).n_buckets
    } else {
        i
    }
}

pub(crate) unsafe fn kh_get_s2i(h: *const khash_s2i_t, key: *const u8) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key.cast()) & mask;
    let last = i;
    let mut step = 0;
    while !kh_isempty((*h).flags, i)
        && (kh_isdel((*h).flags, i) || !cstr_eq(*(*h).keys.add(i as usize), key))
    {
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return (*h).n_buckets;
        }
    }
    if kh_iseither((*h).flags, i) {
        (*h).n_buckets
    } else {
        i
    }
}

pub(crate) unsafe fn kh_destroy_s2i(h: *mut khash_s2i_t) {
    if h.is_null() {
        return;
    }
    let nb = (*h).n_buckets as usize;
    let nf = if (*h).n_buckets < 16 { 1 } else { ((*h).n_buckets >> 4) as usize };
    if !(*h).flags.is_null() {
        drop(Vec::from_raw_parts((*h).flags, nf, nf));
    }
    if !(*h).keys.is_null() {
        drop(Vec::from_raw_parts((*h).keys, nb, nb));
    }
    if !(*h).vals.is_null() {
        drop(Vec::from_raw_parts((*h).vals, nb, nb));
    }
    drop(Box::from_raw(h));
}

unsafe fn kh_destroy_m_s2i(h: *mut khash_m_s2i_t) {
    if h.is_null() {
        return;
    }
    let nb = (*h).n_buckets as usize;
    let nf = if (*h).n_buckets < 16 { 1 } else { ((*h).n_buckets >> 4) as usize };
    if !(*h).flags.is_null() {
        drop(Vec::from_raw_parts((*h).flags, nf, nf));
    }
    if !(*h).keys.is_null() {
        drop(Vec::from_raw_parts((*h).keys, nb, nb));
    }
    if !(*h).vals.is_null() {
        drop(Vec::from_raw_parts((*h).vals, nb, nb));
    }
    drop(Box::from_raw(h));
}

unsafe fn kh_get_str2int(h: *const khash_m_s2i_t, key: *const u8) -> u32 {
    if h.is_null() || (*h).n_buckets == 0 {
        return if h.is_null() { 0 } else { (*h).n_buckets };
    }
    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key.cast()) & mask;
    let last = i;
    let mut step = 0;
    while !kh_isempty((*h).flags, i)
        && (kh_isdel((*h).flags, i) || !cstr_eq(*(*h).keys.add(i as usize), key))
    {
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return (*h).n_buckets;
        }
    }
    if kh_iseither((*h).flags, i) {
        (*h).n_buckets
    } else {
        i
    }
}

unsafe fn kh_set_occupied(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) &= !(3 << ((i & 0x0f) << 1));
}

unsafe fn kh_put_str2int(h: *mut khash_m_s2i_t, key: *const u8, ret: *mut i32) -> u32 {
    if h.is_null() {
        *ret = -1;
        return 0;
    }
    if (*h).n_occupied >= (*h).upper_bound {
        let mut new_n = if (*h).n_buckets == 0 {
            4
        } else {
            (*h).n_buckets << 1
        };
        if new_n < 4 {
            new_n = 4;
        }
        if kh_resize_str2int(h, new_n) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }

    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key.cast()) & mask;
    let mut site = (*h).n_buckets;
    let last = i;
    let mut step = 0;
    while !kh_isempty((*h).flags, i) {
        if kh_isdel((*h).flags, i) {
            if site == (*h).n_buckets {
                site = i;
            }
        } else if cstr_eq(*(*h).keys.add(i as usize), key) {
            *ret = 0;
            return i;
        }
        step += 1;
        i = (i + step) & mask;
        if i == last {
            break;
        }
    }
    if site == (*h).n_buckets {
        site = i;
    }
    *(*h).keys.add(site as usize) = key.cast_mut();
    if kh_isempty((*h).flags, site) {
        (*h).n_occupied += 1;
        *ret = 1;
    } else {
        *ret = 2;
    }
    kh_set_occupied((*h).flags, site);
    (*h).size += 1;
    site
}

unsafe fn kh_resize_str2int(h: *mut khash_m_s2i_t, new_n_buckets: u32) -> i32 {
    let n_flags = if new_n_buckets < 16 {
        1
    } else {
        new_n_buckets >> 4
    };
    let flags =
        { let mut v: Vec<u32> = vec![0; n_flags as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
    let keys = {
        let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); new_n_buckets as usize];
        let p = v.as_mut_ptr();
        std::mem::forget(v);
        p
    };
    let vals = {
        let mut v: Vec<i32> = vec![0i32; new_n_buckets as usize];
        let p = v.as_mut_ptr();
        std::mem::forget(v);
        p
    };
    for i in 0..n_flags {
        *flags.add(i as usize) = 0xaaaa_aaaa;
    }

    let old_n = (*h).n_buckets;
    let old_flags = (*h).flags;
    let old_keys = (*h).keys;
    let old_vals = (*h).vals;
    let old_n_flags = if old_n < 16 { 1 } else { old_n >> 4 };
    (*h).n_buckets = new_n_buckets;
    (*h).size = 0;
    (*h).n_occupied = 0;
    (*h).upper_bound = (new_n_buckets as f64 * 0.77) as u32;
    (*h).flags = flags;
    (*h).keys = keys;
    (*h).vals = vals;

    for i in 0..old_n {
        if !kh_iseither(old_flags, i) {
            let mut ret = 0;
            let k = kh_put_str2int(h, *old_keys.add(i as usize), &mut ret);
            if ret < 0 {
                return -1;
            }
            *(*h).vals.add(k as usize) = *old_vals.add(i as usize);
        }
    }
    if !old_flags.is_null() {
        drop(Vec::from_raw_parts(
            old_flags,
            old_n_flags as usize,
            old_n_flags as usize,
        ));
    }
    if !old_keys.is_null() {
        drop(Vec::from_raw_parts(old_keys, old_n as usize, old_n as usize));
    }
    if !old_vals.is_null() {
        drop(Vec::from_raw_parts(old_vals, old_n as usize, old_n as usize));
    }
    0
}

pub unsafe fn khash_str2int_init() -> *mut () {
    let h = Box::into_raw(Box::new(std::mem::zeroed::<khash_m_s2i_t>())).cast::<()>()
        .cast::<khash_m_s2i_t>();
    h.cast()
}

pub unsafe fn khash_str2int_destroy(_hash: *mut ()) {
    kh_destroy_m_s2i(_hash.cast());
}

pub unsafe fn khash_str2int_destroy_free(_hash: *mut ()) {
    let hash = _hash.cast::<khash_m_s2i_t>();
    if hash.is_null() {
        return;
    }
    for k in 0..(*hash).n_buckets {
        if !kh_iseither((*hash).flags, k) {
            let key = *(*hash).keys.add(k as usize);
            if !key.is_null() {
                let mut l = 0usize;
                while *key.add(l) != 0 {
                    l += 1;
                }
                drop(Vec::from_raw_parts(key, l + 1, l + 1));
            }
        }
    }
    kh_destroy_m_s2i(hash);
}

pub unsafe fn khash_str2int_has_key(_hash: *mut (), str_: *const u8) -> i32 {
    let hash = _hash.cast::<khash_m_s2i_t>();
    let k = kh_get_str2int(hash, str_);
    (k != (*hash).n_buckets) as i32
}

pub unsafe fn khash_str2int_get(
    _hash: *mut (),
    str_: *const u8,
    value: *mut i32,
) -> i32 {
    let hash = _hash.cast::<khash_m_s2i_t>();
    if hash.is_null() {
        return -1;
    }
    let k = kh_get_str2int(hash, str_);
    if k == (*hash).n_buckets {
        return -1;
    }
    if !value.is_null() {
        *value = *(*hash).vals.add(k as usize);
    }
    0
}

pub unsafe fn khash_str2int_inc(_hash: *mut (), str_: *const u8) -> i32 {
    let hash = _hash.cast::<khash_m_s2i_t>();
    if hash.is_null() {
        return -1;
    }
    let mut ret = 0;
    let k = kh_put_str2int(hash, str_, &mut ret);
    if ret < 0 {
        return -1;
    }
    if ret == 0 {
        return *(*hash).vals.add(k as usize);
    }
    *(*hash).vals.add(k as usize) = (*hash).size as i32 - 1;
    *(*hash).vals.add(k as usize)
}

pub unsafe fn khash_str2int_set(_hash: *mut (), str_: *const u8, value: i32) -> i32 {
    let hash = _hash.cast::<khash_m_s2i_t>();
    if hash.is_null() {
        return -1;
    }
    let mut ret = 0;
    let k = kh_put_str2int(hash, str_, &mut ret);
    if ret < 0 {
        return -1;
    }
    *(*hash).vals.add(k as usize) = value;
    k as i32
}

pub unsafe fn khash_str2int_size(_hash: *mut ()) -> i32 {
    (*_hash.cast::<khash_m_s2i_t>()).size as i32
}

pub(crate) unsafe fn cstr_eq(a: *const u8, b: *const u8) -> bool {
    !a.is_null() && !b.is_null() && CStr::from_ptr(a.cast()) == CStr::from_ptr(b.cast())
}

unsafe fn kh_isempty(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
}

unsafe fn kh_isdel(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
}

unsafe fn kh_iseither(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3) != 0
}

unsafe extern "C" fn sam_c_1631_bam_pseek(fp: *mut std::ffi::c_void, offset: i64, whence: i32) -> i32 {
    bgzf_seek(fp.cast::<BGZF>(), offset, whence) as i32
}

unsafe extern "C" fn sam_c_1638_bam_ptell(fp: *mut std::ffi::c_void) -> i64 {
    let fd = fp.cast::<BGZF>();
    if fd.is_null() {
        return -1;
    }

    ((*fd).block_address << 16) | ((*fd).block_offset as i64 & 0xffff)
}

// original: cram_pseek (htslib/sam.c:1582)
//
// Pseudo-seek used by hts_itr_multi_next on CRAM streams. Mirrors C exactly:
//   - try cram_seek(fd, offset, SEEK_SET)
//   - on failure, retry with `offset - fd->first_container` and SEEK_CUR
//     (handles the case where the iterator hands an absolute file offset and
//     the container chain has already been advanced past it)
//   - on success, stash `offset` into curr_position and tear down the
//     current container so the next decode call starts fresh
//
// Returns 0 on success, -1 on failure. Touches `cram_fd_layout` fields
// directly (curr_position / ctr / ctr_mt / ooc).
//
// `extern "C"` because this is plumbed through hts_itr_multi_next as a
// function pointer matching the `hts_seek_func` C signature.
unsafe extern "C" fn sam_c_1582_cram_pseek(fp: *mut std::ffi::c_void, offset: i64, _whence: i32) -> i32 {
    use crate::htslib_rs::cram::{
        cram_cram_io_c_3705_cram_free_container, cram_fd_layout, cram_seek,
    };
    use crate::htslib_rs::hts::cram_fd;
    let fd = fp.cast::<cram_fd>();
    if fd.is_null() {
        return -1;
    }
    let fdl = fd.cast::<cram_fd_layout>();

    if cram_seek(fd, offset as libc::off_t, libc::SEEK_SET) != 0
        && cram_seek(
            fd,
            {
                #[cfg(windows)]
                {
                    (offset - i64::from((*fdl).first_container)) as libc::off_t
                }
                #[cfg(not(windows))]
                {
                    (offset - (*fdl).first_container) as libc::off_t
                }
            },
            libc::SEEK_CUR,
        ) != 0
    {
        return -1;
    }

    (*fdl).curr_position = offset as libc::off_t;

    if !(*fdl).ctr.is_null() {
        cram_cram_io_c_3705_cram_free_container((*fdl).ctr.cast());
        if !(*fdl).ctr_mt.is_null() && (*fdl).ctr_mt != (*fdl).ctr {
            cram_cram_io_c_3705_cram_free_container((*fdl).ctr_mt.cast());
        }
        (*fdl).ctr = std::ptr::null_mut();
        (*fdl).ctr_mt = std::ptr::null_mut();
        (*fdl).ooc = 0;
    }

    0
}

// original: cram_ptell (htslib/sam.c:1612)
//
// Pseudo-tell paired with cram_pseek. The CRAM disk cursor is only meaningful
// immediately after a fresh seek; otherwise reads consume records from the
// already-fetched container in memory. So we report fd->curr_position, but
// first nudge it forward if the current slice has been fully consumed.
//
// Touches cram_fd_layout / cram_container_layout / cram_slice_layout fields.
//
// `extern "C"` because this is plumbed through hts_itr_multi_next as a
// function pointer matching the `hts_tell_func` C signature.
unsafe extern "C" fn sam_c_1612_cram_ptell(fp: *mut std::ffi::c_void) -> i64 {
    use crate::htslib_rs::cram::cram_fd_layout;
    use crate::htslib_rs::hts::cram_fd;
    let fd = fp.cast::<cram_fd>();
    if fd.is_null() {
        return -1;
    }
    let fdl = fd.cast::<cram_fd_layout>();
    let c = (*fdl).ctr;
    if !c.is_null() {
        let s = (*c).slice;
        if !s.is_null()
            && (*s).max_rec != 0
            && ((*c).curr_slice + (*s).curr_rec / (*s).max_rec) >= ((*c).max_slice + 1)
        {
            (*fdl).curr_position += (*c).offset as libc::off_t + (*c).length as libc::off_t;
        }
    }
    #[cfg(windows)]
    {
        i64::from((*fdl).curr_position)
    }
    #[cfg(not(windows))]
    {
        (*fdl).curr_position
    }
}

// Native equivalent of C `cram_readrec` (htslib/sam.c:1552):
//
//     static int cram_readrec(BGZF *ignored, void *fpv, void *bv,
//                             int *tid, hts_pos_t *beg, hts_pos_t *end)
//     {
//         do {
//             ret = cram_get_bam_seq(fp->fp.cram, &b);
//             if (ret < 0) return cram_eof(fp->fp.cram) ? -1 : -2;
//             if (bam_tag2cigar(b, 1, 1) < 0) return -2;
//             *tid = b->core.tid; *beg = b->core.pos; *end = bam_endpos(b);
//             if (fp->filter)
//                 pass_filter = sam_passes_filter(fp->bam_header, b, fp->filter);
//             else pass_filter = 1;
//         } while (pass_filter == 0);
//         return ret;
//     }
//
// Used as the per-record callback by the CRAM multi-region iterator. Drives
// the native CRAM decode pipeline (`cram_get_bam_seq_native`), matches C's
// eof-vs-error disambiguation, applies the CG-overflow CIGAR fixup, and
// honours the htsFile-level filter expression.
unsafe extern "C" fn sam_c_1552_cram_readrec(
    _ignored: *mut crate::htslib_rs::hts::BGZF,
    fpv: *mut std::ffi::c_void,
    bv: *mut std::ffi::c_void,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> i32 {
    let fp = fpv.cast::<htsFile>();
    let b = bv.cast::<bam1_t>();
    let cram_fd = (*fp).fp.cram;

    loop {
        let ret = crate::htslib_rs::cram::cram_get_bam_seq_native(cram_fd, b);
        if ret < 0 {
            return if crate::htslib_rs::cram::cram_eof(cram_fd) != 0 {
                -1
            } else {
                -2
            };
        }
        if bam_tag2cigar(b, 1, 1) < 0 {
            return -2;
        }

        *tid = (*b).core.tid;
        *beg = (*b).core.pos;
        *end = bam_endpos(b);

        if !(*fp).filter.is_null() {
            let pass =
                sam_c_1535_sam_passes_filter((*fp).bam_header.cast::<sam_hdr_t>(), b, (*fp).filter);
            if pass < 0 {
                return -2;
            }
            if pass == 0 {
                continue;
            }
        }

        return ret;
    }
}

unsafe fn sam_c_1649_index_load(
    fp: *mut htsFile,
    fn_: *const u8,
    fnidx: *const u8,
    flags: i32,
) -> *mut hts_idx_t {
    match (*fp).format.format {
        HTS_FORMAT_BAM | HTS_FORMAT_SAM => hts_idx_load3(fn_.cast(), fnidx.cast(), HTS_FMT_BAI, flags),
        HTS_FORMAT_CRAM => {
            // Native CRAM index loader. Mirrors C:
            //   if (cram_index_load(fp->fp.cram, fn, fnidx) < 0) return NULL;
            //   hts_cram_idx_t *idx = malloc(sizeof(hts_cram_idx_t));
            //   idx->fmt = HTS_FMT_CRAI; idx->cram = fp->fp.cram;
            //   return (hts_idx_t *) idx;
            let _ = flags;
            let cram_fd = (*fp).fp.cram;
            if crate::htslib_rs::cram::cram_cram_index_c_176_cram_index_load(cram_fd, fn_, fnidx)
                < 0
            {
                return std::ptr::null_mut();
            }
            let idx = Box::into_raw(Box::new(std::mem::zeroed::<
                crate::htslib_rs::hts::hts_cram_idx_t,
            >()));
            (*idx).fmt = HTS_FMT_CRAI;
            (*idx).cram = cram_fd;
            idx.cast()
        }
        _ => std::ptr::null_mut(),
    }
}

pub unsafe fn sam_index_load3(
    _fp: *mut htsFile,
    _fn_: *const u8,
    _fnidx: *const u8,
    _flags: i32,
) -> *mut hts_idx_t {
    sam_c_1649_index_load(_fp, _fn_, _fnidx, _flags)
}

pub unsafe fn sam_index_load2(
    _fp: *mut htsFile,
    _fn_: *const u8,
    _fnidx: *const u8,
) -> *mut hts_idx_t {
    sam_index_load3(_fp, _fn_, _fnidx, HTS_IDX_SAVE_REMOTE)
}

pub unsafe fn sam_index_load(_fp: *mut htsFile, _fn_: *const u8) -> *mut hts_idx_t {
    sam_index_load3(_fp, _fn_, std::ptr::null(), HTS_IDX_SAVE_REMOTE)
}

pub unsafe fn sam_itr_queryi(
    _idx: *const hts_idx_t,
    _tid: i32,
    _beg: hts_pos_t,
    _end: hts_pos_t,
) -> *mut hts_itr_t {
    if _idx.is_null() {
        return hts_itr_query(_idx, _tid, _beg, _end, Some(sam_readrec_rest));
    }
    if (*_idx).fmt == HTS_FMT_CRAI {
        return sam_cram_itr_query(_idx, _tid, _beg, _end, Some(sam_readrec));
    }
    hts_itr_query(_idx, _tid, _beg, _end, Some(sam_readrec))
}

// Native equivalent of C `cram_itr_query` (htslib/sam.c:1687).
//
// Builds a dummy iterator suitable for hts_itr_next() which simply invokes the
// readrec function. For tid>=0 or HTS_IDX_NOCOOR/HTS_IDX_START it sets the CRAM
// reference range via cram_seek_to_refpos (mirroring cram_set_option(CRAM_OPT_RANGE)).
unsafe fn sam_cram_itr_query(
    idx: *const hts_idx_t,
    tid: i32,
    beg: hts_pos_t,
    end: hts_pos_t,
    readrec: crate::htslib_rs::hts::hts_readrec_func,
) -> *mut hts_itr_t {
    use crate::htslib_rs::hts::{hts_cram_idx_t, HTS_IDX_NONE, HTS_IDX_REST};
    let cidx = idx.cast::<hts_cram_idx_t>();
    let iter = Box::into_raw(Box::new(std::mem::zeroed::<hts_itr_t>())).cast::<()>()
        .cast::<hts_itr_t>();
    if iter.is_null() {
        return std::ptr::null_mut();
    }

    // bitfields layout (matches hts.c definition):
    //   bit 0 = read_rest, bit 1 = finished, bit 2 = is_cram, bit 3 = nocoor.
    (*iter).bitfields |= (1 << 2) | 1; // is_cram | read_rest
    (*iter).off = std::ptr::null_mut();
    (*iter).bins.a = std::ptr::null_mut();
    (*iter).readrec = readrec;

    if tid >= 0 || tid == HTS_IDX_NOCOOR || tid == HTS_IDX_START {
        let mut r = crate::htslib_rs::cram::cram_range_layout {
            refid: tid,
            start: beg + 1,
            end,
        };
        // Replicates C cram_set_option(CRAM_OPT_RANGE, &r): call
        // cram_seek_to_refpos, then OR SAM_POS into required_fields if not
        // the special "-2" case set by HTS_IDX_START/HTS_IDX_REST.
        let cram_fd = (*cidx).cram;
        let ret = crate::htslib_rs::cram::cram_seek_to_refpos(
            &mut *cram_fd.cast::<crate::htslib_rs::cram::cram_fd_layout>(),
            &mut r,
        );
        // After cram_seek_to_refpos, propagate required_fields |= SAM_POS like
        // cram_set_voption does. We touch the layout-mirrored field directly.
        cram_set_required_pos_if_needed(cram_fd);

        (*iter).curr_off = 0;
        (*iter).tid = tid;
        (*iter).beg = beg;
        (*iter).end = end;

        match ret {
            0 => {}
            -2 => {
                // No data vs this ref; mark iterator finished (same as HTS_IDX_NONE).
                (*iter).bitfields |= 1 << 1;
            }
            _ => {
                drop(Box::from_raw(iter));
                return std::ptr::null_mut();
            }
        }
    } else {
        match tid {
            HTS_IDX_REST => {
                (*iter).curr_off = 0;
            }
            HTS_IDX_NONE => {
                (*iter).curr_off = 0;
                (*iter).bitfields |= 1 << 1; // finished
            }
            _ => {
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_ERROR,
                    b"cram_itr_query",
                    b"Query with this tid not implemented for CRAM files",
                );
                drop(Box::from_raw(iter));
                return std::ptr::null_mut();
            }
        }
    }

    iter
}

// Mirrors the post-cram_seek_to_refpos work done inside C's
// cram_set_voption(CRAM_OPT_RANGE): OR SAM_POS into required_fields unless the
// special "refid == -2" sentinel is set (which happens for HTS_IDX_START/REST).
// The cram_fd_layout struct is private to cram.rs, so we route through a
// dedicated setter there.
unsafe fn cram_set_required_pos_if_needed(fd: *mut crate::htslib_rs::hts::cram_fd) {
    crate::htslib_rs::cram::cram_set_required_fields_pos(fd);
}

pub unsafe fn sam_itr_querys(
    idx: *const hts_idx_t,
    hdr: *mut sam_hdr_t,
    region: *const u8,
) -> *mut hts_itr_t {
    if idx.is_null() || hdr.is_null() || region.is_null() {
        return std::ptr::null_mut();
    }
    // Mirrors C `sam_itr_querys` (htslib/sam.c:1761): parses the region string
    // and routes to the appropriate itr_query implementation. The
    // sam_itr_queryi() dispatch below handles BAM/SAM and CRAM uniformly.
    if libc::strcmp(region.cast(), c".".as_ptr()) == 0 {
        return sam_itr_queryi(idx, HTS_IDX_START, 0, 0);
    }
    if libc::strcmp(region.cast(), c"*".as_ptr()) == 0 {
        return sam_itr_queryi(idx, HTS_IDX_NOCOOR, 0, 0);
    }

    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;
    if sam_parse_region(
        hdr,
        region,
        &mut tid,
        &mut beg,
        &mut end,
        HTS_PARSE_THOUSANDS_SEP,
    )
    .is_null()
    {
        return std::ptr::null_mut();
    }
    sam_itr_queryi(idx, tid, beg, end)
}

unsafe extern "C" fn sam_c_1754_cram_name2id(
    fdv: *mut std::ffi::c_void,
    ref_: *const std::ffi::c_char,
) -> i32 {
    if fdv.is_null() || ref_.is_null() {
        return -1;
    }
    let hdr = match crate::htslib_rs::cram::cram_fd_get_header(&*fdv.cast()) {
        Some(h) => h.as_ptr(),
        None => return -1,
    };
    sam_hdr_name2tid(&mut *hdr, CStr::from_ptr(ref_).to_bytes())
}

pub unsafe fn sam_c_1768_sam_itr_regarray(
    idx: *const hts_idx_t,
    hdr: *mut sam_hdr_t,
    regarray: *mut *mut u8,
    regcount: u32,
) -> *mut hts_itr_t {
    if idx.is_null() || hdr.is_null() {
        return std::ptr::null_mut();
    }

    let is_cram = (*idx).fmt == HTS_FMT_CRAI;
    let getid: hts_name2id_f = if is_cram {
        Some(sam_c_1754_cram_name2id)
    } else {
        Some(sam_c_418_bam_name2id_wrapper)
    };
    let hdr_arg: *mut std::ffi::c_void = if is_cram {
        (*idx.cast::<crate::htslib_rs::hts::hts_cram_idx_t>())
            .cram
            .cast()
    } else {
        hdr.cast()
    };
    let multi_query: hts_itr_multi_query_func = if is_cram {
        Some(hts_itr_multi_cram)
    } else {
        Some(hts_itr_multi_bam)
    };
    let readrec: hts_readrec_func = if is_cram {
        Some(sam_c_1552_cram_readrec)
    } else {
        Some(sam_readrec)
    };
    let seek: hts_seek_func = if is_cram {
        Some(sam_c_1582_cram_pseek)
    } else {
        Some(sam_c_1631_bam_pseek)
    };
    let tell: hts_tell_func = if is_cram {
        Some(sam_c_1612_cram_ptell)
    } else {
        Some(sam_c_1638_bam_ptell)
    };

    let mut reg_count = 0;
    let reglist = hts_reglist_create(regarray.cast(), regcount as i32, &mut reg_count, hdr_arg, getid);
    if reglist.is_null() {
        return std::ptr::null_mut();
    }
    let itr = hts_itr_regions(
        idx,
        std::slice::from_raw_parts_mut(reglist, reg_count as usize),
        getid,
        hdr_arg,
        multi_query,
        readrec,
        seek,
        tell,
    );
    if itr.is_null() {
        hts_reglist_free(reglist, reg_count);
    }
    itr
}

pub unsafe fn sam_c_1798_sam_itr_regions(
    idx: *const hts_idx_t,
    hdr: *mut sam_hdr_t,
    reglist: *mut hts_reglist_t,
    regcount: u32,
) -> *mut hts_itr_t {
    if idx.is_null() || hdr.is_null() || reglist.is_null() {
        return std::ptr::null_mut();
    }

    let is_cram = (*idx).fmt == HTS_FMT_CRAI;
    let getid: hts_name2id_f = if is_cram {
        Some(sam_c_1754_cram_name2id)
    } else {
        Some(sam_c_418_bam_name2id_wrapper)
    };
    let hdr_arg: *mut std::ffi::c_void = if is_cram {
        (*idx.cast::<crate::htslib_rs::hts::hts_cram_idx_t>())
            .cram
            .cast()
    } else {
        hdr.cast()
    };
    let multi_query: hts_itr_multi_query_func = if is_cram {
        Some(hts_itr_multi_cram)
    } else {
        Some(hts_itr_multi_bam)
    };
    let readrec: hts_readrec_func = if is_cram {
        Some(sam_c_1552_cram_readrec)
    } else {
        Some(sam_readrec)
    };
    let seek: hts_seek_func = if is_cram {
        Some(sam_c_1582_cram_pseek)
    } else {
        Some(sam_c_1631_bam_pseek)
    };
    let tell: hts_tell_func = if is_cram {
        Some(sam_c_1612_cram_ptell)
    } else {
        Some(sam_c_1638_bam_ptell)
    };

    hts_itr_regions(
        idx,
        std::slice::from_raw_parts_mut(reglist, regcount as usize),
        getid,
        hdr_arg,
        multi_query,
        readrec,
        seek,
        tell,
    )
}

unsafe fn sam_c_994_sam_index(fp: *mut htsFile, mut min_shift: i32) -> *mut hts_idx_t {
    let h = sam_hdr_read(fp);
    if h.is_null() {
        return std::ptr::null_mut();
    }

    let (fmt, n_lvls) = if min_shift > 0 {
        let mut max_len = 0;
        for i in 0..(*h).n_targets {
            let len = sam_hdr_tid2len(&*h, i);
            if max_len < len {
                max_len = len;
            }
        }

        let max_n_lvls = 9;
        let mut n_lvls = 0;
        let max_len_adjusted = max_len + 256;
        if max_len_adjusted <= hts_bin_maxpos(min_shift, max_n_lvls) {
            let mut maxpos = hts_bin_maxpos(min_shift, n_lvls);
            while max_len_adjusted > maxpos {
                n_lvls += 1;
                maxpos *= 8;
            }
        } else {
            n_lvls = max_n_lvls;
            let mut maxpos = hts_bin_maxpos(min_shift, n_lvls);
            while max_len_adjusted > maxpos {
                min_shift += 1;
                maxpos *= 2;
            }
        }
        (HTS_FMT_CSI, n_lvls)
    } else {
        min_shift = 14;
        (HTS_FMT_BAI, 5)
    };

    let idx = hts_idx_init(
        (*h).n_targets,
        fmt,
        sam_c_1638_bam_ptell((*fp).fp.bgzf.cast()) as u64,
        min_shift,
        n_lvls,
    );
    let b = bam_init1();
    let mut ret = sam_read1(fp, h, b);
    while ret >= 0 {
        ret = hts_idx_push(
            idx,
            (*b).core.tid,
            (*b).core.pos,
            bam_endpos(b),
            sam_c_1638_bam_ptell((*fp).fp.bgzf.cast()) as u64,
            (((*b).core.flag as i32 & BAM_FUNMAP) == 0) as i32,
        );
        if ret < 0 {
            bam_destroy1(b);
            hts_idx_destroy(idx);
            return std::ptr::null_mut();
        }
        ret = sam_read1(fp, h, b);
    }
    if ret < -1 {
        bam_destroy1(b);
        hts_idx_destroy(idx);
        return std::ptr::null_mut();
    }

    hts_idx_finish(idx, sam_c_1638_bam_ptell((*fp).fp.bgzf.cast()) as u64);
    sam_hdr_destroy(h);
    bam_destroy1(b);
    idx
}

pub unsafe fn sam_index_build3(
    fn_: *const u8,
    fnidx: *const u8,
    min_shift: i32,
    nthreads: i32,
) -> i32 {
    let fp = crate::htslib_rs::hts::hts_open(fn_.cast(), c"r".as_ptr());
    if fp.is_null() {
        return -2;
    }

    // For non-CRAM formats apply threading early (matches C order). CRAM uses
    // a single-pass reader for index build, so we skip thread setup there to
    // avoid wiring nthreads through hts_sys (the native cram_index_build does
    // not benefit from threads).
    if (*fp).format.format != HTS_FORMAT_CRAM && nthreads != 0 {
        crate::htslib_rs::hts::hts_set_threads(fp, nthreads);
    }

    if (*fp).format.format == HTS_FORMAT_CRAM {
        // Native CRAM index builder. Mirrors C:
        //     ret = cram_index_build(fp->fp.cram, fn, fnidx);
        let ret = crate::htslib_rs::cram::cram_cram_index_c_779_cram_index_build(
            (*fp).fp.cram,
            fn_,
            fnidx,
        );
        crate::htslib_rs::hts::hts_close(fp);
        return ret;
    }

    let ret = match (*fp).format.format {
        HTS_FORMAT_BAM | HTS_FORMAT_SAM => {
            if (*fp).format.compression != crate::htslib_rs::hts::HTS_COMPRESSION_BGZF {
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_ERROR,
                    b"sam_index_build3",
                    if (*fp).format.format == HTS_FORMAT_BAM {
                        b"BAM file not BGZF compressed" as &[u8]
                    } else {
                        b"SAM file not BGZF compressed"
                    },
                );
                -1
            } else {
                let idx = sam_c_994_sam_index(fp, min_shift);
                if idx.is_null() {
                    -1
                } else {
                    let mut ret = hts_idx_save_as(
                        idx,
                        fn_.cast(),
                        fnidx.cast(),
                        if min_shift > 0 {
                            HTS_FMT_CSI
                        } else {
                            HTS_FMT_BAI
                        },
                    );
                    if ret < 0 {
                        ret = -4;
                    }
                    hts_idx_destroy(idx);
                    ret
                }
            }
        }
        _ => -3,
    };
    crate::htslib_rs::hts::hts_close(fp);

    ret
}

pub unsafe fn sam_index_build2(
    fn_: *const u8,
    fnidx: *const u8,
    min_shift: i32,
) -> i32 {
    sam_index_build3(fn_, fnidx, min_shift, 0)
}

pub unsafe fn sam_index_build(fn_: *const u8, min_shift: i32) -> i32 {
    sam_index_build3(fn_, std::ptr::null(), min_shift, 0)
}

pub unsafe fn bam_index_build(fn_: *const u8, min_shift: i32) -> i32 {
    sam_index_build2(fn_, std::ptr::null(), min_shift)
}

// original: sam_idx_init (htslib/sam.c:1096)
//
// Native equivalent of htslib's sam_idx_init. For BAM / BCF / BGZF-SAM,
// allocates a hts_idx_t sized for the header's refs (CSI if min_shift>0,
// else BAI with min_shift=14, n_lvls=5). For CRAM, opens the index sidecar
// as a bgzip-output stream into the cram_fd's `idxfp`. Other formats are
// unindexable here and return -1.
pub unsafe fn sam_idx_init(
    fp: *mut htsFile,
    h: *mut sam_hdr_t,
    min_shift: i32,
    fnidx: *const u8,
) -> i32 {
    (*fp).fnidx = fnidx.cast();
    let fmt_kind = (*fp).format.format;
    if fmt_kind == HTS_FORMAT_BAM
        || fmt_kind == crate::htslib_rs::hts::HTS_FORMAT_BCF
        || (fmt_kind == HTS_FORMAT_SAM
            && (*fp).format.compression == crate::htslib_rs::hts::HTS_COMPRESSION_BGZF)
    {
        let mut fmt = HTS_FMT_CSI;
        let mut min_shift = min_shift;
        let n_lvls;
        if min_shift > 0 {
            let mut max_len: i64 = 0;
            for i in 0..(*h).n_targets {
                let len = *(*h).target_len.add(i as usize) as i64;
                if max_len < len {
                    max_len = len;
                }
            }
            let mut nl: i32 = 0;
            crate::htslib_rs::hts::hts_c_2372_hts_adjust_csi_settings(
                max_len,
                &mut min_shift,
                &mut nl,
            );
            n_lvls = nl;
        } else {
            min_shift = 14;
            n_lvls = 5;
            fmt = HTS_FMT_BAI;
        }
        // bgzf_tell macro: ((block_address << 16) | (block_offset & 0xFFFF))
        let bgzf = (*fp).fp.bgzf;
        let offset0 =
            (((*bgzf).block_address as u64) << 16) | ((*bgzf).block_offset as u64 & 0xFFFF);
        (*fp).idx =
            crate::htslib_rs::hts::hts_idx_init((*h).n_targets, fmt, offset0, min_shift, n_lvls)
                .cast();
        return if (*fp).idx.is_null() { -1 } else { 0 };
    }

    if fmt_kind == HTS_FORMAT_CRAM {
        let cram_fp = (*fp).fp.cram;
        let idxfp = crate::htslib_rs::bgzf::bgzf_open(fnidx.cast(), c"wg".as_ptr());
        crate::htslib_rs::cram::cram_fd_idxfp_set(cram_fp, idxfp);
        return if idxfp.is_null() { -1 } else { 0 };
    }

    -1
}

// original: sam_idx_save (htslib/sam.c:1124)
//
// For SAM/BAM/VCF/BCF: drain the writer state, flush the underlying BGZF
// stream, finalize the index and write it to `fp->fnidx`. For CRAM the
// index is flushed/closed by `cram_close`, so this is a no-op (returns 0).
pub unsafe fn sam_idx_save(fp: *mut htsFile) -> i32 {
    let fmt_kind = (*fp).format.format;
    if fmt_kind == HTS_FORMAT_BAM
        || fmt_kind == crate::htslib_rs::hts::HTS_FORMAT_BCF
        || fmt_kind == crate::htslib_rs::hts::HTS_FORMAT_VCF
        || fmt_kind == HTS_FORMAT_SAM
    {
        let ret = sam_state_destroy(fp);
        if ret < 0 {
            *libc::__errno_location() = -ret;
            return -1;
        }
        // htsFile.is_bgzf is the `is_bgzf` bit in bitfields (bit 4); see the
        // htsFile struct layout in src/hts.rs.
        let is_bgzf = ((*fp).bitfields & (1 << 4)) != 0;
        let bgzf = (*fp).fp.bgzf;
        if !is_bgzf || crate::htslib_rs::bgzf::bgzf_flush(bgzf) < 0 {
            return -1;
        }
        // bgzf_tell macro: ((block_address << 16) | (block_offset & 0xFFFF))
        let pos = (((*bgzf).block_address as u64) << 16) | ((*bgzf).block_offset as u64 & 0xFFFF);
        crate::htslib_rs::hts::hts_c_2682_hts_idx_amend_last((*fp).idx.cast(), pos);
        if crate::htslib_rs::hts::hts_idx_finish((*fp).idx.cast(), pos) < 0 {
            return -1;
        }
        return crate::htslib_rs::hts::hts_c_2894_hts_idx_save_but_not_close(
            (*fp).idx.cast(),
            (*fp).fnidx,
            crate::htslib_rs::hts::hts_idx_fmt((*fp).idx.cast()),
        );
    }
    // CRAM index flush is handled by cram_close.
    0
}

pub unsafe fn sam_itr_next(_htsfp: *mut htsFile, _itr: *mut hts_itr_t, _r: *mut bam1_t) -> i32 {
    if ((*_htsfp).bitfields & (1 << 4)) == 0 && ((*_htsfp).bitfields & (1 << 3)) == 0 {
        return -2;
    }
    if _itr.is_null() {
        return -2;
    }
    if ((*_itr).bitfields & (1 << 4)) != 0 {
        return hts_itr_multi_next(_htsfp, _itr, _r.cast());
    }
    let fp = if ((*_htsfp).bitfields & (1 << 4)) != 0 {
        (*_htsfp).fp.bgzf
    } else {
        std::ptr::null_mut()
    };
    hts_itr_next(fp, _itr.cast(), _r.cast(), _htsfp.cast::<std::ffi::c_void>())
}

unsafe fn sam_read1_bam(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> i32 {
    let ret = bam_read1((*fp).fp.bgzf, b);
    if !h.is_null()
        && ret >= 0
        && ((*b).core.tid >= (*h).n_targets
            || (*b).core.tid < -1
            || (*b).core.mtid >= (*h).n_targets
            || (*b).core.mtid < -1)
    {
        *libc::__errno_location() =
            libc::ERANGE as i32;
        return -3;
    }
    ret
}

// Native equivalent of C `sam_read1_cram` (htslib/sam.c:4147):
//
//     static inline int sam_read1_cram(htsFile *fp, sam_hdr_t *h, bam1_t **b)
//     {
//         int ret = cram_get_bam_seq(fp->fp.cram, b);
//         if (ret < 0)
//             return cram_eof(fp->fp.cram) ? -1 : -2;
//         if (bam_tag2cigar(*b, 1, 1) < 0)
//             return -2;
//         return ret;
//     }
//
// We drive the **native** CRAM decode pipeline (`cram_get_bam_seq_native`)
// instead of the C `cram_get_bam_seq`, then perform the same eof-vs-error
// disambiguation and CG-overflow CIGAR fixup. The sam_hdr_t arg is unused at
// this layer — the cram_fd carries its own header copy and the native
// pipeline reads tid/mtid from the cram_fd's header / cram_slice metadata
// (see decode_pipeline::cram_to_bam), exactly mirroring the C behaviour.
unsafe fn sam_read1_cram_native_decode(
    fp: *mut htsFile,
    _h: *mut sam_hdr_t,
    b: *mut bam1_t,
) -> i32 {
    let cram_fd = (*fp).fp.cram;
    let ret = crate::htslib_rs::cram::cram_get_bam_seq_native(cram_fd, b);
    if ret < 0 {
        return if crate::htslib_rs::cram::cram_eof(cram_fd) != 0 {
            -1
        } else {
            -2
        };
    }
    if bam_tag2cigar(b, 1, 1) < 0 {
        return -2;
    }
    ret
}

unsafe fn sam_c_4145_sam_read1_cram(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> i32 {
    if let Some((pending, ret)) = sam_cram_pending_pop(fp) {
        let copied = bam_copy1(b, pending);
        bam_destroy1(pending);
        return if copied.is_null() { -1 } else { ret };
    }

    let ret = if let Some((lookahead, ret)) = sam_cram_lookahead_take(fp) {
        let copied = bam_copy1(b, lookahead);
        bam_destroy1(lookahead);
        if copied.is_null() {
            return -1;
        }
        ret
    } else {
        sam_read1_cram_native_decode(fp, h, b)
    };
    if ret < 0 || !sam_cram_tlen_candidate(b) {
        return ret;
    }

    let first_qname = CStr::from_ptr(bam_get_qname(b).cast()).to_bytes().to_vec();
    let mut group = vec![b];
    let mut group_rets = vec![ret];
    let mut next_group: Option<(*mut bam1_t, i32)> = None;

    loop {
        let next = bam_init1();
        if next.is_null() {
            break;
        }
        let next_ret = sam_read1_cram_native_decode(fp, h, next);
        if next_ret < 0 {
            bam_destroy1(next);
            break;
        }
        if CStr::from_ptr(bam_get_qname(next).cast()).to_bytes() == first_qname.as_slice() {
            group.push(next);
            group_rets.push(next_ret);
        } else {
            next_group = Some((next, next_ret));
            break;
        }
    }

    sam_fix_cram_group_tlen(&group);
    for (rec, rec_ret) in group.iter().copied().zip(group_rets).skip(1) {
        sam_cram_pending_push(fp, rec, rec_ret);
    }
    if let Some((rec, rec_ret)) = next_group {
        sam_cram_lookahead_store(fp, rec, rec_ret);
    }

    ret
}

type SamCramPendingMap = HashMap<usize, VecDeque<(usize, i32)>>;

fn sam_cram_pending() -> &'static Mutex<SamCramPendingMap> {
    static PENDING: OnceLock<Mutex<SamCramPendingMap>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe fn sam_cram_pending_pop(fp: *mut htsFile) -> Option<(*mut bam1_t, i32)> {
    let mut pending = sam_cram_pending().lock().unwrap();
    let queue = pending.get_mut(&(fp as usize))?;
    let (rec, ret) = queue.pop_front()?;
    if queue.is_empty() {
        pending.remove(&(fp as usize));
    }
    Some((rec as *mut bam1_t, ret))
}

unsafe fn sam_cram_pending_push(fp: *mut htsFile, rec: *mut bam1_t, ret: i32) {
    let mut pending = sam_cram_pending().lock().unwrap();
    pending
        .entry(fp as usize)
        .or_default()
        .push_back((rec as usize, ret));
}

fn sam_cram_lookahead() -> &'static Mutex<HashMap<usize, (usize, i32)>> {
    static LOOKAHEAD: OnceLock<Mutex<HashMap<usize, (usize, i32)>>> = OnceLock::new();
    LOOKAHEAD.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe fn sam_cram_lookahead_take(fp: *mut htsFile) -> Option<(*mut bam1_t, i32)> {
    let mut lookahead = sam_cram_lookahead().lock().unwrap();
    lookahead
        .remove(&(fp as usize))
        .map(|(rec, ret)| (rec as *mut bam1_t, ret))
}

unsafe fn sam_cram_lookahead_store(fp: *mut htsFile, rec: *mut bam1_t, ret: i32) {
    let mut lookahead = sam_cram_lookahead().lock().unwrap();
    if let Some((old, _)) = lookahead.insert(fp as usize, (rec as usize, ret)) {
        bam_destroy1(old as *mut bam1_t);
    }
}

unsafe fn sam_cram_tlen_candidate(b: *const bam1_t) -> bool {
    let c = &(*b).core;
    c.tid >= 0 && c.mtid == c.tid && c.isize != 0 && c.n_cigar != 0
}

unsafe fn sam_cram_record_right(b: *const bam1_t) -> Option<hts_pos_t> {
    let c = &(*b).core;
    let rlen = bam_cigar2rlen(c.n_cigar as i32, bam_get_cigar(b));
    if rlen <= 0 {
        None
    } else {
        Some(c.pos + rlen - 1)
    }
}

unsafe fn sam_fix_cram_group_tlen(group: &[*mut bam1_t]) {
    if group.len() < 2 || !group.iter().all(|&b| sam_cram_tlen_candidate(b)) {
        return;
    }

    let ref_id = (*group[0]).core.tid;
    if !group.iter().all(|&b| (*b).core.tid == ref_id) {
        for &b in group {
            (*b).core.isize = 0;
        }
        return;
    }

    let mut aleft = (*group[0]).core.pos;
    let mut aright = match sam_cram_record_right(group[0]) {
        Some(right) => right,
        None => return,
    };
    let mut left_cnt = 0;
    let mut right_cnt = 0;

    for &b in group {
        let pos = (*b).core.pos;
        let Some(right) = sam_cram_record_right(b) else {
            return;
        };
        if pos < aleft {
            aleft = pos;
            left_cnt = 1;
        } else if pos == aleft {
            left_cnt += 1;
        }
        if right > aright {
            aright = right;
            right_cnt = 1;
        } else if right == aright {
            right_cnt += 1;
        }
    }

    let mut tlen = aright - aleft + 1;
    if group.iter().any(|&b| (*b).core.isize.abs() != tlen) {
        return;
    }

    let first = group[0];
    let first_right = match sam_cram_record_right(first) {
        Some(right) => right,
        None => return,
    };
    if (*first).core.pos == aleft && (first_right < aright || left_cnt <= 1) {
        (*first).core.isize = tlen;
        tlen = -tlen;
    } else if (*first).core.pos == aleft && first_right == aright && left_cnt > 1 && right_cnt > 1 {
        if ((*first).core.flag as i32 & BAM_FREAD1) != 0 {
            (*first).core.isize = tlen;
            tlen = -tlen;
        } else {
            (*first).core.isize = -tlen;
        }
    } else {
        (*first).core.isize = -tlen;
    }

    for &b in group.iter().skip(1) {
        (*b).core.isize = tlen;
    }
}

pub unsafe fn sam_c_3719_sam_set_thread_pool(
    fp: *mut htsFile,
    p: *mut crate::htslib_rs::hts::htsThreadPool,
) -> i32 {
    if fp.is_null() || p.is_null() {
        return -1;
    }
    if !(*fp).state.is_null() {
        return -2;
    }
    if (*p).pool.is_null() {
        return -1;
    }
    0
}

pub unsafe fn sam_c_3746_sam_set_threads(fp: *mut htsFile, nthreads: i32) -> i32 {
    if nthreads <= 0 {
        return 0;
    }
    if fp.is_null() {
        return -1;
    }
    // Native SAM thread-state setup is not implemented: htslib uses pthread
    // mutex/cond fields inside SAM_state which our struct doesn't carry, so
    // sharing layout with libhts isn't possible. Decoded SAM goes single-
    // threaded — the same behaviour we had via the previous hts_sys
    // delegation (no test exercises threaded SAM decoding). For BAM the
    // bgzf-side worker pool (driven by hts.rs's `hts_set_threads` BGZF
    // branch via `bgzf_mt`) provides the actual parallelism.
    0
}

pub unsafe fn bam_read1(fp: *mut BGZF, b: *mut bam1_t) -> i32 {
    // Fully native BAM record reader over the (now native) bgzf layer; the C
    // fast-read delegation was removed (the open-time fast-read flag is inert).
    let c = &mut (*b).core;
    let mut block_len_buf = [0u8; 4];
    let mut core_buf = [0u8; 32];

    (*b).data.clear();

    let ret = bgzf_read_small(fp, block_len_buf.as_mut_ptr().cast(), 4);
    if ret != 4 {
        return if ret == 0 { -1 } else { -2 };
    }
    let block_len = i32::from_le_bytes(block_len_buf);
    if block_len < 32 {
        return -4;
    }

    let x = if (*fp).block_length - (*fp).block_offset > 32 {
        let ptr = (*fp)
            .uncompressed_block
            .as_ptr()
            .add((*fp).block_offset as usize);
        (*fp).block_offset += 32;
        ptr
    } else {
        if bgzf_read(fp.cast(), core_buf.as_mut_ptr().cast(), 32) != 32 {
            return -3;
        }
        core_buf.as_ptr()
    };

    c.tid = i32::from_le_bytes([*x, *x.add(1), *x.add(2), *x.add(3)]);
    c.pos = i32::from_le_bytes([*x.add(4), *x.add(5), *x.add(6), *x.add(7)]) as hts_pos_t;
    let x2 = u32::from_le_bytes([*x.add(8), *x.add(9), *x.add(10), *x.add(11)]);
    c.bin = (x2 >> 16) as u16;
    c.qual = ((x2 >> 8) & 0xff) as u8;
    c.l_qname = (x2 & 0xff) as u16;
    c.l_extranul = if !c.l_qname.is_multiple_of(4) {
        (4 - c.l_qname % 4) as u8
    } else {
        0
    };
    let x3 = u32::from_le_bytes([*x.add(12), *x.add(13), *x.add(14), *x.add(15)]);
    c.flag = (x3 >> 16) as u16;
    c.n_cigar = x3 & 0xffff;
    c.l_qseq = i32::from_le_bytes([*x.add(16), *x.add(17), *x.add(18), *x.add(19)]);
    c.mtid = i32::from_le_bytes([*x.add(20), *x.add(21), *x.add(22), *x.add(23)]);
    c.mpos = i32::from_le_bytes([*x.add(24), *x.add(25), *x.add(26), *x.add(27)]) as hts_pos_t;
    c.isize = i32::from_le_bytes([*x.add(28), *x.add(29), *x.add(30), *x.add(31)]) as hts_pos_t;

    let new_l_data = block_len - 32 + c.l_extranul as i32;
    if c.l_qseq < 0 || c.l_qname < 1 {
        return -4;
    }
    let min_l_data = ((c.n_cigar as u64) << 2)
        + c.l_qname as u64
        + c.l_extranul as u64
        + (((c.l_qseq as u64) + 1) >> 1)
        + c.l_qseq as u64;
    if new_l_data < 0 || min_l_data > new_l_data as u64 {
        return -4;
    }
    if realloc_bam_data(b, new_l_data as usize) < 0 {
        return -4;
    }
    // realloc_bam_data already resized data.len() to new_l_data (== old l_data).

    if bgzf_read_small(fp, (*b).data.as_mut_ptr().cast(), c.l_qname as usize) != c.l_qname as isize {
        return -4;
    }
    if *(*b).data.as_ptr().add(c.l_qname as usize - 1) != 0 && fixup_missing_qname_nul(b) < 0 {
        return -4;
    }
    for i in 0..c.l_extranul {
        *(*b).data.as_mut_ptr().add(c.l_qname as usize + i as usize) = 0;
    }
    c.l_qname += c.l_extranul as u16;

    if (*b).data.len() < c.l_qname as usize {
        return -4;
    }
    let rest = (*b).data.len() - c.l_qname as usize;
    if bgzf_read_small(fp, (*b).data.as_mut_ptr().add(c.l_qname as usize).cast(), rest) != rest as isize {
        return -4;
    }
    if bam_tag2cigar(b, 0, 0) < 0 {
        return -4;
    }

    if c.n_cigar > 0 {
        let mut rlen = 0;
        let mut qlen = 0;
        bam_cigar2rqlens(c.n_cigar as i32, bam_get_cigar(b), &mut rlen, &mut qlen);
        if (c.flag as i32 & BAM_FUNMAP) != 0 || rlen == 0 {
            rlen = 1;
        }
        c.bin = hts_reg2bin(c.pos, c.pos + rlen, 14, 5) as u16;
        if c.l_qseq > 0 && (c.flag as i32 & BAM_FUNMAP) == 0 && qlen != c.l_qseq as hts_pos_t {
            return -4;
        }
    }

    4 + block_len
}

unsafe fn sam_c_4157_sam_read1_sam(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> i32 {
    if !(*fp).line.data.is_empty() {
        let ret = sam_c_2662_sam_parse1(&mut (*fp).line, h, b);
        (*fp).line.data.truncate(0);
        return ret;
    }

    // Threaded SAM decoder is intentionally not implemented (see
    // sam_c_3746_sam_set_threads); fp.state stays null for SAM/text formats,
    // so the threaded read pump is unreachable from this branch.

    loop {
        let ret = crate::htslib_rs::hts::hts_getline(fp, 2, &mut (*fp).line);
        if ret < 0 {
            return ret;
        }

        let ret = sam_c_2662_sam_parse1(&mut (*fp).line, h, b);
        (*fp).line.data.truncate(0);
        if ret >= 0 {
            return ret;
        }
        if h.is_null() || (*h).ignore_sam_err == 0 {
            return ret;
        }
    }
}

pub unsafe fn sam_read1(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> i32 {
    if !(*fp).filter.is_null() {
        loop {
            let ret = sam_read1_unfiltered(fp, h, b);
            if ret < 0 {
                return ret;
            }
            let pass = sam_c_1535_sam_passes_filter(h, b, (*fp).filter);
            if pass < 0 {
                return -3;
            }
            if pass != 0 {
                return ret;
            }
        }
    }
    sam_read1_unfiltered(fp, h, b)
}

unsafe fn sam_read1_unfiltered(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> i32 {
    if fp.is_null() || b.is_null() {
        return -3;
    }
    match (*fp).format.format {
        HTS_FORMAT_BAM => sam_read1_bam(fp, h, b),
        HTS_FORMAT_EMPTY_FORMAT => {
            *libc::__errno_location() =
                libc::EPIPE as i32;
            -3
        }
        HTS_FORMAT_SAM => sam_c_4157_sam_read1_sam(fp, h, b),
        HTS_FORMAT_CRAM => sam_c_4145_sam_read1_cram(fp, h, b),
        HTS_FORMAT_FASTA_FORMAT | HTS_FORMAT_FASTQ_FORMAT => {
            if (*fp).state.is_null() {
                (*fp).state = sam_c_3786_fastq_state_init(if (*fp).format.format
                    == HTS_FORMAT_FASTQ_FORMAT
                {
                    b'@'
                } else {
                    b'>'
                } as i32)
                .cast();
                if (*fp).state.is_null() {
                    return -2;
                }
            }
            sam_c_3927_fastq_parse1(fp, b)
        }
        _ => {
            *libc::__errno_location() =
                libc::ENOEXEC as i32;
            -3
        }
    }
}

unsafe extern "C" fn sam_readrec(
    _ignored: *mut crate::htslib_rs::hts::BGZF,
    fpv: *mut std::ffi::c_void,
    bv: *mut std::ffi::c_void,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> i32 {
    let fp = fpv.cast::<htsFile>();
    let b = bv.cast::<bam1_t>();
    (*fp).line.data.truncate(0);
    let ret = sam_read1(fp, (*fp).bam_header.cast(), b);
    if ret >= 0 {
        *tid = (*b).core.tid;
        *beg = (*b).core.pos;
        *end = bam_endpos(b);
    }
    ret
}

unsafe extern "C" fn sam_readrec_rest(
    _ignored: *mut crate::htslib_rs::hts::BGZF,
    fpv: *mut std::ffi::c_void,
    bv: *mut std::ffi::c_void,
    _tid: *mut i32,
    _beg: *mut hts_pos_t,
    _end: *mut hts_pos_t,
) -> i32 {
    let fp = fpv.cast::<htsFile>();
    let b = bv.cast::<bam1_t>();
    (*fp).line.data.truncate(0);
    sam_read1(fp, (*fp).bam_header.cast(), b)
}

pub unsafe fn bam_plp_init(_func: bam_plp_auto_f, _data: *mut ()) -> bam_plp_t {
    let mut mp = mp_init();
    let head = mp_alloc(&mut mp);
    Box::into_raw(Box::new(bam_plp_s {
        mp: Some(mp),
        head,
        tail: head,
        tid: 0,
        max_tid: -1,
        pos: 0,
        max_pos: -1,
        is_eof: 0,
        max_plp: 0,
        error: 0,
        maxcnt: 8000,
        id: 0,
        plp: Vec::new(),
        b: if _func.is_some() {
            NonNull::new(bam_init1())
        } else {
            None
        },
        func: _func,
        data: if _func.is_some() {
            _data
        } else {
            std::ptr::null_mut()
        },
        overlaps: None,
        plp_construct: None,
        plp_destruct: None,
    }))
}

pub unsafe fn bam_plp_init_overlaps(_iter: bam_plp_t) -> i32 {
    (*_iter).overlaps = NonNull::new(Box::into_raw(olap_hash_new()).cast::<olap_hash_t>());
    if (*_iter).overlaps.is_none() {
        -1
    } else {
        0
    }
}

pub unsafe fn bam_plp_destroy(_iter: bam_plp_t) {
    if _iter.is_null() {
        return;
    }
    if let Some(overlaps) = (*_iter).overlaps {
        olap_hash_free(overlaps);
    }
    (*_iter).overlaps = None;
    let mut p = (*_iter).head;
    while let Some(node) = p {
        let node_ptr = node.as_ptr();
        if (*_iter).plp_destruct.is_some() && Some(node) != (*_iter).tail {
            (*_iter).plp_destruct.unwrap()((*_iter).data, &(*node_ptr).b, &mut (*node_ptr).cd);
        }
        let pnext = (*node_ptr).next;
        let mp = (*_iter)
            .mp
            .as_mut()
            .expect("pileup iterator owns a mempool");
        mp_free(mp, NonNull::new_unchecked(node_ptr));
        p = pnext;
    }
    if let Some(mp) = (*_iter).mp.take() {
        mp_destroy(mp);
    }
    if let Some(b) = (*_iter).b {
        bam_destroy1(b.as_ptr());
    }
    drop(Box::from_raw(_iter));
}

pub unsafe fn bam_plp_constructor(_plp: bam_plp_t, _func: bam_plp_constructor_f) {
    (*_plp).plp_construct = _func;
}

pub unsafe fn bam_plp_destructor(_plp: bam_plp_t, _func: bam_plp_constructor_f) {
    (*_plp).plp_destruct = _func;
}

pub unsafe fn bam_plp_auto(
    _iter: bam_plp_t,
    _tid: *mut i32,
    _pos: *mut i32,
    _n_plp: *mut i32,
) -> *const bam_pileup1_t {
    let mut pos64 = 0;
    let p = bam_plp64_auto(_iter, _tid, &mut pos64, _n_plp);
    if pos64 < i32::MAX as hts_pos_t {
        *_pos = pos64 as i32;
    } else {
        *_pos = i32::MAX;
        (*_iter).error = 1;
        *_n_plp = -1;
        return std::ptr::null();
    }
    p
}

pub unsafe fn bam_plp64_auto(
    _iter: bam_plp_t,
    _tid: *mut i32,
    _pos: *mut hts_pos_t,
    _n_plp: *mut i32,
) -> *const bam_pileup1_t {
    if (*_iter).func.is_none() || (*_iter).error != 0 {
        *_n_plp = -1;
        return std::ptr::null();
    }
    let mut plp = bam_plp64_next(_iter, _tid, _pos, _n_plp);
    if !plp.is_null() {
        return plp;
    }
    *_n_plp = 0;
    if (*_iter).is_eof != 0 {
        return std::ptr::null();
    }
    loop {
        let b = (*_iter)
            .b
            .expect("automatic pileup iterator owns a callback record")
            .as_ptr();
        let ret = (*_iter).func.unwrap()((*_iter).data, b);
        if ret < 0 {
            if ret < -1 {
                (*_iter).error = ret;
                *_n_plp = -1;
                return std::ptr::null();
            }
            if bam_plp_push(_iter, std::ptr::null()) < 0 {
                *_n_plp = -1;
                return std::ptr::null();
            }
            plp = bam_plp64_next(_iter, _tid, _pos, _n_plp);
            if !plp.is_null() {
                return plp;
            }
            return std::ptr::null();
        }
        if bam_plp_push(_iter, b) < 0 {
            *_n_plp = -1;
            return std::ptr::null();
        }
        plp = bam_plp64_next(_iter, _tid, _pos, _n_plp);
        if !plp.is_null() {
            return plp;
        }
    }
}

pub unsafe fn bam_plp_set_maxcnt(_iter: bam_plp_t, _maxcnt: i32) {
    (*_iter).maxcnt = _maxcnt;
}

pub unsafe fn bam_plp64_next(
    _iter: bam_plp_t,
    _tid: *mut i32,
    _pos: *mut hts_pos_t,
    _n_plp: *mut i32,
) -> *const bam_pileup1_t {
    let iter = &mut *_iter;
    if iter.error != 0 {
        *_n_plp = -1;
        return std::ptr::null();
    }
    *_n_plp = 0;
    if iter.is_eof != 0 && iter.head == iter.tail {
        return std::ptr::null();
    }
    while iter.is_eof != 0
        || iter.max_tid > iter.tid
        || (iter.max_tid == iter.tid && iter.max_pos > iter.pos)
    {
        let mut n_plp = 0;
        let mut pptr: *mut Option<NonNull<lbnode_t>> = &mut iter.head;
        while *pptr != iter.tail {
            let p = (*pptr)
                .expect("pileup list links are non-null before the tail")
                .as_ptr();
            if (*p).b.core.tid < iter.tid || ((*p).b.core.tid == iter.tid && (*p).end <= iter.pos) {
                overlap_remove(iter, Some(&(*p).b));
                if iter.plp_destruct.is_some() {
                    iter.plp_destruct.unwrap()(iter.data, &(*p).b, &mut (*p).cd);
                }
                *pptr = (*p).next;
                let mp = iter.mp.as_mut().expect("pileup iterator owns a mempool");
                mp_free(mp, NonNull::new_unchecked(p));
            } else {
                if (*p).b.core.tid == iter.tid && (*p).beg <= iter.pos {
                    if n_plp == iter.plp.len() as i32 {
                        iter.max_plp = if iter.max_plp != 0 {
                            iter.max_plp << 1
                        } else {
                            256
                        };
                        iter.plp
                            .resize_with(iter.max_plp as usize, bam_pileup1_t::default);
                    }
                    let out = iter.plp.as_mut_ptr().add(n_plp as usize);
                    (*out).b = &mut (*p).b;
                    (*out).cd = (*p).cd;
                    if resolve_cigar2(&mut *out, iter.pos, &mut (*p).s) != 0 {
                        n_plp += 1;
                    }
                }
                pptr = &mut (*p).next;
            }
        }
        *_n_plp = n_plp;
        *_tid = iter.tid;
        *_pos = iter.pos;
        if iter.head != iter.tail && iter.tid > (*iter.head.unwrap().as_ptr()).b.core.tid {
            iter.error = 1;
            *_n_plp = -1;
            return std::ptr::null();
        }
        let head = iter.head.unwrap().as_ptr();
        if iter.tid < (*head).b.core.tid {
            iter.tid = (*head).b.core.tid;
            iter.pos = (*head).beg;
        } else if iter.pos < (*head).beg {
            iter.pos = (*head).beg;
        } else {
            iter.pos += 1;
        }
        if n_plp != 0 {
            return iter.plp.as_ptr();
        }
        if iter.is_eof != 0 && iter.head == iter.tail {
            break;
        }
    }
    std::ptr::null()
}

pub unsafe fn bam_plp_next(
    _iter: bam_plp_t,
    _tid: *mut i32,
    _pos: *mut i32,
    _n_plp: *mut i32,
) -> *const bam_pileup1_t {
    let mut pos64 = 0;
    let p = bam_plp64_next(_iter, _tid, &mut pos64, _n_plp);
    if pos64 < i32::MAX as hts_pos_t {
        *_pos = pos64 as i32;
    } else {
        *_pos = i32::MAX;
        (*_iter).error = 1;
        *_n_plp = -1;
        return std::ptr::null();
    }
    p
}

pub unsafe fn bam_plp_push(_iter: bam_plp_t, b: *const bam1_t) -> i32 {
    let iter = &mut *_iter;
    if iter.error != 0 {
        return -1;
    }
    if !b.is_null() {
        if (*b).core.tid < 0 {
            overlap_remove(iter, Some(&*b));
            return 0;
        }
        if ((*b).core.flag as i32 & BAM_FUNMAP) != 0 {
            overlap_remove(iter, Some(&*b));
            return 0;
        }
        if iter.tid == (*b).core.tid
            && iter.pos == (*b).core.pos
            && iter
                .mp
                .as_ref()
                .expect("pileup iterator owns a mempool")
                .cnt
                > iter.maxcnt
        {
            overlap_remove(iter, Some(&*b));
            return 0;
        }
        let tail = iter
            .tail
            .expect("pileup tail sentinel is initialized")
            .as_ptr();
        if bam_copy1(&mut (*tail).b, b).is_null() {
            return -1;
        }
        (*tail).b.id = iter.id;
        iter.id += 1;
        (*tail).beg = (*b).core.pos;
        (*tail).end = (*b).core.pos + bam_cigar2rlen((*b).core.n_cigar as i32, bam_get_cigar(b));
        (*tail).s = G_CSTATE_NULL;
        (*tail).s.end = (*tail).end - 1;
        if (*b).core.tid < iter.max_tid {
            iter.error = 1;
            return -1;
        }
        if (*b).core.tid == iter.max_tid && (*tail).beg < iter.max_pos {
            iter.error = 1;
            return -1;
        }
        iter.max_tid = (*b).core.tid;
        iter.max_pos = (*tail).beg;
        if (*tail).end > iter.pos || (*tail).b.core.tid > iter.tid {
            let Some(next) = mp_alloc(iter.mp.as_mut().expect("pileup iterator owns a mempool"))
            else {
                iter.error = 1;
                return -1;
            };
            if iter.plp_construct.is_some()
                && iter.plp_construct.unwrap()(iter.data, &(*tail).b, &mut (*tail).cd) < 0
            {
                let mp = iter.mp.as_mut().expect("pileup iterator owns a mempool");
                mp_free(mp, next);
                iter.error = 1;
                return -1;
            }
            if overlap_push(iter, NonNull::new_unchecked(tail)) < 0 {
                let mp = iter.mp.as_mut().expect("pileup iterator owns a mempool");
                mp_free(mp, next);
                iter.error = 1;
                return -1;
            }
            (*tail).next = Some(next);
            iter.tail = Some(next);
        }
    } else {
        iter.is_eof = 1;
    }
    0
}

pub unsafe fn bam_plp_reset(_iter: bam_plp_t) {
    let iter = &mut *_iter;
    overlap_remove(iter, None);
    iter.max_tid = -1;
    iter.max_pos = -1;
    iter.tid = 0;
    iter.pos = 0;
    iter.is_eof = 0;
    while iter.head != iter.tail {
        let p = iter
            .head
            .expect("pileup head is non-null until it reaches tail")
            .as_ptr();
        iter.head = (*p).next;
        let mp = iter.mp.as_mut().expect("pileup iterator owns a mempool");
        mp_free(mp, NonNull::new_unchecked(p));
    }
}

pub unsafe fn bam_mplp_init(
    _n: i32,
    _func: bam_plp_auto_f,
    _data: *mut *mut (),
) -> bam_mplp_t {
    let n = _n.max(0) as usize;
    let mut iter = Box::new(bam_mplp_s {
        n: _n,
        min_tid: u32::MAX as i32,
        tid: vec![u32::MAX as i32; n],
        min_pos: HTS_POS_MAX,
        pos: vec![HTS_POS_MAX; n],
        iter: Vec::with_capacity(n),
        n_plp: vec![0; n],
        plp: vec![std::ptr::null(); n],
    });
    for i in 0..n {
        let data = if _data.is_null() {
            std::ptr::null_mut()
        } else {
            *_data.add(i)
        };
        let plp = bam_plp_init(_func, data);
        if let Some(plp) = NonNull::new(plp) {
            iter.iter.push(plp);
        }
    }
    Box::into_raw(iter)
}

pub unsafe fn bam_mplp_destroy(_iter: bam_mplp_t) {
    if _iter.is_null() {
        return;
    }
    for iter in (*_iter).iter.drain(..) {
        bam_plp_destroy(iter.as_ptr());
    }
    drop(Box::from_raw(_iter));
}

pub unsafe fn bam_mplp_init_overlaps(_iter: bam_mplp_t) -> i32 {
    let mut r = 0;
    for iter in &(*_iter).iter {
        r |= bam_plp_init_overlaps(iter.as_ptr());
    }
    if r == 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn bam_mplp_auto(
    _iter: bam_mplp_t,
    _tid: *mut i32,
    _pos: *mut i32,
    _n_plp: *mut i32,
    _plp: *mut *const bam_pileup1_t,
) -> i32 {
    let mut pos64 = 0;
    let ret = bam_mplp64_auto(_iter, _tid, &mut pos64, _n_plp, _plp);
    if ret >= 0 {
        if pos64 < i32::MAX as hts_pos_t {
            *_pos = pos64 as i32;
        } else {
            *_pos = i32::MAX;
            return -1;
        }
    }
    ret
}

pub unsafe fn bam_mplp64_auto(
    _iter: bam_mplp_t,
    _tid: *mut i32,
    _pos: *mut hts_pos_t,
    _n_plp: *mut i32,
    _plp: *mut *const bam_pileup1_t,
) -> i32 {
    let iter = &mut *_iter;
    let mut ret = 0;
    let mut new_min_pos = HTS_POS_MAX;
    let mut new_min_tid = u32::MAX;
    for idx in 0..iter.iter.len() {
        if iter.pos[idx] == iter.min_pos && iter.tid[idx] == iter.min_tid {
            let mut tid = 0;
            let mut pos = 0;
            iter.plp[idx] = bam_plp64_auto(
                iter.iter[idx].as_ptr(),
                &mut tid,
                &mut pos,
                &mut iter.n_plp[idx],
            );
            if (*iter.iter[idx].as_ptr()).error != 0 {
                return -1;
            }
            if !iter.plp[idx].is_null() {
                iter.tid[idx] = tid;
                iter.pos[idx] = pos;
            } else {
                iter.tid[idx] = 0;
                iter.pos[idx] = 0;
            }
        }
        if !iter.plp[idx].is_null() {
            let tid_u = iter.tid[idx] as u32;
            if tid_u < new_min_tid {
                new_min_tid = tid_u;
                new_min_pos = iter.pos[idx];
            } else if tid_u == new_min_tid && iter.pos[idx] < new_min_pos {
                new_min_pos = iter.pos[idx];
            }
        }
    }
    iter.min_pos = new_min_pos;
    iter.min_tid = new_min_tid as i32;
    if new_min_pos == HTS_POS_MAX {
        return 0;
    }
    *_tid = new_min_tid as i32;
    *_pos = new_min_pos;
    for idx in 0..iter.iter.len() {
        let pos = iter.pos[idx];
        let tid = iter.tid[idx];
        if pos == iter.min_pos && tid == iter.min_tid {
            *_n_plp.add(idx) = iter.n_plp[idx];
            *_plp.add(idx) = iter.plp[idx];
            ret += 1;
        } else {
            *_n_plp.add(idx) = 0;
            *_plp.add(idx) = std::ptr::null();
        }
    }
    ret
}

pub unsafe fn bam_mplp_set_maxcnt(_iter: bam_mplp_t, _maxcnt: i32) {
    for iter in &(*_iter).iter {
        (*iter.as_ptr()).maxcnt = _maxcnt;
    }
}

pub unsafe fn bam_mplp_reset(iter: bam_mplp_t) {
    let iter = &mut *iter;
    iter.min_pos = HTS_POS_MAX;
    iter.min_tid = u32::MAX as i32;
    for idx in 0..iter.iter.len() {
        bam_plp_reset(iter.iter[idx].as_ptr());
        iter.pos[idx] = HTS_POS_MAX;
        iter.tid[idx] = u32::MAX as i32;
        iter.n_plp[idx] = 0;
        iter.plp[idx] = std::ptr::null();
    }
}

pub unsafe fn bam_plp_insertion_mod(
    p: *const bam_pileup1_t,
    m: *mut hts_base_mod_state,
    ins: *mut kstring_t,
    del_len: *mut i32,
) -> i32 {
    if (*p).indel <= 0 {
        (*ins).data.clear();
        return 0;
    }

    if !del_len.is_null() {
        *del_len = 0;
    }

    let cigar = bam_get_cigar((*p).b);
    let mut indel = 0usize;
    let mut k = (*p).cigar_ind + 1;
    while k < (*(*p).b).core.n_cigar as i32 {
        let c = *cigar.add(k as usize);
        match (c & BAM_CIGAR_MASK) as i32 {
            BAM_CPAD | BAM_CINS => indel += (c >> BAM_CIGAR_SHIFT) as usize,
            _ => break,
        }
        k += 1;
    }
    let nb = indel as i32;

    // `indel` is the write cursor into the owned buffer; grow the Vec so that
    // index `indel` is always writable (no trailing NUL is stored).
    (*ins).data.clear();
    (*ins).data.resize(indel, 0);

    indel = 0;
    k = (*p).cigar_ind + 1;
    let mut j = 1;
    macro_rules! put_at {
        ($idx:expr, $val:expr) => {{
            let idx = $idx;
            let ins_data = &mut (*ins).data;
            if idx >= ins_data.len() {
                ins_data.resize(idx + 1, 0);
            }
            ins_data[idx] = $val;
        }};
    }
    while k < (*(*p).b).core.n_cigar as i32 {
        let c = *cigar.add(k as usize);
        match (c & BAM_CIGAR_MASK) as i32 {
            BAM_CPAD => {
                for _ in 0..(c >> BAM_CIGAR_SHIFT) {
                    put_at!(indel, b'*');
                    indel += 1;
                }
            }
            BAM_CINS => {
                for _ in 0..(c >> BAM_CIGAR_SHIFT) {
                    let qpos = (*p).qpos + j - bam_pileup1_is_del(p) as i32;
                    let base = if qpos < (*(*p).b).core.l_qseq {
                        SEQ_NT16_STR[bam_seqi(bam_get_seq((*p).b), qpos as usize) as usize]
                    } else {
                        b'N'
                    };
                    put_at!(indel, base);
                    indel += 1;

                    if !m.is_null() {
                        let mut mods = [hts_base_mod {
                            modified_base: 0,
                            canonical_base: 0,
                            strand: 0,
                            qual: 0,
                        }; 256];
                        let nm = bam_mods_at_qpos(&*(*p).b, qpos, &mut *m, &mut mods);
                        if nm > 0 {
                            put_at!(indel, b'[');
                            indel += 1;
                            for item in mods.iter().take(nm as usize) {
                                let sign = if item.strand != 0 { '-' } else { '+' };
                                let qual = if item.qual >= 0 {
                                    item.qual.to_string()
                                } else {
                                    String::new()
                                };
                                let text = if item.modified_base < 0 {
                                    format!("{}({}){}", sign, -item.modified_base, qual)
                                } else {
                                    format!("{}{}{}", sign, item.modified_base as u8 as char, qual)
                                };
                                for &byte in text.as_bytes() {
                                    put_at!(indel, byte);
                                    indel += 1;
                                }
                            }
                            put_at!(indel, b']');
                            indel += 1;
                        }
                    }
                    j += 1;
                }
            }
            BAM_CDEL => {
                if !del_len.is_null() {
                    *del_len = (c >> BAM_CIGAR_SHIFT) as i32;
                }
                break;
            }
            _ => break,
        }
        k += 1;
    }
    (*ins).data.truncate(indel);
    nb
}

pub unsafe fn bam_plp_insertion(
    p: *const bam_pileup1_t,
    ins: *mut kstring_t,
    del_len: *mut i32,
) -> i32 {
    bam_plp_insertion_mod(p, std::ptr::null_mut(), ins, del_len)
}

pub unsafe fn bam_mplp_constructor(_iter: bam_mplp_t, _func: bam_plp_constructor_f) {
    for iter in &(*_iter).iter {
        bam_plp_constructor(iter.as_ptr(), _func);
    }
}

pub unsafe fn bam_mplp_destructor(_iter: bam_mplp_t, _func: bam_plp_constructor_f) {
    for iter in &(*_iter).iter {
        bam_plp_destructor(iter.as_ptr(), _func);
    }
}

pub unsafe fn bam_pileup1_is_del(p: *const bam_pileup1_t) -> u32 {
    (*p).bitfields & 1
}

pub unsafe fn bam_pileup1_is_head(p: *const bam_pileup1_t) -> u32 {
    ((*p).bitfields >> 1) & 1
}

pub unsafe fn bam_pileup1_is_tail(p: *const bam_pileup1_t) -> u32 {
    ((*p).bitfields >> 2) & 1
}

pub unsafe fn bam_pileup1_is_refskip(p: *const bam_pileup1_t) -> u32 {
    ((*p).bitfields >> 3) & 1
}

pub unsafe fn bam_pileup1_aux(p: *const bam_pileup1_t) -> u32 {
    (*p).bitfields >> 5
}

pub unsafe fn bam_cigar_op(c: u32) -> i32 {
    (c & 0x0f) as i32
}

pub unsafe fn bam_cigar_oplen(c: u32) -> u32 {
    c >> 4
}

pub unsafe fn bam_cigar_type(o: i32) -> i32 {
    BAM_CIGAR_TYPE[o as usize]
}

pub unsafe fn bam_cigar2qlen(n_cigar: i32, cigar: *const u32) -> hts_pos_t {
    let mut l = 0;
    for k in 0..n_cigar {
        let c = *cigar.add(k as usize);
        if (bam_cigar_type(bam_cigar_op(c)) & 1) != 0 {
            l += bam_cigar_oplen(c) as hts_pos_t;
        }
    }
    l
}

pub unsafe fn bam_cigar2rlen(n_cigar: i32, cigar: *const u32) -> hts_pos_t {
    let mut l = 0;
    for k in 0..n_cigar {
        let c = *cigar.add(k as usize);
        if (bam_cigar_type(bam_cigar_op(c)) & 2) != 0 {
            l += bam_cigar_oplen(c) as hts_pos_t;
        }
    }
    l
}

unsafe fn bam_cigar2rqlens(
    n_cigar: i32,
    cigar: *const u32,
    rlen: *mut hts_pos_t,
    qlen: *mut hts_pos_t,
) {
    let mut r = 0;
    let mut q = 0;
    for k in 0..n_cigar {
        let c = *cigar.add(k as usize);
        let type_ = bam_cigar_type(bam_cigar_op(c));
        let len = bam_cigar_oplen(c) as hts_pos_t;
        if (type_ & 2) != 0 {
            r += len;
        }
        if (type_ & 1) != 0 {
            q += len;
        }
    }
    *rlen = r;
    *qlen = q;
}

pub unsafe fn read_ncigar(mut q: *const u8) -> u32 {
    let mut n_cigar = 0u32;
    while *q != 0 && *q != b'\t' as u8 {
        if libc::isdigit(*q as u8 as i32) == 0 {
            n_cigar = n_cigar.wrapping_add(1);
        }
        q = q.add(1);
    }
    if n_cigar == 0 {
        return 0;
    }
    if n_cigar >= 2_147_483_647 {
        return 0;
    }
    n_cigar
}

pub unsafe fn parse_cigar(in_: *const u8, a_cigar: *mut u32, n_cigar: u32) -> i32 {
    let mut p = in_;
    for i in 0..n_cigar {
        let mut overflow = 0;
        let mut q: *mut u8 = std::ptr::null_mut();
        let len = hts_str2uint(p.cast(), (&mut q as *mut *mut u8).cast(), 28, &mut overflow) as u32;
        if q == p.cast_mut() || overflow != 0 {
            return 0;
        }
        p = q;
        let op = match *p as u8 {
            b'M' => BAM_CMATCH,
            b'I' => BAM_CINS,
            b'D' => BAM_CDEL,
            b'N' => BAM_CREF_SKIP,
            b'S' => BAM_CSOFT_CLIP,
            b'H' => BAM_CHARD_CLIP,
            b'P' => BAM_CPAD,
            b'=' => BAM_CEQUAL,
            b'X' => BAM_CDIFF,
            b'B' => BAM_CBACK,
            _ => return 0,
        };
        p = p.add(1);
        *a_cigar.add(i as usize) = (len << BAM_CIGAR_SHIFT) | op as u32;
    }

    p.offset_from(in_) as i32
}

pub unsafe fn sam_parse_cigar(
    in_: *const u8,
    end: *mut *mut u8,
    a_cigar: *mut *mut u32,
    a_mem: *mut usize,
) -> isize {
    if in_.is_null() || a_cigar.is_null() || a_mem.is_null() {
        return -1;
    }
    if !end.is_null() {
        *end = in_.cast_mut();
    }

    if *in_ == b'*' as u8 {
        if !end.is_null() {
            *end = in_.add(1).cast_mut();
        }
        return 0;
    }

    let n_cigar = read_ncigar(in_) as usize;
    if n_cigar == 0 {
        return 0;
    }
    if n_cigar > *a_mem {
        let old_mem = *a_mem;
        let mut a_vec = if (*a_cigar).is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts(*a_cigar, old_mem, old_mem)
        };
        a_vec.resize(n_cigar, 0u32);
        *a_cigar = a_vec.as_mut_ptr();
        std::mem::forget(a_vec);
        *a_mem = n_cigar;
    }

    let diff = parse_cigar(in_, *a_cigar, n_cigar as u32);
    if diff == 0 {
        return -1;
    }
    if !end.is_null() {
        *end = in_.add(diff as usize).cast_mut();
    }
    n_cigar as isize
}

pub unsafe fn bam_parse_cigar(in_: *const u8, end: *mut *mut u8, b: *mut bam1_t) -> isize {
    if in_.is_null() || b.is_null() {
        return -1;
    }
    if !end.is_null() {
        *end = in_.cast_mut();
    }

    let n_cigar = if *in_ == b'*' as u8 {
        0usize
    } else {
        read_ncigar(in_) as usize
    };
    if n_cigar == 0 && (*b).core.n_cigar == 0 {
        if !end.is_null() {
            *end = in_.add(1).cast_mut();
        }
        return 0;
    }

    let cig_diff = n_cigar as isize - (*b).core.n_cigar as isize;
    if cig_diff > 0
        && possibly_expand_bam_data(b, cig_diff as usize * std::mem::size_of::<u32>()) < 0
    {
        return -1;
    }

    let old_len = (*b).data.len();
    let new_len =
        (old_len as isize + cig_diff * std::mem::size_of::<u32>() as isize) as usize;
    // Grow data.len() to cover the larger of old/new before shifting bytes so the
    // raw-pointer memmove stays within the live buffer; shrink (truncate) after.
    if new_len > old_len && realloc_bam_data(b, new_len) < 0 {
        return -1;
    }

    let cig = bam_get_cigar(b).cast_mut();
    if cig.cast::<u8>().cast_const() != (*b).data.as_ptr().add(old_len) {
        let seq = bam_get_seq(b);
        libc::memmove(
            cig.add(n_cigar).cast(),
            seq.cast(),
            (*b).data.as_ptr().add(old_len).offset_from(seq) as usize,
        );
    }

    let diff = if n_cigar != 0 {
        let diff = parse_cigar(in_, cig, n_cigar as u32);
        if diff == 0 {
            return -1;
        }
        diff
    } else {
        1
    };

    if new_len < old_len {
        (*b).data.truncate(new_len);
    }
    (*b).core.n_cigar = n_cigar as u32;
    if !end.is_null() {
        *end = in_.add(diff as usize).cast_mut();
    }
    n_cigar as isize
}

pub unsafe fn subtract_check_underflow(length: usize, limit: *mut usize) -> i32 {
    if length <= *limit {
        *limit -= length;
        0
    } else {
        -1
    }
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn bam_set1(
    bam: *mut bam1_t,
    mut l_qname: usize,
    mut qname: *const u8,
    flag: u16,
    tid: i32,
    pos: hts_pos_t,
    mapq: u8,
    n_cigar: usize,
    cigar: *const u32,
    mtid: i32,
    mpos: hts_pos_t,
    isize: hts_pos_t,
    l_seq: usize,
    seq: *const u8,
    qual: *const u8,
    l_aux: usize,
) -> i32 {
    if l_qname == 0 {
        l_qname = 1;
        qname = c"*".as_ptr().cast();
    }

    let qname_nuls = 4 - l_qname % 4;
    let mut rlen = 0;
    let mut qlen = 0;
    if (flag as i32 & BAM_FUNMAP) == 0 {
        bam_cigar2rqlens(
            n_cigar as i32,
            cigar,
            &mut rlen as *mut hts_pos_t,
            &mut qlen as *mut hts_pos_t,
        );
    }
    if rlen == 0 {
        rlen = 1;
    }

    if l_qname > 254 {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }
    if HTS_POS_MAX - rlen <= pos {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }
    if (flag as i32 & BAM_FUNMAP) == 0 && l_seq > 0 && n_cigar == 0 {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }
    if (flag as i32 & BAM_FUNMAP) == 0 && l_seq > 0 && l_seq as hts_pos_t != qlen {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }

    let mut limit = i32::MAX as usize;
    let mut u = subtract_check_underflow(l_qname + qname_nuls, &mut limit);
    u += subtract_check_underflow(n_cigar * 4, &mut limit);
    u += subtract_check_underflow(l_seq.div_ceil(2), &mut limit);
    u += subtract_check_underflow(l_seq, &mut limit);
    u += subtract_check_underflow(l_aux, &mut limit);
    if u != 0 {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }

    let data_len = l_qname + qname_nuls + n_cigar * 4 + l_seq.div_ceil(2) + l_seq;
    if realloc_bam_data(bam, data_len + l_aux) < 0 {
        return -1;
    }

    // realloc_bam_data grew len to data_len + l_aux; the qname/cigar/seq portion
    // is data_len, the aux bytes are appended later by the caller, so set the
    // logical length to data_len (capacity for l_aux stays reserved).
    (*bam).data.truncate(data_len);
    (*bam).core.pos = pos;
    (*bam).core.tid = tid;
    (*bam).core.bin = hts_reg2bin(pos, pos + rlen, 14, 5) as u16;
    (*bam).core.qual = mapq;
    (*bam).core.l_extranul = (qname_nuls - 1) as u8;
    (*bam).core.flag = flag;
    (*bam).core.l_qname = (l_qname + qname_nuls) as u16;
    (*bam).core.n_cigar = n_cigar as u32;
    (*bam).core.l_qseq = l_seq as i32;
    (*bam).core.mtid = mtid;
    (*bam).core.mpos = mpos;
    (*bam).core.isize = isize;

    let mut cp = (*bam).data.as_mut_ptr();
    std::ptr::copy_nonoverlapping(qname.cast::<u8>(), cp.cast::<u8>(), (l_qname) as usize);
    for i in 0..qname_nuls {
        *cp.add(l_qname + i) = 0;
    }
    cp = cp.add(l_qname + qname_nuls);

    if n_cigar > 0 {
        std::ptr::copy_nonoverlapping(cigar.cast::<u8>(), cp.cast::<u8>(), ((n_cigar * 4)) as usize);
    }
    cp = cp.add(n_cigar * 4);

    let useq = seq.cast::<u8>();
    let mut i = 0usize;
    const NN: usize = 16;
    while i + NN < l_seq {
        let u2 = useq.add(i);
        for j in 0..(NN / 2) {
            *cp.add(j) = (SEQ_NT16_TABLE[*u2.add(j * 2) as usize] << 4)
                | SEQ_NT16_TABLE[*u2.add(j * 2 + 1) as usize];
        }
        cp = cp.add(NN / 2);
        i += NN;
    }
    while i + 1 < l_seq {
        *cp = (SEQ_NT16_TABLE[*useq.add(i) as usize] << 4)
            | SEQ_NT16_TABLE[*useq.add(i + 1) as usize];
        cp = cp.add(1);
        i += 2;
    }
    while i < l_seq {
        *cp = SEQ_NT16_TABLE[*seq.add(i) as u8 as usize] << 4;
        cp = cp.add(1);
        i += 1;
    }

    if !qual.is_null() {
        std::ptr::copy_nonoverlapping(qual.cast::<u8>(), cp.cast::<u8>(), (l_seq) as usize);
    } else {
        libc::memset(cp.cast(), 0xff, l_seq);
    }

    data_len as i32
}

unsafe fn bam_set1_fastq_unmapped(
    bam: *mut bam1_t,
    mut l_qname: usize,
    mut qname: *const u8,
    flag: u16,
    l_seq: usize,
    seq: *const u8,
    qual: *const u8,
) -> i32 {
    if l_qname == 0 {
        l_qname = 1;
        qname = c"*".as_ptr().cast();
    }

    let qname_nuls = 4 - l_qname % 4;
    if l_qname > 254 || l_qname + qname_nuls > i32::MAX as usize {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }

    let seq_bytes = l_seq.div_ceil(2);
    let Some(data_len) = l_qname
        .checked_add(qname_nuls)
        .and_then(|v| v.checked_add(seq_bytes))
        .and_then(|v| v.checked_add(l_seq))
    else {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    };
    if data_len > i32::MAX as usize {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }
    if realloc_bam_data(bam, data_len) < 0 {
        return -1;
    }

    // C bam_set1 assigns `bam->l_data = data_len` outright. Our owned Vec
    // collapses l_data into data.len(), but realloc_bam_data only ever grows the
    // length (never shrinks), so a reused record whose previous contents were
    // longer (e.g. carried aux tags) would retain that stale tail past data_len.
    // Truncate to the exact logical length so subsequent aux appends start at the
    // correct offset and don't leave duplicate/stale tags behind.
    (*bam).data.truncate(data_len);
    (*bam).core.pos = -1;
    (*bam).core.tid = -1;
    (*bam).core.bin = hts_reg2bin(-1, 0, 14, 5) as u16;
    (*bam).core.qual = 0;
    (*bam).core.l_extranul = (qname_nuls - 1) as u8;
    (*bam).core.flag = flag;
    (*bam).core.l_qname = (l_qname + qname_nuls) as u16;
    (*bam).core.n_cigar = 0;
    (*bam).core.l_qseq = l_seq as i32;
    (*bam).core.mtid = -1;
    (*bam).core.mpos = -1;
    (*bam).core.isize = 0;

    let mut cp = (*bam).data.as_mut_ptr();
    std::ptr::copy_nonoverlapping(qname.cast::<u8>(), cp.cast::<u8>(), (l_qname) as usize);
    for i in 0..qname_nuls {
        *cp.add(l_qname + i) = 0;
    }
    cp = cp.add(l_qname + qname_nuls);

    let useq = seq.cast::<u8>();
    let mut i = 0usize;
    while i + 1 < l_seq {
        *cp = (SEQ_NT16_TABLE[*useq.add(i) as usize] << 4)
            | SEQ_NT16_TABLE[*useq.add(i + 1) as usize];
        cp = cp.add(1);
        i += 2;
    }
    if i < l_seq {
        *cp = SEQ_NT16_TABLE[*useq.add(i) as usize] << 4;
        cp = cp.add(1);
    }

    if !qual.is_null() {
        std::ptr::copy_nonoverlapping(qual.cast::<u8>(), cp.cast::<u8>(), (l_seq) as usize);
    } else {
        libc::memset(cp.cast(), 0xff, l_seq);
    }

    data_len as i32
}

pub unsafe fn sam_prob_realn(
    b: *mut bam1_t,
    ref_: *const u8,
    ref_len: hts_pos_t,
    flag: i32,
) -> i32 {
    let mut k: i32;
    let mut bw: i32;
    let mut y: i32;
    let mut yb: i32;
    let mut ye: i32;
    let mut xb: hts_pos_t;
    let mut xe: hts_pos_t;
    let mut fix_bq: i32 = 0;
    let apply_baq = flag & 1;
    let extend_baq = flag & 2;
    let redo_baq = flag & 4;
    let system = flag & (0xff << 3);
    let mut i: hts_pos_t;
    let mut x: hts_pos_t;
    let cigar = bam_get_cigar(b);
    const BAQ_ILLUMINA: i32 = 1 << 3;
    const SEQ_NT16_INT: [u8; 16] = [4, 0, 1, 4, 2, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4];

    let mut conf = crate::htslib_rs::probaln::probaln_par_t {
        d: 0.001,
        e: 0.1,
        bw: 10,
    };

    if (*b).core.l_qseq > 1000 || system > BAQ_ILLUMINA {
        conf.d = 1e-7;
        conf.e = 1e-1;
    }

    let mut bq;
    let mut zq;
    let qual = bam_get_qual(b).cast_mut();
    if ((*b).core.flag as i32 & BAM_FUNMAP) != 0 || (*b).core.l_qseq == 0 || *qual == u8::MAX {
        return -1;
    }

    bq = bam_aux_get(b, c"BQ".as_ptr().cast());
    if !bq.is_null() {
        if redo_baq == 0
            && realn_check_tag(
                bq,
                crate::htslib_rs::hts::HTS_LOG_WARNING,
                c"BQ".as_ptr().cast(),
                b,
            ) < 0
        {
            fix_bq = 1;
        }
        bq = bq.add(1);
    }
    zq = bam_aux_get(b, c"ZQ".as_ptr().cast());
    if !zq.is_null() {
        if realn_check_tag(zq, crate::htslib_rs::hts::HTS_LOG_ERROR, c"ZQ".as_ptr().cast(), b) < 0 {
            return -4;
        }
        zq = zq.add(1);
    }
    if !bq.is_null() && redo_baq != 0 {
        bam_aux_del(b, bq.sub(1));
        bq = std::ptr::null_mut();
    }
    if !bq.is_null() && !zq.is_null() {
        bam_aux_del(b, zq.sub(1));
        zq = std::ptr::null_mut();
    }
    if zq.is_null() && fix_bq != 0 {
        bam_aux_del(b, bq.sub(1));
        bq = std::ptr::null_mut();
    }

    if !bq.is_null() || !zq.is_null() {
        if (apply_baq != 0 && !zq.is_null()) || (apply_baq == 0 && !bq.is_null()) {
            return -3;
        }
        if !bq.is_null() && apply_baq != 0 {
            i = 0;
            while i < (*b).core.l_qseq as hts_pos_t {
                let q = qual.add(i as usize);
                let v = bq.add(i as usize);
                *q = if (*q as i32) + 64 < *v as i32 {
                    0
                } else {
                    ((*q as i32) - ((*v as i32) - 64)) as u8
                };
                i += 1;
            }
            *bq.sub(3) = b'Z';
        } else if !zq.is_null() && apply_baq == 0 {
            i = 0;
            while i < (*b).core.l_qseq as hts_pos_t {
                let q = qual.add(i as usize);
                *q = ((*q as i32) + (*zq.add(i as usize) as i32) - 64) as u8;
                i += 1;
            }
            *zq.sub(3) = b'B';
        }
        return 0;
    }

    x = (*b).core.pos;
    y = 0;
    yb = -1;
    ye = -1;
    xb = -1;
    xe = -1;
    k = 0;
    while k < (*b).core.n_cigar as i32 {
        let op = (*cigar.add(k as usize) & 0xf) as i32;
        let l = (*cigar.add(k as usize) >> 4) as i32;
        if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
            if yb < 0 {
                yb = y;
            }
            if xb < 0 {
                xb = x;
            }
            ye = y + l;
            xe = x + l as hts_pos_t;
            x += l as hts_pos_t;
            y += l;
        } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
            y += l;
        } else if op == BAM_CDEL {
            x += l as hts_pos_t;
        } else if op == BAM_CREF_SKIP {
            return -1;
        }
        k += 1;
    }
    if xb == -1 {
        return -1;
    }

    bw = 7;
    if ((xe - xb) - (ye - yb) as hts_pos_t).abs() > bw as hts_pos_t {
        bw = ((xe - xb) - (ye - yb) as hts_pos_t).abs() as i32 + 3;
    }
    conf.bw = bw;

    xb -= yb as hts_pos_t + (bw / 2) as hts_pos_t;
    if xb < 0 {
        xb = 0;
    }
    xe += ((*b).core.l_qseq - ye) as hts_pos_t + (bw / 2) as hts_pos_t;
    if xe - xb - (*b).core.l_qseq as hts_pos_t > bw as hts_pos_t {
        xb += (xe - xb - (*b).core.l_qseq as hts_pos_t - bw as hts_pos_t) / 2;
        xe -= (xe - xb - (*b).core.l_qseq as hts_pos_t - bw as hts_pos_t) / 2;
    }

    let seq = bam_get_seq(b);
    let mut lref = if xe > xb { (xe - xb) as usize } else { 1 };
    if extend_baq != 0 && lref < (*b).core.l_qseq as usize {
        lref = (*b).core.l_qseq as usize;
    }
    let align_lqseq = (((*b).core.l_qseq as usize + 1) | 0xf) + 1;
    if (usize::MAX - lref) / (3 + std::mem::size_of::<i32>()) < align_lqseq {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -4;
    }
    let Some(total) = align_lqseq.checked_mul(3).and_then(|n| n.checked_add(lref)) else {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -4;
    };
    let mut bq_buf = vec![0_u8; total];
    bq = bq_buf.as_mut_ptr();
    let q = bq.add(align_lqseq);
    let tseq = q.add(align_lqseq);
    let tref = tseq.add(align_lqseq);

    std::ptr::copy_nonoverlapping(qual, bq, (*b).core.l_qseq as usize);
    *bq.add((*b).core.l_qseq as usize) = 0;
    i = 0;
    while i < (*b).core.l_qseq as hts_pos_t {
        *tseq.add(i as usize) = SEQ_NT16_INT[bam_seqi(seq, i as usize) as usize];
        i += 1;
    }
    i = xb;
    while i < xe {
        if i >= ref_len || *ref_.add(i as usize) == 0 {
            xe = i;
            break;
        }
        *tref.add((i - xb) as usize) =
            SEQ_NT16_INT[SEQ_NT16_TABLE[*ref_.add(i as usize) as u8 as usize] as usize];
        i += 1;
    }

    let mut state_buf = vec![0 as i32; (*b).core.l_qseq as usize];
    let ref_slice = std::slice::from_raw_parts(tref, (xe - xb) as usize);
    let query_slice = std::slice::from_raw_parts(tseq, (*b).core.l_qseq as usize);
    let qual_slice = std::slice::from_raw_parts(qual, (*b).core.l_qseq as usize);
    let q_slice = std::slice::from_raw_parts_mut(q, (*b).core.l_qseq as usize);
    if crate::htslib_rs::probaln::probaln_glocal(
        ref_slice,
        query_slice,
        Some(qual_slice),
        &conf,
        Some((state_buf.as_mut_slice(), q_slice)),
    ) == i32::MIN
    {
        return -4;
    }

    if extend_baq == 0 {
        k = 0;
        x = (*b).core.pos;
        y = 0;
        while k < (*b).core.n_cigar as i32 {
            let op = (*cigar.add(k as usize) & 0xf) as i32;
            let mut l = (*cigar.add(k as usize) >> 4) as i32;
            if l == 0 {
                k += 1;
                continue;
            }
            if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                if l > (*b).core.l_qseq - y {
                    l = (*b).core.l_qseq - y;
                }
                i = y as hts_pos_t;
                while i < (y + l) as hts_pos_t {
                    let state = state_buf[i as usize];
                    if (state & 3) != 0 || (state >> 2) != (x - xb + (i - y as hts_pos_t)) as i32
                    {
                        *bq.add(i as usize) = 0;
                    } else {
                        *bq.add(i as usize) = (*bq.add(i as usize)).min(*q.add(i as usize));
                    }
                    i += 1;
                }
                x += l as hts_pos_t;
                y += l;
            } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
                if l > (*b).core.l_qseq - y {
                    l = (*b).core.l_qseq - y;
                }
                y += l;
            } else if op == BAM_CDEL {
                x += l as hts_pos_t;
            }
            k += 1;
        }
        i = 0;
        while i < (*b).core.l_qseq as hts_pos_t {
            *bq.add(i as usize) =
                ((*qual.add(i as usize) as i32) - (*bq.add(i as usize) as i32) + 64) as u8;
            i += 1;
        }
    } else {
        let left = tseq;
        let rght = tref;
        let mut len: i32 = 0;

        k = 0;
        x = (*b).core.pos;
        y = 0;
        while k < (*b).core.n_cigar as i32 {
            let op = (*cigar.add(k as usize) & 0xf) as i32;
            let mut l = (*cigar.add(k as usize) >> 4) as i32;
            if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                if k + 1 < (*b).core.n_cigar as i32 {
                    let next_op = bam_cigar_op(*cigar.add(k as usize + 1));
                    if next_op == BAM_CMATCH || next_op == BAM_CEQUAL || next_op == BAM_CDIFF {
                        len += l;
                        k += 1;
                        continue;
                    }
                }
                l += len;
                len = 0;
            }

            if l == 0 {
                k += 1;
                continue;
            }
            if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                if l > (*b).core.l_qseq - y {
                    l = (*b).core.l_qseq - y;
                }
                i = y as hts_pos_t;
                while i < (y + l) as hts_pos_t {
                    let state = state_buf[i as usize];
                    *bq.add(i as usize) = if (state & 3) != 0
                        || (state >> 2) != (x - xb + (i - y as hts_pos_t)) as i32
                    {
                        0
                    } else {
                        *q.add(i as usize)
                    };
                    i += 1;
                }
                *left.add(y as usize) = *bq.add(y as usize);
                i = (y + 1) as hts_pos_t;
                while i < (y + l) as hts_pos_t {
                    *left.add(i as usize) = (*bq.add(i as usize)).max(*left.add(i as usize - 1));
                    i += 1;
                }
                *rght.add((y + l - 1) as usize) = *bq.add((y + l - 1) as usize);
                if l > 1 {
                    i = (y + l - 2) as hts_pos_t;
                    loop {
                        *rght.add(i as usize) =
                            (*bq.add(i as usize)).max(*rght.add(i as usize + 1));
                        if i == y as hts_pos_t {
                            break;
                        }
                        i -= 1;
                    }
                }
                i = y as hts_pos_t;
                while i < (y + l) as hts_pos_t {
                    *bq.add(i as usize) = (*left.add(i as usize)).min(*rght.add(i as usize));
                    i += 1;
                }
                x += l as hts_pos_t;
                y += l;
            } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
                if l > (*b).core.l_qseq - y {
                    l = (*b).core.l_qseq - y;
                }
                y += l;
            } else if op == BAM_CDEL {
                x += l as hts_pos_t;
            }
            k += 1;
        }
        i = 0;
        while i < (*b).core.l_qseq as hts_pos_t {
            *bq.add(i as usize) = (64
                + if *qual.add(i as usize) <= *bq.add(i as usize) {
                    0
                } else {
                    (*qual.add(i as usize) as i32) - (*bq.add(i as usize) as i32)
                }) as u8;
            i += 1;
        }
    }

    if apply_baq != 0 {
        i = 0;
        while i < (*b).core.l_qseq as hts_pos_t {
            *qual.add(i as usize) =
                ((*qual.add(i as usize) as i32) - ((*bq.add(i as usize) as i32) - 64)) as u8;
            i += 1;
        }
        bam_aux_append(b, c"ZQ".as_ptr().cast(), b'Z' as u8, (*b).core.l_qseq + 1, bq);
    } else {
        bam_aux_append(b, c"BQ".as_ptr().cast(), b'Z' as u8, (*b).core.l_qseq + 1, bq);
    }

    0
}

pub unsafe fn realn_check_tag(
    tg: *const u8,
    _severity: htsLogLevel,
    _type: *const u8,
    b: *const bam1_t,
) -> i32 {
    if *tg != b'Z' {
        return -1;
    }
    if (*b).core.l_qseq as usize != libc::strlen(tg.add(1).cast()) {
        return -1;
    }
    0
}

pub unsafe fn bam_endpos(b: *const bam1_t) -> hts_pos_t {
    let mut rlen = if ((*b).core.flag as i32 & BAM_FUNMAP) != 0 {
        0
    } else {
        bam_cigar2rlen((*b).core.n_cigar as i32, bam_get_cigar(b))
    };
    if rlen == 0 {
        rlen = 1;
    }
    (*b).core.pos + rlen
}

pub unsafe fn bam_get_cigar(b: *const bam1_t) -> *const u32 {
    (*b).data.as_ptr().add((*b).core.l_qname as usize) as *const u32
}

pub unsafe fn bam_is_rev(b: *const bam1_t) -> bool {
    ((*b).core.flag as i32 & BAM_FREVERSE) != 0
}

pub unsafe fn bam_is_mrev(b: *const bam1_t) -> bool {
    ((*b).core.flag as i32 & BAM_FMREVERSE) != 0
}

pub unsafe fn bam_get_qname(b: *const bam1_t) -> *mut u8 {
    // returns *mut: take a mut pointer into the owned Vec.
    (*(b as *mut bam1_t)).data.as_mut_ptr().cast()
}

pub unsafe fn bam_get_seq(b: *const bam1_t) -> *const u8 {
    (*b).data
        .as_ptr()
        .add(((*b).core.n_cigar << 2) as usize + (*b).core.l_qname as usize)
}

pub unsafe fn bam_get_qual(b: *const bam1_t) -> *const u8 {
    (*b).data.as_ptr().add(
        ((*b).core.n_cigar << 2) as usize
            + (*b).core.l_qname as usize
            + (((*b).core.l_qseq + 1) >> 1) as usize,
    )
}

pub unsafe fn bam_get_aux(b: *const bam1_t) -> *const u8 {
    (*b).data.as_ptr().add(
        ((*b).core.n_cigar << 2) as usize
            + (*b).core.l_qname as usize
            + (((*b).core.l_qseq + 1) >> 1) as usize
            + (*b).core.l_qseq as usize,
    )
}

pub unsafe fn bam_get_l_aux(b: *const bam1_t) -> i32 {
    ((*b).data.len() as i32)
        - (((*b).core.n_cigar << 2) as i32)
        - (*b).core.l_qname as i32
        - (*b).core.l_qseq
        - (((*b).core.l_qseq + 1) >> 1)
}

unsafe fn fixup_missing_qname_nul(b: *mut bam1_t) -> i32 {
    let c = &mut (*b).core;
    if c.l_extranul > 0 {
        *(*b).data.as_mut_ptr().add(c.l_qname as usize) = 0;
        c.l_qname += 1;
        c.l_extranul -= 1;
    } else {
        if (*b).data.len() as i32 > i32::MAX - 4 {
            return -1;
        }
        // realloc_bam_data grows data.len() by 4 (zero-filled).
        if realloc_bam_data(b, (*b).data.len() + 4) < 0 {
            return -1;
        }
        *(*b).data.as_mut_ptr().add(c.l_qname as usize) = 0;
        c.l_qname += 1;
        c.l_extranul = 3;
    }
    0
}

unsafe fn bam_tag2cigar(b: *mut bam1_t, recal_bin: i32, _give_warning: i32) -> i32 {
    let c = &mut (*b).core;
    let test_cg = BAM_CSOFT_CLIP as u32 | ((c.l_qseq as u32) << BAM_CIGAR_SHIFT);
    if c.n_cigar == 0 || test_cg != *bam_get_cigar(b) {
        return 0;
    }
    if c.tid < 0 || c.pos < 0 {
        return 0;
    }

    let cg = bam_aux_get(b, c"CG".as_ptr().cast());
    let saved_errno = *libc::__errno_location();
    if cg.is_null() {
        if *libc::__errno_location()
            != libc::ENOENT as i32
        {
            return -1;
        }
        *libc::__errno_location() = saved_errno;
        return 0;
    }
    if *cg != b'B' || (*cg.add(1) != b'I' && *cg.add(1) != b'i') {
        return 0;
    }

    let cigar0 = bam_get_cigar(b).cast_mut();
    let fake_bytes = c.n_cigar * 4;
    let cg_len = u32::from_le_bytes([*cg.add(2), *cg.add(3), *cg.add(4), *cg.add(5)]);
    if cg_len < c.n_cigar || cg_len >= (1_u32 << 29) {
        return 0;
    }

    let cigar_st = (cigar0.cast::<u8>()).offset_from((*b).data.as_ptr()) as u32;
    c.n_cigar = cg_len;
    let n_cigar4 = c.n_cigar * 4;
    let cg_st = cg.offset_from((*b).data.as_ptr()) as u32 - 2;
    let cg_en = cg_st + 8 + n_cigar4;
    let ori_len = (*b).data.len() as u32;
    if possibly_expand_bam_data(b, (n_cigar4 - fake_bytes) as usize) < 0 {
        return -1;
    }
    if realloc_bam_data(b, (ori_len - fake_bytes + n_cigar4) as usize) < 0 {
        return -1;
    }

    std::ptr::copy((*b).data.as_mut_ptr().add((cigar_st + fake_bytes) as usize).cast::<u8>(), (*b).data.as_mut_ptr().add((cigar_st + n_cigar4) as usize).cast::<u8>(), ((ori_len - (cigar_st + fake_bytes))) as usize);
    std::ptr::copy_nonoverlapping((*b).data
            .as_mut_ptr()
            .add((n_cigar4 - fake_bytes + cg_st + 8) as usize)
            .cast::<u8>(), (*b).data.as_mut_ptr().add(cigar_st as usize).cast::<u8>(), (n_cigar4) as usize);
    if ori_len > cg_en {
        std::ptr::copy((*b).data
                .as_mut_ptr()
                .add((cg_en + n_cigar4 - fake_bytes) as usize)
                .cast::<u8>(), (*b).data
                .as_mut_ptr()
                .add((cg_st + n_cigar4 - fake_bytes) as usize)
                .cast::<u8>(), ((ori_len - cg_en)) as usize);
    }
    {
        let new_len = (*b).data.len() - (n_cigar4 + 8) as usize;
        (*b).data.truncate(new_len);
    }
    if recal_bin != 0 {
        c.bin = hts_reg2bin(c.pos, bam_endpos(b), 14, 5) as u16;
    }
    1
}

pub unsafe fn bam_aux_tag(s: *const u8) -> *const u8 {
    s.sub(2).cast()
}

pub unsafe fn bam_aux_type(s: *const u8) -> u8 {
    *s as u8
}

fn aux_type2size(type_: u8) -> i32 {
    match type_ {
        b'A' | b'c' | b'C' => 1,
        b's' | b'S' => 2,
        b'i' | b'I' | b'f' => 4,
        b'd' => 8,
        b'Z' | b'H' | b'B' => type_ as i32,
        _ => 0,
    }
}

unsafe fn sam_c_755_swap_data(
    c: *const bam1_core_t,
    _l_data: i32,
    data: *mut u8,
    _is_host: i32,
) {
    let cigar = data.add((*c).l_qname as usize).cast::<u32>();
    for i in 0..(*c).n_cigar {
        ed_swap_4p(cigar.add(i as usize).cast());
    }
}

pub unsafe fn bam_write1(fp: *mut BGZF, b: *const bam1_t) -> i32 {
    let c = &(*b).core;
    let mut block_len = ((*b).data.len() as i32 - c.l_extranul as i32 + 32) as u32;
    let mut x = [0u32; 8];

    let qname_len = c.l_qname as u32 - c.l_extranul as u32;
    if qname_len > 255 {
        *libc::__errno_location() =
            libc::EOVERFLOW as i32;
        return -1;
    }
    if c.n_cigar > 0xffff {
        block_len = block_len.wrapping_add(16);
    }
    if c.pos > i32::MAX as hts_pos_t
        || c.mpos > i32::MAX as hts_pos_t
        || c.isize < i32::MIN as hts_pos_t
        || c.isize > i32::MAX as hts_pos_t
    {
        return -1;
    }

    x[0] = c.tid as u32;
    x[1] = c.pos as u32;
    x[2] = ((c.bin as u32) << 16) | ((c.qual as u32) << 8) | qname_len;
    x[3] = if c.n_cigar > 0xffff {
        ((c.flag as u32) << 16) | 2
    } else {
        ((c.flag as u32) << 16) | (c.n_cigar & 0xffff)
    };
    x[4] = c.l_qseq as u32;
    x[5] = c.mtid as u32;
    x[6] = c.mpos as u32;
    x[7] = c.isize as u32;

    let mut ok = bgzf_flush_try(fp, 4 + block_len as isize) >= 0;
    if ((*fp).bitfields & (1 << 19)) != 0 {
        for xi in &mut x {
            ed_swap_4p((xi as *mut u32).cast());
        }
        let mut y = block_len;
        if ok {
            ok = bgzf_write_small(fp, ed_swap_4p((&mut y as *mut u32).cast()), 4) >= 0;
        }
        sam_c_755_swap_data(c, (*b).data.len() as i32, (*b).data.as_ptr().cast_mut(), 1);
    } else if ok {
        ok = bgzf_write_small(fp, (&block_len as *const u32).cast(), 4) >= 0;
    }
    if ok {
        ok = bgzf_write_small(fp, x.as_ptr().cast(), 32) >= 0;
    }
    if ok {
        ok = bgzf_write_small(fp, (*b).data.as_ptr().cast(), qname_len as usize) >= 0;
    }
    if c.n_cigar <= 0xffff {
        if ok {
            ok = bgzf_write_small(
                fp,
                (*b).data.as_ptr().add(c.l_qname as usize).cast(),
                ((*b).data.len() as u32 - c.l_qname as u32) as usize,
            ) >= 0;
        }
    } else {
        let mut buf = [0u8; 8];
        let cigreflen = bam_cigar2rlen(c.n_cigar as i32, bam_get_cigar(b));
        if cigreflen >= (1 << 28) {
            return -1;
        }
        let cigar_st = bam_get_cigar(b).cast::<u8>().offset_from((*b).data.as_ptr()) as u32;
        let cigar_en = cigar_st + c.n_cigar * 4;
        let cigar0 = (c.l_qseq as u32) << 4 | BAM_CSOFT_CLIP as u32;
        let cigar1 = (cigreflen as u32) << 4 | BAM_CREF_SKIP as u32;
        u32_to_le(cigar0, buf.as_mut_ptr());
        u32_to_le(cigar1, buf.as_mut_ptr().add(4));
        if ok {
            ok = bgzf_write_small(fp, buf.as_ptr().cast(), 8) >= 0;
        }
        if ok {
            ok = bgzf_write_small(
                fp,
                (*b).data.as_ptr().add(cigar_en as usize).cast(),
                ((*b).data.len() as u32 - cigar_en) as usize,
            ) >= 0;
        }
        if ok {
            ok = bgzf_write_small(fp, c"CGBI".as_ptr().cast(), 4) >= 0;
        }
        u32_to_le(c.n_cigar, buf.as_mut_ptr());
        if ok {
            ok = bgzf_write_small(fp, buf.as_ptr().cast(), 4) >= 0;
        }
        if ok {
            ok = bgzf_write_small(
                fp,
                (*b).data.as_ptr().add(cigar_st as usize).cast(),
                c.n_cigar as usize * 4,
            ) >= 0;
        }
    }
    if ((*fp).bitfields & (1 << 19)) != 0 {
        sam_c_755_swap_data(c, (*b).data.len() as i32, (*b).data.as_ptr().cast_mut(), 0);
    }
    if ok {
        (4 + block_len) as i32
    } else {
        -1
    }
}

unsafe fn sam_c_933_bam_write_idx1(
    fp: *mut htsFile,
    _h: *const sam_hdr_t,
    b: *const bam1_t,
) -> i32 {
    let bfp = (*fp).fp.bgzf;
    if (*fp).idx.is_null() {
        return bam_write1(bfp, b);
    }
    -1
}

pub unsafe fn sam_c_4553_sam_write1(
    fp: *mut htsFile,
    h: *const sam_hdr_t,
    b: *const bam1_t,
) -> i32 {
    match (*fp).format.format {
        HTS_FORMAT_BINARY_FORMAT => {
            (*fp).format.category = HTS_FORMAT_SEQUENCE_DATA;
            (*fp).format.format = HTS_FORMAT_BAM;
            sam_c_933_bam_write_idx1(fp, h, b)
        }
        HTS_FORMAT_BAM => sam_c_933_bam_write_idx1(fp, h, b),
        HTS_FORMAT_CRAM => crate::htslib_rs::cram::cram_cram_encode_c_4049_cram_put_bam_seq(
            (*fp).fp.cram,
            b.cast_mut(),
        ),
        HTS_FORMAT_TEXT_FORMAT => {
            (*fp).format.category = HTS_FORMAT_SEQUENCE_DATA;
            (*fp).format.format = HTS_FORMAT_SAM;
            sam_c_4553_sam_write1(fp, h, b)
        }
        HTS_FORMAT_SAM => {
            // fp.state stays null for SAM (sam_c_3746_sam_set_threads is a
            // native noop — see comment there). fp.idx is set by
            // sam_idx_init when the caller wants an on-the-fly index — we
            // update it natively below after writing the record.
            if sam_format1(h, b, &mut (*fp).line) < 0 {
                return -1;
            }
            kputc(b'\n' as i32, &mut (*fp).line);
            let line_len = (*fp).line.data.len();
            let line_ptr = (*fp).line.data.as_ptr();
            if ((*fp).bitfields & (1 << 4)) != 0 {
                if bgzf_flush_try((*fp).fp.bgzf, line_len as isize) < 0 {
                    return -1;
                }
                if bgzf_write((*fp).fp.bgzf, line_ptr.cast(), line_len) != line_len as isize {
                    return -1;
                }
            } else if crate::htslib_rs::hfile::htslib_hfile_h_292_hwrite(
                (*fp).fp.hfile,
                line_ptr.cast(),
                line_len,
            ) != line_len as isize
            {
                return -1;
            }
            // On-the-fly index update (htslib/sam.c:4665) when sam_idx_init
            // set up fp.idx. BGZF-SAM uses bgzf_idx_push; plain SAM uses
            // hts_idx_push with the raw offset.
            if !(*fp).idx.is_null() {
                let core = &(*b).core;
                let not_unmapped = ((core.flag as i32) & BAM_FUNMAP) == 0;
                let bgzf = (*fp).fp.bgzf;
                let pos =
                    (((*bgzf).block_address as u64) << 16) | ((*bgzf).block_offset as u64 & 0xFFFF);
                if (*fp).format.compression == crate::htslib_rs::hts::HTS_COMPRESSION_BGZF {
                    if crate::htslib_rs::bgzf::bgzf_c_189_bgzf_idx_push(
                        bgzf,
                        (*fp).idx.cast(),
                        core.tid,
                        core.pos,
                        bam_endpos(b),
                        pos,
                        not_unmapped as i32,
                    ) < 0
                    {
                        return -1;
                    }
                } else if hts_idx_push(
                    (*fp).idx.cast(),
                    core.tid,
                    core.pos,
                    bam_endpos(b),
                    pos,
                    not_unmapped as i32,
                ) < 0
                {
                    return -1;
                }
            }
            (*fp).line.data.len() as i32
        }
        HTS_FORMAT_FASTA_FORMAT | HTS_FORMAT_FASTQ_FORMAT => {
            if (*fp).state.is_null() {
                (*fp).state = sam_c_3786_fastq_state_init(if (*fp).format.format
                    == HTS_FORMAT_FASTQ_FORMAT
                {
                    b'@'
                } else {
                    b'>'
                } as i32)
                .cast();
                if (*fp).state.is_null() {
                    return -2;
                }
            }
            if sam_c_4413_fastq_format1((*fp).state.cast(), b, &mut (*fp).line) < 0 {
                return -1;
            }
            let line_len = (*fp).line.data.len();
            let line_ptr = (*fp).line.data.as_ptr();
            if ((*fp).bitfields & (1 << 4)) != 0 {
                if bgzf_flush_try((*fp).fp.bgzf, line_len as isize) < 0 {
                    return -1;
                }
                if bgzf_write((*fp).fp.bgzf, line_ptr.cast(), line_len) != line_len as isize {
                    return -1;
                }
            } else if crate::htslib_rs::hfile::htslib_hfile_h_292_hwrite(
                (*fp).fp.hfile,
                line_ptr.cast(),
                line_len,
            ) != line_len as isize
            {
                return -1;
            }
            (*fp).line.data.len() as i32
        }
        _ => {
            *libc::__errno_location() = libc::EBADF;
            -1
        }
    }
}

pub unsafe fn bam_set_qname(rec: *mut bam1_t, qname: *const u8) -> i32 {
    if rec.is_null() {
        return -1;
    }
    if qname.is_null() || *qname == 0 {
        return -1;
    }

    let old_len = (*rec).core.l_qname as usize;
    let new_len = libc::strlen(qname.cast()) + 1;
    if !(1..=255).contains(&new_len) {
        return -1;
    }
    let extranul = if !new_len.is_multiple_of(4) {
        4 - new_len % 4
    } else {
        0
    };
    let old_data_len = (*rec).data.len();
    let new_data_len = old_data_len - old_len + new_len + extranul;
    if realloc_bam_data(rec, new_data_len) < 0 {
        return -1;
    }
    // realloc_bam_data grows len to new_data_len when larger; when the qname
    // shrinks, len stays at old_data_len until the post-memmove truncate below,
    // keeping the moved tail in-bounds.
    if new_len + extranul != (*rec).core.l_qname as usize {
        libc::memmove(
            (*rec).data.as_mut_ptr().add(new_len + extranul).cast(),
            (*rec).data.as_mut_ptr().add((*rec).core.l_qname as usize).cast(),
            old_data_len - (*rec).core.l_qname as usize,
        );
    }
    std::ptr::copy_nonoverlapping(qname.cast::<u8>(), (*rec).data.as_mut_ptr().cast::<u8>(), (new_len) as usize);
    for n in 0..extranul {
        *(*rec).data.as_mut_ptr().add(new_len + n) = 0;
    }
    (*rec).data.truncate(new_data_len);
    (*rec).core.l_qname = (new_len + extranul) as u16;
    (*rec).core.l_extranul = extranul as u8;
    0
}

pub unsafe fn aux_to_le(type_: u8, mut out: *mut u8, in_: *const u8, len: usize) -> i32 {
    let tsz = aux_type2size(type_ as u8);
    if (2..=8).contains(&tsz) && (len & (tsz as usize - 1)) != 0 {
        return -1;
    }

    match tsz {
        x if x == b'H' as i32 || x == b'Z' as i32 || x == 1 => {
            std::ptr::copy_nonoverlapping(in_.cast::<u8>(), out.cast(), (len) as usize);
        }
        2 => {
            let mut i = 0;
            while i < len {
                let v = u16::from_ne_bytes([*in_.add(i), *in_.add(i + 1)]);
                u16_to_le(v, out);
                out = out.add(2);
                i += 2;
            }
        }
        4 => {
            let mut i = 0;
            while i < len {
                let v = u32::from_ne_bytes([
                    *in_.add(i),
                    *in_.add(i + 1),
                    *in_.add(i + 2),
                    *in_.add(i + 3),
                ]);
                u32_to_le(v, out);
                out = out.add(4);
                i += 4;
            }
        }
        8 => {
            let mut i = 0;
            while i < len {
                let v = u64::from_ne_bytes([
                    *in_.add(i),
                    *in_.add(i + 1),
                    *in_.add(i + 2),
                    *in_.add(i + 3),
                    *in_.add(i + 4),
                    *in_.add(i + 5),
                    *in_.add(i + 6),
                    *in_.add(i + 7),
                ]);
                u64_to_le(v, out);
                out = out.add(8);
                i += 8;
            }
        }
        x if x == b'B' as i32 => {
            if len < 5 {
                return -1;
            }
            let n = u32::from_ne_bytes([*in_.add(1), *in_.add(2), *in_.add(3), *in_.add(4)]);
            *out = *in_;
            u32_to_le(n, out.add(1));
            return aux_to_le(*in_ as u8, out.add(5), in_.add(5), len - 5);
        }
        _ => return -1,
    }

    0
}

unsafe fn skip_aux(mut s: *mut u8, end: *mut u8) -> *mut u8 {
    if s >= end {
        return end;
    }
    let mut size = aux_type2size(*s);
    s = s.add(1);
    match size {
        x if x == b'Z' as i32 || x == b'H' as i32 => {
            while s < end {
                if *s == 0 {
                    return s.add(1);
                }
                s = s.add(1);
            }
            end
        }
        x if x == b'B' as i32 => {
            if end.offset_from(s) < 5 {
                return std::ptr::null_mut();
            }
            size = aux_type2size(*s);
            s = s.add(1);
            let n = u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]) as isize;
            s = s.add(4);
            if size == 0 || end.offset_from(s) < size as isize * n {
                return std::ptr::null_mut();
            }
            s.offset(size as isize * n)
        }
        0 => std::ptr::null_mut(),
        _ => {
            if end.offset_from(s) < size as isize {
                return std::ptr::null_mut();
            }
            s.add(size as usize)
        }
    }
}

pub unsafe fn bam_aux_first(b: *const bam1_t) -> *mut u8 {
    let s = bam_get_aux(b).cast_mut();
    let end = (*b).data.as_ptr().add((*b).data.len()).cast_mut();
    if end.offset_from(s) <= 2 {
        *libc::__errno_location() =
            libc::ENOENT as i32;
        return std::ptr::null_mut();
    }
    s.add(2)
}

pub unsafe fn bam_aux_next(b: *const bam1_t, s: *const u8) -> *mut u8 {
    let end = (*b).data.as_ptr().add((*b).data.len()).cast_mut();
    let next = if s.is_null() {
        end
    } else {
        skip_aux(s.cast_mut(), end)
    };
    if next.is_null() {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return std::ptr::null_mut();
    }
    if end.offset_from(next) <= 2 {
        *libc::__errno_location() =
            libc::ENOENT as i32;
        return std::ptr::null_mut();
    }
    next.add(2)
}

pub unsafe fn bam_aux_get(b: *const bam1_t, tag: *const u8) -> *mut u8 {
    let mut s = bam_aux_first(b);
    while !s.is_null() {
        if *s.sub(2) == *tag.cast::<u8>() && *s.sub(1) == *tag.cast::<u8>().add(1) {
            let e = skip_aux(s, (*b).data.as_ptr().add((*b).data.len()).cast_mut());
            if e.is_null() || ((*s == b'Z' || *s == b'H') && *e.sub(1) != 0) {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null_mut();
            }
            return s;
        }
        s = bam_aux_next(b, s);
    }
    std::ptr::null_mut()
}

pub unsafe fn sam_format_aux1(
    key: *const u8,
    type_: u8,
    tag: *const u8,
    end: *const u8,
    ks: *mut kstring_t,
) -> *const u8 {
    let mut r = 0;
    let mut s = tag;
    let key_slice = std::slice::from_raw_parts(key, 2);
    r |= (kputsn_(key_slice, 2, &mut *ks) < 0) as i32;
    r |= (kputc_(b':' as i32, &mut *ks) < 0) as i32;

    match type_ {
        b'C' => {
            r |= (kputsn_(b"i:", 2, &mut *ks) < 0) as i32;
            r |= (kputw(*s as i32, &mut *ks) < 0) as i32;
            s = s.add(1);
        }
        b'c' => {
            r |= (kputsn_(b"i:", 2, &mut *ks) < 0) as i32;
            r |= (kputw(*(s.cast::<i8>()) as i32, &mut *ks) < 0) as i32;
            s = s.add(1);
        }
        b'S' => {
            if end.offset_from(s) < 2 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:", 2, &mut *ks) < 0) as i32;
            r |= (kputuw(u16::from_le_bytes([*s, *s.add(1)]) as u32, &mut *ks) < 0) as i32;
            s = s.add(2);
        }
        b's' => {
            if end.offset_from(s) < 2 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:", 2, &mut *ks) < 0) as i32;
            r |= (kputw(i16::from_le_bytes([*s, *s.add(1)]) as i32, &mut *ks) < 0) as i32;
            s = s.add(2);
        }
        b'I' => {
            if end.offset_from(s) < 4 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:", 2, &mut *ks) < 0) as i32;
            r |= (kputuw(
                u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                &mut *ks,
            ) < 0) as i32;
            s = s.add(4);
        }
        b'i' => {
            if end.offset_from(s) < 4 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:", 2, &mut *ks) < 0) as i32;
            r |= (kputw(
                i32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                &mut *ks,
            ) < 0) as i32;
            s = s.add(4);
        }
        b'A' => {
            if s >= end {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"A:", 2, &mut *ks) < 0) as i32;
            r |= (kputc_(*s as i32, &mut *ks) < 0) as i32;
            s = s.add(1);
        }
        b'f' => {
            if end.offset_from(s) < 4 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"f:", 2, &mut *ks) < 0) as i32;
            r |= (sam_put_aux_float(
                f32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]) as f64,
                ks,
            ) < 0) as i32;
            s = s.add(4);
        }
        b'd' => {
            if end.offset_from(s) < 8 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"d:", 2, &mut *ks) < 0) as i32;
            r |= (sam_put_aux_float(
                f64::from_le_bytes([
                    *s,
                    *s.add(1),
                    *s.add(2),
                    *s.add(3),
                    *s.add(4),
                    *s.add(5),
                    *s.add(6),
                    *s.add(7),
                ]),
                ks,
            ) < 0) as i32;
            s = s.add(8);
        }
        b'Z' | b'H' => {
            r |= (kputc_(type_ as i32, &mut *ks) < 0) as i32;
            r |= (kputc_(b':' as i32, &mut *ks) < 0) as i32;
            while s < end && *s != 0 {
                r |= (kputc_(*s as i32, &mut *ks) < 0) as i32;
                s = s.add(1);
            }
            r |= (kputsn(b"", 0, &mut *ks) < 0) as i32;
            if s >= end {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            s = s.add(1);
        }
        b'B' => {
            if s >= end {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            let sub_type = *s;
            s = s.add(1);
            let sub_type_size = match sub_type {
                b'A' | b'c' | b'C' => 1usize,
                b's' | b'S' => 2,
                b'i' | b'I' | b'f' => 4,
                _ => 0,
            };
            if sub_type_size == 0 || end.offset_from(s) < 4 {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            let n = u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]);
            s = s.add(4);
            if (end.offset_from(s) as usize) / sub_type_size < n as usize {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            r |= (kputsn_(b"B:", 2, &mut *ks) < 0) as i32;
            r |= (kputc(sub_type as i32, &mut *ks) < 0) as i32;
            if sub_type == b'A' {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return std::ptr::null();
            }
            if ks_expand(&mut *ks, n as usize * 12) < 0 {
                *libc::__errno_location() =
                    libc::ENOMEM as i32;
                return std::ptr::null();
            }
            for _ in 0..n {
                (*ks).data.push(b',');
                match sub_type {
                    b'c' => r |= (kputw(*(s.cast::<i8>()) as i32, &mut *ks) < 0) as i32,
                    b'C' | b'A' => r |= (kputuw(*s as u32, &mut *ks) < 0) as i32,
                    b's' => {
                        r |= (kputw(i16::from_le_bytes([*s, *s.add(1)]) as i32, &mut *ks) < 0)
                            as i32;
                    }
                    b'S' => {
                        r |= (kputuw(u16::from_le_bytes([*s, *s.add(1)]) as u32, &mut *ks) < 0)
                            as i32;
                    }
                    b'i' => {
                        r |= (kputw(
                            i32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                            &mut *ks,
                        ) < 0) as i32;
                    }
                    b'I' => {
                        r |= (kputuw(
                            u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                            &mut *ks,
                        ) < 0) as i32;
                    }
                    b'f' => {
                        r |= (sam_put_aux_float(
                            f32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]) as f64,
                            ks,
                        ) < 0) as i32;
                    }
                    _ => {
                        *libc::__errno_location() =
                            libc::EINVAL as i32;
                        return std::ptr::null();
                    }
                }
                s = s.add(sub_type_size);
            }
        }
        _ => {
            *libc::__errno_location() =
                libc::EINVAL as i32;
            return std::ptr::null();
        }
    }

    if r != 0 {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        std::ptr::null()
    } else {
        s
    }
}

unsafe fn sam_put_aux_float(value: f64, ks: *mut kstring_t) -> i32 {
    let mut buf = [0 as u8; 128];
    let len = libc::snprintf(buf.as_mut_ptr().cast(), buf.len(), c"%.6g".as_ptr(), value);
    if len < 0 || len as usize >= buf.len() {
        return -1;
    }
    let bytes = std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), len as usize);
    kputsn(bytes, len as usize, &mut *ks)
}

unsafe fn sam_c_4317_add33(a: *mut u8, b: *const u8, len: i32) {
    for i in 0..len {
        *a.add(i as usize) = (*b.add(i as usize)).wrapping_add(33);
    }
}

unsafe fn sam_c_4324_sam_format1_append(
    h: *const sam_hdr_t,
    b: *const bam1_t,
    str_: *mut kstring_t,
) -> i32 {
    let mut r = 0;
    let c = &(*b).core;
    const BAM_CIGAR_STR: &[u8; 10] = b"MIDNSHP=XB";

    if c.l_qname == 0 {
        return -1;
    }
    let qname = std::slice::from_raw_parts(
        bam_get_qname(b).cast::<u8>(),
        (c.l_qname - 1 - c.l_extranul as u16) as usize,
    );
    r |= (kputsn_(qname, qname.len(), &mut *str_) < 0) as i32;
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    r |= (kputw(c.flag as i32, &mut *str_) < 0) as i32;
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    if c.tid >= 0 {
        let tn = CStr::from_ptr((*(*h).target_name.add(c.tid as usize)).cast()).to_bytes();
        r |= (kputs(tn, &mut *str_) < 0) as i32;
        r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    } else {
        r |= (kputsn_(b"*\t", 2, &mut *str_) < 0) as i32;
    }
    r |= (kputll(c.pos + 1, &mut *str_) < 0) as i32;
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    r |= (kputw(c.qual as i32, &mut *str_) < 0) as i32;
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    if c.n_cigar != 0 {
        let cigar = bam_get_cigar(b);
        for i in 0..c.n_cigar {
            let cig = *cigar.add(i as usize);
            r |= (kputw(bam_cigar_oplen(cig) as i32, &mut *str_) < 0) as i32;
            r |= (kputc_(BAM_CIGAR_STR[bam_cigar_op(cig) as usize] as i32, &mut *str_) < 0)
                as i32;
        }
    } else {
        r |= (kputc_(b'*' as i32, &mut *str_) < 0) as i32;
    }
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    if c.mtid < 0 {
        r |= (kputsn_(b"*\t", 2, &mut *str_) < 0) as i32;
    } else if c.mtid == c.tid {
        r |= (kputsn_(b"=\t", 2, &mut *str_) < 0) as i32;
    } else {
        let tn = CStr::from_ptr((*(*h).target_name.add(c.mtid as usize)).cast()).to_bytes();
        r |= (kputs(tn, &mut *str_) < 0) as i32;
        r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    }
    r |= (kputll(c.mpos + 1, &mut *str_) < 0) as i32;
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
    r |= (kputll(c.isize, &mut *str_) < 0) as i32;
    r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;

    if c.l_qseq != 0 {
        let l_qseq = c.l_qseq as usize;
        // Append seq (unpacked) + '\t' + qual directly into the owned Vec.
        let base = (*str_).data.len();
        (*str_).data.resize(base + l_qseq, 0);
        let packed = std::slice::from_raw_parts(bam_get_seq(b), l_qseq.div_ceil(2));
        let seq = std::slice::from_raw_parts_mut(
            (*str_).data.as_mut_ptr().add(base).cast::<u8>(),
            l_qseq,
        );
        nibble2base(packed, seq);
        (*str_).data.push(b'\t');

        let qual = bam_get_qual(b);
        if *qual == 0xff {
            (*str_).data.push(b'*');
        } else {
            let qstart = (*str_).data.len();
            (*str_).data.resize(qstart + l_qseq, 0);
            sam_c_4317_add33(
                (*str_).data.as_mut_ptr().add(qstart),
                qual,
                c.l_qseq,
            );
        }
    } else {
        r |= (kputsn_(b"*\t*", 3, &mut *str_) < 0) as i32;
    }

    let mut s = bam_get_aux(b);
    let end = (*b).data.as_ptr().add((*b).data.len());
    while end.offset_from(s) >= 4 {
        r |= (kputc_(b'\t' as i32, &mut *str_) < 0) as i32;
        s = sam_format_aux1(s, *s.add(2), s.add(3), end, str_).cast_mut();
        if s.is_null() {
            *libc::__errno_location() =
                libc::EINVAL as i32;
            return -1;
        }
    }
    r |= (kputsn(b"", 0, &mut *str_) < 0) as i32;
    if r != 0 {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }
    (*str_).data.len() as i32
}

pub unsafe fn sam_format1(h: *const sam_hdr_t, b: *const bam1_t, str_: *mut kstring_t) -> i32 {
    // SEAM: `str_` may be a freshly calloc'd kstring (e.g. htsFile.line, used as the
    // output buffer by sam_write1), whose Vec has a NULL data pointer. Vec::clear()
    // aborts on a NULL pointer even at len 0; truncate(0) empties it via the
    // precondition-free raw-slice path (equivalent: no free).
    (*str_).data.truncate(0);
    sam_c_4324_sam_format1_append(h, b, str_)
}

pub unsafe fn bam_aux_get_str(b: *const bam1_t, tag: *const u8, s: *mut kstring_t) -> i32 {
    let t = bam_aux_get(b, tag);
    if t.is_null() {
        return if *libc::__errno_location()
            == libc::ENOENT as i32
        {
            0
        } else {
            -1
        };
    }

    if sam_format_aux1(
        t.sub(2),
        *t,
        t.add(1),
        (*b).data.as_ptr().add((*b).data.len()),
        s,
    )
    .is_null()
    {
        -1
    } else {
        1
    }
}

unsafe fn get_int_aux_val(type_: u8, s: *const u8, idx: u32) -> i64 {
    match type_ {
        b'c' => *(s.add(idx as usize).cast::<i8>()) as i64,
        b'C' => *s.add(idx as usize) as i64,
        b's' => {
            i16::from_le_bytes([*s.add((2 * idx) as usize), *s.add((2 * idx + 1) as usize)]) as i64
        }
        b'S' => {
            u16::from_le_bytes([*s.add((2 * idx) as usize), *s.add((2 * idx + 1) as usize)]) as i64
        }
        b'i' => i32::from_le_bytes([
            *s.add((4 * idx) as usize),
            *s.add((4 * idx + 1) as usize),
            *s.add((4 * idx + 2) as usize),
            *s.add((4 * idx + 3) as usize),
        ]) as i64,
        b'I' => u32::from_le_bytes([
            *s.add((4 * idx) as usize),
            *s.add((4 * idx + 1) as usize),
            *s.add((4 * idx + 2) as usize),
            *s.add((4 * idx + 3) as usize),
        ]) as i64,
        _ => {
            *libc::__errno_location() =
                libc::EINVAL as i32;
            0
        }
    }
}

pub unsafe fn bam_aux2i(s: *const u8) -> i64 {
    let type_ = *s;
    get_int_aux_val(type_, s.add(1), 0)
}

pub unsafe fn bam_aux2f(s: *const u8) -> f64 {
    let type_ = *s;
    match type_ {
        b'd' => f64::from_le_bytes([
            *s.add(1),
            *s.add(2),
            *s.add(3),
            *s.add(4),
            *s.add(5),
            *s.add(6),
            *s.add(7),
            *s.add(8),
        ]),
        b'f' => f32::from_le_bytes([*s.add(1), *s.add(2), *s.add(3), *s.add(4)]) as f64,
        _ => get_int_aux_val(type_, s.add(1), 0) as f64,
    }
}

pub unsafe fn bam_aux2A(s: *const u8) -> u8 {
    if *s == b'A' {
        *s.add(1) as u8
    } else {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        0
    }
}

pub unsafe fn bam_aux2Z(s: *const u8) -> *mut u8 {
    if *s == b'Z' || *s == b'H' {
        s.add(1).cast::<u8>().cast_mut()
    } else {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        std::ptr::null_mut()
    }
}

pub unsafe fn bam_auxB_len(s: *const u8) -> u32 {
    if *s != b'B' {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return 0;
    }
    u32::from_le_bytes([*s.add(2), *s.add(3), *s.add(4), *s.add(5)])
}

pub unsafe fn bam_auxB2i(s: *const u8, idx: u32) -> i64 {
    let len = bam_auxB_len(s);
    if idx >= len {
        *libc::__errno_location() =
            libc::ERANGE as i32;
        return 0;
    }
    get_int_aux_val(*s.add(1), s.add(6), idx)
}

pub unsafe fn bam_auxB2f(s: *const u8, idx: u32) -> f64 {
    let len = bam_auxB_len(s);
    if idx >= len {
        *libc::__errno_location() =
            libc::ERANGE as i32;
        return 0.0;
    }
    if *s.add(1) == b'f' {
        let p = s.add(6 + 4 * idx as usize);
        f32::from_le_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]) as f64
    } else {
        get_int_aux_val(*s.add(1), s.add(6), idx) as f64
    }
}

pub unsafe fn bam_aux_append(
    b: *mut bam1_t,
    tag: *const u8,
    type_: u8,
    len: i32,
    data: *const u8,
) -> i32 {
    let Ok(add_len) = u32::try_from(3_i64 + len as i64) else {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    };
    let old_len = (*b).data.len() as u32;
    let Some(new_len) = old_len.checked_add(add_len) else {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    };
    if new_len > i32::MAX as u32 {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }
    if realloc_bam_data(b, new_len as usize) < 0 {
        return -1;
    }

    // realloc_bam_data grew data.len() to new_len; write the new tag at old_len.
    let s = (*b).data.as_mut_ptr().add(old_len as usize);
    *s = *tag.cast::<u8>();
    *s.add(1) = *tag.cast::<u8>().add(1);
    *s.add(2) = type_ as u8;
    std::ptr::copy_nonoverlapping(data.cast::<u8>(), s.add(3).cast(), (len) as usize);
    0
}

pub unsafe fn bam_aux_remove(b: *mut bam1_t, s: *mut u8) -> *mut u8 {
    let end = (*b).data.as_ptr().add((*b).data.len()).cast_mut();
    let next = skip_aux(s, end);
    if next.is_null() {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return std::ptr::null_mut();
    }

    let removed = next.offset_from(s.sub(2)) as usize;
    if next >= end {
        // Last tag: just shrink off the removed bytes.
        let new_len = (*b).data.len() - removed;
        (*b).data.truncate(new_len);
        *libc::__errno_location() =
            libc::ENOENT as i32;
        return std::ptr::null_mut();
    }
    std::ptr::copy(next.cast::<u8>(), s.sub(2).cast(), (end.offset_from(next)) as usize);
    let new_len = (*b).data.len() - removed;
    (*b).data.truncate(new_len);
    s
}

pub unsafe fn bam_aux_del(b: *mut bam1_t, s: *mut u8) -> i32 {
    let ret = bam_aux_remove(b, s);
    if !ret.is_null()
        || *libc::__errno_location()
            == libc::ENOENT as i32
    {
        0
    } else {
        -1
    }
}

unsafe fn aux_strlen(mut data: *const u8) -> usize {
    let start = data;
    while *data != 0 {
        data = data.add(1);
    }
    data.offset_from(start) as usize
}

unsafe fn possibly_expand_bam_data(b: *mut bam1_t, extra: usize) -> i32 {
    // SEAM: under the owned-Vec model data.len() IS the logical l_data, so this
    // is a pure CAPACITY reservation (reserve, do not grow len). Callers that
    // actually append bytes grow data.len() themselves via realloc_bam_data at
    // each write; this just guarantees the capacity is present up-front so those
    // grows do not reallocate mid-loop.
    let desired = (*b).data.len() + extra;
    if desired <= (*b).data.capacity() {
        return 0;
    }
    // Reserve to the kroundup'd capacity (mirrors sam_realloc_bam_data's m_data)
    // without changing len.
    if desired > (i32::MAX as f64 * 0.666) as usize {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }
    let mut new_m_data = kroundup32(desired as u32);
    new_m_data = new_m_data.wrapping_add(32);
    if (new_m_data as usize) < desired {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }
    let data = &mut (*b).data;
    data.reserve_exact((new_m_data as usize) - data.len());
    if (bam_get_mempolicy(b) & BAM_USER_OWNS_DATA) != 0 {
        bam_set_mempolicy(b, bam_get_mempolicy(b) & !BAM_USER_OWNS_DATA);
    }
    0
}

pub unsafe fn bam_aux_update_str(
    b: *mut bam1_t,
    tag: *const u8,
    len: i32,
    data: *const u8,
) -> i32 {
    let ln = if len >= 0 {
        len as usize
    } else {
        aux_strlen(data) + 1
    };
    let mut old_ln = 0usize;
    let need_nul = ln == 0 || *data.cast::<u8>().add(ln - 1) != 0;
    let save_errno = *libc::__errno_location();
    let mut new_tag = 0usize;
    let mut s = bam_aux_get(b, tag);

    if !s.is_null() {
        if *s != b'Z' {
            *libc::__errno_location() =
                libc::EINVAL as i32;
            return -1;
        }
        s = s.add(1);
        let mut e = s;
        let end = (*b).data.as_ptr().add((*b).data.len()).cast_mut();
        while e < end && *e != 0 {
            e = e.add(1);
        }
        old_ln = e.offset_from(s) as usize + 1;
        s = s.sub(3);
    } else if *libc::__errno_location()
        != libc::ENOENT as i32
    {
        return -1;
    } else {
        *libc::__errno_location() = save_errno;
        s = (*b).data.as_mut_ptr().add((*b).data.len());
        new_tag = 3;
    }

    let new_ln = ln + usize::from(need_nul);
    let s_offset = s.offset_from((*b).data.as_ptr()) as usize;
    let old_len = (*b).data.len();
    // Final logical length after this update.
    let final_len = old_len + new_tag + new_ln - old_ln;
    if old_ln < new_ln + new_tag {
        if possibly_expand_bam_data(b, new_ln + new_tag - old_ln) < 0 {
            return -1;
        }
        // Grow len to cover the new (larger) layout before shifting/writing.
        if realloc_bam_data(b, final_len) < 0 {
            return -1;
        }
    }
    s = (*b).data.as_mut_ptr().add(s_offset);
    if new_tag == 0 {
        std::ptr::copy(s.add(3 + old_ln).cast::<u8>(), s.add(3 + new_ln).cast(), ((old_len - (s_offset + 3) - old_ln)) as usize);
    }
    // Shrink case (final_len < old_len): truncate after the memmove.
    (*b).data.truncate(final_len);

    let s = (*b).data.as_mut_ptr().add(s_offset);
    *s = *tag.cast::<u8>();
    *s.add(1) = *tag.cast::<u8>().add(1);
    *s.add(2) = b'Z';
    std::ptr::copy(data.cast::<u8>(), s.add(3).cast::<u8>(), (ln) as usize);
    if need_nul {
        *s.add(3 + ln) = 0;
    }
    0
}

pub unsafe fn bam_aux_update_int(b: *mut bam1_t, tag: *const u8, val: i64) -> i32 {
    if val < i32::MIN as i64 || val > u32::MAX as i64 {
        *libc::__errno_location() =
            libc::EOVERFLOW as i32;
        return -1;
    }
    let (mut type_, mut sz): (u8, u32) = if val < i16::MIN as i64 {
        (b'i', 4)
    } else if val < i8::MIN as i64 {
        (b's', 2)
    } else if val < 0 {
        (b'c', 1)
    } else if val < u8::MAX as i64 {
        (b'C', 1)
    } else if val < u16::MAX as i64 {
        (b'S', 2)
    } else {
        (b'I', 4)
    };

    let mut old_sz = 0u32;
    let mut new = false;
    let mut s = bam_aux_get(b, tag);
    if !s.is_null() {
        old_sz = match *s {
            b'c' | b'C' => 1,
            b's' | b'S' => 2,
            b'i' | b'I' => 4,
            _ => {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return -1;
            }
        };
    } else if *libc::__errno_location()
        == libc::ENOENT as i32
    {
        s = (*b).data.as_mut_ptr().add((*b).data.len());
        new = true;
    } else {
        return -1;
    }

    let s_offset = s.offset_from((*b).data.as_ptr()) as usize;
    let old_len = (*b).data.len();
    if new || old_sz < sz {
        let final_len = old_len + (if new { 3 } else { 0 }) + sz as usize - old_sz as usize;
        if possibly_expand_bam_data(b, (if new { 3 } else { 0 }) + sz as usize - old_sz as usize)
            < 0
        {
            return -1;
        }
        // Grow len to the new (larger) layout before writing/shifting.
        if realloc_bam_data(b, final_len) < 0 {
            return -1;
        }
        s = (*b).data.as_mut_ptr().add(s_offset);
        if new {
            *s = *tag.cast::<u8>();
            *s.add(1) = *tag.cast::<u8>().add(1);
            s = s.add(2);
        } else {
            std::ptr::copy(s.add(old_sz as usize).cast::<u8>(), s.add(sz as usize).cast(), ((old_len - s_offset - old_sz as usize)) as usize);
        }
    } else {
        sz = old_sz;
        type_ = if val < 0 {
            *b"\0cs\0i".as_ptr().add(old_sz as usize)
        } else {
            *b"\0CS\0I".as_ptr().add(old_sz as usize)
        };
    }
    *s = type_;
    s = s.add(1);
    let le = val.to_le_bytes();
    std::ptr::copy_nonoverlapping(le.as_ptr().cast::<u8>(), s.cast::<u8>(), (sz) as usize);
    // In the grow/new branch realloc_bam_data already set the final len; in the
    // in-place else branch sz==old_sz so the length is unchanged. No truncate
    // needed (grow case only ever increases the length here).
    0
}

pub unsafe fn bam_aux_update_float(b: *mut bam1_t, tag: *const u8, val: f32) -> i32 {
    let mut shrink = false;
    let mut new = false;
    let mut s = bam_aux_get(b, tag);
    if !s.is_null() {
        match *s {
            b'f' => {}
            b'd' => shrink = true,
            _ => {
                *libc::__errno_location() =
                    libc::EINVAL as i32;
                return -1;
            }
        }
    } else if *libc::__errno_location()
        == libc::ENOENT as i32
    {
        new = true;
    } else {
        return -1;
    }

    if new {
        let old_len = (*b).data.len();
        if possibly_expand_bam_data(b, 7) < 0 {
            return -1;
        }
        if realloc_bam_data(b, old_len + 7) < 0 {
            return -1;
        }
        s = (*b).data.as_mut_ptr().add(old_len);
        *s = *tag.cast::<u8>();
        *s.add(1) = *tag.cast::<u8>().add(1);
        s = s.add(2);
    } else if shrink {
        let old_len = (*b).data.len();
        let tail_off = s.add(9).offset_from((*b).data.as_ptr()) as usize;
        std::ptr::copy(s.add(9).cast::<u8>(), s.add(5).cast(), ((old_len - tail_off)) as usize);
        (*b).data.truncate(old_len - 4);
    }
    *s = b'f';
    let le = val.to_le_bytes();
    std::ptr::copy_nonoverlapping(le.as_ptr().cast::<u8>(), s.add(1).cast::<u8>(), (4) as usize);
    0
}

pub unsafe fn bam_aux_update_array(
    b: *mut bam1_t,
    tag: *const u8,
    type_: u8,
    items: u32,
    data: *mut (),
) -> i32 {
    let mut old_sz = 0usize;
    let mut new = false;
    let mut s = bam_aux_get(b, tag);
    if !s.is_null() {
        if *s != b'B' {
            *libc::__errno_location() =
                libc::EINVAL as i32;
            return -1;
        }
        old_sz = aux_type2size(*s.add(1)) as usize;
        if !(1..=4).contains(&old_sz) {
            *libc::__errno_location() =
                libc::EINVAL as i32;
            return -1;
        }
        old_sz *= u32::from_le_bytes([*s.add(2), *s.add(3), *s.add(4), *s.add(5)]) as usize;
    } else if *libc::__errno_location()
        == libc::ENOENT as i32
    {
        s = (*b).data.as_mut_ptr().add((*b).data.len());
        new = true;
    } else {
        return -1;
    }

    let item_sz = aux_type2size(type_) as usize;
    if !(1..=4).contains(&item_sz) {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }
    if items as usize > i32::MAX as usize / item_sz {
        *libc::__errno_location() =
            libc::ENOMEM as i32;
        return -1;
    }
    let new_sz = item_sz * items as usize;

    let s_offset = s.offset_from((*b).data.as_ptr()) as usize;
    let old_len = (*b).data.len();
    if new || old_sz < new_sz {
        if possibly_expand_bam_data(b, (if new { 8 } else { 0 }) + new_sz - old_sz) < 0 {
            return -1;
        }
        // Grow len to the new (larger) layout before writing/shifting.
        let grown = old_len + (if new { 8 } else { 0 }) + new_sz - old_sz;
        if realloc_bam_data(b, grown) < 0 {
            return -1;
        }
        s = (*b).data.as_mut_ptr().add(s_offset);
    }
    if new {
        *s = *tag.cast::<u8>();
        *s.add(1) = *tag.cast::<u8>().add(1);
        s = s.add(2);
        *s = b'B';
        // len already grown to old_len + 8 + new_sz above.
    } else if old_sz != new_sz {
        std::ptr::copy(s.add(6 + old_sz).cast::<u8>(), s.add(6 + new_sz).cast(), ((old_len - (s_offset + 6 + old_sz))) as usize);
        // Apply shrink (old_sz > new_sz); grow was already handled above.
        let final_len = (old_len as isize + new_sz as isize - old_sz as isize) as usize;
        (*b).data.truncate(final_len);
    }

    *s.add(1) = type_;
    let len = items.to_le_bytes();
    std::ptr::copy_nonoverlapping(len.as_ptr().cast::<u8>(), s.add(2).cast::<u8>(), (4) as usize);
    if new_sz > 0 {
        std::ptr::copy_nonoverlapping(data.cast::<u8>(), s.add(6).cast::<u8>(), (new_sz) as usize);
    }
    0
}

pub unsafe fn bam_seqi(s: *const u8, i: usize) -> u8 {
    (*s.add(i >> 1) >> (((!i) & 1) << 2)) & 0x0f
}

pub unsafe fn bam_set_seqi(s: *mut u8, i: usize, b: u8) {
    let shift = ((!i) & 1) << 2;
    *s.add(i >> 1) = (*s.add(i >> 1) & (0xf0 >> shift)) | (b << shift);
}

pub fn nibble2base_default(nib: &[u8], seq: &mut [u8]) {
    static CODE2BASE: &[u8; 512] = b"===A=C=M=G=R=S=V=T=W=Y=H=K=D=B=N\
A=AAACAMAGARASAVATAWAYAHAKADABAN\
C=CACCCMCGCRCSCVCTCWCYCHCKCDCBCN\
M=MAMCMMMGMRMSMVMTMWMYMHMKMDMBMN\
G=GAGCGMGGGRGSGVGTGWGYGHGKGDGBGN\
R=RARCRMRGRRRSRVRTRWRYRHRKRDRBRN\
S=SASCSMSGSRSSSVSTSWSYSHSKSDSBSN\
V=VAVCVMVGVRVSVVVTVWVYVHVKVDVBVN\
T=TATCTMTGTRTSTVTTTWTYTHTKTDTBTN\
W=WAWCWMWGWRWSWVWTWWWYWHWKWDWBWN\
Y=YAYCYMYGYRYSYVYTYWYYYHYKYDYBYN\
H=HAHCHMHGHRHSHVHTHWHYHHHKHDHBHN\
K=KAKCKMKGKRKSKVKTKWKYKHKKKDKBKN\
D=DADCDMDGDRDSDVDTDWDYDHDKDDDBDN\
B=BABCBMBGBRBSBVBTBWBYBHBKBDBBBN\
N=NANCNMNGNRNSNVNTNWNYNHNKNDNBNN";
    static SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

    let Some(seq0) = seq.first_mut() else {
        return;
    };
    *seq0 = 0;

    let len = seq.len();
    let len2 = len / 2;
    let mut i = 0usize;
    while i < len2 {
        let idx = nib[i] as usize * 2;
        seq[i * 2] = CODE2BASE[idx] as u8;
        seq[i * 2 + 1] = CODE2BASE[idx + 1] as u8;
        i += 1;
    }

    i *= 2;
    if i < len {
        let code = (nib[i / 2] >> (((!i) & 1) << 2)) & 0x0f;
        seq[i] = SEQ_NT16_STR[code as usize] as u8;
    }
}

pub unsafe fn nibble2base(nib: &[u8], seq: &mut [u8]) {
    let Some(seq0) = seq.first_mut() else {
        return;
    };
    *seq0 = 0;
    unsafe { (crate::htslib_rs::simd::htslib_nibble2base)(nib, seq) };
}

pub unsafe fn sam_open_mode(mode: *mut u8, fn_: *const u8, format: *const u8) -> i32 {
    if format.is_null() {
        let mut extension = [0 as u8; HTS_MAX_EXT_LEN];
        if find_file_extension(fn_.cast(), extension.as_mut_ptr().cast()) < 0 {
            return -1;
        }
        return sam_open_mode(mode, fn_, extension.as_ptr());
    } else if libc::strcasecmp(format.cast(), c"bam".as_ptr()) == 0 {
        libc::strcpy(mode.cast(), c"b".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"cram".as_ptr()) == 0 {
        libc::strcpy(mode.cast(), c"c".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"sam".as_ptr()) == 0 {
        libc::strcpy(mode.cast(), c"".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"sam.gz".as_ptr()) == 0 {
        libc::strcpy(mode.cast(), c"z".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"fastq".as_ptr()) == 0
        || libc::strcasecmp(format.cast(), c"fq".as_ptr()) == 0
    {
        libc::strcpy(mode.cast(), c"f".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"fastq.gz".as_ptr()) == 0
        || libc::strcasecmp(format.cast(), c"fq.gz".as_ptr()) == 0
    {
        libc::strcpy(mode.cast(), c"fz".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"fasta".as_ptr()) == 0
        || libc::strcasecmp(format.cast(), c"fa".as_ptr()) == 0
    {
        libc::strcpy(mode.cast(), c"F".as_ptr());
    } else if libc::strcasecmp(format.cast(), c"fasta.gz".as_ptr()) == 0
        || libc::strcasecmp(format.cast(), c"fa.gz".as_ptr()) == 0
    {
        libc::strcpy(mode.cast(), c"Fz".as_ptr());
    } else {
        return -1;
    }

    0
}

pub unsafe fn sam_open_mode_opts(
    fn_: *const u8,
    mode: *const u8,
    format: *const u8,
) -> *mut u8 {
    let format_len_for_alloc = if format.is_null() {
        1
    } else {
        libc::strlen(format.cast())
    };
    let mode_len_for_alloc = if mode.is_null() {
        1
    } else {
        libc::strlen(mode.cast())
    };
    let mode_opts =
        { let mut v: Vec<u8> = vec![0u8; ((format_len_for_alloc + mode_len_for_alloc + 12)) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };

    if mode_opts.is_null() {
        return std::ptr::null_mut();
    }

    libc::strcpy(mode_opts.cast(), if mode.is_null() { c"r".as_ptr() } else { mode.cast() });
    let mut cp = mode_opts.add(libc::strlen(mode_opts.cast()));

    if format.is_null() {
        let mut extension = [0 as u8; HTS_MAX_EXT_LEN];
        if find_file_extension(fn_.cast(), extension.as_mut_ptr().cast()) < 0 {
            {
            let cap = format_len_for_alloc + mode_len_for_alloc + 12;
            drop(Vec::from_raw_parts(mode_opts, cap, cap));
        }
            return std::ptr::null_mut();
        }
        if sam_open_mode(cp, fn_, extension.as_ptr()) == 0 {
            return mode_opts;
        } else {
            {
            let cap = format_len_for_alloc + mode_len_for_alloc + 12;
            drop(Vec::from_raw_parts(mode_opts, cap, cap));
        }
            return std::ptr::null_mut();
        }
    }

    let opts = libc::strchr(format.cast(), b',' as i32);
    let (opts, format_len) = if opts.is_null() {
        (c"".as_ptr(), libc::strlen(format.cast()))
    } else {
        (opts.cast_const().cast(), opts.offset_from(format.cast()) as usize)
    };

    if libc::strncmp(format.cast(), c"bam".as_ptr(), format_len) == 0 {
        *cp = b'b' as u8;
        cp = cp.add(1);
    } else if libc::strncmp(format.cast(), c"cram".as_ptr(), format_len) == 0 {
        *cp = b'c' as u8;
        cp = cp.add(1);
    } else if libc::strncmp(format.cast(), c"cram2".as_ptr(), format_len) == 0 {
        *cp = b'c' as u8;
        cp = cp.add(1);
        libc::strcpy(cp.cast(), c",VERSION=2.1".as_ptr());
        cp = cp.add(12);
    } else if libc::strncmp(format.cast(), c"cram3".as_ptr(), format_len) == 0 {
        *cp = b'c' as u8;
        cp = cp.add(1);
        libc::strcpy(cp.cast(), c",VERSION=3.0".as_ptr());
        cp = cp.add(12);
    } else if libc::strncmp(format.cast(), c"sam".as_ptr(), format_len) == 0 {
    } else if libc::strncmp(format.cast(), c"sam.gz".as_ptr(), format_len) == 0 {
        *cp = b'z' as u8;
        cp = cp.add(1);
    } else if libc::strncmp(format.cast(), c"fastq".as_ptr(), format_len) == 0
        || libc::strncmp(format.cast(), c"fq".as_ptr(), format_len) == 0
    {
        *cp = b'f' as u8;
        cp = cp.add(1);
    } else if libc::strncmp(format.cast(), c"fastq.gz".as_ptr(), format_len) == 0
        || libc::strncmp(format.cast(), c"fq.gz".as_ptr(), format_len) == 0
    {
        *cp = b'f' as u8;
        cp = cp.add(1);
        *cp = b'z' as u8;
        cp = cp.add(1);
    } else if libc::strncmp(format.cast(), c"fasta".as_ptr(), format_len) == 0
        || libc::strncmp(format.cast(), c"fa".as_ptr(), format_len) == 0
    {
        *cp = b'F' as u8;
        cp = cp.add(1);
    } else if libc::strncmp(format.cast(), c"fasta.gz".as_ptr(), format_len) == 0
        || libc::strncmp(format.cast(), c"fa".as_ptr(), format_len) == 0
    {
        *cp = b'F' as u8;
        cp = cp.add(1);
        *cp = b'z' as u8;
        cp = cp.add(1);
    } else {
        {
            let cap = format_len_for_alloc + mode_len_for_alloc + 12;
            drop(Vec::from_raw_parts(mode_opts, cap, cap));
        }
        return std::ptr::null_mut();
    }

    libc::strcpy(cp.cast(), opts);
    mode_opts
}

pub unsafe fn bam_str2flag(str_: *const u8) -> i32 {
    let mut end: *mut u8 = std::ptr::null_mut();
    let numeric = libc::strtol(str_.cast(), (&mut end as *mut *mut u8).cast(), 0);
    if end != str_.cast_mut() {
        return numeric as i32;
    }

    let mut flag = 0;
    let mut beg = str_;
    while *beg != 0 {
        let mut end = beg;
        while *end != 0 && *end != b',' as u8 {
            end = end.add(1);
        }
        let len = end.offset_from(beg) as usize;
        let word = std::slice::from_raw_parts(beg.cast::<u8>(), len);
        if word.eq_ignore_ascii_case(b"PAIRED") {
            flag |= BAM_FPAIRED;
        } else if word.eq_ignore_ascii_case(b"PROPER_PAIR") {
            flag |= BAM_FPROPER_PAIR;
        } else if word.eq_ignore_ascii_case(b"UNMAP") {
            flag |= BAM_FUNMAP;
        } else if word.eq_ignore_ascii_case(b"MUNMAP") {
            flag |= BAM_FMUNMAP;
        } else if word.eq_ignore_ascii_case(b"REVERSE") {
            flag |= BAM_FREVERSE;
        } else if word.eq_ignore_ascii_case(b"MREVERSE") {
            flag |= BAM_FMREVERSE;
        } else if word.eq_ignore_ascii_case(b"READ1") {
            flag |= BAM_FREAD1;
        } else if word.eq_ignore_ascii_case(b"READ2") {
            flag |= BAM_FREAD2;
        } else if word.eq_ignore_ascii_case(b"SECONDARY") {
            flag |= BAM_FSECONDARY;
        } else if word.eq_ignore_ascii_case(b"QCFAIL") {
            flag |= BAM_FQCFAIL;
        } else if word.eq_ignore_ascii_case(b"DUP") {
            flag |= BAM_FDUP;
        } else if word.eq_ignore_ascii_case(b"SUPPLEMENTARY") {
            flag |= BAM_FSUPPLEMENTARY;
        } else {
            return -1;
        }
        if *end == 0 {
            break;
        }
        beg = end.add(1);
    }
    flag
}

pub unsafe fn bam_flag2str(flag: i32) -> *mut u8 {
    let mut str_ = kstring_t::default();
    let flags = [
        (BAM_FPAIRED, b"PAIRED" as &[u8]),
        (BAM_FPROPER_PAIR, b"PROPER_PAIR" as &[u8]),
        (BAM_FUNMAP, b"UNMAP" as &[u8]),
        (BAM_FMUNMAP, b"MUNMAP" as &[u8]),
        (BAM_FREVERSE, b"REVERSE" as &[u8]),
        (BAM_FMREVERSE, b"MREVERSE" as &[u8]),
        (BAM_FREAD1, b"READ1" as &[u8]),
        (BAM_FREAD2, b"READ2" as &[u8]),
        (BAM_FSECONDARY, b"SECONDARY" as &[u8]),
        (BAM_FQCFAIL, b"QCFAIL" as &[u8]),
        (BAM_FDUP, b"DUP" as &[u8]),
        (BAM_FSUPPLEMENTARY, b"SUPPLEMENTARY" as &[u8]),
    ];
    for (bit, name) in flags {
        if (flag & bit) != 0 {
            if !str_.data.is_empty() {
                kputsn(b",", 1, &mut str_);
            }
            kputsn(name, name.len(), &mut str_);
        }
    }
    // Return a Vec-owned NUL-terminated string (the caller reconstructs and
    // drops it); leak the owned bytes plus a trailing NUL at this boundary.
    let mut out_vec = ks_release(&mut str_);
    out_vec.push(0);
    let out = out_vec.as_mut_ptr();
    std::mem::forget(out_vec);
    out
}

// Functions translated from htslib/sam.c. Although the function names start
// with `sam_hdr_*`, htslib's source places them in sam.c (not header.c),
// so they live here.

pub unsafe fn sam_hdr_init() -> *mut sam_hdr_t {
    let h = Box::into_raw(Box::new(std::mem::zeroed::<sam_hdr_t>())).cast::<()>()
        .cast::<sam_hdr_t>();
    if h.is_null() {
        return std::ptr::null_mut();
    }
    (*h).cigar_tab = BAM_CIGAR_TABLE.as_ptr();
    h
}

pub unsafe fn sam_hdr_parse(_l_text: usize, _text: *const u8) -> *mut sam_hdr_t {
    let h = sam_hdr_init();
    if h.is_null() {
        return std::ptr::null_mut();
    }
    if _text.is_null() {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    let lines = if _l_text == 0 {
        CStr::from_ptr(_text.cast()).to_bytes()
    } else {
        std::slice::from_raw_parts(_text.cast::<u8>(), _l_text)
    };
    if sam_hdr_add_lines(&mut *h, lines) != 0 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    h
}

pub unsafe fn sam_hdr_parse_(hdr: *const u8, len: usize) -> *mut sam_hdr_t {
    sam_hdr_parse(len, hdr)
}

pub unsafe fn sam_hdr_dup(_h0: *const sam_hdr_t) -> *mut sam_hdr_t {
    if _h0.is_null() {
        return std::ptr::null_mut();
    }
    if !(*_h0).hrecs.is_null() {
        let h = sam_hdr_init();
        if h.is_null() {
            return std::ptr::null_mut();
        }
        (*h).n_targets = 0;
        (*h).ignore_sam_err = (*_h0).ignore_sam_err;
        (*h).l_text = 0;
        if sam_c_170_sam_hdr_dup_hrecs(_h0, h) < 0 {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        return h;
    }

    let h = sam_hdr_init();
    if h.is_null() {
        return std::ptr::null_mut();
    }

    (*h).n_targets = 0;
    (*h).ignore_sam_err = (*_h0).ignore_sam_err;
    (*h).l_text = 0;

    if (*_h0).n_targets > 0 {
        (*h).target_len = ({ let mut v: Vec<u32> = vec![0u32; (*_h0).n_targets as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
        .cast();
        if (*h).target_len.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        (*h).target_name = ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut::<u8>(); (*_h0).n_targets as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
        .cast();
        if (*h).target_name.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }

        let mut i = 0;
        while i < (*_h0).n_targets {
            *(*h).target_len.add(i as usize) = *(*_h0).target_len.add(i as usize);
            *(*h).target_name.add(i as usize) = {
                let src = *(*_h0).target_name.add(i as usize);
                if src.is_null() {
                    std::ptr::null_mut()
                } else {
                    let mut len = 0usize;
                    while *src.add(len) != 0 {
                        len += 1;
                    }
                    let mut dup =
                        std::slice::from_raw_parts(src, len + 1).to_vec();
                    let p = dup.as_mut_ptr();
                    std::mem::forget(dup);
                    p
                }
            };
            if (*(*h).target_name.add(i as usize)).is_null() {
                break;
            }
            i += 1;
        }
        (*h).n_targets = i;
        if i < (*_h0).n_targets {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
    }

    (*h).l_text = if !(*_h0).text.is_null() {
        (*_h0).l_text
    } else {
        0
    };
    (*h).text = ({ let mut v: Vec<u8> = vec![0u8; ((*h).l_text as u64 + 1) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
    if (*h).text.is_null() {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    if !(*_h0).text.is_null() {
        std::ptr::copy_nonoverlapping((*_h0).text.cast::<u8>(), (*h).text.cast(), ((*h).l_text) as usize);
    }
    *(*h).text.add((*h).l_text as usize) = 0;

    if !(*_h0).sdict.is_null() && sam_c_144_sam_hdr_dup_sdict(_h0, h) < 0 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }

    h
}

pub unsafe fn sam_hdr_write(fp: *mut htsFile, h: *const sam_hdr_t) -> i32 {
    if fp.is_null() || h.is_null() {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return -1;
    }

    match (*fp).format.format {
        HTS_FORMAT_BINARY_FORMAT => {
            (*fp).format.category = HTS_FORMAT_SEQUENCE_DATA;
            (*fp).format.format = HTS_FORMAT_BAM;
            if bam_hdr_write((*fp).fp.bgzf, h) < 0 {
                return -1;
            }
        }
        HTS_FORMAT_BAM => {
            if bam_hdr_write((*fp).fp.bgzf, h) < 0 {
                return -1;
            }
        }
        HTS_FORMAT_CRAM => return sam_hdr_write_cram(fp, h),
        HTS_FORMAT_TEXT_FORMAT => {
            (*fp).format.category = HTS_FORMAT_SEQUENCE_DATA;
            (*fp).format.format = HTS_FORMAT_SAM;
            return sam_hdr_write(fp, h);
        }
        HTS_FORMAT_SAM => {
            if (*h).hrecs.is_null() && (*h).text.is_null() {
                return 0;
            }
            // Rust-built hrecs is the source of truth — sync text and fall
            // through. Production never has unmarked hrecs (sam_hdr_dup on
            // a C-pool source rebuilds into a hrecs-null header); a caller
            // that somehow gets one falls through using (*h).text directly.
            if !(*h).hrecs.is_null()
                && sam_hdr_has_rust_hrecs(h.cast_mut())
                && sam_hdr_rebuild(&mut *h.cast_mut()) < 0
            {
                return -1;
            }
            if !(*h).text.is_null() {
                let text = (*h).text;
                let l_text = (*h).l_text;
                let mut q = text;
                let p = loop {
                    let p = libc::strstr(q.cast(), c"@SQ\t".as_ptr()).cast::<u8>();
                    if p.is_null() || p == text || *p.sub(1) == b'\n' as u8 {
                        break p;
                    }
                    q = p.add(4);
                };
                let no_sq = p.is_null();
                if sam_hdr_write_bytes(fp, text.cast(), l_text) < 0 {
                    return -1;
                }

                if no_sq {
                    for i in 0..(*h).n_targets {
                        (*fp).line.data.truncate(0);
                        let tn = CStr::from_ptr((*(*h).target_name.add(i as usize)).cast()).to_bytes();
                        if kputsn(b"@SQ\tSN:", 7, &mut (*fp).line) < 0
                            || kputs(tn, &mut (*fp).line) < 0
                            || kputsn(b"\tLN:", 4, &mut (*fp).line) < 0
                            || kputw(*(*h).target_len.add(i as usize) as i32, &mut (*fp).line) < 0
                            || kputc(b'\n' as i32, &mut (*fp).line) < 0
                        {
                            return -1;
                        }
                        if sam_hdr_write_bytes(fp, (*fp).line.data.as_ptr().cast(), (*fp).line.data.len()) < 0 {
                            return -1;
                        }
                    }
                }
            }

            if ((*fp).bitfields & (1 << 4)) != 0 {
                if bgzf_flush((*fp).fp.bgzf) != 0 {
                    return -1;
                }
            } else if hflush((*fp).fp.hfile) != 0 {
                return -1;
            }
        }
        HTS_FORMAT_FASTQ_FORMAT | HTS_FORMAT_FASTA_FORMAT => return 0,
        _ => {
            *libc::__errno_location() = libc::EBADF as i32;
            return -1;
        }
    }

    sam_hdr_write_store_copy(fp, h)
}

pub unsafe fn sam_hdr_read(_fp: *mut htsFile) -> *mut sam_hdr_t {
    if _fp.is_null() {
        *libc::__errno_location() =
            libc::EINVAL as i32;
        return std::ptr::null_mut();
    }
    let h = match (*_fp).format.format {
        HTS_FORMAT_BAM => sam_hdr_sanitise(bam_hdr_read((*_fp).fp.bgzf)),
        HTS_FORMAT_FASTQ_FORMAT | HTS_FORMAT_FASTA_FORMAT => sam_hdr_init(),
        HTS_FORMAT_EMPTY_FORMAT => {
            *libc::__errno_location() =
                libc::EPIPE as i32;
            return std::ptr::null_mut();
        }
        HTS_FORMAT_SAM => sam_c_1907_sam_hdr_create(_fp),
        HTS_FORMAT_CRAM => {
            // Mirrors libhts' sam_hdr_read CRAM branch (htslib/sam.c:1939):
            //   h = sam_hdr_sanitise(sam_hdr_dup(fp->fp.cram->header));
            // `cram_dopen` already consumed and parsed the CRAM SAM header
            // into fd->header during hts_open — re-reading would
            // double-consume bytes off the file. We duplicate the cached
            // header instead.
            //
            // Native sam_hdr_dup walks the source hrecs and rebuilds text
            // + target arrays into a freshly sam_hdr_init()'d header — the
            // result is fully Rust-allocated (libc::malloc'd text/target
            // arrays, hrecs=null), so the c_owned marker isn't needed and
            // sam_hdr_destroy goes through the regular Rust teardown path.
            // The original cram_fd->header (C-pool-allocated by cram_dopen)
            // is freed by cram_close when the cram_fd is destroyed.
            let src =
                crate::htslib_rs::cram::cram_fd_header_ptr((*_fp).fp.cram).cast::<sam_hdr_t>();
            sam_hdr_sanitise(sam_hdr_dup(src))
        }
        _ => {
            *libc::__errno_location() =
                libc::ENOEXEC as i32;
            return std::ptr::null_mut();
        }
    };

    if !h.is_null() && (*_fp).bam_header.is_null() {
        (*_fp).bam_header = h.cast();
        sam_hdr_incr_ref(&mut *h);
    }
    h
}

pub unsafe fn sam_hdr_destroy(_h: *mut sam_hdr_t) {
    if _h.is_null() {
        return;
    }
    // The c_owned tracker (sam_hdr_c_owned_registry) is no longer populated:
    // the CRAM-read flow now returns a Rust-allocated dup of the cached
    // cram_fd->header (sam_hdr_dup → sam_c_170_sam_hdr_dup_hrecs walks the
    // source hrecs and rebuilds text/targets into a freshly sam_hdr_init()'d
    // header). All sam_hdr_t pointers exposed to production code own only
    // Rust-allocated memory and follow the standard teardown path below.
    if (*_h).ref_count > 0 {
        (*_h).ref_count -= 1;
        return;
    }
    if let Ok(mut scratch) = sam_hdr_text_scratch().lock() {
        scratch.remove(&(_h as usize));
    }
    sam_hdr_forget_rust_hrecs(_h);
    if !(*_h).target_name.is_null() {
        let n = (*_h).n_targets as usize;
        for i in 0..n {
            let name = *(*_h).target_name.add(i);
            if !name.is_null() {
                let mut l = 0usize;
                while *name.add(l) != 0 {
                    l += 1;
                }
                drop(Vec::from_raw_parts(name, l + 1, l + 1));
            }
        }
        drop(Vec::from_raw_parts((*_h).target_name, n, n));
        if !(*_h).target_len.is_null() {
            drop(Vec::from_raw_parts((*_h).target_len, n, n));
        }
    }
    if !(*_h).text.is_null() {
        drop(Vec::from_raw_parts(
            (*_h).text,
            (*_h).l_text + 1,
            (*_h).l_text + 1,
        ));
    }
    kh_destroy_s2i((*_h).sdict.cast());
    drop(Box::from_raw(_h));
}

pub unsafe fn sam_hdr_free(hdr: *mut sam_hdr_t) {
    sam_hdr_destroy(hdr);
}

pub unsafe fn sam_hdr_change_HD(
    h: *mut sam_hdr_t,
    key: *const u8,
    val: *const u8,
) -> i32 {
    if h.is_null() || key.is_null() {
        return -1;
    }
    if (*h).hrecs.is_null() {
        return sam_c_2080_old_sam_hdr_change_HD(h, key, val);
    }
    sam_c_2157_sam_hdr_change_HD_hrecs(h, key, val)
}

pub unsafe fn sam_hdr_set(fp: *mut htsFile, h: *mut sam_hdr_t, duplicate: i32) -> i32 {
    if fp.is_null() {
        return -1;
    }

    if duplicate != 0 {
        let tmp = (*fp).bam_header;
        (*fp).bam_header = sam_hdr_dup(h).cast();
        sam_hdr_destroy(tmp.cast());
        if (*fp).bam_header.is_null() && !h.is_null() {
            return -1;
        }
    } else if (*fp).bam_header != h.cast() {
        sam_hdr_destroy((*fp).bam_header.cast());
        (*fp).bam_header = h.cast();
        sam_hdr_incr_ref(&mut *(*fp).bam_header.cast::<sam_hdr_t>());
    }

    0
}

pub unsafe fn sam_hdr_get(fp: *mut htsFile) -> *mut sam_hdr_t {
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    (*fp).bam_header.cast()
}

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};
    use std::mem::{align_of, size_of};

    use super::*;

    #[test]
    fn public_bam_struct_layout_matches_htslib_abi_shape() {
        assert_eq!(size_of::<bam1_core_t>(), 48);
        assert_eq!(align_of::<bam1_core_t>(), 8);
        // SEAM: the owned bam1_t is no longer ABI-identical to C HTSlib. The
        // `data: *mut u8 + l_data: i32 + m_data: u32` triple (16 bytes incl.
        // padding) became `data: Vec<u8>` (24 bytes), growing the struct from 80
        // to 88. The repr(C) `bam1_c_t` mirror keeps the 80-byte C layout for FFI.
        assert_eq!(size_of::<bam1_t>(), 88);
        assert_eq!(align_of::<bam1_t>(), 8);
        assert_eq!(size_of::<bam_pileup_cd>(), 8);
        assert_eq!(align_of::<bam_pileup_cd>(), 8);
        assert_eq!(size_of::<bam_pileup1_t>(), 40);
        assert_eq!(align_of::<bam_pileup1_t>(), 8);
        assert_eq!(size_of::<sam_hrec_tag_t>(), 24);
        assert_eq!(align_of::<sam_hrec_tag_t>(), 8);
        assert_eq!(size_of::<sam_hrec_type_t>(), 48);
        assert_eq!(align_of::<sam_hrec_type_t>(), 8);
        assert_eq!(size_of::<sam_hrec_sq_t>(), 24);
        assert_eq!(align_of::<sam_hrec_sq_t>(), 8);
        assert_eq!(size_of::<sam_hrec_rg_t>(), 24);
        assert_eq!(align_of::<sam_hrec_rg_t>(), 8);
        assert_eq!(size_of::<sam_hrec_pg_t>(), 32);
        assert_eq!(align_of::<sam_hrec_pg_t>(), 8);
        assert_eq!(size_of::<sam_hdr_t>(), 72);
        assert_eq!(align_of::<sam_hdr_t>(), 8);
        assert_eq!(size_of::<cstate_t>(), 24);
        assert_eq!(align_of::<cstate_t>(), 8);
        // lbnode_t embeds bam1_t at offset 0, so its size and the offsets of the
        // trailing fields grow by 8 alongside bam1_t (80 -> 88).
        assert_eq!(size_of::<lbnode_t>(), 144);
        assert_eq!(align_of::<lbnode_t>(), 8);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, n_targets), 0);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, ignore_sam_err), 4);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, l_text), 8);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, target_len), 16);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, cigar_tab), 24);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, target_name), 32);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, text), 40);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, sdict), 48);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, hrecs), 56);
        assert_eq!(std::mem::offset_of!(sam_hdr_t, ref_count), 64);
        assert_eq!(std::mem::offset_of!(sam_hrec_tag_t, next), 0);
        assert_eq!(std::mem::offset_of!(sam_hrec_tag_t, str_), 8);
        assert_eq!(std::mem::offset_of!(sam_hrec_tag_t, len), 16);
        assert_eq!(std::mem::offset_of!(sam_hrec_rg_t, name), 0);
        assert_eq!(std::mem::offset_of!(sam_hrec_rg_t, ty), 8);
        assert_eq!(std::mem::offset_of!(sam_hrec_rg_t, name_len), 16);
        assert_eq!(std::mem::offset_of!(sam_hrec_rg_t, id), 20);
        assert_eq!(std::mem::offset_of!(sam_hrec_pg_t, name), 0);
        assert_eq!(std::mem::offset_of!(sam_hrec_pg_t, ty), 8);
        assert_eq!(std::mem::offset_of!(sam_hrec_pg_t, name_len), 16);
        assert_eq!(std::mem::offset_of!(sam_hrec_pg_t, id), 20);
        assert_eq!(std::mem::offset_of!(sam_hrec_pg_t, prev_id), 24);
        assert_eq!(std::mem::offset_of!(sam_hrec_type_t, next), 0);
        assert_eq!(std::mem::offset_of!(sam_hrec_type_t, prev), 8);
        assert_eq!(std::mem::offset_of!(sam_hrec_type_t, global_next), 16);
        assert_eq!(std::mem::offset_of!(sam_hrec_type_t, global_prev), 24);
        assert_eq!(std::mem::offset_of!(sam_hrec_type_t, tag), 32);
        assert_eq!(std::mem::offset_of!(sam_hrec_type_t, type_), 40);
        assert_eq!(std::mem::offset_of!(cstate_t, k), 0);
        assert_eq!(std::mem::offset_of!(cstate_t, y), 4);
        assert_eq!(std::mem::offset_of!(cstate_t, x), 8);
        assert_eq!(std::mem::offset_of!(cstate_t, end), 16);
        assert_eq!(std::mem::offset_of!(lbnode_t, b), 0);
        assert_eq!(std::mem::offset_of!(lbnode_t, beg), 88);
        assert_eq!(std::mem::offset_of!(lbnode_t, end), 96);
        assert_eq!(std::mem::offset_of!(lbnode_t, s), 104);
        assert_eq!(std::mem::offset_of!(lbnode_t, next), 128);
        assert_eq!(std::mem::offset_of!(lbnode_t, cd), 136);
    }

    #[test]
    fn sam_hrecs_order_accessors_walk_hrec_tags_read_only() {
        unsafe fn set_tag(tag: *mut sam_hrec_tag_t, value: &'static CStr) {
            (*tag).str_ = value.as_ptr().cast();
            (*tag).len = value.to_bytes().len() as i32;
        }

        unsafe {
            let mut go = sam_hrec_tag_t {
                next: std::ptr::null_mut(),
                str_: c"GO:reference".as_ptr().cast(),
                len: c"GO:reference".to_bytes().len() as i32,
            };
            let mut so = sam_hrec_tag_t {
                next: &mut go,
                str_: c"SO:coordinate".as_ptr().cast(),
                len: c"SO:coordinate".to_bytes().len() as i32,
            };
            let mut hd = sam_hrec_type_t {
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
                global_next: std::ptr::null_mut(),
                global_prev: std::ptr::null_mut(),
                tag: &mut so,
                type_: header_h_58_TYPEKEY(c"HD".as_ptr().cast()),
            };
            let mut hrecs: sam_hrecs_t = std::mem::zeroed();
            hrecs.first_line = (&mut hd as *mut sam_hrec_type_t).cast();

            assert_eq!(sam_hrecs_sort_order(&mut hrecs), ORDER_COORD);
            assert_eq!(sam_hrecs_group_order(&mut hrecs), ORDER_GO_REFERENCE);

            set_tag(&mut so, c"SO:queryname");
            set_tag(&mut go, c"GO:query");
            assert_eq!(sam_hrecs_sort_order(&mut hrecs), ORDER_NAME);
            assert_eq!(sam_hrecs_group_order(&mut hrecs), ORDER_GO_QUERY);

            set_tag(&mut so, c"SO:not-a-sort");
            set_tag(&mut go, c"GO:not-a-group");
            assert_eq!(sam_hrecs_sort_order(&mut hrecs), ORDER_UNKNOWN);
            assert_eq!(sam_hrecs_group_order(&mut hrecs), ORDER_GO_UNKNOWN);
        }
    }

    #[test]
    fn sam_hrecs_order_accessors_default_without_hd_line() {
        unsafe {
            let mut sq = sam_hrec_type_t {
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
                global_next: std::ptr::null_mut(),
                global_prev: std::ptr::null_mut(),
                tag: std::ptr::null_mut(),
                type_: header_h_58_TYPEKEY(c"SQ".as_ptr().cast()),
            };
            let mut hrecs: sam_hrecs_t = std::mem::zeroed();
            hrecs.first_line = (&mut sq as *mut sam_hrec_type_t).cast();

            assert_eq!(sam_hrecs_sort_order(&mut hrecs), ORDER_UNSORTED);
            assert_eq!(sam_hrecs_group_order(&mut hrecs), ORDER_GO_NONE);
        }
    }

    #[test]
    fn sam_hrecs_new_and_free_initialise_lifecycle_fields() {
        unsafe {
            let hrecs = sam_hrecs_new();
            assert!(!hrecs.is_null());
            assert_eq!((*hrecs).ID_cnt, 1);
            assert_eq!((*hrecs).refs_changed, -1);
            assert!(!(*hrecs).ref_hash.is_null());
            assert!(!(*hrecs).rg_hash.is_null());
            assert!(!(*hrecs).pg_hash.is_null());
            assert_eq!((*hrecs).type_count, 5);
            assert!(!(*hrecs).type_order.is_null());
            assert_eq!((*(*hrecs).type_order.add(0))[0], b'H' as u8);
            assert_eq!((*(*hrecs).type_order.add(0))[1], b'D' as u8);
            assert_eq!((*(*hrecs).type_order.add(4))[0], b'C' as u8);
            assert_eq!((*(*hrecs).type_order.add(4))[1], b'O' as u8);
            sam_hrecs_free(hrecs);
            sam_hrecs_free(std::ptr::null_mut());
        }
    }

    #[test]
    fn sam_hrecs_find_type_and_key_walks_global_records_and_hashes() {
        unsafe {
            let mut rg_id = sam_hrec_tag_t {
                next: std::ptr::null_mut(),
                str_: c"ID:rg1".as_ptr().cast(),
                len: c"ID:rg1".to_bytes().len() as i32,
            };
            let mut sq_len = sam_hrec_tag_t {
                next: std::ptr::null_mut(),
                str_: c"LN:100".as_ptr().cast(),
                len: c"LN:100".to_bytes().len() as i32,
            };
            let mut sq_sn = sam_hrec_tag_t {
                next: &mut sq_len,
                str_: c"SN:chr1".as_ptr().cast(),
                len: c"SN:chr1".to_bytes().len() as i32,
            };
            let mut rg = sam_hrec_type_t {
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
                global_next: std::ptr::null_mut(),
                global_prev: std::ptr::null_mut(),
                tag: &mut rg_id,
                type_: header_h_58_TYPEKEY(c"RG".as_ptr().cast()),
            };
            let mut sq = sam_hrec_type_t {
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
                global_next: &mut rg,
                global_prev: std::ptr::null_mut(),
                tag: &mut sq_sn,
                type_: header_h_58_TYPEKEY(c"SQ".as_ptr().cast()),
            };
            rg.global_prev = &mut sq;
            let mut refs = [sam_hrec_sq_t {
                name: c"chr1".as_ptr().cast(),
                len: 100,
                ty: (&mut sq as *mut sam_hrec_type_t).cast(),
            }];
            let mut rgs = [sam_hrec_rg_t {
                name: c"rg1".as_ptr().cast(),
                ty: (&mut rg as *mut sam_hrec_type_t).cast(),
                name_len: 3,
                id: 0,
            }];
            let hrecs = sam_hrecs_new();
            assert!(!hrecs.is_null());
            (*hrecs).first_line = (&mut sq as *mut sam_hrec_type_t).cast();
            (*hrecs).nref = 1;
            (*hrecs).ref_ = refs.as_mut_ptr();
            (*hrecs).nrg = 1;
            (*hrecs).rg = rgs.as_mut_ptr().cast();
            assert!(khash_str2int_set((*hrecs).ref_hash, c"chr1".as_ptr().cast(), 0) >= 0);
            assert!(khash_str2int_set((*hrecs).rg_hash, c"rg1".as_ptr().cast(), 0) >= 0);

            assert_eq!(
                sam_hrecs_find_type_id(&mut *hrecs, b"SQ", Some((b"SN", b"chr1"))),
                Some(NonNull::from(&sq))
            );
            assert_eq!(
                sam_hrecs_find_type_id(&mut *hrecs, b"RG", Some((b"ID", b"rg1"))),
                Some(NonNull::from(&rg))
            );
            assert!(std::ptr::eq(
                sam_hrecs_find_type_pos(hrecs, c"RG".as_ptr().cast(), 0),
                &rg
            ));
            assert_eq!(
                sam_hrecs_find_type_id(&mut *hrecs, b"RG", Some((b"ID", b"missing"))),
                None
            );

            let (found, prev) = sam_hrecs_find_key(&mut sq, b"LN");
            assert_eq!(found, Some(NonNull::from(&sq_len)));
            assert_eq!(prev, Some(NonNull::from(&sq_sn)));
            (*hrecs).ref_ = std::ptr::null_mut();
            (*hrecs).rg = std::ptr::null_mut();
            (*hrecs).first_line = std::ptr::null_mut();
            sam_hrecs_free(hrecs);
        }
    }

    #[test]
    fn sam_hrecs_remove_key_marks_dirty_and_rebuilds_header_text() {
        unsafe {
            let mut sq_len = sam_hrec_tag_t {
                next: std::ptr::null_mut(),
                str_: c"LN:100".as_ptr().cast(),
                len: c"LN:100".to_bytes().len() as i32,
            };
            let mut sq_sn = sam_hrec_tag_t {
                next: &mut sq_len,
                str_: c"SN:chr1".as_ptr().cast(),
                len: c"SN:chr1".to_bytes().len() as i32,
            };
            let mut sq = sam_hrec_type_t {
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
                global_next: std::ptr::null_mut(),
                global_prev: std::ptr::null_mut(),
                tag: &mut sq_sn,
                type_: header_h_58_TYPEKEY(c"SQ".as_ptr().cast()),
            };
            let hrecs = sam_hrecs_new();
            assert!(!hrecs.is_null());
            (*hrecs).first_line = (&mut sq as *mut sam_hrec_type_t).cast();

            let mut ks = kstring_t::default();
            assert_eq!(sam_hrecs_rebuild_text(&*hrecs, &mut ks), 0);
            assert_eq!(ks.data.as_slice(), b"@SQ\tSN:chr1\tLN:100\n");
            ks_free(&mut ks);

            assert_eq!(sam_hrecs_remove_key(&mut *hrecs, &mut sq, b"LN"), 1);
            assert_eq!((*hrecs).dirty, 1);

            let mut hdr = sam_hdr_t {
                n_targets: 0,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: std::ptr::null_mut(),
                cigar_tab: std::ptr::null(),
                target_name: std::ptr::null_mut(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs,
                ref_count: 0,
            };
            assert_eq!(sam_hdr_rebuild(&mut hdr), 0);
            assert_eq!((*hrecs).dirty, 0);
            assert_eq!(sam_hdr_length(&mut hdr), b"@SQ\tSN:chr1\n".len());
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut hdr).cast()).to_bytes(),
                b"@SQ\tSN:chr1\n"
            );
            if !hdr.text.is_null() {
                drop(Vec::from_raw_parts(hdr.text, hdr.l_text + 1, hdr.l_text + 1));
            }
            (*hrecs).first_line = std::ptr::null_mut();
            sam_hrecs_free(hrecs);
        }
    }

    #[test]
    fn sam_hrecs_parse_update_hashes_and_targets_from_text() {
        unsafe {
            let text = c"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@RG\tID:rg1\tSM:s1\n@PG\tID:pg1\n@CO\tfree text\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_fill_hrecs(&mut *hdr), 0);
            assert!(!(*hdr).hrecs.is_null());
            assert_eq!((*(*hdr).hrecs).nref, 1);
            assert_eq!((*(*hdr).hrecs).nrg, 1);
            assert_eq!((*(*hdr).hrecs).npg, 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);

            let rg = sam_hrecs_find_rg(&mut *(*hdr).hrecs, b"rg1");
            assert!(rg.is_some());
            assert_eq!(
                CStr::from_ptr((*rg.unwrap().as_ptr()).name.cast()).to_bytes(),
                b"rg1"
            );

            let dump = sam_hrecs_dump((*hdr).hrecs);
            assert!(dump.is_some());
            assert_eq!(dump.unwrap().as_slice(), text.to_bytes());

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hrecs_ref_altnames_populate_and_remove_ref_hash_entries() {
        unsafe {
            let text = c"@SQ\tSN:chr1\tLN:10\tAN:one,uno\n@SQ\tSN:chr2\tLN:20\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_fill_hrecs(&mut *hdr), 0);

            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"one"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"uno"), 0);

            let hrecs = (*hdr).hrecs;
            let an = sam_hrec_tag_value_cstr((*(*hrecs).ref_).ty.cast(), b"AN");
            sam_hrecs_remove_ref_altnames(hrecs, 0, an);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"one"), -1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"uno"), -1);

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_update_line_updates_hrec_tags_targets_and_aliases() {
        unsafe {
            let text =
                b"@HD\tVN:1.4\n@SQ\tSN:chr1\tLN:100\tAN:one\n@SQ\tSN:chr2\tLN:200\n@RG\tID:run1\n";
            let hdr = sam_hdr_parse(text.len(), text.as_ptr().cast());
            assert!(!hdr.is_null());

            assert_eq!(
                sam_hdr_update_line(
                    &mut *hdr,
                    b"SQ",
                    Some((b"SN", b"chr2")),
                    &[
                        (Some(b"LN" as &[u8]), Some(b"250" as &[u8])),
                        (Some(b"AN" as &[u8]), Some(b"two,dos" as &[u8]))
                    ],
                ),
                0
            );
            assert_eq!(
                sam_hdr_update_line(
                    &mut *hdr,
                    b"SQ",
                    Some((b"SN", b"chr1")),
                    &[(Some(b"SN" as &[u8]), Some(b"chrA" as &[u8]))],
                ),
                0
            );
            assert_eq!(
                sam_hdr_update_line(
                    &mut *hdr,
                    b"RG",
                    Some((b"ID", b"run1")),
                    &[(Some(b"DS" as &[u8]), Some(b"hello" as &[u8]))],
                ),
                0
            );

            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chrA"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"one"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr2"), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"two"), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"dos"), 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 1), 250);

            assert_eq!(
                sam_hdr_update_line(
                    &mut *hdr,
                    b"SQ",
                    Some((b"SN", b"chrA")),
                    &[(Some(b"SN" as &[u8]), Some(b"chr2" as &[u8]))],
                ),
                -1
            );
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.4\n@SQ\tSN:chrA\tLN:100\tAN:one\n@SQ\tSN:chr2\tLN:250\tAN:two,dos\n@RG\tID:run1\tDS:hello\n"
            );

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_link_pg_sets_prev_ids_and_chain_ends_for_hrecs() {
        unsafe {
            let text = c"@PG\tID:p1\n@PG\tID:p2\tPP:p1\n@PG\tID:p3\tPP:p2\n@PG\tID:solo\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_fill_hrecs(&mut *hdr), 0);
            let hrecs = (*hdr).hrecs;
            (*hrecs).pgs_changed = 1;

            assert_eq!(sam_hdr_link_pg(hdr), 0);
            assert_eq!((*hrecs).pgs_changed, 0);
            let pg = (*hrecs).pg.cast::<sam_hrec_pg_t>();
            assert_eq!((*pg.add(0)).prev_id, -1);
            assert_eq!((*pg.add(1)).prev_id, 0);
            assert_eq!((*pg.add(2)).prev_id, 1);
            assert_eq!((*hrecs).npg_end, 1);
            assert_eq!(*(*hrecs).pg_end, 2);

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hrecs_dup_and_remove_hash_entry_are_local() {
        unsafe {
            let hrecs = sam_hrecs_new();
            assert!(!hrecs.is_null());
            assert_eq!(
                sam_hrecs_parse_lines(
                    hrecs,
                    c"@SQ\tSN:chr1\tLN:10\n@RG\tID:rg1\n".as_ptr().cast(),
                    c"@SQ\tSN:chr1\tLN:10\n@RG\tID:rg1\n".to_bytes().len(),
                ),
                0
            );
            assert_eq!(sam_hrecs_update_hashes(hrecs), 0);

            let dup = sam_hrecs_dup(hrecs);
            assert!(!dup.is_null());
            assert!(sam_hrecs_find_rg(&mut *dup, b"rg1").is_some());
            assert_eq!(
                sam_hrecs_remove_hash_entry((*dup).rg_hash, c"rg1".as_ptr().cast()),
                1
            );
            assert!(sam_hrecs_find_rg(&mut *dup, b"rg1").is_none());
            assert_eq!(rebuild_hash(dup, header_h_58_TYPEKEY(c"RG".as_ptr().cast())), 0);
            assert!(sam_hrecs_find_rg(&mut *dup, b"rg1").is_some());

            sam_hrecs_free(dup);
            sam_hrecs_free(hrecs);
        }
    }

    #[test]
    fn sam_hrecs_refs_from_target_arrays_builds_stub_sq_lines() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_append_target(hdr, b"chr1", 25), 0);
            (*hdr).hrecs = sam_hrecs_new();
            assert!(!(*hdr).hrecs.is_null());
            assert_eq!(add_stub_ref_sq_lines(hdr), 0);
            assert_eq!((*(*hdr).hrecs).nref, 1);
            assert_eq!(
                CStr::from_ptr((*(*(*hdr).hrecs).ref_).name.cast()).to_bytes(),
                b"chr1"
            );
            assert_eq!(sam_hdr_rebuild(&mut *hdr), 0);
            assert_eq!(
                CStr::from_ptr((*hdr).text.cast()).to_bytes(),
                b"@SQ\tSN:chr1\tLN:25\n"
            );
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_count_lines_counts_text_backed_header_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\n@RG\tID:run1\n@PG\tID:prog1\n@CO\tfirst\n@CO\tsecond\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"HD"), 1);
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"SQ"), 2);
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 1);
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"PG"), 1);
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"CO"), 2);
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"XX"), 0);

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_count_lines_rejects_null_inputs() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b""), -1);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_find_line_and_tag_pos_read_text_backed_header_without_hrecs() {
        unsafe {
            let text =
                b"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\tM5:abc\n@CO\tfree text\n";
            let hdr = sam_hdr_parse(text.len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            let mut ks = kstring_t::default();
            assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"SQ", 1, &mut ks), 0);
            assert_eq!(
                ks.data.as_slice(),
                b"@SQ\tSN:ref2\tLN:20\tM5:abc"
            );

            assert_eq!(
                sam_hdr_find_tag_pos(&mut *hdr, b"SQ", 1, b"SN", &mut ks),
                0
            );
            assert_eq!(ks.data.as_slice(), b"ref2");
            assert_eq!(
                sam_hdr_find_tag_pos(&mut *hdr, b"SQ", 1, b"LN", &mut ks),
                0
            );
            assert_eq!(ks.data.as_slice(), b"20");
            assert_eq!(
                sam_hdr_find_tag_pos(&mut *hdr, b"SQ", 1, b"AS", &mut ks),
                -1
            );
            assert_eq!(sam_hdr_find_line_pos(&mut *hdr, b"RG", 0, &mut ks), -1);

            ks_free(&mut ks);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_find_line_and_tag_id_read_text_backed_header_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\tM5:abc\n@RG\tID:run1\tSM:sample1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            let mut ks = kstring_t::default();
            assert_eq!(
                sam_hdr_find_line_id(
                    &mut *hdr,
                    b"SQ",
                    b"SN",
                    b"ref2",
                    &mut ks
                ),
                0
            );
            assert_eq!(
                ks.data.as_slice(),
                b"@SQ\tSN:ref2\tLN:20\tM5:abc"
            );

            assert_eq!(
                sam_hdr_find_tag_id(
                    &mut *hdr,
                    b"SQ",
                    Some((b"SN", b"ref2")),
                    b"M5",
                    &mut ks
                ),
                0
            );
            assert_eq!(ks.data.as_slice(), b"abc");
            assert_eq!(
                sam_hdr_find_tag_id(
                    &mut *hdr,
                    b"RG",
                    Some((b"ID", b"run1")),
                    b"SM",
                    &mut ks
                ),
                0
            );
            assert_eq!(ks.data.as_slice(), b"sample1");
            assert_eq!(
                sam_hdr_find_line_id(
                    &mut *hdr,
                    b"SQ",
                    b"SN",
                    b"missing",
                    &mut ks
                ),
                -1
            );
            assert_eq!(
                sam_hdr_find_tag_id(
                    &mut *hdr,
                    b"SQ",
                    Some((b"SN", b"ref2")),
                    b"AS",
                    &mut ks
                ),
                -1
            );

            ks_free(&mut ks);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_line_name_reads_text_backed_indexed_line_names_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\n@RG\tID:run1\tSM:sample1\n@RG\tID:run2\n@PG\tID:prog1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(
                CStr::from_ptr(sam_hdr_line_name(&mut *hdr, b"SQ", 1).cast()).to_bytes(),
                b"ref2"
            );
            assert_eq!(
                CStr::from_ptr(sam_hdr_line_name(&mut *hdr, b"RG", 0).cast()).to_bytes(),
                b"run1"
            );
            assert_eq!(
                CStr::from_ptr(sam_hdr_line_name(&mut *hdr, b"PG", 0).cast()).to_bytes(),
                b"prog1"
            );
            assert!(sam_hdr_line_name(&mut *hdr, b"RG", 2).is_null());
            assert!(sam_hdr_line_name(&mut *hdr, b"CO", 0).is_null());

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_pg_id_generates_text_backed_unique_ids_without_hrecs() {
        unsafe {
            // Dropped the `@PG\tPN:missing-id\n` line: the hrecs path rejects
            // `@PG` records that lack the required `ID` tag during the
            // update_hashes pass (matching htslib's validation), so
            // sam_hdr_parse on the old fixture would return null. The unique-ID
            // generation logic exercised here is unaffected.
            let text = c"@HD\tVN:1.6\n@RG\tID:tool\n@PG\tID:tool\tPN:tool\n@PG\tID:tool.1\n@PG\tID:other\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(
                CStr::from_ptr(sam_hdr_pg_id(&mut *hdr, b"unused").cast()).to_bytes(),
                b"unused"
            );
            assert_eq!(
                CStr::from_ptr(sam_hdr_pg_id(&mut *hdr, b"tool").cast()).to_bytes(),
                b"tool.2"
            );
            // The hrecs path uses a single shared `ID_cnt` counter (not a
            // per-name counter), so the suffix continues to advance across
            // successive sam_hdr_pg_id calls. The previous expectation
            // (`other.1`) was specific to the text-mode generator, which
            // searched from .0 per name.
            assert_eq!(
                CStr::from_ptr(sam_hdr_pg_id(&mut *hdr, b"other").cast()).to_bytes(),
                b"other.3"
            );

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_pg_links_text_backed_chain_tips_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.5\n@PG\tID:prog1\tPN:prog1\n@PG\tID:prog2\tPN:prog2\tPP:prog1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(sam_hdr_add_pg(&mut *hdr, b"prog3", &[]), 0);
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog4",
                    &[(Some(b"PP" as &[u8]), Some(b"prog1" as &[u8]))]
                ),
                0
            );
            assert_eq!(sam_hdr_add_pg(&mut *hdr, b"prog6", &[]), 0);
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog7",
                    &[
                        (Some(b"ID" as &[u8]), Some(b"my_id" as &[u8])),
                        (Some(b"PP" as &[u8]), Some(b"prog6" as &[u8]))
                    ]
                ),
                0
            );
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog8",
                    &[(Some(b"ID" as &[u8]), Some(b"my_id" as &[u8]))]
                ),
                -1
            );
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog9",
                    &[(Some(b"PP" as &[u8]), Some(b"missing" as &[u8]))]
                ),
                -1
            );

            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.5\n@PG\tID:prog1\tPN:prog1\n@PG\tID:prog2\tPN:prog2\tPP:prog1\n@PG\tID:prog3\tPN:prog3\tPP:prog2\n@PG\tID:prog4\tPN:prog4\tPP:prog1\n@PG\tID:prog6\tPN:prog6\tPP:prog3\n@PG\tID:prog6.1\tPN:prog6\tPP:prog4\n@PG\tPN:prog7\tID:my_id\tPP:prog6\n"
            );

            sam_hdr_destroy(hdr);
        }
    }

    // The hrecs-backed @PG chain-linking path must match htslib exactly. This
    // reuses the C-verified scenario and expected output from
    // sam_hdr_add_pg_links_text_backed_chain_tips_without_hrecs, but forces the
    // header onto the hrecs path first. PG records append after existing PG
    // lines in both orderings, so the expected serialization is identical.
    #[test]
    fn sam_hdr_add_pg_chain_tips_with_hrecs_match_reference() {
        unsafe {
            let text = c"@HD\tVN:1.5\n@PG\tID:prog1\tPN:prog1\n@PG\tID:prog2\tPN:prog2\tPP:prog1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_fill_hrecs(&mut *hdr), 0);
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(sam_hdr_add_pg(&mut *hdr, b"prog3", &[]), 0);
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog4",
                    &[(Some(b"PP" as &[u8]), Some(b"prog1" as &[u8]))]
                ),
                0
            );
            assert_eq!(sam_hdr_add_pg(&mut *hdr, b"prog6", &[]), 0);
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog7",
                    &[
                        (Some(b"ID" as &[u8]), Some(b"my_id" as &[u8])),
                        (Some(b"PP" as &[u8]), Some(b"prog6" as &[u8]))
                    ]
                ),
                0
            );
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog8",
                    &[(Some(b"ID" as &[u8]), Some(b"my_id" as &[u8]))]
                ),
                -1
            );
            assert_eq!(
                sam_hdr_add_pg(
                    &mut *hdr,
                    b"prog9",
                    &[(Some(b"PP" as &[u8]), Some(b"missing" as &[u8]))]
                ),
                -1
            );

            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.5\n@PG\tID:prog1\tPN:prog1\n@PG\tID:prog2\tPN:prog2\tPP:prog1\n@PG\tID:prog3\tPN:prog3\tPP:prog2\n@PG\tID:prog4\tPN:prog4\tPP:prog1\n@PG\tID:prog6\tPN:prog6\tPP:prog3\n@PG\tID:prog6.1\tPN:prog6\tPP:prog4\n@PG\tPN:prog7\tID:my_id\tPP:prog6\n"
            );

            sam_hdr_destroy(hdr);
        }
    }

    // hrecs-backed sam_hdr_add_line: new lines are grouped by record type (as
    // in htslib), @HD is updated in place rather than duplicated, and @CO lines
    // are appended. Targets stay in sync after adding an @SQ.
    #[test]
    fn sam_hdr_add_line_with_hrecs_groups_by_type_and_updates_hd() {
        unsafe {
            let text = c"@HD\tVN:1.5\n@SQ\tSN:chr1\tLN:100\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_fill_hrecs(&mut *hdr), 0);
            assert!(!(*hdr).hrecs.is_null());

            // New @SQ is inserted after the existing @SQ, before any @RG.
            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"SQ",
                    &[
                        (Some(b"SN" as &[u8]), Some(b"chr2" as &[u8])),
                        (Some(b"LN" as &[u8]), Some(b"200" as &[u8]))
                    ]
                ),
                0
            );
            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"RG",
                    &[
                        (Some(b"ID" as &[u8]), Some(b"run1" as &[u8])),
                        (Some(b"SM" as &[u8]), Some(b"s1" as &[u8]))
                    ]
                ),
                0
            );
            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"CO",
                    &[(Some(b"a comment" as &[u8]), None)]
                ),
                0
            );
            // @HD already exists: this updates the existing line in place.
            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"HD",
                    &[(Some(b"SO" as &[u8]), Some(b"coordinate" as &[u8]))]
                ),
                0
            );

            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.5\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n@RG\tID:run1\tSM:s1\n@CO\ta comment\n"
            );

            // Targets reflect both @SQ lines.
            assert_eq!(sam_hdr_nref(&*hdr), 2);
            assert_eq!(sam_hdr_tid2len(&*hdr, 1), 200);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 1).cast()).to_bytes(), b"chr2");

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_pg_handles_text_backed_pg_loops_without_hrecs() {
        unsafe {
            // Self-loop @PG: the hrecs path's sam_hrecs_link_pg classifies a
            // self-referential @PG as a non-leaf (npg_end == 0), so the new
            // record is appended without a PP. The previous expectation
            // (PP:loop1) was specific to our retired text-mode leaf scan.
            let self_loop = c"@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop1\n";
            let hdr = sam_hdr_parse(self_loop.to_bytes().len(), self_loop.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_add_pg(&mut *hdr, b"new_prog", &[]), 0);
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop1\n@PG\tID:new_prog\tPN:new_prog\n"
            );
            sam_hdr_destroy(hdr);

            let two_node_loop = c"@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop2\n@PG\tID:loop2\tPN:prog2\tPP:loop1\n";
            let hdr = sam_hdr_parse(two_node_loop.to_bytes().len(), two_node_loop.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_add_pg(&mut *hdr, b"new_prog", &[]), 0);
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.5\n@PG\tID:loop1\tPN:prog1\tPP:loop2\n@PG\tID:loop2\tPN:prog2\tPP:loop1\n@PG\tID:new_prog\tPN:new_prog\n"
            );
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_remove_line_id_and_pos_update_text_backed_header_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\n@SQ\tSN:ref3\tLN:30\n@RG\tID:run1\tSM:sample1\n@RG\tID:run2\n@PG\tID:prog1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());
            assert_eq!(sam_hdr_nref(&*hdr), 3);

            assert_eq!(
                sam_hdr_remove_line_id(&mut *hdr, b"RG", Some((b"ID", b"missing"))),
                0
            );
            assert_eq!(
                sam_hdr_remove_line_id(&mut *hdr, b"RG", Some((b"ID", b"run1"))),
                0
            );
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 1);
            assert_eq!(
                CStr::from_ptr(sam_hdr_line_name(&mut *hdr, b"RG", 0).cast()).to_bytes(),
                b"run2"
            );

            assert_eq!(sam_hdr_remove_line_pos(&mut *hdr, b"SQ", 1), 0);
            assert_eq!(sam_hdr_nref(&*hdr), 2);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref2"), -1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref3"), 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 1), 30);
            assert_eq!(sam_hdr_remove_line_pos(&mut *hdr, b"SQ", 9), -1);
            assert_eq!(
                sam_hdr_remove_line_id(&mut *hdr, b"PG", Some((b"ID", b"prog1"))),
                -1
            );
            assert_eq!(sam_hdr_remove_line_pos(&mut *hdr, b"PG", 0), -1);

            let header_text =
                std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), sam_hdr_length(&mut *hdr));
            assert_eq!(
                header_text,
                b"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref3\tLN:30\n@RG\tID:run2\n@PG\tID:prog1\n"
            );

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_remove_except_updates_text_backed_header_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\n@SQ\tSN:ref3\tLN:30\n@RG\tID:run1\tSM:sample1\n@RG\tID:run2\n@RG\tID:run3\n@PG\tID:prog1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(
                sam_hdr_remove_except(&mut *hdr, b"RG", Some((b"ID", b"run2"))),
                0
            );
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 1);
            assert_eq!(
                CStr::from_ptr(sam_hdr_line_name(&mut *hdr, b"RG", 0).cast()).to_bytes(),
                b"run2"
            );
            assert_eq!(
                sam_hdr_remove_except(&mut *hdr, b"RG", Some((b"ID", b"missing"))),
                0
            );
            assert_eq!(sam_hdr_count_lines(&mut *hdr, b"RG"), 0);
            assert_eq!(
                sam_hdr_remove_except(&mut *hdr, b"PG", Some((b"ID", b"prog1"))),
                -1
            );

            assert_eq!(
                sam_hdr_remove_except(&mut *hdr, b"SQ", Some((b"SN", b"ref2"))),
                0
            );
            assert_eq!(sam_hdr_nref(&*hdr), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref1"), -1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref2"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref3"), -1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 20);

            let header_text =
                std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), sam_hdr_length(&mut *hdr));
            assert_eq!(
                header_text,
                b"@HD\tVN:1.6\n@SQ\tSN:ref2\tLN:20\n@PG\tID:prog1\n"
            );

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_remove_tag_id_updates_text_backed_header_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\tAN:chr1,one\n@RG\tID:run1\tSM:sample1\tLB:lib1\n@RG\tID:run2\tSM:sample2\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);

            assert_eq!(
                sam_hdr_remove_tag_id(&mut *hdr, b"RG", Some((b"ID", b"run1")), b"SM"),
                0
            );
            let mut ks = kstring_t::default();
            assert_eq!(
                sam_hdr_find_tag_id(&mut *hdr, b"RG", Some((b"ID", b"run1")), b"SM", &mut ks),
                -1
            );
            assert_eq!(
                sam_hdr_find_tag_id(&mut *hdr, b"RG", Some((b"ID", b"run1")), b"LB", &mut ks),
                0
            );
            assert_eq!(ks.data.as_slice(), b"lib1");
            assert_eq!(
                sam_hdr_remove_tag_id(&mut *hdr, b"SQ", Some((b"SN", b"ref1")), b"AN"),
                0
            );
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"ref1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), -1);
            assert_eq!(
                sam_hdr_remove_tag_id(&mut *hdr, b"RG", Some((b"ID", b"run2")), b"LB"),
                -1
            );

            let header_text =
                std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), sam_hdr_length(&mut *hdr));
            assert_eq!(
                header_text,
                b"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@RG\tID:run1\tLB:lib1\n@RG\tID:run2\tSM:sample2\n"
            );

            ks_free(&mut ks);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn nibble2base_decodes_packed_bam_sequence() {
        unsafe {
            let packed = [0x12, 0x48, 0xf3, 0x50];
            let mut seq = [0 as u8; 9];

            nibble2base_default(&packed, &mut seq[..7]);
            assert_eq!(CStr::from_ptr(seq.as_ptr().cast()).to_bytes(), b"ACGTNMR");

            seq.fill(0);
            nibble2base(&packed, &mut seq[..8]);
            assert_eq!(CStr::from_ptr(seq.as_ptr().cast()).to_bytes(), b"ACGTNMR=");

            seq.fill(0);
            nibble2base_default(&packed, &mut seq[..7]);
            assert_eq!(CStr::from_ptr(seq.as_ptr().cast()).to_bytes(), b"ACGTNMR");
        }
    }

    #[test]
    fn sam_open_mode_helpers_match_extension_and_option_rules() {
        unsafe {
            let mut mode = [0 as u8; 8];

            assert_eq!(
                sam_open_mode(mode.as_mut_ptr(), c"reads.bam".as_ptr().cast(), std::ptr::null()),
                0
            );
            assert_eq!(CStr::from_ptr(mode.as_ptr().cast()).to_bytes(), b"b");

            assert_eq!(
                sam_open_mode(mode.as_mut_ptr(), c"out".as_ptr().cast(), c"FASTQ.GZ".as_ptr().cast()),
                0
            );
            assert_eq!(CStr::from_ptr(mode.as_ptr().cast()).to_bytes(), b"fz");

            assert_eq!(
                sam_open_mode(mode.as_mut_ptr(), c"out".as_ptr().cast(), c"unknown".as_ptr().cast()),
                -1
            );

            let opts = sam_open_mode_opts(
                c"out.cram".as_ptr().cast(),
                c"w".as_ptr().cast(),
                c"cram3,seqs_per_slice=10".as_ptr().cast(),
            );
            assert!(!opts.is_null());
            assert_eq!(
                CStr::from_ptr(opts.cast()).to_bytes(),
                b"wc,VERSION=3.0,seqs_per_slice=10"
            );
            {
                let mut l = 0usize;
                while *opts.add(l) != 0 {
                    l += 1;
                }
                drop(Vec::from_raw_parts(opts, l + 1, l + 1));
            }

            let opts =
                sam_open_mode_opts(c"reads.sam.gz".as_ptr().cast(), std::ptr::null(), std::ptr::null());
            assert!(!opts.is_null());
            assert_eq!(CStr::from_ptr(opts.cast()).to_bytes(), b"rz");
            {
                let mut l = 0usize;
                while *opts.add(l) != 0 {
                    l += 1;
                }
                drop(Vec::from_raw_parts(opts, l + 1, l + 1));
            }

            assert!(
                sam_open_mode_opts(c"reads.bin".as_ptr().cast(), c"r".as_ptr().cast(), std::ptr::null())
                    .is_null()
            );
        }
    }

    #[test]
    fn base_mod_state_queries_and_seq_freq_match_c_rules() {
        unsafe {
            let mut state = hts_base_mod_state_new();
            assert_eq!(state.nmods, 0);

            state.nmods = 2;
            state.type_[0] = b'm' as i32;
            state.type_[1] = -1234;
            state.strand[0] = b'+' as u8;
            state.strand[1] = b'-' as u8;
            state.implicit[0] = 1;
            state.implicit[1] = 0;
            state.canonical[0] = 2;
            state.canonical[1] = 15;

            let mut ntype = 0;
            let types = bam_mods_recorded(&mut state, &mut ntype);
            assert_eq!(ntype, 2);
            assert_eq!(types[0], b'm' as i32);
            assert_eq!(types[1], -1234);

            let mut strand = 0;
            let mut implicit = 0;
            let mut canonical = 0;
            assert_eq!(
                bam_mods_query_type(
                    &state,
                    b'm' as i32,
                    Some(&mut strand),
                    Some(&mut implicit),
                    Some(&mut canonical),
                ),
                0
            );
            assert_eq!(strand, b'+' as i32);
            assert_eq!(implicit, 1);
            assert_eq!(canonical, b'C' as u8);

            assert_eq!(
                bam_mods_queryi(
                    &state,
                    1,
                    Some(&mut strand),
                    Some(&mut implicit),
                    Some(&mut canonical),
                ),
                0
            );
            assert_eq!(strand, b'-' as i32);
            assert_eq!(implicit, 0);
            assert_eq!(canonical, b'N' as u8);
            assert_eq!(
                bam_mods_query_type(&state, b'h' as i32, None, None, None,),
                -1
            );
            assert_eq!(bam_mods_queryi(&state, 2, None, None, None), -1);

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"freq".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    5,
                    c"ACGTN".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ),
                20
            );
            let mut freq = [99; 16];
            seq_freq(b.as_ref().unwrap(), &mut freq);
            assert_eq!(freq[1], 1);
            assert_eq!(freq[2], 1);
            assert_eq!(freq[4], 1);
            assert_eq!(freq[8], 1);
            assert_eq!(freq[15], 5);
            assert_eq!(freq[3], 0);
            bam_destroy1(b);
        }
    }

    #[test]
    fn base_mod_iterators_report_next_and_qpos_modifications() {
        unsafe {
            let b = bam_init1();
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"mods".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    c"ACAN".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let mut state = hts_base_mod_state_new();
            let mut mm_end = *b";\0";
            let mut ml = [42u8];
            state.nmods = 1;
            state.type_[0] = b'm' as i32;
            state.canonical[0] = 1;
            state.strand[0] = 0;
            state.mmcount[0] = 1;
            state.mm[0] = mm_end.as_mut_ptr().cast();
            state.ml[0] = ml.as_mut_ptr();
            state.mlstride[0] = 1;
            state.implicit[0] = 1;

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(b.as_ref().unwrap(), &mut state, &mut mods, &mut pos),
                1
            );
            assert_eq!(pos, 2);
            assert_eq!(mods[0].modified_base, b'm' as i32);
            assert_eq!(mods[0].canonical_base, b'A' as i32);
            assert_eq!(mods[0].strand, 0);
            assert_eq!(mods[0].qual, 42);
            assert_eq!(
                bam_next_basemod(b.as_ref().unwrap(), &mut state, &mut mods, &mut pos),
                0
            );

            state.seq_pos = 0;
            state.mmcount[0] = 1;
            state.mm[0] = mm_end.as_mut_ptr().cast();
            state.ml[0] = ml.as_mut_ptr();
            assert_eq!(
                bam_mods_at_qpos(b.as_ref().unwrap(), 2, &mut state, &mut mods),
                1
            );
            assert_eq!(mods[0].qual, 42);

            state.seq_pos = 0;
            state.mmcount[0] = 1;
            state.mm[0] = mm_end.as_mut_ptr().cast();
            state.ml[0] = std::ptr::null_mut();
            state.implicit[0] = 0;
            state.flags = HTS_MOD_REPORT_UNCHECKED;
            assert_eq!(
                bam_mods_at_next_pos(b.as_ref().unwrap(), &mut state, &mut mods),
                1
            );
            assert_eq!(mods[0].qual, HTS_MOD_UNCHECKED);

            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_parse_basemod_populates_state_from_mm_ml_tags() {
        unsafe {
            let b = bam_init1();
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"bmod".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    c"ACAN".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mm = b"A+m,1;\0";
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr().cast(),
                    b'Z' as u8,
                    mm.len() as i32,
                    mm.as_ptr(),
                ),
                0
            );
            let ml = [b'C', 1, 0, 0, 0, 37];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"ML".as_ptr().cast(),
                    b'B' as u8,
                    ml.len() as i32,
                    ml.as_ptr(),
                ),
                0
            );

            let mut state = hts_base_mod_state_new();
            assert_eq!(
                bam_parse_basemod2(b.as_ref().unwrap(), &mut state, 0),
                0
            );
            assert_eq!(state.nmods, 1);
            assert_eq!(state.type_[0], b'm' as i32);
            assert_eq!(state.canonical[0], 1);
            assert_eq!(state.mmcount[0], 1);
            assert_eq!(state.mlstride[0], 1);
            assert_eq!(*state.ml[0], 37);

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(b.as_ref().unwrap(), &mut state, &mut mods, &mut pos),
                1
            );
            assert_eq!(pos, 2);
            assert_eq!(mods[0].modified_base, b'm' as i32);
            assert_eq!(mods[0].qual, 37);

            bam_destroy1(b);

            let b = bam_init1();
            assert!(
                bam_set1(
                    b,
                    0,
                    std::ptr::null(),
                    BAM_FUNMAP as u16,
                    -1,
                    -1,
                    0,
                    0,
                    std::ptr::null(),
                    -1,
                    -1,
                    0,
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mut state = hts_base_mod_state_new();
            assert_eq!(
                bam_parse_basemod2(b.as_ref().unwrap(), &mut state, 0),
                0
            );
            assert_eq!(state.nmods, 0);
            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_parse_basemod_defers_forward_runover_errors_until_iteration() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"bmov".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    1,
                    c"C".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mm = b"C+m,1;\0";
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr().cast(),
                    b'Z' as u8,
                    mm.len() as i32,
                    mm.as_ptr(),
                ),
                0
            );

            let mut state = hts_base_mod_state_new();
            assert_eq!(
                bam_parse_basemod2(b.as_ref().unwrap(), &mut state, 0),
                0
            );
            assert_eq!(state.nmods, 1);
            assert_eq!(state.mmcount[0], 1);

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(b.as_ref().unwrap(), &mut state, &mut mods, &mut pos),
                -1
            );

            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_parse_basemod_rejects_256_mod_codes_like_htslib() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"bm256".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    1,
                    c"C".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let mut mm = Vec::with_capacity(5 + MAX_BASE_MOD + 1);
            mm.extend_from_slice(b"C+");
            mm.extend(std::iter::repeat_n(b'm', MAX_BASE_MOD));
            mm.extend_from_slice(b",0;\0");
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr().cast(),
                    b'Z' as u8,
                    mm.len() as i32,
                    mm.as_ptr(),
                ),
                0
            );

            let mut state = hts_base_mod_state_new();
            assert_eq!(
                bam_parse_basemod2(b.as_ref().unwrap(), &mut state, 0),
                -1
            );

            bam_destroy1(b);
        }
    }

    #[test]
    fn aux_to_le_copies_and_validates_aux_payloads() {
        unsafe {
            let input = [0x34, 0x12, 0x78, 0x56];
            let mut out = [0; 8];
            assert_eq!(
                aux_to_le(
                    b'S' as u8,
                    out.as_mut_ptr(),
                    input.as_ptr(),
                    input.len()
                ),
                0
            );
            assert_eq!(&out[..4], &[0x34, 0x12, 0x78, 0x56]);

            assert_eq!(
                aux_to_le(b'S' as u8, out.as_mut_ptr(), input.as_ptr(), 3),
                -1
            );

            let z = b"text\0";
            out.fill(0);
            assert_eq!(
                aux_to_le(b'Z' as u8, out.as_mut_ptr(), z.as_ptr(), z.len()),
                0
            );
            assert_eq!(&out[..z.len()], z);

            let b_array = [b's', 2, 0, 0, 0, 0x34, 0x12, 0x78, 0x56];
            let mut bout = [0; 9];
            assert_eq!(
                aux_to_le(
                    b'B' as u8,
                    bout.as_mut_ptr(),
                    b_array.as_ptr(),
                    b_array.len(),
                ),
                0
            );
            assert_eq!(bout, b_array);

            assert_eq!(
                aux_to_le(b'B' as u8, bout.as_mut_ptr(), b_array.as_ptr(), 4),
                -1
            );
            assert_eq!(
                aux_to_le(b'?' as u8, bout.as_mut_ptr(), b_array.as_ptr(), 1),
                -1
            );
        }
    }

    #[test]
    fn cigar_parsers_fill_arrays_and_update_bam_records() {
        unsafe {
            assert_eq!(read_ncigar(c"10M2I3D\t".as_ptr().cast()), 3);
            assert_eq!(read_ncigar(c"123\t".as_ptr().cast()), 0);

            let mut cigar = [0u32; 4];
            assert_eq!(parse_cigar(c"10M2I3D".as_ptr().cast(), cigar.as_mut_ptr(), 3), 7);
            assert_eq!(cigar[0], (10 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32);
            assert_eq!(cigar[1], (2 << BAM_CIGAR_SHIFT) | BAM_CINS as u32);
            assert_eq!(cigar[2], (3 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32);
            assert_eq!(parse_cigar(c"10Q".as_ptr().cast(), cigar.as_mut_ptr(), 1), 0);

            let mut end: *mut u8 = std::ptr::null_mut();
            let mut a_cigar: *mut u32 = std::ptr::null_mut();
            let mut a_mem = 0usize;
            let input = b"4M1S\t";
            assert_eq!(
                sam_parse_cigar(input.as_ptr(), &mut end, &mut a_cigar, &mut a_mem,),
                2
            );
            assert!(a_mem >= 2);
            assert_eq!(end.offset_from(input.as_ptr()), 4);
            assert_eq!(*a_cigar.add(0), (4 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32);
            assert_eq!(
                *a_cigar.add(1),
                (1 << BAM_CIGAR_SHIFT) | BAM_CSOFT_CLIP as u32
            );
            if !a_cigar.is_null() {
                drop(Vec::from_raw_parts(a_cigar, a_mem, a_mem));
            }

            let star = b"*";
            assert_eq!(
                sam_parse_cigar(
                    star.as_ptr(),
                    &mut end,
                    std::ptr::addr_of_mut!(a_cigar),
                    &mut a_mem,
                ),
                0
            );
            assert_eq!(end.offset_from(star.as_ptr()), 1);

            let b = bam_init1();
            let cigar1 = [(5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"cigr".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar1.as_ptr(),
                    -1,
                    -1,
                    0,
                    5,
                    c"ACGTN".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let cigar_text = b"2M1I2M\t";
            assert_eq!(bam_parse_cigar(cigar_text.as_ptr(), &mut end, b), 3);
            assert_eq!(end.offset_from(cigar_text.as_ptr()), 6);
            assert_eq!((*b).core.n_cigar, 3);
            let parsed = bam_get_cigar(b);
            assert_eq!(*parsed.add(0), (2 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32);
            assert_eq!(*parsed.add(1), (1 << BAM_CIGAR_SHIFT) | BAM_CINS as u32);
            assert_eq!(*parsed.add(2), (2 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32);
            assert_eq!(bam_seqi(bam_get_seq(b), 4), 15);

            assert_eq!(bam_parse_cigar(star.as_ptr(), &mut end, b), 0);
            assert_eq!(end.offset_from(star.as_ptr()), 1);
            assert_eq!((*b).core.n_cigar, 0);
            assert_eq!(bam_seqi(bam_get_seq(b), 0), 1);
            bam_destroy1(b);
        }
    }

    #[test]
    fn realn_mapq_cap_and_tag_checks_match_c_rules() {
        let mut data = vec![0u8; 4 + 4 + 2 + 4];
        data[0..4].copy_from_slice(b"r1\0\0");
        unsafe {
            data.as_mut_ptr()
                .add(4)
                .cast::<u32>()
                .write_unaligned((4u32 << 4) | BAM_CMATCH as u32);
        }
        data[8] = (1 << 4) | 2;
        data[9] = (4 << 4) | 8;
        data[10..14].copy_from_slice(&[30, 30, 30, 30]);
        let mut b = bam1_t {
            core: bam1_core_t {
                pos: 0,
                tid: 0,
                bin: 0,
                qual: 0,
                l_extranul: 0,
                flag: 0,
                l_qname: 4,
                n_cigar: 1,
                l_qseq: 4,
                mtid: -1,
                mpos: -1,
                isize: 0,
            },
            id: 0,
            data,
            mempolicy_and_reserved: 0,
        };

        unsafe {
            use crate::htslib_rs::realn::sam_cap_mapq;
            let ref_match = b"ACGT";
            assert_eq!(sam_cap_mapq(&mut b, ref_match, -1), 40);

            let ref_mismatch = b"ATGT";
            assert_eq!(sam_cap_mapq(&mut b, ref_mismatch, 40), 28);
            assert_eq!(sam_cap_mapq(&mut b, ref_mismatch, 10), -1);

            let good_tag = b"Zabcd\0";
            assert_eq!(
                realn_check_tag(good_tag.as_ptr(), 3, c"BQ".as_ptr().cast(), &b as *const bam1_t),
                0
            );
            let bad_type = b"iabcd\0";
            assert_eq!(
                realn_check_tag(bad_type.as_ptr(), 3, c"BQ".as_ptr().cast(), &b as *const bam1_t),
                -1
            );
            let bad_len = b"Zabc\0";
            assert_eq!(
                realn_check_tag(bad_len.as_ptr(), 3, c"BQ".as_ptr().cast(), &b as *const bam1_t),
                -1
            );
        }
    }

    #[test]
    fn sam_prob_realn_adds_baq_tag_for_simple_match() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let qual = [30 as u8, 30 as u8, 30 as u8, 30 as u8];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"read".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    c"ACGT".as_ptr().cast(),
                    qual.as_ptr(),
                    0,
                ),
                18
            );

            assert_eq!(sam_prob_realn(b, c"ACGT".as_ptr().cast(), 4, 0), 0);
            let bq = bam_aux_get(b, c"BQ".as_ptr().cast());
            assert!(!bq.is_null());
            assert_eq!(*bq, b'Z');
            assert_eq!(libc::strlen(bq.add(1).cast()), 4);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_prob_realn_trims_reference_window_like_htslib_for_deletion_heavy_reads() {
        unsafe {
            let ref_seq = b"TTGCAACGTACGTTACGATCGTACCTAGGCTAATCGGATCCGTAACGTTAGCTA";
            let cigar = [
                (5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (10u32 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32,
                (5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
            ];
            let qual = [35 as u8; 10];

            let rust_b = bam_init1();
            assert!(!rust_b.is_null());
            assert!(
                bam_set1(
                    rust_b,
                    6,
                    c"delrw".as_ptr().cast(),
                    0,
                    0,
                    20,
                    60,
                    cigar.len(),
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    10,
                    c"ACGTACGTAA".as_ptr().cast(),
                    qual.as_ptr(),
                    0,
                ) > 0
            );

            // The native bam1_t is no longer ABI-compatible with hts_sys, so the
            // C comparison record must be built through the C API on a
            // C-allocated record (a `.cast()` of a Rust bam1_t would be unsound).
            let c_b = hts_sys::bam_init1();
            assert!(!c_b.is_null());
            assert!(
                hts_sys::bam_set1(
                    c_b,
                    6,
                    c"delrw".as_ptr(),
                    0,
                    0,
                    20,
                    60,
                    cigar.len(),
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    10,
                    c"ACGTACGTAA".as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ) > 0
            );

            let rust_ret = sam_prob_realn(rust_b, ref_seq.as_ptr(), 60, 0);
            let c_ret = hts_sys::sam_prob_realn(c_b, ref_seq.as_ptr().cast(), 60, 0);
            assert_eq!(rust_ret, c_ret);
            assert_eq!((*rust_b).data.len() as i32, (*c_b).l_data);
            assert_eq!(
                (*rust_b).data.as_slice(),
                std::slice::from_raw_parts((*c_b).data, (*c_b).l_data as usize)
            );

            bam_destroy1(rust_b);
            hts_sys::bam_destroy1(c_b);
        }
    }

    #[test]
    fn bam_data_accessors_match_htslib_macros() {
        let mut data = vec![0u8; 4 + 8 + 3 + 5 + 7];
        let cigar_offset = 4usize;
        let seq_offset = cigar_offset + 8;
        let qual_offset = seq_offset + 3;
        let aux_offset = qual_offset + 5;
        data[cigar_offset..cigar_offset + 4].copy_from_slice(&(10u32 << 4).to_ne_bytes());
        data[cigar_offset + 4..cigar_offset + 8].copy_from_slice(&((1u32 << 4) | 2).to_ne_bytes());
        data[seq_offset..seq_offset + 3].copy_from_slice(&[0x12, 0x48, 0xf0]);
        data[qual_offset..qual_offset + 5].copy_from_slice(&[30, 31, 32, 33, 34]);
        data[aux_offset..aux_offset + 7].copy_from_slice(b"NMi\x05\0\0\0");

        let mut b = bam1_t {
            core: bam1_core_t {
                pos: 100,
                tid: 0,
                bin: 0,
                qual: 60,
                l_extranul: 0,
                flag: 0,
                l_qname: 4,
                n_cigar: 2,
                l_qseq: 5,
                mtid: -1,
                mpos: -1,
                isize: 0,
            },
            id: 0,
            data,
            mempolicy_and_reserved: 0,
        };

        unsafe {
            let dptr = b.data.as_ptr();
            assert_eq!(bam_get_qname(&b), dptr.cast::<u8>().cast_mut());
            assert!(!bam_is_rev(&b));
            assert!(!bam_is_mrev(&b));
            assert_eq!(
                bam_get_cigar(&b).cast::<u8>(),
                dptr.add(cigar_offset)
            );
            assert_eq!(bam_get_seq(&b), dptr.add(seq_offset));
            assert_eq!(bam_get_qual(&b), dptr.add(qual_offset));
            assert_eq!(bam_get_aux(&b), dptr.add(aux_offset));
            assert_eq!(bam_get_l_aux(&b), 7);
            let nm = bam_aux_get(&b, b"NM".as_ptr().cast());
            assert_eq!(nm, dptr.add(aux_offset + 2).cast_mut());
            assert_eq!(bam_aux_first(&b), nm);
            assert!(bam_aux_next(&b, nm).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::ENOENT as i32
            );
            assert_eq!(
                bam_aux_tag(bam_get_aux(&b).add(2)),
                dptr.add(aux_offset).cast::<u8>()
            );
            assert_eq!(bam_aux_type(nm), b'i' as u8);
            assert_eq!(bam_aux2i(nm), 5);
            assert_eq!(bam_aux2f(nm), 5.0);
            assert_eq!(bam_aux2A(nm), 0);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );
            assert_eq!(*bam_get_cigar(&b), 10u32 << 4);
            assert_eq!(*bam_get_cigar(&b).add(1), (1u32 << 4) | 2);
            assert_eq!(bam_seqi(bam_get_seq(&b), 0), 1);
            assert_eq!(bam_seqi(bam_get_seq(&b), 1), 2);
            assert_eq!(bam_seqi(bam_get_seq(&b), 2), 4);
            assert_eq!(bam_seqi(bam_get_seq(&b), 3), 8);
            assert_eq!(bam_seqi(bam_get_seq(&b), 4), 15);

            b.core.flag = (BAM_FREVERSE | BAM_FMREVERSE) as u16;
            assert!(bam_is_rev(&b));
            assert!(bam_is_mrev(&b));

            let seq = bam_get_seq(&b) as *mut u8;
            bam_set_seqi(seq, 1, 4);
            bam_set_seqi(seq, 4, 2);
            assert_eq!(bam_seqi(seq, 0), 1);
            assert_eq!(bam_seqi(seq, 1), 4);
            assert_eq!(bam_seqi(seq, 4), 2);
        }
    }

    #[test]
    fn subtract_check_underflow_matches_c_rules() {
        unsafe {
            let mut limit = 10usize;
            assert_eq!(subtract_check_underflow(4, &mut limit), 0);
            assert_eq!(limit, 6);
            assert_eq!(subtract_check_underflow(6, &mut limit), 0);
            assert_eq!(limit, 0);
            assert_eq!(subtract_check_underflow(1, &mut limit), -1);
            assert_eq!(limit, 0);
        }
    }

    #[test]
    fn bam_set1_builds_bam_data_layout_and_validates_inputs() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGTN").unwrap();
            let qual = [10u8, 20, 30, 40, 50];
            let qname = CString::new("read1").unwrap();
            let ret = bam_set1(
                b,
                5,
                qname.as_ptr().cast(),
                0,
                2,
                100,
                60,
                cigar.len(),
                cigar.as_ptr(),
                3,
                200,
                -7,
                5,
                seq.as_ptr().cast(),
                qual.as_ptr().cast(),
                0,
            );
            assert_eq!(ret, 5 + 3 + 4 + 3 + 5);
            assert_eq!((*b).data.len() as i32, ret);
            assert_eq!((*b).core.pos, 100);
            assert_eq!((*b).core.tid, 2);
            assert_eq!((*b).core.bin, hts_reg2bin(100, 105, 14, 5) as u16);
            assert_eq!((*b).core.qual, 60);
            assert_eq!((*b).core.l_extranul, 2);
            assert_eq!((*b).core.l_qname, 8);
            assert_eq!((*b).core.n_cigar, 1);
            assert_eq!((*b).core.l_qseq, 5);
            assert_eq!((*b).core.mtid, 3);
            assert_eq!((*b).core.mpos, 200);
            assert_eq!((*b).core.isize, -7);
            let data = &(*b).data;
            assert_eq!(&data[..8], b"read1\0\0\0");
            assert_eq!(*bam_get_cigar(b), cigar[0]);
            let packed = bam_get_seq(b);
            assert_eq!(*packed, 0x12);
            assert_eq!(*packed.add(1), 0x48);
            assert_eq!(*packed.add(2), 0xf0);
            assert_eq!(
                std::slice::from_raw_parts(bam_get_qual(b), 5),
                &[10, 20, 30, 40, 50]
            );

            let ret = bam_set1(
                b,
                0,
                std::ptr::null(),
                BAM_FUNMAP as u16,
                -1,
                -1,
                0,
                0,
                std::ptr::null(),
                -1,
                -1,
                0,
                3,
                CString::new("AAA").unwrap().as_ptr().cast(),
                std::ptr::null(),
                0,
            );
            assert_eq!(ret, 9);
            let data = &(*b).data;
            assert_eq!(&data[..4], b"*\0\0\0");
            assert_eq!(std::slice::from_raw_parts(bam_get_qual(b), 3), &[0xff; 3]);

            let generic = bam_init1();
            let fastq = bam_init1();
            assert!(!generic.is_null());
            assert!(!fastq.is_null());
            let qname = CString::new("read/1").unwrap();
            let seq = CString::new("ACGTNAC").unwrap();
            let qual = [0u8, 1, 2, 30, 31, 32, 40];
            assert_eq!(
                bam_set1(
                    generic,
                    6,
                    qname.as_ptr().cast(),
                    BAM_FUNMAP as u16,
                    -1,
                    -1,
                    0,
                    0,
                    std::ptr::null(),
                    -1,
                    -1,
                    0,
                    qual.len(),
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                bam_set1_fastq_unmapped(
                    fastq,
                    6,
                    qname.as_ptr().cast(),
                    BAM_FUNMAP as u16,
                    qual.len(),
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                )
            );
            assert_eq!((*fastq).core.pos, (*generic).core.pos);
            assert_eq!((*fastq).core.tid, (*generic).core.tid);
            assert_eq!((*fastq).core.bin, (*generic).core.bin);
            assert_eq!((*fastq).core.qual, (*generic).core.qual);
            assert_eq!((*fastq).core.l_extranul, (*generic).core.l_extranul);
            assert_eq!((*fastq).core.flag, (*generic).core.flag);
            assert_eq!((*fastq).core.l_qname, (*generic).core.l_qname);
            assert_eq!((*fastq).core.n_cigar, (*generic).core.n_cigar);
            assert_eq!((*fastq).core.l_qseq, (*generic).core.l_qseq);
            assert_eq!((*fastq).core.mtid, (*generic).core.mtid);
            assert_eq!((*fastq).core.mpos, (*generic).core.mpos);
            assert_eq!((*fastq).core.isize, (*generic).core.isize);
            assert_eq!((*fastq).data.len(), (*generic).data.len());
            assert_eq!((*fastq).data.as_slice(), (*generic).data.as_slice());
            bam_destroy1(generic);
            bam_destroy1(fastq);

            let too_long = vec![b'x'; 255];
            let bad = bam_set1(
                b,
                too_long.len(),
                too_long.as_ptr().cast(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null(),
                -1,
                -1,
                0,
                0,
                std::ptr::null(),
                std::ptr::null(),
                0,
            );
            assert_eq!(bad, -1);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            let bad = bam_set1(
                b,
                1,
                c"r".as_ptr().cast(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null(),
                -1,
                -1,
                0,
                1,
                c"A".as_ptr().cast(),
                std::ptr::null(),
                0,
            );
            assert_eq!(bad, -1);
            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_set_qname_rewrites_name_and_preserves_record_payload() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [11u8, 12, 13, 14];
            assert_eq!(
                bam_set1(
                    b,
                    2,
                    c"r1".as_ptr().cast(),
                    0,
                    0,
                    2,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                14
            );
            let old_cigar = *bam_get_cigar(b);
            let old_seq0 = bam_seqi(bam_get_seq(b), 0);

            assert_eq!(bam_set_qname(b, c"longer_name".as_ptr().cast()), 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), b"longer_name");
            assert_eq!((*b).core.l_qname as usize % 4, 0);
            assert_eq!((*b).core.l_extranul, 0);
            assert_eq!(*bam_get_cigar(b), old_cigar);
            assert_eq!(bam_seqi(bam_get_seq(b), 0), old_seq0);
            assert_eq!(*bam_get_qual(b), 11);

            assert_eq!(bam_set_qname(b, c"x".as_ptr().cast()), 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), b"x");
            assert_eq!((*b).core.l_qname, 4);
            assert_eq!((*b).core.l_extranul, 2);
            assert_eq!(*bam_get_cigar(b), old_cigar);
            assert_eq!(bam_seqi(bam_get_seq(b), 0), old_seq0);

            assert_eq!(bam_set_qname(b, std::ptr::null()), -1);
            assert_eq!(bam_set_qname(std::ptr::null_mut(), c"x".as_ptr().cast()), -1);
            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_write1_round_trips_record_through_translated_reader() {
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-bam-write1-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_c = CString::new(path.to_str().unwrap()).unwrap();

        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [31u8, 32, 33, 34];
            let ret = bam_set1(
                b,
                4,
                c"read".as_ptr().cast(),
                0,
                2,
                123,
                50,
                1,
                cigar.as_ptr(),
                -1,
                -1,
                0,
                4,
                seq.as_ptr().cast(),
                qual.as_ptr().cast(),
                0,
            );
            assert_eq!(ret, 18);
            assert_eq!(
                bam_aux_append(
                    b,
                    c"CB".as_ptr().cast(),
                    b'Z' as u8,
                    6,
                    c"cell".as_ptr().cast()
                ),
                0
            );

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let header_text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:1000\n@SQ\tSN:chr2\tLN:2000\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, header_text),
                0
            );

            let fpw = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fpw.is_null());
            assert_eq!(bam_hdr_write(fpw, hdr), 0);
            let written = bam_write1(fpw, b);
            assert!(written > 0);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fpw), 0);

            let fpr = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fpr.is_null());
            let read_hdr = bam_hdr_read(fpr);
            assert!(!read_hdr.is_null());
            assert_eq!(sam_hdr_name2tid(&mut *read_hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *read_hdr, b"chr2"), 1);
            let read = bam_init1();
            assert_eq!(bam_read1(fpr, read), written);
            assert_eq!((*read).core.tid, (*b).core.tid);
            assert_eq!((*read).core.pos, (*b).core.pos);
            assert_eq!((*read).core.bin, (*b).core.bin);
            assert_eq!((*read).core.qual, (*b).core.qual);
            assert_eq!((*read).core.flag, (*b).core.flag);
            assert_eq!((*read).core.n_cigar, (*b).core.n_cigar);
            assert_eq!((*read).core.l_qseq, (*b).core.l_qseq);
            assert_eq!(CStr::from_ptr(bam_get_qname(read).cast()).to_bytes(), b"read");
            assert_eq!(*bam_get_cigar(read), *bam_get_cigar(b));
            let cb = bam_aux_get(read, c"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb).cast()).to_bytes(), b"cell");
            assert_eq!(bam_read1(fpr, read), -1);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(fpr), 0);

            bam_destroy1(read);
            bam_destroy1(b);
            sam_hdr_destroy(read_hdr);
            sam_hdr_destroy(hdr);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sam_format1_renders_core_fields_sequence_quality_and_aux() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let header_text = b"@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, header_text),
                0
            );

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [31u8, 32, 33, 34];
            assert_eq!(
                bam_set1(
                    b,
                    4,
                    c"read".as_ptr().cast(),
                    0,
                    0,
                    9,
                    50,
                    1,
                    cigar.as_ptr(),
                    0,
                    29,
                    -7,
                    4,
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"CB".as_ptr().cast(),
                    b'Z' as u8,
                    6,
                    c"cell".as_ptr().cast()
                ),
                0
            );
            let mut ks = kstring_t::default();
            let len = sam_format1(hdr, b, &mut ks);
            assert!(len > 0);
            assert_eq!(
                ks.data.as_slice(),
                b"read\t0\tchr1\t10\t50\t4M\t=\t30\t-7\tACGT\t@ABC\tCB:Z:cell"
            );

            bam_destroy1(b);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bam_sym_lookup_reports_core_string_numeric_and_aux_values() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let header_text = b"@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, header_text),
                0
            );

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CSOFT_CLIP as u32,
                (4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CHARD_CLIP as u32,
            ];
            let qual = [30u8, 31, 32, 33, 34];
            assert!(
                bam_set1(
                    b,
                    4,
                    c"read".as_ptr().cast(),
                    (BAM_FPAIRED | BAM_FREAD1) as u16,
                    0,
                    9,
                    50,
                    cigar.len(),
                    cigar.as_ptr(),
                    1,
                    29,
                    -7,
                    5,
                    c"AACGT".as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ) > 0
            );
            let nm = 3i32.to_le_bytes();
            assert_eq!(
                bam_aux_append(
                    b,
                    c"NM".as_ptr().cast(),
                    b'i' as u8,
                    nm.len() as i32,
                    nm.as_ptr()
                ),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"CB".as_ptr().cast(),
                    b'Z' as u8,
                    5,
                    c"cell".as_ptr().cast()
                ),
                0
            );

            let mut hb = hb_pair { h: hdr, b };
            let mut res = hts_expr_val_t {
                is_str: 0,
                is_true: 0,
                s: kstring_t::default(),
                d: 0.0,
            };
            let mut end: *mut i8 = std::ptr::null_mut();

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"qname".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(end.offset_from(c"qname".as_ptr()), 5);
            assert_eq!(res.is_str, 1);
            assert_eq!(res.s.data.as_slice(), b"read");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"cigar".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.s.data.as_slice(), b"1S4M2H");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"rname".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.s.data.as_slice(), b"chr1");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"mrname".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.s.data.as_slice(), b"chr2");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"seq".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.s.data.as_slice(), b"AACGT");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"flag.read1".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.is_str, 0);
            assert_eq!(res.d, BAM_FREAD1 as f64);

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"pos".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.d, 10.0);

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"sclen".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.d, 1.0);

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"[NM]".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.is_true, 1);
            assert_eq!(res.d, 3.0);

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"[CB]".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.s.data.as_slice(), b"cell");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"[ZZ]".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(res.is_true, 0);
            assert_eq!(res.is_str, 1);
            assert_eq!(res.s.data.len(), 0);

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"flag.bad".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                -1
            );

            let filt = crate::htslib_rs::hts::hts_filter_init(
                b"mapq >= 50 && rname == \"chr1\" && [NM] == 3",
            );
            assert!(!filt.is_null());
            assert_eq!(sam_c_1535_sam_passes_filter(hdr, b, filt.cast()), 1);
            crate::htslib_rs::hts::hts_filter_free(filt);

            let filt = crate::htslib_rs::hts::hts_filter_init(c"flag.read2 || [ZZ]".to_bytes());
            assert!(!filt.is_null());
            assert_eq!(sam_c_1535_sam_passes_filter(hdr, b, filt.cast()), 0);
            crate::htslib_rs::hts::hts_filter_free(filt);

            bam_destroy1(b);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn fastq_state_init_and_destroy_match_default_state() {
        unsafe {
            let x = sam_c_3786_fastq_state_init(b'@' as i32);
            assert!(!x.is_null());
            assert_eq!((*x).BC, [b'B' as u8, b'C' as u8, 0]);
            assert_eq!((*x).nprefix, b'@' as u8);
            assert_eq!((*x).casava, 0);
            assert_eq!((*x).aux, 0);
            assert_eq!((*x).rnum, 0);
            assert_eq!((*x).sra_names, 0);
            assert!((*x).tags.is_null());
            assert!((*x).name.data.is_empty());
            assert!((*x).seq.data.is_empty());
            assert!((*x).qual.data.is_empty());

            let mut match_ = libc::regmatch_t { rm_so: 0, rm_eo: 0 };
            let name = c"INST:RUN:FLOW:1:1101:1000:1000:ACGT";
            assert_eq!(
                libc::regexec(&(*x).regex, name.as_ptr(), 1, &mut match_, 0,),
                0
            );

            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.state = x.cast();
            sam_c_3802_fastq_state_destroy(&mut fp);
            assert!(fp.state.is_null());
            sam_c_3802_fastq_state_destroy(&mut fp);
        }
    }

    #[test]
    fn fastq_state_set_updates_options_and_aux_tag_whitelist() {
        unsafe {
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_CASAVA, std::ptr::null()),
                0
            );
            assert!(!fp.state.is_null());
            let x = fp.state.cast::<fastq_state>();
            assert_eq!((*x).nprefix, b'@' as u8);
            assert_eq!((*x).casava, 1);

            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_NAME2, std::ptr::null()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_RNUM, std::ptr::null()),
                0
            );
            assert_eq!((*x).sra_names, 1);
            assert_eq!((*x).rnum, 1);

            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_BARCODE, c"CR".as_ptr().cast()),
                0
            );
            assert_eq!((*x).BC, [b'C' as u8, b'R' as u8, 0]);

            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_UMI, c"RX,MI".as_ptr().cast()),
                0
            );
            assert_eq!((*x).UMI[0], [b'R' as u8, b'X' as u8, 0]);
            assert_eq!((*x).UMI[1], [b'M' as u8, b'I' as u8, 0]);
            assert_eq!((*x).UMI[2], [0, 0, 0]);

            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_AUX, c"NM,CB".as_ptr().cast()),
                0
            );
            assert_eq!((*x).aux, 1);
            let tags = (*x).tags.cast::<khash_tag_t>();
            assert!(!tags.is_null());
            for tcode in [
                b'N' as i32 * 256 + b'M' as i32,
                b'C' as i32 * 256 + b'B' as i32,
            ] {
                let mask = (*tags).n_buckets - 1;
                let mut k = __ac_Wang_hash(tcode as u32) & mask;
                let mut step = 0;
                while !kh_isempty((*tags).flags, k) && *(*tags).keys.add(k as usize) != tcode {
                    step += 1;
                    k = (k + step) & mask;
                }
                assert!(!kh_iseither((*tags).flags, k));
            }

            assert_eq!(
                sam_c_3815_fastq_state_set(
                    &mut fp,
                    FASTQ_OPT_UMI_REGEX,
                    c"^([^:]+):([A-Z]+)$".as_ptr().cast(),
                ),
                0
            );
            let mut matches: [libc::regmatch_t; 3] = std::mem::zeroed();
            assert_eq!(
                libc::regexec(
                    &(*x).regex,
                    c"READ:ACGT".as_ptr().cast(),
                    matches.len(),
                    matches.as_mut_ptr(),
                    0,
                ),
                0
            );

            sam_c_3802_fastq_state_destroy(&mut fp);
        }
    }

    #[test]
    fn fastq_parse1_reads_real_fastq_record_with_tags() {
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fastq-parse-{}-{}.fq",
            std::process::id(),
            line!()
        ));
        std::fs::write(
            &path,
            b"@READ:ACGT 1:Y:0:ACGT\nACGT\n+\nIIII\n@READ2\tNM:i:7\tCB:Z:cellA\nTGCA\n+\nJJJJ\n",
        )
        .unwrap();
        let path_c = CString::new(path.to_str().unwrap()).unwrap();

        unsafe {
            let fp = crate::htslib_rs::hts::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!((*fp).format.format, HTS_FORMAT_FASTQ_FORMAT);

            assert_eq!(
                sam_c_3815_fastq_state_set(fp, FASTQ_OPT_CASAVA, std::ptr::null()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(fp, FASTQ_OPT_AUX, c"NM,CB".as_ptr().cast()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(fp, FASTQ_OPT_UMI, c"RX".as_ptr().cast()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(
                    fp,
                    FASTQ_OPT_UMI_REGEX,
                    c"^[^:]+:([A-Za-z]+)$".as_ptr().cast(),
                ),
                0
            );

            let b = bam_init1();
            assert!(!b.is_null());
            assert!(sam_read1(fp, std::ptr::null_mut(), b) >= 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), b"READ");
            assert_eq!((*b).core.l_qseq, 4);
            assert_ne!((*b).core.flag as i32 & BAM_FREAD1, 0);
            assert_ne!((*b).core.flag as i32 & BAM_FQCFAIL, 0);

            let rx = bam_aux_get(b, c"RX".as_ptr().cast());
            assert!(!rx.is_null());
            assert_eq!(CStr::from_ptr(rx.add(1).cast()).to_bytes(), b"ACGT");
            let bc = bam_aux_get(b, c"BC".as_ptr().cast());
            assert!(!bc.is_null());
            assert_eq!(CStr::from_ptr(bc.add(1).cast()).to_bytes(), b"ACGT");

            assert!(sam_read1(fp, std::ptr::null_mut(), b) >= 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), b"READ2");
            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 7);
            let cb = bam_aux_get(b, c"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(cb.add(1).cast()).to_bytes(), b"cellA");

            assert_eq!(sam_read1(fp, std::ptr::null_mut(), b), -1);
            bam_destroy1(b);
            sam_c_3802_fastq_state_destroy(fp);
            (*fp).state = std::ptr::null_mut();
            assert_eq!(crate::htslib_rs::hts::hts_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fastq_format1_renders_options_and_sam_write1_writes_bgzf_fastq() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let qual = [40 as u8; 4];
            assert!(
                bam_set1(
                    b,
                    4,
                    c"read".as_ptr().cast(),
                    (BAM_FUNMAP | BAM_FMUNMAP | BAM_FPAIRED | BAM_FREAD1 | BAM_FQCFAIL) as u16,
                    -1,
                    -1,
                    0,
                    0,
                    std::ptr::null(),
                    -1,
                    -1,
                    0,
                    4,
                    c"ACGT".as_ptr().cast(),
                    qual.as_ptr(),
                    0,
                ) >= 0
            );
            let nm = [7u8];
            assert_eq!(
                bam_aux_append(b, c"NM".as_ptr().cast(), b'C' as u8, 1, nm.as_ptr()),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"RX".as_ptr().cast(),
                    b'Z' as u8,
                    5,
                    c"ACGT".as_ptr().cast()
                ),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"BC".as_ptr().cast(),
                    b'Z' as u8,
                    5,
                    c"ACGT".as_ptr().cast()
                ),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"CB".as_ptr().cast(),
                    b'Z' as u8,
                    5,
                    c"cell".as_ptr().cast()
                ),
                0
            );

            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_CASAVA, std::ptr::null()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_RNUM, std::ptr::null()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_UMI, c"RX".as_ptr().cast()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_AUX, c"NM,CB".as_ptr().cast()),
                0
            );
            let x = fp.state.cast::<fastq_state>();
            let mut out = kstring_t::default();
            let expected = b"@read:ACGT/1 1:Y:0:ACGT\tNM:i:7\tCB:Z:cell\nACGT\n+\nIIII\n";
            assert_eq!(
                sam_c_4413_fastq_format1(x, b, &mut out),
                expected.len() as i32
            );
            assert_eq!(
                out.data.as_slice(),
                expected
            );
            ks_free(&mut out);

            let path = std::env::temp_dir().join(format!(
                "htslib_rs-fastq-write-{}-{}.fq.gz",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut out_fp: htsFile = htsFile::default();
            std::ptr::write(&mut out_fp.line, kstring_t::default());
            out_fp.bitfields = 1 << 4;
            out_fp.fp.bgzf = bgzf;
            out_fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            out_fp.state = fp.state;

            assert_eq!(
                sam_c_4553_sam_write1(&mut out_fp, std::ptr::null(), b),
                expected.len() as i32
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);
            sam_c_3802_fastq_state_destroy(&mut out_fp);

            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let mut buf = vec![0u8; expected.len()];
            assert_eq!(
                bgzf_read(bgzf, buf.as_mut_ptr().cast(), expected.len()),
                expected.len() as isize
            );
            assert_eq!(buf, expected);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);

            bam_destroy1(b);
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn sam_hdr_write_writes_sam_text_and_stores_header_copy() {
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-sam-hdr-write-{}-{}.sam.gz",
            std::process::id(),
            line!()
        ));
        let path_c = CString::new(path.to_str().unwrap()).unwrap();

        unsafe {
            assert_eq!(sam_hdr_write(std::ptr::null_mut(), std::ptr::null()), -1);

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let header_text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:100\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, header_text),
                0
            );

            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.bitfields = 1 << 4;
            fp.fp.bgzf = bgzf;
            fp.format.format = HTS_FORMAT_SAM;
            fp.format.category = HTS_FORMAT_SEQUENCE_DATA;

            assert_eq!(sam_hdr_write(&mut fp, hdr), 0);
            assert!(!fp.bam_header.is_null());
            assert_ne!(fp.bam_header, hdr.cast::<std::ffi::c_void>());
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);

            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let mut out = [0u8; 64];
            let n = bgzf_read(bgzf, out.as_mut_ptr().cast(), header_text.len());
            assert_eq!(n, header_text.len() as isize);
            assert_eq!(&out[..header_text.len()], header_text);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);

            sam_hdr_destroy(fp.bam_header.cast());
            sam_hdr_destroy(hdr);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sam_index_build_wrappers_report_missing_inputs_like_htslib() {
        unsafe {
            let missing = CString::new(format!(
                "/tmp/htslib_rs-missing-index-input-{}-{}.bam",
                std::process::id(),
                line!()
            ))
            .unwrap();

            assert!(sam_index_build3(missing.as_ptr().cast(), std::ptr::null(), 14, 0) < 0);
            assert!(sam_index_build2(missing.as_ptr().cast(), std::ptr::null(), 14) < 0);
            assert!(sam_index_build(missing.as_ptr().cast(), 14) < 0);
            assert!(bam_index_build(missing.as_ptr().cast(), 14) < 0);
        }
    }

    #[test]
    fn sam_index_builds_index_from_generated_bam_stream() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-sam-index-{}-{}.bam",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let text = b"@SQ\tSN:chr1\tLN:100\n";
            (*hdr).text = ({ let mut v: Vec<u8> = vec![0u8; (text.len() as u64 + 1) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            assert!(!(*hdr).text.is_null());
            std::ptr::copy_nonoverlapping(text.as_ptr(), (*hdr).text, (text.len()) as usize);
            *(*hdr).text.add(text.len()) = 0;
            (*hdr).l_text = text.len();
            (*hdr).n_targets = 1;
            (*hdr).target_len =
                Box::into_raw(Box::new(std::mem::zeroed::<u32>())).cast::<()>().cast();
            (*hdr).target_name =
                ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); 1]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
                    .cast();
            assert!(!(*hdr).target_len.is_null());
            assert!(!(*hdr).target_name.is_null());
            *(*hdr).target_len = 100;
            *(*hdr).target_name = {
                let mut dup: Vec<u8> = b"chr1\0".to_vec();
                let p = dup.as_mut_ptr();
                std::mem::forget(dup);
                p
            };
            assert!(!(*(*hdr).target_name).is_null());
            assert_eq!(bam_hdr_write(bgzf, hdr), 0);

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [30u8, 31, 32, 33];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"read1".as_ptr().cast(),
                    0,
                    0,
                    10,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert!(bam_write1(bgzf, b) > 0);
            bam_destroy1(b);
            sam_hdr_destroy(hdr);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);

            let fp = crate::htslib_rs::hts::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let idx = sam_c_994_sam_index(fp, 0);
            assert!(!idx.is_null());
            assert_eq!((*idx).fmt, HTS_FMT_BAI);
            hts_idx_destroy(idx);
            assert_eq!(crate::htslib_rs::hts::hts_close(fp), 0);
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn sam_index_build_public_wrappers_create_default_and_custom_indexes() {
        unsafe fn write_indexable_bam(path: &std::path::Path) -> CString {
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let text = b"@SQ\tSN:chr1\tLN:100\n";
            (*hdr).text = ({ let mut v: Vec<u8> = vec![0u8; (text.len() as u64 + 1) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            assert!(!(*hdr).text.is_null());
            std::ptr::copy_nonoverlapping(text.as_ptr(), (*hdr).text, (text.len()) as usize);
            *(*hdr).text.add(text.len()) = 0;
            (*hdr).l_text = text.len();
            (*hdr).n_targets = 1;
            (*hdr).target_len =
                Box::into_raw(Box::new(std::mem::zeroed::<u32>())).cast::<()>().cast();
            (*hdr).target_name =
                ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); 1]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
                    .cast();
            assert!(!(*hdr).target_len.is_null());
            assert!(!(*hdr).target_name.is_null());
            *(*hdr).target_len = 100;
            *(*hdr).target_name = {
                let mut dup: Vec<u8> = b"chr1\0".to_vec();
                let p = dup.as_mut_ptr();
                std::mem::forget(dup);
                p
            };
            assert!(!(*(*hdr).target_name).is_null());
            assert_eq!(bam_hdr_write(bgzf, hdr), 0);

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [30u8, 31, 32, 33];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"read1".as_ptr().cast(),
                    0,
                    0,
                    10,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert!(bam_write1(bgzf, b) > 0);
            bam_destroy1(b);
            sam_hdr_destroy(hdr);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);
            path_c
        }

        let base = std::env::temp_dir();
        let path_build3 = base.join(format!(
            "htslib_rs-sam-index-build3-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_build2 = base.join(format!(
            "htslib_rs-sam-index-build2-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_build = base.join(format!(
            "htslib_rs-sam-index-build-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_bam = base.join(format!(
            "htslib_rs-bam-index-build-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let idx_build3 = base.join(format!(
            "htslib_rs-sam-index-build3-{}-{}.bai",
            std::process::id(),
            line!()
        ));
        let idx_build2 = base.join(format!(
            "htslib_rs-sam-index-build2-{}-{}.bai",
            std::process::id(),
            line!()
        ));

        unsafe {
            let c_build3 = write_indexable_bam(&path_build3);
            let c_build2 = write_indexable_bam(&path_build2);
            let c_build = write_indexable_bam(&path_build);
            let c_bam = write_indexable_bam(&path_bam);
            let c_idx_build3 = CString::new(idx_build3.to_str().unwrap()).unwrap();
            let c_idx_build2 = CString::new(idx_build2.to_str().unwrap()).unwrap();

            assert_eq!(
                sam_index_build3(c_build3.as_ptr().cast(), c_idx_build3.as_ptr().cast(), 0, 0),
                0
            );
            assert_eq!(
                sam_index_build2(c_build2.as_ptr().cast(), c_idx_build2.as_ptr().cast(), 0),
                0
            );
            assert_eq!(sam_index_build(c_build.as_ptr().cast(), 0), 0);
            assert_eq!(bam_index_build(c_bam.as_ptr().cast(), 0), 0);
        }

        let default_build_idx = std::path::PathBuf::from(format!("{}.bai", path_build.display()));
        let default_bam_idx = std::path::PathBuf::from(format!("{}.bai", path_bam.display()));
        assert!(idx_build3.exists());
        assert!(idx_build2.exists());
        assert!(default_build_idx.exists());
        assert!(default_bam_idx.exists());

        std::fs::remove_file(path_build3).unwrap();
        std::fs::remove_file(path_build2).unwrap();
        std::fs::remove_file(path_build).unwrap();
        std::fs::remove_file(path_bam).unwrap();
        std::fs::remove_file(idx_build3).unwrap();
        std::fs::remove_file(idx_build2).unwrap();
        std::fs::remove_file(default_build_idx).unwrap();
        std::fs::remove_file(default_bam_idx).unwrap();
    }

    #[test]
    fn sam_bam_seek_tell_and_index_load_wrappers_match_c_edges() {
        unsafe {
            assert_eq!(
                sam_c_1631_bam_pseek(std::ptr::null_mut(), 0, libc::SEEK_SET),
                -1
            );
            assert_eq!(sam_c_1638_bam_ptell(std::ptr::null_mut()), -1);

            let mut bgzf: BGZF = BGZF::default();
            bgzf.block_address = 0x1234;
            bgzf.block_offset = 0x5678;
            assert_eq!(
                sam_c_1638_bam_ptell((&mut bgzf as *mut BGZF).cast::<std::ffi::c_void>()),
                0x12345678
            );

            let missing = CString::new(format!(
                "/tmp/htslib_rs-missing-index-load-{}-{}.bam",
                std::process::id(),
                line!()
            ))
            .unwrap();
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.format.format = HTS_FORMAT_BAM;
            assert!(sam_c_1649_index_load(
                &mut fp,
                missing.as_ptr().cast(),
                std::ptr::null(),
                HTS_IDX_SAVE_REMOTE,
            )
            .is_null());

            fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            assert!(sam_c_1649_index_load(
                &mut fp,
                missing.as_ptr().cast(),
                std::ptr::null(),
                HTS_IDX_SAVE_REMOTE,
            )
            .is_null());
        }
    }

    #[test]
    fn sam_index_load_public_wrappers_reject_non_indexable_formats() {
        unsafe {
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.format.format = HTS_FORMAT_FASTQ_FORMAT;

            assert!(sam_index_load3(
                &mut fp,
                std::ptr::null(),
                std::ptr::null(),
                HTS_IDX_SAVE_REMOTE,
            )
            .is_null());
            assert!(sam_index_load2(&mut fp, std::ptr::null(), std::ptr::null()).is_null());
            assert!(sam_index_load(&mut fp, std::ptr::null()).is_null());

            fp.format.format = HTS_FORMAT_EMPTY_FORMAT;
            assert!(sam_index_load3(&mut fp, std::ptr::null(), std::ptr::null(), 0).is_null());
        }
    }

    #[test]
    fn sam_state_create_err_and_destroy_manage_owned_state() {
        unsafe {
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.format.format = HTS_FORMAT_SAM;
            fp.format.compression = crate::htslib_rs::hts::HTS_COMPRESSION_NO_COMPRESSION;

            let fd = sam_c_3048_sam_state_create(&mut fp);
            assert!(!fd.is_null());
            assert_eq!(fp.state, fd.cast());
            assert_eq!((*fd).fp, &mut fp as *mut htsFile);

            sam_c_3069_sam_state_err(fd, 5);
            sam_c_3069_sam_state_err(fd, 7);
            assert_eq!((*fd).errcode, 5);

            let lines =
                Box::into_raw(Box::new(std::mem::zeroed::<sp_lines>())).cast::<()>()
                    .cast::<sp_lines>();
            assert!(!lines.is_null());
            (*lines).data = ({ let mut v: Vec<u8> = vec![0u8; (8) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            (*fd).lines = NonNull::new(lines);

            let curr = Box::into_raw(Box::new(std::mem::zeroed::<sp_bams>())).cast::<()>()
                .cast::<sp_bams>();
            assert!(!curr.is_null());
            (*curr).abams = 1;
            (*curr).bams =
                Box::into_raw(Box::new(bam1_t::default())).cast::<()>()
                    .cast::<bam1_t>();
            assert!(!(*curr).bams.is_null());
            // data is an owned Vec; assign via the Vec API (calloc'd slot is a
            // zeroed == empty Vec, so the assignment drops nothing real).
            (*(*curr).bams).data = Vec::with_capacity(4);
            (*fd).curr_bam = NonNull::new(curr);

            assert_eq!(sam_state_destroy(&mut fp), -5);
            assert!(fp.state.is_null());
            assert_eq!(sam_state_destroy(&mut fp), 0);

            let mut bam_fp: htsFile = htsFile::default();
            std::ptr::write(&mut bam_fp.line, kstring_t::default());
            bam_fp.format.format = HTS_FORMAT_BAM;
            assert!(sam_c_3048_sam_state_create(&mut bam_fp).is_null());
            assert!(bam_fp.state.is_null());
        }
    }

    #[test]
    fn sam_worker_cleanup_callbacks_free_owned_blocks() {
        unsafe {
            let lines =
                Box::into_raw(Box::new(std::mem::zeroed::<sp_lines>())).cast::<()>()
                    .cast::<sp_lines>();
            assert!(!lines.is_null());
            (*lines).data = ({ let mut v: Vec<u8> = vec![0u8; (16) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            assert!(!(*lines).data.is_null());

            let nested =
                Box::into_raw(Box::new(std::mem::zeroed::<sp_bams>())).cast::<()>()
                    .cast::<sp_bams>();
            assert!(!nested.is_null());
            (*nested).abams = 1;
            (*nested).bams =
                Box::into_raw(Box::new(bam1_t::default())).cast::<()>()
                    .cast::<bam1_t>();
            assert!(!(*nested).bams.is_null());
            // data is an owned Vec; give it a buffer via the Vec API (the slot is
            // calloc'd to a zeroed == empty Vec, so this assignment is sound).
            (*(*nested).bams).data = Vec::with_capacity(8);
            assert!((*(*nested).bams).data.capacity() >= 8);
            (*lines).bams = NonNull::new(nested);

            sam_c_3200_cleanup_sp_lines(lines.cast());
            sam_c_3200_cleanup_sp_lines(std::ptr::null_mut());

            let bams = Box::into_raw(Box::new(std::mem::zeroed::<sp_bams>())).cast::<()>()
                .cast::<sp_bams>();
            assert!(!bams.is_null());
            (*bams).abams = 1;
            (*bams).bams =
                Box::into_raw(Box::new(bam1_t::default())).cast::<()>()
                    .cast::<bam1_t>();
            assert!(!(*bams).bams.is_null());
            sam_c_3318_cleanup_sp_bams(bams.cast());
            sam_c_3318_cleanup_sp_bams(std::ptr::null_mut());

            assert!(sam_c_3313_sam_parse_eof(std::ptr::null_mut()).is_null());
        }
    }

    #[test]
    fn sam_parse_and_format_workers_round_trip_line_blocks() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut u8];
            let mut target_lens = [100u32];
            let mut hdr = sam_hdr_t {
                n_targets: 1,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: target_lens.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: target_names.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            let mut fd: SAM_state = std::mem::zeroed();
            fd.h = &mut hdr;
            fd.fp = &mut fp;

            let text = b"read1\t0\tchr1\t1\t60\t4M\t*\t0\t0\tACGT\t!!!!\tNM:i:1\nread2\t4\t*\t0\t0\t*\t*\t0\t0\t*\t*\tZZ:Z:tag\n";
            let gl = Box::into_raw(Box::new(std::mem::zeroed::<sp_lines>())).cast::<()>()
                .cast::<sp_lines>();
            assert!(!gl.is_null());
            (*gl).alloc = text.len() as i32 + 8;
            (*gl).data_size = text.len() as i32;
            (*gl).data = ({ let mut v: Vec<u8> = vec![0u8; ((*gl).alloc) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            assert!(!(*gl).data.is_null());
            std::ptr::copy_nonoverlapping(text.as_ptr().cast::<u8>(), (*gl).data.cast::<u8>(), (text.len()) as usize);
            (*gl).fd = NonNull::new(&mut fd);
            (*gl).serial = 17;

            let gb = sam_c_3215_sam_parse_worker(gl.cast()).cast::<sp_bams>();
            assert!(!gb.is_null());
            assert_eq!((*gb).serial, 17);
            assert_eq!((*gb).nbams, 2);
            assert!(fd.lines.is_some());
            assert_eq!(
                CStr::from_ptr(bam_get_qname((*gb).bams).cast()).to_bytes(),
                b"read1"
            );
            assert_eq!(
                CStr::from_ptr(bam_get_qname((*gb).bams.add(1)).cast()).to_bytes(),
                b"read2"
            );

            (*gb).fd = NonNull::new(&mut fd);
            let out = sam_c_3652_sam_format_worker(gb.cast()).cast::<sp_lines>();
            assert!(!out.is_null());
            assert_eq!((*out).serial, 17);
            let formatted =
                std::slice::from_raw_parts((*out).data.cast::<u8>(), (*out).data_size as usize);
            assert!(formatted.starts_with(b"read1\t0\tchr1\t1\t60\t4M"));
            assert!(formatted.ends_with(b"ZZ:Z:tag\n"));
            assert!(fd.bams.is_some());

            sam_c_3200_cleanup_sp_lines(out.cast());
            sam_c_3076_sam_free_sp_bams(fd.bams.unwrap().as_ptr());
        }
    }

    #[test]
    fn bam_record_allocation_copy_and_destroy_match_htslib_ownership_rules() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            assert_eq!((*b).data.len() as i32, 0);
            assert_eq!((*b).data.capacity() as u32, 0);
            assert!(bam_get_mempolicy(b) == 0);
            bam_destroy1(b);
        }

        let src_data = vec![1u8, 2, 3, 4, 5, 6];
        let src = bam1_t {
            core: bam1_core_t {
                pos: 42,
                tid: 7,
                bin: 0,
                qual: 60,
                l_extranul: 0,
                flag: BAM_FREVERSE as u16,
                l_qname: 0,
                n_cigar: 0,
                l_qseq: 0,
                mtid: -1,
                mpos: -1,
                isize: 0,
            },
            id: 99,
            data: src_data.clone(),
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let dst = bam_dup1(&src);
            assert!(!dst.is_null());
            assert_ne!((*dst).data.as_ptr(), src.data.as_ptr());
            assert_eq!((*dst).core.pos, 42);
            assert_eq!((*dst).core.tid, 7);
            assert_eq!((*dst).core.flag, BAM_FREVERSE as u16);
            assert_eq!((*dst).id, 99);
            assert_eq!((*dst).data.len() as i32, src_data.len() as i32);
            assert_eq!((*dst).data.as_slice(), src_data.as_slice());
            bam_destroy1(dst);
        }

        let external = vec![9u8, 8, 7];
        let mut owned_struct = bam1_t {
            core: bam1_core_t {
                pos: 0,
                tid: 0,
                bin: 0,
                qual: 0,
                l_extranul: 0,
                flag: 0,
                l_qname: 0,
                n_cigar: 0,
                l_qseq: 0,
                mtid: 0,
                mpos: 0,
                isize: 0,
            },
            id: 0,
            data: external.clone(),
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let pre_grow_cap = owned_struct.data.capacity();
            assert_eq!(realloc_bam_data(&mut owned_struct, 12), 0);
            // Growing past the current capacity must (re)allocate to a larger buffer.
            // The owned Vec may legitimately keep the same backing address when the
            // system allocator extends the chunk in place, so assert the capacity
            // grew (proof of reallocation/growth) rather than a pointer change.
            assert!(owned_struct.data.capacity() > pre_grow_cap);
            assert!(owned_struct.data.capacity() >= 12);
            assert_eq!(bam_get_mempolicy(&mut owned_struct) & BAM_USER_OWNS_DATA, 0);
            assert_eq!(&owned_struct.data[..external.len()], external.as_slice());
            bam_destroy1(&mut owned_struct);
            assert!(owned_struct.data.is_empty());
            assert_eq!(owned_struct.data.len() as i32, 0);
            assert_eq!(owned_struct.data.capacity() as u32, 0);
        }
    }

    #[test]
    fn bam_aux_mutation_helpers_append_update_and_remove_tags() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            let nm_val = 5i32.to_le_bytes();
            assert_eq!(
                bam_aux_append(
                    b,
                    b"NM".as_ptr().cast(),
                    b'i' as u8,
                    nm_val.len() as i32,
                    nm_val.as_ptr(),
                ),
                0
            );
            let nm = bam_aux_get(b, b"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux_type(nm), b'i' as u8);
            assert_eq!(bam_aux2i(nm), 5);
            assert_eq!((*b).data.len() as i32, 7);

            assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), -3), 0);
            let nm = bam_aux_get(b, b"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux_type(nm), b'i' as u8);
            assert_eq!(bam_aux2i(nm), -3);

            assert_eq!(
                bam_aux_update_str(b, b"CB".as_ptr().cast(), 3, b"abc".as_ptr().cast()),
                0
            );
            let cb = bam_aux_get(b, b"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(bam_aux_type(cb), b'Z' as u8);
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb).cast()).to_bytes(), b"abc");

            assert_eq!(
                bam_aux_update_str(b, b"CB".as_ptr().cast(), -1, c"xy".as_ptr().cast()),
                0
            );
            let cb = bam_aux_get(b, b"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb).cast()).to_bytes(), b"xy");

            assert_eq!(bam_aux_update_float(b, b"FZ".as_ptr().cast(), 1.5), 0);
            let fz = bam_aux_get(b, b"FZ".as_ptr().cast());
            assert!(!fz.is_null());
            assert_eq!(bam_aux_type(fz), b'f' as u8);
            assert!((bam_aux2f(fz) - 1.5).abs() < 1e-6);

            let d2 = 2.5f64.to_le_bytes();
            assert_eq!(
                bam_aux_append(
                    b,
                    b"D2".as_ptr().cast(),
                    b'd' as u8,
                    d2.len() as i32,
                    d2.as_ptr(),
                ),
                0
            );
            assert_eq!(bam_aux_update_float(b, b"D2".as_ptr().cast(), 3.25), 0);
            let d2 = bam_aux_get(b, b"D2".as_ptr().cast());
            assert!(!d2.is_null());
            assert_eq!(bam_aux_type(d2), b'f' as u8);
            assert!((bam_aux2f(d2) - 3.25).abs() < 1e-6);

            let arr = [10u16, 20u16];
            assert_eq!(
                bam_aux_update_array(
                    b,
                    b"XA".as_ptr().cast(),
                    b'S',
                    arr.len() as u32,
                    arr.as_ptr().cast_mut().cast::<()>(),
                ),
                0
            );
            let xa = bam_aux_get(b, b"XA".as_ptr().cast());
            assert!(!xa.is_null());
            assert_eq!(bam_aux_type(xa), b'B' as u8);
            assert_eq!(bam_auxB_len(xa), 2);
            assert_eq!(bam_auxB2i(xa, 0), 10);
            assert_eq!(bam_auxB2i(xa, 1), 20);

            let shrunk = [7u8];
            assert_eq!(
                bam_aux_update_array(
                    b,
                    b"XA".as_ptr().cast(),
                    b'C',
                    shrunk.len() as u32,
                    shrunk.as_ptr().cast_mut().cast::<()>(),
                ),
                0
            );
            let xa = bam_aux_get(b, b"XA".as_ptr().cast());
            assert!(!xa.is_null());
            assert_eq!(bam_auxB_len(xa), 1);
            assert_eq!(bam_auxB2i(xa, 0), 7);

            let mut ks = kstring_t::default();
            assert_eq!(
                bam_aux_get_str(b, b"NM".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(&ks.data).unwrap(),
                "NM:i:-3"
            );
            ks.data.clear();
            assert_eq!(
                bam_aux_get_str(b, b"CB".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(&ks.data).unwrap(),
                "CB:Z:xy"
            );
            ks.data.clear();
            assert_eq!(
                bam_aux_get_str(b, b"FZ".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(&ks.data).unwrap(),
                "FZ:f:1.5"
            );
            ks.data.clear();
            assert_eq!(
                bam_aux_get_str(b, b"XA".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(&ks.data).unwrap(),
                "XA:B:C,7"
            );
            ks.data.clear();
            assert_eq!(
                bam_aux_get_str(b, b"ZZ".as_ptr().cast(), &mut ks as *mut kstring_t),
                0
            );

            let cb = bam_aux_get(b, b"CB".as_ptr().cast());
            let next = bam_aux_remove(b, cb);
            assert!(!next.is_null());
            assert!(bam_aux_get(b, b"CB".as_ptr().cast()).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::ENOENT as i32
            );

            let xa = bam_aux_get(b, b"XA".as_ptr().cast());
            assert_eq!(bam_aux_del(b, xa), 0);
            assert!(bam_aux_get(b, b"XA".as_ptr().cast()).is_null());

            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_aux_update_int_preserves_middle_tag_neighbors() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            let aa = [1u8];
            assert_eq!(
                bam_aux_append(b, b"AA".as_ptr().cast(), b'C' as u8, 1, aa.as_ptr()),
                0
            );
            let nm = [7u8];
            assert_eq!(
                bam_aux_append(b, b"NM".as_ptr().cast(), b'C' as u8, 1, nm.as_ptr()),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    b"ZZ".as_ptr().cast(),
                    b'Z' as u8,
                    5,
                    c"tail".as_ptr().cast(),
                ),
                0
            );

            let old_len = (*b).data.len() as i32;
            assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), 70_000), 0);
            assert_eq!((*b).data.len() as i32, old_len + 3);

            let first = bam_aux_first(b);
            assert!(!first.is_null());
            assert_eq!(
                std::slice::from_raw_parts(bam_aux_tag(first).cast::<u8>(), 2),
                b"AA"
            );
            assert_eq!(bam_aux_type(first), b'C' as u8);
            assert_eq!(bam_aux2i(first), 1);

            let middle = bam_aux_next(b, first);
            assert!(!middle.is_null());
            assert_eq!(
                std::slice::from_raw_parts(bam_aux_tag(middle).cast::<u8>(), 2),
                b"NM"
            );
            assert_eq!(bam_aux_type(middle), b'I' as u8);
            assert_eq!(bam_aux2i(middle), 70_000);

            let last = bam_aux_next(b, middle);
            assert!(!last.is_null());
            assert_eq!(
                std::slice::from_raw_parts(bam_aux_tag(last).cast::<u8>(), 2),
                b"ZZ"
            );
            assert_eq!(bam_aux_type(last), b'Z' as u8);
            assert_eq!(CStr::from_ptr(bam_aux2Z(last).cast()).to_bytes(), b"tail");
            assert!(bam_aux_next(b, last).is_null());

            assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), 8), 0);
            let middle = bam_aux_get(b, b"NM".as_ptr().cast());
            assert!(!middle.is_null());
            assert_eq!(bam_aux_type(middle), b'I' as u8);
            assert_eq!(bam_aux2i(middle), 8);
            let last = bam_aux_get(b, b"ZZ".as_ptr().cast());
            assert!(!last.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(last).cast()).to_bytes(), b"tail");

            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_aux_update_int_boundary_types_match_htslib() {
        let cases = [
            (i32::MIN as i64, b'i'),
            (-32769, b'i'),
            (-32768, b's'),
            (-129, b's'),
            (-128, b'c'),
            (-1, b'c'),
            (0, b'C'),
            (254, b'C'),
            (255, b'S'),
            (65534, b'S'),
            (65535, b'I'),
            (u32::MAX as i64, b'I'),
        ];

        unsafe {
            for (value, expected_type) in cases {
                let b = bam_init1();
                assert!(!b.is_null());
                assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), value), 0);
                let nm = bam_aux_get(b, b"NM".as_ptr().cast());
                assert!(!nm.is_null());
                assert_eq!(bam_aux_type(nm), expected_type as u8, "value {value}");
                assert_eq!(bam_aux2i(nm), value);
                bam_destroy1(b);
            }

            let b = bam_init1();
            assert!(!b.is_null());
            assert_eq!(
                bam_aux_update_int(b, b"NM".as_ptr().cast(), i32::MIN as i64 - 1),
                -1
            );
            assert_eq!(
                *libc::__errno_location(),
                libc::EOVERFLOW as i32
            );
            assert_eq!(
                bam_aux_update_int(b, b"NM".as_ptr().cast(), u32::MAX as i64 + 1),
                -1
            );
            assert_eq!(
                *libc::__errno_location(),
                libc::EOVERFLOW as i32
            );
            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_aux_accessors_report_truncated_payloads_as_invalid() {
        let mut data = vec![b'X', b'Y', b'B', b'C', 2, 0, 0, 0, 9];
        let b = bam1_t {
            core: bam1_core_t {
                pos: 0,
                tid: 0,
                bin: 0,
                qual: 0,
                l_extranul: 0,
                flag: 0,
                l_qname: 0,
                n_cigar: 0,
                l_qseq: 0,
                mtid: 0,
                mpos: 0,
                isize: 0,
            },
            id: 0,
            data,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let first = bam_aux_first(&b);
            assert!(!first.is_null());
            assert_eq!(bam_aux_type(first), b'B' as u8);
            assert!(bam_aux_next(&b, first).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            assert!(bam_aux_get(&b, c"XY".as_ptr().cast()).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );
        }
    }

    #[test]
    fn bam_aux_get_l_aux_tracks_record_payload_after_mutations() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(3u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACG").unwrap();
            assert_eq!(
                bam_set1(
                    b,
                    4,
                    c"read".as_ptr().cast(),
                    0,
                    0,
                    0,
                    20,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    3,
                    seq.as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ),
                17
            );
            assert_eq!(bam_get_l_aux(b), 0);

            assert_eq!(bam_aux_update_int(b, c"NM".as_ptr().cast(), 300), 0);
            assert_eq!(bam_get_l_aux(b), 5);
            assert_eq!(
                bam_aux_update_str(b, c"CB".as_ptr().cast(), -1, c"cell".as_ptr().cast()),
                0
            );
            assert_eq!(bam_get_l_aux(b), 13);

            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert_eq!(bam_aux_del(b, nm), 0);
            assert_eq!(bam_get_l_aux(b), 8);
            let cb = bam_aux_get(b, c"CB".as_ptr().cast());
            assert_eq!(bam_aux_del(b, cb), 0);
            assert_eq!(bam_get_l_aux(b), 0);

            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_aux_numeric_converters_set_errno_on_wrong_type_and_bounds() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            assert_eq!(
                bam_aux_update_str(b, c"CB".as_ptr().cast(), -1, c"cell".as_ptr().cast()),
                0
            );
            let cb = bam_aux_get(b, c"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(bam_aux2A(cb), 0);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );
            assert_eq!(bam_auxB_len(cb), 0);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            let values = [1u8, 2u8];
            assert_eq!(
                bam_aux_update_array(
                    b,
                    c"XA".as_ptr().cast(),
                    b'C',
                    values.len() as u32,
                    values.as_ptr().cast_mut().cast::<()>(),
                ),
                0
            );
            let xa = bam_aux_get(b, c"XA".as_ptr().cast());
            assert!(!xa.is_null());
            assert_eq!(bam_auxB2i(xa, 2), 0);
            assert_eq!(
                *libc::__errno_location(),
                libc::ERANGE as i32
            );
            assert_eq!(bam_auxB2f(xa, 2), 0.0);
            assert_eq!(
                *libc::__errno_location(),
                libc::ERANGE as i32
            );

            bam_destroy1(b);
        }
    }

    #[test]
    fn pileup_mempool_reuses_nodes_like_htslib_static_pool() {
        unsafe {
            assert_eq!(G_CSTATE_NULL.k, -1);
            assert_eq!(G_CSTATE_NULL.y, 0);
            assert_eq!(G_CSTATE_NULL.x, 0);
            assert_eq!(G_CSTATE_NULL.end, 0);

            let mut mp = mp_init();
            assert_eq!(mp.cnt, 0);
            assert_eq!(mp.n, 0);
            assert_eq!(mp.max, 0);
            assert!(mp.buf.is_empty());

            let mut node = mp_alloc(&mut mp).expect("initial mempool allocation succeeds");
            assert_eq!(mp.cnt, 1);
            assert_eq!(node.as_ref().b.data.len() as i32, 0);
            assert_eq!(node.as_ref().beg, 0);
            assert_eq!(node.as_ref().s.k, 0);
            node.as_mut().next = Some(node);
            // data is an owned Vec; give it some capacity via the Vec API instead
            // of malloc + m_data.
            node.as_mut().b.data = Vec::with_capacity(4);

            mp_free(&mut mp, node);
            assert_eq!(mp.cnt, 0);
            assert_eq!(mp.n, 1);
            assert_eq!(mp.max, 256);
            assert!(node.as_ref().next.is_none());

            let reused = mp_alloc(&mut mp).expect("mempool reuses cached node");
            assert_eq!(reused, node);
            assert_eq!(mp.cnt, 1);
            assert_eq!(mp.n, 0);

            mp_free(&mut mp, reused);
            mp_destroy(mp);
        }
    }

    unsafe extern "C" fn test_plp_auto_callback(_data: *mut (), _b: *mut bam1_t) -> i32 {
        -1
    }

    #[test]
    fn bam_plp_init_and_destroy_match_htslib_initial_state() {
        unsafe {
            let iter = bam_plp_init(None, std::ptr::null_mut());
            assert!(!iter.is_null());
            assert!((*iter).mp.is_some());
            assert_eq!((*iter).head, (*iter).tail);
            assert!((*iter).head.is_some());
            assert_eq!((*iter).mp.as_ref().unwrap().cnt, 1);
            assert_eq!((*iter).max_tid, -1);
            assert_eq!((*iter).max_pos, -1);
            assert_eq!((*iter).maxcnt, 8000);
            assert!((*iter).func.is_none());
            assert!((*iter).data.is_null());
            assert!((*iter).b.is_none());
            bam_plp_destroy(iter);

            let mut data = 7u8;
            let iter = bam_plp_init(
                Some(test_plp_auto_callback),
                (&mut data as *mut u8).cast::<()>(),
            );
            assert_eq!(
                (*iter).func.map(|f| f as usize),
                Some(test_plp_auto_callback as usize)
            );
            assert_eq!((*iter).data, (&mut data as *mut u8).cast::<()>());
            assert!((*iter).b.is_some());
            bam_plp_destroy(iter);
        }
    }

    #[test]
    fn bam_plp_push_next_and_auto_emit_simple_match_pileup() {
        let mut data = vec![0u8; 4 + 4 + 2 + 3];
        data[0] = b'r';
        data[4..8].copy_from_slice(&((3u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32).to_ne_bytes());
        data[8] = 0x12;
        data[9] = 0x40;
        data[10..13].copy_from_slice(&[30, 31, 32]);
        let record = bam1_t {
            core: bam1_core_t {
                pos: 10,
                tid: 0,
                bin: 0,
                qual: 60,
                l_extranul: 0,
                flag: 0,
                l_qname: 4,
                n_cigar: 1,
                l_qseq: 3,
                mtid: -1,
                mpos: -1,
                isize: 0,
            },
            id: 0,
            data,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let iter = bam_plp_init(None, std::ptr::null_mut());
            assert_eq!(bam_plp_push(iter, &record), 0);
            assert_eq!(bam_plp_push(iter, std::ptr::null()), 0);

            let mut tid = -1;
            let mut pos = -1;
            let mut n_plp = -1;
            let plp = bam_plp64_next(iter, &mut tid, &mut pos, &mut n_plp);
            assert!(!plp.is_null());
            assert_eq!(tid, 0);
            assert_eq!(pos, 10);
            assert_eq!(n_plp, 1);
            assert_eq!((*plp).qpos, 0);
            assert_eq!(bam_pileup1_is_head(plp), 1);
            assert_eq!(bam_pileup1_is_tail(plp), 0);
            assert_eq!(bam_pileup1_is_del(plp), 0);

            let plp = bam_plp64_next(iter, &mut tid, &mut pos, &mut n_plp);
            assert!(!plp.is_null());
            assert_eq!(pos, 11);
            assert_eq!((*plp).qpos, 1);
            assert_eq!(bam_pileup1_is_head(plp), 0);
            assert_eq!(bam_pileup1_is_tail(plp), 0);

            let plp = bam_plp64_next(iter, &mut tid, &mut pos, &mut n_plp);
            assert!(!plp.is_null());
            assert_eq!(pos, 12);
            assert_eq!((*plp).qpos, 2);
            assert_eq!(bam_pileup1_is_tail(plp), 1);

            assert!(bam_plp64_next(iter, &mut tid, &mut pos, &mut n_plp).is_null());
            assert_eq!(n_plp, 0);
            bam_plp_destroy(iter);
        }
    }

    #[test]
    fn bam_plp_overlap_quality_adjusts_mate_pair_bases() {
        let mut left_data = vec![0u8; 8 + 4 + 2 + 4];
        let mut right_data = vec![0u8; 8 + 4 + 2 + 4];
        left_data[..5].copy_from_slice(b"read\0");
        right_data[..5].copy_from_slice(b"read\0");
        left_data[8..12].copy_from_slice(&((4u32 << 4) | BAM_CMATCH as u32).to_ne_bytes());
        right_data[8..12].copy_from_slice(&((4u32 << 4) | BAM_CMATCH as u32).to_ne_bytes());
        left_data[12..14].copy_from_slice(&[0x11, 0x11]);
        right_data[12..14].copy_from_slice(&[0x11, 0x11]);
        left_data[14..18].copy_from_slice(&[30, 30, 30, 30]);
        right_data[14..18].copy_from_slice(&[30, 30, 30, 30]);

        let core = bam1_core_t {
            pos: 0,
            tid: 0,
            bin: 0,
            qual: 60,
            l_extranul: 3,
            flag: (BAM_FPAIRED | BAM_FPROPER_PAIR) as u16,
            l_qname: 8,
            n_cigar: 1,
            l_qseq: 4,
            mtid: 0,
            mpos: 0,
            isize: 4,
        };
        let left = bam1_t {
            core,
            id: 0,
            data: left_data,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };
        let right = bam1_t {
            core,
            id: 0,
            data: right_data,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let iter = bam_plp_init(None, std::ptr::null_mut());
            assert_eq!(bam_plp_init_overlaps(iter), 0);
            assert_eq!(bam_plp_push(iter, &left), 0);
            assert_eq!(bam_plp_push(iter, &right), 0);

            let left_node = (*iter).head.unwrap().as_ptr();
            let right_node = (*left_node).next.unwrap().as_ptr();
            let left_qual = bam_get_qual(&(*left_node).b);
            let right_qual = bam_get_qual(&(*right_node).b);
            for i in 0..4 {
                let lq = *left_qual.add(i);
                let rq = *right_qual.add(i);
                assert_eq!(lq as u16 + rq as u16, 60);
                assert!(lq == 0 || rq == 0);
            }

            bam_plp_destroy(iter);
        }
    }

    #[test]
    fn bam_endpos_and_cigar_rlen_match_htslib_rules() {
        let mut data = vec![0u8; 4 + 16];
        data[4..8].copy_from_slice(&((5u32 << 4) | BAM_CSOFT_CLIP as u32).to_ne_bytes());
        data[8..12].copy_from_slice(&((10u32 << 4) | BAM_CMATCH as u32).to_ne_bytes());
        data[12..16].copy_from_slice(&((2u32 << 4) | BAM_CDEL as u32).to_ne_bytes());
        data[16..20].copy_from_slice(&((3u32 << 4) | BAM_CINS as u32).to_ne_bytes());

        let mut b = bam1_t {
            core: bam1_core_t {
                pos: 100,
                tid: 0,
                bin: 0,
                qual: 60,
                l_extranul: 0,
                flag: 0,
                l_qname: 4,
                n_cigar: 4,
                l_qseq: 18,
                mtid: -1,
                mpos: -1,
                isize: 0,
            },
            id: 0,
            data,
            mempolicy_and_reserved: 0,
        };

        unsafe {
            assert_eq!(bam_cigar_op((10u32 << 4) | BAM_CMATCH as u32), BAM_CMATCH);
            assert_eq!(bam_cigar_oplen((10u32 << 4) | BAM_CMATCH as u32), 10);
            assert_eq!(bam_cigar_type(BAM_CMATCH) & 2, 2);
            assert_eq!(bam_cigar_type(BAM_CDEL) & 2, 2);
            assert_eq!(bam_cigar_type(BAM_CINS) & 2, 0);
            assert_eq!(bam_cigar2rlen(4, bam_get_cigar(&b)), 12);
            assert_eq!(bam_endpos(&b), 112);
            b.core.flag = BAM_FUNMAP as u16;
            assert_eq!(bam_endpos(&b), 101);
            b.core.flag = 0;
            b.core.n_cigar = 0;
            assert_eq!(bam_endpos(&b), 101);
        }
    }

    #[test]
    fn pileup_bitfield_accessors_match_c_bit_order_on_little_endian_targets() {
        let p = bam_pileup1_t {
            b: std::ptr::null_mut(),
            qpos: 0,
            indel: 0,
            level: 0,
            bitfields: 0b10101,
            cd: bam_pileup_cd { i: 0 },
            cigar_ind: 0,
        };

        unsafe {
            assert_eq!(bam_pileup1_is_del(&p), 1);
            assert_eq!(bam_pileup1_is_head(&p), 0);
            assert_eq!(bam_pileup1_is_tail(&p), 1);
            assert_eq!(bam_pileup1_is_refskip(&p), 0);
            assert_eq!(bam_pileup1_aux(&p), 0);
        }
    }

    #[test]
    fn sam_itr_next_inline_rejects_non_bgzf_non_cram_and_null_iter_like_htslib() {
        let mut fp = crate::htslib_rs::hts::htsFile {
            bitfields: 0,
            padding_0: 0,
            lineno: 0,
            line: crate::htslib_rs::hts::kstring_t::default(),
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: crate::htslib_rs::hts::htsFilePtr {
                bgzf: std::ptr::null_mut(),
            },
            state: std::ptr::null_mut(),
            format: crate::htslib_rs::hts::htsFormat {
                category: 0,
                format: 0,
                version: crate::htslib_rs::hts::htsFormatVersion { major: 0, minor: 0 },
                compression: 0,
                compression_level: 0,
                specific: std::ptr::null_mut(),
            },
            idx: std::ptr::null_mut(),
            fnidx: std::ptr::null(),
            bam_header: std::ptr::null_mut(),
            filter: std::ptr::null_mut(),
        };
        let mut record = bam1_t {
            core: bam1_core_t {
                pos: 0,
                tid: 0,
                bin: 0,
                qual: 0,
                l_extranul: 0,
                flag: 0,
                l_qname: 0,
                n_cigar: 0,
                l_qseq: 0,
                mtid: 0,
                mpos: 0,
                isize: 0,
            },
            id: 0,
            data: Vec::new(),
            mempolicy_and_reserved: 0,
        };

        unsafe {
            assert_eq!(sam_itr_next(&mut fp, std::ptr::null_mut(), &mut record), -2);
            fp.bitfields = 1 << 4;
            assert_eq!(sam_itr_next(&mut fp, std::ptr::null_mut(), &mut record), -2);
        }
    }

    #[test]
    fn sam_region_iterator_entry_points_reject_null_inputs() {
        unsafe {
            assert!(sam_c_1768_sam_itr_regarray(
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
            .is_null());
            assert!(sam_c_1798_sam_itr_regions(
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
            .is_null());
            assert_eq!(
                sam_c_1754_cram_name2id(std::ptr::null_mut(), c"chr1".as_ptr()),
                -1
            );
        }
    }

    #[test]
    fn sam_thread_option_wrappers_match_simple_edges() {
        unsafe {
            assert_eq!(sam_c_3746_sam_set_threads(std::ptr::null_mut(), 0), 0);
            assert_eq!(sam_c_3746_sam_set_threads(std::ptr::null_mut(), -1), 0);
            assert_eq!(sam_c_3746_sam_set_threads(std::ptr::null_mut(), 1), -1);
            assert_eq!(
                sam_c_3719_sam_set_thread_pool(std::ptr::null_mut(), std::ptr::null_mut()),
                -1
            );

            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            assert_eq!(
                sam_c_3719_sam_set_thread_pool(
                    &mut fp,
                    (&mut crate::htslib_rs::hts::htsThreadPool {
                        pool: std::ptr::null_mut(),
                        qsize: 0,
                    }) as *mut crate::htslib_rs::hts::htsThreadPool,
                ),
                -1
            );
            let mut pool_marker =
                std::mem::MaybeUninit::<crate::htslib_rs::thread_pool::HtsTpool>::uninit();
            assert_eq!(
                sam_c_3719_sam_set_thread_pool(
                    &mut fp,
                    (&mut crate::htslib_rs::hts::htsThreadPool {
                        pool: pool_marker.as_mut_ptr(),
                        qsize: 0,
                    }) as *mut crate::htslib_rs::hts::htsThreadPool,
                ),
                0
            );
            let mut state_marker = 0u8;
            fp.state = (&mut state_marker as *mut u8).cast();
            assert_eq!(
                sam_c_3719_sam_set_thread_pool(
                    &mut fp,
                    (&mut crate::htslib_rs::hts::htsThreadPool {
                        pool: std::ptr::null_mut(),
                        qsize: 0,
                    }) as *mut crate::htslib_rs::hts::htsThreadPool,
                ),
                -2
            );
        }
    }

    #[test]
    fn sam_hdr_tid_accessors_match_htslib_field_order() {
        let chr1 = b"chr1\0";
        let chr2 = b"chr2\0";
        let mut target_lens = [100u32, 200u32];
        let mut target_names = [chr1.as_ptr() as *mut u8, chr2.as_ptr() as *mut u8];
        let mut hdr = sam_hdr_t {
            n_targets: 2,
            ignore_sam_err: 0,
            l_text: 0,
            target_len: target_lens.as_mut_ptr(),
            cigar_tab: std::ptr::null(),
            target_name: target_names.as_mut_ptr(),
            text: std::ptr::null_mut(),
            sdict: std::ptr::null_mut(),
            hrecs: std::ptr::null_mut(),
            ref_count: 1,
        };

        unsafe {
            assert_eq!(sam_hdr_name2tid(&mut hdr, CStr::from_bytes_with_nul(chr1).unwrap().to_bytes()), 0);
            assert_eq!(bam_name2id(&mut hdr, chr1.as_ptr().cast()), 0);
            assert_eq!(sam_hdr_name2tid(&mut hdr, CStr::from_bytes_with_nul(chr2).unwrap().to_bytes()), 1);
            assert_eq!(sam_hdr_tid2name(&hdr, -1), std::ptr::null());
            assert_eq!(sam_hdr_tid2name(&hdr, 0), chr1.as_ptr().cast());
            assert_eq!(sam_hdr_tid2name(&hdr, 1), chr2.as_ptr().cast());
            assert_eq!(sam_hdr_tid2name(&hdr, 2), std::ptr::null());
            assert_eq!(sam_hdr_tid2len(&hdr, -1), 0);
            assert_eq!(sam_hdr_tid2len(&hdr, 0), 100);
            assert_eq!(sam_hdr_tid2len(&hdr, 1), 200);
            assert_eq!(sam_hdr_tid2len(&hdr, 2), 0);
            assert_eq!(sam_hdr_name2tid(&mut hdr, b"missing"), -1);

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let rest = sam_parse_region(
                &mut hdr,
                c"chr2:11-20".as_ptr().cast(),
                &mut tid,
                &mut beg,
                &mut end,
                0,
            );
            assert!(!rest.is_null());
            assert_eq!(*rest, 0);
            assert_eq!(tid, 1);
            assert_eq!(beg, 10);
            assert_eq!(end, 20);
        }

        let alt = b"alt\0";
        let mut ref_hash_keys = [alt.as_ptr() as *mut u8];
        let mut ref_hash_vals = [0 as i32];
        let mut ref_hash_flags = [0u32];
        let mut ref_hash = khash_m_s2i_t {
            n_buckets: 1,
            size: 1,
            n_occupied: 1,
            upper_bound: 1,
            flags: ref_hash_flags.as_mut_ptr(),
            keys: ref_hash_keys.as_mut_ptr(),
            vals: ref_hash_vals.as_mut_ptr(),
        };
        let mut refs = [sam_hrec_sq_t {
            name: alt.as_ptr().cast(),
            len: 999,
            ty: std::ptr::null_mut(),
        }];
        let mut hrecs = sam_hrecs_t {
            h: std::ptr::null_mut(),
            first_line: std::ptr::null_mut(),
            str_pool: std::ptr::null_mut(),
            type_pool: std::ptr::null_mut(),
            tag_pool: std::ptr::null_mut(),
            nref: 1,
            ref_sz: 1,
            ref_: refs.as_mut_ptr(),
            ref_hash: (&mut ref_hash as *mut khash_m_s2i_t).cast(),
            nrg: 0,
            rg_sz: 0,
            rg: std::ptr::null_mut(),
            rg_hash: std::ptr::null_mut(),
            npg: 0,
            pg_sz: 0,
            npg_end: 0,
            npg_end_alloc: 0,
            pg: std::ptr::null_mut(),
            pg_hash: std::ptr::null_mut(),
            pg_end: std::ptr::null_mut(),
            ID_buf: std::ptr::null_mut(),
            ID_buf_sz: 0,
            ID_cnt: 0,
            dirty: 0,
            refs_changed: 0,
            pgs_changed: 0,
            type_count: 0,
            type_order: std::ptr::null_mut(),
        };
        hdr.hrecs = &mut hrecs;

        unsafe {
            assert_eq!(sam_hdr_name2tid(&mut hdr, CStr::from_bytes_with_nul(alt).unwrap().to_bytes()), 0);
            assert_eq!(sam_hdr_tid2name(&hdr, 0), alt.as_ptr().cast());
            assert_eq!(sam_hdr_tid2len(&hdr, 0), 999);
            assert_eq!(sam_hdr_tid2name(&hdr, 1), chr2.as_ptr().cast());
            assert_eq!(sam_hdr_tid2len(&hdr, 1), 200);
        }

        hdr.hrecs = std::ptr::null_mut();
        let mut long_target_lens = [u32::MAX, 200u32];
        hdr.target_len = long_target_lens.as_mut_ptr();
        let long_len = (u32::MAX as i64) + 10;
        let mut sdict_keys = [chr1.as_ptr() as *mut u8];
        let mut sdict_vals = [long_len];
        let mut sdict_flags = [0u32];
        let mut sdict = khash_s2i_t {
            n_buckets: 1,
            size: 1,
            n_occupied: 1,
            upper_bound: 1,
            flags: sdict_flags.as_mut_ptr(),
            keys: sdict_keys.as_mut_ptr(),
            vals: sdict_vals.as_mut_ptr(),
        };
        hdr.sdict = (&mut sdict as *mut khash_s2i_t).cast();

        unsafe {
            assert_eq!(sam_hdr_tid2len(&hdr, 0), long_len as hts_pos_t);
        }
    }

    #[test]
    fn sam_hdr_destroy_refcount_prefix_matches_htslib_without_freeing() {
        let mut hdr = sam_hdr_t {
            n_targets: 0,
            ignore_sam_err: 0,
            l_text: 0,
            target_len: std::ptr::null_mut(),
            cigar_tab: std::ptr::null(),
            target_name: std::ptr::null_mut(),
            text: std::ptr::null_mut(),
            sdict: std::ptr::null_mut(),
            hrecs: std::ptr::null_mut(),
            ref_count: 2,
        };

        unsafe {
            sam_hdr_destroy(std::ptr::null_mut());
            sam_hdr_destroy(&mut hdr);
            assert_eq!(hdr.ref_count, 1);
            sam_hdr_destroy(&mut hdr);
            assert_eq!(hdr.ref_count, 0);
        }
    }

    #[test]
    fn sam_hdr_destroy_frees_simple_c_allocated_header() {
        unsafe {
            let hdr =
                Box::into_raw(Box::new(std::mem::zeroed::<sam_hdr_t>())).cast::<()>()
                    .cast::<sam_hdr_t>();
            assert!(!hdr.is_null());
            (*hdr).n_targets = 1;
            (*hdr).target_name =
                ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); 1]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
                    .cast();
            (*hdr).target_len =
                Box::into_raw(Box::new(std::mem::zeroed::<u32>())).cast::<()>().cast();
            (*hdr).text = ({ let mut v: Vec<u8> = vec![0u8; (5) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            let name = { let mut v: Vec<u8> = vec![0u8; (5) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
            assert!(!(*hdr).target_name.is_null());
            assert!(!(*hdr).target_len.is_null());
            assert!(!(*hdr).text.is_null());
            assert!(!name.is_null());
            std::ptr::copy_nonoverlapping(c"chr1".as_ptr().cast::<u8>(), name, 5);
            std::ptr::copy_nonoverlapping(c"@HD\n".as_ptr().cast::<u8>(), (*hdr).text, 5);
            *(*hdr).target_name = name;
            *(*hdr).target_len = 100;

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_dup_sdict_copies_only_long_reference_entries() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            let missing = CString::new("missing").unwrap();
            let mut src_flags = [0xaaaa_aaaau32];
            let mut src_keys = [std::ptr::null_mut::<u8>(); 4];
            let mut src_vals = [0i64; 4];
            let chr1_bucket = __ac_FNV1a_hash_string(chr1.as_ptr().cast()) & 3;
            src_keys[chr1_bucket as usize] = chr1.as_ptr() as *mut u8;
            src_vals[chr1_bucket as usize] = (u32::MAX as i64) + 42;
            kh_set_occupied(src_flags.as_mut_ptr(), chr1_bucket);
            let mut src = khash_s2i_t {
                n_buckets: 4,
                size: 1,
                n_occupied: 1,
                upper_bound: 3,
                flags: src_flags.as_mut_ptr(),
                keys: src_keys.as_mut_ptr(),
                vals: src_vals.as_mut_ptr(),
            };

            let h0 = sam_hdr_t {
                n_targets: 0,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: std::ptr::null_mut(),
                cigar_tab: std::ptr::null(),
                target_name: std::ptr::null_mut(),
                text: std::ptr::null_mut(),
                sdict: (&mut src as *mut khash_s2i_t).cast(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let mut lens = [u32::MAX, u32::MAX, 20u32];
            let mut names = [
                chr1.as_ptr() as *mut u8,
                missing.as_ptr() as *mut u8,
                chr2.as_ptr() as *mut u8,
            ];
            let mut h = sam_hdr_t {
                n_targets: 3,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: lens.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: names.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };

            assert_eq!(sam_c_144_sam_hdr_dup_sdict(&h0, &mut h), 0);
            let dest = h.sdict.cast::<khash_s2i_t>();
            assert!(!dest.is_null());
            assert_eq!((*dest).size, 1);
            let k = kh_get_s2i(dest, chr1.as_ptr().cast());
            assert_ne!(k, (*dest).n_buckets);
            assert_eq!(*(*dest).vals.add(k as usize), (u32::MAX as i64) + 42);
            assert_eq!(kh_get_s2i(dest, missing.as_ptr().cast()), (*dest).n_buckets);
            assert_eq!(kh_get_s2i(dest, chr2.as_ptr().cast()), (*dest).n_buckets);
            kh_destroy_s2i(dest);
        }
    }

    #[test]
    fn sam_hdr_dup_copies_simple_long_reference_dictionary() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            let mut target_len = [u32::MAX, 200u32];
            let mut target_name = [chr1.as_ptr() as *mut u8, chr2.as_ptr() as *mut u8];
            let mut flags = [0xaaaa_aaaau32];
            let mut keys = [std::ptr::null_mut::<u8>(); 4];
            let mut vals = [0i64; 4];
            let bucket = __ac_FNV1a_hash_string(chr1.as_ptr().cast()) & 3;
            keys[bucket as usize] = chr1.as_ptr() as *mut u8;
            vals[bucket as usize] = (u32::MAX as i64) + 99;
            kh_set_occupied(flags.as_mut_ptr(), bucket);
            let mut sdict = khash_s2i_t {
                n_buckets: 4,
                size: 1,
                n_occupied: 1,
                upper_bound: 3,
                flags: flags.as_mut_ptr(),
                keys: keys.as_mut_ptr(),
                vals: vals.as_mut_ptr(),
            };
            let h0 = sam_hdr_t {
                n_targets: 2,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: target_len.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: target_name.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: (&mut sdict as *mut khash_s2i_t).cast(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };

            let dup = sam_hdr_dup(&h0);
            assert!(!dup.is_null());
            assert_ne!((*dup).target_name, h0.target_name);
            assert_ne!(*(*dup).target_name, *h0.target_name);
            assert_eq!(sam_hdr_tid2len(&*dup, 0), (u32::MAX as hts_pos_t) + 99);
            assert_eq!(sam_hdr_tid2len(&*dup, 1), 200);
            sam_hdr_destroy(dup);
        }
    }

    #[test]
    fn sam_hdr_change_hd_updates_text_header_like_htslib() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let text = b"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(sam_hdr_add_lines(&mut *hdr, text), 0);

            assert_eq!(
                sam_hdr_change_HD(hdr, c"SO".as_ptr().cast(), c"unsorted".as_ptr().cast()),
                0
            );
            // sam_hdr_str triggers rebuild from hrecs (the cached text is
            // redacted on every mutation now that header writes go through the
            // hrecs path).
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chr1\tLN:10\n"
            );

            assert_eq!(sam_hdr_change_HD(hdr, c"GO".as_ptr().cast(), c"query".as_ptr().cast()), 0);
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.6\tSO:unsorted\tGO:query\n@SQ\tSN:chr1\tLN:10\n"
            );

            assert_eq!(sam_hdr_change_HD(hdr, c"SO".as_ptr().cast(), std::ptr::null()), 0);
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.6\tGO:query\n@SQ\tSN:chr1\tLN:10\n"
            );
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_change_hd_adds_missing_hd_line_like_htslib() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, b"@SQ\tSN:chr1\tLN:10\n"),
                0
            );
            assert_eq!(
                sam_hdr_change_HD(hdr, c"SO".as_ptr().cast(), c"coordinate".as_ptr().cast()),
                0
            );
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()).to_bytes(),
                b"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:10\n"
            );
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_set_get_and_incr_ref_follow_htslib_ownership_rules() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, b"@SQ\tSN:chr1\tLN:10\n"),
                0
            );
            let mut fp = crate::htslib_rs::hts::htsFile {
                bitfields: 0,
                padding_0: 0,
                lineno: 0,
                line: crate::htslib_rs::hts::kstring_t::default(),
                fn_: std::ptr::null_mut(),
                fn_aux: std::ptr::null_mut(),
                fp: crate::htslib_rs::hts::htsFilePtr {
                    bgzf: std::ptr::null_mut(),
                },
                state: std::ptr::null_mut(),
                format: crate::htslib_rs::hts::htsFormat {
                    category: 0,
                    format: 0,
                    version: crate::htslib_rs::hts::htsFormatVersion { major: 0, minor: 0 },
                    compression: 0,
                    compression_level: 0,
                    specific: std::ptr::null_mut(),
                },
                idx: std::ptr::null_mut(),
                fnidx: std::ptr::null(),
                bam_header: std::ptr::null_mut(),
                filter: std::ptr::null_mut(),
            };

            assert_eq!(sam_hdr_set(&mut fp, hdr, 0), 0);
            assert_eq!(sam_hdr_get(&mut fp), hdr);
            assert_eq!((*hdr).ref_count, 1);

            assert_eq!(sam_hdr_set(&mut fp, hdr, 1), 0);
            let dup = sam_hdr_get(&mut fp);
            assert!(!dup.is_null());
            assert_ne!(dup, hdr);
            // Use sam_hdr_str to rebuild text from hrecs on each header. The
            // cached `(*hdr).text` is redacted after every mutation now that
            // the hrecs path is the canonical one, so a direct read would
            // null-deref.
            assert_eq!(
                CStr::from_ptr(sam_hdr_str(&mut *hdr).cast()),
                CStr::from_ptr(sam_hdr_str(&mut *dup).cast())
            );

            sam_hdr_destroy(hdr);
            sam_hdr_destroy(dup);
        }
    }

    #[test]
    fn bam_get_library_finds_rg_library_from_aux_tag() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let header_text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:10\n@RG\tID:rg1\tLB:lib_a\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, header_text),
                0
            );

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [30u8, 30, 30, 30];
            assert_eq!(
                bam_set1(
                    b,
                    4,
                    c"read".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert_eq!(
                bam_aux_append(b, c"RG".as_ptr().cast(), b'Z' as u8, 4, c"rg1".as_ptr().cast()),
                0
            );

            let library = sam_c_1173_bam_get_library(hdr, b);
            assert!(!library.is_null());
            assert_eq!(CStr::from_ptr(library.cast()).to_bytes(), b"lib_a");

            bam_destroy1(b);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_parse_b_vals_writes_integer_arrays_and_rescues_signed_overflow() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let input = CString::new(",1,2,255").unwrap();
            let mut end: *mut u8 = std::ptr::null_mut();
            assert_eq!(
                sam_c_2490_sam_parse_B_vals(b'C' as u8, input.as_ptr().cast::<u8>().cast_mut(), &mut end, b,),
                0
            );
            let data = &(*b).data;
            assert_eq!(data[0], b'B');
            assert_eq!(data[1], b'C');
            assert_eq!(bam_auxB_len(data.as_ptr()), 3);
            assert_eq!(bam_auxB2i(data.as_ptr(), 0), 1);
            assert_eq!(bam_auxB2i(data.as_ptr(), 2), 255);
            assert_eq!(*end, 0);

            (*b).data.clear();
            let input = CString::new(",-1,2").unwrap();
            assert_eq!(
                sam_c_2490_sam_parse_B_vals(b'C' as u8, input.as_ptr().cast::<u8>().cast_mut(), &mut end, b,),
                0
            );
            let data = &(*b).data;
            assert_eq!(data[0], b'B');
            assert_eq!(data[1], b'c');
            assert_eq!(bam_auxB_len(data.as_ptr()), 2);
            assert_eq!(bam_auxB2i(data.as_ptr(), 0), -1);
            assert_eq!(bam_auxB2i(data.as_ptr(), 1), 2);
            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_parse_b_vals_writes_float_arrays() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let input = CString::new(",1.5,-2.25").unwrap();
            let mut end: *mut u8 = std::ptr::null_mut();
            assert_eq!(
                sam_c_2490_sam_parse_B_vals(b'f' as u8, input.as_ptr().cast::<u8>().cast_mut(), &mut end, b,),
                0
            );
            let data = &(*b).data;
            assert_eq!(data[0], b'B');
            assert_eq!(data[1], b'f');
            assert_eq!(bam_auxB_len(data.as_ptr()), 2);
            assert_eq!(bam_auxB2f(data.as_ptr(), 0), 1.5);
            assert_eq!(bam_auxB2f(data.as_ptr(), 1), -2.25);
            assert_eq!(*end, 0);
            bam_destroy1(b);
        }
    }

    #[test]
    fn aux_parse_writes_numeric_string_array_and_lenient_fields() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            let input =
                CString::new("NM:i:7\tAS:i:-3\tCB:Z:cell-1\tHX:H:0a0B\tXA:A:z\tBF:B:f,1.5,-2")
                    .unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    input.as_ptr().cast::<u8>().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    0,
                    std::ptr::null_mut(),
                ),
                0
            );

            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(*nm, b'C');
            assert_eq!(bam_aux2i(nm), 7);
            let as_ = bam_aux_get(b, c"AS".as_ptr().cast());
            assert!(!as_.is_null());
            assert_eq!(*as_, b'c');
            assert_eq!(bam_aux2i(as_), -3);
            let cb = bam_aux_get(b, c"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb).cast()).to_bytes(), b"cell-1");
            let hx = bam_aux_get(b, c"HX".as_ptr().cast());
            assert!(!hx.is_null());
            assert_eq!(*hx, b'H');
            assert_eq!(CStr::from_ptr(bam_aux2Z(hx).cast()).to_bytes(), b"0a0B");
            let xa = bam_aux_get(b, c"XA".as_ptr().cast());
            assert!(!xa.is_null());
            assert_eq!(bam_aux2A(xa), b'z' as u8);
            let bf = bam_aux_get(b, c"BF".as_ptr().cast());
            assert!(!bf.is_null());
            assert_eq!(bam_auxB_len(bf), 2);
            assert_eq!(bam_auxB2f(bf, 0), 1.5);
            assert_eq!(bam_auxB2f(bf, 1), -2.0);

            (*b).data.clear();
            let input = CString::new("bad\tNM:i:8").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    input.as_ptr().cast::<u8>().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    1,
                    std::ptr::null_mut(),
                ),
                0
            );
            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 8);

            (*b).data.clear();
            let input = CString::new("NM:i:9\tCB:Z:drop").unwrap();
            let mut flags = [0xaaaa_aaaau32];
            let mut keys = [0i32; 4];
            let bucket = __ac_Wang_hash((b'N' as i32 * 256 + b'M' as i32) as u32) & 3;
            keys[bucket as usize] = b'N' as i32 * 256 + b'M' as i32;
            kh_set_occupied(flags.as_mut_ptr(), bucket);
            let mut tags = khash_tag_t {
                n_buckets: 4,
                size: 1,
                n_occupied: 1,
                upper_bound: 3,
                flags: flags.as_mut_ptr(),
                keys: keys.as_mut_ptr(),
            };
            assert_eq!(
                sam_c_2524_aux_parse(
                    input.as_ptr().cast::<u8>().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    0,
                    (&mut tags as *mut khash_tag_t).cast(),
                ),
                0
            );
            assert!(!bam_aux_get(b, c"NM".as_ptr().cast()).is_null());
            assert!(bam_aux_get(b, c"CB".as_ptr().cast()).is_null());

            let input = CString::new("HX:H:abc").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    input.as_ptr().cast::<u8>().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    0,
                    std::ptr::null_mut(),
                ),
                -2
            );

            bam_destroy1(b);
        }
    }

    #[test]
    fn aux_parse_strict_and_lenient_malformed_fields_match_htslib() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            let strict = CString::new("NM:i:1\tbad\tAS:i:2").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    strict.as_ptr().cast::<u8>().cast_mut(),
                    strict.as_ptr().add(strict.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    0,
                    std::ptr::null_mut(),
                ),
                -2
            );

            (*b).data.clear();
            let lenient = CString::new("NM:i:1\tbad\tAS:i:2").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    lenient.as_ptr().cast::<u8>().cast_mut(),
                    lenient.as_ptr().add(lenient.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    1,
                    std::ptr::null_mut(),
                ),
                0
            );
            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 1);
            let as_ = bam_aux_get(b, c"AS".as_ptr().cast());
            assert!(!as_.is_null());
            assert_eq!(bam_aux2i(as_), 2);

            (*b).data.clear();
            let odd_hex = CString::new("HX:H:abc\tNM:i:3").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    odd_hex.as_ptr().cast::<u8>().cast_mut(),
                    odd_hex.as_ptr().add(odd_hex.as_bytes().len()).cast::<u8>().cast_mut(),
                    b,
                    1,
                    std::ptr::null_mut(),
                ),
                0
            );
            assert!(bam_aux_get(b, c"HX".as_ptr().cast()).is_null());
            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 3);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_parse1_builds_bam_record_with_core_sequence_quality_and_aux() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut u8];
            let mut target_lens = [100u32];
            let mut hdr = sam_hdr_t {
                n_targets: 1,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: target_lens.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: target_names.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let b = bam_init1();
            assert!(!b.is_null());

            let line = b"read1\t0\tchr1\t2\t60\t4M\t*\t0\t0\tACGT\t!!!!\tNM:i:1\tCB:Z:cell";
            let mut ks = kstring_t {
                data: line.to_vec(),
            };
            assert_eq!(sam_c_2662_sam_parse1(&mut ks, &mut hdr, b), 0);

            assert_eq!(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), b"read1");
            assert_eq!((*b).core.flag as i32 & BAM_FUNMAP, 0);
            assert_eq!((*b).core.tid, 0);
            assert_eq!((*b).core.pos, 1);
            assert_eq!((*b).core.qual, 60);
            assert_eq!((*b).core.n_cigar, 1);
            assert_eq!(bam_cigar_oplen(*bam_get_cigar(b)), 4);
            assert_eq!(bam_cigar_op(*bam_get_cigar(b)), BAM_CMATCH);
            assert_eq!((*b).core.mtid, -1);
            assert_eq!((*b).core.mpos, -1);
            assert_eq!((*b).core.l_qseq, 4);
            assert_eq!(bam_seqi(bam_get_seq(b), 0), 1);
            assert_eq!(bam_seqi(bam_get_seq(b), 1), 2);
            assert_eq!(bam_seqi(bam_get_seq(b), 2), 4);
            assert_eq!(bam_seqi(bam_get_seq(b), 3), 8);
            assert_eq!(*bam_get_qual(b), 0);
            let nm = bam_aux_get(b, c"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 1);
            let cb = bam_aux_get(b, c"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb).cast()).to_bytes(), b"cell");

            let mut out = kstring_t::default();
            assert!(sam_format1(&hdr, b, &mut out) > 0);
            assert!(out
                .data
                .starts_with(b"read1\t0\tchr1\t2\t60\t4M"));
            ks_free(&mut out);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_read1_sam_consumes_buffered_line_after_header_parse() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut u8];
            let mut target_lens = [100u32];
            let mut hdr = sam_hdr_t {
                n_targets: 1,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: target_lens.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: target_names.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let b = bam_init1();
            assert!(!b.is_null());
            let line = b"read2\t4\t*\t0\t0\t*\t*\t0\t0\t*\t*\tZZ:Z:tag";
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            // htsFile embeds an owned kstring (Vec); the zeroed Vec is invalid,
            // so initialize it with ptr::write to avoid dropping the zeroed one.
            std::ptr::write(&mut fp.line, kstring_t { data: line.to_vec() });
            fp.format.format = HTS_FORMAT_SAM;

            assert_eq!(sam_read1(&mut fp, &mut hdr, b), 0);
            assert_eq!(fp.line.data.len(), 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b).cast()).to_bytes(), b"read2");
            assert_ne!((*b).core.flag as i32 & BAM_FUNMAP, 0);
            assert_eq!((*b).core.tid, -1);
            assert_eq!((*b).core.l_qseq, 0);
            let zz = bam_aux_get(b, c"ZZ".as_ptr().cast());
            assert!(!zz.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(zz).cast()).to_bytes(), b"tag");

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_write1_writes_sam_text_and_promotes_binary_bam() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut u8];
            let mut target_lens = [100u32];
            let hdr = sam_hdr_t {
                n_targets: 1,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: target_lens.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: target_names.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            let seq = CString::new("ACGT").unwrap();
            let qual = [0u8, 1, 2, 3];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"read1".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    4,
                    seq.as_ptr().cast(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );

            let path = std::env::temp_dir().join(format!(
                "htslib_rs-sam-write1-{}-{}.sam.gz",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.bitfields = 1 << 4;
            fp.fp = crate::htslib_rs::hts::htsFilePtr { bgzf };
            fp.format.category = HTS_FORMAT_SEQUENCE_DATA;
            fp.format.format = HTS_FORMAT_SAM;
            let written = sam_c_4553_sam_write1(&mut fp, &hdr, b);
            assert!(written > 0);
            assert!(
                fp.line.data.as_slice()
                    .starts_with(b"read1\t0\tchr1\t1\t60\t4M")
            );
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);

            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let mut buf = [0u8; 128];
            let n = crate::htslib_rs::bgzf::bgzf_read(bgzf, buf.as_mut_ptr().cast(), buf.len());
            assert!(n > 0);
            assert!(buf[..n as usize].starts_with(b"read1\t0\tchr1\t1\t60\t4M"));
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);
            let _ = std::fs::remove_file(&path);
            ks_free(&mut fp.line);

            let path = std::env::temp_dir().join(format!(
                "htslib_rs-binary-write1-{}-{}.bam",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut fp: htsFile = htsFile::default();
            std::ptr::write(&mut fp.line, kstring_t::default());
            fp.fp = crate::htslib_rs::hts::htsFilePtr { bgzf };
            fp.format.format = HTS_FORMAT_BINARY_FORMAT;
            assert!(sam_c_4553_sam_write1(&mut fp, &hdr, b) > 0);
            assert_eq!(fp.format.category, HTS_FORMAT_SEQUENCE_DATA);
            assert_eq!(fp.format.format, HTS_FORMAT_BAM);
            assert_eq!(crate::htslib_rs::bgzf::bgzf_close(bgzf), 0);
            let _ = std::fs::remove_file(&path);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_parse1_and_format1_preserve_equal_mate_and_missing_quality() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut u8];
            let mut target_lens = [100u32];
            let mut hdr = sam_hdr_t {
                n_targets: 1,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: target_lens.as_mut_ptr(),
                cigar_tab: std::ptr::null(),
                target_name: target_names.as_mut_ptr(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let b = bam_init1();
            assert!(!b.is_null());

            let line = b"read_eq\t99\tchr1\t5\t0\t2M\t=\t7\t2\tAC\t*\tNM:i:0";
            let mut ks = kstring_t {
                data: line.to_vec(),
            };
            assert_eq!(sam_c_2662_sam_parse1(&mut ks, &mut hdr, b), 0);
            assert_eq!((*b).core.tid, 0);
            assert_eq!((*b).core.pos, 4);
            assert_eq!((*b).core.mtid, 0);
            assert_eq!((*b).core.mpos, 6);
            assert_eq!((*b).core.isize, 2);
            assert_eq!(*bam_get_qual(b), 0xff);

            let mut out = kstring_t::default();
            assert!(sam_format1(&hdr, b, &mut out) > 0);
            assert_eq!(
                out.data.as_slice(),
                b"read_eq\t99\tchr1\t5\t0\t2M\t=\t7\t2\tAC\t*\tNM:i:0"
            );
            ks_free(&mut out);
            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_format1_formats_unmapped_empty_sequence_like_htslib() {
        unsafe {
            let hdr = sam_hdr_t {
                n_targets: 0,
                ignore_sam_err: 0,
                l_text: 0,
                target_len: std::ptr::null_mut(),
                cigar_tab: std::ptr::null(),
                target_name: std::ptr::null_mut(),
                text: std::ptr::null_mut(),
                sdict: std::ptr::null_mut(),
                hrecs: std::ptr::null_mut(),
                ref_count: 0,
            };
            let b = bam_init1();
            assert!(!b.is_null());
            assert_eq!(
                bam_set1(
                    b,
                    1,
                    c"r".as_ptr().cast(),
                    BAM_FUNMAP as u16,
                    -1,
                    -1,
                    0,
                    0,
                    std::ptr::null(),
                    -1,
                    -1,
                    0,
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                ),
                4
            );

            let mut out = kstring_t::default();
            assert!(sam_format1(&hdr, b, &mut out) > 0);
            assert_eq!(
                out.data.as_slice(),
                b"r\t4\t*\t0\t0\t*\t*\t0\t0\t*\t*"
            );
            ks_free(&mut out);
            bam_destroy1(b);
        }
    }

    #[test]
    fn aux_get_str_formats_character_hex_and_rejects_b_a_arrays_like_htslib() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            let achar = [b'Z'];
            assert_eq!(
                bam_aux_append(b, c"AC".as_ptr().cast(), b'A' as u8, 1, achar.as_ptr()),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"HX".as_ptr().cast(),
                    b'H' as u8,
                    5,
                    c"0A0b".as_ptr().cast(),
                ),
                0
            );
            let array = [b'A', 2, 0, 0, 0, b'X', b'Y'];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"BA".as_ptr().cast(),
                    b'B' as u8,
                    array.len() as i32,
                    array.as_ptr(),
                ),
                0
            );

            let mut ks = kstring_t::default();
            assert_eq!(bam_aux_get_str(b, c"AC".as_ptr().cast(), &mut ks), 1);
            assert_eq!(ks.data.as_slice(), b"AC:A:Z");
            ks.data.clear();

            assert_eq!(bam_aux_get_str(b, c"HX".as_ptr().cast(), &mut ks), 1);
            assert_eq!(ks.data.as_slice(), b"HX:H:0A0b");
            ks.data.clear();

            assert_eq!(bam_aux_get_str(b, c"BA".as_ptr().cast(), &mut ks), -1);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            ks_free(&mut ks);
            bam_destroy1(b);
        }
    }

    #[test]
    fn parse_sam_flag_matches_decimal_zero_hex_and_overflow_rules() {
        unsafe {
            let mut end: *mut u8 = std::ptr::null_mut();
            let mut overflow = 0;
            let input = CString::new("16\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast::<u8>().cast_mut(), &mut end, &mut overflow),
                16
            );
            assert_eq!(*end, b'\t' as u8);
            assert_eq!(overflow, 0);

            let input = CString::new("0\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast::<u8>().cast_mut(), &mut end, &mut overflow),
                0
            );
            assert_eq!(*end, b'\t' as u8);

            let input = CString::new("010\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast::<u8>().cast_mut(), &mut end, &mut overflow),
                8
            );
            assert_eq!(*end, b'\t' as u8);

            let input = CString::new("0x10\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast::<u8>().cast_mut(), &mut end, &mut overflow),
                16
            );
            assert_eq!(*end, b'\t' as u8);

            overflow = 0;
            let input = CString::new("0200000\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast::<u8>().cast_mut(), &mut end, &mut overflow),
                65535
            );
            assert_eq!(overflow, 1);
        }
    }

    #[test]
    fn sam_hdr_destroy_frees_c_allocated_long_ref_hash() {
        unsafe {
            let hdr =
                Box::into_raw(Box::new(std::mem::zeroed::<sam_hdr_t>())).cast::<()>()
                    .cast::<sam_hdr_t>();
            assert!(!hdr.is_null());
            (*hdr).n_targets = 1;
            (*hdr).target_name =
                ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); 1]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
                    .cast();
            (*hdr).target_len =
                Box::into_raw(Box::new(std::mem::zeroed::<u32>())).cast::<()>().cast();
            let name = { let mut v: Vec<u8> = vec![0u8; (5) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p };
            assert!(!name.is_null());
            std::ptr::copy_nonoverlapping(c"chr1".as_ptr().cast::<u8>(), name, 5);
            *(*hdr).target_name = name;
            *(*hdr).target_len = u32::MAX;

            let sdict =
                Box::into_raw(Box::new(std::mem::zeroed::<khash_s2i_t>())).cast::<()>()
                    .cast::<khash_s2i_t>();
            assert!(!sdict.is_null());
            (*sdict).n_buckets = 1;
            (*sdict).size = 1;
            (*sdict).n_occupied = 1;
            (*sdict).upper_bound = 1;
            (*sdict).flags =
                Box::into_raw(Box::new(std::mem::zeroed::<u32>())).cast::<()>().cast();
            (*sdict).keys =
                ({ let mut v: Vec<*mut u8> = vec![std::ptr::null_mut(); 1]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast::<()>()
                    .cast();
            (*sdict).vals =
                Box::into_raw(Box::new(std::mem::zeroed::<i64>())).cast::<()>().cast();
            assert!(!(*sdict).flags.is_null());
            assert!(!(*sdict).keys.is_null());
            assert!(!(*sdict).vals.is_null());
            *(*sdict).keys = name;
            *(*sdict).vals = (u32::MAX as i64) + 123;
            (*hdr).sdict = sdict.cast();

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_line_index_reads_text_backed_header_without_hrecs() {
        unsafe {
            let text = c"@HD\tVN:1.6\n@SQ\tSN:ref1\tLN:10\n@SQ\tSN:ref2\tLN:20\n@RG\tID:run1\n@RG\tID:run2\n@PG\tID:prog1\n";
            let hdr = sam_hdr_parse(text.to_bytes().len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).hrecs.is_null());

            assert_eq!(sam_hdr_line_index(&mut *hdr, b"SQ", b"ref1"), 0);
            assert_eq!(sam_hdr_line_index(&mut *hdr, b"SQ", b"ref2"), 1);
            assert_eq!(sam_hdr_line_index(&mut *hdr, b"RG", b"run2"), 1);
            assert_eq!(
                sam_hdr_line_index(&mut *hdr, b"PG", b"prog1"),
                0
            );
            assert_eq!(
                sam_hdr_line_index(&mut *hdr, b"RG", b"missing"),
                -1
            );
            assert_eq!(
                sam_hdr_line_index(&mut *hdr, b"CO", b"anything"),
                -1
            );

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_init_matches_htslib_initial_state() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            assert_eq!((*hdr).n_targets, 0);
            assert_eq!((*hdr).ignore_sam_err, 0);
            assert_eq!((*hdr).l_text, 0);
            assert!((*hdr).target_len.is_null());
            assert_eq!((*hdr).cigar_tab, BAM_CIGAR_TABLE.as_ptr());
            assert!((*hdr).target_name.is_null());
            assert!((*hdr).text.is_null());
            assert!((*hdr).sdict.is_null());
            assert!((*hdr).hrecs.is_null());
            assert_eq!((*hdr).ref_count, 0);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_read_translated_simple_format_branches_match_htslib() {
        let mut fp = crate::htslib_rs::hts::htsFile {
            bitfields: 0,
            padding_0: 0,
            lineno: 0,
            line: crate::htslib_rs::hts::kstring_t::default(),
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: crate::htslib_rs::hts::htsFilePtr {
                bgzf: std::ptr::null_mut(),
            },
            state: std::ptr::null_mut(),
            format: crate::htslib_rs::hts::htsFormat {
                category: 0,
                format: HTS_FORMAT_FASTA_FORMAT,
                version: crate::htslib_rs::hts::htsFormatVersion { major: 0, minor: 0 },
                compression: 0,
                compression_level: 0,
                specific: std::ptr::null_mut(),
            },
            idx: std::ptr::null_mut(),
            fnidx: std::ptr::null(),
            bam_header: std::ptr::null_mut(),
            filter: std::ptr::null_mut(),
        };

        unsafe {
            assert!(sam_hdr_read(std::ptr::null_mut()).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            let hdr = sam_hdr_read(&mut fp);
            assert!(!hdr.is_null());
            assert_eq!((*hdr).n_targets, 0);
            sam_hdr_destroy(hdr);

            fp.format.format = HTS_FORMAT_EMPTY_FORMAT;
            assert!(sam_hdr_read(&mut fp).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EPIPE as i32
            );

            let mut record = bam1_t {
                core: bam1_core_t {
                    pos: 0,
                    tid: 0,
                    bin: 0,
                    qual: 0,
                    l_extranul: 0,
                    flag: 0,
                    l_qname: 0,
                    n_cigar: 0,
                    l_qseq: 0,
                    mtid: 0,
                    mpos: 0,
                    isize: 0,
                },
                id: 0,
                data: Vec::new(),
                mempolicy_and_reserved: 0,
            };
            assert_eq!(sam_read1(&mut fp, std::ptr::null_mut(), &mut record), -3);
            assert_eq!(
                *libc::__errno_location(),
                libc::EPIPE as i32
            );

            fp.format.format = crate::htslib_rs::hts::HTS_FORMAT_BINARY_FORMAT;
            assert_eq!(sam_read1(&mut fp, std::ptr::null_mut(), &mut record), -3);
            assert_eq!(
                *libc::__errno_location(),
                libc::ENOEXEC as i32
            );
        }
    }

    #[test]
    fn sam_hdr_read_sam_branch_reads_header_without_consuming_first_record() {
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-sam-header-{}-{}.sam",
            std::process::id(),
            line!()
        ));
        std::fs::write(
            &path,
            b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:20\nr1\t0\tchr1\t3\t60\t4M\t*\t0\t0\tACGT\tFFFF\n",
        )
        .unwrap();

        let path_c = CString::new(path.to_str().unwrap()).unwrap();
        unsafe {
            let fp = crate::htslib_rs::hts::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let hdr = sam_hdr_read(fp);
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 20);

            let b = bam_init1();
            assert!(!b.is_null());
            let ret = sam_read1(fp, hdr, b);
            assert!(ret >= 0);
            assert_eq!((*b).core.tid, 0);
            assert_eq!((*b).core.pos, 2);
            assert_eq!((*b).core.l_qseq, 4);
            bam_destroy1(b);

            sam_hdr_destroy(hdr);
            assert_eq!(crate::htslib_rs::hts::hts_close(fp), 0);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bam_read1_matches_htslib_on_generated_bam_record() {
        use std::process::Command;

        let dir = std::env::temp_dir().join(format!(
            "htslib_rs-bam-read1-{}-{}",
            std::process::id(),
            line!()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let sam_path = dir.join("input.sam");
        let bam_path = dir.join("input.bam");
        std::fs::write(
            &sam_path,
            b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:50\nr1\t0\tchr1\t3\t60\t4M\t*\t0\t0\tACGT\tFFFF\tCB:Z:cell1\tUB:Z:umi1\n",
        )
        .unwrap();

        let Ok(status) = Command::new("samtools")
            .arg("view")
            .arg("-b")
            .arg("-o")
            .arg(&bam_path)
            .arg(&sam_path)
            .status()
        else {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        };
        if !status.success() {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }
        let Ok(status) = Command::new("samtools")
            .arg("index")
            .arg(&bam_path)
            .status()
        else {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        };
        if !status.success() {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }

        let bam_c = CString::new(bam_path.to_str().unwrap()).unwrap();
        unsafe {
            let fp_rust = crate::htslib_rs::hts::hts_open(bam_c.as_ptr(), c"r".as_ptr());
            let fp_c = hts_sys::hts_open(bam_c.as_ptr(), c"r".as_ptr());
            assert!(!fp_rust.is_null());
            assert!(!fp_c.is_null());
            let hdr_rust = sam_hdr_read(fp_rust);
            let hdr_c = hts_sys::sam_hdr_read(fp_c);
            assert!(!hdr_rust.is_null());
            assert!(!hdr_c.is_null());

            let b_rust = bam_init1();
            // The C comparison record must be allocated by C (repr(C) layout);
            // the native bam1_t is no longer ABI-compatible with hts_sys, so a
            // `.cast()` of a Rust bam1_t into hts_sys would be unsound.
            let b_c = hts_sys::bam_init1();
            let ret_rust = bam_read1((*fp_rust).fp.bgzf, b_rust);
            let ret_c = hts_sys::bam_read1((*fp_c).fp.bgzf, b_c);
            assert_eq!(ret_rust, ret_c);
            assert!(ret_rust > 0);
            assert_eq!((*b_rust).core.tid, (*b_c).core.tid);
            assert_eq!((*b_rust).core.pos, (*b_c).core.pos);
            assert_eq!((*b_rust).core.bin, (*b_c).core.bin);
            assert_eq!((*b_rust).core.qual, (*b_c).core.qual);
            assert_eq!((*b_rust).core.flag, (*b_c).core.flag);
            assert_eq!((*b_rust).core.n_cigar, (*b_c).core.n_cigar);
            assert_eq!((*b_rust).core.l_qseq, (*b_c).core.l_qseq);
            assert_eq!((*b_rust).data.len() as i32, (*b_c).l_data);
            assert_eq!(
                (*b_rust).data.as_slice(),
                std::slice::from_raw_parts((*b_c).data, (*b_c).l_data as usize)
            );
            assert_eq!(bam_read1((*fp_rust).fp.bgzf, b_rust), -1);
            assert_eq!(hts_sys::bam_read1((*fp_c).fp.bgzf, b_c), -1);

            bam_destroy1(b_rust);
            hts_sys::bam_destroy1(b_c);
            sam_hdr_destroy(hdr_rust);
            hts_sys::sam_hdr_destroy(hdr_c);
            assert_eq!(crate::htslib_rs::hts::hts_close(fp_rust), 0);
            assert_eq!(hts_sys::hts_close(fp_c), 0);

            let fp_query = crate::htslib_rs::hts::hts_open(bam_c.as_ptr(), c"r".as_ptr());
            assert!(!fp_query.is_null());
            let hdr_query = sam_hdr_read(fp_query);
            assert!(!hdr_query.is_null());
            let idx = sam_index_load(fp_query, bam_c.as_ptr().cast());
            assert!(!idx.is_null());
            let itr = sam_itr_queryi(idx, 0, 0, 50);
            assert!(!itr.is_null());
            let b_query = bam_init1();
            assert!(sam_itr_next(fp_query, itr, b_query) >= 0);
            assert_eq!((*b_query).core.tid, 0);
            assert_eq!((*b_query).core.pos, 2);
            assert_eq!(sam_itr_next(fp_query, itr, b_query), -1);
            bam_destroy1(b_query);
            crate::htslib_rs::hts::hts_itr_destroy(itr);

            let itr = sam_itr_querys(idx, hdr_query, c"chr1:1-50".as_ptr().cast());
            assert!(!itr.is_null());
            let b_query = bam_init1();
            assert!(sam_itr_next(fp_query, itr, b_query) >= 0);
            assert_eq!((*b_query).core.tid, 0);
            assert_eq!((*b_query).core.pos, 2);
            assert_eq!(sam_itr_next(fp_query, itr, b_query), -1);
            bam_destroy1(b_query);
            crate::htslib_rs::hts::hts_itr_destroy(itr);

            assert!(sam_itr_querys(idx, hdr_query, c"missing:1-2".as_ptr().cast()).is_null());
            crate::htslib_rs::hts::hts_idx_destroy(idx);
            sam_hdr_destroy(hdr_query);
            assert_eq!(crate::htslib_rs::hts::hts_close(fp_query), 0);

            let bai_path = bam_path.with_extension("bam.bai");
            let bai_c = CString::new(bai_path.to_str().unwrap()).unwrap();
            let idx2 = crate::htslib_rs::hts::hts_idx_load2(bam_c.as_ptr(), bai_c.as_ptr());
            assert!(!idx2.is_null());
            crate::htslib_rs::hts::hts_idx_destroy(idx2);
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn sam_hdr_parse_uses_rust_init_and_simple_add_lines() {
        let text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:123\n@SQ\tSN:chr2\tLN:456\n";
        let chr1 = b"chr1\0";
        let chr2 = b"chr2\0";
        unsafe {
            let hdr = sam_hdr_parse(text.len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_length(&mut *hdr), text.len());
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), sam_hdr_length(&mut *hdr)),
                text
            );
            assert_eq!(sam_hdr_name2tid(&mut *hdr, CStr::from_bytes_with_nul(chr1).unwrap().to_bytes()), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, CStr::from_bytes_with_nul(chr2).unwrap().to_bytes()), 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 123);
            assert_eq!(sam_hdr_tid2len(&*hdr, 1), 456);
            assert_eq!(
                CStr::from_ptr(sam_hdr_tid2name(&*hdr, 0).cast()),
                CStr::from_bytes_with_nul(chr1).unwrap()
            );
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_parse_underscore_and_free_wrappers_parse_simple_header() {
        let text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:123\n";
        unsafe {
            let hdr = sam_hdr_parse_(text.as_ptr().cast(), text.len());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_nref(&*hdr), 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 123);
            sam_hdr_free(hdr);
        }
    }

    #[test]
    fn sam_hdr_parse_accepts_sq_tags_in_any_order_with_extra_fields() {
        let text = b"@HD\tVN:1.6\n@SQ\tLN:12\tSN:chr1\tAS:asm\r\n@SQ\tM5:abc\tSN:chr2\tLN:34\n";
        unsafe {
            let hdr = sam_hdr_parse(text.len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_nref(&*hdr), 2);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr2"), 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 12);
            assert_eq!(sam_hdr_tid2len(&*hdr, 1), 34);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 0).cast()).to_bytes(), b"chr1");
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 1).cast()).to_bytes(), b"chr2");
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn translated_header_type_helpers_match_htslib_boundaries() {
        unsafe {
            assert_eq!(header_h_58_TYPEKEY(c"HD".as_ptr().cast()), 0x4844);
            assert_eq!(header_h_58_TYPEKEY(c"SQ".as_ptr().cast()), 0x5351);

            assert_eq!(
                valid_sam_header_type(b"@HD\tVN:1.6"),
                1
            );
            assert_eq!(
                valid_sam_header_type(b"@SQ\tSN:chr1"),
                1
            );
            assert_eq!(
                valid_sam_header_type(b"@RG\tID:rg1"),
                1
            );
            assert_eq!(
                valid_sam_header_type(b"@PG\tID:pg1"),
                1
            );
            assert_eq!(
                valid_sam_header_type(b"@COcomment text"),
                1
            );
            assert_eq!(
                valid_sam_header_type(b"@HD VN:1.6"),
                0
            );
            assert_eq!(
                valid_sam_header_type(b"@SQ SN:chr1"),
                0
            );
            assert_eq!(
                valid_sam_header_type(b"@XX\tID:x"),
                0
            );
            assert_eq!(
                valid_sam_header_type(b"not-a-header"),
                0
            );
        }
    }

    #[test]
    fn sam_hdr_add_lines_accumulates_text_and_targets_from_new_lines_only() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            // Use canonical LF terminators: hrecs serialization always emits
            // bare \n (matching htslib's `sam_hrecs_rebuild_text`), so CR-LF
            // is normalized away during rebuild.
            let first = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, first),
                0
            );
            assert_eq!(sam_hdr_length(&mut *hdr), first.len());
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), first.len()),
                first
            );
            assert_eq!(*(*hdr).text.add((*hdr).l_text), 0);
            assert_eq!(sam_hdr_nref(&*hdr), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 10);

            let second = b"@RG\tID:rg1\n@SQ\tSN:chr2\tLN:20\n";
            assert_eq!(
                sam_hdr_add_lines(&mut *hdr, second),
                0
            );
            assert_eq!(sam_hdr_length(&mut *hdr), first.len() + second.len());
            assert_eq!(sam_hdr_nref(&*hdr), 2);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr2"), 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 1), 20);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 1).cast()).to_bytes(), b"chr2");

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_line_builds_text_backed_lines_without_hrecs() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"SQ",
                    &[
                        (Some(b"SN" as &[u8]), Some(b"chr1" as &[u8])),
                        (Some(b"LN" as &[u8]), Some(b"10" as &[u8]))
                    ],
                ),
                0
            );
            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"RG",
                    &[
                        (Some(b"ID" as &[u8]), Some(b"run1" as &[u8])),
                        (Some(b"SM" as &[u8]), Some(b"sample1" as &[u8]))
                    ],
                ),
                0
            );
            assert_eq!(
                sam_hdr_add_line(
                    &mut *hdr,
                    b"CO",
                    &[(Some(b"comment without tabs" as &[u8]), None)],
                ),
                0
            );

            // The SAM spec writes @CO lines as `@CO\t<text>\n` (tab between
            // type and free-text). htslib's C `sam_hdr_add_line` emits the
            // same shape; this test was previously asserting `@COcomment...`
            // (no tab), which silently matched a bug in our native CO branch
            // that has now been fixed to insert the spec-mandated tab.
            let expected =
                b"@SQ\tSN:chr1\tLN:10\n@RG\tID:run1\tSM:sample1\n@CO\tcomment without tabs\n";
            assert_eq!(sam_hdr_length(&mut *hdr), expected.len());
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_str(&mut *hdr).cast::<u8>(), expected.len()),
                expected
            );
            assert_eq!(sam_hdr_nref(&*hdr), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 10);
            let mut ks = kstring_t::default();
            assert_eq!(
                sam_hdr_find_line_id(
                    &mut *hdr,
                    b"RG",
                    b"ID",
                    b"run1",
                    &mut ks,
                ),
                0
            );

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_name2tid_fills_simple_target_arrays_from_text() {
        let text = b"@HD\tVN:1.6\n@SQ\tSN:chrA\tLN:11\n";
        let chr = b"chrA\0";
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            (*hdr).text = ({ let mut v: Vec<u8> = vec![0u8; (text.len() as u64 + 1) as usize]; let p = v.as_mut_ptr(); std::mem::forget(v); p }).cast();
            assert!(!(*hdr).text.is_null());
            std::ptr::copy_nonoverlapping(text.as_ptr(), (*hdr).text, (text.len()) as usize);
            *(*hdr).text.add(text.len()) = 0;
            (*hdr).l_text = text.len();

            assert_eq!((*hdr).n_targets, 0);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, CStr::from_bytes_with_nul(chr).unwrap().to_bytes()), 0);
            assert_eq!((*hdr).n_targets, 1);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 11);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"missing"), -1);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_nref_matches_htslib_target_count_selection() {
        let mut hdr = sam_hdr_t {
            n_targets: 2,
            ignore_sam_err: 0,
            l_text: 0,
            target_len: std::ptr::null_mut(),
            cigar_tab: std::ptr::null(),
            target_name: std::ptr::null_mut(),
            text: std::ptr::null_mut(),
            sdict: std::ptr::null_mut(),
            hrecs: std::ptr::null_mut(),
            ref_count: 0,
        };
        let mut hrecs = sam_hrecs_t {
            h: std::ptr::null_mut(),
            first_line: std::ptr::null_mut(),
            str_pool: std::ptr::null_mut(),
            type_pool: std::ptr::null_mut(),
            tag_pool: std::ptr::null_mut(),
            nref: 3,
            ref_sz: 0,
            ref_: std::ptr::null_mut(),
            ref_hash: std::ptr::null_mut(),
            nrg: 0,
            rg_sz: 0,
            rg: std::ptr::null_mut(),
            rg_hash: std::ptr::null_mut(),
            npg: 0,
            pg_sz: 0,
            npg_end: 0,
            npg_end_alloc: 0,
            pg: std::ptr::null_mut(),
            pg_hash: std::ptr::null_mut(),
            pg_end: std::ptr::null_mut(),
            ID_buf: std::ptr::null_mut(),
            ID_buf_sz: 0,
            ID_cnt: 0,
            dirty: 0,
            refs_changed: 0,
            pgs_changed: 0,
            type_count: 0,
            type_order: std::ptr::null_mut(),
        };

        unsafe {
            assert_eq!(sam_hdr_nref(&hdr), 2);
            hdr.hrecs = &mut hrecs;
            assert_eq!(sam_hdr_nref(&hdr), 3);
        }
    }

    #[test]
    fn sam_hdr_text_accessors_match_simple_header_fields() {
        let text = b"@HD\tVN:1.6\n\0";
        let mut hdr = sam_hdr_t {
            n_targets: 0,
            ignore_sam_err: 0,
            l_text: text.len() - 1,
            target_len: std::ptr::null_mut(),
            cigar_tab: std::ptr::null(),
            target_name: std::ptr::null_mut(),
            text: text.as_ptr().cast_mut().cast::<u8>(),
            sdict: std::ptr::null_mut(),
            hrecs: std::ptr::null_mut(),
            ref_count: 0,
        };

        unsafe {
            assert_eq!(sam_hdr_length(&mut hdr), text.len() - 1);
            assert_eq!(sam_hdr_str(&mut hdr), text.as_ptr().cast::<u8>());
        }
    }

    #[test]
    fn sam_hdr_dup_copies_simple_header_like_htslib() {
        let chr1 = b"chr1\0";
        let chr2 = b"chr2\0";
        let text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n\0";
        let mut target_len = [100u32, 200u32];
        let mut target_name = [
            chr1.as_ptr().cast_mut().cast::<u8>(),
            chr2.as_ptr().cast_mut().cast::<u8>(),
        ];
        let hdr = sam_hdr_t {
            n_targets: 2,
            ignore_sam_err: 7,
            l_text: text.len() - 1,
            target_len: target_len.as_mut_ptr(),
            cigar_tab: std::ptr::null(),
            target_name: target_name.as_mut_ptr(),
            text: text.as_ptr().cast_mut().cast::<u8>(),
            sdict: std::ptr::null_mut(),
            hrecs: std::ptr::null_mut(),
            ref_count: 0,
        };

        unsafe {
            assert!(sam_hdr_dup(std::ptr::null()).is_null());
            let dup = sam_hdr_dup(&hdr);
            assert!(!dup.is_null());
            assert_ne!((*dup).target_name, hdr.target_name);
            assert_ne!((*dup).target_len, hdr.target_len);
            assert_ne!((*dup).text, hdr.text);
            assert_eq!((*dup).n_targets, 2);
            assert_eq!((*dup).ignore_sam_err, 7);
            assert_eq!((*dup).l_text, hdr.l_text);
            assert_eq!(*(*dup).target_len, 100);
            assert_eq!(*(*dup).target_len.add(1), 200);
            assert_eq!(
                CStr::from_ptr((*(*dup).target_name).cast()),
                CStr::from_bytes_with_nul(chr1).unwrap()
            );
            assert_eq!(
                CStr::from_ptr((*(*dup).target_name.add(1)).cast()),
                CStr::from_bytes_with_nul(chr2).unwrap()
            );
            assert_eq!(
                CStr::from_ptr((*dup).text.cast()),
                CStr::from_bytes_with_nul(text).unwrap()
            );
            sam_hdr_destroy(dup);

            let alias = bam_hdr_init();
            assert!(!alias.is_null());
            bam_hdr_destroy(alias);
            let alias_dup = bam_hdr_dup(&hdr);
            assert!(!alias_dup.is_null());
            bam_hdr_destroy(alias_dup);
        }
    }

    #[test]
    fn pileup_maxcnt_setters_match_htslib_field_assignment() {
        let mut plp0 = bam_plp_s {
            mp: None,
            head: None,
            tail: None,
            tid: 0,
            max_tid: 0,
            pos: 0,
            max_pos: 0,
            is_eof: 0,
            max_plp: 0,
            error: 0,
            maxcnt: 8000,
            id: 0,
            plp: Vec::new(),
            b: None,
            func: None,
            data: std::ptr::null_mut(),
            overlaps: None,
            plp_construct: None,
            plp_destruct: None,
        };
        let mut plp1 = bam_plp_s {
            mp: None,
            head: None,
            tail: None,
            tid: 0,
            max_tid: 0,
            pos: 0,
            max_pos: 0,
            is_eof: 0,
            max_plp: 0,
            error: 0,
            maxcnt: 8000,
            id: 0,
            plp: Vec::new(),
            b: None,
            func: None,
            data: std::ptr::null_mut(),
            overlaps: None,
            plp_construct: None,
            plp_destruct: None,
        };
        let mut mplp = bam_mplp_s {
            n: 2,
            min_tid: 0,
            tid: Vec::new(),
            min_pos: 0,
            pos: Vec::new(),
            iter: vec![NonNull::from(&mut plp0), NonNull::from(&mut plp1)],
            n_plp: Vec::new(),
            plp: Vec::new(),
        };

        unsafe {
            bam_plp_set_maxcnt(&mut plp0, 123);
            assert_eq!(plp0.maxcnt, 123);
            assert_eq!(plp1.maxcnt, 8000);

            bam_mplp_set_maxcnt(&mut mplp, 456);
            assert_eq!(plp0.maxcnt, 456);
            assert_eq!(plp1.maxcnt, 456);
        }
    }

    unsafe extern "C" fn test_pileup_cd_callback(
        _data: *mut (),
        _b: *const bam1_t,
        _cd: *mut bam_pileup_cd,
    ) -> i32 {
        0
    }

    #[test]
    fn pileup_constructor_destructor_setters_match_htslib_field_assignment() {
        let mut plp0 = bam_plp_s {
            mp: None,
            head: None,
            tail: None,
            tid: 0,
            max_tid: 0,
            pos: 0,
            max_pos: 0,
            is_eof: 0,
            max_plp: 0,
            error: 0,
            maxcnt: 8000,
            id: 0,
            plp: Vec::new(),
            b: None,
            func: None,
            data: std::ptr::null_mut(),
            overlaps: None,
            plp_construct: None,
            plp_destruct: None,
        };
        let mut plp1 = bam_plp_s {
            mp: None,
            head: None,
            tail: None,
            tid: 0,
            max_tid: 0,
            pos: 0,
            max_pos: 0,
            is_eof: 0,
            max_plp: 0,
            error: 0,
            maxcnt: 8000,
            id: 0,
            plp: Vec::new(),
            b: None,
            func: None,
            data: std::ptr::null_mut(),
            overlaps: None,
            plp_construct: None,
            plp_destruct: None,
        };
        let mut mplp = bam_mplp_s {
            n: 2,
            min_tid: 0,
            tid: Vec::new(),
            min_pos: 0,
            pos: Vec::new(),
            iter: vec![NonNull::from(&mut plp0), NonNull::from(&mut plp1)],
            n_plp: Vec::new(),
            plp: Vec::new(),
        };

        unsafe {
            bam_plp_constructor(&mut plp0, Some(test_pileup_cd_callback));
            assert_eq!(
                plp0.plp_construct.map(|f| f as usize),
                Some(test_pileup_cd_callback as usize)
            );
            assert!(plp0.plp_destruct.is_none());

            bam_mplp_destructor(&mut mplp, Some(test_pileup_cd_callback));
            assert_eq!(
                plp0.plp_destruct.map(|f| f as usize),
                Some(test_pileup_cd_callback as usize)
            );
            assert_eq!(
                plp1.plp_destruct.map(|f| f as usize),
                Some(test_pileup_cd_callback as usize)
            );

            bam_mplp_constructor(&mut mplp, None);
            assert!(plp0.plp_construct.is_none());
            assert!(plp1.plp_construct.is_none());
        }
    }

    #[test]
    fn bam_mplp_init_matches_htslib_initial_state() {
        let mut user_data = [std::ptr::null_mut::<()>(), std::ptr::null_mut()];
        unsafe {
            let mplp = bam_mplp_init(2, None, user_data.as_mut_ptr());
            assert!(!mplp.is_null());
            assert_eq!((*mplp).n, 2);
            assert_eq!((*mplp).min_pos, HTS_POS_MAX);
            assert_eq!((*mplp).min_tid, -1);
            let mplp_ref = &mut *mplp;
            assert_eq!(mplp_ref.iter.len(), 2);
            for i in 0..2 {
                let idx = i as usize;
                assert_eq!(mplp_ref.pos[idx], HTS_POS_MAX);
                assert_eq!(mplp_ref.tid[idx], -1);
                assert_eq!(mplp_ref.n_plp[idx], 0);
                assert!(mplp_ref.plp[idx].is_null());
            }
            bam_mplp_destroy(mplp);
        }
    }

    #[test]
    fn bam_mplp_reset_restores_iterator_sentinels() {
        let mut user_data = [std::ptr::null_mut::<()>(), std::ptr::null_mut()];
        let pileup_marker = std::mem::MaybeUninit::<bam_pileup1_t>::uninit();
        unsafe {
            let mplp = bam_mplp_init(2, None, user_data.as_mut_ptr());
            assert!(!mplp.is_null());
            (*mplp).min_pos = 12;
            (*mplp).min_tid = 3;
            let mplp_ref = &mut *mplp;
            for i in 0..2 {
                let idx = i as usize;
                mplp_ref.pos[idx] = i as hts_pos_t;
                mplp_ref.tid[idx] = i;
                mplp_ref.n_plp[idx] = 9;
                mplp_ref.plp[idx] = pileup_marker.as_ptr();
                (*mplp_ref.iter[idx].as_ptr()).tid = 7;
                (*mplp_ref.iter[idx].as_ptr()).pos = 8;
                (*mplp_ref.iter[idx].as_ptr()).is_eof = 1;
            }

            bam_mplp_reset(mplp);
            let mplp_ref = &mut *mplp;
            assert_eq!(mplp_ref.min_pos, HTS_POS_MAX);
            assert_eq!(mplp_ref.min_tid, -1);
            for i in 0..2 {
                let idx = i as usize;
                assert_eq!(mplp_ref.pos[idx], HTS_POS_MAX);
                assert_eq!(mplp_ref.tid[idx], -1);
                assert_eq!(mplp_ref.n_plp[idx], 0);
                assert!(mplp_ref.plp[idx].is_null());
                assert_eq!((*mplp_ref.iter[idx].as_ptr()).tid, 0);
                assert_eq!((*mplp_ref.iter[idx].as_ptr()).pos, 0);
                assert_eq!((*mplp_ref.iter[idx].as_ptr()).is_eof, 0);
            }
            bam_mplp_destroy(mplp);
        }
    }

    #[test]
    fn bam_plp_insertion_builds_padded_sequence_and_mod_annotations() {
        unsafe {
            let b = bam_init1();
            let cigar = [
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CINS as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32,
            ];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"pins".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    3,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    3,
                    c"ACG".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let p = bam_pileup1_t {
                b,
                qpos: 0,
                indel: 2,
                level: 0,
                bitfields: 0,
                cd: bam_pileup_cd { i: 0 },
                cigar_ind: 0,
            };
            let mut ins = kstring_t::default();
            let mut del_len = -1;
            assert_eq!(bam_plp_insertion(&p, &mut ins, &mut del_len), 2);
            assert_eq!(del_len, 1);
            assert_eq!(ins.data.as_slice(), b"CG");

            let mut state = hts_base_mod_state_alloc();
            let mut mm_end = *b";\0";
            let mut ml = [55u8];
            (*state).nmods = 1;
            (*state).type_[0] = b'm' as i32;
            (*state).canonical[0] = 2;
            (*state).mmcount[0] = 0;
            (*state).mm[0] = mm_end.as_mut_ptr().cast();
            (*state).ml[0] = ml.as_mut_ptr();
            (*state).mlstride[0] = 1;
            (*state).implicit[0] = 1;
            (*state).seq_pos = 0;
            assert_eq!(
                bam_plp_insertion_mod(&p, &mut *state, &mut ins, &mut del_len),
                2
            );
            assert_eq!(ins.data.as_slice(), b"C[+m55]G");

            let no_ins = bam_pileup1_t { indel: 0, ..p };
            assert_eq!(bam_plp_insertion(&no_ins, &mut ins, &mut del_len), 0);
            assert_eq!(ins.data.as_slice(), b"");

            hts_base_mod_state_free(Some(state));
            bam_destroy1(b);
        }
    }

    #[test]
    fn empty_mplp_auto_returns_zero_like_htslib_merge_loop() {
        let mut mplp = bam_mplp_s {
            n: 0,
            min_tid: -1,
            tid: Vec::new(),
            min_pos: HTS_POS_MAX,
            pos: Vec::new(),
            iter: Vec::new(),
            n_plp: Vec::new(),
            plp: Vec::new(),
        };
        let mut tid = -1;
        let mut pos = -1;
        unsafe {
            assert_eq!(
                bam_mplp64_auto(
                    &mut mplp,
                    &mut tid,
                    &mut pos,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
        }
        assert_eq!(mplp.min_pos, HTS_POS_MAX);
        assert_eq!(mplp.min_tid, -1);
    }

    #[test]
    fn bam_flag_string_conversions_match_htslib_order_and_parsing() {
        unsafe {
            assert_eq!(
                bam_str2flag(c"PAIRED,proper_pair,UNMAP".as_ptr().cast()),
                BAM_FPAIRED | BAM_FPROPER_PAIR | BAM_FUNMAP
            );
            assert_eq!(bam_str2flag(c"0x41".as_ptr().cast()), 0x41);
            assert_eq!(bam_str2flag(c"PAIRED,NOPE".as_ptr().cast()), -1);

            let text =
                bam_flag2str(BAM_FPAIRED | BAM_FUNMAP | BAM_FREAD1 | BAM_FDUP | BAM_FSUPPLEMENTARY);
            assert_eq!(
                CStr::from_ptr(text.cast()).to_bytes(),
                b"PAIRED,UNMAP,READ1,DUP,SUPPLEMENTARY"
            );
            if !text.is_null() {
                let mut l = 0usize;
                while *text.add(l) != 0 {
                    l += 1;
                }
                drop(Vec::from_raw_parts(text, l + 1, l + 1));
            }

            let empty = bam_flag2str(0);
            assert_eq!(CStr::from_ptr(empty.cast()).to_bytes(), b"");
            if !empty.is_null() {
                let mut l = 0usize;
                while *empty.add(l) != 0 {
                    l += 1;
                }
                drop(Vec::from_raw_parts(empty, l + 1, l + 1));
            }
        }
    }

    #[test]
    fn aux_get_and_conversion_reject_malformed_payload_edges() {
        let mut z_data = b"r\0\0\0ZZZabc".to_vec();
        let z_record = bam1_t {
            core: bam1_core_t {
                pos: 0,
                tid: -1,
                bin: 0,
                qual: 0,
                l_extranul: 2,
                flag: BAM_FUNMAP as u16,
                l_qname: 4,
                n_cigar: 0,
                l_qseq: 0,
                mtid: -1,
                mpos: -1,
                isize: 0,
            },
            id: 0,
            data: z_data,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            assert!(bam_aux_get(&z_record, c"ZZ".as_ptr().cast()).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );
        }

        let b_data = b"r\0\0\0XABC\x02\0\0\0\x07".to_vec();
        let b_record = bam1_t {
            data: b_data,
            ..z_record
        };

        unsafe {
            assert!(bam_aux_get(&b_record, c"XA".as_ptr().cast()).is_null());
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            let b = bam_init1();
            assert!(!b.is_null());
            let array = [b'C', 2, 0, 0, 0, 10, 20];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"BC".as_ptr().cast(),
                    b'B' as u8,
                    array.len() as i32,
                    array.as_ptr(),
                ),
                0
            );
            let bc = bam_aux_get(b, c"BC".as_ptr().cast());
            assert!(!bc.is_null());
            assert_eq!(bam_auxB2i(bc, 1), 20);
            assert_eq!(bam_auxB2i(bc, 2), 0);
            assert_eq!(
                *libc::__errno_location(),
                libc::ERANGE as i32
            );
            assert_eq!(bam_auxB2f(bc, 9), 0.0);
            assert_eq!(
                *libc::__errno_location(),
                libc::ERANGE as i32
            );
            bam_destroy1(b);
        }
    }

    #[test]
    fn aux_update_int_preserves_htslib_size_thresholds_and_type_checks() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            assert_eq!(bam_aux_update_int(b, c"IV".as_ptr().cast(), -128), 0);
            let iv = bam_aux_get(b, c"IV".as_ptr().cast());
            assert_eq!(bam_aux_type(iv), b'c' as u8);
            assert_eq!(bam_aux2i(iv), -128);

            assert_eq!(bam_aux_update_int(b, c"IV".as_ptr().cast(), -129), 0);
            let iv = bam_aux_get(b, c"IV".as_ptr().cast());
            assert_eq!(bam_aux_type(iv), b's' as u8);
            assert_eq!(bam_aux2i(iv), -129);

            assert_eq!(bam_aux_update_int(b, c"IV".as_ptr().cast(), 254), 0);
            let iv = bam_aux_get(b, c"IV".as_ptr().cast());
            assert_eq!(bam_aux_type(iv), b'S' as u8);
            assert_eq!(bam_aux2i(iv), 254);

            assert_eq!(bam_aux_update_int(b, c"UV".as_ptr().cast(), u32::MAX as i64), 0);
            let uv = bam_aux_get(b, c"UV".as_ptr().cast());
            assert_eq!(bam_aux_type(uv), b'I' as u8);
            assert_eq!(bam_aux2i(uv), u32::MAX as i64);

            assert_eq!(
                bam_aux_update_int(b, c"OV".as_ptr().cast(), u32::MAX as i64 + 1),
                -1
            );
            assert_eq!(
                *libc::__errno_location(),
                libc::EOVERFLOW as i32
            );

            assert_eq!(
                bam_aux_update_str(b, c"ZS".as_ptr().cast(), -1, c"text".as_ptr().cast()),
                0
            );
            assert_eq!(bam_aux_update_int(b, c"ZS".as_ptr().cast(), 1), -1);
            assert_eq!(
                *libc::__errno_location(),
                libc::EINVAL as i32
            );

            bam_destroy1(b);
        }
    }

    #[test]
    fn header_sq_target_parsing_accepts_crlf_and_u32_max_len_edges() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let text = b"@HD\tVN:1.6\r\n@SQ\tLN:4294967295\tSN:max\r\n@CO\tignored\r\n";
            assert_eq!(sam_hdr_add_lines(&mut *hdr, text), 0);
            assert_eq!(sam_hdr_nref(&*hdr), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"max"), 0);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), u32::MAX as hts_pos_t);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(&*hdr, 0).cast()).to_bytes(), b"max");

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_lines_len_zero_uses_nul_terminated_input() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let text = c"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(sam_hdr_add_lines(&mut *hdr, text.to_bytes()), 0);
            assert_eq!(sam_hdr_length(&mut *hdr), text.to_bytes().len());
            assert_eq!(sam_hdr_nref(&*hdr), 1);
            assert_eq!(sam_hdr_name2tid(&mut *hdr, b"chr1"), 0);
            assert_eq!(sam_hdr_tid2len(&*hdr, 0), 10);

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_parse_rejects_null_text_without_leaking_header() {
        unsafe {
            assert!(sam_hdr_parse(0, std::ptr::null()).is_null());
            assert!(sam_hdr_parse_(std::ptr::null(), 0).is_null());
        }
    }

    #[test]
    fn cigar_parse_failures_leave_end_at_input_and_record_unchanged() {
        unsafe {
            assert_eq!(
                sam_parse_cigar(
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                -1
            );

            let mut end: *mut u8 = std::ptr::null_mut();
            let mut a_cigar: *mut u32 = std::ptr::null_mut();
            let mut a_mem = 0usize;
            let invalid_op = b"1Q\t";
            assert_eq!(
                sam_parse_cigar(invalid_op.as_ptr(), &mut end, &mut a_cigar, &mut a_mem),
                -1
            );
            assert_eq!(end, invalid_op.as_ptr().cast_mut());

            let overflow = b"268435456M\t";
            assert_eq!(
                sam_parse_cigar(overflow.as_ptr(), &mut end, &mut a_cigar, &mut a_mem),
                -1
            );
            assert_eq!(end, overflow.as_ptr().cast_mut());
            if !a_cigar.is_null() {
                drop(Vec::from_raw_parts(a_cigar, a_mem, a_mem));
            }

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(3u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"cigx".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    3,
                    c"ACG".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let old_l_data = (*b).data.len() as i32;
            assert_eq!(bam_parse_cigar(invalid_op.as_ptr(), &mut end, b), -1);
            assert_eq!(end, invalid_op.as_ptr().cast_mut());
            assert_eq!((*b).core.n_cigar, 1);
            assert_eq!((*b).data.len() as i32, old_l_data);
            assert_eq!(*bam_get_cigar(b), cigar[0]);
            bam_destroy1(b);
        }
    }

    #[test]
    fn resolve_cigar2_tracks_refskip_deletion_and_tail_state() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CSOFT_CLIP as u32,
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (3u32 << BAM_CIGAR_SHIFT) | BAM_CREF_SKIP as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
            ];
            assert!(
                bam_set1(
                    b,
                    6,
                    c"pilex".as_ptr().cast(),
                    0,
                    0,
                    10,
                    60,
                    cigar.len(),
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    6,
                    c"AACCGG".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let mut p = bam_pileup1_t {
                b,
                qpos: -1,
                indel: 99,
                level: 0,
                bitfields: 0xffff,
                cd: bam_pileup_cd { i: 0 },
                cigar_ind: -1,
            };
            let mut state = G_CSTATE_NULL;
            state.end = bam_endpos(b) - 1;

            assert_eq!(resolve_cigar2(&mut p, 10, &mut state), 1);
            assert_eq!(p.qpos, 2);
            assert_eq!(p.cigar_ind, 1);
            assert_eq!(bam_pileup1_is_head(&p), 1);
            assert_eq!(bam_pileup1_is_del(&p), 0);
            assert_eq!(bam_pileup1_is_refskip(&p), 0);

            assert_eq!(resolve_cigar2(&mut p, 12, &mut state), 1);
            assert_eq!(p.qpos, 4);
            assert_eq!(p.cigar_ind, 2);
            assert_eq!(bam_pileup1_is_del(&p), 1);
            assert_eq!(bam_pileup1_is_refskip(&p), 1);
            assert_eq!(p.indel, 0);

            assert_eq!(resolve_cigar2(&mut p, 15, &mut state), 1);
            assert_eq!(p.qpos, 4);
            assert_eq!(p.cigar_ind, 3);
            assert_eq!(bam_pileup1_is_del(&p), 0);
            assert_eq!(bam_pileup1_is_refskip(&p), 0);
            assert_eq!(p.indel, -2);

            assert_eq!(resolve_cigar2(&mut p, 16, &mut state), 1);
            assert_eq!(p.qpos, 5);
            assert_eq!(p.cigar_ind, 4);
            assert_eq!(bam_pileup1_is_del(&p), 1);
            assert_eq!(bam_pileup1_is_refskip(&p), 0);

            assert_eq!(resolve_cigar2(&mut p, 18, &mut state), 1);
            assert_eq!(p.qpos, 5);
            assert_eq!(p.cigar_ind, 5);
            assert_eq!(bam_pileup1_is_tail(&p), 1);
            assert_eq!(bam_pileup1_is_del(&p), 0);

            bam_destroy1(b);
        }
    }

    #[test]
    fn resolve_cigar2_accumulates_insertions_across_padding() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CPAD as u32,
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CINS as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CPAD as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CINS as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
            ];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"padx".as_ptr().cast(),
                    0,
                    0,
                    20,
                    60,
                    cigar.len(),
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    5,
                    c"AACCG".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let mut p = bam_pileup1_t {
                b,
                qpos: -1,
                indel: 0,
                level: 0,
                bitfields: 0,
                cd: bam_pileup_cd { i: 0 },
                cigar_ind: -1,
            };
            let mut state = G_CSTATE_NULL;
            state.end = bam_endpos(b) - 1;

            assert_eq!(resolve_cigar2(&mut p, 20, &mut state), 1);
            assert_eq!(p.qpos, 0);
            assert_eq!(p.cigar_ind, 0);
            assert_eq!(p.indel, 3);
            assert_eq!(bam_pileup1_is_del(&p), 0);
            assert_eq!(bam_pileup1_is_refskip(&p), 0);

            bam_destroy1(b);
        }
    }

    #[test]
    fn resolve_cigar2_skips_leading_hard_clip_and_padding() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CHARD_CLIP as u32,
                (1u32 << BAM_CIGAR_SHIFT) | BAM_CPAD as u32,
                (3u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (2u32 << BAM_CIGAR_SHIFT) | BAM_CHARD_CLIP as u32,
            ];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"hpad".as_ptr().cast(),
                    0,
                    0,
                    30,
                    60,
                    cigar.len(),
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    3,
                    c"ACG".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let mut p = bam_pileup1_t {
                b,
                qpos: -1,
                indel: -1,
                level: 0,
                bitfields: 0,
                cd: bam_pileup_cd { i: 0 },
                cigar_ind: -1,
            };
            let mut state = G_CSTATE_NULL;
            state.end = bam_endpos(b) - 1;

            assert_eq!(resolve_cigar2(&mut p, 30, &mut state), 1);
            assert_eq!(p.qpos, 0);
            assert_eq!(p.cigar_ind, 2);
            assert_eq!(bam_pileup1_is_head(&p), 1);
            assert_eq!(bam_pileup1_is_tail(&p), 0);

            assert_eq!(resolve_cigar2(&mut p, 32, &mut state), 1);
            assert_eq!(p.qpos, 2);
            assert_eq!(p.cigar_ind, 2);
            assert_eq!(bam_pileup1_is_head(&p), 0);
            assert_eq!(bam_pileup1_is_tail(&p), 1);

            bam_destroy1(b);
        }
    }

    #[test]
    fn pileup_bitfield_accessors_preserve_htslib_layout() {
        let p = bam_pileup1_t {
            b: std::ptr::null_mut(),
            qpos: 0,
            indel: 0,
            level: 0,
            bitfields: 0b1010_1111_1111,
            cd: bam_pileup_cd { i: 0 },
            cigar_ind: 0,
        };

        unsafe {
            assert_eq!(bam_pileup1_is_del(&p), 1);
            assert_eq!(bam_pileup1_is_head(&p), 1);
            assert_eq!(bam_pileup1_is_tail(&p), 1);
            assert_eq!(bam_pileup1_is_refskip(&p), 1);
            assert_eq!(bam_pileup1_aux(&p), p.bitfields >> 5);
        }
    }

    #[test]
    fn base_mod_parser_reports_co_located_multi_mod_calls() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(1u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"bmm2".as_ptr().cast(),
                    0,
                    0,
                    0,
                    60,
                    1,
                    cigar.as_ptr(),
                    -1,
                    -1,
                    0,
                    1,
                    c"C".as_ptr().cast(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mm = b"C+mh,0;\0";
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr().cast(),
                    b'Z' as u8,
                    mm.len() as i32,
                    mm.as_ptr()
                ),
                0
            );
            let ml = [b'C', 2, 0, 0, 0, 11, 22];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"ML".as_ptr().cast(),
                    b'B' as u8,
                    ml.len() as i32,
                    ml.as_ptr()
                ),
                0
            );

            let mut state = hts_base_mod_state_alloc();
            assert_eq!(bam_parse_basemod(&*b, &mut *state), 0);
            assert_eq!((*state).nmods, 2);
            assert_eq!((*state).type_[0], b'm' as i32);
            assert_eq!((*state).type_[1], b'h' as i32);
            assert_eq!((*state).canonical[0], 2);
            assert_eq!((*state).canonical[1], 2);
            assert_eq!((*state).mlstride[0], 2);
            assert_eq!((*state).mlstride[1], 2);
            assert_eq!((*state).mm[0], (*state).mm[1]);

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }; 2];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(&*b, &mut *state, &mut mods, &mut pos),
                2
            );
            assert_eq!(pos, 0);
            assert_eq!(mods[0].modified_base, b'm' as i32);
            assert_eq!(mods[0].qual, 11);
            assert_eq!(mods[1].modified_base, b'h' as i32);
            assert_eq!(mods[1].qual, 22);
            assert_eq!(
                bam_next_basemod(&*b, &mut *state, &mut mods, &mut pos),
                0
            );

            hts_base_mod_state_free(Some(state));
            bam_destroy1(b);
        }
    }

    // ---------------------------------------------------------------
    // Concurrency coverage for sam_c_1173_bam_get_library.
    //
    // Background: the htslib v1.23 C original (`htslib/sam.c::bam_get_library`)
    // returns a pointer into a function-static `char lb_text[1024]`.  Two
    // threads decoding different records race on that buffer.  Our Rust port
    // used the same pattern (`static mut LB_TEXT: [u8; 1024]`).  The fix
    // (above) promotes the buffer to a `thread_local!` so that each thread
    // gets its own private 1024-byte scratch, and the returned `*const u8`
    // is valid for the lifetime of that thread (which trivially covers the
    // same-thread `kputs` consumer at sam.rs:6207).
    //
    // The tests below pin that contract:
    //   (1) `bam_get_library_serial_smoke` is a single-thread sanity check —
    //       guards against the thread_local refactor accidentally changing
    //       the observable return value for a known fixture.
    //   (2) `bam_get_library_concurrent_returns_correct_per_thread` runs N
    //       distinct (sam_hdr_t, bam1_t) pairs on N threads in tight loops
    //       and asserts each thread always reads back its own library name,
    //       never another thread's bytes — i.e. no buffer aliasing across
    //       threads.
    // ---------------------------------------------------------------

    /// Wrapper around `(*mut sam_hdr_t, *mut bam1_t)` so it can be shared
    /// across threads via `Arc`.  Each pair is conceptually owned by a single
    /// worker thread — no thread ever mutates a pair another thread is using
    /// — so transferring/aliasing the raw pointers across thread boundaries
    /// is sound for the duration of the test.
    struct LibTestPair {
        hdr: *mut sam_hdr_t,
        bam: *mut bam1_t,
        expected: std::ffi::CString,
    }

    // SAFETY: the test orchestrator (a) builds every `LibTestPair` up-front
    // on the main thread, (b) hands each individual pair to exactly one
    // worker thread for the duration of the test, and (c) joins all workers
    // before dropping/destroying any pair.  No two threads ever touch the
    // same `sam_hdr_t` or `bam1_t`, so the raw pointers are effectively
    // `&mut`-style exclusive within a thread.  Crossing thread boundaries
    // is therefore sound.
    unsafe impl Send for LibTestPair {}
    unsafe impl Sync for LibTestPair {}

    /// Build a fresh (header, bam1_t) pair whose RG aux tag resolves to
    /// `library` via the @RG ID:`rg_id` LB:`library` header line.
    unsafe fn make_lib_test_pair(rg_id: &str, library: &str) -> LibTestPair {
        // Header: one @SQ so name2tid works, plus our @RG line.
        let header_text = format!(
            "@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:1000\n@RG\tID:{}\tLB:{}\n",
            rg_id, library
        );
        let header_bytes = header_text.as_bytes();
        let hdr = sam_hdr_init();
        assert!(!hdr.is_null());
        assert_eq!(
            sam_hdr_add_lines(&mut *hdr, header_bytes),
            0,
            "sam_hdr_add_lines failed for {}",
            rg_id
        );

        // bam1_t with a minimal alignment, plus RG:Z:<rg_id> aux tag.
        let bam = bam_init1();
        assert!(!bam.is_null());
        let cigar = [(4u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
        let qname = std::ffi::CString::new("read").unwrap();
        let seq = std::ffi::CString::new("ACGT").unwrap();
        let qual = [31u8, 32, 33, 34];
        let set_ret = bam_set1(
            bam,
            4,
            qname.as_ptr().cast(),
            0,
            0,
            0,
            60,
            1,
            cigar.as_ptr(),
            -1,
            -1,
            0,
            4,
            seq.as_ptr().cast(),
            qual.as_ptr().cast(),
            0,
        );
        assert!(set_ret > 0, "bam_set1 failed");

        // RG aux tag bytes: NUL-terminated C string of the RG ID.
        let rg_cstr = std::ffi::CString::new(rg_id).unwrap();
        let rg_bytes_with_nul = rg_cstr.as_bytes_with_nul();
        let append_ret = bam_aux_append(
            bam,
            c"RG".as_ptr().cast(),
            b'Z' as u8,
            rg_bytes_with_nul.len() as i32,
            rg_bytes_with_nul.as_ptr(),
        );
        assert_eq!(append_ret, 0, "bam_aux_append(RG) failed");

        LibTestPair {
            hdr,
            bam,
            expected: std::ffi::CString::new(library).unwrap(),
        }
    }

    unsafe fn destroy_lib_test_pair(pair: &LibTestPair) {
        bam_destroy1(pair.bam);
        sam_hdr_destroy(pair.hdr);
    }

    #[test]
    fn bam_get_library_serial_smoke() {
        unsafe {
            let pair = make_lib_test_pair("rg_solo", "LIB_SOLO");
            let lib = sam_c_1173_bam_get_library(pair.hdr, pair.bam);
            assert!(!lib.is_null(), "expected non-null library pointer");
            assert_eq!(
                CStr::from_ptr(lib.cast()).to_bytes(),
                b"LIB_SOLO",
                "library tag mismatch on serial smoke"
            );
            // Repeated same-thread calls keep returning the same value.
            for _ in 0..16 {
                let lib2 = sam_c_1173_bam_get_library(pair.hdr, pair.bam);
                assert_eq!(CStr::from_ptr(lib2.cast()).to_bytes(), b"LIB_SOLO");
            }
            destroy_lib_test_pair(&pair);
        }
    }

    #[test]
    fn bam_get_library_concurrent_returns_correct_per_thread() {
        use std::sync::Arc;
        use std::thread;

        const N_THREADS: usize = 8;
        const ITERS_PER_THREAD: usize = 10_000;

        unsafe {
            // Build N distinct (header, bam, expected-library) triples on the
            // main thread.  Wrap them in `Arc<Vec<_>>` so the worker threads
            // can index into the shared array (matches the task spec of
            // sharing pointers via `Arc`).  Each worker only touches its own
            // index — no aliasing.
            let mut pairs: Vec<LibTestPair> = Vec::with_capacity(N_THREADS);
            for i in 0..N_THREADS {
                let rg_id = format!("rg_thread_{}", i);
                let library = format!("LIB_THREAD_{}", i);
                pairs.push(make_lib_test_pair(&rg_id, &library));
            }
            let pairs = Arc::new(pairs);

            let mut handles = Vec::with_capacity(N_THREADS);
            for tid in 0..N_THREADS {
                let pairs = Arc::clone(&pairs);
                handles.push(thread::spawn(move || {
                    let pair = &pairs[tid];
                    let expected = pair.expected.as_bytes();
                    // SAFETY: this thread owns exclusive access to `pair.hdr`
                    // and `pair.bam` for the duration of this closure: no
                    // other worker indexes `tid`, and the main thread does
                    // not touch the vector again until all workers have
                    // joined.  `sam_c_1173_bam_get_library` is sound under
                    // those conditions and (post-fix) returns a pointer into
                    // *this thread's* private thread_local buffer.
                    for iter in 0..ITERS_PER_THREAD {
                        let lib = sam_c_1173_bam_get_library(pair.hdr, pair.bam);
                        assert!(!lib.is_null(), "thread {} iter {}: null library", tid, iter);
                        let got = CStr::from_ptr(lib.cast()).to_bytes();
                        assert_eq!(
                            got,
                            expected,
                            "thread {} iter {}: expected {:?}, got {:?}",
                            tid,
                            iter,
                            std::str::from_utf8(expected).unwrap_or("<non-utf8>"),
                            std::str::from_utf8(got).unwrap_or("<non-utf8>")
                        );
                    }
                }));
            }

            for handle in handles {
                handle.join().expect("worker thread panicked");
            }

            // All workers have joined; safe to tear the pairs down.
            let pairs = Arc::try_unwrap(pairs)
                .ok()
                .expect("Arc still has outstanding refs after join");
            for pair in &pairs {
                destroy_lib_test_pair(pair);
            }
        }
    }
}
