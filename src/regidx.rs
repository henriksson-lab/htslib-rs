use std::ffi::{c_char, c_int, c_void};

use crate::htslib_rs::{
    hts::{
        hts_close, hts_getline, hts_open, hts_parse_decimal, hts_pos_t, hts_resize_array_,
        isspace_c, kputsn, ks_clear, ks_free, kstring_t, HTS_RESIZE_CLEAR,
    },
    sam::{khash_str2int_destroy_free, khash_str2int_get, khash_str2int_inc, khash_str2int_init},
};

pub const REGIDX_MAX: hts_pos_t = 1_i64 << 35;
pub const MAX_COOR_0: hts_pos_t = REGIDX_MAX;

fn ibin(x: hts_pos_t) -> c_int {
    (x >> 13) as c_int
}

unsafe fn hts_resize_i32<T>(num: usize, size: *mut c_int, ptr: *mut *mut T, flags: c_int) -> c_int {
    if num <= *size as usize {
        return 0;
    }
    hts_resize_array_(
        std::mem::size_of::<T>(),
        num,
        std::mem::size_of::<c_int>(),
        size.cast(),
        ptr.cast::<*mut c_void>(),
        flags,
        c"regidx".as_ptr(),
    )
}

unsafe fn hts_resize_u32<T>(num: usize, size: *mut u32, ptr: *mut *mut T, flags: c_int) -> c_int {
    if num <= *size as usize {
        return 0;
    }
    hts_resize_array_(
        std::mem::size_of::<T>(),
        num,
        std::mem::size_of::<u32>(),
        size.cast(),
        ptr.cast::<*mut c_void>(),
        flags,
        c"regidx".as_ptr(),
    )
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
    pub payload: *mut c_void,
    pub seq: *mut c_char,
    pub itr: *mut c_void,
}

// original: itr_t_ (htslib/regidx.c:53)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itr_t_ {
    beg: hts_pos_t,
    end: hts_pos_t,
    ireg: u32,
    ridx: *mut regidx_t,
    list: *mut reglist_t,
    active: c_int,
}

// original: reglist_t (htslib/regidx.c:66)
#[repr(C)]
pub struct reglist_t {
    idx: *mut u32,
    nidx: u32,
    nreg: u32,
    mreg: u32,
    reg: *mut reg_t,
    dat: *mut u8,
    seq: *mut c_char,
    unsorted: c_int,
}

