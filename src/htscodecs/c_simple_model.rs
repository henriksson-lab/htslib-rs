//! Translation of htscodecs `c_simple_model.h`.
//!
//! A simple, adaptive frequency model used by the range coder. In C this is a
//! macro-instantiated template: `#define NSYM <n>; #include "c_simple_model.h"`
//! generates a concrete `SIMPLE_MODEL(NSYM,_)` struct plus its `_init`,
//! `_normalize`, `_encodeSymbol` and `_decodeSymbol` methods.
//!
//! Here it is represented as a single `SimpleModel` struct carrying its symbol
//! count (`nsym`) at runtime, plus one fn per templated method.  Callers in the
//! C code instantiate this for NSYM = 256 and 258.
//!
//! C struct layout (contiguous, by `SymFreqs`):
//! ```text
//!   sentinel, F[0], F[1], ... F[NSYM-1], F[NSYM], terminal
//! ```
//! The encode/decode loops walk a pointer `s` starting at `F[0]` and use
//! `s[-1]` (which reaches back to `sentinel`) for the approximate bubble-sort.
//! `F[NSYM].Freq == 0` terminates the `normalize` loop.
//!
//! We mirror this exactly with a single flat `Vec<SymFreqs> f` laid out as
//! `[sentinel, F[0..=NSYM], terminal]`, i.e. length `NSYM + 3`.  Index mapping:
//!   - `sentinel`  -> `f[0]`
//!   - `F[k]`      -> `f[1 + k]`   (k in 0..=NSYM)
//!   - `terminal`  -> `f[NSYM + 2]`
//! The `s` pointer of the C code is therefore an index `si` into `f` that
//! starts at 1 (== `&F[0]`); `s[-1]` == `f[si-1]`.

use crate::htscodecs::c_range_coder::{RangeCoder, RC_Decode, RC_Encode, RC_GetFreq};

// #define MAX_FREQ (1<<16)-17               // c_simple_model.h:63
pub const MAX_FREQ: u32 = (1 << 16) - 17;

// #define STEP 16                           // c_simple_model.h:66
pub const STEP: u16 = 16;

/// ```c
/// typedef struct {
///     uint16_t Freq;
///     uint16_t Symbol;
/// } SymFreqs;
/// ```
// c_simple_model.h:67
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SymFreqs {
    pub freq: u16,
    pub symbol: u16,
}

/// Concrete instantiation of `SIMPLE_MODEL(NSYM,_)`.
///
/// ```c
/// typedef struct {
///     uint32_t TotFreq;  // Total frequency
///     SymFreqs sentinel, F[NSYM+1], terminal;
/// } SIMPLE_MODEL(NSYM,_);
/// ```
// c_simple_model.h:77
pub struct SimpleModel {
    pub tot_freq: u32,
    /// Flat layout `[sentinel, F[0..=NSYM], terminal]`, length `nsym + 3`.
    pub f: Vec<SymFreqs>,
    /// The NSYM template parameter, kept at runtime.
    pub nsym: usize,
}

impl SimpleModel {
    /// Allocate an (uninitialised) model for the given NSYM, mirroring the C
    /// struct of `sentinel, F[NSYM+1], terminal`.  Call `SIMPLE_MODEL_init`
    /// before use.
    pub fn new(nsym: usize) -> Self {
        SimpleModel {
            tot_freq: 0,
            f: vec![SymFreqs::default(); nsym + 3],
            nsym,
        }
    }
}

/// `static inline void SIMPLE_MODEL(NSYM,_init)(SIMPLE_MODEL(NSYM,_) *m, int max_sym)`
// c_simple_model.h:85
pub fn SIMPLE_MODEL_init(m: &mut SimpleModel, max_sym: i32) {
    let nsym = m.nsym;
    // F[k] lives at f[1+k].
    let mut i: usize = 0;
    while (i as i32) < max_sym {
        m.f[1 + i].symbol = i as u16;
        m.f[1 + i].freq = 1;
        i += 1;
    }
    while i < nsym {
        m.f[1 + i].symbol = i as u16;
        m.f[1 + i].freq = 0;
        i += 1;
    }

    m.tot_freq = max_sym as u32;
    // sentinel == f[0]
    m.f[0].symbol = 0;
    m.f[0].freq = MAX_FREQ as u16; // Always first; simplifies sorting.
    // terminal == f[nsym + 2]
    m.f[nsym + 2].symbol = 0;
    m.f[nsym + 2].freq = MAX_FREQ as u16;
    // m->F[NSYM].Freq = 0;  F[NSYM] == f[1+nsym]
    m.f[1 + nsym].freq = 0; // terminates normalize() loop.
}

