use crate::htslib_mini_rs::cram;
use crate::original_stubs::{functions, structs};
use std::ffi::{c_char, c_int, c_uint, c_ulong};

const TRANSACT_KEEP_ALIVE: c_uint = 2;
const TRANSACT_TEXT_CONST: c_uint = 1;
const TRANSACT_WAITING_TEXT: c_int = 0;
const TRANSACT_GOT_TEXT: c_int = 1;
const TRANSACT_SENDING_TEXT: c_int = 2;
const TRANSACT_SENDING_FILE: c_int = 3;
const TRANSACT_FINISHED: c_int = 4;
const TRANSACT_RANGE_FROM: c_uint = 8;
const TRANSACT_RANGE_TO: c_uint = 16;
const TRANSACT_RANGE_SUFFIX: c_uint = 32;
const REF_CACHE_TRANSACT_MASK: c_uint = 0x3ff;
const REF_CACHE_MAX_REQUEST_LEN: usize = 64;
const HTTP_1_0: c_int = 1;
const HTTP_1_1: c_int = 2;
const REF_CACHE_WRITE_BLOCKED: c_int = 0;
const REF_CACHE_WRITE_BLOCKED_UPSTREAM: c_int = 1;
const REF_CACHE_WRITE_COMPLETE: c_int = 3;
const REF_CACHE_WRITE_ERROR: c_int = 5;

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
struct TransactionPrefixLayout {
    state: c_int,
    flags: c_uint,
    next: *mut structs::Transaction,
    next_id: *mut structs::Transaction,
    client: *mut structs::Client,
    user_agent: *mut c_char,
    referrer: *mut c_char,
    req_str: *mut c_char,
    text: *mut c_char,
    range_from: libc::off_t,
    range_to: libc::off_t,
    sz: usize,
    out: usize,
    ref_: *mut structs::RefFile,
    fd_sz: libc::off_t,
    fd_sent: libc::off_t,
    rc: c_uint,
    http_vers: c_int,
}

static mut REF_CACHE_TRANSACTION_POOL: *mut cram::pool_alloc_t = std::ptr::null_mut();
static mut REF_CACHE_TRANSACTIONS: [*mut structs::Transaction; 0x400] =
    [std::ptr::null_mut(); 0x400];

// original: Transaction (htslib/ref_cache/transaction.c:70)
#[repr(C)]
pub struct Transaction {
    _private: [u8; 0],
}

// original: new_transaction (htslib/ref_cache/transaction.c:136)
pub unsafe fn ref_cache_transaction_c_136_new_transaction(
    client: *mut structs::Client,
    parser: *mut structs::Http_Parser,
) -> *mut structs::Transaction {
    if REF_CACHE_TRANSACTION_POOL.is_null() {
        REF_CACHE_TRANSACTION_POOL = cram::cram_pooled_alloc_c_64_pool_create(std::mem::size_of::<
            TransactionPrefixLayout,
        >());
        if REF_CACHE_TRANSACTION_POOL.is_null() {
            return std::ptr::null_mut();
        }
    }

    let transact = cram::cram_pooled_alloc_c_115_pool_alloc(REF_CACHE_TRANSACTION_POOL)
        .cast::<TransactionPrefixLayout>();
    if transact.is_null() {
        return std::ptr::null_mut();
    }
    libc::memset(
        transact.cast(),
        0,
        std::mem::size_of::<TransactionPrefixLayout>(),
    );

    let parser_layout = parser.cast::<HttpParserLayout>();
    (*transact).client = client;
    (*transact).ref_ = std::ptr::null_mut();
    (*transact).user_agent =
        functions::ref_cache_http_parser_c_637_steal_user_agent_from_parser(parser);
    (*transact).referrer =
        functions::ref_cache_http_parser_c_643_steal_referrer_from_parser(parser);
    (*transact).req_str = std::ptr::null_mut();
    (*transact).range_from = (*parser_layout).range_from;
    (*transact).range_to = (*parser_layout).range_to;
    (*transact).flags = (*parser_layout).flags
        & (TRANSACT_KEEP_ALIVE | TRANSACT_RANGE_FROM | TRANSACT_RANGE_TO | TRANSACT_RANGE_SUFFIX);
    (*transact).rc = 0;
    (*transact).next_id = std::ptr::null_mut();
    (*transact).http_vers = (*parser_layout).http_vers;

    transact.cast()
}

