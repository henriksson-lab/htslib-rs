use std::{
    ffi::{c_char, c_int, c_void, CStr},
    fs,
    io::Write,
};

use super::bgzf::{
    bgzf_close, bgzf_getc, bgzf_open, bgzf_read, bgzf_set_cache_size, bgzf_thread_pool, bgzf_useek,
};
use super::hts::{hts_pos_t, BGZF, HTS_POS_MAX};
use super::thread_pool::hts_tpool;
use super::{path_bytes, path_from_bytes};

pub const FAI_CREATE: c_int = 0x01;
pub type fai_format_options = u32;
pub const FAI_NONE: fai_format_options = 0;
pub const FAI_FASTA: fai_format_options = 1;
pub const FAI_FASTQ: fai_format_options = 2;
const HTS_PARSE_THOUSANDS_SEP: c_int = 1;
const HTS_PARSE_ONE_COORD: c_int = 2;
const HTS_PARSE_LIST: c_int = 4;

extern "C" {
    fn free(ptr: *mut c_void);
    fn malloc(size: usize) -> *mut c_void;
}

pub fn isgraph_(c: u8) -> c_int {
    (c > b' ' && c <= b'~') as c_int
}

pub unsafe fn bgzf_getc_(fp: *mut BGZF) -> c_int {
    if (*fp).block_offset + 1 < (*fp).block_length {
        let c = *(*fp)
            .uncompressed_block
            .cast::<u8>()
            .add((*fp).block_offset as usize);
        (*fp).block_offset += 1;
        (*fp).uncompressed_address += 1;
        return c as c_int;
    }

    bgzf_getc(fp)
}

pub unsafe fn fai_path(fa: *const c_char) -> *mut c_char {
    if fa.is_null() {
        return std::ptr::null_mut();
    }
    let delim = c"##idx##";
    let fai_tmp = libc::strstr(fa, delim.as_ptr());
    if fai_tmp.is_null() {
        return std::ptr::null_mut();
    }
    libc::strdup(fai_tmp.add(delim.to_bytes().len()))
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct faidx1_t {
    pub id: c_int,
    pub line_len: u32,
    pub line_blen: u32,
    pub len: u64,
    pub seq_offset: u64,
    pub qual_offset: u64,
}

#[repr(C)]
pub struct faidx_hash_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    pub flags: *mut u32,
    pub keys: *mut *mut c_char,
    pub vals: *mut faidx1_t,
}

#[repr(C)]
pub struct faidx_t {
    pub bgzf: *mut BGZF,
    pub n: c_int,
    pub m: c_int,
    pub name: *mut *mut c_char,
    pub hash: *mut faidx_hash_t,
    pub format: c_int,
}

