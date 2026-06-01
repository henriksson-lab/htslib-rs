//! Translation of htscodecs `fqzcomp_qual.c` + `fqzcomp_qual.h`.
//!
//! FQZComp quality-value codec: context-modelled adaptive arithmetic coding of
//! CRAM/BAM quality strings.
//!
//! Uses the range coder (`c_range_coder`) and the adaptive symbol model
//! (`c_simple_model`). The C unit instantiates `SIMPLE_MODEL` with `NSYM = 2`
//! and `NSYM = QMAX (256)`.

#![allow(unused_imports)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

use crate::htscodecs::c_range_coder::*;
use crate::htscodecs::c_simple_model::*;
use crate::htscodecs::utils::{fast_log, htscodecs_tls_alloc, htscodecs_tls_free};
use crate::htscodecs::varint::{var_get_u32, var_put_u32};

// C `INT_MAX` sentinel value stored in qmap[].  fqzcomp_qual.c:855 etc.
const INT_MAX_U32: u32 = i32::MAX as u32;

#[inline]
fn min_i32(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}
#[inline]
fn max_i32(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

//------------------------------------------------------------------------------
// fqzcomp_qual.h types and constants

// Bit flags, deliberately mirroring BAM ones
pub const FQZ_FREVERSE: i32 = 16; // fqzcomp_qual.h:44
pub const FQZ_FREAD2: i32 = 128; // fqzcomp_qual.h:45

// Current FQZ format version
pub const FQZ_VERS: i32 = 5; // fqzcomp_qual.h:48
pub const FQZ_MAX_STRAT: i32 = 3; // fqzcomp_qual.h:50

// Global flags
pub const GFLAG_MULTI_PARAM: i32 = 1; // fqzcomp_qual.h:67
pub const GFLAG_HAVE_STAB: i32 = 2; // fqzcomp_qual.h:68
pub const GFLAG_DO_REV: i32 = 4; // fqzcomp_qual.h:69

// Param flags
pub const PFLAG_DO_DEDUP: i32 = 2; // fqzcomp_qual.h:73
pub const PFLAG_DO_LEN: i32 = 4; // fqzcomp_qual.h:74
pub const PFLAG_DO_SEL: i32 = 8; // fqzcomp_qual.h:75
pub const PFLAG_HAVE_QMAP: i32 = 16; // fqzcomp_qual.h:76
pub const PFLAG_HAVE_PTAB: i32 = 32; // fqzcomp_qual.h:77
pub const PFLAG_HAVE_DTAB: i32 = 64; // fqzcomp_qual.h:78
pub const PFLAG_HAVE_QTAB: i32 = 128; // fqzcomp_qual.h:79

/// ```c
/// typedef struct {
///     int num_records;
///     uint32_t *len;    // of size num_records
///     uint32_t *flags;  // of size num_records
/// } fqz_slice;
/// ```
// fqzcomp_qual.h:59
#[repr(C)]
pub struct fqz_slice {
    pub num_records: i32,
    pub len: *mut u32,
    pub flags: *mut u32,
}

/// A single parameter block.
/// ```c
/// typedef struct {
///     uint16_t context;
///     unsigned int pflags;
///     unsigned int do_sel, do_dedup, store_qmap, fixed_len;
///     unsigned char use_qtab, use_dtab, use_ptab;
///     unsigned int qbits, qloc;
///     unsigned int pbits, ploc;
///     unsigned int dbits, dloc;
///     unsigned int sbits, sloc;
///     int max_sym, nsym, max_sel;
///     unsigned int qmap[256];
///     unsigned int qtab[256];
///     unsigned int ptab[1024];
///     unsigned int dtab[256];
///     int qshift;
///     int pshift;
///     int dshift;
///     int sshift;
///     unsigned int qmask; // (1<<qbits)-1
///     int do_r2, do_qa;
/// } fqz_param;
/// ```
// fqzcomp_qual.h:90
#[repr(C)]
pub struct fqz_param {
    pub context: u16,
    pub pflags: u32,
    pub do_sel: u32,
    pub do_dedup: u32,
    pub store_qmap: u32,
    pub fixed_len: u32,
    pub use_qtab: u8,
    pub use_dtab: u8,
    pub use_ptab: u8,
    pub qbits: u32,
    pub qloc: u32,
    pub pbits: u32,
    pub ploc: u32,
    pub dbits: u32,
    pub dloc: u32,
    pub sbits: u32,
    pub sloc: u32,
    pub max_sym: i32,
    pub nsym: i32,
    pub max_sel: i32,
    pub qmap: [u32; 256],
    pub qtab: [u32; 256],
    pub ptab: [u32; 1024],
    pub dtab: [u32; 256],
    pub qshift: i32,
    pub pshift: i32,
    pub dshift: i32,
    pub sshift: i32,
    pub qmask: u32,
    pub do_r2: i32,
    pub do_qa: i32,
}

/// Global params: a collection of parameter blocks plus meta-data.
/// ```c
/// typedef struct {
///     int vers;
///     unsigned int gflags;
///     int nparam;
///     int max_sel;
///     unsigned int stab[256];
///     int max_sym;
///     fqz_param *p;
/// } fqz_gparams;
/// ```
// fqzcomp_qual.h:126
#[repr(C)]
pub struct fqz_gparams {
    pub vers: i32,
    pub gflags: u32,
    pub nparam: i32,
    pub max_sel: i32,
    pub stab: [u32; 256],
    pub max_sym: i32,
    pub p: *mut fqz_param,
}

impl fqz_gparams {
    /// Zeroed gparams, matching the C `memset(gp, 0, sizeof(*gp))`.
    pub fn zeroed() -> Self {
        fqz_gparams {
            vers: 0,
            gflags: 0,
            nparam: 0,
            max_sel: 0,
            stab: [0u32; 256],
            max_sym: 0,
            p: std::ptr::null_mut(),
        }
    }
}

//------------------------------------------------------------------------------
// fqzcomp_qual.c internal constants and types

pub const CTX_BITS: i32 = 16; // fqzcomp_qual.c:73
pub const CTX_SIZE: i32 = 1 << CTX_BITS; // fqzcomp_qual.c:74
pub const QMAX: i32 = 256; // fqzcomp_qual.c:81
pub const QBITS: i32 = 12; // fqzcomp_qual.c:82
pub const QSIZE: i32 = 1 << QBITS; // fqzcomp_qual.c:83
pub const NP: i32 = 32; // fqzcomp_qual.c:397

/// ```c
/// typedef struct {
///     unsigned int qctx;
///     unsigned int p;
///     unsigned int delta;
///     unsigned int prevq;
///     unsigned int s;
///     unsigned int qtot, qlen;
///     unsigned int first_len;
///     unsigned int last_len;
///     ssize_t rec;
///     unsigned int ctx;
/// } fqz_state;
/// ```
// fqzcomp_qual.c:216
#[repr(C)]
pub struct fqz_state {
    pub qctx: u32,
    pub p: u32,
    pub delta: u32,
    pub prevq: u32,
    pub s: u32,
    pub qtot: u32,
    pub qlen: u32,
    pub first_len: u32,
    pub last_len: u32,
    pub rec: isize,
    pub ctx: u32,
}

impl fqz_state {
    /// Zeroed state, matching the C `fqz_state state = {0};` initialiser.
    pub fn zeroed() -> Self {
        fqz_state {
            qctx: 0,
            p: 0,
            delta: 0,
            prevq: 0,
            s: 0,
            qtot: 0,
            qlen: 0,
            first_len: 0,
            last_len: 0,
            rec: 0,
            ctx: 0,
        }
    }
}

/// ```c
/// typedef struct {
///     SIMPLE_MODEL(QMAX,_) *qual;
///     SIMPLE_MODEL(256,_)   len[4];
///     SIMPLE_MODEL(2,_)     revcomp;
///     SIMPLE_MODEL(256,_)   sel;
///     SIMPLE_MODEL(2,_)     dup;
/// } fqz_model;
/// ```
// fqzcomp_qual.c:312
pub struct fqz_model {
    pub qual: Vec<SimpleModel>, // SIMPLE_MODEL(QMAX,_) *qual (CTX_SIZE entries)
    pub len: [SimpleModel; 4],
    pub revcomp: SimpleModel,
    pub sel: SimpleModel,
    pub dup: SimpleModel,
}

impl fqz_model {
    /// Allocate an empty `fqz_model` shell.  Mirrors the C `fqz_model model;`
    /// stack declaration whose fields are populated by `fqz_create_models`.
    pub fn new() -> Self {
        fqz_model {
            qual: Vec::new(),
            len: [
                SimpleModel::new(256),
                SimpleModel::new(256),
                SimpleModel::new(256),
                SimpleModel::new(256),
            ],
            revcomp: SimpleModel::new(2),
            sel: SimpleModel::new(256),
            dup: SimpleModel::new(2),
        }
    }
}

impl Default for fqz_model {
    fn default() -> Self {
        Self::new()
    }
}

// static int strat_opts[][12]               // fqzcomp_qual.c:195
pub static STRAT_OPTS: [[i32; 12]; 5] = [
    [10, 5, 4, -1, 2, 1, 0, 14, 10, 14, 0, -1],
    [8, 5, 7, 0, 0, 0, 0, 14, 8, 14, 1, -1],
    [12, 6, 2, 0, 2, 3, 0, 9, 12, 14, 0, 0],
    [12, 6, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

// static int nstrats                        // fqzcomp_qual.c:203
pub static NSTRATS: i32 = 5;

//------------------------------------------------------------------------------
// Functions

/// `static int store_array(unsigned char *out, unsigned int *array, int size)`
// fqzcomp_qual.c:102
pub fn store_array(out: &mut [u8], array: &[u32], size: i32) -> i32 {
    let mut tmp = [0u8; 2048];

    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    while i < size {
        let run_start = i;
        while i < size && array[i as usize] == j as u32 {
            i += 1;
        }
        let mut run_len = i - run_start;

        let mut r: i32;
        loop {
            r = min_i32(255, run_len);
            tmp[k as usize] = r as u8;
            k += 1;
            run_len -= r;
            if r != 255 {
                break;
            }
        }
        j += 1;
    }
    // C source: `while (i < size) tmp[k++] = 0, j++;` — `j` is incremented
    // here but C immediately reuses `j` as the RLE loop counter so this last
    // increment is dead. Rust renames the second counter to `jj`, leaving
    // this `j` increment as a faithful but dead artifact.
    while i < size {
        tmp[k as usize] = 0;
        k += 1;
        #[allow(unused_assignments)]
        {
            j += 1;
        }
    }

    // RLE on out.
    let mut last: i32 = -1;
    let mut oi: i32 = 0;
    let mut jj: i32 = 0;
    while jj < k {
        out[oi as usize] = tmp[jj as usize];
        jj += 1;
        if out[oi as usize] as i32 == last {
            let n = jj;
            while jj < k && tmp[jj as usize] as i32 == last {
                jj += 1;
            }
            oi += 1;
            out[oi as usize] = (jj - n) as u8;
        } else {
            last = out[oi as usize] as i32;
        }
        oi += 1;
    }

    oi
}

/// `static int read_array(unsigned char *in, size_t in_size, unsigned int *array, int size)`
// fqzcomp_qual.c:146
pub fn read_array(r#in: &[u8], array: &mut [u32], size: i32) -> i32 {
    let in_size = r#in.len();
    let mut R = [0u8; 1024];
    let mut last: i32 = -1;

    let size = min_i32(1024, size);

    // Remove level one of run-len encoding
    let mut i: usize = 0;
    let mut j: i32 = 0;
    let mut z: i32 = 0;
    while z < size && i < in_size {
        let run = r#in[i] as i32;
        R[j as usize] = run as u8;
        j += 1;
        z += run;
        if run == last {
            if i + 1 >= in_size {
                return -1;
            }
            i += 1;
            let mut copy = r#in[i] as i32;
            z += run * copy;
            while {
                let c = copy;
                copy -= 1;
                c != 0
            } && z <= size
                && j < 1024
            {
                R[j as usize] = run as u8;
                j += 1;
            }
        }
        if j >= 1024 {
            return -1;
        }
        last = run;
        i += 1;
    }
    let nb = i as i32;

    // Now expand inner level of run-length encoding
    let R_max = j;
    let mut ii: i32 = 0;
    let mut jj: i32 = 0;
    let mut zz: i32 = 0;
    while jj < size {
        let mut run_len: i32 = 0;
        let mut run_part: i32;
        if zz >= R_max {
            return -1;
        }
        loop {
            run_part = R[zz as usize] as i32;
            zz += 1;
            run_len += run_part;
            if !(run_part == 255 && zz < R_max) {
                break;
            }
        }
        if run_part == 255 {
            return -1;
        }

        while run_len != 0 && jj < size {
            run_len -= 1;
            array[jj as usize] = ii as u32;
            jj += 1;
        }
        ii += 1;
    }

    nb
}

/// `static inline void mm_prefetch(void *x)`
// fqzcomp_qual.c:206
pub fn mm_prefetch(_x: *mut std::ffi::c_void) {
    // Fetch-and-discard prefetch hint; a no-op in safe Rust.
}

/// `static void dump_table(unsigned int *tab, int size, char *name)`
// fqzcomp_qual.c:229
/// Debug-only (stderr); never on the byte path.  Left as a no-op.
pub fn dump_table(_tab: &[u32], _size: i32, _name: &str) {}

/// `static void dump_map(unsigned int *map, int size, char *name)`
// fqzcomp_qual.c:264
/// Debug-only (stderr); never on the byte path.  Left as a no-op.
pub fn dump_map(_map: &[u32], _size: i32, _name: &str) {}

/// `static void dump_params(fqz_gparams *gp)`
// fqzcomp_qual.c:274
/// Debug-only (stderr); never on the byte path.  Left as a no-op.
pub fn dump_params(_gp: &fqz_gparams) {}

/// `static int fqz_create_models(fqz_model *m, fqz_gparams *gp)`
// fqzcomp_qual.c:320
///
/// Safety note (vs. C and vs. the rANS TLS-reuse hazard):
///
/// In the C source, `m->qual` is backed by `htscodecs_tls_alloc`, a per-thread
/// pool that returns the same buffer to successive callers and only `calloc`s
/// on first use.  The C path then unconditionally walks every slot calling
/// `SIMPLE_MODEL(QMAX,_init)`, so any stale bytes from a prior decode are
/// overwritten before use.
///
/// The Rust port replaces the TLS allocation with an owned `Vec<SimpleModel>`
/// that is freshly constructed (`vec![SymFreqs::default(); nsym + 3]` inside
/// `SimpleModel::new`, which zero-fills) on every call and dropped on exit
/// via `fqz_destroy_models`.  Every `SimpleModel` is then re-initialised
/// here.  There is therefore no cross-call stale-state hazard analogous to
/// the one fixed in `rans_static32x16pr` / `rans_static4x16pr`; the
/// decoder cannot read leftovers from a previous decode of a different
/// (possibly larger) parameter set.
pub fn fqz_create_models(m: &mut fqz_model, gp: &fqz_gparams) -> i32 {
    // m->qual = htscodecs_tls_alloc(sizeof(*m->qual) * CTX_SIZE)
    // Modelled with a Rust Vec of CTX_SIZE QMAX-symbol models.  Vec::push
    // of fresh SimpleModel values zero-initialises all SymFreqs before the
    // explicit SIMPLE_MODEL_init loop below runs, so even slots that the
    // init loop's `nsym`-bounded inner loop touches start from a known
    // state.
    m.qual = Vec::with_capacity(CTX_SIZE as usize);
    for _ in 0..CTX_SIZE {
        m.qual.push(SimpleModel::new(QMAX as usize));
    }

    for i in 0..CTX_SIZE as usize {
        SIMPLE_MODEL_init(&mut m.qual[i], gp.max_sym + 1);
    }

    for i in 0..4 {
        SIMPLE_MODEL_init(&mut m.len[i], 256);
    }

    SIMPLE_MODEL_init(&mut m.revcomp, 2);
    SIMPLE_MODEL_init(&mut m.dup, 2);
    if gp.max_sel > 0 {
        SIMPLE_MODEL_init(&mut m.sel, gp.max_sel + 1);
    }

    0
}

/// `static void fqz_destroy_models(fqz_model *m)`
// fqzcomp_qual.c:340
pub fn fqz_destroy_models(m: &mut fqz_model) {
    // htscodecs_tls_free(m->qual): the Vec is owned, dropping it frees it.
    m.qual = Vec::new();
}

/// `static inline unsigned int fqz_update_ctx(fqz_param *pm, fqz_state *state, int q)`
// fqzcomp_qual.c:344
pub fn fqz_update_ctx(pm: &fqz_param, state: &mut fqz_state, q: i32) -> u32 {
    let mut last: u32 = 0; // pm->context
    state.qctx = (state.qctx << pm.qshift).wrapping_add(pm.qtab[q as usize]);
    last = last.wrapping_add((state.qctx & pm.qmask) << pm.qloc);

    // The final shifts have been factored into the tables already.
    last = last.wrapping_add(pm.ptab[min_i32(1023, state.p as i32) as usize]); // << pm->ploc
    last = last.wrapping_add(pm.dtab[min_i32(255, state.delta as i32) as usize]); // << pm->dloc
    last = last.wrapping_add(state.s << pm.sloc);

    // Only update delta after 1st base.
    state.delta += (state.prevq != q as u32) as u32;
    state.prevq = q as u32;

    state.p = state.p.wrapping_sub(1);

    last & (CTX_SIZE as u32 - 1)
}

/// `void fqz_qual_stats(fqz_slice *s, unsigned char *in, size_t in_size,
///                      fqz_param *pm, uint32_t qhist[256], int one_param)`
// fqzcomp_qual.c:392
///
/// `s` is `&mut` because the READ1/2 and quality-average auto-tuning steps stash
/// a selector index in the top 16 bits of `s->flags[]` (C mutates the slice).
pub fn fqz_qual_stats(
    s: &mut fqz_slice,
    r#in: &[u8],
    pm: &mut fqz_param,
    qhist: &mut [u32; 256],
    one_param: i32,
) {
    let in_size = r#in.len();
    let num_records = s.num_records;
    // s->len[] and s->flags[] views.
    let s_len = unsafe { std::slice::from_raw_parts(s.len, num_records as usize) };
    let s_flags = unsafe { std::slice::from_raw_parts_mut(s.flags, num_records as usize) };

    // qhistb[NP][256], qhist1, qhist2
    let mut qhistb = vec![[0u32; 256]; NP as usize];
    let mut qhist1 = vec![[0u32; 256]; NP as usize];
    let mut qhist2 = vec![[0u32; 256]; NP as usize];
    let mut t1 = [0u64; NP as usize];
    let mut t2 = [0u64; NP as usize];
    let mut avg = [0u32; 2560];

    let mut dir;
    let mut last_len: i32 = 0;
    let mut do_dedup: i32 = 0;
    let mut rec: usize;
    let mut i: usize;
    let mut j: usize;

    // See what info we've been given.
    let mut max_sel: i32 = 0;
    let mut has_r2: i32 = 0;
    let mut num_rec: i32 = 0;
    for rec in 0..num_records as usize {
        if one_param >= 0 && (s_flags[rec] >> 16) as i32 != one_param {
            continue;
        }
        num_rec += 1;
        if max_sel < (s_flags[rec] >> 16) as i32 {
            max_sel = (s_flags[rec] >> 16) as i32;
        }
        if s_flags[rec] & FQZ_FREAD2 as u32 != 0 {
            has_r2 = 1;
        }
    }

    // Dedup detection and histogram stats gathering
    let mut avg_qual = vec![0i32; (num_records + 1) as usize];

    rec = 0;
    i = 0;
    while i < in_size {
        if one_param >= 0
            && rec < num_records as usize
            && (s_flags[rec] >> 16) as i32 != one_param
        {
            avg_qual[rec] = 0;
            i += s_len[rec] as usize;
            rec += 1;
            continue;
        }
        if rec < num_records as usize {
            j = s_len[rec] as usize;
            dir = if s_flags[rec] & FQZ_FREAD2 as u32 != 0 { 1 } else { 0 };
            if i > 0
                && j == last_len as usize
                && r#in[i - last_len as usize..i - last_len as usize + j] == r#in[i..i + j]
            {
                do_dedup += 1;
            }
        } else {
            j = in_size - i;
            dir = 0;
        }
        last_len = j as i32;

        let mut tot: u32 = 0;
        while i < in_size && j > 0 {
            let q = r#in[i] as usize;
            tot += q as u32;
            qhist[q] += 1;
            qhistb[j & (NP as usize - 1)][q] += 1;
            if dir != 0 {
                qhist2[j & (NP as usize - 1)][q] += 1;
                t2[j & (NP as usize - 1)] += 1;
            } else {
                qhist1[j & (NP as usize - 1)][q] += 1;
                t1[j & (NP as usize - 1)] += 1;
            }
            i += 1;
            j -= 1;
        }
        let tot = if last_len != 0 {
            (tot as f64 * 10.0 / last_len as f64 + 0.5) as u32
        } else {
            0
        };

        avg_qual[rec] = tot as i32;
        avg[min_i32(2559, tot as i32) as usize] += 1;

        rec += 1;
    }
    pm.do_dedup = ((rec as i32 + 1) / (do_dedup + 1) < 500) as u32;

    // C source: `last_len = 0;` — never read again in this function.
    // Faithful but dead translation artifact.
    #[allow(unused_assignments)]
    {
        last_len = 0;
    }

    // Unique symbol count
    pm.max_sym = 0;
    pm.nsym = 0;
    for i in 0..256usize {
        if qhist[i] != 0 {
            pm.max_sym = i as i32;
            pm.nsym += 1;
        }
    }

    // Auto tune: does average quality help us?
    if pm.do_qa != 0 {
        let qf0 = if pm.nsym > 8 { 0.2 } else { 0.05 };
        let qf1 = if pm.nsym > 8 { 0.5 } else { 0.22 };
        let qf2 = if pm.nsym > 8 { 0.8 } else { 0.60 };

        let mut total: i32 = 0;
        let mut i: usize = 0;
        while i < 2560 {
            total += avg[i] as i32;
            if total as f64 > qf0 * num_rec as f64 {
                break;
            }
            avg[i] = 0;
            i += 1;
        }
        while i < 2560 {
            total += avg[i] as i32;
            if total as f64 > qf1 * num_rec as f64 {
                break;
            }
            avg[i] = 1;
            i += 1;
        }
        while i < 2560 {
            total += avg[i] as i32;
            if total as f64 > qf2 * num_rec as f64 {
                break;
            }
            avg[i] = 2;
            i += 1;
        }
        while i < 2560 {
            avg[i] = 3;
            i += 1;
        }

        // Compute simple entropy of merged signal vs split signal.
        let mut i: usize = 0;
        let mut rec: usize = 0;

        let mut qbin4 = vec![[[0i32; 256]; NP as usize]; 4];
        let mut qbin2 = vec![[[0i32; 256]; NP as usize]; 2];
        let mut qbin1 = vec![[0i32; 256]; NP as usize];
        let mut qcnt4 = vec![[0i32; NP as usize]; 4];
        let mut qcnt2 = vec![[0i32; NP as usize]; 4];
        let mut qcnt1 = [0i32; NP as usize];
        while i < in_size {
            if one_param >= 0
                && rec < num_records as usize
                && (s_flags[rec] >> 16) as i32 != one_param
            {
                i += s_len[rec] as usize;
                rec += 1;
                continue;
            }
            if (rec & 7) != 0 && rec < num_records as usize {
                // subsample for speed
                i += s_len[rec] as usize;
                rec += 1;
                continue;
            }
            let jlen = if rec < num_records as usize {
                s_len[rec] as usize
            } else {
                in_size - i
            };
            let mut j = jlen;
            // C source: `last_len = j;` — written but not read again in the
            // remainder of this function. Faithful but dead translation
            // artifact.
            #[allow(unused_assignments)]
            {
                last_len = jlen as i32;
            }

            let tot = avg_qual[rec] as u32;
            let qb4 = avg[min_i32(2559, tot as i32) as usize] as usize;
            let qb2 = qb4 / 2;

            while i < in_size && j > 0 {
                let x = j & (NP as usize - 1);
                let q = r#in[i] as usize;
                qbin4[qb4][x][q] += 1;
                qcnt4[qb4][x] += 1;
                qbin2[qb2][x][q] += 1;
                qcnt2[qb2][x] += 1;
                qbin1[x][q] += 1;
                qcnt1[x] += 1;
                i += 1;
                j -= 1;
            }
            rec += 1;
        }

        let mut e1: f64 = 0.0;
        let mut e2: f64 = 0.0;
        let mut e4: f64 = 0.0;
        for j in 0..NP as usize {
            for i in 0..256usize {
                if qbin1[j][i] != 0 {
                    e1 += qbin1[j][i] as f64
                        * fast_log(qbin1[j][i] as f64 / qcnt1[j] as f64);
                }
                if qbin2[0][j][i] != 0 {
                    e2 += qbin2[0][j][i] as f64
                        * fast_log(qbin2[0][j][i] as f64 / qcnt2[0][j] as f64);
                }
                if qbin2[1][j][i] != 0 {
                    e2 += qbin2[1][j][i] as f64
                        * fast_log(qbin2[1][j][i] as f64 / qcnt2[1][j] as f64);
                }
                if qbin4[0][j][i] != 0 {
                    e4 += qbin4[0][j][i] as f64
                        * fast_log(qbin4[0][j][i] as f64 / qcnt4[0][j] as f64);
                }
                if qbin4[1][j][i] != 0 {
                    e4 += qbin4[1][j][i] as f64
                        * fast_log(qbin4[1][j][i] as f64 / qcnt4[1][j] as f64);
                }
                if qbin4[2][j][i] != 0 {
                    e4 += qbin4[2][j][i] as f64
                        * fast_log(qbin4[2][j][i] as f64 / qcnt4[2][j] as f64);
                }
                if qbin4[3][j][i] != 0 {
                    e4 += qbin4[3][j][i] as f64
                        * fast_log(qbin4[3][j][i] as f64 / qcnt4[3][j] as f64);
                }
            }
        }
        e1 /= -(2f64.ln()) / 8.0;
        e2 /= -(2f64.ln()) / 8.0;
        e4 /= -(2f64.ln()) / 8.0;

        let qm = if pm.do_qa > 0 { 1.0 } else { 0.98 };
        if (pm.do_qa == -1 || pm.do_qa >= 4)
            && e4 + num_records as f64 / 4.0 < e2 * qm + num_records as f64 / 8.0
            && e4 + num_records as f64 / 4.0 < e1 * qm
        {
            for i in 0..num_records as usize {
                s_flags[i] |= avg[min_i32(2559, avg_qual[i]) as usize] << 16;
            }
            pm.do_sel = 1;
            max_sel = 3;
        } else if (pm.do_qa == -1 || pm.do_qa >= 2)
            && e2 + num_records as f64 / 8.0 < e1 * qm
        {
            for i in 0..num_records as usize {
                s_flags[i] |= (avg[min_i32(2559, avg_qual[i]) as usize] >> 1) << 16;
            }
            pm.do_sel = 1;
            max_sel = 1;
        }

        if pm.do_qa == -1 {
            // assume qual, pos, delta in that order.
            if pm.pbits > 0 && pm.dbits > 0 {
                pm.sloc = pm.dloc - 1;
                pm.pbits -= 1;
                pm.dbits -= 1;
                pm.dloc += 1;
            } else if pm.dbits >= 2 {
                pm.sloc = pm.dloc;
                pm.dbits -= 2;
                pm.dloc += 2;
            } else if pm.qbits >= 2 {
                pm.qbits -= 2;
                pm.ploc -= 2;
                pm.sloc = (16 - 2 - pm.do_r2) as u32;
                if pm.qbits == 6 && pm.qshift == 5 {
                    pm.qbits -= 1;
                }
            }
            pm.do_qa = 4;
        }
    }

    // Auto tune: does splitting up READ1 and READ2 help us?
    if has_r2 != 0 || pm.do_r2 != 0 {
        let mut e1: f64 = 0.0;
        let mut e2: f64 = 0.0;

        for j in 0..NP as usize {
            if t1[j] == 0 || t2[j] == 0 {
                continue;
            }
            for i in 0..256usize {
                if qhistb[j][i] == 0 {
                    continue;
                }
                e1 -= qhistb[j][i] as f64
                    * (qhistb[j][i] as f64 / (t1[j] + t2[j]) as f64).ln();
                if qhist1[j][i] != 0 {
                    e2 -= qhist1[j][i] as f64 * (qhist1[j][i] as f64 / t1[j] as f64).ln();
                }
                if qhist2[j][i] != 0 {
                    e2 -= qhist2[j][i] as f64 * (qhist2[j][i] as f64 / t2[j] as f64).ln();
                }
            }
        }
        e1 /= 2f64.ln() * 8.0;
        e2 /= 2f64.ln() * 8.0;

        let qm = if pm.do_r2 > 0 { 1.0 } else { 0.95 };
        if e2 + (8.0 + num_records as f64 / 8.0) < e1 * qm {
            for rec in 0..num_records as usize {
                if one_param >= 0 && (s_flags[rec] >> 16) as i32 != one_param {
                    continue;
                }
                let sel = (s_flags[rec] >> 16) as i32;
                s_flags[rec] = (s_flags[rec] & 0xffff)
                    | if s_flags[rec] & FQZ_FREAD2 as u32 != 0 {
                        (((sel * 2) + 1) as u32) << 16
                    } else {
                        (((sel * 2) + 0) as u32) << 16
                    };
                if max_sel < (s_flags[rec] >> 16) as i32 {
                    max_sel = (s_flags[rec] >> 16) as i32;
                }
            }
        }
    }

    // We provided explicit selector data or auto-tuned it
    if max_sel > 0 {
        pm.do_sel = 1;
        pm.max_sel = max_sel;
    }
    let _ = &mut avg_qual;
}

/// `int fqz_store_parameters1(fqz_param *pm, unsigned char *comp)`
// fqzcomp_qual.c:675
pub fn fqz_store_parameters1(pm: &fqz_param, comp: &mut [u8]) -> i32 {
    let mut comp_idx: usize = 0;

    // Starting context
    comp[comp_idx] = pm.context as u8;
    comp_idx += 1;
    comp[comp_idx] = (pm.context >> 8) as u8;
    comp_idx += 1;

    comp[comp_idx] = pm.pflags as u8;
    comp_idx += 1;
    comp[comp_idx] = pm.max_sym as u8;
    comp_idx += 1;

    comp[comp_idx] = ((pm.qbits << 4) | pm.qshift as u32) as u8;
    comp_idx += 1;
    comp[comp_idx] = ((pm.qloc << 4) | pm.sloc) as u8;
    comp_idx += 1;
    comp[comp_idx] = ((pm.ploc << 4) | pm.dloc) as u8;
    comp_idx += 1;

    if pm.store_qmap != 0 {
        for i in 0..256usize {
            if pm.qmap[i] != INT_MAX_U32 {
                comp[comp_idx] = i as u8;
                comp_idx += 1;
            }
        }
    }

    if pm.qbits != 0 && pm.use_qtab != 0 {
        comp_idx += store_array(&mut comp[comp_idx..], &pm.qtab, 256) as usize;
    }

    if pm.pbits != 0 && pm.use_ptab != 0 {
        comp_idx += store_array(&mut comp[comp_idx..], &pm.ptab, 1024) as usize;
    }

    if pm.dbits != 0 && pm.use_dtab != 0 {
        comp_idx += store_array(&mut comp[comp_idx..], &pm.dtab, 256) as usize;
    }

    comp_idx as i32
}

/// `int fqz_store_parameters(fqz_gparams *gp, unsigned char *comp)`
// fqzcomp_qual.c:711
pub fn fqz_store_parameters(gp: &fqz_gparams, comp: &mut [u8]) -> i32 {
    let mut comp_idx: usize = 0;
    comp[comp_idx] = gp.vers as u8;
    comp_idx += 1;

    comp[comp_idx] = gp.gflags as u8;
    comp_idx += 1;

    if gp.gflags & GFLAG_MULTI_PARAM as u32 != 0 {
        comp[comp_idx] = gp.nparam as u8;
        comp_idx += 1;
    }

    if gp.gflags & GFLAG_HAVE_STAB as u32 != 0 {
        comp[comp_idx] = gp.max_sel as u8;
        comp_idx += 1;
        comp_idx += store_array(&mut comp[comp_idx..], &gp.stab, 256) as usize;
    }

    let gp_p = unsafe { std::slice::from_raw_parts(gp.p, gp.nparam as usize) };
    for i in 0..gp.nparam as usize {
        comp_idx += fqz_store_parameters1(&gp_p[i], &mut comp[comp_idx..]) as usize;
    }

    comp_idx as i32
}

/// `int fqz_pick_parameters(fqz_gparams *gp, int vers, int strat,
///                          fqz_slice *s, unsigned char *in, size_t in_size)`
// fqzcomp_qual.c:736
///
/// `s` is `&mut` because length-clamping and the selector auto-tuning inside
/// `fqz_qual_stats` mutate `s->len[]` / `s->flags[]`.
pub fn fqz_pick_parameters(
    gp: &mut fqz_gparams,
    vers: i32,
    strat: i32,
    s: &mut fqz_slice,
    r#in: &[u8],
) -> i32 {
    let in_size = r#in.len();
    // approx sqrt(delta), must be sequential
    let mut dsqr: [i32; 64] = [
        0, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7,
    ];
    let mut qhist = [0u32; 256];

    let mut strat = strat;
    if strat >= NSTRATS {
        strat = NSTRATS - 1;
    }

    // Start with 1 set of parameters.
    // memset(gp, 0, sizeof(*gp));
    gp.vers = 0;
    gp.gflags = 0;
    gp.nparam = 0;
    gp.max_sel = 0;
    gp.stab = [0u32; 256];
    gp.max_sym = 0;
    gp.p = std::ptr::null_mut();

    gp.vers = FQZ_VERS;

    let p_ptr = unsafe { crate::c_compat::calloc(1, std::mem::size_of::<fqz_param>() as u64) }
        as *mut fqz_param;
    if p_ptr.is_null() {
        return -1;
    }
    gp.p = p_ptr;
    gp.nparam = 1;
    gp.max_sel = 0;

    if vers == 3 {
        gp.gflags |= GFLAG_DO_REV as u32;
    }

    let pm: &mut fqz_param = unsafe { &mut *gp.p };

    // Programmed strategies.
    pm.qbits = STRAT_OPTS[strat as usize][0] as u32;
    pm.qshift = STRAT_OPTS[strat as usize][1];
    pm.pbits = STRAT_OPTS[strat as usize][2] as u32;
    pm.pshift = STRAT_OPTS[strat as usize][3];
    pm.dbits = STRAT_OPTS[strat as usize][4] as u32;
    pm.dshift = STRAT_OPTS[strat as usize][5];
    pm.qloc = STRAT_OPTS[strat as usize][6] as u32;
    pm.sloc = STRAT_OPTS[strat as usize][7] as u32;
    pm.ploc = STRAT_OPTS[strat as usize][8] as u32;
    pm.dloc = STRAT_OPTS[strat as usize][9] as u32;

    pm.do_r2 = STRAT_OPTS[strat as usize][10];
    pm.do_qa = STRAT_OPTS[strat as usize][11];

    // Validity check input lengths and buffer size.
    {
        let num_records = s.num_records;
        let s_len = unsafe { std::slice::from_raw_parts_mut(s.len, num_records as usize) };
        let mut tlen: usize = 0;
        for i in 0..num_records as usize {
            if tlen + s_len[i] as usize > in_size {
                s_len[i] = (in_size - tlen) as u32;
            }
            tlen += s_len[i] as usize;
        }
        if num_records > 0 && tlen < in_size {
            s_len[(num_records - 1) as usize] += (in_size - tlen) as u32;
        }
    }

    // Quality metrics, for all recs.
    fqz_qual_stats(s, r#in, pm, &mut qhist, -1);

    pm.store_qmap = (pm.nsym <= 8 && pm.nsym * 2 < pm.max_sym) as u32;

    // Check for fixed length.
    let num_records = s.num_records;
    let s_len = unsafe { std::slice::from_raw_parts(s.len, num_records as usize) };
    let first_len = s_len[0];
    let mut i: usize = 1;
    while i < num_records as usize {
        if s_len[i] != first_len {
            break;
        }
        i += 1;
    }
    pm.fixed_len = (i == num_records as usize) as u32;
    pm.use_qtab = 0;

    let mut manually_set = false;
    if strat >= NSTRATS - 1 {
        manually_set = true;
    }

    if !manually_set {
        if pm.pshift < 0 {
            pm.pshift = max_i32(
                0,
                (((s_len[0] as f64 / (1u32 << pm.pbits) as f64).ln() / 2f64.ln()) + 0.5) as i32,
            );
        }

        if pm.nsym <= 4 {
            // NovaSeq
            pm.qshift = 2;
            if in_size < 5000000 {
                pm.pbits = 2;
                pm.pshift = 5;
            }
        } else if pm.nsym <= 8 {
            // HiSeqX
            pm.qbits = min_i32(pm.qbits as i32, 9) as u32;
            pm.qshift = 3;
            if in_size < 5000000 {
                pm.qbits = 6;
            }
        }

        if in_size < 300000 {
            pm.qbits = pm.qshift as u32;
            pm.dbits = 2;
        }
    }

    // manually_set:
    for i in 0..dsqr.len() {
        if dsqr[i] > (1 << pm.dbits) - 1 {
            dsqr[i] = (1 << pm.dbits) - 1;
        }
    }

    if pm.store_qmap != 0 {
        let mut j: i32 = 0;
        for i in 0..256usize {
            if qhist[i] != 0 {
                pm.qmap[i] = j as u32;
                j += 1;
            } else {
                pm.qmap[i] = INT_MAX_U32;
            }
        }
        pm.max_sym = pm.nsym;
    } else {
        pm.nsym = 255;
        for i in 0..256usize {
            pm.qmap[i] = i as u32;
        }
    }
    if gp.max_sym < pm.max_sym {
        gp.max_sym = pm.max_sym;
    }

    // Produce ptab from pshift.
    if pm.qbits != 0 {
        for i in 0..256usize {
            pm.qtab[i] = i as u32; // 1:1
        }
    }
    pm.qmask = (1 << pm.qbits) - 1;

    if pm.pbits != 0 {
        for i in 0..1024usize {
            pm.ptab[i] = min_i32((1 << pm.pbits) - 1, (i >> pm.pshift) as i32) as u32;
        }
    }

    if pm.dbits != 0 {
        for i in 0..256usize {
            pm.dtab[i] = dsqr[min_i32(dsqr.len() as i32 - 1, (i >> pm.dshift) as i32) as usize]
                as u32;
        }
    }

    pm.use_ptab = (pm.pbits > 0) as u8;
    pm.use_dtab = (pm.dbits > 0) as u8;

    pm.pflags = (if pm.use_qtab != 0 { PFLAG_HAVE_QTAB } else { 0 } as u32)
        | (if pm.use_dtab != 0 { PFLAG_HAVE_DTAB } else { 0 } as u32)
        | (if pm.use_ptab != 0 { PFLAG_HAVE_PTAB } else { 0 } as u32)
        | (if pm.do_sel != 0 { PFLAG_DO_SEL } else { 0 } as u32)
        | (if pm.fixed_len != 0 { PFLAG_DO_LEN } else { 0 } as u32)
        | (if pm.do_dedup != 0 { PFLAG_DO_DEDUP } else { 0 } as u32)
        | (if pm.store_qmap != 0 { PFLAG_HAVE_QMAP } else { 0 } as u32);

    gp.max_sel = 0;
    if pm.do_sel != 0 {
        gp.max_sel = 1; // indicator to check recs
        gp.gflags |= GFLAG_HAVE_STAB as u32;
        // NB: stab is already all zero
    }

    if gp.max_sel != 0 && num_records != 0 {
        let s_flags = unsafe { std::slice::from_raw_parts(s.flags, num_records as usize) };
        let mut max: i32 = 0;
        for i in 0..num_records as usize {
            if max < (s_flags[i] >> 16) as i32 {
                max = (s_flags[i] >> 16) as i32;
            }
        }
        gp.max_sel = max;
    }

    0
}

/// `static void fqz_free_parameters(fqz_gparams *gp)`
// fqzcomp_qual.c:926
pub fn fqz_free_parameters(gp: &mut fqz_gparams) {
    if !gp.p.is_null() {
        unsafe { crate::c_compat::free(gp.p as *mut std::ffi::c_void) };
        gp.p = std::ptr::null_mut();
    }
}

/// `static int compress_new_read(fqz_slice *s, fqz_state *state, fqz_gparams *gp,
///     fqz_param *pm, fqz_model *model, RangeCoder *rc, unsigned char *in,
///     size_t *in_i, unsigned int *last)`
// fqzcomp_qual.c:930
pub fn compress_new_read(
    s: &fqz_slice,
    state: &mut fqz_state,
    gp: &fqz_gparams,
    _pm: &fqz_param,
    model: &mut fqz_model,
    rc: &mut RangeCoder,
    r#in: &[u8],
    in_i: &mut usize,
    last: &mut u32,
) -> i32 {
    let rec = state.rec;
    let mut i = *in_i;
    let s_len = unsafe { std::slice::from_raw_parts(s.len, s.num_records as usize) };
    let s_flags = unsafe { std::slice::from_raw_parts(s.flags, s.num_records as usize) };

    if _pm.do_sel != 0 || (gp.gflags & GFLAG_MULTI_PARAM as u32) != 0 {
        state.s = if rec < s.num_records as isize {
            s_flags[rec as usize] >> 16
        } else {
            0
        };
        SIMPLE_MODEL_encodeSymbol(&mut model.sel, rc, state.s as u16);
    } else {
        state.s = 0;
    }
    let x = if gp.gflags & GFLAG_HAVE_STAB as u32 != 0 {
        gp.stab[state.s as usize]
    } else {
        state.s
    };
    let pm: &fqz_param = unsafe { &*gp.p.add(x as usize) };

    let len = s_len[rec as usize];
    if pm.fixed_len == 0 || state.first_len != 0 {
        SIMPLE_MODEL_encodeSymbol(&mut model.len[0], rc, ((len >> 0) & 0xff) as u16);
        SIMPLE_MODEL_encodeSymbol(&mut model.len[1], rc, ((len >> 8) & 0xff) as u16);
        SIMPLE_MODEL_encodeSymbol(&mut model.len[2], rc, ((len >> 16) & 0xff) as u16);
        SIMPLE_MODEL_encodeSymbol(&mut model.len[3], rc, ((len >> 24) & 0xff) as u16);
        state.first_len = 0;
    }

    if gp.gflags & GFLAG_DO_REV as u32 != 0 {
        if s_flags[rec as usize] & FQZ_FREVERSE as u32 != 0 {
            SIMPLE_MODEL_encodeSymbol(&mut model.revcomp, rc, 1);
        } else {
            SIMPLE_MODEL_encodeSymbol(&mut model.revcomp, rc, 0);
        }
    }

    state.rec += 1;

    state.qtot = 0;
    state.qlen = 0;

    state.p = len;
    state.delta = 0;
    state.qctx = 0;
    state.prevq = 0;

    *last = pm.context as u32;

    if pm.do_dedup != 0 {
        // Possible dup of previous read?
        if i != 0
            && len == state.last_len
            && r#in[i - state.last_len as usize..i - state.last_len as usize + len as usize]
                == r#in[i..i + len as usize]
        {
            SIMPLE_MODEL_encodeSymbol(&mut model.dup, rc, 1);
            i += (len - 1) as usize;
            state.p = 0;
            *in_i = i;
            return 1; // is a dup
        } else {
            SIMPLE_MODEL_encodeSymbol(&mut model.dup, rc, 0);
        }

        state.last_len = len;
    }

    *in_i = i;

    0
}

/// `unsigned char *compress_block_fqz2f(int vers, int strat, fqz_slice *s,
///     unsigned char *in, size_t in_size, size_t *out_size, fqz_gparams *gp)`
// fqzcomp_qual.c:1004
/// Returns an empty `Vec` to model the C `NULL` failure return.
/// `s` and `in` are `&mut`: the encoder reverses-complements quality runs
/// in-place for `GFLAG_DO_REV` and stashes/clears selector bits in `s->flags`.
/// `gp_opt` models the optional `fqz_gparams *gp` (None == C `NULL`).
pub fn compress_block_fqz2f(
    vers: i32,
    strat: i32,
    s: &mut fqz_slice,
    r#in: &mut [u8],
    out_size: &mut usize,
    gp_opt: Option<&mut fqz_gparams>,
) -> Vec<u8> {
    let in_size = r#in.len();
    let mut local_gp = fqz_gparams::zeroed();
    let mut free_params = false;

    let mut last: u32 = 0;

    let mut comp_idx: usize;
    let mut rc = RangeCoder::new();

    // Pick and store params
    let gp: &mut fqz_gparams = match gp_opt {
        Some(g) => g,
        None => {
            if fqz_pick_parameters(&mut local_gp, vers, strat, s, r#in) < 0 {
                return Vec::new();
            }
            free_params = true;
            &mut local_gp
        }
    };

    // Worst-case sizing
    let mut sel_bits: i32 = 0;
    let mut sel = gp.max_sel;
    while sel != 0 {
        sel_bits += 1;
        sel >>= 1;
    }
    let gp_p0_fixed_len = unsafe { (*gp.p).fixed_len };
    let mut len_sz: f64 = if gp_p0_fixed_len != 0 { 0.25 } else { 4.25 };
    len_sz += sel_bits as f64 / 8.0;
    let comp_sz: usize =
        ((s.num_records as f64 * len_sz + in_size as f64) * 1.1) as usize + 10000;

    // malloc(comp_sz) modelled with a zeroed Vec of that capacity/length.
    let mut comp: Vec<u8> = vec![0u8; comp_sz];
    let comp_ptr = comp.as_mut_ptr();

    // comp_idx = var_put_u32(comp, compe, in_size)
    comp_idx = var_put_u32(&mut comp, Some(comp_sz), in_size as u32) as usize;
    comp_idx += fqz_store_parameters(gp, &mut comp[comp_idx..]) as usize;

    // Optimise tables to remove shifts in loop.
    {
        let gp_p = unsafe { std::slice::from_raw_parts_mut(gp.p, gp.nparam as usize) };
        for jp in 0..gp.nparam as usize {
            let pm = &mut gp_p[jp];
            for i in 0..1024usize {
                pm.ptab[i] <<= pm.ploc;
            }
            for i in 0..256usize {
                pm.dtab[i] <<= pm.dloc;
            }
        }
    }

    // Create models and initialise range coder.
    let mut model = fqz_model::new();
    if fqz_create_models(&mut model, gp) < 0 {
        return Vec::new();
    }

    unsafe {
        RC_SetOutput(&mut rc, comp_ptr.add(comp_idx));
        RC_SetOutputEnd(&mut rc, comp_ptr.add(comp_sz));
    }
    RC_StartEncode(&mut rc);

    let num_records = s.num_records;
    let s_len = unsafe { std::slice::from_raw_parts(s.len, num_records as usize) };
    let s_flags = unsafe { std::slice::from_raw_parts(s.flags, num_records as usize) };

    // For CRAM3.1, reverse upfront if needed.
    if gp.gflags & GFLAG_DO_REV as u32 != 0 {
        let mut i: usize = 0;
        let mut rec: isize = 0;
        while i < in_size {
            let len = if rec < num_records as isize - 1 {
                s_len[rec as usize] as usize
            } else {
                in_size - i
            };
            if s_flags[rec as usize] & FQZ_FREVERSE as u32 != 0 {
                let (mut bi, mut bj) = (0usize, len - 1);
                while bi < bj {
                    r#in.swap(i + bi, i + bj);
                    bi += 1;
                    bj -= 1;
                }
            }
            i += len;
            rec += 1;
        }
    }

    let mut state = fqz_state::zeroed();
    state.p = 0;
    state.first_len = 1;
    state.last_len = 0;
    state.rec = 0;

    // Main encode loop.
    let mut i: usize = 0;
    let mut errored = false;
    while i < in_size {
        if state.p == 0 {
            if state.rec >= num_records as isize
                || s_len[state.rec as usize] == 0
            {
                errored = true;
                break;
            }

            if compress_new_read(s, &mut state, gp, unsafe { &*gp.p }, &mut model, &mut rc,
                                 r#in, &mut i, &mut last) != 0
            {
                i += 1;
                continue;
            }
        }

        let pm: &fqz_param = unsafe { &*gp.p };
        let mut jrel: isize = -1;

        while state.p >= 4 && i as isize + jrel + 4 < in_size as isize {
            let l1 = last;
            jrel += 1;
            let qm1 = pm.qmap[r#in[(i as isize + jrel) as usize] as usize];
            last = fqz_update_ctx(pm, &mut state, qm1 as i32);
            let l2 = last;

            jrel += 1;
            let qm2 = pm.qmap[r#in[(i as isize + jrel) as usize] as usize];
            last = fqz_update_ctx(pm, &mut state, qm2 as i32);
            let l3 = last;

            jrel += 1;
            let qm3 = pm.qmap[r#in[(i as isize + jrel) as usize] as usize];
            last = fqz_update_ctx(pm, &mut state, qm3 as i32);
            let l4 = last;

            jrel += 1;
            let qm4 = pm.qmap[r#in[(i as isize + jrel) as usize] as usize];
            last = fqz_update_ctx(pm, &mut state, qm4 as i32);

            SIMPLE_MODEL_encodeSymbol(&mut model.qual[l1 as usize], &mut rc, qm1 as u16);
            SIMPLE_MODEL_encodeSymbol(&mut model.qual[l2 as usize], &mut rc, qm2 as u16);
            SIMPLE_MODEL_encodeSymbol(&mut model.qual[l3 as usize], &mut rc, qm3 as u16);
            SIMPLE_MODEL_encodeSymbol(&mut model.qual[l4 as usize], &mut rc, qm4 as u16);
        }

        while state.p > 0 {
            let l2 = last;
            jrel += 1;
            let qm = pm.qmap[r#in[(i as isize + jrel) as usize] as usize];
            last = fqz_update_ctx(pm, &mut state, qm as i32);
            SIMPLE_MODEL_encodeSymbol(&mut model.qual[l2 as usize], &mut rc, qm as u16);
        }
        i = (i as isize + jrel) as usize;
        i += 1; // for-loop increment
    }

    if !errored && RC_FinishEncode(&mut rc) < 0 {
        *out_size = 0;
        errored = true;
    }

    if errored {
        fqz_destroy_models(&mut model);
        if free_params {
            fqz_free_parameters(gp);
        }
        return Vec::new();
    }

    // For CRAM3.1, undo our earlier reversal step.
    if gp.gflags & GFLAG_DO_REV as u32 != 0 {
        let mut i: usize = 0;
        let mut rec: isize = 0;
        while i < in_size {
            let len = if rec < num_records as isize - 1 {
                s_len[rec as usize] as usize
            } else {
                in_size - i
            };
            if s_flags[rec as usize] & FQZ_FREVERSE as u32 != 0 {
                let (mut bi, mut bj) = (0usize, len - 1);
                while bi < bj {
                    r#in.swap(i + bi, i + bj);
                    bi += 1;
                    bj -= 1;
                }
            }
            i += len;
            rec += 1;
        }
    }

    // Clear selector abuse of flags.
    let s_flags_mut = unsafe { std::slice::from_raw_parts_mut(s.flags, num_records as usize) };
    for rec in 0..num_records as usize {
        s_flags_mut[rec] &= 0xffff;
    }

    *out_size = comp_idx + RC_OutSize(&rc);

    fqz_destroy_models(&mut model);
    if free_params {
        fqz_free_parameters(gp);
    }

    // Hand back the full malloc-sized buffer; caller uses *out_size bytes.
    comp
}

/// `int fqz_read_parameters1(fqz_param *pm, unsigned char *in, size_t in_size)`
// fqzcomp_qual.c:1241
pub fn fqz_read_parameters1(pm: &mut fqz_param, r#in: &[u8]) -> i32 {
    let in_size = r#in.len();
    let mut in_idx: usize = 0;

    if in_size < 7 {
        return -1;
    }

    // Starting context
    pm.context = (r#in[in_idx] as u16) + ((r#in[in_idx + 1] as u16) << 8);
    in_idx += 2;

    // Bit flags
    pm.pflags = r#in[in_idx] as u32;
    in_idx += 1;
    pm.use_qtab = (pm.pflags & PFLAG_HAVE_QTAB as u32) as u8;
    pm.use_dtab = (pm.pflags & PFLAG_HAVE_DTAB as u32) as u8;
    pm.use_ptab = (pm.pflags & PFLAG_HAVE_PTAB as u32) as u8;
    pm.do_sel = pm.pflags & PFLAG_DO_SEL as u32;
    pm.fixed_len = pm.pflags & PFLAG_DO_LEN as u32;
    pm.do_dedup = pm.pflags & PFLAG_DO_DEDUP as u32;
    pm.store_qmap = pm.pflags & PFLAG_HAVE_QMAP as u32;
    pm.max_sym = r#in[in_idx] as i32;
    in_idx += 1;

    // Sub-context sizes and locations
    pm.qbits = (r#in[in_idx] >> 4) as u32;
    pm.qmask = (1 << pm.qbits) - 1;
    pm.qshift = (r#in[in_idx] & 15) as i32;
    in_idx += 1;
    pm.qloc = (r#in[in_idx] >> 4) as u32;
    pm.sloc = (r#in[in_idx] & 15) as u32;
    in_idx += 1;
    pm.ploc = (r#in[in_idx] >> 4) as u32;
    pm.dloc = (r#in[in_idx] & 15) as u32;
    in_idx += 1;

    // Maps and tables
    if pm.store_qmap != 0 {
        for i in 0..256usize {
            pm.qmap[i] = INT_MAX_U32;
        }
        if in_idx + pm.max_sym as usize > in_size {
            return -1;
        }
        for i in 0..pm.max_sym as usize {
            pm.qmap[i] = r#in[in_idx] as u32;
            in_idx += 1;
        }
    } else {
        for i in 0..256usize {
            pm.qmap[i] = i as u32;
        }
    }

    if pm.qbits != 0 {
        if pm.use_qtab != 0 {
            let used = read_array(&r#in[in_idx..], &mut pm.qtab, 256);
            if used < 0 {
                return -1;
            }
            in_idx += used as usize;
        } else {
            for i in 0..256usize {
                pm.qtab[i] = i as u32;
            }
        }
    }

    if pm.use_ptab != 0 {
        let used = read_array(&r#in[in_idx..], &mut pm.ptab, 1024);
        if used < 0 {
            return -1;
        }
        in_idx += used as usize;
    } else {
        for i in 0..1024usize {
            pm.ptab[i] = 0;
        }
    }

    if pm.use_dtab != 0 {
        let used = read_array(&r#in[in_idx..], &mut pm.dtab, 256);
        if used < 0 {
            return -1;
        }
        in_idx += used as usize;
    } else {
        for i in 0..256usize {
            pm.dtab[i] = 0;
        }
    }

    in_idx as i32
}

/// `int fqz_read_parameters(fqz_gparams *gp, unsigned char *in, size_t in_size)`
// fqzcomp_qual.c:1320
pub fn fqz_read_parameters(gp: &mut fqz_gparams, r#in: &[u8]) -> i32 {
    let in_size = r#in.len();
    let mut in_idx: usize = 0;

    if in_size < 10 {
        return -1;
    }

    // Format version
    gp.vers = r#in[in_idx] as i32;
    in_idx += 1;
    if gp.vers != FQZ_VERS {
        return -1;
    }

    // Global flags
    gp.gflags = r#in[in_idx] as u32;
    in_idx += 1;

    // Number of param blocks and param selector details
    gp.nparam = if gp.gflags & GFLAG_MULTI_PARAM as u32 != 0 {
        let v = r#in[in_idx] as i32;
        in_idx += 1;
        v
    } else {
        1
    };
    if gp.nparam <= 0 {
        return -1;
    }
    gp.max_sel = if gp.nparam > 1 { gp.nparam } else { 0 };

    if gp.gflags & GFLAG_HAVE_STAB as u32 != 0 {
        gp.max_sel = r#in[in_idx] as i32;
        in_idx += 1;
        let used = read_array(&r#in[in_idx..], &mut gp.stab, 256);
        if used < 0 {
            // err: (gp->p is still NULL here)
            fqz_free_parameters(gp);
            gp.nparam = 0;
            return -1;
        }
        in_idx += used as usize;
    } else {
        let mut i: usize = 0;
        while (i as i32) < gp.nparam {
            gp.stab[i] = i as u32;
            i += 1;
        }
        while i < 256 {
            gp.stab[i] = (gp.nparam - 1) as u32;
            i += 1;
        }
    }

    // Load the individual parameter blocks
    let p_ptr = unsafe {
        crate::c_compat::malloc(gp.nparam as u64 * std::mem::size_of::<fqz_param>() as u64)
    } as *mut fqz_param;
    if p_ptr.is_null() {
        return -1;
    }
    gp.p = p_ptr;

    gp.max_sym = 0;
    for i in 0..gp.nparam as usize {
        let pm = unsafe { &mut *gp.p.add(i) };
        let e = fqz_read_parameters1(pm, &r#in[in_idx..]);
        if e < 0 {
            fqz_free_parameters(gp);
            gp.nparam = 0;
            return -1;
        }
        if pm.do_sel != 0 && gp.max_sel == 0 {
            fqz_free_parameters(gp);
            gp.nparam = 0;
            return -1;
        }
        in_idx += e as usize;

        if gp.max_sym < pm.max_sym {
            gp.max_sym = pm.max_sym;
        }
    }

    in_idx as i32
}

/// `static int decompress_new_read(fqz_slice *s, fqz_state *state, fqz_gparams *gp,
///     fqz_param *pm, fqz_model *model, RangeCoder *rc, unsigned char *in,
///     ssize_t *in_i, unsigned char *uncomp, size_t *out_size, int *rev,
///     char *rev_a, int *len_a, int *lengths, int nlengths)`
// fqzcomp_qual.c:1381
#[allow(clippy::too_many_arguments)]
pub fn decompress_new_read(
    _s: Option<&fqz_slice>,
    state: &mut fqz_state,
    gp: &fqz_gparams,
    _pm: &fqz_param,
    model: &mut fqz_model,
    rc: &mut RangeCoder,
    _in: &[u8],
    in_i: &mut isize,
    uncomp: &mut [u8],
    out_size: &mut usize,
    rev: &mut i32,
    rev_a: &mut [u8],
    len_a: &mut [i32],
    lengths: &mut [i32],
    nlengths: i32,
) -> i32 {
    let mut i = *in_i;
    let rec = state.rec;

    if _pm.do_sel != 0 {
        state.s = SIMPLE_MODEL_decodeSymbol(&mut model.sel, rc) as u32;
    } else {
        state.s = 0;
    }

    let x = if gp.gflags & GFLAG_HAVE_STAB as u32 != 0 {
        gp.stab[min_i32(255, state.s as i32) as usize] as i32
    } else {
        state.s as i32
    };
    if x >= gp.nparam {
        return -1;
    }
    let pm: &fqz_param = unsafe { &*gp.p.add(x as usize) };

    let mut len = state.last_len;
    if pm.fixed_len == 0 || state.first_len != 0 {
        len = SIMPLE_MODEL_decodeSymbol(&mut model.len[0], rc) as u32;
        len |= (SIMPLE_MODEL_decodeSymbol(&mut model.len[1], rc) as u32) << 8;
        len |= (SIMPLE_MODEL_decodeSymbol(&mut model.len[2], rc) as u32) << 16;
        len |= (SIMPLE_MODEL_decodeSymbol(&mut model.len[3], rc) as u32) << 24;
        state.first_len = 0;
        state.last_len = len;
    }
    if (len as isize) > *out_size as isize - i || (len as i32) <= 0 {
        return -1;
    }

    if !lengths.is_empty() && rec < nlengths as isize {
        lengths[rec as usize] = len as i32;
    }

    if gp.gflags & GFLAG_DO_REV as u32 != 0 {
        *rev = SIMPLE_MODEL_decodeSymbol(&mut model.revcomp, rc) as i32;
        rev_a[rec as usize] = *rev as u8;
        len_a[rec as usize] = len as i32;
    }

    if pm.do_dedup != 0 && SIMPLE_MODEL_decodeSymbol(&mut model.dup, rc) != 0 {
        // Dup of last line
        if len as isize > i {
            return -1;
        }
        let src = (i - len as isize) as usize;
        let dst = i as usize;
        uncomp.copy_within(src..src + len as usize, dst);
        i += len as isize;
        state.p = 0;
        state.rec += 1;
        *in_i = i;
        return 1; // dup => continue
    }

    state.rec += 1;
    state.p = len;
    state.delta = 0;
    state.prevq = 0;
    state.qctx = 0;
    state.ctx = pm.context as u32;

    *in_i = i;

    0
}

/// `unsigned char *uncompress_block_fqz2f(fqz_slice *s, unsigned char *in,
///     size_t in_size, size_t *out_size, int *lengths, int nlengths)`
// fqzcomp_qual.c:1456
/// Returns an empty `Vec` to model the C `NULL` failure return.
pub fn uncompress_block_fqz2f(
    s: Option<&fqz_slice>,
    r#in: &[u8],
    out_size: &mut usize,
    lengths: &mut [i32],
    nlengths: i32,
) -> Vec<u8> {
    let in_size = r#in.len();
    // s is only used under TEST_MAIN (s->num_records = rec); not in production.
    let _ = s;
    let mut gp = fqz_gparams::zeroed();

    let mut len: u32 = 0;
    let in_idx0 = var_get_u32(r#in, Some(in_size), &mut len) as usize;
    *out_size = len as usize;

    let mut rc = RangeCoder::new();
    let mut last: u32 = 0;

    // Decode parameter blocks
    let pr = fqz_read_parameters(&mut gp, &r#in[in_idx0..]);
    if pr < 0 {
        return Vec::new();
    }
    let in_idx = in_idx0 + pr as usize;

    // Optimisations to remove shifts from main loop
    {
        let gp_p = unsafe { std::slice::from_raw_parts_mut(gp.p, gp.nparam as usize) };
        for ip in 0..gp.nparam as usize {
            let pm = &mut gp_p[ip];
            for j in 0..1024usize {
                pm.ptab[j] <<= pm.ploc;
            }
            for j in 0..256usize {
                pm.dtab[j] <<= pm.dloc;
            }
        }
    }

    // Initialise models and entropy coder
    let mut model = fqz_model::new();
    if fqz_create_models(&mut model, &gp) < 0 {
        fqz_free_parameters(&mut gp);
        return Vec::new();
    }

    // Allocate output buffer (Rust-owned).
    let mut uncomp: Vec<u8> = vec![0u8; *out_size];

    // Set up the range coder over the remaining input.  We need a stable
    // pointer to the input bytes for the duration of decoding.
    let in_ptr = r#in.as_ptr() as *mut u8;
    unsafe {
        RC_SetInput(
            &mut rc,
            in_ptr.add(in_idx),
            in_ptr.add(in_size),
        );
    }
    RC_StartDecode(&mut rc);

    let mut nrec: usize = 1000;
    let mut rev_a = vec![0u8; nrec];
    let mut len_a = vec![0i32; nrec];

    // Main decode loop
    let mut state = fqz_state::zeroed();
    state.delta = 0;
    state.prevq = 0;
    state.qctx = 0;
    state.p = 0;
    state.s = 0;
    state.first_len = 1;
    state.last_len = 0;
    state.rec = 0;
    state.ctx = last;

    let mut rev: i32 = 0;
    let x = 0usize;
    let pm: &fqz_param = unsafe { &*gp.p.add(x) };

    let mut errored = false;
    let mut i: isize = 0;
    while (i as u32) < len {
        if state.rec as usize >= nrec {
            nrec *= 2;
            rev_a.resize(nrec, 0);
            len_a.resize(nrec, 0);
        }

        if state.p == 0 {
            // dummy pm passed for the _pm (do_sel) check is &gp.p[0]; the
            // C code passes the outer pm which is &gp.p[0].
            let r = decompress_new_read(
                None,
                &mut state,
                &gp,
                pm,
                &mut model,
                &mut rc,
                r#in,
                &mut i,
                &mut uncomp,
                out_size,
                &mut rev,
                &mut rev_a,
                &mut len_a,
                lengths,
                nlengths,
            );
            if r < 0 {
                errored = true;
                break;
            }
            if r > 0 {
                continue;
            }
            last = state.ctx;
        }

        // Decode and update context.
        //
        // Bounds analysis (rANS-style stale-table hazard does NOT apply here):
        //   * `last` is `state.ctx`, which is either 0 (initial / new-read
        //     reset) or the return value of `fqz_update_ctx`, which is
        //     masked with `CTX_SIZE - 1` (= 0xFFFF).  `model.qual` is
        //     allocated with `CTX_SIZE` (65536) freshly-zeroed entries
        //     above, so `model.qual[last]` is always in range.
        //   * `q` is the symbol returned by `SIMPLE_MODEL_decodeSymbol` on a
        //     QMAX-symbol model.  `SimpleModel::new(QMAX=256)` zero-fills
        //     the symbol-frequency table and `SIMPLE_MODEL_init` writes
        //     `f[1+i].symbol = i` for `i in 0..nsym=256`, so the maximum
        //     symbol value reachable through any (corrupt-stream-driven)
        //     traversal is 255.  `pm.qmap` is `[u32; 256]`, so `qmap[q]`
        //     is in range without an extra runtime check.
        //   * `pm.qtab[q]`, `pm.ptab[...]`, `pm.dtab[...]` indices in
        //     `fqz_update_ctx` are similarly bounded (q<256, p/delta
        //     clamped to 1023/255).
        loop {
            let q = SIMPLE_MODEL_decodeSymbol(&mut model.qual[last as usize], &mut rc);
            last = fqz_update_ctx(pm, &mut state, q as i32);
            uncomp[i as usize] = pm.qmap[q as usize] as u8;
            i += 1;
            if !(state.p != 0 && (i as u32) < len) {
                break;
            }
        }
    }

    if !errored {
        let rec = state.rec;
        if rec as usize >= nrec {
            nrec *= 2;
            rev_a.resize(nrec, 0);
            len_a.resize(nrec, 0);
        }
        rev_a[rec as usize] = rev as u8;
        len_a[rec as usize] = len as i32;

        if gp.gflags & GFLAG_DO_REV as u32 != 0 {
            let mut i2: isize = 0;
            let mut rec2: usize = 0;
            while (i2 as u32) < len && rec2 < nrec {
                let l = len_a[rec2];
                if rev_a[rec2] != 0 {
                    let base = i2 as usize;
                    let (mut bi, mut bj) = (0usize, (l - 1) as usize);
                    while bi < bj {
                        uncomp.swap(base + bi, base + bj);
                        bi += 1;
                        bj -= 1;
                    }
                }
                i2 += l as isize;
                rec2 += 1;
            }
        }

        if RC_FinishDecode(&mut rc) < 0 {
            errored = true;
        }
    }

    fqz_destroy_models(&mut model);
    fqz_free_parameters(&mut gp);
    let _ = in_idx;

    if errored {
        return Vec::new();
    }

    uncomp
}

/// `char *fqz_compress(int vers, fqz_slice *s, char *in, size_t uncomp_size,
///                     size_t *comp_size, int strat, fqz_gparams *gp)`
// fqzcomp_qual.c:1615
///
/// `uncomp_size` is the C `uncomp_size` argument (here it equals `in.len()`).
/// `s` and `in` are `&mut` (the encoder reverses `in` in place and uses
/// `s->flags`).  `gp` is optional (None == C `NULL`).
pub fn fqz_compress(
    vers: i32,
    s: &mut fqz_slice,
    r#in: &mut [u8],
    comp_size: &mut usize,
    strat: i32,
    gp: Option<&mut fqz_gparams>,
) -> Vec<u8> {
    let uncomp_size = r#in.len();
    if uncomp_size > i32::MAX as usize {
        *comp_size = 0;
        return Vec::new();
    }

    compress_block_fqz2f(vers, strat, s, r#in, comp_size, gp)
}

/// `char *fqz_decompress(char *in, size_t comp_size, size_t *uncomp_size,
///                       int *lengths, int nlengths)`
// fqzcomp_qual.c:1626
pub fn fqz_decompress(
    r#in: &[u8],
    uncomp_size: &mut usize,
    lengths: &mut [i32],
    nlengths: i32,
) -> Vec<u8> {
    uncompress_block_fqz2f(None, r#in, uncomp_size, lengths, nlengths)
}

#[cfg(test)]
mod tests;
