//! Native translation of `tokenise_name3.c` + `tokenise_name3.h` (htscodecs).
//!
//! Read-name tokeniser. It generates a series of byte streams (per token) and
//! compresses these either using static rANS (`rans_static4x16pr`) or dynamic
//! arithmetic coding (`arith_dynamic`). It uses the pooled allocator
//! (`pooled_alloc`) for the encoder trie nodes.
//!
//! Source: htslib/htscodecs/htscodecs/tokenise_name3.c and tokenise_name3.h
//!
//! The original C code is intensely pointer-based.  This Rust translation keeps
//! raw pointers at the remaining C-style allocation/API boundaries, but owns the
//! read-name history and descriptor payloads with Rust containers.

#![allow(non_snake_case, non_camel_case_types, unused_assignments)]

use super::arith_dynamic::{arith_compress_bound, arith_compress_to, arith_uncompress_to};
use super::rans_static4x16pr::{rans_compress_4x16, rans_uncompress_4x16};
use super::varint::{var_get_u32, var_put_u32};

//-----------------------------------------------------------------------------
// #define constants (tokenise_name3.c)

/// `#define MAX_TOKENS 128`
// tokenise_name3.c:115
pub const MAX_TOKENS: usize = 128;

/// `#define MAX_TBLOCKS (MAX_TOKENS<<4)`
// tokenise_name3.c:116
pub const MAX_TBLOCKS: usize = MAX_TOKENS << 4;

/// `#define MAX_NAMES 1000000`
// tokenise_name3.c:119
pub const MAX_NAMES: usize = 1000000;

//-----------------------------------------------------------------------------
// enums / structs (tokenise_name3.c)

/// ```c
/// enum name_type {N_ERR = -1, N_TYPE = 0, N_ALPHA, N_CHAR, N_DIGITS0, N_DZLEN,
///                 N_DUP, N_DIFF, N_DIGITS, N_DDELTA, N_DDELTA0, N_MATCH, N_NOP,
///                 N_END, N_ALL};
/// ```
// tokenise_name3.c:121
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum name_type {
    N_ERR = -1,
    N_TYPE = 0,
    N_ALPHA,
    N_CHAR,
    N_DIGITS0,
    N_DZLEN,
    N_DUP,
    N_DIFF,
    N_DIGITS,
    N_DDELTA,
    N_DDELTA0,
    N_MATCH,
    N_NOP,
    N_END,
    N_ALL,
}

use name_type::*;

impl name_type {
    fn from_i32(v: i32) -> name_type {
        match v {
            0 => N_TYPE,
            1 => N_ALPHA,
            2 => N_CHAR,
            3 => N_DIGITS0,
            4 => N_DZLEN,
            5 => N_DUP,
            6 => N_DIFF,
            7 => N_DIGITS,
            8 => N_DDELTA,
            9 => N_DDELTA0,
            10 => N_MATCH,
            11 => N_NOP,
            12 => N_END,
            13 => N_ALL,
            _ => N_ERR,
        }
    }
}

/// ```c
/// typedef struct trie {
///     struct trie *next, *sibling;
///     int count;
///     uint32_t c:8;
///     uint32_t n:24; // Nth line
/// } trie_t;
/// ```
// tokenise_name3.c:124
///
/// The original C uses a pool-allocated intrusive linked list of nodes joined by
/// raw `next`/`sibling` pointers.  This Rust translation owns all nodes in an
/// arena (`Vec<trie_t>` on the context) and joins them with `Option<usize>`
/// indices, so no raw pointers or custom allocator are needed.
#[derive(Debug, Default, Clone)]
pub struct trie_t {
    pub next: Option<usize>,
    pub sibling: Option<usize>,
    pub count: i32,
    /// bitfield `uint32_t c:8` + `uint32_t n:24` packed into one u32
    pub cn: u32,
}

impl trie_t {
    #[inline]
    fn get_c(&self) -> u32 {
        self.cn & 0xff
    }
    #[inline]
    fn set_c(&mut self, c: u32) {
        self.cn = (self.cn & !0xff) | (c & 0xff);
    }
    #[inline]
    fn get_n(&self) -> u32 {
        (self.cn >> 8) & 0xff_ffff
    }
    #[inline]
    fn set_n(&mut self, n: u32) {
        self.cn = (self.cn & 0xff) | ((n & 0xff_ffff) << 8);
    }
}

/// ```c
/// typedef struct {
///     enum name_type token_type;
///     int token_int;
///     int token_str;
/// } last_context_tok;
/// ```
// tokenise_name3.c:131
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct last_context_tok {
    pub token_type: name_type,
    pub token_int: i32,
    pub token_str: i32,
}

/// Per-name history used for duplicate and token-match encoding.
#[derive(Debug, Default)]
pub struct last_context {
    pub last_name: Vec<u8>,
    pub last_ntok: i32,
    pub last: Vec<last_context_tok>,
}

/// Descriptor payload plus cursor/metadata for a token stream.
#[derive(Debug, Clone, Default)]
pub struct descriptor {
    pub buf: Vec<u8>,
    /// used length while encoding; read cursor while decoding.
    pub buf_l: usize,
    pub tnum: i32,
    pub ttype: i32,
    pub dup_from: i32,
}

/// Tokenisation context.  Most state is Rust-owned; the trie still uses the
/// translated pooled allocator.
pub struct name_context {
    pub lc: Vec<last_context>,

    /// For finding entire line dups
    pub counter: i32,

    /// Trie used in encoder only.  Node 0 (when present) is the head; all nodes
    /// live in this arena and reference each other by index.  Empty means no
    /// trie has been built yet.
    pub trie: Vec<trie_t>,

    /// token blocks
    pub desc: Vec<descriptor>,

    /// summary stats per token
    pub token_dcount: [i32; MAX_TOKENS],
    pub token_icount: [i32; MAX_TOKENS],

    /// tracks which desc/[id]count elements have been initialised
    pub max_tok: i32,
    pub max_names: i32,
}

//-----------------------------------------------------------------------------
// ctype helpers (matching C's <ctype.h> behaviour for 7-bit ASCII)

#[inline]
fn isalpha(c: i32) -> bool {
    (c >= 'a' as i32 && c <= 'z' as i32) || (c >= 'A' as i32 && c <= 'Z' as i32)
}
#[inline]
fn isdigit(c: i32) -> bool {
    c >= '0' as i32 && c <= '9' as i32
}
#[inline]
fn isxdigit(c: i32) -> bool {
    isdigit(c) || (c >= 'a' as i32 && c <= 'f' as i32) || (c >= 'A' as i32 && c <= 'F' as i32)
}
#[inline]
fn isspace(c: i32) -> bool {
    c == ' ' as i32
        || c == '\t' as i32
        || c == '\n' as i32
        || c == 11
        || c == 12
        || c == '\r' as i32
}
#[inline]
fn ispunct(c: i32) -> bool {
    // C ispunct: printable, not space, not alnum
    (0x21..=0x7e).contains(&c) && !isalpha(c) && !isdigit(c)
}

//-----------------------------------------------------------------------------
// Context create / destroy

/// `static name_context *create_context(int max_names)`
// tokenise_name3.c:172
pub fn create_context(mut max_names: i32) -> Option<name_context> {
    if max_names <= 0 {
        return None;
    }

    if max_names as f64 > 1e7 {
        eprintln!("Name codec currently has a max of 10 million rec.");
        return None;
    }

    max_names += 1;
    Some(name_context {
        lc: (0..max_names).map(|_| last_context::default()).collect(),
        counter: 0,
        trie: Vec::new(),
        desc: (0..MAX_TBLOCKS).map(|_| descriptor::default()).collect(),
        token_dcount: [0; MAX_TOKENS],
        token_icount: [0; MAX_TOKENS],
        max_tok: 1,
        max_names,
    })
}

// tokenise_name3.c:211: the original `free_context` is no longer needed; the
// owned `name_context` (including its trie arena) is freed by Drop when the
// value goes out of scope.

