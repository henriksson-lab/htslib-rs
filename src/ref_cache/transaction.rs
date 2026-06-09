use super::http_parser::Http_Parser;
use super::ref_files::{
    ref_cache_ref_files_c_145_get_ref_size, ref_cache_ref_files_c_149_get_ref_available,
    ref_cache_ref_files_c_153_get_ref_id, ref_cache_ref_files_c_157_get_ref_complete,
    ref_cache_ref_files_c_161_get_ref_fd, ref_cache_ref_files_c_165_update_ref_download_started,
    ref_cache_ref_files_c_174_update_ref_available,
    ref_cache_ref_files_c_179_update_ref_with_content_len,
    ref_cache_ref_files_c_185_set_ref_complete, ref_cache_ref_files_c_193_release_ref_file,
    RefFile,
};
use super::sendfile_wrap::ref_cache_sendfile_wrap;
use super::server::{
    ref_cache_server_c_395_queue_transaction_write, ref_cache_server_c_840_client_host, Client,
};
use crate::htslib_rs::cram;
use std::ffi::{c_char, c_int, c_uint, c_ulong};
use std::ptr::NonNull;

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

fn ref_cache_errno_is_would_block(errno: c_int) -> bool {
    errno == libc::EAGAIN || (libc::EWOULDBLOCK != libc::EAGAIN && errno == libc::EWOULDBLOCK)
}

fn ref_cache_errno_is_transient_write(errno: c_int) -> bool {
    ref_cache_errno_is_would_block(errno) || errno == libc::EINTR
}

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
struct TransactionPrefixLayout {
    state: c_int,
    flags: c_uint,
    next: *mut Transaction,
    next_id: *mut Transaction,
    client: *mut Client,
    user_agent: *mut c_char,
    referrer: *mut c_char,
    req_str: *mut c_char,
    text: *mut c_char,
    range_from: libc::off_t,
    range_to: libc::off_t,
    sz: usize,
    out: usize,
    ref_: *mut RefFile,
    fd_sz: libc::off_t,
    fd_sent: libc::off_t,
    rc: c_uint,
    http_vers: c_int,
    user_agent_buf: Vec<u8>,
    referrer_buf: Vec<u8>,
    req_str_buf: Vec<u8>,
    text_buf: Vec<u8>,
}

// Concurrency note (audit 2026-05):
//
// Both statics belong to the `ref-cache` daemon server worker, which is
// single-threaded by construction (fork-based process model, epoll event
// loop — see `src/ref_cache/main.rs` and `src/ref_cache/server.rs`). The
// transaction pool is lazily created on the first call to
// `ref_cache_transaction_c_136_new_transaction`; because that call runs on
// the daemon's single owning thread, no synchronization is required.
// `REF_CACHE_TRANSACTIONS` is a fixed-size hash bucket array indexed by
// `id & REF_CACHE_TRANSACT_MASK` and is also touched only from that thread.
// `grep -rn 'pthread_create\|thread::spawn' src/ref_cache/` returns no
// matches; do not add additional threads here without revisiting this.
//
// SAFETY: single-threaded daemon worker.
static mut REF_CACHE_TRANSACTION_POOL: Option<Box<cram::pool_alloc_t>> = None;
static mut REF_CACHE_TRANSACTIONS: [*mut Transaction; 0x400] = [std::ptr::null_mut(); 0x400];

// original: Transaction (htslib/ref_cache/transaction.c:70)
#[repr(C)]
pub struct Transaction {
    _private: [u8; 0],
}

fn raw_ptr_from_vec(buf: &mut Vec<u8>) -> *mut c_char {
    if buf.is_empty() {
        std::ptr::null_mut()
    } else {
        buf.as_mut_ptr().cast()
    }
}

unsafe fn c_bytes<'a>(ptr: *const c_char) -> &'a [u8] {
    if ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(ptr.cast::<u8>(), libc::strlen(ptr))
    }
}

unsafe fn take_parser_c_buf(buf: &mut Vec<u8>, raw: &mut *mut c_char) -> Vec<u8> {
    *raw = std::ptr::null_mut();
    std::mem::take(buf)
}

