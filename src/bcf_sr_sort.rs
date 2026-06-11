// Functions translated from htslib/bcf_sr_sort.c.
// Extracted from src/vcf.rs.

use crate::htslib_rs::hts::{hts_pos_t, kbs_destroy};
use crate::htslib_rs::vcf::*;
use std::mem::zeroed;
use std::ptr::NonNull;

unsafe fn readers_slice_mut(readers: &mut bcf_srs_t) -> Option<&mut [bcf_sr_t]> {
    if readers.nreaders < 0 || readers.readers.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts_mut(readers.readers, readers.nreaders as usize) })
}

unsafe fn reader_and_has_line_slices_mut(
    readers: &mut bcf_srs_t,
) -> Option<(&mut [bcf_sr_t], &mut [i32])> {
    if readers.nreaders < 0 || readers.readers.is_null() || readers.has_line.is_null() {
        return None;
    }

    let len = readers.nreaders as usize;
    Some(unsafe {
        (
            std::slice::from_raw_parts_mut(readers.readers, len),
            std::slice::from_raw_parts_mut(readers.has_line, len),
        )
    })
}

unsafe fn active_slice(srt: &BcfSrSort) -> Option<&[i32]> {
    if srt.nactive < 0 || srt.active.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(srt.active, srt.nactive as usize) })
}

unsafe fn active_slice_mut(srt: &mut BcfSrSort) -> Option<&mut [i32]> {
    if srt.mactive < 0 || srt.active.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts_mut(srt.active, srt.mactive as usize) })
}

unsafe fn vcf_buf_slice_mut(srt: &mut BcfSrSort) -> Option<&mut [BcfSrSortVcfBuf]> {
    if srt.nsr < 0 || srt.vcf_buf.is_null() {
        return None;
    }
    Some(unsafe {
        std::slice::from_raw_parts_mut(srt.vcf_buf.cast::<BcfSrSortVcfBuf>(), srt.nsr as usize)
    })
}

unsafe fn rec_slice_mut(buf: &mut BcfSrSortVcfBuf) -> Option<&mut [*mut bcf1_t]> {
    if buf.nrec < 0 || buf.rec.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts_mut(buf.rec, buf.nrec as usize) })
}

unsafe fn rec_storage_slice_mut(buf: &mut BcfSrSortVcfBuf) -> Option<&mut [*mut bcf1_t]> {
    if buf.mrec < 0 || buf.rec.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts_mut(buf.rec, buf.mrec as usize) })
}

unsafe fn take_active_vec(srt: &mut BcfSrSort) -> Vec<i32> {
    unsafe {
        let Some(active) = NonNull::new(srt.active) else {
            return Vec::new();
        };
        let len = srt.mactive.max(0) as usize;
        let active = Vec::from_raw_parts(active.as_ptr(), len, len);
        srt.active = std::ptr::null_mut();
        srt.mactive = 0;
        active
    }
}

fn store_active_vec(srt: &mut BcfSrSort, active: Vec<i32>) {
    if active.is_empty() {
        srt.active = std::ptr::null_mut();
        srt.mactive = 0;
        return;
    }

    let mut active = active.into_boxed_slice();
    srt.active = active.as_mut_ptr();
    srt.mactive = active.len() as i32;
    std::mem::forget(active);
}

unsafe fn take_rec_vec(buf: &mut BcfSrSortVcfBuf) -> Vec<*mut bcf1_t> {
    unsafe {
        let Some(rec) = NonNull::new(buf.rec) else {
            return Vec::new();
        };
        let len = buf.mrec.max(0) as usize;
        let rec = Vec::from_raw_parts(rec.as_ptr(), len, len);
        buf.rec = std::ptr::null_mut();
        buf.mrec = 0;
        rec
    }
}

fn store_rec_vec(buf: &mut BcfSrSortVcfBuf, rec: Vec<*mut bcf1_t>) {
    if rec.is_empty() {
        buf.rec = std::ptr::null_mut();
        buf.mrec = 0;
        return;
    }

    let mut rec = rec.into_boxed_slice();
    buf.rec = rec.as_mut_ptr();
    buf.mrec = rec.len() as i32;
    std::mem::forget(rec);
}

