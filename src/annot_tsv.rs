#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]

use crate::htslib_rs::{
    bgzf,
    hts::{htsFile, hts_close, hts_getline, hts_open, hts_pos_t, kstring_t, BGZF},
    regidx,
};
use std::ffi::{c_void, CStr};

const ANNOT_TSV_ANN_NBP: i32 = 1;
const ANNOT_TSV_ANN_FRAC: i32 = 2;
const ANNOT_TSV_ANN_CNT: i32 = 4;
const ANNOT_TSV_PRINT_MATCHING: i32 = 1;
const ANNOT_TSV_PRINT_NONMATCHING: i32 = 2;

/// A split line: each column is an owned, NUL-free byte string.
///
/// This replaces the C idiom of a single `rmme` buffer plus an array of
/// interior `*mut i8` offsets. The number of live columns is simply
/// `self.cols.len()`.
#[derive(Default)]
struct AnnotTsvCols {
    cols: Vec<Vec<u8>>,
}

struct AnnotTsvHdr {
    // Maps a column name to its index. Replaces the C khash_str2int handle.
    name2idx: std::collections::HashMap<Vec<u8>, i32>,
    cols: Option<Box<AnnotTsvCols>>,
    annots: Option<Box<AnnotTsvCols>>,
    dummy: i32,
}

struct AnnotTsvDatHeaderOnly {
    fname: Vec<u8>,
    hdr: AnnotTsvHdr,
}

pub struct AnnotTsvDat {
    fname: Vec<u8>,
    hdr: AnnotTsvHdr,
    core: Option<Box<AnnotTsvCols>>,
    match_: Option<Box<AnnotTsvCols>>,
    transfer: Option<Box<AnnotTsvCols>>,
    annots: Option<Box<AnnotTsvCols>>,
    core_idx: Vec<i32>,
    match_idx: Vec<i32>,
    transfer_idx: Vec<i32>,
    annots_idx: Vec<i32>,
    nannots_added: Vec<i32>,
    coor_base: [i32; 2],
    delim: u8,
    grow_n: i32,
    line: kstring_t,
    fp: *mut htsFile,
}

impl Default for AnnotTsvDat {
    fn default() -> Self {
        Self {
            fname: Vec::new(),
            hdr: AnnotTsvHdr {
                name2idx: std::collections::HashMap::new(),
                cols: None,
                annots: None,
                dummy: 0,
            },
            core: None,
            match_: None,
            transfer: None,
            annots: None,
            core_idx: Vec::new(),
            match_idx: Vec::new(),
            transfer_idx: Vec::new(),
            annots_idx: Vec::new(),
            nannots_added: Vec::new(),
            coor_base: [0; 2],
            delim: 0,
            grow_n: 0,
            line: kstring_t { data: Vec::new() },
            fp: std::ptr::null_mut(),
        }
    }
}

#[derive(Default)]
struct AnnotTsvNbp {
    regs: Vec<hts_pos_t>,
    beg: hts_pos_t,
    end: hts_pos_t,
}

struct ParsedTab {
    cols: Box<AnnotTsvCols>,
    /// Chromosome name (the first core column), NUL-free.
    chr: Vec<u8>,
    beg: hts_pos_t,
    end: hts_pos_t,
}

pub struct AnnotTsvArgs {
    nbp: Option<Box<AnnotTsvNbp>>,
    dst: AnnotTsvDat,
    src: AnnotTsvDat,
    core_str: Vec<u8>,
    coords_str: Vec<u8>,
    match_str: Vec<u8>,
    transfer_str: Vec<u8>,
    annots_str: Vec<u8>,
    headers_str: Vec<u8>,
    delim_str: Vec<u8>,
    temp_dir: Vec<u8>,
    out_fname: Vec<u8>,
    out_fp: *mut BGZF,
    allow_dups: i32,
    max_annots: i32,
    mode: i32,
    no_write_hdr: i32,
    overlap_either: i32,
    overlap_src: f64,
    overlap_dst: f64,
    idx: Option<Box<regidx::regidx_t>>,
    itr: Option<Box<regidx::regitr_t>>,
    tmp_kstr: kstring_t,
    tmp_cols: Vec<AnnotTsvCols>,
    // One de-dup set per transfer column. Replaces the C khash_str2int handles.
    tmp_hash: Vec<std::collections::HashSet<Vec<u8>>>,
}

impl Default for AnnotTsvArgs {
    fn default() -> Self {
        Self {
            nbp: None,
            dst: AnnotTsvDat::default(),
            src: AnnotTsvDat::default(),
            core_str: Vec::new(),
            coords_str: Vec::new(),
            match_str: Vec::new(),
            transfer_str: Vec::new(),
            annots_str: Vec::new(),
            headers_str: Vec::new(),
            delim_str: Vec::new(),
            temp_dir: Vec::new(),
            out_fname: Vec::new(),
            out_fp: std::ptr::null_mut(),
            allow_dups: 0,
            max_annots: 0,
            mode: 0,
            no_write_hdr: 0,
            overlap_either: 0,
            overlap_src: 0.0,
            overlap_dst: 0.0,
            idx: None,
            itr: None,
            tmp_kstr: kstring_t { data: Vec::new() },
            tmp_cols: Vec::new(),
            tmp_hash: Vec::new(),
        }
    }
}

static ANNOT_TSV_USAGE_TEXT: &[u8] =
    b"About: Annotate regions of the target file (TGT) with information from
       overlapping regions of the source file (SRC). Multiple columns can be
       transferred (-f) and the transfer can be conditioned on requiring
       matching values in one or more columns (-m).
       In addition to column transfer (-f) and special annotations (-a), the
       program can operate in a simple grep-like mode and print matching lines
       (when neither -f nor -a are given) or drop matching lines (-x).
       All indexes and coordinates are 1-based and inclusive.

Usage: annot-tsv [OPTIONS] -s source.txt -t target.txt > output.txt

Common options:
   -c, --core SRC:TGT      Core columns in SRC and TGT file
                             [chr,beg,end:chr,beg,end]
   -f, --transfer SRC:TGT  Columns to transfer. If SRC column does not exist,
                           interpret as the default value to use. If the TGT
                           column does not exist, a new column is created. If
                           the TGT column does exist, its values are overwritten
                           when overlap is found or left as is otherwise.
   -m, --match SRC:TGT     Require match in these columns for annotation
                           transfer
   -o, --output FILE       Output file name [STDOUT]
   -s, --source-file FILE  Source file to take annotations from
   -t, --target-file FILE  Target file to be extend with annotations from -s