// original: transaction_clear_ref (htslib/ref_cache/transaction.c:166)
pub unsafe fn ref_cache_transaction_c_166_transaction_clear_ref(
    transact: *mut structs::Transaction,
) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    if (*transact_layout).ref_.is_null() {
        return;
    }
    let id = functions::ref_cache_ref_files_c_153_get_ref_id((*transact_layout).ref_);
    let slot = (id & REF_CACHE_TRANSACT_MASK) as usize;
    if REF_CACHE_TRANSACTIONS[slot] == transact {
        REF_CACHE_TRANSACTIONS[slot] = (*transact_layout).next_id;
    } else {
        let mut t = REF_CACHE_TRANSACTIONS[slot];
        while !t.is_null() && (*(t.cast::<TransactionPrefixLayout>())).next_id != transact {
            t = (*(t.cast::<TransactionPrefixLayout>())).next_id;
        }
        if !t.is_null() {
            (*(t.cast::<TransactionPrefixLayout>())).next_id = (*transact_layout).next_id;
        }
    }
    functions::ref_cache_ref_files_c_193_release_ref_file((*transact_layout).ref_);
    (*transact_layout).ref_ = std::ptr::null_mut();
    (*transact_layout).next_id = std::ptr::null_mut();
}

// original: free_transaction (htslib/ref_cache/transaction.c:181)
pub unsafe fn ref_cache_transaction_c_181_free_transaction(transact: *mut structs::Transaction) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    ref_cache_transaction_c_166_transaction_clear_ref(transact);
    if ((*transact_layout).flags & TRANSACT_TEXT_CONST) == 0 && !(*transact_layout).text.is_null() {
        libc::free((*transact_layout).text.cast());
    }
    libc::free((*transact_layout).user_agent.cast());
    libc::free((*transact_layout).referrer.cast());
    libc::free((*transact_layout).req_str.cast());
    libc::memset(
        transact_layout.cast(),
        0,
        std::mem::size_of::<TransactionPrefixLayout>(),
    );
    cram::cram_pooled_alloc_c_144_pool_free(REF_CACHE_TRANSACTION_POOL, transact.cast());
}

// original: free_transaction_list (htslib/ref_cache/transaction.c:196)
pub unsafe fn ref_cache_transaction_c_196_free_transaction_list(
    mut head: *mut structs::Transaction,
) {
    while !head.is_null() {
        let next = (*(head.cast::<TransactionPrefixLayout>())).next;
        ref_cache_transaction_c_181_free_transaction(head);
        head = next;
    }
}

// original: switch_to_next_transaction (htslib/ref_cache/transaction.c:204)
pub unsafe fn ref_cache_transaction_c_204_switch_to_next_transaction(
    transact: *mut structs::Transaction,
) -> *mut structs::Transaction {
    let next = (*(transact.cast::<TransactionPrefixLayout>())).next;
    ref_cache_transaction_c_181_free_transaction(transact);
    next
}

// original: transaction_get_client (htslib/ref_cache/transaction.c:210)
pub unsafe fn ref_cache_transaction_c_210_transaction_get_client(
    transact: *mut structs::Transaction,
) -> *mut structs::Client {
    (*(transact.cast::<TransactionPrefixLayout>())).client
}

// original: transaction_get_keep_alive (htslib/ref_cache/transaction.c:214)
pub unsafe fn ref_cache_transaction_c_214_transaction_get_keep_alive(
    transact: *mut structs::Transaction,
) -> c_int {
    (((*(transact.cast::<TransactionPrefixLayout>())).flags & TRANSACT_KEEP_ALIVE) != 0) as c_int
}

// original: transaction_set_ref (htslib/ref_cache/transaction.c:218)
pub unsafe fn ref_cache_transaction_c_218_transaction_set_ref(
    transact: *mut structs::Transaction,
    ref_: *mut structs::RefFile,
) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    (*transact_layout).ref_ = ref_;
    let id = functions::ref_cache_ref_files_c_153_get_ref_id(ref_);
    let slot = (id & REF_CACHE_TRANSACT_MASK) as usize;
    (*transact_layout).next_id = REF_CACHE_TRANSACTIONS[slot];
    REF_CACHE_TRANSACTIONS[slot] = transact;
}