/// `static inline void SIMPLE_MODEL(NSYM,_normalize)(SIMPLE_MODEL(NSYM,_) *m)`
// c_simple_model.h:106
pub fn SIMPLE_MODEL_normalize(m: &mut SimpleModel) {
    // for (s = m->F; s->Freq; s++)  -> start at f[1] (== &F[0])
    m.tot_freq = 0;
    let mut si: usize = 1;
    while m.f[si].freq != 0 {
        m.f[si].freq -= m.f[si].freq >> 1;
        m.tot_freq += m.f[si].freq as u32;
        si += 1;
    }
}

/// `static inline void SIMPLE_MODEL(NSYM,_encodeSymbol)(SIMPLE_MODEL(NSYM,_) *m,
///                                                      RangeCoder *rc, uint16_t sym)`
// c_simple_model.h:117
pub fn SIMPLE_MODEL_encodeSymbol(m: &mut SimpleModel, rc: &mut RangeCoder, sym: u16) {
    // SymFreqs *s = m->F;  -> index 1
    let mut si: usize = 1;
    let mut acc_freq: u32 = 0;

    while m.f[si].symbol != sym {
        acc_freq += m.f[si].freq as u32;
        si += 1;
    }

    RC_Encode(rc, acc_freq, m.f[si].freq as u32, m.tot_freq);
    m.f[si].freq = m.f[si].freq.wrapping_add(STEP);
    m.tot_freq += STEP as u32;

    if m.tot_freq > MAX_FREQ {
        SIMPLE_MODEL_normalize(m);
    }

    // Keep approx sorted: if (s[0].Freq > s[-1].Freq) swap.
    if m.f[si].freq > m.f[si - 1].freq {
        m.f.swap(si, si - 1);
    }
}

/// `static inline uint16_t SIMPLE_MODEL(NSYM,_decodeSymbol)(SIMPLE_MODEL(NSYM,_) *m, RangeCoder *rc)`
// c_simple_model.h:140
pub fn SIMPLE_MODEL_decodeSymbol(m: &mut SimpleModel, rc: &mut RangeCoder) -> u16 {
    // SymFreqs *s = m->F;  -> index 1
    let mut si: usize = 1;
    let freq = RC_GetFreq(rc, m.tot_freq);

    if freq > MAX_FREQ {
        return 0; // error
    }

    // for (AccFreq = 0; (AccFreq += s->Freq) <= freq; s++);
    let mut acc_freq: u32 = 0;
    loop {
        acc_freq += m.f[si].freq as u32;
        if acc_freq > freq {
            break;
        }
        si += 1;
    }
    // if (s - m->F > NSYM) return 0;   s - m->F == si - 1
    if (si - 1) as i64 > m.nsym as i64 {
        return 0; // error
    }

    acc_freq -= m.f[si].freq as u32;

    RC_Decode(rc, acc_freq, m.f[si].freq as u32, m.tot_freq);
    m.f[si].freq = m.f[si].freq.wrapping_add(STEP);
    m.tot_freq += STEP as u32;

    if m.tot_freq > MAX_FREQ {
        SIMPLE_MODEL_normalize(m);
    }

    // Keep approx sorted: if (s[0].Freq > s[-1].Freq) { swap; return t.Symbol; }
    if m.f[si].freq > m.f[si - 1].freq {
        // t = s[0]; s[0]=s[-1]; s[-1]=t; return t.Symbol;
        let t = m.f[si];
        m.f.swap(si, si - 1);
        return t.symbol;
    }

    m.f[si].symbol
}
