use std::ffi::{c_char, c_int, c_void};

use super::bgzf::bgzf_getline;
use super::hts::{
    hts_c_2372_hts_adjust_csi_settings, hts_idx_destroy, hts_idx_finish, hts_idx_get_meta,
    hts_idx_init, hts_idx_load3, hts_idx_push, hts_idx_set_meta, hts_is_utf16_text, hts_pos_t,
    i32_to_le, kstring_t, le_to_i32, le_to_u32, svlen_on_ref_for_vcf_alt, toupper_c, BGZF,
};

pub type tbx_conf_t = hts_sys::tbx_conf_t;
pub type tbx_t = hts_sys::tbx_t;

const TBX_GENERIC: c_int = 0;
const TBX_SAM: c_int = 1;
const TBX_VCF: c_int = 2;
const TBX_GAF: c_int = 3;
const TBX_UCSC: c_int = 0x10000;
const TBX_MAX_SHIFT: c_int = 31;

#[repr(C)]
pub struct kh_s2i_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut *const c_char,
    pub vals: *mut i64,
}

unsafe extern "C" {
    fn kh_init_s2i() -> *mut kh_s2i_t;
    fn kh_destroy_s2i(h: *mut kh_s2i_t);
    fn kh_get_s2i(h: *const kh_s2i_t, key: *const c_char) -> u32;
    fn kh_put_s2i(h: *mut kh_s2i_t, key: *const c_char, ret: *mut c_int) -> u32;
    fn kh_del_s2i(h: *mut kh_s2i_t, x: u32);
}

#[repr(C)]
pub struct tbx_intv_t {
    pub beg: i64,
    pub end: i64,
    pub ss: *mut c_char,
    pub se: *mut c_char,
    pub tid: c_int,
}

pub unsafe fn tbx_conf_gff() -> tbx_conf_t {
    hts_sys::tbx_conf_gff
}

pub unsafe fn tbx_conf_bed() -> tbx_conf_t {
    hts_sys::tbx_conf_bed
}

pub unsafe fn tbx_conf_psltbl() -> tbx_conf_t {
    hts_sys::tbx_conf_psltbl
}

pub unsafe fn tbx_conf_sam() -> tbx_conf_t {
    hts_sys::tbx_conf_sam
}

pub unsafe fn tbx_conf_vcf() -> tbx_conf_t {
    hts_sys::tbx_conf_vcf
}

pub unsafe fn tbx_name2id(tbx: *mut tbx_t, ss: *const c_char) -> c_int {
    tbx_c_91_tbx_name2id(tbx, ss)
}

pub unsafe fn tbx_c_64_get_tid(tbx: *mut tbx_t, ss: *const c_char, is_add: c_int) -> c_int {
    if ((*tbx).conf.preset & 0xffff) == TBX_GAF {
        return 0;
    }
    if (*tbx).dict.is_null() {
        (*tbx).dict = kh_init_s2i().cast();
    }
    if (*tbx).dict.is_null() {
        return -1;
    }
    let d = (*tbx).dict.cast::<kh_s2i_t>();
    let k = if is_add != 0 {
        let mut absent = 0;
        let k = kh_put_s2i(d, ss, &mut absent);
        if absent < 0 {
            return -1;
        } else if absent != 0 {
            let ss_dup = libc::strdup(ss);
            if !ss_dup.is_null() {
                *(*d).keys.add(k as usize) = ss_dup;
                *(*d).vals.add(k as usize) = (*d).size as i64 - 1;
            } else {
                kh_del_s2i(d, k);
                return -1;
            }
        }
        k
    } else {
        kh_get_s2i(d, ss)
    };
    if k == (*d).n_buckets {
        -1
    } else {
        *(*d).vals.add(k as usize) as c_int
    }
}

pub unsafe fn tbx_c_91_tbx_name2id(tbx: *mut tbx_t, ss: *const c_char) -> c_int {
    tbx_c_64_get_tid(tbx, ss, 0)
}

