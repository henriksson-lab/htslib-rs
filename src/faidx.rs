use std::{
    collections::HashSet,
    ffi::{c_char, c_int, c_void, CStr},
    fs,
    io::Write,
    ptr,
};

use super::bgzf::{
    bgzf_close, bgzf_compression, bgzf_getc, bgzf_index_build_init, bgzf_index_dump,
    bgzf_index_load, bgzf_open, bgzf_read, bgzf_set_cache_size, bgzf_thread_pool, bgzf_useek,
    bgzf_utell,
};
use super::hfile::{hclose, hclose_abruptly, hisremote, hopen, hputs2, htslib_hfile_h_195_hgetln};
use super::hts::hFILE;
use super::hts::{
    hts_c_4756_hts_idx_check_local, hts_c_4920_hts_idx_locatefn, hts_parse_region, hts_pos_t, BGZF,
    HTS_FMT_FAI, HTS_POS_MAX,
};
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
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
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
    if !fai_tmp.is_null() {
        return libc::strdup(fai_tmp.add(delim.to_bytes().len()));
    }

    if hisremote(fa) != 0 {
        return hts_c_4920_hts_idx_locatefn(fa, c".fai".as_ptr());
    }

    let mut fai = std::ptr::null_mut();
    if hts_c_4756_hts_idx_check_local(fa, HTS_FMT_FAI, &mut fai) == 0 && !fai.is_null() {
        if fai_build3(fa, fai, std::ptr::null()) == -1 {
            free(fai.cast());
            return std::ptr::null_mut();
        }
    }
    fai
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
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
) -> *mut faidx_t {
    fai_load3_core(fn_, fnfai, fngzi, flags, FAI_FASTA as c_int)
}

pub unsafe fn fai_load(_fn_: *const c_char) -> *mut faidx_t {
    fai_load3(_fn_, std::ptr::null(), std::ptr::null(), FAI_CREATE)
}

pub unsafe fn fai_build3(fn_: *const c_char, fnfai: *const c_char, fngzi: *const c_char) -> c_int {
    faidx_c_557_fai_build3(fn_, fnfai, fngzi)
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
    fai_load3_core(fn_, fnfai, fngzi, flags, format as c_int)
}

pub unsafe fn fai_load_format(fn_: *const c_char, format: fai_format_options) -> *mut faidx_t {
    fai_load3_core(fn_, ptr::null(), ptr::null(), FAI_CREATE, format as c_int)
}

unsafe fn fai_load3_core(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
    format: c_int,
) -> *mut faidx_t {
    if fn_.is_null()
        || (format != FAI_NONE as c_int
            && format != FAI_FASTA as c_int
            && format != FAI_FASTQ as c_int)
    {
        return ptr::null_mut();
    }
    let fai_path = resolved_index_path(fn_, fnfai, b".fai");
    if fai_path.is_none() {
        return ptr::null_mut();
    }
    let fngzi_path = resolved_index_path(fn_, fngzi, b".gzi");
    if fngzi_path.is_none() {
        return ptr::null_mut();
    }
    let mut build_index = !fai_path.as_ref().unwrap().exists();
    if !build_index {
        let bgzf = bgzf_open(fn_, b"r\0".as_ptr().cast());
        if bgzf.is_null() {
            return ptr::null_mut();
        }
        if bgzf_compression(bgzf) == 2 && !fngzi_path.as_ref().unwrap().exists() {
            build_index = true;
        }
        bgzf_close(bgzf);
    }
    if build_index {
        if (flags & FAI_CREATE) == 0 || fai_build3_core(fn_, fnfai, fngzi) != 0 {
            return ptr::null_mut();
        }
    }
    let Some(fai) = fai_read(fn_, fnfai, format) else {
        return ptr::null_mut();
    };
    if bgzf_compression((*fai).bgzf) == 2 {
        let mut gzi_bytes = path_bytes(fngzi_path.as_ref().unwrap()).into_owned();
        gzi_bytes.push(0);
        if bgzf_index_load((*fai).bgzf, gzi_bytes.as_ptr().cast(), ptr::null()) < 0 {
            fai_destroy(fai);
            return ptr::null_mut();
        }
    }
    fai
}

unsafe fn fai_build3_core(fn_: *const c_char, fnfai: *const c_char, fngzi: *const c_char) -> c_int {
    if fn_.is_null() {
        return -1;
    }
    let bgzf = bgzf_open(fn_, b"r\0".as_ptr().cast());
    if bgzf.is_null() {
        return -1;
    }
    if bgzf_compression(bgzf) == 2 && bgzf_index_build_init(bgzf) != 0 {
        bgzf_close(bgzf);
        return -1;
    }
    let fai = fai_build_core(bgzf);
    if fai.is_null() {
        bgzf_close(bgzf);
        return -1;
    }
    let fai_path = resolved_index_path(fn_, fnfai, b".fai");
    let gzi_path = resolved_index_path(fn_, fngzi, b".gzi");
    if fai_path.is_none() || gzi_path.is_none() {
        bgzf_close(bgzf);
        fai_destroy(fai);
        return -1;
    }
    if bgzf_compression(bgzf) == 2 {
        let mut gzi_bytes = path_bytes(gzi_path.as_ref().unwrap()).into_owned();
        gzi_bytes.push(0);
        if bgzf_index_dump(bgzf, gzi_bytes.as_ptr().cast(), ptr::null()) < 0 {
            bgzf_close(bgzf);
            fai_destroy(fai);
            return -1;
        }
    }
    if bgzf_close(bgzf) < 0 {
        fai_destroy(fai);
        return -1;
    }
    let ret = fai_save(fai, fai_path.as_ref().unwrap());
    fai_destroy(fai);
    ret
}

fn resolved_index_path(
    fn_: *const c_char,
    explicit: *const c_char,
    suffix: &[u8],
) -> Option<std::path::PathBuf> {
    unsafe {
        if fn_.is_null() {
            return None;
        }
        if explicit.is_null() {
            let fasta_path = path_from_bytes(CStr::from_ptr(fn_).to_bytes());
            let mut bytes = path_bytes(&fasta_path).into_owned();
            bytes.extend_from_slice(suffix);
            Some(path_from_bytes(&bytes))
        } else {
            Some(path_from_bytes(CStr::from_ptr(explicit).to_bytes()))
        }
    }
}

unsafe fn fai_read(
    fn_: *const c_char,
    fnfai: *const c_char,
    format: c_int,
) -> Option<*mut faidx_t> {
    let fai_name = owned_index_cstring(fn_, fnfai, b".fai")?;
    let fp = hopen(fai_name.as_ptr(), c"rb".as_ptr());
    if fp.is_null() {
        return None;
    }

    let fai = faidx_c_380_fai_read(fp, fai_name.as_ptr(), format);
    if fai.is_null() {
        hclose_abruptly(fp);
        return None;
    }
    if hclose(fp) < 0 {
        fai_destroy(fai);
        return None;
    }

    (*fai).bgzf = bgzf_open(fn_, c"rb".as_ptr());
    if (*fai).bgzf.is_null() {
        fai_destroy(fai);
        return None;
    }
    Some(fai)
}

fn is_fai_index_space(b: u8) -> bool {
    matches!(b, b'\t' | b'\n' | 0x0b | 0x0c | b'\r' | b' ')
}

