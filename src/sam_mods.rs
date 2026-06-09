// Functions translated from htslib/sam_mods.c (base-modification API).
// Extracted from src/sam.rs.

use std::ffi::{c_char, c_int};
use std::ptr::NonNull;

use crate::htslib_rs::hts::hts_str2uint;
use crate::htslib_rs::sam::*;

struct ModQueryOutputs<'a> {
    strand: Option<&'a mut c_int>,
    implicit: Option<&'a mut c_int>,
    canonical: Option<&'a mut c_char>,
}

struct AuxBytes<'a> {
    bytes: &'a [u8],
    ptr: NonNull<u8>,
}

struct MlData<'a> {
    bytes: &'a [u8],
    ptr: NonNull<u8>,
}

impl ModQueryOutputs<'_> {
    fn write(self, strand: c_int, implicit: c_int, canonical: c_char) {
        if let Some(out) = self.strand {
            *out = strand;
        }
        if let Some(out) = self.implicit {
            *out = implicit;
        }
        if let Some(out) = self.canonical {
            *out = canonical;
        }
    }
}

impl Default for hts_base_mod_state {
    fn default() -> Self {
        Self {
            type_: [0; MAX_BASE_MOD],
            canonical: [0; MAX_BASE_MOD],
            strand: [0; MAX_BASE_MOD],
            mmcount: [0; MAX_BASE_MOD],
            mm: [std::ptr::null_mut(); MAX_BASE_MOD],
            mmend: [std::ptr::null_mut(); MAX_BASE_MOD],
            ml: [std::ptr::null_mut(); MAX_BASE_MOD],
            mlstride: [0; MAX_BASE_MOD],
            implicit: [0; MAX_BASE_MOD],
            seq_pos: 0,
            nmods: 0,
            flags: 0,
        }
    }
}

fn bam_is_reverse(b: &bam1_t) -> bool {
    (b.core.flag as c_int & BAM_FREVERSE) != 0
}

unsafe fn bam_aux_get_ref(b: &bam1_t, tag: &[u8]) -> Option<NonNull<u8>> {
    if tag.contains(&0) {
        return None;
    }

    let mut nul_tag = Vec::with_capacity(tag.len() + 1);
    nul_tag.extend_from_slice(tag);
    nul_tag.push(0);

    NonNull::new(bam_aux_get(b as *const bam1_t, nul_tag.as_ptr().cast()))
}

unsafe fn aux_z_bytes<'a>(b: &'a bam1_t, tags: &[&[u8]]) -> Result<Option<AuxBytes<'a>>, c_int> {
    let Some(aux) = tags.iter().find_map(|tag| bam_aux_get_ref(b, tag)) else {
        return Ok(None);
    };
    let ptr = aux.as_ptr();
    if *ptr != b'Z' {
        return Err(-1);
    }

    let data = ptr.add(1);
    let mut len = 0usize;
    while *data.add(len) != 0 {
        len += 1;
    }

    Ok(Some(AuxBytes {
        bytes: std::slice::from_raw_parts(data, len),
        ptr: NonNull::new_unchecked(data),
    }))
}

unsafe fn aux_ml_bytes<'a>(b: &'a bam1_t) -> Result<Option<MlData<'a>>, c_int> {
    let Some(aux) = bam_aux_get_ref(b, b"ML").or_else(|| bam_aux_get_ref(b, b"Ml")) else {
        return Ok(None);
    };
    let ptr = aux.as_ptr();
    if *ptr != b'B' || *ptr.add(1) != b'C' {
        return Err(-1);
    }
    let len = u32::from_le_bytes([*ptr.add(2), *ptr.add(3), *ptr.add(4), *ptr.add(5)]) as usize;
    let data = ptr.add(6);
    Ok(Some(MlData {
        bytes: std::slice::from_raw_parts(data, len),
        ptr: NonNull::new_unchecked(data),
    }))
}

unsafe fn bam_seq_base(b: &bam1_t, qpos: c_int) -> c_int {
    bam_seqi(bam_get_seq(b as *const bam1_t), qpos as usize) as c_int
}