pub unsafe fn tbx_c_96_tbx_parse1(
    conf: *const tbx_conf_t,
    len: usize,
    line: *mut c_char,
    intv: *mut tbx_intv_t,
) -> c_int {
    let mut b = 0usize;
    let mut id = 1;
    let mut getlen = 0;
    let mut alcnt = 0;
    let mut use_svlen = 0;
    let mut lenpos = -1;
    let mut svlenals = [0u8; 8192];
    let mut reflen = 0i64;
    let mut svlen = 0i64;
    let mut fmtlen = 0i64;
    let preset = (*conf).preset & 0xffff;

    (*intv).ss = std::ptr::null_mut();
    (*intv).se = std::ptr::null_mut();
    (*intv).beg = -1;
    (*intv).end = -1;

    for i in 0..=len {
        let ch = *line.add(i);
        if ch == b'\t' as c_char || ch == 0 {
            if id == (*conf).sc {
                (*intv).ss = line.add(b);
                (*intv).se = line.add(i);
            } else if id == (*conf).bc {
                if preset == TBX_GAF {
                    let mut s = line.add(b + 1);
                    while s < line.add(i) {
                        let mut t: *mut c_char = std::ptr::null_mut();
                        let nodeid = libc::strtoll(s, &mut t, 0);
                        if (*intv).beg == -1 {
                            (*intv).beg = nodeid;
                            (*intv).end = nodeid;
                        } else {
                            if nodeid < (*intv).beg {
                                (*intv).beg = nodeid;
                            }
                            if nodeid > (*intv).end {
                                (*intv).end = nodeid;
                            }
                        }
                        s = t.add(1);
                    }
                } else {
                    let mut s: *mut c_char = std::ptr::null_mut();
                    (*intv).beg = libc::strtoll(line.add(b), &mut s, 0);
                    if (*conf).bc <= (*conf).ec {
                        (*intv).end = (*intv).beg;
                    }
                    if s == line.add(b) {
                        return -1;
                    }
                    if ((*conf).preset & TBX_UCSC) == 0 {
                        (*intv).beg -= 1;
                    } else if (*conf).bc <= (*conf).ec {
                        (*intv).end += 1;
                    }
                    if (*intv).beg < 0 {
                        (*intv).beg = 0;
                    }
                    if (*intv).end < 1 {
                        (*intv).end = 1;
                    }
                }
            } else if preset == TBX_GENERIC {
                if id == (*conf).ec {
                    let mut s: *mut c_char = std::ptr::null_mut();
                    (*intv).end = libc::strtoll(line.add(b), &mut s, 0);
                    if s == line.add(b) {
                        return -1;
                    }
                }
            } else if preset == TBX_SAM {
                if id == 6 {
                    let mut l = 0i64;
                    let mut s = line.add(b);
                    while s < line.add(i) {
                        let mut t: *mut c_char = std::ptr::null_mut();
                        let x = libc::strtol(s, &mut t, 10);
                        let op = toupper_c(*t);
                        if op == b'M' as c_char || op == b'D' as c_char || op == b'N' as c_char {
                            l += x as i64;
                        }
                        s = t.add(1);
                    }
                    if l == 0 {
                        l = 1;
                    }
                    (*intv).end = (*intv).beg + l;
                }
            } else if preset == TBX_VCF {
                if id == 4 {
                    if b < i {
                        (*intv).end = (*intv).beg + (i - b) as i64;
                    }
                    alcnt += 1;
                    reflen = (i - b) as i64;
                } else if id == 5 {
                    let mut lastbyte = 0usize;
                    let c = *line.add(i);
                    svlenals[lastbyte] = 0;
                    *line.add(i) = 0;
                    let mut s = line.add(b);
                    loop {
                        let t = libc::strchr(s, b',' as c_int);
                        if (alcnt >> 3) as usize != lastbyte {
                            lastbyte = (alcnt >> 3) as usize;
                            svlenals[lastbyte] = 0;
                        }
                        alcnt += 1;
                        if !t.is_null() {
                            *t = 0;
                        }
                        if svlen_on_ref_for_vcf_alt(s, -1) != 0 {
                            svlenals[lastbyte] |= 1 << ((alcnt - 1) & 7);
                            use_svlen = 1;
                        } else if libc::strcmp(c"<*>".as_ptr(), s) == 0
                            || libc::strcmp(c"<NON_REF>".as_ptr(), s) == 0
                        {
                            getlen = 1;
                        }
                        if t.is_null() || alcnt >= 65536 {
                            break;
                        }
                        *t = b',' as c_char;
                        s = t.add(1);
                    }
                    *line.add(i) = c;
                } else if id == 8 {
                    let c = *line.add(i);
                    let mut d = 1;
                    *line.add(i) = 0;
                    let mut s = libc::strstr(line.add(b), c"END=".as_ptr());
                    if s == line.add(b) {
                        s = s.add(4);
                    } else if !s.is_null() {
                        s = libc::strstr(line.add(b), c";END=".as_ptr());
                        if !s.is_null() {
                            s = s.add(5);
                        }
                    }
                    if !s.is_null() && *s != b'.' as c_char {
                        let end = libc::strtoll(s, &mut s, 0);
                        if end > (*intv).beg {
                            (*intv).end = end;
                        }
                    }
                    s = libc::strstr(line.add(b), c"SVLEN=".as_ptr());
                    if s == line.add(b) {
                        s = s.add(6);
                    } else if !s.is_null() {
                        s = libc::strstr(line.add(b), c";SVLEN=".as_ptr());
                        if !s.is_null() {
                            s = s.add(7);
                        }
                    }
                    while !s.is_null() && d < alcnt {
                        let t = libc::strchr(s, b',' as c_int);
                        let tmp = if use_svlen != 0
                            && (svlenals[(d >> 3) as usize] & (1 << (d & 7))) != 0
                        {
                            let x = libc::atoll(s);
                            if x < 0 {
                                x.wrapping_abs()
                            } else {
                                x
                            }
                        } else {
                            1
                        };
                        if svlen < tmp {
                            svlen = tmp;
                        }
                        s = if t.is_null() {
                            std::ptr::null_mut()
                        } else {
                            t.add(1)
                        };
                        d += 1;
                    }
                    *line.add(i) = c;
                } else if getlen != 0 && id == 9 {
                    let c = *line.add(i);
                    let mut pos = -1;
                    *line.add(i) = 0;
                    let mut s = line.add(b);
                    while !s.is_null() {
                        pos += 1;
                        let t = libc::strchr(s, b':' as c_int);
                        if t.is_null() {
                            if libc::strcmp(s, c"LEN".as_ptr()) == 0 {
                                lenpos = pos;
                            }
                            break;
                        } else {
                            *t = 0;
                            if libc::strcmp(s, c"LEN".as_ptr()) == 0 {
                                lenpos = pos;
                                *t = b':' as c_char;
                                break;
                            }
                            *t = b':' as c_char;
                            s = t.add(1);
                        }
                    }
                    *line.add(i) = c;
                    if lenpos == -1 {
                        break;
                    }
                } else if id > 9 && getlen != 0 && lenpos != -1 {
                    let c = *line.add(i);
                    *line.add(i) = 0;
                    let mut tmp = 0i64;
                    let mut s = line.add(b);
                    for d in 0..=lenpos {
                        if d == lenpos {
                            tmp = libc::atoll(s);
                            break;
                        }
                        let t = libc::strchr(s, b':' as c_int);
                        if !t.is_null() {
                            s = t.add(1);
                        } else {
                            break;
                        }
                    }
                    if fmtlen < tmp {
                        fmtlen = tmp;
                    }
                    *line.add(i) = c;
                }
            }
            b = i + 1;
            id += 1;
        }
    }

    if preset == TBX_VCF {
        let tmp = reflen.max(svlen).max(fmtlen) + (*intv).beg;
        if (*intv).end < tmp {
            (*intv).end = tmp;
        }
    }
    if (*intv).ss.is_null() || (*intv).se.is_null() || (*intv).beg < 0 || (*intv).end < 0 {
        return -1;
    }
    0
}