// original: calculate_range_available (htslib/ref_cache/transaction.c:225)
pub unsafe fn ref_cache_transaction_c_225_calculate_range_available(
    transact: *mut structs::Transaction,
    size: libc::off_t,
    range_start_out: *mut libc::off_t,
    range_end_out: *mut libc::off_t,
) {
    let transact = transact.cast::<TransactionPrefixLayout>();
    let mut range_start: libc::off_t = -1;
    let mut range_end: libc::off_t = -1;
    let mut have_range = ((*transact).flags & (TRANSACT_RANGE_FROM | TRANSACT_RANGE_SUFFIX)) != 0;

    if have_range {
        if ((*transact).flags & TRANSACT_RANGE_SUFFIX) != 0 {
            range_end = size;
            range_start = if (*transact).range_to < range_end {
                range_end - (*transact).range_to
            } else {
                0
            };
            if range_start == 0 || (*transact).range_to == 0 {
                have_range = false;
            }
        } else if (*transact).range_from > (*transact).range_to || (*transact).range_from >= size {
            have_range = false;
        } else {
            range_start = (*transact).range_from;
            range_end = if ((*transact).flags & TRANSACT_RANGE_TO) != 0 {
                if (*transact).range_to + 1 < size {
                    (*transact).range_to + 1
                } else {
                    size
                }
            } else {
                size
            };
        }
    }
    *range_start_out = if have_range { range_start } else { -1 };
    *range_end_out = if have_range { range_end } else { -1 };
}

// original: transaction_set_req_str (htslib/ref_cache/transaction.c:264)
pub unsafe fn ref_cache_transaction_c_264_transaction_set_req_str(
    transact: *mut structs::Transaction,
    requested: *const c_char,
) {
    let len = libc::strlen(requested);
    (*(transact.cast::<TransactionPrefixLayout>())).req_str = libc::strndup(
        requested,
        if len < REF_CACHE_MAX_REQUEST_LEN {
            len
        } else {
            REF_CACHE_MAX_REQUEST_LEN
        },
    );
}

// original: transaction_by_id (htslib/ref_cache/transaction.c:270)
pub unsafe fn ref_cache_transaction_c_270_transaction_by_id(
    id: c_uint,
    start: *mut structs::Transaction,
) -> *mut structs::Transaction {
    let mut r = if start.is_null() {
        REF_CACHE_TRANSACTIONS[(id & REF_CACHE_TRANSACT_MASK) as usize]
    } else {
        (*(start.cast::<TransactionPrefixLayout>())).next_id
    };
    while !r.is_null()
        && !(*(r.cast::<TransactionPrefixLayout>())).ref_.is_null()
        && functions::ref_cache_ref_files_c_153_get_ref_id(
            (*(r.cast::<TransactionPrefixLayout>())).ref_,
        ) != id
    {
        r = (*(r.cast::<TransactionPrefixLayout>())).next_id;
    }
    r
}

