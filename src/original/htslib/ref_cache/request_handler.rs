use super::misc::ref_cache_misc_h_38_hexval;
use crate::original_stubs::{functions, structs};
use std::ffi::{c_char, c_int, c_uint, c_ulong};

const REF_DOWNLOAD_STARTED: c_int = 2;
const REF_NOT_FOUND: c_int = 1;
const TRANSACT_KEEP_ALIVE: c_uint = 2;
const HTTP_1_1: c_int = 2;
const REF_CACHE_READING_REQUEST_LINE: c_int = 0;
const REF_CACHE_SHUTTING_DOWN: c_int = 7;
const REF_CACHE_ERR_BAD_REQUEST: c_int = 400;
const REF_CACHE_ERR_NOT_FOUND: c_int = 404;
const REF_CACHE_ERR_INTERNAL: c_int = 500;
const REF_CACHE_ERR_UNIMPLEMENTED: c_int = 501;
const REF_CACHE_REQ_GET: c_int = 1;

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

static TEXT_PLAIN: &[u8] = b"text/plain\0";

// original: is_hexmd5 (htslib/ref_cache/request_handler.c:49)
pub unsafe fn ref_cache_request_handler_c_49_is_hexmd5(str_: *mut c_char) -> c_int {
    for i in 0..32 {
        if ref_cache_misc_h_38_hexval(*str_.add(i)) == -1 {
            return 0;
        }
    }
    if ref_cache_misc_h_38_hexval(*str_.add(32)) == -1 {
        1
    } else {
        0
    }
}

// original: decode_uri (htslib/ref_cache/request_handler.c:56)
pub unsafe fn ref_cache_request_handler_c_56_decode_uri(
    parser: *mut structs::Http_Parser,
) -> *mut c_char {
    let parser = parser.cast::<HttpParserLayout>();
    let mut uri = (*parser).uri;
    let mut querypart: *mut c_char;
    let mut out: *mut c_char;
    let mut last: c_char;
    let mut in_: *const c_char;
    let mut d1: c_int;
    let mut d2: c_int;

    if uri.is_null() {
        return std::ptr::null_mut();
    }

    /* Deal with absolute URLs */
    if libc::strncasecmp(uri, c"http://".as_ptr(), 7) == 0 {
        let localpart = libc::strchr(uri.add(7), b'/' as c_int);
        if localpart.is_null() {
            return std::ptr::null_mut();
        }
        uri = localpart;
    }

    /* Should always start with / now */
    if *uri != b'/' as c_char {
        return std::ptr::null_mut();
    }

    /* Hack off query part */
    querypart = libc::strchr(uri, b'?' as c_int);
    if !querypart.is_null() {
        *querypart = b'\0' as c_char;
    }

    /* Deal with multiple // and % decoding. URI will always shrink. */
    last = b'\0' as c_char;
    in_ = uri;
    out = uri;
    while *in_ != 0 {
        if *in_ == b'/' as c_char && last == b'/' as c_char {
            in_ = in_.add(1);
            continue;
        }
        if *in_ == b'%' as c_char {
            d1 = ref_cache_misc_h_38_hexval(*in_.add(1));
            d2 = ref_cache_misc_h_38_hexval(*in_.add(2));
            if d1 >= 0 && d2 >= 0 {
                let v = (d1 << 4 | d2) as c_char;
                in_ = in_.add(3);
                if v == b'/' as c_char && last == b'/' as c_char {
                    continue;
                }
                *out = v;
                last = v;
                out = out.add(1);
                continue;
            }
        }
        *out = *in_;
        last = *in_;
        out = out.add(1);
        in_ = in_.add(1);
    }
    *out = b'\0' as c_char;
    uri
}

// original: handle_hello (htslib/ref_cache/request_handler.c:94)
pub unsafe fn ref_cache_request_handler_c_94_handle_hello(transact: *mut structs::Transaction) {
    let resp = c"Hello\r\n".as_ptr();
    functions::ref_cache_transaction_c_408_set_message_response(
        transact,
        TEXT_PLAIN.as_ptr().cast(),
        resp,
        libc::strlen(resp),
    );
}

