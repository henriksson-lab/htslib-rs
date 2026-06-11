use std::collections::HashMap;

use crate::htslib_rs::hts::{
    hts_close, hts_getline, hts_open, hts_parse_decimal, hts_pos_t, isspace_c, ks_free, kstring_t,
};

pub const REGIDX_MAX: hts_pos_t = 1_i64 << 35;
pub const MAX_COOR_0: hts_pos_t = REGIDX_MAX;

fn ibin(x: hts_pos_t) -> i32 {
    (x >> 13) as i32
}

#[derive(Clone, Copy)]
pub struct regidx_reg_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
}

type reg_t = regidx_reg_t;

/// A parsing callback maps a single line into a chromosome name byte-range
/// within the line plus a begin/end coordinate and an optional payload.
/// It returns 0 on success, -1 to skip the line, and -2 on a parse error.
pub type regidx_parse_f =
    fn(line: &[u8], out: &mut ParsedRegion, payload: &mut [u8], usr: Option<&mut Vec<u8>>) -> i32;

pub struct regitr_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    /// A copy of the current region's payload bytes, empty when there is none.
    pub payload: Vec<u8>,
    /// The sequence name of the current region, empty when unset.
    pub seq: Vec<u8>,
    itr: Box<itr_t_>,
}

// original: itr_t_ (htslib/regidx.c:53)
#[derive(Clone)]
pub struct itr_t_ {
    beg: hts_pos_t,
    end: hts_pos_t,
    ireg: u32,
    /// Index of the owning regidx; the iterator borrows it at call time.
    ridx: *mut regidx_t,
    /// Index into `regidx_t.seq` of the active sequence list, if any.
    list: Option<usize>,
    active: bool,
}

// original: reglist_t (htslib/regidx.c:66)
#[derive(Default)]
pub struct reglist_t {
    idx: Vec<u32>,
    reg: Vec<reg_t>,
    dat: Vec<u8>,
    /// Index into `regidx_t.seq_names` of this list's sequence name.
    seq: usize,
    unsorted: bool,
}

// original: regidx_t (htslib/regidx.c:77)
pub struct regidx_t {
    seq: Vec<reglist_t>,
    /// Maps a sequence name to its index in `seq` / `seq_names`.
    seq2regs: HashMap<Vec<u8>, usize>,
    seq_names: Vec<Vec<u8>>,
    free: Option<fn(&mut [u8])>,
    parse: regidx_parse_f,
    usr: Option<Vec<u8>>,
    payload_size: usize,
    payload: Vec<u8>,
}

impl regidx_t {
    fn new(
        parse: regidx_parse_f,
        free: Option<fn(&mut [u8])>,
        payload_size: usize,
        usr: Option<Vec<u8>>,
    ) -> Self {
        Self {
            seq: Vec::new(),
            seq2regs: HashMap::new(),
            seq_names: Vec::new(),
            free,
            parse,
            usr,
            payload_size,
            payload: if payload_size == 0 {
                Vec::new()
            } else {
                vec![0; payload_size]
            },
        }
    }
}

#[derive(Default)]
pub struct ParsedRegion {
    pub chr: Option<std::ops::RangeInclusive<usize>>,
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
}

impl ParsedRegion {
    fn set_chr(&mut self, chr_beg: usize, chr_end: usize) {
        self.chr = Some(chr_beg..=chr_end);
    }
}

impl itr_t_ {
    fn new(regidx: *mut regidx_t) -> Self {
        Self {
            beg: 0,
            end: 0,
            ireg: 0,
            ridx: regidx,
            list: None,
            active: false,
        }
    }

    fn reset(&mut self, regidx: *mut regidx_t) {
        *self = Self::new(regidx);
    }
}

pub fn regidx_c_91_regidx_seq_nregs(idx: &regidx_t, seq: &[u8]) -> i32 {
    match idx.seq2regs.get(seq) {
        Some(&iseq) => idx.seq[iseq].reg.len() as i32,
        None => 0,
    }
}

pub fn regidx_c_98_regidx_nregs(idx: &regidx_t) -> i32 {
    idx.seq.iter().map(|list| list.reg.len() as i32).sum()
}

pub fn regidx_c_105_regidx_seq_names(idx: &regidx_t) -> &[Vec<u8>] {
    &idx.seq_names
}

pub fn regidx_c_111_regidx_insert_list(idx: &mut regidx_t, line: &[u8], delim: u8) -> i32 {
    let mut ss = 0;
    while ss < line.len() {
        let mut se = ss;
        while se < line.len() && line[se] != delim {
            se += 1;
        }
        if regidx_insert(idx, &line[ss..se]) < 0 {
            return -1;
        }
        if se >= line.len() {
            break;
        }
        ss = se + 1;
    }
    0
}

pub fn regidx_c_132_cmp_regs(a: &regidx_reg_t, b: &regidx_reg_t) -> i32 {
    if a.beg < b.beg {
        return -1;
    }
    if a.beg > b.beg {
        return 1;
    }
    if a.end < b.end {
        return 1;
    }
    if a.end > b.end {
        return -1;
    }
    let a_ptr = std::ptr::from_ref(a);
    let b_ptr = std::ptr::from_ref(b);
    if a_ptr < b_ptr {
        return -1;
    }
    if a_ptr > b_ptr {
        return 1;
    }
    0
}

