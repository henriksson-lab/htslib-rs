use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    ptr::NonNull,
};

use crate::htslib_rs::{
    hts::{
        hts_close, hts_getline, hts_open, hts_parse_decimal, hts_pos_t, isspace_c, kputsn,
        ks_clear, ks_free, kstring_t,
    },
    sam::{
        khash_s2i_t, khash_str2int_destroy, khash_str2int_get, khash_str2int_inc,
        khash_str2int_init,
    },
};

pub const REGIDX_MAX: hts_pos_t = 1_i64 << 35;
pub const MAX_COOR_0: hts_pos_t = REGIDX_MAX;

fn ibin(x: hts_pos_t) -> c_int {
    (x >> 13) as c_int
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct regidx_reg_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
}

type reg_t = regidx_reg_t;

#[repr(C)]
pub struct regitr_t {
    pub beg: hts_pos_t,
    pub end: hts_pos_t,
    pub payload: Option<NonNull<c_void>>,
    pub seq: Option<NonNull<c_char>>,
    itr: Box<itr_t_>,
}

// original: itr_t_ (htslib/regidx.c:53)
#[derive(Clone, Copy)]
pub struct itr_t_ {
    beg: hts_pos_t,
    end: hts_pos_t,
    ireg: u32,
    ridx: NonNull<regidx_t>,
    list: Option<NonNull<reglist_t>>,
    active: c_int,
}

// original: reglist_t (htslib/regidx.c:66)
#[derive(Default)]
pub struct reglist_t {
    idx: Vec<u32>,
    reg: Vec<reg_t>,
    dat: Vec<u8>,
    seq: Option<NonNull<c_char>>,
    unsorted: c_int,
}

// original: regidx_t (htslib/regidx.c:77)
pub struct regidx_t {
    seq: Vec<reglist_t>,
    seq2regs: Option<NonNull<khash_s2i_t>>,
    seq_names: Vec<CString>,
    seq_name_ptrs: Vec<NonNull<c_char>>,
    free: regidx_free_f,
    parse: regidx_parse_f,
    usr: Option<NonNull<c_void>>,
    payload_size: c_int,
    payload: Vec<u8>,
    str: kstring_t,
}

struct OwnedKString {
    inner: kstring_t,
}

impl OwnedKString {
    fn new() -> Self {
        Self {
            inner: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
        }
    }

    fn as_mut(&mut self) -> &mut kstring_t {
        &mut self.inner
    }

    fn s(&self) -> *mut c_char {
        self.inner.s
    }
}

impl Drop for OwnedKString {
    fn drop(&mut self) {
        unsafe {
            ks_free(&mut self.inner);
        }
    }
}

pub type regidx_parse_f = Option<
    unsafe extern "C" fn(
        *const c_char,
        *mut *mut c_char,
        *mut *mut c_char,
        *mut hts_pos_t,
        *mut hts_pos_t,
        *mut c_void,
        *mut c_void,
    ) -> c_int,
>;
pub type regidx_free_f = Option<unsafe extern "C" fn(*mut c_void)>;

impl regidx_t {
    fn new(
        parse: regidx_parse_f,
        free: regidx_free_f,
        payload_size: usize,
        usr: Option<NonNull<c_void>>,
    ) -> Self {
        Self {
            seq: Vec::new(),
            seq2regs: None,
            seq_names: Vec::new(),
            seq_name_ptrs: Vec::new(),
            free,
            parse,
            usr,
            payload_size: payload_size as c_int,
            payload: if payload_size == 0 {
                Vec::new()
            } else {
                vec![0; payload_size]
            },
            str: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
        }
    }

    fn seq2regs_ptr(&self) -> *mut c_void {
        self.seq2regs
            .map_or(std::ptr::null_mut(), |ptr| ptr.as_ptr().cast())
    }

    fn payload_ptr(&mut self) -> *mut c_void {
        self.payload.as_mut_ptr().cast()
    }

    fn payload_nonnull(&mut self) -> Option<NonNull<c_void>> {
        (self.payload_size != 0)
            .then(|| NonNull::new(self.payload.as_mut_ptr().cast()))
            .flatten()
    }

    fn usr_ptr(&self) -> *mut c_void {
        self.usr.map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }
}

#[derive(Default)]
struct ParsedRegion {
    chr: Option<std::ops::RangeInclusive<usize>>,
    beg: hts_pos_t,
    end: hts_pos_t,
}

impl ParsedRegion {
    unsafe fn set_chr(&mut self, line: &CStr, beg: *const c_char, end: *const c_char) {
        let base = line.as_ptr();
        let chr_beg = beg.offset_from(base) as usize;
        let chr_end = end.offset_from(base) as usize;
        self.chr = Some(chr_beg..=chr_end);
    }

    unsafe fn write_to_raw(
        &self,
        line: &CStr,
        chr_beg: *mut *mut c_char,
        chr_end: *mut *mut c_char,
        beg: *mut hts_pos_t,
        end: *mut hts_pos_t,
    ) {
        let line = line.as_ptr().cast_mut();
        if let Some(chr) = &self.chr {
            *chr_beg = line.add(*chr.start());
            *chr_end = line.add(*chr.end());
        } else {
            *chr_beg = std::ptr::null_mut();
            *chr_end = std::ptr::null_mut();
        }
        *beg = self.beg;
        *end = self.end;
    }
}

impl Drop for regidx_t {
    fn drop(&mut self) {
        unsafe {
            ks_free(&mut self.str);
            if let Some(seq2regs) = self.seq2regs.take() {
                khash_str2int_destroy(seq2regs.as_ptr().cast());
            }
        }
    }
}

impl itr_t_ {
    fn new(regidx: NonNull<regidx_t>) -> Self {
        Self {
            beg: 0,
            end: 0,
            ireg: 0,
            ridx: regidx,
            list: None,
            active: 0,
        }
    }

    fn reset(&mut self, regidx: NonNull<regidx_t>) {
        *self = Self::new(regidx);
    }
}