unsafe fn clear_owned_text(transact: &mut TransactionPrefixLayout) {
    transact.text_buf.clear();
    transact.text = std::ptr::null_mut();
    transact.sz = 0;
    transact.out = 0;
    transact.flags &= !TRANSACT_TEXT_CONST;
}

unsafe fn set_owned_text(transact: &mut TransactionPrefixLayout, text: Vec<u8>) {
    transact.text_buf = text;
    transact.text = raw_ptr_from_vec(&mut transact.text_buf);
    transact.sz = transact.text_buf.len();
    transact.out = 0;
    transact.flags &= !TRANSACT_TEXT_CONST;
}

unsafe fn set_static_text(transact: &mut TransactionPrefixLayout, text: &'static [u8]) {
    transact.text_buf.clear();
    transact.text = text.as_ptr().cast::<c_char>().cast_mut();
    transact.sz = text.len();
    transact.out = 0;
    transact.flags |= TRANSACT_TEXT_CONST;
}

fn http_vers_bytes(http_vers: c_int) -> Option<&'static [u8]> {
    match http_vers {
        HTTP_1_0 => Some(b"HTTP/1.0"),
        HTTP_1_1 => Some(b"HTTP/1.1"),
        _ => None,
    }
}

unsafe fn ref_cache_transaction_pool() -> Option<&'static mut cram::pool_alloc_t> {
    let pool_slot = std::ptr::addr_of_mut!(REF_CACHE_TRANSACTION_POOL);
    if (*pool_slot).is_none() {
        *pool_slot = Some(cram::pool_create(std::mem::size_of::<
            TransactionPrefixLayout,
        >()));
    }
    (*pool_slot).as_deref_mut()
}

// original: new_transaction (htslib/ref_cache/transaction.c:136)
pub unsafe fn ref_cache_transaction_c_136_new_transaction(
    client: *mut Client,
    parser: *mut Http_Parser,
) -> *mut Transaction {
    let Some(pool) = ref_cache_transaction_pool() else {
        return std::ptr::null_mut();
    };
    let Some(transact) = cram::pool_alloc(pool).map(NonNull::<u8>::cast::<TransactionPrefixLayout>)
    else {
        return std::ptr::null_mut();
    };
    let transact = transact.as_ptr();

    let parser_layout = parser.cast::<HttpParserLayout>();
    let mut user_agent_buf = take_parser_c_buf(
        &mut (*parser_layout).user_agent_buf,
        &mut (*parser_layout).user_agent,
    );
    let mut referrer_buf = take_parser_c_buf(
        &mut (*parser_layout).referrer_buf,
        &mut (*parser_layout).referrer,
    );
    let user_agent = raw_ptr_from_vec(&mut user_agent_buf);
    let referrer = raw_ptr_from_vec(&mut referrer_buf);

    std::ptr::write(
        transact,
        TransactionPrefixLayout {
            state: 0,
            flags: (*parser_layout).flags
                & (TRANSACT_KEEP_ALIVE
                    | TRANSACT_RANGE_FROM
                    | TRANSACT_RANGE_TO
                    | TRANSACT_RANGE_SUFFIX),
            next: std::ptr::null_mut(),
            next_id: std::ptr::null_mut(),
            client,
            user_agent,
            referrer,
            req_str: std::ptr::null_mut(),
            text: std::ptr::null_mut(),
            range_from: (*parser_layout).range_from,
            range_to: (*parser_layout).range_to,
            sz: 0,
            out: 0,
            ref_: std::ptr::null_mut(),
            fd_sz: 0,
            fd_sent: 0,
            rc: 0,
            http_vers: (*parser_layout).http_vers,
            user_agent_buf,
            referrer_buf,
            req_str_buf: Vec::new(),
            text_buf: Vec::new(),
        },
    );

    transact.cast()
}

// original: transaction_clear_ref (htslib/ref_cache/transaction.c:166)
pub unsafe fn ref_cache_transaction_c_166_transaction_clear_ref(transact: *mut Transaction) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    if (*transact_layout).ref_.is_null() {
        return;
    }
    let id = ref_cache_ref_files_c_153_get_ref_id((*transact_layout).ref_);
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
    ref_cache_ref_files_c_193_release_ref_file((*transact_layout).ref_);
    (*transact_layout).ref_ = std::ptr::null_mut();
    (*transact_layout).next_id = std::ptr::null_mut();
}