pub fn regidx_c_142_cmp_reg_ptrs(a: &regidx_reg_t, b: &regidx_reg_t) -> i32 {
    regidx_c_132_cmp_regs(a, b)
}

pub fn regidx_c_146_cmp_reg_ptrs2(a: &regidx_reg_t, b: &regidx_reg_t) -> i32 {
    regidx_c_132_cmp_regs(a, b)
}

fn cmp_reg_values(a: reg_t, b: reg_t) -> std::cmp::Ordering {
    a.beg.cmp(&b.beg).then_with(|| b.end.cmp(&a.end))
}

pub fn regidx_c_151_regidx_push(
    idx: &mut regidx_t,
    chr: &[u8],
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    payload: &[u8],
) -> i32 {
    beg = beg.clamp(0, MAX_COOR_0);
    end = end.clamp(0, MAX_COOR_0);

    let rid = match idx.seq2regs.get(chr) {
        Some(&rid) => rid,
        None => {
            let rid = idx.seq_names.len();
            idx.seq_names.push(chr.to_vec());
            idx.seq.push(reglist_t {
                seq: rid,
                ..Default::default()
            });
            idx.seq2regs.insert(chr.to_vec(), rid);
            rid
        }
    };

    let payload_size = idx.payload_size;
    let list = &mut idx.seq[rid];
    list.seq = rid;
    let prev = list.reg.last().copied();
    list.reg.push(reg_t { beg, end });
    if payload_size != 0 {
        if payload.len() < payload_size {
            return -1;
        }
        list.dat.extend_from_slice(&payload[..payload_size]);
    }
    if !list.unsorted
        && prev.is_some()
        && cmp_reg_values(prev.unwrap(), *list.reg.last().unwrap()).is_gt()
    {
        list.unsorted = true;
    }
    0
}

pub fn regidx_insert(idx: &mut regidx_t, line: &[u8]) -> i32 {
    let mut out = ParsedRegion::default();
    let mut payload = std::mem::take(&mut idx.payload);
    let mut usr = idx.usr.take();
    let ret = (idx.parse)(line, &mut out, &mut payload, usr.as_mut());
    idx.payload = payload;
    idx.usr = usr;
    if ret == -2 {
        return -1;
    }
    if ret == -1 {
        return 0;
    }
    let Some(chr) = out.chr else {
        return -1;
    };
    let chr_bytes = line[*chr.start()..=*chr.end()].to_vec();
    let payload = std::mem::take(&mut idx.payload);
    let ret = regidx_c_151_regidx_push(idx, &chr_bytes, out.beg, out.end, &payload);
    idx.payload = payload;
    ret
}

pub fn regidx_c_198_regidx_insert(idx: &mut regidx_t, line: &[u8]) -> i32 {
    regidx_insert(idx, line)
}

pub fn regidx_c_209_regidx_init_string(
    string: &[u8],
    parsef: Option<regidx_parse_f>,
    freef: Option<fn(&mut [u8])>,
    payload_size: usize,
    usr: Option<Vec<u8>>,
) -> Option<Box<regidx_t>> {
    let parse = parsef.unwrap_or(regidx_c_498_regidx_parse_tab as regidx_parse_f);
    let mut idx = Box::new(regidx_t::new(parse, freef, payload_size, usr));

    let mut ss = 0;
    while ss < string.len() {
        while ss < string.len() && isspace_c(string[ss] as i8) != 0 {
            ss += 1;
        }
        let mut se = ss;
        while se < string.len() && string[se] != b'\r' && string[se] != b'\n' {
            se += 1;
        }
        if regidx_insert(&mut idx, &string[ss..se]) < 0 {
            return None;
        }
        while se < string.len() && isspace_c(string[se] as i8) != 0 {
            se += 1;
        }
        ss = se;
    }
    Some(idx)
}

