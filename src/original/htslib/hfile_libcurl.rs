#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]
use crate::htslib_mini_rs::hts::{hFILE, kstring_t};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

const HFILE_LIBCURL_PAUSED: c_uint = 1 << 0;
const HFILE_LIBCURL_CLOSING: c_uint = 1 << 1;

#[repr(C)]
struct HFileLibcurlAuthToken {
    path: *mut c_char,
    token: *mut c_char,
    expiry: libc::time_t,
    failed: c_int,
    lock: libc::pthread_mutex_t,
}

#[repr(C)]
struct HFileLibcurlCurlSlist {
    data: *mut c_char,
    next: *mut HFileLibcurlCurlSlist,
}

#[repr(C)]
struct HFileLibcurlHdrList {
    list: *mut HFileLibcurlCurlSlist,
    num: c_uint,
    size: c_uint,
}

#[repr(C)]
struct HFileLibcurlBuffer {
    ptr: *mut c_char,
    len: usize,
}

#[repr(C)]
struct HFileLibcurlCallbackPrefix {
    base: hFILE,
    easy: *mut c_void,
    multi: *mut c_void,
    file_size: libc::off_t,
    buffer: HFileLibcurlBuffer,
    final_result: c_int,
    flags: c_uint,
}

type HFileLibcurlHttpHeaderCallback =
    unsafe extern "C" fn(*mut c_void, *mut *mut *mut c_char) -> c_int;

#[repr(C)]
struct HFileLibcurlHeaders {
    fixed: HFileLibcurlHdrList,
    extra: HFileLibcurlHdrList,
    callback: Option<HFileLibcurlHttpHeaderCallback>,
    callback_data: *mut c_void,
    auth: *mut HFileLibcurlAuthToken,
    auth_hdr_num: c_int,
    redirect: *mut c_void,
    redirect_data: *mut c_void,
    http_response_ptr: *mut libc::c_long,
    fail_on_error: c_int,
}

#[repr(C)]
struct HFileLibcurlHeaderPrefix {
    base: hFILE,
    easy: *mut c_void,
    multi: *mut c_void,
    file_size: libc::off_t,
    buffer: HFileLibcurlBuffer,
    final_result: c_int,
    flags: c_uint,
    nrunning: c_int,
    headers: HFileLibcurlHeaders,
}

static mut HFILE_LIBCURL_SHARE_LOCK: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;

const CURLE_OK: c_int = 0;
const CURLINFO_RESPONSE_CODE: c_int = 0x200000 + 2;
const CURLINFO_OS_ERRNO: c_int = 0x200000 + 25;

#[link(name = "curl")]
unsafe extern "C" {
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_long(curl: *mut c_void, info: c_int, value: *mut libc::c_long) -> c_int;
    fn curl_easy_strerror(code: c_int) -> *const c_char;
    fn curl_multi_strerror(code: c_int) -> *const c_char;
}

// original: http_status_errno (htslib/hfile_libcurl.c:130)
pub unsafe fn hfile_libcurl_c_130_http_status_errno(status: c_int) -> c_int {
    if status >= 500 {
        match status {
            501 => libc::ENOSYS,
            503 => libc::EBUSY,
            504 => libc::ETIMEDOUT,
            _ => libc::EIO,
        }
    } else if status >= 400 {
        match status {
            401 => libc::EPERM,
            403 => libc::EACCES,
            404 => libc::ENOENT,
            405 => libc::EROFS,
            407 => libc::EPERM,
            408 => libc::ETIMEDOUT,
            410 => libc::ENOENT,
            _ => libc::EINVAL,
        }
    } else {
        0
    }
}