// original: free_transaction (htslib/ref_cache/transaction.c:181)
pub unsafe fn ref_cache_transaction_c_181_free_transaction(transact: *mut Transaction) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    ref_cache_transaction_c_166_transaction_clear_ref(transact);
    std::ptr::drop_in_place(transact_layout);
    let pool_slot = std::ptr::addr_of_mut!(REF_CACHE_TRANSACTION_POOL);
    if let (Some(pool), Some(transact)) = (
        (*pool_slot).as_deref_mut(),
        NonNull::new(transact.cast::<u8>()),
    ) {
        cram::pool_free(pool, transact);
    }
}

// original: free_transaction_list (htslib/ref_cache/transaction.c:196)
pub unsafe fn ref_cache_transaction_c_196_free_transaction_list(mut head: *mut Transaction) {
    while !head.is_null() {
        let next = (*(head.cast::<TransactionPrefixLayout>())).next;
        ref_cache_transaction_c_181_free_transaction(head);
        head = next;
    }
}

// original: switch_to_next_transaction (htslib/ref_cache/transaction.c:204)
pub unsafe fn ref_cache_transaction_c_204_switch_to_next_transaction(
    transact: *mut Transaction,
) -> *mut Transaction {
    let next = (*(transact.cast::<TransactionPrefixLayout>())).next;
    ref_cache_transaction_c_181_free_transaction(transact);
    next
}

// original: transaction_get_client (htslib/ref_cache/transaction.c:210)
pub unsafe fn ref_cache_transaction_c_210_transaction_get_client(
    transact: *mut Transaction,
) -> *mut Client {
    (*(transact.cast::<TransactionPrefixLayout>())).client
}

// original: transaction_get_keep_alive (htslib/ref_cache/transaction.c:214)
pub unsafe fn ref_cache_transaction_c_214_transaction_get_keep_alive(
    transact: *mut Transaction,
) -> c_int {
    (((*(transact.cast::<TransactionPrefixLayout>())).flags & TRANSACT_KEEP_ALIVE) != 0) as c_int
}

// original: transaction_set_ref (htslib/ref_cache/transaction.c:218)
pub unsafe fn ref_cache_transaction_c_218_transaction_set_ref(
    transact: *mut Transaction,
    ref_: *mut RefFile,
) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    (*transact_layout).ref_ = ref_;
    let id = ref_cache_ref_files_c_153_get_ref_id(ref_);
    let slot = (id & REF_CACHE_TRANSACT_MASK) as usize;
    (*transact_layout).next_id = REF_CACHE_TRANSACTIONS[slot];
    REF_CACHE_TRANSACTIONS[slot] = transact;
}

// original: calculate_range_available (htslib/ref_cache/transaction.c:225)
pub unsafe fn ref_cache_transaction_c_225_calculate_range_available(
    transact: *mut Transaction,
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
    transact: *mut Transaction,
    requested: *const c_char,
) {
    let transact = &mut *transact.cast::<TransactionPrefixLayout>();
    let requested = c_bytes(requested);
    let len = requested.len().min(REF_CACHE_MAX_REQUEST_LEN);
    transact.req_str_buf.clear();
    transact.req_str_buf.extend_from_slice(&requested[..len]);
    transact.req_str_buf.push(0);
    transact.req_str = raw_ptr_from_vec(&mut transact.req_str_buf);
}

// original: transaction_by_id (htslib/ref_cache/transaction.c:270)
pub unsafe fn ref_cache_transaction_c_270_transaction_by_id(
    id: c_uint,
    start: *mut Transaction,
) -> *mut Transaction {
    let mut r = if start.is_null() {
        REF_CACHE_TRANSACTIONS[(id & REF_CACHE_TRANSACT_MASK) as usize]
    } else {
        (*(start.cast::<TransactionPrefixLayout>())).next_id
    };
    while !r.is_null()
        && !(*(r.cast::<TransactionPrefixLayout>())).ref_.is_null()
        && ref_cache_ref_files_c_153_get_ref_id((*(r.cast::<TransactionPrefixLayout>())).ref_) != id
    {
        r = (*(r.cast::<TransactionPrefixLayout>())).next_id;
    }
    r
}

