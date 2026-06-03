//! Native translation of `tokenise_name3.c` + `tokenise_name3.h` (htscodecs).
//!
//! Read-name tokeniser. It generates a series of byte streams (per token) and
//! compresses these either using static rANS (`rans_static4x16pr`) or dynamic
//! arithmetic coding (`arith_dynamic`). It uses the pooled allocator
//! (`pooled_alloc`) for the encoder trie nodes.
//!
//! Source: htslib/htscodecs/htscodecs/tokenise_name3.c and tokenise_name3.h
//!
//! The C code is intensely pointer-based: the context struct stores raw
//! pointers into the caller's input block (`last_name`), into malloc'd token
//! descriptor buffers, and into a pooled trie.  To stay byte-for-byte faithful
//! we mirror that with raw pointers + libc allocation via `crate::c_compat`.

#![allow(non_snake_case, non_camel_case_types, unused_assignments)]

use std::os::raw::c_char;
use std::os::raw::c_void;

use super::arith_dynamic::{arith_compress_bound, arith_compress_to, arith_uncompress_to};
use super::pooled_alloc::{pool_alloc, pool_alloc_t, pool_create, pool_destroy};
use super::rans_static4x16pr::{rans_compress_4x16, rans_uncompress_4x16};
use super::utils::{htscodecs_tls_alloc, htscodecs_tls_free};
use super::varint::{var_get_u32, var_put_u32};
use crate::c_compat;

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
#[derive(Debug)]
#[repr(C)]
pub struct trie_t {
    pub next: *mut trie_t,
    pub sibling: *mut trie_t,
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

/// ```c
/// typedef struct {
///     char *last_name;
///     int last_ntok;
///     last_context_tok *last; // [last_ntok]
/// } last_context;
/// ```
// tokenise_name3.c:137
#[derive(Debug)]
#[repr(C)]
pub struct last_context {
    pub last_name: *mut c_char,
    pub last_ntok: i32,
    pub last: *mut last_context_tok,
}

/// ```c
/// typedef struct {
///     uint8_t *buf;
///     size_t buf_a, buf_l; // alloc and used length.
///     int tnum, ttype;
///     int dup_from;
/// } descriptor;
/// ```
// tokenise_name3.c:143
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct descriptor {
    pub buf: *mut u8,
    /// allocated length
    pub buf_a: usize,
    /// used length
    pub buf_l: usize,
    pub tnum: i32,
    pub ttype: i32,
    pub dup_from: i32,
}

/// ```c
/// typedef struct {
///     last_context *lc;
///     int counter;
///     trie_t *t_head;
///     pool_alloc_t *pool;
///     descriptor desc[MAX_TBLOCKS];
///     int token_dcount[MAX_TOKENS];
///     int token_icount[MAX_TOKENS];
///     int max_tok;
///     int max_names;
/// } name_context;
/// ```
// tokenise_name3.c:150
#[repr(C)]
pub struct name_context {
    pub lc: *mut last_context,

    /// For finding entire line dups
    pub counter: i32,

    /// Trie used in encoder only
    pub t_head: *mut trie_t,
    pub pool: *mut pool_alloc_t,

    /// token blocks
    pub desc: [descriptor; MAX_TBLOCKS],

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
pub fn create_context(mut max_names: i32) -> *mut name_context {
    if max_names <= 0 {
        return std::ptr::null_mut();
    }

    if max_names as f64 > 1e7 {
        eprintln!("Name codec currently has a max of 10 million rec.");
        return std::ptr::null_mut();
    }

    max_names += 1;
    let ctx = htscodecs_tls_alloc(
        std::mem::size_of::<name_context>()
            + (max_names as usize) * std::mem::size_of::<last_context>(),
    ) as *mut name_context;
    if ctx.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        (*ctx).max_names = max_names;
        (*ctx).counter = 0;
        (*ctx).t_head = std::ptr::null_mut();

        // ctx->lc = (last_context *)(((char *)ctx) + sizeof(*ctx));
        (*ctx).lc = (ctx as *mut u8).add(std::mem::size_of::<name_context>()) as *mut last_context;
        (*ctx).pool = std::ptr::null_mut();

        // memset(&ctx->desc[0], 0, 2*16 * sizeof(ctx->desc[0]));
        std::ptr::write_bytes((*ctx).desc.as_mut_ptr(), 0, 2 * 16);
        // memset(&ctx->token_dcount[0], 0, sizeof(int));
        (*ctx).token_dcount[0] = 0;
        // memset(&ctx->token_icount[0], 0, sizeof(int));
        (*ctx).token_icount[0] = 0;
        // memset(&ctx->lc[0], 0, max_names*sizeof(ctx->lc[0]));
        std::ptr::write_bytes((*ctx).lc, 0, max_names as usize);
        (*ctx).max_tok = 1;

        (*(*ctx).lc.add(0)).last_ntok = 0;
    }