unsafe fn take_vcf_buf_vec(srt: &mut BcfSrSort, len: i32) -> Vec<BcfSrSortVcfBuf> {
    unsafe {
        let Some(vcf_buf) = NonNull::new(srt.vcf_buf.cast::<BcfSrSortVcfBuf>()) else {
            return Vec::new();
        };
        if len <= 0 {
            return Vec::new();
        }
        let len = len as usize;
        let vcf_buf = Vec::from_raw_parts(vcf_buf.as_ptr(), len, len);
        srt.vcf_buf = std::ptr::null_mut();
        srt.msr = 0;
        vcf_buf
    }
}

fn store_vcf_buf_vec(srt: &mut BcfSrSort, vcf_buf: Vec<BcfSrSortVcfBuf>) {
    if vcf_buf.is_empty() {
        srt.vcf_buf = std::ptr::null_mut();
        srt.msr = 0;
        return;
    }

    let mut vcf_buf = vcf_buf.into_boxed_slice();
    srt.vcf_buf = vcf_buf.as_mut_ptr().cast();
    srt.msr = vcf_buf.len() as i32;
    std::mem::forget(vcf_buf);
}

unsafe fn bcf_sr_sort_reserve_active(srt: &mut BcfSrSort, need: i32) -> i32 {
    unsafe {
        if need < 0 {
            return -1;
        }
        if need <= srt.mactive {
            return 0;
        }

        let mut active = take_active_vec(srt);
        active.resize(need as usize, 0);
        store_active_vec(srt, active);
        0
    }
}

unsafe fn bcf_sr_sort_reserve_row(buf: &mut BcfSrSortVcfBuf, need: i32) -> i32 {
    unsafe {
        if need < 0 {
            return -1;
        }
        if need <= buf.mrec {
            return 0;
        }

        let mut rec = take_rec_vec(buf);
        rec.resize(need as usize, std::ptr::null_mut());
        store_rec_vec(buf, rec);
        0
    }
}

unsafe fn bcf_sr_sort_reserve_vcf_buf(readers: &mut bcf_srs_t, srt: &mut BcfSrSort) -> i32 {
    unsafe {
        if readers.nreaders < 0 {
            return -1;
        }
        if srt.nsr == readers.nreaders {
            return 0;
        }

        srt.sr = readers as *mut bcf_srs_t;
        if srt.nsr < readers.nreaders {
            let allocated = srt.msr.max(srt.nsr).max(0);
            let mut vcf_buf = take_vcf_buf_vec(srt, allocated);
            vcf_buf.resize_with(readers.nreaders as usize, || zeroed());
            store_vcf_buf_vec(srt, vcf_buf);
        }
        srt.nsr = readers.nreaders;
        srt.chr = std::ptr::null();
        0
    }
}

unsafe fn bcf_sr_sort_append_empty_row(vcf_buf: &mut [BcfSrSortVcfBuf]) -> i32 {
    unsafe {
        if vcf_buf.is_empty() {
            return -1;
        }
        let row = vcf_buf[0].nrec + 1;
        for buf in vcf_buf.iter_mut() {
            if bcf_sr_sort_reserve_row(buf, row) < 0 {
                return -1;
            }
            let Some(recs) = rec_storage_slice_mut(buf) else {
                return -1;
            };
            recs[(row - 1) as usize] = std::ptr::null_mut();
        }
        for buf in vcf_buf {
            buf.nrec = row;
        }
        0
    }
}

unsafe fn bcf_sr_sort_shift_reader_buffer(reader: &mut bcf_sr_t, j: i32) -> i32 {
    unsafe {
        if reader.buffer.is_null() || j < 1 || j > reader.nbuffer {
            return -1;
        }
        let Some(len) = reader.nbuffer.checked_add(1) else {
            return -1;
        };
        let buffer = std::slice::from_raw_parts_mut(reader.buffer, len as usize);
        let tmp = buffer[0];
        buffer[0] = buffer[j as usize];
        buffer.copy_within((j + 1) as usize..len as usize, j as usize);
        buffer[reader.nbuffer as usize] = tmp;
        reader.nbuffer -= 1;
        0
    }
}