// original: regidx_t (htslib/regidx.c:77)
#[repr(C)]
pub struct regidx_t {
    nseq: c_int,
    mseq: c_int,
    seq: *mut reglist_t,
    seq2regs: *mut c_void,
    seq_names: *mut *mut c_char,
    free: regidx_free_f,
    parse: regidx_parse_f,
    usr: *mut c_void,
    payload_size: c_int,
    payload: *mut c_void,
    str: kstring_t,
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

pub unsafe fn regidx_c_91_regidx_seq_nregs(idx: *mut regidx_t, seq: *const c_char) -> c_int {
    let mut iseq = 0;
    if khash_str2int_get((*idx).seq2regs, seq, &mut iseq) != 0 {
        return 0;
    }
    (*(*idx).seq.add(iseq as usize)).nreg as c_int
}

pub unsafe fn regidx_c_98_regidx_nregs(idx: *mut regidx_t) -> c_int {
    let mut nreg = 0;
    for i in 0..(*idx).nseq {
        nreg += (*(*idx).seq.add(i as usize)).nreg as c_int;
    }
    nreg
}

pub unsafe fn regidx_c_105_regidx_seq_names(idx: *mut regidx_t, n: *mut c_int) -> *mut *mut c_char {
    *n = (*idx).nseq;
    (*idx).seq_names
}

pub unsafe fn regidx_c_111_regidx_insert_list(
    idx: *mut regidx_t,
    line: *mut c_char,
    delim: c_char,
) -> c_int {
    let mut tmp = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let mut ss = line;
    while *ss != 0 {
        let mut se = ss;
        while *se != 0 && *se != delim {
            se = se.add(1);
        }
        if kputsn(ss, se.offset_from(ss) as usize, ks_clear(&mut tmp)) < 0 {
            ks_free(&mut tmp);
            return -1;
        }
        if regidx_c_198_regidx_insert(idx, tmp.s) < 0 {
            ks_free(&mut tmp);
            return -1;
        }
        if *se == 0 {
            break;
        }
        ss = se.add(1);
    }
    ks_free(&mut tmp);
    0
}

pub unsafe fn regidx_c_132_cmp_regs(a: *mut regidx_reg_t, b: *mut regidx_reg_t) -> c_int {
    if (*a).beg < (*b).beg {
        return -1;
    }
    if (*a).beg > (*b).beg {
        return 1;
    }
    if (*a).end < (*b).end {
        return 1;
    }
    if (*a).end > (*b).end {
        return -1;
    }
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    0
}

pub unsafe extern "C" fn regidx_c_142_cmp_reg_ptrs(a: *const c_void, b: *const c_void) -> c_int {
    regidx_c_132_cmp_regs(
        a.cast::<regidx_reg_t>() as *mut _,
        b.cast::<regidx_reg_t>() as *mut _,
    )
}

pub unsafe extern "C" fn regidx_c_146_cmp_reg_ptrs2(a: *const c_void, b: *const c_void) -> c_int {
    let ap = *(a.cast::<*mut regidx_reg_t>());
    let bp = *(b.cast::<*mut regidx_reg_t>());
    regidx_c_132_cmp_regs(ap, bp)
}

pub unsafe fn regidx_c_151_regidx_push(
    idx: *mut regidx_t,
    chr_beg: *mut c_char,
    chr_end: *mut c_char,
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    payload: *mut c_void,
) -> c_int {
    if beg < 0 {
        beg = 0;
    }
    if end < 0 {
        end = 0;
    }
    if beg > MAX_COOR_0 {
        beg = MAX_COOR_0;
    }
    if end > MAX_COOR_0 {
        end = MAX_COOR_0;
    }

    let mut rid = 0;
    if kputsn(
        chr_beg,
        chr_end.offset_from(chr_beg) as usize + 1,
        ks_clear(&mut (*idx).str),
    ) < 0
    {
        return -1;
    }
    if khash_str2int_get((*idx).seq2regs, (*idx).str.s, &mut rid) != 0 {
        let nseq = (*idx).nseq as usize;
        let mut m_tmp = (*idx).mseq;
        if hts_resize_i32::<*mut c_char>(
            nseq + 1,
            &mut m_tmp,
            &mut (*idx).seq_names,
            HTS_RESIZE_CLEAR,
        ) < 0
        {
            return -1;
        }
        if hts_resize_i32::<reglist_t>(
            nseq + 1,
            &mut (*idx).mseq,
            &mut (*idx).seq,
            HTS_RESIZE_CLEAR,
        ) < 0
        {
            return -1;
        }
        *(*idx).seq_names.add(nseq) = libc::strdup((*idx).str.s);
        if (*(*idx).seq_names.add(nseq)).is_null() {
            return -1;
        }
        rid = khash_str2int_inc((*idx).seq2regs, *(*idx).seq_names.add(nseq));
        if rid < 0 {
            return -1;
        }
        (*idx).nseq += 1;
    }

    let list = (*idx).seq.add(rid as usize);
    (*list).seq = *(*idx).seq_names.add(rid as usize);
    let mreg = (*list).mreg;
    if hts_resize_u32::<reg_t>(
        (*list).nreg as usize + 1,
        &mut (*list).mreg,
        &mut (*list).reg,
        0,
    ) < 0
    {
        return -1;
    }
    (*(*list).reg.add((*list).nreg as usize)).beg = beg;
    (*(*list).reg.add((*list).nreg as usize)).end = end;
    if (*idx).payload_size != 0 {
        if mreg != (*list).mreg {
            let Some(bytes) = ((*idx).payload_size as usize).checked_mul((*list).mreg as usize)
            else {
                return -1;
            };
            let new_dat = libc::realloc((*list).dat.cast(), bytes).cast::<u8>();
            if new_dat.is_null() {
                return -1;
            }
            (*list).dat = new_dat;
        }
        libc::memcpy(
            (*list)
                .dat
                .add((*idx).payload_size as usize * (*list).nreg as usize)
                .cast(),
            payload,
            (*idx).payload_size as usize,
        );
    }
    (*list).nreg += 1;
    if (*list).unsorted == 0
        && (*list).nreg > 1
        && regidx_c_132_cmp_regs(
            (*list).reg.add((*list).nreg as usize - 2),
            (*list).reg.add((*list).nreg as usize - 1),
        ) > 0
    {
        (*list).unsorted = 1;
    }
    0
}

pub unsafe fn regidx_c_198_regidx_insert(idx: *mut regidx_t, line: *mut c_char) -> c_int {
    if line.is_null() {
        return 0;
    }
    let mut chr_from = std::ptr::null_mut();
    let mut chr_to = std::ptr::null_mut();
    let mut beg = 0;
    let mut end = 0;
    let ret = (*idx).parse.unwrap()(
        line,
        &mut chr_from,
        &mut chr_to,
        &mut beg,
        &mut end,
        (*idx).payload,
        (*idx).usr,
    );
    if ret == -2 {
        return -1;
    }
    if ret == -1 {
        return 0;
    }
    regidx_c_151_regidx_push(idx, chr_from, chr_to, beg, end, (*idx).payload)
}

pub unsafe fn regidx_c_209_regidx_init_string(
    string: *const c_char,
    parsef: regidx_parse_f,
    freef: regidx_free_f,
    payload_size: usize,
    usr: *mut c_void,
) -> *mut regidx_t {
    let mut tmp = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let idx = libc::calloc(1, std::mem::size_of::<regidx_t>()).cast::<regidx_t>();
    if idx.is_null() {
        return std::ptr::null_mut();
    }
    (*idx).free = freef;
    (*idx).parse = if parsef.is_some() {
        parsef
    } else {
        Some(regidx_c_498_regidx_parse_tab)
    };
    (*idx).usr = usr;
    (*idx).seq2regs = khash_str2int_init();
    if (*idx).seq2regs.is_null() {
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }
    (*idx).payload_size = payload_size as c_int;
    if payload_size != 0 {
        (*idx).payload = libc::malloc(payload_size);
        if (*idx).payload.is_null() {
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        }
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
        if kputsn(ss, se.offset_from(ss) as usize, ks_clear(&mut tmp)) < 0 {
            regidx_c_311_regidx_destroy(idx);
            ks_free(&mut tmp);
            return std::ptr::null_mut();
        }
        if regidx_c_198_regidx_insert(idx, tmp.s) < 0 {
            regidx_c_311_regidx_destroy(idx);
            ks_free(&mut tmp);
            return std::ptr::null_mut();
        }
        while *se != 0 && isspace_c(*se) != 0 {
            se = se.add(1);
        }
        ss = se;
    }
    ks_free(&mut tmp);
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
            let len = libc::strlen(fname);
            if (len >= 7 && libc::strcasecmp(c".bed.gz".as_ptr(), fname.add(len - 7)) == 0)
                || (len >= 8 && libc::strcasecmp(c".bed.bgz".as_ptr(), fname.add(len - 8)) == 0)
                || (len >= 4 && libc::strcasecmp(c".bed".as_ptr(), fname.add(len - 4)) == 0)
            {
                parsef = Some(regidx_c_466_regidx_parse_bed);
            } else if (len >= 4 && libc::strcasecmp(c".vcf".as_ptr(), fname.add(len - 4)) == 0)
                || (len >= 7 && libc::strcasecmp(c".vcf.gz".as_ptr(), fname.add(len - 7)) == 0)
            {
                parsef = Some(regidx_c_538_regidx_parse_vcf);
            } else {
                parsef = Some(regidx_c_498_regidx_parse_tab);
            }
        }
    }

    let mut str_ = kstring_t {
        l: 0,
        m: 0,
        s: std::ptr::null_mut(),
    };
    let idx = libc::calloc(1, std::mem::size_of::<regidx_t>()).cast::<regidx_t>();
    if idx.is_null() {
        return std::ptr::null_mut();
    }
    (*idx).free = freef;
    (*idx).parse = parsef;
    (*idx).usr = usr;
    (*idx).seq2regs = khash_str2int_init();
    if (*idx).seq2regs.is_null() {
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }
    (*idx).payload_size = payload_size as c_int;
    if payload_size != 0 {
        (*idx).payload = libc::malloc(payload_size);
        if (*idx).payload.is_null() {
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        }
    }

    if fname.is_null() {
        return idx;
    }

    let mut fp = hts_open(fname, c"r".as_ptr());
    if fp.is_null() {
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }

    let mut ret = hts_getline(fp, b'\n' as c_int, &mut str_);
    while ret > 0 {
        if regidx_c_198_regidx_insert(idx, str_.s) != 0 {
            ks_free(&mut str_);
            hts_close(fp);
            regidx_c_311_regidx_destroy(idx);
            return std::ptr::null_mut();
        }
        ret = hts_getline(fp, b'\n' as c_int, &mut str_);
    }
    if ret < -1 {
        ks_free(&mut str_);
        hts_close(fp);
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }

    ret = hts_close(fp);
    fp = std::ptr::null_mut();
    if ret != 0 {
        let _ = fp;
        ks_free(&mut str_);
        regidx_c_311_regidx_destroy(idx);
        return std::ptr::null_mut();
    }
    ks_free(&mut str_);
    idx
}

