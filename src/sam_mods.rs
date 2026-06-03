// Functions translated from htslib/sam_mods.c (base-modification API).
// Extracted from src/sam.rs.

use std::ffi::{c_char, c_int};

use crate::htslib_rs::hts::hts_str2uint;
use crate::htslib_rs::sam::*;

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

pub unsafe fn hts_base_mod_state_alloc() -> *mut hts_base_mod_state {
    crate::htslib_rs::c_compat::calloc(1, std::mem::size_of::<hts_base_mod_state>() as u64).cast()
}

pub unsafe fn hts_base_mod_state_free(state: *mut hts_base_mod_state) {
    crate::htslib_rs::c_compat::free(state.cast());
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
    loop {
        if (*state).seq_pos > qpos {
            break;
        }
        r = bam_mods_at_next_pos(b, state, mods, n_mods);
        if r < 0 {
            break;
        }
    }
    r
}
