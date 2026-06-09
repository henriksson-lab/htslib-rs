// Functions translated from htslib/vcf_sweep.c.
// Extracted from src/synced_bcf_reader.rs (2026-06-01).

use std::ffi::c_int;

use crate::htslib_rs::bgzf::{bgzf_index_build_init, bgzf_useek};
use crate::htslib_rs::hfile::hseek;
use crate::htslib_rs::hts::{hts_close, hts_get_bgzfp, hts_open};
use crate::htslib_rs::vcf::{
    bcf1_t, bcf_empty1, bcf_hdr_destroy, bcf_hdr_read, bcf_hdr_t, bcf_read1, bcf_sweep_t,
    bcf_unpack, sw_utell, BCF_UN_STR, SW_BWD, SW_FWD,
};

pub unsafe fn bcf_sweep_init(fname: &[u8]) -> Option<Box<bcf_sweep_t>> {
    if fname.contains(&0) {
        return None;
    }
    let mut fname = fname.to_vec();
    fname.push(0);

    let file = hts_open(fname.as_ptr().cast(), c"r".as_ptr());
    if file.is_null() {
        return None;
    }

    let fp = hts_get_bgzfp(file);
    if !fp.is_null() {
        bgzf_index_build_init(fp);
    }

    let hdr = bcf_hdr_read(file);
    if hdr.is_null() {
        hts_close(file);
        return None;
    }

    Some(Box::new(bcf_sweep_t {
        file,
        hdr,
        fp,
        direction: SW_FWD,
        block_size: 1024 * 1024 * 3,
        rec: vec![std::mem::zeroed::<bcf1_t>()],
        nrec: 0,
        lrid: 0,
        lpos: 0,
        lnals: 0,
        lals: Vec::new(),
        idx: Vec::new(),
        iidx: 0,
        idx_done: 0,
    }))
}

pub unsafe fn bcf_sweep_destroy(sw: Option<Box<bcf_sweep_t>>) {
    let Some(mut sw) = sw else { return };
    for rec in sw.rec.iter_mut() {
        bcf_empty1(rec);
    }
    bcf_hdr_destroy(sw.hdr);
    hts_close(sw.file);
}

unsafe fn sweep_seek(sw: &mut bcf_sweep_t, direction: i32) {
    sw.direction = direction;
    if direction == SW_FWD {
        if let Some(offset) = index_slice(sw).first().copied() {
            sweep_useek(sw, offset as i64, 0);
        }
    } else {
        sw.iidx = sw.idx.len();
        sw.nrec = 0;
    }
}

unsafe fn sweep_fill_buffer(sw: &mut bcf_sweep_t) {
    if sw.iidx == 0 {
        return;
    }
    sw.iidx -= 1;

    let ret = sweep_useek(sw, index_slice(sw)[sw.iidx as usize] as i64, 0);
    assert!(ret == 0);

    sw.nrec = 0;
    loop {
        sweep_grow_records(sw, sw.nrec + 1);
        let rec = sw.rec.as_mut_ptr().add(sw.nrec);
        if bcf_read1(sw.file, sw.hdr, rec) != 0 {
            break;
        }
        bcf_unpack(rec, BCF_UN_STR as i32);

        // if not in the last block, stop at the saved record
        if sw.iidx + 1 < sw.idx.len() && sweep_rec_equal(sw, &*rec) {
            break;
        }

        sw.nrec += 1;
        sweep_grow_records(sw, sw.nrec + 1);
    }
    let saved = sweep_rec_snapshot(sw.rec.first().expect("sweep record buffer"));
    sweep_store_saved_rec(sw, saved);
}

unsafe fn sweep_tell(sw: &mut bcf_sweep_t) -> i64 {
    sw_utell(sw.file)
}

unsafe fn sweep_useek(sw: &mut bcf_sweep_t, uoffset: i64, where_: i32) -> i32 {
    if ((*sw.file).bitfields & (1 << 4)) != 0 {
        bgzf_useek((*sw.file).fp.bgzf, uoffset, where_)
    } else if hseek((*sw.file).fp.hfile, uoffset as libc::off_t, libc::SEEK_SET) >= 0 {
        0
    } else {
        -1
    }
}