unsafe fn fai_save(fai: *const faidx_t, path: &std::path::Path) -> c_int {
    let mut out = Vec::new();
    for i in 0..(*fai).n {
        let name = *(*fai).name.add(i as usize);
        let k = kh_get_s((*fai).hash, name);
        if k == (*(*fai).hash).n_buckets {
            return -1;
        }
        let val = *(*(*fai).hash).vals.add(k as usize);
        out.extend_from_slice(CStr::from_ptr(name).to_bytes());
        if (*fai).format == FAI_FASTQ as c_int {
            out.extend_from_slice(
                format!(
                    "\t{}\t{}\t{}\t{}\t{}\n",
                    val.len, val.seq_offset, val.line_blen, val.line_len, val.qual_offset
                )
                .as_bytes(),
            );
        } else {
            out.extend_from_slice(
                format!(
                    "\t{}\t{}\t{}\t{}\n",
                    val.len, val.seq_offset, val.line_blen, val.line_len
                )
                .as_bytes(),
            );
        }
    }
    fs::write(path, out).map(|_| 0).unwrap_or(-1)
}

unsafe fn fai_build_core(bgzf: *mut BGZF) -> *mut faidx_t {
    let pathless = ptr::null();
    let _ = bgzf_utell(bgzf);
    let data = read_all_bgzf(bgzf);
    let Some(data) = data else {
        return ptr::null_mut();
    };
    let Some((rows, format)) = parse_fasta_fastq_index_rows(&data) else {
        return ptr::null_mut();
    };
    match fai_from_rows(pathless, rows, format) {
        Some(fai) => {
            (*fai).bgzf = ptr::null_mut();
            fai
        }
        None => ptr::null_mut(),
    }
}

unsafe fn read_all_bgzf(bgzf: *mut BGZF) -> Option<Vec<u8>> {
    let mut data = Vec::new();
    loop {
        let c = bgzf_getc(bgzf);
        if c == -1 {
            break;
        }
        if c < 0 {
            return None;
        }
        data.push(c as u8);
    }
    Some(data)
}

unsafe fn fai_insert_index(
    rows: &mut Vec<(Vec<u8>, faidx1_t)>,
    name: Vec<u8>,
    len: u64,
    line_len: u32,
    line_blen: u32,
    seq_offset: u64,
    qual_offset: u64,
) -> c_int {
    if rows.iter().any(|(n, _)| *n == name) {
        return 0;
    }
    rows.push((
        name,
        faidx1_t {
            id: rows.len() as c_int,
            line_len,
            line_blen,
            len,
            seq_offset,
            qual_offset,
        },
    ));
    0
}

