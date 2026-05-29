//! Native translation of `htslib/htscodecs/htscodecs/rANS_byte.h`
//! (ryg_rans byte-aligned rANS primitives).
//!
//! Cursor representation. The C `uint8_t** pptr` cursor is modelled here as a
//! pair of nested mutable references:
//!
//! * encode (cursor moves *backwards*): `pptr: &mut &mut [u8]` represents the
//!   **prefix of the output buffer up to the current write position**. A write
//!   `*--ptr = b` becomes `slice[slice.len() - 1] = b` followed by shrinking
//!   the slice by one byte from the back via [`core::mem::take`].
//! * decode (cursor moves *forwards*): `pptr: &mut &[u8]` represents the
//!   **remaining unread bytes**. A read `*ptr++` becomes `slice[0]` followed
//!   by shrinking the slice by one byte from the front.
//!
//! This matches the `uint8_t**` semantics one-to-one (no raw pointers) while
//! retaining bounds checking. Sibling rANS modules (`rans_4x8`, `rans_word`)
//! use the alternative `(buf, ptr: &mut usize)` pair; either representation is
//! equivalent — we keep the stub-declared signatures here.
#![allow(non_snake_case, non_camel_case_types, unused_variables, dead_code, clippy::too_many_arguments)]

// rANS_byte.h:64
/// `#define RANS_BYTE_L (1u << 23)` — lower bound of normalization interval.
pub const RANS_BYTE_L: u32 = 1 << 23;

// rANS_byte.h:67
/// `typedef uint32_t RansState;`
pub type RansState = u32;

// rANS_byte.h:175
/// Encoder symbol description.
/// ```c
/// typedef struct {
///     uint32_t x_max;
///     uint32_t rcp_freq;
///     uint32_t bias;
///     uint16_t cmpl_freq;
///     uint16_t rcp_shift;
/// } RansEncSymbol;
/// ```
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RansEncSymbol {
    pub x_max: u32,     // (Exclusive) upper bound of pre-normalization interval
    pub rcp_freq: u32,  // Fixed-point reciprocal frequency
    pub bias: u32,      // Bias
    pub cmpl_freq: u16, // Complement of frequency: (1 << scale_bits) - freq
    pub rcp_shift: u16, // Reciprocal shift
}

// rANS_byte.h:186
/// ```c
/// typedef struct {
///     uint16_t freq;
///     uint16_t start;
/// } RansDecSymbol;
/// ```
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RansDecSymbol {
    pub freq: u16,  // Symbol frequency.
    pub start: u16, // Start of range.
}

// rANS_byte.h:191
/// ```c
/// typedef struct {
///     uint32_t freq;
///     uint32_t start;
/// } RansDecSymbol32;
/// ```
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RansDecSymbol32 {
    pub freq: u32,  // Symbol frequency.
    pub start: u32, // Start of range.
}

// ---------------------------------------------------------------------------
// Internal cursor helpers (not in the C header — just to keep the slice
// re-borrowing one-liner readable at each call site).
// ---------------------------------------------------------------------------

/// Write `byte` at position `*--ptr` and shrink the encode prefix slice by 1.
#[inline]
fn enc_push_back(pptr: &mut &mut [u8], byte: u8) {
    let s = core::mem::take(pptr);
    let n = s.len();
    s[n - 1] = byte;
    *pptr = &mut s[..n - 1];
}

/// Read one byte from the front of the decode slice (`*ptr++`).
#[inline]
fn dec_pop_front(pptr: &mut &[u8]) -> u8 {
    let s = *pptr;
    let b = s[0];
    *pptr = &s[1..];
    b
}

// ---------------------------------------------------------------------------
// Public API — direct translations of rANS_byte.h.
// ---------------------------------------------------------------------------

// rANS_byte.h:70
/// `static inline void RansEncInit(RansState* r)`
#[inline]
pub fn RansEncInit(r: &mut RansState) {
    // *r = RANS_BYTE_L;
    *r = RANS_BYTE_L;
}

// rANS_byte.h:109
/// `static inline void RansEncFlush(RansState* r, uint8_t** pptr)`
/// (`pptr` is a write cursor moving backwards through the output buffer.)
#[inline]
pub fn RansEncFlush(r: &mut RansState, pptr: &mut &mut [u8]) {
    let x = *r;
    // ptr -= 4; ptr[0..4] = x as 4 little-endian bytes.
    let s = core::mem::take(pptr);
    let n = s.len();
    s[n - 4] = (x) as u8;
    s[n - 3] = (x >> 8) as u8;
    s[n - 2] = (x >> 16) as u8;
    s[n - 1] = (x >> 24) as u8;
    *pptr = &mut s[..n - 4];
}