unsafe fn regidx_seq_nregs(idx: &regidx_t, seq: &CStr) -> c_int {
    let mut iseq = 0;
    if khash_str2int_get(idx.seq2regs_ptr(), seq.as_ptr(), &mut iseq) != 0 {
        return 0;
    }
    idx.seq[iseq as usize].reg.len() as c_int
}

pub unsafe fn regidx_c_91_regidx_seq_nregs(idx: *mut regidx_t, seq: *const c_char) -> c_int {
    let Some(idx) = idx.as_ref() else {
        return 0;
    };
    if seq.is_null() {
        return 0;
    }
    regidx_seq_nregs(idx, CStr::from_ptr(seq))
}

fn regidx_nregs(idx: &regidx_t) -> c_int {
    idx.seq.iter().map(|list| list.reg.len() as c_int).sum()
}

pub unsafe fn regidx_c_98_regidx_nregs(idx: *mut regidx_t) -> c_int {
    idx.as_ref().map_or(0, regidx_nregs)
}

fn regidx_seq_names(idx: &mut regidx_t, n: &mut c_int) -> *mut *mut c_char {
    *n = idx.seq.len() as c_int;
    idx.seq_name_ptrs.as_mut_ptr().cast::<*mut c_char>()
}

pub unsafe fn regidx_c_105_regidx_seq_names(idx: *mut regidx_t, n: *mut c_int) -> *mut *mut c_char {
    let (Some(idx), Some(n)) = (idx.as_mut(), n.as_mut()) else {
        return std::ptr::null_mut();
    };
    regidx_seq_names(idx, n)
}

unsafe fn regidx_insert_list(idx: &mut regidx_t, line: NonNull<c_char>, delim: c_char) -> c_int {
    let mut tmp = OwnedKString::new();
    let mut ss = line.as_ptr();
    while *ss != 0 {
        let mut se = ss;
        while *se != 0 && *se != delim {
            se = se.add(1);
        }
        if kputsn(ss, se.offset_from(ss) as usize, ks_clear(tmp.as_mut())) < 0 {
            return -1;
        }
        let Some(tmp_line) = NonNull::new(tmp.s()) else {
            return -1;
        };
        if regidx_insert(idx, tmp_line) < 0 {
            return -1;
        }
        if *se == 0 {
            break;
        }
        ss = se.add(1);
    }
    0
}

pub unsafe fn regidx_c_111_regidx_insert_list(
    idx: *mut regidx_t,
    line: *mut c_char,
    delim: c_char,
) -> c_int {
    let (Some(idx), Some(line)) = (idx.as_mut(), NonNull::new(line)) else {
        return 0;
    };
    regidx_insert_list(idx, line, delim)
}

pub fn regidx_c_132_cmp_regs(a: &mut regidx_reg_t, b: &mut regidx_reg_t) -> c_int {
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
    let a_ptr = std::ptr::from_mut(a);
    let b_ptr = std::ptr::from_mut(b);
    if a_ptr < b_ptr {
        return -1;
    }
    if a_ptr > b_ptr {
        return 1;
    }
    0
}

pub unsafe extern "C" fn regidx_c_142_cmp_reg_ptrs(a: *const c_void, b: *const c_void) -> c_int {
    regidx_c_132_cmp_regs(
        &mut *(a.cast::<regidx_reg_t>() as *mut regidx_reg_t),
        &mut *(b.cast::<regidx_reg_t>() as *mut regidx_reg_t),
    )
}

pub unsafe extern "C" fn regidx_c_146_cmp_reg_ptrs2(a: *const c_void, b: *const c_void) -> c_int {
    let ap = *(a.cast::<*mut regidx_reg_t>());
    let bp = *(b.cast::<*mut regidx_reg_t>());
    regidx_c_132_cmp_regs(&mut *ap, &mut *bp)
}

fn cmp_reg_values(a: reg_t, b: reg_t) -> std::cmp::Ordering {
    a.beg.cmp(&b.beg).then_with(|| b.end.cmp(&a.end))
}

pub unsafe fn regidx_c_151_regidx_push(
    idx: &mut regidx_t,
    chr_beg: NonNull<c_char>,
    chr_end: NonNull<c_char>,
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    payload: Option<NonNull<c_void>>,
) -> c_int {
    beg = beg.clamp(0, MAX_COOR_0);
    end = end.clamp(0, MAX_COOR_0);

    let mut rid = 0;
    if kputsn(
        chr_beg.as_ptr(),
        chr_end.as_ptr().offset_from(chr_beg.as_ptr()) as usize + 1,
        ks_clear(&mut idx.str),
    ) < 0
    {
        return -1;
    }
    if khash_str2int_get(idx.seq2regs_ptr(), idx.str.s, &mut rid) != 0 {
        let Ok(seq_name) = CString::new(std::ffi::CStr::from_ptr(idx.str.s).to_bytes()) else {
            return -1;
        };
        idx.seq_names.push(seq_name);
        let seq_ptr = NonNull::new_unchecked(idx.seq_names.last().unwrap().as_ptr() as *mut c_char);
        idx.seq_name_ptrs.push(seq_ptr);
        idx.seq.push(reglist_t {
            seq: Some(seq_ptr),
            ..Default::default()
        });
        rid = khash_str2int_inc(idx.seq2regs_ptr(), seq_ptr.as_ptr());
        if rid < 0 {
            return -1;
        }
    }

    let payload_size = idx.payload_size as usize;
    let list = &mut idx.seq[rid as usize];
    list.seq = Some(idx.seq_name_ptrs[rid as usize]);
    let prev = list.reg.last().copied();
    list.reg.push(reg_t { beg, end });
    if payload_size != 0 {
        let Some(payload) = payload else {
            return -1;
        };
        let old_len = list.dat.len();
        let Some(new_len) = old_len.checked_add(payload_size) else {
            return -1;
        };
        list.dat.resize(new_len, 0);
        std::ptr::copy_nonoverlapping(
            payload.cast::<u8>().as_ptr(),
            list.dat[old_len..].as_mut_ptr(),
            payload_size,
        );
    }
    if list.unsorted == 0
        && prev.is_some()
        && cmp_reg_values(prev.unwrap(), *list.reg.last().unwrap()).is_gt()
    {
        list.unsorted = 1;
    }
    0
}

