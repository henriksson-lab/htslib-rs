#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_variables
)]
use crate::htslib_rs::{
    hfile::{
        hFILE_plugin, hFILE_scheme_handler, hclose, hclose_abruptly, hfile_add_scheme_handler,
        hfile_c_1342_hfile_always_remote, hfile_destroy, hfile_init, hopen, hpeek,
    },
    hts::{
        hFILE, hts_json_alloc_token, hts_json_fnext, hts_json_free_token, hts_json_fskip_value,
        hts_json_token_str, hts_json_token_type, hts_verbose, kputs, kputsn, ks_release, kstring_t,
        size_t,
    },
};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

const HFILE_LIBCURL_PAUSED: c_uint = 1 << 0;
const HFILE_LIBCURL_CLOSING: c_uint = 1 << 1;
const HFILE_LIBCURL_FINISHED: c_uint = 1 << 2;
const HFILE_LIBCURL_PERFORM_AGAIN: c_uint = 1 << 3;
const HFILE_LIBCURL_IS_READ: c_uint = 1 << 4;
const HFILE_LIBCURL_CAN_SEEK: c_uint = 1 << 5;
const HFILE_LIBCURL_IS_RECURSIVE: c_uint = 1 << 6;
const HFILE_LIBCURL_TRIED_SEEK: c_uint = 1 << 7;

const AUTH_REFRESH_EARLY_SECS: libc::time_t = 60;
const MIN_SEEK_FORWARD: libc::off_t = 1_000_000;

type HFileReadFn = unsafe extern "C" fn(*mut hFILE, *mut c_void, size_t) -> libc::ssize_t;
type HFileWriteFn = unsafe extern "C" fn(*mut hFILE, *const c_void, size_t) -> libc::ssize_t;
type HFileSeekFn = unsafe extern "C" fn(*mut hFILE, libc::off_t, c_int) -> libc::off_t;
type HFileFlushFn = unsafe extern "C" fn(*mut hFILE) -> c_int;
type HFileCloseFn = unsafe extern "C" fn(*mut hFILE) -> c_int;

#[repr(C)]
struct HFileBackend {
    read: Option<HFileReadFn>,
    write: Option<HFileWriteFn>,
    seek: Option<HFileSeekFn>,
    flush: Option<HFileFlushFn>,
    close: Option<HFileCloseFn>,
}

#[repr(C)]
struct HFileLayout {
    buffer: *mut c_char,
    begin: *mut c_char,
    end: *mut c_char,
    limit: *mut c_char,
    backend: *const HFileBackend,
    offset: libc::off_t,
    flags: c_uint,
    has_errno: c_int,
}

#[repr(C)]
struct HFileLibcurlAuthToken {
    path: *mut c_char,
    token: *mut c_char,
    expiry: libc::time_t,
    failed: c_int,
    lock: libc::pthread_mutex_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
    base: HFileLayout,
    easy: *mut c_void,
    multi: *mut c_void,
    file_size: libc::off_t,
    buffer: HFileLibcurlBuffer,
    final_result: c_int,
    flags: c_uint,
}

type HFileLibcurlHttpHeaderCallback =
    unsafe extern "C" fn(*mut c_void, *mut *mut *mut c_char) -> c_int;
type HFileLibcurlRedirectCallback =
    unsafe extern "C" fn(*mut c_void, libc::c_long, *mut kstring_t, *mut kstring_t) -> c_int;
type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn =
    unsafe extern "C" fn(*const c_char, *const c_char, *mut hts_sys::__va_list_tag) -> *mut hFILE;

#[repr(C)]
struct HFileSchemeHandlerLayout {
    open: Option<HFileOpenFn>,
    isremote: Option<HFileIsRemoteFn>,
    provider: *const c_char,
    priority: c_int,
    vopen: Option<HFileVOpenFn>,
}

unsafe impl Sync for HFileSchemeHandlerLayout {}

#[repr(C)]
struct HFilePluginLayout {
    api_version: c_int,
    obj: *mut c_void,
    name: *const c_char,
    destroy: *const c_void,
}

#[repr(C)]
pub struct HFileLibcurlHeaders {
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
    base: HFileLayout,
    easy: *mut c_void,
    multi: *mut c_void,
    file_size: libc::off_t,
    buffer: HFileLibcurlBuffer,
    final_result: c_int,
    flags: c_uint,
    nrunning: c_int,
    headers: HFileLibcurlHeaders,
    delayed_seek: libc::off_t,
    last_offset: libc::off_t,
    preserved: *mut c_char,
    preserved_bytes: usize,
    preserved_size: usize,
}

static mut HFILE_LIBCURL_SHARE_LOCK: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
static mut HFILE_LIBCURL_AUTH_LOCK: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
static mut HFILE_LIBCURL_USERAGENT: kstring_t = kstring_t {
    l: 0,
    m: 0,
    s: std::ptr::null_mut(),
};
static mut HFILE_LIBCURL_SHARE: *mut c_void = std::ptr::null_mut();
static mut HFILE_LIBCURL_AUTH_PATH: *mut c_char = std::ptr::null_mut();
static mut HFILE_LIBCURL_AUTH_MAP: *mut Vec<usize> = std::ptr::null_mut();
static mut HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER: c_int = 0;
static mut HFILE_LIBCURL_RETRY_MAX: c_int = 0;
static mut HFILE_LIBCURL_RETRY_DELAY_MS: libc::c_long = 1000;

const CURLE_OK: c_int = 0;
const CURLE_HTTP_RETURNED_ERROR: c_int = 22;
const CURLM_CALL_MULTI_PERFORM: c_int = -1;
const CURLM_OK: c_int = 0;
const CURLMSG_DONE: c_int = 1;
const CURLINFO_RESPONSE_CODE: c_int = 0x200000 + 2;
const CURLINFO_OS_ERRNO: c_int = 0x200000 + 25;
const CURLINFO_CONTENT_LENGTH_DOWNLOAD_T: c_int = 0x600000 + 15;
const CURL_GLOBAL_ALL: libc::c_long = 3;
const CURLPAUSE_CONT: c_int = 0;
const CURLFTPMETHOD_NOCWD: libc::c_long = 1;

const CURLOPT_WRITEDATA: c_int = 10_001;
const CURLOPT_URL: c_int = 10_002;
const CURLOPT_READDATA: c_int = 10_009;
const CURLOPT_WRITEFUNCTION: c_int = 20_011;
const CURLOPT_READFUNCTION: c_int = 20_012;
const CURLOPT_USERAGENT: c_int = 10_018;
const CURLOPT_HTTPHEADER: c_int = 10_023;
const CURLOPT_HEADERDATA: c_int = 10_029;
const CURLOPT_VERBOSE: c_int = 41;
const CURLOPT_FAILONERROR: c_int = 45;
const CURLOPT_UPLOAD: c_int = 46;
const CURLOPT_FOLLOWLOCATION: c_int = 52;
const CURLOPT_CAINFO: c_int = 10_065;
const CURLOPT_HEADERFUNCTION: c_int = 20_079;
const CURLOPT_SHARE: c_int = 10_100;
const CURLOPT_PRIVATE: c_int = 10_103;
const CURLOPT_RESUME_FROM_LARGE: c_int = 30_116;
const CURLOPT_FTP_FILEMETHOD: c_int = 138;

const CURL_LOCK_DATA_DNS: c_int = 3;
const CURLSHE_OK: c_int = 0;
const CURLSHOPT_SHARE: c_int = 1;
const CURLSHOPT_LOCKFUNC: c_int = 3;
const CURLSHOPT_UNLOCKFUNC: c_int = 4;
const CURLVERSION_NOW: c_int = 10;

#[repr(C)]
struct CurlVersionInfoData {
    age: c_int,
    version: *const c_char,
    version_num: c_uint,
    host: *const c_char,
    features: c_int,
    ssl_version: *const c_char,
    ssl_version_num: libc::c_long,
    libz_version: *const c_char,
    protocols: *mut *const c_char,
}

#[repr(C)]
union CurlMsgData {
    whatever: *mut c_void,
    result: c_int,
}

#[repr(C)]
struct CurlMsg {
    msg: c_int,
    easy_handle: *mut c_void,
    data: CurlMsgData,
}

unsafe extern "C" {
    #[link_name = "hfile_plugin_init_libcurl"]
    fn htslib_hfile_plugin_init_libcurl(self_: *mut hFILE_plugin) -> c_int;
    #[link_name = "hopen"]
    fn htslib_hopen(fname: *const c_char, mode: *const c_char, ...) -> *mut hFILE;
}

