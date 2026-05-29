//! Native translation of `htslib/htscodecs/htscodecs/rANS_word.h`
//! (ryg_rans 16-bit-word rANS primitives used by the rANS Nx16 codecs).
//!
//! Cursor representation: the C `uint8_t** pptr` write cursor (which moves
//! *backwards* through the output buffer during encoding and *forwards* during
//! decoding) is modelled as an explicit `(buf, ptr)` pair where `ptr: usize` is
//! an index into `buf`. This matches the convention already used by the sibling
//! `rans_4x8` module and lets us stay byte-for-byte faithful to the C code.
#![allow(non_snake_case, non_camel_case_types, unused_variables, dead_code, clippy::too_many_arguments)]

// rANS_word.h:64
/// `#define RANS_BYTE_L (1u << 15)` — lower bound of normalization interval.
pub const RANS_BYTE_L: u32 = 1 << 15;

// rANS_word.h:67
/// `typedef uint32_t RansState;`
pub type RansState = u32;

// rANS_word.h:173
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RansEncSymbol {
    pub x_max: u32,     // (Exclusive) upper bound of pre-normalization interval
    pub rcp_freq: u32,  // Fixed-point reciprocal frequency
    pub bias: u32,      // Bias
    pub cmpl_freq: u16, // Complement of frequency: (1 << scale_bits) - freq
    pub rcp_shift: u16, // Reciprocal shift
}

// rANS_word.h:188
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RansEncSymbol_simd {
    pub x_max: u32,
    pub rcp_freq: u32,
    pub bias: u32,
    pub cmpl_freq: u32, // cmpl_freq+rcp_shift
}

// rANS_word.h:197
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RansDecSymbol {
    pub start: u16, // Start of range.
    pub freq: u16,  // Symbol frequency.
}

// rANS_word.h:70
/// `static inline void RansEncInit(RansState* r)`
#[inline]
pub fn RansEncInit(r: &mut RansState) {
    *r = RANS_BYTE_L;
}

// rANS_word.h:76
/// `static inline RansState RansEncRenorm(RansState x, uint8_t** pptr, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansEncRenorm(
    mut x: RansState,
    out: &mut [u8],
    pptr: &mut usize,
    freq: u32,
    scale_bits: u32,
) -> RansState {
    let x_max = ((RANS_BYTE_L >> scale_bits) << 16).wrapping_mul(freq).wrapping_sub(1);
    if x > x_max {
        *pptr -= 2;
        out[*pptr] = (x & 0xff) as u8;
        out[*pptr + 1] = ((x >> 8) & 0xff) as u8;
        x >>= 16;
    }
    x
}

// rANS_word.h:97
/// `static inline void RansEncPut(RansState* r, uint8_t** pptr, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansEncPut(
    r: &mut RansState,
    out: &mut [u8],
    pptr: &mut usize,
    start: u32,
    freq: u32,
    scale_bits: u32,
) {
    let x = RansEncRenorm(*r, out, pptr, freq, scale_bits);
    *r = ((x / freq) << scale_bits) + (x % freq) + start;
}

// rANS_word.h:107
/// `static inline void RansEncFlush(RansState* r, uint8_t** pptr)`
#[inline]
pub fn RansEncFlush(r: &mut RansState, out: &mut [u8], pptr: &mut usize) {
    let x = *r;
    *pptr -= 4;
    out[*pptr] = x as u8;
    out[*pptr + 1] = (x >> 8) as u8;
    out[*pptr + 2] = (x >> 16) as u8;
    out[*pptr + 3] = (x >> 24) as u8;
}

// rANS_word.h:123
/// `static inline void RansDecInit(RansState* r, uint8_t** pptr)`
#[inline]
pub fn RansDecInit(r: &mut RansState, input: &[u8], pptr: &mut usize) {
    let x = (input[*pptr] as u32)
        | ((input[*pptr + 1] as u32) << 8)
        | ((input[*pptr + 2] as u32) << 16)
        | ((input[*pptr + 3] as u32) << 24);
    *pptr += 4;
    *r = x;
}

// rANS_word.h:139
/// `static inline uint32_t RansDecGet(RansState* r, uint32_t scale_bits)`
#[inline]
pub fn RansDecGet(r: &RansState, scale_bits: u32) -> u32 {
    *r & ((1u32 << scale_bits) - 1)
}

// rANS_word.h:147
/// `static inline void RansDecAdvance(RansState* r, uint8_t** pptr, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvance(
    r: &mut RansState,
    input: &[u8],
    pptr: &mut usize,
    start: u32,
    freq: u32,
    scale_bits: u32,
) {
    let mask = (1u32 << scale_bits) - 1;
    let mut x = *r;
    x = freq
        .wrapping_mul(x >> scale_bits)
        .wrapping_add(x & mask)
        .wrapping_sub(start);
    if x < RANS_BYTE_L {
        loop {
            x = (x << 8) | input[*pptr] as u32;
            *pptr += 1;
            if x >= RANS_BYTE_L {
                break;
            }
        }
    }
    *r = x;
}