pub unsafe fn tbx_c_315_get_intv(
    tbx: *mut tbx_t,
    str_: *mut kstring_t,
    intv: *mut tbx_intv_t,
    is_add: c_int,
) -> c_int {
    if tbx_c_96_tbx_parse1(&(*tbx).conf, (*str_).l, (*str_).s, intv) == 0 {
        let c = *(*intv).se;
        *(*intv).se = 0;
        if ((*tbx).conf.preset & 0xffff) == TBX_GAF {
            (*intv).tid = 0;
        } else {
            (*intv).tid = tbx_c_64_get_tid(tbx, (*intv).ss, is_add);
        }
        *(*intv).se = c;
        if (*intv).tid < 0 {
            return -2;
        }
        if (*intv).beg >= 0 && (*intv).end >= 0 {
            0
        } else {
            -1
        }
    } else {
        let type_ = match (*tbx).conf.preset & 0xffff {
            TBX_SAM => c"TBX_SAM".as_ptr(),
            TBX_VCF => c"TBX_VCF".as_ptr(),
            TBX_GAF => c"TBX_GAF".as_ptr(),
            TBX_UCSC => c"TBX_UCSC".as_ptr(),
            _ => c"TBX_GENERIC".as_ptr(),
        };
        if hts_is_utf16_text(str_) != 0 {
            hts_sys::hts_log(
                hts_sys::htsLogLevel_HTS_LOG_ERROR,
                c"get_intv".as_ptr(),
                c"Failed to parse %s: offending line appears to be encoded as UTF-16".as_ptr(),
                type_,
            );
        } else {
            hts_sys::hts_log(
                hts_sys::htsLogLevel_HTS_LOG_ERROR,
                c"get_intv".as_ptr(),
                c"Failed to parse %s: was wrong -p [type] used?\nThe offending line was: \"%s\""
                    .as_ptr(),
                type_,
                (*str_).s,
            );
        }
        -1
    }
}