// original: easy_errno (htslib/hfile_libcurl.c:153)
pub unsafe fn hfile_libcurl_c_153_easy_errno(easy: *mut c_void, err: c_int) -> c_int {
    const CURLE_UNSUPPORTED_PROTOCOL: c_int = 1;
    const CURLE_URL_MALFORMAT: c_int = 3;
    const CURLE_NOT_BUILT_IN: c_int = 4;
    const CURLE_COULDNT_RESOLVE_PROXY: c_int = 5;
    const CURLE_COULDNT_RESOLVE_HOST: c_int = 6;
    const CURLE_COULDNT_CONNECT: c_int = 7;
    const CURLE_REMOTE_ACCESS_DENIED: c_int = 9;
    const CURLE_FTP_CANT_GET_HOST: c_int = 15;
    const CURLE_PARTIAL_FILE: c_int = 18;
    const CURLE_HTTP_RETURNED_ERROR: c_int = 22;
    const CURLE_OUT_OF_MEMORY: c_int = 27;
    const CURLE_OPERATION_TIMEDOUT: c_int = 28;
    const CURLE_RANGE_ERROR: c_int = 33;
    const CURLE_SSL_CONNECT_ERROR: c_int = 35;
    const CURLE_FILE_COULDNT_READ_FILE: c_int = 37;
    const CURLE_TOO_MANY_REDIRECTS: c_int = 47;
    const CURLE_SEND_ERROR: c_int = 55;
    const CURLE_RECV_ERROR: c_int = 56;
    const CURLE_FILESIZE_EXCEEDED: c_int = 63;
    const CURLE_LOGIN_DENIED: c_int = 67;
    const CURLE_TFTP_NOTFOUND: c_int = 68;
    const CURLE_TFTP_PERM: c_int = 69;
    const CURLE_REMOTE_DISK_FULL: c_int = 70;
    const CURLE_REMOTE_FILE_EXISTS: c_int = 73;

    match err {
        CURLE_OK => 0,
        CURLE_UNSUPPORTED_PROTOCOL | CURLE_URL_MALFORMAT => libc::EINVAL,
        CURLE_NOT_BUILT_IN => libc::ENOSYS,
        CURLE_COULDNT_RESOLVE_PROXY | CURLE_COULDNT_RESOLVE_HOST | CURLE_FTP_CANT_GET_HOST => {
            libc::EDESTADDRREQ
        }
        CURLE_COULDNT_CONNECT | CURLE_SEND_ERROR | CURLE_RECV_ERROR => {
            let mut lval: libc::c_long = 0;
            if !easy.is_null()
                && curl_easy_getinfo_long(easy, CURLINFO_OS_ERRNO, &mut lval) == CURLE_OK
            {
                lval as c_int
            } else {
                libc::ECONNABORTED
            }
        }
        CURLE_REMOTE_ACCESS_DENIED | CURLE_LOGIN_DENIED | CURLE_TFTP_PERM => libc::EACCES,
        CURLE_PARTIAL_FILE => libc::EPIPE,
        CURLE_HTTP_RETURNED_ERROR => {
            let mut lval: libc::c_long = 0;
            if !easy.is_null()
                && curl_easy_getinfo_long(easy, CURLINFO_RESPONSE_CODE, &mut lval) == CURLE_OK
            {
                hfile_libcurl_c_130_http_status_errno(lval as c_int)
            } else {
                libc::EIO
            }
        }
        CURLE_OUT_OF_MEMORY => libc::ENOMEM,
        CURLE_OPERATION_TIMEDOUT => libc::ETIMEDOUT,
        CURLE_RANGE_ERROR => libc::ESPIPE,
        CURLE_SSL_CONNECT_ERROR => libc::ECONNABORTED,
        CURLE_FILE_COULDNT_READ_FILE | CURLE_TFTP_NOTFOUND => libc::ENOENT,
        CURLE_TOO_MANY_REDIRECTS => libc::ELOOP,
        CURLE_FILESIZE_EXCEEDED => libc::EFBIG,
        CURLE_REMOTE_DISK_FULL => libc::ENOSPC,
        CURLE_REMOTE_FILE_EXISTS => libc::EEXIST,
        _ => {
            hts_sys::hts_log(
                hts_sys::htsLogLevel_HTS_LOG_ERROR,
                c"easy_errno".as_ptr(),
                c"Libcurl reported error %d (%s)".as_ptr(),
                err,
                curl_easy_strerror(err),
            );
            libc::EIO
        }
    }
}

