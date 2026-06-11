// Functions translated from htslib/sam_mods.c (base-modification API).
// Extracted from src/sam.rs.

use crate::htslib_rs::hts::hts_str2uint;
use crate::htslib_rs::sam::*;

struct ModQueryOutputs<'a> {
    strand: Option<&'a mut i32>,
    implicit: Option<&'a mut i32>,
    canonical: Option<&'a mut u8>,
}

impl ModQueryOutputs<'_> {
    fn write(self, strand: i32, implicit: i32, canonical: u8) {
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
    (b.core.flag as i32 & BAM_FREVERSE) != 0
}

// Returns the aux value as a byte slice that runs from the located tag's value
// byte to the end of the bam record's data block. The caller inspects the type
// byte and slices out the payload.
unsafe fn bam_aux_get_slice<'a>(b: &'a bam1_t, tag: &[u8]) -> Option<&'a [u8]> {
    if tag.contains(&0) {
        return None;
    }

    let mut nul_tag = Vec::with_capacity(tag.len() + 1);
    nul_tag.extend_from_slice(tag);
    nul_tag.push(0);

    let aux = bam_aux_get(b as *const bam1_t, nul_tag.as_ptr().cast());
    if aux.is_null() {
        return None;
    }
    let data = &b.data[..];
    let offset = aux.offset_from(b.data.as_ptr()) as usize;
    Some(&data[offset..])
}

// The Z-string payload, NUL terminator removed.
unsafe fn aux_z_bytes<'a>(b: &'a bam1_t, tags: &[&[u8]]) -> Result<Option<&'a [u8]>, i32> {
    let Some(aux) = tags.iter().find_map(|tag| bam_aux_get_slice(b, tag)) else {
        return Ok(None);
    };
    if aux.first() != Some(&b'Z') {
        return Err(-1);
    }

    let payload = &aux[1..];
    let len = payload.iter().position(|&c| c == 0).unwrap_or(payload.len());
    Ok(Some(&payload[..len]))
}

// The ML/Ml B,C array payload as a byte slice.
unsafe fn aux_ml_bytes<'a>(b: &'a bam1_t) -> Result<Option<&'a [u8]>, i32> {
    let Some(aux) = bam_aux_get_slice(b, b"ML").or_else(|| bam_aux_get_slice(b, b"Ml")) else {
        return Ok(None);
    };
    if aux.first() != Some(&b'B') || aux.get(1) != Some(&b'C') {
        return Err(-1);
    }
    let len = u32::from_le_bytes([aux[2], aux[3], aux[4], aux[5]]) as usize;
    Ok(Some(&aux[6..6 + len]))
}

unsafe fn bam_seq_base(b: &bam1_t, qpos: i32) -> i32 {
    bam_seqi(bam_get_seq(b as *const bam1_t), qpos as usize) as i32
}

fn write_base_mod(out: &mut hts_base_mod, state: &hts_base_mod_state, idx: usize, qual: i32) {
    out.modified_base = state.type_[idx];
    out.canonical_base = SEQ_NT16_STR[state.canonical[idx] as usize] as i32;
    out.strand = state.strand[idx] as i32;
    out.qual = qual;
}

fn parse_u31(bytes: &[u8], start: usize) -> Option<(i32, usize)> {
    let mut pos = start;
    let mut value = 0i64;
    while let Some(&digit) = bytes.get(pos) {
        if !digit.is_ascii_digit() {
            break;
        }
        value = value.checked_mul(10)?.checked_add((digit - b'0') as i64)?;
        if value > i32::MAX as i64 {
            return None;
        }
        pos += 1;
    }

    (pos != start).then_some((value as i32, pos))
}

pub unsafe fn seq_freq(b: &bam1_t, freq: &mut [i32; 16]) {
    freq.fill(0);
    let mut i = 0;
    while i < b.core.l_qseq {
        freq[bam_seq_base(b, i) as usize] += 1;
        i += 1;
    }
    freq[15] = b.core.l_qseq;
}

pub fn hts_base_mod_state_new() -> Box<hts_base_mod_state> {
    Box::new(hts_base_mod_state::default())
}

pub fn hts_base_mod_state_alloc() -> Box<hts_base_mod_state> {
    hts_base_mod_state_new()
}

