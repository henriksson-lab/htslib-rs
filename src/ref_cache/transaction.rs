use super::http_parser::HttpParser;
use super::ref_files::{
    ref_cache_ref_files_c_145_get_ref_size, ref_cache_ref_files_c_149_get_ref_available,
    ref_cache_ref_files_c_153_get_ref_id, ref_cache_ref_files_c_157_get_ref_complete,
    ref_cache_ref_files_c_161_get_ref_fd, ref_cache_ref_files_c_165_update_ref_download_started,
    ref_cache_ref_files_c_174_update_ref_available,
    ref_cache_ref_files_c_179_update_ref_with_content_len,
    ref_cache_ref_files_c_185_set_ref_complete, ref_cache_ref_files_c_193_release_ref_file,
};
use super::sendfile_wrap::ref_cache_sendfile_wrap;
use super::server::{ref_cache_server_c_395_queue_transaction_write, RefCacheClientsLayout};
use std::ffi::{c_int, c_uint};

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

// original: Transaction (htslib/ref_cache/transaction.c:70)
//
// The C code allocated transactions out of a pooled allocator and threaded
// them through two intrusive raw-pointer lists: `next` (the per-client request
// pipeline) and `next_id` (a hash bucket keyed by ref id). Both lists, plus the
// pool, have been restructured into a single owned arena (`TRANSACTION_ARENA`)
// of `Option<Box<Transaction>>` cells indexed by `TransactionId` (a usize
// slot). What used to be `*mut Transaction` is now a `TransactionId`; the
// `next`/`next_id` links are `Option<TransactionId>` indices into the arena.
//
// Concurrency note (audit 2026-05):
//
// The arena and hash bucket array belong to the `ref-cache` daemon server
// worker, which is single-threaded by construction (fork-based process model,
// epoll event loop — see `src/ref_cache/main.rs` and `src/ref_cache/server.rs`).
// `grep -rn 'pthread_create\|thread::spawn' src/ref_cache/` returns no matches;
// do not add additional threads here without revisiting this.
//
// SAFETY: single-threaded daemon worker.
pub struct Transaction {
    state: c_int,
    flags: c_uint,
    next: Option<TransactionId>,
    next_id: Option<TransactionId>,
    // Id of the owning client (an index into the server's client arena); the
    // server module identifies clients by this id rather than by pointer.
    client: Option<usize>,
    user_agent: Vec<u8>,
    referrer: Vec<u8>,
    req_str: Vec<u8>,
    text: Vec<u8>,
    range_from: libc::off_t,
    range_to: libc::off_t,
    out: usize,
    // Id of the associated cached reference (an index into `ref_files`); `None`
    // when the transaction has no backing file. The ref_files module identifies
    // refs by this id rather than by pointer.
    ref_: Option<usize>,
    fd_sz: libc::off_t,
    fd_sent: libc::off_t,
    rc: c_uint,
    http_vers: c_int,
}

/// Index of a live transaction in `TRANSACTION_ARENA`.
pub type TransactionId = usize;

// The owning arena of live transactions. Each occupied slot holds a boxed
// `Transaction`; freeing a transaction sets its slot to `None` and the slot is
// recycled by the next allocation.
static mut TRANSACTION_ARENA: Vec<Option<Box<Transaction>>> = Vec::new();
// Hash bucket array: each bucket is the head `TransactionId` of an intrusive
// `next_id` list, indexed by `ref_id & REF_CACHE_TRANSACT_MASK`.
static mut REF_CACHE_TRANSACTIONS: [Option<TransactionId>; 0x400] = [None; 0x400];

// `txn!(id)` borrows the live transaction in arena slot `id`; `arena!()` and
// `buckets!()` borrow the global owning arena and the per-ref hash bucket array.
// These are macros (not functions) so they expand inline and the new-helper
// prohibition is respected while keeping the call sites readable.
macro_rules! arena {
    () => {
        (&mut *std::ptr::addr_of_mut!(TRANSACTION_ARENA))
    };
}
macro_rules! buckets {
    () => {
        (&mut *std::ptr::addr_of_mut!(REF_CACHE_TRANSACTIONS))
    };
}
macro_rules! txn {
    ($id:expr) => {
        arena!()[$id].as_mut().expect("transaction slot occupied")
    };
}

