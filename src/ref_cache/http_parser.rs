/*  http_parser.c -- ref-cache HTTP protocol handler

    Copyright (C) 2025 Genome Research Ltd.

    Author: Rob Davies <rmd@sanger.ac.uk>
*/

use super::options::Options;
use super::request_handler::{
    ref_cache_request_handler_c_148_handle_request, ref_cache_request_handler_c_167_handle_error,
};
use super::server::ref_cache_server_c_352_client_add_transaction;
use super::server::RefCacheClientsLayout;
use super::transaction::TransactionId;

const REF_CACHE_MAX_UA_LEN: usize = 128;
const REF_CACHE_MAX_REFERRER_LEN: usize = 128;
const REF_CACHE_READING_REQUEST_LINE: i32 = 0;
const REF_CACHE_READING_HEADERS: i32 = 1;
const REF_CACHE_READING_CHUNK_HEADER: i32 = 2;
const REF_CACHE_READING_CHUNK: i32 = 3;
const REF_CACHE_READING_CHUNK_TRAILER: i32 = 4;
const REF_CACHE_READING_BODY: i32 = 5;
const REF_CACHE_GOT_REQUEST: i32 = 6;
const REF_CACHE_SHUTTING_DOWN: i32 = 7;
const REF_CACHE_ERR_BAD_REQUEST: i32 = 400;
const REF_CACHE_ERR_TOO_LARGE: i32 = 413;
const REF_CACHE_ERR_LONG_URI: i32 = 414;
const REF_CACHE_ERR_INTERNAL: i32 = 500;
const REF_CACHE_ERR_UNIMPLEMENTED: i32 = 501;
const REF_CACHE_ERR_HTTP_VERS: i32 = 505;
const REF_CACHE_REQ_OPTIONS: i32 = 0;
const REF_CACHE_REQ_GET: i32 = 1;
const REF_CACHE_REQ_HEAD: i32 = 2;
const REF_CACHE_REQ_POST: i32 = 3;
const REF_CACHE_REQ_PUT: i32 = 4;
const REF_CACHE_REQ_DELETE: i32 = 5;
const REF_CACHE_REQ_TRACE: i32 = 6;
const REF_CACHE_REQ_CONNECT: i32 = 7;
const REF_CACHE_REQ_OTHER: i32 = 8;
const REF_CACHE_HTTP_0_9: i32 = 0;
const HTTP_1_0: i32 = 1;
const HTTP_1_1: i32 = 2;
const REF_CACHE_TE_IDENT: i32 = 0;
const REF_CACHE_TE_CHUNKED: i32 = 1;
const REF_CACHE_READ_BLOCKED: i32 = 0;
const REF_CACHE_READ_MORE: i32 = 1;
const REF_CACHE_READ_EOF: i32 = 2;
const REF_CACHE_READ_ERROR: i32 = 3;
const TRANSACT_KEEP_ALIVE: u32 = 2;
const TRANSACT_RANGE_FROM: u32 = 8;
const TRANSACT_RANGE_TO: u32 = 16;
const TRANSACT_RANGE_SUFFIX: u32 = 32;
const REF_CACHE_BUF_SZ: u32 = 0x400;
const REF_CACHE_BUF_MASK: u32 = REF_CACHE_BUF_SZ - 1;

static REF_CACHE_LWS_CHARS: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static REF_CACHE_TOKEN_CHARS: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// The parser owns all of its scratch state. A request line is reassembled
// into an owned `Vec<u8>` returned by
// `ref_cache_http_parser_c_158_parser_get_line` and consumed before the next
// call (the `ref-cache` daemon is a single-threaded, fork-based epoll worker;
// see `src/ref_cache/main.rs` and `src/ref_cache/server.rs`).