fn parse_fasta_fastq_index_rows(data: &[u8]) -> Option<(Vec<(Vec<u8>, faidx1_t)>, c_int)> {
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut i = 0usize;
    let mut format = FAI_NONE as c_int;
    while i < data.len() {
        while i < data.len() && (data[i] == b'\n' || data[i] == b'\r') {
            i += 1;
        }
        if i >= data.len() {
            break;
        }
        let marker = data[i];
        if marker != b'>' && marker != b'@' {
            return None;
        }
        let this_format = if marker == b'>' {
            FAI_FASTA as c_int
        } else {
            FAI_FASTQ as c_int
        };
        if format != FAI_NONE as c_int && format != this_format {
            return None;
        }
        format = this_format;
        i += 1;
        while i < data.len() && data[i].is_ascii_whitespace() && data[i] != b'\n' {
            i += 1;
        }
        let name_start = i;
        while i < data.len() && !data[i].is_ascii_whitespace() {
            i += 1;
        }
        let name = data[name_start..i].to_vec();
        while i < data.len() && data[i] != b'\n' {
            i += 1;
        }
        if i < data.len() {
            i += 1;
        }
        let seq_offset = i as u64;
        let mut seq_len = 0u64;
        let mut line_len = 0u32;
        let mut line_blen = 0u32;
        let mut final_short_line = false;

        while i < data.len() {
            if format == FAI_FASTA as c_int && data[i] == b'>' {
                break;
            }
            if format == FAI_FASTA as c_int && (data[i] == b'\n' || data[i] == b'\r') && seq_len > 0
            {
                break;
            }
            if format == FAI_FASTQ as c_int && data[i] == b'+' {
                while i < data.len() && data[i] != b'\n' {
                    i += 1;
                }
                if i < data.len() {
                    i += 1;
                }
                break;
            }
            if data[i] == b'\n' || data[i] == b'\r' {
                return None;
            }
            if final_short_line {
                return None;
            }
            let mut ll = 0u32;
            let mut bl = 0u32;
            while i < data.len() && data[i] != b'\n' {
                ll += 1;
                if isgraph_(data[i]) != 0 {
                    bl += 1;
                }
                i += 1;
            }
            ll += 1;
            if i < data.len() {
                i += 1;
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
        }

        if seq_len == 0 || line_len == 0 || line_blen == 0 {
            return None;
        }

        let mut qual_offset = 0;
        if format == FAI_FASTQ as c_int {
            qual_offset = i as u64;
            let mut qual_len = 0u64;
            while i < data.len() {
                if data[i] == b'@' && qual_len == seq_len {
                    break;
                }
                if data[i] == b'\n' || data[i] == b'\r' {
                    return None;
                }
                let mut ll = 0u32;
                let mut bl = 0u32;
                while i < data.len() && data[i] != b'\n' {
                    ll += 1;
                    if isgraph_(data[i]) != 0 {
                        bl += 1;
                    }
                    i += 1;
                }
                ll += 1;
                if i < data.len() {
                    i += 1;
                }
                if ll > line_len {
                    return None;
                }
                qual_len += bl as u64;
                if qual_len > seq_len || (qual_len < seq_len && ll < line_len) {
                    return None;
                }
                if qual_len == seq_len {
                    break;
                }
            }
            if qual_len != seq_len {
                return None;
            }
        }

        if seen.insert(name.clone()) {
            rows.push((
                name,
                faidx1_t {
                    id: rows.len() as c_int,
                    line_len,
                    line_blen,
                    len: seq_len,
                    seq_offset,
                    qual_offset,
                },
            ));
        }
    }
    if rows.is_empty() || format == FAI_NONE as c_int {
        None
    } else {
        Some((rows, format))
    }
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

    fai_from_rows(fn_, rows, FAI_FASTA as c_int)
}

unsafe fn fai_from_rows(
    fn_: *const c_char,
    rows: Vec<(Vec<u8>, faidx1_t)>,
    format: c_int,
) -> Option<*mut faidx_t> {
    let fai = malloc(std::mem::size_of::<faidx_t>()).cast::<faidx_t>();
    if fai.is_null() {
        return None;
    }
    std::ptr::write_bytes(fai, 0, 1);
    (*fai).n = 0;
    (*fai).m = rows.len() as c_int;
    (*fai).format = format;
    if !fn_.is_null() {
        (*fai).bgzf = bgzf_open(fn_, b"r\0".as_ptr().cast());
        if (*fai).bgzf.is_null() {
            free(fai.cast());
            return None;
        }
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
    (*hash).size = 0;
    (*hash).n_occupied = 0;
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

    for (name, mut val) in rows.into_iter() {
        let name_ptr = malloc(name.len() + 1).cast::<c_char>();
        if name_ptr.is_null() {
            fai_destroy(fai);
            return None;
        }
        std::ptr::copy_nonoverlapping(name.as_ptr().cast::<c_char>(), name_ptr, name.len());
        *name_ptr.add(name.len()) = 0;
        if kh_get_s(hash, name_ptr) != (*hash).n_buckets {
            free(name_ptr.cast());
            continue;
        }
        *(*fai).name.add((*fai).n as usize) = name_ptr;
        val.id = (*fai).n;
        (*fai).n += 1;
        (*hash).size += 1;
        (*hash).n_occupied += 1;

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

    fai_from_rows(fn_, rows, FAI_FASTA as c_int)
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
    hts_parse_region(
        s,
        tid,
        beg,
        end,
        Some(fai_name2id),
        fai.cast_mut().cast(),
        flags,
    )
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
            let fai =
                crate::htslib_rs::c_compat::malloc(size_of::<faidx_t>() as u64).cast::<faidx_t>();
            assert!(!fai.is_null());
            let name_array = crate::htslib_rs::c_compat::malloc(size_of::<*mut c_char>() as u64)
                .cast::<*mut c_char>();
            assert!(!name_array.is_null());
            let chr_name = crate::htslib_rs::c_compat::malloc(5).cast::<c_char>();
            assert!(!chr_name.is_null());
            ptr::copy_nonoverlapping(b"chr1\0".as_ptr().cast::<c_char>(), chr_name, 5);
            *name_array = chr_name;

            let hash = crate::htslib_rs::c_compat::malloc(size_of::<faidx_hash_t>() as u64)
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
            "htslib_rs-faidx-fetch-{}-{}.fa",
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
    fn faidx_fetch_seq64_clamps_negative_reversed_and_past_end_coordinates() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-faidx-fetch-clamp-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\nTGCA\n").unwrap();
        fs::write(&fai_path, b"chr1\t8\t6\t4\t5\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let chr1 = CString::new("chr1").unwrap();
            let fai = fai_load3(path_c.as_ptr(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, chr1.as_ptr(), -5, 2, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 3);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"ACG");
            free(seq.cast());

            let seq = faidx_fetch_seq64(fai, chr1.as_ptr(), 6, 2, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 1);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"G");
            free(seq.cast());

            let seq = faidx_fetch_seq64(fai, chr1.as_ptr(), 99, 120, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 0);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"");
            free(seq.cast());

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
            "htslib_rs-parse-region-{}-{}.fa",
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

            let reg = CString::new("chr1:alt").unwrap();
            assert!(fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null());
            assert_eq!(tid, -1);

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_parse_region_braced_ambiguous_name_alone_means_whole_contig() {
        unsafe {
            let fai = fai_from_rows(
                ptr::null(),
                vec![
                    (
                        b"chr1".to_vec(),
                        faidx1_t {
                            id: 0,
                            line_len: 80,
                            line_blen: 80,
                            len: 100,
                            seq_offset: 0,
                            qual_offset: 0,
                        },
                    ),
                    (
                        b"chr1:alt".to_vec(),
                        faidx1_t {
                            id: 1,
                            line_len: 80,
                            line_blen: 80,
                            len: 50,
                            seq_offset: 0,
                            qual_offset: 0,
                        },
                    ),
                ],
                FAI_FASTA as c_int,
            )
            .unwrap();

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let reg = CString::new("{chr1:alt}").unwrap();
            let rest = fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0);
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest).to_bytes(), b"");
            assert_eq!((tid, beg, end), (1, 0, HTS_POS_MAX));

            let reg = CString::new("{chr1:alt}:2-1").unwrap();
            assert!(fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0).is_null());

            fai_destroy(fai);
        }
    }

    #[test]
    fn fai_line_length_uses_parsed_region_name_and_reports_missing_names() {
        unsafe {
            let fai = fai_from_rows(
                ptr::null(),
                vec![
                    (
                        b"chr1".to_vec(),
                        faidx1_t {
                            id: 0,
                            line_len: 81,
                            line_blen: 80,
                            len: 100,
                            seq_offset: 0,
                            qual_offset: 0,
                        },
                    ),
                    (
                        b"chr1:alt".to_vec(),
                        faidx1_t {
                            id: 1,
                            line_len: 51,
                            line_blen: 50,
                            len: 100,
                            seq_offset: 0,
                            qual_offset: 0,
                        },
                    ),
                ],
                FAI_FASTA as c_int,
            )
            .unwrap();

            assert_eq!(fai_line_length(fai, c"chr1:2-3".as_ptr()), 80);
            assert_eq!(fai_line_length(fai, c"{chr1:alt}:2-3".as_ptr()), 50);
            assert_eq!(fai_line_length(fai, c"missing:1-2".as_ptr()), -1);

            fai_destroy(fai);
        }
    }

    #[test]
    fn fai_parse_region_list_mode_matches_htslib_comma_boundaries() {
        unsafe {
            let fai = fai_from_rows(
                ptr::null(),
                vec![
                    (
                        b"chr1".to_vec(),
                        faidx1_t {
                            id: 0,
                            line_len: 80,
                            line_blen: 80,
                            len: 2000,
                            seq_offset: 0,
                            qual_offset: 0,
                        },
                    ),
                    (
                        b"chr3".to_vec(),
                        faidx1_t {
                            id: 1,
                            line_len: 80,
                            line_blen: 80,
                            len: 2000,
                            seq_offset: 0,
                            qual_offset: 0,
                        },
                    ),
                ],
                FAI_FASTA as c_int,
            )
            .unwrap();

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let reg = CString::new("chr1,chr3").unwrap();
            let rest = fai_parse_region(
                fai,
                reg.as_ptr(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST,
            );
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest).to_bytes(), b"chr3");
            assert_eq!((tid, beg, end), (0, 0, HTS_POS_MAX));

            let reg = CString::new("chr3:1,000-1,500").unwrap();
            let rest = fai_parse_region(
                fai,
                reg.as_ptr(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST | HTS_PARSE_ONE_COORD,
            );
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest).to_bytes(), b"000-1,500");
            assert_eq!((tid, beg, end), (1, 0, 1));

            fai_destroy(fai);
        }
    }

    #[test]
    fn fai_parse_region_allows_thousands_separators_only_outside_list_mode() {
        unsafe {
            let fai = fai_from_rows(
                ptr::null(),
                vec![(
                    b"chr1".to_vec(),
                    faidx1_t {
                        id: 0,
                        line_len: 80,
                        line_blen: 80,
                        len: 5000,
                        seq_offset: 0,
                        qual_offset: 0,
                    },
                )],
                FAI_FASTA as c_int,
            )
            .unwrap();

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let reg = CString::new("chr1:1,000-1,002").unwrap();
            let rest = fai_parse_region(fai, reg.as_ptr(), &mut tid, &mut beg, &mut end, 0);
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest).to_bytes(), b"");
            assert_eq!((tid, beg, end), (0, 999, 1002));

            let rest = fai_parse_region(
                fai,
                reg.as_ptr(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST,
            );
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest).to_bytes(), b"000-1,002");
            assert_eq!((tid, beg, end), (0, 0, HTS_POS_MAX));

            fai_destroy(fai);
        }
    }

    #[test]
    fn resolved_index_path_honors_explicit_fai_and_gzi_paths() {
        let fasta = CString::new("/tmp/ref.fa").unwrap();
        let explicit_fai = CString::new("/tmp/custom.index").unwrap();
        let explicit_gzi = CString::new("/tmp/custom.gzi").unwrap();

        let inferred = resolved_index_path(fasta.as_ptr(), ptr::null(), b".fai").unwrap();
        assert_eq!(path_bytes(&inferred).as_ref(), b"/tmp/ref.fa.fai");

        let explicit = resolved_index_path(fasta.as_ptr(), explicit_fai.as_ptr(), b".fai").unwrap();
        assert_eq!(path_bytes(&explicit).as_ref(), b"/tmp/custom.index");

        let explicit = resolved_index_path(fasta.as_ptr(), explicit_gzi.as_ptr(), b".gzi").unwrap();
        assert_eq!(path_bytes(&explicit).as_ref(), b"/tmp/custom.gzi");

        assert!(resolved_index_path(ptr::null(), explicit_fai.as_ptr(), b".fai").is_none());
    }

    #[test]
    fn fai_read_accepts_htslib_whitespace_separated_index_fields() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fai-whitespace-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\n>chr2\nNNNN\n").unwrap();
        fs::write(&fai_path, b"chr1 4 6 4 5\r\nchr2\t4 17\t4 5\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            let fai = fai_load3(path_c.as_ptr(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr()), 4);
            assert_eq!(faidx_seq_len64(fai, chr2.as_ptr()), 4);
            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_read_accepts_all_c_whitespace_between_index_fields() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fai-cspace-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\n>chr2\nNNNN\n").unwrap();
        fs::write(&fai_path, b"chr1\t4\x0b6\x0c4 5\nchr2\t4\t17\t4\t5\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let chr1 = CString::new("chr1").unwrap();
            let chr2 = CString::new("chr2").unwrap();
            let fai = fai_load3(path_c.as_ptr(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr()), 4);
            assert_eq!(faidx_seq_len64(fai, chr2.as_ptr()), 4);
            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_read_existing_index_ignores_duplicate_sequence_rows() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fai-duplicate-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">dup\nAAAA\n>other\nCC\n").unwrap();
        fs::write(
            &fai_path,
            b"dup\t4\t5\t4\t5\ndup\t4\t16\t4\t5\nother\t2\t21\t2\t3\n",
        )
        .unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let dup = CString::new("dup").unwrap();
            let other = CString::new("other").unwrap();
            let fai = fai_load3(path_c.as_ptr(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)).to_bytes(), b"dup");
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)).to_bytes(), b"other");
            assert_eq!(faidx_seq_len64(fai, dup.as_ptr()), 4);
            assert_eq!(faidx_seq_len64(fai, other.as_ptr()), 2);
            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_read_matches_c_field_parsing_without_geometry_validation() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fai-invalid-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let chr1 = CString::new("chr1").unwrap();
            for row in [
                b"chr1\t4\t6\t0\t5\n".as_slice(),
                b"chr1\t4\t6\t5\t4\n".as_slice(),
            ] {
                fs::write(&fai_path, row).unwrap();
                let fai = fai_load3(path_c.as_ptr(), ptr::null(), ptr::null(), 0);
                assert!(!fai.is_null());
                assert_eq!(faidx_seq_len64(fai, chr1.as_ptr()), 4);
                fai_destroy(fai);
            }

            fs::write(&fai_path, b"chr1\t4\tbad\t4\t5\n").unwrap();
            let fai = fai_load3(path_c.as_ptr(), ptr::null(), ptr::null(), 0);
            assert!(fai.is_null());
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_read_fastq_index_keeps_first_duplicate_and_requires_quality_offset() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fai-fastq-index-{}-{}.fq",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fq.fai");
        fs::write(&path, b"@r1\nACGT\n+\n!!!!\n@r2\nNN\n+\n##\n").unwrap();
        fs::write(
            &fai_path,
            b"r1\t4\t4\t4\t5\t11\nr1\t4\t21\t4\t5\t28\nr2\t2\t21\t2\t3\t26\n",
        )
        .unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let r1 = CString::new("r1").unwrap();
            let r2 = CString::new("r2").unwrap();
            let fai = fai_load3_format(path_c.as_ptr(), ptr::null(), ptr::null(), 0, FAI_FASTQ);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)).to_bytes(), b"r1");
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)).to_bytes(), b"r2");

            let mut len = 0;
            let qual = faidx_fetch_qual64(fai, r1.as_ptr(), 0, 3, &mut len);
            assert!(!qual.is_null());
            assert_eq!(len, 4);
            assert_eq!(CStr::from_ptr(qual).to_bytes(), b"!!!!");
            free(qual.cast());
            assert_eq!(faidx_seq_len64(fai, r2.as_ptr()), 2);
            fai_destroy(fai);

            for row in [
                b"r1\t4\t4\t4\t5\n".as_slice(),
                b"r1\t4\t4\t4\t5\tbad\n".as_slice(),
            ] {
                fs::write(&fai_path, row).unwrap();
                let fai = fai_load3_format(path_c.as_ptr(), ptr::null(), ptr::null(), 0, FAI_FASTQ);
                assert!(fai.is_null());
            }

            for row in [
                b"r1\t4\t4\t0\t5\t11\n".as_slice(),
                b"r1\t4\t4\t5\t4\t11\n".as_slice(),
            ] {
                fs::write(&fai_path, row).unwrap();
                let fai = fai_load3_format(path_c.as_ptr(), ptr::null(), ptr::null(), 0, FAI_FASTQ);
                assert!(!fai.is_null());
                fai_destroy(fai);
            }
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn parse_fasta_fastq_duplicate_names_keep_first_index_entry() {
        let data = b">dup\nAAAA\n>dup\nTTTT\n>other\nCC\n";
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTA as c_int);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, b"dup");
        assert_eq!(rows[0].1.id, 0);
        assert_eq!(rows[0].1.seq_offset, 5);
        assert_eq!(rows[1].0, b"other");
        assert_eq!(rows[1].1.id, 1);
    }

    #[test]
    fn parse_fasta_fastq_counts_crlf_line_widths_like_fai() {
        let data = b">r1 comment\r\nACGT\r\nTG\r\n>r2\r\nNN\r\n";
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTA as c_int);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, b"r1");
        assert_eq!(rows[0].1.len, 6);
        assert_eq!(rows[0].1.seq_offset, 13);
        assert_eq!(rows[0].1.line_blen, 4);
        assert_eq!(rows[0].1.line_len, 6);
        assert_eq!(rows[1].0, b"r2");
        assert_eq!(rows[1].1.seq_offset, 28);
        assert_eq!(rows[1].1.line_blen, 2);
        assert_eq!(rows[1].1.line_len, 4);
    }

    #[test]
    fn parse_fasta_final_unterminated_line_counts_htslib_line_width() {
        let data = b">r1\nACGT";
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTA as c_int);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1.len, 4);
        assert_eq!(rows[0].1.seq_offset, 4);
        assert_eq!(rows[0].1.line_blen, 4);
        assert_eq!(rows[0].1.line_len, 5);
    }

    #[test]
    fn fai_load3_builds_fai_with_htslib_line_width_for_unterminated_final_line() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-build-unterminated-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fa.fai");
        fs::write(&path, b">chr1\nACGT").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let name = CString::new("chr1").unwrap();
            let fai = fai_load3(
                path_c.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                FAI_CREATE,
            );
            assert!(!fai.is_null());
            assert_eq!(faidx_seq_len64(fai, name.as_ptr()), 4);
            fai_destroy(fai);
        }

        assert_eq!(fs::read(&fai_path).unwrap(), b"chr1\t4\t6\t4\t5\n");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn parse_fasta_allows_empty_sequence_name_like_fai_fixture() {
        let data = include_bytes!("../htslib/test/faidx/faidx.fa");
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTA as c_int);
        assert_eq!(rows[0].0, b"");
        assert_eq!(rows[0].1.len, 4);
        assert_eq!(rows[0].1.seq_offset, 2);
    }

    #[test]
    fn fai_load3_reads_existing_fai_without_htslib_index_build() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-existing-fai-{}-{}.fa",
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
            "htslib_rs-build-fai-{}-{}.fa",
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
    fn fai_load_format_builds_and_fetches_plain_fastq_quality() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-build-fastq-{}-{}.fq",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fq.fai");
        fs::write(&path, b"@r1 comment\nACGT\n+\n!!!!\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let name = CString::new("r1").unwrap();
            let fai = fai_load3_format(
                path_c.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                FAI_CREATE,
                FAI_FASTQ,
            );
            assert!(!fai.is_null());
            assert!(fai_path.exists());
            assert_eq!((*fai).format, FAI_NONE as c_int);
            assert_eq!(faidx_seq_len64(fai, name.as_ptr()), 4);

            let mut len = 0;
            let qual = faidx_fetch_qual64(fai, name.as_ptr(), 1, 3, &mut len);
            assert!(!qual.is_null());
            assert_eq!(len, 3);
            assert_eq!(CStr::from_ptr(qual).to_bytes(), b"!!!");
            free(qual.cast());
            fai_destroy(fai);
        }

        let index = fs::read(&fai_path).unwrap();
        assert_eq!(index, b"r1\t4\t12\t4\t5\t19\n");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_load_format_fetches_fastq_sequence_and_quality_across_wrapped_lines() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fastq-wrapped-{}-{}.fq",
            std::process::id(),
            stamp
        ));
        let fai_path = path.with_extension("fq.fai");
        fs::write(&path, b"@r1\nACGT\nTG\n+\n!!!!\n??\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let name = CString::new("r1").unwrap();
            let fai = fai_load3_format(
                path_c.as_ptr(),
                ptr::null(),
                ptr::null(),
                FAI_CREATE,
                FAI_FASTQ,
            );
            assert!(!fai.is_null());

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, name.as_ptr(), 2, 5, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 4);
            assert_eq!(CStr::from_ptr(seq).to_bytes(), b"GTTG");
            free(seq.cast());

            let qual = faidx_fetch_qual64(fai, name.as_ptr(), 3, 5, &mut len);
            assert!(!qual.is_null());
            assert_eq!(len, 3);
            assert_eq!(CStr::from_ptr(qual).to_bytes(), b"!??");
            free(qual.cast());

            let empty = faidx_fetch_seq64(fai, name.as_ptr(), 6, 99, &mut len);
            assert!(!empty.is_null());
            assert_eq!(len, 0);
            assert_eq!(CStr::from_ptr(empty).to_bytes(), b"");
            free(empty.cast());

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn parse_fastq_keeps_at_sign_quality_until_expected_length() {
        let data = b"@r1\nABC\nDEF\n+\n@@@\n!!!\n@r2\nN\n+\n#\n";
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTQ as c_int);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, b"r1");
        assert_eq!(rows[0].1.len, 6);
        assert_eq!(rows[0].1.line_blen, 3);
        assert_eq!(rows[0].1.line_len, 4);
        assert_eq!(rows[0].1.qual_offset, 14);
        assert_eq!(rows[1].0, b"r2");
        assert_eq!(rows[1].1.len, 1);
    }

    #[test]
    fn parse_fastq_rejects_short_quality_before_next_record() {
        let data = b"@r1\nABC\n+\n!!\n@r2\nN\n+\n#\n";
        assert!(parse_fasta_fastq_index_rows(data).is_none());
    }

    #[test]
    fn parse_index_rows_rejects_mixed_fasta_fastq_records() {
        let data = b"@r1\nAC\n+\n!!\n>r2\nAC\n";
        assert!(parse_fasta_fastq_index_rows(data).is_none());
    }

    #[test]
    fn parse_index_rows_rejects_nonfinal_short_fasta_and_fastq_lines() {
        assert!(parse_fasta_fastq_index_rows(b">r1\nAC\nACGT\n").is_none());
        assert!(parse_fasta_fastq_index_rows(b"@r1\nAC\nACGT\n+\n!!!!!!\n").is_none());
        assert!(parse_fasta_fastq_index_rows(b"@r1\nACGT\n+\n!!\n!!\n").is_none());
    }

    #[test]
    fn parse_fasta_allows_blank_line_after_completed_sequence() {
        let data = b">r1\nACGT\n\n>r2\nNN\n";
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTA as c_int);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, b"r1");
        assert_eq!(rows[0].1.len, 4);
        assert_eq!(rows[1].0, b"r2");
        assert_eq!(rows[1].1.len, 2);
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
        }
    }

    #[test]
    fn fai_path_returns_or_builds_local_fai_path_like_htslib() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-fai-path-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        let fai_path_buf = path_bytes(&path).into_owned();
        let expected = {
            let mut bytes = fai_path_buf.clone();
            bytes.extend_from_slice(b".fai");
            bytes
        };
        fs::write(&path, b">chr1\nACGT\n").unwrap();

        unsafe {
            let path_c = CString::new(fai_path_buf).unwrap();
            let resolved = fai_path(path_c.as_ptr());
            assert!(!resolved.is_null());
            assert_eq!(CStr::from_ptr(resolved).to_bytes(), expected.as_slice());
            assert!(path_from_bytes(&expected).exists());
            free(resolved.cast());
        }

        let _ = fs::remove_file(path_from_bytes(&expected));
        let _ = fs::remove_file(path);
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

    #[test]
    fn original_fai_build_core_starts_next_fasta_record_at_name_after_marker() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-original-fai-build-fasta-{}-{}.fa",
            std::process::id(),
            stamp
        ));
        fs::write(&path, b">r1\nACGT\n>r2\nNN\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let bgzf = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let fai = faidx_c_132_fai_build_core(bgzf);
            bgzf_close(bgzf);

            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)).to_bytes(), b"r1");
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)).to_bytes(), b"r2");
            assert_eq!(faidx_seq_len64(fai, c"r1".as_ptr()), 4);
            assert_eq!(faidx_seq_len64(fai, c"r2".as_ptr()), 2);
            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn original_fai_build_core_treats_at_after_complete_quality_as_next_fastq_record() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "htslib_rs-original-fai-build-fastq-{}-{}.fq",
            std::process::id(),
            stamp
        ));
        fs::write(&path, b"@r1\nABC\n+\n@@@\n@r2\nN\n+\n#\n").unwrap();

        unsafe {
            let path_c = CString::new(path_bytes(&path).as_ref()).unwrap();
            let bgzf = bgzf_open(path_c.as_ptr(), c"r".as_ptr());
            assert!(!bgzf.is_null());
            let fai = faidx_c_132_fai_build_core(bgzf);
            bgzf_close(bgzf);

            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 0)).to_bytes(), b"r1");
            assert_eq!(CStr::from_ptr(faidx_iseq(fai, 1)).to_bytes(), b"r2");
            assert_eq!(faidx_seq_len64(fai, c"r1".as_ptr()), 3);
            assert_eq!(faidx_seq_len64(fai, c"r2".as_ptr()), 1);
            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
    }
}