//-----------------------------------------------------------------------------
// Fast unsigned integer printing code.
// Returns number of bytes written.

/// `static int append_uint32_fixed(char *cp, uint32_t i, uint8_t l)`
// tokenise_name3.c:233
pub fn append_uint32_fixed(cp: &mut [u8], mut i: u32, l: u8) -> i32 {
    let mut o = 0usize;
    macro_rules! emit {
        ($div:expr, $mod:expr) => {
            cp[o] = (i / $div) as u8 + b'0';
            o += 1;
            i %= $mod;
        };
    }
    // C uses a switch with fall-through from `l` downwards.
    if l >= 9 {
        emit!(100000000, 100000000);
    }
    if l >= 8 {
        emit!(10000000, 10000000);
    }
    if l >= 7 {
        emit!(1000000, 1000000);
    }
    if l >= 6 {
        emit!(100000, 100000);
    }
    if l >= 5 {
        emit!(10000, 10000);
    }
    if l >= 4 {
        emit!(1000, 1000);
    }
    if l >= 3 {
        emit!(100, 100);
    }
    if l >= 2 {
        emit!(10, 10);
    }
    if l >= 1 {
        cp[o] = i as u8 + b'0';
    }
    l as i32
}

/// `static int append_uint32_var(char *cp, uint32_t i)`
// tokenise_name3.c:249
pub fn append_uint32_var(cp: &mut [u8], i: u32) -> i32 {
    // The C goto-machinery is just an optimised unsigned-decimal print with no
    // leading zeros.  Crucially, for i == 0 it emits ZERO bytes (the final
    // `if (i) *cp++ = i+'0'` is false), so we reproduce that exactly.
    let z = b'0';
    if i == 0 {
        return 0;
    }
    let mut buf = [0u8; 10];
    let mut n = 0usize;
    let mut v = i;
    while v != 0 {
        buf[n] = (v % 10) as u8;
        v /= 10;
        n += 1;
    }
    for o in 0..n {
        cp[o] = buf[n - 1 - o] + z;
    }
    n as i32
}

//-----------------------------------------------------------------------------
// Descriptor encoding and IO

/// `static int descriptor_grow(descriptor *fd, uint32_t sz)`
// tokenise_name3.c:299
pub fn descriptor_grow(fd: &mut descriptor, sz: u32) -> i32 {
    let needed = fd.buf_l.saturating_add(sz as usize);
    if needed > fd.buf.capacity() {
        let target = needed.max(if fd.buf.capacity() != 0 {
            fd.buf.capacity() * 2
        } else {
            65536
        });
        fd.buf.reserve(target - fd.buf.capacity());
    }
    0
}