Other options:
       --allow-dups        Add annotations multiple times
       --help              This help message
       --max-annots INT    Adding at most INT annotations per column to save
                           time in big regions
       --version           Print version string and exit
   -a, --annotate LIST     Add special annotations, one or more of:
                             cnt  .. number of overlapping regions
                             frac .. fraction of the target region with an
                                       overlap
                             nbp  .. number of source base pairs in the overlap
   -C, --coords SRC:TGT    Are coordinates 0 or 1-based, BED=01, TSV=11 [11]
   -d, --delim SRC:TGT     Column delimiter in SRC and TGT file
   -h, --headers SRC:TGT   Header row line number, 0:0 is equivalent to -H, negative
                             value counts from the end of comment line block [1:1]
   -H, --ignore-headers    Use numeric indices, ignore the headers completely
   -I, --no-header-idx     Suppress index numbers in the printed header. If given
                           twice, drop the entire header
   -O, --overlap FLOAT[,FLOAT]     Minimum required overlap with respect to SRC,TGT.
                           If single value, the bigger overlap is considered.
                           Identical values are equivalent to running with -r.
   -r, --reciprocal        Apply the -O requirement to both overlapping
                           intervals
   -x, --drop-overlaps     Drop overlapping regions (precludes -f)

Examples:
   # Header is present, match and transfer by column name
   annot-tsv -s src.txt.gz -t tgt.txt.gz -c chr,beg,end:CHR,POS,POS \\
       -m type,sample:TYPE,SMPL -f info:INFO

   # Header is not present, match and transfer by column index (1-based)
   annot-tsv -s src.txt.gz -t tgt.txt.gz -c 1,2,3:1,2,3 -m 4,5:4,5 -f 6:6

   # If the TGT part is not given, the program assumes that the SRC:TGT columns
   # are identical
   annot-tsv -s src.txt.gz -t tgt.txt.gz -c chr,beg,end -m type,sample -f info

   # One of the SRC or TGT file can be streamed from stdin
   gunzip -c src.txt.gz | \\
       annot-tsv -t tgt.txt.gz -c chr,beg,end -m type,sample -f info
   gunzip -c tgt.txt.gz | \\
       annot-tsv -s src.txt.gz -c chr,beg,end -m type,sample -f info

   # Print matching regions as above but without modifying the records
   gunzip -c src.txt.gz | annot-tsv -t tgt.txt.gz -c chr,beg,end -m type,sample

\0";

fn nbp_reset(nbp: &mut AnnotTsvNbp, beg: hts_pos_t, end: hts_pos_t) {
    nbp.regs.clear();
    nbp.beg = beg;
    nbp.end = end;
}

fn nbp_add(nbp: &mut AnnotTsvNbp, beg: hts_pos_t, end: hts_pos_t) {
    nbp.regs.push(beg << 1);
    nbp.regs.push((end << 1) + 1);
}

fn nbp_length(nbp: &mut AnnotTsvNbp) -> hts_pos_t {
    if nbp.regs.is_empty() {
        return 0;
    }
    nbp.regs.sort_unstable();

    let mut nopen = 0;
    let mut beg = 0;
    let mut length = 0;
    for &reg in nbp.regs.iter() {
        if reg & 1 == 0 {
            if nopen == 0 {
                beg = reg >> 1;
            }
            nopen += 1;
        } else {
            nopen -= 1;
        }
        assert!(nopen >= 0);
        if nopen == 0 && beg > 0 {
            length += (reg >> 1) - beg + 1;
        }
    }
    length
}

// original: nbp_destroy (htslib/annot-tsv.c:137)
pub unsafe fn annot_tsv_c_137_nbp_destroy(nbp: *mut c_void) {
    let nbp = nbp.cast::<AnnotTsvNbp>();
    if !nbp.is_null() {
        drop(Box::from_raw(nbp));
    }
}

// original: nbp_reset (htslib/annot-tsv.c:142)
pub unsafe fn annot_tsv_c_142_nbp_reset(nbp: *mut c_void, beg: hts_pos_t, end: hts_pos_t) {
    let nbp = nbp.cast::<AnnotTsvNbp>();
    nbp_reset(&mut *nbp, beg, end);
}

// original: nbp_add (htslib/annot-tsv.c:148)
pub unsafe fn annot_tsv_c_148_nbp_add(nbp: *mut c_void, beg: hts_pos_t, end: hts_pos_t) {
    let nbp = nbp.cast::<AnnotTsvNbp>();
    nbp_add(&mut *nbp, beg, end);
}

// original: compare_hts_pos (htslib/annot-tsv.c:160)
pub unsafe fn annot_tsv_c_160_compare_hts_pos(aptr: *const c_void, bptr: *const c_void) -> i32 {
    let a = *aptr.cast::<hts_pos_t>();
    let b = *bptr.cast::<hts_pos_t>();
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

// original: nbp_length (htslib/annot-tsv.c:168)
pub unsafe fn annot_tsv_c_168_nbp_length(nbp: *mut c_void) -> hts_pos_t {
    let nbp = nbp.cast::<AnnotTsvNbp>();
    nbp_length(&mut *nbp)
}

fn cols_split_into(line: &[u8], cols: &mut AnnotTsvCols, delim: u8) {
    cols.cols.clear();
    // Split on `delim`. A trailing delimiter produces a trailing empty field,
    // matching the C behaviour of always emitting at least one column.
    for field in line.split(|&byte| byte == delim) {
        cols.cols.push(field.to_vec());
    }
}

fn cols_split_box(line: &[u8], delim: u8) -> Box<AnnotTsvCols> {
    let mut cols = Box::<AnnotTsvCols>::default();
    cols_split_into(line, &mut cols, delim);
    cols
}

fn cols_append(cols: &mut AnnotTsvCols, str_: &[u8]) {
    cols.cols.push(str_.to_vec());
}

fn cols_clear(cols: &mut AnnotTsvCols) {
    cols.cols.clear();
}

fn annot_tsv_parse_tab_with_payload(line: &[u8], dat: &mut AnnotTsvDat) -> Option<ParsedTab> {
    if line.first() == Some(&b'#') {
        return None;
    }

    let cols = cols_split_box(line, dat.delim);
    if cols.cols.len() < dat.core_idx[0] as usize {
        std::process::abort();
    }
    let chr = cols.cols[dat.core_idx[0] as usize].clone();

    if cols.cols.len() < dat.core_idx[1] as usize {
        std::process::abort();
    }
    // strtod equivalent: parse the leading numeric prefix; abort if none.
    let beg_field = &cols.cols[dat.core_idx[1] as usize];
    let beg_prefix = {
        let mut k = 0;
        while k < beg_field.len()
            && (beg_field[k].is_ascii_digit()
                || matches!(beg_field[k], b'+' | b'-' | b'.' | b'e' | b'E'))
        {
            k += 1;
        }
        k
    };
    let mut beg = std::str::from_utf8(&beg_field[..beg_prefix])
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| std::process::abort()) as hts_pos_t;

    if cols.cols.len() < dat.core_idx[2] as usize {
        std::process::abort();
    }
    let end_field = &cols.cols[dat.core_idx[2] as usize];
    let end_prefix = {
        let mut k = 0;
        while k < end_field.len()
            && (end_field[k].is_ascii_digit()
                || matches!(end_field[k], b'+' | b'-' | b'.' | b'e' | b'E'))
        {
            k += 1;
        }
        k
    };
    let mut end = std::str::from_utf8(&end_field[..end_prefix])
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| std::process::abort()) as hts_pos_t;

    beg -= dat.coor_base[0] as hts_pos_t - 1;
    end -= dat.coor_base[1] as hts_pos_t - 1;

    if end < beg {
        std::mem::swap(&mut beg, &mut end);
    }

    Some(ParsedTab {
        cols,
        chr,
        beg,
        end,
    })
}