// original: fai_insert_index (htslib/faidx.c:93)
pub unsafe fn faidx_c_93_fai_insert_index(
    idx: *mut faidx_t,
    name: *const c_char,
    len: u64,
    line_len: u32,
    line_blen: u32,
    seq_offset: u64,
    qual_offset: u64,
) -> c_int {
    if idx.is_null() || name.is_null() {
        return -1;
    }

    let name_key = libc::strdup(name);
    if name_key.is_null() {
        return -1;
    }

    let mut absent = 0;
    let k = kh_put_s((*idx).hash, name_key, &mut absent);
    if k == u32::MAX {
        free(name_key.cast());
        return -1;
    }

    if absent == 0 {
        free(name_key.cast());
        return 0;
    }

    if (*idx).n == (*idx).m {
        let new_m = if (*idx).m != 0 { (*idx).m << 1 } else { 16 };
        let tmp = realloc(
            (*idx).name.cast(),
            new_m as usize * std::mem::size_of::<*mut c_char>(),
        )
        .cast::<*mut c_char>();
        if tmp.is_null() {
            return -1;
        }
        (*idx).m = new_m;
        (*idx).name = tmp;
    }

    let v = (*(*idx).hash).vals.add(k as usize);
    (*v).id = (*idx).n;
    *(*idx).name.add((*idx).n as usize) = name_key;
    (*idx).n += 1;
    (*v).len = len;
    (*v).line_len = line_len;
    (*v).line_blen = line_blen;
    (*v).seq_offset = seq_offset;
    (*v).qual_offset = qual_offset;

    0
}