// original: is_retryable (htslib/hfile_libcurl.c:233)
pub unsafe fn hfile_libcurl_c_233_is_retryable(easy: *mut c_void, err: c_int) -> c_int {
    const CURLE_COULDNT_CONNECT: c_int = 7;
    const CURLE_HTTP2: c_int = 16;
    const CURLE_PARTIAL_FILE: c_int = 18;
    const CURLE_HTTP_RETURNED_ERROR: c_int = 22;
    const CURLE_OPERATION_TIMEDOUT: c_int = 28;
    const CURLE_SSL_CONNECT_ERROR: c_int = 35;
    const CURLE_GOT_NOTHING: c_int = 52;
    const CURLE_SEND_ERROR: c_int = 55;
    const CURLE_RECV_ERROR: c_int = 56;
    const CURLE_HTTP2_STREAM: c_int = 92;

    match err {
        CURLE_COULDNT_CONNECT
        | CURLE_SEND_ERROR
        | CURLE_RECV_ERROR
        | CURLE_PARTIAL_FILE
        | CURLE_OPERATION_TIMEDOUT
        | CURLE_GOT_NOTHING
        | CURLE_SSL_CONNECT_ERROR
        | CURLE_HTTP2
        | CURLE_HTTP2_STREAM => 1,
        CURLE_HTTP_RETURNED_ERROR => {
            let mut response: libc::c_long = 0;
            if !easy.is_null()
                && curl_easy_getinfo_long(easy, CURLINFO_RESPONSE_CODE, &mut response) == CURLE_OK
            {
                match response {
                    429 | 500 | 502 | 503 | 504 => 1,
                    _ => 0,
                }
            } else {
                0
            }
        }
        _ => 0,
    }
}

// original: multi_errno (htslib/hfile_libcurl.c:270)
pub unsafe fn hfile_libcurl_c_270_multi_errno(errm: c_int) -> c_int {
    match errm {
        -1 | 0 => 0,
        1 | 2 | 5 => libc::EBADF,
        3 => libc::ENOMEM,
        _ => {
            hts_sys::hts_log(
                hts_sys::htsLogLevel_HTS_LOG_ERROR,
                c"multi_errno".as_ptr(),
                c"Libcurl reported error %d (%s)".as_ptr(),
                errm,
                curl_multi_strerror(errm),
            );
            libc::EIO
        }
    }
}

// original: share_lock (htslib/hfile_libcurl.c:309)
pub unsafe fn hfile_libcurl_c_309_share_lock() {
    libc::pthread_mutex_lock(std::ptr::addr_of_mut!(HFILE_LIBCURL_SHARE_LOCK));
}

// original: share_unlock (htslib/hfile_libcurl.c:314)
pub unsafe fn hfile_libcurl_c_314_share_unlock() {
    libc::pthread_mutex_unlock(std::ptr::addr_of_mut!(HFILE_LIBCURL_SHARE_LOCK));
}

// original: free_auth (htslib/hfile_libcurl.c:318)
pub unsafe fn hfile_libcurl_c_318_free_auth(tok: *mut c_void) {
    let tok = tok.cast::<HFileLibcurlAuthToken>();
    if tok.is_null() {
        return;
    }
    if libc::pthread_mutex_destroy(&mut (*tok).lock) != 0 {
        libc::abort();
    }
    libc::free((*tok).path.cast());
    libc::free((*tok).token.cast());
    libc::free(tok.cast());
}

// original: libcurl_exit (htslib/hfile_libcurl.c:326)
pub unsafe fn hfile_libcurl_c_326_libcurl_exit() {}

// original: append_header (htslib/hfile_libcurl.c:353)
pub unsafe fn hfile_libcurl_c_353_append_header(
    hdrs: *mut c_void,
    data: *const c_char,
    dup: c_int,
) -> c_int {
    let hdrs = hdrs.cast::<HFileLibcurlHdrList>();
    if (*hdrs).num == (*hdrs).size {
        let new_sz = if (*hdrs).size != 0 {
            (*hdrs).size * 2
        } else {
            4
        };
        let new_list = libc::realloc(
            (*hdrs).list.cast(),
            new_sz as usize * std::mem::size_of::<HFileLibcurlCurlSlist>(),
        )
        .cast::<HFileLibcurlCurlSlist>();
        if new_list.is_null() {
            return -1;
        }
        (*hdrs).size = new_sz;
        (*hdrs).list = new_list;
        for i in 1..(*hdrs).num {
            (*(*hdrs).list.add(i as usize - 1)).next = (*hdrs).list.add(i as usize);
        }
    }

    let entry = (*hdrs).list.add((*hdrs).num as usize);
    (*entry).data = if dup != 0 {
        libc::strdup(data)
    } else {
        data.cast_mut()
    };
    if (*entry).data.is_null() {
        return -1;
    }
    if (*hdrs).num > 0 {
        (*(*hdrs).list.add((*hdrs).num as usize - 1)).next = entry;
    }
    (*entry).next = std::ptr::null_mut();
    (*hdrs).num += 1;
    0
}

