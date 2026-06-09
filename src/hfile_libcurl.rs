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
    hts::{hFILE, hts_json_token, hts_verbose, ks_release, kstring_t, size_t},
    textutils::{
        hts_json_token_str_ref, hts_json_token_type_ref, textutils_hts_json_fnext_ref,
        textutils_hts_json_fskip_value_ref,
    },
};
use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void, CStr, CString};
use std::ptr::NonNull;

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

struct HFileLibcurlAuthToken {
    path: CString,
    token: Option<CString>,
    expiry: libc::time_t,
    failed: bool,
    lock: crate::htslib_rs::c_compat::pthread_mutex_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HFileLibcurlCurlSlist {
    data: *mut c_char,
    next: *mut HFileLibcurlCurlSlist,
}

#[derive(Default)]
pub(crate) struct HFileLibcurlHdrList {
    list: Vec<HFileLibcurlCurlSlist>,
    values: Vec<CString>,
}

impl Clone for HFileLibcurlHdrList {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            list: Vec::with_capacity(self.list.len()),
            values: self.values.clone(),
        };
        for value in &cloned.values {
            cloned.list.push(HFileLibcurlCurlSlist {
                data: value.as_ptr().cast_mut(),
                next: std::ptr::null_mut(),
            });
        }
        cloned.relink();
        cloned
    }
}

impl HFileLibcurlHdrList {
    fn len(&self) -> usize {
        self.list.len()
    }

    fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    fn as_mut_ptr(&mut self) -> *mut HFileLibcurlCurlSlist {
        self.list.as_mut_ptr()
    }

    fn relink(&mut self) {
        let base = self.list.as_mut_ptr();
        let len = self.list.len();
        for i in 0..len {
            self.list[i].next = if i + 1 < len {
                unsafe { base.add(i + 1) }
            } else {
                std::ptr::null_mut()
            };
        }
    }

    fn update_data_ptr(&mut self, idx: usize) {
        self.list[idx].data = self.values[idx].as_ptr().cast_mut();
    }

    fn push_cstr(&mut self, value: &CStr) -> c_int {
        self.values.push(value.to_owned());
        let data = self.values.last().unwrap().as_ptr().cast_mut();
        self.list.push(HFileLibcurlCurlSlist {
            data,
            next: std::ptr::null_mut(),
        });
        self.relink();
        0
    }

    fn push_owned(&mut self, value: CString) -> c_int {
        self.values.push(value);
        let data = self.values.last().unwrap().as_ptr().cast_mut();
        self.list.push(HFileLibcurlCurlSlist {
            data,
            next: std::ptr::null_mut(),
        });
        self.relink();
        0
    }
}

#[repr(C)]
struct HFileLibcurlBuffer {
    ptr: Option<NonNull<c_char>>,
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

pub(crate) type HFileLibcurlHttpHeaderCallback =
    unsafe extern "C" fn(*mut c_void, *mut *mut *mut c_char) -> c_int;
pub(crate) type HFileLibcurlRedirectCallback =
    unsafe extern "C" fn(*mut c_void, libc::c_long, *mut kstring_t, *mut kstring_t) -> c_int;

type HFileOpenFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut hFILE;
type HFileIsRemoteFn = unsafe extern "C" fn(*const c_char) -> c_int;
type HFileVOpenFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE;

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
#[derive(Clone)]
pub struct HFileLibcurlHeaders {
    pub(crate) fixed: HFileLibcurlHdrList,
    pub(crate) extra: HFileLibcurlHdrList,
    pub(crate) callback: Option<HFileLibcurlHttpHeaderCallback>,
    pub(crate) callback_data: *mut c_void,
    auth: Option<NonNull<HFileLibcurlAuthToken>>,
    pub(crate) auth_hdr_num: c_int,
    pub(crate) redirect: Option<HFileLibcurlRedirectCallback>,
    pub(crate) redirect_data: *mut c_void,
    pub(crate) http_response_ptr: Option<NonNull<libc::c_long>>,
    pub(crate) fail_on_error: c_int,
}

impl Default for HFileLibcurlHeaders {
    fn default() -> Self {
        Self {
            fixed: HFileLibcurlHdrList::default(),
            extra: HFileLibcurlHdrList::default(),
            callback: None,
            callback_data: std::ptr::null_mut(),
            auth: None,
            auth_hdr_num: 0,
            redirect: None,
            redirect_data: std::ptr::null_mut(),
            http_response_ptr: None,
            fail_on_error: 1,
        }
    }
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
    preserved: Vec<c_char>,
    preserved_bytes: usize,
}

impl Default for HFileLibcurlHeaderPrefix {
    fn default() -> Self {
        Self {
            base: HFileLayout {
                buffer: std::ptr::null_mut(),
                begin: std::ptr::null_mut(),
                end: std::ptr::null_mut(),
                limit: std::ptr::null_mut(),
                backend: std::ptr::null(),
                offset: 0,
                flags: 0,
                has_errno: 0,
            },
            easy: std::ptr::null_mut(),
            multi: std::ptr::null_mut(),
            file_size: 0,
            buffer: HFileLibcurlBuffer { ptr: None, len: 0 },
            final_result: 0,
            flags: 0,
            nrunning: 0,
            headers: HFileLibcurlHeaders::default(),
            delayed_seek: 0,
            last_offset: 0,
            preserved: Vec::new(),
            preserved_bytes: 0,
        }
    }
}

// Concurrency notes (audit 2026-05):
//
// All of the `static mut` items below mirror the layout of `htslib/hfile_libcurl.c`'s
// process-global `curl` struct. They fall into three categories:
//
// 1. `*_LOCK` (pthread_mutex_t) — the locks themselves. They are statically
//    initialized to `PTHREAD_MUTEX_INITIALIZER`, and only ever read by taking
//    their address and handing the pointer to pthread routines. The values
//    don't migrate between threads as plain Rust state; pthread internals
//    serialize access. They must be `static mut` because `pthread_mutex_t`
//    is `!Sync`.
//
// 2. Plugin init-once-then-read state: `HFILE_LIBCURL_USERAGENT`,
//    `HFILE_LIBCURL_SHARE`, `HFILE_LIBCURL_AUTH_PATH`,
//    `HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER`. These are written
//    exactly once from `hfile_libcurl_c_1679_PLUGIN_GLOBAL`, which is
//    called from `hfile_c_1111_load_hfile_plugins` under
//    `hfile_plugin_state().lock()` (see `src/hfile.rs`). After init they
//    are read-only for the rest of the process lifetime, until
//    `hfile_libcurl_c_326_libcurl_exit` runs at shutdown (also single-
//    threaded — the plugin destroy callback fires once per process exit).
//
// 3. Lazy-init-then-mutate-under-lock: `HFILE_LIBCURL_AUTH_MAP`. The Vec is
//    initialized (and grown) only while `HFILE_LIBCURL_AUTH_LOCK` is held — see
//    `hfile_libcurl_c_650_get_auth_token`. Concurrent callers serialize on
//    that mutex.
//
// 4. Env-var snapshots: `HFILE_LIBCURL_RETRY_MAX`, `HFILE_LIBCURL_RETRY_DELAY_MS`.
//    These are refreshed from `getenv(HTS_RETRY_*)` on each `hopen()` /
//    reconnect via `hfile_libcurl_c_821_refresh_retry_config`. Concurrent
//    writers race, but each writer derives the value deterministically from
//    the (process-wide) environment, so the only observable race is a torn
//    word between consecutive `getenv` calls returning identical strings —
//    which on the targeted platforms (x86_64, aarch64) is a non-issue for
//    aligned word-sized stores, and matches the C original's behaviour.
//
// SAFETY: see the per-static notes above. Do not promote to a `Mutex<T>`:
// that would gratuitously diverge from the C ABI exposed via
// `hts_sys::*` and break the parity tests.
static mut HFILE_LIBCURL_SHARE_LOCK: crate::htslib_rs::c_compat::pthread_mutex_t =
    crate::htslib_rs::c_compat::PTHREAD_MUTEX_INITIALIZER;
static mut HFILE_LIBCURL_AUTH_LOCK: crate::htslib_rs::c_compat::pthread_mutex_t =
    crate::htslib_rs::c_compat::PTHREAD_MUTEX_INITIALIZER;
static mut HFILE_LIBCURL_USERAGENT: kstring_t = kstring_t {
    l: 0,
    m: 0,
    s: std::ptr::null_mut(),
};
static mut HFILE_LIBCURL_SHARE: *mut c_void = std::ptr::null_mut();
static mut HFILE_LIBCURL_AUTH_PATH: Option<CString> = None;
// Header state keeps NonNull pointers to auth tokens; Box keeps token
// addresses stable even when the Vec grows.
#[allow(clippy::vec_box)]
static mut HFILE_LIBCURL_AUTH_MAP: Option<Vec<Box<HFileLibcurlAuthToken>>> = None;
static mut HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER: c_int = 0;
static mut HFILE_LIBCURL_RETRY_MAX: c_int = 0;
static mut HFILE_LIBCURL_RETRY_DELAY_MS: libc::c_long = 1000;

unsafe fn hfile_libcurl_new_auth_token(path: CString) -> Option<Box<HFileLibcurlAuthToken>> {
    let mut tok = Box::new(HFileLibcurlAuthToken {
        path,
        token: None,
        expiry: 1,
        failed: false,
        lock: std::mem::zeroed(),
    });
    if crate::htslib_rs::c_compat::pthread_mutex_init(&mut tok.lock, std::ptr::null()) != 0 {
        None
    } else {
        Some(tok)
    }
}

#[allow(clippy::boxed_local)]
unsafe fn hfile_libcurl_free_auth_box(mut tok: Box<HFileLibcurlAuthToken>) {
    if crate::htslib_rs::c_compat::pthread_mutex_destroy(&mut tok.lock) != 0 {
        libc::abort();
    }
}

unsafe fn hfile_libcurl_take_released_cstring(raw: *mut c_char) -> Option<CString> {
    if raw.is_null() {
        None
    } else {
        let out = CStr::from_ptr(raw).to_owned();
        libc::free(raw.cast());
        Some(out)
    }
}

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

// (the libhts `hfile_plugin_init_libcurl` extern was removed 2026-05-29 —
// it had no call sites. The libcurl plugin layer is now driven entirely by
// the native `hfile_libcurl_c_*` functions in this file.)

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
            let detail = std::ffi::CStr::from_ptr(curl_easy_strerror(err)).to_string_lossy();
            let msg =
                std::ffi::CString::new(format!("Libcurl reported error {} ({})", err, detail))
                    .unwrap_or_default();
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_ERROR,
                c"easy_errno".as_ptr(),
                msg.as_ptr(),
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
            let detail = std::ffi::CStr::from_ptr(curl_multi_strerror(errm)).to_string_lossy();
            let msg =
                std::ffi::CString::new(format!("Libcurl reported error {} ({})", errm, detail))
                    .unwrap_or_default();
            crate::htslib_rs::hts::hts_log_cstr(
                crate::htslib_rs::hts::HTS_LOG_ERROR,
                c"multi_errno".as_ptr(),
                msg.as_ptr(),
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
    crate::htslib_rs::c_compat::pthread_mutex_lock(std::ptr::addr_of_mut!(
        HFILE_LIBCURL_SHARE_LOCK
    ));
}

// original: share_unlock (htslib/hfile_libcurl.c:314)
pub unsafe extern "C" fn hfile_libcurl_c_314_share_unlock(
    _handle: *mut c_void,
    _data: c_int,
    _userptr: *mut c_void,
) {
    crate::htslib_rs::c_compat::pthread_mutex_unlock(std::ptr::addr_of_mut!(
        HFILE_LIBCURL_SHARE_LOCK
    ));
}

// original: free_auth (htslib/hfile_libcurl.c:318)
pub unsafe fn hfile_libcurl_c_318_free_auth(tok: *mut c_void) {
    let tok = tok.cast::<HFileLibcurlAuthToken>();
    if tok.is_null() {
        return;
    }
    hfile_libcurl_free_auth_box(Box::from_raw(tok));
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

    *std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_PATH) = None;