unsafe fn mods_slice_from_raw<'a>(
    mods: *mut hts_base_mod,
    n_mods: c_int,
) -> &'a mut [hts_base_mod] {
    if n_mods <= 0 || mods.is_null() {
        &mut []
    } else {
        std::slice::from_raw_parts_mut(mods, n_mods as usize)
    }
}

fn write_base_mod(out: &mut hts_base_mod, state: &hts_base_mod_state, idx: usize, qual: c_int) {
    out.modified_base = state.type_[idx];
    out.canonical_base = SEQ_NT16_STR[state.canonical[idx] as usize] as c_int;
    out.strand = state.strand[idx] as c_int;
    out.qual = qual;
}

fn parse_u31(bytes: &[u8], start: usize) -> Option<(c_int, usize)> {
    let mut pos = start;
    let mut value = 0i64;
    while let Some(&digit) = bytes.get(pos) {
        if !digit.is_ascii_digit() {
            break;
        }
        value = value.checked_mul(10)?.checked_add((digit - b'0') as i64)?;
        if value > c_int::MAX as i64 {
            return None;
        }
        pos += 1;
    }

    (pos != start).then_some((value as c_int, pos))
}

pub unsafe fn seq_freq_ref(b: &bam1_t, freq: &mut [c_int; 16]) {
    freq.fill(0);
    let mut i = 0;
    while i < b.core.l_qseq {
        freq[bam_seq_base(b, i) as usize] += 1;
        i += 1;
    }
    freq[15] = b.core.l_qseq;
}

pub unsafe fn seq_freq(b: *const bam1_t, freq: *mut c_int) {
    let (Some(b), Some(freq)) = (b.as_ref(), freq.as_mut()) else {
        return;
    };
    let freq = std::slice::from_raw_parts_mut(freq, 16);
    let freq: &mut [c_int; 16] = freq.try_into().expect("fixed seq_freq output length");
    seq_freq_ref(b, freq);
}

pub fn hts_base_mod_state_new() -> Box<hts_base_mod_state> {
    Box::new(hts_base_mod_state::default())
}

pub unsafe fn hts_base_mod_state_alloc() -> *mut hts_base_mod_state {
    Box::into_raw(hts_base_mod_state_new())
}

pub unsafe fn hts_base_mod_state_free(state: *mut hts_base_mod_state) {
    if !state.is_null() {
        drop(Box::from_raw(state));
    }
}

pub fn bam_mods_recorded_ref<'a>(
    state: &'a mut hts_base_mod_state,
    ntype: &mut c_int,
) -> &'a mut [c_int] {
    *ntype = state.nmods;
    &mut state.type_[..state.nmods as usize]
}

pub unsafe fn bam_mods_recorded(state: *mut hts_base_mod_state, ntype: *mut c_int) -> *mut c_int {
    let (Some(state), Some(ntype)) = (state.as_mut(), ntype.as_mut()) else {
        return std::ptr::null_mut();
    };
    bam_mods_recorded_ref(state, ntype).as_mut_ptr()
}

fn bam_mods_query_type_ref(
    state: &hts_base_mod_state,
    code: c_int,
) -> Option<(c_int, c_int, c_char)> {
    let mut i = 0;
    while i < state.nmods {
        if state.type_[i as usize] == code {
            break;
        }
        i += 1;
    }
    if i == state.nmods {
        return None;
    }

    let canonical = b"?AC?G???T??????N"[state.canonical[i as usize] as usize] as c_char;
    Some((
        state.strand[i as usize] as c_int,
        state.implicit[i as usize],
        canonical,
    ))
}

fn bam_mods_query_type_write(
    state: &hts_base_mod_state,
    code: c_int,
    outputs: ModQueryOutputs<'_>,
) -> c_int {
    let Some((strand_value, implicit_value, canonical_value)) =
        bam_mods_query_type_ref(state, code)
    else {
        return -1;
    };

    outputs.write(strand_value, implicit_value, canonical_value);
    0
}

pub fn bam_mods_query_type_ref_outputs(
    state: &hts_base_mod_state,
    code: c_int,
    strand: Option<&mut c_int>,
    implicit: Option<&mut c_int>,
    canonical: Option<&mut c_char>,
) -> c_int {
    bam_mods_query_type_write(
        state,
        code,
        ModQueryOutputs {
            strand,
            implicit,
            canonical,
        },
    )
}

