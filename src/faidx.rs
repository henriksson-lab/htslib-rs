use std::{
    collections::HashSet,
    fs,
    io::{BufWriter, Write},
    ptr,
    ptr::NonNull,
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

pub const FAI_CREATE: i32 = 0x01;
pub type fai_format_options = i32;
pub const FAI_NONE: fai_format_options = 0;
pub const FAI_FASTA: fai_format_options = 1;
pub const FAI_FASTQ: fai_format_options = 2;
const HTS_PARSE_THOUSANDS_SEP: i32 = 1;
const HTS_PARSE_ONE_COORD: i32 = 2;
const HTS_PARSE_LIST: i32 = 4;

extern "C" {
    fn free(ptr: *mut ());
}

pub fn isgraph_(c: u8) -> i32 {
    is_graph_byte(c) as i32
}

fn is_graph_byte(c: u8) -> bool {
    c > b' ' && c <= b'~'
}

unsafe fn faidx_bgzf_getc(fp: &mut BGZF) -> i32 {
    if fp.block_offset + 1 < fp.block_length {
        let c = fp.uncompressed_block[fp.block_offset as usize];
        fp.block_offset += 1;
        fp.uncompressed_address += 1;
        return c as i32;
    }

    bgzf_getc(fp)
}

pub unsafe fn fai_path(fa: *const u8) -> *mut u8 {
    if fa.is_null() {
        return std::ptr::null_mut();
    }
    let mut fa_len = 0usize;
    while *fa.add(fa_len) != 0 {
        fa_len += 1;
    }
    let fa_bytes = std::slice::from_raw_parts(fa, fa_len);
    let delim = b"##idx##";
    if let Some(pos) = fa_bytes.windows(delim.len()).position(|w| w == delim) {
        let tail = &fa_bytes[pos + delim.len()..];
        let mut owned = Vec::with_capacity(tail.len() + 1);
        owned.extend_from_slice(tail);
        owned.push(0);
        return malloc_copy_c_bytes(&owned);
    }

    // RECONVERGE: hisremote/hts_idx_locatefn/hts_idx_check_local still take *const i8 (C ABI); cast at boundary.
    if hisremote(fa.cast()) != 0 {
        return hts_c_4920_hts_idx_locatefn(fa.cast(), c".fai".as_ptr()).cast();
    }

    let mut fai = std::ptr::null_mut();
    if hts_c_4756_hts_idx_check_local(fa.cast(), HTS_FMT_FAI, &mut fai) == 0
        && !fai.is_null()
        && fai_build3(fa, fai.cast(), std::ptr::null()) == -1
    {
        free(fai.cast());
        return std::ptr::null_mut();
    }
    fai.cast()
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct faidx1_t {
    pub id: i32,
    pub line_len: u32,
    pub line_blen: u32,
    pub len: u64,
    pub seq_offset: u64,
    pub qual_offset: u64,
}

pub struct faidx_hash_t {
    pub n_buckets: u32,
    pub size: u32,
    pub n_occupied: u32,
    pub upper_bound: u32,
    buckets: Vec<Option<faidx_hash_bucket_t>>,
}

#[derive(Clone, Copy)]
struct faidx_hash_bucket_t {
    name_id: usize,
    val: faidx1_t,
}

pub struct faidx_t {
    pub bgzf: Option<NonNull<BGZF>>,
    pub n: i32,
    pub m: i32,
    pub name: Vec<Vec<u8>>,
    pub hash: Box<faidx_hash_t>,
    pub format: i32,
}

type FaidxRows = Vec<(Vec<u8>, faidx1_t)>;

impl faidx_hash_t {
    fn with_capacity(min_buckets: usize) -> Self {
        let mut n_buckets = 4usize;
        while n_buckets < min_buckets {
            n_buckets <<= 1;
        }
        Self {
            n_buckets: n_buckets as u32,
            size: 0,
            n_occupied: 0,
            upper_bound: (n_buckets as f64 * 0.77) as u32,
            buckets: vec![None; n_buckets],
        }
    }

    fn clear_with_capacity(&mut self, min_buckets: usize) {
        *self = Self::with_capacity(min_buckets);
    }
}

impl faidx_t {
    fn new(format: i32) -> Self {
        Self {
            bgzf: None,
            n: 0,
            m: 0,
            name: Vec::new(),
            hash: Box::new(faidx_hash_t::with_capacity(32)),
            format,
        }
    }

    fn bgzf_ptr(&self) -> *mut BGZF {
        self.bgzf.map_or(ptr::null_mut(), NonNull::as_ptr)
    }

    fn set_bgzf(&mut self, bgzf: Option<NonNull<BGZF>>) -> bool {
        self.bgzf = bgzf;
        self.bgzf.is_some()
    }

    fn name_bytes(&self, i: usize) -> &[u8] {
        self.name[i].strip_suffix(&[0]).unwrap_or(&self.name[i])
    }
}

impl Drop for faidx_t {
    fn drop(&mut self) {
        close_fai_bgzf(self);
    }
}

pub unsafe fn fai_load3(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
) -> *mut faidx_t {
    fai_load3_core(fn_, fnfai, fngzi, flags, FAI_FASTA)
}

pub unsafe fn fai_load(_fn_: *const u8) -> *mut faidx_t {
    fai_load3(_fn_, std::ptr::null(), std::ptr::null(), FAI_CREATE)
}

pub unsafe fn fai_build3(fn_: *const u8, fnfai: *const u8, fngzi: *const u8) -> i32 {
    faidx_c_557_fai_build3(fn_, fnfai, fngzi)
}

pub unsafe fn fai_build(fn_: *const u8) -> i32 {
    fai_build3(fn_, std::ptr::null(), std::ptr::null())
}

pub unsafe fn fai_load3_format(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
    format: fai_format_options,
) -> *mut faidx_t {
    fai_load3_core(fn_, fnfai, fngzi, flags, format)
}

pub unsafe fn fai_load_format(fn_: *const u8, format: fai_format_options) -> *mut faidx_t {
    fai_load3_core(fn_, ptr::null(), ptr::null(), FAI_CREATE, format)
}

unsafe fn fai_load3_core(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
    format: i32,
) -> *mut faidx_t {
    if fn_.is_null() || (format != FAI_NONE && format != FAI_FASTA && format != FAI_FASTQ) {
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
        // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
        let bgzf = bgzf_open(fn_.cast(), c"r".as_ptr().cast());
        if bgzf.is_null() {
            return ptr::null_mut();
        }
        if bgzf_compression(bgzf) == 2 && !fngzi_path.as_ref().unwrap().exists() {
            build_index = true;
        }
        bgzf_close(bgzf);
    }
    if build_index && ((flags & FAI_CREATE) == 0 || fai_build3_core(fn_, fnfai, fngzi) != 0) {
        return ptr::null_mut();
    }
    let Some(fai) = fai_read(fn_, fnfai, format) else {
        return ptr::null_mut();
    };
    if bgzf_compression(fai.bgzf_ptr()) == 2 {
        let mut gzi_bytes = path_bytes(fngzi_path.as_ref().unwrap()).into_owned();
        gzi_bytes.push(0);
        if bgzf_index_load(fai.bgzf_ptr(), gzi_bytes.as_ptr().cast(), ptr::null()) < 0 {
            return ptr::null_mut();
        }
    }
    Box::into_raw(fai)
}

unsafe fn fai_build3_core(fn_: *const u8, fnfai: *const u8, fngzi: *const u8) -> i32 {
    if fn_.is_null() {
        return -1;
    }
    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    let bgzf = bgzf_open(fn_.cast(), c"r".as_ptr().cast());
    if bgzf.is_null() {
        return -1;
    }
    if bgzf_compression(bgzf) == 2 && bgzf_index_build_init(bgzf) != 0 {
        bgzf_close(bgzf);
        return -1;
    }
    let Some(fai) = fai_build_core(&mut *bgzf) else {
        bgzf_close(bgzf);
        return -1;
    };
    let fai_path = resolved_index_path(fn_, fnfai, b".fai");
    let gzi_path = resolved_index_path(fn_, fngzi, b".gzi");
    if fai_path.is_none() || gzi_path.is_none() {
        bgzf_close(bgzf);
        return -1;
    }
    if bgzf_compression(bgzf) == 2 {
        let mut gzi_bytes = path_bytes(gzi_path.as_ref().unwrap()).into_owned();
        gzi_bytes.push(0);
        if bgzf_index_dump(bgzf, gzi_bytes.as_ptr().cast(), ptr::null()) < 0 {
            bgzf_close(bgzf);
            return -1;
        }
    }
    if bgzf_close(bgzf) < 0 {
        return -1;
    }
    fai_save(&fai, fai_path.as_ref().unwrap())
}

fn resolved_index_path(
    fn_: *const u8,
    explicit: *const u8,
    suffix: &[u8],
) -> Option<std::path::PathBuf> {
    unsafe {
        if fn_.is_null() {
            return None;
        }
        if explicit.is_null() {
            let mut len = 0usize;
            while *fn_.add(len) != 0 {
                len += 1;
            }
            let fasta_path = path_from_bytes(std::slice::from_raw_parts(fn_, len));
            let mut bytes = path_bytes(&fasta_path).into_owned();
            bytes.extend_from_slice(suffix);
            Some(path_from_bytes(&bytes))
        } else {
            let mut len = 0usize;
            while *explicit.add(len) != 0 {
                len += 1;
            }
            Some(path_from_bytes(std::slice::from_raw_parts(explicit, len)))
        }
    }
}

unsafe fn fai_read(fn_: *const u8, fnfai: *const u8, format: i32) -> Option<Box<faidx_t>> {
    let fai_name = owned_index_c_bytes(fn_, fnfai, b".fai")?;
    let fp = hopen(fai_name.as_ptr().cast(), c"rb".as_ptr().cast());
    if fp.is_null() {
        return None;
    }

    let Some(mut fai) = fp.as_mut().and_then(|fp| faidx_read_owned(fp, format)) else {
        hclose_abruptly(fp);
        return None;
    };
    if hclose(fp) < 0 {
        return None;
    }

    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    if !fai.set_bgzf(NonNull::new(bgzf_open(fn_.cast(), c"rb".as_ptr().cast()))) {
        return None;
    }
    Some(fai)
}

fn is_fai_index_space(b: u8) -> bool {
    matches!(b, b'\t' | b'\n' | 0x0b | 0x0c | b'\r' | b' ')
}

unsafe fn fai_save(fai: &faidx_t, path: &std::path::Path) -> i32 {
    let file = match fs::File::create(path) {
        Ok(file) => file,
        Err(_) => return -1,
    };
    let mut out = BufWriter::new(file);

    for i in 0..fai.n {
        let name = fai.name_bytes(i as usize);
        let k = kh_get_s_bytes(fai, name);
        if k == fai.hash.n_buckets {
            return -1;
        }
        let val = fai.hash.buckets[k as usize].unwrap().val;

        if out.write_all(name).is_err() {
            return -1;
        }
        if fai.format == FAI_FASTQ {
            if writeln!(
                out,
                "\t{}\t{}\t{}\t{}\t{}",
                val.len, val.seq_offset, val.line_blen, val.line_len, val.qual_offset
            )
            .is_err()
            {
                return -1;
            }
        } else if writeln!(
            out,
            "\t{}\t{}\t{}\t{}",
            val.len, val.seq_offset, val.line_blen, val.line_len
        )
        .is_err()
        {
            return -1;
        }
    }

    out.flush().map(|_| 0).unwrap_or(-1)
}

unsafe fn fai_build_core(bgzf: &mut BGZF) -> Option<Box<faidx_t>> {
    faidx_build_core_owned(bgzf)
}

unsafe fn faidx_build_core_owned(bgzf: &mut BGZF) -> Option<Box<faidx_t>> {
    faidx_build_core_boxed(bgzf)
}

unsafe fn fai_insert_index(
    rows: &mut FaidxRows,
    name: Vec<u8>,
    len: u64,
    line_len: u32,
    line_blen: u32,
    seq_offset: u64,
    qual_offset: u64,
) -> i32 {
    if rows.iter().any(|(n, _)| *n == name) {
        return 0;
    }
    rows.push((
        name,
        faidx1_t {
            id: rows.len() as i32,
            line_len,
            line_blen,
            len,
            seq_offset,
            qual_offset,
        },
    ));
    0
}

fn parse_fasta_fastq_index_rows(data: &[u8]) -> Option<(FaidxRows, i32)> {
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut i = 0usize;
    let mut format = FAI_NONE;
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
        let this_format = if marker == b'>' { FAI_FASTA } else { FAI_FASTQ };
        if format != FAI_NONE && format != this_format {
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
            if format == FAI_FASTA && data[i] == b'>' {
                break;
            }
            if format == FAI_FASTA && (data[i] == b'\n' || data[i] == b'\r') && seq_len > 0 {
                break;
            }
            if format == FAI_FASTQ && data[i] == b'+' {
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
                if is_graph_byte(data[i]) {
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
        if format == FAI_FASTQ {
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
                    if is_graph_byte(data[i]) {
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
                    id: rows.len() as i32,
                    line_len,
                    line_blen,
                    len: seq_len,
                    seq_offset,
                    qual_offset,
                },
            ));
        }
    }
    if rows.is_empty() || format == FAI_NONE {
        None
    } else {
        Some((rows, format))
    }
}

unsafe fn fai_load_existing(fn_: *const u8, fnfai: *const u8) -> Option<Box<faidx_t>> {
    if fn_.is_null() {
        return None;
    }
    let mut fn_len = 0usize;
    while *fn_.add(fn_len) != 0 {
        fn_len += 1;
    }
    let fasta_path = path_from_bytes(std::slice::from_raw_parts(fn_, fn_len));
    let fai_path = if fnfai.is_null() {
        let mut bytes = path_bytes(&fasta_path).into_owned();
        bytes.extend_from_slice(b".fai");
        path_from_bytes(&bytes)
    } else {
        let mut len = 0usize;
        while *fnfai.add(len) != 0 {
            len += 1;
        }
        path_from_bytes(std::slice::from_raw_parts(fnfai, len))
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
                id: id as i32,
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

    fai_from_rows(fn_, rows, FAI_FASTA)
}

unsafe fn fai_from_rows(fn_: *const u8, rows: FaidxRows, format: i32) -> Option<Box<faidx_t>> {
    let mut fai = Box::new(faidx_t::new(format));
    fai.name.reserve(rows.len());
    fai.m = fai.name.capacity() as i32;
    fai.hash.clear_with_capacity((rows.len() * 2).max(4));
    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    if !fn_.is_null() && !fai.set_bgzf(NonNull::new(bgzf_open(fn_.cast(), c"r".as_ptr().cast()))) {
        return None;
    }

    for (name, mut val) in rows.into_iter() {
        if name.contains(&0) {
            close_fai_bgzf(&mut fai);
            return None;
        }
        if kh_get_s_bytes(&fai, &name) != fai.hash.n_buckets {
            continue;
        }
        if faidx_insert_owned_name(&mut fai, name, &mut val) != 0 {
            close_fai_bgzf(&mut fai);
            return None;
        }
    }

    Some(fai)
}

unsafe fn fai_build_plain_fasta(fn_: *const u8, fnfai: *const u8) -> Option<Box<faidx_t>> {
    if fn_.is_null() {
        return None;
    }
    let mut fn_len = 0usize;
    while *fn_.add(fn_len) != 0 {
        fn_len += 1;
    }
    let fasta_path = path_from_bytes(std::slice::from_raw_parts(fn_, fn_len));
    let fai_path = if fnfai.is_null() {
        let mut bytes = path_bytes(&fasta_path).into_owned();
        bytes.extend_from_slice(b".fai");
        path_from_bytes(&bytes)
    } else {
        let mut len = 0usize;
        while *fnfai.add(len) != 0 {
            len += 1;
        }
        path_from_bytes(std::slice::from_raw_parts(fnfai, len))
    };

    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    let bgzf = bgzf_open(fn_.cast(), c"r".as_ptr().cast());
    if bgzf.is_null() {
        return None;
    }

    let Some(mut fai) = fai_build_core(&mut *bgzf) else {
        bgzf_close(bgzf);
        return None;
    };
    if bgzf_close(bgzf) < 0 {
        return None;
    }
    if fai_save(&fai, &fai_path) != 0 {
        return None;
    }

    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    if !fai.set_bgzf(NonNull::new(bgzf_open(fn_.cast(), c"r".as_ptr().cast()))) {
        return None;
    }

    Some(fai)
}

pub unsafe fn fai_destroy(_fai: *mut faidx_t) {
    if _fai.is_null() {
        return;
    }
    drop(Box::from_raw(_fai));
}

fn close_fai_bgzf(fai: &mut faidx_t) {
    if let Some(bgzf) = fai.bgzf.take() {
        unsafe {
            bgzf_close(bgzf.as_ptr());
        }
    }
}

pub unsafe fn faidx_has_seq(_fai: *const faidx_t, _seq: *const u8) -> i32 {
    match (_fai.as_ref(), _seq.as_ref()) {
        (Some(fai), Some(_)) => {
            let mut len = 0usize;
            while *_seq.add(len) != 0 {
                len += 1;
            }
            faidx_has_seq_bytes(fai, std::slice::from_raw_parts(_seq, len)) as i32
        }
        _ => 0,
    }
}

fn faidx_has_seq_bytes(fai: &faidx_t, seq: &[u8]) -> bool {
    kh_get_s_bytes(fai, seq) != fai.hash.n_buckets
}

pub unsafe fn faidx_fetch_nseq(_fai: *const faidx_t) -> i32 {
    (*_fai).n
}

pub unsafe fn faidx_nseq(_fai: *const faidx_t) -> i32 {
    (*_fai).n
}

pub fn faidx_iseq(fai: &faidx_t, i: i32) -> Option<&[u8]> {
    (i >= 0)
        .then_some(i as usize)
        .filter(|&i| i < fai.name.len())
        // Names are stored NUL-terminated; the idiomatic byte API returns the
        // name CONTENT without the trailing NUL (matches `CStr::to_bytes`).
        .map(|i| fai.name_bytes(i))
}

pub unsafe fn faidx_seq_len64(_fai: *const faidx_t, _seq: *const u8) -> hts_pos_t {
    match (_fai.as_ref(), _seq.as_ref()) {
        (Some(fai), Some(_)) => {
            let mut len = 0usize;
            while *_seq.add(len) != 0 {
                len += 1;
            }
            faidx_seq_len64_bytes(fai, std::slice::from_raw_parts(_seq, len)).unwrap_or(-1)
        }
        _ => -1,
    }
}

fn faidx_seq_len64_bytes(fai: &faidx_t, seq: &[u8]) -> Option<hts_pos_t> {
    let k = kh_get_s_bytes(fai, seq);
    (k != fai.hash.n_buckets).then(|| fai.hash.buckets[k as usize].unwrap().val.len as hts_pos_t)
}

pub unsafe fn faidx_seq_len(_fai: *const faidx_t, _seq: *const u8) -> i32 {
    let len = faidx_seq_len64(_fai, _seq);
    if len < i32::MAX as hts_pos_t {
        len as i32
    } else {
        i32::MAX
    }
}

pub fn fai_adjust_region(fai: &faidx_t, tid: i32, beg: &mut hts_pos_t, end: &mut hts_pos_t) -> i32 {
    if tid < 0 || tid >= fai.n {
        return -1;
    }
    let orig_beg = *beg;
    let orig_end = *end;
    let name = fai.name_bytes(tid as usize);
    if faidx_adjust_position(fai, 0, None, name, beg, end, None) != 0 {
        return -1;
    }

    (if orig_beg != *beg { 1 } else { 0 })
        | (if orig_end != *end && orig_end < HTS_POS_MAX {
            2
        } else {
            0
        })
}

pub unsafe fn faidx_fetch_seq64(
    fai: *const faidx_t,
    c_name: *const u8,
    p_beg_i: hts_pos_t,
    p_end_i: hts_pos_t,
    len: *mut hts_pos_t,
) -> *mut u8 {
    let Some((fai, name, len)) = fai
        .as_ref()
        .zip(c_name.as_ref())
        .zip(len.as_mut())
        .map(|((fai, _), len)| {
            let mut name_len = 0usize;
            while *c_name.add(name_len) != 0 {
                name_len += 1;
            }
            (fai, std::slice::from_raw_parts(c_name, name_len), len)
        })
    else {
        return ptr::null_mut();
    };
    malloc_retrieved_c_bytes(faidx_fetch_seq64_bytes(
        fai, name, p_beg_i, p_end_i, len,
    ))
}

unsafe fn faidx_fetch_seq64_bytes(
    fai: &faidx_t,
    name: &[u8],
    mut p_beg_i: hts_pos_t,
    mut p_end_i: hts_pos_t,
    len: &mut hts_pos_t,
) -> Option<Vec<u8>> {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };

    if faidx_adjust_position(
        fai,
        1,
        Some(&mut val),
        name,
        &mut p_beg_i,
        &mut p_end_i,
        Some(len),
    ) != 0
    {
        return None;
    }

    fai_retrieve_bytes(fai, &val, val.seq_offset, p_beg_i, p_end_i + 1, len)
}

pub unsafe fn faidx_fetch_seq(
    fai: *const faidx_t,
    c_name: *const u8,
    p_beg_i: i32,
    p_end_i: i32,
    len: *mut i32,
) -> *mut u8 {
    if len.is_null() {
        return ptr::null_mut();
    }
    let mut len64 = 0;
    let ret = faidx_fetch_seq64(
        fai,
        c_name,
        p_beg_i as hts_pos_t,
        p_end_i as hts_pos_t,
        &mut len64,
    );
    *len = if len64 < i32::MAX as hts_pos_t {
        len64 as i32
    } else {
        i32::MAX
    };
    ret
}

// RECONVERGE: handed to hts_parse_region as the C-ABI hts_name2id_f callback
// (extern "C" fn(*mut c_void, *const c_char)); keep the ABI, body is native.
unsafe extern "C" fn fai_name2id(v: *mut std::ffi::c_void, ref_: *const i8) -> i32 {
    if v.is_null() || ref_.is_null() {
        return -1;
    }
    let fai = &*v.cast::<faidx_t>();
    let ref_ = ref_.cast::<u8>();
    let mut len = 0usize;
    while *ref_.add(len) != 0 {
        len += 1;
    }
    let k = kh_get_s_bytes(fai, std::slice::from_raw_parts(ref_, len));
    if k == fai.hash.n_buckets {
        -1
    } else {
        fai.hash.buckets[k as usize].unwrap().val.id
    }
}

fn fai_name2id_bytes(fai: &faidx_t, ref_: &[u8]) -> i32 {
    let k = kh_get_s_bytes(fai, ref_);
    if k == fai.hash.n_buckets {
        -1
    } else {
        fai.hash.buckets[k as usize].unwrap().val.id
    }
}

pub unsafe fn fai_parse_region(
    fai: *const faidx_t,
    s: *const u8,
    tid: *mut i32,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    flags: i32,
) -> *const u8 {
    // RECONVERGE: hts_parse_region still takes &CStr / *mut c_void and returns
    // *const c_char (C ABI); borrow the byte string as a CStr and cast at boundary.
    hts_parse_region(
        std::ffi::CStr::from_ptr(s.cast()),
        &mut *tid,
        &mut *beg,
        &mut *end,
        Some(fai_name2id),
        fai.cast_mut().cast(),
        flags,
    )
    .cast()
}

pub fn fai_set_cache_size(fai: &mut faidx_t, cache_size: i32) {
    unsafe {
        bgzf_set_cache_size(fai.bgzf_ptr(), cache_size);
    }
}

pub fn fai_thread_pool(fai: &mut faidx_t, pool: Option<NonNull<hts_tpool>>, qsize: i32) -> i32 {
    unsafe {
        bgzf_thread_pool(
            fai.bgzf_ptr(),
            pool.map_or(ptr::null_mut(), NonNull::as_ptr),
            qsize,
        )
    }
}

unsafe fn fai_get_val(
    fai: &faidx_t,
    str_: &[u8],
    len: &mut hts_pos_t,
    val: &mut faidx1_t,
    fbeg: &mut hts_pos_t,
    fend: &mut hts_pos_t,
) -> i32 {
    let mut id = 0;
    let mut beg = 0;
    let mut end = 0;
    let mut region = Vec::with_capacity(str_.len() + 1);
    region.extend_from_slice(str_);
    region.push(0);

    if fai_parse_region(
        (fai as *const faidx_t).cast_mut(),
        region.as_ptr().cast(),
        &mut id,
        &mut beg,
        &mut end,
        0,
    )
    .is_null()
    {
        *len = -2;
        return 1;
    }

    let seq_name = fai.name_bytes(id as usize);
    let iter = kh_get_s_bytes(fai, seq_name);
    if iter >= fai.hash.n_buckets {
        std::process::abort();
    }
    *val = fai.hash.buckets[iter as usize].unwrap().val;

    if beg >= val.len as hts_pos_t {
        beg = val.len as hts_pos_t;
    }
    if end >= val.len as hts_pos_t {
        end = val.len as hts_pos_t;
    }
    if beg > end {
        beg = end;
    }

    *fbeg = beg;
    *fend = end;
    0
}

pub unsafe fn fai_line_length(fai: *const faidx_t, str_: *const u8) -> hts_pos_t {
    if fai.is_null() || str_.is_null() {
        return -1;
    }
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
    let mut str_len = 0usize;
    while *str_.add(str_len) != 0 {
        str_len += 1;
    }
    if fai_get_val(
        &*fai,
        std::slice::from_raw_parts(str_, str_len),
        &mut len,
        &mut val,
        &mut beg,
        &mut end,
    ) != 0
    {
        -1
    } else {
        val.line_blen as hts_pos_t
    }
}

pub unsafe fn fai_fetch64(fai: *const faidx_t, str_: *const u8, len: *mut hts_pos_t) -> *mut u8 {
    let Some((fai, str_, len)) = fai
        .as_ref()
        .zip(str_.as_ref())
        .zip(len.as_mut())
        .map(|((fai, _), len)| {
            let mut n = 0usize;
            while *str_.add(n) != 0 {
                n += 1;
            }
            (fai, std::slice::from_raw_parts(str_, n), len)
        })
    else {
        return ptr::null_mut();
    };
    malloc_retrieved_c_bytes(fai_fetch64_bytes(fai, str_, len))
}

unsafe fn fai_fetch64_bytes(fai: &faidx_t, str_: &[u8], len: &mut hts_pos_t) -> Option<Vec<u8>> {
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
        return None;
    }
    fai_retrieve_bytes(fai, &val, val.seq_offset, beg, end, len)
}

pub unsafe fn fai_fetch(fai: *const faidx_t, str_: *const u8, len: *mut i32) -> *mut u8 {
    if len.is_null() {
        return ptr::null_mut();
    }
    let mut len64 = 0;
    let ret = fai_fetch64(fai, str_, &mut len64);
    *len = if len64 < i32::MAX as hts_pos_t {
        len64 as i32
    } else {
        i32::MAX
    };
    ret
}

pub unsafe fn fai_fetchqual64(
    fai: *const faidx_t,
    str_: *const u8,
    len: *mut hts_pos_t,
) -> *mut u8 {
    let Some((fai, str_, len)) = fai
        .as_ref()
        .zip(str_.as_ref())
        .zip(len.as_mut())
        .map(|((fai, _), len)| {
            let mut n = 0usize;
            while *str_.add(n) != 0 {
                n += 1;
            }
            (fai, std::slice::from_raw_parts(str_, n), len)
        })
    else {
        return ptr::null_mut();
    };
    malloc_retrieved_c_bytes(fai_fetchqual64_bytes(fai, str_, len))
}

unsafe fn fai_fetchqual64_bytes(
    fai: &faidx_t,
    str_: &[u8],
    len: &mut hts_pos_t,
) -> Option<Vec<u8>> {
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
        return None;
    }
    fai_retrieve_bytes(fai, &val, val.qual_offset, beg, end, len)
}