pub unsafe fn fai_load3(
    _fn_: *const c_char,
    _fnfai: *const c_char,
    _fngzi: *const c_char,
    _flags: c_int,
) -> *mut faidx_t {
    if !_fngzi.is_null() {
        return std::ptr::null_mut();
    }
    match fai_load_existing(_fn_, _fnfai) {
        Some(fai) => fai,
        None if (_flags & FAI_CREATE) != 0 => match fai_build_plain_fasta(_fn_, _fnfai) {
            Some(fai) => fai,
            None => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

pub unsafe fn fai_load(_fn_: *const c_char) -> *mut faidx_t {
    fai_load3(_fn_, std::ptr::null(), std::ptr::null(), FAI_CREATE)
}

pub unsafe fn fai_build3(fn_: *const c_char, fnfai: *const c_char, fngzi: *const c_char) -> c_int {
    if !fngzi.is_null() {
        return -1;
    }
    match fai_build_plain_fasta(fn_, fnfai) {
        Some(fai) => {
            fai_destroy(fai);
            0
        }
        None => -1,
    }
}

pub unsafe fn fai_build(fn_: *const c_char) -> c_int {
    fai_build3(fn_, std::ptr::null(), std::ptr::null())
}

pub unsafe fn fai_load3_format(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
    format: fai_format_options,
) -> *mut faidx_t {
    match format {
        FAI_NONE | FAI_FASTA => fai_load3(fn_, fnfai, fngzi, flags),
        FAI_FASTQ => std::ptr::null_mut(),
        _ => std::ptr::null_mut(),
    }
}

pub unsafe fn fai_load_format(fn_: *const c_char, format: fai_format_options) -> *mut faidx_t {
    fai_load3_format(fn_, std::ptr::null(), std::ptr::null(), FAI_CREATE, format)
}

unsafe fn fai_load_existing(fn_: *const c_char, fnfai: *const c_char) -> Option<*mut faidx_t> {
    if fn_.is_null() {
        return None;
    }
    let fasta_path = path_from_bytes(CStr::from_ptr(fn_).to_bytes());
    let fai_path = if fnfai.is_null() {
        let mut bytes = path_bytes(&fasta_path).into_owned();
        bytes.extend_from_slice(b".fai");
        path_from_bytes(&bytes)
    } else {
        path_from_bytes(CStr::from_ptr(fnfai).to_bytes())
    };
    let text = fs::read_to_string(&fai_path).ok()?;
    let mut rows = Vec::new();
    for (id, line) in text.lines().enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 5 {
            return None;
        }
        let name = fields[0].as_bytes().to_vec();
        let len = fields[1].parse::<u64>().ok()?;
        let seq_offset = fields[2].parse::<u64>().ok()?;
        let line_blen = fields[3].parse::<u32>().ok()?;
        let line_len = fields[4].parse::<u32>().ok()?;
        if name.is_empty() || line_blen == 0 || line_len < line_blen {
            return None;
        }
        rows.push((
            name,
            faidx1_t {
                id: id as c_int,
                line_len,
                line_blen,
                len,
                seq_offset,
                qual_offset: 0,
            },
        ));
    }
    if rows.is_empty() {
        return None;
    }

    fai_from_rows(fn_, rows)
}

unsafe fn fai_from_rows(
    fn_: *const c_char,
    rows: Vec<(Vec<u8>, faidx1_t)>,
) -> Option<*mut faidx_t> {
    let fai = malloc(std::mem::size_of::<faidx_t>()).cast::<faidx_t>();
    if fai.is_null() {
        return None;
    }
    std::ptr::write_bytes(fai, 0, 1);
    (*fai).n = 0;
    (*fai).m = rows.len() as c_int;
    (*fai).format = 0;
    (*fai).bgzf = bgzf_open(fn_, b"r\0".as_ptr().cast());
    if (*fai).bgzf.is_null() {
        free(fai.cast());
        return None;
    }
    (*fai).name = malloc(rows.len() * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    if (*fai).name.is_null() {
        fai_destroy(fai);
        return None;
    }

    let mut n_buckets = 2usize;
    while n_buckets < rows.len() * 2 {
        n_buckets <<= 1;
    }
    let hash = malloc(std::mem::size_of::<faidx_hash_t>()).cast::<faidx_hash_t>();
    if hash.is_null() {
        fai_destroy(fai);
        return None;
    }
    std::ptr::write_bytes(hash, 0, 1);
    (*hash).n_buckets = n_buckets as u32;
    (*hash).size = rows.len() as u32;
    (*hash).n_occupied = rows.len() as u32;
    (*hash).upper_bound = (n_buckets as f64 * 0.77) as u32;
    let n_flags = if n_buckets < 16 { 1 } else { n_buckets >> 4 };
    (*hash).flags = malloc(n_flags * std::mem::size_of::<u32>()).cast::<u32>();
    (*hash).keys = malloc(n_buckets * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    (*hash).vals = malloc(n_buckets * std::mem::size_of::<faidx1_t>()).cast::<faidx1_t>();
    if (*hash).flags.is_null() || (*hash).keys.is_null() || (*hash).vals.is_null() {
        (*fai).hash = hash;
        fai_destroy(fai);
        return None;
    }
    for i in 0..n_flags {
        *(*hash).flags.add(i) = 0xaaaa_aaaa;
    }
    (*fai).hash = hash;

    for (i, (name, val)) in rows.into_iter().enumerate() {
        let name_ptr = malloc(name.len() + 1).cast::<c_char>();
        if name_ptr.is_null() {
            fai_destroy(fai);
            return None;
        }
        std::ptr::copy_nonoverlapping(name.as_ptr().cast::<c_char>(), name_ptr, name.len());
        *name_ptr.add(name.len()) = 0;
        *(*fai).name.add(i) = name_ptr;
        (*fai).n += 1;

        let mask = (*hash).n_buckets - 1;
        let mut k = kh_str_hash_string(name_ptr) & mask;
        let mut step = 0;
        while !kh_isempty((*hash).flags, k) {
            step += 1;
            k = (k + step) & mask;
        }
        *(*hash).keys.add(k as usize) = name_ptr;
        *(*hash).vals.add(k as usize) = val;
        let flag = (*hash).flags.add((k >> 4) as usize);
        *flag &= !(3 << ((k & 0x0f) << 1));
    }

    Some(fai)
}

unsafe fn fai_build_plain_fasta(fn_: *const c_char, fnfai: *const c_char) -> Option<*mut faidx_t> {
    if fn_.is_null() {
        return None;
    }
    let fasta_path = path_from_bytes(CStr::from_ptr(fn_).to_bytes());
    let fai_path = if fnfai.is_null() {
        let mut bytes = path_bytes(&fasta_path).into_owned();
        bytes.extend_from_slice(b".fai");
        path_from_bytes(&bytes)
    } else {
        path_from_bytes(CStr::from_ptr(fnfai).to_bytes())
    };

    let data = fs::read(&fasta_path).ok()?;
    let mut rows: Vec<(Vec<u8>, faidx1_t)> = Vec::new();
    let mut i = 0usize;
    while i < data.len() {
        let c = data[i];
        if c == b'\n' || c == b'\r' {
            i += 1;
            continue;
        }
        if c != b'>' {
            return None;
        }
        i += 1;
        while i < data.len() && data[i].is_ascii_whitespace() && data[i] != b'\n' {
            i += 1;
        }
        let name_start = i;
        while i < data.len() && !data[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == name_start {
            return None;
        }
        let name = data[name_start..i].to_vec();
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        if i < data.len() {
            i += 1;
        }
        if i >= data.len() {
            return None;
        }
        let seq_offset = i as u64;
        let mut seq_len = 0u64;
        let mut line_len = 0u32;
        let mut line_blen = 0u32;
        let mut final_short_line = false;

        while i < data.len() {
            if data[i] == b'>' {
                break;
            }
            if data[i] == b'\n' || data[i] == b'\r' {
                i += 1;
                continue;
            }
            if final_short_line {
                return None;
            }

            let line_start = i;
            let mut ll = 0u32;
            let mut bl = 0u32;
            while i < data.len() && data[i] != b'\n' {
                ll += 1;
                if !data[i].is_ascii_whitespace() {
                    bl += 1;
                }
                i += 1;
            }
            if i < data.len() && data[i] == b'\n' {
                ll += 1;
                i += 1;
            }
            if bl == 0 {
                if seq_len == 0 {
                    return None;
                }
                break;
            }
            seq_len += bl as u64;
            if line_len == 0 {
                line_len = ll;
                line_blen = bl;
            } else if line_len < ll {
                return None;
            } else if line_len > ll {
                final_short_line = true;
            }
            if line_start == i {
                return None;
            }
        }

        if seq_len == 0 || line_len == 0 || line_blen == 0 {
            return None;
        }
        rows.push((
            name,
            faidx1_t {
                id: rows.len() as c_int,
                line_len,
                line_blen,
                len: seq_len,
                seq_offset,
                qual_offset: 0,
            },
        ));
    }

    if rows.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for (name, val) in &rows {
        out.extend_from_slice(name);
        writeln!(
            &mut out,
            "\t{}\t{}\t{}\t{}",
            val.len, val.seq_offset, val.line_blen, val.line_len
        )
        .ok()?;
    }
    fs::write(&fai_path, out).ok()?;

    fai_from_rows(fn_, rows)
}

pub unsafe fn fai_destroy(_fai: *mut faidx_t) {
    if _fai.is_null() {
        return;
    }
    for i in 0..(*_fai).n {
        free(*(*_fai).name.add(i as usize).cast::<*mut c_void>());
    }
    free((*_fai).name.cast());
    kh_destroy_s((*_fai).hash);
    if !(*_fai).bgzf.is_null() {
        bgzf_close((*_fai).bgzf);
    }
    free(_fai.cast());
}

pub unsafe fn faidx_has_seq(_fai: *const faidx_t, _seq: *const c_char) -> c_int {
    if _fai.is_null() || _seq.is_null() {
        return 0;
    }
    let h = (*_fai).hash;
    if h.is_null() || kh_get_s(h, _seq) == (*h).n_buckets {
        0
    } else {
        1
    }
}

pub unsafe fn faidx_fetch_nseq(_fai: *const faidx_t) -> c_int {
    (*_fai).n
}

pub unsafe fn faidx_nseq(_fai: *const faidx_t) -> c_int {
    (*_fai).n
}

pub unsafe fn faidx_iseq(_fai: *const faidx_t, _i: c_int) -> *const c_char {
    *(*_fai).name.add(_i as usize)
}

pub unsafe fn faidx_seq_len64(_fai: *const faidx_t, _seq: *const c_char) -> hts_pos_t {
    let k = kh_get_s((*_fai).hash, _seq);
    if k == (*(*_fai).hash).n_buckets {
        -1
    } else {
        (*(*(*_fai).hash).vals.add(k as usize)).len as hts_pos_t
    }
}

pub unsafe fn faidx_seq_len(_fai: *const faidx_t, _seq: *const c_char) -> c_int {
    let len = faidx_seq_len64(_fai, _seq);
    if len < c_int::MAX as hts_pos_t {
        len as c_int
    } else {
        c_int::MAX
    }
}

pub unsafe fn fai_adjust_region(
    _fai: *const faidx_t,
    _tid: c_int,
    _beg: *mut hts_pos_t,
    _end: *mut hts_pos_t,
) -> c_int {
    if _fai.is_null() || _beg.is_null() || _end.is_null() || _tid < 0 || _tid >= (*_fai).n {
        return -1;
    }

    let orig_beg = *_beg;
    let orig_end = *_end;
    if faidx_adjust_position(
        _fai,
        0,
        std::ptr::null_mut(),
        *(*_fai).name.add(_tid as usize),
        _beg,
        _end,
        std::ptr::null_mut(),
    ) != 0
    {
        return -1;
    }

    (if orig_beg != *_beg { 1 } else { 0 })
        | (if orig_end != *_end && orig_end < HTS_POS_MAX {
            2
        } else {
            0
        })
}

pub unsafe fn faidx_fetch_seq64(
    fai: *const faidx_t,
    c_name: *const c_char,
    mut p_beg_i: hts_pos_t,
    mut p_end_i: hts_pos_t,
    len: *mut hts_pos_t,
) -> *mut c_char {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };

    if faidx_adjust_position(fai, 1, &mut val, c_name, &mut p_beg_i, &mut p_end_i, len) != 0 {
        return std::ptr::null_mut();
    }

    fai_retrieve(fai, &val, val.seq_offset, p_beg_i, p_end_i + 1, len)
}

pub unsafe fn faidx_fetch_seq(
    fai: *const faidx_t,
    c_name: *const c_char,
    p_beg_i: c_int,
    p_end_i: c_int,
    len: *mut c_int,
) -> *mut c_char {
    let mut len64 = 0;
    let ret = faidx_fetch_seq64(
        fai,
        c_name,
        p_beg_i as hts_pos_t,
        p_end_i as hts_pos_t,
        &mut len64,
    );
    *len = if len64 < c_int::MAX as hts_pos_t {
        len64 as c_int
    } else {
        c_int::MAX
    };
    ret
}

unsafe extern "C" fn fai_name2id(v: *mut c_void, ref_: *const c_char) -> c_int {
    let fai = v.cast::<faidx_t>();
    let k = kh_get_s((*fai).hash, ref_);
    if k == (*(*fai).hash).n_buckets {
        -1
    } else {
        (*(*(*fai).hash).vals.add(k as usize)).id
    }
}

fn parse_region_decimal(bytes: &[u8], mut i: usize, flags: c_int) -> Option<(hts_pos_t, usize)> {
    let negative = bytes.get(i) == Some(&b'-');
    if negative {
        i += 1;
    }
    let mut value = 0_i64;
    let mut saw_digit = false;
    while let Some(&b) = bytes.get(i) {
        if b.is_ascii_digit() {
            saw_digit = true;
            value = value.saturating_mul(10).saturating_add((b - b'0') as i64);
            i += 1;
        } else if b == b',' && (flags & HTS_PARSE_THOUSANDS_SEP) != 0 {
            i += 1;
        } else {
            break;
        }
    }
    saw_digit.then_some((if negative { -value } else { value }, i))
}

unsafe fn parse_region_name(fai: *const faidx_t, name: &[u8], tid: *mut c_int) -> Option<c_int> {
    let mut nul_name = Vec::with_capacity(name.len() + 1);
    nul_name.extend_from_slice(name);
    nul_name.push(0);
    let id = fai_name2id(fai.cast::<c_void>().cast_mut(), nul_name.as_ptr().cast());
    *tid = id;
    (id >= 0).then_some(id)
}

pub unsafe fn fai_parse_region(
    fai: *const faidx_t,
    s: *const c_char,
    tid: *mut c_int,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    flags: c_int,
) -> *const c_char {
    if fai.is_null() || s.is_null() || tid.is_null() || beg.is_null() || end.is_null() {
        return std::ptr::null();
    }

    let mut parse_flags = flags;
    if (parse_flags & HTS_PARSE_LIST) != 0 {
        parse_flags &= !HTS_PARSE_THOUSANDS_SEP;
    } else {
        parse_flags |= HTS_PARSE_THOUSANDS_SEP;
    }

    let bytes = CStr::from_ptr(s).to_bytes();
    let mut item_len = bytes.len();
    if (parse_flags & HTS_PARSE_LIST) != 0 {
        if let Some(pos) = bytes.iter().position(|&b| b == b',') {
            item_len = pos;
        }
    }
    let item = &bytes[..item_len];
    let s_end = s.add(item_len + usize::from(item_len < bytes.len()));

    let (name_start, name_end, coord_start, quoted) = if item.first() == Some(&b'{') {
        let Some(close) = item.iter().position(|&b| b == b'}') else {
            *tid = -1;
            return std::ptr::null();
        };
        let coord_start = if item.get(close + 1) == Some(&b':') {
            Some(close + 2)
        } else {
            None
        };
        (1, close, coord_start, true)
    } else {
        let colon = item.iter().rposition(|&b| b == b':');
        (0, colon.unwrap_or(item.len()), colon.map(|p| p + 1), false)
    };

    if coord_start.is_none() {
        *beg = 0;
        *end = HTS_POS_MAX;
        return if parse_region_name(fai, &item[name_start..name_end], tid).is_some() {
            s_end
        } else {
            std::ptr::null()
        };
    }

    if !quoted && parse_region_name(fai, item, tid).is_some() {
        let mut prefix_tid = -1;
        if parse_region_name(fai, &item[..name_end], &mut prefix_tid).is_some() {
            *tid = -1;
            return std::ptr::null();
        }
        *beg = 0;
        *end = HTS_POS_MAX;
        return s_end;
    }

    let Some(_) = parse_region_name(fai, &item[name_start..name_end], tid) else {
        return std::ptr::null();
    };
    let mut i = coord_start.unwrap();
    let Some((parsed_beg, next_i)) = parse_region_decimal(item, i, parse_flags) else {
        return std::ptr::null();
    };
    i = next_i;
    *beg = parsed_beg - 1;
    if *beg < 0 {
        if (*beg != -1 && item.get(i) == Some(&b'-') && coord_start != Some(item.len()))
            || !matches!(item.get(i), Some(b'0'..=b'9') | Some(b',') | None)
        {
            return std::ptr::null();
        }
        *end = if *beg == -1 { HTS_POS_MAX } else { -(*beg + 1) };
        *beg = 0;
        return s_end;
    }

    if i == item.len() || ((parse_flags & HTS_PARSE_LIST) != 0 && item.get(i) == Some(&b',')) {
        *end = if (parse_flags & HTS_PARSE_ONE_COORD) != 0 {
            *beg + 1
        } else {
            HTS_POS_MAX
        };
    } else if item.get(i) == Some(&b'-') {
        i += 1;
        if i == item.len() {
            *end = HTS_POS_MAX;
        } else {
            let Some((parsed_end, next_i)) = parse_region_decimal(item, i, parse_flags) else {
                return std::ptr::null();
            };
            i = next_i;
            if i != item.len()
                && !((parse_flags & HTS_PARSE_LIST) != 0 && item.get(i) == Some(&b','))
            {
                return std::ptr::null();
            }
            *end = parsed_end;
        }
    } else {
        return std::ptr::null();
    }

    if *end == 0 {
        *end = HTS_POS_MAX;
    }
    if *beg >= *end {
        return std::ptr::null();
    }
    s_end
}

pub unsafe fn fai_set_cache_size(fai: *mut faidx_t, cache_size: c_int) {
    bgzf_set_cache_size((*fai).bgzf, cache_size);
}

pub unsafe fn fai_thread_pool(fai: *mut faidx_t, pool: *mut hts_tpool, qsize: c_int) -> c_int {
    bgzf_thread_pool((*fai).bgzf, pool, qsize)
}

unsafe fn fai_get_val(
    fai: *const faidx_t,
    str_: *const c_char,
    len: *mut hts_pos_t,
    val: *mut faidx1_t,
    fbeg: *mut hts_pos_t,
    fend: *mut hts_pos_t,
) -> c_int {
    let mut id = 0;
    let mut beg = 0;
    let mut end = 0;

    if fai_parse_region(fai, str_, &mut id, &mut beg, &mut end, 0).is_null() {
        *len = -2;
        return 1;
    }

    let iter = kh_get_s((*fai).hash, faidx_iseq(fai, id));
    if iter >= (*(*fai).hash).n_buckets {
        std::process::abort();
    }
    *val = *(*(*fai).hash).vals.add(iter as usize);

    if beg >= (*val).len as hts_pos_t {
        beg = (*val).len as hts_pos_t;
    }
    if end >= (*val).len as hts_pos_t {
        end = (*val).len as hts_pos_t;
    }
    if beg > end {
        beg = end;
    }

    *fbeg = beg;
    *fend = end;
    0
}

pub unsafe fn fai_line_length(fai: *const faidx_t, str_: *const c_char) -> hts_pos_t {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };
    let mut beg = 0;
    let mut end = 0;
    let mut len = 0;
    if fai_get_val(fai, str_, &mut len, &mut val, &mut beg, &mut end) != 0 {
        -1
    } else {
        val.line_blen as hts_pos_t
    }
}

pub unsafe fn fai_fetch64(
    fai: *const faidx_t,
    str_: *const c_char,
    len: *mut hts_pos_t,
) -> *mut c_char {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };
    let mut beg = 0;
    let mut end = 0;

    if fai_get_val(fai, str_, len, &mut val, &mut beg, &mut end) != 0 {
        return std::ptr::null_mut();
    }
    fai_retrieve(fai, &val, val.seq_offset, beg, end, len)
}

pub unsafe fn fai_fetch(fai: *const faidx_t, str_: *const c_char, len: *mut c_int) -> *mut c_char {
    let mut len64 = 0;
    let ret = fai_fetch64(fai, str_, &mut len64);
    *len = if len64 < c_int::MAX as hts_pos_t {
        len64 as c_int
    } else {
        c_int::MAX
    };
    ret
}

pub unsafe fn fai_fetchqual64(
    fai: *const faidx_t,
    str_: *const c_char,
    len: *mut hts_pos_t,
) -> *mut c_char {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };
    let mut beg = 0;
    let mut end = 0;

    if fai_get_val(fai, str_, len, &mut val, &mut beg, &mut end) != 0 {
        return std::ptr::null_mut();
    }
    fai_retrieve(fai, &val, val.qual_offset, beg, end, len)
}

pub unsafe fn fai_fetchqual(
    fai: *const faidx_t,
    str_: *const c_char,
    len: *mut c_int,
) -> *mut c_char {
    let mut len64 = 0;
    let ret = fai_fetchqual64(fai, str_, &mut len64);
    *len = if len64 < c_int::MAX as hts_pos_t {
        len64 as c_int
    } else {
        c_int::MAX
    };
    ret
}

pub unsafe fn faidx_fetch_qual64(
    fai: *const faidx_t,
    c_name: *const c_char,
    mut p_beg_i: hts_pos_t,
    mut p_end_i: hts_pos_t,
    len: *mut hts_pos_t,
) -> *mut c_char {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };

    if faidx_adjust_position(fai, 1, &mut val, c_name, &mut p_beg_i, &mut p_end_i, len) != 0 {
        return std::ptr::null_mut();
    }

    fai_retrieve(fai, &val, val.qual_offset, p_beg_i, p_end_i + 1, len)
}