// original: cols_split (htslib/annot-tsv.c:187)
fn annot_tsv_c_187_cols_split(
    line: &[u8],
    cols: Option<Box<AnnotTsvCols>>,
    delim: u8,
) -> Box<AnnotTsvCols> {
    let mut cols = cols.unwrap_or_default();
    cols_split_into(line, &mut cols, delim);
    cols
}

// original: cols_append (htslib/annot-tsv.c:217)
fn annot_tsv_c_217_cols_append(cols: &mut AnnotTsvCols, str_: &[u8]) {
    cols_append(cols, str_);
}

// original: cols_clear (htslib/annot-tsv.c:261)
fn annot_tsv_c_261_cols_clear(cols: &mut AnnotTsvCols) {
    cols_clear(cols);
}

// original: cols_destroy (htslib/annot-tsv.c:269)
fn annot_tsv_c_269_cols_destroy(cols: Option<Box<AnnotTsvCols>>) {
    drop(cols);
}

// original: parse_tab_with_payload (htslib/annot-tsv.c:276)
//
// regidx callback. The `usr` byte buffer carries the source `AnnotTsvDat`
// parse config (delim + the three core column indices + the two coordinate
// bases) serialized as: 1 delim byte followed by five little-endian i32s. The
// `payload` slot stores a raw `*mut AnnotTsvCols` pointer (the split line),
// which `annot_tsv_c_322_free_payload` later reclaims. This pointer-in-payload
// scheme is the genuine boundary with regidx's opaque payload machinery.
fn annot_tsv_c_276_parse_tab_with_payload(
    line: &[u8],
    out: &mut regidx::ParsedRegion,
    payload: &mut [u8],
    usr: Option<&mut Vec<u8>>,
) -> i32 {
    if line.first() == Some(&b'#') {
        return -1;
    }
    let cfg = usr.expect("annot-tsv regidx usr config");
    let delim = cfg[0];
    let core0 = i32::from_le_bytes(cfg[1..5].try_into().unwrap());
    let core1 = i32::from_le_bytes(cfg[5..9].try_into().unwrap());
    let core2 = i32::from_le_bytes(cfg[9..13].try_into().unwrap());
    let coor0 = i32::from_le_bytes(cfg[13..17].try_into().unwrap());
    let coor1 = i32::from_le_bytes(cfg[17..21].try_into().unwrap());

    let cols = cols_split_box(line, delim);
    let n = cols.cols.len() as i32;
    if n < core0 || n < core1 || n < core2 {
        std::process::abort();
    }

    // Locate the byte span of the chromosome (core0-th field) within `line`.
    let mut field = 0;
    let mut start = 0;
    let mut chr_range = None;
    let mut idx = 0;
    loop {
        let is_delim = idx == line.len() || line[idx] == delim;
        if is_delim {
            if field == core0 {
                chr_range = Some(start..=idx.saturating_sub(1).max(start));
                break;
            }
            field += 1;
            start = idx + 1;
        }
        if idx == line.len() {
            break;
        }
        idx += 1;
    }
    out.chr = chr_range;

    // strtod equivalent on the beg/end fields.
    let parse_leading = |f: &[u8]| -> hts_pos_t {
        let mut k = 0;
        while k < f.len() && (f[k].is_ascii_digit() || matches!(f[k], b'+' | b'-' | b'.' | b'e' | b'E')) {
            k += 1;
        }
        std::str::from_utf8(&f[..k])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or_else(|| std::process::abort()) as hts_pos_t
    };
    let mut beg = parse_leading(&cols.cols[core1 as usize]) - (coor0 as hts_pos_t - 1);
    let mut end = parse_leading(&cols.cols[core2 as usize]) - (coor1 as hts_pos_t - 1);
    if end < beg {
        std::mem::swap(&mut beg, &mut end);
    }
    out.beg = beg;
    out.end = end;

    // Store the owned cols as a raw pointer in the payload slot.
    let ptr = Box::into_raw(cols);
    let bytes = (ptr as usize).to_ne_bytes();
    payload[..bytes.len()].copy_from_slice(&bytes);
    0
}

// original: free_payload (htslib/annot-tsv.c:322)
fn annot_tsv_c_322_free_payload(payload: &mut [u8]) {
    const N: usize = std::mem::size_of::<usize>();
    let mut bytes = [0u8; N];
    bytes.copy_from_slice(&payload[..N]);
    let ptr = usize::from_ne_bytes(bytes) as *mut AnnotTsvCols;
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}

unsafe fn parse_header(dat: &mut AnnotTsvDat, fname: &[u8], mut nth_row: i32, autodetect: i32) {
    // hts_open still takes a NUL-terminated path (genuine I/O boundary).
    let fname_c = std::ffi::CString::new(fname).unwrap();
    dat.fp = hts_open(fname_c.as_ptr(), c"r".as_ptr());
    if dat.fp.is_null() {
        std::process::abort();
    }

    let mut buf: Vec<Vec<u8>> = Vec::new();
    if nth_row < 0 {
        buf.reserve_exact((-nth_row) as usize);
    }

    let mut irow = 0;
    // hts_getline / kstring is still the external line buffer (I/O boundary);
    // its bytes are read as a slice without the NUL terminator.
    while hts_getline(dat.fp, 2, &mut dat.line) > 0 {
        let line = dat.line.data.clone();
        let line = line.as_slice();
        let first = line.first().copied();
        if autodetect != 0 {
            nth_row = if first == Some(b'#') { 1 } else { 0 };
            break;
        }
        if nth_row == 0 {
            if first == Some(b'#') {
                continue;
            }
            break;
        }
        if nth_row > 0 {
            irow += 1;
            if irow < nth_row {
                continue;
            }
            break;
        }
        if first != Some(b'#') {
            break;
        }
        if buf.len() == (-nth_row) as usize {
            buf.remove(0);
        }
        buf.push(line.to_vec());
    }

    let mut keep_line = 0;
    let mut cols = if nth_row < 0 {
        if buf.len() != (-nth_row) as usize {
            std::process::abort();
        }
        keep_line = 1;
        cols_split_box(&buf[0], dat.delim)
    } else {
        cols_split_box(&dat.line.data, dat.delim)
    };

    if dat.line.data.is_empty() || cols.cols.is_empty() {
        std::process::abort();
    }

    if nth_row == 0 {
        // Build a synthetic "1<delim>2<delim>..." header line.
        let mut synth = Vec::<u8>::new();
        for i in 0..cols.cols.len() {
            if i > 0 {
                synth.push(dat.delim);
            }
            synth.extend_from_slice((i + 1).to_string().as_bytes());
        }
        cols = cols_split_box(&synth, dat.delim);
        dat.hdr.dummy = 1;
        keep_line = 1;
    }

    dat.hdr.name2idx.clear();
    for i in 0..cols.cols.len() {
        // Trim leading '#'/whitespace, an optional "[<digits>]" index tag, then
        // leading '#'/whitespace again.
        let col = &cols.cols[i];
        let mut start = 0;
        while start < col.len() && (col[start] == b'#' || col[start].is_ascii_whitespace()) {
            start += 1;
        }
        if start == col.len() {
            std::process::abort();
        }
        if col[start] == b'[' {
            let mut se = start + 1;
            while se < col.len() && col[se].is_ascii_digit() {
                se += 1;
            }
            if se < col.len() && col[se] == b']' {
                start = se + 1;
            }
        }
        while start < col.len() && (col[start] == b'#' || col[start].is_ascii_whitespace()) {
            start += 1;
        }
        if start == col.len() {
            std::process::abort();
        }
        let trimmed = col[start..].to_vec();
        cols.cols[i] = trimmed;
        dat.hdr.name2idx.insert(cols.cols[i].clone(), i as i32);
    }
    dat.hdr.cols = Some(cols);
    if keep_line == 0 {
        dat.line.data.clear();
    }
}