pub unsafe fn fai_fetchqual(fai: *const faidx_t, str_: *const u8, len: *mut i32) -> *mut u8 {
    if len.is_null() {
        return ptr::null_mut();
    }
    let mut len64 = 0;
    let ret = fai_fetchqual64(fai, str_, &mut len64);
    *len = if len64 < i32::MAX as hts_pos_t {
        len64 as i32
    } else {
        i32::MAX
    };
    ret
}

pub unsafe fn faidx_fetch_qual64(
    fai: *const faidx_t,
    c_name: *const u8,
    p_beg_i: hts_pos_t,
    p_end_i: hts_pos_t,
    len: *mut hts_pos_t,
) -> *mut u8 {
    let Some((fai, name, len)) = fai
        .as_ref()
        .zip(c_name.as_ref())
        .zip(len.as_mut())
        .map(|((fai, _), len)| {
            let mut n = 0usize;
            while *c_name.add(n) != 0 {
                n += 1;
            }
            (fai, std::slice::from_raw_parts(c_name, n), len)
        })
    else {
        return ptr::null_mut();
    };
    malloc_retrieved_c_bytes(faidx_fetch_qual64_bytes(fai, name, p_beg_i, p_end_i, len))
}

unsafe fn faidx_fetch_qual64_bytes(
    fai: &faidx_t,
    name: &[u8],
    mut p_beg_i: hts_pos_t,
    mut p_end_i: hts_pos_t,
    len: &mut hts_pos_t,
) -> Option<Vec<u8>> {
    let mut val = faidx1_t {
        id: 0,
        line_len: 0,
        line_blen: 0,
        len: 0,
        seq_offset: 0,
        qual_offset: 0,
    };

    if faidx_adjust_position(
        fai,
        1,
        Some(&mut val),
        name,
        &mut p_beg_i,
        &mut p_end_i,
        Some(len),
    ) != 0
    {
        return None;
    }

    fai_retrieve_bytes(fai, &val, val.qual_offset, p_beg_i, p_end_i + 1, len)
}