pub struct HttpParser {
    state: i32,
    req_type: i32,
    http_vers: i32,
    trans_enc: i32,
    content_length: u64,
    bytes: u64,
    // Header key/value accumulators. These hold the logical bytes only (no
    // trailing NUL); `.len()` is the used length.
    uri: Vec<u8>,
    key: Vec<u8>,
    val: Vec<u8>,
    user_agent: Vec<u8>,
    referrer: Vec<u8>,
    // Ring buffer of received bytes.
    buffer: Vec<u8>,
    range_from: i64,
    range_to: i64,
    upstream: i32,
    flags: u32,
    in_: u32,
    out: u32,
    pos: u32,
    used: u32,
}

impl HttpParser {
    pub fn uri(&self) -> &[u8] {
        &self.uri
    }
    pub fn upstream(&self) -> i32 {
        self.upstream
    }
    pub fn req_type(&self) -> i32 {
        self.req_type
    }
    pub fn http_vers(&self) -> i32 {
        self.http_vers
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn state(&self) -> i32 {
        self.state
    }
    pub fn set_state(&mut self, state: i32) {
        self.state = state;
    }
    pub fn range_from(&self) -> i64 {
        self.range_from
    }
    pub fn range_to(&self) -> i64 {
        self.range_to
    }
    // Move the accumulated User-Agent / Referer out of the parser into the
    // transaction that is taking ownership of them.
    pub fn take_user_agent(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.user_agent)
    }
    pub fn take_referrer(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.referrer)
    }
}

fn ref_cache_http_parser_is_lws(ch: u8) -> bool {
    REF_CACHE_LWS_CHARS[ch as usize] != 0
}

