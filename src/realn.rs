use std::ptr::NonNull;

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

const BAQ_APPLY: i32 = 1;
const BAQ_EXTEND: i32 = 2;
const BAQ_REDO: i32 = 4;
const BAQ_ILLUMINA: i32 = 1 << 3;

const SEQ_NT16_INT: [u8; 16] = [4, 0, 1, 4, 2, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4];

unsafe fn bam_cigar_slice<'a>(b: &bam1_t) -> &'a [u32] {
    std::slice::from_raw_parts(bam_get_cigar(b), b.core.n_cigar as usize)
}

unsafe fn bam_seq_slice<'a>(b: &bam1_t) -> &'a [u8] {
    std::slice::from_raw_parts(bam_get_seq(b), (b.core.l_qseq as usize + 1) / 2)
}

unsafe fn bam_qual_slice<'a>(b: &bam1_t) -> &'a [u8] {
    std::slice::from_raw_parts(bam_get_qual(b), b.core.l_qseq as usize)
}

unsafe fn bam_qual_slice_mut<'a>(b: &mut bam1_t) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(bam_get_qual(b).cast_mut(), b.core.l_qseq as usize)
}

unsafe fn bam_aux_get_ref(b: &bam1_t, tag: &[u8]) -> Option<NonNull<u8>> {
    if tag.contains(&0) {
        return None;
    }

    let mut nul_tag = Vec::with_capacity(tag.len() + 1);
    nul_tag.extend_from_slice(tag);
    nul_tag.push(0);

    NonNull::new(bam_aux_get(b, nul_tag.as_ptr().cast()))
}

unsafe fn bam_aux_del_ref(b: &mut bam1_t, s: NonNull<u8>) -> i32 {
    bam_aux_del(b, s.as_ptr())
}

unsafe fn bam_aux_append_ref(b: &mut bam1_t, tag: &[u8], type_: i8, len: i32, data: &[u8]) -> i32 {
    if tag.contains(&0) {
        return -1;
    }

    let mut nul_tag = Vec::with_capacity(tag.len() + 1);
    nul_tag.extend_from_slice(tag);
    nul_tag.push(0);

    bam_aux_append(b, nul_tag.as_ptr().cast(), type_, len, data.as_ptr())
}

unsafe fn realn_check_tag(
    tg: NonNull<u8>,
    _severity: htsLogLevel,
    _type: &[u8],
    b: &bam1_t,
) -> i32 {
    let tg = tg.as_ptr();
    if *tg != b'Z' {
        return -1;
    }
    let tag_value = std::slice::from_raw_parts(tg.add(1), b.core.l_qseq as usize + 1);
    let value_len = tag_value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(tag_value.len());
    if b.core.l_qseq as usize != value_len {
        return -1;
    }
    0
}

// original: sam_prob_realn (htslib/realn.c:106)
pub unsafe fn realn_c_106_sam_prob_realn(b: &mut bam1_t, ref_: &[u8], flag: i32) -> i32 {
    sam_prob_realn_impl(b, ref_, flag)
}