// rANS_byte.h:125
/// `static inline void RansDecInit(RansState* r, uint8_t** pptr)`
#[inline]
pub fn RansDecInit(r: &mut RansState, pptr: &mut &[u8]) {
    let s = *pptr;
    let x = (s[0] as u32)
        | ((s[1] as u32) << 8)
        | ((s[2] as u32) << 16)
        | ((s[3] as u32) << 24);
    *pptr = &s[4..];
    *r = x;
}

// rANS_byte.h:141
/// `static inline uint32_t RansDecGet(RansState* r, uint32_t scale_bits)`
#[inline]
pub fn RansDecGet(r: &mut RansState, scale_bits: u32) -> u32 {
    // return *r & ((1u << scale_bits) - 1);
    *r & ((1u32 << scale_bits) - 1)
}

// rANS_byte.h:149
/// `static inline void RansDecAdvance(RansState* r, uint8_t** pptr, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvance(
    r: &mut RansState,
    pptr: &mut &[u8],
    start: u32,
    freq: u32,
    scale_bits: u32,
) {
    let mask = (1u32 << scale_bits) - 1;

    // s, x = D(x)
    let mut x = *r;
    x = freq
        .wrapping_mul(x >> scale_bits)
        .wrapping_add(x & mask)
        .wrapping_sub(start);

    // renormalize
    if x < RANS_BYTE_L {
        loop {
            let b = dec_pop_front(pptr) as u32;
            x = (x << 8) | b;
            if x >= RANS_BYTE_L {
                break;
            }
        }
    }

    *r = x;
}

// rANS_byte.h:197
/// `static inline void RansEncSymbolInit(RansEncSymbol* s, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansEncSymbolInit(s: &mut RansEncSymbol, start: u32, freq: u32, scale_bits: u32) {
    // RansAssert(scale_bits <= 16);
    // RansAssert(start <= (1u << scale_bits));
    // RansAssert(freq  <= (1u << scale_bits) - start);

    s.x_max = ((RANS_BYTE_L >> scale_bits) << 8).wrapping_mul(freq);
    s.cmpl_freq = ((1u32 << scale_bits).wrapping_sub(freq)) as u16;
    if freq < 2 {
        // See header for derivation of the freq<2 case.
        s.rcp_freq = !0u32;
        s.rcp_shift = 0;
        s.bias = start + (1u32 << scale_bits) - 1;
    } else {
        // Alverson, "Integer Division using reciprocals"
        // shift = ceil(log2(freq))
        let mut shift: u32 = 0;
        while freq > (1u32 << shift) {
            shift += 1;
        }

        s.rcp_freq =
            (((1u64 << (shift + 31)) + freq as u64 - 1) / freq as u64) as u32;
        s.rcp_shift = (shift - 1) as u16;

        // With these values, 'q' is the correct quotient, so we have
        // bias=start.
        s.bias = start;
    }

    s.rcp_shift += 32; // Avoid the extra >>32 in RansEncPutSymbol
}

// rANS_byte.h:271
/// `static inline void RansDecSymbolInit(RansDecSymbol* s, uint32_t start, uint32_t freq)`
#[inline]
pub fn RansDecSymbolInit(s: &mut RansDecSymbol, start: u32, freq: u32) {
    // RansAssert(start <= (1 << 16));
    // RansAssert(freq  <= (1 << 16) - start);
    s.start = start as u16;
    s.freq = freq as u16;
}

// rANS_byte.h:283
/// `static inline void RansEncPutSymbol(RansState* r, uint8_t** pptr, RansEncSymbol const* sym)`
#[inline]
pub fn RansEncPutSymbol(r: &mut RansState, pptr: &mut &mut [u8], sym: &RansEncSymbol) {
    // RansAssert(sym->x_max != 0); // can't encode symbol with freq=0

    // renormalize
    let mut x = *r;
    let x_max = sym.x_max;

    // Branchless byte-emit, mirrored from the C source. Even when the byte
    // is not actually consumed we still touch ptr[-1]; preserve that by
    // writing into the byte at slice.len()-1 and then optionally shrinking
    // the slice. (This matches `ptr[-1] = x; ptr -= o; x >>= o*8`.)
    let o: u32 = if x >= x_max { 1 } else { 0 };
    {
        let s = core::mem::take(pptr);
        let n = s.len();
        s[n - 1] = (x & 0xff) as u8;
        *pptr = &mut s[..n - o as usize];
    }
    x >>= o * 8;

    if x >= x_max {
        // *--ptr = (uint8_t)(x & 0xff); x >>= 8;
        enc_push_back(pptr, (x & 0xff) as u8);
        x >>= 8;
    }

    // The extra >>32 has already been baked into rcp_shift in
    // RansEncSymbolInit.
    let q =
        (((x as u64).wrapping_mul(sym.rcp_freq as u64)) >> sym.rcp_shift) as u32;
    *r = q
        .wrapping_mul(sym.cmpl_freq as u32)
        .wrapping_add(x)
        .wrapping_add(sym.bias);
}