pub unsafe fn faidx_fetch_qual(
    fai: *const faidx_t,
    c_name: *const c_char,
    p_beg_i: c_int,
    p_end_i: c_int,
    len: *mut c_int,
) -> *mut c_char {
    let mut len64 = 0;
    let ret = faidx_fetch_qual64(
        fai,
        c_name,
        p_beg_i as hts_pos_t,
        p_end_i as hts_pos_t,
        &mut len64,
    );
    *len = if len64 < c_int::MAX as hts_pos_t {
        len64 as c_int
    } else {
        c_int::MAX
    };
    ret
}

unsafe fn kh_get_s(h: *const faidx_hash_t, key: *const c_char) -> u32 {
    if (*h).n_buckets == 0 {
        return 0;
    }
    let mask = (*h).n_buckets - 1;
    let k = kh_str_hash_string(key);
    let mut i = k & mask;
    let last = i;
    let mut step = 0;
    while !kh_isempty((*h).flags, i)
        && (kh_isdel((*h).flags, i) || !cstr_eq(*(*h).keys.add(i as usize), key))
    {
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return (*h).n_buckets;
        }
    }
    if kh_iseither((*h).flags, i) {
        (*h).n_buckets
    } else {
        i
    }
}

unsafe fn faidx_adjust_position(
    fai: *const faidx_t,
    end_adjust: c_int,
    val_out: *mut faidx1_t,
    c_name: *const c_char,
    p_beg_i: *mut hts_pos_t,
    p_end_i: *mut hts_pos_t,
    len: *mut hts_pos_t,
) -> c_int {
    let iter = kh_get_s((*fai).hash, c_name);
    if iter == (*(*fai).hash).n_buckets {
        if !len.is_null() {
            *len = -2;
        }
        return 1;
    }

    let val = (*(*fai).hash).vals.add(iter as usize);
    if !val_out.is_null() {
        *val_out = *val;
    }

    if *p_end_i < *p_beg_i {
        *p_beg_i = *p_end_i;
    }

    if *p_beg_i < 0 {
        *p_beg_i = 0;
    } else if (*val).len as hts_pos_t <= *p_beg_i {
        *p_beg_i = (*val).len as hts_pos_t;
    }

    if *p_end_i < 0 {
        *p_end_i = 0;
    } else if (*val).len as hts_pos_t <= *p_end_i {
        *p_end_i = (*val).len as hts_pos_t - end_adjust as hts_pos_t;
    }

    0
}