pub unsafe fn bam_mods_query_type(
    state: *mut hts_base_mod_state,
    code: c_int,
    strand: *mut c_int,
    implicit: *mut c_int,
    canonical: *mut c_char,
) -> c_int {
    let Some(state) = state.as_ref() else {
        return -1;
    };
    bam_mods_query_type_write(
        state,
        code,
        ModQueryOutputs {
            strand: strand.as_mut(),
            implicit: implicit.as_mut(),
            canonical: canonical.as_mut(),
        },
    )
}

fn bam_mods_queryi_ref(state: &hts_base_mod_state, i: c_int) -> Option<(c_int, c_int, c_char)> {
    if i < 0 || i >= state.nmods {
        return None;
    }

    let canonical = b"?AC?G???T??????N"[state.canonical[i as usize] as usize] as c_char;
    Some((
        state.strand[i as usize] as c_int,
        state.implicit[i as usize],
        canonical,
    ))
}

fn bam_mods_queryi_write(
    state: &hts_base_mod_state,
    i: c_int,
    outputs: ModQueryOutputs<'_>,
) -> c_int {
    let Some((strand_value, implicit_value, canonical_value)) = bam_mods_queryi_ref(state, i)
    else {
        return -1;
    };

    outputs.write(strand_value, implicit_value, canonical_value);
    0
}

pub fn bam_mods_queryi_ref_outputs(
    state: &hts_base_mod_state,
    i: c_int,
    strand: Option<&mut c_int>,
    implicit: Option<&mut c_int>,
    canonical: Option<&mut c_char>,
) -> c_int {
    bam_mods_queryi_write(
        state,
        i,
        ModQueryOutputs {
            strand,
            implicit,
            canonical,
        },
    )
}

pub unsafe fn bam_mods_queryi(
    state: *mut hts_base_mod_state,
    i: c_int,
    strand: *mut c_int,
    implicit: *mut c_int,
    canonical: *mut c_char,
) -> c_int {
    let Some(state) = state.as_ref() else {
        return -1;
    };
    bam_mods_queryi_write(
        state,
        i,
        ModQueryOutputs {
            strand: strand.as_mut(),
            implicit: implicit.as_mut(),
            canonical: canonical.as_mut(),
        },
    )
}