#[link(name = "curl")]
unsafe extern "C" {
    fn curl_global_init(flags: libc::c_long) -> c_int;
    fn curl_global_cleanup();
    fn curl_version_info(type_: c_int) -> *mut CurlVersionInfoData;
    fn curl_share_init() -> *mut c_void;
    fn curl_share_cleanup(share: *mut c_void) -> c_int;
    fn curl_share_setopt(share: *mut c_void, option: c_int, ...) -> c_int;
    fn curl_easy_init() -> *mut c_void;
    fn curl_easy_cleanup(curl: *mut c_void);
    fn curl_easy_reset(curl: *mut c_void);
    fn curl_easy_duphandle(curl: *mut c_void) -> *mut c_void;
    fn curl_easy_pause(curl: *mut c_void, bitmask: c_int) -> c_int;
    fn curl_easy_setopt(curl: *mut c_void, option: c_int, ...) -> c_int;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_long(curl: *mut c_void, info: c_int, value: *mut libc::c_long) -> c_int;
    #[link_name = "curl_easy_getinfo"]
    fn curl_easy_getinfo_off_t(curl: *mut c_void, info: c_int, value: *mut libc::off_t) -> c_int;
    fn curl_easy_strerror(code: c_int) -> *const c_char;
    fn curl_multi_init() -> *mut c_void;
    fn curl_multi_cleanup(multi_handle: *mut c_void) -> c_int;
    fn curl_multi_add_handle(multi_handle: *mut c_void, curl_handle: *mut c_void) -> c_int;
    fn curl_multi_remove_handle(multi_handle: *mut c_void, curl_handle: *mut c_void) -> c_int;
    fn curl_multi_perform(multi_handle: *mut c_void, running_handles: *mut c_int) -> c_int;
    fn curl_multi_info_read(multi_handle: *mut c_void, msgs_in_queue: *mut c_int) -> *mut CurlMsg;
    fn curl_multi_wait(
        multi_handle: *mut c_void,
        extra_fds: *mut c_void,
        extra_nfds: c_uint,
        timeout_ms: c_int,
        ret: *mut c_int,
    ) -> c_int;
    fn curl_multi_timeout(multi_handle: *mut c_void, timeout_ms: *mut libc::c_long) -> c_int;
    fn curl_multi_strerror(code: c_int) -> *const c_char;
}

unsafe extern "C" fn hfile_libcurl_open(url: *const c_char, modes: *const c_char) -> *mut hFILE {
    hfile_libcurl_c_1313_libcurl_open(url, modes, std::ptr::null_mut())
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
            429 => libc::EBUSY,
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

unsafe fn hfile_libcurl_set_callback_failure_errno() {
    let errno = crate::htslib_rs::c_compat::__errno_location();
    if *errno == 0 {
        *errno = libc::EIO;
    }
}

// original: share_lock (htslib/hfile_libcurl.c:309)
pub unsafe extern "C" fn hfile_libcurl_c_309_share_lock(
    _handle: *mut c_void,
    _data: c_int,
    _access: c_int,
    _userptr: *mut c_void,
) {
    libc::pthread_mutex_lock(std::ptr::addr_of_mut!(HFILE_LIBCURL_SHARE_LOCK));
}

// original: share_unlock (htslib/hfile_libcurl.c:314)
pub unsafe extern "C" fn hfile_libcurl_c_314_share_unlock(
    _handle: *mut c_void,
    _data: c_int,
    _userptr: *mut c_void,
) {
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
pub unsafe extern "C" fn hfile_libcurl_c_326_libcurl_exit() {
    if !HFILE_LIBCURL_SHARE.is_null() && curl_share_cleanup(HFILE_LIBCURL_SHARE) == CURLSHE_OK {
        HFILE_LIBCURL_SHARE = std::ptr::null_mut();
    }

    libc::free(HFILE_LIBCURL_USERAGENT.s.cast());
    HFILE_LIBCURL_USERAGENT.l = 0;
    HFILE_LIBCURL_USERAGENT.m = 0;
    HFILE_LIBCURL_USERAGENT.s = std::ptr::null_mut();

    libc::free(HFILE_LIBCURL_AUTH_PATH.cast());
    HFILE_LIBCURL_AUTH_PATH = std::ptr::null_mut();

    if !HFILE_LIBCURL_AUTH_MAP.is_null() {
        let mut map = Box::from_raw(HFILE_LIBCURL_AUTH_MAP);
        for tok in map.drain(..) {
            hfile_libcurl_c_318_free_auth(tok as *mut c_void);
        }
        HFILE_LIBCURL_AUTH_MAP = std::ptr::null_mut();
    }
    curl_global_cleanup();
}

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
pub unsafe fn hfile_libcurl_c_454_read_auth_json(tok: *mut c_void, auth_fp: *mut hFILE) -> c_int {
    let tok = tok.cast::<HFileLibcurlAuthToken>();
    let t = hts_json_alloc_token();
    let mut str_: kstring_t = std::mem::zeroed();
    let mut token: *mut c_char = std::ptr::null_mut();
    let mut type_: *mut c_char = std::ptr::null_mut();
    let mut expiry: *mut c_char = std::ptr::null_mut();
    let mut ret = b'i' as c_int;

    if t.is_null() {
        ret = b'm' as c_int;
        goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret)
    } else {
        if hts_json_fnext(auth_fp, t, &mut str_) != b'{' as c_char {
            return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
        }
        while hts_json_fnext(auth_fp, t, &mut str_) != b'}' as c_char {
            if hts_json_token_type(t) != b's' as c_char {
                ret = b'?' as c_int;
                return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
            }
            let key = hts_json_token_str(t);
            if key.is_null() {
                ret = b'm' as c_int;
                return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
            }
            if libc::strcmp(key, c"access_token".as_ptr()) == 0 {
                ret = hts_json_fnext(auth_fp, t, &mut str_) as c_int;
                if ret != b's' as c_int {
                    return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
                }
                token = ks_release(&mut str_);
            } else if libc::strcmp(key, c"token_type".as_ptr()) == 0 {
                ret = hts_json_fnext(auth_fp, t, &mut str_) as c_int;
                if ret != b's' as c_int {
                    return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
                }
                type_ = ks_release(&mut str_);
            } else if libc::strcmp(key, c"expires_in".as_ptr()) == 0 {
                ret = hts_json_fnext(auth_fp, t, &mut str_) as c_int;
                if ret != b'n' as c_int {
                    return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
                }
                expiry = ks_release(&mut str_);
            } else if hts_json_fskip_value(auth_fp, 0) != b'v' as c_char {
                ret = b'?' as c_int;
                return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
            }
        }

        if token.is_null() || (!type_.is_null() && libc::strcmp(type_, c"Bearer".as_ptr()) != 0) {
            return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, b'i' as c_int);
        }

        ret = b'm' as c_int;
        str_.l = 0;
        if kputs(c"Authorization: Bearer ".as_ptr(), &mut str_) < 0 || kputs(token, &mut str_) < 0 {
            return goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, ret);
        }
        libc::free((*tok).token.cast());
        (*tok).token = ks_release(&mut str_);
        if !expiry.is_null() {
            let mut exp = libc::strtol(expiry, std::ptr::null_mut(), 10);
            if exp < 0 {
                exp = 0;
            }
            (*tok).expiry = libc::time(std::ptr::null_mut()) + exp;
        } else {
            (*tok).expiry = 0;
        }
        goto_auth_json_error(tok, t, &mut str_, token, type_, expiry, b'v' as c_int)
    }
}

unsafe fn goto_auth_json_error(
    _tok: *mut HFileLibcurlAuthToken,
    t: *mut crate::htslib_rs::hts::hts_json_token,
    str_: *mut kstring_t,
    token: *mut c_char,
    type_: *mut c_char,
    expiry: *mut c_char,
    ret: c_int,
) -> c_int {
    libc::free(token.cast());
    libc::free(type_.cast());
    libc::free(expiry.cast());
    libc::free((*str_).s.cast());
    hts_json_free_token(t);
    ret
}

// original: read_auth_plain (htslib/hfile_libcurl.c:515)
pub unsafe fn hfile_libcurl_c_515_read_auth_plain(tok: *mut c_void, auth_fp: *mut hFILE) -> c_int {
    let tok = tok.cast::<HFileLibcurlAuthToken>();
    let mut line: kstring_t = std::mem::zeroed();
    let mut token: kstring_t = std::mem::zeroed();

    if crate::htslib_rs::hfile::khgetline(&mut line, auth_fp) < 0 {
        libc::free(line.s.cast());
        libc::free(token.s.cast());
        return -1;
    }
    if crate::htslib_rs::hts::kputc(0, &mut line) < 0 {
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
        if crate::htslib_rs::hts::kputs(c"Authorization: Bearer ".as_ptr(), &mut token) < 0
            || crate::htslib_rs::hts::kputsn(start, end.offset_from(start) as usize, &mut token) < 0
        {
            libc::free(line.s.cast());
            libc::free(token.s.cast());
            return -1;
        }
    }

    libc::free((*tok).token.cast());
    (*tok).token = crate::htslib_rs::hts::ks_release(&mut token);
    (*tok).expiry = 0;
    libc::free(line.s.cast());
    0
}

// original: renew_auth_token (htslib/hfile_libcurl.c:543)
pub unsafe fn hfile_libcurl_c_543_renew_auth_token(tok: *mut c_void, changed: *mut c_int) -> c_int {
    let tok = tok.cast::<HFileLibcurlAuthToken>();
    let mut buffer = [0 as c_char; 16];

    *changed = 0;
    if (*tok).expiry == 0
        || libc::time(std::ptr::null_mut()) + AUTH_REFRESH_EARLY_SECS < (*tok).expiry
    {
        return 0;
    }
    if (*tok).failed != 0 {
        return -1;
    }

    *changed = 1;
    let auth_fp = hopen((*tok).path, c"rR".as_ptr());
    if auth_fp.is_null() {
        if *crate::htslib_rs::c_compat::__errno_location() != libc::ENOENT {
            (*tok).failed = 1;
            return -1;
        }
        (*tok).expiry = 0;
        libc::free((*tok).token.cast());
        (*tok).token = std::ptr::null_mut();
        return 0;
    }

    let len = hpeek(auth_fp, buffer.as_mut_ptr().cast(), buffer.len());
    if len < 0 {
        (*tok).failed = 1;
        hclose_abruptly(auth_fp);
        return -1;
    }

    let ok = if !libc::memchr(buffer.as_ptr().cast(), b'{' as c_int, len as usize).is_null() {
        hfile_libcurl_c_454_read_auth_json(tok.cast(), auth_fp) == b'v' as c_int
    } else {
        hfile_libcurl_c_515_read_auth_plain(tok.cast(), auth_fp) >= 0
    };
    if !ok {
        (*tok).failed = 1;
        hclose_abruptly(auth_fp);
        return -1;
    }

    if hclose(auth_fp) < 0 {
        -1
    } else {
        0
    }
}