// original: fai_build_core (htslib/faidx.c:132)
pub unsafe fn faidx_c_132_fai_build_core(bgzf: *mut BGZF) -> *mut faidx_t {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum ReadState {
        OutRead,
        InName,
        InSeq,
        SeqEnd,
        InQual,
    }

    let idx = calloc_faidx();
    if idx.is_null() {
        return ptr::null_mut();
    }
    (*idx).hash = kh_init_s();
    if (*idx).hash.is_null() {
        fai_destroy(idx);
        return ptr::null_mut();
    }
    (*idx).format = FAI_NONE as c_int;

    let mut name = Vec::<u8>::new();
    let mut state = ReadState::OutRead;
    let mut read_done = false;
    let mut line_num = 1;
    let mut seq_offset = 0_u64;
    let mut qual_offset = 0_u64;
    let mut seq_len = 0_u64;
    let mut qual_len = 0_u64;
    let mut char_len = 0_u64;
    let mut line_len = 0_u64;

    let mut c = bgzf_getc(bgzf);
    while c >= 0 {
        match state {
            ReadState::OutRead => match c {
                x if x == b'>' as c_int => {
                    if (*idx).format == FAI_FASTQ as c_int {
                        goto_fai_build_core_fail(idx);
                        return ptr::null_mut();
                    }
                    (*idx).format = FAI_FASTA as c_int;
                    state = ReadState::InName;
                }
                x if x == b'@' as c_int => {
                    if (*idx).format == FAI_FASTA as c_int {
                        goto_fai_build_core_fail(idx);
                        return ptr::null_mut();
                    }
                    (*idx).format = FAI_FASTQ as c_int;
                    state = ReadState::InName;
                }
                x if x == b'\r' as c_int => {
                    c = bgzf_getc(bgzf);
                    if c == b'\n' as c_int {
                        line_num += 1;
                    } else {
                        goto_fai_build_core_fail(idx);
                        return ptr::null_mut();
                    }
                }
                x if x == b'\n' as c_int => {
                    line_num += 1;
                }
                _ => {
                    goto_fai_build_core_fail(idx);
                    return ptr::null_mut();
                }
            },
            ReadState::InName => {
                if read_done {
                    name.push(0);
                    if faidx_c_93_fai_insert_index(
                        idx,
                        name.as_ptr().cast(),
                        seq_len,
                        line_len as u32,
                        char_len as u32,
                        seq_offset,
                        qual_offset,
                    ) != 0
                    {
                        goto_fai_build_core_fail(idx);
                        return ptr::null_mut();
                    }
                    name.pop();
                    read_done = false;
                }

                name.clear();
                loop {
                    if libc::isspace(c as u8 as c_int) == 0 {
                        name.push(c as u8);
                    } else if !name.is_empty() || c == b'\n' as c_int {
                        break;
                    }

                    c = bgzf_getc(bgzf);
                    if c < 0 {
                        break;
                    }
                }

                if c < 0 {
                    goto_fai_build_core_fail(idx);
                    return ptr::null_mut();
                }

                while c != b'\n' as c_int {
                    c = bgzf_getc(bgzf);
                    if c < 0 {
                        break;
                    }
                }

                state = ReadState::InSeq;
                seq_len = 0;
                qual_len = 0;
                char_len = 0;
                line_len = 0;
                seq_offset = bgzf_utell(bgzf) as u64;
                line_num += 1;
            }
            ReadState::InSeq => {
                if (*idx).format == FAI_FASTA as c_int {
                    if c == b'\n' as c_int {
                        state = ReadState::OutRead;
                        line_num += 1;
                        c = bgzf_getc(bgzf);
                        continue;
                    } else if c == b'>' as c_int {
                        state = ReadState::InName;
                        c = bgzf_getc(bgzf);
                        continue;
                    }
                } else if (*idx).format == FAI_FASTQ as c_int {
                    if c == b'+' as c_int {
                        state = ReadState::InQual;
                        while c != b'\n' as c_int {
                            c = bgzf_getc(bgzf);
                            if c < 0 {
                                break;
                            }
                        }
                        qual_offset = bgzf_utell(bgzf) as u64;
                        line_num += 1;
                        c = bgzf_getc(bgzf);
                        continue;
                    } else if c == b'\n' as c_int {
                        goto_fai_build_core_fail(idx);
                        return ptr::null_mut();
                    }
                }

                let mut ll = 0_u64;
                let mut cl = 0_u64;
                if (*idx).format == FAI_FASTA as c_int {
                    read_done = true;
                }

                loop {
                    ll += 1;
                    if isgraph_(c as u8) != 0 {
                        cl += 1;
                    }
                    c = bgzf_getc(bgzf);
                    if c < 0 || c == b'\n' as c_int {
                        break;
                    }
                }

                ll += 1;
                seq_len += cl;
                if line_len == 0 {
                    line_len = ll;
                    char_len = cl;
                } else if line_len > ll {
                    state = if (*idx).format == FAI_FASTA as c_int {
                        ReadState::OutRead
                    } else {
                        ReadState::SeqEnd
                    };
                } else if line_len < ll {
                    goto_fai_build_core_fail(idx);
                    return ptr::null_mut();
                }
                line_num += 1;
            }
            ReadState::SeqEnd => {
                if c == b'+' as c_int {
                    state = ReadState::InQual;
                    while c != b'\n' as c_int {
                        c = bgzf_getc(bgzf);
                        if c < 0 {
                            break;
                        }
                    }
                    qual_offset = bgzf_utell(bgzf) as u64;
                    line_num += 1;
                } else {
                    goto_fai_build_core_fail(idx);
                    return ptr::null_mut();
                }
            }
            ReadState::InQual => {
                if c == b'\n' as c_int {
                    if !read_done {
                        goto_fai_build_core_fail(idx);
                        return ptr::null_mut();
                    }
                    state = ReadState::OutRead;
                    line_num += 1;
                    c = bgzf_getc(bgzf);
                    continue;
                } else if c == b'@' as c_int && read_done {
                    state = ReadState::InName;
                    c = bgzf_getc(bgzf);
                    continue;
                }

                let mut ll = 0_u64;
                let mut cl = 0_u64;
                loop {
                    ll += 1;
                    if isgraph_(c as u8) != 0 {
                        cl += 1;
                    }
                    c = bgzf_getc(bgzf);
                    if c < 0 || c == b'\n' as c_int {
                        break;
                    }
                }

                ll += 1;
                qual_len += cl;
                if line_len < ll {
                    goto_fai_build_core_fail(idx);
                    return ptr::null_mut();
                } else if qual_len == seq_len {
                    read_done = true;
                } else if qual_len > seq_len || line_len > ll {
                    goto_fai_build_core_fail(idx);
                    return ptr::null_mut();
                }
                line_num += 1;
            }
        }
        let _ = line_num;
        c = bgzf_getc(bgzf);
    }

    if read_done {
        name.push(0);
        if faidx_c_93_fai_insert_index(
            idx,
            name.as_ptr().cast(),
            seq_len,
            line_len as u32,
            char_len as u32,
            seq_offset,
            qual_offset,
        ) != 0
        {
            goto_fai_build_core_fail(idx);
            return ptr::null_mut();
        }
    } else {
        goto_fai_build_core_fail(idx);
        return ptr::null_mut();
    }

    idx
}