pub unsafe fn faidx_fetch_qual(
    fai: *const faidx_t,
    c_name: *const u8,
    p_beg_i: i32,
    p_end_i: i32,
    len: *mut i32,
) -> *mut u8 {
    if len.is_null() {
        return ptr::null_mut();
    }
    let mut len64 = 0;
    let ret = faidx_fetch_qual64(
        fai,
        c_name,
        p_beg_i as hts_pos_t,
        p_end_i as hts_pos_t,
        &mut len64,
    );
    *len = if len64 < i32::MAX as hts_pos_t {
        len64 as i32
    } else {
        i32::MAX
    };
    ret
}

fn kh_get_s_bytes(fai: &faidx_t, key: &[u8]) -> u32 {
    if fai.hash.n_buckets == 0 {
        return 0;
    }
    let h = &fai.hash;
    let mask = h.n_buckets - 1;
    let k = kh_str_hash_bytes(key);
    let mut i = k & mask;
    let last = i;
    let mut step = 0;
    while let Some(bucket) = h.buckets[i as usize] {
        if fai.name_bytes(bucket.name_id) == key {
            return i;
        }
        step += 1;
        i = (i + step) & mask;
        if i == last {
            return h.n_buckets;
        }
    }
    h.n_buckets
}

fn faidx_adjust_position(
    fai: &faidx_t,
    end_adjust: i32,
    val_out: Option<&mut faidx1_t>,
    c_name: &[u8],
    p_beg_i: &mut hts_pos_t,
    p_end_i: &mut hts_pos_t,
    len: Option<&mut hts_pos_t>,
) -> i32 {
    let iter = kh_get_s_bytes(fai, c_name);
    if iter == fai.hash.n_buckets {
        if let Some(len) = len {
            *len = -2;
        }
        return 1;
    }

    let val_ref = &fai.hash.buckets[iter as usize].unwrap().val;
    if let Some(val_out) = val_out {
        *val_out = *val_ref;
    }

    if *p_end_i < *p_beg_i {
        *p_beg_i = *p_end_i;
    }

    if *p_beg_i < 0 {
        *p_beg_i = 0;
    } else if val_ref.len as hts_pos_t <= *p_beg_i {
        *p_beg_i = val_ref.len as hts_pos_t;
    }

    if *p_end_i < 0 {
        *p_end_i = 0;
    } else if val_ref.len as hts_pos_t <= *p_end_i {
        *p_end_i = val_ref.len as hts_pos_t - end_adjust as hts_pos_t;
    }

    0
}