// original: set_error_response (htslib/ref_cache/transaction.c:277)
pub unsafe fn ref_cache_transaction_c_277_set_error_response(
    transact: *mut structs::Transaction,
    code: c_uint,
) {
    let transact = transact.cast::<TransactionPrefixLayout>();

    if (*transact).state >= TRANSACT_SENDING_TEXT {
        (*transact).state = TRANSACT_FINISHED;
        (*transact).flags &= !TRANSACT_KEEP_ALIVE;
        return;
    }

    if ((*transact).flags & TRANSACT_TEXT_CONST) == 0 && !(*transact).text.is_null() {
        libc::free((*transact).text.cast());
    }

    let vers = if (*transact).http_vers == HTTP_1_1 {
        1
    } else {
        0
    };
    (*transact).text = match (code, vers) {
            (400, 0) => c"HTTP/1.0 400 Bad Request\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n400 Bad Request\r\n".as_ptr().cast_mut(),
            (400, _) => c"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n400 Bad Request\r\n".as_ptr().cast_mut(),
            (404, 0) => c"HTTP/1.0 404 Not found\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 15\r\n\r\n404 Not found\r\n".as_ptr().cast_mut(),
            (404, _) => c"HTTP/1.1 404 Not found\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 15\r\n\r\n404 Not found\r\n".as_ptr().cast_mut(),
            (413, 0) => c"HTTP/1.0 413 Request Entity Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 30\r\n\r\n413 Request Entity Too Large\r\n".as_ptr().cast_mut(),
            (413, _) => c"HTTP/1.1 413 Request Entity Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 30\r\n\r\n413 Request Entity Too Large\r\n".as_ptr().cast_mut(),
            (414, 0) => c"HTTP/1.0 414 Request-URI Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n414 Request-URI Too Large\r\n".as_ptr().cast_mut(),
            (414, _) => c"HTTP/1.1 414 Request-URI Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n414 Request-URI Too Large\r\n".as_ptr().cast_mut(),
            (500, 0) => c"HTTP/1.0 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n".as_ptr().cast_mut(),
            (500, _) => c"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n".as_ptr().cast_mut(),
            (501, 0) => c"HTTP/1.0 501 Not Implemented\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 21\r\n\r\n501 Not Implemented\r\n".as_ptr().cast_mut(),
            (501, _) => c"HTTP/1.1 501 Not Implemented\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 21\r\n\r\n501 Not Implemented\r\n".as_ptr().cast_mut(),
            (502, 0) => c"HTTP/1.0 502 Bad Gateway\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n502 Bad Gateway\r\n".as_ptr().cast_mut(),
            (502, _) => c"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n502 Bad Gateway\r\n".as_ptr().cast_mut(),
            (505, 0) => c"HTTP/1.0 505 HTTP Version not supported\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 32\r\n\r\n505 HTTP Version not supported\r\n".as_ptr().cast_mut(),
            (505, _) => c"HTTP/1.1 505 HTTP Version not supported\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 32\r\n\r\n505 HTTP Version not supported\r\n".as_ptr().cast_mut(),
            (_, 0) => c"HTTP/1.0 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n".as_ptr().cast_mut(),
            _ => c"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n".as_ptr().cast_mut(),
        };

    (*transact).rc = code;
    (*transact).sz = libc::strlen((*transact).text);
    ref_cache_transaction_c_166_transaction_clear_ref(transact.cast());
    (*transact).flags &= !TRANSACT_KEEP_ALIVE;
    (*transact).flags |= TRANSACT_TEXT_CONST;
    (*transact).state = TRANSACT_GOT_TEXT;
}

// original: http_vers_string (htslib/ref_cache/transaction.c:313)
pub unsafe fn ref_cache_transaction_c_313_http_vers_string(
    transact: *mut structs::Transaction,
) -> *const c_char {
    match (*(transact.cast::<TransactionPrefixLayout>())).http_vers {
        HTTP_1_0 => c"HTTP/1.0".as_ptr(),
        HTTP_1_1 => c"HTTP/1.1".as_ptr(),
        _ => std::ptr::null(),
    }
}