unsafe fn fai_retrieve(
    fai: *const faidx_t,
    val: *const faidx1_t,
    offset: u64,
    beg: hts_pos_t,
    end: hts_pos_t,
    len: *mut hts_pos_t,
) -> *mut c_char {
    if (end as u64).wrapping_sub(beg as u64) >= usize::MAX as u64 - 2 {
        *len = -1;
        return std::ptr::null_mut();
    }

    if (*val).line_blen == 0 {
        *len = -1;
        return std::ptr::null_mut();
    }

    let ret = bgzf_useek(
        (*fai).bgzf,
        (offset
            + (beg as u64 / (*val).line_blen as u64) * (*val).line_len as u64
            + beg as u64 % (*val).line_blen as u64) as i64,
        0,
    );
    if ret < 0 {
        *len = -1;
        return std::ptr::null_mut();
    }

    let buffer_len = (end - beg) as usize + ((*val).line_len - (*val).line_blen) as usize + 1;
    let buffer = malloc(buffer_len).cast::<c_char>();
    if buffer.is_null() {
        *len = -1;
        return std::ptr::null_mut();
    }

    *len = end - beg;
    let mut remaining = *len as isize;
    let firstline_blen = (*val).line_blen as isize - (beg % (*val).line_blen as hts_pos_t) as isize;

    if remaining <= firstline_blen {
        let nread = bgzf_read((*fai).bgzf, buffer.cast(), remaining as usize);
        if nread < remaining {
            free(buffer.cast());
            *len = -1;
            return std::ptr::null_mut();
        }
        *buffer.add(nread as usize) = 0;
        return buffer;
    }

    let mut s = buffer;
    let firstline_len = (*val).line_len as isize - (beg % (*val).line_blen as hts_pos_t) as isize;
    let mut nread = bgzf_read((*fai).bgzf, s.cast(), firstline_len as usize);
    if nread < firstline_len {
        free(buffer.cast());
        *len = -1;
        return std::ptr::null_mut();
    }
    s = s.add(firstline_blen as usize);
    remaining -= firstline_blen;

    while remaining > (*val).line_blen as isize {
        nread = bgzf_read((*fai).bgzf, s.cast(), (*val).line_len as usize);
        if nread < (*val).line_len as isize {
            free(buffer.cast());
            *len = -1;
            return std::ptr::null_mut();
        }
        s = s.add((*val).line_blen as usize);
        remaining -= (*val).line_blen as isize;
    }

    if remaining > 0 {
        nread = bgzf_read((*fai).bgzf, s.cast(), remaining as usize);
        if nread < remaining {
            free(buffer.cast());
            *len = -1;
            return std::ptr::null_mut();
        }
        s = s.add(remaining as usize);
    }

    *s = 0;
    buffer
}