// original: fai_save (htslib/faidx.c:352)
pub unsafe fn faidx_c_352_fai_save(fai: *const faidx_t, fp: *mut hFILE) -> c_int {
    for i in 0..(*fai).n {
        let k = kh_get_s((*fai).hash, *(*fai).name.add(i as usize));
        if k >= (*(*fai).hash).n_buckets {
            return -1;
        }
        let x = *(*(*fai).hash).vals.add(k as usize);
        let buf = if (*fai).format == FAI_FASTA as c_int {
            format!(
                "\t{}\t{}\t{}\t{}\n",
                x.len, x.seq_offset, x.line_blen, x.line_len
            )
        } else {
            format!(
                "\t{}\t{}\t{}\t{}\t{}\n",
                x.len, x.seq_offset, x.line_blen, x.line_len, x.qual_offset
            )
        };

        let name = *(*fai).name.add(i as usize);
        if hputs2(name, CStr::from_ptr(name).to_bytes().len(), 0, fp) != 0 {
            return -1;
        }
        if hputs2(buf.as_ptr().cast(), buf.len(), 0, fp) != 0 {
            return -1;
        }
    }
    0
}

// original: fai_read (htslib/faidx.c:380)
pub unsafe fn faidx_c_380_fai_read(
    fp: *mut hFILE,
    _fname: *const c_char,
    format: c_int,
) -> *mut faidx_t {
    let fai = calloc_faidx();
    if fai.is_null() {
        return ptr::null_mut();
    }
    (*fai).hash = kh_init_s();
    if (*fai).hash.is_null() {
        fai_destroy(fai);
        return ptr::null_mut();
    }

    let buf = libc::calloc(0x10000, 1).cast::<c_char>();
    if buf.is_null() {
        fai_destroy(fai);
        return ptr::null_mut();
    }

    loop {
        let l = htslib_hfile_h_195_hgetln(buf, 0x10000, fp);
        if l <= 0 {
            if l < 0 {
                free(buf.cast());
                fai_destroy(fai);
                return ptr::null_mut();
            }
            break;
        }

        let mut p = buf;
        while *p != 0 && libc::isspace(*p as u8 as c_int) == 0 {
            p = p.add(1);
        }
        if p.offset_from(buf) < l {
            *p = 0;
            p = p.add(1);
        }

        let mut len = 0 as libc::c_ulong;
        let mut seq_offset = 0 as libc::c_ulong;
        let mut line_blen = 0 as libc::c_uint;
        let mut line_len = 0 as libc::c_uint;
        let mut qual_offset = 0 as libc::c_ulong;

        let n = if format == FAI_FASTA as c_int {
            libc::sscanf(
                p,
                c"%lu%lu%u%u".as_ptr(),
                &mut len,
                &mut seq_offset,
                &mut line_blen,
                &mut line_len,
            )
        } else {
            libc::sscanf(
                p,
                c"%lu%lu%u%u%lu".as_ptr(),
                &mut len,
                &mut seq_offset,
                &mut line_blen,
                &mut line_len,
                &mut qual_offset,
            )
        };

        if n != if format == FAI_FASTA as c_int { 4 } else { 5 } {
            free(buf.cast());
            fai_destroy(fai);
            return ptr::null_mut();
        }

        if faidx_c_93_fai_insert_index(
            fai,
            buf,
            len as u64,
            line_len as u32,
            line_blen as u32,
            seq_offset as u64,
            qual_offset as u64,
        ) != 0
        {
            free(buf.cast());
            fai_destroy(fai);
            return ptr::null_mut();
        }
    }

    free(buf.cast());
    fai
}