pub unsafe fn regidx_c_311_regidx_destroy(idx: *mut regidx_t) {
    if idx.is_null() {
        return;
    }
    for i in 0..(*idx).nseq {
        let list = (*idx).seq.add(i as usize);
        if let Some(free) = (*idx).free {
            for j in 0..(*list).nreg {
                free(
                    (*list)
                        .dat
                        .add((*idx).payload_size as usize * j as usize)
                        .cast(),
                );
            }
        }
        libc::free((*list).dat.cast());
        libc::free((*list).reg.cast());
        libc::free((*list).idx.cast());
    }
    libc::free((*idx).seq_names.cast());
    libc::free((*idx).seq.cast());
    libc::free((*idx).str.s.cast());
    libc::free((*idx).payload);
    khash_str2int_destroy_free((*idx).seq2regs);
    libc::free(idx.cast());
}

// original: reglist_build_index_ (htslib/regidx.c:335)
pub unsafe fn regidx_c_335_reglist_build_index_(
    regidx: *mut regidx_t,
    list: *mut reglist_t,
) -> c_int {
    if (*list).unsorted != 0 {
        if (*regidx).payload_size == 0 {
            libc::qsort(
                (*list).reg.cast(),
                (*list).nreg as usize,
                std::mem::size_of::<reg_t>(),
                Some(regidx_c_142_cmp_reg_ptrs),
            );
        } else {
            let Some(ptr_bytes) =
                std::mem::size_of::<*mut reg_t>().checked_mul((*list).nreg as usize)
            else {
                return -1;
            };
            let ptr = libc::malloc(ptr_bytes).cast::<*mut reg_t>();
            if ptr.is_null() {
                return -1;
            }
            for i in 0..(*list).nreg as usize {
                *ptr.add(i) = (*list).reg.add(i);
            }
            libc::qsort(
                ptr.cast(),
                (*list).nreg as usize,
                std::mem::size_of::<*mut reg_t>(),
                Some(regidx_c_146_cmp_reg_ptrs2),
            );

            let Some(dat_bytes) =
                ((*regidx).payload_size as usize).checked_mul((*list).nreg as usize)
            else {
                libc::free(ptr.cast());
                return -1;
            };
            let tmp_dat = libc::malloc(dat_bytes).cast::<u8>();
            if tmp_dat.is_null() {
                libc::free(ptr.cast());
                return -1;
            }
            for i in 0..(*list).nreg as usize {
                let iori = (*ptr.add(i)).offset_from((*list).reg) as usize;
                libc::memcpy(
                    tmp_dat.add(i * (*regidx).payload_size as usize).cast(),
                    (*list)
                        .dat
                        .add(iori * (*regidx).payload_size as usize)
                        .cast(),
                    (*regidx).payload_size as usize,
                );
            }
            libc::free((*list).dat.cast());
            (*list).dat = tmp_dat;

            let Some(reg_bytes) = std::mem::size_of::<reg_t>().checked_mul((*list).nreg as usize)
            else {
                libc::free(ptr.cast());
                return -1;
            };
            let tmp_reg = libc::malloc(reg_bytes).cast::<reg_t>();
            if tmp_reg.is_null() {
                libc::free(ptr.cast());
                return -1;
            }
            for i in 0..(*list).nreg as usize {
                let iori = (*ptr.add(i)).offset_from((*list).reg) as usize;
                *tmp_reg.add(i) = *(*list).reg.add(iori);
            }
            libc::free(ptr.cast());
            libc::free((*list).reg.cast());
            (*list).reg = tmp_reg;
            (*list).mreg = (*list).nreg;
        }
        (*list).unsorted = 0;
    }

    (*list).nidx = 0;
    let mut midx: u32 = 0;
    for j in 0..(*list).nreg as usize {
        let iend = ibin((*(*list).reg.add(j)).end) as u32;
        if midx <= iend {
            midx = iend;
        }
    }
    midx += 1;
    let new_idx = libc::calloc(midx as usize, std::mem::size_of::<u32>()).cast::<u32>();
    if new_idx.is_null() {
        return -1;
    }
    libc::free((*list).idx.cast());
    (*list).idx = new_idx;
    (*list).nidx = midx;

    for j in 0..(*list).nreg {
        let ibeg = ibin((*(*list).reg.add(j as usize)).beg) as u32;
        let iend = ibin((*(*list).reg.add(j as usize)).end) as u32;
        if ibeg == iend {
            if *(*list).idx.add(ibeg as usize) == 0 {
                *(*list).idx.add(ibeg as usize) = j + 1;
            }
        } else {
            for k in ibeg..=iend {
                if *(*list).idx.add(k as usize) == 0 {
                    *(*list).idx.add(k as usize) = j + 1;
                }
            }
        }
    }

    0
}