// original: set_error_response (htslib/ref_cache/transaction.c:277)
pub unsafe fn ref_cache_transaction_c_277_set_error_response(
    transact: *mut Transaction,
    code: c_uint,
) {
    let transact = transact.cast::<TransactionPrefixLayout>();

    if (*transact).state >= TRANSACT_SENDING_TEXT {
        (*transact).state = TRANSACT_FINISHED;
        (*transact).flags &= !TRANSACT_KEEP_ALIVE;
        return;
    }

    clear_owned_text(&mut *transact);

    let vers = if (*transact).http_vers == HTTP_1_1 {
        1
    } else {
        0
    };
    let text = match (code, vers) {
        (400, 0) => b"HTTP/1.0 400 Bad Request\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n400 Bad Request\r\n".as_slice(),
        (400, _) => b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n400 Bad Request\r\n",
        (404, 0) => b"HTTP/1.0 404 Not found\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 15\r\n\r\n404 Not found\r\n",
        (404, _) => b"HTTP/1.1 404 Not found\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 15\r\n\r\n404 Not found\r\n",
        (413, 0) => b"HTTP/1.0 413 Request Entity Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 30\r\n\r\n413 Request Entity Too Large\r\n",
        (413, _) => b"HTTP/1.1 413 Request Entity Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 30\r\n\r\n413 Request Entity Too Large\r\n",
        (414, 0) => b"HTTP/1.0 414 Request-URI Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n414 Request-URI Too Large\r\n",
        (414, _) => b"HTTP/1.1 414 Request-URI Too Large\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n414 Request-URI Too Large\r\n",
        (500, 0) => b"HTTP/1.0 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n",
        (500, _) => b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n",
        (501, 0) => b"HTTP/1.0 501 Not Implemented\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 21\r\n\r\n501 Not Implemented\r\n",
        (501, _) => b"HTTP/1.1 501 Not Implemented\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 21\r\n\r\n501 Not Implemented\r\n",
        (502, 0) => b"HTTP/1.0 502 Bad Gateway\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n502 Bad Gateway\r\n",
        (502, _) => b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 17\r\n\r\n502 Bad Gateway\r\n",
        (505, 0) => b"HTTP/1.0 505 HTTP Version not supported\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 32\r\n\r\n505 HTTP Version not supported\r\n",
        (505, _) => b"HTTP/1.1 505 HTTP Version not supported\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 32\r\n\r\n505 HTTP Version not supported\r\n",
        (_, 0) => b"HTTP/1.0 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n",
        _ => b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: 27\r\n\r\n500 Internal Server Error\r\n",
    };
    set_static_text(&mut *transact, text);

    (*transact).rc = code;
    ref_cache_transaction_c_166_transaction_clear_ref(transact.cast());
    (*transact).flags &= !TRANSACT_KEEP_ALIVE;
    (*transact).state = TRANSACT_GOT_TEXT;
}

// original: http_vers_string (htslib/ref_cache/transaction.c:313)
pub unsafe fn ref_cache_transaction_c_313_http_vers_string(
    transact: *mut Transaction,
) -> *const c_char {
    match (*(transact.cast::<TransactionPrefixLayout>())).http_vers {
        HTTP_1_0 => c"HTTP/1.0".as_ptr(),
        HTTP_1_1 => c"HTTP/1.1".as_ptr(),
        _ => std::ptr::null(),
    }
}