unsafe fn regidx_insert(idx: &mut regidx_t, line: NonNull<c_char>) -> c_int {
    let mut chr_from = std::ptr::null_mut();
    let mut chr_to = std::ptr::null_mut();
    let mut beg = 0;
    let mut end = 0;
    let payload = idx.payload_ptr();
    let payload_ref = idx.payload_nonnull();
    let ret = idx.parse.unwrap()(
        line.as_ptr(),
        &mut chr_from,
        &mut chr_to,
        &mut beg,
        &mut end,
        payload,
        idx.usr_ptr(),
    );
    if ret == -2 {
        return -1;
    }
    if ret == -1 {
        return 0;
    }
    let Some(chr_from) = NonNull::new(chr_from) else {
        return -1;
    };
    let Some(chr_to) = NonNull::new(chr_to) else {
        return -1;
    };
    regidx_c_151_regidx_push(idx, chr_from, chr_to, beg, end, payload_ref)
}

pub unsafe fn regidx_c_198_regidx_insert(idx: *mut regidx_t, line: *mut c_char) -> c_int {
    let (Some(idx), Some(line)) = (idx.as_mut(), NonNull::new(line)) else {
        return 0;
    };
    regidx_insert(idx, line)
}

pub unsafe fn regidx_c_209_regidx_init_string(
    string: *const c_char,
    parsef: regidx_parse_f,
    freef: regidx_free_f,
    payload_size: usize,
    usr: *mut c_void,
) -> *mut regidx_t {
    let mut tmp = OwnedKString::new();
    let parse = parsef.or(Some(
        regidx_c_498_regidx_parse_tab as unsafe extern "C" fn(_, _, _, _, _, _, _) -> _,
    ));
    let idx = Box::into_raw(Box::new(regidx_t::new(
        parse,
        freef,
        payload_size,
        NonNull::new(usr),
    )));
    (*idx).seq2regs = NonNull::new(khash_str2int_init().cast::<khash_s2i_t>());
    if (*idx).seq2regs.is_none() {
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }

    let mut ss = string;
    while *ss != 0 {
        while *ss != 0 && isspace_c(*ss) != 0 {
            ss = ss.add(1);
        }
        let mut se = ss;
        while *se != 0 && *se != b'\r' as c_char && *se != b'\n' as c_char {
            se = se.add(1);
        }
        if kputsn(ss, se.offset_from(ss) as usize, ks_clear(tmp.as_mut())) < 0 {
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        }
        let Some(tmp_line) = NonNull::new(tmp.s()) else {
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        };
        if regidx_insert(&mut *idx, tmp_line) < 0 {
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        }
        while *se != 0 && isspace_c(*se) != 0 {
            se = se.add(1);
        }
        ss = se;
    }
    idx
}

pub unsafe fn regidx_c_246_regidx_init(
    fname: *const c_char,
    mut parsef: regidx_parse_f,
    freef: regidx_free_f,
    payload_size: usize,
    usr: *mut c_void,
) -> *mut regidx_t {
    if parsef.is_none() {
        if fname.is_null() {
            parsef = Some(regidx_c_498_regidx_parse_tab);
        } else {
            let fname = CStr::from_ptr(fname).to_bytes();
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
                parsef = Some(regidx_c_466_regidx_parse_bed);
            } else if is_vcf {
                parsef = Some(regidx_c_538_regidx_parse_vcf);
            } else {
                parsef = Some(regidx_c_498_regidx_parse_tab);
            }
        }
    }

    let mut str_ = OwnedKString::new();
    let idx = Box::into_raw(Box::new(regidx_t::new(
        parsef,
        freef,
        payload_size,
        NonNull::new(usr),
    )));
    (*idx).seq2regs = NonNull::new(khash_str2int_init().cast::<khash_s2i_t>());
    if (*idx).seq2regs.is_none() {
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }

    if fname.is_null() {
        return idx;
    }

    let mut fp = hts_open(fname, c"r".as_ptr());
    if fp.is_null() {
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }

    let mut ret = hts_getline(fp, b'\n' as c_int, str_.as_mut());
    while ret > 0 {
        let Some(line) = NonNull::new(str_.s()) else {
            hts_close(fp);
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        };
        if regidx_insert(&mut *idx, line) != 0 {
            hts_close(fp);
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        }
        ret = hts_getline(fp, b'\n' as c_int, str_.as_mut());
    }
    if ret < -1 {
        hts_close(fp);
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }

    ret = hts_close(fp);
    fp = std::ptr::null_mut();
    if ret != 0 {
        let _ = fp;
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }
    idx
}

pub unsafe fn regidx_c_311_regidx_destroy(idx: *mut regidx_t) {
    if idx.is_null() {
        return;
    }
    let mut idx_box = Box::from_raw(idx);
    for list in &mut idx_box.seq {
        if let Some(free) = idx_box.free {
            let payload_size = idx_box.payload_size as usize;
            if payload_size != 0 {
                for chunk in list.dat.chunks_mut(payload_size) {
                    free(chunk.as_mut_ptr().cast());
                }
            }
        }
    }
    drop(idx_box);
}