fn bcf_sr_sort_disambiguate_duplicate_key_nonnull(
    key: &mut Vec<u8>,
    seen: &[(Vec<u8>, i32, NonNull<bcf1_t>)],
    reader_idx: i32,
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

pub unsafe fn bcf_sr_sort_set_active(srt: &mut BcfSrSort, idx: i32) -> i32 {
    unsafe {
        let Some(need) = idx.checked_add(1) else {
            return -1;
        };
        if idx < 0 || bcf_sr_sort_reserve_active(srt, need) < 0 {
            return -1;
        }
        {
            let Some(active) = active_slice_mut(srt) else {
                return -1;
            };
            active[0] = idx;
        }
        srt.nactive = 1;
        0
    }
}

pub unsafe fn bcf_sr_sort_add_active(srt: &mut BcfSrSort, idx: i32) -> i32 {
    unsafe {
        if idx < 0 {
            return -1;
        }
        let Some(idx_need) = idx.checked_add(1) else {
            return -1;
        };
        let Some(active_need) = srt.nactive.checked_add(1) else {
            return -1;
        };
        let need = idx_need.max(active_need);
        if bcf_sr_sort_reserve_active(srt, need) < 0 {
            return -1;
        }
        let pos = srt.nactive as usize;
        {
            let Some(active) = active_slice_mut(srt) else {
                return -1;
            };
            active[pos] = idx;
        }
        srt.nactive += 1;
        0
    }
}

// RECONVERGE: callers in vcf.rs still pass `&CStr` (e.g. c"chr1"); param is now a
// NUL-terminated `&[u8]`.
pub unsafe fn bcf_sr_sort_set(
    readers: &mut bcf_srs_t,
    srt: &mut BcfSrSort,
    chr: &[u8],
    min_pos: hts_pos_t,
) -> i32 {
    unsafe {
        if bcf_sr_sort_reserve_vcf_buf(readers, srt) < 0 {
            return -1;
        }
        // srt.chr is a shared (repr(C)) *const u8 field that the still-C-mirror
        // grouping code dereferences as a NUL-terminated string, so it must alias the
        // caller-owned NUL-terminated buffer that `chr` borrows.
        let chr_ptr = chr.as_ptr();

        let nsr = srt.nsr;
        let active = match active_slice(srt) {
            Some(active) => active.to_vec(),
            None => return -1,
        };
        let reader_count = readers.nreaders;
        let reader_slice = match readers_slice_mut(readers) {
            Some(readers) => readers,
            None => return -1,
        };
        let vcf_buf = match vcf_buf_slice_mut(srt) {
            Some(vcf_buf) => vcf_buf,
            None => return -1,
        };
        for buf in vcf_buf.iter_mut() {
            buf.nrec = 0;
        }

        let mut records: Vec<(Vec<u8>, i32, NonNull<bcf1_t>)> = Vec::new();
        for reader_idx in active {
            if reader_idx < 0 || reader_idx >= reader_count {
                return -1;
            }
            let reader = &mut reader_slice[reader_idx as usize];
            if reader.buffer.is_null() || reader.nbuffer <= 0 {
                continue;
            }

            let first_buffered = NonNull::new(*reader.buffer.add(1));
            let rid = if !reader.header.is_null() {
                bcf_hdr_name2id(reader.header, chr_ptr)
            } else if let Some(first_buffered) = first_buffered {
                first_buffered.as_ref().rid
            } else {
                -1
            };
            if rid < 0 {
                continue;
            }

            let buffered_records =
                std::slice::from_raw_parts(reader.buffer.add(1), reader.nbuffer as usize);
            for &rec in buffered_records {
                let Some(rec) = NonNull::new(rec) else {
                    break;
                };
                if rec.as_ref().rid != rid || rec.as_ref().pos != min_pos {
                    break;
                }
                let Some(mut key) = bcf_sr_sort_record_key(reader.header, rec.as_ptr()) else {
                    return -1;
                };
                bcf_sr_sort_disambiguate_duplicate_key_nonnull(&mut key, &records, reader_idx);
                records.push((key, reader_idx, rec));
            }
        }

        let mut row_keys: Vec<Vec<u8>> = Vec::new();
        for (key, reader_idx, rec) in records {
            let row = row_keys.iter().position(|row_key| row_key == &key);
            let row = match row {
                Some(row) => row,
                None => {
                    if bcf_sr_sort_append_empty_row(vcf_buf) < 0 {
                        return -1;
                    }
                    row_keys.push(key);
                    row_keys.len() - 1
                }
            };
            let buf = &mut vcf_buf[reader_idx as usize];
            let occupied = {
                let recs = match rec_slice_mut(buf) {
                    Some(recs) => recs,
                    None => return -1,
                };
                if recs[row] == rec.as_ptr() {
                    continue;
                }
                !recs[row].is_null()
            };
            if occupied {
                if bcf_sr_sort_append_empty_row(vcf_buf) < 0 {
                    return -1;
                }
                row_keys.push(row_keys[row].clone());
                let next_row = row_keys.len() - 1;
                let Some(recs) = rec_slice_mut(&mut vcf_buf[reader_idx as usize]) else {
                    return -1;
                };
                recs[next_row] = rec.as_ptr();
            } else {
                let Some(recs) = rec_slice_mut(&mut vcf_buf[reader_idx as usize]) else {
                    return -1;
                };
                recs[row] = rec.as_ptr();
            }
        }

        srt.chr = chr_ptr;
        srt.pos = min_pos;
        srt.nsr = nsr;
        0
    }
}

// RECONVERGE: callers in vcf.rs still pass `&CStr` (e.g. c"chr1"); param is now a
// NUL-terminated `&[u8]`.
pub unsafe fn bcf_sr_sort_next(
    readers: &mut bcf_srs_t,
    srt: &mut BcfSrSort,
    chr: &[u8],
    min_pos: hts_pos_t,
) -> i32 {
    unsafe {
        if srt.nactive <= 0 || srt.active.is_null() {
            return -1;
        }
        if bcf_sr_sort_reserve_vcf_buf(readers, srt) < 0 {
            return -1;
        }
        let reader_count = readers.nreaders;

        if srt.nactive == 1 {
            let (reader_slice, has_line) = match reader_and_has_line_slices_mut(readers) {
                Some(parts) => parts,
                None => return -1,
            };
            if reader_count > 1 {
                has_line.fill(0);
            }
            let active = match active_slice(srt).and_then(|active| active.first().copied()) {
                Some(active) => active,
                None => return -1,
            };
            if active < 0 || active >= reader_count {
                return -1;
            }
            let reader = &mut reader_slice[active as usize];
            if reader.buffer.is_null()
                || reader.nbuffer < 1
                || (*reader.buffer.add(1)).is_null()
                || (*(*reader.buffer.add(1))).pos != min_pos
            {
                return -1;
            }
            if bcf_sr_sort_shift_reader_buffer(reader, 1) < 0 {
                return -1;
            }
            has_line[active as usize] = 1;
            return 1;
        }

        // Compare the NUL-terminated bytes of `srt.chr` against `chr` (also
        // NUL-terminated).
        let chr_bytes = chr.split(|&b| b == 0).next().unwrap_or(&[]);
        let srt_chr_matches = !srt.chr.is_null() && {
            let mut len = 0usize;
            while *srt.chr.add(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(srt.chr, len) == chr_bytes
        };
        if (srt.chr.is_null() || srt.pos != min_pos || !srt_chr_matches)
            && bcf_sr_sort_set(readers, srt, chr, min_pos) < 0
        {
            return -1;
        }

        let (reader_slice, has_line) = match reader_and_has_line_slices_mut(readers) {
            Some(parts) => parts,
            None => return -1,
        };
        let vcf_buf = match vcf_buf_slice_mut(srt) {
            Some(vcf_buf) => vcf_buf,
            None => return -1,
        };
        if vcf_buf.first().is_none_or(|buf| buf.nrec <= 0) {
            return 0;
        }

        let mut nret = 0;
        for (i, buf) in vcf_buf.iter_mut().enumerate() {
            if buf.nrec > 0 && !buf.rec.is_null() && !(*buf.rec).is_null() {
                if i >= reader_slice.len() || i >= has_line.len() {
                    return -1;
                }
                let reader = &mut reader_slice[i];
                let rec = *buf.rec;
                if reader.buffer.is_null() || reader.nbuffer < 1 {
                    return -1;
                }
                let queue =
                    std::slice::from_raw_parts(reader.buffer.add(1), reader.nbuffer as usize);
                let Some(pos) = queue.iter().position(|&queued_rec| queued_rec == rec) else {
                    return -1;
                };
                let j = (pos + 1) as i32;
                if bcf_sr_sort_shift_reader_buffer(reader, j) < 0 {
                    return -1;
                }
                nret += 1;
                has_line[i] = 1;
            } else {
                if i >= has_line.len() {
                    return -1;
                }
                has_line[i] = 0;
            }

            if buf.nrec > 0 {
                if buf.nrec > 1 {
                    let recs = match rec_slice_mut(buf) {
                        Some(recs) => recs,
                        None => return -1,
                    };
                    recs.copy_within(1.., 0);
                }
                buf.nrec -= 1;
            }
        }
        nret
    }
}

pub unsafe fn bcf_sr_sort_remove_reader(srt: &mut BcfSrSort, i: i32) {
    unsafe {
        if srt.vcf_buf.is_null() || i < 0 || i >= srt.nsr {
            return;
        }

        let nsr = srt.nsr;
        let Some(vcf_buf) = vcf_buf_slice_mut(srt) else {
            return;
        };
        let idx = i as usize;
        drop(take_rec_vec(&mut vcf_buf[idx]));
        if i + 1 < nsr {
            let ptr = vcf_buf.as_mut_ptr();
            std::ptr::copy(ptr.add(idx + 1), ptr.add(idx), (nsr - i - 1) as usize);
        }
        vcf_buf[nsr as usize - 1] = zeroed();
    }
}

pub fn bcf_sr_sort_init(srt: &mut BcfSrSort) {
    *srt = BcfSrSort::default();
}

pub fn bcf_sr_sort_reset(srt: &mut BcfSrSort) {
    srt.chr = std::ptr::null();
}

pub unsafe fn bcf_sr_sort_destroy(srt: &mut BcfSrSort) {
    unsafe {
        drop(take_active_vec(srt));
        crate::sam::khash_str2int_destroy_free(srt.var_str2int.cast());
        crate::sam::khash_str2int_destroy_free(srt.grp_str2int.cast());

        let nbuf = srt.msr.max(srt.nsr).max(0);
        let vcf_buf = srt.vcf_buf.cast::<BcfSrSortVcfBuf>();
        if !vcf_buf.is_null() {
            for buf in std::slice::from_raw_parts_mut(vcf_buf, nbuf as usize) {
                drop(take_rec_vec(buf));
            }
        }
        drop(take_vcf_buf_vec(srt, nbuf));

        let var = srt.var.cast::<BcfSrSortVar>();
        if !var.is_null() {
            for var in std::slice::from_raw_parts_mut(var, srt.mvar.max(0) as usize) {
                libc::free(var.str_.cast());
                libc::free(var.vcf.cast());
                libc::free(var.rec.cast());
                kbs_destroy(var.mask);
            }
        }
        libc::free(srt.var.cast());

        let grp = srt.grp.cast::<BcfSrSortGrp>();
        if !grp.is_null() {
            for grp in std::slice::from_raw_parts_mut(grp, srt.mgrp.max(0) as usize) {
                libc::free(grp.var.cast());
            }
        }
        libc::free(srt.grp.cast());

        let vset = srt.vset.cast::<BcfSrSortVarSet>();
        if !vset.is_null() {
            for vset in std::slice::from_raw_parts_mut(vset, srt.mvset.max(0) as usize) {
                kbs_destroy(vset.mask);
                libc::free(vset.var.cast());
            }
        }
        libc::free(srt.vset.cast());

        crate::htslib_rs::hts::ks_free(&mut srt.str_);
        libc::free(srt.off.cast());
        libc::free(srt.charp.cast());
        libc::free(srt.cnt.cast());
        libc::free(srt.pmat.cast());
        *srt = BcfSrSort::default();
    }
}
