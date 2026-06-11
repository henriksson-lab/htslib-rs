// Functions translated from htslib/vcf_sweep.c.
// Extracted from src/synced_bcf_reader.rs (2026-06-01).

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

    // Re-NUL-terminate at the libhts boundary.
    let fname_c = std::ffi::CString::new(fname).unwrap();
    let file = hts_open(fname_c.as_ptr(), c"r".as_ptr());
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
        rec: vec![bcf1_t::default()],
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
        if let Some(&offset) = sw.idx.first() {
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

    let ret = sweep_useek(sw, sw.idx[sw.iidx] as i64, 0);
    assert!(ret == 0);

    sw.nrec = 0;
    loop {
        if sw.rec.len() < sw.nrec + 1 {
            sw.rec.resize_with(sw.nrec + 1, bcf1_t::default);
        }
        let rec = &mut sw.rec[sw.nrec];
        if bcf_read1(sw.file, sw.hdr, rec) != 0 {
            break;
        }
        bcf_unpack(rec, BCF_UN_STR as i32);

        // if not in the last block, stop at the saved record
        if sw.iidx + 1 < sw.idx.len() {
            let rec_snapshot = (
                sw.rec[sw.nrec].rid,
                sw.rec[sw.nrec].pos as i32,
                sw.rec[sw.nrec].n_allele() as i32,
                sweep_alleles(&sw.rec[sw.nrec]),
            );
            if sw.lrid == rec_snapshot.0
                && sw.lpos == rec_snapshot.1
                && sw.lnals == rec_snapshot.2
                && sw.lals.as_slice() == rec_snapshot.3.as_slice()
            {
                break;
            }
        }

        sw.nrec += 1;
        if sw.rec.len() < sw.nrec + 1 {
            sw.rec.resize_with(sw.nrec + 1, bcf1_t::default);
        }
    }
    let first = sw.rec.first().expect("sweep record buffer");
    let lrid = first.rid;
    let lpos = first.pos as i32;
    let lnals = first.n_allele() as i32;
    let lals = sweep_alleles(first);
    sw.lrid = lrid;
    sw.lpos = lpos;
    sw.lnals = lnals;
    sw.lals = lals;
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

unsafe fn sweep_alleles(rec: &bcf1_t) -> Vec<u8> {
    // The C original returned a slice of the packed `als` buffer spanning
    // `allele[0]` through the last byte of `allele[n-1]` (the inter-allele
    // NUL separators included, the final allele's terminating NUL excluded).
    // With owned alleles, reconstruct that comparable byte sequence by
    // joining each allele with a single NUL separator.
    let mut out = Vec::new();
    let n = rec.n_allele() as usize;
    for i in 0..n {
        if i != 0 {
            out.push(0);
        }
        out.extend_from_slice(&rec.d.allele[i]);
    }
    out
}

pub unsafe fn bcf_sweep_fwd(sw: &mut bcf_sweep_t) -> Option<&mut bcf1_t> {
    // Native translation of htslib/vcf_sweep.c bcf_sweep_fwd().
    if sw.direction == SW_BWD {
        sweep_seek(sw, SW_FWD);
    }

    let pos = sweep_tell(sw);

    let ret = bcf_read1(sw.file, sw.hdr, &mut sw.rec[0]);

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
    Some(&mut sw.rec[0])
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
    let record_index = sw.nrec;
    sw.rec.get_mut(record_index)
}

pub unsafe fn bcf_sweep_hdr(sw: &mut bcf_sweep_t) -> Option<&mut bcf_hdr_t> {
    sw.hdr.as_mut()
}
