use std::ffi::{c_char, c_int, c_void, CStr};
use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};

use super::bgzf::{bgzf_index_build_init, bgzf_useek, bgzf_utell};
use super::hfile::{hseek, htslib_hfile_h_155_htell as htell};
use super::hts::{
    htsFile, hts_close, hts_get_bgzfp, hts_getline, hts_idx_t, hts_open, hts_pos_t, i16_to_le,
    i32_to_le, i64_to_le, kbitset_t, kbs_destroy, kbs_exists, kbs_init, kbs_insert, kputc, kputc_,
    kputd, kputsn, kputw, ks_resize, kstring_t, le_to_float, le_to_i16, le_to_i32, le_to_i64,
    le_to_i8, le_to_u32, size_t,
    toupper_c, BGZF, HTS_FORMAT_VCF, KS_SEP_LINE,
};

pub type bcf_hdr_t = hts_sys::bcf_hdr_t;
pub type bcf1_t = hts_sys::bcf1_t;
pub type bcf_fmt_t = hts_sys::bcf_fmt_t;
pub type bcf_info_t = hts_sys::bcf_info_t;
pub type bcf_hrec_t = hts_sys::bcf_hrec_t;
pub type bcf_idinfo_t = hts_sys::bcf_idinfo_t;
pub type bcf_idpair_t = hts_sys::bcf_idpair_t;
pub type bcf_sr_t = hts_sys::bcf_sr_t;
pub type bcf_sr_regions_t = hts_sys::bcf_sr_regions_t;
pub type bcf_srs_t = hts_sys::bcf_srs_t;
pub type bcf_variant_t = hts_sys::variant_t;
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
pub const BCF_SR_REQUIRE_IDX: hts_sys::bcf_sr_opt_t = 0;
pub const BCF_SR_PAIR_LOGIC: hts_sys::bcf_sr_opt_t = 1;
pub const BCF_SR_ALLOW_NO_IDX: hts_sys::bcf_sr_opt_t = 2;
pub const BCF_SR_REGIONS_OVERLAP: hts_sys::bcf_sr_opt_t = 3;
pub const BCF_SR_TARGETS_OVERLAP: hts_sys::bcf_sr_opt_t = 4;
const BCF_IS_64BIT: c_int = 1 << 30;
const BCF_HT_LONG: c_int = BCF_HT_INT as c_int | 0x100;
const BCF_MIN_BT_INT32: i64 = -2_147_483_640;
const BCF_MIN_BT_INT64: i64 = -9_223_372_036_854_775_800;
const REQUIRE_IDX_: c_int = 1;
const ALLOW_NO_IDX_: c_int = 2;
const MAX_CSI_COOR: hts_pos_t = (1_i64 << 44) - 1;
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
    score: [u8; 256],
    nvar: c_int,
    mvar: c_int,
    var: *mut c_void,
    nvset: c_int,
    mvset: c_int,
    mpmat: c_int,
    pmat: *mut c_int,
    ngrp: c_int,
    mgrp: c_int,
    mcnt: c_int,
    cnt: *mut c_int,
    grp: *mut c_void,
    vset: *mut c_void,
    vcf_buf: *mut c_void,
    sr: *mut bcf_srs_t,
    grp_str2int: *mut c_void,
    var_str2int: *mut c_void,
    str_: kstring_t,
    moff: c_int,
    noff: c_int,
    off: *mut c_int,
    mcharp: c_int,
    charp: *mut *mut c_char,
    chr: *const c_char,
    pos: hts_pos_t,
    nsr: c_int,
    msr: c_int,
    pair: c_int,
    nactive: c_int,
    mactive: c_int,
    active: *mut c_int,
}

#[repr(C)]
struct BcfSrSortVcfBuf {
    nrec: c_int,
    mrec: c_int,
    rec: *mut *mut bcf1_t,
}

#[repr(C)]
struct BcfSrSortVar {
    str_: *mut c_char,
    type_: c_int,
    nalt: c_int,
    nvcf: c_int,
    mvcf: c_int,
    vcf: *mut c_int,
    rec: *mut *mut bcf1_t,
    mask: *mut kbitset_t,
}

#[repr(C)]
struct BcfSrSortGrp {
    key: *mut c_char,
    nvar: c_int,
    mvar: c_int,
    var: *mut c_int,
    nvcf: c_int,
}

#[repr(C)]
struct BcfSrSortVarSet {
    nvar: c_int,
    mvar: c_int,
    var: *mut c_int,
    cnt: c_int,
    mask: *mut kbitset_t,
}