// original: add_auth_header (htslib/hfile_libcurl.c:587)
pub unsafe fn hfile_libcurl_c_587_add_auth_header(fp: *mut c_void) -> c_int {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    let mut changed = 0;

    if (*fp).headers.auth_hdr_num < 0 || (*fp).headers.auth.is_null() {
        return 0;
    }

    libc::pthread_mutex_lock(&mut (*(*fp).headers.auth).lock);
    if hfile_libcurl_c_543_renew_auth_token((*fp).headers.auth.cast(), &mut changed) < 0 {
        libc::pthread_mutex_unlock(&mut (*(*fp).headers.auth).lock);
        return -1;
    }

    if changed == 0 && (*fp).headers.auth_hdr_num > 0 {
        libc::pthread_mutex_unlock(&mut (*(*fp).headers.auth).lock);
        return 0;
    }

    if (*fp).headers.auth_hdr_num > 0 {
        let header = (*(*fp).headers.auth).token;
        let header_copy = if !header.is_null() {
            libc::strdup(header)
        } else {
            std::ptr::null_mut()
        };
        let idx = ((*fp).headers.auth_hdr_num - 1) as usize;
        if !header.is_null() && header_copy.is_null() {
            libc::pthread_mutex_unlock(&mut (*(*fp).headers.auth).lock);
            return -1;
        }

        libc::free((*(*fp).headers.extra.list.add(idx)).data.cast());
        if !header_copy.is_null() {
            (*(*fp).headers.extra.list.add(idx)).data = header_copy;
        } else {
            let mut j = idx + 1;
            while j < (*fp).headers.extra.num as usize {
                *(*fp).headers.extra.list.add(j - 1) = *(*fp).headers.extra.list.add(j);
                (*(*fp).headers.extra.list.add(j - 1)).next = (*fp).headers.extra.list.add(j);
                j += 1;
            }
            (*fp).headers.extra.num -= 1;
            if (*fp).headers.extra.num > 0 {
                (*(*fp)
                    .headers
                    .extra
                    .list
                    .add((*fp).headers.extra.num as usize - 1))
                .next = std::ptr::null_mut();
            } else if (*fp).headers.fixed.num > 0 {
                (*(*fp)
                    .headers
                    .fixed
                    .list
                    .add((*fp).headers.fixed.num as usize - 1))
                .next = std::ptr::null_mut();
            }
            (*fp).headers.auth_hdr_num = 0;
        }
    } else if !(*(*fp).headers.auth).token.is_null() {
        if hfile_libcurl_c_353_append_header(
            (&mut (*fp).headers.extra as *mut HFileLibcurlHdrList).cast(),
            (*(*fp).headers.auth).token,
            1,
        ) < 0
        {
            libc::pthread_mutex_unlock(&mut (*(*fp).headers.auth).lock);
            return -1;
        }
        (*fp).headers.auth_hdr_num = (*fp).headers.extra.num as c_int;
    }

    libc::pthread_mutex_unlock(&mut (*(*fp).headers.auth).lock);
    0
}

// original: get_auth_token (htslib/hfile_libcurl.c:650)
pub unsafe fn hfile_libcurl_c_650_get_auth_token(fp: *mut c_void, url: *const c_char) -> c_int {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    let mut name: kstring_t = std::mem::zeroed();

    if HFILE_LIBCURL_AUTH_PATH.is_null()
        || ((*fp).flags & HFILE_LIBCURL_IS_RECURSIVE) != 0
        || (*fp).headers.auth_hdr_num != 0
    {
        return 0;
    }
    if HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER == 0
        && libc::strncmp(url, c"https://".as_ptr(), 8) != 0
    {
        return 0;
    }

    let mut host = libc::strstr(url, c"://".as_ptr());
    let mut host_len = 0usize;
    if !host.is_null() {
        host = host.add(3);
        host_len = libc::strcspn(host, c"/".as_ptr());
    }

    let mut p = HFILE_LIBCURL_AUTH_PATH;
    loop {
        let q = libc::strstr(p, c"%h".as_ptr());
        if q.is_null() {
            break;
        }
        if q.offset_from(p) > c_int::MAX as isize || host_len > c_int::MAX as usize {
            libc::free(name.s.cast());
            return -1;
        }
        if kputsn(p, q.offset_from(p) as usize, &mut name) < 0
            || kputsn(host, host_len, &mut name) < 0
        {
            libc::free(name.s.cast());
            return -1;
        }
        p = q.add(2);
    }
    if kputs(p, &mut name) < 0 {
        libc::free(name.s.cast());
        return -1;
    }

    libc::pthread_mutex_lock(std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_LOCK));
    if HFILE_LIBCURL_AUTH_MAP.is_null() {
        HFILE_LIBCURL_AUTH_MAP = Box::into_raw(Box::new(Vec::new()));
    }
    let map = &mut *HFILE_LIBCURL_AUTH_MAP;
    let mut tok: *mut HFileLibcurlAuthToken = std::ptr::null_mut();
    for entry in map.iter().copied() {
        let candidate = entry as *mut HFileLibcurlAuthToken;
        if libc::strcmp((*candidate).path, name.s) == 0 {
            tok = candidate;
            break;
        }
    }
    if tok.is_null() {
        tok = libc::calloc(1, std::mem::size_of::<HFileLibcurlAuthToken>())
            .cast::<HFileLibcurlAuthToken>();
        if !tok.is_null() && libc::pthread_mutex_init(&mut (*tok).lock, std::ptr::null()) != 0 {
            libc::free(tok.cast());
            tok = std::ptr::null_mut();
        }
        if !tok.is_null() {
            (*tok).path = ks_release(&mut name);
            (*tok).expiry = 1;
            map.push(tok as usize);
        }
    }
    libc::pthread_mutex_unlock(std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_LOCK));

    (*fp).headers.auth = tok;
    libc::free(name.s.cast());
    if tok.is_null() {
        -1
    } else {
        hfile_libcurl_c_587_add_auth_header(fp.cast())
    }
}

// original: process_messages (htslib/hfile_libcurl.c:718)
pub unsafe fn hfile_libcurl_c_718_process_messages(fp: *mut c_void) {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    let mut remaining = 0;
    loop {
        let msg = curl_multi_info_read((*fp).multi, &mut remaining);
        if msg.is_null() {
            break;
        }
        if (*msg).msg == CURLMSG_DONE {
            (*fp).flags |= HFILE_LIBCURL_FINISHED;
            (*fp).final_result = (*msg).data.result;
        }
    }
}

// original: wait_perform (htslib/hfile_libcurl.c:736)
pub unsafe fn hfile_libcurl_c_736_wait_perform(fp: *mut c_void) -> c_int {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    if ((*fp).flags & HFILE_LIBCURL_PERFORM_AGAIN) == 0 {
        let mut timeout: libc::c_long = 1000;
        if curl_multi_timeout((*fp).multi, &mut timeout) != CURLM_OK || timeout < 0 {
            timeout = 1000;
        }
        if timeout > 100 {
            timeout = 100;
        }
        let mut numfds = 0;
        let errm = curl_multi_wait(
            (*fp).multi,
            std::ptr::null_mut(),
            0,
            timeout as c_int,
            &mut numfds,
        );
        if errm != CURLM_OK {
            *crate::htslib_rs::c_compat::__errno_location() = hfile_libcurl_c_270_multi_errno(errm);
            return -1;
        }
    }

    let mut nrunning = 0;
    let errm = curl_multi_perform((*fp).multi, &mut nrunning);
    (*fp).flags &= !HFILE_LIBCURL_PERFORM_AGAIN;
    if errm == CURLM_CALL_MULTI_PERFORM {
        (*fp).flags |= HFILE_LIBCURL_PERFORM_AGAIN;
    } else if errm != CURLM_OK {
        *crate::htslib_rs::c_compat::__errno_location() = hfile_libcurl_c_270_multi_errno(errm);
        return -1;
    }
    if nrunning < (*fp).nrunning {
        hfile_libcurl_c_718_process_messages(fp.cast());
    }
    0
}