pub fn hts_base_mod_state_free(state: Option<Box<hts_base_mod_state>>) {
    drop(state);
}

pub fn bam_mods_recorded<'a>(state: &'a mut hts_base_mod_state, ntype: &mut i32) -> &'a mut [i32] {
    *ntype = state.nmods;
    &mut state.type_[..state.nmods as usize]
}

fn bam_mods_query_type_lookup(
    state: &hts_base_mod_state,
    code: i32,
) -> Option<(i32, i32, u8)> {
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

    let canonical = b"?AC?G???T??????N"[state.canonical[i as usize] as usize];
    Some((
        state.strand[i as usize] as i32,
        state.implicit[i as usize],
        canonical,
    ))
}

fn bam_mods_query_type_write(
    state: &hts_base_mod_state,
    code: i32,
    outputs: ModQueryOutputs<'_>,
) -> i32 {
    let Some((strand_value, implicit_value, canonical_value)) =
        bam_mods_query_type_lookup(state, code)
    else {
        return -1;
    };

    outputs.write(strand_value, implicit_value, canonical_value);
    0
}

pub fn bam_mods_query_type(
    state: &hts_base_mod_state,
    code: i32,
    strand: Option<&mut i32>,
    implicit: Option<&mut i32>,
    canonical: Option<&mut u8>,
) -> i32 {
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

fn bam_mods_queryi_lookup(state: &hts_base_mod_state, i: i32) -> Option<(i32, i32, u8)> {
    if i < 0 || i >= state.nmods {
        return None;
    }

    let canonical = b"?AC?G???T??????N"[state.canonical[i as usize] as usize];
    Some((
        state.strand[i as usize] as i32,
        state.implicit[i as usize],
        canonical,
    ))
}

fn bam_mods_queryi_write(
    state: &hts_base_mod_state,
    i: i32,
    outputs: ModQueryOutputs<'_>,
) -> i32 {
    let Some((strand_value, implicit_value, canonical_value)) = bam_mods_queryi_lookup(state, i)
    else {
        return -1;
    };

    outputs.write(strand_value, implicit_value, canonical_value);
    0
}

pub fn bam_mods_queryi(
    state: &hts_base_mod_state,
    i: i32,
    strand: Option<&mut i32>,
    implicit: Option<&mut i32>,
    canonical: Option<&mut u8>,
) -> i32 {
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

pub unsafe fn bam_parse_basemod2(b: &bam1_t, state: &mut hts_base_mod_state, flags: u32) -> i32 {
    state.seq_pos = 0;
    state.nmods = 0;
    state.flags = flags;

    let mm = match aux_z_bytes(b, &[b"MM", b"Mm"]) {
        Ok(Some(mm)) => mm,
        Ok(None) => return 0,
        Err(ret) => return ret,
    };
    // Base pointer into `b.data` (the start of the buffer), used to convert the
    // byte offsets computed below into the raw `*mut u8` pointers stored in
    // `state.mm`/`mmend`/`ml`.
    let data_ptr = b.data.as_ptr() as *mut u8;
    // Offset of the MM payload's first byte within `b.data`.
    let mm_base = mm.as_ptr().offset_from(b.data.as_ptr()) as usize;

    if let Some(mi) = bam_aux_get_slice(b, b"MN") {
        if bam_aux2i(mi.as_ptr()) != b.core.l_qseq as i64 && b.core.l_qseq != 0 {
            return -1;
        }
    }

    let ml = match aux_ml_bytes(b) {
        Ok(ml) => ml,
        Err(ret) => return ret,
    };
    // Offset of the ML payload's first byte within `b.data`.
    let ml_base = ml.map(|ml| ml.as_ptr().offset_from(b.data.as_ptr()) as usize);
    let mut ml_pos = 0usize;

    let mut freq = [0; 16];
    seq_freq(b, &mut freq);

    let mm_bytes = mm;
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
        let btype = SEQ_NT16_TABLE[btype as usize] as i32;

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
        let stride = (me - ms) as i32;
        let mut ndelta = 0usize;
        let delta: i32;

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
            delta = freq[SEQI_RC[btype as usize] as usize] - total_seq as i32;
        } else if mm_bytes.get(cp) == Some(&b',') {
            let Some((value, end)) = parse_u31(mm_bytes, cp + 1) else {
                return -1;
            };
            delta = value;
            cp_end = Some(end);
        } else {
            delta = i32::MAX;
            cp_end = Some(cp);
        }

        for ms_iter in ms..me {
            state.type_[mod_num] = if chebi != 0 {
                -chebi
            } else {
                mm_bytes[ms_iter] as i32
            };
            state.strand[mod_num] = (strand == b'-') as u8;
            state.canonical[mod_num] = btype;
            state.mlstride[mod_num] = stride;
            state.implicit[mod_num] = implicit;
            if delta < 0 {
                return -1;
            }
            state.mmcount[mod_num] = delta;
            if bam_is_reverse(b) {
                state.mm[mod_num] = data_ptr.add(mm_base + me + 1);
                state.mmend[mod_num] = data_ptr.add(mm_base + cp_end.unwrap_or(cp));
                state.ml[mod_num] = if let Some(ml_base) = ml_base {
                    data_ptr.offset(
                        (ml_base + ml_pos + n) as isize
                            + (ndelta as isize - 1) * stride as isize,
                    )
                } else {
                    std::ptr::null_mut()
                };
            } else {
                state.mm[mod_num] = data_ptr.add(mm_base + cp_end.unwrap_or(cp));
                state.mmend[mod_num] = std::ptr::null_mut();
                state.ml[mod_num] = if let Some(ml_base) = ml_base {
                    data_ptr.add(ml_base + ml_pos + n)
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
            if ml_pos > ml.len() {
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

    if ml.is_some_and(|ml| ml_pos != ml.len()) {
        return -1;
    }
    state.nmods = mod_num as i32;
    0
}

pub unsafe fn bam_parse_basemod(b: &bam1_t, state: &mut hts_base_mod_state) -> i32 {
    bam_parse_basemod2(b, state, 0)
}

pub unsafe fn bam_mods_at_next_pos(
    b: &bam1_t,
    state: &mut hts_base_mod_state,
    mods: &mut [hts_base_mod],
) -> i32 {
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
                *state.ml[idx] as i32
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
                if *cp == b',' {
                    break;
                }
                cp = cp.sub(1);
            }
            state.mmend[idx] = cp;
            if cp != state.mm[idx] {
                let mut tmp = std::ptr::null_mut();
                state.mmcount[idx] =
                    hts_str2uint(cp.add(1).cast(), &mut tmp, 31, &mut failed) as i32;
            } else {
                state.mmcount[idx] = i32::MAX;
            }
        } else if *state.mm[idx] == b',' {
            let mut next = std::ptr::null_mut();
            state.mmcount[idx] =
                hts_str2uint(state.mm[idx].add(1).cast(), &mut next, 31, &mut failed) as i32;
            state.mm[idx] = next.cast::<u8>();
        } else {
            state.mmcount[idx] = i32::MAX;
        }
        if failed != 0 {
            return -1;
        }

        let mut j = i + 1;
        while j < state.nmods && state.mm[j as usize] == mmptr {
            let jidx = j as usize;
            if let Some(out) = mods.get_mut(n as usize) {
                let qual = if !state.ml[jidx].is_null() {
                    *state.ml[jidx] as i32
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

pub unsafe fn bam_next_basemod(
    b: &bam1_t,
    state: &mut hts_base_mod_state,
    mods: &mut [hts_base_mod],
    pos: &mut i32,
) -> i32 {
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
                    || (*state.mm[idx] != 0 && *state.mm[idx] != b';')
                {
                    return -1;
                }
            }
        }
        return 0;
    }

    let r = bam_mods_at_next_pos(b, state, mods);
    if r > 0 {
        r
    } else {
        0
    }
}

pub unsafe fn bam_mods_at_qpos(
    b: &bam1_t,
    qpos: i32,
    state: &mut hts_base_mod_state,
    mods: &mut [hts_base_mod],
) -> i32 {
    let mut r = 0;
    loop {
        if state.seq_pos > qpos {
            break;
        }
        r = bam_mods_at_next_pos(b, state, mods);
        if r < 0 {
            break;
        }
    }
    r
}
