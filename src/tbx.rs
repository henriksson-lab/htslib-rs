use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr::NonNull;

use super::bgzf::{bgzf_close, bgzf_compression, bgzf_getline, bgzf_mt, bgzf_open};
use super::hts::{
    hts_c_2372_hts_adjust_csi_settings, hts_c_2405_hts_idx_init, hts_c_2515_hts_idx_finish,
    hts_c_2558_hts_idx_push, hts_c_2869_hts_idx_save_as, hts_c_3062_hts_idx_set_meta,
    hts_c_3084_hts_idx_get_meta, hts_idx_destroy, hts_idx_load3, hts_is_utf16_text, hts_itr_query,
    hts_itr_t, hts_log_cstr, hts_parse_region, hts_pos_t, ks_free, kstring_t,
    svlen_on_ref_for_vcf_alt, BGZF, HTS_FMT_CSI, HTS_IDX_NOCOOR, HTS_IDX_START,
    HTS_LOG_ERROR, HTS_PARSE_THOUSANDS_SEP,
};

// HTS_FMT_TBI is 2 (htslib/htslib/hts.h); hts.rs only re-exports CSI/BAI/CRAI/FAI.
const HTS_FMT_TBI: c_int = 2;
// htsCompression::bgzf == 2 (htslib/htslib/hts.h).
const HTS_COMPRESSION_BGZF: c_int = 2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tbx_conf_t {
    pub preset: i32,
    pub sc: i32,
    pub bc: i32,
    pub ec: i32,
    pub meta_char: i32,
    pub line_skip: i32,
}

pub struct tbx_t {
    pub conf: tbx_conf_t,
    pub idx: Option<NonNull<crate::htslib_rs::hts::hts_idx_t>>,
    pub dict: Option<kh_s2i_t>,
}

const TBX_GENERIC: c_int = 0;
const TBX_SAM: c_int = 1;
const TBX_VCF: c_int = 2;
const TBX_GAF: c_int = 3;
const TBX_UCSC: c_int = 0x10000;
const TBX_MAX_SHIFT: c_int = 31;

pub struct kh_s2i_t {
    pub names: Vec<Vec<u8>>,
}

impl kh_s2i_t {
    fn new() -> Self {
        Self { names: Vec::new() }
    }

    fn get(&self, key: &[u8]) -> Option<c_int> {
        self.names
            .iter()
            .position(|name| name.strip_suffix(&[0]) == Some(key))
            .map(|tid| tid as c_int)
    }

    fn get_or_insert(&mut self, key: &[u8]) -> Result<c_int, ()> {
        if let Some(tid) = self.get(key) {
            return Ok(tid);
        }
        let tid = self.names.len();
        self.names.try_reserve(1).map_err(|_| ())?;
        let mut name = Vec::new();
        name.try_reserve_exact(key.len() + 1).map_err(|_| ())?;
        name.extend_from_slice(key);
        name.push(0);
        self.names.push(name);
        Ok(tid as c_int)
    }
}

pub struct tbx_intv_t {
    pub beg: i64,
    pub end: i64,
    pub ss: Option<usize>,
    pub se: Option<usize>,
    pub tid: c_int,
}

impl tbx_t {
    fn new(conf: tbx_conf_t) -> Self {
        Self {
            conf,
            idx: None,
            dict: None,
        }
    }

    pub(crate) fn idx_ptr(&self) -> *mut crate::htslib_rs::hts::hts_idx_t {
        self.idx.map_or(std::ptr::null_mut(), |idx| idx.as_ptr())
    }

    fn set_idx(&mut self, idx: Option<NonNull<crate::htslib_rs::hts::hts_idx_t>>) -> bool {
        self.idx = idx;
        self.idx.is_some()
    }
}

impl Drop for tbx_t {
    fn drop(&mut self) {
        unsafe {
            hts_idx_destroy(
                self.idx
                    .take()
                    .map_or(std::ptr::null_mut(), |idx| idx.as_ptr())
                    .cast(),
            );
        }
    }
}

struct KStringGuard {
    raw: kstring_t,
}

impl KStringGuard {
    fn new() -> Self {
        Self {
            raw: kstring_t { data: Vec::new() },
        }
    }
}

impl Drop for KStringGuard {
    fn drop(&mut self) {
        ks_free(&mut self.raw);
    }
}

fn tbx_intv_empty() -> tbx_intv_t {
    tbx_intv_t {
        beg: 0,
        end: 0,
        ss: None,
        se: None,
        tid: 0,
    }
}

pub unsafe fn tbx_conf_gff() -> tbx_conf_t {
    // htslib/tbx.c:44 { 0, 1, 4, 5, '#', 0 }
    tbx_conf_t {
        preset: 0,
        sc: 1,
        bc: 4,
        ec: 5,
        meta_char: b'#' as c_int,
        line_skip: 0,
    }
}

pub unsafe fn tbx_conf_bed() -> tbx_conf_t {
    // htslib/tbx.c:47 { TBX_UCSC, 1, 2, 3, '#', 0 }
    tbx_conf_t {
        preset: TBX_UCSC,
        sc: 1,
        bc: 2,
        ec: 3,
        meta_char: b'#' as c_int,
        line_skip: 0,
    }
}

pub unsafe fn tbx_conf_psltbl() -> tbx_conf_t {
    // htslib/tbx.c:50 { TBX_UCSC, 15, 17, 18, '#', 0 }
    tbx_conf_t {
        preset: TBX_UCSC,
        sc: 15,
        bc: 17,
        ec: 18,
        meta_char: b'#' as c_int,
        line_skip: 0,
    }
}

pub unsafe fn tbx_conf_sam() -> tbx_conf_t {
    // htslib/tbx.c:53 { TBX_SAM, 3, 4, 0, '@', 0 }
    tbx_conf_t {
        preset: TBX_SAM,
        sc: 3,
        bc: 4,
        ec: 0,
        meta_char: b'@' as c_int,
        line_skip: 0,
    }
}