pub unsafe fn bam_parse_basemod2_ref(
    b: &bam1_t,
    state: &mut hts_base_mod_state,
    flags: u32,
) -> c_int {
    state.seq_pos = 0;
    state.nmods = 0;
    state.flags = flags;

    let mm = match aux_z_bytes(b, &[b"MM", b"Mm"]) {
        Ok(Some(mm)) => mm,
        Ok(None) => return 0,
        Err(ret) => return ret,
    };

    if let Some(mi) = bam_aux_get_ref(b, b"MN") {
        if bam_aux2i(mi.as_ptr()) != b.core.l_qseq as i64 && b.core.l_qseq != 0 {
            return -1;
        }
    }

    let ml = match aux_ml_bytes(b) {
        Ok(ml) => ml,
        Err(ret) => return ret,
    };
    let mut ml_pos = 0usize;

    let mut freq = [0; 16];
    seq_freq_ref(b, &mut freq);

    let mm_bytes = mm.bytes;
    let mm_ptr = mm.ptr.as_ptr();
    let mut cp = 0usize;
    let mut mod_num = 0usize;
    while cp < mm_bytes.len() {
        let mut btype = mm_bytes[cp];
        cp += 1;
        if !matches!(btype, b'A' | b'C' | b'G' | b'T' | b'U' | b'N') {
            return -1;
        }
        if btype == b'U' {
            btype = b'T';
        }
        let btype = SEQ_NT16_TABLE[btype as usize] as c_int;

        let Some(&strand) = mm_bytes.get(cp) else {
            return -1;
        };
        if strand != b'+' && strand != b'-' {
            return -1;
        }
        cp += 1;

        let mut ms = cp;
        let mut chebi = 0;
        if mm_bytes.get(cp).is_some_and(u8::is_ascii_digit) {
            let Some((value, end)) = parse_u31(mm_bytes, cp) else {
                return -1;
            };
            chebi = value;
            cp = end;
            ms = cp - 1;
        } else {
            while mm_bytes.get(cp).is_some_and(u8::is_ascii_alphabetic) {
                cp += 1;
            }
            if cp == mm_bytes.len() {
                return -1;
            }
        }
        let me = cp;

        let Some(&separator) = mm_bytes.get(cp) else {
            return -1;
        };
        let implicit = if separator == b'.' {
            cp += 1;
            1
        } else if separator == b'?' {
            cp += 1;
            0
        } else if separator == b',' || separator == b';' {
            1
        } else {
            return -1;
        };

        let mut cp_end = None;
        let mut n = 0usize;
        let stride = (me - ms) as c_int;
        let mut ndelta = 0usize;
        let delta: c_int;

        if bam_is_reverse(b) {
            let mut total_seq = 0i64;
            loop {
                if mm_bytes.get(cp) == Some(&b',') {
                    cp += 1;
                }
                if cp == mm_bytes.len() || mm_bytes[cp] == b';' {
                    break;
                }
                let Some((d, end)) = parse_u31(mm_bytes, cp) else {
                    return -1;
                };
                cp = end;
                cp_end = Some(end);
                total_seq += i64::from(d) + 1;
                ndelta += 1;
            }
            delta = freq[SEQI_RC[btype as usize] as usize] - total_seq as c_int;
        } else if mm_bytes.get(cp) == Some(&b',') {
            let Some((value, end)) = parse_u31(mm_bytes, cp + 1) else {
                return -1;
            };
            delta = value;
            cp_end = Some(end);
        } else {
            delta = c_int::MAX;
            cp_end = Some(cp);
        }

        for ms_iter in ms..me {
            state.type_[mod_num] = if chebi != 0 {
                -chebi
            } else {
                mm_bytes[ms_iter] as c_int
            };
            state.strand[mod_num] = (strand == b'-') as c_char;
            state.canonical[mod_num] = btype;
            state.mlstride[mod_num] = stride;
            state.implicit[mod_num] = implicit;
            if delta < 0 {
                return -1;
            }
            state.mmcount[mod_num] = delta;
            if bam_is_reverse(b) {
                state.mm[mod_num] = mm_ptr.add(me + 1).cast::<c_char>();
                state.mmend[mod_num] = mm_ptr.add(cp_end.unwrap_or(cp)).cast::<c_char>();
                state.ml[mod_num] = if let Some(ml) = &ml {
                    ml.ptr
                        .as_ptr()
                        .add(ml_pos + n)
                        .wrapping_offset((ndelta as isize - 1) * stride as isize)
                } else {
                    std::ptr::null_mut()
                };
            } else {
                state.mm[mod_num] = mm_ptr.add(cp_end.unwrap_or(cp)).cast::<c_char>();
                state.mmend[mod_num] = std::ptr::null_mut();
                state.ml[mod_num] = if let Some(ml) = &ml {
                    ml.ptr.as_ptr().add(ml_pos + n)
                } else {
                    std::ptr::null_mut()
                };
            }
            mod_num += 1;
            if mod_num >= MAX_BASE_MOD {
                return -1;
            }
            n += 1;
        }

        if let Some(ml) = &ml {
            if bam_is_reverse(b) {
                ml_pos += ndelta * stride as usize;
            } else {
                while cp < mm_bytes.len() && mm_bytes[cp] != b';' {
                    if mm_bytes[cp] == b',' {
                        ml_pos += stride as usize;
                    }
                    cp += 1;
                }
            }
            if ml_pos > ml.bytes.len() {
                return -1;
            }
        } else if let Some(end) = cp_end.filter(|_| bam_is_reverse(b)) {
            cp = end;
        } else {
            while cp < mm_bytes.len() && mm_bytes[cp] != b';' {
                cp += 1;
            }
        }

        if cp == mm_bytes.len() {
            return -1;
        }
        cp += 1;
    }

    if ml.as_ref().is_some_and(|ml| ml_pos != ml.bytes.len()) {
        return -1;
    }
    state.nmods = mod_num as c_int;
    0
}

