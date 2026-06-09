/*  http_parser.c -- ref-cache HTTP protocol handler

    Copyright (C) 2025 Genome Research Ltd.

    Author: Rob Davies <rmd@sanger.ac.uk>
*/

use super::options::Options;
use super::request_handler::{
    ref_cache_request_handler_c_148_handle_request, ref_cache_request_handler_c_167_handle_error,
};
use super::server::ref_cache_server_c_352_client_add_transaction;
use super::server::Client;
use super::transaction::Transaction;
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong};

const REF_CACHE_MAX_UA_LEN: usize = 128;
const REF_CACHE_MAX_REFERRER_LEN: usize = 128;
const REF_CACHE_READING_REQUEST_LINE: c_int = 0;
const REF_CACHE_READING_HEADERS: c_int = 1;
const REF_CACHE_READING_CHUNK_HEADER: c_int = 2;
const REF_CACHE_READING_CHUNK: c_int = 3;
const REF_CACHE_READING_CHUNK_TRAILER: c_int = 4;
const REF_CACHE_READING_BODY: c_int = 5;
const REF_CACHE_GOT_REQUEST: c_int = 6;
const REF_CACHE_SHUTTING_DOWN: c_int = 7;
const REF_CACHE_ERR_BAD_REQUEST: c_int = 400;
const REF_CACHE_ERR_TOO_LARGE: c_int = 413;
const REF_CACHE_ERR_LONG_URI: c_int = 414;
const REF_CACHE_ERR_INTERNAL: c_int = 500;
const REF_CACHE_ERR_UNIMPLEMENTED: c_int = 501;
const REF_CACHE_ERR_HTTP_VERS: c_int = 505;
const REF_CACHE_REQ_OPTIONS: c_int = 0;
const REF_CACHE_REQ_GET: c_int = 1;
const REF_CACHE_REQ_HEAD: c_int = 2;
const REF_CACHE_REQ_POST: c_int = 3;
const REF_CACHE_REQ_PUT: c_int = 4;
const REF_CACHE_REQ_DELETE: c_int = 5;
const REF_CACHE_REQ_TRACE: c_int = 6;
const REF_CACHE_REQ_CONNECT: c_int = 7;
const REF_CACHE_REQ_OTHER: c_int = 8;
const REF_CACHE_HTTP_0_9: c_int = 0;
const HTTP_1_0: c_int = 1;
const HTTP_1_1: c_int = 2;
const REF_CACHE_TE_IDENT: c_int = 0;
const REF_CACHE_TE_CHUNKED: c_int = 1;
const REF_CACHE_READ_BLOCKED: c_int = 0;
const REF_CACHE_READ_MORE: c_int = 1;
const REF_CACHE_READ_EOF: c_int = 2;
const REF_CACHE_READ_ERROR: c_int = 3;
const TRANSACT_KEEP_ALIVE: c_uint = 2;
const TRANSACT_RANGE_FROM: c_uint = 8;
const TRANSACT_RANGE_TO: c_uint = 16;
const TRANSACT_RANGE_SUFFIX: c_uint = 32;
const REF_CACHE_BUF_SZ: c_uint = 0x400;
const REF_CACHE_BUF_MASK: c_uint = REF_CACHE_BUF_SZ - 1;