pub unsafe fn regidx_c_401_regidx_overlap(
    idx: *mut regidx_t,
    chr: *const c_char,
    mut beg: hts_pos_t,
    mut end: hts_pos_t,
    itr: *mut regitr_t,
) -> c_int {
    if !itr.is_null() {
        (*itr).seq = std::ptr::null_mut();
    }
    if beg < 0 {
        beg = 0;
    }
    if end > MAX_COOR_0 {
        end = MAX_COOR_0;
    }
    if end < 0 {
        end = 0;
    }

    let mut iseq = 0;
    if khash_str2int_get((*idx).seq2regs, chr, &mut iseq) != 0 {
        return 0;
    }

    let list = (*idx).seq.add(iseq as usize);
    if (*list).nreg == 0 {
        return 0;
    }

    let mut ireg: u32;
    if (*list).nreg == 1 {
        if beg > (*(*list).reg).end {
            return 0;
        }
        if end < (*(*list).reg).beg {
            return 0;
        }
        ireg = 0;
    } else {
        if (*list).idx.is_null() && regidx_c_335_reglist_build_index_(idx, list) < 0 {
            return -1;
        }

        let ibeg = ibin(beg);
        if ibeg >= (*list).nidx as c_int {
            return 0;
        }

        let mut i = *(*list).idx.add(ibeg as usize);
        if i == 0 {
            let mut iend = ibin(end);
            if iend > (*list).nidx as c_int {
                iend = (*list).nidx as c_int;
            }
            let mut k = ibeg;
            while k <= iend {
                if *(*list).idx.add(k as usize) != 0 {
                    break;
                }
                k += 1;
            }
            if k > iend {
                return 0;
            }
            i = *(*list).idx.add(k as usize);
        }
        ireg = i - 1;
        while ireg < (*list).nreg {
            if (*(*list).reg.add(ireg as usize)).beg > end {
                return 0;
            }
            if (*(*list).reg.add(ireg as usize)).end >= beg
                && (*(*list).reg.add(ireg as usize)).beg <= end
            {
                break;
            }
            ireg += 1;
        }

        if ireg >= (*list).nreg {
            return 0;
        }
    }

    if itr.is_null() {
        return 1;
    }

    let itr_ = (*itr).itr.cast::<itr_t_>();
    (*itr_).ridx = idx;
    (*itr_).list = list;
    (*itr_).beg = beg;
    (*itr_).end = end;
    (*itr_).ireg = ireg;
    (*itr_).active = 0;

    (*itr).seq = (*list).seq;
    (*itr).beg = (*(*list).reg.add(ireg as usize)).beg;
    (*itr).end = (*(*list).reg.add(ireg as usize)).end;
    if (*idx).payload_size != 0 {
        (*itr).payload = (*list)
            .dat
            .add((*idx).payload_size as usize * ireg as usize)
            .cast();
    }

    1
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
    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }

    *chr_beg = ss;
    *chr_end = se.sub(1);

    if *se == 0 {
        *beg = 0;
        *end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    *beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }

    ss = se.add(1);
    *end = hts_parse_decimal(ss, &mut se, 0) - 1;
    if ss == se {
        return -2;
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
    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se = ss;
    while *se != 0 && isspace_c(*se) == 0 {
        se = se.add(1);
    }

    *chr_beg = ss;
    *chr_end = se.sub(1);

    if *se == 0 {
        *beg = 0;
        *end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    *beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }
    if *beg == 0 {
        return -2;
    }
    *beg -= 1;

    if *se == 0 || *se.add(1) == 0 {
        *end = *beg;
    } else {
        ss = se.add(1);
        *end = hts_parse_decimal(ss, &mut se, 0);
        if ss == se || (*se != 0 && isspace_c(*se) == 0) {
            *end = *beg;
        } else if *end == 0 {
            return -2;
        } else {
            *end -= 1;
        }
    }
    0
}