pub unsafe fn bam_parse_basemod2(
    b: *const bam1_t,
    state: *mut hts_base_mod_state,
    flags: u32,
) -> c_int {
    let (Some(b), Some(state)) = (b.as_ref(), state.as_mut()) else {
        return -1;
    };
    bam_parse_basemod2_ref(b, state, flags)
}

pub unsafe fn bam_parse_basemod(b: *const bam1_t, state: *mut hts_base_mod_state) -> c_int {
    let (Some(b), Some(state)) = (b.as_ref(), state.as_mut()) else {
        return -1;
    };
    bam_parse_basemod_ref(b, state)
}

pub unsafe fn bam_parse_basemod_ref(b: &bam1_t, state: &mut hts_base_mod_state) -> c_int {
    bam_parse_basemod2_ref(b, state, 0)
}

pub unsafe fn bam_mods_at_next_pos_ref(
    b: &bam1_t,
    state: &mut hts_base_mod_state,
    mods: &mut [hts_base_mod],
) -> c_int {
    if state.seq_pos >= b.core.l_qseq {
        return 0;
    }

    let mut n = 0;
    let mut base = bam_seq_base(b, state.seq_pos);
    state.seq_pos += 1;
    if bam_is_reverse(b) {
        base = SEQI_RC[base as usize];
    }

    let mut i = 0;
    while i < state.nmods {
        let idx = i as usize;
        let mut unchecked = 0;
        if state.canonical[idx] != base && state.canonical[idx] != 15 {
            i += 1;
            continue;
        }

        if state.mmcount[idx] > 0 {
            state.mmcount[idx] -= 1;
            if state.implicit[idx] == 0 && (state.flags & HTS_MOD_REPORT_UNCHECKED) != 0 {
                unchecked = 1;
            } else {
                i += 1;
                continue;
            }
        } else {
            state.mmcount[idx] -= 1;
        }

        let mmptr = state.mm[idx];
        if let Some(out) = mods.get_mut(n as usize) {
            let qual = if unchecked != 0 {
                HTS_MOD_UNCHECKED
            } else if !state.ml[idx].is_null() {
                *state.ml[idx] as c_int
            } else {
                HTS_MOD_UNKNOWN
            };
            write_base_mod(out, state, idx, qual);
        }
        n += 1;

        if unchecked != 0 {
            i += 1;
            continue;
        }

        if !state.ml[idx].is_null() {
            if bam_is_reverse(b) {
                state.ml[idx] = state.ml[idx].sub(state.mlstride[idx] as usize);
            } else {
                state.ml[idx] = state.ml[idx].add(state.mlstride[idx] as usize);
            }
        }

        let mut failed = 0;
        if bam_is_reverse(b) {
            if state.mmend[idx].sub(1) < state.mm[idx] {
                return -1;
            }
            let mut cp = state.mmend[idx].sub(1);
            while cp != state.mm[idx] {
                if *cp == b',' as c_char {
                    break;
                }
                cp = cp.sub(1);
            }
            state.mmend[idx] = cp;
            if cp != state.mm[idx] {
                let mut tmp = std::ptr::null_mut();
                state.mmcount[idx] = hts_str2uint(cp.add(1), &mut tmp, 31, &mut failed) as c_int;
            } else {
                state.mmcount[idx] = c_int::MAX;
            }
        } else if *state.mm[idx] == b',' as c_char {
            let mut next = state.mm[idx];
            state.mmcount[idx] = hts_str2uint(next.add(1), &mut next, 31, &mut failed) as c_int;
            state.mm[idx] = next;
        } else {
            state.mmcount[idx] = c_int::MAX;
        }
        if failed != 0 {
            return -1;
        }

        let mut j = i + 1;
        while j < state.nmods && state.mm[j as usize] == mmptr {
            let jidx = j as usize;
            if let Some(out) = mods.get_mut(n as usize) {
                let qual = if !state.ml[jidx].is_null() {
                    *state.ml[jidx] as c_int
                } else {
                    -1
                };
                write_base_mod(out, state, jidx, qual);
            }
            n += 1;
            state.mmcount[jidx] = state.mmcount[idx];
            state.mm[jidx] = state.mm[idx];
            if !state.ml[jidx].is_null() {
                if bam_is_reverse(b) {
                    state.ml[jidx] = state.ml[jidx].sub(state.mlstride[jidx] as usize);
                } else {
                    state.ml[jidx] = state.ml[jidx].add(state.mlstride[jidx] as usize);
                }
            }
            j += 1;
        }
        i = j;
    }

    n
}