unsafe fn sam_prob_realn_impl(b: &mut bam1_t, ref_: &[u8], flag: i32) -> i32 {
    let mut k: i32;
    let mut bw: i32;
    let mut y: i32;
    let mut yb: i32;
    let mut ye: i32;
    let mut xb: hts_pos_t;
    let mut xe: hts_pos_t;
    let mut fix_bq: i32 = 0;
    let apply_baq = flag & BAQ_APPLY;
    let extend_baq = flag & BAQ_EXTEND;
    let redo_baq = flag & BAQ_REDO;
    let system = flag & (0xff << 3);
    let l_qseq = b.core.l_qseq;
    let mut i: hts_pos_t;
    let mut x: hts_pos_t;
    let cigar = bam_cigar_slice(b);

    // d(I) e(M) band
    let mut conf = probaln_par_t {
        d: 0.001,
        e: 0.1,
        bw: 10,
    }; // Illumina

    if l_qseq > 1000 || system > BAQ_ILLUMINA {
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

    if (b.core.flag as i32 & BAM_FUNMAP) != 0
        || l_qseq == 0
        || bam_qual_slice(b).first().copied() == Some(u8::MAX)
    {
        return -1; // do nothing
    }

    // test if BQ or ZQ is present, and make sanity checks
    let mut bq = bam_aux_get_ref(b, b"BQ");
    if let Some(bq_tag) = bq {
        if redo_baq == 0 && realn_check_tag(bq_tag, HTS_LOG_WARNING, b"BQ", b) < 0 {
            fix_bq = 1;
        }
        bq = NonNull::new(bq_tag.as_ptr().add(1));
    }
    let mut zq = bam_aux_get_ref(b, b"ZQ");
    if let Some(zq_tag) = zq {
        if realn_check_tag(zq_tag, HTS_LOG_ERROR, b"ZQ", b) < 0 {
            return -4;
        }
        zq = NonNull::new(zq_tag.as_ptr().add(1));
    }
    if let Some(bq_ptr) = bq.filter(|_| redo_baq != 0) {
        bam_aux_del_ref(b, NonNull::new_unchecked(bq_ptr.as_ptr().sub(1)));
        bq = None;
    }
    if bq.is_some() && zq.is_some() {
        // remove the ZQ tag
        bam_aux_del_ref(b, NonNull::new_unchecked(zq.unwrap().as_ptr().sub(1)));
        zq = None;
    }
    if zq.is_none() && fix_bq != 0 {
        // Need to fix invalid BQ tag (by realigning)
        debug_assert!(bq.is_some());
        bam_aux_del_ref(b, NonNull::new_unchecked(bq.unwrap().as_ptr().sub(1)));
        bq = None;
    }

    if bq.is_some() || zq.is_some() {
        let qual = bam_qual_slice_mut(b);
        if (apply_baq != 0 && zq.is_some()) || (apply_baq == 0 && bq.is_some()) {
            return -3; // in both cases, do nothing
        }
        if let Some(bq_ptr) = bq.filter(|_| apply_baq != 0) {
            // then convert BQ to ZQ
            let bq = bq_ptr.as_ptr();
            i = 0;
            while i < l_qseq as hts_pos_t {
                let q = &mut qual[i as usize];
                let v = bq.add(i as usize);
                *q = if (*q as i32) + 64 < *v as i32 {
                    0
                } else {
                    ((*q as i32) - ((*v as i32) - 64)) as u8
                };
                i += 1;
            }
            *bq.sub(3) = b'Z';
        } else if let Some(zq_ptr) = zq.filter(|_| apply_baq == 0) {
            // then convert ZQ to BQ
            let zq = zq_ptr.as_ptr();
            i = 0;
            while i < l_qseq as hts_pos_t {
                qual[i as usize] =
                    ((qual[i as usize] as i32) + (*zq.add(i as usize) as i32) - 64) as u8;
                i += 1;
            }
            *zq.sub(3) = b'B';
        }
        return 0;
    }

    // find the start and end of the alignment
    x = b.core.pos;
    y = 0;
    yb = -1;
    ye = -1;
    xb = -1;
    xe = -1;
    k = 0;
    while k < b.core.n_cigar as i32 {
        let op = (cigar[k as usize] & 0xf) as i32;
        let l = (cigar[k as usize] >> 4) as i32;
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
        bw = ((xe - xb) - (ye - yb) as hts_pos_t).abs() as i32 + 3;
    }
    conf.bw = bw;

    xb -= yb as hts_pos_t + (bw / 2) as hts_pos_t;
    if xb < 0 {
        xb = 0;
    }
    xe += (l_qseq - ye) as hts_pos_t + (bw / 2) as hts_pos_t;
    if xe - xb - l_qseq as hts_pos_t > bw as hts_pos_t {
        xb += (xe - xb - l_qseq as hts_pos_t - bw as hts_pos_t) / 2;
        xe -= (xe - xb - l_qseq as hts_pos_t - bw as hts_pos_t) / 2;
    }

    {
        // glocal
        let seq = bam_seq_slice(b);
        let qual = bam_qual_slice_mut(b);
        let mut lref = if xe > xb { (xe - xb) as usize } else { 1 };
        if extend_baq != 0 && lref < l_qseq as usize {
            lref = l_qseq as usize; // So we can recycle tseq,tref for left,rght below
        }
        // Try to make q,tref,tseq reasonably well aligned
        let align_lqseq = ((l_qseq as usize + 1) | 0xf) + 1;
        // Overflow check - 3 for *bq, sizeof(int) for *state
        if (usize::MAX - lref) / (3 + std::mem::size_of::<i32>()) < align_lqseq {
            *__errno_location() = ENOMEM as i32;
            return -4;
        }

        debug_assert!(bq.is_none()); // bq was used above, but should now be NULL
        let Some(total) = align_lqseq.checked_mul(3).and_then(|n| n.checked_add(lref)) else {
            *__errno_location() = ENOMEM as i32;
            return -4;
        };
        let mut bq_buf = vec![0_u8; total];
        let (bq, rest) = bq_buf.split_at_mut(align_lqseq);
        let (q, rest) = rest.split_at_mut(align_lqseq);
        let (tseq, tref) = rest.split_at_mut(align_lqseq);

        bq[..l_qseq as usize].copy_from_slice(&qual[..l_qseq as usize]);
        bq[l_qseq as usize] = 0;
        i = 0;
        while i < l_qseq as hts_pos_t {
            tseq[i as usize] = SEQ_NT16_INT[bam_seqi(seq.as_ptr(), i as usize) as usize];
            i += 1;
        }
        i = xb;
        while i < xe {
            let Some(&base) = ref_.get(i as usize) else {
                xe = i;
                break;
            };
            if base == 0 {
                xe = i;
                break;
            }
            tref[(i - xb) as usize] = SEQ_NT16_INT[SEQ_NT16_TABLE[base as usize] as usize];
            i += 1;
        }

        let mut state_buf = vec![0_i32; l_qseq as usize];
        if probaln_glocal(
            &tref[..(xe - xb) as usize],
            &tseq[..l_qseq as usize],
            Some(&qual[..l_qseq as usize]),
            &conf,
            Some((state_buf.as_mut_slice(), &mut q[..l_qseq as usize])),
        ) == i32::MIN
        {
            return -4;
        }

        if extend_baq == 0 {
            // in this block, bq[] is capped by base quality qual[]
            k = 0;
            x = b.core.pos;
            y = 0;
            while k < b.core.n_cigar as i32 {
                let op = (cigar[k as usize] & 0xf) as i32;
                let mut l = (cigar[k as usize] >> 4) as i32;
                if l == 0 {
                    k += 1;
                    continue;
                }
                if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                    // Sanity check running off the end of the sequence
                    // Can only happen if the alignment is broken
                    if l > l_qseq - y {
                        l = l_qseq - y;
                    }
                    i = y as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        let state = state_buf[i as usize];
                        if (state & 3) != 0
                            || (state >> 2) != (x - xb + (i - y as hts_pos_t)) as i32
                        {
                            bq[i as usize] = 0;
                        } else {
                            bq[i as usize] = if bq[i as usize] < q[i as usize] {
                                bq[i as usize]
                            } else {
                                q[i as usize]
                            };
                        }
                        i += 1;
                    }
                    x += l as hts_pos_t;
                    y += l;
                } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
                    // Need sanity check here too.
                    if l > l_qseq - y {
                        l = l_qseq - y;
                    }
                    y += l;
                } else if op == BAM_CDEL {
                    x += l as hts_pos_t;
                }
                k += 1;
            }
            i = 0;
            while i < l_qseq as hts_pos_t {
                bq[i as usize] = ((qual[i as usize] as i32) - (bq[i as usize] as i32) + 64) as u8; // finalize BQ
                i += 1;
            }
        } else {
            // in this block, bq[] is BAQ that can be larger than qual[] (different from the above!)
            // tseq,tref are no longer needed, so we can steal them to avoid mallocs
            let left = tseq;
            let rght = tref;
            let mut len: i32 = 0;

            k = 0;
            x = b.core.pos;
            y = 0;
            while k < b.core.n_cigar as i32 {
                let op = (cigar[k as usize] & 0xf) as i32;
                let mut l = (cigar[k as usize] >> 4) as i32;

                // concatenate alignment matches (including sequence (mis)matches)
                // otherwise 50M50M gives a different result to 100M
                if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
                    if k + 1 < b.core.n_cigar as i32 {
                        let next_op = bam_cigar_op(cigar[k as usize + 1]);

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
                    if l > l_qseq - y {
                        l = l_qseq - y;
                    }
                    i = y as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        let state = state_buf[i as usize];
                        bq[i as usize] = if (state & 3) != 0
                            || (state >> 2) != (x - xb + (i - y as hts_pos_t)) as i32
                        {
                            0
                        } else {
                            q[i as usize]
                        };
                        i += 1;
                    }
                    left[y as usize] = bq[y as usize];
                    i = (y + 1) as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        left[i as usize] = if bq[i as usize] > left[i as usize - 1] {
                            bq[i as usize]
                        } else {
                            left[i as usize - 1]
                        };
                        i += 1;
                    }
                    rght[(y + l - 1) as usize] = bq[(y + l - 1) as usize];
                    if l > 1 {
                        i = (y + l - 2) as hts_pos_t;
                        loop {
                            rght[i as usize] = if bq[i as usize] > rght[i as usize + 1] {
                                bq[i as usize]
                            } else {
                                rght[i as usize + 1]
                            };
                            if i == y as hts_pos_t {
                                break;
                            }
                            i -= 1;
                        }
                    }
                    i = y as hts_pos_t;
                    while i < (y + l) as hts_pos_t {
                        bq[i as usize] = if left[i as usize] < rght[i as usize] {
                            left[i as usize]
                        } else {
                            rght[i as usize]
                        };
                        i += 1;
                    }
                    x += l as hts_pos_t;
                    y += l;
                } else if op == BAM_CSOFT_CLIP || op == BAM_CINS {
                    // Need sanity check here too.
                    if l > l_qseq - y {
                        l = l_qseq - y;
                    }
                    y += l;
                } else if op == BAM_CDEL {
                    x += l as hts_pos_t;
                }
                k += 1;
            }
            i = 0;
            while i < l_qseq as hts_pos_t {
                bq[i as usize] = (64
                    + if qual[i as usize] <= bq[i as usize] {
                        0
                    } else {
                        (qual[i as usize] as i32) - (bq[i as usize] as i32)
                    }) as u8; // finalize BQ
                i += 1;
            }
        }
        if apply_baq != 0 {
            i = 0;
            while i < l_qseq as hts_pos_t {
                qual[i as usize] =
                    ((qual[i as usize] as i32) - ((bq[i as usize] as i32) - 64)) as u8; // modify qual
                i += 1;
            }
            bam_aux_append_ref(b, b"ZQ", b'Z' as _, l_qseq + 1, bq);
        } else {
            bam_aux_append_ref(b, b"BQ", b'Z' as _, l_qseq + 1, bq);
        }
    }

    0
}