// rANS_byte.h:323
/// `static inline void RansEncPutSymbol4(RansState *r0, RansState *r1, RansState *r2, RansState *r3, uint8_t** pptr, RansEncSymbol const *sym0, RansEncSymbol const *sym1, RansEncSymbol const *sym2, RansEncSymbol const *sym3)`
#[inline]
pub fn RansEncPutSymbol4(
    r0: &mut RansState,
    r1: &mut RansState,
    r2: &mut RansState,
    r3: &mut RansState,
    pptr: &mut &mut [u8],
    sym0: &RansEncSymbol,
    sym1: &RansEncSymbol,
    sym2: &RansEncSymbol,
    sym3: &RansEncSymbol,
) {
    // RansAssert(sym{0..3}->x_max != 0);

    let m = [sym0.x_max, sym1.x_max, sym2.x_max, sym3.x_max];

    // -- state 0 --
    let mut x0 = *r0;
    {
        let o: u32 = if x0 >= m[0] { 1 } else { 0 };
        let s = core::mem::take(pptr);
        let n = s.len();
        s[n - 1] = x0 as u8;
        *pptr = &mut s[..n - o as usize];
        x0 >>= o * 8;
    }
    if x0 >= m[0] {
        enc_push_back(pptr, x0 as u8);
        x0 >>= 8;
    }

    // -- state 1 --
    let mut x1 = *r1;
    {
        let o: u32 = if x1 >= m[1] { 1 } else { 0 };
        let s = core::mem::take(pptr);
        let n = s.len();
        s[n - 1] = x1 as u8;
        *pptr = &mut s[..n - o as usize];
        x1 >>= o * 8;
    }
    if x1 >= m[1] {
        enc_push_back(pptr, x1 as u8);
        x1 >>= 8;
    }

    // -- state 2 --
    let mut x2 = *r2;
    {
        let o: u32 = if x2 >= m[2] { 1 } else { 0 };
        let s = core::mem::take(pptr);
        let n = s.len();
        s[n - 1] = x2 as u8;
        *pptr = &mut s[..n - o as usize];
        x2 >>= o * 8;
    }
    if x2 >= m[2] {
        enc_push_back(pptr, x2 as u8);
        x2 >>= 8;
    }

    // -- state 3 --
    let mut x3 = *r3;
    {
        let o: u32 = if x3 >= m[3] { 1 } else { 0 };
        let s = core::mem::take(pptr);
        let n = s.len();
        s[n - 1] = x3 as u8;
        *pptr = &mut s[..n - o as usize];
        x3 >>= o * 8;
    }
    if x3 >= m[3] {
        enc_push_back(pptr, x3 as u8);
        x3 >>= 8;
    }

    // x = C(s,x)
    let qa = (((x0 as u64).wrapping_mul(sym0.rcp_freq as u64)) >> sym0.rcp_shift)
        as u32;
    let big_x0 = qa.wrapping_mul(sym0.cmpl_freq as u32);
    let qb = (((x1 as u64).wrapping_mul(sym1.rcp_freq as u64)) >> sym1.rcp_shift)
        as u32;
    let big_x1 = qb.wrapping_mul(sym1.cmpl_freq as u32);

    *r0 = big_x0.wrapping_add(x0).wrapping_add(sym0.bias);
    *r1 = big_x1.wrapping_add(x1).wrapping_add(sym1.bias);

    let qa = (((x2 as u64).wrapping_mul(sym2.rcp_freq as u64)) >> sym2.rcp_shift)
        as u32;
    let big_x2 = qa.wrapping_mul(sym2.cmpl_freq as u32);
    let qb = (((x3 as u64).wrapping_mul(sym3.rcp_freq as u64)) >> sym3.rcp_shift)
        as u32;
    let big_x3 = qb.wrapping_mul(sym3.cmpl_freq as u32);

    *r2 = big_x2.wrapping_add(x2).wrapping_add(sym2.bias);
    *r3 = big_x3.wrapping_add(x3).wrapping_add(sym3.bias);
}