// original: parse_header (htslib/annot-tsv.c:335)
pub unsafe fn annot_tsv_c_335_parse_header(
    dat: &mut AnnotTsvDat,
    fname: &[u8],
    nth_row: i32,
    autodetect: i32,
) {
    parse_header(dat, fname, nth_row, autodetect);
}

unsafe fn write_header(args: &mut AnnotTsvArgs, dat: &mut AnnotTsvDat) {
    if dat.hdr.dummy != 0 || args.no_write_hdr > 1 {
        return;
    }
    let cols = dat.hdr.cols.as_ref().expect("header cols must be parsed");
    let mut out = Vec::<u8>::new();
    out.push(b'#');
    for i in 0..cols.cols.len() {
        if i > 0 {
            out.push(dat.delim);
        }
        if args.no_write_hdr == 0 {
            out.push(b'[');
            out.extend_from_slice((i + 1).to_string().as_bytes());
            out.push(b']');
        }
        out.extend_from_slice(&cols.cols[i]);
    }
    if let Some(annots) = dat.hdr.annots.as_ref() {
        for col in &annots.cols {
            if out.len() > 1 {
                out.push(dat.delim);
            }
            out.extend_from_slice(col);
        }
    }
    out.push(b'\n');
    // bgzf_write is the genuine I/O boundary; hand it the slice ptr + len.
    if bgzf::bgzf_write(args.out_fp, out.as_ptr().cast(), out.len()) != out.len() as isize {
        std::process::abort();
    }
}

// original: write_header (htslib/annot-tsv.c:440)
pub unsafe fn annot_tsv_c_440_write_header(args: &mut AnnotTsvArgs, dat_is_src: bool) {
    if dat_is_src {
        let mut dat = std::mem::take(&mut args.src);
        write_header(args, &mut dat);
        args.src = dat;
    } else {
        let mut dat = std::mem::take(&mut args.dst);
        write_header(args, &mut dat);
        args.dst = dat;
    }
}

fn destroy_header(dat: &mut AnnotTsvDatHeaderOnly) {
    dat.hdr.cols = None;
    dat.hdr.name2idx.clear();
}

// original: destroy_header (htslib/annot-tsv.c:465)
fn annot_tsv_c_465_destroy_header(dat: &mut AnnotTsvDatHeaderOnly) {
    destroy_header(dat);
}

unsafe fn read_next_line(dat: &mut AnnotTsvDat) -> i32 {
    if !dat.line.data.is_empty() {
        return dat.line.data.len() as i32;
    }
    let ret = crate::htslib_rs::hts::hts_getline(dat.fp, 2, &mut dat.line);
    if ret > 0 {
        return dat.line.data.len() as i32;
    }
    if ret < -1 {
        std::process::abort();
    }
    0
}

// original: read_next_line (htslib/annot-tsv.c:471)
pub unsafe fn annot_tsv_c_471_read_next_line(dat: &mut AnnotTsvDat) -> i32 {
    read_next_line(dat)
}

fn sanity_check_columns(hdr: &AnnotTsvHdr, cols: &AnnotTsvCols, force: i32) -> Vec<i32> {
    let mut col2idx = vec![0; cols.cols.len()];
    for i in 0..cols.cols.len() {
        let idx = match hdr.name2idx.get(&cols.cols[i]) {
            Some(&v) => v,
            None => {
                if force == 0 {
                    std::process::abort();
                }
                -1
            }
        };
        col2idx[i] = idx;
    }
    col2idx
}

// original: sanity_check_columns (htslib/annot-tsv.c:480)
fn annot_tsv_c_480_sanity_check_columns(
    _fname: &[u8],
    hdr: &AnnotTsvHdr,
    cols: &AnnotTsvCols,
    force: i32,
) -> Vec<i32> {
    sanity_check_columns(hdr, cols, force)
}

fn parse_coor_base(str_: &[u8], dat: &mut AnnotTsvDat) {
    let fname = &dat.fname;
    let mut beg = 1;
    let mut end = 1;
    if !str_.is_empty() {
        beg = match str_[0] {
            b'0' => 0,
            b'1' => 1,
            _ => std::process::abort(),
        };
        end = match str_.get(1) {
            Some(b'0') => 0,
            Some(b'1') => 1,
            _ => std::process::abort(),
        };
    } else if fname.len() >= 4 && fname[fname.len() - 4..].eq_ignore_ascii_case(b".bed")
        || fname.len() >= 7 && fname[fname.len() - 7..].eq_ignore_ascii_case(b".bed.gz")
    {
        beg = 0;
    }
    dat.coor_base[0] = beg;
    dat.coor_base[1] = end;
}

// original: parse_coor_base (htslib/annot-tsv.c:495)
fn annot_tsv_c_495_parse_coor_base(_args: &mut AnnotTsvArgs, str_: &[u8], dat: &mut AnnotTsvDat) {
    parse_coor_base(str_, dat);
}

