// Functions translated from htslib/vcf_sweep.c.
// Extracted from src/synced_bcf_reader.rs (2026-06-01).

use std::ffi::c_char;

use crate::htslib_rs::bgzf::bgzf_index_build_init;
use crate::htslib_rs::hts::{hts_close, hts_get_bgzfp, hts_open};
use crate::htslib_rs::vcf::{
    bcf1_t, bcf_empty1, bcf_hdr_destroy, bcf_hdr_read, bcf_hdr_t, bcf_read1, bcf_sweep_t,
    sw_fill_buffer, sw_seek, sw_utell, SW_BWD, SW_FWD,
};

pub unsafe fn bcf_sweep_init(fname: *const c_char) -> *mut bcf_sweep_t {
    let file = hts_open(fname, c"r".as_ptr());
    let fp = hts_get_bgzfp(file);
    if !fp.is_null() {
        bgzf_index_build_init(fp);
    }

    let mrec = 1;
    let rec = libc::calloc(mrec as usize, size_of::<bcf1_t>()).cast::<bcf1_t>();

    Box::into_raw(Box::new(bcf_sweep_t {
        file,
        hdr: bcf_hdr_read(file),
        fp,
        direction: SW_FWD,
        block_size: 1024 * 1024 * 3,
        rec,
        nrec: 0,
        mrec,
        lrid: 0,
        lpos: 0,
        lnals: 0,
        lals_len: 0,
        mlals: 0,
        lals: std::ptr::null_mut(),
        idx: std::ptr::null_mut(),
        iidx: 0,
        nidx: 0,
        midx: 0,
        idx_done: 0,
    }))
}

pub unsafe fn bcf_sweep_destroy(sw: *mut bcf_sweep_t) {
    if sw.is_null() {
        return;
    }

    let sw = Box::from_raw(sw);
    for i in 0..sw.mrec {
        bcf_empty1(sw.rec.add(i as usize));
    }
    if !sw.idx.is_null() {
        drop(Vec::from_raw_parts(
            sw.idx,
            sw.nidx as usize,
            sw.midx as usize,
        ));
    }
    libc::free(sw.rec.cast());
    libc::free(sw.lals.cast());
    bcf_hdr_destroy(sw.hdr);
    hts_close(sw.file);
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
        let mut idx = if (*sw).idx.is_null() {
            Vec::new()
        } else {
            Vec::from_raw_parts((*sw).idx, (*sw).nidx as usize, (*sw).midx as usize)
        };
        idx.push(pos as u64);
        (*sw).idx = idx.as_mut_ptr();
        (*sw).nidx = idx.len() as _;
        (*sw).midx = idx.capacity() as _;
        std::mem::forget(idx);
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