unsafe fn kh_destroy_s(h: *mut faidx_hash_t) {
    if h.is_null() {
        return;
    }
    free((*h).flags.cast());
    free((*h).keys.cast());
    free((*h).vals.cast());
    free(h.cast());
}

unsafe fn kh_str_hash_string(s: *const c_char) -> u32 {
    let mut h = *s as u8 as u32;
    if h != 0 {
        let mut p = s.add(1);
        while *p != 0 {
            h = (h << 5).wrapping_sub(h).wrapping_add(*p as u8 as u32);
            p = p.add(1);
        }
    }
    h
}

unsafe fn cstr_eq(a: *const c_char, b: *const c_char) -> bool {
    !a.is_null() && !b.is_null() && CStr::from_ptr(a) == CStr::from_ptr(b)
}

unsafe fn kh_isempty(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 2) != 0
}

unsafe fn kh_isdel(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 1) != 0
}

unsafe fn kh_iseither(flags: *const u32, i: u32) -> bool {
    ((*flags.add((i >> 4) as usize) >> ((i & 0x0f) << 1)) & 3) != 0
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{c_void, CString},
        fs,
        mem::{align_of, size_of},
        ptr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn public_faidx_struct_layout_matches_htslib_abi_shape() {
        assert_eq!(size_of::<faidx1_t>(), 40);
        assert_eq!(align_of::<faidx1_t>(), 8);
        assert_eq!(size_of::<faidx_hash_t>(), 40);
        assert_eq!(align_of::<faidx_hash_t>(), 8);
        assert_eq!(size_of::<faidx_t>(), 40);
        assert_eq!(align_of::<faidx_t>(), 8);
        assert_eq!(std::mem::offset_of!(faidx_t, bgzf), 0);
        assert_eq!(std::mem::offset_of!(faidx_t, n), 8);
        assert_eq!(std::mem::offset_of!(faidx_t, m), 12);
        assert_eq!(std::mem::offset_of!(faidx_t, name), 16);
        assert_eq!(std::mem::offset_of!(faidx_t, hash), 24);
        assert_eq!(std::mem::offset_of!(faidx_t, format), 32);
    }

    #[test]
    fn faidx_has_seq_matches_khash_string_lookup_rules() {
        let present = CString::new("chr1").unwrap();
        let absent = CString::new("chr2").unwrap();
        let mut names = vec![present.as_ptr() as *mut c_char];
        let mut keys = vec![present.as_ptr() as *mut c_char];
        let mut vals = vec![faidx1_t {
            id: 0,
            line_len: 80,
            line_blen: 81,
            len: 100,
            seq_offset: 10,
            qual_offset: 0,
        }];
        let mut flags = vec![0u32];
        let mut hash = faidx_hash_t {
            n_buckets: 1,
            size: 1,
            n_occupied: 1,
            upper_bound: 1,
            flags: flags.as_mut_ptr(),
            keys: keys.as_mut_ptr(),
            vals: vals.as_mut_ptr(),
        };
        let fai = faidx_t {
            bgzf: std::ptr::null_mut(),
            n: 1,
            m: 1,
            name: names.as_mut_ptr(),
            hash: &mut hash,
            format: 0,
        };

        unsafe {
            assert_eq!(faidx_has_seq(&fai, present.as_ptr()), 1);
            assert_eq!(faidx_has_seq(&fai, absent.as_ptr()), 0);
            assert_eq!(faidx_has_seq(std::ptr::null(), present.as_ptr()), 0);
            assert_eq!(faidx_fetch_nseq(&fai), 1);
            assert_eq!(faidx_nseq(&fai), 1);
            assert_eq!(CStr::from_ptr(faidx_iseq(&fai, 0)), present.as_c_str());
            assert_eq!(faidx_seq_len64(&fai, present.as_ptr()), 100);
            assert_eq!(faidx_seq_len64(&fai, absent.as_ptr()), -1);
            assert_eq!(faidx_seq_len(&fai, present.as_ptr()), 100);
            assert_eq!(faidx_seq_len(&fai, absent.as_ptr()), -1);

            let mut beg = -5;
            let mut end = 150;
            assert_eq!(fai_adjust_region(&fai, 0, &mut beg, &mut end), 3);
            assert_eq!(beg, 0);
            assert_eq!(end, 100);

            let mut beg = 90;
            let mut end = 10;
            assert_eq!(fai_adjust_region(&fai, 0, &mut beg, &mut end), 1);
            assert_eq!(beg, 10);
            assert_eq!(end, 10);

            assert_eq!(fai_adjust_region(&fai, -1, &mut beg, &mut end), -1);
            assert_eq!(fai_adjust_region(&fai, 1, &mut beg, &mut end), -1);
        }
    }

    #[test]
    fn fai_destroy_frees_c_allocated_index_shape() {
        unsafe {
            let fai = crate::htslib_mini_rs::c_compat::malloc(size_of::<faidx_t>() as u64)
                .cast::<faidx_t>();
            assert!(!fai.is_null());
            let name_array =
                crate::htslib_mini_rs::c_compat::malloc(size_of::<*mut c_char>() as u64)
                    .cast::<*mut c_char>();
            assert!(!name_array.is_null());
            let chr_name = crate::htslib_mini_rs::c_compat::malloc(5).cast::<c_char>();
            assert!(!chr_name.is_null());
            ptr::copy_nonoverlapping(b"chr1\0".as_ptr().cast::<c_char>(), chr_name, 5);
            *name_array = chr_name;

            let hash = crate::htslib_mini_rs::c_compat::malloc(size_of::<faidx_hash_t>() as u64)
                .cast::<faidx_hash_t>();
            assert!(!hash.is_null());
            ptr::write(
                hash,
                faidx_hash_t {
                    n_buckets: 0,
                    size: 0,
                    n_occupied: 0,
                    upper_bound: 0,
                    flags: ptr::null_mut(),
                    keys: ptr::null_mut(),
                    vals: ptr::null_mut(),
                },
            );

            ptr::write(
                fai,
                faidx_t {
                    bgzf: ptr::null_mut(),
                    n: 1,
                    m: 1,
                    name: name_array,
                    hash,
                    format: 0,
                },
            );

            fai_destroy(fai);
            fai_destroy(ptr::null_mut());
            free(ptr::null_mut::<c_void>());
        }
    }

    #[test]
    fn faidx_fetch_seq64_reads_reference_span_across_wrapped_lines() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-faidx-fetch-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\nTGCA\n>chr2\nNNNN\n").unwrap();

        unsafe {
            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let chr1 = CString::new("chr1").unwrap();
            let fai = fai_load(path_c.as_ptr());
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_has_seq(fai, chr1.as_ptr()), 1);
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr()), 8);

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, chr1.as_ptr(), 2, 6, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 5);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"GTTGC");
            free(seq.cast());

            let seq = faidx_fetch_seq64(fai, chr1.as_ptr(), 1, 2, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 2);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"CG");
            free(seq.cast());

            let mut len32 = 0;
            let seq = faidx_fetch_seq(fai, chr1.as_ptr(), 2, 6, &mut len32);
            assert!(!seq.is_null());
            assert_eq!(len32, 5);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"GTTGC");
            free(seq.cast());

            let reg = CString::new("chr1:3-7").unwrap();
            assert_eq!(fai_line_length(fai, reg.as_ptr()), 4);
            let seq = fai_fetch64(fai, reg.as_ptr(), &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 5);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"GTTGC");
            free(seq.cast());

            let seq = fai_fetch(fai, reg.as_ptr(), &mut len32);
            assert!(!seq.is_null());
            assert_eq!(len32, 5);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"GTTGC");
            free(seq.cast());

            let absent = CString::new("absent").unwrap();
            let seq = faidx_fetch_seq64(fai, absent.as_ptr(), 0, 1, &mut len);
            assert!(seq.is_null());
            assert_eq!(len, -2);

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_parse_region_handles_names_ranges_and_braces_without_htslib() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-parse-region-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\n>chr1:alt\nACGT\n").unwrap();
        fs::write(&fai_path, b"chr1\t4\t6\t4\t5\nchr1:alt\t4\t22\t4\t5\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let fai = fai_load3(path_c.as_ptr(), std::ptr::null(), std::ptr::null(), 0);
            assert!(!fai.is_null());

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;

            let reg = CString::new("chr1:2-4").unwrap();
            assert!(
                !fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
            assert_eq!((tid, beg, end), (0, 1, 4));

            let reg = CString::new("{chr1:alt}:1-2").unwrap();
            assert!(
                !fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
            assert_eq!((tid, beg, end), (1, 0, 2));

            let reg = CString::new("chr1:3").unwrap();
            assert!(
                !fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
            assert_eq!((tid, beg, end), (0, 2, HTS_POS_MAX));

            let reg = CString::new("chr1:3").unwrap();
            assert!(!fai_parse_region(
                fai,
                reg.as_ptr(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_ONE_COORD,
            )
            .is_null());
            assert_eq!((tid, beg, end), (0, 2, 3));

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_load3_reads_existing_fai_without_htslib_index_build() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-existing-fai-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\nTGCA\n>chr2\nNNNN\n").unwrap();
        fs::write(&fai_path, b"chr1\t8\t6\t4\t5\nchr2\t4\t22\t4\t5\n").unwrap();

        unsafe {
            let path_c = CString::new(path.to_string_lossy().as_bytes()).unwrap();
            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            let fai = fai_load3(path_c.as_ptr(), std::ptr::null(), std::ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)).to_bytes(), b"chr1");
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)).to_bytes(), b"chr2");
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr()), 8);
            assert_eq!(faidx_seq_len64(fai, chr2.as_ptr()), 4);

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, chr1.as_ptr(), 4, 7, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 4);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"TGCA");
            free(seq.cast());

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_load3_builds_plain_fasta_index_with_raw_header_bytes() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib-mini-rs-build-fai-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">sq\xff comment\nACG\nTT\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let seq_name = CString::new(b"sq\xff".as_slice()).unwrap();
            let fai = fai_load3(
                path_c.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                FAI_CREATE,
            );
            assert!(!fai.is_null());
            assert!(fai_path.exists());
            assert_eq!(faidx_nseq(fai), 1);
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)).to_bytes(), b"sq\xff");
            assert_eq!(faidx_seq_len64(fai, seq_name.as_ptr()), 5);

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, seq_name.as_ptr(), 0, 4, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 5);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"ACGTT");
            free(seq.cast());
            fai_destroy(fai);
        }

        let index = fs::read(&fai_path).unwrap();
        assert!(index.starts_with(b"sq\xff\t5\t13\t3\t4\n"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn faidx_inline_ascii_and_bgzf_getc_wrappers_match_c_rules() {
        assert_eq!(isgraph_(b'!'), 1);
        assert_eq!(isgraph_(b'~'), 1);
        assert_eq!(isgraph_(b' '), 0);
        assert_eq!(isgraph_(0x7f), 0);

        let mut data = *b"xyz";
        let mut fp = BGZF {
            bitfields: 0,
            cache_size: 0,
            block_length: 3,
            block_clength: 0,
            block_offset: 0,
            block_address: 0,
            uncompressed_address: 0,
            uncompressed_block: data.as_mut_ptr().cast(),
            compressed_block: std::ptr::null_mut(),
            cache: std::ptr::null_mut(),
            fp: std::ptr::null_mut(),
            mt: std::ptr::null_mut(),
            idx: std::ptr::null_mut(),
            idx_build_otf: 0,
            gz_stream: std::ptr::null_mut(),
            seeked: 0,
        };
        unsafe {
            assert_eq!(bgzf_getc_(&mut fp), b'x' as c_int);
            assert_eq!(fp.block_offset, 1);
            assert_eq!(fp.uncompressed_address, 1);

            let explicit = fai_path(c"ref.fa##idx##custom.fai".as_ptr());
            assert!(!explicit.is_null());
            assert_eq!(CStr::from_ptr(explicit).to_bytes(), b"custom.fai");
            free(explicit.cast());
            assert!(fai_path(c"ref.fa".as_ptr()).is_null());
        }
    }

    #[test]
    fn fai_set_cache_size_forwards_to_bgzf_when_cache_exists() {
        let mut cache_marker = 0u8;
        let mut bgzf = BGZF {
            bitfields: 0,
            cache_size: 0,
            block_length: 0,
            block_clength: 0,
            block_offset: 0,
            block_address: 0,
            uncompressed_address: 0,
            uncompressed_block: std::ptr::null_mut(),
            compressed_block: std::ptr::null_mut(),
            cache: (&mut cache_marker as *mut u8).cast(),
            fp: std::ptr::null_mut(),
            mt: std::ptr::null_mut(),
            idx: std::ptr::null_mut(),
            idx_build_otf: 0,
            gz_stream: std::ptr::null_mut(),
            seeked: 0,
        };
        let mut fai = faidx_t {
            bgzf: &mut bgzf,
            n: 0,
            m: 0,
            name: std::ptr::null_mut(),
            hash: std::ptr::null_mut(),
            format: 0,
        };
        unsafe {
            fai_set_cache_size(&mut fai, 4096);
        }
        assert_eq!(bgzf.cache_size, 4096);
    }
}