// original: init_data (htslib/annot-tsv.c:515)
pub unsafe fn annot_tsv_c_515_init_data(args: &mut AnnotTsvArgs) {
    if args.delim_str.is_empty() {
        args.dst.delim = b'\t';
        args.src.delim = b'\t';
    } else if args.delim_str.len() == 1 {
        args.dst.delim = args.delim_str[0];
        args.src.delim = args.delim_str[0];
    } else if args.delim_str.len() == 3 && args.delim_str[1] == b':' {
        args.src.delim = args.delim_str[0];
        args.dst.delim = args.delim_str[2];
    } else {
        std::process::abort();
    }

    let mut isrc = 0;
    let mut idst = 0;
    let mut autodetect = 1;
    if !args.headers_str.is_empty() {
        let tmp = cols_split_box(&args.headers_str, b':');
        // strtol equivalent: parse each side as a base-10 integer; abort on any
        // trailing/garbage characters.
        isrc = std::str::from_utf8(&tmp.cols[0])
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or_else(|| std::process::abort());
        let dst_str = if tmp.cols.len() == 2 {
            &tmp.cols[1]
        } else {
            &tmp.cols[0]
        };
        idst = std::str::from_utf8(dst_str)
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or_else(|| std::process::abort());
        autodetect = 0;
    }
    let dst_fname = args.dst.fname.clone();
    let src_fname = args.src.fname.clone();
    parse_header(&mut args.dst, &dst_fname, idst, autodetect);
    parse_header(&mut args.src, &src_fname, isrc, autodetect);

    if args.core_str.is_empty() {
        args.core_str = b"chr,beg,end:chr,beg,end".to_vec();
    }
    let tmp = cols_split_box(&args.core_str, b':');
    args.src.core = Some(cols_split_box(&tmp.cols[0], b','));
    args.dst.core = Some(cols_split_box(
        if tmp.cols.len() == 2 {
            &tmp.cols[1]
        } else {
            &tmp.cols[0]
        },
        b',',
    ));
    args.src.core_idx = sanity_check_columns(&args.src.hdr, args.src.core.as_ref().unwrap(), 0);
    args.dst.core_idx = sanity_check_columns(&args.dst.hdr, args.dst.core.as_ref().unwrap(), 0);
    if args.src.core.as_ref().unwrap().cols.len() != 3
        || args.dst.core.as_ref().unwrap().cols.len() != 3
    {
        std::process::abort();
    }

    if args.coords_str.is_empty() {
        args.coords_str = b":".to_vec();
    }
    let tmp = cols_split_box(&args.coords_str, b':');
    let coords0 = tmp.cols[0].clone();
    let coords1 = if tmp.cols.len() == 2 {
        tmp.cols[1].clone()
    } else {
        tmp.cols[0].clone()
    };
    parse_coor_base(&coords0, &mut args.src);
    parse_coor_base(&coords1, &mut args.dst);

    if !args.match_str.is_empty() {
        let tmp = cols_split_box(&args.match_str, b':');
        args.src.match_ = Some(cols_split_box(&tmp.cols[0], b','));
        args.dst.match_ = Some(cols_split_box(
            if tmp.cols.len() == 2 {
                &tmp.cols[1]
            } else {
                &tmp.cols[0]
            },
            b',',
        ));
        args.src.match_idx =
            sanity_check_columns(&args.src.hdr, args.src.match_.as_ref().unwrap(), 0);
        args.dst.match_idx =
            sanity_check_columns(&args.dst.hdr, args.dst.match_.as_ref().unwrap(), 0);
        if args.src.match_.as_ref().unwrap().cols.len()
            != args.dst.match_.as_ref().unwrap().cols.len()
        {
            std::process::abort();
        }
    }

    if !args.transfer_str.is_empty() {
        let tmp = cols_split_box(&args.transfer_str, b':');
        args.src.transfer = Some(cols_split_box(&tmp.cols[0], b','));
        args.dst.transfer = Some(cols_split_box(
            if tmp.cols.len() == 2 {
                &tmp.cols[1]
            } else {
                &tmp.cols[0]
            },
            b',',
        ));
        args.src.transfer_idx =
            sanity_check_columns(&args.src.hdr, args.src.transfer.as_ref().unwrap(), 1);
        args.dst.transfer_idx =
            sanity_check_columns(&args.dst.hdr, args.dst.transfer.as_ref().unwrap(), 1);
        let n_transfer = args.src.transfer.as_ref().unwrap().cols.len();
        if n_transfer != args.dst.transfer.as_ref().unwrap().cols.len() {
            std::process::abort();
        }
        for i in 0..n_transfer {
            if args.src.transfer_idx[i] == -1 {
                let name = args.src.transfer.as_ref().unwrap().cols[i].clone();
                let hdr_cols = args.src.hdr.cols.as_mut().unwrap();
                cols_append(hdr_cols, &name);
                args.src.transfer_idx[i] = -(hdr_cols.cols.len() as i32);
                args.src.grow_n += 1;
            }
        }
        for i in 0..n_transfer {
            if args.dst.transfer_idx[i] == -1 {
                let name = args.dst.transfer.as_ref().unwrap().cols[i].clone();
                let hdr_cols = args.dst.hdr.cols.as_mut().unwrap();
                cols_append(hdr_cols, &name);
                args.dst.transfer_idx[i] = hdr_cols.cols.len() as i32 - 1;
                args.dst.grow_n += 1;
            }
        }
        args.tmp_cols = (0..n_transfer).map(|_| AnnotTsvCols::default()).collect();
        args.tmp_hash = (0..n_transfer)
            .map(|_| std::collections::HashSet::new())
            .collect();
    } else {
        args.src.transfer = Some(Box::<AnnotTsvCols>::default());
    }
    args.src.nannots_added = vec![0; args.src.transfer.as_ref().unwrap().cols.len()];

    if !args.annots_str.is_empty() {
        let tmp = cols_split_box(&args.annots_str, b':');
        args.src.annots = Some(cols_split_box(&tmp.cols[0], b','));
        args.dst.annots = Some(cols_split_box(
            if tmp.cols.len() == 2 {
                &tmp.cols[1]
            } else {
                &tmp.cols[0]
            },
            b',',
        ));
        let n_annots = args.src.annots.as_ref().unwrap().cols.len();
        if n_annots != args.dst.annots.as_ref().unwrap().cols.len() {
            std::process::abort();
        }
        args.dst.annots_idx = vec![0; n_annots];
        let two_sided = tmp.cols.len() == 2;
        for i in 0..n_annots {
            let src = args.src.annots.as_ref().unwrap().cols[i].clone();
            let dst_name = args.dst.annots.as_ref().unwrap().cols[i].clone();
            let (flag, default): (i32, &[u8]) = if src.eq_ignore_ascii_case(b"nbp") {
                (ANNOT_TSV_ANN_NBP, b"nbp")
            } else if src.eq_ignore_ascii_case(b"frac") {
                (ANNOT_TSV_ANN_FRAC, b"frac")
            } else if src.eq_ignore_ascii_case(b"cnt") {
                (ANNOT_TSV_ANN_CNT, b"cnt")
            } else {
                std::process::abort();
            };
            args.dst.annots_idx[i] = flag;
            let name = if two_sided { dst_name } else { default.to_vec() };
            cols_append(args.dst.hdr.cols.as_mut().unwrap(), &name);
        }
        args.nbp = Some(Box::<AnnotTsvNbp>::default());
    }

    // regidx owns the source index. The parse callback needs the source
    // parse-config, which is serialized into the `usr` byte buffer as
    // delim + core_idx[0..3] + coor_base[0..2] (1 byte + five LE i32s).
    let mut usr_cfg = Vec::with_capacity(1 + 5 * 4);
    usr_cfg.push(args.src.delim);
    usr_cfg.extend_from_slice(&args.src.core_idx[0].to_le_bytes());
    usr_cfg.extend_from_slice(&args.src.core_idx[1].to_le_bytes());
    usr_cfg.extend_from_slice(&args.src.core_idx[2].to_le_bytes());
    usr_cfg.extend_from_slice(&args.src.coor_base[0].to_le_bytes());
    usr_cfg.extend_from_slice(&args.src.coor_base[1].to_le_bytes());
    let mut idx = regidx::regidx_c_246_regidx_init(
        None,
        Some(annot_tsv_c_276_parse_tab_with_payload),
        Some(annot_tsv_c_322_free_payload),
        std::mem::size_of::<*mut AnnotTsvCols>(),
        Some(usr_cfg),
    )
    .expect("regidx_init");
    while read_next_line(&mut args.src) != 0 {
        let line = args.src.line.data.clone();
        if regidx::regidx_c_198_regidx_insert(&mut idx, &line) != 0 {
            std::process::abort();
        }
        args.src.line.data.clear();
    }
    args.itr = Some(regidx::regidx_c_584_regitr_init(&mut idx));
    args.idx = Some(idx);
    if hts_close(args.src.fp) != 0 {
        std::process::abort();
    }

    args.out_fp = if !args.out_fname.is_empty() {
        let name = &args.out_fname;
        let compress_output = (name.len() >= 3
            && name[name.len() - 3..].eq_ignore_ascii_case(b".gz"))
            || (name.len() >= 4 && name[name.len() - 4..].eq_ignore_ascii_case(b".bgz"));
        let path = std::ffi::CString::new(name.as_slice()).unwrap();
        bgzf::bgzf_open(
            path.as_ptr().cast(),
            if compress_output {
                c"wg".as_ptr().cast()
            } else {
                c"wu".as_ptr().cast()
            },
        )
    } else {
        bgzf::bgzf_open(c"-".as_ptr().cast(), c"wu".as_ptr().cast())
    };
    if args.out_fp.is_null() {
        std::process::abort();
    }
}