pub unsafe fn tbx_conf_vcf() -> tbx_conf_t {
    // htslib/tbx.c:56 { TBX_VCF, 1, 2, 0, '#', 0 }
    tbx_conf_t {
        preset: TBX_VCF,
        sc: 1,
        bc: 2,
        ec: 0,
        meta_char: b'#' as c_int,
        line_skip: 0,
    }
}

pub unsafe fn tbx_conf_gaf() -> tbx_conf_t {
    tbx_conf_t {
        preset: TBX_GAF,
        sc: 1,
        bc: 6,
        ec: 0,
        meta_char: b'#' as c_int,
        line_skip: 0,
    }
}

pub fn tbx_name2id(tbx: &tbx_t, ss: &[u8]) -> c_int {
    tbx_c_91_tbx_name2id(tbx, ss)
}

pub fn tbx_c_64_get_tid(tbx: &mut tbx_t, ss: &[u8], is_add: c_int) -> c_int {
    if (tbx.conf.preset & 0xffff) == TBX_GAF {
        return 0;
    }
    if is_add != 0 {
        tbx.dict
            .get_or_insert_with(kh_s2i_t::new)
            .get_or_insert(ss)
            .unwrap_or(-1)
    } else {
        tbx.dict
            .as_ref()
            .and_then(|dict| dict.get(ss))
            .unwrap_or(-1)
    }
}

pub fn tbx_c_91_tbx_name2id(tbx: &tbx_t, ss: &[u8]) -> c_int {
    if (tbx.conf.preset & 0xffff) == TBX_GAF {
        return 0;
    }
    tbx.dict
        .as_ref()
        .and_then(|dict| dict.get(ss))
        .unwrap_or(-1)
}

pub fn tbx_c_96_tbx_parse1(conf: &tbx_conf_t, line: &[u8], intv: &mut tbx_intv_t) -> c_int {
    tbx_parse1_line(conf, line, intv)
}

fn parse_i64_prefix(field: &[u8], radix: u32) -> Option<i64> {
    let mut i = 0usize;
    while field.get(i).is_some_and(|byte| byte.is_ascii_whitespace()) {
        i += 1;
    }

    let sign = match field.get(i) {
        Some(b'-') => {
            i += 1;
            -1i64
        }
        Some(b'+') => {
            i += 1;
            1i64
        }
        _ => 1i64,
    };
    let digit_start = i;
    let mut value = 0i64;
    while let Some(digit) = field
        .get(i)
        .and_then(|byte| char::from(*byte).to_digit(radix))
    {
        value = value
            .saturating_mul(radix as i64)
            .saturating_add(digit as i64);
        i += 1;
    }
    (i != digit_start).then_some(value.saturating_mul(sign))
}

fn parse_i64_prefix_c_radix(field: &[u8]) -> Option<i64> {
    let mut i = 0usize;
    while field.get(i).is_some_and(|byte| byte.is_ascii_whitespace()) {
        i += 1;
    }
    let sign_len = matches!(field.get(i), Some(b'-' | b'+')) as usize;
    let digits = &field[i + sign_len..];
    let sign = if field.get(i) == Some(&b'-') {
        -1i64
    } else {
        1i64
    };
    if digits.starts_with(b"0x") || digits.starts_with(b"0X") {
        parse_i64_prefix(&digits[2..], 16).map(|value| value.saturating_mul(sign))
    } else if digits.starts_with(b"0") && digits.len() > 1 {
        parse_i64_prefix(&digits[1..], 8).map(|value| value.saturating_mul(sign))
    } else {
        parse_i64_prefix(field, 10)
    }
}

fn find_vcf_info_value<'a>(info: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    info.split(|&byte| byte == b';').find_map(|entry| {
        entry
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(b"="))
    })
}

fn nul_terminated_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.contains(&0) {
        return None;
    }
    let mut nul_terminated = Vec::new();
    nul_terminated.try_reserve_exact(bytes.len() + 1).ok()?;
    nul_terminated.extend_from_slice(bytes);
    nul_terminated.push(0);
    Some(nul_terminated)
}

