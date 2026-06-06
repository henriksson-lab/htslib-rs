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
use std::ffi::{c_char, c_int, c_uint, c_void, CStr};

const ANNOT_TSV_ANN_NBP: c_int = 1;
const ANNOT_TSV_ANN_FRAC: c_int = 2;
const ANNOT_TSV_ANN_CNT: c_int = 4;
const ANNOT_TSV_PRINT_MATCHING: c_int = 1;
const ANNOT_TSV_PRINT_NONMATCHING: c_int = 2;

#[derive(Default)]
struct AnnotTsvOffsets(Vec<*mut c_char>);

impl AnnotTsvOffsets {
    fn add(&mut self, i: usize) -> *mut *mut c_char {
        unsafe { self.0.as_mut_ptr().add(i) }
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn push(&mut self, ptr: *mut c_char) {
        self.0.push(ptr);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> std::slice::Iter<'_, *mut c_char> {
        self.0.iter()
    }
}

#[derive(Default)]
struct AnnotTsvCols {
    n: c_uint,
    off: AnnotTsvOffsets,
    rmme: Option<Box<[c_char]>>,
}

#[repr(C)]
struct AnnotTsvHdr {
    name2idx: *mut c_void,
    cols: *mut AnnotTsvCols,
    annots: *mut AnnotTsvCols,
    dummy: c_int,
}

#[repr(C)]
struct AnnotTsvDatHeaderOnly {
    fname: *mut c_char,
    hdr: AnnotTsvHdr,
}

#[repr(C)]
struct AnnotTsvDat {
    fname: *mut c_char,
    hdr: AnnotTsvHdr,
    core: *mut AnnotTsvCols,
    match_: *mut AnnotTsvCols,
    transfer: *mut AnnotTsvCols,
    annots: *mut AnnotTsvCols,
    core_idx: *mut c_int,
    match_idx: *mut c_int,
    transfer_idx: *mut c_int,
    annots_idx: *mut c_int,
    nannots_added: *mut c_int,
    coor_base: [c_int; 2],
    delim: c_char,
    grow_n: c_int,
    line: kstring_t,
    fp: *mut htsFile,
}

#[derive(Default)]
struct AnnotTsvNbp {
    regs: Vec<hts_pos_t>,
    beg: hts_pos_t,
    end: hts_pos_t,
}

#[repr(C)]
struct AnnotTsvArgs {
    nbp: *mut AnnotTsvNbp,
    dst: AnnotTsvDat,
    src: AnnotTsvDat,
    core_str: *mut c_char,
    coords_str: *mut c_char,
    match_str: *mut c_char,
    transfer_str: *mut c_char,
    annots_str: *mut c_char,
    headers_str: *mut c_char,
    delim_str: *mut c_char,
    temp_dir: *mut c_char,
    out_fname: *mut c_char,
    out_fp: *mut BGZF,
    allow_dups: c_int,
    max_annots: c_int,
    mode: c_int,
    no_write_hdr: c_int,
    overlap_either: c_int,
    overlap_src: f64,
    overlap_dst: f64,
    idx: *mut regidx::regidx_t,
    itr: *mut regidx::regitr_t,
    tmp_kstr: kstring_t,
    tmp_cols: *mut Vec<AnnotTsvCols>,
    tmp_hash: *mut *mut c_void,
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

// original: nbp_init (htslib/annot-tsv.c:131)
unsafe fn nbp_init() -> *mut AnnotTsvNbp {
    Box::into_raw(Box::<AnnotTsvNbp>::default())
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
    (*nbp).regs.clear();
    (*nbp).beg = beg;
    (*nbp).end = end;
}

// original: nbp_add (htslib/annot-tsv.c:148)
pub unsafe fn annot_tsv_c_148_nbp_add(nbp: *mut c_void, beg: hts_pos_t, end: hts_pos_t) {
    let nbp = nbp.cast::<AnnotTsvNbp>();
    (*nbp).regs.push(beg << 1);
    (*nbp).regs.push((end << 1) + 1);
}

// original: compare_hts_pos (htslib/annot-tsv.c:160)
pub unsafe fn annot_tsv_c_160_compare_hts_pos(aptr: *const c_void, bptr: *const c_void) -> c_int {
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
    if (*nbp).regs.is_empty() {
        return 0;
    }
    (*nbp).regs.sort_unstable();

    let mut nopen = 0;
    let mut beg = 0;
    let mut length = 0;
    for &reg in (*nbp).regs.iter() {
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

// original: cols_split (htslib/annot-tsv.c:187)
pub unsafe fn annot_tsv_c_187_cols_split(
    line: *const c_char,
    cols: *mut c_void,
    delim: c_char,
) -> *mut c_void {
    let cols = if cols.is_null() {
        Box::into_raw(Box::<AnnotTsvCols>::default())
    } else {
        cols.cast::<AnnotTsvCols>()
    };
    if cols.is_null() {
        libc::abort();
    }
    (*cols).n = 0;
    (*cols).off.clear();
    let bytes = CStr::from_ptr(line).to_bytes_with_nul();
    let mut rmme: Box<[c_char]> = bytes
        .iter()
        .map(|&byte| byte as c_char)
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
        (*cols).off.push(ss);
        (*cols).n += 1;
        if tmp == 0 {
            break;
        }
        ss = se.add(1);
    }
    (*cols).rmme = Some(rmme);
    cols.cast()
}

// original: cols_append (htslib/annot-tsv.c:217)
pub unsafe fn annot_tsv_c_217_cols_append(cols: *mut c_void, str_: *mut c_char) {
    let cols = cols.cast::<AnnotTsvCols>();
    if (*cols).rmme.is_some() {
        let mut bytes = Vec::<c_char>::new();
        for &old in (*cols).off.iter().take((*cols).n as usize) {
            let old_bytes = CStr::from_ptr(old).to_bytes_with_nul();
            bytes.extend(old_bytes.iter().map(|&byte| byte as c_char));
        }
        let str_bytes = CStr::from_ptr(str_).to_bytes_with_nul();
        bytes.extend(str_bytes.iter().map(|&byte| byte as c_char));

        let mut rmme = bytes.into_boxed_slice();
        let mut off = AnnotTsvOffsets(Vec::with_capacity((*cols).off.len() + 1));
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
        (*cols).rmme = Some(rmme);
        (*cols).off = off;
        (*cols).n += 1;
        return;
    }
    if ((*cols).n as usize) < (*cols).off.len() {
        *(*cols).off.add((*cols).n as usize) = str_;
    } else {
        (*cols).off.push(str_);
    }
    (*cols).n += 1;
}

// original: cols_clear (htslib/annot-tsv.c:261)
pub unsafe fn annot_tsv_c_261_cols_clear(cols: *mut c_void) {
    let cols = cols.cast::<AnnotTsvCols>();
    if cols.is_null() {
        return;
    }
    (*cols).rmme = None;
    (*cols).n = 0;
    (*cols).off.clear();
}

// original: cols_destroy (htslib/annot-tsv.c:269)
pub unsafe fn annot_tsv_c_269_cols_destroy(cols: *mut c_void) {
    let cols = cols.cast::<AnnotTsvCols>();
    if cols.is_null() {
        return;
    }
    drop(Box::from_raw(cols));
}

// original: parse_tab_with_payload (htslib/annot-tsv.c:276)
pub unsafe extern "C" fn annot_tsv_c_276_parse_tab_with_payload(
    line: *const c_char,
    chr_beg: *mut *mut c_char,
    chr_end: *mut *mut c_char,
    beg: *mut hts_pos_t,
    end: *mut hts_pos_t,
    payload: *mut c_void,
    usr: *mut c_void,
) -> c_int {
    if *line == b'#' as c_char {
        *(payload.cast::<*mut AnnotTsvCols>()) = std::ptr::null_mut();
        return -1;
    }

    let dat = usr.cast::<AnnotTsvDat>();
    let cols =
        annot_tsv_c_187_cols_split(line, std::ptr::null_mut(), (*dat).delim).cast::<AnnotTsvCols>();
    *(payload.cast::<*mut AnnotTsvCols>()) = cols;

    if (*cols).n < *(*dat).core_idx.add(0) as c_uint {
        libc::abort();
    }
    *chr_beg = *(*cols).off.add(*(*dat).core_idx.add(0) as usize);
    *chr_end = (*chr_beg).add(libc::strlen(*chr_beg) - 1);

    if (*cols).n < *(*dat).core_idx.add(1) as c_uint {
        libc::abort();
    }
    let mut tmp: *mut c_char = std::ptr::null_mut();
    let mut ptr = *(*cols).off.add(*(*dat).core_idx.add(1) as usize);
    *beg = libc::strtod(ptr, &mut tmp) as hts_pos_t;
    if tmp == ptr {
        libc::abort();
    }

    if (*cols).n < *(*dat).core_idx.add(2) as c_uint {
        libc::abort();
    }
    ptr = *(*cols).off.add(*(*dat).core_idx.add(2) as usize);
    *end = libc::strtod(ptr, &mut tmp) as hts_pos_t;
    if tmp == ptr {
        libc::abort();
    }

    *beg -= (*dat).coor_base[0] as hts_pos_t - 1;
    *end -= (*dat).coor_base[1] as hts_pos_t - 1;

    if *end < *beg {
        core::ptr::swap(beg, end);
    }

    0
}

// original: free_payload (htslib/annot-tsv.c:322)
pub unsafe extern "C" fn annot_tsv_c_322_free_payload(payload: *mut c_void) {
    let cols = *(payload.cast::<*mut AnnotTsvCols>());
    annot_tsv_c_269_cols_destroy(cols.cast());
}

// original: parse_header (htslib/annot-tsv.c:335)
pub unsafe fn annot_tsv_c_335_parse_header(
    dat: *mut c_void,
    fname: *mut c_char,
    mut nth_row: c_int,
    autodetect: c_int,
) {
    let dat = dat.cast::<AnnotTsvDat>();
    (*dat).fp = hts_open(fname, c"r".as_ptr());
    if (*dat).fp.is_null() {
        libc::abort();
    }

    let mut nbuf: c_int = 0;
    let mut buf: *mut *mut c_char = std::ptr::null_mut();
    if nth_row < 0 {
        buf = libc::calloc((-nth_row) as usize, std::mem::size_of::<*mut c_char>()).cast();
        if buf.is_null() {
            libc::abort();
        }
    }

    let mut irow = 0;
    let mut cols: *mut AnnotTsvCols;
    while hts_getline((*dat).fp, 2, &mut (*dat).line) > 0 {
        if autodetect != 0 {
            nth_row = if *(*dat).line.s == b'#' as c_char {
                1
            } else {
                0
            };
            break;
        }
        if nth_row == 0 {
            if *(*dat).line.s == b'#' as c_char {
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
        if *(*dat).line.s != b'#' as c_char {
            break;
        }
        if nbuf == -nth_row {
            libc::free((*buf).cast());
            libc::memmove(
                buf.cast(),
                buf.add(1).cast(),
                (nbuf as usize - 1) * std::mem::size_of::<*mut c_char>(),
            );
            nbuf -= 1;
        }
        *buf.add(nbuf as usize) = libc::strdup((*dat).line.s);
        if (*buf.add(nbuf as usize)).is_null() {
            libc::abort();
        }
        nbuf += 1;
    }

    let mut keep_line = 0;
    if nth_row < 0 {
        if nbuf != -nth_row {
            libc::abort();
        }
        cols = annot_tsv_c_187_cols_split(*buf, std::ptr::null_mut(), (*dat).delim)
            .cast::<AnnotTsvCols>();
        keep_line = 1;
    } else {
        cols = annot_tsv_c_187_cols_split((*dat).line.s, std::ptr::null_mut(), (*dat).delim)
            .cast::<AnnotTsvCols>();
    }

    if (*dat).line.l == 0 || cols.is_null() || (*cols).n == 0 {
        libc::abort();
    }

    if nth_row == 0 {
        let mut str_: kstring_t = std::mem::zeroed();
        for i in 0..(*cols).n as c_int {
            if i > 0 {
                kputc((*dat).delim as c_int, &mut str_);
            }
            kputw(i + 1, &mut str_);
        }
        annot_tsv_c_269_cols_destroy(cols.cast());
        cols = annot_tsv_c_187_cols_split(str_.s, std::ptr::null_mut(), (*dat).delim)
            .cast::<AnnotTsvCols>();
        libc::free(str_.s.cast());
        (*dat).hdr.dummy = 1;
        keep_line = 1;
    }

    (*dat).hdr.name2idx = sam::khash_str2int_init();
    for i in 0..(*cols).n as usize {
        let mut ss = *(*cols).off.add(i);
        while *ss != 0 && (*ss == b'#' as c_char || isspace_c(*ss) != 0) {
            ss = ss.add(1);
        }
        if *ss == 0 {
            libc::abort();
        }
        if *ss == b'[' as c_char {
            let mut se = ss.add(1);
            while *se != 0 && isdigit_c(*se) != 0 {
                se = se.add(1);
            }
            if *se == b']' as c_char {
                ss = se.add(1);
            }
        }
        while *ss != 0 && (*ss == b'#' as c_char || isspace_c(*ss) != 0) {
            ss = ss.add(1);
        }
        if *ss == 0 {
            libc::abort();
        }
        *(*cols).off.add(i) = ss;
        sam::khash_str2int_set((*dat).hdr.name2idx, ss, i as c_int);
    }
    (*dat).hdr.cols = cols;
    if keep_line == 0 {
        (*dat).line.l = 0;
    }

    for i in 0..nbuf as usize {
        libc::free((*buf.add(i)).cast());
    }
    libc::free(buf.cast());
}

// original: write_header (htslib/annot-tsv.c:440)
pub unsafe fn annot_tsv_c_440_write_header(args: *mut c_void, dat: *mut c_void) {
    let args = args.cast::<AnnotTsvArgs>();
    let dat = dat.cast::<AnnotTsvDat>();
    if (*dat).hdr.dummy != 0 || (*args).no_write_hdr > 1 {
        return;
    }
    let mut str_: kstring_t = std::mem::zeroed();
    kputc(b'#' as c_int, &mut str_);
    for i in 0..(*(*dat).hdr.cols).n as usize {
        if i > 0 {
            kputc((*dat).delim as c_int, &mut str_);
        }
        if (*args).no_write_hdr == 0 {
            kputc(b'[' as c_int, &mut str_);
            kputw(i as c_int + 1, &mut str_);
            kputc(b']' as c_int, &mut str_);
        }
        kputs(*(*(*dat).hdr.cols).off.add(i), &mut str_);
    }
    if !(*dat).hdr.annots.is_null() {
        for i in 0..(*(*dat).hdr.annots).n as usize {
            if str_.l > 1 {
                kputc((*dat).delim as c_int, &mut str_);
            }
            kputs(*(*(*dat).hdr.annots).off.add(i), &mut str_);
        }
    }
    kputc(b'\n' as c_int, &mut str_);
    if bgzf::bgzf_write((*args).out_fp, str_.s.cast(), str_.l) != str_.l as isize {
        libc::abort();
    }
    libc::free(str_.s.cast());
}

// original: destroy_header (htslib/annot-tsv.c:465)
pub unsafe fn annot_tsv_c_465_destroy_header(dat: *mut c_void) {
    let dat = dat.cast::<AnnotTsvDatHeaderOnly>();
    if !(*dat).hdr.cols.is_null() {
        annot_tsv_c_269_cols_destroy((*dat).hdr.cols.cast());
    }
    sam::khash_str2int_destroy((*dat).hdr.name2idx);
}

// original: read_next_line (htslib/annot-tsv.c:471)
pub unsafe fn annot_tsv_c_471_read_next_line(dat: *mut c_void) -> c_int {
    let dat = dat.cast::<AnnotTsvDat>();
    if (*dat).line.l != 0 {
        return (*dat).line.l as c_int;
    }
    let ret = crate::htslib_rs::hts::hts_getline((*dat).fp, 2, &mut (*dat).line);
    if ret > 0 {
        return (*dat).line.l as c_int;
    }
    if ret < -1 {
        libc::abort();
    }
    0
}

// original: sanity_check_columns (htslib/annot-tsv.c:480)
pub unsafe fn annot_tsv_c_480_sanity_check_columns(
    _fname: *mut c_char,
    hdr: *mut c_void,
    cols: *mut c_void,
    col2idx: *mut *mut c_int,
    force: c_int,
) {
    let hdr = hdr.cast::<AnnotTsvHdr>();
    let cols = cols.cast::<AnnotTsvCols>();
    *col2idx = libc::malloc((*cols).n as usize * std::mem::size_of::<c_int>()).cast::<c_int>();
    if (*col2idx).is_null() {
        libc::abort();
    }
    for i in 0..(*cols).n as usize {
        let mut idx = 0;
        if sam::khash_str2int_get((*hdr).name2idx, *(*cols).off.add(i), &mut idx) < 0 {
            if force == 0 {
                libc::abort();
            }
            idx = -1;
        }
        *(*col2idx).add(i) = idx;
    }
}

// original: parse_coor_base (htslib/annot-tsv.c:495)
pub unsafe fn annot_tsv_c_495_parse_coor_base(
    _args: *mut c_void,
    str_: *mut c_char,
    dat: *mut c_void,
) {
    let dat = dat.cast::<AnnotTsvDat>();
    let len = libc::strlen((*dat).fname);
    let mut beg = 1;
    let mut end = 1;
    if *str_ != 0 {
        if *str_ == b'0' as c_char {
            beg = 0;
        } else if *str_ == b'1' as c_char {
            beg = 1;
        } else {
            libc::abort();
        }

        if *str_.add(1) == b'0' as c_char {
            end = 0;
        } else if *str_.add(1) == b'1' as c_char {
            end = 1;
        } else {
            libc::abort();
        }
    } else if (len >= 4 && libc::strcasecmp(c".bed".as_ptr(), (*dat).fname.add(len - 4)) == 0)
        || (len >= 7 && libc::strcasecmp(c".bed.gz".as_ptr(), (*dat).fname.add(len - 7)) == 0)
    {
        beg = 0;
    }
    (*dat).coor_base[0] = beg;
    (*dat).coor_base[1] = end;
}

// original: init_data (htslib/annot-tsv.c:515)
pub unsafe fn annot_tsv_c_515_init_data(args: *mut c_void) {
    let args = args.cast::<AnnotTsvArgs>();
    if (*args).delim_str.is_null() {
        (*args).dst.delim = b'\t' as c_char;
        (*args).src.delim = b'\t' as c_char;
    } else if libc::strlen((*args).delim_str) == 1 {
        (*args).dst.delim = *(*args).delim_str;
        (*args).src.delim = *(*args).delim_str;
    } else if libc::strlen((*args).delim_str) == 3 && *(*args).delim_str.add(1) == b':' as c_char {
        (*args).src.delim = *(*args).delim_str;
        (*args).dst.delim = *(*args).delim_str.add(2);
    } else {
        libc::abort();
    }

    let mut isrc = 0;
    let mut idst = 0;
    let mut autodetect = 1;
    if !(*args).headers_str.is_null() {
        let tmp =
            annot_tsv_c_187_cols_split((*args).headers_str, std::ptr::null_mut(), b':' as c_char)
                .cast::<AnnotTsvCols>();
        let mut rmme: *mut c_char = std::ptr::null_mut();
        isrc = libc::strtol(*(*tmp).off.add(0), &mut rmme, 10) as c_int;
        if *rmme != 0 || *(*tmp).off.add(0) == rmme {
            libc::abort();
        }
        let dst_str = if (*tmp).n == 2 {
            *(*tmp).off.add(1)
        } else {
            *(*tmp).off.add(0)
        };
        idst = libc::strtol(dst_str, &mut rmme, 10) as c_int;
        if *rmme != 0 || dst_str == rmme {
            libc::abort();
        }
        annot_tsv_c_269_cols_destroy(tmp.cast());
        autodetect = 0;
    }
    annot_tsv_c_335_parse_header(
        (&mut (*args).dst as *mut AnnotTsvDat).cast(),
        (*args).dst.fname,
        idst,
        autodetect,
    );
    annot_tsv_c_335_parse_header(
        (&mut (*args).src as *mut AnnotTsvDat).cast(),
        (*args).src.fname,
        isrc,
        autodetect,
    );

    if (*args).core_str.is_null() {
        (*args).core_str = c"chr,beg,end:chr,beg,end".as_ptr().cast_mut();
    }
    let mut tmp =
        annot_tsv_c_187_cols_split((*args).core_str, std::ptr::null_mut(), b':' as c_char)
            .cast::<AnnotTsvCols>();
    (*args).src.core =
        annot_tsv_c_187_cols_split(*(*tmp).off.add(0), std::ptr::null_mut(), b',' as c_char).cast();
    (*args).dst.core = annot_tsv_c_187_cols_split(
        if (*tmp).n == 2 {
            *(*tmp).off.add(1)
        } else {
            *(*tmp).off.add(0)
        },
        std::ptr::null_mut(),
        b',' as c_char,
    )
    .cast();
    annot_tsv_c_480_sanity_check_columns(
        (*args).src.fname,
        (&mut (*args).src.hdr as *mut AnnotTsvHdr).cast(),
        (*args).src.core.cast(),
        &mut (*args).src.core_idx,
        0,
    );
    annot_tsv_c_480_sanity_check_columns(
        (*args).dst.fname,
        (&mut (*args).dst.hdr as *mut AnnotTsvHdr).cast(),
        (*args).dst.core.cast(),
        &mut (*args).dst.core_idx,
        0,
    );
    if (*(*args).src.core).n != 3 || (*(*args).dst.core).n != 3 {
        libc::abort();
    }
    annot_tsv_c_269_cols_destroy(tmp.cast());

    if (*args).coords_str.is_null() {
        (*args).coords_str = c":".as_ptr().cast_mut();
    }
    tmp = annot_tsv_c_187_cols_split((*args).coords_str, std::ptr::null_mut(), b':' as c_char)
        .cast::<AnnotTsvCols>();
    annot_tsv_c_495_parse_coor_base(
        args.cast(),
        *(*tmp).off.add(0),
        (&mut (*args).src as *mut AnnotTsvDat).cast(),
    );
    annot_tsv_c_495_parse_coor_base(
        args.cast(),
        if (*tmp).n == 2 {
            *(*tmp).off.add(1)
        } else {
            *(*tmp).off.add(0)
        },
        (&mut (*args).dst as *mut AnnotTsvDat).cast(),
    );
    annot_tsv_c_269_cols_destroy(tmp.cast());

    if !(*args).match_str.is_null() {
        tmp = annot_tsv_c_187_cols_split((*args).match_str, std::ptr::null_mut(), b':' as c_char)
            .cast::<AnnotTsvCols>();
        (*args).src.match_ =
            annot_tsv_c_187_cols_split(*(*tmp).off.add(0), std::ptr::null_mut(), b',' as c_char)
                .cast();
        (*args).dst.match_ = annot_tsv_c_187_cols_split(
            if (*tmp).n == 2 {
                *(*tmp).off.add(1)
            } else {
                *(*tmp).off.add(0)
            },
            std::ptr::null_mut(),
            b',' as c_char,
        )
        .cast();
        annot_tsv_c_480_sanity_check_columns(
            (*args).src.fname,
            (&mut (*args).src.hdr as *mut AnnotTsvHdr).cast(),
            (*args).src.match_.cast(),
            &mut (*args).src.match_idx,
            0,
        );
        annot_tsv_c_480_sanity_check_columns(
            (*args).dst.fname,
            (&mut (*args).dst.hdr as *mut AnnotTsvHdr).cast(),
            (*args).dst.match_.cast(),
            &mut (*args).dst.match_idx,
            0,
        );
        if (*(*args).src.match_).n != (*(*args).dst.match_).n {
            libc::abort();
        }
        annot_tsv_c_269_cols_destroy(tmp.cast());
    }

    if !(*args).transfer_str.is_null() {
        tmp =
            annot_tsv_c_187_cols_split((*args).transfer_str, std::ptr::null_mut(), b':' as c_char)
                .cast::<AnnotTsvCols>();
        (*args).src.transfer =
            annot_tsv_c_187_cols_split(*(*tmp).off.add(0), std::ptr::null_mut(), b',' as c_char)
                .cast();
        (*args).dst.transfer = annot_tsv_c_187_cols_split(
            if (*tmp).n == 2 {
                *(*tmp).off.add(1)
            } else {
                *(*tmp).off.add(0)
            },
            std::ptr::null_mut(),
            b',' as c_char,
        )
        .cast();
        annot_tsv_c_480_sanity_check_columns(
            (*args).src.fname,
            (&mut (*args).src.hdr as *mut AnnotTsvHdr).cast(),
            (*args).src.transfer.cast(),
            &mut (*args).src.transfer_idx,
            1,
        );
        annot_tsv_c_480_sanity_check_columns(
            (*args).dst.fname,
            (&mut (*args).dst.hdr as *mut AnnotTsvHdr).cast(),
            (*args).dst.transfer.cast(),
            &mut (*args).dst.transfer_idx,
            1,
        );
        if (*(*args).src.transfer).n != (*(*args).dst.transfer).n {
            libc::abort();
        }
        for i in 0..(*(*args).src.transfer).n as usize {
            if *(*args).src.transfer_idx.add(i) == -1 {
                annot_tsv_c_217_cols_append(
                    (*args).src.hdr.cols.cast(),
                    *(*(*args).src.transfer).off.add(i),
                );
                *(*args).src.transfer_idx.add(i) = -((*(*args).src.hdr.cols).n as c_int);
                (*args).src.grow_n += 1;
            }
        }
        for i in 0..(*(*args).dst.transfer).n as usize {
            if *(*args).dst.transfer_idx.add(i) == -1 {
                annot_tsv_c_217_cols_append(
                    (*args).dst.hdr.cols.cast(),
                    *(*(*args).dst.transfer).off.add(i),
                );
                *(*args).dst.transfer_idx.add(i) = (*(*args).dst.hdr.cols).n as c_int - 1;
                (*args).dst.grow_n += 1;
            }
        }
        (*args).tmp_cols = Box::into_raw(Box::new(
            (0..(*(*args).src.transfer).n)
                .map(|_| AnnotTsvCols::default())
                .collect::<Vec<_>>(),
        ));
        (*args).tmp_hash = libc::calloc(
            (*(*args).src.transfer).n as usize,
            std::mem::size_of::<*mut c_void>(),
        )
        .cast();
        if (*args).tmp_hash.is_null() {
            libc::abort();
        }
        for i in 0..(*(*args).src.transfer).n as usize {
            *(*args).tmp_hash.add(i) = sam::khash_str2int_init();
        }
        annot_tsv_c_269_cols_destroy(tmp.cast());
    } else {
        (*args).src.transfer = Box::into_raw(Box::<AnnotTsvCols>::default());
    }
    (*args).src.nannots_added = libc::calloc(
        (*(*args).src.transfer).n as usize,
        std::mem::size_of::<c_int>(),
    )
    .cast();
    if (*args).src.nannots_added.is_null() {
        libc::abort();
    }

    if !(*args).annots_str.is_null() {
        tmp = annot_tsv_c_187_cols_split((*args).annots_str, std::ptr::null_mut(), b':' as c_char)
            .cast::<AnnotTsvCols>();
        (*args).src.annots =
            annot_tsv_c_187_cols_split(*(*tmp).off.add(0), std::ptr::null_mut(), b',' as c_char)
                .cast();
        (*args).dst.annots = annot_tsv_c_187_cols_split(
            if (*tmp).n == 2 {
                *(*tmp).off.add(1)
            } else {
                *(*tmp).off.add(0)
            },
            std::ptr::null_mut(),
            b',' as c_char,
        )
        .cast();
        if (*(*args).src.annots).n != (*(*args).dst.annots).n {
            libc::abort();
        }
        (*args).dst.annots_idx =
            libc::malloc((*(*args).dst.annots).n as usize * std::mem::size_of::<c_int>()).cast();
        if (*args).dst.annots_idx.is_null() {
            libc::abort();
        }
        for i in 0..(*(*args).src.annots).n as usize {
            let src = *(*(*args).src.annots).off.add(i);
            if libc::strcasecmp(src, c"nbp".as_ptr()) == 0 {
                *(*args).dst.annots_idx.add(i) = ANNOT_TSV_ANN_NBP;
                annot_tsv_c_217_cols_append(
                    (*args).dst.hdr.cols.cast(),
                    if (*tmp).n == 2 {
                        *(*(*args).dst.annots).off.add(i)
                    } else {
                        c"nbp".as_ptr().cast_mut()
                    },
                );
            } else if libc::strcasecmp(src, c"frac".as_ptr()) == 0 {
                *(*args).dst.annots_idx.add(i) = ANNOT_TSV_ANN_FRAC;
                annot_tsv_c_217_cols_append(
                    (*args).dst.hdr.cols.cast(),
                    if (*tmp).n == 2 {
                        *(*(*args).dst.annots).off.add(i)
                    } else {
                        c"frac".as_ptr().cast_mut()
                    },
                );
            } else if libc::strcasecmp(src, c"cnt".as_ptr()) == 0 {
                *(*args).dst.annots_idx.add(i) = ANNOT_TSV_ANN_CNT;
                annot_tsv_c_217_cols_append(
                    (*args).dst.hdr.cols.cast(),
                    if (*tmp).n == 2 {
                        *(*(*args).dst.annots).off.add(i)
                    } else {
                        c"cnt".as_ptr().cast_mut()
                    },
                );
            } else {
                libc::abort();
            }
        }
        (*args).nbp = nbp_init();
        annot_tsv_c_269_cols_destroy(tmp.cast());
    }

    (*args).idx = regidx::regidx_c_246_regidx_init(
        std::ptr::null(),
        Some(annot_tsv_c_276_parse_tab_with_payload),
        Some(annot_tsv_c_322_free_payload),
        std::mem::size_of::<AnnotTsvCols>(),
        (&mut (*args).src as *mut AnnotTsvDat).cast(),
    );
    while annot_tsv_c_471_read_next_line((&mut (*args).src as *mut AnnotTsvDat).cast()) != 0 {
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
    if crate::htslib_rs::bgzf::bgzf_close((*args).out_fp) != 0 {
        libc::abort();
    }
    if crate::htslib_rs::hts::hts_close((*args).dst.fp) != 0 {
        libc::abort();
    }
    for i in 0..(*(*args).src.transfer).n as usize {
        sam::khash_str2int_destroy(*(*args).tmp_hash.add(i));
    }
    libc::free((*args).tmp_hash.cast());
    if !(*args).tmp_cols.is_null() {
        for col in (*(*args).tmp_cols).iter_mut() {
            annot_tsv_c_261_cols_clear((col as *mut AnnotTsvCols).cast());
        }
        drop(Box::from_raw((*args).tmp_cols));
    }
    annot_tsv_c_269_cols_destroy((*args).src.core.cast());
    annot_tsv_c_269_cols_destroy((*args).dst.core.cast());
    annot_tsv_c_269_cols_destroy((*args).src.match_.cast());
    annot_tsv_c_269_cols_destroy((*args).dst.match_.cast());
    annot_tsv_c_269_cols_destroy((*args).src.transfer.cast());
    annot_tsv_c_269_cols_destroy((*args).dst.transfer.cast());
    if !(*args).src.annots.is_null() {
        annot_tsv_c_269_cols_destroy((*args).src.annots.cast());
    }
    if !(*args).dst.annots.is_null() {
        annot_tsv_c_269_cols_destroy((*args).dst.annots.cast());
    }
    if !(*args).nbp.is_null() {
        annot_tsv_c_137_nbp_destroy((*args).nbp.cast());
    }
    annot_tsv_c_465_destroy_header((&mut (*args).src as *mut AnnotTsvDat).cast());
    annot_tsv_c_465_destroy_header((&mut (*args).dst as *mut AnnotTsvDat).cast());
    libc::free((*args).src.nannots_added.cast());
    libc::free((*args).src.core_idx.cast());
    libc::free((*args).dst.core_idx.cast());
    libc::free((*args).src.match_idx.cast());
    libc::free((*args).dst.match_idx.cast());
    libc::free((*args).src.transfer_idx.cast());
    libc::free((*args).dst.transfer_idx.cast());
    libc::free((*args).src.annots_idx.cast());
    libc::free((*args).dst.annots_idx.cast());
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

// original: write_string (htslib/annot-tsv.c:703)
pub unsafe fn annot_tsv_c_703_write_string(
    args: *mut c_void,
    mut str_: *mut c_char,
    mut len: usize,
) {
    let args = args.cast::<AnnotTsvArgs>();
    if len == 0 {
        len = libc::strlen(str_);
    }
    if len == 0 {
        str_ = c".".as_ptr().cast_mut();
        len = 1;
    }
    if crate::htslib_rs::bgzf::bgzf_write((*args).out_fp, str_.cast(), len) != len as isize {
        libc::abort();
    }
}

// original: write_annots (htslib/annot-tsv.c:709)
pub unsafe fn annot_tsv_c_709_write_annots(args: *mut c_void) {
    let args = args.cast::<AnnotTsvArgs>();
    if (*args).dst.annots.is_null() {
        return;
    }

    (*args).tmp_kstr.l = 0;
    let len = annot_tsv_c_168_nbp_length((*args).nbp.cast());
    for i in 0..(*(*args).dst.annots).n as usize {
        crate::htslib_rs::hts::kputc((*args).dst.delim as c_int, &mut (*args).tmp_kstr);
        let ann = *(*args).dst.annots_idx.add(i);
        if ann == ANNOT_TSV_ANN_NBP {
            crate::htslib_rs::hts::kputw(len as c_int, &mut (*args).tmp_kstr);
        } else if ann == ANNOT_TSV_ANN_FRAC {
            crate::htslib_rs::hts::kputd(
                len as f64 / ((*(*args).nbp).end - (*(*args).nbp).beg + 1) as f64,
                &mut (*args).tmp_kstr,
            );
        } else if ann == ANNOT_TSV_ANN_CNT {
            crate::htslib_rs::hts::kputw(
                ((*(*args).nbp).regs.len() / 2) as c_int,
                &mut (*args).tmp_kstr,
            );
        }
    }
    annot_tsv_c_703_write_string(args.cast(), (*args).tmp_kstr.s, (*args).tmp_kstr.l);
}

// original: process_line (htslib/annot-tsv.c:737)
pub unsafe fn annot_tsv_c_737_process_line(args: *mut c_void, line: *mut c_char, size: usize) {
    let args = args.cast::<AnnotTsvArgs>();
    let mut chr_beg: *mut c_char = std::ptr::null_mut();
    let mut chr_end: *mut c_char = std::ptr::null_mut();
    let mut beg: hts_pos_t = 0;
    let mut end: hts_pos_t = 0;
    let mut dst_cols: *mut AnnotTsvCols = std::ptr::null_mut();
    let ret = annot_tsv_c_276_parse_tab_with_payload(
        line,
        &mut chr_beg,
        &mut chr_end,
        &mut beg,
        &mut end,
        (&mut dst_cols as *mut *mut AnnotTsvCols).cast(),
        (&mut (*args).dst as *mut AnnotTsvDat).cast(),
    );
    if ret == -1 {
        annot_tsv_c_269_cols_destroy(dst_cols.cast());
        return;
    }

    if !(*args).nbp.is_null() {
        annot_tsv_c_142_nbp_reset((*args).nbp.cast(), beg, end);
    }

    if regidx::regidx_c_401_regidx_overlap((*args).idx, chr_beg, beg, end, (*args).itr) == 0 {
        if (*args).mode & ANNOT_TSV_PRINT_NONMATCHING != 0 {
            annot_tsv_c_703_write_string(args.cast(), line, size);
            annot_tsv_c_709_write_annots(args.cast());
            annot_tsv_c_703_write_string(args.cast(), c"\n".as_ptr().cast_mut(), 1);
        }
        annot_tsv_c_269_cols_destroy(dst_cols.cast());
        return;
    }

    for i in 0..(*(*args).src.transfer).n as usize {
        *(*args).src.nannots_added.add(i) = 0;
        (&mut (*(*args).tmp_cols))[i].n = 0;
        sam::khash_str2int_destroy(*(*args).tmp_hash.add(i));
        *(*args).tmp_hash.add(i) = sam::khash_str2int_init();
    }

    let mut has_match = 0;
    let mut annot_len: usize = 0;
    while regidx::regidx_c_612_regitr_overlap((*args).itr) != 0 {
        if (*args).overlap_src != 0.0 || (*args).overlap_dst != 0.0 {
            let len_dst = (end - beg + 1) as f64;
            let len_src = ((*(*args).itr).end - (*(*args).itr).beg + 1) as f64;
            let isec = (((*(*args).itr).end.min(end)) - ((*(*args).itr).beg.max(beg)) + 1) as f64;
            let pass_dst = (isec / len_dst >= (*args).overlap_dst) as c_int;
            let pass_src = (isec / len_src >= (*args).overlap_src) as c_int;
            if (*args).overlap_either != 0 {
                if pass_dst == 0 && pass_src == 0 {
                    continue;
                }
            } else if pass_dst == 0 || pass_src == 0 {
                continue;
            }
        }
        let src_cols = *(*(*args).itr).payload.cast::<*mut AnnotTsvCols>();
        if !(*args).dst.match_.is_null() && (*(*args).dst.match_).n != 0 {
            let mut i = 0usize;
            while i < (*(*args).dst.match_).n as usize {
                if *(*args).dst.match_idx.add(i) > (*dst_cols).n as c_int {
                    libc::abort();
                }
                let dst = *(*dst_cols).off.add(*(*args).dst.match_idx.add(i) as usize);
                let src = *(*src_cols).off.add(*(*args).src.match_idx.add(i) as usize);
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

        if !(*args).nbp.is_null() {
            annot_tsv_c_148_nbp_add(
                (*args).nbp.cast(),
                (*(*args).itr).beg.max(beg),
                (*(*args).itr).end.min(end),
            );
        }

        let mut max_annots_reached = 0;
        for i in 0..(*(*args).src.transfer).n as usize {
            let mut str_ = if *(*args).src.transfer_idx.add(i) >= 0 {
                *(*src_cols)
                    .off
                    .add(*(*args).src.transfer_idx.add(i) as usize)
            } else {
                *(*(*args).src.hdr.cols)
                    .off
                    .add((-*(*args).src.transfer_idx.add(i) - 1) as usize)
            };
            if str_.is_null() || *str_ == 0 {
                str_ = c".".as_ptr().cast_mut();
            }
            if (*args).allow_dups == 0 {
                if sam::khash_str2int_has_key(*(*args).tmp_hash.add(i), str_) != 0 {
                    continue;
                }
                sam::khash_str2int_set(*(*args).tmp_hash.add(i), str_, 1);
            }
            if (*args).max_annots != 0 {
                *(*args).src.nannots_added.add(i) += 1;
                if *(*args).src.nannots_added.add(i) >= (*args).max_annots {
                    max_annots_reached = 1;
                }
            }
            annot_tsv_c_217_cols_append(
                (&mut (&mut (*(*args).tmp_cols))[i] as *mut AnnotTsvCols).cast(),
                str_,
            );
            annot_len += libc::strlen(str_);
        }
        if max_annots_reached != 0 {
            break;
        }
    }

    if has_match == 0 {
        if (*args).mode & ANNOT_TSV_PRINT_NONMATCHING != 0 {
            annot_tsv_c_703_write_string(args.cast(), line, size);
            annot_tsv_c_709_write_annots(args.cast());
            annot_tsv_c_703_write_string(args.cast(), c"\n".as_ptr().cast_mut(), 1);
        }
        annot_tsv_c_269_cols_destroy(dst_cols.cast());
        return;
    }
    if (*args).mode & ANNOT_TSV_PRINT_MATCHING == 0 {
        annot_tsv_c_269_cols_destroy(dst_cols.cast());
        return;
    }

    (*args).tmp_kstr.l = 0;
    ks_resize(
        &mut (*args).tmp_kstr,
        annot_len * 3 + (*(*args).src.transfer).n as usize * 2,
    );
    for i in 0..(*(*args).src.transfer).n as usize {
        let mut off = (*args).tmp_kstr.s.add((*args).tmp_kstr.l);
        *(*dst_cols)
            .off
            .add(*(*args).dst.transfer_idx.add(i) as usize) = off;
        let ann = &mut (&mut (*(*args).tmp_cols))[i] as *mut AnnotTsvCols;
        if (*ann).n == 0 {
            *off = b'.' as c_char;
            *off.add(1) = 0;
            (*args).tmp_kstr.l += 2;
            continue;
        }
        for j in 0..(*ann).n as usize {
            if j > 0 {
                *off = b',' as c_char;
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
    annot_tsv_c_703_write_string(args.cast(), *(*dst_cols).off.add(0), 0);
    for i in 1..(*dst_cols).n as usize {
        annot_tsv_c_703_write_string(args.cast(), &mut (*args).dst.delim as *mut c_char, 1);
        annot_tsv_c_703_write_string(args.cast(), *(*dst_cols).off.add(i), 0);
    }
    annot_tsv_c_709_write_annots(args.cast());
    annot_tsv_c_703_write_string(args.cast(), c"\n".as_ptr().cast_mut(), 1);
    annot_tsv_c_269_cols_destroy(dst_cols.cast());
}

// original: usage_text (htslib/annot-tsv.c:880)
pub unsafe fn annot_tsv_c_880_usage_text() -> *const c_char {
    ANNOT_TSV_USAGE_TEXT.as_ptr().cast()
}

// original: main (htslib/annot-tsv.c:956)
pub unsafe fn annot_tsv_c_956_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let args = libc::calloc(1, std::mem::size_of::<AnnotTsvArgs>()).cast::<AnnotTsvArgs>();
    if args.is_null() {
        libc::abort();
    }
    let mut reciprocal = 0;
    let argv_slice = std::slice::from_raw_parts(argv, argc as usize);
    let mut i = 1usize;
    while i < argv_slice.len() {
        let arg = CStr::from_ptr(argv_slice[i]).to_bytes();
        let mut optarg: *mut c_char = std::ptr::null_mut();
        let c = if let Some(long) = arg.strip_prefix(b"--") {
            let (name, value) = match long.iter().position(|&ch| ch == b'=') {
                Some(eq) => (
                    &long[..eq],
                    Some(argv_slice[i].cast::<c_char>().add(2 + eq + 1)),
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
                        b"core" => b'c' as c_int,
                        b"coords" => b'C' as c_int,
                        b"transfer" => b'f' as c_int,
                        b"match" => b'm' as c_int,
                        b"output" => b'o' as c_int,
                        b"source-file" => b's' as c_int,
                        b"target-file" => b't' as c_int,
                        b"annotate" => b'a' as c_int,
                        b"headers" => b'h' as c_int,
                        b"overlap" => b'O' as c_int,
                        b"delim" => b'd' as c_int,
                        _ => libc::abort(),
                    }
                }
                b"no-header-idx" => b'I' as c_int,
                b"ignore-headers" => b'H' as c_int,
                b"reciprocal" => b'r' as c_int,
                b"drop-overlaps" => b'x' as c_int,
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
                            argv_slice[i].cast::<c_char>().add(pos + 1)
                        } else {
                            i += 1;
                            if i >= argv_slice.len() {
                                libc::abort();
                            }
                            argv_slice[i]
                        };
                        parsed = opt as c_int;
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
                let mut tmp: *mut c_char = std::ptr::null_mut();
                (*args).max_annots = libc::strtod(optarg, &mut tmp) as c_int;
                if tmp == optarg || *tmp != 0 {
                    libc::abort();
                }
            }
            x if x == b'I' as c_int => (*args).no_write_hdr += 1,
            x if x == b'd' as c_int => (*args).delim_str = optarg,
            x if x == b'h' as c_int => (*args).headers_str = optarg,
            x if x == b'H' as c_int => (*args).headers_str = c"0:0".as_ptr().cast_mut(),
            x if x == b'r' as c_int => reciprocal = 1,
            x if x == b'c' as c_int => (*args).core_str = optarg,
            x if x == b'C' as c_int => (*args).coords_str = optarg,
            x if x == b't' as c_int => (*args).dst.fname = optarg,
            x if x == b'm' as c_int => (*args).match_str = optarg,
            x if x == b'a' as c_int => (*args).annots_str = optarg,
            x if x == b'o' as c_int => (*args).out_fname = optarg,
            x if x == b'O' as c_int => {
                let mut tmp: *mut c_char = std::ptr::null_mut();
                (*args).overlap_src = libc::strtod(optarg, &mut tmp);
                if tmp == optarg || (*tmp != 0 && *tmp != b',' as c_char) {
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
            x if x == b's' as c_int => (*args).src.fname = optarg,
            x if x == b'f' as c_int => (*args).transfer_str = optarg,
            x if x == b'x' as c_int => (*args).mode = ANNOT_TSV_PRINT_NONMATCHING,
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
    annot_tsv_c_440_write_header(args.cast(), (&mut (*args).dst as *mut AnnotTsvDat).cast());
    while annot_tsv_c_471_read_next_line((&mut (*args).dst as *mut AnnotTsvDat).cast()) != 0 {
        for _ in 0..(*args).dst.grow_n {
            kputc((*args).dst.delim as c_int, &mut (*args).dst.line);
            kputc(b'.' as c_int, &mut (*args).dst.line);
        }
        annot_tsv_c_737_process_line(args.cast(), (*args).dst.line.s, (*args).dst.line.l);
        (*args).dst.line.l = 0;
    }
    annot_tsv_c_666_destroy_data(args.cast());
    libc::free(args.cast());
    0
}