pub unsafe fn sam_cap_mapq(b: &mut bam1_t, ref_: &[u8], thres: i32) -> i32 {
    sam_cap_mapq_impl(b, ref_, thres)
}

unsafe fn sam_cap_mapq_impl(b: &mut bam1_t, ref_: &[u8], mut thres: i32) -> i32 {
    let seq = bam_seq_slice(b);
    let qual = bam_qual_slice(b);
    let cigar = bam_cigar_slice(b);
    let mut mm = 0;
    let mut q = 0;
    let mut len = 0;
    let mut clip_l = 0;
    let mut clip_q = 0;

    if thres < 0 {
        thres = 40;
    }

    let mut y = 0;
    let mut x = b.core.pos;
    for &cigar_i in cigar {
        let l = (cigar_i >> 4) as i32;
        let op = (cigar_i & 0x0f) as i32;
        if op == BAM_CMATCH || op == BAM_CEQUAL || op == BAM_CDIFF {
            let mut j = 0;
            while j < l {
                let z = y + j;
                let Some(&base) = ref_.get((x + j as hts_pos_t) as usize) else {
                    break;
                };
                if base == 0 {
                    break;
                }
                let c1 = bam_seqi(seq.as_ptr(), z as usize) as i32;
                let c2 = SEQ_NT16_TABLE[base as usize] as i32;
                if c2 != 15 && c1 != 15 && qual[z as usize] >= 13 {
                    len += 1;
                    if c1 != 0 && c1 != c2 && qual[z as usize] >= 13 {
                        mm += 1;
                        q += if qual[z as usize] > 33 {
                            33
                        } else {
                            qual[z as usize] as i32
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
                let Some(&base) = ref_.get((x + j as hts_pos_t) as usize) else {
                    break;
                };
                if base == 0 {
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
                clip_q += qual[(y + j) as usize] as i32;
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
    (t + 0.499) as i32
}
