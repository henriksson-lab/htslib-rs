/*  http_parser.c -- ref-cache HTTP protocol handler

    Copyright (C) 2025 Genome Research Ltd.

    Author: Rob Davies <rmd@sanger.ac.uk>
*/

use super::misc::ref_cache_misc_h_91_lim_strdup;
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
    libc::memset(parser.cast(), 0, std::mem::size_of::<HttpParserLayout>());

    (*parser).uri = std::ptr::null_mut();
    (*parser).key = std::ptr::null_mut();
    (*parser).val = std::ptr::null_mut();
    (*parser).user_agent = std::ptr::null_mut();
    (*parser).referrer = std::ptr::null_mut();

    (*parser).buffer = libc::malloc(REF_CACHE_BUF_SZ as usize).cast();
    if (*parser).buffer.is_null() {
        return -1;
    }

    (*parser).upstream = upstream;
    0
}

// original: cleanup_http_parser (htslib/ref_cache/http_parser.c:131)
pub unsafe fn ref_cache_http_parser_c_131_cleanup_http_parser(parser: *mut Http_Parser) {
    let parser = parser.cast::<HttpParserLayout>();
    if !(*parser).uri.is_null() {
        libc::free((*parser).uri.cast());
        (*parser).uri = std::ptr::null_mut();
    }
    if !(*parser).key.is_null() {
        libc::free((*parser).key.cast());
        (*parser).key = std::ptr::null_mut();
    }
    if !(*parser).val.is_null() {
        libc::free((*parser).val.cast());
        (*parser).val = std::ptr::null_mut();
    }
    if !(*parser).user_agent.is_null() {
        libc::free((*parser).user_agent.cast());
        (*parser).user_agent = std::ptr::null_mut();
    }
    if !(*parser).referrer.is_null() {
        libc::free((*parser).referrer.cast());
        (*parser).referrer = std::ptr::null_mut();
    }
    if !(*parser).buffer.is_null() {
        libc::free((*parser).buffer.cast());
        (*parser).buffer = std::ptr::null_mut();
    }
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
        if *(*parser).buffer.add((*parser).pos as usize) == b'\n' as c_char {
            break;
        }
        (*parser).pos = ((*parser).pos + 1) & REF_CACHE_BUF_MASK;
        if (*parser).pos == (*parser).in_ {
            break;
        }
    }
    if (*parser).pos == (*parser).in_
        && *(*parser).buffer.add((*parser).pos as usize) != b'\n' as c_char
    {
        if (*parser).used == REF_CACHE_BUF_SZ {
            (*parser).state = REF_CACHE_ERR_TOO_LARGE;
            return std::ptr::null_mut();
        }
        return std::ptr::null_mut();
    }

    lpos = 0;
    if (*parser).pos <= (*parser).out {
        let n = (REF_CACHE_BUF_SZ - (*parser).out) as usize;
        libc::memcpy(
            line.add(lpos).cast(),
            (*parser).buffer.add((*parser).out as usize).cast(),
            n,
        );
        lpos += n;
        (*parser).out = 0;
    }
    let n = ((*parser).pos - (*parser).out) as usize;
    libc::memcpy(
        line.add(lpos).cast(),
        (*parser).buffer.add((*parser).out as usize).cast(),
        n,
    );
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
    let mut iov = [
        libc::iovec {
            iov_base: (*parser).buffer.add((*parser).in_ as usize).cast(),
            iov_len: 0,
        },
        libc::iovec {
            iov_base: (*parser).buffer.cast(),
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

    let reqlen = ref_cache_http_parser_c_99_lws_cspn(line);
    let uripos = ref_cache_http_parser_c_93_lws_spn(line.add(reqlen)) + reqlen;
    let urilen = ref_cache_http_parser_c_99_lws_cspn(line.add(uripos));
    let verpos = ref_cache_http_parser_c_93_lws_spn(line.add(uripos + urilen)) + uripos + urilen;

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
            if libc::strncmp(line, c"GET".as_ptr(), 3) == 0 {
                (*parser).req_type = REF_CACHE_REQ_GET;
            } else if libc::strncmp(line, c"PUT".as_ptr(), 3) == 0 {
                (*parser).req_type = REF_CACHE_REQ_PUT;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        4 => {
            if libc::strncmp(line, c"HEAD".as_ptr(), 4) == 0 {
                (*parser).req_type = REF_CACHE_REQ_HEAD;
            } else if libc::strncmp(line, c"POST".as_ptr(), 4) == 0 {
                (*parser).req_type = REF_CACHE_REQ_POST;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        5 => {
            if libc::strncmp(line, c"TRACE".as_ptr(), 5) == 0 {
                (*parser).req_type = REF_CACHE_REQ_TRACE;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        6 => {
            if libc::strncmp(line, c"DELETE".as_ptr(), 6) == 0 {
                (*parser).req_type = REF_CACHE_REQ_DELETE;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        7 => {
            if libc::strncmp(line, c"OPTIONS".as_ptr(), 7) == 0 {
                (*parser).req_type = REF_CACHE_REQ_OPTIONS;
            } else if libc::strncmp(line, c"CONNECT".as_ptr(), 7) == 0 {
                (*parser).req_type = REF_CACHE_REQ_CONNECT;
            } else {
                (*parser).req_type = REF_CACHE_REQ_OTHER;
            }
        }
        _ => {
            (*parser).req_type = REF_CACHE_REQ_OTHER;
        }
    }

    (*parser).uri = libc::malloc(urilen + 1).cast();
    if (*parser).uri.is_null() {
        (*parser).state = REF_CACHE_ERR_INTERNAL;
        return;
    }
    libc::memcpy((*parser).uri.cast(), line.add(uripos).cast(), urilen);
    *(*parser).uri.add(urilen) = 0;

    if *line.add(verpos) == 0 {
        (*parser).http_vers = REF_CACHE_HTTP_0_9;
    } else {
        let mut p: *mut c_char = std::ptr::null_mut();
        if libc::strncmp(line.add(verpos), c"HTTP/".as_ptr(), 5) != 0 {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }

        let major = libc::strtol(line.add(verpos + 5), &mut p, 10);
        if *p != b'.' as c_char {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }
        let minor = libc::strtol(p.add(1), std::ptr::null_mut(), 10);
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
    let mut v = (*parser).val;
    let mut end: *mut c_char = std::ptr::null_mut();
    let off_max: libc::off_t = if std::mem::size_of::<libc::off_t>() < 8 {
        i32::MAX as libc::off_t
    } else {
        i64::MAX as libc::off_t
    };

    if libc::strncasecmp(v, c"bytes".as_ptr(), 5) != 0 {
        (*parser).flags &= !(TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
        return;
    }
    v = v.add(5 + ref_cache_http_parser_c_93_lws_spn(v.add(5)));
    if *v != b'=' as c_char {
        (*parser).flags &= !(TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
        return;
    }
    v = v.add(1 + ref_cache_http_parser_c_93_lws_spn(v.add(1)));

    (*parser).range_from = off_max;
    (*parser).range_to = 0;

    loop {
        if *v == b'-' as c_char {
            let ll = libc::strtoll(v.add(1), &mut end, 10);
            if ll < 0 {
                break;
            }
            (*parser).flags |= TRANSACT_RANGE_SUFFIX;
            if (*parser).range_to < ll as libc::off_t {
                (*parser).range_to = ll as libc::off_t;
            }
            v = end.add(ref_cache_http_parser_c_93_lws_spn(end));
        } else {
            let ll = libc::strtoll(v, &mut end, 10);
            if ll < 0 {
                break;
            }
            (*parser).flags |= TRANSACT_RANGE_FROM;
            if (*parser).range_from > ll as libc::off_t {
                (*parser).range_from = ll as libc::off_t;
            }
            v = end.add(ref_cache_http_parser_c_93_lws_spn(end));
            if *v != b'-' as c_char {
                break;
            }
            v = v.add(1 + ref_cache_http_parser_c_93_lws_spn(v.add(1)));
            if *v == 0 {
                return;
            }
            let ll = libc::strtoll(v, &mut end, 10);
            (*parser).flags |= TRANSACT_RANGE_TO;
            if (*parser).range_to < ll as libc::off_t {
                (*parser).range_to = ll as libc::off_t;
            }
            v = end.add(ref_cache_http_parser_c_93_lws_spn(end));
        }
        if *v == 0 {
            if ((*parser).flags & (TRANSACT_RANGE_FROM | TRANSACT_RANGE_SUFFIX))
                == (TRANSACT_RANGE_FROM | TRANSACT_RANGE_SUFFIX)
            {
                (*parser).flags &= !(TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
            }
            return;
        }
        if *v != b',' as c_char {
            break;
        }
    }
    (*parser).flags &= !(TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
}

// original: parser_parse_header (htslib/ref_cache/http_parser.c:394)
pub unsafe fn ref_cache_http_parser_c_394_parser_parse_header(parser: *mut Http_Parser) -> c_int {
    let parser = parser.cast::<HttpParserLayout>();
    let mut res = 0;

    if libc::strcasecmp((*parser).key, c"Content-Length".as_ptr()) == 0 {
        let mut end: *mut c_char = std::ptr::null_mut();
        if (*parser).val_used == 0 {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        } else {
            let len = libc::strtoul((*parser).val, &mut end, 10);
            if *(*parser).val == 0 || *end != 0 {
                (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
                res = 1;
            } else {
                (*parser).content_length = len;
            }
        }
    } else if libc::strcasecmp((*parser).key, c"Transfer-Encoding".as_ptr()) == 0 {
        if (*parser).val_used == 0 {
            (*parser).state = REF_CACHE_ERR_BAD_REQUEST;
            res = 1;
        } else if libc::strcasecmp((*parser).val, c"identity".as_ptr()) == 0 {
            (*parser).trans_enc = REF_CACHE_TE_IDENT;
        } else if libc::strcasecmp((*parser).val, c"chunked".as_ptr()) == 0 {
            (*parser).trans_enc = REF_CACHE_TE_CHUNKED;
        } else {
            (*parser).state = REF_CACHE_ERR_UNIMPLEMENTED;
            res = 1;
        }
    } else if libc::strcasecmp((*parser).key, c"Connection".as_ptr()) == 0 {
        if ((*parser).flags & TRANSACT_KEEP_ALIVE) != 0 {
            let mut v = (*parser).val;
            while *v != 0 {
                let l = ref_cache_http_parser_c_99_lws_cspn(v);
                if l == 5 && libc::strncasecmp(v, c"close".as_ptr(), l) == 0 {
                    (*parser).flags &= !TRANSACT_KEEP_ALIVE;
                    break;
                }
                v = v.add(l + ref_cache_http_parser_c_93_lws_spn(v.add(l)));
            }
        }
    } else if libc::strcasecmp((*parser).key, c"User-Agent".as_ptr()) == 0 {
        if (*parser).val_used != 0 {
            libc::free((*parser).user_agent.cast());
            (*parser).user_agent = ref_cache_misc_h_91_lim_strdup(
                (*parser).val,
                (*parser).val_used,
                REF_CACHE_MAX_UA_LEN,
            );
        }
    } else if libc::strcasecmp((*parser).key, c"Referer".as_ptr()) == 0 {
        if (*parser).val_used != 0 {
            libc::free((*parser).referrer.cast());
            (*parser).referrer = ref_cache_misc_h_91_lim_strdup(
                (*parser).val,
                (*parser).val_used,
                REF_CACHE_MAX_REFERRER_LEN,
            );
        }
    } else if libc::strcasecmp((*parser).key, c"Range".as_ptr()) == 0 && (*parser).val_used != 0 {
        ref_cache_http_parser_c_333_parse_range(parser.cast());
    }

    (*parser).key_used = 0;
    *(*parser).key = 0;
    (*parser).val_used = 0;
    if !(*parser).val.is_null() {
        *(*parser).val = 0;
    }
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

        let keylen = ref_cache_http_parser_c_105_tok_spn(line);
        let mut spaces = ref_cache_http_parser_c_93_lws_spn(line.add(keylen));
        if keylen > 0 {
            if *line.add(keylen + spaces) != b':' as c_char {
                (*parser_l).state = REF_CACHE_ERR_BAD_REQUEST;
                return;
            }
            spaces += 1 + ref_cache_http_parser_c_93_lws_spn(line.add(keylen + spaces + 1));

            if (*parser_l).key_used != 0
                && ref_cache_http_parser_c_394_parser_parse_header(parser) != 0
            {
                return;
            }

            if (*parser_l).key_sz < keylen + 1 {
                if !(*parser_l).key.is_null() {
                    libc::free((*parser_l).key.cast());
                    (*parser_l).key_sz = 0;
                }
                (*parser_l).key = libc::malloc(keylen + 1).cast();
                if (*parser_l).key.is_null() {
                    (*parser_l).state = REF_CACHE_ERR_INTERNAL;
                    return;
                }
                (*parser_l).key_sz = keylen + 1;
            }
            libc::memcpy((*parser_l).key.cast(), line.cast(), keylen);
            *(*parser_l).key.add(keylen) = 0;
            (*parser_l).key_used = keylen;
            (*parser_l).val_used = 0;
        } else if spaces == 0 {
            (*parser_l).state = REF_CACHE_ERR_BAD_REQUEST;
            return;
        }

        let valpos = keylen + spaces;
        if *line.add(valpos) != 0 {
            if (*parser_l).val_used + len - valpos + 2 > (*parser_l).val_sz {
                let mut new_sz = if (*parser_l).val_sz != 0 {
                    (*parser_l).val_sz * 2
                } else {
                    64
                };
                while new_sz < (*parser_l).val_used + len - valpos + 2 {
                    new_sz *= 2;
                }
                let new_val = libc::realloc((*parser_l).val.cast(), new_sz).cast::<c_char>();
                if new_val.is_null() {
                    (*parser_l).state = REF_CACHE_ERR_INTERNAL;
                    return;
                }
                (*parser_l).val = new_val;
                (*parser_l).val_sz = new_sz;
            }
            if (*parser_l).val_used > 0 {
                *(*parser_l).val.add((*parser_l).val_used) = b' ' as c_char;
                (*parser_l).val_used += 1;
            }
            assert!(!(*parser_l).val.is_null());
            libc::memcpy(
                (*parser_l).val.add((*parser_l).val_used).cast(),
                line.add(valpos).cast(),
                len - valpos,
            );
            (*parser_l).val_used += len - valpos;
            *(*parser_l).val.add((*parser_l).val_used) = 0;
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

// original: steal_user_agent_from_parser (htslib/ref_cache/http_parser.c:637)
pub unsafe fn ref_cache_http_parser_c_637_steal_user_agent_from_parser(
    parser: *mut Http_Parser,
) -> *mut c_char {
    let parser = parser.cast::<HttpParserLayout>();
    let ua = (*parser).user_agent;
    (*parser).user_agent = std::ptr::null_mut();
    ua
}

// original: steal_referrer_from_parser (htslib/ref_cache/http_parser.c:643)
pub unsafe fn ref_cache_http_parser_c_643_steal_referrer_from_parser(
    parser: *mut Http_Parser,
) -> *mut c_char {
    let parser = parser.cast::<HttpParserLayout>();
    let referrer = (*parser).referrer;
    (*parser).referrer = std::ptr::null_mut();
    referrer
}

// original: Http_Parser (htslib/ref_cache/http_parser.h:87)
#[repr(C)]
pub struct Http_Parser {
    _private: [u8; 0],
}