pub unsafe fn regidx_c_246_regidx_init(
    fname: Option<&[u8]>,
    parsef: Option<regidx_parse_f>,
    freef: Option<fn(&mut [u8])>,
    payload_size: usize,
    usr: Option<Vec<u8>>,
) -> Option<Box<regidx_t>> {
    let parse = match parsef {
        Some(p) => p,
        None => match fname {
            None => regidx_c_498_regidx_parse_tab as regidx_parse_f,
            Some(fname) => {
                let is_bed = [
                    b".bed.gz".as_slice(),
                    b".bed.bgz".as_slice(),
                    b".bed".as_slice(),
                ]
                .iter()
                .any(|suffix| {
                    fname.len() >= suffix.len()
                        && fname[fname.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
                });
                let is_vcf = [b".vcf".as_slice(), b".vcf.gz".as_slice()]
                    .iter()
                    .any(|suffix| {
                        fname.len() >= suffix.len()
                            && fname[fname.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
                    });
                if is_bed {
                    regidx_c_466_regidx_parse_bed as regidx_parse_f
                } else if is_vcf {
                    regidx_c_538_regidx_parse_vcf as regidx_parse_f
                } else {
                    regidx_c_498_regidx_parse_tab as regidx_parse_f
                }
            }
        },
    };

    let mut idx = Box::new(regidx_t::new(parse, freef, payload_size, usr));

    let Some(fname) = fname else {
        return Some(idx);
    };

    // File reading is the genuine I/O boundary; hts_getline fills the kstring_t.
    let mut str_ = kstring_t::default();
    let mut fname_c = fname.to_vec();
    fname_c.push(0);
    let fp = hts_open(fname_c.as_ptr().cast(), c"r".as_ptr());
    if fp.is_null() {
        ks_free(&mut str_);
        return None;
    }

    let mut ret = hts_getline(fp, b'\n' as i32, &mut str_);
    while ret > 0 {
        let line = str_.data.clone();
        if regidx_insert(&mut idx, &line) != 0 {
            hts_close(fp);
            ks_free(&mut str_);
            return None;
        }
        ret = hts_getline(fp, b'\n' as i32, &mut str_);
    }
    ks_free(&mut str_);
    if ret < -1 {
        hts_close(fp);
        return None;
    }

    if hts_close(fp) != 0 {
        return None;
    }
    Some(idx)
}

pub fn regidx_c_311_regidx_destroy(idx: Box<regidx_t>) {
    let mut idx = idx;
    if let Some(free) = idx.free {
        let payload_size = idx.payload_size;
        if payload_size != 0 {
            for list in &mut idx.seq {
                for chunk in list.dat.chunks_mut(payload_size) {
                    free(chunk);
                }
            }
        }
    }
    drop(idx);
}

// original: reglist_build_index_ (htslib/regidx.c:335)
pub fn regidx_c_335_reglist_build_index_(payload_size: usize, list: &mut reglist_t) -> i32 {
    if list.unsorted {
        if payload_size == 0 {
            list.reg.sort_by(|a, b| cmp_reg_values(*a, *b));
        } else {
            let mut pairs: Vec<(reg_t, Vec<u8>)> = list
                .reg
                .iter()
                .copied()
                .zip(list.dat.chunks(payload_size).map(|chunk| chunk.to_vec()))
                .collect();
            pairs.sort_by(|a, b| cmp_reg_values(a.0, b.0));
            list.reg.clear();
            list.dat.clear();
            for (reg, mut payload) in pairs {
                list.reg.push(reg);
                list.dat.append(&mut payload);
            }
        }
        list.unsorted = false;
    }

    let mut midx: u32 = 0;
    for reg in &list.reg {
        let iend = ibin(reg.end) as u32;
        if midx <= iend {
            midx = iend;
        }
    }
    midx += 1;
    list.idx.clear();
    list.idx.resize(midx as usize, 0);

    for (j, reg) in list.reg.iter().enumerate() {
        let ibeg = ibin(reg.beg) as u32;
        let iend = ibin(reg.end) as u32;
        if ibeg == iend {
            if list.idx[ibeg as usize] == 0 {
                list.idx[ibeg as usize] = j as u32 + 1;
            }
        } else {
            for k in ibeg..=iend {
                if list.idx[k as usize] == 0 {
                    list.idx[k as usize] = j as u32 + 1;
                }
            }
        }
    }

    0
}

fn regitr_set_region(
    regitr: &mut regitr_t,
    seq_name: &[u8],
    list: &reglist_t,
    ireg: u32,
    payload_size: usize,
) {
    regitr.seq = seq_name.to_vec();
    regitr.beg = list.reg[ireg as usize].beg;
    regitr.end = list.reg[ireg as usize].end;
    regitr.payload = if payload_size != 0 {
        let start = payload_size * ireg as usize;
        list.dat[start..start + payload_size].to_vec()
    } else {
        Vec::new()
    };
}

pub fn regidx_c_401_regidx_overlap(
    idx: &mut regidx_t,
    chr: &[u8],
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    mut regitr: Option<&mut regitr_t>,
) -> i32 {
    let idx_ptr: *mut regidx_t = idx;
    if let Some(regitr) = regitr.as_deref_mut() {
        regitr.seq = Vec::new();
        regitr.payload = Vec::new();
    }
    beg = beg.max(0);
    end = end.clamp(0, MAX_COOR_0);

    let Some(&iseq) = idx.seq2regs.get(chr) else {
        return 0;
    };

    let payload_size = idx.payload_size;
    let list = &mut idx.seq[iseq];
    if list.reg.is_empty() {
        return 0;
    }

    let mut ireg: u32;
    if list.reg.len() == 1 {
        if beg > list.reg[0].end {
            return 0;
        }
        if end < list.reg[0].beg {
            return 0;
        }
        ireg = 0;
    } else {
        if list.idx.is_empty() && regidx_c_335_reglist_build_index_(payload_size, list) < 0 {
            return -1;
        }

        let ibeg = ibin(beg);
        if ibeg >= list.idx.len() as i32 {
            return 0;
        }

        let mut i = list.idx[ibeg as usize];
        if i == 0 {
            let mut iend = ibin(end);
            if iend > list.idx.len() as i32 {
                iend = list.idx.len() as i32;
            }
            let mut k = ibeg;
            while k <= iend {
                if list.idx[k as usize] != 0 {
                    break;
                }
                k += 1;
            }
            if k > iend {
                return 0;
            }
            i = list.idx[k as usize];
        }
        ireg = i - 1;
        while (ireg as usize) < list.reg.len() {
            if list.reg[ireg as usize].beg > end {
                return 0;
            }
            if list.reg[ireg as usize].end >= beg && list.reg[ireg as usize].beg <= end {
                break;
            }
            ireg += 1;
        }

        if (ireg as usize) >= list.reg.len() {
            return 0;
        }
    }

    let Some(regitr) = regitr else {
        return 1;
    };

    let seq_name = idx.seq_names[list.seq].clone();
    let itr = &mut regitr.itr;
    itr.ridx = idx_ptr;
    itr.list = Some(iseq);
    itr.beg = beg;
    itr.end = end;
    itr.ireg = ireg;
    itr.active = false;
    regitr_set_region(regitr, &seq_name, &idx.seq[iseq], ireg, payload_size);

    1
}

pub fn regidx_c_466_regidx_parse_bed(
    line: &[u8],
    out: &mut ParsedRegion,
    _payload: &mut [u8],
    _usr: Option<&mut Vec<u8>>,
) -> i32 {
    // hts_parse_decimal walks a NUL-terminated C string; keep a local copy.
    let mut buf = line.to_vec();
    buf.push(0);

    let mut ss = 0;
    while line.get(ss).is_some_and(|&c| isspace_c(c as i8) != 0) {
        ss += 1;
    }
    if ss >= line.len() {
        return -1;
    }
    if line[ss] == b'#' {
        return -1;
    }

    let mut se = ss;
    while se < line.len() && isspace_c(line[se] as i8) == 0 {
        se += 1;
    }

    out.set_chr(ss, se - 1);

    if se >= line.len() {
        out.beg = 0;
        out.end = MAX_COOR_0;
        return 0;
    }

    ss = se + 1;
    se = unsafe {
        let mut end = std::ptr::null_mut();
        let start = buf[ss.min(buf.len() - 1)..].as_ptr().cast();
        out.beg = hts_parse_decimal(start, &mut end, 0);
        ss + end.offset_from(start) as usize
    };
    if ss == se {
        return -2;
    }

    ss = se + 1;
    se = unsafe {
        let mut end = std::ptr::null_mut();
        let start = buf[ss.min(buf.len() - 1)..].as_ptr().cast();
        out.end = hts_parse_decimal(start, &mut end, 0) - 1;
        ss + end.offset_from(start) as usize
    };
    if ss == se {
        return -2;
    }

    0
}

pub fn regidx_c_498_regidx_parse_tab(
    line: &[u8],
    out: &mut ParsedRegion,
    _payload: &mut [u8],
    _usr: Option<&mut Vec<u8>>,
) -> i32 {
    // hts_parse_decimal walks a NUL-terminated C string; keep a local copy.
    let mut buf = line.to_vec();
    buf.push(0);

    let mut ss = 0;
    while line.get(ss).is_some_and(|&c| isspace_c(c as i8) != 0) {
        ss += 1;
    }
    if ss >= line.len() {
        return -1;
    }
    if line[ss] == b'#' {
        return -1;
    }

    let mut se = ss;
    while se < line.len() && isspace_c(line[se] as i8) == 0 {
        se += 1;
    }

    out.set_chr(ss, se - 1);

    if se >= line.len() {
        out.beg = 0;
        out.end = MAX_COOR_0;
        return 0;
    }

    ss = se + 1;
    se = unsafe {
        let mut end = std::ptr::null_mut();
        let start = buf[ss.min(buf.len() - 1)..].as_ptr().cast();
        out.beg = hts_parse_decimal(start, &mut end, 0);
        ss + end.offset_from(start) as usize
    };
    if ss == se {
        return -2;
    }
    if out.beg == 0 {
        return -2;
    }
    out.beg -= 1;

    // se now indexes into `line`; a delimiter at end-of-line means no end field.
    if se >= line.len() || se + 1 >= line.len() {
        out.end = out.beg;
    } else {
        ss = se + 1;
        se = unsafe {
            let mut end = std::ptr::null_mut();
            let start = buf[ss.min(buf.len() - 1)..].as_ptr().cast();
            out.end = hts_parse_decimal(start, &mut end, 0);
            ss + end.offset_from(start) as usize
        };
        if ss == se || (se < line.len() && isspace_c(line[se] as i8) == 0) {
            out.end = out.beg;
        } else if out.end == 0 {
            return -2;
        } else {
            out.end -= 1;
        }
    }
    0
}

pub fn regidx_c_538_regidx_parse_vcf(
    line: &[u8],
    out: &mut ParsedRegion,
    payload: &mut [u8],
    usr: Option<&mut Vec<u8>>,
) -> i32 {
    let ret = regidx_c_498_regidx_parse_tab(line, out, payload, usr);
    if ret == 0 {
        out.end = out.beg;
    }
    ret
}

pub fn regidx_c_545_regidx_parse_reg(
    line: &[u8],
    out: &mut ParsedRegion,
    _payload: &mut [u8],
    _usr: Option<&mut Vec<u8>>,
) -> i32 {
    // hts_parse_decimal walks a NUL-terminated C string; keep a local copy.
    let mut buf = line.to_vec();
    buf.push(0);

    let mut ss = 0;
    while line.get(ss).is_some_and(|&c| isspace_c(c as i8) != 0) {
        ss += 1;
    }
    if ss >= line.len() {
        return -1;
    }
    if line[ss] == b'#' {
        return -1;
    }

    let mut se = ss;
    while se < line.len() && line[se] != b':' {
        se += 1;
    }

    out.set_chr(ss, se - 1);

    if se >= line.len() {
        out.beg = 0;
        out.end = MAX_COOR_0;
        return 0;
    }

    ss = se + 1;
    se = unsafe {
        let mut end = std::ptr::null_mut();
        let start = buf[ss.min(buf.len() - 1)..].as_ptr().cast();
        out.beg = hts_parse_decimal(start, &mut end, 0);
        ss + end.offset_from(start) as usize
    };
    if ss == se {
        return -2;
    }
    if out.beg == 0 {
        return -2;
    }
    out.beg -= 1;

    if se >= line.len() || se + 1 >= line.len() {
        out.end = if se < line.len() && line[se] == b'-' {
            MAX_COOR_0
        } else {
            out.beg
        };
    } else {
        ss = se + 1;
        se = unsafe {
            let mut end = std::ptr::null_mut();
            let start = buf[ss.min(buf.len() - 1)..].as_ptr().cast();
            out.end = hts_parse_decimal(start, &mut end, 0);
            ss + end.offset_from(start) as usize
        };
        if ss == se {
            out.end = out.beg;
        } else if out.end == 0 {
            return -2;
        } else {
            out.end -= 1;
        }
    }
    0
}

pub fn regidx_c_584_regitr_init(regidx: &mut regidx_t) -> Box<regitr_t> {
    Box::new(regitr_t {
        beg: 0,
        end: 0,
        payload: Vec::new(),
        seq: Vec::new(),
        itr: Box::new(itr_t_::new(regidx)),
    })
}

pub fn regidx_c_599_regitr_reset(regidx: &mut regidx_t, regitr: &mut regitr_t) {
    regitr.itr.reset(regidx);
    regitr.beg = 0;
    regitr.end = 0;
    regitr.payload = Vec::new();
    regitr.seq = Vec::new();
}

pub fn regidx_c_606_regitr_destroy(regitr: Box<regitr_t>) {
    drop(regitr);
}

pub fn regidx_c_612_regitr_overlap(regitr: &mut regitr_t) -> i32 {
    if regitr.seq.is_empty() {
        return 0;
    }

    let (iseq, i) = {
        let itr = &mut regitr.itr;
        if !itr.active {
            itr.active = true;
            itr.ireg += 1;
            return 1;
        }

        let Some(iseq) = itr.list else {
            return 0;
        };
        let regidx = unsafe { &*itr.ridx };
        let list = &regidx.seq[iseq];
        let mut i = itr.ireg;
        while (i as usize) < list.reg.len() {
            if list.reg[i as usize].beg > itr.end {
                return 0;
            }
            if list.reg[i as usize].end >= itr.beg && list.reg[i as usize].beg <= itr.end {
                break;
            }
            i += 1;
        }

        if (i as usize) >= list.reg.len() {
            return 0;
        }

        itr.ireg = i + 1;
        (iseq, i)
    };

    let regidx = unsafe { &*regitr.itr.ridx };
    let payload_size = regidx.payload_size;
    let seq_name = regidx.seq_names[regidx.seq[iseq].seq].clone();
    regitr_set_region(regitr, &seq_name, &regidx.seq[iseq], i, payload_size);

    1
}

pub fn regidx_c_646_regitr_loop(regitr: &mut regitr_t) -> i32 {
    let (iseq, ireg) = {
        let itr = &mut regitr.itr;
        let regidx = unsafe { &*itr.ridx };

        if regidx.seq.is_empty() {
            return 0;
        }

        if itr.list.is_none() {
            itr.list = Some(0);
            itr.ireg = 0;
        }

        let mut iseq = itr.list.unwrap();
        if iseq >= regidx.seq.len() {
            return 0;
        }

        if (itr.ireg as usize) >= regidx.seq[iseq].reg.len() {
            iseq += 1;
            if iseq >= regidx.seq.len() {
                return 0;
            }
            itr.ireg = 0;
            itr.list = Some(iseq);
        }

        let ireg = itr.ireg;
        itr.ireg += 1;
        (iseq, ireg)
    };

    let regidx = unsafe { &*regitr.itr.ridx };
    let payload_size = regidx.payload_size;
    let seq_name = regidx.seq_names[regidx.seq[iseq].seq].clone();
    regitr_set_region(regitr, &seq_name, &regidx.seq[iseq], ireg, payload_size);

    1
}

pub fn regidx_c_681_regitr_copy(dst: &mut regitr_t, src: &regitr_t) {
    if std::ptr::eq(dst, src) {
        return;
    }
    dst.beg = src.beg;
    dst.end = src.end;
    dst.payload = src.payload.clone();
    dst.seq = src.seq.clone();
    dst.itr = Box::new((*src.itr).clone());
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse(parser: regidx_parse_f, line: &[u8]) -> (i32, ParsedRegion) {
        let mut out = ParsedRegion::default();
        let mut payload = Vec::new();
        let ret = parser(line, &mut out, &mut payload, None);
        (ret, out)
    }

    #[test]
    fn regidx_parse_bed_uses_zero_based_right_open_coordinates() {
        let (ret, out) = parse(regidx_c_466_regidx_parse_bed, b"chr1\t10\t20");
        assert_eq!(ret, 0);
        assert_eq!(out.beg, 10);
        assert_eq!(out.end, 19);
        let chr = out.chr.unwrap();
        assert_eq!(chr.end() - chr.start(), 3);
    }

    #[test]
    fn regidx_parse_bed_accepts_contig_only_and_rejects_bad_coordinates() {
        let (ret, out) = parse(regidx_c_466_regidx_parse_bed, b"chr1");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (0, MAX_COOR_0));
        let chr = out.chr.unwrap();
        assert_eq!(chr.end() - chr.start(), 3);

        for line in [b"chr1 start 20".as_slice(), b"chr1 10 end".as_slice()] {
            let (ret, _) = parse(regidx_c_466_regidx_parse_bed, line);
            assert_eq!(ret, -2);
        }
    }

    #[test]
    fn regidx_parse_tab_and_vcf_use_one_based_positions() {
        let (ret, out) = parse(regidx_c_498_regidx_parse_tab, b"chr2 11 14");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (10, 13));

        let (ret, out) = parse(regidx_c_538_regidx_parse_vcf, b"chr2 11 14");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (10, 10));
    }

    #[test]
    fn regidx_parse_tab_boundary_cases_match_htslib_fallbacks() {
        let (ret, out) = parse(regidx_c_498_regidx_parse_tab, b"  chr1");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (0, MAX_COOR_0));
        let chr = out.chr.unwrap();
        assert_eq!(chr.end() - chr.start(), 3);

        let (ret, out) = parse(regidx_c_498_regidx_parse_tab, b"chr2 5 -");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (4, 4));

        let (ret, out) = parse(regidx_c_498_regidx_parse_tab, b"chr2 5 9x");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (4, 4));
    }

    #[test]
    fn regidx_parse_tab_and_vcf_reject_zero_start_coordinates() {
        let (ret, _) = parse(regidx_c_498_regidx_parse_tab, b"chr1 0 10");
        assert_eq!(ret, -2);

        let (ret, _) = parse(regidx_c_538_regidx_parse_vcf, b"chr1 0 .");
        assert_eq!(ret, -2);
    }

    #[test]
    fn regidx_parse_and_insert_distinguish_skip_from_malformed_lines() {
        let (ret, _) = parse(regidx_c_498_regidx_parse_tab, b"#comment");
        assert_eq!(ret, -1);

        let (ret, _) = parse(regidx_c_498_regidx_parse_tab, b"chr1 0");
        assert_eq!(ret, -2);

        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_498_regidx_parse_tab), None, 0, None)
        }
        .expect("regidx");

        assert_eq!(regidx_c_198_regidx_insert(&mut idx, b"#comment"), 0);
        assert_eq!(regidx_c_198_regidx_insert(&mut idx, b"chr1 0"), -1);

        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_parse_reg_handles_open_ended_regions() {
        let (ret, out) = parse(regidx_c_545_regidx_parse_reg, b"chr3:42-");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (41, MAX_COOR_0));
    }

    #[test]
    fn regidx_parse_reg_handles_whole_contig_point_and_empty_range_edges() {
        let (ret, out) = parse(regidx_c_545_regidx_parse_reg, b"chr1");
        assert_eq!(ret, 0);
        let chr = out.chr.unwrap();
        assert_eq!(&b"chr1"[*chr.start()..=*chr.end()], b"chr1");
        assert_eq!(chr.end() - chr.start(), 3);
        assert_eq!((out.beg, out.end), (0, MAX_COOR_0));

        let (ret, out) = parse(regidx_c_545_regidx_parse_reg, b"chr1:7");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (6, 6));

        let (ret, _) = parse(regidx_c_545_regidx_parse_reg, b"chr1:");
        assert_eq!(ret, -2);
    }

    #[test]
    fn regidx_parse_reg_rejects_zero_start_and_collapses_bad_end_to_point() {
        let (ret, _) = parse(regidx_c_545_regidx_parse_reg, b"chr1:0-10");
        assert_eq!(ret, -2);

        let (ret, out) = parse(regidx_c_545_regidx_parse_reg, b"chr1:5-end");
        assert_eq!(ret, 0);
        assert_eq!((out.beg, out.end), (4, 4));
    }

    #[test]
    fn regidx_parse_reg_stops_sequence_name_at_first_colon() {
        let line = b"chr1:alt:5-7";
        let (ret, out) = parse(regidx_c_545_regidx_parse_reg, line);
        assert_eq!(ret, -2);
        let chr = out.chr.unwrap();
        assert_eq!(&line[*chr.start()..=*chr.end()], b"chr1");
    }

    #[test]
    fn regidx_comparators_order_by_begin_then_longer_end_then_address() {
        let a = regidx_reg_t { beg: 10, end: 20 };
        let b = regidx_reg_t { beg: 10, end: 25 };
        let c = regidx_reg_t { beg: 11, end: 12 };

        assert_eq!(regidx_c_132_cmp_regs(&a, &b), 1);
        assert_eq!(regidx_c_132_cmp_regs(&c, &b), 1);
        assert_eq!(regidx_c_142_cmp_reg_ptrs(&b, &a), -1);
        assert_eq!(regidx_c_146_cmp_reg_ptrs2(&b, &a), -1);
    }

    fn push(idx: &mut regidx_t, chr: &[u8], beg: hts_pos_t, end: hts_pos_t, payload: &[u8]) {
        assert_eq!(regidx_c_151_regidx_push(idx, chr, beg, end, payload), 0);
    }

    #[test]
    fn regidx_public_api_builds_queries_and_iterates_regions() {
        let mut idx = regidx_c_209_regidx_init_string(
            b"chr1\t5\t7\nchr1\t9\t10\nchr2\t3\n",
            Some(regidx_c_498_regidx_parse_tab),
            None,
            0,
            None,
        )
        .expect("regidx");

        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"chr1"), 2);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"chr3"), 0);
        assert_eq!(regidx_c_98_regidx_nregs(&idx), 3);

        assert_eq!(regidx_c_105_regidx_seq_names(&idx).len(), 2);

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 4, 4, Some(&mut itr)), 1);
        assert_eq!(itr.beg, 4);
        assert_eq!(itr.end, 6);
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 0);

        regidx_c_599_regitr_reset(&mut idx, &mut itr);
        assert_eq!(regidx_c_646_regitr_loop(&mut itr), 1);
        assert_eq!(itr.beg, 4);

        let mut copy = regidx_c_584_regitr_init(&mut idx);
        regidx_c_681_regitr_copy(&mut copy, &itr);
        assert_eq!(copy.beg, itr.beg);

        regidx_c_606_regitr_destroy(copy);
        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_loop_on_empty_index_returns_no_regions() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_498_regidx_parse_tab), None, 0, None)
        }
        .expect("regidx");

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_646_regitr_loop(&mut itr), 0);

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_iterator_copy_keeps_independent_overlap_position() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_498_regidx_parse_tab), None, 0, None)
        }
        .expect("regidx");

        for (beg, end) in [(0, 0), (5, 5), (10, 10)] {
            push(&mut idx, b"chr1", beg, end, &[]);
        }

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 0, 10, Some(&mut itr)), 1);
        assert_eq!((itr.beg, itr.end), (0, 0));

        let mut copy = regidx_c_584_regitr_init(&mut idx);
        regidx_c_681_regitr_copy(&mut copy, &itr);

        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!((itr.beg, itr.end), (0, 0));
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!((itr.beg, itr.end), (5, 5));

        assert_eq!(regidx_c_612_regitr_overlap(&mut copy), 1);
        assert_eq!((copy.beg, copy.end), (0, 0));
        assert_eq!(regidx_c_612_regitr_overlap(&mut copy), 1);
        assert_eq!((copy.beg, copy.end), (5, 5));

        regidx_c_606_regitr_destroy(copy);
        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_iterator_self_copy_matches_c_struct_assignment() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_498_regidx_parse_tab), None, 0, None)
        }
        .expect("regidx");

        for (beg, end) in [(0, 0), (5, 5)] {
            push(&mut idx, b"chr1", beg, end, &[]);
        }

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 0, 5, Some(&mut itr)), 1);
        let itr_ref: *mut regitr_t = &mut *itr;
        regidx_c_681_regitr_copy(unsafe { &mut *itr_ref }, unsafe { &*itr_ref });

        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!((itr.beg, itr.end), (0, 0));
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!((itr.beg, itr.end), (5, 5));

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_unsorted_payloads_stay_attached_to_regions_after_index_build() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(
                None,
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<i32>(),
                None,
            )
        }
        .expect("regidx");

        for (beg, end, payload) in [(30, 30, 30i32), (10, 10, 10), (20, 20, 20)] {
            push(&mut idx, b"chr1", beg, end, &payload.to_ne_bytes());
        }

        let mut itr = regidx_c_584_regitr_init(&mut idx);

        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 10, 10, Some(&mut itr)), 1);
        assert_eq!((itr.beg, itr.end), (10, 10));
        assert_eq!(i32::from_ne_bytes(itr.payload[..4].try_into().unwrap()), 10);

        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 30, 30, Some(&mut itr)), 1);
        assert_eq!((itr.beg, itr.end), (30, 30));
        assert_eq!(i32::from_ne_bytes(itr.payload[..4].try_into().unwrap()), 30);

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_overlap_iteration_uses_sorted_interval_order_with_payloads() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(
                None,
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<i32>(),
                None,
            )
        }
        .expect("regidx");

        for (beg, end, payload) in [(10, 12, 12i32), (5, 5, 5), (10, 20, 20)] {
            push(&mut idx, b"chr1", beg, end, &payload.to_ne_bytes());
        }

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 0, 30, Some(&mut itr)), 1);

        let mut seen = Vec::new();
        while regidx_c_612_regitr_overlap(&mut itr) != 0 {
            seen.push((
                itr.beg,
                itr.end,
                i32::from_ne_bytes(itr.payload[..4].try_into().unwrap()),
            ));
        }
        assert_eq!(seen, [(5, 5, 5), (10, 20, 20), (10, 12, 12)]);

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_overlap_iterator_includes_touching_bin_edges_and_clears_misses() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(
                None,
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<i32>(),
                None,
            )
        }
        .expect("regidx");

        for (beg, end, payload) in [(0, 8191, 1i32), (8192, 8192, 2), (20000, 20010, 3)] {
            push(&mut idx, b"chr1", beg, end, &payload.to_ne_bytes());
        }

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 8191, 8192, Some(&mut itr)), 1);

        let mut seen = Vec::new();
        while regidx_c_612_regitr_overlap(&mut itr) != 0 {
            seen.push((
                itr.beg,
                itr.end,
                i32::from_ne_bytes(itr.payload[..4].try_into().unwrap()),
            ));
        }
        assert_eq!(seen, [(0, 8191, 1), (8192, 8192, 2)]);

        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 8193, 19999, Some(&mut itr)), 0);
        assert!(itr.seq.is_empty());

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_overlap_treats_interval_ends_as_inclusive_points() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_498_regidx_parse_tab), None, 0, None)
        }
        .expect("regidx");

        push(&mut idx, b"chr1", 10, 20, &[]);

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 20, 20, Some(&mut itr)), 1);
        assert_eq!((itr.beg, itr.end), (10, 20));
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 21, 21, Some(&mut itr)), 0);
        assert!(itr.seq.is_empty());

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_loop_preserves_sequence_and_insertion_order_without_overlap_query() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_545_regidx_parse_reg), None, 0, None)
        }
        .expect("regidx");

        assert_eq!(
            regidx_c_111_regidx_insert_list(&mut idx, b"chrB:3,chrA:2-4,chrB:1", b','),
            0
        );

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        let mut seen = Vec::new();
        while regidx_c_646_regitr_loop(&mut itr) != 0 {
            seen.push((itr.seq.clone(), itr.beg, itr.end));
        }

        assert_eq!(
            seen,
            [
                (b"chrB".to_vec(), 2, 2),
                (b"chrB".to_vec(), 0, 0),
                (b"chrA".to_vec(), 1, 3),
            ]
        );

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_insert_list_skips_empty_items_like_other_ignored_lines() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_545_regidx_parse_reg), None, 0, None)
        }
        .expect("regidx");

        assert_eq!(
            regidx_c_111_regidx_insert_list(&mut idx, b"chr1:1,,chr1:3,#comment,chr2", b','),
            0
        );

        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"chr1"), 2);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"chr2"), 1);
        assert_eq!(regidx_c_98_regidx_nregs(&idx), 3);

        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_init_string_skips_comments_and_aborts_on_malformed_records() {
        let idx = regidx_c_209_regidx_init_string(
            b"  #comment\n\nchr1\t1\t3\r\nchr2\t5\n",
            Some(regidx_c_498_regidx_parse_tab),
            None,
            0,
            None,
        )
        .expect("regidx");

        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"chr1"), 1);
        assert_eq!(regidx_c_91_regidx_seq_nregs(&idx, b"chr2"), 1);
        assert_eq!(regidx_c_98_regidx_nregs(&idx), 2);
        regidx_c_311_regidx_destroy(idx);

        let bad = regidx_c_209_regidx_init_string(
            b"chr1\t1\nchr2\t0\n",
            Some(regidx_c_498_regidx_parse_tab),
            None,
            0,
            None,
        );
        assert!(bad.is_none());
    }

    #[test]
    fn regidx_push_clamps_coordinates_to_htslib_regidx_bounds() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_545_regidx_parse_reg), None, 0, None)
        }
        .expect("regidx");

        push(&mut idx, b"chrC", -10, MAX_COOR_0 + 99, &[]);

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(
            regidx_c_401_regidx_overlap(&mut idx, b"chrC", MAX_COOR_0, MAX_COOR_0, Some(&mut itr)),
            1
        );
        assert_eq!((itr.beg, itr.end), (0, MAX_COOR_0));
        assert_eq!(
            regidx_c_401_regidx_overlap(
                &mut idx,
                b"chrC",
                0,
                crate::htslib_rs::hts::HTS_POS_MAX,
                Some(&mut itr)
            ),
            1
        );
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 1);
        assert_eq!(regidx_c_612_regitr_overlap(&mut itr), 0);

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_loop_uses_sorted_payload_order_after_overlap_builds_index() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(
                None,
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<i32>(),
                None,
            )
        }
        .expect("regidx");

        for (beg, end, payload) in [(50, 50, 50i32), (10, 20, 20), (10, 30, 30)] {
            push(&mut idx, b"chr1", beg, end, &payload.to_ne_bytes());
        }

        let mut itr = regidx_c_584_regitr_init(&mut idx);
        assert_eq!(regidx_c_401_regidx_overlap(&mut idx, b"chr1", 0, 60, Some(&mut itr)), 1);

        regidx_c_599_regitr_reset(&mut idx, &mut itr);
        let mut seen = Vec::new();
        while regidx_c_646_regitr_loop(&mut itr) != 0 {
            seen.push((
                itr.beg,
                itr.end,
                i32::from_ne_bytes(itr.payload[..4].try_into().unwrap()),
            ));
        }
        assert_eq!(seen, [(10, 30, 30), (10, 20, 20), (50, 50, 50)]);

        regidx_c_606_regitr_destroy(itr);
        regidx_c_311_regidx_destroy(idx);
    }

    #[test]
    fn regidx_init_insert_push_and_insert_list_match_public_api_edges() {
        let mut idx = unsafe {
            regidx_c_246_regidx_init(None, Some(regidx_c_545_regidx_parse_reg), None, 0, None)
        }
        .expect("regidx");

        assert_eq!(regidx_c_198_regidx_insert(&mut idx, b"chrX:2-4"), 0);

        assert_eq!(
            regidx_c_111_regidx_insert_list(&mut idx, b"chrX:8-9,chrY:1", b','),
            0
        );

        push(&mut idx, b"chrZ", 0, 2, &[]);

        assert_eq!(regidx_c_98_regidx_nregs(&idx), 4);
        regidx_c_311_regidx_destroy(idx);
    }
}