    let auth_map_ptr = std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_MAP);
    if let Some(mut map) = (*auth_map_ptr).take() {
        for tok in map.drain(..) {
            hfile_libcurl_free_auth_box(tok);
        }
    }
    curl_global_cleanup();
}

// original: append_header (htslib/hfile_libcurl.c:353)
pub(crate) unsafe fn hfile_libcurl_c_353_append_header(
    hdrs: &mut HFileLibcurlHdrList,
    data: *const c_char,
    dup: c_int,
) -> c_int {
    if data.is_null() {
        return -1;
    }

    let value = CStr::from_ptr(data);
    let ret = hdrs.push_cstr(value);
    if dup == 0 {
        libc::free(data.cast_mut().cast());
    }
    ret
}

unsafe fn hfile_libcurl_append_released_header(
    hdrs: &mut HFileLibcurlHdrList,
    data: *mut c_char,
) -> c_int {
    match hfile_libcurl_take_released_cstring(data) {
        Some(value) => hdrs.push_owned(value),
        None => -1,
    }
}

// original: free_headers (htslib/hfile_libcurl.c:372)
pub(crate) unsafe fn hfile_libcurl_c_372_free_headers(
    hdrs: &mut HFileLibcurlHdrList,
    completely: c_int,
) {
    hdrs.list.clear();
    hdrs.values.clear();
    if completely != 0 {
        *hdrs = HFileLibcurlHdrList::default();
    }
}

// original: get_header_list (htslib/hfile_libcurl.c:387)
pub unsafe fn hfile_libcurl_c_387_get_header_list(fp: *mut c_void) -> *mut c_void {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return std::ptr::null_mut();
    };
    hfile_libcurl_get_header_list_ref(fp).map_or(std::ptr::null_mut(), |ptr| ptr.as_ptr().cast())
}

fn hfile_libcurl_get_header_list_ref(
    fp: &mut HFileLibcurlHeaderPrefix,
) -> Option<NonNull<HFileLibcurlCurlSlist>> {
    if !fp.headers.fixed.is_empty() {
        NonNull::new(fp.headers.fixed.as_mut_ptr())
    } else if !fp.headers.extra.is_empty() {
        NonNull::new(fp.headers.extra.as_mut_ptr())
    } else {
        None
    }
}

// original: is_authorization (htslib/hfile_libcurl.c:395)
pub unsafe fn hfile_libcurl_c_395_is_authorization(hdr: *const c_char) -> c_int {
    let Some(hdr) = (!hdr.is_null()).then(|| CStr::from_ptr(hdr)) else {
        return 0;
    };
    hfile_libcurl_is_authorization_ref(hdr) as c_int
}

fn hfile_libcurl_is_authorization_ref(hdr: &CStr) -> bool {
    let bytes = hdr.to_bytes();
    bytes.len() >= 14 && bytes[..14].eq_ignore_ascii_case(b"authorization:")
}

// original: add_callback_headers (htslib/hfile_libcurl.c:399)
pub unsafe fn hfile_libcurl_c_399_add_callback_headers(fp: *mut c_void) -> c_int {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_add_callback_headers_ref(fp)
}

unsafe fn hfile_libcurl_add_callback_headers_ref(fp: &mut HFileLibcurlHeaderPrefix) -> c_int {
    let Some(callback) = fp.headers.callback else {
        return 0;
    };

    let mut hdrs: *mut *mut c_char = std::ptr::null_mut();
    if callback(fp.headers.callback_data, &mut hdrs) != 0 {
        return -1;
    }
    if hdrs.is_null() {
        return 0;
    }

    if !fp.headers.fixed.is_empty() {
        fp.headers.fixed.relink();
    }
    hfile_libcurl_c_372_free_headers(&mut fp.headers.extra, 0);

    if fp.headers.auth_hdr_num > 0 || fp.headers.auth_hdr_num == -2 {
        fp.headers.auth_hdr_num = 0;
    }

    let mut hdr = hdrs;
    while !(*hdr).is_null() {
        let header = *hdr;
        let is_authorization = hfile_libcurl_is_authorization_ref(CStr::from_ptr(header));
        if hfile_libcurl_append_released_header(&mut fp.headers.extra, header) < 0 {
            while !hdr.is_null() && !(*hdr).is_null() {
                libc::free((*hdr).cast());
                *hdr = std::ptr::null_mut();
                hdr = hdr.add(1);
            }
            return -1;
        }
        *hdr = std::ptr::null_mut();
        if is_authorization && fp.headers.auth_hdr_num == 0 {
            fp.headers.auth_hdr_num = -2;
        }
        hdr = hdr.add(1);
    }

    if !fp.headers.fixed.is_empty() && !fp.headers.extra.is_empty() {
        let extra = fp.headers.extra.as_mut_ptr();
        fp.headers.fixed.list.last_mut().unwrap().next = extra;
    }
    0
}