// original: free_headers (htslib/hfile_libcurl.c:372)
pub unsafe fn hfile_libcurl_c_372_free_headers(hdrs: *mut c_void, completely: c_int) {
    let hdrs = hdrs.cast::<HFileLibcurlHdrList>();
    for i in 0..(*hdrs).num {
        let entry = (*hdrs).list.add(i as usize);
        libc::free((*entry).data.cast());
        (*entry).data = std::ptr::null_mut();
        (*entry).next = std::ptr::null_mut();
    }
    (*hdrs).num = 0;
    if completely != 0 {
        libc::free((*hdrs).list.cast());
        (*hdrs).size = 0;
        (*hdrs).list = std::ptr::null_mut();
    }
}

// original: get_header_list (htslib/hfile_libcurl.c:387)
pub unsafe fn hfile_libcurl_c_387_get_header_list(fp: *mut c_void) -> *mut c_void {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    if (*fp).headers.fixed.num > 0 {
        return (*fp).headers.fixed.list.cast();
    }
    if (*fp).headers.extra.num > 0 {
        return (*fp).headers.extra.list.cast();
    }
    std::ptr::null_mut()
}

// original: is_authorization (htslib/hfile_libcurl.c:395)
pub unsafe fn hfile_libcurl_c_395_is_authorization(hdr: *const c_char) -> c_int {
    (libc::strncasecmp(c"authorization:".as_ptr(), hdr, 14) == 0) as c_int
}

// original: add_callback_headers (htslib/hfile_libcurl.c:399)
pub unsafe fn hfile_libcurl_c_399_add_callback_headers(fp: *mut c_void) -> c_int {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    let Some(callback) = (*fp).headers.callback else {
        return 0;
    };

    let mut hdrs: *mut *mut c_char = std::ptr::null_mut();
    if callback((*fp).headers.callback_data, &mut hdrs) != 0 {
        return -1;
    }
    if hdrs.is_null() {
        return 0;
    }

    if (*fp).headers.fixed.num > 0 {
        (*(*fp)
            .headers
            .fixed
            .list
            .add((*fp).headers.fixed.num as usize - 1))
        .next = std::ptr::null_mut();
    }
    hfile_libcurl_c_372_free_headers(
        (&mut (*fp).headers.extra as *mut HFileLibcurlHdrList).cast(),
        0,
    );

    if (*fp).headers.auth_hdr_num > 0 || (*fp).headers.auth_hdr_num == -2 {
        (*fp).headers.auth_hdr_num = 0;
    }

    let mut hdr = hdrs;
    while !(*hdr).is_null() {
        if hfile_libcurl_c_353_append_header(
            (&mut (*fp).headers.extra as *mut HFileLibcurlHdrList).cast(),
            *hdr,
            0,
        ) < 0
        {
            while !hdr.is_null() && !(*hdr).is_null() {
                libc::free((*hdr).cast());
                *hdr = std::ptr::null_mut();
                hdr = hdr.add(1);
            }
            return -1;
        }
        if hfile_libcurl_c_395_is_authorization(*hdr) != 0 && (*fp).headers.auth_hdr_num == 0 {
            (*fp).headers.auth_hdr_num = -2;
        }
        hdr = hdr.add(1);
    }

    hdr = hdrs;
    while !(*hdr).is_null() {
        *hdr = std::ptr::null_mut();
        hdr = hdr.add(1);
    }

    if (*fp).headers.fixed.num > 0 && (*fp).headers.extra.num > 0 {
        (*(*fp)
            .headers
            .fixed
            .list
            .add((*fp).headers.fixed.num as usize - 1))
        .next = (*fp).headers.extra.list;
    }
    0
}