// original: recv_callback (htslib/hfile_libcurl.c:789)
pub unsafe extern "C" fn hfile_libcurl_c_789_recv_callback(
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
pub unsafe extern "C" fn hfile_libcurl_c_807_header_callback(
    contents: *mut c_void,
    size: usize,
    nmemb: usize,
    userp: *mut c_void,
) -> usize {
    let realsize = size.saturating_mul(nmemb);
    let resp = userp.cast::<kstring_t>();

    if crate::htslib_rs::hts::kputsn(contents.cast(), realsize, resp) == libc::EOF {
        0
    } else {
        realsize
    }
}

// original: refresh_retry_config (htslib/hfile_libcurl.c:821)
pub unsafe fn hfile_libcurl_c_821_refresh_retry_config() {
    let max = libc::getenv(c"HTS_RETRY_MAX".as_ptr());
    HFILE_LIBCURL_RETRY_MAX = if max.is_null() { 0 } else { libc::atoi(max) };
    let delay = libc::getenv(c"HTS_RETRY_DELAY".as_ptr());
    HFILE_LIBCURL_RETRY_DELAY_MS = if delay.is_null() {
        1000
    } else {
        libc::atol(delay)
    };
    if HFILE_LIBCURL_RETRY_MAX < 0 {
        HFILE_LIBCURL_RETRY_MAX = 0;
    }
    if HFILE_LIBCURL_RETRY_DELAY_MS < 0 {
        HFILE_LIBCURL_RETRY_DELAY_MS = 0;
    }
}

// original: retry_sleep (htslib/hfile_libcurl.c:836)
pub unsafe fn hfile_libcurl_c_836_retry_sleep(delay_ms: libc::c_long) {
    let ts = libc::timespec {
        tv_sec: delay_ms / 1000,
        tv_nsec: (delay_ms % 1000) * 1_000_000,
    };
    libc::nanosleep(&ts, std::ptr::null_mut());
}

// original: retry_reconnect (htslib/hfile_libcurl.c:848)
pub unsafe fn hfile_libcurl_c_848_retry_reconnect(fp: *mut c_void, pos: libc::off_t) -> c_int {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    hfile_libcurl_c_821_refresh_retry_config();
    let mut attempt = 0;
    while attempt < HFILE_LIBCURL_RETRY_MAX {
        hfile_libcurl_c_836_retry_sleep(HFILE_LIBCURL_RETRY_DELAY_MS);
        if hfile_libcurl_c_1134_restart_from_position(fp.cast(), pos) == 0 {
            return 0;
        }
        attempt += 1;
    }
    -1
}

// original: libcurl_read (htslib/hfile_libcurl.c:876)
pub unsafe extern "C" fn hfile_libcurl_c_876_libcurl_read(
    fpv: *mut hFILE,
    bufferv: *mut c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let fp = fpv.cast::<HFileLibcurlHeaderPrefix>();
    let buffer = bufferv.cast::<c_char>();
    let mut to_skip: libc::off_t = -1;
    let mut filled: libc::ssize_t = 0;

    if (*fp).delayed_seek >= 0 {
        if !(*fp).preserved.is_null()
            && (*fp).last_offset > (*fp).delayed_seek
            && (*fp).last_offset - (*fp).preserved_bytes as libc::off_t <= (*fp).delayed_seek
        {
            let n = ((*fp).last_offset - (*fp).delayed_seek) as usize;
            let start = (*fp).preserved.add((*fp).preserved_bytes - n);
            let bytes = n.min(nbytes);
            libc::memcpy(buffer.cast(), start.cast(), bytes);
            if bytes < n {
                (*fp).delayed_seek += bytes as libc::off_t;
            } else {
                (*fp).last_offset = -1;
                (*fp).delayed_seek = -1;
            }
            return bytes as libc::ssize_t;
        }

        if (*fp).last_offset >= 0
            && (*fp).delayed_seek > (*fp).last_offset
            && (*fp).delayed_seek - (*fp).last_offset < MIN_SEEK_FORWARD
        {
            to_skip = (*fp).delayed_seek - (*fp).last_offset;
        } else if hfile_libcurl_c_1134_restart_from_position(fp.cast(), (*fp).delayed_seek) < 0 {
            return -1;
        }
        (*fp).delayed_seek = -1;
        (*fp).last_offset = -1;
        (*fp).preserved_bytes = 0;
    }

    loop {
        if filled as usize >= nbytes {
            return filled;
        }
        let chunk_start = buffer.add(filled as usize);
        (*fp).buffer.ptr = chunk_start;
        (*fp).buffer.len = nbytes - filled as usize;
        (*fp).flags &= !HFILE_LIBCURL_PAUSED;
        if ((*fp).flags & HFILE_LIBCURL_FINISHED) == 0 {
            let err = curl_easy_pause((*fp).easy, CURLPAUSE_CONT);
            if err != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno((*fp).easy, err);
                return -1;
            }
        }

        while ((*fp).flags & HFILE_LIBCURL_PAUSED) == 0
            && ((*fp).flags & HFILE_LIBCURL_FINISHED) == 0
        {
            if hfile_libcurl_c_736_wait_perform(fp.cast()) < 0 {
                return -1;
            }
        }

        let mut got = (*fp).buffer.ptr.offset_from(chunk_start) as libc::ssize_t;
        if to_skip >= 0 {
            if got <= to_skip as libc::ssize_t {
                to_skip -= got as libc::off_t;
                got = 0;
            } else {
                got -= to_skip as libc::ssize_t;
                if got > 0 {
                    libc::memmove(
                        buffer.add(filled as usize).cast(),
                        chunk_start.add(to_skip as usize).cast(),
                        got as usize,
                    );
                    to_skip = -1;
                }
            }
        }

        (*fp).buffer.ptr = std::ptr::null_mut();
        (*fp).buffer.len = 0;
        filled += got;

        if ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 && (*fp).final_result != CURLE_OK {
            let err = (*fp).final_result;
            let pos = (*fp).base.offset + filled as libc::off_t;
            if hfile_libcurl_c_233_is_retryable((*fp).easy, err) != 0
                && hfile_libcurl_c_848_retry_reconnect(fp.cast(), pos) == 0
            {
                continue;
            }
            *crate::htslib_rs::c_compat::__errno_location() =
                hfile_libcurl_c_153_easy_errno((*fp).easy, err);
            return -1;
        }

        if to_skip < 0 || ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 {
            return filled;
        }
    }
}

// original: send_callback (htslib/hfile_libcurl.c:1006)
pub unsafe extern "C" fn hfile_libcurl_c_1006_send_callback(
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
pub unsafe extern "C" fn hfile_libcurl_c_1024_libcurl_write(
    fpv: *mut hFILE,
    bufferv: *const c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let fp = fpv.cast::<HFileLibcurlHeaderPrefix>();
    let buffer = bufferv.cast::<c_char>();
    (*fp).buffer.ptr = buffer.cast_mut();
    (*fp).buffer.len = nbytes;
    (*fp).flags &= !HFILE_LIBCURL_PAUSED;
    let err = curl_easy_pause((*fp).easy, CURLPAUSE_CONT);
    if err != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno((*fp).easy, err);
        return -1;
    }
    while ((*fp).flags & HFILE_LIBCURL_PAUSED) == 0 && ((*fp).flags & HFILE_LIBCURL_FINISHED) == 0 {
        if hfile_libcurl_c_736_wait_perform(fp.cast()) < 0 {
            return -1;
        }
    }
    let done = (*fp).buffer.ptr.offset_from(buffer) as libc::ssize_t;
    (*fp).buffer.ptr = std::ptr::null_mut();
    (*fp).buffer.len = 0;
    if ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 && (*fp).final_result != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno((*fp).easy, (*fp).final_result);
        return -1;
    }
    done
}

// original: preserve_buffer_content (htslib/hfile_libcurl.c:1051)
pub unsafe fn hfile_libcurl_c_1051_preserve_buffer_content(fp: *mut c_void) {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    if (*fp).base.begin == (*fp).base.end {
        (*fp).preserved_bytes = 0;
        return;
    }
    let cap = (*fp).base.limit.offset_from((*fp).base.buffer) as usize;
    if (*fp).preserved.is_null() || (*fp).preserved_size < cap {
        let preserved = libc::malloc(cap).cast::<c_char>();
        if preserved.is_null() {
            return;
        }
        libc::free((*fp).preserved.cast());
        (*fp).preserved = preserved;
        (*fp).preserved_size = cap;
    }
    let n = (*fp).base.end.offset_from((*fp).base.begin) as usize;
    libc::memcpy((*fp).preserved.cast(), (*fp).base.begin.cast(), n);
    (*fp).preserved_bytes = n;
}