// original: set_ref_file_response (htslib/ref_cache/transaction.c:322)
pub unsafe fn ref_cache_transaction_c_322_set_ref_file_response(
    transact: *mut structs::Transaction,
    len: i64,
    range_start: libc::off_t,
    range_end: libc::off_t,
) -> c_int {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    let mut text_len = 128usize;
    let vers = ref_cache_transaction_c_313_http_vers_string(transact);
    let content_type = c"text/plain".as_ptr();
    let mut keep_alive = ((*transact_layout).flags & TRANSACT_KEEP_ALIVE) != 0
        && (*transact_layout).http_vers == HTTP_1_1;
    let mut rc = 200u32;

    assert!((*transact_layout).state < TRANSACT_SENDING_TEXT);

    if vers.is_null() {
        ref_cache_transaction_c_277_set_error_response(transact, 505);
        return -1;
    }

    if ((*transact_layout).flags & TRANSACT_TEXT_CONST) == 0 && !(*transact_layout).text.is_null() {
        libc::free((*transact_layout).text.cast());
        (*transact_layout).text = std::ptr::null_mut();
    }

    if range_start >= 0 {
        text_len += 100;
    }

    let text = libc::malloc(text_len).cast::<c_char>();
    if text.is_null() {
        ref_cache_transaction_c_277_set_error_response(transact, 500);
        return -1;
    }

    let l = if len != 0 {
        if range_start >= 0 {
            assert!(range_end > range_start && range_end > 0);
            rc = 206;
            libc::snprintf(
                    text,
                    text_len,
                    c"%s 206 Partial content\r\nContent-Type: %s\r\nContent-Range: bytes %lld-%lld/%lld\r\nContent-Length: %lld\r\n%s\r\n".as_ptr(),
                    vers,
                    content_type,
                    range_start as libc::c_longlong,
                    (range_end - 1) as libc::c_longlong,
                    len as libc::c_longlong,
                    (range_end - range_start) as libc::c_longlong,
                    if keep_alive { c"".as_ptr() } else { c"Connection: close\r\n".as_ptr() },
                )
        } else {
            libc::snprintf(
                text,
                text_len,
                c"%s 200 OK\r\nContent-Type: %s\r\nContent-Length: %lld\r\n%s\r\n".as_ptr(),
                vers,
                content_type,
                len as libc::c_longlong,
                if keep_alive {
                    c"".as_ptr()
                } else {
                    c"Connection: close\r\n".as_ptr()
                },
            )
        }
    } else {
        keep_alive = false;
        libc::snprintf(
            text,
            text_len,
            c"%s 200 OK\r\nContent-Type: %s\r\nConnection: close\r\n\r\n".as_ptr(),
            vers,
            content_type,
        )
    };

    if l < 0 || l as usize > text_len {
        libc::free(text.cast());
        ref_cache_transaction_c_277_set_error_response(transact, 500);
        return -1;
    }

    (*transact_layout).rc = rc;
    (*transact_layout).text = text;
    (*transact_layout).sz = l as usize;
    (*transact_layout).out = 0;
    (*transact_layout).flags &= !TRANSACT_TEXT_CONST;
    (*transact_layout).state = TRANSACT_GOT_TEXT;

    if !keep_alive {
        (*transact_layout).flags &= !TRANSACT_KEEP_ALIVE;
    }
    0
}

// original: set_message_response (htslib/ref_cache/transaction.c:408)
pub unsafe fn ref_cache_transaction_c_408_set_message_response(
    transact: *mut structs::Transaction,
    content_type: *const c_char,
    message: *const c_char,
    len: usize,
) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    let text_len = 128usize + len;
    let vers = ref_cache_transaction_c_313_http_vers_string(transact);
    let keep_alive = ((*transact_layout).flags & TRANSACT_KEEP_ALIVE) != 0
        && (*transact_layout).http_vers == HTTP_1_1;

    assert!((*transact_layout).state < TRANSACT_SENDING_TEXT);

    if vers.is_null() {
        ref_cache_transaction_c_277_set_error_response(transact, 505);
        return;
    }

    if ((*transact_layout).flags & TRANSACT_TEXT_CONST) == 0 && !(*transact_layout).text.is_null() {
        libc::free((*transact_layout).text.cast());
        (*transact_layout).text = std::ptr::null_mut();
    }

    let text = libc::malloc(text_len).cast::<c_char>();
    if text.is_null() {
        ref_cache_transaction_c_277_set_error_response(transact, 500);
        return;
    }

    let l = libc::snprintf(
        text,
        text_len,
        c"%s 200 OK\r\nContent-Type: %s\r\nContent-Length: %zu\r\n%s\r\n".as_ptr(),
        vers,
        content_type,
        len,
        if keep_alive {
            c"".as_ptr()
        } else {
            c"Connection: close\r\n".as_ptr()
        },
    );

    if l < 0 || l as usize > text_len || text_len - (l as usize) < len {
        libc::free(text.cast());
        ref_cache_transaction_c_277_set_error_response(transact, 500);
        return;
    }

    libc::memcpy(text.add(l as usize).cast(), message.cast(), len);

    (*transact_layout).rc = 200;
    (*transact_layout).text = text;
    (*transact_layout).sz = l as usize + len;
    (*transact_layout).out = 0;
    (*transact_layout).flags &= !TRANSACT_TEXT_CONST;
    (*transact_layout).state = TRANSACT_GOT_TEXT;

    if !keep_alive {
        (*transact_layout).flags &= !TRANSACT_KEEP_ALIVE;
    }
}