impl Transaction {
    fn clear_owned_text(&mut self) {
        self.text.clear();
        self.out = 0;
        self.flags &= !TRANSACT_TEXT_CONST;
    }

    fn set_owned_text(&mut self, text: Vec<u8>) {
        self.text = text;
        self.out = 0;
        self.flags &= !TRANSACT_TEXT_CONST;
    }

    // The C code distinguished static-storage response text (no free needed)
    // from heap text via TRANSACT_TEXT_CONST. With owned `Vec<u8>`, drop handles
    // both uniformly; we still set the flag bit for observable flag-state parity.
    fn set_static_text(&mut self, text: &'static [u8]) {
        self.text = text.to_vec();
        self.out = 0;
        self.flags |= TRANSACT_TEXT_CONST;
    }

    // Number of response-text bytes to emit (former `sz` field, now derived).
    fn text_len(&self) -> usize {
        self.text.len()
    }
}

fn http_vers_bytes(http_vers: c_int) -> Option<&'static [u8]> {
    match http_vers {
        HTTP_1_0 => Some(b"HTTP/1.0"),
        HTTP_1_1 => Some(b"HTTP/1.1"),
        _ => None,
    }
}

// original: new_transaction (htslib/ref_cache/transaction.c:136)
pub unsafe fn ref_cache_transaction_c_136_new_transaction(
    client: Option<usize>,
    parser: &mut HttpParser,
) -> Option<TransactionId> {
    let user_agent = parser.take_user_agent();
    let referrer = parser.take_referrer();

    let transaction = Box::new(Transaction {
        state: 0,
        flags: parser.flags()
            & (TRANSACT_KEEP_ALIVE
                | TRANSACT_RANGE_FROM
                | TRANSACT_RANGE_TO
                | TRANSACT_RANGE_SUFFIX),
        next: None,
        next_id: None,
        user_agent,
        referrer,
        req_str: Vec::new(),
        text: Vec::new(),
        range_from: parser.range_from(),
        range_to: parser.range_to(),
        out: 0,
        client,
        ref_: None,
        fd_sz: 0,
        fd_sent: 0,
        rc: 0,
        http_vers: parser.http_vers(),
    });

    let arena = arena!();
    if let Some(slot) = arena.iter().position(|s| s.is_none()) {
        arena[slot] = Some(transaction);
        Some(slot)
    } else {
        arena.push(Some(transaction));
        Some(arena.len() - 1)
    }
}

// original: transaction_clear_ref (htslib/ref_cache/transaction.c:166)
pub unsafe fn ref_cache_transaction_c_166_transaction_clear_ref(transact: TransactionId) {
    let Some(ref_) = txn!(transact).ref_ else {
        return;
    };
    let id = ref_cache_ref_files_c_153_get_ref_id(ref_);
    let slot = (id & REF_CACHE_TRANSACT_MASK) as usize;
    let next_id = txn!(transact).next_id;
    if buckets!()[slot] == Some(transact) {
        buckets!()[slot] = next_id;
    } else {
        let mut t = buckets!()[slot];
        while let Some(cur) = t {
            if txn!(cur).next_id == Some(transact) {
                break;
            }
            t = txn!(cur).next_id;
        }
        if let Some(cur) = t {
            txn!(cur).next_id = next_id;
        }
    }
    ref_cache_ref_files_c_193_release_ref_file(ref_);
    txn!(transact).ref_ = None;
    txn!(transact).next_id = None;
}

// original: free_transaction (htslib/ref_cache/transaction.c:181)
pub unsafe fn ref_cache_transaction_c_181_free_transaction(transact: TransactionId) {
    ref_cache_transaction_c_166_transaction_clear_ref(transact);
    // Dropping the boxed transaction frees its owned buffers; the freed slot is
    // recycled by the next allocation.
    arena!()[transact] = None;
}