// original: read_auth_json (htslib/hfile_libcurl.c:454)
pub unsafe fn hfile_libcurl_c_454_read_auth_json() {}

// original: read_auth_plain (htslib/hfile_libcurl.c:515)
pub unsafe fn hfile_libcurl_c_515_read_auth_plain(tok: *mut c_void, auth_fp: *mut hFILE) -> c_int {
    let tok = tok.cast::<HFileLibcurlAuthToken>();
    let mut line: kstring_t = std::mem::zeroed();
    let mut token: kstring_t = std::mem::zeroed();

    if crate::htslib_mini_rs::hfile::khgetline(&mut line, auth_fp) < 0 {
        libc::free(line.s.cast());
        libc::free(token.s.cast());
        return -1;
    }
    if crate::htslib_mini_rs::hts::kputc(0, &mut line) < 0 {
        libc::free(line.s.cast());
        libc::free(token.s.cast());
        return -1;
    }

    let mut start = line.s;
    while *start != 0 && libc::isspace(*start as c_uchar as c_int) != 0 {
        start = start.add(1);
    }
    let mut end = start;
    while *end != 0 && libc::isspace(*end as c_uchar as c_int) == 0 {
        end = end.add(1);
    }

    if end > start {
        if crate::htslib_mini_rs::hts::kputs(c"Authorization: Bearer ".as_ptr(), &mut token) < 0
            || crate::htslib_mini_rs::hts::kputsn(
                start,
                end.offset_from(start) as usize,
                &mut token,
            ) < 0
        {
            libc::free(line.s.cast());
            libc::free(token.s.cast());
            return -1;
        }
    }

    libc::free((*tok).token.cast());
    (*tok).token = crate::htslib_mini_rs::hts::ks_release(&mut token);
    (*tok).expiry = 0;
    libc::free(line.s.cast());
    0
}

// original: renew_auth_token (htslib/hfile_libcurl.c:543)
pub unsafe fn hfile_libcurl_c_543_renew_auth_token() {}

// original: add_auth_header (htslib/hfile_libcurl.c:587)
pub unsafe fn hfile_libcurl_c_587_add_auth_header() {}

// original: get_auth_token (htslib/hfile_libcurl.c:650)
pub unsafe fn hfile_libcurl_c_650_get_auth_token() {}

// original: process_messages (htslib/hfile_libcurl.c:718)
pub unsafe fn hfile_libcurl_c_718_process_messages() {}

// original: wait_perform (htslib/hfile_libcurl.c:736)
pub unsafe fn hfile_libcurl_c_736_wait_perform() {}

// original: recv_callback (htslib/hfile_libcurl.c:789)
pub unsafe fn hfile_libcurl_c_789_recv_callback(
    ptr: *mut c_char,
    size: usize,
    nmemb: usize,
    fpv: *mut c_void,
) -> usize {
    const CURL_WRITEFUNC_PAUSE: usize = 0x10000001;
    const HFILE_LIBCURL_PAUSED: c_uint = 1 << 0;

    let fp = fpv.cast::<HFileLibcurlCallbackPrefix>();
    let n = size.saturating_mul(nmemb);

    if n > (*fp).buffer.len {
        (*fp).flags |= HFILE_LIBCURL_PAUSED;
        CURL_WRITEFUNC_PAUSE
    } else if n == 0 {
        0
    } else {
        libc::memcpy((*fp).buffer.ptr.cast(), ptr.cast(), n);
        (*fp).buffer.ptr = (*fp).buffer.ptr.add(n);
        (*fp).buffer.len -= n;
        n
    }
}

// original: header_callback (htslib/hfile_libcurl.c:807)
pub unsafe fn hfile_libcurl_c_807_header_callback(
    contents: *mut c_void,
    size: usize,
    nmemb: usize,
    userp: *mut c_void,
) -> usize {
    let realsize = size.saturating_mul(nmemb);
    let resp = userp.cast::<kstring_t>();

    if crate::htslib_mini_rs::hts::kputsn(contents.cast(), realsize, resp) == libc::EOF {
        0
    } else {
        realsize
    }
}