unsafe fn fai_retrieve_bytes(
    fai: &faidx_t,
    val: &faidx1_t,
    offset: u64,
    beg: hts_pos_t,
    end: hts_pos_t,
    len: &mut hts_pos_t,
) -> Option<Vec<u8>> {
    if (end as u64).wrapping_sub(beg as u64) >= usize::MAX as u64 - 2 {
        *len = -1;
        return None;
    }

    if val.line_blen == 0 {
        *len = -1;
        return None;
    }

    let ret = bgzf_useek(
        fai.bgzf_ptr(),
        (offset
            + (beg as u64 / val.line_blen as u64) * val.line_len as u64
            + beg as u64 % val.line_blen as u64) as i64,
        0,
    );
    if ret < 0 {
        *len = -1;
        return None;
    }

    let buffer_len = (end - beg) as usize + (val.line_len - val.line_blen) as usize + 1;
    let mut buffer = Vec::<u8>::new();
    if buffer.try_reserve_exact(buffer_len).is_err() {
        *len = -1;
        return None;
    }
    buffer.resize(buffer_len, 0);

    *len = end - beg;
    let mut remaining = *len as isize;
    let firstline_blen = val.line_blen as isize - (beg % val.line_blen as hts_pos_t) as isize;

    if remaining <= firstline_blen {
        let nread = bgzf_read(
            fai.bgzf_ptr(),
            buffer.as_mut_ptr().cast(),
            remaining as usize,
        );
        if nread < remaining {
            *len = -1;
            return None;
        }
        buffer[nread as usize] = 0;
        buffer.truncate(nread as usize + 1);
        return Some(buffer);
    }

    let mut write_pos = 0usize;
    let firstline_len = val.line_len as isize - (beg % val.line_blen as hts_pos_t) as isize;
    let mut nread = bgzf_read(
        fai.bgzf_ptr(),
        buffer.as_mut_ptr().add(write_pos).cast(),
        firstline_len as usize,
    );
    if nread < firstline_len {
        *len = -1;
        return None;
    }
    write_pos += firstline_blen as usize;
    remaining -= firstline_blen;

    while remaining > val.line_blen as isize {
        nread = bgzf_read(
            fai.bgzf_ptr(),
            buffer.as_mut_ptr().add(write_pos).cast(),
            val.line_len as usize,
        );
        if nread < val.line_len as isize {
            *len = -1;
            return None;
        }
        write_pos += val.line_blen as usize;
        remaining -= val.line_blen as isize;
    }

    if remaining > 0 {
        nread = bgzf_read(
            fai.bgzf_ptr(),
            buffer.as_mut_ptr().add(write_pos).cast(),
            remaining as usize,
        );
        if nread < remaining {
            *len = -1;
            return None;
        }
        write_pos += remaining as usize;
    }

    buffer[write_pos] = 0;
    buffer.truncate(write_pos + 1);
    Some(buffer)
}

unsafe fn malloc_retrieved_c_bytes(bytes: Option<Vec<u8>>) -> *mut u8 {
    match bytes {
        Some(bytes) => vec_into_returned_c_bytes(bytes),
        None => ptr::null_mut(),
    }
}

unsafe fn malloc_copy_c_bytes(bytes: &[u8]) -> *mut u8 {
    let mut out = Vec::new();
    if out.try_reserve(bytes.len().saturating_add(1)).is_err() {
        return ptr::null_mut();
    }
    out.extend_from_slice(bytes);
    vec_into_returned_c_bytes(out)
}

// Returns an owned (leaked Box<[u8]>) NUL-terminated buffer; reclaimed by
// `faidx_free_returned_c_bytes` in this module — no C allocator involved.
unsafe fn vec_into_returned_c_bytes(mut bytes: Vec<u8>) -> *mut u8 {
    if !bytes.ends_with(&[0]) {
        if bytes.try_reserve(1).is_err() {
            return ptr::null_mut();
        }
        bytes.push(0);
    }
    let mut bytes = bytes.into_boxed_slice();
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr
}

pub unsafe fn faidx_free_returned_c_bytes(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    drop(Box::from_raw(ptr::slice_from_raw_parts_mut(ptr, len + 1)));
}