// original: free_transaction_list (htslib/ref_cache/transaction.c:196)
pub unsafe fn ref_cache_transaction_c_196_free_transaction_list(mut head: Option<TransactionId>) {
    while let Some(cur) = head {
        let next = txn!(cur).next;
        ref_cache_transaction_c_181_free_transaction(cur);
        head = next;
    }
}

// original: switch_to_next_transaction (htslib/ref_cache/transaction.c:204)
pub unsafe fn ref_cache_transaction_c_204_switch_to_next_transaction(
    transact: TransactionId,
) -> Option<TransactionId> {
    let next = txn!(transact).next;
    ref_cache_transaction_c_181_free_transaction(transact);
    next
}

// original: transaction_get_client (htslib/ref_cache/transaction.c:210)
pub unsafe fn ref_cache_transaction_c_210_transaction_get_client(
    transact: TransactionId,
) -> Option<usize> {
    txn!(transact).client
}

// original: transaction_get_keep_alive (htslib/ref_cache/transaction.c:214)
pub unsafe fn ref_cache_transaction_c_214_transaction_get_keep_alive(
    transact: TransactionId,
) -> c_int {
    ((txn!(transact).flags & TRANSACT_KEEP_ALIVE) != 0) as c_int
}

// original: transaction_set_ref (htslib/ref_cache/transaction.c:218)
pub unsafe fn ref_cache_transaction_c_218_transaction_set_ref(
    transact: TransactionId,
    ref_: usize,
) {
    let id = ref_cache_ref_files_c_153_get_ref_id(ref_);
    txn!(transact).ref_ = Some(ref_);
    let slot = (id & REF_CACHE_TRANSACT_MASK) as usize;
    txn!(transact).next_id = buckets!()[slot];
    buckets!()[slot] = Some(transact);
}