// original: set_ref_file_response (htslib/ref_cache/transaction.c:322)
pub unsafe fn ref_cache_transaction_c_322_set_ref_file_response(
    transact: *mut Transaction,
    len: i64,
    range_start: libc::off_t,
    range_end: libc::off_t,
) -> c_int {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    let Some(vers) = http_vers_bytes((*transact_layout).http_vers) else {
        ref_cache_transaction_c_277_set_error_response(transact, 505);
        return -1;
    };
    let mut keep_alive = ((*transact_layout).flags & TRANSACT_KEEP_ALIVE) != 0
        && (*transact_layout).http_vers == HTTP_1_1;
    let mut rc = 200u32;

    assert!((*transact_layout).state < TRANSACT_SENDING_TEXT);

    clear_owned_text(&mut *transact_layout);

    let text = if len != 0 {
        if range_start >= 0 {
            assert!(range_end > range_start && range_end > 0);
            rc = 206;
            format!(
                "{} 206 Partial content\r\nContent-Type: text/plain\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\n{}\r\n",
                std::str::from_utf8_unchecked(vers),
                range_start,
                range_end - 1,
                len,
                range_end - range_start,
                if keep_alive { "" } else { "Connection: close\r\n" },
            )
        } else {
            format!(
                "{} 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n",
                std::str::from_utf8_unchecked(vers),
                len,
                if keep_alive {
                    ""
                } else {
                    "Connection: close\r\n"
                },
            )
        }
    } else {
        keep_alive = false;
        format!(
            "{} 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
            std::str::from_utf8_unchecked(vers),
        )
    };

    (*transact_layout).rc = rc;
    set_owned_text(&mut *transact_layout, text.into_bytes());
    (*transact_layout).state = TRANSACT_GOT_TEXT;

    if !keep_alive {
        (*transact_layout).flags &= !TRANSACT_KEEP_ALIVE;
    }
    0
}

// original: set_message_response (htslib/ref_cache/transaction.c:408)
pub unsafe fn ref_cache_transaction_c_408_set_message_response(
    transact: *mut Transaction,
    content_type: *const c_char,
    message: *const c_char,
    len: usize,
) {
    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    let Some(vers) = http_vers_bytes((*transact_layout).http_vers) else {
        ref_cache_transaction_c_277_set_error_response(transact, 505);
        return;
    };
    let keep_alive = ((*transact_layout).flags & TRANSACT_KEEP_ALIVE) != 0
        && (*transact_layout).http_vers == HTTP_1_1;

    assert!((*transact_layout).state < TRANSACT_SENDING_TEXT);

    clear_owned_text(&mut *transact_layout);
    let mut text = format!(
        "{} 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}\r\n",
        std::str::from_utf8_unchecked(vers),
        std::str::from_utf8_unchecked(c_bytes(content_type)),
        len,
        if keep_alive {
            ""
        } else {
            "Connection: close\r\n"
        },
    )
    .into_bytes();
    text.extend_from_slice(std::slice::from_raw_parts(message.cast::<u8>(), len));

    (*transact_layout).rc = 200;
    set_owned_text(&mut *transact_layout, text);
    (*transact_layout).state = TRANSACT_GOT_TEXT;

    if !keep_alive {
        (*transact_layout).flags &= !TRANSACT_KEEP_ALIVE;
    }
}

// original: send_file (htslib/ref_cache/transaction.c:460)
pub unsafe fn ref_cache_transaction_c_460_send_file(
    transact: *mut Transaction,
    out_fd: c_int,
    in_fd: c_int,
    end: libc::off_t,
) -> libc::ssize_t {
    let transact = transact.cast::<TransactionPrefixLayout>();
    assert!(end >= (*transact).fd_sent);
    ref_cache_sendfile_wrap(
        out_fd,
        in_fd,
        &mut (*transact).fd_sent,
        (end - (*transact).fd_sent) as usize,
    )
}

