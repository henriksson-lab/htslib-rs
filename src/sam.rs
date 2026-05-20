use std::{
    collections::{HashMap, VecDeque},
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    sync::{Mutex, OnceLock},
};

use crate::htslib_mini_rs::bgzf::{
    bgzf_check_EOF, bgzf_flush, bgzf_flush_try, bgzf_peek, bgzf_read, bgzf_read_small, bgzf_seek,
    bgzf_write, bgzf_write_small,
};
use crate::htslib_mini_rs::hfile::hflush;
use crate::htslib_mini_rs::hts::{
    __ac_FNV1a_hash_string, __ac_Wang_hash, __ac_X31_hash_string, double_to_le, ed_swap_4p,
    find_file_extension, float_to_le, htsFile, htsLogLevel, hts_bin_maxpos, hts_expr_val_t,
    hts_filter_eval2, hts_filter_t, hts_idx_destroy, hts_idx_load3, hts_idx_t, hts_itr_multi_bam,
    hts_itr_multi_next, hts_itr_next, hts_itr_query, hts_itr_regions, hts_itr_t, hts_parse_region,
    hts_pos_t, hts_reg2bin, hts_reglist_create, hts_reglist_free, hts_reglist_t, hts_str2int,
    hts_str2uint, i16_to_le, i32_to_le, isalnum_c, isalpha_c, isdigit_c, islower_c, isspace_c,
    isupper_c, kputc, kputc_, kputll, kputs, kputsn, kputsn_, kputuw, kputw, ks_clear, ks_expand,
    ks_free, ks_resize, kstring_t, toupper_c, u16_to_le, u32_to_le, u64_to_le, BGZF, HTS_FMT_BAI,
    HTS_FMT_CRAI, HTS_FMT_CSI, HTS_FORMAT_BAM, HTS_FORMAT_BINARY_FORMAT, HTS_FORMAT_CRAM,
    HTS_FORMAT_EMPTY_FORMAT, HTS_FORMAT_FASTA_FORMAT, HTS_FORMAT_FASTQ_FORMAT, HTS_FORMAT_SAM,
    HTS_FORMAT_SEQUENCE_DATA, HTS_FORMAT_TEXT_FORMAT, HTS_IDX_NOCOOR, HTS_IDX_SAVE_REMOTE,
    HTS_IDX_START, HTS_MAX_EXT_LEN, HTS_PARSE_THOUSANDS_SEP, HTS_POS_MAX,
};

extern "C" {
    fn hpeek(
        fp: *mut crate::htslib_mini_rs::hts::hFILE,
        buffer: *mut c_void,
        nbytes: usize,
    ) -> isize;
}

pub const BAM_CMATCH: c_int = 0;
pub const BAM_CINS: c_int = 1;
pub const BAM_CDEL: c_int = 2;
pub const BAM_CREF_SKIP: c_int = 3;
pub const BAM_CSOFT_CLIP: c_int = 4;
pub const BAM_CHARD_CLIP: c_int = 5;
pub const BAM_CPAD: c_int = 6;
pub const BAM_CEQUAL: c_int = 7;
pub const BAM_CDIFF: c_int = 8;
pub const BAM_CBACK: c_int = 9;
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
pub const FASTQ_OPT_CASAVA: c_int = 1000;
pub const FASTQ_OPT_AUX: c_int = 1001;
pub const FASTQ_OPT_RNUM: c_int = 1002;
pub const FASTQ_OPT_BARCODE: c_int = 1003;
pub const FASTQ_OPT_NAME2: c_int = 1004;
pub const FASTQ_OPT_UMI: c_int = 1005;
pub const FASTQ_OPT_UMI_REGEX: c_int = 1006;

pub const BAM_FPAIRED: c_int = 1;
pub const BAM_FPROPER_PAIR: c_int = 2;
pub const BAM_FUNMAP: c_int = 4;
pub const BAM_FMUNMAP: c_int = 8;
pub const BAM_FREVERSE: c_int = 16;
pub const BAM_FMREVERSE: c_int = 32;
pub const BAM_FREAD1: c_int = 64;
pub const BAM_FREAD2: c_int = 128;
pub const BAM_FSECONDARY: c_int = 256;
pub const BAM_FQCFAIL: c_int = 512;
pub const BAM_FDUP: c_int = 1024;
pub const BAM_FSUPPLEMENTARY: c_int = 2048;

pub const BAM_USER_OWNS_STRUCT: u32 = 1;
pub const BAM_USER_OWNS_DATA: u32 = 2;
pub const SAM_FORMAT_VERSION: &str = "1.6";
pub const HTS_MOD_UNKNOWN: c_int = -1;
pub const HTS_MOD_UNCHECKED: c_int = -2;
pub const HTS_MOD_REPORT_UNCHECKED: u32 = 1;

const SEQI_RC: [c_int; 16] = [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];
const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

fn set_pileup_is_del(p: *mut bam_pileup1_t, value: bool) {
    unsafe {
        if value {
            (*p).bitfields |= 1;
        } else {
            (*p).bitfields &= !1;
        }
    }
}

fn set_pileup_is_head(p: *mut bam_pileup1_t, value: bool) {
    unsafe {
        if value {
            (*p).bitfields |= 1 << 1;
        } else {
            (*p).bitfields &= !(1 << 1);
        }
    }
}

fn set_pileup_is_tail(p: *mut bam_pileup1_t, value: bool) {
    unsafe {
        if value {
            (*p).bitfields |= 1 << 2;
        } else {
            (*p).bitfields &= !(1 << 2);
        }
    }
}

fn set_pileup_is_refskip(p: *mut bam_pileup1_t, value: bool) {
    unsafe {
        if value {
            (*p).bitfields |= 1 << 3;
        } else {
            (*p).bitfields &= !(1 << 3);
        }
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

#[repr(C)]
pub struct bam1_t {
    pub core: bam1_core_t,
    pub id: u64,
    pub data: *mut u8,
    pub l_data: c_int,
    pub m_data: u32,
    pub mempolicy_and_reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union bam_pileup_cd {
    pub p: *mut c_void,
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

pub const MAX_BASE_MOD: usize = 256;

#[repr(C)]
pub struct hts_base_mod_state {
    pub type_: [c_int; MAX_BASE_MOD],
    pub canonical: [c_int; MAX_BASE_MOD],
    pub strand: [c_char; MAX_BASE_MOD],
    pub mmcount: [c_int; MAX_BASE_MOD],
    pub mm: [*mut c_char; MAX_BASE_MOD],
    pub mmend: [*mut c_char; MAX_BASE_MOD],
    pub ml: [*mut u8; MAX_BASE_MOD],
    pub mlstride: [c_int; MAX_BASE_MOD],
    pub implicit: [c_int; MAX_BASE_MOD],
    pub seq_pos: c_int,
    pub nmods: c_int,
    pub flags: u32,
}

#[repr(C)]
pub struct sp_bams {
    pub next: *mut sp_bams,
    pub serial: c_int,
    pub bams: *mut bam1_t,
    pub nbams: c_int,
    pub abams: c_int,
    pub bam_mem: usize,
    pub fd: *mut SAM_state,
}

#[repr(C)]
pub struct sp_lines {
    pub next: *mut sp_lines,
    pub serial: c_int,
    pub data: *mut c_char,
    pub data_size: c_int,
    pub alloc: c_int,
    pub fd: *mut SAM_state,
    pub bams: *mut sp_bams,
}

#[repr(C)]
pub struct SAM_state {
    pub h: *mut sam_hdr_t,
    pub p: *mut c_void,
    pub own_pool: c_int,
    pub lines: *mut sp_lines,
    pub bams: *mut sp_bams,
    pub curr_bam: *mut sp_bams,
    pub curr_idx: c_int,
    pub serial: c_int,
    pub command: c_int,
    pub errcode: c_int,
    pub fp: *mut htsFile,
}

#[repr(C)]
struct hb_pair {
    h: *const sam_hdr_t,
    b: *const bam1_t,
}

#[repr(C)]
pub struct fastq_state {
    pub name: kstring_t,
    pub comment: kstring_t,
    pub seq: kstring_t,
    pub qual: kstring_t,
    pub casava: c_int,
    pub aux: c_int,
    pub rnum: c_int,
    pub BC: [c_char; 3],
    pub UMI: [[c_char; 3]; UMI_TAGS],
    pub tags: *mut c_void,
    pub nprefix: c_char,
    pub sra_names: c_int,
    pub regex: libc::regex_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct hts_base_mod {
    pub modified_base: c_int,
    pub canonical_base: c_int,
    pub strand: c_int,
    pub qual: c_int,
}

#[repr(C)]
pub struct sam_hrec_sq_t {
    pub name: *const c_char,
    pub len: hts_pos_t,
    pub ty: *mut c_void,
}

// original: sam_hrec_tag_s (htslib/header.h:98)
#[repr(C)]
pub struct sam_hrec_tag_t {
    pub next: *mut sam_hrec_tag_t,
    pub str_: *const c_char,
    pub len: c_int,
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
    pub h: *mut c_void,
    pub first_line: *mut c_void,
    pub str_pool: *mut c_void,
    pub type_pool: *mut c_void,
    pub tag_pool: *mut c_void,
    pub nref: c_int,
    pub ref_sz: c_int,
    pub ref_: *mut sam_hrec_sq_t,
    pub ref_hash: *mut c_void,
    pub nrg: c_int,
    pub rg_sz: c_int,
    pub rg: *mut c_void,
    pub rg_hash: *mut c_void,
    pub npg: c_int,
    pub pg_sz: c_int,
    pub npg_end: c_int,
    pub npg_end_alloc: c_int,
    pub pg: *mut c_void,
    pub pg_hash: *mut c_void,
    pub pg_end: *mut c_int,
    pub ID_buf: *mut c_char,
    pub ID_buf_sz: u32,
    pub ID_cnt: c_int,
    pub dirty: c_int,
    pub refs_changed: c_int,
    pub pgs_changed: c_int,
    pub type_count: c_int,
    pub type_order: *mut [c_char; 3],
}

#[repr(C)]
pub struct khash_s2i_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut *mut c_char,
    pub vals: *mut i64,
}

#[repr(C)]
pub struct khash_tag_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut c_int,
}

#[repr(C)]
pub struct khash_m_s2i_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut *mut c_char,
    pub vals: *mut c_int,
}

#[repr(C)]
pub struct sam_hdr_t {
    pub n_targets: i32,
    pub ignore_sam_err: i32,
    pub l_text: usize,
    pub target_len: *mut u32,
    pub cigar_tab: *const i8,
    pub target_name: *mut *mut c_char,
    pub text: *mut c_char,
    pub sdict: *mut c_void,
    pub hrecs: *mut sam_hrecs_t,
    pub ref_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cstate_t {
    pub k: c_int,
    pub y: c_int,
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
    pub next: *mut lbnode_t,
    pub cd: bam_pileup_cd,
}

pub type __linkbuf_t = lbnode_t;

#[repr(C)]
pub struct mempool_t {
    pub cnt: c_int,
    pub n: c_int,
    pub max: c_int,
    pub padding_0: c_int,
    pub buf: *mut *mut lbnode_t,
}

#[repr(C)]
pub struct olap_hash_t {
    _private: [u8; 0],
}

type OlapHash = HashMap<Vec<u8>, *mut lbnode_t>;

pub type bam_plp_t = *mut bam_plp_s;
pub type bam_plp_auto_f = Option<unsafe extern "C" fn(*mut c_void, *mut bam1_t) -> c_int>;
pub type bam_plp_constructor_f =
    Option<unsafe extern "C" fn(*mut c_void, *const bam1_t, *mut bam_pileup_cd) -> c_int>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct bam_plp_s {
    pub mp: *mut mempool_t,
    pub head: *mut lbnode_t,
    pub tail: *mut lbnode_t,
    pub tid: i32,
    pub max_tid: i32,
    pub pos: hts_pos_t,
    pub max_pos: hts_pos_t,
    pub is_eof: c_int,
    pub max_plp: c_int,
    pub error: c_int,
    pub maxcnt: c_int,
    pub id: u64,
    pub plp: *mut bam_pileup1_t,
    pub b: *mut bam1_t,
    pub func: bam_plp_auto_f,
    pub data: *mut c_void,
    pub overlaps: *mut olap_hash_t,
    pub plp_construct: bam_plp_constructor_f,
    pub plp_destruct: bam_plp_constructor_f,
}

pub type bam_mplp_t = *mut bam_mplp_s;

#[repr(C)]
pub struct bam_mplp_s {
    pub n: c_int,
    pub min_tid: i32,
    pub tid: *mut i32,
    pub min_pos: hts_pos_t,
    pub pos: *mut hts_pos_t,
    pub iter: *mut bam_plp_t,
    pub n_plp: *mut c_int,
    pub plp: *mut *const bam_pileup1_t,
}

pub unsafe fn sam_hdr_init() -> *mut sam_hdr_t {
    let h = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sam_hdr_t>() as u64)
        .cast::<sam_hdr_t>();
    if h.is_null() {
        return std::ptr::null_mut();
    }
    (*h).cigar_tab = BAM_CIGAR_TABLE.as_ptr();
    h
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

unsafe fn kh_resize_s2i(h: *mut khash_s2i_t, new_n_buckets: u32) -> c_int {
    let n_flags = if new_n_buckets < 16 {
        1
    } else {
        new_n_buckets >> 4
    };
    let flags =
        crate::htslib_mini_rs::c_compat::malloc(n_flags as u64 * std::mem::size_of::<u32>() as u64)
            .cast::<u32>();
    let keys = crate::htslib_mini_rs::c_compat::malloc(
        new_n_buckets as u64 * std::mem::size_of::<*mut c_char>() as u64,
    )
    .cast::<*mut c_char>();
    let vals = crate::htslib_mini_rs::c_compat::malloc(
        new_n_buckets as u64 * std::mem::size_of::<i64>() as u64,
    )
    .cast::<i64>();
    if flags.is_null() || keys.is_null() || vals.is_null() {
        crate::htslib_mini_rs::c_compat::free(flags.cast());
        crate::htslib_mini_rs::c_compat::free(keys.cast());
        crate::htslib_mini_rs::c_compat::free(vals.cast());
        return -1;
    }
    for i in 0..n_flags {
        *flags.add(i as usize) = 0xaaaa_aaaa;
    }

    let old_flags = (*h).flags;
    let old_keys = (*h).keys;
    let old_vals = (*h).vals;
    let old_n = (*h).n_buckets;

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
            let mut site = __ac_FNV1a_hash_string(key) & mask;
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

    crate::htslib_mini_rs::c_compat::free(old_flags.cast());
    crate::htslib_mini_rs::c_compat::free(old_keys.cast());
    crate::htslib_mini_rs::c_compat::free(old_vals.cast());
    0
}

unsafe fn kh_put_s2i(h: *mut khash_s2i_t, key: *const c_char, ret: *mut c_int) -> u32 {
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
    let mut i = __ac_FNV1a_hash_string(key) & mask;
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
    name: *const c_char,
    len: hts_pos_t,
) -> c_int {
    if (*h).sdict.is_null() {
        let sdict =
            crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<khash_s2i_t>() as u64)
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
    *(*long_refs).vals.add(k as usize) = len as i64;
    0
}

unsafe fn sam_hdr_append_target(h: *mut sam_hdr_t, name: &[u8], len: hts_pos_t) -> c_int {
    let new_n = (*h).n_targets + 1;
    if new_n <= 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int;
        return -1;
    }

    let target_len = crate::htslib_mini_rs::c_compat::realloc(
        (*h).target_len.cast(),
        new_n as u64 * std::mem::size_of::<u32>() as u64,
    )
    .cast::<u32>();
    if target_len.is_null() {
        return -1;
    }
    (*h).target_len = target_len;

    let target_name = crate::htslib_mini_rs::c_compat::realloc(
        (*h).target_name.cast(),
        new_n as u64 * std::mem::size_of::<*mut c_char>() as u64,
    )
    .cast::<*mut c_char>();
    if target_name.is_null() {
        return -1;
    }
    (*h).target_name = target_name;

    let dup = crate::htslib_mini_rs::c_compat::malloc(name.len() as u64 + 1).cast::<c_char>();
    if dup.is_null() {
        return -1;
    }
    crate::htslib_mini_rs::c_compat::memcpy(dup.cast(), name.as_ptr().cast(), name.len() as u64);
    *dup.add(name.len()) = 0;

    let idx = (*h).n_targets as usize;
    *(*h).target_name.add(idx) = dup;
    *(*h).target_len.add(idx) = if len >= u32::MAX as hts_pos_t {
        u32::MAX
    } else {
        len as u32
    };
    if len > u32::MAX as hts_pos_t && sam_hdr_set_long_target_len(h, dup, len) < 0 {
        crate::htslib_mini_rs::c_compat::free(dup.cast());
        *(*h).target_name.add(idx) = std::ptr::null_mut();
        return -1;
    }
    (*h).n_targets = new_n;
    0
}

unsafe fn sam_hdr_free_tmp_targets(tmp: *mut sam_hdr_t) {
    for i in 0..(*tmp).n_targets {
        let name = *(*tmp).target_name.add(i as usize);
        if !name.is_null() {
            crate::htslib_mini_rs::c_compat::free(name.cast());
        }
    }
    crate::htslib_mini_rs::c_compat::free((*tmp).target_name.cast());
    crate::htslib_mini_rs::c_compat::free((*tmp).target_len.cast());
    kh_destroy_s2i((*tmp).sdict.cast());
}

unsafe fn sam_hdr_restore_text_len(h: *mut sam_hdr_t, old_len: usize) {
    (*h).l_text = old_len;
    if old_len == 0 {
        crate::htslib_mini_rs::c_compat::free((*h).text.cast());
        (*h).text = std::ptr::null_mut();
    } else if !(*h).text.is_null() {
        *(*h).text.add(old_len) = 0;
    }
}

unsafe fn sam_hdr_fill_targets_from_text(h: *mut sam_hdr_t) -> c_int {
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

pub unsafe fn sam_hdr_add_lines(_h: *mut sam_hdr_t, _lines: *const c_char, _len: usize) -> c_int {
    if _h.is_null() || _lines.is_null() {
        return -1;
    }
    if _len == 0 && *_lines == 0 {
        return 0;
    }
    if !(*_h).hrecs.is_null() {
        return -1;
    }

    let len = if _len == 0 {
        libc::strlen(_lines)
    } else {
        _len
    };
    let old_len = (*_h).l_text;
    let new_len = match old_len.checked_add(len) {
        Some(v) => v,
        None => {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int;
            return -1;
        }
    };
    let text = crate::htslib_mini_rs::c_compat::realloc((*_h).text.cast(), new_len as u64 + 1)
        .cast::<c_char>();
    if text.is_null() {
        return -1;
    }
    (*_h).text = text;
    if len > 0 {
        crate::htslib_mini_rs::c_compat::memcpy(
            (*_h).text.add(old_len).cast(),
            _lines.cast(),
            len as u64,
        );
    }
    (*_h).l_text = new_len;
    *(*_h).text.add(new_len) = 0;

    let mut tmp = sam_hdr_t {
        n_targets: 0,
        ignore_sam_err: 0,
        l_text: len,
        target_len: std::ptr::null_mut(),
        cigar_tab: std::ptr::null(),
        target_name: std::ptr::null_mut(),
        text: _lines.cast_mut(),
        sdict: std::ptr::null_mut(),
        hrecs: std::ptr::null_mut(),
        ref_count: 0,
    };
    let ret = sam_hdr_fill_targets_from_text(&mut tmp);
    if ret < 0 {
        sam_hdr_restore_text_len(_h, old_len);
        sam_hdr_free_tmp_targets(&mut tmp);
        return ret;
    }
    if tmp.n_targets > 0 {
        let old_n = (*_h).n_targets;
        let new_n = old_n + tmp.n_targets;
        for i in 0..tmp.n_targets {
            let target_name = *tmp.target_name.add(i as usize);
            for j in 0..old_n {
                if cstr_eq(*(*_h).target_name.add(j as usize), target_name) {
                    sam_hdr_restore_text_len(_h, old_len);
                    sam_hdr_free_tmp_targets(&mut tmp);
                    return -1;
                }
            }
            for j in 0..i {
                if cstr_eq(*tmp.target_name.add(j as usize), target_name) {
                    sam_hdr_restore_text_len(_h, old_len);
                    sam_hdr_free_tmp_targets(&mut tmp);
                    return -1;
                }
            }
        }
        let target_len = crate::htslib_mini_rs::c_compat::realloc(
            (*_h).target_len.cast(),
            new_n as u64 * std::mem::size_of::<u32>() as u64,
        )
        .cast::<u32>();
        if target_len.is_null() {
            sam_hdr_restore_text_len(_h, old_len);
            sam_hdr_free_tmp_targets(&mut tmp);
            return -1;
        }
        (*_h).target_len = target_len;
        let target_name = crate::htslib_mini_rs::c_compat::realloc(
            (*_h).target_name.cast(),
            new_n as u64 * std::mem::size_of::<*mut c_char>() as u64,
        )
        .cast::<*mut c_char>();
        if target_name.is_null() {
            sam_hdr_restore_text_len(_h, old_len);
            sam_hdr_free_tmp_targets(&mut tmp);
            return -1;
        }
        (*_h).target_name = target_name;
        for i in 0..tmp.n_targets {
            let target_name = *tmp.target_name.add(i as usize);
            *(*_h).target_name.add((old_n + i) as usize) = target_name;
            *(*_h).target_len.add((old_n + i) as usize) = *tmp.target_len.add(i as usize);
            if *tmp.target_len.add(i as usize) == u32::MAX && !tmp.sdict.is_null() {
                let tmp_long_refs = tmp.sdict.cast::<khash_s2i_t>();
                let k = kh_get_s2i(tmp_long_refs, target_name);
                if k != (*tmp_long_refs).n_buckets {
                    let len = *(*tmp_long_refs).vals.add(k as usize) as hts_pos_t;
                    if len > u32::MAX as hts_pos_t
                        && sam_hdr_set_long_target_len(_h, target_name, len) < 0
                    {
                        for j in i..tmp.n_targets {
                            crate::htslib_mini_rs::c_compat::free(
                                *tmp.target_name.add(j as usize).cast::<*mut c_void>(),
                            );
                        }
                        crate::htslib_mini_rs::c_compat::free(tmp.target_name.cast());
                        crate::htslib_mini_rs::c_compat::free(tmp.target_len.cast());
                        kh_destroy_s2i(tmp.sdict.cast());
                        sam_hdr_restore_text_len(_h, old_len);
                        return -1;
                    }
                }
            }
            *tmp.target_name.add(i as usize) = std::ptr::null_mut();
        }
        (*_h).n_targets = new_n;
    }
    sam_hdr_free_tmp_targets(&mut tmp);
    ret
}

pub unsafe fn sam_hdr_find_line_id(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    id_key: *const c_char,
    id_val: *const c_char,
    ks: *mut kstring_t,
) -> c_int {
    hts_sys::sam_hdr_find_line_id(h.cast(), type_, id_key, id_val, ks.cast())
}

pub unsafe fn sam_hdr_find_line_pos(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    pos: c_int,
    ks: *mut kstring_t,
) -> c_int {
    hts_sys::sam_hdr_find_line_pos(h.cast(), type_, pos, ks.cast())
}

pub unsafe fn sam_hdr_remove_line_id(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    id_key: *const c_char,
    id_value: *const c_char,
) -> c_int {
    hts_sys::sam_hdr_remove_line_id(h.cast(), type_, id_key, id_value)
}

pub unsafe fn sam_hdr_remove_line_pos(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    position: c_int,
) -> c_int {
    hts_sys::sam_hdr_remove_line_pos(h.cast(), type_, position)
}

pub unsafe fn sam_hdr_remove_except(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    id_key: *const c_char,
    id_value: *const c_char,
) -> c_int {
    hts_sys::sam_hdr_remove_except(h.cast(), type_, id_key, id_value)
}

pub unsafe fn sam_hdr_remove_lines(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    id: *const c_char,
    rh: *mut c_void,
) -> c_int {
    hts_sys::sam_hdr_remove_lines(h.cast(), type_, id, rh)
}

pub unsafe fn sam_hdr_count_lines(h: *mut sam_hdr_t, type_: *const c_char) -> c_int {
    hts_sys::sam_hdr_count_lines(h.cast(), type_)
}

pub unsafe fn sam_hdr_line_index(
    bh: *mut sam_hdr_t,
    type_: *const c_char,
    key: *const c_char,
) -> c_int {
    hts_sys::sam_hdr_line_index(bh.cast(), type_, key)
}

pub unsafe fn sam_hdr_line_name(
    bh: *mut sam_hdr_t,
    type_: *const c_char,
    pos: c_int,
) -> *const c_char {
    hts_sys::sam_hdr_line_name(bh.cast(), type_, pos)
}

pub unsafe fn sam_hdr_find_tag_id(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    id_key: *const c_char,
    id_value: *const c_char,
    key: *const c_char,
    ks: *mut kstring_t,
) -> c_int {
    hts_sys::sam_hdr_find_tag_id(h.cast(), type_, id_key, id_value, key, ks.cast())
}

pub unsafe fn sam_hdr_find_tag_pos(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    pos: c_int,
    key: *const c_char,
    ks: *mut kstring_t,
) -> c_int {
    hts_sys::sam_hdr_find_tag_pos(h.cast(), type_, pos, key, ks.cast())
}

pub unsafe fn sam_hdr_remove_tag_id(
    h: *mut sam_hdr_t,
    type_: *const c_char,
    id_key: *const c_char,
    id_value: *const c_char,
    key: *const c_char,
) -> c_int {
    hts_sys::sam_hdr_remove_tag_id(h.cast(), type_, id_key, id_value, key)
}

pub unsafe fn sam_hdr_pg_id(h: *mut sam_hdr_t, name: *const c_char) -> *const c_char {
    hts_sys::sam_hdr_pg_id(h.cast(), name)
}

pub unsafe fn sam_hdr_parse(_l_text: usize, _text: *const c_char) -> *mut sam_hdr_t {
    let h = sam_hdr_init();
    if h.is_null() {
        return std::ptr::null_mut();
    }
    if sam_hdr_add_lines(h, _text, _l_text) != 0 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    h
}

pub unsafe fn sam_hdr_parse_(hdr: *const c_char, len: usize) -> *mut sam_hdr_t {
    sam_hdr_parse(len, hdr)
}

pub unsafe fn sam_hdr_length(_h: *mut sam_hdr_t) -> usize {
    if _h.is_null() {
        return usize::MAX;
    }
    if !(*_h).hrecs.is_null() {
        return hts_sys::sam_hdr_length(_h.cast()) as usize;
    }
    (*_h).l_text as usize
}

pub unsafe fn sam_hdr_str(_h: *mut sam_hdr_t) -> *const c_char {
    if _h.is_null() {
        return std::ptr::null();
    }
    if !(*_h).hrecs.is_null() {
        return hts_sys::sam_hdr_str(_h.cast());
    }
    (*_h).text
}

pub unsafe fn sam_hdr_nref(_h: *const sam_hdr_t) -> c_int {
    if _h.is_null() {
        return -1;
    }
    if !(*_h).hrecs.is_null() {
        return (*(*_h).hrecs).nref;
    }
    (*_h).n_targets
}

// original: KHASH_DECLARE (htslib/header.c:44)
// Translated by the concrete khash_s2i_t declarations and helpers in this file.

// original: TYPEKEY (htslib/header.h:58)
pub unsafe fn header_h_58_TYPEKEY(type_: *const c_char) -> c_uint {
    let u0 = *type_ as u8 as c_uint;
    let u1 = *type_.add(1) as u8 as c_uint;
    (u0 << 8) | u1
}

// original: known_stderr (htslib/header.c:780)
pub unsafe fn header_c_780_known_stderr(tool: *const c_char, advice: *const c_char) {
    hts_sys::hts_log(
        hts_sys::htsLogLevel_HTS_LOG_WARNING,
        c"known_stderr".as_ptr(),
        c"SAM file corrupted by embedded %s error/log message".as_ptr(),
        tool,
    );
    hts_sys::hts_log(
        hts_sys::htsLogLevel_HTS_LOG_WARNING,
        c"known_stderr".as_ptr(),
        c"%s".as_ptr(),
        advice,
    );
}

// original: warn_if_known_stderr (htslib/header.c:788)
pub unsafe fn header_c_788_warn_if_known_stderr(line: *const c_char, len: usize) {
    let ilen = if len < c_int::MAX as usize {
        len as c_int
    } else {
        c_int::MAX
    };

    if !crate::htslib_mini_rs::hts::kmemmem(
        line.cast(),
        ilen,
        c"M::bwa_idx_load_from_disk".as_ptr().cast(),
        25,
        std::ptr::null_mut(),
    )
    .is_null()
    {
        header_c_780_known_stderr(
            c"bwa".as_ptr(),
            c"Use `bwa mem -o file.sam ...` or `bwa sampe -f file.sam ...` instead of `bwa ... > file.sam`"
                .as_ptr(),
        );
    } else if !crate::htslib_mini_rs::hts::kmemmem(
        line.cast(),
        ilen,
        c"M::mem_pestat".as_ptr().cast(),
        13,
        std::ptr::null_mut(),
    )
    .is_null()
    {
        header_c_780_known_stderr(
            c"bwa".as_ptr(),
            c"Use `bwa mem -o file.sam ...` instead of `bwa mem ... > file.sam`".as_ptr(),
        );
    } else if !crate::htslib_mini_rs::hts::kmemmem(
        line.cast(),
        ilen,
        c"loaded/built the index".as_ptr().cast(),
        22,
        std::ptr::null_mut(),
    )
    .is_null()
    {
        header_c_780_known_stderr(
            c"minimap2".as_ptr(),
            c"Use `minimap2 -o file.sam ...` instead of `minimap2 ... > file.sam`".as_ptr(),
        );
    }
}

// original: valid_sam_header_type (htslib/header.c:1325)
pub unsafe fn header_c_1325_valid_sam_header_type(s: *const c_char) -> c_int {
    if *s != b'@' as c_char {
        return 0;
    }
    match *s.add(1) as u8 {
        b'H' => (*s.add(2) == b'D' as c_char && *s.add(3) == b'\t' as c_char) as c_int,
        b'S' => (*s.add(2) == b'Q' as c_char && *s.add(3) == b'\t' as c_char) as c_int,
        b'R' | b'P' => (*s.add(2) == b'G' as c_char && *s.add(3) == b'\t' as c_char) as c_int,
        b'C' => (*s.add(2) == b'O' as c_char) as c_int,
        _ => 0,
    }
}

// original: redact_header_text (htslib/header.c:1530)
pub unsafe fn header_c_1530_redact_header_text(bh: *mut sam_hdr_t) {
    (*bh).l_text = 0;
    crate::htslib_mini_rs::c_compat::free((*bh).text.cast());
    (*bh).text = std::ptr::null_mut();
}

unsafe fn sam_c_144_sam_hdr_dup_sdict(h0: *const sam_hdr_t, h: *mut sam_hdr_t) -> c_int {
    let src_long_refs = (*h0).sdict.cast::<khash_s2i_t>();
    let dest_long_refs =
        crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<khash_s2i_t>() as u64)
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
        (*dest_long_refs).flags = crate::htslib_mini_rs::c_compat::malloc(
            n_flags as u64 * std::mem::size_of::<u32>() as u64,
        )
        .cast::<u32>();
        (*dest_long_refs).keys = crate::htslib_mini_rs::c_compat::malloc(
            n_buckets as u64 * std::mem::size_of::<*mut c_char>() as u64,
        )
        .cast::<*mut c_char>();
        (*dest_long_refs).vals = crate::htslib_mini_rs::c_compat::malloc(
            n_buckets as u64 * std::mem::size_of::<i64>() as u64,
        )
        .cast::<i64>();
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
        let mut kdest = __ac_FNV1a_hash_string(key) & mask;
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

pub unsafe fn sam_hdr_dup(_h0: *const sam_hdr_t) -> *mut sam_hdr_t {
    if _h0.is_null() {
        return std::ptr::null_mut();
    }
    if !(*_h0).hrecs.is_null() {
        return hts_sys::sam_hdr_dup(_h0.cast()).cast();
    }

    let h = sam_hdr_init();
    if h.is_null() {
        return std::ptr::null_mut();
    }

    (*h).n_targets = 0;
    (*h).ignore_sam_err = (*_h0).ignore_sam_err;
    (*h).l_text = 0;

    if (*_h0).n_targets > 0 {
        (*h).target_len = crate::htslib_mini_rs::c_compat::calloc(
            (*_h0).n_targets as u64,
            std::mem::size_of::<u32>() as u64,
        )
        .cast();
        if (*h).target_len.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        (*h).target_name = crate::htslib_mini_rs::c_compat::calloc(
            (*_h0).n_targets as u64,
            std::mem::size_of::<*mut c_char>() as u64,
        )
        .cast();
        if (*h).target_name.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }

        let mut i = 0;
        while i < (*_h0).n_targets {
            *(*h).target_len.add(i as usize) = *(*_h0).target_len.add(i as usize);
            *(*h).target_name.add(i as usize) =
                crate::htslib_mini_rs::c_compat::strdup(*(*_h0).target_name.add(i as usize));
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
    (*h).text = crate::htslib_mini_rs::c_compat::malloc((*h).l_text as u64 + 1).cast();
    if (*h).text.is_null() {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    if !(*_h0).text.is_null() {
        crate::htslib_mini_rs::c_compat::memcpy(
            (*h).text.cast(),
            (*_h0).text.cast(),
            (*h).l_text as u64,
        );
    }
    *(*h).text.add((*h).l_text as usize) = 0;

    if !(*_h0).sdict.is_null() && sam_c_144_sam_hdr_dup_sdict(_h0, h) < 0 {
        sam_hdr_destroy(h);
        return std::ptr::null_mut();
    }

    h
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
    (*h).text = crate::htslib_mini_rs::c_compat::malloc(bufsize as _).cast();
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
        (*h).target_name = crate::htslib_mini_rs::c_compat::calloc(
            (*h).n_targets as u64,
            std::mem::size_of::<*mut c_char>() as u64,
        )
        .cast();
        if (*h).target_name.is_null() {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        (*h).target_len = crate::htslib_mini_rs::c_compat::calloc(
            (*h).n_targets as u64,
            std::mem::size_of::<u32>() as u64,
        )
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
            crate::htslib_mini_rs::c_compat::malloc(name_len as _).cast();
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
            if name_len == c_int::MAX {
                (*h).n_targets = num_names;
                sam_hdr_destroy(h);
                return std::ptr::null_mut();
            }
            let new_name = crate::htslib_mini_rs::c_compat::realloc(
                (*(*h).target_name.add(i as usize)).cast(),
                (name_len as usize + 1) as _,
            )
            .cast::<c_char>();
            if new_name.is_null() {
                (*h).n_targets = num_names;
                sam_hdr_destroy(h);
                return std::ptr::null_mut();
            }
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

pub unsafe fn bam_hdr_write(fp: *mut BGZF, h: *const sam_hdr_t) -> c_int {
    if h.is_null() {
        return -1;
    }
    if !(*h).hrecs.is_null() {
        return hts_sys::bam_hdr_write(fp.cast(), h.cast());
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
        ed_swap_4p((&mut n_targets as *mut c_int).cast());
    }
    if bgzf_write(fp, (&n_targets as *const c_int).cast(), 4) < 0 {
        return -1;
    }

    for i in 0..(*h).n_targets {
        let p = *(*h).target_name.add(i as usize);
        let mut name_len = libc::strlen(p) as c_int + 1;
        if is_be {
            ed_swap_4p((&mut name_len as *mut c_int).cast());
        }
        if bgzf_write(fp, (&name_len as *const c_int).cast(), 4) < 0 {
            return -1;
        }
        let write_name_len = libc::strlen(p) + 1;
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

pub unsafe fn sam_hdr_write(fp: *mut htsFile, h: *const sam_hdr_t) -> c_int {
    if fp.is_null() || h.is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
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
            if !(*h).hrecs.is_null() {
                return hts_sys::sam_hdr_write(fp.cast(), h.cast());
            }
            if !(*h).text.is_null() {
                let text = (*h).text;
                let l_text = (*h).l_text;
                let mut q = text;
                let p = loop {
                    let p = libc::strstr(q, c"@SQ\t".as_ptr());
                    if p.is_null() || p == text || *p.sub(1) == b'\n' as c_char {
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
                        (*fp).line.l = 0;
                        if kputsn(c"@SQ\tSN:".as_ptr(), 7, &mut (*fp).line) < 0
                            || kputs(*(*h).target_name.add(i as usize), &mut (*fp).line) < 0
                            || kputsn(c"\tLN:".as_ptr(), 4, &mut (*fp).line) < 0
                            || kputw(*(*h).target_len.add(i as usize) as c_int, &mut (*fp).line) < 0
                            || kputc(b'\n' as c_int, &mut (*fp).line) < 0
                        {
                            return -1;
                        }
                        if sam_hdr_write_bytes(fp, (*fp).line.s.cast(), (*fp).line.l) < 0 {
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
            *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EBADF as c_int;
            return -1;
        }
    }

    sam_hdr_write_store_copy(fp, h)
}

unsafe fn sam_hdr_write_cram(fp: *mut htsFile, h: *const sam_hdr_t) -> c_int {
    if !(*h).hrecs.is_null() {
        return hts_sys::sam_hdr_write(fp.cast(), h.cast());
    }
    if (*h).text.is_null() {
        return 0;
    }

    let text = std::slice::from_raw_parts((*h).text.cast::<u8>(), (*h).l_text);
    let mut no_sq = true;
    let mut start = 0usize;
    while let Some(pos) = text[start..].windows(4).position(|w| w == b"@SQ\t") {
        let abs = start + pos;
        if abs == 0 || text[abs - 1] == b'\n' {
            no_sq = false;
            break;
        }
        start = abs + 4;
    }

    let mut header_text = text.to_vec();
    if no_sq {
        for i in 0..(*h).n_targets {
            let name = *(*h).target_name.add(i as usize);
            if name.is_null() {
                continue;
            }
            header_text.extend_from_slice(b"@SQ\tSN:");
            header_text.extend_from_slice(CStr::from_ptr(name).to_bytes());
            header_text.extend_from_slice(b"\tLN:");
            header_text
                .extend_from_slice((*(*h).target_len.add(i as usize)).to_string().as_bytes());
            header_text.push(b'\n');
        }
    }

    let hts_hdr = hts_sys::sam_hdr_parse(header_text.len() as u64, header_text.as_ptr().cast());
    if hts_hdr.is_null() {
        return -1;
    }
    let ret = hts_sys::sam_hdr_write(fp.cast(), hts_hdr);
    hts_sys::sam_hdr_destroy(hts_hdr);
    ret
}

unsafe fn sam_hdr_write_bytes(fp: *mut htsFile, bytes: *const c_void, len: usize) -> c_int {
    if len == 0 {
        return 0;
    }
    if ((*fp).bitfields & (1 << 4)) != 0 {
        if bgzf_write((*fp).fp.bgzf, bytes, len) == len as isize {
            0
        } else {
            -1
        }
    } else if crate::htslib_mini_rs::hfile::htslib_hfile_h_292_hwrite((*fp).fp.hfile, bytes, len)
        == len as libc::ssize_t
    {
        0
    } else {
        -1
    }
}

unsafe fn sam_hdr_write_store_copy(fp: *mut htsFile, h: *const sam_hdr_t) -> c_int {
    let tmp = (*fp).bam_header.cast::<sam_hdr_t>();
    (*fp).bam_header = sam_hdr_dup(h).cast();
    sam_hdr_destroy(tmp);
    if (*fp).bam_header.is_null() {
        -1
    } else {
        0
    }
}

unsafe fn sam_hdr_sanitise(_h: *mut sam_hdr_t) -> *mut sam_hdr_t {
    if _h.is_null() {
        return std::ptr::null_mut();
    }
    if (*_h).l_text == 0 {
        return _h;
    }

    let mut i = 0usize;
    let mut last = b'\n' as c_char;
    while i < (*_h).l_text {
        let ch = *(*_h).text.add(i);
        if ch == 0 {
            break;
        }
        if last == b'\n' as c_char && ch != b'@' as c_char {
            sam_hdr_destroy(_h);
            return std::ptr::null_mut();
        }
        last = ch;
        i += 1;
    }

    if last != b'\n' as c_char {
        if (*_h).l_text < 2 || i >= (*_h).l_text - 2 {
            if (*_h).l_text >= usize::MAX - 2 {
                sam_hdr_destroy(_h);
                return std::ptr::null_mut();
            }
            let cp = crate::htslib_mini_rs::c_compat::realloc(
                (*_h).text.cast(),
                ((*_h).l_text + 2) as _,
            )
            .cast::<c_char>();
            if cp.is_null() {
                sam_hdr_destroy(_h);
                return std::ptr::null_mut();
            }
            (*_h).text = cp;
        }
        *(*_h).text.add(i) = b'\n' as c_char;
        i += 1;
        if (*_h).l_text < i {
            (*_h).l_text = i;
        }
        *(*_h).text.add((*_h).l_text) = 0;
    }

    _h
}

unsafe fn sam_c_1907_sam_hdr_create(fp: *mut htsFile) -> *mut sam_hdr_t {
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
                nc as c_int
            } else {
                pret as c_int - 1
            }
        };
        if next_c != b'@' as c_int {
            (*fp).line.l = 0;
            break;
        }

        let ret = hts_sys::hts_getline(
            fp.cast(),
            2,
            (&mut (*fp).line as *mut crate::htslib_mini_rs::hts::kstring_t).cast(),
        );
        if ret < 0 {
            if ret < -1 {
                sam_hdr_destroy(h);
                return std::ptr::null_mut();
            }
            break;
        }
        if (*fp).line.l == 0 || *(*fp).line.s != b'@' as c_char {
            (*fp).line.l = 0;
            break;
        }
        if sam_hdr_add_lines(h, (*fp).line.s, (*fp).line.l) != 0
            || sam_hdr_add_lines(h, b"\n".as_ptr().cast(), 1) != 0
        {
            sam_hdr_destroy(h);
            return std::ptr::null_mut();
        }
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

pub unsafe fn sam_hdr_read(_fp: *mut htsFile) -> *mut sam_hdr_t {
    if _fp.is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return std::ptr::null_mut();
    }
    let h = match (*_fp).format.format {
        HTS_FORMAT_BAM => sam_hdr_sanitise(bam_hdr_read((*_fp).fp.bgzf)),
        HTS_FORMAT_FASTQ_FORMAT | HTS_FORMAT_FASTA_FORMAT => sam_hdr_init(),
        HTS_FORMAT_EMPTY_FORMAT => {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EPIPE as c_int;
            return std::ptr::null_mut();
        }
        HTS_FORMAT_SAM => sam_c_1907_sam_hdr_create(_fp),
        HTS_FORMAT_CRAM => hts_sys::sam_hdr_read(_fp.cast()).cast(),
        _ => {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::ENOEXEC as c_int;
            return std::ptr::null_mut();
        }
    };

    if !h.is_null() && (*_fp).bam_header.is_null() {
        (*_fp).bam_header = h.cast();
        sam_hdr_incr_ref(h);
    }
    h
}

pub unsafe fn sam_hdr_destroy(_h: *mut sam_hdr_t) {
    if _h.is_null() {
        return;
    }
    if (*_h).ref_count > 0 {
        (*_h).ref_count -= 1;
        return;
    }
    if !(*_h).hrecs.is_null() {
        hts_sys::sam_hdr_destroy(_h.cast());
        return;
    }
    if !(*_h).target_name.is_null() {
        for i in 0..(*_h).n_targets {
            crate::htslib_mini_rs::c_compat::free(
                *(*_h).target_name.add(i as usize).cast::<*mut c_void>(),
            );
        }
        crate::htslib_mini_rs::c_compat::free((*_h).target_name.cast());
        crate::htslib_mini_rs::c_compat::free((*_h).target_len.cast());
    }
    crate::htslib_mini_rs::c_compat::free((*_h).text.cast());
    kh_destroy_s2i((*_h).sdict.cast());
    crate::htslib_mini_rs::c_compat::free(_h.cast());
}

pub unsafe fn sam_hdr_free(hdr: *mut sam_hdr_t) {
    sam_hdr_destroy(hdr);
}

pub unsafe fn sam_hdr_incr_ref(bh: *mut sam_hdr_t) {
    if bh.is_null() {
        return;
    }
    (*bh).ref_count = (*bh).ref_count.wrapping_add(1);
}

unsafe fn sam_c_2080_old_sam_hdr_change_HD(
    h: *mut sam_hdr_t,
    key: *const c_char,
    val: *const c_char,
) -> c_int {
    if h.is_null() || key.is_null() {
        return -1;
    }

    let key_bytes = CStr::from_ptr(key).to_bytes();
    let val_bytes = if val.is_null() {
        None
    } else {
        Some(CStr::from_ptr(val).to_bytes())
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

    let newtext =
        crate::htslib_mini_rs::c_compat::malloc(new_text.len() as u64 + 1).cast::<c_char>();
    if newtext.is_null() {
        return -1;
    }
    if !new_text.is_empty() {
        std::ptr::copy_nonoverlapping(new_text.as_ptr(), newtext.cast::<u8>(), new_text.len());
    }
    *newtext.add(new_text.len()) = 0;
    crate::htslib_mini_rs::c_compat::free((*h).text.cast());
    (*h).text = newtext;
    (*h).l_text = new_text.len();
    0
}

pub unsafe fn sam_hdr_change_HD(
    h: *mut sam_hdr_t,
    key: *const c_char,
    val: *const c_char,
) -> c_int {
    if h.is_null() || key.is_null() {
        return -1;
    }
    if (*h).hrecs.is_null() {
        return sam_c_2080_old_sam_hdr_change_HD(h, key, val);
    }
    hts_sys::sam_hdr_change_HD(h.cast(), key, val)
}

pub unsafe fn sam_hdr_set(fp: *mut htsFile, h: *mut sam_hdr_t, duplicate: c_int) -> c_int {
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
        sam_hdr_incr_ref((*fp).bam_header.cast());
    }

    0
}

pub unsafe fn sam_hdr_get(fp: *mut htsFile) -> *mut sam_hdr_t {
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    (*fp).bam_header.cast()
}

unsafe fn sam_c_1173_bam_get_library(h: *const sam_hdr_t, b: *const bam1_t) -> *const c_char {
    static mut LB_TEXT: [c_char; 1024] = [0; 1024];

    let mut lib = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let rg = bam_aux_get(b, c"RG".as_ptr());
    if rg.is_null() {
        return std::ptr::null();
    }

    if hts_sys::sam_hdr_find_tag_id(
        h.cast_mut().cast(),
        c"RG".as_ptr(),
        c"ID".as_ptr(),
        rg.add(1).cast(),
        c"LB".as_ptr(),
        (&mut lib as *mut kstring_t).cast(),
    ) < 0
    {
        return std::ptr::null();
    }

    let len = if lib.l < 1023 { lib.l } else { 1023 };
    let lb_text = std::ptr::addr_of_mut!(LB_TEXT).cast::<c_char>();
    if len > 0 {
        std::ptr::copy_nonoverlapping(lib.s, lb_text, len);
    }
    *lb_text.add(len) = 0;
    crate::htslib_mini_rs::c_compat::free(lib.s.cast());
    lb_text.cast_const()
}

unsafe fn sam_c_2221_grow_B_array(b: *mut bam1_t, n: *mut u32, size: usize) -> i64 {
    if *n > (c_int::MAX as f64 * 0.666) as u32 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
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
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 1) < 0 {
            return std::ptr::null_mut();
        }
        *(*b).data.add((*b).l_data as usize) = hts_str2int(q.add(1), &mut q, 8, overflow) as u8;
        (*b).l_data += 1;
    }
    q
}

unsafe fn sam_c_2258_sam_parse_BC_vals(
    b: *mut bam1_t,
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 1) < 0 {
            return std::ptr::null_mut();
        }
        if *q.add(1) != b'-' as c_char {
            *(*b).data.add((*b).l_data as usize) =
                hts_str2uint(q.add(1), &mut q, 8, overflow) as u8;
            (*b).l_data += 1;
        } else {
            *overflow = 1;
            q = q.add(1);
            while *q > b'\t' as c_char && *q != b',' as c_char {
                q = q.add(1);
            }
        }
    }
    q
}

unsafe fn sam_c_2278_sam_parse_Bs_vals(
    b: *mut bam1_t,
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 2) < 0 {
            return std::ptr::null_mut();
        }
        i16_to_le(
            hts_str2int(q.add(1), &mut q, 16, overflow) as i16,
            (*b).data.add((*b).l_data as usize),
        );
        (*b).l_data += 2;
    }
    q
}

unsafe fn sam_c_2293_sam_parse_BS_vals(
    b: *mut bam1_t,
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 2) < 0 {
            return std::ptr::null_mut();
        }
        if *q.add(1) != b'-' as c_char {
            u16_to_le(
                hts_str2uint(q.add(1), &mut q, 16, overflow) as u16,
                (*b).data.add((*b).l_data as usize),
            );
            (*b).l_data += 2;
        } else {
            *overflow = 1;
            q = q.add(1);
            while *q > b'\t' as c_char && *q != b',' as c_char {
                q = q.add(1);
            }
        }
    }
    q
}

unsafe fn sam_c_2314_sam_parse_Bi_vals(
    b: *mut bam1_t,
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 4) < 0 {
            return std::ptr::null_mut();
        }
        i32_to_le(
            hts_str2int(q.add(1), &mut q, 32, overflow) as i32,
            (*b).data.add((*b).l_data as usize),
        );
        (*b).l_data += 4;
    }
    q
}

unsafe fn sam_c_2329_sam_parse_BI_vals(
    b: *mut bam1_t,
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 4) < 0 {
            return std::ptr::null_mut();
        }
        if *q.add(1) != b'-' as c_char {
            u32_to_le(
                hts_str2uint(q.add(1), &mut q, 32, overflow) as u32,
                (*b).data.add((*b).l_data as usize),
            );
            (*b).l_data += 4;
        } else {
            *overflow = 1;
            q = q.add(1);
            while *q > b'\t' as c_char && *q != b',' as c_char {
                q = q.add(1);
            }
        }
    }
    q
}

unsafe fn sam_c_2350_sam_parse_Bf_vals(
    b: *mut bam1_t,
    mut q: *mut c_char,
    nused: *mut u32,
    nalloc: *mut u32,
    _overflow: *mut c_int,
) -> *mut c_char {
    while *q == b',' as c_char {
        *nused += 1;
        if *nused - 1 >= *nalloc && sam_c_2221_grow_B_array(b, nalloc, 4) < 0 {
            return std::ptr::null_mut();
        }
        let mut end: *mut c_char = std::ptr::null_mut();
        let val = libc::strtod(q.add(1), &mut end);
        q = end;
        float_to_le(val as f32, (*b).data.add((*b).l_data as usize));
        (*b).l_data += 4;
    }
    q
}

unsafe fn sam_c_2364_sam_parse_B_vals_r(
    type_: c_char,
    mut nalloc: u32,
    in_: *mut c_char,
    end: *mut *mut c_char,
    b: *mut bam1_t,
    ctr: *mut c_int,
) -> c_int {
    *ctr += 1;
    if *ctr > 2 {
        return -1;
    }

    let orig_l = (*b).l_data;
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
    *(*b).data.add((*b).l_data as usize) = b'B';
    (*b).l_data += 1;
    *(*b).data.add((*b).l_data as usize) = type_ as u8;
    (*b).l_data += 1;
    let b_len_idx = (*b).l_data;
    (*b).l_data += std::mem::size_of::<u32>() as c_int;

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
    if *q != b'\t' as c_char && *q != 0 {
        return -1;
    }
    i32_to_le(nused as i32, (*b).data.add(b_len_idx as usize));

    if overflow == 0 {
        *end = q;
        return 0;
    }

    let r = q;
    q = in_;
    overflow = 0;
    (*b).l_data = orig_l;
    let mut max = 0i64;
    let mut min = 0i64;
    while q < r {
        let val = hts_str2int(q.add(1), &mut q, 64, &mut overflow);
        if max < val {
            max = val;
        }
        if min > val {
            min = val;
        }
        while *q > b'\t' as c_char && *q != b',' as c_char {
            q = q.add(1);
        }
    }

    if overflow == 0 {
        if min < 0 {
            if min >= i8::MIN as i64 && max <= i8::MAX as i64 {
                return sam_c_2364_sam_parse_B_vals_r(b'c' as c_char, nalloc, in_, end, b, ctr);
            } else if min >= i16::MIN as i64 && max <= i16::MAX as i64 {
                return sam_c_2364_sam_parse_B_vals_r(b's' as c_char, nalloc, in_, end, b, ctr);
            } else if min >= i32::MIN as i64 && max <= i32::MAX as i64 {
                return sam_c_2364_sam_parse_B_vals_r(b'i' as c_char, nalloc, in_, end, b, ctr);
            }
        } else if max < u8::MAX as i64 {
            return sam_c_2364_sam_parse_B_vals_r(b'C' as c_char, nalloc, in_, end, b, ctr);
        } else if max <= u16::MAX as i64 {
            return sam_c_2364_sam_parse_B_vals_r(b'S' as c_char, nalloc, in_, end, b, ctr);
        } else if max <= u32::MAX as i64 {
            return sam_c_2364_sam_parse_B_vals_r(b'I' as c_char, nalloc, in_, end, b, ctr);
        }
    }
    -1
}

unsafe fn sam_c_2490_sam_parse_B_vals(
    type_: c_char,
    in_: *mut c_char,
    end: *mut *mut c_char,
    b: *mut bam1_t,
) -> c_int {
    let mut ctr = 0;
    sam_c_2364_sam_parse_B_vals_r(type_, 0, in_, end, b, &mut ctr)
}

unsafe fn sam_c_2498_parse_sam_flag(
    v: *mut c_char,
    rv: *mut *mut c_char,
    overflow: *mut c_int,
) -> u32 {
    if *v >= b'1' as c_char && *v <= b'9' as c_char {
        hts_str2uint(v, rv, 16, overflow) as u32
    } else if *v == b'0' as c_char {
        if *v.add(1) == b'\t' as c_char {
            *rv = v.add(1);
            0
        } else {
            let val = libc::strtoul(v, rv, 0);
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
    start: *mut c_char,
    end: *mut c_char,
    b: *mut bam1_t,
    lenient: c_int,
    tag_whitelist: *mut c_void,
) -> c_int {
    let mut overflow = 0;
    let mut q = start;
    let p = end;

    'loop_: while q < p {
        let checkpoint = (*b).l_data;
        let parse_err = |cond: bool| cond;

        if p.offset_from(q) < 5 {
            if lenient != 0 {
                break;
            }
            return -2;
        }
        if parse_err(*q < b'!' as c_char || *q.add(1) < b'!' as c_char) {
            if lenient != 0 {
                while q < p && isspace_c(*q) == 0 {
                    q = q.add(1);
                }
                while q < p && isspace_c(*q) != 0 {
                    q = q.add(1);
                }
                (*b).l_data = checkpoint;
                continue 'loop_;
            }
            return -2;
        }

        if lenient != 0 && ((*q.add(2) as u8) | (*q.add(4) as u8)) != b':' {
            while q < p && isspace_c(*q) == 0 {
                q = q.add(1);
            }
            while q < p && isspace_c(*q) != 0 {
                q = q.add(1);
            }
            continue;
        }

        if !tag_whitelist.is_null() {
            let tt = (*q as c_int) * 256 + *q.add(1) as c_int;
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
                while q < p && *q != b'\t' as c_char {
                    q = q.add(1);
                }
                continue;
            }
        }

        if possibly_expand_bam_data(b, 2) < 0 {
            return -2;
        }
        *(*b).data.add((*b).l_data as usize) = *q as u8;
        (*b).l_data += 1;
        *(*b).data.add((*b).l_data as usize) = *q.add(1) as u8;
        (*b).l_data += 1;

        q = q.add(3);
        let mut type_ = *q;
        q = q.add(2);
        if type_ != b'Z' as c_char && type_ != b'H' as c_char && *q <= b'\t' as c_char {
            if lenient != 0 {
                while q < p && isspace_c(*q) == 0 {
                    q = q.add(1);
                }
                while q < p && isspace_c(*q) != 0 {
                    q = q.add(1);
                }
                (*b).l_data = checkpoint;
                continue 'loop_;
            }
            return -2;
        }

        if possibly_expand_bam_data(b, 16) < 0 {
            return -2;
        }

        if matches!(type_ as u8, b'A' | b'a' | b'c' | b'C') {
            *(*b).data.add((*b).l_data as usize) = b'A';
            (*b).l_data += 1;
            *(*b).data.add((*b).l_data as usize) = *q as u8;
            (*b).l_data += 1;
            q = q.add(1);
        } else if matches!(type_ as u8, b'i' | b'I') {
            if *q == b'-' as c_char {
                let x = hts_str2int(q, &mut q, 32, &mut overflow);
                if x >= i8::MIN as i64 {
                    *(*b).data.add((*b).l_data as usize) = b'c';
                    (*b).l_data += 1;
                    *(*b).data.add((*b).l_data as usize) = x as u8;
                    (*b).l_data += 1;
                } else if x >= i16::MIN as i64 {
                    *(*b).data.add((*b).l_data as usize) = b's';
                    (*b).l_data += 1;
                    i16_to_le(x as i16, (*b).data.add((*b).l_data as usize));
                    (*b).l_data += 2;
                } else {
                    *(*b).data.add((*b).l_data as usize) = b'i';
                    (*b).l_data += 1;
                    i32_to_le(x as i32, (*b).data.add((*b).l_data as usize));
                    (*b).l_data += 4;
                }
            } else {
                let x = hts_str2uint(q, &mut q, 32, &mut overflow);
                if x <= u8::MAX as u64 {
                    *(*b).data.add((*b).l_data as usize) = b'C';
                    (*b).l_data += 1;
                    *(*b).data.add((*b).l_data as usize) = x as u8;
                    (*b).l_data += 1;
                } else if x <= u16::MAX as u64 {
                    *(*b).data.add((*b).l_data as usize) = b'S';
                    (*b).l_data += 1;
                    u16_to_le(x as u16, (*b).data.add((*b).l_data as usize));
                    (*b).l_data += 2;
                } else {
                    *(*b).data.add((*b).l_data as usize) = b'I';
                    (*b).l_data += 1;
                    u32_to_le(x as u32, (*b).data.add((*b).l_data as usize));
                    (*b).l_data += 4;
                }
            }
        } else if type_ == b'f' as c_char {
            *(*b).data.add((*b).l_data as usize) = b'f';
            (*b).l_data += 1;
            let value = libc::strtod(q, &mut q);
            float_to_le(value as f32, (*b).data.add((*b).l_data as usize));
            (*b).l_data += std::mem::size_of::<f32>() as c_int;
        } else if type_ == b'd' as c_char {
            *(*b).data.add((*b).l_data as usize) = b'd';
            (*b).l_data += 1;
            let value = libc::strtod(q, &mut q);
            double_to_le(value, (*b).data.add((*b).l_data as usize));
            (*b).l_data += std::mem::size_of::<f64>() as c_int;
        } else if matches!(type_ as u8, b'Z' | b'H') {
            let mut zend = q;
            while zend < p && *zend != b'\t' as c_char && *zend != 0 {
                zend = zend.add(1);
            }
            if type_ == b'H' as c_char && (zend.offset_from(q) & 1) != 0 {
                if lenient != 0 {
                    while q < p && isspace_c(*q) == 0 {
                        q = q.add(1);
                    }
                    while q < p && isspace_c(*q) != 0 {
                        q = q.add(1);
                    }
                    (*b).l_data = checkpoint;
                    continue 'loop_;
                }
                return -2;
            }
            *(*b).data.add((*b).l_data as usize) = type_ as u8;
            (*b).l_data += 1;
            let zlen = zend.offset_from(q) as usize;
            if possibly_expand_bam_data(b, zlen + 1) < 0 {
                return -2;
            }
            crate::htslib_mini_rs::c_compat::memcpy(
                (*b).data.add((*b).l_data as usize).cast(),
                q.cast(),
                zlen as u64,
            );
            (*b).l_data += zlen as c_int;
            *(*b).data.add((*b).l_data as usize) = 0;
            (*b).l_data += 1;
            q = zend;
        } else if type_ == b'B' as c_char {
            type_ = *q;
            q = q.add(1);
            if *q != 0 && *q != b',' as c_char && *q != b'\t' as c_char {
                if lenient != 0 {
                    while q < p && isspace_c(*q) == 0 {
                        q = q.add(1);
                    }
                    while q < p && isspace_c(*q) != 0 {
                        q = q.add(1);
                    }
                    (*b).l_data = checkpoint;
                    continue 'loop_;
                }
                return -2;
            }
            if sam_c_2490_sam_parse_B_vals(type_, q, &mut q, b) < 0 {
                return -2;
            }
        } else if lenient != 0 {
            while q < p && isspace_c(*q) == 0 {
                q = q.add(1);
            }
            while q < p && isspace_c(*q) != 0 {
                q = q.add(1);
            }
            (*b).l_data = checkpoint;
            continue;
        } else {
            return -2;
        }

        while q < p && *q > b'\t' as c_char {
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

pub unsafe fn sam_c_2662_sam_parse1(s: *mut kstring_t, h: *mut sam_hdr_t, b: *mut bam1_t) -> c_int {
    let mut p = (*s).s;
    let mut q: *mut c_char;
    let mut overflow = 0;
    let c = &mut (*b).core;

    macro_rules! read_token {
        ($p:ident) => {{
            let token = $p;
            let tab = libc::strchr($p, b'\t' as c_int);
            if tab.is_null() {
                return -2;
            }
            *tab = 0;
            $p = tab.add(1);
            token
        }};
    }

    (*b).l_data = 0;
    std::ptr::write_bytes((c as *mut bam1_core_t).cast::<u8>(), 0, 32);

    q = read_token!(p);
    if p.offset_from(q) > 255 {
        return -2;
    }
    if possibly_expand_bam_data(b, p.offset_from(q) as usize + 4) < 0 {
        return -2;
    }
    crate::htslib_mini_rs::c_compat::memcpy(
        (*b).data.add((*b).l_data as usize).cast(),
        q.cast(),
        p.offset_from(q) as u64,
    );
    (*b).l_data += p.offset_from(q) as c_int;
    c.l_extranul = ((4 - ((*b).l_data & 3)) & 3) as u8;
    std::ptr::write_bytes(
        (*b).data.add((*b).l_data as usize),
        0,
        c.l_extranul as usize,
    );
    (*b).l_data += c.l_extranul as c_int;
    c.l_qname = (p.offset_from(q) as c_int + c.l_extranul as c_int) as u16;

    c.flag = sam_c_2498_parse_sam_flag(p, &mut p, &mut overflow) as u16;
    if *p != b'\t' as c_char {
        return -2;
    }
    p = p.add(1);

    q = read_token!(p);
    if libc::strcmp(q, c"*".as_ptr()) != 0 {
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

    c.pos = hts_str2uint(p, &mut p, 62, &mut overflow) as hts_pos_t - 1;
    if *p != b'\t' as c_char {
        return -2;
    }
    p = p.add(1);
    if c.pos < 0 && c.tid >= 0 {
        c.tid = -1;
    }
    if c.tid < 0 {
        c.flag |= BAM_FUNMAP as u16;
    }

    c.qual = hts_str2uint(p, &mut p, 8, &mut overflow) as u8;
    if *p != b'\t' as c_char {
        return -2;
    }
    p = p.add(1);

    let cigreflen;
    if *p != b'*' as c_char {
        let old_l_data = (*b).l_data;
        let n_cigar = bam_parse_cigar(p, &mut p, b);
        if n_cigar < 1 || *p != b'\t' as c_char {
            return -2;
        }
        p = p.add(1);
        let cigar = (*b).data.add(old_l_data as usize).cast::<u32>();
        cigreflen = if (c.flag as c_int & BAM_FUNMAP) == 0 {
            bam_cigar2rlen(c.n_cigar as c_int, cigar)
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
    if libc::strcmp(q, c"=".as_ptr()) == 0 {
        c.mtid = c.tid;
    } else if libc::strcmp(q, c"*".as_ptr()) == 0 {
        c.mtid = -1;
    } else {
        c.mtid = bam_name2id(h, q);
        if c.mtid < -1 {
            return -2;
        }
    }

    c.mpos = hts_str2uint(p, &mut p, 62, &mut overflow) as hts_pos_t - 1;
    if *p != b'\t' as c_char {
        return -2;
    }
    p = p.add(1);
    if c.mpos < 0 && c.mtid >= 0 {
        c.mtid = -1;
    }

    c.isize = hts_str2int(p, &mut p, 63, &mut overflow);
    if *p != b'\t' as c_char {
        return -2;
    }
    p = p.add(1);
    if overflow != 0 {
        return -2;
    }

    q = read_token!(p);
    if libc::strcmp(q, c"*".as_ptr()) != 0 {
        let seq_len = p.offset_from(q) - 1;
        if seq_len > c_int::MAX as isize {
            return -2;
        }
        c.l_qseq = seq_len as c_int;
        let ql = bam_cigar2qlen(c.n_cigar as c_int, (*b).data.add(c.l_qname as usize).cast());
        if c.n_cigar != 0 && ql != c.l_qseq as hts_pos_t {
            return -2;
        }
        let seq_bytes = ((c.l_qseq + 1) >> 1) as usize;
        if possibly_expand_bam_data(b, seq_bytes) < 0 {
            return -2;
        }
        let t = (*b).data.add((*b).l_data as usize);
        (*b).l_data += seq_bytes as c_int;
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
    let t = (*b).data.add((*b).l_data as usize);
    (*b).l_data += c.l_qseq;
    if *p == b'*' as c_char && (*p.add(1) == b'\t' as c_char || *p.add(1) == 0) {
        std::ptr::write_bytes(t, 0xff, c.l_qseq as usize);
        p = p.add(2);
    } else {
        if (*s).l < p.offset_from((*s).s) as usize + c.l_qseq as usize
            || (*p.add(c.l_qseq as usize) != b'\t' as c_char && *p.add(c.l_qseq as usize) != 0)
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

    if sam_c_2524_aux_parse(p, (*s).s.add((*s).l), b, 0, std::ptr::null_mut()) < 0 {
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
    let fd = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<SAM_state>() as u64)
        .cast::<SAM_state>();
    if fd.is_null() {
        return std::ptr::null_mut();
    }
    (*fp).state = fd.cast();
    (*fd).fp = fp;
    fd
}

unsafe fn sam_c_3069_sam_state_err(fd: *mut SAM_state, errcode: c_int) {
    if !fd.is_null() && (*fd).errcode == 0 {
        (*fd).errcode = errcode;
    }
}

unsafe fn sam_c_3076_sam_free_sp_bams(b: *mut sp_bams) {
    if b.is_null() {
        return;
    }
    if !(*b).bams.is_null() {
        for i in 0..(*b).abams {
            let bam = (*b).bams.add(i as usize);
            if !(*bam).data.is_null() {
                crate::htslib_mini_rs::c_compat::free((*bam).data.cast());
            }
        }
        crate::htslib_mini_rs::c_compat::free((*b).bams.cast());
    }
    crate::htslib_mini_rs::c_compat::free(b.cast());
}

unsafe extern "C" fn sam_c_3200_cleanup_sp_lines(arg: *mut c_void) {
    let gl = arg.cast::<sp_lines>();
    if gl.is_null() {
        return;
    }

    assert!((*gl).next.is_null());

    crate::htslib_mini_rs::c_compat::free((*gl).data.cast());
    sam_c_3076_sam_free_sp_bams((*gl).bams);
    crate::htslib_mini_rs::c_compat::free(gl.cast());
}

unsafe extern "C" fn sam_c_3313_sam_parse_eof(_arg: *mut c_void) -> *mut c_void {
    std::ptr::null_mut()
}

unsafe extern "C" fn sam_c_3318_cleanup_sp_bams(arg: *mut c_void) {
    sam_c_3076_sam_free_sp_bams(arg.cast::<sp_bams>());
}

unsafe extern "C" fn sam_c_3215_sam_parse_worker(arg: *mut c_void) -> *mut c_void {
    let gl = arg.cast::<sp_lines>();
    let mut gb = std::ptr::null_mut::<sp_bams>();
    let lines = (*gl).data;
    let fd = (*gl).fd;

    if !fd.is_null() && !(*fd).bams.is_null() {
        gb = (*fd).bams;
        (*fd).bams = (*gb).next;
    }

    if gb.is_null() {
        gb = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_bams>() as u64)
            .cast::<sp_bams>();
        if gb.is_null() {
            return std::ptr::null_mut();
        }
        (*gb).abams = 100;
        (*gb).bams = crate::htslib_mini_rs::c_compat::calloc(
            (*gb).abams as u64,
            std::mem::size_of::<bam1_t>() as u64,
        )
        .cast::<bam1_t>();
        if (*gb).bams.is_null() {
            sam_c_3069_sam_state_err(fd, crate::htslib_mini_rs::c_compat::ENOMEM as c_int);
            sam_c_3076_sam_free_sp_bams(gb);
            return std::ptr::null_mut();
        }
        (*gb).nbams = 0;
        (*gb).bam_mem = 0;
    }
    (*gb).serial = (*gl).serial;
    (*gb).next = std::ptr::null_mut();

    let mut b = (*gb).bams;
    if b.is_null() {
        sam_c_3069_sam_state_err(fd, crate::htslib_mini_rs::c_compat::ENOMEM as c_int);
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
            b = crate::htslib_mini_rs::c_compat::realloc(
                (*gb).bams.cast(),
                (*gb).abams as u64 * std::mem::size_of::<bam1_t>() as u64,
            )
            .cast::<bam1_t>();
            if b.is_null() {
                (*gb).abams = old_abams;
                sam_c_3069_sam_state_err(fd, crate::htslib_mini_rs::c_compat::ENOMEM as c_int);
                sam_c_3076_sam_free_sp_bams(gb);
                return std::ptr::null_mut();
            }
            std::ptr::write_bytes(
                b.add(old_abams as usize).cast::<u8>(),
                0,
                ((*gb).abams - old_abams) as usize * std::mem::size_of::<bam1_t>(),
            );
            (*gb).bams = b;
        }

        let mut nl = cp;
        while nl < cp_end && *nl != b'\n' as c_char {
            nl = nl.add(1);
        }
        let mut line_end = nl;
        let next = if nl < cp_end { nl.add(1) } else { cp_end };
        if line_end > cp && *line_end.sub(1) == b'\r' as c_char {
            line_end = line_end.sub(1);
        }
        *line_end = 0;
        let mut ks = kstring_t {
            l: line_end.offset_from(cp) as usize,
            m: (*gl).alloc as usize,
            s: cp,
        };
        if sam_c_2662_sam_parse1(&mut ks, (*fd).h, b.add(i as usize)) < 0 {
            let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
            sam_c_3069_sam_state_err(
                fd,
                if errno != 0 {
                    errno
                } else {
                    libc::EIO as c_int
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
        (*fd).lines = gl;
    }
    gb.cast()
}

unsafe extern "C" fn sam_c_3652_sam_format_worker(arg: *mut c_void) -> *mut c_void {
    let gb = arg.cast::<sp_bams>();
    let fd = (*gb).fd;
    let fp = (*fd).fp;
    let mut gl = std::ptr::null_mut::<sp_lines>();

    if !(*fd).lines.is_null() {
        gl = (*fd).lines;
        (*fd).lines = (*gl).next;
    }

    if gl.is_null() {
        gl = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_lines>() as u64)
            .cast::<sp_lines>();
        if gl.is_null() {
            sam_c_3069_sam_state_err(fd, crate::htslib_mini_rs::c_compat::ENOMEM as c_int);
            return std::ptr::null_mut();
        }
        (*gl).alloc = 0;
        (*gl).data_size = 0;
        (*gl).data = std::ptr::null_mut();
    }
    (*gl).serial = (*gb).serial;
    (*gl).next = std::ptr::null_mut();

    let mut ks = kstring_t {
        l: 0,
        m: (*gl).alloc as usize,
        s: (*gl).data,
    };
    for i in 0..(*gb).nbams {
        if sam_c_4324_sam_format1_append((*fd).h, (*gb).bams.add(i as usize), &mut ks) < 0 {
            let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
            sam_c_3069_sam_state_err(
                fd,
                if errno != 0 {
                    errno
                } else {
                    libc::EIO as c_int
                },
            );
            crate::htslib_mini_rs::c_compat::free(ks.s.cast());
            crate::htslib_mini_rs::c_compat::free(gl.cast());
            return std::ptr::null_mut();
        }
        kputc(b'\n' as c_int, &mut ks);
    }

    (*gl).data_size = ks.l as c_int;
    (*gl).alloc = ks.m as c_int;
    (*gl).data = ks.s;

    if !fp.is_null() && !(*fp).idx.is_null() {
        (*gl).bams = gb;
    } else {
        (*gb).next = (*fd).bams;
        (*fd).bams = gb;
        (*gl).bams = std::ptr::null_mut();
    }

    gl.cast()
}

pub unsafe fn sam_state_destroy(fp: *mut htsFile) -> c_int {
    if fp.is_null() || (*fp).state.is_null() {
        return 0;
    }
    let fd = (*fp).state.cast::<SAM_state>();
    let ret = -(*fd).errcode;

    let mut l = (*fd).lines;
    while !l.is_null() {
        let n = (*l).next;
        crate::htslib_mini_rs::c_compat::free((*l).data.cast());
        crate::htslib_mini_rs::c_compat::free(l.cast());
        l = n;
    }

    let mut b = (*fd).bams;
    while !b.is_null() {
        if (*fd).curr_bam == b {
            (*fd).curr_bam = std::ptr::null_mut();
        }
        let n = (*b).next;
        sam_c_3076_sam_free_sp_bams(b);
        b = n;
    }
    if !(*fd).curr_bam.is_null() {
        sam_c_3076_sam_free_sp_bams((*fd).curr_bam);
    }
    sam_hdr_destroy((*fd).h);
    crate::htslib_mini_rs::c_compat::free((*fp).state);
    (*fp).state = std::ptr::null_mut();
    ret
}

pub unsafe fn sam_hdr_name2tid(_h: *mut sam_hdr_t, _ref_: *const c_char) -> c_int {
    if _h.is_null() || _ref_.is_null() {
        return -1;
    }
    if (*_h).target_name.is_null()
        && (*_h).n_targets == 0
        && (*_h).hrecs.is_null()
        && sam_hdr_fill_targets_from_text(_h) < 0
    {
        return -2;
    }
    if !(*_h).target_name.is_null() {
        let needle = CStr::from_ptr(_ref_);
        for tid in 0..(*_h).n_targets {
            let name = *(*_h).target_name.add(tid as usize);
            if !name.is_null() && CStr::from_ptr(name) == needle {
                return tid;
            }
        }
    }
    let hrecs = (*_h).hrecs;
    if hrecs.is_null() {
        return -1;
    }
    let ref_hash = (*hrecs).ref_hash.cast::<khash_m_s2i_t>();
    if ref_hash.is_null() {
        -1
    } else {
        let k = kh_get_m_s2i(ref_hash, _ref_);
        if k == (*ref_hash).n_buckets {
            -1
        } else {
            *(*ref_hash).vals.add(k as usize)
        }
    }
}

pub unsafe fn bam_name2id(_h: *mut sam_hdr_t, _ref_: *const c_char) -> c_int {
    sam_hdr_name2tid(_h, _ref_)
}

unsafe extern "C" fn sam_c_418_bam_name2id_wrapper(
    vhdr: *mut c_void,
    ref_: *const c_char,
) -> c_int {
    bam_name2id(vhdr.cast(), ref_)
}

pub unsafe fn sam_parse_region(
    h: *mut sam_hdr_t,
    s: *const c_char,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    flags: c_int,
) -> *const c_char {
    hts_parse_region(
        s,
        tid,
        beg,
        end,
        Some(sam_c_418_bam_name2id_wrapper),
        h.cast(),
        flags,
    )
}

unsafe extern "C" fn sam_c_1210_bam_sym_lookup(
    data: *mut c_void,
    str_: *mut c_char,
    end: *mut *mut c_char,
    res: *mut hts_expr_val_t,
) -> c_int {
    let hb = data.cast::<hb_pair>();
    let b = (*hb).b;
    (*res).is_str = 0;

    match *str_ as u8 {
        b'c' if libc::memcmp(str_.cast(), c"cigar".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            let s = ks_clear(&mut (*res).s);
            let cigar = bam_get_cigar(b);
            let n = (*b).core.n_cigar as c_int;
            let mut r = 0;
            if n != 0 {
                for i in 0..n {
                    let c = *cigar.add(i as usize);
                    r |= (kputw(bam_cigar_oplen(c) as c_int, s) < 0) as c_int;
                    r |= (kputc_(b"MIDNSHP=XB??????"[bam_cigar_op(c) as usize] as c_int, s) < 0)
                        as c_int;
                }
                r |= (kputs(c"".as_ptr(), s) < 0) as c_int;
            } else {
                r |= (kputs(c"*".as_ptr(), s) < 0) as c_int;
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
            if *s != b'.' as c_char {
                (*res).d = (*b).core.flag as f64;
                return 0;
            }
            s = s.add(1);
            let flags: &[(&[u8], c_int)] = &[
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
                    (*res).d = ((*b).core.flag as c_int & *flag) as f64;
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
                hclen = bam_cigar_oplen(*cigar) as c_int;
            }
            if ncigar > 1 && bam_cigar_op(*cigar.add(ncigar as usize - 1)) == BAM_CHARD_CLIP {
                hclen += bam_cigar_oplen(*cigar.add(ncigar as usize - 1)) as c_int;
            }
            *end = str_.add(5);
            (*res).d = hclen as f64;
            0
        }
        b'l' if libc::memcmp(str_.cast(), c"library".as_ptr().cast(), 7) == 0 => {
            *end = str_.add(7);
            (*res).is_str = 1;
            let lib = sam_c_1173_bam_get_library((*hb).h, b);
            kputs(
                if lib.is_null() { c"".as_ptr() } else { lib },
                ks_clear(&mut (*res).s),
            );
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
            let rn = sam_hdr_tid2name((*hb).h, (*b).core.mtid);
            kputs(
                if rn.is_null() { c"*".as_ptr() } else { rn },
                ks_clear(&mut (*res).s),
            );
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
            (*res).d = bam_cigar2qlen((*b).core.n_cigar as c_int, bam_get_cigar(b)) as f64;
            0
        }
        b'q' if libc::memcmp(str_.cast(), c"qname".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            kputs(bam_get_qname(b), ks_clear(&mut (*res).s));
            0
        }
        b'q' if libc::memcmp(str_.cast(), c"qual".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            let s = ks_clear(&mut (*res).s);
            if ks_resize(s, (*b).core.l_qseq as usize + 1) < 0 {
                return -1;
            }
            crate::htslib_mini_rs::c_compat::memcpy(
                (*s).s.cast(),
                bam_get_qual(b).cast(),
                (*b).core.l_qseq as u64,
            );
            (*s).l = (*b).core.l_qseq as usize;
            (*res).is_str = 1;
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"rlen".as_ptr().cast(), 4) == 0 => {
            *end = str_.add(4);
            (*res).d = bam_cigar2rlen((*b).core.n_cigar as c_int, bam_get_cigar(b)) as f64;
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"rname".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            let rn = sam_hdr_tid2name((*hb).h, (*b).core.tid);
            kputs(
                if rn.is_null() { c"*".as_ptr() } else { rn },
                ks_clear(&mut (*res).s),
            );
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"rnext".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).is_str = 1;
            let rn = sam_hdr_tid2name((*hb).h, (*b).core.mtid);
            kputs(
                if rn.is_null() { c"*".as_ptr() } else { rn },
                ks_clear(&mut (*res).s),
            );
            0
        }
        b'r' if libc::memcmp(str_.cast(), c"refid".as_ptr().cast(), 5) == 0 => {
            *end = str_.add(5);
            (*res).d = (*b).core.tid as f64;
            0
        }
        b's' if libc::memcmp(str_.cast(), c"seq".as_ptr().cast(), 3) == 0 => {
            *end = str_.add(3);
            let s = ks_clear(&mut (*res).s);
            if ks_resize(s, (*b).core.l_qseq as usize + 1) < 0 {
                return -1;
            }
            nibble2base(bam_get_seq(b).cast_mut(), (*s).s, (*b).core.l_qseq);
            *(*s).s.add((*b).core.l_qseq as usize) = 0;
            (*s).l = (*b).core.l_qseq as usize;
            (*res).is_str = 1;
            0
        }
        b's' if libc::memcmp(str_.cast(), c"sclen".as_ptr().cast(), 5) == 0 => {
            let mut sclen = 0;
            let cigar = bam_get_cigar(b);
            let ncigar = (*b).core.n_cigar as c_int;
            let mut left = 0;
            if ncigar > 0 && bam_cigar_op(*cigar) == BAM_CSOFT_CLIP {
                sclen += bam_cigar_oplen(*cigar) as c_int;
            } else if ncigar > 1
                && bam_cigar_op(*cigar) == BAM_CHARD_CLIP
                && bam_cigar_op(*cigar.add(1)) == BAM_CSOFT_CLIP
            {
                left = 1;
                sclen += bam_cigar_oplen(*cigar.add(1)) as c_int;
            }
            if ncigar - 1 > left && bam_cigar_op(*cigar.add(ncigar as usize - 1)) == BAM_CSOFT_CLIP
            {
                sclen += bam_cigar_oplen(*cigar.add(ncigar as usize - 1)) as c_int;
            } else if ncigar - 2 > left
                && bam_cigar_op(*cigar.add(ncigar as usize - 1)) == BAM_CHARD_CLIP
                && bam_cigar_op(*cigar.add(ncigar as usize - 2)) == BAM_CSOFT_CLIP
            {
                sclen += bam_cigar_oplen(*cigar.add(ncigar as usize - 2)) as c_int;
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
        b'[' if *str_.add(1) != 0 && *str_.add(2) != 0 && *str_.add(3) == b']' as c_char => {
            *end = str_.add(4);
            let aux = bam_aux_get(b, str_.add(1));
            if aux.is_null() {
                (*res).is_str = 1;
                (*res).s.l = 0;
                (*res).d = 0.0;
                (*res).is_true = 0;
                return 0;
            }
            (*res).is_true = 1;
            match *aux as u8 {
                b'Z' | b'H' => {
                    (*res).is_str = 1;
                    kputs(aux.add(1).cast(), ks_clear(&mut (*res).s));
                }
                b'A' => {
                    (*res).is_str = 1;
                    kputsn(aux.add(1).cast(), 1, ks_clear(&mut (*res).s));
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
    filt: *mut c_void,
) -> c_int {
    let mut hb = hb_pair { h, b };
    let mut res = hts_expr_val_t {
        is_str: 0,
        is_true: 0,
        s: kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        },
        d: 0.0,
    };
    if hts_filter_eval2(
        filt.cast::<hts_filter_t>(),
        (&mut hb as *mut hb_pair).cast(),
        Some(sam_c_1210_bam_sym_lookup),
        &mut res,
    ) != 0
    {
        crate::htslib_mini_rs::hts::hts_expr_val_free(&mut res);
        return -1;
    }

    let t = res.is_true as c_int;
    crate::htslib_mini_rs::hts::hts_expr_val_free(&mut res);
    t
}

unsafe fn sam_c_3786_fastq_state_init(name_char: c_int) -> *mut fastq_state {
    let x = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<fastq_state>() as u64)
        .cast::<fastq_state>();
    if x.is_null() {
        return std::ptr::null_mut();
    }
    (*x).BC[0] = b'B' as c_char;
    (*x).BC[1] = b'C' as c_char;
    (*x).BC[2] = 0;
    (*x).nprefix = name_char as c_char;

    if libc::regcomp(
        &mut (*x).regex,
        c"^[^:]+:[^:]+:[^:]+:[^:]+:[^:]+:[^:]+:[^:]+:([^:#/]+)".as_ptr(),
        libc::REG_EXTENDED,
    ) != 0
    {
        crate::htslib_mini_rs::c_compat::free(x.cast());
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
            crate::htslib_mini_rs::c_compat::free((*tags).flags.cast());
            crate::htslib_mini_rs::c_compat::free((*tags).keys.cast());
            crate::htslib_mini_rs::c_compat::free(tags.cast());
        }
        libc::regfree(&mut (*x).regex);
        crate::htslib_mini_rs::c_compat::free((*fp).state);
        (*fp).state = std::ptr::null_mut();
    }
}

pub unsafe fn sam_c_3815_fastq_state_set(
    fp: *mut htsFile,
    opt: c_int,
    arg: *const c_char,
) -> c_int {
    if fp.is_null() {
        return -1;
    }
    if (*fp).state.is_null() {
        (*fp).state =
            sam_c_3786_fastq_state_init(if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
                b'@'
            } else {
                b'>'
            } as c_int)
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
            if !arg.is_null() && libc::strcmp(arg, c"1".as_ptr()) != 0 {
                if (*x).tags.is_null() {
                    let tags = crate::htslib_mini_rs::c_compat::calloc(
                        1,
                        std::mem::size_of::<khash_tag_t>() as u64,
                    )
                    .cast::<khash_tag_t>();
                    if tags.is_null() {
                        return -1;
                    }
                    let tlen = CStr::from_ptr(arg).to_bytes().len();
                    let mut n_buckets = 4u32;
                    while (n_buckets as usize) < ((tlen / 3) + 1) * 2 {
                        n_buckets <<= 1;
                    }
                    let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
                    (*tags).flags = crate::htslib_mini_rs::c_compat::malloc(
                        n_flags as u64 * std::mem::size_of::<u32>() as u64,
                    )
                    .cast::<u32>();
                    (*tags).keys = crate::htslib_mini_rs::c_compat::malloc(
                        n_buckets as u64 * std::mem::size_of::<c_int>() as u64,
                    )
                    .cast::<c_int>();
                    if (*tags).flags.is_null() || (*tags).keys.is_null() {
                        crate::htslib_mini_rs::c_compat::free((*tags).flags.cast());
                        crate::htslib_mini_rs::c_compat::free((*tags).keys.cast());
                        crate::htslib_mini_rs::c_compat::free(tags.cast());
                        return -1;
                    }
                    for i in 0..n_flags {
                        *(*tags).flags.add(i as usize) = 0xaaaa_aaaa;
                    }
                    (*tags).n_buckets = n_buckets;
                    (*tags).upper_bound = (n_buckets as f64 * 0.77) as u32;
                    (*x).tags = tags.cast();
                }

                let tags = (*x).tags.cast::<khash_tag_t>();
                let tag = CStr::from_ptr(arg).to_bytes();
                let tlen = tag.len();
                let mut i = 0usize;
                while i + 3 <= tlen + 1 {
                    let c0 = *arg.add(i);
                    let c1 = *arg.add(i + 1);
                    let c2 = if i + 2 < tlen { *arg.add(i + 2) } else { 0 };
                    if c0 == b',' as c_char
                        || c1 == b',' as c_char
                        || !(c2 == b',' as c_char || c2 == 0)
                    {
                        break;
                    }
                    let tcode = c0 as c_int * 256 + c1 as c_int;
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
                libc::strncpy((*x).BC.as_mut_ptr(), arg, 2);
                (*x).BC[2] = 0;
            }
        }
        FASTQ_OPT_UMI => {
            let bc = if arg.is_null() || libc::strcmp(arg, c"1".as_ptr()) == 0 {
                c"RX".as_ptr()
            } else {
                arg
            };
            let mut p = bc;
            let mut ntags = 0usize;
            let mut err = 0;
            while *p != 0 && ntags < UMI_TAGS {
                if isalpha_c(*p) == 0 || isalnum_c(*p.add(1)) == 0 {
                    err = 1;
                    break;
                }
                (*x).UMI[ntags][0] = *p;
                (*x).UMI[ntags][1] = *p.add(1);
                p = p.add(2);
                if *p != 0 && *p != b',' as c_char {
                    err = 1;
                    break;
                }
                if *p == b',' as c_char {
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
                if libc::regcomp(&mut (*x).regex, arg, libc::REG_EXTENDED) != 0 {
                    return -1;
                }
            }
        }
        _ => {}
    }
    0
}

unsafe fn sam_c_3927_fastq_parse1(fp: *mut htsFile, b: *mut bam1_t) -> c_int {
    let x = (*fp).state.cast::<fastq_state>();
    let mut ret;

    if (*fp).format.format == HTS_FORMAT_FASTA_FORMAT && !(*fp).line.s.is_null() {
        if (*fp).line.l == 0 {
            return -1;
        }
        crate::htslib_mini_rs::c_compat::free((*x).name.s.cast());
        (*x).name = (*fp).line;
        (*fp).line.l = 0;
        (*fp).line.m = 0;
        (*fp).line.s = std::ptr::null_mut();
    } else {
        ret = hts_sys::hts_getline(
            fp.cast(),
            2,
            (&mut (*x).name as *mut crate::htslib_mini_rs::hts::kstring_t).cast(),
        );
        if ret == -1 {
            return -1;
        }
        if ret < -1 {
            return ret;
        }
    }

    if (*x).name.s.is_null() || *(*x).name.s != (*x).nprefix {
        return -2;
    }

    let mut i = 0usize;
    let mut name = (*x).name.s.add(1);
    if (*x).sra_names != 0 {
        let cp0 = libc::strpbrk((*x).name.s, c" \t".as_ptr());
        if !cp0.is_null() {
            let mut cp = cp0;
            while *cp == b' ' as c_char || *cp == b'\t' as c_char {
                cp = cp.add(1);
            }
            cp = cp.sub(1);
            *cp = b'@' as c_char;
            i = cp.offset_from((*x).name.s) as usize;
            name = cp.add(1);
        }
    }

    let l = (*x).name.l;
    let s = (*x).name.s;
    while i < l && isspace_c(*s.add(i)) == 0 {
        i += 1;
    }
    if i < l {
        *s.add(i) = 0;
        (*x).name.l = i;
        i += 1;
    }
    while i < l && isspace_c(*s.add(i)) != 0 {
        i += 1;
    }
    (*x).comment.s = s.add(i);
    (*x).comment.l = l - i;

    (*x).seq.l = 0;
    loop {
        ret = hts_sys::hts_getline(
            fp.cast(),
            2,
            (&mut (*fp).line as *mut crate::htslib_mini_rs::hts::kstring_t).cast(),
        );
        if ret < 0 && ((*fp).format.format == HTS_FORMAT_FASTQ_FORMAT || ret < -1) {
            return -2;
        }
        if ret == -1
            || *(*fp).line.s
                == if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
                    b'+' as c_char
                } else {
                    b'>' as c_char
                }
        {
            break;
        }
        if kputsn((*fp).line.s, (*fp).line.l, &mut (*x).seq) < 0 {
            return -2;
        }
    }

    if (*fp).format.format == HTS_FORMAT_FASTQ_FORMAT {
        let mut remainder = (*x).seq.l;
        (*x).qual.l = 0;
        while remainder > 0 {
            if hts_sys::hts_getline(
                fp.cast(),
                2,
                (&mut (*fp).line as *mut crate::htslib_mini_rs::hts::kstring_t).cast(),
            ) < 0
            {
                return -2;
            }
            if (*fp).line.l > remainder {
                return -2;
            }
            if kputsn((*fp).line.s, (*fp).line.l, &mut (*x).qual) < 0 {
                return -2;
            }
            remainder -= (*fp).line.l;
        }

        for j in 0..(*x).qual.l {
            *(*x).qual.s.add(j) = (*(*x).qual.s.add(j)).wrapping_sub(b'!' as c_char);
        }
    }

    let mut flag = BAM_FUNMAP;
    let pflag = BAM_FMUNMAP | BAM_FPAIRED;
    if (*x).name.l > 2
        && *(*x).name.s.add((*x).name.l - 2) == b'/' as c_char
        && isdigit_c(*(*x).name.s.add((*x).name.l - 1)) != 0
    {
        match *(*x).name.s.add((*x).name.l - 1) as u8 {
            b'1' => flag |= BAM_FREAD1 | pflag,
            b'2' => flag |= BAM_FREAD2 | pflag,
            _ => flag |= BAM_FREAD1 | BAM_FREAD2 | pflag,
        }
        (*x).name.l -= 2;
        *(*x).name.s.add((*x).name.l) = 0;
    }

    let mut umi_seq = [0 as c_char; 256];
    let mut umi_len = 0usize;
    if (*x).UMI[0][0] != 0 {
        let mut mat: [libc::regmatch_t; 3] = std::mem::zeroed();
        if libc::regexec(&mut (*x).regex, (*x).name.s, 2, mat.as_mut_ptr(), 0) == 0
            && mat[0].rm_so >= 0
            && mat[1].rm_so >= 0
        {
            umi_len = (mat[1].rm_eo - mat[1].rm_so) as usize;
            if umi_len > 255 {
                return -2;
            }
            for j in 0..umi_len {
                let c = *(*x).name.s.add(j + mat[1].rm_so as usize);
                umi_seq[j] = if isalpha_c(c) != 0 { c } else { b'-' as c_char };
            }
            if umi_len != 0 {
                umi_seq[umi_len] = 0;
                umi_len += 1;

                (*x).name.l = mat[1].rm_so as usize;
                if (*x).name.l > 0 && *(*x).name.s.add((*x).name.l - 1) == b':' as c_char {
                    (*x).name.l -= 1;
                }
                let mut cp = (*x).name.s.add(mat[1].rm_eo as usize);
                while *cp != 0 {
                    *(*x).name.s.add((*x).name.l) = *cp;
                    (*x).name.l += 1;
                    cp = cp.add(1);
                }
                *(*x).name.s.add((*x).name.l) = 0;
            }
        }
    }

    let l_qname = (*x).name.s.add((*x).name.l).offset_from(name) as usize;
    ret = bam_set1(
        b,
        l_qname,
        name,
        flag as u16,
        -1,
        -1,
        0,
        0,
        std::ptr::null(),
        -1,
        -1,
        0,
        (*x).seq.l,
        (*x).seq.s,
        (*x).qual.s,
        0,
    );
    if ret < 0 {
        return -2;
    }

    if umi_len != 0
        && bam_aux_append(
            b,
            (*x).UMI[0].as_ptr(),
            b'Z' as c_char,
            umi_len as c_int,
            umi_seq.as_ptr().cast(),
        ) < 0
    {
        ret = -2;
    }

    let mut barcode = std::ptr::null_mut::<c_char>();
    let mut barcode_len = 0i32;
    let kc = &mut (*x).comment as *mut kstring_t;
    if (*x).casava != 0 && (*kc).l > 6 {
        let mut endptr: *mut c_char = std::ptr::null_mut();
        if (*(*kc).s.add(1) as u8 | *(*kc).s.add(3) as u8) == b':'
            && isdigit_c(*(*kc).s) != 0
            && libc::strtol((*kc).s.add(4), &mut endptr, 10) >= 0
            && endptr != (*kc).s.add(4)
            && *endptr == b':' as c_char
        {
            match *(*kc).s as u8 {
                b'1' => (*b).core.flag |= (BAM_FREAD1 | pflag) as u16,
                b'2' => (*b).core.flag |= (BAM_FREAD2 | pflag) as u16,
                _ => (*b).core.flag |= (BAM_FREAD1 | BAM_FREAD2 | pflag) as u16,
            }
            if *(*kc).s.add(2) == b'Y' as c_char {
                (*b).core.flag |= BAM_FQCFAIL as u16;
            }
            if isdigit_c(*endptr.add(1)) == 0 {
                barcode = endptr.add(1);
                let mut j = barcode.offset_from((*kc).s) as usize;
                while j < (*kc).l {
                    if isspace_c(*(*kc).s.add(j)) != 0 {
                        break;
                    }
                    j += 1;
                }
                *(*kc).s.add(j) = 0;
                barcode_len = (j + 1 - barcode.offset_from((*kc).s) as usize) as i32;
            }
        }
    }

    if ret >= 0
        && barcode_len != 0
        && bam_aux_append(
            b,
            (*x).BC.as_ptr(),
            b'Z' as c_char,
            barcode_len,
            barcode.cast(),
        ) < 0
    {
        ret = -2;
    }

    if (*x).aux == 0 {
        return ret;
    }

    if sam_c_2524_aux_parse(
        (*kc).s.add(barcode_len as usize),
        (*kc).s.add((*kc).l),
        b,
        1,
        (*x).tags,
    ) < 0
    {
        ret = -2;
    }

    ret
}

unsafe fn sam_c_4413_fastq_format1(
    x: *mut fastq_state,
    b: *const bam1_t,
    str_: *mut kstring_t,
) -> c_int {
    let flag = (*b).core.flag as c_int;
    let len = (*b).core.l_qseq as usize;
    let mut e = 0;
    (*str_).l = 0;

    if kputc((*x).nprefix as c_int, str_) < 0 || kputs(bam_get_qname(b), str_) < 0 {
        return -1;
    }

    if (*x).UMI[0][0] != 0 {
        let mut plex = [0 as c_char; 256];
        let mut name_len = (*str_).l;
        while name_len != 0
            && *(*str_).s.add(name_len) != b':' as c_char
            && *(*str_).s.add(name_len) != b'#' as c_char
        {
            name_len -= 1;
        }

        if *(*str_).s.add(name_len) == b'#' as c_char && (*str_).l - name_len < 255 {
            crate::htslib_mini_rs::c_compat::memcpy(
                plex.as_mut_ptr().cast(),
                (*str_).s.add(name_len).cast(),
                ((*str_).l - name_len) as u64,
            );
            plex[(*str_).l - name_len] = 0;
            (*str_).l = name_len;
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
            if kputc(b':' as c_int, str_) < 0 {
                return -1;
            }
            bc = bc.add(1);
            while *bc != 0 {
                let c = *bc as c_char;
                if kputc(
                    if isalpha_c(c) != 0 {
                        toupper_c(c) as c_int
                    } else {
                        b'+' as c_int
                    },
                    str_,
                ) < 0
                {
                    return -1;
                }
                bc = bc.add(1);
            }
        }

        if plex[0] != 0 && kputs(plex.as_ptr(), str_) < 0 {
            return -1;
        }
    }

    if (*x).rnum != 0 && (flag & BAM_FPAIRED) != 0 {
        let r12 = flag & (BAM_FREAD1 | BAM_FREAD2);
        if r12 == BAM_FREAD1 {
            if kputs(c"/1".as_ptr(), str_) < 0 {
                return -1;
            }
        } else if r12 == BAM_FREAD2 && kputs(c"/2".as_ptr(), str_) < 0 {
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
            b'Y' as c_int
        } else {
            b'N' as c_int
        };
        let bc = bam_aux_get(b, (*x).BC.as_ptr());
        e |= (kputc(b' ' as c_int, str_) < 0) as c_int;
        e |= (kputw(rnum, str_) < 0) as c_int;
        e |= (kputc(b':' as c_int, str_) < 0) as c_int;
        e |= (kputc(filtered, str_) < 0) as c_int;
        e |= (kputsn_(b":0:".as_ptr().cast(), 3, str_) < 0) as c_int;
        if bc.is_null() {
            e |= (kputc(b'0' as c_int, str_) < 0) as c_int;
        } else {
            e |= (kputs(bc.add(1).cast(), str_) < 0) as c_int;
        }
        if e != 0 {
            return -1;
        }

        if !bc.is_null()
            && (*bc != b'Z'
                || (isupper_c(*bc.add(1) as c_char) == 0 && islower_c(*bc.add(1) as c_char) == 0))
        {
            let bc_len = CStr::from_ptr(bc.cast()).to_bytes().len();
            if bc_len >= 2 && (*str_).l >= bc_len - 2 {
                (*str_).l -= bc_len - 2;
                *(*str_).s.add((*str_).l - 1) = b'0' as c_char;
                *(*str_).s.add((*str_).l) = 0;
            }
        } else if !bc.is_null() {
            let bc_len = CStr::from_ptr(bc.add(1).cast()).to_bytes().len();
            let c = (*str_).s.add((*str_).l - bc_len);
            for i in 0..bc_len {
                let ch = *c.add(i);
                if isalpha_c(ch) == 0 {
                    *c.add(i) = b'+' as c_char;
                } else if islower_c(ch) != 0 {
                    *c.add(i) = toupper_c(ch);
                }
            }
        }
    }

    if (*x).aux != 0 {
        let mut s = bam_get_aux(b).cast_mut();
        let end = (*b).data.add((*b).l_data as usize);
        while !s.is_null() && end.offset_from(s) >= 4 {
            let tt = *s as c_int * 256 + *s.add(1) as c_int;
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
                e |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
                s = sam_format_aux1(s, *s.add(2), s.add(3), end, str_).cast_mut();
                if s.is_null() {
                    return -1;
                }
            } else {
                s = skip_aux(s.add(2), end);
            }
        }
        e |= (kputsn(c"".as_ptr(), 0, str_) < 0) as c_int;
    }

    if ks_resize(str_, (*str_).l + 1 + len + 1 + 2 + len + 1 + 1) < 0 {
        return -1;
    }
    e |= (kputc_(b'\n' as c_int, str_) < 0) as c_int;

    let seq = bam_get_seq(b);
    if (flag & BAM_FREVERSE) != 0 {
        for i in (0..len).rev() {
            e |= (kputc_(
                b"!TGKCYSBAWRDMHVN"[bam_seqi(seq, i) as usize] as c_int,
                str_,
            ) < 0) as c_int;
        }
    } else {
        for i in 0..len {
            e |= (kputc_(SEQ_NT16_STR[bam_seqi(seq, i) as usize] as c_int, str_) < 0) as c_int;
        }
    }

    if (*x).nprefix == b'@' as c_char {
        kputsn(c"\n+\n".as_ptr(), 3, str_);
        let qual = bam_get_qual(b);
        if *qual == 0xff {
            for _ in 0..len {
                e |= (kputc_(b'B' as c_int, str_) < 0) as c_int;
            }
        } else if (flag & BAM_FREVERSE) != 0 {
            for i in (0..len).rev() {
                e |= (kputc_(33 + *qual.add(i) as c_int, str_) < 0) as c_int;
            }
        } else {
            for i in 0..len {
                e |= (kputc_(33 + *qual.add(i) as c_int, str_) < 0) as c_int;
            }
        }
    }
    e |= (kputc(b'\n' as c_int, str_) < 0) as c_int;

    if e != 0 {
        -1
    } else {
        (*str_).l as c_int
    }
}

pub unsafe fn sam_hdr_tid2len(_h: *const sam_hdr_t, _tid: c_int) -> hts_pos_t {
    if _h.is_null() || _tid < 0 {
        return 0;
    }
    let hrecs = (*_h).hrecs;
    if !hrecs.is_null() && _tid < (*hrecs).nref {
        return (*(*hrecs).ref_.add(_tid as usize)).len;
    }
    if _tid < (*_h).n_targets {
        let len = *(*_h).target_len.add(_tid as usize);
        if len < u32::MAX || (*_h).sdict.is_null() {
            return len as hts_pos_t;
        }
        let long_refs = (*_h).sdict.cast::<khash_s2i_t>();
        let k = kh_get_s2i(long_refs, *(*_h).target_name.add(_tid as usize));
        return if k == (*long_refs).n_buckets {
            u32::MAX as hts_pos_t
        } else {
            *(*long_refs).vals.add(k as usize) as hts_pos_t
        };
    }
    0
}

pub unsafe fn sam_hdr_tid2name(_h: *const sam_hdr_t, _tid: c_int) -> *const c_char {
    if _h.is_null() || _tid < 0 {
        return std::ptr::null();
    }
    let hrecs = (*_h).hrecs;
    if !hrecs.is_null() && _tid < (*hrecs).nref {
        return (*(*hrecs).ref_.add(_tid as usize)).name;
    }
    if _tid < (*_h).n_targets {
        return *(*_h).target_name.add(_tid as usize);
    }
    std::ptr::null()
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

pub unsafe fn sam_realloc_bam_data(b: *mut bam1_t, desired: usize) -> c_int {
    if desired > (i32::MAX as f64 * 0.666) as usize {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    }
    let mut new_m_data = kroundup32(desired as u32);
    new_m_data = new_m_data.wrapping_add(32);
    if (new_m_data as usize) < desired {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    }

    let new_data = if (bam_get_mempolicy(b) & BAM_USER_OWNS_DATA) == 0 {
        crate::htslib_mini_rs::c_compat::realloc((*b).data.cast(), new_m_data as u64).cast::<u8>()
    } else {
        let ptr = crate::htslib_mini_rs::c_compat::malloc(new_m_data as u64).cast::<u8>();
        if !ptr.is_null() {
            if (*b).l_data > 0 {
                let copied = ((*b).l_data as u32).min((*b).m_data) as u64;
                crate::htslib_mini_rs::c_compat::memcpy(ptr.cast(), (*b).data.cast(), copied);
            }
            bam_set_mempolicy(b, bam_get_mempolicy(b) & !BAM_USER_OWNS_DATA);
        }
        ptr
    };
    if new_data.is_null() {
        return -1;
    }
    (*b).data = new_data;
    (*b).m_data = new_m_data;
    0
}

pub unsafe fn realloc_bam_data(b: *mut bam1_t, desired: usize) -> c_int {
    if desired <= (*b).m_data as usize {
        return 0;
    }
    sam_realloc_bam_data(b, desired)
}

pub unsafe fn bam_init1() -> *mut bam1_t {
    crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<bam1_t>() as u64).cast()
}

pub unsafe fn bam_destroy1(b: *mut bam1_t) {
    if b.is_null() {
        return;
    }
    if (bam_get_mempolicy(b) & BAM_USER_OWNS_DATA) == 0 {
        crate::htslib_mini_rs::c_compat::free((*b).data.cast());
        if (bam_get_mempolicy(b) & BAM_USER_OWNS_STRUCT) != 0 {
            (*b).data = std::ptr::null_mut();
            (*b).m_data = 0;
            (*b).l_data = 0;
        }
    }
    if (bam_get_mempolicy(b) & BAM_USER_OWNS_STRUCT) == 0 {
        crate::htslib_mini_rs::c_compat::free(b.cast());
    }
}

pub unsafe fn bam_copy1(bdst: *mut bam1_t, bsrc: *const bam1_t) -> *mut bam1_t {
    if realloc_bam_data(bdst, (*bsrc).l_data as usize) < 0 {
        return std::ptr::null_mut();
    }
    crate::htslib_mini_rs::c_compat::memcpy(
        (*bdst).data.cast(),
        (*bsrc).data.cast(),
        (*bsrc).l_data as u64,
    );
    (*bdst).core = (*bsrc).core;
    (*bdst).l_data = (*bsrc).l_data;
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

pub unsafe fn mp_init() -> *mut mempool_t {
    crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<mempool_t>() as u64).cast()
}

pub unsafe fn mp_destroy(mp: *mut mempool_t) {
    for k in 0..(*mp).n {
        let node = *(*mp).buf.add(k as usize);
        crate::htslib_mini_rs::c_compat::free((*node).b.data.cast());
        crate::htslib_mini_rs::c_compat::free(node.cast());
    }
    crate::htslib_mini_rs::c_compat::free((*mp).buf.cast());
    crate::htslib_mini_rs::c_compat::free(mp.cast());
}

pub unsafe fn mp_alloc(mp: *mut mempool_t) -> *mut lbnode_t {
    (*mp).cnt += 1;
    if (*mp).n == 0 {
        crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<lbnode_t>() as u64).cast()
    } else {
        (*mp).n -= 1;
        *(*mp).buf.add((*mp).n as usize)
    }
}

pub unsafe fn mp_free(mp: *mut mempool_t, p: *mut lbnode_t) {
    (*mp).cnt -= 1;
    (*p).next = std::ptr::null_mut();
    if (*mp).n == (*mp).max {
        (*mp).max = if (*mp).max != 0 { (*mp).max << 1 } else { 256 };
        (*mp).buf = crate::htslib_mini_rs::c_compat::realloc(
            (*mp).buf.cast(),
            (std::mem::size_of::<*mut lbnode_t>() * (*mp).max as usize) as u64,
        )
        .cast();
    }
    *(*mp).buf.add((*mp).n as usize) = p;
    (*mp).n += 1;
}

pub unsafe fn resolve_cigar2(p: *mut bam_pileup1_t, pos: hts_pos_t, s: *mut cstate_t) -> c_int {
    let b = (*p).b;
    let c = &(*b).core;
    let cigar = bam_get_cigar(b);

    if (*s).k == -1 {
        (*p).qpos = 0;
        if c.n_cigar == 1 {
            let op = bam_cigar_op(*cigar);
            if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                (*s).k = 0;
                (*s).x = c.pos;
                (*s).y = 0;
            }
        } else {
            (*s).x = c.pos;
            (*s).y = 0;
            let mut k = 0;
            while k < c.n_cigar {
                let cg = *cigar.add(k as usize);
                let op = bam_cigar_op(cg);
                let l = bam_cigar_oplen(cg) as c_int;
                if op == BAM_CMATCH
                    || op == BAM_CDEL
                    || op == BAM_CREF_SKIP
                    || op == BAM_CEQUAL
                    || op == BAM_CDIFF
                {
                    break;
                } else if op == BAM_CINS || op == BAM_CSOFT_CLIP {
                    (*s).y += l;
                }
                k += 1;
            }
            assert!(k < c.n_cigar);
            (*s).k = k as c_int;
        }
    } else {
        let mut l = bam_cigar_oplen(*cigar.add((*s).k as usize)) as hts_pos_t;
        if pos - (*s).x >= l {
            assert!((*s).k < c.n_cigar as c_int);
            let op = bam_cigar_op(*cigar.add((*s).k as usize + 1));
            if op == BAM_CMATCH
                || op == BAM_CDEL
                || op == BAM_CREF_SKIP
                || op == BAM_CEQUAL
                || op == BAM_CDIFF
            {
                let cur_op = bam_cigar_op(*cigar.add((*s).k as usize));
                if cur_op == BAM_CMATCH || cur_op == BAM_CEQUAL || cur_op == BAM_CDIFF {
                    (*s).y += l as c_int;
                }
                (*s).x += l;
                (*s).k += 1;
            } else {
                let cur_op = bam_cigar_op(*cigar.add((*s).k as usize));
                if cur_op == BAM_CMATCH || cur_op == BAM_CEQUAL || cur_op == BAM_CDIFF {
                    (*s).y += l as c_int;
                }
                (*s).x += l;
                let mut k = (*s).k + 1;
                while k < c.n_cigar as c_int {
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
                        (*s).y += l as c_int;
                    }
                    k += 1;
                }
                (*s).k = k;
            }
            assert!((*s).k < c.n_cigar as c_int);
        }
    }

    let op = bam_cigar_op(*cigar.add((*s).k as usize));
    let l = bam_cigar_oplen(*cigar.add((*s).k as usize)) as hts_pos_t;
    set_pileup_is_del(p, false);
    (*p).indel = 0;
    set_pileup_is_refskip(p, false);
    if (*s).x + l - 1 == pos && (*s).k + 1 < c.n_cigar as c_int {
        let mut op2 = bam_cigar_op(*cigar.add((*s).k as usize + 1));
        let mut l2 = bam_cigar_oplen(*cigar.add((*s).k as usize + 1)) as c_int;
        if op2 == BAM_CDEL && op != BAM_CDEL {
            (*p).indel = -l2;
            let mut k = (*s).k + 2;
            while k < c.n_cigar as c_int {
                op2 = bam_cigar_op(*cigar.add(k as usize));
                l2 = bam_cigar_oplen(*cigar.add(k as usize)) as c_int;
                if op2 == BAM_CDEL {
                    (*p).indel -= l2;
                } else {
                    break;
                }
                k += 1;
            }
        } else if op2 == BAM_CINS {
            (*p).indel = l2;
            let mut k = (*s).k + 2;
            while k < c.n_cigar as c_int {
                op2 = bam_cigar_op(*cigar.add(k as usize));
                l2 = bam_cigar_oplen(*cigar.add(k as usize)) as c_int;
                if op2 == BAM_CINS {
                    (*p).indel += l2;
                } else if op2 != BAM_CPAD {
                    break;
                }
                k += 1;
            }
        } else if op2 == BAM_CPAD && (*s).k + 2 < c.n_cigar as c_int {
            let mut l3 = 0;
            let mut k = (*s).k + 2;
            while k < c.n_cigar as c_int {
                op2 = bam_cigar_op(*cigar.add(k as usize));
                l2 = bam_cigar_oplen(*cigar.add(k as usize)) as c_int;
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
                (*p).indel = l3;
            }
        }
    }
    if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
        (*p).qpos = (*s).y + (pos - (*s).x) as c_int;
    } else if op == BAM_CDEL || op == BAM_CREF_SKIP {
        set_pileup_is_del(p, true);
        (*p).qpos = (*s).y;
        set_pileup_is_refskip(p, op == BAM_CREF_SKIP);
    }
    set_pileup_is_head(p, pos == c.pos);
    set_pileup_is_tail(p, pos == (*s).end);
    (*p).cigar_ind = (*s).k;
    1
}

unsafe fn cigar_iref2iseq_set(
    cigar: *mut *const u32,
    cigar_max: *const u32,
    icig: *mut hts_pos_t,
    iseq: *mut hts_pos_t,
    iref: *mut hts_pos_t,
) -> c_int {
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
) -> c_int {
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

unsafe fn tweak_overlap_quality(a: *mut bam1_t, b: *mut bam1_t) -> c_int {
    let mut a_cigar = bam_get_cigar(a);
    let a_cigar_max = a_cigar.add((*a).core.n_cigar as usize);
    let mut b_cigar = bam_get_cigar(b);
    let b_cigar_max = b_cigar.add((*b).core.n_cigar as usize);
    let mut a_icig = 0;
    let mut a_iseq = 0;
    let mut b_icig = 0;
    let mut b_iseq = 0;
    let a_qual = bam_get_qual(a) as *mut u8;
    let b_qual = bam_get_qual(b) as *mut u8;
    let a_seq = bam_get_seq(a);
    let b_seq = bam_get_seq(b);
    let mut iref = (*b).core.pos;
    let mut a_iref = iref - (*a).core.pos;
    let mut b_iref = iref - (*b).core.pos;

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

    let (amul, bmul) = if (__ac_Wang_hash(__ac_X31_hash_string(bam_get_qname(a))) & 1) != 0 {
        (1u8, 0u8)
    } else {
        (0u8, 1u8)
    };

    loop {
        while a_ret >= 0 && a_iref >= 0 && a_iref < iref - (*a).core.pos {
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
        while b_ret >= 0 && b_iref >= 0 && b_iref < iref - (*b).core.pos {
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
        if iref < a_iref + (*a).core.pos {
            iref = a_iref + (*a).core.pos;
        }
        if iref < b_iref + (*b).core.pos {
            iref = b_iref + (*b).core.pos;
        }
        iref += 1;

        if a_iref + (*a).core.pos != b_iref + (*b).core.pos {
            if a_iref + (*a).core.pos < b_iref + (*b).core.pos
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
                    if a_iref + (*a).core.pos >= b_iref + (*b).core.pos {
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
                    if b_iref + (*b).core.pos >= a_iref + (*a).core.pos {
                        break;
                    }
                }
            } else {
                continue;
            }
        }

        if a_iseq > (*a).core.l_qseq as hts_pos_t || b_iseq > (*b).core.l_qseq as hts_pos_t {
            return -1;
        }
        let ai = a_iseq as usize;
        let bi = b_iseq as usize;
        if bam_seqi(a_seq, ai) == bam_seqi(b_seq, bi) {
            let qual = *a_qual.add(ai) as c_int + *b_qual.add(bi) as c_int;
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

unsafe fn overlap_push(iter: bam_plp_t, node: *mut lbnode_t) -> c_int {
    if (*iter).overlaps.is_null() {
        return 0;
    }
    if (((*node).b.core.flag as c_int) & BAM_FMUNMAP) != 0
        || (((*node).b.core.flag as c_int) & BAM_FPROPER_PAIR) == 0
    {
        return 0;
    }
    if ((*node).b.core.mtid >= 0 && (*node).b.core.tid != (*node).b.core.mtid)
        || ((*node).b.core.isize.abs() >= 2 * (*node).b.core.l_qseq as hts_pos_t
            && (*node).b.core.mpos >= (*node).end)
    {
        return 0;
    }

    let overlaps = &mut *((*iter).overlaps.cast::<OlapHash>());
    let key = CStr::from_ptr(bam_get_qname(&(*node).b))
        .to_bytes()
        .to_vec();
    if let Some(a) = overlaps.remove(&key) {
        let err = tweak_overlap_quality(&mut (*a).b, &mut (*node).b);
        debug_assert_eq!((*a).end - 1, (*a).s.end);
        err
    } else {
        if (*node).b.core.mpos >= (*node).b.core.pos
            || (((*node).b.core.flag as c_int) & BAM_FPAIRED) != 0 && (*node).b.core.mpos == -1
        {
            overlaps.insert(key, node);
        }
        0
    }
}

unsafe fn overlap_remove(iter: bam_plp_t, b: *const bam1_t) {
    if (*iter).overlaps.is_null() {
        return;
    }
    let overlaps = &mut *((*iter).overlaps.cast::<OlapHash>());
    if b.is_null() {
        overlaps.clear();
        return;
    }
    if ((*b).core.flag as c_int & BAM_FUNMAP) != 0
        || ((*b).core.flag as c_int & BAM_FPROPER_PAIR) == 0
    {
        return;
    }
    let key = CStr::from_ptr(bam_get_qname(b)).to_bytes();
    overlaps.remove(key);
}

unsafe fn kh_get_m_s2i(h: *const khash_m_s2i_t, key: *const c_char) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key) & mask;
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

unsafe fn kh_get_s2i(h: *const khash_s2i_t, key: *const c_char) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key) & mask;
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

unsafe fn kh_destroy_s2i(h: *mut khash_s2i_t) {
    if h.is_null() {
        return;
    }
    crate::htslib_mini_rs::c_compat::free((*h).flags.cast());
    crate::htslib_mini_rs::c_compat::free((*h).keys.cast());
    crate::htslib_mini_rs::c_compat::free((*h).vals.cast());
    crate::htslib_mini_rs::c_compat::free(h.cast());
}

unsafe fn kh_destroy_m_s2i(h: *mut khash_m_s2i_t) {
    if h.is_null() {
        return;
    }
    crate::htslib_mini_rs::c_compat::free((*h).flags.cast());
    crate::htslib_mini_rs::c_compat::free((*h).keys.cast());
    crate::htslib_mini_rs::c_compat::free((*h).vals.cast());
    crate::htslib_mini_rs::c_compat::free(h.cast());
}

unsafe fn kh_get_str2int(h: *const khash_m_s2i_t, key: *const c_char) -> u32 {
    if h.is_null() || (*h).n_buckets == 0 {
        return if h.is_null() { 0 } else { (*h).n_buckets };
    }
    let mask = (*h).n_buckets - 1;
    let mut i = __ac_FNV1a_hash_string(key) & mask;
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

unsafe fn kh_put_str2int(h: *mut khash_m_s2i_t, key: *const c_char, ret: *mut c_int) -> u32 {
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
    let mut i = __ac_FNV1a_hash_string(key) & mask;
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

unsafe fn kh_resize_str2int(h: *mut khash_m_s2i_t, new_n_buckets: u32) -> c_int {
    let n_flags = if new_n_buckets < 16 {
        1
    } else {
        new_n_buckets >> 4
    };
    let flags =
        crate::htslib_mini_rs::c_compat::malloc(n_flags as u64 * std::mem::size_of::<u32>() as u64)
            .cast::<u32>();
    let keys = crate::htslib_mini_rs::c_compat::malloc(
        new_n_buckets as u64 * std::mem::size_of::<*mut c_char>() as u64,
    )
    .cast::<*mut c_char>();
    let vals = crate::htslib_mini_rs::c_compat::malloc(
        new_n_buckets as u64 * std::mem::size_of::<c_int>() as u64,
    )
    .cast::<c_int>();
    if flags.is_null() || keys.is_null() || vals.is_null() {
        crate::htslib_mini_rs::c_compat::free(flags.cast());
        crate::htslib_mini_rs::c_compat::free(keys.cast());
        crate::htslib_mini_rs::c_compat::free(vals.cast());
        return -1;
    }
    for i in 0..n_flags {
        *flags.add(i as usize) = 0xaaaa_aaaa;
    }

    let old_n = (*h).n_buckets;
    let old_flags = (*h).flags;
    let old_keys = (*h).keys;
    let old_vals = (*h).vals;
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
    crate::htslib_mini_rs::c_compat::free(old_flags.cast());
    crate::htslib_mini_rs::c_compat::free(old_keys.cast());
    crate::htslib_mini_rs::c_compat::free(old_vals.cast());
    0
}

pub unsafe fn khash_str2int_init() -> *mut c_void {
    let h = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<khash_m_s2i_t>() as u64)
        .cast::<khash_m_s2i_t>();
    h.cast()
}

pub unsafe fn khash_str2int_destroy(_hash: *mut c_void) {
    kh_destroy_m_s2i(_hash.cast());
}

pub unsafe fn khash_str2int_destroy_free(_hash: *mut c_void) {
    let hash = _hash.cast::<khash_m_s2i_t>();
    if hash.is_null() {
        return;
    }
    for k in 0..(*hash).n_buckets {
        if !kh_iseither((*hash).flags, k) {
            crate::htslib_mini_rs::c_compat::free(
                *(*hash).keys.add(k as usize).cast::<*mut c_void>(),
            );
        }
    }
    kh_destroy_m_s2i(hash);
}

pub unsafe fn khash_str2int_has_key(_hash: *mut c_void, str_: *const c_char) -> c_int {
    let hash = _hash.cast::<khash_m_s2i_t>();
    let k = kh_get_str2int(hash, str_);
    (k != (*hash).n_buckets) as c_int
}

pub unsafe fn khash_str2int_get(
    _hash: *mut c_void,
    str_: *const c_char,
    value: *mut c_int,
) -> c_int {
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

pub unsafe fn khash_str2int_inc(_hash: *mut c_void, str_: *const c_char) -> c_int {
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
    *(*hash).vals.add(k as usize) = (*hash).size as c_int - 1;
    *(*hash).vals.add(k as usize)
}

pub unsafe fn khash_str2int_set(_hash: *mut c_void, str_: *const c_char, value: c_int) -> c_int {
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
    k as c_int
}

pub unsafe fn khash_str2int_size(_hash: *mut c_void) -> c_int {
    (*_hash.cast::<khash_m_s2i_t>()).size as c_int
}

unsafe fn cstr_eq(a: *const c_char, b: *const c_char) -> bool {
    !a.is_null() && !b.is_null() && CStr::from_ptr(a) == CStr::from_ptr(b)
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

unsafe extern "C" fn sam_c_1631_bam_pseek(fp: *mut c_void, offset: i64, whence: c_int) -> c_int {
    bgzf_seek(fp.cast::<BGZF>(), offset, whence) as c_int
}

unsafe extern "C" fn sam_c_1638_bam_ptell(fp: *mut c_void) -> i64 {
    let fd = fp.cast::<BGZF>();
    if fd.is_null() {
        return -1;
    }

    ((*fd).block_address << 16) | ((*fd).block_offset as i64 & 0xffff)
}

unsafe fn sam_c_1649_index_load(
    fp: *mut htsFile,
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut hts_idx_t {
    match (*fp).format.format {
        HTS_FORMAT_BAM | HTS_FORMAT_SAM => hts_idx_load3(fn_, fnidx, HTS_FMT_BAI, flags),
        HTS_FORMAT_CRAM => hts_sys::sam_index_load3(fp.cast(), fn_, fnidx, flags).cast(),
        _ => std::ptr::null_mut(),
    }
}

pub unsafe fn sam_index_load3(
    _fp: *mut htsFile,
    _fn_: *const c_char,
    _fnidx: *const c_char,
    _flags: c_int,
) -> *mut hts_idx_t {
    sam_c_1649_index_load(_fp, _fn_, _fnidx, _flags)
}

pub unsafe fn sam_index_load2(
    _fp: *mut htsFile,
    _fn_: *const c_char,
    _fnidx: *const c_char,
) -> *mut hts_idx_t {
    sam_index_load3(_fp, _fn_, _fnidx, HTS_IDX_SAVE_REMOTE)
}

pub unsafe fn sam_index_load(_fp: *mut htsFile, _fn_: *const c_char) -> *mut hts_idx_t {
    sam_index_load3(_fp, _fn_, std::ptr::null(), HTS_IDX_SAVE_REMOTE)
}

pub unsafe fn sam_itr_queryi(
    _idx: *const hts_idx_t,
    _tid: c_int,
    _beg: hts_pos_t,
    _end: hts_pos_t,
) -> *mut hts_itr_t {
    if _idx.is_null() {
        return hts_itr_query(_idx, _tid, _beg, _end, Some(sam_readrec_rest));
    }
    if (*_idx).fmt == HTS_FMT_CRAI {
        return hts_sys::sam_itr_queryi(_idx.cast(), _tid, _beg, _end).cast();
    }
    hts_itr_query(_idx, _tid, _beg, _end, Some(sam_readrec))
}

pub unsafe fn sam_itr_querys(
    idx: *const hts_idx_t,
    hdr: *mut sam_hdr_t,
    region: *const c_char,
) -> *mut hts_itr_t {
    if idx.is_null() || hdr.is_null() || region.is_null() {
        return std::ptr::null_mut();
    }
    if (*idx).fmt == HTS_FMT_CRAI {
        return hts_sys::sam_itr_querys(idx.cast(), hdr.cast(), region).cast();
    }
    if libc::strcmp(region, c".".as_ptr()) == 0 {
        return sam_itr_queryi(idx, HTS_IDX_START, 0, 0);
    }
    if libc::strcmp(region, c"*".as_ptr()) == 0 {
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

unsafe extern "C" fn sam_c_1754_cram_name2id(fdv: *mut c_void, ref_: *const c_char) -> c_int {
    if fdv.is_null() || ref_.is_null() {
        return -1;
    }
    let hdr = hts_sys::cram_fd_get_header(fdv.cast()).cast::<sam_hdr_t>();
    sam_hdr_name2tid(hdr, ref_)
}

pub unsafe fn sam_c_1768_sam_itr_regarray(
    idx: *const hts_idx_t,
    hdr: *mut sam_hdr_t,
    regarray: *mut *mut c_char,
    regcount: u32,
) -> *mut hts_itr_t {
    if idx.is_null() || hdr.is_null() {
        return std::ptr::null_mut();
    }
    if (*idx).fmt == HTS_FMT_CRAI {
        return hts_sys::sam_itr_regarray(idx.cast(), hdr.cast(), regarray, regcount).cast();
    }

    let mut reg_count = 0;
    let reglist = hts_reglist_create(
        regarray,
        regcount as c_int,
        &mut reg_count,
        hdr.cast(),
        Some(sam_c_418_bam_name2id_wrapper),
    );
    if reglist.is_null() {
        return std::ptr::null_mut();
    }
    let itr = hts_itr_regions(
        idx,
        reglist,
        reg_count,
        Some(sam_c_418_bam_name2id_wrapper),
        hdr.cast(),
        Some(hts_itr_multi_bam),
        Some(sam_readrec),
        Some(sam_c_1631_bam_pseek),
        Some(sam_c_1638_bam_ptell),
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
    hts_sys::sam_itr_regions(idx.cast(), hdr.cast(), reglist.cast(), regcount).cast()
}

unsafe fn sam_c_994_sam_index(fp: *mut htsFile, mut min_shift: c_int) -> *mut hts_idx_t {
    let h = sam_hdr_read(fp);
    if h.is_null() {
        return std::ptr::null_mut();
    }

    let (fmt, n_lvls) = if min_shift > 0 {
        let mut max_len = 0;
        for i in 0..(*h).n_targets {
            let len = sam_hdr_tid2len(h, i);
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

    let idx = hts_sys::hts_idx_init(
        (*h).n_targets,
        fmt,
        sam_c_1638_bam_ptell((*fp).fp.bgzf.cast()) as u64,
        min_shift,
        n_lvls,
    )
    .cast::<hts_idx_t>();
    let b = bam_init1();
    let mut ret = sam_read1(fp, h, b);
    while ret >= 0 {
        ret = hts_sys::hts_idx_push(
            idx.cast(),
            (*b).core.tid,
            (*b).core.pos,
            bam_endpos(b),
            sam_c_1638_bam_ptell((*fp).fp.bgzf.cast()) as u64,
            (((*b).core.flag as c_int & BAM_FUNMAP) == 0) as c_int,
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

    hts_sys::hts_idx_finish(
        idx.cast(),
        sam_c_1638_bam_ptell((*fp).fp.bgzf.cast()) as u64,
    );
    sam_hdr_destroy(h);
    bam_destroy1(b);
    idx
}

pub unsafe fn sam_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    nthreads: c_int,
) -> c_int {
    hts_sys::sam_index_build3(fn_, fnidx, min_shift, nthreads)
}

pub unsafe fn sam_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
) -> c_int {
    sam_index_build3(fn_, fnidx, min_shift, 0)
}

pub unsafe fn sam_index_build(fn_: *const c_char, min_shift: c_int) -> c_int {
    sam_index_build3(fn_, std::ptr::null(), min_shift, 0)
}

pub unsafe fn bam_index_build(fn_: *const c_char, min_shift: c_int) -> c_int {
    sam_index_build2(fn_, std::ptr::null(), min_shift)
}

pub unsafe fn sam_idx_init(
    fp: *mut htsFile,
    h: *mut sam_hdr_t,
    min_shift: c_int,
    fnidx: *const c_char,
) -> c_int {
    hts_sys::sam_idx_init(fp.cast(), h.cast(), min_shift, fnidx)
}

pub unsafe fn sam_idx_save(fp: *mut htsFile) -> c_int {
    hts_sys::sam_idx_save(fp.cast())
}

pub unsafe fn sam_itr_next(_htsfp: *mut htsFile, _itr: *mut hts_itr_t, _r: *mut bam1_t) -> c_int {
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
    hts_itr_next(fp, _itr.cast(), _r.cast(), _htsfp.cast::<c_void>())
}

unsafe fn sam_read1_bam(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> c_int {
    let ret = bam_read1((*fp).fp.bgzf, b);
    if !h.is_null()
        && ret >= 0
        && ((*b).core.tid >= (*h).n_targets
            || (*b).core.tid < -1
            || (*b).core.mtid >= (*h).n_targets
            || (*b).core.mtid < -1)
    {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ERANGE as c_int;
        return -3;
    }
    ret
}

unsafe fn sam_c_4145_sam_read1_cram(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> c_int {
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
        hts_sys::sam_read1(fp.cast(), h.cast(), b.cast())
    };
    if ret < 0 || !sam_cram_tlen_candidate(b) {
        return ret;
    }

    let first_qname = CStr::from_ptr(bam_get_qname(b)).to_bytes().to_vec();
    let mut group = vec![b];
    let mut group_rets = vec![ret];
    let mut next_group: Option<(*mut bam1_t, c_int)> = None;

    loop {
        let next = bam_init1();
        if next.is_null() {
            break;
        }
        let next_ret = hts_sys::sam_read1(fp.cast(), h.cast(), next.cast());
        if next_ret < 0 {
            bam_destroy1(next);
            break;
        }
        if CStr::from_ptr(bam_get_qname(next)).to_bytes() == first_qname.as_slice() {
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

fn sam_cram_pending() -> &'static Mutex<HashMap<usize, VecDeque<(usize, c_int)>>> {
    static PENDING: OnceLock<Mutex<HashMap<usize, VecDeque<(usize, c_int)>>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe fn sam_cram_pending_pop(fp: *mut htsFile) -> Option<(*mut bam1_t, c_int)> {
    let mut pending = sam_cram_pending().lock().unwrap();
    let queue = pending.get_mut(&(fp as usize))?;
    let (rec, ret) = queue.pop_front()?;
    if queue.is_empty() {
        pending.remove(&(fp as usize));
    }
    Some((rec as *mut bam1_t, ret))
}

unsafe fn sam_cram_pending_push(fp: *mut htsFile, rec: *mut bam1_t, ret: c_int) {
    let mut pending = sam_cram_pending().lock().unwrap();
    pending
        .entry(fp as usize)
        .or_default()
        .push_back((rec as usize, ret));
}

fn sam_cram_lookahead() -> &'static Mutex<HashMap<usize, (usize, c_int)>> {
    static LOOKAHEAD: OnceLock<Mutex<HashMap<usize, (usize, c_int)>>> = OnceLock::new();
    LOOKAHEAD.get_or_init(|| Mutex::new(HashMap::new()))
}

unsafe fn sam_cram_lookahead_take(fp: *mut htsFile) -> Option<(*mut bam1_t, c_int)> {
    let mut lookahead = sam_cram_lookahead().lock().unwrap();
    lookahead
        .remove(&(fp as usize))
        .map(|(rec, ret)| (rec as *mut bam1_t, ret))
}

unsafe fn sam_cram_lookahead_store(fp: *mut htsFile, rec: *mut bam1_t, ret: c_int) {
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
    let rlen = bam_cigar2rlen(c.n_cigar as c_int, bam_get_cigar(b));
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
    let first = group[0];
    let first_right = match sam_cram_record_right(first) {
        Some(right) => right,
        None => return,
    };
    if (*first).core.pos == aleft && (first_right < aright || left_cnt <= 1) {
        (*first).core.isize = tlen;
        tlen = -tlen;
    } else if (*first).core.pos == aleft && first_right == aright && left_cnt > 1 && right_cnt > 1 {
        if ((*first).core.flag as c_int & BAM_FREAD1) != 0 {
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
    p: *mut hts_sys::htsThreadPool,
) -> c_int {
    if fp.is_null() || p.is_null() {
        return -1;
    }
    if !(*fp).state.is_null() {
        return -2;
    }
    hts_sys::hts_set_thread_pool(fp.cast(), p)
}

pub unsafe fn sam_c_3746_sam_set_threads(fp: *mut htsFile, nthreads: c_int) -> c_int {
    if nthreads <= 0 {
        return 0;
    }
    if fp.is_null() {
        return -1;
    }
    hts_sys::hts_set_threads(fp.cast(), nthreads)
}

pub unsafe fn bam_read1(fp: *mut BGZF, b: *mut bam1_t) -> c_int {
    let c = &mut (*b).core;
    let mut block_len_buf = [0u8; 4];
    let mut core_buf = [0u8; 32];

    (*b).l_data = 0;

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
            .cast::<u8>()
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
    c.l_extranul = if c.l_qname % 4 != 0 {
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

    let new_l_data = block_len - 32 + c.l_extranul as c_int;
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
    (*b).l_data = new_l_data;

    if bgzf_read_small(fp, (*b).data.cast(), c.l_qname as usize) != c.l_qname as isize {
        return -4;
    }
    if *(*b).data.add(c.l_qname as usize - 1) != 0 && fixup_missing_qname_nul(b) < 0 {
        return -4;
    }
    for i in 0..c.l_extranul {
        *(*b).data.add(c.l_qname as usize + i as usize) = 0;
    }
    c.l_qname += c.l_extranul as u16;

    if (*b).l_data < c.l_qname as c_int {
        return -4;
    }
    let rest = ((*b).l_data - c.l_qname as c_int) as usize;
    if bgzf_read_small(fp, (*b).data.add(c.l_qname as usize).cast(), rest) != rest as isize {
        return -4;
    }
    if bam_tag2cigar(b, 0, 0) < 0 {
        return -4;
    }

    if c.n_cigar > 0 {
        let mut rlen = 0;
        let mut qlen = 0;
        bam_cigar2rqlens(c.n_cigar as c_int, bam_get_cigar(b), &mut rlen, &mut qlen);
        if (c.flag as c_int & BAM_FUNMAP) != 0 || rlen == 0 {
            rlen = 1;
        }
        c.bin = hts_reg2bin(c.pos, c.pos + rlen, 14, 5) as u16;
        if c.l_qseq > 0 && (c.flag as c_int & BAM_FUNMAP) == 0 && qlen != c.l_qseq as hts_pos_t {
            return -4;
        }
    }

    4 + block_len
}

unsafe fn sam_c_4157_sam_read1_sam(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> c_int {
    if (*fp).line.l != 0 {
        let ret = sam_c_2662_sam_parse1(&mut (*fp).line, h, b);
        (*fp).line.l = 0;
        return ret;
    }

    if !(*fp).state.is_null() {
        return hts_sys::sam_read1(fp.cast(), h.cast(), b.cast());
    }

    loop {
        let ret = hts_sys::hts_getline(
            fp.cast(),
            2,
            (&mut (*fp).line as *mut crate::htslib_mini_rs::hts::kstring_t).cast(),
        );
        if ret < 0 {
            return ret;
        }

        let ret = sam_c_2662_sam_parse1(&mut (*fp).line, h, b);
        (*fp).line.l = 0;
        if ret >= 0 {
            return ret;
        }
        if h.is_null() || (*h).ignore_sam_err == 0 {
            return ret;
        }
    }
}

pub unsafe fn sam_read1(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> c_int {
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

unsafe fn sam_read1_unfiltered(fp: *mut htsFile, h: *mut sam_hdr_t, b: *mut bam1_t) -> c_int {
    if fp.is_null() || b.is_null() {
        return -3;
    }
    match (*fp).format.format {
        HTS_FORMAT_BAM => sam_read1_bam(fp, h, b),
        HTS_FORMAT_EMPTY_FORMAT => {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EPIPE as c_int;
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
                } as c_int)
                .cast();
                if (*fp).state.is_null() {
                    return -2;
                }
            }
            sam_c_3927_fastq_parse1(fp, b)
        }
        _ => {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::ENOEXEC as c_int;
            -3
        }
    }
}

unsafe extern "C" fn sam_readrec(
    _ignored: *mut crate::htslib_mini_rs::hts::BGZF,
    fpv: *mut c_void,
    bv: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    let fp = fpv.cast::<htsFile>();
    let b = bv.cast::<bam1_t>();
    (*fp).line.l = 0;
    let ret = sam_read1(fp, (*fp).bam_header.cast(), b);
    if ret >= 0 {
        *tid = (*b).core.tid;
        *beg = (*b).core.pos;
        *end = bam_endpos(b);
    }
    ret
}

unsafe extern "C" fn sam_readrec_rest(
    _ignored: *mut crate::htslib_mini_rs::hts::BGZF,
    fpv: *mut c_void,
    bv: *mut c_void,
    _tid: *mut c_int,
    _beg: *mut hts_pos_t,
    _end: *mut hts_pos_t,
) -> c_int {
    let fp = fpv.cast::<htsFile>();
    let b = bv.cast::<bam1_t>();
    (*fp).line.l = 0;
    sam_read1(fp, (*fp).bam_header.cast(), b)
}

pub unsafe fn bam_plp_init(_func: bam_plp_auto_f, _data: *mut c_void) -> bam_plp_t {
    let iter = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<bam_plp_s>() as u64)
        .cast::<bam_plp_s>();
    (*iter).mp = mp_init();
    (*iter).head = mp_alloc((*iter).mp);
    (*iter).tail = (*iter).head;
    (*iter).max_tid = -1;
    (*iter).max_pos = -1;
    (*iter).maxcnt = 8000;
    if _func.is_some() {
        (*iter).func = _func;
        (*iter).data = _data;
        (*iter).b = bam_init1();
    }
    iter
}

pub unsafe fn bam_plp_init_overlaps(_iter: bam_plp_t) -> c_int {
    (*_iter).overlaps = Box::into_raw(Box::new(OlapHash::new())).cast::<olap_hash_t>();
    if (*_iter).overlaps.is_null() {
        -1
    } else {
        0
    }
}

pub unsafe fn bam_plp_destroy(_iter: bam_plp_t) {
    if _iter.is_null() {
        return;
    }
    if !(*_iter).overlaps.is_null() {
        drop(Box::from_raw((*_iter).overlaps.cast::<OlapHash>()));
    }
    let mut p = (*_iter).head;
    while !p.is_null() {
        if (*_iter).plp_destruct.is_some() && p != (*_iter).tail {
            (*_iter).plp_destruct.unwrap()((*_iter).data, &(*p).b, &mut (*p).cd);
        }
        let pnext = (*p).next;
        mp_free((*_iter).mp, p);
        p = pnext;
    }
    mp_destroy((*_iter).mp);
    if !(*_iter).b.is_null() {
        bam_destroy1((*_iter).b);
    }
    crate::htslib_mini_rs::c_compat::free((*_iter).plp.cast());
    crate::htslib_mini_rs::c_compat::free(_iter.cast());
}

pub unsafe fn bam_plp_constructor(_plp: bam_plp_t, _func: bam_plp_constructor_f) {
    (*_plp).plp_construct = _func;
}

pub unsafe fn bam_plp_destructor(_plp: bam_plp_t, _func: bam_plp_constructor_f) {
    (*_plp).plp_destruct = _func;
}

pub unsafe fn bam_plp_auto(
    _iter: bam_plp_t,
    _tid: *mut c_int,
    _pos: *mut c_int,
    _n_plp: *mut c_int,
) -> *const bam_pileup1_t {
    let mut pos64 = 0;
    let p = bam_plp64_auto(_iter, _tid, &mut pos64, _n_plp);
    if pos64 < c_int::MAX as hts_pos_t {
        *_pos = pos64 as c_int;
    } else {
        *_pos = c_int::MAX;
        (*_iter).error = 1;
        *_n_plp = -1;
        return std::ptr::null();
    }
    p
}

pub unsafe fn bam_plp64_auto(
    _iter: bam_plp_t,
    _tid: *mut c_int,
    _pos: *mut hts_pos_t,
    _n_plp: *mut c_int,
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
        let ret = (*_iter).func.unwrap()((*_iter).data, (*_iter).b);
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
        if bam_plp_push(_iter, (*_iter).b) < 0 {
            *_n_plp = -1;
            return std::ptr::null();
        }
        plp = bam_plp64_next(_iter, _tid, _pos, _n_plp);
        if !plp.is_null() {
            return plp;
        }
    }
}

pub unsafe fn bam_plp_set_maxcnt(_iter: bam_plp_t, _maxcnt: c_int) {
    (*_iter).maxcnt = _maxcnt;
}

pub unsafe fn bam_plp64_next(
    _iter: bam_plp_t,
    _tid: *mut c_int,
    _pos: *mut hts_pos_t,
    _n_plp: *mut c_int,
) -> *const bam_pileup1_t {
    if (*_iter).error != 0 {
        *_n_plp = -1;
        return std::ptr::null();
    }
    *_n_plp = 0;
    if (*_iter).is_eof != 0 && (*_iter).head == (*_iter).tail {
        return std::ptr::null();
    }
    while (*_iter).is_eof != 0
        || (*_iter).max_tid > (*_iter).tid
        || ((*_iter).max_tid == (*_iter).tid && (*_iter).max_pos > (*_iter).pos)
    {
        let mut n_plp = 0;
        let mut pptr: *mut *mut lbnode_t = &mut (*_iter).head;
        while *pptr != (*_iter).tail {
            let p = *pptr;
            if (*p).b.core.tid < (*_iter).tid
                || ((*p).b.core.tid == (*_iter).tid && (*p).end <= (*_iter).pos)
            {
                overlap_remove(_iter, &(*p).b);
                if (*_iter).plp_destruct.is_some() {
                    (*_iter).plp_destruct.unwrap()((*_iter).data, &(*p).b, &mut (*p).cd);
                }
                *pptr = (*p).next;
                mp_free((*_iter).mp, p);
            } else {
                if (*p).b.core.tid == (*_iter).tid && (*p).beg <= (*_iter).pos {
                    if n_plp == (*_iter).max_plp {
                        (*_iter).max_plp = if (*_iter).max_plp != 0 {
                            (*_iter).max_plp << 1
                        } else {
                            256
                        };
                        (*_iter).plp = crate::htslib_mini_rs::c_compat::realloc(
                            (*_iter).plp.cast(),
                            (std::mem::size_of::<bam_pileup1_t>() * (*_iter).max_plp as usize)
                                as u64,
                        )
                        .cast();
                    }
                    let out = (*_iter).plp.add(n_plp as usize);
                    (*out).b = &mut (*p).b;
                    (*out).cd = (*p).cd;
                    if resolve_cigar2(out, (*_iter).pos, &mut (*p).s) != 0 {
                        n_plp += 1;
                    }
                }
                pptr = &mut (**pptr).next;
            }
        }
        *_n_plp = n_plp;
        *_tid = (*_iter).tid;
        *_pos = (*_iter).pos;
        if (*_iter).head != (*_iter).tail && (*_iter).tid > (*(*_iter).head).b.core.tid {
            (*_iter).error = 1;
            *_n_plp = -1;
            return std::ptr::null();
        }
        if (*_iter).tid < (*(*_iter).head).b.core.tid {
            (*_iter).tid = (*(*_iter).head).b.core.tid;
            (*_iter).pos = (*(*_iter).head).beg;
        } else if (*_iter).pos < (*(*_iter).head).beg {
            (*_iter).pos = (*(*_iter).head).beg;
        } else {
            (*_iter).pos += 1;
        }
        if n_plp != 0 {
            return (*_iter).plp;
        }
        if (*_iter).is_eof != 0 && (*_iter).head == (*_iter).tail {
            break;
        }
    }
    std::ptr::null()
}

pub unsafe fn bam_plp_next(
    _iter: bam_plp_t,
    _tid: *mut c_int,
    _pos: *mut c_int,
    _n_plp: *mut c_int,
) -> *const bam_pileup1_t {
    let mut pos64 = 0;
    let p = bam_plp64_next(_iter, _tid, &mut pos64, _n_plp);
    if pos64 < c_int::MAX as hts_pos_t {
        *_pos = pos64 as c_int;
    } else {
        *_pos = c_int::MAX;
        (*_iter).error = 1;
        *_n_plp = -1;
        return std::ptr::null();
    }
    p
}

pub unsafe fn bam_plp_push(_iter: bam_plp_t, b: *const bam1_t) -> c_int {
    if (*_iter).error != 0 {
        return -1;
    }
    if !b.is_null() {
        if (*b).core.tid < 0 {
            overlap_remove(_iter, b);
            return 0;
        }
        if ((*b).core.flag as c_int & BAM_FUNMAP) != 0 {
            overlap_remove(_iter, b);
            return 0;
        }
        if (*_iter).tid == (*b).core.tid
            && (*_iter).pos == (*b).core.pos
            && (*(*_iter).mp).cnt > (*_iter).maxcnt
        {
            overlap_remove(_iter, b);
            return 0;
        }
        if bam_copy1(&mut (*(*_iter).tail).b, b).is_null() {
            return -1;
        }
        (*(*_iter).tail).b.id = (*_iter).id;
        (*_iter).id += 1;
        (*(*_iter).tail).beg = (*b).core.pos;
        (*(*_iter).tail).end =
            (*b).core.pos + bam_cigar2rlen((*b).core.n_cigar as c_int, bam_get_cigar(b));
        (*(*_iter).tail).s = G_CSTATE_NULL;
        (*(*_iter).tail).s.end = (*(*_iter).tail).end - 1;
        if (*b).core.tid < (*_iter).max_tid {
            (*_iter).error = 1;
            return -1;
        }
        if (*b).core.tid == (*_iter).max_tid && (*(*_iter).tail).beg < (*_iter).max_pos {
            (*_iter).error = 1;
            return -1;
        }
        (*_iter).max_tid = (*b).core.tid;
        (*_iter).max_pos = (*(*_iter).tail).beg;
        if (*(*_iter).tail).end > (*_iter).pos || (*(*_iter).tail).b.core.tid > (*_iter).tid {
            let next = mp_alloc((*_iter).mp);
            if next.is_null() {
                (*_iter).error = 1;
                return -1;
            }
            if (*_iter).plp_construct.is_some()
                && (*_iter).plp_construct.unwrap()(
                    (*_iter).data,
                    &(*(*_iter).tail).b,
                    &mut (*(*_iter).tail).cd,
                ) < 0
            {
                mp_free((*_iter).mp, next);
                (*_iter).error = 1;
                return -1;
            }
            if overlap_push(_iter, (*_iter).tail) < 0 {
                mp_free((*_iter).mp, next);
                (*_iter).error = 1;
                return -1;
            }
            (*(*_iter).tail).next = next;
            (*_iter).tail = (*(*_iter).tail).next;
        }
    } else {
        (*_iter).is_eof = 1;
    }
    0
}

pub unsafe fn bam_plp_reset(_iter: bam_plp_t) {
    overlap_remove(_iter, std::ptr::null());
    (*_iter).max_tid = -1;
    (*_iter).max_pos = -1;
    (*_iter).tid = 0;
    (*_iter).pos = 0;
    (*_iter).is_eof = 0;
    while (*_iter).head != (*_iter).tail {
        let p = (*_iter).head;
        (*_iter).head = (*p).next;
        mp_free((*_iter).mp, p);
    }
}

pub unsafe fn bam_mplp_init(
    _n: c_int,
    _func: bam_plp_auto_f,
    _data: *mut *mut c_void,
) -> bam_mplp_t {
    let iter = crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<bam_mplp_s>() as u64)
        .cast::<bam_mplp_s>();
    (*iter).pos =
        crate::htslib_mini_rs::c_compat::calloc(_n as u64, std::mem::size_of::<hts_pos_t>() as u64)
            .cast();
    (*iter).tid =
        crate::htslib_mini_rs::c_compat::calloc(_n as u64, std::mem::size_of::<i32>() as u64)
            .cast();
    (*iter).n_plp =
        crate::htslib_mini_rs::c_compat::calloc(_n as u64, std::mem::size_of::<c_int>() as u64)
            .cast();
    (*iter).plp = crate::htslib_mini_rs::c_compat::calloc(
        _n as u64,
        std::mem::size_of::<*const bam_pileup1_t>() as u64,
    )
    .cast();
    (*iter).iter =
        crate::htslib_mini_rs::c_compat::calloc(_n as u64, std::mem::size_of::<bam_plp_t>() as u64)
            .cast();
    (*iter).n = _n;
    (*iter).min_pos = HTS_POS_MAX;
    (*iter).min_tid = u32::MAX as i32;
    for i in 0.._n {
        *(*iter).iter.add(i as usize) = bam_plp_init(_func, *_data.add(i as usize));
        *(*iter).pos.add(i as usize) = (*iter).min_pos;
        *(*iter).tid.add(i as usize) = (*iter).min_tid;
    }
    iter
}

pub unsafe fn bam_mplp_destroy(_iter: bam_mplp_t) {
    if _iter.is_null() {
        return;
    }
    for i in 0..(*_iter).n {
        bam_plp_destroy(*(*_iter).iter.add(i as usize));
    }
    crate::htslib_mini_rs::c_compat::free((*_iter).iter.cast());
    crate::htslib_mini_rs::c_compat::free((*_iter).pos.cast());
    crate::htslib_mini_rs::c_compat::free((*_iter).tid.cast());
    crate::htslib_mini_rs::c_compat::free((*_iter).n_plp.cast());
    crate::htslib_mini_rs::c_compat::free((*_iter).plp.cast());
    crate::htslib_mini_rs::c_compat::free(_iter.cast());
}

pub unsafe fn bam_mplp_init_overlaps(_iter: bam_mplp_t) -> c_int {
    let mut r = 0;
    for i in 0..(*_iter).n {
        r |= bam_plp_init_overlaps(*(*_iter).iter.add(i as usize));
    }
    if r == 0 {
        0
    } else {
        -1
    }
}

pub unsafe fn bam_mplp_auto(
    _iter: bam_mplp_t,
    _tid: *mut c_int,
    _pos: *mut c_int,
    _n_plp: *mut c_int,
    _plp: *mut *const bam_pileup1_t,
) -> c_int {
    let mut pos64 = 0;
    let ret = bam_mplp64_auto(_iter, _tid, &mut pos64, _n_plp, _plp);
    if ret >= 0 {
        if pos64 < c_int::MAX as hts_pos_t {
            *_pos = pos64 as c_int;
        } else {
            *_pos = c_int::MAX;
            return -1;
        }
    }
    ret
}

pub unsafe fn bam_mplp64_auto(
    _iter: bam_mplp_t,
    _tid: *mut c_int,
    _pos: *mut hts_pos_t,
    _n_plp: *mut c_int,
    _plp: *mut *const bam_pileup1_t,
) -> c_int {
    let mut ret = 0;
    let mut new_min_pos = HTS_POS_MAX;
    let mut new_min_tid = u32::MAX;
    for i in 0..(*_iter).n {
        let idx = i as usize;
        let tid_ptr = (*_iter).tid.add(idx);
        let pos_ptr = (*_iter).pos.add(idx);
        let n_plp_ptr = (*_iter).n_plp.add(idx);
        let plp_ptr = (*_iter).plp.add(idx);
        if *pos_ptr == (*_iter).min_pos && *tid_ptr == (*_iter).min_tid {
            let mut tid = 0;
            let mut pos = 0;
            *plp_ptr = bam_plp64_auto(*(*_iter).iter.add(idx), &mut tid, &mut pos, n_plp_ptr);
            if (*(*(*_iter).iter.add(idx))).error != 0 {
                return -1;
            }
            if !(*plp_ptr).is_null() {
                *tid_ptr = tid;
                *pos_ptr = pos;
            } else {
                *tid_ptr = 0;
                *pos_ptr = 0;
            }
        }
        if !(*plp_ptr).is_null() {
            let tid_u = *tid_ptr as u32;
            if tid_u < new_min_tid {
                new_min_tid = tid_u;
                new_min_pos = *pos_ptr;
            } else if tid_u == new_min_tid && *pos_ptr < new_min_pos {
                new_min_pos = *pos_ptr;
            }
        }
    }
    (*_iter).min_pos = new_min_pos;
    (*_iter).min_tid = new_min_tid as i32;
    if new_min_pos == HTS_POS_MAX {
        return 0;
    }
    *_tid = new_min_tid as c_int;
    *_pos = new_min_pos;
    for i in 0..(*_iter).n {
        let idx = i as usize;
        let pos = *(*_iter).pos.add(idx);
        let tid = *(*_iter).tid.add(idx);
        if pos == (*_iter).min_pos && tid == (*_iter).min_tid {
            *_n_plp.add(idx) = *(*_iter).n_plp.add(idx);
            *_plp.add(idx) = *(*_iter).plp.add(idx);
            ret += 1;
        } else {
            *_n_plp.add(idx) = 0;
            *_plp.add(idx) = std::ptr::null();
        }
    }
    ret
}

pub unsafe fn bam_mplp_set_maxcnt(_iter: bam_mplp_t, _maxcnt: c_int) {
    for i in 0..(*_iter).n {
        (*(*(*_iter).iter.add(i as usize))).maxcnt = _maxcnt;
    }
}

pub unsafe fn bam_mplp_reset(iter: bam_mplp_t) {
    (*iter).min_pos = HTS_POS_MAX;
    (*iter).min_tid = u32::MAX as i32;
    for i in 0..(*iter).n {
        let idx = i as usize;
        bam_plp_reset(*(*iter).iter.add(idx));
        *(*iter).pos.add(idx) = HTS_POS_MAX;
        *(*iter).tid.add(idx) = u32::MAX as i32;
        *(*iter).n_plp.add(idx) = 0;
        *(*iter).plp.add(idx) = std::ptr::null();
    }
}

pub unsafe fn bam_plp_insertion_mod(
    p: *const bam_pileup1_t,
    m: *mut hts_base_mod_state,
    ins: *mut kstring_t,
    del_len: *mut c_int,
) -> c_int {
    if (*p).indel <= 0 {
        if ks_resize(ins, 1) < 0 {
            return -1;
        }
        (*ins).l = 0;
        *(*ins).s = 0;
        return 0;
    }

    if !del_len.is_null() {
        *del_len = 0;
    }

    let cigar = bam_get_cigar((*p).b);
    let mut indel = 0usize;
    let mut k = (*p).cigar_ind + 1;
    while k < (*(*p).b).core.n_cigar as c_int {
        let c = *cigar.add(k as usize);
        match (c & BAM_CIGAR_MASK) as c_int {
            BAM_CPAD | BAM_CINS => indel += (c >> BAM_CIGAR_SHIFT) as usize,
            _ => break,
        }
        k += 1;
    }
    let nb = indel as c_int;

    if ks_resize(ins, indel + 1) < 0 {
        return -1;
    }
    (*ins).l = indel;

    indel = 0;
    k = (*p).cigar_ind + 1;
    let mut j = 1;
    while k < (*(*p).b).core.n_cigar as c_int {
        let c = *cigar.add(k as usize);
        match (c & BAM_CIGAR_MASK) as c_int {
            BAM_CPAD => {
                for _ in 0..(c >> BAM_CIGAR_SHIFT) {
                    *(*ins).s.add(indel) = b'*' as c_char;
                    indel += 1;
                }
            }
            BAM_CINS => {
                for _ in 0..(c >> BAM_CIGAR_SHIFT) {
                    let qpos = (*p).qpos + j - bam_pileup1_is_del(p) as c_int;
                    let base = if qpos < (*(*p).b).core.l_qseq {
                        SEQ_NT16_STR[bam_seqi(bam_get_seq((*p).b), qpos as usize) as usize]
                    } else {
                        b'N'
                    };
                    *(*ins).s.add(indel) = base as c_char;
                    indel += 1;

                    if !m.is_null() {
                        let mut mods = [hts_base_mod {
                            modified_base: 0,
                            canonical_base: 0,
                            strand: 0,
                            qual: 0,
                        }; 256];
                        let nm = bam_mods_at_qpos((*p).b, qpos, m, mods.as_mut_ptr(), 256);
                        if nm > 0 {
                            let o_indel = indel;
                            if ks_resize(ins, (*ins).l + nm as usize * 16 + 3) < 0 {
                                return -1;
                            }
                            *(*ins).s.add(indel) = b'[' as c_char;
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
                                if ks_resize(ins, indel + text.len() + 2) < 0 {
                                    return -1;
                                }
                                std::ptr::copy_nonoverlapping(
                                    text.as_ptr().cast::<c_char>(),
                                    (*ins).s.add(indel),
                                    text.len(),
                                );
                                indel += text.len();
                            }
                            *(*ins).s.add(indel) = b']' as c_char;
                            indel += 1;
                            (*ins).l += indel - o_indel;
                        }
                    }
                    j += 1;
                }
            }
            BAM_CDEL => {
                if !del_len.is_null() {
                    *del_len = (c >> BAM_CIGAR_SHIFT) as c_int;
                }
                break;
            }
            _ => break,
        }
        k += 1;
    }
    *(*ins).s.add(indel) = 0;
    (*ins).l = indel;
    nb
}

pub unsafe fn bam_plp_insertion(
    p: *const bam_pileup1_t,
    ins: *mut kstring_t,
    del_len: *mut c_int,
) -> c_int {
    bam_plp_insertion_mod(p, std::ptr::null_mut(), ins, del_len)
}

pub unsafe fn bam_mplp_constructor(_iter: bam_mplp_t, _func: bam_plp_constructor_f) {
    for i in 0..(*_iter).n {
        bam_plp_constructor(*(*_iter).iter.add(i as usize), _func);
    }
}

pub unsafe fn bam_mplp_destructor(_iter: bam_mplp_t, _func: bam_plp_constructor_f) {
    for i in 0..(*_iter).n {
        bam_plp_destructor(*(*_iter).iter.add(i as usize), _func);
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

pub unsafe fn bam_cigar_op(c: u32) -> c_int {
    (c & 0x0f) as c_int
}

pub unsafe fn bam_cigar_oplen(c: u32) -> u32 {
    c >> 4
}

pub unsafe fn bam_cigar_type(o: c_int) -> c_int {
    BAM_CIGAR_TYPE[o as usize]
}

pub unsafe fn bam_cigar2qlen(n_cigar: c_int, cigar: *const u32) -> hts_pos_t {
    let mut l = 0;
    for k in 0..n_cigar {
        let c = *cigar.add(k as usize);
        if (bam_cigar_type(bam_cigar_op(c)) & 1) != 0 {
            l += bam_cigar_oplen(c) as hts_pos_t;
        }
    }
    l
}

pub unsafe fn bam_cigar2rlen(n_cigar: c_int, cigar: *const u32) -> hts_pos_t {
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
    n_cigar: c_int,
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

pub unsafe fn read_ncigar(mut q: *const c_char) -> u32 {
    let mut n_cigar = 0u32;
    while *q != 0 && *q != b'\t' as c_char {
        if libc::isdigit(*q as u8 as c_int) == 0 {
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

pub unsafe fn parse_cigar(in_: *const c_char, a_cigar: *mut u32, n_cigar: u32) -> c_int {
    let mut p = in_;
    for i in 0..n_cigar {
        let mut overflow = 0;
        let mut q: *mut c_char = std::ptr::null_mut();
        let len = hts_str2uint(p, &mut q, 28, &mut overflow) as u32;
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

    p.offset_from(in_) as c_int
}

pub unsafe fn sam_parse_cigar(
    in_: *const c_char,
    end: *mut *mut c_char,
    a_cigar: *mut *mut u32,
    a_mem: *mut usize,
) -> isize {
    if in_.is_null() || a_cigar.is_null() || a_mem.is_null() {
        return -1;
    }
    if !end.is_null() {
        *end = in_.cast_mut();
    }

    if *in_ == b'*' as c_char {
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
        let a_tmp = crate::htslib_mini_rs::c_compat::realloc(
            (*a_cigar).cast(),
            (n_cigar * std::mem::size_of::<u32>()) as u64,
        )
        .cast::<u32>();
        if !a_tmp.is_null() {
            *a_cigar = a_tmp;
            *a_mem = n_cigar;
        } else {
            return -1;
        }
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

pub unsafe fn bam_parse_cigar(in_: *const c_char, end: *mut *mut c_char, b: *mut bam1_t) -> isize {
    if in_.is_null() || b.is_null() {
        return -1;
    }
    if !end.is_null() {
        *end = in_.cast_mut();
    }

    let n_cigar = if *in_ == b'*' as c_char {
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

    let cig = bam_get_cigar(b).cast_mut();
    if cig.cast::<u8>() != (*b).data.add((*b).l_data as usize) {
        let seq = bam_get_seq(b);
        libc::memmove(
            cig.add(n_cigar).cast(),
            seq.cast(),
            (*b).data.add((*b).l_data as usize).offset_from(seq) as usize,
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

    (*b).l_data = ((*b).l_data as isize + cig_diff * std::mem::size_of::<u32>() as isize) as c_int;
    (*b).core.n_cigar = n_cigar as u32;
    if !end.is_null() {
        *end = in_.add(diff as usize).cast_mut();
    }
    n_cigar as isize
}

pub unsafe fn subtract_check_underflow(length: usize, limit: *mut usize) -> c_int {
    if length <= *limit {
        *limit -= length;
        0
    } else {
        -1
    }
}

pub unsafe fn bam_set1(
    bam: *mut bam1_t,
    mut l_qname: usize,
    mut qname: *const c_char,
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
    seq: *const c_char,
    qual: *const c_char,
    l_aux: usize,
) -> c_int {
    if l_qname == 0 {
        l_qname = 1;
        qname = c"*".as_ptr();
    }

    let qname_nuls = 4 - l_qname % 4;
    let mut rlen = 0;
    let mut qlen = 0;
    if (flag as c_int & BAM_FUNMAP) == 0 {
        bam_cigar2rqlens(
            n_cigar as c_int,
            cigar,
            &mut rlen as *mut hts_pos_t,
            &mut qlen as *mut hts_pos_t,
        );
    }
    if rlen == 0 {
        rlen = 1;
    }

    if l_qname > 254 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return -1;
    }
    if HTS_POS_MAX - rlen <= pos {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return -1;
    }
    if (flag as c_int & BAM_FUNMAP) == 0 && l_seq > 0 && n_cigar == 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return -1;
    }
    if (flag as c_int & BAM_FUNMAP) == 0 && l_seq > 0 && l_seq as hts_pos_t != qlen {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return -1;
    }

    let mut limit = i32::MAX as usize;
    let mut u = subtract_check_underflow(l_qname + qname_nuls, &mut limit);
    u += subtract_check_underflow(n_cigar * 4, &mut limit);
    u += subtract_check_underflow(l_seq.div_ceil(2), &mut limit);
    u += subtract_check_underflow(l_seq, &mut limit);
    u += subtract_check_underflow(l_aux, &mut limit);
    if u != 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return -1;
    }

    let data_len = l_qname + qname_nuls + n_cigar * 4 + l_seq.div_ceil(2) + l_seq;
    if realloc_bam_data(bam, data_len + l_aux) < 0 {
        return -1;
    }

    (*bam).l_data = data_len as c_int;
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

    let mut cp = (*bam).data;
    crate::htslib_mini_rs::c_compat::memcpy(cp.cast(), qname.cast(), l_qname as u64);
    for i in 0..qname_nuls {
        *cp.add(l_qname + i) = 0;
    }
    cp = cp.add(l_qname + qname_nuls);

    if n_cigar > 0 {
        crate::htslib_mini_rs::c_compat::memcpy(cp.cast(), cigar.cast(), (n_cigar * 4) as u64);
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
        crate::htslib_mini_rs::c_compat::memcpy(cp.cast(), qual.cast(), l_seq as u64);
    } else {
        libc::memset(cp.cast(), 0xff, l_seq);
    }

    data_len as c_int
}

pub unsafe fn sam_cap_mapq(
    b: *mut bam1_t,
    ref_: *const c_char,
    ref_len: hts_pos_t,
    mut thres: c_int,
) -> c_int {
    let seq = bam_get_seq(b);
    let qual = bam_get_qual(b);
    let cigar = bam_get_cigar(b);
    let c = std::ptr::addr_of_mut!((*b).core);
    let mut mm = 0;
    let mut q = 0;
    let mut len = 0;
    let mut clip_l = 0;
    let mut clip_q = 0;

    if thres < 0 {
        thres = 40;
    }

    let mut y = 0;
    let mut x = (*c).pos;
    for i in 0..(*c).n_cigar {
        let cigar_i = *cigar.add(i as usize);
        let l = (cigar_i >> 4) as c_int;
        let op = (cigar_i & 0x0f) as c_int;
        if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
            let mut j = 0;
            while j < l {
                let z = y + j;
                if x + j as hts_pos_t >= ref_len || *ref_.add((x + j as hts_pos_t) as usize) == 0 {
                    break;
                }
                let c1 = bam_seqi(seq, z as usize) as c_int;
                let c2 = SEQ_NT16_TABLE[*ref_.add((x + j as hts_pos_t) as usize) as u8 as usize]
                    as c_int;
                if c2 != 15 && c1 != 15 && *qual.add(z as usize) >= 13 {
                    len += 1;
                    if c1 != 0 && c1 != c2 && *qual.add(z as usize) >= 13 {
                        mm += 1;
                        q += if *qual.add(z as usize) > 33 {
                            33
                        } else {
                            *qual.add(z as usize) as c_int
                        };
                    }
                }
                j += 1;
            }
            if j < l {
                break;
            }
            x += l as hts_pos_t;
            y += l;
            len += l;
        } else if op == BAM_CDEL {
            let mut j = 0;
            while j < l {
                if x + j as hts_pos_t >= ref_len || *ref_.add((x + j as hts_pos_t) as usize) == 0 {
                    break;
                }
                j += 1;
            }
            if j < l {
                break;
            }
            x += l as hts_pos_t;
        } else if op == BAM_CSOFT_CLIP {
            for j in 0..l {
                clip_q += *qual.add((y + j) as usize) as c_int;
            }
            clip_l += l;
            y += l;
        } else if op == BAM_CHARD_CLIP {
            clip_q += 13 * l;
            clip_l += l;
        } else if op == BAM_CINS {
            y += l;
        } else if op == BAM_CREF_SKIP {
            x += l as hts_pos_t;
        }
    }

    let mut t = 1.0f64;
    for i in 0..mm {
        t *= len as f64 / (i + 1) as f64;
    }
    let _ = clip_l;
    t = q as f64 - 4.343 * t.ln() + clip_q as f64 / 5.0;
    if t > thres as f64 {
        return -1;
    }
    if t < 0.0 {
        t = 0.0;
    }
    t = ((thres as f64 - t) / thres as f64).sqrt() * thres as f64;
    (t + 0.499) as c_int
}

pub unsafe fn sam_prob_realn(
    b: *mut bam1_t,
    ref_: *const c_char,
    ref_len: hts_pos_t,
    flag: c_int,
) -> c_int {
    let mut k: c_int;
    let mut bw: c_int;
    let mut y: c_int;
    let mut yb: c_int;
    let mut ye: c_int;
    let mut xb: hts_pos_t;
    let mut xe: hts_pos_t;
    let mut fix_bq: c_int = 0;
    let apply_baq = flag & 1;
    let extend_baq = flag & 2;
    let redo_baq = flag & 4;
    let system = flag & (0xff << 3);
    let mut i: hts_pos_t;
    let mut x: hts_pos_t;
    let cigar = bam_get_cigar(b);
    const BAQ_ILLUMINA: c_int = 1 << 3;
    const SEQ_NT16_INT: [u8; 16] = [4, 0, 1, 4, 2, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4];

    let mut conf = crate::htslib_mini_rs::probaln::probaln_par_t {
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
    if ((*b).core.flag as c_int & BAM_FUNMAP) != 0 || (*b).core.l_qseq == 0 || *qual == u8::MAX {
        return -1;
    }

    bq = bam_aux_get(b, c"BQ".as_ptr());
    if !bq.is_null() {
        if redo_baq == 0
            && realn_check_tag(
                bq,
                crate::htslib_mini_rs::hts::HTS_LOG_WARNING,
                c"BQ".as_ptr(),
                b,
            ) < 0
        {
            fix_bq = 1;
        }
        bq = bq.add(1);
    }
    zq = bam_aux_get(b, c"ZQ".as_ptr());
    if !zq.is_null() {
        if realn_check_tag(
            zq,
            crate::htslib_mini_rs::hts::HTS_LOG_ERROR,
            c"ZQ".as_ptr(),
            b,
        ) < 0
        {
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
                *q = if (*q as c_int) + 64 < *v as c_int {
                    0
                } else {
                    ((*q as c_int) - ((*v as c_int) - 64)) as u8
                };
                i += 1;
            }
            *bq.sub(3) = b'Z';
        } else if !zq.is_null() && apply_baq == 0 {
            i = 0;
            while i < (*b).core.l_qseq as hts_pos_t {
                let q = qual.add(i as usize);
                *q = ((*q as c_int) + (*zq.add(i as usize) as c_int) - 64) as u8;
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
    while k < (*b).core.n_cigar as c_int {
        let op = (*cigar.add(k as usize) & 0xf) as c_int;
        let l = (*cigar.add(k as usize) >> 4) as c_int;
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
        bw = ((xe - xb) - (ye - yb) as hts_pos_t).abs() as c_int + 3;
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
    if (usize::MAX - lref) / (3 + std::mem::size_of::<c_int>()) < align_lqseq {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -4;
    }
    let Some(total) = align_lqseq.checked_mul(3).and_then(|n| n.checked_add(lref)) else {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
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

    let mut state_buf = vec![0 as c_int; (*b).core.l_qseq as usize];
    let state = state_buf.as_mut_ptr();
    if crate::htslib_mini_rs::probaln::probaln_glocal(
        tref,
        (xe - xb) as c_int,
        tseq,
        (*b).core.l_qseq,
        qual,
        &conf,
        state,
        q,
    ) == c_int::MIN
    {
        return -4;
    }

    if extend_baq == 0 {
        k = 0;
        x = (*b).core.pos;
        y = 0;
        while k < (*b).core.n_cigar as c_int {
            let op = (*cigar.add(k as usize) & 0xf) as c_int;
            let mut l = (*cigar.add(k as usize) >> 4) as c_int;
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
                    if (*state.add(i as usize) & 3) != 0
                        || (*state.add(i as usize) >> 2) != (x - xb + (i - y as hts_pos_t)) as c_int
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
                ((*qual.add(i as usize) as c_int) - (*bq.add(i as usize) as c_int) + 64) as u8;
            i += 1;
        }
    } else {
        let left = tseq;
        let rght = tref;
        let mut len: c_int = 0;

        k = 0;
        x = (*b).core.pos;
        y = 0;
        while k < (*b).core.n_cigar as c_int {
            let op = (*cigar.add(k as usize) & 0xf) as c_int;
            let mut l = (*cigar.add(k as usize) >> 4) as c_int;
            if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                if k + 1 < (*b).core.n_cigar as c_int {
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
                    *bq.add(i as usize) = if (*state.add(i as usize) & 3) != 0
                        || (*state.add(i as usize) >> 2) != (x - xb + (i - y as hts_pos_t)) as c_int
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
                    (*qual.add(i as usize) as c_int) - (*bq.add(i as usize) as c_int)
                }) as u8;
            i += 1;
        }
    }

    if apply_baq != 0 {
        i = 0;
        while i < (*b).core.l_qseq as hts_pos_t {
            *qual.add(i as usize) =
                ((*qual.add(i as usize) as c_int) - ((*bq.add(i as usize) as c_int) - 64)) as u8;
            i += 1;
        }
        bam_aux_append(b, c"ZQ".as_ptr(), b'Z' as c_char, (*b).core.l_qseq + 1, bq);
    } else {
        bam_aux_append(b, c"BQ".as_ptr(), b'Z' as c_char, (*b).core.l_qseq + 1, bq);
    }

    0
}

pub unsafe fn realn_check_tag(
    tg: *const u8,
    _severity: htsLogLevel,
    _type: *const c_char,
    b: *const bam1_t,
) -> c_int {
    if *tg != b'Z' {
        return -1;
    }
    if (*b).core.l_qseq as usize != libc::strlen(tg.add(1).cast()) {
        return -1;
    }
    0
}

pub unsafe fn bam_endpos(b: *const bam1_t) -> hts_pos_t {
    let mut rlen = if ((*b).core.flag as c_int & BAM_FUNMAP) != 0 {
        0
    } else {
        bam_cigar2rlen((*b).core.n_cigar as c_int, bam_get_cigar(b))
    };
    if rlen == 0 {
        rlen = 1;
    }
    (*b).core.pos + rlen
}

pub unsafe fn bam_get_cigar(b: *const bam1_t) -> *const u32 {
    (*b).data.add((*b).core.l_qname as usize) as *const u32
}

pub unsafe fn bam_is_rev(b: *const bam1_t) -> bool {
    ((*b).core.flag as c_int & BAM_FREVERSE) != 0
}

pub unsafe fn bam_is_mrev(b: *const bam1_t) -> bool {
    ((*b).core.flag as c_int & BAM_FMREVERSE) != 0
}

pub unsafe fn bam_get_qname(b: *const bam1_t) -> *mut c_char {
    (*b).data.cast()
}

pub unsafe fn bam_get_seq(b: *const bam1_t) -> *const u8 {
    (*b).data
        .add(((*b).core.n_cigar << 2) as usize + (*b).core.l_qname as usize)
}

pub unsafe fn bam_get_qual(b: *const bam1_t) -> *const u8 {
    (*b).data.add(
        ((*b).core.n_cigar << 2) as usize
            + (*b).core.l_qname as usize
            + (((*b).core.l_qseq + 1) >> 1) as usize,
    )
}

pub unsafe fn bam_get_aux(b: *const bam1_t) -> *const u8 {
    (*b).data.add(
        ((*b).core.n_cigar << 2) as usize
            + (*b).core.l_qname as usize
            + (((*b).core.l_qseq + 1) >> 1) as usize
            + (*b).core.l_qseq as usize,
    )
}

pub unsafe fn bam_get_l_aux(b: *const bam1_t) -> c_int {
    (*b).l_data
        - (((*b).core.n_cigar << 2) as c_int)
        - (*b).core.l_qname as c_int
        - (*b).core.l_qseq
        - (((*b).core.l_qseq + 1) >> 1)
}

unsafe fn fixup_missing_qname_nul(b: *mut bam1_t) -> c_int {
    let c = &mut (*b).core;
    if c.l_extranul > 0 {
        *(*b).data.add(c.l_qname as usize) = 0;
        c.l_qname += 1;
        c.l_extranul -= 1;
    } else {
        if (*b).l_data > c_int::MAX - 4 {
            return -1;
        }
        if realloc_bam_data(b, ((*b).l_data + 4) as usize) < 0 {
            return -1;
        }
        (*b).l_data += 4;
        *(*b).data.add(c.l_qname as usize) = 0;
        c.l_qname += 1;
        c.l_extranul = 3;
    }
    0
}

unsafe fn bam_tag2cigar(b: *mut bam1_t, recal_bin: c_int, _give_warning: c_int) -> c_int {
    let c = &mut (*b).core;
    let test_cg = BAM_CSOFT_CLIP as u32 | ((c.l_qseq as u32) << BAM_CIGAR_SHIFT);
    if c.n_cigar == 0 || test_cg != *bam_get_cigar(b) {
        return 0;
    }
    if c.tid < 0 || c.pos < 0 {
        return 0;
    }

    let cg = bam_aux_get(b, b"CG\0".as_ptr().cast());
    let saved_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
    if cg.is_null() {
        if *crate::htslib_mini_rs::c_compat::__errno_location()
            != crate::htslib_mini_rs::c_compat::ENOENT as c_int
        {
            return -1;
        }
        *crate::htslib_mini_rs::c_compat::__errno_location() = saved_errno;
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

    let cigar_st = (cigar0.cast::<u8>()).offset_from((*b).data) as u32;
    c.n_cigar = cg_len;
    let n_cigar4 = c.n_cigar * 4;
    let cg_st = cg.offset_from((*b).data) as u32 - 2;
    let cg_en = cg_st + 8 + n_cigar4;
    let ori_len = (*b).l_data as u32;
    if possibly_expand_bam_data(b, (n_cigar4 - fake_bytes) as usize) < 0 {
        return -1;
    }
    (*b).l_data = (ori_len - fake_bytes + n_cigar4) as c_int;

    crate::htslib_mini_rs::c_compat::memmove(
        (*b).data.add((cigar_st + n_cigar4) as usize).cast(),
        (*b).data.add((cigar_st + fake_bytes) as usize).cast(),
        (ori_len - (cigar_st + fake_bytes)) as u64,
    );
    crate::htslib_mini_rs::c_compat::memcpy(
        (*b).data.add(cigar_st as usize).cast(),
        (*b).data
            .add((n_cigar4 - fake_bytes + cg_st + 8) as usize)
            .cast(),
        n_cigar4 as u64,
    );
    if ori_len > cg_en {
        crate::htslib_mini_rs::c_compat::memmove(
            (*b).data
                .add((cg_st + n_cigar4 - fake_bytes) as usize)
                .cast(),
            (*b).data
                .add((cg_en + n_cigar4 - fake_bytes) as usize)
                .cast(),
            (ori_len - cg_en) as u64,
        );
    }
    (*b).l_data -= (n_cigar4 + 8) as c_int;
    if recal_bin != 0 {
        c.bin = hts_reg2bin(c.pos, bam_endpos(b), 14, 5) as u16;
    }
    1
}

pub unsafe fn bam_aux_tag(s: *const u8) -> *const c_char {
    s.sub(2).cast()
}

pub unsafe fn bam_aux_type(s: *const u8) -> c_char {
    *s as c_char
}

fn aux_type2size(type_: u8) -> c_int {
    match type_ {
        b'A' | b'c' | b'C' => 1,
        b's' | b'S' => 2,
        b'i' | b'I' | b'f' => 4,
        b'd' => 8,
        b'Z' | b'H' | b'B' => type_ as c_int,
        _ => 0,
    }
}

unsafe fn sam_c_755_swap_data(
    c: *const bam1_core_t,
    _l_data: c_int,
    data: *mut u8,
    _is_host: c_int,
) {
    let cigar = data.add((*c).l_qname as usize).cast::<u32>();
    for i in 0..(*c).n_cigar {
        ed_swap_4p(cigar.add(i as usize).cast());
    }
}

pub unsafe fn bam_write1(fp: *mut BGZF, b: *const bam1_t) -> c_int {
    let c = &(*b).core;
    let mut block_len = ((*b).l_data - c.l_extranul as c_int + 32) as u32;
    let mut x = [0u32; 8];

    let qname_len = c.l_qname as u32 - c.l_extranul as u32;
    if qname_len > 255 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int;
        return -1;
    }
    if c.n_cigar > 0xffff {
        block_len = block_len.wrapping_add(16);
    }
    if c.pos > c_int::MAX as hts_pos_t
        || c.mpos > c_int::MAX as hts_pos_t
        || c.isize < c_int::MIN as hts_pos_t
        || c.isize > c_int::MAX as hts_pos_t
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
        sam_c_755_swap_data(c, (*b).l_data, (*b).data, 1);
    } else if ok {
        ok = bgzf_write_small(fp, (&block_len as *const u32).cast(), 4) >= 0;
    }
    if ok {
        ok = bgzf_write_small(fp, x.as_ptr().cast(), 32) >= 0;
    }
    if ok {
        ok = bgzf_write_small(fp, (*b).data.cast(), qname_len as usize) >= 0;
    }
    if c.n_cigar <= 0xffff {
        if ok {
            ok = bgzf_write_small(
                fp,
                (*b).data.add(c.l_qname as usize).cast(),
                ((*b).l_data as u32 - c.l_qname as u32) as usize,
            ) >= 0;
        }
    } else {
        let mut buf = [0u8; 8];
        let cigreflen = bam_cigar2rlen(c.n_cigar as c_int, bam_get_cigar(b));
        if cigreflen >= (1 << 28) {
            return -1;
        }
        let cigar_st = bam_get_cigar(b).cast::<u8>().offset_from((*b).data) as u32;
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
                (*b).data.add(cigar_en as usize).cast(),
                ((*b).l_data as u32 - cigar_en) as usize,
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
                (*b).data.add(cigar_st as usize).cast(),
                c.n_cigar as usize * 4,
            ) >= 0;
        }
    }
    if ((*fp).bitfields & (1 << 19)) != 0 {
        sam_c_755_swap_data(c, (*b).l_data, (*b).data, 0);
    }
    if ok {
        (4 + block_len) as c_int
    } else {
        -1
    }
}

unsafe fn sam_c_933_bam_write_idx1(
    fp: *mut htsFile,
    _h: *const sam_hdr_t,
    b: *const bam1_t,
) -> c_int {
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
) -> c_int {
    match (*fp).format.format {
        HTS_FORMAT_BINARY_FORMAT => {
            (*fp).format.category = HTS_FORMAT_SEQUENCE_DATA;
            (*fp).format.format = HTS_FORMAT_BAM;
            sam_c_933_bam_write_idx1(fp, h, b)
        }
        HTS_FORMAT_BAM => sam_c_933_bam_write_idx1(fp, h, b),
        HTS_FORMAT_CRAM => hts_sys::sam_write1(fp.cast(), h.cast(), b.cast()),
        HTS_FORMAT_TEXT_FORMAT => {
            (*fp).format.category = HTS_FORMAT_SEQUENCE_DATA;
            (*fp).format.format = HTS_FORMAT_SAM;
            sam_c_4553_sam_write1(fp, h, b)
        }
        HTS_FORMAT_SAM => {
            if !(*fp).state.is_null() || !(*fp).idx.is_null() {
                return hts_sys::sam_write1(fp.cast(), h.cast(), b.cast());
            }
            if sam_format1(h, b, &mut (*fp).line) < 0 {
                return -1;
            }
            kputc(b'\n' as c_int, &mut (*fp).line);
            if ((*fp).bitfields & (1 << 4)) != 0 {
                if bgzf_flush_try((*fp).fp.bgzf, (*fp).line.l as isize) < 0 {
                    return -1;
                }
                if bgzf_write((*fp).fp.bgzf, (*fp).line.s.cast(), (*fp).line.l)
                    != (*fp).line.l as isize
                {
                    return -1;
                }
            } else {
                if crate::htslib_mini_rs::hfile::htslib_hfile_h_292_hwrite(
                    (*fp).fp.hfile,
                    (*fp).line.s.cast(),
                    (*fp).line.l,
                ) != (*fp).line.l as libc::ssize_t
                {
                    return -1;
                }
            }
            (*fp).line.l as c_int
        }
        HTS_FORMAT_FASTA_FORMAT | HTS_FORMAT_FASTQ_FORMAT => {
            if (*fp).state.is_null() {
                (*fp).state = sam_c_3786_fastq_state_init(if (*fp).format.format
                    == HTS_FORMAT_FASTQ_FORMAT
                {
                    b'@'
                } else {
                    b'>'
                } as c_int)
                .cast();
                if (*fp).state.is_null() {
                    return -2;
                }
            }
            if sam_c_4413_fastq_format1((*fp).state.cast(), b, &mut (*fp).line) < 0 {
                return -1;
            }
            if ((*fp).bitfields & (1 << 4)) != 0 {
                if bgzf_flush_try((*fp).fp.bgzf, (*fp).line.l as isize) < 0 {
                    return -1;
                }
                if bgzf_write((*fp).fp.bgzf, (*fp).line.s.cast(), (*fp).line.l)
                    != (*fp).line.l as isize
                {
                    return -1;
                }
            } else {
                if crate::htslib_mini_rs::hfile::htslib_hfile_h_292_hwrite(
                    (*fp).fp.hfile,
                    (*fp).line.s.cast(),
                    (*fp).line.l,
                ) != (*fp).line.l as libc::ssize_t
                {
                    return -1;
                }
            }
            (*fp).line.l as c_int
        }
        _ => {
            *crate::htslib_mini_rs::c_compat::__errno_location() = libc::EBADF;
            -1
        }
    }
}

pub unsafe fn bam_set_qname(rec: *mut bam1_t, qname: *const c_char) -> c_int {
    if rec.is_null() {
        return -1;
    }
    if qname.is_null() || *qname == 0 {
        return -1;
    }

    let old_len = (*rec).core.l_qname as usize;
    let new_len = libc::strlen(qname) + 1;
    if new_len < 1 || new_len > 255 {
        return -1;
    }
    let extranul = if new_len % 4 != 0 { 4 - new_len % 4 } else { 0 };
    let new_data_len = (*rec).l_data as usize - old_len + new_len + extranul;
    if realloc_bam_data(rec, new_data_len) < 0 {
        return -1;
    }
    if new_len + extranul != (*rec).core.l_qname as usize {
        libc::memmove(
            (*rec).data.add(new_len + extranul).cast(),
            (*rec).data.add((*rec).core.l_qname as usize).cast(),
            (*rec).l_data as usize - (*rec).core.l_qname as usize,
        );
    }
    crate::htslib_mini_rs::c_compat::memcpy((*rec).data.cast(), qname.cast(), new_len as u64);
    for n in 0..extranul {
        *(*rec).data.add(new_len + n) = 0;
    }
    (*rec).l_data = new_data_len as c_int;
    (*rec).core.l_qname = (new_len + extranul) as u16;
    (*rec).core.l_extranul = extranul as u8;
    0
}

pub unsafe fn aux_to_le(type_: c_char, mut out: *mut u8, in_: *const u8, len: usize) -> c_int {
    let tsz = aux_type2size(type_ as u8);
    if (2..=8).contains(&tsz) && (len & (tsz as usize - 1)) != 0 {
        return -1;
    }

    match tsz {
        x if x == b'H' as c_int || x == b'Z' as c_int || x == 1 => {
            crate::htslib_mini_rs::c_compat::memcpy(out.cast(), in_.cast(), len as u64);
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
        x if x == b'B' as c_int => {
            if len < 5 {
                return -1;
            }
            let n = u32::from_ne_bytes([*in_.add(1), *in_.add(2), *in_.add(3), *in_.add(4)]);
            *out = *in_;
            u32_to_le(n, out.add(1));
            return aux_to_le(*in_ as c_char, out.add(5), in_.add(5), len - 5);
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
        x if x == b'Z' as c_int || x == b'H' as c_int => {
            while s < end {
                if *s == 0 {
                    return s.add(1);
                }
                s = s.add(1);
            }
            end
        }
        x if x == b'B' as c_int => {
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
    let end = (*b).data.add((*b).l_data as usize);
    if end.offset_from(s) <= 2 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOENT as c_int;
        return std::ptr::null_mut();
    }
    s.add(2)
}

pub unsafe fn bam_aux_next(b: *const bam1_t, s: *const u8) -> *mut u8 {
    let end = (*b).data.add((*b).l_data as usize);
    let next = if s.is_null() {
        end
    } else {
        skip_aux(s.cast_mut(), end)
    };
    if next.is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return std::ptr::null_mut();
    }
    if end.offset_from(next) <= 2 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOENT as c_int;
        return std::ptr::null_mut();
    }
    next.add(2)
}

pub unsafe fn bam_aux_get(b: *const bam1_t, tag: *const c_char) -> *mut u8 {
    let mut s = bam_aux_first(b);
    while !s.is_null() {
        if *s.sub(2) == *tag.cast::<u8>() && *s.sub(1) == *tag.cast::<u8>().add(1) {
            let e = skip_aux(s, (*b).data.add((*b).l_data as usize));
            if e.is_null() || ((*s == b'Z' || *s == b'H') && *e.sub(1) != 0) {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
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
    r |= (kputsn_(key.cast(), 2, ks) < 0) as c_int;
    r |= (kputc_(b':' as c_int, ks) < 0) as c_int;

    match type_ {
        b'C' => {
            r |= (kputsn_(b"i:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputw(*s as c_int, ks) < 0) as c_int;
            s = s.add(1);
        }
        b'c' => {
            r |= (kputsn_(b"i:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputw(*(s.cast::<i8>()) as c_int, ks) < 0) as c_int;
            s = s.add(1);
        }
        b'S' => {
            if end.offset_from(s) < 2 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputuw(u16::from_le_bytes([*s, *s.add(1)]) as u32, ks) < 0) as c_int;
            s = s.add(2);
        }
        b's' => {
            if end.offset_from(s) < 2 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputw(i16::from_le_bytes([*s, *s.add(1)]) as c_int, ks) < 0) as c_int;
            s = s.add(2);
        }
        b'I' => {
            if end.offset_from(s) < 4 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputuw(
                u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                ks,
            ) < 0) as c_int;
            s = s.add(4);
        }
        b'i' => {
            if end.offset_from(s) < 4 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"i:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputw(
                i32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                ks,
            ) < 0) as c_int;
            s = s.add(4);
        }
        b'A' => {
            if s >= end {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"A:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputc_(*s as c_int, ks) < 0) as c_int;
            s = s.add(1);
        }
        b'f' => {
            if end.offset_from(s) < 4 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"f:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (sam_put_aux_float(
                f32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]) as f64,
                ks,
            ) < 0) as c_int;
            s = s.add(4);
        }
        b'd' => {
            if end.offset_from(s) < 8 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"d:".as_ptr().cast(), 2, ks) < 0) as c_int;
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
            ) < 0) as c_int;
            s = s.add(8);
        }
        b'Z' | b'H' => {
            r |= (kputc_(type_ as c_int, ks) < 0) as c_int;
            r |= (kputc_(b':' as c_int, ks) < 0) as c_int;
            while s < end && *s != 0 {
                r |= (kputc_(*s as c_int, ks) < 0) as c_int;
                s = s.add(1);
            }
            r |= (kputsn(std::ptr::null(), 0, ks) < 0) as c_int;
            if s >= end {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            s = s.add(1);
        }
        b'B' => {
            if s >= end {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
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
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            let n = u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]);
            s = s.add(4);
            if (end.offset_from(s) as usize) / sub_type_size < n as usize {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            r |= (kputsn_(b"B:".as_ptr().cast(), 2, ks) < 0) as c_int;
            r |= (kputc(sub_type as c_int, ks) < 0) as c_int;
            if sub_type == b'A' {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return std::ptr::null();
            }
            if ks_expand(ks, n as usize * 12) < 0 {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
                return std::ptr::null();
            }
            for _ in 0..n {
                *(*ks).s.add((*ks).l) = b',' as c_char;
                (*ks).l += 1;
                match sub_type {
                    b'c' => r |= (kputw(*(s.cast::<i8>()) as c_int, ks) < 0) as c_int,
                    b'C' | b'A' => r |= (kputuw(*s as u32, ks) < 0) as c_int,
                    b's' => {
                        r |= (kputw(i16::from_le_bytes([*s, *s.add(1)]) as c_int, ks) < 0) as c_int;
                    }
                    b'S' => {
                        r |= (kputuw(u16::from_le_bytes([*s, *s.add(1)]) as u32, ks) < 0) as c_int;
                    }
                    b'i' => {
                        r |= (kputw(
                            i32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                            ks,
                        ) < 0) as c_int;
                    }
                    b'I' => {
                        r |= (kputuw(
                            u32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]),
                            ks,
                        ) < 0) as c_int;
                    }
                    b'f' => {
                        r |= (sam_put_aux_float(
                            f32::from_le_bytes([*s, *s.add(1), *s.add(2), *s.add(3)]) as f64,
                            ks,
                        ) < 0) as c_int;
                    }
                    _ => {
                        *crate::htslib_mini_rs::c_compat::__errno_location() =
                            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                        return std::ptr::null();
                    }
                }
                s = s.add(sub_type_size);
            }
        }
        _ => {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EINVAL as c_int;
            return std::ptr::null();
        }
    }

    if r != 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        std::ptr::null()
    } else {
        s
    }
}

unsafe fn sam_put_aux_float(value: f64, ks: *mut kstring_t) -> c_int {
    let mut buf = [0 as c_char; 128];
    let len = libc::snprintf(buf.as_mut_ptr(), buf.len(), c"%.6g".as_ptr(), value);
    if len < 0 || len as usize >= buf.len() {
        return -1;
    }
    kputsn(buf.as_ptr(), len as usize, ks)
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
) -> c_int {
    let mut r = 0;
    let c = &(*b).core;
    const BAM_CIGAR_STR: &[u8; 10] = b"MIDNSHP=XB";

    if c.l_qname == 0 {
        return -1;
    }
    r |= (kputsn_(
        bam_get_qname(b).cast(),
        (c.l_qname - 1 - c.l_extranul as u16) as usize,
        str_,
    ) < 0) as c_int;
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    r |= (kputw(c.flag as c_int, str_) < 0) as c_int;
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    if c.tid >= 0 {
        r |= (kputs(*(*h).target_name.add(c.tid as usize), str_) < 0) as c_int;
        r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    } else {
        r |= (kputsn_(b"*\t".as_ptr().cast(), 2, str_) < 0) as c_int;
    }
    r |= (kputll(c.pos + 1, str_) < 0) as c_int;
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    r |= (kputw(c.qual as c_int, str_) < 0) as c_int;
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    if c.n_cigar != 0 {
        let cigar = bam_get_cigar(b);
        for i in 0..c.n_cigar {
            let cig = *cigar.add(i as usize);
            r |= (kputw(bam_cigar_oplen(cig) as c_int, str_) < 0) as c_int;
            r |= (kputc_(BAM_CIGAR_STR[bam_cigar_op(cig) as usize] as c_int, str_) < 0) as c_int;
        }
    } else {
        r |= (kputc_(b'*' as c_int, str_) < 0) as c_int;
    }
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    if c.mtid < 0 {
        r |= (kputsn_(b"*\t".as_ptr().cast(), 2, str_) < 0) as c_int;
    } else if c.mtid == c.tid {
        r |= (kputsn_(b"=\t".as_ptr().cast(), 2, str_) < 0) as c_int;
    } else {
        r |= (kputs(*(*h).target_name.add(c.mtid as usize), str_) < 0) as c_int;
        r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    }
    r |= (kputll(c.mpos + 1, str_) < 0) as c_int;
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
    r |= (kputll(c.isize, str_) < 0) as c_int;
    r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;

    if c.l_qseq != 0 {
        let l_qseq = c.l_qseq as usize;
        if ks_resize(str_, (*str_).l + 2 + 2 * l_qseq) < 0 {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
            return -1;
        }
        let mut cp = (*str_).s.add((*str_).l);
        nibble2base(bam_get_seq(b).cast_mut(), cp, c.l_qseq);
        *cp.add(l_qseq) = b'\t' as c_char;
        cp = cp.add(l_qseq + 1);

        let qual = bam_get_qual(b);
        let mut i = 0usize;
        if *qual == 0xff {
            *cp = b'*' as c_char;
            i += 1;
        } else {
            sam_c_4317_add33(cp.cast(), qual, c.l_qseq);
            i = l_qseq;
        }
        *cp.add(i) = 0;
        cp = cp.add(i);
        (*str_).l = cp.offset_from((*str_).s) as usize;
    } else {
        r |= (kputsn_(b"*\t*".as_ptr().cast(), 3, str_) < 0) as c_int;
    }

    let mut s = bam_get_aux(b);
    let end = (*b).data.add((*b).l_data as usize);
    while end.offset_from(s) >= 4 {
        r |= (kputc_(b'\t' as c_int, str_) < 0) as c_int;
        s = sam_format_aux1(s, *s.add(2), s.add(3), end, str_).cast_mut();
        if s.is_null() {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EINVAL as c_int;
            return -1;
        }
    }
    r |= (kputsn(std::ptr::null(), 0, str_) < 0) as c_int;
    if r != 0 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    }
    (*str_).l as c_int
}

pub unsafe fn sam_format1(h: *const sam_hdr_t, b: *const bam1_t, str_: *mut kstring_t) -> c_int {
    (*str_).l = 0;
    sam_c_4324_sam_format1_append(h, b, str_)
}

pub unsafe fn bam_aux_get_str(b: *const bam1_t, tag: *const c_char, s: *mut kstring_t) -> c_int {
    let t = bam_aux_get(b, tag);
    if t.is_null() {
        return if *crate::htslib_mini_rs::c_compat::__errno_location()
            == crate::htslib_mini_rs::c_compat::ENOENT as c_int
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
        (*b).data.add((*b).l_data as usize),
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
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EINVAL as c_int;
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

pub unsafe fn bam_aux2A(s: *const u8) -> c_char {
    if *s == b'A' {
        *s.add(1) as c_char
    } else {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        0
    }
}

pub unsafe fn bam_aux2Z(s: *const u8) -> *mut c_char {
    if *s == b'Z' || *s == b'H' {
        s.add(1).cast::<c_char>().cast_mut()
    } else {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        std::ptr::null_mut()
    }
}

pub unsafe fn bam_auxB_len(s: *const u8) -> u32 {
    if *s != b'B' {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return 0;
    }
    u32::from_le_bytes([*s.add(2), *s.add(3), *s.add(4), *s.add(5)])
}

pub unsafe fn bam_auxB2i(s: *const u8, idx: u32) -> i64 {
    let len = bam_auxB_len(s);
    if idx >= len {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ERANGE as c_int;
        return 0;
    }
    get_int_aux_val(*s.add(1), s.add(6), idx)
}

pub unsafe fn bam_auxB2f(s: *const u8, idx: u32) -> f64 {
    let len = bam_auxB_len(s);
    if idx >= len {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ERANGE as c_int;
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
    tag: *const c_char,
    type_: c_char,
    len: c_int,
    data: *const u8,
) -> c_int {
    let Ok(add_len) = u32::try_from(3_i64 + len as i64) else {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    };
    let new_len = (*b).l_data as u32;
    let Some(new_len) = new_len.checked_add(add_len) else {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    };
    if new_len > c_int::MAX as u32 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    }
    if realloc_bam_data(b, new_len as usize) < 0 {
        return -1;
    }

    let s = (*b).data.add((*b).l_data as usize);
    *s = *tag.cast::<u8>();
    *s.add(1) = *tag.cast::<u8>().add(1);
    *s.add(2) = type_ as u8;
    crate::htslib_mini_rs::c_compat::memcpy(s.add(3).cast(), data.cast(), len as u64);
    (*b).l_data = new_len as c_int;
    0
}

pub unsafe fn bam_aux_remove(b: *mut bam1_t, s: *mut u8) -> *mut u8 {
    let end = (*b).data.add((*b).l_data as usize);
    let next = skip_aux(s, end);
    if next.is_null() {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return std::ptr::null_mut();
    }

    (*b).l_data -= next.offset_from(s.sub(2)) as c_int;
    if next >= end {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOENT as c_int;
        return std::ptr::null_mut();
    }
    crate::htslib_mini_rs::c_compat::memmove(
        s.sub(2).cast(),
        next.cast(),
        end.offset_from(next) as u64,
    );
    s
}

pub unsafe fn bam_aux_del(b: *mut bam1_t, s: *mut u8) -> c_int {
    let ret = bam_aux_remove(b, s);
    if !ret.is_null()
        || *crate::htslib_mini_rs::c_compat::__errno_location()
            == crate::htslib_mini_rs::c_compat::ENOENT as c_int
    {
        0
    } else {
        -1
    }
}

unsafe fn aux_strlen(mut data: *const c_char) -> usize {
    let start = data;
    while *data != 0 {
        data = data.add(1);
    }
    data.offset_from(start) as usize
}

unsafe fn possibly_expand_bam_data(b: *mut bam1_t, extra: usize) -> c_int {
    let desired = (*b).l_data as usize + extra;
    realloc_bam_data(b, desired)
}

pub unsafe fn bam_aux_update_str(
    b: *mut bam1_t,
    tag: *const c_char,
    len: c_int,
    data: *const c_char,
) -> c_int {
    let ln = if len >= 0 {
        len as usize
    } else {
        aux_strlen(data) + 1
    };
    let mut old_ln = 0usize;
    let need_nul = ln == 0 || *data.cast::<u8>().add(ln - 1) != 0;
    let save_errno = *crate::htslib_mini_rs::c_compat::__errno_location();
    let mut new_tag = 0usize;
    let mut s = bam_aux_get(b, tag);

    if !s.is_null() {
        if *s != b'Z' {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EINVAL as c_int;
            return -1;
        }
        s = s.add(1);
        let mut e = s;
        let end = (*b).data.add((*b).l_data as usize);
        while e < end && *e != 0 {
            e = e.add(1);
        }
        old_ln = e.offset_from(s) as usize + 1;
        s = s.sub(3);
    } else if *crate::htslib_mini_rs::c_compat::__errno_location()
        != crate::htslib_mini_rs::c_compat::ENOENT as c_int
    {
        return -1;
    } else {
        *crate::htslib_mini_rs::c_compat::__errno_location() = save_errno;
        s = (*b).data.add((*b).l_data as usize);
        new_tag = 3;
    }

    let new_ln = ln + usize::from(need_nul);
    if old_ln < new_ln + new_tag {
        let s_offset = s.offset_from((*b).data) as usize;
        if possibly_expand_bam_data(b, new_ln + new_tag - old_ln) < 0 {
            return -1;
        }
        s = (*b).data.add(s_offset);
    }
    if new_tag == 0 {
        crate::htslib_mini_rs::c_compat::memmove(
            s.add(3 + new_ln).cast(),
            s.add(3 + old_ln).cast(),
            ((*b).l_data as usize - (s.add(3).offset_from((*b).data) as usize) - old_ln) as u64,
        );
    }
    (*b).l_data += new_tag as c_int + new_ln as c_int - old_ln as c_int;

    *s = *tag.cast::<u8>();
    *s.add(1) = *tag.cast::<u8>().add(1);
    *s.add(2) = b'Z';
    crate::htslib_mini_rs::c_compat::memmove(s.add(3).cast(), data.cast(), ln as u64);
    if need_nul {
        *s.add(3 + ln) = 0;
    }
    0
}

pub unsafe fn bam_aux_update_int(b: *mut bam1_t, tag: *const c_char, val: i64) -> c_int {
    if val < i32::MIN as i64 || val > u32::MAX as i64 {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int;
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
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return -1;
            }
        };
    } else if *crate::htslib_mini_rs::c_compat::__errno_location()
        == crate::htslib_mini_rs::c_compat::ENOENT as c_int
    {
        s = (*b).data.add((*b).l_data as usize);
        new = true;
    } else {
        return -1;
    }

    if new || old_sz < sz {
        let s_offset = s.offset_from((*b).data) as usize;
        if possibly_expand_bam_data(b, (if new { 3 } else { 0 }) + sz as usize - old_sz as usize)
            < 0
        {
            return -1;
        }
        s = (*b).data.add(s_offset);
        if new {
            *s = *tag.cast::<u8>();
            *s.add(1) = *tag.cast::<u8>().add(1);
            s = s.add(2);
        } else {
            crate::htslib_mini_rs::c_compat::memmove(
                s.add(sz as usize).cast(),
                s.add(old_sz as usize).cast(),
                ((*b).l_data as usize - s_offset - old_sz as usize) as u64,
            );
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
    crate::htslib_mini_rs::c_compat::memcpy(s.cast(), le.as_ptr().cast(), sz as u64);
    (*b).l_data += (if new { 3 } else { 0 }) + sz as c_int - old_sz as c_int;
    0
}

pub unsafe fn bam_aux_update_float(b: *mut bam1_t, tag: *const c_char, val: f32) -> c_int {
    let mut shrink = false;
    let mut new = false;
    let mut s = bam_aux_get(b, tag);
    if !s.is_null() {
        match *s {
            b'f' => {}
            b'd' => shrink = true,
            _ => {
                *crate::htslib_mini_rs::c_compat::__errno_location() =
                    crate::htslib_mini_rs::c_compat::EINVAL as c_int;
                return -1;
            }
        }
    } else if *crate::htslib_mini_rs::c_compat::__errno_location()
        == crate::htslib_mini_rs::c_compat::ENOENT as c_int
    {
        new = true;
    } else {
        return -1;
    }

    if new {
        if possibly_expand_bam_data(b, 7) < 0 {
            return -1;
        }
        s = (*b).data.add((*b).l_data as usize);
        *s = *tag.cast::<u8>();
        *s.add(1) = *tag.cast::<u8>().add(1);
        s = s.add(2);
    } else if shrink {
        crate::htslib_mini_rs::c_compat::memmove(
            s.add(5).cast(),
            s.add(9).cast(),
            ((*b).l_data as usize - s.add(9).offset_from((*b).data) as usize) as u64,
        );
        (*b).l_data -= 4;
    }
    *s = b'f';
    let le = val.to_le_bytes();
    crate::htslib_mini_rs::c_compat::memcpy(s.add(1).cast(), le.as_ptr().cast(), 4);
    if new {
        (*b).l_data += 7;
    }
    0
}

pub unsafe fn bam_aux_update_array(
    b: *mut bam1_t,
    tag: *const c_char,
    type_: u8,
    items: u32,
    data: *mut c_void,
) -> c_int {
    let mut old_sz = 0usize;
    let mut new = false;
    let mut s = bam_aux_get(b, tag);
    if !s.is_null() {
        if *s != b'B' {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EINVAL as c_int;
            return -1;
        }
        old_sz = aux_type2size(*s.add(1)) as usize;
        if !(1..=4).contains(&old_sz) {
            *crate::htslib_mini_rs::c_compat::__errno_location() =
                crate::htslib_mini_rs::c_compat::EINVAL as c_int;
            return -1;
        }
        old_sz *= u32::from_le_bytes([*s.add(2), *s.add(3), *s.add(4), *s.add(5)]) as usize;
    } else if *crate::htslib_mini_rs::c_compat::__errno_location()
        == crate::htslib_mini_rs::c_compat::ENOENT as c_int
    {
        s = (*b).data.add((*b).l_data as usize);
        new = true;
    } else {
        return -1;
    }

    let item_sz = aux_type2size(type_) as usize;
    if !(1..=4).contains(&item_sz) {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::EINVAL as c_int;
        return -1;
    }
    if items as usize > c_int::MAX as usize / item_sz {
        *crate::htslib_mini_rs::c_compat::__errno_location() =
            crate::htslib_mini_rs::c_compat::ENOMEM as c_int;
        return -1;
    }
    let new_sz = item_sz * items as usize;

    if new || old_sz < new_sz {
        let s_offset = s.offset_from((*b).data) as usize;
        if possibly_expand_bam_data(b, (if new { 8 } else { 0 }) + new_sz - old_sz) < 0 {
            return -1;
        }
        s = (*b).data.add(s_offset);
    }
    if new {
        *s = *tag.cast::<u8>();
        *s.add(1) = *tag.cast::<u8>().add(1);
        s = s.add(2);
        *s = b'B';
        (*b).l_data += (8 + new_sz) as c_int;
    } else if old_sz != new_sz {
        crate::htslib_mini_rs::c_compat::memmove(
            s.add(6 + new_sz).cast(),
            s.add(6 + old_sz).cast(),
            ((*b).l_data as usize - s.add(6 + old_sz).offset_from((*b).data) as usize) as u64,
        );
        (*b).l_data += new_sz as c_int - old_sz as c_int;
    }

    *s.add(1) = type_;
    let len = items.to_le_bytes();
    crate::htslib_mini_rs::c_compat::memcpy(s.add(2).cast(), len.as_ptr().cast(), 4);
    if new_sz > 0 {
        crate::htslib_mini_rs::c_compat::memcpy(s.add(6).cast(), data, new_sz as u64);
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

pub unsafe fn hts_base_mod_state_alloc() -> *mut hts_base_mod_state {
    crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<hts_base_mod_state>() as u64)
        .cast()
}

pub unsafe fn hts_base_mod_state_free(state: *mut hts_base_mod_state) {
    crate::htslib_mini_rs::c_compat::free(state.cast());
}

pub unsafe fn seq_freq(b: *const bam1_t, freq: *mut c_int) {
    libc::memset(freq.cast(), 0, 16 * std::mem::size_of::<c_int>());
    let seq = bam_get_seq(b);
    let mut i = 0;
    while i < (*b).core.l_qseq {
        *freq.add(bam_seqi(seq, i as usize) as usize) += 1;
        i += 1;
    }
    *freq.add(15) = (*b).core.l_qseq;
}

pub unsafe fn bam_mods_recorded(state: *mut hts_base_mod_state, ntype: *mut c_int) -> *mut c_int {
    *ntype = (*state).nmods;
    (*state).type_.as_mut_ptr()
}

pub unsafe fn bam_mods_query_type(
    state: *mut hts_base_mod_state,
    code: c_int,
    strand: *mut c_int,
    implicit: *mut c_int,
    canonical: *mut c_char,
) -> c_int {
    let mut i = 0;
    while i < (*state).nmods {
        if (*state).type_[i as usize] == code {
            break;
        }
        i += 1;
    }
    if i == (*state).nmods {
        return -1;
    }

    if !strand.is_null() {
        *strand = (*state).strand[i as usize] as c_int;
    }
    if !implicit.is_null() {
        *implicit = (*state).implicit[i as usize];
    }
    if !canonical.is_null() {
        *canonical = *b"?AC?G???T??????N"
            .as_ptr()
            .add((*state).canonical[i as usize] as usize) as c_char;
    }

    0
}

pub unsafe fn bam_mods_queryi(
    state: *mut hts_base_mod_state,
    i: c_int,
    strand: *mut c_int,
    implicit: *mut c_int,
    canonical: *mut c_char,
) -> c_int {
    if i < 0 || i >= (*state).nmods {
        return -1;
    }

    if !strand.is_null() {
        *strand = (*state).strand[i as usize] as c_int;
    }
    if !implicit.is_null() {
        *implicit = (*state).implicit[i as usize];
    }
    if !canonical.is_null() {
        *canonical = *b"?AC?G???T??????N"
            .as_ptr()
            .add((*state).canonical[i as usize] as usize) as c_char;
    }

    0
}

pub unsafe fn bam_parse_basemod2(
    b: *const bam1_t,
    state: *mut hts_base_mod_state,
    flags: u32,
) -> c_int {
    (*state).seq_pos = 0;
    (*state).nmods = 0;
    (*state).flags = flags;

    let mut mm = bam_aux_get(b, c"MM".as_ptr());
    if mm.is_null() {
        mm = bam_aux_get(b, c"Mm".as_ptr());
    }
    if mm.is_null() {
        return 0;
    }
    if *mm != b'Z' {
        return -1;
    }

    let mi = bam_aux_get(b, c"MN".as_ptr());
    if !mi.is_null() && bam_aux2i(mi) != (*b).core.l_qseq as i64 && (*b).core.l_qseq != 0 {
        return -1;
    }

    let mut ml = bam_aux_get(b, c"ML".as_ptr());
    if ml.is_null() {
        ml = bam_aux_get(b, c"Ml".as_ptr());
    }
    if !ml.is_null() && (*ml != b'B' || *ml.add(1) != b'C') {
        return -1;
    }
    let ml_end = if !ml.is_null() {
        let len = u32::from_le_bytes([*ml.add(2), *ml.add(3), *ml.add(4), *ml.add(5)]) as usize;
        ml.add(6 + len)
    } else {
        std::ptr::null_mut()
    };
    if !ml.is_null() {
        ml = ml.add(6);
    }

    let mut freq = [0; 16];
    seq_freq(b, freq.as_mut_ptr());

    let mut cp = mm.add(1).cast::<c_char>();
    let mut mod_num = 0usize;
    let mut failed = 0;
    while *cp != 0 {
        let mut btype = *cp as u8;
        cp = cp.add(1);
        if !matches!(btype, b'A' | b'C' | b'G' | b'T' | b'U' | b'N') {
            return -1;
        }
        if btype == b'U' {
            btype = b'T';
        }
        let btype = SEQ_NT16_TABLE[btype as usize] as c_int;

        if *cp != b'+' as c_char && *cp != b'-' as c_char {
            return -1;
        }
        let strand = *cp;
        cp = cp.add(1);

        let mut ms = cp;
        let mut chebi = 0;
        if libc::isdigit(*cp as u8 as c_int) != 0 {
            let mut cp_end: *mut c_char = std::ptr::null_mut();
            chebi = hts_str2uint(cp, &mut cp_end, 31, &mut failed) as c_int;
            if cp_end == cp || failed != 0 {
                return -1;
            }
            cp = cp_end;
            ms = cp.sub(1);
        } else {
            while *cp != 0 && libc::isalpha(*cp as u8 as c_int) != 0 {
                cp = cp.add(1);
            }
            if *cp == 0 {
                return -1;
            }
        }
        let me = cp;

        let implicit = if *cp == b'.' as c_char {
            cp = cp.add(1);
            1
        } else if *cp == b'?' as c_char {
            cp = cp.add(1);
            0
        } else if *cp == b',' as c_char || *cp == b';' as c_char {
            1
        } else {
            return -1;
        };

        let mut cp_end: *mut c_char = std::ptr::null_mut();
        let mut n = 0usize;
        let stride = me.offset_from(ms) as c_int;
        let mut ndelta = 0usize;
        let delta: c_int;

        if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
            let mut total_seq = 0i64;
            loop {
                if *cp == b',' as c_char {
                    cp = cp.add(1);
                }
                if *cp == 0 || *cp == b';' as c_char {
                    break;
                }
                let d = hts_str2uint(cp, &mut cp_end, 31, &mut failed) as i64;
                if cp_end == cp || failed != 0 {
                    return -1;
                }
                cp = cp_end;
                total_seq += d + 1;
                ndelta += 1;
            }
            delta = freq[SEQI_RC[btype as usize] as usize] - total_seq as c_int;
        } else if *cp == b',' as c_char {
            delta = hts_str2uint(cp.add(1), &mut cp_end, 31, &mut failed) as c_int;
            if cp_end == cp.add(1) || failed != 0 {
                return -1;
            }
        } else {
            delta = c_int::MAX;
            cp_end = cp;
        }

        let mut ms_iter = ms;
        while ms_iter < me {
            (*state).type_[mod_num] = if chebi != 0 {
                -chebi
            } else {
                *ms_iter as c_int
            };
            (*state).strand[mod_num] = (strand == b'-' as c_char) as c_char;
            (*state).canonical[mod_num] = btype;
            (*state).mlstride[mod_num] = stride;
            (*state).implicit[mod_num] = implicit;
            if delta < 0 {
                return -1;
            }
            (*state).mmcount[mod_num] = delta;
            if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
                (*state).mm[mod_num] = me.add(1);
                (*state).mmend[mod_num] = cp_end;
                (*state).ml[mod_num] = if !ml.is_null() {
                    ml.add(n)
                        .wrapping_offset((ndelta as isize - 1) * stride as isize)
                } else {
                    std::ptr::null_mut()
                };
            } else {
                (*state).mm[mod_num] = cp_end;
                (*state).mmend[mod_num] = std::ptr::null_mut();
                (*state).ml[mod_num] = if !ml.is_null() {
                    ml.add(n)
                } else {
                    std::ptr::null_mut()
                };
            }
            mod_num += 1;
            if mod_num >= MAX_BASE_MOD {
                return -1;
            }
            ms_iter = ms_iter.add(1);
            n += 1;
        }

        if !ml.is_null() {
            if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
                ml = ml.add(ndelta * stride as usize);
            } else {
                while *cp != 0 && *cp != b';' as c_char {
                    if *cp == b',' as c_char {
                        ml = ml.add(stride as usize);
                    }
                    cp = cp.add(1);
                }
            }
            if ml > ml_end {
                return -1;
            }
        } else if !cp_end.is_null() && ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
            cp = cp_end;
        } else {
            while *cp != 0 && *cp != b';' as c_char {
                cp = cp.add(1);
            }
        }

        if *cp == 0 {
            return -1;
        }
        cp = cp.add(1);
    }

    if !ml.is_null() && ml != ml_end {
        return -1;
    }
    (*state).nmods = mod_num as c_int;
    0
}

pub unsafe fn bam_parse_basemod(b: *const bam1_t, state: *mut hts_base_mod_state) -> c_int {
    bam_parse_basemod2(b, state, 0)
}

pub unsafe fn bam_mods_at_next_pos(
    b: *const bam1_t,
    state: *mut hts_base_mod_state,
    mods: *mut hts_base_mod,
    n_mods: c_int,
) -> c_int {
    if (*state).seq_pos >= (*b).core.l_qseq {
        return 0;
    }

    let mut n = 0;
    let mut base = bam_seqi(bam_get_seq(b), (*state).seq_pos as usize) as c_int;
    (*state).seq_pos += 1;
    if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
        base = SEQI_RC[base as usize];
    }

    let mut i = 0;
    while i < (*state).nmods {
        let idx = i as usize;
        let mut unchecked = 0;
        if (*state).canonical[idx] != base && (*state).canonical[idx] != 15 {
            i += 1;
            continue;
        }

        if (*state).mmcount[idx] > 0 {
            (*state).mmcount[idx] -= 1;
            if (*state).implicit[idx] == 0 && ((*state).flags & HTS_MOD_REPORT_UNCHECKED) != 0 {
                unchecked = 1;
            } else {
                i += 1;
                continue;
            }
        } else {
            (*state).mmcount[idx] -= 1;
        }

        let mmptr = (*state).mm[idx];
        if n < n_mods {
            let out = mods.add(n as usize);
            (*out).modified_base = (*state).type_[idx];
            (*out).canonical_base = SEQ_NT16_STR[(*state).canonical[idx] as usize] as c_int;
            (*out).strand = (*state).strand[idx] as c_int;
            (*out).qual = if unchecked != 0 {
                HTS_MOD_UNCHECKED
            } else if !(*state).ml[idx].is_null() {
                *(*state).ml[idx] as c_int
            } else {
                HTS_MOD_UNKNOWN
            };
        }
        n += 1;

        if unchecked != 0 {
            i += 1;
            continue;
        }

        if !(*state).ml[idx].is_null() {
            if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
                (*state).ml[idx] = (*state).ml[idx].sub((*state).mlstride[idx] as usize);
            } else {
                (*state).ml[idx] = (*state).ml[idx].add((*state).mlstride[idx] as usize);
            }
        }

        let mut failed = 0;
        if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
            if (*state).mmend[idx].sub(1) < (*state).mm[idx] {
                return -1;
            }
            let mut cp = (*state).mmend[idx].sub(1);
            while cp != (*state).mm[idx] {
                if *cp == b',' as c_char {
                    break;
                }
                cp = cp.sub(1);
            }
            (*state).mmend[idx] = cp;
            if cp != (*state).mm[idx] {
                let mut tmp: *mut c_char = std::ptr::null_mut();
                (*state).mmcount[idx] = hts_str2uint(cp.add(1), &mut tmp, 31, &mut failed) as c_int;
            } else {
                (*state).mmcount[idx] = c_int::MAX;
            }
        } else if *(*state).mm[idx] == b',' as c_char {
            let mut next = (*state).mm[idx];
            (*state).mmcount[idx] = hts_str2uint(next.add(1), &mut next, 31, &mut failed) as c_int;
            (*state).mm[idx] = next;
        } else {
            (*state).mmcount[idx] = c_int::MAX;
        }
        if failed != 0 {
            return -1;
        }

        let mut j = i + 1;
        while j < (*state).nmods && (*state).mm[j as usize] == mmptr {
            let jidx = j as usize;
            if n < n_mods {
                let out = mods.add(n as usize);
                (*out).modified_base = (*state).type_[jidx];
                (*out).canonical_base = SEQ_NT16_STR[(*state).canonical[jidx] as usize] as c_int;
                (*out).strand = (*state).strand[jidx] as c_int;
                (*out).qual = if !(*state).ml[jidx].is_null() {
                    *(*state).ml[jidx] as c_int
                } else {
                    -1
                };
            }
            n += 1;
            (*state).mmcount[jidx] = (*state).mmcount[idx];
            (*state).mm[jidx] = (*state).mm[idx];
            if !(*state).ml[jidx].is_null() {
                if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
                    (*state).ml[jidx] = (*state).ml[jidx].sub((*state).mlstride[jidx] as usize);
                } else {
                    (*state).ml[jidx] = (*state).ml[jidx].add((*state).mlstride[jidx] as usize);
                }
            }
            j += 1;
        }
        i = j;
    }

    n
}

pub unsafe fn bam_next_basemod(
    b: *const bam1_t,
    state: *mut hts_base_mod_state,
    mods: *mut hts_base_mod,
    n_mods: c_int,
    pos: *mut c_int,
) -> c_int {
    let mut next = [0x7f7f7f7f; 16];
    let mut freq = [0; 16];
    let unchecked = (*state).flags & HTS_MOD_REPORT_UNCHECKED;

    for i in 0..(*state).nmods {
        let mut base = (*state).canonical[i as usize];
        if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
            base = SEQI_RC[base as usize];
        }
        if unchecked != 0 && (*state).implicit[i as usize] == 0 {
            next[base as usize] = 0;
        } else if next[base as usize] > (*state).mmcount[i as usize] {
            next[base as usize] = (*state).mmcount[i as usize];
        }
    }

    let mut i = (*state).seq_pos;
    while i < (*b).core.l_qseq {
        let bc = bam_seqi(bam_get_seq(b), i as usize) as usize;
        if next[bc] <= freq[bc] || next[15] <= freq[15] {
            break;
        }
        freq[bc] += 1;
        if bc != 15 {
            freq[15] += 1;
        }
        i += 1;
    }
    *pos = i;
    (*state).seq_pos = i;

    if ((*b).core.flag as c_int & BAM_FREVERSE) != 0 {
        for j in 0..(*state).nmods {
            (*state).mmcount[j as usize] -=
                freq[SEQI_RC[(*state).canonical[j as usize] as usize] as usize];
        }
    } else {
        for j in 0..(*state).nmods {
            (*state).mmcount[j as usize] -= freq[(*state).canonical[j as usize] as usize];
        }
    }

    if (*b).core.l_qseq != 0 && (*state).seq_pos >= (*b).core.l_qseq {
        if ((*b).core.flag as c_int & BAM_FREVERSE) == 0 {
            for j in 0..(*state).nmods {
                let idx = j as usize;
                if (*state).mmcount[idx] < 0x7f000000
                    || (*(*state).mm[idx] != 0 && *(*state).mm[idx] != b';' as c_char)
                {
                    return -1;
                }
            }
        }
        return 0;
    }

    let r = bam_mods_at_next_pos(b, state, mods, n_mods);
    if r > 0 {
        r
    } else {
        0
    }
}

pub unsafe fn bam_mods_at_qpos(
    b: *const bam1_t,
    qpos: c_int,
    state: *mut hts_base_mod_state,
    mods: *mut hts_base_mod,
    n_mods: c_int,
) -> c_int {
    let mut r = 0;
    while (*state).seq_pos <= qpos {
        r = bam_mods_at_next_pos(b, state, mods, n_mods);
        if r < 0 {
            break;
        }
    }
    r
}

pub unsafe fn nibble2base_default(nib: *mut u8, seq: *mut c_char, len: c_int) {
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

    if len <= 0 {
        *seq = 0;
        return;
    }

    *seq = 0;
    let len2 = len / 2;
    let mut i = 0;
    while i < len2 {
        let idx = *nib.add(i as usize) as usize * 2;
        std::ptr::copy_nonoverlapping(
            CODE2BASE.as_ptr().add(idx).cast::<c_char>(),
            seq.add((i * 2) as usize),
            2,
        );
        i += 1;
    }

    i *= 2;
    if i < len {
        *seq.add(i as usize) = SEQ_NT16_STR[bam_seqi(nib, i as usize) as usize] as c_char;
    }
}

pub unsafe fn nibble2base(nib: *mut u8, seq: *mut c_char, len: c_int) {
    nibble2base_default(nib, seq, len);
}

pub unsafe fn sam_open_mode(mode: *mut c_char, fn_: *const c_char, format: *const c_char) -> c_int {
    if format.is_null() {
        let mut extension = [0 as c_char; HTS_MAX_EXT_LEN];
        if find_file_extension(fn_, extension.as_mut_ptr()) < 0 {
            return -1;
        }
        return sam_open_mode(mode, fn_, extension.as_ptr());
    } else if libc::strcasecmp(format, c"bam".as_ptr()) == 0 {
        libc::strcpy(mode, c"b".as_ptr());
    } else if libc::strcasecmp(format, c"cram".as_ptr()) == 0 {
        libc::strcpy(mode, c"c".as_ptr());
    } else if libc::strcasecmp(format, c"sam".as_ptr()) == 0 {
        libc::strcpy(mode, c"".as_ptr());
    } else if libc::strcasecmp(format, c"sam.gz".as_ptr()) == 0 {
        libc::strcpy(mode, c"z".as_ptr());
    } else if libc::strcasecmp(format, c"fastq".as_ptr()) == 0
        || libc::strcasecmp(format, c"fq".as_ptr()) == 0
    {
        libc::strcpy(mode, c"f".as_ptr());
    } else if libc::strcasecmp(format, c"fastq.gz".as_ptr()) == 0
        || libc::strcasecmp(format, c"fq.gz".as_ptr()) == 0
    {
        libc::strcpy(mode, c"fz".as_ptr());
    } else if libc::strcasecmp(format, c"fasta".as_ptr()) == 0
        || libc::strcasecmp(format, c"fa".as_ptr()) == 0
    {
        libc::strcpy(mode, c"F".as_ptr());
    } else if libc::strcasecmp(format, c"fasta.gz".as_ptr()) == 0
        || libc::strcasecmp(format, c"fa.gz".as_ptr()) == 0
    {
        libc::strcpy(mode, c"Fz".as_ptr());
    } else {
        return -1;
    }

    0
}

pub unsafe fn sam_open_mode_opts(
    fn_: *const c_char,
    mode: *const c_char,
    format: *const c_char,
) -> *mut c_char {
    let format_len_for_alloc = if format.is_null() {
        1
    } else {
        libc::strlen(format)
    };
    let mode_len_for_alloc = if mode.is_null() {
        1
    } else {
        libc::strlen(mode)
    };
    let mode_opts = crate::htslib_mini_rs::c_compat::malloc(
        (format_len_for_alloc + mode_len_for_alloc + 12) as u64,
    )
    .cast::<c_char>();

    if mode_opts.is_null() {
        return std::ptr::null_mut();
    }

    libc::strcpy(mode_opts, if mode.is_null() { c"r".as_ptr() } else { mode });
    let mut cp = mode_opts.add(libc::strlen(mode_opts));

    if format.is_null() {
        let mut extension = [0 as c_char; HTS_MAX_EXT_LEN];
        if find_file_extension(fn_, extension.as_mut_ptr()) < 0 {
            crate::htslib_mini_rs::c_compat::free(mode_opts.cast());
            return std::ptr::null_mut();
        }
        if sam_open_mode(cp, fn_, extension.as_ptr()) == 0 {
            return mode_opts;
        } else {
            crate::htslib_mini_rs::c_compat::free(mode_opts.cast());
            return std::ptr::null_mut();
        }
    }

    let opts = libc::strchr(format, b',' as c_int);
    let (opts, format_len) = if opts.is_null() {
        (c"".as_ptr(), libc::strlen(format))
    } else {
        (opts.cast_const(), opts.offset_from(format) as usize)
    };

    if libc::strncmp(format, c"bam".as_ptr(), format_len) == 0 {
        *cp = b'b' as c_char;
        cp = cp.add(1);
    } else if libc::strncmp(format, c"cram".as_ptr(), format_len) == 0 {
        *cp = b'c' as c_char;
        cp = cp.add(1);
    } else if libc::strncmp(format, c"cram2".as_ptr(), format_len) == 0 {
        *cp = b'c' as c_char;
        cp = cp.add(1);
        libc::strcpy(cp, c",VERSION=2.1".as_ptr());
        cp = cp.add(12);
    } else if libc::strncmp(format, c"cram3".as_ptr(), format_len) == 0 {
        *cp = b'c' as c_char;
        cp = cp.add(1);
        libc::strcpy(cp, c",VERSION=3.0".as_ptr());
        cp = cp.add(12);
    } else if libc::strncmp(format, c"sam".as_ptr(), format_len) == 0 {
    } else if libc::strncmp(format, c"sam.gz".as_ptr(), format_len) == 0 {
        *cp = b'z' as c_char;
        cp = cp.add(1);
    } else if libc::strncmp(format, c"fastq".as_ptr(), format_len) == 0
        || libc::strncmp(format, c"fq".as_ptr(), format_len) == 0
    {
        *cp = b'f' as c_char;
        cp = cp.add(1);
    } else if libc::strncmp(format, c"fastq.gz".as_ptr(), format_len) == 0
        || libc::strncmp(format, c"fq.gz".as_ptr(), format_len) == 0
    {
        *cp = b'f' as c_char;
        cp = cp.add(1);
        *cp = b'z' as c_char;
        cp = cp.add(1);
    } else if libc::strncmp(format, c"fasta".as_ptr(), format_len) == 0
        || libc::strncmp(format, c"fa".as_ptr(), format_len) == 0
    {
        *cp = b'F' as c_char;
        cp = cp.add(1);
    } else if libc::strncmp(format, c"fasta.gz".as_ptr(), format_len) == 0
        || libc::strncmp(format, c"fa".as_ptr(), format_len) == 0
    {
        *cp = b'F' as c_char;
        cp = cp.add(1);
        *cp = b'z' as c_char;
        cp = cp.add(1);
    } else {
        crate::htslib_mini_rs::c_compat::free(mode_opts.cast());
        return std::ptr::null_mut();
    }

    libc::strcpy(cp, opts);
    mode_opts
}

pub unsafe fn bam_str2flag(str_: *const c_char) -> c_int {
    let mut end: *mut c_char = std::ptr::null_mut();
    let numeric = libc::strtol(str_, &mut end, 0);
    if end != str_.cast_mut() {
        return numeric as c_int;
    }

    let mut flag = 0;
    let mut beg = str_;
    while *beg != 0 {
        let mut end = beg;
        while *end != 0 && *end != b',' as c_char {
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

pub unsafe fn bam_flag2str(flag: c_int) -> *mut c_char {
    let mut str_ = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
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
            if str_.l != 0 {
                kputsn(b",\0".as_ptr().cast(), 1, &mut str_);
            }
            kputsn(name.as_ptr().cast(), name.len(), &mut str_);
        }
    }
    if str_.l == 0 {
        kputsn(b"\0".as_ptr().cast(), 0, &mut str_);
    }
    str_.s
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
        assert_eq!(size_of::<bam1_t>(), 80);
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
        assert_eq!(size_of::<sam_hdr_t>(), 72);
        assert_eq!(align_of::<sam_hdr_t>(), 8);
        assert_eq!(size_of::<cstate_t>(), 24);
        assert_eq!(align_of::<cstate_t>(), 8);
        assert_eq!(size_of::<lbnode_t>(), 136);
        assert_eq!(align_of::<lbnode_t>(), 8);
        assert_eq!(size_of::<mempool_t>(), 24);
        assert_eq!(align_of::<mempool_t>(), 8);
        assert_eq!(size_of::<bam_plp_s>(), 128);
        assert_eq!(align_of::<bam_plp_s>(), 8);
        assert_eq!(size_of::<bam_mplp_s>(), 56);
        assert_eq!(align_of::<bam_mplp_s>(), 8);
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
        assert_eq!(std::mem::offset_of!(lbnode_t, beg), 80);
        assert_eq!(std::mem::offset_of!(lbnode_t, end), 88);
        assert_eq!(std::mem::offset_of!(lbnode_t, s), 96);
        assert_eq!(std::mem::offset_of!(lbnode_t, next), 120);
        assert_eq!(std::mem::offset_of!(lbnode_t, cd), 128);
        assert_eq!(std::mem::offset_of!(mempool_t, cnt), 0);
        assert_eq!(std::mem::offset_of!(mempool_t, n), 4);
        assert_eq!(std::mem::offset_of!(mempool_t, max), 8);
        assert_eq!(std::mem::offset_of!(mempool_t, buf), 16);
        assert_eq!(std::mem::offset_of!(bam_plp_s, mp), 0);
        assert_eq!(std::mem::offset_of!(bam_plp_s, head), 8);
        assert_eq!(std::mem::offset_of!(bam_plp_s, tail), 16);
        assert_eq!(std::mem::offset_of!(bam_plp_s, tid), 24);
        assert_eq!(std::mem::offset_of!(bam_plp_s, max_tid), 28);
        assert_eq!(std::mem::offset_of!(bam_plp_s, pos), 32);
        assert_eq!(std::mem::offset_of!(bam_plp_s, max_pos), 40);
        assert_eq!(std::mem::offset_of!(bam_plp_s, is_eof), 48);
        assert_eq!(std::mem::offset_of!(bam_plp_s, max_plp), 52);
        assert_eq!(std::mem::offset_of!(bam_plp_s, error), 56);
        assert_eq!(std::mem::offset_of!(bam_plp_s, maxcnt), 60);
        assert_eq!(std::mem::offset_of!(bam_plp_s, id), 64);
        assert_eq!(std::mem::offset_of!(bam_plp_s, plp), 72);
        assert_eq!(std::mem::offset_of!(bam_plp_s, b), 80);
        assert_eq!(std::mem::offset_of!(bam_plp_s, func), 88);
        assert_eq!(std::mem::offset_of!(bam_plp_s, data), 96);
        assert_eq!(std::mem::offset_of!(bam_plp_s, overlaps), 104);
        assert_eq!(std::mem::offset_of!(bam_plp_s, plp_construct), 112);
        assert_eq!(std::mem::offset_of!(bam_plp_s, plp_destruct), 120);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, n), 0);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, min_tid), 4);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, tid), 8);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, min_pos), 16);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, pos), 24);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, iter), 32);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, n_plp), 40);
        assert_eq!(std::mem::offset_of!(bam_mplp_s, plp), 48);
    }

    #[test]
    fn nibble2base_decodes_packed_bam_sequence() {
        unsafe {
            let mut packed = [0x12, 0x48, 0xf3, 0x50];
            let mut seq = [0 as c_char; 9];

            nibble2base_default(packed.as_mut_ptr(), seq.as_mut_ptr(), 7);
            assert_eq!(CStr::from_ptr(seq.as_ptr()).to_bytes(), b"ACGTNMR");

            seq.fill(0);
            nibble2base(packed.as_mut_ptr(), seq.as_mut_ptr(), 8);
            assert_eq!(CStr::from_ptr(seq.as_ptr()).to_bytes(), b"ACGTNMR=");

            seq[0] = b'X' as c_char;
            nibble2base_default(packed.as_mut_ptr(), seq.as_mut_ptr(), 0);
            assert_eq!(seq[0], 0);
        }
    }

    #[test]
    fn sam_open_mode_helpers_match_extension_and_option_rules() {
        unsafe {
            let mut mode = [0 as c_char; 8];

            assert_eq!(
                sam_open_mode(mode.as_mut_ptr(), c"reads.bam".as_ptr(), std::ptr::null()),
                0
            );
            assert_eq!(CStr::from_ptr(mode.as_ptr()).to_bytes(), b"b");

            assert_eq!(
                sam_open_mode(mode.as_mut_ptr(), c"out".as_ptr(), c"FASTQ.GZ".as_ptr()),
                0
            );
            assert_eq!(CStr::from_ptr(mode.as_ptr()).to_bytes(), b"fz");

            assert_eq!(
                sam_open_mode(mode.as_mut_ptr(), c"out".as_ptr(), c"unknown".as_ptr()),
                -1
            );

            let opts = sam_open_mode_opts(
                c"out.cram".as_ptr(),
                c"w".as_ptr(),
                c"cram3,seqs_per_slice=10".as_ptr(),
            );
            assert!(!opts.is_null());
            assert_eq!(
                CStr::from_ptr(opts).to_bytes(),
                b"wc,VERSION=3.0,seqs_per_slice=10"
            );
            crate::htslib_mini_rs::c_compat::free(opts.cast());

            let opts =
                sam_open_mode_opts(c"reads.sam.gz".as_ptr(), std::ptr::null(), std::ptr::null());
            assert!(!opts.is_null());
            assert_eq!(CStr::from_ptr(opts).to_bytes(), b"rz");
            crate::htslib_mini_rs::c_compat::free(opts.cast());

            assert!(
                sam_open_mode_opts(c"reads.bin".as_ptr(), c"r".as_ptr(), std::ptr::null())
                    .is_null()
            );
        }
    }

    #[test]
    fn base_mod_state_queries_and_seq_freq_match_c_rules() {
        unsafe {
            let state = hts_base_mod_state_alloc();
            assert!(!state.is_null());
            assert_eq!((*state).nmods, 0);

            (*state).nmods = 2;
            (*state).type_[0] = b'm' as c_int;
            (*state).type_[1] = -1234;
            (*state).strand[0] = b'+' as c_char;
            (*state).strand[1] = b'-' as c_char;
            (*state).implicit[0] = 1;
            (*state).implicit[1] = 0;
            (*state).canonical[0] = 2;
            (*state).canonical[1] = 15;

            let mut ntype = 0;
            let types = bam_mods_recorded(state, &mut ntype);
            assert_eq!(ntype, 2);
            assert_eq!(*types.add(0), b'm' as c_int);
            assert_eq!(*types.add(1), -1234);

            let mut strand = 0;
            let mut implicit = 0;
            let mut canonical = 0;
            assert_eq!(
                bam_mods_query_type(
                    state,
                    b'm' as c_int,
                    &mut strand,
                    &mut implicit,
                    &mut canonical,
                ),
                0
            );
            assert_eq!(strand, b'+' as c_int);
            assert_eq!(implicit, 1);
            assert_eq!(canonical, b'C' as c_char);

            assert_eq!(
                bam_mods_queryi(state, 1, &mut strand, &mut implicit, &mut canonical,),
                0
            );
            assert_eq!(strand, b'-' as c_int);
            assert_eq!(implicit, 0);
            assert_eq!(canonical, b'N' as c_char);
            assert_eq!(
                bam_mods_query_type(
                    state,
                    b'h' as c_int,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                -1
            );
            assert_eq!(
                bam_mods_queryi(
                    state,
                    2,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ),
                -1
            );
            hts_base_mod_state_free(state);

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"freq".as_ptr(),
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
                    c"ACGTN".as_ptr(),
                    std::ptr::null(),
                    0,
                ),
                20
            );
            let mut freq = [99; 16];
            seq_freq(b, freq.as_mut_ptr());
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
                    c"mods".as_ptr(),
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
                    c"ACAN".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let state = hts_base_mod_state_alloc();
            assert!(!state.is_null());
            let mut mm_end = *b";\0";
            let mut ml = [42u8];
            (*state).nmods = 1;
            (*state).type_[0] = b'm' as c_int;
            (*state).canonical[0] = 1;
            (*state).strand[0] = 0;
            (*state).mmcount[0] = 1;
            (*state).mm[0] = mm_end.as_mut_ptr().cast();
            (*state).ml[0] = ml.as_mut_ptr();
            (*state).mlstride[0] = 1;
            (*state).implicit[0] = 1;

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(b, state, mods.as_mut_ptr(), 1, &mut pos),
                1
            );
            assert_eq!(pos, 2);
            assert_eq!(mods[0].modified_base, b'm' as c_int);
            assert_eq!(mods[0].canonical_base, b'A' as c_int);
            assert_eq!(mods[0].strand, 0);
            assert_eq!(mods[0].qual, 42);
            assert_eq!(
                bam_next_basemod(b, state, mods.as_mut_ptr(), 1, &mut pos),
                0
            );

            (*state).seq_pos = 0;
            (*state).mmcount[0] = 1;
            (*state).mm[0] = mm_end.as_mut_ptr().cast();
            (*state).ml[0] = ml.as_mut_ptr();
            assert_eq!(bam_mods_at_qpos(b, 2, state, mods.as_mut_ptr(), 1), 1);
            assert_eq!(mods[0].qual, 42);

            (*state).seq_pos = 0;
            (*state).mmcount[0] = 1;
            (*state).mm[0] = mm_end.as_mut_ptr().cast();
            (*state).ml[0] = std::ptr::null_mut();
            (*state).implicit[0] = 0;
            (*state).flags = HTS_MOD_REPORT_UNCHECKED;
            assert_eq!(bam_mods_at_next_pos(b, state, mods.as_mut_ptr(), 1), 1);
            assert_eq!(mods[0].qual, HTS_MOD_UNCHECKED);

            hts_base_mod_state_free(state);
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
                    c"bmod".as_ptr(),
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
                    c"ACAN".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mm = b"A+m,1;\0";
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr(),
                    b'Z' as c_char,
                    mm.len() as c_int,
                    mm.as_ptr(),
                ),
                0
            );
            let ml = [b'C', 1, 0, 0, 0, 37];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"ML".as_ptr(),
                    b'B' as c_char,
                    ml.len() as c_int,
                    ml.as_ptr(),
                ),
                0
            );

            let state = hts_base_mod_state_alloc();
            assert_eq!(bam_parse_basemod(b, state), 0);
            assert_eq!((*state).nmods, 1);
            assert_eq!((*state).type_[0], b'm' as c_int);
            assert_eq!((*state).canonical[0], 1);
            assert_eq!((*state).mmcount[0], 1);
            assert_eq!((*state).mlstride[0], 1);
            assert_eq!(*(*state).ml[0], 37);

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(b, state, mods.as_mut_ptr(), 1, &mut pos),
                1
            );
            assert_eq!(pos, 2);
            assert_eq!(mods[0].modified_base, b'm' as c_int);
            assert_eq!(mods[0].qual, 37);

            hts_base_mod_state_free(state);
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
            let state = hts_base_mod_state_alloc();
            assert_eq!(bam_parse_basemod(b, state), 0);
            assert_eq!((*state).nmods, 0);
            hts_base_mod_state_free(state);
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
                    c"bmov".as_ptr(),
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
                    c"C".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mm = b"C+m,1;\0";
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr(),
                    b'Z' as c_char,
                    mm.len() as c_int,
                    mm.as_ptr(),
                ),
                0
            );

            let state = hts_base_mod_state_alloc();
            assert!(!state.is_null());
            assert_eq!(bam_parse_basemod(b, state), 0);
            assert_eq!((*state).nmods, 1);
            assert_eq!((*state).mmcount[0], 1);

            let mut mods = [hts_base_mod {
                modified_base: 0,
                canonical_base: 0,
                strand: 0,
                qual: 0,
            }];
            let mut pos = -1;
            assert_eq!(
                bam_next_basemod(b, state, mods.as_mut_ptr(), 1, &mut pos),
                -1
            );

            hts_base_mod_state_free(state);
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
                    c"bm256".as_ptr(),
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
                    c"C".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let mut mm = Vec::with_capacity(5 + MAX_BASE_MOD + 1);
            mm.extend_from_slice(b"C+");
            mm.extend(std::iter::repeat(b'm').take(MAX_BASE_MOD));
            mm.extend_from_slice(b",0;\0");
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr(),
                    b'Z' as c_char,
                    mm.len() as c_int,
                    mm.as_ptr(),
                ),
                0
            );

            let state = hts_base_mod_state_alloc();
            assert!(!state.is_null());
            assert_eq!(bam_parse_basemod(b, state), -1);

            hts_base_mod_state_free(state);
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
                    b'S' as c_char,
                    out.as_mut_ptr(),
                    input.as_ptr(),
                    input.len()
                ),
                0
            );
            assert_eq!(&out[..4], &[0x34, 0x12, 0x78, 0x56]);

            assert_eq!(
                aux_to_le(b'S' as c_char, out.as_mut_ptr(), input.as_ptr(), 3),
                -1
            );

            let z = b"text\0";
            out.fill(0);
            assert_eq!(
                aux_to_le(b'Z' as c_char, out.as_mut_ptr(), z.as_ptr(), z.len()),
                0
            );
            assert_eq!(&out[..z.len()], z);

            let b_array = [b's', 2, 0, 0, 0, 0x34, 0x12, 0x78, 0x56];
            let mut bout = [0; 9];
            assert_eq!(
                aux_to_le(
                    b'B' as c_char,
                    bout.as_mut_ptr(),
                    b_array.as_ptr(),
                    b_array.len(),
                ),
                0
            );
            assert_eq!(bout, b_array);

            assert_eq!(
                aux_to_le(b'B' as c_char, bout.as_mut_ptr(), b_array.as_ptr(), 4),
                -1
            );
            assert_eq!(
                aux_to_le(b'?' as c_char, bout.as_mut_ptr(), b_array.as_ptr(), 1),
                -1
            );
        }
    }

    #[test]
    fn cigar_parsers_fill_arrays_and_update_bam_records() {
        unsafe {
            assert_eq!(read_ncigar(c"10M2I3D\t".as_ptr()), 3);
            assert_eq!(read_ncigar(c"123\t".as_ptr()), 0);

            let mut cigar = [0u32; 4];
            assert_eq!(parse_cigar(c"10M2I3D".as_ptr(), cigar.as_mut_ptr(), 3), 7);
            assert_eq!(cigar[0], (10 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32);
            assert_eq!(cigar[1], (2 << BAM_CIGAR_SHIFT) | BAM_CINS as u32);
            assert_eq!(cigar[2], (3 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32);
            assert_eq!(parse_cigar(c"10Q".as_ptr(), cigar.as_mut_ptr(), 1), 0);

            let mut end: *mut c_char = std::ptr::null_mut();
            let mut a_cigar: *mut u32 = std::ptr::null_mut();
            let mut a_mem = 0usize;
            let input = c"4M1S\t";
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
            crate::htslib_mini_rs::c_compat::free(a_cigar.cast());

            let star = c"*";
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
                    c"cigr".as_ptr(),
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
                    c"ACGTN".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );

            let cigar_text = c"2M1I2M\t";
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
            data: data.as_mut_ptr(),
            l_data: data.len() as c_int,
            m_data: data.len() as u32,
            mempolicy_and_reserved: 0,
        };

        unsafe {
            let ref_match = CString::new("ACGT").unwrap();
            assert_eq!(sam_cap_mapq(&mut b, ref_match.as_ptr(), 4, -1), 40);

            let ref_mismatch = CString::new("ATGT").unwrap();
            assert_eq!(sam_cap_mapq(&mut b, ref_mismatch.as_ptr(), 4, 40), 28);
            assert_eq!(sam_cap_mapq(&mut b, ref_mismatch.as_ptr(), 4, 10), -1);

            let good_tag = b"Zabcd\0";
            assert_eq!(
                realn_check_tag(good_tag.as_ptr(), 3, c"BQ".as_ptr(), &b as *const bam1_t),
                0
            );
            let bad_type = b"iabcd\0";
            assert_eq!(
                realn_check_tag(bad_type.as_ptr(), 3, c"BQ".as_ptr(), &b as *const bam1_t),
                -1
            );
            let bad_len = b"Zabc\0";
            assert_eq!(
                realn_check_tag(bad_len.as_ptr(), 3, c"BQ".as_ptr(), &b as *const bam1_t),
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
            let qual = [30 as c_char, 30 as c_char, 30 as c_char, 30 as c_char];
            assert_eq!(
                bam_set1(
                    b,
                    5,
                    c"read".as_ptr(),
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
                    c"ACGT".as_ptr(),
                    qual.as_ptr(),
                    0,
                ),
                18
            );

            assert_eq!(sam_prob_realn(b, c"ACGT".as_ptr(), 4, 0), 0);
            let bq = bam_aux_get(b, c"BQ".as_ptr());
            assert!(!bq.is_null());
            assert_eq!(*bq, b'Z');
            assert_eq!(libc::strlen(bq.add(1).cast()), 4);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_prob_realn_trims_reference_window_like_htslib_for_deletion_heavy_reads() {
        unsafe fn make_realn_record() -> *mut bam1_t {
            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [
                (5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
                (10u32 << BAM_CIGAR_SHIFT) | BAM_CDEL as u32,
                (5u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32,
            ];
            let qual = [35 as c_char; 10];
            assert!(
                bam_set1(
                    b,
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
                    qual.as_ptr(),
                    0,
                ) > 0
            );
            b
        }

        unsafe {
            let ref_seq = c"TTGCAACGTACGTTACGATCGTACCTAGGCTAATCGGATCCGTAACGTTAGCTA";
            let rust_b = make_realn_record();
            let c_b = make_realn_record();

            let rust_ret = sam_prob_realn(rust_b, ref_seq.as_ptr(), 60, 0);
            let c_ret = hts_sys::sam_prob_realn(c_b.cast(), ref_seq.as_ptr(), 60, 0);
            assert_eq!(rust_ret, c_ret);
            assert_eq!((*rust_b).l_data, (*c_b).l_data);
            assert_eq!(
                std::slice::from_raw_parts((*rust_b).data, (*rust_b).l_data as usize),
                std::slice::from_raw_parts((*c_b).data, (*c_b).l_data as usize)
            );

            bam_destroy1(rust_b);
            bam_destroy1(c_b);
        }
    }

    #[test]
    fn bam_data_accessors_match_htslib_macros() {
        let mut data = vec![0u8; 4 + 8 + 3 + 5 + 7];
        let cigar_offset = 4usize;
        let seq_offset = cigar_offset + 8;
        let qual_offset = seq_offset + 3;
        let aux_offset = qual_offset + 5;
        data[cigar_offset..cigar_offset + 4].copy_from_slice(&((10u32 << 4) | 0).to_ne_bytes());
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
            data: data.as_mut_ptr(),
            l_data: data.len() as c_int,
            m_data: data.len() as u32,
            mempolicy_and_reserved: 0,
        };

        unsafe {
            assert_eq!(bam_get_qname(&b), data.as_mut_ptr().cast::<c_char>());
            assert!(!bam_is_rev(&b));
            assert!(!bam_is_mrev(&b));
            assert_eq!(
                bam_get_cigar(&b).cast::<u8>(),
                data.as_ptr().add(cigar_offset)
            );
            assert_eq!(bam_get_seq(&b), data.as_ptr().add(seq_offset));
            assert_eq!(bam_get_qual(&b), data.as_ptr().add(qual_offset));
            assert_eq!(bam_get_aux(&b), data.as_ptr().add(aux_offset));
            assert_eq!(bam_get_l_aux(&b), 7);
            let nm = bam_aux_get(&b, b"NM".as_ptr().cast());
            assert_eq!(nm, data.as_mut_ptr().add(aux_offset + 2));
            assert_eq!(bam_aux_first(&b), nm);
            assert!(bam_aux_next(&b, nm).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ENOENT as c_int
            );
            assert_eq!(
                bam_aux_tag(bam_get_aux(&b).add(2)),
                data.as_ptr().add(aux_offset).cast::<c_char>()
            );
            assert_eq!(bam_aux_type(nm), b'i' as c_char);
            assert_eq!(bam_aux2i(nm), 5);
            assert_eq!(bam_aux2f(nm), 5.0);
            assert_eq!(bam_aux2A(nm), 0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );
            assert_eq!(*bam_get_cigar(&b), (10u32 << 4) | 0);
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
                qname.as_ptr(),
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
                seq.as_ptr(),
                qual.as_ptr().cast(),
                0,
            );
            assert_eq!(ret, 5 + 3 + 4 + 3 + 5);
            assert_eq!((*b).l_data, ret);
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
            assert_eq!(std::slice::from_raw_parts((*b).data, 8), b"read1\0\0\0");
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
                CString::new("AAA").unwrap().as_ptr(),
                std::ptr::null(),
                0,
            );
            assert_eq!(ret, 1 + 3 + 0 + 2 + 3);
            assert_eq!(std::slice::from_raw_parts((*b).data, 4), b"*\0\0\0");
            assert_eq!(std::slice::from_raw_parts(bam_get_qual(b), 3), &[0xff; 3]);

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
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            let bad = bam_set1(
                b,
                1,
                c"r".as_ptr(),
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
                c"A".as_ptr(),
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
                    c"r1".as_ptr(),
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
                    seq.as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ),
                14
            );
            let old_cigar = *bam_get_cigar(b);
            let old_seq0 = bam_seqi(bam_get_seq(b), 0);

            assert_eq!(bam_set_qname(b, c"longer_name".as_ptr()), 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b)).to_bytes(), b"longer_name");
            assert_eq!((*b).core.l_qname as usize % 4, 0);
            assert_eq!((*b).core.l_extranul, 0);
            assert_eq!(*bam_get_cigar(b), old_cigar);
            assert_eq!(bam_seqi(bam_get_seq(b), 0), old_seq0);
            assert_eq!(*bam_get_qual(b), 11);

            assert_eq!(bam_set_qname(b, c"x".as_ptr()), 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b)).to_bytes(), b"x");
            assert_eq!((*b).core.l_qname, 4);
            assert_eq!((*b).core.l_extranul, 2);
            assert_eq!(*bam_get_cigar(b), old_cigar);
            assert_eq!(bam_seqi(bam_get_seq(b), 0), old_seq0);

            assert_eq!(bam_set_qname(b, std::ptr::null()), -1);
            assert_eq!(bam_set_qname(std::ptr::null_mut(), c"x".as_ptr()), -1);
            bam_destroy1(b);
        }
    }

    #[test]
    fn bam_write1_round_trips_record_through_translated_reader() {
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-bam-write1-{}-{}.bam",
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
                c"read".as_ptr(),
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
                seq.as_ptr(),
                qual.as_ptr().cast(),
                0,
            );
            assert_eq!(ret, 18);
            assert_eq!(
                bam_aux_append(b, c"CB".as_ptr(), b'Z' as c_char, 6, b"cell\0".as_ptr()),
                0
            );

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let header_text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:1000\n@SQ\tSN:chr2\tLN:2000\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, header_text.as_ptr().cast(), header_text.len()),
                0
            );

            let fpw = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!fpw.is_null());
            assert_eq!(bam_hdr_write(fpw, hdr), 0);
            let written = bam_write1(fpw, b);
            assert!(written > 0);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(fpw), 0);

            let fpr = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fpr.is_null());
            let read_hdr = bam_hdr_read(fpr);
            assert!(!read_hdr.is_null());
            assert_eq!(sam_hdr_name2tid(read_hdr, c"chr1".as_ptr()), 0);
            assert_eq!(sam_hdr_name2tid(read_hdr, c"chr2".as_ptr()), 1);
            let read = bam_init1();
            assert_eq!(bam_read1(fpr, read), written);
            assert_eq!((*read).core.tid, (*b).core.tid);
            assert_eq!((*read).core.pos, (*b).core.pos);
            assert_eq!((*read).core.bin, (*b).core.bin);
            assert_eq!((*read).core.qual, (*b).core.qual);
            assert_eq!((*read).core.flag, (*b).core.flag);
            assert_eq!((*read).core.n_cigar, (*b).core.n_cigar);
            assert_eq!((*read).core.l_qseq, (*b).core.l_qseq);
            assert_eq!(CStr::from_ptr(bam_get_qname(read)).to_bytes(), b"read");
            assert_eq!(*bam_get_cigar(read), *bam_get_cigar(b));
            let cb = bam_aux_get(read, c"CB".as_ptr());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb)).to_bytes(), b"cell");
            assert_eq!(bam_read1(fpr, read), -1);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(fpr), 0);

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
                sam_hdr_add_lines(hdr, header_text.as_ptr().cast(), header_text.len()),
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
                    c"read".as_ptr(),
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
                    seq.as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert_eq!(
                bam_aux_append(b, c"CB".as_ptr(), b'Z' as c_char, 6, b"cell\0".as_ptr()),
                0
            );
            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            let len = sam_format1(hdr, b, &mut ks);
            assert!(len > 0);
            assert_eq!(
                CStr::from_ptr(ks.s).to_bytes(),
                b"read\t0\tchr1\t10\t50\t4M\t=\t30\t-7\tACGT\t@ABC\tCB:Z:cell"
            );

            crate::htslib_mini_rs::c_compat::free(ks.s.cast());
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
                sam_hdr_add_lines(hdr, header_text.as_ptr().cast(), header_text.len()),
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
                    c"read".as_ptr(),
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
                    c"AACGT".as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ) > 0
            );
            let nm = 3i32.to_le_bytes();
            assert_eq!(
                bam_aux_append(
                    b,
                    c"NM".as_ptr(),
                    b'i' as c_char,
                    nm.len() as c_int,
                    nm.as_ptr()
                ),
                0
            );
            assert_eq!(
                bam_aux_append(b, c"CB".as_ptr(), b'Z' as c_char, 5, b"cell\0".as_ptr()),
                0
            );

            let mut hb = hb_pair { h: hdr, b };
            let mut res = hts_expr_val_t {
                is_str: 0,
                is_true: 0,
                s: kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                },
                d: 0.0,
            };
            let mut end: *mut c_char = std::ptr::null_mut();

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
            assert_eq!(CStr::from_ptr(res.s.s).to_bytes(), b"read");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"cigar".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(CStr::from_ptr(res.s.s).to_bytes(), b"1S4M2H");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"rname".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(CStr::from_ptr(res.s.s).to_bytes(), b"chr1");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"mrname".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(CStr::from_ptr(res.s.s).to_bytes(), b"chr2");

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"seq".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                0
            );
            assert_eq!(CStr::from_ptr(res.s.s).to_bytes(), b"AACGT");

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
            assert_eq!(CStr::from_ptr(res.s.s).to_bytes(), b"cell");

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
            assert_eq!(res.s.l, 0);

            assert_eq!(
                sam_c_1210_bam_sym_lookup(
                    (&mut hb as *mut hb_pair).cast(),
                    c"flag.bad".as_ptr().cast_mut(),
                    &mut end,
                    &mut res,
                ),
                -1
            );

            let filt = crate::htslib_mini_rs::hts::hts_filter_init(
                c"mapq >= 50 && rname == \"chr1\" && [NM] == 3".as_ptr(),
            );
            assert!(!filt.is_null());
            assert_eq!(sam_c_1535_sam_passes_filter(hdr, b, filt.cast()), 1);
            crate::htslib_mini_rs::hts::hts_filter_free(filt);

            let filt = crate::htslib_mini_rs::hts::hts_filter_init(c"flag.read2 || [ZZ]".as_ptr());
            assert!(!filt.is_null());
            assert_eq!(sam_c_1535_sam_passes_filter(hdr, b, filt.cast()), 0);
            crate::htslib_mini_rs::hts::hts_filter_free(filt);

            crate::htslib_mini_rs::c_compat::free(res.s.s.cast());
            bam_destroy1(b);
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn fastq_state_init_and_destroy_match_default_state() {
        unsafe {
            let x = sam_c_3786_fastq_state_init(b'@' as c_int);
            assert!(!x.is_null());
            assert_eq!((*x).BC, [b'B' as c_char, b'C' as c_char, 0]);
            assert_eq!((*x).nprefix, b'@' as c_char);
            assert_eq!((*x).casava, 0);
            assert_eq!((*x).aux, 0);
            assert_eq!((*x).rnum, 0);
            assert_eq!((*x).sra_names, 0);
            assert!((*x).tags.is_null());
            assert!((*x).name.s.is_null());
            assert!((*x).seq.s.is_null());
            assert!((*x).qual.s.is_null());

            let mut match_ = libc::regmatch_t { rm_so: 0, rm_eo: 0 };
            let name = c"INST:RUN:FLOW:1:1101:1000:1000:ACGT";
            assert_eq!(
                libc::regexec(&mut (*x).regex, name.as_ptr(), 1, &mut match_, 0,),
                0
            );

            let mut fp: htsFile = std::mem::zeroed();
            fp.state = x.cast();
            sam_c_3802_fastq_state_destroy(&mut fp);
            assert!(fp.state.is_null());
            sam_c_3802_fastq_state_destroy(&mut fp);
        }
    }

    #[test]
    fn fastq_state_set_updates_options_and_aux_tag_whitelist() {
        unsafe {
            let mut fp: htsFile = std::mem::zeroed();
            fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_CASAVA, std::ptr::null()),
                0
            );
            assert!(!fp.state.is_null());
            let x = fp.state.cast::<fastq_state>();
            assert_eq!((*x).nprefix, b'@' as c_char);
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
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_BARCODE, c"CR".as_ptr()),
                0
            );
            assert_eq!((*x).BC, [b'C' as c_char, b'R' as c_char, 0]);

            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_UMI, c"RX,MI".as_ptr()),
                0
            );
            assert_eq!((*x).UMI[0], [b'R' as c_char, b'X' as c_char, 0]);
            assert_eq!((*x).UMI[1], [b'M' as c_char, b'I' as c_char, 0]);
            assert_eq!((*x).UMI[2], [0, 0, 0]);

            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_AUX, c"NM,CB".as_ptr()),
                0
            );
            assert_eq!((*x).aux, 1);
            let tags = (*x).tags.cast::<khash_tag_t>();
            assert!(!tags.is_null());
            for tcode in [
                b'N' as c_int * 256 + b'M' as c_int,
                b'C' as c_int * 256 + b'B' as c_int,
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
                    c"^([^:]+):([A-Z]+)$".as_ptr(),
                ),
                0
            );
            let mut matches: [libc::regmatch_t; 3] = std::mem::zeroed();
            assert_eq!(
                libc::regexec(
                    &(*x).regex,
                    c"READ:ACGT".as_ptr(),
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
            "htslib-mini-rs-fastq-parse-{}-{}.fq",
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
            let fp = crate::htslib_mini_rs::hts::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            assert_eq!((*fp).format.format, HTS_FORMAT_FASTQ_FORMAT);

            assert_eq!(
                sam_c_3815_fastq_state_set(fp, FASTQ_OPT_CASAVA, std::ptr::null()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(fp, FASTQ_OPT_AUX, c"NM,CB".as_ptr()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(fp, FASTQ_OPT_UMI, c"RX".as_ptr()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(
                    fp,
                    FASTQ_OPT_UMI_REGEX,
                    c"^[^:]+:([A-Za-z]+)$".as_ptr(),
                ),
                0
            );

            let b = bam_init1();
            assert!(!b.is_null());
            assert!(sam_read1(fp, std::ptr::null_mut(), b) >= 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b)).to_bytes(), b"READ");
            assert_eq!((*b).core.l_qseq, 4);
            assert_ne!((*b).core.flag as c_int & BAM_FREAD1, 0);
            assert_ne!((*b).core.flag as c_int & BAM_FQCFAIL, 0);

            let rx = bam_aux_get(b, c"RX".as_ptr());
            assert!(!rx.is_null());
            assert_eq!(CStr::from_ptr(rx.add(1).cast()).to_bytes(), b"ACGT");
            let bc = bam_aux_get(b, c"BC".as_ptr());
            assert!(!bc.is_null());
            assert_eq!(CStr::from_ptr(bc.add(1).cast()).to_bytes(), b"ACGT");

            assert!(sam_read1(fp, std::ptr::null_mut(), b) >= 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b)).to_bytes(), b"READ2");
            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 7);
            let cb = bam_aux_get(b, c"CB".as_ptr());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(cb.add(1).cast()).to_bytes(), b"cellA");

            assert_eq!(sam_read1(fp, std::ptr::null_mut(), b), -1);
            bam_destroy1(b);
            sam_c_3802_fastq_state_destroy(fp);
            (*fp).state = std::ptr::null_mut();
            assert_eq!(crate::htslib_mini_rs::hts::hts_close(fp), 0);
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fastq_format1_renders_options_and_sam_write1_writes_bgzf_fastq() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let qual = [40 as c_char; 4];
            assert!(
                bam_set1(
                    b,
                    4,
                    c"read".as_ptr(),
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
                    c"ACGT".as_ptr(),
                    qual.as_ptr(),
                    0,
                ) >= 0
            );
            let nm = [7u8];
            assert_eq!(
                bam_aux_append(b, c"NM".as_ptr(), b'C' as c_char, 1, nm.as_ptr()),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"RX".as_ptr(),
                    b'Z' as c_char,
                    5,
                    c"ACGT".as_ptr().cast()
                ),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"BC".as_ptr(),
                    b'Z' as c_char,
                    5,
                    c"ACGT".as_ptr().cast()
                ),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    c"CB".as_ptr(),
                    b'Z' as c_char,
                    5,
                    c"cell".as_ptr().cast()
                ),
                0
            );

            let mut fp: htsFile = std::mem::zeroed();
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
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_UMI, c"RX".as_ptr()),
                0
            );
            assert_eq!(
                sam_c_3815_fastq_state_set(&mut fp, FASTQ_OPT_AUX, c"NM,CB".as_ptr()),
                0
            );
            let x = fp.state.cast::<fastq_state>();
            let mut out: kstring_t = std::mem::zeroed();
            let expected = b"@read:ACGT/1 1:Y:0:ACGT\tNM:i:7\tCB:Z:cell\nACGT\n+\nIIII\n";
            assert_eq!(
                sam_c_4413_fastq_format1(x, b, &mut out),
                expected.len() as c_int
            );
            assert_eq!(
                std::slice::from_raw_parts(out.s.cast::<u8>(), out.l),
                expected
            );
            ks_free(&mut out);

            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-fastq-write-{}-{}.fq.gz",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut out_fp: htsFile = std::mem::zeroed();
            out_fp.bitfields = 1 << 4;
            out_fp.fp.bgzf = bgzf;
            out_fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            out_fp.state = fp.state;

            assert_eq!(
                sam_c_4553_sam_write1(&mut out_fp, std::ptr::null(), b),
                expected.len() as c_int
            );
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);
            sam_c_3802_fastq_state_destroy(&mut out_fp);

            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let mut buf = vec![0u8; expected.len()];
            assert_eq!(
                bgzf_read(bgzf, buf.as_mut_ptr().cast(), expected.len()),
                expected.len() as isize
            );
            assert_eq!(buf, expected);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);

            bam_destroy1(b);
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn sam_hdr_write_writes_sam_text_and_stores_header_copy() {
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-sam-hdr-write-{}-{}.sam.gz",
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
                sam_hdr_add_lines(hdr, header_text.as_ptr().cast(), header_text.len()),
                0
            );

            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut fp: htsFile = std::mem::zeroed();
            fp.bitfields = 1 << 4;
            fp.fp.bgzf = bgzf;
            fp.format.format = HTS_FORMAT_SAM;
            fp.format.category = HTS_FORMAT_SEQUENCE_DATA;

            assert_eq!(sam_hdr_write(&mut fp, hdr), 0);
            assert!(!fp.bam_header.is_null());
            assert_ne!(fp.bam_header, hdr.cast::<c_void>());
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);

            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let mut out = [0u8; 64];
            let n = bgzf_read(bgzf, out.as_mut_ptr().cast(), header_text.len());
            assert_eq!(n, header_text.len() as isize);
            assert_eq!(&out[..header_text.len()], header_text);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);

            sam_hdr_destroy(fp.bam_header.cast());
            sam_hdr_destroy(hdr);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sam_index_build_wrappers_report_missing_inputs_like_htslib() {
        unsafe {
            let missing = CString::new(format!(
                "/tmp/htslib-mini-rs-missing-index-input-{}-{}.bam",
                std::process::id(),
                line!()
            ))
            .unwrap();

            assert!(sam_index_build3(missing.as_ptr(), std::ptr::null(), 14, 0) < 0);
            assert!(sam_index_build2(missing.as_ptr(), std::ptr::null(), 14) < 0);
            assert!(sam_index_build(missing.as_ptr(), 14) < 0);
            assert!(bam_index_build(missing.as_ptr(), 14) < 0);
        }
    }

    #[test]
    fn sam_index_builds_index_from_generated_bam_stream() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-sam-index-{}-{}.bam",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let text = b"@SQ\tSN:chr1\tLN:100\n";
            (*hdr).text = crate::htslib_mini_rs::c_compat::malloc(text.len() as u64 + 1).cast();
            assert!(!(*hdr).text.is_null());
            crate::htslib_mini_rs::c_compat::memcpy(
                (*hdr).text.cast(),
                text.as_ptr().cast(),
                text.len() as u64,
            );
            *(*hdr).text.add(text.len()) = 0;
            (*hdr).l_text = text.len();
            (*hdr).n_targets = 1;
            (*hdr).target_len =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<u32>() as u64)
                    .cast();
            (*hdr).target_name = crate::htslib_mini_rs::c_compat::calloc(
                1,
                std::mem::size_of::<*mut c_char>() as u64,
            )
            .cast();
            assert!(!(*hdr).target_len.is_null());
            assert!(!(*hdr).target_name.is_null());
            *(*hdr).target_len = 100;
            *(*hdr).target_name = crate::htslib_mini_rs::c_compat::strdup(c"chr1".as_ptr());
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
                    c"read1".as_ptr(),
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
                    seq.as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert!(bam_write1(bgzf, b) > 0);
            bam_destroy1(b);
            sam_hdr_destroy(hdr);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);

            let fp = crate::htslib_mini_rs::hts::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let idx = sam_c_994_sam_index(fp, 0);
            assert!(!idx.is_null());
            assert_eq!((*idx).fmt, HTS_FMT_BAI);
            hts_idx_destroy(idx);
            assert_eq!(crate::htslib_mini_rs::hts::hts_close(fp), 0);
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn sam_index_build_public_wrappers_create_default_and_custom_indexes() {
        unsafe fn write_indexable_bam(path: &std::path::Path) -> CString {
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let text = b"@SQ\tSN:chr1\tLN:100\n";
            (*hdr).text = crate::htslib_mini_rs::c_compat::malloc(text.len() as u64 + 1).cast();
            assert!(!(*hdr).text.is_null());
            crate::htslib_mini_rs::c_compat::memcpy(
                (*hdr).text.cast(),
                text.as_ptr().cast(),
                text.len() as u64,
            );
            *(*hdr).text.add(text.len()) = 0;
            (*hdr).l_text = text.len();
            (*hdr).n_targets = 1;
            (*hdr).target_len =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<u32>() as u64)
                    .cast();
            (*hdr).target_name = crate::htslib_mini_rs::c_compat::calloc(
                1,
                std::mem::size_of::<*mut c_char>() as u64,
            )
            .cast();
            assert!(!(*hdr).target_len.is_null());
            assert!(!(*hdr).target_name.is_null());
            *(*hdr).target_len = 100;
            *(*hdr).target_name = crate::htslib_mini_rs::c_compat::strdup(c"chr1".as_ptr());
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
                    c"read1".as_ptr(),
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
                    seq.as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert!(bam_write1(bgzf, b) > 0);
            bam_destroy1(b);
            sam_hdr_destroy(hdr);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);
            path_c
        }

        let base = std::env::temp_dir();
        let path_build3 = base.join(format!(
            "htslib-mini-rs-sam-index-build3-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_build2 = base.join(format!(
            "htslib-mini-rs-sam-index-build2-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_build = base.join(format!(
            "htslib-mini-rs-sam-index-build-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let path_bam = base.join(format!(
            "htslib-mini-rs-bam-index-build-{}-{}.bam",
            std::process::id(),
            line!()
        ));
        let idx_build3 = base.join(format!(
            "htslib-mini-rs-sam-index-build3-{}-{}.bai",
            std::process::id(),
            line!()
        ));
        let idx_build2 = base.join(format!(
            "htslib-mini-rs-sam-index-build2-{}-{}.bai",
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
                sam_index_build3(c_build3.as_ptr(), c_idx_build3.as_ptr(), 0, 0),
                0
            );
            assert_eq!(
                sam_index_build2(c_build2.as_ptr(), c_idx_build2.as_ptr(), 0),
                0
            );
            assert_eq!(sam_index_build(c_build.as_ptr(), 0), 0);
            assert_eq!(bam_index_build(c_bam.as_ptr(), 0), 0);
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

            let mut bgzf: BGZF = std::mem::zeroed();
            bgzf.block_address = 0x1234;
            bgzf.block_offset = 0x5678;
            assert_eq!(
                sam_c_1638_bam_ptell((&mut bgzf as *mut BGZF).cast::<c_void>()),
                0x12345678
            );

            let missing = CString::new(format!(
                "/tmp/htslib-mini-rs-missing-index-load-{}-{}.bam",
                std::process::id(),
                line!()
            ))
            .unwrap();
            let mut fp: htsFile = std::mem::zeroed();
            fp.format.format = HTS_FORMAT_BAM;
            assert!(sam_c_1649_index_load(
                &mut fp,
                missing.as_ptr(),
                std::ptr::null(),
                HTS_IDX_SAVE_REMOTE,
            )
            .is_null());

            fp.format.format = HTS_FORMAT_FASTQ_FORMAT;
            assert!(sam_c_1649_index_load(
                &mut fp,
                missing.as_ptr(),
                std::ptr::null(),
                HTS_IDX_SAVE_REMOTE,
            )
            .is_null());
        }
    }

    #[test]
    fn sam_index_load_public_wrappers_reject_non_indexable_formats() {
        unsafe {
            let mut fp: htsFile = std::mem::zeroed();
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
            let mut fp: htsFile = std::mem::zeroed();
            fp.format.format = HTS_FORMAT_SAM;
            fp.format.compression = crate::htslib_mini_rs::hts::HTS_COMPRESSION_NO_COMPRESSION;

            let fd = sam_c_3048_sam_state_create(&mut fp);
            assert!(!fd.is_null());
            assert_eq!(fp.state, fd.cast());
            assert_eq!((*fd).fp, &mut fp as *mut htsFile);

            sam_c_3069_sam_state_err(fd, 5);
            sam_c_3069_sam_state_err(fd, 7);
            assert_eq!((*fd).errcode, 5);

            let lines =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_lines>() as u64)
                    .cast::<sp_lines>();
            assert!(!lines.is_null());
            (*lines).data = crate::htslib_mini_rs::c_compat::malloc(8).cast();
            (*fd).lines = lines;

            let curr =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_bams>() as u64)
                    .cast::<sp_bams>();
            assert!(!curr.is_null());
            (*curr).abams = 1;
            (*curr).bams =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<bam1_t>() as u64)
                    .cast::<bam1_t>();
            assert!(!(*curr).bams.is_null());
            (*(*curr).bams).data = crate::htslib_mini_rs::c_compat::malloc(4).cast();
            (*fd).curr_bam = curr;

            assert_eq!(sam_state_destroy(&mut fp), -5);
            assert!(fp.state.is_null());
            assert_eq!(sam_state_destroy(&mut fp), 0);

            let mut bam_fp: htsFile = std::mem::zeroed();
            bam_fp.format.format = HTS_FORMAT_BAM;
            assert!(sam_c_3048_sam_state_create(&mut bam_fp).is_null());
            assert!(bam_fp.state.is_null());
        }
    }

    #[test]
    fn sam_worker_cleanup_callbacks_free_owned_blocks() {
        unsafe {
            let lines =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_lines>() as u64)
                    .cast::<sp_lines>();
            assert!(!lines.is_null());
            (*lines).data = crate::htslib_mini_rs::c_compat::malloc(16).cast();
            assert!(!(*lines).data.is_null());

            let nested =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_bams>() as u64)
                    .cast::<sp_bams>();
            assert!(!nested.is_null());
            (*nested).abams = 1;
            (*nested).bams =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<bam1_t>() as u64)
                    .cast::<bam1_t>();
            assert!(!(*nested).bams.is_null());
            (*(*nested).bams).data = crate::htslib_mini_rs::c_compat::malloc(8).cast();
            assert!(!(*(*nested).bams).data.is_null());
            (*lines).bams = nested;

            sam_c_3200_cleanup_sp_lines(lines.cast());
            sam_c_3200_cleanup_sp_lines(std::ptr::null_mut());

            let bams =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_bams>() as u64)
                    .cast::<sp_bams>();
            assert!(!bams.is_null());
            (*bams).abams = 1;
            (*bams).bams =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<bam1_t>() as u64)
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
            let mut target_names = [chr1.as_ptr() as *mut c_char];
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
            let mut fp: htsFile = std::mem::zeroed();
            let mut fd: SAM_state = std::mem::zeroed();
            fd.h = &mut hdr;
            fd.fp = &mut fp;

            let text = b"read1\t0\tchr1\t1\t60\t4M\t*\t0\t0\tACGT\t!!!!\tNM:i:1\nread2\t4\t*\t0\t0\t*\t*\t0\t0\t*\t*\tZZ:Z:tag\n";
            let gl =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sp_lines>() as u64)
                    .cast::<sp_lines>();
            assert!(!gl.is_null());
            (*gl).alloc = text.len() as c_int + 8;
            (*gl).data_size = text.len() as c_int;
            (*gl).data = crate::htslib_mini_rs::c_compat::malloc((*gl).alloc as u64).cast();
            assert!(!(*gl).data.is_null());
            crate::htslib_mini_rs::c_compat::memcpy(
                (*gl).data.cast(),
                text.as_ptr().cast(),
                text.len() as u64,
            );
            (*gl).fd = &mut fd;
            (*gl).serial = 17;

            let gb = sam_c_3215_sam_parse_worker(gl.cast()).cast::<sp_bams>();
            assert!(!gb.is_null());
            assert_eq!((*gb).serial, 17);
            assert_eq!((*gb).nbams, 2);
            assert!(!fd.lines.is_null());
            assert_eq!(
                CStr::from_ptr(bam_get_qname((*gb).bams)).to_bytes(),
                b"read1"
            );
            assert_eq!(
                CStr::from_ptr(bam_get_qname((*gb).bams.add(1))).to_bytes(),
                b"read2"
            );

            (*gb).fd = &mut fd;
            let out = sam_c_3652_sam_format_worker(gb.cast()).cast::<sp_lines>();
            assert!(!out.is_null());
            assert_eq!((*out).serial, 17);
            let formatted =
                std::slice::from_raw_parts((*out).data.cast::<u8>(), (*out).data_size as usize);
            assert!(formatted.starts_with(b"read1\t0\tchr1\t1\t60\t4M"));
            assert!(formatted.ends_with(b"ZZ:Z:tag\n"));
            assert!(!fd.bams.is_null());

            sam_c_3200_cleanup_sp_lines(out.cast());
            sam_c_3076_sam_free_sp_bams(fd.bams);
        }
    }

    #[test]
    fn bam_record_allocation_copy_and_destroy_match_htslib_ownership_rules() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            assert_eq!((*b).l_data, 0);
            assert_eq!((*b).m_data, 0);
            assert!(bam_get_mempolicy(b) == 0);
            bam_destroy1(b);
        }

        let mut src_data = vec![1u8, 2, 3, 4, 5, 6];
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
            data: src_data.as_mut_ptr(),
            l_data: src_data.len() as c_int,
            m_data: src_data.len() as u32,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let dst = bam_dup1(&src);
            assert!(!dst.is_null());
            assert_ne!((*dst).data, src.data);
            assert_eq!((*dst).core.pos, 42);
            assert_eq!((*dst).core.tid, 7);
            assert_eq!((*dst).core.flag, BAM_FREVERSE as u16);
            assert_eq!((*dst).id, 99);
            assert_eq!((*dst).l_data, src_data.len() as c_int);
            assert_eq!(
                std::slice::from_raw_parts((*dst).data, src_data.len()),
                src_data.as_slice()
            );
            bam_destroy1(dst);
        }

        let mut external = vec![9u8, 8, 7];
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
            data: external.as_mut_ptr(),
            l_data: external.len() as c_int,
            m_data: external.len() as u32,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            assert_eq!(realloc_bam_data(&mut owned_struct, 12), 0);
            assert_ne!(owned_struct.data, external.as_mut_ptr());
            assert_eq!(bam_get_mempolicy(&mut owned_struct) & BAM_USER_OWNS_DATA, 0);
            assert_eq!(
                std::slice::from_raw_parts(owned_struct.data, external.len()),
                external.as_slice()
            );
            bam_destroy1(&mut owned_struct);
            assert!(owned_struct.data.is_null());
            assert_eq!(owned_struct.l_data, 0);
            assert_eq!(owned_struct.m_data, 0);
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
                    b'i' as c_char,
                    nm_val.len() as c_int,
                    nm_val.as_ptr(),
                ),
                0
            );
            let nm = bam_aux_get(b, b"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux_type(nm), b'i' as c_char);
            assert_eq!(bam_aux2i(nm), 5);
            assert_eq!((*b).l_data, 7);

            assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), -3), 0);
            let nm = bam_aux_get(b, b"NM".as_ptr().cast());
            assert!(!nm.is_null());
            assert_eq!(bam_aux_type(nm), b'i' as c_char);
            assert_eq!(bam_aux2i(nm), -3);

            assert_eq!(
                bam_aux_update_str(b, b"CB".as_ptr().cast(), 3, b"abc".as_ptr().cast()),
                0
            );
            let cb = bam_aux_get(b, b"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(bam_aux_type(cb), b'Z' as c_char);
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb)).to_bytes(), b"abc");

            assert_eq!(
                bam_aux_update_str(b, b"CB".as_ptr().cast(), -1, b"xy\0".as_ptr().cast()),
                0
            );
            let cb = bam_aux_get(b, b"CB".as_ptr().cast());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb)).to_bytes(), b"xy");

            assert_eq!(bam_aux_update_float(b, b"FZ".as_ptr().cast(), 1.5), 0);
            let fz = bam_aux_get(b, b"FZ".as_ptr().cast());
            assert!(!fz.is_null());
            assert_eq!(bam_aux_type(fz), b'f' as c_char);
            assert!((bam_aux2f(fz) - 1.5).abs() < 1e-6);

            let d2 = 2.5f64.to_le_bytes();
            assert_eq!(
                bam_aux_append(
                    b,
                    b"D2".as_ptr().cast(),
                    b'd' as c_char,
                    d2.len() as c_int,
                    d2.as_ptr(),
                ),
                0
            );
            assert_eq!(bam_aux_update_float(b, b"D2".as_ptr().cast(), 3.25), 0);
            let d2 = bam_aux_get(b, b"D2".as_ptr().cast());
            assert!(!d2.is_null());
            assert_eq!(bam_aux_type(d2), b'f' as c_char);
            assert!((bam_aux2f(d2) - 3.25).abs() < 1e-6);

            let arr = [10u16, 20u16];
            assert_eq!(
                bam_aux_update_array(
                    b,
                    b"XA".as_ptr().cast(),
                    b'S',
                    arr.len() as u32,
                    arr.as_ptr().cast_mut().cast::<c_void>(),
                ),
                0
            );
            let xa = bam_aux_get(b, b"XA".as_ptr().cast());
            assert!(!xa.is_null());
            assert_eq!(bam_aux_type(xa), b'B' as c_char);
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
                    shrunk.as_ptr().cast_mut().cast::<c_void>(),
                ),
                0
            );
            let xa = bam_aux_get(b, b"XA".as_ptr().cast());
            assert!(!xa.is_null());
            assert_eq!(bam_auxB_len(xa), 1);
            assert_eq!(bam_auxB2i(xa, 0), 7);

            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(
                bam_aux_get_str(b, b"NM".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l)).unwrap(),
                "NM:i:-3"
            );
            ks.l = 0;
            assert_eq!(
                bam_aux_get_str(b, b"CB".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l)).unwrap(),
                "CB:Z:xy"
            );
            ks.l = 0;
            assert_eq!(
                bam_aux_get_str(b, b"FZ".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l)).unwrap(),
                "FZ:f:1.5"
            );
            ks.l = 0;
            assert_eq!(
                bam_aux_get_str(b, b"XA".as_ptr().cast(), &mut ks as *mut kstring_t),
                1
            );
            assert_eq!(
                std::str::from_utf8(std::slice::from_raw_parts(ks.s.cast::<u8>(), ks.l)).unwrap(),
                "XA:B:C,7"
            );
            ks.l = 0;
            assert_eq!(
                bam_aux_get_str(b, b"ZZ".as_ptr().cast(), &mut ks as *mut kstring_t),
                0
            );
            crate::htslib_mini_rs::c_compat::free(ks.s.cast());

            let cb = bam_aux_get(b, b"CB".as_ptr().cast());
            let next = bam_aux_remove(b, cb);
            assert!(!next.is_null());
            assert!(bam_aux_get(b, b"CB".as_ptr().cast()).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ENOENT as c_int
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
                bam_aux_append(b, b"AA".as_ptr().cast(), b'C' as c_char, 1, aa.as_ptr()),
                0
            );
            let nm = [7u8];
            assert_eq!(
                bam_aux_append(b, b"NM".as_ptr().cast(), b'C' as c_char, 1, nm.as_ptr()),
                0
            );
            assert_eq!(
                bam_aux_append(
                    b,
                    b"ZZ".as_ptr().cast(),
                    b'Z' as c_char,
                    5,
                    b"tail\0".as_ptr(),
                ),
                0
            );

            let old_len = (*b).l_data;
            assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), 70_000), 0);
            assert_eq!((*b).l_data, old_len + 3);

            let first = bam_aux_first(b);
            assert!(!first.is_null());
            assert_eq!(
                std::slice::from_raw_parts(bam_aux_tag(first).cast::<u8>(), 2),
                b"AA"
            );
            assert_eq!(bam_aux_type(first), b'C' as c_char);
            assert_eq!(bam_aux2i(first), 1);

            let middle = bam_aux_next(b, first);
            assert!(!middle.is_null());
            assert_eq!(
                std::slice::from_raw_parts(bam_aux_tag(middle).cast::<u8>(), 2),
                b"NM"
            );
            assert_eq!(bam_aux_type(middle), b'I' as c_char);
            assert_eq!(bam_aux2i(middle), 70_000);

            let last = bam_aux_next(b, middle);
            assert!(!last.is_null());
            assert_eq!(
                std::slice::from_raw_parts(bam_aux_tag(last).cast::<u8>(), 2),
                b"ZZ"
            );
            assert_eq!(bam_aux_type(last), b'Z' as c_char);
            assert_eq!(CStr::from_ptr(bam_aux2Z(last)).to_bytes(), b"tail");
            assert!(bam_aux_next(b, last).is_null());

            assert_eq!(bam_aux_update_int(b, b"NM".as_ptr().cast(), 8), 0);
            let middle = bam_aux_get(b, b"NM".as_ptr().cast());
            assert!(!middle.is_null());
            assert_eq!(bam_aux_type(middle), b'I' as c_char);
            assert_eq!(bam_aux2i(middle), 8);
            let last = bam_aux_get(b, b"ZZ".as_ptr().cast());
            assert!(!last.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(last)).to_bytes(), b"tail");

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
                assert_eq!(bam_aux_type(nm), expected_type as c_char, "value {value}");
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
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int
            );
            assert_eq!(
                bam_aux_update_int(b, b"NM".as_ptr().cast(), u32::MAX as i64 + 1),
                -1
            );
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int
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
            data: data.as_mut_ptr(),
            l_data: data.len() as c_int,
            m_data: data.len() as u32,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let first = bam_aux_first(&b);
            assert!(!first.is_null());
            assert_eq!(bam_aux_type(first), b'B' as c_char);
            assert!(bam_aux_next(&b, first).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            assert!(bam_aux_get(&b, c"XY".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
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
                    c"read".as_ptr(),
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
                    seq.as_ptr(),
                    std::ptr::null(),
                    0,
                ),
                17
            );
            assert_eq!(bam_get_l_aux(b), 0);

            assert_eq!(bam_aux_update_int(b, c"NM".as_ptr(), 300), 0);
            assert_eq!(bam_get_l_aux(b), 5);
            assert_eq!(
                bam_aux_update_str(b, c"CB".as_ptr(), -1, c"cell".as_ptr()),
                0
            );
            assert_eq!(bam_get_l_aux(b), 13);

            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert_eq!(bam_aux_del(b, nm), 0);
            assert_eq!(bam_get_l_aux(b), 8);
            let cb = bam_aux_get(b, c"CB".as_ptr());
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
                bam_aux_update_str(b, c"CB".as_ptr(), -1, c"cell".as_ptr()),
                0
            );
            let cb = bam_aux_get(b, c"CB".as_ptr());
            assert!(!cb.is_null());
            assert_eq!(bam_aux2A(cb), 0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );
            assert_eq!(bam_auxB_len(cb), 0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            let values = [1u8, 2u8];
            assert_eq!(
                bam_aux_update_array(
                    b,
                    c"XA".as_ptr(),
                    b'C',
                    values.len() as u32,
                    values.as_ptr().cast_mut().cast::<c_void>(),
                ),
                0
            );
            let xa = bam_aux_get(b, c"XA".as_ptr());
            assert!(!xa.is_null());
            assert_eq!(bam_auxB2i(xa, 2), 0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ERANGE as c_int
            );
            assert_eq!(bam_auxB2f(xa, 2), 0.0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ERANGE as c_int
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

            let mp = mp_init();
            assert!(!mp.is_null());
            assert_eq!((*mp).cnt, 0);
            assert_eq!((*mp).n, 0);
            assert_eq!((*mp).max, 0);
            assert!((*mp).buf.is_null());

            let node = mp_alloc(mp);
            assert!(!node.is_null());
            assert_eq!((*mp).cnt, 1);
            assert_eq!((*node).b.l_data, 0);
            assert_eq!((*node).beg, 0);
            assert_eq!((*node).s.k, 0);
            (*node).next = node;
            (*node).b.data = crate::htslib_mini_rs::c_compat::malloc(4).cast();
            (*node).b.m_data = 4;

            mp_free(mp, node);
            assert_eq!((*mp).cnt, 0);
            assert_eq!((*mp).n, 1);
            assert_eq!((*mp).max, 256);
            assert!((*node).next.is_null());

            let reused = mp_alloc(mp);
            assert_eq!(reused, node);
            assert_eq!((*mp).cnt, 1);
            assert_eq!((*mp).n, 0);

            mp_free(mp, reused);
            mp_destroy(mp);
        }
    }

    unsafe extern "C" fn test_plp_auto_callback(_data: *mut c_void, _b: *mut bam1_t) -> c_int {
        -1
    }

    #[test]
    fn bam_plp_init_and_destroy_match_htslib_initial_state() {
        unsafe {
            let iter = bam_plp_init(None, std::ptr::null_mut());
            assert!(!iter.is_null());
            assert!(!(*iter).mp.is_null());
            assert_eq!((*iter).head, (*iter).tail);
            assert!(!(*iter).head.is_null());
            assert_eq!((*(*iter).mp).cnt, 1);
            assert_eq!((*iter).max_tid, -1);
            assert_eq!((*iter).max_pos, -1);
            assert_eq!((*iter).maxcnt, 8000);
            assert!((*iter).func.is_none());
            assert!((*iter).data.is_null());
            assert!((*iter).b.is_null());
            bam_plp_destroy(iter);

            let mut data = 7u8;
            let iter = bam_plp_init(
                Some(test_plp_auto_callback),
                (&mut data as *mut u8).cast::<c_void>(),
            );
            assert_eq!(
                (*iter).func.map(|f| f as usize),
                Some(test_plp_auto_callback as usize)
            );
            assert_eq!((*iter).data, (&mut data as *mut u8).cast::<c_void>());
            assert!(!(*iter).b.is_null());
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
            data: data.as_mut_ptr(),
            l_data: data.len() as c_int,
            m_data: data.len() as u32,
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
            data: left_data.as_mut_ptr(),
            l_data: left_data.len() as c_int,
            m_data: left_data.len() as u32,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };
        let right = bam1_t {
            core,
            id: 0,
            data: right_data.as_mut_ptr(),
            l_data: right_data.len() as c_int,
            m_data: right_data.len() as u32,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            let iter = bam_plp_init(None, std::ptr::null_mut());
            assert_eq!(bam_plp_init_overlaps(iter), 0);
            assert_eq!(bam_plp_push(iter, &left), 0);
            assert_eq!(bam_plp_push(iter, &right), 0);

            let left_node = (*iter).head;
            let right_node = (*left_node).next;
            assert!(!left_node.is_null());
            assert!(!right_node.is_null());
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
            data: data.as_mut_ptr(),
            l_data: data.len() as c_int,
            m_data: data.len() as u32,
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
        let mut fp = crate::htslib_mini_rs::hts::htsFile {
            bitfields: 0,
            padding_0: 0,
            lineno: 0,
            line: crate::htslib_mini_rs::hts::kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: crate::htslib_mini_rs::hts::htsFilePtr {
                bgzf: std::ptr::null_mut(),
            },
            state: std::ptr::null_mut(),
            format: crate::htslib_mini_rs::hts::htsFormat {
                category: 0,
                format: 0,
                version: crate::htslib_mini_rs::hts::htsFormatVersion { major: 0, minor: 0 },
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
            data: std::ptr::null_mut(),
            l_data: 0,
            m_data: 0,
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

            let mut fp: htsFile = std::mem::zeroed();
            fp.state = std::ptr::dangling_mut();
            assert_eq!(
                sam_c_3719_sam_set_thread_pool(
                    &mut fp,
                    (&mut hts_sys::htsThreadPool {
                        pool: std::ptr::null_mut(),
                        qsize: 0,
                    }) as *mut hts_sys::htsThreadPool,
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
        let mut target_names = [chr1.as_ptr() as *mut c_char, chr2.as_ptr() as *mut c_char];
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
            assert_eq!(sam_hdr_name2tid(&mut hdr, chr1.as_ptr().cast()), 0);
            assert_eq!(bam_name2id(&mut hdr, chr1.as_ptr().cast()), 0);
            assert_eq!(sam_hdr_name2tid(&mut hdr, chr2.as_ptr().cast()), 1);
            assert_eq!(sam_hdr_tid2name(&hdr, -1), std::ptr::null());
            assert_eq!(sam_hdr_tid2name(&hdr, 0), chr1.as_ptr().cast());
            assert_eq!(sam_hdr_tid2name(&hdr, 1), chr2.as_ptr().cast());
            assert_eq!(sam_hdr_tid2name(&hdr, 2), std::ptr::null());
            assert_eq!(sam_hdr_tid2len(&hdr, -1), 0);
            assert_eq!(sam_hdr_tid2len(&hdr, 0), 100);
            assert_eq!(sam_hdr_tid2len(&hdr, 1), 200);
            assert_eq!(sam_hdr_tid2len(&hdr, 2), 0);
            assert_eq!(sam_hdr_name2tid(&mut hdr, b"missing\0".as_ptr().cast()), -1);

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let rest = sam_parse_region(
                &mut hdr,
                c"chr2:11-20".as_ptr(),
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
        let mut ref_hash_keys = [alt.as_ptr() as *mut c_char];
        let mut ref_hash_vals = [0 as c_int];
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
            assert_eq!(sam_hdr_name2tid(&mut hdr, alt.as_ptr().cast()), 0);
            assert_eq!(sam_hdr_tid2name(&hdr, 0), alt.as_ptr().cast());
            assert_eq!(sam_hdr_tid2len(&hdr, 0), 999);
            assert_eq!(sam_hdr_tid2name(&hdr, 1), chr2.as_ptr().cast());
            assert_eq!(sam_hdr_tid2len(&hdr, 1), 200);
        }

        hdr.hrecs = std::ptr::null_mut();
        let mut long_target_lens = [u32::MAX, 200u32];
        hdr.target_len = long_target_lens.as_mut_ptr();
        let long_len = (u32::MAX as i64) + 10;
        let mut sdict_keys = [chr1.as_ptr() as *mut c_char];
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
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sam_hdr_t>() as u64)
                    .cast::<sam_hdr_t>();
            assert!(!hdr.is_null());
            (*hdr).n_targets = 1;
            (*hdr).target_name = crate::htslib_mini_rs::c_compat::calloc(
                1,
                std::mem::size_of::<*mut c_char>() as u64,
            )
            .cast();
            (*hdr).target_len =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<u32>() as u64)
                    .cast();
            (*hdr).text = crate::htslib_mini_rs::c_compat::malloc(5).cast();
            let name = crate::htslib_mini_rs::c_compat::malloc(5).cast::<c_char>();
            assert!(!(*hdr).target_name.is_null());
            assert!(!(*hdr).target_len.is_null());
            assert!(!(*hdr).text.is_null());
            assert!(!name.is_null());
            std::ptr::copy_nonoverlapping(b"chr1\0".as_ptr().cast::<c_char>(), name, 5);
            std::ptr::copy_nonoverlapping(b"@HD\n\0".as_ptr().cast::<c_char>(), (*hdr).text, 5);
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
            let mut src_keys = [std::ptr::null_mut::<c_char>(); 4];
            let mut src_vals = [0i64; 4];
            let chr1_bucket = __ac_FNV1a_hash_string(chr1.as_ptr()) & 3;
            src_keys[chr1_bucket as usize] = chr1.as_ptr() as *mut c_char;
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

            let mut h0 = sam_hdr_t {
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
                chr1.as_ptr() as *mut c_char,
                missing.as_ptr() as *mut c_char,
                chr2.as_ptr() as *mut c_char,
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

            assert_eq!(sam_c_144_sam_hdr_dup_sdict(&mut h0, &mut h), 0);
            let dest = h.sdict.cast::<khash_s2i_t>();
            assert!(!dest.is_null());
            assert_eq!((*dest).size, 1);
            let k = kh_get_s2i(dest, chr1.as_ptr());
            assert_ne!(k, (*dest).n_buckets);
            assert_eq!(*(*dest).vals.add(k as usize), (u32::MAX as i64) + 42);
            assert_eq!(kh_get_s2i(dest, missing.as_ptr()), (*dest).n_buckets);
            assert_eq!(kh_get_s2i(dest, chr2.as_ptr()), (*dest).n_buckets);
            kh_destroy_s2i(dest);
        }
    }

    #[test]
    fn sam_hdr_dup_copies_simple_long_reference_dictionary() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            let mut target_len = [u32::MAX, 200u32];
            let mut target_name = [chr1.as_ptr() as *mut c_char, chr2.as_ptr() as *mut c_char];
            let mut flags = [0xaaaa_aaaau32];
            let mut keys = [std::ptr::null_mut::<c_char>(); 4];
            let mut vals = [0i64; 4];
            let bucket = __ac_FNV1a_hash_string(chr1.as_ptr()) & 3;
            keys[bucket as usize] = chr1.as_ptr() as *mut c_char;
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
            let mut h0 = sam_hdr_t {
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

            let dup = sam_hdr_dup(&mut h0);
            assert!(!dup.is_null());
            assert_ne!((*dup).target_name, h0.target_name);
            assert_ne!(*(*dup).target_name, *h0.target_name);
            assert_eq!(sam_hdr_tid2len(dup, 0), (u32::MAX as hts_pos_t) + 99);
            assert_eq!(sam_hdr_tid2len(dup, 1), 200);
            sam_hdr_destroy(dup);
        }
    }

    #[test]
    fn sam_hdr_change_hd_updates_text_header_like_htslib() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            let text = b"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(sam_hdr_add_lines(hdr, text.as_ptr().cast(), text.len()), 0);

            assert_eq!(
                sam_hdr_change_HD(hdr, c"SO".as_ptr(), c"unsorted".as_ptr()),
                0
            );
            assert_eq!(
                CStr::from_ptr((*hdr).text).to_bytes(),
                b"@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chr1\tLN:10\n"
            );

            assert_eq!(sam_hdr_change_HD(hdr, c"GO".as_ptr(), c"query".as_ptr()), 0);
            assert_eq!(
                CStr::from_ptr((*hdr).text).to_bytes(),
                b"@HD\tVN:1.6\tSO:unsorted\tGO:query\n@SQ\tSN:chr1\tLN:10\n"
            );

            assert_eq!(sam_hdr_change_HD(hdr, c"SO".as_ptr(), std::ptr::null()), 0);
            assert_eq!(
                CStr::from_ptr((*hdr).text).to_bytes(),
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
                sam_hdr_add_lines(hdr, b"@SQ\tSN:chr1\tLN:10\n".as_ptr().cast(), 18),
                0
            );
            assert_eq!(
                sam_hdr_change_HD(hdr, c"SO".as_ptr(), c"coordinate".as_ptr()),
                0
            );
            assert_eq!(
                CStr::from_ptr((*hdr).text).to_bytes(),
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
                sam_hdr_add_lines(hdr, b"@SQ\tSN:chr1\tLN:10\n".as_ptr().cast(), 18),
                0
            );
            let mut fp = crate::htslib_mini_rs::hts::htsFile {
                bitfields: 0,
                padding_0: 0,
                lineno: 0,
                line: crate::htslib_mini_rs::hts::kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                },
                fn_: std::ptr::null_mut(),
                fn_aux: std::ptr::null_mut(),
                fp: crate::htslib_mini_rs::hts::htsFilePtr {
                    bgzf: std::ptr::null_mut(),
                },
                state: std::ptr::null_mut(),
                format: crate::htslib_mini_rs::hts::htsFormat {
                    category: 0,
                    format: 0,
                    version: crate::htslib_mini_rs::hts::htsFormatVersion { major: 0, minor: 0 },
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
            assert_eq!(CStr::from_ptr((*dup).text), CStr::from_ptr((*hdr).text));

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
                sam_hdr_add_lines(hdr, header_text.as_ptr().cast(), header_text.len()),
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
                    c"read".as_ptr(),
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
                    seq.as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );
            assert_eq!(
                bam_aux_append(b, c"RG".as_ptr(), b'Z' as c_char, 4, b"rg1\0".as_ptr()),
                0
            );

            let library = sam_c_1173_bam_get_library(hdr, b);
            assert!(!library.is_null());
            assert_eq!(CStr::from_ptr(library).to_bytes(), b"lib_a");

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
            let mut end: *mut c_char = std::ptr::null_mut();
            assert_eq!(
                sam_c_2490_sam_parse_B_vals(b'C' as c_char, input.as_ptr().cast_mut(), &mut end, b,),
                0
            );
            assert_eq!(*(*b).data, b'B');
            assert_eq!(*(*b).data.add(1), b'C');
            assert_eq!(bam_auxB_len((*b).data), 3);
            assert_eq!(bam_auxB2i((*b).data, 0), 1);
            assert_eq!(bam_auxB2i((*b).data, 2), 255);
            assert_eq!(*end, 0);

            (*b).l_data = 0;
            let input = CString::new(",-1,2").unwrap();
            assert_eq!(
                sam_c_2490_sam_parse_B_vals(b'C' as c_char, input.as_ptr().cast_mut(), &mut end, b,),
                0
            );
            assert_eq!(*(*b).data, b'B');
            assert_eq!(*(*b).data.add(1), b'c');
            assert_eq!(bam_auxB_len((*b).data), 2);
            assert_eq!(bam_auxB2i((*b).data, 0), -1);
            assert_eq!(bam_auxB2i((*b).data, 1), 2);
            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_parse_b_vals_writes_float_arrays() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());
            let input = CString::new(",1.5,-2.25").unwrap();
            let mut end: *mut c_char = std::ptr::null_mut();
            assert_eq!(
                sam_c_2490_sam_parse_B_vals(b'f' as c_char, input.as_ptr().cast_mut(), &mut end, b,),
                0
            );
            assert_eq!(*(*b).data, b'B');
            assert_eq!(*(*b).data.add(1), b'f');
            assert_eq!(bam_auxB_len((*b).data), 2);
            assert_eq!(bam_auxB2f((*b).data, 0), 1.5);
            assert_eq!(bam_auxB2f((*b).data, 1), -2.25);
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
                    input.as_ptr().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast_mut(),
                    b,
                    0,
                    std::ptr::null_mut(),
                ),
                0
            );

            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert!(!nm.is_null());
            assert_eq!(*nm, b'C');
            assert_eq!(bam_aux2i(nm), 7);
            let as_ = bam_aux_get(b, c"AS".as_ptr());
            assert!(!as_.is_null());
            assert_eq!(*as_, b'c');
            assert_eq!(bam_aux2i(as_), -3);
            let cb = bam_aux_get(b, c"CB".as_ptr());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb)).to_bytes(), b"cell-1");
            let hx = bam_aux_get(b, c"HX".as_ptr());
            assert!(!hx.is_null());
            assert_eq!(*hx, b'H');
            assert_eq!(CStr::from_ptr(bam_aux2Z(hx)).to_bytes(), b"0a0B");
            let xa = bam_aux_get(b, c"XA".as_ptr());
            assert!(!xa.is_null());
            assert_eq!(bam_aux2A(xa), b'z' as c_char);
            let bf = bam_aux_get(b, c"BF".as_ptr());
            assert!(!bf.is_null());
            assert_eq!(bam_auxB_len(bf), 2);
            assert_eq!(bam_auxB2f(bf, 0), 1.5);
            assert_eq!(bam_auxB2f(bf, 1), -2.0);

            (*b).l_data = 0;
            let input = CString::new("bad\tNM:i:8").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    input.as_ptr().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast_mut(),
                    b,
                    1,
                    std::ptr::null_mut(),
                ),
                0
            );
            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 8);

            (*b).l_data = 0;
            let input = CString::new("NM:i:9\tCB:Z:drop").unwrap();
            let mut flags = [0xaaaa_aaaau32];
            let mut keys = [0i32; 4];
            let bucket = __ac_Wang_hash((b'N' as c_int * 256 + b'M' as c_int) as u32) & 3;
            keys[bucket as usize] = b'N' as c_int * 256 + b'M' as c_int;
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
                    input.as_ptr().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast_mut(),
                    b,
                    0,
                    (&mut tags as *mut khash_tag_t).cast(),
                ),
                0
            );
            assert!(!bam_aux_get(b, c"NM".as_ptr()).is_null());
            assert!(bam_aux_get(b, c"CB".as_ptr()).is_null());

            let input = CString::new("HX:H:abc").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    input.as_ptr().cast_mut(),
                    input.as_ptr().add(input.as_bytes().len()).cast_mut(),
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
                    strict.as_ptr().cast_mut(),
                    strict.as_ptr().add(strict.as_bytes().len()).cast_mut(),
                    b,
                    0,
                    std::ptr::null_mut(),
                ),
                -2
            );

            (*b).l_data = 0;
            let lenient = CString::new("NM:i:1\tbad\tAS:i:2").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    lenient.as_ptr().cast_mut(),
                    lenient.as_ptr().add(lenient.as_bytes().len()).cast_mut(),
                    b,
                    1,
                    std::ptr::null_mut(),
                ),
                0
            );
            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 1);
            let as_ = bam_aux_get(b, c"AS".as_ptr());
            assert!(!as_.is_null());
            assert_eq!(bam_aux2i(as_), 2);

            (*b).l_data = 0;
            let odd_hex = CString::new("HX:H:abc\tNM:i:3").unwrap();
            assert_eq!(
                sam_c_2524_aux_parse(
                    odd_hex.as_ptr().cast_mut(),
                    odd_hex.as_ptr().add(odd_hex.as_bytes().len()).cast_mut(),
                    b,
                    1,
                    std::ptr::null_mut(),
                ),
                0
            );
            assert!(bam_aux_get(b, c"HX".as_ptr()).is_null());
            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 3);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_parse1_builds_bam_record_with_core_sequence_quality_and_aux() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut c_char];
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

            let line = b"read1\t0\tchr1\t2\t60\t4M\t*\t0\t0\tACGT\t!!!!\tNM:i:1\tCB:Z:cell\0";
            let mut buf = line.to_vec();
            let mut ks = kstring_t {
                l: line.len() - 1,
                m: line.len(),
                s: buf.as_mut_ptr().cast(),
            };
            assert_eq!(sam_c_2662_sam_parse1(&mut ks, &mut hdr, b), 0);

            assert_eq!(CStr::from_ptr(bam_get_qname(b)).to_bytes(), b"read1");
            assert_eq!((*b).core.flag as c_int & BAM_FUNMAP, 0);
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
            let nm = bam_aux_get(b, c"NM".as_ptr());
            assert!(!nm.is_null());
            assert_eq!(bam_aux2i(nm), 1);
            let cb = bam_aux_get(b, c"CB".as_ptr());
            assert!(!cb.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(cb)).to_bytes(), b"cell");

            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(sam_format1(&hdr, b, &mut out) > 0);
            assert!(CStr::from_ptr(out.s)
                .to_bytes()
                .starts_with(b"read1\t0\tchr1\t2\t60\t4M"));
            ks_free(&mut out);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_read1_sam_consumes_buffered_line_after_header_parse() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut c_char];
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
            let line = b"read2\t4\t*\t0\t0\t*\t*\t0\t0\t*\t*\tZZ:Z:tag\0";
            let mut buf = line.to_vec();
            let mut fp: htsFile = std::mem::zeroed();
            fp.format.format = HTS_FORMAT_SAM;
            fp.line.l = line.len() - 1;
            fp.line.m = line.len();
            fp.line.s = buf.as_mut_ptr().cast();

            assert_eq!(sam_read1(&mut fp, &mut hdr, b), 0);
            assert_eq!(fp.line.l, 0);
            assert_eq!(CStr::from_ptr(bam_get_qname(b)).to_bytes(), b"read2");
            assert_ne!((*b).core.flag as c_int & BAM_FUNMAP, 0);
            assert_eq!((*b).core.tid, -1);
            assert_eq!((*b).core.l_qseq, 0);
            let zz = bam_aux_get(b, c"ZZ".as_ptr());
            assert!(!zz.is_null());
            assert_eq!(CStr::from_ptr(bam_aux2Z(zz)).to_bytes(), b"tag");

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_write1_writes_sam_text_and_promotes_binary_bam() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut c_char];
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
                    c"read1".as_ptr(),
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
                    seq.as_ptr(),
                    qual.as_ptr().cast(),
                    0,
                ),
                18
            );

            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-sam-write1-{}-{}.sam.gz",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut fp: htsFile = std::mem::zeroed();
            fp.bitfields = 1 << 4;
            fp.fp = crate::htslib_mini_rs::hts::htsFilePtr { bgzf };
            fp.format.category = HTS_FORMAT_SEQUENCE_DATA;
            fp.format.format = HTS_FORMAT_SAM;
            let written = sam_c_4553_sam_write1(&mut fp, &hdr, b);
            assert!(written > 0);
            assert!(
                std::slice::from_raw_parts(fp.line.s.cast::<u8>(), fp.line.l)
                    .starts_with(b"read1\t0\tchr1\t1\t60\t4M")
            );
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);

            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let mut buf = [0u8; 128];
            let n =
                crate::htslib_mini_rs::bgzf::bgzf_read(bgzf, buf.as_mut_ptr().cast(), buf.len());
            assert!(n > 0);
            assert!(buf[..n as usize].starts_with(b"read1\t0\tchr1\t1\t60\t4M"));
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);
            let _ = std::fs::remove_file(&path);
            ks_free(&mut fp.line);

            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-binary-write1-{}-{}.bam",
                std::process::id(),
                line!()
            ));
            let path_c = CString::new(path.to_str().unwrap()).unwrap();
            let bgzf = crate::htslib_mini_rs::bgzf::bgzf_open(path_c.as_ptr(), c"w".as_ptr());
            assert!(!bgzf.is_null());
            let mut fp: htsFile = std::mem::zeroed();
            fp.fp = crate::htslib_mini_rs::hts::htsFilePtr { bgzf };
            fp.format.format = HTS_FORMAT_BINARY_FORMAT;
            assert!(sam_c_4553_sam_write1(&mut fp, &hdr, b) > 0);
            assert_eq!(fp.format.category, HTS_FORMAT_SEQUENCE_DATA);
            assert_eq!(fp.format.format, HTS_FORMAT_BAM);
            assert_eq!(crate::htslib_mini_rs::bgzf::bgzf_close(bgzf), 0);
            let _ = std::fs::remove_file(&path);

            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_parse1_and_format1_preserve_equal_mate_and_missing_quality() {
        unsafe {
            let chr1 = CString::new("chr1").unwrap();
            let mut target_names = [chr1.as_ptr() as *mut c_char];
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

            let line = b"read_eq\t99\tchr1\t5\t0\t2M\t=\t7\t2\tAC\t*\tNM:i:0\0";
            let mut buf = line.to_vec();
            let mut ks = kstring_t {
                l: line.len() - 1,
                m: line.len(),
                s: buf.as_mut_ptr().cast(),
            };
            assert_eq!(sam_c_2662_sam_parse1(&mut ks, &mut hdr, b), 0);
            assert_eq!((*b).core.tid, 0);
            assert_eq!((*b).core.pos, 4);
            assert_eq!((*b).core.mtid, 0);
            assert_eq!((*b).core.mpos, 6);
            assert_eq!((*b).core.isize, 2);
            assert_eq!(*bam_get_qual(b), 0xff);

            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(sam_format1(&hdr, b, &mut out) > 0);
            assert_eq!(
                CStr::from_ptr(out.s).to_bytes(),
                b"read_eq\t99\tchr1\t5\t0\t2M\t=\t7\t2\tAC\t*\tNM:i:0"
            );
            ks_free(&mut out);
            bam_destroy1(b);
        }
    }

    #[test]
    fn sam_format1_formats_unmapped_empty_sequence_like_htslib() {
        unsafe {
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
                ref_count: 0,
            };
            let b = bam_init1();
            assert!(!b.is_null());
            assert_eq!(
                bam_set1(
                    b,
                    1,
                    c"r".as_ptr(),
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

            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(sam_format1(&mut hdr, b, &mut out) > 0);
            assert_eq!(
                CStr::from_ptr(out.s).to_bytes(),
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
                bam_aux_append(b, c"AC".as_ptr(), b'A' as c_char, 1, achar.as_ptr()),
                0
            );
            assert_eq!(
                bam_aux_append(b, c"HX".as_ptr(), b'H' as c_char, 5, b"0A0b\0".as_ptr(),),
                0
            );
            let array = [b'A', 2, 0, 0, 0, b'X', b'Y'];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"BA".as_ptr(),
                    b'B' as c_char,
                    array.len() as c_int,
                    array.as_ptr(),
                ),
                0
            );

            let mut ks = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(bam_aux_get_str(b, c"AC".as_ptr(), &mut ks), 1);
            assert_eq!(CStr::from_ptr(ks.s).to_bytes(), b"AC:A:Z");
            ks.l = 0;

            assert_eq!(bam_aux_get_str(b, c"HX".as_ptr(), &mut ks), 1);
            assert_eq!(CStr::from_ptr(ks.s).to_bytes(), b"HX:H:0A0b");
            ks.l = 0;

            assert_eq!(bam_aux_get_str(b, c"BA".as_ptr(), &mut ks), -1);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            ks_free(&mut ks);
            bam_destroy1(b);
        }
    }

    #[test]
    fn parse_sam_flag_matches_decimal_zero_hex_and_overflow_rules() {
        unsafe {
            let mut end: *mut c_char = std::ptr::null_mut();
            let mut overflow = 0;
            let input = CString::new("16\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast_mut(), &mut end, &mut overflow),
                16
            );
            assert_eq!(*end, b'\t' as c_char);
            assert_eq!(overflow, 0);

            let input = CString::new("0\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast_mut(), &mut end, &mut overflow),
                0
            );
            assert_eq!(*end, b'\t' as c_char);

            let input = CString::new("010\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast_mut(), &mut end, &mut overflow),
                8
            );
            assert_eq!(*end, b'\t' as c_char);

            let input = CString::new("0x10\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast_mut(), &mut end, &mut overflow),
                16
            );
            assert_eq!(*end, b'\t' as c_char);

            overflow = 0;
            let input = CString::new("0200000\t").unwrap();
            assert_eq!(
                sam_c_2498_parse_sam_flag(input.as_ptr().cast_mut(), &mut end, &mut overflow),
                65535
            );
            assert_eq!(overflow, 1);
        }
    }

    #[test]
    fn sam_hdr_destroy_frees_c_allocated_long_ref_hash() {
        unsafe {
            let hdr =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<sam_hdr_t>() as u64)
                    .cast::<sam_hdr_t>();
            assert!(!hdr.is_null());
            (*hdr).n_targets = 1;
            (*hdr).target_name = crate::htslib_mini_rs::c_compat::calloc(
                1,
                std::mem::size_of::<*mut c_char>() as u64,
            )
            .cast();
            (*hdr).target_len =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<u32>() as u64)
                    .cast();
            let name = crate::htslib_mini_rs::c_compat::malloc(5).cast::<c_char>();
            assert!(!name.is_null());
            std::ptr::copy_nonoverlapping(b"chr1\0".as_ptr().cast::<c_char>(), name, 5);
            *(*hdr).target_name = name;
            *(*hdr).target_len = u32::MAX;

            let sdict = crate::htslib_mini_rs::c_compat::calloc(
                1,
                std::mem::size_of::<khash_s2i_t>() as u64,
            )
            .cast::<khash_s2i_t>();
            assert!(!sdict.is_null());
            (*sdict).n_buckets = 1;
            (*sdict).size = 1;
            (*sdict).n_occupied = 1;
            (*sdict).upper_bound = 1;
            (*sdict).flags =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<u32>() as u64)
                    .cast();
            (*sdict).keys = crate::htslib_mini_rs::c_compat::calloc(
                1,
                std::mem::size_of::<*mut c_char>() as u64,
            )
            .cast();
            (*sdict).vals =
                crate::htslib_mini_rs::c_compat::calloc(1, std::mem::size_of::<i64>() as u64)
                    .cast();
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
        let mut fp = crate::htslib_mini_rs::hts::htsFile {
            bitfields: 0,
            padding_0: 0,
            lineno: 0,
            line: crate::htslib_mini_rs::hts::kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
            fn_: std::ptr::null_mut(),
            fn_aux: std::ptr::null_mut(),
            fp: crate::htslib_mini_rs::hts::htsFilePtr {
                bgzf: std::ptr::null_mut(),
            },
            state: std::ptr::null_mut(),
            format: crate::htslib_mini_rs::hts::htsFormat {
                category: 0,
                format: HTS_FORMAT_FASTA_FORMAT,
                version: crate::htslib_mini_rs::hts::htsFormatVersion { major: 0, minor: 0 },
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
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            let hdr = sam_hdr_read(&mut fp);
            assert!(!hdr.is_null());
            assert_eq!((*hdr).n_targets, 0);
            sam_hdr_destroy(hdr);

            fp.format.format = HTS_FORMAT_EMPTY_FORMAT;
            assert!(sam_hdr_read(&mut fp).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EPIPE as c_int
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
                data: std::ptr::null_mut(),
                l_data: 0,
                m_data: 0,
                mempolicy_and_reserved: 0,
            };
            assert_eq!(sam_read1(&mut fp, std::ptr::null_mut(), &mut record), -3);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EPIPE as c_int
            );

            fp.format.format = crate::htslib_mini_rs::hts::HTS_FORMAT_BINARY_FORMAT;
            assert_eq!(sam_read1(&mut fp, std::ptr::null_mut(), &mut record), -3);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ENOEXEC as c_int
            );
        }
    }

    #[test]
    fn sam_hdr_read_sam_branch_reads_header_without_consuming_first_record() {
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-sam-header-{}-{}.sam",
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
            let fp = crate::htslib_mini_rs::hts::hts_open(path_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp.is_null());
            let hdr = sam_hdr_read(fp);
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_name2tid(hdr, b"chr1\0".as_ptr().cast()), 0);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 20);

            let b = bam_init1();
            assert!(!b.is_null());
            let ret = sam_read1(fp, hdr, b);
            assert!(ret >= 0);
            assert_eq!((*b).core.tid, 0);
            assert_eq!((*b).core.pos, 2);
            assert_eq!((*b).core.l_qseq, 4);
            bam_destroy1(b);

            sam_hdr_destroy(hdr);
            assert_eq!(crate::htslib_mini_rs::hts::hts_close(fp), 0);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn bam_read1_matches_htslib_on_generated_bam_record() {
        use std::process::Command;

        let dir = std::env::temp_dir().join(format!(
            "htslib-mini-rs-bam-read1-{}-{}",
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
            let fp_rust =
                crate::htslib_mini_rs::hts::hts_open(bam_c.as_ptr(), b"r\0".as_ptr().cast());
            let fp_c = hts_sys::hts_open(bam_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp_rust.is_null());
            assert!(!fp_c.is_null());
            let hdr_rust = sam_hdr_read(fp_rust);
            let hdr_c = hts_sys::sam_hdr_read(fp_c);
            assert!(!hdr_rust.is_null());
            assert!(!hdr_c.is_null());

            let b_rust = bam_init1();
            let b_c = bam_init1();
            let ret_rust = bam_read1((*fp_rust).fp.bgzf, b_rust);
            let ret_c = hts_sys::bam_read1((*fp_c).fp.bgzf, b_c.cast());
            assert_eq!(ret_rust, ret_c);
            assert!(ret_rust > 0);
            assert_eq!((*b_rust).core.tid, (*b_c).core.tid);
            assert_eq!((*b_rust).core.pos, (*b_c).core.pos);
            assert_eq!((*b_rust).core.bin, (*b_c).core.bin);
            assert_eq!((*b_rust).core.qual, (*b_c).core.qual);
            assert_eq!((*b_rust).core.flag, (*b_c).core.flag);
            assert_eq!((*b_rust).core.n_cigar, (*b_c).core.n_cigar);
            assert_eq!((*b_rust).core.l_qseq, (*b_c).core.l_qseq);
            assert_eq!((*b_rust).l_data, (*b_c).l_data);
            assert_eq!(
                std::slice::from_raw_parts((*b_rust).data, (*b_rust).l_data as usize),
                std::slice::from_raw_parts((*b_c).data, (*b_c).l_data as usize)
            );
            assert_eq!(bam_read1((*fp_rust).fp.bgzf, b_rust), -1);
            assert_eq!(hts_sys::bam_read1((*fp_c).fp.bgzf, b_c.cast()), -1);

            bam_destroy1(b_rust);
            bam_destroy1(b_c);
            sam_hdr_destroy(hdr_rust);
            hts_sys::sam_hdr_destroy(hdr_c);
            assert_eq!(crate::htslib_mini_rs::hts::hts_close(fp_rust), 0);
            assert_eq!(hts_sys::hts_close(fp_c), 0);

            let fp_query =
                crate::htslib_mini_rs::hts::hts_open(bam_c.as_ptr(), b"r\0".as_ptr().cast());
            assert!(!fp_query.is_null());
            let hdr_query = sam_hdr_read(fp_query);
            assert!(!hdr_query.is_null());
            let idx = sam_index_load(fp_query, bam_c.as_ptr());
            assert!(!idx.is_null());
            let itr = sam_itr_queryi(idx, 0, 0, 50);
            assert!(!itr.is_null());
            let b_query = bam_init1();
            assert!(sam_itr_next(fp_query, itr, b_query) >= 0);
            assert_eq!((*b_query).core.tid, 0);
            assert_eq!((*b_query).core.pos, 2);
            assert_eq!(sam_itr_next(fp_query, itr, b_query), -1);
            bam_destroy1(b_query);
            crate::htslib_mini_rs::hts::hts_itr_destroy(itr);

            let itr = sam_itr_querys(idx, hdr_query, c"chr1:1-50".as_ptr());
            assert!(!itr.is_null());
            let b_query = bam_init1();
            assert!(sam_itr_next(fp_query, itr, b_query) >= 0);
            assert_eq!((*b_query).core.tid, 0);
            assert_eq!((*b_query).core.pos, 2);
            assert_eq!(sam_itr_next(fp_query, itr, b_query), -1);
            bam_destroy1(b_query);
            crate::htslib_mini_rs::hts::hts_itr_destroy(itr);

            assert!(sam_itr_querys(idx, hdr_query, c"missing:1-2".as_ptr()).is_null());
            crate::htslib_mini_rs::hts::hts_idx_destroy(idx);
            sam_hdr_destroy(hdr_query);
            assert_eq!(crate::htslib_mini_rs::hts::hts_close(fp_query), 0);

            let bai_path = bam_path.with_extension("bam.bai");
            let bai_c = CString::new(bai_path.to_str().unwrap()).unwrap();
            let idx2 = crate::htslib_mini_rs::hts::hts_idx_load2(bam_c.as_ptr(), bai_c.as_ptr());
            assert!(!idx2.is_null());
            crate::htslib_mini_rs::hts::hts_idx_destroy(idx2);
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
            assert_eq!(sam_hdr_length(hdr), text.len());
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_str(hdr).cast::<u8>(), sam_hdr_length(hdr)),
                text
            );
            assert_eq!(sam_hdr_name2tid(hdr, chr1.as_ptr().cast()), 0);
            assert_eq!(sam_hdr_name2tid(hdr, chr2.as_ptr().cast()), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 123);
            assert_eq!(sam_hdr_tid2len(hdr, 1), 456);
            assert_eq!(
                CStr::from_ptr(sam_hdr_tid2name(hdr, 0)),
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
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 123);
            sam_hdr_free(hdr);
        }
    }

    #[test]
    fn sam_hdr_parse_accepts_sq_tags_in_any_order_with_extra_fields() {
        let text = b"@HD\tVN:1.6\n@SQ\tLN:12\tSN:chr1\tAS:asm\r\n@SQ\tM5:abc\tSN:chr2\tLN:34\n";
        unsafe {
            let hdr = sam_hdr_parse(text.len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_nref(hdr), 2);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr2".as_ptr()), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 12);
            assert_eq!(sam_hdr_tid2len(hdr, 1), 34);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 0)).to_bytes(), b"chr1");
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 1)).to_bytes(), b"chr2");
            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn translated_header_type_helpers_match_htslib_boundaries() {
        unsafe {
            assert_eq!(header_h_58_TYPEKEY(c"HD".as_ptr()), 0x4844);
            assert_eq!(header_h_58_TYPEKEY(c"SQ".as_ptr()), 0x5351);

            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@HD\tVN:1.6".as_ptr()),
                1
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@SQ\tSN:chr1".as_ptr()),
                1
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@RG\tID:rg1".as_ptr()),
                1
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@PG\tID:pg1".as_ptr()),
                1
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@COcomment text".as_ptr()),
                1
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@HD VN:1.6".as_ptr()),
                0
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@SQ SN:chr1".as_ptr()),
                0
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"@XX\tID:x".as_ptr()),
                0
            );
            assert_eq!(
                header_c_1325_valid_sam_header_type(c"not-a-header".as_ptr()),
                0
            );
        }
    }

    #[test]
    fn sam_hdr_add_lines_accumulates_text_and_targets_from_new_lines_only() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let first = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:10\r\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, first.as_ptr().cast(), first.len()),
                0
            );
            assert_eq!(sam_hdr_length(hdr), first.len());
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_str(hdr).cast::<u8>(), first.len()),
                first
            );
            assert_eq!(*(*hdr).text.add((*hdr).l_text), 0);
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 10);

            let second = b"@RG\tID:rg1\n@SQ\tSN:chr2\tLN:20\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, second.as_ptr().cast(), second.len()),
                0
            );
            assert_eq!(sam_hdr_length(hdr), first.len() + second.len());
            assert_eq!(sam_hdr_nref(hdr), 2);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr2".as_ptr()), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 1), 20);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 1)).to_bytes(), b"chr2");

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_lines_rejects_duplicate_sq_names_without_changing_header() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let first = b"@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, first.as_ptr().cast(), first.len()),
                0
            );
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 10);
            assert_eq!(sam_hdr_length(hdr), first.len());

            let duplicate_existing = b"@SQ\tSN:chr1\tLN:20\n";
            assert_eq!(
                sam_hdr_add_lines(
                    hdr,
                    duplicate_existing.as_ptr().cast(),
                    duplicate_existing.len()
                ),
                -1
            );
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 10);
            assert_eq!(sam_hdr_length(hdr), first.len());
            assert_eq!(
                std::slice::from_raw_parts(sam_hdr_str(hdr).cast::<u8>(), sam_hdr_length(hdr)),
                first
            );

            let duplicate_new = b"@SQ\tSN:chr2\tLN:20\n@SQ\tSN:chr2\tLN:20\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, duplicate_new.as_ptr().cast(), duplicate_new.len()),
                -1
            );
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr2".as_ptr()), -1);
            assert_eq!(sam_hdr_length(hdr), first.len());

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn redact_header_text_clears_only_serialized_text() {
        let text = b"@SQ\tSN:chr1\tLN:123\n";
        unsafe {
            let hdr = sam_hdr_parse(text.len(), text.as_ptr().cast());
            assert!(!hdr.is_null());
            assert!(!(*hdr).text.is_null());
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 0);

            header_c_1530_redact_header_text(hdr);
            assert_eq!((*hdr).l_text, 0);
            assert!((*hdr).text.is_null());
            assert_eq!(sam_hdr_length(hdr), 0);
            assert!(sam_hdr_str(hdr).is_null());
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 123);

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
            (*hdr).text = crate::htslib_mini_rs::c_compat::malloc(text.len() as u64 + 1).cast();
            assert!(!(*hdr).text.is_null());
            crate::htslib_mini_rs::c_compat::memcpy(
                (*hdr).text.cast(),
                text.as_ptr().cast(),
                text.len() as u64,
            );
            *(*hdr).text.add(text.len()) = 0;
            (*hdr).l_text = text.len();

            assert_eq!((*hdr).n_targets, 0);
            assert_eq!(sam_hdr_name2tid(hdr, chr.as_ptr().cast()), 0);
            assert_eq!((*hdr).n_targets, 1);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 11);
            assert_eq!(sam_hdr_name2tid(hdr, b"missing\0".as_ptr().cast()), -1);
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
            assert_eq!(sam_hdr_nref(std::ptr::null()), -1);
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
            text: text.as_ptr().cast_mut().cast::<c_char>(),
            sdict: std::ptr::null_mut(),
            hrecs: std::ptr::null_mut(),
            ref_count: 0,
        };

        unsafe {
            assert_eq!(sam_hdr_length(std::ptr::null_mut()), usize::MAX);
            assert_eq!(sam_hdr_str(std::ptr::null_mut()), std::ptr::null());
            assert_eq!(sam_hdr_length(&mut hdr), text.len() - 1);
            assert_eq!(sam_hdr_str(&mut hdr), text.as_ptr().cast::<c_char>());
        }
    }

    #[test]
    fn sam_hdr_dup_copies_simple_header_like_htslib() {
        let chr1 = b"chr1\0";
        let chr2 = b"chr2\0";
        let text = b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:100\n@SQ\tSN:chr2\tLN:200\n\0";
        let mut target_len = [100u32, 200u32];
        let mut target_name = [
            chr1.as_ptr().cast_mut().cast::<c_char>(),
            chr2.as_ptr().cast_mut().cast::<c_char>(),
        ];
        let hdr = sam_hdr_t {
            n_targets: 2,
            ignore_sam_err: 7,
            l_text: text.len() - 1,
            target_len: target_len.as_mut_ptr(),
            cigar_tab: std::ptr::null(),
            target_name: target_name.as_mut_ptr(),
            text: text.as_ptr().cast_mut().cast::<c_char>(),
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
                CStr::from_ptr(*(*dup).target_name),
                CStr::from_bytes_with_nul(chr1).unwrap()
            );
            assert_eq!(
                CStr::from_ptr(*(*dup).target_name.add(1)),
                CStr::from_bytes_with_nul(chr2).unwrap()
            );
            assert_eq!(
                CStr::from_ptr((*dup).text),
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
            mp: std::ptr::null_mut(),
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            tid: 0,
            max_tid: 0,
            pos: 0,
            max_pos: 0,
            is_eof: 0,
            max_plp: 0,
            error: 0,
            maxcnt: 8000,
            id: 0,
            plp: std::ptr::null_mut(),
            b: std::ptr::null_mut(),
            func: None,
            data: std::ptr::null_mut(),
            overlaps: std::ptr::null_mut(),
            plp_construct: None,
            plp_destruct: None,
        };
        let mut plp1 = bam_plp_s {
            maxcnt: 8000,
            ..plp0
        };
        let mut iters = [&mut plp0 as bam_plp_t, &mut plp1 as bam_plp_t];
        let mut mplp = bam_mplp_s {
            n: 2,
            min_tid: 0,
            tid: std::ptr::null_mut(),
            min_pos: 0,
            pos: std::ptr::null_mut(),
            iter: iters.as_mut_ptr(),
            n_plp: std::ptr::null_mut(),
            plp: std::ptr::null_mut(),
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
        _data: *mut c_void,
        _b: *const bam1_t,
        _cd: *mut bam_pileup_cd,
    ) -> c_int {
        0
    }

    #[test]
    fn pileup_constructor_destructor_setters_match_htslib_field_assignment() {
        let mut plp0 = bam_plp_s {
            mp: std::ptr::null_mut(),
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            tid: 0,
            max_tid: 0,
            pos: 0,
            max_pos: 0,
            is_eof: 0,
            max_plp: 0,
            error: 0,
            maxcnt: 8000,
            id: 0,
            plp: std::ptr::null_mut(),
            b: std::ptr::null_mut(),
            func: None,
            data: std::ptr::null_mut(),
            overlaps: std::ptr::null_mut(),
            plp_construct: None,
            plp_destruct: None,
        };
        let mut plp1 = bam_plp_s {
            plp_construct: None,
            plp_destruct: None,
            ..plp0
        };
        let mut iters = [&mut plp0 as bam_plp_t, &mut plp1 as bam_plp_t];
        let mut mplp = bam_mplp_s {
            n: 2,
            min_tid: 0,
            tid: std::ptr::null_mut(),
            min_pos: 0,
            pos: std::ptr::null_mut(),
            iter: iters.as_mut_ptr(),
            n_plp: std::ptr::null_mut(),
            plp: std::ptr::null_mut(),
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
        let mut user_data = [std::ptr::null_mut::<c_void>(), std::ptr::null_mut()];
        unsafe {
            let mplp = bam_mplp_init(2, None, user_data.as_mut_ptr());
            assert!(!mplp.is_null());
            assert_eq!((*mplp).n, 2);
            assert_eq!((*mplp).min_pos, HTS_POS_MAX);
            assert_eq!((*mplp).min_tid, -1);
            for i in 0..2 {
                let idx = i as usize;
                assert_eq!(*(*mplp).pos.add(idx), HTS_POS_MAX);
                assert_eq!(*(*mplp).tid.add(idx), -1);
                assert_eq!(*(*mplp).n_plp.add(idx), 0);
                assert!((*(*mplp).plp.add(idx)).is_null());
                assert!(!(*(*mplp).iter.add(idx)).is_null());
            }
            bam_mplp_destroy(mplp);
        }
    }

    #[test]
    fn bam_mplp_reset_restores_iterator_sentinels() {
        let mut user_data = [std::ptr::null_mut::<c_void>(), std::ptr::null_mut()];
        unsafe {
            let mplp = bam_mplp_init(2, None, user_data.as_mut_ptr());
            assert!(!mplp.is_null());
            (*mplp).min_pos = 12;
            (*mplp).min_tid = 3;
            for i in 0..2 {
                let idx = i as usize;
                *(*mplp).pos.add(idx) = i as hts_pos_t;
                *(*mplp).tid.add(idx) = i;
                *(*mplp).n_plp.add(idx) = 9;
                *(*mplp).plp.add(idx) = std::ptr::dangling();
                (*(*(*mplp).iter.add(idx))).tid = 7;
                (*(*(*mplp).iter.add(idx))).pos = 8;
                (*(*(*mplp).iter.add(idx))).is_eof = 1;
            }

            bam_mplp_reset(mplp);
            assert_eq!((*mplp).min_pos, HTS_POS_MAX);
            assert_eq!((*mplp).min_tid, -1);
            for i in 0..2 {
                let idx = i as usize;
                assert_eq!(*(*mplp).pos.add(idx), HTS_POS_MAX);
                assert_eq!(*(*mplp).tid.add(idx), -1);
                assert_eq!(*(*mplp).n_plp.add(idx), 0);
                assert!((*(*mplp).plp.add(idx)).is_null());
                assert_eq!((*(*(*mplp).iter.add(idx))).tid, 0);
                assert_eq!((*(*(*mplp).iter.add(idx))).pos, 0);
                assert_eq!((*(*(*mplp).iter.add(idx))).is_eof, 0);
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
                    c"pins".as_ptr(),
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
                    c"ACG".as_ptr(),
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
            let mut ins = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            let mut del_len = -1;
            assert_eq!(bam_plp_insertion(&p, &mut ins, &mut del_len), 2);
            assert_eq!(del_len, 1);
            assert_eq!(CStr::from_ptr(ins.s).to_bytes(), b"CG");

            let state = hts_base_mod_state_alloc();
            let mut mm_end = *b";\0";
            let mut ml = [55u8];
            (*state).nmods = 1;
            (*state).type_[0] = b'm' as c_int;
            (*state).canonical[0] = 2;
            (*state).mmcount[0] = 0;
            (*state).mm[0] = mm_end.as_mut_ptr().cast();
            (*state).ml[0] = ml.as_mut_ptr();
            (*state).mlstride[0] = 1;
            (*state).implicit[0] = 1;
            (*state).seq_pos = 0;
            assert_eq!(bam_plp_insertion_mod(&p, state, &mut ins, &mut del_len), 2);
            assert_eq!(CStr::from_ptr(ins.s).to_bytes(), b"C[+m55]G");

            let no_ins = bam_pileup1_t { indel: 0, ..p };
            assert_eq!(bam_plp_insertion(&no_ins, &mut ins, &mut del_len), 0);
            assert_eq!(CStr::from_ptr(ins.s).to_bytes(), b"");

            crate::htslib_mini_rs::c_compat::free(ins.s.cast());
            hts_base_mod_state_free(state);
            bam_destroy1(b);
        }
    }

    #[test]
    fn empty_mplp_auto_returns_zero_like_htslib_merge_loop() {
        let mut mplp = bam_mplp_s {
            n: 0,
            min_tid: -1,
            tid: std::ptr::null_mut(),
            min_pos: HTS_POS_MAX,
            pos: std::ptr::null_mut(),
            iter: std::ptr::null_mut(),
            n_plp: std::ptr::null_mut(),
            plp: std::ptr::null_mut(),
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
                bam_str2flag(c"PAIRED,proper_pair,UNMAP".as_ptr()),
                BAM_FPAIRED | BAM_FPROPER_PAIR | BAM_FUNMAP
            );
            assert_eq!(bam_str2flag(c"0x41".as_ptr()), 0x41);
            assert_eq!(bam_str2flag(c"PAIRED,NOPE".as_ptr()), -1);

            let text =
                bam_flag2str(BAM_FPAIRED | BAM_FUNMAP | BAM_FREAD1 | BAM_FDUP | BAM_FSUPPLEMENTARY);
            assert_eq!(
                CStr::from_ptr(text).to_bytes(),
                b"PAIRED,UNMAP,READ1,DUP,SUPPLEMENTARY"
            );
            crate::htslib_mini_rs::c_compat::free(text.cast());

            let empty = bam_flag2str(0);
            assert_eq!(CStr::from_ptr(empty).to_bytes(), b"");
            crate::htslib_mini_rs::c_compat::free(empty.cast());
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
            data: z_data.as_mut_ptr(),
            l_data: z_data.len() as c_int,
            m_data: z_data.len() as u32,
            mempolicy_and_reserved: BAM_USER_OWNS_STRUCT | BAM_USER_OWNS_DATA,
        };

        unsafe {
            assert!(bam_aux_get(&z_record, c"ZZ".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );
        }

        let mut b_data = b"r\0\0\0XABC\x02\0\0\0\x07".to_vec();
        let b_record = bam1_t {
            data: b_data.as_mut_ptr(),
            l_data: b_data.len() as c_int,
            m_data: b_data.len() as u32,
            ..z_record
        };

        unsafe {
            assert!(bam_aux_get(&b_record, c"XA".as_ptr()).is_null());
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            let b = bam_init1();
            assert!(!b.is_null());
            let array = [b'C', 2, 0, 0, 0, 10, 20];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"BC".as_ptr(),
                    b'B' as c_char,
                    array.len() as c_int,
                    array.as_ptr(),
                ),
                0
            );
            let bc = bam_aux_get(b, c"BC".as_ptr());
            assert!(!bc.is_null());
            assert_eq!(bam_auxB2i(bc, 1), 20);
            assert_eq!(bam_auxB2i(bc, 2), 0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ERANGE as c_int
            );
            assert_eq!(bam_auxB2f(bc, 9), 0.0);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::ERANGE as c_int
            );
            bam_destroy1(b);
        }
    }

    #[test]
    fn aux_update_int_preserves_htslib_size_thresholds_and_type_checks() {
        unsafe {
            let b = bam_init1();
            assert!(!b.is_null());

            assert_eq!(bam_aux_update_int(b, c"IV".as_ptr(), -128), 0);
            let iv = bam_aux_get(b, c"IV".as_ptr());
            assert_eq!(bam_aux_type(iv), b'c' as c_char);
            assert_eq!(bam_aux2i(iv), -128);

            assert_eq!(bam_aux_update_int(b, c"IV".as_ptr(), -129), 0);
            let iv = bam_aux_get(b, c"IV".as_ptr());
            assert_eq!(bam_aux_type(iv), b's' as c_char);
            assert_eq!(bam_aux2i(iv), -129);

            assert_eq!(bam_aux_update_int(b, c"IV".as_ptr(), 254), 0);
            let iv = bam_aux_get(b, c"IV".as_ptr());
            assert_eq!(bam_aux_type(iv), b'S' as c_char);
            assert_eq!(bam_aux2i(iv), 254);

            assert_eq!(bam_aux_update_int(b, c"UV".as_ptr(), u32::MAX as i64), 0);
            let uv = bam_aux_get(b, c"UV".as_ptr());
            assert_eq!(bam_aux_type(uv), b'I' as c_char);
            assert_eq!(bam_aux2i(uv), u32::MAX as i64);

            assert_eq!(
                bam_aux_update_int(b, c"OV".as_ptr(), u32::MAX as i64 + 1),
                -1
            );
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EOVERFLOW as c_int
            );

            assert_eq!(
                bam_aux_update_str(b, c"ZS".as_ptr(), -1, c"text".as_ptr()),
                0
            );
            assert_eq!(bam_aux_update_int(b, c"ZS".as_ptr(), 1), -1);
            assert_eq!(
                *crate::htslib_mini_rs::c_compat::__errno_location(),
                crate::htslib_mini_rs::c_compat::EINVAL as c_int
            );

            bam_destroy1(b);
        }
    }

    #[test]
    fn malformed_header_sq_lines_do_not_create_targets() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let zero_len = b"@SQ\tSN:zero\tLN:0\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, zero_len.as_ptr().cast(), zero_len.len()),
                -1
            );
            assert_eq!(sam_hdr_nref(hdr), 0);
            assert_eq!(sam_hdr_length(hdr), 0);
            assert!(sam_hdr_str(hdr).is_null());
            assert_eq!(sam_hdr_name2tid(hdr, c"zero".as_ptr()), -1);

            let conflicting_len = b"@SQ\tSN:dup\tLN:10\tLN:11\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, conflicting_len.as_ptr().cast(), conflicting_len.len()),
                -1
            );
            assert_eq!(sam_hdr_nref(hdr), 0);
            assert_eq!(sam_hdr_length(hdr), 0);
            assert!(sam_hdr_str(hdr).is_null());
            assert_eq!(sam_hdr_name2tid(hdr, c"dup".as_ptr()), -1);

            let duplicate_same_len = b"@SQ\tSN:same\tLN:12\tLN:12\n";
            assert_eq!(
                sam_hdr_add_lines(
                    hdr,
                    duplicate_same_len.as_ptr().cast(),
                    duplicate_same_len.len()
                ),
                0
            );
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"same".as_ptr()), 0);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 12);
            let text_len_after_good = sam_hdr_length(hdr);

            let bad_missing_len = b"@SQ\tSN:bad\tLN:\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, bad_missing_len.as_ptr().cast(), bad_missing_len.len()),
                -1
            );
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_length(hdr), text_len_after_good);
            assert_eq!(sam_hdr_name2tid(hdr, c"bad".as_ptr()), -1);

            let long = b"@SQ\tSN:long\tLN:4294967296\n";
            assert_eq!(sam_hdr_add_lines(hdr, long.as_ptr().cast(), long.len()), 0);
            assert_eq!(sam_hdr_nref(hdr), 2);
            assert_eq!(sam_hdr_name2tid(hdr, c"long".as_ptr()), 1);
            assert_eq!(sam_hdr_tid2len(hdr, 1), 4_294_967_296);

            let bad_overflow = b"@SQ\tSN:huge\tLN:9223372034707292161\n";
            assert_eq!(
                sam_hdr_add_lines(hdr, bad_overflow.as_ptr().cast(), bad_overflow.len()),
                -1
            );
            assert_eq!(sam_hdr_nref(hdr), 2);
            assert_eq!(sam_hdr_length(hdr), text_len_after_good + long.len());
            assert_eq!(sam_hdr_name2tid(hdr, c"huge".as_ptr()), -1);

            let good = b"@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(sam_hdr_add_lines(hdr, good.as_ptr().cast(), good.len()), 0);
            assert_eq!(sam_hdr_nref(hdr), 3);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 2);
            assert_eq!(sam_hdr_tid2len(hdr, 2), 10);

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn header_sq_target_parsing_accepts_crlf_and_u32_max_len_edges() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let text = b"@HD\tVN:1.6\r\n@SQ\tLN:4294967295\tSN:max\r\n@CO\tignored\r\n";
            assert_eq!(sam_hdr_add_lines(hdr, text.as_ptr().cast(), text.len()), 0);
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"max".as_ptr()), 0);
            assert_eq!(sam_hdr_tid2len(hdr, 0), u32::MAX as hts_pos_t);
            assert_eq!(CStr::from_ptr(sam_hdr_tid2name(hdr, 0)).to_bytes(), b"max");

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_lines_len_zero_uses_nul_terminated_input() {
        unsafe {
            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());

            let text = c"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:10\n";
            assert_eq!(sam_hdr_add_lines(hdr, text.as_ptr(), 0), 0);
            assert_eq!(sam_hdr_length(hdr), text.to_bytes().len());
            assert_eq!(sam_hdr_nref(hdr), 1);
            assert_eq!(sam_hdr_name2tid(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(sam_hdr_tid2len(hdr, 0), 10);

            sam_hdr_destroy(hdr);
        }
    }

    #[test]
    fn sam_hdr_add_lines_rejects_null_and_hrec_backed_inputs() {
        unsafe {
            assert_eq!(
                sam_hdr_add_lines(std::ptr::null_mut(), c"@HD\tVN:1.6\n".as_ptr(), 11),
                -1
            );

            let hdr = sam_hdr_init();
            assert!(!hdr.is_null());
            assert_eq!(sam_hdr_add_lines(hdr, std::ptr::null(), 0), -1);

            let empty = c"";
            assert_eq!(sam_hdr_add_lines(hdr, empty.as_ptr(), 0), 0);
            assert_eq!(sam_hdr_length(hdr), 0);
            assert!(sam_hdr_str(hdr).is_null());

            let mut hrecs: sam_hrecs_t = std::mem::zeroed();
            let line = b"@SQ\tSN:chr1\tLN:1\n";
            (*hdr).hrecs = &mut hrecs;
            assert_eq!(sam_hdr_add_lines(hdr, line.as_ptr().cast(), line.len()), -1);
            (*hdr).hrecs = std::ptr::null_mut();
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

            let mut end: *mut c_char = std::ptr::null_mut();
            let mut a_cigar: *mut u32 = std::ptr::null_mut();
            let mut a_mem = 0usize;
            let invalid_op = c"1Q\t";
            assert_eq!(
                sam_parse_cigar(invalid_op.as_ptr(), &mut end, &mut a_cigar, &mut a_mem),
                -1
            );
            assert_eq!(end, invalid_op.as_ptr().cast_mut());

            let overflow = c"268435456M\t";
            assert_eq!(
                sam_parse_cigar(overflow.as_ptr(), &mut end, &mut a_cigar, &mut a_mem),
                -1
            );
            assert_eq!(end, overflow.as_ptr().cast_mut());
            crate::htslib_mini_rs::c_compat::free(a_cigar.cast());

            let b = bam_init1();
            assert!(!b.is_null());
            let cigar = [(3u32 << BAM_CIGAR_SHIFT) | BAM_CMATCH as u32];
            assert!(
                bam_set1(
                    b,
                    5,
                    c"cigx".as_ptr(),
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
                    c"ACG".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let old_l_data = (*b).l_data;
            assert_eq!(bam_parse_cigar(invalid_op.as_ptr(), &mut end, b), -1);
            assert_eq!(end, invalid_op.as_ptr().cast_mut());
            assert_eq!((*b).core.n_cigar, 1);
            assert_eq!((*b).l_data, old_l_data);
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
                    c"pilex".as_ptr(),
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
                    c"AACCGG".as_ptr(),
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
                    c"padx".as_ptr(),
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
                    c"AACCG".as_ptr(),
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
                    c"hpad".as_ptr(),
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
                    c"ACG".as_ptr(),
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
                    c"bmm2".as_ptr(),
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
                    c"C".as_ptr(),
                    std::ptr::null(),
                    0,
                ) > 0
            );
            let mm = b"C+mh,0;\0";
            assert_eq!(
                bam_aux_append(
                    b,
                    c"MM".as_ptr(),
                    b'Z' as c_char,
                    mm.len() as c_int,
                    mm.as_ptr()
                ),
                0
            );
            let ml = [b'C', 2, 0, 0, 0, 11, 22];
            assert_eq!(
                bam_aux_append(
                    b,
                    c"ML".as_ptr(),
                    b'B' as c_char,
                    ml.len() as c_int,
                    ml.as_ptr()
                ),
                0
            );

            let state = hts_base_mod_state_alloc();
            assert!(!state.is_null());
            assert_eq!(bam_parse_basemod(b, state), 0);
            assert_eq!((*state).nmods, 2);
            assert_eq!((*state).type_[0], b'm' as c_int);
            assert_eq!((*state).type_[1], b'h' as c_int);
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
                bam_next_basemod(b, state, mods.as_mut_ptr(), mods.len() as c_int, &mut pos),
                2
            );
            assert_eq!(pos, 0);
            assert_eq!(mods[0].modified_base, b'm' as c_int);
            assert_eq!(mods[0].qual, 11);
            assert_eq!(mods[1].modified_base, b'h' as c_int);
            assert_eq!(mods[1].qual, 22);
            assert_eq!(
                bam_next_basemod(b, state, mods.as_mut_ptr(), mods.len() as c_int, &mut pos),
                0
            );

            hts_base_mod_state_free(state);
            bam_destroy1(b);
        }
    }
}