// rANS_word.h:203
/// `static inline void RansEncSymbolInit(RansEncSymbol* s, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansEncSymbolInit(s: &mut RansEncSymbol, start: u32, freq: u32, scale_bits: u32) {
    s.x_max = ((RANS_BYTE_L >> scale_bits) << 16).wrapping_mul(freq).wrapping_sub(1);
    s.cmpl_freq = ((1u32 << scale_bits).wrapping_sub(freq)) as u16;
    if freq < 2 {
        s.rcp_freq = !0u32;
        s.rcp_shift = 0;
        s.bias = start.wrapping_add(1u32 << scale_bits).wrapping_sub(1);
    } else {
        let mut shift: u32 = 0;
        while freq > (1u32 << shift) {
            shift += 1;
        }
        s.rcp_freq = (((1u64 << (shift + 31)) + freq as u64 - 1) / freq as u64) as u32;
        s.rcp_shift = (shift - 1) as u16;
        s.bias = start;
    }
    s.rcp_shift += 32; // Avoid the extra >>32 in RansEncPutSymbol
}

// rANS_word.h:277
/// `static inline void RansDecSymbolInit(RansDecSymbol* s, uint32_t start, uint32_t freq)`
#[inline]
pub fn RansDecSymbolInit(s: &mut RansDecSymbol, start: u32, freq: u32) {
    s.start = start as u16;
    s.freq = freq as u16;
}

// rANS_word.h:289
/// `static inline void RansEncPutSymbol(RansState* r, uint8_t** pptr, RansEncSymbol const* sym)`
/// (Branchless little-endian variant.)
#[inline]
pub fn RansEncPutSymbol(r: &mut RansState, out: &mut [u8], pptr: &mut usize, sym: &RansEncSymbol) {
    let mut x = *r;
    let x_max = sym.x_max;

    // int c = (x > x_max); c*=2;  memcpy(*pptr-2,&x,2);  x>>=c*8;  *pptr-=c;
    let c = (x > x_max) as usize * 2;
    out[*pptr - 2] = (x & 0xff) as u8;
    out[*pptr - 1] = ((x >> 8) & 0xff) as u8;
    x >>= (c as u32) * 8;
    *pptr -= c;

    let q = (((x as u64) * (sym.rcp_freq as u64)) >> sym.rcp_shift) as u32;
    *r = x
        .wrapping_add(sym.bias)
        .wrapping_add(q.wrapping_mul(sym.cmpl_freq as u32));
}

// rANS_word.h:340
/// `static inline void RansEncPutSymbol_branched(RansState* r, uint8_t** pptr, RansEncSymbol const* sym)`
#[inline]
pub fn RansEncPutSymbol_branched(
    r: &mut RansState,
    out: &mut [u8],
    pptr: &mut usize,
    sym: &RansEncSymbol,
) {
    let mut x = *r;
    let x_max = sym.x_max;

    if x > x_max {
        *pptr -= 2;
        out[*pptr] = (x & 0xff) as u8;
        out[*pptr + 1] = ((x >> 8) & 0xff) as u8;
        x >>= 16;
    }

    let q = (((x as u64) * (sym.rcp_freq as u64)) >> sym.rcp_shift) as u32;
    *r = x
        .wrapping_add(sym.bias)
        .wrapping_add(q.wrapping_mul(sym.cmpl_freq as u32));
}

// rANS_word.h:384
/// `static inline void RansDecAdvanceSymbol(RansState* r, uint8_t** pptr, RansDecSymbol const* sym, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceSymbol(
    r: &mut RansState,
    input: &[u8],
    pptr: &mut usize,
    sym: &RansDecSymbol,
    scale_bits: u32,
) {
    RansDecAdvance(r, input, pptr, sym.start as u32, sym.freq as u32, scale_bits);
}

// rANS_word.h:392
/// `static inline void RansDecAdvanceStep(RansState* r, uint32_t start, uint32_t freq, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceStep(r: &mut RansState, start: u32, freq: u32, scale_bits: u32) {
    let mask = (1u32 << scale_bits) - 1;
    let x = *r;
    *r = freq
        .wrapping_mul(x >> scale_bits)
        .wrapping_add(x & mask)
        .wrapping_sub(start);
}

// rANS_word.h:402
/// `static inline void RansDecAdvanceSymbolStep(RansState* r, RansDecSymbol const* sym, uint32_t scale_bits)`
#[inline]
pub fn RansDecAdvanceSymbolStep(r: &mut RansState, sym: &RansDecSymbol, scale_bits: u32) {
    RansDecAdvanceStep(r, sym.start as u32, sym.freq as u32, scale_bits);
}

// rANS_word.h:441 (portable branchless variant; the __x86_64 asm variant shares
// the same observable behaviour)
/// `static inline void RansDecRenorm(RansState* r, uint8_t** pptr)`
#[inline]
pub fn RansDecRenorm(r: &mut RansState, input: &[u8], pptr: &mut usize) {
    let x = *r;
    let cmp = (x < RANS_BYTE_L) as usize * 2;
    let y = input[*pptr] as u32 + ((input[*pptr + 1] as u32) << 8);
    let x2 = (x << 16) | y;
    *r = if cmp != 0 { x2 } else { x };
    *pptr += cmp;
}

// rANS_word.h:470
/// `static inline void RansDecRenormSafe(RansState* r, uint8_t** pptr, uint8_t *ptr_end)`
#[inline]
pub fn RansDecRenormSafe(r: &mut RansState, input: &[u8], pptr: &mut usize, ptr_end: usize) {
    let mut x = *r;
    if x >= RANS_BYTE_L || *pptr + 1 >= ptr_end {
        return;
    }
    let y = input[*pptr] as u32 + ((input[*pptr + 1] as u32) << 8);
    x = (x << 16) | y;
    *pptr += 2;
    *r = x;
}