// original: destroy_data (htslib/annot-tsv.c:666)
pub unsafe fn annot_tsv_c_666_destroy_data(args: &mut AnnotTsvArgs) {
    if crate::htslib_rs::bgzf::bgzf_close(args.out_fp) != 0 {
        std::process::abort();
    }
    if crate::htslib_rs::hts::hts_close(args.dst.fp) != 0 {
        std::process::abort();
    }
    // All owned column/hash data is dropped simply by clearing/replacing the
    // owning fields; no manual element-by-element free is needed.
    args.tmp_hash.clear();
    args.tmp_cols.clear();
    args.src.core = None;
    args.dst.core = None;
    args.src.match_ = None;
    args.dst.match_ = None;
    args.src.transfer = None;
    args.dst.transfer = None;
    args.src.annots = None;
    args.dst.annots = None;
    args.nbp = None;
    args.src.hdr.cols = None;
    args.src.hdr.name2idx.clear();
    args.dst.hdr.cols = None;
    args.dst.hdr.name2idx.clear();
    args.src.nannots_added.clear();
    args.src.core_idx.clear();
    args.dst.core_idx.clear();
    args.src.match_idx.clear();
    args.dst.match_idx.clear();
    args.src.transfer_idx.clear();
    args.dst.transfer_idx.clear();
    args.dst.annots_idx.clear();
    // The kstring line buffers now own their bytes directly; releasing the
    // Vec frees them.
    args.src.line.data = Vec::new();
    args.dst.line.data = Vec::new();
    if let Some(itr) = args.itr.take() {
        crate::htslib_rs::regidx::regidx_c_606_regitr_destroy(itr);
    }
    if let Some(idx) = args.idx.take() {
        crate::htslib_rs::regidx::regidx_c_311_regidx_destroy(idx);
    }
    args.tmp_kstr.data = Vec::new();
}

unsafe fn write_string(args: &mut AnnotTsvArgs, str_: &[u8]) {
    // An empty field is written as a single "." placeholder.
    let buf: &[u8] = if str_.is_empty() { b"." } else { str_ };
    if crate::htslib_rs::bgzf::bgzf_write(args.out_fp, buf.as_ptr().cast(), buf.len())
        != buf.len() as isize
    {
        std::process::abort();
    }
}

// original: write_string (htslib/annot-tsv.c:703)
pub unsafe fn annot_tsv_c_703_write_string(args: &mut AnnotTsvArgs, str_: &[u8]) {
    write_string(args, str_);
}

unsafe fn write_annots(args: &mut AnnotTsvArgs) {
    let Some(annots) = args.dst.annots.as_ref() else {
        return;
    };
    let n_annots = annots.cols.len();

    let (len, frac_denominator, cnt) = {
        let nbp = args.nbp.as_mut().expect("annotations require nbp state");
        (
            nbp_length(nbp),
            (nbp.end - nbp.beg + 1) as f64,
            (nbp.regs.len() / 2) as i32,
        )
    };
    let mut out = Vec::<u8>::new();
    for i in 0..n_annots {
        out.push(args.dst.delim);
        let ann = args.dst.annots_idx[i];
        if ann == ANNOT_TSV_ANN_NBP {
            out.extend_from_slice(len.to_string().as_bytes());
        } else if ann == ANNOT_TSV_ANN_FRAC {
            // C uses kputd (a custom %g-style formatter that culls trailing
            // zeros, htslib/annot-tsv.c:727), not a fixed-precision %f.
            let mut frac_kstr = kstring_t { data: Vec::new() };
            crate::htslib_rs::kstring::kputd(len as f64 / frac_denominator, &mut frac_kstr);
            out.extend_from_slice(&frac_kstr.data);
        } else if ann == ANNOT_TSV_ANN_CNT {
            out.extend_from_slice(cnt.to_string().as_bytes());
        }
    }
    write_string(args, &out);
}

// original: write_annots (htslib/annot-tsv.c:709)
pub unsafe fn annot_tsv_c_709_write_annots(args: &mut AnnotTsvArgs) {
    write_annots(args);
}