// original: send_file (htslib/ref_cache/transaction.c:460)
pub unsafe fn ref_cache_transaction_c_460_send_file(
    transact: *mut structs::Transaction,
    out_fd: c_int,
    in_fd: c_int,
    end: libc::off_t,
) -> libc::ssize_t {
    let transact = transact.cast::<TransactionPrefixLayout>();
    assert!(end >= (*transact).fd_sent);
    functions::ref_cache_sendfile_wrap_c_55_sendfile_wrap(
        out_fd,
        in_fd,
        &mut (*transact).fd_sent,
        (end - (*transact).fd_sent) as usize,
    )
}

// original: transaction_send_data (htslib/ref_cache/transaction.c:556)
pub unsafe fn ref_cache_transaction_c_556_transaction_send_data(
    transact: *mut structs::Transaction,
    fd: c_int,
) -> c_int {
    let transact = transact.cast::<TransactionPrefixLayout>();

    match (*transact).state {
        TRANSACT_GOT_TEXT => {
            (*transact).state = TRANSACT_SENDING_TEXT;
        }
        _ => {}
    }

    match (*transact).state {
        TRANSACT_SENDING_TEXT => {
            assert!(!(*transact).text.is_null());
            let bytes = libc::write(
                fd,
                (*transact).text.cast(),
                (*transact).sz - (*transact).out,
            );
            if bytes < 0 {
                let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
                if errno != libc::EAGAIN || errno != libc::EWOULDBLOCK || errno != libc::EINTR {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"Error from fd #%d : %s\n".as_ptr(),
                        fd,
                        libc::strerror(errno),
                    );
                    return REF_CACHE_WRITE_ERROR;
                }
                return REF_CACHE_WRITE_BLOCKED;
            }
            (*transact).out += bytes as usize;
            if (*transact).out < (*transact).sz {
                return REF_CACHE_WRITE_BLOCKED;
            }
            if (*transact).ref_.is_null() {
                (*transact).state = TRANSACT_FINISHED;
                return REF_CACHE_WRITE_COMPLETE;
            }
            (*transact).state = TRANSACT_SENDING_FILE;
        }
        _ => {}
    }

    match (*transact).state {
        TRANSACT_SENDING_FILE => {
            assert!(!(*transact).ref_.is_null());
            let ref_fd = functions::ref_cache_ref_files_c_161_get_ref_fd((*transact).ref_);
            let available =
                functions::ref_cache_ref_files_c_149_get_ref_available((*transact).ref_);
            assert!(ref_fd >= 0);

            let mut end = if functions::ref_cache_ref_files_c_145_get_ref_size((*transact).ref_) > 0
            {
                (*transact).fd_sz
            } else {
                available
            };

            if available <= (*transact).fd_sent {
                return REF_CACHE_WRITE_BLOCKED_UPSTREAM;
            }
            if available < end {
                end = available;
            }
            let sent = ref_cache_transaction_c_460_send_file(transact.cast(), fd, ref_fd, end);
            if sent < 0 {
                let errno = *crate::htslib_mini_rs::c_compat::__errno_location();
                if errno != libc::EAGAIN || errno != libc::EWOULDBLOCK {
                    libc::fprintf(
                        hts_sys::stderr.cast(),
                        c"sendfile fd #%d : %s\n".as_ptr(),
                        fd,
                        libc::strerror(errno),
                    );
                    return REF_CACHE_WRITE_ERROR;
                }
                return REF_CACHE_WRITE_BLOCKED;
            }
            if (*transact).fd_sent < end
                || functions::ref_cache_ref_files_c_157_get_ref_complete((*transact).ref_) == 0
            {
                if (*transact).fd_sent == end {
                    REF_CACHE_WRITE_BLOCKED_UPSTREAM
                } else {
                    REF_CACHE_WRITE_BLOCKED
                }
            } else {
                REF_CACHE_WRITE_COMPLETE
            }
        }
        TRANSACT_FINISHED => REF_CACHE_WRITE_COMPLETE,
        _ => {
            libc::fprintf(
                hts_sys::stderr.cast(),
                c"resp_send_data entered when transaction in wrong state (%d)\n".as_ptr(),
                (*transact).state,
            );
            libc::abort();
        }
    }
}