pub unsafe extern "C" fn regidx_c_538_regidx_parse_vcf(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    payload: *mut c_void,
    usr: *mut c_void,
) -> c_int {
    let ret = regidx_c_498_regidx_parse_tab(line, chr_beg, chr_end, beg, end, payload, usr);
    if ret == 0 {
        *end = *beg;
    }
    ret
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
    let mut ss = line as *mut c_char;
    while *ss != 0 && isspace_c(*ss) != 0 {
        ss = ss.add(1);
    }
    if *ss == 0 {
        return -1;
    }
    if *ss == b'#' as c_char {
        return -1;
    }

    let mut se = ss;
    while *se != 0 && *se != b':' as c_char {
        se = se.add(1);
    }

    *chr_beg = ss;
    *chr_end = se.sub(1);

    if *se == 0 {
        *beg = 0;
        *end = MAX_COOR_0;
        return 0;
    }

    ss = se.add(1);
    *beg = hts_parse_decimal(ss, &mut se, 0);
    if ss == se {
        return -2;
    }
    if *beg == 0 {
        return -2;
    }
    *beg -= 1;

    if *se == 0 || *se.add(1) == 0 {
        *end = if *se == b'-' as c_char {
            MAX_COOR_0
        } else {
            *beg
        };
    } else {
        ss = se.add(1);
        *end = hts_parse_decimal(ss, &mut se, 0);
        if ss == se {
            *end = *beg;
        } else if *end == 0 {
            return -2;
        } else {
            *end -= 1;
        }
    }
    0
}