static REF_CACHE_LWS_CHARS: [c_uchar; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static REF_CACHE_TOKEN_CHARS: [c_uchar; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// Concurrency note (audit 2026-05):
//
// `REF_CACHE_HTTP_LINE` is a scratch buffer used to assemble a single line
// of an HTTP request while the parser is reassembling a wrapped header
// across ring-buffer boundaries (see
// `ref_cache_http_parser_c_158_parser_get_line`). The buffer is owned by
// the `ref-cache` daemon's single epoll worker thread (fork-based process
// model — see `src/ref_cache/main.rs` and `src/ref_cache/server.rs`); no
// threading primitives exist anywhere in the `ref_cache` subsystem, so the
// buffer is touched by exactly one thread at a time.
//
// SAFETY: single-threaded daemon worker; the function reads the parser
// state, decides if a contiguous line is available, and either returns an
// inline pointer into `(*parser).buffer` or copies into this static. The
// returned pointer is consumed before the next call, so there is also no
// re-entrancy hazard within the single thread.
static mut REF_CACHE_HTTP_LINE: [c_char; REF_CACHE_BUF_SZ as usize + 1] =
    [0; REF_CACHE_BUF_SZ as usize + 1];

#[repr(C)]
struct HttpParserLayout {
    state: c_int,
    req_type: c_int,
    http_vers: c_int,
    trans_enc: c_int,
    content_length: c_ulong,
    bytes: c_ulong,
    uri: *mut c_char,
    key: *mut c_char,
    val: *mut c_char,
    buffer: *mut c_char,
    user_agent: *mut c_char,
    referrer: *mut c_char,
    range_from: libc::off_t,
    range_to: libc::off_t,
    key_sz: usize,
    key_used: usize,
    val_sz: usize,
    val_used: usize,
    upstream: c_int,
    flags: c_uint,
    in_: c_uint,
    out: c_uint,
    pos: c_uint,
    used: c_uint,
    uri_buf: Vec<u8>,
    key_buf: Vec<u8>,
    val_buf: Vec<u8>,
    buffer_buf: Vec<u8>,
    user_agent_buf: Vec<u8>,
    referrer_buf: Vec<u8>,
}

#[repr(C)]
struct RefCacheMatchAddrLayout {
    family: libc::sa_family_t,
    mask_bytes: u8,
    mask: u8,
    addr: [libc::c_uchar; 16],
}

#[repr(C)]
struct RefCacheOptionsLayout {
    cache_dir: *const c_char,
    log_dir: *const c_char,
    error_log_file: *const c_char,
    log: *mut libc::FILE,
    upstream_url: *const c_char,
    upstream_url_len: usize,
    match_addrs: *mut RefCacheMatchAddrLayout,
    num_match_addrs: usize,
    match_addrs_size: usize,
    first_ip6: usize,
    max_log_sz: libc::off_t,
    cache_fd: c_int,
    listen_fds: c_int,
    daemon: c_int,
    port: u16,
    nlogs: u16,
    max_kids: u16,
    verbosity: u8,
    no_log: u8,
}

fn ref_cache_http_parser_is_lws(ch: c_uchar) -> bool {
    REF_CACHE_LWS_CHARS[ch as usize] != 0
}

fn ref_cache_http_parser_is_token(ch: c_uchar) -> bool {
    REF_CACHE_TOKEN_CHARS[ch as usize] != 0
}

impl HttpParserLayout {
    fn refresh_raw_ptrs(&mut self) {
        self.uri = nul_terminated_ptr(&mut self.uri_buf);
        self.key = nul_terminated_ptr(&mut self.key_buf);
        self.val = nul_terminated_ptr(&mut self.val_buf);
        self.buffer = self.buffer_buf.as_mut_ptr().cast();
        self.user_agent = nul_terminated_ptr(&mut self.user_agent_buf);
        self.referrer = nul_terminated_ptr(&mut self.referrer_buf);
    }
}

fn nul_terminated_ptr(buf: &mut [u8]) -> *mut c_char {
    if buf.is_empty() {
        std::ptr::null_mut()
    } else {
        buf.as_mut_ptr().cast()
    }
}

fn lws_spn_bytes(s: &[u8]) -> usize {
    s.iter()
        .take_while(|&&ch| ref_cache_http_parser_is_lws(ch))
        .count()
}

fn lws_cspn_bytes(s: &[u8]) -> usize {
    s.iter()
        .take_while(|&&ch| ch != 0 && !ref_cache_http_parser_is_lws(ch))
        .count()
}

fn tok_spn_bytes(s: &[u8]) -> usize {
    s.iter()
        .take_while(|&&ch| ref_cache_http_parser_is_token(ch))
        .count()
}

fn parse_decimal_c_ulong(bytes: &[u8]) -> Option<c_ulong> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut out: c_ulong = 0;
    for &digit in bytes {
        out = out
            .checked_mul(10)?
            .checked_add((digit - b'0') as c_ulong)?;
    }
    Some(out)
}

fn parse_decimal_off_t(bytes: &[u8]) -> Option<(libc::off_t, usize)> {
    let digits = bytes.iter().take_while(|&&ch| ch.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }

    let mut out: i128 = 0;
    for &digit in &bytes[..digits] {
        out = out.checked_mul(10)?.checked_add((digit - b'0') as i128)?;
    }

    let off_max = if std::mem::size_of::<libc::off_t>() < 8 {
        i32::MAX as i128
    } else {
        i64::MAX as i128
    };
    if out > off_max {
        return None;
    }

    Some((out as libc::off_t, digits))
}

fn limited_header_value(src: &[u8], max_len: usize) -> Vec<u8> {
    if src.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if src.len() < max_len {
        out.extend_from_slice(src);
    } else {
        out.extend_from_slice(&src[..max_len - 3]);
        out.extend_from_slice(b"...");
    }
    out.push(0);
    out
}

fn parse_range_bytes(parser: &mut HttpParserLayout) {
    let mut v = parser.val_buf.as_slice();
    let off_max: libc::off_t = if std::mem::size_of::<libc::off_t>() < 8 {
        i32::MAX as libc::off_t
    } else {
        i64::MAX as libc::off_t
    };

    if v.last() == Some(&0) {
        v = &v[..v.len() - 1];
    }

    if v.len() < 5 || !v[..5].eq_ignore_ascii_case(b"bytes") {
        parser.flags &= !(TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
        return;
    }
    v = &v[5 + lws_spn_bytes(&v[5..])..];
    if v.first() != Some(&b'=') {
        parser.flags &= !(TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
        return;
    }
    v = &v[1 + lws_spn_bytes(&v[1..])..];

    parser.range_from = off_max;
    parser.range_to = 0;

    loop {
        if v.first() == Some(&b'-') {
            let Some((ll, used)) = parse_decimal_off_t(&v[1..]) else {
                break;
            };
            parser.flags |= TRANSACT_RANGE_SUFFIX;
            if parser.range_to < ll {
                parser.range_to = ll;
            }
            v = &v[1 + used..];
            v = &v[lws_spn_bytes(v)..];
        } else {
            let Some((ll, used)) = parse_decimal_off_t(v) else {
                break;
            };
            parser.flags |= TRANSACT_RANGE_FROM;
            if parser.range_from > ll {
                parser.range_from = ll;
            }
            v = &v[used..];
            v = &v[lws_spn_bytes(v)..];
            if v.first() != Some(&b'-') {
                break;
            }
            v = &v[1..];
            v = &v[lws_spn_bytes(v)..];
            if v.is_empty() {
                return;
            }
            let Some((ll, used)) = parse_decimal_off_t(v) else {
                break;
            };
            parser.flags |= TRANSACT_RANGE_TO;
            if parser.range_to < ll {
                parser.range_to = ll;
            }
            v = &v[used..];
            v = &v[lws_spn_bytes(v)..];
        }
        if v.is_empty() {
            if (parser.flags & (TRANSACT_RANGE_FROM | TRANSACT_RANGE_SUFFIX))
                == (TRANSACT_RANGE_FROM | TRANSACT_RANGE_SUFFIX)
            {
                parser.flags &= !(TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
            }
            return;
        }
        if v.first() != Some(&b',') {
            break;
        }
        v = &v[1..];
        v = &v[lws_spn_bytes(v)..];
    }
    parser.flags &= !(TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
}

// original: lws_spn (htslib/ref_cache/http_parser.c:93)
pub unsafe fn ref_cache_http_parser_c_93_lws_spn(s: *mut c_char) -> usize {
    let mut c = 0;
    while ref_cache_http_parser_is_lws(*s.add(c) as c_uchar) {
        c += 1;
    }
    c
}

// original: lws_cspn (htslib/ref_cache/http_parser.c:99)
pub unsafe fn ref_cache_http_parser_c_99_lws_cspn(s: *mut c_char) -> usize {
    let mut c = 0;
    while *s.add(c) != 0 && !ref_cache_http_parser_is_lws(*s.add(c) as c_uchar) {
        c += 1;
    }
    c
}

// original: tok_spn (htslib/ref_cache/http_parser.c:105)
pub unsafe fn ref_cache_http_parser_c_105_tok_spn(s: *mut c_char) -> usize {
    let mut c = 0;
    while ref_cache_http_parser_is_token(*s.add(c) as c_uchar) {
        c += 1;
    }
    c
}

// original: init_http_parser (htslib/ref_cache/http_parser.c:111)
pub unsafe fn ref_cache_http_parser_c_111_init_http_parser(
    parser: *mut Http_Parser,
    upstream: c_int,
) -> c_int {
    let parser = parser.cast::<HttpParserLayout>();
    let mut layout = HttpParserLayout {
        state: 0,
        req_type: 0,
        http_vers: 0,
        trans_enc: 0,
        content_length: 0,
        bytes: 0,
        uri: std::ptr::null_mut(),
        key: std::ptr::null_mut(),
        val: std::ptr::null_mut(),
        buffer: std::ptr::null_mut(),
        user_agent: std::ptr::null_mut(),
        referrer: std::ptr::null_mut(),
        range_from: 0,
        range_to: 0,
        key_sz: 0,
        key_used: 0,
        val_sz: 0,
        val_used: 0,
        upstream,
        flags: 0,
        in_: 0,
        out: 0,
        pos: 0,
        used: 0,
        uri_buf: Vec::new(),
        key_buf: Vec::new(),
        val_buf: Vec::new(),
        buffer_buf: vec![0; REF_CACHE_BUF_SZ as usize],
        user_agent_buf: Vec::new(),
        referrer_buf: Vec::new(),
    };
    layout.refresh_raw_ptrs();
    parser.write(layout);
    0
}

// original: cleanup_http_parser (htslib/ref_cache/http_parser.c:131)
pub unsafe fn ref_cache_http_parser_c_131_cleanup_http_parser(parser: *mut Http_Parser) {
    let parser = parser.cast::<HttpParserLayout>();
    if (*parser).buffer.is_null() {
        return;
    }
    std::ptr::drop_in_place(parser);
    parser
        .cast::<u8>()
        .write_bytes(0, std::mem::size_of::<HttpParserLayout>());
}

// original: parser_get_line (htslib/ref_cache/http_parser.c:158)
pub unsafe fn ref_cache_http_parser_c_158_parser_get_line(
    parser: *mut Http_Parser,
    len: *mut usize,
) -> *mut c_char {
    let parser = parser.cast::<HttpParserLayout>();
    let line = std::ptr::addr_of_mut!(REF_CACHE_HTTP_LINE).cast::<c_char>();
    let mut lpos: usize;

    if (*parser).used == 0 {
        assert!((*parser).in_ == (*parser).out);
        assert!((*parser).pos == (*parser).out);
        return std::ptr::null_mut();
    }

    loop {
        if (&(*parser).buffer_buf)[(*parser).pos as usize] == b'\n' {
            break;
        }
        (*parser).pos = ((*parser).pos + 1) & REF_CACHE_BUF_MASK;
        if (*parser).pos == (*parser).in_ {
            break;
        }
    }
    if (*parser).pos == (*parser).in_ && (&(*parser).buffer_buf)[(*parser).pos as usize] != b'\n' {
        if (*parser).used == REF_CACHE_BUF_SZ {
            (*parser).state = REF_CACHE_ERR_TOO_LARGE;
            return std::ptr::null_mut();
        }
        return std::ptr::null_mut();
    }

    lpos = 0;
    if (*parser).pos <= (*parser).out {
        let n = (REF_CACHE_BUF_SZ - (*parser).out) as usize;
        let out = (*parser).out as usize;
        std::slice::from_raw_parts_mut(line.cast::<u8>().add(lpos), n)
            .copy_from_slice(&(&(*parser).buffer_buf)[out..out + n]);
        lpos += n;
        (*parser).out = 0;
    }
    let n = ((*parser).pos - (*parser).out) as usize;
    let out = (*parser).out as usize;
    std::slice::from_raw_parts_mut(line.cast::<u8>().add(lpos), n)
        .copy_from_slice(&(&(*parser).buffer_buf)[out..out + n]);
    lpos += n;
    *line.add(lpos) = 0;
    if lpos > 0 && *line.add(lpos - 1) == b'\r' as c_char {
        lpos -= 1;
        *line.add(lpos) = 0;
    }
    *len = lpos;
    (*parser).pos = ((*parser).pos + 1) & REF_CACHE_BUF_MASK;
    (*parser).out = (*parser).pos;
    (*parser).used = if (*parser).out > (*parser).in_ {
        REF_CACHE_BUF_SZ - (*parser).out + (*parser).in_
    } else {
        (*parser).in_ - (*parser).out
    };
    line
}

// original: parser_read_input (htslib/ref_cache/http_parser.c:201)
pub unsafe fn ref_cache_http_parser_c_201_parser_read_input(
    parser: *mut Http_Parser,
    fd: c_int,
) -> c_int {
    let parser = parser.cast::<HttpParserLayout>();
    let buffer = (*parser).buffer_buf.as_mut_ptr();
    let mut iov = [
        libc::iovec {
            iov_base: buffer.add((*parser).in_ as usize).cast(),
            iov_len: 0,
        },
        libc::iovec {
            iov_base: buffer.cast(),
            iov_len: 0,
        },
    ];
    let mut nio = 1;

    assert!((*parser).used <= REF_CACHE_BUF_SZ);
    if (*parser).used == REF_CACHE_BUF_SZ {
        return REF_CACHE_READ_MORE;
    }

    if (*parser).in_ > (*parser).out || (*parser).used == 0 {
        iov[0].iov_len = (REF_CACHE_BUF_SZ - (*parser).in_) as usize;
        if (*parser).out > 0 {
            iov[1].iov_len = (*parser).out as usize;
            nio = 2;
        }
    } else {
        assert!((*parser).in_ < (*parser).out);
        iov[0].iov_len = ((*parser).out - (*parser).in_) as usize;
    }

    let res = libc::readv(fd, iov.as_ptr(), nio);
    if res < 0 {
        let errno = *crate::htslib_rs::c_compat::__errno_location();
        if errno != libc::EAGAIN && errno != libc::EWOULDBLOCK && errno != libc::EINTR {
            libc::fprintf(
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"Error from fd #%d : %s\n".as_ptr(),
                fd,
                libc::strerror(errno),
            );
            return REF_CACHE_READ_ERROR;
        }
        return REF_CACHE_READ_BLOCKED;
    }

    (*parser).in_ = ((*parser).in_ + res as c_uint) & REF_CACHE_BUF_MASK;
    (*parser).used += res as c_uint;

    if res == 0 {
        REF_CACHE_READ_EOF
    } else if (res as usize) < iov[0].iov_len + iov[1].iov_len {
        REF_CACHE_READ_BLOCKED
    } else {
        REF_CACHE_READ_MORE
    }
}

// original: read_request_line (htslib/ref_cache/http_parser.c:246)
pub unsafe fn ref_cache_http_parser_c_246_read_request_line(
    opts: *const Options,
    parser: *mut Http_Parser,
) {
    let opts = opts.cast::<RefCacheOptionsLayout>();
    let parser = parser.cast::<HttpParserLayout>();
    let mut len = 0usize;
    let mut line: *mut c_char;

    loop {
        line = ref_cache_http_parser_c_158_parser_get_line(parser.cast(), &mut len);
        if line.is_null() || len != 0 {
            break;
        }
    }
    if line.is_null() {
        return;
    }

    if (*opts).verbosity > 2 {
        libc::fprintf(
            crate::htslib_rs::ref_cache::compat::stderr(),
            c"RECV'D: %s\n".as_ptr(),
            line,
        );
    }

    let line_bytes = std::slice::from_raw_parts(line.cast::<u8>(), len + 1);
    let reqlen = lws_cspn_bytes(line_bytes);
    let uripos = lws_spn_bytes(&line_bytes[reqlen..]) + reqlen;
    let urilen = lws_cspn_bytes(&line_bytes[uripos..]);
    let verpos = lws_spn_bytes(&line_bytes[uripos + urilen..]) + uripos + urilen;

    if reqlen == 0 || urilen == 0 {
        (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
        return;
    }
    if urilen > 128 {
        (*parser).state = REF_CACHE_ERR_LONG_URI;
        return;
    }

    match reqlen {
        3 => {
            if &line_bytes[..reqlen] == b"GET" {
                (*parser).req_type = REF_CACHE_REQ_GET;
            } else if &line_bytes[..reqlen] == b"PUT" {
                (*parser).req_type = REF_CACHE_REQ_PUT;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        4 => {
            if &line_bytes[..reqlen] == b"HEAD" {
                (*parser).req_type = REF_CACHE_REQ_HEAD;
            } else if &line_bytes[..reqlen] == b"POST" {
                (*parser).req_type = REF_CACHE_REQ_POST;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        5 => {
            if &line_bytes[..reqlen] == b"TRACE" {
                (*parser).req_type = REF_CACHE_REQ_TRACE;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        6 => {
            if &line_bytes[..reqlen] == b"DELETE" {
                (*parser).req_type = REF_CACHE_REQ_DELETE;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        7 => {
            if &line_bytes[..reqlen] == b"OPTIONS" {
                (*parser).req_type = REF_CACHE_REQ_OPTIONS;
            } else if &line_bytes[..reqlen] == b"CONNECT" {
                (*parser).req_type = REF_CACHE_REQ_CONNECT;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        _ => {
            (*parser).req_type = REF_CACHE_REQ_OTHER;
        }
    }

    (*parser).uri_buf.clear();
    (*parser)
        .uri_buf
        .extend_from_slice(&line_bytes[uripos..uripos + urilen]);
    (*parser).uri_buf.push(0);
    (*parser).refresh_raw_ptrs();

    if line_bytes[verpos] == 0 {
        (*parser).http_vers = REF_CACHE_HTTP_0_9;
    } else {
        let vers = &line_bytes[verpos..len];
        if !vers.starts_with(b"HTTP/") {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }

        let Some(dot) = vers[5..].iter().position(|&ch| ch == b'.') else {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        };
        let major = parse_decimal_c_ulong(&vers[5..5 + dot]);
        let minor = parse_decimal_c_ulong(&vers[6 + dot..]);
        let Some(major) = major else {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        };
        let Some(minor) = minor else {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        };
        if major == 0 {
            (*parser).http_vers = REF_CACHE_HTTP_0_9;
        } else if major == 1 {
            (*parser).http_vers = if minor == 0 { HTTP_1_0 } else { HTTP_1_1 };
        } else {
            (*parser).state = REF_CACHE_ERR_HTTP_VERS;
            return;
        }
    }

    if (*parser).http_vers == HTTP_1_1 {
        (*parser).flags |= TRANSACT_KEEP_ALIVE;
    }

    (*parser).state = REF_CACHE_READING_HEADERS;
}

// original: parse_range (htslib/ref_cache/http_parser.c:333)
pub unsafe fn ref_cache_http_parser_c_333_parse_range(parser: *mut Http_Parser) {
    let parser = parser.cast::<HttpParserLayout>();
    parse_range_bytes(&mut *parser);
}

// original: parser_parse_header (htslib/ref_cache/http_parser.c:394)
pub unsafe fn ref_cache_http_parser_c_394_parser_parse_header(parser: *mut Http_Parser) -> c_int {
    let parser = parser.cast::<HttpParserLayout>();
    let mut res = 0;

    let key = (&(*parser).key_buf)[..(*parser).key_used].to_vec();
    let val = (&(*parser).val_buf)[..(*parser).val_used].to_vec();

    if key.eq_ignore_ascii_case(b"Content-Length") {
        if (*parser).val_used == 0 {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        } else {
            if let Some(len) = parse_decimal_c_ulong(&val) {
                (*parser).content_length = len;
            } else {
                (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
                res = 1;
            }
        }
    } else if key.eq_ignore_ascii_case(b"Transfer-Encoding") {
        if (*parser).val_used == 0 {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        } else if val.eq_ignore_ascii_case(b"identity") {
            (*parser).trans_enc = REF_CACHE_TE_IDENT;
        } else if val.eq_ignore_ascii_case(b"chunked") {
            (*parser).trans_enc = REF_CACHE_TE_CHUNKED;
        } else {
            (*parser).state = REF_CACHE_ERR_UNIMPLEMENTED;
            res = 1;
        }
    } else if key.eq_ignore_ascii_case(b"Connection") {
        if ((*parser).flags & TRANSACT_KEEP_ALIVE) != 0 {
            let mut v = val.as_slice();
            while !v.is_empty() {
                let l = lws_cspn_bytes(v);
                if l == 5 && v[..l].eq_ignore_ascii_case(b"close") {
                    (*parser).flags &= !TRANSACT_KEEP_ALIVE;
                    break;
                }
                v = &v[l + lws_spn_bytes(&v[l..])..];
            }
        }
    } else if key.eq_ignore_ascii_case(b"User-Agent") {
        if (*parser).val_used != 0 {
            (*parser).user_agent_buf = limited_header_value(&val, REF_CACHE_MAX_UA_LEN);
            (*parser).refresh_raw_ptrs();
        }
    } else if key.eq_ignore_ascii_case(b"Referer") {
        if (*parser).val_used != 0 {
            (*parser).referrer_buf = limited_header_value(&val, REF_CACHE_MAX_REFERRER_LEN);
            (*parser).refresh_raw_ptrs();
        }
    } else if key.eq_ignore_ascii_case(b"Range") && (*parser).val_used != 0 {
        ref_cache_http_parser_c_333_parse_range(parser.cast());
    }

    (*parser).key_used = 0;
    (*parser).key_buf.clear();
    (*parser).key_buf.push(0);
    (*parser).val_used = 0;
    (*parser).val_buf.clear();
    (*parser).val_buf.push(0);
    (*parser).refresh_raw_ptrs();
    res
}

// original: read_headers (htslib/ref_cache/http_parser.c:452)
pub unsafe fn ref_cache_http_parser_c_452_read_headers(parser: *mut Http_Parser) {
    let parser_l = parser.cast::<HttpParserLayout>();
    let mut len = 0usize;
    let mut line = ref_cache_http_parser_c_158_parser_get_line(parser, &mut len);

    while !line.is_null() {
        if len == 0 {
            if (*parser_l).key_used != 0
                && ref_cache_http_parser_c_394_parser_parse_header(parser) != 0
            {
                return;
            }

            match (*parser_l).trans_enc {
                REF_CACHE_TE_IDENT => {
                    (*parser_l).state = if (*parser_l).content_length != 0 {
                        REF_CACHE_READING_BODY
                    } else {
                        REF_CACHE_GOT_REQUEST
                    };
                    (*parser_l).bytes = (*parser_l).content_length;
                }
                REF_CACHE_TE_CHUNKED => {
                    (*parser_l).state = REF_CACHE_READING_CHUNK_HEADER;
                }
                _ => {
                    (*parser_l).state = REF_CACHE_ERR_UNIMPLEMENTED;
                }
            }
            return;
        }

        let line_bytes = std::slice::from_raw_parts(line.cast::<u8>(), len + 1);
        let keylen = tok_spn_bytes(line_bytes);
        let mut spaces = lws_spn_bytes(&line_bytes[keylen..]);
        if keylen > 0 {
            if line_bytes[keylen + spaces] != b':' {
                (*parser_l).state = REF_CACHE_ERR_BAD_REQUEST;
                return;
            }
            spaces += 1 + lws_spn_bytes(&line_bytes[keylen + spaces + 1..]);

            if (*parser_l).key_used != 0
                && ref_cache_http_parser_c_394_parser_parse_header(parser) != 0
            {
                return;
            }

            (*parser_l).key_buf.clear();
            (*parser_l).key_buf.extend_from_slice(&line_bytes[..keylen]);
            (*parser_l).key_buf.push(0);
            (*parser_l).key_sz = (*parser_l).key_buf.capacity();
            (*parser_l).key_used = keylen;
            (*parser_l).val_used = 0;
            (*parser_l).val_buf.clear();
            (*parser_l).val_buf.push(0);
            (*parser_l).refresh_raw_ptrs();
        } else if spaces == 0 {
            (*parser_l).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }

        let valpos = keylen + spaces;
        if line_bytes[valpos] != 0 {
            (*parser_l).val_buf.pop();
            if (*parser_l).val_used > 0 {
                (*parser_l).val_buf.push(b' ');
                (*parser_l).val_used += 1;
            }
            (*parser_l)
                .val_buf
                .extend_from_slice(&line_bytes[valpos..len]);
            (*parser_l).val_used += len - valpos;
            (*parser_l).val_buf.push(0);
            (*parser_l).val_sz = (*parser_l).val_buf.capacity();
            (*parser_l).refresh_raw_ptrs();
        }

        line = ref_cache_http_parser_c_158_parser_get_line(parser, &mut len);
    }
}

// original: read_chunk_header (htslib/ref_cache/http_parser.c:529)
pub unsafe fn ref_cache_http_parser_c_529_read_chunk_header(parser: *mut Http_Parser) {
    let parser_l = parser.cast::<HttpParserLayout>();
    let mut len = 0usize;
    let line = ref_cache_http_parser_c_158_parser_get_line(parser, &mut len);
    if line.is_null() {
        return;
    }

    let mut end: *mut c_char = std::ptr::null_mut();
    let bytes = libc::strtoul(line, &mut end, 16);
    if *end != 0 && !ref_cache_http_parser_is_lws(*end as c_uchar) && *end != b';' as c_char {
        (*parser_l).state = REF_CACHE_ERR_BAD_REQUEST;
        return;
    }

    (*parser_l).bytes = bytes;
    (*parser_l).state = if bytes != 0 {
        REF_CACHE_READING_CHUNK
    } else {
        REF_CACHE_READING_CHUNK_TRAILER
    };
}

// original: eat_data (htslib/ref_cache/http_parser.c:546)
pub unsafe fn ref_cache_http_parser_c_546_eat_data(parser: *mut Http_Parser) {
    let parser = parser.cast::<HttpParserLayout>();
    let mut l: c_uint;

    if (*parser).used == 0 {
        assert!((*parser).in_ == (*parser).out);
        assert!((*parser).pos == (*parser).out);
        return;
    }

    if (*parser).in_ <= (*parser).out {
        l = REF_CACHE_BUF_SZ - (*parser).out;
        if (l as c_ulong) > (*parser).bytes {
            l = (*parser).bytes as c_uint;
        }
        assert!(l <= (*parser).used);
        (*parser).out = ((*parser).out + l) & REF_CACHE_BUF_MASK;
        (*parser).bytes -= l as c_ulong;
        (*parser).used -= l;
    }
    if (*parser).out < (*parser).in_ {
        l = (*parser).in_ - (*parser).out;
        if (l as c_ulong) > (*parser).bytes {
            l = (*parser).bytes as c_uint;
        }
        assert!(l <= (*parser).used);
        (*parser).out += l;
        (*parser).bytes -= l as c_ulong;
        (*parser).used -= l;
    }
    (*parser).pos = (*parser).out;
}

// original: read_chunk (htslib/ref_cache/http_parser.c:578)
pub unsafe fn ref_cache_http_parser_c_578_read_chunk(parser: *mut Http_Parser) {
    let parser_l = parser.cast::<HttpParserLayout>();
    ref_cache_http_parser_c_546_eat_data(parser);
    if (*parser_l).bytes == 0 {
        (*parser_l).state = REF_CACHE_READING_CHUNK_TRAILER;
    }
}

// original: read_body (htslib/ref_cache/http_parser.c:584)
pub unsafe fn ref_cache_http_parser_c_584_read_body(parser: *mut Http_Parser) {
    let parser_l = parser.cast::<HttpParserLayout>();
    ref_cache_http_parser_c_546_eat_data(parser);
    if (*parser_l).bytes == 0 {
        (*parser_l).state = REF_CACHE_GOT_REQUEST;
    }
}

// original: read_chunk_trailer (htslib/ref_cache/http_parser.c:590)
pub unsafe fn ref_cache_http_parser_c_590_read_chunk_trailer(parser: *mut Http_Parser) {
    let parser_l = parser.cast::<HttpParserLayout>();
    let mut len = 0usize;
    while !ref_cache_http_parser_c_158_parser_get_line(parser, &mut len).is_null() {
        if len == 0 {
            (*parser_l).state = REF_CACHE_GOT_REQUEST;
            return;
        }
    }
}

// original: parser_read_data (htslib/ref_cache/http_parser.c:601)
pub unsafe fn ref_cache_http_parser_c_601_parser_read_data(
    opts: *const Options,
    client: *mut Client,
    parser: *mut Http_Parser,
    fd: c_int,
) -> c_int {
    let parser_layout = parser.cast::<HttpParserLayout>();
    let res = ref_cache_http_parser_c_201_parser_read_input(parser, fd);
    if res == REF_CACHE_READ_EOF || res == REF_CACHE_READ_ERROR {
        return res;
    }

    loop {
        let mut transact: *mut Transaction = std::ptr::null_mut();
        let last_state = (*parser_layout).state;

        match (*parser_layout).state {
            REF_CACHE_READING_REQUEST_LINE => {
                ref_cache_http_parser_c_246_read_request_line(opts, parser);
            }
            REF_CACHE_READING_HEADERS => {
                ref_cache_http_parser_c_452_read_headers(parser);
            }
            REF_CACHE_READING_CHUNK_HEADER => {
                ref_cache_http_parser_c_529_read_chunk_header(parser);
            }
            REF_CACHE_READING_CHUNK => {
                ref_cache_http_parser_c_578_read_chunk(parser);
            }
            REF_CACHE_READING_BODY => {
                ref_cache_http_parser_c_584_read_body(parser);
            }
            REF_CACHE_READING_CHUNK_TRAILER => {
                ref_cache_http_parser_c_590_read_chunk_trailer(parser);
            }
            REF_CACHE_GOT_REQUEST => {
                ref_cache_request_handler_c_148_handle_request(opts, client, parser, &mut transact);
            }
            REF_CACHE_SHUTTING_DOWN => {
                return REF_CACHE_READ_EOF;
            }
            _ => {
                ref_cache_request_handler_c_167_handle_error(
                    client,
                    parser,
                    (*parser_layout).state,
                    &mut transact,
                );
            }
        }

        if !transact.is_null() {
            ref_cache_server_c_352_client_add_transaction(client, transact);
        }

        if last_state == (*parser_layout).state {
            break;
        }
    }

    res
}

// original: Http_Parser (htslib/ref_cache/http_parser.h:87)
#[repr(C)]
pub struct Http_Parser {
    _private: [u8; 0],
}