fn tbx_parse1_line(conf: &tbx_conf_t, line: &[u8], intv: &mut tbx_intv_t) -> c_int {
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
    let preset = conf.preset & 0xffff;

    intv.ss = None;
    intv.se = None;
    intv.beg = -1;
    intv.end = -1;

    for i in 0..=line.len() {
        if i == line.len() || line[i] == b'\t' {
            let field = &line[b..i];
            if id == conf.sc {
                intv.ss = Some(b);
                intv.se = Some(i);
            } else if id == conf.bc {
                if preset == TBX_GAF {
                    for segment in field
                        .get(1..)
                        .unwrap_or_default()
                        .split(|&byte| !(byte == b'+' || byte == b'-' || byte.is_ascii_hexdigit()))
                    {
                        let Some(nodeid) = parse_i64_prefix_c_radix(segment) else {
                            continue;
                        };
                        if intv.beg == -1 {
                            intv.beg = nodeid;
                            intv.end = nodeid;
                        } else {
                            if nodeid < intv.beg {
                                intv.beg = nodeid;
                            }
                            if nodeid > intv.end {
                                intv.end = nodeid;
                            }
                        }
                    }
                } else {
                    let Some(beg) = parse_i64_prefix_c_radix(field) else {
                        return -1;
                    };
                    intv.beg = beg;
                    if conf.bc <= conf.ec {
                        intv.end = intv.beg;
                    }
                    if (conf.preset & TBX_UCSC) == 0 {
                        intv.beg -= 1;
                    } else if conf.bc <= conf.ec {
                        intv.end += 1;
                    }
                    if intv.beg < 0 {
                        intv.beg = 0;
                    }
                    if intv.end < 1 {
                        intv.end = 1;
                    }
                }
            } else if preset == TBX_GENERIC {
                if id == conf.ec {
                    let Some(end) = parse_i64_prefix_c_radix(field) else {
                        return -1;
                    };
                    intv.end = end;
                }
            } else if preset == TBX_SAM {
                if id == 6 {
                    let mut l = 0i64;
                    let mut rest = field;
                    while !rest.is_empty() {
                        let Some(x) = parse_i64_prefix(rest, 10) else {
                            break;
                        };
                        let digits = rest.iter().take_while(|byte| byte.is_ascii_digit()).count();
                        if let Some(op) = rest
                            .get(digits)
                            .copied()
                            .map(|byte| byte.to_ascii_uppercase())
                        {
                            if op == b'M' || op == b'D' || op == b'N' {
                                l += x;
                            }
                            rest = rest.get(digits + 1..).unwrap_or_default();
                        } else {
                            break;
                        }
                    }
                    if l == 0 {
                        l = 1;
                    }
                    intv.end = intv.beg + l;
                }
            } else if preset == TBX_VCF {
                if id == 4 {
                    if b < i {
                        intv.end = intv.beg + (i - b) as i64;
                    }
                    alcnt += 1;
                    reflen = (i - b) as i64;
                } else if id == 5 {
                    let mut lastbyte = 0usize;
                    svlenals[lastbyte] = 0;
                    for alt in field.split(|&byte| byte == b',') {
                        if (alcnt >> 3) as usize != lastbyte {
                            lastbyte = (alcnt >> 3) as usize;
                            svlenals[lastbyte] = 0;
                        }
                        alcnt += 1;
                        let alt = nul_terminated_bytes(alt);
                        let alt_ptr = alt.as_ref().map_or(c"".as_ptr(), |alt| alt.as_ptr().cast());
                        if unsafe { svlen_on_ref_for_vcf_alt(alt_ptr, -1) } != 0 {
                            svlenals[lastbyte] |= 1 << ((alcnt - 1) & 7);
                            use_svlen = 1;
                        } else if alt
                            .as_deref()
                            .and_then(|alt| alt.strip_suffix(&[0]))
                            .is_some_and(|alt| alt == b"<*>" || alt == b"<NON_REF>")
                        {
                            getlen = 1;
                        }
                        if alcnt >= 65536 {
                            break;
                        }
                    }
                } else if id == 8 {
                    let mut d = 1;
                    if let Some(end_field) = find_vcf_info_value(field, b"END") {
                        if end_field.first() != Some(&b'.') {
                            if let Some(end) = parse_i64_prefix_c_radix(end_field) {
                                if end > intv.beg {
                                    intv.end = end;
                                }
                            }
                        }
                    }
                    if let Some(svlen_field) = find_vcf_info_value(field, b"SVLEN") {
                        for value in svlen_field.split(|&byte| byte == b',') {
                            if d >= alcnt {
                                break;
                            }
                            let tmp = if use_svlen != 0
                                && (svlenals[(d >> 3) as usize] & (1 << (d & 7))) != 0
                            {
                                let x = parse_i64_prefix(value, 10).unwrap_or(0);
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
                            d += 1;
                        }
                    }
                } else if getlen != 0 && id == 9 {
                    let mut pos = -1;
                    for format_key in field.split(|&byte| byte == b':') {
                        pos += 1;
                        if format_key == b"LEN" {
                            lenpos = pos;
                            break;
                        }
                    }
                    if lenpos == -1 {
                        break;
                    }
                } else if id > 9 && getlen != 0 && lenpos != -1 {
                    let mut tmp = None;
                    for (d, value) in field.split(|&byte| byte == b':').enumerate() {
                        if d == lenpos as usize {
                            tmp = Some(parse_i64_prefix(value, 10).unwrap_or(0));
                            break;
                        }
                    }
                    let tmp = tmp.unwrap_or(0);
                    if fmtlen < tmp {
                        fmtlen = tmp;
                    }
                }
            }
            b = i + 1;
            id += 1;
        }
    }

    if preset == TBX_VCF {
        let tmp = reflen.max(svlen).max(fmtlen) + intv.beg;
        if intv.end < tmp {
            intv.end = tmp;
        }
    }
    if intv.ss.is_none() || intv.se.is_none() || intv.beg < 0 || intv.end < 0 {
        return -1;
    }
    0
}

pub unsafe fn tbx_c_315_get_intv(
    tbx: &mut tbx_t,
    str_: &mut kstring_t,
    intv: &mut tbx_intv_t,
    is_add: c_int,
) -> c_int {
    let line = &str_.data[..];
    if tbx_c_96_tbx_parse1(&tbx.conf, line, intv) == 0 {
        let ss = &line[intv.ss.unwrap()..intv.se.unwrap()];
        if (tbx.conf.preset & 0xffff) == TBX_GAF {
            intv.tid = 0;
        } else {
            intv.tid = tbx_c_64_get_tid(tbx, ss, is_add);
        }
        if intv.tid < 0 {
            return -2;
        }
        if intv.beg >= 0 && intv.end >= 0 {
            0
        } else {
            -1
        }
    } else {
        let type_: &str = match tbx.conf.preset & 0xffff {
            TBX_SAM => "TBX_SAM",
            TBX_VCF => "TBX_VCF",
            TBX_GAF => "TBX_GAF",
            TBX_UCSC => "TBX_UCSC",
            _ => "TBX_GENERIC",
        };
        if hts_is_utf16_text(str_) != 0 {
            let msg = std::ffi::CString::new(format!(
                "Failed to parse {type_}: offending line appears to be encoded as UTF-16"
            ))
            .unwrap();
            hts_log_cstr(HTS_LOG_ERROR, c"get_intv".as_ptr(), msg.as_ptr());
        } else {
            let line = String::from_utf8_lossy(&str_.data);
            let msg = std::ffi::CString::new(format!(
                "Failed to parse {type_}: was wrong -p [type] used?\nThe offending line was: \"{line}\""
            ))
            .unwrap_or_else(|_| std::ffi::CString::new(format!(
                "Failed to parse {type_}: was wrong -p [type] used?"
            )).unwrap());
            hts_log_cstr(HTS_LOG_ERROR, c"get_intv".as_ptr(), msg.as_ptr());
        }
        -1
    }
}

pub unsafe extern "C" fn tbx_readrec(
    fp: *mut BGZF,
    tbxv: *mut c_void,
    sv: *mut c_void,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
) -> c_int {
    let Some(fp) = fp.as_mut() else {
        return -2;
    };
    let Some(tbx) = tbxv.cast::<tbx_t>().as_mut() else {
        return -2;
    };
    let Some(s) = sv.cast::<kstring_t>().as_mut() else {
        return -2;
    };
    let Some(tid) = tid.as_mut() else {
        return -2;
    };
    let Some(beg) = beg.as_mut() else {
        return -2;
    };
    let Some(end) = end.as_mut() else {
        return -2;
    };
    tbx_c_353_tbx_readrec(fp, tbx, s, tid, beg, end)
}

pub unsafe fn tbx_c_353_tbx_readrec(
    fp: &mut BGZF,
    tbx: &mut tbx_t,
    s: &mut kstring_t,
    tid: &mut c_int,
    beg: &mut hts_pos_t,
    end: &mut hts_pos_t,
) -> c_int {
    let mut ret;
    loop {
        ret = bgzf_getline(fp, b'\n' as c_int, s);
        if !(ret >= 0 && !s.data.is_empty() && s.data[0] == tbx.conf.meta_char as u8) {
            break;
        }
    }
    if ret >= 0 {
        let mut intv = tbx_intv_empty();
        if tbx_c_315_get_intv(&mut *tbx, &mut *s, &mut intv, 0) < 0 {
            return -2;
        }
        *tid = intv.tid;
        *beg = intv.beg;
        *end = intv.end;
    }
    ret
}

pub unsafe fn tbx_c_375_tbx_set_meta(tbx: &mut tbx_t) -> c_int {
    let Some(dict) = tbx.dict.as_ref() else {
        return -1;
    };
    let name_len: usize = dict.names.iter().map(|name| name.len()).sum();

    let mut meta = Vec::<u8>::new();
    if meta.try_reserve_exact(name_len + 28).is_err() {
        return -1;
    }
    meta.resize(name_len + 28, 0);
    meta[0..4].copy_from_slice(&tbx.conf.preset.to_le_bytes());
    meta[4..8].copy_from_slice(&tbx.conf.sc.to_le_bytes());
    meta[8..12].copy_from_slice(&tbx.conf.bc.to_le_bytes());
    meta[12..16].copy_from_slice(&tbx.conf.ec.to_le_bytes());
    meta[16..20].copy_from_slice(&tbx.conf.meta_char.to_le_bytes());
    meta[20..24].copy_from_slice(&tbx.conf.line_skip.to_le_bytes());
    meta[24..28].copy_from_slice(&(name_len as i32).to_le_bytes());

    let mut off = 28usize;
    for key in &dict.names {
        meta[off..off + key.len()].copy_from_slice(key);
        off += key.len();
    }
    hts_c_3062_hts_idx_set_meta(tbx.idx_ptr().cast(), off as u32, meta.as_mut_ptr(), 1)
}

pub fn tbx_c_412_adjust_max_ref_len_vcf(line: &[u8], max_ref_len: &mut i64) {
    if !line.starts_with(b"##contig") {
        return;
    }
    let Some(length_start) = line[8..]
        .windows(b"length".len())
        .position(|window| window == b"length")
        .map(|pos| pos + 8 + b"length".len())
    else {
        return;
    };
    let mut len_field = &line[length_start..];
    while matches!(len_field.first(), Some(b' ' | b'=')) {
        len_field = &len_field[1..];
    }
    let Some(len) = parse_i64_prefix(len_field, 10) else {
        return;
    };
    if *max_ref_len < len {
        *max_ref_len = len;
    }
}

pub fn tbx_c_425_adjust_max_ref_len_sam(line: &[u8], max_ref_len: &mut i64) {
    if !line.starts_with(b"@SQ") {
        return;
    }
    let Some(length_start) = line[3..]
        .windows(b"\tLN:".len())
        .position(|window| window == b"\tLN:")
        .map(|pos| pos + 3 + b"\tLN:".len())
    else {
        return;
    };
    let Some(len) = parse_i64_prefix(&line[length_start..], 10) else {
        return;
    };
    if *max_ref_len < len {
        *max_ref_len = len;
    }
}

pub unsafe fn tbx_index(fp: *mut BGZF, min_shift: c_int, conf: &tbx_conf_t) -> *mut tbx_t {
    let Some(fp) = fp.as_mut() else {
        return std::ptr::null_mut();
    };
    tbx_index_owned(fp, min_shift, conf).map_or(std::ptr::null_mut(), Box::into_raw)
}

pub unsafe fn tbx_c_437_tbx_index(
    fp: *mut BGZF,
    min_shift: c_int,
    conf: &tbx_conf_t,
) -> *mut tbx_t {
    tbx_index(fp, min_shift, conf)
}

unsafe fn tbx_index_owned(
    fp: &mut BGZF,
    mut min_shift: c_int,
    conf: &tbx_conf_t,
) -> Option<Box<tbx_t>> {
    let mut str_ = KStringGuard::new();
    let mut tbx = Box::new(tbx_t::new(*conf));
    let mut n_lvls;
    let fmt;
    if min_shift > 0 {
        n_lvls = (TBX_MAX_SHIFT - min_shift + 2) / 3;
        fmt = HTS_FMT_CSI;
    } else {
        min_shift = 14;
        n_lvls = 5;
        fmt = HTS_FMT_TBI;
    }

    let mut first = 0;
    let mut lineno = 0i64;
    let mut last_off = 0u64;
    let mut max_ref_len = 0i64;
    let mut ret;
    loop {
        ret = bgzf_getline(fp, b'\n' as c_int, &mut str_.raw);
        if ret < 0 {
            break;
        }
        lineno += 1;
        if str_.raw.data.first().copied() == Some(tbx.conf.meta_char as u8) && fmt == HTS_FMT_CSI {
            let line = &str_.raw.data[..];
            match tbx.conf.preset {
                TBX_SAM => tbx_c_425_adjust_max_ref_len_sam(line, &mut max_ref_len),
                TBX_VCF => tbx_c_412_adjust_max_ref_len_vcf(line, &mut max_ref_len),
                _ => {}
            }
        }
        if lineno <= tbx.conf.line_skip as i64
            || str_.raw.data.first().copied() == Some(tbx.conf.meta_char as u8)
        {
            last_off = tbx_bgzf_tell(&*fp);
            continue;
        }
        if first == 0 {
            if fmt == HTS_FMT_CSI {
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
            if !tbx.set_idx(NonNull::new(
                hts_c_2405_hts_idx_init(0, fmt, last_off, min_shift, n_lvls).cast(),
            )) {
                return None;
            }
            first = 1;
        }
        let mut intv = tbx_intv_empty();
        ret = tbx_c_315_get_intv(&mut tbx, &mut str_.raw, &mut intv, 1);
        if ret < 0
            || hts_c_2558_hts_idx_push(
                tbx.idx_ptr().cast(),
                intv.tid,
                intv.beg,
                intv.end,
                tbx_bgzf_tell(&*fp),
                1,
            ) < 0
        {
            return None;
        }
    }
    if ret < -1 {
        return None;
    }
    if tbx.idx.is_none() {
        tbx.set_idx(NonNull::new(
            hts_c_2405_hts_idx_init(0, fmt, last_off, min_shift, n_lvls).cast(),
        ));
    }
    if tbx.idx.is_none() {
        return None;
    }
    tbx.dict.get_or_insert_with(kh_s2i_t::new);
    if hts_c_2515_hts_idx_finish(tbx.idx_ptr().cast(), tbx_bgzf_tell(&*fp)) != 0
        || tbx_c_375_tbx_set_meta(&mut tbx) != 0
    {
        return None;
    }
    Some(tbx)
}

fn tbx_bgzf_tell(fp: &BGZF) -> u64 {
    (fp.block_address as u64) << 16 | fp.block_offset as u64
}

pub unsafe fn tbx_index_build(fn_: *const c_char, min_shift: c_int, conf: &tbx_conf_t) -> c_int {
    tbx_c_547_tbx_index_build(fn_, min_shift, conf)
}

pub unsafe fn tbx_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    conf: &tbx_conf_t,
) -> c_int {
    tbx_c_542_tbx_index_build2(fn_, fnidx, min_shift, conf)
}

pub unsafe fn tbx_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    n_threads: c_int,
    conf: &tbx_conf_t,
) -> c_int {
    tbx_c_526_tbx_index_build3(fn_, fnidx, min_shift, n_threads, conf)
}