pub unsafe fn tbx_readrec(
    fp: *mut BGZF,
    tbxv: *mut c_void,
    sv: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    tbx_c_353_tbx_readrec(fp, tbxv, sv, tid, beg, end)
}

pub unsafe fn tbx_c_353_tbx_readrec(
    fp: *mut BGZF,
    tbxv: *mut c_void,
    sv: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    let tbx = tbxv.cast::<tbx_t>();
    let s = sv.cast::<kstring_t>();
    let mut ret;
    loop {
        ret = bgzf_getline(fp, b'\n' as c_int, s);
        if !(ret >= 0 && (*s).l != 0 && *(*s).s == (*tbx).conf.meta_char as c_char) {
            break;
        }
    }
    if ret >= 0 {
        let mut intv = tbx_intv_t {
            beg: 0,
            end: 0,
            ss: std::ptr::null_mut(),
            se: std::ptr::null_mut(),
            tid: 0,
        };
        if tbx_c_315_get_intv(tbx, s, &mut intv, 0) < 0 {
            return -2;
        }
        *tid = intv.tid;
        *beg = intv.beg;
        *end = intv.end;
    }
    ret
}

pub unsafe fn tbx_c_375_tbx_set_meta(tbx: *mut tbx_t) -> c_int {
    let d = (*tbx).dict.cast::<kh_s2i_t>();
    let name =
        libc::malloc(std::mem::size_of::<*mut c_char>() * (*d).size as usize).cast::<*mut c_char>();
    if name.is_null() {
        return -1;
    }

    let mut name_len = 0usize;
    for k in 0..(*d).n_buckets {
        if !tbx_kh_exist(d, k) {
            continue;
        }
        let key = *(*d).keys.add(k as usize);
        *name.add(*(*d).vals.add(k as usize) as usize) = key.cast_mut();
        name_len += libc::strlen(key) + 1;
    }

    let meta = libc::malloc(name_len + 28).cast::<u8>();
    if meta.is_null() {
        libc::free(name.cast());
        return -1;
    }
    i32_to_le((*tbx).conf.preset, meta);
    i32_to_le((*tbx).conf.sc, meta.add(4));
    i32_to_le((*tbx).conf.bc, meta.add(8));
    i32_to_le((*tbx).conf.ec, meta.add(12));
    i32_to_le((*tbx).conf.meta_char, meta.add(16));
    i32_to_le((*tbx).conf.line_skip, meta.add(20));
    i32_to_le(name_len as i32, meta.add(24));

    let mut off = 28usize;
    for i in 0..(*d).size {
        let key = *name.add(i as usize);
        let len = libc::strlen(key) + 1;
        libc::memcpy(meta.add(off).cast(), key.cast(), len);
        off += len;
    }
    libc::free(name.cast());
    hts_idx_set_meta((*tbx).idx.cast(), off as u32, meta, 0)
}

unsafe fn tbx_kh_exist(h: *const kh_s2i_t, x: u32) -> bool {
    let flag = *(*h).flags.add((x >> 4) as usize);
    ((flag >> ((x & 0xf) << 1)) & 3) == 0
}

pub unsafe fn tbx_c_412_adjust_max_ref_len_vcf(str_: *const c_char, max_ref_len: *mut i64) {
    if libc::strncmp(str_, c"##contig".as_ptr(), 8) != 0 {
        return;
    }
    let mut ptr = libc::strstr(str_.add(8), c"length".as_ptr());
    if ptr.is_null() {
        return;
    }
    ptr = ptr.add(6);
    while *ptr == b' ' as c_char || *ptr == b'=' as c_char {
        ptr = ptr.add(1);
    }
    let len = libc::strtoll(ptr, std::ptr::null_mut(), 10);
    if *max_ref_len < len {
        *max_ref_len = len;
    }
}