// original: transaction_have_content (htslib/ref_cache/transaction.c:619)
pub unsafe fn ref_cache_transaction_c_619_transaction_have_content(
    transact: *mut structs::Transaction,
) -> c_int {
    ((*(transact.cast::<TransactionPrefixLayout>())).state > TRANSACT_WAITING_TEXT) as c_int
}

// original: transaction_set_next (htslib/ref_cache/transaction.c:623)
pub unsafe fn ref_cache_transaction_c_623_transaction_set_next(
    transact: *mut structs::Transaction,
    next: *mut structs::Transaction,
) {
    let transact = transact.cast::<TransactionPrefixLayout>();
    assert!((*transact).next.is_null());
    (*transact).next = next;
}

// original: transaction_has_data_to_send (htslib/ref_cache/transaction.c:628)
pub unsafe fn ref_cache_transaction_c_628_transaction_has_data_to_send(
    transact: *mut structs::Transaction,
) -> c_int {
    let transact = transact.cast::<TransactionPrefixLayout>();
    let mut have_data = 0;

    match (*transact).state {
        TRANSACT_WAITING_TEXT => {}
        TRANSACT_GOT_TEXT | TRANSACT_SENDING_TEXT => {
            if (*transact).out < (*transact).sz {
                have_data = 1;
            }
        }
        TRANSACT_SENDING_FILE => {
            let available =
                functions::ref_cache_ref_files_c_149_get_ref_available((*transact).ref_);
            let mut end = if functions::ref_cache_ref_files_c_145_get_ref_size((*transact).ref_) > 0
            {
                (*transact).fd_sz
            } else {
                available
            };
            if available < end {
                end = available;
            }
            if (*transact).fd_sent < end {
                have_data = 1;
            }
        }
        TRANSACT_FINISHED => have_data = 1,
        _ => libc::abort(),
    }
    have_data
}

// original: set_transaction_file_range (htslib/ref_cache/transaction.c:654)
pub unsafe fn ref_cache_transaction_c_654_set_transaction_file_range(
    transact: *mut structs::Transaction,
    size: i64,
    ref_data_available: c_int,
) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    let mut res = 0;
    let mut range_start: libc::off_t = -1;
    let mut range_end: libc::off_t = -1;

    if size > 0 {
        ref_cache_transaction_c_225_calculate_range_available(
            transact,
            size as libc::off_t,
            &mut range_start,
            &mut range_end,
        );
    }
    if ref_data_available != 0 {
        res = ref_cache_transaction_c_322_set_ref_file_response(
            transact,
            size,
            range_start,
            range_end,
        );
    }
    if res == 0 {
        (*transact_layout).fd_sz = if range_end >= 0 {
            range_end
        } else {
            functions::ref_cache_ref_files_c_145_get_ref_size((*transact_layout).ref_)
        };
        (*transact_layout).fd_sent = if range_start >= 0 { range_start } else { 0 };
    }
}

// original: update_with_initial_size (htslib/ref_cache/transaction.c:671)
pub unsafe fn ref_cache_transaction_c_671_update_with_initial_size(
    mut transact: *mut structs::Transaction,
    size: i64,
    ref_id: c_uint,
    write_stack: *mut *mut structs::Client,
) {
    while !transact.is_null() {
        ref_cache_transaction_c_654_set_transaction_file_range(transact, size, 1);
        functions::ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(ref_id, transact);
    }
}

// original: got_download_started (htslib/ref_cache/transaction.c:680)
pub unsafe fn ref_cache_transaction_c_680_got_download_started(
    id: c_uint,
    val: i64,
    fd: c_int,
    write_stack: *mut *mut structs::Client,
) {
    let transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    assert!(!(*transact_layout).ref_.is_null());
    functions::ref_cache_ref_files_c_165_update_ref_download_started(
        (*transact_layout).ref_,
        fd,
        val,
    );

    if val >= 0 {
        ref_cache_transaction_c_671_update_with_initial_size(transact, val, id, write_stack);
    }
}