// original: tbx_index_build3 (htslib/tbx.c:526)
pub unsafe fn tbx_c_526_tbx_index_build3(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    n_threads: c_int,
    conf: &tbx_conf_t,
) -> c_int {
    let fp = bgzf_open(fn_, c"r".as_ptr());
    if fp.is_null() {
        return -1;
    }
    if n_threads > 1 && bgzf_mt(fp, n_threads, 256) != 0 {
        bgzf_close(fp);
        return -1;
    }
    if bgzf_compression(fp) != HTS_COMPRESSION_BGZF {
        bgzf_close(fp);
        return -2;
    }
    let tbx = fp
        .as_mut()
        .and_then(|fp| tbx_index_owned(fp, min_shift, conf));
    bgzf_close(fp);
    let Some(tbx) = tbx else {
        return -1;
    };
    let ret = hts_c_2869_hts_idx_save_as(
        tbx.idx_ptr().cast(),
        fn_,
        fnidx,
        if min_shift > 0 {
            HTS_FMT_CSI
        } else {
            HTS_FMT_TBI
        },
    );
    ret
}

// original: tbx_index_build2 (htslib/tbx.c:542)
pub unsafe fn tbx_c_542_tbx_index_build2(
    fn_: *const c_char,
    fnidx: *const c_char,
    min_shift: c_int,
    conf: &tbx_conf_t,
) -> c_int {
    tbx_c_526_tbx_index_build3(fn_, fnidx, min_shift, 0, conf)
}