#[repr(C)]
struct BcfSrAux {
    sort: BcfSrSort,
    regions_overlap: c_int,
    targets_overlap: c_int,
    closefile: *mut c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BcfSrRegion1 {
    start: hts_pos_t,
    end: hts_pos_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BcfSrRegion {
    regs: *mut BcfSrRegion1,
    nregs: c_int,
    mregs: c_int,
    creg: c_int,
}

unsafe fn bcf_sr_aux_mut(readers: *mut bcf_srs_t) -> *mut BcfSrAux {
    unsafe { (*readers).aux.cast::<BcfSrAux>() }
}

unsafe fn bcf_sr_sort_reserve_active(srt: *mut BcfSrSort, need: c_int) -> c_int {
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

pub unsafe fn bcf_sr_sort_c_324_bcf_sr_sort_set_active(srt: *mut BcfSrSort, idx: c_int) -> c_int {
    unsafe {
        let Some(need) = idx.checked_add(1) else {
            return -1;
        };
        if idx < 0 || bcf_sr_sort_reserve_active(srt, need) < 0 {
            return -1;
        }
        (*srt).nactive = 1;
        *(*srt).active = idx;
        0
    }
}

pub unsafe fn bcf_sr_sort_c_331_bcf_sr_sort_add_active(srt: *mut BcfSrSort, idx: c_int) -> c_int {
    unsafe {
        if srt.is_null() || idx < 0 {
            return -1;
        }
        let Some(idx_need) = idx.checked_add(1) else {
            return -1;
        };
        let Some(active_need) = (*srt).nactive.checked_add(1) else {
            return -1;
        };
        let need = idx_need.max(active_need);
        if bcf_sr_sort_reserve_active(srt, need) < 0 {
            return -1;
        }
        *(*srt).active.add((*srt).nactive as usize) = idx;
        (*srt).nactive += 1;
        0
    }
}

unsafe fn bcf_sr_sort_reserve_vcf_buf(readers: *mut bcf_srs_t, srt: *mut BcfSrSort) -> c_int {
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

unsafe fn bcf_sr_sort_shift_reader_buffer(reader: *mut bcf_sr_t, j: c_int) -> c_int {
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

unsafe fn bcf_sr_sort_append_empty_row(vcf_buf: *mut BcfSrSortVcfBuf, nreaders: c_int) -> c_int {
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

unsafe fn bcf_sr_sort_record_key(hdr: *const bcf_hdr_t, rec: *mut bcf1_t) -> Option<Vec<u8>> {
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

fn bcf_sr_sort_disambiguate_duplicate_key(
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

pub unsafe fn bcf_sr_sort_c_338_bcf_sr_sort_set(
    readers: *mut bcf_srs_t,
    srt: *mut BcfSrSort,
    chr: *const c_char,
    min_pos: hts_pos_t,
) -> c_int {
    unsafe {
        if readers.is_null() || srt.is_null() || chr.is_null() {
            return -1;
        }
        if bcf_sr_sort_reserve_vcf_buf(readers, srt) < 0 {
            return -1;
        }
        let vcf_buf = (*srt).vcf_buf.cast::<BcfSrSortVcfBuf>();
        for i in 0..(*srt).nsr.max(0) as usize {
            (*vcf_buf.add(i)).nrec = 0;
        }

        let mut records: Vec<(Vec<u8>, c_int, *mut bcf1_t)> = Vec::new();
        for iact in 0..(*srt).nactive as usize {
            let reader_idx = *(*srt).active.add(iact);
            if reader_idx < 0 || reader_idx >= (*readers).nreaders {
                return -1;
            }
            let reader = (*readers).readers.add(reader_idx as usize);
            if (*reader).buffer.is_null() || (*reader).nbuffer <= 0 {
                continue;
            }

            let rid = if !(*reader).header.is_null() {
                bcf_hdr_name2id((*reader).header, chr)
            } else if !(*(*reader).buffer.add(1)).is_null() {
                (*(*(*reader).buffer.add(1))).rid
            } else {
                -1
            };
            if rid < 0 {
                continue;
            }

            for irec in 1..=(*reader).nbuffer as usize {
                let rec = *(*reader).buffer.add(irec);
                if rec.is_null() || (*rec).rid != rid || (*rec).pos != min_pos {
                    break;
                }
                let Some(mut key) = bcf_sr_sort_record_key((*reader).header, rec) else {
                    return -1;
                };
                bcf_sr_sort_disambiguate_duplicate_key(&mut key, &records, reader_idx);
                records.push((key, reader_idx, rec));
            }
        }

        let mut row_keys: Vec<Vec<u8>> = Vec::new();
        for (key, reader_idx, rec) in records {
            let row = row_keys.iter().position(|row_key| row_key == &key);
            let row = match row {
                Some(row) => row,
                None => {
                    if bcf_sr_sort_append_empty_row(vcf_buf, (*srt).nsr) < 0 {
                        return -1;
                    }
                    row_keys.push(key);
                    row_keys.len() - 1
                }
            };
            let buf = vcf_buf.add(reader_idx as usize);
            if *(*buf).rec.add(row) == rec {
                continue;
            }
            if !(*(*buf).rec.add(row)).is_null() {
                if bcf_sr_sort_append_empty_row(vcf_buf, (*srt).nsr) < 0 {
                    return -1;
                }
                row_keys.push(row_keys[row].clone());
                let next_row = row_keys.len() - 1;
                *(*buf).rec.add(next_row) = rec;
            } else {
                *(*buf).rec.add(row) = rec;
            }
        }

        (*srt).chr = chr;
        (*srt).pos = min_pos;
        0
    }
}

pub unsafe fn bcf_sr_sort_c_593_bcf_sr_sort_next(
    readers: *mut bcf_srs_t,
    srt: *mut BcfSrSort,
    chr: *const c_char,
    min_pos: hts_pos_t,
) -> c_int {
    unsafe {
        if readers.is_null()
            || srt.is_null()
            || chr.is_null()
            || (*srt).nactive <= 0
            || (*srt).active.is_null()
            || (*readers).readers.is_null()
            || (*readers).has_line.is_null()
        {
            return -1;
        }
        if bcf_sr_sort_reserve_vcf_buf(readers, srt) < 0 {
            return -1;
        }

        if (*srt).nactive == 1 {
            if (*readers).nreaders > 1 {
                std::ptr::write_bytes((*readers).has_line, 0, (*readers).nreaders as usize);
            }
            let active = *(*srt).active;
            if active < 0 || active >= (*readers).nreaders {
                return -1;
            }
            let reader = (*readers).readers.add(active as usize);
            if (*reader).buffer.is_null()
                || (*reader).nbuffer < 1
                || (*(*reader).buffer.add(1)).is_null()
                || (*(*(*reader).buffer.add(1))).pos != min_pos
            {
                return -1;
            }
            if bcf_sr_sort_shift_reader_buffer(reader, 1) < 0 {
                return -1;
            }
            *(*readers).has_line.add(active as usize) = 1;
            return 1;
        }

        if (*srt).chr.is_null() || (*srt).pos != min_pos || libc::strcmp((*srt).chr, chr) != 0 {
            if bcf_sr_sort_c_338_bcf_sr_sort_set(readers, srt, chr, min_pos) < 0 {
                return -1;
            }
        }

        let vcf_buf = (*srt).vcf_buf.cast::<BcfSrSortVcfBuf>();
        if vcf_buf.is_null() || (*vcf_buf).nrec <= 0 {
            return 0;
        }

        let mut nret = 0;
        for i in 0..(*srt).nsr.max(0) as usize {
            let buf = vcf_buf.add(i);
            if (*buf).nrec > 0 && !(*buf).rec.is_null() && !(*(*buf).rec).is_null() {
                if (*srt).sr.is_null() || (*(*srt).sr).readers.is_null() {
                    return -1;
                }
                let reader = (*(*srt).sr).readers.add(i);
                let rec = *(*buf).rec;
                let mut j = 1;
                while j <= (*reader).nbuffer && *(*reader).buffer.add(j as usize) != rec {
                    j += 1;
                }
                if j > (*reader).nbuffer || bcf_sr_sort_shift_reader_buffer(reader, j) < 0 {
                    return -1;
                }
                nret += 1;
                *(*readers).has_line.add(i) = 1;
            } else {
                *(*readers).has_line.add(i) = 0;
            }

            if (*buf).nrec > 0 {
                (*buf).nrec -= 1;
                if (*buf).nrec > 0 {
                    std::ptr::copy((*buf).rec.add(1), (*buf).rec, (*buf).nrec as usize);
                }
            }
        }
        nret
    }
}

pub unsafe fn bcf_sr_sort_c_662_bcf_sr_sort_remove_reader(
    _readers: *mut bcf_srs_t,
    srt: *mut BcfSrSort,
    i: c_int,
) {
    unsafe {
        if srt.is_null() || (*srt).vcf_buf.is_null() || i < 0 || i >= (*srt).nsr {
            return;
        }

        let vcf_buf = (*srt).vcf_buf.cast::<BcfSrSortVcfBuf>();
        libc::free((*vcf_buf.add(i as usize)).rec.cast());
        if i + 1 < (*srt).nsr {
            std::ptr::copy(
                vcf_buf.add(i as usize + 1),
                vcf_buf.add(i as usize),
                ((*srt).nsr - i - 1) as usize,
            );
        }
        std::ptr::write_bytes(vcf_buf.add((*srt).nsr as usize - 1), 0, 1);
    }
}

pub unsafe fn bcf_sr_sort_c_675_bcf_sr_sort_init(srt: *mut BcfSrSort) -> *mut BcfSrSort {
    unsafe {
        if srt.is_null() {
            return libc::calloc(1, size_of::<BcfSrSort>()).cast::<BcfSrSort>();
        }
        std::ptr::write_bytes(srt, 0, 1);
        srt
    }
}

pub unsafe fn bcf_sr_sort_c_681_bcf_sr_sort_reset(srt: *mut BcfSrSort) {
    unsafe {
        if !srt.is_null() {
            (*srt).chr = std::ptr::null();
        }
    }
}

pub unsafe fn bcf_sr_sort_c_685_bcf_sr_sort_destroy(srt: *mut BcfSrSort) {
    unsafe {
        if srt.is_null() {
            return;
        }

        libc::free((*srt).active.cast());
        super::sam::khash_str2int_destroy_free((*srt).var_str2int);
        super::sam::khash_str2int_destroy_free((*srt).grp_str2int);

        let vcf_buf = (*srt).vcf_buf.cast::<BcfSrSortVcfBuf>();
        for i in 0..(*srt).nsr.max(0) as usize {
            libc::free((*vcf_buf.add(i)).rec.cast());
        }
        libc::free((*srt).vcf_buf);

        let var = (*srt).var.cast::<BcfSrSortVar>();
        for i in 0..(*srt).mvar.max(0) as usize {
            libc::free((*var.add(i)).str_.cast());
            libc::free((*var.add(i)).vcf.cast());
            libc::free((*var.add(i)).rec.cast());
            kbs_destroy((*var.add(i)).mask);
        }
        libc::free((*srt).var);

        let grp = (*srt).grp.cast::<BcfSrSortGrp>();
        for i in 0..(*srt).mgrp.max(0) as usize {
            libc::free((*grp.add(i)).var.cast());
        }
        libc::free((*srt).grp);

        let vset = (*srt).vset.cast::<BcfSrSortVarSet>();
        for i in 0..(*srt).mvset.max(0) as usize {
            kbs_destroy((*vset.add(i)).mask);
            libc::free((*vset.add(i)).var.cast());
        }
        libc::free((*srt).vset);

        libc::free((*srt).str_.s.cast());
        libc::free((*srt).off.cast());
        libc::free((*srt).charp.cast());
        libc::free((*srt).cnt.cast());
        libc::free((*srt).pmat.cast());
        std::ptr::write_bytes(srt, 0, 1);
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

unsafe fn bcf_sr_regions_set_overlap(regions: *mut bcf_sr_regions_t, overlap: c_int) {
    unsafe {
        *bcf_sr_regions_overlap_ptr(regions) = overlap;
    }
}

unsafe fn bcf_sr_regions_alloc() -> *mut bcf_sr_regions_t {
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

unsafe fn bcf_sr_regions_add(
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

unsafe fn regions_merge(reg: *mut BcfSrRegion) {
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

unsafe fn advance_creg(reg: *mut BcfSrRegion) -> c_int {
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

pub unsafe fn synced_bcf_reader_c_1070_regions_merge(reg: *mut c_void) {
    unsafe { regions_merge(reg.cast::<BcfSrRegion>()) }
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
                || ((*hrec).type_ != BCF_HL_INFO as c_int
                    && (*hrec).type_ != BCF_HL_FMT as c_int)
            {
                continue;
            }

            let number_idx = hts_sys::bcf_hrec_find_key(hrec, c"Number".as_ptr());
            if number_idx < 0 {
                continue;
            }
            let Some(vl_code) = bcf_vcf45_number_code(*(*hrec).vals.add(number_idx as usize))
            else {
                continue;
            };

            let id_idx = hts_sys::bcf_hrec_find_key(hrec, c"ID".as_ptr());
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

unsafe fn regions_sort_and_merge(reg: *mut bcf_sr_regions_t) {
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

pub unsafe fn synced_bcf_reader_c_1085__regions_sort_and_merge(reg: *mut bcf_sr_regions_t) {
    unsafe { regions_sort_and_merge(reg) }
}

unsafe fn bcf_sr_regions_destroy_translated(reg: *mut bcf_sr_regions_t) {
    unsafe {
        if reg.is_null() {
            return;
        }

        libc::free((*reg).fname.cast());
        if !(*reg).itr.is_null() {
            hts_sys::hts_itr_destroy((*reg).itr);
        }
        if !(*reg).tbx.is_null() {
            super::tbx::tbx_destroy((*reg).tbx);
        }
        if !(*reg).file.is_null() {
            let _ = hts_sys::hts_close((*reg).file);
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

unsafe fn regions_init_string(str_: *const c_char) -> *mut bcf_sr_regions_t {
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

unsafe fn regions_parse_line(
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

unsafe fn bcf_sr_seek_start(readers: *mut bcf_srs_t) {
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
const SW_FWD: c_int = 0;
const SW_BWD: c_int = 1;

#[repr(C)]
pub struct bcf_sweep_t {
    file: *mut htsFile,
    hdr: *mut bcf_hdr_t,
    fp: *mut BGZF,

    direction: c_int,  // to tell if the direction has changed
    block_size: c_int, // the size of uncompressed data to hold in memory
    rec: *mut bcf1_t,  // bcf buffer
    nrec: c_int,
    mrec: c_int, // number of used records; total size of the buffer
    lrid: c_int,
    lpos: c_int,
    lnals: c_int,
    lals_len: c_int,
    mlals: c_int, // to check uniqueness of a record
    lals: *mut c_char,

    idx: *mut u64, // uncompressed offsets of VCF/BCF records
    iidx: c_int,
    nidx: c_int,
    midx: c_int,    // i: current offset; n: used; m: allocated
    idx_done: c_int, // the index is built during the first pass
}

pub unsafe fn bcf_hdr_init(mode: *const c_char) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_init(mode)
}

pub unsafe fn bcf_hdr_destroy(h: *mut bcf_hdr_t) {
    hts_sys::bcf_hdr_destroy(h)
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
unsafe fn sw_utell(fp: *mut htsFile) -> i64 {
    if ((*fp).bitfields & (1 << 4)) != 0 {
        bgzf_utell((*fp).fp.bgzf)
    } else {
        htell((*fp).fp.hfile) as i64
    }
}

// htslib/htslib/vcf.h: #define bcf_read1(fp,h,v) bcf_read((fp),(h),(v))
#[inline]
unsafe fn bcf_read1(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    bcf_read(fp, h, v)
}

// htslib/htslib/vcf.h: #define bcf_empty1(v) bcf_empty(v)
#[inline]
unsafe fn bcf_empty1(v: *mut bcf1_t) {
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
    if libc::memcmp(
        (*sw).lals.cast(),
        allele0.cast(),
        len as usize,
    ) != 0
    {
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
unsafe fn sw_fill_buffer(sw: *mut bcf_sweep_t) -> c_int {
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
        hts_expand0_bcf1(
            (*sw).nrec + 1,
            &mut (*sw).mrec,
            &mut (*sw).rec,
        );
        rec = (*sw).rec.add((*sw).nrec as usize);
    }
    sw_rec_save(sw, (*sw).rec);

    0 // FIXME: check for errs in this function
}

pub unsafe fn bcf_sweep_init(fname: *const c_char) -> *mut bcf_sweep_t {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_init().
    let sw = libc::calloc(1, size_of::<bcf_sweep_t>()).cast::<bcf_sweep_t>();
    (*sw).file = hts_open(fname, c"r".as_ptr());
    (*sw).fp = hts_get_bgzfp((*sw).file);
    if !(*sw).fp.is_null() {
        bgzf_index_build_init((*sw).fp);
    }
    (*sw).hdr = bcf_hdr_read((*sw).file);
    (*sw).mrec = 1;
    (*sw).rec = libc::calloc((*sw).mrec as usize, size_of::<bcf1_t>()).cast::<bcf1_t>();
    (*sw).block_size = 1024 * 1024 * 3;
    (*sw).direction = SW_FWD;
    sw
}

pub unsafe fn bcf_sweep_destroy(sw: *mut bcf_sweep_t) {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_destroy().
    for i in 0..(*sw).mrec {
        bcf_empty1((*sw).rec.add(i as usize));
    }
    libc::free((*sw).idx.cast());
    libc::free((*sw).rec.cast());
    libc::free((*sw).lals.cast());
    bcf_hdr_destroy((*sw).hdr);
    hts_close((*sw).file);
    libc::free(sw.cast());
}

// Native translation of htslib/vcf_sweep.c sw_seek().
unsafe fn sw_seek(sw: *mut bcf_sweep_t, direction: c_int) {
    (*sw).direction = direction;
    if direction == SW_FWD {
        sw_useek((*sw).file, *(*sw).idx as i64, 0);
    } else {
        (*sw).iidx = (*sw).nidx;
        (*sw).nrec = 0;
    }
}

pub unsafe fn bcf_sweep_fwd(sw: *mut bcf_sweep_t) -> *mut bcf1_t {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_fwd().
    if (*sw).direction == SW_BWD {
        sw_seek(sw, SW_FWD);
    }

    let pos = sw_utell((*sw).file);

    let rec = (*sw).rec;
    let ret = bcf_read1((*sw).file, (*sw).hdr, rec);

    if ret != 0 {
        // last record, get ready for sweeping backwards
        (*sw).idx_done = 1;
        if !(*sw).fp.is_null() {
            (*(*sw).fp).idx_build_otf = 0;
        }
        sw_seek(sw, SW_BWD);
        return std::ptr::null_mut();
    }

    if (*sw).idx_done == 0
        && ((*sw).nidx == 0
            || pos - *(*sw).idx.add((*sw).nidx as usize - 1) as i64 > (*sw).block_size as i64)
    {
        (*sw).nidx += 1;
        hts_expand_u64((*sw).nidx, &mut (*sw).midx, &mut (*sw).idx);
        *(*sw).idx.add((*sw).nidx as usize - 1) = pos as u64;
    }
    rec
}

pub unsafe fn bcf_sweep_bwd(sw: *mut bcf_sweep_t) -> *mut bcf1_t {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_bwd().
    if (*sw).direction == SW_FWD {
        sw_seek(sw, SW_BWD);
    }
    if (*sw).nrec == 0 {
        sw_fill_buffer(sw);
    }
    if (*sw).nrec == 0 {
        return std::ptr::null_mut();
    }
    (*sw).nrec -= 1;
    (*sw).rec.add((*sw).nrec as usize)
}

pub unsafe fn bcf_sweep_hdr(sw: *mut bcf_sweep_t) -> *mut bcf_hdr_t {
    (*sw).hdr
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
unsafe fn hts_expand_u64(n: c_int, m: *mut c_int, ptr: *mut *mut u64) {
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
        std::ptr::write_bytes(
            (*ptr).add(old_m as usize),
            0,
            (*m - old_m) as usize,
        );
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

pub unsafe fn bcf_hdr_read(fp: *mut htsFile) -> *mut bcf_hdr_t {
    let hdr = hts_sys::bcf_hdr_read(fp.cast());
    bcf_hdr_fix_vcf45_vl_types(hdr);
    hdr
}

pub unsafe fn bcf_hdr_set_samples(
    hdr: *mut bcf_hdr_t,
    samples: *const c_char,
    is_file: c_int,
) -> c_int {
    hts_sys::bcf_hdr_set_samples(hdr, samples, is_file)
}

pub unsafe fn bcf_sr_add_hreader(
    readers: *mut bcf_srs_t,
    file_ptr: *mut htsFile,
    autoclose: c_int,
    idxname: *const c_char,
) -> c_int {
    unsafe {
        if readers.is_null() {
            *libc::__errno_location() = libc::EINVAL;
            return 0;
        }
        if file_ptr.is_null() {
            (*readers).errnum = hts_sys::bcf_sr_error_api_usage_error;
            *libc::__errno_location() = libc::EINVAL;
            return 0;
        }
        if !idxname.is_null() || (*file_ptr).fn_.is_null() {
            (*readers).errnum = hts_sys::bcf_sr_error_api_usage_error;
            *libc::__errno_location() = libc::EINVAL;
            return 0;
        }
        let ret = bcf_sr_add_reader(readers, (*file_ptr).fn_);
        if autoclose != 0 {
            let _ = hts_close(file_ptr);
        }
        ret
    }
}

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
        (*rec).indiv.l -=
            ((*fmt_i).p_len as usize - (dst as usize - (*fmt_i).p as usize)) as u64;
        (*fmt_i).p_len = (dst as usize - (*fmt_i).p as usize) as u32;
    }
    (*rec).unpacked |= BCF_UN_FMT as c_int;

    (*rec).set_n_sample((*hdr).n[BCF_DT_SAMPLE as usize] as u32);
    0
}

pub unsafe fn bcf_hdr_write(fp: *mut htsFile, h: *mut bcf_hdr_t) -> c_int {
    hts_sys::bcf_hdr_write(fp.cast(), h)
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
        let ret = hts_sys::vcf_parse(s.cast(), h, v);
        if ret == 0 {
            vcf44_repair_prefixed_gt_phasing(h, v, &phasing);
        }
        ret
    } else {
        hts_sys::vcf_parse(s.cast(), h, v)
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
    if hdr.is_null() {
        return false;
    }
    let version = hts_sys::bcf_hdr_get_version(hdr);
    if version.is_null() {
        return false;
    }
    let version = std::ffi::CStr::from_ptr(version).to_bytes();
    vcf_version_number(version).is_some_and(|version| version >= 4_004_000)
}

unsafe fn vcf_hdr_maybe_version_ge_44(hdr: *const bcf_hdr_t) -> bool {
    if hdr.is_null() {
        return false;
    }
    let version = hts_sys::bcf_hdr_get_version(hdr);
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
            x if x == BCF_BT_INT16 as c_int => {
                le_to_i16(ptr.add(i * size_of::<i16>())) as i32
            }
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

pub unsafe fn vcf_format(h: *const bcf_hdr_t, v: *const bcf1_t, s: *mut kstring_t) -> c_int {
    let ret = hts_sys::vcf_format(h, v, s.cast());
    if ret == 0 && vcf_hdr_maybe_version_ge_44(h) {
        if vcf44_format_gt_fields(h, v.cast_mut(), s) < 0 {
            return -1;
        }
    }
    ret
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
            x if x == BCF_BT_INT16 as c_int => {
                le_to_i16(ptr.add(i * size_of::<i16>())) as i32
            }
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

pub unsafe fn bcf_read(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    if !fp.is_null() && (*fp).format.format == HTS_FORMAT_VCF {
        let ret = hts_getline(fp, KS_SEP_LINE, &mut (*fp).line);
        if ret < 0 {
            return ret;
        }
        return vcf_parse(&mut (*fp).line, h, v);
    }

    hts_sys::bcf_read(fp.cast(), h, v)
}

// htslib/hts_internal.h HTS_MAX_EXT_LEN
const HTS_MAX_EXT_LEN: usize = 9;
// htslib/htslib/hts.h HTS_IDX_DELIM
const HTS_IDX_DELIM: &[u8] = b"##idx##";

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

const ORIG_VAR_TYPES: u32 = VCF_SNP
    | VCF_MNP
    | VCF_INDEL
    | VCF_OTHER
    | VCF_BND
    | VCF_OVERLAP;

pub unsafe fn vcf_c_5474_bcf_get_variant_types(rec: *mut bcf1_t) -> c_int {
    if (*rec).d.var_type == -1 && vcf_c_5444_bcf_set_variant_types(rec) != 0 {
        let err = CStr::from_ptr(libc::strerror(*libc::__errno_location())).to_string_lossy();
        let msg =
            std::ffi::CString::new(format!("Couldn't get variant types: {}", err)).unwrap_or_default();
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
        let msg =
            std::ffi::CString::new(format!("Couldn't get variant types: {}", err)).unwrap_or_default();
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
        return ((*(*rec).d.var.add(ith_allele as usize)).type_ == VCF_REF as c_int)
            as c_int;
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
    if rembuffer
        > libc::strlen(description) + (if rembuffer == maxbuffer { 0 } else { 1 })
    {
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
            branch!(
                le_to_i16,
                size_of::<i16>(),
                bcf_int16_vector_end as i16
            );
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

    if vcf_hdr_version_ge_44(hdr) {
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

pub unsafe fn bcf_write(fp: *mut htsFile, h: *mut bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::bcf_write(fp.cast(), h, v)
}

pub unsafe fn vcf_hdr_read(fp: *mut htsFile) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_read(fp.cast())
}

pub unsafe fn vcf_hdr_write(fp: *mut htsFile, h: *const bcf_hdr_t) -> c_int {
    hts_sys::vcf_hdr_write(fp.cast(), h)
}

pub unsafe fn vcf_read(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    bcf_read(fp, h, v)
}

pub unsafe fn vcf_write(fp: *mut htsFile, h: *const bcf_hdr_t, v: *mut bcf1_t) -> c_int {
    hts_sys::vcf_write(fp.cast(), h, v)
}

pub unsafe fn bcf_readrec(
    fp: *mut BGZF,
    null: *mut c_void,
    v: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    hts_sys::bcf_readrec(fp.cast(), null, v, tid, beg, end)
}

pub unsafe fn vcf_write_line(fp: *mut htsFile, line: *mut kstring_t) -> c_int {
    hts_sys::vcf_write_line(fp.cast(), line.cast())
}

pub unsafe fn bcf_hdr_dup(hdr: *const bcf_hdr_t) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_dup(hdr)
}

pub unsafe fn bcf_hdr_combine(dst: *mut bcf_hdr_t, src: *const bcf_hdr_t) -> c_int {
    hts_sys::bcf_hdr_combine(dst, src)
}

pub unsafe fn bcf_hdr_merge(dst: *mut bcf_hdr_t, src: *const bcf_hdr_t) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_merge(dst, src)
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
        let is_integer = (1u32 << BCF_BT_INT8)
            | (1u32 << BCF_BT_INT16)
            | (1u32 << BCF_BT_INT32);
        let is_valid_type = is_integer
            | (1u32 << BCF_BT_NULL)
            | (1u32 << BCF_BT_FLOAT)
            | (1u32 << BCF_BT_CHAR);
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
            if ((1u32 << type_) & is_valid_type) == 0
                || (type_ == BCF_BT_NULL as c_int && num > 0)
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
            if ((1u32 << type_) & is_valid_type) == 0
                || (type_ == BCF_BT_NULL as c_int && num > 0)
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
                        x if x == BCF_BT_INT8 as c_int => {
                            le_to_i8((*svlen).vptr.add(i)) as i64
                        }
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

pub unsafe fn bcf_hdr_subset(
    h0: *const bcf_hdr_t,
    n: c_int,
    samples: *const *mut c_char,
    imap: *mut c_int,
) -> *mut bcf_hdr_t {
    hts_sys::bcf_hdr_subset(h0, n, samples, imap)
}

pub unsafe fn bcf_hdr_add_sample(hdr: *mut bcf_hdr_t, sample: *const c_char) -> c_int {
    hts_sys::bcf_hdr_add_sample(hdr, sample)
}

pub unsafe fn bcf_hdr_set(hdr: *mut bcf_hdr_t, fname: *const c_char) -> c_int {
    hts_sys::bcf_hdr_set(hdr, fname)
}

pub unsafe fn bcf_hdr_format(hdr: *const bcf_hdr_t, is_bcf: c_int, str_: *mut kstring_t) -> c_int {
    hts_sys::bcf_hdr_format(hdr, is_bcf, str_.cast())
}

pub unsafe fn bcf_hdr_fmt_text(
    hdr: *const bcf_hdr_t,
    is_bcf: c_int,
    len: *mut c_int,
) -> *mut c_char {
    hts_sys::bcf_hdr_fmt_text(hdr, is_bcf, len)
}

pub unsafe fn bcf_hdr_append(h: *mut bcf_hdr_t, line: *const c_char) -> c_int {
    let ret = hts_sys::bcf_hdr_append(h, line);
    if ret == 0 {
        bcf_hdr_fix_vcf45_vl_types(h);
    }
    ret
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

pub unsafe fn bcf_hdr_set_version(hdr: *mut bcf_hdr_t, version: *const c_char) -> c_int {
    hts_sys::bcf_hdr_set_version(hdr, version)
}

pub unsafe fn bcf_hdr_remove(h: *mut bcf_hdr_t, type_: c_int, key: *const c_char) {
    hts_sys::bcf_hdr_remove(h, type_, key)
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

pub unsafe fn bcf_hdr_parse(hdr: *mut bcf_hdr_t, htxt: *mut c_char) -> c_int {
    let ret = hts_sys::bcf_hdr_parse(hdr, htxt);
    if ret == 0 {
        bcf_hdr_fix_vcf45_vl_types(hdr);
    }
    ret
}

pub unsafe fn bcf_hdr_sync(h: *mut bcf_hdr_t) -> c_int {
    let ret = hts_sys::bcf_hdr_sync(h);
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
        (((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info
            [type_ as usize]
            >> 8)
            & 0xf) as c_int
    }
}

unsafe fn bcf_hdr_id2number_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        ((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info[type_ as usize]
            >> 12) as c_int
    }
}

unsafe fn bcf_hdr_id2type_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        (((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info
            [type_ as usize]
            >> 4)
            & 0xf) as c_int
    }
}

unsafe fn bcf_hdr_id2coltype_rs(hdr: *const bcf_hdr_t, type_: c_int, int_id: c_int) -> c_int {
    unsafe {
        ((*(*(*hdr).id[BCF_DT_ID as usize].add(int_id as usize)).val).info[type_ as usize]
            & 0xf) as c_int
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

pub unsafe fn bcf_hdr_parse_line(
    h: *const bcf_hdr_t,
    line: *const c_char,
    len: *mut c_int,
) -> *mut bcf_hrec_t {
    hts_sys::bcf_hdr_parse_line(h, line, len)
}

pub unsafe fn bcf_hrec_format(hrec: *const bcf_hrec_t, str_: *mut kstring_t) -> c_int {
    hts_sys::bcf_hrec_format(hrec, str_.cast())
}

pub unsafe fn bcf_hdr_add_hrec(hdr: *mut bcf_hdr_t, hrec: *mut bcf_hrec_t) -> c_int {
    hts_sys::bcf_hdr_add_hrec(hdr, hrec)
}

pub unsafe fn bcf_hdr_get_hrec(
    hdr: *const bcf_hdr_t,
    type_: c_int,
    key: *const c_char,
    value: *const c_char,
    str_class: *const c_char,
) -> *mut bcf_hrec_t {
    hts_sys::bcf_hdr_get_hrec(hdr, type_, key, value, str_class)
}

pub unsafe fn bcf_hrec_dup(hrec: *mut bcf_hrec_t) -> *mut bcf_hrec_t {
    hts_sys::bcf_hrec_dup(hrec)
}

pub unsafe fn bcf_hrec_add_key(hrec: *mut bcf_hrec_t, str_: *const c_char, len: usize) -> c_int {
    hts_sys::bcf_hrec_add_key(hrec, str_, len as hts_sys::size_t)
}

pub unsafe fn bcf_hrec_set_val(
    hrec: *mut bcf_hrec_t,
    i: c_int,
    str_: *const c_char,
    len: usize,
    is_quoted: c_int,
) -> c_int {
    hts_sys::bcf_hrec_set_val(hrec, i, str_, len as hts_sys::size_t, is_quoted)
}

pub unsafe fn bcf_hrec_find_key(hrec: *mut bcf_hrec_t, key: *const c_char) -> c_int {
    hts_sys::bcf_hrec_find_key(hrec, key)
}

pub unsafe fn hrec_add_idx(hrec: *mut bcf_hrec_t, idx: c_int) -> c_int {
    hts_sys::hrec_add_idx(hrec, idx)
}

pub unsafe fn bcf_hrec_destroy(hrec: *mut bcf_hrec_t) {
    hts_sys::bcf_hrec_destroy(hrec)
}

// Native translation of htslib/vcf.c bcf_subset().
pub unsafe fn bcf_subset(_h: *const bcf_hdr_t, v: *mut bcf1_t, n: c_int, imap: *mut c_int) -> c_int {
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
    (*v).indiv.l = ind.l as u64;
    (*v).indiv.m = ind.m as u64;
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
        if c0 == b'<' as c_char
            && c1 == b'X' as c_char
            && *allele.add(2) == b'>' as c_char
        {
            i += 1;
            continue;
        }
        if c0 == b'<' as c_char
            && c1 == b'*' as c_char
            && *allele.add(2) == b'>' as c_char
        {
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

pub unsafe fn bcf_add_id(hdr: *const bcf_hdr_t, line: *mut bcf1_t, id: *const c_char) -> c_int {
    hts_sys::bcf_add_id(hdr, line, id)
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
        kputsn(values.cast::<c_char>(), (nps * nsamples) as usize, &mut str_);
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
        *dst = crate::htslib_rs::c_compat::malloc(
            (size_of::<*mut c_char>() as u64) * (nsmpl as u64),
        )
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
    let mut ret: c_int;

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
    // __ac_X31_hash_string for KHASH_MAP_INIT_STR. Match it exactly.
    let mut i = super::hts::__ac_X31_hash_string(key) & mask;
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

pub unsafe fn bcf_enc_vchar(s: *mut kstring_t, l: c_int, a: *const c_char) -> c_int {
    hts_sys::bcf_enc_vchar(s.cast(), l, a)
}

pub unsafe fn bcf_enc_vint(s: *mut kstring_t, n: c_int, a: *mut i32, wsize: c_int) -> c_int {
    hts_sys::bcf_enc_vint(s.cast(), n, a, wsize)
}

pub unsafe fn bcf_enc_vfloat(s: *mut kstring_t, n: c_int, a: *mut f32) -> c_int {
    hts_sys::bcf_enc_vfloat(s.cast(), n, a)
}

pub unsafe fn bcf_index_load2(fn_: *const c_char, fnidx: *const c_char) -> *mut hts_idx_t {
    hts_sys::bcf_index_load2(fn_, fnidx).cast()
}

pub unsafe fn bcf_index_load3(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut hts_idx_t {
    hts_sys::bcf_index_load3(fn_, fnidx, flags).cast()
}

pub unsafe fn bcf_index_build(fn_: *const c_char, min_shift: c_int) -> c_int {
    hts_sys::bcf_index_build(fn_, min_shift)
}

pub unsafe fn bcf_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
) -> c_int {
    hts_sys::bcf_index_build2(fn_, fnidx, min_shift)
}

pub unsafe fn bcf_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    n_threads: c_int,
) -> c_int {
    hts_sys::bcf_index_build3(fn_, fnidx, min_shift, n_threads)
}

pub unsafe fn bcf_idx_init(
    fp: *mut htsFile,
    h: *mut bcf_hdr_t,
    min_shift: c_int,
    fnidx: *const c_char,
) -> c_int {
    hts_sys::bcf_idx_init(fp.cast(), h, min_shift, fnidx)
}

pub unsafe fn bcf_idx_save(fp: *mut htsFile) -> c_int {
    super::sam::sam_idx_save(fp)
}

pub unsafe fn vcfutils_c_254_is_special_info_type(name: *const c_char) -> c_int {
    match *name as u8 {
        b'C' => {
            if *name.add(1) == b'I' as c_char
                && (libc::strcmp(name.add(2), c"CN".as_ptr()) == 0
                    || libc::strcmp(name.add(2), c"END".as_ptr()) == 0
                    || libc::strcmp(name.add(2), c"LEN".as_ptr()) == 0
                    || libc::strcmp(name.add(2), c"POS".as_ptr()) == 0)
            {
                return 2;
            }
        }
        b'M' => {
            if *name.add(1) == b'E' as c_char
                && (libc::strcmp(name, c"MEINFO".as_ptr()) == 0
                    || libc::strcmp(name, c"METRANS".as_ptr()) == 0)
            {
                return 4;
            }
        }
        _ => {}
    }
    1
}

pub unsafe fn vcfutils_c_280_get_int32_info_value(info: *const bcf_info_t, index: usize) -> i32 {
    let len = if (*info).len > 0 {
        (*info).len as usize
    } else {
        0
    };
    if index >= len {
        return bcf_int32_missing;
    }

    match (*info).type_ {
        x if x == BCF_BT_INT8 as c_int => {
            let val = le_to_i8((*info).vptr.add(index)) as i32;
            if val > bcf_int8_vector_end {
                val
            } else {
                bcf_int32_vector_end - (bcf_int8_vector_end - val)
            }
        }
        x if x == BCF_BT_INT16 as c_int => {
            let val = le_to_i16((*info).vptr.add(index * size_of::<i16>())) as i32;
            if val > bcf_int16_vector_end {
                val
            } else {
                bcf_int32_vector_end - (bcf_int16_vector_end - val)
            }
        }
        x if x == BCF_BT_INT32 as c_int => {
            le_to_i32((*info).vptr.add(index * size_of::<i32>()))
        }
        x if x == BCF_BT_FLOAT as c_int => {
            let f = le_to_float((*info).vptr.add(index * size_of::<f32>()));
            if f.to_bits() == bcf_float_missing {
                bcf_int32_missing
            } else if f.to_bits() == bcf_float_vector_end {
                bcf_int32_vector_end
            } else {
                f as i32
            }
        }
        _ => bcf_int32_missing,
    }
}

pub unsafe fn vcfutils_c_315_get_rn_value(rn: *const bcf_info_t, index: usize) -> i32 {
    let val = if rn.is_null() {
        1
    } else {
        vcfutils_c_280_get_int32_info_value(rn, index)
    };
    if val >= 0 {
        val
    } else {
        0
    }
}

pub unsafe fn vcfutils_c_325_set_info_v1(info: *mut bcf_info_t) {
    match (*info).type_ {
        x if x == BCF_BT_INT8 as c_int => (*info).v1.i = le_to_i8((*info).vptr) as i64,
        x if x == BCF_BT_INT16 as c_int => (*info).v1.i = le_to_i16((*info).vptr) as i64,
        x if x == BCF_BT_INT32 as c_int => (*info).v1.i = le_to_i32((*info).vptr) as i64,
        x if x == BCF_BT_INT64 as c_int => (*info).v1.i = le_to_i64((*info).vptr),
        x if x == BCF_BT_FLOAT as c_int => (*info).v1.f = le_to_float((*info).vptr),
        _ => {}
    }
}

pub unsafe fn vcfutils_c_349_fixup_info_length_code(info: *mut bcf_info_t) -> c_int {
    const BCF_TYPE_SHIFT: [usize; 16] = [0, 0, 1, 2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buf = [0u8; 24];
    let mut ptr = buf.as_mut_ptr();

    let mut type_ = if (*info).key <= 0x7f && (*info).key >= -120 {
        BCF_BT_INT8 as c_int
    } else if (*info).key <= 0x7fff && (*info).key >= -32760 {
        BCF_BT_INT16 as c_int
    } else {
        BCF_BT_INT32 as c_int
    };
    *ptr = ((1 << 4) | type_) as u8;
    ptr = ptr.add(1);
    i32_to_le((*info).key, ptr);
    ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);

    type_ = if (*info).len <= 0x7f && (*info).len >= -120 {
        BCF_BT_INT8 as c_int
    } else if (*info).len <= 0x7fff && (*info).len >= -32760 {
        BCF_BT_INT16 as c_int
    } else {
        BCF_BT_INT32 as c_int
    };
    if (*info).len < 15 {
        *ptr = (((*info).len << 4) | (*info).type_) as u8;
        ptr = ptr.add(1);
    } else {
        *ptr = (0xf0 | (*info).type_) as u8;
        ptr = ptr.add(1);
        *ptr = ((1 << 4) | type_) as u8;
        ptr = ptr.add(1);
        i32_to_le((*info).len, ptr);
        ptr = ptr.add(1 << BCF_TYPE_SHIFT[type_ as usize]);
    }

    let new_len = ptr.offset_from(buf.as_ptr()) as isize;
    let old_len = (*info).vptr_off() as isize;
    if new_len == old_len {
        libc::memcpy(
            (*info).vptr.offset(-old_len).cast(),
            buf.as_ptr().cast(),
            new_len as usize,
        );
    } else if new_len < old_len {
        let adjust = old_len - new_len;
        libc::memcpy(
            (*info).vptr.offset(-old_len).cast(),
            buf.as_ptr().cast(),
            new_len as usize,
        );
        libc::memmove(
            (*info).vptr.offset(-adjust).cast(),
            (*info).vptr.cast(),
            (*info).vptr_len as usize,
        );
        (*info).vptr = (*info).vptr.offset(-adjust);
        (*info).set_vptr_off(((*info).vptr_off() as isize - adjust) as u32);
    } else {
        let new_info = libc::malloc((*info).vptr_len as usize + new_len as usize).cast::<u8>();
        if new_info.is_null() {
            return -1;
        }
        libc::memcpy(new_info.cast(), buf.as_ptr().cast(), new_len as usize);
        libc::memcpy(
            new_info.add(new_len as usize).cast(),
            (*info).vptr.cast(),
            (*info).vptr_len as usize,
        );
        if (*info).vptr_free() != 0 {
            libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
        }
        (*info).set_vptr_off(new_len as u32);
        (*info).vptr = new_info.add(new_len as usize);
        (*info).set_vptr_free(1);
    }
    0
}

pub unsafe fn vcfutils_c_407_mark_for_removal(info: *mut bcf_info_t) -> c_int {
    if (*info).vptr_free() != 0 {
        libc::free((*info).vptr.sub((*info).vptr_off() as usize).cast());
        (*info).set_vptr_free(0);
    }
    (*info).vptr = std::ptr::null_mut();
    (*info).set_vptr_off(0);
    (*info).vptr_len = 0;
    0
}

pub unsafe fn vcfutils_c_423_trim_int_cnv_tr_int_tags(
    info: *mut bcf_info_t,
    header: *const bcf_hdr_t,
    rm_set: *const hts_sys::kbitset_t,
    id: *const c_char,
    rn: *const bcf_info_t,
    ruc: *const bcf_info_t,
    num_alt_orig: usize,
    orig_total: usize,
) -> c_int {
    let count = if *id == b'C' as c_char {
        2usize
    } else {
        1usize
    };
    let key = (*info).key as usize;
    let type_ = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info
        [BCF_HL_INFO as usize]
        >> 4
        & 0xf) as c_int;
    let vlen = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info
        [BCF_HL_INFO as usize]
        >> 8
        & 0xf) as c_int;
    let element_sizes = [0usize, 1, 2, 4, 0, 4, 0, 0];
    let element_size = element_sizes[((*info).type_ & 0x7) as usize];
    let mut unit = 0usize;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;
    let mut new_total = 0i32;

    if (type_ != BCF_HT_INT as c_int && type_ != BCF_HT_REAL as c_int)
        || element_size == 0
        || vlen != BCF_VL_VAR as c_int
        || (*info).len != orig_total as c_int
        || ((*info).vptr_len as usize)
            < orig_total
                .saturating_mul(element_size)
                .saturating_mul(count)
    {
        return 1;
    }

    for allele in 0..num_alt_orig {
        let mut n_repeats = vcfutils_c_315_get_rn_value(rn, allele);
        let mut n_items = n_repeats;
        if !ruc.is_null() {
            n_items = 0;
            while n_repeats > 0 {
                let n_units = vcfutils_c_280_get_int32_info_value(ruc, unit);
                n_items += if n_units >= 0 { n_units } else { 0 };
                unit += 1;
                n_repeats -= 1;
            }
        }

        let byte_len = (n_items as usize) * element_size * count;
        if kbs_exists(rm_set.cast::<kbitset_t>(), (allele + 1) as c_int) != 0 {
            orig_pos += byte_len;
            continue;
        }
        if new_pos < orig_pos {
            libc::memmove(
                (*info).vptr.add(new_pos).cast(),
                (*info).vptr.add(orig_pos).cast(),
                byte_len,
            );
        }
        orig_pos += byte_len;
        new_pos += byte_len;
        new_total += n_items;
    }

    if new_total == 0 {
        return vcfutils_c_407_mark_for_removal(info);
    }

    (*info).vptr_len = new_pos as u32;
    (*info).len = new_total;
    if (*info).len == 1 {
        vcfutils_c_325_set_info_v1(info);
    }
    vcfutils_c_349_fixup_info_length_code(info)
}

pub unsafe fn vcfutils_c_498_trim_int_cnv_tr_str_tags(
    info: *mut bcf_info_t,
    header: *const bcf_hdr_t,
    rm_set: *const hts_sys::kbitset_t,
    rn: *const bcf_info_t,
    num_alt_orig: usize,
    _orig_total: usize,
) -> c_int {
    let key = (*info).key as usize;
    let type_ = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info
        [BCF_HL_INFO as usize]
        >> 4
        & 0xf) as c_int;
    let vlen = ((*(*(*header).id[BCF_DT_ID as usize].add(key)).val).info
        [BCF_HL_INFO as usize]
        >> 8
        & 0xf) as c_int;
    let mut orig_pos = 0usize;
    let mut new_pos = 0usize;

    if type_ != BCF_HT_STR as c_int
        || (*info).type_ != BCF_BT_CHAR as c_int
        || vlen != BCF_VL_VAR as c_int
    {
        return 1;
    }

    for allele in 0..num_alt_orig {
        let mut n_items = vcfutils_c_315_get_rn_value(rn, allele);
        let start = (*info).vptr.add(orig_pos);
        let mut end = start;
        let lim = (*info).vptr.add((*info).vptr_len as usize);

        while n_items > 0 {
            while end < lim && *end != 0 && *end != b',' {
                end = end.add(1);
            }
            if end == lim || *end == 0 {
                break;
            }
            end = end.add(1);
            n_items -= 1;
        }

        let span = end.offset_from(start) as usize;
        if kbs_exists(rm_set.cast::<kbitset_t>(), (allele + 1) as c_int) != 0 {
            orig_pos += span;
            continue;
        }
        if new_pos < orig_pos {
            libc::memmove(
                (*info).vptr.add(new_pos).cast(),
                (*info).vptr.add(orig_pos).cast(),
                span,
            );
        }
        orig_pos += span;
        new_pos += span;
    }

    if new_pos == 0 {
        return vcfutils_c_407_mark_for_removal(info);
    }
    if new_pos < orig_pos {
        *(*info).vptr.add(new_pos) = 0;
        if new_pos > 0 && *(*info).vptr.add(new_pos - 1) == b',' {
            new_pos -= 1;
            *(*info).vptr.add(new_pos) = 0;
        }
        (*info).len = new_pos as c_int;
        (*info).vptr_len = new_pos as u32;
        return vcfutils_c_349_fixup_info_length_code(info);
    }
    0
}

pub unsafe fn vcfutils_c_561_fixup_cnv_tr_info_tags(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    num_alt_orig: usize,
    rm_set: *const hts_sys::kbitset_t,
) -> c_int {
    let rn = bcf_get_info(header, line, c"RN".as_ptr());
    let ruc = bcf_get_info(header, line, c"RUC".as_ptr());
    let mut orig_total_repeats = 0i64;
    let mut orig_total_units = 0i64;
    let mut unit = 0usize;

    for allele in 0..num_alt_orig {
        let mut n_repeats = vcfutils_c_315_get_rn_value(rn, allele);
        orig_total_repeats += n_repeats as i64;
        if !ruc.is_null() {
            while n_repeats > 0 {
                let n_units = vcfutils_c_280_get_int32_info_value(ruc, unit);
                orig_total_units += if n_units >= 0 { n_units as i64 } else { 0 };
                unit += 1;
                n_repeats -= 1;
            }
        }
    }

    for i in 0..(*line).n_info() {
        let info = (*line).d.info.add(i as usize);
        let id = (*(*header).id[BCF_DT_ID as usize].add((*info).key as usize)).key;
        let orig_ptr = (*info).vptr.sub((*info).vptr_off() as usize);
        if *id != b'C' as c_char && *id != b'R' as c_char {
            continue;
        }

        if libc::strcmp(id, c"RB".as_ptr()) == 0
            || libc::strcmp(id, c"RUL".as_ptr()) == 0
            || libc::strcmp(id, c"CIRB".as_ptr()) == 0
            || libc::strcmp(id, c"CIRUC".as_ptr()) == 0
        {
            let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
                info,
                header,
                rm_set,
                id,
                rn,
                std::ptr::null(),
                num_alt_orig,
                orig_total_repeats as usize,
            );
            if res < 0 {
                return res;
            }
        } else if libc::strcmp(id, c"RUS".as_ptr()) == 0 {
            let res = vcfutils_c_498_trim_int_cnv_tr_str_tags(
                info,
                header,
                rm_set,
                rn,
                num_alt_orig,
                orig_total_repeats as usize,
            );
            if res < 0 {
                return res;
            }
        } else if !ruc.is_null() && libc::strcmp(id, c"RUB".as_ptr()) == 0 {
            let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
                info,
                header,
                rm_set,
                id,
                rn,
                ruc,
                num_alt_orig,
                orig_total_units as usize,
            );
            if res < 0 {
                return res;
            }
        }

        if (*info).vptr.is_null() || (*info).vptr.sub((*info).vptr_off() as usize) != orig_ptr {
            (*line).d.shared_dirty |= BCF1_DIRTY_INF as c_int;
        }
    }

    if !ruc.is_null() {
        let res = vcfutils_c_423_trim_int_cnv_tr_int_tags(
            ruc,
            header,
            rm_set,
            c"RUC".as_ptr(),
            rn,
            std::ptr::null(),
            num_alt_orig,
            orig_total_repeats as usize,
        );
        if res < 0 {
            return res;
        }
    }
    0
}

pub unsafe fn bcf_trim_alleles(header: *const bcf_hdr_t, line: *mut bcf1_t) -> c_int {
    vcfutils_c_186_bcf_trim_alleles(header, line)
}

pub unsafe fn vcfutils_c_186_bcf_trim_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
) -> c_int {
    let mut ret = 0;
    let mut nrm = 0;
    let gt = bcf_get_fmt(header, line, c"GT".as_ptr());
    if gt.is_null() {
        return 0;
    }

    let ac = libc::calloc((*line).n_allele() as usize, size_of::<c_int>()).cast::<c_int>();
    if ac.is_null() {
        return -1;
    }

    for i in 0..(*line).n_sample() as usize {
        let p = (*gt).p.add(i * (*gt).size as usize);
        for ial in 0..(*gt).n as usize {
            let val = match (*gt).type_ {
                x if x == BCF_BT_INT8 as c_int => {
                    let v = le_to_i8(p.add(ial)) as c_int;
                    if v == bcf_int8_vector_end {
                        break;
                    }
                    v
                }
                x if x == BCF_BT_INT16 as c_int => {
                    let v = le_to_i16(p.add(ial * size_of::<i16>())) as c_int;
                    if v == bcf_int16_vector_end {
                        break;
                    }
                    v
                }
                x if x == BCF_BT_INT32 as c_int => {
                    let v = le_to_i32(p.add(ial * size_of::<i32>()));
                    if v == bcf_int32_vector_end {
                        break;
                    }
                    v
                }
                _ => {
                    ret = -1;
                    break;
                }
            };
            if val >> 1 == 0 {
                continue;
            }
            let allele = (val >> 1) - 1;
            if allele >= (*line).n_allele() as c_int {
                ret = -1;
                break;
            }
            *ac.add(allele as usize) += 1;
        }
        if ret != 0 {
            break;
        }
    }
    if ret != 0 {
        libc::free(ac.cast());
        return ret;
    }

    let rm_set = kbs_init((*line).n_allele() as usize);
    if rm_set.is_null() {
        libc::free(ac.cast());
        return -1;
    }
    for i in 1..(*line).n_allele() as c_int {
        if *ac.add(i as usize) == 0 {
            kbs_insert(rm_set, i);
            nrm += 1;
        }
    }

    if nrm != 0 && bcf_remove_allele_set(header, line, rm_set.cast()) != 0 {
        ret = -2;
    }

    libc::free(ac.cast());
    kbs_destroy(rm_set);
    if ret != 0 {
        ret
    } else {
        nrm
    }
}

pub unsafe fn bcf_remove_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    mask: c_int,
) -> c_int {
    vcfutils_c_241_bcf_remove_alleles(header, line, mask)
}

pub unsafe fn vcfutils_c_241_bcf_remove_alleles(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    mask: c_int,
) -> c_int {
    let rm_set = kbs_init((*line).n_allele() as usize);
    if rm_set.is_null() {
        return -1;
    }
    for i in 1..(*line).n_allele() as c_int {
        if (mask & (1 << i)) != 0 {
            kbs_insert(rm_set, i);
        }
    }
    bcf_remove_allele_set(header, line, rm_set.cast());
    kbs_destroy(rm_set);
    0
}

pub unsafe fn bcf_remove_allele_set(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    rm_set: *const hts_sys::kbitset_t,
) -> c_int {
    super::vcfutils::vcfutils_c_659_bcf_remove_allele_set(header, line, rm_set.cast())
}

pub unsafe fn bcf_calc_ac(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    ac: *mut c_int,
    which: c_int,
) -> c_int {
    vcfutils_c_32_bcf_calc_ac(header, line, ac, which)
}

pub unsafe fn vcfutils_c_32_bcf_calc_ac(
    header: *const bcf_hdr_t,
    line: *mut bcf1_t,
    ac: *mut c_int,
    which: c_int,
) -> c_int {
    for i in 0..(*line).n_allele() as usize {
        *ac.add(i) = 0;
    }

    if (which & BCF_UN_INFO as c_int) != 0 {
        bcf_unpack(line, BCF_UN_INFO as c_int);
        let an_id = bcf_hdr_id2int(header, BCF_DT_ID as c_int, c"AN".as_ptr());
        let ac_id = bcf_hdr_id2int(header, BCF_DT_ID as c_int, c"AC".as_ptr());
        let mut an = -1;
        let mut ac_len = 0;
        let mut ac_type = 0;
        let mut ac_ptr: *mut u8 = std::ptr::null_mut();

        if an_id >= 0 && ac_id >= 0 {
            for i in 0..(*line).n_info() as usize {
                let z = (*line).d.info.add(i);
                if (*z).key == an_id {
                    an = (*z).v1.i as c_int;
                } else if (*z).key == ac_id {
                    ac_ptr = (*z).vptr;
                    ac_len = (*z).len;
                    ac_type = (*z).type_;
                }
            }
        }

        if an >= 0 && !ac_ptr.is_null() {
            if ac_len != (*line).n_allele() as c_int - 1 {
                return 0;
            }
            let mut nac = 0;
            for i in 0..ac_len as usize {
                let val = match ac_type {
                    x if x == BCF_BT_INT8 as c_int => le_to_i8(ac_ptr.add(i)) as c_int,
                    x if x == BCF_BT_INT16 as c_int => {
                        le_to_i16(ac_ptr.add(i * size_of::<i16>())) as c_int
                    }
                    x if x == BCF_BT_INT32 as c_int => {
                        le_to_i32(ac_ptr.add(i * size_of::<i32>()))
                    }
                    _ => libc::exit(1),
                };
                *ac.add(i + 1) = val;
                nac += val;
            }
            if an < nac {
                libc::exit(1);
            }
            *ac = an - nac;
            return 1;
        }
    }

    if (which & BCF_UN_FMT as c_int) != 0 {
        let gt_id = bcf_hdr_id2int(header, BCF_DT_ID as c_int, c"GT".as_ptr());
        if gt_id < 0 {
            return 0;
        }
        bcf_unpack(line, BCF_UN_FMT as c_int);

        let mut fmt_gt: *mut bcf_fmt_t = std::ptr::null_mut();
        for i in 0..(*line).n_fmt() as usize {
            let fmt = (*line).d.fmt.add(i);
            if (*fmt).id == gt_id {
                fmt_gt = fmt;
                break;
            }
        }
        if fmt_gt.is_null() {
            return 0;
        }

        for i in 0..(*line).n_sample() as usize {
            let p = (*fmt_gt).p.add(i * (*fmt_gt).size as usize);
            for ial in 0..(*fmt_gt).n as usize {
                let val = match (*fmt_gt).type_ {
                    x if x == BCF_BT_INT8 as c_int => {
                        let v = le_to_i8(p.add(ial)) as c_int;
                        if v == bcf_int8_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == BCF_BT_INT16 as c_int => {
                        let v = le_to_i16(p.add(ial * size_of::<i16>())) as c_int;
                        if v == bcf_int16_vector_end {
                            break;
                        }
                        v
                    }
                    x if x == BCF_BT_INT32 as c_int => {
                        let v = le_to_i32(p.add(ial * size_of::<i32>()));
                        if v == bcf_int32_vector_end {
                            break;
                        }
                        v
                    }
                    _ => libc::exit(1),
                };
                if val >> 1 == 0 {
                    continue;
                }
                if val >> 1 > (*line).n_allele() as c_int {
                    libc::exit(1);
                }
                *ac.add(((val >> 1) - 1) as usize) += 1;
            }
        }
        return 1;
    }

    0
}

pub unsafe fn bcf_gt_type(
    fmt_ptr: *mut bcf_fmt_t,
    isample: c_int,
    ial: *mut c_int,
    jal: *mut c_int,
) -> c_int {
    vcfutils_c_134_bcf_gt_type(fmt_ptr, isample, ial, jal)
}

pub unsafe fn vcfutils_c_134_bcf_gt_type(
    fmt_ptr: *mut bcf_fmt_t,
    isample: c_int,
    ial_out: *mut c_int,
    jal_out: *mut c_int,
) -> c_int {
    let mut nals = 0;
    let mut has_ref = 0;
    let mut has_alt = 0;
    let mut ial = 0;
    let mut jal = 0;
    let p = (*fmt_ptr)
        .p
        .add(isample as usize * (*fmt_ptr).size as usize);

    for i in 0..(*fmt_ptr).n as usize {
        let val = match (*fmt_ptr).type_ {
            x if x == BCF_BT_INT8 as c_int => {
                let v = le_to_i8(p.add(i)) as c_int;
                if v == bcf_int8_vector_end {
                    break;
                }
                v
            }
            x if x == BCF_BT_INT16 as c_int => {
                let v = le_to_i16(p.add(i * size_of::<i16>())) as c_int;
                if v == bcf_int16_vector_end {
                    break;
                }
                v
            }
            x if x == BCF_BT_INT32 as c_int => {
                let v = le_to_i32(p.add(i * size_of::<i32>()));
                if v == bcf_int32_vector_end {
                    break;
                }
                v
            }
            _ => libc::exit(1),
        };
        if val >> 1 == 0 {
            return hts_sys::GT_UNKN as c_int;
        }
        let tmp = val >> 1;
        if tmp > 1 {
            if ial == 0 {
                ial = tmp;
                has_alt = 1;
            } else if tmp != ial {
                if tmp < ial {
                    jal = ial;
                    ial = tmp;
                } else {
                    jal = tmp;
                }
                has_alt = 2;
            }
        } else {
            has_ref = 1;
        }
        nals += 1;
    }

    if !ial_out.is_null() {
        *ial_out = if ial > 0 { ial - 1 } else { ial };
    }
    if !jal_out.is_null() {
        *jal_out = if jal > 0 { jal - 1 } else { jal };
    }
    if nals == 0 {
        return hts_sys::GT_UNKN as c_int;
    }
    if nals == 1 {
        return if has_ref != 0 {
            hts_sys::GT_HAPL_R
        } else {
            hts_sys::GT_HAPL_A
        } as c_int;
    }
    if has_ref == 0 {
        return if has_alt == 1 {
            hts_sys::GT_HOM_AA
        } else {
            hts_sys::GT_HET_AA
        } as c_int;
    }
    if has_alt == 0 {
        return hts_sys::GT_HOM_RR as c_int;
    }
    hts_sys::GT_HET_RA as c_int
}

pub unsafe fn bcf_sr_init() -> *mut bcf_srs_t {
    hts_sys::bcf_sr_init()
}

unsafe fn bcf_sr_destroy1(reader: *mut bcf_sr_t, closefile: c_int) {
    unsafe {
        if reader.is_null() {
            return;
        }

        if !(*reader).file.is_null() && closefile != 0 {
            let _ = hts_sys::hts_close((*reader).file);
        }
        libc::free((*reader).fname.cast());
        if !(*reader).tbx_idx.is_null() {
            super::tbx::tbx_destroy((*reader).tbx_idx);
        }
        if !(*reader).bcf_idx.is_null() {
            hts_sys::hts_idx_destroy((*reader).bcf_idx);
        }
        bcf_hdr_destroy((*reader).header);
        if !(*reader).itr.is_null() {
            hts_sys::hts_itr_destroy((*reader).itr);
        }
        for j in 0..(*reader).mbuffer {
            bcf_destroy(*(*reader).buffer.add(j as usize));
        }
        libc::free((*reader).buffer.cast());
        libc::free((*reader).samples.cast());
        libc::free((*reader).filter_ids.cast());
    }
}

pub unsafe fn synced_bcf_reader_c_461_bcf_sr_destroy1(reader: *mut bcf_sr_t, closefile: c_int) {
    unsafe { bcf_sr_destroy1(reader, closefile) }
}

pub unsafe fn bcf_sr_destroy(readers: *mut bcf_srs_t) {
    hts_sys::bcf_sr_destroy(readers)
}

const BCF_SR_ERROR_NOIDX_ERROR: c_int = 10;

pub unsafe fn bcf_sr_strerror(errnum: c_int) -> *mut c_char {
    match errnum {
        x if x == hts_sys::bcf_sr_error_open_failed as c_int => unsafe {
            libc::strerror(*libc::__errno_location())
        },
        x if x == hts_sys::bcf_sr_error_not_bgzf as c_int => {
            c"not compressed with bgzip".as_ptr().cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_idx_load_failed as c_int => {
            c"could not load index".as_ptr().cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_file_type_error as c_int => {
            c"unknown file type".as_ptr().cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_api_usage_error as c_int => {
            c"API usage error".as_ptr().cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_header_error as c_int => {
            c"could not parse header".as_ptr().cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_no_eof as c_int => {
            c"no BGZF EOF marker; file may be truncated"
                .as_ptr()
                .cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_no_memory as c_int => c"Out of memory".as_ptr().cast_mut(),
        x if x == hts_sys::bcf_sr_error_vcf_parse_error as c_int => {
            c"VCF parse error".as_ptr().cast_mut()
        }
        x if x == hts_sys::bcf_sr_error_bcf_read_error as c_int => {
            c"BCF read error".as_ptr().cast_mut()
        }
        BCF_SR_ERROR_NOIDX_ERROR => c"merge of unindexed files failed".as_ptr().cast_mut(),
        _ => c"".as_ptr().cast_mut(),
    }
}

pub unsafe fn bcf_sr_set_threads(files: *mut bcf_srs_t, n_threads: c_int) -> c_int {
    hts_sys::bcf_sr_set_threads(files, n_threads)
}

pub unsafe fn bcf_sr_set_opt_require_idx(readers: *mut bcf_srs_t) -> c_int {
    unsafe {
        (*readers).require_index = REQUIRE_IDX_;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_allow_no_idx(readers: *mut bcf_srs_t) -> c_int {
    unsafe {
        (*readers).require_index = ALLOW_NO_IDX_;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_pair_logic(readers: *mut bcf_srs_t, pair_logic: c_int) -> c_int {
    unsafe {
        (*bcf_sr_aux_mut(readers)).sort.pair = pair_logic;
    }
    0
}

pub unsafe fn bcf_sr_set_opt_regions_overlap(readers: *mut bcf_srs_t, overlap: c_int) -> c_int {
    unsafe {
        (*bcf_sr_aux_mut(readers)).regions_overlap = overlap;
        if !(*readers).regions.is_null() {
            bcf_sr_regions_set_overlap((*readers).regions, overlap);
        }
    }
    0
}

pub unsafe fn bcf_sr_set_opt_targets_overlap(readers: *mut bcf_srs_t, overlap: c_int) -> c_int {
    unsafe {
        (*bcf_sr_aux_mut(readers)).targets_overlap = overlap;
        if !(*readers).targets.is_null() {
            bcf_sr_regions_set_overlap((*readers).targets, overlap);
        }
    }
    0
}

pub unsafe fn bcf_sr_set_opt(
    readers: *mut bcf_srs_t,
    opt: hts_sys::bcf_sr_opt_t,
    value: c_int,
) -> c_int {
    match opt {
        BCF_SR_REQUIRE_IDX => unsafe { bcf_sr_set_opt_require_idx(readers) },
        BCF_SR_ALLOW_NO_IDX => unsafe { bcf_sr_set_opt_allow_no_idx(readers) },
        BCF_SR_PAIR_LOGIC => unsafe { bcf_sr_set_opt_pair_logic(readers, value) },
        BCF_SR_REGIONS_OVERLAP => unsafe { bcf_sr_set_opt_regions_overlap(readers, value) },
        BCF_SR_TARGETS_OVERLAP => unsafe { bcf_sr_set_opt_targets_overlap(readers, value) },
        _ => 1,
    }
}

pub unsafe fn bcf_sr_destroy_threads(files: *mut bcf_srs_t) {
    hts_sys::bcf_sr_destroy_threads(files)
}

pub unsafe fn bcf_sr_add_reader(readers: *mut bcf_srs_t, fname: *const c_char) -> c_int {
    hts_sys::bcf_sr_add_reader(readers, fname)
}

pub unsafe fn bcf_sr_remove_reader(files: *mut bcf_srs_t, i: c_int) {
    hts_sys::bcf_sr_remove_reader(files, i)
}

pub unsafe fn bcf_sr_next_line(readers: *mut bcf_srs_t) -> c_int {
    hts_sys::bcf_sr_next_line(readers)
}

pub unsafe fn bcf_sr_has_line(readers: *mut bcf_srs_t, i: c_int) -> c_int {
    if readers.is_null() || i < 0 || i >= (*readers).nreaders || (*readers).has_line.is_null() {
        return 0;
    }
    *(*readers).has_line.add(i as usize)
}

pub unsafe fn bcf_sr_get_line(readers: *mut bcf_srs_t, i: c_int) -> *mut bcf1_t {
    if bcf_sr_has_line(readers, i) == 0 || (*readers).readers.is_null() {
        return std::ptr::null_mut();
    }
    let reader = (*readers).readers.add(i as usize);
    if (*reader).buffer.is_null() {
        return std::ptr::null_mut();
    }
    *(*reader).buffer
}

pub unsafe fn bcf_sr_get_header(readers: *mut bcf_srs_t, i: c_int) -> *mut bcf_hdr_t {
    if readers.is_null() || i < 0 || i >= (*readers).nreaders || (*readers).readers.is_null() {
        return std::ptr::null_mut();
    }
    (*(*readers).readers.add(i as usize)).header
}

pub unsafe fn bcf_sr_seek(readers: *mut bcf_srs_t, seq: *const c_char, pos: hts_pos_t) -> c_int {
    unsafe {
        if !readers.is_null() && !(*readers).regions.is_null() && seq.is_null() && pos == 0 {
            (*bcf_sr_aux_mut(readers)).sort.chr = std::ptr::null();
            bcf_sr_seek_start(readers);
            return 0;
        }
    }
    hts_sys::bcf_sr_seek(readers, seq, pos)
}

pub unsafe fn bcf_sr_set_samples(
    readers: *mut bcf_srs_t,
    samples: *const c_char,
    is_file: c_int,
) -> c_int {
    hts_sys::bcf_sr_set_samples(readers, samples, is_file)
}

pub unsafe fn bcf_sr_set_targets(
    readers: *mut bcf_srs_t,
    targets: *const c_char,
    is_file: c_int,
    alleles: c_int,
) -> c_int {
    hts_sys::bcf_sr_set_targets(readers, targets, is_file, alleles)
}

pub unsafe fn bcf_sr_set_regions(
    readers: *mut bcf_srs_t,
    regions: *const c_char,
    is_file: c_int,
) -> c_int {
    hts_sys::bcf_sr_set_regions(readers, regions, is_file)
}

pub unsafe fn bcf_sr_regions_init(
    regions: *const c_char,
    is_file: c_int,
    chr: c_int,
    from: c_int,
    to: c_int,
) -> *mut bcf_sr_regions_t {
    if is_file == 0 {
        let reg = unsafe { regions_init_string(regions) };
        unsafe { regions_sort_and_merge(reg) };
        return reg;
    }
    hts_sys::bcf_sr_regions_init(regions, is_file, chr, from, to)
}

pub unsafe fn bcf_sr_regions_destroy(regions: *mut bcf_sr_regions_t) {
    unsafe { bcf_sr_regions_destroy_translated(regions) }
}

pub unsafe fn bcf_sr_regions_seek(regions: *mut bcf_sr_regions_t, chr: *const c_char) -> c_int {
    unsafe {
        if regions.is_null() || chr.is_null() {
            return -1;
        }

        if !(*regions).regs.is_null() {
            (*regions).iseq = -1;
            (*regions).start = -1;
            (*regions).end = -1;

            let mut iseq = -1;
            if super::sam::khash_str2int_get((*regions).seq_hash, chr, &mut iseq) < 0 {
                return -1;
            }
            (*regions).iseq = iseq;
            (*(*regions).regs.cast::<BcfSrRegion>().add(iseq as usize)).creg = -1;
            return 0;
        }
    }
    hts_sys::bcf_sr_regions_seek(regions, chr)
}

pub unsafe fn bcf_sr_regions_next(reg: *mut bcf_sr_regions_t) -> c_int {
    unsafe {
        if reg.is_null() || (*reg).iseq < 0 {
            return -1;
        }

        if !(*reg).regs.is_null() {
            (*reg).start = -1;
            (*reg).end = -1;
            (*reg).nals = 0;

            let regs = (*reg).regs.cast::<BcfSrRegion>();
            while (*reg).iseq < (*reg).nseqs {
                if advance_creg(regs.add((*reg).iseq as usize)) == 0 {
                    break;
                }
                (*reg).iseq += 1;
            }
            if (*reg).iseq >= (*reg).nseqs {
                (*reg).iseq = -1;
                return -1;
            }

            let seq_reg = regs.add((*reg).iseq as usize);
            let creg = (*seq_reg).regs.add((*seq_reg).creg as usize);
            (*reg).start = (*creg).start;
            (*reg).end = (*creg).end;
            return 0;
        }
    }
    hts_sys::bcf_sr_regions_next(reg)
}

unsafe fn bcf_sr_regions_overlap_inner(
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

pub unsafe fn bcf_sr_regions_overlap(
    reg: *mut bcf_sr_regions_t,
    seq: *const c_char,
    start: hts_pos_t,
    end: hts_pos_t,
) -> c_int {
    unsafe {
        if reg.is_null() || seq.is_null() {
            return -1;
        }
        if !(*reg).regs.is_null() {
            return bcf_sr_regions_overlap_inner(reg, seq, start, end, 1);
        }
    }
    hts_sys::bcf_sr_regions_overlap(reg, seq, start, end)
}

pub unsafe fn bcf_sr_regions_flush(regs: *mut bcf_sr_regions_t) -> c_int {
    unsafe {
        if regs.is_null() {
            return -1;
        }
        if !(*regs).regs.is_null() {
            let Some(handler) = (*regs).missed_reg_handler else {
                return 0;
            };
            if (*regs).prev_seq == -1 {
                return 0;
            }
            while bcf_sr_regions_next(regs) == 0 {
                handler(regs, (*regs).missed_reg_data);
            }
            return 0;
        }
    }
    hts_sys::bcf_sr_regions_flush(regs)
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
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("htslib/test/tabix/vcf_file.bcf");
        assert!(path.exists(), "fixture missing: {}", path.display());
        let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();

        let (native_fwd, native_bwd) = sweep_positions_native(&c_path);
        let (lib_fwd, lib_bwd) = sweep_positions_libhts(&c_path);

        assert_eq!(native_fwd, lib_fwd, "forward sweep positions differ");
        assert_eq!(native_bwd, lib_bwd, "backward sweep positions differ");
        // Backward sweep must reverse the forward order.
        let mut rev = native_fwd.clone();
        rev.reverse();
        assert_eq!(native_bwd, rev, "backward sweep is not the reverse of forward");
        assert!(!native_fwd.is_empty(), "expected at least one record");
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
            let explicit: &[&std::ffi::CStr] = &[c"bcf", c"vcf", c"vcf.gz", c"vcf.bgz", c"BCF", c"junk"];
            for fmt in explicit {
                let mut native = [0 as c_char; 16];
                let mut libv = [0 as c_char; 16];
                let nr = vcf_open_mode(native.as_mut_ptr(), c"f".as_ptr(), fmt.as_ptr());
                let lr = libhts_sweep::vcf_open_mode(libv.as_mut_ptr(), c"f".as_ptr(), fmt.as_ptr());
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
                0i32, 1, 2, 4, 8, 16, 32, 64, 1 | 2, 4 | 16 | 64, 1 | 2 | 4 | 8 | 16 | 32 | 64,
                128, // undescribed -> "Unknown error"
                1 | 128,
            ];
            for &code in &codes {
                for &cap in &[4usize, 8, 16, 64, 256] {
                    let mut native = vec![0xAAu8 as c_char; cap];
                    let mut libv = vec![0xAAu8 as c_char; cap];
                    let nr = bcf_strerror(code, native.as_mut_ptr(), cap);
                    let lr = libhts_sweep::bcf_strerror(code, libv.as_mut_ptr(), cap);
                    assert_eq!(nr.is_null(), lr.is_null(), "null-ness differs code={code} cap={cap}");
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
            let dup = bcf_hdr_dup(hdr);
            assert!(!dup.is_null());
            hts_sys::bcf_hdr_destroy(dup);
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
                        hts_sys::bcf_has_filter(hdr, rec, p),
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
            assert!(bcf_hdr_idinfo_exists_rs(
                hdr,
                BCF_HL_INFO as c_int,
                id
            ));
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
                vcf_c_796_bcf_hdr_set_idx(
                    hdr,
                    BCF_DT_ID as c_int,
                    c"DP".as_ptr(),
                    &mut first,
                ),
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
                vcf_c_796_bcf_hdr_set_idx(
                    hdr,
                    BCF_DT_ID as c_int,
                    c"AF".as_ptr(),
                    &mut explicit,
                ),
                0
            );
            assert_eq!((*hdr).n[BCF_DT_ID as usize], explicit.id + 1);
            assert!(
                (*(*hdr).id[BCF_DT_ID as usize].add((start_n + 1) as usize))
                    .key
                    .is_null()
            );
            assert!(
                (*(*hdr).id[BCF_DT_ID as usize].add((start_n + 2) as usize))
                    .key
                    .is_null()
            );
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
                vcf_c_796_bcf_hdr_set_idx(
                    hdr,
                    BCF_DT_ID as c_int,
                    c"MQ".as_ptr(),
                    &mut conflict,
                ),
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
            assert_eq!(
                bcf_hdr_id2int(hdr, BCF_DT_ID as c_int, c"DP".as_ptr()),
                id
            );

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
                vcfutils_c_32_bcf_calc_ac(
                    hdr,
                    rec,
                    gt_ac.as_mut_ptr(),
                    BCF_UN_FMT as c_int
                ),
                1
            );
            assert_eq!(gt_ac, [1, 1, 2]);

            let fmt = bcf_get_fmt(hdr, rec, c"GT".as_ptr());
            assert!(!fmt.is_null());
            let mut ial = -1;
            let mut jal = -1;
            assert_eq!(
                vcfutils_c_134_bcf_gt_type(fmt, 0, &mut ial, &mut jal),
                hts_sys::GT_HET_RA as c_int
            );
            assert_eq!(ial, 1);
            assert_eq!(jal, 0);
            assert_eq!(
                vcfutils_c_134_bcf_gt_type(fmt, 1, &mut ial, &mut jal),
                hts_sys::GT_HOM_AA as c_int
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
                    x if x == BCF_BT_INT32 as c_int => {
                        le_to_i32(p.add(idx * size_of::<i32>()))
                    }
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
            assert!(
                (*info).type_ != BCF_BT_INT64 as c_int
                    || (*info).v1.i != BCF_MIN_BT_INT64 - 1
            );

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

            assert_eq!(
                bcf_enc_size(&mut str_, 14, BCF_BT_INT8 as c_int),
                0
            );
            assert_eq!(str_.l, 1);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((14 << 4) | BCF_BT_INT8 as c_int) as u8
            );

            str_.l = 0;
            assert_eq!(
                bcf_enc_size(&mut str_, 15, BCF_BT_INT16 as c_int),
                0
            );
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
            assert_eq!(
                bcf_enc_size(&mut str_, 32768, BCF_BT_INT32 as c_int),
                0
            );
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
            assert_eq!(
                le_to_i8(str_.s.add(1).cast()) as i32,
                bcf_int8_missing
            );

            str_.l = 0;
            assert_eq!(bcf_enc_int1(&mut str_, bcf_int32_vector_end), 0);
            assert_eq!(str_.l, 2);
            assert_eq!(
                *str_.s.cast::<u8>(),
                ((1 << 4) | BCF_BT_INT8 as c_int) as u8
            );
            assert_eq!(
                le_to_i8(str_.s.add(1).cast()) as i32,
                bcf_int8_vector_end
            );

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
            i16_to_le(
                bcf_int16_vector_end as i16,
                int16_values.as_mut_ptr(),
            );
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
            assert_eq!(
                (*(*rec).d.var.add(2)).type_,
                (VCF_INDEL | VCF_INS) as c_int
            );
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
            assert_eq!(
                bcf_has_variant_types(rec, VCF_INDEL, 0),
                VCF_INDEL as c_int
            );
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

            assert_eq!(
                bcf_has_variant_types(rec, VCF_INDEL, 0),
                VCF_INDEL as c_int
            );
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
            assert_eq!(
                bcf_has_variant_type(rec, 2, VCF_SNP),
                VCF_SNP as c_int
            );
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
            assert_eq!(
                bcf_translate_id_size(src_info_id),
                BCF_BT_INT8 as c_int
            );
            assert_eq!(
                bcf_translate_id_size(src_fmt_id),
                BCF_BT_INT8 as c_int
            );
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
                CStr::from_ptr(bcf_sr_strerror(hts_sys::bcf_sr_error_not_bgzf as c_int)).to_bytes(),
                b"not compressed with bgzip"
            );
            assert_eq!(
                CStr::from_ptr(bcf_sr_strerror(
                    hts_sys::bcf_sr_error_idx_load_failed as c_int
                ))
                .to_bytes(),
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
                CStr::from_ptr(bcf_sr_strerror(hts_sys::bcf_sr_error_open_failed as c_int))
                    .to_bytes(),
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
            let fp = hts_sys::hts_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());

            let mut reader: bcf_sr_t = std::mem::zeroed();
            reader.file = fp;
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
            assert_eq!(hts_sys::hts_close(fp), 0);
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
            (*readers).errnum = hts_sys::bcf_sr_error_open_failed;

            *errno = 0;
            assert_eq!(
                bcf_sr_add_hreader(readers, std::ptr::null_mut(), 0, std::ptr::null()),
                0
            );
            assert_eq!(*errno, libc::EINVAL);
            assert_eq!((*readers).errnum, hts_sys::bcf_sr_error_api_usage_error);

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
            bcf_hdr_append(
                hdr,
                c"##FILTER=<ID=q10,Description=\"q10\">".as_ptr()
            ),
            0
        );
        assert_eq!(
            bcf_hdr_append(
                hdr,
                c"##FILTER=<ID=s50,Description=\"s50\">".as_ptr()
            ),
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
        unsafe fn bytes<'x>(s: *const c_char, l: u64) -> &'x [u8] {
            if l == 0 || s.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(s.cast::<u8>(), l as usize)
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
                bcf_update_info(hdr, a, c"DP".as_ptr(), dp.as_ptr().cast(), 1, BCF_HT_INT as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr,
                    b,
                    c"DP".as_ptr(),
                    dp.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );

            let af = [0.25f32, 0.75f32];
            assert_eq!(
                bcf_update_info(hdr, a, c"AF".as_ptr(), af.as_ptr().cast(), 2, BCF_HT_REAL as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr,
                    b,
                    c"AF".as_ptr(),
                    af.as_ptr().cast(),
                    2,
                    BCF_HT_REAL as c_int
                ),
                0
            );

            let st = c"hello";
            assert_eq!(
                bcf_update_info(hdr, a, c"ST".as_ptr(), st.as_ptr().cast(), 5, BCF_HT_STR as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr,
                    b,
                    c"ST".as_ptr(),
                    st.as_ptr().cast(),
                    5,
                    BCF_HT_STR as c_int
                ),
                0
            );

            assert_eq!(
                bcf_update_info(hdr, a, c"FL".as_ptr(), std::ptr::null(), 1, BCF_HT_FLAG as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr,
                    b,
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
                bcf_update_info(hdr, a, c"DP".as_ptr(), dp2.as_ptr().cast(), 1, BCF_HT_INT as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr,
                    b,
                    c"DP".as_ptr(),
                    dp2.as_ptr().cast(),
                    1,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                bcf_update_info(hdr, a, c"ST".as_ptr(), std::ptr::null(), 0, BCF_HT_STR as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_info(
                    hdr,
                    b,
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
                bcf_update_format(hdr, a, c"GQ".as_ptr(), gq.as_ptr().cast(), 2, BCF_HT_INT as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr,
                    b,
                    c"GQ".as_ptr(),
                    gq.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );

            let ds = [0.1f32, 1.9f32];
            assert_eq!(
                bcf_update_format(hdr, a, c"DS".as_ptr(), ds.as_ptr().cast(), 2, BCF_HT_REAL as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr,
                    b,
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
                    hdr,
                    b,
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
                bcf_update_format(hdr, a, c"GQ".as_ptr(), gq2.as_ptr().cast(), 2, BCF_HT_INT as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr,
                    b,
                    c"GQ".as_ptr(),
                    gq2.as_ptr().cast(),
                    2,
                    BCF_HT_INT as c_int
                ),
                0
            );
            assert_eq!(
                bcf_update_format(hdr, a, c"DS".as_ptr(), std::ptr::null(), 0, BCF_HT_REAL as c_int),
                0
            );
            assert_eq!(
                hts_sys::bcf_update_format(
                    hdr,
                    b,
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

            assert_eq!(bcf_add_filter(hdr, a, q10), hts_sys::bcf_add_filter(hdr, b, q10));
            assert_eq!(bcf_add_filter(hdr, a, s50), hts_sys::bcf_add_filter(hdr, b, s50));
            // adding an already-present filter
            assert_eq!(bcf_add_filter(hdr, a, q10), hts_sys::bcf_add_filter(hdr, b, q10));
            assert_eq!(
                bcf_remove_filter(hdr, a, q10, 0),
                hts_sys::bcf_remove_filter(hdr, b, q10, 0)
            );
            // remove last, request PASS
            assert_eq!(
                bcf_remove_filter(hdr, a, s50, 1),
                hts_sys::bcf_remove_filter(hdr, b, s50, 1)
            );

            assert_eq!(bcf_update_id(hdr, a, c"rs123".as_ptr()), 0);
            assert_eq!(hts_sys::bcf_update_id(hdr, b, c"rs123".as_ptr()), 0);

            let flt = [q10, s50];
            assert_eq!(
                bcf_update_filter(hdr, a, flt.as_ptr().cast_mut(), 2),
                hts_sys::bcf_update_filter(hdr, b, flt.as_ptr().cast_mut(), 2)
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
                bcf_update_info(hdr, src, c"DP".as_ptr(), dp.as_ptr().cast(), 1, BCF_HT_INT as c_int),
                0
            );
            let gq = [5i32, 6i32];
            assert_eq!(
                bcf_update_format(hdr, src, c"GQ".as_ptr(), gq.as_ptr().cast(), 2, BCF_HT_INT as c_int),
                0
            );

            let native = bcf_dup(src);
            let csys = hts_sys::bcf_dup(src);
            assert_records_equal(native, csys);

            // bcf_copy into an existing record
            let dst = bcf_init();
            bcf_copy(dst, src);
            assert_records_equal(dst, csys);

            bcf_destroy(native);
            bcf_destroy(csys);
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
                bcf_update_info(hdr, src, c"DP".as_ptr(), dp.as_ptr().cast(), 1, BCF_HT_INT as c_int),
                0
            );
            let af = [0.25f32, 0.75f32];
            assert_eq!(
                bcf_update_info(hdr, src, c"AF".as_ptr(), af.as_ptr().cast(), 2, BCF_HT_REAL as c_int),
                0
            );
            assert_eq!(
                bcf_update_info(hdr, src, c"FL".as_ptr(), std::ptr::null(), 1, BCF_HT_FLAG as c_int),
                0
            );
            let gq = [11i32, 22i32];
            assert_eq!(
                bcf_update_format(hdr, src, c"GQ".as_ptr(), gq.as_ptr().cast(), 2, BCF_HT_INT as c_int),
                0
            );
            vcf_c_2332_bcf1_sync(src);

            // Two records carrying identical wire bytes but unpacked = 0.
            let a = bcf_dup(src);
            let b = bcf_dup(src);
            (*a).unpacked = 0;
            (*b).unpacked = 0;

            assert_eq!(bcf_unpack(a, BCF_UN_ALL as c_int), 0);
            assert_eq!(hts_sys::bcf_unpack(b, BCF_UN_ALL as c_int), 0);

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
            let names_b = hts_sys::bcf_hdr_seqnames(hdr, &mut nb);
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
                CStr::from_ptr(hts_sys::bcf_hdr_get_version(hdr)),
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
                    bcf_update_format(hdr, r, c"GQ".as_ptr(), gq.as_ptr().cast(), 2, BCF_HT_INT as c_int),
                    0
                );
                let ds = [1.5f32, 2.5f32];
                assert_eq!(
                    bcf_update_format(hdr, r, c"DS".as_ptr(), ds.as_ptr().cast(), 2, BCF_HT_REAL as c_int),
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
                hts_sys::bcf_subset(hdr, b, 1, imap.as_mut_ptr()),
                0
            );
            assert_records_equal(a, b);

            bcf_destroy(a);
            bcf_destroy(b);
            bcf_hdr_destroy(hdr);
        }
    }
}