// original: fai_build3_core (htslib/faidx.c:460)
pub unsafe fn faidx_c_460_fai_build3_core(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
) -> c_int {
    let bgzf = bgzf_open(fn_, c"r".as_ptr());
    if bgzf.is_null() {
        return -1;
    }

    if bgzf_compression(bgzf) != 0 && bgzf_index_build_init(bgzf) != 0 {
        bgzf_close(bgzf);
        return -1;
    }

    let fai = faidx_c_132_fai_build_core(bgzf);
    if fai.is_null() {
        bgzf_close(bgzf);
        return -1;
    }

    let fai_name = owned_index_cstring(fn_, fnfai, b".fai");
    let gzi_name = owned_index_cstring(fn_, fngzi, b".gzi");
    let Some(fai_name) = fai_name else {
        bgzf_close(bgzf);
        fai_destroy(fai);
        return -1;
    };
    let Some(gzi_name) = gzi_name else {
        bgzf_close(bgzf);
        fai_destroy(fai);
        return -1;
    };

    if bgzf_compression(bgzf) != 0 && bgzf_index_dump(bgzf, gzi_name.as_ptr(), ptr::null()) < 0 {
        bgzf_close(bgzf);
        fai_destroy(fai);
        return -1;
    }

    if bgzf_close(bgzf) < 0 {
        fai_destroy(fai);
        return -1;
    }

    let fp = hopen(fai_name.as_ptr(), c"wb".as_ptr());
    if fp.is_null() {
        fai_destroy(fai);
        return -1;
    }

    if faidx_c_352_fai_save(fai, fp) != 0 {
        hclose_abruptly(fp);
        fai_destroy(fai);
        return -1;
    }

    if hclose(fp) != 0 {
        fai_destroy(fai);
        return -1;
    }

    fai_destroy(fai);
    0
}

// original: fai_build3 (htslib/faidx.c:557)
pub unsafe fn faidx_c_557_fai_build3(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
) -> c_int {
    faidx_c_460_fai_build3_core(fn_, fnfai, fngzi)
}

// original: fai_build (htslib/faidx.c:562)
pub unsafe fn faidx_c_562_fai_build(fn_: *const c_char) -> c_int {
    faidx_c_557_fai_build3(fn_, ptr::null(), ptr::null())
}