// original: process_line (htslib/annot-tsv.c:737)
pub unsafe fn annot_tsv_c_737_process_line(args: &mut AnnotTsvArgs, line: &[u8]) {
    let Some(parsed) = annot_tsv_parse_tab_with_payload(line, &mut args.dst) else {
        return;
    };
    let chr = parsed.chr.clone();
    let beg = parsed.beg;
    let end = parsed.end;
    let mut dst_cols = parsed.cols;

    if let Some(nbp) = args.nbp.as_mut() {
        nbp_reset(nbp, beg, end);
    }

    let overlap = {
        let idx = args.idx.as_mut().expect("regidx index");
        let itr = args.itr.as_mut().expect("regitr");
        regidx::regidx_c_401_regidx_overlap(idx, &chr, beg, end, Some(itr))
    };
    if overlap == 0 {
        if args.mode & ANNOT_TSV_PRINT_NONMATCHING != 0 {
            write_string(args, line);
            write_annots(args);
            write_string(args, b"\n");
        }
        return;
    }

    let n_transfer = args.src.transfer.as_ref().unwrap().cols.len();
    for i in 0..n_transfer {
        args.src.nannots_added[i] = 0;
        args.tmp_cols[i].cols.clear();
        args.tmp_hash[i].clear();
    }

    let mut has_match = 0;
    loop {
        let (itr_beg, itr_end, src_cols) = {
            let itr = args.itr.as_mut().expect("regitr");
            if regidx::regidx_c_612_regitr_overlap(itr) == 0 {
                break;
            }
            // The regitr payload carries the raw `*mut AnnotTsvCols` pointer bytes
            // that the parse callback stored; reconstruct the borrow here.
            const N: usize = std::mem::size_of::<usize>();
            let mut bytes = [0u8; N];
            bytes.copy_from_slice(&itr.payload[..N]);
            let ptr = usize::from_ne_bytes(bytes) as *const AnnotTsvCols;
            (itr.beg, itr.end, &*ptr)
        };
        if args.overlap_src != 0.0 || args.overlap_dst != 0.0 {
            let len_dst = (end - beg + 1) as f64;
            let len_src = (itr_end - itr_beg + 1) as f64;
            let isec = (itr_end.min(end) - itr_beg.max(beg) + 1) as f64;
            let pass_dst = (isec / len_dst >= args.overlap_dst) as i32;
            let pass_src = (isec / len_src >= args.overlap_src) as i32;
            if args.overlap_either != 0 {
                if pass_dst == 0 && pass_src == 0 {
                    continue;
                }
            } else if pass_dst == 0 || pass_src == 0 {
                continue;
            }
        }
        if let Some(dst_match) = args.dst.match_.as_ref() {
            if !dst_match.cols.is_empty() {
                let mut i = 0usize;
                while i < dst_match.cols.len() {
                    if args.dst.match_idx[i] > dst_cols.cols.len() as i32 {
                        std::process::abort();
                    }
                    let dst = &dst_cols.cols[args.dst.match_idx[i] as usize];
                    let src = &src_cols.cols[args.src.match_idx[i] as usize];
                    if dst != src {
                        break;
                    }
                    i += 1;
                }
                if i != dst_match.cols.len() {
                    continue;
                }
            }
        }
        has_match = 1;

        if let Some(nbp) = args.nbp.as_mut() {
            nbp_add(nbp, itr_beg.max(beg), itr_end.min(end));
        }

        let mut max_annots_reached = 0;
        for i in 0..n_transfer {
            let idx = args.src.transfer_idx[i];
            let value: Vec<u8> = if idx >= 0 {
                src_cols.cols[idx as usize].clone()
            } else {
                args.src.hdr.cols.as_ref().unwrap().cols[(-idx - 1) as usize].clone()
            };
            let str_: Vec<u8> = if value.is_empty() { b".".to_vec() } else { value };
            if args.allow_dups == 0 {
                if args.tmp_hash[i].contains(&str_) {
                    continue;
                }
                args.tmp_hash[i].insert(str_.clone());
            }
            if args.max_annots != 0 {
                args.src.nannots_added[i] += 1;
                if args.src.nannots_added[i] >= args.max_annots {
                    max_annots_reached = 1;
                }
            }
            cols_append(&mut args.tmp_cols[i], &str_);
        }
        if max_annots_reached != 0 {
            break;
        }
    }

    if has_match == 0 {
        if args.mode & ANNOT_TSV_PRINT_NONMATCHING != 0 {
            write_string(args, line);
            write_annots(args);
            write_string(args, b"\n");
        }
        return;
    }
    if args.mode & ANNOT_TSV_PRINT_MATCHING == 0 {
        return;
    }

    // Build the joined transfer values directly into the destination columns,
    // replacing the C trick of pointing column offsets into a scratch kstring.
    for i in 0..n_transfer {
        let ann = &args.tmp_cols[i];
        let joined: Vec<u8> = if ann.cols.is_empty() {
            b".".to_vec()
        } else {
            ann.cols.join(&b","[..])
        };
        let target = args.dst.transfer_idx[i] as usize;
        dst_cols.cols[target] = joined;
    }
    let delim = [args.dst.delim];
    write_string(args, &dst_cols.cols[0]);
    for i in 1..dst_cols.cols.len() {
        write_string(args, &delim);
        write_string(args, &dst_cols.cols[i]);
    }
    write_annots(args);
    write_string(args, b"\n");
}

// original: usage_text (htslib/annot-tsv.c:880)
pub fn annot_tsv_c_880_usage_text() -> &'static [u8] {
    // Strip the trailing NUL terminator that the C original carried.
    let text = ANNOT_TSV_USAGE_TEXT;
    match text.last() {
        Some(0) => &text[..text.len() - 1],
        _ => text,
    }
}