// original: libcurl_seek (htslib/hfile_libcurl.c:1071)
pub unsafe extern "C" fn hfile_libcurl_c_1071_libcurl_seek(
    fpv: *mut hFILE,
    offset: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    let fp = fpv.cast::<HFileLibcurlHeaderPrefix>();
    if ((*fp).flags & HFILE_LIBCURL_IS_READ) == 0 || ((*fp).flags & HFILE_LIBCURL_CAN_SEEK) == 0 {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
        return -1;
    }
    let origin = match whence {
        libc::SEEK_SET => 0,
        libc::SEEK_CUR => {
            let curpos =
                (*fp).base.offset + (*fp).base.begin.offset_from((*fp).base.buffer) as libc::off_t;
            match curpos.checked_add(offset) {
                Some(pos) if pos >= 0 => pos,
                _ => {
                    let err = if offset < 0 {
                        libc::EINVAL
                    } else {
                        crate::htslib_rs::c_compat::EOVERFLOW
                    };
                    *crate::htslib_rs::c_compat::__errno_location() = err;
                    return -1;
                }
            }
        }
        libc::SEEK_END => {
            if (*fp).file_size < 0 {
                *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
                return -1;
            }
            (*fp).file_size
        }
        _ => {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return -1;
        }
    };
    if (offset < 0 && origin + offset < 0)
        || (offset >= 0 && (*fp).file_size >= 0 && offset > (*fp).file_size - origin)
    {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    let pos = origin + offset;
    if ((*fp).flags & HFILE_LIBCURL_TRIED_SEEK) != 0 {
        if (*fp).delayed_seek < 0 {
            (*fp).last_offset =
                (*fp).base.offset + (*fp).base.end.offset_from((*fp).base.buffer) as libc::off_t;
            hfile_libcurl_c_1051_preserve_buffer_content(fp.cast());
        }
        (*fp).delayed_seek = pos;
        return pos;
    }
    if hfile_libcurl_c_1134_restart_from_position(fp.cast(), pos) < 0 {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
        return -1;
    }
    (*fp).flags |= HFILE_LIBCURL_TRIED_SEEK;
    pos
}

// original: restart_from_position (htslib/hfile_libcurl.c:1134)
pub unsafe fn hfile_libcurl_c_1134_restart_from_position(
    fp: *mut c_void,
    pos: libc::off_t,
) -> c_int {
    let fp = fp.cast::<HFileLibcurlHeaderPrefix>();
    let mut temp_fp: HFileLibcurlHeaderPrefix = std::ptr::read(fp);
    let save_errno: c_int;
    let mut update_headers = 0;

    if (*fp).headers.callback.is_some() {
        if hfile_libcurl_c_399_add_callback_headers(fp.cast()) != 0 {
            return -1;
        }
        update_headers = 1;
    }
    if (*fp).headers.auth_hdr_num > 0 && !(*fp).headers.auth.is_null() {
        if hfile_libcurl_c_587_add_auth_header(fp.cast()) != 0 {
            return -1;
        }
        update_headers = 1;
    }
    if update_headers != 0 {
        let list = hfile_libcurl_c_387_get_header_list(fp.cast());
        if !list.is_null() {
            let err = curl_easy_setopt((*fp).easy, CURLOPT_HTTPHEADER, list);
            if err != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno((*fp).easy, err);
                return -1;
            }
        }
    }

    temp_fp.buffer.len = 0;
    temp_fp.buffer.ptr = std::ptr::null_mut();
    temp_fp.easy = curl_easy_duphandle((*fp).easy);
    if temp_fp.easy.is_null() {
        (*fp).flags &= !HFILE_LIBCURL_CAN_SEEK;
        return -1;
    }

    let mut err = curl_easy_setopt(temp_fp.easy, CURLOPT_RESUME_FROM_LARGE, pos);
    err |= curl_easy_setopt(
        temp_fp.easy,
        CURLOPT_PRIVATE,
        (&mut temp_fp as *mut HFileLibcurlHeaderPrefix).cast::<c_void>(),
    );
    err |= curl_easy_setopt(
        temp_fp.easy,
        CURLOPT_WRITEDATA,
        (&mut temp_fp as *mut HFileLibcurlHeaderPrefix).cast::<c_void>(),
    );
    if err != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno(temp_fp.easy, err);
        curl_easy_cleanup(temp_fp.easy);
        (*fp).flags &= !HFILE_LIBCURL_CAN_SEEK;
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }

    temp_fp.buffer.len = 0;
    temp_fp.flags &= !(HFILE_LIBCURL_PAUSED | HFILE_LIBCURL_FINISHED);
    let mut errm = curl_multi_add_handle((*fp).multi, temp_fp.easy);
    if errm != CURLM_OK {
        save_errno = hfile_libcurl_c_270_multi_errno(errm);
        curl_easy_cleanup(temp_fp.easy);
        (*fp).flags &= !HFILE_LIBCURL_CAN_SEEK;
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }
    (*fp).nrunning += 1;
    temp_fp.nrunning = (*fp).nrunning;

    while (temp_fp.flags & HFILE_LIBCURL_PAUSED) == 0
        && (temp_fp.flags & HFILE_LIBCURL_FINISHED) == 0
    {
        if hfile_libcurl_c_736_wait_perform((&mut temp_fp as *mut HFileLibcurlHeaderPrefix).cast())
            < 0
        {
            save_errno = *crate::htslib_rs::c_compat::__errno_location();
            errm = curl_multi_remove_handle((*fp).multi, temp_fp.easy);
            if errm == CURLM_OK {
                (*fp).nrunning -= 1;
            }
            curl_easy_cleanup(temp_fp.easy);
            (*fp).flags &= !HFILE_LIBCURL_CAN_SEEK;
            *crate::htslib_rs::c_compat::__errno_location() = save_errno;
            return -1;
        }
    }
    if (temp_fp.flags & HFILE_LIBCURL_FINISHED) != 0 && temp_fp.final_result != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno(temp_fp.easy, temp_fp.final_result);
        curl_multi_remove_handle((*fp).multi, temp_fp.easy);
        (*fp).nrunning -= 1;
        curl_easy_cleanup(temp_fp.easy);
        (*fp).flags &= !HFILE_LIBCURL_CAN_SEEK;
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }

    errm = curl_multi_remove_handle((*fp).multi, (*fp).easy);
    if errm != CURLM_OK {
        curl_multi_remove_handle((*fp).multi, temp_fp.easy);
        (*fp).nrunning -= 1;
        curl_easy_cleanup(temp_fp.easy);
        *crate::htslib_rs::c_compat::__errno_location() = hfile_libcurl_c_270_multi_errno(errm);
        return -1;
    }
    (*fp).nrunning -= 1;
    curl_easy_cleanup((*fp).easy);
    (*fp).easy = temp_fp.easy;
    err = curl_easy_setopt((*fp).easy, CURLOPT_WRITEDATA, fp.cast::<c_void>());
    err |= curl_easy_setopt((*fp).easy, CURLOPT_PRIVATE, fp.cast::<c_void>());
    if err != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno((*fp).easy, err);
        curl_easy_reset((*fp).easy);
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }
    (*fp).buffer.len = 0;
    (*fp).flags = ((*fp).flags
        & !(HFILE_LIBCURL_PAUSED | HFILE_LIBCURL_FINISHED | HFILE_LIBCURL_PERFORM_AGAIN))
        | (temp_fp.flags
            & (HFILE_LIBCURL_PAUSED | HFILE_LIBCURL_FINISHED | HFILE_LIBCURL_PERFORM_AGAIN));
    (*fp).final_result = temp_fp.final_result;
    0
}

// original: libcurl_close (htslib/hfile_libcurl.c:1266)
pub unsafe extern "C" fn hfile_libcurl_c_1266_libcurl_close(fpv: *mut hFILE) -> c_int {
    let fp = fpv.cast::<HFileLibcurlHeaderPrefix>();
    let mut save_errno = 0;
    (*fp).buffer.len = 0;
    (*fp).flags |= HFILE_LIBCURL_CLOSING;
    (*fp).flags &= !HFILE_LIBCURL_PAUSED;
    if ((*fp).flags & HFILE_LIBCURL_FINISHED) == 0 {
        let err = curl_easy_pause((*fp).easy, CURLPAUSE_CONT);
        if err != CURLE_OK {
            save_errno = hfile_libcurl_c_153_easy_errno((*fp).easy, err);
        }
    }
    while save_errno == 0
        && ((*fp).flags & HFILE_LIBCURL_PAUSED) == 0
        && ((*fp).flags & HFILE_LIBCURL_FINISHED) == 0
    {
        if hfile_libcurl_c_736_wait_perform(fp.cast()) < 0 {
            save_errno = *crate::htslib_rs::c_compat::__errno_location();
        }
    }
    if ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 && (*fp).final_result != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno((*fp).easy, (*fp).final_result);
    }
    let errm = curl_multi_remove_handle((*fp).multi, (*fp).easy);
    if errm != CURLM_OK && save_errno == 0 {
        save_errno = hfile_libcurl_c_270_multi_errno(errm);
    }
    (*fp).nrunning -= 1;
    curl_easy_cleanup((*fp).easy);
    curl_multi_cleanup((*fp).multi);
    if let Some(callback) = (*fp).headers.callback {
        callback((*fp).headers.callback_data, std::ptr::null_mut());
    }
    hfile_libcurl_c_372_free_headers(
        (&mut (*fp).headers.fixed as *mut HFileLibcurlHdrList).cast(),
        1,
    );
    hfile_libcurl_c_372_free_headers(
        (&mut (*fp).headers.extra as *mut HFileLibcurlHdrList).cast(),
        1,
    );
    libc::free((*fp).preserved.cast());
    if save_errno != 0 {
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        -1
    } else {
        0
    }
}

// original: libcurl_open (htslib/hfile_libcurl.c:1313)
pub unsafe fn hfile_libcurl_c_1313_libcurl_open(
    url: *const c_char,
    modes: *const c_char,
    headers: *mut HFileLibcurlHeaders,
) -> *mut hFILE {
    let mut s = libc::strpbrk(modes, c"rwa+".as_ptr());
    let mode = if !s.is_null() {
        let m = *s;
        s = s.add(1);
        if !libc::strpbrk(s, c"rwa+".as_ptr()).is_null() {
            b'e' as c_char
        } else {
            m
        }
    } else {
        0
    };
    if mode != b'r' as c_char && mode != b'w' as c_char {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    }

    hfile_libcurl_c_821_refresh_retry_config();
    let mut attempt = 0;
    loop {
        let fp = hfile_libcurl_open_once(url, modes, headers, mode);
        if !fp.is_null() {
            return fp;
        }
        let err = *crate::htslib_rs::c_compat::__errno_location();
        if attempt >= HFILE_LIBCURL_RETRY_MAX
            || !matches!(
                err,
                libc::EBUSY | libc::ETIMEDOUT | libc::ECONNABORTED | libc::EPIPE | libc::EIO
            )
        {
            return std::ptr::null_mut();
        }
        attempt += 1;
        hfile_libcurl_c_836_retry_sleep(HFILE_LIBCURL_RETRY_DELAY_MS);
    }
}