// rANS_byte.h:412
/// `static inline void RansDecAdvanceSymbol(RansState* r, uint8_t** pptr, RansDecSymbol const* sym, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceSymbol(
    r: &mut RansState,
    pptr: &mut &[u8],
    sym: &RansDecSymbol,
    scale_bits: u32,
) {
    RansDecAdvance(r, pptr, sym.start as u32, sym.freq as u32, scale_bits);
}

// rANS_byte.h:420
/// `static inline void RansDecAdvanceStep(RansState* r, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceStep(r: &mut RansState, start: u32, freq: u32, scale_bits: u32) {
    let mask = (1u32 << scale_bits) - 1;

    // s, x = D(x)
    let x = *r;
    *r = freq
        .wrapping_mul(x >> scale_bits)
        .wrapping_add(x & mask)
        .wrapping_sub(start);
}

// rANS_byte.h:430
/// `static inline void RansDecAdvanceSymbolStep(RansState* r, RansDecSymbol const* sym, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceSymbolStep(r: &mut RansState, sym: &RansDecSymbol, scale_bits: u32) {
    RansDecAdvanceStep(r, sym.start as u32, sym.freq as u32, scale_bits);
}

// rANS_byte.h:442 / rANS_byte.h:512
/// `static inline void RansDecRenorm(RansState* r, uint8_t** pptr)`
///
/// Portable scalar variant (the C source also has an x86_64 inline-asm
/// variant that produces identical results; we translate the portable C
/// fallback path).
#[inline]
pub fn RansDecRenorm(r: &mut RansState, pptr: &mut &[u8]) {
    // renormalize
    let mut x = *r;

    if x >= RANS_BYTE_L {
        return;
    }
    let b0 = dec_pop_front(pptr) as u32;
    x = (x << 8) | b0;
    if x < RANS_BYTE_L {
        let b1 = dec_pop_front(pptr) as u32;
        x = (x << 8) | b1;
    }

    *r = x;
}

// rANS_byte.h:467 / rANS_byte.h:537
/// `static inline void RansDecRenorm2(RansState* r1, RansState* r2, uint8_t** pptr)`
#[inline]
pub fn RansDecRenorm2(r1: &mut RansState, r2: &mut RansState, pptr: &mut &[u8]) {
    // Portable variant: just call RansDecRenorm twice.
    RansDecRenorm(r1, pptr);
    RansDecRenorm(r2, pptr);
}

// rANS_byte.h:544
/// `static inline void RansDecRenormSafe(RansState* r, uint8_t** pptr, uint8_t *ptr_end)`
///
/// `ptr_end` would be a pointer in C; here we accept the number of bytes
/// **remaining** in `*pptr` past which we must not read. (When called from
/// sibling Rust modules the slice itself often already encodes that bound,
/// but the explicit parameter is kept for parity with the C signature.)
#[inline]
pub fn RansDecRenormSafe(r: &mut RansState, pptr: &mut &[u8], ptr_end: usize) {
    let mut x = *r;
    if x >= RANS_BYTE_L || pptr.len() == 0 || pptr.as_ptr() as usize >= ptr_end {
        // First condition matches `x >= RANS_BYTE_L`; the latter two match
        // `ptr >= ptr_end` (either because we already exhausted the slice,
        // or the caller-supplied end pointer says so).
        return;
    }
    let b0 = dec_pop_front(pptr) as u32;
    x = (x << 8) | b0;
    if x < RANS_BYTE_L && !pptr.is_empty() && (pptr.as_ptr() as usize) < ptr_end {
        let b1 = dec_pop_front(pptr) as u32;
        x = (x << 8) | b1;
    }
    *r = x;
}

// rANS_byte.h:556
/// `static inline void RansDecSymbolInit32(RansDecSymbol32* s, uint32_t start, uint32_t freq)`
#[inline]
pub fn RansDecSymbolInit32(s: &mut RansDecSymbol32, start: u32, freq: u32) {
    // RansAssert(start <= (1 << 16));
    // RansAssert(freq  <= (1 << 16) - start);
    s.start = start;
    s.freq = freq;
}

// rANS_byte.h:564
/// `static inline void RansDecAdvanceSymbol32(RansState* r, uint8_t** pptr, RansDecSymbol32 const* sym, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceSymbol32(
    r: &mut RansState,
    pptr: &mut &[u8],
    sym: &RansDecSymbol32,
    scale_bits: u32,
) {
    RansDecAdvance(r, pptr, sym.start, sym.freq, scale_bits);
}

#[cfg(test)]
mod tests;