// original: main (htslib/annot-tsv.c:956)
pub unsafe fn annot_tsv_c_956_main(argc: i32, argv: *mut *mut i8) -> i32 {
    let args = Box::into_raw(Box::<AnnotTsvArgs>::default());
    let mut reciprocal = 0;
    let argv_slice = std::slice::from_raw_parts(argv, argc as usize);
    let mut i = 1usize;
    while i < argv_slice.len() {
        // argv entries are OS-provided C strings; copy each to an owned slice.
        let arg = CStr::from_ptr(argv_slice[i]).to_bytes().to_vec();
        let mut optarg: Option<Vec<u8>> = None;
        let c = if let Some(long) = arg.strip_prefix(b"--") {
            let (name, value): (&[u8], Option<Vec<u8>>) =
                match long.iter().position(|&ch| ch == b'=') {
                    Some(eq) => (&long[..eq], Some(long[eq + 1..].to_vec())),
                    None => (long, None),
                };
            match name {
                b"allow-dups" => 0,
                b"version" => 1,
                b"max-annots" => {
                    optarg = Some(value.unwrap_or_else(|| {
                        i += 1;
                        if i >= argv_slice.len() {
                            std::process::abort();
                        }
                        CStr::from_ptr(argv_slice[i]).to_bytes().to_vec()
                    }));
                    2
                }
                b"help" => 4,
                b"core" | b"coords" | b"transfer" | b"match" | b"output" | b"source-file"
                | b"target-file" | b"annotate" | b"headers" | b"overlap" | b"delim" => {
                    optarg = Some(value.unwrap_or_else(|| {
                        i += 1;
                        if i >= argv_slice.len() {
                            std::process::abort();
                        }
                        CStr::from_ptr(argv_slice[i]).to_bytes().to_vec()
                    }));
                    match name {
                        b"core" => b'c' as i32,
                        b"coords" => b'C' as i32,
                        b"transfer" => b'f' as i32,
                        b"match" => b'm' as i32,
                        b"output" => b'o' as i32,
                        b"source-file" => b's' as i32,
                        b"target-file" => b't' as i32,
                        b"annotate" => b'a' as i32,
                        b"headers" => b'h' as i32,
                        b"overlap" => b'O' as i32,
                        b"delim" => b'd' as i32,
                        _ => std::process::abort(),
                    }
                }
                b"no-header-idx" => b'I' as i32,
                b"ignore-headers" => b'H' as i32,
                b"reciprocal" => b'r' as i32,
                b"drop-overlaps" => b'x' as i32,
                _ => std::process::abort(),
            }
        } else if arg.starts_with(b"-") && arg.len() > 1 {
            let mut pos = 1usize;
            let mut parsed = 0;
            while pos < arg.len() {
                let opt = arg[pos];
                match opt {
                    b'I' => (*args).no_write_hdr += 1,
                    b'H' => (*args).headers_str = b"0:0".to_vec(),
                    b'r' => reciprocal = 1,
                    b'x' => (*args).mode = ANNOT_TSV_PRINT_NONMATCHING,
                    b'c' | b'C' | b'f' | b'm' | b'o' | b's' | b't' | b'a' | b'O' | b'h' | b'd' => {
                        optarg = Some(if pos + 1 < arg.len() {
                            arg[pos + 1..].to_vec()
                        } else {
                            i += 1;
                            if i >= argv_slice.len() {
                                std::process::abort();
                            }
                            CStr::from_ptr(argv_slice[i]).to_bytes().to_vec()
                        });
                        parsed = opt as i32;
                        break;
                    }
                    _ => std::process::abort(),
                }
                pos += 1;
            }
            if parsed == 0 {
                i += 1;
                continue;
            }
            parsed
        } else {
            std::process::abort();
        };

        match c {
            0 => (*args).allow_dups = 1,
            1 => return 0,
            2 => {
                // strtod-as-int: parse the whole argument as an integer.
                let optarg = optarg.take().unwrap();
                (*args).max_annots = std::str::from_utf8(&optarg)
                    .ok()
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .filter(|_| std::str::from_utf8(&optarg).map(|s| !s.is_empty()).unwrap_or(false))
                    .map(|v| v as i32)
                    .unwrap_or_else(|| std::process::abort());
            }
            x if x == b'I' as i32 => (*args).no_write_hdr += 1,
            x if x == b'd' as i32 => (*args).delim_str = optarg.take().unwrap(),
            x if x == b'h' as i32 => (*args).headers_str = optarg.take().unwrap(),
            x if x == b'H' as i32 => (*args).headers_str = b"0:0".to_vec(),
            x if x == b'r' as i32 => reciprocal = 1,
            x if x == b'c' as i32 => (*args).core_str = optarg.take().unwrap(),
            x if x == b'C' as i32 => (*args).coords_str = optarg.take().unwrap(),
            x if x == b't' as i32 => (*args).dst.fname = optarg.take().unwrap(),
            x if x == b'm' as i32 => (*args).match_str = optarg.take().unwrap(),
            x if x == b'a' as i32 => (*args).annots_str = optarg.take().unwrap(),
            x if x == b'o' as i32 => (*args).out_fname = optarg.take().unwrap(),
            x if x == b'O' as i32 => {
                let optarg = optarg.take().unwrap();
                // Parse "FLOAT[,FLOAT]" overlap requirement.
                let comma = optarg.iter().position(|&ch| ch == b',');
                let src_part = &optarg[..comma.unwrap_or(optarg.len())];
                (*args).overlap_src = std::str::from_utf8(src_part)
                    .ok()
                    .filter(|s| !s.is_empty())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or_else(|| std::process::abort());
                if (*args).overlap_src < 0.0 || (*args).overlap_src > 1.0 {
                    std::process::abort();
                }
                if let Some(comma) = comma {
                    let dst_part = &optarg[comma + 1..];
                    (*args).overlap_dst = std::str::from_utf8(dst_part)
                        .ok()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or_else(|| std::process::abort());
                    if (*args).overlap_dst < 0.0 || (*args).overlap_dst > 1.0 {
                        std::process::abort();
                    }
                } else {
                    (*args).overlap_either = 1;
                }
            }
            x if x == b's' as i32 => (*args).src.fname = optarg.take().unwrap(),
            x if x == b'f' as i32 => (*args).transfer_str = optarg.take().unwrap(),
            x if x == b'x' as i32 => (*args).mode = ANNOT_TSV_PRINT_NONMATCHING,
            4 => return 0,
            _ => std::process::abort(),
        }
        i += 1;
    }
    if argc == 1 {
        std::process::abort();
    }
    if (*args).dst.fname.is_empty() && (*args).src.fname.is_empty() {
        std::process::abort();
    }
    if (*args).dst.fname.is_empty() {
        (*args).dst.fname = b"-".to_vec();
    }
    if (*args).src.fname.is_empty() {
        (*args).src.fname = b"-".to_vec();
    }
    if (*args).mode == 0 {
        (*args).mode = if (*args).transfer_str.is_empty() && (*args).annots_str.is_empty() {
            ANNOT_TSV_PRINT_MATCHING
        } else {
            ANNOT_TSV_PRINT_MATCHING | ANNOT_TSV_PRINT_NONMATCHING
        };
    }
    if (!(*args).transfer_str.is_empty() || !(*args).annots_str.is_empty())
        && (*args).mode & ANNOT_TSV_PRINT_MATCHING == 0
    {
        std::process::abort();
    }
    if reciprocal != 0 {
        if (*args).overlap_dst != 0.0
            && (*args).overlap_src != 0.0
            && (*args).overlap_dst != (*args).overlap_src
        {
            std::process::abort();
        }
        if (*args).overlap_src == 0.0 {
            (*args).overlap_src = (*args).overlap_dst;
        } else {
            (*args).overlap_dst = (*args).overlap_src;
        }
        (*args).overlap_either = 0;
    }

    annot_tsv_c_515_init_data(&mut *args);
    {
        let mut dst = std::mem::take(&mut (*args).dst);
        write_header(&mut *args, &mut dst);
        (*args).dst = dst;
    }
    while read_next_line(&mut (*args).dst) != 0 {
        // Pad the line with "<delim>." for each newly created destination column.
        let mut padded = (*args).dst.line.data.clone();
        for _ in 0..(*args).dst.grow_n {
            padded.push((*args).dst.delim);
            padded.push(b'.');
        }
        annot_tsv_c_737_process_line(&mut *args, &padded);
        (*args).dst.line.data.clear();
    }
    annot_tsv_c_666_destroy_data(&mut *args);
    drop(Box::from_raw(args));
    0
}