fn ref_cache_http_parser_is_token(ch: u8) -> bool {
    REF_CACHE_TOKEN_CHARS[ch as usize] != 0
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

fn parse_decimal_c_ulong(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut out: u64 = 0;
    for &digit in bytes {
        out = out
            .checked_mul(10)?
            .checked_add((digit - b'0') as u64)?;
    }
    Some(out)
}

fn parse_decimal_off_t(bytes: &[u8]) -> Option<(i64, usize)> {
    let digits = bytes.iter().take_while(|&&ch| ch.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }

    let mut out: i128 = 0;
    for &digit in &bytes[..digits] {
        out = out.checked_mul(10)?.checked_add((digit - b'0') as i128)?;
    }

    let off_max = i64::MAX as i128;
    if out > off_max {
        return None;
    }

    Some((out as i64, digits))
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
    out
}

fn parse_range_bytes(parser: &mut HttpParser) {
    let mut v = parser.val.as_slice();
    let off_max: i64 = i64::MAX;

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
pub fn ref_cache_http_parser_c_93_lws_spn(s: &[u8]) -> usize {
    lws_spn_bytes(s)
}

// original: lws_cspn (htslib/ref_cache/http_parser.c:99)
pub fn ref_cache_http_parser_c_99_lws_cspn(s: &[u8]) -> usize {
    lws_cspn_bytes(s)
}

// original: tok_spn (htslib/ref_cache/http_parser.c:105)
pub fn ref_cache_http_parser_c_105_tok_spn(s: &[u8]) -> usize {
    tok_spn_bytes(s)
}

// original: init_http_parser (htslib/ref_cache/http_parser.c:111)
pub fn ref_cache_http_parser_c_111_init_http_parser(upstream: i32) -> HttpParser {
    HttpParser {
        state: 0,
        req_type: 0,
        http_vers: 0,
        trans_enc: 0,
        content_length: 0,
        bytes: 0,
        uri: Vec::new(),
        key: Vec::new(),
        val: Vec::new(),
        user_agent: Vec::new(),
        referrer: Vec::new(),
        buffer: vec![0; REF_CACHE_BUF_SZ as usize],
        range_from: 0,
        range_to: 0,
        upstream,
        flags: 0,
        in_: 0,
        out: 0,
        pos: 0,
        used: 0,
    }
}

// original: cleanup_http_parser (htslib/ref_cache/http_parser.c:131)
pub fn ref_cache_http_parser_c_131_cleanup_http_parser(_parser: &mut HttpParser) {
    // Owned `Vec`s are released by `HttpParser`'s `Drop`; nothing to do here.
}

// original: parser_get_line (htslib/ref_cache/http_parser.c:158)
//
// Reassembles a single line (up to a `\n`) out of the ring buffer and returns
// it as owned bytes, with any trailing `\r` stripped. Returns `None` when no
// complete line is currently available.
pub fn ref_cache_http_parser_c_158_parser_get_line(parser: &mut HttpParser) -> Option<Vec<u8>> {
    if parser.used == 0 {
        assert!(parser.in_ == parser.out);
        assert!(parser.pos == parser.out);
        return None;
    }

    loop {
        if parser.buffer[parser.pos as usize] == b'\n' {
            break;
        }
        parser.pos = (parser.pos + 1) & REF_CACHE_BUF_MASK;
        if parser.pos == parser.in_ {
            break;
        }
    }
    if parser.pos == parser.in_ && parser.buffer[parser.pos as usize] != b'\n' {
        if parser.used == REF_CACHE_BUF_SZ {
            parser.state = REF_CACHE_ERR_TOO_LARGE;
        }
        return None;
    }

    let mut line = Vec::new();
    if parser.pos <= parser.out {
        let n = (REF_CACHE_BUF_SZ - parser.out) as usize;
        let out = parser.out as usize;
        line.extend_from_slice(&parser.buffer[out..out + n]);
        parser.out = 0;
    }
    let n = (parser.pos - parser.out) as usize;
    let out = parser.out as usize;
    line.extend_from_slice(&parser.buffer[out..out + n]);
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    parser.pos = (parser.pos + 1) & REF_CACHE_BUF_MASK;
    parser.out = parser.pos;
    parser.used = if parser.out > parser.in_ {
        REF_CACHE_BUF_SZ - parser.out + parser.in_
    } else {
        parser.in_ - parser.out
    };
    Some(line)
}

// original: parser_read_input (htslib/ref_cache/http_parser.c:201)
pub fn ref_cache_http_parser_c_201_parser_read_input(parser: &mut HttpParser, fd: i32) -> i32 {
    assert!(parser.used <= REF_CACHE_BUF_SZ);
    if parser.used == REF_CACHE_BUF_SZ {
        return REF_CACHE_READ_MORE;
    }

    // Compute the two contiguous free regions of the ring buffer.
    let (head_off, head_len, tail_len) = if parser.in_ > parser.out || parser.used == 0 {
        (
            parser.in_ as usize,
            (REF_CACHE_BUF_SZ - parser.in_) as usize,
            parser.out as usize,
        )
    } else {
        assert!(parser.in_ < parser.out);
        (
            parser.in_ as usize,
            (parser.out - parser.in_) as usize,
            0,
        )
    };

    // Scatter read into the (up to) two free regions. `readv` is a genuine
    // syscall with no std equivalent for the gather form, so it stays; only the
    // pointers handed to it are derived from the owned `Vec`.
    let buffer = parser.buffer.as_mut_ptr();
    let iov = [
        libc::iovec {
            iov_base: unsafe { buffer.add(head_off) }.cast(),
            iov_len: head_len,
        },
        libc::iovec {
            iov_base: buffer.cast(),
            iov_len: tail_len,
        },
    ];
    let nio = if tail_len > 0 { 2 } else { 1 };

    let res = unsafe { libc::readv(fd, iov.as_ptr(), nio) };
    if res < 0 {
        let err = std::io::Error::last_os_error();
        let errno = err.raw_os_error().unwrap_or(0);
        if errno != libc::EAGAIN && errno != libc::EWOULDBLOCK && errno != libc::EINTR {
            eprintln!("Error from fd #{} : {}", fd, err);
            return REF_CACHE_READ_ERROR;
        }
        return REF_CACHE_READ_BLOCKED;
    }

    parser.in_ = (parser.in_ + res as u32) & REF_CACHE_BUF_MASK;
    parser.used += res as u32;

    if res == 0 {
        REF_CACHE_READ_EOF
    } else if (res as usize) < head_len + tail_len {
        REF_CACHE_READ_BLOCKED
    } else {
        REF_CACHE_READ_MORE
    }
}

// original: read_request_line (htslib/ref_cache/http_parser.c:246)
pub fn ref_cache_http_parser_c_246_read_request_line(opts: &Options, parser: &mut HttpParser) {
    let line = loop {
        match ref_cache_http_parser_c_158_parser_get_line(parser) {
            None => return,
            Some(l) if !l.is_empty() => break l,
            Some(_) => continue,
        }
    };
    let len = line.len();

    if opts.verbosity > 2 {
        eprintln!("RECV'D: {}", String::from_utf8_lossy(&line));
    }

    let reqlen = lws_cspn_bytes(&line);
    let uripos = lws_spn_bytes(&line[reqlen..]) + reqlen;
    let urilen = lws_cspn_bytes(&line[uripos..]);
    let verpos = lws_spn_bytes(&line[uripos + urilen..]) + uripos + urilen;

    if reqlen == 0 || urilen == 0 {
        parser.state = REF_CACHE_ERR_BAD_REQUEST;
        return;
    }
    if urilen > 128 {
        parser.state = REF_CACHE_ERR_LONG_URI;
        return;
    }

    parser.req_type = match &line[..reqlen] {
        b"GET" => REF_CACHE_REQ_GET,
        b"PUT" => REF_CACHE_REQ_PUT,
        b"HEAD" => REF_CACHE_REQ_HEAD,
        b"POST" => REF_CACHE_REQ_POST,
        b"TRACE" => REF_CACHE_REQ_TRACE,
        b"DELETE" => REF_CACHE_REQ_DELETE,
        b"OPTIONS" => REF_CACHE_REQ_OPTIONS,
        b"CONNECT" => REF_CACHE_REQ_CONNECT,
        _ => REF_CACHE_REQ_OTHER,
    };

    parser.uri.clear();
    parser.uri.extend_from_slice(&line[uripos..uripos + urilen]);

    if verpos >= len {
        parser.http_vers = REF_CACHE_HTTP_0_9;
    } else {
        let vers = &line[verpos..len];
        if !vers.starts_with(b"HTTP/") {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }

        let Some(dot) = vers[5..].iter().position(|&ch| ch == b'.') else {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        };
        let major = parse_decimal_c_ulong(&vers[5..5 + dot]);
        let minor = parse_decimal_c_ulong(&vers[6 + dot..]);
        let Some(major) = major else {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        };
        let Some(minor) = minor else {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        };
        if major == 0 {
            parser.http_vers = REF_CACHE_HTTP_0_9;
        } else if major == 1 {
            parser.http_vers = if minor == 0 { HTTP_1_0 } else { HTTP_1_1 };
        } else {
            parser.state = REF_CACHE_ERR_HTTP_VERS;
            return;
        }
    }

    if parser.http_vers == HTTP_1_1 {
        parser.flags |= TRANSACT_KEEP_ALIVE;
    }

    parser.state = REF_CACHE_READING_HEADERS;
}

// original: parse_range (htslib/ref_cache/http_parser.c:333)
pub fn ref_cache_http_parser_c_333_parse_range(parser: &mut HttpParser) {
    parse_range_bytes(parser);
}

// original: parser_parse_header (htslib/ref_cache/http_parser.c:394)
pub fn ref_cache_http_parser_c_394_parser_parse_header(parser: &mut HttpParser) -> i32 {
    let mut res = 0;

    let key = std::mem::take(&mut parser.key);
    let val = std::mem::take(&mut parser.val);

    if key.eq_ignore_ascii_case(b"Content-Length") {
        if val.is_empty() {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        } else if let Some(len) = parse_decimal_c_ulong(&val) {
            parser.content_length = len;
        } else {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        }
    } else if key.eq_ignore_ascii_case(b"Transfer-Encoding") {
        if val.is_empty() {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        } else if val.eq_ignore_ascii_case(b"identity") {
            parser.trans_enc = REF_CACHE_TE_IDENT;
        } else if val.eq_ignore_ascii_case(b"chunked") {
            parser.trans_enc = REF_CACHE_TE_CHUNKED;
        } else {
            parser.state = REF_CACHE_ERR_UNIMPLEMENTED;
            res = 1;
        }
    } else if key.eq_ignore_ascii_case(b"Connection") {
        if (parser.flags & TRANSACT_KEEP_ALIVE) != 0 {
            let mut v = val.as_slice();
            while !v.is_empty() {
                let l = lws_cspn_bytes(v);
                if l == 5 && v[..l].eq_ignore_ascii_case(b"close") {
                    parser.flags &= !TRANSACT_KEEP_ALIVE;
                    break;
                }
                v = &v[l + lws_spn_bytes(&v[l..])..];
            }
        }
    } else if key.eq_ignore_ascii_case(b"User-Agent") {
        if !val.is_empty() {
            parser.user_agent = limited_header_value(&val, REF_CACHE_MAX_UA_LEN);
        }
    } else if key.eq_ignore_ascii_case(b"Referer") {
        if !val.is_empty() {
            parser.referrer = limited_header_value(&val, REF_CACHE_MAX_REFERRER_LEN);
        }
    } else if key.eq_ignore_ascii_case(b"Range") && !val.is_empty() {
        parser.val = val;
        ref_cache_http_parser_c_333_parse_range(parser);
    }

    parser.key.clear();
    parser.val.clear();
    res
}

// original: read_headers (htslib/ref_cache/http_parser.c:452)
pub fn ref_cache_http_parser_c_452_read_headers(parser: &mut HttpParser) {
    while let Some(line) = ref_cache_http_parser_c_158_parser_get_line(parser) {
        let len = line.len();
        if len == 0 {
            if !parser.key.is_empty() && ref_cache_http_parser_c_394_parser_parse_header(parser) != 0
            {
                return;
            }

            match parser.trans_enc {
                REF_CACHE_TE_IDENT => {
                    parser.state = if parser.content_length != 0 {
                        REF_CACHE_READING_BODY
                    } else {
                        REF_CACHE_GOT_REQUEST
                    };
                    parser.bytes = parser.content_length;
                }
                REF_CACHE_TE_CHUNKED => {
                    parser.state = REF_CACHE_READING_CHUNK_HEADER;
                }
                _ => {
                    parser.state = REF_CACHE_ERR_UNIMPLEMENTED;
                }
            }
            return;
        }

        let keylen = tok_spn_bytes(&line);
        let mut spaces = lws_spn_bytes(&line[keylen..]);
        if keylen > 0 {
            if line.get(keylen + spaces) != Some(&b':') {
                parser.state = REF_CACHE_ERR_BAD_REQUEST;
                return;
            }
            spaces += 1 + lws_spn_bytes(&line[keylen + spaces + 1..]);

            if !parser.key.is_empty() && ref_cache_http_parser_c_394_parser_parse_header(parser) != 0
            {
                return;
            }

            parser.key.clear();
            parser.key.extend_from_slice(&line[..keylen]);
            parser.val.clear();
        } else if spaces == 0 {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }

        let valpos = keylen + spaces;
        if valpos < len {
            if !parser.val.is_empty() {
                parser.val.push(b' ');
            }
            parser.val.extend_from_slice(&line[valpos..len]);
        }
    }
}

// original: read_chunk_header (htslib/ref_cache/http_parser.c:529)
pub fn ref_cache_http_parser_c_529_read_chunk_header(parser: &mut HttpParser) {
    let Some(line) = ref_cache_http_parser_c_158_parser_get_line(parser) else {
        return;
    };

    // Parse a hexadecimal chunk size, like strtoul(.., 16) did, skipping any
    // leading whitespace and reading hex digits up to the first non-digit.
    let start = lws_spn_bytes(&line);
    let digits = line[start..]
        .iter()
        .take_while(|b| b.is_ascii_hexdigit())
        .count();
    let mut bytes: u64 = 0;
    for &b in &line[start..start + digits] {
        bytes = bytes.wrapping_mul(16).wrapping_add((b as char).to_digit(16).unwrap() as u64);
    }
    let end = &line[start + digits..];
    if let Some(&ch) = end.first() {
        if !ref_cache_http_parser_is_lws(ch) && ch != b';' {
            parser.state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }
    }

    parser.bytes = bytes;
    parser.state = if bytes != 0 {
        REF_CACHE_READING_CHUNK
    } else {
        REF_CACHE_READING_CHUNK_TRAILER
    };
}

// original: eat_data (htslib/ref_cache/http_parser.c:546)
pub fn ref_cache_http_parser_c_546_eat_data(parser: &mut HttpParser) {
    let mut l: u32;

    if parser.used == 0 {
        assert!(parser.in_ == parser.out);
        assert!(parser.pos == parser.out);
        return;
    }

    if parser.in_ <= parser.out {
        l = REF_CACHE_BUF_SZ - parser.out;
        if (l as u64) > parser.bytes {
            l = parser.bytes as u32;
        }
        assert!(l <= parser.used);
        parser.out = (parser.out + l) & REF_CACHE_BUF_MASK;
        parser.bytes -= l as u64;
        parser.used -= l;
    }
    if parser.out < parser.in_ {
        l = parser.in_ - parser.out;
        if (l as u64) > parser.bytes {
            l = parser.bytes as u32;
        }
        assert!(l <= parser.used);
        parser.out += l;
        parser.bytes -= l as u64;
        parser.used -= l;
    }
    parser.pos = parser.out;
}

// original: read_chunk (htslib/ref_cache/http_parser.c:578)
pub fn ref_cache_http_parser_c_578_read_chunk(parser: &mut HttpParser) {
    ref_cache_http_parser_c_546_eat_data(parser);
    if parser.bytes == 0 {
        parser.state = REF_CACHE_READING_CHUNK_TRAILER;
    }
}

// original: read_body (htslib/ref_cache/http_parser.c:584)
pub fn ref_cache_http_parser_c_584_read_body(parser: &mut HttpParser) {
    ref_cache_http_parser_c_546_eat_data(parser);
    if parser.bytes == 0 {
        parser.state = REF_CACHE_GOT_REQUEST;
    }
}

// original: read_chunk_trailer (htslib/ref_cache/http_parser.c:590)
pub fn ref_cache_http_parser_c_590_read_chunk_trailer(parser: &mut HttpParser) {
    while let Some(line) = ref_cache_http_parser_c_158_parser_get_line(parser) {
        if line.is_empty() {
            parser.state = REF_CACHE_GOT_REQUEST;
            return;
        }
    }
}

// original: parser_read_data (htslib/ref_cache/http_parser.c:601)
//
// Ownership: the parser is borrowed out of its owning client slot by the caller;
// the client is identified by its arena index `client` into `clients`. A produced
// transaction is an arena index (`TransactionId`) appended to the client's
// pipeline via `client_add_transaction`.
pub unsafe fn ref_cache_http_parser_c_601_parser_read_data(
    opts: &Options,
    clients: &mut RefCacheClientsLayout,
    client: usize,
    parser: &mut HttpParser,
    fd: i32,
) -> i32 {
    let res = ref_cache_http_parser_c_201_parser_read_input(parser, fd);
    if res == REF_CACHE_READ_EOF || res == REF_CACHE_READ_ERROR {
        return res;
    }

    loop {
        let mut transact: Option<TransactionId> = None;
        let last_state = parser.state;

        match parser.state {
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
                ref_cache_request_handler_c_148_handle_request(
                    opts,
                    clients,
                    client,
                    parser,
                    &mut transact,
                );
            }
            REF_CACHE_SHUTTING_DOWN => {
                return REF_CACHE_READ_EOF;
            }
            _ => {
                ref_cache_request_handler_c_167_handle_error(
                    clients,
                    client,
                    parser,
                    last_state,
                    &mut transact,
                );
            }
        }

        if let Some(transact) = transact {
            ref_cache_server_c_352_client_add_transaction(clients, client, transact);
        }

        if last_state == parser.state {
            break;
        }
    }

    res
}