pub unsafe fn tbx_c_425_adjust_max_ref_len_sam(str_: *const c_char, max_ref_len: *mut i64) {
    if libc::strncmp(str_, c"@SQ".as_ptr(), 3) != 0 {
        return;
    }
    let mut ptr = libc::strstr(str_.add(3), c"\tLN:".as_ptr());
    if ptr.is_null() {
        return;
    }
    ptr = ptr.add(4);
    let len = libc::strtoll(ptr, std::ptr::null_mut(), 10);
    if *max_ref_len < len {
        *max_ref_len = len;
    }
}

pub unsafe fn tbx_index(fp: *mut BGZF, min_shift: c_int, conf: *const tbx_conf_t) -> *mut tbx_t {
    tbx_c_437_tbx_index(fp, min_shift, conf)
}

pub unsafe fn tbx_c_437_tbx_index(
    fp: *mut BGZF,
    mut min_shift: c_int,
    conf: *const tbx_conf_t,
) -> *mut tbx_t {
    let mut str_: kstring_t = std::mem::zeroed();
    let tbx = libc::calloc(1, std::mem::size_of::<tbx_t>()).cast::<tbx_t>();
    if tbx.is_null() {
        return std::ptr::null_mut();
    }
    (*tbx).conf = *conf;
    let mut n_lvls;
    let fmt;
    if min_shift > 0 {
        n_lvls = (TBX_MAX_SHIFT - min_shift + 2) / 3;
        fmt = hts_sys::HTS_FMT_CSI as c_int;
    } else {
        min_shift = 14;
        n_lvls = 5;
        fmt = hts_sys::HTS_FMT_TBI as c_int;
    }

    let mut first = 0;
    let mut lineno = 0i64;
    let mut last_off = 0u64;
    let mut max_ref_len = 0i64;
    let mut ret;
    loop {
        ret = bgzf_getline(fp, b'\n' as c_int, &mut str_);
        if ret < 0 {
            break;
        }
        lineno += 1;
        if *str_.s == (*tbx).conf.meta_char as c_char && fmt == hts_sys::HTS_FMT_CSI as c_int {
            match (*tbx).conf.preset {
                TBX_SAM => tbx_c_425_adjust_max_ref_len_sam(str_.s, &mut max_ref_len),
                TBX_VCF => tbx_c_412_adjust_max_ref_len_vcf(str_.s, &mut max_ref_len),
                _ => {}
            }
        }
        if lineno <= (*tbx).conf.line_skip as i64 || *str_.s == (*tbx).conf.meta_char as c_char {
            last_off = tbx_bgzf_tell(fp);
            continue;
        }
        if first == 0 {
            if fmt == hts_sys::HTS_FMT_CSI as c_int {
                if max_ref_len != 0 {
                    hts_c_2372_hts_adjust_csi_settings(max_ref_len, &mut min_shift, &mut n_lvls);
                } else {
                    let max_n_lvls = 9;
                    n_lvls = if min_shift < 10 {
                        max_n_lvls
                    } else if min_shift < 25 {
                        max_n_lvls - (min_shift - 10) / 3
                    } else {
                        4
                    };
                }
            }
            (*tbx).idx = hts_idx_init(0, fmt, last_off, min_shift, n_lvls).cast();
            if (*tbx).idx.is_null() {
                libc::free(str_.s.cast());
                tbx_c_512_tbx_destroy(tbx);
                return std::ptr::null_mut();
            }
            first = 1;
        }
        let mut intv: tbx_intv_t = std::mem::zeroed();
        ret = tbx_c_315_get_intv(tbx, &mut str_, &mut intv, 1);
        if ret < 0
            || hts_idx_push(
                (*tbx).idx.cast(),
                intv.tid,
                intv.beg,
                intv.end,
                tbx_bgzf_tell(fp),
                1,
            ) < 0
        {
            libc::free(str_.s.cast());
            tbx_c_512_tbx_destroy(tbx);
            return std::ptr::null_mut();
        }
    }
    if ret < -1 {
        libc::free(str_.s.cast());
        tbx_c_512_tbx_destroy(tbx);
        return std::ptr::null_mut();
    }
    if (*tbx).idx.is_null() {
        (*tbx).idx = hts_idx_init(0, fmt, last_off, min_shift, n_lvls).cast();
    }
    if (*tbx).idx.is_null() {
        libc::free(str_.s.cast());
        tbx_c_512_tbx_destroy(tbx);
        return std::ptr::null_mut();
    }
    if (*tbx).dict.is_null() {
        (*tbx).dict = kh_init_s2i().cast();
    }
    if (*tbx).dict.is_null()
        || hts_idx_finish((*tbx).idx.cast(), tbx_bgzf_tell(fp)) != 0
        || tbx_c_375_tbx_set_meta(tbx) != 0
    {
        libc::free(str_.s.cast());
        tbx_c_512_tbx_destroy(tbx);
        return std::ptr::null_mut();
    }
    libc::free(str_.s.cast());
    tbx
}

