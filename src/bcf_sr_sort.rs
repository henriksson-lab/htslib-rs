// Functions translated from htslib/bcf_sr_sort.c.
// Extracted from src/vcf.rs.

use std::ffi::{c_char, c_int};

use crate::htslib_rs::vcf::*;
use crate::htslib_rs::hts::{hts_pos_t, kbs_destroy};

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
        crate::sam::khash_str2int_destroy_free((*srt).var_str2int);
        crate::sam::khash_str2int_destroy_free((*srt).grp_str2int);

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
