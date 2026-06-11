use super::http_parser::HttpParser;
use super::misc::ref_cache_misc_h_38_hexval;
use super::options::Options;
use super::ref_files::{
    ref_cache_ref_files_c_141_get_ref_status, ref_cache_ref_files_c_145_get_ref_size,
    ref_cache_ref_files_c_193_release_ref_file, ref_cache_ref_files_c_94_get_ref_file,
};
use super::server::RefCacheClientsLayout;
use super::transaction::{
    ref_cache_transaction_c_136_new_transaction, ref_cache_transaction_c_218_transaction_set_ref,
    ref_cache_transaction_c_264_transaction_set_req_str,
    ref_cache_transaction_c_277_set_error_response,
    ref_cache_transaction_c_408_set_message_response,
    ref_cache_transaction_c_654_set_transaction_file_range, TransactionId,
};
use std::ffi::{c_int, c_uint};

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

static TEXT_PLAIN: &[u8] = b"text/plain";

// original: is_hexmd5 (htslib/ref_cache/request_handler.c:49)
//
// `str_` is owned bytes with no trailing NUL; a valid md5 is exactly 32 hex
// digits with nothing following (so byte 32 is end-of-slice, which in C was the
// NUL terminator).
pub fn ref_cache_request_handler_c_49_is_hexmd5(str_: &[u8]) -> c_int {
    if str_.len() != 32 {
        return 0;
    }
    for &b in &str_[..32] {
        if ref_cache_misc_h_38_hexval(b) == -1 {
            return 0;
        }
    }
    1
}

// original: decode_uri (htslib/ref_cache/request_handler.c:56)
pub fn ref_cache_request_handler_c_56_decode_uri(parser: &mut HttpParser) -> Option<Vec<u8>> {
    let mut uri: &[u8] = parser.uri();
    if uri.is_empty() {
        return None;
    }

    /* Deal with absolute URLs */
    if uri.len() >= 7 && uri[..7].eq_ignore_ascii_case(b"http://") {
        match uri[7..].iter().position(|&b| b == b'/') {
            Some(pos) => uri = &uri[7 + pos..],
            None => return None,
        }
    }

    /* Should always start with / now */
    if uri.first() != Some(&b'/') {
        return None;
    }

    /* Hack off query part */
    if let Some(pos) = uri.iter().position(|&b| b == b'?') {
        uri = &uri[..pos];
    }

    /* Deal with multiple // and % decoding. URI will always shrink. */
    let mut out: Vec<u8> = Vec::with_capacity(uri.len());
    let mut last: u8 = b'\0';
    let mut i = 0usize;
    while i < uri.len() {
        let c = uri[i];
        if c == b'/' && last == b'/' {
            i += 1;
            continue;
        }
        if c == b'%' && i + 2 < uri.len() {
            let d1 = ref_cache_misc_h_38_hexval(uri[i + 1]);
            let d2 = ref_cache_misc_h_38_hexval(uri[i + 2]);
            if d1 >= 0 && d2 >= 0 {
                let v = (d1 << 4 | d2) as u8;
                i += 3;
                if v == b'/' && last == b'/' {
                    continue;
                }
                out.push(v);
                last = v;
                continue;
            }
        }
        out.push(c);
        last = c;
        i += 1;
    }
    Some(out)
}

// original: handle_hello (htslib/ref_cache/request_handler.c:94)
pub unsafe fn ref_cache_request_handler_c_94_handle_hello(transact: TransactionId) {
    let resp: &[u8] = b"Hello\r\n";
    ref_cache_transaction_c_408_set_message_response(transact, TEXT_PLAIN, resp);
}

// original: handle_md5 (htslib/ref_cache/request_handler.c:99)
//
// `md5` is the 32 hex-digit ref id as owned bytes (no trailing NUL).
pub unsafe fn ref_cache_request_handler_c_99_handle_md5(
    opts: &Options,
    parser: &mut HttpParser,
    transact: TransactionId,
    md5: &[u8],
) {
    let mut md5_arr = [0u8; 32];
    md5_arr.copy_from_slice(&md5[..32]);
    let Some(ref_file) =
        ref_cache_ref_files_c_94_get_ref_file(opts, &md5_arr, parser.upstream())
    else {
        ref_cache_transaction_c_277_set_error_response(transact, REF_CACHE_ERR_INTERNAL as c_uint);
        return;
    };

    let status = ref_cache_ref_files_c_141_get_ref_status(ref_file);
    if status == REF_NOT_FOUND {
        ref_cache_transaction_c_277_set_error_response(transact, REF_CACHE_ERR_NOT_FOUND as c_uint);
        ref_cache_ref_files_c_193_release_ref_file(ref_file);
        return;
    }

    let size = ref_cache_ref_files_c_145_get_ref_size(ref_file);

    ref_cache_transaction_c_218_transaction_set_ref(transact, ref_file);

    ref_cache_transaction_c_654_set_transaction_file_range(
        transact,
        size,
        (status >= REF_DOWNLOAD_STARTED) as c_int,
    );
}