// original: calculate_range_available (htslib/ref_cache/transaction.c:225)
pub unsafe fn ref_cache_transaction_c_225_calculate_range_available(
    transact: TransactionId,
    size: libc::off_t,
    range_start_out: &mut libc::off_t,
    range_end_out: &mut libc::off_t,
) {
    let transact = txn!(transact);
    let mut range_start: libc::off_t = -1;
    let mut range_end: libc::off_t = -1;
    let mut have_range = (transact.flags & (TRANSACT_RANGE_FROM | TRANSACT_RANGE_SUFFIX)) != 0;

    if have_range {
        if (transact.flags & TRANSACT_RANGE_SUFFIX) != 0 {
            range_end = size;
            range_start = if transact.range_to < range_end {
                range_end - transact.range_to
            } else {
                0
            };
            if range_start == 0 || transact.range_to == 0 {
                have_range = false;
            }
        } else if transact.range_from > transact.range_to || transact.range_from >= size {
            have_range = false;
        } else {
            range_start = transact.range_from;
            range_end = if (transact.flags & TRANSACT_RANGE_TO) != 0 {
                if transact.range_to + 1 < size {
                    transact.range_to + 1
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
    transact: TransactionId,
    requested: &[u8],
) {
    let transact = txn!(transact);
    let len = requested.len().min(REF_CACHE_MAX_REQUEST_LEN);
    transact.req_str.clear();
    transact.req_str.extend_from_slice(&requested[..len]);
}

// original: transaction_by_id (htslib/ref_cache/transaction.c:270)
pub unsafe fn ref_cache_transaction_c_270_transaction_by_id(
    id: c_uint,
    start: Option<TransactionId>,
) -> Option<TransactionId> {
    let mut r = match start {
        None => buckets!()[(id & REF_CACHE_TRANSACT_MASK) as usize],
        Some(start) => txn!(start).next_id,
    };
    while let Some(cur) = r {
        let Some(ref_) = txn!(cur).ref_ else {
            break;
        };
        if ref_cache_ref_files_c_153_get_ref_id(ref_) == id {
            break;
        }
        r = txn!(cur).next_id;
    }
    r
}

// original: set_error_response (htslib/ref_cache/transaction.c:277)
pub unsafe fn ref_cache_transaction_c_277_set_error_response(transact: TransactionId, code: c_uint) {
    if txn!(transact).state >= TRANSACT_SENDING_TEXT {
        let t = txn!(transact);
        t.state = TRANSACT_FINISHED;
        t.flags &= !TRANSACT_KEEP_ALIVE;
        return;
    }

    txn!(transact).clear_owned_text();

    let vers = if txn!(transact).http_vers == HTTP_1_1 {
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
    txn!(transact).set_static_text(text);

    txn!(transact).rc = code;
    ref_cache_transaction_c_166_transaction_clear_ref(transact);
    let t = txn!(transact);
    t.flags &= !TRANSACT_KEEP_ALIVE;
    t.state = TRANSACT_GOT_TEXT;
}

// original: http_vers_string (htslib/ref_cache/transaction.c:313)
pub unsafe fn ref_cache_transaction_c_313_http_vers_string(
    transact: TransactionId,
) -> Option<&'static [u8]> {
    http_vers_bytes(txn!(transact).http_vers)
}

// original: set_ref_file_response (htslib/ref_cache/transaction.c:322)
pub unsafe fn ref_cache_transaction_c_322_set_ref_file_response(
    transact: TransactionId,
    len: i64,
    range_start: libc::off_t,
    range_end: libc::off_t,
) -> c_int {
    let Some(vers) = http_vers_bytes(txn!(transact).http_vers) else {
        ref_cache_transaction_c_277_set_error_response(transact, 505);
        return -1;
    };
    let mut keep_alive =
        (txn!(transact).flags & TRANSACT_KEEP_ALIVE) != 0 && txn!(transact).http_vers == HTTP_1_1;
    let mut rc = 200u32;

    assert!(txn!(transact).state < TRANSACT_SENDING_TEXT);

    txn!(transact).clear_owned_text();

    let vers = std::str::from_utf8(vers).unwrap();
    let text = if len != 0 {
        if range_start >= 0 {
            assert!(range_end > range_start && range_end > 0);
            rc = 206;
            format!(
                "{} 206 Partial content\r\nContent-Type: text/plain\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\n{}\r\n",
                vers,
                range_start,
                range_end - 1,
                len,
                range_end - range_start,
                if keep_alive { "" } else { "Connection: close\r\n" },
            )
        } else {
            format!(
                "{} 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n",
                vers,
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
        format!("{} 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n", vers)
    };

    let t = txn!(transact);
    t.rc = rc;
    t.set_owned_text(text.into_bytes());
    t.state = TRANSACT_GOT_TEXT;

    if !keep_alive {
        t.flags &= !TRANSACT_KEEP_ALIVE;
    }
    0
}

// original: set_message_response (htslib/ref_cache/transaction.c:408)
pub unsafe fn ref_cache_transaction_c_408_set_message_response(
    transact: TransactionId,
    content_type: &[u8],
    message: &[u8],
) {
    let Some(vers) = http_vers_bytes(txn!(transact).http_vers) else {
        ref_cache_transaction_c_277_set_error_response(transact, 505);
        return;
    };
    let keep_alive =
        (txn!(transact).flags & TRANSACT_KEEP_ALIVE) != 0 && txn!(transact).http_vers == HTTP_1_1;

    assert!(txn!(transact).state < TRANSACT_SENDING_TEXT);

    txn!(transact).clear_owned_text();
    let mut text = format!(
        "{} 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}\r\n",
        std::str::from_utf8(vers).unwrap(),
        String::from_utf8_lossy(content_type),
        message.len(),
        if keep_alive {
            ""
        } else {
            "Connection: close\r\n"
        },
    )
    .into_bytes();
    text.extend_from_slice(message);

    let t = txn!(transact);
    t.rc = 200;
    t.set_owned_text(text);
    t.state = TRANSACT_GOT_TEXT;

    if !keep_alive {
        t.flags &= !TRANSACT_KEEP_ALIVE;
    }
}

// original: send_file (htslib/ref_cache/transaction.c:460)
pub unsafe fn ref_cache_transaction_c_460_send_file(
    transact: TransactionId,
    out_fd: c_int,
    in_fd: c_int,
    end: libc::off_t,
) -> libc::ssize_t {
    let transact = txn!(transact);
    assert!(end >= transact.fd_sent);
    let count = (end - transact.fd_sent) as usize;
    ref_cache_sendfile_wrap(out_fd, in_fd, Some(&mut transact.fd_sent), count)
}

// original: transaction_send_data (htslib/ref_cache/transaction.c:556)
pub unsafe fn ref_cache_transaction_c_556_transaction_send_data(
    transact: TransactionId,
    fd: c_int,
) -> c_int {
    if txn!(transact).state == TRANSACT_GOT_TEXT {
        txn!(transact).state = TRANSACT_SENDING_TEXT;
    }

    if txn!(transact).state == TRANSACT_SENDING_TEXT {
        let t = txn!(transact);
        // Single write syscall boundary: pass the owned slice's pointer/length.
        let remaining = &t.text[t.out..];
        let bytes = libc::write(fd, remaining.as_ptr().cast(), remaining.len());
        if bytes < 0 {
            let errno = *crate::htslib_rs::c_compat::__errno_location();
            if !ref_cache_errno_is_transient_write(errno) {
                eprintln!(
                    "Error from fd #{} : {}",
                    fd,
                    std::io::Error::from_raw_os_error(errno)
                );
                return REF_CACHE_WRITE_ERROR;
            }
            return REF_CACHE_WRITE_BLOCKED;
        }
        t.out += bytes as usize;
        if t.out < t.text_len() {
            return REF_CACHE_WRITE_BLOCKED;
        }
        if t.ref_.is_none() {
            t.state = TRANSACT_FINISHED;
            return REF_CACHE_WRITE_COMPLETE;
        }
        t.state = TRANSACT_SENDING_FILE;
    }

    let state = txn!(transact).state;
    match state {
        TRANSACT_SENDING_FILE => {
            let ref_ = txn!(transact).ref_.expect("ref set for sending file");
            let ref_fd = ref_cache_ref_files_c_161_get_ref_fd(ref_);
            let available = ref_cache_ref_files_c_149_get_ref_available(ref_);
            assert!(ref_fd >= 0);

            let mut end = if ref_cache_ref_files_c_145_get_ref_size(ref_) > 0 {
                txn!(transact).fd_sz
            } else {
                available
            };

            if available <= txn!(transact).fd_sent {
                return REF_CACHE_WRITE_BLOCKED_UPSTREAM;
            }
            if available < end {
                end = available;
            }
            let sent = ref_cache_transaction_c_460_send_file(transact, fd, ref_fd, end);
            if sent < 0 {
                let errno = *crate::htslib_rs::c_compat::__errno_location();
                if !ref_cache_errno_is_would_block(errno) {
                    eprintln!(
                        "sendfile fd #{} : {}",
                        fd,
                        std::io::Error::from_raw_os_error(errno)
                    );
                    return REF_CACHE_WRITE_ERROR;
                }
                return REF_CACHE_WRITE_BLOCKED;
            }
            if txn!(transact).fd_sent < end || ref_cache_ref_files_c_157_get_ref_complete(ref_) == 0 {
                if txn!(transact).fd_sent == end {
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
            eprintln!(
                "resp_send_data entered when transaction in wrong state ({})",
                state
            );
            std::process::abort();
        }
    }
}

// original: transaction_have_content (htslib/ref_cache/transaction.c:619)
pub unsafe fn ref_cache_transaction_c_619_transaction_have_content(
    transact: TransactionId,
) -> c_int {
    (txn!(transact).state > TRANSACT_WAITING_TEXT) as c_int
}

// original: transaction_set_next (htslib/ref_cache/transaction.c:623)
pub unsafe fn ref_cache_transaction_c_623_transaction_set_next(
    transact: TransactionId,
    next: TransactionId,
) {
    let transact = txn!(transact);
    assert!(transact.next.is_none());
    transact.next = Some(next);
}

// original: transaction_has_data_to_send (htslib/ref_cache/transaction.c:628)
pub unsafe fn ref_cache_transaction_c_628_transaction_has_data_to_send(
    transact: TransactionId,
) -> c_int {
    let mut have_data = 0;

    match txn!(transact).state {
        TRANSACT_WAITING_TEXT => {}
        TRANSACT_GOT_TEXT | TRANSACT_SENDING_TEXT => {
            let t = txn!(transact);
            if t.out < t.text_len() {
                have_data = 1;
            }
        }
        TRANSACT_SENDING_FILE => {
            let ref_ = txn!(transact).ref_.expect("ref set for sending file");
            let available = ref_cache_ref_files_c_149_get_ref_available(ref_);
            let mut end = if ref_cache_ref_files_c_145_get_ref_size(ref_) > 0 {
                txn!(transact).fd_sz
            } else {
                available
            };
            if available < end {
                end = available;
            }
            if txn!(transact).fd_sent < end {
                have_data = 1;
            }
        }
        TRANSACT_FINISHED => have_data = 1,
        _ => std::process::abort(),
    }
    have_data
}

// original: set_transaction_file_range (htslib/ref_cache/transaction.c:654)
pub unsafe fn ref_cache_transaction_c_654_set_transaction_file_range(
    transact: TransactionId,
    size: i64,
    ref_data_available: c_int,
) {
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
        let ref_size = txn!(transact)
            .ref_
            .map(|r| ref_cache_ref_files_c_145_get_ref_size(r))
            .unwrap_or(0);
        let t = txn!(transact);
        t.fd_sz = if range_end >= 0 { range_end } else { ref_size };
        t.fd_sent = if range_start >= 0 { range_start } else { 0 };
    }
}

// original: update_with_initial_size (htslib/ref_cache/transaction.c:671)
pub unsafe fn ref_cache_transaction_c_671_update_with_initial_size(
    clients: &mut RefCacheClientsLayout,
    mut transact: Option<TransactionId>,
    size: i64,
    ref_id: c_uint,
    write_stack: &mut usize,
) {
    while let Some(cur) = transact {
        ref_cache_transaction_c_654_set_transaction_file_range(cur, size, 1);
        ref_cache_server_c_395_queue_transaction_write(clients, cur, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(ref_id, Some(cur));
    }
}

// original: got_download_started (htslib/ref_cache/transaction.c:680)
pub unsafe fn ref_cache_transaction_c_680_got_download_started(
    clients: &mut RefCacheClientsLayout,
    id: c_uint,
    val: i64,
    fd: c_int,
    write_stack: &mut usize,
) {
    let Some(transact) = ref_cache_transaction_c_270_transaction_by_id(id, None) else {
        return;
    };

    let ref_ = txn!(transact).ref_.expect("ref set on download start");
    ref_cache_ref_files_c_165_update_ref_download_started(ref_, fd, val);

    if val >= 0 {
        ref_cache_transaction_c_671_update_with_initial_size(
            clients,
            Some(transact),
            val,
            id,
            write_stack,
        );
    }
}

// original: got_download_part (htslib/ref_cache/transaction.c:698)
pub unsafe fn ref_cache_transaction_c_698_got_download_part(
    clients: &mut RefCacheClientsLayout,
    id: c_uint,
    val: i64,
    write_stack: &mut usize,
) {
    assert!(val >= 0);

    let Some(first) = ref_cache_transaction_c_270_transaction_by_id(id, None) else {
        return;
    };

    let ref_ = txn!(first).ref_.expect("ref set on download part");
    ref_cache_ref_files_c_174_update_ref_available(ref_, val);

    let mut transact = Some(first);
    while let Some(cur) = transact {
        ref_cache_server_c_395_queue_transaction_write(clients, cur, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(id, Some(cur));
    }
}

// original: got_download_clen (htslib/ref_cache/transaction.c:715)
pub unsafe fn ref_cache_transaction_c_715_got_download_clen(
    clients: &mut RefCacheClientsLayout,
    id: c_uint,
    val: i64,
    write_stack: &mut usize,
) {
    assert!(val >= 0);

    let Some(transact) = ref_cache_transaction_c_270_transaction_by_id(id, None) else {
        return;
    };

    let ref_ = txn!(transact).ref_.expect("ref set on download clen");
    ref_cache_ref_files_c_179_update_ref_with_content_len(ref_, val);

    ref_cache_transaction_c_671_update_with_initial_size(
        clients,
        Some(transact),
        val,
        id,
        write_stack,
    );
}

// original: got_download_result (htslib/ref_cache/transaction.c:730)
pub unsafe fn ref_cache_transaction_c_730_got_download_result(
    clients: &mut RefCacheClientsLayout,
    id: c_uint,
    val: i64,
    write_stack: &mut usize,
) {
    assert!((0..1000).contains(&val));

    let Some(first) = ref_cache_transaction_c_270_transaction_by_id(id, None) else {
        return;
    };

    if val != 200 {
        let mut transact = Some(first);
        while let Some(cur) = transact {
            ref_cache_transaction_c_277_set_error_response(cur, val as c_uint);
            ref_cache_server_c_395_queue_transaction_write(clients, cur, write_stack);
            transact = ref_cache_transaction_c_270_transaction_by_id(id, Some(cur));
        }
        return;
    }

    let ref_ = txn!(first).ref_.expect("ref set on download result");

    let no_initial_content_length = ref_cache_ref_files_c_185_set_ref_complete(ref_);
    let available = ref_cache_ref_files_c_149_get_ref_available(ref_);

    let mut transact = Some(first);
    while let Some(cur) = transact {
        let t = txn!(cur);
        if no_initial_content_length != 0 || t.fd_sz > available {
            t.fd_sz = available;
        }
        if t.fd_sent >= t.fd_sz {
            t.state = TRANSACT_FINISHED;
        } else if t.state < TRANSACT_SENDING_TEXT {
            ref_cache_transaction_c_654_set_transaction_file_range(cur, available, 1);
        }
        ref_cache_server_c_395_queue_transaction_write(clients, cur, write_stack);
        transact = ref_cache_transaction_c_270_transaction_by_id(id, Some(cur));
    }
}

// original: make_log_message (htslib/ref_cache/transaction.c:769)
//
// Returns the formatted Common-Log-Format line as owned bytes (the C version
// snprintf'd into a caller buffer). The client hostname is resolved by the
// caller (which owns the client arena) and passed in as `client_host`, since a
// transaction now stores only the client id, not a back-pointer to the arena.
pub unsafe fn ref_cache_transaction_c_769_make_log_message(
    transact: TransactionId,
    client_host: &[u8],
) -> Vec<u8> {
    let t = txn!(transact);
    let time = libc::time(std::ptr::null_mut());
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (year, month, day, hour, minute, second, _) =
        crate::htslib_rs::c_compat::unix_time_utc_parts(time);
    let timestamp = format!(
        "{:02}/{}/{:04}:{:02}:{:02}:{:02} +0000",
        day,
        MONTHS[(month - 1) as usize],
        year,
        hour,
        minute,
        second
    );

    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"REQ ");
    out.extend_from_slice(client_host);
    out.extend_from_slice(b" - - ");
    out.extend_from_slice(timestamp.as_bytes());
    out.extend_from_slice(b" \"");
    out.extend_from_slice(&t.req_str);
    out.extend_from_slice(b"\" ");
    out.extend_from_slice(format!("{}", t.rc as c_int).as_bytes());
    out.push(b' ');
    out.extend_from_slice(format!("{}", t.out + t.fd_sent as usize).as_bytes());
    out.extend_from_slice(b" \"");
    out.extend_from_slice(&t.user_agent);
    out.extend_from_slice(b"\" \"");
    out.extend_from_slice(&t.referrer);
    out.extend_from_slice(b"\"\n");
    out
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