unsafe fn hfile_libcurl_open_once(
    url: *const c_char,
    modes: *const c_char,
    headers: *mut HFileLibcurlHeaders,
    mode: c_char,
) -> *mut hFILE {
    let fp = hfile_init(std::mem::size_of::<HFileLibcurlHeaderPrefix>(), modes, 0)
        .cast::<HFileLibcurlHeaderPrefix>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    if !headers.is_null() {
        (*fp).headers = std::ptr::read(headers);
    } else {
        std::ptr::write_bytes(&mut (*fp).headers as *mut HFileLibcurlHeaders, 0, 1);
        (*fp).headers.fail_on_error = 1;
    }
    (*fp).file_size = -1;
    (*fp).buffer.ptr = std::ptr::null_mut();
    (*fp).buffer.len = 0;
    (*fp).final_result = -1;
    (*fp).flags = HFILE_LIBCURL_CAN_SEEK;
    if mode == b'r' as c_char {
        (*fp).flags |= HFILE_LIBCURL_IS_READ;
    }
    if !libc::strchr(modes, b'R' as c_int).is_null() {
        (*fp).flags |= HFILE_LIBCURL_IS_RECURSIVE;
    }
    (*fp).delayed_seek = -1;
    (*fp).last_offset = -1;
    (*fp).preserved = std::ptr::null_mut();
    (*fp).preserved_bytes = 0;
    (*fp).preserved_size = 0;
    (*fp).nrunning = 0;
    (*fp).easy = std::ptr::null_mut();
    (*fp).multi = curl_multi_init();
    if (*fp).multi.is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    (*fp).easy = curl_easy_init();
    if (*fp).easy.is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
        goto_open_error(fp);
        return std::ptr::null_mut();
    }

    let mut err = curl_easy_setopt((*fp).easy, CURLOPT_PRIVATE, fp.cast::<c_void>());
    err |= curl_easy_setopt((*fp).easy, CURLOPT_FTP_FILEMETHOD, CURLFTPMETHOD_NOCWD);
    if mode == b'r' as c_char {
        err |= curl_easy_setopt(
            (*fp).easy,
            CURLOPT_WRITEFUNCTION,
            hfile_libcurl_c_789_recv_callback
                as unsafe extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize,
        );
        err |= curl_easy_setopt((*fp).easy, CURLOPT_WRITEDATA, fp.cast::<c_void>());
    } else {
        err |= curl_easy_setopt(
            (*fp).easy,
            CURLOPT_READFUNCTION,
            hfile_libcurl_c_1006_send_callback
                as unsafe extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize,
        );
        err |= curl_easy_setopt((*fp).easy, CURLOPT_READDATA, fp.cast::<c_void>());
        err |= curl_easy_setopt((*fp).easy, CURLOPT_UPLOAD, 1 as libc::c_long);
        if hfile_libcurl_c_353_append_header(
            (&mut (*fp).headers.fixed as *mut HFileLibcurlHdrList).cast(),
            c"Transfer-Encoding: chunked".as_ptr(),
            1,
        ) < 0
        {
            goto_open_error(fp);
            return std::ptr::null_mut();
        }
    }
    err |= curl_easy_setopt((*fp).easy, CURLOPT_SHARE, HFILE_LIBCURL_SHARE);
    err |= curl_easy_setopt((*fp).easy, CURLOPT_URL, url);
    let ca = libc::getenv(c"CURL_CA_BUNDLE".as_ptr());
    if !ca.is_null() {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_CAINFO, ca);
    }
    err |= curl_easy_setopt((*fp).easy, CURLOPT_USERAGENT, HFILE_LIBCURL_USERAGENT.s);
    if (*fp).headers.callback.is_some() && hfile_libcurl_c_399_add_callback_headers(fp.cast()) != 0
    {
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    if hfile_libcurl_c_650_get_auth_token(fp.cast(), url) < 0 {
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    let list = hfile_libcurl_c_387_get_header_list(fp.cast());
    if !list.is_null() {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_HTTPHEADER, list);
    }
    if hts_verbose <= 8 && (*fp).headers.fail_on_error != 0 {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_FAILONERROR, 1 as libc::c_long);
    }
    if hts_verbose >= 8 {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_VERBOSE, 1 as libc::c_long);
    }
    let mut in_header: kstring_t = std::mem::zeroed();
    if !(*fp).headers.redirect.is_null() {
        err |= curl_easy_setopt(
            (*fp).easy,
            CURLOPT_HEADERFUNCTION,
            hfile_libcurl_c_807_header_callback
                as unsafe extern "C" fn(*mut c_void, usize, usize, *mut c_void) -> usize,
        );
        err |= curl_easy_setopt(
            (*fp).easy,
            CURLOPT_HEADERDATA,
            (&mut in_header as *mut kstring_t).cast::<c_void>(),
        );
    } else {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_FOLLOWLOCATION, 1 as libc::c_long);
    }
    if err != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno((*fp).easy, err);
        goto_open_error(fp);
        return std::ptr::null_mut();
    }

    let errm = curl_multi_add_handle((*fp).multi, (*fp).easy);
    if errm != CURLM_OK {
        *crate::htslib_rs::c_compat::__errno_location() = hfile_libcurl_c_270_multi_errno(errm);
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    (*fp).nrunning += 1;
    while ((*fp).flags & HFILE_LIBCURL_PAUSED) == 0 && ((*fp).flags & HFILE_LIBCURL_FINISHED) == 0 {
        if hfile_libcurl_c_736_wait_perform(fp.cast()) < 0 {
            curl_multi_remove_handle((*fp).multi, (*fp).easy);
            (*fp).nrunning -= 1;
            goto_open_error(fp);
            return std::ptr::null_mut();
        }
    }

    let mut response: libc::c_long = 0;
    curl_easy_getinfo_long((*fp).easy, CURLINFO_RESPONSE_CODE, &mut response);
    if !(*fp).headers.http_response_ptr.is_null() {
        *(*fp).headers.http_response_ptr = response;
    }
    if ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 && (*fp).final_result != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno((*fp).easy, (*fp).final_result);
        curl_multi_remove_handle((*fp).multi, (*fp).easy);
        (*fp).nrunning -= 1;
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    if !(*fp).headers.redirect.is_null() {
        if response >= 300 && response < 400 {
            let mut new_url: kstring_t = std::mem::zeroed();
            let redirect: HFileLibcurlRedirectCallback =
                std::mem::transmute((*fp).headers.redirect);
            if redirect(
                (*fp).headers.redirect_data,
                response,
                &mut in_header,
                &mut new_url,
            ) != 0
            {
                hfile_libcurl_set_callback_failure_errno();
                libc::free(in_header.s.cast());
                libc::free(new_url.s.cast());
                goto_open_error(fp);
                return std::ptr::null_mut();
            }

            err = curl_easy_setopt((*fp).easy, CURLOPT_URL, new_url.s);
            err |= curl_easy_setopt(
                (*fp).easy,
                CURLOPT_HEADERFUNCTION,
                std::ptr::null::<c_void>(),
            );
            err |= curl_easy_setopt((*fp).easy, CURLOPT_HEADERDATA, std::ptr::null::<c_void>());
            libc::free(in_header.s.cast());
            libc::free(new_url.s.cast());
            if err != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno((*fp).easy, err);
                goto_open_error(fp);
                return std::ptr::null_mut();
            }
            if hfile_libcurl_c_1134_restart_from_position(fp.cast(), 0) < 0 {
                goto_open_error(fp);
                return std::ptr::null_mut();
            }
            if !(*fp).headers.http_response_ptr.is_null() {
                curl_easy_getinfo_long(
                    (*fp).easy,
                    CURLINFO_RESPONSE_CODE,
                    (*fp).headers.http_response_ptr,
                );
            }
            if ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 && (*fp).final_result != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno((*fp).easy, (*fp).final_result);
                curl_multi_remove_handle((*fp).multi, (*fp).easy);
                (*fp).nrunning -= 1;
                goto_open_error(fp);
                return std::ptr::null_mut();
            }
        } else {
            err = curl_easy_setopt(
                (*fp).easy,
                CURLOPT_HEADERFUNCTION,
                std::ptr::null::<c_void>(),
            );
            err |= curl_easy_setopt((*fp).easy, CURLOPT_HEADERDATA, std::ptr::null::<c_void>());
            libc::free(in_header.s.cast());
            if err != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno((*fp).easy, err);
                goto_open_error(fp);
                return std::ptr::null_mut();
            }
        }
    }
    if mode == b'r' as c_char {
        let mut offset: libc::off_t = 0;
        if curl_easy_getinfo_off_t((*fp).easy, CURLINFO_CONTENT_LENGTH_DOWNLOAD_T, &mut offset)
            == CURLE_OK
            && offset > 0
        {
            (*fp).file_size = offset;
        }
    }
    (*fp).base.backend = &LIBCURL_BACKEND;
    fp.cast()
}

unsafe fn goto_open_error(fp: *mut HFileLibcurlHeaderPrefix) {
    let save = *crate::htslib_rs::c_compat::__errno_location();
    if !fp.is_null() {
        if !(*fp).easy.is_null() {
            curl_easy_cleanup((*fp).easy);
        }
        if !(*fp).multi.is_null() {
            curl_multi_cleanup((*fp).multi);
        }
        hfile_libcurl_c_372_free_headers(
            (&mut (*fp).headers.extra as *mut HFileLibcurlHdrList).cast(),
            1,
        );
        hfile_destroy(fp.cast());
    }
    *crate::htslib_rs::c_compat::__errno_location() = save;
}

static LIBCURL_BACKEND: HFileBackend = HFileBackend {
    read: Some(hfile_libcurl_c_876_libcurl_read),
    write: Some(hfile_libcurl_c_1024_libcurl_write),
    seek: Some(hfile_libcurl_c_1071_libcurl_seek),
    flush: None,
    close: Some(hfile_libcurl_c_1266_libcurl_close),
};