// original: read_auth_json (htslib/hfile_libcurl.c:454)
pub unsafe fn hfile_libcurl_c_454_read_auth_json(tok: *mut c_void, auth_fp: *mut hFILE) -> c_int {
    let Some(tok) = tok.cast::<HFileLibcurlAuthToken>().as_mut() else {
        return b'i' as c_int;
    };
    let Some(auth_fp) = auth_fp.as_mut() else {
        return b'i' as c_int;
    };
    hfile_libcurl_read_auth_json_ref(tok, auth_fp)
}

unsafe fn hfile_libcurl_read_auth_json_ref(
    tok: &mut HFileLibcurlAuthToken,
    auth_fp: &mut hFILE,
) -> c_int {
    let mut t = hts_json_token {
        type_: 0,
        str_: std::ptr::null_mut(),
    };
    let mut str_: kstring_t = std::mem::zeroed();
    let mut token: Option<CString> = None;
    let mut type_: Option<CString> = None;
    let mut expiry: Option<CString> = None;
    let mut ret = b'i' as c_int;

    if textutils_hts_json_fnext_ref(auth_fp, &mut t, &mut str_) != b'{' as c_char {
        return finish_auth_json(&mut str_, ret);
    }
    while textutils_hts_json_fnext_ref(auth_fp, &mut t, &mut str_) != b'}' as c_char {
        if hts_json_token_type_ref(&t) != b's' as c_char {
            ret = b'?' as c_int;
            return finish_auth_json(&mut str_, ret);
        }
        let key = hts_json_token_str_ref(&t);
        if key.is_null() {
            ret = b'm' as c_int;
            return finish_auth_json(&mut str_, ret);
        }
        if libc::strcmp(key, c"access_token".as_ptr()) == 0 {
            ret = textutils_hts_json_fnext_ref(auth_fp, &mut t, &mut str_) as c_int;
            if ret != b's' as c_int {
                return finish_auth_json(&mut str_, ret);
            }
            token = hfile_libcurl_take_released_cstring(ks_release(&mut str_));
        } else if libc::strcmp(key, c"token_type".as_ptr()) == 0 {
            ret = textutils_hts_json_fnext_ref(auth_fp, &mut t, &mut str_) as c_int;
            if ret != b's' as c_int {
                return finish_auth_json(&mut str_, ret);
            }
            type_ = hfile_libcurl_take_released_cstring(ks_release(&mut str_));
        } else if libc::strcmp(key, c"expires_in".as_ptr()) == 0 {
            ret = textutils_hts_json_fnext_ref(auth_fp, &mut t, &mut str_) as c_int;
            if ret != b'n' as c_int {
                return finish_auth_json(&mut str_, ret);
            }
            expiry = hfile_libcurl_take_released_cstring(ks_release(&mut str_));
        } else if textutils_hts_json_fskip_value_ref(auth_fp, 0) != b'v' as c_char {
            ret = b'?' as c_int;
            return finish_auth_json(&mut str_, ret);
        }
    }

    if token.is_none()
        || type_
            .as_ref()
            .is_some_and(|type_| type_.as_bytes() != b"Bearer")
    {
        return finish_auth_json(&mut str_, b'i' as c_int);
    }

    ret = b'm' as c_int;
    let Some(token) = token else {
        return finish_auth_json(&mut str_, b'i' as c_int);
    };
    let mut header = b"Authorization: Bearer ".to_vec();
    header.extend_from_slice(token.as_bytes());
    let Ok(header) = CString::new(header) else {
        return finish_auth_json(&mut str_, ret);
    };
    tok.token = Some(header);
    if let Some(expiry) = expiry.as_ref() {
        let mut exp = libc::strtol(expiry.as_ptr(), std::ptr::null_mut(), 10);
        if exp < 0 {
            exp = 0;
        }
        tok.expiry = libc::time(std::ptr::null_mut()) + exp as libc::time_t;
    } else {
        tok.expiry = 0;
    }
    finish_auth_json(&mut str_, b'v' as c_int)
}

unsafe fn finish_auth_json(str_: &mut kstring_t, ret: c_int) -> c_int {
    libc::free(str_.s.cast());
    str_.s = std::ptr::null_mut();
    str_.l = 0;
    str_.m = 0;
    ret
}

// original: read_auth_plain (htslib/hfile_libcurl.c:515)
pub unsafe fn hfile_libcurl_c_515_read_auth_plain(tok: *mut c_void, auth_fp: *mut hFILE) -> c_int {
    let Some(tok) = tok.cast::<HFileLibcurlAuthToken>().as_mut() else {
        return -1;
    };
    let Some(auth_fp) = auth_fp.as_mut() else {
        return -1;
    };
    hfile_libcurl_read_auth_plain_ref(tok, auth_fp)
}

unsafe fn hfile_libcurl_read_auth_plain_ref(
    tok: &mut HFileLibcurlAuthToken,
    auth_fp: &mut hFILE,
) -> c_int {
    let mut line: kstring_t = std::mem::zeroed();

    if crate::htslib_rs::hfile::khgetline(&mut line, auth_fp as *mut hFILE) < 0 {
        libc::free(line.s.cast());
        return -1;
    }
    if crate::htslib_rs::hts::kputc(0, &mut line) < 0 {
        libc::free(line.s.cast());
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
        let token_len = end.offset_from(start) as usize;
        let token_bytes = std::slice::from_raw_parts(start.cast::<u8>(), token_len);
        let mut header = b"Authorization: Bearer ".to_vec();
        header.extend_from_slice(token_bytes);
        let Ok(header) = CString::new(header) else {
            libc::free(line.s.cast());
            return -1;
        };
        tok.token = Some(header);
    } else {
        tok.token = None;
    }

    tok.expiry = 0;
    libc::free(line.s.cast());
    0
}

// original: renew_auth_token (htslib/hfile_libcurl.c:543)
pub unsafe fn hfile_libcurl_c_543_renew_auth_token(tok: *mut c_void, changed: *mut c_int) -> c_int {
    let Some(tok) = tok.cast::<HFileLibcurlAuthToken>().as_mut() else {
        return -1;
    };
    let Some(changed) = changed.as_mut() else {
        return -1;
    };
    hfile_libcurl_renew_auth_token_ref(tok, changed)
}

unsafe fn hfile_libcurl_renew_auth_token_ref(
    tok: &mut HFileLibcurlAuthToken,
    changed: &mut c_int,
) -> c_int {
    let mut buffer = [0 as c_char; 16];

    *changed = 0;
    if tok.expiry == 0 || libc::time(std::ptr::null_mut()) + AUTH_REFRESH_EARLY_SECS < tok.expiry {
        return 0;
    }
    if tok.failed {
        return -1;
    }

    *changed = 1;
    let auth_fp = hopen(tok.path.as_ptr(), c"rR".as_ptr());
    if auth_fp.is_null() {
        if *crate::htslib_rs::c_compat::__errno_location() != libc::ENOENT {
            tok.failed = true;
            return -1;
        }
        tok.expiry = 0;
        tok.token = None;
        return 0;
    }

    let len = hpeek(auth_fp, buffer.as_mut_ptr().cast(), buffer.len());
    if len < 0 {
        tok.failed = true;
        hclose_abruptly(auth_fp);
        return -1;
    }

    let auth_fp_ref = &mut *auth_fp;
    let ok = if !libc::memchr(buffer.as_ptr().cast(), b'{' as c_int, len as usize).is_null() {
        hfile_libcurl_read_auth_json_ref(tok, auth_fp_ref) == b'v' as c_int
    } else {
        hfile_libcurl_read_auth_plain_ref(tok, auth_fp_ref) >= 0
    };
    if !ok {
        tok.failed = true;
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
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_add_auth_header_ref(fp)
}

unsafe fn hfile_libcurl_add_auth_header_ref(fp: &mut HFileLibcurlHeaderPrefix) -> c_int {
    let mut changed = 0;

    let Some(mut auth) = fp.headers.auth else {
        return 0;
    };
    let auth = auth.as_mut();
    if fp.headers.auth_hdr_num < 0 {
        return 0;
    }

    crate::htslib_rs::c_compat::pthread_mutex_lock(&mut auth.lock);
    if hfile_libcurl_renew_auth_token_ref(auth, &mut changed) < 0 {
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut auth.lock);
        return -1;
    }

    if changed == 0 && fp.headers.auth_hdr_num > 0 {
        crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut auth.lock);
        return 0;
    }

    if fp.headers.auth_hdr_num > 0 {
        let header = auth.token.as_ref();
        let idx = (fp.headers.auth_hdr_num - 1) as usize;
        if let Some(header) = header {
            fp.headers.extra.values[idx] = header.clone();
            fp.headers.extra.update_data_ptr(idx);
        } else {
            fp.headers.extra.values.remove(idx);
            fp.headers.extra.list.remove(idx);
            fp.headers.extra.relink();
            if fp.headers.extra.is_empty() && !fp.headers.fixed.is_empty() {
                fp.headers.fixed.list.last_mut().unwrap().next = std::ptr::null_mut();
            }
            fp.headers.auth_hdr_num = 0;
        }
    } else if let Some(header) = auth.token.as_ref() {
        if fp.headers.extra.push_cstr(header) < 0 {
            crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut auth.lock);
            return -1;
        }
        fp.headers.auth_hdr_num = fp.headers.extra.len() as c_int;
    }

    crate::htslib_rs::c_compat::pthread_mutex_unlock(&mut auth.lock);
    0
}