pub unsafe fn bam_mods_at_next_pos(
    b: *const bam1_t,
    state: *mut hts_base_mod_state,
    mods: *mut hts_base_mod,
    n_mods: c_int,
) -> c_int {
    let (Some(b), Some(state)) = (b.as_ref(), state.as_mut()) else {
        return -1;
    };
    let mods = mods_slice_from_raw(mods, n_mods);
    bam_mods_at_next_pos_ref(b, state, mods)
}

pub unsafe fn bam_next_basemod_ref(
    b: &bam1_t,
    state: &mut hts_base_mod_state,
    mods: &mut [hts_base_mod],
    pos: &mut c_int,
) -> c_int {
    let mut next = [0x7f7f7f7f; 16];
    let mut freq = [0; 16];
    let unchecked = state.flags & HTS_MOD_REPORT_UNCHECKED;

    for i in 0..state.nmods {
        let mut base = state.canonical[i as usize];
        if bam_is_reverse(b) {
            base = SEQI_RC[base as usize];
        }
        if unchecked != 0 && state.implicit[i as usize] == 0 {
            next[base as usize] = 0;
        } else if next[base as usize] > state.mmcount[i as usize] {
            next[base as usize] = state.mmcount[i as usize];
        }
    }

    let mut i = state.seq_pos;
    while i < b.core.l_qseq {
        let bc = bam_seq_base(b, i) as usize;
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
    state.seq_pos = i;

    if bam_is_reverse(b) {
        for j in 0..state.nmods {
            state.mmcount[j as usize] -=
                freq[SEQI_RC[state.canonical[j as usize] as usize] as usize];
        }
    } else {
        for j in 0..state.nmods {
            state.mmcount[j as usize] -= freq[state.canonical[j as usize] as usize];
        }
    }

    if b.core.l_qseq != 0 && state.seq_pos >= b.core.l_qseq {
        if !bam_is_reverse(b) {
            for j in 0..state.nmods {
                let idx = j as usize;
                if state.mmcount[idx] < 0x7f000000
                    || (*state.mm[idx] != 0 && *state.mm[idx] != b';' as c_char)
                {
                    return -1;
                }
            }
        }
        return 0;
    }

    let r = bam_mods_at_next_pos_ref(b, state, mods);
    if r > 0 {
        r
    } else {
        0
    }
}

pub unsafe fn bam_next_basemod(
    b: *const bam1_t,
    state: *mut hts_base_mod_state,
    mods: *mut hts_base_mod,
    n_mods: c_int,
    pos: *mut c_int,
) -> c_int {
    let (Some(b), Some(state), Some(pos)) = (b.as_ref(), state.as_mut(), pos.as_mut()) else {
        return -1;
    };
    let mods = mods_slice_from_raw(mods, n_mods);
    bam_next_basemod_ref(b, state, mods, pos)
}

pub unsafe fn bam_mods_at_qpos_ref(
    b: &bam1_t,
    qpos: c_int,
    state: &mut hts_base_mod_state,
    mods: &mut [hts_base_mod],
) -> c_int {
    let mut r = 0;
    loop {
        if state.seq_pos > qpos {
            break;
        }
        r = bam_mods_at_next_pos_ref(b, state, mods);
        if r < 0 {
            break;
        }
    }
    r
}

pub unsafe fn bam_mods_at_qpos(
    b: *const bam1_t,
    qpos: c_int,
    state: *mut hts_base_mod_state,
    mods: *mut hts_base_mod,
    n_mods: c_int,
) -> c_int {
    let (Some(b), Some(state)) = (b.as_ref(), state.as_mut()) else {
        return -1;
    };
    let mods = mods_slice_from_raw(mods, n_mods);
    bam_mods_at_qpos_ref(b, qpos, state, mods)
}
