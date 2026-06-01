use std::ffi::{c_char, c_int};

use crate::htslib_rs::{
    c_compat::{__errno_location, ENOMEM},
    hts::{htsLogLevel, hts_pos_t, HTS_LOG_ERROR, HTS_LOG_WARNING},
    probaln::{probaln_glocal, probaln_par_t},
    sam::{
        bam1_t, bam_aux_append, bam_aux_del, bam_aux_get, bam_cigar_op, bam_get_cigar,
        bam_get_qual, bam_get_seq, bam_seqi, BAM_CDEL, BAM_CDIFF, BAM_CEQUAL, BAM_CHARD_CLIP,
        BAM_CINS, BAM_CMATCH, BAM_CREF_SKIP, BAM_CSOFT_CLIP, BAM_FUNMAP, SEQ_NT16_TABLE,
    },
};

const BAQ_APPLY: c_int = 1;
const BAQ_EXTEND: c_int = 2;
const BAQ_REDO: c_int = 4;
const BAQ_ILLUMINA: c_int = 1 << 3;

const SEQ_NT16_INT: [u8; 16] = [4, 0, 1, 4, 2, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4];

unsafe fn realn_check_tag(
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

// original: sam_prob_realn (htslib/realn.c:106)
pub unsafe fn realn_c_106_sam_prob_realn(
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
    let apply_baq = flag & BAQ_APPLY;
    let extend_baq = flag & BAQ_EXTEND;
    let redo_baq = flag & BAQ_REDO;
    let system = flag & (0xff << 3);
    let mut i: hts_pos_t;
    let mut x: hts_pos_t;
    let cigar = bam_get_cigar(b);

    // d(I) e(M) band
    let mut conf = probaln_par_t {
        d: 0.001,
        e: 0.1,
        bw: 10,
    }; // Illumina

    if (*b).core.l_qseq > 1000 || system > BAQ_ILLUMINA {
        // Params that work well on PacBio CCS 15k.  Unknown if they
        // help other long-read platforms yet, but likely better than
        // the short-read tuned ones.
        //
        // This function has no access to the SAM header.
        // Ideally the calling function would check for e.g.
        // @RG PL = "PACBIO" and DS contains "READTYPE=CCS".
        //
        // In the absense of this, we simply auto-detect via a crude
        // short vs long strategy.
        conf.d = 1e-7;
        conf.e = 1e-1;
    }

    let qual = bam_get_qual(b).cast_mut();
    if ((*b).core.flag as c_int & BAM_FUNMAP) != 0 || (*b).core.l_qseq == 0 || *qual == u8::MAX {
        return -1; // do nothing
    }

    // test if BQ or ZQ is present, and make sanity checks
    let mut bq = bam_aux_get(b, c"BQ".as_ptr());
    if !bq.is_null() {
        if redo_baq == 0 && realn_check_tag(bq, HTS_LOG_WARNING, c"BQ".as_ptr(), b) < 0 {
            fix_bq = 1;
        }
        bq = bq.add(1);
    }
    let mut zq = bam_aux_get(b, c"ZQ".as_ptr());
    if !zq.is_null() {
        if realn_check_tag(zq, HTS_LOG_ERROR, c"ZQ".as_ptr(), b) < 0 {
            return -4;
        }
        zq = zq.add(1);
    }
    if !bq.is_null() && redo_baq != 0 {
        bam_aux_del(b, bq.sub(1));
        bq = std::ptr::null_mut();
    }
    if !bq.is_null() && !zq.is_null() {
        // remove the ZQ tag
        bam_aux_del(b, zq.sub(1));
        zq = std::ptr::null_mut();
    }
    if zq.is_null() && fix_bq != 0 {
        // Need to fix invalid BQ tag (by realigning)
        debug_assert!(!bq.is_null());
        bam_aux_del(b, bq.sub(1));
        bq = std::ptr::null_mut();
    }

    if !bq.is_null() || !zq.is_null() {
        if (apply_baq != 0 && !zq.is_null()) || (apply_baq == 0 && !bq.is_null()) {
            return -3; // in both cases, do nothing
        }
        if !bq.is_null() && apply_baq != 0 {
            // then convert BQ to ZQ
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
            // then convert ZQ to BQ
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

    // find the start and end of the alignment
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
            return -1; // do nothing if there is a reference skip
        }
        k += 1;
    }
    if xb == -1 {
        // No matches in CIGAR.
        return -1;
    }

    // set bandwidth and the start and the end
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

    {
        // glocal
        let seq = bam_get_seq(b);
        let mut lref = if xe > xb { (xe - xb) as usize } else { 1 };
        if extend_baq != 0 && lref < (*b).core.l_qseq as usize {
            lref = (*b).core.l_qseq as usize; // So we can recycle tseq,tref for left,rght below
        }
        // Try to make q,tref,tseq reasonably well aligned
        let align_lqseq = (((*b).core.l_qseq as usize + 1) | 0xf) + 1;
        // Overflow check - 3 for *bq, sizeof(int) for *state
        if (usize::MAX - lref) / (3 + std::mem::size_of::<c_int>()) < align_lqseq {
            *__errno_location() = ENOMEM as c_int;
            return -4;
        }

        debug_assert!(bq.is_null()); // bq was used above, but should now be NULL
        let Some(total) = align_lqseq.checked_mul(3).and_then(|n| n.checked_add(lref)) else {
            *__errno_location() = ENOMEM as c_int;
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
        if probaln_glocal(
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
            // in this block, bq[] is capped by base quality qual[]
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
                    // Sanity check running off the end of the sequence
                    // Can only happen if the alignment is broken
                    if l > (*b).core.l_qseq - y {
                        l = (*b).core.l_qseq - y;
                    }
                    i = y as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        if (*state.add(i as usize) & 3) != 0
                            || (*state.add(i as usize) >> 2)
                                != (x - xb + (i - y as hts_pos_t)) as c_int
                        {
                            *bq.add(i as usize) = 0;
                        } else {
                            *bq.add(i as usize) = if *bq.add(i as usize) < *q.add(i as usize) {
                                *bq.add(i as usize)
                            } else {
                                *q.add(i as usize)
                            };
                        }
                        i += 1;
                    }
                    x += l as hts_pos_t;
                    y += l;
                } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
                    // Need sanity check here too.
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
                    ((*qual.add(i as usize) as c_int) - (*bq.add(i as usize) as c_int) + 64) as u8; // finalize BQ
                i += 1;
            }
        } else {
            // in this block, bq[] is BAQ that can be larger than qual[] (different from the above!)
            // tseq,tref are no longer needed, so we can steal them to avoid mallocs
            let left = tseq;
            let rght = tref;
            let mut len: c_int = 0;

            k = 0;
            x = (*b).core.pos;
            y = 0;
            while k < (*b).core.n_cigar as c_int {
                let op = (*cigar.add(k as usize) & 0xf) as c_int;
                let mut l = (*cigar.add(k as usize) >> 4) as c_int;

                // concatenate alignment matches (including sequence (mis)matches)
                // otherwise 50M50M gives a different result to 100M
                if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                    if k + 1 < (*b).core.n_cigar as c_int {
                        let next_op = bam_cigar_op(*cigar.add(k as usize + 1));

                        if next_op == BAM_CMATCH || next_op == BAM_CEQUAL || next_op == BAM_CDIFF {
                            len += l;
                            k += 1;
                            continue;
                        }
                    }

                    // last of M/X/= ops
                    l += len;
                    len = 0;
                }

                if l == 0 {
                    k += 1;
                    continue;
                }
                if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                    // Sanity check running off the end of the sequence
                    // Can only happen if the alignment is broken
                    if l > (*b).core.l_qseq - y {
                        l = (*b).core.l_qseq - y;
                    }
                    i = y as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        *bq.add(i as usize) = if (*state.add(i as usize) & 3) != 0
                            || (*state.add(i as usize) >> 2)
                                != (x - xb + (i - y as hts_pos_t)) as c_int
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
                        *left.add(i as usize) = if *bq.add(i as usize) > *left.add(i as usize - 1) {
                            *bq.add(i as usize)
                        } else {
                            *left.add(i as usize - 1)
                        };
                        i += 1;
                    }
                    *rght.add((y + l - 1) as usize) = *bq.add((y + l - 1) as usize);
                    if l > 1 {
                        i = (y + l - 2) as hts_pos_t;
                        loop {
                            *rght.add(i as usize) =
                                if *bq.add(i as usize) > *rght.add(i as usize + 1) {
                                    *bq.add(i as usize)
                                } else {
                                    *rght.add(i as usize + 1)
                                };
                            if i == y as hts_pos_t {
                                break;
                            }
                            i -= 1;
                        }
                    }
                    i = y as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        *bq.add(i as usize) = if *left.add(i as usize) < *rght.add(i as usize) {
                            *left.add(i as usize)
                        } else {
                            *rght.add(i as usize)
                        };
                        i += 1;
                    }
                    x += l as hts_pos_t;
                    y += l;
                } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
                    // Need sanity check here too.
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
                    }) as u8; // finalize BQ
                i += 1;
            }
        }
        if apply_baq != 0 {
            i = 0;
            while i < (*b).core.l_qseq as hts_pos_t {
                *qual.add(i as usize) = ((*qual.add(i as usize) as c_int)
                    - ((*bq.add(i as usize) as c_int) - 64))
                    as u8; // modify qual
                i += 1;
            }
            bam_aux_append(b, c"ZQ".as_ptr(), b'Z' as c_char, (*b).core.l_qseq + 1, bq);
        } else {
            bam_aux_append(b, c"BQ".as_ptr(), b'Z' as c_char, (*b).core.l_qseq + 1, bq);
        }
    }

    0
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