// original: handle_md5 (htslib/ref_cache/request_handler.c:99)
pub unsafe fn ref_cache_request_handler_c_99_handle_md5(
    opts: *const structs::Options,
    parser: *mut structs::Http_Parser,
    transact: *mut structs::Transaction,
    md5: *mut c_char,
) {
    // off_t range_start = -1, range_end = 0;
    // int have_range = 0;
    let parser = parser.cast::<HttpParserLayout>();
    let ref_file = functions::ref_cache_ref_files_c_94_get_ref_file(opts, md5, (*parser).upstream);
    let mut status: c_int;
    let mut size: libc::off_t;

    if ref_file.is_null() {
        functions::ref_cache_transaction_c_277_set_error_response(
            transact,
            REF_CACHE_ERR_INTERNAL as c_uint,
        );
        return;
    }

    status = functions::ref_cache_ref_files_c_141_get_ref_status(ref_file);
    if status == REF_NOT_FOUND {
        functions::ref_cache_transaction_c_277_set_error_response(
            transact,
            REF_CACHE_ERR_NOT_FOUND as c_uint,
        );
        functions::ref_cache_ref_files_c_193_release_ref_file(ref_file);
        return;
    }

    size = functions::ref_cache_ref_files_c_145_get_ref_size(ref_file);

    functions::ref_cache_transaction_c_218_transaction_set_ref(transact, ref_file);

    functions::ref_cache_transaction_c_654_set_transaction_file_range(
        transact,
        size,
        (status >= REF_DOWNLOAD_STARTED) as c_int,
    );
}

// original: handle_get (htslib/ref_cache/request_handler.c:126)
pub unsafe fn ref_cache_request_handler_c_126_handle_get(
    opts: *const structs::Options,
    parser: *mut structs::Http_Parser,
    transact: *mut structs::Transaction,
) {
    let mut requested = ref_cache_request_handler_c_56_decode_uri(parser);

    if (*(opts.cast::<RefCacheOptionsLayout>())).verbosity > 1 {
        libc::fprintf(
            hts_sys::stderr.cast(),
            c"Request: GET %s\n".as_ptr(),
            if requested.is_null() {
                c"".as_ptr()
            } else {
                requested
            },
        );
    }

    functions::ref_cache_transaction_c_264_transaction_set_req_str(
        transact,
        if requested.is_null() {
            c"".as_ptr()
        } else {
            requested
        },
    );

    if requested.is_null() || *requested != b'/' as c_char {
        functions::ref_cache_transaction_c_277_set_error_response(
            transact,
            REF_CACHE_ERR_BAD_REQUEST as c_uint,
        );
        return;
    }
    requested = requested.add(1);
    if ref_cache_request_handler_c_49_is_hexmd5(requested) != 0
        && *requested.add(32) == b'\0' as c_char
    {
        ref_cache_request_handler_c_99_handle_md5(opts, parser, transact, requested);
        return;
    }
    if libc::strcmp(requested, c"hello".as_ptr()) == 0 {
        ref_cache_request_handler_c_94_handle_hello(transact);
        return;
    }
    functions::ref_cache_transaction_c_277_set_error_response(
        transact,
        REF_CACHE_ERR_NOT_FOUND as c_uint,
    );
}

// original: handle_request (htslib/ref_cache/request_handler.c:148)
pub unsafe fn ref_cache_request_handler_c_148_handle_request(
    opts: *const structs::Options,
    client: *mut structs::Client,
    parser: *mut structs::Http_Parser,
    transact_out: *mut *mut structs::Transaction,
) {
    let transact = functions::ref_cache_transaction_c_136_new_transaction(client, parser);
    let parser = parser.cast::<HttpParserLayout>();
    if transact.is_null() {
        (*parser).state = REF_CACHE_ERR_INTERNAL;
        return;
    }

    match (*parser).req_type {
        REF_CACHE_REQ_GET => {
            ref_cache_request_handler_c_126_handle_get(opts, parser.cast(), transact);
        }
        /* case REQ_HEAD: handle_head(parser, transact); break; */
        _ => {
            functions::ref_cache_transaction_c_277_set_error_response(
                transact,
                REF_CACHE_ERR_UNIMPLEMENTED as c_uint,
            );
        }
    }
    *transact_out = transact;

    if ((*parser).flags & TRANSACT_KEEP_ALIVE) != 0 && (*parser).http_vers == HTTP_1_1 {
        (*parser).state = REF_CACHE_READING_REQUEST_LINE;
    } else {
        (*parser).state = REF_CACHE_SHUTTING_DOWN;
    }
}

// original: handle_error (htslib/ref_cache/request_handler.c:167)
pub unsafe fn ref_cache_request_handler_c_167_handle_error(
    client: *mut structs::Client,
    parser: *mut structs::Http_Parser,
    code: c_int,
    transact_out: *mut *mut structs::Transaction,
) {
    let transact = functions::ref_cache_transaction_c_136_new_transaction(client, parser);
    let parser = parser.cast::<HttpParserLayout>();
    if transact.is_null() {
        (*parser).state = REF_CACHE_ERR_INTERNAL;
        return;
    }
    functions::ref_cache_transaction_c_277_set_error_response(
        transact,
        if code >= REF_CACHE_ERR_BAD_REQUEST {
            code as c_uint
        } else {
            500
        },
    );
    *transact_out = transact;
}