// original: transaction_send_data (htslib/ref_cache/transaction.c:556)
pub unsafe fn ref_cache_transaction_c_556_transaction_send_data(
    transact: *mut Transaction,
    fd: c_int,
) -> c_int {
    let transact = transact.cast::<TransactionPrefixLayout>();

    if (*transact).state == TRANSACT_GOT_TEXT {
        (*transact).state = TRANSACT_SENDING_TEXT;
    }

    if (*transact).state == TRANSACT_SENDING_TEXT {
        assert!(!(*transact).text.is_null());
        let bytes = libc::write(
            fd,
            (*transact).text.cast(),
            (*transact).sz - (*transact).out,
        );
        if bytes < 0 {
            let errno = *crate::htslib_rs::c_compat::__errno_location();
            if !ref_cache_errno_is_transient_write(errno) {
                libc::fprintf(
                    crate::htslib_rs::ref_cache::compat::stderr(),
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

    match (*transact).state {
        TRANSACT_SENDING_FILE => {
            assert!(!(*transact).ref_.is_null());
            let ref_fd = ref_cache_ref_files_c_161_get_ref_fd((*transact).ref_);
            let available = ref_cache_ref_files_c_149_get_ref_available((*transact).ref_);
            assert!(ref_fd >= 0);

            let mut end = if ref_cache_ref_files_c_145_get_ref_size((*transact).ref_) > 0 {
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
                let errno = *crate::htslib_rs::c_compat::__errno_location();
                if !ref_cache_errno_is_would_block(errno) {
                    libc::fprintf(
                        crate::htslib_rs::ref_cache::compat::stderr(),
                        c"sendfile fd #%d : %s\n".as_ptr(),
                        fd,
                        libc::strerror(errno),
                    );
                    return REF_CACHE_WRITE_ERROR;
                }
                return REF_CACHE_WRITE_BLOCKED;
            }
            if (*transact).fd_sent < end
                || ref_cache_ref_files_c_157_get_ref_complete((*transact).ref_) == 0
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
                crate::htslib_rs::ref_cache::compat::stderr(),
                c"resp_send_data entered when transaction in wrong state (%d)\n".as_ptr(),
                (*transact).state,
            );
            libc::abort();
        }
    }
}

// original: transaction_have_content (htslib/ref_cache/transaction.c:619)
pub unsafe fn ref_cache_transaction_c_619_transaction_have_content(
    transact: *mut Transaction,
) -> c_int {
    ((*(transact.cast::<TransactionPrefixLayout>())).state > TRANSACT_WAITING_TEXT) as c_int
}

// original: transaction_set_next (htslib/ref_cache/transaction.c:623)
pub unsafe fn ref_cache_transaction_c_623_transaction_set_next(
    transact: *mut Transaction,
    next: *mut Transaction,
) {
    let transact = transact.cast::<TransactionPrefixLayout>();
    assert!((*transact).next.is_null());
    (*transact).next = next;
}

// original: transaction_has_data_to_send (htslib/ref_cache/transaction.c:628)
pub unsafe fn ref_cache_transaction_c_628_transaction_has_data_to_send(
    transact: *mut Transaction,
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
            let available = ref_cache_ref_files_c_149_get_ref_available((*transact).ref_);
            let mut end = if ref_cache_ref_files_c_145_get_ref_size((*transact).ref_) > 0 {
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
    transact: *mut Transaction,
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
            ref_cache_ref_files_c_145_get_ref_size((*transact_layout).ref_)
        };
        (*transact_layout).fd_sent = if range_start >= 0 { range_start } else { 0 };
    }
}

// original: update_with_initial_size (htslib/ref_cache/transaction.c:671)
pub unsafe fn ref_cache_transaction_c_671_update_with_initial_size(
    mut transact: *mut Transaction,
    size: i64,
    ref_id: c_uint,
    write_stack: *mut *mut Client,
) {
    while !transact.is_null() {
        ref_cache_transaction_c_654_set_transaction_file_range(transact, size, 1);
        ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(ref_id, transact);
    }
}

// original: got_download_started (htslib/ref_cache/transaction.c:680)
pub unsafe fn ref_cache_transaction_c_680_got_download_started(
    id: c_uint,
    val: i64,
    fd: c_int,
    write_stack: *mut *mut Client,
) {
    let transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    assert!(!(*transact_layout).ref_.is_null());
    ref_cache_ref_files_c_165_update_ref_download_started((*transact_layout).ref_, fd, val);

    if val >= 0 {
        ref_cache_transaction_c_671_update_with_initial_size(transact, val, id, write_stack);
    }
}

// original: got_download_part (htslib/ref_cache/transaction.c:698)
pub unsafe fn ref_cache_transaction_c_698_got_download_part(
    id: c_uint,
    val: i64,
    write_stack: *mut *mut Client,
) {
    assert!(val >= 0);

    let mut transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    assert!(!(*(transact.cast::<TransactionPrefixLayout>()))
        .ref_
        .is_null());
    ref_cache_ref_files_c_174_update_ref_available(
        (*(transact.cast::<TransactionPrefixLayout>())).ref_,
        val,
    );

    while !transact.is_null() {
        ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(id, transact);
    }
}

// original: got_download_clen (htslib/ref_cache/transaction.c:715)
pub unsafe fn ref_cache_transaction_c_715_got_download_clen(
    id: c_uint,
    val: i64,
    write_stack: *mut *mut Client,
) {
    assert!(val >= 0);

    let transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    let transact_layout = transact.cast::<TransactionPrefixLayout>();
    assert!(!(*transact_layout).ref_.is_null());
    ref_cache_ref_files_c_179_update_ref_with_content_len((*transact_layout).ref_, val);

    ref_cache_transaction_c_671_update_with_initial_size(transact, val, id, write_stack);
}

// original: got_download_result (htslib/ref_cache/transaction.c:730)
pub unsafe fn ref_cache_transaction_c_730_got_download_result(
    id: c_uint,
    val: i64,
    write_stack: *mut *mut Client,
) {
    assert!((0..1000).contains(&val));

    let mut transact = ref_cache_transaction_c_270_transaction_by_id(id, std::ptr::null_mut());
    if transact.is_null() {
        return;
    }

    if val != 200 {
        while !transact.is_null() {
            ref_cache_transaction_c_277_set_error_response(transact, val as c_uint);
            ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
            transact = ref_cache_transaction_c_270_transaction_by_id(id, transact);
        }
        return;
    }

    assert!(!(*(transact.cast::<TransactionPrefixLayout>()))
        .ref_
        .is_null());

    let no_initial_content_length = ref_cache_ref_files_c_185_set_ref_complete(
        (*(transact.cast::<TransactionPrefixLayout>())).ref_,
    );
    let available = ref_cache_ref_files_c_149_get_ref_available(
        (*(transact.cast::<TransactionPrefixLayout>())).ref_,
    );

    while !transact.is_null() {
        let t = transact.cast::<TransactionPrefixLayout>();
        if no_initial_content_length != 0 || (*t).fd_sz > available {
            (*t).fd_sz = available;
        }
        if (*t).fd_sent >= (*t).fd_sz {
            (*t).state = TRANSACT_FINISHED;
        } else if (*t).state < TRANSACT_SENDING_TEXT {
            ref_cache_transaction_c_654_set_transaction_file_range(transact, available, 1);
        }
        ref_cache_server_c_395_queue_transaction_write(transact, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(id, transact);
    }
}

// original: make_log_message (htslib/ref_cache/transaction.c:769)
pub unsafe fn ref_cache_transaction_c_769_make_log_message(
    transact: *mut Transaction,
    buffer: *mut c_char,
    size: usize,
) -> usize {
    let transact = transact.cast::<TransactionPrefixLayout>();
    let mut timestamp = [0 as c_char; 32];
    let t = libc::time(std::ptr::null_mut());
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (year, month, day, hour, minute, second, _) =
        crate::htslib_rs::c_compat::unix_time_utc_parts(t);
    let timestamp_text = format!(
        "{:02}/{}/{:04}:{:02}:{:02}:{:02} +0000",
        day,
        MONTHS[(month - 1) as usize],
        year,
        hour,
        minute,
        second
    );
    crate::htslib_rs::c_compat::write_c_str(&mut timestamp, &timestamp_text);
    let bytes = libc::snprintf(
        buffer,
        size,
        c"REQ %s - - %s \"%s\" %d %zu \"%s\" \"%s\"\n".as_ptr(),
        ref_cache_server_c_840_client_host((*transact).client),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errno_predicates_treat_nonblocking_aliases_as_transient() {
        assert!(ref_cache_errno_is_would_block(libc::EAGAIN));
        assert!(ref_cache_errno_is_would_block(libc::EWOULDBLOCK));
        assert!(ref_cache_errno_is_transient_write(libc::EAGAIN));
        assert!(ref_cache_errno_is_transient_write(libc::EWOULDBLOCK));
        assert!(ref_cache_errno_is_transient_write(libc::EINTR));
        assert!(!ref_cache_errno_is_would_block(libc::EBADF));
        assert!(!ref_cache_errno_is_transient_write(libc::EBADF));
    }
}