    ctx
}

/// `static void free_context(name_context *ctx)`
// tokenise_name3.c:211
fn free_context(ctx: *mut name_context) {
    if ctx.is_null() {
        return;
    }

    unsafe {
        if !(*ctx).t_head.is_null() {
            c_compat::free((*ctx).t_head as *mut c_void);
        }
        if !(*ctx).pool.is_null() {
            pool_destroy((*ctx).pool);
        }

        for i in 0..((*ctx).max_tok * 16) as usize {
            c_compat::free((*ctx).desc[i].buf as *mut c_void);
        }

        for i in 0..(*ctx).max_names as usize {
            c_compat::free((*(*ctx).lc.add(i)).last as *mut c_void);
        }

        htscodecs_tls_free(ctx as *mut c_void);
    }
}

//-----------------------------------------------------------------------------
// Fast unsigned integer printing code.
// Returns number of bytes written.

/// `static int append_uint32_fixed(char *cp, uint32_t i, uint8_t l)`
// tokenise_name3.c:233
pub fn append_uint32_fixed(cp: &mut [c_char], mut i: u32, l: u8) -> i32 {
    let mut o = 0usize;
    macro_rules! emit {
        ($div:expr, $mod:expr) => {
            cp[o] = (i / $div) as c_char + b'0' as c_char;
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
        cp[o] = i as c_char + b'0' as c_char;
    }
    l as i32
}

/// `static int append_uint32_var(char *cp, uint32_t i)`
// tokenise_name3.c:249
pub fn append_uint32_var(cp: &mut [c_char], i: u32) -> i32 {
    // The C goto-machinery is just an optimised unsigned-decimal print with no
    // leading zeros.  Crucially, for i == 0 it emits ZERO bytes (the final
    // `if (i) *cp++ = i+'0'` is false), so we reproduce that exactly.
    let z = b'0' as c_char;
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
        cp[o] = buf[n - 1 - o] as c_char + z;
    }
    n as i32
}

//-----------------------------------------------------------------------------
// Descriptor encoding and IO

/// `static int descriptor_grow(descriptor *fd, uint32_t sz)`
// tokenise_name3.c:299
pub fn descriptor_grow(fd: &mut descriptor, sz: u32) -> i32 {
    while fd.buf_l + sz as usize > fd.buf_a {
        let buf_a = if fd.buf_a != 0 { fd.buf_a * 2 } else { 65536 };
        let buf = unsafe { c_compat::realloc(fd.buf as *mut c_void, buf_a as u64) } as *mut u8;
        if buf.is_null() {
            return -1;
        }
        fd.buf = buf;
        fd.buf_a = buf_a;
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
    unsafe {
        *ctx.desc[id].buf.add(ctx.desc[id].buf_l) = r#type as i32 as u8;
    }
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
    if ctx.desc[id].buf_l >= ctx.desc[id].buf_a {
        return N_ERR;
    }
    let v = unsafe { *ctx.desc[id].buf.add(ctx.desc[id].buf_l) };
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

    unsafe {
        let cp = ctx.desc[id].buf.add(ctx.desc[id].buf_l);
        *cp.add(0) = (val & 0xff) as u8;
        *cp.add(1) = ((val >> 8) & 0xff) as u8;
        *cp.add(2) = ((val >> 16) & 0xff) as u8;
        *cp.add(3) = ((val >> 24) & 0xff) as u8;
    }
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

    if ctx.desc[id].buf_l + 4 > ctx.desc[id].buf_a {
        return -1;
    }
    unsafe {
        let cp = ctx.desc[id].buf.add(ctx.desc[id].buf_l);
        *val = (*cp.add(0) as u32)
            + ((*cp.add(1) as u32) << 8)
            + ((*cp.add(2) as u32) << 16)
            + ((*cp.add(3) as u32) << 24);
    }
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
    unsafe {
        *ctx.desc[id].buf.add(ctx.desc[id].buf_l) = val as u8;
    }
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
    unsafe {
        *ctx.desc[id].buf.add(ctx.desc[id].buf_l) = val as u8;
    }
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

    if ctx.desc[id].buf_l >= ctx.desc[id].buf_a {
        return -1;
    }
    unsafe {
        *val = *ctx.desc[id].buf.add(ctx.desc[id].buf_l) as u32;
    }
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int encode_token_alpha(name_context *ctx, int ntok, char *str, int len)`
// tokenise_name3.c:411
pub fn encode_token_alpha(ctx: &mut name_context, ntok: i32, str: &[c_char], len: i32) -> i32 {
    let id = ((ntok << 4) | N_ALPHA as i32) as usize;

    if encode_token_type(ctx, ntok, N_ALPHA) < 0 {
        return -1;
    }
    if descriptor_grow(&mut ctx.desc[id], (len + 1) as u32) < 0 {
        return -1;
    }
    unsafe {
        let dst = ctx.desc[id].buf.add(ctx.desc[id].buf_l);
        std::ptr::copy_nonoverlapping(str.as_ptr() as *const u8, dst, len as usize);
        *dst.add(len as usize) = 0;
    }
    ctx.desc[id].buf_l += (len + 1) as usize;
    0
}

/// `static int decode_token_alpha(name_context *ctx, int ntok, char *str, int max_len)`
// tokenise_name3.c:426
pub fn decode_token_alpha(
    ctx: &mut name_context,
    ntok: i32,
    str: &mut [c_char],
    max_len: i32,
) -> i32 {
    let id = ((ntok << 4) | N_ALPHA as i32) as usize;
    let mut len = 0i32;
    if ctx.desc[id].buf_l >= ctx.desc[id].buf_a {
        return -1;
    }
    let mut c: u8;
    loop {
        c = unsafe { *ctx.desc[id].buf.add(ctx.desc[id].buf_l) };
        ctx.desc[id].buf_l += 1;
        str[len as usize] = c as c_char;
        len += 1;
        if !(c != 0 && len < max_len && ctx.desc[id].buf_l < ctx.desc[id].buf_a) {
            break;
        }
    }
    len - 1
}

/// `static int encode_token_char(name_context *ctx, int ntok, char c)`
// tokenise_name3.c:440
pub fn encode_token_char(ctx: &mut name_context, ntok: i32, c: c_char) -> i32 {
    let id = ((ntok << 4) | N_CHAR as i32) as usize;

    if encode_token_type(ctx, ntok, N_CHAR) < 0 {
        return -1;
    }
    if descriptor_grow(&mut ctx.desc[id], 1) < 0 {
        return -1;
    }
    unsafe {
        *ctx.desc[id].buf.add(ctx.desc[id].buf_l) = c as u8;
    }
    ctx.desc[id].buf_l += 1;
    0
}

/// `static int decode_token_char(name_context *ctx, int ntok, char *str)`
// tokenise_name3.c:452
pub fn decode_token_char(ctx: &mut name_context, ntok: i32, str: &mut c_char) -> i32 {
    let id = ((ntok << 4) | N_CHAR as i32) as usize;

    if ctx.desc[id].buf_l >= ctx.desc[id].buf_a {
        return -1;
    }
    *str = unsafe { *ctx.desc[id].buf.add(ctx.desc[id].buf_l) } as c_char;
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

//-----------------------------------------------------------------------------
// Trie implementation for tracking common name prefixes.

/// `static int build_trie(name_context *ctx, char *data, size_t len, int n)`
// tokenise_name3.c:476
pub fn build_trie(ctx: &mut name_context, data: &[c_char], len: usize, n: i32) -> i32 {
    if ctx.t_head.is_null() {
        ctx.t_head =
            unsafe { c_compat::calloc(1, std::mem::size_of::<trie_t>() as u64) } as *mut trie_t;
        if ctx.t_head.is_null() {
            return -1;
        }
    }

    let mut i = 0usize;
    while i < len {
        let mut t = ctx.t_head;
        unsafe {
            (*t).count += 1;
        }
        while i < len && (data[i] as u8) > b'\n' {
            let c0 = data[i] as u8;
            i += 1;
            if c0 & 0x80 != 0 {
                return -1;
            }
            let c = (c0 & 127) as u32;

            unsafe {
                let mut x = (*t).next;
                let mut l: *mut trie_t = std::ptr::null_mut();
                while !x.is_null() && (*x).get_c() != c {
                    l = x;
                    x = (*x).sibling;
                }
                if x.is_null() {
                    if ctx.pool.is_null() {
                        ctx.pool = pool_create(std::mem::size_of::<trie_t>());
                    }
                    x = pool_alloc(&mut *ctx.pool) as *mut trie_t;
                    if x.is_null() {
                        return -1;
                    }
                    std::ptr::write_bytes(x, 0, 1);
                    if l.is_null() {
                        (*t).next = x;
                    } else {
                        (*l).sibling = x;
                    }
                    (*x).set_n(n as u32);
                    (*x).set_c(c);
                }
                t = x;
                (*t).set_c(c);
                (*t).count += 1;
            }
        }
        i += 1;
    }

    0
}

/// `static int search_trie(name_context *ctx, char *data, size_t len, int n, int *exact, int *is_fixed, int *fixed_len)`
// tokenise_name3.c:589
pub fn search_trie(
    ctx: &mut name_context,
    data: &[c_char],
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
    let at = data[0] as u8 == b'@';
    let d_off = if at { 1usize } else { 0 };
    let l = if at { (len as i32) - 1 } else { len as i32 };
    let f = if data[0] as u8 == b'>' { 1usize } else { 0 };

    let db = |k: usize| -> i32 { data[d_off + k] as u8 as i32 };

    if l > 70
        && db(f) == b'm' as i32
        && data[7] as u8 == b'_'
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
        while i < len && data[i] as u8 > b' ' {
            i += 1;
        }
        while i > 0 && colons < 4 {
            i -= 1;
            if data[i] as u8 == b':' {
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

    if ctx.t_head.is_null() {
        ctx.t_head =
            unsafe { c_compat::calloc(1, std::mem::size_of::<trie_t>() as u64) } as *mut trie_t;
        if ctx.t_head.is_null() {
            return -1;
        }
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
        let mut t = ctx.t_head;
        while i < len && data[i] as u8 > b'\n' {
            let c0 = data[i] as u8;
            i += 1;
            if c0 & 0x80 != 0 {
                return -1;
            }
            let c = (c0 & 127) as u32;

            unsafe {
                let mut x = (*t).next;
                while !x.is_null() && (*x).get_c() != c {
                    x = (*x).sibling;
                }
                t = x;

                from = (*t).get_n() as i32;
                if (ispunct(c as i32) || isspace(c as i32)) && from != n {
                    from_punct = (*t).get_n() as i32;
                }
                if i as i64 == prefix_len_i {
                    p3 = (*t).get_n() as i32;
                }
                (*t).set_n(n as u32);
            }
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
pub fn encode_name(ctx: &mut name_context, name: &mut [c_char], len: i32, mode: i32) -> i32 {
    let name_ptr = name.as_mut_ptr();
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

    let lc = ctx.lc;
    macro_rules! lc {
        ($i:expr) => {
            unsafe { &mut *lc.add($i as usize) }
        };
    }
    let tok_size = std::mem::size_of::<last_context_tok>();

    // Return DUP or DIFF switch, plus the distance.
    let pnum_strlen = {
        let lp = lc!(pnum);
        if lp.last_name.is_null() {
            usize::MAX
        } else {
            unsafe { libc::strlen(lp.last_name) }
        }
    };
    if exact != 0 && len as usize == pnum_strlen {
        encode_token_dup(ctx, (cnum - pnum) as u32);
        let last_ntok_p = lc!(pnum).last_ntok;
        let last_p = lc!(pnum).last;
        let cl = lc!(cnum);
        cl.last_name = name_ptr;
        cl.last_ntok = last_ntok_p;
        let nc = if cl.last_ntok != 0 {
            cl.last_ntok as usize
        } else {
            MAX_TOKENS
        };
        cl.last = unsafe { c_compat::malloc((nc * tok_size) as u64) } as *mut last_context_tok;
        if cl.last.is_null() {
            return -1;
        }
        unsafe {
            c_compat::memcpy(
                cl.last as *mut c_void,
                last_p as *const c_void,
                (cl.last_ntok as usize * tok_size) as u64,
            );
        }
        return 0;
    }

    lc!(cnum).last =
        unsafe { c_compat::malloc((MAX_TOKENS * tok_size) as u64) } as *mut last_context_tok;
    if lc!(cnum).last.is_null() {
        return -1;
    }
    encode_token_diff(ctx, (cnum - pnum) as u32);
    let mut ntok = 1i32;

    // closures to read name bytes
    let nb = |k: usize| -> i32 { unsafe { *name_ptr.add(k) as i32 } };

    // htslib v1.23: dedicated `fixed_len == 36` ONT uuid4 char-block path,
    // then the generic `is_fixed` path, else `i = 0`.
    let mut i: i32 = 0;

    if fixed_len == 36 {
        // ONT uuid4 format data
        if 37 >= ctx.max_tok {
            loop {
                let mt = ctx.max_tok;
                unsafe {
                    std::ptr::write_bytes(ctx.desc.as_mut_ptr().add((mt << 4) as usize), 0, 16);
                }
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
            encode_token_char(ctx, ntok, nb(k as usize) as c_char);
            let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
            unsafe {
                std::ptr::write_bytes(ctx.desc.as_mut_ptr().add((mt << 4) as usize), 0, 16);
            }
            ctx.token_dcount[mt as usize] = 0;
            ctx.token_icount[mt as usize] = 0;
            ctx.max_tok = ntok + 1;
        }
        let use_match = if pnum < cnum
            && ntok < lc!(pnum).last_ntok
            && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_ALPHA
        {
            let pl = unsafe { &*lc!(pnum).last.add(ntok as usize) };
            let pname = lc!(pnum).last_name;
            pl.token_int == fixed_len
                && unsafe {
                    libc::memcmp(
                        name_ptr as *const c_void,
                        pname as *const c_void,
                        fixed_len as usize,
                    ) == 0
                }
        } else {
            false
        };
        if pnum < cnum
            && ntok < lc!(pnum).last_ntok
            && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_ALPHA
        {
            if use_match {
                encode_token_match(ctx, ntok);
            } else {
                encode_token_alpha(ctx, ntok, name, fixed_len);
            }
        } else {
            encode_token_alpha(ctx, ntok, name, fixed_len);
        }
        let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
            unsafe {
                std::ptr::write_bytes(ctx.desc.as_mut_ptr().add((mt << 4) as usize), 0, 16);
            }
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
                    && ntok < lc!(pnum).last_ntok
                    && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_ALPHA;
                if matched {
                    let pl = unsafe { &*lc!(pnum).last.add(ntok as usize) };
                    let pname = lc!(pnum).last_name;
                    let is_match = s - i == pl.token_int
                        && unsafe {
                            libc::memcmp(
                                name_ptr.add(i as usize) as *const c_void,
                                pname.add(pl.token_str as usize) as *const c_void,
                                (s - i) as usize,
                            ) == 0
                        };
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

                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
                && ntok < lc!(pnum).last_ntok
                && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_DIGITS0
                && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_str } as u32 == s - i as u32
            {
                do_digits0 = true;
            } else {
                if pnum < cnum
                    && ntok < lc!(pnum).last_ntok
                    && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_DIGITS
                {
                    d = v as i32 - unsafe { (*lc!(pnum).last.add(ntok as usize)).token_int };
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

                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
                && ntok < lc!(pnum).last_ntok
                && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_DIGITS0
            {
                d = v as i32 - unsafe { (*lc!(pnum).last.add(ntok as usize)).token_int };
                let pstr = unsafe { (*lc!(pnum).last.add(ntok as usize)).token_str };
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

            let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
            lct.token_str = (s - i as u32) as i32; // length
            lct.token_int = v as i32;
            lct.token_type = N_DIGITS0;

            i = s as i32 - 1;
        }

        if do_char {
            let ci = nb(i as usize);
            if pnum < cnum
                && ntok < lc!(pnum).last_ntok
                && unsafe { (*lc!(pnum).last.add(ntok as usize)).token_type } == N_CHAR
            {
                if ci == unsafe { (*lc!(pnum).last.add(ntok as usize)).token_int } {
                    if encode_token_match(ctx, ntok) < 0 {
                        return -1;
                    }
                } else if encode_token_char(ctx, ntok, ci as c_char) < 0 {
                    return -1;
                }
            } else if encode_token_char(ctx, ntok, ci as c_char) < 0 {
                return -1;
            }

            let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
        unsafe {
            std::ptr::write_bytes(ctx.desc.as_mut_ptr().add((mt << 4) as usize), 0, 16);
        }
        ctx.token_dcount[mt as usize] = 0;
        ctx.token_icount[mt as usize] = 0;
        ctx.max_tok = ntok + 1;
    }
    if encode_token_end(ctx, ntok) < 0 {
        return -1;
    }

    let cl = lc!(cnum);
    cl.last_name = name_ptr;
    cl.last_ntok = ntok;
    let shrunk = unsafe {
        c_compat::realloc(
            cl.last as *mut c_void,
            ((ntok + 1) as usize * tok_size) as u64,
        )
    } as *mut last_context_tok;
    if !shrunk.is_null() {
        cl.last = shrunk;
    }
    if cl.last.is_null() {
        return -1;
    }

    0
}

//-----------------------------------------------------------------------------
// Name decoder

/// `static int decode_name(name_context *ctx, char *name, int name_len)`
// tokenise_name3.c:1021
#[allow(unused_unsafe)]
pub fn decode_name(ctx: &mut name_context, name: &mut [c_char], name_len: i32) -> i32 {
    let name_ptr = name.as_mut_ptr();
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

    let lc = ctx.lc;
    macro_rules! lc {
        ($i:expr) => {
            unsafe { &mut *lc.add($i as usize) }
        };
    }
    let tok_size = std::mem::size_of::<last_context_tok>();

    if t0 == N_DUP {
        if pnum == cnum {
            return -1;
        }
        let pname = lc!(pnum).last_name;
        let plen = unsafe { libc::strlen(pname) };
        if plen + 1 >= name_len as usize {
            return -1;
        }
        unsafe {
            libc::strcpy(name_ptr, pname);
        }
        let last_ntok = lc!(pnum).last_ntok;
        let last_p = lc!(pnum).last;
        let cl = lc!(cnum);
        cl.last_name = name_ptr;
        cl.last_ntok = last_ntok;
        let nc = if cl.last_ntok != 0 {
            cl.last_ntok as usize
        } else {
            MAX_TOKENS
        };
        cl.last = unsafe { c_compat::malloc((nc * tok_size) as u64) } as *mut last_context_tok;
        if cl.last.is_null() {
            return -1;
        }
        unsafe {
            c_compat::memcpy(
                cl.last as *mut c_void,
                last_p as *const c_void,
                (cl.last_ntok as usize * tok_size) as u64,
            );
        }
        return (unsafe { libc::strlen(name_ptr) } + 1) as i32;
    }

    unsafe {
        *name_ptr = 0;
    }
    let mut len = 0i32;
    lc!(cnum).last =
        unsafe { c_compat::malloc((MAX_TOKENS * tok_size) as u64) } as *mut last_context_tok;
    if lc!(cnum).last.is_null() {
        return -1;
    }

    let nslice = |off: i32| -> &mut [c_char] {
        unsafe {
            std::slice::from_raw_parts_mut(name_ptr.add(off as usize), (name_len - off) as usize)
        }
    };

    let mut ntok = 1i32;
    while ntok < MAX_TOKENS as i32 && ntok < ctx.max_tok {
        let tok = decode_token_type(ctx, ntok);
        lc!(cnum).last_ntok = 0;

        match tok {
            N_CHAR => {
                if len + 1 >= name_len {
                    return -1;
                }
                let mut c: c_char = 0;
                if decode_token_char(ctx, ntok, &mut c) < 0 {
                    return -1;
                }
                unsafe {
                    *name_ptr.add(len as usize) = c;
                }
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                lct.token_type = N_CHAR;
                lct.token_int = c as i32;
                len += 1;
            }
            N_ALPHA => {
                let len2 = decode_token_alpha(ctx, ntok, nslice(len), name_len - len);
                if len2 < 0 {
                    return -1;
                }
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
                len += append_uint32_fixed(nslice(len), v, vl as u8);
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                lct.token_type = N_DIGITS0;
                lct.token_int = v as i32;
                lct.token_str = vl as i32;
            }
            N_DDELTA0 => {
                if ntok >= lc!(pnum).last_ntok {
                    return -1;
                }
                let mut v: u32 = 0;
                if decode_token_int1(ctx, ntok, N_DDELTA0, &mut v) < 0 {
                    return -1;
                }
                let pl = unsafe { *lc!(pnum).last.add(ntok as usize) };
                v = v.wrapping_add(pl.token_int as u32);
                if len + pl.token_str + 1 >= name_len {
                    return -1;
                }
                len += append_uint32_fixed(nslice(len), v, pl.token_str as u8);
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
                len += append_uint32_var(nslice(len), v);
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                lct.token_type = N_DIGITS;
                lct.token_int = v as i32;
            }
            N_DDELTA => {
                if ntok >= lc!(pnum).last_ntok {
                    return -1;
                }
                let mut v: u32 = 0;
                if decode_token_int1(ctx, ntok, N_DDELTA, &mut v) < 0 {
                    return -1;
                }
                let pl = unsafe { *lc!(pnum).last.add(ntok as usize) };
                v = v.wrapping_add(pl.token_int as u32);
                if len + 20 >= name_len {
                    return -1;
                }
                len += append_uint32_var(nslice(len), v);
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                lct.token_type = N_DIGITS;
                lct.token_int = v as i32;
            }
            N_NOP => {
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                lct.token_type = N_NOP;
            }
            N_MATCH => {
                if ntok >= lc!(pnum).last_ntok {
                    return -1;
                }
                let pl = unsafe { *lc!(pnum).last.add(ntok as usize) };
                match pl.token_type {
                    N_CHAR => {
                        if len + 1 >= name_len {
                            return -1;
                        }
                        unsafe {
                            *name_ptr.add(len as usize) = pl.token_int as c_char;
                        }
                        len += 1;
                        let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                        lct.token_type = N_CHAR;
                        lct.token_int = pl.token_int;
                    }
                    N_ALPHA => {
                        if pl.token_int < 0 || len + pl.token_int >= name_len {
                            return -1;
                        }
                        let pname = lc!(pnum).last_name;
                        unsafe {
                            c_compat::memcpy(
                                name_ptr.add(len as usize) as *mut c_void,
                                pname.add(pl.token_str as usize) as *const c_void,
                                pl.token_int as u64,
                            );
                        }
                        let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                        lct.token_type = N_ALPHA;
                        lct.token_str = len;
                        lct.token_int = pl.token_int;
                        len += pl.token_int;
                    }
                    N_DIGITS => {
                        if len + 20 >= name_len {
                            return -1;
                        }
                        len += append_uint32_var(nslice(len), pl.token_int as u32);
                        let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                        lct.token_type = N_DIGITS;
                        lct.token_int = pl.token_int;
                    }
                    N_DIGITS0 => {
                        if len + pl.token_str >= name_len {
                            return -1;
                        }
                        len += append_uint32_fixed(
                            nslice(len),
                            pl.token_int as u32,
                            pl.token_str as u8,
                        );
                        let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
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
                unsafe {
                    *name_ptr.add(len as usize) = 0;
                }
                len += 1;
                let lct = unsafe { &mut *lc!(cnum).last.add(ntok as usize) };
                lct.token_type = N_END;

                let cl = lc!(cnum);
                cl.last_name = name_ptr;
                cl.last_ntok = ntok;
                let shrunk = unsafe {
                    c_compat::realloc(
                        cl.last as *mut c_void,
                        ((ntok + 1) as usize * tok_size) as u64,
                    )
                } as *mut last_context_tok;
                if !shrunk.is_null() {
                    cl.last = shrunk;
                }
                if cl.last.is_null() {
                    return -1;
                }
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
    unsafe {
        c_compat::memmove(
            out.as_mut_ptr().add(nb) as *mut c_void,
            out.as_ptr().add(6) as *const c_void,
            olen as u64,
        );
    }
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
    unsafe {
        c_compat::memmove(
            out.as_mut_ptr().add(nb) as *mut c_void,
            out.as_ptr().add(6) as *const c_void,
            olen as u64,
        );
    }
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
    blk: &mut [c_char],
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

    let blk_ptr = blk.as_mut_ptr();

    // Count lines
    let mut nreads = 0i32;
    {
        let mut i = 0i32;
        while i < len {
            if (unsafe { *blk_ptr.add(i as usize) } as u8) <= b'\n' {
                nreads += 1;
            }
            i += 1;
        }
    }

    let ctx = create_context(nreads);
    if ctx.is_null() {
        return None;
    }
    let ctx = unsafe { &mut *ctx };

    // Construct trie
    let mut ctr = 0i32;
    {
        let mut i = 0i32;
        let mut j = 0i32;
        while i < len {
            while i < len && (unsafe { *blk_ptr.add(i as usize) } as u8) > b'\n' {
                i += 1;
            }
            if i >= len {
                break;
            }
            last_start = i + 1;
            let slice =
                unsafe { std::slice::from_raw_parts(blk_ptr.add(j as usize), (len - j) as usize) };
            if build_trie(ctx, slice, (i - j) as usize, ctr) < 0 {
                free_context(ctx);
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
            while i < len && (unsafe { *blk_ptr.add(i as usize) } as i32) >= b' ' as i32 {
                i += 1;
            }
            if i >= len {
                break;
            }
            let bi = unsafe { *blk_ptr.add(i as usize) } as u8;
            if bi != 0 && bi != b'\n' {
                free_context(ctx);
                return None;
            }
            unsafe {
                *blk_ptr.add(i as usize) = 0;
            }
            let slice = unsafe {
                std::slice::from_raw_parts_mut(blk_ptr.add(j as usize), (len - j) as usize)
            };
            if encode_name(ctx, slice, i - j, 1) < 0 {
                free_context(ctx);
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
                    if unsafe { *ctx.desc[i].buf.add(z) } != N_MATCH as i32 as u8 {
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
                        unsafe {
                            c_compat::free(ctx.desc[i].buf as *mut c_void);
                        }
                        ctx.desc[i].buf = std::ptr::null_mut();
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
            let out_buf = unsafe { c_compat::malloc(bound) } as *mut u8;
            if out_buf.is_null() {
                free_context(ctx);
                return None;
            }
            let mut out_len_l: u64 = bound;

            let in_slice =
                unsafe { std::slice::from_raw_parts(ctx.desc[i].buf, ctx.desc[i].buf_l) };
            let out_slice = unsafe { std::slice::from_raw_parts_mut(out_buf, bound as usize) };
            if compress(
                in_slice,
                ctx.desc[i].buf_l as u64,
                name_type::from_i32((i & 0xf) as i32),
                level,
                use_arith,
                out_slice,
                &mut out_len_l,
            ) < 0
            {
                free_context(ctx);
                unsafe { c_compat::free(out_buf as *mut c_void) };
                return None;
            }

            unsafe {
                c_compat::free(ctx.desc[i].buf as *mut c_void);
            }
            ctx.desc[i].buf = out_buf;
            ctx.desc[i].buf_l = out_len_l as usize;
            ctx.desc[i].tnum = tnum;
            ctx.desc[i].ttype = ttype;

            // Find dups
            let mut j = 0usize;
            while j < i {
                if ctx.desc[j].buf.is_null() {
                    j += 1;
                    continue;
                }
                if ctx.desc[i].buf_l != ctx.desc[j].buf_l || ctx.desc[i].buf_l <= 4 {
                    j += 1;
                    continue;
                }
                if unsafe {
                    libc::memcmp(
                        ctx.desc[i].buf as *const c_void,
                        ctx.desc[j].buf as *const c_void,
                        ctx.desc[i].buf_l,
                    )
                } == 0
                {
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
    let out = unsafe { c_compat::malloc((tot_size + 13) as u64) } as *mut u8;
    if out.is_null() {
        free_context(ctx);
        return None;
    }

    let mut cp = 0usize;
    *out_len = tot_size as i32;
    macro_rules! putb {
        ($v:expr) => {{
            unsafe {
                *out.add(cp) = $v;
            }
            cp += 1;
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
                unsafe {
                    c_compat::memcpy(
                        out.add(cp) as *mut c_void,
                        ctx.desc[i].buf as *const c_void,
                        ctx.desc[i].buf_l as u64,
                    );
                }
                cp += ctx.desc[i].buf_l;
            }
            i += 1;
        }
    }

    let result = unsafe { std::slice::from_raw_parts(out, tot_size as usize).to_vec() };
    unsafe {
        c_compat::free(out as *mut c_void);
    }
    free_context(ctx);

    Some(result)
}

/// `uint8_t *encode_names(char *blk, int len, int level, int use_arith, int *out_len, int *last_start_p)`
// tokenise_name3.c:1665
pub fn encode_names(
    blk: &mut [c_char],
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

    let in_ptr = r#in.as_ptr();
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
    let ctx = create_context(nreads);
    if ctx.is_null() {
        return None;
    }
    let ctx = unsafe { &mut *ctx };

    let sz = sz as usize;

    // Unpack descriptors
    let mut tnum = -1i32;
    let err = |ctx: &mut name_context| -> Option<Vec<u8>> {
        free_context(ctx);
        None
    };

    while o < sz {
        let ttype = r#in[o] as i32;
        o += 1;
        if ttype & 64 != 0 {
            if o + 2 > sz {
                return err(ctx);
            }
            let mut j = (r#in[o] as i32) << 4;
            o += 1;
            j += r#in[o] as i32;
            o += 1;
            if ttype & 128 != 0 {
                tnum += 1;
                if tnum >= MAX_TOKENS as i32 {
                    return err(ctx);
                }
                ctx.max_tok = tnum + 1;
                unsafe {
                    std::ptr::write_bytes(ctx.desc.as_mut_ptr().add((tnum << 4) as usize), 0, 16);
                }
            }
            if (ttype & 15) != 0 && (ttype & 128) != 0 {
                if tnum < 0 {
                    return err(ctx);
                }
                let base = (tnum << 4) as usize;
                ctx.desc[base].buf = unsafe { c_compat::malloc(nreads as u64) } as *mut u8;
                if ctx.desc[base].buf.is_null() {
                    return err(ctx);
                }
                ctx.desc[base].buf_l = 0;
                ctx.desc[base].buf_a = nreads as usize;
                unsafe {
                    *ctx.desc[base].buf.add(0) = (ttype & 15) as u8;
                    std::ptr::write_bytes(
                        ctx.desc[base].buf.add(1),
                        N_MATCH as i32 as u8,
                        (nreads - 1) as usize,
                    );
                }
            }
            if tnum < 0 {
                return err(ctx);
            }
            let i = ((tnum << 4) | (ttype & 15)) as usize;
            if j as usize >= i {
                return err(ctx);
            }
            if ctx.desc[j as usize].buf.is_null() {
                return err(ctx);
            }
            ctx.desc[i].buf_l = 0;
            ctx.desc[i].buf_a = ctx.desc[j as usize].buf_a;
            if !ctx.desc[i].buf.is_null() {
                unsafe { c_compat::free(ctx.desc[i].buf as *mut c_void) };
            }
            ctx.desc[i].buf = unsafe { c_compat::malloc(ctx.desc[i].buf_a as u64) } as *mut u8;
            if ctx.desc[i].buf.is_null() {
                return err(ctx);
            }
            unsafe {
                c_compat::memcpy(
                    ctx.desc[i].buf as *mut c_void,
                    ctx.desc[j as usize].buf as *const c_void,
                    ctx.desc[i].buf_a as u64,
                );
            }
            continue;
        }

        if ttype & 128 != 0 {
            tnum += 1;
            if tnum >= MAX_TOKENS as i32 {
                return err(ctx);
            }
            ctx.max_tok = tnum + 1;
            unsafe {
                std::ptr::write_bytes(ctx.desc.as_mut_ptr().add((tnum << 4) as usize), 0, 16);
            }
        }

        if (ttype & 15) != 0 && (ttype & 128) != 0 {
            if tnum < 0 {
                return err(ctx);
            }
            let base = (tnum << 4) as usize;
            if !ctx.desc[base].buf.is_null() {
                unsafe { c_compat::free(ctx.desc[base].buf as *mut c_void) };
            }
            ctx.desc[base].buf = unsafe { c_compat::malloc(nreads as u64) } as *mut u8;
            if ctx.desc[base].buf.is_null() {
                return err(ctx);
            }
            ctx.desc[base].buf_l = 0;
            ctx.desc[base].buf_a = nreads as usize;
            unsafe {
                *ctx.desc[base].buf.add(0) = (ttype & 15) as u8;
                std::ptr::write_bytes(
                    ctx.desc[base].buf.add(1),
                    N_MATCH as i32 as u8,
                    (nreads - 1) as usize,
                );
            }
        }

        // Load compressed block
        let in_sub = unsafe { std::slice::from_raw_parts(in_ptr.add(o), sz - o) };
        let ulen_blk = uncompressed_size(in_sub, (sz - o) as u64) as i64;
        if ulen_blk < 0 || ulen_blk >= i32::MAX as i64 {
            return err(ctx);
        }
        if tnum < 0 {
            return err(ctx);
        }
        let i = ((tnum << 4) | (ttype & 15)) as usize;
        if i >= MAX_TBLOCKS {
            return err(ctx);
        }

        ctx.desc[i].buf_l = 0;
        if !ctx.desc[i].buf.is_null() {
            unsafe { c_compat::free(ctx.desc[i].buf as *mut c_void) };
        }
        ctx.desc[i].buf = unsafe { c_compat::malloc(ulen_blk as u64) } as *mut u8;
        if ctx.desc[i].buf.is_null() {
            return err(ctx);
        }
        ctx.desc[i].buf_a = ulen_blk as usize;
        let mut usz: u64 = ctx.desc[i].buf_a as u64;
        let out_slice =
            unsafe { std::slice::from_raw_parts_mut(ctx.desc[i].buf, ctx.desc[i].buf_a) };
        let clen = uncompress(use_arith, in_sub, (sz - o) as u64, out_slice, &mut usz);
        ctx.desc[i].buf_a = usz as usize;
        if clen < 0 || ctx.desc[i].buf_a as i64 != ulen_blk {
            return err(ctx);
        }
        o += clen as usize;
    }

    ulen += 1024;
    let mut ulen_rem = ulen;
    let out = unsafe { c_compat::malloc(ulen as u64) } as *mut u8;
    if out.is_null() {
        return err(ctx);
    }

    let mut out_sz = 0usize;
    let mut ret;
    loop {
        let name_slice = unsafe {
            std::slice::from_raw_parts_mut(out.add(out_sz) as *mut c_char, ulen_rem as usize)
        };
        ret = decode_name(ctx, name_slice, ulen_rem as i32);
        if ret > 0 {
            out_sz += ret as usize;
            ulen_rem -= ret as i64;
        } else {
            break;
        }
    }

    if ret < 0 {
        unsafe { c_compat::free(out as *mut c_void) };
    }

    let result = if ret == 0 {
        Some(unsafe { std::slice::from_raw_parts(out, out_sz).to_vec() })
    } else {
        None
    };
    if ret == 0 {
        unsafe { c_compat::free(out as *mut c_void) };
    }

    free_context(ctx);
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