// original: hopen_libcurl (htslib/hfile_libcurl.c:1549)
pub unsafe fn hfile_libcurl_c_1549_hopen_libcurl(
    url: *const c_char,
    modes: *const c_char,
) -> *mut hFILE {
    hfile_libcurl_c_1313_libcurl_open(url, modes, std::ptr::null_mut())
}

unsafe fn hfile_libcurl_va_arg_word(args: *mut hts_sys::__va_list_tag) -> usize {
    let args = &mut *args;
    if args.gp_offset <= 40 {
        let p = args.reg_save_area.cast::<u8>().add(args.gp_offset as usize);
        args.gp_offset += 8;
        std::ptr::read_unaligned(p.cast::<usize>())
    } else {
        let p = args.overflow_arg_area.cast::<u8>();
        args.overflow_arg_area = p.add(8).cast();
        std::ptr::read_unaligned(p.cast::<usize>())
    }
}

// original: parse_va_list (htslib/hfile_libcurl.c:1554)
pub unsafe fn hfile_libcurl_c_1554_parse_va_list(
    headers: *mut HFileLibcurlHeaders,
    args: *mut hts_sys::__va_list_tag,
) -> c_int {
    if headers.is_null() || args.is_null() {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }

    loop {
        let argtype = hfile_libcurl_va_arg_word(args) as *const c_char;
        if argtype.is_null() {
            return 0;
        }

        if libc::strcmp(argtype, c"httphdr:v".as_ptr()) == 0 {
            let mut hdr = hfile_libcurl_va_arg_word(args) as *mut *const c_char;
            if hdr.is_null() {
                continue;
            }
            while !(*hdr).is_null() {
                if hfile_libcurl_c_353_append_header(
                    (&mut (*headers).fixed as *mut HFileLibcurlHdrList).cast(),
                    *hdr,
                    1,
                ) < 0
                {
                    return -1;
                }
                if hfile_libcurl_c_395_is_authorization(*hdr) != 0 {
                    (*headers).auth_hdr_num = -1;
                }
                hdr = hdr.add(1);
            }
        } else if libc::strcmp(argtype, c"httphdr:l".as_ptr()) == 0 {
            loop {
                let hdr = hfile_libcurl_va_arg_word(args) as *const c_char;
                if hdr.is_null() {
                    break;
                }
                if hfile_libcurl_c_353_append_header(
                    (&mut (*headers).fixed as *mut HFileLibcurlHdrList).cast(),
                    hdr,
                    1,
                ) < 0
                {
                    return -1;
                }
                if hfile_libcurl_c_395_is_authorization(hdr) != 0 {
                    (*headers).auth_hdr_num = -1;
                }
            }
        } else if libc::strcmp(argtype, c"httphdr".as_ptr()) == 0 {
            let hdr = hfile_libcurl_va_arg_word(args) as *const c_char;
            if !hdr.is_null() {
                if hfile_libcurl_c_353_append_header(
                    (&mut (*headers).fixed as *mut HFileLibcurlHdrList).cast(),
                    hdr,
                    1,
                ) < 0
                {
                    return -1;
                }
                if hfile_libcurl_c_395_is_authorization(hdr) != 0 {
                    (*headers).auth_hdr_num = -1;
                }
            }
        } else if libc::strcmp(argtype, c"httphdr_callback".as_ptr()) == 0 {
            let callback = hfile_libcurl_va_arg_word(args);
            (*headers).callback = if callback == 0 {
                None
            } else {
                Some(std::mem::transmute::<usize, HFileLibcurlHttpHeaderCallback>(callback))
            };
        } else if libc::strcmp(argtype, c"httphdr_callback_data".as_ptr()) == 0 {
            (*headers).callback_data = hfile_libcurl_va_arg_word(args) as *mut c_void;
        } else if libc::strcmp(argtype, c"va_list".as_ptr()) == 0 {
            let args2 = hfile_libcurl_va_arg_word(args) as *mut hts_sys::__va_list_tag;
            if !args2.is_null() && hfile_libcurl_c_1554_parse_va_list(headers, args2) < 0 {
                return -1;
            }
        } else if libc::strcmp(argtype, c"auth_token_enabled".as_ptr()) == 0 {
            let flag = hfile_libcurl_va_arg_word(args) as *const c_char;
            if !flag.is_null() && libc::strcmp(flag, c"false".as_ptr()) == 0 {
                (*headers).auth_hdr_num = -3;
            }
        } else if libc::strcmp(argtype, c"redirect_callback".as_ptr()) == 0 {
            (*headers).redirect = hfile_libcurl_va_arg_word(args) as *mut c_void;
        } else if libc::strcmp(argtype, c"redirect_callback_data".as_ptr()) == 0 {
            (*headers).redirect_data = hfile_libcurl_va_arg_word(args) as *mut c_void;
        } else if libc::strcmp(argtype, c"http_response_ptr".as_ptr()) == 0 {
            (*headers).http_response_ptr = hfile_libcurl_va_arg_word(args) as *mut libc::c_long;
        } else if libc::strcmp(argtype, c"fail_on_error".as_ptr()) == 0 {
            (*headers).fail_on_error = hfile_libcurl_va_arg_word(args) as c_int;
        } else {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return -1;
        }
    }
}

// original: vhopen_libcurl (htslib/hfile_libcurl.c:1664)
pub unsafe extern "C" fn hfile_libcurl_c_1664_vhopen_libcurl(
    url: *const c_char,
    modes: *const c_char,
    args: *mut hts_sys::__va_list_tag,
) -> *mut hFILE {
    let mut headers: HFileLibcurlHeaders = std::mem::zeroed();
    headers.fail_on_error = 1;
    let fp = if hfile_libcurl_c_1554_parse_va_list(&mut headers, args) == 0 {
        hfile_libcurl_c_1313_libcurl_open(url, modes, &mut headers)
    } else {
        std::ptr::null_mut()
    };
    if fp.is_null() {
        hfile_libcurl_c_372_free_headers(
            (&mut headers.fixed as *mut HFileLibcurlHdrList).cast(),
            1,
        );
    }
    fp
}

