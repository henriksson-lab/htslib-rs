#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]

use crate::htslib_rs::{
    bgzf,
    hts::{
        htsFile, hts_close, hts_getline, hts_open, hts_pos_t, isdigit_c, isspace_c, kputc, kputs,
        kputw, ks_resize, kstring_t, BGZF,
    },
    regidx, sam,
};
use std::ffi::{c_void, CStr};

const ANNOT_TSV_ANN_NBP: i32 = 1;
const ANNOT_TSV_ANN_FRAC: i32 = 2;
const ANNOT_TSV_ANN_CNT: i32 = 4;
const ANNOT_TSV_PRINT_MATCHING: i32 = 1;
const ANNOT_TSV_PRINT_NONMATCHING: i32 = 2;

#[derive(Default)]
struct AnnotTsvOffsets(Vec<*mut i8>);

impl AnnotTsvOffsets {
    fn add(&mut self, i: usize) -> *mut *mut i8 {
        unsafe { self.0.as_mut_ptr().add(i) }
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn push(&mut self, ptr: *mut i8) {
        self.0.push(ptr);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> std::slice::Iter<'_, *mut i8> {
        self.0.iter()
    }

    fn get(&self, i: usize) -> *mut i8 {
        self.0[i]
    }

    fn set(&mut self, i: usize, ptr: *mut i8) {
        self.0[i] = ptr;
    }
}

#[derive(Default)]
struct AnnotTsvCols {
    n: usize,
    off: AnnotTsvOffsets,
    rmme: Option<Box<[i8]>>,
}

#[repr(C)]
struct AnnotTsvHdr {
    name2idx: *mut c_void,
    cols: *mut AnnotTsvCols,
    annots: *mut AnnotTsvCols,
    dummy: i32,
}

#[repr(C)]
struct AnnotTsvDatHeaderOnly {
    fname: *mut i8,
    hdr: AnnotTsvHdr,
}

#[repr(C)]
struct AnnotTsvDat {
    fname: *mut i8,
    hdr: AnnotTsvHdr,
    core: *mut AnnotTsvCols,
    match_: *mut AnnotTsvCols,
    transfer: *mut AnnotTsvCols,
    annots: *mut AnnotTsvCols,
    core_idx: Vec<i32>,
    match_idx: Vec<i32>,
    transfer_idx: Vec<i32>,
    annots_idx: Vec<i32>,
    nannots_added: Vec<i32>,
    coor_base: [i32; 2],
    delim: i8,
    grow_n: i32,
    line: kstring_t,
    fp: *mut htsFile,
}

impl Default for AnnotTsvDat {
    fn default() -> Self {
        Self {
            fname: std::ptr::null_mut(),
            hdr: AnnotTsvHdr {
                name2idx: std::ptr::null_mut(),
                cols: std::ptr::null_mut(),
                annots: std::ptr::null_mut(),
                dummy: 0,
            },
            core: std::ptr::null_mut(),
            match_: std::ptr::null_mut(),
            transfer: std::ptr::null_mut(),
            annots: std::ptr::null_mut(),
            core_idx: Vec::new(),
            match_idx: Vec::new(),
            transfer_idx: Vec::new(),
            annots_idx: Vec::new(),
            nannots_added: Vec::new(),
            coor_base: [0; 2],
            delim: 0,
            grow_n: 0,
            line: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
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
    chr_beg: *mut i8,
    chr_end: *mut i8,
    beg: hts_pos_t,
    end: hts_pos_t,
}

#[repr(C)]
struct AnnotTsvArgs {
    nbp: Option<Box<AnnotTsvNbp>>,
    dst: AnnotTsvDat,
    src: AnnotTsvDat,
    core_str: *mut i8,
    coords_str: *mut i8,
    match_str: *mut i8,
    transfer_str: *mut i8,
    annots_str: *mut i8,
    headers_str: *mut i8,
    delim_str: *mut i8,
    temp_dir: *mut i8,
    out_fname: *mut i8,
    out_fp: *mut BGZF,
    allow_dups: i32,
    max_annots: i32,
    mode: i32,
    no_write_hdr: i32,
    overlap_either: i32,
    overlap_src: f64,
    overlap_dst: f64,
    idx: *mut regidx::regidx_t,
    itr: *mut regidx::regitr_t,
    tmp_kstr: kstring_t,
    tmp_cols: *mut Vec<AnnotTsvCols>,
    tmp_hash: Vec<*mut c_void>,
}

impl Default for AnnotTsvArgs {
    fn default() -> Self {
        Self {
            nbp: None,
            dst: AnnotTsvDat::default(),
            src: AnnotTsvDat::default(),
            core_str: std::ptr::null_mut(),
            coords_str: std::ptr::null_mut(),
            match_str: std::ptr::null_mut(),
            transfer_str: std::ptr::null_mut(),
            annots_str: std::ptr::null_mut(),
            headers_str: std::ptr::null_mut(),
            delim_str: std::ptr::null_mut(),
            temp_dir: std::ptr::null_mut(),
            out_fname: std::ptr::null_mut(),
            out_fp: std::ptr::null_mut(),
            allow_dups: 0,
            max_annots: 0,
            mode: 0,
            no_write_hdr: 0,
            overlap_either: 0,
            overlap_src: 0.0,
            overlap_dst: 0.0,
            idx: std::ptr::null_mut(),
            itr: std::ptr::null_mut(),
            tmp_kstr: kstring_t {
                l: 0,
                m: 0,
                s: std::ptr::null_mut(),
            },
            tmp_cols: std::ptr::null_mut(),
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

unsafe fn cols_split_into(line: *const i8, cols: &mut AnnotTsvCols, delim: i8) {
    cols.n = 0;
    cols.off.clear();
    let bytes = CStr::from_ptr(line).to_bytes_with_nul();
    let mut rmme: Box<[i8]> = bytes
        .iter()
        .map(|&byte| byte as i8)
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let mut ss = rmme.as_mut_ptr();
    loop {
        let mut se = ss;
        while *se != 0 && *se != delim {
            se = se.add(1);
        }
        let tmp = *se;
        *se = 0;
        cols.off.push(ss);
        cols.n += 1;
        if tmp == 0 {
            break;
        }
        ss = se.add(1);
    }
    cols.rmme = Some(rmme);
}

unsafe fn cols_split_box(line: *const i8, delim: i8) -> Box<AnnotTsvCols> {
    let mut cols = Box::<AnnotTsvCols>::default();
    cols_split_into(line, &mut cols, delim);
    cols
}

unsafe fn cols_split_raw(line: *const i8, delim: i8) -> *mut AnnotTsvCols {
    Box::into_raw(cols_split_box(line, delim))
}

unsafe fn cols_append(cols: &mut AnnotTsvCols, str_: *mut i8) {
    if cols.rmme.is_some() {
        let mut bytes = Vec::<i8>::new();
        for &old in cols.off.iter().take(cols.n) {
            let old_bytes = CStr::from_ptr(old).to_bytes_with_nul();
            bytes.extend(old_bytes.iter().map(|&byte| byte as i8));
        }
        let str_bytes = CStr::from_ptr(str_).to_bytes_with_nul();
        bytes.extend(str_bytes.iter().map(|&byte| byte as i8));

        let mut rmme = bytes.into_boxed_slice();
        let mut off = AnnotTsvOffsets(Vec::with_capacity(cols.off.len() + 1));
        let mut ptr = rmme.as_mut_ptr();
        loop {
            off.push(ptr);
            while *ptr != 0 {
                ptr = ptr.add(1);
            }
            ptr = ptr.add(1);
            if ptr >= rmme.as_mut_ptr().add(rmme.len()) {
                break;
            }
        }
        cols.rmme = Some(rmme);
        cols.off = off;
        cols.n += 1;
        return;
    }
    if cols.n < cols.off.len() {
        cols.off.set(cols.n, str_);
    } else {
        cols.off.push(str_);
    }
    cols.n += 1;
}

unsafe fn cols_clear(cols: &mut AnnotTsvCols) {
    cols.rmme = None;
    cols.n = 0;
    cols.off.clear();
}

unsafe fn cols_destroy_ptr(cols: *mut AnnotTsvCols) {
    if !cols.is_null() {
        drop(Box::from_raw(cols));
    }
}

unsafe fn parse_tab_with_payload_ref(line: *const i8, dat: &mut AnnotTsvDat) -> Option<ParsedTab> {
    if *line == b'#' as i8 {
        return None;
    }

    let cols = cols_split_box(line, dat.delim);
    if cols.n < dat.core_idx[0] as usize {
        libc::abort();
    }
    let chr_beg = cols.off.get(dat.core_idx[0] as usize);
    let chr_end = chr_beg.add(libc::strlen(chr_beg) - 1);

    if cols.n < dat.core_idx[1] as usize {
        libc::abort();
    }
    let mut tmp: *mut i8 = std::ptr::null_mut();
    let mut ptr = cols.off.get(dat.core_idx[1] as usize);
    let mut beg = libc::strtod(ptr, &mut tmp) as hts_pos_t;
    if tmp == ptr {
        libc::abort();
    }

    if cols.n < dat.core_idx[2] as usize {
        libc::abort();
    }
    ptr = cols.off.get(dat.core_idx[2] as usize);
    let mut end = libc::strtod(ptr, &mut tmp) as hts_pos_t;
    if tmp == ptr {
        libc::abort();
    }

    beg -= dat.coor_base[0] as hts_pos_t - 1;
    end -= dat.coor_base[1] as hts_pos_t - 1;

    if end < beg {
        std::mem::swap(&mut beg, &mut end);
    }

    Some(ParsedTab {
        cols,
        chr_beg,
        chr_end,
        beg,
        end,
    })
}

// original: cols_split (htslib/annot-tsv.c:187)
pub unsafe fn annot_tsv_c_187_cols_split(
    line: *const i8,
    cols: *mut c_void,
    delim: i8,
) -> *mut c_void {
    let cols = if cols.is_null() {
        Box::into_raw(Box::<AnnotTsvCols>::default())
    } else {
        cols.cast::<AnnotTsvCols>()
    };
    if cols.is_null() {
        libc::abort();
    }
    cols_split_into(line, &mut *cols, delim);
    cols.cast()
}

// original: cols_append (htslib/annot-tsv.c:217)
pub unsafe fn annot_tsv_c_217_cols_append(cols: *mut c_void, str_: *mut i8) {
    let cols = cols.cast::<AnnotTsvCols>();
    cols_append(&mut *cols, str_);
}

// original: cols_clear (htslib/annot-tsv.c:261)
pub unsafe fn annot_tsv_c_261_cols_clear(cols: *mut c_void) {
    let cols = cols.cast::<AnnotTsvCols>();
    if cols.is_null() {
        return;
    }
    cols_clear(&mut *cols);
}

// original: cols_destroy (htslib/annot-tsv.c:269)
pub unsafe fn annot_tsv_c_269_cols_destroy(cols: *mut c_void) {
    cols_destroy_ptr(cols.cast::<AnnotTsvCols>());
}

// original: parse_tab_with_payload (htslib/annot-tsv.c:276)
pub unsafe extern "C" fn annot_tsv_c_276_parse_tab_with_payload(
    line: *const i8,
    chr_beg: *mut *mut i8,
    chr_end: *mut *mut i8,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    payload: *mut c_void,
    usr: *mut c_void,
) -> i32 {
    let Some(parsed) = parse_tab_with_payload_ref(line, &mut *usr.cast::<AnnotTsvDat>()) else {
        *(payload.cast::<*mut AnnotTsvCols>()) = std::ptr::null_mut();
        return -1;
    };
    *chr_beg = parsed.chr_beg;
    *chr_end = parsed.chr_end;
    *beg = parsed.beg;
    *end = parsed.end;
    *(payload.cast::<*mut AnnotTsvCols>()) = Box::into_raw(parsed.cols);

    0
}

// original: free_payload (htslib/annot-tsv.c:322)
pub unsafe extern "C" fn annot_tsv_c_322_free_payload(payload: *mut c_void) {
    let cols = *(payload.cast::<*mut AnnotTsvCols>());
    annot_tsv_c_269_cols_destroy(cols.cast());
}

unsafe fn parse_header(dat: &mut AnnotTsvDat, fname: *mut i8, mut nth_row: i32, autodetect: i32) {
    dat.fp = hts_open(fname, c"r".as_ptr());
    if dat.fp.is_null() {
        libc::abort();
    }

    let mut buf: Vec<Box<[i8]>> = Vec::new();
    if nth_row < 0 {
        buf.reserve_exact((-nth_row) as usize);
    }

    let mut irow = 0;
    while hts_getline(dat.fp, 2, &mut dat.line) > 0 {
        if autodetect != 0 {
            nth_row = if *dat.line.s == b'#' as i8 { 1 } else { 0 };
            break;
        }
        if nth_row == 0 {
            if *dat.line.s == b'#' as i8 {
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
        if *dat.line.s != b'#' as i8 {
            break;
        }
        if buf.len() == (-nth_row) as usize {
            drop(buf.remove(0));
        }
        let saved = CStr::from_ptr(dat.line.s)
            .to_bytes_with_nul()
            .iter()
            .map(|&byte| byte as i8)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        buf.push(saved);
    }

    let mut keep_line = 0;
    let mut cols = if nth_row < 0 {
        if buf.len() != (-nth_row) as usize {
            libc::abort();
        }
        keep_line = 1;
        cols_split_box(buf[0].as_ptr(), dat.delim)
    } else {
        cols_split_box(dat.line.s, dat.delim)
    };

    if dat.line.l == 0 || cols.n == 0 {
        libc::abort();
    }

    if nth_row == 0 {
        let mut str_: kstring_t = std::mem::zeroed();
        for i in 0..cols.n as i32 {
            if i > 0 {
                kputc(dat.delim as i32, &mut str_);
            }
            kputw(i + 1, &mut str_);
        }
        cols = cols_split_box(str_.s, dat.delim);
        libc::free(str_.s.cast());
        dat.hdr.dummy = 1;
        keep_line = 1;
    }

    dat.hdr.name2idx = sam::khash_str2int_init();
    for i in 0..cols.n as usize {
        let mut ss = cols.off.get(i);
        while *ss != 0 && (*ss == b'#' as i8 || isspace_c(*ss) != 0) {
            ss = ss.add(1);
        }
        if *ss == 0 {
            libc::abort();
        }
        if *ss == b'[' as i8 {
            let mut se = ss.add(1);
            while *se != 0 && isdigit_c(*se) != 0 {
                se = se.add(1);
            }
            if *se == b']' as i8 {
                ss = se.add(1);
            }
        }
        while *ss != 0 && (*ss == b'#' as i8 || isspace_c(*ss) != 0) {
            ss = ss.add(1);
        }
        if *ss == 0 {
            libc::abort();
        }
        cols.off.set(i, ss);
        sam::khash_str2int_set(dat.hdr.name2idx, ss, i as i32);
    }
    dat.hdr.cols = Box::into_raw(cols);
    if keep_line == 0 {
        dat.line.l = 0;
    }
}

// original: parse_header (htslib/annot-tsv.c:335)
pub unsafe fn annot_tsv_c_335_parse_header(
    dat: *mut c_void,
    fname: *mut i8,
    nth_row: i32,
    autodetect: i32,
) {
    parse_header(&mut *dat.cast::<AnnotTsvDat>(), fname, nth_row, autodetect);
}

unsafe fn write_header(args: &mut AnnotTsvArgs, dat: &mut AnnotTsvDat) {
    if dat.hdr.dummy != 0 || args.no_write_hdr > 1 {
        return;
    }
    let mut str_: kstring_t = std::mem::zeroed();
    kputc(b'#' as i32, &mut str_);
    for i in 0..(*dat.hdr.cols).n as usize {
        if i > 0 {
            kputc(dat.delim as i32, &mut str_);
        }
        if args.no_write_hdr == 0 {
            kputc(b'[' as i32, &mut str_);
            kputw(i as i32 + 1, &mut str_);
            kputc(b']' as i32, &mut str_);
        }
        kputs((*dat.hdr.cols).off.get(i), &mut str_);
    }
    if !dat.hdr.annots.is_null() {
        for i in 0..(*dat.hdr.annots).n as usize {
            if str_.l > 1 {
                kputc(dat.delim as i32, &mut str_);
            }
            kputs((*dat.hdr.annots).off.get(i), &mut str_);
        }
    }
    kputc(b'\n' as i32, &mut str_);
    if bgzf::bgzf_write(args.out_fp, str_.s.cast(), str_.l) != str_.l as isize {
        libc::abort();
    }
    libc::free(str_.s.cast());
}

// original: write_header (htslib/annot-tsv.c:440)
pub unsafe fn annot_tsv_c_440_write_header(args: *mut c_void, dat: *mut c_void) {
    write_header(
        &mut *args.cast::<AnnotTsvArgs>(),
        &mut *dat.cast::<AnnotTsvDat>(),
    );
}

unsafe fn destroy_header(dat: &mut AnnotTsvDatHeaderOnly) {
    if !dat.hdr.cols.is_null() {
        cols_destroy_ptr(dat.hdr.cols);
    }
    sam::khash_str2int_destroy(dat.hdr.name2idx);
}

// original: destroy_header (htslib/annot-tsv.c:465)
pub unsafe fn annot_tsv_c_465_destroy_header(dat: *mut c_void) {
    destroy_header(&mut *dat.cast::<AnnotTsvDatHeaderOnly>());
}

unsafe fn read_next_line(dat: &mut AnnotTsvDat) -> i32 {
    if dat.line.l != 0 {
        return dat.line.l as i32;
    }
    let ret = crate::htslib_rs::hts::hts_getline(dat.fp, 2, &mut dat.line);
    if ret > 0 {
        return dat.line.l as i32;
    }
    if ret < -1 {
        libc::abort();
    }
    0
}

// original: read_next_line (htslib/annot-tsv.c:471)
pub unsafe fn annot_tsv_c_471_read_next_line(dat: *mut c_void) -> i32 {
    read_next_line(&mut *dat.cast::<AnnotTsvDat>())
}

unsafe fn sanity_check_columns(hdr: &AnnotTsvHdr, cols: &AnnotTsvCols, force: i32) -> Vec<i32> {
    let mut col2idx = vec![0; cols.n];
    for i in 0..cols.n {
        let mut idx = 0;
        if sam::khash_str2int_get(hdr.name2idx, cols.off.get(i), &mut idx) < 0 {
            if force == 0 {
                libc::abort();
            }
            idx = -1;
        }
        col2idx[i] = idx;
    }
    col2idx
}

// original: sanity_check_columns (htslib/annot-tsv.c:480)
pub unsafe fn annot_tsv_c_480_sanity_check_columns(
    _fname: *mut i8,
    hdr: *mut c_void,
    cols: *mut c_void,
    col2idx: *mut *mut i32,
    force: i32,
) {
    let hdr = hdr.cast::<AnnotTsvHdr>();
    let cols = cols.cast::<AnnotTsvCols>();
    *col2idx = Box::into_raw(sanity_check_columns(&*hdr, &*cols, force).into_boxed_slice()).cast();
}

unsafe fn parse_coor_base(str_: *mut i8, dat: &mut AnnotTsvDat) {
    let len = libc::strlen(dat.fname);
    let mut beg = 1;
    let mut end = 1;
    if *str_ != 0 {
        if *str_ == b'0' as i8 {
            beg = 0;
        } else if *str_ == b'1' as i8 {
            beg = 1;
        } else {
            libc::abort();
        }

        if *str_.add(1) == b'0' as i8 {
            end = 0;
        } else if *str_.add(1) == b'1' as i8 {
            end = 1;
        } else {
            libc::abort();
        }
    } else if (len >= 4 && libc::strcasecmp(c".bed".as_ptr(), dat.fname.add(len - 4)) == 0)
        || (len >= 7 && libc::strcasecmp(c".bed.gz".as_ptr(), dat.fname.add(len - 7)) == 0)
    {
        beg = 0;
    }
    dat.coor_base[0] = beg;
    dat.coor_base[1] = end;
}

// original: parse_coor_base (htslib/annot-tsv.c:495)
pub unsafe fn annot_tsv_c_495_parse_coor_base(_args: *mut c_void, str_: *mut i8, dat: *mut c_void) {
    parse_coor_base(str_, &mut *dat.cast::<AnnotTsvDat>());
}

// original: init_data (htslib/annot-tsv.c:515)
pub unsafe fn annot_tsv_c_515_init_data(args: *mut c_void) {
    let args = args.cast::<AnnotTsvArgs>();
    if (*args).delim_str.is_null() {
        (*args).dst.delim = b'\t' as i8;
        (*args).src.delim = b'\t' as i8;
    } else if libc::strlen((*args).delim_str) == 1 {
        (*args).dst.delim = *(*args).delim_str;
        (*args).src.delim = *(*args).delim_str;
    } else if libc::strlen((*args).delim_str) == 3 && *(*args).delim_str.add(1) == b':' as i8 {
        (*args).src.delim = *(*args).delim_str;
        (*args).dst.delim = *(*args).delim_str.add(2);
    } else {
        libc::abort();
    }

    let mut isrc = 0;
    let mut idst = 0;
    let mut autodetect = 1;
    if !(*args).headers_str.is_null() {
        let tmp = cols_split_raw((*args).headers_str, b':' as i8);
        let mut rmme: *mut i8 = std::ptr::null_mut();
        isrc = libc::strtol(*(*tmp).off.add(0), &mut rmme, 10) as i32;
        if *rmme != 0 || *(*tmp).off.add(0) == rmme {
            libc::abort();
        }
        let dst_str = if (*tmp).n == 2 {
            *(*tmp).off.add(1)
        } else {
            *(*tmp).off.add(0)
        };
        idst = libc::strtol(dst_str, &mut rmme, 10) as i32;
        if *rmme != 0 || dst_str == rmme {
            libc::abort();
        }
        cols_destroy_ptr(tmp);
        autodetect = 0;
    }
    parse_header(&mut (*args).dst, (*args).dst.fname, idst, autodetect);
    parse_header(&mut (*args).src, (*args).src.fname, isrc, autodetect);

    if (*args).core_str.is_null() {
        (*args).core_str = c"chr,beg,end:chr,beg,end".as_ptr().cast_mut();
    }
    let mut tmp = cols_split_raw((*args).core_str, b':' as i8);
    (*args).src.core = cols_split_raw((*tmp).off.get(0), b',' as i8);
    (*args).dst.core = cols_split_raw(
        if (*tmp).n == 2 {
            (*tmp).off.get(1)
        } else {
            (*tmp).off.get(0)
        },
        b',' as i8,
    );
    (*args).src.core_idx = sanity_check_columns(&(*args).src.hdr, &*(*args).src.core, 0);
    (*args).dst.core_idx = sanity_check_columns(&(*args).dst.hdr, &*(*args).dst.core, 0);
    if (*(*args).src.core).n != 3 || (*(*args).dst.core).n != 3 {
        libc::abort();
    }
    cols_destroy_ptr(tmp);

    if (*args).coords_str.is_null() {
        (*args).coords_str = c":".as_ptr().cast_mut();
    }
    tmp = cols_split_raw((*args).coords_str, b':' as i8);
    parse_coor_base((*tmp).off.get(0), &mut (*args).src);
    parse_coor_base(
        if (*tmp).n == 2 {
            (*tmp).off.get(1)
        } else {
            (*tmp).off.get(0)
        },
        &mut (*args).dst,
    );
    cols_destroy_ptr(tmp);

    if !(*args).match_str.is_null() {
        tmp = cols_split_raw((*args).match_str, b':' as i8);
        (*args).src.match_ = cols_split_raw((*tmp).off.get(0), b',' as i8);
        (*args).dst.match_ = cols_split_raw(
            if (*tmp).n == 2 {
                (*tmp).off.get(1)
            } else {
                (*tmp).off.get(0)
            },
            b',' as i8,
        );
        (*args).src.match_idx = sanity_check_columns(&(*args).src.hdr, &*(*args).src.match_, 0);
        (*args).dst.match_idx = sanity_check_columns(&(*args).dst.hdr, &*(*args).dst.match_, 0);
        if (*(*args).src.match_).n != (*(*args).dst.match_).n {
            libc::abort();
        }
        cols_destroy_ptr(tmp);
    }

    if !(*args).transfer_str.is_null() {
        tmp = cols_split_raw((*args).transfer_str, b':' as i8);
        (*args).src.transfer = cols_split_raw((*tmp).off.get(0), b',' as i8);
        (*args).dst.transfer = cols_split_raw(
            if (*tmp).n == 2 {
                (*tmp).off.get(1)
            } else {
                (*tmp).off.get(0)
            },
            b',' as i8,
        );
        (*args).src.transfer_idx =
            sanity_check_columns(&(*args).src.hdr, &*(*args).src.transfer, 1);
        (*args).dst.transfer_idx =
            sanity_check_columns(&(*args).dst.hdr, &*(*args).dst.transfer, 1);
        if (*(*args).src.transfer).n != (*(*args).dst.transfer).n {
            libc::abort();
        }
        for i in 0..(*(*args).src.transfer).n as usize {
            if (&(*args).src.transfer_idx)[i] == -1 {
                cols_append(
                    &mut *(*args).src.hdr.cols,
                    (*(*args).src.transfer).off.get(i),
                );
                (&mut (*args).src.transfer_idx)[i] = -((*(*args).src.hdr.cols).n as i32);
                (*args).src.grow_n += 1;
            }
        }
        for i in 0..(*(*args).dst.transfer).n as usize {
            if (&(*args).dst.transfer_idx)[i] == -1 {
                cols_append(
                    &mut *(*args).dst.hdr.cols,
                    (*(*args).dst.transfer).off.get(i),
                );
                (&mut (*args).dst.transfer_idx)[i] = (*(*args).dst.hdr.cols).n as i32 - 1;
                (*args).dst.grow_n += 1;
            }
        }
        (*args).tmp_cols = Box::into_raw(Box::new(
            (0..(*(*args).src.transfer).n)
                .map(|_| AnnotTsvCols::default())
                .collect::<Vec<_>>(),
        ));
        (*args).tmp_hash = vec![std::ptr::null_mut(); (*(*args).src.transfer).n];
        for i in 0..(*(*args).src.transfer).n {
            (&mut (*args).tmp_hash)[i] = sam::khash_str2int_init();
        }
        cols_destroy_ptr(tmp);
    } else {
        (*args).src.transfer = Box::into_raw(Box::<AnnotTsvCols>::default());
    }
    (*args).src.nannots_added = vec![0; (*(*args).src.transfer).n];

    if !(*args).annots_str.is_null() {
        tmp = cols_split_raw((*args).annots_str, b':' as i8);
        (*args).src.annots = cols_split_raw((*tmp).off.get(0), b',' as i8);
        (*args).dst.annots = cols_split_raw(
            if (*tmp).n == 2 {
                (*tmp).off.get(1)
            } else {
                (*tmp).off.get(0)
            },
            b',' as i8,
        );
        if (*(*args).src.annots).n != (*(*args).dst.annots).n {
            libc::abort();
        }
        (*args).dst.annots_idx = vec![0; (*(*args).dst.annots).n];
        for i in 0..(*(*args).src.annots).n {
            let src = (*(*args).src.annots).off.get(i);
            if libc::strcasecmp(src, c"nbp".as_ptr()) == 0 {
                (&mut (*args).dst.annots_idx)[i] = ANNOT_TSV_ANN_NBP;
                cols_append(
                    &mut *(*args).dst.hdr.cols,
                    if (*tmp).n == 2 {
                        (*(*args).dst.annots).off.get(i)
                    } else {
                        c"nbp".as_ptr().cast_mut()
                    },
                );
            } else if libc::strcasecmp(src, c"frac".as_ptr()) == 0 {
                (&mut (*args).dst.annots_idx)[i] = ANNOT_TSV_ANN_FRAC;
                cols_append(
                    &mut *(*args).dst.hdr.cols,
                    if (*tmp).n == 2 {
                        (*(*args).dst.annots).off.get(i)
                    } else {
                        c"frac".as_ptr().cast_mut()
                    },
                );
            } else if libc::strcasecmp(src, c"cnt".as_ptr()) == 0 {
                (&mut (*args).dst.annots_idx)[i] = ANNOT_TSV_ANN_CNT;
                cols_append(
                    &mut *(*args).dst.hdr.cols,
                    if (*tmp).n == 2 {
                        (*(*args).dst.annots).off.get(i)
                    } else {
                        c"cnt".as_ptr().cast_mut()
                    },
                );
            } else {
                libc::abort();
            }
        }
        (*args).nbp = Some(Box::<AnnotTsvNbp>::default());
        cols_destroy_ptr(tmp);
    }

    (*args).idx = regidx::regidx_c_246_regidx_init(
        std::ptr::null(),
        Some(annot_tsv_c_276_parse_tab_with_payload),
        Some(annot_tsv_c_322_free_payload),
        std::mem::size_of::<AnnotTsvCols>(),
        (&mut (*args).src as *mut AnnotTsvDat).cast(),
    );
    while read_next_line(&mut (*args).src) != 0 {
        if regidx::regidx_c_198_regidx_insert((*args).idx, (*args).src.line.s) != 0 {
            libc::abort();
        }
        (*args).src.line.l = 0;
    }
    (*args).itr = regidx::regidx_c_584_regitr_init((*args).idx);
    if hts_close((*args).src.fp) != 0 {
        libc::abort();
    }

    let len = if !(*args).out_fname.is_null() {
        libc::strlen((*args).out_fname)
    } else {
        0
    };
    (*args).out_fp = if len != 0 {
        let compress_output = (len >= 3
            && libc::strcasecmp(c".gz".as_ptr(), (*args).out_fname.add(len - 3)) == 0)
            || (len >= 4
                && libc::strcasecmp(c".bgz".as_ptr(), (*args).out_fname.add(len - 4)) == 0);
        bgzf::bgzf_open(
            (*args).out_fname,
            if compress_output {
                c"wg".as_ptr()
            } else {
                c"wu".as_ptr()
            },
        )
    } else {
        bgzf::bgzf_open(c"-".as_ptr(), c"wu".as_ptr())
    };
    if (*args).out_fp.is_null() {
        libc::abort();
    }
}

// original: destroy_data (htslib/annot-tsv.c:666)
pub unsafe fn annot_tsv_c_666_destroy_data(args: *mut c_void) {
    let args = args.cast::<AnnotTsvArgs>();
    let src_core_n = if (*args).src.core.is_null() {
        0
    } else {
        (*(*args).src.core).n
    };
    let dst_core_n = if (*args).dst.core.is_null() {
        0
    } else {
        (*(*args).dst.core).n
    };
    let src_match_n = if (*args).src.match_.is_null() {
        0
    } else {
        (*(*args).src.match_).n
    };
    let dst_match_n = if (*args).dst.match_.is_null() {
        0
    } else {
        (*(*args).dst.match_).n
    };
    let src_transfer_n = if (*args).src.transfer.is_null() {
        0
    } else {
        (*(*args).src.transfer).n
    };
    let dst_transfer_n = if (*args).dst.transfer.is_null() {
        0
    } else {
        (*(*args).dst.transfer).n
    };
    let dst_annots_n = if (*args).dst.annots.is_null() {
        0
    } else {
        (*(*args).dst.annots).n
    };
    if crate::htslib_rs::bgzf::bgzf_close((*args).out_fp) != 0 {
        libc::abort();
    }
    if crate::htslib_rs::hts::hts_close((*args).dst.fp) != 0 {
        libc::abort();
    }
    for hash in (*args).tmp_hash.drain(..) {
        sam::khash_str2int_destroy(hash);
    }
    if !(*args).tmp_cols.is_null() {
        for col in (*(*args).tmp_cols).iter_mut() {
            cols_clear(col);
        }
        drop(Box::from_raw((*args).tmp_cols));
    }
    cols_destroy_ptr((*args).src.core);
    cols_destroy_ptr((*args).dst.core);
    cols_destroy_ptr((*args).src.match_);
    cols_destroy_ptr((*args).dst.match_);
    cols_destroy_ptr((*args).src.transfer);
    cols_destroy_ptr((*args).dst.transfer);
    if !(*args).src.annots.is_null() {
        cols_destroy_ptr((*args).src.annots);
    }
    if !(*args).dst.annots.is_null() {
        cols_destroy_ptr((*args).dst.annots);
    }
    (*args).nbp = None;
    destroy_header(&mut *((&mut (*args).src as *mut AnnotTsvDat).cast::<AnnotTsvDatHeaderOnly>()));
    destroy_header(&mut *((&mut (*args).dst as *mut AnnotTsvDat).cast::<AnnotTsvDatHeaderOnly>()));
    (*args).src.nannots_added.clear();
    (*args).src.core_idx.clear();
    (*args).dst.core_idx.clear();
    (*args).src.match_idx.clear();
    (*args).dst.match_idx.clear();
    (*args).src.transfer_idx.clear();
    (*args).dst.transfer_idx.clear();
    (*args).dst.annots_idx.clear();
    libc::free((*args).src.line.s.cast());
    libc::free((*args).dst.line.s.cast());
    if !(*args).itr.is_null() {
        crate::htslib_rs::regidx::regidx_c_606_regitr_destroy(
            (*args).itr.cast::<crate::htslib_rs::regidx::regitr_t>(),
        );
    }
    if !(*args).idx.is_null() {
        crate::htslib_rs::regidx::regidx_c_311_regidx_destroy(
            (*args).idx.cast::<crate::htslib_rs::regidx::regidx_t>(),
        );
    }
    libc::free((*args).tmp_kstr.s.cast());
}

unsafe fn write_string(args: &mut AnnotTsvArgs, mut str_: *mut i8, mut len: usize) {
    if len == 0 {
        len = libc::strlen(str_);
    }
    if len == 0 {
        str_ = c".".as_ptr().cast_mut();
        len = 1;
    }
    if crate::htslib_rs::bgzf::bgzf_write(args.out_fp, str_.cast(), len) != len as isize {
        libc::abort();
    }
}

// original: write_string (htslib/annot-tsv.c:703)
pub unsafe fn annot_tsv_c_703_write_string(args: *mut c_void, str_: *mut i8, len: usize) {
    write_string(&mut *args.cast::<AnnotTsvArgs>(), str_, len);
}

unsafe fn write_annots(args: &mut AnnotTsvArgs) {
    if args.dst.annots.is_null() {
        return;
    }

    args.tmp_kstr.l = 0;
    let (len, frac_denominator, cnt) = {
        let nbp = args.nbp.as_mut().expect("annotations require nbp state");
        (
            nbp_length(nbp),
            (nbp.end - nbp.beg + 1) as f64,
            (nbp.regs.len() / 2) as i32,
        )
    };
    for i in 0..(*args.dst.annots).n as usize {
        crate::htslib_rs::hts::kputc(args.dst.delim as i32, &mut args.tmp_kstr);
        let ann = args.dst.annots_idx[i];
        if ann == ANNOT_TSV_ANN_NBP {
            crate::htslib_rs::hts::kputw(len as i32, &mut args.tmp_kstr);
        } else if ann == ANNOT_TSV_ANN_FRAC {
            crate::htslib_rs::hts::kputd(len as f64 / frac_denominator, &mut args.tmp_kstr);
        } else if ann == ANNOT_TSV_ANN_CNT {
            crate::htslib_rs::hts::kputw(cnt, &mut args.tmp_kstr);
        }
    }
    write_string(args, args.tmp_kstr.s, args.tmp_kstr.l);
}

// original: write_annots (htslib/annot-tsv.c:709)
pub unsafe fn annot_tsv_c_709_write_annots(args: *mut c_void) {
    write_annots(&mut *args.cast::<AnnotTsvArgs>());
}

// original: process_line (htslib/annot-tsv.c:737)
pub unsafe fn annot_tsv_c_737_process_line(args: *mut c_void, line: *mut i8, size: usize) {
    let args = args.cast::<AnnotTsvArgs>();
    let Some(parsed) = parse_tab_with_payload_ref(line, &mut (*args).dst) else {
        return;
    };
    let chr_beg = parsed.chr_beg;
    let beg = parsed.beg;
    let end = parsed.end;
    let mut dst_cols = parsed.cols;

    if let Some(nbp) = (*args).nbp.as_mut() {
        nbp_reset(nbp, beg, end);
    }

    if regidx::regidx_c_401_regidx_overlap((*args).idx, chr_beg, beg, end, (*args).itr) == 0 {
        if (*args).mode & ANNOT_TSV_PRINT_NONMATCHING != 0 {
            write_string(&mut *args, line, size);
            write_annots(&mut *args);
            write_string(&mut *args, c"\n".as_ptr().cast_mut(), 1);
        }
        return;
    }

    for i in 0..(*(*args).src.transfer).n as usize {
        (&mut (*args).src.nannots_added)[i] = 0;
        (&mut (*(*args).tmp_cols))[i].n = 0;
        sam::khash_str2int_destroy((&(*args).tmp_hash)[i]);
        (&mut (*args).tmp_hash)[i] = sam::khash_str2int_init();
    }

    let mut has_match = 0;
    let mut annot_len: usize = 0;
    while regidx::regidx_c_612_regitr_overlap((*args).itr) != 0 {
        if (*args).overlap_src != 0.0 || (*args).overlap_dst != 0.0 {
            let len_dst = (end - beg + 1) as f64;
            let len_src = ((*(*args).itr).end - (*(*args).itr).beg + 1) as f64;
            let isec = (((*(*args).itr).end.min(end)) - ((*(*args).itr).beg.max(beg)) + 1) as f64;
            let pass_dst = (isec / len_dst >= (*args).overlap_dst) as i32;
            let pass_src = (isec / len_src >= (*args).overlap_src) as i32;
            if (*args).overlap_either != 0 {
                if pass_dst == 0 && pass_src == 0 {
                    continue;
                }
            } else if pass_dst == 0 || pass_src == 0 {
                continue;
            }
        }
        let src_cols = *(*(*args).itr)
            .payload
            .expect("regidx overlap payload")
            .cast::<*mut AnnotTsvCols>()
            .as_ptr();
        if !(*args).dst.match_.is_null() && (*(*args).dst.match_).n != 0 {
            let mut i = 0usize;
            while i < (*(*args).dst.match_).n as usize {
                if (&(*args).dst.match_idx)[i] > dst_cols.n as i32 {
                    libc::abort();
                }
                let dst = dst_cols.off.get((&(*args).dst.match_idx)[i] as usize);
                let src = (*src_cols).off.get((&(*args).src.match_idx)[i] as usize);
                if libc::strcmp(dst, src) != 0 {
                    break;
                }
                i += 1;
            }
            if i != (*(*args).dst.match_).n as usize {
                continue;
            }
        }
        has_match = 1;

        if let Some(nbp) = (*args).nbp.as_mut() {
            nbp_add(
                nbp,
                (*(*args).itr).beg.max(beg),
                (*(*args).itr).end.min(end),
            );
        }

        let mut max_annots_reached = 0;
        for i in 0..(*(*args).src.transfer).n as usize {
            let mut str_ = if (&(*args).src.transfer_idx)[i] >= 0 {
                (*src_cols).off.get((&(*args).src.transfer_idx)[i] as usize)
            } else {
                (*(*args).src.hdr.cols)
                    .off
                    .get((-(&(*args).src.transfer_idx)[i] - 1) as usize)
            };
            if str_.is_null() || *str_ == 0 {
                str_ = c".".as_ptr().cast_mut();
            }
            if (*args).allow_dups == 0 {
                if sam::khash_str2int_has_key((&(*args).tmp_hash)[i], str_) != 0 {
                    continue;
                }
                sam::khash_str2int_set((&(*args).tmp_hash)[i], str_, 1);
            }
            if (*args).max_annots != 0 {
                (&mut (*args).src.nannots_added)[i] += 1;
                if (&(*args).src.nannots_added)[i] >= (*args).max_annots {
                    max_annots_reached = 1;
                }
            }
            cols_append(&mut (&mut (*(*args).tmp_cols))[i], str_);
            annot_len += libc::strlen(str_);
        }
        if max_annots_reached != 0 {
            break;
        }
    }

    if has_match == 0 {
        if (*args).mode & ANNOT_TSV_PRINT_NONMATCHING != 0 {
            write_string(&mut *args, line, size);
            write_annots(&mut *args);
            write_string(&mut *args, c"\n".as_ptr().cast_mut(), 1);
        }
        return;
    }
    if (*args).mode & ANNOT_TSV_PRINT_MATCHING == 0 {
        return;
    }

    (*args).tmp_kstr.l = 0;
    ks_resize(
        &mut (*args).tmp_kstr,
        annot_len * 3 + (*(*args).src.transfer).n as usize * 2,
    );
    for i in 0..(*(*args).src.transfer).n as usize {
        let mut off = (*args).tmp_kstr.s.add((*args).tmp_kstr.l);
        dst_cols
            .off
            .set((&(*args).dst.transfer_idx)[i] as usize, off);
        let ann = &mut (&mut (*(*args).tmp_cols))[i] as *mut AnnotTsvCols;
        if (*ann).n == 0 {
            *off = b'.' as i8;
            *off.add(1) = 0;
            (*args).tmp_kstr.l += 2;
            continue;
        }
        for j in 0..(*ann).n as usize {
            if j > 0 {
                *off = b',' as i8;
                off = off.add(1);
                (*args).tmp_kstr.l += 1;
            }
            let len = libc::strlen(*(*ann).off.add(j));
            libc::memcpy(off.cast(), (*(*ann).off.add(j)).cast(), len);
            off = off.add(len);
            (*args).tmp_kstr.l += len;
        }
        *off = 0;
        (*args).tmp_kstr.l += 1;
    }
    write_string(&mut *args, dst_cols.off.get(0), 0);
    for i in 1..dst_cols.n {
        write_string(&mut *args, &mut (*args).dst.delim as *mut i8, 1);
        write_string(&mut *args, dst_cols.off.get(i), 0);
    }
    write_annots(&mut *args);
    write_string(&mut *args, c"\n".as_ptr().cast_mut(), 1);
}

// original: usage_text (htslib/annot-tsv.c:880)
pub unsafe fn annot_tsv_c_880_usage_text() -> *const i8 {
    ANNOT_TSV_USAGE_TEXT.as_ptr().cast()
}

// original: main (htslib/annot-tsv.c:956)
pub unsafe fn annot_tsv_c_956_main(argc: i32, argv: *mut *mut i8) -> i32 {
    let args = Box::into_raw(Box::<AnnotTsvArgs>::default());
    let mut reciprocal = 0;
    let argv_slice = std::slice::from_raw_parts(argv, argc as usize);
    let mut i = 1usize;
    while i < argv_slice.len() {
        let arg = CStr::from_ptr(argv_slice[i]).to_bytes();
        let mut optarg: *mut i8 = std::ptr::null_mut();
        let c = if let Some(long) = arg.strip_prefix(b"--") {
            let (name, value) = match long.iter().position(|&ch| ch == b'=') {
                Some(eq) => (
                    &long[..eq],
                    Some(argv_slice[i].cast::<i8>().add(2 + eq + 1)),
                ),
                None => (long, None),
            };
            match name {
                b"allow-dups" => 0,
                b"version" => 1,
                b"max-annots" => {
                    optarg = value.unwrap_or_else(|| {
                        i += 1;
                        if i >= argv_slice.len() {
                            libc::abort();
                        }
                        argv_slice[i]
                    });
                    2
                }
                b"help" => 4,
                b"core" | b"coords" | b"transfer" | b"match" | b"output" | b"source-file"
                | b"target-file" | b"annotate" | b"headers" | b"overlap" | b"delim" => {
                    optarg = value.unwrap_or_else(|| {
                        i += 1;
                        if i >= argv_slice.len() {
                            libc::abort();
                        }
                        argv_slice[i]
                    });
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
                        _ => libc::abort(),
                    }
                }
                b"no-header-idx" => b'I' as i32,
                b"ignore-headers" => b'H' as i32,
                b"reciprocal" => b'r' as i32,
                b"drop-overlaps" => b'x' as i32,
                _ => libc::abort(),
            }
        } else if arg.starts_with(b"-") && arg.len() > 1 {
            let mut pos = 1usize;
            let mut parsed = 0;
            while pos < arg.len() {
                let opt = arg[pos];
                match opt {
                    b'I' => (*args).no_write_hdr += 1,
                    b'H' => (*args).headers_str = c"0:0".as_ptr().cast_mut(),
                    b'r' => reciprocal = 1,
                    b'x' => (*args).mode = ANNOT_TSV_PRINT_NONMATCHING,
                    b'c' | b'C' | b'f' | b'm' | b'o' | b's' | b't' | b'a' | b'O' | b'h' | b'd' => {
                        optarg = if pos + 1 < arg.len() {
                            argv_slice[i].cast::<i8>().add(pos + 1)
                        } else {
                            i += 1;
                            if i >= argv_slice.len() {
                                libc::abort();
                            }
                            argv_slice[i]
                        };
                        parsed = opt as i32;
                        break;
                    }
                    _ => libc::abort(),
                }
                pos += 1;
            }
            if parsed == 0 {
                i += 1;
                continue;
            }
            parsed
        } else {
            libc::abort();
        };

        match c {
            0 => (*args).allow_dups = 1,
            1 => return 0,
            2 => {
                let mut tmp: *mut i8 = std::ptr::null_mut();
                (*args).max_annots = libc::strtod(optarg, &mut tmp) as i32;
                if tmp == optarg || *tmp != 0 {
                    libc::abort();
                }
            }
            x if x == b'I' as i32 => (*args).no_write_hdr += 1,
            x if x == b'd' as i32 => (*args).delim_str = optarg,
            x if x == b'h' as i32 => (*args).headers_str = optarg,
            x if x == b'H' as i32 => (*args).headers_str = c"0:0".as_ptr().cast_mut(),
            x if x == b'r' as i32 => reciprocal = 1,
            x if x == b'c' as i32 => (*args).core_str = optarg,
            x if x == b'C' as i32 => (*args).coords_str = optarg,
            x if x == b't' as i32 => (*args).dst.fname = optarg,
            x if x == b'm' as i32 => (*args).match_str = optarg,
            x if x == b'a' as i32 => (*args).annots_str = optarg,
            x if x == b'o' as i32 => (*args).out_fname = optarg,
            x if x == b'O' as i32 => {
                let mut tmp: *mut i8 = std::ptr::null_mut();
                (*args).overlap_src = libc::strtod(optarg, &mut tmp);
                if tmp == optarg || (*tmp != 0 && *tmp != b',' as i8) {
                    libc::abort();
                }
                if (*args).overlap_src < 0.0 || (*args).overlap_src > 1.0 {
                    libc::abort();
                }
                if *tmp != 0 {
                    (*args).overlap_dst = libc::strtod(tmp.add(1), &mut tmp);
                    if *tmp != 0 || (*args).overlap_dst < 0.0 || (*args).overlap_dst > 1.0 {
                        libc::abort();
                    }
                } else {
                    (*args).overlap_either = 1;
                }
            }
            x if x == b's' as i32 => (*args).src.fname = optarg,
            x if x == b'f' as i32 => (*args).transfer_str = optarg,
            x if x == b'x' as i32 => (*args).mode = ANNOT_TSV_PRINT_NONMATCHING,
            4 => return 0,
            _ => libc::abort(),
        }
        i += 1;
    }
    if argc == 1 {
        libc::abort();
    }
    if (*args).dst.fname.is_null() && (*args).src.fname.is_null() {
        libc::abort();
    }
    if (*args).dst.fname.is_null() {
        (*args).dst.fname = c"-".as_ptr().cast_mut();
    }
    if (*args).src.fname.is_null() {
        (*args).src.fname = c"-".as_ptr().cast_mut();
    }
    if (*args).mode == 0 {
        (*args).mode = if (*args).transfer_str.is_null() && (*args).annots_str.is_null() {
            ANNOT_TSV_PRINT_MATCHING
        } else {
            ANNOT_TSV_PRINT_MATCHING | ANNOT_TSV_PRINT_NONMATCHING
        };
    }
    if (!(*args).transfer_str.is_null() || !(*args).annots_str.is_null())
        && (*args).mode & ANNOT_TSV_PRINT_MATCHING == 0
    {
        libc::abort();
    }
    if reciprocal != 0 {
        if (*args).overlap_dst != 0.0
            && (*args).overlap_src != 0.0
            && (*args).overlap_dst != (*args).overlap_src
        {
            libc::abort();
        }
        if (*args).overlap_src == 0.0 {
            (*args).overlap_src = (*args).overlap_dst;
        } else {
            (*args).overlap_dst = (*args).overlap_src;
        }
        (*args).overlap_either = 0;
    }

    annot_tsv_c_515_init_data(args.cast());
    write_header(&mut *args, &mut (*args).dst);
    while read_next_line(&mut (*args).dst) != 0 {
        for _ in 0..(*args).dst.grow_n {
            kputc((*args).dst.delim as i32, &mut (*args).dst.line);
            kputc(b'.' as i32, &mut (*args).dst.line);
        }
        annot_tsv_c_737_process_line(args.cast(), (*args).dst.line.s, (*args).dst.line.l);
        (*args).dst.line.l = 0;
    }
    annot_tsv_c_666_destroy_data(args.cast());
    drop(Box::from_raw(args));
    0
}