pub unsafe fn regidx_c_584_regitr_init(regidx: *mut regidx_t) -> *mut regitr_t {
    let regitr = libc::calloc(1, std::mem::size_of::<regitr_t>()).cast::<regitr_t>();
    if regitr.is_null() {
        return std::ptr::null_mut();
    }
    (*regitr).itr = libc::calloc(1, std::mem::size_of::<itr_t_>());
    if (*regitr).itr.is_null() {
        libc::free(regitr.cast());
        return std::ptr::null_mut();
    }
    let itr = (*regitr).itr.cast::<itr_t_>();
    (*itr).ridx = regidx;
    (*itr).list = std::ptr::null_mut();
    regitr
}

pub unsafe fn regidx_c_599_regitr_reset(regidx: *mut regidx_t, regitr: *mut regitr_t) {
    let itr = (*regitr).itr.cast::<itr_t_>();
    libc::memset(itr.cast(), 0, std::mem::size_of::<itr_t_>());
    (*itr).ridx = regidx;
}

pub unsafe fn regidx_c_606_regitr_destroy(regitr: *mut regitr_t) {
    if regitr.is_null() {
        return;
    }
    libc::free((*regitr).itr);
    libc::free(regitr.cast());
}

pub unsafe fn regidx_c_612_regitr_overlap(regitr: *mut regitr_t) -> c_int {
    if regitr.is_null() || (*regitr).seq.is_null() || (*regitr).itr.is_null() {
        return 0;
    }

    let itr = (*regitr).itr.cast::<itr_t_>();
    if (*itr).active == 0 {
        (*itr).active = 1;
        (*itr).ireg += 1;
        return 1;
    }

    let list = (*itr).list;
    let mut i = (*itr).ireg;
    while i < (*list).nreg {
        if (*(*list).reg.add(i as usize)).beg > (*itr).end {
            return 0;
        }
        if (*(*list).reg.add(i as usize)).end >= (*itr).beg
            && (*(*list).reg.add(i as usize)).beg <= (*itr).end
        {
            break;
        }
        i += 1;
    }

    if i >= (*list).nreg {
        return 0;
    }

    (*itr).ireg = i + 1;
    (*regitr).seq = (*list).seq;
    (*regitr).beg = (*(*list).reg.add(i as usize)).beg;
    (*regitr).end = (*(*list).reg.add(i as usize)).end;
    if (*(*itr).ridx).payload_size != 0 {
        (*regitr).payload = (*list)
            .dat
            .add((*(*itr).ridx).payload_size as usize * i as usize)
            .cast();
    }

    1
}