// original: PLUGIN_GLOBAL (htslib/hfile_libcurl.c:1679)
pub unsafe fn hfile_libcurl_c_1679_PLUGIN_GLOBAL(self_: *mut hFILE_plugin) -> c_int {
    static HANDLER: HFileSchemeHandlerLayout = HFileSchemeHandlerLayout {
        open: Some(hfile_libcurl_open),
        isremote: Some(hfile_c_1342_hfile_always_remote),
        provider: c"libcurl".as_ptr(),
        priority: 2050,
        vopen: Some(hfile_libcurl_c_1664_vhopen_libcurl),
    };

    let err = curl_global_init(CURL_GLOBAL_ALL);
    if err != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno(std::ptr::null_mut(), err);
        return -1;
    }
    HFILE_LIBCURL_SHARE = curl_share_init();
    if HFILE_LIBCURL_SHARE.is_null() {
        curl_global_cleanup();
        *crate::htslib_rs::c_compat::__errno_location() = libc::EIO;
        return -1;
    }
    let mut errsh = curl_share_setopt(
        HFILE_LIBCURL_SHARE,
        CURLSHOPT_LOCKFUNC,
        hfile_libcurl_c_309_share_lock as *mut c_void,
    );
    errsh |= curl_share_setopt(
        HFILE_LIBCURL_SHARE,
        CURLSHOPT_UNLOCKFUNC,
        hfile_libcurl_c_314_share_unlock as *mut c_void,
    );
    errsh |= curl_share_setopt(
        HFILE_LIBCURL_SHARE,
        CURLSHOPT_SHARE,
        CURL_LOCK_DATA_DNS as libc::c_long,
    );
    if errsh != CURLSHE_OK {
        curl_share_cleanup(HFILE_LIBCURL_SHARE);
        HFILE_LIBCURL_SHARE = std::ptr::null_mut();
        curl_global_cleanup();
        *crate::htslib_rs::c_compat::__errno_location() = libc::EIO;
        return -1;
    }

    let auth = libc::getenv(c"HTS_AUTH_LOCATION".as_ptr());
    if !auth.is_null() {
        HFILE_LIBCURL_AUTH_PATH = libc::strdup(auth);
        HFILE_LIBCURL_AUTH_MAP = Box::into_raw(Box::new(Vec::new()));
        if HFILE_LIBCURL_AUTH_PATH.is_null() {
            hfile_libcurl_c_326_libcurl_exit();
            *crate::htslib_rs::c_compat::__errno_location() = libc::ENOMEM;
            return -1;
        }
    }
    let allow = libc::getenv(c"HTS_ALLOW_UNENCRYPTED_AUTHORIZATION_HEADER".as_ptr());
    if !allow.is_null() && libc::strcmp(allow, c"I understand the risks".as_ptr()) == 0 {
        HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER = 1;
    }

    let info = curl_version_info(CURLVERSION_NOW);
    let version = crate::htslib_rs::hts::hts_version();
    if !info.is_null() {
        hts_sys::ksprintf(
            std::ptr::addr_of_mut!(HFILE_LIBCURL_USERAGENT).cast(),
            c"htslib/%s libcurl/%s".as_ptr(),
            version,
            (*info).version,
        );
    }

    if !self_.is_null() {
        (*(self_.cast::<HFilePluginLayout>())).name = c"libcurl".as_ptr();
        (*(self_.cast::<HFilePluginLayout>())).destroy =
            hfile_libcurl_c_326_libcurl_exit as *const c_void;
    }

    if !info.is_null() && !(*info).protocols.is_null() {
        let mut protocol = (*info).protocols;
        while !(*protocol).is_null() {
            hfile_add_scheme_handler(
                *protocol,
                (&HANDLER as *const HFileSchemeHandlerLayout).cast::<hFILE_scheme_handler>(),
            );
            protocol = protocol.add(1);
        }
    } else {
        for scheme in [
            c"http".as_ptr(),
            c"https".as_ptr(),
            c"ftp".as_ptr(),
            c"ftps".as_ptr(),
        ] {
            hfile_add_scheme_handler(
                scheme,
                (&HANDLER as *const HFileSchemeHandlerLayout).cast::<hFILE_scheme_handler>(),
            );
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::{Mutex, OnceLock};

    fn auth_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    unsafe extern "C" fn test_header_callback(
        _data: *mut c_void,
        _headers: *mut *mut *mut c_char,
    ) -> c_int {
        0
    }

    unsafe fn parse_words(headers: *mut HFileLibcurlHeaders, words: &[usize]) -> c_int {
        let mut reg_save = [0usize; 6];
        let mut overflow = vec![0usize; words.len().saturating_sub(reg_save.len())];
        for (i, word) in words.iter().copied().enumerate() {
            if i < reg_save.len() {
                reg_save[i] = word;
            } else {
                overflow[i - reg_save.len()] = word;
            }
        }
        let mut args = hts_sys::__va_list_tag {
            gp_offset: 0,
            fp_offset: 48,
            overflow_arg_area: overflow.as_mut_ptr().cast(),
            reg_save_area: reg_save.as_mut_ptr().cast(),
        };
        hfile_libcurl_c_1554_parse_va_list(headers, &mut args)
    }

    #[test]
    fn libcurl_parse_va_list_decodes_headers_and_options() {
        unsafe {
            let mut headers: HFileLibcurlHeaders = std::mem::zeroed();
            let response = 0 as libc::c_long;
            let callback_data = 0x1234usize as *mut c_void;
            let words = [
                c"httphdr".as_ptr() as usize,
                c"Authorization: Bearer token".as_ptr() as usize,
                c"httphdr:l".as_ptr() as usize,
                c"X-One: 1".as_ptr() as usize,
                c"X-Two: 2".as_ptr() as usize,
                0,
                c"httphdr_callback".as_ptr() as usize,
                test_header_callback as usize,
                c"httphdr_callback_data".as_ptr() as usize,
                callback_data as usize,
                c"http_response_ptr".as_ptr() as usize,
                (&response as *const libc::c_long).cast_mut() as usize,
                c"fail_on_error".as_ptr() as usize,
                0,
                c"auth_token_enabled".as_ptr() as usize,
                c"false".as_ptr() as usize,
                0,
            ];

            assert_eq!(parse_words(&mut headers, &words), 0);
            assert_eq!(headers.fixed.num, 3);
            assert_eq!(
                CStr::from_ptr((*headers.fixed.list).data).to_bytes(),
                b"Authorization: Bearer token"
            );
            assert_eq!(
                CStr::from_ptr((*headers.fixed.list.add(1)).data).to_bytes(),
                b"X-One: 1"
            );
            assert_eq!(
                CStr::from_ptr((*headers.fixed.list.add(2)).data).to_bytes(),
                b"X-Two: 2"
            );
            assert!(headers.callback.is_some());
            assert_eq!(headers.callback_data, callback_data);
            assert_eq!(
                headers.http_response_ptr,
                (&response as *const libc::c_long).cast_mut()
            );
            assert_eq!(headers.fail_on_error, 0);
            assert_eq!(headers.auth_hdr_num, -3);

            hfile_libcurl_c_372_free_headers(
                (&mut headers.fixed as *mut HFileLibcurlHdrList).cast(),
                1,
            );
        }
    }

    #[test]
    fn libcurl_parse_va_list_decodes_vector_and_nested_lists() {
        unsafe {
            let mut headers: HFileLibcurlHeaders = std::mem::zeroed();
            let vector = [
                c"X-Vec: 1".as_ptr(),
                c"Authorization: vector".as_ptr(),
                std::ptr::null(),
            ];
            let nested_words = [
                c"httphdr".as_ptr() as usize,
                c"X-Nested: 1".as_ptr() as usize,
                0,
            ];
            let mut nested_reg = [0usize; 6];
            nested_reg[..nested_words.len()].copy_from_slice(&nested_words);
            let mut nested_overflow = [0usize; 1];
            let mut nested = hts_sys::__va_list_tag {
                gp_offset: 0,
                fp_offset: 48,
                overflow_arg_area: nested_overflow.as_mut_ptr().cast(),
                reg_save_area: nested_reg.as_mut_ptr().cast(),
            };
            let words = [
                c"httphdr:v".as_ptr() as usize,
                vector.as_ptr() as usize,
                c"va_list".as_ptr() as usize,
                (&mut nested as *mut hts_sys::__va_list_tag) as usize,
                0,
            ];

            assert_eq!(parse_words(&mut headers, &words), 0);
            assert_eq!(headers.fixed.num, 3);
            assert_eq!(
                CStr::from_ptr((*headers.fixed.list).data).to_bytes(),
                b"X-Vec: 1"
            );
            assert_eq!(headers.auth_hdr_num, -1);
            assert_eq!(
                CStr::from_ptr((*headers.fixed.list.add(2)).data).to_bytes(),
                b"X-Nested: 1"
            );

            hfile_libcurl_c_372_free_headers(
                (&mut headers.fixed as *mut HFileLibcurlHdrList).cast(),
                1,
            );
        }
    }

    #[test]
    fn libcurl_parse_va_list_rejects_unknown_option() {
        unsafe {
            let mut headers: HFileLibcurlHeaders = std::mem::zeroed();
            *crate::htslib_rs::c_compat::__errno_location() = 0;
            assert_eq!(
                parse_words(&mut headers, &[c"unknown".as_ptr() as usize, 0]),
                -1
            );
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EINVAL
            );
        }
    }

    #[test]
    fn libcurl_errno_mappings_keep_enosys_only_for_remote_unsupported_cases() {
        unsafe {
            assert_eq!(hfile_libcurl_c_130_http_status_errno(501), libc::ENOSYS);
            assert_eq!(
                hfile_libcurl_c_153_easy_errno(std::ptr::null_mut(), 4),
                libc::ENOSYS
            );
            assert_eq!(hfile_libcurl_c_130_http_status_errno(500), libc::EIO);
            assert_eq!(hfile_libcurl_c_130_http_status_errno(503), libc::EBUSY);
            assert_eq!(hfile_libcurl_c_130_http_status_errno(404), libc::ENOENT);
        }
    }

    #[test]
    fn libcurl_callback_failure_errno_preserves_callback_error_or_defaults_to_eio() {
        unsafe {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EACCES;
            hfile_libcurl_set_callback_failure_errno();
            assert_eq!(
                *crate::htslib_rs::c_compat::__errno_location(),
                libc::EACCES
            );

            *crate::htslib_rs::c_compat::__errno_location() = 0;
            hfile_libcurl_set_callback_failure_errno();
            assert_eq!(*crate::htslib_rs::c_compat::__errno_location(), libc::EIO);
        }
    }

    #[test]
    fn libcurl_auth_token_path_expands_host_and_respects_https_gate() {
        let _guard = auth_env_lock();
        unsafe {
            let tmp = std::env::temp_dir();
            let prefix = format!(
                "{}/htslib-rs-libcurl-auth-{}-%h.txt",
                tmp.display(),
                std::process::id()
            );
            let token_path = prefix.replace("%h", "example.test");
            std::fs::write(&token_path, b"  secret-token  \n").unwrap();

            let prefix_c = std::ffi::CString::new(prefix).unwrap();
            let old_path = HFILE_LIBCURL_AUTH_PATH;
            let old_allow = HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER;
            HFILE_LIBCURL_AUTH_PATH = libc::strdup(prefix_c.as_ptr());
            HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER = 0;

            let mut https_fp: HFileLibcurlHeaderPrefix = std::mem::zeroed();
            assert_eq!(
                hfile_libcurl_c_650_get_auth_token(
                    (&mut https_fp as *mut HFileLibcurlHeaderPrefix).cast(),
                    c"https://example.test/data.bam".as_ptr(),
                ),
                0
            );
            assert_eq!(https_fp.headers.extra.num, 1);
            assert_eq!(https_fp.headers.auth_hdr_num, 1);
            assert_eq!(
                CStr::from_ptr((*https_fp.headers.extra.list).data).to_bytes(),
                b"Authorization: Bearer secret-token"
            );

            let mut http_fp: HFileLibcurlHeaderPrefix = std::mem::zeroed();
            assert_eq!(
                hfile_libcurl_c_650_get_auth_token(
                    (&mut http_fp as *mut HFileLibcurlHeaderPrefix).cast(),
                    c"http://example.test/data.bam".as_ptr(),
                ),
                0
            );
            assert_eq!(http_fp.headers.extra.num, 0);
            assert!(http_fp.headers.auth.is_null());

            hfile_libcurl_c_372_free_headers(
                (&mut https_fp.headers.extra as *mut HFileLibcurlHdrList).cast(),
                1,
            );
            libc::free(HFILE_LIBCURL_AUTH_PATH.cast());
            HFILE_LIBCURL_AUTH_PATH = old_path;
            HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER = old_allow;
            let _ = std::fs::remove_file(token_path);
        }
    }
}
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