// original: fai_load3_core (htslib/faidx.c:567)
pub unsafe fn faidx_c_567_fai_load3_core(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
    format: c_int,
) -> *mut faidx_t {
    if fn_.is_null() {
        return ptr::null_mut();
    }

    let fai_name = owned_index_cstring(fn_, fnfai, b".fai");
    let gzi_name = owned_index_cstring(fn_, fngzi, b".gzi");
    let Some(fai_name) = fai_name else {
        return ptr::null_mut();
    };
    let Some(gzi_name) = gzi_name else {
        return ptr::null_mut();
    };

    let mut fp = hopen(fai_name.as_ptr(), c"rb".as_ptr());
    let mut gzi_index_needed = false;

    if !fp.is_null() {
        let bgzf = bgzf_open(fn_, c"rb".as_ptr());
        if bgzf.is_null() {
            hclose_abruptly(fp);
            return ptr::null_mut();
        }
        if bgzf_compression(bgzf) == 2 {
            let gz = hopen(gzi_name.as_ptr(), c"rb".as_ptr());
            if gz.is_null() {
                if (flags & FAI_CREATE) == 0 {
                    bgzf_close(bgzf);
                    hclose_abruptly(fp);
                    return ptr::null_mut();
                }
                gzi_index_needed = true;
                if hclose(fp) < 0 {
                    bgzf_close(bgzf);
                    return ptr::null_mut();
                }
                fp = ptr::null_mut();
            } else if hclose(gz) < 0 {
                bgzf_close(bgzf);
                hclose_abruptly(fp);
                return ptr::null_mut();
            }
        }
        bgzf_close(bgzf);
    }

    if fp.is_null() || gzi_index_needed {
        if (flags & FAI_CREATE) == 0 {
            return ptr::null_mut();
        }
        if faidx_c_460_fai_build3_core(fn_, fai_name.as_ptr(), gzi_name.as_ptr()) < 0 {
            return ptr::null_mut();
        }
        fp = hopen(fai_name.as_ptr(), c"rb".as_ptr());
        if fp.is_null() {
            return ptr::null_mut();
        }
    }

    let fai = faidx_c_380_fai_read(fp, fai_name.as_ptr(), format);
    if fai.is_null() {
        hclose_abruptly(fp);
        return ptr::null_mut();
    }

    if hclose(fp) < 0 {
        fai_destroy(fai);
        return ptr::null_mut();
    }

    (*fai).bgzf = bgzf_open(fn_, c"rb".as_ptr());
    if (*fai).bgzf.is_null() {
        fai_destroy(fai);
        return ptr::null_mut();
    }

    if bgzf_compression((*fai).bgzf) == 2
        && bgzf_index_load((*fai).bgzf, gzi_name.as_ptr(), ptr::null()) < 0
    {
        fai_destroy(fai);
        return ptr::null_mut();
    }

    fai
}

// original: fai_load3_format (htslib/faidx.c:705)
pub unsafe fn faidx_c_705_fai_load3_format(
    fn_: *const c_char,
    fnfai: *const c_char,
    fngzi: *const c_char,
    flags: c_int,
    format: fai_format_options,
) -> *mut faidx_t {
    faidx_c_567_fai_load3_core(fn_, fnfai, fngzi, flags, format as c_int)
}

// original: fai_load_format (htslib/faidx.c:711)
pub unsafe fn faidx_c_711_fai_load_format(
    fn_: *const c_char,
    format: fai_format_options,
) -> *mut faidx_t {
    faidx_c_705_fai_load3_format(fn_, ptr::null(), ptr::null(), FAI_CREATE, format)
}

// original: fai_thread_pool (htslib/faidx.c:1033)
pub unsafe fn faidx_c_1033_fai_thread_pool(
    fai: *mut faidx_t,
    pool: *mut hts_tpool,
    qsize: c_int,
) -> c_int {
    bgzf_thread_pool((*fai).bgzf, pool, qsize)
}

unsafe fn calloc_faidx() -> *mut faidx_t {
    libc::calloc(1, std::mem::size_of::<faidx_t>()).cast::<faidx_t>()
}

unsafe fn goto_fai_build_core_fail(idx: *mut faidx_t) {
    fai_destroy(idx);
}

unsafe fn kh_init_s() -> *mut faidx_hash_t {
    let h = libc::calloc(1, std::mem::size_of::<faidx_hash_t>()).cast::<faidx_hash_t>();
    if h.is_null() {
        return ptr::null_mut();
    }
    if kh_resize_s(h, 32) != 0 {
        free(h.cast());
        return ptr::null_mut();
    }
    h
}

unsafe fn kh_put_s(h: *mut faidx_hash_t, key: *const c_char, absent: *mut c_int) -> u32 {
    if (*h).n_occupied >= (*h).upper_bound {
        let new_n = if (*h).n_buckets != 0 {
            (*h).n_buckets << 1
        } else {
            32
        };
        if kh_resize_s(h, new_n) != 0 {
            return u32::MAX;
        }
    }

    let mask = (*h).n_buckets - 1;
    let mut k = kh_str_hash_string(key) & mask;
    let mut step = 0;
    while !kh_isempty((*h).flags, k) {
        if !kh_isdel((*h).flags, k) && cstr_eq(*(*h).keys.add(k as usize), key) {
            *absent = 0;
            return k;
        }
        step += 1;
        k = (k + step) & mask;
    }

    *(*h).keys.add(k as usize) = key.cast_mut();
    let flag = (*h).flags.add((k >> 4) as usize);
    *flag &= !(3 << ((k & 0x0f) << 1));
    (*h).size += 1;
    (*h).n_occupied += 1;
    *absent = 1;
    k
}

unsafe fn kh_resize_s(h: *mut faidx_hash_t, new_n_buckets: u32) -> c_int {
    let mut n_buckets = 4_u32;
    while n_buckets < new_n_buckets {
        n_buckets <<= 1;
    }
    let n_flags = if n_buckets < 16 {
        1
    } else {
        (n_buckets >> 4) as usize
    };

    let new_flags = malloc(n_flags * std::mem::size_of::<u32>()).cast::<u32>();
    let new_keys =
        malloc(n_buckets as usize * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    let new_vals = malloc(n_buckets as usize * std::mem::size_of::<faidx1_t>()).cast::<faidx1_t>();
    if new_flags.is_null() || new_keys.is_null() || new_vals.is_null() {
        free(new_flags.cast());
        free(new_keys.cast());
        free(new_vals.cast());
        return -1;
    }
    for i in 0..n_flags {
        *new_flags.add(i) = 0xaaaa_aaaa;
    }

    if !(*h).keys.is_null() {
        let old_n = (*h).n_buckets;
        let old_flags = (*h).flags;
        let old_keys = (*h).keys;
        let old_vals = (*h).vals;
        for i in 0..old_n {
            if !kh_iseither(old_flags, i) {
                let key = *old_keys.add(i as usize);
                let mask = n_buckets - 1;
                let mut k = kh_str_hash_string(key) & mask;
                let mut step = 0;
                while !kh_isempty(new_flags, k) {
                    step += 1;
                    k = (k + step) & mask;
                }
                *new_keys.add(k as usize) = key;
                *new_vals.add(k as usize) = *old_vals.add(i as usize);
                let flag = new_flags.add((k >> 4) as usize);
                *flag &= !(3 << ((k & 0x0f) << 1));
            }
        }
        free(old_flags.cast());
        free(old_keys.cast());
        free(old_vals.cast());
    }

    (*h).n_buckets = n_buckets;
    (*h).n_occupied = (*h).size;
    (*h).upper_bound = (n_buckets as f64 * 0.77) as u32;
    (*h).flags = new_flags;
    (*h).keys = new_keys;
    (*h).vals = new_vals;
    0
}

unsafe fn owned_index_cstring(
    fn_: *const c_char,
    explicit: *const c_char,
    suffix: &[u8],
) -> Option<std::ffi::CString> {
    let mut bytes = if explicit.is_null() {
        CStr::from_ptr(fn_).to_bytes().to_vec()
    } else {
        CStr::from_ptr(explicit).to_bytes().to_vec()
    };
    if explicit.is_null() {
        bytes.extend_from_slice(suffix);
    }
    std::ffi::CString::new(bytes).ok()
}