fn kh_str_hash_bytes(bytes: &[u8]) -> u32 {
    let Some((&first, rest)) = bytes.split_first() else {
        return 0;
    };
    let mut h = first as u32;
    for &b in rest {
        h = (h << 5).wrapping_sub(h).wrapping_add(b as u32);
    }
    h
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use std::{
        ffi::{CStr, CString},
        fs,
        mem::{align_of, size_of},
        ptr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn faidx_record_layout_keeps_htslib_value_shape() {
        assert_eq!(size_of::<faidx1_t>(), 40);
        assert_eq!(align_of::<faidx1_t>(), 8);
    }

    #[test]
    fn faidx_has_seq_matches_khash_string_lookup_rules() {
        let absent = CString::new("chr2").unwrap();
        let mut fai = faidx_t::new(FAI_FASTA);
        let present = CString::new("chr1").unwrap();

        unsafe {
            assert_eq!(
                faidx_insert_index(&mut fai, present.to_bytes(), 100, 80, 81, 10, 0),
                0
            );
            assert_eq!(faidx_has_seq(&fai, present.as_ptr().cast()), 1);
            assert_eq!(faidx_has_seq(&fai, absent.as_ptr().cast()), 0);
            assert_eq!(faidx_has_seq(std::ptr::null(), present.as_ptr().cast()), 0);
            assert_eq!(faidx_fetch_nseq(&fai), 1);
            assert_eq!(faidx_nseq(&fai), 1);
            assert_eq!(faidx_iseq(&fai, 0).unwrap(), present.as_bytes());
            assert_eq!(faidx_seq_len64(&fai, present.as_ptr().cast()), 100);
            assert_eq!(faidx_seq_len64(&fai, absent.as_ptr().cast()), -1);
            assert_eq!(faidx_seq_len(&fai, present.as_ptr().cast()), 100);
            assert_eq!(faidx_seq_len(&fai, absent.as_ptr().cast()), -1);

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
    fn fai_destroy_accepts_box_allocated_index_shape() {
        unsafe {
            let fai = Box::into_raw(Box::new(faidx_t::new(FAI_FASTA)));
            fai_destroy(fai);
            fai_destroy(ptr::null_mut());
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
            let fai = fai_load(path_c.as_ptr().cast());
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_has_seq(fai, chr1.as_ptr().cast()), 1);
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr().cast()), 8);

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, chr1.as_ptr().cast(), 2, 6, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 5);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"GTTGC");
            faidx_free_returned_c_bytes(seq);

            let seq = faidx_fetch_seq64(fai, chr1.as_ptr().cast(), 1, 2, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 2);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"CG");
            faidx_free_returned_c_bytes(seq);

            let mut len32 = 0;
            let seq = faidx_fetch_seq(fai, chr1.as_ptr().cast(), 2, 6, &mut len32);
            assert!(!seq.is_null());
            assert_eq!(len32, 5);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"GTTGC");
            faidx_free_returned_c_bytes(seq);

            let reg = CString::new("chr1:3-7").unwrap();
            assert_eq!(fai_line_length(fai, reg.as_ptr().cast()), 4);
            let seq = fai_fetch64(fai, reg.as_ptr().cast(), &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 5);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"GTTGC");
            faidx_free_returned_c_bytes(seq);

            let seq = fai_fetch(fai, reg.as_ptr().cast(), &mut len32);
            assert!(!seq.is_null());
            assert_eq!(len32, 5);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"GTTGC");
            faidx_free_returned_c_bytes(seq);

            let absent = CString::new("absent").unwrap();
            let seq = faidx_fetch_seq64(fai, absent.as_ptr().cast(), 0, 1, &mut len);
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
            let fai = fai_load3(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, chr1.as_ptr().cast(), -5, 2, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 3);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"ACG");
            faidx_free_returned_c_bytes(seq);

            let seq = faidx_fetch_seq64(fai, chr1.as_ptr().cast(), 6, 2, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 1);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"G");
            faidx_free_returned_c_bytes(seq);

            let seq = faidx_fetch_seq64(fai, chr1.as_ptr().cast(), 99, 120, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 0);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"");
            faidx_free_returned_c_bytes(seq);

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
            let fai = fai_load3(path_c.as_ptr().cast(), std::ptr::null(), std::ptr::null(), 0);
            assert!(!fai.is_null());

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;

            let reg = CString::new("chr1:2-4").unwrap();
            assert!(
                !fai_parse_region(fai, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
            assert_eq!((tid, beg, end), (0, 1, 4));

            let reg = CString::new("{chr1:alt}:1-2").unwrap();
            assert!(
                !fai_parse_region(fai, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
            assert_eq!((tid, beg, end), (1, 0, 2));

            let reg = CString::new("chr1:3").unwrap();
            assert!(
                !fai_parse_region(fai, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
            assert_eq!((tid, beg, end), (0, 2, HTS_POS_MAX));

            let reg = CString::new("chr1:3").unwrap();
            assert!(!fai_parse_region(
                fai,
                reg.as_ptr().cast(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_ONE_COORD,
            )
            .is_null());
            assert_eq!((tid, beg, end), (0, 2, 3));

            let reg = CString::new("chr1:alt").unwrap();
            assert!(fai_parse_region(fai, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0).is_null());
            assert_eq!(tid, -1);

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn fai_parse_region_braced_ambiguous_name_alone_means_whole_contig() {
        unsafe {
            let mut fai = fai_from_rows(
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
                FAI_FASTA,
            )
            .unwrap();

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let reg = CString::new("{chr1:alt}").unwrap();
            let fai_ptr = &mut *fai as *mut faidx_t;
            let rest = fai_parse_region(fai_ptr, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0);
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest.cast()).to_bytes(), b"");
            assert_eq!((tid, beg, end), (1, 0, HTS_POS_MAX));

            let reg = CString::new("{chr1:alt}:2-1").unwrap();
            assert!(
                fai_parse_region(fai_ptr, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0).is_null()
            );
        }
    }

    #[test]
    fn fai_line_length_uses_parsed_region_name_and_reports_missing_names() {
        unsafe {
            let mut fai = fai_from_rows(
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
                FAI_FASTA,
            )
            .unwrap();

            let fai_ptr = &mut *fai as *mut faidx_t;
            assert_eq!(fai_line_length(fai_ptr, c"chr1:2-3".as_ptr().cast()), 80);
            assert_eq!(fai_line_length(fai_ptr, c"{chr1:alt}:2-3".as_ptr().cast()), 50);
            assert_eq!(fai_line_length(fai_ptr, c"missing:1-2".as_ptr().cast()), -1);
        }
    }

    #[test]
    fn fai_parse_region_list_mode_matches_htslib_comma_boundaries() {
        unsafe {
            let mut fai = fai_from_rows(
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
                FAI_FASTA,
            )
            .unwrap();

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let reg = CString::new("chr1,chr3").unwrap();
            let fai_ptr = &mut *fai as *mut faidx_t;
            let rest = fai_parse_region(
                fai_ptr,
                reg.as_ptr().cast(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST,
            );
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest.cast()).to_bytes(), b"chr3");
            assert_eq!((tid, beg, end), (0, 0, HTS_POS_MAX));

            let reg = CString::new("chr3:1,000-1,500").unwrap();
            let rest = fai_parse_region(
                fai_ptr,
                reg.as_ptr().cast(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST | HTS_PARSE_ONE_COORD,
            );
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest.cast()).to_bytes(), b"000-1,500");
            assert_eq!((tid, beg, end), (1, 0, 1));
        }
    }

    #[test]
    fn fai_parse_region_allows_thousands_separators_only_outside_list_mode() {
        unsafe {
            let mut fai = fai_from_rows(
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
                FAI_FASTA,
            )
            .unwrap();

            let mut tid = -1;
            let mut beg = -1;
            let mut end = -1;
            let reg = CString::new("chr1:1,000-1,002").unwrap();
            let fai_ptr = &mut *fai as *mut faidx_t;
            let rest = fai_parse_region(fai_ptr, reg.as_ptr().cast(), &mut tid, &mut beg, &mut end, 0);
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest.cast()).to_bytes(), b"");
            assert_eq!((tid, beg, end), (0, 999, 1002));

            let rest = fai_parse_region(
                fai_ptr,
                reg.as_ptr().cast(),
                &mut tid,
                &mut beg,
                &mut end,
                HTS_PARSE_LIST,
            );
            assert!(!rest.is_null());
            assert_eq!(CStr::from_ptr(rest.cast()).to_bytes(), b"000-1,002");
            assert_eq!((tid, beg, end), (0, 0, HTS_POS_MAX));
        }
    }

    #[test]
    fn resolved_index_path_honors_explicit_fai_and_gzi_paths() {
        let fasta = CString::new("/tmp/ref.fa").unwrap();
        let explicit_fai = CString::new("/tmp/custom.index").unwrap();
        let explicit_gzi = CString::new("/tmp/custom.gzi").unwrap();

        let inferred = resolved_index_path(fasta.as_ptr().cast(), ptr::null(), b".fai").unwrap();
        assert_eq!(path_bytes(&inferred).as_ref(), b"/tmp/ref.fa.fai");

        let explicit = resolved_index_path(fasta.as_ptr().cast(), explicit_fai.as_ptr().cast(), b".fai").unwrap();
        assert_eq!(path_bytes(&explicit).as_ref(), b"/tmp/custom.index");

        let explicit = resolved_index_path(fasta.as_ptr().cast(), explicit_gzi.as_ptr().cast(), b".gzi").unwrap();
        assert_eq!(path_bytes(&explicit).as_ref(), b"/tmp/custom.gzi");

        assert!(resolved_index_path(ptr::null(), explicit_fai.as_ptr().cast(), b".fai").is_none());
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
            let fai = fai_load3(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr().cast()), 4);
            assert_eq!(faidx_seq_len64(fai, chr2.as_ptr().cast()), 4);
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
            let fai = fai_load3(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr().cast()), 4);
            assert_eq!(faidx_seq_len64(fai, chr2.as_ptr().cast()), 4);
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
            let fai = fai_load3(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"dup");
            assert_eq!(faidx_iseq(&*fai, 1).unwrap(), b"other");
            assert_eq!(faidx_seq_len64(fai, dup.as_ptr().cast()), 4);
            assert_eq!(faidx_seq_len64(fai, other.as_ptr().cast()), 2);
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
                let fai = fai_load3(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0);
                assert!(!fai.is_null());
                assert_eq!(faidx_seq_len64(fai, chr1.as_ptr().cast()), 4);
                fai_destroy(fai);
            }

            fs::write(&fai_path, b"chr1\t4\tbad\t4\t5\n").unwrap();
            let fai = fai_load3(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0);
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
            let fai = fai_load3_format(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0, FAI_FASTQ);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"r1");
            assert_eq!(faidx_iseq(&*fai, 1).unwrap(), b"r2");

            let mut len = 0;
            let qual = faidx_fetch_qual64(fai, r1.as_ptr().cast(), 0, 3, &mut len);
            assert!(!qual.is_null());
            assert_eq!(len, 4);
            assert_eq!(CStr::from_ptr(qual.cast()).to_bytes(), b"!!!!");
            faidx_free_returned_c_bytes(qual);
            assert_eq!(faidx_seq_len64(fai, r2.as_ptr().cast()), 2);
            fai_destroy(fai);

            for row in [
                b"r1\t4\t4\t4\t5\n".as_slice(),
                b"r1\t4\t4\t4\t5\tbad\n".as_slice(),
            ] {
                fs::write(&fai_path, row).unwrap();
                let fai = fai_load3_format(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0, FAI_FASTQ);
                assert!(fai.is_null());
            }

            for row in [
                b"r1\t4\t4\t0\t5\t11\n".as_slice(),
                b"r1\t4\t4\t5\t4\t11\n".as_slice(),
            ] {
                fs::write(&fai_path, row).unwrap();
                let fai = fai_load3_format(path_c.as_ptr().cast(), ptr::null(), ptr::null(), 0, FAI_FASTQ);
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

        assert_eq!(format, FAI_FASTA);
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

        assert_eq!(format, FAI_FASTA);
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

        assert_eq!(format, FAI_FASTA);
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
                path_c.as_ptr().cast(),
                std::ptr::null(),
                std::ptr::null(),
                FAI_CREATE,
            );
            assert!(!fai.is_null());
            assert_eq!(faidx_seq_len64(fai, name.as_ptr().cast()), 4);
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

        assert_eq!(format, FAI_FASTA);
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
            let fai = fai_load3(path_c.as_ptr().cast(), std::ptr::null(), std::ptr::null(), 0);
            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"chr1");
            assert_eq!(faidx_iseq(&*fai, 1).unwrap(), b"chr2");
            assert_eq!(faidx_seq_len64(fai, chr1.as_ptr().cast()), 8);
            assert_eq!(faidx_seq_len64(fai, chr2.as_ptr().cast()), 4);

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, chr1.as_ptr().cast(), 4, 7, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 4);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"TGCA");
            faidx_free_returned_c_bytes(seq);

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
                path_c.as_ptr().cast(),
                std::ptr::null(),
                std::ptr::null(),
                FAI_CREATE,
            );
            assert!(!fai.is_null());
            assert!(fai_path.exists());
            assert_eq!(faidx_nseq(fai), 1);
            assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"sq\xff");
            assert_eq!(faidx_seq_len64(fai, seq_name.as_ptr().cast()), 5);

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, seq_name.as_ptr().cast(), 0, 4, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 5);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"ACGTT");
            faidx_free_returned_c_bytes(seq);
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
                path_c.as_ptr().cast(),
                std::ptr::null(),
                std::ptr::null(),
                FAI_CREATE,
                FAI_FASTQ,
            );
            assert!(!fai.is_null());
            assert!(fai_path.exists());
            assert_eq!((*fai).format, FAI_NONE);
            assert_eq!(faidx_seq_len64(fai, name.as_ptr().cast()), 4);

            let mut len = 0;
            let qual = faidx_fetch_qual64(fai, name.as_ptr().cast(), 1, 3, &mut len);
            assert!(!qual.is_null());
            assert_eq!(len, 3);
            assert_eq!(CStr::from_ptr(qual.cast()).to_bytes(), b"!!!");
            faidx_free_returned_c_bytes(qual);
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
                path_c.as_ptr().cast(),
                ptr::null(),
                ptr::null(),
                FAI_CREATE,
                FAI_FASTQ,
            );
            assert!(!fai.is_null());

            let mut len = 0;
            let seq = faidx_fetch_seq64(fai, name.as_ptr().cast(), 2, 5, &mut len);
            assert!(!seq.is_null());
            assert_eq!(len, 4);
            assert_eq!(CStr::from_ptr(seq.cast()).to_bytes(), b"GTTG");
            faidx_free_returned_c_bytes(seq);

            let qual = faidx_fetch_qual64(fai, name.as_ptr().cast(), 3, 5, &mut len);
            assert!(!qual.is_null());
            assert_eq!(len, 3);
            assert_eq!(CStr::from_ptr(qual.cast()).to_bytes(), b"!??");
            faidx_free_returned_c_bytes(qual);

            let empty = faidx_fetch_seq64(fai, name.as_ptr().cast(), 6, 99, &mut len);
            assert!(!empty.is_null());
            assert_eq!(len, 0);
            assert_eq!(CStr::from_ptr(empty.cast()).to_bytes(), b"");
            faidx_free_returned_c_bytes(empty);

            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&fai_path);
    }

    #[test]
    fn parse_fastq_keeps_at_sign_quality_until_expected_length() {
        let data = b"@r1\nABC\nDEF\n+\n@@@\n!!!\n@r2\nN\n+\n#\n";
        let (rows, format) = parse_fasta_fastq_index_rows(data).unwrap();

        assert_eq!(format, FAI_FASTQ);
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

        assert_eq!(format, FAI_FASTA);
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

        let mut fp = BGZF {
            bitfields: 0,
            cache_size: 0,
            block_length: 3,
            block_clength: 0,
            block_offset: 0,
            block_address: 0,
            uncompressed_address: 0,
            uncompressed_block: b"xyz".to_vec(),
            compressed_block: Vec::new(),
            cache: None,
            fp: crate::htslib_rs::bgzf::BgzfFp::None,
            mt: None,
            idx: None,
            idx_build_otf: 0,
            gz_stream: None,
            seeked: 0,
        };
        unsafe {
            assert_eq!(faidx_bgzf_getc(&mut fp), b'x' as i32);
            assert_eq!(fp.block_offset, 1);
            assert_eq!(fp.uncompressed_address, 1);

            let explicit = fai_path(c"ref.fa##idx##custom.fai".as_ptr().cast());
            assert!(!explicit.is_null());
            assert_eq!(CStr::from_ptr(explicit.cast()).to_bytes(), b"custom.fai");
            faidx_free_returned_c_bytes(explicit);
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
            let resolved = fai_path(path_c.as_ptr().cast());
            assert!(!resolved.is_null());
            assert_eq!(CStr::from_ptr(resolved.cast()).to_bytes(), expected.as_slice());
            assert!(path_from_bytes(&expected).exists());
            faidx_free_returned_c_bytes(resolved);
        }

        let _ = fs::remove_file(path_from_bytes(&expected));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn fai_set_cache_size_forwards_to_bgzf_when_cache_exists() {
        let mut bgzf = BGZF {
            bitfields: 0,
            cache_size: 0,
            block_length: 0,
            block_clength: 0,
            block_offset: 0,
            block_address: 0,
            uncompressed_address: 0,
            uncompressed_block: Vec::new(),
            compressed_block: Vec::new(),
            cache: Some(Box::new(super::super::bgzf::bgzf_cache_t::default())),
            fp: crate::htslib_rs::bgzf::BgzfFp::None,
            mt: None,
            idx: None,
            idx_build_otf: 0,
            gz_stream: None,
            seeked: 0,
        };
        let mut fai = faidx_t {
            bgzf: NonNull::new((&mut bgzf) as *mut BGZF),
            n: 0,
            m: 0,
            name: Vec::new(),
            hash: Box::new(faidx_hash_t::with_capacity(4)),
            format: 0,
        };
        unsafe {
            fai_set_cache_size(&mut fai, 4096);
        }
        assert_eq!(bgzf.cache_size, 4096);
        fai.bgzf = None;
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
            let bgzf = bgzf_open(path_c.as_ptr().cast(), c"r".as_ptr().cast());
            assert!(!bgzf.is_null());
            let fai = faidx_c_132_fai_build_core(bgzf);
            bgzf_close(bgzf);

            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"r1");
            assert_eq!(faidx_iseq(&*fai, 1).unwrap(), b"r2");
            assert_eq!(faidx_seq_len64(fai, c"r1".as_ptr().cast()), 4);
            assert_eq!(faidx_seq_len64(fai, c"r2".as_ptr().cast()), 2);
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
            let bgzf = bgzf_open(path_c.as_ptr().cast(), c"r".as_ptr().cast());
            assert!(!bgzf.is_null());
            let fai = faidx_c_132_fai_build_core(bgzf);
            bgzf_close(bgzf);

            assert!(!fai.is_null());
            assert_eq!(faidx_nseq(fai), 2);
            assert_eq!(faidx_iseq(&*fai, 0).unwrap(), b"r1");
            assert_eq!(faidx_iseq(&*fai, 1).unwrap(), b"r2");
            assert_eq!(faidx_seq_len64(fai, c"r1".as_ptr().cast()), 3);
            assert_eq!(faidx_seq_len64(fai, c"r2".as_ptr().cast()), 1);
            fai_destroy(fai);
        }

        let _ = fs::remove_file(&path);
    }
}