// original: reglist_build_index_ (htslib/regidx.c:335)
pub fn regidx_c_335_reglist_build_index_(payload_size: usize, list: &mut reglist_t) -> c_int {
    if list.unsorted != 0 {
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
        list.unsorted = 0;
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

fn regitr_set_region(regitr: &mut regitr_t, list: &mut reglist_t, ireg: u32, payload_size: c_int) {
    regitr.seq = list.seq;
    regitr.beg = list.reg[ireg as usize].beg;
    regitr.end = list.reg[ireg as usize].end;
    regitr.payload = if payload_size != 0 {
        let payload = unsafe {
            list.dat
                .as_mut_ptr()
                .add(payload_size as usize * ireg as usize)
                .cast()
        };
        NonNull::new(payload)
    } else {
        None
    };
}

unsafe fn regidx_overlap(
    idx: &mut regidx_t,
    idx_ptr: NonNull<regidx_t>,
    chr: &CStr,
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    mut regitr: Option<&mut regitr_t>,
) -> c_int {
    if let Some(regitr) = regitr.as_deref_mut() {
        regitr.seq = None;
        regitr.payload = None;
    }
    beg = beg.max(0);
    end = end.clamp(0, MAX_COOR_0);

    let mut iseq = 0;
    if khash_str2int_get(idx.seq2regs_ptr(), chr.as_ptr(), &mut iseq) != 0 {
        return 0;
    }

    let list = &mut idx.seq[iseq as usize];
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
        if list.idx.is_empty()
            && regidx_c_335_reglist_build_index_(idx.payload_size as usize, list) < 0
        {
            return -1;
        }

        let ibeg = ibin(beg);
        if ibeg >= list.idx.len() as c_int {
            return 0;
        }

        let mut i = list.idx[ibeg as usize];
        if i == 0 {
            let mut iend = ibin(end);
            if iend > list.idx.len() as c_int {
                iend = list.idx.len() as c_int;
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

    let itr = &mut regitr.itr;
    itr.ridx = idx_ptr;
    itr.list = Some(NonNull::from(&mut *list));
    itr.beg = beg;
    itr.end = end;
    itr.ireg = ireg;
    itr.active = 0;
    regitr_set_region(regitr, list, ireg, idx.payload_size);

    1
}

pub unsafe fn regidx_c_401_regidx_overlap(
    idx: *mut regidx_t,
    chr: *const c_char,
    beg: hts_pos_t,
    end: hts_pos_t,
    itr: *mut regitr_t,
) -> c_int {
    let Some(mut idx_ptr) = NonNull::new(idx) else {
        return 0;
    };
    if chr.is_null() {
        return 0;
    }
    regidx_overlap(
        idx_ptr.as_mut(),
        idx_ptr,
        CStr::from_ptr(chr),
        beg,
        end,
        itr.as_mut(),
    )
}

unsafe fn regidx_parse_bed(line: &CStr, out: &mut ParsedRegion) -> c_int {
    let mut ss: *mut c_char = line.as_ptr().cast_mut();
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se: *mut c_char = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }

    out.set_chr(line, ss, se.sub(1));

    if *se == 0 {
        out.beg = 0;
        out.end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    out.beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }

    ss = se.add(1);
    out.end = hts_parse_decimal(ss, &mut se, 0) - 1;
    if ss == se {
        return -2;
    }

    0
}

pub unsafe extern "C" fn regidx_c_466_regidx_parse_bed(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    if line.is_null() {
        return -2;
    };
    let line = CStr::from_ptr(line);
    let mut out = ParsedRegion::default();
    let ret = regidx_parse_bed(line, &mut out);
    out.write_to_raw(line, chr_beg, chr_end, beg, end);
    ret
}

unsafe fn regidx_parse_tab(line: &CStr, out: &mut ParsedRegion) -> c_int {
    let mut ss: *mut c_char = line.as_ptr().cast_mut();
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se: *mut c_char = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }

    out.set_chr(line, ss, se.sub(1));

    if *se == 0 {
        out.beg = 0;
        out.end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    out.beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }
    if out.beg == 0 {
        return -2;
    }
    out.beg -= 1;

    if *se == 0 || *se.add(1) == 0 {
        out.end = out.beg;
    } else {
        ss = se.add(1);
        out.end = hts_parse_decimal(ss, &mut se, 0);
        if ss == se || (*se != 0 && isspace_c(*se) == 0) {
            out.end = out.beg;
        } else if out.end == 0 {
            return -2;
        } else {
            out.end -= 1;
        }
    }
    0
}

pub unsafe extern "C" fn regidx_c_498_regidx_parse_tab(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    if line.is_null() {
        return -2;
    };
    let line = CStr::from_ptr(line);
    let mut out = ParsedRegion::default();
    let ret = regidx_parse_tab(line, &mut out);
    out.write_to_raw(line, chr_beg, chr_end, beg, end);
    ret
}

unsafe fn regidx_parse_vcf(line: &CStr, out: &mut ParsedRegion) -> c_int {
    let ret = regidx_parse_tab(line, out);
    if ret == 0 {
        out.end = out.beg;
    }
    ret
}

pub unsafe extern "C" fn regidx_c_538_regidx_parse_vcf(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    if line.is_null() {
        return -2;
    };
    let line = CStr::from_ptr(line);
    let mut out = ParsedRegion::default();
    let ret = regidx_parse_vcf(line, &mut out);
    out.write_to_raw(line, chr_beg, chr_end, beg, end);
    ret
}

unsafe fn regidx_parse_reg(line: &CStr, out: &mut ParsedRegion) -> c_int {
    let mut ss: *mut c_char = line.as_ptr().cast_mut();
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se: *mut c_char = ss;
    while *se != 0 && *se != b':' as c_char {
        se = se.add(1);
    }

    out.set_chr(line, ss, se.sub(1));

    if *se == 0 {
        out.beg = 0;
        out.end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    out.beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }
    if out.beg == 0 {
        return -2;
    }
    out.beg -= 1;

    if *se == 0 || *se.add(1) == 0 {
        out.end = if *se == b'-' as c_char {
            MAX_COOR_0
        } else {
            out.beg
        };
    } else {
        ss = se.add(1);
        out.end = hts_parse_decimal(ss, &mut se, 0);
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

pub unsafe extern "C" fn regidx_c_545_regidx_parse_reg(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    _payload: *mut c_void,
    _usr: *mut c_void,
) -> c_int {
    if line.is_null() {
        return -2;
    };
    let line = CStr::from_ptr(line);
    let mut out = ParsedRegion::default();
    let ret = regidx_parse_reg(line, &mut out);
    out.write_to_raw(line, chr_beg, chr_end, beg, end);
    ret
}

fn regitr_new(regidx: NonNull<regidx_t>) -> regitr_t {
    regitr_t {
        beg: 0,
        end: 0,
        payload: None,
        seq: None,
        itr: Box::new(itr_t_::new(regidx)),
    }
}

pub unsafe fn regidx_c_584_regitr_init(regidx: *mut regidx_t) -> *mut regitr_t {
    let Some(regidx) = NonNull::new(regidx) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(regitr_new(regidx)))
}

fn regitr_reset(regidx: Option<NonNull<regidx_t>>, regitr: &mut regitr_t) {
    if let Some(regidx) = regidx {
        regitr.itr.reset(regidx);
    }
    regitr.beg = 0;
    regitr.end = 0;
    regitr.payload = None;
    regitr.seq = None;
}

pub unsafe fn regidx_c_599_regitr_reset(regidx: *mut regidx_t, regitr: *mut regitr_t) {
    let Some(regitr) = regitr.as_mut() else {
        return;
    };
    regitr_reset(NonNull::new(regidx), regitr);
}

pub unsafe fn regidx_c_606_regitr_destroy(regitr: *mut regitr_t) {
    if regitr.is_null() {
        return;
    }
    drop(Box::from_raw(regitr));
}

unsafe fn regitr_overlap(regitr: &mut regitr_t) -> c_int {
    if regitr.seq.is_none() {
        return 0;
    }

    let (mut list_ptr, i, payload_size) = {
        let itr = &mut regitr.itr;
        if itr.active == 0 {
            itr.active = 1;
            itr.ireg += 1;
            return 1;
        }

        let Some(list_ptr) = itr.list else {
            return 0;
        };
        let list = list_ptr.as_ref();
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
        let regidx = itr.ridx.as_ref();
        (list_ptr, i, regidx.payload_size)
    };

    let list = list_ptr.as_mut();
    regitr_set_region(regitr, list, i, payload_size);

    1
}

pub unsafe fn regidx_c_612_regitr_overlap(regitr: *mut regitr_t) -> c_int {
    let Some(regitr) = regitr.as_mut() else {
        return 0;
    };
    regitr_overlap(regitr)
}

unsafe fn regitr_loop(regitr: &mut regitr_t) -> c_int {
    let (mut list_ptr, ireg, payload_size) = {
        let itr = &mut regitr.itr;
        let regidx = itr.ridx.as_mut();

        if regidx.seq.is_empty() {
            return 0;
        }

        if itr.list.is_none() {
            itr.list = Some(NonNull::from(&mut regidx.seq[0]));
            itr.ireg = 0;
        }

        let list_ptr = itr.list.unwrap().as_ptr();
        let mut iseq = regidx
            .seq
            .iter()
            .position(|list| std::ptr::addr_eq(list as *const reglist_t, list_ptr.cast_const()))
            .unwrap_or(regidx.seq.len());
        if iseq >= regidx.seq.len() {
            return 0;
        }

        if (itr.ireg as usize) >= regidx.seq[iseq].reg.len() {
            iseq += 1;
            if iseq >= regidx.seq.len() {
                return 0;
            }
            itr.ireg = 0;
            itr.list = Some(NonNull::from(&mut regidx.seq[iseq]));
        }

        let list_ptr = itr.list.unwrap();
        let ireg = itr.ireg;
        itr.ireg += 1;
        (list_ptr, ireg, regidx.payload_size)
    };

    let list = list_ptr.as_mut();
    regitr_set_region(regitr, list, ireg, payload_size);

    1
}

pub unsafe fn regidx_c_646_regitr_loop(regitr: *mut regitr_t) -> c_int {
    let Some(regitr) = regitr.as_mut() else {
        return 0;
    };
    regitr_loop(regitr)
}

fn regitr_copy(dst: &mut regitr_t, src: &regitr_t) {
    dst.beg = src.beg;
    dst.end = src.end;
    dst.payload = src.payload;
    dst.seq = src.seq;
    dst.itr = Box::new(*src.itr);
}

pub unsafe fn regidx_c_681_regitr_copy(dst: *mut regitr_t, src: *mut regitr_t) {
    if std::ptr::addr_eq(dst, src) {
        return;
    }
    let (Some(dst), Some(src)) = (dst.as_mut(), src.as_ref()) else {
        return;
    };
    regitr_copy(dst, src);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn regidx_parse_bed_uses_zero_based_right_open_coordinates() {
        unsafe {
            let line = CString::new("chr1\t10\t20").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;
            assert_eq!(
                regidx_c_466_regidx_parse_bed(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!(beg, 10);
            assert_eq!(end, 19);
            assert_eq!(chr_end.offset_from(chr_beg), 3);
        }
    }

    #[test]
    fn regidx_parse_bed_accepts_contig_only_and_rejects_bad_coordinates() {
        unsafe {
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            let whole_contig = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_466_regidx_parse_bed(
                    whole_contig.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (0, MAX_COOR_0));
            assert_eq!(chr_end.offset_from(chr_beg), 3);

            for line in ["chr1 start 20", "chr1 10 end"] {
                let line = CString::new(line).unwrap();
                assert_eq!(
                    regidx_c_466_regidx_parse_bed(
                        line.as_ptr(),
                        &mut chr_beg,
                        &mut chr_end,
                        &mut beg,
                        &mut end,
                        std::ptr::null_mut(),
                        std::ptr::null_mut()
                    ),
                    -2
                );
            }
        }
    }

    #[test]
    fn regidx_parse_tab_and_vcf_use_one_based_positions() {
        unsafe {
            let line = CString::new("chr2 11 14").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (10, 13));

            assert_eq!(
                regidx_c_538_regidx_parse_vcf(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (10, 10));
        }
    }

    #[test]
    fn regidx_parse_tab_boundary_cases_match_htslib_fallbacks() {
        unsafe {
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            let whole_contig = CString::new("  chr1").unwrap();
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    whole_contig.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (0, MAX_COOR_0));
            assert_eq!(chr_end.offset_from(chr_beg), 3);

            let trailing_dash = CString::new("chr2 5 -").unwrap();
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    trailing_dash.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (4, 4));

            let junk_after_end = CString::new("chr2 5 9x").unwrap();
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    junk_after_end.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (4, 4));
        }
    }

    #[test]
    fn regidx_parse_tab_and_vcf_reject_zero_start_coordinates() {
        unsafe {
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            let line = CString::new("chr1 0 10").unwrap();
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -2
            );

            let line = CString::new("chr1 0 .").unwrap();
            assert_eq!(
                regidx_c_538_regidx_parse_vcf(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -2
            );
        }
    }

    #[test]
    fn regidx_parse_and_insert_distinguish_skip_from_malformed_lines() {
        unsafe {
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            let comment = CString::new("#comment").unwrap();
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    comment.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -1
            );

            let zero_pos = CString::new("chr1 0").unwrap();
            assert_eq!(
                regidx_c_498_regidx_parse_tab(
                    zero_pos.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -2
            );

            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut comment_line = CString::new("#comment").unwrap().into_bytes_with_nul();
            assert_eq!(
                regidx_c_198_regidx_insert(idx, comment_line.as_mut_ptr().cast()),
                0
            );

            let mut malformed_line = CString::new("chr1 0").unwrap().into_bytes_with_nul();
            assert_eq!(
                regidx_c_198_regidx_insert(idx, malformed_line.as_mut_ptr().cast()),
                -1
            );

            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_parse_reg_handles_open_ended_regions() {
        unsafe {
            let line = CString::new("chr3:42-").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (41, MAX_COOR_0));
        }
    }

    #[test]
    fn regidx_parse_reg_handles_whole_contig_point_and_empty_range_edges() {
        unsafe {
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            let whole = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    whole.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(chr_beg).to_bytes(), b"chr1");
            assert_eq!(chr_end.offset_from(chr_beg), 3);
            assert_eq!((beg, end), (0, MAX_COOR_0));

            let point = CString::new("chr1:7").unwrap();
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    point.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (6, 6));

            let empty_range = CString::new("chr1:").unwrap();
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    empty_range.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -2
            );
        }
    }

    #[test]
    fn regidx_parse_reg_rejects_zero_start_and_collapses_bad_end_to_point() {
        unsafe {
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            let zero = CString::new("chr1:0-10").unwrap();
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    zero.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -2
            );

            let bad_end = CString::new("chr1:5-end").unwrap();
            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    bad_end.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            assert_eq!((beg, end), (4, 4));
        }
    }

    #[test]
    fn regidx_parse_reg_stops_sequence_name_at_first_colon() {
        unsafe {
            let line = CString::new("chr1:alt:5-7").unwrap();
            let mut chr_beg = std::ptr::null_mut();
            let mut chr_end = std::ptr::null_mut();
            let mut beg = -1;
            let mut end = -1;

            assert_eq!(
                regidx_c_545_regidx_parse_reg(
                    line.as_ptr(),
                    &mut chr_beg,
                    &mut chr_end,
                    &mut beg,
                    &mut end,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                -2
            );
            assert_eq!(
                std::slice::from_raw_parts(
                    chr_beg.cast::<u8>(),
                    chr_end.offset_from(chr_beg) as usize + 1,
                ),
                b"chr1"
            );
        }
    }

    #[test]
    fn regidx_comparators_order_by_begin_then_longer_end_then_address() {
        unsafe {
            let mut a = regidx_reg_t { beg: 10, end: 20 };
            let mut b = regidx_reg_t { beg: 10, end: 25 };
            let mut c = regidx_reg_t { beg: 11, end: 12 };

            assert_eq!(regidx_c_132_cmp_regs(&mut a, &mut b), 1);
            assert_eq!(regidx_c_132_cmp_regs(&mut c, &mut b), 1);
            assert_eq!(
                regidx_c_142_cmp_reg_ptrs(
                    (&mut b as *mut regidx_reg_t).cast(),
                    (&mut a as *mut regidx_reg_t).cast()
                ),
                -1
            );

            let mut ap = &mut a as *mut regidx_reg_t;
            let mut bp = &mut b as *mut regidx_reg_t;
            assert_eq!(
                regidx_c_146_cmp_reg_ptrs2(
                    (&mut bp as *mut *mut regidx_reg_t).cast(),
                    (&mut ap as *mut *mut regidx_reg_t).cast()
                ),
                -1
            );
        }
    }

    #[test]
    fn regidx_public_api_builds_queries_and_iterates_regions() {
        unsafe {
            let data = CString::new("chr1\t5\t7\nchr1\t9\t10\nchr2\t3\n").unwrap();
            let idx = regidx_c_209_regidx_init_string(
                data.as_ptr(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let chr1 = CString::new("chr1").unwrap();
            let chr3 = CString::new("chr3").unwrap();
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr1.as_ptr()), 2);
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr3.as_ptr()), 0);
            assert_eq!(regidx_c_98_regidx_nregs(idx), 3);

            let mut n = 0;
            let names = regidx_c_105_regidx_seq_names(idx, &mut n);
            assert_eq!(n, 2);
            assert!(!names.is_null());

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, chr1.as_ptr(), 4, 4, itr),
                1
            );
            assert_eq!((*itr).beg, 4);
            assert_eq!((*itr).end, 6);
            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(regidx_c_612_regitr_overlap(itr), 0);

            regidx_c_599_regitr_reset(idx, itr);
            assert_eq!(regidx_c_646_regitr_loop(itr), 1);
            assert_eq!((*itr).beg, 4);
            let copy = regidx_c_584_regitr_init(idx);
            assert!(!copy.is_null());
            regidx_c_681_regitr_copy(copy, itr);
            assert_eq!((*copy).beg, (*itr).beg);

            regidx_c_606_regitr_destroy(copy);
            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_loop_on_empty_index_returns_no_regions() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            assert_eq!(regidx_c_646_regitr_loop(itr), 0);

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_iterator_copy_keeps_independent_overlap_position() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            for (beg, end) in [(0, 0), (5, 5), (10, 10)] {
                assert_eq!(
                    regidx_c_151_regidx_push(
                        &mut *idx,
                        NonNull::new(chr_beg).unwrap(),
                        NonNull::new(chr_end).unwrap(),
                        beg,
                        end,
                        None
                    ),
                    0
                );
            }

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 0, 10, itr),
                1
            );
            assert_eq!(((*itr).beg, (*itr).end), (0, 0));

            let copy = regidx_c_584_regitr_init(idx);
            assert!(!copy.is_null());
            regidx_c_681_regitr_copy(copy, itr);

            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(((*itr).beg, (*itr).end), (0, 0));
            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(((*itr).beg, (*itr).end), (5, 5));

            assert_eq!(regidx_c_612_regitr_overlap(copy), 1);
            assert_eq!(((*copy).beg, (*copy).end), (0, 0));
            assert_eq!(regidx_c_612_regitr_overlap(copy), 1);
            assert_eq!(((*copy).beg, (*copy).end), (5, 5));

            regidx_c_606_regitr_destroy(copy);
            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_iterator_self_copy_matches_c_struct_assignment() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            for (beg, end) in [(0, 0), (5, 5)] {
                assert_eq!(
                    regidx_c_151_regidx_push(
                        &mut *idx,
                        NonNull::new(chr_beg).unwrap(),
                        NonNull::new(chr_end).unwrap(),
                        beg,
                        end,
                        None
                    ),
                    0
                );
            }

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 0, 5, itr),
                1
            );
            regidx_c_681_regitr_copy(itr, itr);

            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(((*itr).beg, (*itr).end), (0, 0));
            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(((*itr).beg, (*itr).end), (5, 5));

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_unsorted_payloads_stay_attached_to_regions_after_index_build() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<c_int>(),
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            for (beg, end, mut payload) in [(30, 30, 30), (10, 10, 10), (20, 20, 20)] {
                assert_eq!(
                    regidx_c_151_regidx_push(
                        &mut *idx,
                        NonNull::new(chr_beg).unwrap(),
                        NonNull::new(chr_end).unwrap(),
                        beg,
                        end,
                        NonNull::new((&mut payload as *mut c_int).cast())
                    ),
                    0
                );
            }

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());

            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 10, 10, itr),
                1
            );
            assert_eq!(((*itr).beg, (*itr).end), (10, 10));
            assert_eq!(
                *((*itr)
                    .payload
                    .expect("regidx payload")
                    .cast::<c_int>()
                    .as_ptr()),
                10
            );

            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 30, 30, itr),
                1
            );
            assert_eq!(((*itr).beg, (*itr).end), (30, 30));
            assert_eq!(
                *((*itr)
                    .payload
                    .expect("regidx payload")
                    .cast::<c_int>()
                    .as_ptr()),
                30
            );

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_overlap_iteration_uses_sorted_interval_order_with_payloads() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<c_int>(),
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            for (beg, end, mut payload) in [(10, 12, 12), (5, 5, 5), (10, 20, 20)] {
                assert_eq!(
                    regidx_c_151_regidx_push(
                        &mut *idx,
                        NonNull::new(chr_beg).unwrap(),
                        NonNull::new(chr_end).unwrap(),
                        beg,
                        end,
                        NonNull::new((&mut payload as *mut c_int).cast())
                    ),
                    0
                );
            }

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 0, 30, itr),
                1
            );

            let mut seen = Vec::new();
            while regidx_c_612_regitr_overlap(itr) != 0 {
                seen.push((
                    (*itr).beg,
                    (*itr).end,
                    *((*itr)
                        .payload
                        .expect("regidx payload")
                        .cast::<c_int>()
                        .as_ptr()),
                ));
            }

            assert_eq!(seen, [(5, 5, 5), (10, 20, 20), (10, 12, 12)]);

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_overlap_iterator_includes_touching_bin_edges_and_clears_misses() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<c_int>(),
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            for (beg, end, mut payload) in [(0, 8191, 1), (8192, 8192, 2), (20000, 20010, 3)] {
                assert_eq!(
                    regidx_c_151_regidx_push(
                        &mut *idx,
                        NonNull::new(chr_beg).unwrap(),
                        NonNull::new(chr_end).unwrap(),
                        beg,
                        end,
                        NonNull::new((&mut payload as *mut c_int).cast())
                    ),
                    0
                );
            }

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 8191, 8192, itr),
                1
            );

            let mut seen = Vec::new();
            while regidx_c_612_regitr_overlap(itr) != 0 {
                seen.push((
                    (*itr).beg,
                    (*itr).end,
                    *((*itr)
                        .payload
                        .expect("regidx payload")
                        .cast::<c_int>()
                        .as_ptr()),
                ));
            }
            assert_eq!(seen, [(0, 8191, 1), (8192, 8192, 2)]);

            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 8193, 19999, itr),
                0
            );
            assert!((*itr).seq.is_none());

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_overlap_treats_interval_ends_as_inclusive_points() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            assert_eq!(
                regidx_c_151_regidx_push(
                    &mut *idx,
                    NonNull::new(chr_beg).unwrap(),
                    NonNull::new(chr_end).unwrap(),
                    10,
                    20,
                    None
                ),
                0
            );

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 20, 20, itr),
                1
            );
            assert_eq!(((*itr).beg, (*itr).end), (10, 20));
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 21, 21, itr),
                0
            );
            assert!((*itr).seq.is_none());

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_loop_preserves_sequence_and_insertion_order_without_overlap_query() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_545_regidx_parse_reg),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut list = CString::new("chrB:3,chrA:2-4,chrB:1")
                .unwrap()
                .into_bytes_with_nul();
            assert_eq!(
                regidx_c_111_regidx_insert_list(idx, list.as_mut_ptr().cast(), b',' as c_char),
                0
            );

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let mut seen = Vec::new();
            while regidx_c_646_regitr_loop(itr) != 0 {
                seen.push((
                    CStr::from_ptr((*itr).seq.expect("regidx seq").as_ptr())
                        .to_bytes()
                        .to_vec(),
                    (*itr).beg,
                    (*itr).end,
                ));
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
    }

    #[test]
    fn regidx_insert_list_skips_empty_items_like_other_ignored_lines() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_545_regidx_parse_reg),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut list = CString::new("chr1:1,,chr1:3,#comment,chr2")
                .unwrap()
                .into_bytes_with_nul();
            assert_eq!(
                regidx_c_111_regidx_insert_list(idx, list.as_mut_ptr().cast(), b',' as c_char),
                0
            );

            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr1.as_ptr()), 2);
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr2.as_ptr()), 1);
            assert_eq!(regidx_c_98_regidx_nregs(idx), 3);

            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_init_string_skips_comments_and_aborts_on_malformed_records() {
        unsafe {
            let good = CString::new("  #comment\n\nchr1\t1\t3\r\nchr2\t5\n").unwrap();
            let idx = regidx_c_209_regidx_init_string(
                good.as_ptr(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr1.as_ptr()), 1);
            assert_eq!(regidx_c_91_regidx_seq_nregs(idx, chr2.as_ptr()), 1);
            assert_eq!(regidx_c_98_regidx_nregs(idx), 2);
            regidx_c_311_regidx_destroy(idx);

            let bad = CString::new("chr1\t1\nchr2\t0\n").unwrap();
            let idx = regidx_c_209_regidx_init_string(
                bad.as_ptr(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(idx.is_null());
        }
    }

    #[test]
    fn regidx_push_clamps_coordinates_to_htslib_regidx_bounds() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_545_regidx_parse_reg),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chrC").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            assert_eq!(
                regidx_c_151_regidx_push(
                    &mut *idx,
                    NonNull::new(chr_beg).unwrap(),
                    NonNull::new(chr_end).unwrap(),
                    -10,
                    MAX_COOR_0 + 99,
                    None
                ),
                0
            );

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chrC").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), MAX_COOR_0, MAX_COOR_0, itr),
                1
            );
            assert_eq!(((*itr).beg, (*itr).end), (0, MAX_COOR_0));
            assert_eq!(
                regidx_c_401_regidx_overlap(
                    idx,
                    query_chr.as_ptr(),
                    0,
                    crate::htslib_rs::hts::HTS_POS_MAX,
                    itr
                ),
                1
            );
            assert_eq!(regidx_c_612_regitr_overlap(itr), 1);
            assert_eq!(regidx_c_612_regitr_overlap(itr), 0);

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_loop_uses_sorted_payload_order_after_overlap_builds_index() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_498_regidx_parse_tab),
                None,
                std::mem::size_of::<c_int>(),
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut chr = CString::new("chr1").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            for (beg, end, mut payload) in [(50, 50, 50), (10, 20, 20), (10, 30, 30)] {
                assert_eq!(
                    regidx_c_151_regidx_push(
                        &mut *idx,
                        NonNull::new(chr_beg).unwrap(),
                        NonNull::new(chr_end).unwrap(),
                        beg,
                        end,
                        NonNull::new((&mut payload as *mut c_int).cast())
                    ),
                    0
                );
            }

            let itr = regidx_c_584_regitr_init(idx);
            assert!(!itr.is_null());
            let query_chr = CString::new("chr1").unwrap();
            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 0, 60, itr),
                1
            );

            regidx_c_599_regitr_reset(idx, itr);
            let mut seen = Vec::new();
            while regidx_c_646_regitr_loop(itr) != 0 {
                seen.push((
                    (*itr).beg,
                    (*itr).end,
                    *((*itr)
                        .payload
                        .expect("regidx payload")
                        .cast::<c_int>()
                        .as_ptr()),
                ));
            }
            assert_eq!(seen, [(10, 30, 30), (10, 20, 20), (50, 50, 50)]);

            regidx_c_606_regitr_destroy(itr);
            regidx_c_311_regidx_destroy(idx);
        }
    }

    #[test]
    fn regidx_init_insert_push_and_insert_list_match_public_api_edges() {
        unsafe {
            let idx = regidx_c_246_regidx_init(
                std::ptr::null(),
                Some(regidx_c_545_regidx_parse_reg),
                None,
                0,
                std::ptr::null_mut(),
            );
            assert!(!idx.is_null());

            let mut line = CString::new("chrX:2-4").unwrap().into_bytes_with_nul();
            assert_eq!(regidx_c_198_regidx_insert(idx, line.as_mut_ptr().cast()), 0);

            let mut list = CString::new("chrX:8-9,chrY:1")
                .unwrap()
                .into_bytes_with_nul();
            assert_eq!(
                regidx_c_111_regidx_insert_list(idx, list.as_mut_ptr().cast(), b',' as c_char),
                0
            );

            let mut chr = CString::new("chrZ").unwrap().into_bytes_with_nul();
            let chr_beg = chr.as_mut_ptr().cast::<c_char>();
            let chr_end = chr_beg.add(3);
            assert_eq!(
                regidx_c_151_regidx_push(
                    &mut *idx,
                    NonNull::new(chr_beg).unwrap(),
                    NonNull::new(chr_end).unwrap(),
                    0,
                    2,
                    None
                ),
                0
            );

            assert_eq!(regidx_c_98_regidx_nregs(idx), 4);
            regidx_c_311_regidx_destroy(idx);
        }
    }
}