// original: handle_get (htslib/ref_cache/request_handler.c:126)
pub unsafe fn ref_cache_request_handler_c_126_handle_get(
    opts: &Options,
    parser: &mut HttpParser,
    transact: TransactionId,
) {
    let requested = ref_cache_request_handler_c_56_decode_uri(parser);

    if opts.verbosity > 1 {
        match requested.as_ref() {
            Some(r) => eprintln!("Request: GET {}", String::from_utf8_lossy(r)),
            None => eprintln!("Request: GET "),
        }
    }

    // set_req_str takes owned bytes (no NUL needed).
    ref_cache_transaction_c_264_transaction_set_req_str(
        transact,
        requested.as_deref().unwrap_or(b""),
    );

    let requested = match requested {
        Some(r) if r.first() == Some(&b'/') => r,
        _ => {
            ref_cache_transaction_c_277_set_error_response(
                transact,
                REF_CACHE_ERR_BAD_REQUEST as c_uint,
            );
            return;
        }
    };

    let rest = &requested[1..];
    // A valid ref id is exactly 32 hex digits (the C version required the 32nd
    // byte to be the NUL terminator; with owned bytes that is a 32-byte slice).
    if ref_cache_request_handler_c_49_is_hexmd5(rest) != 0 {
        ref_cache_request_handler_c_99_handle_md5(opts, parser, transact, rest);
        return;
    }
    if rest == b"hello" {
        ref_cache_request_handler_c_94_handle_hello(transact);
        return;
    }
    ref_cache_transaction_c_277_set_error_response(transact, REF_CACHE_ERR_NOT_FOUND as c_uint);
}

// original: handle_request (htslib/ref_cache/request_handler.c:148)
//
// Ownership model: a transaction is an index (`TransactionId`) into the
// transaction arena owned by `transaction.rs`. The client is identified by its
// arena index in the server's client arena. On success, the newly created
// transaction's id is written into `transact_out` for the caller (which then
// appends it to the client's request pipeline). `_clients` is threaded through
// so the handler can reach the client arena if needed; the client id is `client`.
pub unsafe fn ref_cache_request_handler_c_148_handle_request(
    opts: &Options,
    _clients: &mut RefCacheClientsLayout,
    client: usize,
    parser: &mut HttpParser,
    transact_out: &mut Option<TransactionId>,
) {
    let Some(transact) = ref_cache_transaction_c_136_new_transaction(Some(client), parser) else {
        parser.set_state(REF_CACHE_ERR_INTERNAL);
        return;
    };

    match parser.req_type() {
        REF_CACHE_REQ_GET => {
            ref_cache_request_handler_c_126_handle_get(opts, parser, transact);
        }
        /* case REQ_HEAD: handle_head(parser, transact); break; */
        _ => {
            ref_cache_transaction_c_277_set_error_response(
                transact,
                REF_CACHE_ERR_UNIMPLEMENTED as c_uint,
            );
        }
    }
    *transact_out = Some(transact);

    if (parser.flags() & TRANSACT_KEEP_ALIVE) != 0 && parser.http_vers() == HTTP_1_1 {
        parser.set_state(REF_CACHE_READING_REQUEST_LINE);
    } else {
        parser.set_state(REF_CACHE_SHUTTING_DOWN);
    }
}

// original: handle_error (htslib/ref_cache/request_handler.c:167)
pub unsafe fn ref_cache_request_handler_c_167_handle_error(
    _clients: &mut RefCacheClientsLayout,
    client: usize,
    parser: &mut HttpParser,
    code: c_int,
    transact_out: &mut Option<TransactionId>,
) {
    let Some(transact) = ref_cache_transaction_c_136_new_transaction(Some(client), parser) else {
        parser.set_state(REF_CACHE_ERR_INTERNAL);
        return;
    };
    ref_cache_transaction_c_277_set_error_response(
        transact,
        if code >= REF_CACHE_ERR_BAD_REQUEST {
            code as c_uint
        } else {
            500
        },
    );
    *transact_out = Some(transact);
}