// original: fai_build_core (htslib/faidx.c:132)
pub unsafe fn faidx_c_132_fai_build_core(bgzf: *mut BGZF) -> *mut faidx_t {
    let Some(bgzf) = bgzf.as_mut() else {
        return ptr::null_mut();
    };
    faidx_build_core_boxed(bgzf).map_or(ptr::null_mut(), Box::into_raw)
}

unsafe fn faidx_build_core_boxed(bgzf: &mut BGZF) -> Option<Box<faidx_t>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum ReadState {
        OutRead,
        InName,
        InSeq,
        SeqEnd,
        InQual,
    }

    let mut idx = Box::new(faidx_t::new(FAI_NONE));

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
                x if x == b'>' as i32 => {
                    if idx.format == FAI_FASTQ {
                        return None;
                    }
                    idx.format = FAI_FASTA;
                    state = ReadState::InName;
                }
                x if x == b'@' as i32 => {
                    if idx.format == FAI_FASTA {
                        return None;
                    }
                    idx.format = FAI_FASTQ;
                    state = ReadState::InName;
                }
                x if x == b'\r' as i32 => {
                    c = bgzf_getc(bgzf);
                    if c == b'\n' as i32 {
                        line_num += 1;
                    } else {
                        return None;
                    }
                }
                x if x == b'\n' as i32 => {
                    line_num += 1;
                }
                _ => {
                    return None;
                }
            },
            ReadState::InName => {
                if read_done {
                    if name.contains(&0) {
                        return None;
                    }
                    if faidx_insert_index(
                        &mut idx,
                        &name,
                        seq_len,
                        line_len as u32,
                        char_len as u32,
                        seq_offset,
                        qual_offset,
                    ) != 0
                    {
                        return None;
                    }
                    read_done = false;
                }

                name.clear();
                loop {
                    if !is_fai_index_space(c as u8) {
                        name.push(c as u8);
                    } else if !name.is_empty() || c == b'\n' as i32 {
                        break;
                    }

                    c = bgzf_getc(bgzf);
                    if c < 0 {
                        break;
                    }
                }

                if c < 0 {
                    return None;
                }

                while c != b'\n' as i32 {
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
                if idx.format == FAI_FASTA {
                    if c == b'\n' as i32 {
                        state = ReadState::OutRead;
                        line_num += 1;
                        c = bgzf_getc(bgzf);
                        continue;
                    } else if c == b'>' as i32 {
                        state = ReadState::InName;
                        c = bgzf_getc(bgzf);
                        continue;
                    }
                } else if idx.format == FAI_FASTQ {
                    if c == b'+' as i32 {
                        state = ReadState::InQual;
                        while c != b'\n' as i32 {
                            c = bgzf_getc(bgzf);
                            if c < 0 {
                                break;
                            }
                        }
                        qual_offset = bgzf_utell(bgzf) as u64;
                        line_num += 1;
                        c = bgzf_getc(bgzf);
                        continue;
                    } else if c == b'\n' as i32 {
                        return None;
                    }
                }

                let mut ll = 0_u64;
                let mut cl = 0_u64;
                if idx.format == FAI_FASTA {
                    read_done = true;
                }

                loop {
                    ll += 1;
                    if is_graph_byte(c as u8) {
                        cl += 1;
                    }
                    c = bgzf_getc(bgzf);
                    if c < 0 || c == b'\n' as i32 {
                        break;
                    }
                }

                ll += 1;
                seq_len += cl;
                if line_len == 0 {
                    line_len = ll;
                    char_len = cl;
                } else if line_len > ll {
                    state = if idx.format == FAI_FASTA {
                        ReadState::OutRead
                    } else {
                        ReadState::SeqEnd
                    };
                } else if line_len < ll {
                    return None;
                }
                line_num += 1;
            }
            ReadState::SeqEnd => {
                if c == b'+' as i32 {
                    state = ReadState::InQual;
                    while c != b'\n' as i32 {
                        c = bgzf_getc(bgzf);
                        if c < 0 {
                            break;
                        }
                    }
                    qual_offset = bgzf_utell(bgzf) as u64;
                    line_num += 1;
                } else {
                    return None;
                }
            }
            ReadState::InQual => {
                if c == b'\n' as i32 {
                    if !read_done {
                        return None;
                    }
                    state = ReadState::OutRead;
                    line_num += 1;
                    c = bgzf_getc(bgzf);
                    continue;
                } else if c == b'@' as i32 && read_done {
                    state = ReadState::InName;
                    c = bgzf_getc(bgzf);
                    continue;
                }

                let mut ll = 0_u64;
                let mut cl = 0_u64;
                loop {
                    ll += 1;
                    if is_graph_byte(c as u8) {
                        cl += 1;
                    }
                    c = bgzf_getc(bgzf);
                    if c < 0 || c == b'\n' as i32 {
                        break;
                    }
                }

                ll += 1;
                qual_len += cl;
                if line_len < ll {
                    return None;
                } else if qual_len == seq_len {
                    read_done = true;
                } else if qual_len > seq_len || line_len > ll {
                    return None;
                }
                line_num += 1;
            }
        }
        let _ = line_num;
        c = bgzf_getc(bgzf);
    }

    if read_done {
        if name.contains(&0) {
            return None;
        }
        if faidx_insert_index(
            &mut idx,
            &name,
            seq_len,
            line_len as u32,
            char_len as u32,
            seq_offset,
            qual_offset,
        ) != 0
        {
            return None;
        }
    } else {
        return None;
    }

    Some(idx)
}