// original: tbx_index_build (htslib/tbx.c:547)
pub unsafe fn tbx_c_547_tbx_index_build(
    fn_: *const c_char,
    min_shift: c_int,
    conf: &tbx_conf_t,
) -> c_int {
    tbx_c_526_tbx_index_build3(fn_, std::ptr::null(), min_shift, 0, conf)
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

pub unsafe fn tbx_seqnames(tbx: *const tbx_t, n: *mut c_int) -> *mut *const c_char {
    let Some(tbx) = tbx.as_ref() else {
        return std::ptr::null_mut();
    };
    let Some(n) = n.as_mut() else {
        return std::ptr::null_mut();
    };
    let names_ref = tbx_c_614_tbx_seqnames(tbx);
    let names = libc::calloc(names_ref.len().max(1), std::mem::size_of::<*const c_char>())
        .cast::<*const c_char>();
    if names.is_null() {
        *n = 0;
        return std::ptr::null_mut();
    }
    for (i, name) in names_ref.iter().enumerate() {
        *names.add(i) = name.as_ptr().cast();
    }
    *n = names_ref.len() as c_int;
    names
}

pub unsafe fn tbx_destroy(tbx: *mut tbx_t) {
    tbx_c_512_tbx_destroy(tbx)
}

unsafe fn tbx_log_invalid_index_header(target: &[u8]) {
    let msg = std::ffi::CString::new(format!(
        "Invalid index header for {}",
        String::from_utf8_lossy(target)
    ))
    .unwrap();
    hts_log_cstr(HTS_LOG_ERROR, c"index_load".as_ptr(), msg.as_ptr());
}

fn tbx_index_name_bytes(record: &[u8]) -> Option<&[u8]> {
    let name = record.strip_suffix(&[0])?;
    (!name.contains(&0)).then_some(name)
}

pub unsafe fn tbx_c_552_index_load(
    fn_: *const c_char,
    fnidx: *const c_char,
    flags: c_int,
) -> *mut tbx_t {
    let Some(fn_) = (!fn_.is_null()).then(|| CStr::from_ptr(fn_)) else {
        return std::ptr::null_mut();
    };
    let fnidx = (!fnidx.is_null()).then(|| CStr::from_ptr(fnidx));
    tbx_index_load_owned(fn_, fnidx, flags).map_or(std::ptr::null_mut(), Box::into_raw)
}

unsafe fn tbx_index_load_owned(
    fn_: &CStr,
    fnidx: Option<&CStr>,
    flags: c_int,
) -> Option<Box<tbx_t>> {
    let mut tbx = Box::new(tbx_t::new(tbx_conf_t {
        preset: 0,
        sc: 0,
        bc: 0,
        ec: 0,
        meta_char: 0,
        line_skip: 0,
    }));
    let fnidx_ptr = fnidx.map_or(std::ptr::null(), CStr::as_ptr);
    if !tbx.set_idx(NonNull::new(
        hts_idx_load3(fn_.as_ptr(), fnidx_ptr, HTS_FMT_TBI, flags).cast(),
    )) {
        return None;
    }

    let mut l_meta = 0u32;
    let meta_ptr = hts_c_3084_hts_idx_get_meta(tbx.idx_ptr().cast(), &mut l_meta);
    if meta_ptr.is_null() || l_meta < 28 {
        tbx_log_invalid_index_header(fnidx.unwrap_or(fn_).to_bytes());
        return None;
    }
    let meta = std::slice::from_raw_parts(meta_ptr, l_meta as usize);

    tbx.conf.preset = i32::from_le_bytes(meta[0..4].try_into().unwrap());
    tbx.conf.sc = i32::from_le_bytes(meta[4..8].try_into().unwrap());
    tbx.conf.bc = i32::from_le_bytes(meta[8..12].try_into().unwrap());
    tbx.conf.ec = i32::from_le_bytes(meta[12..16].try_into().unwrap());
    tbx.conf.meta_char = i32::from_le_bytes(meta[16..20].try_into().unwrap());
    tbx.conf.line_skip = i32::from_le_bytes(meta[20..24].try_into().unwrap());
    let l_nm = u32::from_le_bytes(meta[24..28].try_into().unwrap());
    if l_nm > l_meta - 28 {
        tbx_log_invalid_index_header(fnidx.unwrap_or(fn_).to_bytes());
        return None;
    }

    let names = &meta[28..28 + l_nm as usize];
    for name in names.split_inclusive(|&byte| byte == 0) {
        let Some(name) = tbx_index_name_bytes(name) else {
            tbx_log_invalid_index_header(fnidx.unwrap_or(fn_).to_bytes());
            return None;
        };
        if tbx_c_64_get_tid(&mut tbx, name, 1) < 0 {
            let msg = std::ffi::CString::new(std::io::Error::last_os_error().to_string())
                .unwrap_or_else(|_| c"index_load failed".to_owned());
            hts_log_cstr(HTS_LOG_ERROR, c"index_load".as_ptr(), msg.as_ptr());
            return None;
        }
    }
    Some(tbx)
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

pub fn tbx_c_614_tbx_seqnames(tbx: &tbx_t) -> &[Vec<u8>] {
    tbx.dict
        .as_ref()
        .map_or([].as_slice(), |dict| dict.names.as_slice())
}

pub unsafe extern "C" fn tbx_c_644_tbx_name2id_wrapper(
    vhdr: *mut c_void,
    ref_: *const c_char,
) -> c_int {
    let Some(tbx) = vhdr.cast::<tbx_t>().as_ref() else {
        return -1;
    };
    let Some(ref_) = (!ref_.is_null()).then(|| CStr::from_ptr(ref_)) else {
        return -1;
    };
    tbx_c_91_tbx_name2id(tbx, ref_.to_bytes())
}

pub unsafe fn tbx_itr_querys1(tbx: *mut tbx_t, region: *const c_char) -> *mut hts_itr_t {
    let Some(tbx) = tbx.as_mut() else {
        return std::ptr::null_mut();
    };
    let Some(region) = (!region.is_null()).then(|| CStr::from_ptr(region)) else {
        return std::ptr::null_mut();
    };
    tbx_c_649_tbx_itr_querys1(tbx, region.to_bytes(), region.as_ptr())
}

// original: tbx_itr_querys1 (htslib/tbx.c:649)
pub unsafe fn tbx_c_649_tbx_itr_querys1(
    tbx: &mut tbx_t,
    region: &[u8],
    region_ptr: *const c_char,
) -> *mut hts_itr_t {
    let mut tid = 0;
    let mut beg = 0;
    let mut end = 0;

    if region == b"." {
        return hts_itr_query(tbx.idx_ptr().cast(), HTS_IDX_START, 0, 0, Some(tbx_readrec));
    } else if region == b"*" {
        return hts_itr_query(
            tbx.idx_ptr().cast(),
            HTS_IDX_NOCOOR,
            0,
            0,
            Some(tbx_readrec),
        );
    }

    if hts_parse_region(
        region_ptr,
        &mut tid,
        &mut beg,
        &mut end,
        Some(tbx_c_644_tbx_name2id_wrapper),
        (tbx as *mut tbx_t).cast(),
        HTS_PARSE_THOUSANDS_SEP,
    )
    .is_null()
    {
        return std::ptr::null_mut();
    }

    hts_itr_query(tbx.idx_ptr().cast(), tid, beg, end, Some(tbx_readrec))
}

pub unsafe fn tbx_c_512_tbx_destroy(tbx: *mut tbx_t) {
    if tbx.is_null() {
        return;
    }
    drop(Box::from_raw(tbx));
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

            let gaf = tbx_conf_gaf();
            assert_eq!(gaf.preset, TBX_GAF);
            assert_eq!(gaf.sc, 1);
            assert_eq!(gaf.bc, 6);
            assert_eq!(gaf.ec, 0);
            assert_eq!(gaf.meta_char, b'#' as c_int);
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
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 99);
            assert_eq!(intv.end, 111);
            assert_eq!(
                c"chr1".to_bytes(),
                &line[intv.ss.unwrap()..intv.se.unwrap()]
            );
        }
    }

    #[test]
    fn tbx_parse1_vcf_gvcf_symbolic_alt_uses_format_len() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line = b"chr1\t100\t.\tA\t<*>\t.\t.\t.\tGT:LEN\t0/0:12\t0/1:5\0".to_vec();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 99);
            assert_eq!(intv.end, 111);
        }
    }

    #[test]
    fn tbx_parse1_vcf_gvcf_len_field_can_follow_other_format_fields() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line =
                b"chr1\t100\t.\tA\t<NON_REF>\t.\t.\t.\tGT:DP:LEN\t0/0:7:4\t0/0:8:11\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: -1,
                end: -1,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 110));
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_parse1_vcf_gvcf_without_len_uses_ref_span_and_restores_line() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line = b"chr1\t100\t.\tAC\t<NON_REF>\t.\t.\t.\tGT:DP\t0/0:12\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: -1,
                end: -1,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 101));
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_parse1_vcf_clamps_non_positive_pos_to_first_base() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line = b"chr1\t0\t.\tA\tC\t.\t.\t.\0".to_vec();
            let mut intv = tbx_intv_t {
                beg: -1,
                end: -1,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 0);
            assert_eq!(intv.end, 1);
        }
    }

    #[test]
    fn tbx_parse1_vcf_end_requires_info_key_boundary_and_restores_line() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            let mut non_key_end = b"chr1\t100\t.\tA\tC\t.\t.\tXEND=500\0".to_vec();
            let original = non_key_end.clone();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &non_key_end[..non_key_end.len() - 1],
                    &mut intv
                ),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 100));
            assert_eq!(non_key_end, original);

            let mut semicolon_end = b"chr1\t100\t.\tA\tC\t.\t.\tNS=1;END=105\0".to_vec();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &semicolon_end[..semicolon_end.len() - 1],
                    &mut intv
                ),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 105));
        }
    }

    #[test]
    fn tbx_parse1_vcf_ignores_missing_or_non_advancing_end_values() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            let mut missing_end = b"chr1\t100\t.\tAC\tA\t.\t.\tEND=.\0".to_vec();
            let original = missing_end.clone();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &missing_end[..missing_end.len() - 1],
                    &mut intv
                ),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 101));
            assert_eq!(missing_end, original);

            let mut before_beg_end = b"chr1\t100\t.\tAC\tA\t.\t.\tEND=50\0".to_vec();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &before_beg_end[..before_beg_end.len() - 1],
                    &mut intv
                ),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 101));
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
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 6);
            assert_eq!(intv.end, 22);
        }
    }

    #[test]
    fn tbx_parse1_sam_zero_reference_cigar_spans_one_base() {
        unsafe {
            let conf = tbx_conf_sam();
            let mut line = b"r001\t0\tref\t7\t30\t5S10I\t*\t0\t0\tACGT\t*\0".to_vec();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 6);
            assert_eq!(intv.end, 7);
        }
    }

    #[test]
    fn tbx_parse1_generic_ucsc_single_position_is_half_open() {
        unsafe {
            let conf = tbx_conf_t {
                preset: TBX_GENERIC | TBX_UCSC,
                sc: 1,
                bc: 2,
                ec: 2,
                meta_char: b'#' as c_int,
                line_skip: 0,
            };
            let mut line = b"chr7\t100\0".to_vec();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 100);
            assert_eq!(intv.end, 101);
            assert_eq!(&line[intv.ss.unwrap()..intv.se.unwrap()], b"chr7");
        }
    }

    #[test]
    fn tbx_parse1_rejects_missing_sequence_or_nonnumeric_begin() {
        unsafe {
            let conf = tbx_conf_t {
                preset: TBX_GENERIC,
                sc: 1,
                bc: 2,
                ec: 3,
                meta_char: b'#' as c_int,
                line_skip: 0,
            };
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            let mut no_seq = b"chr7\0".to_vec();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &no_seq[..no_seq.len() - 1],
                    &mut intv
                ),
                -1
            );

            let mut nonnumeric_begin = b"chr7\tabc\t12\0".to_vec();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &nonnumeric_begin[..nonnumeric_begin.len() - 1],
                    &mut intv
                ),
                -1
            );
        }
    }

    #[test]
    fn tbx_parse1_generic_one_based_closed_interval_is_half_open() {
        unsafe {
            let conf = tbx_conf_t {
                preset: TBX_GENERIC,
                sc: 1,
                bc: 2,
                ec: 3,
                meta_char: b'#' as c_int,
                line_skip: 0,
            };
            let mut line = b"chr7\t10\t12\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 9);
            assert_eq!(intv.end, 12);
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_parse1_generic_missing_end_column_falls_back_to_begin_column() {
        unsafe {
            let conf = tbx_conf_t {
                preset: TBX_GENERIC,
                sc: 1,
                bc: 2,
                ec: 3,
                meta_char: b'#' as c_int,
                line_skip: 0,
            };
            let mut line = b"chr7\t10\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!(intv.beg, 9);
            assert_eq!(intv.end, 10);
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_parse1_bed_preset_keeps_zero_based_half_open_interval() {
        unsafe {
            let conf = tbx_conf_bed();
            assert_eq!(conf.preset & 0xffff, TBX_GENERIC);
            assert_ne!(conf.preset & TBX_UCSC, 0);

            let mut line = b"chr7\t10\t12\tfeature1\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: -1,
                end: -1,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!((intv.beg, intv.end), (10, 12));
            assert_eq!(&line[intv.ss.unwrap()..intv.se.unwrap()], b"chr7");
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_parse1_vcf_svlen_requires_info_key_boundary_and_restores_line() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            let mut non_key_svlen = b"chr1\t100\t.\tA\t<DEL>\t.\t.\tXSVLEN=-50\0".to_vec();
            let original = non_key_svlen.clone();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &non_key_svlen[..non_key_svlen.len() - 1],
                    &mut intv
                ),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 100));
            assert_eq!(non_key_svlen, original);

            let mut semicolon_svlen = b"chr1\t100\t.\tA\t<DEL>\t.\t.\tNS=1;SVLEN=-6\0".to_vec();
            assert_eq!(
                tbx_c_96_tbx_parse1(
                    &conf,
                    &semicolon_svlen[..semicolon_svlen.len() - 1],
                    &mut intv
                ),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 105));
        }
    }

    #[test]
    fn tbx_parse1_vcf_symbolic_svlen_uses_largest_absolute_annotated_alt() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line = b"chr1\t100\t.\tA\t<DEL>,<DUP>,G\t.\t.\tSVLEN=-8,12,50\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 111));
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_parse1_vcf_zero_svlen_does_not_shrink_ref_or_end_span() {
        unsafe {
            let conf = tbx_conf_vcf();
            let mut line = b"chr1\t100\t.\tACGT\t<DEL>\t.\t.\tEND=102;SVLEN=0\0".to_vec();
            let original = line.clone();
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: 0,
            };

            assert_eq!(
                tbx_c_96_tbx_parse1(&conf, &line[..line.len() - 1], &mut intv),
                0
            );
            assert_eq!((intv.beg, intv.end), (99, 103));
            assert_eq!(line, original);
        }
    }

    #[test]
    fn tbx_header_ref_len_scanners_keep_maximum() {
        let mut max_ref_len = 100i64;
        tbx_c_412_adjust_max_ref_len_vcf(b"##contig=<ID=chr1,length=248956422>", &mut max_ref_len);
        assert_eq!(max_ref_len, 248956422);

        tbx_c_425_adjust_max_ref_len_sam(b"@SQ\tSN:chr2\tLN:242193529", &mut max_ref_len);
        assert_eq!(max_ref_len, 248956422);
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
                idx: None,
                dict: None,
            };
            let line = b"read1\t10\t0\t10\t+\t>12>3>99\t0\t0\t255".to_vec();
            let mut str_ = kstring_t { data: line };
            let mut intv = tbx_intv_t {
                beg: 0,
                end: 0,
                ss: None,
                se: None,
                tid: -1,
            };

            assert_eq!(tbx_c_315_get_intv(&mut tbx, &mut str_, &mut intv, 1), 0);
            assert_eq!(intv.tid, 0);
            assert_eq!(intv.beg, 3);
            assert_eq!(intv.end, 99);
            assert!(tbx.dict.is_none());
        }
    }

    #[test]
    fn tbx_gaf_name_lookup_is_single_tid_and_does_not_allocate_dictionary() {
        let mut tbx = tbx_t {
            conf: tbx_conf_t {
                preset: TBX_GAF,
                sc: 1,
                bc: 6,
                ec: 0,
                meta_char: b'#' as c_int,
                line_skip: 0,
            },
            idx: None,
            dict: None,
        };

        assert_eq!(tbx_c_91_tbx_name2id(&tbx, b"any-path"), 0);
        assert_eq!(tbx_c_64_get_tid(&mut tbx, b"other-path", 1), 0);
        assert!(tbx.dict.is_none());
    }

    #[test]
    fn tbx_seqnames_returns_empty_slice_for_null_dictionary() {
        unsafe {
            let tbx = tbx_t {
                conf: tbx_conf_vcf(),
                idx: None,
                dict: None,
            };
            let names = tbx_c_614_tbx_seqnames(&tbx);
            assert!(names.is_empty());
        }
    }

    #[test]
    fn tbx_seqnames_abi_returns_freeable_empty_array_for_null_dictionary() {
        unsafe {
            let tbx = tbx_t {
                conf: tbx_conf_vcf(),
                idx: None,
                dict: None,
            };
            let mut n = -1;
            let names = tbx_seqnames(&tbx, &mut n);
            assert!(!names.is_null());
            assert_eq!(n, 0);
            libc::free(names.cast());
        }
    }

    #[test]
    fn tbx_name_lookup_on_empty_dictionary_reports_missing_without_names() {
        unsafe {
            let tbx = Box::into_raw(Box::new(tbx_t::new(tbx_conf_vcf())));
            let tbx_ref = &*tbx;

            assert_eq!(tbx_name2id(tbx_ref, b"chr1"), -1);
            assert!(tbx_ref.dict.is_none());
            tbx_c_512_tbx_destroy(tbx);
        }
    }

    #[test]
    fn tbx_get_tid_seqnames_and_destroy_use_s2i_dictionary() {
        unsafe {
            let tbx = Box::into_raw(Box::new(tbx_t::new(tbx_conf_vcf())));
            let tbx_ref = &mut *tbx;

            assert_eq!(tbx_c_64_get_tid(tbx_ref, b"chr2", 1), 0);
            assert_eq!(tbx_c_64_get_tid(tbx_ref, b"chr1", 1), 1);
            assert_eq!(tbx_c_91_tbx_name2id(tbx_ref, b"chr2"), 0);
            assert_eq!(tbx_c_91_tbx_name2id(tbx_ref, b"missing"), -1);

            let names = tbx_c_614_tbx_seqnames(tbx_ref);
            assert_eq!(names.len(), 2);
            assert_eq!(names[0], b"chr2\0");
            assert_eq!(names[1], b"chr1\0");

            tbx_c_512_tbx_destroy(tbx);
        }
    }

    #[test]
    fn tbx_index_builds_dictionary_from_bgzf_records() {
        unsafe {
            let path = std::env::temp_dir().join(format!(
                "htslib_rs-tbx-index-{}-{}.vcf.gz",
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

            let names = tbx_c_614_tbx_seqnames(&*tbx);
            assert_eq!(names.len(), 2);
            assert_eq!(names[0], b"chr2\0");
            assert_eq!(names[1], b"chr1\0");
            tbx_c_512_tbx_destroy(tbx);
            libc::unlink(c_path.as_ptr());
        }
    }
}