unsafe fn tbx_bgzf_tell(fp: *const BGZF) -> u64 {
    ((*fp).block_address as u64) << 16 | (*fp).block_offset as u64
}

pub unsafe fn tbx_index_build(
    fn_: *const c_char,
    min_shift: c_int,
    conf: *const tbx_conf_t,
) -> c_int {
    hts_sys::tbx_index_build(fn_, min_shift, conf)
}

pub unsafe fn tbx_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    conf: *const tbx_conf_t,
) -> c_int {
    hts_sys::tbx_index_build2(fn_, fnidx, min_shift, conf)
}

pub unsafe fn tbx_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    n_threads: c_int,
    conf: *const tbx_conf_t,
) -> c_int {
    hts_sys::tbx_index_build3(fn_, fnidx, min_shift, n_threads, conf)
}

pub unsafe fn tbx_index_load(fn_: *const c_char) -> *mut tbx_t {
    tbx_c_609_tbx_index_load(fn_)
}

pub unsafe fn tbx_index_load2(fn_: *const c_char, fnidx: *const c_char) -> *mut tbx_t {
    tbx_c_604_tbx_index_load2(fn_, fnidx)
}

pub unsafe fn tbx_index_load3(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut tbx_t {
    tbx_c_599_tbx_index_load3(fn_, fnidx, flags)
}

pub unsafe fn tbx_seqnames(tbx: *mut tbx_t, n: *mut c_int) -> *mut *const c_char {
    tbx_c_614_tbx_seqnames(tbx, n)
}

pub unsafe fn tbx_destroy(tbx: *mut tbx_t) {
    tbx_c_512_tbx_destroy(tbx)
}

pub unsafe fn tbx_c_552_index_load(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut tbx_t {
    let tbx = libc::calloc(1, std::mem::size_of::<tbx_t>()).cast::<tbx_t>();
    if tbx.is_null() {
        return std::ptr::null_mut();
    }
    (*tbx).idx = hts_idx_load3(fn_, fnidx, hts_sys::HTS_FMT_TBI as c_int, flags).cast();
    if (*tbx).idx.is_null() {
        libc::free(tbx.cast());
        return std::ptr::null_mut();
    }

    let mut l_meta = 0u32;
    let meta = hts_idx_get_meta((*tbx).idx.cast(), &mut l_meta);
    if meta.is_null() || l_meta < 28 {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"index_load".as_ptr(),
            c"Invalid index header for %s".as_ptr(),
            if !fnidx.is_null() { fnidx } else { fn_ },
        );
        tbx_c_512_tbx_destroy(tbx);
        return std::ptr::null_mut();
    }

    (*tbx).conf.preset = le_to_i32(meta);
    (*tbx).conf.sc = le_to_i32(meta.add(4));
    (*tbx).conf.bc = le_to_i32(meta.add(8));
    (*tbx).conf.ec = le_to_i32(meta.add(12));
    (*tbx).conf.meta_char = le_to_i32(meta.add(16));
    (*tbx).conf.line_skip = le_to_i32(meta.add(20));
    let l_nm = le_to_u32(meta.add(24));
    if l_nm > l_meta - 28 {
        hts_sys::hts_log(
            hts_sys::htsLogLevel_HTS_LOG_ERROR,
            c"index_load".as_ptr(),
            c"Invalid index header for %s".as_ptr(),
            if !fnidx.is_null() { fnidx } else { fn_ },
        );
        tbx_c_512_tbx_destroy(tbx);
        return std::ptr::null_mut();
    }

    let nm = meta.add(28).cast::<c_char>();
    let mut p = nm;
    while p.offset_from(nm) < l_nm as isize {
        if tbx_c_64_get_tid(tbx, p, 1) < 0 {
            hts_sys::hts_log(
                hts_sys::htsLogLevel_HTS_LOG_ERROR,
                c"index_load".as_ptr(),
                c"%s".as_ptr(),
                libc::strerror(*libc::__errno_location()),
            );
            tbx_c_512_tbx_destroy(tbx);
            return std::ptr::null_mut();
        }
        p = p.add(libc::strlen(p) + 1);
    }
    tbx
}

pub unsafe fn tbx_c_599_tbx_index_load3(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut tbx_t {
    tbx_c_552_index_load(fn_, fnidx, flags)
}

pub unsafe fn tbx_c_604_tbx_index_load2(fn_: *const c_char, fnidx: *const c_char) -> *mut tbx_t {
    tbx_c_552_index_load(fn_, fnidx, 1)
}

pub unsafe fn tbx_c_609_tbx_index_load(fn_: *const c_char) -> *mut tbx_t {
    tbx_c_552_index_load(fn_, std::ptr::null(), 1)
}

pub unsafe fn tbx_c_614_tbx_seqnames(tbx: *mut tbx_t, n: *mut c_int) -> *mut *const c_char {
    let d = (*tbx).dict.cast::<kh_s2i_t>();
    if d.is_null() {
        *n = 0;
        return libc::calloc(1, std::mem::size_of::<*const c_char>()).cast::<*const c_char>();
    }
    let names = libc::calloc((*d).size as usize, std::mem::size_of::<*const c_char>())
        .cast::<*const c_char>();
    if names.is_null() {
        return std::ptr::null_mut();
    }
    for k in 0..(*d).n_buckets {
        if tbx_kh_exist(d, k) {
            *names.add(*(*d).vals.add(k as usize) as usize) = *(*d).keys.add(k as usize);
        }
    }
    *n = (*d).size as c_int;
    names
}

pub unsafe fn tbx_c_644_tbx_name2id_wrapper(vhdr: *mut c_void, ref_: *const c_char) -> c_int {
    tbx_c_91_tbx_name2id(vhdr.cast::<tbx_t>(), ref_)
}

pub unsafe fn tbx_c_512_tbx_destroy(tbx: *mut tbx_t) {
    if tbx.is_null() {
        return;
    }
    let d = (*tbx).dict.cast::<kh_s2i_t>();
    if !d.is_null() {
        for k in 0..(*d).n_buckets {
            if tbx_kh_exist(d, k) {
                libc::free((*(*d).keys.add(k as usize)).cast_mut().cast());
            }
        }
    }
    hts_idx_destroy((*tbx).idx.cast());
    kh_destroy_s2i(d);
    libc::free(tbx.cast());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tbx_public_conf_statics_are_accessible() {
        unsafe {
            let vcf = tbx_conf_vcf();
            assert_eq!(vcf.sc, 1);
            assert_eq!(vcf.bc, 2);
            assert_eq!(vcf.meta_char, b'#' as c_int);

            let sam = tbx_conf_sam();
            assert_eq!(sam.sc, 3);
            assert_eq!(sam.bc, 4);
            assert_eq!(sam.meta_char, b'@' as c_int);
        }
    }

    #[test]
    fn tbx_parse1_vcf_uses_info_end_and_symbolic_svlen() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line = b"chr1\t100\t.\tA\t<DEL>,G\t.\t.\tEND=105;SVLEN=-12,1\0".to_vec();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: std::ptr::null_mut(),
                se: std::ptr::null_mut(),
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, line.len() - 1, line.as_mut_ptr().cast(), &mut intv),
                0
            );
            assert_eq!(intv.beg, 99);
            assert_eq!(intv.end, 111);
            assert_eq!(
                std::ffi::CStr::from_bytes_with_nul_unchecked(b"chr1\0").to_bytes(),
                std::slice::from_raw_parts(
                    intv.ss.cast::<u8>(),
                    intv.se.offset_from(intv.ss) as usize
                )
            );
        }
    }

    #[test]
    fn tbx_parse1_sam_uses_cigar_reference_span() {
        unsafe {
            let conf = tbx_conf_sam();
            let mut line = b"r001\t0\tref\t7\t30\t8M2I4M1D3M\t*\t0\t0\tACGT\t*\0".to_vec();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: std::ptr::null_mut(),
                se: std::ptr::null_mut(),
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, line.len() - 1, line.as_mut_ptr().cast(), &mut intv),
                0
            );
            assert_eq!(intv.beg, 6);
            assert_eq!(intv.end, 22);
        }
    }

    #[test]
    fn tbx_header_ref_len_scanners_keep_maximum() {
        unsafe {
            let mut max_ref_len = 100i64;
            tbx_c_412_adjust_max_ref_len_vcf(
                c"##contig=<ID=chr1,length=248956422>".as_ptr(),
                &mut max_ref_len,
            );
            assert_eq!(max_ref_len, 248956422);

            tbx_c_425_adjust_max_ref_len_sam(
                c"@SQ\tSN:chr2\tLN:242193529".as_ptr(),
                &mut max_ref_len,
            );
            assert_eq!(max_ref_len, 248956422);
        }
    }

    #[test]
    fn tbx_get_intv_gaf_uses_single_tid_without_dictionary() {
        unsafe {
            let mut tbx = tbx_t {
                conf: tbx_conf_t {
                    preset: TBX_GAF,
                    sc: 1,
                    bc: 6,
                    ec: 0,
                    meta_char: b'#' as c_int,
                    line_skip: 0,
                },
                idx: std::ptr::null_mut(),
                dict: std::ptr::null_mut(),
            };
            let mut line = b"read1\t10\t0\t10\t+\t>12>3>99\t0\t0\t255\0".to_vec();
            let mut str_ = kstring_t {
                l: line.len() - 1,
                m: line.len(),
                s: line.as_mut_ptr().cast(),
            };
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: std::ptr::null_mut(),
                se: std::ptr::null_mut(),
                tid: -1,
            };

            assert_eq!(tbx_c_315_get_intv(&mut tbx, &mut str_, &mut intv, 1), 0);
            assert_eq!(intv.tid, 0);
            assert_eq!(intv.beg, 3);
            assert_eq!(intv.end, 99);
            assert!(tbx.dict.is_null());
        }
    }

    #[test]
    fn tbx_get_tid_seqnames_and_destroy_use_s2i_dictionary() {
        unsafe {
            let tbx = libc::calloc(1, std::mem::size_of::<tbx_t>()).cast::<tbx_t>();
            assert!(!tbx.is_null());
            (*tbx).conf = tbx_conf_vcf();

            assert_eq!(tbx_c_64_get_tid(tbx, c"chr2".as_ptr(), 1), 0);
            assert_eq!(tbx_c_64_get_tid(tbx, c"chr1".as_ptr(), 1), 1);
            assert_eq!(tbx_c_91_tbx_name2id(tbx, c"chr2".as_ptr()), 0);
            assert_eq!(tbx_c_91_tbx_name2id(tbx, c"missing".as_ptr()), -1);

            let mut n = -1;
            let names = tbx_c_614_tbx_seqnames(tbx, &mut n);
            assert!(!names.is_null());
            assert_eq!(n, 2);
            assert_eq!(std::ffi::CStr::from_ptr(*names.add(0)), c"chr2");
            assert_eq!(std::ffi::CStr::from_ptr(*names.add(1)), c"chr1");
            libc::free(names.cast());

            tbx_c_512_tbx_destroy(tbx);
        }
    }

    #[test]
    fn tbx_index_builds_dictionary_from_bgzf_records() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib-mini-rs-tbx-index-{}-{}.vcf.gz",
                std::process::id(),
                line!()
            ));
            let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let payload = b"##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr2\t2\t.\tA\tC\t.\t.\t.\nchr1\t3\t.\tG\tT\t.\t.\t.\n";

            let out = super::super::bgzf::bgzf_open(c_path.as_ptr(), c"w".as_ptr());
            assert!(!out.is_null());
            assert_eq!(
                super::super::bgzf::bgzf_write(out, payload.as_ptr().cast(), payload.len()),
                payload.len() as isize
            );
            assert_eq!(super::super::bgzf::bgzf_close(out), 0);

            let fp = super::super::bgzf::bgzf_open(c_path.as_ptr(), c"r".as_ptr());
            assert!(!fp.is_null());
            let conf = tbx_conf_vcf();
            let tbx = tbx_c_437_tbx_index(fp, 0, &conf);
            assert!(!tbx.is_null());
            assert_eq!(super::super::bgzf::bgzf_close(fp), 0);

            let mut n = 0;
            let names = tbx_c_614_tbx_seqnames(tbx, &mut n);
            assert_eq!(n, 2);
            assert_eq!(std::ffi::CStr::from_ptr(*names.add(0)), c"chr2");
            assert_eq!(std::ffi::CStr::from_ptr(*names.add(1)), c"chr1");
            libc::free(names.cast());
            tbx_c_512_tbx_destroy(tbx);
            libc::unlink(c_path.as_ptr());
        }
    }
}