// original: fai_save (htslib/faidx.c:352)
pub unsafe fn faidx_c_352_fai_save(fai: *const faidx_t, fp: *mut hFILE) -> i32 {
    match (fai.as_ref(), fp.as_mut()) {
        (Some(fai), Some(fp)) => faidx_save_hfile(fai, fp),
        _ => -1,
    }
}

unsafe fn faidx_save_hfile(fai_ref: &faidx_t, fp: &mut hFILE) -> i32 {
    for i in 0..fai_ref.n {
        let name = fai_ref.name_bytes(i as usize);
        let k = kh_get_s_bytes(fai_ref, name);
        if k >= fai_ref.hash.n_buckets {
            return -1;
        }
        let x = fai_ref.hash.buckets[k as usize].unwrap().val;
        let buf = if fai_ref.format == FAI_FASTA {
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

        if hputs2(name.as_ptr().cast(), name.len(), 0, fp) != 0 {
            return -1;
        }
        if hputs2(buf.as_ptr().cast(), buf.len(), 0, fp) != 0 {
            return -1;
        }
    }
    0
}

// original: fai_read (htslib/faidx.c:380)
pub unsafe fn faidx_c_380_fai_read(fp: *mut hFILE, _fname: *const u8, format: i32) -> *mut faidx_t {
    let Some(fp) = fp.as_mut() else {
        return ptr::null_mut();
    };
    faidx_read_owned(fp, format).map_or(ptr::null_mut(), Box::into_raw)
}

unsafe fn faidx_read_owned(fp: &mut hFILE, format: i32) -> Option<Box<faidx_t>> {
    let mut fai = Box::new(faidx_t::new(FAI_NONE));

    let mut buf = Vec::new();
    if buf.try_reserve_exact(0x10000).is_err() {
        return None;
    }
    buf.resize(0x10000, 0_u8);
    let buf_ptr = buf.as_mut_ptr();

    loop {
        let l = htslib_hfile_h_195_hgetln(buf_ptr.cast(), buf.len(), fp);
        if l <= 0 {
            if l < 0 {
                return None;
            }
            break;
        }

        let line = &buf[..l as usize];
        let Some((name, rest)) = split_fai_name_and_fields(line) else {
            return None;
        };
        let Some((len, seq_offset, line_blen, line_len, qual_offset)) =
            parse_fai_numeric_fields(rest, format)
        else {
            return None;
        };
        if name.contains(&0) {
            return None;
        }

        if faidx_insert_index(
            &mut fai,
            name,
            len,
            line_len,
            line_blen,
            seq_offset,
            qual_offset,
        ) != 0
        {
            return None;
        }
    }

    Some(fai)
}

fn split_fai_name_and_fields(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let split = line.iter().position(|&b| is_fai_index_space(b))?;
    let name = &line[..split];
    let rest = line[split..].iter().position(|&b| !is_fai_index_space(b))?;
    Some((name, &line[split + rest..]))
}

fn parse_fai_numeric_fields(rest: &[u8], format: i32) -> Option<(u64, u64, u32, u32, u64)> {
    // Mirror the C `sscanf("%"SCNu64"%"SCNu64"%u%u"...)` behaviour: each conversion
    // skips leading whitespace then reads digits, leaving the cursor immediately
    // after the digits. Trailing junk after the final field is tolerated (sscanf has
    // already matched every required conversion), but junk attached to an earlier
    // field makes the following conversion start on a non-digit and fail.
    let mut cur = 0usize;
    let len = scan_ascii_u64(rest, &mut cur)?;
    let seq_offset = scan_ascii_u64(rest, &mut cur)?;
    let line_blen = scan_ascii_u64(rest, &mut cur)?.try_into().ok()?;
    let line_len = scan_ascii_u64(rest, &mut cur)?.try_into().ok()?;
    let qual_offset = if format == FAI_FASTA {
        0
    } else {
        scan_ascii_u64(rest, &mut cur)?
    };
    Some((len, seq_offset, line_blen, line_len, qual_offset))
}

fn scan_ascii_u64(buf: &[u8], cur: &mut usize) -> Option<u64> {
    while *cur < buf.len() && is_fai_index_space(buf[*cur]) {
        *cur += 1;
    }
    let start = *cur;
    let mut value = 0_u64;
    while *cur < buf.len() {
        let digit = buf[*cur].wrapping_sub(b'0');
        if digit > 9 {
            break;
        }
        value = value.checked_mul(10)?.checked_add(digit as u64)?;
        *cur += 1;
    }
    if *cur == start {
        return None;
    }
    Some(value)
}

// original: fai_build3_core (htslib/faidx.c:460)
pub unsafe fn faidx_c_460_fai_build3_core(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
) -> i32 {
    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    let bgzf = bgzf_open(fn_.cast(), c"r".as_ptr().cast());
    if bgzf.is_null() {
        return -1;
    }

    if bgzf_compression(bgzf) != 0 && bgzf_index_build_init(bgzf) != 0 {
        bgzf_close(bgzf);
        return -1;
    }

    let Some(bgzf_ref) = bgzf.as_mut() else {
        bgzf_close(bgzf);
        return -1;
    };
    let Some(fai) = faidx_build_core_boxed(bgzf_ref) else {
        bgzf_close(bgzf);
        return -1;
    };

    let fai_name = owned_index_c_bytes(fn_, fnfai, b".fai");
    let gzi_name = owned_index_c_bytes(fn_, fngzi, b".gzi");
    let Some(fai_name) = fai_name else {
        bgzf_close(bgzf);
        return -1;
    };
    let Some(gzi_name) = gzi_name else {
        bgzf_close(bgzf);
        return -1;
    };

    if bgzf_compression(bgzf) != 0
        && bgzf_index_dump(bgzf, gzi_name.as_ptr().cast(), ptr::null()) < 0
    {
        bgzf_close(bgzf);
        return -1;
    }

    if bgzf_close(bgzf) < 0 {
        return -1;
    }

    let fp = hopen(fai_name.as_ptr().cast(), c"wb".as_ptr().cast());
    if fp.is_null() {
        return -1;
    }

    let Some(fp_ref) = fp.as_mut() else {
        hclose_abruptly(fp);
        return -1;
    };
    if faidx_save_hfile(&fai, fp_ref) != 0 {
        hclose_abruptly(fp);
        return -1;
    }

    if hclose(fp) != 0 {
        return -1;
    }

    0
}

// original: fai_build3 (htslib/faidx.c:557)
pub unsafe fn faidx_c_557_fai_build3(fn_: *const u8, fnfai: *const u8, fngzi: *const u8) -> i32 {
    faidx_c_460_fai_build3_core(fn_, fnfai, fngzi)
}

// original: fai_build (htslib/faidx.c:562)
pub unsafe fn faidx_c_562_fai_build(fn_: *const u8) -> i32 {
    faidx_c_557_fai_build3(fn_, ptr::null(), ptr::null())
}

// original: fai_load3_core (htslib/faidx.c:567)
pub unsafe fn faidx_c_567_fai_load3_core(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
    format: i32,
) -> *mut faidx_t {
    if fn_.is_null() {
        return ptr::null_mut();
    }

    let fai_name = owned_index_c_bytes(fn_, fnfai, b".fai");
    let gzi_name = owned_index_c_bytes(fn_, fngzi, b".gzi");
    let Some(fai_name) = fai_name else {
        return ptr::null_mut();
    };
    let Some(gzi_name) = gzi_name else {
        return ptr::null_mut();
    };

    let mut fp = hopen(fai_name.as_ptr().cast(), c"rb".as_ptr().cast());
    let mut gzi_index_needed = false;

    if !fp.is_null() {
        // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
        let bgzf = bgzf_open(fn_.cast(), c"rb".as_ptr().cast());
        if bgzf.is_null() {
            hclose_abruptly(fp);
            return ptr::null_mut();
        }
        if bgzf_compression(bgzf) == 2 {
            let gz = hopen(gzi_name.as_ptr().cast(), c"rb".as_ptr().cast());
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
        if faidx_c_460_fai_build3_core(fn_, fai_name.as_ptr().cast(), gzi_name.as_ptr().cast()) < 0
        {
            return ptr::null_mut();
        }
        fp = hopen(fai_name.as_ptr().cast(), c"rb".as_ptr().cast());
        if fp.is_null() {
            return ptr::null_mut();
        }
    }

    let Some(mut fai) = fp.as_mut().and_then(|fp| faidx_read_owned(fp, format)) else {
        hclose_abruptly(fp);
        return ptr::null_mut();
    };

    if hclose(fp) < 0 {
        return ptr::null_mut();
    }

    // RECONVERGE: bgzf_open still takes *const c_char (C ABI); cast at boundary.
    if !fai.set_bgzf(NonNull::new(bgzf_open(fn_.cast(), c"rb".as_ptr().cast()))) {
        return ptr::null_mut();
    }

    if bgzf_compression(fai.bgzf_ptr()) == 2
        && bgzf_index_load(fai.bgzf_ptr(), gzi_name.as_ptr().cast(), ptr::null()) < 0
    {
        return ptr::null_mut();
    }

    Box::into_raw(fai)
}

// original: fai_load3_format (htslib/faidx.c:705)
pub unsafe fn faidx_c_705_fai_load3_format(
    fn_: *const u8,
    fnfai: *const u8,
    fngzi: *const u8,
    flags: i32,
    format: fai_format_options,
) -> *mut faidx_t {
    faidx_c_567_fai_load3_core(fn_, fnfai, fngzi, flags, format)
}

// original: fai_load_format (htslib/faidx.c:711)
pub unsafe fn faidx_c_711_fai_load_format(
    fn_: *const u8,
    format: fai_format_options,
) -> *mut faidx_t {
    faidx_c_705_fai_load3_format(fn_, ptr::null(), ptr::null(), FAI_CREATE, format)
}

// original: fai_thread_pool (htslib/faidx.c:1033)
pub unsafe fn faidx_c_1033_fai_thread_pool(
    fai: *mut faidx_t,
    pool: *mut hts_tpool,
    qsize: i32,
) -> i32 {
    bgzf_thread_pool((*fai).bgzf_ptr(), pool, qsize)
}

pub fn faidx_insert_index(
    idx: &mut faidx_t,
    name: &[u8],
    len: u64,
    line_len: u32,
    line_blen: u32,
    seq_offset: u64,
    qual_offset: u64,
) -> i32 {
    if kh_get_s_bytes(idx, name) != idx.hash.n_buckets {
        return 0;
    }
    let mut val = faidx1_t {
        id: 0,
        line_len,
        line_blen,
        len,
        seq_offset,
        qual_offset,
    };
    faidx_insert_owned_name(idx, name.to_vec(), &mut val)
}

fn faidx_insert_owned_name(idx: &mut faidx_t, mut name_key: Vec<u8>, val: &mut faidx1_t) -> i32 {
    if name_key.contains(&0) {
        return -1;
    }
    if idx.hash.n_occupied >= idx.hash.upper_bound {
        let new_n = if idx.hash.n_buckets != 0 {
            idx.hash.n_buckets << 1
        } else {
            32
        };
        if kh_resize_s(idx, new_n) != 0 {
            return -1;
        }
    }

    let name_id = idx.name.len();
    val.id = name_id as i32;
    let k = kh_empty_slot_bytes(idx, &name_key);
    if k == u32::MAX {
        return -1;
    }
    name_key.push(0);
    idx.name.push(name_key);
    idx.n = idx.name.len() as i32;
    idx.m = idx.name.capacity() as i32;
    idx.hash.buckets[k as usize] = Some(faidx_hash_bucket_t { name_id, val: *val });
    idx.hash.size += 1;
    idx.hash.n_occupied = idx.hash.size;
    0
}

fn kh_empty_slot_bytes(idx: &faidx_t, key: &[u8]) -> u32 {
    let h = &idx.hash;
    if h.n_buckets == 0 {
        return u32::MAX;
    }
    let mask = h.n_buckets - 1;
    let mut k = kh_str_hash_bytes(key) & mask;
    let mut step = 0;
    while h.buckets[k as usize].is_some() {
        step += 1;
        k = (k + step) & mask;
    }
    k
}

fn kh_resize_s(idx: &mut faidx_t, new_n_buckets: u32) -> i32 {
    let old_buckets = std::mem::take(&mut idx.hash.buckets);
    idx.hash.clear_with_capacity(new_n_buckets as usize);
    for bucket in old_buckets.into_iter().flatten() {
        let key = idx.name_bytes(bucket.name_id);
        let k = kh_empty_slot_bytes(idx, key);
        if k == u32::MAX {
            return -1;
        }
        idx.hash.buckets[k as usize] = Some(bucket);
        idx.hash.size += 1;
    }
    idx.hash.n_occupied = idx.hash.size;
    0
}

unsafe fn owned_index_c_bytes(
    fn_: *const u8,
    explicit: *const u8,
    suffix: &[u8],
) -> Option<Vec<u8>> {
    let src = if explicit.is_null() { fn_ } else { explicit };
    let mut src_len = 0usize;
    while *src.add(src_len) != 0 {
        src_len += 1;
    }
    let mut bytes = std::slice::from_raw_parts(src, src_len).to_vec();
    if bytes.contains(&0) {
        return None;
    }
    if explicit.is_null() {
        bytes.extend_from_slice(suffix);
    }
    bytes.push(0);
    Some(bytes)
}