// original: refresh_retry_config (htslib/hfile_libcurl.c:821)
pub unsafe fn hfile_libcurl_c_821_refresh_retry_config() {}

// original: retry_sleep (htslib/hfile_libcurl.c:836)
pub unsafe fn hfile_libcurl_c_836_retry_sleep(delay_ms: libc::c_long) {
    let ts = libc::timespec {
        tv_sec: delay_ms / 1000,
        tv_nsec: (delay_ms % 1000) * 1_000_000,
    };
    libc::nanosleep(&ts, std::ptr::null_mut());
}

// original: retry_reconnect (htslib/hfile_libcurl.c:848)
pub unsafe fn hfile_libcurl_c_848_retry_reconnect() {}

// original: libcurl_read (htslib/hfile_libcurl.c:876)
pub unsafe fn hfile_libcurl_c_876_libcurl_read() {}

// original: send_callback (htslib/hfile_libcurl.c:1006)
pub unsafe fn hfile_libcurl_c_1006_send_callback(
    ptr: *mut c_char,
    size: usize,
    nmemb: usize,
    fpv: *mut c_void,
) -> usize {
    const CURL_READFUNC_PAUSE: usize = 0x10000001;
    const HFILE_LIBCURL_PAUSED: c_uint = 1 << 0;
    const HFILE_LIBCURL_CLOSING: c_uint = 1 << 1;

    let fp = fpv.cast::<HFileLibcurlCallbackPrefix>();
    let mut n = size.saturating_mul(nmemb);

    if (*fp).buffer.len == 0 {
        if ((*fp).flags & HFILE_LIBCURL_CLOSING) != 0 {
            0
        } else {
            (*fp).flags |= HFILE_LIBCURL_PAUSED;
            CURL_READFUNC_PAUSE
        }
    } else {
        if n > (*fp).buffer.len {
            n = (*fp).buffer.len;
        }
        libc::memcpy(ptr.cast(), (*fp).buffer.ptr.cast(), n);
        (*fp).buffer.ptr = (*fp).buffer.ptr.add(n);
        (*fp).buffer.len -= n;
        n
    }
}

// original: libcurl_write (htslib/hfile_libcurl.c:1024)
pub unsafe fn hfile_libcurl_c_1024_libcurl_write() {}

// original: preserve_buffer_content (htslib/hfile_libcurl.c:1051)
pub unsafe fn hfile_libcurl_c_1051_preserve_buffer_content() {}

// original: libcurl_seek (htslib/hfile_libcurl.c:1071)
pub unsafe fn hfile_libcurl_c_1071_libcurl_seek() {}

// original: restart_from_position (htslib/hfile_libcurl.c:1134)
pub unsafe fn hfile_libcurl_c_1134_restart_from_position() {}

// original: libcurl_close (htslib/hfile_libcurl.c:1266)
pub unsafe fn hfile_libcurl_c_1266_libcurl_close() {}

// original: libcurl_open (htslib/hfile_libcurl.c:1313)
pub unsafe fn hfile_libcurl_c_1313_libcurl_open() {}

// original: hopen_libcurl (htslib/hfile_libcurl.c:1549)
pub unsafe fn hfile_libcurl_c_1549_hopen_libcurl() {}

// original: parse_va_list (htslib/hfile_libcurl.c:1554)
pub unsafe fn hfile_libcurl_c_1554_parse_va_list() {}

// original: vhopen_libcurl (htslib/hfile_libcurl.c:1664)
pub unsafe fn hfile_libcurl_c_1664_vhopen_libcurl() {}

// original: PLUGIN_GLOBAL (htslib/hfile_libcurl.c:1679)
pub unsafe fn hfile_libcurl_c_1679_PLUGIN_GLOBAL() {}
/*  hfile_libcurl.c -- libcurl backend for low-level file streams.

    Copyright (C) 2015-2017, 2019-2020 Genome Research Ltd.

    Author: John Marshall <jm18@sanger.ac.uk>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.  */
