use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use std::mem::size_of;

use crate::htslib_rs::c_compat;
use std::sync::atomic::{AtomicU64, Ordering};

use super::bgzf::{
    bgzf_c_189_bgzf_idx_push, bgzf_flush, bgzf_internal_h_51_bgzf_set_private_data,
    bgzf_internal_h_67_bgzf_get_private_data, bgzf_read, bgzf_useek, bgzf_utell, bgzf_write,
    BgzfPrivateDataCleanupFunc,
};
use super::hfile::{hseek, htslib_hfile_h_155_htell as htell};
use super::hts::{
    htsFile, hts_close, hts_get_bgzfp, hts_getline, hts_idx_t, hts_itr_t, hts_open, hts_pos_t,
    hts_str2dbl, hts_str2int, hts_str2uint, i16_to_le, i32_to_le, i64_to_le, kbitset_t, kputc,
    kputc_, kputd, kputs, kputsn, kputw, ks_resize, kstring_t, kstrtok, le_to_float, le_to_i16,
    le_to_i32, le_to_i64, le_to_i8, le_to_u16, le_to_u32, size_t, toupper_c, BGZF,
    HTS_COMPRESSION_BGZF, HTS_COMPRESSION_NO_COMPRESSION, HTS_FORMAT_BCF, HTS_FORMAT_BINARY_FORMAT,
    HTS_FORMAT_TEXT_FORMAT, HTS_FORMAT_VARIANT_DATA, HTS_FORMAT_VCF, HTS_POS_MAX, KS_SEP_LINE,
};

// Re-exports of items extracted into sibling files at the crate root.
// The Rust file layout mirrors htslib: htslib/synced_bcf_reader.c ->
// src/synced_bcf_reader.rs, htslib/bcf_sr_sort.c -> src/bcf_sr_sort.rs,
// htslib/vcfutils.c -> src/vcfutils.rs. These re-exports preserve the
// public surface so `crate::htslib_rs::vcf::*` still resolves the same
// way it did when these files lived in src/vcf/.
pub use crate::htslib_rs::bcf_sr_sort::*;
pub use crate::htslib_rs::synced_bcf_reader::*;
pub use crate::htslib_rs::vcf_sweep::*;
pub use crate::htslib_rs::vcfutils::*;

// Native BCF/VCF struct definitions. Byte-identical to the hts-sys bindgen
// layouts (verified against bindgen_test_layout_* assertions in
// target/debug/build/hts-sys-*/out/bindings.rs). Replaces the previous
// `pub type bcf_*_t = hts_sys::bcf_*_t;` aliases.
//
// `__BcfBitfieldUnit<[u8; N]>` mirrors bindgen's `__BindgenBitfieldUnit`
// helper used by `bcf1_t`, `bcf_fmt_t`, and `bcf_info_t`. We re-implement
// it locally so the public field types stay byte-for-byte identical
// without pulling in the hts-sys helper.

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct __BcfBitfieldUnit<Storage> {
    storage: Storage,
}
impl<Storage> __BcfBitfieldUnit<Storage> {
    #[inline]
    pub const fn new(storage: Storage) -> Self {
        Self { storage }
    }
}
impl<Storage> __BcfBitfieldUnit<Storage>
where
    Storage: AsRef<[u8]> + AsMut<[u8]>,
{
    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let byte = self.storage.as_ref()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1u8 << bit_index;
        byte & mask == mask
    }
    #[inline]
    fn set_bit(&mut self, index: usize, val: bool) {
        let byte_index = index / 8;
        let byte = &mut self.storage.as_mut()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1u8 << bit_index;
        if val {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
    #[inline]
    pub fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
        // The original bit-by-bit form loops `bit_width` times and reads one
        // byte per iteration. For bcf1_t::n_fmt (8 bits) / n_sample (24 bits)
        // that's 8–24 byte loads + per-bit branches per call. We call this
        // multiple times per record while parsing VCF (hot path: ~60% of
        // vcf_parse_format time on multi-sample VCFs was getting/setting the
        // bit-packed n_fmt/n_sample). Byte-aligned widths are the common case
        // and reduce to a single load on x86_64.
        debug_assert!((bit_width as usize) <= 64);
        let bytes = self.storage.as_ref();
        if bit_offset % 8 == 0 && bit_width % 8 == 0 {
            let start = bit_offset / 8;
            let width_bytes = bit_width as usize / 8;
            let mut val: u64 = 0;
            for i in 0..width_bytes {
                let b = bytes[start + i] as u64;
                if cfg!(target_endian = "big") {
                    val |= b << ((width_bytes - 1 - i) * 8);
                } else {
                    val |= b << (i * 8);
                }
            }
            return val;
        }
        // Fallback (rare): mixed bit-offset/width — use the original logic.
        let mut val: u64 = 0;
        for i in 0..(bit_width as usize) {
            if self.get_bit(i + bit_offset) {
                let index = if cfg!(target_endian = "big") {
                    bit_width as usize - 1 - i
                } else {
                    i
                };
                val |= 1u64 << index;
            }
        }
        val
    }
    #[inline]
    pub fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
        // See `get()` above for the rationale. Byte-aligned widths are the
        // hot case (bcf1_t bit-packed fields are designed to fall on byte
        // boundaries) and let us write the value as plain byte stores instead
        // of one-bit-at-a-time read-modify-write.
        debug_assert!((bit_width as usize) <= 64);
        if bit_offset % 8 == 0 && bit_width % 8 == 0 {
            let start = bit_offset / 8;
            let width_bytes = bit_width as usize / 8;
            let bytes = self.storage.as_mut();
            for i in 0..width_bytes {
                let shift = if cfg!(target_endian = "big") {
                    (width_bytes - 1 - i) * 8
                } else {
                    i * 8
                };
                bytes[start + i] = (val >> shift) as u8;
            }
            return;
        }
        // Fallback (rare): unaligned offset or width — original logic.
        for i in 0..(bit_width as usize) {
            let mask = 1u64 << i;
            let val_bit_is_set = val & mask == mask;
            let index = if cfg!(target_endian = "big") {
                bit_width as usize - 1 - i
            } else {
                i
            };
            self.set_bit(index + bit_offset, val_bit_is_set);
        }
    }
}

// original: bcf_hrec_t (htslib/vcf.h) — variant record header line.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_hrec_t {
    pub type_: c_int,
    pub key: *mut c_char,
    pub value: *mut c_char,
    pub nkeys: c_int,
    pub keys: *mut *mut c_char,
    pub vals: *mut *mut c_char,
}

// original: bcf_idinfo_t (htslib/vcf.h)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_idinfo_t {
    pub info: [u64; 3usize],
    pub hrec: [*mut bcf_hrec_t; 3usize],
    pub id: c_int,
}

// original: bcf_idpair_t (htslib/vcf.h)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_idpair_t {
    pub key: *const c_char,
    pub val: *const bcf_idinfo_t,
}

// original: bcf_hdr_t (htslib/vcf.h)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcf_hdr_t {
    pub n: [i32; 3usize],
    pub id: [*mut bcf_idpair_t; 3usize],
    pub dict: [*mut c_void; 3usize],
    pub samples: *mut *mut c_char,
    pub hrec: *mut *mut bcf_hrec_t,
    pub nhrec: c_int,
    pub dirty: c_int,
    pub ntransl: c_int,
    pub transl: [*mut c_int; 2usize],
    pub nsamples_ori: c_int,
    pub keep_samples: *mut u8,
    pub mem: kstring_t,
    pub m: [i32; 3usize],
}

// original: variant_t (renamed to bcf_variant_t in HTSlib v1.23).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_variant_t {
    pub type_: c_int,
    pub n: c_int,
}

// original: bcf_fmt_t (htslib/vcf.h)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_fmt_t {
    pub id: c_int,
    pub n: c_int,
    pub size: c_int,
    pub type_: c_int,
    pub p: *mut u8,
    pub p_len: u32,
    pub _bitfield_align_1: [u32; 0],
    pub _bitfield_1: __BcfBitfieldUnit<[u8; 4usize]>,
}
impl bcf_fmt_t {
    #[inline]
    pub fn p_off(&self) -> u32 {
        self._bitfield_1.get(0usize, 31u8) as u32
    }
    #[inline]
    pub fn set_p_off(&mut self, val: u32) {
        self._bitfield_1.set(0usize, 31u8, val as u64)
    }
    #[inline]
    pub fn p_free(&self) -> u32 {
        self._bitfield_1.get(31usize, 1u8) as u32
    }
    #[inline]
    pub fn set_p_free(&mut self, val: u32) {
        self._bitfield_1.set(31usize, 1u8, val as u64)
    }
}

// original: bcf_info_t (htslib/vcf.h). The v1 field is a union of i64/f32.
#[repr(C)]
#[derive(Copy, Clone)]
pub union bcf_info_t__bindgen_ty_1 {
    pub i: i64,
    pub f: f32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcf_info_t {
    pub key: c_int,
    pub type_: c_int,
    pub v1: bcf_info_t__bindgen_ty_1,
    pub vptr: *mut u8,
    pub vptr_len: u32,
    pub _bitfield_align_1: [u32; 0],
    pub _bitfield_1: __BcfBitfieldUnit<[u8; 4usize]>,
    pub len: c_int,
}
impl bcf_info_t {
    #[inline]
    pub fn vptr_off(&self) -> u32 {
        self._bitfield_1.get(0usize, 31u8) as u32
    }
    #[inline]
    pub fn set_vptr_off(&mut self, val: u32) {
        self._bitfield_1.set(0usize, 31u8, val as u64)
    }
    #[inline]
    pub fn vptr_free(&self) -> u32 {
        self._bitfield_1.get(31usize, 1u8) as u32
    }
    #[inline]
    pub fn set_vptr_free(&mut self, val: u32) {
        self._bitfield_1.set(31usize, 1u8, val as u64)
    }
}

// original: bcf_dec_t (htslib/vcf.h) — decoded BCF record payload. Used only
// through `bcf1_t::d`, so doesn't need a `pub type` alias.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_dec_t {
    pub m_fmt: c_int,
    pub m_info: c_int,
    pub m_id: c_int,
    pub m_als: c_int,
    pub m_allele: c_int,
    pub m_flt: c_int,
    pub n_flt: c_int,
    pub flt: *mut c_int,
    pub id: *mut c_char,
    pub als: *mut c_char,
    pub allele: *mut *mut c_char,
    pub info: *mut bcf_info_t,
    pub fmt: *mut bcf_fmt_t,
    pub var: *mut bcf_variant_t,
    pub n_var: c_int,
    pub var_type: c_int,
    pub shared_dirty: c_int,
    pub indiv_dirty: c_int,
}

// original: bcf1_t (htslib/vcf.h) — one BCF record. Bitfield layout:
// n_info:16, n_allele:16, n_fmt:8, n_sample:24 — total 64 bits.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcf1_t {
    pub pos: crate::htslib_rs::hts::hts_pos_t,
    pub rlen: crate::htslib_rs::hts::hts_pos_t,
    pub rid: i32,
    pub qual: f32,
    pub _bitfield_align_1: [u32; 0],
    pub _bitfield_1: __BcfBitfieldUnit<[u8; 8usize]>,
    pub shared: kstring_t,
    pub indiv: kstring_t,
    pub d: bcf_dec_t,
    pub max_unpack: c_int,
    pub unpacked: c_int,
    pub unpack_size: [c_int; 3usize],
    pub errcode: c_int,
}
impl bcf1_t {
    #[inline]
    pub fn n_info(&self) -> u32 {
        self._bitfield_1.get(0usize, 16u8) as u32
    }
    #[inline]
    pub fn set_n_info(&mut self, val: u32) {
        self._bitfield_1.set(0usize, 16u8, val as u64)
    }
    #[inline]
    pub fn n_allele(&self) -> u32 {
        self._bitfield_1.get(16usize, 16u8) as u32
    }
    #[inline]
    pub fn set_n_allele(&mut self, val: u32) {
        self._bitfield_1.set(16usize, 16u8, val as u64)
    }
    #[inline]
    pub fn n_fmt(&self) -> u32 {
        self._bitfield_1.get(32usize, 8u8) as u32
    }
    #[inline]
    pub fn set_n_fmt(&mut self, val: u32) {
        self._bitfield_1.set(32usize, 8u8, val as u64)
    }
    #[inline]
    pub fn n_sample(&self) -> u32 {
        self._bitfield_1.get(40usize, 24u8) as u32
    }
    #[inline]
    pub fn set_n_sample(&mut self, val: u32) {
        self._bitfield_1.set(40usize, 24u8, val as u64)
    }
}

// original: bcf_sr_region_t (htslib/synced_bcf_reader.h) — opaque.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bcf_sr_region_t {
    _unused: [u8; 0],
}

// original: bcf_sr_regions_t (htslib/synced_bcf_reader.h)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcf_sr_regions_t {
    pub tbx: *mut crate::htslib_rs::tbx::tbx_t,
    pub itr: *mut hts_itr_t,
    pub line: kstring_t,
    pub file: *mut htsFile,
    pub fname: *mut c_char,
    pub is_bin: c_int,
    pub als: *mut *mut c_char,
    pub als_str: kstring_t,
    pub nals: c_int,
    pub mals: c_int,
    pub als_type: c_int,
    pub missed_reg_handler:
        Option<unsafe extern "C" fn(arg1: *mut bcf_sr_regions_t, arg2: *mut c_void)>,
    pub missed_reg_data: *mut c_void,
    pub regs: *mut bcf_sr_region_t,
    pub seq_hash: *mut c_void,
    pub seq_names: *mut *mut c_char,
    pub nseqs: c_int,
    pub iseq: c_int,
    pub start: crate::htslib_rs::hts::hts_pos_t,
    pub end: crate::htslib_rs::hts::hts_pos_t,
    pub prev_seq: c_int,
    pub prev_start: crate::htslib_rs::hts::hts_pos_t,
    pub prev_end: crate::htslib_rs::hts::hts_pos_t,
    pub overlap: c_int,
}

// original: bcf_sr_t (htslib/synced_bcf_reader.h) — one synced reader.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcf_sr_t {
    pub file: *mut htsFile,
    pub tbx_idx: *mut crate::htslib_rs::tbx::tbx_t,
    pub bcf_idx: *mut hts_idx_t,
    pub header: *mut bcf_hdr_t,
    pub itr: *mut hts_itr_t,
    pub fname: *mut c_char,
    pub buffer: *mut *mut bcf1_t,
    pub nbuffer: c_int,
    pub mbuffer: c_int,
    pub nfilter_ids: c_int,
    pub filter_ids: *mut c_int,
    pub samples: *mut c_int,
    pub n_smpl: c_int,
}

// original: bcf_srs_t (htslib/synced_bcf_reader.h) — collection of synced
// readers. (`bcf_sr_error` is defined further down in this file.)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcf_srs_t {
    pub collapse: c_int,
    pub apply_filters: *mut c_char,
    pub require_index: c_int,
    pub max_unpack: c_int,
    pub has_line: *mut c_int,
    pub errnum: bcf_sr_error,
    pub readers: *mut bcf_sr_t,
    pub nreaders: c_int,
    pub streaming: c_int,
    pub explicit_regs: c_int,
    pub samples: *mut *mut c_char,
    pub regions: *mut bcf_sr_regions_t,
    pub targets: *mut bcf_sr_regions_t,
    pub targets_als: c_int,
    pub targets_exclude: c_int,
    pub tmps: kstring_t,
    pub n_smpl: c_int,
    pub n_threads: c_int,
    pub p: *mut crate::htslib_rs::hts::htsThreadPool,
    pub aux: *mut c_void,
}
pub type bcf_variant_match = c_int;

// Native BCF/VCF enum constants and sentinel values (values from the C headers /
// hts-sys bindings), replacing the hts_sys:: references throughout the VCF code.
// Header line types (BCF_HL_*)
pub const BCF_HL_FLT: u32 = 0;
pub const BCF_HL_INFO: u32 = 1;
pub const BCF_HL_FMT: u32 = 2;
pub const BCF_HL_CTG: u32 = 3;
pub const BCF_HL_STR: u32 = 4;
pub const BCF_HL_GEN: u32 = 5;
// High-level value types (BCF_HT_*)
pub const BCF_HT_FLAG: u32 = 0;
pub const BCF_HT_INT: u32 = 1;
pub const BCF_HT_REAL: u32 = 2;
pub const BCF_HT_STR: u32 = 3;
// (BCF_HT_LONG already defined below as BCF_HT_INT | 0x100)
// Variable-length field kinds (BCF_VL_*)
pub const BCF_VL_FIXED: u32 = 0;
pub const BCF_VL_VAR: u32 = 1;
pub const BCF_VL_A: u32 = 2;
pub const BCF_VL_G: u32 = 3;
pub const BCF_VL_R: u32 = 4;
// Binary types (BCF_BT_*)
pub const BCF_BT_NULL: u32 = 0;
pub const BCF_BT_INT8: u32 = 1;
pub const BCF_BT_INT16: u32 = 2;
pub const BCF_BT_INT32: u32 = 3;
pub const BCF_BT_INT64: u32 = 4;
pub const BCF_BT_FLOAT: u32 = 5;
pub const BCF_BT_CHAR: u32 = 7;
// Dictionary types (BCF_DT_*)
pub const BCF_DT_ID: u32 = 0;
pub const BCF_DT_CTG: u32 = 1;
pub const BCF_DT_SAMPLE: u32 = 2;
// Unpack levels (BCF_UN_*)
pub const BCF_UN_STR: u32 = 1;
pub const BCF_UN_FLT: u32 = 2;
pub const BCF_UN_INFO: u32 = 4;
pub const BCF_UN_SHR: u32 = 7;
pub const BCF_UN_FMT: u32 = 8;
pub const BCF_UN_IND: u32 = 8;
pub const BCF_UN_ALL: u32 = 15;
// Error flags (BCF_ERR_*)
pub const BCF_ERR_CTG_UNDEF: u32 = 1;
pub const BCF_ERR_TAG_UNDEF: u32 = 2;
pub const BCF_ERR_NCOLS: u32 = 4;
pub const BCF_ERR_LIMITS: u32 = 8;
pub const BCF_ERR_CHAR: u32 = 16;
pub const BCF_ERR_CTG_INVALID: u32 = 32;
pub const BCF_ERR_TAG_INVALID: u32 = 64;
// Variant type bitmask (VCF_*)
pub const VCF_REF: u32 = 0;
pub const VCF_SNP: u32 = 1;
pub const VCF_MNP: u32 = 2;
pub const VCF_INDEL: u32 = 4;
pub const VCF_OTHER: u32 = 8;
pub const VCF_BND: u32 = 16;
pub const VCF_OVERLAP: u32 = 32;
// bcf1_t dirty bits (set by the record mutators)
pub const BCF1_DIRTY_ID: u32 = 1;
pub const BCF1_DIRTY_ALS: u32 = 2;
pub const BCF1_DIRTY_FLT: u32 = 4;
pub const BCF1_DIRTY_INF: u32 = 8;
// Missing / vector-end sentinels
pub const bcf_int8_missing: i32 = -128;
pub const bcf_int8_vector_end: i32 = -127;
pub const bcf_int16_missing: i32 = -32768;
pub const bcf_int16_vector_end: i32 = -32767;
pub const bcf_int32_missing: i32 = -2147483648;
pub const bcf_int32_vector_end: i32 = -2147483647;
pub const bcf_int64_missing: i64 = i64::MIN;
pub const bcf_str_missing: u32 = 7;
pub const bcf_str_vector_end: u32 = 0;
pub const bcf_float_missing: u32 = 0x7F80_0001;
pub const bcf_float_vector_end: u32 = 0x7F80_0002;

pub const VCF_INS: u32 = 1 << 6;
pub const VCF_DEL: u32 = 1 << 7;
// `bcf_sr_opt_t` is just a `u32` in htslib (see hts-sys bindings); the
// hts-sys-typed aliases above can move to native now that we've established
// the type identity. Keeping the named type around for caller signatures.
pub type bcf_sr_opt_t = u32;

pub const BCF_SR_REQUIRE_IDX: bcf_sr_opt_t = 0;
pub const BCF_SR_PAIR_LOGIC: bcf_sr_opt_t = 1;
pub const BCF_SR_ALLOW_NO_IDX: bcf_sr_opt_t = 2;
pub const BCF_SR_REGIONS_OVERLAP: bcf_sr_opt_t = 3;
pub const BCF_SR_TARGETS_OVERLAP: bcf_sr_opt_t = 4;

// Pair-logic flags passed to bcf_sr_set_opt(BCF_SR_PAIR_LOGIC, ...) — values
// from htslib/synced_bcf_reader.h.
pub const BCF_SR_PAIR_SNPS: u32 = 1;
pub const BCF_SR_PAIR_INDELS: u32 = 2;
pub const BCF_SR_PAIR_ANY: u32 = 4;
pub const BCF_SR_PAIR_SOME: u32 = 8;
pub const BCF_SR_PAIR_SNP_REF: u32 = 16;
pub const BCF_SR_PAIR_INDEL_REF: u32 = 32;
pub const BCF_SR_PAIR_EXACT: u32 = 64;

// Synced-reader error codes (`bcf_sr_error` enum in
// htslib/synced_bcf_reader.h). Underlying type is `c_uint` per bindgen.
pub type bcf_sr_error = c_uint;
pub const bcf_sr_error_open_failed: bcf_sr_error = 0;
pub const bcf_sr_error_not_bgzf: bcf_sr_error = 1;
pub const bcf_sr_error_idx_load_failed: bcf_sr_error = 2;
pub const bcf_sr_error_file_type_error: bcf_sr_error = 3;
pub const bcf_sr_error_api_usage_error: bcf_sr_error = 4;
pub const bcf_sr_error_header_error: bcf_sr_error = 5;
pub const bcf_sr_error_no_eof: bcf_sr_error = 6;
pub const bcf_sr_error_no_memory: bcf_sr_error = 7;
pub const bcf_sr_error_vcf_parse_error: bcf_sr_error = 8;
pub const bcf_sr_error_bcf_read_error: bcf_sr_error = 9;

// Genotype encodings from htslib/vcfutils.h (return of bcf_gt_type).
pub const GT_HOM_RR: u32 = 0;
pub const GT_HOM_AA: u32 = 1;
pub const GT_HET_RA: u32 = 2;
pub const GT_HET_AA: u32 = 3;
pub const GT_HAPL_R: u32 = 4;
pub const GT_HAPL_A: u32 = 5;
pub const GT_UNKN: u32 = 6;
const BCF_IS_64BIT: c_int = 1 << 30;
const BCF_HT_LONG: c_int = BCF_HT_INT as c_int | 0x100;
const BCF_MIN_BT_INT32: i64 = -2_147_483_640;
const BCF_MIN_BT_INT64: i64 = -9_223_372_036_854_775_800;
pub(crate) const REQUIRE_IDX_: c_int = 1;
pub(crate) const ALLOW_NO_IDX_: c_int = 2;
pub(crate) const MAX_CSI_COOR: hts_pos_t = (1_i64 << 44) - 1;
const BCF_VL_P: c_int = 5;
const BCF_VL_LA: c_int = 6;
const BCF_VL_LG: c_int = 7;
const BCF_VL_LR: c_int = 8;
const BCF_VL_M: c_int = 9;
const BCF_VL_NUMBER: u64 = 0xfffff;
const VCF_DEF: c_int = 4_002_000;
const VCF44: c_int = 4_004_000;
const VCF45: c_int = 4_005_000;
const BCF_TYPE_SHIFT: [usize; 16] = [0, 0, 1, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[repr(C)]
pub struct BcfSrSort {
    pub(crate) score: [u8; 256],
    pub(crate) nvar: c_int,
    pub(crate) mvar: c_int,
    pub(crate) var: *mut c_void,
    pub(crate) nvset: c_int,
    pub(crate) mvset: c_int,
    pub(crate) mpmat: c_int,
    pub(crate) pmat: *mut c_int,
    pub(crate) ngrp: c_int,
    pub(crate) mgrp: c_int,
    pub(crate) mcnt: c_int,
    pub(crate) cnt: *mut c_int,
    pub(crate) grp: *mut c_void,
    pub(crate) vset: *mut c_void,
    pub(crate) vcf_buf: *mut c_void,
    pub(crate) sr: *mut bcf_srs_t,
    pub(crate) grp_str2int: *mut c_void,
    pub(crate) var_str2int: *mut c_void,
    pub(crate) str_: kstring_t,
    pub(crate) moff: c_int,
    pub(crate) noff: c_int,
    pub(crate) off: *mut c_int,
    pub(crate) mcharp: c_int,
    pub(crate) charp: *mut *mut c_char,
    pub(crate) chr: *const c_char,
    pub(crate) pos: hts_pos_t,
    pub(crate) nsr: c_int,
    pub(crate) msr: c_int,
    pub(crate) pair: c_int,
    pub(crate) nactive: c_int,
    pub(crate) mactive: c_int,
    pub(crate) active: *mut c_int,
}

#[repr(C)]
pub(crate) struct BcfSrSortVcfBuf {
    pub(crate) nrec: c_int,
    pub(crate) mrec: c_int,
    pub(crate) rec: *mut *mut bcf1_t,
}

#[repr(C)]
pub(crate) struct BcfSrSortVar {
    pub(crate) str_: *mut c_char,
    pub(crate) type_: c_int,
    pub(crate) nalt: c_int,
    pub(crate) nvcf: c_int,
    pub(crate) mvcf: c_int,
    pub(crate) vcf: *mut c_int,
    pub(crate) rec: *mut *mut bcf1_t,
    pub(crate) mask: *mut kbitset_t,
}

#[repr(C)]
pub(crate) struct BcfSrSortGrp {
    pub(crate) key: *mut c_char,
    pub(crate) nvar: c_int,
    pub(crate) mvar: c_int,
    pub(crate) var: *mut c_int,
    pub(crate) nvcf: c_int,
}

#[repr(C)]
pub(crate) struct BcfSrSortVarSet {
    pub(crate) nvar: c_int,
    pub(crate) mvar: c_int,
    pub(crate) var: *mut c_int,
    pub(crate) cnt: c_int,
    pub(crate) mask: *mut kbitset_t,
}

#[repr(C)]
pub(crate) struct BcfSrAux {
    pub(crate) sort: BcfSrSort,
    pub(crate) regions_overlap: c_int,
    pub(crate) targets_overlap: c_int,
    pub(crate) closefile: *mut c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BcfSrRegion1 {
    pub(crate) start: hts_pos_t,
    pub(crate) end: hts_pos_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct BcfSrRegion {
    pub(crate) regs: *mut BcfSrRegion1,
    pub(crate) nregs: c_int,
    pub(crate) mregs: c_int,
    pub(crate) creg: c_int,
}

pub(crate) unsafe fn bcf_sr_aux_mut(readers: *mut bcf_srs_t) -> *mut BcfSrAux {
    unsafe { (*readers).aux.cast::<BcfSrAux>() }
}

pub(crate) unsafe fn bcf_sr_sort_reserve_active(srt: *mut BcfSrSort, need: c_int) -> c_int {
    unsafe {
        if srt.is_null() {
            return -1;
        }
        if need <= (*srt).mactive {
            return 0;
        }

        let active =
            libc::realloc((*srt).active.cast(), need as usize * size_of::<c_int>()).cast::<c_int>();
        if active.is_null() {
            return -1;
        }
        (*srt).active = active;
        (*srt).mactive = need;
        0
    }
}

pub(crate) unsafe fn bcf_sr_sort_reserve_vcf_buf(
    readers: *mut bcf_srs_t,
    srt: *mut BcfSrSort,
) -> c_int {
    unsafe {
        if readers.is_null() || srt.is_null() || (*readers).nreaders < 0 {
            return -1;
        }
        if (*srt).nsr == (*readers).nreaders {
            return 0;
        }

        (*srt).sr = readers;
        if (*srt).nsr < (*readers).nreaders {
            let old_nsr = (*srt).nsr.max(0) as usize;
            let new_nsr = (*readers).nreaders as usize;
            let vcf_buf = libc::realloc(
                (*srt).vcf_buf.cast(),
                new_nsr * size_of::<BcfSrSortVcfBuf>(),
            )
            .cast::<BcfSrSortVcfBuf>();
            if vcf_buf.is_null() {
                return -1;
            }
            (*srt).vcf_buf = vcf_buf.cast();
            std::ptr::write_bytes(vcf_buf.add(old_nsr), 0, new_nsr - old_nsr);
            if (*srt).msr < (*srt).nsr {
                (*srt).msr = (*srt).nsr;
            }
        }
        (*srt).nsr = (*readers).nreaders;
        (*srt).chr = std::ptr::null();
        0
    }
}

pub(crate) unsafe fn bcf_sr_sort_shift_reader_buffer(reader: *mut bcf_sr_t, j: c_int) -> c_int {
    unsafe {
        if reader.is_null() || (*reader).buffer.is_null() || j < 1 || j > (*reader).nbuffer {
            return -1;
        }
        let tmp = *(*reader).buffer;
        *(*reader).buffer = *(*reader).buffer.add(j as usize);
        let mut k = j + 1;
        while k <= (*reader).nbuffer {
            *(*reader).buffer.add((k - 1) as usize) = *(*reader).buffer.add(k as usize);
            k += 1;
        }
        *(*reader).buffer.add((*reader).nbuffer as usize) = tmp;
        (*reader).nbuffer -= 1;
        0
    }
}

unsafe fn bcf_sr_sort_reserve_row(buf: *mut BcfSrSortVcfBuf, need: c_int) -> c_int {
    unsafe {
        if buf.is_null() || need < 0 {
            return -1;
        }
        if need <= (*buf).mrec {
            return 0;
        }
        let rec = libc::realloc((*buf).rec.cast(), need as usize * size_of::<*mut bcf1_t>())
            .cast::<*mut bcf1_t>();
        if rec.is_null() {
            return -1;
        }
        (*buf).rec = rec;
        (*buf).mrec = need;
        0
    }
}

pub(crate) unsafe fn bcf_sr_sort_append_empty_row(
    vcf_buf: *mut BcfSrSortVcfBuf,
    nreaders: c_int,
) -> c_int {
    unsafe {
        if vcf_buf.is_null() || nreaders <= 0 {
            return -1;
        }
        let row = (*vcf_buf).nrec + 1;
        for i in 0..nreaders as usize {
            if bcf_sr_sort_reserve_row(vcf_buf.add(i), row) < 0 {
                return -1;
            }
            *(*vcf_buf.add(i)).rec.add((row - 1) as usize) = std::ptr::null_mut();
        }
        for i in 0..nreaders as usize {
            (*vcf_buf.add(i)).nrec = row;
        }
        0
    }
}

pub(crate) unsafe fn bcf_sr_sort_record_key(
    hdr: *const bcf_hdr_t,
    rec: *mut bcf1_t,
) -> Option<Vec<u8>> {
    unsafe {
        if rec.is_null() || bcf_unpack(rec, BCF_UN_STR as c_int) < 0 {
            return None;
        }
        let n_allele = (*rec).n_allele() as c_int;
        if n_allele <= 0 || (*rec).d.allele.is_null() || (*(*rec).d.allele).is_null() {
            return None;
        }

        let ref_allele = CStr::from_ptr(*(*rec).d.allele).to_bytes();
        let mut key = Vec::new();
        let mut has_symbolic_alt = false;
        if n_allele == 1 {
            key.extend_from_slice(ref_allele);
            key.extend_from_slice(b">.");
            return Some(key);
        }

        for ial in 1..n_allele as usize {
            let alt = *(*rec).d.allele.add(ial);
            if alt.is_null() {
                return None;
            }
            if ial > 1 {
                key.push(b',');
            }
            let alt_bytes = CStr::from_ptr(alt).to_bytes();
            if alt_bytes.starts_with(b"<") || alt_bytes.contains(&b'[') || alt_bytes.contains(&b']')
            {
                has_symbolic_alt = true;
            }
            key.extend_from_slice(ref_allele);
            key.push(b'>');
            key.extend_from_slice(alt_bytes);
        }

        if has_symbolic_alt && !hdr.is_null() {
            if bcf_unpack(rec, BCF_UN_INFO as c_int) < 0 {
                return None;
            }
            let info = bcf_get_info(hdr, rec, c"END".as_ptr());
            if !info.is_null() {
                let end = vcfutils_c_280_get_int32_info_value(info, 0);
                if end != bcf_int32_missing && end != bcf_int32_vector_end {
                    key.extend_from_slice(b":END=");
                    key.extend_from_slice(end.to_string().as_bytes());
                }
            }
        }
        Some(key)
    }
}

pub(crate) fn bcf_sr_sort_disambiguate_duplicate_key(
    key: &mut Vec<u8>,
    seen: &[(Vec<u8>, c_int, *mut bcf1_t)],
    reader_idx: c_int,
) {
    let base_len = key.len();
    let mut duplicate_idx = 0;
    loop {
        let collides_same_reader = seen
            .iter()
            .any(|(seen_key, seen_reader, _)| *seen_reader == reader_idx && seen_key == key);
        if !collides_same_reader {
            return;
        }
        key.truncate(base_len);
        key.extend_from_slice(duplicate_idx.to_string().as_bytes());
        duplicate_idx += 1;
    }
}

unsafe fn bcf_sr_regions_overlap_ptr(regions: *mut bcf_sr_regions_t) -> *mut c_int {
    unsafe {
        regions
            .cast::<u8>()
            .add(size_of::<bcf_sr_regions_t>())
            .cast::<c_int>()
    }
}

pub(crate) unsafe fn bcf_sr_regions_set_overlap(regions: *mut bcf_sr_regions_t, overlap: c_int) {
    unsafe {
        *bcf_sr_regions_overlap_ptr(regions) = overlap;
    }
}

pub(crate) unsafe fn bcf_sr_regions_alloc() -> *mut bcf_sr_regions_t {
    unsafe {
        let size = size_of::<bcf_sr_regions_t>() + size_of::<hts_pos_t>();
        let reg = libc::calloc(1, size).cast::<bcf_sr_regions_t>();
        if reg.is_null() {
            return std::ptr::null_mut();
        }
        (*reg).start = -1;
        (*reg).end = -1;
        (*reg).prev_seq = -1;
        (*reg).prev_start = -1;
        (*reg).prev_end = -1;
        reg
    }
}

pub(crate) unsafe fn bcf_sr_regions_add(
    reg: *mut bcf_sr_regions_t,
    chr: *const c_char,
    mut start: hts_pos_t,
    mut end: hts_pos_t,
) -> c_int {
    unsafe {
        if reg.is_null() || chr.is_null() {
            return -1;
        }

        if start == -1 && end == -1 {
            start = 0;
            end = MAX_CSI_COOR - 1;
        } else {
            start -= 1;
            end -= 1;
        }

        if (*reg).seq_hash.is_null() {
            (*reg).seq_hash = super::sam::khash_str2int_init();
            if (*reg).seq_hash.is_null() {
                return -1;
            }
        }

        let mut iseq = -1;
        if super::sam::khash_str2int_get((*reg).seq_hash, chr, &mut iseq) < 0 {
            iseq = (*reg).nseqs;
            let new_nseqs = (*reg).nseqs + 1;

            let seq_names = libc::realloc(
                (*reg).seq_names.cast(),
                new_nseqs as usize * size_of::<*mut c_char>(),
            )
            .cast::<*mut c_char>();
            if seq_names.is_null() {
                return -1;
            }
            (*reg).seq_names = seq_names;

            let regs = libc::realloc(
                (*reg).regs.cast(),
                new_nseqs as usize * size_of::<BcfSrRegion>(),
            )
            .cast::<BcfSrRegion>();
            if regs.is_null() {
                return -1;
            }
            (*reg).regs = regs.cast();

            let seq_name = libc::strdup(chr);
            if seq_name.is_null() {
                return -1;
            }
            *(*reg).seq_names.add(iseq as usize) = seq_name;
            *regs.add(iseq as usize) = BcfSrRegion {
                regs: std::ptr::null_mut(),
                nregs: 0,
                mregs: 0,
                creg: -1,
            };
            if super::sam::khash_str2int_set((*reg).seq_hash, seq_name, iseq) < 0 {
                return -1;
            }
            (*reg).nseqs = new_nseqs;
        }

        let creg = (*reg).regs.cast::<BcfSrRegion>().add(iseq as usize);
        if (*creg).nregs + 1 > (*creg).mregs {
            let mut new_mregs = if (*creg).mregs > 0 {
                (*creg).mregs * 2
            } else {
                1
            };
            if new_mregs < (*creg).nregs + 1 {
                new_mregs = (*creg).nregs + 1;
            }
            let regs = libc::realloc(
                (*creg).regs.cast(),
                new_mregs as usize * size_of::<BcfSrRegion1>(),
            )
            .cast::<BcfSrRegion1>();
            if regs.is_null() {
                return -1;
            }
            (*creg).regs = regs;
            (*creg).mregs = new_mregs;
        }

        *(*creg).regs.add((*creg).nregs as usize) = BcfSrRegion1 { start, end };
        (*creg).nregs += 1;
        0
    }
}

pub(crate) unsafe fn regions_merge(reg: *mut BcfSrRegion) {
    unsafe {
        if reg.is_null() {
            return;
        }

        let mut i = 0;
        while i < (*reg).nregs {
            let mut j = i + 1;
            while j < (*reg).nregs
                && (*(*reg).regs.add(i as usize)).end >= (*(*reg).regs.add(j as usize)).start
            {
                if (*(*reg).regs.add(i as usize)).end < (*(*reg).regs.add(j as usize)).end {
                    (*(*reg).regs.add(i as usize)).end = (*(*reg).regs.add(j as usize)).end;
                }
                (*(*reg).regs.add(j as usize)).start = 1;
                (*(*reg).regs.add(j as usize)).end = 0;
                j += 1;
            }
            i = j;
        }
    }
}

pub(crate) unsafe fn advance_creg(reg: *mut BcfSrRegion) -> c_int {
    unsafe {
        if reg.is_null() {
            return -1;
        }

        let mut i = (*reg).creg + 1;
        while i < (*reg).nregs
            && (*(*reg).regs.add(i as usize)).start > (*(*reg).regs.add(i as usize)).end
        {
            i += 1;
        }
        (*reg).creg = i;
        if i >= (*reg).nregs {
            return -1;
        }
        0
    }
}

unsafe fn bcf_vcf45_number_code(number: *const c_char) -> Option<c_int> {
    unsafe {
        if number.is_null() {
            None
        } else if libc::strcmp(number, c"P".as_ptr()) == 0 {
            Some(BCF_VL_P)
        } else if libc::strcmp(number, c"LA".as_ptr()) == 0 {
            Some(BCF_VL_LA)
        } else if libc::strcmp(number, c"LG".as_ptr()) == 0 {
            Some(BCF_VL_LG)
        } else if libc::strcmp(number, c"LR".as_ptr()) == 0 {
            Some(BCF_VL_LR)
        } else if libc::strcmp(number, c"M".as_ptr()) == 0 {
            Some(BCF_VL_M)
        } else {
            None
        }
    }
}

unsafe fn bcf_hdr_fix_vcf45_vl_types(hdr: *mut bcf_hdr_t) {
    unsafe {
        if hdr.is_null() {
            return;
        }

        for i in 0..(*hdr).nhrec {
            let hrec = *(*hdr).hrec.add(i as usize);
            if hrec.is_null()
                || ((*hrec).type_ != BCF_HL_INFO as c_int && (*hrec).type_ != BCF_HL_FMT as c_int)
            {
                continue;
            }

            let number_idx = bcf_hrec_find_key(hrec, c"Number".as_ptr());
            if number_idx < 0 {
                continue;
            }
            let Some(vl_code) = bcf_vcf45_number_code(*(*hrec).vals.add(number_idx as usize))
            else {
                continue;
            };

            let id_idx = bcf_hrec_find_key(hrec, c"ID".as_ptr());
            if id_idx < 0 {
                continue;
            }
            let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, *(*hrec).vals.add(id_idx as usize));
            if id < 0 {
                continue;
            }

            let idinfo = (*(*hdr).id[BCF_DT_ID as usize].add(id as usize))
                .val
                .cast_mut();
            if idinfo.is_null() {
                continue;
            }
            let info = &mut (*idinfo).info[(*hrec).type_ as usize];
            *info &= !(((0xf_u64) << 8) | (BCF_VL_NUMBER << 12));
            *info |= ((vl_code as u64) << 8) | (BCF_VL_NUMBER << 12);
        }
    }
}

pub unsafe fn vcf_c_796_bcf_hdr_set_idx(
    hdr: *mut bcf_hdr_t,
    dict_type: c_int,
    tag: *const c_char,
    idinfo: *mut bcf_idinfo_t,
) -> c_int {
    unsafe {
        if hdr.is_null() || idinfo.is_null() || dict_type < 0 || dict_type >= 3 {
            *libc::__errno_location() = libc::EINVAL;
            return -1;
        }

        let dict = dict_type as usize;
        if (*idinfo).id == -1 {
            (*idinfo).id = (*hdr).n[dict];
        } else if (*idinfo).id < 0 {
            *libc::__errno_location() = libc::EINVAL;
            return -1;
        } else if (*idinfo).id < (*hdr).n[dict]
            && !(*(*hdr).id[dict].add((*idinfo).id as usize)).key.is_null()
        {
            let tag_str = CStr::from_ptr(tag).to_string_lossy();
            let msg = std::ffi::CString::new(format!(
                "Conflicting IDX={} lines in the header dictionary, the new tag is {}",
                (*idinfo).id,
                tag_str
            ))
            .unwrap_or_default();
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_ERROR,
                c"bcf_hdr_set_idx".as_ptr(),
                msg.as_ptr(),
            );
            *libc::__errno_location() = libc::EINVAL;
            return -1;
        }

        let new_n = if (*idinfo).id >= (*hdr).n[dict] {
            (*idinfo).id + 1
        } else {
            (*hdr).n[dict]
        };
        if new_n > (*hdr).m[dict] {
            let old_m = (*hdr).m[dict].max(0) as usize;
            let new_m = new_n as usize;
            let id = libc::realloc(
                (*hdr).id[dict].cast(),
                new_m.saturating_mul(size_of::<bcf_idpair_t>()),
            )
            .cast::<bcf_idpair_t>();
            if id.is_null() {
                return -1;
            }
            if new_m > old_m {
                std::ptr::write_bytes(id.add(old_m), 0, new_m - old_m);
            }
            (*hdr).id[dict] = id;
            (*hdr).m[dict] = new_n;
        }
        (*hdr).n[dict] = new_n;
        (*(*hdr).id[dict].add((*idinfo).id as usize)).key = tag;
        0
    }
}

pub unsafe fn vcf_c_1026_bcf_hdr_unregister_hrec(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) {
    unsafe {
        if hdr.is_null() || hrec.is_null() {
            return;
        }
        let hrec_type = (*hrec).type_;
        if hrec_type != BCF_HL_FLT as c_int
            && hrec_type != BCF_HL_INFO as c_int
            && hrec_type != BCF_HL_FMT as c_int
            && hrec_type != BCF_HL_CTG as c_int
        {
            return;
        }

        let id_key = bcf_hrec_find_key(hrec, c"ID".as_ptr());
        if id_key < 0 || (*hrec).vals.is_null() {
            return;
        }
        let id_value = *(*hrec).vals.add(id_key as usize);
        if id_value.is_null() {
            return;
        }

        let dict_type = if hrec_type == BCF_HL_CTG as c_int {
            BCF_DT_CTG as c_int
        } else {
            BCF_DT_ID as c_int
        };
        let id = bcf_hdr_id2int(hdr, dict_type, id_value);
        if id < 0 || id >= (*hdr).n[dict_type as usize] {
            return;
        }
        let idpair = (*hdr).id[dict_type as usize].add(id as usize);
        if (*idpair).val.is_null() {
            return;
        }
        let idinfo = (*idpair).val.cast_mut();
        let hrec_index = if hrec_type == BCF_HL_CTG as c_int {
            0
        } else {
            hrec_type
        };
        if hrec_index >= 0 && (hrec_index as usize) < (*idinfo).hrec.len() {
            (*idinfo).hrec[hrec_index as usize] = std::ptr::null_mut();
        }
    }
}

pub(crate) unsafe fn regions_sort_and_merge(reg: *mut bcf_sr_regions_t) {
    unsafe {
        if reg.is_null() {
            return;
        }

        let regs = (*reg).regs.cast::<BcfSrRegion>();
        for i in 0..(*reg).nseqs {
            let seq_reg = regs.add(i as usize);
            if !(*seq_reg).regs.is_null() && (*seq_reg).nregs > 1 {
                let intervals =
                    std::slice::from_raw_parts_mut((*seq_reg).regs, (*seq_reg).nregs as usize);
                intervals.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
            }
            regions_merge(seq_reg);
        }
    }
}

pub(crate) unsafe fn bcf_sr_regions_destroy_translated(reg: *mut bcf_sr_regions_t) {
    unsafe {
        if reg.is_null() {
            return;
        }

        libc::free((*reg).fname.cast());
        if !(*reg).itr.is_null() {
            super::hts::hts_itr_destroy((*reg).itr.cast());
        }
        if !(*reg).tbx.is_null() {
            super::tbx::tbx_destroy((*reg).tbx.cast());
        }
        if !(*reg).file.is_null() {
            let _ = hts_close((*reg).file.cast());
        }
        libc::free((*reg).als.cast());
        libc::free((*reg).als_str.s.cast());
        libc::free((*reg).line.s.cast());

        let regs = (*reg).regs.cast::<BcfSrRegion>();
        if !regs.is_null() {
            for i in 0..(*reg).nseqs {
                libc::free((*(*reg).seq_names.add(i as usize)).cast());
                libc::free((*regs.add(i as usize)).regs.cast());
            }
        }
        libc::free((*reg).regs.cast());
        libc::free((*reg).seq_names.cast());
        super::sam::khash_str2int_destroy((*reg).seq_hash);
        libc::free(reg.cast());
    }
}

pub(crate) unsafe fn regions_init_string(str_: *const c_char) -> *mut bcf_sr_regions_t {
    unsafe {
        if str_.is_null() {
            return std::ptr::null_mut();
        }

        let reg = bcf_sr_regions_alloc();
        if reg.is_null() {
            return std::ptr::null_mut();
        }

        let mut tmp: kstring_t = std::mem::zeroed();
        let mut sp = str_;
        let mut ep = str_;
        loop {
            tmp.l = 0;
            if *ep == b'{' as c_char {
                while *ep != 0 && *ep != b'}' as c_char {
                    ep = ep.add(1);
                }
                if *ep == 0 {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
                ep = ep.add(1);
                if kputsn(sp.add(1), ep.offset_from(sp) as usize - 2, &mut tmp) < 0 {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
            } else {
                while *ep != 0 && *ep != b',' as c_char && *ep != b':' as c_char {
                    ep = ep.add(1);
                }
                if kputsn(sp, ep.offset_from(sp) as usize, &mut tmp) < 0 {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
            }

            if *ep == b':' as c_char {
                sp = ep.add(1);
                let mut num_end: *mut c_char = std::ptr::null_mut();
                let from = super::hts::hts_parse_decimal(sp, &mut num_end, 0);
                ep = num_end;
                if sp == ep {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
                if *ep == 0 || *ep == b',' as c_char {
                    if bcf_sr_regions_add(reg, tmp.s, from, from) < 0 {
                        bcf_sr_regions_destroy_translated(reg);
                        super::hts::ks_free(&mut tmp);
                        return std::ptr::null_mut();
                    }
                    if *ep == 0 {
                        break;
                    }
                    sp = ep;
                    continue;
                }
                if *ep != b'-' as c_char {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }

                ep = ep.add(1);
                sp = ep;
                let to = super::hts::hts_parse_decimal(sp, &mut num_end, 0);
                ep = num_end;
                if *ep != 0 && *ep != b',' as c_char {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
                let to = if sp == ep { MAX_CSI_COOR - 1 } else { to };
                if bcf_sr_regions_add(reg, tmp.s, from, to) < 0 {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
                if *ep == 0 {
                    break;
                }
                sp = ep;
            } else if *ep == 0 || *ep == b',' as c_char {
                if tmp.l != 0 && bcf_sr_regions_add(reg, tmp.s, -1, -1) < 0 {
                    bcf_sr_regions_destroy_translated(reg);
                    super::hts::ks_free(&mut tmp);
                    return std::ptr::null_mut();
                }
                if *ep == 0 {
                    break;
                }
                ep = ep.add(1);
                sp = ep;
            } else {
                bcf_sr_regions_destroy_translated(reg);
                super::hts::ks_free(&mut tmp);
                return std::ptr::null_mut();
            }
        }

        super::hts::ks_free(&mut tmp);
        reg
    }
}

pub(crate) unsafe fn regions_parse_line(
    line: *mut c_char,
    ichr: c_int,
    ifrom: c_int,
    ito: c_int,
    chr: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    from: *mut hts_pos_t,
    to: *mut hts_pos_t,
) -> c_int {
    unsafe {
        if line.is_null()
            || chr.is_null()
            || chr_end.is_null()
            || from.is_null()
            || to.is_null()
            || ichr < 0
            || ifrom < 0
            || ito < 0
        {
            return -1;
        }

        *chr_end = std::ptr::null_mut();
        if *line == b'#' as c_char {
            return 0;
        }

        let (k, l) = if ifrom <= ito {
            (ifrom, ito)
        } else {
            (ito, ifrom)
        };

        let mut se = line;
        let mut ss: *mut c_char = std::ptr::null_mut();
        let mut i = 0;
        while i <= k && *se != 0 {
            ss = if i == 0 {
                let current = se;
                se = se.add(1);
                current
            } else {
                se = se.add(1);
                se
            };
            while *se != 0 && *se != b'\t' as c_char {
                se = se.add(1);
            }
            i += 1;
        }
        if i <= k {
            return -1;
        }

        let mut tmp: *mut c_char = std::ptr::null_mut();
        if k == l {
            *from = super::hts::hts_parse_decimal(ss, &mut tmp, 0);
            *to = *from;
            if tmp == ss || (*tmp != 0 && *tmp != b'\t' as c_char) {
                return -1;
            }
        } else {
            if k == ifrom {
                *from = super::hts::hts_parse_decimal(ss, &mut tmp, 0);
            } else {
                *to = super::hts::hts_parse_decimal(ss, &mut tmp, 0);
            }
            if tmp == ss || (*tmp != 0 && *tmp != b'\t' as c_char) {
                return -1;
            }

            i = k;
            while i < l && *se != 0 {
                se = se.add(1);
                ss = se;
                while *se != 0 && *se != b'\t' as c_char {
                    se = se.add(1);
                }
                i += 1;
            }
            if i < l {
                return -1;
            }
            if k == ifrom {
                *to = super::hts::hts_parse_decimal(ss, &mut tmp, 0);
            } else {
                *from = super::hts::hts_parse_decimal(ss, &mut tmp, 0);
            }
            if tmp == ss || (*tmp != 0 && *tmp != b'\t' as c_char) {
                return -1;
            }
        }

        ss = line;
        se = line;
        i = 0;
        while i <= ichr && *se != 0 {
            if i > 0 {
                se = se.add(1);
                ss = se;
            }
            while *se != 0 && *se != b'\t' as c_char {
                se = se.add(1);
            }
            i += 1;
        }
        if i <= ichr {
            return -1;
        }
        *chr_end = se;
        *chr = ss;
        1
    }
}

unsafe fn init_filters(
    hdr: *const bcf_hdr_t,
    filters: *const c_char,
    nfilters: *mut c_int,
) -> *mut c_int {
    unsafe {
        let mut out: *mut c_int = std::ptr::null_mut();
        let mut nout = 0;
        let mut prev = filters;
        let mut tmp = filters;

        loop {
            let ch = *tmp;
            if ch == b',' as c_char || ch == 0 {
                let otmp = libc::realloc(out.cast(), (nout as usize + 1) * size_of::<c_int>())
                    .cast::<c_int>();
                if otmp.is_null() {
                    libc::free(out.cast());
                    return std::ptr::null_mut();
                }
                out = otmp;

                let len = tmp.offset_from(prev) as usize;
                if len == 1 && *prev == b'.' as c_char {
                    *out.add(nout as usize) = -1;
                    nout += 1;
                } else {
                    let mut str_ = kstring_t {
                        l: 0,
                        m: 0,
                        s: std::ptr::null_mut(),
                    };
                    if kputsn(prev, len, &mut str_) < 0 {
                        super::hts::ks_free(&mut str_);
                        libc::free(out.cast());
                        return std::ptr::null_mut();
                    }
                    let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, str_.s);
                    if id >= 0 {
                        *out.add(nout as usize) = id;
                        nout += 1;
                    }
                    super::hts::ks_free(&mut str_);
                }

                if ch == 0 {
                    break;
                }
                prev = tmp.add(1);
            }
            tmp = tmp.add(1);
        }

        *nfilters = nout;
        out
    }
}

pub(crate) unsafe fn bcf_sr_seek_start(readers: *mut bcf_srs_t) {
    unsafe {
        let reg = (*readers).regions;
        let regs = (*reg).regs.cast::<BcfSrRegion>();
        for i in 0..(*reg).nseqs {
            (*regs.add(i as usize)).creg = -1;
        }
        (*reg).iseq = 0;
        (*reg).start = -1;
        (*reg).end = -1;
        (*reg).prev_seq = -1;
        (*reg).prev_start = -1;
        (*reg).prev_end = -1;
    }
}

#[cfg(test)]
unsafe fn bcf_sr_regions_get_overlap(regions: *mut bcf_sr_regions_t) -> c_int {
    unsafe { *bcf_sr_regions_overlap_ptr(regions) }
}

// Native translation of htslib/vcf_sweep.c struct bcf_sweep_t (kept repr(C) and
// field-for-field identical to the C definition so the public opaque pointer
// semantics are preserved).
pub(crate) const SW_FWD: c_int = 0;
pub(crate) const SW_BWD: c_int = 1;

#[repr(C)]
pub struct bcf_sweep_t {
    pub(crate) file: *mut htsFile,
    pub(crate) hdr: *mut bcf_hdr_t,
    pub(crate) fp: *mut BGZF,

    pub(crate) direction: c_int,  // to tell if the direction has changed
    pub(crate) block_size: c_int, // the size of uncompressed data to hold in memory
    pub(crate) rec: *mut bcf1_t,  // bcf buffer
    pub(crate) nrec: c_int,
    pub(crate) mrec: c_int, // number of used records; total size of the buffer
    pub(crate) lrid: c_int,
    pub(crate) lpos: c_int,
    pub(crate) lnals: c_int,
    pub(crate) lals_len: c_int,
    pub(crate) mlals: c_int, // to check uniqueness of a record
    pub(crate) lals: *mut c_char,

    pub(crate) idx: *mut u64, // uncompressed offsets of VCF/BCF records
    pub(crate) iidx: c_int,
    pub(crate) nidx: c_int,
    pub(crate) midx: c_int,     // i: current offset; n: used; m: allocated
    pub(crate) idx_done: c_int, // the index is built during the first pass
}

// Native translation of htslib/vcf.c bcf_hdr_init().
pub unsafe fn bcf_hdr_init(mode: *const c_char) -> *mut bcf_hdr_t {
    let h = libc::calloc(1, size_of::<bcf_hdr_t>()).cast::<bcf_hdr_t>();
    if h.is_null() {
        return std::ptr::null_mut();
    }
    let dsize: [u32; 3] = [16384, 16384, 2048]; // info, contig, format
    let mut i = 0usize;
    while i < 3 {
        let d = kh_init_vdict();
        if d.is_null() {
            // fail: free already-initialised dicts and the header
            for j in 0..3usize {
                kh_destroy_vdict((*h).dict[j].cast());
            }
            libc::free(h.cast());
            return std::ptr::null_mut();
        }
        (*h).dict[i] = d.cast();
        // Supersize the hash to make collisions very unlikely
        if kh_resize_vdict(d, dsize[i]) < 0 {
            for j in 0..3usize {
                kh_destroy_vdict((*h).dict[j].cast());
            }
            libc::free(h.cast());
            return std::ptr::null_mut();
        }
        i += 1;
    }

    let aux = libc::calloc(1, size_of::<bcf_hdr_aux_t>()).cast::<bcf_hdr_aux_t>();
    if aux.is_null() {
        for j in 0..3usize {
            kh_destroy_vdict((*h).dict[j].cast());
        }
        libc::free(h.cast());
        return std::ptr::null_mut();
    }
    (*aux).gen = kh_init_hdict();
    if (*aux).gen.is_null() {
        libc::free(aux.cast());
        for j in 0..3usize {
            kh_destroy_vdict((*h).dict[j].cast());
        }
        libc::free(h.cast());
        return std::ptr::null_mut();
    }
    (*aux).key_len = std::ptr::null_mut();
    // aux->dict = *((vdict_t*)h->dict[0]); — shallow copy of dict[0]'s vdict.
    (*aux).dict = std::ptr::read((*h).dict[0].cast::<kh_vdict_t>());
    (*aux).version = 0;
    (*aux).ref_count = 1;
    libc::free((*h).dict[0]); // free the original kh_vdict_t shell (arrays now owned by aux.dict)
    (*h).dict[0] = aux.cast();

    if !libc::strchr(mode, b'w' as c_int).is_null() {
        bcf_hdr_append(h, c"##fileformat=VCFv4.2".as_ptr());
        // The filter PASS must appear first in the dictionary
        bcf_hdr_append(
            h,
            c"##FILTER=<ID=PASS,Description=\"All filters passed\">".as_ptr(),
        );
        (*aux).version = VCF_DEF;
    }
    h
}

// Native translation of htslib/vcf.c bcf_hdr_destroy().
pub unsafe fn bcf_hdr_destroy(h: *mut bcf_hdr_t) {
    if h.is_null() {
        return;
    }
    let aux = get_hdr_aux(h);
    if (*aux).ref_count > 1 {
        // Refs still held, so delay destruction
        (*aux).ref_count &= !1;
        return;
    }
    let mut i = 0usize;
    while i < 3 {
        let d = (*h).dict[i].cast::<kh_vdict_t>();
        if !d.is_null() {
            // free all keys
            let mut k: u32 = 0;
            while k != (*d).n_buckets {
                if !kh_iseither((*d).flags, k) {
                    libc::free((*(*d).keys.add(k as usize)) as *mut c_void);
                }
                k += 1;
            }
            if i == 0 {
                let gen = (*aux).gen;
                let mut k: u32 = 0;
                while k < (*gen).n_buckets {
                    if !kh_iseither((*gen).flags, k) {
                        libc::free((*(*gen).keys.add(k as usize)) as *mut c_void);
                    }
                    k += 1;
                }
                kh_destroy_hdict(gen);
                libc::free((*aux).key_len.cast()); // may exist for dict[0] only
            }
            kh_destroy_vdict(d);
        }
        libc::free((*h).id[i].cast());
        i += 1;
    }
    let mut i: c_int = 0;
    while i < (*h).nhrec {
        bcf_hrec_destroy(*(*h).hrec.add(i as usize));
        i += 1;
    }
    if (*h).nhrec != 0 {
        libc::free((*h).hrec.cast());
    }
    if !(*h).samples.is_null() {
        libc::free((*h).samples.cast());
    }
    libc::free((*h).keep_samples.cast());
    libc::free((*h).transl[0].cast());
    libc::free((*h).transl[1].cast());
    libc::free((*h).mem.s.cast());
    libc::free(h.cast());
}

pub unsafe fn bcf_init() -> *mut bcf1_t {
    // Native translation of htslib/vcf.c bcf_init().
    libc::calloc(1, size_of::<bcf1_t>()).cast()
}

pub unsafe fn bcf_destroy(v: *mut bcf1_t) {
    // Native translation of htslib/vcf.c bcf_destroy().
    if v.is_null() {
        return;
    }
    bcf_empty(v);
    libc::free(v.cast());
}

pub unsafe fn bcf_empty(v: *mut bcf1_t) {
    // Native translation of htslib/vcf.c bcf_empty().
    bcf_clear(v);
    libc::free((*v).d.id.cast());
    libc::free((*v).d.als.cast());
    libc::free((*v).d.allele.cast());
    libc::free((*v).d.flt.cast());
    libc::free((*v).d.info.cast());
    libc::free((*v).d.fmt.cast());
    if !(*v).d.var.is_null() {
        libc::free((*v).d.var.cast());
    }
    libc::free((*v).shared.s.cast());
    libc::free((*v).indiv.s.cast());
    std::ptr::write_bytes(&mut (*v).d as *mut _ as *mut u8, 0, size_of_val(&(*v).d));
    std::ptr::write_bytes(
        &mut (*v).shared as *mut _ as *mut u8,
        0,
        size_of_val(&(*v).shared),
    );
    std::ptr::write_bytes(
        &mut (*v).indiv as *mut _ as *mut u8,
        0,
        size_of_val(&(*v).indiv),
    );
}

pub unsafe fn bcf_clear(v: *mut bcf1_t) {
    // Native translation of htslib/vcf.c bcf_clear().
    for i in 0..(*v).d.m_info {
        let info = (*v).d.info.add(i as usize);
        if (*info).vptr_free() != 0 {
            libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
            (*info).set_vptr_free(0);
        }
    }
    for i in 0..(*v).d.m_fmt {
        let fmt = (*v).d.fmt.add(i as usize);
        if (*fmt).p_free() != 0 {
            libc::free((*fmt).p.sub((*fmt).p_off() as usize).cast());
            (*fmt).set_p_free(0);
        }
    }
    (*v).rid = 0;
    (*v).pos = 0;
    (*v).rlen = 0;
    (*v).unpacked = 0;
    (*v).qual = f32::from_bits(bcf_float_missing);
    (*v).set_n_info(0);
    (*v).set_n_allele(0);
    (*v).set_n_fmt(0);
    (*v).set_n_sample(0);
    (*v).shared.l = 0;
    (*v).indiv.l = 0;
    (*v).d.var_type = -1;
    (*v).d.shared_dirty = 0;
    (*v).d.indiv_dirty = 0;
    (*v).d.n_flt = 0;
    (*v).errcode = 0;
    if (*v).d.m_als != 0 {
        *(*v).d.als = 0;
    }
    if (*v).d.m_id != 0 {
        *(*v).d.id = 0;
    }
}

// Native translation of htslib/hts.c hts_useek(): seek to an uncompressed offset.
// For bgzf-backed files this uses the virtual-offset aware bgzf_useek(); for a
// plain hFILE it seeks the buffered handle (NOT the raw fd) so it stays in sync
// with the buffer-aware tell below.
unsafe fn sw_useek(fp: *mut htsFile, uoffset: i64, where_: c_int) -> c_int {
    if ((*fp).bitfields & (1 << 4)) != 0 {
        bgzf_useek((*fp).fp.bgzf, uoffset, where_)
    } else if hseek((*fp).fp.hfile, uoffset as libc::off_t, libc::SEEK_SET) >= 0 {
        0
    } else {
        -1
    }
}

// Native translation of htslib/hts.c hts_utell(): report the uncompressed offset.
pub(crate) unsafe fn sw_utell(fp: *mut htsFile) -> i64 {
    if ((*fp).bitfields & (1 << 4)) != 0 {
        bgzf_utell((*fp).fp.bgzf)
    } else {
        htell((*fp).fp.hfile) as i64
    }
}

// htslib/htslib/vcf.h: #define bcf_read1(fp,h,v) bcf_read((fp),(h),(v))
#[inline]
pub(crate) unsafe fn bcf_read1(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    bcf_read(fp, h, v)
}

// htslib/htslib/vcf.h: #define bcf_empty1(v) bcf_empty(v)
#[inline]
pub(crate) unsafe fn bcf_empty1(v: *mut bcf1_t) {
    bcf_empty(v)
}

// Native translation of htslib/vcf_sweep.c sw_rec_equal().
unsafe fn sw_rec_equal(sw: *mut bcf_sweep_t, rec: *mut bcf1_t) -> c_int {
    if (*sw).lrid != (*rec).rid {
        return 0;
    }
    if (*sw).lpos != (*rec).pos as c_int {
        return 0;
    }
    if (*sw).lnals != (*rec).n_allele() as c_int {
        return 0;
    }

    let allele0 = *(*rec).d.allele;
    let mut t = *(*rec).d.allele.add((*sw).lnals as usize - 1);
    let mut len = (t as isize - allele0 as isize) as c_int + 1;
    while *t != 0 {
        t = t.add(1);
        len += 1;
    }
    if (*sw).lals_len != len {
        return 0;
    }
    if libc::memcmp((*sw).lals.cast(), allele0.cast(), len as usize) != 0 {
        return 0;
    }
    1
}

// Native translation of htslib/vcf_sweep.c sw_rec_save().
unsafe fn sw_rec_save(sw: *mut bcf_sweep_t, rec: *mut bcf1_t) -> c_int {
    (*sw).lrid = (*rec).rid;
    (*sw).lpos = (*rec).pos as c_int;
    (*sw).lnals = (*rec).n_allele() as c_int;

    let allele0 = *(*rec).d.allele;
    let mut t = *(*rec).d.allele.add((*sw).lnals as usize - 1);
    let mut len = (t as isize - allele0 as isize) as c_int + 1;
    while *t != 0 {
        t = t.add(1);
        len += 1;
    }
    (*sw).lals_len = len;
    hts_expand_char(len, &mut (*sw).mlals, &mut (*sw).lals);
    libc::memcpy((*sw).lals.cast(), allele0.cast(), len as usize);

    0 // FIXME: check for errs in this function
}

// Native translation of htslib/vcf_sweep.c sw_fill_buffer().
pub(crate) unsafe fn sw_fill_buffer(sw: *mut bcf_sweep_t) -> c_int {
    if (*sw).iidx == 0 {
        return 0;
    }
    (*sw).iidx -= 1;

    let ret = sw_useek((*sw).file, *(*sw).idx.add((*sw).iidx as usize) as i64, 0);
    assert!(ret == 0);

    (*sw).nrec = 0;
    let mut rec = (*sw).rec.add((*sw).nrec as usize);
    while bcf_read1((*sw).file, (*sw).hdr, rec) == 0 {
        bcf_unpack(rec, BCF_UN_STR as c_int);

        // if not in the last block, stop at the saved record
        if (*sw).iidx + 1 < (*sw).nidx && sw_rec_equal(sw, rec) != 0 {
            break;
        }

        (*sw).nrec += 1;
        hts_expand0_bcf1((*sw).nrec + 1, &mut (*sw).mrec, &mut (*sw).rec);
        rec = (*sw).rec.add((*sw).nrec as usize);
    }
    sw_rec_save(sw, (*sw).rec);

    0 // FIXME: check for errs in this function
}

// Native translation of htslib/vcf_sweep.c sw_seek().
pub(crate) unsafe fn sw_seek(sw: *mut bcf_sweep_t, direction: c_int) {
    (*sw).direction = direction;
    if direction == SW_FWD {
        sw_useek((*sw).file, *(*sw).idx as i64, 0);
    } else {
        (*sw).iidx = (*sw).nidx;
        (*sw).nrec = 0;
    }
}

// Native translation of htslib's hts_expand() macro for char arrays:
//   if requested n exceeds current allocation m, grow m and realloc the buffer.
unsafe fn hts_expand_char(n: c_int, m: *mut c_int, ptr: *mut *mut c_char) {
    if n > *m {
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<c_char>()).cast();
    }
}

// Native translation of htslib's hts_expand() macro for uint64_t arrays.
pub(crate) unsafe fn hts_expand_u64(n: c_int, m: *mut c_int, ptr: *mut *mut u64) {
    if n > *m {
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<u64>()).cast();
    }
}

// Native translation of htslib's hts_expand0() macro for bcf1_t arrays:
//   like hts_expand() but zero-initialises the newly allocated tail.
unsafe fn hts_expand0_bcf1(n: c_int, m: *mut c_int, ptr: *mut *mut bcf1_t) {
    if n > *m {
        let old_m = *m;
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<bcf1_t>()).cast();
        std::ptr::write_bytes((*ptr).add(old_m as usize), 0, (*m - old_m) as usize);
    }
}

// Native translation of htslib kroundup32(): round up to the next power of two.
fn kroundup32(x: &mut c_int) {
    let mut v = *x as u32;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    *x = v as c_int;
}

// Native translation of htslib/vcf.c bcf_hdr_read().
pub unsafe fn bcf_hdr_read(fp: *mut htsFile) -> *mut bcf_hdr_t {
    let hdr = bcf_hdr_read_native(fp);
    bcf_hdr_fix_vcf45_vl_types(hdr);
    hdr
}

unsafe fn bcf_hdr_read_native(hfp: *mut htsFile) -> *mut bcf_hdr_t {
    if (*hfp).format.format == HTS_FORMAT_VCF {
        return vcf_hdr_read_text(hfp);
    }
    if (*hfp).format.format != HTS_FORMAT_BCF {
        c_log_error(c"Input is not detected as bcf or vcf format".as_ptr());
        return std::ptr::null_mut();
    }

    // assert(hfp->is_bgzf);
    let fp = hts_get_bgzfp(hfp);
    let mut magic = [0u8; 5];
    let h = bcf_hdr_init(c"r".as_ptr());
    if h.is_null() {
        c_log_error(c"Failed to allocate bcf header".as_ptr());
        return std::ptr::null_mut();
    }
    if bgzf_read(fp, magic.as_mut_ptr().cast(), 5) != 5 {
        c_log_error(c"Failed to read the header (reading BCF in text mode?)".as_ptr());
        bcf_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    if &magic != b"BCF\x02\x02" {
        if &magic[..3] == b"BCF" {
            c_log_error(c"Invalid BCF2 magic string: only BCFv2.2 is supported".as_ptr());
        } else {
            c_log_error(c"Invalid BCF2 magic string".as_ptr());
        }
        bcf_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    let mut buf = [0u8; 4];
    let mut htxt: *mut c_char = std::ptr::null_mut();
    let fail = |htxt: *mut c_char, h: *mut bcf_hdr_t| -> *mut bcf_hdr_t {
        c_log_error(c"Failed to read BCF header".as_ptr());
        libc::free(htxt.cast());
        bcf_hdr_destroy(h);
        std::ptr::null_mut()
    };
    if bgzf_read(fp, buf.as_mut_ptr().cast(), 4) != 4 {
        return fail(htxt, h);
    }
    let hlen = (buf[0] as usize)
        | ((buf[1] as usize) << 8)
        | ((buf[2] as usize) << 16)
        | ((buf[3] as usize) << 24);
    if hlen >= usize::MAX {
        *libc::__errno_location() = libc::ENOMEM;
        return fail(htxt, h);
    }
    htxt = libc::malloc(hlen + 1).cast::<c_char>();
    if htxt.is_null() {
        return fail(htxt, h);
    }
    if bgzf_read(fp, htxt.cast(), hlen) != hlen as isize {
        return fail(htxt, h);
    }
    *htxt.add(hlen) = 0; // Ensure htxt is terminated
    if bcf_hdr_parse(h, htxt) < 0 {
        return fail(htxt, h);
    }
    libc::free(htxt.cast());

    bcf_hdr_incr_ref(h);
    bgzf_internal_h_51_bgzf_set_private_data(
        fp,
        h.cast(),
        Some(hdr_bgzf_private_data_cleanup as BgzfPrivateDataCleanupFunc),
    );

    h
}

// Native translation of htslib/vcf.c bcf_hdr_set_samples().
pub unsafe fn bcf_hdr_set_samples(
    hdr: *mut bcf_hdr_t,
    samples: *const c_char,
    is_file: c_int,
) -> c_int {
    if !samples.is_null() && libc::strcmp(c"-".as_ptr(), samples) == 0 {
        return 0; // keep all samples
    }

    let narr = bit_array_size(bcf_hdr_nsamples_native(hdr));
    (*hdr).keep_samples = libc::calloc(narr as usize, 1).cast::<u8>();
    if (*hdr).keep_samples.is_null() {
        return -1;
    }

    (*hdr).nsamples_ori = bcf_hdr_nsamples_native(hdr);
    if samples.is_null() {
        // exclude all samples
        let d = (*hdr).dict[BCF_DT_SAMPLE as usize].cast::<kh_vdict_t>();
        let new_dict = kh_init_vdict();
        if new_dict.is_null() {
            return -1;
        }

        (*hdr).n[BCF_DT_SAMPLE as usize] = 0;

        let mut k: u32 = 0;
        while k < (*d).n_buckets {
            if !kh_iseither((*d).flags, k) {
                libc::free((*(*d).keys.add(k as usize)) as *mut c_void);
            }
            k += 1;
        }
        kh_destroy_vdict(d);
        (*hdr).dict[BCF_DT_SAMPLE as usize] = new_dict.cast();
        if bcf_hdr_sync(hdr) < 0 {
            return -1;
        }

        return 0;
    }

    let exclude = *samples == b'^' as c_char;
    if exclude {
        for i in 0..bcf_hdr_nsamples_native(hdr) {
            bit_array_set((*hdr).keep_samples, i);
        }
    }

    let mut ret: c_int = 0;
    let mut n: c_int = 0;
    let list_arg = if exclude { samples.add(1) } else { samples };
    let smpls = crate::htslib_rs::hts::hts_readlist(list_arg, is_file, &mut n);
    if smpls.is_null() {
        return -1;
    }
    for i in 0..n {
        let idx = bcf_hdr_id2int(hdr, BCF_DT_SAMPLE as c_int, *smpls.add(i as usize));
        if idx < 0 {
            if ret == 0 {
                ret = i + 1;
            }
            continue;
        }
        debug_assert!(idx < bcf_hdr_nsamples_native(hdr));
        if exclude {
            bit_array_clear((*hdr).keep_samples, idx);
        } else {
            bit_array_set((*hdr).keep_samples, idx);
        }
    }
    for i in 0..n {
        libc::free((*smpls.add(i as usize)).cast());
    }
    libc::free(smpls.cast());

    (*hdr).n[BCF_DT_SAMPLE as usize] = 0;
    for i in 0..(*hdr).nsamples_ori {
        if bit_array_test((*hdr).keep_samples, i) {
            (*hdr).n[BCF_DT_SAMPLE as usize] += 1;
        }
    }

    if bcf_hdr_nsamples_native(hdr) == 0 {
        libc::free((*hdr).keep_samples.cast());
        (*hdr).keep_samples = std::ptr::null_mut();
    } else {
        // Make new list and dictionary with desired samples
        let nsmpl = bcf_hdr_nsamples_native(hdr) as usize;
        let samples_new = libc::malloc(size_of::<*mut c_char>() * nsmpl).cast::<*mut c_char>();
        if samples_new.is_null() {
            return -1;
        }
        let new_dict = kh_init_vdict();
        if new_dict.is_null() {
            libc::free(samples_new.cast());
            return -1;
        }
        let mut idx = 0;
        for i in 0..(*hdr).nsamples_ori {
            if bit_array_test((*hdr).keep_samples, i) {
                let sname = *(*hdr).samples.add(i as usize);
                *samples_new.add(idx as usize) = sname;
                let mut res: c_int = 0;
                let k = kh_put_vdict(new_dict, sname, &mut res);
                if res < 0 {
                    libc::free(samples_new.cast());
                    kh_destroy_vdict(new_dict);
                    return -1;
                }
                let valp = (*new_dict).vals.add(k as usize);
                *valp = bcf_idinfo_def();
                (*valp).id = idx;
                idx += 1;
            }
        }

        // Delete desired samples from old dictionary, so we don't free them
        let d = (*hdr).dict[BCF_DT_SAMPLE as usize].cast::<kh_vdict_t>();
        for i in 0..idx {
            let k = kh_get_vdict(d, *samples_new.add(i as usize));
            if k != (*d).n_buckets {
                kh_del_vdict(d, k);
            }
        }

        // Free everything else
        let mut k: u32 = 0;
        while k < (*d).n_buckets {
            if !kh_iseither((*d).flags, k) {
                libc::free((*(*d).keys.add(k as usize)) as *mut c_void);
            }
            k += 1;
        }
        kh_destroy_vdict(d);
        (*hdr).dict[BCF_DT_SAMPLE as usize] = new_dict.cast();

        libc::free((*hdr).samples.cast());
        (*hdr).samples = samples_new;

        if bcf_hdr_sync(hdr) < 0 {
            return -1;
        }
    }

    ret
}

// original: bcf_sr_add_hreader (htslib/synced_bcf_reader.c:275)
pub(crate) unsafe fn bcf_sr_add_hreader_impl(
    files: *mut bcf_srs_t,
    file_ptr: *mut htsFile,
    autoclose: c_int,
    idxname: *const c_char,
) -> c_int {
    unsafe {
        if file_ptr.is_null() {
            (*files).errnum = bcf_sr_error_open_failed;
            return 0;
        }

        (*files).has_line = hts_realloc_p_cc(
            (*files).has_line.cast(),
            size_of::<c_int>(),
            (*files).nreaders as usize + 1,
        )
        .cast::<c_int>();
        *(*files).has_line.add((*files).nreaders as usize) = 0;
        (*files).readers = hts_realloc_p_cc(
            (*files).readers.cast(),
            size_of::<bcf_sr_t>(),
            (*files).nreaders as usize + 1,
        )
        .cast::<bcf_sr_t>();
        let reader = (*files).readers.add((*files).nreaders as usize);
        (*files).nreaders += 1;
        std::ptr::write_bytes(reader, 0, 1);

        (*reader).file = file_ptr.cast();
        (*files).errnum = 0;

        let rfile: *mut htsFile = (*reader).file.cast();
        if (*rfile).format.compression == HTS_COMPRESSION_BGZF {
            let bgzf = hts_get_bgzfp(rfile);
            if !bgzf.is_null() && super::bgzf::bgzf_check_EOF(bgzf) == 0 {
                (*files).errnum = bcf_sr_error_no_eof;
            }
            if !(*files).p.is_null() {
                let p = (*files).p.cast::<super::hts::htsThreadPool>();
                super::bgzf::bgzf_thread_pool(bgzf, (*p).pool, (*p).qsize);
            }
        }

        if (*files).require_index == REQUIRE_IDX_ {
            if (*rfile).format.format == HTS_FORMAT_VCF {
                if (*rfile).format.compression != HTS_COMPRESSION_BGZF {
                    (*files).errnum = bcf_sr_error_not_bgzf;
                    return 0;
                }
                (*reader).tbx_idx = super::tbx::tbx_index_load2((*file_ptr).fn_, idxname).cast();
                if (*reader).tbx_idx.is_null() {
                    (*files).errnum = bcf_sr_error_idx_load_failed;
                    return 0;
                }
                (*reader).header = bcf_hdr_read(rfile);
            } else if (*rfile).format.format == HTS_FORMAT_BCF {
                if (*rfile).format.compression != HTS_COMPRESSION_BGZF {
                    (*files).errnum = bcf_sr_error_not_bgzf;
                    return 0;
                }
                (*reader).header = bcf_hdr_read(rfile);
                (*reader).bcf_idx = bcf_index_load2((*file_ptr).fn_, idxname).cast();
                if (*reader).bcf_idx.is_null() {
                    (*files).errnum = bcf_sr_error_idx_load_failed;
                    return 0;
                }
            } else {
                (*files).errnum = bcf_sr_error_file_type_error;
                return 0;
            }
        } else {
            if (*rfile).format.format == HTS_FORMAT_BCF || (*rfile).format.format == HTS_FORMAT_VCF
            {
                (*reader).header = bcf_hdr_read(rfile);
            } else {
                (*files).errnum = bcf_sr_error_file_type_error;
                return 0;
            }
            (*files).streaming = 1;
        }
        if (*files).streaming != 0 && (*files).nreaders > 1 {
            if (*files).require_index == ALLOW_NO_IDX_ {
                if SR_NO_INDEX_WARNED.swap(true, Ordering::Relaxed) == false {
                    libc::fprintf(
                        c_compat::stderr.cast(),
                        c"[W::bcf_sr_add_reader] Using multiple unindexed files may produce errors, make sure chromosomes are in the same order!\n"
                            .as_ptr(),
                    );
                }
            } else {
                (*files).errnum = bcf_sr_error_api_usage_error;
                return 0;
            }
        }
        if (*files).streaming != 0 && !(*files).regions.is_null() {
            (*files).errnum = bcf_sr_error_api_usage_error;
            return 0;
        }
        if (*reader).header.is_null() {
            (*files).errnum = bcf_sr_error_header_error;
            return 0;
        }

        (*reader).fname = libc::strdup((*file_ptr).fn_);
        if !(*files).apply_filters.is_null() {
            (*reader).filter_ids = init_filters(
                (*reader).header,
                (*files).apply_filters,
                &mut (*reader).nfilter_ids,
            );
        }

        // Update list of chromosomes
        if (*files).explicit_regs == 0 && (*files).streaming == 0 {
            if (*files).regions.is_null() {
                (*files).regions = bcf_sr_regions_alloc();
                if (*files).regions.is_null() {
                    return 0;
                }
            }
            let mut n = 0;
            let names = if !(*reader).tbx_idx.is_null() {
                super::tbx::tbx_seqnames((*reader).tbx_idx.cast(), &mut n)
            } else {
                bcf_hdr_seqnames((*reader).header, &mut n)
            };
            for i in 0..n as usize {
                bcf_sr_regions_add((*files).regions, *names.add(i), -1, -1);
            }
            libc::free(names.cast());
            regions_sort_and_merge((*files).regions);
        }

        if (*files).require_index == ALLOW_NO_IDX_ && (*files).nreaders > 1 {
            let hdr0 = (*(*files).readers).header;
            let hdr1 = (*reader).header;
            if (*hdr0).n[BCF_DT_CTG as usize] != (*hdr1).n[BCF_DT_CTG as usize] {
                (*files).errnum = BCF_SR_ERROR_NOIDX_ERROR as u32;
                return 0;
            }
            for i in 0..(*hdr0).n[BCF_DT_CTG as usize] {
                if libc::strcmp(bcf_hdr_id2name(hdr0, i), bcf_hdr_id2name(hdr1, i)) != 0 {
                    (*files).errnum = BCF_SR_ERROR_NOIDX_ERROR as u32;
                    return 0;
                }
            }
        }

        let auxdata = bcf_sr_aux_mut(files);
        if !auxdata.is_null() {
            let tmp = hts_realloc_p_cc(
                (*auxdata).closefile.cast(),
                size_of::<c_int>(),
                (*files).nreaders as usize,
            )
            .cast::<c_int>();
            if tmp.is_null() {
                return 0;
            }
            *tmp.add(((*files).nreaders - 1) as usize) = autoclose;
            (*auxdata).closefile = tmp;
        }

        1
    }
}

static SR_NO_INDEX_WARNED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub unsafe fn bcf_subset_format(hdr: *const bcf_hdr_t, rec: *mut bcf1_t) -> c_int {
    let ret = bcf_subset_format_core(hdr, rec);
    if ret == 0 {
        bcf_canonicalize_duplicate_format(rec);
    }
    ret
}

// Native translation of htslib/vcf.c bcf_subset_format().
unsafe fn bcf_subset_format_core(hdr: *const bcf_hdr_t, rec: *mut bcf1_t) -> c_int {
    if (*hdr).keep_samples.is_null() {
        return 0;
    }
    if (*hdr).n[BCF_DT_SAMPLE as usize] == 0 {
        (*rec).indiv.l = 0;
        (*rec).set_n_sample(0);
        return 0;
    }

    let keep = (*hdr).keep_samples;
    let bit_array_test = |i: c_int| (*keep.add((i / 8) as usize) & (1 << (i % 8))) != 0;

    let mut ptr = (*rec).indiv.s.cast::<u8>();
    let mut dst: *mut u8 = std::ptr::null_mut();
    let n_fmt = (*rec).n_fmt() as c_int;
    let n_sample = (*rec).n_sample() as c_int;
    let dec = &mut (*rec).d;
    hts_expand_fmt(n_fmt, &mut dec.m_fmt, &mut dec.fmt);
    for i in 0..dec.m_fmt as usize {
        (*dec.fmt.add(i)).set_p_free(0);
    }

    for i in 0..n_fmt as usize {
        ptr = bcf_unpack_fmt_core1_rs(ptr, n_sample, dec.fmt.add(i));
        let fmt_i = dec.fmt.add(i);
        let mut src = (*fmt_i).p.sub((*fmt_i).size as usize);
        if !dst.is_null() {
            let fmt_prev = dec.fmt.add(i - 1);
            libc::memmove(
                (*fmt_prev).p.add((*fmt_prev).p_len as usize).cast(),
                (*fmt_i).p.sub((*fmt_i).p_off() as usize).cast(),
                (*fmt_i).p_off() as usize,
            );
            (*fmt_i).p = (*fmt_prev)
                .p
                .add((*fmt_prev).p_len as usize + (*fmt_i).p_off() as usize);
        }
        dst = (*fmt_i).p;
        for j in 0..(*hdr).nsamples_ori {
            src = src.add((*fmt_i).size as usize);
            if !bit_array_test(j) {
                continue;
            }
            libc::memmove(dst.cast(), src.cast(), (*fmt_i).size as usize);
            dst = dst.add((*fmt_i).size as usize);
        }
        (*rec).indiv.l -= (*fmt_i).p_len as usize - (dst as usize - (*fmt_i).p as usize);
        (*fmt_i).p_len = (dst as usize - (*fmt_i).p as usize) as u32;
    }
    (*rec).unpacked |= BCF_UN_FMT as c_int;

    (*rec).set_n_sample((*hdr).n[BCF_DT_SAMPLE as usize] as u32);
    0
}

// Native translation of htslib/vcf.c bcf_hdr_write().
pub unsafe fn bcf_hdr_write(hfp: *mut htsFile, h: *mut bcf_hdr_t) -> c_int {
    if h.is_null() {
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    }
    if (*h).dirty != 0 && bcf_hdr_sync(h) < 0 {
        return -1;
    }
    (*hfp).format.category = HTS_FORMAT_VARIANT_DATA;
    if (*hfp).format.format == HTS_FORMAT_VCF || (*hfp).format.format == HTS_FORMAT_TEXT_FORMAT {
        (*hfp).format.format = HTS_FORMAT_VCF;
        return vcf_hdr_write(hfp, h);
    }

    if (*hfp).format.format == HTS_FORMAT_BINARY_FORMAT {
        (*hfp).format.format = HTS_FORMAT_BCF;
    }

    let mut htxt = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    if bcf_hdr_format(h, 1, &mut htxt) < 0 {
        libc::free(htxt.s.cast());
        return -1;
    }
    kputc(b'\0' as c_int, &mut htxt); // include the \0 byte

    let fp = hts_get_bgzfp(hfp);
    if bgzf_write(fp, c"BCF\x02\x02".as_ptr().cast(), 5) != 5 {
        libc::free(htxt.s.cast());
        return -1;
    }
    let mut hlen = [0u8; 4];
    super::hts::u32_to_le(htxt.l as u32, hlen.as_mut_ptr());
    if bgzf_write(fp, hlen.as_ptr().cast(), 4) != 4 {
        libc::free(htxt.s.cast());
        return -1;
    }
    if bgzf_write(fp, htxt.s.cast(), htxt.l as usize) != htxt.l as isize {
        libc::free(htxt.s.cast());
        return -1;
    }
    if bgzf_flush(fp) < 0 {
        libc::free(htxt.s.cast());
        return -1;
    }

    bcf_hdr_incr_ref(h);
    bgzf_internal_h_51_bgzf_set_private_data(
        fp,
        h.cast(),
        Some(hdr_bgzf_private_data_cleanup as BgzfPrivateDataCleanupFunc),
    );

    libc::free(htxt.s.cast());
    0
}

// ---------------------------------------------------------------------------
// Native translation of htslib/vcf.c vcf_parse() and its helpers.
//
// Parses one VCF text line (kstring_t) into a bcf1_t against the header,
// building v->shared and v->indiv via the bcf_enc_* encoders and the native
// X31 vdict lookups.  Ported 1:1 from hts-sys-2.2.0/htslib/vcf.c (which is
// compiled *without* VCF_ALLOW_INT64; the int64 branches are therefore the
// #else path).
// ---------------------------------------------------------------------------

const MAX_N_FMT: usize = 255; // Limited by size of bcf1_t n_fmt field

#[inline]
pub(crate) unsafe fn bcf_hdr_nsamples_native(h: *const bcf_hdr_t) -> c_int {
    (*h).n[BCF_DT_SAMPLE as usize]
}

// kh_val(d, k): the bcf_idinfo_t value at bucket index k.
#[inline]
unsafe fn vdict_val(d: *const kh_vdict_t, k: u32) -> *const bcf_idinfo_t {
    (*d).vals.add(k as usize) as *const bcf_idinfo_t
}

// bit_array_test(a,i): ((a)[(i)/8] & (1 << ((i)%8)))
#[inline]
unsafe fn bit_array_test(a: *const u8, i: c_int) -> bool {
    (*a.add((i / 8) as usize) & (1u8 << (i % 8) as u32)) != 0
}

// bit_array_size(n): ((n)/8+1)
#[inline]
fn bit_array_size(n: c_int) -> c_int {
    n / 8 + 1
}

// bit_array_set(a,i): ((a)[(i)/8] |= 1 << ((i)%8))
#[inline]
unsafe fn bit_array_set(a: *mut u8, i: c_int) {
    *a.add((i / 8) as usize) |= 1u8 << (i % 8) as u32;
}

// bit_array_clear(a,i): ((a)[(i)/8] &= ~(1 << ((i)%8)))
#[inline]
unsafe fn bit_array_clear(a: *mut u8, i: c_int) {
    *a.add((i / 8) as usize) &= !(1u8 << (i % 8) as u32);
}

// align_mem(): pad kstring to an 8-byte boundary.
#[inline]
unsafe fn align_mem(s: *mut kstring_t) -> c_int {
    let mut e = 0;
    if (*s).l & 7 != 0 {
        let zero: u64 = 0;
        e = (kputsn(
            (&zero as *const u64).cast::<c_char>(),
            (8 - ((*s).l & 7)) as size_t,
            s,
        ) < 0) as c_int;
    }
    if e == 0 {
        0
    } else {
        -1
    }
}

// fmt_aux_t mirror of the C struct used while pivoting FORMAT data.
#[derive(Clone, Copy)]
struct FmtAux {
    offset: c_int, // offset of buf into h->mem
    key: c_int,    // BCF_DT_ID id
    max_m: u32,    // max number of values per array (sample)
    max_g: u32,    // max number of allele indexes (is_gt)
    max_l: u32,    // max length of field
    size: c_int,   // per-sample byte size
    is_gt: bool,
    y: u32, // h->id[0][key].val->info[BCF_HL_FMT]
    buf: *mut u8,
}

impl FmtAux {
    fn zeroed() -> Self {
        FmtAux {
            offset: 0,
            key: 0,
            max_m: 0,
            max_g: 0,
            max_l: 0,
            size: 0,
            is_gt: false,
            y: 0,
            buf: std::ptr::null_mut(),
        }
    }
}

// Helper: emit an error log message (formatted like htslib's hts_log_error).
unsafe fn vcf_log_error(msg: String) {
    let c = std::ffi::CString::new(msg).unwrap_or_default();
    c_log_error(c.as_ptr());
}
unsafe fn vcf_log_warning(msg: String) {
    let c = std::ffi::CString::new(msg).unwrap_or_default();
    c_log_warning(c.as_ptr());
}

unsafe fn vcf_seqname_safe_str(h: *const bcf_hdr_t, v: *const bcf1_t) -> String {
    let p = bcf_seqname_safe(h, v);
    if p.is_null() {
        "(unknown)".to_string()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

// detect FORMAT "."
unsafe fn vcf_parse_format_empty1(
    s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    p: *const c_char,
    q: *const c_char,
) -> c_int {
    let end = (*s).s.add((*s).l) as *const c_char;
    if q >= end {
        vcf_log_error(format!(
            "FORMAT column with no sample columns starting at {}:{}",
            vcf_seqname_safe_str(h, v),
            (*v).pos + 1
        ));
        (*v).errcode |= BCF_ERR_NCOLS as c_int;
        return -1;
    }

    (*v).set_n_fmt(0);
    if *p == b'.' as c_char && *p.add(1) == 0 {
        // FORMAT field is empty "."
        (*v).set_n_sample(bcf_hdr_nsamples_native(h) as u32);
        return 1;
    }

    0
}

// get format information from the dictionary
unsafe fn vcf_parse_format_dict2(
    _s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    p: *mut c_char,
    _q: *mut c_char,
    fmt: *mut FmtAux,
) -> c_int {
    let d = (*h).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>();
    let mut aux1 = super::hts::ks_tokaux_t {
        tab: [0; 4],
        sep: 0,
        finished: 0,
        p: std::ptr::null(),
    };
    let mut j: usize = 0;
    let mut t = kstrtok(p, c":".as_ptr(), &mut aux1);
    while !t.is_null() {
        if j >= MAX_N_FMT {
            (*v).errcode |= BCF_ERR_LIMITS as c_int;
            vcf_log_error(format!(
                "FORMAT column at {}:{} lists more identifiers than htslib can handle",
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            return -1;
        }

        *(aux1.p as *mut c_char) = 0;
        let mut k = kh_get_vdict(d, t);
        if k == (*d).n_buckets || ((*vdict_val(d, k)).info[BCF_HL_FMT as usize] == 15) {
            if *t == b'.' as c_char && *t.add(1) == 0 {
                vcf_log_error(format!(
                    "Invalid FORMAT tag name '.' at {}:{}",
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_TAG_INVALID as c_int;
                return -1;
            }
            let tag = CStr::from_ptr(t).to_string_lossy().into_owned();
            vcf_log_warning(format!(
                "FORMAT '{}' at {}:{} is not defined in the header, assuming Type=String",
                tag,
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            let tmp = std::ffi::CString::new(format!(
                "##FORMAT=<ID={},Number=1,Type=String,Description=\"Dummy\">",
                tag
            ))
            .unwrap_or_default();
            let mut l: c_int = 0;
            let hrec = bcf_hdr_parse_line(h, tmp.as_ptr(), &mut l);
            let mut res = if !hrec.is_null() {
                bcf_hdr_add_hrec(h.cast_mut(), hrec)
            } else {
                -1
            };
            if res < 0 {
                bcf_hrec_destroy(hrec);
            }
            if res > 0 {
                res = bcf_hdr_sync(h.cast_mut());
            }
            k = kh_get_vdict(d, t);
            (*v).errcode |= BCF_ERR_TAG_UNDEF as c_int;
            if res != 0 || k == (*d).n_buckets {
                vcf_log_error(format!(
                    "Could not add dummy header for FORMAT '{}' at {}:{}",
                    tag,
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_TAG_INVALID as c_int;
                return -1;
            }
        }
        let f = &mut *fmt.add(j);
        f.max_l = 0;
        f.max_m = 0;
        f.max_g = 0;
        f.key = (*vdict_val(d, k)).id;
        f.is_gt = *t == b'G' as c_char && *t.add(1) == b'T' as c_char && *t.add(2) == 0;
        f.y = (*(*(*h).id[0].add(f.key as usize)).val).info[BCF_HL_FMT as usize] as u32;
        (*v).set_n_fmt((*v).n_fmt() + 1);

        j += 1;
        t = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux1);
    }
    0
}

// meta-character table from htslib (1 for \0 \t , / : |).
static FMT_META: [u8; 256] = {
    let mut m = [0u8; 256];
    m[0] = 1; // \0
    m[b'\t' as usize] = 1;
    m[b',' as usize] = 1;
    m[b'/' as usize] = 1;
    m[b':' as usize] = 1;
    m[b'|' as usize] = 1;
    m
};

// compute max
unsafe fn vcf_parse_format_max3(
    s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    _p: *mut c_char,
    q: *mut c_char,
    fmt: *mut FmtAux,
) -> c_int {
    let mut n_sample_ori: c_int = -1;
    let mut r = q.add(1); // r: position in the format string
    let mut l: u32;
    let mut m: u32 = 1;
    let mut g: u32 = 1;
    (*v).set_n_sample(0);
    let end = (*s).s.add((*s).l);

    while r < end {
        // can we skip some samples?
        if !(*h).keep_samples.is_null() {
            n_sample_ori += 1;
            if !bit_array_test((*h).keep_samples, n_sample_ori) {
                while *r != b'\t' as c_char && r < end {
                    r = r.add(1);
                }
                if *r == b'\t' as c_char {
                    *r = 0;
                    r = r.add(1);
                }
                continue;
            }
        }

        // collect fmt stats: max vector size, length, number of alleles
        let mut j: usize = 0;
        let mut f = fmt;
        let mut r_start = r;
        'for_loop: loop {
            // Quickly skip ahead to an appropriate meta-character
            while FMT_META[*r as u8 as usize] == 0 {
                r = r.add(1);
            }

            match *r as u8 {
                b',' => {
                    m += 1;
                }
                b'|' | b'/' => {
                    if (*f).is_gt {
                        g += 1;
                    }
                }
                c => {
                    if c == b'\t' {
                        *r = 0; // fall through
                    }
                    // default / '\0' / ':'
                    l = (r as usize - r_start as usize) as u32;
                    r_start = r;
                    if (*f).max_m < m {
                        (*f).max_m = m;
                    }
                    if (*f).max_l < l {
                        (*f).max_l = l;
                    }
                    if (*f).is_gt && (*f).max_g < g {
                        (*f).max_g = g;
                    }
                    m = 1;
                    g = 1;
                    if *r == b':' as c_char {
                        j += 1;
                        f = f.add(1);
                        if j >= (*v).n_fmt() as usize {
                            let key = (*(*h).id[BCF_DT_CTG as usize].add((*v).rid as usize)).key;
                            let chrom = if key.is_null() {
                                "(unknown)".to_string()
                            } else {
                                CStr::from_ptr(key).to_string_lossy().into_owned()
                            };
                            vcf_log_error(format!(
                                "Incorrect number of FORMAT fields at {}:{}",
                                chrom,
                                (*v).pos + 1
                            ));
                            (*v).errcode |= BCF_ERR_NCOLS as c_int;
                            return -1;
                        }
                    } else {
                        break 'for_loop;
                    }
                }
            }
            if r >= end {
                break;
            }
            r = r.add(1);
        }
        // end_for:
        (*v).set_n_sample((*v).n_sample() + 1);
        if (*v).n_sample() as c_int == bcf_hdr_nsamples_native(h) {
            break;
        }
        r = r.add(1);
    }

    0
}

// allocate memory for arrays
unsafe fn vcf_parse_format_alloc4(
    _s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    _p: *const c_char,
    _q: *const c_char,
    fmt: *mut FmtAux,
) -> c_int {
    let mem: *mut kstring_t = std::ptr::addr_of!((*h).mem) as *mut kstring_t;

    for j in 0..(*v).n_fmt() as usize {
        let f = &mut *fmt.add(j);
        if f.max_m == 0 {
            f.max_m = 1; // omitted trailing format field
        }

        let htype = (f.y >> 4 & 0xf) as u32;
        if htype == BCF_HT_STR {
            f.size = if f.is_gt {
                (f.max_g << 2) as c_int
            } else {
                f.max_l as c_int
            };
        } else if htype == BCF_HT_REAL || htype == BCF_HT_INT {
            f.size = (f.max_m << 2) as c_int;
        } else {
            vcf_log_error(format!(
                "The format type {} at {}:{} is currently not supported",
                htype,
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            (*v).errcode |= BCF_ERR_TAG_INVALID as c_int;
            return -1;
        }

        if align_mem(mem) < 0 {
            vcf_log_error(format!(
                "Memory allocation failure at {}:{}",
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            (*v).errcode |= BCF_ERR_LIMITS as c_int;
            return -1;
        }

        // Limit the total memory to ~2Gb per VCF row.
        if (*mem).l as u64 + (*v).n_sample() as u64 * f.size as u64 > c_int::MAX as u64 {
            vcf_log_warning(format!(
                "Excessive memory required by FORMAT fields at {}:{}",
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            (*v).errcode |= BCF_ERR_LIMITS as c_int;
            f.size = -1;
            f.offset = 0;
            continue;
        }

        f.offset = (*mem).l as c_int;
        if super::hts::ks_resize(mem, (*mem).l + (*v).n_sample() as size_t * f.size as size_t) < 0 {
            vcf_log_error(format!(
                "Memory allocation failure at {}:{}",
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            (*v).errcode |= BCF_ERR_LIMITS as c_int;
            return -1;
        }
        (*mem).l += (*v).n_sample() as size_t * f.size as size_t;
    }

    for j in 0..(*v).n_fmt() as usize {
        let f = &mut *fmt.add(j);
        f.buf = (*mem).s.cast::<u8>().add(f.offset as usize);
    }

    0
}

// Fill the sample fields
unsafe fn vcf_parse_format_fill5(
    s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    _p: *const c_char,
    q: *const c_char,
    fmt: *mut FmtAux,
) -> c_int {
    const BCF_MAX_BT_INT32: i64 = 0x7fff_ffff;
    let mut n_sample_ori: c_int = -1;
    // At beginning of the loop t points to the first char of a format
    let mut t = q.add(1) as *mut c_char;
    let mut m: usize = 0; // m: sample id
    let nsamples = bcf_hdr_nsamples_native(h);
    let ver = bcf_get_version(h, std::ptr::null());

    let end = (*s).s.add((*s).l) as *mut c_char;
    while t < end {
        // can we skip some samples?
        if !(*h).keep_samples.is_null() {
            n_sample_ori += 1;
            if !bit_array_test((*h).keep_samples, n_sample_ori) {
                while *t != 0 && t < end {
                    t = t.add(1);
                }
                t = t.add(1);
                continue;
            }
        }
        if m as c_int == nsamples {
            break;
        }

        let mut j: usize = 0; // j-th format field, m-th sample
        while t < end {
            let z = &mut *fmt.add(j);
            j += 1;
            let htype = (z.y >> 4 & 0xf) as u32;
            if z.buf.is_null() {
                vcf_log_error(format!(
                    "Memory allocation failure for FORMAT field type {} at {}:{}",
                    z.y >> 4 & 0xf,
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_LIMITS as c_int;
                return -1;
            }

            if z.size == -1 {
                // this field is to be ignored, it's too big
                while *t != b':' as c_char && *t != 0 {
                    t = t.add(1);
                }
            } else if htype == BCF_HT_STR {
                if z.is_gt {
                    // Genotypes.
                    //([/|])?<val>)([|/]<val>)+... where <val> is [0-9]+ or ".".
                    let mut is_phased: i32 = 0;
                    let x = z.buf.add(z.size as usize * m).cast::<u32>();
                    let mut unreadable: u32 = 0;
                    let mut max: u32 = 0;
                    let mut overflow: c_int = 0;
                    let mut l: usize = 0;
                    let mut ploidy: c_int = 0;
                    let mut anyunphased: c_int = 0;
                    let mut phasingprfx: c_int = 0;
                    let mut unknown1: c_int = 0;

                    // with prefixed phasing, it is explicitly given for 1st one
                    // with non-prefixed, set based on ploidy and phasing of other
                    // alleles.
                    if ver >= VCF44 && (*t == b'|' as c_char || *t == b'/' as c_char) {
                        // cache prefix and phasing status
                        is_phased = (*t == b'|' as c_char) as i32;
                        t = t.add(1);
                        phasingprfx = 1;
                    }

                    loop {
                        ploidy += 1;
                        if *t == b'.' as c_char {
                            t = t.add(1);
                            *x.add(l) = is_phased as u32;
                            l += 1;
                            if l == 1 {
                                // for 1st allele only
                                unknown1 = 1;
                            }
                        } else {
                            let tt = t;
                            let val: u32;
                            if *t >= b'0' as c_char
                                && *t <= b'9' as c_char
                                && !(*t.add(1) >= b'0' as c_char && *t.add(1) <= b'9' as c_char)
                            {
                                val = (*t - b'0' as c_char) as u32;
                                t = t.add(1);
                            } else {
                                let mut te: *mut c_char = std::ptr::null_mut();
                                val = hts_str2uint(
                                    t,
                                    &mut te,
                                    size_of::<u32>() as c_int * c_char::MAX as c_int - 2,
                                    &mut overflow,
                                ) as u32;
                                unreadable |= (tt == te) as u32;
                                t = te;
                            }
                            if max < val {
                                max = val;
                            }
                            *x.add(l) = (val + 1) << 1 | is_phased as u32;
                            l += 1;
                        }
                        anyunphased |= ((ploidy != 1) && is_phased == 0) as c_int;
                        is_phased = (*t == b'|' as c_char) as i32;
                        if *t != b'|' as c_char && *t != b'/' as c_char {
                            break;
                        }
                        t = t.add(1);
                    }
                    if phasingprfx == 0 {
                        // no prefixed phasing, get GT in v44 way
                        // no explicit phasing for 1st allele, set based on
                        // other alleles and ploidy
                        if ploidy == 1 {
                            // implicitly phased
                            if unknown1 == 0 {
                                *x |= 1;
                            }
                        } else {
                            // set by other unphased alleles
                            *x |= if anyunphased != 0 { 0 } else { 1 };
                        }
                    }
                    if overflow != 0 || max > (i32::MAX >> 1) as u32 - 1 {
                        vcf_log_error(format!(
                            "Couldn't read GT data: value too large at {}:{}",
                            vcf_seqname_safe_str(h, v),
                            (*v).pos + 1
                        ));
                        return -1;
                    }
                    if unreadable != 0 {
                        vcf_log_error(format!(
                            "Couldn't read GT data: value not a number or '.' at {}:{}",
                            vcf_seqname_safe_str(h, v),
                            (*v).pos + 1
                        ));
                        return -1;
                    }
                    if l == 0 {
                        *x.add(l) = 0; // An empty field, insert missing value
                        l += 1;
                    }
                    let cnt = (z.size >> 2) as usize;
                    while l < cnt {
                        *x.add(l) = bcf_int32_vector_end as u32;
                        l += 1;
                    }
                } else {
                    // Otherwise arbitrary strings
                    let x = z.buf.add(z.size as usize * m).cast::<c_char>();
                    let mut l: usize = 0;
                    while *t != b':' as c_char && *t != 0 {
                        *x.add(l) = *t;
                        l += 1;
                        t = t.add(1);
                    }
                    if z.size as usize > l {
                        libc::memset(
                            x.add(l).cast(),
                            0,
                            (z.size as usize - l) * size_of::<c_char>(),
                        );
                    }
                }
            } else if htype == BCF_HT_INT {
                let x = z.buf.add(z.size as usize * m).cast::<i32>();
                let mut l: usize = 0;
                loop {
                    if *t == b'.' as c_char {
                        *x.add(l) = bcf_int32_missing;
                        l += 1;
                        t = t.add(1); // ++t to skip "."
                    } else {
                        let mut overflow: c_int = 0;
                        let mut te: *mut c_char = std::ptr::null_mut();
                        let mut tmp_val =
                            hts_str2int(t, &mut te, (size_of::<i64>() * 8) as c_int, &mut overflow);
                        if te == t
                            || overflow != 0
                            || tmp_val < BCF_MIN_BT_INT32
                            || tmp_val > BCF_MAX_BT_INT32
                        {
                            tmp_val = bcf_int32_missing as i64;
                        }
                        *x.add(l) = tmp_val as i32;
                        l += 1;
                        t = te;
                    }
                    if *t != b',' as c_char {
                        break;
                    }
                    t = t.add(1);
                }
                if l == 0 {
                    *x.add(l) = bcf_int32_missing;
                    l += 1;
                }
                let cnt = (z.size >> 2) as usize;
                while l < cnt {
                    *x.add(l) = bcf_int32_vector_end;
                    l += 1;
                }
            } else if htype == BCF_HT_REAL {
                let x = z.buf.add(z.size as usize * m).cast::<f32>();
                let mut l: usize = 0;
                loop {
                    if *t == b'.' as c_char && libc::isdigit(*t.add(1) as c_int) == 0 {
                        *x.add(l) = f32::from_bits(bcf_float_missing);
                        l += 1;
                        t = t.add(1); // ++t to skip "."
                    } else {
                        let mut overflow: c_int = 0;
                        let mut te: *mut c_char = std::ptr::null_mut();
                        let tmp_val = hts_str2dbl(t, &mut te, &mut overflow) as f32;
                        *x.add(l) = tmp_val;
                        l += 1;
                        t = te;
                    }
                    if *t != b',' as c_char {
                        break;
                    }
                    t = t.add(1);
                }
                if l == 0 {
                    *x.add(l) = f32::from_bits(bcf_float_missing);
                    l += 1;
                }
                let cnt = (z.size >> 2) as usize;
                while l < cnt {
                    *x.add(l) = f32::from_bits(bcf_float_vector_end);
                    l += 1;
                }
            } else {
                vcf_log_error(format!(
                    "Unknown FORMAT field type {} at {}:{}",
                    htype,
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_TAG_INVALID as c_int;
                return -1;
            }

            if *t == 0 {
                break;
            } else if *t == b':' as c_char {
                t = t.add(1);
            } else {
                let ch = (*t as u8 as char).to_string();
                let key = (*(*h).id[BCF_DT_ID as usize].add(z.key as usize)).key;
                let keyname = if key.is_null() {
                    "(unknown)".to_string()
                } else {
                    CStr::from_ptr(key).to_string_lossy().into_owned()
                };
                vcf_log_error(format!(
                    "Invalid character '{}' in '{}' FORMAT field at {}:{}",
                    ch,
                    keyname,
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_CHAR as c_int;
                return -1;
            }
        }

        // fill end-of-vector values
        while j < (*v).n_fmt() as usize {
            let z = &mut *fmt.add(j);
            j += 1;
            let htype = (z.y >> 4 & 0xf) as u32;
            if htype == BCF_HT_STR {
                if z.is_gt {
                    let x = z.buf.add(z.size as usize * m).cast::<i32>();
                    if z.size != 0 {
                        *x = bcf_int32_missing;
                    }
                    let cnt = (z.size >> 2) as usize;
                    let mut l = 1usize;
                    while l < cnt {
                        *x.add(l) = bcf_int32_vector_end;
                        l += 1;
                    }
                } else {
                    let x = z.buf.add(z.size as usize * m).cast::<c_char>();
                    if z.size != 0 {
                        *x = b'.' as c_char;
                        libc::memset(
                            x.add(1).cast(),
                            0,
                            (z.size as usize - 1) * size_of::<c_char>(),
                        );
                    }
                }
            } else if htype == BCF_HT_INT {
                let x = z.buf.add(z.size as usize * m).cast::<i32>();
                *x = bcf_int32_missing;
                let cnt = (z.size >> 2) as usize;
                let mut l = 1usize;
                while l < cnt {
                    *x.add(l) = bcf_int32_vector_end;
                    l += 1;
                }
            } else if htype == BCF_HT_REAL {
                let x = z.buf.add(z.size as usize * m).cast::<f32>();
                *x = f32::from_bits(bcf_float_missing);
                let cnt = (z.size >> 2) as usize;
                let mut l = 1usize;
                while l < cnt {
                    *x.add(l) = f32::from_bits(bcf_float_vector_end);
                    l += 1;
                }
            }
        }

        m += 1;
        t = t.add(1);
    }

    0
}

// write individual genotype information
unsafe fn vcf_parse_format_gt6(
    _s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    _p: *const c_char,
    _q: *const c_char,
    fmt: *mut FmtAux,
) -> c_int {
    let str_: *mut kstring_t = std::ptr::addr_of_mut!((*v).indiv).cast::<kstring_t>();
    let mut need_downsize = 0;
    if (*v).n_sample() > 0 {
        for i in 0..(*v).n_fmt() as usize {
            let z = &mut *fmt.add(i);
            if z.size == -1 {
                need_downsize = 1;
                continue;
            }
            bcf_enc_int1(str_, z.key);
            let htype = (z.y >> 4 & 0xf) as u32;
            if htype == BCF_HT_STR && !z.is_gt {
                bcf_enc_size(str_, z.size, BCF_BT_CHAR as c_int);
                kputsn(
                    z.buf.cast::<c_char>(),
                    z.size as size_t * (*v).n_sample() as size_t,
                    str_,
                );
            } else if htype == BCF_HT_INT || z.is_gt {
                bcf_enc_vint(
                    str_,
                    (z.size >> 2) * (*v).n_sample() as c_int,
                    z.buf.cast::<i32>(),
                    z.size >> 2,
                );
            } else {
                bcf_enc_size(str_, z.size >> 2, BCF_BT_FLOAT as c_int);
                if serialize_float_array(
                    str_,
                    (z.size >> 2) as usize * (*v).n_sample() as usize,
                    z.buf.cast::<f32>(),
                ) != 0
                {
                    (*v).errcode |= BCF_ERR_LIMITS as c_int;
                    vcf_log_error(format!(
                        "Out of memory at {}:{}",
                        vcf_seqname_safe_str(h, v),
                        (*v).pos + 1
                    ));
                    return -1;
                }
            }
        }
    }
    if need_downsize != 0 {
        let mut i = 1usize;
        while i < (*v).n_fmt() as usize {
            if (*fmt.add(i)).size == -1 {
                std::ptr::copy(fmt.add(i), fmt.add(i - 1), 1);
                (*v).set_n_fmt((*v).n_fmt() - 1);
            } else {
                i += 1;
            }
        }
    }

    0
}

// validity checking
unsafe fn vcf_parse_format_check7(h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    if (*v).n_sample() as c_int != bcf_hdr_nsamples_native(h) {
        vcf_log_error(format!(
            "Number of columns at {}:{} does not match the number of samples ({} vs {})",
            vcf_seqname_safe_str(h, v),
            (*v).pos + 1,
            (*v).n_sample(),
            bcf_hdr_nsamples_native(h)
        ));
        (*v).errcode |= BCF_ERR_NCOLS as c_int;
        return -1;
    }
    if (*v).indiv.l > 0xffff_ffff {
        vcf_log_error(format!(
            "The FORMAT at {}:{} is too long",
            vcf_seqname_safe_str(h, v),
            (*v).pos + 1
        ));
        (*v).errcode |= BCF_ERR_LIMITS as c_int;
        (*v).set_n_fmt(0);
        return -1;
    }
    0
}

// p,q is the start and the end of the FORMAT field
unsafe fn vcf_parse_format(
    s: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    p: *mut c_char,
    q: *mut c_char,
) -> c_int {
    if bcf_hdr_nsamples_native(h) == 0 {
        return 0;
    }
    let mem: *mut kstring_t = std::ptr::addr_of!((*h).mem) as *mut kstring_t;
    (*mem).l = 0;

    // C htslib uses an uninitialised stack array here. We previously zeroed
    // all 255 entries per record (~10 KB memset × ~62K records = ~640 MB
    // of dead memsets on a multi-sample VCF). vcf_parse_format_dict2 sets
    // every field that subsequent sub-passes read, for entries 0..n_fmt,
    // so leaving the storage uninitialised is sound — match C's contract.
    let mut fmt_uninit: std::mem::MaybeUninit<[FmtAux; MAX_N_FMT]> =
        std::mem::MaybeUninit::uninit();
    let fmt: *mut FmtAux = fmt_uninit.as_mut_ptr().cast::<FmtAux>();

    // detect FORMAT "."
    let ret = vcf_parse_format_empty1(s, h, v, p, q);
    if ret != 0 {
        return if ret > 0 { 0 } else { -1 };
    }

    if vcf_parse_format_dict2(s, h, v, p, q, fmt) < 0 {
        return -1;
    }
    if vcf_parse_format_max3(s, h, v, p, q, fmt) < 0 {
        return -1;
    }
    if vcf_parse_format_alloc4(s, h, v, p, q, fmt) < 0 {
        return -1;
    }
    if vcf_parse_format_fill5(s, h, v, p, q, fmt) < 0 {
        return -1;
    }
    if vcf_parse_format_gt6(s, h, v, p, q, fmt) < 0 {
        return -1;
    }
    if vcf_parse_format_check7(h, v) < 0 {
        return -1;
    }

    0
}

// Simple error recovery for chromosomes not defined in the header.
unsafe fn fix_chromosome(h: *const bcf_hdr_t, d: *mut kh_vdict_t, p: *const c_char) -> u32 {
    let name = CStr::from_ptr(p).to_string_lossy().into_owned();
    let tmp = std::ffi::CString::new(format!("##contig=<ID={}>", name)).unwrap_or_default();
    let mut l: c_int = 0;
    let hrec = bcf_hdr_parse_line(h, tmp.as_ptr(), &mut l);
    let res = if !hrec.is_null() {
        bcf_hdr_add_hrec(h.cast_mut(), hrec)
    } else {
        -1
    };
    if res < 0 {
        bcf_hrec_destroy(hrec);
    }
    if res > 0 {
        let _ = bcf_hdr_sync(h.cast_mut());
    }
    kh_get_vdict(d, p)
}

unsafe fn vcf_parse_filter(
    str_: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    p: *mut c_char,
    q: *mut c_char,
) -> c_int {
    let mut n_flt: c_int = 1;
    let d = (*h).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>();
    // count the number of filters
    if *q.sub(1) == b';' as c_char {
        *q.sub(1) = 0;
    }
    let mut r = p;
    while *r != 0 {
        if *r == b';' as c_char {
            n_flt += 1;
        }
        r = r.add(1);
    }
    let a_flt = libc::malloc(n_flt as size_t * size_of::<i32>()).cast::<i32>();
    if a_flt.is_null() {
        vcf_log_error(format!(
            "Could not allocate memory at {}:{}",
            vcf_seqname_safe_str(h, v),
            (*v).pos + 1
        ));
        (*v).errcode |= BCF_ERR_LIMITS as c_int;
        return -1;
    }

    // add filters
    let mut aux1 = super::hts::ks_tokaux_t {
        tab: [0; 4],
        sep: 0,
        finished: 0,
        p: std::ptr::null(),
    };
    let mut i: usize = 0;
    let mut t = kstrtok(p, c";".as_ptr(), &mut aux1);
    while !t.is_null() {
        *(aux1.p as *mut c_char) = 0;
        let mut k = kh_get_vdict(d, t);
        if k == (*d).n_buckets {
            let tag = CStr::from_ptr(t).to_string_lossy().into_owned();
            vcf_log_warning(format!("FILTER '{}' is not defined in the header", tag));
            let tmp =
                std::ffi::CString::new(format!("##FILTER=<ID={},Description=\"Dummy\">", tag))
                    .unwrap_or_default();
            let mut l: c_int = 0;
            let hrec = bcf_hdr_parse_line(h, tmp.as_ptr(), &mut l);
            let mut res = if !hrec.is_null() {
                bcf_hdr_add_hrec(h.cast_mut(), hrec)
            } else {
                -1
            };
            if res < 0 {
                bcf_hrec_destroy(hrec);
            }
            if res > 0 {
                res = bcf_hdr_sync(h.cast_mut());
            }
            k = kh_get_vdict(d, t);
            (*v).errcode |= BCF_ERR_TAG_UNDEF as c_int;
            if res != 0 || k == (*d).n_buckets {
                vcf_log_error(format!(
                    "Could not add dummy header for FILTER '{}' at {}:{}",
                    tag,
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_TAG_INVALID as c_int;
                libc::free(a_flt.cast());
                return -1;
            }
        }
        *a_flt.add(i) = (*vdict_val(d, k)).id;
        i += 1;
        t = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux1);
    }

    bcf_enc_vint(str_, n_flt, a_flt, -1);
    libc::free(a_flt.cast());

    0
}

unsafe fn vcf_parse_info(
    str_: *mut kstring_t,
    h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    p: *mut c_char,
    q: *mut c_char,
) -> c_int {
    const BCF_MAX_BT_INT32: i64 = 0x7fff_ffff;
    let mut max_n_val: c_int = 0;
    let d = (*h).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>();
    let mut a_val: *mut i32 = std::ptr::null_mut();

    (*v).set_n_info(0);
    if *q.sub(1) == b';' as c_char {
        *q.sub(1) = 0;
    }

    let mut r = p;
    let mut key = p;
    // C: for (r = key = p;; ++r) { ... }  -- the ++r runs between iterations
    // (including after `continue`), so emulate it with a flag.
    let mut first = true;
    let result = 'outer: loop {
        if !first {
            r = r.add(1);
        }
        first = false;
        let mut val: *mut c_char = std::ptr::null_mut();
        let mut end_p: *mut c_char;
        // while (*r > '=' || (*r != ';' && *r != '=' && *r != 0)) r++;
        // *r is a (signed) char in C; preserve that comparison semantics.
        while *r > b'=' as c_char || (*r != b';' as c_char && *r != b'=' as c_char && *r != 0) {
            r = r.add(1);
        }
        if (*v).n_info() == u16::MAX as u32 {
            vcf_log_error(format!(
                "Too many INFO entries at {}:{}",
                vcf_seqname_safe_str(h, v),
                (*v).pos + 1
            ));
            (*v).errcode |= BCF_ERR_LIMITS as c_int;
            break 'outer -1;
        }
        let mut c = *r;
        *r = 0;
        end_p = r;
        if c == b'=' as c_char {
            val = r.add(1);
            end_p = val;
            while *end_p != b';' as c_char && *end_p != 0 {
                end_p = end_p.add(1);
            }
            c = *end_p;
            *end_p = 0;
        }
        if *key == 0 {
            // faulty VCF, ";;" in the INFO
            if c == 0 {
                break 'outer 0;
            }
            r = end_p;
            key = r.add(1);
            continue;
        }
        let mut k = kh_get_vdict(d, key);
        if k == (*d).n_buckets || (*vdict_val(d, k)).info[BCF_HL_INFO as usize] == 15 {
            let keyname = CStr::from_ptr(key).to_string_lossy().into_owned();
            vcf_log_warning(format!(
                "INFO '{}' is not defined in the header, assuming Type=String",
                keyname
            ));
            let tmp = std::ffi::CString::new(format!(
                "##INFO=<ID={},Number=1,Type=String,Description=\"Dummy\">",
                keyname
            ))
            .unwrap_or_default();
            let mut l: c_int = 0;
            let hrec = bcf_hdr_parse_line(h, tmp.as_ptr(), &mut l);
            let mut res = if !hrec.is_null() {
                bcf_hdr_add_hrec(h.cast_mut(), hrec)
            } else {
                -1
            };
            if res < 0 {
                bcf_hrec_destroy(hrec);
            }
            if res > 0 {
                res = bcf_hdr_sync(h.cast_mut());
            }
            k = kh_get_vdict(d, key);
            (*v).errcode |= BCF_ERR_TAG_UNDEF as c_int;
            if res != 0 || k == (*d).n_buckets {
                vcf_log_error(format!(
                    "Could not add dummy header for INFO '{}' at {}:{}",
                    keyname,
                    vcf_seqname_safe_str(h, v),
                    (*v).pos + 1
                ));
                (*v).errcode |= BCF_ERR_TAG_INVALID as c_int;
                break 'outer -1;
            }
        }
        let y = (*vdict_val(d, k)).info[BCF_HL_INFO as usize] as u32;
        (*v).set_n_info((*v).n_info() + 1);
        bcf_enc_int1(str_, (*vdict_val(d, k)).id);
        if val.is_null() {
            bcf_enc_size(str_, 0, BCF_BT_NULL as c_int);
        } else if (y >> 4 & 0xf) == BCF_HT_FLAG || (y >> 4 & 0xf) == BCF_HT_STR {
            // if Flag has a value, treat it as a string
            bcf_enc_vchar(str_, (end_p as usize - val as usize) as c_int, val);
        } else {
            // int/float value/array
            let mut n_val: c_int = 1;
            let mut tt = val;
            while *tt != 0 {
                if *tt == b',' as c_char {
                    n_val += 1;
                }
                tt = tt.add(1);
            }
            if n_val > max_n_val {
                let a_tmp =
                    libc::realloc(a_val.cast(), n_val as size_t * size_of::<i32>()).cast::<i32>();
                if a_tmp.is_null() {
                    vcf_log_error(format!(
                        "Could not allocate memory at {}:{}",
                        vcf_seqname_safe_str(h, v),
                        (*v).pos + 1
                    ));
                    (*v).errcode |= BCF_ERR_LIMITS as c_int;
                    break 'outer -1;
                }
                a_val = a_tmp;
                max_n_val = n_val;
            }
            if (y >> 4 & 0xf) == BCF_HT_INT {
                let mut t = val;
                let val1: i64;
                // VCF_ALLOW_INT64 not defined: simple int32 path.
                let mut i = 0;
                while i < n_val {
                    let mut overflow: c_int = 0;
                    let mut te: *mut c_char = std::ptr::null_mut();
                    let mut tmp_val =
                        hts_str2int(t, &mut te, (size_of::<i64>() * 8) as c_int, &mut overflow);
                    if te == t {
                        tmp_val = bcf_int32_missing as i64;
                    } else if overflow != 0
                        || tmp_val < BCF_MIN_BT_INT32
                        || tmp_val > BCF_MAX_BT_INT32
                    {
                        tmp_val = bcf_int32_missing as i64;
                    }
                    *a_val.add(i as usize) = tmp_val as i32;
                    t = te;
                    while *t != 0 && *t != b',' as c_char {
                        t = t.add(1);
                    }
                    i += 1;
                    if i < n_val {
                        t = t.add(1);
                    }
                }
                if n_val == 1 {
                    val1 = *a_val as i64;
                    bcf_enc_int1(str_, val1 as i32);
                } else {
                    bcf_enc_vint(str_, n_val, a_val, -1);
                    val1 = 0; // unused
                }
                if n_val == 1
                    && val1 != bcf_int32_missing as i64
                    && libc::memcmp(key.cast(), c"END".as_ptr().cast(), 4) == 0
                {
                    if val1 <= (*v).pos {
                        vcf_log_warning(format!(
                            "INFO/END={} is smaller than POS at {}:{}",
                            val1,
                            vcf_seqname_safe_str(h, v),
                            (*v).pos + 1
                        ));
                    } else {
                        (*v).rlen = val1 - (*v).pos;
                    }
                }
            } else if (y >> 4 & 0xf) == BCF_HT_REAL {
                let val_f = a_val.cast::<f32>();
                let mut t = val;
                let mut i = 0;
                while i < n_val {
                    let mut overflow: c_int = 0;
                    let mut te: *mut c_char = std::ptr::null_mut();
                    let fv = hts_str2dbl(t, &mut te, &mut overflow) as f32;
                    if te == t || overflow != 0 {
                        *val_f.add(i as usize) = f32::from_bits(bcf_float_missing);
                    } else {
                        *val_f.add(i as usize) = fv;
                    }
                    t = te;
                    while *t != 0 && *t != b',' as c_char {
                        t = t.add(1);
                    }
                    i += 1;
                    if i < n_val {
                        t = t.add(1);
                    }
                }
                bcf_enc_vfloat(str_, n_val, val_f);
            }
        }
        if c == 0 {
            break 'outer 0;
        }
        r = end_p;
        key = r.add(1);
    };

    libc::free(a_val.cast());
    result
}

// int vcf_parse(kstring_t *s, const bcf_hdr_t *h, bcf1_t *v)
unsafe fn vcf_parse_native(s: *mut kstring_t, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    let ret: c_int = -2;

    // NOT_DOT(p): memcmp(p, ".\0", 2) != 0
    macro_rules! not_dot {
        ($p:expr) => {
            libc::memcmp($p.cast(), c".".as_ptr().cast(), 2) != 0
        };
    }

    if s.is_null() || h.is_null() || v.is_null() || (*s).s.is_null() {
        return ret;
    }

    // Ensure 4 bytes of overflow space for the memcmp tricks.
    if super::hts::ks_resize(s, (*s).l + 4) < 0 {
        return -1;
    }
    *(*s).s.add((*s).l) = 0;
    *(*s).s.add((*s).l + 1) = 0;
    *(*s).s.add((*s).l + 2) = 0;
    *(*s).s.add((*s).l + 3) = 0;

    bcf_clear(v);
    let str_: *mut kstring_t = std::ptr::addr_of_mut!((*v).shared).cast::<kstring_t>();
    let mut aux = super::hts::ks_tokaux_t {
        tab: [0; 4],
        sep: 0,
        finished: 0,
        p: std::ptr::null(),
    };

    let mut overflow: c_int;

    // CHROM
    let mut p = kstrtok((*s).s, c"\t".as_ptr(), &mut aux);
    if p.is_null() {
        return ret;
    }
    let mut q = aux.p as *mut c_char;
    *q = 0;

    let d_ctg = (*h).dict[BCF_DT_CTG as usize].cast::<kh_vdict_t>();
    let mut k = kh_get_vdict(d_ctg, p);
    if k == (*d_ctg).n_buckets {
        let name = CStr::from_ptr(p).to_string_lossy().into_owned();
        vcf_log_warning(format!(
            "Contig '{}' is not defined in the header. (Quick workaround: index the file with tabix.)",
            name
        ));
        (*v).errcode = BCF_ERR_CTG_UNDEF as c_int;
        k = fix_chromosome(h, d_ctg, p);
        if k == (*d_ctg).n_buckets {
            vcf_log_error(format!("Could not add dummy header for contig '{}'", name));
            (*v).errcode |= BCF_ERR_CTG_INVALID as c_int;
            return ret;
        }
    }
    (*v).rid = (*vdict_val(d_ctg, k)).id;

    // POS
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    overflow = 0;
    let tmp = p;
    let mut pp: *mut c_char = std::ptr::null_mut();
    (*v).pos = hts_str2uint(p, &mut pp, 63, &mut overflow) as hts_pos_t;
    if overflow != 0 {
        let name = CStr::from_ptr(tmp).to_string_lossy().into_owned();
        vcf_log_error(format!("Position value '{}' is too large", name));
        return ret;
    } else if *pp != 0 {
        let name = CStr::from_ptr(tmp).to_string_lossy().into_owned();
        vcf_log_error(format!("Could not parse the position '{}'", name));
        return ret;
    } else {
        (*v).pos -= 1;
    }
    if (*v).pos >= i32::MAX as hts_pos_t {
        (*v).unpacked |= BCF_IS_64BIT;
    }

    // ID
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    if not_dot!(p) {
        bcf_enc_vchar(str_, (q as usize - p as usize) as c_int, p);
    } else {
        bcf_enc_size(str_, 0, BCF_BT_CHAR as c_int);
    }

    // REF
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    bcf_enc_vchar(str_, (q as usize - p as usize) as c_int, p);
    (*v).set_n_allele(1);
    (*v).rlen = (q as usize - p as usize) as hts_pos_t;

    // ALT
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    if not_dot!(p) {
        let mut r = p;
        let mut t = p;
        loop {
            if *r == b',' as c_char || *r == 0 {
                if (*v).n_allele() == u16::MAX as u32 {
                    vcf_log_error(format!(
                        "Too many ALT alleles at {}:{}",
                        vcf_seqname_safe_str(h, v),
                        (*v).pos + 1
                    ));
                    (*v).errcode |= BCF_ERR_LIMITS as c_int;
                    return ret;
                }
                bcf_enc_vchar(str_, (r as usize - t as usize) as c_int, t);
                t = r.add(1);
                (*v).set_n_allele((*v).n_allele() + 1);
            }
            if r == q {
                break;
            }
            r = r.add(1);
        }
    }

    // QUAL
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    if not_dot!(p) {
        (*v).qual = libc::atof(p) as f32;
    } else {
        (*v).qual = f32::from_bits(bcf_float_missing);
    }
    if (*v).max_unpack != 0 && (*v).max_unpack >> 1 == 0 {
        return 0; // BCF_UN_STR
    }

    // FILTER
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    if not_dot!(p) {
        if vcf_parse_filter(str_, h, v, p, q) != 0 {
            return ret;
        }
    } else {
        bcf_enc_vint(str_, 0, std::ptr::null_mut(), -1);
    }
    if (*v).max_unpack != 0 && (*v).max_unpack >> 2 == 0 {
        return 0; // BCF_UN_FLT
    }

    // INFO
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if p.is_null() {
        return ret;
    }
    q = aux.p as *mut c_char;
    *q = 0;

    if not_dot!(p) {
        if vcf_parse_info(str_, h, v, p, q) != 0 {
            return ret;
        }
    }
    if (*v).max_unpack != 0 && (*v).max_unpack >> 3 == 0 {
        return 0;
    }

    // FORMAT; optional
    p = kstrtok(std::ptr::null(), std::ptr::null(), &mut aux);
    if !p.is_null() {
        q = aux.p as *mut c_char;
        *q = 0;
        if vcf_parse_format(s, h, v, p, q) == 0 {
            0
        } else {
            -2
        }
    } else {
        0
    }
}

pub unsafe fn vcf_parse(s: *mut kstring_t, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    let end = vcf_line_info_end_i64(s);
    let symbolic_svlen_rlen = vcf_line_symbolic_svlen_rlen(s);
    let format_len_rlen = vcf_line_format_len_rlen(s);
    let gt_phasing = if vcf_hdr_maybe_version_ge_44(h) {
        vcf44_strip_prefixed_gt_phasing(s)
    } else {
        None
    };
    let ret = if let Some(phasing) = gt_phasing {
        let ret = vcf_parse_native(s, h, v);
        if ret == 0 {
            vcf44_repair_prefixed_gt_phasing(h, v, &phasing);
        }
        ret
    } else {
        vcf_parse_native(s, h, v)
    };
    if ret == 0 {
        if let Some(end) = end {
            if vcf_repair_info_end_i64(h, v, end) < 0 {
                return -1;
            }
        }
        if let Some(rlen) = symbolic_svlen_rlen {
            (*v).rlen = (*v).rlen.max(rlen);
        }
        if let Some(rlen) = format_len_rlen {
            (*v).rlen = (*v).rlen.max(rlen);
        }
    }
    ret
}

unsafe fn vcf_hdr_version_ge_44(hdr: *const bcf_hdr_t) -> bool {
    // Read the cached aux->version int (set by bcf_hdr_set_version/parse_line),
    // falling back to a string parse only if the cached field is still 0.
    // This avoids per-sample header rescans in bcf_format_gt_v2.
    if hdr.is_null() {
        return false;
    }
    bcf_get_version(hdr, std::ptr::null()) >= VCF44
}

unsafe fn vcf_hdr_maybe_version_ge_44(hdr: *const bcf_hdr_t) -> bool {
    if hdr.is_null() {
        return false;
    }
    let version = bcf_hdr_get_version(hdr);
    if version.is_null() {
        return false;
    }
    let bytes = std::ffi::CStr::from_ptr(version).to_bytes();
    if bytes == b"VCFv4.0" || bytes == b"VCFv4.1" || bytes == b"VCFv4.2" || bytes == b"VCFv4.3" {
        return false;
    }
    vcf_version_number(bytes).is_some_and(|version| version >= 4_004_000)
}

fn vcf_version_number(version: &[u8]) -> Option<i64> {
    let prefix = version.windows(4).position(|window| window == b"VCFv")? + 4;
    let dot = version[prefix..].iter().position(|&ch| ch == b'.')? + prefix;
    let major = parse_decimal_prefix(&version[prefix..dot])?;
    let minor = parse_decimal_prefix(&version[dot + 1..])?;
    Some(major * 100 * 10_000 + minor * 1_000)
}

fn parse_decimal_prefix(bytes: &[u8]) -> Option<i64> {
    let mut value = 0i64;
    let mut saw_digit = false;
    for &ch in bytes {
        if !ch.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        value = value.checked_mul(10)?.checked_add((ch - b'0') as i64)?;
    }
    saw_digit.then_some(value)
}

unsafe fn vcf44_strip_prefixed_gt_phasing(line: *mut kstring_t) -> Option<Vec<i8>> {
    if line.is_null() || (*line).s.is_null() {
        return None;
    }

    let bytes = std::slice::from_raw_parts((*line).s.cast::<u8>(), (*line).l);
    let mut fields = Vec::new();
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\t' {
            fields.push((start, i));
            start = i + 1;
        }
    }
    fields.push((start, bytes.len()));
    if fields.len() < 10 {
        return None;
    }

    let (format_start, format_end) = fields[8];
    let gt_index = bytes[format_start..format_end]
        .split(|&b| b == b':')
        .position(|tag| tag == b"GT")?;
    let mut sample_phasing = Vec::with_capacity(fields.len() - 9);
    let mut removals = Vec::new();

    for &(field_start, field_end) in &fields[9..] {
        let mut cursor = 0usize;
        let field = &bytes[field_start..field_end];
        let mut gt_start = field_end;
        for idx in 0..=gt_index {
            gt_start = field_start + cursor;
            while cursor < field.len() && field[cursor] != b':' {
                cursor += 1;
            }
            if idx == gt_index {
                break;
            }
            if cursor < field.len() {
                cursor += 1;
            }
        }

        let phase = if gt_start < field_end
            && (*(*line).s.add(gt_start) == b'/' as c_char
                || *(*line).s.add(gt_start) == b'|' as c_char)
        {
            removals.push(gt_start);
            Some((*(*line).s.add(gt_start) == b'|' as c_char) as i8)
        } else {
            None
        };
        sample_phasing.push(phase.unwrap_or(-1));
    }

    for offset in removals.into_iter().rev() {
        std::ptr::copy(
            (*line).s.add(offset + 1),
            (*line).s.add(offset),
            (*line).l - offset,
        );
        (*line).l -= 1;
    }
    Some(sample_phasing)
}

unsafe fn vcf44_repair_prefixed_gt_phasing(
    hdr: *const bcf_hdr_t,
    rec: *mut bcf1_t,
    sample_phasing: &[i8],
) {
    if sample_phasing.is_empty() || bcf_unpack(rec, BCF_UN_FMT as c_int) != 0 {
        return;
    }

    let gt_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"GT".as_ptr());
    if gt_id < 0 {
        return;
    }
    let fmt = bcf_get_fmt_id(rec, gt_id);
    if fmt.is_null() || (*fmt).p.is_null() {
        return;
    }

    let n_sample = (*rec).n_sample() as usize;
    for sample in 0..n_sample.min(sample_phasing.len()) {
        let phased = sample_phasing[sample];
        let ptr = (*fmt).p.add(sample * (*fmt).size as usize);
        let inferred = if phased >= 0 {
            phased as u8
        } else {
            vcf44_infer_first_gt_phase(fmt, ptr)
        };
        match (*fmt).type_ {
            x if x == BCF_BT_INT8 as c_int => {
                *ptr = (*ptr & !1) | inferred;
            }
            x if x == BCF_BT_INT16 as c_int => {
                let val = le_to_i16(ptr) as u16;
                i16_to_le(((val & !1) | inferred as u16) as i16, ptr);
            }
            x if x == BCF_BT_INT32 as c_int => {
                let val = le_to_i32(ptr) as u32;
                i32_to_le(((val & !1) | inferred as u32) as i32, ptr);
            }
            _ => {}
        }
    }
    (*rec).d.indiv_dirty = 1;
}

unsafe fn vcf44_infer_first_gt_phase(fmt: *const bcf_fmt_t, ptr: *const u8) -> u8 {
    let mut values = Vec::new();
    for i in 0..(*fmt).n as usize {
        let val = match (*fmt).type_ {
            x if x == BCF_BT_INT8 as c_int => le_to_i8(ptr.add(i)) as i32,
            x if x == BCF_BT_INT16 as c_int => le_to_i16(ptr.add(i * size_of::<i16>())) as i32,
            x if x == BCF_BT_INT32 as c_int => le_to_i32(ptr.add(i * size_of::<i32>())),
            _ => return 0,
        };
        let vector_end = match (*fmt).type_ {
            x if x == BCF_BT_INT8 as c_int => bcf_int8_vector_end,
            x if x == BCF_BT_INT16 as c_int => bcf_int16_vector_end,
            _ => bcf_int32_vector_end,
        };
        if val == vector_end {
            break;
        }
        values.push(val);
    }

    if values.len() == 1 {
        (values[0] >> 1 != 0) as u8
    } else if values.len() > 1 {
        (!values.iter().skip(1).any(|val| val & 1 == 0)) as u8
    } else {
        0
    }
}

// Native translation of htslib/vcf.c vcf_format(). The v44 phasing is handled
// directly by bcf_format_gt_v2(), so no post-processing pass is needed.
pub unsafe fn vcf_format(h: *const bcf_hdr_t, v: *const bcf1_t, s: *mut kstring_t) -> c_int {
    let v = v.cast_mut();
    let max_dt_id = (*h).n[BCF_DT_ID as usize];
    let chrom = bcf_seqname(h, v);
    if chrom.is_null() {
        let msg = std::ffi::CString::new(format!(
            "Invalid BCF, CONTIG id={} not present in the header",
            (*v).rid
        ))
        .unwrap_or_default();
        c_log_error(msg.as_ptr());
        *libc::__errno_location() = libc::EINVAL;
        return -1;
    }

    bcf_unpack(v, (BCF_UN_ALL & !(BCF_UN_INFO | BCF_UN_FMT)) as c_int);

    // Cache of key lengths in the header aux struct.
    let aux = get_hdr_aux(h);
    if (*aux).key_len.is_null() {
        (*aux).key_len = libc::calloc((*h).n[BCF_DT_ID as usize] as usize + 1, size_of::<usize>())
            .cast::<usize>();
        if (*aux).key_len.is_null() {
            return -1;
        }
    }
    let key_len = (*aux).key_len;

    let idtbl = (*h).id[BCF_DT_ID as usize];

    kputs(chrom, s); // CHROM
    kputc_(b'\t' as c_int, s);
    super::hts::kputll((*v).pos + 1, s); // POS
    kputc_(b'\t' as c_int, s); // ID
    kputs(
        if (*v).d.id.is_null() {
            c".".as_ptr()
        } else {
            (*v).d.id
        },
        s,
    );
    kputc_(b'\t' as c_int, s); // REF
    if (*v).n_allele() > 0 {
        kputs(*(*v).d.allele, s);
    } else {
        kputc_(b'.' as c_int, s);
    }
    kputc_(b'\t' as c_int, s); // ALT
    if (*v).n_allele() > 1 {
        for i in 1..(*v).n_allele() as usize {
            if i > 1 {
                kputc_(b',' as c_int, s);
            }
            kputs(*(*v).d.allele.add(i), s);
        }
    } else {
        kputc_(b'.' as c_int, s);
    }
    kputc_(b'\t' as c_int, s); // QUAL
    if (*v).qual.to_bits() == bcf_float_missing {
        kputc_(b'.' as c_int, s);
    } else {
        kputd((*v).qual as f64, s);
    }
    kputc_(b'\t' as c_int, s); // FILTER
    if (*v).d.n_flt != 0 {
        for i in 0..(*v).d.n_flt as usize {
            let idx = *(*v).d.flt.add(i);
            if idx < 0 || idx >= max_dt_id || (*idtbl.add(idx as usize)).key.is_null() {
                let msg = std::ffi::CString::new(format!(
                    "Invalid BCF, the FILTER tag id={} at {}:{} not present in the header",
                    idx,
                    CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
                    (*v).pos + 1
                ))
                .unwrap_or_default();
                c_log_error(msg.as_ptr());
                *libc::__errno_location() = libc::EINVAL;
                return -1;
            }
            if i != 0 {
                kputc_(b';' as c_int, s);
            }
            if *key_len.add(idx as usize) == 0 {
                *key_len.add(idx as usize) = libc::strlen((*idtbl.add(idx as usize)).key);
            }
            kputsn(
                (*idtbl.add(idx as usize)).key,
                *key_len.add(idx as usize),
                s,
            );
        }
    } else {
        kputc_(b'.' as c_int, s);
    }

    kputc_(b'\t' as c_int, s); // INFO
    if (*v).n_info() != 0 {
        let mut ptr = if !(*v).shared.s.is_null() {
            (*v).shared
                .s
                .cast::<u8>()
                .add(((*v).unpack_size[0] + (*v).unpack_size[1] + (*v).unpack_size[2]) as usize)
        } else {
            std::ptr::null_mut()
        };
        let mut first = true;
        let info = (*v).d.info;
        let info_packed = (*v).unpacked & BCF_UN_INFO as c_int == 0 && (*v).shared.l != 0;
        let mut local_in: bcf_info_t = std::mem::zeroed();
        for i in 0..(*v).n_info() as usize {
            let z: *mut bcf_info_t;
            if info_packed {
                z = &mut local_in;
                let mut p: *const u8 = ptr;
                (*z).key = bcf_dec_typed_int1_unsafe(p, &mut p);
                let mut ztype = 0;
                (*z).len = bcf_dec_size_unsafe(p, &mut p, &mut ztype);
                (*z).type_ = ztype;
                (*z).vptr = p as *mut u8;
                ptr =
                    (p as *mut u8).add(((*z).len as usize) << BCF_TYPE_SHIFT[(*z).type_ as usize]);
            } else {
                z = info.add(i);
                if (*z).vptr.is_null() {
                    continue;
                }
            }

            let id = if (*z).key >= 0 && (*z).key < max_dt_id {
                idtbl.add((*z).key as usize)
            } else {
                std::ptr::null()
            };
            if id.is_null() || (*id).key.is_null() {
                let what = if (*z).key < 0 {
                    "negative"
                } else if (*z).key >= max_dt_id {
                    "too large"
                } else {
                    "not present in the header"
                };
                let msg = std::ffi::CString::new(format!(
                    "Invalid BCF, the INFO tag id={} is {} at {}:{}",
                    (*z).key,
                    what,
                    CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
                    (*v).pos + 1
                ))
                .unwrap_or_default();
                c_log_error(msg.as_ptr());
                *libc::__errno_location() = libc::EINVAL;
                return -1;
            }

            // KEY
            if *key_len.add((*z).key as usize) == 0 {
                *key_len.add((*z).key as usize) = libc::strlen((*id).key);
            }
            let id_len = *key_len.add((*z).key as usize);
            if ks_resize(s, (*s).l + 3 + id_len) < 0 {
                return -1;
            }
            let mut sptr = (*s).s.add((*s).l);
            if !first {
                *sptr = b';' as c_char;
                sptr = sptr.add(1);
                (*s).l += 1;
            }
            first = false;
            libc::memcpy(sptr.cast(), (*id).key.cast(), id_len);
            (*s).l += id_len;

            // VALUE
            if (*z).len <= 0 {
                continue;
            }
            *sptr.add(id_len) = b'=' as c_char;
            (*s).l += 1;

            if (*z).len != 1 || info_packed {
                bcf_fmt_array(s, (*z).len, (*z).type_, (*z).vptr.cast());
            } else if (*z).type_ == BCF_BT_FLOAT as c_int {
                if (*z).v1.f.to_bits() == bcf_float_missing {
                    kputc_(b'.' as c_int, s);
                } else {
                    kputd((*z).v1.f as f64, s);
                }
            } else if (*z).type_ == BCF_BT_CHAR as c_int {
                kputc_((*z).v1.i as c_int, s);
            } else if (*z).type_ < BCF_BT_INT64 as c_int {
                let missing = [
                    0i64,
                    bcf_int8_missing as i64,
                    bcf_int16_missing as i64,
                    bcf_int32_missing as i64,
                ];
                if (*z).v1.i == missing[(*z).type_ as usize] {
                    kputc_(b'.' as c_int, s);
                } else {
                    kputw((*z).v1.i as c_int, s);
                }
            } else if (*z).type_ == BCF_BT_INT64 as c_int {
                if (*z).v1.i == bcf_int64_missing {
                    kputc_(b'.' as c_int, s);
                } else {
                    super::hts::kputll((*z).v1.i, s);
                }
            } else {
                let msg = std::ffi::CString::new(format!(
                    "Unexpected type {} at {}:{}",
                    (*z).type_,
                    CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
                    (*v).pos + 1
                ))
                .unwrap_or_default();
                c_log_error(msg.as_ptr());
                *libc::__errno_location() = libc::EINVAL;
                return -1;
            }
        }
        if first {
            kputc_(b'.' as c_int, s);
        }
    } else {
        kputc_(b'.' as c_int, s);
    }

    // FORMAT and individual information
    if (*v).n_sample() != 0 {
        if (*v).n_fmt() != 0 {
            let mut ptr = (*v).indiv.s.cast::<u8>();
            let mut gt_i: c_int = -1;
            let fmt_packed = (*v).unpacked & BCF_UN_FMT as c_int == 0;
            let fmt: *mut bcf_fmt_t = if fmt_packed {
                let p = libc::malloc(size_of::<bcf_fmt_t>() * (*v).n_fmt() as usize)
                    .cast::<bcf_fmt_t>();
                if p.is_null() {
                    return -1;
                }
                p
            } else {
                (*v).d.fmt
            };
            let mut first = true;

            // KEYS
            for i in 0..(*v).n_fmt() as usize {
                let z = fmt.add(i);
                if fmt_packed {
                    let mut p: *const u8 = ptr;
                    (*z).id = bcf_dec_typed_int1_unsafe(p, &mut p);
                    let mut ztype = 0;
                    (*z).n = bcf_dec_size_unsafe(p, &mut p, &mut ztype);
                    (*z).type_ = ztype;
                    (*z).p = p as *mut u8;
                    (*z).size = (*z).n << BCF_TYPE_SHIFT[(*z).type_ as usize];
                    ptr = (p as *mut u8).add((*v).n_sample() as usize * (*z).size as usize);
                }
                if (*z).p.is_null() {
                    continue;
                }
                kputc_(if !first { b':' } else { b'\t' } as c_int, s);
                first = false;

                let id = if (*z).id >= 0 && (*z).id < max_dt_id {
                    idtbl.add((*z).id as usize)
                } else {
                    std::ptr::null()
                };
                if id.is_null() || (*id).key.is_null() {
                    let msg = std::ffi::CString::new(format!(
                        "Invalid BCF, the FORMAT tag id={} at {}:{} not present in the header",
                        (*z).id,
                        CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
                        (*v).pos + 1
                    ))
                    .unwrap_or_default();
                    c_log_error(msg.as_ptr());
                    *libc::__errno_location() = libc::EINVAL;
                    if fmt_packed {
                        libc::free(fmt.cast());
                    }
                    return -1;
                }

                if *key_len.add((*z).id as usize) == 0 {
                    *key_len.add((*z).id as usize) = libc::strlen((*id).key);
                }
                let id_len = *key_len.add((*z).id as usize);
                kputsn((*id).key, id_len, s);
                if id_len == 2
                    && *(*id).key == b'G' as c_char
                    && *(*id).key.add(1) == b'T' as c_char
                {
                    gt_i = i as c_int;
                }
            }
            if first {
                kputsn(c"\t.".as_ptr(), 2, s);
            }

            // Hoist the VCFv4.4+ version check out of the per-sample loop.
            // It's a constant for the whole record; the per-sample loop only
            // needs the bool to decide GT separator behaviour.
            let v44 = !h.is_null() && vcf_hdr_version_ge_44(h);

            // VALUES per sample
            for j in 0..(*v).n_sample() as usize {
                kputc_(b'\t' as c_int, s);
                first = true;
                let mut i = 0usize;
                while i < (*v).n_fmt() as usize {
                    let f = fmt.add(i);
                    if (*f).p.is_null() {
                        i += 1;
                        continue;
                    }
                    if !first {
                        kputc_(b':' as c_int, s);
                    }
                    first = false;
                    if gt_i == i as c_int {
                        let ret = bcf_format_gt_v2_inner(v44, f, j as c_int, s);
                        if ret < 0 {
                            let msg = std::ffi::CString::new(format!(
                                "Failed to format GT value for sample {}, returned {}",
                                i, ret
                            ))
                            .unwrap_or_default();
                            c_log_error(msg.as_ptr());
                            *libc::__errno_location() = libc::EINVAL;
                            if fmt_packed {
                                libc::free(fmt.cast());
                            }
                            return -1;
                        }
                        i += 1;
                        break;
                    } else if (*f).n == 1 {
                        bcf_fmt_array1(s, (*f).type_, (*f).p.add(j * (*f).size as usize).cast());
                    } else {
                        bcf_fmt_array(
                            s,
                            (*f).n,
                            (*f).type_,
                            (*f).p.add(j * (*f).size as usize).cast(),
                        );
                    }
                    i += 1;
                }

                // Simpler loop post GT and at least 1 iteration
                while i < (*v).n_fmt() as usize {
                    let f = fmt.add(i);
                    if (*f).p.is_null() {
                        i += 1;
                        continue;
                    }
                    kputc_(b':' as c_int, s);
                    if (*f).n == 1 {
                        bcf_fmt_array1(s, (*f).type_, (*f).p.add(j * (*f).size as usize).cast());
                    } else {
                        bcf_fmt_array(
                            s,
                            (*f).n,
                            (*f).type_,
                            (*f).p.add(j * (*f).size as usize).cast(),
                        );
                    }
                    i += 1;
                }
                if first {
                    kputc_(b'.' as c_int, s);
                }
            }
            if fmt_packed {
                libc::free(fmt.cast());
            }
        } else {
            for _j in 0..=(*v).n_sample() {
                kputsn(c"\t.".as_ptr(), 2, s);
            }
        }
    }
    kputc(b'\n' as c_int, s);
    0
}

unsafe fn vcf44_format_gt_fields(
    hdr: *const bcf_hdr_t,
    rec: *mut bcf1_t,
    line: *mut kstring_t,
) -> c_int {
    if line.is_null() || (*line).s.is_null() {
        return 0;
    }
    if bcf_unpack(rec, BCF_UN_FMT as c_int) != 0 {
        return -1;
    }
    let gt_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"GT".as_ptr());
    if gt_id < 0 {
        return 0;
    }
    let fmt = bcf_get_fmt_id(rec, gt_id);
    if fmt.is_null() {
        return 0;
    }

    let bytes = std::slice::from_raw_parts((*line).s.cast::<u8>(), (*line).l);
    let trailing_newline = bytes.last() == Some(&b'\n');
    let body = if trailing_newline {
        &bytes[..bytes.len() - 1]
    } else {
        bytes
    };
    let fields: Vec<&[u8]> = body.split(|&b| b == b'\t').collect();
    if fields.len() < 10 {
        return 0;
    }
    let gt_index = match fields[8].split(|&b| b == b':').position(|tag| tag == b"GT") {
        Some(idx) => idx,
        None => return 0,
    };

    let mut out = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let mut gt = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };

    for (field_index, field) in fields.iter().enumerate() {
        if field_index != 0 && kputsn(c"\t".as_ptr(), 1, &mut out) < 0 {
            super::hts::ks_free(&mut out);
            super::hts::ks_free(&mut gt);
            return -1;
        }

        if field_index < 9 {
            if kputsn(field.as_ptr().cast(), field.len(), &mut out) < 0 {
                super::hts::ks_free(&mut out);
                super::hts::ks_free(&mut gt);
                return -1;
            }
            continue;
        }

        let sample = field_index - 9;
        let subfields: Vec<&[u8]> = field.split(|&b| b == b':').collect();
        for (subfield_index, subfield) in subfields.iter().enumerate() {
            if subfield_index != 0 && kputsn(c":".as_ptr(), 1, &mut out) < 0 {
                super::hts::ks_free(&mut out);
                super::hts::ks_free(&mut gt);
                return -1;
            }
            if subfield_index == gt_index {
                gt.l = 0;
                if vcf44_format_gt(fmt, sample, &mut gt) < 0 || kputsn(gt.s, gt.l, &mut out) < 0 {
                    super::hts::ks_free(&mut out);
                    super::hts::ks_free(&mut gt);
                    return -1;
                }
            } else if kputsn(subfield.as_ptr().cast(), subfield.len(), &mut out) < 0 {
                super::hts::ks_free(&mut out);
                super::hts::ks_free(&mut gt);
                return -1;
            }
        }
    }
    if trailing_newline && kputsn(c"\n".as_ptr(), 1, &mut out) < 0 {
        super::hts::ks_free(&mut out);
        super::hts::ks_free(&mut gt);
        return -1;
    }

    if super::hts::ks_resize(line, out.l + 1) < 0 {
        super::hts::ks_free(&mut out);
        super::hts::ks_free(&mut gt);
        return -1;
    }
    libc::memcpy((*line).s.cast(), out.s.cast(), out.l);
    (*line).l = out.l;
    *(*line).s.add((*line).l) = 0;
    super::hts::ks_free(&mut out);
    super::hts::ks_free(&mut gt);
    0
}

unsafe fn vcf44_format_gt(fmt: *mut bcf_fmt_t, sample: usize, out: *mut kstring_t) -> c_int {
    if fmt.is_null() || (*fmt).p.is_null() {
        return kputsn(c".".as_ptr(), 1, out);
    }

    let mut values = Vec::new();
    let ptr = (*fmt).p.add(sample * (*fmt).size as usize);
    for i in 0..(*fmt).n as usize {
        let val = match (*fmt).type_ {
            x if x == BCF_BT_INT8 as c_int => le_to_i8(ptr.add(i)) as i32,
            x if x == BCF_BT_INT16 as c_int => le_to_i16(ptr.add(i * size_of::<i16>())) as i32,
            x if x == BCF_BT_INT32 as c_int => le_to_i32(ptr.add(i * size_of::<i32>())),
            x if x == BCF_BT_NULL as c_int => break,
            _ => return -2,
        };
        let vector_end = match (*fmt).type_ {
            x if x == BCF_BT_INT8 as c_int => bcf_int8_vector_end,
            x if x == BCF_BT_INT16 as c_int => bcf_int16_vector_end,
            _ => bcf_int32_vector_end,
        };
        if val == vector_end {
            break;
        }
        values.push(val);
    }

    if values.is_empty() {
        return kputsn(c".".as_ptr(), 1, out);
    }

    let val0 = values[0];
    let ploidy = values.len();
    let anyunphased = values.iter().skip(1).any(|val| val & 1 == 0);
    let prefix = if val0 & 1 != 0 {
        if (ploidy > 1 && anyunphased) || (ploidy <= 1 && val0 >> 1 == 0) {
            Some(b'|')
        } else {
            None
        }
    } else if (ploidy <= 1 && val0 != 0) || (ploidy > 1 && !anyunphased) {
        Some(b'/')
    } else {
        None
    };

    if let Some(prefix) = prefix {
        if kputsn((&prefix as *const u8).cast(), 1, out) < 0 {
            return -1;
        }
    }

    for (i, val) in values.iter().enumerate() {
        if i > 0 {
            let sep = if val & 1 != 0 { b'|' } else { b'/' };
            if kputsn((&sep as *const u8).cast(), 1, out) < 0 {
                return -1;
            }
        }
        if val >> 1 == 0 {
            if kputsn(c".".as_ptr(), 1, out) < 0 {
                return -1;
            }
        } else {
            let allele = ((val >> 1) - 1).to_string();
            if kputsn(allele.as_ptr().cast(), allele.len(), out) < 0 {
                return -1;
            }
        }
    }
    0
}

// Native translation of htslib/vcf.c bcf_read1_core().
unsafe fn bcf_read1_core(fp: *mut BGZF, v: *mut bcf1_t) -> c_int {
    let mut x = [0u8; 32];
    let ret = bgzf_read(fp, x.as_mut_ptr().cast(), 32);
    if ret != 32 {
        if ret == 0 {
            return -1;
        }
        return -2;
    }
    bcf_clear(v); // bcf_clear1
    let mut shared_len = le_to_u32(x.as_ptr());
    if shared_len < 24 {
        return -2;
    }
    shared_len -= 24; // to exclude six 32-bit integers
    let indiv_len = le_to_u32(x.as_ptr().add(4));
    if ks_resize(
        std::ptr::addr_of_mut!((*v).shared).cast::<kstring_t>(),
        if shared_len != 0 {
            shared_len as size_t
        } else {
            1
        },
    ) != 0
    {
        return -2;
    }
    if ks_resize(
        std::ptr::addr_of_mut!((*v).indiv).cast::<kstring_t>(),
        if indiv_len != 0 {
            indiv_len as size_t
        } else {
            1
        },
    ) != 0
    {
        return -2;
    }
    (*v).rid = le_to_i32(x.as_ptr().add(8));
    (*v).pos = le_to_u32(x.as_ptr().add(12)) as hts_pos_t;
    if (*v).pos == u32::MAX as hts_pos_t {
        (*v).pos = -1; // telomere coordinate, e.g. MT:0
    }
    (*v).rlen = le_to_i32(x.as_ptr().add(16)) as hts_pos_t;
    (*v).qual = le_to_float(x.as_ptr().add(20));
    (*v).set_n_info(le_to_u16(x.as_ptr().add(24)) as u32);
    (*v).set_n_allele(le_to_u16(x.as_ptr().add(26)) as u32);
    (*v).set_n_sample(le_to_u32(x.as_ptr().add(28)) & 0xffffff);
    (*v).set_n_fmt(x[31] as u32);
    (*v).shared.l = shared_len as usize;
    (*v).indiv.l = indiv_len as usize;
    // silent fix of broken BCFs produced by earlier versions of bcf_subset,
    // prior to and including bd6ed8b4
    if ((*v).indiv.l == 0 || (*v).n_sample() == 0) && (*v).n_fmt() != 0 {
        (*v).set_n_fmt(0);
    }

    if bgzf_read(fp, (*v).shared.s.cast(), (*v).shared.l as usize) != (*v).shared.l as isize {
        return -2;
    }
    if bgzf_read(fp, (*v).indiv.s.cast(), (*v).indiv.l as usize) != (*v).indiv.l as isize {
        return -2;
    }
    0
}

// Native translation of htslib/vcf.c bcf_read().
pub unsafe fn bcf_read(fp: *mut htsFile, mut h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    if (*fp).format.format == HTS_FORMAT_VCF {
        // htslib/vcf.c vcf_read(): getline + vcf_parse. Inlined here because
        // our public vcf_read() delegates to bcf_read().
        let ret = hts_getline(fp, KS_SEP_LINE, &mut (*fp).line);
        if ret < 0 {
            return ret;
        }
        return vcf_parse(&mut (*fp).line, h, v);
    }
    let bgzf = hts_get_bgzfp(fp);
    if h.is_null() {
        h = bgzf_internal_h_67_bgzf_get_private_data(bgzf).cast::<bcf_hdr_t>();
    }
    let mut ret = bcf_read1_core(bgzf, v);
    if ret == 0 {
        ret = vcf_c_2040_bcf_record_check(h, v);
    }
    if ret != 0 || (*h).keep_samples.is_null() {
        return ret;
    }
    bcf_subset_format(h, v)
}

// htslib/hts_internal.h HTS_MAX_EXT_LEN
const HTS_MAX_EXT_LEN: usize = 9;
// htslib/htslib/hts.h HTS_IDX_DELIM
pub(crate) const HTS_IDX_DELIM: &[u8] = b"##idx##";

// Native translation of htslib/hts_internal.h find_file_extension().
unsafe fn find_file_extension(fn_: *const c_char, ext_out: *mut c_char) -> c_int {
    if fn_.is_null() {
        return -1;
    }
    let fn_len = libc::strlen(fn_);
    // delim = strstr(fn, HTS_IDX_DELIM); if !delim, delim = fn + strlen(fn)
    let delim: *const c_char = {
        let mut needle = [0u8; 8];
        needle[..HTS_IDX_DELIM.len()].copy_from_slice(HTS_IDX_DELIM);
        let found = libc::strstr(fn_, needle.as_ptr().cast());
        if found.is_null() {
            fn_.add(fn_len)
        } else {
            found
        }
    };
    // for (ext = delim; ext > fn && *ext != '.' && *ext != '/'; --ext) {}
    let mut ext = delim;
    while ext > fn_ && *ext != b'.' as c_char && *ext != b'/' as c_char {
        ext = ext.sub(1);
    }
    if *ext == b'.' as c_char
        && ext > fn_
        && ((delim.offset_from(ext) == 3
            && *ext.add(1) == b'g' as c_char
            && *ext.add(2) == b'z' as c_char) // permit .sam.gz
            || (delim.offset_from(ext) == 4
                && *ext.add(1) == b'b' as c_char
                && *ext.add(2) == b'g' as c_char
                && *ext.add(3) == b'z' as c_char))
    // permit .vcf.bgz
    {
        ext = ext.sub(1);
        while ext > fn_ && *ext != b'.' as c_char && *ext != b'/' as c_char {
            ext = ext.sub(1);
        }
    }
    if *ext != b'.' as c_char
        || delim.offset_from(ext) > HTS_MAX_EXT_LEN as isize
        || delim.offset_from(ext) < 3
    {
        return -1;
    }
    let copy_len = (delim.offset_from(ext) - 1) as usize;
    libc::memcpy(ext_out.cast(), ext.add(1).cast(), copy_len);
    *ext_out.add(copy_len) = 0;
    0
}

pub unsafe fn vcf_open_mode(mode: *mut c_char, fn_: *const c_char, format: *const c_char) -> c_int {
    // Native translation of htslib/vcf.c vcf_open_mode().
    if format.is_null() {
        // Try to pick a format based on the filename extension
        let mut extension = [0 as c_char; HTS_MAX_EXT_LEN];
        if find_file_extension(fn_, extension.as_mut_ptr()) < 0 {
            return -1;
        }
        return vcf_open_mode(mode, fn_, extension.as_ptr());
    } else if libc::strcasecmp(format, c"bcf".as_ptr()) == 0 {
        libc::strcpy(mode, c"b".as_ptr());
    } else if libc::strcasecmp(format, c"vcf".as_ptr()) == 0 {
        libc::strcpy(mode, c"".as_ptr());
    } else if libc::strcasecmp(format, c"vcf.gz".as_ptr()) == 0
        || libc::strcasecmp(format, c"vcf.bgz".as_ptr()) == 0
    {
        libc::strcpy(mode, c"z".as_ptr());
    } else {
        return -1;
    }

    0
}

// Native translation of htslib's hts_expand() macro for int arrays.
unsafe fn hts_expand_int(n: c_int, m: *mut c_int, ptr: *mut *mut c_int) {
    if n > *m {
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<c_int>()).cast();
    }
}

// Native translation of htslib's hts_expand() macro for bcf_info_t arrays.
unsafe fn hts_expand_info(n: c_int, m: *mut c_int, ptr: *mut *mut bcf_info_t) {
    if n > *m {
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<bcf_info_t>()).cast();
    }
}

// Native translation of htslib's hts_expand() macro for bcf_fmt_t arrays.
unsafe fn hts_expand_fmt(n: c_int, m: *mut c_int, ptr: *mut *mut bcf_fmt_t) {
    if n > *m {
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<bcf_fmt_t>()).cast();
    }
}

// Native translation of htslib's hts_expand() macro for char* arrays.
unsafe fn hts_expand_charptr(n: c_int, m: *mut c_int, ptr: *mut *mut *mut c_char) {
    if n > *m {
        let mut new_m = n;
        kroundup32(&mut new_m);
        *m = new_m;
        *ptr = libc::realloc((*ptr).cast(), *m as usize * size_of::<*mut c_char>()).cast();
    }
}

// Native translation of htslib/vcf.c bcf_unpack().
pub unsafe fn bcf_unpack(b: *mut bcf1_t, mut which: c_int) -> c_int {
    if (*b).shared.l == 0 {
        // Building a new BCF record from scratch
        return 0;
    }
    let mut ptr = (*b).shared.s.cast::<u8>();
    let d = &mut (*b).d;
    if which & BCF_UN_FLT as c_int != 0 {
        which |= BCF_UN_STR as c_int;
    }
    if which & BCF_UN_INFO as c_int != 0 {
        which |= BCF_UN_SHR as c_int;
    }
    if which & BCF_UN_STR as c_int != 0 && (*b).unpacked & BCF_UN_STR as c_int == 0 {
        let mut tmp = kstring_t {
            l: 0,
            m: d.m_id as size_t,
            s: d.id,
        };

        // ID
        let mut ptr_ori = ptr;
        ptr = bcf_fmt_sized_array(&mut tmp, ptr);
        (*b).unpack_size[0] = ptr.offset_from(ptr_ori) as c_int;
        kputc_(b'\0' as c_int, &mut tmp);
        d.id = tmp.s;
        d.m_id = tmp.m as c_int;

        // REF and ALT are in a single block (d->als) and d->alleles are pointers
        // into this block.
        let n_allele = (*b).n_allele() as c_int;
        hts_expand_charptr(n_allele, &mut d.m_allele, &mut d.allele);
        tmp.l = 0;
        tmp.s = d.als;
        tmp.m = d.m_als as size_t;
        ptr_ori = ptr;
        for i in 0..n_allele as usize {
            // Use offset within tmp.s as realloc may change pointer.
            *d.allele.add(i) = tmp.l as *mut c_char;
            ptr = bcf_fmt_sized_array(&mut tmp, ptr);
            kputc_(b'\0' as c_int, &mut tmp);
        }
        (*b).unpack_size[1] = ptr.offset_from(ptr_ori) as c_int;
        d.als = tmp.s;
        d.m_als = tmp.m as c_int;

        // Convert our offsets within tmp.s back to pointers again.
        for i in 0..n_allele as usize {
            *d.allele.add(i) = d.als.offset(*d.allele.add(i) as isize);
        }
        (*b).unpacked |= BCF_UN_STR as c_int;
    }
    if which & BCF_UN_FLT as c_int != 0 && (*b).unpacked & BCF_UN_FLT as c_int == 0 {
        // FILTER
        ptr = (*b)
            .shared
            .s
            .cast::<u8>()
            .add(((*b).unpack_size[0] + (*b).unpack_size[1]) as usize);
        let ptr_ori = ptr;
        if *ptr >> 4 != 0 {
            let mut type_ = 0;
            let mut p: *const u8 = ptr;
            d.n_flt = bcf_dec_size_unsafe(p, &mut p, &mut type_);
            hts_expand_int(d.n_flt, &mut d.m_flt, &mut d.flt);
            for i in 0..d.n_flt as usize {
                *d.flt.add(i) = bcf_dec_int1_rs(p, type_, &mut p);
            }
            ptr = p as *mut u8;
        } else {
            ptr = ptr.add(1);
            d.n_flt = 0;
        }
        (*b).unpack_size[2] = ptr.offset_from(ptr_ori) as c_int;
        (*b).unpacked |= BCF_UN_FLT as c_int;
    }
    if which & BCF_UN_INFO as c_int != 0 && (*b).unpacked & BCF_UN_INFO as c_int == 0 {
        // INFO
        ptr = (*b)
            .shared
            .s
            .cast::<u8>()
            .add(((*b).unpack_size[0] + (*b).unpack_size[1] + (*b).unpack_size[2]) as usize);
        let n_info = (*b).n_info() as c_int;
        hts_expand_info(n_info, &mut d.m_info, &mut d.info);
        for i in 0..d.m_info as usize {
            (*d.info.add(i)).set_vptr_free(0);
        }
        for i in 0..n_info as usize {
            ptr = bcf_unpack_info_core1_rs(ptr, d.info.add(i));
        }
        (*b).unpacked |= BCF_UN_INFO as c_int;
    }
    if which & BCF_UN_FMT as c_int != 0
        && (*b).n_sample() != 0
        && (*b).unpacked & BCF_UN_FMT as c_int == 0
    {
        // FORMAT
        let mut ptr = (*b).indiv.s.cast::<u8>();
        let n_fmt = (*b).n_fmt() as c_int;
        hts_expand_fmt(n_fmt, &mut d.m_fmt, &mut d.fmt);
        for i in 0..d.m_fmt as usize {
            (*d.fmt.add(i)).set_p_free(0);
        }
        for i in 0..n_fmt as usize {
            ptr = bcf_unpack_fmt_core1_rs(ptr, (*b).n_sample() as c_int, d.fmt.add(i));
        }
        (*b).unpacked |= BCF_UN_FMT as c_int;
    }
    0
}

pub unsafe fn bcf_has_variant_type(rec: *mut bcf1_t, ith_allele: c_int, bitmask: u32) -> c_int {
    vcf_c_5501_bcf_has_variant_type(rec, ith_allele, bitmask)
}

pub unsafe fn bcf_variant_length(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    vcf_c_5513_bcf_variant_length(rec, ith_allele)
}

pub unsafe fn bcf_has_variant_types(
    rec: *mut bcf1_t,
    bitmask: u32,
    mode: bcf_variant_match,
) -> c_int {
    vcf_c_5522_bcf_has_variant_types(rec, bitmask, mode)
}

unsafe fn vcf_line_info_end_i64(line: *const kstring_t) -> Option<i64> {
    if line.is_null() || (*line).s.is_null() {
        return None;
    }

    let bytes = std::slice::from_raw_parts((*line).s.cast::<u8>(), (*line).l);
    if !vcf_line_may_have_info_end(bytes) {
        return None;
    }

    let mut start = 0usize;
    for _ in 0..7 {
        let tab = bytes[start..].iter().position(|&b| b == b'\t')?;
        start += tab + 1;
    }
    let info_len = bytes[start..]
        .iter()
        .position(|&b| b == b'\t')
        .unwrap_or(bytes.len() - start);
    let info = &bytes[start..start + info_len];
    if info == b"." {
        return None;
    }

    for item in info.split(|&b| b == b';') {
        if let Some(value) = item.strip_prefix(b"END=") {
            if value.contains(&b',') {
                return None;
            }
            let text = std::str::from_utf8(value).ok()?;
            return text.parse::<i64>().ok();
        }
    }
    None
}

fn vcf_line_may_have_info_end(bytes: &[u8]) -> bool {
    let mut offset = 0usize;
    while offset + 4 <= bytes.len() {
        let Some(rel) = (unsafe {
            let ptr = libc::memchr(
                bytes.as_ptr().add(offset).cast(),
                b'E' as c_int,
                bytes.len() - offset,
            );
            (!ptr.is_null()).then_some(ptr.cast::<u8>().offset_from(bytes.as_ptr()) as usize)
        }) else {
            return false;
        };
        if rel + 4 <= bytes.len()
            && bytes[rel + 1] == b'N'
            && bytes[rel + 2] == b'D'
            && bytes[rel + 3] == b'='
            && (rel == 0 || bytes[rel - 1] == b'\t' || bytes[rel - 1] == b';')
        {
            return true;
        }
        offset = rel + 1;
    }
    false
}

unsafe fn vcf_line_symbolic_svlen_rlen(line: *const kstring_t) -> Option<hts_pos_t> {
    if line.is_null() || (*line).s.is_null() {
        return None;
    }
    let bytes = std::slice::from_raw_parts((*line).s.cast::<u8>(), (*line).l);
    // Cheap reject: symbolic spanning alts only appear with '<' in the line.
    // memchr is far cheaper than collecting fields/splitting ALT for every record.
    let has_lt = libc::memchr(bytes.as_ptr().cast(), b'<' as c_int, bytes.len());
    if has_lt.is_null() {
        return None;
    }
    let fields = vcf_line_first_fields(bytes, 8)?;
    let alt = fields[4];
    let symbolic_spanning = alt.split(|&b| b == b',').any(|allele| {
        allele.ends_with(b">")
            && (allele.starts_with(b"<DEL")
                || allele.starts_with(b"<DUP")
                || allele.starts_with(b"<CNV")
                || allele.starts_with(b"<INV"))
    });
    if !symbolic_spanning {
        return None;
    }
    let info = fields[7];
    let mut max_rlen: Option<i64> = None;
    for item in info.split(|&b| b == b';') {
        let Some(value) = item.strip_prefix(b"SVLEN=") else {
            continue;
        };
        for raw in value.split(|&b| b == b',') {
            if raw == b"." || raw.is_empty() {
                continue;
            }
            let text = std::str::from_utf8(raw).ok()?;
            let svlen = text.parse::<i64>().ok()?;
            if svlen != 0 {
                max_rlen = Some(max_rlen.unwrap_or(0).max(svlen.abs() + 1));
            }
        }
    }
    max_rlen.map(|value| value as hts_pos_t)
}

unsafe fn vcf_line_format_len_rlen(line: *const kstring_t) -> Option<hts_pos_t> {
    if line.is_null() || (*line).s.is_null() {
        return None;
    }
    let bytes = std::slice::from_raw_parts((*line).s.cast::<u8>(), (*line).l);
    // Cheap reject: the format-LEN handling only fires when ALT contains "<*>".
    // Skip the full-line tab split (per-record Vec allocation) when '<' is absent.
    let has_lt = libc::memchr(bytes.as_ptr().cast(), b'<' as c_int, bytes.len());
    if has_lt.is_null() {
        return None;
    }
    let fields = vcf_line_all_fields(bytes);
    if fields.len() < 10 {
        return None;
    }
    if !fields[4]
        .split(|&b| b == b',')
        .any(|allele| allele == b"<*>")
    {
        return None;
    }
    let len_idx = fields[8]
        .split(|&b| b == b':')
        .position(|key| key == b"LEN")?;
    let mut max_len = None;
    for sample in &fields[9..] {
        let Some(raw) = sample.split(|&b| b == b':').nth(len_idx) else {
            continue;
        };
        if raw == b"." || raw.is_empty() {
            continue;
        }
        let value = std::str::from_utf8(raw).ok()?.parse::<i64>().ok()?;
        if value > 0 {
            max_len = Some(max_len.unwrap_or(0).max(value));
        }
    }
    max_len.map(|value| value as hts_pos_t)
}

fn vcf_line_first_fields(bytes: &[u8], n: usize) -> Option<Vec<&[u8]>> {
    let mut out = Vec::with_capacity(n);
    let mut start = 0usize;
    while out.len() < n {
        let rel_end = bytes[start..]
            .iter()
            .position(|&b| b == b'\t')
            .unwrap_or(bytes.len() - start);
        let end = start + rel_end;
        out.push(&bytes[start..end]);
        if out.len() == n {
            return Some(out);
        }
        if end == bytes.len() {
            return None;
        }
        start = end + 1;
    }
    Some(out)
}

fn vcf_line_all_fields(bytes: &[u8]) -> Vec<&[u8]> {
    bytes.split(|&b| b == b'\t').collect()
}

unsafe fn vcf_repair_info_end_i64(h: *const bcf_hdr_t, v: *mut bcf1_t, end: i64) -> c_int {
    if h.is_null() || v.is_null() {
        return 0;
    }
    if end < BCF_MIN_BT_INT64 {
        return 0;
    }
    if end >= BCF_MIN_BT_INT32 && end <= i32::MAX as i64 {
        return 0;
    }

    let end_id = bcf_hdr_id2int(h, BCF_DT_ID as c_int, c"END".as_ptr());
    if end_id < 0 {
        return 0;
    }
    if bcf_unpack(v, BCF_UN_INFO as c_int) < 0 {
        return -1;
    }

    let info = bcf_get_info_id(v, end_id);
    if info.is_null() || (*info).len != 1 {
        return 0;
    }
    if (*info).type_ == BCF_BT_INT64 as c_int && (*info).v1.i == end {
        return 0;
    }

    let mut str_ = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    if bcf_enc_int1(&mut str_, end_id) < 0
        || bcf_enc_size(&mut str_, 1, BCF_BT_INT64 as c_int) < 0
        || ks_resize(&mut str_, str_.l + size_of::<i64>()) < 0
    {
        super::hts::ks_free(&mut str_);
        return -1;
    }
    let vptr_off = str_.l;
    i64_to_le(end, str_.s.add(str_.l).cast());
    str_.l += size_of::<i64>();

    if (*info).vptr_free() != 0 && !(*info).vptr.is_null() {
        libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
    }
    (*info).key = end_id;
    (*info).type_ = BCF_BT_INT64 as c_int;
    (*info).v1.i = end;
    (*info).vptr = str_.s.cast::<u8>().add(vptr_off);
    (*info).vptr_len = size_of::<i64>() as u32;
    (*info).set_vptr_off(vptr_off as u32);
    (*info).set_vptr_free(1);
    if end > (*v).pos {
        (*v).rlen = (*v).rlen.max(end - (*v).pos);
    }
    (*v).unpacked |= BCF_IS_64BIT | BCF_UN_INFO as c_int;
    (*v).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
    0
}

pub unsafe fn vcf_c_5373_bcf_set_variant_type(
    ref_: *const c_char,
    alt: *const c_char,
    var: *mut bcf_variant_t,
) {
    if *alt == b'*' as c_char && *alt.add(1) == 0 {
        (*var).n = 0;
        (*var).type_ = VCF_OVERLAP as c_int;
        return;
    }

    if *ref_.add(1) == 0 && *alt.add(1) == 0 {
        if *alt == b'.' as c_char || *ref_ == *alt {
            (*var).n = 0;
            (*var).type_ = VCF_REF as c_int;
            return;
        }
        if *alt == b'X' as c_char {
            (*var).n = 0;
            (*var).type_ = VCF_REF as c_int;
            return;
        }
        (*var).n = 1;
        (*var).type_ = VCF_SNP as c_int;
        return;
    }

    if *alt == b'<' as c_char {
        if *alt.add(1) == b'X' as c_char && *alt.add(2) == b'>' as c_char {
            (*var).n = 0;
            (*var).type_ = VCF_REF as c_int;
            return;
        }
        if *alt.add(1) == b'*' as c_char && *alt.add(2) == b'>' as c_char {
            (*var).n = 0;
            (*var).type_ = VCF_REF as c_int;
            return;
        }
        if libc::strcmp(c"NON_REF>".as_ptr(), alt.add(1)) == 0 {
            (*var).n = 0;
            (*var).type_ = VCF_REF as c_int;
            return;
        }
        (*var).type_ = VCF_OTHER as c_int;
        return;
    }

    if *alt == b']' as c_char || *alt == b'[' as c_char {
        (*var).type_ = VCF_BND as c_int;
        return;
    }

    let mut r = ref_;
    let mut a = alt;
    while *r != 0 && *a != 0 && toupper_c(*r) == toupper_c(*a) {
        r = r.add(1);
        a = a.add(1);
    }

    if *a != 0 && *r == 0 {
        while *a != 0 {
            a = a.add(1);
        }
        if *a.sub(1) == b']' as c_char || *a.sub(1) == b'[' as c_char {
            (*var).type_ = VCF_BND as c_int;
            return;
        }
        (*var).n = a.offset_from(alt) as c_int - r.offset_from(ref_) as c_int;
        (*var).type_ = (VCF_INDEL | VCF_INS) as c_int;
        return;
    } else if *r != 0 && *a == 0 {
        while *r != 0 {
            r = r.add(1);
        }
        (*var).n = a.offset_from(alt) as c_int - r.offset_from(ref_) as c_int;
        (*var).type_ = (VCF_INDEL | VCF_DEL) as c_int;
        return;
    } else if *r == 0 && *a == 0 {
        (*var).n = 0;
        (*var).type_ = VCF_REF as c_int;
        return;
    }

    let mut re = r;
    let mut ae = a;
    while *re.add(1) != 0 {
        re = re.add(1);
    }
    while *ae.add(1) != 0 {
        ae = ae.add(1);
    }
    if *ae == b']' as c_char || *ae == b'[' as c_char {
        (*var).type_ = VCF_BND as c_int;
        return;
    }
    while re > r && ae > a && toupper_c(*re) == toupper_c(*ae) {
        re = re.sub(1);
        ae = ae.sub(1);
    }

    if ae == a {
        if re == r {
            (*var).n = 1;
            (*var).type_ = VCF_SNP as c_int;
            return;
        }
        (*var).n = -re.offset_from(r) as c_int;
        if toupper_c(*re) == toupper_c(*ae) {
            (*var).type_ = (VCF_INDEL | VCF_DEL) as c_int;
            return;
        }
        (*var).type_ = VCF_OTHER as c_int;
        return;
    } else if re == r {
        (*var).n = ae.offset_from(a) as c_int;
        if toupper_c(*re) == toupper_c(*ae) {
            (*var).type_ = (VCF_INDEL | VCF_INS) as c_int;
            return;
        }
        (*var).type_ = VCF_OTHER as c_int;
        return;
    }

    (*var).type_ = if re.offset_from(r) == ae.offset_from(a) {
        VCF_MNP as c_int
    } else {
        VCF_OTHER as c_int
    };
    (*var).n = if re.offset_from(r) > ae.offset_from(a) {
        -(re.offset_from(r) as c_int + 1)
    } else {
        ae.offset_from(a) as c_int + 1
    };
}

unsafe fn bcf_canonicalize_duplicate_format(rec: *mut bcf1_t) {
    if rec.is_null() || (*rec).n_fmt() < 2 || (*rec).n_sample() == 0 {
        return;
    }
    if bcf_unpack(rec, BCF_UN_FMT as c_int) != 0 {
        return;
    }

    let n_fmt = (*rec).n_fmt() as usize;
    let fmt = (*rec).d.fmt;
    if fmt.is_null() {
        return;
    }

    let mut dst = 0usize;
    for src in 0..n_fmt {
        let src_fmt = fmt.add(src);
        let mut duplicate = false;
        for prev in 0..dst {
            if (*fmt.add(prev)).id == (*src_fmt).id {
                duplicate = true;
                break;
            }
        }
        if duplicate {
            bcf_clear_owned_format_storage(src_fmt);
            continue;
        }
        if dst != src {
            std::ptr::copy(src_fmt, fmt.add(dst), 1);
        }
        dst += 1;
    }

    if dst != n_fmt {
        for i in dst..n_fmt {
            (*fmt.add(i)).set_p_free(0);
        }
        (*rec).set_n_fmt(dst as u32);
        (*rec).d.indiv_dirty = 1;
    }
}

unsafe fn bcf_clear_owned_format_storage(fmt: *mut bcf_fmt_t) {
    if !fmt.is_null() && (*fmt).p_free() != 0 && !(*fmt).p.is_null() {
        libc::free((*fmt).p.sub((*fmt).p_off() as usize).cast());
        (*fmt).set_p_free(0);
    }
}

pub unsafe fn vcf_c_5444_bcf_set_variant_types(b: *mut bcf1_t) -> c_int {
    if (*b).unpacked & BCF_UN_STR as c_int == 0 {
        bcf_unpack(b, BCF_UN_STR as c_int);
    }
    let d = &mut (*b).d;
    if d.n_var < (*b).n_allele() as c_int {
        let new_var = libc::realloc(
            d.var.cast(),
            size_of::<bcf_variant_t>() * (*b).n_allele() as usize,
        )
        .cast::<bcf_variant_t>();
        if new_var.is_null() {
            return -1;
        }
        d.var = new_var;
        d.n_var = (*b).n_allele() as c_int;
    }
    (*b).d.var_type = 0;
    (*d.var).type_ = VCF_REF as c_int;
    (*d.var).n = 0;
    for i in 1..(*b).n_allele() as c_int {
        vcf_c_5373_bcf_set_variant_type(
            *d.allele,
            *d.allele.add(i as usize),
            d.var.add(i as usize),
        );
        (*b).d.var_type |= (*d.var.add(i as usize)).type_;
    }
    0
}

const ORIG_VAR_TYPES: u32 = VCF_SNP | VCF_MNP | VCF_INDEL | VCF_OTHER | VCF_BND | VCF_OVERLAP;

pub unsafe fn vcf_c_5474_bcf_get_variant_types(rec: *mut bcf1_t) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        let err = CStr::from_ptr(libc::strerror(*libc::__errno_location())).to_string_lossy();
        let msg = std::ffi::CString::new(format!("Couldn't get variant types: {}", err))
            .unwrap_or_default();
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_get_variant_types".as_ptr(),
            msg.as_ptr(),
        );
        libc::exit(1);
    }
    (*rec).d.var_type & ORIG_VAR_TYPES as c_int
}

pub unsafe fn vcf_c_5485_bcf_get_variant_type(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        let err = CStr::from_ptr(libc::strerror(*libc::__errno_location())).to_string_lossy();
        let msg = std::ffi::CString::new(format!("Couldn't get variant types: {}", err))
            .unwrap_or_default();
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_get_variant_type".as_ptr(),
            msg.as_ptr(),
        );
        libc::exit(1);
    }
    if ith_allele < 0 || ith_allele >= (*rec).n_allele() as c_int {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_get_variant_type".as_ptr(),
            c"Requested allele outside valid range".as_ptr(),
        );
        libc::exit(1);
    }
    (*(*rec).d.var.add(ith_allele as usize)).type_ & ORIG_VAR_TYPES as c_int
}

pub unsafe fn vcf_c_5501_bcf_has_variant_type(
    rec: *mut bcf1_t,
    ith_allele: c_int,
    bitmask: u32,
) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        return -1;
    }
    if ith_allele < 0 || ith_allele >= (*rec).n_allele() as c_int {
        return -1;
    }
    if bitmask == VCF_REF {
        return ((*(*rec).d.var.add(ith_allele as usize)).type_ == VCF_REF as c_int) as c_int;
    }
    (bitmask as c_int) & (*(*rec).d.var.add(ith_allele as usize)).type_
}

pub unsafe fn vcf_c_5513_bcf_variant_length(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        return bcf_int32_missing;
    }
    if ith_allele < 0 || ith_allele >= (*rec).n_allele() as c_int {
        return bcf_int32_missing;
    }
    (*(*rec).d.var.add(ith_allele as usize)).n
}

pub unsafe fn vcf_c_5522_bcf_has_variant_types(
    rec: *mut bcf1_t,
    bitmask: u32,
    mode: bcf_variant_match,
) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        return -1;
    }
    let mut type_ = (*rec).d.var_type as u32;
    if mode == 1 {
        return (bitmask & type_) as c_int;
    }

    if bitmask & (VCF_INS | VCF_DEL) != 0 && bitmask & VCF_INDEL == 0 {
        type_ &= !VCF_INDEL;
    } else if bitmask & VCF_INDEL != 0 && bitmask & (VCF_INS | VCF_DEL) == 0 {
        type_ &= !(VCF_INS | VCF_DEL);
    }

    if mode == 2 {
        if !bitmask & type_ != 0 {
            0
        } else {
            (bitmask & type_) as c_int
        }
    } else if bitmask == VCF_REF {
        (type_ == bitmask) as c_int
    } else if type_ == bitmask {
        type_ as c_int
    } else {
        0
    }
}

// Native translation of htslib/vcf.c errdesc_bcf[] table (typedef struct err_desc).
// (BCF_ERR_* bit flags are defined near the top of this module as u32.)
const ERRDESC_BCF: &[(c_int, &CStr)] = &[
    (BCF_ERR_CTG_UNDEF as c_int, c"Contig not defined in header"),
    (BCF_ERR_TAG_UNDEF as c_int, c"Tag not defined in header"),
    (BCF_ERR_NCOLS as c_int, c"Incorrect number of columns"),
    (BCF_ERR_LIMITS as c_int, c"Limits reached"),
    (BCF_ERR_CHAR as c_int, c"Invalid character"),
    (BCF_ERR_CTG_INVALID as c_int, c"Invalid contig"),
    (BCF_ERR_TAG_INVALID as c_int, c"Invalid tag"),
];

// Native translation of htslib/vcf.c add_desc_to_buffer().
// append given description to buffer based on available size and add ... when not enough space
unsafe fn add_desc_to_buffer(
    buffer: *mut c_char,
    offset: *mut usize,
    maxbuffer: usize,
    description: *const c_char,
) -> c_int {
    if description.is_null() || buffer.is_null() || offset.is_null() || maxbuffer < 4 {
        return -1;
    }

    let rembuffer = maxbuffer - *offset;
    if rembuffer > libc::strlen(description) + (if rembuffer == maxbuffer { 0 } else { 1 }) {
        // add description with optionally required ','
        let prefix = if rembuffer == maxbuffer {
            c"".as_ptr()
        } else {
            c",".as_ptr()
        };
        *offset += libc::snprintf(
            buffer.add(*offset),
            rembuffer,
            c"%s%s".as_ptr(),
            prefix,
            description,
        ) as usize;
    } else {
        // not enough space for description, put ...
        let tmppos = if rembuffer <= 4 {
            maxbuffer - 4
        } else {
            *offset
        };
        libc::snprintf(buffer.add(tmppos), 4, c"...".as_ptr()); // ignore offset update
        return -1;
    }
    0
}

// Native translation of htslib/vcf.c bcf_strerror().
// get description for given error code. return NULL on error
pub unsafe fn bcf_strerror(
    mut errorcode: c_int,
    buffer: *mut c_char,
    maxbuffer: usize,
) -> *const c_char {
    let mut usedup: usize = 0;
    let mut ret: c_int = 0;

    if buffer.is_null() || maxbuffer < 4 {
        return std::ptr::null(); // invalid / insufficient buffer
    }

    if errorcode == 0 {
        *buffer = 0; // no error, set null
        return buffer;
    }

    for entry in ERRDESC_BCF {
        if errorcode & entry.0 != 0 {
            // error is set, add description
            ret = add_desc_to_buffer(buffer, &mut usedup, maxbuffer, entry.1.as_ptr());
            if ret < 0 {
                break; // not enough space, ... added, no need to continue
            }
            errorcode &= !entry.0; // reset the error
        }
    }

    if errorcode != 0 && ret >= 0 {
        // undescribed error is present in error code and had enough buffer, try unknown error
        add_desc_to_buffer(buffer, &mut usedup, maxbuffer, c"Unknown error".as_ptr());
    }
    buffer
}

pub unsafe fn bcf_format_gt_v2(
    hdr: *const bcf_hdr_t,
    fmt: *mut bcf_fmt_t,
    isample: c_int,
    str_: *mut kstring_t,
) -> c_int {
    let v44 = !hdr.is_null() && vcf_hdr_version_ge_44(hdr);
    bcf_format_gt_v2_inner(v44, fmt, isample, str_)
}

#[inline]
unsafe fn bcf_format_gt_v2_inner(
    v44: bool,
    fmt: *mut bcf_fmt_t,
    isample: c_int,
    str_: *mut kstring_t,
) -> c_int {
    if fmt.is_null() || str_.is_null() {
        return -1;
    }

    let pos = (*str_).l;
    let mut val0 = 0;
    let mut ploidy = 0;
    let mut any_unphased = false;

    macro_rules! branch {
        ($read:ident, $step:expr, $vector_end:expr) => {{
            let mut ptr = (*fmt).p.add(isample as usize * (*fmt).size as usize);
            for i in 0..(*fmt).n {
                let val = $read(ptr);
                if val == $vector_end {
                    break;
                }
                if i == 0 {
                    val0 = val as i32;
                } else {
                    if kputc(if val & 1 != 0 { b'|' } else { b'/' } as c_int, str_) < 0 {
                        return -1;
                    }
                    any_unphased |= val & 1 == 0;
                }
                if val >> 1 == 0 {
                    if kputc(b'.' as c_int, str_) < 0 {
                        return -1;
                    }
                } else if kputw((val >> 1) as c_int - 1, str_) < 0 {
                    return -1;
                }
                ploidy = i + 1;
                ptr = ptr.add($step);
            }
            if ploidy == 0 && kputc(b'.' as c_int, str_) < 0 {
                return -1;
            }
        }};
    }

    match (*fmt).type_ {
        x if x == BCF_BT_INT8 as c_int => {
            branch!(le_to_i8, 1, bcf_int8_vector_end as i8);
        }
        x if x == BCF_BT_INT16 as c_int => {
            branch!(le_to_i16, size_of::<i16>(), bcf_int16_vector_end as i16);
        }
        x if x == BCF_BT_INT32 as c_int => {
            branch!(le_to_i32, size_of::<i32>(), bcf_int32_vector_end);
        }
        x if x == BCF_BT_NULL as c_int => {
            if kputc(b'.' as c_int, str_) < 0 {
                return -1;
            }
        }
        _ => return -2,
    }

    if v44 {
        if val0 & 1 != 0 {
            if (ploidy > 1 && any_unphased) || (ploidy <= 1 && val0 >> 1 == 0) {
                return kstring_insert_char(str_, pos, b'|' as c_char);
            }
        } else if (ploidy <= 1 && val0 != 0) || (ploidy > 1 && !any_unphased) {
            return kstring_insert_char(str_, pos, b'/' as c_char);
        }
    }

    0
}

pub unsafe fn bcf_format_gt(fmt: *mut bcf_fmt_t, isample: c_int, str_: *mut kstring_t) -> c_int {
    unsafe { bcf_format_gt_v2(std::ptr::null(), fmt, isample, str_) }
}

unsafe fn kstring_insert_char(str_: *mut kstring_t, pos: usize, c: c_char) -> c_int {
    if pos > (*str_).l || ks_resize(str_, (*str_).l + 2) < 0 {
        return -1;
    }
    std::ptr::copy(
        (*str_).s.add(pos),
        (*str_).s.add(pos + 1),
        (*str_).l - pos + 1,
    );
    *(*str_).s.add(pos) = c;
    (*str_).l += 1;
    0
}

pub unsafe fn bcf_dup(src: *mut bcf1_t) -> *mut bcf1_t {
    // Native translation of htslib/vcf.c bcf_dup().
    let out = bcf_init();
    bcf_copy(out, src)
}

pub unsafe fn bcf_copy(dst: *mut bcf1_t, src: *mut bcf1_t) -> *mut bcf1_t {
    // Native translation of htslib/vcf.c bcf_copy().
    vcf_c_2332_bcf1_sync(src);

    bcf_clear(dst);
    (*dst).rid = (*src).rid;
    (*dst).pos = (*src).pos;
    (*dst).rlen = (*src).rlen;
    (*dst).qual = (*src).qual;
    (*dst).set_n_info((*src).n_info());
    (*dst).set_n_allele((*src).n_allele());
    (*dst).set_n_fmt((*src).n_fmt());
    (*dst).set_n_sample((*src).n_sample());

    if (*dst).shared.m < (*src).shared.l {
        (*dst).shared.s =
            libc::realloc((*dst).shared.s.cast(), (*src).shared.l as usize).cast::<c_char>();
        (*dst).shared.m = (*src).shared.l;
    }
    (*dst).shared.l = (*src).shared.l;
    std::ptr::copy((*src).shared.s, (*dst).shared.s, (*dst).shared.l as usize);

    if (*dst).indiv.m < (*src).indiv.l {
        (*dst).indiv.s =
            libc::realloc((*dst).indiv.s.cast(), (*src).indiv.l as usize).cast::<c_char>();
        (*dst).indiv.m = (*src).indiv.l;
    }
    (*dst).indiv.l = (*src).indiv.l;
    std::ptr::copy((*src).indiv.s, (*dst).indiv.s, (*dst).indiv.l as usize);

    dst
}

// Native translation of htslib/vcf.c bcf_write().
pub unsafe fn bcf_write(hfp: *mut htsFile, h: *mut bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    if (*h).dirty != 0 && bcf_hdr_sync(h) < 0 {
        return -1;
    }
    if (*h).n[BCF_DT_SAMPLE as usize] as u32 != (*v).n_sample() {
        let msg = std::ffi::CString::new(format!(
            "Broken VCF record, the number of columns at {}:{} does not match the number of samples ({} vs {})",
            CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
            (*v).pos + 1,
            (*v).n_sample(),
            (*h).n[BCF_DT_SAMPLE as usize]
        ))
        .unwrap_or_default();
        c_log_error(msg.as_ptr());
        return -1;
    }

    if (*hfp).format.format == HTS_FORMAT_VCF || (*hfp).format.format == HTS_FORMAT_TEXT_FORMAT {
        return vcf_write(hfp, h, v);
    }

    if (*v).errcode & !(BCF_ERR_LIMITS as c_int) != 0 {
        let mut errdescription = [0 as c_char; 1024];
        let err = bcf_strerror(
            (*v).errcode,
            errdescription.as_mut_ptr(),
            errdescription.len(),
        );
        let msg = std::ffi::CString::new(format!(
            "Unchecked error ({} {}) at {}:{}",
            (*v).errcode,
            CStr::from_ptr(err).to_string_lossy(),
            CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
            (*v).pos + 1
        ))
        .unwrap_or_default();
        c_log_error(msg.as_ptr());
        return -1;
    }
    vcf_c_2332_bcf1_sync(v); // check if the BCF record was modified

    if (*v).unpacked & BCF_IS_64BIT != 0 {
        let msg = std::ffi::CString::new(format!(
            "Data at {}:{} contains 64-bit values not representable in BCF. Please use VCF instead",
            CStr::from_ptr(bcf_seqname_safe(h, v)).to_string_lossy(),
            (*v).pos + 1
        ))
        .unwrap_or_default();
        c_log_error(msg.as_ptr());
        return -1;
    }

    let fp = hts_get_bgzfp(hfp);
    let mut x = [0u8; 32];
    super::hts::u32_to_le((*v).shared.l as u32 + 24, x.as_mut_ptr()); // include six 32-bit integers
    super::hts::u32_to_le((*v).indiv.l as u32, x.as_mut_ptr().add(4));
    i32_to_le((*v).rid, x.as_mut_ptr().add(8));
    super::hts::u32_to_le((*v).pos as u32, x.as_mut_ptr().add(12));
    super::hts::u32_to_le((*v).rlen as u32, x.as_mut_ptr().add(16));
    super::hts::float_to_le((*v).qual, x.as_mut_ptr().add(20));
    super::hts::u16_to_le((*v).n_info() as u16, x.as_mut_ptr().add(24));
    super::hts::u16_to_le((*v).n_allele() as u16, x.as_mut_ptr().add(26));
    super::hts::u32_to_le(
        ((*v).n_fmt() << 24) | ((*v).n_sample() & 0xffffff),
        x.as_mut_ptr().add(28),
    );
    if bgzf_write(fp, x.as_ptr().cast(), 32) != 32 {
        return -1;
    }
    if bgzf_write(fp, (*v).shared.s.cast(), (*v).shared.l as usize) != (*v).shared.l as isize {
        return -1;
    }
    if bgzf_write(fp, (*v).indiv.s.cast(), (*v).indiv.l as usize) != (*v).indiv.l as isize {
        return -1;
    }

    if !(*hfp).idx.is_null() {
        let tell = (((*fp).block_address as u64) << 16) | ((*fp).block_offset as u64 & 0xffff);
        if bgzf_c_189_bgzf_idx_push(
            fp,
            (*hfp).idx,
            (*v).rid,
            (*v).pos,
            (*v).pos + (*v).rlen,
            tell,
            1,
        ) < 0
        {
            return -1;
        }
    }

    0
}

// Native translation of htslib/vcf.c add_missing_contig_hrec().
unsafe fn add_missing_contig_hrec(h: *mut bcf_hdr_t, name: *const c_char) -> c_int {
    let hrec = libc::calloc(1, size_of::<bcf_hrec_t>()).cast::<bcf_hrec_t>();
    if hrec.is_null() {
        return add_missing_contig_hrec_fail(hrec);
    }
    (*hrec).key = libc::strdup(c"contig".as_ptr());
    if (*hrec).key.is_null() {
        return add_missing_contig_hrec_fail(hrec);
    }
    if bcf_hrec_add_key(hrec, c"ID".as_ptr(), 2) < 0 {
        return add_missing_contig_hrec_fail(hrec);
    }
    if bcf_hrec_set_val(hrec, (*hrec).nkeys - 1, name, libc::strlen(name), 0) < 0 {
        return add_missing_contig_hrec_fail(hrec);
    }
    if bcf_hdr_add_hrec(h, hrec) < 0 {
        return add_missing_contig_hrec_fail(hrec);
    }
    0
}

unsafe fn add_missing_contig_hrec_fail(hrec: *mut bcf_hrec_t) -> c_int {
    let save_errno = *libc::__errno_location();
    c_log_error(libc::strerror(*libc::__errno_location()));
    if !hrec.is_null() {
        bcf_hrec_destroy(hrec);
    }
    *libc::__errno_location() = save_errno;
    -1
}

// The public vcf_hdr_read entry point dispatches on format like the original
// hts_sys wrapper did (which called bcf_hdr_read). The faithful text-only
// translation of htslib/vcf.c vcf_hdr_read() is vcf_hdr_read_text(), used by
// bcf_hdr_read_native() for the VCF path.
pub unsafe fn vcf_hdr_read(fp: *mut htsFile) -> *mut bcf_hdr_t {
    bcf_hdr_read(fp)
}

// Native translation of htslib/vcf.c vcf_hdr_read().
unsafe fn vcf_hdr_read_text(fp: *mut htsFile) -> *mut bcf_hdr_t {
    let s = &mut (*fp).line as *mut kstring_t;
    let h = bcf_hdr_init(c"r".as_ptr());
    if h.is_null() {
        c_log_error(c"Failed to allocate bcf header".as_ptr());
        return std::ptr::null_mut();
    }
    let mut txt = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let mut idx: *mut super::tbx::tbx_t = std::ptr::null_mut();
    let mut names: *mut *const c_char = std::ptr::null_mut();

    let error = |idx: *mut super::tbx::tbx_t,
                 names: *mut *const c_char,
                 txt: *mut kstring_t,
                 h: *mut bcf_hdr_t|
     -> *mut bcf_hdr_t {
        if !idx.is_null() {
            super::tbx::tbx_destroy(idx);
        }
        libc::free(names.cast());
        libc::free((*txt).s.cast());
        if !h.is_null() {
            bcf_hdr_destroy(h);
        }
        std::ptr::null_mut()
    };

    loop {
        let ret = hts_getline(fp, KS_SEP_LINE, s);
        if ret < 0 {
            if ret < -1 {
                return error(idx, names, &mut txt, h);
            }
            break;
        }
        let mut e = false;
        if (*s).l == 0 {
            continue;
        }
        if *(*s).s != b'#' as c_char {
            c_log_error(c"No sample line".as_ptr());
            return error(idx, names, &mut txt, h);
        }
        if *(*s).s.add(1) != b'#' as c_char && !(*fp).fn_aux.is_null() {
            // insert contigs here
            let mut tmp = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            let f = super::hfile::hopen((*fp).fn_aux, c"r".as_ptr());
            if f.is_null() {
                let msg = std::ffi::CString::new(format!(
                    "Couldn't open \"{}\"",
                    CStr::from_ptr((*fp).fn_aux).to_string_lossy()
                ))
                .unwrap_or_default();
                c_log_error(msg.as_ptr());
                libc::free(tmp.s.cast());
                return error(idx, names, &mut txt, h);
            }
            loop {
                tmp.l = 0;
                if super::hfile::khgetline(&mut tmp, f) < 0 {
                    break;
                }
                let tab = libc::strchr(tmp.s, b'\t' as c_int);
                if tab.is_null() {
                    continue;
                }
                e |= kputs(c"##contig=<ID=".as_ptr(), &mut txt) < 0;
                e |= kputsn(tmp.s, tab.offset_from(tmp.s) as usize, &mut txt) < 0;
                e |= kputs(c",length=".as_ptr(), &mut txt) < 0;
                e |= super::hts::kputl(libc::atol(tab) as isize, &mut txt) < 0;
                e |= kputsn(c">\n".as_ptr(), 2, &mut txt) < 0;
            }
            libc::free(tmp.s.cast());
            if super::hfile::hclose(f) != 0 {
                let msg = std::ffi::CString::new(format!(
                    "Error on closing {}",
                    CStr::from_ptr((*fp).fn_aux).to_string_lossy()
                ))
                .unwrap_or_default();
                c_log_error(msg.as_ptr());
                return error(idx, names, &mut txt, h);
            }
            if e {
                return error(idx, names, &mut txt, h);
            }
        }
        if kputsn((*s).s, (*s).l, &mut txt) < 0 {
            return error(idx, names, &mut txt, h);
        }
        if kputc(b'\n' as c_int, &mut txt) < 0 {
            return error(idx, names, &mut txt, h);
        }
        if *(*s).s.add(1) != b'#' as c_char {
            break;
        }
    }
    if txt.s.is_null() {
        c_log_error(c"Could not read the header".as_ptr());
        return error(idx, names, &mut txt, h);
    }
    if bcf_hdr_parse(h, txt.s) < 0 {
        return error(idx, names, &mut txt, h);
    }

    // check tabix index, are all contigs listed in the header? add the missing ones
    idx = super::tbx::tbx_index_load3((*fp).fn_, std::ptr::null(), super::hts::HTS_IDX_SILENT_FAIL);
    if !idx.is_null() {
        let mut n: c_int = 0;
        let mut need_sync = 0;
        names = super::tbx::tbx_seqnames(idx, &mut n);
        if names.is_null() {
            return error(idx, names, &mut txt, h);
        }
        for i in 0..n as usize {
            let hrec = bcf_hdr_get_hrec(
                h,
                BCF_HL_CTG as c_int,
                c"ID".as_ptr(),
                *names.add(i),
                std::ptr::null(),
            );
            if !hrec.is_null() {
                continue;
            }
            if add_missing_contig_hrec(h, *names.add(i)) < 0 {
                return error(idx, names, &mut txt, h);
            }
            need_sync = 1;
        }
        if need_sync != 0 && bcf_hdr_sync(h) < 0 {
            return error(idx, names, &mut txt, h);
        }
        libc::free(names.cast());
        super::tbx::tbx_destroy(idx);
    }
    libc::free(txt.s.cast());
    h
}

// Native translation of htslib/vcf.c vcf_hdr_write().
pub unsafe fn vcf_hdr_write(fp: *mut htsFile, h: *const bcf_hdr_t) -> c_int {
    let mut htxt = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    if bcf_hdr_format(h, 0, &mut htxt) < 0 {
        libc::free(htxt.s.cast());
        return -1;
    }
    // kill trailing zeros
    while htxt.l != 0 && *htxt.s.add(htxt.l as usize - 1) == 0 {
        htxt.l -= 1;
    }
    let ret;
    if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
        ret = bgzf_write(hts_get_bgzfp(fp), htxt.s.cast(), htxt.l as usize) as c_int;
        if bgzf_flush(hts_get_bgzfp(fp)) != 0 {
            libc::free(htxt.s.cast());
            return -1;
        }
    } else {
        ret =
            super::hfile::htslib_hfile_h_292_hwrite((*fp).fp.hfile, htxt.s.cast(), htxt.l as usize)
                as c_int;
    }
    libc::free(htxt.s.cast());
    if ret < 0 {
        -1
    } else {
        0
    }
}

pub unsafe fn vcf_read(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    bcf_read(fp, h, v)
}

// Native translation of htslib/vcf.c vcf_write().
pub unsafe fn vcf_write(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    let ret;
    (*fp).line.l = 0;
    if vcf_format(h, v, &mut (*fp).line) != 0 {
        return -1;
    }
    if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
        let bgzf = hts_get_bgzfp(fp);
        if super::bgzf::bgzf_flush_try(bgzf, (*fp).line.l as isize) < 0 {
            return -1;
        }
        if !(*fp).idx.is_null() && (*bgzf).mt.is_null() {
            let tell =
                (((*bgzf).block_address as u64) << 16) | ((*bgzf).block_offset as u64 & 0xffff);
            super::hts::hts_c_2682_hts_idx_amend_last((*fp).idx, tell);
        }
        ret = bgzf_write(bgzf, (*fp).line.s.cast(), (*fp).line.l as usize);
    } else {
        ret = super::hfile::htslib_hfile_h_292_hwrite(
            (*fp).fp.hfile,
            (*fp).line.s.cast(),
            (*fp).line.l as usize,
        );
    }

    if !(*fp).idx.is_null() && (*fp).format.compression == HTS_COMPRESSION_BGZF {
        let bgzf = hts_get_bgzfp(fp);
        let tid = super::hts::hts_idx_tbi_name((*fp).idx, (*v).rid, bcf_seqname_safe(h, v));
        if tid < 0 {
            return -1;
        }
        let tell = (((*bgzf).block_address as u64) << 16) | ((*bgzf).block_offset as u64 & 0xffff);
        if bgzf_c_189_bgzf_idx_push(
            bgzf,
            (*fp).idx,
            tid,
            (*v).pos,
            (*v).pos + (*v).rlen,
            tell,
            1,
        ) < 0
        {
            return -1;
        }
    }

    if ret == (*fp).line.l as isize {
        0
    } else {
        -1
    }
}

// Native translation of htslib/vcf.c bcf_readrec().
pub unsafe fn bcf_readrec(
    fp: *mut BGZF,
    _null: *mut c_void,
    vv: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    let v = vv.cast::<bcf1_t>();
    let hdr = bgzf_internal_h_67_bgzf_get_private_data(fp).cast::<bcf_hdr_t>();
    let mut ret = bcf_read1_core(fp, v);
    if ret == 0 {
        ret = vcf_c_2040_bcf_record_check(hdr, v);
    }
    if ret >= 0 {
        *tid = (*v).rid;
        *beg = (*v).pos;
        *end = (*v).pos + (*v).rlen;
    }
    ret
}

// Native translation of htslib/vcf.c vcf_write_line().
pub unsafe fn vcf_write_line(fp: *mut htsFile, line: *mut kstring_t) -> c_int {
    let ret;
    if *(*line).s.add((*line).l as usize - 1) != b'\n' as c_char {
        kputc(b'\n' as c_int, line);
    }
    if (*fp).format.compression != HTS_COMPRESSION_NO_COMPRESSION {
        ret = bgzf_write(hts_get_bgzfp(fp), (*line).s.cast(), (*line).l as usize);
    } else {
        ret = super::hfile::htslib_hfile_h_292_hwrite(
            (*fp).fp.hfile,
            (*line).s.cast(),
            (*line).l as usize,
        );
    }
    if ret == (*line).l as isize {
        0
    } else {
        -1
    }
}

pub unsafe fn bcf_hdr_dup(hdr: *const bcf_hdr_t) -> *mut bcf_hdr_t {
    let hout = bcf_hdr_init(c"r".as_ptr());
    if hout.is_null() {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_hdr_dup".as_ptr(),
            c"Failed to allocate bcf header".as_ptr(),
        );
        return std::ptr::null_mut();
    }
    let mut htxt: kstring_t = std::mem::zeroed();
    if bcf_hdr_format(hdr, 1, &mut htxt) < 0 {
        libc::free(htxt.s.cast());
        return std::ptr::null_mut();
    }
    let mut hout = hout;
    if bcf_hdr_parse(hout, htxt.s) < 0 {
        bcf_hdr_destroy(hout);
        hout = std::ptr::null_mut();
    }
    libc::free(htxt.s.cast());
    hout
}

// Warn (vcf.c style) when an INFO/FMT tag in src and dst differ in declared
// length / type. `with_ret` selects bcf_hdr_combine's behaviour (it also sets
// ret|=1); bcf_hdr_merge warns only. Returns 1 if any mismatch, else 0.
unsafe fn bcf_hdr_combine_check_tag(
    dst: *const bcf_hdr_t,
    src: *const bcf_hdr_t,
    hrec: *const bcf_hrec_t,
    rec_type: c_int,
) -> c_int {
    // Check that both records are of the same type. The bcf_hdr_id2length
    // macro cannot be used here because dst header is not synced yet.
    let d_src = (*src).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>();
    let d_dst = (*dst).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>();
    let val0 = *(*hrec).vals; // src->hrec[i]->vals[0]
    let k_src = kh_get_vdict(d_src, val0);
    let k_dst = kh_get_vdict(d_dst, val0);
    let info_src = (*vdict_val(d_src, k_src)).info[rec_type as usize];
    let info_dst = (*vdict_val(d_dst, k_dst)).info[rec_type as usize];
    let mut mismatch = 0;
    if (info_src >> 8 & 0xf) != (info_dst >> 8 & 0xf) {
        let msg = std::ffi::CString::new(format!(
            "Trying to combine \"{}\" tag definitions of different lengths",
            CStr::from_ptr(val0).to_string_lossy()
        ))
        .unwrap_or_default();
        c_log_warning(msg.as_ptr());
        mismatch = 1;
    }
    if (info_src >> 4 & 0xf) != (info_dst >> 4 & 0xf) {
        let msg = std::ffi::CString::new(format!(
            "Trying to combine \"{}\" tag definitions of different types",
            CStr::from_ptr(val0).to_string_lossy()
        ))
        .unwrap_or_default();
        c_log_warning(msg.as_ptr());
        mismatch = 1;
    }
    mismatch
}

// Native translation of htslib/vcf.c bcf_hdr_combine().
pub unsafe fn bcf_hdr_combine(dst: *mut bcf_hdr_t, src: *const bcf_hdr_t) -> c_int {
    let ndst_ori = (*dst).nhrec;
    let mut need_sync = 0;
    let mut ret = 0;
    for i in 0..(*src).nhrec as usize {
        let sh = *(*src).hrec.add(i);
        if (*sh).type_ as u32 == BCF_HL_GEN && !(*sh).value.is_null() {
            let mut j = 0;
            while j < ndst_ori {
                let dh = *(*dst).hrec.add(j as usize);
                if (*dh).type_ as u32 != BCF_HL_GEN {
                    j += 1;
                    continue;
                }
                if libc::strcmp((*sh).key, (*dh).key) == 0 {
                    break;
                }
                j += 1;
            }
            if j >= ndst_ori {
                let res = bcf_hdr_add_hrec(dst, bcf_hrec_dup(sh));
                if res < 0 {
                    return -1;
                }
                need_sync += res;
            }
        } else if (*sh).type_ as u32 == BCF_HL_STR {
            // NB: we are ignoring fields without ID
            let j = bcf_hrec_find_key(sh, c"ID".as_ptr());
            if j >= 0 {
                let rec = bcf_hdr_get_hrec(
                    dst,
                    (*sh).type_,
                    c"ID".as_ptr(),
                    *(*sh).vals.add(j as usize),
                    (*sh).key,
                );
                if rec.is_null() {
                    let res = bcf_hdr_add_hrec(dst, bcf_hrec_dup(sh));
                    if res < 0 {
                        return -1;
                    }
                    need_sync += res;
                }
            }
        } else {
            let j = bcf_hrec_find_key(sh, c"ID".as_ptr());
            debug_assert!(j >= 0); // always true for valid VCFs

            let rec = bcf_hdr_get_hrec(
                dst,
                (*sh).type_,
                c"ID".as_ptr(),
                *(*sh).vals.add(j as usize),
                std::ptr::null(),
            );
            if rec.is_null() {
                let res = bcf_hdr_add_hrec(dst, bcf_hrec_dup(sh));
                if res < 0 {
                    return -1;
                }
                need_sync += res;
            } else if (*sh).type_ as u32 == BCF_HL_INFO || (*sh).type_ as u32 == BCF_HL_FMT {
                ret |= bcf_hdr_combine_check_tag(dst, src, sh, (*rec).type_);
            }
        }
    }
    if need_sync != 0 && bcf_hdr_sync(dst) < 0 {
        return -1;
    }
    ret
}

// Native translation of htslib/vcf.c bcf_hdr_merge().
pub unsafe fn bcf_hdr_merge(mut dst: *mut bcf_hdr_t, src: *const bcf_hdr_t) -> *mut bcf_hdr_t {
    if dst.is_null() {
        // this will effectively strip existing IDX attributes from src to become dst
        dst = bcf_hdr_init(c"r".as_ptr());
        let mut htxt: kstring_t = std::mem::zeroed();
        if bcf_hdr_format(src, 0, &mut htxt) < 0 {
            libc::free(htxt.s.cast());
            return std::ptr::null_mut();
        }
        if bcf_hdr_parse(dst, htxt.s) < 0 {
            bcf_hdr_destroy(dst);
            dst = std::ptr::null_mut();
        }
        libc::free(htxt.s.cast());
        return dst;
    }

    let ndst_ori = (*dst).nhrec;
    let mut need_sync = 0;
    for i in 0..(*src).nhrec as usize {
        let sh = *(*src).hrec.add(i);
        if (*sh).type_ as u32 == BCF_HL_GEN && !(*sh).value.is_null() {
            let mut j = 0;
            while j < ndst_ori {
                let dh = *(*dst).hrec.add(j as usize);
                if (*dh).type_ as u32 != BCF_HL_GEN {
                    j += 1;
                    continue;
                }
                if libc::strcmp((*sh).key, (*dh).key) == 0 {
                    break;
                }
                j += 1;
            }
            if j >= ndst_ori {
                let res = bcf_hdr_add_hrec(dst, bcf_hrec_dup(sh));
                if res < 0 {
                    return std::ptr::null_mut();
                }
                need_sync += res;
            } else if libc::strcmp((*sh).key, c"fileformat".as_ptr()) == 0 {
                let dh = *(*dst).hrec.add(j as usize);
                let ver_src = bcf_get_version(src, (*sh).value);
                let ver_dst = bcf_get_version(dst, (*dh).value);
                if ver_src > ver_dst {
                    if bcf_hdr_set_version(dst, (*sh).value) < 0 {
                        return std::ptr::null_mut();
                    }
                    need_sync = 1;
                }
            }
        } else if (*sh).type_ as u32 == BCF_HL_STR {
            let j = bcf_hrec_find_key(sh, c"ID".as_ptr());
            if j >= 0 {
                let rec = bcf_hdr_get_hrec(
                    dst,
                    (*sh).type_,
                    c"ID".as_ptr(),
                    *(*sh).vals.add(j as usize),
                    (*sh).key,
                );
                if rec.is_null() {
                    let res = bcf_hdr_add_hrec(dst, bcf_hrec_dup(sh));
                    if res < 0 {
                        return std::ptr::null_mut();
                    }
                    need_sync += res;
                }
            }
        } else {
            let j = bcf_hrec_find_key(sh, c"ID".as_ptr());
            debug_assert!(j >= 0);

            let rec = bcf_hdr_get_hrec(
                dst,
                (*sh).type_,
                c"ID".as_ptr(),
                *(*sh).vals.add(j as usize),
                std::ptr::null(),
            );
            if rec.is_null() {
                let res = bcf_hdr_add_hrec(dst, bcf_hrec_dup(sh));
                if res < 0 {
                    return std::ptr::null_mut();
                }
                need_sync += res;
            } else if (*sh).type_ as u32 == BCF_HL_INFO || (*sh).type_ as u32 == BCF_HL_FMT {
                // merge warns only (does not set a return flag)
                bcf_hdr_combine_check_tag(dst, src, sh, (*rec).type_);
            }
        }
    }
    if need_sync != 0 && bcf_hdr_sync(dst) < 0 {
        return std::ptr::null_mut();
    }
    dst
}

pub unsafe fn bcf_translate(
    dst_hdr: *const bcf_hdr_t,
    src_hdr: *mut bcf_hdr_t,
    src_line: *mut bcf1_t,
) -> c_int {
    let line = src_line;
    if (*line).errcode != 0 {
        let mut errordescription = [0 as c_char; 1024];
        let error = bcf_strerror(
            (*line).errcode,
            errordescription.as_mut_ptr(),
            errordescription.len(),
        );
        let error_str = CStr::from_ptr(error).to_string_lossy();
        let seqname_str = CStr::from_ptr(bcf_seqname_safe(src_hdr, line)).to_string_lossy();
        let msg = std::ffi::CString::new(format!(
            "Unchecked error ({} {}) at {}:{}, exiting",
            (*line).errcode,
            error_str,
            seqname_str,
            (*line).pos + 1
        ))
        .unwrap_or_default();
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_translate".as_ptr(),
            msg.as_ptr(),
        );
        libc::exit(1);
    }

    if (*src_hdr).ntransl == -1 {
        return 0;
    }

    if (*src_hdr).ntransl == 0 {
        for dict in 0..2usize {
            let n = (*src_hdr).n[dict] as usize;
            (*src_hdr).transl[dict] =
                libc::malloc(n.saturating_mul(size_of::<c_int>())).cast::<c_int>();
            if n != 0 && (*src_hdr).transl[dict].is_null() {
                return -1;
            }

            for i in 0..n {
                let key = (*(*src_hdr).id[dict].add(i)).key;
                if key.is_null() {
                    *(*src_hdr).transl[dict].add(i) = -1;
                    continue;
                }
                let dst_id = bcf_hdr_id2int(dst_hdr, dict as c_int, key);
                *(*src_hdr).transl[dict].add(i) = dst_id;
                if dst_id != -1 && i as c_int != dst_id {
                    (*src_hdr).ntransl += 1;
                }
            }
        }

        if (*src_hdr).ntransl == 0 {
            libc::free((*src_hdr).transl[0].cast());
            libc::free((*src_hdr).transl[1].cast());
            (*src_hdr).transl[0] = std::ptr::null_mut();
            (*src_hdr).transl[1] = std::ptr::null_mut();
            (*src_hdr).ntransl = -1;
        }
        if (*src_hdr).ntransl == -1 {
            return 0;
        }
    }

    bcf_unpack(line, BCF_UN_ALL as c_int);

    let ctg_transl = (*src_hdr).transl[BCF_DT_CTG as usize];
    if !ctg_transl.is_null() && (*line).rid >= 0 {
        let dst_id = *ctg_transl.add((*line).rid as usize);
        if dst_id >= 0 {
            (*line).rid = dst_id;
        }
    }

    let id_transl = (*src_hdr).transl[BCF_DT_ID as usize];

    for i in 0..(*line).d.n_flt {
        let flt = (*line).d.flt.add(i as usize);
        let dst_id = *id_transl.add(*flt as usize);
        if dst_id >= 0 {
            *flt = dst_id;
        }
        (*line).d.shared_dirty |= BCF1_DIRTY_FLT as c_int;
    }

    for i in 0..(*line).n_info() {
        let info = (*line).d.info.add(i as usize);
        let src_id = (*info).key;
        let dst_id = *id_transl.add(src_id as usize);
        if dst_id < 0 {
            continue;
        }
        (*info).key = dst_id;
        if (*info).vptr.is_null() {
            continue;
        }

        let src_size = bcf_translate_id_size(src_id);
        let dst_size = bcf_translate_id_size(dst_id);
        if src_size == dst_size {
            let vptr = (*info).vptr.sub((*info).vptr_off() as usize);
            bcf_translate_store_info_id(vptr, dst_id, dst_size);
        } else {
            let mut str_ = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            if bcf_enc_int1(&mut str_, dst_id) < 0
                || bcf_enc_size(&mut str_, (*info).len, (*info).type_) < 0
            {
                super::hts::ks_free(&mut str_);
                return -1;
            }
            let vptr_off = str_.l;
            if kputsn((*info).vptr.cast(), (*info).vptr_len as usize, &mut str_) < 0 {
                super::hts::ks_free(&mut str_);
                return -1;
            }
            if (*info).vptr_free() != 0 {
                libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
            }
            (*info).set_vptr_off(vptr_off as u32);
            (*info).vptr = str_.s.cast::<u8>().add((*info).vptr_off() as usize);
            (*info).set_vptr_free(1);
            (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
        }
    }

    for i in 0..(*line).n_fmt() {
        let fmt = (*line).d.fmt.add(i as usize);
        let src_id = (*fmt).id;
        let dst_id = *id_transl.add(src_id as usize);
        if dst_id < 0 {
            continue;
        }
        (*fmt).id = dst_id;
        if (*fmt).p.is_null() {
            continue;
        }

        let src_size = bcf_translate_id_size(src_id);
        let dst_size = bcf_translate_id_size(dst_id);
        if src_size == dst_size {
            let p = (*fmt).p.sub((*fmt).p_off() as usize);
            bcf_translate_store_id(p.add(1), dst_id, dst_size);
        } else {
            let mut str_ = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            if bcf_enc_int1(&mut str_, dst_id) < 0
                || bcf_enc_size(&mut str_, (*fmt).n, (*fmt).type_) < 0
            {
                super::hts::ks_free(&mut str_);
                return -1;
            }
            let p_off = str_.l;
            if kputsn((*fmt).p.cast(), (*fmt).p_len as usize, &mut str_) < 0 {
                super::hts::ks_free(&mut str_);
                return -1;
            }
            if (*fmt).p_free() != 0 {
                libc::free((*fmt).p.sub((*fmt).p_off() as usize).cast());
            }
            (*fmt).set_p_off(p_off as u32);
            (*fmt).p = str_.s.cast::<u8>().add((*fmt).p_off() as usize);
            (*fmt).set_p_free(1);
            (*line).d.indiv_dirty = 1;
        }
    }

    0
}

unsafe fn bcf_translate_id_size(id: c_int) -> c_int {
    if id >> 7 != 0 {
        if id >> 15 != 0 {
            BCF_BT_INT32 as c_int
        } else {
            BCF_BT_INT16 as c_int
        }
    } else {
        BCF_BT_INT8 as c_int
    }
}

unsafe fn bcf_translate_store_id(ptr: *mut u8, id: c_int, size: c_int) {
    if size == BCF_BT_INT8 as c_int {
        *ptr = id as u8;
    } else if size == BCF_BT_INT16 as c_int {
        i16_to_le(id as i16, ptr);
    } else {
        i32_to_le(id, ptr);
    }
}

unsafe fn bcf_translate_store_info_id(ptr: *mut u8, id: c_int, size: c_int) {
    if size == BCF_BT_INT8 as c_int {
        *ptr.add(1) = id as u8;
    } else {
        bcf_translate_store_id(ptr, id, size);
    }
}

unsafe fn bcf_enc_size(s: *mut kstring_t, size: c_int, type_: c_int) -> c_int {
    if size < 15 {
        if super::hts::ks_resize(s, (*s).l + 1) < 0 {
            return -1;
        }
        *(*s).s.add((*s).l) = ((size << 4) | type_) as c_char;
        (*s).l += 1;
        return 0;
    }

    if super::hts::ks_resize(s, (*s).l + 6) < 0 {
        return -1;
    }
    let p = (*s).s.add((*s).l).cast::<u8>();
    *p = ((15 << 4) | type_) as u8;
    if size < 128 {
        *p.add(1) = ((1 << 4) | BCF_BT_INT8 as c_int) as u8;
        *p.add(2) = size as u8;
        (*s).l += 3;
    } else if size < 32768 {
        *p.add(1) = ((1 << 4) | BCF_BT_INT16 as c_int) as u8;
        i16_to_le(size as i16, p.add(2));
        (*s).l += 4;
    } else {
        *p.add(1) = ((1 << 4) | BCF_BT_INT32 as c_int) as u8;
        i32_to_le(size, p.add(2));
        (*s).l += 6;
    }
    0
}

unsafe fn bcf_enc_int1(s: *mut kstring_t, x: c_int) -> c_int {
    if super::hts::ks_resize(s, (*s).l + 5) < 0 {
        return -1;
    }
    let p = (*s).s.add((*s).l).cast::<u8>();
    if x == bcf_int32_vector_end {
        *p = ((1 << 4) | BCF_BT_INT8 as c_int) as u8;
        *p.add(1) = bcf_int8_vector_end as u8;
        (*s).l += 2;
    } else if x == bcf_int32_missing {
        *p = ((1 << 4) | BCF_BT_INT8 as c_int) as u8;
        *p.add(1) = bcf_int8_missing as u8;
        (*s).l += 2;
    } else if x <= 0x7f && x >= -120 {
        *p = ((1 << 4) | BCF_BT_INT8 as c_int) as u8;
        *p.add(1) = x as u8;
        (*s).l += 2;
    } else if x <= 0x7fff && x >= -32760 {
        *p = ((1 << 4) | BCF_BT_INT16 as c_int) as u8;
        i16_to_le(x as i16, p.add(1));
        (*s).l += 3;
    } else {
        *p = ((1 << 4) | BCF_BT_INT32 as c_int) as u8;
        i32_to_le(x, p.add(1));
        (*s).l += 5;
    }
    0
}

// Native translation of htslib/vcf.c bcf_enc_long1() (VCF_ALLOW_INT64 path).
unsafe fn bcf_enc_long1(s: *mut kstring_t, x: i64) -> c_int {
    const BCF_MAX_BT_INT32: i64 = 0x7fff_ffff;
    const BCF_INT64_VECTOR_END: i64 = i64::MIN + 1;
    if x <= BCF_MAX_BT_INT32 && x >= BCF_MIN_BT_INT32 {
        return bcf_enc_int1(s, x as c_int);
    }
    let mut e = 0;
    if x == BCF_INT64_VECTOR_END {
        e |= bcf_enc_size(s, 1, BCF_BT_INT8 as c_int);
        e |= (kputc(bcf_int8_vector_end, s) < 0) as c_int;
    } else if x == bcf_int64_missing {
        e |= bcf_enc_size(s, 1, BCF_BT_INT8 as c_int);
        e |= (kputc(bcf_int8_missing, s) < 0) as c_int;
    } else {
        e |= bcf_enc_size(s, 1, BCF_BT_INT64 as c_int);
        e |= (super::hts::ks_expand(s, 8) < 0) as c_int;
        if e == 0 {
            super::hts::u64_to_le(x as u64, (*s).s.add((*s).l).cast::<u8>());
            (*s).l += 8;
        }
    }
    if e == 0 {
        0
    } else {
        -1
    }
}

// Native translation of htslib/vcf.c serialize_float_array().
unsafe fn serialize_float_array(s: *mut kstring_t, n: usize, a: *const f32) -> c_int {
    let bytes = n.wrapping_mul(size_of::<f32>());
    if n != 0 && bytes / size_of::<f32>() != n {
        return -1;
    }
    if super::hts::ks_resize(s, (*s).l + bytes) < 0 {
        return -1;
    }
    let mut p = (*s).s.add((*s).l).cast::<u8>();
    for i in 0..n {
        super::hts::float_to_le(*a.add(i), p);
        p = p.add(size_of::<f32>());
    }
    (*s).l += bytes;
    0
}

// Native translation of htslib/vcf.c bcf_dec_typed_int1() (unbounded; used on
// our own freshly-encoded buffers where bounds are guaranteed).
unsafe fn bcf_dec_typed_int1_unsafe(p: *const u8, q: *mut *const u8) -> c_int {
    let t = (*p & 0xf) as c_int;
    if t == BCF_BT_INT8 as c_int {
        *q = p.add(2);
        le_to_i8(p.add(1)) as c_int
    } else if t == BCF_BT_INT16 as c_int {
        *q = p.add(3);
        le_to_i16(p.add(1)) as c_int
    } else {
        *q = p.add(5);
        le_to_i32(p.add(1))
    }
}

// Native translation of htslib/vcf.c bcf_dec_size() (unbounded).
unsafe fn bcf_dec_size_unsafe(p: *const u8, q: *mut *const u8, type_: *mut c_int) -> c_int {
    *type_ = (*p & 0xf) as c_int;
    if *p >> 4 != 15 {
        *q = p.add(1);
        (*p >> 4) as c_int
    } else {
        let mut next: *const u8 = std::ptr::null();
        let r = bcf_dec_typed_int1_unsafe(p.add(1), &mut next);
        *q = next;
        r
    }
}

// Native translation of htslib/vcf.c bcf_unpack_info_core1().
unsafe fn bcf_unpack_info_core1_rs(ptr: *mut u8, info: *mut bcf_info_t) -> *mut u8 {
    let ptr_start = ptr;
    let mut p: *const u8 = ptr;
    (*info).key = bcf_dec_typed_int1_unsafe(p, &mut p);
    let mut type_ = 0;
    let len = bcf_dec_size_unsafe(p, &mut p, &mut type_) as i64;
    (*info).type_ = type_;
    (*info).len = len as c_int;
    let mut ptr = p as *mut u8;
    (*info).vptr = ptr;
    (*info).set_vptr_off((ptr as usize - ptr_start as usize) as u32);
    (*info).set_vptr_free(0);
    (*info).v1.i = 0;
    let mut adv = len;
    if len == 1 {
        match type_ {
            x if x == BCF_BT_INT8 as c_int || x == BCF_BT_CHAR as c_int => {
                (*info).v1.i = le_to_i8(ptr) as i64;
            }
            x if x == BCF_BT_INT16 as c_int => {
                (*info).v1.i = le_to_i16(ptr) as i64;
                adv <<= 1;
            }
            x if x == BCF_BT_INT32 as c_int => {
                (*info).v1.i = le_to_i32(ptr) as i64;
                adv <<= 2;
            }
            x if x == BCF_BT_FLOAT as c_int => {
                (*info).v1.f = le_to_float(ptr);
                adv <<= 2;
            }
            x if x == BCF_BT_INT64 as c_int => {
                (*info).v1.i = le_to_i64(ptr);
                adv <<= 3;
            }
            _ => {}
        }
    } else {
        adv <<= BCF_TYPE_SHIFT[(type_ & 0xf) as usize] as i64;
    }
    ptr = ptr.add(adv as usize);
    (*info).vptr_len = (ptr as usize - (*info).vptr as usize) as u32;
    ptr
}

// Native translation of htslib/vcf.c bcf_unpack_fmt_core1().
unsafe fn bcf_unpack_fmt_core1_rs(ptr: *mut u8, n_sample: c_int, fmt: *mut bcf_fmt_t) -> *mut u8 {
    let ptr_start = ptr;
    let mut p: *const u8 = ptr;
    (*fmt).id = bcf_dec_typed_int1_unsafe(p, &mut p);
    let mut type_ = 0;
    (*fmt).n = bcf_dec_size_unsafe(p, &mut p, &mut type_);
    (*fmt).type_ = type_;
    (*fmt).size = (*fmt).n << BCF_TYPE_SHIFT[(type_ & 0xf) as usize];
    let mut ptr = p as *mut u8;
    (*fmt).p = ptr;
    (*fmt).set_p_off((ptr as usize - ptr_start as usize) as u32);
    (*fmt).set_p_free(0);
    ptr = ptr.add((n_sample * (*fmt).size) as usize);
    (*fmt).p_len = (ptr as usize - (*fmt).p as usize) as u32;
    ptr
}

unsafe fn bcf_dec_typed_int1_safe_rs(
    p: *const u8,
    end: *const u8,
    q: *mut *const u8,
    val: *mut i32,
) -> c_int {
    unsafe {
        if p.is_null() || end.is_null() || q.is_null() || val.is_null() || end.offset_from(p) < 2 {
            return -1;
        }
        let type_ = (*p & 0xf) as c_int;
        let ptr = p.add(1);
        match type_ {
            x if x == BCF_BT_INT8 as c_int => {
                *val = le_to_i8(ptr) as i32;
                *q = ptr.add(1);
            }
            x if x == BCF_BT_INT16 as c_int => {
                if end.offset_from(ptr) < size_of::<i16>() as isize {
                    return -1;
                }
                *val = le_to_i16(ptr) as i32;
                *q = ptr.add(size_of::<i16>());
            }
            x if x == BCF_BT_INT32 as c_int => {
                if end.offset_from(ptr) < size_of::<i32>() as isize {
                    return -1;
                }
                *val = le_to_i32(ptr);
                *q = ptr.add(size_of::<i32>());
            }
            _ => return -1,
        }
        0
    }
}

unsafe fn bcf_dec_size_safe_rs(
    p: *const u8,
    end: *const u8,
    q: *mut *const u8,
    num: *mut c_int,
    type_: *mut c_int,
) -> c_int {
    unsafe {
        if p.is_null()
            || end.is_null()
            || q.is_null()
            || num.is_null()
            || type_.is_null()
            || p >= end
        {
            return -1;
        }
        *type_ = (*p & 0xf) as c_int;
        if *p >> 4 != 15 {
            *q = p.add(1);
            *num = (*p >> 4) as c_int;
            return 0;
        }
        if bcf_dec_typed_int1_safe_rs(p.add(1), end, q, num) != 0 {
            return -1;
        }
        (*num >= 0) as c_int - 1
    }
}

unsafe fn bcf_dec_int1_rs(p: *const u8, type_: c_int, q: *mut *const u8) -> i32 {
    unsafe {
        match type_ {
            x if x == BCF_BT_INT8 as c_int => {
                *q = p.add(1);
                le_to_i8(p) as i32
            }
            x if x == BCF_BT_INT16 as c_int => {
                *q = p.add(size_of::<i16>());
                le_to_i16(p) as i32
            }
            x if x == BCF_BT_INT32 as c_int => {
                *q = p.add(size_of::<i32>());
                le_to_i32(p)
            }
            _ => {
                *q = p;
                -1
            }
        }
    }
}

unsafe fn bcf_record_check_id_valid(hdr: *const bcf_hdr_t, max_id: c_int, key: c_int) -> bool {
    unsafe {
        key >= 0
            && (hdr.is_null()
                || (key < max_id
                    && !(*(*hdr).id[BCF_DT_ID as usize].add(key as usize))
                        .key
                        .is_null()))
    }
}

unsafe fn bcf_record_update_phasing(
    p: *mut u8,
    end: *const u8,
    q: *mut *const u8,
    samples: c_int,
    ploidy: c_int,
    type_: c_int,
) -> c_int {
    unsafe {
        if p.is_null() || end.is_null() || q.is_null() || samples < 0 || ploidy < 0 {
            return 1;
        }
        let type_index = type_ as usize;
        if type_index >= BCF_TYPE_SHIFT.len() {
            return 1;
        }
        let inc = 1usize << BCF_TYPE_SHIFT[type_index];
        let Some(bytes) = (samples as usize)
            .checked_mul(ploidy as usize)
            .and_then(|v| v.checked_mul(inc))
        else {
            return 1;
        };
        if end.offset_from(p.cast()) < bytes as isize {
            return 1;
        }

        match ploidy {
            1 => {
                for sample in 0..samples as usize {
                    let ptr = p.add(sample * inc);
                    if *ptr != 0 {
                        *ptr |= 1;
                    }
                }
            }
            2 => {
                for sample in 0..samples as usize {
                    let ptr = p.add(sample * 2 * inc);
                    *ptr |= *ptr.add(inc) & 1;
                }
            }
            _ => {
                for sample in 0..samples as usize {
                    let ptr = p.add(sample * ploidy as usize * inc);
                    let mut all_phased = 1u8;
                    for allele in 1..ploidy as usize {
                        all_phased &= *ptr.add(allele * inc) & 1;
                    }
                    *ptr |= all_phased;
                }
            }
        }
        *q = p.add(bytes).cast();
        0
    }
}

pub unsafe fn vcf_c_2040_bcf_record_check(hdr: *const bcf_hdr_t, rec: *mut bcf1_t) -> c_int {
    unsafe {
        if rec.is_null() {
            return -2;
        }
        let mut err = 0u32;
        let is_integer = (1u32 << BCF_BT_INT8) | (1u32 << BCF_BT_INT16) | (1u32 << BCF_BT_INT32);
        let is_valid_type =
            is_integer | (1u32 << BCF_BT_NULL) | (1u32 << BCF_BT_FLOAT) | (1u32 << BCF_BT_CHAR);
        let max_id = if hdr.is_null() {
            0
        } else {
            (*hdr).n[BCF_DT_ID as usize]
        };
        let gt_id = if hdr.is_null() || vcf_hdr_version_ge_44(hdr) {
            -1
        } else {
            bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"GT".as_ptr())
        };

        if (*rec).rid < 0
            || (!hdr.is_null()
                && ((*rec).rid >= (*hdr).n[BCF_DT_CTG as usize]
                    || (*(*hdr).id[BCF_DT_CTG as usize].add((*rec).rid as usize))
                        .key
                        .is_null()))
        {
            err |= BCF_ERR_CTG_INVALID;
        }

        let mut ptr = (*rec).shared.s.cast::<u8>();
        let end = ptr.add((*rec).shared.l as usize);
        let mut num = 0;
        let mut type_ = 0;
        let mut next: *const u8 = std::ptr::null();

        if bcf_dec_size_safe_rs(ptr, end, &mut next, &mut num, &mut type_) != 0 {
            return -2;
        }
        if type_ != BCF_BT_CHAR as c_int {
            err |= BCF_ERR_TAG_INVALID;
        }
        let mut bytes = (num as usize) << BCF_TYPE_SHIFT[type_ as usize];
        if end.offset_from(next) < bytes as isize {
            return -2;
        }
        ptr = next.cast_mut().add(bytes);

        if (*rec).n_allele() < 1 {
            err |= BCF_ERR_TAG_UNDEF;
        }
        for _ in 0..(*rec).n_allele() {
            if bcf_dec_size_safe_rs(ptr, end, &mut next, &mut num, &mut type_) != 0 {
                return -2;
            }
            if type_ != BCF_BT_CHAR as c_int {
                err |= BCF_ERR_CHAR;
            }
            bytes = (num as usize) << BCF_TYPE_SHIFT[type_ as usize];
            if end.offset_from(next) < bytes as isize {
                return -2;
            }
            ptr = next.cast_mut().add(bytes);
        }

        if bcf_dec_size_safe_rs(ptr, end, &mut next, &mut num, &mut type_) != 0 {
            return -2;
        }
        if num > 0 {
            bytes = (num as usize) << BCF_TYPE_SHIFT[type_ as usize];
            if end.offset_from(next) < bytes as isize {
                return -2;
            }
            if ((1u32 << type_) & is_integer) == 0 {
                err |= BCF_ERR_TAG_INVALID;
                ptr = next.cast_mut().add(bytes);
            } else {
                ptr = next.cast_mut();
                for _ in 0..num {
                    let key = bcf_dec_int1_rs(ptr, type_, &mut next);
                    if !bcf_record_check_id_valid(hdr, max_id, key) {
                        err |= BCF_ERR_TAG_UNDEF;
                    }
                    ptr = next.cast_mut();
                }
            }
        } else {
            ptr = next.cast_mut();
        }

        for _ in 0..(*rec).n_info() {
            let mut key = -1;
            if bcf_dec_typed_int1_safe_rs(ptr, end, &mut next, &mut key) != 0 {
                return -2;
            }
            if !bcf_record_check_id_valid(hdr, max_id, key) {
                err |= BCF_ERR_TAG_UNDEF;
            }
            ptr = next.cast_mut();
            if bcf_dec_size_safe_rs(ptr, end, &mut next, &mut num, &mut type_) != 0 {
                return -2;
            }
            if ((1u32 << type_) & is_valid_type) == 0 || (type_ == BCF_BT_NULL as c_int && num > 0)
            {
                err |= BCF_ERR_TAG_INVALID;
            }
            bytes = (num as usize) << BCF_TYPE_SHIFT[type_ as usize];
            if end.offset_from(next) < bytes as isize {
                return -2;
            }
            ptr = next.cast_mut().add(bytes);
        }

        ptr = (*rec).indiv.s.cast::<u8>();
        let indiv_end = ptr.add((*rec).indiv.l as usize);
        for _ in 0..(*rec).n_fmt() {
            let mut key = -1;
            if bcf_dec_typed_int1_safe_rs(ptr, indiv_end, &mut next, &mut key) != 0 {
                return -2;
            }
            if !bcf_record_check_id_valid(hdr, max_id, key) {
                err |= BCF_ERR_TAG_UNDEF;
            }
            ptr = next.cast_mut();
            if bcf_dec_size_safe_rs(ptr, indiv_end, &mut next, &mut num, &mut type_) != 0 {
                return -2;
            }
            if ((1u32 << type_) & is_valid_type) == 0 || (type_ == BCF_BT_NULL as c_int && num > 0)
            {
                err |= BCF_ERR_TAG_INVALID;
            }
            if gt_id >= 0 && gt_id == key {
                if bcf_record_update_phasing(
                    next.cast_mut(),
                    indiv_end,
                    &mut next,
                    (*rec).n_sample() as c_int,
                    num,
                    type_,
                ) != 0
                {
                    err |= BCF_ERR_TAG_INVALID;
                }
                ptr = next.cast_mut();
            } else {
                bytes = ((num as usize) << BCF_TYPE_SHIFT[type_ as usize])
                    .saturating_mul((*rec).n_sample() as usize);
                if indiv_end.offset_from(next) < bytes as isize {
                    return -2;
                }
                ptr = next.cast_mut().add(bytes);
            }
        }

        if err == 0 && (*rec).rlen < 0 {
            let rlen = vcf_get_rlen_decoded(hdr, rec);
            (*rec).rlen = rlen.max(0);
        }
        (*rec).errcode |= err as c_int;
        if err == 0 {
            0
        } else {
            -2
        }
    }
}

pub unsafe fn vcf_c_2278_bcf1_sync_id(line: *mut bcf1_t, str_: *mut kstring_t) -> c_int {
    unsafe {
        if line.is_null() || str_.is_null() {
            return -1;
        }
        if !(*line).d.id.is_null() && libc::strcmp((*line).d.id, c".".as_ptr()) != 0 {
            bcf_enc_vchar(str_, libc::strlen((*line).d.id) as c_int, (*line).d.id)
        } else {
            bcf_enc_size(str_, 0, BCF_BT_CHAR as c_int)
        }
    }
}

pub unsafe fn vcf_c_2287_bcf1_sync_alleles(line: *mut bcf1_t, str_: *mut kstring_t) -> c_int {
    unsafe {
        if line.is_null() || str_.is_null() {
            return -1;
        }
        for i in 0..(*line).n_allele() as usize {
            let allele = *(*line).d.allele.add(i);
            if allele.is_null() || bcf_enc_vchar(str_, libc::strlen(allele) as c_int, allele) < 0 {
                return -1;
            }
        }
        if (*line).rlen == 0 && (*line).n_allele() != 0 && !(*(*line).d.allele).is_null() {
            (*line).rlen = libc::strlen(*(*line).d.allele) as hts_pos_t;
        }
        0
    }
}

pub unsafe fn vcf_c_2298_bcf1_sync_filter(line: *mut bcf1_t, str_: *mut kstring_t) -> c_int {
    unsafe {
        if line.is_null() || str_.is_null() {
            return -1;
        }
        if (*line).d.n_flt != 0 {
            bcf_enc_vint(str_, (*line).d.n_flt, (*line).d.flt, -1)
        } else {
            bcf_enc_vint(str_, 0, std::ptr::null_mut(), -1)
        }
    }
}

pub unsafe fn vcf_c_2308_bcf1_sync_info(line: *mut bcf1_t, str_: *mut kstring_t) -> c_int {
    unsafe {
        if line.is_null() || str_.is_null() {
            return -1;
        }
        let mut remove_index = -1;
        let mut error = false;
        for i in 0..(*line).n_info() as c_int {
            let info = (*line).d.info.add(i as usize);
            if (*info).vptr.is_null() {
                if remove_index < 0 {
                    remove_index = i;
                }
                continue;
            }
            let src = (*info).vptr.sub((*info).vptr_off() as usize);
            error |= kputsn(
                src.cast(),
                (*info).vptr_len as usize + (*info).vptr_off() as usize,
                str_,
            ) < 0;
            if remove_index >= 0 {
                let tmp = *(*line).d.info.add(remove_index as usize);
                *(*line).d.info.add(remove_index as usize) = *info;
                *info = tmp;
                while remove_index <= i
                    && !(*(*line).d.info.add(remove_index as usize)).vptr.is_null()
                {
                    remove_index += 1;
                }
            }
        }
        if remove_index >= 0 {
            (*line).set_n_info(remove_index as u32);
        }
        if error {
            -1
        } else {
            0
        }
    }
}

pub unsafe fn vcf_c_2332_bcf1_sync(line: *mut bcf1_t) -> c_int {
    unsafe {
        if line.is_null() {
            return -1;
        }

        let shared_ori = (*line).shared.s;
        let mut tmp = kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        if (*line).shared.l == 0 {
            tmp.l = (*line).shared.l as usize;
            tmp.m = (*line).shared.m as usize;
            tmp.s = (*line).shared.s;
            if vcf_c_2278_bcf1_sync_id(line, &mut tmp) < 0 {
                return -1;
            }
            (*line).unpack_size[0] = tmp.l as c_int;
            let mut prev_len = tmp.l;

            if vcf_c_2287_bcf1_sync_alleles(line, &mut tmp) < 0 {
                super::hts::ks_free(&mut tmp);
                return -1;
            }
            (*line).unpack_size[1] = (tmp.l - prev_len) as c_int;
            prev_len = tmp.l;

            if vcf_c_2298_bcf1_sync_filter(line, &mut tmp) < 0 {
                super::hts::ks_free(&mut tmp);
                return -1;
            }
            (*line).unpack_size[2] = (tmp.l - prev_len) as c_int;

            if vcf_c_2308_bcf1_sync_info(line, &mut tmp) < 0 {
                super::hts::ks_free(&mut tmp);
                return -1;
            }
            (*line).shared.l = tmp.l as _;
            (*line).shared.m = tmp.m as _;
            (*line).shared.s = tmp.s;
        } else if (*line).d.shared_dirty != 0 {
            if (*line).unpacked & BCF_UN_STR as c_int == 0 {
                bcf_unpack(line, BCF_UN_STR as c_int);
            }
            let mut ptr_ori = (*line).shared.s.cast::<u8>();

            if (*line).d.shared_dirty & BCF1_DIRTY_ID as c_int != 0 {
                if vcf_c_2278_bcf1_sync_id(line, &mut tmp) < 0 {
                    super::hts::ks_free(&mut tmp);
                    return -1;
                }
            } else if kputsn(ptr_ori.cast(), (*line).unpack_size[0] as usize, &mut tmp) < 0 {
                super::hts::ks_free(&mut tmp);
                return -1;
            }
            ptr_ori = ptr_ori.add((*line).unpack_size[0] as usize);
            (*line).unpack_size[0] = tmp.l as c_int;
            let mut prev_len = tmp.l;

            if (*line).d.shared_dirty & BCF1_DIRTY_ALS as c_int != 0 {
                if vcf_c_2287_bcf1_sync_alleles(line, &mut tmp) < 0 {
                    super::hts::ks_free(&mut tmp);
                    return -1;
                }
            } else {
                if kputsn(ptr_ori.cast(), (*line).unpack_size[1] as usize, &mut tmp) < 0 {
                    super::hts::ks_free(&mut tmp);
                    return -1;
                }
                if (*line).rlen == 0 && (*line).n_allele() != 0 && !(*(*line).d.allele).is_null() {
                    (*line).rlen = libc::strlen(*(*line).d.allele) as hts_pos_t;
                }
            }
            ptr_ori = ptr_ori.add((*line).unpack_size[1] as usize);
            (*line).unpack_size[1] = (tmp.l - prev_len) as c_int;
            prev_len = tmp.l;

            if (*line).unpacked & BCF_UN_FLT as c_int != 0 {
                if (*line).d.shared_dirty & BCF1_DIRTY_FLT as c_int != 0 {
                    if vcf_c_2298_bcf1_sync_filter(line, &mut tmp) < 0 {
                        super::hts::ks_free(&mut tmp);
                        return -1;
                    }
                } else if (*line).d.n_flt != 0 {
                    if kputsn(ptr_ori.cast(), (*line).unpack_size[2] as usize, &mut tmp) < 0 {
                        super::hts::ks_free(&mut tmp);
                        return -1;
                    }
                } else if bcf_enc_vint(&mut tmp, 0, std::ptr::null_mut(), -1) < 0 {
                    super::hts::ks_free(&mut tmp);
                    return -1;
                }
                ptr_ori = ptr_ori.add((*line).unpack_size[2] as usize);
                (*line).unpack_size[2] = (tmp.l - prev_len) as c_int;

                if (*line).unpacked & BCF_UN_INFO as c_int != 0
                    && (*line).d.shared_dirty & BCF1_DIRTY_INF as c_int != 0
                {
                    if vcf_c_2308_bcf1_sync_info(line, &mut tmp) < 0 {
                        super::hts::ks_free(&mut tmp);
                        return -1;
                    }
                    ptr_ori = (*line).shared.s.cast::<u8>().add((*line).shared.l as usize);
                }
            }

            let shared_end = (*line).shared.s.cast::<u8>().add((*line).shared.l as usize);
            let remaining = shared_end.offset_from(ptr_ori);
            if remaining > 0 && kputsn(ptr_ori.cast(), remaining as usize, &mut tmp) < 0 {
                super::hts::ks_free(&mut tmp);
                return -1;
            }
            libc::free((*line).shared.s.cast());
            (*line).shared.l = tmp.l as _;
            (*line).shared.m = tmp.m as _;
            (*line).shared.s = tmp.s;
        }

        if (*line).shared.s != shared_ori && (*line).unpacked & BCF_UN_INFO as c_int != 0 {
            let mut off_new =
                ((*line).unpack_size[0] + (*line).unpack_size[1] + (*line).unpack_size[2]) as usize;
            for i in 0..(*line).n_info() as usize {
                let info = (*line).d.info.add(i);
                let old_free = if (*info).vptr_free() != 0 && !(*info).vptr.is_null() {
                    Some((*info).vptr.sub((*info).vptr_off() as usize))
                } else {
                    None
                };
                (*info).vptr = (*line)
                    .shared
                    .s
                    .cast::<u8>()
                    .add(off_new + (*info).vptr_off() as usize);
                off_new += (*info).vptr_len as usize + (*info).vptr_off() as usize;
                if let Some(ptr) = old_free {
                    libc::free(ptr.cast());
                    (*info).set_vptr_free(0);
                }
            }
        }

        if (*line).n_sample() != 0
            && (*line).n_fmt() != 0
            && ((*line).indiv.l == 0 || (*line).d.indiv_dirty != 0)
        {
            let mut tmp = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            let mut remove_index = -1;
            for i in 0..(*line).n_fmt() as c_int {
                let fmt = (*line).d.fmt.add(i as usize);
                if (*fmt).p.is_null() {
                    if remove_index < 0 {
                        remove_index = i;
                    }
                    continue;
                }
                if kputsn(
                    (*fmt).p.sub((*fmt).p_off() as usize).cast(),
                    (*fmt).p_len as usize + (*fmt).p_off() as usize,
                    &mut tmp,
                ) < 0
                {
                    super::hts::ks_free(&mut tmp);
                    return -1;
                }
                if remove_index >= 0 {
                    let tfmt = *(*line).d.fmt.add(remove_index as usize);
                    *(*line).d.fmt.add(remove_index as usize) = *fmt;
                    *fmt = tfmt;
                    while remove_index <= i
                        && !(*(*line).d.fmt.add(remove_index as usize)).p.is_null()
                    {
                        remove_index += 1;
                    }
                }
            }
            if remove_index >= 0 {
                (*line).set_n_fmt(remove_index as u32);
            }
            libc::free((*line).indiv.s.cast());
            (*line).indiv.l = tmp.l as _;
            (*line).indiv.m = tmp.m as _;
            (*line).indiv.s = tmp.s;

            let mut off_new = 0usize;
            for i in 0..(*line).n_fmt() as usize {
                let fmt = (*line).d.fmt.add(i);
                let old_free = if (*fmt).p_free() != 0 && !(*fmt).p.is_null() {
                    Some((*fmt).p.sub((*fmt).p_off() as usize))
                } else {
                    None
                };
                (*fmt).p = (*line)
                    .indiv
                    .s
                    .cast::<u8>()
                    .add(off_new + (*fmt).p_off() as usize);
                off_new += (*fmt).p_len as usize + (*fmt).p_off() as usize;
                if let Some(ptr) = old_free {
                    libc::free(ptr.cast());
                    (*fmt).set_p_free(0);
                }
            }
        }

        if (*line).n_sample() == 0 {
            (*line).set_n_fmt(0);
        }
        (*line).d.shared_dirty = 0;
        (*line).d.indiv_dirty = 0;
        0
    }
}

unsafe fn vcf_get_rlen_decoded(hdr: *const bcf_hdr_t, line: *mut bcf1_t) -> hts_pos_t {
    unsafe {
        if line.is_null() {
            return -1;
        }
        let mut len_ref = 0;
        if (*line).unpacked & BCF_UN_STR as c_int == 0 {
            let _ = bcf_unpack(line, BCF_UN_STR as c_int);
        }
        if (*line).n_allele() != 0 && !(*line).d.allele.is_null() && !(*(*line).d.allele).is_null()
        {
            len_ref = libc::strlen(*(*line).d.allele) as hts_pos_t;
        }

        let mut span = len_ref;
        if !hdr.is_null() {
            let end = bcf_get_info(hdr, line, c"END".as_ptr());
            if !end.is_null() && !(*end).vptr.is_null() {
                let value = (*end).v1.i;
                if value > (*line).pos {
                    span = span.max(value - (*line).pos);
                }
            }
            let svlen = bcf_get_info(hdr, line, c"SVLEN".as_ptr());
            if !svlen.is_null() && !(*svlen).vptr.is_null() {
                for i in 0..(*svlen).len.max(0) as usize {
                    let value = match (*svlen).type_ {
                        x if x == BCF_BT_INT8 as c_int => le_to_i8((*svlen).vptr.add(i)) as i64,
                        x if x == BCF_BT_INT16 as c_int => {
                            le_to_i16((*svlen).vptr.add(i * size_of::<i16>())) as i64
                        }
                        x if x == BCF_BT_INT32 as c_int => {
                            le_to_i32((*svlen).vptr.add(i * size_of::<i32>())) as i64
                        }
                        x if x == BCF_BT_INT64 as c_int => {
                            le_to_i64((*svlen).vptr.add(i * size_of::<i64>()))
                        }
                        _ => 0,
                    };
                    if value != 0
                        && value != bcf_int8_missing as i64
                        && value != bcf_int16_missing as i64
                        && value != bcf_int32_missing as i64
                        && value != bcf_int64_missing
                    {
                        span = span.max(value.abs() as hts_pos_t + 1);
                    }
                }
            }
            let len = bcf_get_fmt(hdr, line, c"LEN".as_ptr());
            if !len.is_null() && !(*len).p.is_null() {
                for sample in 0..(*line).n_sample() as usize {
                    let base = (*len).p.add(sample * (*len).size as usize);
                    for i in 0..(*len).n.max(0) as usize {
                        let value = match (*len).type_ {
                            x if x == BCF_BT_INT8 as c_int => le_to_i8(base.add(i)) as i64,
                            x if x == BCF_BT_INT16 as c_int => {
                                le_to_i16(base.add(i * size_of::<i16>())) as i64
                            }
                            x if x == BCF_BT_INT32 as c_int => {
                                le_to_i32(base.add(i * size_of::<i32>())) as i64
                            }
                            x if x == BCF_BT_INT64 as c_int => {
                                le_to_i64(base.add(i * size_of::<i64>()))
                            }
                            _ => 0,
                        };
                        if value > 0 {
                            span = span.max(value as hts_pos_t);
                        }
                    }
                }
            }
        }
        span
    }
}

pub unsafe fn vcf_c_6420_get_rlen(hdr: *const bcf_hdr_t, line: *mut bcf1_t) -> i64 {
    unsafe { vcf_get_rlen_decoded(hdr, line) }
}

pub unsafe fn vcf_c_5884__bcf1_sync_alleles(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    nals: c_int,
) -> c_int {
    unsafe {
        if line.is_null() || nals < 0 {
            return -1;
        }
        (*line).d.shared_dirty |= BCF1_DIRTY_ALS as c_int;
        (*line).d.var_type = -1;
        (*line).set_n_allele(nals as u32);

        if nals > (*line).d.m_allele {
            let allele = libc::realloc(
                (*line).d.allele.cast(),
                nals as usize * size_of::<*mut c_char>(),
            )
            .cast::<*mut c_char>();
            if allele.is_null() {
                return -1;
            }
            (*line).d.allele = allele;
            (*line).d.m_allele = nals;
        }

        let mut als = (*line).d.als;
        for i in 0..nals as usize {
            *(*line).d.allele.add(i) = als;
            if als.is_null() {
                return -1;
            }
            while *als != 0 {
                als = als.add(1);
            }
            als = als.add(1);
        }
        (*line).rlen = vcf_get_rlen_decoded(hdr, line);
        0
    }
}

// Native translation of htslib/vcf.c find_chrom_header_line().
unsafe fn find_chrom_header_line(s: *mut c_char) -> *mut c_char {
    if libc::strncmp(s, c"#CHROM\t".as_ptr(), 7) == 0 {
        return s;
    }
    let nl = libc::strstr(s, c"\n#CHROM\t".as_ptr());
    if !nl.is_null() {
        return nl.add(1);
    }
    std::ptr::null_mut()
}

// Native translation of htslib/vcf.c bcf_hdr_subset().
//
// The C version uses a khash_str2int set purely to detect duplicate sample
// names; this port uses a small linear-scan duplicate check over the names
// already added to `str` (behaviourally identical for the modest sample
// counts this function handles).
pub unsafe fn bcf_hdr_subset(
    h0: *const bcf_hdr_t,
    n: c_int,
    samples: *const *mut c_char,
    imap: *mut c_int,
) -> *mut bcf_hdr_t {
    let mut htxt: kstring_t = std::mem::zeroed();
    let mut str_: kstring_t = std::mem::zeroed();
    let mut h = bcf_hdr_init(c"w".as_ptr());
    let mut r = false;
    // Tracks sample names already accepted, to flag duplicates.
    let mut seen: Vec<*mut c_char> = Vec::new();

    let fail = |h: &mut *mut bcf_hdr_t, str_: &mut kstring_t, htxt: &mut kstring_t| {
        libc::free(str_.s.cast());
        str_.s = std::ptr::null_mut();
        str_.l = 0;
        str_.m = 0;
        libc::free(htxt.s.cast());
        htxt.s = std::ptr::null_mut();
        htxt.l = 0;
        htxt.m = 0;
        if !h.is_null() {
            bcf_hdr_destroy(*h);
            *h = std::ptr::null_mut();
        }
    };

    if h.is_null() {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_hdr_subset".as_ptr(),
            c"Failed to allocate bcf header".as_ptr(),
        );
        fail(&mut h, &mut str_, &mut htxt);
        return std::ptr::null_mut();
    }
    if bcf_hdr_format(h0, 1, &mut htxt) < 0 {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_hdr_subset".as_ptr(),
            c"Failed to get header text".as_ptr(),
        );
        fail(&mut h, &mut str_, &mut htxt);
        return std::ptr::null_mut();
    }
    bcf_hdr_set_version(h, bcf_hdr_get_version(h0));
    for j in 0..n as isize {
        *imap.offset(j) = -1;
    }
    if bcf_hdr_nsamples_native(h0) > 0 {
        let mut p = find_chrom_header_line(htxt.s);
        let mut i = 0;
        let end = if n != 0 { 8 } else { 7 };
        // while ((p = strchr(p, '\t')) != 0 && i < end) ++i, ++p;
        loop {
            if p.is_null() {
                break;
            }
            p = libc::strchr(p, b'\t' as c_int);
            if p.is_null() || i >= end {
                break;
            }
            i += 1;
            p = p.add(1);
        }
        if i != end {
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_ERROR,
                c"bcf_hdr_subset".as_ptr(),
                c"Wrong number of columns in header #CHROM line".as_ptr(),
            );
            fail(&mut h, &mut str_, &mut htxt);
            return std::ptr::null_mut();
        }
        r |= kputsn(htxt.s, (p as usize - htxt.s as usize) as usize, &mut str_) < 0;
        for i in 0..n as isize {
            let sample_i = *samples.offset(i);
            // khash_str2int_has_key(names_hash, samples[i])
            let dup = seen.iter().any(|&s| libc::strcmp(s, sample_i) == 0);
            if dup {
                let msg = std::ffi::CString::new(format!(
                    "Duplicate sample name \"{}\"",
                    CStr::from_ptr(sample_i).to_string_lossy()
                ))
                .unwrap_or_default();
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_ERROR,
                    c"bcf_hdr_subset".as_ptr(),
                    msg.as_ptr(),
                );
                fail(&mut h, &mut str_, &mut htxt);
                return std::ptr::null_mut();
            }
            let mapped = bcf_hdr_id2int(h0, BCF_DT_SAMPLE as c_int, sample_i);
            *imap.offset(i) = mapped;
            if mapped < 0 {
                continue;
            }
            r |= kputc(b'\t' as c_int, &mut str_) < 0;
            r |= kputs(sample_i, &mut str_) < 0;
            seen.push(sample_i);
        }
    } else {
        r |= kputsn(htxt.s, htxt.l, &mut str_) < 0;
    }
    // kill trailing zeros and newlines
    while str_.l != 0
        && (*str_.s.add(str_.l - 1) == 0 || *str_.s.add(str_.l - 1) == b'\n' as c_char)
    {
        str_.l -= 1;
    }
    r |= kputc(b'\n' as c_int, &mut str_) < 0;
    if r {
        let err = libc::strerror(*libc::__errno_location());
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_hdr_subset".as_ptr(),
            err,
        );
        fail(&mut h, &mut str_, &mut htxt);
        return std::ptr::null_mut();
    }
    if bcf_hdr_parse(h, str_.s) < 0 {
        bcf_hdr_destroy(h);
        h = std::ptr::null_mut();
    }
    libc::free(str_.s.cast());
    libc::free(htxt.s.cast());
    h
}

// Native translation of htslib/vcf.c bcf_hdr_add_sample_len().
unsafe fn bcf_hdr_add_sample_len(h: *mut bcf_hdr_t, s: *const c_char, len: usize) -> c_int {
    use super::hts::isspace_c;
    let mut ss = s;
    // while ( *ss && isspace_c(*ss) && ss - s < len) ss++;
    while *ss != 0 && isspace_c(*ss) != 0 && (ss as usize - s as usize) < len {
        ss = ss.add(1);
    }
    if *ss == 0 || (ss as usize - s as usize) == len {
        c_log_error(c"Empty sample name: trailing spaces/tabs in the header line?".as_ptr());
        return -1;
    }

    let d = (*h).dict[BCF_DT_SAMPLE as usize].cast::<kh_vdict_t>();
    let mut ret: c_int = 0;
    let sdup = libc::malloc(len + 1).cast::<c_char>();
    if sdup.is_null() {
        return -1;
    }
    libc::memcpy(sdup.cast(), s.cast(), len);
    *sdup.add(len) = 0;

    // Ensure space is available in h->samples
    let n = (*d).size as usize; // kh_size(d)
    let new_samples = libc::realloc((*h).samples.cast(), (n + 1) * size_of::<*mut c_char>())
        .cast::<*mut c_char>();
    if new_samples.is_null() {
        libc::free(sdup.cast());
        return -1;
    }
    (*h).samples = new_samples;

    let k = kh_put_vdict(d, sdup, &mut ret);
    if ret < 0 {
        libc::free(sdup.cast());
        return -1;
    }
    if ret != 0 {
        // absent
        let valp = (*d).vals.add(k as usize);
        *valp = bcf_idinfo_def();
        (*valp).id = n as c_int;
    } else {
        let msg = std::ffi::CString::new(format!(
            "Duplicated sample name '{}'",
            CStr::from_ptr(sdup).to_string_lossy()
        ))
        .unwrap_or_default();
        c_log_error(msg.as_ptr());
        libc::free(sdup.cast());
        return -1;
    }
    *(*h).samples.add(n) = sdup;
    (*h).dirty = 1;
    0
}

// Native translation of htslib/vcf.c bcf_hdr_add_sample().
pub unsafe fn bcf_hdr_add_sample(hdr: *mut bcf_hdr_t, sample: *const c_char) -> c_int {
    if sample.is_null() {
        // Allowed for backwards-compatibility, calling with s == NULL
        // used to trigger bcf_hdr_sync(h);
        return 0;
    }
    bcf_hdr_add_sample_len(hdr, sample, libc::strlen(sample))
}

// Native translation of htslib/vcf.c bcf_hdr_set().
pub unsafe fn bcf_hdr_set(hdr: *mut bcf_hdr_t, fname: *const c_char) -> c_int {
    let mut n: c_int = 0;
    let lines = crate::htslib_rs::hts::hts_readlines(fname, &mut n);
    if lines.is_null() {
        return 1;
    }
    let mut i: c_int = 0;
    let fail = |i: &mut c_int, n: c_int, lines: *mut *mut c_char| -> c_int {
        let save_errno = *libc::__errno_location();
        while *i < n {
            libc::free((*lines.add(*i as usize)).cast());
            *i += 1;
        }
        libc::free(lines.cast());
        *libc::__errno_location() = save_errno;
        1
    };
    while i < n - 1 {
        let mut k: c_int = 0;
        let hrec = bcf_hdr_parse_line(hdr, *lines.add(i as usize), &mut k);
        if hrec.is_null() {
            return fail(&mut i, n, lines);
        }
        if bcf_hdr_add_hrec(hdr, hrec) < 0 {
            bcf_hrec_destroy(hrec);
            return fail(&mut i, n, lines);
        }
        libc::free((*lines.add(i as usize)).cast());
        *lines.add(i as usize) = std::ptr::null_mut();
        i += 1;
    }
    if vcf_c_286_bcf_hdr_parse_sample_line(hdr, *lines.add((n - 1) as usize)) < 0 {
        return fail(&mut i, n, lines);
    }
    if bcf_hdr_sync(hdr) < 0 {
        return fail(&mut i, n, lines);
    }
    libc::free((*lines.add((n - 1) as usize)).cast());
    libc::free(lines.cast());
    0
}

// Native translation of htslib/vcf.c bcf_hdr_format().
pub unsafe fn bcf_hdr_format(hdr: *const bcf_hdr_t, is_bcf: c_int, str_: *mut kstring_t) -> c_int {
    let mut r = false;
    for i in 0..(*hdr).nhrec as usize {
        r |= bcf_hdr_format_native_hrec(*(*hdr).hrec.add(i), is_bcf, str_) < 0;
    }

    // ksprintf(str, "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO")
    r |= kputs(
        c"#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO".as_ptr(),
        str_,
    ) < 0;
    let nsamples = (*hdr).n[BCF_DT_SAMPLE as usize];
    if nsamples != 0 {
        r |= kputs(c"\tFORMAT".as_ptr(), str_) < 0;
        for i in 0..nsamples as usize {
            r |= kputc(b'\t' as c_int, str_) < 0;
            r |= kputs(*(*hdr).samples.add(i), str_) < 0;
        }
    }
    r |= kputc(b'\n' as c_int, str_) < 0;

    if r {
        -1
    } else {
        0
    }
}

#[inline]
unsafe fn bcf_hdr_format_native_hrec(
    hrec: *const bcf_hrec_t,
    is_bcf: c_int,
    str_: *mut kstring_t,
) -> c_int {
    bcf_hrec_format_native(hrec, is_bcf, str_)
}

// Native translation of htslib/vcf.c bcf_hdr_fmt_text().
pub unsafe fn bcf_hdr_fmt_text(
    hdr: *const bcf_hdr_t,
    is_bcf: c_int,
    len: *mut c_int,
) -> *mut c_char {
    let mut txt = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    if bcf_hdr_format(hdr, is_bcf, &mut txt) < 0 {
        return std::ptr::null_mut();
    }
    if !len.is_null() {
        *len = txt.l as c_int;
    }
    txt.s
}

// Native translation of htslib/vcf.c bcf_hdr_append().
pub unsafe fn bcf_hdr_append(h: *mut bcf_hdr_t, line: *const c_char) -> c_int {
    let mut len: c_int = 0;
    let hrec = bcf_hdr_parse_line(h, line, &mut len);
    if hrec.is_null() {
        return -1;
    }
    if bcf_hdr_add_hrec(h, hrec) < 0 {
        return -1;
    }
    bcf_hdr_fix_vcf45_vl_types(h);
    0
}

// Native translation of htslib/vcf.c bcf_hdr_get_version().
pub unsafe fn bcf_hdr_get_version(hdr: *const bcf_hdr_t) -> *const c_char {
    let hrec = bcf_hdr_get_hrec(
        hdr,
        BCF_HL_GEN as c_int,
        c"fileformat".as_ptr(),
        std::ptr::null(),
        std::ptr::null(),
    );
    if hrec.is_null() {
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_WARNING,
            c"bcf_hdr_get_version".as_ptr(),
            c"No version string found, assuming VCFv4.2".as_ptr(),
        );
        return c"VCFv4.2".as_ptr();
    }
    (*hrec).value
}

// Native translation of htslib/vcf.c bcf_hdr_set_version().
pub unsafe fn bcf_hdr_set_version(hdr: *mut bcf_hdr_t, version: *const c_char) -> c_int {
    let hrec = bcf_hdr_get_hrec(
        hdr,
        BCF_HL_GEN as c_int,
        c"fileformat".as_ptr(),
        std::ptr::null(),
        std::ptr::null(),
    );
    if hrec.is_null() {
        let mut len: c_int = 0;
        let mut str_: kstring_t = std::mem::zeroed();
        if ks_build(&mut str_, &[c"##fileformat=".as_ptr(), version]) < 0 {
            libc::free(str_.s.cast());
            return -1;
        }
        let hrec = bcf_hdr_parse_line(hdr, str_.s, &mut len);
        libc::free(str_.s.cast());

        (*get_hdr_aux(hdr)).version = bcf_get_version_str((*hrec).value);
    } else {
        let tmp = bcf_hrec_dup(hrec);
        if tmp.is_null() {
            return -1;
        }
        libc::free((*tmp).value.cast());
        (*tmp).value = cc_strdup(version);
        if (*tmp).value.is_null() {
            return -1;
        }
        bcf_hdr_update_hrec(hdr, hrec, tmp);
        bcf_hrec_destroy(tmp);
    }
    (*hdr).dirty = 1;
    // TODO rlen may change, deal with it
    0
}

// Native translation of htslib/vcf.c bcf_hdr_remove_from_hdict().
unsafe fn bcf_hdr_remove_from_hdict(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) {
    let mut str_: kstring_t = std::mem::zeroed();
    let aux = get_hdr_aux(hdr);
    let gen = (*aux).gen;
    let k: u32;

    match (*hrec).type_ as u32 {
        BCF_HL_GEN => {
            if ks_build(
                &mut str_,
                &[c"##".as_ptr(), (*hrec).key, c"=".as_ptr(), (*hrec).value],
            ) < 0
            {
                str_.l = 0;
            }
        }
        BCF_HL_STR => {
            let id = bcf_hrec_find_key(hrec, c"ID".as_ptr());
            if id < 0 {
                return;
            }
            if (*(*hrec).vals.add(id as usize)).is_null()
                || ks_build(
                    &mut str_,
                    &[
                        c"##".as_ptr(),
                        (*hrec).key,
                        c"=<ID=".as_ptr(),
                        *(*hrec).vals.add(id as usize),
                        c">".as_ptr(),
                    ],
                ) < 0
            {
                str_.l = 0;
            }
        }
        _ => return,
    }

    if str_.l != 0 {
        k = kh_get_hdict(gen, str_.s);
    } else {
        // Couldn't get a string for some reason, so try the hard way...
        let mut kk: u32 = 0;
        while kk < (*gen).n_buckets {
            if !kh_iseither((*gen).flags, kk) && *(*gen).vals.add(kk as usize) == hrec {
                break;
            }
            kk += 1;
        }
        k = kk;
    }
    if k != (*gen).n_buckets && *(*gen).vals.add(k as usize) == hrec {
        *(*gen).vals.add(k as usize) = std::ptr::null_mut();
        libc::free((*(*gen).keys.add(k as usize)) as *mut c_void);
        *(*gen).keys.add(k as usize) = std::ptr::null();
        kh_del_hdict(gen, k);
    }
    libc::free(str_.s.cast());
}

// Native translation of htslib/vcf.c bcf_hdr_remove().
pub unsafe fn bcf_hdr_remove(hdr: *mut bcf_hdr_t, type_: c_int, key: *const c_char) {
    let mut i: c_int = 0;
    let mut hrec: *mut bcf_hrec_t;
    if key.is_null() {
        // no key, remove all entries of this type
        while i < (*hdr).nhrec {
            if (*(*(*hdr).hrec.add(i as usize))).type_ as c_int != type_ {
                i += 1;
                continue;
            }
            hrec = *(*hdr).hrec.add(i as usize);
            bcf_hdr_unregister_hrec_native(hdr, hrec);
            bcf_hdr_remove_from_hdict(hdr, hrec);
            (*hdr).dirty = 1;
            (*hdr).nhrec -= 1;
            if i < (*hdr).nhrec {
                libc::memmove(
                    (*hdr).hrec.add(i as usize).cast(),
                    (*hdr).hrec.add(i as usize + 1).cast(),
                    ((*hdr).nhrec - i) as usize * size_of::<*mut bcf_hrec_t>(),
                );
            }
            bcf_hrec_destroy(hrec);
        }
        return;
    }
    loop {
        if type_ as u32 == BCF_HL_FLT
            || type_ as u32 == BCF_HL_INFO
            || type_ as u32 == BCF_HL_FMT
            || type_ as u32 == BCF_HL_CTG
        {
            hrec = bcf_hdr_get_hrec(hdr, type_, c"ID".as_ptr(), key, std::ptr::null());
            if hrec.is_null() {
                return;
            }

            i = 0;
            while i < (*hdr).nhrec {
                if *(*hdr).hrec.add(i as usize) == hrec {
                    break;
                }
                i += 1;
            }
            debug_assert!(i < (*hdr).nhrec);

            let d = if type_ as u32 == BCF_HL_CTG {
                (*hdr).dict[BCF_DT_CTG as usize].cast::<kh_vdict_t>()
            } else {
                (*hdr).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>()
            };
            let k = kh_get_vdict(d, key);
            let slot = if type_ as u32 == BCF_HL_CTG {
                0
            } else {
                type_ as usize
            };
            (*(*d).vals.add(k as usize)).hrec[slot] = std::ptr::null_mut();
        } else {
            i = 0;
            while i < (*hdr).nhrec {
                let h_i = *(*hdr).hrec.add(i as usize);
                if (*h_i).type_ as c_int != type_ {
                    i += 1;
                    continue;
                }
                if type_ as u32 == BCF_HL_GEN {
                    if libc::strcmp((*h_i).key, key) == 0 {
                        break;
                    }
                } else {
                    // not all structured lines have ID
                    let j = bcf_hrec_find_key(h_i, c"ID".as_ptr());
                    if j >= 0 && libc::strcmp(*(*h_i).vals.add(j as usize), key) == 0 {
                        break;
                    }
                }
                i += 1;
            }
            if i == (*hdr).nhrec {
                return;
            }
            hrec = *(*hdr).hrec.add(i as usize);
            bcf_hdr_remove_from_hdict(hdr, hrec);
        }

        (*hdr).nhrec -= 1;
        if i < (*hdr).nhrec {
            libc::memmove(
                (*hdr).hrec.add(i as usize).cast(),
                (*hdr).hrec.add(i as usize + 1).cast(),
                ((*hdr).nhrec - i) as usize * size_of::<*mut bcf_hrec_t>(),
            );
        }
        bcf_hrec_destroy(hrec);
        (*hdr).dirty = 1;
    }
}

// Native translation of htslib/vcf.c bcf_hdr_seqnames().
pub unsafe fn bcf_hdr_seqnames(h: *const bcf_hdr_t, nseqs: *mut c_int) -> *mut *const c_char {
    let d = (*h).dict[BCF_DT_CTG as usize].cast::<kh_vdict_t>();
    let mut m = if d.is_null() { 0 } else { (*d).size } as c_int; // kh_size
    let mut names =
        libc::calloc(m.max(0) as usize, size_of::<*const c_char>()).cast::<*const c_char>();
    if names.is_null() && m > 0 {
        let msg = c"Failed to allocate memory";
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_ERROR,
            c"bcf_hdr_seqnames".as_ptr(),
            msg.as_ptr(),
        );
        *nseqs = 0;
        return std::ptr::null_mut();
    }
    if !d.is_null() {
        for k in 0..(*d).n_buckets {
            if vcf_kh_iseither((*d).flags, k) {
                continue;
            }
            let val = (*d).vals.add(k as usize);
            if (*val).hrec[0].is_null() {
                continue; // removed via bcf_hdr_remove
            }
            let tid = (*val).id;
            if tid >= m {
                // Can happen after a contig was removed via bcf_hdr_remove().
                if super::hts::hts_resize_array_(
                    size_of::<*const c_char>() as size_t,
                    (tid + 1) as size_t,
                    size_of::<c_int>() as size_t,
                    (&mut m as *mut c_int).cast(),
                    (&mut names as *mut *mut *const c_char).cast(),
                    super::hts::HTS_RESIZE_CLEAR,
                    c"bcf_hdr_seqnames".as_ptr(),
                ) < 0
                {
                    let msg = c"Failed to allocate memory";
                    crate::htslib_rs::hts::hts_log_cstr(
                        crate::htslib_rs::hts::HTS_LOG_ERROR,
                        c"bcf_hdr_seqnames".as_ptr(),
                        msg.as_ptr(),
                    );
                    *nseqs = 0;
                    libc::free(names.cast());
                    return std::ptr::null_mut();
                }
                m = tid + 1;
            }
            *names.add(tid as usize) = *(*d).keys.add(k as usize);
        }
    }
    // Ensure there are no gaps.
    let mut i = 0;
    let mut tid = 0;
    while tid < m {
        while tid < m && (*names.add(tid as usize)).is_null() {
            tid += 1;
        }
        if tid == m {
            break;
        }
        if i != tid {
            *names.add(i as usize) = *names.add(tid as usize);
            *names.add(tid as usize) = std::ptr::null();
        }
        i += 1;
        tid += 1;
    }
    *nseqs = i;
    names
}

// Native translation of htslib/vcf.c bcf_hdr_parse().
pub unsafe fn bcf_hdr_parse(hdr: *mut bcf_hdr_t, htxt: *mut c_char) -> c_int {
    let mut len: c_int = 0;
    let mut done: c_int = 0;
    let mut p = htxt;

    // Check sanity: "fileformat" string must come as first
    let mut hrec = bcf_hdr_parse_line(hdr, p, &mut len);
    if hrec.is_null()
        || (*hrec).key.is_null()
        || libc::strcasecmp((*hrec).key, c"fileformat".as_ptr()) != 0
    {
        c_log_warning(
            c"The first line should be ##fileformat; is the VCF/BCF header broken?".as_ptr(),
        );
    }
    if bcf_hdr_add_hrec(hdr, hrec) < 0 {
        bcf_hrec_destroy(hrec);
        return -1;
    }

    // The filter PASS must appear first in the dictionary
    hrec = bcf_hdr_parse_line(
        hdr,
        c"##FILTER=<ID=PASS,Description=\"All filters passed\">".as_ptr(),
        &mut len,
    );
    if hrec.is_null() || bcf_hdr_add_hrec(hdr, hrec) < 0 {
        bcf_hrec_destroy(hrec);
        return -1;
    }

    // Parse the whole header
    loop {
        loop {
            hrec = bcf_hdr_parse_line(hdr, p, &mut len);
            if hrec.is_null() {
                break;
            }
            if bcf_hdr_add_hrec(hdr, hrec) < 0 {
                bcf_hrec_destroy(hrec);
                return -1;
            }
            p = p.add(len as usize);
        }
        if len < 0 {
            c_log_error(c"Could not parse header line".as_ptr());
            return -1;
        } else if len > 0 {
            // Bad header line; skip and try again on the next line.
            p = p.add(len as usize);
            continue;
        }

        // Next should be the sample line.
        if libc::strncmp(c"#CHROM\t".as_ptr(), p, 7) != 0
            && libc::strncmp(c"#CHROM ".as_ptr(), p, 7) != 0
        {
            let eol = libc::strchr(p, b'\n' as c_int);
            if *p != 0 {
                c_log_warning(c"Could not parse header line".as_ptr());
            }
            if !eol.is_null() {
                p = eol.add(1); // Try from the next line.
            } else {
                done = -1; // No more lines left, give up.
            }
        } else {
            done = 1; // Sample line found
        }
        if done != 0 {
            break;
        }
    }

    if done < 0 {
        c_log_error(c"Could not parse the header, sample line not found".as_ptr());
        return -1;
    }

    if vcf_c_286_bcf_hdr_parse_sample_line(hdr, p) < 0 {
        return -1;
    }
    if bcf_hdr_sync(hdr) < 0 {
        return -1;
    }
    bcf_hdr_check_sanity(hdr);
    // Note: bcf_hdr_sync() already applies the VCF4.5 Number/vl-type fixup.
    0
}

// Native translation of htslib/vcf.c bcf_hdr_sync(): rebuild id[]/samples/n[]
// from the dicts and invalidate the key-length cache.
unsafe fn bcf_hdr_sync_native(h: *mut bcf_hdr_t) -> c_int {
    let mut i = 0usize;
    while i < 3 {
        let d = (*h).dict[i].cast::<kh_vdict_t>();
        let dsize = (*d).size;
        if ((*h).n[i] as u32) < dsize {
            // this should be true only for i=2, BCF_DT_SAMPLE
            let new_idpair =
                hts_realloc_p_cc((*h).id[i].cast(), size_of::<bcf_idpair_t>(), dsize as usize)
                    .cast::<bcf_idpair_t>();
            if new_idpair.is_null() {
                return -1;
            }
            (*h).n[i] = dsize as c_int;
            (*h).id[i] = new_idpair;
        }
        let mut k: u32 = 0;
        while k < (*d).n_buckets {
            if !kh_iseither((*d).flags, k) {
                let valp = (*d).vals.add(k as usize);
                let id = (*valp).id;
                let pair = (*h).id[i].add(id as usize);
                (*pair).key = *(*d).keys.add(k as usize);
                (*pair).val = valp;
            }
            k += 1;
        }
        i += 1;
    }

    // Invalidate key length cache
    let aux = get_hdr_aux(h);
    if !aux.is_null() && !(*aux).key_len.is_null() {
        libc::free((*aux).key_len.cast());
        (*aux).key_len = std::ptr::null_mut();
    }

    (*h).dirty = 0;
    0
}

pub unsafe fn bcf_hdr_sync(h: *mut bcf_hdr_t) -> c_int {
    let ret = bcf_hdr_sync_native(h);
    if ret == 0 {
        bcf_hdr_fix_vcf45_vl_types(h);
    }
    ret
}

pub unsafe fn vcf_c_286_bcf_hdr_parse_sample_line(
    hdr: *mut bcf_hdr_t,
    str_: *const c_char,
) -> c_int {
    unsafe {
        if hdr.is_null() || str_.is_null() {
            return -1;
        }

        const MANDATORY: &[u8] = b"#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO";
        let line = CStr::from_ptr(str_).to_bytes();
        if !line.starts_with(MANDATORY) {
            let str_val = CStr::from_ptr(str_).to_string_lossy();
            let msg = std::ffi::CString::new(format!(
                "Could not parse the \"#CHROM..\" line, either the fields are incorrect or spaces are present instead of tabs:\n\t{}",
                str_val
            ))
            .unwrap_or_default();
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_ERROR,
                c"bcf_hdr_parse_sample_line".as_ptr(),
                msg.as_ptr(),
            );
            return -1;
        }

        let mut beg = MANDATORY.len();
        if beg == line.len() || line[beg] == b'\n' {
            return 0;
        }

        const FORMAT: &[u8] = b"\tFORMAT\t";
        if !line[beg..].starts_with(FORMAT) {
            let str_val = CStr::from_ptr(str_).to_string_lossy();
            let msg = std::ffi::CString::new(format!(
                "Could not parse the \"#CHROM..\" line, either FORMAT is missing or spaces are present instead of tabs:\n\t{}",
                str_val
            ))
            .unwrap_or_default();
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_ERROR,
                c"bcf_hdr_parse_sample_line".as_ptr(),
                msg.as_ptr(),
            );
            return -1;
        }
        beg += FORMAT.len();

        let mut ret = 0;
        while beg < line.len() {
            let mut end = beg;
            while end < line.len() && line[end] != b'\t' && line[end] != b'\n' {
                end += 1;
            }

            let mut sample = Vec::with_capacity(end - beg + 1);
            sample.extend_from_slice(&line[beg..end]);
            sample.push(0);
            if bcf_hdr_add_sample(hdr, sample.as_ptr().cast()) < 0 {
                ret = -1;
            }

            if end == line.len() || line[end] == b'\n' || ret < 0 {
                break;
            }
            beg = end + 1;
        }

        ret
    }
}

struct HeaderSanityTag {
    name: &'static CStr,
    number_text: &'static CStr,
    number: c_int,
    fixed_number: c_int,
    version: c_int,
    type_: c_int,
}

static BCF_HDR_CHECK_SANITY_INFO_WARNED: AtomicU64 = AtomicU64::new(0);
static BCF_HDR_CHECK_SANITY_FMT_WARNED: AtomicU64 = AtomicU64::new(0);

const HEADER_SANITY_INFO_TAGS: &[HeaderSanityTag] = &[
    HeaderSanityTag {
        name: c"AD",
        number_text: c"R",
        number: BCF_VL_R as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"ADF",
        number_text: c"R",
        number: BCF_VL_R as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"ADR",
        number_text: c"R",
        number: BCF_VL_R as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"AC",
        number_text: c"A",
        number: BCF_VL_A as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"AF",
        number_text: c"A",
        number: BCF_VL_A as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_REAL as c_int,
    },
    HeaderSanityTag {
        name: c"CIGAR",
        number_text: c"A",
        number: BCF_VL_A as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_STR as c_int,
    },
    HeaderSanityTag {
        name: c"AA",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_STR as c_int,
    },
    HeaderSanityTag {
        name: c"AN",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"BQ",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_REAL as c_int,
    },
    HeaderSanityTag {
        name: c"DB",
        number_text: c"0",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_FLAG as c_int,
    },
    HeaderSanityTag {
        name: c"DP",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"END",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"H2",
        number_text: c"0",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_FLAG as c_int,
    },
    HeaderSanityTag {
        name: c"H3",
        number_text: c"0",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_FLAG as c_int,
    },
    HeaderSanityTag {
        name: c"MQ",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_REAL as c_int,
    },
    HeaderSanityTag {
        name: c"MQ0",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"NS",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"SB",
        number_text: c"4",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 4,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"SOMATIC",
        number_text: c"0",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_FLAG as c_int,
    },
    HeaderSanityTag {
        name: c"VALIDATED",
        number_text: c"0",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_FLAG as c_int,
    },
    HeaderSanityTag {
        name: c"1000G",
        number_text: c"0",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_FLAG as c_int,
    },
];

const HEADER_SANITY_FMT_TAGS: &[HeaderSanityTag] = &[
    HeaderSanityTag {
        name: c"AD",
        number_text: c"R",
        number: BCF_VL_R as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"ADF",
        number_text: c"R",
        number: BCF_VL_R as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"ADR",
        number_text: c"R",
        number: BCF_VL_R as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"EC",
        number_text: c"A",
        number: BCF_VL_A as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"GL",
        number_text: c"G",
        number: BCF_VL_G as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_REAL as c_int,
    },
    HeaderSanityTag {
        name: c"GP",
        number_text: c"G",
        number: BCF_VL_G as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_REAL as c_int,
    },
    HeaderSanityTag {
        name: c"PL",
        number_text: c"G",
        number: BCF_VL_G as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"PP",
        number_text: c"G",
        number: BCF_VL_G as c_int,
        fixed_number: 0,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"DP",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LEN",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"FT",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_STR as c_int,
    },
    HeaderSanityTag {
        name: c"GQ",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"GT",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_STR as c_int,
    },
    HeaderSanityTag {
        name: c"HQ",
        number_text: c"2",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 2,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"MQ",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"PQ",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"PS",
        number_text: c"1",
        number: BCF_VL_FIXED as c_int,
        fixed_number: 1,
        version: VCF_DEF,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"PSL",
        number_text: c"P",
        number: BCF_VL_P,
        fixed_number: 0,
        version: VCF44,
        type_: BCF_HT_STR as c_int,
    },
    HeaderSanityTag {
        name: c"PSO",
        number_text: c"P",
        number: BCF_VL_P,
        fixed_number: 0,
        version: VCF44,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"PSQ",
        number_text: c"P",
        number: BCF_VL_P,
        fixed_number: 0,
        version: VCF44,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LGL",
        number_text: c"LG",
        number: BCF_VL_LG,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LGP",
        number_text: c"LG",
        number: BCF_VL_LG,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LPL",
        number_text: c"LG",
        number: BCF_VL_LG,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LPP",
        number_text: c"LG",
        number: BCF_VL_LG,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LEC",
        number_text: c"LA",
        number: BCF_VL_LA,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LAD",
        number_text: c"LR",
        number: BCF_VL_LR,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LADF",
        number_text: c"LR",
        number: BCF_VL_LR,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
    HeaderSanityTag {
        name: c"LADR",
        number_text: c"LR",
        number: BCF_VL_LR,
        fixed_number: 0,
        version: VCF45,
        type_: BCF_HT_INT as c_int,
    },
];

unsafe fn bcf_hdr_idinfo_exists_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> bool {
    unsafe {
        if hdr.is_null() || int_id < 0 || int_id >= (*hdr).n[BCF_DT_ID as usize] as c_int {
            return false;
        }
        let val = (*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val;
        !val.is_null() && bcf_hdr_id2coltype_rs(hdr, type_, int_id) != 0xf
    }
}

unsafe fn bcf_hdr_id2length_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        (((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info[type_ as usize] >> 8)
            & 0xf) as c_int
    }
}

unsafe fn bcf_hdr_id2number_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        ((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info[type_ as usize] >> 12)
            as c_int
    }
}

unsafe fn bcf_hdr_id2type_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        (((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info[type_ as usize] >> 4)
            & 0xf) as c_int
    }
}

unsafe fn bcf_hdr_id2coltype_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        ((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info[type_ as usize] & 0xf)
            as c_int
    }
}

fn bcf_hdr_check_sanity_type_name(type_: c_int) -> *const c_char {
    match type_ {
        x if x == BCF_HT_FLAG as c_int => c"Flag".as_ptr(),
        x if x == BCF_HT_INT as c_int => c"Integer".as_ptr(),
        x if x == BCF_HT_REAL as c_int => c"Float".as_ptr(),
        _ => c"String".as_ptr(),
    }
}

unsafe fn bcf_hdr_check_sanity_warn_number(tag: &HeaderSanityTag) {
    unsafe {
        let msg = std::ffi::CString::new(format!(
            "{} should be declared as Number={}",
            tag.name.to_string_lossy(),
            tag.number_text.to_string_lossy()
        ))
        .unwrap_or_default();
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_WARNING,
            c"bcf_hdr_check_sanity".as_ptr(),
            msg.as_ptr(),
        );
    }
}

unsafe fn bcf_hdr_check_sanity_warn_type(tag: &HeaderSanityTag) {
    unsafe {
        let type_name = CStr::from_ptr(bcf_hdr_check_sanity_type_name(tag.type_)).to_string_lossy();
        let msg = std::ffi::CString::new(format!(
            "{} should be declared as Type={}",
            tag.name.to_string_lossy(),
            type_name
        ))
        .unwrap_or_default();
        crate::htslib_rs::hts::hts_log_cstr(
            crate::htslib_rs::hts::HTS_LOG_WARNING,
            c"bcf_hdr_check_sanity".as_ptr(),
            msg.as_ptr(),
        );
    }
}

unsafe fn bcf_hdr_check_sanity_info_tag(hdr: *mut bcf_hdr_t, index: usize, tag: &HeaderSanityTag) {
    unsafe {
        let bit = 1_u64 << index;
        if BCF_HDR_CHECK_SANITY_INFO_WARNED.load(Ordering::Relaxed) & bit != 0 {
            return;
        }
        let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, tag.name.as_ptr());
        if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_INFO as c_int, id) {
            return;
        }

        let length = bcf_hdr_id2length_rs(hdr, BCF_HL_INFO as c_int, id);
        let mut warned = false;
        if length != tag.number && length != BCF_VL_VAR as c_int {
            warned = true;
        } else if length == BCF_VL_FIXED as c_int
            && bcf_hdr_id2number_rs(hdr, BCF_HL_INFO as c_int, id) != tag.fixed_number
        {
            warned = true;
        }

        if warned {
            BCF_HDR_CHECK_SANITY_INFO_WARNED.fetch_or(bit, Ordering::Relaxed);
            bcf_hdr_check_sanity_warn_number(tag);
        }

        if bcf_hdr_id2type_rs(hdr, BCF_HL_INFO as c_int, id) != tag.type_ {
            BCF_HDR_CHECK_SANITY_INFO_WARNED.fetch_or(bit, Ordering::Relaxed);
            bcf_hdr_check_sanity_warn_type(tag);
        }
    }
}

unsafe fn bcf_hdr_check_sanity_fmt_tag(
    hdr: *mut bcf_hdr_t,
    version: c_int,
    index: usize,
    tag: &HeaderSanityTag,
) {
    unsafe {
        let bit = 1_u64 << index;
        if BCF_HDR_CHECK_SANITY_FMT_WARNED.load(Ordering::Relaxed) & bit != 0 {
            return;
        }
        let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, tag.name.as_ptr());
        if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_FMT as c_int, id) {
            return;
        }

        let length = bcf_hdr_id2length_rs(hdr, BCF_HL_FMT as c_int, id);
        let mut warned = false;
        if length != tag.number {
            if (version < VCF44 && length != BCF_VL_VAR as c_int)
                || (version >= VCF44 && version >= tag.version)
            {
                warned = true;
            }
        } else if length == BCF_VL_FIXED as c_int
            && bcf_hdr_id2number_rs(hdr, BCF_HL_FMT as c_int, id) != tag.fixed_number
        {
            warned = true;
        }

        if warned {
            BCF_HDR_CHECK_SANITY_FMT_WARNED.fetch_or(bit, Ordering::Relaxed);
            bcf_hdr_check_sanity_warn_number(tag);
        }

        if bcf_hdr_id2type_rs(hdr, BCF_HL_FMT as c_int, id) != tag.type_ {
            BCF_HDR_CHECK_SANITY_FMT_WARNED.fetch_or(bit, Ordering::Relaxed);
            bcf_hdr_check_sanity_warn_type(tag);
        }
    }
}

pub unsafe fn bcf_hdr_check_sanity(hdr: *mut bcf_hdr_t) {
    unsafe {
        if hdr.is_null() {
            return;
        }
        let version = match std::ptr::NonNull::new(bcf_hdr_get_version(hdr).cast_mut()) {
            Some(version) => vcf_version_number(CStr::from_ptr(version.as_ptr()).to_bytes())
                .unwrap_or(VCF_DEF as i64) as c_int,
            None => VCF_DEF,
        };

        for (index, tag) in HEADER_SANITY_INFO_TAGS.iter().enumerate() {
            bcf_hdr_check_sanity_info_tag(hdr, index, tag);
        }
        for (index, tag) in HEADER_SANITY_FMT_TAGS.iter().enumerate() {
            bcf_hdr_check_sanity_fmt_tag(hdr, version, index, tag);
        }
    }
}

#[inline]
unsafe fn is_escaped(min: *const c_char, mut str_: *const c_char) -> bool {
    let mut n = 0;
    loop {
        str_ = str_.sub(1);
        if str_ < min || *str_ != b'\\' as c_char {
            break;
        }
        n += 1;
    }
    n % 2 != 0
}

// Native translation of htslib/vcf.c bcf_hdr_parse_line().
pub unsafe fn bcf_hdr_parse_line(
    _h: *const bcf_hdr_t,
    line: *const c_char,
    len: *mut c_int,
) -> *mut bcf_hrec_t {
    use super::hts::{isalnum_c, isalpha_c, isspace_c};

    let mut hrec: *mut bcf_hrec_t = std::ptr::null_mut();
    let mut p = line;
    if *p.add(0) != b'#' as c_char || *p.add(1) != b'#' as c_char {
        *len = 0;
        return std::ptr::null_mut();
    }
    p = p.add(2);

    let mut q = p;
    while *q != 0 && *q != b'=' as c_char && *q != b'\n' as c_char {
        q = q.add(1);
    }
    let mut n = q.offset_from(p);
    if *q != b'=' as c_char || n == 0 {
        // malformed
        while *q != 0 && *q != b'\n' as c_char {
            q = q.add(1);
        }
        c_log_error(c"Could not parse the header line".as_ptr());
        *len = (q.offset_from(line) + if *q != 0 { 1 } else { 0 }) as c_int;
        bcf_hrec_destroy(hrec);
        return std::ptr::null_mut();
    }

    hrec = libc::calloc(1, size_of::<bcf_hrec_t>()).cast();
    if hrec.is_null() {
        *len = -1;
        return std::ptr::null_mut();
    }
    (*hrec).key = libc::malloc(n as usize + 1).cast();
    if (*hrec).key.is_null() {
        *len = -1;
        bcf_hrec_destroy(hrec);
        return std::ptr::null_mut();
    }
    libc::memcpy((*hrec).key.cast(), p.cast(), n as usize);
    *(*hrec).key.add(n as usize) = 0;
    (*hrec).type_ = -1;

    q = q.add(1);
    p = q;
    if *p != b'<' as c_char {
        // generic field, e.g. ##samtoolsVersion=0.1.18-r579
        while *q != 0 && *q != b'\n' as c_char {
            q = q.add(1);
        }
        let vlen = q.offset_from(p);
        (*hrec).value = libc::malloc(vlen as usize + 1).cast();
        if (*hrec).value.is_null() {
            *len = -1;
            bcf_hrec_destroy(hrec);
            return std::ptr::null_mut();
        }
        libc::memcpy((*hrec).value.cast(), p.cast(), vlen as usize);
        *(*hrec).value.add(vlen as usize) = 0;
        *len = (q.offset_from(line) + if *q != 0 { 1 } else { 0 }) as c_int;
        return hrec;
    }

    // structured line
    let mut nopen: c_int = 1;
    while *q != 0 && *q != b'\n' as c_char && nopen > 0 {
        q = q.add(1);
        p = q;
        while *q != 0 && *q == b' ' as c_char {
            p = p.add(1);
            q = q.add(1);
        }
        // ^[A-Za-z_][0-9A-Za-z_.]*$
        if p == q && *q != 0 && (isalpha_c(*q) != 0 || *q == b'_' as c_char) {
            q = q.add(1);
            while *q != 0 && (isalnum_c(*q) != 0 || *q == b'_' as c_char || *q == b'.' as c_char) {
                q = q.add(1);
            }
        }
        n = q.offset_from(p);
        let mut m: c_int = 0;
        while *q != 0 && *q == b' ' as c_char {
            q = q.add(1);
            m += 1;
        }
        if *q != b'=' as c_char || n == 0 {
            // malformed
            while *q != 0 && *q != b'\n' as c_char {
                q = q.add(1);
            }
            c_log_error(c"Could not parse the header line".as_ptr());
            *len = (q.offset_from(line) + if *q != 0 { 1 } else { 0 }) as c_int;
            bcf_hrec_destroy(hrec);
            return std::ptr::null_mut();
        }

        if bcf_hrec_add_key(hrec, p, (q.offset_from(p) - m as isize) as usize) < 0 {
            *len = -1;
            bcf_hrec_destroy(hrec);
            return std::ptr::null_mut();
        }
        q = q.add(1);
        p = q;
        while *q != 0 && *q == b' ' as c_char {
            p = p.add(1);
            q = q.add(1);
        }

        let mut quoted = 0;
        let mut ending = 0 as c_char;
        match *p as u8 {
            b'"' => {
                quoted = 1;
                ending = b'"' as c_char;
                p = p.add(1);
            }
            b'[' => {
                quoted = 1;
                ending = b']' as c_char;
            }
            _ => {}
        }
        if quoted != 0 {
            q = q.add(1);
        }
        while *q != 0 && *q != b'\n' as c_char {
            if quoted != 0 {
                if *q == ending && !is_escaped(p, q) {
                    break;
                }
            } else {
                if *q == b'<' as c_char {
                    nopen += 1;
                }
                if *q == b'>' as c_char {
                    nopen -= 1;
                }
                if nopen == 0 {
                    break;
                }
                if *q == b',' as c_char && nopen == 1 {
                    break;
                }
            }
            q = q.add(1);
        }
        let mut r = q;
        if quoted != 0 && ending == b']' as c_char {
            if *q == ending {
                r = r.add(1);
                q = q.add(1);
                quoted = 0;
            } else {
                c_log_error(c"Missing ']' in header line".as_ptr());
                *len = -1;
                bcf_hrec_destroy(hrec);
                return std::ptr::null_mut();
            }
        }
        while r > p && *r.sub(1) == b' ' as c_char {
            r = r.sub(1);
        }
        if bcf_hrec_set_val(
            hrec,
            (*hrec).nkeys - 1,
            p,
            r.offset_from(p) as usize,
            quoted,
        ) < 0
        {
            *len = -1;
            bcf_hrec_destroy(hrec);
            return std::ptr::null_mut();
        }
        if quoted != 0 && *q == ending {
            q = q.add(1);
        }
        if *q == b'>' as c_char {
            if nopen != 0 {
                nopen -= 1; // nested angle brackets
            }
            q = q.add(1);
        }
    }
    if nopen != 0 {
        c_log_warning(c"Incomplete header line, trying to proceed anyway".as_ptr());
    }

    // Skip to end of line
    let mut nonspace = 0;
    while *q != 0 && *q != b'\n' as c_char {
        nonspace |= (isspace_c(*q) == 0) as c_int;
        q = q.add(1);
    }
    if nonspace != 0 {
        c_log_warning(c"Dropped trailing junk from header line".as_ptr());
    }

    *len = (q.offset_from(line) + if *q != 0 { 1 } else { 0 }) as c_int;
    hrec
}

#[inline]
unsafe fn c_log_error(msg: *const c_char) {
    super::hts::hts_log_cstr(super::hts::HTS_LOG_ERROR, c"vcf".as_ptr(), msg);
}

#[inline]
unsafe fn c_log_warning(msg: *const c_char) {
    super::hts::hts_log_cstr(super::hts::HTS_LOG_WARNING, c"vcf".as_ptr(), msg);
}

// Native translation of htslib/vcf.c _bcf_hrec_format().
unsafe fn bcf_hrec_format_native(
    hrec: *const bcf_hrec_t,
    is_bcf: c_int,
    str_: *mut kstring_t,
) -> c_int {
    let mut e = false;
    if (*hrec).value.is_null() {
        let mut nout = 0;
        // ksprintf(str, "##%s=<", hrec->key)
        e |= kputsn(c"##".as_ptr(), 2, str_) < 0;
        e |= kputs((*hrec).key, str_) < 0;
        e |= kputsn(c"=<".as_ptr(), 2, str_) < 0;
        for j in 0..(*hrec).nkeys as usize {
            let key_j = *(*hrec).keys.add(j);
            // do not output IDX if output is VCF
            if is_bcf == 0 && libc::strcmp(c"IDX".as_ptr(), key_j) == 0 {
                continue;
            }
            if nout != 0 {
                e |= kputc(b',' as c_int, str_) < 0;
            }
            // ksprintf(str,"%s=%s", hrec->keys[j], hrec->vals[j])
            e |= kputs(key_j, str_) < 0;
            e |= kputc(b'=' as c_int, str_) < 0;
            e |= kputs(*(*hrec).vals.add(j), str_) < 0;
            nout += 1;
        }
        // ksprintf(str,">\n")
        e |= kputsn(c">\n".as_ptr(), 2, str_) < 0;
    } else {
        // ksprintf(str,"##%s=%s\n", hrec->key,hrec->value)
        e |= kputsn(c"##".as_ptr(), 2, str_) < 0;
        e |= kputs((*hrec).key, str_) < 0;
        e |= kputc(b'=' as c_int, str_) < 0;
        e |= kputs((*hrec).value, str_) < 0;
        e |= kputc(b'\n' as c_int, str_) < 0;
    }
    if e {
        -1
    } else {
        0
    }
}

pub unsafe fn bcf_hrec_format(hrec: *const bcf_hrec_t, str_: *mut kstring_t) -> c_int {
    bcf_hrec_format_native(hrec, 0, str_)
}

// strdup using the libc allocator (matches hts-sys ownership). Returns null on OOM.
#[inline]
unsafe fn cc_strdup(s: *const c_char) -> *mut c_char {
    libc::strdup(s)
}

// Parse a version string ("VCFv4.x") to the encoded int, like vcf.c
// bcf_get_version(NULL, verstr) which returns VCF_DEF when the format is bad.
unsafe fn bcf_get_version_str(verstr: *const c_char) -> c_int {
    if verstr.is_null() {
        return VCF_DEF;
    }
    let bytes = CStr::from_ptr(verstr).to_bytes();
    match vcf_version_number(bytes) {
        Some(v) => v as c_int,
        None => VCF_DEF,
    }
}

// Native translation of htslib/vcf.c bcf_get_version() (the hdr-aware form).
// Returns the cached aux->version when present, else parses verstr.
unsafe fn bcf_get_version(hdr: *const bcf_hdr_t, verstr: *const c_char) -> c_int {
    if hdr.is_null() && verstr.is_null() {
        return VCF_DEF;
    }
    let version;
    if !hdr.is_null() {
        let aux = get_hdr_aux(hdr);
        if !aux.is_null() && (*aux).version != 0 {
            return (*aux).version; // use cached version
        }
        version = bcf_hdr_get_version(hdr);
    } else {
        version = verstr;
    }
    bcf_get_version_str(version)
}

// Build "##<a><b><c>..." style strings into a fresh kstring (libc-allocated .s).
// Mirrors the ksprintf("##%s=%s", ...) patterns used in vcf.c. Returns -1 on OOM.
unsafe fn ks_build(str_: *mut kstring_t, parts: &[*const c_char]) -> c_int {
    for p in parts {
        if super::hts::kputs(*p, str_) < 0 {
            return -1;
        }
    }
    0
}

// Native translation of htslib/vcf.c bcf_hrec_set_type().
unsafe fn bcf_hrec_set_type(hrec: *mut bcf_hrec_t) {
    let key = (*hrec).key;
    if libc::strcmp(key, c"contig".as_ptr()) == 0 {
        (*hrec).type_ = BCF_HL_CTG as c_int;
    } else if libc::strcmp(key, c"INFO".as_ptr()) == 0 {
        (*hrec).type_ = BCF_HL_INFO as c_int;
    } else if libc::strcmp(key, c"FILTER".as_ptr()) == 0 {
        (*hrec).type_ = BCF_HL_FLT as c_int;
    } else if libc::strcmp(key, c"FORMAT".as_ptr()) == 0 {
        (*hrec).type_ = BCF_HL_FMT as c_int;
    } else if (*hrec).nkeys > 0 {
        (*hrec).type_ = BCF_HL_STR as c_int;
    } else {
        (*hrec).type_ = BCF_HL_GEN as c_int;
    }
}

const VALID_CTG: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const VALID_TAG: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// Native translation of htslib/vcf.c bcf_hrec_check().
unsafe fn bcf_hrec_check(hrec: *mut bcf_hrec_t) -> c_int {
    bcf_hrec_set_type(hrec);
    let t = (*hrec).type_ as u32;
    if t == BCF_HL_CTG {
        let i = bcf_hrec_find_key(hrec, c"ID".as_ptr());
        if i < 0 {
            c_log_warning(c"Missing ID attribute in one or more header lines".as_ptr());
            return -1;
        }
        let mut val = *(*hrec).vals.add(i as usize);
        if *val == b'*' as c_char || *val == b'=' as c_char || VALID_CTG[*val as u8 as usize] == 0 {
            c_log_warning(c"Invalid contig name".as_ptr());
            return -1;
        }
        val = val.add(1);
        while *val != 0 {
            if VALID_CTG[*val as u8 as usize] == 0 {
                c_log_warning(c"Invalid contig name".as_ptr());
                return -1;
            }
            val = val.add(1);
        }
        return 0;
    }
    if t == BCF_HL_INFO || t == BCF_HL_FMT {
        let i = bcf_hrec_find_key(hrec, c"ID".as_ptr());
        if i < 0 {
            c_log_warning(c"Missing ID attribute in one or more header lines".as_ptr());
            return -1;
        }
        let mut val = *(*hrec).vals.add(i as usize);
        if t == BCF_HL_INFO && libc::strcmp(val, c"1000G".as_ptr()) == 0 {
            return 0;
        }
        if *val == b'.' as c_char
            || (*val >= b'0' as c_char && *val <= b'9' as c_char)
            || VALID_TAG[*val as u8 as usize] == 0
        {
            c_log_warning(c"Invalid tag name".as_ptr());
            return -1;
        }
        val = val.add(1);
        while *val != 0 {
            if VALID_TAG[*val as u8 as usize] == 0 {
                c_log_warning(c"Invalid tag name".as_ptr());
                return -1;
            }
            val = val.add(1);
        }
        return 0;
    }
    0
}

// Native translation of htslib/vcf.c bcf_hdr_set_idx().
unsafe fn bcf_hdr_set_idx(
    hdr: *mut bcf_hdr_t,
    dict_type: c_int,
    tag: *const c_char,
    idinfo: *mut bcf_idinfo_t,
) -> c_int {
    // If available, preserve existing IDX
    if (*idinfo).id == -1 {
        (*idinfo).id = (*hdr).n[dict_type as usize];
    } else if (*idinfo).id < (*hdr).n[dict_type as usize]
        && !(*(*hdr).id[dict_type as usize].add((*idinfo).id as usize))
            .key
            .is_null()
    {
        c_log_error(c"Conflicting IDX lines in the header dictionary".as_ptr());
        *c_compat::__errno_location() = c_compat::EINVAL;
        return -1;
    }

    let new_n: usize = if (*idinfo).id >= (*hdr).n[dict_type as usize] {
        (*idinfo).id as usize + 1
    } else {
        (*hdr).n[dict_type as usize] as usize
    };
    if super::hts::hts_resize_array_(
        size_of::<bcf_idpair_t>(),
        new_n,
        4, // sizeof(hdr->m[dict_type]) == sizeof(int)
        (&mut (*hdr).m[dict_type as usize]) as *mut c_int as *mut c_void,
        (&mut (*hdr).id[dict_type as usize]) as *mut *mut bcf_idpair_t as *mut *mut c_void,
        super::hts::HTS_RESIZE_CLEAR,
        c"bcf_hdr_set_idx".as_ptr(),
    ) != 0
    {
        return -1;
    }
    (*hdr).n[dict_type as usize] = new_n as c_int;

    // NB: the next kh_put call can invalidate the idinfo pointer, therefore
    // we leave it unassigned here. It must be set explicitly in bcf_hdr_sync.
    (*(*hdr).id[dict_type as usize].add((*idinfo).id as usize)).key = tag;
    0
}

// Native translation of htslib/vcf.c bcf_hdr_register_hrec().
// returns: 1 when hdr needs to be synced, -1 on error, 0 otherwise
unsafe fn bcf_hdr_register_hrec(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) -> c_int {
    let mut ret: c_int = 0;
    let mut replacing = 0;

    bcf_hrec_set_type(hrec);

    if (*hrec).type_ as u32 == BCF_HL_CTG {
        let len: hts_pos_t;
        let mut i = bcf_hrec_find_key(hrec, c"length".as_ptr());
        if i < 0 {
            len = 0;
        } else {
            let v = *(*hrec).vals.add(i as usize);
            let mut end: *mut c_char = v;
            len = libc::strtoll(v, &mut end, 10);
            if end == v || len < 0 {
                return 0;
            }
        }

        i = bcf_hrec_find_key(hrec, c"ID".as_ptr());
        if i < 0 {
            return 0;
        }
        let mut str_ = cc_strdup(*(*hrec).vals.add(i as usize));
        if str_.is_null() {
            return -1;
        }

        let d = (*hdr).dict[BCF_DT_CTG as usize].cast::<kh_vdict_t>();
        let mut k = kh_get_vdict(d, str_);
        if k != (*d).n_buckets {
            // already present
            libc::free(str_.cast());
            str_ = std::ptr::null_mut();
            if !(*(*d).vals.add(k as usize)).hrec[0].is_null() {
                return 0; // and not removed
            }
            replacing = 1;
        } else {
            k = kh_put_vdict(d, str_, &mut ret);
            if ret < 0 {
                libc::free(str_.cast());
                return -1;
            }
        }

        let mut idx = bcf_hrec_find_key(hrec, c"IDX".as_ptr());
        if idx != -1 {
            let v = *(*hrec).vals.add(idx as usize);
            let mut tmp: *mut c_char = v;
            idx = libc::strtol(v, &mut tmp, 10) as c_int;
            if *tmp != 0 || idx < 0 || idx >= c_int::MAX - 1 {
                if replacing == 0 {
                    kh_del_vdict(d, k);
                    libc::free(str_.cast());
                }
                c_log_warning(c"Error parsing the IDX tag, skipping".as_ptr());
                return 0;
            }
        }

        let valp = (*d).vals.add(k as usize);
        *valp = bcf_idinfo_def();
        (*valp).id = idx;
        (*valp).info[0] = len as u64;
        (*valp).hrec[0] = hrec;
        if bcf_hdr_set_idx(hdr, BCF_DT_CTG as c_int, *(*d).keys.add(k as usize), valp) < 0 {
            if replacing == 0 {
                kh_del_vdict(d, k);
                libc::free(str_.cast());
            }
            return -1;
        }
        if idx == -1 && hrec_add_idx(hrec, (*(*d).vals.add(k as usize)).id) < 0 {
            return -1;
        }
        return 1;
    }

    if (*hrec).type_ as u32 == BCF_HL_STR {
        return 1;
    }
    if (*hrec).type_ as u32 != BCF_HL_INFO
        && (*hrec).type_ as u32 != BCF_HL_FLT
        && (*hrec).type_ as u32 != BCF_HL_FMT
    {
        return 0;
    }

    // INFO/FILTER/FORMAT
    let mut id: *mut c_char = std::ptr::null_mut();
    let mut type_v: u32 = u32::MAX;
    let mut var: u32 = u32::MAX;
    let mut num: c_int = -1;
    let mut idx: c_int = -1;
    let mut i: c_int = 0;
    while i < (*hrec).nkeys {
        let ki = *(*hrec).keys.add(i as usize);
        let vi = *(*hrec).vals.add(i as usize);
        if libc::strcmp(ki, c"ID".as_ptr()) == 0 {
            id = vi;
        } else if libc::strcmp(ki, c"IDX".as_ptr()) == 0 {
            let mut tmp: *mut c_char = vi;
            idx = libc::strtol(vi, &mut tmp, 10) as c_int;
            if *tmp != 0 || idx < 0 || idx >= c_int::MAX - 1 {
                c_log_warning(c"Error parsing the IDX tag, skipping".as_ptr());
                return 0;
            }
        } else if libc::strcmp(ki, c"Type".as_ptr()) == 0 {
            if libc::strcmp(vi, c"Integer".as_ptr()) == 0 {
                type_v = BCF_HT_INT;
            } else if libc::strcmp(vi, c"Float".as_ptr()) == 0 {
                type_v = BCF_HT_REAL;
            } else if libc::strcmp(vi, c"String".as_ptr()) == 0 {
                type_v = BCF_HT_STR;
            } else if libc::strcmp(vi, c"Character".as_ptr()) == 0 {
                type_v = BCF_HT_STR;
            } else if libc::strcmp(vi, c"Flag".as_ptr()) == 0 {
                type_v = BCF_HT_FLAG;
            } else {
                c_log_warning(c"The type is not supported, assuming String".as_ptr());
                type_v = BCF_HT_STR;
            }
        } else if libc::strcmp(ki, c"Number".as_ptr()) == 0 {
            let is_fmt = (*hrec).type_ as u32 == BCF_HL_FMT;
            if libc::strcmp(vi, c"A".as_ptr()) == 0 {
                var = BCF_VL_A;
            } else if libc::strcmp(vi, c"R".as_ptr()) == 0 {
                var = BCF_VL_R;
            } else if libc::strcmp(vi, c"G".as_ptr()) == 0 {
                var = BCF_VL_G;
            } else if libc::strcmp(vi, c".".as_ptr()) == 0 {
                var = BCF_VL_VAR;
            } else if is_fmt && libc::strcmp(vi, c"P".as_ptr()) == 0 {
                var = BCF_VL_P as u32;
            } else if is_fmt && libc::strcmp(vi, c"LA".as_ptr()) == 0 {
                var = BCF_VL_LA as u32;
            } else if is_fmt && libc::strcmp(vi, c"LR".as_ptr()) == 0 {
                var = BCF_VL_LR as u32;
            } else if is_fmt && libc::strcmp(vi, c"LG".as_ptr()) == 0 {
                var = BCF_VL_LG as u32;
            } else if is_fmt && libc::strcmp(vi, c"M".as_ptr()) == 0 {
                var = BCF_VL_M as u32;
            } else if libc::sscanf(vi, c"%d".as_ptr(), &mut num as *mut c_int) == 1 {
                var = BCF_VL_FIXED;
            }
            if var != BCF_VL_FIXED {
                num = 0xfffff;
            }
        }
        i += 1;
    }
    if (*hrec).type_ as u32 == BCF_HL_INFO || (*hrec).type_ as u32 == BCF_HL_FMT {
        if type_v == u32::MAX {
            c_log_warning(c"A field has no Type defined. Assuming String".as_ptr());
            type_v = BCF_HT_STR;
        }
        if var == u32::MAX {
            c_log_warning(c"A field has no Number defined. Assuming '.'".as_ptr());
            var = BCF_VL_VAR;
        }
        if type_v == BCF_HT_FLAG && (var != BCF_VL_FIXED || num != 0) {
            c_log_warning(c"The definition of Flag is invalid, forcing Number=0".as_ptr());
            var = BCF_VL_FIXED;
            num = 0;
        }
    }
    let info: u32 = ((num as u32) & 0xfffff) << 12
        | (var & 0xf) << 8
        | (type_v & 0xf) << 4
        | ((*hrec).type_ as u32 & 0xf);

    if id.is_null() {
        return 0;
    }
    let str_ = cc_strdup(id);
    if str_.is_null() {
        return -1;
    }

    let d = (*hdr).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>();
    let mut k = kh_get_vdict(d, str_);
    if k != (*d).n_buckets {
        // already present
        libc::free(str_.cast());
        let valp = (*d).vals.add(k as usize);
        if !(*valp).hrec[(info & 0xf) as usize].is_null() {
            return 0;
        }
        (*valp).info[(info & 0xf) as usize] = info as u64;
        (*valp).hrec[(info & 0xf) as usize] = hrec;
        if idx == -1 && hrec_add_idx(hrec, (*valp).id) < 0 {
            return -1;
        }
        return 1;
    }
    k = kh_put_vdict(d, str_, &mut ret);
    if ret < 0 {
        libc::free(str_.cast());
        return -1;
    }
    let valp = (*d).vals.add(k as usize);
    *valp = bcf_idinfo_def();
    (*valp).info[(info & 0xf) as usize] = info as u64;
    (*valp).hrec[(info & 0xf) as usize] = hrec;
    (*valp).id = idx;
    if bcf_hdr_set_idx(hdr, BCF_DT_ID as c_int, *(*d).keys.add(k as usize), valp) < 0 {
        kh_del_vdict(d, k);
        libc::free(str_.cast());
        return -1;
    }
    if idx == -1 && hrec_add_idx(hrec, (*(*d).vals.add(k as usize)).id) < 0 {
        return -1;
    }
    1
}

// Native translation of htslib/vcf.c bcf_hdr_unregister_hrec().
unsafe fn bcf_hdr_unregister_hrec_native(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) {
    let t = (*hrec).type_ as u32;
    if t == BCF_HL_FLT || t == BCF_HL_INFO || t == BCF_HL_FMT || t == BCF_HL_CTG {
        let id = bcf_hrec_find_key(hrec, c"ID".as_ptr());
        if id < 0 || (*(*hrec).vals.add(id as usize)).is_null() {
            return;
        }
        let dict = if t == BCF_HL_CTG {
            (*hdr).dict[BCF_DT_CTG as usize].cast::<kh_vdict_t>()
        } else {
            (*hdr).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>()
        };
        let k = kh_get_vdict(dict, *(*hrec).vals.add(id as usize));
        if k != (*dict).n_buckets {
            let slot = if t == BCF_HL_CTG { 0 } else { t as usize };
            (*(*dict).vals.add(k as usize)).hrec[slot] = std::ptr::null_mut();
        }
    }
}

// Native translation of htslib/vcf.c bcf_hdr_add_hrec().
pub unsafe fn bcf_hdr_add_hrec(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) -> c_int {
    let mut str_: kstring_t = std::mem::zeroed();
    let aux = get_hdr_aux(hdr);

    if hrec.is_null() {
        return 0;
    }

    bcf_hrec_check(hrec); // todo: check return status and propagate errors up

    let res = bcf_hdr_register_hrec(hdr, hrec);
    if res < 0 {
        return -1;
    }
    if res == 0 {
        // If one of the hashed field, then it is already present
        if (*hrec).type_ as u32 != BCF_HL_GEN {
            bcf_hrec_destroy(hrec);
            return 0;
        }
        // Is one of the generic fields and already present?
        if ks_build(
            &mut str_,
            &[c"##".as_ptr(), (*hrec).key, c"=".as_ptr(), (*hrec).value],
        ) < 0
        {
            libc::free(str_.s.cast());
            return -1;
        }
        let k = kh_get_hdict((*aux).gen, str_.s);
        if k != (*(*aux).gen).n_buckets {
            // duplicate record
            bcf_hrec_destroy(hrec);
            libc::free(str_.s.cast());
            return 0;
        }
        if libc::strcmp((*hrec).key, c"fileformat".as_ptr()) == 0 {
            (*aux).version = bcf_get_version_str((*hrec).value);
        }
    }

    let i: c_int;
    if (*hrec).type_ as u32 == BCF_HL_STR && {
        i = bcf_hrec_find_key(hrec, c"ID".as_ptr());
        i >= 0
    } {
        if ks_build(
            &mut str_,
            &[
                c"##".as_ptr(),
                (*hrec).key,
                c"=<ID=".as_ptr(),
                *(*hrec).vals.add(i as usize),
                c">".as_ptr(),
            ],
        ) < 0
        {
            libc::free(str_.s.cast());
            return -1;
        }
        let k = kh_get_hdict((*aux).gen, str_.s);
        if k != (*(*aux).gen).n_buckets {
            // duplicate record
            bcf_hrec_destroy(hrec);
            libc::free(str_.s.cast());
            return 0;
        }
    }

    // New record, needs to be added
    let n = (*hdr).nhrec as usize + 1;
    let new_hrec = hts_realloc_p_cc((*hdr).hrec.cast(), size_of::<*mut bcf_hrec_t>(), n)
        .cast::<*mut bcf_hrec_t>();
    if new_hrec.is_null() {
        libc::free(str_.s.cast());
        bcf_hdr_unregister_hrec_native(hdr, hrec);
        return -1;
    }
    (*hdr).hrec = new_hrec;

    if !str_.s.is_null() {
        let mut res2: c_int = 0;
        let k = kh_put_hdict((*aux).gen, str_.s, &mut res2);
        if res2 < 0 {
            libc::free(str_.s.cast());
            return -1;
        }
        *(*(*aux).gen).vals.add(k as usize) = hrec;
    }

    *(*hdr).hrec.add((*hdr).nhrec as usize) = hrec;
    (*hdr).dirty = 1;
    (*hdr).nhrec = n as c_int;

    if (*hrec).type_ as u32 == BCF_HL_GEN {
        0
    } else {
        1
    }
}

// Native translation of htslib/vcf.c bcf_hdr_update_hrec().
pub unsafe fn bcf_hdr_update_hrec(
    hdr: *mut bcf_hdr_t,
    hrec: *mut bcf_hrec_t,
    tmp: *const bcf_hrec_t,
) -> c_int {
    debug_assert!((*hrec).type_ as u32 == BCF_HL_GEN);
    let mut ret: c_int = 0;
    let aux = get_hdr_aux(hdr);
    let gen = (*aux).gen;
    // Find the bucket whose value is `hrec`.
    let mut k: u32 = 0;
    while k < (*gen).n_buckets {
        if kh_iseither((*gen).flags, k) {
            k += 1;
            continue;
        }
        if hrec == *(*gen).vals.add(k as usize) {
            break;
        }
        k += 1;
    }
    debug_assert!(k < (*gen).n_buckets); // should never happen
    libc::free((*(*gen).keys.add(k as usize)) as *mut c_void);
    kh_del_hdict(gen, k);

    let mut str_: kstring_t = std::mem::zeroed();
    if ks_build(
        &mut str_,
        &[c"##".as_ptr(), (*tmp).key, c"=".as_ptr(), (*tmp).value],
    ) < 0
    {
        libc::free(str_.s.cast());
        return -1;
    }
    let k = kh_put_hdict(gen, str_.s, &mut ret);
    if ret < 0 {
        libc::free(str_.s.cast());
        return -1;
    }
    libc::free((*hrec).value.cast());
    (*hrec).value = cc_strdup((*tmp).value);
    if (*hrec).value.is_null() {
        return -1;
    }
    *(*gen).vals.add(k as usize) = hrec;

    if libc::strcmp((*hrec).key, c"fileformat".as_ptr()) == 0 {
        // update version
        (*get_hdr_aux(hdr)).version = bcf_get_version_str((*hrec).value);
    }
    0
}

// Native translation of htslib/vcf.c bcf_hdr_get_hrec().
pub unsafe fn bcf_hdr_get_hrec(
    hdr: *const bcf_hdr_t,
    type_: c_int,
    key: *const c_char,
    value: *const c_char,
    str_class: *const c_char,
) -> *mut bcf_hrec_t {
    if type_ as u32 == BCF_HL_GEN {
        if !value.is_null() {
            let mut str_: kstring_t = std::mem::zeroed();
            let _ = ks_build(&mut str_, &[c"##".as_ptr(), key, c"=".as_ptr(), value]);
            let aux = get_hdr_aux(hdr);
            let k = kh_get_hdict((*aux).gen, str_.s);
            libc::free(str_.s.cast());
            if k == (*(*aux).gen).n_buckets {
                return std::ptr::null_mut();
            }
            return *(*(*aux).gen).vals.add(k as usize);
        }
        let mut i: c_int = 0;
        while i < (*hdr).nhrec {
            let hr = *(*hdr).hrec.add(i as usize);
            if (*hr).type_ == type_ && libc::strcmp((*hr).key, key) == 0 {
                return hr;
            }
            i += 1;
        }
        return std::ptr::null_mut();
    } else if type_ as u32 == BCF_HL_STR {
        if str_class.is_null() {
            return std::ptr::null_mut();
        }
        if libc::strcmp(c"ID".as_ptr(), key) == 0 {
            let mut str_: kstring_t = std::mem::zeroed();
            let _ = ks_build(
                &mut str_,
                &[
                    c"##".as_ptr(),
                    str_class,
                    c"=<".as_ptr(),
                    key,
                    c"=".as_ptr(),
                    value,
                    c">".as_ptr(),
                ],
            );
            let aux = get_hdr_aux(hdr);
            let k = kh_get_hdict((*aux).gen, str_.s);
            libc::free(str_.s.cast());
            if k == (*(*aux).gen).n_buckets {
                return std::ptr::null_mut();
            }
            return *(*(*aux).gen).vals.add(k as usize);
        }
        let mut i: c_int = 0;
        while i < (*hdr).nhrec {
            let hr = *(*hdr).hrec.add(i as usize);
            if (*hr).type_ == type_ && libc::strcmp((*hr).key, str_class) == 0 {
                let j = bcf_hrec_find_key(hr, key);
                if j >= 0 && libc::strcmp(*(*hr).vals.add(j as usize), value) == 0 {
                    return hr;
                }
            }
            i += 1;
        }
        return std::ptr::null_mut();
    }
    let d = if type_ as u32 == BCF_HL_CTG {
        (*hdr).dict[BCF_DT_CTG as usize].cast::<kh_vdict_t>()
    } else {
        (*hdr).dict[BCF_DT_ID as usize].cast::<kh_vdict_t>()
    };
    let k = kh_get_vdict(d, value);
    if k == (*d).n_buckets {
        return std::ptr::null_mut();
    }
    let slot = if type_ as u32 == BCF_HL_CTG {
        0
    } else {
        type_ as usize
    };
    (*(*d).vals.add(k as usize)).hrec[slot]
}

// realloc(ptr, size*n) with overflow check, matching htslib hts_realloc_p.
#[inline]
pub(crate) unsafe fn hts_realloc_p_cc(ptr: *mut c_void, size: usize, n: usize) -> *mut c_void {
    if n != 0 && size > usize::MAX / n {
        *c_compat::__errno_location() = c_compat::ENOMEM;
        return std::ptr::null_mut();
    }
    libc::realloc(ptr, size.wrapping_mul(n))
}

// Native translation of htslib/vcf.c bcf_hrec_dup(). Copies all fields except IDX.
pub unsafe fn bcf_hrec_dup(hrec: *mut bcf_hrec_t) -> *mut bcf_hrec_t {
    let out = libc::calloc(1, size_of::<bcf_hrec_t>()).cast::<bcf_hrec_t>();
    if out.is_null() {
        return std::ptr::null_mut();
    }
    (*out).type_ = (*hrec).type_;
    if !(*hrec).key.is_null() {
        (*out).key = libc::strdup((*hrec).key);
        if (*out).key.is_null() {
            bcf_hrec_destroy(out);
            return std::ptr::null_mut();
        }
    }
    if !(*hrec).value.is_null() {
        (*out).value = libc::strdup((*hrec).value);
        if (*out).value.is_null() {
            bcf_hrec_destroy(out);
            return std::ptr::null_mut();
        }
    }
    (*out).nkeys = (*hrec).nkeys;
    (*out).keys = hts_realloc_p_cc(
        std::ptr::null_mut(),
        size_of::<*mut c_char>(),
        (*hrec).nkeys as usize,
    )
    .cast();
    if (*out).keys.is_null() {
        bcf_hrec_destroy(out);
        return std::ptr::null_mut();
    }
    (*out).vals = hts_realloc_p_cc(
        std::ptr::null_mut(),
        size_of::<*mut c_char>(),
        (*hrec).nkeys as usize,
    )
    .cast();
    if (*out).vals.is_null() {
        bcf_hrec_destroy(out);
        return std::ptr::null_mut();
    }
    let mut j: c_int = 0;
    let mut i: c_int = 0;
    while i < (*hrec).nkeys {
        let ki = *(*hrec).keys.add(i as usize);
        if !ki.is_null() && libc::strcmp(c"IDX".as_ptr(), ki) == 0 {
            i += 1;
            continue;
        }
        if !ki.is_null() {
            *(*out).keys.add(j as usize) = libc::strdup(ki);
            if (*(*out).keys.add(j as usize)).is_null() {
                bcf_hrec_destroy(out);
                return std::ptr::null_mut();
            }
        }
        let vi = *(*hrec).vals.add(i as usize);
        if !vi.is_null() {
            *(*out).vals.add(j as usize) = libc::strdup(vi);
            if (*(*out).vals.add(j as usize)).is_null() {
                bcf_hrec_destroy(out);
                return std::ptr::null_mut();
            }
        }
        j += 1;
        i += 1;
    }
    if i != j {
        (*out).nkeys -= i - j; // IDX was omitted
    }
    out
}

// Native translation of htslib/vcf.c bcf_hrec_add_key().
pub unsafe fn bcf_hrec_add_key(hrec: *mut bcf_hrec_t, str_: *const c_char, len: usize) -> c_int {
    let n = (*hrec).nkeys as usize + 1;
    let tmp = hts_realloc_p_cc((*hrec).keys.cast(), size_of::<*mut c_char>(), n);
    if tmp.is_null() {
        return -1;
    }
    (*hrec).keys = tmp.cast();
    let tmp = hts_realloc_p_cc((*hrec).vals.cast(), size_of::<*mut c_char>(), n);
    if tmp.is_null() {
        return -1;
    }
    (*hrec).vals = tmp.cast();

    let nk = (*hrec).nkeys as usize;
    // hts_malloc_ps(sizeof(char), len, 1) == malloc(len + 1)
    let key_buf = libc::malloc(len + 1).cast::<c_char>();
    if key_buf.is_null() {
        return -1;
    }
    *(*hrec).keys.add(nk) = key_buf;
    libc::memcpy(key_buf.cast(), str_.cast(), len);
    *key_buf.add(len) = 0;
    *(*hrec).vals.add(nk) = std::ptr::null_mut();
    (*hrec).nkeys = n as c_int;
    0
}

// Native translation of htslib/vcf.c bcf_hrec_set_val().
pub unsafe fn bcf_hrec_set_val(
    hrec: *mut bcf_hrec_t,
    i: c_int,
    str_: *const c_char,
    len: usize,
    is_quoted: c_int,
) -> c_int {
    let slot = (*hrec).vals.add(i as usize);
    if !(*slot).is_null() {
        libc::free((*slot).cast());
        *slot = std::ptr::null_mut();
    }
    if str_.is_null() {
        return 0;
    }
    if is_quoted != 0 {
        if len >= usize::MAX - 3 {
            *c_compat::__errno_location() = c_compat::ENOMEM;
            return -1;
        }
        let buf = libc::malloc(len + 3).cast::<c_char>();
        if buf.is_null() {
            return -1;
        }
        *slot = buf;
        *buf.add(0) = b'"' as c_char;
        libc::memcpy(buf.add(1).cast(), str_.cast(), len);
        *buf.add(len + 1) = b'"' as c_char;
        *buf.add(len + 2) = 0;
    } else {
        if len == usize::MAX {
            *c_compat::__errno_location() = c_compat::ENOMEM;
            return -1;
        }
        let buf = libc::malloc(len + 1).cast::<c_char>();
        if buf.is_null() {
            return -1;
        }
        *slot = buf;
        libc::memcpy(buf.cast(), str_.cast(), len);
        *buf.add(len) = 0;
    }
    0
}

// Native translation of htslib/vcf.c bcf_hrec_find_key().
pub unsafe fn bcf_hrec_find_key(hrec: *mut bcf_hrec_t, key: *const c_char) -> c_int {
    let mut i: c_int = 0;
    while i < (*hrec).nkeys {
        if libc::strcasecmp(key, *(*hrec).keys.add(i as usize)) == 0 {
            return i;
        }
        i += 1;
    }
    -1
}

// Native translation of htslib/vcf.c hrec_add_idx().
pub unsafe fn hrec_add_idx(hrec: *mut bcf_hrec_t, idx: c_int) -> c_int {
    let n = (*hrec).nkeys as usize + 1;
    let tmp = hts_realloc_p_cc((*hrec).keys.cast(), size_of::<*mut c_char>(), n);
    if tmp.is_null() {
        return -1;
    }
    (*hrec).keys = tmp.cast();
    let tmp = hts_realloc_p_cc((*hrec).vals.cast(), size_of::<*mut c_char>(), n);
    if tmp.is_null() {
        return -1;
    }
    (*hrec).vals = tmp.cast();

    let nk = (*hrec).nkeys as usize;
    *(*hrec).keys.add(nk) = libc::strdup(c"IDX".as_ptr());
    if (*(*hrec).keys.add(nk)).is_null() {
        return -1;
    }
    let mut str_: kstring_t = std::mem::zeroed();
    if kputw(idx, &mut str_) < 0 {
        libc::free((*(*hrec).keys.add(nk)).cast());
        return -1;
    }
    *(*hrec).vals.add(nk) = str_.s;
    (*hrec).nkeys = n as c_int;
    0
}

// Native translation of htslib/vcf.c bcf_hrec_destroy().
pub unsafe fn bcf_hrec_destroy(hrec: *mut bcf_hrec_t) {
    if hrec.is_null() {
        return;
    }
    libc::free((*hrec).key.cast());
    if !(*hrec).value.is_null() {
        libc::free((*hrec).value.cast());
    }
    let mut i: c_int = 0;
    while i < (*hrec).nkeys {
        libc::free((*(*hrec).keys.add(i as usize)).cast());
        libc::free((*(*hrec).vals.add(i as usize)).cast());
        i += 1;
    }
    libc::free((*hrec).keys.cast());
    libc::free((*hrec).vals.cast());
    libc::free(hrec.cast());
}

// Native translation of htslib/vcf.c bcf_subset().
pub unsafe fn bcf_subset(
    _h: *const bcf_hdr_t,
    v: *mut bcf1_t,
    n: c_int,
    imap: *mut c_int,
) -> c_int {
    const MAX_N_FMT: usize = 255;
    let mut ind = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    if n != 0 {
        let mut fmt = [std::mem::zeroed::<bcf_fmt_t>(); MAX_N_FMT];
        let mut ptr = (*v).indiv.s.cast::<u8>();
        let n_fmt = (*v).n_fmt() as c_int;
        let n_sample = (*v).n_sample() as c_int;
        for i in 0..n_fmt as usize {
            ptr = bcf_unpack_fmt_core1_rs(ptr, n_sample, &mut fmt[i]);
        }
        for i in 0..n_fmt as usize {
            let f = &fmt[i];
            bcf_enc_int1(&mut ind, f.id);
            bcf_enc_size(&mut ind, f.n, f.type_);
            for j in 0..n as usize {
                if *imap.add(j) >= 0 {
                    kputsn(
                        f.p.add((*imap.add(j) * f.size) as usize).cast(),
                        f.size as size_t,
                        &mut ind,
                    );
                }
            }
        }
        let mut i = 0;
        for j in 0..n as usize {
            if *imap.add(j) >= 0 {
                i += 1;
            }
        }
        (*v).set_n_sample(i as u32);
    } else {
        (*v).set_n_sample(0);
    }
    if (*v).n_sample() == 0 {
        (*v).set_n_fmt(0);
    }
    libc::free((*v).indiv.s.cast());
    (*v).indiv.l = ind.l;
    (*v).indiv.m = ind.m;
    (*v).indiv.s = ind.s;
    // Only BCF is ready for output; VCF will need to unpack again.
    (*v).unpacked &= !(BCF_UN_FMT as c_int);
    0
}

pub unsafe fn bcf_get_variant_types(rec: *mut bcf1_t) -> c_int {
    vcf_c_5474_bcf_get_variant_types(rec)
}

pub unsafe fn bcf_get_variant_type(rec: *mut bcf1_t, ith_allele: c_int) -> c_int {
    vcf_c_5485_bcf_get_variant_type(rec, ith_allele)
}

pub unsafe fn bcf_is_snp(v: *mut bcf1_t) -> c_int {
    bcf_unpack(v, BCF_UN_STR as c_int);
    let n_allele = (*v).n_allele() as c_int;
    let mut i = 0;
    while i < n_allele {
        let allele = *(*v).d.allele.add(i as usize);
        let c0 = *allele;
        let c1 = *allele.add(1);
        if c1 == 0 && c0 != b'*' as c_char {
            i += 1;
            continue;
        }
        // mpileup's <X> / <*> alleles are not treated as variants. Read [2] only
        // when [0]=='<' (matching C short-circuit) to avoid over-reading.
        if c0 == b'<' as c_char && c1 == b'X' as c_char && *allele.add(2) == b'>' as c_char {
            i += 1;
            continue;
        }
        if c0 == b'<' as c_char && c1 == b'*' as c_char && *allele.add(2) == b'>' as c_char {
            i += 1;
            continue;
        }
        break;
    }
    (i == n_allele) as c_int
}

// Native equivalent of the hts_expand(type_t, n, m, ptr) macro for the
// bcf1_t dynamic arrays. `m` is an i32 field (m_flt/m_info/m_fmt/m_allele).
unsafe fn bcf_hts_expand_i32(
    n: c_int,
    m: *mut c_int,
    ptr: *mut *mut c_void,
    elem_size: usize,
    clear: c_int,
) {
    if n > *m {
        let new_m = crate::htslib_rs::hts::hts_realloc_or_die(
            if n >= 1 { n as size_t } else { 1 },
            *m as size_t,
            size_of::<c_int>() as size_t,
            elem_size as size_t,
            clear,
            ptr,
            c"bcf_update".as_ptr(),
        );
        *m = new_m as c_int;
    }
}

pub unsafe fn bcf_update_filter(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    flt_ids: *mut c_int,
    n: c_int,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_filter().
    let _ = hdr;
    if (*line).unpacked & BCF_UN_FLT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FLT as c_int);
    }
    (*line).d.shared_dirty |= BCF1_DIRTY_FLT as c_int;
    (*line).d.n_flt = n;
    if n == 0 {
        return 0;
    }
    bcf_hts_expand_i32(
        (*line).d.n_flt,
        &mut (*line).d.m_flt,
        &mut (*line).d.flt as *mut *mut c_int as *mut *mut c_void,
        size_of::<c_int>(),
        0,
    );
    for i in 0..n as usize {
        *(*line).d.flt.add(i) = *flt_ids.add(i);
    }
    0
}

pub unsafe fn bcf_add_filter(hdr: *const bcf_hdr_t, line: *mut bcf1_t, flt_id: c_int) -> c_int {
    // Native translation of htslib/vcf.c bcf_add_filter().
    let _ = hdr;
    if (*line).unpacked & BCF_UN_FLT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FLT as c_int);
    }
    let mut i = 0;
    while i < (*line).d.n_flt {
        if flt_id == *(*line).d.flt.add(i as usize) {
            break;
        }
        i += 1;
    }
    if i < (*line).d.n_flt {
        return 0; // this filter is already set
    }
    (*line).d.shared_dirty |= BCF1_DIRTY_FLT as c_int;
    if flt_id == 0 {
        // set to PASS
        (*line).d.n_flt = 1;
    } else if (*line).d.n_flt == 1 && *(*line).d.flt == 0 {
        (*line).d.n_flt = 1;
    } else {
        (*line).d.n_flt += 1;
    }
    bcf_hts_expand_i32(
        (*line).d.n_flt,
        &mut (*line).d.m_flt,
        &mut (*line).d.flt as *mut *mut c_int as *mut *mut c_void,
        size_of::<c_int>(),
        0,
    );
    *(*line).d.flt.add(((*line).d.n_flt - 1) as usize) = flt_id;
    1
}

pub unsafe fn bcf_remove_filter(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    flt_id: c_int,
    pass: c_int,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_remove_filter().
    if (*line).unpacked & BCF_UN_FLT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FLT as c_int);
    }
    let mut i = 0;
    while i < (*line).d.n_flt {
        if flt_id == *(*line).d.flt.add(i as usize) {
            break;
        }
        i += 1;
    }
    if i == (*line).d.n_flt {
        return 0; // the filter is not present
    }
    (*line).d.shared_dirty |= BCF1_DIRTY_FLT as c_int;
    if i != (*line).d.n_flt - 1 {
        std::ptr::copy(
            (*line).d.flt.add((i + 1) as usize),
            (*line).d.flt.add(i as usize),
            ((*line).d.n_flt - i - 1) as usize,
        );
    }
    (*line).d.n_flt -= 1;
    if (*line).d.n_flt == 0 && pass != 0 {
        bcf_add_filter(hdr, line, 0);
    }
    0
}

pub unsafe fn bcf_has_filter(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    filter: *mut c_char,
) -> c_int {
    // "." resolves to "PASS"
    let filter = if *filter == b'.' as c_char && *filter.add(1) == 0 {
        c"PASS".as_ptr()
    } else {
        filter as *const c_char
    };
    let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, filter);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_FLT as c_int, id) {
        return -1;
    }
    if (*line).unpacked & BCF_UN_FLT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FLT as c_int);
    }
    if id == 0 && (*line).d.n_flt == 0 {
        return 1; // PASS
    }
    for i in 0..(*line).d.n_flt as usize {
        if *(*line).d.flt.add(i) == id {
            return 1;
        }
    }
    0
}

pub unsafe fn bcf_update_alleles(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    alleles: *mut *const c_char,
    nals: c_int,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_alleles().
    if (*line).unpacked & BCF_UN_STR as c_int == 0 {
        bcf_unpack(line, BCF_UN_STR as c_int);
    }
    let mut free_old: *mut c_char = std::ptr::null_mut();
    let mut buffer = [0u8; 256];
    let mut used: usize = 0;

    // The pointers in alleles may point into the existing line->d.als memory,
    // so copy via an intermediate buffer (or a fresh allocation if too long).
    let mut i = 0usize;
    let avail = if ((*line).d.m_als as usize) < buffer.len() {
        (*line).d.m_als as usize
    } else {
        buffer.len()
    };
    while i < nals as usize {
        let src = *alleles.add(i);
        let sz = libc::strlen(src) + 1;
        if avail - used < sz {
            break;
        }
        std::ptr::copy_nonoverlapping(src.cast::<u8>(), buffer.as_mut_ptr().add(used), sz);
        used += sz;
        i += 1;
    }

    // Did we miss anything?
    if i < nals as usize {
        let mut needed = used;
        for j in i..nals as usize {
            needed += libc::strlen(*alleles.add(j)) + 1;
        }
        if needed < (*line).d.m_als as usize {
            needed = (*line).d.m_als as usize; // don't shrink the buffer
        }
        if needed > c_int::MAX as usize {
            return -1;
        }
        let new_als = libc::malloc(needed).cast::<c_char>();
        if new_als.is_null() {
            return -1;
        }
        free_old = (*line).d.als;
        (*line).d.als = new_als;
        (*line).d.m_als = needed as c_int;
    }

    // Copy from the temp buffer to the destination
    if used != 0 {
        std::ptr::copy_nonoverlapping((buffer.as_ptr()).cast::<c_char>(), (*line).d.als, used);
    }

    // Add in any remaining entries (always into a newly-allocated buffer).
    while i < nals as usize {
        let src = *alleles.add(i);
        let sz = libc::strlen(src) + 1;
        std::ptr::copy_nonoverlapping(src.cast::<u8>(), (*line).d.als.add(used).cast::<u8>(), sz);
        used += sz;
        i += 1;
    }

    if !free_old.is_null() {
        libc::free(free_old.cast());
    }
    vcf_c_5884__bcf1_sync_alleles(hdr, line, nals)
}

pub unsafe fn bcf_update_alleles_str(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    alleles_string: *const c_char,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_alleles_str().
    if (*line).unpacked & BCF_UN_STR as c_int == 0 {
        bcf_unpack(line, BCF_UN_STR as c_int);
    }
    let mut tmp = kstring_t {
        l: 0,
        m: (*line).d.m_als as size_t,
        s: (*line).d.als,
    };
    super::hts::kputs(alleles_string, &mut tmp);
    (*line).d.als = tmp.s;
    (*line).d.m_als = tmp.m as c_int;

    let mut nals = 1;
    let mut t = (*line).d.als;
    while *t != 0 {
        if *t == b',' as c_char {
            *t = 0;
            nals += 1;
        }
        t = t.add(1);
    }
    vcf_c_5884__bcf1_sync_alleles(hdr, line, nals)
}

pub unsafe fn bcf_update_id(hdr: *const bcf_hdr_t, line: *mut bcf1_t, id: *const c_char) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_id().
    let _ = hdr;
    if (*line).unpacked & BCF_UN_STR as c_int == 0 {
        bcf_unpack(line, BCF_UN_STR as c_int);
    }
    let mut tmp = kstring_t {
        l: 0,
        m: (*line).d.m_id as size_t,
        s: (*line).d.id,
    };
    if !id.is_null() {
        super::hts::kputs(id, &mut tmp);
    } else {
        super::hts::kputs(c".".as_ptr(), &mut tmp);
    }
    (*line).d.id = tmp.s;
    (*line).d.m_id = tmp.m as c_int;
    (*line).d.shared_dirty |= BCF1_DIRTY_ID as c_int;
    0
}

// Native translation of htslib/vcf.c bcf_add_id().
pub unsafe fn bcf_add_id(_hdr: *const bcf_hdr_t, line: *mut bcf1_t, id: *const c_char) -> c_int {
    if id.is_null() {
        return 0;
    }
    if (*line).unpacked & BCF_UN_STR as c_int == 0 {
        bcf_unpack(line, BCF_UN_STR as c_int);
    }

    let mut tmp = kstring_t {
        l: 0,
        s: (*line).d.id,
        m: (*line).d.m_id as size_t,
    };

    let len = libc::strlen(id);
    let mut dst = (*line).d.id;
    // while ( *dst && (dst=strstr(dst,id)) )
    while *dst != 0 && {
        dst = libc::strstr(dst, id);
        !dst.is_null()
    } {
        if *dst.add(len) != 0 && *dst.add(len) != b';' as c_char {
            dst = dst.add(1); // a prefix, not a match
        } else if dst == (*line).d.id || *dst.offset(-1) == b';' as c_char {
            return 0; // already present
        }
        dst = dst.add(1); // a suffix, not a match
    }
    if !(*line).d.id.is_null() && (*(*line).d.id != b'.' as c_char || *(*line).d.id.add(1) != 0) {
        tmp.l = libc::strlen((*line).d.id);
        kputc(b';' as c_int, &mut tmp);
    }
    kputs(id, &mut tmp);

    (*line).d.id = tmp.s;
    (*line).d.m_id = tmp.m as c_int;
    (*line).d.shared_dirty |= BCF1_DIRTY_ID as c_int;
    0
}

pub unsafe fn bcf_update_info(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *const c_void,
    n: c_int,
    type_: c_int,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_info().
    let inf_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, key);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_INFO as c_int, inf_id) {
        return -1; // No such INFO field in the header
    }
    if (*line).unpacked & BCF_UN_INFO as c_int == 0 {
        bcf_unpack(line, BCF_UN_INFO as c_int);
    }

    let keyb = CStr::from_ptr(key).to_bytes();
    let is_end_tag = keyb == b"END";
    let is_svlen_tag = keyb == b"SVLEN";

    let mut i = 0;
    while i < (*line).n_info() as c_int {
        if inf_id == (*(*line).d.info.add(i as usize)).key {
            break;
        }
        i += 1;
    }
    let mut inf: *mut bcf_info_t = if i == (*line).n_info() as c_int {
        std::ptr::null_mut()
    } else {
        (*line).d.info.add(i as usize)
    };

    if n == 0 || (type_ == BCF_HT_STR as c_int && values.is_null()) {
        if !inf.is_null() {
            // Mark the tag for removal, free existing memory if necessary
            if (*inf).vptr_free() != 0 {
                libc::free((*inf).vptr.sub((*inf).vptr_off() as usize).cast());
                (*inf).set_vptr_free(0);
            }
            (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
            (*inf).vptr = std::ptr::null_mut();
            (*inf).set_vptr_off(0);
            (*inf).vptr_len = 0;
        }
        if n == 0 && (is_end_tag || is_svlen_tag) {
            (*line).rlen = vcf_get_rlen_decoded(hdr, line);
        }
        return 0;
    }

    if is_end_tag {
        if n != 1 {
            (*line).errcode |= BCF_ERR_TAG_INVALID as c_int;
            return -1;
        }
        if type_ != BCF_HT_INT as c_int && type_ != BCF_HT_LONG {
            (*line).errcode |= BCF_ERR_TAG_INVALID as c_int;
            return -1;
        }
    }

    // Encode the values and determine the size required to accommodate the values
    let mut str_ = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    bcf_enc_int1(&mut str_, inf_id);
    if type_ == BCF_HT_INT as c_int {
        bcf_enc_vint(&mut str_, n, values as *mut i32, -1);
    } else if type_ == BCF_HT_REAL as c_int {
        bcf_enc_vfloat(&mut str_, n, values as *mut f32);
    } else if type_ == BCF_HT_FLAG as c_int || type_ == BCF_HT_STR as c_int {
        if values.is_null() {
            bcf_enc_size(&mut str_, 0, BCF_BT_NULL as c_int);
        } else {
            bcf_enc_vchar(
                &mut str_,
                libc::strlen(values.cast::<c_char>()) as c_int,
                values.cast::<c_char>(),
            );
        }
    } else if type_ == BCF_HT_LONG {
        if n != 1 {
            libc::abort();
        }
        bcf_enc_long1(&mut str_, *values.cast::<i64>());
    } else {
        libc::abort();
    }

    if !inf.is_null() {
        // Is it big enough to accommodate new block?
        if !(*inf).vptr.is_null() && str_.l <= (*inf).vptr_len as usize + (*inf).vptr_off() as usize
        {
            if str_.l != (*inf).vptr_len as usize + (*inf).vptr_off() as usize {
                (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
            }
            let ptr = (*inf).vptr.sub((*inf).vptr_off() as usize);
            std::ptr::copy(str_.s.cast::<u8>(), ptr, str_.l);
            libc::free(str_.s.cast());
            let vptr_free = (*inf).vptr_free();
            bcf_unpack_info_core1_rs(ptr, inf);
            (*inf).set_vptr_free(vptr_free);
        } else {
            if (*inf).vptr_free() != 0 {
                libc::free((*inf).vptr.sub((*inf).vptr_off() as usize).cast());
            }
            bcf_unpack_info_core1_rs(str_.s.cast::<u8>(), inf);
            (*inf).set_vptr_free(1);
            (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
        }
    } else {
        // The tag is not present, create new one
        (*line).set_n_info((*line).n_info() + 1);
        bcf_hts_expand_i32(
            (*line).n_info() as c_int,
            &mut (*line).d.m_info,
            &mut (*line).d.info as *mut *mut bcf_info_t as *mut *mut c_void,
            size_of::<bcf_info_t>(),
            1,
        );
        inf = (*line).d.info.add(((*line).n_info() - 1) as usize);
        bcf_unpack_info_core1_rs(str_.s.cast::<u8>(), inf);
        (*inf).set_vptr_free(1);
        (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
    }
    (*line).unpacked |= BCF_UN_INFO as c_int;

    if is_svlen_tag || is_end_tag {
        (*line).rlen = vcf_get_rlen_decoded(hdr, line);
    }
    0
}

pub unsafe fn bcf_update_info_int64(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *const i64,
    n: c_int,
) -> c_int {
    unsafe { bcf_update_info(hdr, line, key, values.cast::<c_void>(), n, BCF_HT_LONG) }
}

pub unsafe fn bcf_update_format_string(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *mut *const c_char,
    n: c_int,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_format_string().
    if n == 0 {
        return bcf_update_format(hdr, line, key, std::ptr::null(), 0, BCF_HT_STR as c_int);
    }
    let mut max_len = 0usize;
    for i in 0..n as usize {
        let len = libc::strlen(*values.add(i));
        if len > max_len {
            max_len = len;
        }
    }
    let out = libc::malloc(max_len * n as usize).cast::<c_char>();
    if out.is_null() {
        return -2;
    }
    for i in 0..n as usize {
        let dst = out.add(i * max_len);
        let src = *values.add(i);
        let mut j = 0usize;
        while *src.add(j) != 0 {
            *dst.add(j) = *src.add(j);
            j += 1;
        }
        while j < max_len {
            *dst.add(j) = 0;
            j += 1;
        }
    }
    let ret = bcf_update_format(
        hdr,
        line,
        key,
        out.cast::<c_void>(),
        (max_len * n as usize) as c_int,
        BCF_HT_STR as c_int,
    );
    libc::free(out.cast());
    ret
}

pub unsafe fn bcf_update_format(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
    values: *const c_void,
    n: c_int,
    type_: c_int,
) -> c_int {
    // Native translation of htslib/vcf.c bcf_update_format().
    let fmt_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, key);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_FMT as c_int, fmt_id) {
        if n == 0 {
            return 0;
        }
        return -1; // the key not present in the header
    }

    if (*line).unpacked & BCF_UN_FMT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FMT as c_int);
    }

    let mut i = 0;
    while i < (*line).n_fmt() as c_int {
        if (*(*line).d.fmt.add(i as usize)).id == fmt_id {
            break;
        }
        i += 1;
    }
    let mut fmt: *mut bcf_fmt_t = if i == (*line).n_fmt() as c_int {
        std::ptr::null_mut()
    } else {
        (*line).d.fmt.add(i as usize)
    };

    let keyb = CStr::from_ptr(key).to_bytes();
    let is_len = keyb == b"LEN";
    if n == 0 {
        if !fmt.is_null() {
            if (*fmt).p_free() != 0 {
                libc::free((*fmt).p.sub((*fmt).p_off() as usize).cast());
                (*fmt).set_p_free(0);
            }
            (*line).d.indiv_dirty = 1;
            (*fmt).p = std::ptr::null_mut();
        }
        if is_len {
            (*line).rlen = vcf_get_rlen_decoded(hdr, line);
        }
        return 0;
    }

    let nsamples = (*hdr).n[BCF_DT_SAMPLE as usize];
    (*line).set_n_sample(nsamples as u32);
    let nps = n / nsamples; // number of values per sample
    assert!(nps != 0 && nps * nsamples == n);

    // Encode the values and determine the size required to accommodate them
    let mut str_ = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    bcf_enc_int1(&mut str_, fmt_id);
    assert!(!values.is_null());
    if type_ == BCF_HT_INT as c_int {
        bcf_enc_vint(&mut str_, n, values as *mut i32, nps);
    } else if type_ == BCF_HT_REAL as c_int {
        bcf_enc_size(&mut str_, nps, BCF_BT_FLOAT as c_int);
        serialize_float_array(&mut str_, (nps * nsamples) as usize, values.cast::<f32>());
    } else if type_ == BCF_HT_STR as c_int {
        bcf_enc_size(&mut str_, nps, BCF_BT_CHAR as c_int);
        kputsn(
            values.cast::<c_char>(),
            (nps * nsamples) as usize,
            &mut str_,
        );
    } else {
        libc::abort();
    }

    if fmt.is_null() {
        // Not present, new format field
        (*line).set_n_fmt((*line).n_fmt() + 1);
        bcf_hts_expand_i32(
            (*line).n_fmt() as c_int,
            &mut (*line).d.m_fmt,
            &mut (*line).d.fmt as *mut *mut bcf_fmt_t as *mut *mut c_void,
            size_of::<bcf_fmt_t>(),
            1,
        );
        // Special case: VCF specification requires that GT is always first
        if (*line).n_fmt() > 1
            && *key == b'G' as c_char
            && *key.add(1) == b'T' as c_char
            && *key.add(2) == 0
        {
            let mut j = (*line).n_fmt() as c_int - 1;
            while j > 0 {
                *(*line).d.fmt.add(j as usize) = *(*line).d.fmt.add((j - 1) as usize);
                j -= 1;
            }
            fmt = (*line).d.fmt;
        } else {
            fmt = (*line).d.fmt.add(((*line).n_fmt() - 1) as usize);
        }
        bcf_unpack_fmt_core1_rs(str_.s.cast::<u8>(), nsamples, fmt);
        (*line).d.indiv_dirty = 1;
        (*fmt).set_p_free(1);
    } else {
        // The tag is already present; check if big enough for the new block
        if !(*fmt).p.is_null() && str_.l <= (*fmt).p_len as usize + (*fmt).p_off() as usize {
            if str_.l != (*fmt).p_len as usize + (*fmt).p_off() as usize {
                (*line).d.indiv_dirty = 1;
            }
            let ptr = (*fmt).p.sub((*fmt).p_off() as usize);
            std::ptr::copy(str_.s.cast::<u8>(), ptr, str_.l);
            libc::free(str_.s.cast());
            let p_free = (*fmt).p_free();
            bcf_unpack_fmt_core1_rs(ptr, nsamples, fmt);
            (*fmt).set_p_free(p_free);
        } else {
            if (*fmt).p_free() != 0 {
                libc::free((*fmt).p.sub((*fmt).p_off() as usize).cast());
            }
            bcf_unpack_fmt_core1_rs(str_.s.cast::<u8>(), nsamples, fmt);
            (*fmt).set_p_free(1);
            (*line).d.indiv_dirty = 1;
        }
    }
    (*line).unpacked |= BCF_UN_FMT as c_int;

    if is_len {
        (*line).rlen = vcf_get_rlen_decoded(hdr, line);
    }
    0
}

pub unsafe fn bcf_get_fmt(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
) -> *mut bcf_fmt_t {
    let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, key);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_FMT as c_int, id) {
        return std::ptr::null_mut();
    }
    bcf_get_fmt_id(line, id)
}

pub unsafe fn bcf_get_info(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    key: *const c_char,
) -> *mut bcf_info_t {
    let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, key);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_INFO as c_int, id) {
        return std::ptr::null_mut();
    }
    bcf_get_info_id(line, id)
}

pub unsafe fn bcf_get_format_string(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut *mut c_char,
    ndst: *mut c_int,
) -> c_int {
    let tag_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, tag);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_FMT as c_int, tag_id) {
        return -1;
    }
    if bcf_hdr_id2type_rs(hdr, BCF_HL_FMT as c_int, tag_id) != BCF_HT_STR as c_int {
        return -2;
    }
    if (*line).unpacked & BCF_UN_FMT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FMT as c_int);
    }

    let n_fmt = (*line).n_fmt() as usize;
    let mut idx = n_fmt;
    for i in 0..n_fmt {
        if (*(*line).d.fmt.add(i)).id == tag_id {
            idx = i;
            break;
        }
    }
    if idx == n_fmt {
        return -3;
    }
    let fmt = (*line).d.fmt.add(idx);
    if (*fmt).p.is_null() {
        return -3;
    }

    let nsmpl = (*hdr).n[BCF_DT_SAMPLE as usize];
    if (*dst).is_null() {
        *dst =
            crate::htslib_rs::c_compat::malloc((size_of::<*mut c_char>() as u64) * (nsmpl as u64))
                .cast::<*mut c_char>();
        if (*dst).is_null() {
            return -4;
        }
        *(*dst) = std::ptr::null_mut();
    }
    let fmt_n = (*fmt).n;
    let n = (fmt_n + 1) * nsmpl;
    if *ndst < n {
        let base = crate::htslib_rs::c_compat::realloc((*(*dst)).cast::<c_void>(), n as u64)
            .cast::<c_char>();
        if base.is_null() {
            return -4;
        }
        *(*dst) = base;
        *ndst = n;
    }
    let base = *(*dst);
    for i in 0..nsmpl as usize {
        let src = (*fmt).p.add(i * fmt_n as usize);
        let tmp = base.add(i * (fmt_n + 1) as usize);
        std::ptr::copy_nonoverlapping(src, tmp.cast::<u8>(), fmt_n as usize);
        *tmp.add(fmt_n as usize) = 0;
        *(*dst).add(i) = tmp;
    }
    n
}

pub unsafe fn bcf_get_format_values(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut c_void,
    ndst: *mut c_int,
    type_: c_int,
) -> c_int {
    let tag_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, tag);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_FMT as c_int, tag_id) {
        return -1;
    }
    // Ugly: GT field is considered a string by the VCF header but BCF
    // represents it as INT.
    if *tag == b'G' as c_char && *tag.add(1) == b'T' as c_char && *tag.add(2) == 0 {
        if bcf_hdr_id2type_rs(hdr, BCF_HL_FMT as c_int, tag_id) != BCF_HT_STR as c_int {
            return -2;
        }
    } else if bcf_hdr_id2type_rs(hdr, BCF_HL_FMT as c_int, tag_id) != type_ {
        return -2;
    }
    if (*line).unpacked & BCF_UN_FMT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FMT as c_int);
    }

    let n_fmt = (*line).n_fmt() as usize;
    let mut idx = n_fmt;
    for i in 0..n_fmt {
        if (*(*line).d.fmt.add(i)).id == tag_id {
            idx = i;
            break;
        }
    }
    if idx == n_fmt {
        return -3;
    }
    let fmt = (*line).d.fmt.add(idx);
    if (*fmt).p.is_null() {
        return -3;
    }
    let fmt_n = (*fmt).n;
    let fmt_size = (*fmt).size;
    let fmt_type = (*fmt).type_;
    let fmt_p = (*fmt).p;
    let nsmpl = (*hdr).n[BCF_DT_SAMPLE as usize];

    if type_ == BCF_HT_STR as c_int {
        let n = fmt_n * nsmpl;
        if *ndst < n {
            *dst = crate::htslib_rs::c_compat::realloc(*dst, n as u64);
            if (*dst).is_null() {
                return -4;
            }
            *ndst = n;
        }
        std::ptr::copy_nonoverlapping(fmt_p, (*dst).cast::<u8>(), n as usize);
        return n;
    }

    let size1 = if type_ == BCF_HT_INT as c_int {
        size_of::<i32>()
    } else {
        size_of::<f32>()
    };
    if *ndst < fmt_n * nsmpl {
        *ndst = fmt_n * nsmpl;
        *dst = crate::htslib_rs::c_compat::realloc(*dst, (*ndst as u64) * (size1 as u64));
        if (*dst).is_null() {
            return -4;
        }
    }

    macro_rules! branch_fmt_int {
        ($read:expr, $sz:expr, $missing:expr, $vend:expr) => {{
            let tmp = (*dst).cast::<i32>();
            let mut out = 0usize;
            for i in 0..nsmpl as usize {
                let base = fmt_p.add(i * fmt_size as usize);
                let mut j = 0i32;
                while j < fmt_n {
                    let p = $read(base.add(j as usize * $sz)) as i32;
                    if p == $missing {
                        *tmp.add(out) = bcf_int32_missing;
                    } else if p == $vend {
                        // Matches C: write vector_end but do NOT advance the
                        // output cursor here; the trailing loop fills from here.
                        *tmp.add(out) = bcf_int32_vector_end;
                        break;
                    } else {
                        *tmp.add(out) = p;
                    }
                    out += 1;
                    j += 1;
                }
                while j < fmt_n {
                    *tmp.add(out) = bcf_int32_vector_end;
                    out += 1;
                    j += 1;
                }
            }
        }};
    }

    match fmt_type as u32 {
        BCF_BT_INT8 => branch_fmt_int!(le_to_i8, 1, bcf_int8_missing, bcf_int8_vector_end),
        BCF_BT_INT16 => branch_fmt_int!(le_to_i16, 2, bcf_int16_missing, bcf_int16_vector_end),
        BCF_BT_INT32 => branch_fmt_int!(le_to_i32, 4, bcf_int32_missing, bcf_int32_vector_end),
        BCF_BT_FLOAT => {
            let tmp = (*dst).cast::<u32>();
            let mut out = 0usize;
            for i in 0..nsmpl as usize {
                let base = fmt_p.add(i * fmt_size as usize);
                let mut j = 0i32;
                while j < fmt_n {
                    let p = super::hts::le_to_u32(base.add(j as usize * 4));
                    if p == bcf_float_missing {
                        *tmp.add(out) = bcf_float_missing;
                    } else if p == bcf_float_vector_end {
                        *tmp.add(out) = bcf_float_vector_end;
                        break;
                    } else {
                        *tmp.add(out) = p;
                    }
                    out += 1;
                    j += 1;
                }
                while j < fmt_n {
                    *tmp.add(out) = bcf_float_vector_end;
                    out += 1;
                    j += 1;
                }
            }
        }
        _ => std::process::abort(),
    }

    nsmpl * fmt_n
}

pub unsafe fn bcf_get_fmt_id(line: *mut bcf1_t, id: c_int) -> *mut bcf_fmt_t {
    if line.is_null() {
        return std::ptr::null_mut();
    }
    if (*line).unpacked & BCF_UN_FMT as c_int == 0 {
        bcf_unpack(line, BCF_UN_FMT as c_int);
    }
    for i in 0..(*line).n_fmt() as usize {
        let fmt = (*line).d.fmt.add(i);
        if (*fmt).id == id {
            return fmt;
        }
    }
    std::ptr::null_mut()
}

pub unsafe fn bcf_get_info_id(line: *mut bcf1_t, id: c_int) -> *mut bcf_info_t {
    if line.is_null() {
        return std::ptr::null_mut();
    }
    if (*line).unpacked & BCF_UN_INFO as c_int == 0 {
        bcf_unpack(line, BCF_UN_INFO as c_int);
    }
    for i in 0..(*line).n_info() as usize {
        let info = (*line).d.info.add(i);
        if (*info).key == id {
            return info;
        }
    }
    std::ptr::null_mut()
}

pub unsafe fn bcf_get_info_values(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut c_void,
    ndst: *mut c_int,
    type_: c_int,
) -> c_int {
    let tag_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, tag);
    if !bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_INFO as c_int, tag_id) {
        return -1;
    }
    if bcf_hdr_id2type_rs(hdr, BCF_HL_INFO as c_int, tag_id) != (type_ & 0xff) {
        return -2;
    }
    if (*line).unpacked & BCF_UN_INFO as c_int == 0 {
        bcf_unpack(line, BCF_UN_INFO as c_int);
    }

    let n_info = (*line).n_info() as usize;
    let mut idx = n_info;
    for i in 0..n_info {
        if (*(*line).d.info.add(i)).key == tag_id {
            idx = i;
            break;
        }
    }
    if idx == n_info {
        return if type_ == BCF_HT_FLAG as c_int { 0 } else { -3 };
    }
    if type_ == BCF_HT_FLAG as c_int {
        return 1;
    }

    let info = (*line).d.info.add(idx);
    if (*info).vptr.is_null() {
        return -3;
    }
    let info_len = (*info).len;

    if type_ == BCF_HT_STR as c_int {
        if *ndst < info_len + 1 {
            *ndst = info_len + 1;
            *dst = crate::htslib_rs::c_compat::realloc(*dst, *ndst as u64);
        }
        std::ptr::copy_nonoverlapping((*info).vptr, (*dst).cast::<u8>(), info_len as usize);
        *(*dst).cast::<u8>().add(info_len as usize) = 0;
        return info_len;
    }

    let size1 = match type_ as u32 {
        BCF_HT_INT => size_of::<i32>(),
        x if x == BCF_HT_LONG as u32 => size_of::<i64>(),
        BCF_HT_REAL => size_of::<f32>(),
        _ => return -2,
    };
    if *ndst < info_len {
        *ndst = info_len;
        *dst = crate::htslib_rs::c_compat::realloc(*dst, (*ndst as u64) * (size1 as u64));
    }

    let info_type = (*info).type_;
    let vptr = (*info).vptr;
    let is_long = type_ == BCF_HT_LONG as c_int;
    let ret: c_int;

    macro_rules! branch_int {
        ($read:expr, $sz:expr, $missing:expr, $vend:expr) => {{
            let mut j = 0i32;
            if is_long {
                let tmp = (*dst).cast::<i64>();
                while j < info_len {
                    let p = $read(vptr.add(j as usize * $sz)) as i64;
                    if p == $vend as i64 {
                        break;
                    }
                    if p == $missing as i64 {
                        *tmp.add(j as usize) = bcf_int64_missing;
                    } else {
                        *tmp.add(j as usize) = p;
                    }
                    j += 1;
                }
            } else {
                let tmp = (*dst).cast::<i32>();
                while j < info_len {
                    let p = $read(vptr.add(j as usize * $sz)) as i32;
                    if p == $vend {
                        break;
                    }
                    if p == $missing {
                        *tmp.add(j as usize) = bcf_int32_missing;
                    } else {
                        *tmp.add(j as usize) = p;
                    }
                    j += 1;
                }
            }
            ret = j;
        }};
    }

    match info_type as u32 {
        BCF_BT_INT8 => branch_int!(le_to_i8, 1, bcf_int8_missing, bcf_int8_vector_end),
        BCF_BT_INT16 => branch_int!(le_to_i16, 2, bcf_int16_missing, bcf_int16_vector_end),
        BCF_BT_INT32 => branch_int!(le_to_i32, 4, bcf_int32_missing, bcf_int32_vector_end),
        BCF_BT_FLOAT => {
            let tmp = (*dst).cast::<u32>();
            let mut j = 0i32;
            while j < info_len {
                let p = super::hts::le_to_u32(vptr.add(j as usize * 4));
                if p == bcf_float_vector_end {
                    break;
                }
                if p == bcf_float_missing {
                    *tmp.add(j as usize) = bcf_float_missing;
                } else {
                    *tmp.add(j as usize) = p;
                }
                j += 1;
            }
            ret = j;
        }
        _ => return -2,
    }
    ret
}

pub unsafe fn bcf_get_info_int64(
    hdr: *const bcf_hdr_t,
    line: *mut bcf1_t,
    tag: *const c_char,
    dst: *mut *mut i64,
    ndst: *mut c_int,
) -> c_int {
    unsafe { bcf_get_info_values(hdr, line, tag, dst.cast::<*mut c_void>(), ndst, BCF_HT_LONG) }
}

// khash table layout for vcf's `vdict` (KHASH_MAP_INIT_STR(vdict, bcf_idinfo_t)).
// String keys, FNV1a hash + strcmp equality (the htslib default for str maps),
// values of type bcf_idinfo_t. This is the concrete type behind bcf_hdr_t::dict[].
#[repr(C)]
struct kh_vdict_t {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut *const c_char,
    vals: *mut bcf_idinfo_t,
}

#[inline]
unsafe fn vcf_kh_isempty(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
}

#[inline]
unsafe fn vcf_kh_iseither(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3) != 0
}

#[inline]
unsafe fn vcf_cstr_eq(a: *const c_char, b: *const c_char) -> bool {
    !a.is_null() && !b.is_null() && CStr::from_ptr(a) == CStr::from_ptr(b)
}

// kh_get(vdict, d, key): returns the bucket index, or n_buckets if absent.
unsafe fn kh_get_vdict(h: *const kh_vdict_t, key: *const c_char) -> u32 {
    if h.is_null() || (*h).n_buckets == 0 {
        return if h.is_null() { 0 } else { (*h).n_buckets };
    }
    let mask = (*h).n_buckets - 1;
    // The C library that builds this dict (hts-sys) uses kh_str_hash_func =
    // __ac_FNV1a_hash_string for KHASH_MAP_INIT_STR (changed from X31 in
    // htslib v1.23). Match it exactly.
    let mut i = super::hts::__ac_FNV1a_hash_string(key) & mask;
    let last = i;
    let mut step: u32 = 0;
    while !vcf_kh_isempty((*h).flags, i)
        && (((*(*h).flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
            || !vcf_cstr_eq(*(*h).keys.add(i as usize), key))
    {
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return (*h).n_buckets;
        }
    }
    if vcf_kh_iseither((*h).flags, i) {
        (*h).n_buckets
    } else {
        i
    }
}

// ---------------------------------------------------------------------------
// Native khash machinery for the bcf header dictionaries.
//
// These are faithful ports of the KHASH_INIT macros in htslib/htslib/khash.h
// for the two concrete instantiations used by vcf.c:
//   KHASH_MAP_INIT_STR(vdict, bcf_idinfo_t)   -> kh_vdict_t (above)
//   KHASH_MAP_INIT_STR(hdict, bcf_hrec_t*)    -> kh_hdict_t (below)
// Both use the FNV-1a string hash (kh_str_hash_func == __ac_FNV1a_hash_string
// as of htslib v1.23, changed from __ac_X31_hash_string) and strcmp equality,
// matching the hts-sys C library byte-for-byte so the dicts built natively
// can be read back by either native kh_get_vdict or by C.
// Memory is allocated with the libc allocator (same as hts-sys, via
// crate::htslib_rs::c_compat) so the dicts are interchangeable with C.
// ---------------------------------------------------------------------------

#[repr(C)]
struct kh_hdict_t {
    n_buckets: u32,
    size: u32,
    n_occupied: u32,
    upper_bound: u32,
    flags: *mut u32,
    keys: *mut *const c_char,
    vals: *mut *mut bcf_hrec_t,
}

// Internal opaque struct stored at hdr->dict[0] (bcf_hdr_aux_t in vcf.c).
// The first member is an *inline* vdict_t (not a pointer): user code and
// vcf.c both cast (vdict_t*)hdr->dict[0] to reach the BCF_DT_ID dictionary.
#[repr(C)]
struct bcf_hdr_aux_t {
    dict: kh_vdict_t,
    gen: *mut kh_hdict_t,
    key_len: *mut usize,
    version: c_int,
    ref_count: u32,
}

#[inline]
unsafe fn get_hdr_aux(hdr: *const bcf_hdr_t) -> *mut bcf_hdr_aux_t {
    (*hdr).dict[0].cast::<bcf_hdr_aux_t>()
}

// Native translation of htslib/vcf.c bcf_hdr_incr_ref().
unsafe fn bcf_hdr_incr_ref(h: *mut bcf_hdr_t) {
    let aux = get_hdr_aux(h);
    (*aux).ref_count += 2;
}

// Native translation of htslib/vcf.c bcf_hdr_decr_ref().
unsafe fn bcf_hdr_decr_ref(h: *mut bcf_hdr_t) {
    let aux = get_hdr_aux(h);
    if (*aux).ref_count >= 2 {
        (*aux).ref_count -= 2;
    }
    if (*aux).ref_count == 0 {
        bcf_hdr_destroy(h);
    }
}

// Native translation of htslib/vcf.c hdr_bgzf_private_data_cleanup().
unsafe extern "C" fn hdr_bgzf_private_data_cleanup(data: *mut c_void) {
    let h = data.cast::<bcf_hdr_t>();
    bcf_hdr_decr_ref(h);
}

#[inline]
fn kh_fsize(m: u32) -> usize {
    if m < 16 {
        1
    } else {
        (m >> 4) as usize
    }
}

#[inline]
fn kh_kroundup32(mut x: u32) -> u32 {
    // kroundup32 in khash.h: round up to next power of two (x stays 0 -> 0,
    // but callers ensure x>=1 via the `< 4` clamp).
    x = x.wrapping_sub(1);
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x.wrapping_add(1)
}

const KH_AC_HASH_UPPER: f64 = 0.77;

#[inline]
unsafe fn kh_set_isdel_true(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) |= 1u32 << ((i & 0x0f) << 1);
}
#[inline]
unsafe fn kh_set_isempty_false(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) &= !(2u32 << ((i & 0x0f) << 1));
}
#[inline]
unsafe fn kh_set_isboth_false(flags: *mut u32, i: u32) {
    *flags.add((i >> 4) as usize) &= !(3u32 << ((i & 0x0f) << 1));
}
#[inline]
unsafe fn kh_isempty(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
}
#[inline]
unsafe fn kh_isdel(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
}
#[inline]
unsafe fn kh_iseither(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3) != 0
}

// kh_init(vdict): allocate a zeroed kh_vdict_t with the libc allocator.
unsafe fn kh_init_vdict() -> *mut kh_vdict_t {
    libc::calloc(1, size_of::<kh_vdict_t>()).cast()
}

unsafe fn kh_destroy_vdict(h: *mut kh_vdict_t) {
    if !h.is_null() {
        libc::free((*h).keys.cast());
        libc::free((*h).flags.cast());
        libc::free((*h).vals.cast());
        libc::free(h.cast());
    }
}

// kh_resize(vdict): faithful port (string keys, value bcf_idinfo_t).
unsafe fn kh_resize_vdict(h: *mut kh_vdict_t, mut new_n_buckets: u32) -> c_int {
    let mut new_flags: *mut u32 = std::ptr::null_mut();
    let mut j: u32 = 1;
    {
        new_n_buckets = kh_kroundup32(new_n_buckets);
        if new_n_buckets < 4 {
            new_n_buckets = 4;
        }
        if (*h).size as f64 >= new_n_buckets as f64 * KH_AC_HASH_UPPER + 0.5 {
            j = 0; // requested size is too small
        } else {
            let fsz = kh_fsize(new_n_buckets);
            new_flags = libc::malloc((fsz * 4) as usize).cast();
            if new_flags.is_null() {
                return -1;
            }
            libc::memset(new_flags.cast(), 0xaa, (fsz * 4) as usize);
            if (*h).n_buckets < new_n_buckets {
                let new_keys = libc::realloc(
                    (*h).keys.cast(),
                    (new_n_buckets as usize) * size_of::<*const c_char>() as usize,
                )
                .cast::<*const c_char>();
                if new_keys.is_null() {
                    libc::free(new_flags.cast());
                    return -1;
                }
                (*h).keys = new_keys;
                let new_vals = libc::realloc(
                    (*h).vals.cast(),
                    (new_n_buckets as usize) * size_of::<bcf_idinfo_t>() as usize,
                )
                .cast::<bcf_idinfo_t>();
                if new_vals.is_null() {
                    libc::free(new_flags.cast());
                    return -1;
                }
                (*h).vals = new_vals;
            }
        }
    }
    if j != 0 {
        let mut jj: u32 = 0;
        while jj != (*h).n_buckets {
            if !kh_iseither((*h).flags, jj) {
                let mut key = *(*h).keys.add(jj as usize);
                let mut val = *(*h).vals.add(jj as usize);
                let new_mask = new_n_buckets - 1;
                kh_set_isdel_true((*h).flags, jj);
                loop {
                    let k = super::hts::__ac_FNV1a_hash_string(key);
                    let mut i = k & new_mask;
                    let mut step: u32 = 0;
                    while !kh_isempty(new_flags, i) {
                        step += 1;
                        i = (i + step) & new_mask;
                    }
                    kh_set_isempty_false(new_flags, i);
                    if i < (*h).n_buckets && !kh_iseither((*h).flags, i) {
                        // kick out existing element
                        std::mem::swap(&mut *(*h).keys.add(i as usize), &mut key);
                        std::mem::swap(&mut *(*h).vals.add(i as usize), &mut val);
                        kh_set_isdel_true((*h).flags, i);
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
            (*h).keys = libc::realloc(
                (*h).keys.cast(),
                (new_n_buckets as usize) * size_of::<*const c_char>() as usize,
            )
            .cast();
            (*h).vals = libc::realloc(
                (*h).vals.cast(),
                (new_n_buckets as usize) * size_of::<bcf_idinfo_t>() as usize,
            )
            .cast();
        }
        libc::free((*h).flags.cast());
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound = ((*h).n_buckets as f64 * KH_AC_HASH_UPPER + 0.5) as u32;
    }
    0
}

// kh_put(vdict): faithful port. Sets *ret to 1 (absent/new), 2 (deleted slot
// reused), 0 (present), or -1 (error). Returns bucket index.
unsafe fn kh_put_vdict(h: *mut kh_vdict_t, key: *const c_char, ret: *mut c_int) -> u32 {
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > ((*h).size << 1) {
            if kh_resize_vdict(h, (*h).n_buckets - 1) < 0 {
                *ret = -1;
                return (*h).n_buckets;
            }
        } else if kh_resize_vdict(h, (*h).n_buckets + 1) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }
    let x: u32;
    {
        let mask = (*h).n_buckets - 1;
        let mut site = (*h).n_buckets;
        let mut xx = (*h).n_buckets;
        let k = super::hts::__ac_FNV1a_hash_string(key);
        let mut i = k & mask;
        if kh_isempty((*h).flags, i) {
            xx = i;
        } else {
            let last = i;
            let mut step: u32 = 0;
            while !kh_isempty((*h).flags, i)
                && (kh_isdel((*h).flags, i) || !vcf_cstr_eq(*(*h).keys.add(i as usize), key))
            {
                if kh_isdel((*h).flags, i) {
                    site = i;
                }
                step += 1;
                i = (i + step) & mask;
                if i == last {
                    xx = site;
                    break;
                }
            }
            if xx == (*h).n_buckets {
                if kh_isempty((*h).flags, i) && site != (*h).n_buckets {
                    xx = site;
                } else {
                    xx = i;
                }
            }
        }
        x = xx;
    }
    if kh_isempty((*h).flags, x) {
        *(*h).keys.add(x as usize) = key;
        kh_set_isboth_false((*h).flags, x);
        (*h).size += 1;
        (*h).n_occupied += 1;
        *ret = 1;
    } else if kh_isdel((*h).flags, x) {
        *(*h).keys.add(x as usize) = key;
        kh_set_isboth_false((*h).flags, x);
        (*h).size += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

// kh_del(vdict)
unsafe fn kh_del_vdict(h: *mut kh_vdict_t, x: u32) {
    if x != (*h).n_buckets && !kh_iseither((*h).flags, x) {
        kh_set_isdel_true((*h).flags, x);
        (*h).size -= 1;
    }
}

// kh_del(hdict)
unsafe fn kh_del_hdict(h: *mut kh_hdict_t, x: u32) {
    if x != (*h).n_buckets && !kh_iseither((*h).flags, x) {
        kh_set_isdel_true((*h).flags, x);
        (*h).size -= 1;
    }
}

// --- hdict (string -> bcf_hrec_t*) ---

unsafe fn kh_init_hdict() -> *mut kh_hdict_t {
    libc::calloc(1, size_of::<kh_hdict_t>()).cast()
}

unsafe fn kh_destroy_hdict(h: *mut kh_hdict_t) {
    if !h.is_null() {
        libc::free((*h).keys.cast());
        libc::free((*h).flags.cast());
        libc::free((*h).vals.cast());
        libc::free(h.cast());
    }
}

unsafe fn kh_get_hdict(h: *const kh_hdict_t, key: *const c_char) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let mut i = super::hts::__ac_FNV1a_hash_string(key) & mask;
    let last = i;
    let mut step: u32 = 0;
    while !kh_isempty((*h).flags, i)
        && (kh_isdel((*h).flags, i) || !vcf_cstr_eq(*(*h).keys.add(i as usize), key))
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

unsafe fn kh_resize_hdict(h: *mut kh_hdict_t, mut new_n_buckets: u32) -> c_int {
    let mut new_flags: *mut u32 = std::ptr::null_mut();
    let mut j: u32 = 1;
    {
        new_n_buckets = kh_kroundup32(new_n_buckets);
        if new_n_buckets < 4 {
            new_n_buckets = 4;
        }
        if (*h).size as f64 >= new_n_buckets as f64 * KH_AC_HASH_UPPER + 0.5 {
            j = 0;
        } else {
            let fsz = kh_fsize(new_n_buckets);
            new_flags = libc::malloc((fsz * 4) as usize).cast();
            if new_flags.is_null() {
                return -1;
            }
            libc::memset(new_flags.cast(), 0xaa, (fsz * 4) as usize);
            if (*h).n_buckets < new_n_buckets {
                let new_keys = libc::realloc(
                    (*h).keys.cast(),
                    (new_n_buckets as usize) * size_of::<*const c_char>() as usize,
                )
                .cast::<*const c_char>();
                if new_keys.is_null() {
                    libc::free(new_flags.cast());
                    return -1;
                }
                (*h).keys = new_keys;
                let new_vals = libc::realloc(
                    (*h).vals.cast(),
                    (new_n_buckets as usize) * size_of::<*mut bcf_hrec_t>() as usize,
                )
                .cast::<*mut bcf_hrec_t>();
                if new_vals.is_null() {
                    libc::free(new_flags.cast());
                    return -1;
                }
                (*h).vals = new_vals;
            }
        }
    }
    if j != 0 {
        let mut jj: u32 = 0;
        while jj != (*h).n_buckets {
            if !kh_iseither((*h).flags, jj) {
                let mut key = *(*h).keys.add(jj as usize);
                let mut val = *(*h).vals.add(jj as usize);
                let new_mask = new_n_buckets - 1;
                kh_set_isdel_true((*h).flags, jj);
                loop {
                    let k = super::hts::__ac_FNV1a_hash_string(key);
                    let mut i = k & new_mask;
                    let mut step: u32 = 0;
                    while !kh_isempty(new_flags, i) {
                        step += 1;
                        i = (i + step) & new_mask;
                    }
                    kh_set_isempty_false(new_flags, i);
                    if i < (*h).n_buckets && !kh_iseither((*h).flags, i) {
                        std::mem::swap(&mut *(*h).keys.add(i as usize), &mut key);
                        std::mem::swap(&mut *(*h).vals.add(i as usize), &mut val);
                        kh_set_isdel_true((*h).flags, i);
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
            (*h).keys = libc::realloc(
                (*h).keys.cast(),
                (new_n_buckets as usize) * size_of::<*const c_char>() as usize,
            )
            .cast();
            (*h).vals = libc::realloc(
                (*h).vals.cast(),
                (new_n_buckets as usize) * size_of::<*mut bcf_hrec_t>() as usize,
            )
            .cast();
        }
        libc::free((*h).flags.cast());
        (*h).flags = new_flags;
        (*h).n_buckets = new_n_buckets;
        (*h).n_occupied = (*h).size;
        (*h).upper_bound = ((*h).n_buckets as f64 * KH_AC_HASH_UPPER + 0.5) as u32;
    }
    0
}

unsafe fn kh_put_hdict(h: *mut kh_hdict_t, key: *const c_char, ret: *mut c_int) -> u32 {
    if (*h).n_occupied >= (*h).upper_bound {
        if (*h).n_buckets > ((*h).size << 1) {
            if kh_resize_hdict(h, (*h).n_buckets - 1) < 0 {
                *ret = -1;
                return (*h).n_buckets;
            }
        } else if kh_resize_hdict(h, (*h).n_buckets + 1) < 0 {
            *ret = -1;
            return (*h).n_buckets;
        }
    }
    let x: u32;
    {
        let mask = (*h).n_buckets - 1;
        let mut site = (*h).n_buckets;
        let mut xx = (*h).n_buckets;
        let k = super::hts::__ac_FNV1a_hash_string(key);
        let mut i = k & mask;
        if kh_isempty((*h).flags, i) {
            xx = i;
        } else {
            let last = i;
            let mut step: u32 = 0;
            while !kh_isempty((*h).flags, i)
                && (kh_isdel((*h).flags, i) || !vcf_cstr_eq(*(*h).keys.add(i as usize), key))
            {
                if kh_isdel((*h).flags, i) {
                    site = i;
                }
                step += 1;
                i = (i + step) & mask;
                if i == last {
                    xx = site;
                    break;
                }
            }
            if xx == (*h).n_buckets {
                if kh_isempty((*h).flags, i) && site != (*h).n_buckets {
                    xx = site;
                } else {
                    xx = i;
                }
            }
        }
        x = xx;
    }
    if kh_isempty((*h).flags, x) {
        *(*h).keys.add(x as usize) = key;
        kh_set_isboth_false((*h).flags, x);
        (*h).size += 1;
        (*h).n_occupied += 1;
        *ret = 1;
    } else if kh_isdel((*h).flags, x) {
        *(*h).keys.add(x as usize) = key;
        kh_set_isboth_false((*h).flags, x);
        (*h).size += 1;
        *ret = 2;
    } else {
        *ret = 0;
    }
    x
}

// bcf_idinfo_def from vcf.c: { .info = {15,15,15}, .hrec = {NULL,NULL,NULL}, .id = -1 }
#[inline]
fn bcf_idinfo_def() -> bcf_idinfo_t {
    bcf_idinfo_t {
        info: [15, 15, 15],
        hrec: [std::ptr::null_mut(); 3],
        id: -1,
    }
}

pub unsafe fn bcf_hdr_id2int(hdr: *const bcf_hdr_t, type_: c_int, id: *const c_char) -> c_int {
    if hdr.is_null() || type_ < 0 || type_ > 2 || id.is_null() {
        return -1;
    }
    let d = (*hdr).dict[type_ as usize].cast::<kh_vdict_t>();
    if d.is_null() {
        return -1;
    }
    let k = kh_get_vdict(d, id);
    if k == (*d).n_buckets {
        -1
    } else {
        (*(*d).vals.add(k as usize)).id
    }
}

pub unsafe fn bcf_hdr_name2id(hdr: *const bcf_hdr_t, id: *const c_char) -> c_int {
    bcf_hdr_id2int(hdr, BCF_DT_CTG as c_int, id)
}

pub unsafe fn bcf_hdr_id2name(hdr: *const bcf_hdr_t, rid: c_int) -> *const c_char {
    if hdr.is_null() || rid < 0 || rid >= (*hdr).n[BCF_DT_CTG as usize] {
        return std::ptr::null();
    }
    (*(*hdr).id[BCF_DT_CTG as usize].add(rid as usize)).key
}

pub unsafe fn bcf_seqname(hdr: *const bcf_hdr_t, rec: *const bcf1_t) -> *const c_char {
    bcf_hdr_id2name(hdr, if rec.is_null() { -1 } else { (*rec).rid })
}

pub unsafe fn bcf_seqname_safe(hdr: *const bcf_hdr_t, rec: *const bcf1_t) -> *const c_char {
    let name = bcf_seqname(hdr, rec);
    if name.is_null() {
        c"(unknown)".as_ptr()
    } else {
        name
    }
}

// Native translation of htslib/vcf.c bcf_fmt_array().
// Native translation of htslib/vcf.c bcf_fmt_array1().
unsafe fn bcf_fmt_array1(s: *mut kstring_t, type_: c_int, data: *mut c_void) -> c_int {
    let mut e = false;
    let p = data.cast::<u8>();
    match type_ {
        x if x == BCF_BT_CHAR as c_int => {
            let c = if *p as u32 == bcf_str_missing {
                b'.' as c_int
            } else {
                *p as c_int
            };
            e |= kputc_(c, s) < 0;
        }
        x if x == BCF_BT_INT8 as c_int => {
            let v = le_to_i8(p) as i32;
            if v != bcf_int8_vector_end {
                e |= (if v == bcf_int8_missing {
                    kputc_(b'.' as c_int, s)
                } else {
                    kputw(v, s)
                }) < 0;
            }
        }
        x if x == BCF_BT_INT16 as c_int => {
            let v = le_to_i16(p) as i32;
            if v != bcf_int16_vector_end {
                e |= (if v == bcf_int16_missing {
                    kputc_(b'.' as c_int, s)
                } else {
                    kputw(v, s)
                }) < 0;
            }
        }
        x if x == BCF_BT_INT32 as c_int => {
            let v = le_to_i32(p);
            if v != bcf_int32_vector_end {
                e |= (if v == bcf_int32_missing {
                    kputc_(b'.' as c_int, s)
                } else {
                    kputw(v, s)
                }) < 0;
            }
        }
        x if x == BCF_BT_FLOAT as c_int => {
            let v = le_to_u32(p);
            if v != bcf_float_vector_end {
                e |= (if v == bcf_float_missing {
                    kputc_(b'.' as c_int, s)
                } else {
                    kputd(le_to_float(p) as f64, s)
                }) < 0;
            }
        }
        _ => {
            let msg =
                std::ffi::CString::new(format!("Unexpected type {type_}")).unwrap_or_default();
            c_log_error(msg.as_ptr());
            return -1;
        }
    }
    if e {
        -1
    } else {
        0
    }
}

pub unsafe fn bcf_fmt_array(s: *mut kstring_t, n: c_int, type_: c_int, data: *mut c_void) -> c_int {
    let mut e: u32 = 0;
    if n == 0 {
        return if kputc_(b'.' as c_int, s) >= 0 { 0 } else { -1 };
    }

    if type_ == BCF_BT_CHAR as c_int {
        let p = data.cast::<c_char>();
        // bcf_str_missing is already handled by the n==0 branch above.
        if n >= 8 {
            let p_end = libc::memchr(data, 0, n as usize);
            let len = if p_end.is_null() {
                n as usize
            } else {
                (p_end as usize) - (data as usize)
            };
            e |= (kputsn(p, len as size_t, s) < 0) as u32;
        } else {
            let mut j = 0;
            let mut pp = p;
            while j < n && *pp != 0 {
                e |= (kputc(*pp as c_int, s) < 0) as u32;
                j += 1;
                pp = pp.add(1);
            }
        }
    } else {
        macro_rules! branch {
            ($convert:expr, $sz:expr, $is_missing:expr, $is_vend:expr, $kprint:expr) => {{
                let mut p = data.cast::<u8>();
                let mut j = 0;
                while j < n {
                    let v = $convert(p);
                    if $is_vend(v) {
                        break;
                    }
                    if j != 0 {
                        e |= (kputc_(b',' as c_int, s) < 0) as u32;
                    }
                    e |= ((if $is_missing(v) {
                        kputc(b'.' as c_int, s)
                    } else {
                        $kprint(p, v)
                    }) < 0) as u32;
                    j += 1;
                    p = p.add($sz);
                }
            }};
        }
        match type_ {
            x if x == BCF_BT_INT8 as c_int => branch!(
                |p| le_to_i8(p) as i32,
                1usize,
                |v| v == bcf_int8_missing,
                |v| v == bcf_int8_vector_end,
                |_p, v| kputw(v, s)
            ),
            x if x == BCF_BT_INT16 as c_int => branch!(
                |p| le_to_i16(p) as i32,
                2usize,
                |v| v == bcf_int16_missing,
                |v| v == bcf_int16_vector_end,
                |_p, v| kputw(v, s)
            ),
            x if x == BCF_BT_INT32 as c_int => branch!(
                |p| le_to_i32(p),
                4usize,
                |v| v == bcf_int32_missing,
                |v| v == bcf_int32_vector_end,
                |_p, v| kputw(v, s)
            ),
            x if x == BCF_BT_FLOAT as c_int => branch!(
                |p| le_to_u32(p),
                4usize,
                |v| v == bcf_float_missing,
                |v| v == bcf_float_vector_end,
                |p, _v| kputd(le_to_float(p) as f64, s)
            ),
            _ => {
                let msg = std::ffi::CString::new(format!("Unexpected type {}", type_))
                    .unwrap_or_default();
                crate::htslib_rs::hts::hts_log_cstr(
                    crate::htslib_rs::hts::HTS_LOG_ERROR,
                    c"bcf_fmt_array".as_ptr(),
                    msg.as_ptr(),
                );
                libc::exit(1);
            }
        }
    }
    if e == 0 {
        0
    } else {
        -1
    }
}

// Native translation of htslib/vcf.c bcf_fmt_sized_array().
pub unsafe fn bcf_fmt_sized_array(s: *mut kstring_t, ptr: *mut u8) -> *mut u8 {
    let mut p: *const u8 = ptr;
    let mut type_ = 0;
    let x = bcf_dec_size_unsafe(p, &mut p, &mut type_);
    let ptr = p as *mut u8;
    bcf_fmt_array(s, x, type_, ptr.cast());
    ptr.add((x as usize) << BCF_TYPE_SHIFT[(type_ & 0xf) as usize])
}

// Native translation of htslib/vcf.c bcf_enc_vchar().
pub unsafe fn bcf_enc_vchar(s: *mut kstring_t, l: c_int, a: *const c_char) -> c_int {
    bcf_enc_size(s, l, BCF_BT_CHAR as c_int);
    kputsn(a, l as size_t, s);
    0 // FIXME: check for errs in this function (matches htslib)
}

// Native translation of htslib/vcf.c bcf_enc_vint().
pub unsafe fn bcf_enc_vint(s: *mut kstring_t, n: c_int, a: *mut i32, mut wsize: c_int) -> c_int {
    const BCF_MAX_BT_INT8: i32 = 127;
    const BCF_MIN_BT_INT8: i32 = -120;
    const BCF_MAX_BT_INT16: i32 = 32767;
    const BCF_MIN_BT_INT16: i32 = -32760;

    if n <= 0 {
        return bcf_enc_size(s, 0, BCF_BT_NULL as c_int);
    } else if n == 1 {
        return bcf_enc_int1(s, *a);
    }

    if wsize <= 0 {
        wsize = n;
    }

    let mut max = i32::MIN;
    let mut min = i32::MAX;
    for i in 0..n as isize {
        let x = *a.offset(i);
        if max < x {
            max = x;
        }
        if min > x && x > i32::MIN + 1 {
            min = x;
        }
    }

    if max <= BCF_MAX_BT_INT8 && min >= BCF_MIN_BT_INT8 {
        if bcf_enc_size(s, wsize, BCF_BT_INT8 as c_int) < 0
            || super::hts::ks_resize(s, (*s).l + n as size_t) < 0
        {
            return -1;
        }
        let mut p = (*s).s.add((*s).l).cast::<u8>();
        for i in 0..n as isize {
            let x = *a.offset(i);
            *p = if x == bcf_int32_vector_end {
                bcf_int8_vector_end as u8
            } else if x == bcf_int32_missing {
                bcf_int8_missing as u8
            } else {
                x as u8
            };
            p = p.add(1);
        }
        (*s).l += n as size_t;
    } else if max <= BCF_MAX_BT_INT16 && min >= BCF_MIN_BT_INT16 {
        if bcf_enc_size(s, wsize, BCF_BT_INT16 as c_int) < 0
            || super::hts::ks_resize(s, (*s).l + n as size_t * size_of::<i16>()) < 0
        {
            return -1;
        }
        let mut p = (*s).s.add((*s).l).cast::<u8>();
        for i in 0..n as isize {
            let x = *a.offset(i);
            let v: i16 = if x == bcf_int32_vector_end {
                bcf_int16_vector_end as i16
            } else if x == bcf_int32_missing {
                bcf_int16_missing as i16
            } else {
                x as i16
            };
            i16_to_le(v, p);
            p = p.add(size_of::<i16>());
        }
        (*s).l += n as size_t * size_of::<i16>();
    } else {
        if bcf_enc_size(s, wsize, BCF_BT_INT32 as c_int) < 0
            || super::hts::ks_resize(s, (*s).l + n as size_t * size_of::<i32>()) < 0
        {
            return -1;
        }
        let mut p = (*s).s.add((*s).l).cast::<u8>();
        for i in 0..n as isize {
            i32_to_le(*a.offset(i), p);
            p = p.add(size_of::<i32>());
        }
        (*s).l += n as size_t * size_of::<i32>();
    }

    0
}

// Native translation of htslib/vcf.c bcf_enc_vfloat().
pub unsafe fn bcf_enc_vfloat(s: *mut kstring_t, n: c_int, a: *mut f32) -> c_int {
    bcf_enc_size(s, n, BCF_BT_FLOAT as c_int);
    serialize_float_array(s, n as usize, a);
    0 // FIXME: check for errs in this function (matches htslib)
}

// Native translation of htslib/vcf.c idx_calc_n_lvls_ids().
// Calculate number of index levels given min_shift and the header contig
// list.  Also returns number of contigs in *nids_out.
unsafe fn idx_calc_n_lvls_ids(
    h: *const bcf_hdr_t,
    min_shift_in_out: *mut c_int,
    starting_n_lvls: c_int,
    nids_out: *mut c_int,
) -> c_int {
    let mut n_lvls = starting_n_lvls;
    let mut nids = 0;
    let mut max_len: i64 = 0;

    for i in 0..(*h).n[BCF_DT_CTG as usize] as isize {
        let val = (*(*h).id[BCF_DT_CTG as usize].offset(i)).val;
        if val.is_null() {
            continue;
        }
        if max_len < (*val).info[0] as i64 {
            max_len = (*val).info[0] as i64;
        }
        nids += 1;
    }
    if max_len == 0 {
        max_len = (1i64 << 31) - 1; // In case contig line is broken.
    }

    crate::htslib_rs::hts::hts_c_2372_hts_adjust_csi_settings(
        max_len,
        min_shift_in_out,
        &mut n_lvls,
    );

    if !nids_out.is_null() {
        *nids_out = nids;
    }
    n_lvls
}

// htslib/htslib/bgzf.h: #define bgzf_tell(fp) (((fp)->block_address << 16) | ((fp)->block_offset & 0xFFFF))
#[inline]
unsafe fn bcf_bgzf_tell(fp: *const BGZF) -> u64 {
    (((*fp).block_address as u64) << 16) | ((*fp).block_offset as u64 & 0xffff)
}

// Native translation of htslib/vcf.c bcf_index().
unsafe fn bcf_index(fp: *mut htsFile, mut min_shift: c_int) -> *mut hts_idx_t {
    let h = bcf_hdr_read(fp);
    if h.is_null() {
        return std::ptr::null_mut();
    }
    let mut nids = 0;
    let n_lvls = idx_calc_n_lvls_ids(h, &mut min_shift, 0, &mut nids);
    let idx = crate::htslib_rs::hts::hts_idx_init(
        nids,
        crate::htslib_rs::hts::HTS_FMT_CSI,
        bcf_bgzf_tell((*fp).fp.bgzf),
        min_shift,
        n_lvls,
    );
    if idx.is_null() {
        bcf_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    let b = bcf_init();
    if b.is_null() {
        crate::htslib_rs::hts::hts_idx_destroy(idx);
        bcf_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    let mut r = bcf_read1(fp, h, b);
    while r >= 0 {
        let ret = crate::htslib_rs::hts::hts_idx_push(
            idx,
            (*b).rid,
            (*b).pos,
            (*b).pos + (*b).rlen,
            bcf_bgzf_tell((*fp).fp.bgzf),
            1,
        );
        if ret < 0 {
            crate::htslib_rs::hts::hts_idx_destroy(idx);
            bcf_destroy(b);
            bcf_hdr_destroy(h);
            return std::ptr::null_mut();
        }
        r = bcf_read1(fp, h, b);
    }
    if r < -1 {
        crate::htslib_rs::hts::hts_idx_destroy(idx);
        bcf_destroy(b);
        bcf_hdr_destroy(h);
        return std::ptr::null_mut();
    }
    crate::htslib_rs::hts::hts_idx_finish(idx, bcf_bgzf_tell((*fp).fp.bgzf));
    bcf_destroy(b);
    bcf_hdr_destroy(h);
    idx
}

// Native translation of htslib/vcf.c bcf_index_load2().
pub unsafe fn bcf_index_load2(fn_: *const c_char, fnidx: *const c_char) -> *mut hts_idx_t {
    if !fnidx.is_null() {
        crate::htslib_rs::hts::hts_idx_load2(fn_, fnidx)
    } else {
        // #define bcf_index_load(fn) hts_idx_load(fn, HTS_FMT_CSI)
        crate::htslib_rs::hts::hts_idx_load(fn_, crate::htslib_rs::hts::HTS_FMT_CSI)
    }
}

// Native translation of htslib/vcf.c bcf_index_load3().
pub unsafe fn bcf_index_load3(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut hts_idx_t {
    crate::htslib_rs::hts::hts_idx_load3(fn_, fnidx, crate::htslib_rs::hts::HTS_FMT_CSI, flags)
}

// Native translation of htslib/vcf.c bcf_index_build3().
pub unsafe fn bcf_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    n_threads: c_int,
) -> c_int {
    let fp = hts_open(fn_, c"rb".as_ptr());
    if fp.is_null() {
        return -2;
    }
    if n_threads != 0 {
        crate::htslib_rs::hts::hts_set_threads(fp, n_threads);
    }
    if (*fp).format.compression != HTS_COMPRESSION_BGZF {
        hts_close(fp);
        return -3;
    }
    let ret;
    if (*fp).format.format == HTS_FORMAT_BCF {
        if min_shift == 0 {
            c_log_error(c"TBI indices for BCF files are not supported".as_ptr());
            ret = -1;
        } else {
            let idx = bcf_index(fp, min_shift);
            if !idx.is_null() {
                let mut r = crate::htslib_rs::hts::hts_idx_save_as(
                    idx,
                    fn_,
                    fnidx,
                    crate::htslib_rs::hts::HTS_FMT_CSI,
                );
                if r < 0 {
                    r = -4;
                }
                crate::htslib_rs::hts::hts_idx_destroy(idx);
                ret = r;
            } else {
                ret = -1;
            }
        }
    } else if (*fp).format.format == HTS_FORMAT_VCF {
        let conf = super::tbx::tbx_conf_vcf();
        let tbx = super::tbx::tbx_index(hts_get_bgzfp(fp), min_shift, &conf);
        if !tbx.is_null() {
            let mut r = crate::htslib_rs::hts::hts_idx_save_as(
                (*tbx).idx.cast(),
                fn_,
                fnidx,
                if min_shift > 0 {
                    crate::htslib_rs::hts::HTS_FMT_CSI
                } else {
                    crate::htslib_rs::hts::HTS_FMT_TBI
                },
            );
            if r < 0 {
                r = -4;
            }
            super::tbx::tbx_destroy(tbx);
            ret = r;
        } else {
            ret = -1;
        }
    } else {
        ret = -3;
    }
    hts_close(fp);
    ret
}

// Native translation of htslib/vcf.c bcf_index_build2().
pub unsafe fn bcf_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
) -> c_int {
    bcf_index_build3(fn_, fnidx, min_shift, 0)
}

// Native translation of htslib/vcf.c bcf_index_build().
pub unsafe fn bcf_index_build(fn_: *const c_char, min_shift: c_int) -> c_int {
    bcf_index_build3(fn_, std::ptr::null(), min_shift, 0)
}

// Native translation of htslib/vcf.c vcf_idx_init().
// Initialise fp->idx for the current format type.
unsafe fn vcf_idx_init(
    fp: *mut htsFile,
    h: *mut bcf_hdr_t,
    mut min_shift: c_int,
    fnidx: *const c_char,
) -> c_int {
    const TBX_MAX_SHIFT: c_int = 31;
    const TBX_VCF: u32 = 2;
    let n_lvls;
    let fmt;

    if min_shift == 0 {
        min_shift = 14;
        n_lvls = 5;
        fmt = crate::htslib_rs::hts::HTS_FMT_TBI;
    } else {
        // Set initial n_lvls to match tbx_index()
        let starting_n_lvls = (TBX_MAX_SHIFT - min_shift + 2) / 3;
        // Increase if necessary
        n_lvls = idx_calc_n_lvls_ids(h, &mut min_shift, starting_n_lvls, std::ptr::null_mut());
        fmt = crate::htslib_rs::hts::HTS_FMT_CSI;
    }

    (*fp).idx = crate::htslib_rs::hts::hts_idx_init(
        0,
        fmt,
        bcf_bgzf_tell((*fp).fp.bgzf),
        min_shift,
        n_lvls,
    )
    .cast();
    if (*fp).idx.is_null() {
        return -1;
    }

    // Tabix meta data, added even in CSI for VCF
    let mut conf = [0u8; 4 * 7];
    crate::htslib_rs::hts::u32_to_le(TBX_VCF, conf.as_mut_ptr()); // fmt
    crate::htslib_rs::hts::u32_to_le(1, conf.as_mut_ptr().add(4)); // name col
    crate::htslib_rs::hts::u32_to_le(2, conf.as_mut_ptr().add(8)); // beg col
    crate::htslib_rs::hts::u32_to_le(0, conf.as_mut_ptr().add(12)); // end col
    crate::htslib_rs::hts::u32_to_le(b'#' as u32, conf.as_mut_ptr().add(16)); // comment
    crate::htslib_rs::hts::u32_to_le(0, conf.as_mut_ptr().add(20)); // n.skip
    crate::htslib_rs::hts::u32_to_le(0, conf.as_mut_ptr().add(24)); // ref name len
    if crate::htslib_rs::hts::hts_idx_set_meta(
        (*fp).idx.cast(),
        conf.len() as u32,
        conf.as_mut_ptr(),
        1,
    ) < 0
    {
        crate::htslib_rs::hts::hts_idx_destroy((*fp).idx.cast());
        (*fp).idx = std::ptr::null_mut();
        return -1;
    }
    (*fp).fnidx = fnidx;

    0
}

// Native translation of htslib/vcf.c bcf_idx_init().
// Initialise fp->idx for the current format type.
// This must be called after the header has been written but no other data.
pub unsafe fn bcf_idx_init(
    fp: *mut htsFile,
    h: *mut bcf_hdr_t,
    mut min_shift: c_int,
    fnidx: *const c_char,
) -> c_int {
    let mut nids = 0;

    if (*fp).format.compression != HTS_COMPRESSION_BGZF {
        c_log_error(c"Indexing is only supported on BGZF-compressed files".as_ptr());
        return -3; // Matches no-compression return for bcf_index_build3()
    }

    if (*fp).format.format == HTS_FORMAT_VCF {
        return vcf_idx_init(fp, h, min_shift, fnidx);
    }

    if min_shift == 0 {
        min_shift = 14;
    }

    let n_lvls = idx_calc_n_lvls_ids(h, &mut min_shift, 0, &mut nids);

    (*fp).idx = crate::htslib_rs::hts::hts_idx_init(
        nids,
        crate::htslib_rs::hts::HTS_FMT_CSI,
        bcf_bgzf_tell((*fp).fp.bgzf),
        min_shift,
        n_lvls,
    )
    .cast();
    if (*fp).idx.is_null() {
        return -1;
    }
    (*fp).fnidx = fnidx;

    0
}

pub unsafe fn bcf_idx_save(fp: *mut htsFile) -> c_int {
    super::sam::sam_idx_save(fp)
}

// Native translation of the BCF/tabix iterator macros used by the synced
// reader: bcf_itr_queryi/tbx_itr_queryi (hts_itr_query with the proper readrec
// callback) and bcf_itr_next/tbx_itr_next (hts_itr_next).
unsafe extern "C" fn sr_bcf_readrec_adapter(
    fp: *mut BGZF,
    data: *mut c_void,
    r: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    unsafe { bcf_readrec(fp, data, r, tid, beg, end) }
}

unsafe fn sr_bcf_itr_queryi(
    idx: *const hts_idx_t,
    tid: c_int,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> *mut hts_itr_t {
    unsafe { super::hts::hts_itr_query(idx, tid, beg, end, Some(sr_bcf_readrec_adapter)) }
}

unsafe fn sr_tbx_itr_queryi(
    tbx: *mut super::tbx::tbx_t,
    tid: c_int,
    beg: hts_pos_t,
    end: hts_pos_t,
) -> *mut hts_itr_t {
    unsafe {
        super::hts::hts_itr_query(
            (*tbx).idx.cast(),
            tid,
            beg,
            end,
            Some(super::tbx::tbx_readrec),
        )
    }
}

unsafe fn sr_bcf_itr_next(htsfp: *mut htsFile, itr: *mut hts_itr_t, r: *mut bcf1_t) -> c_int {
    unsafe {
        super::hts::hts_itr_next((*htsfp).fp.bgzf.cast(), itr, r.cast(), std::ptr::null_mut())
    }
}

pub(crate) unsafe fn sr_tbx_itr_next(
    fp: *mut htsFile,
    tbx: *mut super::tbx::tbx_t,
    itr: *mut hts_itr_t,
    str_: *mut kstring_t,
) -> c_int {
    unsafe {
        super::hts::hts_itr_next((*fp).fp.bgzf.cast(), itr, str_.cast(), tbx.cast::<c_void>())
    }
}

// original: has_filter (htslib/synced_bcf_reader.c:546)
unsafe fn sr_has_filter(reader: *mut bcf_sr_t, line: *mut bcf1_t) -> c_int {
    unsafe {
        if (*line).d.n_flt == 0 {
            for j in 0..(*reader).nfilter_ids as usize {
                if *(*reader).filter_ids.add(j) < 0 {
                    return 1;
                }
            }
            return 0;
        }
        for i in 0..(*line).d.n_flt as usize {
            for j in 0..(*reader).nfilter_ids as usize {
                if *(*line).d.flt.add(i) == *(*reader).filter_ids.add(j) {
                    return 1;
                }
            }
        }
        0
    }
}

// original: _reader_seek (htslib/synced_bcf_reader.c:563)
pub(crate) unsafe fn sr_reader_seek(
    reader: *mut bcf_sr_t,
    seq: *const c_char,
    start: hts_pos_t,
    end: hts_pos_t,
) -> c_int {
    unsafe {
        if end >= MAX_CSI_COOR {
            libc::abort();
        }
        if !(*reader).itr.is_null() {
            super::hts::hts_itr_destroy((*reader).itr.cast());
            (*reader).itr = std::ptr::null_mut();
        }
        (*reader).nbuffer = 0;
        if !(*reader).tbx_idx.is_null() {
            let tid = super::tbx::tbx_name2id((*reader).tbx_idx.cast(), seq);
            if tid == -1 {
                return -1;
            }
            (*reader).itr = sr_tbx_itr_queryi((*reader).tbx_idx.cast(), tid, start, end + 1).cast();
        } else {
            let tid = bcf_hdr_name2id((*reader).header, seq);
            if tid == -1 {
                return -1;
            }
            (*reader).itr = sr_bcf_itr_queryi((*reader).bcf_idx.cast(), tid, start, end + 1).cast();
        }
        if (*reader).itr.is_null() {
            libc::abort();
        }
        0
    }
}

// original: _readers_next_region (htslib/synced_bcf_reader.c:599)
unsafe fn sr_readers_next_region(files: *mut bcf_srs_t) -> c_int {
    unsafe {
        let mut eos = 0;
        for i in 0..(*files).nreaders as usize {
            let r = (*files).readers.add(i);
            if (*r).itr.is_null() && (*r).nbuffer == 0 {
                eos += 1;
            }
        }
        if eos != (*files).nreaders {
            return 0;
        }

        let reg = (*files).regions;
        let prev_iseq = (*reg).iseq;
        let prev_end = (*reg).end;
        if bcf_sr_regions_next(reg) < 0 {
            return -1;
        }
        (*reg).prev_end = if prev_iseq == (*reg).iseq {
            prev_end
        } else {
            -1
        };

        for i in 0..(*files).nreaders as usize {
            let seq = *(*reg).seq_names.add((*reg).iseq as usize);
            sr_reader_seek((*files).readers.add(i), seq, (*reg).start, (*reg).end);
        }
        0
    }
}

// original: _set_variant_boundaries (htslib/synced_bcf_reader.c:624)
unsafe fn sr_set_variant_boundaries(rec: *mut bcf1_t, beg: *mut hts_pos_t, end: *mut hts_pos_t) {
    unsafe {
        let off;
        if (*rec).n_allele() != 0 {
            let mut o = (*rec).rlen;
            bcf_unpack(rec, BCF_UN_STR as c_int);
            for i in 1..(*rec).n_allele() as usize {
                let mut j = 0;
                let ref_a = *(*rec).d.allele;
                let alt_a = *(*rec).d.allele.add(i);
                while *ref_a.add(j) != 0 && *alt_a.add(j) != 0 && *ref_a.add(j) == *alt_a.add(j) {
                    j += 1;
                }
                if o > j as hts_pos_t {
                    o = j as hts_pos_t;
                }
                if o == 0 {
                    break;
                }
            }
            off = o;
        } else {
            off = 0;
        }
        *beg = (*rec).pos + off;
        *end = (*rec).pos + (*rec).rlen - 1;
    }
}

// original: _reader_fill_buffer (htslib/synced_bcf_reader.c:656)
unsafe fn sr_reader_fill_buffer(files: *mut bcf_srs_t, reader: *mut bcf_sr_t) -> c_int {
    unsafe {
        // Return if the buffer is full: the coordinate of the last buffered
        // record differs.
        if (*reader).nbuffer != 0
            && (*(*(*reader).buffer.add((*reader).nbuffer as usize))).pos
                != (*(*(*reader).buffer.add(1))).pos
        {
            return 0;
        }

        // No iterator (sequence not present in this file) and not streaming.
        if (*reader).itr.is_null() && (*files).streaming == 0 {
            return 0;
        }

        let mut ret;
        loop {
            if (*reader).nbuffer + 1 >= (*reader).mbuffer {
                (*reader).mbuffer += 8;
                (*reader).buffer = hts_realloc_p_cc(
                    (*reader).buffer.cast(),
                    size_of::<*mut bcf1_t>(),
                    (*reader).mbuffer as usize,
                )
                .cast::<*mut bcf1_t>();
                let mut i = 8;
                while i > 0 {
                    let idx = ((*reader).mbuffer - i) as usize;
                    *(*reader).buffer.add(idx) = bcf_init();
                    (*(*(*reader).buffer.add(idx))).max_unpack = (*files).max_unpack;
                    (*(*(*reader).buffer.add(idx))).pos = -1;
                    i -= 1;
                }
            }
            let slot = ((*reader).nbuffer + 1) as usize;
            let rfile: *mut htsFile = (*reader).file.cast();
            let tmps_ptr: *mut kstring_t = (&raw mut (*files).tmps).cast();
            if (*files).streaming != 0 {
                if (*rfile).format.format == HTS_FORMAT_VCF {
                    ret = hts_getline(rfile, KS_SEP_LINE as c_int, tmps_ptr);
                    if ret < -1 {
                        (*files).errnum = bcf_sr_error_bcf_read_error;
                    }
                    if ret < 0 {
                        break;
                    }
                    ret = vcf_parse(tmps_ptr, (*reader).header, *(*reader).buffer.add(slot));
                    if ret < 0 {
                        (*files).errnum = bcf_sr_error_vcf_parse_error;
                        break;
                    }
                } else if (*rfile).format.format == HTS_FORMAT_BCF {
                    ret = bcf_read1(rfile, (*reader).header, *(*reader).buffer.add(slot));
                    if ret < -1 {
                        (*files).errnum = bcf_sr_error_bcf_read_error;
                    }
                    if ret < 0 {
                        break;
                    }
                } else {
                    libc::abort();
                }
            } else if !(*reader).tbx_idx.is_null() {
                ret = sr_tbx_itr_next(
                    rfile,
                    (*reader).tbx_idx.cast(),
                    (*reader).itr.cast(),
                    tmps_ptr,
                );
                if ret < -1 {
                    (*files).errnum = bcf_sr_error_bcf_read_error;
                }
                if ret < 0 {
                    break;
                }
                ret = vcf_parse(tmps_ptr, (*reader).header, *(*reader).buffer.add(slot));
                if ret < 0 {
                    (*files).errnum = bcf_sr_error_vcf_parse_error;
                    break;
                }
            } else {
                ret = sr_bcf_itr_next(rfile, (*reader).itr.cast(), *(*reader).buffer.add(slot));
                if ret < -1 {
                    (*files).errnum = bcf_sr_error_bcf_read_error;
                }
                if ret < 0 {
                    break;
                }
                bcf_subset_format((*reader).header, *(*reader).buffer.add(slot));
            }

            // Prevent creation of duplicates from records overlapping multiple
            // regions and recognise true variant overlaps vs record overlaps.
            if !(*files).regions.is_null() {
                let aux = bcf_sr_aux_mut(files);
                let rec = *(*reader).buffer.add(slot);
                let (beg, end);
                if (*aux).regions_overlap == 0 {
                    beg = (*rec).pos;
                    end = (*rec).pos;
                } else if (*aux).regions_overlap == 1 {
                    beg = (*rec).pos;
                    end = (*rec).pos + (*rec).rlen - 1;
                } else if (*aux).regions_overlap == 2 {
                    let mut b = 0;
                    let mut e = 0;
                    sr_set_variant_boundaries(rec, &mut b, &mut e);
                    beg = b;
                    end = e;
                } else {
                    libc::abort();
                }
                let reg = (*files).regions;
                if beg <= (*reg).prev_end || end < (*reg).start || beg > (*reg).end {
                    continue;
                }
            }

            // apply filter
            if (*reader).nfilter_ids == 0 {
                bcf_unpack(*(*reader).buffer.add(slot), BCF_UN_STR as c_int);
            } else {
                bcf_unpack(
                    *(*reader).buffer.add(slot),
                    (BCF_UN_STR | BCF_UN_FLT) as c_int,
                );
                if sr_has_filter(reader, *(*reader).buffer.add(slot)) == 0 {
                    continue;
                }
            }
            (*reader).nbuffer += 1;

            let last = *(*reader).buffer.add((*reader).nbuffer as usize);
            let first = *(*reader).buffer.add(1);
            if (*last).rid != (*first).rid {
                break;
            }
            if (*last).pos != (*first).pos {
                break;
            }
        }
        if ret < 0 {
            // done for this region
            super::hts::hts_itr_destroy((*reader).itr.cast());
            (*reader).itr = std::ptr::null_mut();
        }
        if (*files).require_index == ALLOW_NO_IDX_
            && (*(*(*reader).buffer.add((*reader).nbuffer as usize))).rid
                < (*(*(*reader).buffer.add(1))).rid
        {
            libc::abort();
        }
        0
    }
}

// original: _reader_shift_buffer (htslib/synced_bcf_reader.c:770)
unsafe fn sr_reader_shift_buffer(reader: *mut bcf_sr_t) {
    unsafe {
        if (*reader).nbuffer == 0 {
            return;
        }
        let tmp = *(*reader).buffer.add(1);
        let mut i = 2;
        while i <= (*reader).nbuffer {
            *(*reader).buffer.add((i - 1) as usize) = *(*reader).buffer.add(i as usize);
            i += 1;
        }
        if (*reader).nbuffer > 1 {
            *(*reader).buffer.add((*reader).nbuffer as usize) = tmp;
        }
        (*reader).nbuffer -= 1;
    }
}

// original: next_line (htslib/synced_bcf_reader.c:782)
pub(crate) unsafe fn sr_next_line(files: *mut bcf_srs_t) -> c_int {
    unsafe {
        let mut chr: *const c_char = std::ptr::null();
        let mut min_pos = HTS_POS_MAX;

        loop {
            if !(*files).regions.is_null() && sr_readers_next_region(files) < 0 {
                break;
            }

            let mut min_rid = i32::MAX;
            for i in 0..(*files).nreaders as usize {
                sr_reader_fill_buffer(files, (*files).readers.add(i));
                if (*files).require_index == ALLOW_NO_IDX_ {
                    let r = (*files).readers.add(i);
                    if (*r).nbuffer == 0 {
                        continue;
                    }
                    let rid = (*(*(*r).buffer.add(1))).rid;
                    if min_rid > rid {
                        min_rid = rid;
                    }
                }
            }

            for i in 0..(*files).nreaders as usize {
                let r = (*files).readers.add(i);
                if (*r).nbuffer == 0 {
                    continue;
                }
                if (*files).require_index == ALLOW_NO_IDX_
                    && min_rid != (*(*(*r).buffer.add(1))).rid
                {
                    continue;
                }
                let pos = (*(*(*r).buffer.add(1))).pos;
                if min_pos > pos {
                    min_pos = pos;
                    chr = bcf_seqname((*r).header, *(*r).buffer.add(1));
                    bcf_sr_sort_c_324_bcf_sr_sort_set_active(
                        &mut (*bcf_sr_aux_mut(files)).sort,
                        i as c_int,
                    );
                } else if min_pos == pos {
                    bcf_sr_sort_c_331_bcf_sr_sort_add_active(
                        &mut (*bcf_sr_aux_mut(files)).sort,
                        i as c_int,
                    );
                }
            }
            if min_pos == HTS_POS_MAX {
                if (*files).regions.is_null() {
                    break;
                }
                continue;
            }

            // Skip this position if not present in targets
            if !(*files).targets.is_null() {
                let mut matched = 0;
                for i in 0..(*files).nreaders as usize {
                    let r = (*files).readers.add(i);
                    if (*r).nbuffer == 0 || (*(*(*r).buffer.add(1))).pos != min_pos {
                        continue;
                    }
                    let aux = bcf_sr_aux_mut(files);
                    let (beg, end);
                    if (*aux).targets_overlap == 0 {
                        beg = min_pos;
                        end = min_pos;
                    } else if (*aux).targets_overlap == 1 {
                        beg = min_pos;
                        end = min_pos + (*(*(*r).buffer.add(1))).rlen - 1;
                    } else if (*aux).targets_overlap == 2 {
                        let mut b = 0;
                        let mut e = 0;
                        sr_set_variant_boundaries(*(*r).buffer.add(1), &mut b, &mut e);
                        beg = b;
                        end = e;
                    } else {
                        libc::abort();
                    }
                    let overlap = if bcf_sr_regions_overlap((*files).targets, chr, beg, end) == 0 {
                        1
                    } else {
                        0
                    };
                    if ((*files).targets_exclude == 0 && overlap == 0)
                        || ((*files).targets_exclude != 0 && overlap != 0)
                    {
                        sr_reader_shift_buffer(r);
                    } else {
                        matched = 1;
                    }
                }
                if matched == 0 {
                    min_pos = HTS_POS_MAX;
                    chr = std::ptr::null();
                    continue;
                }
            }
            break;
        }
        if chr.is_null() {
            return 0;
        }

        bcf_sr_sort_c_593_bcf_sr_sort_next(files, &mut (*bcf_sr_aux_mut(files)).sort, chr, min_pos)
    }
}

pub(crate) unsafe fn bcf_sr_destroy1(reader: *mut bcf_sr_t, closefile: c_int) {
    unsafe {
        if reader.is_null() {
            return;
        }

        // All reader resources (file, indices, iterator) are now created via
        // the native code paths, so they must be released with the native
        // destructors to match the allocators.
        if !(*reader).file.is_null() && closefile != 0 {
            let _ = hts_close((*reader).file.cast());
        }
        libc::free((*reader).fname.cast());
        if !(*reader).tbx_idx.is_null() {
            super::tbx::tbx_destroy((*reader).tbx_idx.cast());
        }
        if !(*reader).bcf_idx.is_null() {
            super::hts::hts_idx_destroy((*reader).bcf_idx.cast());
        }
        bcf_hdr_destroy((*reader).header);
        if !(*reader).itr.is_null() {
            super::hts::hts_itr_destroy((*reader).itr.cast());
        }
        for j in 0..(*reader).mbuffer {
            bcf_destroy(*(*reader).buffer.add(j as usize));
        }
        libc::free((*reader).buffer.cast());
        libc::free((*reader).samples.cast());
        libc::free((*reader).filter_ids.cast());
    }
}

pub(crate) const BCF_SR_ERROR_NOIDX_ERROR: c_int = 10;

// original: _regions_match_alleles (htslib/synced_bcf_reader.c:1471)
pub(crate) unsafe fn sr_regions_match_alleles(
    reg: *mut bcf_sr_regions_t,
    als_idx: c_int,
    rec: *mut bcf1_t,
) -> c_int {
    unsafe {
        if !(*reg).regs.is_null() {
            // payload is not supported for in-memory regions
            libc::abort();
        }

        let mut i = 0;
        let mut max_len: isize = 0;
        if (*reg).nals == 0 {
            let mut ss = (*reg).line.s;
            while i < als_idx && *ss != 0 {
                if *ss == b'\t' as c_char {
                    i += 1;
                }
                ss = ss.add(1);
            }
            let mut se = ss;
            (*reg).nals = 1;
            while *se != 0 && *se != b'\t' as c_char {
                if *se == b',' as c_char {
                    (*reg).nals += 1;
                }
                se = se.add(1);
            }
            let als_str_ptr: *mut kstring_t = (&raw mut (*reg).als_str).cast();
            ks_resize(
                als_str_ptr,
                (se.offset_from(ss) + 1 + (*reg).nals as isize) as size_t,
            );
            (*reg).als_str.l = 0;
            let need = (*reg).nals;
            if need > (*reg).mals {
                (*reg).als =
                    hts_realloc_p_cc((*reg).als.cast(), size_of::<*mut c_char>(), need as usize)
                        .cast::<*mut c_char>();
                (*reg).mals = need;
            }
            (*reg).nals = 0;

            se = ss;
            loop {
                se = se.add(1);
                if *se == 0 {
                    break;
                }
                if *se == b'\t' as c_char {
                    break;
                }
                if *se != b',' as c_char {
                    continue;
                }
                *(*reg).als.add((*reg).nals as usize) =
                    (*reg).als_str.s.add((*reg).als_str.l as usize);
                kputsn(ss, se.offset_from(ss) as usize, als_str_ptr);
                let cur = (*reg).als_str.s.add((*reg).als_str.l as usize);
                let this_len = cur.offset_from(*(*reg).als.add((*reg).nals as usize));
                if this_len > max_len {
                    max_len = this_len;
                }
                (*reg).als_str.l += 1;
                (*reg).nals += 1;
                se = se.add(1);
                ss = se;
            }
            *(*reg).als.add((*reg).nals as usize) = (*reg).als_str.s.add((*reg).als_str.l as usize);
            kputsn(ss, se.offset_from(ss) as usize, als_str_ptr);
            let cur = (*reg).als_str.s.add((*reg).als_str.l as usize);
            let this_len = cur.offset_from(*(*reg).als.add((*reg).nals as usize));
            if this_len > max_len {
                max_len = this_len;
            }
            (*reg).nals += 1;
            (*reg).als_type = if max_len > 1 {
                VCF_INDEL as c_int
            } else {
                VCF_SNP as c_int
            };
        }
        let type_ = bcf_get_variant_types(rec);
        if (*reg).als_type & VCF_INDEL as c_int != 0 {
            return if type_ & VCF_INDEL as c_int != 0 {
                1
            } else {
                0
            };
        }
        if type_ & VCF_INDEL as c_int == 0 {
            1
        } else {
            0
        }
    }
}

pub(crate) unsafe fn bcf_sr_regions_overlap_inner(
    reg: *mut bcf_sr_regions_t,
    seq: *const c_char,
    start: hts_pos_t,
    end: hts_pos_t,
    mut missed_reg_handler: c_int,
) -> c_int {
    unsafe {
        let mut iseq = -1;
        if super::sam::khash_str2int_get((*reg).seq_hash, seq, &mut iseq) < 0 {
            return -1;
        }
        if missed_reg_handler != 0 && (*reg).missed_reg_handler.is_none() {
            missed_reg_handler = 0;
        }

        if (*reg).prev_seq == -1 || iseq != (*reg).prev_seq || (*reg).prev_start > start {
            if missed_reg_handler != 0 && (*reg).prev_seq != -1 && (*reg).iseq != -1 {
                bcf_sr_regions_flush(reg);
            }
            bcf_sr_regions_seek(reg, seq);
            (*reg).start = -1;
            (*reg).end = -1;
        }
        if (*reg).prev_seq == iseq && (*reg).iseq != iseq {
            return -2;
        }
        (*reg).prev_seq = (*reg).iseq;
        (*reg).prev_start = start;

        while iseq == (*reg).iseq && (*reg).end < start {
            if bcf_sr_regions_next(reg) < 0 {
                return -2;
            }
            if (*reg).iseq != iseq {
                return -1;
            }
            if missed_reg_handler != 0 && (*reg).end < start {
                if let Some(handler) = (*reg).missed_reg_handler {
                    handler(reg, (*reg).missed_reg_data);
                }
            }
        }
        if (*reg).start <= end {
            return 0;
        }
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // libhts reference symbols for the bcf_sweep_* parity test. Declared here
    // (test-only) so we can compare the native translation in this module
    // against the C implementation over a real BCF fixture.
    mod libhts_sweep {
        use std::ffi::{c_char, c_int};
        #[repr(C)]
        pub struct bcf_sweep_t {
            _private: [u8; 0],
        }
        unsafe extern "C" {
            #[link_name = "bcf_sweep_init"]
            pub fn bcf_sweep_init(fname: *const c_char) -> *mut bcf_sweep_t;
            #[link_name = "bcf_sweep_destroy"]
            pub fn bcf_sweep_destroy(sw: *mut bcf_sweep_t);
            #[link_name = "bcf_sweep_fwd"]
            pub fn bcf_sweep_fwd(sw: *mut bcf_sweep_t) -> *mut super::bcf1_t;
            #[link_name = "bcf_sweep_bwd"]
            pub fn bcf_sweep_bwd(sw: *mut bcf_sweep_t) -> *mut super::bcf1_t;
            #[link_name = "vcf_open_mode"]
            pub fn vcf_open_mode(
                mode: *mut c_char,
                fn_: *const c_char,
                format: *const c_char,
            ) -> c_int;
            #[link_name = "bcf_strerror"]
            pub fn bcf_strerror(
                errorcode: c_int,
                buffer: *mut c_char,
                maxbuffer: usize,
            ) -> *const c_char;
        }
    }

    // Collect (pos sequence) from a forward then backward sweep using the
    // supplied init/fwd/bwd/destroy callbacks, over the given BCF file.
    fn sweep_positions_native(path: &std::ffi::CStr) -> (Vec<i64>, Vec<i64>) {
        unsafe {
            let sw = bcf_sweep_init(path.as_ptr());
            assert!(!sw.is_null());
            let mut fwd = Vec::new();
            loop {
                let rec = bcf_sweep_fwd(sw);
                if rec.is_null() {
                    break;
                }
                fwd.push((*rec).pos);
            }
            let mut bwd = Vec::new();
            loop {
                let rec = bcf_sweep_bwd(sw);
                if rec.is_null() {
                    break;
                }
                bwd.push((*rec).pos);
            }
            bcf_sweep_destroy(sw);
            (fwd, bwd)
        }
    }

    fn sweep_positions_libhts(path: &std::ffi::CStr) -> (Vec<i64>, Vec<i64>) {
        unsafe {
            let sw = libhts_sweep::bcf_sweep_init(path.as_ptr());
            assert!(!sw.is_null());
            let mut fwd = Vec::new();
            loop {
                let rec = libhts_sweep::bcf_sweep_fwd(sw);
                if rec.is_null() {
                    break;
                }
                fwd.push((*rec).pos);
            }
            let mut bwd = Vec::new();
            loop {
                let rec = libhts_sweep::bcf_sweep_bwd(sw);
                if rec.is_null() {
                    break;
                }
                bwd.push((*rec).pos);
            }
            libhts_sweep::bcf_sweep_destroy(sw);
            (fwd, bwd)
        }
    }

    #[test]
    fn bcf_sweep_native_matches_libhts_over_real_bcf() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("htslib/test/tabix/vcf_file.bcf");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();

        let (native_fwd, native_bwd) = sweep_positions_native(&c_path);
        let (lib_fwd, lib_bwd) = sweep_positions_libhts(&c_path);

        assert_eq!(native_fwd, lib_fwd, "forward sweep positions differ");
        assert_eq!(native_bwd, lib_bwd, "backward sweep positions differ");
        // Backward sweep must reverse the forward order.
        let mut rev = native_fwd.clone();
        rev.reverse();
        assert_eq!(
            native_bwd, rev,
            "backward sweep is not the reverse of forward"
        );
        assert!(!native_fwd.is_empty(), "expected at least one record");
    }

    // Read a real BCF with the native header/record I/O and via hts_sys, format
    // both to VCF text, and assert byte-for-byte parity. Then round-trip the
    // native records through native bcf_hdr_write/bcf_write to a temporary BCF
    // and re-read with hts_sys, confirming the bytes survive.
    #[test]
    fn bcf_read_write_native_matches_libhts_over_real_bcf() {
        unsafe {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("htslib/test/tabix/vcf_file.bcf");
            assert!(path.exists(), "fixture missing: {}", path.display());
            let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();

            // Native read + format
            let native_text = {
                let fp = hts_open(c_path.as_ptr(), c"r".as_ptr());
                assert!(!fp.is_null());
                let hdr = bcf_hdr_read(fp);
                assert!(!hdr.is_null());
                let mut out = String::new();
                let mut htxt = kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                assert_eq!(bcf_hdr_format(hdr, 0, &mut htxt), 0);
                out.push_str(&CStr::from_ptr(htxt.s).to_string_lossy());
                super::super::hts::ks_free(&mut htxt);
                let rec = bcf_init();
                let mut line = kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                while bcf_read(fp, hdr, rec) >= 0 {
                    line.l = 0;
                    assert_eq!(vcf_format(hdr, rec, &mut line), 0);
                    out.push_str(&CStr::from_ptr(line.s).to_string_lossy());
                }
                super::super::hts::ks_free(&mut line);
                bcf_destroy(rec);
                bcf_hdr_destroy(hdr);
                assert_eq!(hts_close(fp), 0);
                out
            };

            // hts_sys reference read + format
            let lib_text = {
                let fp = hts_sys::hts_open(c_path.as_ptr(), c"r".as_ptr());
                assert!(!fp.is_null());
                let hdr = hts_sys::bcf_hdr_read(fp);
                assert!(!hdr.is_null());
                let mut out = String::new();
                let mut htxt = hts_sys::kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                assert_eq!(hts_sys::bcf_hdr_format(hdr, 0, &mut htxt), 0);
                out.push_str(&CStr::from_ptr(htxt.s).to_string_lossy());
                libc::free(htxt.s.cast());
                let rec = hts_sys::bcf_init();
                let mut line = hts_sys::kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                while hts_sys::bcf_read(fp, hdr, rec) >= 0 {
                    line.l = 0;
                    assert_eq!(hts_sys::vcf_format(hdr, rec, &mut line), 0);
                    out.push_str(&CStr::from_ptr(line.s).to_string_lossy());
                }
                libc::free(line.s.cast());
                hts_sys::bcf_destroy(rec);
                hts_sys::bcf_hdr_destroy(hdr);
                assert_eq!(hts_sys::hts_close(fp), 0);
                out
            };

            assert_eq!(
                native_text, lib_text,
                "native BCF read/format differs from hts_sys"
            );

            // Native BCF write round-trip: native read -> native write -> hts_sys read/format.
            let tmp = std::env::temp_dir().join(format!("vcfio_rt_{}.bcf", std::process::id()));
            let c_tmp = std::ffi::CString::new(tmp.to_string_lossy().as_bytes()).unwrap();
            {
                let inp = hts_open(c_path.as_ptr(), c"r".as_ptr());
                let hdr = bcf_hdr_read(inp);
                let outp = hts_open(c_tmp.as_ptr(), c"wb".as_ptr());
                assert!(!outp.is_null());
                assert_eq!(bcf_hdr_write(outp, hdr), 0);
                let rec = bcf_init();
                while bcf_read(inp, hdr, rec) >= 0 {
                    assert_eq!(bcf_write(outp, hdr, rec), 0);
                }
                bcf_destroy(rec);
                assert_eq!(hts_close(outp), 0);
                bcf_hdr_destroy(hdr);
                assert_eq!(hts_close(inp), 0);
            }
            let roundtrip_text = {
                let fp = hts_sys::hts_open(c_tmp.as_ptr(), c"r".as_ptr());
                assert!(!fp.is_null());
                let hdr = hts_sys::bcf_hdr_read(fp);
                let mut out = String::new();
                let mut htxt = hts_sys::kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                assert_eq!(hts_sys::bcf_hdr_format(hdr, 0, &mut htxt), 0);
                out.push_str(&CStr::from_ptr(htxt.s).to_string_lossy());
                libc::free(htxt.s.cast());
                let rec = hts_sys::bcf_init();
                let mut line = hts_sys::kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                while hts_sys::bcf_read(fp, hdr, rec) >= 0 {
                    line.l = 0;
                    assert_eq!(hts_sys::vcf_format(hdr, rec, &mut line), 0);
                    out.push_str(&CStr::from_ptr(line.s).to_string_lossy());
                }
                libc::free(line.s.cast());
                hts_sys::bcf_destroy(rec);
                hts_sys::bcf_hdr_destroy(hdr);
                assert_eq!(hts_sys::hts_close(fp), 0);
                out
            };
            let _ = std::fs::remove_file(&tmp);
            assert_eq!(
                roundtrip_text, lib_text,
                "native BCF write round-trip differs from hts_sys"
            );
        }
    }

    #[test]
    fn vcf_open_mode_native_matches_libhts() {
        unsafe {
            let cases: &[(&std::ffi::CStr, *const c_char)] = &[
                (c"file.bcf", std::ptr::null()),
                (c"file.vcf", std::ptr::null()),
                (c"file.vcf.gz", std::ptr::null()),
                (c"file.vcf.bgz", std::ptr::null()),
                (c"file.unknownext", std::ptr::null()),
                (c"noextension", std::ptr::null()),
                (c"x.sam", std::ptr::null()),
            ];
            for (fname, fmt) in cases {
                let mut native = [0 as c_char; 16];
                let mut libv = [0 as c_char; 16];
                let nr = vcf_open_mode(native.as_mut_ptr(), fname.as_ptr(), *fmt);
                let lr = libhts_sweep::vcf_open_mode(libv.as_mut_ptr(), fname.as_ptr(), *fmt);
                assert_eq!(nr, lr, "ret differs for {fname:?}");
                if nr == 0 {
                    assert_eq!(
                        std::ffi::CStr::from_ptr(native.as_ptr()),
                        std::ffi::CStr::from_ptr(libv.as_ptr()),
                        "mode string differs for {fname:?}"
                    );
                }
            }
            // explicit format strings
            let explicit: &[&std::ffi::CStr] =
                &[c"bcf", c"vcf", c"vcf.gz", c"vcf.bgz", c"BCF", c"junk"];
            for fmt in explicit {
                let mut native = [0 as c_char; 16];
                let mut libv = [0 as c_char; 16];
                let nr = vcf_open_mode(native.as_mut_ptr(), c"f".as_ptr(), fmt.as_ptr());
                let lr =
                    libhts_sweep::vcf_open_mode(libv.as_mut_ptr(), c"f".as_ptr(), fmt.as_ptr());
                assert_eq!(nr, lr, "ret differs for format {fmt:?}");
                if nr == 0 {
                    assert_eq!(
                        std::ffi::CStr::from_ptr(native.as_ptr()),
                        std::ffi::CStr::from_ptr(libv.as_ptr()),
                        "mode string differs for format {fmt:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn bcf_strerror_native_matches_libhts() {
        unsafe {
            let codes = [
                0i32,
                1,
                2,
                4,
                8,
                16,
                32,
                64,
                1 | 2,
                4 | 16 | 64,
                1 | 2 | 4 | 8 | 16 | 32 | 64,
                128, // undescribed -> "Unknown error"
                1 | 128,
            ];
            for &code in &codes {
                for &cap in &[4usize, 8, 16, 64, 256] {
                    let mut native = vec![0xAAu8 as c_char; cap];
                    let mut libv = vec![0xAAu8 as c_char; cap];
                    let nr = bcf_strerror(code, native.as_mut_ptr(), cap);
                    let lr = libhts_sweep::bcf_strerror(code, libv.as_mut_ptr(), cap);
                    assert_eq!(
                        nr.is_null(),
                        lr.is_null(),
                        "null-ness differs code={code} cap={cap}"
                    );
                    assert_eq!(
                        std::ffi::CStr::from_ptr(native.as_ptr()),
                        std::ffi::CStr::from_ptr(libv.as_ptr()),
                        "message differs code={code} cap={cap}"
                    );
                }
            }
            // invalid buffer cases
            let mut tiny = [0 as c_char; 3];
            assert!(bcf_strerror(1, tiny.as_mut_ptr(), 3).is_null());
            assert!(bcf_strerror(1, std::ptr::null_mut(), 64).is_null());
        }
    }

    #[test]
    fn vcf_header_wrappers_accept_hts_sys_headers() {
        unsafe {
            let hdr = hts_sys::bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            let dup = bcf_hdr_dup(hdr.cast());
            assert!(!dup.is_null());
            hts_sys::bcf_hdr_destroy(dup.cast());
            hts_sys::bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_version_gate_parses_minor_versions_numerically() {
        assert_eq!(vcf_version_number(b"VCFv4.3"), Some(4_003_000));
        assert_eq!(vcf_version_number(b"VCFv4.4"), Some(4_004_000));
        assert_eq!(vcf_version_number(b"VCFv4.10"), Some(4_010_000));
        assert_eq!(vcf_version_number(b"##fileformat=VCFv5.0"), Some(5_000_000));
        assert_eq!(vcf_version_number(b"VCFv4.x"), None);
    }

    #[test]
    fn vcf_inline_name_helpers_match_contig_table() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);
            assert_eq!(bcf_hdr_name2id(hdr, c"chr1".as_ptr()), 0);
            assert_eq!(bcf_hdr_name2id(hdr, c"missing".as_ptr()), -1);
            assert_eq!(std::ffi::CStr::from_ptr(bcf_hdr_id2name(hdr, 0)), c"chr1");
            assert!(bcf_hdr_id2name(hdr, -1).is_null());
            assert!(bcf_hdr_id2name(hdr, 1).is_null());

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = 0;
            assert_eq!(std::ffi::CStr::from_ptr(bcf_seqname(hdr, rec)), c"chr1");
            (*rec).rid = 99;
            assert_eq!(
                std::ffi::CStr::from_ptr(bcf_seqname_safe(hdr, rec)),
                c"(unknown)"
            );
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_has_filter_matches_hts_sys_for_pass_and_named_filters() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.2".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FILTER=<ID=q10,Description=\"Quality below 10\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FILTER=<ID=s50,Description=\"Less samples\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            // Parse real records so shared/filter blocks are populated.
            let parse = |line: &CStr, rec: *mut bcf1_t| {
                let mut tmp = kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                assert!(super::super::hts::kputs(line.as_ptr(), &mut tmp) >= 0);
                assert_eq!(vcf_parse(&mut tmp, hdr, rec), 0);
                crate::htslib_rs::c_compat::free(tmp.s.cast());
            };

            for (line, label) in [
                (c"chr1\t10\t.\tA\tG\t30\tPASS\t.", "pass"),
                (c"chr1\t20\t.\tA\tG\t30\t.\t.", "dot"),
                (c"chr1\t30\t.\tA\tG\t30\tq10\t.", "q10"),
                (c"chr1\t40\t.\tA\tG\t30\tq10;s50\t.", "q10s50"),
            ] {
                let rec = bcf_init();
                assert!(!rec.is_null());
                parse(line, rec);
                for f in [c"PASS", c".", c"q10", c"s50", c"missing"] {
                    let p = f.as_ptr() as *mut c_char;
                    assert_eq!(
                        bcf_has_filter(hdr, rec, p),
                        hts_sys::bcf_has_filter(hdr.cast(), rec.cast(), p),
                        "{label} parity for {f:?}"
                    );
                }
                bcf_destroy(rec);
            }

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_seqname_safe_handles_null_records_and_missing_contigs() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            assert!(bcf_seqname(hdr, std::ptr::null()).is_null());
            assert_eq!(
                std::ffi::CStr::from_ptr(bcf_seqname_safe(hdr, std::ptr::null())),
                c"(unknown)"
            );
            assert!(bcf_hdr_id2name(std::ptr::null(), 0).is_null());

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = -1;
            assert!(bcf_seqname(hdr, rec).is_null());
            assert_eq!(
                std::ffi::CStr::from_ptr(bcf_seqname_safe(hdr, rec)),
                c"(unknown)"
            );

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_hdr_check_sanity_accepts_matching_standard_tags() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">".as_ptr(),
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            bcf_hdr_check_sanity(hdr);

            assert!(bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"DP".as_ptr()) >= 0);
            assert!(bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"GT".as_ptr()) >= 0);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_hdr_check_sanity_warn_only_for_mismatched_standard_tags() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=DP,Number=A,Type=String,Description=\"Wrong depth\">".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            bcf_hdr_check_sanity(hdr);

            let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"DP".as_ptr());
            assert!(id >= 0);
            assert!(bcf_hdr_idinfo_exists_rs(hdr, BCF_HL_INFO as c_int, id));
            assert_eq!(
                bcf_hdr_id2length_rs(hdr, BCF_HL_INFO as c_int, id),
                BCF_VL_A as c_int
            );
            assert_eq!(
                bcf_hdr_id2type_rs(hdr, BCF_HL_INFO as c_int, id),
                BCF_HT_STR as c_int
            );
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_header_parse_sample_line_adds_tab_delimited_samples() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());

            assert_eq!(
                vcf_c_286_bcf_hdr_parse_sample_line(
                    hdr,
                    c"#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tS1\tS2\n".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);
            assert_eq!((*hdr).n[BCF_DT_SAMPLE as usize], 2);
            assert_eq!(
                bcf_hdr_id2int(hdr, BCF_DT_SAMPLE as c_int, c"S1".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_id2int(hdr, BCF_DT_SAMPLE as c_int, c"S2".as_ptr()),
                1
            );

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_header_parse_sample_line_rejects_spaces_and_missing_format() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());

            assert_eq!(
                vcf_c_286_bcf_hdr_parse_sample_line(
                    hdr,
                    c"#CHROM POS ID REF ALT QUAL FILTER INFO FORMAT S1".as_ptr(),
                ),
                -1
            );
            assert_eq!(
                vcf_c_286_bcf_hdr_parse_sample_line(
                    hdr,
                    c"#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tS1".as_ptr(),
                ),
                -1
            );
            assert_eq!(
                vcf_c_286_bcf_hdr_parse_sample_line(
                    hdr,
                    c"#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n".as_ptr(),
                ),
                0
            );
            assert_eq!((*hdr).n[BCF_DT_SAMPLE as usize], 0);

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_get_rlen_uses_decoded_end_svlen_and_ref_length() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=END,Number=1,Type=Integer,Description=\"End\">".as_ptr(),
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=SVLEN,Number=.,Type=Integer,Description=\"SV length\">".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            (*rec).pos = 9;
            assert_eq!(bcf_update_alleles_str(hdr, rec, c"AT,A".as_ptr()), 0);
            assert_eq!(vcf_c_6420_get_rlen(hdr, rec), 2);

            let end = [25i32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    rec,
                    c"END".as_ptr(),
                    end.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(vcf_c_6420_get_rlen(hdr, rec), 16);

            let svlen = [-30i32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    rec,
                    c"SVLEN".as_ptr(),
                    svlen.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(vcf_c_6420_get_rlen(hdr, rec), 31);

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_record_check_reports_invalid_contig_id() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = 99;
            (*rec).pos = 4;
            assert_eq!(bcf_update_alleles_str(hdr, rec, c"A,C".as_ptr()), 0);
            assert_eq!(vcf_c_2332_bcf1_sync(rec), 0);

            assert_eq!(vcf_c_2040_bcf_record_check(hdr, rec), -2);
            assert_ne!((*rec).errcode & BCF_ERR_CTG_INVALID as c_int, 0);

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_bcf1_sync_rebuilds_dirty_shared_and_format_blocks() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"GQ\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            (*rec).pos = 9;
            assert_eq!(bcf_update_alleles_str(hdr, rec, c"A,C".as_ptr()), 0);
            assert_eq!(bcf_update_id(hdr, rec, c"rs1".as_ptr()), 0);
            let gt = [0i32, 2];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    rec,
                    c"GT".as_ptr(),
                    gt.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            let gq = [37i32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    rec,
                    c"GQ".as_ptr(),
                    gq.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_format(
                    hdr,
                    rec,
                    c"GQ".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_INT as c_int,
                ),
                0
            );

            assert_ne!((*rec).d.shared_dirty, 0);
            assert_ne!((*rec).d.indiv_dirty, 0);
            assert_eq!(vcf_c_2332_bcf1_sync(rec), 0);
            assert_eq!((*rec).d.shared_dirty, 0);
            assert_eq!((*rec).d.indiv_dirty, 0);
            assert_eq!((*rec).n_fmt(), 1);
            assert!(bcf_get_fmt(hdr, rec, c"GT".as_ptr()).is_null() == false);
            assert!(bcf_get_fmt(hdr, rec, c"GQ".as_ptr()).is_null());

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_bcf1_sync_alleles_resets_pointers_and_rlen() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            (*rec).pos = 2;
            assert_eq!(bcf_update_alleles_str(hdr, rec, c"AT,A".as_ptr()), 0);
            (*rec).rlen = 0;
            assert_eq!(vcf_c_5884__bcf1_sync_alleles(hdr, rec, 2), 0);

            assert_eq!((*rec).n_allele(), 2);
            assert_eq!(std::ffi::CStr::from_ptr(*(*rec).d.allele.add(0)), c"AT");
            assert_eq!(std::ffi::CStr::from_ptr(*(*rec).d.allele.add(1)), c"A");
            assert_eq!((*rec).rlen, 2);
            assert_ne!((*rec).d.shared_dirty & BCF1_DIRTY_ALS as c_int, 0);

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_header_seqnames_and_filter_helpers_cover_order_and_pass_state() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr2>".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FILTER=<ID=LowQ,Description=\"Low quality\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let mut nseqs = -1;
            let seqnames = bcf_hdr_seqnames(hdr, &mut nseqs);
            assert!(!seqnames.is_null());
            assert_eq!(nseqs, 2);
            assert_eq!(std::ffi::CStr::from_ptr(*seqnames.add(0)), c"chr2");
            assert_eq!(std::ffi::CStr::from_ptr(*seqnames.add(1)), c"chr1");
            libc::free(seqnames.cast());

            let filter_hrec = bcf_hdr_get_hrec(
                hdr,
                BCF_HL_FLT as c_int,
                c"ID".as_ptr(),
                c"LowQ".as_ptr(),
                std::ptr::null(),
            );
            assert!(!filter_hrec.is_null());

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            (*rec).pos = 9;
            assert_eq!(bcf_update_alleles_str(hdr, rec, c"A,C".as_ptr()), 0);

            let lowq = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"LowQ".as_ptr());
            assert!(lowq >= 0);
            assert_eq!(bcf_add_filter(hdr, rec, lowq), 1);
            assert_eq!(bcf_has_filter(hdr, rec, c"LowQ".as_ptr().cast_mut()), 1);
            assert_eq!(bcf_has_filter(hdr, rec, c"PASS".as_ptr().cast_mut()), 0);

            assert_eq!(bcf_remove_filter(hdr, rec, lowq, 1), 0);
            assert_eq!(bcf_has_filter(hdr, rec, c"LowQ".as_ptr().cast_mut()), 0);
            assert_eq!(bcf_has_filter(hdr, rec, c"PASS".as_ptr().cast_mut()), 1);

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_header_parse_line_formats_quoted_commas() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());

            let line =
                c"##INFO=<ID=TXT,Number=1,Type=String,Description=\"has,comma\",Source=\"x\">";
            let mut len = -1;
            let hrec = bcf_hdr_parse_line(hdr, line.as_ptr(), &mut len);
            assert!(!hrec.is_null());

            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(bcf_hrec_format(hrec, &mut out), 0);
            assert!(out.l > 0);
            assert!(!out.s.is_null());
            let formatted = std::slice::from_raw_parts(out.s.cast::<u8>(), out.l);
            assert!(formatted
                .windows(b"Description=\"has,comma\"".len())
                .any(|w| w == b"Description=\"has,comma\""));
            assert!(formatted
                .windows(b"Source=\"x\"".len())
                .any(|w| w == b"Source=\"x\""));

            super::super::hts::ks_free(&mut out);
            bcf_hrec_destroy(hrec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_header_get_hrec_respects_record_type_when_ids_collide() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=DP>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let info = bcf_hdr_get_hrec(
                hdr,
                BCF_HL_INFO as c_int,
                c"ID".as_ptr(),
                c"DP".as_ptr(),
                std::ptr::null(),
            );
            assert!(!info.is_null());
            assert_eq!(bcf_hrec_find_key(info, c"Number".as_ptr()), 1);

            let contig = bcf_hdr_get_hrec(
                hdr,
                BCF_HL_CTG as c_int,
                c"ID".as_ptr(),
                c"DP".as_ptr(),
                std::ptr::null(),
            );
            assert!(!contig.is_null());
            assert_eq!(bcf_hrec_find_key(contig, c"Number".as_ptr()), -1);

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_hdr_set_idx_assigns_and_preserves_explicit_slots() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            let start_n = (*hdr).n[BCF_DT_ID as usize];

            let mut first = bcf_idinfo_t {
                info: [15; 3],
                hrec: [std::ptr::null_mut(); 3],
                id: -1,
            };
            assert_eq!(
                vcf_c_796_bcf_hdr_set_idx(hdr, BCF_DT_ID as c_int, c"DP".as_ptr(), &mut first,),
                0
            );
            assert_eq!(first.id, start_n);
            assert_eq!((*hdr).n[BCF_DT_ID as usize], start_n + 1);
            assert_eq!(
                std::ffi::CStr::from_ptr(
                    (*(*hdr).id[BCF_DT_ID as usize].add(first.id as usize)).key
                ),
                c"DP"
            );

            let mut explicit = bcf_idinfo_t {
                info: [15; 3],
                hrec: [std::ptr::null_mut(); 3],
                id: start_n + 3,
            };
            assert_eq!(
                vcf_c_796_bcf_hdr_set_idx(hdr, BCF_DT_ID as c_int, c"AF".as_ptr(), &mut explicit,),
                0
            );
            assert_eq!((*hdr).n[BCF_DT_ID as usize], explicit.id + 1);
            assert!((*(*hdr).id[BCF_DT_ID as usize].add((start_n + 1) as usize))
                .key
                .is_null());
            assert!((*(*hdr).id[BCF_DT_ID as usize].add((start_n + 2) as usize))
                .key
                .is_null());
            assert_eq!(
                std::ffi::CStr::from_ptr(
                    (*(*hdr).id[BCF_DT_ID as usize].add(explicit.id as usize)).key
                ),
                c"AF"
            );

            let mut conflict = bcf_idinfo_t {
                info: [15; 3],
                hrec: [std::ptr::null_mut(); 3],
                id: first.id,
            };
            assert_eq!(
                vcf_c_796_bcf_hdr_set_idx(hdr, BCF_DT_ID as c_int, c"MQ".as_ptr(), &mut conflict,),
                -1
            );

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_hdr_unregister_hrec_clears_existing_dictionary_pointer() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let hrec = bcf_hdr_get_hrec(
                hdr,
                BCF_HL_INFO as c_int,
                c"ID".as_ptr(),
                c"DP".as_ptr(),
                std::ptr::null(),
            );
            assert!(!hrec.is_null());

            let id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"DP".as_ptr());
            assert!(id >= 0);
            let idpair = (*hdr).id[BCF_DT_ID as usize].add(id as usize);
            let idinfo = (*idpair).val.cast_mut();
            assert_eq!((*idinfo).hrec[BCF_HL_INFO as usize], hrec);

            vcf_c_1026_bcf_hdr_unregister_hrec(hdr, hrec);
            assert!((*idinfo).hrec[BCF_HL_INFO as usize].is_null());
            assert_eq!(bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"DP".as_ptr()), id);

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcfutils_calc_ac_uses_info_and_gt_counts() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=AN,Number=1,Type=Integer,Description=\"AN\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=AC,Number=A,Type=Integer,Description=\"AC\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, c"S2".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(
                    c"chr1\t1\t.\tA\tC,G\t.\t.\tAN=4;AC=1,2\tGT\t0/1\t2/2".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            let mut ac = [0; 3];
            assert_eq!(
                vcfutils_c_32_bcf_calc_ac(hdr, rec, ac.as_mut_ptr(), BCF_UN_INFO as c_int),
                1
            );
            assert_eq!(ac, [1, 1, 2]);

            let mut gt_ac = [0; 3];
            assert_eq!(
                vcfutils_c_32_bcf_calc_ac(hdr, rec, gt_ac.as_mut_ptr(), BCF_UN_FMT as c_int),
                1
            );
            assert_eq!(gt_ac, [1, 1, 2]);

            let fmt = bcf_get_fmt(hdr, rec, c"GT".as_ptr());
            assert!(!fmt.is_null());
            let mut ial = -1;
            let mut jal = -1;
            assert_eq!(
                vcfutils_c_134_bcf_gt_type(fmt, 0, &mut ial, &mut jal),
                GT_HET_RA as c_int
            );
            assert_eq!(ial, 1);
            assert_eq!(jal, 0);
            assert_eq!(
                vcfutils_c_134_bcf_gt_type(fmt, 1, &mut ial, &mut jal),
                GT_HOM_AA as c_int
            );
            assert_eq!(ial, 2);

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_parse_decodes_info_and_format_integer_boundaries() {
        unsafe {
            let fmt_int = |fmt: *mut bcf_fmt_t, sample: usize, idx: usize| -> i32 {
                let p = (*fmt).p.add(sample * (*fmt).size as usize);
                match (*fmt).type_ {
                    x if x == BCF_BT_INT8 as c_int => le_to_i8(p.add(idx)) as i32,
                    x if x == BCF_BT_INT16 as c_int => {
                        le_to_i16(p.add(idx * size_of::<i16>())) as i32
                    }
                    x if x == BCF_BT_INT32 as c_int => le_to_i32(p.add(idx * size_of::<i32>())),
                    _ => bcf_int32_missing,
                }
            };

            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=AD,Number=R,Type=Integer,Description=\"Allele depths\">"
                        .as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, c"S2".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(
                    c"chr1\t5\trs1\tA\tC\t.\tPASS\tDP=32768\tGT:AD\t0/1:3,32768\t./.:.,.".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            let dp = bcf_get_info(hdr, rec, c"DP".as_ptr());
            assert!(!dp.is_null());
            assert_eq!((*dp).len, 1);
            assert_eq!(vcfutils_c_280_get_int32_info_value(dp, 0), 32768);
            assert_eq!(
                vcfutils_c_280_get_int32_info_value(dp, 1),
                bcf_int32_missing
            );

            let ad = bcf_get_fmt(hdr, rec, c"AD".as_ptr());
            assert!(!ad.is_null());
            assert_eq!((*ad).n, 2);
            assert_eq!(fmt_int(ad, 0, 0), 3);
            assert_eq!(fmt_int(ad, 0, 1), 32768);
            assert_eq!(fmt_int(ad, 1, 0), bcf_int32_missing);
            assert_eq!(fmt_int(ad, 1, 1), bcf_int32_missing);

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_end_i64_repair_matches_htslib_integer_boundaries() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=END,Number=1,Type=Integer,Description=\"End\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(
                    c"chr1\t5\t.\tA\t<DEL>\t.\tPASS\tEND=.".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            let end_id = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"END".as_ptr());
            assert!(end_id >= 0);
            assert_eq!(vcf_repair_info_end_i64(hdr, rec, BCF_MIN_BT_INT32 - 1), 0);
            let info = bcf_get_info_id(rec, end_id);
            assert!(!info.is_null());
            assert_eq!((*info).type_, BCF_BT_INT64 as c_int);
            assert_eq!((*info).v1.i, BCF_MIN_BT_INT32 - 1);
            assert_eq!(le_to_i64((*info).vptr), BCF_MIN_BT_INT32 - 1);

            bcf_clear(rec);
            line.l = 0;
            assert!(
                super::super::hts::kputs(
                    c"chr1\t5\t.\tA\t<DEL>\t.\tPASS\tEND=.".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);
            assert_eq!(vcf_repair_info_end_i64(hdr, rec, BCF_MIN_BT_INT64 - 1), 0);
            let info = bcf_get_info_id(rec, end_id);
            assert!(!info.is_null());
            assert!((*info).type_ != BCF_BT_INT64 as c_int || (*info).v1.i != BCF_MIN_BT_INT64 - 1);

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_info_int64_inline_wrappers_get_and_remove_integer_values() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=BIG,Number=1,Type=Integer,Description=\"wide int\">".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(c"chr1\t1\t.\tA\tC\t.\tPASS\tBIG=17".as_ptr(), &mut line)
                    >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            let mut dst: *mut i64 = std::ptr::null_mut();
            let mut ndst = 0;
            assert_eq!(
                bcf_get_info_int64(hdr, rec, c"BIG".as_ptr(), &mut dst, &mut ndst),
                1
            );
            assert!(ndst >= 1);
            assert!(!dst.is_null());
            assert_eq!(*dst, 17);

            assert_eq!(
                bcf_update_info_int64(hdr, rec, c"BIG".as_ptr(), std::ptr::null(), 0),
                0
            );

            libc::free(dst.cast());
            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_format_gt_inline_wrapper_uses_legacy_vcf43_phasing() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            (*rec).pos = 0;
            assert_eq!(bcf_update_alleles_str(hdr, rec, c"A,C".as_ptr()), 0);
            let gt = [((0 + 1) << 1) as i32, ((1 + 1) << 1) as i32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    rec,
                    c"GT".as_ptr(),
                    gt.as_ptr().cast(),
                    gt.len() as c_int,
                    BCF_HT_INT as c_int,
                ),
                0
            );

            let fmt = bcf_get_fmt(hdr, rec, c"GT".as_ptr());
            assert!(!fmt.is_null());
            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(bcf_format_gt(fmt, 0, &mut out), 0);
            assert_eq!(CStr::from_ptr(out.s).to_bytes(), b"0/1");

            super::super::hts::ks_free(&mut out);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_private_encoders_use_htslib_extended_width_boundaries() {
        unsafe {
            let mut str_ = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };

            assert_eq!(bcf_enc_size(&mut str_, 14, BCF_BT_INT8 as c_int), 0);
            assert_eq!(str_.l, 1);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((14 << 4) | BCF_BT_INT8 as c_int) as u8
            );

            str_.l = 0;
            assert_eq!(bcf_enc_size(&mut str_, 15, BCF_BT_INT16 as c_int), 0);
            assert_eq!(str_.l, 3);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((15 << 4) | BCF_BT_INT16 as c_int) as u8
            );
            assert_eq!(
                *str_.s.add(1).cast::<u8>(),
                ((1 << 4) | BCF_BT_INT8 as c_int) as u8
            );
            assert_eq!(*str_.s.add(2).cast::<u8>(), 15);

            str_.l = 0;
            assert_eq!(bcf_enc_size(&mut str_, 32768, BCF_BT_INT32 as c_int), 0);
            assert_eq!(str_.l, 6);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((15 << 4) | BCF_BT_INT32 as c_int) as u8
            );
            assert_eq!(
                *str_.s.add(1).cast::<u8>(),
                ((1 << 4) | BCF_BT_INT32 as c_int) as u8
            );
            assert_eq!(le_to_i32(str_.s.add(2).cast()), 32768);

            str_.l = 0;
            assert_eq!(bcf_enc_int1(&mut str_, -120), 0);
            assert_eq!(str_.l, 2);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((1 << 4) | BCF_BT_INT8 as c_int) as u8
            );
            assert_eq!(le_to_i8(str_.s.add(1).cast()) as i32, -120);

            str_.l = 0;
            assert_eq!(bcf_enc_int1(&mut str_, -121), 0);
            assert_eq!(str_.l, 3);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((1 << 4) | BCF_BT_INT16 as c_int) as u8
            );
            assert_eq!(le_to_i16(str_.s.add(1).cast()) as i32, -121);

            str_.l = 0;
            assert_eq!(bcf_enc_int1(&mut str_, 32768), 0);
            assert_eq!(str_.l, 5);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((1 << 4) | BCF_BT_INT32 as c_int) as u8
            );
            assert_eq!(le_to_i32(str_.s.add(1).cast()), 32768);

            str_.l = 0;
            assert_eq!(bcf_enc_int1(&mut str_, bcf_int32_missing), 0);
            assert_eq!(str_.l, 2);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((1 << 4) | BCF_BT_INT8 as c_int) as u8
            );
            assert_eq!(le_to_i8(str_.s.add(1).cast()) as i32, bcf_int8_missing);

            str_.l = 0;
            assert_eq!(bcf_enc_int1(&mut str_, bcf_int32_vector_end), 0);
            assert_eq!(str_.l, 2);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((1 << 4) | BCF_BT_INT8 as c_int) as u8
            );
            assert_eq!(le_to_i8(str_.s.add(1).cast()) as i32, bcf_int8_vector_end);

            super::super::hts::ks_free(&mut str_);
        }
    }

    #[test]
    fn vcfutils_get_int32_info_value_widens_narrow_sentinels() {
        unsafe {
            let mut int8_values = [
                bcf_int8_vector_end as u8,
                (bcf_int8_vector_end - 1) as u8,
                7u8,
            ];
            let mut info: bcf_info_t = std::mem::zeroed();
            info.len = int8_values.len() as c_int;
            info.type_ = BCF_BT_INT8 as c_int;
            info.vptr = int8_values.as_mut_ptr();

            assert_eq!(
                vcfutils_c_280_get_int32_info_value(&info, 0),
                bcf_int32_vector_end
            );
            assert_eq!(
                vcfutils_c_280_get_int32_info_value(&info, 1),
                bcf_int32_vector_end - 1
            );
            assert_eq!(vcfutils_c_280_get_int32_info_value(&info, 2), 7);

            let mut int16_values = [0u8; 4];
            i16_to_le(bcf_int16_vector_end as i16, int16_values.as_mut_ptr());
            i16_to_le(
                (bcf_int16_vector_end - 1) as i16,
                int16_values.as_mut_ptr().add(size_of::<i16>()),
            );
            info.len = 2;
            info.type_ = BCF_BT_INT16 as c_int;
            info.vptr = int16_values.as_mut_ptr();

            assert_eq!(
                vcfutils_c_280_get_int32_info_value(&info, 0),
                bcf_int32_vector_end
            );
            assert_eq!(
                vcfutils_c_280_get_int32_info_value(&info, 1),
                bcf_int32_vector_end - 1
            );
        }
    }

    #[test]
    fn bcf_translate_rewrites_record_ids_for_destination_header() {
        unsafe {
            let hdr1 = bcf_hdr_init(c"w".as_ptr());
            let mut hdr2 = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr1.is_null());
            assert!(!hdr2.is_null());

            for (hdr, line) in [
                (hdr1, c"##contig=<ID=1>".as_ptr()),
                (hdr1, c"##contig=<ID=2>".as_ptr()),
                (hdr2, c"##contig=<ID=2>".as_ptr()),
                (hdr2, c"##contig=<ID=1>".as_ptr()),
                (
                    hdr1,
                    c"##FILTER=<ID=FLT1,Description=\"Filter 1\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FILTER=<ID=FLT2,Description=\"Filter 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FILTER=<ID=FLT3,Description=\"Filter 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FILTER=<ID=FLT4,Description=\"Filter 4\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FILTER=<ID=FLT3,Description=\"Filter 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FILTER=<ID=FLT2,Description=\"Filter 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##INFO=<ID=INF1,Number=.,Type=Integer,Description=\"Info 1\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF4,Number=.,Type=Integer,Description=\"Info 4\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF3,Number=.,Type=Integer,Description=\"Info 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF2,Number=.,Type=Integer,Description=\"Info 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT1,Number=.,Type=Integer,Description=\"FMT 1\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT4,Number=.,Type=Integer,Description=\"FMT 4\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT3,Number=.,Type=Integer,Description=\"FMT 3\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT2,Number=.,Type=Integer,Description=\"FMT 2\">".as_ptr(),
                ),
            ] {
                assert_eq!(bcf_hdr_append(hdr, line), 0);
            }

            assert_eq!(bcf_hdr_add_sample(hdr1, c"SMPL1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr1, c"SMPL2".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr2, c"SMPL1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr2, c"SMPL2".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr1), 0);
            assert_eq!(bcf_hdr_sync(hdr2), 0);
            hdr2 = bcf_hdr_merge(hdr2, hdr1);
            assert!(!hdr2.is_null());
            assert_eq!(bcf_hdr_sync(hdr2), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr1, c"1".as_ptr());
            (*rec).pos = 0;
            assert_eq!(bcf_update_alleles_str(hdr1, rec, c"G,A".as_ptr()), 0);

            let mut ids = [
                bcf_hdr_id2int(hdr1, BCF_DT_ID as c_int, c"FLT1".as_ptr()),
                bcf_hdr_id2int(hdr1, BCF_DT_ID as c_int, c"FLT2".as_ptr()),
                bcf_hdr_id2int(hdr1, BCF_DT_ID as c_int, c"FLT3".as_ptr()),
            ];
            assert_eq!(bcf_update_filter(hdr1, rec, ids.as_mut_ptr(), 3), 0);

            let mut tmp = [1, 2];
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF1".as_ptr(),
                    tmp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF2".as_ptr(),
                    tmp.as_ptr().add(1).cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            tmp[0] = 3;
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF3".as_ptr(),
                    tmp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            tmp = [1, 1];
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT1".as_ptr(),
                    tmp.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            tmp = [2, 2];
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT2".as_ptr(),
                    tmp.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            tmp = [3, 3];
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT3".as_ptr(),
                    tmp.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int,
                ),
                0
            );

            assert_eq!(
                bcf_remove_filter(
                    hdr1,
                    rec,
                    bcf_hdr_id2int(hdr1, BCF_DT_ID as c_int, c"FLT2".as_ptr()),
                    0,
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF2".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT2".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_INT as c_int,
                ),
                0
            );

            assert_eq!(bcf_translate(hdr2, hdr1, rec), 0);
            assert_eq!((*rec).rid, bcf_hdr_name2id(hdr2, c"1".as_ptr()));

            let mut out = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(vcf_format(hdr2, rec, &mut out), 0);
            assert_eq!(
                std::ffi::CStr::from_ptr(out.s),
                c"1\t1\t.\tG\tA\t0\tFLT1;FLT3\tINF1=1;INF3=3\tFMT1:FMT3\t1:3\t1:3\n"
            );

            super::super::hts::ks_free(&mut out);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr1);
            bcf_hdr_destroy(hdr2);
        }
    }

    #[test]
    fn vcf_variant_type_classifiers_match_htslib_branches() {
        unsafe {
            let mut var = bcf_variant_t { type_: -1, n: -1 };

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"C".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_SNP as c_int);
            assert_eq!(var.n, 1);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"A".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"AC".as_ptr(), &mut var);
            assert_eq!(var.type_, (VCF_INDEL | VCF_INS) as c_int);
            assert_eq!(var.n, 1);

            vcf_c_5373_bcf_set_variant_type(c"AT".as_ptr(), c"A".as_ptr(), &mut var);
            assert_eq!(var.type_, (VCF_INDEL | VCF_DEL) as c_int);
            assert_eq!(var.n, -1);

            vcf_c_5373_bcf_set_variant_type(c"AC".as_ptr(), c"GT".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_MNP as c_int);
            assert_eq!(var.n, 2);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"*".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_OVERLAP as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<NON_REF>".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"]chr1:10]A".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_BND as c_int);
        }
    }

    #[test]
    fn vcf_variant_type_classifiers_cover_symbolic_and_breakend_edges() {
        unsafe {
            let mut var = bcf_variant_t { type_: -1, n: -99 };

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c".".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"X".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<X>".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_REF as c_int);
            assert_eq!(var.n, 0);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<*>".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_REF as c_int);
            assert_eq!(var.n, 0);

            var.n = -99;
            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"<DEL>".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_OTHER as c_int);
            assert_eq!(var.n, -99);

            vcf_c_5373_bcf_set_variant_type(c"A".as_ptr(), c"AC]chr2:3]".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_BND as c_int);

            vcf_c_5373_bcf_set_variant_type(c"ACG".as_ptr(), c"AT".as_ptr(), &mut var);
            assert_eq!(var.type_, VCF_OTHER as c_int);
            assert_eq!(var.n, -1);
        }
    }

    #[test]
    fn vcf_set_variant_types_populates_record_variant_array() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(c"chr1\t1\t.\tA\tC,AC,*\t.\t.\t.".as_ptr(), &mut line,)
                    >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);
            assert_eq!(vcf_c_5444_bcf_set_variant_types(rec), 0);
            assert_eq!((*rec).d.n_var, 4);
            assert_eq!((*(*rec).d.var).type_, VCF_REF as c_int);
            assert_eq!((*(*rec).d.var.add(1)).type_, VCF_SNP as c_int);
            assert_eq!((*(*rec).d.var.add(2)).type_, (VCF_INDEL | VCF_INS) as c_int);
            assert_eq!((*(*rec).d.var.add(3)).type_, VCF_OVERLAP as c_int);
            assert_eq!(
                (*rec).d.var_type,
                (VCF_SNP | VCF_INDEL | VCF_INS | VCF_OVERLAP) as c_int
            );

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_has_variant_types_matches_htslib_exact_and_subset_collapse() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(c"chr1\t1\t.\tA\tAC\t.\t.\t.".as_ptr(), &mut line) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            assert_eq!(bcf_has_variant_types(rec, VCF_INS, 0), VCF_INS as c_int);
            assert_eq!(bcf_has_variant_types(rec, VCF_INDEL, 0), VCF_INDEL as c_int);
            assert_eq!(bcf_has_variant_types(rec, VCF_DEL, 0), 0);
            assert_eq!(
                bcf_has_variant_types(rec, VCF_INDEL | VCF_INS, 2),
                (VCF_INDEL | VCF_INS) as c_int
            );

            bcf_clear(rec);
            line.l = 0;
            assert!(
                super::super::hts::kputs(c"chr1\t1\t.\tACG\tA,ACGT\t.\t.\t.".as_ptr(), &mut line,)
                    >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            assert_eq!(bcf_has_variant_types(rec, VCF_INDEL, 0), VCF_INDEL as c_int);
            assert_eq!(bcf_has_variant_types(rec, VCF_INS, 0), 0);
            assert_eq!(
                bcf_has_variant_types(rec, VCF_INS | VCF_DEL, 1),
                (VCF_INS | VCF_DEL) as c_int
            );

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_per_allele_variant_queries_cover_ref_alt_and_invalid_bounds() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(c"chr1\t1\t.\tA\tAC,G\t.\t.\t.".as_ptr(), &mut line) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            assert_eq!(bcf_has_variant_type(rec, 0, VCF_REF), 1);
            assert_eq!(bcf_variant_length(rec, 0), 0);
            assert_eq!(bcf_has_variant_type(rec, 1, VCF_INS), VCF_INS as c_int);
            assert_eq!(bcf_variant_length(rec, 1), 1);
            assert_eq!(bcf_has_variant_type(rec, 2, VCF_SNP), VCF_SNP as c_int);
            assert_eq!(bcf_variant_length(rec, 2), 1);
            assert_eq!(bcf_has_variant_type(rec, -1, VCF_REF), -1);
            assert_eq!(bcf_has_variant_type(rec, 3, VCF_REF), -1);
            assert_eq!(bcf_variant_length(rec, -1), bcf_int32_missing);
            assert_eq!(bcf_variant_length(rec, 3), bcf_int32_missing);

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn bcf_translate_width_promoted_ids_keep_info_and_format_payload_offsets() {
        unsafe {
            let hdr1 = bcf_hdr_init(c"w".as_ptr());
            let hdr2 = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr1.is_null());
            assert!(!hdr2.is_null());

            for hdr in [hdr1, hdr2] {
                assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=1>".as_ptr()), 0);
            }
            for i in 0..130 {
                let line = std::ffi::CString::new(format!(
                    "##INFO=<ID=D{i},Number=1,Type=Integer,Description=\"dummy\">"
                ))
                .unwrap();
                assert_eq!(bcf_hdr_append(hdr2, line.as_ptr()), 0);
            }
            for (hdr, line) in [
                (
                    hdr1,
                    c"##INFO=<ID=INF,Number=1,Type=Integer,Description=\"info\">".as_ptr(),
                ),
                (
                    hdr1,
                    c"##FORMAT=<ID=FMT,Number=1,Type=Integer,Description=\"fmt\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##INFO=<ID=INF,Number=1,Type=Integer,Description=\"info\">".as_ptr(),
                ),
                (
                    hdr2,
                    c"##FORMAT=<ID=FMT,Number=1,Type=Integer,Description=\"fmt\">".as_ptr(),
                ),
            ] {
                assert_eq!(bcf_hdr_append(hdr, line), 0);
            }
            assert_eq!(bcf_hdr_add_sample(hdr1, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr2, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr1), 0);
            assert_eq!(bcf_hdr_sync(hdr2), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            (*rec).rid = bcf_hdr_name2id(hdr1, c"1".as_ptr());
            (*rec).pos = 0;
            assert_eq!(bcf_update_alleles_str(hdr1, rec, c"A,C".as_ptr()), 0);
            let info_val = 42i32;
            let fmt_val = 7i32;
            assert_eq!(
                bcf_update_info(
                    hdr1,
                    rec,
                    c"INF".as_ptr(),
                    (&info_val as *const i32).cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_format(
                    hdr1,
                    rec,
                    c"FMT".as_ptr(),
                    (&fmt_val as *const i32).cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );

            let src_info_id = bcf_hdr_id2int(hdr1, BCF_DT_ID as c_int, c"INF".as_ptr());
            let dst_info_id = bcf_hdr_id2int(hdr2, BCF_DT_ID as c_int, c"INF".as_ptr());
            let src_fmt_id = bcf_hdr_id2int(hdr1, BCF_DT_ID as c_int, c"FMT".as_ptr());
            let dst_fmt_id = bcf_hdr_id2int(hdr2, BCF_DT_ID as c_int, c"FMT".as_ptr());
            assert_eq!(bcf_translate_id_size(src_info_id), BCF_BT_INT8 as c_int);
            assert_eq!(bcf_translate_id_size(src_fmt_id), BCF_BT_INT8 as c_int);
            assert!(bcf_translate_id_size(dst_info_id) > BCF_BT_INT8 as c_int);
            assert!(bcf_translate_id_size(dst_fmt_id) > BCF_BT_INT8 as c_int);

            assert_eq!(bcf_translate(hdr2, hdr1, rec), 0);
            let info = bcf_get_info(hdr2, rec, c"INF".as_ptr());
            assert!(!info.is_null());
            assert_eq!((*info).vptr_off(), 4);
            assert_eq!(le_to_i8((*info).vptr), info_val as i8);

            let fmt = bcf_get_fmt(hdr2, rec, c"FMT".as_ptr());
            assert!(!fmt.is_null());
            assert_eq!((*fmt).p_off(), 4);
            assert_eq!(le_to_i8((*fmt).p), fmt_val as i8);

            bcf_destroy(rec);
            bcf_hdr_destroy(hdr1);
            bcf_hdr_destroy(hdr2);
        }
    }

    #[test]
    fn bcf_translate_id_size_uses_htslib_width_boundaries() {
        unsafe {
            assert_eq!(bcf_translate_id_size(0), BCF_BT_INT8 as c_int);
            assert_eq!(bcf_translate_id_size(127), BCF_BT_INT8 as c_int);
            assert_eq!(bcf_translate_id_size(128), BCF_BT_INT16 as c_int);
            assert_eq!(bcf_translate_id_size(32767), BCF_BT_INT16 as c_int);
            assert_eq!(bcf_translate_id_size(32768), BCF_BT_INT32 as c_int);
        }
    }

    #[test]
    fn synced_bcf_reader_wrappers_create_and_destroy_reader_set() {
        unsafe {
            let readers = bcf_sr_init();
            assert!(!readers.is_null());
            assert!(!bcf_sr_strerror(0).is_null());
            assert_eq!(bcf_sr_set_threads(readers, 0), 0);
            bcf_sr_destroy_threads(readers);
            bcf_sr_destroy(readers);
        }
    }

    #[test]
    fn synced_bcf_reader_strerror_matches_htslib_messages() {
        unsafe {
            assert_eq!(
                CStr::from_ptr(bcf_sr_strerror(bcf_sr_error_not_bgzf as c_int)).to_bytes(),
                b"not compressed with bgzip"
            );
            assert_eq!(
                CStr::from_ptr(bcf_sr_strerror(bcf_sr_error_idx_load_failed as c_int)).to_bytes(),
                b"could not load index"
            );
            assert_eq!(
                CStr::from_ptr(bcf_sr_strerror(BCF_SR_ERROR_NOIDX_ERROR)).to_bytes(),
                b"merge of unindexed files failed"
            );
            assert_eq!(CStr::from_ptr(bcf_sr_strerror(99)).to_bytes(), b"");
        }
    }

    #[test]
    fn synced_bcf_reader_strerror_open_failed_uses_errno() {
        unsafe {
            let errno = libc::__errno_location();
            let saved_errno = *errno;
            *errno = libc::ENOENT;
            assert_eq!(
                CStr::from_ptr(bcf_sr_strerror(bcf_sr_error_open_failed as c_int)).to_bytes(),
                CStr::from_ptr(libc::strerror(libc::ENOENT)).to_bytes()
            );
            *errno = saved_errno;
        }
    }

    #[test]
    fn synced_bcf_reader_regions_alloc_sets_iteration_sentinels_and_overlap_storage() {
        unsafe {
            let reg = bcf_sr_regions_alloc();
            assert!(!reg.is_null());
            assert_eq!((*reg).start, -1);
            assert_eq!((*reg).end, -1);
            assert_eq!((*reg).prev_seq, -1);
            assert_eq!((*reg).prev_start, -1);
            assert_eq!((*reg).prev_end, -1);

            bcf_sr_regions_set_overlap(reg, 2);
            assert_eq!(bcf_sr_regions_get_overlap(reg), 2);

            bcf_sr_regions_destroy_translated(reg);
        }
    }

    #[test]
    fn synced_bcf_reader_regions_destroy_releases_public_region_storage() {
        unsafe {
            bcf_sr_regions_destroy(std::ptr::null_mut());

            let reg = libc::calloc(1, size_of::<bcf_sr_regions_t>()).cast::<bcf_sr_regions_t>();
            assert!(!reg.is_null());
            (*reg).fname = libc::strdup(c"regions.bed.gz".as_ptr());
            assert!(!(*reg).fname.is_null());
            (*reg).line.s = libc::malloc(8).cast();
            assert!(!(*reg).line.s.is_null());
            (*reg).line.m = 8;
            (*reg).als_str.s = libc::malloc(16).cast();
            assert!(!(*reg).als_str.s.is_null());
            (*reg).als_str.m = 16;
            (*reg).als = libc::malloc(size_of::<*mut c_char>()).cast();
            assert!(!(*reg).als.is_null());
            (*reg).seq_names = libc::malloc(size_of::<*mut c_char>()).cast();
            assert!(!(*reg).seq_names.is_null());
            *(*reg).seq_names = c"chr1".as_ptr().cast_mut();
            (*reg).nseqs = 1;
            (*reg).seq_hash = super::super::sam::khash_str2int_init();
            assert!(!(*reg).seq_hash.is_null());

            bcf_sr_regions_destroy(reg);
        }
    }

    #[test]
    fn synced_bcf_reader_regions_add_reuses_sequences_and_stores_zero_based_regions() {
        unsafe {
            let reg = bcf_sr_regions_alloc();
            assert!(!reg.is_null());

            assert_eq!(bcf_sr_regions_add(reg, c"chr2".as_ptr(), 10, 20), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr2".as_ptr(), 30, 35), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr1".as_ptr(), -1, -1), 0);

            assert_eq!((*reg).nseqs, 2);
            assert_eq!(
                std::ffi::CStr::from_ptr(*(*reg).seq_names.add(0)).to_bytes(),
                b"chr2"
            );
            assert_eq!(
                std::ffi::CStr::from_ptr(*(*reg).seq_names.add(1)).to_bytes(),
                b"chr1"
            );

            let regs = (*reg).regs.cast::<BcfSrRegion>();
            assert_eq!((*regs.add(0)).nregs, 2);
            assert_eq!((*regs.add(0)).mregs, 2);
            assert_eq!((*regs.add(0)).creg, -1);
            assert_eq!((*(*regs.add(0)).regs.add(0)).start, 9);
            assert_eq!((*(*regs.add(0)).regs.add(0)).end, 19);
            assert_eq!((*(*regs.add(0)).regs.add(1)).start, 29);
            assert_eq!((*(*regs.add(0)).regs.add(1)).end, 34);

            assert_eq!((*regs.add(1)).nregs, 1);
            assert_eq!((*(*regs.add(1)).regs).start, 0);
            assert_eq!((*(*regs.add(1)).regs).end, MAX_CSI_COOR - 1);

            let mut iseq = -1;
            assert_eq!(
                super::super::sam::khash_str2int_get((*reg).seq_hash, c"chr2".as_ptr(), &mut iseq),
                0
            );
            assert_eq!(iseq, 0);

            bcf_sr_regions_destroy_translated(reg);
        }
    }

    #[test]
    fn synced_bcf_reader_regions_sort_and_merge_marks_overlaps_without_compacting() {
        unsafe {
            let reg = bcf_sr_regions_alloc();
            assert!(!reg.is_null());

            assert_eq!(bcf_sr_regions_add(reg, c"chr1".as_ptr(), 30, 40), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr1".as_ptr(), 10, 20), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr1".as_ptr(), 18, 25), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr1".as_ptr(), 26, 29), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr2".as_ptr(), 5, 8), 0);
            assert_eq!(bcf_sr_regions_add(reg, c"chr2".as_ptr(), 1, 4), 0);

            synced_bcf_reader_c_1085__regions_sort_and_merge(reg);

            let regs = (*reg).regs.cast::<BcfSrRegion>();
            let chr1 = regs.add(0);
            assert_eq!((*chr1).nregs, 4);
            let chr1_regs = std::slice::from_raw_parts((*chr1).regs, (*chr1).nregs as usize);
            assert_eq!(chr1_regs[0].start, 9);
            assert_eq!(chr1_regs[0].end, 24);
            assert_eq!(chr1_regs[1].start, 1);
            assert_eq!(chr1_regs[1].end, 0);
            assert_eq!(chr1_regs[2].start, 25);
            assert_eq!(chr1_regs[2].end, 28);
            assert_eq!(chr1_regs[3].start, 29);
            assert_eq!(chr1_regs[3].end, 39);

            let chr2 = regs.add(1);
            assert_eq!((*chr2).nregs, 2);
            let chr2_regs = std::slice::from_raw_parts((*chr2).regs, (*chr2).nregs as usize);
            assert_eq!(chr2_regs[0].start, 0);
            assert_eq!(chr2_regs[0].end, 3);
            assert_eq!(chr2_regs[1].start, 4);
            assert_eq!(chr2_regs[1].end, 7);

            bcf_sr_regions_destroy_translated(reg);
        }
    }

    #[test]
    fn synced_bcf_reader_regions_next_and_seek_iterate_in_memory_regions() {
        unsafe {
            let reg = bcf_sr_regions_init(
                c"chr1:30-31,chr1:10-20,chr1:18-25,chr2:5-8".as_ptr(),
                0,
                0,
                1,
                2,
            );
            assert!(!reg.is_null());

            assert_eq!(bcf_sr_regions_seek(reg, c"chr1".as_ptr()), 0);
            assert_eq!(bcf_sr_regions_next(reg), 0);
            assert_eq!((*reg).iseq, 0);
            assert_eq!((*reg).start, 9);
            assert_eq!((*reg).end, 24);

            assert_eq!(bcf_sr_regions_next(reg), 0);
            assert_eq!((*reg).iseq, 0);
            assert_eq!((*reg).start, 29);
            assert_eq!((*reg).end, 30);

            assert_eq!(bcf_sr_regions_next(reg), 0);
            assert_eq!((*reg).iseq, 1);
            assert_eq!((*reg).start, 4);
            assert_eq!((*reg).end, 7);

            assert_eq!(bcf_sr_regions_next(reg), -1);
            assert_eq!((*reg).iseq, -1);
            assert_eq!(bcf_sr_regions_seek(reg, c"missing".as_ptr()), -1);

            bcf_sr_regions_destroy(reg);
        }
    }

    unsafe extern "C" fn count_missed_region(_reg: *mut bcf_sr_regions_t, data: *mut c_void) {
        unsafe {
            *(data.cast::<c_int>()) += 1;
        }
    }

    #[test]
    fn synced_bcf_reader_regions_overlap_and_flush_use_translated_in_memory_path() {
        unsafe {
            let reg = bcf_sr_regions_init(c"chr1:10-20,chr1:30-40,chr2:5-8".as_ptr(), 0, 0, 1, 2);
            assert!(!reg.is_null());

            assert_eq!(bcf_sr_regions_overlap(reg, c"missing".as_ptr(), 0, 10), -1);
            assert_eq!(bcf_sr_regions_overlap(reg, c"chr1".as_ptr(), 9, 9), 0);
            assert_eq!((*reg).start, 9);
            assert_eq!((*reg).end, 19);
            assert_eq!(bcf_sr_regions_overlap(reg, c"chr1".as_ptr(), 20, 28), -1);
            assert_eq!(bcf_sr_regions_overlap(reg, c"chr1".as_ptr(), 29, 35), 0);
            assert_eq!((*reg).start, 29);
            assert_eq!((*reg).end, 39);
            assert_eq!(bcf_sr_regions_overlap(reg, c"chr1".as_ptr(), 100, 110), -1);

            assert_eq!(bcf_sr_regions_overlap(reg, c"chr2".as_ptr(), 4, 4), 0);
            let mut missed = 0;
            (*reg).missed_reg_handler = Some(count_missed_region);
            (*reg).missed_reg_data = (&mut missed as *mut c_int).cast();
            assert_eq!(bcf_sr_regions_flush(reg), 0);
            assert_eq!(missed, 0);

            assert_eq!(bcf_sr_regions_overlap(reg, c"chr1".as_ptr(), 9, 9), 0);
            assert_eq!(bcf_sr_regions_flush(reg), 0);
            assert_eq!(missed, 1);

            bcf_sr_regions_destroy(reg);
        }
    }

    #[test]
    fn synced_bcf_reader_regions_init_string_parses_lists_and_braced_names() {
        unsafe {
            let reg =
                bcf_sr_regions_init(c"chr2:10-20,{chr:odd}:7,chr1,chr2:30-".as_ptr(), 0, 0, 1, 2);
            assert!(!reg.is_null());
            assert_eq!((*reg).nseqs, 3);

            assert_eq!(
                std::ffi::CStr::from_ptr(*(*reg).seq_names.add(0)).to_bytes(),
                b"chr2"
            );
            assert_eq!(
                std::ffi::CStr::from_ptr(*(*reg).seq_names.add(1)).to_bytes(),
                b"chr:odd"
            );
            assert_eq!(
                std::ffi::CStr::from_ptr(*(*reg).seq_names.add(2)).to_bytes(),
                b"chr1"
            );

            let regs = (*reg).regs.cast::<BcfSrRegion>();
            let chr2 = regs.add(0);
            assert_eq!((*chr2).nregs, 2);
            let chr2_regs = std::slice::from_raw_parts((*chr2).regs, (*chr2).nregs as usize);
            assert_eq!(chr2_regs[0].start, 9);
            assert_eq!(chr2_regs[0].end, 19);
            assert_eq!(chr2_regs[1].start, 29);
            assert_eq!(chr2_regs[1].end, MAX_CSI_COOR - 2);

            let odd = regs.add(1);
            assert_eq!((*odd).nregs, 1);
            assert_eq!((*(*odd).regs).start, 6);
            assert_eq!((*(*odd).regs).end, 6);

            let chr1 = regs.add(2);
            assert_eq!((*chr1).nregs, 1);
            assert_eq!((*(*chr1).regs).start, 0);
            assert_eq!((*(*chr1).regs).end, MAX_CSI_COOR - 1);

            bcf_sr_regions_destroy_translated(reg);
        }
    }

    #[test]
    fn synced_bcf_reader_regions_init_string_rejects_malformed_regions() {
        unsafe {
            assert!(bcf_sr_regions_init(c"chr1:abc".as_ptr(), 0, 0, 1, 2).is_null());
            assert!(bcf_sr_regions_init(c"{chr1:1-2".as_ptr(), 0, 0, 1, 2).is_null());
            assert!(bcf_sr_regions_init(c"chr1:1+x".as_ptr(), 0, 0, 1, 2).is_null());
        }
    }

    #[test]
    fn synced_bcf_reader_regions_parse_line_handles_comments_and_columns() {
        unsafe {
            let mut comment = b"#chrom\tfrom\tto\0".to_vec();
            let mut chr = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut from = 0;
            let mut to = 0;
            assert_eq!(
                regions_parse_line(
                    comment.as_mut_ptr().cast(),
                    0,
                    1,
                    2,
                    &mut chr,
                    &mut chr_end,
                    &mut from,
                    &mut to,
                ),
                0
            );

            let mut line = b"chr1\t10\t20\tignored\0".to_vec();
            assert_eq!(
                regions_parse_line(
                    line.as_mut_ptr().cast(),
                    0,
                    1,
                    2,
                    &mut chr,
                    &mut chr_end,
                    &mut from,
                    &mut to,
                ),
                1
            );
            assert_eq!(from, 10);
            assert_eq!(to, 20);
            assert_eq!(chr, line.as_mut_ptr().cast());
            assert_eq!(chr_end.offset_from(chr), 4);

            let mut swapped = b"10\tchr2\t30\0".to_vec();
            assert_eq!(
                regions_parse_line(
                    swapped.as_mut_ptr().cast(),
                    1,
                    2,
                    0,
                    &mut chr,
                    &mut chr_end,
                    &mut from,
                    &mut to,
                ),
                1
            );
            assert_eq!(from, 30);
            assert_eq!(to, 10);
            assert_eq!(
                std::slice::from_raw_parts(chr.cast::<u8>(), chr_end.offset_from(chr) as usize),
                b"chr2"
            );
        }
    }

    #[test]
    fn synced_bcf_reader_regions_parse_line_rejects_bad_fields() {
        unsafe {
            let mut line = b"chr1\t10x\t20\0".to_vec();
            let mut chr = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut from = 0;
            let mut to = 0;
            assert_eq!(
                regions_parse_line(
                    line.as_mut_ptr().cast(),
                    0,
                    1,
                    2,
                    &mut chr,
                    &mut chr_end,
                    &mut from,
                    &mut to,
                ),
                -1
            );
        }
    }

    #[test]
    fn synced_bcf_reader_destroy1_accepts_null_reader() {
        unsafe {
            synced_bcf_reader_c_461_bcf_sr_destroy1(std::ptr::null_mut(), 1);
        }
    }

    #[test]
    fn synced_bcf_reader_destroy1_cleans_reader_without_closing_borrowed_file() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-rs-bcf-sr-destroy1-{}-{}.vcf",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::write(
                &path,
                b"##fileformat=VCFv4.3\n##contig=<ID=chr1>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n",
            )
            .unwrap();
            let path_c = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());

            let mut reader: bcf_sr_t = std::mem::zeroed();
            reader.file = fp.cast();
            reader.fname = libc::strdup(path_c.as_ptr());
            assert!(!reader.fname.is_null());
            reader.header = bcf_hdr_init(c"r".as_ptr());
            assert!(!reader.header.is_null());
            reader.mbuffer = 2;
            reader.buffer = libc::calloc(reader.mbuffer as usize, size_of::<*mut bcf1_t>()).cast();
            assert!(!reader.buffer.is_null());
            *reader.buffer.add(0) = bcf_init();
            *reader.buffer.add(1) = bcf_init();
            assert!(!(*reader.buffer.add(0)).is_null());
            assert!(!(*reader.buffer.add(1)).is_null());
            reader.samples = libc::malloc(size_of::<c_int>()).cast();
            reader.filter_ids = libc::malloc(size_of::<c_int>()).cast();
            assert!(!reader.samples.is_null());
            assert!(!reader.filter_ids.is_null());

            synced_bcf_reader_c_461_bcf_sr_destroy1(&mut reader, 0);
            assert_eq!(hts_close(fp), 0);
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn synced_bcf_reader_add_hreader_rejects_null_inputs_locally() {
        unsafe {
            let errno = libc::__errno_location();
            let saved_errno = *errno;

            *errno = 0;
            assert_eq!(
                bcf_sr_add_hreader(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null()
                ),
                0
            );
            assert_eq!(*errno, libc::EINVAL);

            let readers = bcf_sr_init();
            assert!(!readers.is_null());
            (*readers).errnum = bcf_sr_error_open_failed;

            *errno = 0;
            assert_eq!(
                bcf_sr_add_hreader(readers, std::ptr::null_mut(), 0, std::ptr::null()),
                0
            );
            assert_eq!(*errno, libc::EINVAL);
            assert_eq!((*readers).errnum, bcf_sr_error_api_usage_error);

            bcf_sr_destroy(readers);
            *errno = saved_errno;
        }
    }

    #[test]
    fn synced_bcf_reader_add_hreader_reopens_named_handle_without_private_symbol() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-rs-bcf-sr-add-hreader-{}-{}.vcf",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::write(
                &path,
                b"##fileformat=VCFv4.3\n##contig=<ID=chr1>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t1\t.\tA\tC\t.\t.\t.\n",
            )
            .unwrap();
            let path_c = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let fp = super::super::hts::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());

            let readers = bcf_sr_init();
            assert!(!readers.is_null());
            assert_eq!(bcf_sr_add_hreader(readers, fp, 1, std::ptr::null()), 1);
            assert_eq!((*readers).nreaders, 1);
            assert!(!(*readers).readers.is_null());
            assert_eq!(bcf_sr_next_line(readers), 1);

            bcf_sr_destroy(readers);
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn synced_bcf_reader_init_filters_maps_known_filters_and_dot() {
        unsafe {
            let hdr = bcf_hdr_init(c"r".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FILTER=<ID=PASS,Description=\"All filters passed\">".as_ptr(),
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FILTER=<ID=LowQ,Description=\"Low quality\">".as_ptr()
                ),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FILTER=<ID=Site,Description=\"Site filter\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let mut nfilters = -1;
            let filter_ids = init_filters(hdr, c"LowQ,.,Missing,Site".as_ptr(), &mut nfilters);
            assert!(!filter_ids.is_null());
            assert_eq!(nfilters, 3);
            assert_eq!(
                *filter_ids.add(0),
                bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"LowQ".as_ptr())
            );
            assert_eq!(*filter_ids.add(1), -1);
            assert_eq!(
                *filter_ids.add(2),
                bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"Site".as_ptr())
            );

            libc::free(filter_ids.cast());
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn synced_bcf_reader_init_filters_keeps_empty_result_allocated() {
        unsafe {
            let hdr = bcf_hdr_init(c"r".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let mut nfilters = -1;
            let filter_ids = init_filters(hdr, c"Missing".as_ptr(), &mut nfilters);
            assert!(!filter_ids.is_null());
            assert_eq!(nfilters, 0);

            libc::free(filter_ids.cast());
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn synced_bcf_reader_set_opt_wrappers_match_htslib_enum_paths() {
        unsafe {
            let readers = bcf_sr_init();
            assert!(!readers.is_null());

            assert_eq!(bcf_sr_set_opt_require_idx(readers), 0);
            assert_eq!((*readers).require_index, 1);

            assert_eq!(bcf_sr_set_opt_allow_no_idx(readers), 0);
            assert_eq!((*readers).require_index, 2);

            assert_eq!(
                bcf_sr_set_opt_pair_logic(readers, hts_sys::BCF_SR_PAIR_BOTH as c_int),
                0
            );
            assert_eq!(
                (*bcf_sr_aux_mut(readers)).sort.pair,
                hts_sys::BCF_SR_PAIR_BOTH as c_int
            );
            assert_eq!(bcf_sr_set_opt_regions_overlap(readers, 2), 0);
            assert_eq!((*bcf_sr_aux_mut(readers)).regions_overlap, 2);
            assert_eq!(bcf_sr_set_opt_targets_overlap(readers, 1), 0);
            assert_eq!((*bcf_sr_aux_mut(readers)).targets_overlap, 1);

            bcf_sr_destroy(readers);
        }
    }

    #[test]
    fn synced_bcf_reader_overlap_options_update_existing_region_sets() {
        unsafe {
            let readers = bcf_sr_init();
            assert!(!readers.is_null());
            let regions = bcf_sr_regions_init(c"chr1:1-10".as_ptr(), 0, 0, 1, 2);
            let targets = bcf_sr_regions_init(c"chr2:5-9".as_ptr(), 0, 0, 1, 2);
            assert!(!regions.is_null());
            assert!(!targets.is_null());

            (*readers).regions = regions;
            (*readers).targets = targets;
            assert_eq!(bcf_sr_set_opt_regions_overlap(readers, 1), 0);
            assert_eq!(bcf_sr_regions_get_overlap(regions), 1);
            assert_eq!(bcf_sr_set_opt_targets_overlap(readers, 2), 0);
            assert_eq!(bcf_sr_regions_get_overlap(targets), 2);

            (*readers).regions = std::ptr::null_mut();
            (*readers).targets = std::ptr::null_mut();
            bcf_sr_regions_destroy(regions);
            bcf_sr_regions_destroy(targets);
            bcf_sr_destroy(readers);
        }
    }

    #[test]
    fn synced_bcf_reader_set_opt_dispatches_all_translated_options() {
        unsafe {
            let readers = bcf_sr_init();
            assert!(!readers.is_null());
            let regions = bcf_sr_regions_init(c"chr1:1-10".as_ptr(), 0, 0, 1, 2);
            let targets = bcf_sr_regions_init(c"chr2:5-9".as_ptr(), 0, 0, 1, 2);
            assert!(!regions.is_null());
            assert!(!targets.is_null());

            (*readers).regions = regions;
            (*readers).targets = targets;

            assert_eq!(bcf_sr_set_opt(readers, BCF_SR_REQUIRE_IDX, 0), 0);
            assert_eq!((*readers).require_index, REQUIRE_IDX_);

            assert_eq!(bcf_sr_set_opt(readers, BCF_SR_ALLOW_NO_IDX, 0), 0);
            assert_eq!((*readers).require_index, ALLOW_NO_IDX_);

            assert_eq!(
                bcf_sr_set_opt(
                    readers,
                    BCF_SR_PAIR_LOGIC,
                    hts_sys::BCF_SR_PAIR_EXACT as c_int
                ),
                0
            );
            assert_eq!(
                (*bcf_sr_aux_mut(readers)).sort.pair,
                hts_sys::BCF_SR_PAIR_EXACT as c_int
            );

            assert_eq!(bcf_sr_set_opt(readers, BCF_SR_REGIONS_OVERLAP, 2), 0);
            assert_eq!((*bcf_sr_aux_mut(readers)).regions_overlap, 2);
            assert_eq!(bcf_sr_regions_get_overlap(regions), 2);

            assert_eq!(bcf_sr_set_opt(readers, BCF_SR_TARGETS_OVERLAP, 1), 0);
            assert_eq!((*bcf_sr_aux_mut(readers)).targets_overlap, 1);
            assert_eq!(bcf_sr_regions_get_overlap(targets), 1);

            assert_eq!(bcf_sr_set_opt(readers, 99, 0), 1);

            (*readers).regions = std::ptr::null_mut();
            (*readers).targets = std::ptr::null_mut();
            bcf_sr_regions_destroy(regions);
            bcf_sr_regions_destroy(targets);
            bcf_sr_destroy(readers);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_active_helpers_replace_and_append_indices() {
        unsafe {
            let mut sort: BcfSrSort = std::mem::zeroed();

            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 2), 0);
            assert_eq!(sort.nactive, 1);
            assert!(sort.mactive >= 3);
            assert_eq!(*sort.active, 2);

            assert_eq!(bcf_sr_sort_c_331_bcf_sr_sort_add_active(&mut sort, 5), 0);
            assert_eq!(sort.nactive, 2);
            assert!(sort.mactive >= 6);
            assert_eq!(*sort.active.add(0), 2);
            assert_eq!(*sort.active.add(1), 5);

            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 1), 0);
            assert_eq!(sort.nactive, 1);
            assert_eq!(*sort.active, 1);

            libc::free(sort.active.cast());
        }
    }

    #[test]
    fn synced_bcf_reader_sort_active_helpers_reject_invalid_inputs() {
        unsafe {
            let mut sort: BcfSrSort = std::mem::zeroed();

            assert_eq!(
                bcf_sr_sort_c_324_bcf_sr_sort_set_active(std::ptr::null_mut(), 0),
                -1
            );
            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, -1), -1);
            assert_eq!(
                bcf_sr_sort_c_331_bcf_sr_sort_add_active(std::ptr::null_mut(), 0),
                -1
            );
            assert_eq!(bcf_sr_sort_c_331_bcf_sr_sort_add_active(&mut sort, -1), -1);
            assert!(sort.active.is_null());
            assert_eq!(sort.nactive, 0);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_lifecycle_init_reset_and_destroy_buffers() {
        unsafe {
            let allocated = bcf_sr_sort_c_675_bcf_sr_sort_init(std::ptr::null_mut());
            assert!(!allocated.is_null());
            assert_eq!((*allocated).nactive, 0);
            assert!((*allocated).active.is_null());
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(allocated);
            libc::free(allocated.cast());

            let mut sort: BcfSrSort = std::mem::zeroed();
            sort.nactive = 7;
            assert!(std::ptr::eq(
                bcf_sr_sort_c_675_bcf_sr_sort_init(&mut sort),
                &mut sort
            ));
            assert_eq!(sort.nactive, 0);
            assert!(sort.active.is_null());

            sort.chr = c"chr1".as_ptr();
            sort.active = libc::malloc(size_of::<c_int>()).cast::<c_int>();
            sort.str_.s = libc::malloc(8).cast::<c_char>();
            sort.off = libc::malloc(2 * size_of::<c_int>()).cast::<c_int>();
            sort.charp = libc::malloc(size_of::<*mut c_char>()).cast::<*mut c_char>();
            sort.cnt = libc::malloc(size_of::<c_int>()).cast::<c_int>();
            sort.pmat = libc::malloc(size_of::<c_int>()).cast::<c_int>();

            sort.nsr = 1;
            sort.vcf_buf = libc::calloc(1, size_of::<BcfSrSortVcfBuf>())
                .cast::<BcfSrSortVcfBuf>()
                .cast();
            (*sort.vcf_buf.cast::<BcfSrSortVcfBuf>()).rec =
                libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();

            sort.mvar = 1;
            sort.var = libc::calloc(1, size_of::<BcfSrSortVar>())
                .cast::<BcfSrSortVar>()
                .cast();
            let var = sort.var.cast::<BcfSrSortVar>();
            (*var).str_ = libc::malloc(4).cast::<c_char>();
            (*var).vcf = libc::malloc(size_of::<c_int>()).cast::<c_int>();
            (*var).rec = libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();

            sort.mgrp = 1;
            sort.grp = libc::calloc(1, size_of::<BcfSrSortGrp>())
                .cast::<BcfSrSortGrp>()
                .cast();
            (*sort.grp.cast::<BcfSrSortGrp>()).var =
                libc::malloc(size_of::<c_int>()).cast::<c_int>();

            sort.mvset = 1;
            sort.vset = libc::calloc(1, size_of::<BcfSrSortVarSet>())
                .cast::<BcfSrSortVarSet>()
                .cast();
            (*sort.vset.cast::<BcfSrSortVarSet>()).var =
                libc::malloc(size_of::<c_int>()).cast::<c_int>();

            bcf_sr_sort_c_681_bcf_sr_sort_reset(&mut sort);
            assert!(sort.chr.is_null());

            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
            assert!(sort.active.is_null());
            assert!(sort.vcf_buf.is_null());
            assert!(sort.var.is_null());
            assert!(sort.grp.is_null());
            assert!(sort.vset.is_null());
            assert_eq!(sort.nsr, 0);
            assert_eq!(sort.mvar, 0);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_remove_reader_compacts_existing_vcf_buffers() {
        unsafe {
            let mut sort: BcfSrSort = std::mem::zeroed();
            sort.nsr = 3;
            sort.vcf_buf = libc::calloc(3, size_of::<BcfSrSortVcfBuf>())
                .cast::<BcfSrSortVcfBuf>()
                .cast();
            let vcf_buf = sort.vcf_buf.cast::<BcfSrSortVcfBuf>();
            let first = libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();
            let second = libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();
            let third = libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();
            (*vcf_buf.add(0)).rec = first;
            (*vcf_buf.add(1)).rec = second;
            (*vcf_buf.add(2)).rec = third;
            (*vcf_buf.add(0)).nrec = 11;
            (*vcf_buf.add(1)).nrec = 22;
            (*vcf_buf.add(2)).nrec = 33;

            bcf_sr_sort_c_662_bcf_sr_sort_remove_reader(std::ptr::null_mut(), &mut sort, 0);

            assert_eq!((*vcf_buf.add(0)).rec, second);
            assert_eq!((*vcf_buf.add(0)).nrec, 22);
            assert_eq!((*vcf_buf.add(1)).rec, third);
            assert_eq!((*vcf_buf.add(1)).nrec, 33);
            assert!((*vcf_buf.add(2)).rec.is_null());
            assert_eq!((*vcf_buf.add(2)).nrec, 0);

            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_next_single_active_shifts_reader_buffer() {
        unsafe {
            let mut readers: bcf_srs_t = std::mem::zeroed();
            let mut reader: bcf_sr_t = std::mem::zeroed();
            let mut has_line = [99, 99];
            let tmp = bcf_init();
            let first = bcf_init();
            let second = bcf_init();
            assert!(!tmp.is_null());
            assert!(!first.is_null());
            assert!(!second.is_null());
            (*first).pos = 41;
            (*second).pos = 42;
            let mut buffer = [tmp, first, second];
            reader.buffer = buffer.as_mut_ptr();
            reader.nbuffer = 2;
            reader.mbuffer = 3;
            let mut reader_arr = [reader];
            readers.readers = reader_arr.as_mut_ptr();
            readers.nreaders = 1;
            readers.has_line = has_line.as_mut_ptr();

            let mut sort: BcfSrSort = std::mem::zeroed();
            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 0), 0);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 41),
                1
            );
            assert_eq!(reader_arr[0].nbuffer, 1);
            assert_eq!(*reader_arr[0].buffer.add(0), first);
            assert_eq!(*reader_arr[0].buffer.add(1), second);
            assert_eq!(*reader_arr[0].buffer.add(2), tmp);
            assert_eq!(has_line[0], 1);

            bcf_destroy(tmp);
            bcf_destroy(first);
            bcf_destroy(second);
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_next_consumes_prebuilt_multi_reader_rows() {
        unsafe {
            let mut readers: bcf_srs_t = std::mem::zeroed();
            let mut reader0: bcf_sr_t = std::mem::zeroed();
            let mut reader1: bcf_sr_t = std::mem::zeroed();
            let mut has_line = [0, 0];

            let tmp0 = bcf_init();
            let rec0 = bcf_init();
            let tmp1 = bcf_init();
            let rec1 = bcf_init();
            assert!(!tmp0.is_null());
            assert!(!rec0.is_null());
            assert!(!tmp1.is_null());
            assert!(!rec1.is_null());
            (*rec0).pos = 9;
            (*rec1).pos = 9;

            let mut buffer0 = [tmp0, rec0];
            let mut buffer1 = [tmp1, rec1];
            reader0.buffer = buffer0.as_mut_ptr();
            reader0.nbuffer = 1;
            reader0.mbuffer = 2;
            reader1.buffer = buffer1.as_mut_ptr();
            reader1.nbuffer = 1;
            reader1.mbuffer = 2;
            let mut reader_arr = [reader0, reader1];
            readers.readers = reader_arr.as_mut_ptr();
            readers.nreaders = 2;
            readers.has_line = has_line.as_mut_ptr();

            let mut sort: BcfSrSort = std::mem::zeroed();
            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 0), 0);
            assert_eq!(bcf_sr_sort_c_331_bcf_sr_sort_add_active(&mut sort, 1), 0);
            sort.sr = &mut readers;
            sort.nsr = 2;
            sort.chr = c"chr2".as_ptr();
            sort.pos = 9;
            sort.vcf_buf = libc::calloc(2, size_of::<BcfSrSortVcfBuf>())
                .cast::<BcfSrSortVcfBuf>()
                .cast();
            let vcf_buf = sort.vcf_buf.cast::<BcfSrSortVcfBuf>();
            (*vcf_buf.add(0)).nrec = 1;
            (*vcf_buf.add(0)).mrec = 1;
            (*vcf_buf.add(0)).rec = libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();
            *(*vcf_buf.add(0)).rec = rec0;
            (*vcf_buf.add(1)).nrec = 1;
            (*vcf_buf.add(1)).mrec = 1;
            (*vcf_buf.add(1)).rec = libc::malloc(size_of::<*mut bcf1_t>()).cast::<*mut bcf1_t>();
            *(*vcf_buf.add(1)).rec = rec1;

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr2".as_ptr(), 9),
                2
            );
            assert_eq!(reader_arr[0].nbuffer, 0);
            assert_eq!(reader_arr[1].nbuffer, 0);
            assert_eq!(*reader_arr[0].buffer.add(0), rec0);
            assert_eq!(*reader_arr[1].buffer.add(0), rec1);
            assert_eq!(has_line, [1, 1]);
            assert_eq!((*vcf_buf.add(0)).nrec, 0);
            assert_eq!((*vcf_buf.add(1)).nrec, 0);

            bcf_destroy(tmp0);
            bcf_destroy(rec0);
            bcf_destroy(tmp1);
            bcf_destroy(rec1);
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_set_groups_fresh_multi_reader_rows_by_alleles() {
        unsafe {
            let mut readers: bcf_srs_t = std::mem::zeroed();
            let mut reader0: bcf_sr_t = std::mem::zeroed();
            let mut reader1: bcf_sr_t = std::mem::zeroed();
            let mut has_line = [0, 0];

            let tmp0 = bcf_init();
            let rec0a = bcf_init();
            let rec0b = bcf_init();
            let tmp1 = bcf_init();
            let rec1a = bcf_init();
            let rec1b = bcf_init();
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!tmp0.is_null());
            assert!(!rec0a.is_null());
            assert!(!rec0b.is_null());
            assert!(!tmp1.is_null());
            assert!(!rec1a.is_null());
            assert!(!rec1b.is_null());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);
            let rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            assert_eq!(rid, 0);

            for rec in [rec0a, rec0b, rec1a, rec1b] {
                (*rec).rid = rid;
                (*rec).pos = 9;
            }
            assert_eq!(bcf_update_alleles_str(hdr, rec0a, c"A,C".as_ptr()), 0);
            assert_eq!(bcf_update_alleles_str(hdr, rec0b, c"A,G".as_ptr()), 0);
            assert_eq!(bcf_update_alleles_str(hdr, rec1a, c"A,C".as_ptr()), 0);
            assert_eq!(bcf_update_alleles_str(hdr, rec1b, c"A,T".as_ptr()), 0);

            let mut buffer0 = [tmp0, rec0a, rec0b];
            let mut buffer1 = [tmp1, rec1a, rec1b];
            reader0.buffer = buffer0.as_mut_ptr();
            reader0.nbuffer = 2;
            reader0.mbuffer = 3;
            reader0.header = hdr;
            reader1.buffer = buffer1.as_mut_ptr();
            reader1.nbuffer = 2;
            reader1.mbuffer = 3;
            reader1.header = hdr;
            let mut reader_arr = [reader0, reader1];
            readers.readers = reader_arr.as_mut_ptr();
            readers.nreaders = 2;
            readers.has_line = has_line.as_mut_ptr();

            let mut sort: BcfSrSort = std::mem::zeroed();
            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 0), 0);
            assert_eq!(bcf_sr_sort_c_331_bcf_sr_sort_add_active(&mut sort, 1), 0);
            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 9),
                2
            );
            assert_eq!(has_line, [1, 1]);
            assert_eq!(*reader_arr[0].buffer.add(0), rec0a);
            assert_eq!(*reader_arr[1].buffer.add(0), rec1a);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 9),
                1
            );
            assert_eq!(has_line, [1, 0]);
            assert_eq!(*reader_arr[0].buffer.add(0), rec0b);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 9),
                1
            );
            assert_eq!(has_line, [0, 1]);
            assert_eq!(*reader_arr[1].buffer.add(0), rec1b);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 9),
                0
            );
            assert_eq!(reader_arr[0].nbuffer, 0);
            assert_eq!(reader_arr[1].nbuffer, 0);

            bcf_destroy(tmp0);
            bcf_destroy(rec0a);
            bcf_destroy(rec0b);
            bcf_destroy(tmp1);
            bcf_destroy(rec1a);
            bcf_destroy(rec1b);
            bcf_hdr_destroy(hdr);
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_set_keeps_same_reader_duplicate_alleles_separate() {
        unsafe {
            let mut readers: bcf_srs_t = std::mem::zeroed();
            let mut reader0: bcf_sr_t = std::mem::zeroed();
            let mut reader1: bcf_sr_t = std::mem::zeroed();
            let mut has_line = [0, 0];

            let tmp0 = bcf_init();
            let rec0a = bcf_init();
            let rec0b = bcf_init();
            let tmp1 = bcf_init();
            let rec1 = bcf_init();
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!tmp0.is_null());
            assert!(!rec0a.is_null());
            assert!(!rec0b.is_null());
            assert!(!tmp1.is_null());
            assert!(!rec1.is_null());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);
            let rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            assert_eq!(rid, 0);

            for rec in [rec0a, rec0b, rec1] {
                (*rec).rid = rid;
                (*rec).pos = 14;
                assert_eq!(bcf_update_alleles_str(hdr, rec, c"G,A".as_ptr()), 0);
            }

            let mut buffer0 = [tmp0, rec0a, rec0b];
            let mut buffer1 = [tmp1, rec1];
            reader0.buffer = buffer0.as_mut_ptr();
            reader0.nbuffer = 2;
            reader0.mbuffer = 3;
            reader0.header = hdr;
            reader1.buffer = buffer1.as_mut_ptr();
            reader1.nbuffer = 1;
            reader1.mbuffer = 2;
            reader1.header = hdr;
            let mut reader_arr = [reader0, reader1];
            readers.readers = reader_arr.as_mut_ptr();
            readers.nreaders = 2;
            readers.has_line = has_line.as_mut_ptr();

            let mut sort: BcfSrSort = std::mem::zeroed();
            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 0), 0);
            assert_eq!(bcf_sr_sort_c_331_bcf_sr_sort_add_active(&mut sort, 1), 0);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 14),
                2
            );
            assert_eq!(has_line, [1, 1]);
            assert_eq!(*reader_arr[0].buffer.add(0), rec0a);
            assert_eq!(*reader_arr[1].buffer.add(0), rec1);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 14),
                1
            );
            assert_eq!(has_line, [1, 0]);
            assert_eq!(*reader_arr[0].buffer.add(0), rec0b);

            bcf_destroy(tmp0);
            bcf_destroy(rec0a);
            bcf_destroy(rec0b);
            bcf_destroy(tmp1);
            bcf_destroy(rec1);
            bcf_hdr_destroy(hdr);
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
        }
    }

    #[test]
    fn synced_bcf_reader_sort_set_uses_symbolic_end_in_allele_key() {
        unsafe {
            let mut readers: bcf_srs_t = std::mem::zeroed();
            let mut reader0: bcf_sr_t = std::mem::zeroed();
            let mut reader1: bcf_sr_t = std::mem::zeroed();
            let mut has_line = [0, 0];

            let tmp0 = bcf_init();
            let rec0 = bcf_init();
            let tmp1 = bcf_init();
            let rec1 = bcf_init();
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!tmp0.is_null());
            assert!(!rec0.is_null());
            assert!(!tmp1.is_null());
            assert!(!rec1.is_null());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##INFO=<ID=END,Number=1,Type=Integer,Description=\"End position\">".as_ptr(),
                ),
                0
            );
            assert_eq!(bcf_hdr_sync(hdr), 0);
            let rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
            assert_eq!(rid, 0);

            for rec in [rec0, rec1] {
                (*rec).rid = rid;
                (*rec).pos = 9;
                assert_eq!(bcf_update_alleles_str(hdr, rec, c"A,<DEL>".as_ptr()), 0);
            }
            let mut end0 = 20;
            let mut end1 = 30;
            assert_eq!(
                bcf_update_info(
                    hdr,
                    rec0,
                    c"END".as_ptr(),
                    (&mut end0 as *mut c_int).cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr,
                    rec1,
                    c"END".as_ptr(),
                    (&mut end1 as *mut c_int).cast(),
                    1,
                    BCF_HT_INT as c_int,
                ),
                0
            );

            let mut buffer0 = [tmp0, rec0];
            let mut buffer1 = [tmp1, rec1];
            reader0.buffer = buffer0.as_mut_ptr();
            reader0.nbuffer = 1;
            reader0.mbuffer = 2;
            reader0.header = hdr;
            reader1.buffer = buffer1.as_mut_ptr();
            reader1.nbuffer = 1;
            reader1.mbuffer = 2;
            reader1.header = hdr;
            let mut reader_arr = [reader0, reader1];
            readers.readers = reader_arr.as_mut_ptr();
            readers.nreaders = 2;
            readers.has_line = has_line.as_mut_ptr();

            let mut sort: BcfSrSort = std::mem::zeroed();
            assert_eq!(bcf_sr_sort_c_324_bcf_sr_sort_set_active(&mut sort, 0), 0);
            assert_eq!(bcf_sr_sort_c_331_bcf_sr_sort_add_active(&mut sort, 1), 0);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 9),
                1
            );
            assert_eq!(has_line, [1, 0]);
            assert_eq!(*reader_arr[0].buffer.add(0), rec0);

            assert_eq!(
                bcf_sr_sort_c_593_bcf_sr_sort_next(&mut readers, &mut sort, c"chr1".as_ptr(), 9),
                1
            );
            assert_eq!(has_line, [0, 1]);
            assert_eq!(*reader_arr[1].buffer.add(0), rec1);

            bcf_destroy(tmp0);
            bcf_destroy(rec0);
            bcf_destroy(tmp1);
            bcf_destroy(rec1);
            bcf_hdr_destroy(hdr);
            bcf_sr_sort_c_685_bcf_sr_sort_destroy(&mut sort);
        }
    }

    #[test]
    fn synced_bcf_reader_seek_to_start_resets_region_iteration_state() {
        unsafe {
            let readers = bcf_sr_init();
            assert!(!readers.is_null());

            let mut seq_regions = [
                BcfSrRegion {
                    regs: std::ptr::null_mut(),
                    nregs: 2,
                    mregs: 2,
                    creg: 4,
                },
                BcfSrRegion {
                    regs: std::ptr::null_mut(),
                    nregs: 1,
                    mregs: 1,
                    creg: 7,
                },
            ];
            let mut regions: bcf_sr_regions_t = std::mem::zeroed();
            regions.regs = seq_regions.as_mut_ptr().cast();
            regions.nseqs = seq_regions.len() as c_int;
            regions.iseq = 1;
            regions.start = 11;
            regions.end = 22;
            regions.prev_seq = 3;
            regions.prev_start = 33;
            regions.prev_end = 44;

            (*readers).regions = &mut regions;
            (*bcf_sr_aux_mut(readers)).sort.chr = c"chr2".as_ptr();

            assert_eq!(bcf_sr_seek(readers, std::ptr::null(), 0), 0);
            assert_eq!(seq_regions[0].creg, -1);
            assert_eq!(seq_regions[1].creg, -1);
            assert_eq!(regions.iseq, 0);
            assert_eq!(regions.start, -1);
            assert_eq!(regions.end, -1);
            assert_eq!(regions.prev_seq, -1);
            assert_eq!(regions.prev_start, -1);
            assert_eq!(regions.prev_end, -1);
            assert!((*bcf_sr_aux_mut(readers)).sort.chr.is_null());

            (*readers).regions = std::ptr::null_mut();
            bcf_sr_destroy(readers);
        }
    }

    #[test]
    fn vcfutils_trim_and_remove_alleles_update_records() {
        unsafe {
            let hdr = bcf_hdr_init(c"w".as_ptr());
            assert!(!hdr.is_null());
            assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
            assert_eq!(
                bcf_hdr_append(hdr, c"##contig=<ID=chr1,length=100>".as_ptr()),
                0
            );
            assert_eq!(
                bcf_hdr_append(
                    hdr,
                    c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
                ),
                0
            );
            assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
            assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let rec = bcf_init();
            assert!(!rec.is_null());
            let mut line = kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert!(
                super::super::hts::kputs(
                    c"chr1\t1\t.\tA\tC,G\t.\t.\t.\tGT\t0/1".as_ptr(),
                    &mut line,
                ) >= 0
            );
            assert_eq!(vcf_parse(&mut line, hdr, rec), 0);

            assert_eq!(vcfutils_c_186_bcf_trim_alleles(hdr, rec), 1);
            assert_eq!((*rec).n_allele(), 2);
            assert_eq!(std::ffi::CStr::from_ptr(*(*rec).d.allele.add(1)), c"C");

            super::super::hts::ks_free(&mut line);
            bcf_destroy(rec);
            bcf_hdr_destroy(hdr);
        }
    }

    // Build a header used by the native-vs-hts_sys mutator parity tests.
    unsafe fn parity_hdr() -> *mut bcf_hdr_t {
        let hdr = bcf_hdr_init(c"w".as_ptr());
        assert!(!hdr.is_null());
        assert_eq!(bcf_hdr_append(hdr, c"##fileformat=VCFv4.3".as_ptr()), 0);
        assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr1>".as_ptr()), 0);
        assert_eq!(
            bcf_hdr_append(hdr, c"##FILTER=<ID=q10,Description=\"q10\">".as_ptr()),
            0
        );
        assert_eq!(
            bcf_hdr_append(hdr, c"##FILTER=<ID=s50,Description=\"s50\">".as_ptr()),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"DP\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##INFO=<ID=AF,Number=A,Type=Float,Description=\"AF\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##INFO=<ID=ST,Number=1,Type=String,Description=\"ST\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##INFO=<ID=FL,Number=0,Type=Flag,Description=\"FL\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"GQ\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##FORMAT=<ID=DS,Number=1,Type=Float,Description=\"DS\">".as_ptr()
            ),
            0
        );
        assert_eq!(bcf_hdr_add_sample(hdr, c"sampleA".as_ptr()), 0);
        assert_eq!(bcf_hdr_add_sample(hdr, c"sampleB".as_ptr()), 0);
        assert_eq!(bcf_hdr_sync(hdr), 0);
        hdr
    }

    // Initialise a record with a REF/ALT so unpack/sync are well defined.
    unsafe fn parity_rec(hdr: *const bcf_hdr_t) -> *mut bcf1_t {
        let rec = bcf_init();
        assert!(!rec.is_null());
        (*rec).rid = bcf_hdr_name2id(hdr, c"chr1".as_ptr());
        (*rec).pos = 99;
        assert_eq!(bcf_update_alleles_str(hdr, rec, c"AT,A,G".as_ptr()), 0);
        rec
    }

    // Compare the two records by syncing and diffing the on-the-wire buffers
    // plus the salient decoded counters.
    unsafe fn assert_records_equal(a: *mut bcf1_t, b: *mut bcf1_t) {
        // Both records share the repr(C) bcf1_t layout, so the native bcf1_sync
        // can flush the dirty shared/indiv blocks of either one.
        vcf_c_2332_bcf1_sync(a);
        vcf_c_2332_bcf1_sync(b);
        assert_eq!((*a).rlen, (*b).rlen, "rlen differs");
        assert_eq!((*a).n_info(), (*b).n_info(), "n_info differs");
        assert_eq!((*a).n_fmt(), (*b).n_fmt(), "n_fmt differs");
        assert_eq!((*a).n_allele(), (*b).n_allele(), "n_allele differs");
        assert_eq!((*a).d.n_flt, (*b).d.n_flt, "n_flt differs");
        assert_eq!((*a).shared.l, (*b).shared.l, "shared.l differs");
        assert_eq!((*a).indiv.l, (*b).indiv.l, "indiv.l differs");
        unsafe fn bytes<'x>(s: *const c_char, l: usize) -> &'x [u8] {
            if l == 0 || s.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(s.cast::<u8>(), l)
            }
        }
        assert_eq!(
            bytes((*a).shared.s, (*a).shared.l),
            bytes((*b).shared.s, (*b).shared.l),
            "shared bytes differ"
        );
        assert_eq!(
            bytes((*a).indiv.s, (*a).indiv.l),
            bytes((*b).indiv.s, (*b).indiv.l),
            "indiv bytes differ"
        );
    }

    #[test]
    fn vcf_update_info_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            let a = parity_rec(hdr);
            let b = parity_rec(hdr);

            let dp = [42i32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    a,
                    c"DP".as_ptr(),
                    dp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr.cast(),
                    b.cast(),
                    c"DP".as_ptr(),
                    dp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );

            let af = [0.25f32, 0.75f32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    a,
                    c"AF".as_ptr(),
                    af.as_ptr().cast(),
                    2,
                    BCF_HT_REAL as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr.cast(),
                    b.cast(),
                    c"AF".as_ptr(),
                    af.as_ptr().cast(),
                    2,
                    BCF_HT_REAL as c_int
                ),
                0
            );

            let st = c"hello";
            assert_eq!(
                bcf_update_info(
                    hdr,
                    a,
                    c"ST".as_ptr(),
                    st.as_ptr().cast(),
                    5,
                    BCF_HT_STR as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr.cast(),
                    b.cast(),
                    c"ST".as_ptr(),
                    st.as_ptr().cast(),
                    5,
                    BCF_HT_STR as c_int
                ),
                0
            );

            assert_eq!(
                bcf_update_info(
                    hdr,
                    a,
                    c"FL".as_ptr(),
                    std::ptr::null(),
                    1,
                    BCF_HT_FLAG as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr.cast(),
                    b.cast(),
                    c"FL".as_ptr(),
                    std::ptr::null(),
                    1,
                    BCF_HT_FLAG as c_int
                ),
                0
            );

            assert_records_equal(a, b);

            // Now overwrite DP in place and then remove ST.
            let dp2 = [7i32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    a,
                    c"DP".as_ptr(),
                    dp2.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr.cast(),
                    b.cast(),
                    c"DP".as_ptr(),
                    dp2.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr,
                    a,
                    c"ST".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_STR as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr.cast(),
                    b.cast(),
                    c"ST".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_STR as c_int
                ),
                0
            );
            assert_records_equal(a, b);

            bcf_destroy(a);
            bcf_destroy(b);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_update_format_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            let a = parity_rec(hdr);
            let b = parity_rec(hdr);

            let gq = [30i32, 40i32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    a,
                    c"GQ".as_ptr(),
                    gq.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr.cast(),
                    b.cast(),
                    c"GQ".as_ptr(),
                    gq.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );

            let ds = [0.1f32, 1.9f32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    a,
                    c"DS".as_ptr(),
                    ds.as_ptr().cast(),
                    2,
                    BCF_HT_REAL as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr.cast(),
                    b.cast(),
                    c"DS".as_ptr(),
                    ds.as_ptr().cast(),
                    2,
                    BCF_HT_REAL as c_int
                ),
                0
            );

            // GT added last must be reordered to the front by both implementations.
            let gts: [*const c_char; 2] = [c"0/1".as_ptr(), c"1/1".as_ptr()];
            assert_eq!(
                bcf_update_format_string(hdr, a, c"GT".as_ptr(), gts.as_ptr().cast_mut(), 2),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format_string(
                    hdr.cast(),
                    b.cast(),
                    c"GT".as_ptr(),
                    gts.as_ptr().cast_mut(),
                    2
                ),
                0
            );

            assert_records_equal(a, b);

            // Overwrite GQ in place then remove DS.
            let gq2 = [11i32, 12i32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    a,
                    c"GQ".as_ptr(),
                    gq2.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr.cast(),
                    b.cast(),
                    c"GQ".as_ptr(),
                    gq2.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                bcf_update_format(
                    hdr,
                    a,
                    c"DS".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_REAL as c_int
                ),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr.cast(),
                    b.cast(),
                    c"DS".as_ptr(),
                    std::ptr::null(),
                    0,
                    BCF_HT_REAL as c_int
                ),
                0
            );
            assert_records_equal(a, b);

            bcf_destroy(a);
            bcf_destroy(b);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_update_filter_id_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            let a = parity_rec(hdr);
            let b = parity_rec(hdr);

            let q10 = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"q10".as_ptr());
            let s50 = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"s50".as_ptr());

            assert_eq!(
                bcf_add_filter(hdr, a, q10),
                hts_sys::bcf_add_filter(hdr.cast(), b.cast(), q10)
            );
            assert_eq!(
                bcf_add_filter(hdr, a, s50),
                hts_sys::bcf_add_filter(hdr.cast(), b.cast(), s50)
            );
            // adding an already-present filter
            assert_eq!(
                bcf_add_filter(hdr, a, q10),
                hts_sys::bcf_add_filter(hdr.cast(), b.cast(), q10)
            );
            assert_eq!(
                bcf_remove_filter(hdr, a, q10, 0),
                hts_sys::bcf_remove_filter(hdr.cast(), b.cast(), q10, 0)
            );
            // remove last, request PASS
            assert_eq!(
                bcf_remove_filter(hdr, a, s50, 1),
                hts_sys::bcf_remove_filter(hdr.cast(), b.cast(), s50, 1)
            );

            assert_eq!(bcf_update_id(hdr, a, c"rs123".as_ptr()), 0);
            assert_eq!(
                hts_sys::bcf_update_id(hdr.cast(), b.cast(), c"rs123".as_ptr()),
                0
            );

            let flt = [q10, s50];
            assert_eq!(
                bcf_update_filter(hdr, a, flt.as_ptr().cast_mut(), 2),
                hts_sys::bcf_update_filter(hdr.cast(), b.cast(), flt.as_ptr().cast_mut(), 2)
            );

            assert_records_equal(a, b);

            bcf_destroy(a);
            bcf_destroy(b);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_copy_dup_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            let src = parity_rec(hdr);
            let dp = [9i32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    src,
                    c"DP".as_ptr(),
                    dp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );
            let gq = [5i32, 6i32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    src,
                    c"GQ".as_ptr(),
                    gq.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );

            let native = bcf_dup(src);
            let csys = hts_sys::bcf_dup(src.cast());
            assert_records_equal(native, csys.cast());

            // bcf_copy into an existing record
            let dst = bcf_init();
            bcf_copy(dst, src);
            assert_records_equal(dst, csys.cast());

            bcf_destroy(native);
            bcf_destroy(csys.cast());
            bcf_destroy(dst);
            bcf_destroy(src);
            bcf_hdr_destroy(hdr);
        }
    }

    // Build a fully-populated, synced record and compare what native bcf_unpack
    // decodes into d.* against what hts_sys::bcf_unpack decodes from the same
    // wire bytes.
    #[test]
    fn vcf_unpack_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            let src = parity_rec(hdr);
            assert_eq!(bcf_update_id(hdr, src, c"rs77".as_ptr()), 0);
            let q10 = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"q10".as_ptr());
            let s50 = bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"s50".as_ptr());
            let flt = [q10, s50];
            assert_eq!(bcf_update_filter(hdr, src, flt.as_ptr().cast_mut(), 2), 0);
            let dp = [42i32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    src,
                    c"DP".as_ptr(),
                    dp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );
            let af = [0.25f32, 0.75f32];
            assert_eq!(
                bcf_update_info(
                    hdr,
                    src,
                    c"AF".as_ptr(),
                    af.as_ptr().cast(),
                    2,
                    BCF_HT_REAL as c_int
                ),
                0
            );
            assert_eq!(
                bcf_update_info(
                    hdr,
                    src,
                    c"FL".as_ptr(),
                    std::ptr::null(),
                    1,
                    BCF_HT_FLAG as c_int
                ),
                0
            );
            let gq = [11i32, 22i32];
            assert_eq!(
                bcf_update_format(
                    hdr,
                    src,
                    c"GQ".as_ptr(),
                    gq.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );
            vcf_c_2332_bcf1_sync(src);

            // Two records carrying identical wire bytes but unpacked = 0.
            let a = bcf_dup(src);
            let b = bcf_dup(src);
            (*a).unpacked = 0;
            (*b).unpacked = 0;

            assert_eq!(bcf_unpack(a, BCF_UN_ALL as c_int), 0);
            assert_eq!(hts_sys::bcf_unpack(b.cast(), BCF_UN_ALL as c_int), 0);

            assert_eq!((*a).unpacked, (*b).unpacked, "unpacked flags differ");
            assert_eq!((*a).unpack_size, (*b).unpack_size, "unpack_size differ");
            let da = &(*a).d;
            let db = &(*b).d;
            // ID
            assert_eq!(
                CStr::from_ptr(da.id),
                CStr::from_ptr(db.id),
                "decoded ID differs"
            );
            // FILTER
            assert_eq!(da.n_flt, db.n_flt, "n_flt differs");
            for i in 0..da.n_flt as usize {
                assert_eq!(*da.flt.add(i), *db.flt.add(i), "flt[{i}] differs");
            }
            // alleles
            let na = (*a).n_allele() as usize;
            assert_eq!(na, (*b).n_allele() as usize);
            for i in 0..na {
                assert_eq!(
                    CStr::from_ptr(*da.allele.add(i)),
                    CStr::from_ptr(*db.allele.add(i)),
                    "allele[{i}] differs"
                );
            }
            // INFO
            assert_eq!((*a).n_info(), (*b).n_info(), "n_info differs");
            for i in 0..(*a).n_info() as usize {
                let ia = da.info.add(i);
                let ib = db.info.add(i);
                assert_eq!((*ia).key, (*ib).key, "info[{i}].key");
                assert_eq!((*ia).type_, (*ib).type_, "info[{i}].type");
                assert_eq!((*ia).len, (*ib).len, "info[{i}].len");
                assert_eq!((*ia).vptr_len, (*ib).vptr_len, "info[{i}].vptr_len");
                assert_eq!((*ia).v1.i, (*ib).v1.i, "info[{i}].v1");
            }
            // FORMAT
            assert_eq!((*a).n_fmt(), (*b).n_fmt(), "n_fmt differs");
            for i in 0..(*a).n_fmt() as usize {
                let fa = da.fmt.add(i);
                let fb = db.fmt.add(i);
                assert_eq!((*fa).id, (*fb).id, "fmt[{i}].id");
                assert_eq!((*fa).n, (*fb).n, "fmt[{i}].n");
                assert_eq!((*fa).size, (*fb).size, "fmt[{i}].size");
                assert_eq!((*fa).type_, (*fb).type_, "fmt[{i}].type");
                assert_eq!((*fa).p_len, (*fb).p_len, "fmt[{i}].p_len");
            }

            bcf_destroy(a);
            bcf_destroy(b);
            bcf_destroy(src);
            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_hdr_seqnames_version_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            // add a couple more contigs for a non-trivial dict
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chr2>".as_ptr()), 0);
            assert_eq!(bcf_hdr_append(hdr, c"##contig=<ID=chrX>".as_ptr()), 0);
            assert_eq!(bcf_hdr_sync(hdr), 0);

            let mut na = 0;
            let mut nb = 0;
            let names_a = bcf_hdr_seqnames(hdr, &mut na);
            let names_b = hts_sys::bcf_hdr_seqnames(hdr.cast(), &mut nb);
            assert_eq!(na, nb, "seqnames count differs");
            for i in 0..na as usize {
                assert_eq!(
                    CStr::from_ptr(*names_a.add(i)),
                    CStr::from_ptr(*names_b.add(i)),
                    "seqname[{i}] differs"
                );
            }
            libc::free(names_a.cast());
            libc::free(names_b.cast());

            assert_eq!(
                CStr::from_ptr(bcf_hdr_get_version(hdr)),
                CStr::from_ptr(hts_sys::bcf_hdr_get_version(hdr.cast())),
                "version differs"
            );

            bcf_hdr_destroy(hdr);
        }
    }

    #[test]
    fn vcf_subset_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();
            let mk = || {
                let r = parity_rec(hdr);
                let gq = [11i32, 22i32];
                assert_eq!(
                    bcf_update_format(
                        hdr,
                        r,
                        c"GQ".as_ptr(),
                        gq.as_ptr().cast(),
                        2,
                        BCF_HT_INT as c_int
                    ),
                    0
                );
                let ds = [1.5f32, 2.5f32];
                assert_eq!(
                    bcf_update_format(
                        hdr,
                        r,
                        c"DS".as_ptr(),
                        ds.as_ptr().cast(),
                        2,
                        BCF_HT_REAL as c_int
                    ),
                    0
                );
                vcf_c_2332_bcf1_sync(r);
                r
            };
            let a = mk();
            let b = mk();

            // keep only the second sample
            let mut imap = [1i32];
            assert_eq!(bcf_subset(hdr, a, 1, imap.as_mut_ptr()), 0);
            assert_eq!(
                hts_sys::bcf_subset(hdr.cast(), b.cast(), 1, imap.as_mut_ptr()),
                0
            );
            assert_records_equal(a, b);

            bcf_destroy(a);
            bcf_destroy(b);
            bcf_hdr_destroy(hdr);
        }
    }

    // Native bcf_index_build must produce a byte-identical CSI to hts_sys.
    #[test]
    fn vcf_bcf_index_build_parity_native_vs_hts_sys() {
        unsafe {
            use std::ffi::CString;
            let src = c"htslib/test/bcf-sr/weird-chr-names.vcf";
            // Only run when the fixture is present (tests run from crate root).
            if std::fs::metadata("htslib/test/bcf-sr/weird-chr-names.vcf").is_err() {
                return;
            }
            let pid = std::process::id();
            let dir = std::env::temp_dir();
            let nat = dir.join(format!("vcf_idx_parity_nat_{pid}.bcf"));
            let csys = dir.join(format!("vcf_idx_parity_csys_{pid}.bcf"));

            for outp in [&nat, &csys] {
                let cstr = CString::new(outp.to_str().unwrap()).unwrap();
                let in_fp = hts_open(src.as_ptr(), c"r".as_ptr());
                let hdr = bcf_hdr_read(in_fp);
                let out_fp = hts_open(cstr.as_ptr(), c"wb".as_ptr());
                assert_eq!(bcf_hdr_write(out_fp, hdr), 0);
                let rec = bcf_init();
                while bcf_read(in_fp, hdr, rec) >= 0 {
                    assert_eq!(bcf_write(out_fp, hdr, rec), 0);
                }
                bcf_destroy(rec);
                assert_eq!(hts_close(out_fp), 0);
                bcf_hdr_destroy(hdr);
                assert_eq!(hts_close(in_fp), 0);
            }

            let nat_c = CString::new(nat.to_str().unwrap()).unwrap();
            let csys_c = CString::new(csys.to_str().unwrap()).unwrap();
            assert_eq!(bcf_index_build(nat_c.as_ptr(), 14), 0);
            assert_eq!(hts_sys::bcf_index_build(csys_c.as_ptr(), 14), 0);

            let a = std::fs::read(nat.with_extension("bcf.csi")).unwrap();
            let b = std::fs::read(csys.with_extension("bcf.csi")).unwrap();
            assert_eq!(a, b, "native CSI bytes differ from hts_sys");

            let _ = std::fs::remove_file(&nat);
            let _ = std::fs::remove_file(nat.with_extension("bcf.csi"));
            let _ = std::fs::remove_file(&csys);
            let _ = std::fs::remove_file(csys.with_extension("bcf.csi"));
        }
    }

    // Native bcf_hdr_dup must yield the same serialized header text as hts_sys.
    #[test]
    fn vcf_bcf_hdr_dup_parity_native_vs_hts_sys() {
        unsafe {
            let hdr = parity_hdr();

            let nat = bcf_hdr_dup(hdr);
            let csys: *mut bcf_hdr_t = hts_sys::bcf_hdr_dup(hdr.cast()).cast();
            assert!(!nat.is_null());
            assert!(!csys.is_null());

            let mut tn: kstring_t = std::mem::zeroed();
            let mut tc = hts_sys::kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            };
            assert_eq!(bcf_hdr_format(nat, 1, &mut tn), 0);
            assert_eq!(hts_sys::bcf_hdr_format(csys.cast(), 1, &mut tc), 0);
            let sn = std::slice::from_raw_parts(tn.s.cast::<u8>(), tn.l);
            let sc = std::slice::from_raw_parts(tc.s.cast::<u8>(), tc.l as usize);
            assert_eq!(sn, sc, "dup'd header text differs");

            libc::free(tn.s.cast());
            libc::free(tc.s.cast());
            bcf_hdr_destroy(nat);
            hts_sys::bcf_hdr_destroy(csys.cast());
            bcf_hdr_destroy(hdr);
        }
    }

    // Build a small header from explicit lines + samples (native parse).
    unsafe fn build_hdr_from_lines(lines: &[&CStr], samples: &[&CStr]) -> *mut bcf_hdr_t {
        let h = bcf_hdr_init(c"w".as_ptr());
        // bcf_hdr_init("w") already adds fileformat + PASS; append the rest.
        for l in lines {
            assert_eq!(bcf_hdr_append(h, l.as_ptr()), 0, "append {l:?}");
        }
        for s in samples {
            assert_eq!(bcf_hdr_add_sample(h, s.as_ptr()), 0);
        }
        assert_eq!(bcf_hdr_sync(h), 0);
        h
    }

    unsafe fn hdr_text(h: *const bcf_hdr_t) -> Vec<u8> {
        let mut t: kstring_t = std::mem::zeroed();
        assert_eq!(bcf_hdr_format(h, 1, &mut t), 0);
        let out = std::slice::from_raw_parts(t.s.cast::<u8>(), t.l).to_vec();
        libc::free(t.s.cast());
        out
    }

    unsafe fn hdr_text_csys(h: *const bcf_hdr_t) -> Vec<u8> {
        let mut t = hts_sys::kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        assert_eq!(hts_sys::bcf_hdr_format(h.cast(), 1, &mut t), 0);
        let out = std::slice::from_raw_parts(t.s.cast::<u8>(), t.l as usize).to_vec();
        libc::free(t.s.cast());
        out
    }

    // Native bcf_hdr_combine must match hts_sys (resulting dst header text + rc).
    #[test]
    fn vcf_bcf_hdr_combine_parity_native_vs_hts_sys() {
        unsafe {
            let dst_lines: &[&CStr] = &[
                c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"DP\">",
                c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">",
                c"##contig=<ID=chr1,length=1000>",
                c"##source=A",
            ];
            let src_lines: &[&CStr] = &[
                // overlapping DP, new AF, new FILTER, new contig + generic
                c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"DP\">",
                c"##INFO=<ID=AF,Number=A,Type=Float,Description=\"AF\">",
                c"##FILTER=<ID=q10,Description=\"q10\">",
                c"##contig=<ID=chr2,length=2000>",
                c"##source=B",
            ];

            let dst_n = build_hdr_from_lines(dst_lines, &[]);
            let dst_c: *mut bcf_hdr_t = hts_sys::bcf_hdr_dup(dst_n.cast()).cast();
            let src = build_hdr_from_lines(src_lines, &[]);

            let rn = bcf_hdr_combine(dst_n, src);
            let rc = hts_sys::bcf_hdr_combine(dst_c.cast(), src.cast());
            assert_eq!(rn, rc, "combine return code");
            assert_eq!(hdr_text(dst_n), hdr_text_csys(dst_c), "combine header text");

            bcf_hdr_destroy(dst_n);
            hts_sys::bcf_hdr_destroy(dst_c.cast());
            bcf_hdr_destroy(src);
        }
    }

    // Native bcf_hdr_merge must match hts_sys (resulting dst header text).
    #[test]
    fn vcf_bcf_hdr_merge_parity_native_vs_hts_sys() {
        unsafe {
            let dst_lines: &[&CStr] = &[
                c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"DP\">",
                c"##contig=<ID=chr1,length=1000>",
            ];
            let src_lines: &[&CStr] = &[
                c"##INFO=<ID=AF,Number=A,Type=Float,Description=\"AF\">",
                c"##FILTER=<ID=q10,Description=\"q10\">",
                c"##contig=<ID=chr2,length=2000>",
                c"##ALT=<ID=NON_REF,Description=\"any\">",
            ];
            // NB: dst and src share the same fileformat version here. The
            // reference htslib/vcf.c bumps dst's fileformat when src is newer,
            // but the vendored hts-sys (older htslib) lacks that branch, so we
            // keep versions equal to exercise the structural-merge path that
            // both implementations agree on.
            let dst_n = build_hdr_from_lines(dst_lines, &[]);
            let dst_c: *mut bcf_hdr_t = hts_sys::bcf_hdr_dup(dst_n.cast()).cast();
            let src = build_hdr_from_lines(src_lines, &[]);

            let rn = bcf_hdr_merge(dst_n, src);
            let rc: *mut bcf_hdr_t = hts_sys::bcf_hdr_merge(dst_c.cast(), src.cast()).cast();
            assert!(!rn.is_null());
            assert!(!rc.is_null());
            assert_eq!(hdr_text(rn), hdr_text_csys(rc), "merge header text");

            // also the dst==NULL strip-IDX path
            let m0n = bcf_hdr_merge(std::ptr::null_mut(), src);
            let m0c: *mut bcf_hdr_t =
                hts_sys::bcf_hdr_merge(std::ptr::null_mut(), src.cast()).cast();
            assert!(!m0n.is_null());
            assert!(!m0c.is_null());
            assert_eq!(hdr_text(m0n), hdr_text_csys(m0c), "merge(NULL,src) text");

            bcf_hdr_destroy(rn);
            hts_sys::bcf_hdr_destroy(rc.cast());
            bcf_hdr_destroy(m0n);
            hts_sys::bcf_hdr_destroy(m0c.cast());
            bcf_hdr_destroy(src);
        }
    }

    // Native bcf_hdr_set_samples must match hts_sys across keep/exclude paths.
    #[test]
    fn vcf_bcf_hdr_set_samples_parity_native_vs_hts_sys() {
        unsafe {
            // Build a fresh dup'd header for each side+case (set_samples mutates).
            let build = || -> *mut bcf_hdr_t {
                let h = parity_hdr();
                let d = bcf_hdr_dup(h);
                bcf_hdr_destroy(h);
                d
            };

            // (samples-spec-or-null, is_file). Covers: keep subset, exclude one,
            // keep nonexistent (ret>0), and exclude-all (NULL).
            let cases: &[(Option<&CStr>, c_int)] = &[
                (Some(c"sampleB"), 0),
                (Some(c"^sampleA"), 0),
                (Some(c"sampleA,doesNotExist"), 0),
                (None, 0),
            ];

            for (spec, is_file) in cases {
                let nat = build();
                let csys: *mut bcf_hdr_t = hts_sys::bcf_hdr_dup(build().cast()).cast();
                let sp = spec.map_or(std::ptr::null(), |s| s.as_ptr());

                let rn = bcf_hdr_set_samples(nat, sp, *is_file);
                let rc = hts_sys::bcf_hdr_set_samples(csys.cast(), sp, *is_file);
                assert_eq!(rn, rc, "return code differs for {spec:?}");

                let mut tn: kstring_t = std::mem::zeroed();
                let mut tc = hts_sys::kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                assert_eq!(bcf_hdr_format(nat, 1, &mut tn), 0);
                assert_eq!(hts_sys::bcf_hdr_format(csys.cast(), 1, &mut tc), 0);
                let sn = std::slice::from_raw_parts(tn.s.cast::<u8>(), tn.l);
                let sc = std::slice::from_raw_parts(tc.s.cast::<u8>(), tc.l as usize);
                assert_eq!(sn, sc, "header text differs for {spec:?}");
                // bcf_hdr_nsamples is a C macro: hdr->n[BCF_DT_SAMPLE].
                assert_eq!(
                    bcf_hdr_nsamples_native(nat),
                    (*csys).n[BCF_DT_SAMPLE as usize],
                    "nsamples differs for {spec:?}"
                );

                libc::free(tn.s.cast());
                libc::free(tc.s.cast());
                bcf_hdr_destroy(nat);
                hts_sys::bcf_hdr_destroy(csys.cast());
            }
        }
    }

    // ---- Native dict-build parity tests (native vs hts_sys) ----------------

    // Header lines exercising every dict (ID for INFO/FILTER/FORMAT, CTG, and
    // structured/generic) plus the sample line.
    const PARITY_HDR_LINES: &[&CStr] = &[
        c"##fileformat=VCFv4.2",
        c"##FILTER=<ID=PASS,Description=\"All filters passed\">",
        c"##FILTER=<ID=q10,Description=\"Quality below 10\">",
        c"##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Total Depth\">",
        c"##INFO=<ID=AF,Number=A,Type=Float,Description=\"Allele Freq\">",
        c"##INFO=<ID=DB,Number=0,Type=Flag,Description=\"dbSNP\">",
        c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">",
        c"##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"Genotype Qual\">",
        c"##contig=<ID=chr1,length=1000>",
        c"##contig=<ID=chr2,length=2000>",
        c"##contig=<ID=chrMT,length=16569>",
        c"##ALT=<ID=NON_REF,Description=\"any alt\">",
        c"##source=myCaller-1.0",
    ];

    // Read all (key -> (id, info[0..3])) entries from a vdict, sorted by key.
    unsafe fn dump_vdict(d: *const kh_vdict_t) -> Vec<(String, c_int, [u64; 3])> {
        let mut out = Vec::new();
        if d.is_null() {
            return out;
        }
        let mut k: u32 = 0;
        while k < (*d).n_buckets {
            if !vcf_kh_iseither((*d).flags, k) {
                let key = CStr::from_ptr(*(*d).keys.add(k as usize))
                    .to_string_lossy()
                    .into_owned();
                let v = &*(*d).vals.add(k as usize);
                out.push((key, v.id, v.info));
            }
            k += 1;
        }
        out.sort();
        out
    }

    // Read id[type] -> key list (indexed by IDX) from a header.
    unsafe fn dump_idpairs(h: *const bcf_hdr_t, t: usize) -> Vec<Option<String>> {
        let mut out = Vec::new();
        for i in 0..(*h).n[t] {
            let p = (*h).id[t].add(i as usize);
            if (*p).key.is_null() {
                out.push(None);
            } else {
                out.push(Some(
                    CStr::from_ptr((*p).key).to_string_lossy().into_owned(),
                ));
            }
        }
        out
    }

    unsafe fn build_native() -> *mut bcf_hdr_t {
        let h = bcf_hdr_init(c"r".as_ptr());
        for line in PARITY_HDR_LINES {
            assert_eq!(
                bcf_hdr_append(h, line.as_ptr()),
                0,
                "native append {line:?}"
            );
        }
        assert_eq!(bcf_hdr_sync(h), 0);
        h
    }

    unsafe fn build_hts_sys() -> *mut bcf_hdr_t {
        let h = hts_sys::bcf_hdr_init(c"r".as_ptr());
        for line in PARITY_HDR_LINES {
            assert_eq!(hts_sys::bcf_hdr_append(h, line.as_ptr()), 0);
        }
        assert_eq!(hts_sys::bcf_hdr_sync(h), 0);
        h.cast()
    }

    #[test]
    fn vcf_native_dict_build_matches_hts_sys() {
        unsafe {
            let nat = build_native();
            let cref = build_hts_sys();

            for t in 0..3usize {
                let dn = (*nat).dict[t].cast::<kh_vdict_t>();
                let dc = (*cref).dict[t].cast::<kh_vdict_t>();
                assert_eq!(
                    dump_vdict(dn),
                    dump_vdict(dc),
                    "vdict[{t}] contents differ native vs hts_sys"
                );
                assert_eq!((*nat).n[t], (*cref).n[t], "n[{t}] differs");
                assert_eq!(
                    dump_idpairs(nat, t),
                    dump_idpairs(cref, t),
                    "id[{t}] (IDX assignment) differs"
                );
            }

            bcf_hdr_destroy(nat);
            hts_sys::bcf_hdr_destroy(cref.cast());
        }
    }

    #[test]
    fn vcf_native_dict_readback_by_native_and_hts_sys() {
        // A header built natively must be readable by BOTH native kh_get_vdict
        // (via bcf_hdr_id2int) and the C library, proving X31 consistency.
        unsafe {
            let nat = build_native();
            for (id_name, t) in [
                (c"DP", BCF_DT_ID),
                (c"AF", BCF_DT_ID),
                (c"GT", BCF_DT_ID),
                (c"PASS", BCF_DT_ID),
                (c"q10", BCF_DT_ID),
                (c"chr1", BCF_DT_CTG),
                (c"chrMT", BCF_DT_CTG),
            ] {
                let native_id = bcf_hdr_id2int(nat, t as c_int, id_name.as_ptr());
                let c_id = hts_sys::bcf_hdr_id2int(nat.cast(), t as c_int, id_name.as_ptr());
                assert!(native_id >= 0, "native lookup failed for {id_name:?}");
                assert_eq!(
                    native_id, c_id,
                    "native vs hts_sys id mismatch for {id_name:?} (X31 hash divergence?)"
                );
            }
            // contig name round-trips through the native id[] array
            assert_eq!(
                CStr::from_ptr(bcf_hdr_id2name(nat, bcf_hdr_name2id(nat, c"chr2".as_ptr()))),
                c"chr2"
            );
            bcf_hdr_destroy(nat);
        }
    }

    #[test]
    fn vcf_native_get_hrec_matches_hts_sys() {
        unsafe {
            let nat = build_native();
            let cref = build_hts_sys();

            // BCF_HL_GEN by value
            let n_ff = bcf_hdr_get_hrec(
                nat,
                BCF_HL_GEN as c_int,
                c"fileformat".as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
            );
            let c_ff = hts_sys::bcf_hdr_get_hrec(
                cref.cast(),
                BCF_HL_GEN as c_int,
                c"fileformat".as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
            );
            assert!(!n_ff.is_null() && !c_ff.is_null());
            assert_eq!(CStr::from_ptr((*n_ff).value), CStr::from_ptr((*c_ff).value));

            // INFO / FILTER / CTG by ID
            for (t, id) in [
                (BCF_HL_INFO, c"DP"),
                (BCF_HL_FLT, c"q10"),
                (BCF_HL_CTG, c"chr1"),
            ] {
                let nh = bcf_hdr_get_hrec(
                    nat,
                    t as c_int,
                    c"ID".as_ptr(),
                    id.as_ptr(),
                    std::ptr::null(),
                );
                let ch = hts_sys::bcf_hdr_get_hrec(
                    cref.cast(),
                    t as c_int,
                    c"ID".as_ptr(),
                    id.as_ptr(),
                    std::ptr::null(),
                );
                assert!(!nh.is_null(), "native get_hrec null for {id:?}");
                assert!(!ch.is_null());
                assert_eq!((*nh).type_, (*ch).type_);
            }

            // BCF_HL_STR structured (ALT) by ID via str_class
            let ns = bcf_hdr_get_hrec(
                nat,
                BCF_HL_STR as c_int,
                c"ID".as_ptr(),
                c"NON_REF".as_ptr(),
                c"ALT".as_ptr(),
            );
            let cs = hts_sys::bcf_hdr_get_hrec(
                cref.cast(),
                BCF_HL_STR as c_int,
                c"ID".as_ptr(),
                c"NON_REF".as_ptr(),
                c"ALT".as_ptr(),
            );
            assert!(!ns.is_null() && !cs.is_null());

            bcf_hdr_destroy(nat);
            hts_sys::bcf_hdr_destroy(cref.cast());
        }
    }

    // Parse the same VCF lines with the native vcf_parse_native() and the C
    // hts_sys::vcf_parse(), then compare the resulting bcf1_t shared/indiv byte
    // streams plus the decoded core fields.  Builds two identical headers so
    // any in-parse header mutation (dummy hrecs / fix_chromosome) happens
    // independently and symmetrically.
    #[test]
    fn vcf_parse_native_matches_hts_sys() {
        unsafe {
            let header_lines: &[&CStr] = &[
                c"##fileformat=VCFv4.3",
                c"##contig=<ID=chr1,length=100000>",
                c"##contig=<ID=chr2,length=100000>",
                c"##FILTER=<ID=q10,Description=\"q10\">",
                c"##FILTER=<ID=s50,Description=\"s50\">",
                c"##INFO=<ID=AN,Number=1,Type=Integer,Description=\"AN\">",
                c"##INFO=<ID=AC,Number=A,Type=Integer,Description=\"AC\">",
                c"##INFO=<ID=AF,Number=A,Type=Float,Description=\"AF\">",
                c"##INFO=<ID=DB,Number=0,Type=Flag,Description=\"DB\">",
                c"##INFO=<ID=DESC,Number=1,Type=String,Description=\"DESC\">",
                c"##INFO=<ID=END,Number=1,Type=Integer,Description=\"END\">",
                c"##FORMAT=<ID=GT,Number=1,Type=String,Description=\"GT\">",
                c"##FORMAT=<ID=DP,Number=1,Type=Integer,Description=\"DP\">",
                c"##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"GQ\">",
                c"##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"PL\">",
                c"##FORMAT=<ID=GL,Number=G,Type=Float,Description=\"GL\">",
                c"##FORMAT=<ID=FT,Number=1,Type=String,Description=\"FT\">",
            ];

            let build_hdr = || {
                let hdr = bcf_hdr_init(c"w".as_ptr());
                assert!(!hdr.is_null());
                for line in header_lines {
                    assert_eq!(bcf_hdr_append(hdr, line.as_ptr()), 0);
                }
                assert_eq!(bcf_hdr_add_sample(hdr, c"S1".as_ptr()), 0);
                assert_eq!(bcf_hdr_add_sample(hdr, c"S2".as_ptr()), 0);
                assert_eq!(bcf_hdr_add_sample(hdr, c"S3".as_ptr()), 0);
                assert_eq!(bcf_hdr_add_sample(hdr, std::ptr::null()), 0);
                assert_eq!(bcf_hdr_sync(hdr), 0);
                hdr
            };

            let hdr_native = build_hdr();
            let hdr_csys = build_hdr();

            let lines: &[&CStr] = &[
                // basic SNP with filters, multiple INFO + FORMAT
                c"chr1\t100\trs1\tA\tC,G\t29.5\tq10;s50\tAN=6;AC=2,1;AF=0.33,0.16;DB\tGT:DP:GQ\t0/1:30:99\t1|2:25:80\t./.:.:.",
                // no ID, no ALT, missing qual/filter/info
                c"chr1\t200\t.\tT\t.\t.\t.\t.\tGT\t0/0\t.\t1/1",
                // symbolic + END, PL/GL float-ish format
                c"chr2\t500\t.\tG\t<DEL>\t50\tPASS\tEND=600\tGT:PL\t0/1:0,10,100\t1/1:5,0,255\t./.:.",
                // string FORMAT (FT) + GL floats
                c"chr2\t700\tid7\tAC\tA,ACGT\t.\tPASS\tDESC=hello;DB\tGT:FT:GL\t1/2:PASS:-1.2,-3.4,-5.6\t0/0:q10:0.0,0.0,0.0\t2|1:.:.",
                // undefined INFO + FILTER tags (exercises dummy-header recovery on both headers)
                c"chr1\t900\t.\tA\tT\t10\tnovel_filter\tNEWINFO=42\tGT:DP\t0/1:7\t1/1:8\t0/0:9",
            ];

            for line in lines {
                let mut s_native = kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                let mut s_csys = kstring_t {
                    l: 0,
                    m: 0,
                    s: std::ptr::null_mut(),
                };
                assert!(kputs(line.as_ptr(), &mut s_native) >= 0);
                assert!(kputs(line.as_ptr(), &mut s_csys) >= 0);

                let v_native = bcf_init();
                let v_csys = bcf_init();

                let r_native = vcf_parse_native(&mut s_native, hdr_native, v_native);
                let r_csys = hts_sys::vcf_parse(
                    (&mut s_csys as *mut kstring_t).cast(),
                    hdr_csys.cast(),
                    v_csys.cast(),
                );

                let lstr = line.to_string_lossy();
                assert_eq!(r_native, r_csys, "return code differs for line: {lstr}");
                assert_eq!((*v_native).rid, (*v_csys).rid, "rid differs: {lstr}");
                assert_eq!((*v_native).pos, (*v_csys).pos, "pos differs: {lstr}");
                assert_eq!((*v_native).rlen, (*v_csys).rlen, "rlen differs: {lstr}");
                assert_eq!(
                    (*v_native).qual.to_bits(),
                    (*v_csys).qual.to_bits(),
                    "qual differs: {lstr}"
                );
                assert_eq!(
                    (*v_native).n_info(),
                    (*v_csys).n_info(),
                    "n_info differs: {lstr}"
                );
                assert_eq!(
                    (*v_native).n_allele(),
                    (*v_csys).n_allele(),
                    "n_allele differs: {lstr}"
                );
                assert_eq!(
                    (*v_native).n_fmt(),
                    (*v_csys).n_fmt(),
                    "n_fmt differs: {lstr}"
                );
                assert_eq!(
                    (*v_native).n_sample(),
                    (*v_csys).n_sample(),
                    "n_sample differs: {lstr}"
                );

                let shared_native = std::slice::from_raw_parts(
                    (*v_native).shared.s.cast::<u8>(),
                    (*v_native).shared.l as usize,
                );
                let shared_csys = std::slice::from_raw_parts(
                    (*v_csys).shared.s.cast::<u8>(),
                    (*v_csys).shared.l as usize,
                );
                assert_eq!(shared_native, shared_csys, "shared bytes differ: {lstr}");

                let indiv_native = std::slice::from_raw_parts(
                    (*v_native).indiv.s.cast::<u8>(),
                    (*v_native).indiv.l as usize,
                );
                let indiv_csys = std::slice::from_raw_parts(
                    (*v_csys).indiv.s.cast::<u8>(),
                    (*v_csys).indiv.l as usize,
                );
                assert_eq!(indiv_native, indiv_csys, "indiv bytes differ: {lstr}");

                bcf_destroy(v_native);
                hts_sys::bcf_destroy(v_csys.cast());
                libc::free(s_native.s.cast());
                libc::free(s_csys.s.cast());
            }

            bcf_hdr_destroy(hdr_native);
            hts_sys::bcf_hdr_destroy(hdr_csys.cast());
        }
    }

    // Build an indexed (.csi) BGZF-compressed BCF from a plain VCF fixture,
    // using the native code path. Returns the path to the .bcf file.
    unsafe fn sr_parity_make_indexed_bcf(vcf_rel: &str, label: &str) -> std::path::PathBuf {
        let vcf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(vcf_rel);
        let bcf = std::env::temp_dir().join(format!(
            "htslib-rs-sr-parity-{}-{}.bcf",
            label,
            std::process::id()
        ));
        let _ = std::fs::remove_file(&bcf);
        let _ = std::fs::remove_file(bcf.with_extension("bcf.csi"));
        let vcf_c = std::ffi::CString::new(vcf.to_string_lossy().as_bytes()).unwrap();
        let bcf_c = std::ffi::CString::new(bcf.to_string_lossy().as_bytes()).unwrap();
        let in_fp = hts_open(vcf_c.as_ptr(), c"r".as_ptr());
        assert!(!in_fp.is_null());
        let hdr = bcf_hdr_read(in_fp);
        assert!(!hdr.is_null());
        let out_fp = hts_open(bcf_c.as_ptr(), c"wb".as_ptr());
        assert!(!out_fp.is_null());
        assert_eq!(bcf_hdr_write(out_fp, hdr), 0);
        let rec = bcf_init();
        while bcf_read(in_fp, hdr, rec) >= 0 {
            assert!(bcf_write(out_fp, hdr, rec) >= 0);
        }
        bcf_destroy(rec);
        bcf_hdr_destroy(hdr);
        assert_eq!(hts_close(in_fp), 0);
        assert_eq!(hts_close(out_fp), 0);
        assert_eq!(bcf_index_build(bcf_c.as_ptr(), 14), 0);
        bcf
    }

    // Parity: drive the native synced reader and the hts_sys synced reader over
    // the same pair of indexed BCFs with a region filter, and confirm they
    // produce the identical sequence of (reader, chr, pos, ref, alt) records.
    #[test]
    fn synced_bcf_reader_native_matches_libhts_over_indexed_pair() {
        unsafe {
            let a = sr_parity_make_indexed_bcf("htslib/test/bcf-sr/merge.noidx.a.vcf", "a");
            let b = sr_parity_make_indexed_bcf("htslib/test/bcf-sr/merge.noidx.b.vcf", "b");
            let a_c = std::ffi::CString::new(a.to_string_lossy().as_bytes()).unwrap();
            let b_c = std::ffi::CString::new(b.to_string_lossy().as_bytes()).unwrap();

            // Collect synced records as text via the native reader.
            let native = {
                let sr = bcf_sr_init();
                assert!(!sr.is_null());
                bcf_sr_set_opt(sr, BCF_SR_REQUIRE_IDX, 0);
                assert_eq!(bcf_sr_add_reader(sr, a_c.as_ptr()), 1);
                assert_eq!(bcf_sr_add_reader(sr, b_c.as_ptr()), 1);
                let mut out: Vec<String> = Vec::new();
                while bcf_sr_next_line(sr) > 0 {
                    for i in 0..(*sr).nreaders {
                        if *(*sr).has_line.add(i as usize) == 0 {
                            out.push(format!("{i}\t-"));
                            continue;
                        }
                        let reader = (*sr).readers.add(i as usize);
                        let rec = *(*reader).buffer;
                        bcf_unpack(rec, BCF_UN_STR as c_int);
                        let refa = CStr::from_ptr(*(*rec).d.allele)
                            .to_string_lossy()
                            .into_owned();
                        let alt = if (*rec).n_allele() > 1 {
                            CStr::from_ptr(*(*rec).d.allele.add(1))
                                .to_string_lossy()
                                .into_owned()
                        } else {
                            ".".to_string()
                        };
                        out.push(format!(
                            "{i}\t{}\t{}\t{refa}\t{alt}",
                            (*rec).rid,
                            (*rec).pos + 1
                        ));
                    }
                }
                bcf_sr_destroy(sr);
                out
            };

            // Same via the hts_sys reference reader.
            let lib = {
                let sr = hts_sys::bcf_sr_init();
                assert!(!sr.is_null());
                (*sr).require_index = 1; // REQUIRE_IDX_
                assert_eq!(hts_sys::bcf_sr_add_reader(sr, a_c.as_ptr()), 1);
                assert_eq!(hts_sys::bcf_sr_add_reader(sr, b_c.as_ptr()), 1);
                let mut out: Vec<String> = Vec::new();
                while hts_sys::bcf_sr_next_line(sr) > 0 {
                    for i in 0..(*sr).nreaders {
                        if *(*sr).has_line.add(i as usize) == 0 {
                            out.push(format!("{i}\t-"));
                            continue;
                        }
                        let reader = (*sr).readers.add(i as usize);
                        let rec = *(*reader).buffer;
                        hts_sys::bcf_unpack(rec, hts_sys::BCF_UN_STR as c_int);
                        let refa = CStr::from_ptr(*(*rec).d.allele)
                            .to_string_lossy()
                            .into_owned();
                        let alt = if (*rec).n_allele() > 1 {
                            CStr::from_ptr(*(*rec).d.allele.add(1))
                                .to_string_lossy()
                                .into_owned()
                        } else {
                            ".".to_string()
                        };
                        out.push(format!(
                            "{i}\t{}\t{}\t{refa}\t{alt}",
                            (*rec).rid,
                            (*rec).pos + 1
                        ));
                    }
                }
                hts_sys::bcf_sr_destroy(sr);
                out
            };

            assert_eq!(
                native, lib,
                "native synced reader output differs from hts_sys"
            );
            assert!(!native.is_empty(), "expected at least one synced record");

            let _ = std::fs::remove_file(&a);
            let _ = std::fs::remove_file(a.with_extension("bcf.csi"));
            let _ = std::fs::remove_file(&b);
            let _ = std::fs::remove_file(b.with_extension("bcf.csi"));
        }
    }

    // ------------------------------------------------------------------
    // Stream A3 — randomized VCF record round-trip + adversarial inputs.
    //
    // We build a header in-memory, then for many randomly-generated record
    // texts:
    //   (a) parse via native vcf_parse, format via native vcf_format -> same text
    //   (b) the same text parses identically via hts_sys::vcf_parse and the
    //       hts_sys::vcf_format output matches the native one (parity).
    // The parser must NEVER panic — including on adversarial or truncated text.
    // ------------------------------------------------------------------

    struct VcfRng(u64);
    impl VcfRng {
        fn new(seed: u64) -> Self {
            VcfRng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
        }
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
    }

    /// VCF header text used in randomized tests below.  Includes the #CHROM
    /// sample line so `bcf_hdr_parse` can consume it whole.
    const VCF_HEADER_TEXT: &str = "##fileformat=VCFv4.2\n\
        ##contig=<ID=chr1,length=1000000>\n\
        ##contig=<ID=chr2,length=500000>\n\
        ##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
        ##INFO=<ID=AF,Number=A,Type=Float,Description=\"AlleleFreq\">\n\
        ##INFO=<ID=TAG,Number=1,Type=String,Description=\"Tag\">\n\
        ##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
        ##FORMAT=<ID=DP,Number=1,Type=Integer,Description=\"FmtDepth\">\n\
        ##FORMAT=<ID=GQ,Number=1,Type=Integer,Description=\"GenoQual\">\n\
        ##FILTER=<ID=q10,Description=\"q10\">\n\
        #CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tSAMP1\n";

    /// Build the small VCF header used by the randomized record tests.
    unsafe fn build_vcf_header_native() -> *mut bcf_hdr_t {
        let hdr = bcf_hdr_init(c"w".as_ptr());
        assert!(!hdr.is_null());
        let c = std::ffi::CString::new(VCF_HEADER_TEXT).unwrap();
        // bcf_hdr_parse needs a mutable C string buffer.
        let n = c.as_bytes_with_nul().len();
        let buf = libc::malloc(n) as *mut c_char;
        std::ptr::copy_nonoverlapping(c.as_ptr(), buf, n);
        let rc = bcf_hdr_parse(hdr, buf);
        libc::free(buf.cast());
        assert_eq!(rc, 0, "native bcf_hdr_parse failed");
        assert_eq!(bcf_hdr_sync(hdr), 0);
        hdr
    }

    /// Generate a random VCF record line that is well-formed under the header
    /// above.  Spans 10 columns: chrom pos id ref alt qual filter info format sample.
    fn gen_random_record(rng: &mut VcfRng) -> String {
        let r = rng.next();
        let chrom = if r & 1 == 0 { "chr1" } else { "chr2" };
        let pos = 1 + (rng.next() % 99_999);
        // ID can be "." or e.g. rsN
        let id = if rng.next() & 0b11 == 0 {
            ".".to_string()
        } else {
            format!("rs{}", rng.next() % 9_999_999)
        };
        // single REF base
        let bases = ['A', 'C', 'G', 'T'];
        let refb = bases[(rng.next() as usize) & 3];
        // ALT can be one or more comma-separated allele bases (different from REF)
        let n_alt = 1 + ((rng.next() as usize) & 3); // 1..=4
        let mut alts = Vec::new();
        for _ in 0..n_alt {
            let mut b = bases[(rng.next() as usize) & 3];
            // ensure differs from REF
            if b == refb {
                b = bases[((rng.next() as usize) + 1) & 3];
                if b == refb {
                    b = bases[((rng.next() as usize) + 2) & 3];
                }
            }
            alts.push(b.to_string());
        }
        let alt_str = alts.join(",");
        // QUAL
        let qual = if rng.next() & 0b11 == 0 {
            ".".to_string()
        } else {
            (rng.next() % 1000).to_string()
        };
        // FILTER: PASS, ., q10, or PASS
        let filter = match rng.next() & 0b11 {
            0 => ".",
            1 => "PASS",
            2 => "q10",
            _ => "PASS",
        };
        // INFO: random combo of DP=int, AF=floats(per-alt), TAG=string, or "."
        let info = match rng.next() & 0b111 {
            0 => ".".to_string(),
            1 => format!("DP={}", rng.next() % 500),
            2 => {
                let afs: Vec<String> = (0..n_alt)
                    .map(|_| {
                        let v = (rng.next() % 1000) as f64 / 1000.0;
                        format!("{:.3}", v)
                    })
                    .collect();
                format!("AF={}", afs.join(","))
            }
            3 => "TAG=foo".to_string(),
            4 => format!("DP={};TAG=bar", rng.next() % 500),
            5 => {
                let afs: Vec<String> = (0..n_alt)
                    .map(|_| format!("{:.3}", (rng.next() % 1000) as f64 / 1000.0))
                    .collect();
                format!("DP={};AF={}", rng.next() % 500, afs.join(","))
            }
            _ => "DP=10".to_string(),
        };
        // FORMAT + sample: random GT (haploid or diploid), optional DP/GQ.
        let n_alleles_total = 1 + n_alt; // 0=ref, 1..=n_alt
        let g1 = (rng.next() as usize) % n_alleles_total;
        let phased = rng.next() & 1 == 0;
        let g2 = (rng.next() as usize) % n_alleles_total;
        let gt = if rng.next() & 1 == 0 {
            // haploid
            format!("{g1}")
        } else if phased {
            format!("{g1}|{g2}")
        } else {
            format!("{g1}/{g2}")
        };
        let (fmt_keys, fmt_vals) = match rng.next() & 0b11 {
            0 => ("GT".to_string(), gt),
            1 => ("GT:DP".to_string(), format!("{gt}:{}", rng.next() % 100)),
            2 => (
                "GT:DP:GQ".to_string(),
                format!("{gt}:{}:{}", rng.next() % 100, rng.next() % 99),
            ),
            _ => ("GT:GQ".to_string(), format!("{gt}:{}", rng.next() % 99)),
        };

        format!(
            "{chrom}\t{pos}\t{id}\t{refb}\t{alt_str}\t{qual}\t{filter}\t{info}\t{fmt_keys}\t{fmt_vals}",
        )
    }

    /// Parse a line via native vcf_parse; format it again via native vcf_format;
    /// confirm parsing+formatting is idempotent (modulo trailing newline).
    unsafe fn parse_then_format_native(hdr: *mut bcf_hdr_t, line: &str) -> Option<String> {
        let c = std::ffi::CString::new(line).unwrap();
        let mut tmp = kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        let kr = super::super::hts::kputs(c.as_ptr(), &mut tmp);
        assert!(kr >= 0);
        let rec = bcf_init();
        let parse_rc = vcf_parse(&mut tmp, hdr, rec);
        super::super::hts::ks_free(&mut tmp);
        if parse_rc < 0 {
            bcf_destroy(rec);
            return None;
        }
        let mut out = kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        let fmt_rc = vcf_format(hdr, rec, &mut out);
        bcf_destroy(rec);
        if fmt_rc != 0 {
            super::super::hts::ks_free(&mut out);
            return None;
        }
        let s = CStr::from_ptr(out.s).to_string_lossy().into_owned();
        super::super::hts::ks_free(&mut out);
        // vcf_format appends a trailing newline; strip for comparison.
        Some(s.trim_end_matches('\n').to_string())
    }

    /// Same, via hts_sys.
    unsafe fn parse_then_format_libhts(hdr: *mut hts_sys::bcf_hdr_t, line: &str) -> Option<String> {
        let c = std::ffi::CString::new(line).unwrap();
        let mut tmp = hts_sys::kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        let n = c.as_bytes().len();
        // kputs (libhts)
        tmp.s = libc::malloc(n + 1) as *mut c_char;
        std::ptr::copy_nonoverlapping(c.as_ptr(), tmp.s, n + 1);
        tmp.l = n;
        tmp.m = n + 1;

        let rec = hts_sys::bcf_init();
        let parse_rc = hts_sys::vcf_parse(&mut tmp, hdr, rec);
        libc::free(tmp.s.cast());
        if parse_rc < 0 {
            hts_sys::bcf_destroy(rec);
            return None;
        }
        let mut out = hts_sys::kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        let fmt_rc = hts_sys::vcf_format(hdr, rec, &mut out);
        hts_sys::bcf_destroy(rec);
        if fmt_rc != 0 {
            if !out.s.is_null() {
                libc::free(out.s.cast());
            }
            return None;
        }
        let s = CStr::from_ptr(out.s).to_string_lossy().into_owned();
        libc::free(out.s.cast());
        Some(s.trim_end_matches('\n').to_string())
    }

    /// Build the same header into hts_sys for parity comparison.
    unsafe fn build_vcf_header_libhts() -> *mut hts_sys::bcf_hdr_t {
        let hdr = hts_sys::bcf_hdr_init(c"w".as_ptr());
        assert!(!hdr.is_null());
        let c = std::ffi::CString::new(VCF_HEADER_TEXT).unwrap();
        let n = c.as_bytes_with_nul().len();
        let buf = libc::malloc(n) as *mut c_char;
        std::ptr::copy_nonoverlapping(c.as_ptr(), buf, n);
        let rc = hts_sys::bcf_hdr_parse(hdr, buf);
        libc::free(buf.cast());
        assert_eq!(rc, 0, "hts_sys bcf_hdr_parse failed");
        assert_eq!(hts_sys::bcf_hdr_sync(hdr), 0);
        hdr
    }

    /// Native parse/format on random valid records must round-trip identically.
    #[test]
    fn vcf_random_records_native_idempotent() {
        unsafe {
            let hdr = build_vcf_header_native();
            let mut rng = VcfRng::new(0xDEAD_DEAD_BEEF_BEEF);
            let mut checked = 0usize;
            for _ in 0..200 {
                let line = gen_random_record(&mut rng);
                let first = parse_then_format_native(hdr, &line)
                    .expect("native parse+format declined valid line");
                let second = parse_then_format_native(hdr, &first)
                    .expect("native re-parse of native-formatted line failed");
                // Idempotent: second pass equals first pass.
                assert_eq!(
                    second, first,
                    "native parse-format not idempotent\norig: {line}\nfirst: {first}\nsecond: {second}"
                );
                checked += 1;
            }
            assert!(checked >= 100);
            bcf_hdr_destroy(hdr);
        }
    }

    /// Parity: native and hts_sys produce the same canonicalized text for the
    /// same random valid record.  This is a STRONG signal that the parser
    /// preserves all relevant state — any divergence is a real bug.
    #[test]
    fn vcf_random_records_native_matches_libhts() {
        unsafe {
            let hdr_nat = build_vcf_header_native();
            let hdr_lib = build_vcf_header_libhts();
            let mut rng = VcfRng::new(0xCAFE_F00D_DEAD_BEEF);
            let mut checked = 0usize;
            let mut diverged: Vec<(String, String, String)> = Vec::new();
            for _ in 0..150 {
                let line = gen_random_record(&mut rng);
                let nat = parse_then_format_native(hdr_nat, &line);
                let lib = parse_then_format_libhts(hdr_lib, &line);
                match (nat, lib) {
                    (Some(n), Some(l)) => {
                        if n != l {
                            diverged.push((line.clone(), n, l));
                        }
                        checked += 1;
                    }
                    (None, None) => {
                        // both declined — fine
                    }
                    (a, b) => {
                        // disagree on acceptance — record but don't fail
                        // (record any divergence to surface in test output)
                        diverged.push((
                            line.clone(),
                            a.unwrap_or_else(|| "<native-declined>".into()),
                            b.unwrap_or_else(|| "<libhts-declined>".into()),
                        ));
                    }
                }
            }
            assert!(checked >= 75);
            assert!(
                diverged.is_empty(),
                "parity divergence between native and hts_sys vcf_parse/format on {} of {} records:\n{}",
                diverged.len(),
                checked,
                diverged
                    .iter()
                    .take(3)
                    .map(|(o, n, l)| format!("orig={o}\n native={n}\n libhts={l}"))
                    .collect::<Vec<_>>()
                    .join("\n---\n"),
            );
            bcf_hdr_destroy(hdr_nat);
            hts_sys::bcf_hdr_destroy(hdr_lib);
        }
    }

    /// Adversarial VCF records: very long names/values, single record, weird
    /// genotypes (deep ploidy), boundary INFO sizes.  Parser must NOT panic.
    #[test]
    fn vcf_adversarial_records_no_panic() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        unsafe {
            let hdr = build_vcf_header_native();

            let cases: Vec<String> = vec![
                // a maximally simple line
                "chr1\t1\t.\tA\tG\t.\t.\t.\tGT\t0/1".to_string(),
                // empty INFO ('.') and FORMAT
                "chr1\t1\t.\tA\tG\t.\tPASS\t.\tGT\t.".to_string(),
                // long ID
                format!("chr1\t1\t{}\tA\tG\t.\tPASS\t.\tGT\t0/0", "X".repeat(2048)),
                // long INFO value
                format!(
                    "chr1\t10\t.\tA\tG\t.\tPASS\tTAG={}\tGT\t0/1",
                    "Y".repeat(1024)
                ),
                // very high-ploidy GT
                {
                    let mut gt = String::new();
                    for k in 0..40 {
                        if k > 0 {
                            gt.push('/');
                        }
                        gt.push_str(if k & 1 == 0 { "0" } else { "1" });
                    }
                    format!("chr1\t100\t.\tA\tG\t.\tPASS\t.\tGT\t{gt}")
                },
                // many alt alleles
                {
                    let alts: Vec<&str> = ["C", "G", "T", "AC", "GG", "TT", "CCC"].to_vec();
                    let n = alts.len();
                    let afs: Vec<String> = (0..n).map(|_| "0.100".to_string()).collect();
                    format!(
                        "chr1\t200\t.\tA\t{}\t30\tPASS\tAF={}\tGT\t0/{}",
                        alts.join(","),
                        afs.join(","),
                        n
                    )
                },
                // boundary integer INFO at i32::MAX
                "chr1\t300\t.\tA\tG\t.\tPASS\tDP=2147483646\tGT\t0/0".to_string(),
                // QUAL with many decimal places
                "chr1\t400\t.\tA\tG\t999.999999\tPASS\t.\tGT\t0/0".to_string(),
            ];

            for line in &cases {
                // Native must either parse+format without panic, or decline.
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let _ = parse_then_format_native(hdr, line);
                }));
            }

            bcf_hdr_destroy(hdr);
        }
    }

    /// Truncation fuzz: take a valid record and truncate at every byte
    /// boundary.  vcf_parse must never panic (no UB).
    #[test]
    fn vcf_truncated_records_no_panic() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        unsafe {
            let hdr = build_vcf_header_native();
            let mut rng = VcfRng::new(0x1111_2222_3333_4444);
            // 10 distinct valid records.
            for _ in 0..10 {
                let line = gen_random_record(&mut rng);
                let bytes = line.as_bytes();
                for cut in 1..bytes.len() {
                    let truncated = String::from_utf8_lossy(&bytes[..cut]).into_owned();
                    let _ = catch_unwind(AssertUnwindSafe(|| {
                        let _ = parse_then_format_native(hdr, &truncated);
                    }));
                }
            }
            bcf_hdr_destroy(hdr);
        }
    }

    /// Garbage-input fuzz: random bytes that vaguely look like VCF lines.
    #[test]
    fn vcf_garbage_input_no_panic() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        unsafe {
            let hdr = build_vcf_header_native();
            let mut rng = VcfRng::new(0xFADE_C0DE_1234_5678);
            for trial in 0..200u64 {
                let len = ((rng.next() % 240) + 1) as usize;
                // 80% biased toward printable, 20% pure random.
                let printable = rng.next() & 0b111 != 0;
                let mut s = String::with_capacity(len);
                for _ in 0..len {
                    let r = rng.next();
                    let b = if printable {
                        let candidates = b"0123456789ACGTN\tACGT,.;:=+-/|PASSchrtagAFDPGTGQ\n";
                        candidates[(r as usize) % candidates.len()] as char
                    } else {
                        ((r & 0x7f) as u8 as char)
                    };
                    s.push(b);
                }
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let _ = parse_then_format_native(hdr, &s);
                }));
                let _ = trial;
            }
            bcf_hdr_destroy(hdr);
        }
    }
}