pub unsafe fn regidx_c_646_regitr_loop(regitr: *mut regitr_t) -> c_int {
    if regitr.is_null() || (*regitr).itr.is_null() {
        return 0;
    }

    let itr = (*regitr).itr.cast::<itr_t_>();
    let regidx = (*itr).ridx;

    if (*regidx).nseq == 0 {
        return 0;
    }

    if (*itr).list.is_null() {
        (*itr).list = (*regidx).seq;
        (*itr).ireg = 0;
    }

    let mut iseq = (*itr).list.offset_from((*regidx).seq) as usize;
    if iseq >= (*regidx).nseq as usize {
        return 0;
    }

    if (*itr).ireg >= (*(*itr).list).nreg {
        iseq += 1;
        if iseq >= (*regidx).nseq as usize {
            return 0;
        }
        (*itr).ireg = 0;
        (*itr).list = (*regidx).seq.add(iseq);
    }

    (*regitr).seq = (*(*itr).list).seq;
    (*regitr).beg = (*(*(*itr).list).reg.add((*itr).ireg as usize)).beg;
    (*regitr).end = (*(*(*itr).list).reg.add((*itr).ireg as usize)).end;
    if (*regidx).payload_size != 0 {
        (*regitr).payload = (*(*itr).list)
            .dat
            .add((*regidx).payload_size as usize * (*itr).ireg as usize)
            .cast();
    }
    (*itr).ireg += 1;

    1
}

pub unsafe fn regidx_c_681_regitr_copy(dst: *mut regitr_t, src: *mut regitr_t) {
    let dst_itr = (*dst).itr.cast::<itr_t_>();
    let src_itr = (*src).itr.cast::<itr_t_>();
    *dst_itr = *src_itr;
    std::ptr::copy(src, dst, 1);
    (*dst).itr = dst_itr.cast();
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
                    regidx_c_151_regidx_push(idx, chr_beg, chr_end, beg, end, std::ptr::null_mut()),
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
                    regidx_c_151_regidx_push(idx, chr_beg, chr_end, beg, end, std::ptr::null_mut()),
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
                        idx,
                        chr_beg,
                        chr_end,
                        beg,
                        end,
                        (&mut payload as *mut c_int).cast()
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
            assert_eq!(*((*itr).payload.cast::<c_int>()), 10);

            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 30, 30, itr),
                1
            );
            assert_eq!(((*itr).beg, (*itr).end), (30, 30));
            assert_eq!(*((*itr).payload.cast::<c_int>()), 30);

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
                        idx,
                        chr_beg,
                        chr_end,
                        beg,
                        end,
                        (&mut payload as *mut c_int).cast()
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
                seen.push(((*itr).beg, (*itr).end, *((*itr).payload.cast::<c_int>())));
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
                        idx,
                        chr_beg,
                        chr_end,
                        beg,
                        end,
                        (&mut payload as *mut c_int).cast()
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
                seen.push(((*itr).beg, (*itr).end, *((*itr).payload.cast::<c_int>())));
            }
            assert_eq!(seen, [(0, 8191, 1), (8192, 8192, 2)]);

            assert_eq!(
                regidx_c_401_regidx_overlap(idx, query_chr.as_ptr(), 8193, 19999, itr),
                0
            );
            assert!((*itr).seq.is_null());

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
                regidx_c_151_regidx_push(idx, chr_beg, chr_end, 10, 20, std::ptr::null_mut()),
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
            assert!((*itr).seq.is_null());

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
                    CStr::from_ptr((*itr).seq).to_bytes().to_vec(),
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
                    idx,
                    chr_beg,
                    chr_end,
                    -10,
                    MAX_COOR_0 + 99,
                    std::ptr::null_mut()
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
                        idx,
                        chr_beg,
                        chr_end,
                        beg,
                        end,
                        (&mut payload as *mut c_int).cast()
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
                seen.push(((*itr).beg, (*itr).end, *((*itr).payload.cast::<c_int>())));
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
                regidx_c_151_regidx_push(idx, chr_beg, chr_end, 0, 2, std::ptr::null_mut()),
                0
            );

            assert_eq!(regidx_c_98_regidx_nregs(idx), 4);
            regidx_c_311_regidx_destroy(idx);
        }
    }
}