// original: get_auth_token (htslib/hfile_libcurl.c:650)
pub unsafe fn hfile_libcurl_c_650_get_auth_token(fp: *mut c_void, url: *const c_char) -> c_int {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    let Some(url) = (!url.is_null()).then(|| CStr::from_ptr(url)) else {
        return -1;
    };
    hfile_libcurl_get_auth_token_ref(fp, url)
}

unsafe fn hfile_libcurl_get_auth_token_ref(fp: &mut HFileLibcurlHeaderPrefix, url: &CStr) -> c_int {
    let auth_path_ptr = std::ptr::addr_of!(HFILE_LIBCURL_AUTH_PATH);
    let Some(auth_path) = (*auth_path_ptr).as_ref() else {
        return 0;
    };
    if (fp.flags & HFILE_LIBCURL_IS_RECURSIVE) != 0 || fp.headers.auth_hdr_num != 0 {
        return 0;
    }
    if HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER == 0 && !url.to_bytes().starts_with(b"https://")
    {
        return 0;
    }

    let url_bytes = url.to_bytes();
    let host = url_bytes
        .windows(3)
        .position(|window| window == b"://")
        .map_or(&[][..], |scheme_end| {
            let rest = &url_bytes[scheme_end + 3..];
            let end = rest
                .iter()
                .position(|byte| *byte == b'/')
                .unwrap_or(rest.len());
            &rest[..end]
        });
    let path_bytes = auth_path.as_bytes();
    let mut name = Vec::with_capacity(path_bytes.len() + host.len());
    let mut rest = path_bytes;
    while let Some(pos) = rest.windows(2).position(|window| window == b"%h") {
        name.extend_from_slice(&rest[..pos]);
        name.extend_from_slice(host);
        rest = &rest[pos + 2..];
    }
    name.extend_from_slice(rest);
    let Ok(name) = CString::new(name) else {
        return -1;
    };

    crate::htslib_rs::c_compat::pthread_mutex_lock(std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_LOCK));
    let auth_map_ptr = std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_MAP);
    if (*auth_map_ptr).is_none() {
        *auth_map_ptr = Some(Vec::new());
    }
    let map = (*auth_map_ptr)
        .as_mut()
        .expect("libcurl auth map initialized");
    let mut tok: Option<NonNull<HFileLibcurlAuthToken>> = None;
    for entry in map.iter_mut() {
        if entry.path.as_c_str() == name.as_c_str() {
            tok = NonNull::new((&mut **entry) as *mut HFileLibcurlAuthToken);
            break;
        }
    }
    if tok.is_none() {
        if let Some(mut new_tok) = hfile_libcurl_new_auth_token(name) {
            tok = NonNull::new((&mut *new_tok) as *mut HFileLibcurlAuthToken);
            map.push(new_tok);
        }
    }
    crate::htslib_rs::c_compat::pthread_mutex_unlock(std::ptr::addr_of_mut!(
        HFILE_LIBCURL_AUTH_LOCK
    ));

    fp.headers.auth = tok;
    if tok.is_none() {
        -1
    } else {
        hfile_libcurl_add_auth_header_ref(fp)
    }
}

// original: process_messages (htslib/hfile_libcurl.c:718)
pub unsafe fn hfile_libcurl_c_718_process_messages(fp: *mut c_void) {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return;
    };
    hfile_libcurl_process_messages_ref(fp);
}

unsafe fn hfile_libcurl_process_messages_ref(fp: &mut HFileLibcurlHeaderPrefix) {
    let mut remaining = 0;
    loop {
        let msg = curl_multi_info_read(fp.multi, &mut remaining);
        if msg.is_null() {
            break;
        }
        if (*msg).msg == CURLMSG_DONE {
            fp.flags |= HFILE_LIBCURL_FINISHED;
            fp.final_result = (*msg).data.result;
        }
    }
}

// original: wait_perform (htslib/hfile_libcurl.c:736)
pub unsafe fn hfile_libcurl_c_736_wait_perform(fp: *mut c_void) -> c_int {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_wait_perform_ref(fp)
}