/// `static int encode_token_type(name_context *ctx, int ntok, enum name_type type)`
// tokenise_name3.c:312
pub fn encode_token_type(ctx: &mut name_context, ntok: i32, r#type: name_type) -> i32 {
    let id = (ntok << 4) as usize;

    if descriptor_grow(&mut ctx.desc[id], 1) < 0 {
        return -1;
    }
    ctx.desc[id].buf.push(r#type as i32 as u8);
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int encode_token_match(name_context *ctx, int ntok)`
// tokenise_name3.c:323
pub fn encode_token_match(ctx: &mut name_context, ntok: i32) -> i32 {
    encode_token_type(ctx, ntok, N_MATCH)
}

/// `static int encode_token_end(name_context *ctx, int ntok)`
// tokenise_name3.c:327
pub fn encode_token_end(ctx: &mut name_context, ntok: i32) -> i32 {
    encode_token_type(ctx, ntok, N_END)
}

/// `static enum name_type decode_token_type(name_context *ctx, int ntok)`
// tokenise_name3.c:331
pub fn decode_token_type(ctx: &mut name_context, ntok: i32) -> name_type {
    let id = (ntok << 4) as usize;
    if ctx.desc[id].buf_l >= ctx.desc[id].buf.len() {
        return N_ERR;
    }
    let v = ctx.desc[id].buf[ctx.desc[id].buf_l];
    ctx.desc[id].buf_l += 1;
    name_type::from_i32(v as i32)
}

/// `static int encode_token_int(name_context *ctx, int ntok, enum name_type type, uint32_t val)`
// tokenise_name3.c:338
pub fn encode_token_int(ctx: &mut name_context, ntok: i32, r#type: name_type, val: u32) -> i32 {
    let id = ((ntok << 4) | r#type as i32) as usize;

    if encode_token_type(ctx, ntok, r#type) < 0 {
        return -1;
    }
    if descriptor_grow(&mut ctx.desc[id], 4) < 0 {
        return -1;
    }

    ctx.desc[id].buf.extend_from_slice(&val.to_le_bytes());
    ctx.desc[id].buf_l += 4;
    0
}

/// `static int decode_token_int(name_context *ctx, int ntok, enum name_type type, uint32_t *val)`
// tokenise_name3.c:356
pub fn decode_token_int(
    ctx: &mut name_context,
    ntok: i32,
    r#type: name_type,
    val: &mut u32,
) -> i32 {
    let id = ((ntok << 4) | r#type as i32) as usize;

    if ctx.desc[id].buf_l + 4 > ctx.desc[id].buf.len() {
        return -1;
    }
    *val = u32::from_le_bytes(
        ctx.desc[id].buf[ctx.desc[id].buf_l..ctx.desc[id].buf_l + 4]
            .try_into()
            .unwrap(),
    );
    ctx.desc[id].buf_l += 4;
    0
}

/// `static int encode_token_int1(name_context *ctx, int ntok, enum name_type type, uint32_t val)`
// tokenise_name3.c:371
pub fn encode_token_int1(ctx: &mut name_context, ntok: i32, r#type: name_type, val: u32) -> i32 {
    let id = ((ntok << 4) | r#type as i32) as usize;

    if encode_token_type(ctx, ntok, r#type) < 0 {
        return -1;
    }
    if descriptor_grow(&mut ctx.desc[id], 1) < 0 {
        return -1;
    }
    ctx.desc[id].buf.push(val as u8);
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int encode_token_int1_(name_context *ctx, int ntok, enum name_type type, uint32_t val)`
// tokenise_name3.c:383
pub fn encode_token_int1_(ctx: &mut name_context, ntok: i32, r#type: name_type, val: u32) -> i32 {
    let id = ((ntok << 4) | r#type as i32) as usize;

    if descriptor_grow(&mut ctx.desc[id], 1) < 0 {
        return -1;
    }
    ctx.desc[id].buf.push(val as u8);
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int decode_token_int1(name_context *ctx, int ntok, enum name_type type, uint32_t *val)`
// tokenise_name3.c:395
pub fn decode_token_int1(
    ctx: &mut name_context,
    ntok: i32,
    r#type: name_type,
    val: &mut u32,
) -> i32 {
    let id = ((ntok << 4) | r#type as i32) as usize;

    if ctx.desc[id].buf_l >= ctx.desc[id].buf.len() {
        return -1;
    }
    *val = ctx.desc[id].buf[ctx.desc[id].buf_l] as u32;
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int encode_token_alpha(name_context *ctx, int ntok, char *str, int len)`
// tokenise_name3.c:411
pub fn encode_token_alpha(ctx: &mut name_context, ntok: i32, str: &[u8], len: i32) -> i32 {
    let id = ((ntok << 4) | N_ALPHA as i32) as usize;

    if encode_token_type(ctx, ntok, N_ALPHA) < 0 {
        return -1;
    }
    if descriptor_grow(&mut ctx.desc[id], (len + 1) as u32) < 0 {
        return -1;
    }
    ctx.desc[id].buf.extend_from_slice(&str[..len as usize]);
    ctx.desc[id].buf.push(0);
    ctx.desc[id].buf_l += (len + 1) as usize;
    0
}

/// `static int decode_token_alpha(name_context *ctx, int ntok, char *str, int max_len)`
// tokenise_name3.c:426
pub fn decode_token_alpha(
    ctx: &mut name_context,
    ntok: i32,
    str: &mut [u8],
    max_len: i32,
) -> i32 {
    let id = ((ntok << 4) | N_ALPHA as i32) as usize;
    let mut len = 0i32;
    if ctx.desc[id].buf_l >= ctx.desc[id].buf.len() {
        return -1;
    }
    let mut c: u8;
    loop {
        c = ctx.desc[id].buf[ctx.desc[id].buf_l];
        ctx.desc[id].buf_l += 1;
        str[len as usize] = c;
        len += 1;
        if !(c != 0 && len < max_len && ctx.desc[id].buf_l < ctx.desc[id].buf.len()) {
            break;
        }
    }
    len - 1
}

/// `static int encode_token_char(name_context *ctx, int ntok, char c)`
// tokenise_name3.c:440
pub fn encode_token_char(ctx: &mut name_context, ntok: i32, c: u8) -> i32 {
    let id = ((ntok << 4) | N_CHAR as i32) as usize;

    if encode_token_type(ctx, ntok, N_CHAR) < 0 {
        return -1;
    }
    if descriptor_grow(&mut ctx.desc[id], 1) < 0 {
        return -1;
    }
    ctx.desc[id].buf.push(c);
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int decode_token_char(name_context *ctx, int ntok, char *str)`
// tokenise_name3.c:452
pub fn decode_token_char(ctx: &mut name_context, ntok: i32, str: &mut u8) -> i32 {
    let id = ((ntok << 4) | N_CHAR as i32) as usize;

    if ctx.desc[id].buf_l >= ctx.desc[id].buf.len() {
        return -1;
    }
    *str = ctx.desc[id].buf[ctx.desc[id].buf_l];
    ctx.desc[id].buf_l += 1;
    1
}

/// `static int encode_token_dup(name_context *ctx, uint32_t val)`
// tokenise_name3.c:464
pub fn encode_token_dup(ctx: &mut name_context, val: u32) -> i32 {
    encode_token_int(ctx, 0, N_DUP, val)
}

/// `static int encode_token_diff(name_context *ctx, uint32_t val)`
// tokenise_name3.c:469
pub fn encode_token_diff(ctx: &mut name_context, val: u32) -> i32 {
    encode_token_int(ctx, 0, N_DIFF, val)
}

#[inline]
fn reset_desc_block(ctx: &mut name_context, token: i32) {
    for desc in &mut ctx.desc[(token << 4) as usize..((token + 1) << 4) as usize] {
        *desc = descriptor::default();
    }
}

#[inline]
fn ensure_last_tokens(lc: &mut last_context, len: usize) {
    if lc.last.len() <= len {
        lc.last.resize(
            len + 1,
            last_context_tok {
                token_type: N_NOP,
                token_int: 0,
                token_str: 0,
            },
        );
    }
}

fn seed_type_descriptor(desc: &mut descriptor, nreads: i32, ttype: i32) {
    desc.buf.clear();
    if nreads > 0 {
        desc.buf.push((ttype & 15) as u8);
        desc.buf.resize(nreads as usize, N_MATCH as i32 as u8);
    }
    desc.buf_l = 0;
}

//-----------------------------------------------------------------------------
// Trie implementation for tracking common name prefixes.

/// `static int build_trie(name_context *ctx, char *data, size_t len, int n)`
// tokenise_name3.c:476
pub fn build_trie(ctx: &mut name_context, data: &[u8], len: usize, n: i32) -> i32 {
    if ctx.trie.is_empty() {
        // Node 0 is the head.
        ctx.trie.push(trie_t::default());
    }

    let mut i = 0usize;
    while i < len {
        let mut t = 0usize; // head
        ctx.trie[t].count += 1;
        while i < len && data[i] > b'\n' {
            let c0 = data[i];
            i += 1;
            if c0 & 0x80 != 0 {
                return -1;
            }
            let c = (c0 & 127) as u32;

            // Walk the sibling chain of t.next looking for a node with char `c`.
            let mut x = ctx.trie[t].next;
            let mut l: Option<usize> = None;
            while let Some(xi) = x {
                if ctx.trie[xi].get_c() == c {
                    break;
                }
                l = x;
                x = ctx.trie[xi].sibling;
            }
            let xi = match x {
                Some(xi) => xi,
                None => {
                    // Allocate a fresh node in the arena.
                    let new = ctx.trie.len();
                    ctx.trie.push(trie_t::default());
                    match l {
                        None => ctx.trie[t].next = Some(new),
                        Some(li) => ctx.trie[li].sibling = Some(new),
                    }
                    ctx.trie[new].set_n(n as u32);
                    ctx.trie[new].set_c(c);
                    new
                }
            };
            t = xi;
            ctx.trie[t].set_c(c);
            ctx.trie[t].count += 1;
        }
        i += 1;
    }

    0
}

/// `static int search_trie(name_context *ctx, char *data, size_t len, int n, int *exact, int *is_fixed, int *fixed_len)`
// tokenise_name3.c:589
pub fn search_trie(
    ctx: &mut name_context,
    data: &[u8],
    len: usize,
    n: i32,
    exact: &mut i32,
    is_fixed: &mut i32,
    fixed_len: &mut i32,
) -> i32 {
    let mut from: i32 = -1;
    let mut p3: i32 = -1;
    *exact = 0;
    *fixed_len = 0;
    *is_fixed = 0;

    let prefix_len: usize;
    // char *d = *data == '@' ? data+1 : data;
    let at = data[0] == b'@';
    let d_off = if at { 1usize } else { 0 };
    let l = if at { (len as i32) - 1 } else { len as i32 };
    let f = if data[0] == b'>' { 1usize } else { 0 };

    let db = |k: usize| -> i32 { data[d_off + k] as i32 };

    if l > 70
        && db(f) == b'm' as i32
        && data[7] == b'_'
        && db(f + 14) == b'_' as i32
        && db(f + 61) == b'/' as i32
    {
        prefix_len = 60; // PacBio
        *is_fixed = 0;
    } else if l == 17 && db(f + 5) == b':' as i32 && db(f + 11) == b':' as i32 {
        prefix_len = 6; // IonTorrent
        *fixed_len = 6;
        *is_fixed = 1;
    } else if l >= 36
        && db(f + 8) == b'-' as i32
        && db(f + 13) == b'-' as i32
        && db(f + 18) == b'-' as i32
        && db(f + 23) == b'-' as i32
        && isxdigit(db(f))
        && isxdigit(db(f + 7))
        && isxdigit(db(f + 9))
        && isxdigit(db(f + 12))
        && isxdigit(db(f + 14))
        && isxdigit(db(f + 17))
        && isxdigit(db(f + 19))
        && isxdigit(db(f + 22))
        && isxdigit(db(f + 24))
        && isxdigit(db(f + 35))
    {
        // ONT: f33d30d5-6eb8-4115-8f46-154c2620a5da_Basecall_1D_template...
        // (htslib v1.23: l >= 36, ten isxdigit checks, prefix/fixed_len = 36)
        prefix_len = 36;
        *fixed_len = 36;
        *is_fixed = 1;
    } else {
        // Check Illumina and trim back to lane:tile:x:y.
        let mut colons = 0i32;
        let mut i = 0usize;
        while i < len && data[i] > b' ' {
            i += 1;
        }
        while i > 0 && colons < 4 {
            i -= 1;
            if data[i] == b':' {
                colons += 1;
            }
        }

        if colons == 4 {
            *fixed_len = (i + 1) as i32;
            prefix_len = i + 1;
            *is_fixed = 1;
        } else {
            prefix_len = usize::MAX; // INT_MAX
            *is_fixed = 0;
        }
    }

    if ctx.trie.is_empty() {
        ctx.trie.push(trie_t::default());
    }

    // INT_MAX representation for prefix_len comparison `i == prefix_len`
    let prefix_len_i: i64 = if prefix_len == usize::MAX {
        i32::MAX as i64
    } else {
        prefix_len as i64
    };

    // Find an item in the trie
    let mut from_punct: i32 = from;
    let mut i = 0usize;
    while i < len {
        let mut t = 0usize; // head
        while i < len && data[i] > b'\n' {
            let c0 = data[i];
            i += 1;
            if c0 & 0x80 != 0 {
                return -1;
            }
            let c = (c0 & 127) as u32;

            let mut x = ctx.trie[t].next;
            while let Some(xi) = x {
                if ctx.trie[xi].get_c() == c {
                    break;
                }
                x = ctx.trie[xi].sibling;
            }
            // In valid input every traversed char was inserted by build_trie, so
            // the node exists; absence would have been UB in the original C.
            let xi = match x {
                Some(xi) => xi,
                None => return -1,
            };
            t = xi;

            from = ctx.trie[t].get_n() as i32;
            if (ispunct(c as i32) || isspace(c as i32)) && from != n {
                from_punct = ctx.trie[t].get_n() as i32;
            }
            if i as i64 == prefix_len_i {
                p3 = ctx.trie[t].get_n() as i32;
            }
            ctx.trie[t].set_n(n as u32);
        }
        i += 1;
    }

    // htslib v1.23: *exact = (n != from) && len;
    //               return *exact ? from : (p3 != -1 ? p3 : from_punct);
    *exact = ((n != from) && len != 0) as i32;
    if *exact != 0 {
        from
    } else if p3 != -1 {
        p3
    } else {
        from_punct
    }
}

//-----------------------------------------------------------------------------
// Name encoder

/// `static int encode_name(name_context *ctx, char *name, int len, int mode)`
// tokenise_name3.c:695
#[allow(unused_unsafe)]
pub fn encode_name(ctx: &mut name_context, name: &mut [u8], len: i32, mode: i32) -> i32 {
    let mut is_fixed = 0i32;
    let mut fixed_len = 0i32;
    let mut exact = 0i32;
    let cnum = ctx.counter;
    ctx.counter += 1;
    let mut pnum = search_trie(
        ctx,
        name,
        len as usize,
        cnum,
        &mut exact,
        &mut is_fixed,
        &mut fixed_len,
    );
    if pnum < 0 {
        pnum = if cnum != 0 { cnum - 1 } else { 0 };
    }

    // Return DUP or DIFF switch, plus the distance.
    let pnum_strlen = ctx.lc[pnum as usize].last_name.len();
    if exact != 0 && len as usize == pnum_strlen {
        encode_token_dup(ctx, (cnum - pnum) as u32);
        let last_ntok_p = ctx.lc[pnum as usize].last_ntok;
        let last_p = ctx.lc[pnum as usize].last.clone();
        let cl = &mut ctx.lc[cnum as usize];
        cl.last_name = name[..len as usize].to_vec();
        cl.last_ntok = last_ntok_p;
        cl.last = last_p;
        return 0;
    }

    ctx.lc[cnum as usize].last.clear();
    ensure_last_tokens(&mut ctx.lc[cnum as usize], MAX_TOKENS);
    encode_token_diff(ctx, (cnum - pnum) as u32);
    let mut ntok = 1i32;

    // closures to read name bytes
    let nb = |k: usize| -> i32 { name[k] as i32 };

    // htslib v1.23: dedicated `fixed_len == 36` ONT uuid4 char-block path,
    // then the generic `is_fixed` path, else `i = 0`.
    let mut i: i32 = 0;

    if fixed_len == 36 {
        // ONT uuid4 format data
        if 37 >= ctx.max_tok {
            loop {
                let mt = ctx.max_tok;
                reset_desc_block(ctx, mt);
                ctx.token_dcount[mt as usize] = 0;
                ctx.token_icount[mt as usize] = 0;
                let old = ctx.max_tok;
                ctx.max_tok += 1;
                if old >= 37 {
                    break;
                }
            }
        }
        let mut k = 0i32;
        while k < 36 {
            encode_token_char(ctx, ntok, nb(k as usize) as u8);
            let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
            lct.token_int = nb(k as usize);
            lct.token_type = N_CHAR;
            k += 1;
            ntok += 1;
        }
        is_fixed = 0;
        i = 36;
    } else if is_fixed != 0 {
        if ntok >= ctx.max_tok {
            let mt = ctx.max_tok;
            reset_desc_block(ctx, mt);
            ctx.token_dcount[mt as usize] = 0;
            ctx.token_icount[mt as usize] = 0;
            ctx.max_tok = ntok + 1;
        }
        let use_match = if pnum < cnum
            && ntok < ctx.lc[pnum as usize].last_ntok
            && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_ALPHA
        {
            let pl = ctx.lc[pnum as usize].last[ntok as usize];
            let pname = &ctx.lc[pnum as usize].last_name;
            pl.token_int == fixed_len && name[..fixed_len as usize] == pname[..fixed_len as usize]
        } else {
            false
        };
        if pnum < cnum
            && ntok < ctx.lc[pnum as usize].last_ntok
            && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_ALPHA
        {
            if use_match {
                encode_token_match(ctx, ntok);
            } else {
                encode_token_alpha(ctx, ntok, name, fixed_len);
            }
        } else {
            encode_token_alpha(ctx, ntok, name, fixed_len);
        }
        let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
        lct.token_int = fixed_len;
        lct.token_str = 0;
        lct.token_type = N_ALPHA;
        ntok += 1;
        i = fixed_len;
    } else {
        i = 0;
    }

    while i < len {
        if ntok >= ctx.max_tok {
            if ctx.max_tok >= MAX_TOKENS as i32 {
                return -1;
            }
            let mt = ctx.max_tok;
            reset_desc_block(ctx, mt);
            ctx.token_dcount[mt as usize] = 0;
            ctx.token_icount[mt as usize] = 0;
            ctx.max_tok = ntok + 1;
        }

        let ci = nb(i as usize);
        // We use a flag to emulate the `goto n_char` and `goto digits0`.
        let mut do_char = false;
        let mut do_digits0 = false;

        if isalpha(ci) {
            let mut s = i + 1;
            while s < len && (isalpha(nb(s as usize)) || ispunct(nb(s as usize))) {
                s += 1;
            }

            if s - i == 1 {
                do_char = true;
            } else {
                let matched = pnum < cnum
                    && ntok < ctx.lc[pnum as usize].last_ntok
                    && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_ALPHA;
                if matched {
                    let pl = ctx.lc[pnum as usize].last[ntok as usize];
                    let pname = &ctx.lc[pnum as usize].last_name;
                    let is_match = s - i == pl.token_int
                        && name[i as usize..s as usize]
                            == pname[pl.token_str as usize..(pl.token_str + pl.token_int) as usize];
                    if is_match {
                        if encode_token_match(ctx, ntok) < 0 {
                            return -1;
                        }
                    } else if encode_token_alpha(ctx, ntok, &name[i as usize..], s - i) < 0 {
                        return -1;
                    }
                } else if encode_token_alpha(ctx, ntok, &name[i as usize..], s - i) < 0 {
                    return -1;
                }

                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_int = s - i;
                lct.token_str = i;
                lct.token_type = N_ALPHA;

                i = s - 1;
            }
        } else if ci == b'0' as i32 {
            do_digits0 = true;
        } else if isdigit(ci) {
            // digits starting 1-9; encode value
            let mut s = i as u32;
            let mut v: u32 = 0;
            let d: i32;
            let lenu = len as u32;
            while s < lenu && isdigit(nb(s as usize)) && s - i as u32 <= 8 {
                v = v
                    .wrapping_mul(10)
                    .wrapping_add((nb(s as usize) - b'0' as i32) as u32);
                s += 1;
            }

            if pnum < cnum
                && ntok < ctx.lc[pnum as usize].last_ntok
                && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_DIGITS0
                && ctx.lc[pnum as usize].last[ntok as usize].token_str as u32 == s - i as u32
            {
                do_digits0 = true;
            } else {
                if pnum < cnum
                    && ntok < ctx.lc[pnum as usize].last_ntok
                    && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_DIGITS
                {
                    d = v as i32 - ctx.lc[pnum as usize].last[ntok as usize].token_int;
                    if d == 0 {
                        if encode_token_match(ctx, ntok) < 0 {
                            return -1;
                        }
                    } else if mode == 1
                        && (0..256).contains(&d)
                        && (5 + ctx.token_dcount[ntok as usize]) > ctx.token_icount[ntok as usize]
                    {
                        if encode_token_int1(ctx, ntok, N_DDELTA, d as u32) < 0 {
                            return -1;
                        }
                        ctx.token_dcount[ntok as usize] += 1;
                    } else {
                        if encode_token_int(ctx, ntok, N_DIGITS, v) < 0 {
                            return -1;
                        }
                        ctx.token_icount[ntok as usize] += 1;
                    }
                } else if encode_token_int(ctx, ntok, N_DIGITS, v) < 0 {
                    return -1;
                }

                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_int = v as i32;
                lct.token_type = N_DIGITS;

                i = s as i32 - 1;
            }
        } else {
            do_char = true;
        }

        if do_digits0 {
            // Digits starting with zero; encode length + value
            let mut s = i as u32;
            let mut v: u32 = 0;
            let d: i32;
            let lenu = len as u32;
            while s < lenu && isdigit(nb(s as usize)) && s - i as u32 <= 8 {
                v = v
                    .wrapping_mul(10)
                    .wrapping_add((nb(s as usize) - b'0' as i32) as u32);
                s += 1;
            }

            if pnum < cnum
                && ntok < ctx.lc[pnum as usize].last_ntok
                && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_DIGITS0
            {
                d = v as i32 - ctx.lc[pnum as usize].last[ntok as usize].token_int;
                let pstr = ctx.lc[pnum as usize].last[ntok as usize].token_str;
                if d == 0 && pstr as u32 == s - i as u32 {
                    if encode_token_match(ctx, ntok) < 0 {
                        return -1;
                    }
                } else if mode == 1 && (0..256).contains(&d) && pstr as u32 == s - i as u32 {
                    if encode_token_int1(ctx, ntok, N_DDELTA0, d as u32) < 0 {
                        return -1;
                    }
                } else {
                    if encode_token_int1_(ctx, ntok, N_DZLEN, s - i as u32) < 0 {
                        return -1;
                    }
                    if encode_token_int(ctx, ntok, N_DIGITS0, v) < 0 {
                        return -1;
                    }
                }
            } else {
                if encode_token_int1_(ctx, ntok, N_DZLEN, s - i as u32) < 0 {
                    return -1;
                }
                if encode_token_int(ctx, ntok, N_DIGITS0, v) < 0 {
                    return -1;
                }
            }

            let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
            lct.token_str = (s - i as u32) as i32; // length
            lct.token_int = v as i32;
            lct.token_type = N_DIGITS0;

            i = s as i32 - 1;
        }

        if do_char {
            let ci = nb(i as usize);
            if pnum < cnum
                && ntok < ctx.lc[pnum as usize].last_ntok
                && ctx.lc[pnum as usize].last[ntok as usize].token_type == N_CHAR
            {
                if ci == ctx.lc[pnum as usize].last[ntok as usize].token_int {
                    if encode_token_match(ctx, ntok) < 0 {
                        return -1;
                    }
                } else if encode_token_char(ctx, ntok, ci as u8) < 0 {
                    return -1;
                }
            } else if encode_token_char(ctx, ntok, ci as u8) < 0 {
                return -1;
            }

            let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
            lct.token_int = ci;
            lct.token_type = N_CHAR;
        }

        ntok += 1;
        i += 1;
    }

    if ntok >= ctx.max_tok {
        if ctx.max_tok >= MAX_TOKENS as i32 {
            return -1;
        }
        let mt = ctx.max_tok;
        reset_desc_block(ctx, mt);
        ctx.token_dcount[mt as usize] = 0;
        ctx.token_icount[mt as usize] = 0;
        ctx.max_tok = ntok + 1;
    }
    if encode_token_end(ctx, ntok) < 0 {
        return -1;
    }

    let cl = &mut ctx.lc[cnum as usize];
    cl.last_name = name[..len as usize].to_vec();
    cl.last_ntok = ntok;
    cl.last.truncate((ntok + 1) as usize);

    0
}

//-----------------------------------------------------------------------------
// Name decoder

/// `static int decode_name(name_context *ctx, char *name, int name_len)`
// tokenise_name3.c:1021
#[allow(unused_unsafe)]
pub fn decode_name(ctx: &mut name_context, name: &mut [u8], name_len: i32) -> i32 {
    let t0v = decode_token_type(ctx, 0) as i32;
    let mut dist: u32 = 0;
    let cnum = ctx.counter;
    ctx.counter += 1;

    if cnum >= ctx.max_names {
        return -1;
    }
    if t0v < 0 || t0v >= ctx.max_tok * 16 {
        return 0;
    }
    let t0 = name_type::from_i32(t0v);

    if decode_token_int(ctx, 0, t0, &mut dist) < 0 || dist > cnum as u32 {
        return -1;
    }
    let mut pnum = cnum - dist as i32;
    if pnum < 0 {
        pnum = 0;
    }

    if t0 == N_DUP {
        if pnum == cnum {
            return -1;
        }
        let plen = ctx.lc[pnum as usize].last_name.len();
        if plen + 1 >= name_len as usize {
            return -1;
        }
        name[..plen].copy_from_slice(&ctx.lc[pnum as usize].last_name);
        name[plen] = 0;
        let last_ntok = ctx.lc[pnum as usize].last_ntok;
        let last_p = ctx.lc[pnum as usize].last.clone();
        let last_name = ctx.lc[pnum as usize].last_name.clone();
        let cl = &mut ctx.lc[cnum as usize];
        cl.last_name = last_name;
        cl.last_ntok = last_ntok;
        cl.last = last_p;
        return (plen + 1) as i32;
    }

    name[0] = 0;
    let mut len = 0i32;
    ctx.lc[cnum as usize].last.clear();
    ensure_last_tokens(&mut ctx.lc[cnum as usize], MAX_TOKENS);

    let mut ntok = 1i32;
    while ntok < MAX_TOKENS as i32 && ntok < ctx.max_tok {
        let tok = decode_token_type(ctx, ntok);
        ctx.lc[cnum as usize].last_ntok = 0;

        match tok {
            N_CHAR => {
                if len + 1 >= name_len {
                    return -1;
                }
                let mut c: u8 = 0;
                if decode_token_char(ctx, ntok, &mut c) < 0 {
                    return -1;
                }
                name[len as usize] = c;
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_CHAR;
                lct.token_int = c as i32;
                len += 1;
            }
            N_ALPHA => {
                let len2 = decode_token_alpha(ctx, ntok, &mut name[len as usize..], name_len - len);
                if len2 < 0 {
                    return -1;
                }
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_ALPHA;
                lct.token_str = len;
                lct.token_int = len2;
                len += len2;
            }
            N_DIGITS0 => {
                let mut vl: u32 = 0;
                let mut v: u32 = 0;
                if decode_token_int1(ctx, ntok, N_DZLEN, &mut vl) < 0 {
                    return -1;
                }
                if decode_token_int(ctx, ntok, N_DIGITS0, &mut v) < 0 {
                    return -1;
                }
                if len as u32 + 20 + vl >= name_len as u32 {
                    return -1;
                }
                len += append_uint32_fixed(&mut name[len as usize..], v, vl as u8);
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_DIGITS0;
                lct.token_int = v as i32;
                lct.token_str = vl as i32;
            }
            N_DDELTA0 => {
                if ntok >= ctx.lc[pnum as usize].last_ntok {
                    return -1;
                }
                let mut v: u32 = 0;
                if decode_token_int1(ctx, ntok, N_DDELTA0, &mut v) < 0 {
                    return -1;
                }
                let pl = ctx.lc[pnum as usize].last[ntok as usize];
                v = v.wrapping_add(pl.token_int as u32);
                if len + pl.token_str + 1 >= name_len {
                    return -1;
                }
                len += append_uint32_fixed(&mut name[len as usize..], v, pl.token_str as u8);
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_DIGITS0;
                lct.token_int = v as i32;
                lct.token_str = pl.token_str;
            }
            N_DIGITS => {
                let mut v: u32 = 0;
                if decode_token_int(ctx, ntok, N_DIGITS, &mut v) < 0 {
                    return -1;
                }
                if len + 20 >= name_len {
                    return -1;
                }
                len += append_uint32_var(&mut name[len as usize..], v);
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_DIGITS;
                lct.token_int = v as i32;
            }
            N_DDELTA => {
                if ntok >= ctx.lc[pnum as usize].last_ntok {
                    return -1;
                }
                let mut v: u32 = 0;
                if decode_token_int1(ctx, ntok, N_DDELTA, &mut v) < 0 {
                    return -1;
                }
                let pl = ctx.lc[pnum as usize].last[ntok as usize];
                v = v.wrapping_add(pl.token_int as u32);
                if len + 20 >= name_len {
                    return -1;
                }
                len += append_uint32_var(&mut name[len as usize..], v);
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_DIGITS;
                lct.token_int = v as i32;
            }
            N_NOP => {
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_NOP;
            }
            N_MATCH => {
                if ntok >= ctx.lc[pnum as usize].last_ntok {
                    return -1;
                }
                let pl = ctx.lc[pnum as usize].last[ntok as usize];
                match pl.token_type {
                    N_CHAR => {
                        if len + 1 >= name_len {
                            return -1;
                        }
                        name[len as usize] = pl.token_int as u8;
                        len += 1;
                        let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                        lct.token_type = N_CHAR;
                        lct.token_int = pl.token_int;
                    }
                    N_ALPHA => {
                        if pl.token_int < 0 || len + pl.token_int >= name_len {
                            return -1;
                        }
                        let start = pl.token_str as usize;
                        let end = start + pl.token_int as usize;
                        name[len as usize..len as usize + pl.token_int as usize]
                            .copy_from_slice(&ctx.lc[pnum as usize].last_name[start..end]);
                        let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                        lct.token_type = N_ALPHA;
                        lct.token_str = len;
                        lct.token_int = pl.token_int;
                        len += pl.token_int;
                    }
                    N_DIGITS => {
                        if len + 20 >= name_len {
                            return -1;
                        }
                        len += append_uint32_var(&mut name[len as usize..], pl.token_int as u32);
                        let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                        lct.token_type = N_DIGITS;
                        lct.token_int = pl.token_int;
                    }
                    N_DIGITS0 => {
                        if len + pl.token_str >= name_len {
                            return -1;
                        }
                        len += append_uint32_fixed(
                            &mut name[len as usize..],
                            pl.token_int as u32,
                            pl.token_str as u8,
                        );
                        let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                        lct.token_type = N_DIGITS0;
                        lct.token_int = pl.token_int;
                        lct.token_str = pl.token_str;
                    }
                    _ => {
                        return -1;
                    }
                }
            }
            _ => {
                // default and N_END
                if len + 1 >= name_len {
                    return -1;
                }
                name[len as usize] = 0;
                len += 1;
                let lct = &mut ctx.lc[cnum as usize].last[ntok as usize];
                lct.token_type = N_END;

                let cl = &mut ctx.lc[cnum as usize];
                cl.last_name = name[..(len - 1) as usize].to_vec();
                cl.last_ntok = ntok;
                cl.last.truncate((ntok + 1) as usize);
                return len;
            }
        }
        ntok += 1;
    }

    -1
}

//-----------------------------------------------------------------------------
// arith adaptive codec or static rANS 4x16pr codec

/// `static int arith_encode(uint8_t *in, uint64_t in_len, uint8_t *out, uint64_t *out_len, int method)`
// tokenise_name3.c:1212
pub fn arith_encode(
    r#in: &[u8],
    in_len: u64,
    out: &mut [u8],
    out_len: &mut u64,
    method: i32,
) -> i32 {
    let mut olen = (*out_len - 6) as u32;
    // arith_compress_to(in, in_len, out+6, &olen, method)
    let v = {
        let cap = olen as usize;
        let dst = &mut out[6..6 + cap];
        arith_compress_to(&r#in[..in_len as usize], Some(dst), &mut olen, method)
    };
    if v.is_empty() {
        return -1;
    }
    // The wrapper wrote into out[6..]; copy authoritative bytes back.
    out[6..6 + olen as usize].copy_from_slice(&v[..olen as usize]);

    let nb = var_put_u32(out, Some(*out_len as usize), olen);
    let nb = nb as usize;
    out.copy_within(6..6 + olen as usize, nb);
    *out_len = (olen as usize + nb) as u64;
    0
}

/// `static int64_t arith_decode(uint8_t *in, uint64_t in_len, uint8_t *out, uint64_t *out_len)`
// tokenise_name3.c:1226
pub fn arith_decode(r#in: &[u8], in_len: u64, out: &mut [u8], out_len: &mut u64) -> i64 {
    let mut olen = *out_len as u32;
    let mut clen: u32 = 0;
    let nb = var_get_u32(r#in, Some(in_len as usize), &mut clen);
    let nb = nb as usize;
    let dst_cap = olen as usize;
    let v = {
        let dst = &mut out[..dst_cap];
        arith_uncompress_to(&r#in[nb..in_len as usize], Some(dst), &mut olen)
    };
    if v.is_empty() {
        return -1;
    }
    out[..olen as usize].copy_from_slice(&v[..olen as usize]);
    *out_len = olen as u64;
    (clen as u64 + nb as u64) as i64
}

/// `static int rans_encode(uint8_t *in, uint64_t in_len, uint8_t *out, uint64_t *out_len, int method)`
// tokenise_name3.c:1239
pub fn rans_encode(
    r#in: &[u8],
    in_len: u64,
    out: &mut [u8],
    out_len: &mut u64,
    method: i32,
) -> i32 {
    let mut olen: u32 = 0;
    // rans_compress_to_4x16(in, in_len, out+6, &olen, method)
    let v = rans_compress_4x16(&r#in[..in_len as usize], &mut olen, method);
    if v.is_empty() {
        return -1;
    }
    out[6..6 + olen as usize].copy_from_slice(&v[..olen as usize]);

    let nb = var_put_u32(out, Some(*out_len as usize), olen);
    let nb = nb as usize;
    out.copy_within(6..6 + olen as usize, nb);
    *out_len = (olen as usize + nb) as u64;
    0
}

/// `static int64_t rans_decode(uint8_t *in, uint64_t in_len, uint8_t *out, uint64_t *out_len)`
// tokenise_name3.c:1253
pub fn rans_decode(r#in: &[u8], in_len: u64, out: &mut [u8], out_len: &mut u64) -> i64 {
    let mut olen: u32 = 0;
    let mut clen: u32 = 0;
    let nb = var_get_u32(r#in, Some(in_len as usize), &mut clen);
    let nb = nb as usize;
    let v = rans_uncompress_4x16(&r#in[nb..in_len as usize], &mut olen);
    if v.is_empty() {
        return -1;
    }
    out[..olen as usize].copy_from_slice(&v[..olen as usize]);
    *out_len = olen as u64;
    (clen as u64 + nb as u64) as i64
}

/// `static int compress(uint8_t *in, uint64_t in_len, enum name_type type, int level, int use_arith, uint8_t *out, uint64_t *out_len)`
// tokenise_name3.c:1266
pub fn compress(
    r#in: &[u8],
    in_len: u64,
    r#type: name_type,
    mut level: i32,
    use_arith: i32,
    out: &mut [u8],
    out_len: &mut u64,
) -> i32 {
    let mut best_sz: u64 = u64::MAX;
    let olen = *out_len;
    let mut ret = -1;

    level = (level - 1) / 2;
    level = level.clamp(0, 4);

    // R[5][N_ALL][7]
    let mut r_tab: [[[i32; 7]; N_ALL as usize]; 5] = [
        [
            [1, 128, 0, 0, 0, 0, 0],
            [1, 129, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 8, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 8, 0, 0, 0, 0, 0],
            [1, 8, 0, 0, 0, 0, 0],
            [1, 8, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 128, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
        ],
        [
            [2, 192, 0, 0, 0, 0, 0],
            [2, 129, 1, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [2, 128 + 8, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 192 + 8, 0, 0, 0, 0, 0],
            [1, 128 + 8, 0, 0, 0, 0, 0],
            [1, 192 + 8, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 128, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
        ],
        [
            [2, 192, 0, 0, 0, 0, 0],
            [4, 1, 128, 0, 129, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [2, 200, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 200, 0, 0, 0, 0, 0],
            [2, 192, 200, 0, 0, 0, 0],
            [2, 132, 201, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 128, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
        ],
        [
            [3, 193, 0, 1, 0, 0, 0],
            [5, 128, 1, 128, 0, 129, 0],
            [2, 1, 0, 0, 0, 0, 0],
            [2, 200, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 201, 0, 0, 0, 0, 0],
            [2, 192, 200, 0, 0, 0, 0],
            [2, 132, 201, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 128, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
        ],
        [
            [6, 192, 0, 1, 65, 193, 132],
            [4, 132, 1, 0, 129, 0, 0],
            [3, 1, 0, 192, 0, 0, 0],
            [4, 201, 0, 192, 64, 0, 0],
            [3, 0, 128, 1, 0, 0, 0],
            [1, 201, 0, 0, 0, 0, 0],
            [3, 192, 201, 65, 0, 0, 0],
            [6, 132, 201, 1, 192, 129, 193],
            [3, 1, 0, 192, 0, 0, 0],
            [3, 192, 1, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
        ],
    ];
    // Minor tweak to level 3 DIGITS if arithmetic, to use O(201) instead.
    if use_arith != 0 {
        r_tab[1][N_DIGITS as usize][1] = 201;
    }

    let meth = &mut r_tab[level as usize][r#type as usize];

    let mut last = 0;
    let mut best_dat: Vec<u8> = Vec::new();
    let mut m = 1usize;
    let mut errored = false;
    while m <= meth[0] as usize {
        *out_len = olen;

        if use_arith == 0 && (meth[m] & 4) != 0 {
            meth[m] &= !4;
        }
        if !in_len.is_multiple_of(4) && (meth[m] & 8) != 0 {
            m += 1;
            continue;
        }

        last = 0;
        if use_arith != 0 {
            if arith_encode(r#in, in_len, out, out_len, meth[m]) < 0 {
                errored = true;
                break;
            }
        } else if rans_encode(r#in, in_len, out, out_len, meth[m]) < 0 {
            errored = true;
            break;
        }

        if best_sz > *out_len {
            best_sz = *out_len;
            last = 1;

            if m + 1 > meth[0] as usize {
                break;
            }
            best_dat = out[..best_sz as usize].to_vec();
        }
        m += 1;
    }

    // C: on `goto err` we skip straight to the err: label (no memcpy / out_len).
    if errored {
        return -1;
    }

    if last == 0 {
        out[..best_sz as usize].copy_from_slice(&best_dat[..best_sz as usize]);
    }
    *out_len = best_sz;
    ret = 0;

    ret
}

/// `static uint64_t uncompressed_size(uint8_t *in, uint64_t in_len)`
// tokenise_name3.c:1417
pub fn uncompressed_size(r#in: &[u8], in_len: u64) -> u64 {
    let mut clen: u32 = 0;
    let mut ulen: u32 = 0;
    let nb = var_get_u32(r#in, Some(in_len as usize), &mut clen);
    let nb = nb as usize;
    var_get_u32(&r#in[nb + 1..], Some(in_len as usize - (nb + 1)), &mut ulen);
    ulen as u64
}

/// `static int uncompress(int use_arith, uint8_t *in, uint64_t in_len, uint8_t *out, uint64_t *out_len)`
// tokenise_name3.c:1429
pub fn uncompress(
    use_arith: i32,
    r#in: &[u8],
    in_len: u64,
    out: &mut [u8],
    out_len: &mut u64,
) -> i64 {
    let mut clen: u32 = 0;
    var_get_u32(r#in, Some(in_len as usize), &mut clen);
    if use_arith != 0 {
        arith_decode(r#in, in_len, out, out_len)
    } else {
        rans_decode(r#in, in_len, out, out_len)
    }
}

//-----------------------------------------------------------------------------
// Public API

/// `uint8_t *tok3_encode_names(char *blk, int len, int level, int use_arith, int *out_len, int *last_start_p)`
// tokenise_name3.c:1449
pub fn tok3_encode_names(
    blk: &mut [u8],
    len: i32,
    level: i32,
    use_arith: i32,
    out_len: &mut i32,
    last_start_p: Option<&mut i32>,
) -> Option<Vec<u8>> {
    let mut last_start = 0i32;

    if len < 0 {
        *out_len = 0;
        return None;
    }

    // Count lines
    let mut nreads = 0i32;
    {
        let mut i = 0i32;
        while i < len {
            if blk[i as usize] <= b'\n' {
                nreads += 1;
            }
            i += 1;
        }
    }

    let mut ctx = create_context(nreads)?;
    let ctx = &mut ctx;

    // Construct trie
    let mut ctr = 0i32;
    {
        let mut i = 0i32;
        let mut j = 0i32;
        while i < len {
            while i < len && blk[i as usize] > b'\n' {
                i += 1;
            }
            if i >= len {
                break;
            }
            last_start = i + 1;
            let slice = &blk[j as usize..len as usize];
            if build_trie(ctx, slice, (i - j) as usize, ctr) < 0 {
                return None;
            }
            ctr += 1;
            i += 1;
            j = i;
        }
    }
    if let Some(p) = last_start_p {
        *p = last_start;
    }

    // Encode name
    {
        let mut i = 0i32;
        let mut j = 0i32;
        while i < len {
            while i < len && (blk[i as usize] as i32) >= b' ' as i32 {
                i += 1;
            }
            if i >= len {
                break;
            }
            let bi = blk[i as usize];
            if bi != 0 && bi != b'\n' {
                return None;
            }
            blk[i as usize] = 0;
            let slice = &mut blk[j as usize..len as usize];
            if encode_name(ctx, slice, i - j, 1) < 0 {
                return None;
            }
            i += 1;
            j = i;
        }
    }

    // Drop N_TYPE blocks if they all contain matches bar the first item.
    {
        let mut i = 0usize;
        while i < (ctx.max_tok * 16) as usize {
            if ctx.desc[i].buf_l != 0 {
                let mut z = 1usize;
                while z < ctx.desc[i].buf_l {
                    if ctx.desc[i].buf[z] != N_MATCH as i32 as u8 {
                        break;
                    }
                    z += 1;
                }
                if z == ctx.desc[i].buf_l {
                    let mut k = 1usize;
                    while k < 16 {
                        if ctx.desc[i + k].buf_l != 0 {
                            break;
                        }
                        k += 1;
                    }
                    if k < 16 {
                        ctx.desc[i].buf_l = 0;
                        ctx.desc[i].buf.clear();
                    }
                }
            }
            i += 16;
        }
    }

    // Serialise descriptors
    let mut tot_size: u32 = 9;
    {
        let mut i = 0usize;
        while i < (ctx.max_tok * 16) as usize {
            if ctx.desc[i].buf_l == 0 {
                i += 1;
                continue;
            }
            let tnum = (i >> 4) as i32;
            let ttype = (i & 15) as i32;

            let bound = (1.5 * arith_compress_bound(ctx.desc[i].buf_l as u32, 1) as f64) as u64;
            let mut out_buf = vec![0u8; bound as usize];
            let mut out_len_l: u64 = bound;

            let in_slice = &ctx.desc[i].buf[..ctx.desc[i].buf_l];
            if compress(
                in_slice,
                ctx.desc[i].buf_l as u64,
                name_type::from_i32((i & 0xf) as i32),
                level,
                use_arith,
                &mut out_buf,
                &mut out_len_l,
            ) < 0
            {
                return None;
            }

            out_buf.truncate(out_len_l as usize);
            ctx.desc[i].buf = out_buf;
            ctx.desc[i].buf_l = out_len_l as usize;
            ctx.desc[i].tnum = tnum;
            ctx.desc[i].ttype = ttype;

            // Find dups
            let mut j = 0usize;
            while j < i {
                if ctx.desc[j].buf.is_empty() {
                    j += 1;
                    continue;
                }
                if ctx.desc[i].buf_l != ctx.desc[j].buf_l || ctx.desc[i].buf_l <= 4 {
                    j += 1;
                    continue;
                }
                if ctx.desc[i].buf[..ctx.desc[i].buf_l] == ctx.desc[j].buf[..ctx.desc[j].buf_l] {
                    break;
                }
                j += 1;
            }
            if j < i {
                ctx.desc[i].dup_from = j as i32;
                tot_size += 3;
            } else {
                ctx.desc[i].dup_from = -1;
                tot_size += out_len_l as u32 + 1;
            }
            i += 1;
        }
    }

    // Write
    let mut out = Vec::with_capacity((tot_size + 13) as usize);

    *out_len = tot_size as i32;
    macro_rules! putb {
        ($v:expr) => {{
            out.push($v);
        }};
    }
    putb!((last_start & 0xff) as u8);
    putb!(((last_start >> 8) & 0xff) as u8);
    putb!(((last_start >> 16) & 0xff) as u8);
    putb!(((last_start >> 24) & 0xff) as u8);
    putb!((nreads & 0xff) as u8);
    putb!(((nreads >> 8) & 0xff) as u8);
    putb!(((nreads >> 16) & 0xff) as u8);
    putb!(((nreads >> 24) & 0xff) as u8);
    putb!(use_arith as u8);

    let mut last_tnum = -1i32;
    {
        let mut i = 0usize;
        while i < (ctx.max_tok * 16) as usize {
            if ctx.desc[i].buf_l == 0 {
                i += 1;
                continue;
            }
            let mut ttype8 = ctx.desc[i].ttype as u8;
            if ctx.desc[i].tnum != last_tnum {
                ttype8 |= 128;
                last_tnum = ctx.desc[i].tnum;
            }
            if ctx.desc[i].dup_from >= 0 {
                putb!(ttype8 | 64);
                putb!((ctx.desc[i].dup_from >> 4) as u8);
                putb!((ctx.desc[i].dup_from & 15) as u8);
            } else {
                putb!(ttype8);
                out.extend_from_slice(&ctx.desc[i].buf[..ctx.desc[i].buf_l]);
            }
            i += 1;
        }
    }

    Some(out)
}

/// `uint8_t *encode_names(char *blk, int len, int level, int use_arith, int *out_len, int *last_start_p)`
// tokenise_name3.c:1665
pub fn encode_names(
    blk: &mut [u8],
    len: i32,
    level: i32,
    use_arith: i32,
    out_len: &mut i32,
    last_start_p: Option<&mut i32>,
) -> Option<Vec<u8>> {
    tok3_encode_names(blk, len, level, use_arith, out_len, last_start_p)
}

/// `uint8_t *tok3_decode_names(uint8_t *in, uint32_t sz, uint32_t *out_len)`
// tokenise_name3.c:1677
pub fn tok3_decode_names(r#in: &[u8], sz: u32, out_len: &mut u32) -> Option<Vec<u8>> {
    if sz < 9 {
        return None;
    }

    let mut o = 9usize;
    let mut ulen: i64 = ((r#in[0] as u32)
        | ((r#in[1] as u32) << 8)
        | ((r#in[2] as u32) << 16)
        | ((r#in[3] as u32) << 24)) as i32 as i64;

    if ulen < 0 || ulen >= (i32::MAX - 1024) as i64 {
        return None;
    }

    let nreads = ((r#in[4] as u32)
        | ((r#in[5] as u32) << 8)
        | ((r#in[6] as u32) << 16)
        | ((r#in[7] as u32) << 24)) as i32;
    let use_arith = r#in[8] as i32;
    let mut ctx = create_context(nreads)?;
    let ctx = &mut ctx;

    let sz = sz as usize;

    // Unpack descriptors.  On any error we just `return None`; the owned `ctx`
    // and its buffers are released by Drop.
    let mut tnum = -1i32;

    while o < sz {
        let ttype = r#in[o] as i32;
        o += 1;
        if ttype & 64 != 0 {
            if o + 2 > sz {
                return None;
            }
            let mut j = (r#in[o] as i32) << 4;
            o += 1;
            j += r#in[o] as i32;
            o += 1;
            if ttype & 128 != 0 {
                tnum += 1;
                if tnum >= MAX_TOKENS as i32 {
                    return None;
                }
                ctx.max_tok = tnum + 1;
                reset_desc_block(ctx, tnum);
            }
            if (ttype & 15) != 0 && (ttype & 128) != 0 {
                if tnum < 0 {
                    return None;
                }
                let base = (tnum << 4) as usize;
                seed_type_descriptor(&mut ctx.desc[base], nreads, ttype);
            }
            if tnum < 0 {
                return None;
            }
            let i = ((tnum << 4) | (ttype & 15)) as usize;
            if j as usize >= i {
                return None;
            }
            if ctx.desc[j as usize].buf.is_empty() {
                return None;
            }
            ctx.desc[i].buf_l = 0;
            ctx.desc[i].buf = ctx.desc[j as usize].buf.clone();
            continue;
        }

        if ttype & 128 != 0 {
            tnum += 1;
            if tnum >= MAX_TOKENS as i32 {
                return None;
            }
            ctx.max_tok = tnum + 1;
            reset_desc_block(ctx, tnum);
        }

        if (ttype & 15) != 0 && (ttype & 128) != 0 {
            if tnum < 0 {
                return None;
            }
            let base = (tnum << 4) as usize;
            seed_type_descriptor(&mut ctx.desc[base], nreads, ttype);
        }

        // Load compressed block
        let in_sub = &r#in[o..sz];
        let ulen_blk = uncompressed_size(in_sub, (sz - o) as u64) as i64;
        if ulen_blk < 0 || ulen_blk >= i32::MAX as i64 {
            return None;
        }
        if tnum < 0 {
            return None;
        }
        let i = ((tnum << 4) | (ttype & 15)) as usize;
        if i >= MAX_TBLOCKS {
            return None;
        }

        ctx.desc[i].buf_l = 0;
        ctx.desc[i].buf = vec![0u8; ulen_blk as usize];
        let mut usz: u64 = ctx.desc[i].buf.len() as u64;
        let out_slice = &mut ctx.desc[i].buf;
        let clen = uncompress(use_arith, in_sub, (sz - o) as u64, out_slice, &mut usz);
        ctx.desc[i].buf.truncate(usz as usize);
        if clen < 0 || ctx.desc[i].buf.len() as i64 != ulen_blk {
            return None;
        }
        o += clen as usize;
    }

    ulen += 1024;
    let mut ulen_rem = ulen;
    let mut out = vec![0u8; ulen as usize];

    let mut out_sz = 0usize;
    let mut ret;
    loop {
        let name_slice = &mut out[out_sz..];
        ret = decode_name(ctx, name_slice, ulen_rem as i32);
        if ret > 0 {
            out_sz += ret as usize;
            ulen_rem -= ret as i64;
        } else {
            break;
        }
    }

    let result = if ret == 0 {
        out.truncate(out_sz);
        Some(out)
    } else {
        None
    };

    *out_len = out_sz as u32;
    result
}

/// `uint8_t *decode_names(uint8_t *in, uint32_t sz, uint32_t *out_len)`
// tokenise_name3.c:1835
pub fn decode_names(r#in: &[u8], sz: u32, out_len: &mut u32) -> Option<Vec<u8>> {
    tok3_decode_names(r#in, sz, out_len)
}

#[cfg(test)]
mod tests;