unsafe fn sweep_grow_records(sw: &mut bcf_sweep_t, n: usize) {
    if n <= sw.rec.len() {
        return;
    }

    sw.rec.resize_with(n, || std::mem::zeroed::<bcf1_t>());
}

unsafe fn sweep_rec_equal(sw: &bcf_sweep_t, rec: &bcf1_t) -> bool {
    if sw.lrid != rec.rid {
        return false;
    }
    if sw.lpos != rec.pos as i32 {
        return false;
    }
    if sw.lnals != rec.n_allele() as i32 {
        return false;
    }

    let alleles = sweep_alleles(rec);
    if sw.lals.len() != alleles.len() {
        return false;
    }
    sw.lals == alleles
}

unsafe fn sweep_rec_save(sw: &mut bcf_sweep_t, rec: &bcf1_t) {
    let saved = sweep_rec_snapshot(rec);
    sweep_store_saved_rec(sw, saved);
}

fn sweep_store_saved_rec(sw: &mut bcf_sweep_t, saved: (c_int, c_int, c_int, Vec<u8>)) {
    let (lrid, lpos, lnals, alleles) = saved;
    sw.lrid = lrid;
    sw.lpos = lpos;
    sw.lnals = lnals;
    sw.lals.clear();
    sw.lals.extend_from_slice(&alleles);
}

unsafe fn sweep_rec_snapshot(rec: &bcf1_t) -> (c_int, c_int, c_int, Vec<u8>) {
    let alleles = sweep_alleles(rec);
    (
        rec.rid,
        rec.pos as i32,
        rec.n_allele() as i32,
        alleles.to_vec(),
    )
}

unsafe fn sweep_alleles(rec: &bcf1_t) -> &[u8] {
    let allele0 = *rec.d.allele;
    let mut t = *rec.d.allele.add(rec.n_allele() as usize - 1);
    let mut len = (t as isize - allele0 as isize) as usize + 1;
    while *t != 0 {
        t = t.add(1);
        len += 1;
    }
    std::slice::from_raw_parts(allele0.cast::<u8>(), len)
}

pub unsafe fn bcf_sweep_fwd(sw: &mut bcf_sweep_t) -> Option<&mut bcf1_t> {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_fwd().
    if sw.direction == SW_BWD {
        sweep_seek(sw, SW_FWD);
    }

    let pos = sweep_tell(sw);

    let rec = sw.rec.as_mut_ptr();
    let ret = bcf_read1(sw.file, sw.hdr, rec);

    if ret != 0 {
        // last record, get ready for sweeping backwards
        sw.idx_done = 1;
        if let Some(fp) = sw.fp.as_mut() {
            fp.idx_build_otf = 0;
        }
        sweep_seek(sw, SW_BWD);
        return None;
    }

    if sw.idx_done == 0
        && (sw.idx.is_empty()
            || pos - *sw.idx.last().expect("sweep index") as i64 > sw.block_size as i64)
    {
        sw.idx.push(pos as u64);
    }
    rec.as_mut()
}

pub unsafe fn bcf_sweep_bwd(sw: &mut bcf_sweep_t) -> Option<&mut bcf1_t> {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_bwd().
    if sw.direction == SW_FWD {
        sweep_seek(sw, SW_BWD);
    }
    if sw.nrec == 0 {
        sweep_fill_buffer(sw);
    }
    if sw.nrec == 0 {
        return None;
    }
    sw.nrec -= 1;
    let record_index = sw.nrec as usize;
    sw.rec.get_mut(record_index)
}

pub unsafe fn bcf_sweep_hdr(sw: &mut bcf_sweep_t) -> Option<&mut bcf_hdr_t> {
    sw.hdr.as_mut()
}

unsafe fn index_slice(sw: &bcf_sweep_t) -> &[u64] {
    &sw.idx
}