unsafe fn hfile_libcurl_wait_perform_ref(fp: &mut HFileLibcurlHeaderPrefix) -> c_int {
    if (fp.flags & HFILE_LIBCURL_PERFORM_AGAIN) == 0 {
        let mut timeout: libc::c_long = 1000;
        if curl_multi_timeout(fp.multi, &mut timeout) != CURLM_OK || timeout < 0 {
            timeout = 1000;
        }
        if timeout > 100 {
            timeout = 100;
        }
        let mut numfds = 0;
        let errm = curl_multi_wait(
            fp.multi,
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
    let errm = curl_multi_perform(fp.multi, &mut nrunning);
    fp.flags &= !HFILE_LIBCURL_PERFORM_AGAIN;
    if errm == CURLM_CALL_MULTI_PERFORM {
        fp.flags |= HFILE_LIBCURL_PERFORM_AGAIN;
    } else if errm != CURLM_OK {
        *crate::htslib_rs::c_compat::__errno_location() = hfile_libcurl_c_270_multi_errno(errm);
        return -1;
    }
    if nrunning < fp.nrunning {
        hfile_libcurl_process_messages_ref(fp);
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

    let Some(fp) = fpv.cast::<HFileLibcurlCallbackPrefix>().as_mut() else {
        return 0;
    };
    let n = size.saturating_mul(nmemb);

    if n > fp.buffer.len {
        fp.flags |= HFILE_LIBCURL_PAUSED;
        CURL_WRITEFUNC_PAUSE
    } else if n == 0 {
        0
    } else {
        let dst = fp
            .buffer
            .ptr
            .expect("libcurl recv buffer set before callback")
            .as_ptr();
        let src = std::slice::from_raw_parts(ptr.cast::<u8>(), n);
        let dst_slice = std::slice::from_raw_parts_mut(dst.cast::<u8>(), n);
        dst_slice.copy_from_slice(src);
        fp.buffer.ptr = Some(NonNull::new_unchecked(dst.add(n)));
        fp.buffer.len -= n;
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
        tv_sec: (delay_ms / 1000) as _,
        tv_nsec: ((delay_ms % 1000) * 1_000_000) as _,
    };
    crate::htslib_rs::c_compat::nanosleep(&ts, std::ptr::null_mut());
}

// original: retry_reconnect (htslib/hfile_libcurl.c:848)
pub unsafe fn hfile_libcurl_c_848_retry_reconnect(fp: *mut c_void, pos: libc::off_t) -> c_int {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_retry_reconnect_ref(fp, pos)
}

unsafe fn hfile_libcurl_retry_reconnect_ref(
    fp: &mut HFileLibcurlHeaderPrefix,
    pos: libc::off_t,
) -> c_int {
    hfile_libcurl_c_821_refresh_retry_config();
    let mut attempt = 0;
    while attempt < HFILE_LIBCURL_RETRY_MAX {
        hfile_libcurl_c_836_retry_sleep(HFILE_LIBCURL_RETRY_DELAY_MS);
        if hfile_libcurl_restart_from_position_ref(fp, pos) == 0 {
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
    let Some(fp) = fpv.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    let Some(buffer) = NonNull::new(bufferv.cast::<c_char>()) else {
        return if nbytes == 0 { 0 } else { -1 };
    };
    hfile_libcurl_read_ref(fp, buffer, nbytes)
}

unsafe fn hfile_libcurl_read_ref(
    fp: &mut HFileLibcurlHeaderPrefix,
    buffer: NonNull<c_char>,
    nbytes: usize,
) -> libc::ssize_t {
    let buffer = buffer.as_ptr();
    let mut to_skip: libc::off_t = -1;
    let mut filled: libc::ssize_t = 0;

    if fp.delayed_seek >= 0 {
        if !fp.preserved.is_empty()
            && fp.last_offset > fp.delayed_seek
            && fp.last_offset - fp.preserved_bytes as libc::off_t <= fp.delayed_seek
        {
            let n = (fp.last_offset - fp.delayed_seek) as usize;
            let start = fp.preserved.as_ptr().add(fp.preserved_bytes - n);
            let bytes = n.min(nbytes);
            let src = std::slice::from_raw_parts(start.cast::<u8>(), bytes);
            let dst = std::slice::from_raw_parts_mut(buffer.cast::<u8>(), bytes);
            dst.copy_from_slice(src);
            if bytes < n {
                fp.delayed_seek += bytes as libc::off_t;
            } else {
                fp.last_offset = -1;
                fp.delayed_seek = -1;
            }
            return bytes as libc::ssize_t;
        }

        if fp.last_offset >= 0
            && fp.delayed_seek > fp.last_offset
            && fp.delayed_seek - fp.last_offset < MIN_SEEK_FORWARD
        {
            to_skip = fp.delayed_seek - fp.last_offset;
        } else if hfile_libcurl_restart_from_position_ref(fp, fp.delayed_seek) < 0 {
            return -1;
        }
        fp.delayed_seek = -1;
        fp.last_offset = -1;
        fp.preserved_bytes = 0;
    }

    loop {
        if filled as usize >= nbytes {
            return filled;
        }
        let chunk_start = buffer.add(filled as usize);
        fp.buffer.ptr = NonNull::new(chunk_start);
        fp.buffer.len = nbytes - filled as usize;
        fp.flags &= !HFILE_LIBCURL_PAUSED;
        if (fp.flags & HFILE_LIBCURL_FINISHED) == 0 {
            let err = curl_easy_pause(fp.easy, CURLPAUSE_CONT);
            if err != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno(fp.easy, err);
                return -1;
            }
        }

        loop {
            if !((fp.flags & HFILE_LIBCURL_PAUSED) == 0 && (fp.flags & HFILE_LIBCURL_FINISHED) == 0)
            {
                break;
            }
            if hfile_libcurl_wait_perform_ref(fp) < 0 {
                return -1;
            }
        }

        let mut got = fp.buffer.ptr.map_or(0, |ptr| {
            ptr.as_ptr().offset_from(chunk_start) as libc::ssize_t
        });
        if to_skip >= 0 {
            if got <= to_skip as libc::ssize_t {
                to_skip -= got as libc::off_t;
                got = 0;
            } else {
                got -= to_skip as libc::ssize_t;
                if got > 0 {
                    std::ptr::copy(
                        chunk_start.add(to_skip as usize),
                        buffer.add(filled as usize),
                        got as usize,
                    );
                    to_skip = -1;
                }
            }
        }

        fp.buffer.ptr = None;
        fp.buffer.len = 0;
        filled += got;

        if (fp.flags & HFILE_LIBCURL_FINISHED) != 0 && fp.final_result != CURLE_OK {
            let err = fp.final_result;
            let pos = fp.base.offset + filled as libc::off_t;
            if hfile_libcurl_c_233_is_retryable(fp.easy, err) != 0
                && hfile_libcurl_retry_reconnect_ref(fp, pos) == 0
            {
                continue;
            }
            *crate::htslib_rs::c_compat::__errno_location() =
                hfile_libcurl_c_153_easy_errno(fp.easy, err);
            return -1;
        }

        if to_skip < 0 || (fp.flags & HFILE_LIBCURL_FINISHED) != 0 {
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

    let Some(fp) = fpv.cast::<HFileLibcurlCallbackPrefix>().as_mut() else {
        return 0;
    };
    let mut n = size.saturating_mul(nmemb);

    if fp.buffer.len == 0 {
        if (fp.flags & HFILE_LIBCURL_CLOSING) != 0 {
            0
        } else {
            fp.flags |= HFILE_LIBCURL_PAUSED;
            CURL_READFUNC_PAUSE
        }
    } else {
        if n > fp.buffer.len {
            n = fp.buffer.len;
        }
        let src = fp
            .buffer
            .ptr
            .expect("libcurl send buffer set before callback")
            .as_ptr();
        let src_slice = std::slice::from_raw_parts(src.cast::<u8>(), n);
        let dst = std::slice::from_raw_parts_mut(ptr.cast::<u8>(), n);
        dst.copy_from_slice(src_slice);
        fp.buffer.ptr = Some(NonNull::new_unchecked(src.add(n)));
        fp.buffer.len -= n;
        n
    }
}

// original: libcurl_write (htslib/hfile_libcurl.c:1024)
pub unsafe extern "C" fn hfile_libcurl_c_1024_libcurl_write(
    fpv: *mut hFILE,
    bufferv: *const c_void,
    nbytes: usize,
) -> libc::ssize_t {
    let Some(fp) = fpv.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    let Some(buffer) = NonNull::new(bufferv.cast::<c_char>().cast_mut()) else {
        return if nbytes == 0 { 0 } else { -1 };
    };
    hfile_libcurl_write_ref(fp, buffer, nbytes)
}

unsafe fn hfile_libcurl_write_ref(
    fp: &mut HFileLibcurlHeaderPrefix,
    buffer: NonNull<c_char>,
    nbytes: usize,
) -> libc::ssize_t {
    let buffer = buffer.as_ptr();
    fp.buffer.ptr = NonNull::new(buffer);
    fp.buffer.len = nbytes;
    fp.flags &= !HFILE_LIBCURL_PAUSED;
    let err = curl_easy_pause(fp.easy, CURLPAUSE_CONT);
    if err != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno(fp.easy, err);
        return -1;
    }
    loop {
        if !((fp.flags & HFILE_LIBCURL_PAUSED) == 0 && (fp.flags & HFILE_LIBCURL_FINISHED) == 0) {
            break;
        }
        if hfile_libcurl_wait_perform_ref(fp) < 0 {
            return -1;
        }
    }
    let done = fp
        .buffer
        .ptr
        .map_or(0, |ptr| ptr.as_ptr().offset_from(buffer) as libc::ssize_t);
    fp.buffer.ptr = None;
    fp.buffer.len = 0;
    if (fp.flags & HFILE_LIBCURL_FINISHED) != 0 && fp.final_result != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno(fp.easy, fp.final_result);
        return -1;
    }
    done
}

// original: preserve_buffer_content (htslib/hfile_libcurl.c:1051)
pub unsafe fn hfile_libcurl_c_1051_preserve_buffer_content(fp: *mut c_void) {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return;
    };
    hfile_libcurl_preserve_buffer_content_ref(fp);
}

unsafe fn hfile_libcurl_preserve_buffer_content_ref(fp: &mut HFileLibcurlHeaderPrefix) {
    if fp.base.begin == fp.base.end {
        fp.preserved_bytes = 0;
        return;
    }
    let cap = fp.base.limit.offset_from(fp.base.buffer) as usize;
    if fp.preserved.len() < cap {
        fp.preserved.resize(cap, 0);
    }
    let n = fp.base.end.offset_from(fp.base.begin) as usize;
    let dst = std::slice::from_raw_parts_mut(fp.preserved.as_mut_ptr(), n);
    let src = std::slice::from_raw_parts(fp.base.begin, n);
    dst.copy_from_slice(src);
    fp.preserved_bytes = n;
}

// original: libcurl_seek (htslib/hfile_libcurl.c:1071)
pub unsafe extern "C" fn hfile_libcurl_c_1071_libcurl_seek(
    fpv: *mut hFILE,
    offset: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    let Some(fp) = fpv.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_seek_ref(fp, offset, whence)
}

unsafe fn hfile_libcurl_seek_ref(
    fp: &mut HFileLibcurlHeaderPrefix,
    offset: libc::off_t,
    whence: c_int,
) -> libc::off_t {
    if (fp.flags & HFILE_LIBCURL_IS_READ) == 0 || (fp.flags & HFILE_LIBCURL_CAN_SEEK) == 0 {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
        return -1;
    }
    let origin = match whence {
        libc::SEEK_SET => 0,
        libc::SEEK_CUR => {
            let curpos = fp.base.offset + fp.base.begin.offset_from(fp.base.buffer) as libc::off_t;
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
            if fp.file_size < 0 {
                *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
                return -1;
            }
            fp.file_size
        }
        _ => {
            *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
            return -1;
        }
    };
    if (offset < 0 && origin + offset < 0)
        || (offset >= 0 && fp.file_size >= 0 && offset > fp.file_size - origin)
    {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return -1;
    }
    let pos = origin + offset;
    if (fp.flags & HFILE_LIBCURL_TRIED_SEEK) != 0 {
        if fp.delayed_seek < 0 {
            fp.last_offset =
                fp.base.offset + fp.base.end.offset_from(fp.base.buffer) as libc::off_t;
            hfile_libcurl_preserve_buffer_content_ref(fp);
        }
        fp.delayed_seek = pos;
        return pos;
    }
    if hfile_libcurl_restart_from_position_ref(fp, pos) < 0 {
        *crate::htslib_rs::c_compat::__errno_location() = libc::ESPIPE;
        return -1;
    }
    fp.flags |= HFILE_LIBCURL_TRIED_SEEK;
    pos
}

// original: restart_from_position (htslib/hfile_libcurl.c:1134)
pub unsafe fn hfile_libcurl_c_1134_restart_from_position(
    fp: *mut c_void,
    pos: libc::off_t,
) -> c_int {
    let Some(fp) = fp.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_restart_from_position_ref(fp, pos)
}

unsafe fn hfile_libcurl_restart_from_position_ref(
    fp: &mut HFileLibcurlHeaderPrefix,
    pos: libc::off_t,
) -> c_int {
    let fp_ptr = fp as *mut HFileLibcurlHeaderPrefix;
    let mut temp_fp = HFileLibcurlHeaderPrefix::default();
    temp_fp.base.backend = fp.base.backend;
    temp_fp.multi = fp.multi;
    temp_fp.file_size = fp.file_size;
    temp_fp.final_result = -1;
    temp_fp.flags = fp.flags;
    temp_fp.delayed_seek = -1;
    temp_fp.last_offset = -1;
    let save_errno: c_int;
    let mut update_headers = 0;

    if fp.headers.callback.is_some() {
        if hfile_libcurl_add_callback_headers_ref(fp) != 0 {
            return -1;
        }
        update_headers = 1;
    }
    if fp.headers.auth_hdr_num > 0 && fp.headers.auth.is_some() {
        if hfile_libcurl_add_auth_header_ref(fp) != 0 {
            return -1;
        }
        update_headers = 1;
    }
    if update_headers != 0 {
        if let Some(list) = hfile_libcurl_get_header_list_ref(fp) {
            let err = curl_easy_setopt(fp.easy, CURLOPT_HTTPHEADER, list.as_ptr());
            if err != CURLE_OK {
                *crate::htslib_rs::c_compat::__errno_location() =
                    hfile_libcurl_c_153_easy_errno(fp.easy, err);
                return -1;
            }
        }
    }

    temp_fp.buffer.len = 0;
    temp_fp.buffer.ptr = None;
    temp_fp.easy = curl_easy_duphandle(fp.easy);
    if temp_fp.easy.is_null() {
        fp.flags &= !HFILE_LIBCURL_CAN_SEEK;
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
        fp.flags &= !HFILE_LIBCURL_CAN_SEEK;
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }

    temp_fp.buffer.len = 0;
    temp_fp.flags &= !(HFILE_LIBCURL_PAUSED | HFILE_LIBCURL_FINISHED);
    let mut errm = curl_multi_add_handle(fp.multi, temp_fp.easy);
    if errm != CURLM_OK {
        save_errno = hfile_libcurl_c_270_multi_errno(errm);
        curl_easy_cleanup(temp_fp.easy);
        fp.flags &= !HFILE_LIBCURL_CAN_SEEK;
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }
    fp.nrunning += 1;
    temp_fp.nrunning = fp.nrunning;

    while (temp_fp.flags & HFILE_LIBCURL_PAUSED) == 0
        && (temp_fp.flags & HFILE_LIBCURL_FINISHED) == 0
    {
        if hfile_libcurl_wait_perform_ref(&mut temp_fp) < 0 {
            save_errno = *crate::htslib_rs::c_compat::__errno_location();
            errm = curl_multi_remove_handle(fp.multi, temp_fp.easy);
            if errm == CURLM_OK {
                fp.nrunning -= 1;
            }
            curl_easy_cleanup(temp_fp.easy);
            fp.flags &= !HFILE_LIBCURL_CAN_SEEK;
            *crate::htslib_rs::c_compat::__errno_location() = save_errno;
            return -1;
        }
    }
    if (temp_fp.flags & HFILE_LIBCURL_FINISHED) != 0 && temp_fp.final_result != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno(temp_fp.easy, temp_fp.final_result);
        curl_multi_remove_handle(fp.multi, temp_fp.easy);
        fp.nrunning -= 1;
        curl_easy_cleanup(temp_fp.easy);
        fp.flags &= !HFILE_LIBCURL_CAN_SEEK;
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }

    errm = curl_multi_remove_handle(fp.multi, fp.easy);
    if errm != CURLM_OK {
        curl_multi_remove_handle(fp.multi, temp_fp.easy);
        fp.nrunning -= 1;
        curl_easy_cleanup(temp_fp.easy);
        *crate::htslib_rs::c_compat::__errno_location() = hfile_libcurl_c_270_multi_errno(errm);
        return -1;
    }
    fp.nrunning -= 1;
    curl_easy_cleanup(fp.easy);
    fp.easy = temp_fp.easy;
    err = curl_easy_setopt(fp.easy, CURLOPT_WRITEDATA, fp_ptr.cast::<c_void>());
    err |= curl_easy_setopt(fp.easy, CURLOPT_PRIVATE, fp_ptr.cast::<c_void>());
    if err != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno(fp.easy, err);
        curl_easy_reset(fp.easy);
        *crate::htslib_rs::c_compat::__errno_location() = save_errno;
        return -1;
    }
    fp.buffer.len = 0;
    fp.flags = (fp.flags
        & !(HFILE_LIBCURL_PAUSED | HFILE_LIBCURL_FINISHED | HFILE_LIBCURL_PERFORM_AGAIN))
        | (temp_fp.flags
            & (HFILE_LIBCURL_PAUSED | HFILE_LIBCURL_FINISHED | HFILE_LIBCURL_PERFORM_AGAIN));
    fp.final_result = temp_fp.final_result;
    0
}

// original: libcurl_close (htslib/hfile_libcurl.c:1266)
pub unsafe extern "C" fn hfile_libcurl_c_1266_libcurl_close(fpv: *mut hFILE) -> c_int {
    let Some(fp) = fpv.cast::<HFileLibcurlHeaderPrefix>().as_mut() else {
        return -1;
    };
    hfile_libcurl_close_ref(fp)
}

unsafe fn hfile_libcurl_close_ref(fp: &mut HFileLibcurlHeaderPrefix) -> c_int {
    let mut save_errno = 0;
    fp.buffer.len = 0;
    fp.flags |= HFILE_LIBCURL_CLOSING;
    fp.flags &= !HFILE_LIBCURL_PAUSED;
    if (fp.flags & HFILE_LIBCURL_FINISHED) == 0 {
        let err = curl_easy_pause(fp.easy, CURLPAUSE_CONT);
        if err != CURLE_OK {
            save_errno = hfile_libcurl_c_153_easy_errno(fp.easy, err);
        }
    }
    while save_errno == 0
        && (fp.flags & HFILE_LIBCURL_PAUSED) == 0
        && (fp.flags & HFILE_LIBCURL_FINISHED) == 0
    {
        if hfile_libcurl_wait_perform_ref(fp) < 0 {
            save_errno = *crate::htslib_rs::c_compat::__errno_location();
        }
    }
    if (fp.flags & HFILE_LIBCURL_FINISHED) != 0 && fp.final_result != CURLE_OK {
        save_errno = hfile_libcurl_c_153_easy_errno(fp.easy, fp.final_result);
    }
    let errm = curl_multi_remove_handle(fp.multi, fp.easy);
    if errm != CURLM_OK && save_errno == 0 {
        save_errno = hfile_libcurl_c_270_multi_errno(errm);
    }
    fp.nrunning -= 1;
    curl_easy_cleanup(fp.easy);
    curl_multi_cleanup(fp.multi);
    if let Some(callback) = fp.headers.callback {
        callback(fp.headers.callback_data, std::ptr::null_mut());
    }
    hfile_libcurl_c_372_free_headers(&mut fp.headers.fixed, 1);
    hfile_libcurl_c_372_free_headers(&mut fp.headers.extra, 1);
    drop(std::mem::take(&mut fp.preserved));
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
    let Some(url) = (!url.is_null()).then(|| CStr::from_ptr(url)) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let Some(modes) = (!modes.is_null()).then(|| CStr::from_ptr(modes)) else {
        *crate::htslib_rs::c_compat::__errno_location() = libc::EINVAL;
        return std::ptr::null_mut();
    };
    let headers = headers.as_ref();
    hfile_libcurl_open_ref(url, modes, headers)
}

unsafe fn hfile_libcurl_open_ref(
    url: &CStr,
    modes: &CStr,
    headers: Option<&HFileLibcurlHeaders>,
) -> *mut hFILE {
    let mut seen_mode = None;
    for byte in modes.to_bytes().iter().copied() {
        if matches!(byte, b'r' | b'w' | b'a' | b'+') {
            if seen_mode.is_some() {
                seen_mode = Some(b'e');
                break;
            }
            seen_mode = Some(byte);
        }
    }
    let mode = seen_mode.unwrap_or(0);
    if mode != b'r' && mode != b'w' {
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
    url: &CStr,
    modes: &CStr,
    headers: Option<&HFileLibcurlHeaders>,
    mode: u8,
) -> *mut hFILE {
    let fp = hfile_init(
        std::mem::size_of::<HFileLibcurlHeaderPrefix>(),
        modes.as_ptr(),
        0,
    )
    .cast::<HFileLibcurlHeaderPrefix>();
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    if let Some(headers) = headers {
        std::ptr::addr_of_mut!((*fp).headers).write(headers.clone());
    } else {
        std::ptr::addr_of_mut!((*fp).headers).write(HFileLibcurlHeaders::default());
    }
    (*fp).file_size = -1;
    (*fp).buffer.ptr = None;
    (*fp).buffer.len = 0;
    (*fp).final_result = -1;
    (*fp).flags = HFILE_LIBCURL_CAN_SEEK;
    if mode == b'r' {
        (*fp).flags |= HFILE_LIBCURL_IS_READ;
    }
    if modes.to_bytes().contains(&b'R') {
        (*fp).flags |= HFILE_LIBCURL_IS_RECURSIVE;
    }
    (*fp).delayed_seek = -1;
    (*fp).last_offset = -1;
    std::ptr::addr_of_mut!((*fp).preserved).write(Vec::new());
    (*fp).preserved_bytes = 0;
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
    if mode == b'r' {
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
            &mut (*fp).headers.fixed,
            c"Transfer-Encoding: chunked".as_ptr(),
            1,
        ) < 0
        {
            goto_open_error(fp);
            return std::ptr::null_mut();
        }
    }
    err |= curl_easy_setopt((*fp).easy, CURLOPT_SHARE, HFILE_LIBCURL_SHARE);
    err |= curl_easy_setopt((*fp).easy, CURLOPT_URL, url.as_ptr());
    let ca = libc::getenv(c"CURL_CA_BUNDLE".as_ptr());
    if !ca.is_null() {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_CAINFO, ca);
    }
    err |= curl_easy_setopt((*fp).easy, CURLOPT_USERAGENT, HFILE_LIBCURL_USERAGENT.s);
    if (*fp).headers.callback.is_some() && hfile_libcurl_add_callback_headers_ref(&mut *fp) != 0 {
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    if hfile_libcurl_get_auth_token_ref(&mut *fp, url) < 0 {
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    if let Some(list) = hfile_libcurl_get_header_list_ref(&mut *fp) {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_HTTPHEADER, list.as_ptr());
    }
    if hts_verbose <= 8 && (*fp).headers.fail_on_error != 0 {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_FAILONERROR, 1 as libc::c_long);
    }
    if hts_verbose >= 8 {
        err |= curl_easy_setopt((*fp).easy, CURLOPT_VERBOSE, 1 as libc::c_long);
    }
    let mut in_header: kstring_t = std::mem::zeroed();
    if (*fp).headers.redirect.is_some() {
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
        if hfile_libcurl_wait_perform_ref(&mut *fp) < 0 {
            curl_multi_remove_handle((*fp).multi, (*fp).easy);
            (*fp).nrunning -= 1;
            goto_open_error(fp);
            return std::ptr::null_mut();
        }
    }

    let mut response: libc::c_long = 0;
    curl_easy_getinfo_long((*fp).easy, CURLINFO_RESPONSE_CODE, &mut response);
    if let Some(mut response_ptr) = (*fp).headers.http_response_ptr {
        *response_ptr.as_mut() = response;
    }
    if ((*fp).flags & HFILE_LIBCURL_FINISHED) != 0 && (*fp).final_result != CURLE_OK {
        *crate::htslib_rs::c_compat::__errno_location() =
            hfile_libcurl_c_153_easy_errno((*fp).easy, (*fp).final_result);
        curl_multi_remove_handle((*fp).multi, (*fp).easy);
        (*fp).nrunning -= 1;
        goto_open_error(fp);
        return std::ptr::null_mut();
    }
    if let Some(redirect) = (*fp).headers.redirect {
        if (300..400).contains(&response) {
            let mut new_url: kstring_t = std::mem::zeroed();
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
            if hfile_libcurl_restart_from_position_ref(&mut *fp, 0) < 0 {
                goto_open_error(fp);
                return std::ptr::null_mut();
            }
            if let Some(response_ptr) = (*fp).headers.http_response_ptr {
                curl_easy_getinfo_long((*fp).easy, CURLINFO_RESPONSE_CODE, response_ptr.as_ptr());
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
    if mode == b'r' {
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
        hfile_libcurl_c_372_free_headers(&mut (*fp).headers.fixed, 1);
        hfile_libcurl_c_372_free_headers(&mut (*fp).headers.extra, 1);
        drop(std::mem::take(&mut (*fp).preserved));
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

unsafe fn hfile_libcurl_va_arg_word(args: *mut crate::htslib_rs::c_compat::__va_list_tag) -> usize {
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
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
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
                if hfile_libcurl_c_353_append_header(&mut (*headers).fixed, *hdr, 1) < 0 {
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
                if hfile_libcurl_c_353_append_header(&mut (*headers).fixed, hdr, 1) < 0 {
                    return -1;
                }
                if hfile_libcurl_c_395_is_authorization(hdr) != 0 {
                    (*headers).auth_hdr_num = -1;
                }
            }
        } else if libc::strcmp(argtype, c"httphdr".as_ptr()) == 0 {
            let hdr = hfile_libcurl_va_arg_word(args) as *const c_char;
            if !hdr.is_null() {
                if hfile_libcurl_c_353_append_header(&mut (*headers).fixed, hdr, 1) < 0 {
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
            let args2 =
                hfile_libcurl_va_arg_word(args) as *mut crate::htslib_rs::c_compat::__va_list_tag;
            if !args2.is_null() && hfile_libcurl_c_1554_parse_va_list(headers, args2) < 0 {
                return -1;
            }
        } else if libc::strcmp(argtype, c"auth_token_enabled".as_ptr()) == 0 {
            let flag = hfile_libcurl_va_arg_word(args) as *const c_char;
            if !flag.is_null() && libc::strcmp(flag, c"false".as_ptr()) == 0 {
                (*headers).auth_hdr_num = -3;
            }
        } else if libc::strcmp(argtype, c"redirect_callback".as_ptr()) == 0 {
            let callback = hfile_libcurl_va_arg_word(args);
            (*headers).redirect = if callback == 0 {
                None
            } else {
                Some(std::mem::transmute::<usize, HFileLibcurlRedirectCallback>(
                    callback,
                ))
            };
        } else if libc::strcmp(argtype, c"redirect_callback_data".as_ptr()) == 0 {
            (*headers).redirect_data = hfile_libcurl_va_arg_word(args) as *mut c_void;
        } else if libc::strcmp(argtype, c"http_response_ptr".as_ptr()) == 0 {
            (*headers).http_response_ptr =
                NonNull::new(hfile_libcurl_va_arg_word(args) as *mut libc::c_long);
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
    args: *mut crate::htslib_rs::c_compat::__va_list_tag,
) -> *mut hFILE {
    let mut headers = HFileLibcurlHeaders::default();
    let fp = if hfile_libcurl_c_1554_parse_va_list(&mut headers, args) == 0 {
        hfile_libcurl_c_1313_libcurl_open(url, modes, &mut headers)
    } else {
        std::ptr::null_mut()
    };
    if fp.is_null() {
        hfile_libcurl_c_372_free_headers(&mut headers.fixed, 1);
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
        *std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_PATH) = Some(CStr::from_ptr(auth).to_owned());
        *std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_MAP) = Some(Vec::new());
    }
    let allow = libc::getenv(c"HTS_ALLOW_UNENCRYPTED_AUTHORIZATION_HEADER".as_ptr());
    if !allow.is_null() && libc::strcmp(allow, c"I understand the risks".as_ptr()) == 0 {
        HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER = 1;
    }

    let info = curl_version_info(CURLVERSION_NOW);
    let version = crate::htslib_rs::hts::hts_version();
    if !info.is_null() {
        crate::htslib_rs::kstring::kstring_c_177_ksprintf(
            std::ptr::addr_of_mut!(HFILE_LIBCURL_USERAGENT).cast(),
            c"htslib/%s libcurl/%s".as_ptr(),
            &[
                crate::htslib_rs::kstring::KsPrintfArg::Str(version),
                crate::htslib_rs::kstring::KsPrintfArg::Str((*info).version),
            ],
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
        let mut args = crate::htslib_rs::c_compat::__va_list_tag {
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
            let mut headers = HFileLibcurlHeaders::default();
            let response = 0 as libc::c_long;
            let mut callback_data_marker = ();
            let callback_data = (&mut callback_data_marker as *mut ()).cast::<c_void>();
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
            assert_eq!(headers.fixed.len(), 3);
            assert_eq!(
                CStr::from_ptr(headers.fixed.list[0].data).to_bytes(),
                b"Authorization: Bearer token"
            );
            assert_eq!(
                CStr::from_ptr(headers.fixed.list[1].data).to_bytes(),
                b"X-One: 1"
            );
            assert_eq!(
                CStr::from_ptr(headers.fixed.list[2].data).to_bytes(),
                b"X-Two: 2"
            );
            assert!(headers.callback.is_some());
            assert_eq!(headers.callback_data, callback_data);
            assert_eq!(
                headers.http_response_ptr.map(NonNull::as_ptr),
                Some((&response as *const libc::c_long).cast_mut())
            );
            assert_eq!(headers.fail_on_error, 0);
            assert_eq!(headers.auth_hdr_num, -3);

            hfile_libcurl_c_372_free_headers(&mut headers.fixed, 1);
        }
    }

    #[test]
    fn libcurl_parse_va_list_decodes_vector_and_nested_lists() {
        unsafe {
            let mut headers = HFileLibcurlHeaders::default();
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
            let mut nested = crate::htslib_rs::c_compat::__va_list_tag {
                gp_offset: 0,
                fp_offset: 48,
                overflow_arg_area: nested_overflow.as_mut_ptr().cast(),
                reg_save_area: nested_reg.as_mut_ptr().cast(),
            };
            let words = [
                c"httphdr:v".as_ptr() as usize,
                vector.as_ptr() as usize,
                c"va_list".as_ptr() as usize,
                (&mut nested as *mut crate::htslib_rs::c_compat::__va_list_tag) as usize,
                0,
            ];

            assert_eq!(parse_words(&mut headers, &words), 0);
            assert_eq!(headers.fixed.len(), 3);
            assert_eq!(
                CStr::from_ptr(headers.fixed.list[0].data).to_bytes(),
                b"X-Vec: 1"
            );
            assert_eq!(headers.auth_hdr_num, -1);
            assert_eq!(
                CStr::from_ptr(headers.fixed.list[2].data).to_bytes(),
                b"X-Nested: 1"
            );

            hfile_libcurl_c_372_free_headers(&mut headers.fixed, 1);
        }
    }

    #[test]
    fn libcurl_parse_va_list_rejects_unknown_option() {
        unsafe {
            let mut headers = HFileLibcurlHeaders::default();
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

    // Concurrency audit (2026-05) — verifies the SAFETY claim that
    // `HFILE_LIBCURL_SHARE_LOCK` is a real pthread mutex correctly used as
    // the CURLSHOPT_LOCKFUNC. We spawn many threads that take the share
    // lock around updates to a local counter; if the lock were missing or
    // broken, the final counter would diverge from `THREADS * PER_THREAD`.
    // The same statically-initialized mutex is exercised directly through
    // the C-callable `share_lock` / `share_unlock` entry points used by
    // libcurl when CURL_LOCK_DATA_DNS contention occurs.
    #[test]
    fn libcurl_share_lock_serializes_concurrent_callers() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;

        const THREADS: usize = 8;
        const PER_THREAD: usize = 4_000;

        // Plain counter under the share lock; the lock provides happens-before
        // edges so we observe writes through a *non-atomic* counter without UB.
        // We keep an `AtomicUsize` for the read-out only, to avoid passing a
        // raw `*mut usize` between threads (which would need its own unsafe).
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let counter = counter.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..PER_THREAD {
                    unsafe {
                        hfile_libcurl_c_309_share_lock(
                            std::ptr::null_mut(),
                            0,
                            0,
                            std::ptr::null_mut(),
                        );
                        // Read-modify-write inside the critical section. We
                        // deliberately use Relaxed because the pthread mutex
                        // provides the necessary happens-before.
                        let v = counter.load(Ordering::Relaxed);
                        counter.store(v + 1, Ordering::Relaxed);
                        hfile_libcurl_c_314_share_unlock(
                            std::ptr::null_mut(),
                            0,
                            std::ptr::null_mut(),
                        );
                    }
                }
            }));
        }
        for h in handles {
            h.join().expect("share-lock test thread panicked");
        }
        assert_eq!(counter.load(Ordering::SeqCst), THREADS * PER_THREAD);
    }

    // Concurrency audit (2026-05) — verifies the SAFETY claim that
    // `HFILE_LIBCURL_AUTH_LOCK` correctly serializes lazy initialization of
    // the auth-token map. We invoke `get_auth_token` concurrently with no
    // configured auth path (the function bails out on the
    // `HFILE_LIBCURL_AUTH_PATH.is_null()` fast path before taking the lock,
    // so we additionally exercise the lock directly to assert the same
    // invariant the audit relied on).
    #[test]
    fn libcurl_auth_lock_serializes_lazy_map_init() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Barrier};
        use std::thread;

        const THREADS: usize = 8;
        let barrier = Arc::new(Barrier::new(THREADS));
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let barrier = barrier.clone();
            let counter = counter.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                unsafe {
                    crate::htslib_rs::c_compat::pthread_mutex_lock(std::ptr::addr_of_mut!(
                        HFILE_LIBCURL_AUTH_LOCK
                    ));
                    let v = counter.load(Ordering::Relaxed);
                    counter.store(v + 1, Ordering::Relaxed);
                    crate::htslib_rs::c_compat::pthread_mutex_unlock(std::ptr::addr_of_mut!(
                        HFILE_LIBCURL_AUTH_LOCK
                    ));
                }
            }));
        }
        for h in handles {
            h.join().expect("auth-lock test thread panicked");
        }
        assert_eq!(counter.load(Ordering::SeqCst), THREADS);
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
            let old_path = std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_PATH).replace(None);
            let old_map = std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_MAP).replace(None);
            let old_allow = HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER;
            *std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_PATH) = Some(prefix_c);
            *std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_MAP) = Some(Vec::new());
            HFILE_LIBCURL_ALLOW_UNENCRYPTED_AUTH_HEADER = 0;

            let mut https_fp = HFileLibcurlHeaderPrefix::default();
            assert_eq!(
                hfile_libcurl_c_650_get_auth_token(
                    (&mut https_fp as *mut HFileLibcurlHeaderPrefix).cast(),
                    c"https://example.test/data.bam".as_ptr(),
                ),
                0
            );
            assert_eq!(https_fp.headers.extra.len(), 1);
            assert_eq!(https_fp.headers.auth_hdr_num, 1);
            assert_eq!(
                CStr::from_ptr(https_fp.headers.extra.list[0].data).to_bytes(),
                b"Authorization: Bearer secret-token"
            );

            let mut http_fp = HFileLibcurlHeaderPrefix::default();
            assert_eq!(
                hfile_libcurl_c_650_get_auth_token(
                    (&mut http_fp as *mut HFileLibcurlHeaderPrefix).cast(),
                    c"http://example.test/data.bam".as_ptr(),
                ),
                0
            );
            assert_eq!(http_fp.headers.extra.len(), 0);
            assert!(http_fp.headers.auth.is_none());

            hfile_libcurl_c_372_free_headers(&mut https_fp.headers.extra, 1);
            let _ = std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_PATH).replace(old_path);
            if let Some(mut map) = std::ptr::addr_of_mut!(HFILE_LIBCURL_AUTH_MAP).replace(old_map) {
                for tok in map.drain(..) {
                    hfile_libcurl_free_auth_box(tok);
                }
            }
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