// original: got_download_part (htslib/ref_cache/transaction.c:698)
pub unsafe fn ref_cache_transaction_c_698_got_download_part(
    id: c_uint,
    val: i64,
    write_stack: *mut *mut structs::Client,
) {
    assert!(val >= 0);

    let mut transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    assert!(!(*(transact.cast::<TransactionPrefixLayout>()))
        .ref_
        .is_null());
    functions::ref_cache_ref_files_c_174_update_ref_available(
        (*(transact.cast::<TransactionPrefixLayout>())).ref_,
        val,
    );

    while !transact.is_null() {
        functions::ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(id, transact);
    }
}

// original: got_download_clen (htslib/ref_cache/transaction.c:715)
pub unsafe fn ref_cache_transaction_c_715_got_download_clen(
    id: c_uint,
    val: i64,
    write_stack: *mut *mut structs::Client,
) {
    assert!(val >= 0);

    let transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    assert!(!(*transact_layout).ref_.is_null());
    functions::ref_cache_ref_files_c_179_update_ref_with_content_len((*transact_layout).ref_, val);

    ref_cache_transaction_c_671_update_with_initial_size(transact, val, id, write_stack);
}

// original: got_download_result (htslib/ref_cache/transaction.c:730)
pub unsafe fn ref_cache_transaction_c_730_got_download_result(
    id: c_uint,
    val: i64,
    write_stack: *mut *mut structs::Client,
) {
    assert!(val >= 0 && val < 1000);

    let mut transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    if val != 200 {
        while !transact.is_null() {
            ref_cache_transaction_c_277_set_error_response(transact, val as c_uint);
            functions::ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
            transact = ref_cache_transaction_c_270_transaction_by_id(id, transact);
        }
        return;
    }

    assert!(!(*(transact.cast::<TransactionPrefixLayout>()))
        .ref_
        .is_null());

    let no_initial_content_length = functions::ref_cache_ref_files_c_185_set_ref_complete(
        (*(transact.cast::<TransactionPrefixLayout>())).ref_,
    );
    let available = functions::ref_cache_ref_files_c_149_get_ref_available(
        (*(transact.cast::<TransactionPrefixLayout>())).ref_,
    );

    while !transact.is_null() {
        let t = transact.cast::<TransactionPrefixLayout>();
        if no_initial_content_length != 0 {
            (*t).fd_sz = available;
        } else if (*t).fd_sz > available {
            (*t).fd_sz = available;
        }
        if (*t).fd_sent >= (*t).fd_sz {
            (*t).state = TRANSACT_FINISHED;
        } else if (*t).state < TRANSACT_SENDING_TEXT {
            ref_cache_transaction_c_654_set_transaction_file_range(transact, available, 1);
        }
        functions::ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(id, transact);
    }
}

// original: make_log_message (htslib/ref_cache/transaction.c:769)
pub unsafe fn ref_cache_transaction_c_769_make_log_message(
    transact: *mut structs::Transaction,
    buffer: *mut c_char,
    size: usize,
) -> usize {
    let transact = transact.cast::<TransactionPrefixLayout>();
    let mut timestamp = [0 as c_char; 32];
    let t = libc::time(std::ptr::null_mut());
    libc::strftime(
        timestamp.as_mut_ptr(),
        timestamp.len(),
        c"%d/%b/%Y:%H:%M:%S +0000".as_ptr(),
        libc::gmtime(&t),
    );
    let bytes = libc::snprintf(
        buffer,
        size,
        c"REQ %s - - %s \"%s\" %d %zu \"%s\" \"%s\"\n".as_ptr(),
        functions::ref_cache_server_c_840_client_host((*transact).client),
        timestamp.as_ptr(),
        if !(*transact).req_str.is_null() {
            (*transact).req_str
        } else {
            c"".as_ptr().cast_mut()
        },
        (*transact).rc as c_int,
        (*transact).out + (*transact).fd_sent as usize,
        if !(*transact).user_agent.is_null() {
            (*transact).user_agent
        } else {
            c"".as_ptr().cast_mut()
        },
        if !(*transact).referrer.is_null() {
            (*transact).referrer
        } else {
            c"".as_ptr().cast_mut()
        },
    );
    bytes as usize
}
